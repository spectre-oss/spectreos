//! Unified DRM/KMS + libinput + Wayland Compositor engine inside Wisp.
//!
//! Directly owns and binds `WispShell` (`LauncherDemo`) to real Linux mobile hardware:
//! - DRM/KMS + GBM + EGL/GLES 3.0 scanout (`libbhoo::GlesRenderer` + Smithay `GlesRenderer`)
//! - Touchscreen + hardware Power (short-press sleep/wake, long-press Power Menu) + Volume rocker
//! - Live hardware bindings directly synced with `shell.sys` & `shell.power_state`:
//!   - Battery percentage & charging state (`/sys/class/power_supply/*`)
//!   - Panel backlight & pre-sleep dim + inactivity auto-lock (`/sys/class/backlight/*`)
//!   - Flashlight / LED control (`/sys/class/leds/*`)
//!   - Wi-Fi / Bluetooth / Airplane Mode (`nmcli` / `rfkill`)
//!   - Audio volume & mute (`wpctl`)
//!   - System poweroff / reboot (`systemctl`)
//! - Real Wayland `xdg-shell` app launching & multitasking (`epiphany`, `nautilus`,
//!   `spectre-camera`/`megapixels`, `gnome-calls`, `kgx`) with live touch forwarding.
use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    ffi::CString,
    os::unix::io::{AsFd, BorrowedFd, OwnedFd, RawFd},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use smithay::{
    backend::{
        allocator::{
            Fourcc,
            dmabuf::Dmabuf,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
        },
        drm::{
            DrmDevice, DrmDeviceFd, DrmEvent, DrmEventMetadata, DrmNode, NodeType,
            compositor::DrmCompositor,
            exporter::gbm::GbmFramebufferExporter,
        },
        egl::{EGLContext, EGLDisplay, context::ContextPriority},
        input::{
            AbsolutePositionEvent, Event as _, InputEvent, KeyState, KeyboardKeyEvent, TouchEvent,
        },
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            Color32F, ImportDma,
            element::{
                AsRenderElements, Element, Id, RenderElement,
                surface::WaylandSurfaceRenderElement,
            },
            gles::{GlesError, GlesFrame, GlesRenderer, Uniform, UniformName, UniformType, element::PixelShaderElement},
            utils::{CommitCounter, OpaqueRegions},
        },
        session::{Event as SessionEvent, Session, libseat::LibSeatSession},
    },
    delegate_compositor, delegate_data_control, delegate_data_device, delegate_dmabuf,
    delegate_fractional_scale, delegate_input_method_manager, delegate_layer_shell,
    delegate_output, delegate_seat, delegate_shm, delegate_text_input_manager, delegate_viewporter,
    delegate_virtual_keyboard_manager, delegate_xdg_shell,
    desktop::{
        LayerSurface as DesktopLayerSurface, PopupKind, PopupManager, Space, Window,
        WindowSurfaceType, find_popup_root_surface, get_popup_toplevel_coords,
        layer_map_for_output,
    },
    input::{
        Seat, SeatHandler, SeatState,
        keyboard::FilterResult,
        touch::{DownEvent, MotionEvent, UpEvent},
    },
    output::{Mode as OutputMode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            EventLoop, Interest, LoopSignal, Mode, PostAction,
            generic::Generic,
            timer::{TimeoutAction, Timer},
        },
        drm::control::{Device as ControlDevice, connector, crtc},
        input::Libinput,
        rustix::fs::OFlags,
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_protocols_wlr::foreign_toplevel::v1::server::zwlr_foreign_toplevel_manager_v1::{
            self, ZwlrForeignToplevelManagerV1,
        },
        wayland_server::{
            Client, DataInit, Dispatch, Display, DisplayHandle, GlobalDispatch, New, Resource,
            backend::{
                ClientData, ClientId, DisconnectReason, GlobalHandler, GlobalId,
                Handle as WaylandBackendHandle, ObjectData, ObjectId, protocol as wl_proto,
            },
            protocol::{wl_buffer, wl_output::WlOutput, wl_seat, wl_surface::WlSurface},
        },
    },
    utils::{
        Buffer as BufferCoords, DeviceFd, Logical, Physical, Point, Rectangle, SERIAL_COUNTER,
        Scale, Size, Transform,
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState, with_states},
        dmabuf::{
            DmabufFeedbackBuilder, DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier,
        },
        fractional_scale::{
            FractionalScaleHandler, FractionalScaleManagerState, with_fractional_scale,
        },
        input_method::{
            InputMethodHandler, InputMethodManagerState, PopupSurface as InputMethodPopupSurface,
        },
        output::{OutputHandler, OutputManagerState},
        selection::{
            SelectionHandler, SelectionTarget,
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler, set_data_device_selection,
            },
            wlr_data_control::{DataControlHandler, DataControlState},
        },
        shell::{
            wlr_layer::{
                Layer as WlrLayer, LayerSurface as WlrLayerSurface, WlrLayerShellHandler,
                WlrLayerShellState,
            },
            xdg::{
                PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
                XdgToplevelSurfaceData,
            },
        },
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
        text_input::TextInputManagerState,
        viewporter::ViewporterState,
        virtual_keyboard::VirtualKeyboardManagerState,
    },
};

use crate::{
    AppId, AppTransition, HostCommand, NoopEmbeddedApps, PowerState, ShellMode, WispShell,
    notifications::{NotificationDaemon, NotificationEvent},
};

/// Non-owning `AsFd` wrapper around `sd-bus`'s poll fd (closed by `NotificationDaemon::drop`).
struct BorrowedBusFd(RawFd);

impl AsFd for BorrowedBusFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.0) }
    }
}

const TOP_BAR_H: f32 = 31.0;
const EDGE_BACK_STRIP: f64 = 22.0;

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

#[derive(Default)]
struct PendingInput {
    kind: bhoo::PointerKind,
    touch: Option<i32>,
    pointer: Option<[f32; 2]>,
    events: VecDeque<bhoo::Input>,
}

impl PendingInput {
    fn push(&mut self, pressed: bool, released: bool, cancelled: bool, time: f64) {
        let event = bhoo::Input {
            pointer: self.pointer,
            pointer_kind: self.kind,
            pressed,
            released,
            cancelled,
            time,
            ..bhoo::Input::default()
        };
        if !pressed
            && !released
            && !cancelled
            && self
                .events
                .back()
                .is_some_and(|e| !e.pressed && !e.released && !e.cancelled && e.key.is_none())
        {
            *self.events.back_mut().unwrap() = event;
        } else {
            self.events.push_back(event);
        }
    }

    fn push_key(&mut self, key: bhoo::Key, time: f64) {
        self.events.push_back(bhoo::Input {
            key: Some(bhoo::KeyInput {
                key,
                shift: false,
                pressed: true,
                repeat: false,
            }),
            time,
            ..bhoo::Input::default()
        });
    }

    fn pop_or_idle(&mut self, fallback_time: f64) -> bhoo::Input {
        let inp = self.events.pop_front().unwrap_or(bhoo::Input {
            pointer: self.pointer,
            pointer_kind: self.kind,
            time: fallback_time,
            ..bhoo::Input::default()
        });
        if (inp.released || inp.cancelled) && inp.pointer_kind == bhoo::PointerKind::Touch {
            self.pointer = None;
        }
        inp
    }
}

use crate::services::{HapticEffect, HardwareEvent, SystemServices};

fn spawn_detached(bin: &Path, args: &[&str]) {
    let _ = Command::new(bin)
        .args(args)
        .env_remove("LD_LIBRARY_PATH")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

struct ManagedWaylandApp {
    app_id: Option<AppId>,
    desktop_id: Option<String>,
    pid: Option<u32>,
    window: Window,
    has_buffer: bool,
}

/// Prefer the most recently mapped matching toplevel that has committed a real buffer,
/// falling back to the most recently mapped matching toplevel before first commit.
/// This prevents multi-toplevel apps (such as SDL2/Shiraz recreating a GLES2 window
/// after an initial bufferless placeholder toplevel) from refocusing a dead window.
fn select_focus_index<T, F>(
    items: &[T],
    has_buffer: impl Fn(&T) -> bool,
    predicate: F,
) -> Option<usize>
where
    F: Fn(&T) -> bool,
{
    items
        .iter()
        .rposition(|item| has_buffer(item) && predicate(item))
        .or_else(|| items.iter().rposition(predicate))
}

#[cfg(test)]
mod focus_tests {
    use super::select_focus_index;

    #[derive(Debug)]
    struct DummyToplevel {
        desktop_id: Option<&'static str>,
        has_buffer: bool,
    }

    #[test]
    fn focus_prefers_buffered_toplevel_over_dead_placeholder() {
        let windows = vec![
            DummyToplevel {
                desktop_id: Some("chat.delta.Shiraz.desktop"),
                has_buffer: false,
            },
            DummyToplevel {
                desktop_id: Some("chat.delta.Shiraz.desktop"),
                has_buffer: true,
            },
        ];
        assert_eq!(
            select_focus_index(
                &windows,
                |w| w.has_buffer,
                |w| w.desktop_id == Some("chat.delta.Shiraz.desktop")
            ),
            Some(1)
        );
    }

    #[test]
    fn focus_prefers_buffered_even_if_unbuffered_was_mapped_later() {
        let windows = vec![
            DummyToplevel {
                desktop_id: Some("chat.delta.Shiraz.desktop"),
                has_buffer: true,
            },
            DummyToplevel {
                desktop_id: Some("chat.delta.Shiraz.desktop"),
                has_buffer: false,
            },
        ];
        assert_eq!(
            select_focus_index(
                &windows,
                |w| w.has_buffer,
                |w| w.desktop_id == Some("chat.delta.Shiraz.desktop")
            ),
            Some(0)
        );
    }

    #[test]
    fn focus_falls_back_to_latest_when_none_buffered_yet() {
        let windows = vec![
            DummyToplevel {
                desktop_id: Some("chat.delta.Shiraz.desktop"),
                has_buffer: false,
            },
            DummyToplevel {
                desktop_id: Some("chat.delta.Shiraz.desktop"),
                has_buffer: false,
            },
        ];
        assert_eq!(
            select_focus_index(
                &windows,
                |w| w.has_buffer,
                |w| w.desktop_id == Some("chat.delta.Shiraz.desktop")
            ),
            Some(1)
        );
    }
}

fn native_candidates(app: AppId) -> &'static [(&'static str, &'static [&'static str])] {
    match app {
        AppId::Browser => &[("epiphany", &[])],
        AppId::Camera => &[("spectre-camera", &[]), ("megapixels", &[])],
        AppId::Phone => &[("gnome-calls", &[])],
        _ => &[],
    }
}

fn find_in_path(bin: &str) -> Option<PathBuf> {
    let search = [
        "/run/current-system/sw/bin",
        "/run/wrappers/bin",
        "/usr/bin",
        "/bin",
    ];
    for dir in search {
        let p = Path::new(dir).join(bin);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BacklightWake {
    LockScreen(Instant),
    Fingerprint(Instant),
}

type WispDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    Option<BacklightWake>,
    DrmDeviceFd,
>;

struct BhooElement {
    id: Id,
    commit: CommitCounter,
    renderer: Rc<RefCell<bhoo::GlesRenderer>>,
    commands: Rc<RefCell<Vec<bhoo::DrawCommand>>>,
    phys_size: [u32; 2],
    scale: f32,
    opaque: bool,
}

impl Element for BhooElement {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit
    }

    fn src(&self) -> Rectangle<f64, BufferCoords> {
        Rectangle::from_size((self.phys_size[0] as f64, self.phys_size[1] as f64).into())
    }

    fn geometry(&self, _scale: Scale<f64>) -> Rectangle<i32, Physical> {
        Rectangle::from_size((self.phys_size[0] as i32, self.phys_size[1] as i32).into())
    }

    fn opaque_regions(&self, _scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        if self.opaque {
            OpaqueRegions::from_slice(&[Rectangle::from_size(
                (self.phys_size[0] as i32, self.phys_size[1] as i32).into(),
            )])
        } else {
            OpaqueRegions::default()
        }
    }
}

/// Real-time hardware VBlank & frame-budget telemetry (`WISP_PERF=1` / `WISP_BENCH=1`).
struct FramePerf {
    enabled: bool,
    window_start: Instant,
    last_vblank_seq: Option<u32>,
    last_vblank_at: Option<Instant>,
    /// True once the previous VBlank already belonged to a continuous animation stream.
    in_continuous_stream: bool,
    /// True if the frame currently in-flight on the CRTC was scheduled directly from the previous VBlank.
    inflight_was_continuous: bool,
    frames: u32,
    continuous_frames: u32,
    dropped_vblanks: u32,
    vblank_sum_ms: f64,
    vblank_max_ms: f64,
    ui_sum_ms: f64,
    ui_max_ms: f64,
    drm_sum_ms: f64,
    drm_max_ms: f64,
    last_quads: u32,
    last_flushes: u32,
    last_layers: u32,
    last_cmds: u32,
}

impl FramePerf {
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            window_start: Instant::now(),
            last_vblank_seq: None,
            last_vblank_at: None,
            in_continuous_stream: false,
            inflight_was_continuous: false,
            frames: 0,
            continuous_frames: 0,
            dropped_vblanks: 0,
            vblank_sum_ms: 0.0,
            vblank_max_ms: 0.0,
            ui_sum_ms: 0.0,
            ui_max_ms: 0.0,
            drm_sum_ms: 0.0,
            drm_max_ms: 0.0,
            last_quads: 0,
            last_flushes: 0,
            last_layers: 0,
            last_cmds: 0,
        }
    }

    fn on_vblank(&mut self, metadata: Option<DrmEventMetadata>, frame_period_ms: f64, gpu_mhz: u32) {
        let now = Instant::now();
        if self.inflight_was_continuous && self.in_continuous_stream {
            if let Some(prev) = self.last_vblank_at {
                let dt_ms = prev.elapsed().as_secs_f64() * 1000.0;
                self.continuous_frames += 1;
                self.vblank_sum_ms += dt_ms;
                if dt_ms > self.vblank_max_ms {
                    self.vblank_max_ms = dt_ms;
                }
                if let Some(meta) = metadata
                    && let Some(prev_seq) = self.last_vblank_seq
                {
                    let diff = meta.sequence.wrapping_sub(prev_seq);
                    if diff > 1 && diff < 60 {
                        self.dropped_vblanks += diff - 1;
                        if self.enabled {
                            tracing::warn!(
                                "VBLANK DROP: seq_diff={diff} dt={dt_ms:.2}ms cmds={} quads={} layers={} gpu={}MHz",
                                self.last_cmds,
                                self.last_quads,
                                self.last_layers,
                                gpu_mhz,
                            );
                        }
                    }
                } else if dt_ms > frame_period_ms * 1.30 {
                    let extra = ((dt_ms / frame_period_ms).round() as u32).saturating_sub(1);
                    self.dropped_vblanks += extra;
                }
            }
        }
        self.in_continuous_stream = self.inflight_was_continuous;
        self.last_vblank_at = Some(now);
        if let Some(meta) = metadata {
            self.last_vblank_seq = Some(meta.sequence);
        }

        if self.enabled && self.frames > 0 && self.window_start.elapsed() >= Duration::from_secs(1) {
            let elapsed = self.window_start.elapsed().as_secs_f64().max(0.001);
            let fps = self.frames as f64 / elapsed;
            let n = self.frames as f64;
            let vb_avg = if self.continuous_frames > 0 {
                self.vblank_sum_ms / self.continuous_frames as f64
            } else {
                frame_period_ms
            };
            let eff_fps = if vb_avg > 0.1 { 1000.0 / vb_avg } else { fps };
            tracing::info!(
                "PERF anim_fps={eff_fps:.2} total_fps={fps:.1} frames={} dropped={} vblank={vb_avg:.2}ms(max {:.2}ms) ui={:.2}ms(max {:.2}ms) gl+drm={:.2}ms(max {:.2}ms) cmds={} quads={} flushes={} layers={} gpu={}MHz",
                self.frames,
                self.dropped_vblanks,
                self.vblank_max_ms,
                self.ui_sum_ms / n,
                self.ui_max_ms,
                self.drm_sum_ms / n,
                self.drm_max_ms,
                self.last_cmds,
                self.last_quads,
                self.last_flushes,
                self.last_layers,
                gpu_mhz,
            );
            self.window_start = now;
            self.frames = 0;
            self.continuous_frames = 0;
            self.dropped_vblanks = 0;
            self.vblank_sum_ms = 0.0;
            self.vblank_max_ms = 0.0;
            self.ui_sum_ms = 0.0;
            self.ui_max_ms = 0.0;
            self.drm_sum_ms = 0.0;
            self.drm_max_ms = 0.0;
        }
    }

    fn record_frame(
        &mut self,
        ui_ms: f64,
        drm_ms: f64,
        cmds: u32,
        stats: bhoo::GlesRenderStats,
        from_vblank: bool,
    ) {
        self.frames += 1;
        self.inflight_was_continuous = from_vblank;
        self.ui_sum_ms += ui_ms;
        if ui_ms > self.ui_max_ms {
            self.ui_max_ms = ui_ms;
        }
        self.drm_sum_ms += drm_ms;
        if drm_ms > self.drm_max_ms {
            self.drm_max_ms = drm_ms;
        }
        self.last_cmds = cmds;
        self.last_quads = stats.quads;
        self.last_flushes = stats.flushes;
        self.last_layers = stats.layers;
    }
}

impl RenderElement<GlesRenderer> for BhooElement {
    fn draw(
        &self,
        _frame: &mut GlesFrame<'_, '_>,
        _src: Rectangle<f64, BufferCoords>,
        _dst: Rectangle<i32, Physical>,
        _damage: &[Rectangle<i32, Physical>],
        _opaque_regions: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        let commands = self.commands.borrow();
        if let Err(err) =
            self.renderer
                .borrow_mut()
                .render(&commands, self.phys_size, self.scale)
        {
            tracing::warn!("bhoo render error: {err}");
        }
        Ok(())
    }
}

smithay::render_elements! {
    WispRenderElement<=GlesRenderer>;
    Surface=WaylandSurfaceRenderElement<GlesRenderer>,
    Bhoo=BhooElement,
    LockFade=PixelShaderElement,
    Memory=smithay::backend::renderer::element::memory::MemoryRenderBufferRenderElement<GlesRenderer>,
}

const LOCK_FADE_DURATION: Duration = Duration::from_micros(206_667);

struct ScreenOffTransition {
    started: Instant,
    duration: Duration,
    reason: String,
    black_queued: bool,
    presented_frames: u32,
}

struct WispCompositorState {
    display_handle: DisplayHandle,
    socket_name: String,
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    layer_shell_state: WlrLayerShellState,
    shm_state: ShmState,
    dmabuf_state: DmabufState,
    #[allow(dead_code)]
    dmabuf_global: DmabufGlobal,
    seat_state: SeatState<Self>,
    data_device_state: DataDeviceState,
    data_control_state: DataControlState,
    #[allow(dead_code)]
    output_manager_state: OutputManagerState,
    #[allow(dead_code)]
    fractional_scale_manager_state: FractionalScaleManagerState,
    #[allow(dead_code)]
    viewporter_state: ViewporterState,
    space: Space<Window>,
    popups: PopupManager,
    seat: Seat<Self>,
    #[allow(dead_code)]
    loop_signal: LoopSignal,

    // Native Wayland apps, popups, OSK layer surfaces & multitasking
    wayland_apps: Vec<ManagedWaylandApp>,
    spawned_children: Vec<std::process::Child>,
    pending_kills: Vec<(Instant, u32, ObjectId)>,
    active_wayland_idx: Option<usize>,
    active_popup: Option<PopupSurface>,
    active_touch_surface: Option<(WlSurface, Point<f64, Logical>, bool)>,
    last_back_sent: Option<Instant>,
    pending_launch: Option<AppId>,
    pending_launch_desktop: Option<String>,
    touch_target_wayland: bool,
    active_wayland_touch_slots: HashSet<i32>,
    shade_capture_attempted: bool,
    osk_bottom_inset: i32,
    warmed_user_services: bool,

    // DRM + Renderer
    renderer: GlesRenderer,
    bhoo_renderer: Rc<RefCell<bhoo::GlesRenderer>>,
    bhoo_id: Id,
    bhoo_commit: CommitCounter,
    previous_frame_is_settings: bool,
    previous_scene_command_count: usize,
    drm_compositor: WispDrmCompositor,
    output: Output,
    phys_size: [u32; 2],
    scale: f32,
    frame_period_s: f64,
    last_anim_time: f64,
    continuous_anim: bool,
    needs_render: bool,
    waiting_for_vblank: bool,
    waiting_for_vblank_since: Option<Instant>,
    last_watchdog_ping: Instant,
    screen_on: bool,
    park_black_frame: bool,
    lock_fade: Option<PixelShaderElement>,
    lock_reveal: Option<(Instant, Duration)>,
    screen_off_transition: Option<ScreenOffTransition>,
    screen_dimmed: bool,
    pending_backlight_on: Option<BacklightWake>,
    dt2w_tap_down: Option<(i32, Instant, [f64; 2])>,
    dt2w_last_tap: Option<(Instant, [f64; 2])>,
    last_activity: Instant,
    power_pressed_at: Option<Instant>,
    power_long_handled: bool,
    hw: SystemServices,
    perf: FramePerf,
    bench_phase: Option<(u32, Instant)>,

    // Directly embedded Wisp System UI & Launcher + Freedesktop D-Bus Notifications daemon
    shell: Box<WispShell<NoopEmbeddedApps>>,
    notif_daemon: Option<NotificationDaemon>,
    last_settings_sync: Instant,
    last_desktop_reload: Instant,
    ui: bhoo::Ui,
    commands: Rc<RefCell<Vec<bhoo::DrawCommand>>>,
    compose_body: String,
    input: PendingInput,
    started: Instant,
    #[cfg(feature = "remote_control")]
    remote_control: Option<crate::remote_control::RemoteControl>,
}

impl BufferHandler for WispCompositorState {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl CompositorHandler for WispCompositorState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        smithay::backend::renderer::utils::on_commit_buffer_handler::<Self>(surface);
        self.popups.commit(surface);
        self.popups.cleanup();

        let surf_id = surface.id();
        let committed_buffer = Self::surface_has_buffer(surface);
        let mut newly_matched_desktop: Option<(usize, String)> = None;
        let mut newly_buffered_idx: Option<usize> = None;
        for (idx, app) in self.wayland_apps.iter_mut().enumerate() {
            app.window.on_commit();
            let is_this_toplevel = app
                .window
                .toplevel()
                .is_some_and(|t| t.wl_surface().id() == surf_id);
            if is_this_toplevel {
                if committed_buffer && !app.has_buffer {
                    newly_buffered_idx = Some(idx);
                }
                app.has_buffer = committed_buffer;
                if app.desktop_id.is_none() {
                    let wl_app_id = with_states(surface, |states| {
                        states
                            .data_map
                            .get::<XdgToplevelSurfaceData>()
                            .and_then(|data| data.lock().ok().and_then(|g| g.app_id.clone()))
                    });
                    if let Some(wl_id) = wl_app_id
                        && let Some(entry) =
                            self.shell.desktop_registry.find_by_wayland_app_id(&wl_id)
                    {
                        app.desktop_id = Some(entry.id.clone());
                        if app.app_id.is_none() && self.active_wayland_idx == Some(idx) {
                            newly_matched_desktop = Some((idx, entry.id.clone()));
                        }
                    }
                }
            }
        }
        if let Some((_, did)) = newly_matched_desktop {
            self.shell.set_active_wayland_desktop(Some(&did));
        }
        if let Some(buf_idx) = newly_buffered_idx
            && let Some(active_idx) = self.active_wayland_idx
            && active_idx != buf_idx
            && let (Some(active), Some(buf)) = (
                self.wayland_apps.get(active_idx),
                self.wayland_apps.get(buf_idx),
            )
            && !active.has_buffer
        {
            let same_app = (active.desktop_id.is_some() && active.desktop_id == buf.desktop_id)
                || (active.app_id.is_some() && active.app_id == buf.app_id)
                || (active.pid.is_some() && active.pid == buf.pid);
            if same_app {
                let promoted_did = buf.desktop_id.clone();
                let promoted_app = buf.app_id;
                self.active_wayland_idx = Some(buf_idx);
                if let Some(a) = promoted_app {
                    self.shell.set_active_wayland_app(Some(a));
                } else if let Some(ref did) = promoted_did {
                    self.shell.set_active_wayland_desktop(Some(did));
                }
                if let Some(kb) = self.seat.get_keyboard() {
                    kb.set_focus(self, Some(surface.clone()), SERIAL_COUNTER.next_serial());
                }
                tracing::info!(
                    "Promoted active Wayland toplevel #{active_idx} -> #{buf_idx} after buffer commit"
                );
            }
        }

        // Re-arrange wlr-layer-shell surfaces (e.g. SpectreOS Keyboard) on commit & adapt app height
        let mut layer_changed = false;
        {
            let mut map = layer_map_for_output(&self.output);
            if let Some(layer) = map
                .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
                .cloned()
            {
                map.arrange();
                layer.layer_surface().send_pending_configure();
                layer_changed = true;
            }
        }
        if layer_changed {
            self.update_osk_inset_and_reconfigure();
        }
        self.needs_render = true;
    }
}

impl ShmHandler for WispCompositorState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl XdgShellHandler for WispCompositorState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let (logical_w, logical_h) = self.app_viewport_size();
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Maximized);
            state.states.set(xdg_toplevel::State::Activated);
            state.size = Some(Size::from((logical_w, logical_h)));
        });
        surface.send_configure();

        let wl_surface = surface.wl_surface().clone();
        let pref_scale = self.scale as f64;
        with_states(&wl_surface, |states| {
            with_fractional_scale(states, |fs| fs.set_preferred_scale(pref_scale));
        });
        let pid = wl_surface
            .client()
            .and_then(|c| c.get_credentials(&self.display_handle).ok())
            .map(|cred| cred.pid as u32);
        let wl_app_id = with_states(&wl_surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|data| data.lock().ok().and_then(|g| g.app_id.clone()))
        });
        let window = Window::new_wayland_window(surface);
        self.space.map_element(window.clone(), (0, TOP_BAR_H as i32), true);

        let mut app_id = self.pending_launch.take();
        let mut desktop_id = self.pending_launch_desktop.take().or_else(|| {
            wl_app_id.as_deref().and_then(|id| {
                self.shell
                    .desktop_registry
                    .find_by_wayland_app_id(id)
                    .map(|a| a.id.clone())
            })
        });
        if app_id.is_none()
            && desktop_id.is_none()
            && let Some(p) = pid
            && let Some(sibling) = self.wayland_apps.iter().rev().find(|e| e.pid == Some(p))
        {
            app_id = sibling.app_id;
            desktop_id = sibling.desktop_id.clone();
        }
        let has_buffer = Self::surface_has_buffer(&wl_surface);
        self.wayland_apps.push(ManagedWaylandApp {
            app_id,
            desktop_id: desktop_id.clone(),
            pid,
            window,
            has_buffer,
        });
        let idx = self.wayland_apps.len() - 1;
        self.active_wayland_idx = Some(idx);
        self.active_popup = None;
        if let Some(a) = app_id {
            self.shell.set_active_wayland_app(Some(a));
        } else if let Some(ref did) = desktop_id {
            self.shell.set_active_wayland_desktop(Some(did));
        }
        if let Some(kb) = self.seat.get_keyboard() {
            kb.set_focus(self, Some(wl_surface), SERIAL_COUNTER.next_serial());
        }
        tracing::info!(
            "Mapped Wayland toplevel #{idx} (app={app_id:?}, desktop={desktop_id:?}, pid={pid:?}, wl_app_id={wl_app_id:?}) {logical_w}x{logical_h}"
        );
        self.needs_render = true;
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let target_id = surface.wl_surface().id();
        self.pending_kills.retain(|(_, _, oid)| *oid != target_id);
        if let Some(pos) = self.wayland_apps.iter().position(|entry| {
            entry
                .window
                .toplevel()
                .is_some_and(|t| t.wl_surface().id() == target_id)
        }) {
            let removed = self.wayland_apps.remove(pos);
            self.space.unmap_elem(&removed.window);
            self.active_popup = None;
            let still_has_app = removed
                .app_id
                .is_some_and(|a| self.wayland_apps.iter().any(|e| e.app_id == Some(a)));
            let still_has_desktop = removed.desktop_id.as_deref().is_some_and(|did| {
                self.wayland_apps
                    .iter()
                    .any(|e| e.desktop_id.as_deref() == Some(did))
            });
            let remove_app = if still_has_app { None } else { removed.app_id };
            let remove_desktop = if still_has_desktop {
                None
            } else {
                removed.desktop_id.as_deref()
            };
            if remove_app.is_some() || remove_desktop.is_some() {
                self.shell.remove_wayland_task(remove_app, remove_desktop);
            }
            if self.active_wayland_idx == Some(pos) {
                let sibling_idx = self.find_wayland_app_for_focus(|e| {
                    (removed.desktop_id.is_some() && e.desktop_id == removed.desktop_id)
                        || (removed.app_id.is_some() && e.app_id == removed.app_id)
                        || (removed.pid.is_some() && e.pid == removed.pid)
                });
                if let Some(next_idx) = sibling_idx {
                    self.active_wayland_idx = Some(next_idx);
                    if let Some(surf) = self.active_wayland_surface()
                        && let Some(kb) = self.seat.get_keyboard()
                    {
                        kb.set_focus(self, Some(surf), SERIAL_COUNTER.next_serial());
                    }
                } else {
                    self.active_wayland_idx = None;
                    if let Some(kb) = self.seat.get_keyboard() {
                        kb.set_focus(self, None, SERIAL_COUNTER.next_serial());
                    }
                    self.shell.go_home_now(self.started.elapsed().as_secs_f64());
                }
            } else if let Some(active) = self.active_wayland_idx
                && active > pos
            {
                self.active_wayland_idx = Some(active - 1);
            }
            tracing::info!(
                "Closed Wayland toplevel (app={:?}, desktop={:?})",
                removed.app_id,
                removed.desktop_id
            );
            self.needs_render = true;
        }
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        let pref_scale = self.scale as f64;
        with_states(surface.wl_surface(), |states| {
            with_fractional_scale(states, |fs| fs.set_preferred_scale(pref_scale));
        });
        self.constrain_popup(&surface, positioner);
        if let Err(err) = self.popups.track_popup(PopupKind::Xdg(surface.clone())) {
            tracing::warn!("Failed to track xdg_popup: {err}");
        }
        if let Err(err) = surface.send_configure() {
            tracing::warn!("Failed to configure xdg_popup: {err}");
        }
        self.needs_render = true;
    }

    fn grab(
        &mut self,
        surface: PopupSurface,
        _seat: wl_seat::WlSeat,
        _serial: smithay::utils::Serial,
    ) {
        self.active_popup = Some(surface);
        self.needs_render = true;
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        self.constrain_popup(&surface, positioner);
        surface.send_repositioned(token);
        if let Err(err) = surface.send_configure() {
            tracing::warn!("Failed to reconfigure repositioned xdg_popup: {err}");
        }
        self.needs_render = true;
    }

    fn popup_destroyed(&mut self, surface: PopupSurface) {
        if self
            .active_popup
            .as_ref()
            .is_some_and(|p| p.wl_surface().id() == surface.wl_surface().id())
        {
            self.active_popup = None;
        }
        self.popups.cleanup();
        self.needs_render = true;
    }
}

impl SeatHandler for WispCompositorState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }
}

impl SelectionHandler for WispCompositorState {
    type SelectionUserData = std::sync::Arc<[u8]>;

    fn send_selection(&mut self, target: SelectionTarget, mime_type: String, fd: OwnedFd,
        _seat: Seat<Self>, data: &Self::SelectionUserData)
    {
        if target == SelectionTarget::Clipboard && mime_type == "text/uri-list" {
            let data = data.clone();
            // A client controls the read end of this pipe, so keep writes off the render thread.
            std::thread::spawn(move || {
                use std::io::Write;
                let _ = std::fs::File::from(fd).write_all(&data);
            });
        }
    }
}

impl DataDeviceHandler for WispCompositorState {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl DataControlHandler for WispCompositorState {
    fn data_control_state(&self) -> &DataControlState {
        &self.data_control_state
    }
}

impl GlobalDispatch<ZwlrForeignToplevelManagerV1, ()> for WispCompositorState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<ZwlrForeignToplevelManagerV1>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        data_init.init(resource, ());
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for WispCompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        resource: &ZwlrForeignToplevelManagerV1,
        request: zwlr_foreign_toplevel_manager_v1::Request,
        _data: &(),
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        if let zwlr_foreign_toplevel_manager_v1::Request::Stop = request {
            resource.finished();
        }
    }
}

// Minimal `phoc-device-state-unstable-v1` (v2) compatibility global for third-party Wayland
// mobile clients that query touch/tablet-mode capabilities (`capabilities = 0x2`, tablet enabled).
static ZPHOC_TABLET_MODE_SWITCH_V1_INTERFACE: wl_proto::Interface = wl_proto::Interface {
    name: "zphoc_tablet_mode_switch_v1",
    version: 1,
    requests: &[wl_proto::MessageDesc {
        name: "destroy",
        signature: &[],
        since: 1,
        is_destructor: true,
        child_interface: None,
        arg_interfaces: &[],
    }],
    events: &[
        wl_proto::MessageDesc {
            name: "disabled",
            signature: &[],
            since: 1,
            is_destructor: false,
            child_interface: None,
            arg_interfaces: &[],
        },
        wl_proto::MessageDesc {
            name: "enabled",
            signature: &[],
            since: 1,
            is_destructor: false,
            child_interface: None,
            arg_interfaces: &[],
        },
    ],
    c_ptr: None,
};

static ZPHOC_LID_SWITCH_V1_INTERFACE: wl_proto::Interface = wl_proto::Interface {
    name: "zphoc_lid_switch_v1",
    version: 1,
    requests: &[wl_proto::MessageDesc {
        name: "destroy",
        signature: &[],
        since: 1,
        is_destructor: true,
        child_interface: None,
        arg_interfaces: &[],
    }],
    events: &[
        wl_proto::MessageDesc {
            name: "opened",
            signature: &[],
            since: 1,
            is_destructor: false,
            child_interface: None,
            arg_interfaces: &[],
        },
        wl_proto::MessageDesc {
            name: "closed",
            signature: &[],
            since: 1,
            is_destructor: false,
            child_interface: None,
            arg_interfaces: &[],
        },
    ],
    c_ptr: None,
};

static ZPHOC_DEVICE_STATE_V1_INTERFACE: wl_proto::Interface = wl_proto::Interface {
    name: "zphoc_device_state_v1",
    version: 2,
    requests: &[
        wl_proto::MessageDesc {
            name: "get_tablet_mode_switch",
            signature: &[wl_proto::ArgumentType::NewId],
            since: 1,
            is_destructor: false,
            child_interface: Some(&ZPHOC_TABLET_MODE_SWITCH_V1_INTERFACE),
            arg_interfaces: &[&ZPHOC_TABLET_MODE_SWITCH_V1_INTERFACE],
        },
        wl_proto::MessageDesc {
            name: "get_lid_switch",
            signature: &[wl_proto::ArgumentType::NewId],
            since: 1,
            is_destructor: false,
            child_interface: Some(&ZPHOC_LID_SWITCH_V1_INTERFACE),
            arg_interfaces: &[&ZPHOC_LID_SWITCH_V1_INTERFACE],
        },
    ],
    events: &[wl_proto::MessageDesc {
        name: "capabilities",
        signature: &[wl_proto::ArgumentType::Uint],
        since: 2,
        is_destructor: false,
        child_interface: None,
        arg_interfaces: &[],
    }],
    c_ptr: None,
};

struct PhocDeviceStateGlobal;
struct PhocDeviceStateObject;
struct PhocSwitchObject;

impl GlobalHandler<WispCompositorState> for PhocDeviceStateGlobal {
    fn bind(
        self: Arc<Self>,
        handle: &WaylandBackendHandle,
        _data: &mut WispCompositorState,
        _client_id: ClientId,
        _global_id: GlobalId,
        object_id: ObjectId,
    ) -> Arc<dyn ObjectData<WispCompositorState>> {
        if handle
            .object_info(object_id.clone())
            .is_ok_and(|info| info.version >= 2)
        {
            // Bit 1 (0x2) = touch; bit 2 (0x4) = hardware keyboard (unset on mobile touchscreen).
            let _ = handle.send_event(wl_proto::Message::<ObjectId, RawFd> {
                sender_id: object_id,
                opcode: 0,
                args: [wl_proto::Argument::Uint(0x2)].into_iter().collect(),
            });
        }
        Arc::new(PhocDeviceStateObject)
    }
}

impl ObjectData<WispCompositorState> for PhocDeviceStateObject {
    fn request(
        self: Arc<Self>,
        handle: &WaylandBackendHandle,
        _data: &mut WispCompositorState,
        _client_id: ClientId,
        msg: wl_proto::Message<ObjectId, OwnedFd>,
    ) -> Option<Arc<dyn ObjectData<WispCompositorState>>> {
        let new_id = msg.args.into_iter().find_map(|arg| match arg {
            wl_proto::Argument::NewId(id) => Some(id),
            _ => None,
        })?;
        match msg.opcode {
            // get_tablet_mode_switch -> send `enabled` (opcode 1)
            0 => {
                let _ = handle.send_event(wl_proto::Message::<ObjectId, RawFd> {
                    sender_id: new_id,
                    opcode: 1,
                    args: Default::default(),
                });
                Some(Arc::new(PhocSwitchObject))
            }
            // get_lid_switch -> send `opened` (opcode 0)
            1 => {
                let _ = handle.send_event(wl_proto::Message::<ObjectId, RawFd> {
                    sender_id: new_id,
                    opcode: 0,
                    args: Default::default(),
                });
                Some(Arc::new(PhocSwitchObject))
            }
            _ => None,
        }
    }

    fn destroyed(
        self: Arc<Self>,
        _handle: &WaylandBackendHandle,
        _data: &mut WispCompositorState,
        _client_id: ClientId,
        _object_id: ObjectId,
    ) {
    }
}

impl ObjectData<WispCompositorState> for PhocSwitchObject {
    fn request(
        self: Arc<Self>,
        _handle: &WaylandBackendHandle,
        _data: &mut WispCompositorState,
        _client_id: ClientId,
        _msg: wl_proto::Message<ObjectId, OwnedFd>,
    ) -> Option<Arc<dyn ObjectData<WispCompositorState>>> {
        None
    }

    fn destroyed(
        self: Arc<Self>,
        _handle: &WaylandBackendHandle,
        _data: &mut WispCompositorState,
        _client_id: ClientId,
        _object_id: ObjectId,
    ) {
    }
}

impl ClientDndGrabHandler for WispCompositorState {}
impl ServerDndGrabHandler for WispCompositorState {}
impl OutputHandler for WispCompositorState {}

impl DmabufHandler for WispCompositorState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        if self.renderer.import_dmabuf(&dmabuf, None).is_ok() {
            let _ = notifier.successful::<WispCompositorState>();
        } else {
            notifier.failed();
        }
    }
}

impl WlrLayerShellHandler for WispCompositorState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        _output: Option<WlOutput>,
        layer: WlrLayer,
        namespace: String,
    ) {
        let pref_scale = self.scale as f64;
        with_states(surface.wl_surface(), |states| {
            with_fractional_scale(states, |fs| fs.set_preferred_scale(pref_scale));
        });
        let desktop_layer = DesktopLayerSurface::new(surface, namespace.clone());
        {
            let mut map = layer_map_for_output(&self.output);
            let _ = map.map_layer(&desktop_layer);
            map.arrange();
            desktop_layer.layer_surface().send_pending_configure();
        }
        self.update_osk_inset_and_reconfigure();
        tracing::info!("Mapped wlr-layer-shell surface namespace='{namespace}' layer={layer:?}");
        self.needs_render = true;
    }

    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        {
            let mut map = layer_map_for_output(&self.output);
            let target = map
                .layers()
                .find(|l| l.layer_surface() == &surface)
                .cloned();
            if let Some(layer) = target {
                map.unmap_layer(&layer);
                map.arrange();
            }
        }
        self.update_osk_inset_and_reconfigure();
        self.needs_render = true;
    }
}

impl InputMethodHandler for WispCompositorState {
    fn new_popup(&mut self, surface: InputMethodPopupSurface) {
        let _ = self.popups.track_popup(PopupKind::InputMethod(surface));
        self.needs_render = true;
    }

    fn dismiss_popup(&mut self, _surface: InputMethodPopupSurface) {
        self.popups.cleanup();
        self.needs_render = true;
    }

    fn popup_repositioned(&mut self, _surface: InputMethodPopupSurface) {
        self.needs_render = true;
    }

    fn parent_geometry(&self, _parent: &WlSurface) -> Rectangle<i32, Logical> {
        let (w, h) = self.app_viewport_size();
        Rectangle::new((0, TOP_BAR_H as i32).into(), (w, h).into())
    }
}

impl FractionalScaleHandler for WispCompositorState {
    fn new_fractional_scale(&mut self, surface: WlSurface) {
        let pref_scale = self.scale as f64;
        with_states(&surface, |states| {
            with_fractional_scale(states, |fs| fs.set_preferred_scale(pref_scale));
        });
    }
}

delegate_compositor!(WispCompositorState);
delegate_shm!(WispCompositorState);
delegate_dmabuf!(WispCompositorState);
delegate_xdg_shell!(WispCompositorState);
delegate_layer_shell!(WispCompositorState);
delegate_seat!(WispCompositorState);
delegate_data_device!(WispCompositorState);
delegate_data_control!(WispCompositorState);
delegate_output!(WispCompositorState);
delegate_fractional_scale!(WispCompositorState);
delegate_viewporter!(WispCompositorState);
delegate_text_input_manager!(WispCompositorState);
delegate_input_method_manager!(WispCompositorState);
delegate_virtual_keyboard_manager!(WispCompositorState);

impl WispCompositorState {
    fn app_viewport_size(&self) -> (i32, i32) {
        let logical_w = (self.phys_size[0] as f32 / self.scale).round() as i32;
        let logical_h = (self.phys_size[1] as f32 / self.scale).round() as i32;
        let bottom_reserve = (self.shell.nav_bar_height() as i32).max(self.osk_bottom_inset);
        let app_h = (logical_h - TOP_BAR_H.round() as i32 - bottom_reserve).max(200);
        (logical_w, app_h)
    }

    fn update_osk_inset_and_reconfigure(&mut self) {
        let logical_h = (self.phys_size[1] as f32 / self.scale).round() as i32;
        let new_inset = {
            let map = layer_map_for_output(&self.output);
            let zone = map.non_exclusive_zone();
            let zone_bottom = zone.loc.y + zone.size.h;
            (logical_h - zone_bottom).max(0)
        };
        if new_inset == self.osk_bottom_inset {
            return;
        }
        self.osk_bottom_inset = new_inset;
        self.reconfigure_wayland_viewport();
    }

    fn reconfigure_wayland_viewport(&mut self) {
        let (w, h) = self.app_viewport_size();
        for entry in &self.wayland_apps {
            if let Some(tl) = entry.window.toplevel() {
                tl.with_pending_state(|state| {
                    state.size = Some(Size::from((w, h)));
                });
                tl.send_configure();
            }
        }
        self.hw
            .shiraz
            .set_wayland_env(&self.socket_name, w.max(120) as u32, h.max(120) as u32, self.scale);
        tracing::info!("Adjusted Wayland app viewport to {w}x{h} (osk_bottom_inset={})", self.osk_bottom_inset);
        self.needs_render = true;
    }

    fn constrain_popup(&self, surface: &PopupSurface, positioner: PositionerState) {
        let (app_w, app_h) = self.app_viewport_size();
        let parent_offset = get_popup_toplevel_coords(&PopupKind::Xdg(surface.clone()));
        let target = Rectangle::new(
            (-parent_offset.x, -parent_offset.y).into(),
            (app_w, app_h).into(),
        );
        let geom = positioner.get_unconstrained_geometry(target);
        surface.with_pending_state(|state| {
            state.geometry = geom;
            state.positioner = positioner;
        });
    }

    fn dismiss_active_popup(&mut self) -> bool {
        let Some(popup) = self.active_popup.take() else {
            return false;
        };
        let kind = PopupKind::Xdg(popup.clone());
        if let Ok(root) = find_popup_root_surface(&kind) {
            let _ = PopupManager::dismiss_popup(&root, &kind);
        }
        popup.send_popup_done();
        self.needs_render = true;
        true
    }

    fn prewarm_user_services(&mut self) {
        if self.warmed_user_services {
            return;
        }
        self.warmed_user_services = true;

        // 1. Launch SpectreOS's default Rust + libbhoo keyboard (`spectre-keyboard`), which speaks
        //    `zwlr_layer_shell_v1`, `zwp_input_method_v2`, `zwp_virtual_keyboard_v1`, and D-Bus
        //    `org.spectreos.OSK0`.
        let mut osk_bin: Option<PathBuf> = std::env::var_os("SPECTRE_KEYBOARD_BIN")
            .map(PathBuf::from)
            .filter(|p| p.is_file());
        if osk_bin.is_none() {
            let mut dropin_dirs = vec![PathBuf::from("/etc/systemd/user/org.spectreos.OSK.service.d")];
            if let Some(home) = std::env::var_os("HOME") {
                dropin_dirs.push(
                    PathBuf::from(&home).join(".config/systemd/user/org.spectreos.OSK.service.d"),
                );
            }
            for dir in dropin_dirs {
                let Ok(entries) = std::fs::read_dir(&dir) else {
                    continue;
                };
                let mut files: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("conf"))
                    .collect();
                files.sort();
                for conf in files {
                    let Ok(text) = std::fs::read_to_string(&conf) else {
                        continue;
                    };
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if let Some(rest) = trimmed.strip_prefix("ExecStart=") {
                            let first = rest.split_whitespace().next().unwrap_or("");
                            if !first.is_empty() && Path::new(first).is_file() {
                                osk_bin = Some(PathBuf::from(first));
                            }
                        }
                    }
                }
            }
        }
        let osk_bin = osk_bin.or_else(|| {
            if let Some(home) = std::env::var_os("HOME") {
                let local = PathBuf::from(home).join("spectre-keyboard");
                if local.is_file() {
                    return Some(local);
                }
            }
            if let Ok(exe) = std::env::current_exe()
                && let Some(dir) = exe.parent()
            {
                let sibling = dir.join("spectre-keyboard");
                if sibling.is_file() {
                    return Some(sibling);
                }
            }
            find_in_path("spectre-keyboard")
        });
        if let Some(bin) = osk_bin {
            let mut cmd = Command::new(&bin);
            cmd.env_remove("RUST_LOG")
                .env("WAYLAND_DISPLAY", &self.socket_name)
                .env("XDG_SESSION_TYPE", "wayland")
                .env("MESA_DISK_CACHE_SINGLE_FILE", "1")
                .env("XDG_CURRENT_DESKTOP", "Wisp:SpectreOS:GNOME")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            if let Some(font_dirs) = self.hw.fonts.bhoo_font_dirs_env() {
                cmd.env("BHOO_FONT_DIRS", font_dirs);
            }
            if std::env::var_os("BHOO_FONT").is_none()
                && let Some(primary) = self.hw.fonts.primary_font_path()
            {
                cmd.env("BHOO_FONT", primary);
            }
            match cmd.spawn() {
                Ok(child) => {
                    let pid = child.id();
                    self.spawned_children.push(child);
                    tracing::info!(
                        "Started SpectreOS Keyboard (spectre-keyboard): {} (pid={})",
                        bin.display(),
                        pid
                    );
                }
                Err(err) => {
                    tracing::warn!("Failed to start SpectreOS Keyboard {}: {err}", bin.display());
                }
            }
        } else {
            tracing::warn!(
                "SpectreOS Keyboard binary (`spectre-keyboard`) not found in SPECTRE_KEYBOARD_BIN, ~/spectre-keyboard, or PATH"
            );
        }

        // 2. Pre-warm Nautilus GApplication service in background (initializes GVFS + dconf +
        //    GTK4/GLES3 shader cache once so the very first Files launch opens in ~150ms).
        if let Some(nautilus) = find_in_path("nautilus") {
            let mut cmd = Command::new(&nautilus);
            cmd.arg("--gapplication-service")
                .env_remove("LD_LIBRARY_PATH")
                .env_remove("BHOO_FONT")
                .env_remove("RUST_LOG")
                .env_remove("GTK_IM_MODULE")
                .env("WAYLAND_DISPLAY", &self.socket_name)
                .env("XDG_SESSION_TYPE", "wayland")
                .env("GDK_BACKEND", "wayland")
                .env("GSK_RENDERER", "gl")
                .env("NO_AT_BRIDGE", "1")
                .env("MESA_DISK_CACHE_SINGLE_FILE", "1")
                .env("XDG_CURRENT_DESKTOP", "Wisp:SpectreOS:GNOME")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            if let Ok(child) = cmd.spawn() {
                let pid = child.id();
                self.spawned_children.push(child);
                tracing::info!("Pre-warmed nautilus --gapplication-service (pid={pid})");
            }
        }
    }

    fn surface_has_buffer(surface: &WlSurface) -> bool {
        smithay::backend::renderer::utils::with_renderer_surface_state(surface, |state| {
            state.buffer().is_some()
        })
        .unwrap_or(false)
    }

    fn find_wayland_app_for_focus<F>(&self, predicate: F) -> Option<usize>
    where
        F: Fn(&ManagedWaylandApp) -> bool,
    {
        select_focus_index(&self.wayland_apps, |a| a.has_buffer, predicate)
    }

    fn active_wayland_surface(&self) -> Option<WlSurface> {
        let idx = self.active_wayland_idx?;
        self.wayland_apps
            .get(idx)?
            .window
            .toplevel()
            .map(|t| t.wl_surface().clone())
    }

    fn begin_screen_off(&mut self, reason: &str) {
        if self.screen_off_transition.is_some() {
            return;
        }
        self.hw.fingerprint.update_listening(
            &self.shell.sys, true, self.shell.is_fingerprint_lab_foreground(),
        );
        let duration = LOCK_FADE_DURATION.mul_f32(self.shell.sys.anim_scale());
        if self.lock_fade.is_none() || duration.is_zero() || self.pending_backlight_on.is_some() {
            self.sleep_and_lock(reason);
            return;
        }
        self.input.events.clear();
        self.handle_touch_cancel();
        // A second power press reverses the reveal at its current radius.
        let elapsed = self.lock_reveal.take().map_or(Duration::ZERO, |(start, span)| {
            duration.mul_f32(1.0 - (start.elapsed().as_secs_f32() / span.as_secs_f32()).min(1.0))
        });
        self.screen_off_transition = Some(ScreenOffTransition {
            started: Instant::now() - elapsed,
            duration,
            reason: reason.into(),
            black_queued: false,
            presented_frames: 0,
        });
        self.needs_render = true;
    }

    fn sleep_and_lock(&mut self, reason: &str) {
        // The LCD can expose the last scanout when its backlight powers up.
        self.screen_off_transition = None;
        self.lock_reveal = None;
        self.hw.set_backlight_off();
        self.pending_backlight_on = None;
        self.screen_dimmed = false;
        self.touch_target_wayland = false;
        self.active_wayland_touch_slots.clear();
        self.active_touch_surface = None;
        self.dt2w_tap_down = None;
        self.continuous_anim = false;
        self.input.touch = None;
        self.input.pointer = None;
        self.dismiss_active_popup();
        if let Some(kb) = self.seat.get_keyboard() {
            kb.set_focus(self, None, SERIAL_COUNTER.next_serial());
        }
        self.shell.lockscreen.lock();
        self.screen_on = false;
        self.park_black_frame = true;
        self.needs_render = true;
        self.hw.release_gpu_boost(true);
        tracing::info!("Screen OFF ({reason})");
    }

    fn wake_to_lockscreen(&mut self) {
        self.screen_off_transition = None;
        self.lock_reveal = None;
        self.shell.lockscreen.lock();
        self.park_black_frame = false;
        self.screen_on = true;
        self.screen_dimmed = false;
        self.dt2w_tap_down = None;
        self.dt2w_last_tap = None;
        self.last_activity = Instant::now();
        self.hw.boost_gpu(Duration::from_millis(800));
        // Defer backlight-ON until VBlank confirms the Lock Screen frame has physically scanned out
        // on the DSI panel (eliminates the 1-frame flash of the previous app!).
        self.pending_backlight_on = Some(BacklightWake::LockScreen(Instant::now()));
        self.drm_compositor.reset_buffers();
        self.needs_render = true;
        self.render_frame();
        tracing::info!("Screen ON (lock screen, VBlank-gated backlight)");
    }

    fn wake_unlocked(&mut self) {
        self.screen_off_transition = None;
        self.lock_reveal = None;
        self.park_black_frame = false;
        self.screen_on = true;
        self.screen_dimmed = false;
        self.dt2w_tap_down = None;
        self.dt2w_last_tap = None;
        self.last_activity = Instant::now();
        self.hw.boost_gpu(Duration::from_millis(900));
        // VBlank-gated backlight wake directly to unlocked Home/app (zero lockscreen flash!)
        self.pending_backlight_on = Some(BacklightWake::Fingerprint(Instant::now()));
        self.drm_compositor.reset_buffers();
        self.needs_render = true;
        self.render_frame();
        tracing::info!("Screen ON (unlocked directly via Touch ID, VBlank-gated backlight)");
    }

    fn spawn_desktop_app(&mut self, desktop: &crate::freedesktop::DesktopApp) -> bool {
        let mut path_env = std::env::var("PATH").unwrap_or_default();
        for extra in [
            "/run/current-system/sw/bin",
            "/run/wrappers/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
        ] {
            if !path_env.split(':').any(|p| p == extra) {
                if !path_env.is_empty() {
                    path_env.push(':');
                }
                path_env.push_str(extra);
            }
        }
        let mut xdg_data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
        for extra in ["/run/current-system/sw/share", "/usr/local/share", "/usr/share"] {
            if !xdg_data_dirs.split(':').any(|p| p == extra) {
                if !xdg_data_dirs.is_empty() {
                    xdg_data_dirs.push(':');
                }
                xdg_data_dirs.push_str(extra);
            }
        }

        let mut cmd = if !desktop.exec_argv.is_empty() {
            if desktop.terminal {
                let term_candidates: &[(&str, &[&str])] = &[
                    ("kgx", &["--"]),
                    ("gnome-terminal", &["--"]),
                    ("foot", &[]),
                    ("alacritty", &["-e"]),
                    ("kitty", &[]),
                    ("xterm", &["-e"]),
                ];
                let Some((term_path, term_prefix)) = term_candidates
                    .iter()
                    .find_map(|&(bin, pref)| find_in_path(bin).map(|p| (p, pref)))
                else {
                    tracing::warn!(
                        "No terminal emulator found for Terminal=true app {}",
                        desktop.id
                    );
                    return false;
                };
                let mut c = Command::new(term_path);
                c.args(term_prefix);
                c.args(&desktop.exec_argv);
                c
            } else {
                let prog = &desktop.exec_argv[0];
                let resolved = if prog.contains('/') {
                    PathBuf::from(prog)
                } else {
                    find_in_path(prog).unwrap_or_else(|| PathBuf::from(prog))
                };
                let mut c = Command::new(resolved);
                if desktop.exec_argv.len() > 1 {
                    c.args(&desktop.exec_argv[1..]);
                }
                c
            }
        } else if desktop.dbus_activatable {
            // Desktop Entry Spec §8: D-Bus Activation via org.freedesktop.Application.Activate
            let Some(gdbus) = find_in_path("gdbus") else {
                tracing::warn!("gdbus not found for DBusActivatable app {}", desktop.id);
                return false;
            };
            let bus_name = desktop.id.strip_suffix(".desktop").unwrap_or(&desktop.id);
            let obj_path = format!("/{}", bus_name.replace('.', "/").replace('-', "_"));
            let mut c = Command::new(gdbus);
            c.args([
                "call",
                "--session",
                "--dest",
                bus_name,
                "--object-path",
                &obj_path,
                "--method",
                "org.freedesktop.Application.Activate",
                "{}",
            ]);
            c
        } else {
            return false;
        };

        if let Some(ref wd) = desktop.working_dir
            && wd.is_dir()
        {
            cmd.current_dir(wd);
        }
        self.hw.boost_gpu(Duration::from_millis(2500));
        cmd.env_remove("LD_LIBRARY_PATH")
            .env_remove("BHOO_FONT")
            .env_remove("RUST_LOG")
            .env_remove("GTK_IM_MODULE")
            .env("WAYLAND_DISPLAY", &self.socket_name)
            .env("XDG_SESSION_TYPE", "wayland")
            .env("GDK_BACKEND", "wayland")
            .env("QT_QPA_PLATFORM", "wayland")
            .env("SDL_VIDEODRIVER", "wayland")
            .env("NO_AT_BRIDGE", "1")
            .env("MESA_DISK_CACHE_SINGLE_FILE", "1")
            .env("XDG_CURRENT_DESKTOP", "Wisp:SpectreOS:GNOME")
            .env("XDG_DATA_DIRS", &xdg_data_dirs)
            .env("GIO_LAUNCHED_DESKTOP_FILE", &desktop.desktop_path)
            .env("PATH", &path_env)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if std::env::var_os("GSK_RENDERER").is_none() {
            // Skip slow radv/Vulkan probing on Adreno 630 and go straight to hardware GLES3 (freedreno).
            cmd.env("GSK_RENDERER", "gl");
        }

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                self.spawned_children.push(child);
                tracing::info!(
                    "Launched Freedesktop app '{}' ({}) -> {:?} (pid={})",
                    desktop.name,
                    desktop.id,
                    desktop.exec_argv,
                    pid
                );
                true
            }
            Err(err) => {
                tracing::warn!(
                    "Failed to launch Freedesktop app '{}' ({}): {err}",
                    desktop.name,
                    desktop.id
                );
                false
            }
        }
    }

    fn launch_or_focus_shiraz_package(&mut self, package: &str, desktop_id: Option<&str>) {
        let canonical_did = desktop_id
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("{package}.Shiraz.desktop"));
        if let Some(idx) = self.find_wayland_app_for_focus(|a| {
            a.desktop_id.as_deref() == Some(canonical_did.as_str())
        }) {
            let has_buffer = self.wayland_apps[idx].has_buffer;
            self.active_wayland_idx = Some(idx);
            self.hw.boost_gpu(Duration::from_millis(800));
            self.shell
                .set_active_wayland_desktop(Some(canonical_did.as_str()));
            if let Some(surf) = self.active_wayland_surface()
                && let Some(kb) = self.seat.get_keyboard()
            {
                kb.set_focus(self, Some(surf), SERIAL_COUNTER.next_serial());
            }
            tracing::info!(
                "Focused running Shiraz Android app {package} (toplevel #{idx}, has_buffer={has_buffer})"
            );
            self.needs_render = true;
            return;
        }
        let (app_w, app_h) = self.app_viewport_size();
        self.pending_launch = None;
        self.pending_launch_desktop = Some(canonical_did);
        self.hw.boost_gpu(Duration::from_millis(2500));
        if let Err(err) = self.hw.shiraz.launch_package(
            &mut self.shell.sys,
            package,
            app_w.max(120) as u32,
            app_h.max(120) as u32,
        ) {
            tracing::warn!("Failed to launch Shiraz Android app {package}: {err}");
            self.pending_launch_desktop = None;
        }
    }

    fn launch_or_focus_desktop_app(&mut self, desktop_id: &str, argument: Option<&str>) {
        let Some(mut desktop) = self.shell.desktop_registry.get(desktop_id).cloned() else {
            tracing::warn!("Unknown Freedesktop app id: {desktop_id}");
            return;
        };
        if let Some(argument) = argument {
            desktop.exec_argv.push(argument.to_owned());
        }
        if let Some(idx) =
            self.find_wayland_app_for_focus(|a| a.desktop_id.as_deref() == Some(desktop_id))
        {
            if argument.is_some() {
                // GNOME Settings forwards the panel argument to its running instance.
                self.spawn_desktop_app(&desktop);
            }
            let has_buffer = self.wayland_apps[idx].has_buffer;
            self.active_wayland_idx = Some(idx);
            self.hw.boost_gpu(Duration::from_millis(800));
            self.shell.set_active_wayland_desktop(Some(desktop_id));
            if let Some(surf) = self.active_wayland_surface()
                && let Some(kb) = self.seat.get_keyboard()
            {
                kb.set_focus(self, Some(surf), SERIAL_COUNTER.next_serial());
            }
            tracing::info!(
                "Focused running Freedesktop app {desktop_id} (toplevel #{idx}, has_buffer={has_buffer})"
            );
            self.needs_render = true;
            return;
        }

        if let Some(ref pkg) = desktop.shiraz_package {
            self.launch_or_focus_shiraz_package(pkg, Some(&desktop.id));
            return;
        }
        self.pending_launch = None;
        self.pending_launch_desktop = Some(desktop.id.clone());
        if !self.spawn_desktop_app(&desktop) {
            self.pending_launch_desktop = None;
        }
    }

    fn launch_or_focus_app(&mut self, app: AppId) {
        let matched_desktop = self.shell.desktop_registry.find_for_builtin(app).cloned();
        if let Some(idx) = self.find_wayland_app_for_focus(|a| {
            a.app_id == Some(app)
                || matched_desktop
                    .as_ref()
                    .is_some_and(|d| a.desktop_id.as_deref() == Some(d.id.as_str()))
        }) {
            let has_buffer = self.wayland_apps[idx].has_buffer;
            self.active_wayland_idx = Some(idx);
            self.hw.boost_gpu(Duration::from_millis(800));
            self.shell.set_active_wayland_app(Some(app));
            if let Some(surf) = self.active_wayland_surface()
                && let Some(kb) = self.seat.get_keyboard()
            {
                kb.set_focus(self, Some(surf), SERIAL_COUNTER.next_serial());
            }
            tracing::info!(
                "Focused running Wayland app {app:?} (toplevel #{idx}, has_buffer={has_buffer})"
            );
            self.needs_render = true;
            return;
        }

        // Prefer real installed Freedesktop `.desktop` entry when present.
        if let Some(desktop) = matched_desktop {
            if let Some(ref pkg) = desktop.shiraz_package {
                self.launch_or_focus_shiraz_package(pkg, Some(&desktop.id));
                return;
            }
            self.pending_launch = Some(app);
            self.pending_launch_desktop = Some(desktop.id.clone());
            if self.spawn_desktop_app(&desktop) {
                return;
            }
            self.pending_launch = None;
            self.pending_launch_desktop = None;
        }

        for &(bin, args) in native_candidates(app) {
            if let Some(path) = find_in_path(bin) {
                self.pending_launch = Some(app);
                self.hw.boost_gpu(Duration::from_millis(2500));
                let mut cmd = Command::new(&path);
                cmd.args(args)
                    .env_remove("LD_LIBRARY_PATH")
                    .env_remove("BHOO_FONT")
                    .env_remove("RUST_LOG")
                    .env_remove("GTK_IM_MODULE")
                    .env("WAYLAND_DISPLAY", &self.socket_name)
                    .env("XDG_SESSION_TYPE", "wayland")
                    .env("GDK_BACKEND", "wayland")
                    .env("QT_QPA_PLATFORM", "wayland")
                    .env("SDL_VIDEODRIVER", "wayland")
                    .env("NO_AT_BRIDGE", "1")
                    .env("MESA_DISK_CACHE_SINGLE_FILE", "1")
                    .env("XDG_CURRENT_DESKTOP", "Wisp:SpectreOS:GNOME")
                    .env("PATH", "/run/current-system/sw/bin:/run/wrappers/bin:/usr/bin:/bin")
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                if std::env::var_os("GSK_RENDERER").is_none() {
                    cmd.env("GSK_RENDERER", "gl");
                }
                match cmd.spawn() {
                    Ok(child) => {
                        let pid = child.id();
                        self.spawned_children.push(child);
                        tracing::info!(
                            "Launched real Wayland app {app:?} -> {} (pid={})",
                            path.display(),
                            pid
                        );
                        return;
                    }
                    Err(err) => {
                        tracing::warn!("Failed to launch {}: {err}", path.display());
                    }
                }
            }
        }
    }

    fn request_close_entry(&mut self, idx: usize) {
        let Some(entry) = self.wayland_apps.get(idx) else {
            return;
        };
        if let Some(tl) = entry.window.toplevel() {
            let oid = tl.wl_surface().id();
            tl.send_close();
            if let Some(pid) = entry.pid {
                self.pending_kills
                    .push((Instant::now() + Duration::from_millis(600), pid, oid));
            }
        }
    }

    fn process_host_commands(&mut self) {
        self.spawned_children
            .retain_mut(|child| matches!(child.try_wait(), Ok(None)));
        for cmd in self.shell.take_host_commands() {
            match cmd {
                HostCommand::LaunchOrFocus(app) => self.launch_or_focus_app(app),
                HostCommand::JoinWifi(network) => self.hw.wifi.join(network),
                HostCommand::ConnectBluetooth(address, connect) => self.hw.bluetooth.connect(address, connect),
                HostCommand::LaunchFilesWindow(folder) => {
                    let program = std::env::var_os("HOME").map(PathBuf::from)
                        .map(|home| home.join(".local/bin/spectre-files")).filter(|path| path.is_file())
                        .unwrap_or_else(|| PathBuf::from("spectre-files"));
                    let mut command = Command::new(program);
                    command.env("WAYLAND_DISPLAY", &self.socket_name);
                    if let Some(folder) = folder { command.arg("--start").arg(folder); }
                    match command.spawn() {
                        Ok(child) => self.spawned_children.push(child),
                        Err(error) => self.shell.show_toast(format!("Could not open Files window: {error}"), self.started.elapsed().as_secs_f64()),
                    }
                }
                HostCommand::OpenFile(path) => {
                    match Command::new("xdg-open").arg(path).env("WAYLAND_DISPLAY", &self.socket_name)
                        .env("GDK_BACKEND", "wayland").env_remove("LD_LIBRARY_PATH").spawn()
                    {
                        Ok(child) => self.spawned_children.push(child),
                        Err(error) => self.shell.show_toast(format!("Could not open file: {error}"), self.started.elapsed().as_secs_f64()),
                    }
                }
                HostCommand::ShareFiles(paths) => {
                    let uris: String = paths.iter().map(|path| format!("file://{}\r\n", spectre_files::uri_path(path))).collect();
                    set_data_device_selection(&self.display_handle, &self.seat, vec!["text/uri-list".into()], uris.into_bytes().into());
                    self.shell.show_toast("File links copied", self.started.elapsed().as_secs_f64());
                }
                HostCommand::LaunchDesktopApp(id, argument) => self.launch_or_focus_desktop_app(&id, argument),
                HostCommand::ShirazLaunchPackage {
                    package,
                    desktop_id,
                } => self.launch_or_focus_shiraz_package(&package, desktop_id.as_deref()),
                HostCommand::ShirazStopPackage(pkg) => {
                    let did = format!("{pkg}.Shiraz.desktop");
                    let targets: Vec<usize> = self
                        .wayland_apps
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.desktop_id.as_deref() == Some(did.as_str()))
                        .map(|(i, _)| i)
                        .collect();
                    for idx in targets {
                        self.request_close_entry(idx);
                    }
                    self.hw.shiraz.stop_package(&pkg);
                }
                HostCommand::ShirazStopAll => {
                    let targets: Vec<usize> = self
                        .wayland_apps
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| {
                            e.desktop_id
                                .as_deref()
                                .is_some_and(|id| id.ends_with(".Shiraz.desktop"))
                        })
                        .map(|(i, _)| i)
                        .collect();
                    for idx in targets {
                        self.request_close_entry(idx);
                    }
                    self.hw.shiraz.stop_all();
                }
                HostCommand::ShirazRefresh => {
                    self.hw.shiraz.refresh();
                    if self.shell.refresh_desktop_apps_if_changed() {
                        self.needs_render = true;
                    }
                }
                HostCommand::MinimizeAll => {
                    self.save_active_wayland_frame();
                    self.dismiss_active_popup();
                    self.active_wayland_idx = None;
                    self.touch_target_wayland = false;
                    self.active_wayland_touch_slots.clear();
                    self.active_touch_surface = None;
                    if let Some(kb) = self.seat.get_keyboard() {
                        kb.set_focus(self, None, SERIAL_COUNTER.next_serial());
                    }
                }
                HostCommand::CloseApp(app) => {
                    let matched_did = self
                        .shell
                        .desktop_registry
                        .find_for_builtin(app)
                        .map(|d| d.id.clone());
                    let targets: Vec<usize> = self
                        .wayland_apps
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| {
                            e.app_id == Some(app)
                                || matched_did
                                    .as_deref()
                                    .is_some_and(|did| e.desktop_id.as_deref() == Some(did))
                        })
                        .map(|(i, _)| i)
                        .collect();
                    for idx in targets {
                        self.request_close_entry(idx);
                    }
                }
                HostCommand::CloseDesktopApp(id) => {
                    let targets: Vec<usize> = self
                        .wayland_apps
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| {
                            e.desktop_id.as_deref() == Some(id.as_str())
                                || e.app_id.is_some_and(|a| {
                                    self.shell
                                        .desktop_registry
                                        .find_for_builtin(a)
                                        .is_some_and(|d| d.id == id)
                                })
                        })
                        .map(|(i, _)| i)
                        .collect();
                    for idx in targets {
                        self.request_close_entry(idx);
                    }
                }
                HostCommand::CloseAllApps => {
                    for idx in 0..self.wayland_apps.len() {
                        self.request_close_entry(idx);
                    }
                    self.dismiss_active_popup();
                    self.active_wayland_idx = None;
                    if let Some(kb) = self.seat.get_keyboard() {
                        kb.set_focus(self, None, SERIAL_COUNTER.next_serial());
                    }
                }
                HostCommand::SendBack => {
                    // 1. Dismiss any open GTK/Wayland menu popup first.
                    if self.dismiss_active_popup() {
                        continue;
                    }
                    // 2. Dismiss visible OSK keyboard on Back before navigating away.
                    if self.osk_bottom_inset > 0 {
                        if let Some(ref mut daemon) = self.notif_daemon {
                            daemon.dismiss_osk();
                        }
                        if let Some(kb) = self.seat.get_keyboard() {
                            kb.set_focus(self, None, SERIAL_COUNTER.next_serial());
                        }
                        self.needs_render = true;
                        continue;
                    }
                    // 3. Quick second Back swipe within 1.5s returns cleanly to Wisp Home.
                    if self
                        .last_back_sent
                        .is_some_and(|t| t.elapsed() < Duration::from_millis(1500))
                    {
                        self.last_back_sent = None;
                        self.active_wayland_idx = None;
                        self.touch_target_wayland = false;
                        self.active_wayland_touch_slots.clear();
                        self.active_touch_surface = None;
                        if let Some(kb) = self.seat.get_keyboard() {
                            kb.set_focus(self, None, SERIAL_COUNTER.next_serial());
                        }
                        self.shell.go_home_now(self.started.elapsed().as_secs_f64());
                        self.needs_render = true;
                        continue;
                    }
                    // 4. Forward standard XF86Back (evdev 158 + 8 = 166) to the active Wayland app.
                    self.last_back_sent = Some(Instant::now());
                    if let Some(kb) = self.seat.get_keyboard() {
                        let serial = SERIAL_COUNTER.next_serial();
                        let time = self.started.elapsed().as_millis() as u32;
                        kb.input::<(), _>(
                            self,
                            166u32.into(),
                            KeyState::Pressed,
                            serial,
                            time,
                            |_, _, _| FilterResult::Forward,
                        );
                        kb.input::<(), _>(
                            self,
                            166u32.into(),
                            KeyState::Released,
                            SERIAL_COUNTER.next_serial(),
                            time + 10,
                            |_, _, _| FilterResult::Forward,
                        );
                    }
                }
                HostCommand::NotifyClosed { id, reason } => {
                    if let Some(ref mut daemon) = self.notif_daemon {
                        daemon.emit_closed(id, reason);
                    }
                }
                HostCommand::NotifyActionInvoked { id, action_key } => {
                    if let Some(ref mut daemon) = self.notif_daemon {
                        daemon.emit_action_invoked(id, &action_key);
                    }
                }
            }
        }
    }

    fn drain_notification_events(&mut self) {
        let events = match self.notif_daemon.as_mut() {
            Some(daemon) => daemon.process(),
            None => return,
        };
        if events.is_empty() {
            return;
        }
        let now = self.started.elapsed().as_secs_f64().max(self.last_anim_time);
        for ev in events {
            match ev {
                NotificationEvent::Notify(incoming) => {
                    let should_wake = self.shell.push_freedesktop_notification(incoming, now);
                    if !self.screen_on && should_wake {
                        self.wake_to_lockscreen();
                    } else {
                        self.needs_render = true;
                    }
                }
                NotificationEvent::ClosedByClient(id) => {
                    self.shell.close_freedesktop_notification(id);
                    self.needs_render = true;
                }
            }
        }
    }

    fn next_frame_time(&mut self) -> f64 {
        let wall = self.started.elapsed().as_secs_f64();
        let now = if self.continuous_anim && self.last_anim_time > 0.0 {
            // Phase-lock animations to exact hardware VSync cadence (`frame_period_s`)
            // whenever we are within +/-45% of a VBlank tick to eliminate epoll jitter.
            let stepped = self.last_anim_time + self.frame_period_s;
            if (stepped - wall).abs() <= self.frame_period_s * 0.45 {
                stepped
            } else {
                wall.max(self.last_anim_time)
            }
        } else {
            wall.max(self.last_anim_time)
        };
        self.last_anim_time = now;
        now
    }

    fn handle_touch_down(&mut self, slot: i32, pos: Point<f64, Logical>, time: u32) {
        let logical_w = (self.phys_size[0] as f32 / self.scale).round();
        let logical_h = (self.phys_size[1] as f32 / self.scale).round();
        let on_status_bar = pos.y <= (TOP_BAR_H + 14.0) as f64;

        self.last_activity = Instant::now();
        if self.screen_dimmed {
            self.screen_dimmed = false;
            self.hw.set_backlight_on();
        }

        // 1. Check active wlr-layer-shell surfaces (e.g. SpectreOS Keyboard) first!
        if !on_status_bar && !self.shell.lockscreen.locked && !self.shell.notification_shade_visible() {
            let layer_hit: Option<(WlSurface, Point<f64, Logical>)> = {
                let map = layer_map_for_output(&self.output);
                map.layer_under(WlrLayer::Overlay, pos)
                    .or_else(|| map.layer_under(WlrLayer::Top, pos))
                    .and_then(|layer| {
                        let geo = map.layer_geometry(layer)?;
                        let rel: Point<f64, Logical> =
                            (pos.x - geo.loc.x as f64, pos.y - geo.loc.y as f64).into();
                        layer
                            .surface_under(rel, WindowSurfaceType::ALL)
                            .map(|(s, off)| {
                                (
                                    s,
                                    ((geo.loc.x + off.x) as f64, (geo.loc.y + off.y) as f64).into(),
                                )
                            })
                    })
            };
            if let Some((surface, origin)) = layer_hit
                && let Some(touch) = self.seat.get_touch()
            {
                self.active_wayland_touch_slots.insert(slot);
                self.touch_target_wayland = true;
                self.active_touch_surface = Some((surface.clone(), origin, true));
                let serial = SERIAL_COUNTER.next_serial();
                touch.down(
                    self,
                    Some((surface, origin)),
                    &DownEvent {
                        slot: Some(slot as u32).into(),
                        location: pos,
                        serial,
                        time,
                    },
                );
                touch.frame(self);
                self.needs_render = true;
                return;
            }
        }

        // 2. Active Wayland app region (excluding top bar, bottom bar, heads-up banner, and Edge-Back strips)
        let on_heads_up = self
            .shell
            .active_heads_up_rect()
            .is_some_and(|r| r.contains([pos.x as f32, pos.y as f32]));
        let on_edge_back =
            pos.x <= EDGE_BACK_STRIP || pos.x >= (logical_w as f64 - EDGE_BACK_STRIP);
        let nav_touch_height = match self.shell.sys.nav_mode {
            crate::NavMode::Gestural => 34.0,
            crate::NavMode::ThreeButton => self.shell.nav_bar_height(),
        };
        let in_app_region = !on_heads_up
            && !on_edge_back
            && !on_status_bar
            && pos.y < (logical_h - nav_touch_height) as f64;
        if in_app_region
            && self.shell.is_wayland_passthrough()
            && let Some(idx) = self.active_wayland_idx
            && let Some(managed) = self.wayland_apps.get(idx)
        {
            let app_loc: Point<f64, Logical> = (pos.x, pos.y - TOP_BAR_H as f64).into();
            let popup_hit = managed
                .window
                .surface_under(app_loc, WindowSurfaceType::POPUP);
            // Dismiss grabbed GTK menu popup if the user tapped outside the popup
            if self.active_popup.is_some() && popup_hit.is_none() {
                self.dismiss_active_popup();
                return;
            }
            let hit = popup_hit
                .or_else(|| {
                    managed
                        .window
                        .surface_under(app_loc, WindowSurfaceType::ALL)
                })
                .or_else(|| self.active_wayland_surface().map(|s| (s, (0, 0).into())));
            if let Some((surface, surf_off)) = hit
                && let Some(touch) = self.seat.get_touch()
            {
                let origin: Point<f64, Logical> =
                    (surf_off.x as f64, surf_off.y as f64 + TOP_BAR_H as f64).into();
                self.active_wayland_touch_slots.insert(slot);
                self.touch_target_wayland = true;
                self.active_touch_surface = Some((surface.clone(), origin, false));
                let serial = SERIAL_COUNTER.next_serial();
                touch.down(
                    self,
                    Some((surface, origin)),
                    &DownEvent {
                        slot: Some(slot as u32).into(),
                        location: pos,
                        serial,
                        time,
                    },
                );
                touch.frame(self);
                self.needs_render = true;
                return;
            }
        }

        if self.active_wayland_touch_slots.is_empty() {
            self.touch_target_wayland = false;
            self.active_touch_surface = None;
        }
        self.hw.boost_gpu(Duration::from_millis(800));
        if self.input.touch.is_none() {
            if on_status_bar && self.shell.is_wayland_passthrough() {
                self.capture_wayland_shade_frame();
            }
            let t = self.started.elapsed().as_secs_f64();
            self.input.touch = Some(slot);
            self.input.kind = bhoo::PointerKind::Touch;
            self.input.pointer = Some([pos.x as f32, pos.y as f32]);
            self.input.push(true, false, false, t);
            self.needs_render = true;
        }
    }
    fn handle_touch_motion(&mut self, slot: i32, pos: Point<f64, Logical>, time: u32) {
        self.last_activity = Instant::now();
        if self.touch_target_wayland && self.active_wayland_touch_slots.contains(&slot) {
            let focus = self
                .active_touch_surface
                .as_ref()
                .map(|(s, o, _)| (s.clone(), *o))
                .or_else(|| {
                    self.active_wayland_surface()
                        .map(|s| (s, (0.0, TOP_BAR_H as f64).into()))
                });
            if let Some((surface, origin)) = focus
                && let Some(touch) = self.seat.get_touch()
            {
                touch.motion(
                    self,
                    Some((surface, origin)),
                    &MotionEvent {
                        slot: Some(slot as u32).into(),
                        location: pos,
                        time,
                    },
                );
                touch.frame(self);
                self.needs_render = true;
            }
            return;
        }
        if self.input.touch == Some(slot) {
            let t = self.started.elapsed().as_secs_f64();
            self.hw.boost_gpu(Duration::from_millis(600));
            self.input.pointer = Some([pos.x as f32, pos.y as f32]);
            self.input.push(false, false, false, t);
            self.needs_render = true;
        }
    }
    fn handle_touch_up(&mut self, slot: i32, time: u32) {
        self.last_activity = Instant::now();
        if self.active_wayland_touch_slots.remove(&slot) {
            if let Some(touch) = self.seat.get_touch() {
                touch.up(
                    self,
                    &UpEvent {
                        slot: Some(slot as u32).into(),
                        serial: SERIAL_COUNTER.next_serial(),
                        time,
                    },
                );
                touch.frame(self);
            }
            if self.active_wayland_touch_slots.is_empty() {
                self.touch_target_wayland = false;
                self.active_touch_surface = None;
            }
            self.needs_render = true;
            return;
        }
        if self.input.touch == Some(slot) {
            let t = self.started.elapsed().as_secs_f64();
            self.hw.boost_gpu(Duration::from_millis(800));
            self.input.touch = None;
            self.input.push(false, true, false, t);
            self.needs_render = true;
        }
    }
    fn cancel_wayland_touches(&mut self) {
        if self.touch_target_wayland || !self.active_wayland_touch_slots.is_empty() {
            if let Some(touch) = self.seat.get_touch() {
                touch.cancel(self);
                touch.frame(self);
            }
            self.touch_target_wayland = false;
            self.active_wayland_touch_slots.clear();
            self.active_touch_surface = None;
        }
    }

    fn handle_touch_cancel(&mut self) {
        self.cancel_wayland_touches();
        self.dt2w_tap_down = None;
        let t = self.started.elapsed().as_secs_f64();
        self.input.touch = None;
        self.input.pointer = None;
        self.input.push(false, false, true, t);
        self.needs_render = true;
    }

    fn capture_settings_return(&mut self) {
        // Rasterize the last fullscreen Settings frame once, before the gesture
        // changes its geometry. Keep the bitmap within Bhoo's texture limit.
        let ratio = (960.0 / self.phys_size[0].max(self.phys_size[1]) as f32).min(1.0);
        let size = self.phys_size.map(|dimension| (dimension as f32 * ratio).round().max(1.0) as u32);
        let scale = self.scale * ratio;
        let phone = self.shell.last_phone_rect;
        let commands = self.commands.borrow()
            .iter()
            .take(self.previous_scene_command_count)
            // Display tint/dimming is applied once to the entire animated scene.
            .filter(|command| !matches!(command,
                bhoo::DrawCommand::RoundedRect { bounds, radius, color }
                    if *bounds == phone && *radius == 0.0 && color.3 < 255))
            .cloned()
            .collect();
        let element = BhooElement {
            id: self.bhoo_id.clone(),
            commit: self.bhoo_commit,
            renderer: Rc::clone(&self.bhoo_renderer),
            commands: Rc::new(RefCell::new(commands)),
            phys_size: size,
            scale,
            opaque: true,
        };
        match crate::capture::capture(&mut self.renderer, &[element], size, scale as f64) {
            Ok(mut pixels) => {
                // Keep the fullscreen aspect ratio while dropping the system bars.
                let top = (TOP_BAR_H * scale).round() as usize;
                let nav = self.shell.nav_bar_height();
                let height = pixels.height as usize;
                let bottom = height.saturating_sub((nav * scale).round() as usize);
                let stride = pixels.width as usize * 4;
                if top < bottom && pixels.rgba.len() == height * stride {
                    let bhoo::Color(r, g, b, a) = self.shell.sys.palette().settings_bg;
                    let (header, content) = pixels.rgba.split_at_mut(top * stride);
                    let footer = &mut content[(bottom - top) * stride..];
                    for pixel in header.chunks_exact_mut(4).chain(footer.chunks_exact_mut(4)) {
                        pixel.copy_from_slice(&[r, g, b, a]);
                    }
                    self.shell.settings_return_snapshot =
                        bhoo::Image::rgba(pixels.width, pixels.height, pixels.rgba);
                }
            }
            Err(error) => tracing::warn!("Settings return snapshot unavailable: {error}"),
        }
    }

    fn prepare_wayland_launch_frame(&mut self) {
        let active = self.active_wayland_idx.and_then(|idx| self.wayland_apps.get(idx));
        self.shell.wayland_surface_ready = active.is_some_and(|app| app.has_buffer);
        let opening = matches!(self.shell.app_transition,
            AppTransition::OpeningDesktop { .. } | AppTransition::Opening { from_shade: false, .. });
        if !opening {
            self.shell.wayland_launch_frame = None;
            return;
        }
        let Some(app) = active.filter(|app| app.has_buffer) else {
            return;
        };
        let target_is_active = match self.shell.mode {
            ShellMode::FullscreenDesktop(idx) => self.shell.active_wayland_desktop == Some(idx),
            ShellMode::Fullscreen(id) => self.shell.active_wayland_app == Some(id),
            _ => false,
        };
        if !target_is_active || self.shell.wayland_launch_capture_attempted || self.shell.wayland_launch_frame.is_some() {
            return;
        }
        self.shell.wayland_launch_capture_attempted = true;
        self.shell.wayland_launch_frame = self.capture_wayland_window(&app.window.clone());
    }

    fn save_active_wayland_frame(&mut self) {
        let Some(app) = self.active_wayland_idx.and_then(|idx| self.wayland_apps.get(idx))
            .filter(|app| app.has_buffer) else {
            return;
        };
        let (builtin, desktop_id, window) = (app.app_id, app.desktop_id.clone(), app.window.clone());
        if (builtin.is_some() || desktop_id.is_some())
            && let Some(frame) = self.capture_wayland_window(&window)
        {
            if let Some(app) = builtin {
                self.shell.wayland_task_frames.insert(app.package().to_owned(), frame.clone());
            }
            if let Some(id) = desktop_id {
                self.shell.wayland_task_frames.insert(id, frame);
            }
        }
    }

    fn capture_wayland_window(&mut self, window: &Window) -> Option<bhoo::Image> {
        let (width, height) = self.app_viewport_size();
        let scale = self.scale.min(960.0 / width.max(height) as f32);
        let size = [width, height].map(|dimension| (dimension as f32 * scale).round() as u32);
        let elements = window.render_elements::<WaylandSurfaceRenderElement<GlesRenderer>>(
            &mut self.renderer, (0, 0).into(), Scale::from(f64::from(scale)), 1.0,
        );
        if elements.is_empty() {
            return None;
        }
        match crate::capture::capture(&mut self.renderer, &elements, size, f64::from(scale)) {
            Ok(pixels) => bhoo::Image::rgba(pixels.width, pixels.height, pixels.rgba),
            Err(error) => {
                tracing::warn!("Wayland window snapshot unavailable: {error}");
                None
            }
        }
    }

    fn capture_wayland_shade_frame(&mut self) {
        self.shade_capture_attempted = true;
        if let Some(window) = self.active_wayland_idx
            .and_then(|idx| self.wayland_apps.get(idx))
            .filter(|app| app.has_buffer)
            .map(|app| app.window.clone())
        {
            self.shell.wayland_shade_frame = self.capture_wayland_window(&window);
        }
    }

    fn render_frame(&mut self) {
        if !self.screen_on && !self.park_black_frame {
            self.needs_render = false;
            return;
        }
        if self.waiting_for_vblank {
            self.needs_render = true;
            return;
        }
        self.needs_render = false;
        if self.park_black_frame {
            // Queue a single black scanout even when sleep interrupted an outstanding flip.
            self.drm_compositor.reset_buffers();
            let result = self.drm_compositor.render_frame::<_, WispRenderElement>(
                &mut self.renderer, &[], Color32F::new(0.0, 0.0, 0.0, 1.0),
                smithay::backend::drm::compositor::FrameFlags::empty(),
            );
            match result {
                Ok(frame) if frame.is_empty => self.park_black_frame = false,
                Ok(_) => match self.drm_compositor.queue_frame(None) {
                    Ok(()) => {
                        self.park_black_frame = false;
                        self.waiting_for_vblank = true;
                        self.waiting_for_vblank_since = Some(Instant::now());
                    }
                    Err(err) => tracing::warn!("Failed to queue sleeping display: {err}"),
                },
                Err(err) => tracing::warn!("Failed to render sleeping display: {err}"),
            }
            return;
        }
        self.hw.boost_gpu(Duration::from_millis(600));
        self.prepare_wayland_launch_frame();
        if !self.shell.shows_wayland_surface()
            || (!self.shell.notification_shade_visible() && self.input.touch.is_none() && self.input.events.is_empty())
        {
            self.shell.wayland_shade_frame = None;
            self.shade_capture_attempted = false;
        } else if self.shell.notification_shade_visible() && !self.shade_capture_attempted {
            self.capture_wayland_shade_frame();
        }

        let logical_w = self.phys_size[0] as f32 / self.scale;
        let logical_h = self.phys_size[1] as f32 / self.scale;
        let frame_rect = bhoo::Rect {
            x: 0.0,
            y: 0.0,
            width: logical_w,
            height: logical_h,
        };
        self.ui.viewport = Some(frame_rect);

        let returning_from_settings = matches!(self.shell.app_transition,
            AppTransition::Closing { app: AppId::Settings, target_icon: Some(_), .. });
        let starting_home_gesture = self.input.events.iter().any(|input| {
            input.pressed && input.pointer.is_some_and(|[_, y]| y >= logical_h - 44.0)
        });
        if self.previous_frame_is_settings
            && (starting_home_gesture || returning_from_settings
                || matches!(self.shell.mode, ShellMode::Overview))
            && self.shell.settings_return_snapshot.is_none()
        {
            self.capture_settings_return();
        }


        let t_ui = Instant::now();
        let mut cursor_time = self.last_anim_time;
        let frame_now = self.next_frame_time();
        let nav_mode = self.shell.sys.nav_mode;
        let shell = &mut self.shell;
        let compose_body = &mut self.compose_body;
        let mut had_edge = false;

        // Drain any earlier discrete touch/key edges accumulated during the previous VSync
        // interval using their real input timestamps so taps respond in 1 frame and flings
        // retain exact pre-release VelocityTracker velocity with zero extra frame delay.
        {
            let mut cmds = self.commands.borrow_mut();
            while self.input.events.len() > 1 {
                let mut ev = self.input.pop_or_idle(cursor_time);
                ev.time = ev.time.max(cursor_time);
                cursor_time = ev.time;
                had_edge |= ev.pressed || ev.released || ev.cancelled || ev.key.is_some();
                self.ui
                    .frame_into(ev, frame_rect, 12.0, &mut cmds, |frame| {
                        let vp = frame.viewport();
                        let inp = frame.input();
                        shell.show(frame, vp.width, vp.height, inp, compose_body);
                    });
            }

            let mut input = self.input.pop_or_idle(frame_now);
            input.time = if input.pressed || input.released || input.cancelled {
                input.time.max(cursor_time)
            } else {
                frame_now.max(input.time).max(cursor_time)
            };
            self.last_anim_time = input.time;
            had_edge |= input.pressed || input.released || input.cancelled || input.key.is_some();

            self.ui
                .frame_into(input, frame_rect, 12.0, &mut cmds, |frame| {
                    let vp = frame.viewport();
                    let inp = frame.input();
                    shell.show(frame, vp.width, vp.height, inp, compose_body);
                });
        }
        self.previous_frame_is_settings = matches!(self.shell.mode, ShellMode::Fullscreen(AppId::Settings))
            && matches!(self.shell.app_transition, AppTransition::None)
            && !self.shell.lockscreen.locked
            && self.shell.shade_stage == 0
            && self.shell.active_heads_up_rect().is_none()
            && self.input.touch.is_none();
        self.previous_scene_command_count = self.commands.borrow().len();
        if self.input.touch.is_none()
            && self.shell.mode != ShellMode::Overview
            && !matches!(self.shell.app_transition,
                AppTransition::Closing { app: AppId::Settings, .. }
                | AppTransition::Opening { app: AppId::Settings, from_overview: true, .. })
        {
            self.shell.settings_return_snapshot = None;
        }
        let ui_ms = t_ui.elapsed().as_secs_f64() * 1000.0;
        let cmd_count = self.commands.borrow().len() as u32;

        // Direct platform-agnostic hardware + Wayland app lifecycle sync bound to WispShell state
        self.process_host_commands();
        if self.shell.notification_shade_visible() {
            self.cancel_wayland_touches();
        }
        if nav_mode != self.shell.sys.nav_mode {
            self.reconfigure_wayland_viewport();
        }
        for eff in self.shell.drain_haptics() {
            self.hw.play_haptic(eff, &self.shell.sys);
        }
        if self.shell.take_touch_id_retry() {
            self.hw.request_fingerprint_retry();
        }
        let power_off = matches!(self.shell.power_state, PowerState::PoweredOff);
        let reboot = matches!(self.shell.power_state, PowerState::Rebooting(_));
        self.hw
            .sync_from_shell(&mut self.shell.sys, power_off, reboot, self.screen_on);

        let phys_size = self.phys_size;
        let scale = self.scale;
        let opaque = !self.shell.composites_wayland_surface();
        self.bhoo_commit.increment();

        let mut elements: Vec<WispRenderElement> = Vec::with_capacity(12);

        // Render active wlr-layer-shell Overlay/Top surfaces (e.g. SpectreOS Keyboard)
        // on top when unlocked and shade is closed.
        let show_layers = !self.shell.lockscreen.locked && !self.shell.notification_shade_visible();
        let visible_layers: Vec<(DesktopLayerSurface, Rectangle<i32, Logical>)> = if show_layers {
            let map = layer_map_for_output(&self.output);
            map.layers_on(WlrLayer::Overlay)
                .chain(map.layers_on(WlrLayer::Top))
                .filter_map(|l| map.layer_geometry(l).map(|g| (l.clone(), g)))
                .collect()
        } else {
            Vec::new()
        };
        for (layer, geo) in &visible_layers {
            let loc_px: Point<i32, Physical> = (
                (geo.loc.x as f32 * scale).round() as i32,
                (geo.loc.y as f32 * scale).round() as i32,
            )
                .into();
            let layer_els = layer.render_elements::<WaylandSurfaceRenderElement<GlesRenderer>>(
                &mut self.renderer,
                loc_px,
                Scale::from(scale as f64),
                1.0,
            );
            for el in layer_els {
                elements.push(WispRenderElement::from(el));
            }
            layer.send_frame(
                &self.output,
                self.started.elapsed(),
                Some(Duration::ZERO),
                |_, _| Some(self.output.clone()),
            );
        }

        elements.push(WispRenderElement::Bhoo(BhooElement {
            id: self.bhoo_id.clone(),
            commit: self.bhoo_commit,
            renderer: Rc::clone(&self.bhoo_renderer),
            commands: Rc::clone(&self.commands),
            phys_size,
            scale,
            opaque: opaque && visible_layers.is_empty(),
        }));

        if let Some(idx) = self.active_wayland_idx
            && self.shell.shows_wayland_surface()
            && let Some(managed) = self.wayland_apps.get(idx)
        {
            if !opaque {
                let top_px = (TOP_BAR_H * scale).round() as i32;
                let app_elements = managed
                    .window
                    .render_elements::<WaylandSurfaceRenderElement<GlesRenderer>>(
                        &mut self.renderer,
                        (0, top_px).into(),
                        Scale::from(scale as f64),
                        1.0,
                    );
                for el in app_elements {
                    elements.push(WispRenderElement::from(el));
                }
            }
            managed.window.send_frame(
                &self.output,
                self.started.elapsed(),
                Some(Duration::ZERO),
                |_, _| Some(self.output.clone()),
            );
        }

        let mut lock_black = false;
        let fade_progress = if let Some(transition) = &self.screen_off_transition {
            let progress = (transition.started.elapsed().as_secs_f32()
                / transition.duration.as_secs_f32()).min(1.0);
            lock_black = progress >= 1.0;
            Some(progress)
        } else if matches!(self.pending_backlight_on, Some(BacklightWake::LockScreen(_)))
            && self.shell.sys.anim_scale() > 0.0 {
            // Present black first. The reveal clock starts when the backlight turns on.
            Some(1.0)
        } else if let Some((start, duration)) = self.lock_reveal {
            let progress = 1.0 - start.elapsed().as_secs_f32() / duration.as_secs_f32();
            if progress > 0.0 && self.shell.lockscreen.locked {
                Some(progress)
            } else {
                self.lock_reveal = None;
                None
            }
        } else {
            None
        };
        if let (Some(progress), Some(effect)) = (fade_progress, &mut self.lock_fade) {
            effect.update_uniforms(vec![Uniform::new("progress", progress)]);
            elements.insert(0, WispRenderElement::LockFade(effect.clone()));
        }

        #[cfg(feature = "remote_control")]
        self.remote_render(&mut elements);

        let t0 = Instant::now();
        let render_result = self.drm_compositor.render_frame::<_, WispRenderElement>(
            &mut self.renderer,
            &elements,
            Color32F::new(0.05, 0.06, 0.08, 1.0),
            smithay::backend::drm::compositor::FrameFlags::empty(),
        );

        match render_result {
            Ok(res) => {
                if !res.is_empty {
                    if let Err(err) = self.drm_compositor.queue_frame(self.pending_backlight_on) {
                        tracing::warn!("drm queue_frame error: {err}");
                        self.drm_compositor.reset_state();
                        self.waiting_for_vblank = false;
                        self.waiting_for_vblank_since = None;
                    } else {
                        self.waiting_for_vblank = true;
                        self.waiting_for_vblank_since = Some(Instant::now());
                        if let Some(transition) = &mut self.screen_off_transition {
                            transition.presented_frames += 1;
                            transition.black_queued = lock_black;
                        }
                    }
                }
            }
            Err(err) => {
                tracing::warn!("drm render_frame error: {err}");
                self.drm_compositor.reset_state();
                self.waiting_for_vblank = false;
                self.waiting_for_vblank_since = None;
            }
        }
        let drm_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let stats = self.bhoo_renderer.borrow().last_stats();
        let was_continuous = self.continuous_anim;
        self.perf
            .record_frame(ui_ms, drm_ms, cmd_count, stats, was_continuous);
        if ui_ms + drm_ms > 15.0 {
            tracing::warn!(
                "slow frame: ui={ui_ms:.2}ms gl+drm={drm_ms:.2}ms cmds={cmd_count} quads={} flushes={} layers={}",
                stats.quads,
                stats.flushes,
                stats.layers,
            );
        }

        let lock_animating = self.lock_reveal.is_some() || self.screen_off_transition.as_ref()
            .is_some_and(|transition| !transition.black_queued);
        let retry_wake = self.pending_backlight_on.is_some() && !self.waiting_for_vblank;
        let want_next = retry_wake || had_edge || self.ui.needs_redraw() || lock_animating || !self.input.events.is_empty();
        self.continuous_anim = self.ui.needs_redraw() || lock_animating;
        if want_next {
            self.needs_render = true;
        }
    }
}

fn resolve_font() -> Result<PathBuf, Box<dyn std::error::Error>> {
    use crate::services::{FontBackend, FontPolicy};
    let backend = crate::backends::linux::fonts::LinuxFontBackend::discover();
    backend
        .resolve_primary_font(&FontPolicy::default())
        .ok_or_else(|| "No default font found. Set BHOO_FONT to a TTF/OTF file.".into())
}

/// Lightweight pure-Rust `sd_notify` client (supports both filesystem and `@abstract`
/// `$NOTIFY_SOCKET` without external crates) for systemd `Type=notify` and `WatchdogSec=`.
fn sd_notify(payload: &str) {
    let Ok(raw) = std::env::var("NOTIFY_SOCKET") else {
        return;
    };
    if raw.is_empty() {
        return;
    }
    let Ok(sock) = std::os::unix::net::UnixDatagram::unbound() else {
        return;
    };
    let _ = sock.set_nonblocking(true);
    #[cfg(target_os = "linux")]
    if let Some(abstract_name) = raw.strip_prefix('@') {
        use std::os::linux::net::SocketAddrExt;
        if let Ok(addr) = std::os::unix::net::SocketAddr::from_abstract_name(abstract_name) {
            let _ = sock.send_to_addr(payload.as_bytes(), &addr);
        }
        return;
    }
    let _ = sock.send_to(payload.as_bytes(), &raw);
}

/// Run Wisp as the native DRM/KMS + libinput Wayland compositor & mobile shell.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Ensure any unexpected panic logs clearly and terminates the process immediately so
    // systemd (`Restart=always`, `RestartSec=1`) auto-restarts Wisp instead of leaving a
    // wedged thread or dead hardware power button.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("WISP_FATAL_PANIC: {info}");
        default_hook(info);
        std::process::exit(101);
    }));

    let font_path = resolve_font()?;
    let scale: f32 = std::env::var("WISP_SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2.7);

    let mut event_loop: EventLoop<WispCompositorState> = EventLoop::try_new()?;
    let loop_handle = event_loop.handle();
    let loop_signal = event_loop.get_signal();

    let (mut session, session_notifier) = LibSeatSession::new()?;
    let seat_name = session.seat();
    tracing::info!("libseat session opened on seat '{seat_name}'");

    let card_path = std::env::var("WISP_DRM_DEVICE").unwrap_or_else(|_| "/dev/dri/card0".into());
    let fd: OwnedFd = session.open(
        Path::new(&card_path),
        OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY,
    )?;
    let drm_fd = DrmDeviceFd::new(DeviceFd::from(fd));
    let (mut drm, drm_notifier) = DrmDevice::new(drm_fd.clone(), true)?;
    let gbm = GbmDevice::new(drm_fd.clone())?;
    let egl_display = unsafe { EGLDisplay::new(gbm.clone())? };
    let egl_context = EGLContext::new_with_priority(&egl_display, ContextPriority::High)
        .or_else(|_| EGLContext::new(&egl_display))?;
    let mut renderer = unsafe { GlesRenderer::new(egl_context)? };

    let mut bhoo_renderer = renderer.with_context(|_| {
        let glow_ctx = unsafe {
            bhoo::glow::Context::from_loader_function(|name| {
                let c_name = CString::new(name).unwrap();
                smithay::backend::egl::ffi::egl::GetProcAddress(c_name.as_ptr()) as *const _
            })
        };
        bhoo::GlesRenderer::new(glow_ctx, &font_path).expect("bhoo GLES init failed")
    })?;

    let res_handles = drm.resource_handles()?;
    let connector = res_handles
        .connectors()
        .iter()
        .filter_map(|&h| drm.get_connector(h, true).ok())
        .find(|c| c.state() == connector::State::Connected)
        .ok_or("No connected DRM display connector found")?;
    let mode = *connector
        .modes()
        .first()
        .ok_or("Connected DRM connector has no modes")?;
    let phys_size = [mode.size().0 as u32, mode.size().1 as u32];
    let lock_fade = match renderer.compile_custom_pixel_shader(
        include_str!("lock_fade.glsl"),
        &[UniformName::new("progress", UniformType::_1f)],
    ) {
        Ok(shader) => {
            let area = Rectangle::from_size((
                (phys_size[0] as f32 / scale).ceil() as i32,
                (phys_size[1] as f32 / scale).ceil() as i32,
            ).into());
            Some(PixelShaderElement::new(
                shader, area, None, 1.0, vec![Uniform::new("progress", 0.0_f32)], smithay::backend::renderer::element::Kind::Unspecified,
            ))
        }
        Err(error) => {
            tracing::warn!("Lock animation shader unavailable: {error}");
            None
        }
    };
    let exact_hz = {
        let htotal = mode.hsync().2 as f64;
        let vtotal = mode.vsync().2 as f64;
        if htotal > 0.0 && vtotal > 0.0 && mode.clock() > 0 {
            (mode.clock() as f64 * 1000.0) / (htotal * vtotal)
        } else {
            mode.vrefresh().max(60) as f64
        }
    };
    let frame_period_s = 1.0 / exact_hz;
    tracing::info!(
        "DRM connector {:?}: {}x{}@{:.2}Hz (period={:.3}ms, scale={})",
        connector.interface(),
        phys_size[0],
        phys_size[1],
        exact_hz,
        frame_period_s * 1000.0,
        scale
    );

    let crtc_handle: crtc::Handle = *res_handles
        .crtcs()
        .first()
        .ok_or("No DRM CRTC available")?;
    let drm_surface = drm.create_surface(crtc_handle, mode, &[connector.handle()])?;

    let output = Output::new(
        format!("{:?}-{}", connector.interface(), connector.interface_id()),
        PhysicalProperties {
            size: (
                connector.size().map(|s| s.0 as i32).unwrap_or(68),
                connector.size().map(|s| s.1 as i32).unwrap_or(145),
            )
                .into(),
            subpixel: Subpixel::Unknown,
            make: "SpectreOS".into(),
            model: "Wisp".into(),
        },
    );
    let output_mode = OutputMode {
        size: (phys_size[0] as i32, phys_size[1] as i32).into(),
        refresh: (mode.vrefresh() * 1000) as i32,
    };
    output.change_current_state(
        Some(output_mode),
        Some(Transform::Normal),
        Some(smithay::output::Scale::Fractional(scale as f64)),
        Some((0, 0).into()),
    );
    output.set_preferred(output_mode);

    let allocator = GbmAllocator::new(
        gbm.clone(),
        GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
    );
    let drm_node = DrmNode::from_file(&drm_fd)?;
    let render_node = drm_node
        .node_with_type(NodeType::Render)
        .and_then(|r| r.ok())
        .unwrap_or(drm_node);
    let exporter = GbmFramebufferExporter::new(gbm, Some(drm_node));
    let color_formats = [Fourcc::Argb8888, Fourcc::Xrgb8888];
    let renderer_formats = renderer
        .egl_context()
        .dmabuf_render_formats()
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let drm_compositor = DrmCompositor::new(
        &output,
        drm_surface,
        None,
        allocator,
        exporter,
        color_formats,
        renderer_formats,
        drm.cursor_size(),
        None,
    )?;

    let display: Display<WispCompositorState> = Display::new()?;
    let display_handle = display.handle();
    let socket = ListeningSocketSource::new_auto()?;
    let socket_name = socket.socket_name().to_string_lossy().into_owned();
    tracing::info!("Wisp unified compositor listening on WAYLAND_DISPLAY={socket_name}");

    // Export WAYLAND_DISPLAY into user D-Bus/systemd session so xdg-desktop-portal & GNOME apps connect immediately.
    if let Some(dbus_upd) = find_in_path("dbus-update-activation-environment") {
        let _ = Command::new(dbus_upd)
            .args([
                "--systemd",
                "WAYLAND_DISPLAY",
                "XDG_CURRENT_DESKTOP",
                "XDG_SESSION_TYPE",
            ])
            .env_remove("LD_LIBRARY_PATH")
            .env("WAYLAND_DISPLAY", &socket_name)
            .env("XDG_CURRENT_DESKTOP", "Wisp:SpectreOS:GNOME")
            .env("XDG_SESSION_TYPE", "wayland")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }

    // Expose hardware GPU buffers (`zwp_linux_dmabuf_v1` v4 feedback) so GTK4/WebKit/Qt clients
    // render directly on the Adreno 630 GPU instead of falling back to slow CPU llvmpipe.
    let dmabuf_formats: Vec<_> = renderer.dmabuf_formats().into_iter().collect();
    let dmabuf_format_count = dmabuf_formats.len();
    let default_feedback = DmabufFeedbackBuilder::new(render_node.dev_id(), dmabuf_formats)
        .build()?;
    let mut dmabuf_state = DmabufState::new();
    let dmabuf_global = dmabuf_state.create_global_with_default_feedback::<WispCompositorState>(
        &display_handle,
        &default_feedback,
    );
    tracing::info!(
        "Exposed zwp_linux_dmabuf_v1 on render node {:?} ({dmabuf_format_count} formats)",
        render_node.dev_path(),
    );

    let compositor_state = CompositorState::new::<WispCompositorState>(&display_handle);
    let xdg_shell_state = XdgShellState::new::<WispCompositorState>(&display_handle);
    let layer_shell_state = WlrLayerShellState::new::<WispCompositorState>(&display_handle);
    let shm_state = ShmState::new::<WispCompositorState>(&display_handle, vec![]);
    let mut seat_state = SeatState::new();
    let data_device_state = DataDeviceState::new::<WispCompositorState>(&display_handle);
    let data_control_state = DataControlState::new::<WispCompositorState, _>(&display_handle, None, |_| true);
    let _foreign_toplevel_global =
        display_handle.create_global::<WispCompositorState, ZwlrForeignToplevelManagerV1, _>(3, ());
    let _phoc_device_state_global = display_handle.backend_handle().create_global::<WispCompositorState>(
        &ZPHOC_DEVICE_STATE_V1_INTERFACE,
        2,
        Arc::new(PhocDeviceStateGlobal),
    );
    let output_manager_state =
        OutputManagerState::new_with_xdg_output::<WispCompositorState>(&display_handle);
    let fractional_scale_manager_state =
        FractionalScaleManagerState::new::<WispCompositorState>(&display_handle);
    let viewporter_state = ViewporterState::new::<WispCompositorState>(&display_handle);
    TextInputManagerState::new::<WispCompositorState>(&display_handle);
    InputMethodManagerState::new::<WispCompositorState, _>(&display_handle, |_| true);
    VirtualKeyboardManagerState::new::<WispCompositorState, _>(&display_handle, |_| true);
    let mut seat = seat_state.new_wl_seat(&display_handle, seat_name.clone());
    seat.add_touch();
    let _ = seat.add_keyboard(Default::default(), 200, 25);
    let _output_global = output.create_global::<WispCompositorState>(&display_handle);

    let mut space = Space::default();
    space.map_output(&output, (0, 0));

    let mut libinput_ctx = Libinput::new_with_udev(LibinputSessionInterface::from(session.clone()));
    libinput_ctx
        .udev_assign_seat(&seat_name)
        .map_err(|_| "Failed to assign seat to libinput")?;
    let libinput_backend = LibinputInputBackend::new(libinput_ctx.clone());

    let mut ui = bhoo::Ui::default();
    ui.text_metrics = bhoo_renderer.text_metrics();

    let mut shell = Box::new(WispShell::<NoopEmbeddedApps>::default());
    shell.set_native_compositor(true);
    let mut hw = SystemServices::new(crate::backends::discover(), &mut shell.sys);
    shell.time_service = hw.time.clone();
    hw.boost_gpu(Duration::from_millis(1200));

    // Pre-decode & upload built-in high-res icons + discovered Freedesktop XDG icons into the
    // GLES3 mipmap atlas and pre-allocate 2 offscreen layer FBOs once at boot for 60 Hz launch.
    {
        let palette = shell.sys.palette();
        let mut prewarm = crate::icons::prewarm_all(&palette);
        let builtin_count = prewarm.len();
        let xdg_apps_count = shell.desktop_registry.apps.len();
        let xdg_prewarm = shell.desktop_registry.prewarm_icons(48);
        let xdg_icon_count = xdg_prewarm.len();
        prewarm.extend(xdg_prewarm);
        let t_pw = Instant::now();
        let mut br = bhoo_renderer;
        let _ = renderer.with_context(|_| {
            let _ = br.prewarm_images(&prewarm);
            let _ = br.prewarm_layers(phys_size, 2);
        });
        bhoo_renderer = br;
        tracing::info!(
            "Freedesktop: discovered {} .desktop apps; pre-warmed {} built-in + {} XDG icons + 2 layer FBOs into GPU in {:.1}ms",
            xdg_apps_count,
            builtin_count,
            xdg_icon_count,
            t_pw.elapsed().as_secs_f32() * 1000.0
        );
    }
    tracing::info!(
        "Wisp services ready: profile={} gpu_mhz={} torch={} notif_led={} als={} haptics={} touch_id={} fonts={} (fa={:?})",
        hw.profile_name,
        hw.current_gpu_mhz(),
        hw.flashlight.backend_label(),
        hw.notification_led.backend_label(),
        hw.ambient_light.is_available(),
        hw.haptics.is_available(),
        hw.fingerprint.is_available(),
        hw.fonts.backend_label(),
        hw.fonts.persian_font_path(),
    );
    let bench_enabled = std::env::var("WISP_BENCH").as_deref() == Ok("1");
    let perf_enabled = bench_enabled || std::env::var("WISP_PERF").as_deref() == Ok("1");
    if bench_enabled || std::env::var("WISP_UNLOCKED").as_deref() == Ok("1") {
        shell.lockscreen.locked = false;
    } else {
        shell.lockscreen.lock();
    }

    let notif_daemon = match NotificationDaemon::start() {
        Ok(d) => Some(d),
        Err(err) => {
            tracing::warn!("Freedesktop Notifications D-Bus service unavailable: {err}");
            None
        }
    };
    let notif_fd = notif_daemon.as_ref().map(NotificationDaemon::fd);

    let mut state = WispCompositorState {
        display_handle: display_handle.clone(),
        socket_name,
        compositor_state,
        xdg_shell_state,
        layer_shell_state,
        shm_state,
        dmabuf_state,
        dmabuf_global,
        seat_state,
        data_device_state,
        data_control_state,
        output_manager_state,
        fractional_scale_manager_state,
        viewporter_state,
        space,
        popups: PopupManager::default(),
        seat,
        loop_signal,
        wayland_apps: Vec::new(),
        spawned_children: Vec::new(),
        pending_kills: Vec::new(),
        active_wayland_idx: None,
        active_popup: None,
        active_touch_surface: None,
        last_back_sent: None,
        pending_launch: None,
        pending_launch_desktop: None,
        touch_target_wayland: false,
        active_wayland_touch_slots: HashSet::new(),
        shade_capture_attempted: false,
        osk_bottom_inset: 0,
        warmed_user_services: false,
        renderer,
        bhoo_renderer: Rc::new(RefCell::new(bhoo_renderer)),
        bhoo_id: Id::new(),
        previous_frame_is_settings: false,
        previous_scene_command_count: 0,
        bhoo_commit: CommitCounter::default(),
        drm_compositor,
        output,
        phys_size,
        scale,
        frame_period_s,
        last_anim_time: 0.0,
        continuous_anim: false,
        needs_render: true,
        waiting_for_vblank: false,
        waiting_for_vblank_since: None,
        last_watchdog_ping: Instant::now(),
        screen_on: true,
        park_black_frame: false,
        lock_fade,
        lock_reveal: None,
        screen_off_transition: None,
        screen_dimmed: false,
        pending_backlight_on: None,
        dt2w_tap_down: None,
        dt2w_last_tap: None,
        last_activity: Instant::now(),
        power_pressed_at: None,
        power_long_handled: false,
        hw,
        perf: FramePerf::new(perf_enabled),
        bench_phase: bench_enabled.then(|| (0, Instant::now())),
        shell,
        notif_daemon,
        last_settings_sync: Instant::now(),
        last_desktop_reload: Instant::now(),
        ui,
        commands: Rc::new(RefCell::new(Vec::with_capacity(512))),
        compose_body: String::new(),
        input: PendingInput::default(),
        started: Instant::now(),
        #[cfg(feature = "remote_control")]
        remote_control: crate::remote_control::RemoteControl::from_env()?,
    };

    let (init_app_w, init_app_h) = state.app_viewport_size();
    state.hw.shiraz.set_wayland_env(
        &state.socket_name,
        init_app_w.max(120) as u32,
        init_app_h.max(120) as u32,
        state.scale,
    );
    // Sync persisted settings immediately on boot so Shiraz auto-starts warm if enabled.
    state.hw.sync_from_shell(&mut state.shell.sys, false, false, state.screen_on);

    loop_handle.insert_source(socket, |client_stream, _, state| {
        if let Err(err) = state
            .display_handle
            .insert_client(client_stream, Arc::new(ClientState::default()))
        {
            tracing::warn!("Failed to insert Wayland client: {err}");
        }
    })?;

    loop_handle.insert_source(
        Generic::new(display, Interest::READ, Mode::Level),
        |_, display, state| {
            unsafe {
                let _ = display.get_mut().dispatch_clients(state);
            }
            Ok(PostAction::Continue)
        },
    )?;

    if let Some(bus_fd) = notif_fd {
        loop_handle.insert_source(
            Generic::new(BorrowedBusFd(bus_fd), Interest::READ, Mode::Level),
            |_, _, state| {
                state.drain_notification_events();
                Ok(PostAction::Continue)
            },
        )?;
    }

    loop_handle.insert_source(drm_notifier, |event, metadata, state| match event {
        DrmEvent::VBlank(_crtc) => {
            let presented_wake = state.drm_compositor.frame_submitted().ok().flatten().flatten();
            state.waiting_for_vblank = false;
            state.waiting_for_vblank_since = None;
            if state.screen_on && presented_wake.is_some() && presented_wake == state.pending_backlight_on {
                state.pending_backlight_on = None;
                state.hw.set_backlight_on();
                let duration = LOCK_FADE_DURATION.mul_f32(state.shell.sys.anim_scale());
                if matches!(presented_wake, Some(BacklightWake::LockScreen(_)))
                    && state.lock_fade.is_some() && !duration.is_zero() {
                    state.lock_reveal = Some((Instant::now(), duration));
                    state.needs_render = true;
                }
            }
            let gpu_mhz = state.hw.current_gpu_mhz();
            state
                .perf
                .on_vblank(*metadata, state.frame_period_s * 1000.0, gpu_mhz);
            if state.screen_off_transition.as_ref().is_some_and(|transition| transition.black_queued) {
                let transition = state.screen_off_transition.take().unwrap();
                tracing::info!("Lock animation presented {} frames in {:.0}ms", transition.presented_frames, transition.started.elapsed().as_secs_f64() * 1000.0);
                state.sleep_and_lock(&transition.reason);
                return;
            }
            // Zero-gap continuous benchmark driver: chain next animation immediately on VBlank!
            if let Some((phase, _)) = state.bench_phase
                && !state.continuous_anim
            {
                let now = state.started.elapsed().as_secs_f64().max(state.last_anim_time);
                state.shell.drive_benchmark_step(phase, now);
                state.bench_phase = Some((phase + 1, Instant::now()));
                state.continuous_anim = true;
                state.needs_render = true;
            }
            if state.needs_render {
                state.render_frame();
            }
        }
        DrmEvent::Error(err) => {
            tracing::error!("DRM error (recovering scanout): {err}");
            state.drm_compositor.reset_state();
            state.waiting_for_vblank = false;
            state.waiting_for_vblank_since = None;
            if state.screen_on {
                
                state.needs_render = true;
            }
        }
    })?;

    loop_handle.insert_source(session_notifier, move |event, _, _state| match event {
        SessionEvent::PauseSession => {
            libinput_ctx.suspend();
            tracing::info!("Session paused");
        }
        SessionEvent::ActivateSession => {
            let _ = libinput_ctx.resume();
            tracing::info!("Session resumed");
        }
    })?;

    loop_handle.insert_source(libinput_backend, |event, _, state| {
        let logical_w = (state.phys_size[0] as f32 / state.scale).round();
        let logical_h = (state.phys_size[1] as f32 / state.scale).round();
        let size = Size::<i32, Logical>::from((logical_w as i32, logical_h as i32));

        if state.screen_off_transition.is_some() {
            return;
        }

        match event {
            // Double-Tap to Wake (DT2W) while screen is off
            InputEvent::TouchDown { event } if !state.screen_on => {
                #[cfg(feature = "remote_control")]
                state.remote_local_takeover();
                let pos = event.position_transformed(size);
                // Ignore accidental edge/palm touches
                if pos.x >= 24.0
                    && pos.x <= (logical_w as f64 - 24.0)
                    && pos.y >= 36.0
                    && pos.y <= (logical_h as f64 - 36.0)
                {
                    state.dt2w_tap_down = Some((i32::from(event.slot()), Instant::now(), [pos.x, pos.y]));
                }
            }
            InputEvent::TouchUp { event } if !state.screen_on => {
                let slot = i32::from(event.slot());
                if let Some((down_slot, t_down, down_pos)) = state.dt2w_tap_down
                    && down_slot == slot
                {
                    state.dt2w_tap_down = None;
                    let now = Instant::now();
                    if now.duration_since(t_down) <= Duration::from_millis(280) {
                        if let Some((prev_t, prev_pos)) = state.dt2w_last_tap
                            && now.duration_since(prev_t) <= Duration::from_millis(420)
                            && (down_pos[0] - prev_pos[0]).hypot(down_pos[1] - prev_pos[1]) <= 56.0
                        {
                            state.dt2w_last_tap = None;
                            tracing::info!("Double-tap to wake triggered");
                            state.wake_to_lockscreen();
                        } else {
                            state.dt2w_last_tap = Some((now, down_pos));
                        }
                    }
                }
            }
            InputEvent::TouchDown { event } if state.screen_on => {
                #[cfg(feature = "remote_control")]
                if state.remote_local_takeover() { return; }
                state.handle_touch_down(i32::from(event.slot()), event.position_transformed(size), event.time_msec());
            }
            InputEvent::TouchMotion { event } if state.screen_on => {
                state.handle_touch_motion(i32::from(event.slot()), event.position_transformed(size), event.time_msec());
            }
            InputEvent::TouchUp { event } if state.screen_on => {
                state.handle_touch_up(i32::from(event.slot()), event.time_msec());
            }
            InputEvent::TouchCancel { .. } => {
                state.handle_touch_cancel();
            }
            InputEvent::Keyboard { event } => {
                #[cfg(feature = "remote_control")]
                if event.state() == KeyState::Pressed { state.remote_local_takeover(); }
                let raw = event.key_code().raw();
                match (raw, event.state()) {
                    // Hardware Wakeup key (e.g. kernel DT2W KEY_WAKEUP=143 / XKB 151)
                    (143 | 151, KeyState::Pressed) if !state.screen_on => {
                        state.wake_to_lockscreen();
                    }
                    // Power button press/release: short press = sleep/wake, long press (>=650ms) = Power menu
                    (116 | 124, KeyState::Pressed) => {
                        state.last_activity = Instant::now();
                        state.power_pressed_at = Some(Instant::now());
                        state.power_long_handled = false;
                        if !state.screen_on {
                            state.wake_to_lockscreen();
                            state.power_long_handled = true;
                        } else if state.screen_dimmed {
                            state.screen_dimmed = false;
                            state.hw.set_backlight_on();
                        }
                    }
                    (116 | 124, KeyState::Released) => {
                        let long = state.power_long_handled;
                        state.power_pressed_at = None;
                        state.power_long_handled = false;
                        if !long && state.screen_on {
                            state.begin_screen_off("power key");
                        }
                    }
                    // Volume Up
                    (115 | 123, KeyState::Pressed) if state.screen_on => {
                        state.last_activity = Instant::now();
                        let t = state.started.elapsed().as_secs_f64();
                        state.hw.boost_gpu(Duration::from_millis(600));
                        state.input.push_key(bhoo::Key::PageUp, t);
                        state.needs_render = true;
                    }
                    // Volume Down
                    (114 | 122, KeyState::Pressed) if state.screen_on => {
                        state.last_activity = Instant::now();
                        let t = state.started.elapsed().as_secs_f64();
                        state.hw.boost_gpu(Duration::from_millis(600));
                        state.input.push_key(bhoo::Key::PageDown, t);
                        state.needs_render = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    })?;

    // Watchdog timer: systemd heartbeat, VBlank stuck recovery, power long-press menu,
    // VBlank backlight fallback, pre-warming, pending closes, GPU boost release, battery poll & auto-lock.
    loop_handle.insert_source(
        Timer::from_duration(Duration::from_millis(50)),
        |_, _, state| {
            #[cfg(feature = "remote_control")]
            state.poll_remote_control();

            // Systemd watchdog heartbeat (proves main compositor + input loop is responsive)
            if state.last_watchdog_ping.elapsed() >= Duration::from_secs(3) {
                state.last_watchdog_ping = Instant::now();
                sd_notify("WATCHDOG=1");
            }

            // Recover automatically if DRM ever misses a VBlank so screen/power button never wedge
            if state.waiting_for_vblank
                && state
                    .waiting_for_vblank_since
                    .is_some_and(|t| t.elapsed() >= Duration::from_millis(500))
            {
                tracing::warn!("DRM VBlank watchdog timeout (>500ms); resetting compositor scanout");
                state.drm_compositor.reset_state();
                state.waiting_for_vblank = false;
                state.waiting_for_vblank_since = None;
                if state.screen_on {
                    
                    state.needs_render = true;
                }
            }

            if state.screen_off_transition.as_ref().is_some_and(|transition| {
                transition.started.elapsed() >= transition.duration + Duration::from_millis(200)
            }) {
                // A failed flip must never leave the LCD on or the session unlocked.
                let transition = state.screen_off_transition.take().unwrap();
                tracing::warn!("Lock animation presentation timed out");
                state.sleep_and_lock(&transition.reason);
            }

            // One-shot background warm-up 1.5s after boot: starts SpectreOS Keyboard (`spectre-keyboard`) + pre-warms
            // Nautilus/GVFS/dconf so the very first GTK4 app launch is fast!
            if !state.warmed_user_services
                && state.started.elapsed() >= Duration::from_millis(1500)
            {
                state.prewarm_user_services();
            }

            // Safety fallback for VBlank-gated backlight wake (in case DRM skipped an identical flip)
            if state.screen_on
                && let Some(BacklightWake::LockScreen(t_req)) = state.pending_backlight_on
                && (state.lock_fade.is_none() || state.shell.sys.anim_scale() == 0.0)
                && t_req.elapsed() >= Duration::from_millis(45)
            {
                state.pending_backlight_on = None;
                state.hw.set_backlight_on();
            }

            // Enforce graceful SIGTERM fallback for apps that ignored xdg_toplevel.close
            let now_inst = Instant::now();
            let mut due_pids = Vec::new();
            state.pending_kills.retain(|(deadline, pid, oid)| {
                let still_open = state.wayland_apps.iter().any(|e| {
                    e.window
                        .toplevel()
                        .is_some_and(|t| t.wl_surface().id() == *oid)
                });
                if !still_open {
                    return false;
                }
                if now_inst >= *deadline {
                    due_pids.push(*pid);
                    return false;
                }
                true
            });
            if !due_pids.is_empty()
                && let Some(kill_bin) = find_in_path("kill")
            {
                for pid in due_pids {
                    tracing::info!("Sending SIGTERM fallback to unresponsive closed app pid={pid}");
                    let pid_s = pid.to_string();
                    spawn_detached(&kill_bin, &["-TERM", &pid_s]);
                }
            }

            // Automated animation stress benchmark when WISP_BENCH=1
            if let Some((phase, t_phase)) = state.bench_phase
                && t_phase.elapsed() >= Duration::from_millis(480)
            {
                let now = state.started.elapsed().as_secs_f64().max(state.last_anim_time);
                state.shell.drive_benchmark_step(phase, now);
                state.bench_phase = Some((phase + 1, Instant::now()));
                state.last_activity = Instant::now();
                state.needs_render = true;
            }

            // Long-press Power button (>= 650ms) opens Wisp's Power menu!
            if let Some(t0) = state.power_pressed_at
                && !state.power_long_handled
                && state.screen_on
                && t0.elapsed() >= Duration::from_millis(650)
            {
                state.power_long_handled = true;
                let t = state.started.elapsed().as_secs_f64();
                state.hw.play_haptic(HapticEffect::LongPress, &state.shell.sys);
                state.hw.boost_gpu(Duration::from_millis(800));
                state.input.push_key(bhoo::Key::End, t);
                state.needs_render = true;
            }

            // Drain any pending D-Bus notification traffic & handle expiration/timestamp ticks
            state.drain_notification_events();
            let notif_now = state.started.elapsed().as_secs_f64().max(state.last_anim_time);
            if state.shell.tick_notifications(notif_now) {
                state.process_host_commands();
                if state.screen_on {
                    state.needs_render = true;
                }
            }

            // Live-sync settings edited in standalone `spectre-settings` (~/.config/wisp/*.conf)
            if state.last_settings_sync.elapsed() >= Duration::from_millis(250) {
                state.last_settings_sync = Instant::now();
                if state.shell.sys.reload_persisted_if_changed() {
                    let power_off = matches!(state.shell.power_state, PowerState::PoweredOff);
                    let reboot = matches!(state.shell.power_state, PowerState::Rebooting(_));
                    state.hw.sync_from_shell(
                        &mut state.shell.sys,
                        power_off,
                        reboot,
                        state.screen_on,
                    );
                    if state.screen_on {
                        state.needs_render = true;
                    }
                }
            }

            // Hot-reload XDG `.desktop` applications (e.g. newly installed Shiraz Android APKs)
            if state.last_desktop_reload.elapsed() >= Duration::from_secs(1) {
                state.last_desktop_reload = Instant::now();
                if state.shell.refresh_desktop_apps_if_changed() && state.screen_on {
                    state.needs_render = true;
                }
            }

            // Poll platform-agnostic hardware connectors (always-on Touch ID even while screen OFF,
            // Notification/Charging LED, ALS, Wi-Fi, Bluetooth, Battery)
            let has_unread = state.shell.has_unread_notifications();
            let pause_fp = state.shell.is_fingerprint_lab_foreground();
            let (hw_changed, hw_events) = state.hw.poll(
                &mut state.shell.sys,
                state.screen_on,
                state.shell.lockscreen.locked || state.screen_off_transition.is_some(),
                has_unread,
                pause_fp,
            );
            for ev in hw_events {
                match ev {
                    HardwareEvent::FingerprintStatusChanged(st) => {
                        state.shell.lockscreen.on_touch_id_status(st);
                        if state.screen_on {
                            state.needs_render = true;
                        }
                    }
                    HardwareEvent::FingerprintMatched => {
                        let was_screen_off = !state.screen_on || state.pending_backlight_on.is_some();
                        tracing::info!(
                            "Touch ID fingerprint match (screen_was_off={was_screen_off})"
                        );
                        state.last_activity = Instant::now();
                        state.lock_reveal = None;
                        let interrupted_fade = state.screen_off_transition.take().is_some();
                        let act = if interrupted_fade && !state.shell.lockscreen.locked {
                            // The outgoing app is still on screen. Cancel the pending black
                            // flip as a sleep trigger and redraw the current scene.
                            state.hw.play_haptic(crate::services::haptics::HapticEffect::Confirm, &state.shell.sys);
                            crate::lockscreen::LockAction::None
                        } else {
                            state.shell.lockscreen.fingerprint_unlock(was_screen_off)
                        };
                        for eff in state.shell.lockscreen.pending_haptics.drain(..) {
                            state.hw.play_haptic(eff, &state.shell.sys);
                        }
                        let now = state.started.elapsed().as_secs_f64().max(state.last_anim_time);
                        match act {
                            crate::lockscreen::LockAction::OpenNotification(id) => {
                                state.shell.invoke_notification_action(id, "default", None, now);
                            }
                            crate::lockscreen::LockAction::OpenShortcut(shortcut) => state.shell.open_lock_shortcut(shortcut, now),
                            _ => {}
                        }
                        state.process_host_commands();
                        if was_screen_off {
                            state.wake_unlocked();
                        } else {
                            if state.screen_dimmed {
                                state.screen_dimmed = false;
                                state.hw.set_backlight_on();
                            }
                            state.hw.boost_gpu(Duration::from_millis(800));
                            state.needs_render = true;
                        }
                    }
                    HardwareEvent::FingerprintRejected(msg) => {
                        let was_screen_off = !state.screen_on;
                        tracing::info!("Touch ID mismatch (screen_was_off={was_screen_off}): {msg}");
                        state.last_activity = Instant::now();
                        if was_screen_off {
                            state.wake_to_lockscreen();
                        } else if state.screen_dimmed {
                            state.screen_dimmed = false;
                            state.hw.set_backlight_on();
                        }
                        state.shell.lockscreen.on_touch_id_rejected(msg);
                        for eff in state.shell.lockscreen.pending_haptics.drain(..) {
                            state.hw.play_haptic(eff, &state.shell.sys);
                        }
                        state.needs_render = true;
                    }
                    HardwareEvent::AmbientLightLux(_) => {
                        if state.screen_on {
                            state.needs_render = true;
                        }
                    }
                }
            }

            if state.screen_on {
                if !state.needs_render && !state.continuous_anim {
                    state.hw.release_gpu_boost(false);
                }
                if hw_changed {
                    state.needs_render = true;
                }
                if let Some(deadline) = state.ui.next_wakeup()
                    && state.started.elapsed().as_secs_f64() >= deadline
                {
                    state.needs_render = true;
                }
                if state.bench_phase.is_none()
                    && let Some(timeout_secs) = state.shell.screen_timeout_secs()
                {
                    let effective_secs = if state.shell.lockscreen.locked {
                        timeout_secs.min(15) as f32
                    } else {
                        timeout_secs as f32
                    };
                    let idle = state.last_activity.elapsed().as_secs_f32();
                    if idle >= effective_secs {
                        state.begin_screen_off(&format!("auto-lock {}s", effective_secs as u32));
                    } else if effective_secs > 6.0
                        && idle >= effective_secs - 5.0
                        && !state.screen_dimmed
                    {
                        state.screen_dimmed = true;
                        state.hw.set_backlight_dimmed();
                    }
                }
            }
            TimeoutAction::ToDuration(Duration::from_millis(50))
        },
    )?;

    state.render_frame();
    sd_notify("READY=1\nWATCHDOG=1");
    event_loop.run(None, &mut state, |state| {
        if state.needs_render && !state.waiting_for_vblank {
            state.render_frame();
        }
        let _ = state.display_handle.flush_clients();
    })?;
    state.hw.shiraz.shutdown();

    Ok(())
}

#[cfg(feature = "remote_control")]
include!("remote_control/integration.rs");
