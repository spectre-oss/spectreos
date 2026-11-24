use crate::{
    Options,
    input::{Input, Pointer},
    network::{self, Command, Shared, Worker},
    video::{Layout, TOOLBAR},
};
use font8x8::UnicodeFonts;
use serde_json::{Value, json};
use std::{
    num::NonZeroU32,
    rc::Rc,
    sync::{Arc, Mutex},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

pub fn run(options: Options) -> Result<(), String> {
    let event_loop = EventLoop::<()>::with_user_event()
        .build()
        .map_err(|e| e.to_string())?;
    let proxy = event_loop.create_proxy();
    let state = Arc::new(Mutex::new(Shared::default()));
    let worker = network::start(
        options.endpoint.clone(),
        state.clone(),
        Arc::new(move || {
            let _ = proxy.send_event(());
        }),
    );
    let mut app = Viewer {
        options,
        state,
        worker,
        window: None,
        context: None,
        surface: None,
        input: Input::default(),
        cursor: [0.0; 2],
        layout: Layout::default(),
        error: None,
    };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    if let Some(error) = app.error {
        return Err(error);
    }
    Ok(())
}
struct Viewer {
    options: Options,
    state: Arc<Mutex<Shared>>,
    worker: Worker,
    window: Option<Rc<Window>>,
    context: Option<softbuffer::Context<Rc<Window>>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    input: Input,
    cursor: [f64; 2],
    layout: Layout,
    error: Option<String>,
}
impl Viewer {
    fn send(&self, value: Option<Value>) {
        if let Some(value) = value {
            self.worker.send(Command::Input(value));
        }
    }
    fn release(&mut self) {
        for value in self.input.release() {
            self.worker.send(Command::Input(value));
        }
    }
    fn logical(&self) -> [f64; 2] {
        self.state.lock().unwrap().logical
    }
    fn connected(&self) -> bool {
        self.state.lock().unwrap().connected
    }
    fn quit(&mut self, event_loop: &ActiveEventLoop) {
        self.release();
        self.worker.stop();
        event_loop.exit();
    }
    fn button(&mut self, index: usize, event_loop: &ActiveEventLoop) {
        if index == 5 {
            self.quit(event_loop);
            return;
        }
        if !self.connected() {
            return;
        }
        let command = match index {
            0 => json!({"type":"wake"}),
            1 => json!({"type":"sleep"}),
            2 => json!({"type":"action","name":"back"}),
            3 => json!({"type":"action","name":"home"}),
            4 => json!({"type":"action","name":"overview"}),
            _ => return,
        };
        self.worker.send(Command::Input(command));
    }
    fn draw(&mut self) -> Result<(), String> {
        let Some(window) = &self.window else {
            return Ok(());
        };
        let size = window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return Ok(());
        };
        let (frame, message, frames, connected) = {
            let state = self.state.lock().unwrap();
            (
                state.frame.clone(),
                state.message.clone(),
                state.frames,
                state.connected,
            )
        };
        window.set_title(&format!(
            "Wisp remote_control · {} · {frames} frames{}",
            message,
            if self.options.endpoint.attached() {
                " · attached"
            } else {
                ""
            }
        ));
        let surface = self.surface.as_mut().ok_or("missing window surface")?;
        surface.resize(width, height).map_err(|e| e.to_string())?;
        let mut buffer = surface.buffer_mut().map_err(|e| e.to_string())?;
        buffer.fill(0x141820);
        if let Some(frame) = frame {
            self.layout = Layout::fit([size.width, size.height], [frame.width, frame.height]);
            for y in 0..self.layout.height {
                let sy = y as u64 * frame.height as u64 / self.layout.height as u64;
                let dest = ((self.layout.y + y) * size.width + self.layout.x) as usize;
                for x in 0..self.layout.width {
                    let sx = x as u64 * frame.width as u64 / self.layout.width as u64;
                    buffer[dest + x as usize] =
                        frame.pixels[(sy * frame.width as u64 + sx) as usize];
                }
            }
        } else {
            self.layout = Layout::default();
            text(
                &mut buffer,
                size.width,
                size.height,
                12,
                60,
                &message,
                0xffffff,
            );
        }
        let labels = [
            "WAKE",
            "SLEEP",
            "BACK",
            "HOME",
            "RECENTS",
            if self.options.endpoint.attached() {
                "DETACH"
            } else {
                "STOP"
            },
        ];
        for (index, label) in labels.iter().enumerate() {
            let left = index as u32 * size.width / 6;
            let right = (index as u32 + 1) * size.width / 6;
            for y in 0..TOOLBAR.min(size.height) {
                for x in left..right {
                    buffer[(y * size.width + x) as usize] = if x == left {
                        0x141820
                    } else if index == 5 {
                        0x813038
                    } else {
                        0x293342
                    };
                }
            }
            let x = left + (right - left).saturating_sub(label.len() as u32 * 8) / 2;
            text(
                &mut buffer,
                size.width,
                size.height,
                x,
                16,
                label,
                if connected || index == 5 {
                    0xffffff
                } else {
                    0x88909a
                },
            );
        }
        if !connected && frames > 0 {
            let y = TOOLBAR + 6;
            text(
                &mut buffer,
                size.width,
                size.height,
                8,
                y,
                "DISCONNECTED - RECONNECT EXPLICITLY",
                0xff6464,
            );
        }
        buffer.present().map_err(|e| e.to_string())
    }
}
fn text(pixels: &mut [u32], width: u32, height: u32, x: u32, y: u32, label: &str, color: u32) {
    for (index, ch) in label.chars().enumerate() {
        if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    let px = x + index as u32 * 8 + col;
                    let py = y + row as u32;
                    if px < width && py < height && bits & (1 << col) != 0 {
                        pixels[(py * width + px) as usize] = color;
                    }
                }
            }
        }
    }
}
impl ApplicationHandler<()> for Viewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let result = (|| -> Result<(), String> {
            let window = Rc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Wisp remote_control")
                            .with_inner_size(LogicalSize::new(440.0, 900.0))
                            .with_min_inner_size(LogicalSize::new(360.0, 400.0)),
                    )
                    .map_err(|e| e.to_string())?,
            );
            let context = softbuffer::Context::new(window.clone()).map_err(|e| e.to_string())?;
            let surface =
                softbuffer::Surface::new(&context, window.clone()).map_err(|e| e.to_string())?;
            self.context = Some(context);
            self.surface = Some(surface);
            self.window = Some(window.clone());
            window.request_redraw();
            Ok(())
        })();
        if let Err(error) = result {
            self.error = Some(error);
            self.quit(event_loop);
        }
    }
    fn user_event(&mut self, event_loop: &ActiveEventLoop, _: ()) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        let state = self.state.lock().unwrap();
        let done = self.options.exit_after.is_some_and(|n| state.frames >= n);
        let failed = self.options.exit_after.is_some() && state.finished;
        let error = state.message.clone();
        drop(state);
        if failed {
            self.error = Some(error);
        }
        if done || failed {
            self.quit(event_loop);
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.quit(event_loop),
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.draw() {
                    self.error = Some(error);
                    self.quit(event_loop);
                }
            }
            WindowEvent::Focused(false) => self.release(),
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                self.cursor = [x, y];
                if self.connected() {
                    if let Some(point) = self.layout.point(self.cursor, self.logical(), true) {
                        for pointer in [Pointer::Mouse(0), Pointer::Mouse(1)] {
                            self.send(self.input.motion(pointer, point));
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pointer = match button {
                    MouseButton::Left => Pointer::Mouse(0),
                    MouseButton::Right => Pointer::Mouse(1),
                    _ => return,
                };
                if state == ElementState::Released {
                    let command = self.input.up(pointer);
                    self.send(command);
                } else if self.cursor[1] >= 0.0 && self.cursor[1] < TOOLBAR as f64 {
                    if button == MouseButton::Left {
                        if let Some(window) = &self.window {
                            let width = window.inner_size().width.max(1);
                            self.button(
                                ((self.cursor[0].max(0.0) as u32 * 6 / width).min(5)) as usize,
                                event_loop,
                            );
                        }
                    }
                } else if self.connected() {
                    if let Some(point) = self.layout.point(self.cursor, self.logical(), false) {
                        let command = self.input.down(pointer, point);
                        self.send(command);
                    }
                }
            }
            WindowEvent::Touch(touch) if self.connected() => {
                let pointer = Pointer::Touch(touch.id);
                let position = [touch.location.x, touch.location.y];
                let command = match touch.phase {
                    TouchPhase::Started => self
                        .layout
                        .point(position, self.logical(), false)
                        .and_then(|p| self.input.down(pointer, p)),
                    TouchPhase::Moved => self
                        .layout
                        .point(position, self.logical(), true)
                        .and_then(|p| self.input.motion(pointer, p)),
                    TouchPhase::Ended => self.input.up(pointer),
                    TouchPhase::Cancelled => {
                        self.release();
                        None
                    }
                };
                self.send(command);
            }
            WindowEvent::KeyboardInput { event, .. } if self.connected() => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let command = self.input.key(code, event.state == ElementState::Pressed);
                    self.send(command);
                }
            }
            _ => {}
        }
    }
    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.worker.stop();
    }
}
