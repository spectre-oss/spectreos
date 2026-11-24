//! Native Linux window runner: Wayland only, EGL and OpenGL ES 3.0.
use crate::{
    DrawCommand, Frame, Input, Key as UiKey, KeyInput, MotionPreference, PointerKind, Rect, Ui,
    platform::gles::Renderer,
};
use glutin::{
    config::{Api, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version},
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::{
    collections::VecDeque,
    error::Error,
    ffi::CString,
    num::NonZeroU32,
    path::PathBuf,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    platform::wayland::{EventLoopBuilderExtWayland, WindowAttributesExtWayland},
    window::{Window, WindowId},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct WindowOptions {
    pub title: String,
    /// Optional Wayland `app_id` (e.g. `"org.spectreos.Settings"`). Defaults to `"org.spectreos.bhoo"`.
    pub app_id: Option<String>,
    /// Initial content size in logical pixels.
    pub size: [u32; 2],
    /// Whether the window should request client/server window decorations (titlebar/borders).
    /// Set `false` for mobile/fullscreen Wayland apps inside Wisp.
    pub decorations: bool,
    /// TTF/OTF path. Otherwise tries BHOO_FONT, then common Linux/NixOS font paths.
    pub font: Option<PathBuf>,
    /// Optional PPM capture of the frame presented with FrameAction::Exit.
    pub capture: Option<PathBuf>,
    pub motion: MotionPreference,
    pub ripple_style: crate::RippleStyle,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Bhoo".into(),
            app_id: None,
            size: [420, 360],
            decorations: true,
            font: None,
            capture: None,
            motion: MotionPreference::Full,
            ripple_style: crate::RippleStyle::default(),
        }
    }
}

/// Returned after building a frame. Normal apps should use Wait.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FrameAction {
    #[default]
    Wait,
    /// Request another frame (e.g. a running animation).
    Redraw,
    /// Present this frame, then close the window.
    Exit,
}

/// Runs on the main thread. Requires a Wayland compositor and EGL/GLES 3.0.
/// Desktop OpenGL and X11 are deliberately not used as fallback paths.
/// Escape dismisses active transient UI/capture before closing the window.
/// Widgets support keyboard focus/activation and primary touch input.
pub fn run(options: WindowOptions, build: impl FnMut(&mut Frame<'_>) -> FrameAction) -> Result<()> {
    if options.size.contains(&0) {
        return Err("Window dimensions must be nonzero".into());
    }
    let event_loop = EventLoop::builder().with_wayland().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let motion = options.motion;
    let ripple_style = options.ripple_style;
    let mut app = App {
        options,
        build,
        ui: Ui {
            motion_preference: motion,
            ripple_style,
            ..Ui::default()
        },
        commands: Vec::new(),
        graphics: None,
        input: PendingInput::default(),
        error: None,
        exit_requested: false,
        started: Instant::now(),
        shift: false,
        shortcut_modifier: false,
        occluded: false,
    };
    event_loop.run_app(&mut app)?;
    // Ensure GL resources are released while their context is still current.
    app.graphics.take();
    match app.error {
        Some(error) => Err(error.into()),
        None => Ok(()),
    }
}

// Drop order is intentional: renderer while context is current, then surface,
// context, and finally the native window used by EGL.
struct Graphics {
    renderer: Renderer,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    window: Window,
}

impl Graphics {
    fn new(event_loop: &ActiveEventLoop, options: &WindowOptions) -> Result<Self> {
        let font = resolve_font(options.font.as_ref())?;
        let app_id = options.app_id.as_deref().unwrap_or("org.spectreos.bhoo");
        let attributes = Window::default_attributes()
            .with_title(&options.title)
            .with_name(app_id, &options.title)
            .with_decorations(options.decorations)
            .with_inner_size(LogicalSize::new(options.size[0], options.size[1]));
        let template = ConfigTemplateBuilder::new()
            .with_api(Api::GLES3)
            .with_alpha_size(8)
            .with_depth_size(0)
            .with_stencil_size(0);
        let (window, config) = DisplayBuilder::new()
            .with_window_attributes(Some(attributes))
            .build(event_loop, template, |configs| {
                configs
                    .min_by_key(|config| config.num_samples())
                    .expect("EGL returned no GLES configs")
            })?;
        let window = window.ok_or("Wayland window creation returned no window")?;
        let handle = window.window_handle()?.as_raw();
        let display = config.display();
        let attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(Some(Version::new(3, 0))))
            .build(Some(handle));
        let size = window.inner_size();
        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            handle,
            nonzero(size.width),
            nonzero(size.height),
        );
        // SAFETY: the Wayland window outlives its EGL surface and context.
        let (surface, context) = unsafe {
            let context = display.create_context(&config, &attributes)?;
            let surface = display.create_window_surface(&config, &surface_attributes)?;
            let context = context.make_current(&surface)?;
            (surface, context)
        };
        surface.set_swap_interval(&context, SwapInterval::Wait(nonzero(1)))?;
        // SAFETY: functions are loaded from the display of the current GLES context.
        let gl = unsafe {
            glow::Context::from_loader_function(|name| {
                display.get_proc_address(&CString::new(name).expect("GL symbol contains NUL"))
            })
        };
        let renderer = Renderer::new(gl, &font)?;
        Ok(Self {
            renderer,
            surface,
            context,
            window,
        })
    }
}

fn nonzero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).unwrap()
}

fn resolve_font(explicit: Option<&PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.clone());
    }
    if let Some(path) = std::env::var_os("BHOO_FONT") {
        return Ok(path.into());
    }
    super::gles::fonts::find_default_primary_font()
        .ok_or_else(|| "No default font found. Set BHOO_FONT to a TTF/OTF file.".into())
}

#[derive(Default)]
struct PendingInput {
    kind: PointerKind,
    touch: Option<u64>,
    pointer: Option<[f32; 2]>,
    events: VecDeque<Input>,
}

impl PendingInput {
    fn push(&mut self, pressed: bool, released: bool, cancelled: bool) {
        let event = Input {
            pointer: self.pointer,
            pointer_kind: self.kind,
            pressed,
            released,
            cancelled,
            ..Input::default()
        };
        // Coalesce motion, but preserve press/release ordering even between redraws.
        if !pressed
            && !released
            && !cancelled
            && self.events.back().is_some_and(|e| {
                !e.pressed
                    && !e.released
                    && !e.cancelled
                    && e.key.is_none()
                    && e.scroll == [0.0; 2]
                    && !e.blurred
            })
        {
            *self.events.back_mut().unwrap() = event;
        } else {
            self.events.push_back(event);
        }
    }
    fn next(&mut self) -> Input {
        self.events.pop_front().unwrap_or(Input {
            pointer: self.pointer,
            pointer_kind: self.kind,
            ..Input::default()
        })
    }
    fn cancel(&mut self) {
        self.touch = None;
        self.events.clear();
        self.pointer = None;
        self.push(false, false, true);
    }
}

struct App<F> {
    options: WindowOptions,
    build: F,
    ui: Ui,
    commands: Vec<DrawCommand>,
    graphics: Option<Graphics>,
    input: PendingInput,
    error: Option<String>,
    exit_requested: bool,
    started: Instant,
    shift: bool,
    shortcut_modifier: bool,
    occluded: bool,
}

impl<F: FnMut(&mut Frame<'_>) -> FrameAction> App<F> {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl std::fmt::Display) {
        self.error = Some(error.to_string());
        event_loop.exit();
    }

    fn draw(&mut self) -> Result<()> {
        if self.occluded {
            return Ok(());
        }
        let Some(graphics) = self.graphics.as_mut() else {
            return Ok(());
        };
        let size = graphics.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }
        let scale = graphics.window.scale_factor() as f32;
        let logical = size.to_logical::<f32>(scale as f64);
        let mut action = FrameAction::Wait;
        let mut input = self.input.next();
        input.time = self.started.elapsed().as_secs_f64();
        self.ui.viewport = Some(Rect {
            x: 0.0,
            y: 0.0,
            width: logical.width,
            height: logical.height,
        });
        self.ui.frame_into(
            input,
            Rect {
                x: 24.0,
                y: 24.0,
                width: (logical.width - 48.0).max(0.0),
                height: (logical.height - 48.0).max(0.0),
            },
            12.0,
            &mut self.commands,
            |frame| action = (self.build)(frame),
        );
        graphics
            .renderer
            .render(&self.commands, [size.width, size.height], scale)?;
        if action == FrameAction::Exit
            && let Some(path) = self.options.capture.take()
        {
            graphics
                .renderer
                .capture(&path, [size.width, size.height])?;
        }
        graphics.window.pre_present_notify();
        graphics.surface.swap_buffers(&graphics.context)?;
        if action == FrameAction::Exit {
            self.exit_requested = true;
        }
        if action == FrameAction::Redraw || self.ui.needs_redraw() || !self.input.events.is_empty()
        {
            graphics.window.request_redraw();
        }
        Ok(())
    }
}

impl<F: FnMut(&mut Frame<'_>) -> FrameAction> ApplicationHandler for App<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_some() {
            return;
        }
        match Graphics::new(event_loop, &self.options) {
            Ok(graphics) => {
                self.ui.text_metrics = graphics.renderer.text_metrics();
                graphics.window.request_redraw();
                self.graphics = Some(graphics);
            }
            Err(error) => self.fail(event_loop, error),
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.graphics.take();
        self.input.cancel();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(deadline) = self.ui.next_wakeup() {
            let when = self.started + Duration::from_secs_f64(deadline.max(0.0));
            if Instant::now() >= when {
                if let Some(graphics) = &self.graphics {
                    graphics.window.request_redraw();
                }
                event_loop.set_control_flow(ControlFlow::Wait);
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(when));
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(graphics) = self.graphics.as_ref() else {
            return;
        };
        if graphics.window.id() != id {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                let key = match event.logical_key {
                    Key::Named(NamedKey::Tab) => Some(UiKey::Tab),
                    Key::Named(NamedKey::Enter) => Some(UiKey::Enter),
                    Key::Named(NamedKey::Space) => Some(UiKey::Space),
                    Key::Named(NamedKey::ArrowLeft) => Some(UiKey::Left),
                    Key::Named(NamedKey::ArrowRight) => Some(UiKey::Right),
                    Key::Named(NamedKey::ArrowUp) => Some(UiKey::Up),
                    Key::Named(NamedKey::ArrowDown) => Some(UiKey::Down),
                    Key::Named(NamedKey::Home) => Some(UiKey::Home),
                    Key::Named(NamedKey::End) => Some(UiKey::End),
                    Key::Named(NamedKey::PageUp) => Some(UiKey::PageUp),
                    Key::Named(NamedKey::PageDown) => Some(UiKey::PageDown),
                    Key::Named(NamedKey::Escape) => Some(UiKey::Escape),
                    Key::Named(NamedKey::ContextMenu) => Some(UiKey::Menu),
                    Key::Named(NamedKey::F10) if self.shift => Some(UiKey::Menu),
                    Key::Character(ref value) if !self.shortcut_modifier => {
                        let mut chars = value.chars();
                        chars
                            .next()
                            .filter(|c| !c.is_control())
                            .filter(|_| chars.next().is_none())
                            .map(UiKey::Character)
                    }
                    _ => None,
                };
                if let Some(key) = key {
                    if key == UiKey::Escape
                        && event.state == ElementState::Pressed
                        && !self.ui.has_escape_target()
                    {
                        event_loop.exit();
                    } else {
                        self.input.events.push_back(Input {
                            pointer: self.input.pointer,
                            key: Some(KeyInput {
                                key,
                                pressed: event.state == ElementState::Pressed,
                                repeat: event.repeat,
                                shift: self.shift,
                            }),
                            ..Input::default()
                        });
                        graphics.window.request_redraw();
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                self.shift = state.shift_key();
                self.shortcut_modifier =
                    state.control_key() || state.alt_key() || state.super_key();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(x, y) => [-x * 40.0, -y * 40.0],
                    MouseScrollDelta::PixelDelta(p) => {
                        let p = p.to_logical::<f32>(graphics.window.scale_factor());
                        [-p.x, -p.y]
                    }
                };
                self.input.events.push_back(Input {
                    pointer: self.input.pointer,
                    scroll,
                    ..Input::default()
                });
                graphics.window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    graphics.surface.resize(
                        &graphics.context,
                        nonzero(size.width),
                        nonzero(size.height),
                    );
                    graphics.window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.input.cancel();
                graphics.window.request_redraw();
            }
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                if occluded {
                    self.ui.pause_animations();
                    self.input.cancel();
                    event_loop.set_control_flow(ControlFlow::Wait);
                } else {
                    graphics.window.request_redraw();
                }
            }
            WindowEvent::Touch(touch) => {
                let owned = self.input.touch == Some(touch.id);
                if touch.phase == TouchPhase::Started && self.input.touch.is_none() {
                    self.input.touch = Some(touch.id);
                }
                if owned || self.input.touch == Some(touch.id) {
                    let p = touch
                        .location
                        .to_logical::<f32>(graphics.window.scale_factor());
                    self.input.pointer = Some([p.x, p.y]);
                    self.input.kind = PointerKind::Touch;
                    match touch.phase {
                        TouchPhase::Started => self.input.push(true, false, false),
                        TouchPhase::Moved => self.input.push(false, false, false),
                        TouchPhase::Ended => {
                            self.input.push(false, true, false);
                            self.input.touch = None;
                            self.input.pointer = None;
                        }
                        TouchPhase::Cancelled => {
                            self.input.cancel();
                            self.input.touch = None;
                        }
                    }
                    graphics.window.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } if self.input.touch.is_none() => {
                self.input.kind = PointerKind::Mouse;
                let p = position.to_logical::<f32>(graphics.window.scale_factor());
                self.input.pointer = Some([p.x, p.y]);
                self.input.push(false, false, false);
                graphics.window.request_redraw();
            }
            WindowEvent::Focused(false) => {
                self.input.cancel();
                if let Some(input) = self.input.events.back_mut() {
                    input.blurred = true;
                }
                self.shift = false;
                graphics.window.request_redraw();
            }
            WindowEvent::CursorLeft { .. } if self.input.touch.is_none() => {
                self.input.cancel();
                graphics.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } if self.input.touch.is_none() => {
                self.input.events.push_back(Input {
                    pointer: self.input.pointer,
                    key: Some(KeyInput {
                        key: UiKey::Menu,
                        pressed: true,
                        repeat: false,
                        shift: self.shift,
                    }),
                    ..Input::default()
                });
                graphics.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } if self.input.touch.is_none() => {
                self.input.kind = PointerKind::Mouse;
                self.input.push(
                    state == ElementState::Pressed,
                    state == ElementState::Released,
                    false,
                );
                graphics.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.draw() {
                    self.fail(event_loop, error);
                }
                if self.exit_requested {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_keeps_fast_click_edges_in_order() {
        let mut input = PendingInput {
            pointer: Some([10.0, 10.0]),
            ..Default::default()
        };
        input.push(true, false, false);
        input.push(false, true, false);
        assert!(input.next().pressed);
        assert!(input.next().released);
        assert!(!input.next().released);
    }
    #[test]
    fn focus_loss_discards_pending_clicks() {
        let mut input = PendingInput::default();
        input.push(true, false, false);
        input.cancel();
        assert!(input.next().cancelled);
        assert!(input.events.is_empty());
    }
    #[test]
    fn pointer_motion_coalesces() {
        let mut input = PendingInput::default();
        for i in 0..100 {
            input.pointer = Some([i as f32, 0.0]);
            input.push(false, false, false);
        }
        assert_eq!(input.events.len(), 1);
        assert_eq!(input.next().pointer, Some([99.0, 0.0]));
    }
}
