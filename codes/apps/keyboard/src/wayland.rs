//! Wayland OSK daemon runtime for SpectreOS Keyboard.
//!
//! Speaks standard Wayland protocols implemented by Wisp (and wlroots/Smithay compositors):
//! - `zwlr_layer_shell_v1`: bottom-anchored `Overlay` surface with `exclusive_zone` and
//!   `KeyboardInteractivity::None` so taps never steal focus from the active app.
//! - `zwp_input_method_v2`: double-buffered activate/deactivate, surrounding text, content
//!   purpose, `commit_string`, `delete_surrounding_text`, and `commit(serial)`.
//! - `zwp_virtual_keyboard_v1`: evdev control keys, Ctrl/Alt modifiers, and dynamic XKB
//!   Unicode fallback when shown without an active `zwp_text_input_v3` session.
//! - `wp_fractional_scale_v1` + `wp_viewporter`: crisp pixel-exact `libbhoo::GlesRenderer`
//!   rendering over `wayland-egl` + `libEGL` at any fractional DPI (e.g. `2.7×` on Poco F1).

use crate::{
    InputPurpose, KeyboardEngine, KeyboardOutput, Modifiers, SpecialKey,
    dbus::{OskDbusEvent, OskDbusServer},
    keymap::{
        MOD_ALT, MOD_CTRL, MOD_SHIFT, UNICODE_FALLBACK_KEYCODE, build_xkb_keymap,
        create_keymap_memfd, evdev_for_ascii, evdev_for_special,
    },
    layout::LayoutId,
    touch::erase_last_grapheme_bytes,
};
use bhoo::{DrawCommand, GlesRenderer, Rect};
use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_char, c_int, c_uint, c_void},
    os::fd::AsFd,
    path::{Path, PathBuf},
    ptr,
    time::Instant,
};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
    protocol::{
        wl_compositor::WlCompositor,
        wl_output::{self, WlOutput},
        wl_pointer::{self, WlPointer},
        wl_registry::{self, WlRegistry},
        wl_seat::{self, WlSeat},
        wl_surface::WlSurface,
        wl_touch::{self, WlTouch},
    },
};
use wayland_egl::WlEglSurface;
use wayland_protocols::wp::{
    fractional_scale::v1::client::{
        wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
        wp_fractional_scale_v1::{self, WpFractionalScaleV1},
    },
    viewporter::client::{wp_viewport::WpViewport, wp_viewporter::WpViewporter},
};
use wayland_protocols_misc::{
    zwp_input_method_v2::client::{
        zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
        zwp_input_method_v2::{self, ZwpInputMethodV2},
    },
    zwp_virtual_keyboard_v1::client::{
        zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
        zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
    },
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};

/// Daemon startup options.
#[derive(Clone, Debug)]
pub struct DaemonOptions {
    pub initial_layout: LayoutId,
    pub start_visible: bool,
    pub compat_dbus: bool,
}

// Minimal direct EGL 1.5 FFI (zero build-script overhead, works identically on x86_64 & aarch64).
mod egl {
    use super::*;

    pub type EGLDisplay = *mut c_void;
    pub type EGLContext = *mut c_void;
    pub type EGLSurface = *mut c_void;
    pub type EGLConfig = *mut c_void;
    pub type EGLint = i32;
    pub type EGLenum = u32;
    pub type EGLBoolean = u32;

    pub const EGL_FALSE: EGLBoolean = 0;
    pub const EGL_NONE: EGLint = 0x3038;
    pub const EGL_SURFACE_TYPE: EGLint = 0x3033;
    pub const EGL_WINDOW_BIT: EGLint = 0x0004;
    pub const EGL_RED_SIZE: EGLint = 0x3024;
    pub const EGL_GREEN_SIZE: EGLint = 0x3023;
    pub const EGL_BLUE_SIZE: EGLint = 0x3022;
    pub const EGL_ALPHA_SIZE: EGLint = 0x3021;
    pub const EGL_RENDERABLE_TYPE: EGLint = 0x3040;
    pub const EGL_OPENGL_ES3_BIT: EGLint = 0x0040;
    pub const EGL_OPENGL_ES_API: EGLenum = 0x30A0;
    pub const EGL_CONTEXT_CLIENT_VERSION: EGLint = 0x3098;
    pub const EGL_PLATFORM_WAYLAND_KHR: EGLenum = 0x31D8;

    #[link(name = "EGL")]
    unsafe extern "C" {
        pub fn eglGetDisplay(display_id: *mut c_void) -> EGLDisplay;
        pub fn eglGetPlatformDisplay(
            platform: EGLenum,
            native_display: *mut c_void,
            attrib_list: *const isize,
        ) -> EGLDisplay;
        pub fn eglInitialize(dpy: EGLDisplay, major: *mut EGLint, minor: *mut EGLint) -> EGLBoolean;
        pub fn eglBindAPI(api: EGLenum) -> EGLBoolean;
        pub fn eglChooseConfig(
            dpy: EGLDisplay,
            attrib_list: *const EGLint,
            configs: *mut EGLConfig,
            config_size: EGLint,
            num_config: *mut EGLint,
        ) -> EGLBoolean;
        pub fn eglCreateContext(
            dpy: EGLDisplay,
            config: EGLConfig,
            share_context: EGLContext,
            attrib_list: *const EGLint,
        ) -> EGLContext;
        pub fn eglCreateWindowSurface(
            dpy: EGLDisplay,
            config: EGLConfig,
            win: *mut c_void,
            attrib_list: *const EGLint,
        ) -> EGLSurface;
        pub fn eglDestroySurface(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
        pub fn eglDestroyContext(dpy: EGLDisplay, ctx: EGLContext) -> EGLBoolean;
        pub fn eglTerminate(dpy: EGLDisplay) -> EGLBoolean;
        pub fn eglMakeCurrent(
            dpy: EGLDisplay,
            draw: EGLSurface,
            read: EGLSurface,
            ctx: EGLContext,
        ) -> EGLBoolean;
        pub fn eglSwapBuffers(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
        pub fn eglSwapInterval(dpy: EGLDisplay, interval: EGLint) -> EGLBoolean;
        pub fn eglGetProcAddress(procname: *const c_char) -> *const c_void;
        pub fn eglGetError() -> EGLint;
    }
}

struct EglRuntime {
    display: egl::EGLDisplay,
    config: egl::EGLConfig,
    context: egl::EGLContext,
    renderer: GlesRenderer,
}

impl EglRuntime {
    fn new(wl_display_ptr: *mut c_void, font_path: &Path) -> Result<Self, String> {
        unsafe {
            let mut display = egl::eglGetPlatformDisplay(
                egl::EGL_PLATFORM_WAYLAND_KHR,
                wl_display_ptr,
                ptr::null(),
            );
            if display.is_null() {
                display = egl::eglGetDisplay(wl_display_ptr);
            }
            if display.is_null() {
                return Err("eglGetPlatformDisplay/eglGetDisplay returned null".into());
            }
            let mut major = 0;
            let mut minor = 0;
            if egl::eglInitialize(display, &mut major, &mut minor) == egl::EGL_FALSE {
                return Err(format!("eglInitialize failed: 0x{:x}", egl::eglGetError()));
            }
            if egl::eglBindAPI(egl::EGL_OPENGL_ES_API) == egl::EGL_FALSE {
                return Err(format!("eglBindAPI(GLES) failed: 0x{:x}", egl::eglGetError()));
            }
            let cfg_attribs = [
                egl::EGL_SURFACE_TYPE,
                egl::EGL_WINDOW_BIT,
                egl::EGL_RED_SIZE,
                8,
                egl::EGL_GREEN_SIZE,
                8,
                egl::EGL_BLUE_SIZE,
                8,
                egl::EGL_ALPHA_SIZE,
                8,
                egl::EGL_RENDERABLE_TYPE,
                egl::EGL_OPENGL_ES3_BIT,
                egl::EGL_NONE,
            ];
            let mut config: egl::EGLConfig = ptr::null_mut();
            let mut num_configs = 0;
            if egl::eglChooseConfig(display, cfg_attribs.as_ptr(), &mut config, 1, &mut num_configs)
                == egl::EGL_FALSE
                || num_configs == 0
            {
                return Err(format!("eglChooseConfig failed: 0x{:x}", egl::eglGetError()));
            }
            let ctx_attribs = [egl::EGL_CONTEXT_CLIENT_VERSION, 3, egl::EGL_NONE];
            let context =
                egl::eglCreateContext(display, config, ptr::null_mut(), ctx_attribs.as_ptr());
            if context.is_null() {
                return Err(format!("eglCreateContext(GLES3) failed: 0x{:x}", egl::eglGetError()));
            }
            if egl::eglMakeCurrent(display, ptr::null_mut(), ptr::null_mut(), context)
                == egl::EGL_FALSE
            {
                egl::eglDestroyContext(display, context);
                egl::eglTerminate(display);
                return Err(format!(
                    "eglMakeCurrent(surfaceless) failed: 0x{:x}",
                    egl::eglGetError()
                ));
            }
            let glow_ctx = bhoo::glow::Context::from_loader_function(|name| {
                let c_name = CString::new(name).unwrap();
                egl::eglGetProcAddress(c_name.as_ptr())
            });
            let renderer = GlesRenderer::new(glow_ctx, font_path)
                .map_err(|e| format!("bhoo::GlesRenderer::new failed: {e}"))?;
            Ok(Self {
                display,
                config,
                context,
                renderer,
            })
        }
    }
}

impl Drop for EglRuntime {
    fn drop(&mut self) {
        unsafe {
            egl::eglMakeCurrent(self.display, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
            egl::eglDestroyContext(self.display, self.context);
            egl::eglTerminate(self.display);
        }
    }
}

struct MappedSurface {
    wl_surface: WlSurface,
    layer_surface: ZwlrLayerSurfaceV1,
    viewport: Option<WpViewport>,
    fractional_scale: Option<WpFractionalScaleV1>,
    egl_window: Option<WlEglSurface>,
    egl_surface: egl::EGLSurface,
    configured: bool,
    logical_width: u32,
    logical_height: u32,
    phys_width: u32,
    phys_height: u32,
}

impl MappedSurface {
    fn destroy(self, egl_display: egl::EGLDisplay) {
        unsafe {
            if !self.egl_surface.is_null() {
                egl::eglMakeCurrent(egl_display, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
                egl::eglDestroySurface(egl_display, self.egl_surface);
            }
        }
        drop(self.egl_window);
        if let Some(fs) = self.fractional_scale {
            fs.destroy();
        }
        if let Some(vp) = self.viewport {
            vp.destroy();
        }
        self.layer_surface.destroy();
        self.wl_surface.destroy();
    }
}

struct PendingImState {
    active: bool,
    surrounding_text: String,
    surrounding_cursor: usize,
    surrounding_anchor: usize,
    purpose: InputPurpose,
}

impl Default for PendingImState {
    fn default() -> Self {
        Self {
            active: false,
            surrounding_text: String::new(),
            surrounding_cursor: 0,
            surrounding_anchor: 0,
            purpose: InputPurpose::Normal,
        }
    }
}

struct WaylandAppState {
    started: Instant,
    engine: KeyboardEngine,
    commands: Vec<DrawCommand>,

    // Wayland globals
    compositor: Option<WlCompositor>,
    layer_shell: Option<ZwlrLayerShellV1>,
    seat: Option<WlSeat>,
    touch: Option<WlTouch>,
    pointer: Option<WlPointer>,
    output: Option<WlOutput>,
    viewporter: Option<WpViewporter>,
    fractional_scale_mgr: Option<WpFractionalScaleManagerV1>,
    im_manager: Option<ZwpInputMethodManagerV2>,
    vk_manager: Option<ZwpVirtualKeyboardManagerV1>,

    // Input method & virtual keyboard instances
    input_method: Option<ZwpInputMethodV2>,
    virtual_keyboard: Option<ZwpVirtualKeyboardV1>,
    vk_keymap_loaded: bool,
    vk_unicode_char: Option<char>,

    // Input method state (double-buffered, committed on Done)
    pending_im: PendingImState,
    im_active: bool,
    im_serial: u32,
    surrounding_text: String,
    surrounding_cursor: usize,
    surrounding_anchor: usize,

    // Visibility state
    im_wants_visible: bool,
    dbus_wants_visible: bool,
    surface: Option<MappedSurface>,
    scale: f32,
    output_scale: i32,
    output_width: u32,
    output_height: u32,

    // Pointer / touch tracking
    active_touch_id: Option<i32>,
    touch_positions: HashMap<i32, (f32, f32)>,
    pointer_pos: (f32, f32),
    pointer_down: bool,

    // Pending actions for main loop
    needs_render: bool,
    needs_sync_visibility: bool,
    needs_resize_surface: bool,
    pending_outputs: Vec<KeyboardOutput>,
    should_exit: bool,
}

impl WaylandAppState {
    fn new(initial_layout: LayoutId, start_visible: bool) -> Self {
        Self {
            started: Instant::now(),
            engine: KeyboardEngine::new(initial_layout),
            commands: Vec::with_capacity(256),
            compositor: None,
            layer_shell: None,
            seat: None,
            touch: None,
            pointer: None,
            output: None,
            viewporter: None,
            fractional_scale_mgr: None,
            im_manager: None,
            vk_manager: None,
            input_method: None,
            virtual_keyboard: None,
            vk_keymap_loaded: false,
            vk_unicode_char: None,
            pending_im: PendingImState::default(),
            im_active: false,
            im_serial: 0,
            surrounding_text: String::new(),
            surrounding_cursor: 0,
            surrounding_anchor: 0,
            im_wants_visible: false,
            dbus_wants_visible: start_visible,
            surface: None,
            scale: 1.0,
            output_scale: 1,
            output_width: 400,
            output_height: 832,
            active_touch_id: None,
            touch_positions: HashMap::new(),
            pointer_pos: (0.0, 0.0),
            pointer_down: false,
            needs_render: false,
            needs_sync_visibility: start_visible,
            needs_resize_surface: false,
            pending_outputs: Vec::new(),
            should_exit: false,
        }
    }

    fn now_secs(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    fn now_ms(&self) -> u32 {
        self.started.elapsed().as_millis() as u32
    }

    fn wants_visible(&self) -> bool {
        self.im_wants_visible || self.dbus_wants_visible
    }

    fn current_size(&self) -> (f32, f32) {
        let w = self
            .surface
            .as_ref()
            .filter(|s| s.logical_width > 0)
            .map(|s| s.logical_width as f32)
            .unwrap_or(self.output_width.max(320) as f32);
        let h = self
            .surface
            .as_ref()
            .filter(|s| s.logical_height > 0)
            .map(|s| s.logical_height as f32)
            .unwrap_or_else(|| self.engine.desired_height_for_viewport(w, self.output_height as f32));
        (w, h)
    }

    fn ensure_input_objects(&mut self, qh: &QueueHandle<Self>) {
        if let Some(ref seat) = self.seat {
            if self.input_method.is_none()
                && let Some(ref mgr) = self.im_manager
            {
                self.input_method = Some(mgr.get_input_method(seat, qh, ()));
            }
            if self.virtual_keyboard.is_none()
                && let Some(ref mgr) = self.vk_manager
            {
                let vk = mgr.create_virtual_keyboard(seat, qh, ());
                self.virtual_keyboard = Some(vk);
                self.upload_vk_keymap(None);
            }
        }
    }

    fn upload_vk_keymap(&mut self, unicode: Option<char>) {
        let Some(ref vk) = self.virtual_keyboard else {
            return;
        };
        if self.vk_keymap_loaded && self.vk_unicode_char == unicode {
            return;
        }
        let map = build_xkb_keymap(unicode);
        if let Ok((fd, size)) = create_keymap_memfd(&map) {
            // WL_KEYBOARD_KEYMAP_FORMAT_XKB_V1 = 1
            vk.keymap(1, fd.as_fd(), size);
            self.vk_keymap_loaded = true;
            self.vk_unicode_char = unicode;
        }
    }

    fn send_vk_key(&mut self, evdev: u32, mods: Modifiers, shift_override: bool) {
        self.upload_vk_keymap(self.vk_unicode_char);
        let Some(ref vk) = self.virtual_keyboard else {
            return;
        };
        let mut depressed = 0u32;
        if mods.shift || shift_override {
            depressed |= MOD_SHIFT;
        }
        if mods.ctrl {
            depressed |= MOD_CTRL;
        }
        if mods.alt {
            depressed |= MOD_ALT;
        }
        let t = self.now_ms();
        if depressed != 0 {
            vk.modifiers(depressed, 0, 0, 0);
        }
        vk.key(t, evdev, 1);
        vk.key(t.wrapping_add(1), evdev, 0);
        if depressed != 0 {
            vk.modifiers(0, 0, 0, 0);
        }
    }

    fn emit_outputs(&mut self, outputs: Vec<KeyboardOutput>, dbus: &mut Option<OskDbusServer>) {
        for output in outputs {
            match output {
                KeyboardOutput::CommitText { text, modifiers } => {
                    let has_ctrl_alt = modifiers.ctrl || modifiers.alt;
                    if !has_ctrl_alt
                        && self.im_active
                        && let Some(ref im) = self.input_method
                    {
                        im.commit_string(text.clone());
                        im.commit(self.im_serial);
                    } else if text.chars().count() == 1 {
                        let ch = text.chars().next().unwrap();
                        if let Some((evdev, need_shift)) = evdev_for_ascii(ch) {
                            self.send_vk_key(evdev, modifiers, need_shift);
                        } else {
                            self.upload_vk_keymap(Some(ch));
                            self.send_vk_key(UNICODE_FALLBACK_KEYCODE, modifiers, false);
                        }
                    } else {
                        for ch in text.chars() {
                            if let Some((evdev, need_shift)) = evdev_for_ascii(ch) {
                                self.send_vk_key(evdev, modifiers, need_shift);
                            } else {
                                self.upload_vk_keymap(Some(ch));
                                self.send_vk_key(UNICODE_FALLBACK_KEYCODE, modifiers, false);
                            }
                        }
                    }
                }
                KeyboardOutput::SpecialKey { key, modifiers } => {
                    if key == SpecialKey::Backspace
                        && !modifiers.ctrl
                        && !modifiers.alt
                        && self.im_active
                        && !self.surrounding_text.is_empty()
                        && self.surrounding_cursor > 0
                        && self.surrounding_cursor == self.surrounding_anchor
                    {
                        let (byte_len, cp_count) =
                            erase_last_grapheme_bytes(&self.surrounding_text, self.surrounding_cursor);
                        if cp_count > 1
                            && byte_len > 0
                            && let Some(ref im) = self.input_method
                        {
                            im.delete_surrounding_text(byte_len as u32, 0);
                            im.commit(self.im_serial);
                            continue;
                        }
                    }
                    let evdev = evdev_for_special(key);
                    self.send_vk_key(evdev, modifiers, false);
                }
                KeyboardOutput::HideRequested => {
                    self.im_wants_visible = false;
                    self.dbus_wants_visible = false;
                    self.needs_sync_visibility = true;
                }
                KeyboardOutput::LayoutChanged(layout_id) => {
                    if let Some(srv) = dbus.as_mut() {
                        srv.set_active_layout_state(layout_id.as_str());
                    }
                    self.needs_resize_surface = true;
                    self.needs_render = true;
                }
            }
        }
    }

    fn sync_visibility(
        &mut self,
        qh: &QueueHandle<Self>,
        egl_rt: &EglRuntime,
        dbus: &mut Option<OskDbusServer>,
    ) {
        self.needs_sync_visibility = false;
        let visible = self.wants_visible();
        if visible && self.surface.is_none() {
            let (Some(compositor), Some(layer_shell)) = (&self.compositor, &self.layer_shell) else {
                return;
            };
            let wl_surface = compositor.create_surface(qh, ());
            let layer_surface = layer_shell.get_layer_surface(
                &wl_surface,
                self.output.as_ref(),
                zwlr_layer_shell_v1::Layer::Overlay,
                "org.spectreos.keyboard".to_string(),
                qh,
                (),
            );
            let init_w = self.output_width.max(360);
            let init_h = self
                .engine
                .desired_height_for_viewport(init_w as f32, self.output_height as f32)
                .ceil() as u32;
            layer_surface.set_anchor(
                zwlr_layer_surface_v1::Anchor::Bottom
                    | zwlr_layer_surface_v1::Anchor::Left
                    | zwlr_layer_surface_v1::Anchor::Right,
            );
            layer_surface.set_size(0, init_h);
            layer_surface.set_exclusive_zone(init_h as i32);
            layer_surface.set_keyboard_interactivity(zwlr_layer_surface_v1::KeyboardInteractivity::None);

            let viewport = self.viewporter.as_ref().map(|vp| vp.get_viewport(&wl_surface, qh, ()));
            let fractional_scale = self
                .fractional_scale_mgr
                .as_ref()
                .map(|fs| fs.get_fractional_scale(&wl_surface, qh, ()));

            wl_surface.commit();

            self.surface = Some(MappedSurface {
                wl_surface,
                layer_surface,
                viewport,
                fractional_scale,
                egl_window: None,
                egl_surface: ptr::null_mut(),
                configured: false,
                logical_width: init_w,
                logical_height: init_h,
                phys_width: 0,
                phys_height: 0,
            });
            if let Some(srv) = dbus.as_mut() {
                srv.set_visible_state(true);
            }
        } else if !visible && self.surface.is_some() {
            self.engine.cancel_press();
            self.active_touch_id = None;
            self.touch_positions.clear();
            self.pointer_down = false;
            if let Some(surface) = self.surface.take() {
                surface.destroy(egl_rt.display);
            }
            if let Some(srv) = dbus.as_mut() {
                srv.set_visible_state(false);
            }
        }
    }

    fn apply_resize_if_needed(&mut self) {
        if !self.needs_resize_surface {
            return;
        }
        self.needs_resize_surface = false;
        let Some(ref mut surf) = self.surface else {
            return;
        };
        let w = surf.logical_width.max(320);
        let desired_h = self
            .engine
            .desired_height_for_viewport(w as f32, self.output_height as f32)
            .ceil() as u32;
        if desired_h != surf.logical_height {
            surf.logical_height = desired_h;
            surf.layer_surface.set_size(0, desired_h);
            surf.layer_surface.set_exclusive_zone(desired_h as i32);
            surf.wl_surface.commit();
        }
    }

    fn render_if_ready(&mut self, egl_rt: &mut EglRuntime) {
        if !self.needs_render {
            return;
        }
        let Some(ref mut surf) = self.surface else {
            self.needs_render = false;
            return;
        };
        if !surf.configured || surf.logical_width == 0 || surf.logical_height == 0 {
            return;
        }

        let scale = if self.fractional_scale_mgr.is_some() {
            self.scale.max(0.5)
        } else {
            self.output_scale.max(1) as f32
        };
        let phys_w = ((surf.logical_width as f32) * scale).round().max(1.0) as u32;
        let phys_h = ((surf.logical_height as f32) * scale).round().max(1.0) as u32;

        if surf.egl_window.is_none() {
            let Ok(win) = WlEglSurface::new(surf.wl_surface.id(), phys_w as i32, phys_h as i32)
            else {
                eprintln!("spectre-keyboard: WlEglSurface::new failed");
                return;
            };
            let egl_surf = unsafe {
                egl::eglCreateWindowSurface(
                    egl_rt.display,
                    egl_rt.config,
                    win.ptr() as *mut c_void,
                    ptr::null(),
                )
            };
            if egl_surf.is_null() {
                eprintln!("spectre-keyboard: eglCreateWindowSurface failed: 0x{:x}", unsafe {
                    egl::eglGetError()
                });
                return;
            }
            surf.egl_window = Some(win);
            surf.egl_surface = egl_surf;
            surf.phys_width = phys_w;
            surf.phys_height = phys_h;
        } else if surf.phys_width != phys_w || surf.phys_height != phys_h {
            if let Some(ref win) = surf.egl_window {
                win.resize(phys_w as i32, phys_h as i32, 0, 0);
            }
            surf.phys_width = phys_w;
            surf.phys_height = phys_h;
        }

        if let Some(ref vp) = surf.viewport {
            vp.set_destination(surf.logical_width as i32, surf.logical_height as i32);
        } else {
            surf.wl_surface.set_buffer_scale(self.output_scale.max(1));
        }

        unsafe {
            if egl::eglMakeCurrent(egl_rt.display, surf.egl_surface, surf.egl_surface, egl_rt.context)
                == egl::EGL_FALSE
            {
                eprintln!("spectre-keyboard: eglMakeCurrent failed: 0x{:x}", egl::eglGetError());
                return;
            }
            egl::eglSwapInterval(egl_rt.display, 0);
        }

        self.commands.clear();
        self.commands.push(DrawCommand::Clear(crate::ui::COLOR_BG));
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: surf.logical_width as f32,
            height: surf.logical_height as f32,
        };
        self.engine.render(bounds, &mut self.commands);

        let effective_scale = phys_w as f32 / surf.logical_width as f32;
        if let Err(err) = egl_rt
            .renderer
            .render(&self.commands, [phys_w, phys_h], effective_scale)
        {
            eprintln!("spectre-keyboard: libbhoo render error: {err}");
        }
        unsafe {
            egl::eglSwapBuffers(egl_rt.display, surf.egl_surface);
        }
        self.needs_render = false;
    }
}

impl Dispatch<WlRegistry, ()> for WaylandAppState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind::<WlCompositor, _, _>(name, version.min(5), qh, ()));
                }
                "wl_seat" => {
                    state.seat = Some(registry.bind::<WlSeat, _, _>(name, version.min(7), qh, ()));
                    state.ensure_input_objects(qh);
                }
                "wl_output" => {
                    if state.output.is_none() {
                        state.output = Some(registry.bind::<WlOutput, _, _>(name, version.min(4), qh, ()));
                    }
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell =
                        Some(registry.bind::<ZwlrLayerShellV1, _, _>(name, version.min(4), qh, ()));
                }
                "wp_viewporter" => {
                    state.viewporter =
                        Some(registry.bind::<WpViewporter, _, _>(name, version.min(1), qh, ()));
                }
                "wp_fractional_scale_manager_v1" => {
                    state.fractional_scale_mgr = Some(
                        registry.bind::<WpFractionalScaleManagerV1, _, _>(name, version.min(1), qh, ()),
                    );
                }
                "zwp_input_method_manager_v2" => {
                    state.im_manager = Some(
                        registry.bind::<ZwpInputMethodManagerV2, _, _>(name, version.min(1), qh, ()),
                    );
                    state.ensure_input_objects(qh);
                }
                "zwp_virtual_keyboard_manager_v1" => {
                    state.vk_manager = Some(registry.bind::<ZwpVirtualKeyboardManagerV1, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    ));
                    state.ensure_input_objects(qh);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlCompositor, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &WlCompositor, _: wayland_client::protocol::wl_compositor::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<WlSurface, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &WlSurface, _: wayland_client::protocol::wl_surface::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<ZwlrLayerShellV1, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &ZwlrLayerShellV1, _: zwlr_layer_shell_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<WpViewporter, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &WpViewporter, _: wayland_protocols::wp::viewporter::client::wp_viewporter::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<WpViewport, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &WpViewport, _: wayland_protocols::wp::viewporter::client::wp_viewport::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<WpFractionalScaleManagerV1, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &WpFractionalScaleManagerV1, _: wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<ZwpInputMethodManagerV2, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &ZwpInputMethodManagerV2, _: wayland_protocols_misc::zwp_input_method_v2::client::zwp_input_method_manager_v2::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &ZwpVirtualKeyboardManagerV1, _: wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_manager_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<ZwpVirtualKeyboardV1, ()> for WaylandAppState {
    fn event(_: &mut Self, _: &ZwpVirtualKeyboardV1, _: wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<WlOutput, ()> for WaylandAppState {
    fn event(
        state: &mut Self,
        _: &WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_output::Event::Scale { factor } => {
                state.output_scale = factor.max(1);
                if state.fractional_scale_mgr.is_none() {
                    state.scale = state.output_scale as f32;
                }
            }
            wl_output::Event::Mode { width, height, .. } => {
                if width > 0 {
                    let logical_w = ((width as f32) / state.scale.max(1.0)).round() as u32;
                    state.output_width = logical_w.max(320);
                }
                if height > 0 {
                    let logical_h = ((height as f32) / state.scale.max(1.0)).round() as u32;
                    state.output_height = logical_h.max(240);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlSeat, ()> for WaylandAppState {
    fn event(
        state: &mut Self,
        seat: &WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(caps),
        } = event
        {
            if caps.contains(wl_seat::Capability::Touch) && state.touch.is_none() {
                state.touch = Some(seat.get_touch(qh, ()));
            }
            if caps.contains(wl_seat::Capability::Pointer) && state.pointer.is_none() {
                state.pointer = Some(seat.get_pointer(qh, ()));
            }
        }
    }
}

impl Dispatch<WpFractionalScaleV1, ()> for WaylandAppState {
    fn event(
        state: &mut Self,
        _: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event
            && scale > 0
        {
            state.scale = scale as f32 / 120.0;
            state.needs_render = true;
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WaylandAppState {
    fn event(
        state: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                layer_surface.ack_configure(serial);
                let Some(ref mut surf) = state.surface else {
                    return;
                };
                if width > 0 {
                    surf.logical_width = width;
                    state.output_width = width;
                }
                let desired_h = state
                    .engine
                    .desired_height_for_viewport(surf.logical_width as f32, state.output_height as f32)
                    .ceil() as u32;
                if height == 0 || height != desired_h {
                    surf.logical_height = desired_h;
                    layer_surface.set_size(0, desired_h);
                    layer_surface.set_exclusive_zone(desired_h as i32);
                } else {
                    surf.logical_height = height;
                }
                surf.configured = true;
                state.needs_render = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                state.im_wants_visible = false;
                state.dbus_wants_visible = false;
                state.needs_sync_visibility = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for WaylandAppState {
    fn event(
        state: &mut Self,
        _: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_v2::Event::Activate => {
                state.pending_im.active = true;
            }
            zwp_input_method_v2::Event::Deactivate => {
                state.pending_im.active = false;
            }
            zwp_input_method_v2::Event::SurroundingText {
                text,
                cursor,
                anchor,
            } => {
                state.pending_im.surrounding_text = text;
                state.pending_im.surrounding_cursor = cursor as usize;
                state.pending_im.surrounding_anchor = anchor as usize;
            }
            zwp_input_method_v2::Event::ContentType { purpose, .. } => {
                state.pending_im.purpose = InputPurpose::from_wayland_purpose(purpose.into());
            }
            zwp_input_method_v2::Event::Done => {
                state.im_serial = state.im_serial.wrapping_add(1);
                let was_active = state.im_active;
                state.im_active = state.pending_im.active;
                state.surrounding_text = state.pending_im.surrounding_text.clone();
                state.surrounding_cursor = state.pending_im.surrounding_cursor;
                state.surrounding_anchor = state.pending_im.surrounding_anchor;

                if state.im_active {
                    let prefix_src = state
                        .surrounding_text
                        .get(..state.surrounding_cursor.min(state.surrounding_text.len()));
                    if let Some(ev) = state
                        .engine
                        .set_input_context(prefix_src, state.pending_im.purpose)
                    {
                        state.pending_outputs.push(ev);
                    }
                    if !was_active {
                        state.im_wants_visible = true;
                        state.needs_sync_visibility = true;
                    }
                    state.needs_render = true;
                } else {
                    state.im_wants_visible = false;
                    if was_active {
                        state.needs_sync_visibility = true;
                    }
                }
            }
            zwp_input_method_v2::Event::Unavailable => {
                eprintln!("spectre-keyboard: zwp_input_method_v2 seat already owned by another OSK");
                state.input_method = None;
                state.im_active = false;
            }
            _ => {}
        }
    }
}

impl Dispatch<WlTouch, ()> for WaylandAppState {
    fn event(
        state: &mut Self,
        _: &WlTouch,
        event: wl_touch::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let (width, height) = state.current_size();
        let now = state.now_secs();
        match event {
            wl_touch::Event::Down { id, x, y, .. } => {
                let (fx, fy) = (x as f32, y as f32);
                state.touch_positions.insert(id, (fx, fy));
                if let Some(prev_id) = state.active_touch_id.take() {
                    // Rollover typing: commit previous finger immediately when next finger lands!
                    if let Some((px, py)) = state.touch_positions.remove(&prev_id) {
                        let out = state.engine.pointer_up(px, py, width, height, now);
                        state.pending_outputs.extend(out);
                    }
                }
                state.active_touch_id = Some(id);
                let out = state.engine.pointer_down(fx, fy, width, height, now);
                state.pending_outputs.extend(out);
                state.needs_render = true;
            }
            wl_touch::Event::Motion { id, x, y, .. } => {
                let (fx, fy) = (x as f32, y as f32);
                state.touch_positions.insert(id, (fx, fy));
                if state.active_touch_id == Some(id) {
                    let out = state.engine.pointer_move(fx, fy, width, height, now);
                    state.pending_outputs.extend(out);
                    state.needs_render = true;
                }
            }
            wl_touch::Event::Up { id, .. } => {
                let pos = state.touch_positions.remove(&id);
                if state.active_touch_id == Some(id) {
                    state.active_touch_id = None;
                    if let Some((fx, fy)) = pos {
                        let out = state.engine.pointer_up(fx, fy, width, height, now);
                        state.pending_outputs.extend(out);
                    } else {
                        state.engine.cancel_press();
                    }
                    state.needs_render = true;
                }
            }
            wl_touch::Event::Cancel => {
                state.active_touch_id = None;
                state.touch_positions.clear();
                state.engine.cancel_press();
                state.needs_render = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<WlPointer, ()> for WaylandAppState {
    fn event(
        state: &mut Self,
        _: &WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let (width, height) = state.current_size();
        let now = state.now_secs();
        match event {
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                state.pointer_pos = (surface_x as f32, surface_y as f32);
                if state.pointer_down {
                    let out = state
                        .engine
                        .pointer_move(state.pointer_pos.0, state.pointer_pos.1, width, height, now);
                    state.pending_outputs.extend(out);
                    state.needs_render = true;
                }
            }
            wl_pointer::Event::Button {
                button,
                state: WEnum::Value(btn_state),
                ..
            } if button == 0x110 => {
                // BTN_LEFT
                match btn_state {
                    wl_pointer::ButtonState::Pressed => {
                        state.pointer_down = true;
                        let out = state.engine.pointer_down(
                            state.pointer_pos.0,
                            state.pointer_pos.1,
                            width,
                            height,
                            now,
                        );
                        state.pending_outputs.extend(out);
                        state.needs_render = true;
                    }
                    wl_pointer::ButtonState::Released => {
                        if state.pointer_down {
                            state.pointer_down = false;
                            let out = state.engine.pointer_up(
                                state.pointer_pos.0,
                                state.pointer_pos.1,
                                width,
                                height,
                                now,
                            );
                            state.pending_outputs.extend(out);
                            state.needs_render = true;
                        }
                    }
                    _ => {}
                }
            }
            wl_pointer::Event::Leave { .. } => {
                if state.pointer_down {
                    state.pointer_down = false;
                    state.engine.cancel_press();
                    state.needs_render = true;
                }
            }
            _ => {}
        }
    }
}

/// Discover a suitable UI font (Roboto / Adwaita / DejaVu / Liberation / Noto) matching `libbhoo`.
pub fn find_ui_font() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("BHOO_FONT") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let mut dirs: Vec<PathBuf> = vec![
        PathBuf::from("/run/current-system/sw/share/fonts/truetype"),
        PathBuf::from("/run/current-system/sw/share/fonts/Adwaita"),
        PathBuf::from("/run/current-system/sw/share/fonts/noto"),
        PathBuf::from("/run/current-system/sw/share/fonts"),
        PathBuf::from("/usr/share/fonts/truetype/roboto"),
        PathBuf::from("/usr/share/fonts/roboto"),
        PathBuf::from("/usr/share/fonts/Adwaita"),
        PathBuf::from("/usr/share/fonts/truetype/dejavu"),
        PathBuf::from("/usr/share/fonts/TTF"),
        PathBuf::from("/usr/share/fonts/truetype/liberation"),
        PathBuf::from("/usr/share/fonts/noto"),
        PathBuf::from("/usr/share/fonts/truetype/noto"),
    ];
    for cfg in ["/etc/fonts/conf.d/00-nixos-cache.conf", "/etc/fonts/fonts.conf"] {
        if let Ok(text) = std::fs::read_to_string(cfg) {
            let mut rest = text.as_str();
            while let Some(start) = rest.find("<dir>") {
                let after = &rest[start + 5..];
                let Some(end) = after.find("</dir>") else {
                    break;
                };
                let raw = after[..end].trim();
                if raw.starts_with('/') {
                    let root = PathBuf::from(raw);
                    dirs.push(root.clone());
                    let sf = root.join("share/fonts");
                    if sf.is_dir() {
                        dirs.push(sf.clone());
                        if let Ok(entries) = std::fs::read_dir(&sf) {
                            for e in entries.flatten() {
                                if e.path().is_dir() {
                                    dirs.push(e.path());
                                }
                            }
                        }
                    }
                }
                rest = &after[end + 6..];
            }
        }
    }
    for name in [
        "Roboto-Regular.ttf",
        "RobotoStatic-Regular.ttf",
        "AdwaitaSans-Regular.ttf",
        "DejaVuSans.ttf",
        "LiberationSans-Regular.ttf",
        "NotoSans-Regular.ttf",
    ] {
        for dir in &dirs {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Run the standalone SpectreOS Wayland + Freedesktop D-Bus on-screen keyboard daemon.
pub fn run_daemon(options: DaemonOptions) -> Result<(), String> {
    let conn = Connection::connect_to_env()
        .map_err(|e| format!("Failed to connect to Wayland display: {e}"))?;
    let mut event_queue = conn.new_event_queue::<WaylandAppState>();
    let qh = event_queue.handle();

    let display = conn.display();
    let _registry = display.get_registry(&qh, ());

    let mut state = WaylandAppState::new(options.initial_layout, options.start_visible);
    event_queue
        .roundtrip(&mut state)
        .map_err(|e| format!("Wayland initial roundtrip failed: {e}"))?;
    // Second roundtrip to receive wl_seat capabilities, wl_output mode/scale, and IM events.
    event_queue
        .roundtrip(&mut state)
        .map_err(|e| format!("Wayland secondary roundtrip failed: {e}"))?;

    if state.compositor.is_none() || state.layer_shell.is_none() {
        return Err(
            "Compositor does not advertise required wl_compositor and zwlr_layer_shell_v1 globals"
                .into(),
        );
    }

    let font_path = find_ui_font()
        .ok_or_else(|| "No UI TrueType font found (set BHOO_FONT or install Roboto/DejaVu)".to_string())?;
    let wl_display_ptr = conn.backend().display_ptr() as *mut c_void;
    let mut egl_rt = EglRuntime::new(wl_display_ptr, &font_path)?;

    let mut dbus = match OskDbusServer::start(
        options.start_visible,
        options.initial_layout.as_str(),
        options.compat_dbus,
    ) {
        Ok(srv) => Some(srv),
        Err(err) => {
            eprintln!("spectre-keyboard: warning: D-Bus unavailable ({err}); continuing in Wayland-only mode");
            None
        }
    };

    eprintln!(
        "spectre-keyboard: ready (layout={}, scale={:.2}, im={}, vk={})",
        state.engine.layout_id.as_str(),
        state.scale,
        state.input_method.is_some(),
        state.virtual_keyboard.is_some()
    );

    while !state.should_exit {
        // 1. Drain D-Bus events.
        if let Some(ref mut srv) = dbus {
            for ev in srv.process() {
                match ev {
                    OskDbusEvent::SetVisible(vis) => {
                        state.dbus_wants_visible = vis;
                        if !vis {
                            state.im_wants_visible = false;
                        }
                        state.needs_sync_visibility = true;
                    }
                    OskDbusEvent::SetLayout(name) => {
                        if let Some(id) = LayoutId::parse(&name) {
                            let ev = state.engine.set_layout(id);
                            state.pending_outputs.push(ev);
                        }
                    }
                }
            }
        }

        // 2. Advance long-press / auto-repeat timers.
        let now = state.now_secs();
        let (width, height) = state.current_size();
        let had_popover = state.engine.popover.is_some();
        let timer_out = state.engine.tick(now, width, height);
        if !timer_out.is_empty() || (!had_popover && state.engine.popover.is_some()) {
            state.pending_outputs.extend(timer_out);
            state.needs_render = true;
        }

        // 3. Emit queued text/key/layout/hide outputs.
        if !state.pending_outputs.is_empty() {
            let outputs = std::mem::take(&mut state.pending_outputs);
            state.emit_outputs(outputs, &mut dbus);
        }

        // 4. Sync layer-shell visibility, resize, and render via libbhoo::GlesRenderer.
        if state.needs_sync_visibility {
            state.sync_visibility(&qh, &egl_rt, &mut dbus);
        }
        if state.needs_resize_surface {
            state.apply_resize_if_needed();
        }
        state.render_if_ready(&mut egl_rt);

        // 5. Flush Wayland requests and sleep in poll() until Wayland, D-Bus, or timer wakes us.
        let _ = conn.flush();
        event_queue
            .dispatch_pending(&mut state)
            .map_err(|e| format!("Wayland dispatch error: {e}"))?;
        if state.needs_render || state.needs_sync_visibility || !state.pending_outputs.is_empty() {
            continue;
        }

        let read_guard = event_queue.prepare_read();
        let wl_fd = read_guard
            .as_ref()
            .map(|g| std::os::fd::AsRawFd::as_raw_fd(&g.connection_fd()));

        let timeout_ms: c_int = match state.engine.next_timer_deadline() {
            Some(deadline) => {
                let rem = ((deadline - state.now_secs()) * 1000.0).ceil();
                (rem as c_int).clamp(1, 500)
            }
            None => -1,
        };

        let mut fds = [
            libc::pollfd {
                fd: wl_fd.unwrap_or(-1),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: dbus.as_ref().map(|d| d.fd()).unwrap_or(-1),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let nfds: libc::nfds_t = if fds[1].fd >= 0 { 2 } else { 1 };
        let poll_rc = unsafe { libc::poll(fds.as_mut_ptr(), nfds, timeout_ms) };
        if poll_rc < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                drop(read_guard);
                continue;
            }
            drop(read_guard);
            return Err(format!("poll failed: {err}"));
        }
        if let Some(guard) = read_guard {
            if (fds[0].revents & libc::POLLIN) != 0 {
                if let Err(e) = guard.read() {
                    return Err(format!("Wayland connection closed: {e}"));
                }
            } else {
                drop(guard);
            }
        }
        if (fds[0].revents & (libc::POLLHUP | libc::POLLERR)) != 0 {
            return Err("Wayland display disconnected".into());
        }
        event_queue
            .dispatch_pending(&mut state)
            .map_err(|e| format!("Wayland dispatch error: {e}"))?;
    }

    if let Some(surface) = state.surface.take() {
        surface.destroy(egl_rt.display);
    }
    let _ = CStr::from_bytes_with_nul(b"\0");
    let _ = c_uint::default();
    Ok(())
}
