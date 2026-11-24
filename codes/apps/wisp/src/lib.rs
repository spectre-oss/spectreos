//! Wisp — lightweight SpectreOS System UI & Launcher built on `libbhoo`.
//! Covers Home Workspace / Dock / App Drawer / Widgets / Wallpaper & Style,
//! two-stage Notification Shade / Quick Settings / Volume / Power,
//! Multitasking (Overview / Split-Screen / Freeform), System Settings, and built-in apps.

use bhoo::*;
use services::time::{ClockReading, TimeService};

pub use spectre_settings as settings;
pub use spectre_settings::{
    ClockPos, DeviceRuntimeInfo, DnsMode, ID_DNS_HOSTNAME, ID_SETTINGS_SEARCH, InstalledAppInfo,
    LockConfig, LockMethod, LockScreenNotifMode, NavMode, NotificationHistoryEntry, Palette,
    RingerMode, SystemState, paint_h_slider, paint_pref_switch, paint_radio, paint_switch,
};

mod apps;
mod icons;
mod settings_apps;
mod home_layout;
use home_layout::{HomeApp, HomeCell, HomeLayout};
mod motion;
pub mod backends;
pub mod freedesktop;
pub mod lockscreen;
mod wallpaper;
mod wallpaper_ui;
mod qs_editor;
pub mod notifications;
pub mod services;
#[cfg(feature = "compositor")]
pub mod compositor;
#[cfg(feature = "compositor")]
#[doc(hidden)]
pub mod capture;

/// Optional host-supplied external apps (for example, `bhoo-web`'s XML Contacts and Mail demos)
/// rendered inside Wisp's live app surface, split-screen panes, and freeform windows.
pub trait EmbeddedApps: Default {
    fn render_contacts(&mut self, f: &mut Frame<'_>, bounds: Rect) {
        let t = f.theme();
        f.paint_rect(bounds, 0.0, t.background);
        f.within(bounds.inset(24.0), |f| {
            f.empty_state(
                0x5749_5350_0001,
                "Contacts",
                "Standalone Wisp session — connect a Contacts provider to browse entries.",
                EmptyStateOptions {
                    icon: Some(MaterialIcon::Person),
                    action_label: None,
                    error: false,
                },
            );
        });
    }

    fn open_new_contact(&mut self, _f: &mut Frame<'_>) {}

    fn render_gmail(&mut self, f: &mut Frame<'_>, bounds: Rect, _compose_body: &mut String) {
        let t = f.theme();
        f.paint_rect(bounds, 0.0, t.background);
        f.within(bounds.inset(24.0), |f| {
            f.empty_state(
                0x5749_5350_0002,
                "Mail",
                "Standalone Wisp session — connect a Mail provider to browse messages.",
                EmptyStateOptions {
                    icon: Some(MaterialIcon::Mail),
                    action_label: None,
                    error: false,
                },
            );
        });
    }

    fn edit_text_for(&self, _id: u64) -> &str {
        ""
    }

    /// Update an embedded app's editable field text. Returns `true` when the Mail compose body changed.
    fn set_edit_text(&mut self, _id: u64, _value: &str) -> bool {
        false
    }
}

/// Default standalone host adapter used when Wisp runs without external demo apps.
#[derive(Default)]
pub struct NoopEmbeddedApps;

impl EmbeddedApps for NoopEmbeddedApps {}

/// Primary SpectreOS launcher shell type (alias of `LauncherDemo<E>` for compatibility).
pub type WispShell<E = NoopEmbeddedApps> = LauncherDemo<E>;

pub const ID_DRAWER_SEARCH: u64 = 0x4C01_0000_0000_F001;
pub const ID_FILES_SEARCH: u64 = spectre_files::SEARCH_FIELD_ID;
pub const ID_FILES_EDIT: u64 = spectre_files::EDIT_FIELD_ID;
pub const ID_WIDGETS_SEARCH: u64 = 0x4C01_0000_0000_F002;
pub const ID_BROWSER_URL: u64 = 0x4C05_0000_0000_F005;
pub const ID_CHAT_COMPOSER: u64 = 0x4C05_0000_0000_F006;

const SHELL_ID: u64 = 0x4C00_0000_0000_0000;
const LAUNCHER_ID: u64 = 0x4C01_0000_0000_0000;
const SHADE_ID: u64 = 0x4C02_0000_0000_0000;
const QS_COMPACT_TILE_HEIGHT: f32 = 58.0;
const QS_MIN_TILE_HEIGHT: f32 = 32.0;
const QS_ROW_GAP: f32 = 8.0;
const MULTI_ID: u64 = 0x4C03_0000_0000_0000;
const SETTINGS_ID: u64 = 0x4C04_0000_0000_0000;
const APP_ID_BASE: u64 = 0x4C05_0000_0000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppId {
    ArcaneChat,
    Browser,
    Calculator,
    Calendar,
    Camera,
    Clock,
    Contacts,
    DeltaChat,
    FDroid,
    Files,
    Gallery,
    Gmail,
    Messaging,
    Music,
    Phone,
    Recorder,
    Settings,
    Telegram,
}

impl AppId {
    pub const ALL: &'static [Self] = &[
        Self::ArcaneChat,
        Self::Browser,
        Self::Calculator,
        Self::Calendar,
        Self::Camera,
        Self::Clock,
        Self::Contacts,
        Self::DeltaChat,
        Self::FDroid,
        Self::Files,
        Self::Gallery,
        Self::Gmail,
        Self::Messaging,
        Self::Music,
        Self::Phone,
        Self::Recorder,
        Self::Settings,
        Self::Telegram,
    ];

    fn is_launcher_placeholder(self) -> bool {
        matches!(
            self,
            Self::FDroid | Self::ArcaneChat | Self::Gallery | Self::Music
                | Self::Telegram | Self::Recorder | Self::Contacts
                | Self::Gmail | Self::Browser
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ArcaneChat => "ArcaneChat",
            Self::Browser => "Browser",
            Self::Calculator => "Calculator",
            Self::Calendar => "Calendar",
            Self::Camera => "Camera",
            Self::Clock => "Clock",
            Self::Contacts => "Contacts",
            Self::DeltaChat => "Delta Chat",
            Self::FDroid => "F-Droid",
            Self::Files => "Files",
            Self::Gallery => "Gallery",
            Self::Gmail => "Gmail",
            Self::Messaging => "Messages",
            Self::Music => "Music",
            Self::Phone => "Phone",
            Self::Recorder => "Recorder",
            Self::Settings => "Settings",
            Self::Telegram => "Telegram",
        }
    }

    pub fn package(self) -> &'static str {
        match self {
            Self::ArcaneChat => "chat.arcane.messenger",
            Self::Browser => "org.lineageos.jelly",
            Self::Calculator => "com.android.calculator2",
            Self::Calendar => "org.spectreos.Calendar",
            Self::Camera => "org.lineageos.aperture",
            Self::Clock => "com.android.deskclock",
            Self::Contacts => "com.android.contacts",
            Self::DeltaChat => "chat.delta.android",
            Self::FDroid => "org.fdroid.fdroid",
            Self::Files => "com.android.documentsui",
            Self::Gallery => "org.lineageos.glimpse",
            Self::Gmail => "com.google.android.gm",
            Self::Messaging => "com.android.messaging",
            Self::Music => "org.lineageos.twelve",
            Self::Phone => "com.android.dialer",
            Self::Recorder => "org.lineageos.recorder",
            Self::Settings => "com.android.settings",
            Self::Telegram => "org.telegram.messenger",
        }
    }

    pub fn version(self) -> &'static str {
        match self {
            Self::Contacts => "1.7.34",
            Self::Gmail => "2026.11.02",
            Self::Settings => "13 (LineageOS 20)",
            Self::Clock => "13.0",
            Self::FDroid => "1.21.0",
            Self::ArcaneChat => "2.4.1",
            _ => "13.0.4",
        }
    }

    pub fn storage_mb(self) -> u32 {
        match self {
            Self::Gallery => 148,
            Self::Telegram => 214,
            Self::Gmail => 164,
            Self::ArcaneChat => 96,
            Self::Browser => 112,
            Self::Camera => 88,
            Self::FDroid => 64,
            Self::Music => 72,
            Self::DeltaChat => 58,
            Self::Settings => 42,
            Self::Contacts => 34,
            _ => 24,
        }
    }

    pub fn shortcut_label(self) -> Option<&'static str> {
        match self {
            Self::Contacts => Some("Add contact"),
            Self::Gmail => Some("Compose email"),
            Self::Clock => Some("New alarm"),
            Self::Settings => Some("Battery"),
            Self::Camera => Some("Take photo"),
            Self::Phone => Some("New call"),
            Self::ArcaneChat | Self::Messaging | Self::DeltaChat | Self::Telegram => {
                Some("New message")
            }
            _ => None,
        }
    }

    pub fn uses_light_status_ink(self, dark_theme: bool) -> bool {
        match self {
            Self::Clock | Self::Camera | Self::Contacts | Self::Recorder => true,
            Self::Gmail => false,
            _ => dark_theme,
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&a| a == self).unwrap_or(0)
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        Self::ALL.get(idx).copied()
    }
}

#[allow(dead_code)]
pub use bhoo::SystemIcon as SysIcon;

#[allow(dead_code)]
/// Commands emitted by `WispShell` to the native Wayland compositor host for real app lifecycle
/// and Freedesktop Desktop Notifications v1.3 D-Bus signals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostCommand {
    LaunchOrFocus(AppId),
    JoinWifi(settings::state::WifiNetwork),
    ConnectBluetooth(String, bool),
    OpenFile(std::path::PathBuf),
    LaunchFilesWindow(Option<std::path::PathBuf>),
    ShareFiles(Vec<std::path::PathBuf>),
    LaunchDesktopApp(String, Option<&'static str>),
    ShirazLaunchPackage {
        package: String,
        desktop_id: Option<String>,
    },
    ShirazStopPackage(String),
    ShirazStopAll,
    ShirazRefresh,
    MinimizeAll,
    CloseApp(AppId),
    CloseDesktopApp(String),
    CloseAllApps,
    SendBack,
    NotifyClosed { id: u32, reason: u32 },
    NotifyActionInvoked { id: u32, action_key: String },
}

pub enum ShellAction {
    None,
    OpenApp(AppId),
    FocusTextField(u64),
    BlurTextField,
    JoinWifi(settings::state::WifiNetwork),
    ConnectBluetooth(String, bool),
    OpenFile(std::path::PathBuf),
    LaunchFilesWindow(Option<std::path::PathBuf>),
    ShareFiles(Vec<std::path::PathBuf>),
    OpenDesktopApp(usize),
    AdvancedRadioSettings(&'static str),
    OpenSettingsSub(settings::SettingsPage),
    OpenAppInfo(AppId),
    ShowToast(String),
    PostNotification {
        app: String,
        title: String,
        body: String,
    },
    ClearNotificationHistory,
    OpenVolume,
    GoHome,
    GoBack,
    LockNow,
    ShirazLaunchPackage(String),
    ShirazStopPackage(String),
    ShirazStopAll,
    ShirazRefresh,
}

#[derive(Clone, Debug)]
pub(crate) struct NotificationItem {
    pub(crate) id: u32,
    pub(crate) app: String,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) big_text: String,
    pub(crate) time_label: String,
    pub(crate) silent: bool,
    pub(crate) ongoing: bool,
    pub(crate) urgency: u8,
    pub(crate) category: Option<String>,
    pub(crate) desktop_entry: Option<String>,
    pub(crate) resident: bool,
    #[allow(dead_code)]
    pub(crate) transient: bool,
    pub(crate) actions: Vec<(String, String)>,
    pub(crate) icon_image: Option<bhoo::Image>,
    pub(crate) expires_at: Option<f64>,
    pub(crate) arrived_at: f64,
    pub(crate) heads_up_until: Option<f64>,
    pub(crate) from_dbus: bool,
    pub(crate) expanded: bool,
    pub(crate) expand_anim: Anim,
    pub(crate) controls_open: bool,
    pub(crate) controls_anim: Anim,
    pub(crate) swipe_anim: Anim,
    pub(crate) dismiss_anim: Anim,
    pub(crate) dismissing: bool,
    pub(crate) ripple: Option<([f32; 2], f64, bool)>, // (touch_pos, start_time, held)
    pub(crate) snoozed: bool,
    pub(crate) has_reply: bool,
    pub(crate) target_app: Option<AppId>,
}

impl NotificationItem {
    fn shade_card_height(&self, prominent: bool) -> f32 {
        let collapsed = if prominent { 108.0 } else { 86.0 };
        let expanded = lerp(collapsed, 150.0, self.expand_anim.value.clamp(0.0, 1.0));
        lerp(expanded, 126.0, self.controls_anim.value.clamp(0.0, 1.0))
    }

    fn shade_card_advance(&self, height: f32) -> f32 {
        let visible = if self.dismissing {
            self.dismiss_anim.value.clamp(0.0, 1.0)
        } else {
            1.0
        };
        (height + 3.0) * visible
    }
}

#[derive(Clone, Copy, Debug)]
enum LaunchTarget {
    Builtin(AppId),
    Desktop(usize),
}

#[derive(Clone, Copy, Debug)]
enum AppTransition {
    None,
    /// Expanding from an exact Home/Dock/Drawer icon OR Overview card OR Notification card!
    Opening {
        app: AppId,
        origin: Rect,
        from_icon: bool,
        from_shade: bool,
        from_overview: bool,
        themed_icon: bool,
        start: f64,
        duration: f64,
    },
    OpeningDesktop {
        origin: Rect,
        from_icon: bool,
        from_overview: bool,
        start: f64,
        duration: f64,
    },
    /// Shrinking smoothly back into the exact Home/Dock icon (matching strip-02-close-contacts.png!)
    Closing {
        app: AppId,
        from_rect: Rect,
        from_rad: f32,
        target_icon: Option<Rect>,
        themed_icon: bool,
        start: f64,
        duration: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QsTileKind {
    Wifi,
    Bluetooth,
    Flashlight,
    Shiraz,
    Alarm,
    Airplane,
    DeviceControls,
    AutoRotate,
    BatterySaver,
    DarkTheme,
    Dnd,
    ScreenRecord,
    NightLight,
    CameraAccess,
    MicAccess,
    Aod,
    Hotspot,
}

impl QsTileKind {
    const DEFAULT_ORDER: &'static [Self] = &[
        Self::Wifi,
        Self::Bluetooth,
        Self::Flashlight,
        Self::Shiraz,
        Self::Alarm,
        Self::Airplane,
        Self::AutoRotate,
        Self::BatterySaver,
        Self::DarkTheme,
        Self::Dnd,
        Self::ScreenRecord,
        Self::NightLight,
        Self::CameraAccess,
        Self::MicAccess,
        Self::Aod,
        Self::Hotspot,
        Self::DeviceControls,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TaskEntry {
    Single(AppId),
    SplitPair(AppId, AppId),
    Desktop(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShellMode {
    Home,
    Fullscreen(AppId),
    FullscreenDesktop(usize),
    Overview,
    SplitSelect(AppId),
    Split {
        top: AppId,
        bottom: AppId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LauncherSub {
    None,
    WallpaperStyle,
    WallpaperPicker,
    WallpaperFiles,
    WallpaperPreview,
    WallpaperSettings(AppearancePage),
    HomeSettings,
    HiddenApps,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppearancePage {
    Grid,
    Shortcuts,
    Icon,
    Font,
    Shape,
}

impl AppearancePage {
    const ALL: [Self; 5] = [Self::Grid, Self::Shortcuts, Self::Icon, Self::Font, Self::Shape];

    fn title(self) -> &'static str {
        match self {
            Self::Grid => "App grid",
            Self::Shortcuts => "Shortcuts",
            Self::Icon => "Icon style",
            Self::Font => "Font",
            Self::Shape => "Icon shape",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum HomePopup {
    None,
    BlankOptions([f32; 2]),
    AppShortcuts { app: AppId, anchor: [f32; 2] },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PowerState {
    Running,
    ShuttingDown(u64),
    PoweredOff,
    Rebooting(u64),
}

#[derive(Clone, Copy, Debug)]
struct FreeformWin {
    app: AppId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Anim {
    value: f32,
    from: f32,
    target: f32,
    start: f64,
    duration: f64,
    opening: bool,
}

impl Anim {
    fn at(v: f32) -> Self {
        Self {
            value: v,
            from: v,
            target: v,
            start: 0.0,
            duration: 0.0,
            opening: true,
        }
    }

    fn set(&mut self, target: f32, now: f64, open_dur: f64, close_dur: f64) {
        if (self.target - target).abs() > 0.0001 {
            self.from = self.value;
            self.target = target;
            self.start = now;
            self.opening = target >= self.from;
            self.duration = if self.opening { open_dur } else { close_dur };
        }
    }

    fn snap(&mut self, v: f32) {
        self.value = v;
        self.from = v;
        self.target = v;
        self.duration = 0.0;
    }

    fn step(&mut self, now: f64, reduced: bool, duration_scale: f64) -> bool {
        let easing = if self.opening { emphasized_decelerate } else { emphasized_accelerate };
        self.step_with_easing(now, reduced, duration_scale, easing)
    }

    fn step_with_easing(
        &mut self,
        now: f64,
        reduced: bool,
        duration_scale: f64,
        easing: fn(f32) -> f32,
    ) -> bool {
        if (self.value - self.target).abs() <= 0.0005 {
            self.value = self.target;
            return false;
        }
        let duration = self.duration * duration_scale;
        if reduced || duration <= 0.0 {
            self.value = self.target;
            return false;
        }
        let raw = ((now - self.start) / duration).clamp(0.0, 1.0) as f32;
        let eased = easing(raw);
        self.value = self.from + (self.target - self.from) * eased;
        if raw >= 1.0 {
            self.value = self.target;
            false
        } else {
            true
        }
    }
}

pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0 || x >= 1.0 {
        return x;
    }
    let mut t = x;
    for _ in 0..6 {
        let u = 1.0 - t;
        let bx = 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t;
        let dbx = 3.0 * u * u * x1 + 6.0 * u * t * (x2 - x1) + 3.0 * t * t * (1.0 - x2);
        if dbx.abs() < 1e-6 {
            break;
        }
        t = (t - (bx - x) / dbx).clamp(0.0, 1.0);
    }
    let u = 1.0 - t;
    (3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t).clamp(0.0, 1.0)
}

pub fn emphasized_decelerate(t: f32) -> f32 {
    cubic_bezier(0.05, 0.7, 0.1, 1.0, t)
}

pub fn emphasized_accelerate(t: f32) -> f32 {
    cubic_bezier(0.3, 0.0, 0.8, 0.15, t)
}

const TOUCH_SLOP: f32 = 7.0;

pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_rect(a: Rect, b: Rect, tx: f32, ty: f32) -> Rect {
    Rect {
        x: lerp(a.x, b.x, tx),
        y: lerp(a.y, b.y, ty),
        width: lerp(a.width, b.width, tx),
        height: lerp(a.height, b.height, ty),
    }
}

pub(crate) fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if (edge1 - edge0).abs() < 1e-6 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[derive(Clone, Copy, Default)]
struct VelocitySample {
    t: f64,
    pos: [f32; 2],
}

#[derive(Clone, Copy, Default)]
struct VelocityTracker {
    samples: [VelocitySample; 8],
    len: usize,
}

impl VelocityTracker {
    fn clear(&mut self) {
        self.len = 0;
    }

    fn push(&mut self, t: f64, pos: [f32; 2]) {
        if self.len > 0 && (t - self.samples[self.len - 1].t).abs() < 1e-4 {
            self.samples[self.len - 1] = VelocitySample { t, pos };
            return;
        }
        if self.len < self.samples.len() {
            self.samples[self.len] = VelocitySample { t, pos };
            self.len += 1;
        } else {
            self.samples.copy_within(1.., 0);
            self.samples[self.len - 1] = VelocitySample { t, pos };
        }
    }

    fn velocity(&self, now: f64) -> [f32; 2] {
        if self.len < 2 {
            return [0.0, 0.0];
        }
        let last = self.samples[self.len - 1];
        if now - last.t > 0.11 {
            return [0.0, 0.0];
        }
        let mut first = self.samples[0];
        for i in (0..self.len - 1).rev() {
            first = self.samples[i];
            if last.t - first.t >= 0.08 {
                break;
            }
        }
        let dt = (last.t - first.t) as f32;
        if dt < 0.004 {
            return [0.0, 0.0];
        }
        [
            ((last.pos[0] - first.pos[0]) / dt).clamp(-5000.0, 5000.0),
            ((last.pos[1] - first.pos[1]) / dt).clamp(-5000.0, 5000.0),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScrollTarget {
    Drawer = 0,
    ShadeNotifs = 1,
    Widgets = 2,
    WallpaperStyle = 3,
    HomeSettings = 4,
    QsEdit = 5,
    Settings = 6,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PressContext {
    None,
    QsEditTile {
        idx: usize,
    },
    EdgeBack {
        right_edge: bool,
    },
    StatusBar,
    NavBar {
        from_app: bool,
    },
    ShadeBody {
        in_notifs: bool,
        notif: Option<(u32, bool)>, // (id, ongoing)
    },
    QsGrid,
    HomeBlank,
    HomeIcon {
        ws_idx: usize,
        anchor: [f32; 2],
        cell_center: [f32; 2],
    },
    DockIcon {
        app: AppId,
        anchor: [f32; 2],
    },
    DrawerBody {
        in_header: bool,
    },
    WidgetsBody {
        in_header: bool,
    },
    OverviewCard {
        task_idx: Option<usize>,
        step: f32,
        max_scroll: f32,
    },
    ScrollList {
        target: ScrollTarget,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ActiveGesture {
    None,
    EdgeBack {
        right_edge: bool,
        anchor_x: f32,
        y: f32,
        pull: f32,
        committed: bool,
    },
    ShadeScrub {
        start_stage: u8,
        start_val: f32,
        anchor_y: f32,
        span: f32,
    },
    DrawerScrub {
        start_val: f32,
        anchor_y: f32,
        span: f32,
    },
    WidgetsScrub {
        start_val: f32,
        anchor_y: f32,
        span: f32,
    },
    NavGesture {
        origin: [f32; 2],
        delta: [f32; 2],
        from_app: bool,
        hold_time: f64,
    },
    OverviewHorizontal {
        start_scroll: f32,
        anchor_x: f32,
        step: f32,
        max_scroll: f32,
    },
    OverviewVertical {
        task_idx: usize,
        anchor_y: f32,
        offset_y: f32,
        original_card: Rect,
    },
    QsPageSwipe {
        start_page: f32,
        anchor_x: f32,
        width: f32,
    },
    NotifSwipe {
        notif_id: u32,
        ongoing: bool,
        anchor_x: f32,
        offset_x: f32,
    },
    HomeIconDrag {
        ws_idx: Option<usize>,
        pos: [f32; 2],
        offset: [f32; 2],
    },
    HomePageSwipe {
        start_page: f32,
        anchor_x: f32,
        width: f32,
    },
    QsEditDrag {
        idx: usize,
        pos: [f32; 2],
    },
    ListScroll {
        target: ScrollTarget,
        last_y: f32,
        max_scroll: f32,
    },
    SplitDivider,
    FreeformMove {
        offset: [f32; 2],
    },
    FreeformResize {
        origin_ptr: [f32; 2],
        origin_size: [f32; 2],
    },
    BrightnessSlider {
        bar: Rect,
    },
    VolumeSlider {
        channel: u8,
        track: Rect,
    },
}

/// C1 velocity-matched cubic Hermite deceleration glide for Android 13 Overview / Recents:
/// matches exact finger release velocity `start_vel` at `t=0` and smoothly decelerates to
/// `v=0` at `target_pos` over `duration` (allowing slow short drags to settle back to the same
/// card and fast flicks to coast smoothly across multiple cards with zero velocity jump!).
#[derive(Clone, Copy, Debug)]
struct OverviewGlide {
    start_time: f64,
    duration: f32,
    start_pos: f32,
    start_vel: f32,
    target_pos: f32,
}

impl OverviewGlide {
    fn start(start_pos: f32, target_pos: f32, start_vel: f32, now: f64) -> Self {
        let dist = target_pos - start_pos;
        let abs_d = dist.abs();
        let duration = if abs_d < 3.0 {
            0.16
        } else {
            ((2.2 * abs_d) / (start_vel.abs().max(260.0) + 160.0)).clamp(0.32, 0.68)
        };
        // Clamp Hermite tangent `v0 * dur` so glide is strictly monotone with zero reverse hump
        let max_tan = abs_d * 2.55;
        let tan = if dist.abs() > 1.0 && start_vel.signum() == dist.signum() {
            (start_vel * duration).clamp(-max_tan, max_tan)
        } else if dist.abs() > 1.0 {
            // Finger flicked slightly forward then settled back: gentle initial drift before returning
            (start_vel * duration).clamp(-abs_d * 0.45, abs_d * 0.45)
        } else {
            0.0
        };
        Self {
            start_time: now,
            duration,
            start_pos,
            start_vel: tan / duration.max(0.001),
            target_pos,
        }
    }

    fn sample(&self, now: f64, reduced: bool) -> (f32, bool) {
        if reduced || self.duration <= 0.001 {
            return (self.target_pos, false);
        }
        let u = ((now - self.start_time) as f32 / self.duration).clamp(0.0, 1.0);
        if u >= 1.0 {
            return (self.target_pos, false);
        }
        let u2 = u * u;
        let u3 = u2 * u;
        let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
        let h10 = u3 - 2.0 * u2 + u;
        let h01 = -2.0 * u3 + 3.0 * u2;
        let pos = h00 * self.start_pos + h10 * (self.start_vel * self.duration) + h01 * self.target_pos;
        (pos, true)
    }
}

pub fn text_style(size: f32, weight: u16, align: u8) -> TextStyle {
    TextStyle {
        size,
        weight,
        align,
        ..TextStyle::default()
    }
}

pub fn format_clock_time(now: f64, base_h: u32, base_m: u32) -> String {
    let mins = base_m + (now.max(0.0) as u32 / 60);
    let h = ((base_h - 1 + mins / 60) % 12) + 1;
    let m = mins % 60;
    format!("{h}:{m:02}")
}

pub struct LauncherDemo<E: EmbeddedApps = NoopEmbeddedApps> {
    pub(crate) sys: SystemState,
    pub lockscreen: lockscreen::LockScreen,
    pub(crate) desktop_registry: freedesktop::DesktopRegistry,
    pub(crate) native_compositor: bool,
    pub(crate) active_wayland_app: Option<AppId>,
    pub(crate) active_wayland_desktop: Option<usize>,
    wayland_surface_ready: bool,
    wayland_launch_frame: Option<Image>,
    wayland_shade_frame: Option<Image>,
    wayland_launch_capture_attempted: bool,
    wayland_task_frames: std::collections::HashMap<String, Image>,
    pub(crate) host_commands: Vec<HostCommand>,
    pub(crate) pending_haptics: Vec<services::HapticEffect>,
    pub(crate) pending_touch_id_retry: bool,
    pub(crate) mode: ShellMode,
    pub(crate) tasks: Vec<TaskEntry>,
    freeform: Option<FreeformWin>,
    split_ratio: f32,

    // Embedded host-supplied & built-in apps (boxed to keep WASM/thread stack frames tiny!)
    embedded: E,
    gmail_body_pending: Option<String>,
    settings_app: Box<settings::SettingsApp>,
    time_service: TimeService,
    clock: ClockReading,
    builtin_apps: Box<apps::BuiltInApps>,

    // Launcher states
    home_icons: Vec<HomeCell>,
    home_icon_pages: Vec<usize>,
    home_page: usize,
    home_page_count: usize,
    home_page_anim: Anim,
    home_reorder_anim: Anim,
    home_stretch: EdgeStretch,
    launcher_sub: LauncherSub,
    prev_launcher_sub: LauncherSub,
    launcher_sub_push: bool,
    launcher_sub_anim: Anim,
    wallpaper_tab_basic: bool,
    drawer_open: bool,
    drawer_anim: Anim,
    drawer_scroll: f32,
    drawer_search: String,
    ime_target: Option<u64>,
    ime_anim: Anim,
    ime_shift: bool,
    ime_symbols: bool,
    ime_cursor: usize,
    ime_pending_open: bool,
    last_ime_field_rect: Rect,
    widgets_open: bool,
    widgets_anim: Anim,
    widgets_scroll: f32,
    widgets_search: String,
    widgets_expanded: [bool; 4],
    wallpaper_scroll: f32,
    wallpaper_picker_scroll: f32,
    wallpaper_color_page: usize,
    wallpaper_shortcut_side: usize,
    wallpapers: wallpaper::Library,
    wallpaper_files: spectre_files::Browser,
    wallpaper_pending: Option<wallpaper::Pending>,
    wallpaper_load: Option<std::sync::mpsc::Receiver<std::io::Result<wallpaper::Source>>>,
    home_settings_scroll: f32,
    home_popup: HomePopup,
    last_home_popup: HomePopup,
    popup_anim: Anim,
    app_anim: Anim,
    app_transition: AppTransition,
    settings_return_snapshot: Option<Image>,
    preview_transition_t: Option<f32>,
    preview_edge_back: Option<(bool, f32, f32)>, // (right_edge, y_frac, pull_dp)
    last_phone_rect: Rect,

    // Notification Shade & QS
    pub(crate) shade_stage: u8, // 0=closed, 1=shade, 2=expanded QS
    pub(crate) shade_anim: Anim,
    notification_sheet_bounce: Anim,
    shade_scroll: f32,
    qs_page: usize,
    qs_page_anim: Anim,
    qs_tiles: Vec<QsTileKind>,
    qs_edit_open: bool,
    qs_edit_anim: Anim,
    qs_edit_origin: [f32; 2],
    qs_edit_scroll: f32,
    qs_edit_drag: Option<(usize, [f32; 2])>,
    qs_edit_last_frame: Option<f64>,
    active_apps_open: bool,
    active_bg_apps: Vec<(AppId, &'static str)>,
    notifications: Vec<NotificationItem>,
    notif_history: Vec<NotificationHistoryEntry>,
    settings_apps: Option<settings_apps::SettingsApps>,
    device_info: DeviceRuntimeInfo,
    last_heads_up_rect: Option<Rect>,
    next_notif_id: u32,

    // Power & Volume HUDs
    pub(crate) power_open: bool,
    pub(crate) power_anim: Anim,
    pub(crate) power_state: PowerState,
    flash_until: f64,
    pub(crate) vol_open: bool,
    vol_expanded: bool,
    pub(crate) vol_anim: Anim,
    vol_expand_anim: Anim,
    pub(crate) vol_hide_at: f64,

    // Overview & Multitasking
    overview_scroll: f32,
    overview_glide: Option<OverviewGlide>,
    overview_vertical_settle: Option<(TaskEntry, Anim, bool)>,
    overview_reopen_progress: f32,
    overview_edge_spring: EdgeStretch,
    overview_anim: Anim,
    overview_exiting: bool,
    overview_enter_from: Option<(Rect, f32)>,
    settings_launch_themed: Option<bool>,
    task_menu_idx: Option<usize>,

    // Unified Android MotionEvent + VelocityTracker Gesture Engine
    pointer_down: bool,
    press_origin: [f32; 2],
    press_time: f64,
    press_ctx: PressContext,
    gesture: ActiveGesture,
    vel_tracker: VelocityTracker,
    last_frame_time: f64,
    long_press_fired: bool,
    scroll_fling: Option<(ScrollTarget, f32)>,
    scroll_stretches: [EdgeStretch; 7],
    preview_notif_swipe: Option<(u32, f32)>,
    preview_stretch: Option<(ScrollTarget, f32)>,

    // Cached last-frame geometry for exact Z-ordered touch/mouse interception
    last_ws_rect: Rect,
    last_ws_widget_rects: Vec<Rect>,
    last_home_icon_rects: Vec<(usize, Rect, [f32; 2])>,
    last_drawer_icon_rects: Vec<(HomeApp, Rect, [f32; 2])>,
    pressed_drawer_icon: Option<(HomeApp, [f32; 2])>,
    last_dock_icon_rects: Vec<(AppId, Rect)>,
    last_notif_container: Rect,
    last_notif_rects: Vec<(u32, bool, Rect)>,
    last_qs_grid_rect: Rect,
    last_radio_tile_rects: [Option<Rect>; 2],
    last_qs_edit_btn: Rect,
    last_qs_edit_tiles: Vec<(usize, Rect)>,
    last_brightness_rect: Rect,
    last_vol_tracks: [Option<(u8, Rect)>; 4],
    last_overview_rect: Rect,
    last_overview_cards: Vec<(usize, Rect)>,
    last_overview_step: f32,
    last_overview_max: f32,
    last_scroll_max: [f32; 7],

    // Toast
    toast: Option<(String, f64)>,
}

impl<E: EmbeddedApps> Default for LauncherDemo<E> {
    fn default() -> Self {
        let mk_demo = |id: u32,
                       app: &str,
                       title: &str,
                       body: &str,
                       big_text: &str,
                       time_label: &str,
                       silent: bool,
                       ongoing: bool,
                       has_reply: bool,
                       target_app: Option<AppId>| NotificationItem {
            id,
            app: app.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            big_text: big_text.to_string(),
            time_label: time_label.to_string(),
            silent,
            ongoing,
            urgency: if silent { 0 } else { 1 },
            category: None,
            desktop_entry: None,
            resident: ongoing,
            transient: false,
            actions: Vec::new(),
            icon_image: None,
            expires_at: None,
            arrived_at: 0.0,
            heads_up_until: None,
            from_dbus: false,
            expanded: false,
            expand_anim: Anim::at(0.0),
            controls_open: false,
            controls_anim: Anim::at(0.0),
            swipe_anim: Anim::at(0.0),
            dismiss_anim: Anim::at(1.0),
            dismissing: false,
            ripple: None,
            snoozed: false,
            has_reply,
            target_app,
        };
        let notifications = vec![
            mk_demo(
                1,
                "Shell",
                "Downloads",
                "bhoo-demo-bundle.zip downloaded (4.2 MB)",
                "bhoo-demo-bundle.zip (4.2 MB) saved to Internal Storage / Downloads. Tap to view in Files.",
                "2m",
                false,
                false,
                false,
                Some(AppId::Files),
            ),
            mk_demo(
                2,
                "Shell",
                "Calendar",
                "Design review starts in 15 minutes — Conferen...",
                "Design review starts in 15 minutes — Conference Room B · SpectreOS System UI & libbhoo parity review.",
                "5m",
                false,
                false,
                false,
                Some(AppId::Calendar),
            ),
            mk_demo(
                3,
                "Shell",
                "Alice Johnson",
                "Hey! Are we still meeting at 3 PM to revie...",
                "Hey! Are we still meeting at 3 PM to review the SpectreOS System UI layout and animations?",
                "now",
                false,
                false,
                true,
                Some(AppId::ArcaneChat),
            ),
            mk_demo(
                4,
                "Android System",
                "USB debugging connected",
                "Tap to turn off USB debugging",
                "USB debugging is active. Tap to open Developer options in Settings.",
                "14m",
                false,
                true,
                false,
                Some(AppId::Settings),
            ),
            mk_demo(
                5,
                "Android System",
                "Charging this device via USB",
                "Tap for more options.",
                "Battery 100% — Fully charged via USB-C.",
                "1h",
                false,
                true,
                false,
                Some(AppId::Settings),
            ),
            mk_demo(
                6,
                "F-Droid",
                "F-Droid was updated. Tap to open",
                "Repositories refreshed · 2h",
                "F-Droid client updated to 1.21.0 with offline local index support.",
                "2h",
                false,
                false,
                false,
                Some(AppId::FDroid),
            ),
            mk_demo(
                7,
                "ArcaneChat",
                "ArcaneChat",
                "Background connection enabled",
                "Persistent IMAP / WebXDC sync active in background.",
                "3h",
                true,
                true,
                false,
                Some(AppId::ArcaneChat),
            ),
        ];
        #[cfg(test)]
        let notif_history = Vec::new();
        #[cfg(not(test))]
        let notif_history = notifications::load_notification_history();
        let time_service = TimeService::default();
        let mut shell = Self {
            sys: SystemState::default(),
            lockscreen: lockscreen::LockScreen::default(),
            desktop_registry: freedesktop::DesktopRegistry::discover(),
            native_compositor: false,
            active_wayland_app: None,
            active_wayland_desktop: None,
            wayland_surface_ready: false,
            wayland_launch_frame: None,
            wayland_shade_frame: None,
            wayland_launch_capture_attempted: false,
            wayland_task_frames: std::collections::HashMap::new(),
            host_commands: Vec::new(),
            pending_haptics: Vec::new(),
            pending_touch_id_retry: false,
            mode: ShellMode::Home,
            tasks: vec![
                TaskEntry::Single(AppId::Contacts),
                TaskEntry::Single(AppId::Clock),
                TaskEntry::Single(AppId::Calculator),
                TaskEntry::Single(AppId::Settings),
                TaskEntry::Single(AppId::Gmail),
            ],
            freeform: None,
            split_ratio: 0.50,
            embedded: E::default(),
            gmail_body_pending: None,
            settings_app: Box::default(),
            clock: time_service.now(),
            time_service,
            builtin_apps: Box::default(),
            home_icons: vec![
                (1, 0, HomeApp::Builtin(AppId::ArcaneChat)),
                (0, 4, HomeApp::Builtin(AppId::FDroid)),
                (1, 4, HomeApp::Builtin(AppId::Gallery)),
                (2, 4, HomeApp::Builtin(AppId::Music)),
                (3, 4, HomeApp::Builtin(AppId::Settings)),
                (4, 4, HomeApp::Builtin(AppId::Telegram)),
            ],
            home_icon_pages: vec![0; 6],
            home_page: 0,
            home_page_count: 1,
            home_page_anim: Anim::at(0.0),
            home_reorder_anim: Anim::at(0.0),
            home_stretch: EdgeStretch::default(),
            launcher_sub: LauncherSub::None,
            prev_launcher_sub: LauncherSub::None,
            launcher_sub_push: true,
            launcher_sub_anim: Anim::at(1.0),
            wallpaper_tab_basic: true,
            drawer_open: false,
            drawer_anim: Anim::at(0.0),
            drawer_scroll: 0.0,
            drawer_search: String::new(),
            ime_target: None,
            ime_anim: Anim::at(0.0),
            ime_shift: false,
            ime_symbols: false,
            ime_cursor: 0,
            ime_pending_open: false,
            last_ime_field_rect: Rect::default(),
            widgets_open: false,
            widgets_anim: Anim::at(0.0),
            widgets_scroll: 0.0,
            widgets_search: String::new(),
            widgets_expanded: [false, true, false, false],
            wallpaper_scroll: 0.0,
            wallpaper_picker_scroll: 0.0,
            wallpaper_color_page: 0,
            wallpaper_shortcut_side: 0,
            wallpapers: wallpaper::Library::default(),
            wallpaper_files: spectre_files::Browser::default(),
            wallpaper_pending: None,
            wallpaper_load: None,
            home_settings_scroll: 0.0,
            home_popup: HomePopup::None,
            last_home_popup: HomePopup::None,
            popup_anim: Anim::at(0.0),
            app_anim: Anim::at(1.0),
            app_transition: AppTransition::None,
            settings_return_snapshot: None,
            preview_transition_t: None,
            preview_edge_back: None,
            last_phone_rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 411.0,
                height: 846.0,
            },
            shade_stage: 0,
            shade_anim: Anim::at(0.0),
            notification_sheet_bounce: Anim::at(1.0),
            shade_scroll: 0.0,
            qs_page: 0,
            qs_page_anim: Anim::at(0.0),
            qs_tiles: qs_editor::load_tiles(),
            qs_edit_open: false,
            qs_edit_anim: Anim::at(0.0),
            qs_edit_origin: [375.0, 520.0],
            qs_edit_scroll: 0.0,
            qs_edit_drag: None,
            qs_edit_last_frame: None,
            active_apps_open: false,
            active_bg_apps: vec![(AppId::ArcaneChat, "3 hr")],
            notifications,
            notif_history,
            settings_apps: None,
            device_info: DeviceRuntimeInfo::probe(),
            last_heads_up_rect: None,
            next_notif_id: 20,
            power_open: false,
            power_anim: Anim::at(0.0),
            power_state: PowerState::Running,
            flash_until: 0.0,
            vol_open: false,
            vol_expanded: false,
            vol_anim: Anim::at(0.0),
            vol_expand_anim: Anim::at(0.0),
            vol_hide_at: 0.0,
            overview_scroll: 0.0,
            overview_glide: None,
            overview_vertical_settle: None,
            overview_reopen_progress: 0.0,
            overview_edge_spring: EdgeStretch::default(),
            overview_anim: Anim::at(0.0),
            overview_exiting: false,
            overview_enter_from: None,
            settings_launch_themed: None,
            task_menu_idx: None,
            pointer_down: false,
            press_origin: [0.0; 2],
            press_time: 0.0,
            press_ctx: PressContext::None,
            gesture: ActiveGesture::None,
            vel_tracker: VelocityTracker::default(),
            last_frame_time: 0.0,
            long_press_fired: false,
            scroll_fling: None,
            scroll_stretches: [EdgeStretch::default(); 7],
            preview_notif_swipe: None,
            preview_stretch: None,
            last_ws_rect: Rect::default(),
            last_ws_widget_rects: Vec::new(),
            last_home_icon_rects: Vec::new(),
            last_drawer_icon_rects: Vec::new(),
            pressed_drawer_icon: None,
            last_dock_icon_rects: Vec::new(),
            last_notif_container: Rect::default(),
            last_notif_rects: Vec::new(),
            last_qs_grid_rect: Rect::default(),
            last_radio_tile_rects: [None; 2],
            last_qs_edit_btn: Rect::default(),
            last_qs_edit_tiles: Vec::new(),
            last_brightness_rect: Rect::default(),
            last_vol_tracks: [None; 4],
            last_overview_rect: Rect::default(),
            last_overview_cards: Vec::new(),
            last_overview_step: 300.0,
            last_overview_max: 0.0,
            last_scroll_max: [0.0; 7],
            toast: None,
        };
        if shell.sys.grid_dims() != (5, 5) { shell.reflow_home_grid((5, 5)); }
        shell
    }
}

const HEADS_UP_EXIT_SECONDS: f64 = 0.18;
const SETTINGS_RETURN_SECONDS: f64 = 0.60;
const SETTINGS_DOCK_FRACTION: f32 = 0.56;

impl<E: EmbeddedApps> LauncherDemo<E> {
    pub fn open_ime_for(&mut self, id: u64) {
        let already_open = self.ime_target.is_some() && self.ime_anim.value > 0.90;
        self.ime_target = Some(id);
        self.ime_cursor = self.edit_text_for(id).chars().count();
        if !already_open {
            self.ime_pending_open = true;
        }
    }

    fn blur_app_text_field(&mut self, now: f64) {
        if self.ime_target.is_some_and(|id| matches!(id, ID_FILES_SEARCH | ID_FILES_EDIT)
            || spectre_calendar::field_is_calendar(id)) {
            self.ime_target = None;
            self.ime_anim.set(0.0, now, 0.20, 0.18);
        }
    }

    pub fn edit_text_for(&self, id: u64) -> &str {
        match id {
            ID_DRAWER_SEARCH => &self.drawer_search,
            ID_WIDGETS_SEARCH => &self.widgets_search,
            ID_SETTINGS_SEARCH | ID_DNS_HOSTNAME => self.settings_app.edit_text_for(id),
            ID_FILES_SEARCH | ID_FILES_EDIT if self.launcher_sub == LauncherSub::WallpaperFiles => {
                self.wallpaper_files.edit_text(id).unwrap_or_default()
            }
            ID_BROWSER_URL | ID_CHAT_COMPOSER | ID_FILES_SEARCH | ID_FILES_EDIT => self.builtin_apps.edit_text_for(id, self.mode == ShellMode::Fullscreen(AppId::Calendar)),
            id if spectre_calendar::field_is_calendar(id) => self.builtin_apps.edit_text_for(id, self.mode == ShellMode::Fullscreen(AppId::Calendar)),
            _ => self.embedded.edit_text_for(id),
        }
    }

    pub fn set_edit_text(&mut self, id: u64, value: &str) {
        match id {
            ID_DRAWER_SEARCH => {
                self.drawer_search = value.to_owned();
                self.drawer_scroll = 0.0;
            }
            ID_WIDGETS_SEARCH => {
                self.widgets_search = value.to_owned();
                self.widgets_scroll = 0.0;
            }
            ID_SETTINGS_SEARCH => {
                self.settings_app.set_edit_text(id, value);
            }
            ID_DNS_HOSTNAME => {
                self.sys.dns_host = value.to_owned();
                self.settings_app.set_edit_text(id, value);
            }
            ID_FILES_SEARCH | ID_FILES_EDIT if self.launcher_sub == LauncherSub::WallpaperFiles => {
                self.wallpaper_files.set_edit_text(id, value);
            }
            ID_BROWSER_URL | ID_CHAT_COMPOSER | ID_FILES_SEARCH | ID_FILES_EDIT => {
                self.builtin_apps.set_edit_text(id, value, self.mode == ShellMode::Fullscreen(AppId::Calendar));
            }
            id if spectre_calendar::field_is_calendar(id) => self.builtin_apps.set_edit_text(id, value, self.mode == ShellMode::Fullscreen(AppId::Calendar)),
            _ => {
                if self.embedded.set_edit_text(id, value) {
                    self.gmail_body_pending = Some(value.to_owned());
                }
            }
        }
    }

    /// Snap the launcher directly into a deterministic state for native GLES visual verification.
    #[allow(dead_code)]
    pub fn brightness(&self) -> f32 {
        self.sys.brightness
    }

    /// Screen inactivity auto-lock timeout in seconds (`None` if disabled or Stay Awake is on).
    pub fn screen_timeout_secs(&self) -> Option<u64> {
        if !self.sys.auto_lock_enabled || self.sys.stay_awake {
            return None;
        }
        const SECS: [u64; 6] = [15, 30, 60, 120, 300, 600];
        Some(SECS[self.sys.screen_timeout_idx.min(5)])
    }

    pub fn set_native_compositor(&mut self, enabled: bool) {
        self.native_compositor = enabled;
        if enabled {
            // Start with an empty Overview/Recents & Notification list on real hardware so only
            // genuinely launched apps and real D-Bus Freedesktop notifications appear.
            self.tasks.clear();
            self.notifications.clear();
            self.active_bg_apps.clear();
            let previous_grid = self.sys.grid_dims();
            #[cfg(not(target_arch = "wasm32"))]
            let previous_grid = if let Some(layout) = HomeLayout::load() {
                self.home_icons = layout.cells;
                self.home_icon_pages = layout.pages;
                layout.grid
            } else {
                previous_grid
            };
            self.reflow_home_grid(previous_grid);
        }
    }

    pub fn active_heads_up_rect(&self) -> Option<Rect> {
        self.last_heads_up_rect
    }

    fn is_app_unavailable(&self, app: AppId) -> bool {
        self.native_compositor
            && (app.is_launcher_placeholder()
                || (app == AppId::Messaging && self.desktop_registry.find_for_builtin(app).is_none()))
    }

    fn is_app_hidden(&self, app: AppId) -> bool {
        self.is_app_unavailable(app)
            || self.sys
            .hidden_apps
            .iter()
            .any(|s| s == app.package() || s == app.name())
    }

    fn record_notif_history(&mut self, id: u32, app: &str, title: &str, body: &str, urgency: u8) {
        if !self.sys.notification_history_enabled {
            return;
        }
        let stamp = self.time_service.now().hm();
        if let Some(existing) = self.notif_history.iter_mut().find(|e| e.id == id) {
            existing.app = app.to_string();
            existing.title = title.to_string();
            existing.body = body.to_string();
            existing.time_label = stamp;
            existing.dismissed = false;
            existing.urgency = urgency;
        } else {
            self.notif_history.insert(
                0,
                NotificationHistoryEntry {
                    id,
                    app: app.to_string(),
                    title: title.to_string(),
                    body: body.to_string(),
                    time_label: stamp,
                    dismissed: false,
                    urgency,
                },
            );
            if self.notif_history.len() > 80 {
                self.notif_history.truncate(80);
            }
        }
        notifications::save_notification_history(&self.notif_history);
    }

    fn mark_history_dismissed(&mut self, id: u32) {
        if let Some(e) = self.notif_history.iter_mut().find(|e| e.id == id) {
            e.dismissed = true;
            notifications::save_notification_history(&self.notif_history);
        }
    }

    /// Ingest a real `org.freedesktop.Notifications.Notify` payload (Desktop Notifications v1.3).
    /// Returns `true` if the compositor should wake the screen from sleep to the lock screen.
    pub fn push_freedesktop_notification(
        &mut self,
        incoming: notifications::IncomingNotification,
        now: f64,
    ) -> bool {
        // Resolve icon per Freedesktop v1.3 §5.1 priority:
        // 1. image-data  2. image-path  3. app_icon  4. icon_data  5. desktop-entry fallback
        let mut icon_image = incoming
            .image_data
            .as_ref()
            .and_then(notifications::RawImageData::to_bhoo_image);

        if icon_image.is_none()
            && let Some(ref p) = incoming.image_path
        {
            icon_image = self.resolve_notif_icon_spec(p);
        }
        if icon_image.is_none() && !incoming.app_icon.trim().is_empty() {
            icon_image = self.resolve_notif_icon_spec(&incoming.app_icon);
        }
        if icon_image.is_none()
            && let Some(ref ic) = incoming.icon_data
        {
            icon_image = ic.to_bhoo_image();
        }
        if icon_image.is_none()
            && let Some(ref de) = incoming.desktop_entry
        {
            let did = if de.ends_with(".desktop") {
                de.clone()
            } else {
                format!("{de}.desktop")
            };
            icon_image = self.desktop_registry.icon_for(&did);
        }

        // Spec §7: Critical notifications (urgency == 2) MUST NOT auto-expire.
        // expire_timeout == 0 never expires; > 0 expires after N ms; -1 uses server policy
        // (transient expires after 8s, persistent stays in shade until dismissed).
        let expires_at = if incoming.urgency >= 2 || incoming.expire_timeout_ms == 0 {
            None
        } else if incoming.expire_timeout_ms > 0 {
            Some(now + (incoming.expire_timeout_ms as f64) / 1000.0)
        } else if incoming.transient {
            Some(now + 8.0)
        } else {
            None
        };

        let silent = incoming.urgency == 0 || incoming.suppress_sound;
        let show_heads_up = !self.lockscreen.locked
            && self.shade_stage == 0
            && !self.sys.dnd_enabled
            && incoming.urgency >= 1;
        let heads_up_until = if show_heads_up {
            let deadline = now + if incoming.urgency == 2 { 7.0 } else { 5.5 };
            Some(expires_at.map_or(deadline, |expiry| expiry.min(deadline)))
        } else {
            None
        };

        let body_single = incoming.body.replace('\n', " ");
        let body_short = if body_single.chars().count() > 86 {
            let mut s: String = body_single.chars().take(83).collect();
            s.push_str("...");
            s
        } else {
            body_single
        };

        let has_reply = incoming
            .actions
            .iter()
            .any(|(k, l)| k.eq_ignore_ascii_case("reply") || l.eq_ignore_ascii_case("reply"));

        let item = NotificationItem {
            id: incoming.id,
            app: incoming.app_name.clone(),
            title: incoming.summary.clone(),
            body: body_short,
            big_text: incoming.body.clone(),
            time_label: "now".into(),
            silent,
            ongoing: false,
            urgency: incoming.urgency,
            category: incoming.category.clone(),
            desktop_entry: incoming.desktop_entry.clone(),
            resident: incoming.resident,
            transient: incoming.transient,
            actions: incoming.actions.clone(),
            icon_image,
            expires_at,
            arrived_at: now,
            heads_up_until,
            from_dbus: true,
            expanded: false,
            expand_anim: Anim::at(0.0),
            controls_open: false,
            controls_anim: Anim::at(0.0),
            swipe_anim: Anim::at(0.0),
            dismiss_anim: Anim::at(1.0),
            dismissing: false,
            ripple: None,
            snoozed: false,
            has_reply,
            target_app: None,
        };

        // Spec §9.1.2: atomic replace when replaces_id matches an active notification
        if incoming.replaces_id != 0
            && let Some(slot) = self
                .notifications
                .iter_mut()
                .find(|n| n.id == incoming.replaces_id)
        {
            *slot = item;
        } else {
            self.notifications.retain(|n| n.id != incoming.id);
            self.notifications.insert(0, item);
        }

        if !incoming.transient {
            self.record_notif_history(
                incoming.id,
                &incoming.app_name,
                &incoming.summary,
                &incoming.body,
                incoming.urgency,
            );
        }

        incoming.urgency >= 1 && self.sys.wake_screen_for_notifs && !self.sys.dnd_enabled
    }

    fn resolve_notif_icon_spec(&mut self, spec: &str) -> Option<bhoo::Image> {
        if let Some(path) = notifications::resolve_icon_uri_or_path(spec)
            && let Some(s) = path.to_str()
        {
            return self.desktop_registry.icon_resolver.load_icon_image(s);
        }
        self.desktop_registry.icon_resolver.load_icon_image(spec)
    }

    /// Remove a notification after the D-Bus daemon handled `CloseNotification(id)` (reason 3).
    pub fn close_freedesktop_notification(&mut self, id: u32) {
        self.notifications.retain(|n| n.id != id);
        self.mark_history_dismissed(id);
    }

    /// Periodic notification tick: expire timed-out items (reason=1), clear finished heads-ups,
    /// and refresh relative timestamps (`now`, `1m`, `1h`). Returns `true` if UI should redraw.
    pub fn tick_notifications(&mut self, now: f64) -> bool {
        let mut changed = false;
        if self.native_compositor {
            for (title, body) in self.builtin_apps.calendar_reminders() {
                self.post_notification("Calendar".into(), title, body);
                if let Some(notification) = self.notifications.first_mut() {
                    notification.target_app = Some(AppId::Calendar);
                }
                changed = true;
            }
        }
        let mut expired: Vec<(u32, bool)> = Vec::new();
        let exit_duration = HEADS_UP_EXIT_SECONDS * self.sys.anim_scale() as f64;
        for n in &mut self.notifications {
            if let Some(exp) = n.expires_at
                && now >= exp
                && !n.heads_up_until.is_some_and(|until| now < until + exit_duration)
            {
                expired.push((n.id, n.from_dbus));
                continue;
            }
            if let Some(hu) = n.heads_up_until
                && now >= hu + exit_duration
            {
                n.heads_up_until = None;
                changed = true;
            }
            if n.from_dbus && n.arrived_at > 0.0 {
                let age = (now - n.arrived_at).max(0.0) as u64;
                let lbl = if age < 60 {
                    "now".to_string()
                } else if age < 3600 {
                    format!("{}m", age / 60)
                } else {
                    format!("{}h", age / 3600)
                };
                if n.time_label != lbl {
                    n.time_label = lbl;
                    changed = true;
                }
            }
        }
        for (id, from_dbus) in expired {
            self.notifications.retain(|n| n.id != id);
            self.mark_history_dismissed(id);
            if from_dbus {
                self.host_commands.push(HostCommand::NotifyClosed {
                    id,
                    reason: notifications::CLOSE_REASON_EXPIRED,
                });
            }
            changed = true;
        }
        changed
    }

    fn dismiss_notification(&mut self, id: u32, reason: u32) {
        if let Some(pos) = self.notifications.iter().position(|n| n.id == id) {
            let from_dbus = self.notifications[pos].from_dbus;
            self.notifications.remove(pos);
            self.mark_history_dismissed(id);
            if from_dbus {
                self.host_commands
                    .push(HostCommand::NotifyClosed { id, reason });
            }
        }
    }

    fn clear_all_dismissible_notifications(&mut self, now: f64) {
        let ids: Vec<(u32, bool)> = self
            .notifications
            .iter()
            .filter(|n| !n.ongoing)
            .map(|n| (n.id, n.from_dbus))
            .collect();
        self.notifications.retain(|n| n.ongoing);
        for (id, from_dbus) in ids {
            self.mark_history_dismissed(id);
            if from_dbus {
                self.host_commands.push(HostCommand::NotifyClosed {
                    id,
                    reason: notifications::CLOSE_REASON_DISMISSED,
                });
            }
        }
        self.show_toast("Cleared notifications", now);
    }

    fn invoke_notification_action(
        &mut self,
        id: u32,
        action_key: &str,
        card_rect: Option<Rect>,
        now: f64,
    ) {
        let Some(pos) = self.notifications.iter().position(|n| n.id == id) else {
            return;
        };
        let from_dbus = self.notifications[pos].from_dbus;
        let resident = self.notifications[pos].resident;
        let ongoing = self.notifications[pos].ongoing;
        let desktop_entry = self.notifications[pos].desktop_entry.clone();
        let target_app = self.notifications[pos].target_app;
        let has_named_action = self.notifications[pos]
            .actions
            .iter()
            .any(|(k, _)| k == action_key);

        if from_dbus && (has_named_action || action_key == "default") {
            self.host_commands.push(HostCommand::NotifyActionInvoked {
                id,
                action_key: action_key.to_string(),
            });
        }

        // Close heads-up & shade cleanly when activating a notification
        self.notifications[pos].heads_up_until = None;
        if let Some(app) = target_app {
            if app == AppId::Settings {
                self.settings_app
                    .navigate(settings::SettingsPage::DevOptions, now);
            }
            if let Some(rect) = card_rect {
                self.open_app_from(app, Some(rect), false, true, false, now);
            } else {
                self.open_app(app, now);
            }
        } else if let Some(ref de) = desktop_entry {
            let found_idx = self.desktop_registry.apps.iter().position(|a| {
                a.id.eq_ignore_ascii_case(de)
                    || a.id
                        .strip_suffix(".desktop")
                        .is_some_and(|stem| stem.eq_ignore_ascii_case(de))
            });
            self.shade_stage = 0;
            self.shade_anim.set(0.0, now, 0.22, 0.18);
            if let Some(idx) = found_idx {
                self.open_desktop_app(idx, now);
            }
        } else if from_dbus {
            self.shade_stage = 0;
            self.shade_anim.set(0.0, now, 0.22, 0.18);
        }

        // Spec §8 `resident`: unless resident/ongoing, remove notification after action invocation
        if !resident && !ongoing && from_dbus {
            self.dismiss_notification(id, notifications::CLOSE_REASON_DISMISSED);
        }
    }

    pub fn remove_wayland_task(&mut self, app: Option<AppId>, desktop_id: Option<&str>) {
        if let Some(id) = desktop_id {
            self.wayland_task_frames.remove(id);
        }
        if let Some(a) = app {
            self.wayland_task_frames.remove(a.package());
            self.tasks.retain(|t| *t != TaskEntry::Single(a));
            if self.active_wayland_app == Some(a) {
                self.active_wayland_app = None;
            }
            if let Some(d) = self.desktop_registry.find_for_builtin(a)
                && let Some(idx) = self.desktop_registry.apps.iter().position(|e| e.id == d.id)
            {
                self.tasks.retain(|t| *t != TaskEntry::Desktop(idx));
                if self.active_wayland_desktop == Some(idx) {
                    self.active_wayland_desktop = None;
                }
            }
        }
        if let Some(id) = desktop_id
            && let Some(idx) = self.desktop_registry.apps.iter().position(|e| e.id == id)
        {
            self.tasks.retain(|t| *t != TaskEntry::Desktop(idx));
            if self.active_wayland_desktop == Some(idx) {
                self.active_wayland_desktop = None;
            }
        }
    }

    pub fn set_active_wayland_app(&mut self, app: Option<AppId>) {
        if self.active_wayland_app != app {
            self.wayland_surface_ready = false;
            self.wayland_shade_frame = None;
        }
        self.active_wayland_app = app;
        if let Some(a) = app {
            self.active_wayland_desktop = None;
            self.tasks.retain(|t| *t != TaskEntry::Single(a));
            self.tasks.insert(0, TaskEntry::Single(a));
            if self.tasks.len() > 8 {
                self.tasks.truncate(8);
            }
            let continuing = self.mode == ShellMode::Fullscreen(a)
                && matches!(self.app_transition, AppTransition::Opening { app, .. } if app == a);
            self.mode = ShellMode::Fullscreen(a);
            self.app_anim.snap(1.0);
            if !continuing {
                self.app_transition = AppTransition::None;
            }
        }
    }

    pub fn set_active_wayland_desktop(&mut self, desktop_id: Option<&str>) {
        let Some(id) = desktop_id else {
            self.active_wayland_desktop = None;
            self.wayland_surface_ready = false;
            self.wayland_shade_frame = None;
            return;
        };
        if let Some(idx) = self.desktop_registry.apps.iter().position(|a| a.id == id) {
            if self.active_wayland_desktop != Some(idx) {
                self.wayland_surface_ready = false;
                self.wayland_shade_frame = None;
            }
            self.active_wayland_app = None;
            self.active_wayland_desktop = Some(idx);
            self.tasks.retain(|t| *t != TaskEntry::Desktop(idx));
            self.tasks.insert(0, TaskEntry::Desktop(idx));
            if self.tasks.len() > 8 {
                self.tasks.truncate(8);
            }
            let continuing = self.mode == ShellMode::FullscreenDesktop(idx)
                && matches!(self.app_transition, AppTransition::OpeningDesktop { .. });
            self.mode = ShellMode::FullscreenDesktop(idx);
            self.app_anim.snap(1.0);
            if !continuing {
                self.app_transition = AppTransition::None;
            }
        }
    }

    pub fn take_host_commands(&mut self) -> Vec<HostCommand> {
        std::mem::take(&mut self.host_commands)
    }

    pub fn is_wayland_passthrough(&self) -> bool {
        self.shows_wayland_surface()
            && !self.lockscreen.is_unlocking()
            && !self.notification_shade_visible()
            && !self.power_open
            && !self.vol_open
            && !self.active_apps_open
    }

    pub(crate) fn shows_wayland_surface(&self) -> bool {
        !self.lockscreen.locked
            && matches!(self.power_state, PowerState::Running)
            && self.wayland_surface_ready
            && matches!(self.app_transition, AppTransition::None)
            && match self.mode {
                ShellMode::Fullscreen(app) => self.active_wayland_app == Some(app),
                ShellMode::FullscreenDesktop(idx) => self.active_wayland_desktop == Some(idx),
                _ => false,
            }
    }

    pub(crate) fn notification_shade_visible(&self) -> bool {
        self.shade_anim.value > 0.005
            || self.shade_stage > 0
            || self.qs_edit_open
            || self.qs_edit_anim.value > 0.005
    }

    pub(crate) fn composites_wayland_surface(&self) -> bool {
        self.shows_wayland_surface()
            && !(self.notification_shade_visible() && self.wayland_shade_frame.is_some())
    }

    pub(crate) fn nav_bar_height(&self) -> f32 {
        match self.sys.nav_mode {
            NavMode::Gestural => 24.0,
            NavMode::ThreeButton => 44.0,
        }
    }

    fn wayland_task_frame(&self, task: TaskEntry) -> Option<&Image> {
        let id = match task {
            TaskEntry::Single(app) => app.package(),
            TaskEntry::Desktop(idx) => self.desktop_registry.apps.get(idx)?.id.as_str(),
            TaskEntry::SplitPair(..) => return None,
        };
        self.wayland_task_frames.get(id)
    }

    pub fn go_home_now(&mut self, now: f64) {
        self.active_wayland_app = None;
        self.active_wayland_desktop = None;
        self.go_home(now);
    }

    pub fn open_overview_now(&mut self, now: f64) {
        self.active_wayland_app = None;
        self.active_wayland_desktop = None;
        self.open_overview(now);
    }

    /// Pin civil time for a preview driven by an external clock.
    pub fn set_clock_hm(&mut self, hh: u8, mm: u8) {
        use chrono::Timelike;
        if let Some(value) = self.clock.0.with_hour(hh.into()).and_then(|t| t.with_minute(mm.into())) {
            self.clock = ClockReading(value);
            self.time_service = TimeService(std::sync::Arc::new(self.clock));
        }
    }

    pub fn trigger_haptic(&mut self, effect: services::HapticEffect) {
        if self.sys.haptic_enabled {
            self.pending_haptics.push(effect);
        }
    }

    pub fn drain_haptics(&mut self) -> Vec<services::HapticEffect> {
        std::mem::take(&mut self.pending_haptics)
    }

    pub fn take_touch_id_retry(&mut self) -> bool {
        std::mem::take(&mut self.pending_touch_id_retry)
    }

    pub fn has_unread_notifications(&self) -> bool {
        self.notifications
            .iter()
            .any(|n| !n.snoozed && !n.dismissing && !n.ongoing)
    }

    pub fn is_fingerprint_lab_foreground(&self) -> bool {
        self.active_wayland_desktop
            .and_then(|idx| self.desktop_registry.apps.get(idx))
            .is_some_and(|a| {
                a.id.to_ascii_lowercase().contains("fingerprint")
                    || a.name.to_ascii_lowercase().contains("fingerprint")
            })
    }

    #[cfg(feature = "compositor")]
    pub(crate) fn drive_benchmark_step(&mut self, phase: u32, now: f64) {
        self.lockscreen.locked = false;
        match phase % 8 {
            0 => {
                self.go_home_now(now);
                self.drawer_open = true;
                self.drawer_anim.set(1.0, now, 0.34, 0.26);
            }
            1 => {
                self.drawer_open = false;
                self.drawer_anim.set(0.0, now, 0.34, 0.26);
            }
            2 => {
                self.shade_stage = 2;
                self.shade_anim.set(2.0, now, 0.38, 0.28);
            }
            3 => {
                self.shade_stage = 0;
                self.shade_anim.set(0.0, now, 0.38, 0.28);
            }
            4 => {
                self.open_app(AppId::Settings, now);
            }
            5 => {
                self.go_home(now);
            }
            6 => {
                self.open_overview_now(now);
            }
            _ => {
                self.go_home_now(now);
            }
        }
    }

    pub fn set_capture_preset(&mut self, preset: &str) {
        match preset {
            "lock" => {
                self.lockscreen.lock();
            }
            "lock_pin" => {
                self.sys.lock_method = lockscreen::LockMethod::Pin;
                self.lockscreen.lock();
                self.lockscreen.bouncer_open = true;
            }
            "lock_password" => {
                self.sys.lock_method = lockscreen::LockMethod::Password;
                self.lockscreen.lock();
                self.lockscreen.bouncer_open = true;
            }
            "drawer" => {
                self.drawer_open = true;
                self.drawer_anim.snap(1.0);
            }
            "drawer_half" => {
                self.drawer_open = true;
                self.drawer_anim.snap(0.50);
            }
            "shade" => {
                self.shade_stage = 1;
                self.shade_anim.snap(1.0);
            }
            "shade_half" | "shade_closing" => {
                self.shade_stage = 1;
                self.shade_anim.snap(0.45);
            }
            "qs_morph" => {
                self.shade_stage = 2;
                self.shade_anim.snap(1.48);
            }
            "qs" => {
                self.shade_stage = 2;
                self.shade_anim.snap(2.0);
            }
            "qs_edit_mid" => {
                self.shade_stage = 2;
                self.shade_anim.snap(2.0);
                self.qs_edit_open = true;
                self.qs_edit_origin = [375.0, 520.0];
                self.qs_edit_anim.snap(0.48);
            }
            "qs_edit" => {
                self.shade_stage = 2;
                self.shade_anim.snap(2.0);
                self.qs_edit_open = true;
                self.qs_edit_origin = [375.0, 520.0];
                self.qs_edit_anim.snap(1.0);
            }
            "notif_swipe" => {
                self.shade_stage = 1;
                self.shade_anim.snap(1.0);
                self.preview_notif_swipe = Some((1, 148.0));
            }
            "overview_clock" => {
                self.mode = ShellMode::Overview;
        self.active_wayland_app = None;
        self.active_wayland_desktop = None;
        if self.native_compositor {
            self.host_commands.push(HostCommand::MinimizeAll);
        }
                self.overview_anim.snap(1.0);
                self.overview_scroll = 301.48;
            }
            "power" => {
                self.shade_stage = 2;
                self.shade_anim.snap(2.0);
                self.power_open = true;
                self.power_anim.snap(1.0);
            }
            "volume" => {
                self.vol_open = true;
                self.vol_expanded = true;
                self.vol_anim.snap(1.0);
                self.vol_expand_anim.snap(1.0);
                self.vol_hide_at = 9999.0;
            }
            "overview" => {
                self.mode = ShellMode::Overview;
        self.active_wayland_app = None;
        self.active_wayland_desktop = None;
        if self.native_compositor {
            self.host_commands.push(HostCommand::MinimizeAll);
        }
                self.overview_anim.snap(1.0);
            }
            "split" => {
                self.mode = ShellMode::Split {
                    top: AppId::Contacts,
                    bottom: AppId::Clock,
                };
                self.split_ratio = 0.50;
            }
            "freeform" => {
                self.mode = ShellMode::Home;
                self.freeform = Some(FreeformWin {
                    app: AppId::Contacts,
                    x: 16.0,
                    y: 44.0,
                    w: 230.0,
                    h: 410.0,
                });
            }
            "settings" => {
                self.mode = ShellMode::Fullscreen(AppId::Settings);
                self.app_anim.snap(1.0);
            }
            "settings_scrolled" => {
                self.mode = ShellMode::Fullscreen(AppId::Settings);
                self.app_anim.snap(1.0);
                self.settings_app.set_home_scroll(140.0);
            }
            "apps_collapse_mid" => {
                self.mode = ShellMode::Fullscreen(AppId::Settings);
                self.app_anim.snap(1.0);
                self.settings_app
                    .set_subpage_preview(settings::SettingsPage::Apps, 42.0);
            }
            "apps_collapsed" => {
                self.mode = ShellMode::Fullscreen(AppId::Settings);
                self.app_anim.snap(1.0);
                self.settings_app
                    .set_subpage_preview(settings::SettingsPage::Apps, 120.0);
            }
            "about_stretch_bottom" => {
                self.mode = ShellMode::Fullscreen(AppId::Settings);
                self.app_anim.snap(1.0);
                self.settings_app
                    .set_subpage_preview(settings::SettingsPage::About, 260.0);
                self.preview_stretch = Some((ScrollTarget::Settings, -0.55));
            }
            "about_stretch_top" => {
                self.mode = ShellMode::Fullscreen(AppId::Settings);
                self.app_anim.snap(1.0);
                self.settings_app
                    .set_subpage_preview(settings::SettingsPage::About, 0.0);
                self.preview_stretch = Some((ScrollTarget::Settings, 0.55));
            }
            "settings_search" => {
                self.mode = ShellMode::Fullscreen(AppId::Settings);
                self.app_anim.snap(1.0);
                self.settings_app.set_search_preview("");
            }
            "settings_search_results" => {
                self.mode = ShellMode::Fullscreen(AppId::Settings);
                self.app_anim.snap(1.0);
                self.settings_app.set_search_preview("display");
            }
            "home_popup" => {
                self.mode = ShellMode::Home;
                let popup = HomePopup::BlankOptions([235.0, 152.0]);
                self.home_popup = popup;
                self.last_home_popup = popup;
                self.popup_anim.snap(1.0);
            }
            "edge_back" => {
                self.mode = ShellMode::Fullscreen(AppId::Contacts);
                self.app_anim.snap(1.0);
                self.preview_edge_back = Some((true, 0.52, 62.0));
            }
            "app_open_mid" => {
                let phone = self.last_phone_rect;
                let origin = self
                    .find_home_or_dock_icon_rect(AppId::Contacts, phone, 31.0, 24.0)
                    .unwrap_or(Rect {
                        x: 179.0,
                        y: 750.0,
                        width: 54.0,
                        height: 54.0,
                    });
                self.mode = ShellMode::Fullscreen(AppId::Contacts);
                self.app_transition = AppTransition::Opening {
                    app: AppId::Contacts,
                    origin,
                    from_icon: true,
                    from_shade: false,
                    from_overview: false,
                    themed_icon: self.sys.themed_icons,
                    start: 0.0,
                    duration: 0.36,
                };
                self.preview_transition_t = Some(0.16);
            }
            "app_close_mid" => {
                let phone = self.last_phone_rect;
                let target = self.find_home_or_dock_icon_rect(AppId::Contacts, phone, 31.0, 24.0);
                self.mode = ShellMode::Home;
                self.app_transition = AppTransition::Closing {
                    app: AppId::Contacts,
                    from_rect: phone,
                    from_rad: 16.0,
                    target_icon: target,
                    themed_icon: self.sys.themed_icons,
                    start: 0.0,
                    duration: 0.34,
                };
                self.preview_transition_t = Some(0.42);
            }
            "notif_launch_mid" => {
                self.shade_stage = 1;
                self.shade_anim.snap(1.0);
                self.mode = ShellMode::Fullscreen(AppId::FDroid);
                self.app_transition = AppTransition::Opening {
                    app: AppId::FDroid,
                    origin: Rect {
                        x: 16.0,
                        y: 356.0,
                        width: 379.0,
                        height: 72.0,
                    },
                    from_icon: false,
                    from_shade: true,
                    from_overview: false,
                    themed_icon: false,
                    start: 0.0,
                    duration: 0.30,
                };
                self.preview_transition_t = Some(0.14);
            }
            _ => {}
        }
    }

    /// 1=Home, 2=Recents, 3=Back, 4=VolUp, 5=VolDown, 6=Power, 7=Shade
    pub fn system_key(&mut self, code: u32, now: f64) {
        if matches!(self.power_state, PowerState::PoweredOff) {
            if code == 6 || code == 1 {
                self.power_state = PowerState::Rebooting((now * 1000.0) as u64);
            }
            return;
        }
        match code {
            1 => self.go_home(now),
            2 => self.open_overview(now),
            3 => self.go_back(now),
            4 => self.adjust_volume(0.10, now),
            5 => self.adjust_volume(-0.10, now),
            6 => {
                self.power_open = !self.power_open;
                self.power_anim
                    .set(f32::from(self.power_open), now, 0.24, 0.18);
                if self.power_open {
                    self.vol_open = false;
                    self.vol_anim.set(0.0, now, 0.20, 0.18);
                }
            }
            7 => {
                self.tap_shade_header(now);
                self.qs_edit_open = false;
                self.qs_edit_anim.set(0.0, now, 0.24, 0.20);
                self.qs_edit_drag = None;
                self.active_apps_open = false;
            }
            _ => {}
        }
    }

    fn show_toast(&mut self, msg: impl Into<String>, now: f64) {
        self.toast = Some((msg.into(), now + 2.2));
    }

    fn post_notification(&mut self, app: String, title: String, body: String) {
        let now = self.last_frame_time;
        let id = self.next_notif_id;
        self.next_notif_id += 1;
        let show_hu = !self.lockscreen.locked && self.shade_stage == 0 && !self.sys.dnd_enabled;
        self.record_notif_history(id, &app, &title, &body, 1);
        self.notifications.insert(
            0,
            NotificationItem {
                id,
                app,
                title,
                big_text: body.clone(),
                body,
                time_label: "now".into(),
                silent: false,
                ongoing: false,
                urgency: 1,
                category: None,
                desktop_entry: None,
                resident: false,
                transient: false,
                actions: Vec::new(),
                icon_image: None,
                expires_at: None,
                arrived_at: now,
                heads_up_until: show_hu.then_some(now + 4.5),
                from_dbus: false,
                expanded: false,
                expand_anim: Anim::at(0.0),
                controls_open: false,
                controls_anim: Anim::at(0.0),
                swipe_anim: Anim::at(0.0),
                dismiss_anim: Anim::at(1.0),
                dismissing: false,
                ripple: None,
                snoozed: false,
                has_reply: false,
                target_app: None,
            },
        );
    }

    fn adjust_volume(&mut self, delta: f32, now: f64) {
        self.sys.vol_media = (self.sys.vol_media + delta).clamp(0.0, 1.0);
        self.vol_open = true;
        self.vol_anim.set(1.0, now, 0.22, 0.18);
        self.vol_hide_at = now + 4.5;
    }

    fn save_home_layout(&mut self, now: f64) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.native_compositor
            && let Err(error) = HomeLayout::save(self.sys.grid_dims(), &self.home_icons, &self.home_icon_pages)
        {
            eprintln!("Wisp could not save Home layout: {error}");
            self.show_toast("Could not save Home layout", now);
        }
    }

    fn desktop_shortcut_index(&self, id: &str) -> Option<usize> {
        self.desktop_registry.apps.iter().position(|app| app.id == id)
    }

    fn paint_home_shortcut(&mut self, f: &mut Frame<'_>, rect: Rect, app: &HomeApp, palette: &Palette) {
        match app {
            HomeApp::Builtin(app) => {
                icons::paint_styled_app_icon(self.clock, f, rect, *app, self.sys.themed_icons, palette, self.sys.icon_shape);
            }
            HomeApp::Desktop(id) => {
                if let Some(index) = self.desktop_shortcut_index(id) {
                    let app = &self.desktop_registry.apps[index];
                    let image = app.icon.as_deref().and_then(|name| self.desktop_registry.icon_resolver.load_icon_image(name));
                    freedesktop::paint_desktop_app_icon(f, rect, &app.name, id, image.as_ref(), palette);
                }
            }
        }
    }

    fn trim_home_pages(&mut self, now: f64) {
        self.home_page_count = self.home_icon_pages.iter().max().map_or(1, |page| page + 1);
        if self.home_page >= self.home_page_count {
            self.home_page = self.home_page_count - 1;
            self.home_page_anim.set(self.home_page as f32, now, 0.26, 0.22);
        }
    }

    fn home_shortcut_index(&self, app: &HomeApp) -> Option<usize> {
        let desktop_backend = match app {
            HomeApp::Builtin(app @ (AppId::Phone | AppId::Messaging)) => {
                self.desktop_registry.find_for_builtin(*app)
            }
            _ => None,
        };
        self.home_icons.iter().position(|(_, _, shortcut)| {
            shortcut == app
                || matches!(shortcut, HomeApp::Desktop(id) if desktop_backend.is_some_and(|entry| entry.id == *id))
        })
    }

    fn home_drop_slot(&self, pos: [f32; 2], existing: Option<usize>, from_drawer: bool) -> Option<(usize, usize)> {
        let workspace = self.last_ws_rect;
        if workspace.width <= 0.0 || workspace.height <= 0.0 || !workspace.contains(pos) {
            return None;
        }
        let (columns, rows) = self.sys.grid_dims();
        let column = (((pos[0] - workspace.x) / workspace.width * columns as f32) as usize).min(columns - 1);
        let row = (((pos[1] - workspace.y) / workspace.height * rows as f32) as usize).min(rows - 1);
        if !from_drawer {
            return Some((column, row));
        }

        // New shortcuts go in the closest vacant cell on the current page.
        let mut occupied = std::collections::BTreeSet::new();
        for (index, &(c, r, _)) in self.home_icons.iter().enumerate() {
            if Some(index) != existing && self.home_icon_pages[index] == self.home_page {
                occupied.insert((c, r));
            }
        }
        let cell_w = workspace.width / columns as f32;
        let cell_h = workspace.height / rows as f32;
        (0..rows).flat_map(|r| (0..columns).map(move |c| (c, r)))
            .filter(|&(c, r)| {
                let cell = Rect {
                    x: workspace.x + c as f32 * cell_w,
                    y: workspace.y + r as f32 * cell_h,
                    width: cell_w,
                    height: cell_h,
                };
                !occupied.contains(&(c, r)) && self.last_ws_widget_rects.iter().all(|widget| {
                    let overlap = widget.intersect(cell);
                    overlap.width == 0.0 || overlap.height == 0.0
                })
            })
            .min_by_key(|&(c, r)| c.abs_diff(column).pow(2) + r.abs_diff(row).pow(2))
    }

    fn reflow_home_grid(&mut self, previous: (usize, usize)) {
        let (columns, rows) = self.sys.grid_dims();
        let (old_columns, old_rows) = previous;
        let adapt = |value: usize, old: usize, new: usize| {
            (value * (new - 1) + (old - 1) / 2) / (old - 1)
        };
        self.home_icon_pages.resize(self.home_icons.len(), 0);
        let mut occupied = std::collections::BTreeSet::new();
        // Allocate visible icons first so a hidden app cannot displace Settings on the Poco.
        let hidden = |index: usize| match self.home_icons[index].2 {
            HomeApp::Builtin(app) => self.is_app_hidden(app),
            HomeApp::Desktop(_) => false,
        };
        let order: Vec<_> = (0..self.home_icons.len()).filter(|&index| !hidden(index))
            .chain((0..self.home_icons.len()).filter(|&index| hidden(index))).collect();
        let mut pages = 1;
        for index in order {
            let (x, y, _) = self.home_icons[index];
            let mut page = self.home_icon_pages[index];
            let desired = (adapt(x, old_columns, columns), adapt(y, old_rows, rows));
            let slot = loop {
                if !occupied.contains(&(page, desired.0, desired.1)) { break desired; }
                let free = (0..rows).rev().flat_map(|row| (0..columns).map(move |col| (col, row)))
                    .find(|&(col, row)| !occupied.contains(&(page, col, row)));
                if let Some(slot) = free { break slot; }
                page += 1;
            };
            occupied.insert((page, slot.0, slot.1));
            self.home_icons[index].0 = slot.0;
            self.home_icons[index].1 = slot.1;
            self.home_icon_pages[index] = page;
            pages = pages.max(page + 1);
        }
        self.home_page_count = pages;
        self.home_page = self.home_page.min(pages - 1);
        self.home_page_anim.snap(self.home_page as f32);
    }

    fn find_home_or_dock_icon_rect(
        &self,
        app: AppId,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
    ) -> Option<Rect> {
        let dock_h = 84.0;
        let dock_y = phone.y + phone.height - nav_h - dock_h - 4.0;
        let dock_apps = [
            AppId::Phone,
            AppId::Messaging,
            AppId::Contacts,
            AppId::Browser,
            AppId::Camera,
        ];
        if let Some(i) = dock_apps.iter().copied().filter(|&a| !self.is_app_hidden(a)).position(|a| a == app) {
            let d_cell_w = (phone.width - 16.0) / dock_apps.iter().filter(|&&a| !self.is_app_hidden(a)).count().max(1) as f32;
            let d_icon = (d_cell_w * 0.72).clamp(44.0, 56.0);
            let cell_x = phone.x + 8.0 + i as f32 * d_cell_w;
            return Some(Rect {
                x: cell_x + (d_cell_w - d_icon) * 0.5,
                y: dock_y + (dock_h - d_icon) * 0.5,
                width: d_icon,
                height: d_icon,
            });
        }
        if !self.is_app_hidden(app)
            && let Some((_, &(raw_c, raw_r, _))) = self.home_icons.iter().enumerate().find(|(index, (_, _, candidate))| {
                *candidate == HomeApp::Builtin(app) && self.home_icon_pages[*index] == self.home_page
            })
        {
            let ws_top = phone.y + status_h + 16.0;
            let ws_rect = Rect {
                x: phone.x + 8.0,
                y: ws_top,
                width: phone.width - 16.0,
                height: (dock_y - ws_top - 10.0).max(160.0),
            };
            let (cols, rows) = self.sys.grid_dims();
            let cell_w = ws_rect.width / cols as f32;
            let cell_h = ws_rect.height / rows as f32;
            let icon_size = (cell_w * 0.72).clamp(44.0, 56.0);
            let c = raw_c.min(cols.saturating_sub(1));
            let r = raw_r.min(rows.saturating_sub(1));
            let cell_x = ws_rect.x + c as f32 * cell_w;
            let cell_y = ws_rect.y + r as f32 * cell_h;
            return Some(Rect {
                x: cell_x + (cell_w - icon_size) * 0.5,
                y: cell_y + (cell_h - icon_size - 20.0).max(4.0) * 0.5,
                width: icon_size,
                height: icon_size,
            });
        }
        None
    }

    fn set_launcher_sub(&mut self, next: LauncherSub, is_push: bool, now: f64) {
        if self.launcher_sub == next {
            return;
        }
        self.prev_launcher_sub = self.launcher_sub;
        self.launcher_sub = next;
        self.launcher_sub_push = is_push;
        let dur = 0.28;
        self.launcher_sub_anim.snap(0.0);
        self.launcher_sub_anim.set(1.0, now, dur, dur);
    }

    fn can_go_back(&self) -> bool {
        self.power_open
            || self.active_apps_open
            || self.qs_edit_open
            || self.shade_stage > 0
            || self.vol_open
            || self.task_menu_idx.is_some()
            || self.home_popup != HomePopup::None
            || self.widgets_open
            || self.drawer_open
            || self.launcher_sub != LauncherSub::None
            || self.mode != ShellMode::Home
            || self.freeform.is_some()
    }

    fn can_exit_app_to_home(&self) -> bool {
        if self.power_open
            || self.active_apps_open
            || self.qs_edit_open
            || self.shade_stage > 0
            || self.vol_open
        {
            return false;
        }
        match self.mode {
            ShellMode::Fullscreen(AppId::Settings) => {
                !self.settings_app.is_dialog_open()
                    && self.settings_app.current_page() == settings::SettingsPage::Home
            }
            ShellMode::Fullscreen(_)
            | ShellMode::FullscreenDesktop(_)
            | ShellMode::Split { .. } => true,
            _ => false,
        }
    }

    fn open_app(&mut self, app: AppId, now: f64) {
        let nav_h = match self.sys.nav_mode {
            NavMode::Gestural => 24.0,
            NavMode::ThreeButton => 44.0,
        };
        let home_ir = self.find_home_or_dock_icon_rect(app, self.last_phone_rect, 31.0, nav_h);
        let themed = self.sys.themed_icons && home_ir.is_some();
        self.open_app_from(app, home_ir, home_ir.is_some(), false, themed, now);
    }

    fn open_app_from(
        &mut self,
        app: AppId,
        origin: Option<Rect>,
        from_icon: bool,
        from_shade: bool,
        themed_icon: bool,
        now: f64,
    ) {
        if self.is_app_unavailable(app) {
            return;
        }
        if app == AppId::Settings && from_icon {
            self.settings_launch_themed = Some(themed_icon);
        }
        if !from_shade {
            self.shade_stage = 0;
            self.shade_anim.set(0.0, now, 0.24, 0.18);
        }
        self.drawer_open = false;
        self.drawer_anim.set(0.0, now, 0.24, 0.18);
        self.widgets_open = false;
        self.widgets_anim.set(0.0, now, 0.24, 0.18);
        self.home_popup = HomePopup::None;
        self.popup_anim.snap(0.0);
        self.launcher_sub = LauncherSub::None;
        self.prev_launcher_sub = LauncherSub::None;
        self.qs_edit_open = false;
        self.qs_edit_anim.snap(0.0);
        self.qs_edit_drag = None;
        self.active_apps_open = false;
        self.power_open = false;
        self.power_anim.set(0.0, now, 0.20, 0.16);
        self.task_menu_idx = None;
        self.freeform = None;
        self.preview_transition_t = None;

        self.tasks.retain(|t| *t != TaskEntry::Single(app));
        if self.native_compositor
            && let Some(d) = self.desktop_registry.find_for_builtin(app)
            && let Some(d_idx) = self.desktop_registry.apps.iter().position(|e| e.id == d.id)
        {
            self.tasks.retain(|t| *t != TaskEntry::Desktop(d_idx));
        }
        self.tasks.insert(0, TaskEntry::Single(app));
        if self.tasks.len() > 8 {
            self.tasks.truncate(8);
        }
        self.active_wayland_desktop = None;
        self.wayland_surface_ready = false;
        self.wayland_launch_frame = self.wayland_task_frame(TaskEntry::Single(app)).cloned();
        self.wayland_launch_capture_attempted = false;
        self.mode = ShellMode::Fullscreen(app);
        self.app_anim.snap(1.0);
        if self.native_compositor && !matches!(app, AppId::Files | AppId::Calendar) {
            self.host_commands.push(HostCommand::LaunchOrFocus(app));
        }

        let scale = self.sys.anim_scale();
        if scale > 0.01 {
            let start_rect = origin.unwrap_or_else(|| self.default_launch_origin());
            let base_dur = if from_shade {
                0.30
            } else if app == AppId::Settings && from_icon {
                0.46
            } else {
                0.36
            };
            self.app_transition = AppTransition::Opening {
                app,
                origin: start_rect,
                from_icon,
                from_shade,
                from_overview: false,
                themed_icon,
                start: now,
                duration: base_dur * scale as f64,
            };
        } else {
            self.app_transition = AppTransition::None;
            if from_shade {
                self.shade_stage = 0;
                self.shade_anim.snap(0.0);
            }
        }
    }

    fn default_launch_origin(&self) -> Rect {
        let phone = self.last_phone_rect;
        Rect {
            x: phone.x + phone.width * 0.5 - 32.0,
            y: phone.y + phone.height * 0.5 - 32.0,
            width: 64.0,
            height: 64.0,
        }
    }

    fn finish_app_opening(&mut self, now: f64, reduced: bool) {
        let timing = match self.app_transition {
            AppTransition::Opening { start, duration, from_shade, .. } => (start, duration, from_shade),
            AppTransition::OpeningDesktop { start, duration, .. } => (start, duration, false),
            _ => return,
        };
        let (start, duration, from_shade) = timing;
        if self.preview_transition_t.is_none() && (reduced || now - start >= duration) {
            self.app_transition = AppTransition::None;
            self.wayland_launch_frame = None;
            if from_shade {
                self.shade_stage = 0;
                self.shade_anim.snap(0.0);
            }
        }
    }

    fn open_desktop_app(&mut self, idx: usize, now: f64) {
        self.open_desktop_app_from(idx, None, None, false, now);
    }

    fn open_advanced_radio_settings(&mut self, panel: &'static str, now: f64) {
        if let Some(idx) = self.desktop_registry.apps.iter().position(|app| app.id == "org.gnome.Settings.desktop") {
            self.open_desktop_app_from(idx, Some(panel), None, false, now);
        } else {
            self.show_toast("Advanced radio settings are unavailable", now);
        }
    }

    fn open_desktop_app_from(&mut self, idx: usize, argument: Option<&'static str>, origin: Option<Rect>, from_overview: bool, now: f64) {
        let Some(entry) = self.desktop_registry.apps.get(idx) else {
            return;
        };
        let desktop_id = entry.id.clone();
        let entry_name = entry.name.clone();
        let shiraz_pkg = entry.shiraz_package.clone();
        self.shade_stage = 0;
        self.shade_anim.set(0.0, now, 0.24, 0.18);
        self.drawer_open = false;
        self.drawer_anim.set(0.0, now, 0.24, 0.18);
        self.widgets_open = false;
        self.widgets_anim.set(0.0, now, 0.24, 0.18);
        self.home_popup = HomePopup::None;
        self.popup_anim.snap(0.0);
        self.launcher_sub = LauncherSub::None;
        self.prev_launcher_sub = LauncherSub::None;
        self.qs_edit_open = false;
        self.qs_edit_anim.snap(0.0);
        self.qs_edit_drag = None;
        self.active_apps_open = false;
        self.power_open = false;
        self.power_anim.set(0.0, now, 0.20, 0.16);
        self.task_menu_idx = None;
        self.freeform = None;
        self.preview_transition_t = None;
        self.wayland_launch_frame = self.wayland_task_frame(TaskEntry::Desktop(idx)).cloned();
        self.wayland_launch_capture_attempted = false;
        self.wayland_surface_ready = false;

        self.active_wayland_app = None;
        self.active_wayland_desktop = None;
        self.tasks.retain(|t| *t != TaskEntry::Desktop(idx));
        if self.native_compositor {
            let dup_builtin: Vec<AppId> = AppId::ALL
                .iter()
                .copied()
                .filter(|&a| {
                    self.desktop_registry
                        .find_for_builtin(a)
                        .is_some_and(|d| d.id == desktop_id)
                })
                .collect();
            for a in dup_builtin {
                self.tasks.retain(|t| *t != TaskEntry::Single(a));
            }
        }
        self.tasks.insert(0, TaskEntry::Desktop(idx));
        if self.tasks.len() > 8 {
            self.tasks.truncate(8);
        }
        self.mode = ShellMode::FullscreenDesktop(idx);
        self.app_anim.snap(1.0);
        let scale = f64::from(self.sys.anim_scale());
        self.app_transition = if scale > 0.01 {
            AppTransition::OpeningDesktop {
                origin: origin.unwrap_or_else(|| self.default_launch_origin()),
                from_icon: origin.is_some() && !from_overview,
                from_overview,
                start: now,
                duration: (if from_overview { 0.34 } else { 0.36 }) * scale,
            }
        } else {
            AppTransition::None
        };
        if let Some(pkg) = shiraz_pkg {
            if !self.sys.shiraz_enabled {
                self.sys.shiraz_enabled = true;
                self.sys.save_persisted_settings();
                self.show_toast(format!("Enabling Shiraz for {entry_name}…"), now);
            }
            if self.native_compositor {
                self.host_commands.push(HostCommand::ShirazLaunchPackage {
                    package: pkg,
                    desktop_id: Some(desktop_id),
                });
            }
        } else if self.native_compositor {
            self.host_commands
                .push(HostCommand::LaunchDesktopApp(desktop_id, argument));
        }
    }

    /// Hot-reload Freedesktop `.desktop` entries (e.g. newly installed Shiraz Android APKs)
    /// while safely remapping active task & mode indices by stable `desktop_id`.
    pub fn refresh_desktop_apps_if_changed(&mut self) -> bool {
        let active_id = self
            .active_wayland_desktop
            .and_then(|i| self.desktop_registry.apps.get(i))
            .map(|a| a.id.clone());
        let mode_id = match self.mode {
            ShellMode::FullscreenDesktop(i) => {
                self.desktop_registry.apps.get(i).map(|a| a.id.clone())
            }
            _ => None,
        };
        let task_ids: Vec<Option<String>> = self
            .tasks
            .iter()
            .map(|t| match *t {
                TaskEntry::Desktop(i) => self.desktop_registry.apps.get(i).map(|a| a.id.clone()),
                _ => None,
            })
            .collect();

        if !self.desktop_registry.reload_if_changed() {
            return false;
        }

        self.settings_apps = None;
        self.active_wayland_desktop = active_id.and_then(|id| {
            self.desktop_registry
                .apps
                .iter()
                .position(|a| a.id == id)
        });
        if let Some(mid) = mode_id {
            if let Some(new_idx) = self.desktop_registry.apps.iter().position(|a| a.id == mid) {
                self.mode = ShellMode::FullscreenDesktop(new_idx);
            } else {
                self.mode = ShellMode::Home;
                self.app_transition = AppTransition::None;
            }
        }
        let mut remapped_tasks = Vec::with_capacity(self.tasks.len());
        for (task, maybe_id) in self.tasks.iter().copied().zip(task_ids.into_iter()) {
            match task {
                TaskEntry::Desktop(_) => {
                    if let Some(id) = maybe_id
                        && let Some(new_idx) =
                            self.desktop_registry.apps.iter().position(|a| a.id == id)
                    {
                        remapped_tasks.push(TaskEntry::Desktop(new_idx));
                    }
                }
                other => remapped_tasks.push(other),
            }
        }
        self.tasks = remapped_tasks;
        true
    }

    fn open_split(&mut self, top: AppId, bottom: AppId, now: f64) {
        if top == bottom {
            self.open_app(top, now);
            return;
        }
        self.tasks.retain(|t| *t != TaskEntry::SplitPair(top, bottom));
        self.tasks.insert(0, TaskEntry::SplitPair(top, bottom));
        self.split_ratio = 0.50;
        self.mode = ShellMode::Split { top, bottom };
        self.task_menu_idx = None;
        self.app_transition = AppTransition::None;
        self.app_anim.snap(0.0);
        self.app_anim.set(1.0, now, 0.28, 0.20);
    }

    fn open_freeform(&mut self, app: AppId, phone: Rect, now: f64) {
        let w = (phone.width * 0.62).clamp(220.0, 290.0);
        let h = (phone.height * 0.54).clamp(320.0, 480.0);
        self.freeform = Some(FreeformWin {
            app,
            x: phone.x + 16.0,
            y: phone.y + 44.0,
            w,
            h,
        });
        self.mode = ShellMode::Home;
        self.app_transition = AppTransition::None;
        self.task_menu_idx = None;
        self.overview_anim.set(0.0, now, 0.24, 0.20);
    }

    fn go_home(&mut self, now: f64) {
        self.go_home_with_rect(None, now);
    }

    fn go_home_with_rect(&mut self, from_override: Option<(Rect, f32)>, now: f64) {
        if self.mode == ShellMode::Overview && !self.tasks.is_empty() {
            self.overview_exiting = true;
            self.overview_enter_from = None;
            self.overview_vertical_settle = None;
        }
        let closing_app = match self.mode {
            ShellMode::Fullscreen(app) => Some(app),
            ShellMode::Split { top, .. } => Some(top),
            _ => None,
        };
        let was_sub = self.launcher_sub;
        self.shade_stage = 0;
        self.shade_anim.set(0.0, now, 0.26, 0.20);
        self.qs_edit_open = false;
        self.qs_edit_anim.set(0.0, now, 0.24, 0.20);
        self.qs_edit_drag = None;
        self.active_apps_open = false;
        self.power_open = false;
        self.power_anim.set(0.0, now, 0.20, 0.16);
        self.vol_open = false;
        self.vol_expanded = false;
        self.vol_anim.set(0.0, now, 0.20, 0.16);
        self.vol_expand_anim.set(0.0, now, 0.20, 0.16);
        self.drawer_open = false;
        self.drawer_anim.set(0.0, now, 0.28, 0.20);
        self.ime_target = None;
        self.ime_anim.set(0.0, now, 0.20, 0.18);
        self.widgets_open = false;
        self.widgets_anim.set(0.0, now, 0.28, 0.20);
        if self.home_popup != HomePopup::None {
            self.last_home_popup = self.home_popup;
            self.home_popup = HomePopup::None;
            self.popup_anim.set(0.0, now, 0.18, 0.13);
        }
        if was_sub != LauncherSub::None {
            self.set_launcher_sub(LauncherSub::None, false, now);
        }
        self.task_menu_idx = None;
        self.mode = ShellMode::Home;
        self.active_wayland_app = None;
        self.active_wayland_desktop = None;
        if self.native_compositor {
            self.host_commands.push(HostCommand::MinimizeAll);
        }
        let overview_close = if self.overview_exiting { 0.24 } else { 0.20 };
        self.overview_anim.set(0.0, now, 0.24, overview_close);
        self.preview_transition_t = None;

        let scale = self.sys.anim_scale();
        if let Some(app) = closing_app
            && scale > 0.01
        {
            let phone = self.last_phone_rect;
            let nav_h = match self.sys.nav_mode {
                NavMode::Gestural => 24.0,
                NavMode::ThreeButton => 44.0,
            };
            let target_icon = self.find_home_or_dock_icon_rect(app, phone, 31.0, nav_h);
            let (from_rect, from_rad) = from_override.unwrap_or((phone, 12.0));
            self.app_transition = AppTransition::Closing {
                app,
                from_rect,
                from_rad,
                target_icon,
                themed_icon: self.sys.themed_icons,
                start: now,
                duration: if app == AppId::Settings && target_icon.is_some() {
                    SETTINGS_RETURN_SECONDS
                } else {
                    0.34
                } * scale as f64,
            };
        } else {
            self.app_transition = AppTransition::None;
        }
    }

    fn settings_nav_preview(&self, phone: Rect, delta: [f32; 2], hold_time: f64) -> (Rect, f32) {
        let up = (-delta[1]).max(0.0);
        let drag = (up / 300.0).clamp(0.0, 1.0);
        let hold = smoothstep(0.02, 0.18, hold_time as f32) * smoothstep(68.0, 155.0, up);
        let scale = lerp(lerp(1.0, 0.67, drag), 0.52, hold);
        let width = phone.width * scale;
        let height = phone.height * scale;
        (Rect {
            x: phone.x + (phone.width - width) * 0.5 + delta[0] * 0.44,
            y: phone.y + (phone.height - height) * 0.5 - 44.0 * drag * (1.0 - hold),
            width,
            height,
        }, lerp(22.0 * drag, 22.0, hold))
    }

    fn overview_drag_progress(offset: f32, phone: Rect) -> f32 {
        (offset / (phone.height * 0.16).max(1.0)).clamp(0.0, 1.0)
    }

    fn overview_drag_tint(progress: f32) -> f32 {
        1.0 - smoothstep(0.0, 1.0, progress)
    }

    fn reopen_overview_task(&mut self, task: TaskEntry, origin: Rect, drag_progress: f32, now: f64) {
        self.overview_vertical_settle = None;
        match task {
            TaskEntry::Single(app) => {
                self.open_app_from(app, Some(origin), false, false, false, now);
                if let AppTransition::Opening { from_overview, duration, .. } = &mut self.app_transition {
                    *from_overview = true;
                    let remaining = 1.0 - drag_progress;
                    *duration = if remaining < 0.001 {
                        0.0
                    } else {
                        0.34 * (remaining as f64).max(0.28) * self.sys.anim_scale() as f64
                    };
                    self.overview_reopen_progress = drag_progress;
                }
            }
            TaskEntry::SplitPair(a, b) => self.open_split(a, b, now),
            TaskEntry::Desktop(idx) => {
                self.open_desktop_app_from(idx, None, Some(origin), true, now);
                self.overview_reopen_progress = drag_progress;
            }
        }
    }

    fn dismiss_overview_task(&mut self, task_idx: usize, now: f64) {
        let removed = self.tasks.remove(task_idx);
        if self.native_compositor {
            match removed {
                TaskEntry::Single(a) => {
                    self.host_commands.push(HostCommand::CloseApp(a));
                }
                TaskEntry::SplitPair(a, b) => {
                    self.host_commands.push(HostCommand::CloseApp(a));
                    self.host_commands.push(HostCommand::CloseApp(b));
                }
                TaskEntry::Desktop(idx) => {
                    if let Some(entry) = self.desktop_registry.apps.get(idx) {
                        self.host_commands.push(HostCommand::CloseDesktopApp(
                            entry.id.clone(),
                        ));
                    }
                }
            }
        }
        let label = match removed {
            TaskEntry::Single(a) => format!("Closed {}", a.name()),
            TaskEntry::SplitPair(a, b) => {
                format!("Closed {} + {}", a.name(), b.name())
            }
            TaskEntry::Desktop(idx) => self
                .desktop_registry
                .apps
                .get(idx)
                .map(|e| format!("Closed {}", e.name))
                .unwrap_or_else(|| "Closed app".into()),
        };
        self.show_toast(label, now);
        if self.tasks.is_empty() {
            self.go_home(now);
        } else {
            let step = self.last_overview_step.max(220.0);
            let new_max = ((self.tasks.len().saturating_sub(1)) as f32 * step
                + 116.0)
                .max(0.0);
            let clamped = self.overview_scroll.clamp(0.0, new_max);
            let snapped = ((clamped / step).round() * step).clamp(0.0, new_max);
            self.overview_glide = Some(OverviewGlide::start(
                self.overview_scroll,
                snapped,
                0.0,
                now,
            ));
        }
    }

    fn open_overview(&mut self, now: f64) {
        self.open_overview_from(None, now);
    }

    fn open_overview_from(&mut self, from_rect: Option<(Rect, f32)>, now: f64) {
        let from_empty_home = self.mode == ShellMode::Home && self.tasks.is_empty();
        self.overview_exiting = false;
        self.shade_stage = 0;
        self.shade_anim.set(0.0, now, 0.24, 0.18);
        self.qs_edit_open = false;
        self.qs_edit_anim.snap(0.0);
        self.qs_edit_drag = None;
        self.vol_open = false;
        self.vol_expanded = false;
        self.vol_anim.set(0.0, now, 0.20, 0.16);
        self.vol_expand_anim.set(0.0, now, 0.20, 0.16);
        self.drawer_open = false;
        self.drawer_anim.set(0.0, now, 0.24, 0.18);
        self.widgets_open = false;
        self.widgets_anim.set(0.0, now, 0.24, 0.18);
        self.home_popup = HomePopup::None;
        self.launcher_sub = LauncherSub::None;
        self.power_open = false;
        self.power_anim.set(0.0, now, 0.18, 0.16);
        self.task_menu_idx = None;
        if let ShellMode::Split { top, bottom } = self.mode {
            self.tasks.retain(|t| *t != TaskEntry::SplitPair(top, bottom));
            self.tasks.insert(0, TaskEntry::SplitPair(top, bottom));
        }
        self.mode = ShellMode::Overview;
        self.active_wayland_app = None;
        self.active_wayland_desktop = None;
        if self.native_compositor {
            self.host_commands.push(HostCommand::MinimizeAll);
        }
        self.overview_scroll = 0.0;
        self.overview_edge_spring.reset();
        self.overview_enter_from = from_rect;
        if !from_empty_home {
            self.overview_anim.snap(0.0);
        }
        self.overview_anim.set(1.0, now, 0.32, 0.24);
    }

    fn compact_notification_sheet_top(phone: Rect, status_h: f32) -> f32 {
        phone.y + status_h + 12.0 + 2.0 * QS_COMPACT_TILE_HEIGHT + QS_ROW_GAP + 28.0
    }

    fn settle_shade(&mut self, stage: u8, now: f64) {
        if stage == 1 && self.shade_stage == 0 {
            let delay = if self.shade_anim.value < 1.0 {
                0.22 * f64::from(self.sys.anim_scale())
            } else {
                0.0
            };
            self.notification_sheet_bounce.snap(0.0);
            self.notification_sheet_bounce.set(1.0, now + delay, 0.16, 0.16);
        }
        let open_duration = if stage == 2 { 0.40 } else { 0.28 };
        let close_duration = if stage == 1 && self.shade_anim.value > 1.0 {
            0.24 * f64::from((self.shade_anim.value - 1.0).clamp(0.15, 1.0))
        } else {
            0.24
        };
        self.shade_stage = stage;
        self.shade_anim.set(stage as f32, now, open_duration, close_duration);
    }

    fn tap_shade_header(&mut self, now: f64) {
        self.settle_shade(if self.shade_stage == 1 { 2 } else { 1 }, now);
    }

    fn go_back(&mut self, now: f64) {
        self.go_back_with_rect(None, now);
    }

    fn go_back_with_rect(&mut self, from_override: Option<(Rect, f32)>, now: f64) {
        if self.power_open {
            self.power_open = false;
            self.power_anim.set(0.0, now, 0.20, 0.16);
            return;
        }
        if self.active_apps_open {
            self.active_apps_open = false;
            return;
        }
        if self.ime_target.is_some() {
            self.ime_target = None;
            self.ime_anim.set(0.0, now, 0.20, 0.18);
            return;
        }
        if self.qs_edit_open {
            self.qs_edit_open = false;
            self.qs_edit_anim.set(0.0, now, 0.28, 0.24);
            self.qs_edit_drag = None;
            return;
        }
        if self.shade_stage > 0 {
            self.settle_shade(self.shade_stage - 1, now);
            return;
        }
        if self.vol_open {
            if self.vol_expanded {
                self.vol_expanded = false;
                self.vol_expand_anim.set(0.0, now, 0.22, 0.18);
            } else {
                self.vol_open = false;
                self.vol_anim.set(0.0, now, 0.20, 0.16);
            }
            return;
        }
        if self.task_menu_idx.is_some() {
            self.task_menu_idx = None;
            return;
        }
        if self.home_popup != HomePopup::None {
            self.last_home_popup = self.home_popup;
            self.home_popup = HomePopup::None;
            self.popup_anim.set(0.0, now, 0.18, 0.13);
            return;
        }
        if self.widgets_open {
            self.widgets_open = false;
            self.widgets_anim.set(0.0, now, 0.26, 0.20);
            return;
        }
        if self.drawer_open {
            if !self.drawer_search.is_empty() {
                self.drawer_search.clear();
            } else {
                self.drawer_open = false;
                self.drawer_anim.set(0.0, now, 0.26, 0.20);
            }
            return;
        }
        if self.launcher_sub != LauncherSub::None {
            let next = match self.launcher_sub {
                LauncherSub::WallpaperPicker | LauncherSub::WallpaperSettings(_) => LauncherSub::WallpaperStyle,
                LauncherSub::WallpaperFiles => {
                    if self.wallpaper_load.is_none() && self.wallpaper_files.back() { return; }
                    self.leave_wallpaper_files(now);
                    return;
                }
                LauncherSub::WallpaperPreview => {
                    if let Some(pending) = self.wallpaper_pending.as_mut() {
                        if pending.show_apply {
                            pending.show_apply = false;
                            return;
                        }
                        if pending.edit || pending.show_info {
                            pending.edit = false;
                            pending.show_info = false;
                            return;
                        }
                    }
                    self.wallpaper_pending.as_ref().map_or(LauncherSub::WallpaperStyle, |pending| pending.return_to)
                }
                LauncherSub::HiddenApps => LauncherSub::HomeSettings,
                _ => LauncherSub::None,
            };
            self.set_launcher_sub(next, false, now);
            return;
        }
        match self.mode {
            ShellMode::SplitSelect(_) => self.mode = ShellMode::Overview,
            ShellMode::Overview => self.go_home(now),
            ShellMode::Fullscreen(AppId::Settings) => {
                if !self.settings_app.go_back(now) {
                    self.go_home_with_rect(from_override, now);
                }
            }
            ShellMode::Fullscreen(app) => {
                if self.native_compositor && self.is_wayland_passthrough() {
                    self.host_commands.push(HostCommand::SendBack);
                } else if !self.builtin_apps.handle_back(app) {
                    self.go_home_with_rect(from_override, now);
                }
            }
            ShellMode::FullscreenDesktop(_) => {
                if self.native_compositor && self.is_wayland_passthrough() {
                    self.host_commands.push(HostCommand::SendBack);
                } else {
                    self.go_home_with_rect(from_override, now);
                }
            }
            ShellMode::Split { .. } => self.go_home_with_rect(from_override, now),
            ShellMode::Home => {
                if self.freeform.is_some() {
                    self.freeform = None;
                }
            }
        }
    }

    fn open_lock_shortcut(&mut self, shortcut: usize, now: f64) {
        match shortcut {
            2 | 3 => {
                self.builtin_apps.set_camera_mode(shortcut == 2);
                self.open_app(AppId::Camera, now);
            }
            6 => {
                self.go_home(now);
                self.settle_shade(2, now);
            }
            _ => {}
        }
    }

    fn apply_shell_action(&mut self, action: ShellAction, now: f64) {
        match action {
            ShellAction::None => {}
            ShellAction::OpenApp(app) => self.open_app(app, now),
            ShellAction::JoinWifi(network) => self.host_commands.push(HostCommand::JoinWifi(network)),
            ShellAction::ConnectBluetooth(address, connect) => self.host_commands.push(HostCommand::ConnectBluetooth(address, connect)),
            ShellAction::AdvancedRadioSettings(panel) => self.open_advanced_radio_settings(panel, now),
            ShellAction::FocusTextField(id) => self.open_ime_for(id),
            ShellAction::BlurTextField => self.blur_app_text_field(now),
            ShellAction::LaunchFilesWindow(folder) => {
                if self.native_compositor { self.host_commands.push(HostCommand::LaunchFilesWindow(folder)); }
                else { self.show_toast("Open Files in a Wayland session to use another window", now); }
            }
            ShellAction::OpenFile(path) if self.native_compositor => self.host_commands.push(HostCommand::OpenFile(path)),
            ShellAction::ShareFiles(paths) if self.native_compositor => self.host_commands.push(HostCommand::ShareFiles(paths)),
            ShellAction::OpenFile(path) => {
                if let Err(error) = std::process::Command::new("xdg-open").arg(path).spawn() {
                    self.show_toast(format!("Could not open file: {error}"), now);
                }
            }
            ShellAction::ShareFiles(paths) => {
                let message = match spectre_files::share_to_clipboard(&paths, None) {
                    Ok(()) => "File links copied".into(),
                    Err(error) => format!("Could not copy file links: {error}"),
                };
                self.show_toast(message, now);
            }
            ShellAction::OpenDesktopApp(idx) => self.open_desktop_app(idx, now),
            ShellAction::OpenSettingsSub(page) => {
                self.settings_app.navigate(page, now);
                self.open_app(AppId::Settings, now);
            }
            ShellAction::OpenAppInfo(app) => {
                self.settings_app
                    .navigate(settings::SettingsPage::AppInfo(app.index()), now);
                self.open_app(AppId::Settings, now);
            }
            ShellAction::ShowToast(msg) => self.show_toast(msg, now),
            ShellAction::PostNotification { app, title, body } => {
                self.post_notification(app, title, body);
            }
            ShellAction::ClearNotificationHistory => {
                self.notif_history.clear();
                notifications::save_notification_history(&self.notif_history);
                self.show_toast("Notification history cleared", now);
            }
            ShellAction::OpenVolume => {
                self.vol_open = true;
                self.vol_anim.set(1.0, now, 0.22, 0.18);
                self.vol_hide_at = now + 4.5;
            }
            ShellAction::GoHome => self.go_home(now),
            ShellAction::GoBack => self.go_back(now),
            ShellAction::LockNow => self.lockscreen.lock(),
            ShellAction::ShirazLaunchPackage(pkg) => {
                if let Some(idx) = self
                    .desktop_registry
                    .apps
                    .iter()
                    .position(|a| a.shiraz_package.as_deref() == Some(pkg.as_str()))
                {
                    self.open_desktop_app(idx, now);
                } else {
                    if !self.sys.shiraz_enabled {
                        self.sys.shiraz_enabled = true;
                        self.sys.save_persisted_settings();
                    }
                    self.host_commands.push(HostCommand::ShirazLaunchPackage {
                        package: pkg.clone(),
                        desktop_id: None,
                    });
                    self.show_toast(format!("Launching {pkg}…"), now);
                }
            }
            ShellAction::ShirazStopPackage(pkg) => {
                self.host_commands
                    .push(HostCommand::ShirazStopPackage(pkg.clone()));
                self.show_toast(format!("Stopped {pkg}"), now);
            }
            ShellAction::ShirazStopAll => {
                self.host_commands.push(HostCommand::ShirazStopAll);
                self.show_toast("Stopped all Android apps", now);
            }
            ShellAction::ShirazRefresh => {
                self.host_commands.push(HostCommand::ShirazRefresh);
                let _ = self.refresh_desktop_apps_if_changed();
                self.show_toast("Optimizing Shiraz memory & AOT cache…", now);
            }
        }
    }

    fn take_screenshot(&mut self, now: f64) {
        self.power_open = false;
        self.power_anim.set(0.0, now, 0.16, 0.12);
        self.shade_stage = 0;
        self.shade_anim.set(0.0, now, 0.20, 0.16);
        self.flash_until = now + 0.32;
        self.post_notification(
            "Android System".into(),
            "Screenshot saved".into(),
            "Captured 411×846 dp screen · Tap to view in Gallery".into(),
        );
        self.show_toast("Screenshot saved", now);
    }

    fn scroll_mut_for(&mut self, target: ScrollTarget) -> (&mut f32, f32) {
        match target {
            ScrollTarget::Drawer => (&mut self.drawer_scroll, self.last_scroll_max[0]),
            ScrollTarget::ShadeNotifs => (&mut self.shade_scroll, self.last_scroll_max[1]),
            ScrollTarget::Widgets => (&mut self.widgets_scroll, self.last_scroll_max[2]),
            ScrollTarget::WallpaperStyle if self.launcher_sub == LauncherSub::WallpaperPicker =>
                (&mut self.wallpaper_picker_scroll, self.last_scroll_max[3]),
            ScrollTarget::WallpaperStyle => (&mut self.wallpaper_scroll, self.last_scroll_max[3]),
            ScrollTarget::HomeSettings => (&mut self.home_settings_scroll, self.last_scroll_max[4]),
            ScrollTarget::QsEdit => (&mut self.qs_edit_scroll, self.last_scroll_max[5]),
            ScrollTarget::Settings => self.settings_app.scroll_state_mut(),
        }
    }

    fn current_scroll_and_max(&mut self, target: ScrollTarget) -> (f32, f32) {
        let (s, m) = self.scroll_mut_for(target);
        (*s, m)
    }

    fn target_stretch(&self, target: ScrollTarget) -> (f32, f32) {
        if let Some((t, d)) = self.preview_stretch
            && t == target
        {
            let mut st = EdgeStretch::default();
            let _ = st.pull(d * 800.0, 800.0);
            return st.scale_and_sign();
        }
        self.scroll_stretches[target as usize].scale_and_sign()
    }

    fn step_scroll_fling(&mut self, dt: f32, viewport_h: f32, reduced: bool) -> bool {
        let Some((target, mut vel)) = self.scroll_fling else {
            return false;
        };
        if reduced || vel.abs() < 24.0 || dt <= 0.0 {
            self.scroll_fling = None;
            return false;
        }
        let (scroll, max_s) = self.scroll_mut_for(target);
        let next = *scroll + vel * dt;
        if next <= 0.0 {
            *scroll = 0.0;
            // Fling slammed into TOP edge -> absorb into Android 13 EdgeStretch!
            self.scroll_stretches[target as usize].absorb(-vel, viewport_h);
            self.scroll_fling = None;
            return true;
        } else if next >= max_s {
            *scroll = max_s;
            // Fling slammed into BOTTOM edge -> absorb into Android 13 EdgeStretch!
            self.scroll_stretches[target as usize].absorb(-vel, viewport_h);
            self.scroll_fling = None;
            return true;
        }
        *scroll = next;
        vel *= (-5.2 * dt).exp();
        if vel.abs() < 24.0 {
            self.scroll_fling = None;
            false
        } else {
            self.scroll_fling = Some((target, vel));
            true
        }
    }

    fn classify_press(
        &mut self,
        pt: [f32; 2],
        phone: Rect,
        status_h: f32,
        nav_h: f32,
    ) -> (PressContext, ActiveGesture) {
        if !phone.contains(pt) {
            return (PressContext::None, ActiveGesture::None);
        }
        if self.power_open
            || self.power_anim.value > 0.08
            || self.active_apps_open
        {
            return (PressContext::None, ActiveGesture::None);
        }

        // 1. Volume side panel sliders (highest z-order when open)
        if self.vol_open || self.vol_anim.value > 0.08 {
            for entry in self.last_vol_tracks.into_iter().flatten() {
                if entry.1.contains(pt) {
                    return (
                        PressContext::None,
                        ActiveGesture::VolumeSlider {
                            channel: entry.0,
                            track: entry.1,
                        },
                    );
                }
            }
            let vol_w = if self.vol_expanded { 256.0 } else { 68.0 };
            if pt[0] >= phone.x + phone.width - vol_w - 10.0
                && pt[1] >= phone.y + phone.height * 0.24
                && pt[1] <= phone.y + phone.height * 0.78
            {
                return (PressContext::None, ActiveGesture::None);
            }
        }

        // 1b. Android 13 Edge-Back gesture zone on left/right phone edges (works everywhere on WebXDC!)
        let on_qs_edit_btn = self.shade_anim.value > 1.45
            && !self.qs_edit_open
            && self.last_qs_edit_btn.width > 0.0
            && self.last_qs_edit_btn.contains(pt);
        let edge_strip = if self.shows_wayland_surface() { 22.0 } else { 18.0 };
        if self.can_go_back()
            && !on_qs_edit_btn
            && !self.vol_open
            && self.vol_anim.value < 0.08
            && pt[1] >= phone.y + status_h + 14.0
            && pt[1] <= phone.y + phone.height - nav_h - 14.0
        {
            if pt[0] <= phone.x + edge_strip {
                return (PressContext::EdgeBack { right_edge: false }, ActiveGesture::None);
            }
            if pt[0] >= phone.x + phone.width - edge_strip {
                return (PressContext::EdgeBack { right_edge: true }, ActiveGesture::None);
            }
        }

        // 2. QS Tile Editor
        if self.qs_edit_open {
            for &(idx, rect) in &self.last_qs_edit_tiles {
                if rect.contains(pt) {
                    return (PressContext::QsEditTile { idx }, ActiveGesture::None);
                }
            }
            if pt[1] >= phone.y + status_h + 67.0 {
                return (
                    PressContext::ScrollList {
                        target: ScrollTarget::QsEdit,
                    },
                    ActiveGesture::None,
                );
            }
            return (PressContext::None, ActiveGesture::None);
        }

        // 3. Open Notification Shade / Expanded Quick Settings (drag ANYWHERE!)
        if self.shade_anim.value > 0.05 || self.shade_stage > 0 {
            let s2 = (self.shade_anim.value - 1.0).clamp(0.0, 1.0);
            // Direct capture on Brightness slider when expanded enough
            if s2 > 0.35 && self.last_brightness_rect.width > 0.0 && self.last_brightness_rect.contains(pt) {
                return (
                    PressContext::None,
                    ActiveGesture::BrightnessSlider {
                        bar: self.last_brightness_rect,
                    },
                );
            }
            // Horizontal page swipe or vertical collapse on 2x4 QS grid
            if s2 > 0.45 && self.last_qs_grid_rect.width > 0.0 && self.last_qs_grid_rect.contains(pt) {
                return (PressContext::QsGrid, ActiveGesture::None);
            }
            // Notification list / card swipe in Stage 1
            if s2 < 0.55 {
                let in_notifs = self.last_notif_container.width > 0.0
                    && self.last_notif_container.contains(pt);
                let mut hit_notif = None;
                if in_notifs {
                    for &(id, ongoing, rect) in &self.last_notif_rects {
                        if rect.contains(pt) {
                            hit_notif = Some((id, ongoing));
                            break;
                        }
                    }
                }
                return (
                    PressContext::ShadeBody {
                        in_notifs,
                        notif: hit_notif,
                    },
                    ActiveGesture::None,
                );
            }
            return (
                PressContext::ShadeBody {
                    in_notifs: false,
                    notif: None,
                },
                ActiveGesture::None,
            );
        }

        // 4. Top Status Bar pull-down trigger (works on EVERY screen & app!)
        if pt[1] <= phone.y + status_h + 14.0 {
            return (PressContext::StatusBar, ActiveGesture::None);
        }

        // 5. Bottom Gesture Navigation Pill (works in Gestural mode on EVERY screen & app!)
        if self.sys.nav_mode == NavMode::Gestural && pt[1] >= phone.y + phone.height - 34.0 {
            let from_app = matches!(
                self.mode,
                ShellMode::Fullscreen(_)
                    | ShellMode::FullscreenDesktop(_)
                    | ShellMode::Split { .. }
            );
            return (PressContext::NavBar { from_app }, ActiveGesture::None);
        }

        // 6. Freeform Floating Window (move caption bar or corner resize grip)
        if self.mode == ShellMode::Home
            && let Some(win) = self.freeform
        {
            let frame = Rect {
                x: win.x,
                y: win.y,
                width: win.w,
                height: win.h,
            };
            let grip = Rect {
                x: frame.x + frame.width - 28.0,
                y: frame.y + frame.height - 28.0,
                width: 30.0,
                height: 30.0,
            };
            if grip.contains(pt) {
                return (
                    PressContext::None,
                    ActiveGesture::FreeformResize {
                        origin_ptr: pt,
                        origin_size: [win.w, win.h],
                    },
                );
            }
            let drag_bar = Rect {
                x: frame.x,
                y: frame.y,
                width: (frame.width - 76.0).max(40.0),
                height: 38.0,
            };
            if drag_bar.contains(pt) {
                return (
                    PressContext::None,
                    ActiveGesture::FreeformMove {
                        offset: [pt[0] - win.x, pt[1] - win.y],
                    },
                );
            }
            if frame.contains(pt) {
                return (PressContext::None, ActiveGesture::None);
            }
        }

        // 7. Split-Screen Divider direct grab
        if let ShellMode::Split { .. } = self.mode {
            let app_y = phone.y + status_h;
            let app_h = (phone.height - status_h - nav_h).max(120.0);
            let div_h = 16.0;
            let top_h = ((app_h - div_h) * self.split_ratio).clamp(96.0, app_h - 112.0);
            let div_hit = Rect {
                x: phone.x,
                y: app_y + top_h - 10.0,
                width: phone.width,
                height: div_h + 20.0,
            };
            if div_hit.contains(pt) {
                return (PressContext::None, ActiveGesture::SplitDivider);
            }
        }

        // 8. Overview / Recents Carousel (horizontal swipe & vertical swipe-up dismiss!)
        if matches!(self.mode, ShellMode::Overview | ShellMode::SplitSelect(_))
            && self.task_menu_idx.is_none()
        {
            let mut hit_idx = None;
            for &(idx, rect) in &self.last_overview_cards {
                if rect.contains(pt) {
                    hit_idx = Some(idx);
                    break;
                }
            }
            if hit_idx.is_some()
                || (self.last_overview_rect.width > 0.0 && self.last_overview_rect.contains(pt))
            {
                return (
                    PressContext::OverviewCard {
                        task_idx: hit_idx,
                        step: self.last_overview_step.max(200.0),
                        max_scroll: self.last_overview_max,
                    },
                    ActiveGesture::None,
                );
            }
            return (PressContext::None, ActiveGesture::None);
        }

        // 9. Settings Fullscreen App list drag-scrolling & EdgeStretch overscroll
        if matches!(self.mode, ShellMode::Fullscreen(AppId::Settings))
            && !self.settings_app.is_dialog_open()
            && pt[1] > phone.y + status_h + 8.0
        {
            return (
                PressContext::ScrollList {
                    target: ScrollTarget::Settings,
                },
                ActiveGesture::None,
            );
        }

        // 10. Home Screen & Launcher Overlays
        if self.mode == ShellMode::Home {
            if self.home_popup != HomePopup::None {
                return (PressContext::None, ActiveGesture::None);
            }
            match self.launcher_sub {
                LauncherSub::WallpaperStyle | LauncherSub::WallpaperPicker => {
                    return (
                        PressContext::ScrollList {
                            target: ScrollTarget::WallpaperStyle,
                        },
                        ActiveGesture::None,
                    );
                }
                LauncherSub::HomeSettings | LauncherSub::HiddenApps => {
                    return (
                        PressContext::ScrollList {
                            target: ScrollTarget::HomeSettings,
                        },
                        ActiveGesture::None,
                    );
                }
                LauncherSub::WallpaperFiles | LauncherSub::WallpaperPreview | LauncherSub::WallpaperSettings(_) => {
                    return (PressContext::None, ActiveGesture::None);
                }
                LauncherSub::None => {}
            }
            if self.widgets_open || self.widgets_anim.value > 0.08 {
                let sheet_y = phone.y
                    + status_h
                    + 10.0
                    + (1.0 - self.widgets_anim.value.clamp(0.0, 1.0)) * phone.height * 0.85;
                if pt[1] >= sheet_y {
                    return (
                        PressContext::WidgetsBody {
                            in_header: pt[1] < sheet_y + 135.0,
                        },
                        ActiveGesture::None,
                    );
                }
                return (PressContext::None, ActiveGesture::None);
            }
            if self.drawer_open || self.drawer_anim.value > 0.08 {
                return (
                    PressContext::DrawerBody {
                        in_header: pt[1] < phone.y + status_h + 74.0,
                    },
                    ActiveGesture::None,
                );
            }

            // Plain Home screen: icons, dock icons, or blank workspace
            for &(ws_idx, rect, cell_center) in &self.last_home_icon_rects {
                if rect.contains(pt) {
                    return (
                        PressContext::HomeIcon {
                            ws_idx,
                            anchor: [cell_center[0], cell_center[1] - 15.0],
                            cell_center,
                        },
                        ActiveGesture::None,
                    );
                }
            }
            for &(app, rect) in &self.last_dock_icon_rects {
                if rect.contains(pt) {
                    return (
                        PressContext::DockIcon {
                            app,
                            anchor: [rect.x + rect.width * 0.5, rect.y + 8.0],
                        },
                        ActiveGesture::None,
                    );
                }
            }
            return (PressContext::HomeBlank, ActiveGesture::None);
        }

        (PressContext::None, ActiveGesture::None)
    }

    fn apply_volume_slider(&mut self, channel: u8, track: Rect, py: f32, now: f64) {
        let val = (1.0 - (py - track.y) / track.height.max(1.0)).clamp(0.0, 1.0);
        match channel {
            0 => self.sys.vol_media = val,
            1 => self.sys.vol_alarm = val,
            2 => self.sys.vol_notif = val,
            _ => self.sys.vol_ring = val,
        }
        self.vol_hide_at = now + 4.5;
    }

    fn process_pointer_gestures(
        &mut self,
        f: &mut Frame<'_>,
        input: Input,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
        now: f64,
        dt: f32,
    ) -> bool {
        let shade_span = (phone.height * 0.34).clamp(235.0, 305.0);
        let drawer_span = (phone.height * 0.50).clamp(260.0, 440.0);
        let widgets_span = (phone.height * 0.55).clamp(280.0, 460.0);
        let mut suppress_child_input = false;

        // 1. Handle pointer press edge (ACTION_DOWN)
        if input.pressed
            && !input.cancelled
            && !input.blurred
            && let Some(pt) = input.pointer
        {
            self.scroll_fling = None;
            if self.ime_target.is_some() {
                let kb_top = phone.y + phone.height - nav_h - 236.0;
                let in_top_search = pt[1] <= phone.y + status_h + 76.0;
                if pt[1] < kb_top && !in_top_search && !self.last_ime_field_rect.contains(pt) {
                    self.ime_target = None;
                    self.ime_anim.set(0.0, now, 0.20, 0.18);
                }
            }
            self.overview_glide = None;
            self.preview_notif_swipe = None;
            self.preview_edge_back = None;
            self.preview_stretch = None;
            self.pointer_down = true;
            self.press_origin = pt;
            self.press_time = now;
            self.long_press_fired = false;
            self.vel_tracker.clear();
            self.vel_tracker.push(now, pt);

            let (ctx, immediate) = self.classify_press(pt, phone, status_h, nav_h);
            // Trigger authentic Android radial ink ripple immediately on notification touch-down!
            if let PressContext::ShadeBody {
                notif: Some((nid, _)),
                ..
            } = ctx
                && let Some(n) = self.notifications.iter_mut().find(|n| n.id == nid)
            {
                n.ripple = Some((pt, now, true));
            }
            self.pressed_drawer_icon = None;
            if matches!(ctx, PressContext::DrawerBody { in_header: false })
                && self.drawer_open && !self.sys.lock_layout
                && let Some((app, _, center)) = self.last_drawer_icon_rects.iter().find(|(_, rect, _)| rect.contains(pt))
            {
                self.pressed_drawer_icon = Some((app.clone(), *center));
            }
            self.press_ctx = ctx;
            self.gesture = immediate;
            match immediate {
                ActiveGesture::BrightnessSlider { bar } => {
                    self.sys.brightness = ((pt[0] - bar.x) / bar.width.max(1.0)).clamp(0.15, 1.0);
                    suppress_child_input = true;
                }
                ActiveGesture::VolumeSlider { channel, track } => {
                    self.apply_volume_slider(channel, track, pt[1], now);
                    suppress_child_input = true;
                }
                ActiveGesture::SplitDivider
                | ActiveGesture::FreeformMove { .. }
                | ActiveGesture::FreeformResize { .. } => {
                    suppress_child_input = true;
                }
                _ => {}
            }
        }

        // 2. Handle active pointer hold & movement (ACTION_MOVE, including final position on ACTION_UP!)
        if self.pointer_down && !input.cancelled && !input.blurred {
            if let Some(pt) = input.pointer {
                self.vel_tracker.push(now, pt);
                let dx = pt[0] - self.press_origin[0];
                let dy = pt[1] - self.press_origin[1];
                let dist = dx.hypot(dy);

                // A stationary hold on a radio tile opens the corresponding Settings page.
                if self.gesture == ActiveGesture::None && !self.long_press_fired
                    && !self.lockscreen.locked && !self.qs_edit_open && dist < 12.0
                    && matches!(self.press_ctx, PressContext::QsGrid | PressContext::ShadeBody { notif: None, .. })
                    && let Some(slot) = self.last_radio_tile_rects.iter().position(|rect| rect.is_some_and(|r| r.contains(self.press_origin)))
                {
                    if now - self.press_time >= 0.45 {
                        self.long_press_fired = true;
                        suppress_child_input = true;
                        let page = if slot == 0 { settings::SettingsPage::Internet } else { settings::SettingsPage::Connected };
                        self.settings_app.navigate(page, now);
                        self.open_app_from(AppId::Settings, None, false, true, false, now);
                    } else {
                        f.continuous(phone);
                    }
                }

                // Long-press check on Home screen before slop is exceeded
                if self.gesture == ActiveGesture::None {
                    match self.press_ctx {
                        PressContext::HomeBlank => {
                            if !self.long_press_fired && dist < 14.0 {
                                if now - self.press_time >= 0.36 {
                                    self.long_press_fired = true;
                                    let popup = HomePopup::BlankOptions(pt);
                                    self.home_popup = popup;
                                    self.last_home_popup = popup;
                                    self.popup_anim.snap(0.0);
                                    self.popup_anim.set(1.0, now, 0.18, 0.13);
                                    suppress_child_input = true;
                                } else {
                                    f.continuous(phone);
                                }
                            }
                        }
                        PressContext::QsEditTile { idx } => {
                            if dist < 14.0 {
                                if now - self.press_time >= 0.24 {
                                    self.long_press_fired = true;
                                    self.qs_edit_drag = Some((idx, pt));
                                    self.gesture = ActiveGesture::QsEditDrag { idx, pos: pt };
                                    suppress_child_input = true;
                                } else {
                                    f.continuous(phone);
                                }
                            } else {
                                // Fast drag before hold threshold -> scroll the QS edit list
                                let (_, max_s) = self.current_scroll_and_max(ScrollTarget::QsEdit);
                                self.gesture = ActiveGesture::ListScroll {
                                    target: ScrollTarget::QsEdit,
                                    last_y: self.press_origin[1],
                                    max_scroll: max_s,
                                };
                                suppress_child_input = true;
                            }
                        }
                        PressContext::DrawerBody { .. } if self.pressed_drawer_icon.is_some() => {
                            if !self.long_press_fired && dist < TOUCH_SLOP {
                                if now - self.press_time >= 0.36 {
                                    self.long_press_fired = true;
                                    suppress_child_input = true;
                                } else {
                                    f.continuous(phone);
                                }
                            } else if self.long_press_fired && dist >= TOUCH_SLOP
                                && !input.released
                                && let Some((_, center)) = self.pressed_drawer_icon.as_ref()
                            {
                                self.drawer_open = false;
                                self.drawer_anim.set(0.0, now, 0.28, 0.24);
                                self.ime_target = None;
                                self.ime_anim.set(0.0, now, 0.20, 0.18);
                                self.scroll_fling = None;
                                self.gesture = ActiveGesture::HomeIconDrag {
                                    ws_idx: None,
                                    pos: pt,
                                    offset: [self.press_origin[0] - center[0], self.press_origin[1] - center[1]],
                                };
                            }
                        }
                        PressContext::HomeIcon {
                            ws_idx,
                            anchor,
                            cell_center,
                        } => {
                            if !self.long_press_fired && dist < 14.0 {
                                if now - self.press_time >= 0.36 {
                                    self.long_press_fired = true;
                                    if let Some((_, _, HomeApp::Builtin(app))) = self.home_icons.get(ws_idx) {
                                        let popup = HomePopup::AppShortcuts { app: *app, anchor };
                                        self.home_popup = popup;
                                        self.last_home_popup = popup;
                                        self.popup_anim.snap(0.0);
                                        self.popup_anim.set(1.0, now, 0.18, 0.13);
                                    }
                                    suppress_child_input = true;
                                } else {
                                    f.continuous(phone);
                                }
                            } else if self.long_press_fired && !self.sys.lock_layout && dist >= 12.0
                            {
                                // Keep holding & drag -> lift Home icon to reposition on grid!
                                self.home_popup = HomePopup::None;
                                self.popup_anim.snap(0.0);
                                self.home_reorder_anim.set(1.0, now, 0.26, 0.20);
                                self.gesture = ActiveGesture::HomeIconDrag {
                                    ws_idx: Some(ws_idx),
                                    pos: pt,
                                    offset: [
                                        self.press_origin[0] - cell_center[0],
                                        self.press_origin[1] - cell_center[1],
                                    ],
                                };
                            }
                        }
                        PressContext::DockIcon { app, anchor } => {
                            if !self.long_press_fired && dist < 14.0 {
                                if now - self.press_time >= 0.36 {
                                    self.long_press_fired = true;
                                    let popup = HomePopup::AppShortcuts { app, anchor };
                                    self.home_popup = popup;
                                    self.last_home_popup = popup;
                                    self.popup_anim.snap(0.0);
                                    self.popup_anim.set(1.0, now, 0.18, 0.13);
                                    suppress_child_input = true;
                                } else {
                                    f.continuous(phone);
                                }
                            }
                        }
                        PressContext::ShadeBody {
                            notif: Some((nid, _)),
                            ..
                        } => {
                            if !self.long_press_fired && dist < 12.0 {
                                if now - self.press_time >= 0.42 {
                                    self.long_press_fired = true;
                                    if let Some(n) =
                                        self.notifications.iter_mut().find(|n| n.id == nid)
                                    {
                                        n.controls_open = !n.controls_open;
                                        n.controls_anim.set(
                                            f32::from(n.controls_open),
                                            now,
                                            0.22,
                                            0.18,
                                        );
                                        n.ripple = None;
                                    }
                                    suppress_child_input = true;
                                } else {
                                    f.continuous(phone);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Intercept drag once movement exceeds TOUCH_SLOP (Android onInterceptTouchEvent!)
                if self.gesture == ActiveGesture::None && !self.long_press_fired && dist >= TOUCH_SLOP
                {
                    let anchor_x = self.press_origin[0] + dx.signum() * TOUCH_SLOP.min(dx.abs());
                    let anchor_y = self.press_origin[1] + dy.signum() * TOUCH_SLOP.min(dy.abs());
                    match self.press_ctx {
                        PressContext::EdgeBack { right_edge } => {
                            let inward = if right_edge { -dx } else { dx };
                            if inward > 2.0 && dx.abs() >= dy.abs() * 0.85 {
                                self.gesture = ActiveGesture::EdgeBack {
                                    right_edge,
                                    anchor_x,
                                    y: pt[1],
                                    pull: 0.0,
                                    committed: false,
                                };
                            }
                        }
                        PressContext::StatusBar => {
                            if dy > 3.0 && dy.abs() >= dx.abs() * 0.70 {
                                self.gesture = ActiveGesture::ShadeScrub {
                                    start_stage: self.shade_stage,
                                    start_val: self.shade_anim.value,
                                    anchor_y,
                                    span: shade_span,
                                };
                            }
                        }
                        PressContext::ShadeBody { in_notifs, notif } => {
                            if dx.abs() > dy.abs() * 1.15
                                && let Some((notif_id, ongoing)) = notif
                            {
                                self.gesture = ActiveGesture::NotifSwipe {
                                    notif_id,
                                    ongoing,
                                    anchor_x,
                                    offset_x: 0.0,
                                };
                            } else if dy.abs() >= dx.abs() * 0.80 {
                                let (cur_s, max_s) =
                                    self.current_scroll_and_max(ScrollTarget::ShadeNotifs);
                                let scroll_instead = in_notifs
                                    && self.shade_anim.value <= 1.12
                                    && max_s > 1.0
                                    && ((dy < 0.0 && cur_s < max_s - 1.0)
                                        || (dy > 0.0 && cur_s > 1.0));
                                if scroll_instead {
                                    self.gesture = ActiveGesture::ListScroll {
                                        target: ScrollTarget::ShadeNotifs,
                                        last_y: anchor_y,
                                        max_scroll: max_s,
                                    };
                                } else {
                                    self.gesture = ActiveGesture::ShadeScrub {
                                        start_stage: self.shade_stage,
                                        start_val: self.shade_anim.value,
                                        anchor_y,
                                        span: shade_span,
                                    };
                                }
                            }
                        }
                        PressContext::QsGrid => {
                            if dx.abs() > dy.abs() * 1.10 {
                                self.gesture = ActiveGesture::QsPageSwipe {
                                    start_page: self.qs_page_anim.value,
                                    anchor_x,
                                    width: phone.width.max(240.0),
                                };
                            } else if dy.abs() >= dx.abs() * 0.80 {
                                self.gesture = ActiveGesture::ShadeScrub {
                                    start_stage: self.shade_stage,
                                    start_val: self.shade_anim.value,
                                    anchor_y,
                                    span: shade_span,
                                };
                            }
                        }
                        PressContext::HomeBlank
                        | PressContext::HomeIcon { .. }
                        | PressContext::DockIcon { .. } => {
                            if dy < 0.0 && dy.abs() > dx.abs() * 1.05 {
                                self.gesture = ActiveGesture::DrawerScrub {
                                    start_val: self.drawer_anim.value,
                                    anchor_y,
                                    span: drawer_span,
                                };
                            } else if dy > 0.0 && dy.abs() > dx.abs() * 1.05 {
                                self.gesture = ActiveGesture::ShadeScrub {
                                    start_stage: self.shade_stage,
                                    start_val: self.shade_anim.value,
                                    anchor_y,
                                    span: shade_span,
                                };
                            } else if dx.abs() > dy.abs() * 1.05 {
                                self.gesture = ActiveGesture::HomePageSwipe {
                                    start_page: self.home_page_anim.value,
                                    anchor_x,
                                    width: phone.width.max(240.0),
                                };
                            }
                        }
                        PressContext::DrawerBody { in_header } => {
                            if dy.abs() >= dx.abs() * 0.85 {
                                let (cur_s, max_s) =
                                    self.current_scroll_and_max(ScrollTarget::Drawer);
                                if dy > 0.0 && (in_header || cur_s <= 1.0) {
                                    self.gesture = ActiveGesture::DrawerScrub {
                                        start_val: self.drawer_anim.value,
                                        anchor_y,
                                        span: drawer_span,
                                    };
                                } else if max_s > 1.0 || dy < 0.0 {
                                    self.gesture = ActiveGesture::ListScroll {
                                        target: ScrollTarget::Drawer,
                                        last_y: anchor_y,
                                        max_scroll: max_s,
                                    };
                                } else if dy > 0.0 {
                                    self.gesture = ActiveGesture::DrawerScrub {
                                        start_val: self.drawer_anim.value,
                                        anchor_y,
                                        span: drawer_span,
                                    };
                                }
                            }
                        }
                        PressContext::WidgetsBody { in_header } => {
                            if dy.abs() >= dx.abs() * 0.85 {
                                let (cur_s, max_s) =
                                    self.current_scroll_and_max(ScrollTarget::Widgets);
                                if dy > 0.0 && (in_header || cur_s <= 1.0) {
                                    self.gesture = ActiveGesture::WidgetsScrub {
                                        start_val: self.widgets_anim.value,
                                        anchor_y,
                                        span: widgets_span,
                                    };
                                } else if max_s > 1.0 || dy < 0.0 {
                                    self.gesture = ActiveGesture::ListScroll {
                                        target: ScrollTarget::Widgets,
                                        last_y: anchor_y,
                                        max_scroll: max_s,
                                    };
                                } else if dy > 0.0 {
                                    self.gesture = ActiveGesture::WidgetsScrub {
                                        start_val: self.widgets_anim.value,
                                        anchor_y,
                                        span: widgets_span,
                                    };
                                }
                            }
                        }
                        PressContext::NavBar { from_app } => {
                            self.gesture = ActiveGesture::NavGesture {
                                origin: self.press_origin,
                                delta: [dx, dy],
                                from_app,
                                hold_time: 0.0,
                            };
                        }
                        PressContext::OverviewCard {
                            task_idx,
                            step,
                            max_scroll,
                        } => {
                            if dy.abs() > dx.abs() * 1.25
                                && matches!(self.mode, ShellMode::Overview)
                                && self.overview_vertical_settle.is_none()
                                && let Some(idx) = task_idx
                                && let Some((_, original_card)) =
                                    self.last_overview_cards.iter().find(|(i, _)| *i == idx)
                            {
                                self.overview_glide = None;
                                self.gesture = ActiveGesture::OverviewVertical {
                                    task_idx: idx,
                                    anchor_y: self.press_origin[1],
                                    offset_y: dy,
                                    original_card: *original_card,
                                };
                            } else if dx.abs() >= dy.abs() * 0.70 {
                                self.overview_glide = None;
                                self.gesture = ActiveGesture::OverviewHorizontal {
                                    start_scroll: self.overview_scroll,
                                    anchor_x,
                                    step,
                                    max_scroll,
                                };
                            }
                        }
                        PressContext::ScrollList { target } => {
                            let (_cur_s, max_s) = self.current_scroll_and_max(target);
                            // ALWAYS engage ListScroll on vertical drags past TOUCH_SLOP (even if max_s == 0!)
                            // so top/bottom EdgeEffect stretch works everywhere AND held ripples cancel cleanly!
                            if dy.abs() >= dx.abs() * 1.05 {
                                self.gesture = ActiveGesture::ListScroll {
                                    target,
                                    last_y: anchor_y,
                                    max_scroll: max_s,
                                };
                            }
                        }
                        PressContext::None | PressContext::QsEditTile { .. } => {}
                    }
                }

                // Update active gesture 1:1 with finger/cursor!
                if self.gesture != ActiveGesture::None {
                    suppress_child_input = true;
                    f.continuous(phone);
                    let mut handoff: Option<ActiveGesture> = None;
                    match &mut self.gesture {
                        ActiveGesture::EdgeBack {
                            right_edge,
                            anchor_x,
                            y,
                            pull,
                            committed,
                        } => {
                            *y = pt[1].clamp(phone.y + status_h + 28.0, phone.y + phone.height - nav_h - 28.0);
                            let raw = if *right_edge {
                                *anchor_x - pt[0]
                            } else {
                                pt[0] - *anchor_x
                            };
                            *pull = raw.max(0.0);
                            *committed = *pull >= 44.0;
                        }
                        ActiveGesture::ShadeScrub {
                            start_stage,
                            start_val,
                            anchor_y,
                            span,
                        } => {
                            let dy = pt[1] - *anchor_y;
                            let sheet_travel = (phone.y + phone.height
                                - Self::compact_notification_sheet_top(phone, status_h)).max(1.0);
                            // Convert progress to distance first so the sheet moves 1:1 with touch.
                            let start = *start_val - 1.0;
                            let offset = start * if start > 0.0 { sheet_travel } else { *span } + dy;
                            let travel = if offset > 0.0 { sheet_travel } else { *span };
                            let min_v = start_stage.saturating_sub(1) as f32;
                            let max_v = (*start_stage + 1).min(2) as f32;
                            let val = (1.0 + offset / travel).clamp(min_v, max_v);
                            self.shade_anim.snap(val);
                        }
                        ActiveGesture::DrawerScrub {
                            start_val,
                            anchor_y,
                            span,
                        } => {
                            let val = (*start_val - (pt[1] - *anchor_y) / *span).clamp(0.0, 1.0);
                            self.drawer_anim.snap(val);
                        }
                        ActiveGesture::WidgetsScrub {
                            start_val,
                            anchor_y,
                            span,
                        } => {
                            let val = (*start_val - (pt[1] - *anchor_y) / *span).clamp(0.0, 1.0);
                            self.widgets_anim.snap(val);
                        }
                        ActiveGesture::NavGesture {
                            origin,
                            delta,
                            from_app,
                            hold_time,
                        } => {
                            *delta = [pt[0] - origin[0], pt[1] - origin[1]];
                            let vel = self.vel_tracker.velocity(now);
                            if *from_app && delta[1] < -64.0 && vel[1].abs() < 85.0 {
                                *hold_time += dt as f64;
                            } else if vel[1].abs() > 140.0 {
                                *hold_time = (*hold_time - dt as f64 * 2.0).max(0.0);
                            }
                            // The bottom pill previews empty Recents without pulling up the drawer.
                            if !*from_app
                                && self.mode == ShellMode::Home
                                && !self.drawer_open
                            {
                                let upward = -delta[1];
                                let vertical = upward > delta[0].abs() * 1.1;
                                if self.tasks.is_empty() {
                                    let preview = if vertical { 0.86 * smoothstep(65.0, 270.0, upward) } else { 0.0 };
                                    self.overview_anim.snap(preview);
                                } else if upward > 12.0 && vertical {
                                    self.drawer_anim.snap((upward / drawer_span).clamp(0.0, 1.0));
                                }
                            }
                        }
                        ActiveGesture::OverviewHorizontal {
                            start_scroll,
                            anchor_x,
                            max_scroll,
                            ..
                        } => {
                            // TRUE ANDROID DIRECTION:
                            // Dragging RIGHT (pt[0] > anchor_x) INCREASES overview_scroll,
                            // moving older cards from the LEFT smoothly rightward into center!
                            let raw = *start_scroll + (pt[0] - *anchor_x);
                            if raw < 0.0 {
                                self.overview_scroll = 0.0;
                                let _ = self.overview_edge_spring.pull(raw - self.overview_edge_spring.distance() * phone.width, phone.width);
                            } else if raw > *max_scroll {
                                self.overview_scroll = *max_scroll;
                                let excess = raw - *max_scroll;
                                let _ = self.overview_edge_spring.pull(excess - self.overview_edge_spring.distance() * phone.width, phone.width);
                            } else {
                                self.overview_edge_spring.reset();
                                self.overview_scroll = raw;
                            }
                        }
                        ActiveGesture::OverviewVertical {
                            anchor_y,
                            offset_y,
                            ..
                        } => {
                            *offset_y = pt[1] - *anchor_y;
                        }
                        ActiveGesture::QsPageSwipe {
                            start_page,
                            anchor_x,
                            width,
                        } => {
                            let raw = *start_page - (pt[0] - *anchor_x) / *width;
                            self.qs_page_anim.snap(raw.clamp(-0.14, self.qs_tiles.len().div_ceil(8).saturating_sub(1) as f32 + 0.14));
                        }
                        ActiveGesture::NotifSwipe {
                            ongoing,
                            anchor_x,
                            offset_x,
                            ..
                        } => {
                            let raw = pt[0] - *anchor_x;
                            *offset_x = if *ongoing {
                                (raw * 0.22).clamp(-38.0, 38.0)
                            } else {
                                // Android magnetic notification swipe detent:
                                // tethered resistance before detach threshold, then pops free 1:1
                                let detach = phone.width * 0.26;
                                let abs_r = raw.abs();
                                let sign = raw.signum();
                                if abs_r < detach {
                                    sign * abs_r * 0.52
                                } else {
                                    let t = ((abs_r - detach) / 28.0).clamp(0.0, 1.0);
                                    sign * lerp(detach * 0.52, abs_r, t)
                                }
                            };
                        }
                        ActiveGesture::QsEditDrag { idx, pos } => {
                            *pos = pt;
                            if let Some((cur_idx, _)) = self.qs_edit_drag {
                                *idx = cur_idx;
                            }
                            self.qs_edit_drag = Some((*idx, pt));
                        }
                        ActiveGesture::HomeIconDrag { ws_idx, pos, .. } => {
                            *pos = pt;
                            if self.last_ws_rect.width > 0.0 {
                                let max_occupied = self
                                    .home_icon_pages
                                    .iter()
                                    .enumerate()
                                    .filter(|&(i, _)| Some(i) != *ws_idx)
                                    .map(|(_, &p)| p)
                                    .max()
                                    .unwrap_or(0);
                                if pt[0] >= self.last_ws_rect.x + self.last_ws_rect.width - 24.0
                                    && (self.home_page_anim.value - self.home_page as f32).abs() < 0.05
                                    && self.home_page <= max_occupied
                                {
                                    let np = self.home_page + 1;
                                    if np >= self.home_page_count {
                                        self.home_page_count = np + 1;
                                    }
                                    self.home_page = np;
                                    self.home_page_anim.set(np as f32, now, 0.28, 0.24);
                                } else if pt[0] <= self.last_ws_rect.x + 24.0
                                    && self.home_page > 0
                                    && (self.home_page_anim.value - self.home_page as f32).abs() < 0.05
                                {
                                    let pp = self.home_page - 1;
                                    self.home_page = pp;
                                    self.home_page_anim.set(pp as f32, now, 0.28, 0.24);
                                    while self.home_page_count > 1
                                        && self.home_page < self.home_page_count - 1
                                        && !self.home_icon_pages.iter().any(|&p| p == self.home_page_count - 1)
                                    {
                                        self.home_page_count -= 1;
                                    }
                                }
                            }
                        }
                        ActiveGesture::HomePageSwipe {
                            start_page,
                            anchor_x,
                            width,
                        } => {
                            let max_p = self.home_page_count.saturating_sub(1) as f32;
                            let raw = *start_page - (pt[0] - *anchor_x) / *width;
                            if raw < 0.0 {
                                self.home_page_anim.snap(0.0);
                                let excess_px = -raw * *width;
                                let _ = self.home_stretch.pull(
                                    excess_px - self.home_stretch.distance() * *width,
                                    *width,
                                );
                            } else if raw > max_p {
                                self.home_page_anim.snap(max_p);
                                let excess_px = -(raw - max_p) * *width;
                                let _ = self.home_stretch.pull(
                                    excess_px - self.home_stretch.distance() * *width,
                                    *width,
                                );
                            } else {
                                self.home_stretch.reset();
                                self.home_page_anim.snap(raw);
                            }
                        }
                        ActiveGesture::ListScroll {
                            target,
                            last_y,
                            max_scroll,
                        } => {
                            let target = *target;
                            let max_s = *max_scroll;
                            let step_dy = pt[1] - *last_y; // + = pulling DOWN, - = pulling UP
                            *last_y = pt[1];
                            let vp_h = (phone.height - status_h - nav_h).max(200.0);
                            let t_idx = target as usize;

                            // 1. If currently stretched, let EdgeStretch consume reverse pull BEFORE scrolling!
                            let mut rem_dy = step_dy;
                            if self.scroll_stretches[t_idx].distance().abs() > 1e-4 {
                                let consumed = self.scroll_stretches[t_idx].pull(rem_dy, vp_h);
                                rem_dy -= consumed;
                            }

                            // 2. Scroll content normally; any leftover past 0 or max_s either hands off
                            //    nested-scroll (Drawer/Shade/Widgets top) OR feeds Android 13 EdgeStretch!
                            if rem_dy.abs() > 1e-4 {
                                let cur_val = *self.scroll_mut_for(target).0;
                                let desired = cur_val - rem_dy;
                                if desired < 0.0 {
                                    let excess_down = -desired; // positive (pulling down past top)
                                    if matches!(target, ScrollTarget::ShadeNotifs) && excess_down > 3.0 {
                                        self.shade_scroll = 0.0;
                                        handoff = Some(ActiveGesture::ShadeScrub {
                                            start_stage: self.shade_stage,
                                            start_val: self.shade_anim.value,
                                            anchor_y: pt[1],
                                            span: shade_span,
                                        });
                                    } else if matches!(target, ScrollTarget::Drawer) && excess_down > 3.0 {
                                        self.drawer_scroll = 0.0;
                                        handoff = Some(ActiveGesture::DrawerScrub {
                                            start_val: self.drawer_anim.value,
                                            anchor_y: pt[1],
                                            span: drawer_span,
                                        });
                                    } else if matches!(target, ScrollTarget::Widgets) && excess_down > 3.0 {
                                        self.widgets_scroll = 0.0;
                                        handoff = Some(ActiveGesture::WidgetsScrub {
                                            start_val: self.widgets_anim.value,
                                            anchor_y: pt[1],
                                            span: widgets_span,
                                        });
                                    } else {
                                        *self.scroll_mut_for(target).0 = 0.0;
                                        let _ = self.scroll_stretches[t_idx].pull(excess_down, vp_h);
                                    }
                                } else if desired > max_s {
                                    let excess_up = max_s - desired; // negative (pulling up past bottom)
                                    *self.scroll_mut_for(target).0 = max_s;
                                    let _ = self.scroll_stretches[t_idx].pull(excess_up, vp_h);
                                } else {
                                    *self.scroll_mut_for(target).0 = desired;
                                }
                            }
                        }
                        ActiveGesture::SplitDivider => {
                            let app_y = phone.y + status_h;
                            let app_h = (phone.height - status_h - nav_h).max(120.0);
                            self.split_ratio = ((pt[1] - app_y) / app_h).clamp(0.15, 0.85);
                        }
                        ActiveGesture::FreeformMove { offset } => {
                            if let Some(win) = &mut self.freeform {
                                win.x = (pt[0] - offset[0])
                                    .clamp(phone.x + 4.0, phone.x + phone.width - win.w - 4.0);
                                win.y = (pt[1] - offset[1]).clamp(
                                    phone.y + status_h + 4.0,
                                    phone.y + phone.height - nav_h - 80.0,
                                );
                            }
                        }
                        ActiveGesture::FreeformResize {
                            origin_ptr,
                            origin_size,
                        } => {
                            if let Some(win) = &mut self.freeform {
                                win.w = (origin_size[0] + pt[0] - origin_ptr[0])
                                    .clamp(210.0, phone.width - 16.0);
                                win.h = (origin_size[1] + pt[1] - origin_ptr[1])
                                    .clamp(240.0, phone.height - status_h - nav_h - 24.0);
                            }
                        }
                        ActiveGesture::BrightnessSlider { bar } => {
                            let bar = *bar;
                            self.sys.brightness =
                                ((pt[0] - bar.x) / bar.width.max(1.0)).clamp(0.15, 1.0);
                        }
                        ActiveGesture::VolumeSlider { channel, track } => {
                            let (ch, tr) = (*channel, *track);
                            self.apply_volume_slider(ch, tr, pt[1], now);
                        }
                        ActiveGesture::None => {}
                    }
                    if let Some(next_g) = handoff {
                        self.gesture = next_g;
                    }
                }
            }
        }

        // 3. Handle release / cancel (ACTION_UP / ACTION_CANCEL)
        if self.pointer_down && (input.released || input.cancelled || input.blurred) {
            let [vx, vy] = self.vel_tracker.velocity(now);
            let active = self.gesture;
            let was_long_press = self.long_press_fired;
            let pressed_drawer_icon = self.pressed_drawer_icon.take();
            self.pointer_down = false;
            self.press_ctx = PressContext::None;
            self.gesture = ActiveGesture::None;
            self.long_press_fired = false;
            self.vel_tracker.clear();
            // Release any held notification ripple so it smoothly fades out!
            for n in &mut self.notifications {
                if let Some((_, _, ref mut held)) = n.ripple {
                    *held = false;
                }
            }

            if was_long_press || active != ActiveGesture::None {
                suppress_child_input = true;
            }

            match active {
                ActiveGesture::EdgeBack {
                    right_edge,
                    y,
                    pull,
                    committed,
                    ..
                } => {
                    let inward_vel = if right_edge { -vx } else { vx };
                    if committed || (pull >= 22.0 && inward_vel > 320.0) {
                        let pred = if self.can_exit_app_to_home() {
                            let prog = (pull / 120.0).clamp(0.0, 1.0);
                            let sc = lerp(1.0, 0.90, prog);
                            let avail_x = ((phone.width - 0.90 * phone.width) * 0.5 - 8.0).max(0.0);
                            let tx = lerp(0.0, avail_x, prog) * if right_edge { -1.0 } else { 1.0 };
                            let ty = ((y - self.press_origin[1]) * 0.16).clamp(-22.0, 22.0);
                            let w = phone.width * sc;
                            let h = phone.height * sc;
                            Some((
                                Rect {
                                    x: phone.x + (phone.width - w) * 0.5 + tx,
                                    y: phone.y + (phone.height - h) * 0.5 + ty,
                                    width: w,
                                    height: h,
                                },
                                lerp(12.0, 26.0, prog),
                            ))
                        } else {
                            None
                        };
                        self.go_back_with_rect(pred, now);
                    }
                }
                ActiveGesture::ShadeScrub { start_stage, .. } => {
                    let v = self.shade_anim.value;
                    // A fling can finish this step, including a fast upward fling from QS.
                    let min_stage = start_stage.saturating_sub(1);
                    let max_stage = (start_stage + 1).min(2);
                    let open_velocity = if start_stage == 0 { 320.0 } else { 360.0 };
                    let expanded_threshold = if start_stage == 2 { 1.75 } else { 1.30 };
                    let stage = if vy < -360.0 {
                        min_stage
                    } else if vy > open_velocity {
                        max_stage
                    } else if v >= expanded_threshold {
                        2
                    } else if v >= 0.42 {
                        1
                    } else {
                        0
                    };
                    self.settle_shade(stage.clamp(min_stage, max_stage), now);
                }
                ActiveGesture::DrawerScrub { .. } => {
                    let v = self.drawer_anim.value;
                    let open = if vy < -340.0 {
                        true
                    } else if vy > 340.0 {
                        false
                    } else {
                        v >= 0.34
                    };
                    self.drawer_open = open;
                    self.drawer_anim
                        .set(if open { 1.0 } else { 0.0 }, now, 0.28, 0.22);
                }
                ActiveGesture::WidgetsScrub { .. } => {
                    let v = self.widgets_anim.value;
                    let open = if vy < -340.0 {
                        true
                    } else if vy > 340.0 {
                        false
                    } else {
                        v >= 0.38
                    };
                    self.widgets_open = open;
                    self.widgets_anim
                        .set(if open { 1.0 } else { 0.0 }, now, 0.26, 0.20);
                }
                ActiveGesture::NavGesture {
                    delta,
                    from_app,
                    hold_time,
                    ..
                } => {
                    if from_app {
                        let is_horiz_switch = delta[0].abs() > 46.0
                            && delta[0].abs() > delta[1].abs() * 1.35
                            && delta[1] > -50.0;
                        if is_horiz_switch && self.tasks.len() > 1 {
                            let target_entry = if delta[0] > 0.0 {
                                self.tasks[1]
                            } else {
                                *self.tasks.last().unwrap_or(&self.tasks[0])
                            };
                            match target_entry {
                                TaskEntry::Single(a) => self.open_app(a, now),
                                TaskEntry::SplitPair(a, b) => self.open_split(a, b, now),
                                TaskEntry::Desktop(idx) => self.open_desktop_app(idx, now),
                            }
                        } else if delta[1] < -68.0 && (hold_time >= 0.14 || vy > -260.0) {
                            let up = (-delta[1]).max(0.0);
                            let drag_p = (up / 300.0).clamp(0.0, 1.0);
                            let final_s = lerp(1.0, 0.67, drag_p);
                            let w = phone.width * final_s;
                            let h = phone.height * final_s;
                            let tx = delta[0] * 0.44;
                            let ty = lerp(0.0, -44.0, drag_p);
                            let cur = Rect {
                                x: phone.x + (phone.width - w) * 0.5 + tx,
                                y: phone.y + (phone.height - h) * 0.5 + ty,
                                width: w,
                                height: h,
                            };
                            let source = if self.mode == ShellMode::Fullscreen(AppId::Settings) {
                                self.settings_nav_preview(phone, delta, hold_time)
                            } else {
                                (cur, 26.0)
                            };
                            self.open_overview_from(Some(source), now);
                        } else if vy < -420.0 || delta[1] < -42.0 {
                            let up = (-delta[1]).max(0.0);
                            let drag_p = (up / 300.0).clamp(0.0, 1.0);
                            let final_s = lerp(1.0, 0.67, drag_p);
                            let w = phone.width * final_s;
                            let h = phone.height * final_s;
                            let tx = delta[0] * 0.44;
                            let ty = lerp(0.0, -44.0, drag_p);
                            let cur = Rect {
                                x: phone.x + (phone.width - w) * 0.5 + tx,
                                y: phone.y + (phone.height - h) * 0.5 + ty,
                                width: w,
                                height: h,
                            };
                            let source = if self.mode == ShellMode::Fullscreen(AppId::Settings) {
                                self.settings_nav_preview(phone, delta, hold_time)
                            } else {
                                (cur, 26.0)
                            };
                            self.go_home_with_rect(Some(source), now);
                        }
                    } else if self.mode == ShellMode::Overview {
                        if delta[1] < -36.0 || vy < -360.0 {
                            self.go_home(now);
                        }
                    } else if self.mode == ShellMode::Home {
                        if self.drawer_open {
                            if delta[1] < -32.0 || vy < -340.0 {
                                self.drawer_open = false;
                                self.drawer_anim.set(0.0, now, 0.26, 0.20);
                            }
                        } else if self.tasks.is_empty() {
                            if !input.cancelled && !input.blurred
                                && delta[1] < -110.0 && -delta[1] > delta[0].abs() * 1.1
                                && vy.abs() < 160.0
                            {
                                self.open_overview(now);
                            } else {
                                self.overview_anim.set(0.0, now, 0.24, 0.20);
                            }
                        } else if delta[1] < -110.0 && vy.abs() < 160.0 {
                            self.drawer_open = false;
                            self.drawer_anim.set(0.0, now, 0.18, 0.16);
                            self.open_overview(now);
                        } else if self.drawer_anim.value > 0.01 {
                            let open = vy < -340.0 || self.drawer_anim.value >= 0.34;
                            self.drawer_open = open;
                            self.drawer_anim
                                .set(if open { 1.0 } else { 0.0 }, now, 0.28, 0.22);
                        }
                    }
                }
                ActiveGesture::OverviewHorizontal {
                    step, max_scroll, ..
                } => {
                    self.overview_edge_spring.release();
                    // Real Android 13 RecentsView variable-speed gradual deceleration:
                    // - Slow short swipe (<~50% card with low velocity) settles smoothly BACK to current card!
                    // - Normal swipe glides smoothly 1 card!
                    // - Fast flick carries full momentum (NO ±1 card cap!), coasting across 2–4 cards with gradual deceleration!
                    let decel_rate = 5.4_f32;
                    let raw_proj = self.overview_scroll + vx / decel_rate;
                    if self.overview_scroll <= 0.01 && vx < -350.0 {
                        self.overview_edge_spring.absorb(vx * 0.6, phone.width);
                    } else if self.overview_scroll >= max_scroll - 0.01 && vx > 350.0 {
                        self.overview_edge_spring.absorb(vx * 0.6, phone.width);
                    }
                    let projected = raw_proj.clamp(0.0, max_scroll);
                    let target = if max_scroll > 0.0 && projected >= max_scroll - 58.0 {
                        max_scroll
                    } else {
                        ((projected / step).round() * step).clamp(0.0, max_scroll)
                    };
                    self.overview_glide = Some(OverviewGlide::start(
                        self.overview_scroll,
                        target,
                        vx,
                        now,
                    ));
                }
                ActiveGesture::OverviewVertical { task_idx, offset_y, original_card, .. } => {
                    if let Some(&entry) = self.tasks.get(task_idx) {
                        let released = !input.cancelled && !input.blurred;
                        if released && (offset_y > 90.0 || (offset_y > 24.0 && vy > 480.0)) {
                            let progress = Self::overview_drag_progress(offset_y, phone);
                            let card = lerp_rect(original_card, phone, progress, progress);
                            self.reopen_overview_task(entry, card, progress, now);
                        } else {
                            let dismiss = released && (offset_y < -105.0 || (offset_y < -24.0 && vy < -480.0));
                            let mut slide = Anim::at(offset_y);
                            slide.set(if dismiss { -phone.height } else { 0.0 }, now, 0.22, 0.24);
                            self.overview_vertical_settle = Some((entry, slide, dismiss));
                        }
                    }
                }
                ActiveGesture::QsPageSwipe { start_page, .. } => {
                    let p = self.qs_page_anim.value;
                    let max_page = self.qs_tiles.len().div_ceil(8).saturating_sub(1);
                    let start = start_page.round().max(0.0) as usize;
                    let next_page = if vx < -320.0 {
                        (start + 1).min(max_page)
                    } else if vx > 320.0 {
                        start.saturating_sub(1)
                    } else {
                        (p + 0.55).floor().max(0.0) as usize
                    }.min(max_page);
                    self.qs_page = next_page;
                    self.qs_page_anim.set(next_page as f32, now, 0.24, 0.24);
                }
                ActiveGesture::NotifSwipe {
                    notif_id,
                    ongoing,
                    offset_x,
                    ..
                } => {
                    let dismiss = !ongoing
                        && (offset_x.abs() > phone.width * 0.30 || vx.abs() > 460.0);
                    let mut dismissed_dbus = false;
                    if let Some(n) = self.notifications.iter_mut().find(|n| n.id == notif_id) {
                        n.swipe_anim.snap(offset_x);
                        if dismiss {
                            let dir = if offset_x.abs() > 4.0 {
                                offset_x.signum()
                            } else if vx.abs() > 1.0 {
                                vx.signum()
                            } else {
                                1.0
                            };
                            n.dismissing = true;
                            dismissed_dbus = n.from_dbus;
                            let rem = (phone.width + 56.0 - offset_x.abs()).max(40.0);
                            let fling_dur = (rem / vx.abs().max(1100.0)).clamp(0.12, 0.24) as f64;
                            n.swipe_anim
                                .set(dir * (phone.width + 56.0), now, fling_dur, fling_dur);
                            n.dismiss_anim.snap(1.0);
                            n.dismiss_anim.set(0.0, now + 0.04, 0.26, 0.26);
                        } else {
                            n.swipe_anim.set(0.0, now, 0.26, 0.24);
                        }
                    }
                    if dismiss {
                        self.mark_history_dismissed(notif_id);
                        if dismissed_dbus {
                            self.host_commands.push(HostCommand::NotifyClosed {
                                id: notif_id,
                                reason: notifications::CLOSE_REASON_DISMISSED,
                            });
                        }
                        self.show_toast("Notification dismissed", now);
                    }
                }
                ActiveGesture::QsEditDrag { .. } => {
                    if input.released && !input.cancelled && !input.blurred
                        && let Some((index, point)) = self.qs_edit_drag
                    {
                        self.drop_qs_editor_tile(phone, status_h, index, point, now);
                    }
                    self.qs_edit_drag = None;
                }
                ActiveGesture::HomeIconDrag { ws_idx, pos, .. } => {
                    self.home_reorder_anim.set(0.0, now, 0.26, 0.20);
                    if input.released && !input.cancelled && !input.blurred {
                        if let Some(index) = ws_idx {
                            if phone.contains(pos) && pos[1] <= phone.y + 84.0 {
                                let (_, _, app) = self.home_icons.remove(index);
                                self.home_icon_pages.remove(index);
                                let name = match &app {
                                    HomeApp::Builtin(app) => app.name(),
                                    HomeApp::Desktop(id) => self.desktop_shortcut_index(id)
                                        .map_or(id.as_str(), |idx| self.desktop_registry.apps[idx].name.as_str()),
                                };
                                if matches!(app, HomeApp::Builtin(_)) && pos[0] >= phone.x + phone.width * 0.5 {
                                    self.show_toast(format!("Uninstalled {name}"), now);
                                } else {
                                    self.show_toast(format!("Removed {name} from Home"), now);
                                }
                                self.save_home_layout(now);
                            } else if let Some((col, row)) = self.home_drop_slot(pos, Some(index), false) {
                                let page = self.home_page;
                                if let Some(other) = self.home_icons.iter().enumerate().position(|(idx, &(c, r, _))| {
                                    idx != index && self.home_icon_pages[idx] == page && (c, r) == (col, row)
                                }) {
                                    self.home_icons[other].0 = self.home_icons[index].0;
                                    self.home_icons[other].1 = self.home_icons[index].1;
                                    self.home_icon_pages[other] = self.home_icon_pages[index];
                                }
                                self.home_icons[index].0 = col;
                                self.home_icons[index].1 = row;
                                self.home_icon_pages[index] = page;
                                self.save_home_layout(now);
                            }
                        } else if let Some((app, _)) = pressed_drawer_icon {
                            let existing = self.home_shortcut_index(&app);
                            if let Some((col, row)) = self.home_drop_slot(pos, existing, true) {
                                if let Some(index) = existing {
                                    self.home_icons[index] = (col, row, app);
                                    self.home_icon_pages[index] = self.home_page;
                                } else {
                                    self.home_icons.push((col, row, app));
                                    self.home_icon_pages.push(self.home_page);
                                }
                                self.save_home_layout(now);
                            } else if self.last_ws_rect.contains(pos) {
                                self.show_toast("No room on this Home screen", now);
                            }
                        }
                    }
                    self.trim_home_pages(now);
                }
                ActiveGesture::HomePageSwipe { .. } => {
                    self.home_stretch.release();
                    let max_p = self.home_page_count.saturating_sub(1);
                    let cur = self.home_page_anim.value;
                    let next = if vx < -320.0 {
                        (self.home_page + 1).min(max_p)
                    } else if vx > 320.0 {
                        self.home_page.saturating_sub(1)
                    } else {
                        (cur.round() as isize).clamp(0, max_p as isize) as usize
                    };
                    self.home_page = next;
                    self.home_page_anim.set(next as f32, now, 0.28, 0.24);
                }
                ActiveGesture::ListScroll { target, .. } => {
                    let t_idx = target as usize;
                    self.scroll_stretches[t_idx].release();
                    if self.scroll_stretches[t_idx].distance().abs() < 0.01 && vy.abs() > 80.0 {
                        // Finger moving UP (vy < 0) increases scroll offset!
                        self.scroll_fling = Some((target, -vy));
                    }
                }
                ActiveGesture::SplitDivider => {
                    if let ShellMode::Split { top, bottom } = self.mode {
                        if self.split_ratio <= 0.22 {
                            self.open_app(bottom, now);
                        } else if self.split_ratio >= 0.78 {
                            self.open_app(top, now);
                        } else {
                            self.split_ratio = self.split_ratio.clamp(0.25, 0.75);
                        }
                    }
                }
                ActiveGesture::FreeformMove { .. }
                | ActiveGesture::FreeformResize { .. }
                | ActiveGesture::BrightnessSlider { .. }
                | ActiveGesture::VolumeSlider { .. }
                | ActiveGesture::None => {}
            }
        }

        // Mouse wheel scrolling on Overview carousel (horizontal or vertical wheel)
        if matches!(self.mode, ShellMode::Overview | ShellMode::SplitSelect(_))
            && self.task_menu_idx.is_none()
            && self.shade_anim.value < 0.05
            && !self.power_open
            && input.pointer.is_some_and(|p| self.last_overview_rect.contains(p))
            && (input.scroll[0].abs() > 0.01 || input.scroll[1].abs() > 0.01)
        {
            let delta = if input.scroll[0].abs() > 0.01 {
                -input.scroll[0]
            } else {
                input.scroll[1]
            };
            self.overview_glide = None;
            self.overview_scroll = (self.overview_scroll + delta).clamp(0.0, self.last_overview_max);
        }

        suppress_child_input
    }

    pub fn show(
        &mut self,
        f: &mut Frame<'_>,
        width: f32,
        height: f32,
        input: Input,
        gmail_text: &mut String,
    ) {
        self.clock = self.time_service.now();
        f.wake_at(f.time() + self.clock.until_minute());
        let width = width.max(1.0);
        let height = height.max(1.0);
        let now = f.time();
        let dt = if self.last_frame_time > 0.0 {
            (now - self.last_frame_time).clamp(0.0, 0.05) as f32
        } else {
            1.0 / 60.0
        };
        self.last_frame_time = now;
        let reduced = f.reduced_motion();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        };

        if let Some(pending) = self.gmail_body_pending.take() {
            *gmail_text = pending;
        }
        self.sys.alarm_summary = self.builtin_apps.alarm_summary();

        // Sync demo USB debugging notification with Settings toggle (only in non-compositor preview mode)
        if !self.native_compositor {
            let has_usb = self.notifications.iter().any(|n| n.id == 4);
            if self.sys.usb_debugging && !has_usb {
                self.notifications.push(NotificationItem {
                    id: 4,
                    app: "Android System".into(),
                    title: "USB debugging connected".into(),
                    body: "Tap to turn off USB debugging".into(),
                    big_text: "USB debugging is active. Tap to open Developer options in Settings."
                        .into(),
                    time_label: "now".into(),
                    silent: false,
                    ongoing: true,
                    urgency: 1,
                    category: Some("device".into()),
                    desktop_entry: None,
                    resident: true,
                    transient: false,
                    actions: Vec::new(),
                    icon_image: None,
                    expires_at: None,
                    arrived_at: now,
                    heads_up_until: None,
                    from_dbus: false,
                    expanded: false,
                    expand_anim: Anim::at(0.0),
                    controls_open: false,
                    controls_anim: Anim::at(0.0),
                    swipe_anim: Anim::at(0.0),
                    dismiss_anim: Anim::at(1.0),
                    dismissing: false,
                    ripple: None,
                    snoozed: false,
                    has_reply: false,
                    target_app: Some(AppId::Settings),
                });
            } else if !self.sys.usb_debugging && has_usb {
                self.notifications.retain(|n| n.id != 4);
            }
        }

        let pointer = input.pointer;

        // Global hardware keyboard shortcuts (when no text field owns focus)
        let text_focused = matches!(
            f.focused_id(),
            Some(
                ID_DRAWER_SEARCH
                    | ID_WIDGETS_SEARCH
                    | ID_SETTINGS_SEARCH
                    | ID_DNS_HOSTNAME
                    | ID_BROWSER_URL
                    | ID_CHAT_COMPOSER
            )
        );
        if !text_focused && !self.lockscreen.bouncer_open {
            if f.pressed_key(Key::Home) {
                self.system_key(1, now);
            } else if f.pressed_key(Key::Menu) {
                self.system_key(2, now);
            } else if f.pressed_key(Key::Escape) {
                self.system_key(3, now);
            } else if f.pressed_key(Key::PageUp)
                || f.pressed_key(Key::Character('+'))
                || f.pressed_key(Key::Character('='))
            {
                self.system_key(4, now);
            } else if f.pressed_key(Key::PageDown) || f.pressed_key(Key::Character('-')) {
                self.system_key(5, now);
            } else if f.pressed_key(Key::End) {
                self.system_key(6, now);
            }
        }

        // Handle desktop bezel vs mobile edge-to-edge layout
        let desktop_frame = width > 540.0 && width > height * 0.72;
        let phone = if desktop_frame {
            f.paint_rect(viewport, 0.0, Color(0x13, 0x15, 0x1B, 255));
            let pw = 411.0_f32.min(width - 72.0);
            let ph = (height - 36.0).clamp(560.0, 846.0);
            let px = ((width - pw) * 0.5).floor();
            let py = ((height - ph) * 0.5).floor();
            let phone = Rect {
                x: px,
                y: py,
                width: pw,
                height: ph,
            };
            self.paint_desktop_bezel(f, phone, now);
            phone
        } else {
            viewport
        };

        self.last_phone_rect = phone;
        self.last_heads_up_rect = None;

        if self.lockscreen.locked {
            let p = self.sys.palette();
            let lock_act = self
                .lockscreen
                .show(f, phone, input, &mut self.sys, &self.notifications, p, &self.wallpapers.lock, self.clock);
            if !self.lockscreen.pending_haptics.is_empty() {
                self.pending_haptics
                    .extend(self.lockscreen.pending_haptics.drain(..));
            }
            match lock_act {
                lockscreen::LockAction::OpenShortcut(shortcut) => self.open_lock_shortcut(shortcut, now),
                lockscreen::LockAction::DismissNotification(id) => {
                    self.trigger_haptic(services::HapticEffect::Tick);
                    self.dismiss_notification(id, notifications::CLOSE_REASON_DISMISSED);
                    f.continuous(phone);
                    return;
                }
                lockscreen::LockAction::ClearAllNotifications => {
                    self.trigger_haptic(services::HapticEffect::Click);
                    self.clear_all_dismissible_notifications(now);
                    f.continuous(phone);
                    return;
                }
                lockscreen::LockAction::OpenNotification(id) => {
                    self.invoke_notification_action(id, "default", None, now);
                }
                lockscreen::LockAction::RetryTouchId => {
                    self.pending_touch_id_retry = true;
                    f.continuous(phone);
                    return;
                }
                lockscreen::LockAction::Unlocked => {}
                lockscreen::LockAction::None => return,
            }
        }

        self.lockscreen.advance_unlock(now, self.sys.anim_scale(), reduced);
        self.finish_app_opening(now, reduced);
        let status_h = 31.0;
        let nav_h = self.nav_bar_height();
        let suppress_child_input = self.lockscreen.is_unlocking()
            || (matches!(self.power_state, PowerState::Running)
                && self.process_pointer_gestures(f, input, phone, status_h, nav_h, now, dt));

        // Step animations (skip stepping whichever property is actively scrubbed 1:1!)
        let mut animating = false;
        let duration_scale = self.sys.anim_scale() as f64;
        if !matches!(self.gesture, ActiveGesture::DrawerScrub { .. } | ActiveGesture::NavGesture { .. }) {
            animating |= self.drawer_anim.step(now, reduced, duration_scale);
        }
        if !matches!(self.gesture, ActiveGesture::WidgetsScrub { .. }) {
            animating |= self.widgets_anim.step(now, reduced, duration_scale);
        }
        animating |= self.popup_anim.step(now, reduced, duration_scale);
        if self.home_popup == HomePopup::None && self.popup_anim.value <= 0.005 {
            self.last_home_popup = HomePopup::None;
        }
        animating |= self.launcher_sub_anim.step(now, reduced, duration_scale);
        if self.launcher_sub_anim.value >= 0.999 {
            self.prev_launcher_sub = LauncherSub::None;
        }
        animating |= self.app_anim.step(now, reduced, duration_scale);
        if !matches!(self.gesture, ActiveGesture::ShadeScrub { .. }) {
            animating |= if self.shade_stage == 1 && self.shade_anim.from > 1.0 {
                self.shade_anim.step_with_easing(now, reduced, duration_scale, emphasized_decelerate)
            } else {
                self.shade_anim.step(now, reduced, duration_scale)
            };
        }
        if self.shade_stage == 1 && !matches!(self.gesture, ActiveGesture::ShadeScrub { .. }) {
            animating |= self.notification_sheet_bounce.step_with_easing(now, reduced, duration_scale, |t| t);
        } else {
            self.notification_sheet_bounce.snap(1.0);
        }
        // Once the expanded grid is gone, discard its page without a visible jump.
        if self.shade_anim.value <= 1.0 {
            self.qs_page = 0;
            self.qs_page_anim.snap(0.0);
        } else if !matches!(self.gesture, ActiveGesture::QsPageSwipe { .. }) {
            animating |= self.qs_page_anim.step(now, reduced, duration_scale);
        }
        animating |= self.qs_edit_anim.step(now, reduced, duration_scale);
        animating |= self.power_anim.step(now, reduced, duration_scale);
        animating |= self.vol_anim.step(now, reduced, duration_scale);
        animating |= self.vol_expand_anim.step(now, reduced, duration_scale);
        if self.overview_exiting && self.mode == ShellMode::Home {
            animating |= self.overview_anim.step_with_easing(now, reduced, duration_scale, emphasized_decelerate);
            self.overview_exiting = self.overview_anim.value > 0.0;
        } else {
            self.overview_exiting = false;
            animating |= self.overview_anim.step(now, reduced, duration_scale);
        }
        if let Some((task, mut slide, dismiss)) = self.overview_vertical_settle.take()
            && self.mode == ShellMode::Overview
        {
            if slide.step(now, reduced, duration_scale) {
                self.overview_vertical_settle = Some((task, slide, dismiss));
                animating = true;
            } else if dismiss && let Some(idx) = self.tasks.iter().position(|&entry| entry == task) {
                self.dismiss_overview_task(idx, now);
            }
        }
        if self.overview_anim.value >= 0.995 {
            self.overview_enter_from = None;
        }
        let active_swipe_id = match self.gesture {
            ActiveGesture::NotifSwipe { notif_id, .. } => Some(notif_id),
            _ => None,
        };
        for n in &mut self.notifications {
            animating |= n.expand_anim.step(now, reduced, duration_scale);
            animating |= n.controls_anim.step(now, reduced, duration_scale);
            if active_swipe_id != Some(n.id) {
                animating |= n.swipe_anim.step(now, reduced, duration_scale);
            }
            if n.dismissing {
                animating |= n.dismiss_anim.step(now, reduced, duration_scale);
            }
            if let Some((_, t_start, held)) = n.ripple {
                if !held && now - t_start >= 0.42 {
                    n.ripple = None;
                } else {
                    animating = true;
                }
            }
        }
        self.notifications
            .retain(|n| !(n.dismissing && n.dismiss_anim.value <= 0.005));
        self.finish_app_opening(now, reduced);
        let active_transition_t: Option<f32> = match self.app_transition {
            AppTransition::None => None,
            AppTransition::Opening {
                start,
                duration,
                ..
            } | AppTransition::OpeningDesktop { start, duration, .. } => {
                animating = true;
                Some(self.preview_transition_t
                    .unwrap_or_else(|| ((now - start) / duration) as f32)
                    .clamp(0.0, 1.0))
            }
            AppTransition::Closing {
                start, duration, ..
            } => {
                let t = if let Some(pt) = self.preview_transition_t {
                    pt.clamp(0.0, 1.0)
                } else if reduced || duration <= 0.0 {
                    1.0
                } else {
                    ((now - start) / duration).clamp(0.0, 1.0) as f32
                };
                if self.preview_transition_t.is_none() && t >= 1.0 {
                    self.app_transition = AppTransition::None;
                    None
                } else {
                    animating = true;
                    Some(t)
                }
            }
        };
        if let Some(glide) = self.overview_glide
            && !matches!(self.gesture, ActiveGesture::OverviewHorizontal { .. })
        {
            let (pos, moving) = glide.sample(now, reduced);
            self.overview_scroll = pos;
            if moving {
                animating = true;
            } else {
                self.overview_glide = None;
            }
        }
        let vp_h = (phone.height - status_h - nav_h).max(200.0);
        animating |= self.step_scroll_fling(dt, vp_h, reduced);
        for st in &mut self.scroll_stretches {
            animating |= st.step(dt, vp_h, reduced);
        }
        animating |= self.overview_edge_spring.step(
            dt / (duration_scale as f32).max(0.01),
            phone.width,
            reduced || duration_scale == 0.0,
        );
        animating |= self.home_page_anim.step(now, reduced, duration_scale);
        animating |= self.home_reorder_anim.step(now, reduced, duration_scale);
        animating |= self.home_stretch.step(dt, phone.width, reduced);
        if self.ime_pending_open {
            self.ime_pending_open = false;
            self.ime_anim.set(1.0, now, 0.24, 0.20);
        }
        animating |= self.ime_anim.step(now, reduced, duration_scale);
        self.settings_app
            .set_edge_stretch(self.target_stretch(ScrollTarget::Settings));

        // Auto-hide Volume panel after inactivity
        if self.vol_open && self.vol_hide_at > 0.0 {
            if now >= self.vol_hide_at {
                self.vol_open = false;
                self.vol_expanded = false;
                self.vol_anim.set(0.0, now, 0.22, 0.20);
                self.vol_expand_anim.set(0.0, now, 0.20, 0.18);
                animating = true;
            } else {
                f.wake_at(self.vol_hide_at);
            }
        }

        // Power-off / Reboot sequence
        match self.power_state {
            PowerState::ShuttingDown(t0_ms) => {
                let elapsed = now - (t0_ms as f64 / 1000.0);
                f.paint_rect(phone, 0.0, Color(0x0A, 0x0B, 0x0F, 255));
                f.styled_text(
                    Rect {
                        x: phone.x,
                        y: phone.y + phone.height * 0.45,
                        width: phone.width,
                        height: 28.0,
                    },
                    "Shutting down...",
                    Color(0xE4, 0xE2, 0xE6, 255),
                    text_style(18.0, 500, 1),
                );
                let ring = Rect {
                    x: phone.x + phone.width * 0.5 - 18.0,
                    y: phone.y + phone.height * 0.52,
                    width: 36.0,
                    height: 36.0,
                };
                f.paint_arc(
                    ring,
                    (elapsed * 5.0) as f32,
                    3.8,
                    3.0,
                    self.sys.palette().primary,
                );
                f.continuous(phone);
                if elapsed >= 1.4 {
                    self.power_state = PowerState::PoweredOff;
                }
                return;
            }
            PowerState::PoweredOff => {
                f.paint_rect(phone, 0.0, Color(0x06, 0x07, 0x09, 255));
                f.styled_text(
                    Rect {
                        x: phone.x,
                        y: phone.y + phone.height * 0.46,
                        width: phone.width,
                        height: 24.0,
                    },
                    "Device powered off",
                    Color(0x88, 0x8C, 0x96, 255),
                    text_style(16.0, 500, 1),
                );
                f.styled_text(
                    Rect {
                        x: phone.x,
                        y: phone.y + phone.height * 0.51,
                        width: phone.width,
                        height: 22.0,
                    },
                    "Tap screen or press Power to wake",
                    Color(0x5E, 0x62, 0x6D, 255),
                    text_style(13.0, 400, 1),
                );
                if f.target(SHELL_ID + 99, phone, "Wake device", true, Role::Button)
                    .clicked
                {
                    self.power_state = PowerState::Rebooting((now * 1000.0) as u64);
                }
                return;
            }
            PowerState::Rebooting(t0_ms) => {
                let elapsed = now - (t0_ms as f64 / 1000.0);
                f.paint_rect(phone, 0.0, Color(0x0C, 0x0E, 0x13, 255));
                // LineageOS-inspired cyan arc boot logo
                let cy = phone.y + phone.height * 0.46;
                let cx = phone.x + phone.width * 0.5;
                let pulse = ((elapsed * 3.2).sin() as f32 * 0.5 + 0.5) * 14.0;
                f.paint_arc(
                    Rect {
                        x: cx - 54.0,
                        y: cy - 22.0,
                        width: 108.0,
                        height: 44.0,
                    },
                    std::f32::consts::PI,
                    std::f32::consts::PI,
                    4.0,
                    Color(0x16, 0x7C, 0x80, 255),
                );
                f.paint_rect(
                    Rect {
                        x: cx - 10.0 - pulse * 0.3,
                        y: cy - 10.0,
                        width: 20.0,
                        height: 20.0,
                    },
                    10.0,
                    Color(0x4E, 0xC6, 0xC9, 255),
                );
                f.styled_text(
                    Rect {
                        x: phone.x,
                        y: cy + 34.0,
                        width: phone.width,
                        height: 24.0,
                    },
                    "LineageOS 20 · SpectreOS",
                    Color(0xCC, 0xD2, 0xDC, 255),
                    text_style(14.0, 500, 1),
                );
                f.continuous(phone);
                if elapsed >= 1.3 {
                    self.power_state = PowerState::Running;
                    self.go_home(now);
                }
                return;
            }
            PowerState::Running => {}
        }

        let app_rect = Rect {
            x: phone.x,
            y: phone.y + status_h,
            width: phone.width,
            height: (phone.height - status_h - nav_h).max(120.0),
        };
        let status_rect = Rect {
            x: phone.x,
            y: phone.y,
            width: phone.width,
            height: status_h,
        };
        let nav_rect = Rect {
            x: phone.x,
            y: phone.y + phone.height - nav_h,
            width: phone.width,
            height: nav_h,
        };

        let ime_hovered = (self.ime_target.is_some() || self.ime_anim.value > 0.1)
            && pointer.is_some_and(|p| p[1] >= phone.y + phone.height - nav_h - 236.0);
        let vol_hovered = (self.vol_open || self.vol_anim.value > 0.05)
            && pointer.is_some_and(|[px, py]| {
                let w = if self.vol_expanded { 256.0 } else { 68.0 };
                px >= phone.x + phone.width - w - 10.0
                    && py >= phone.y + phone.height * 0.25
                    && py <= phone.y + phone.height * 0.78
            });
        let shade_visible = self.notification_shade_visible();
        let live_wayland = self.shows_wayland_surface();
        if self.composites_wayland_surface() {
            f.clear(Color(0, 0, 0, 0));
        }
        let top_overlay_blocks = shade_visible
            || self.power_open
            || self.power_anim.value > 0.05
            || self.active_apps_open;
        let modal_over_shade = self.power_open
            || self.power_anim.value > 0.05
            || self.active_apps_open;

        // Reset cached hit-geometry for this frame before painting layers
        self.last_vol_tracks = [None; 4];
        self.last_brightness_rect = Rect::default();
        self.last_qs_grid_rect = Rect::default();
        self.last_radio_tile_rects = [None; 2];
        self.last_qs_edit_btn = Rect::default();
        self.last_notif_container = Rect::default();
        self.last_notif_rects.clear();
        self.last_overview_cards.clear();
        self.last_overview_rect = Rect::default();

        // Live bottom-pill swipe transform when dragging up from a Fullscreen/Split app
        let (nav_dx, nav_dy, nav_hold) = match self.gesture {
            ActiveGesture::NavGesture {
                delta,
                from_app: true,
                hold_time,
                ..
            } => (delta[0], delta[1], hold_time),
            _ => (0.0, 0.0, 0.0),
        };

        // Active Edge-Back state (live drag OR deterministic capture preview)
        let active_edge_back: Option<(bool, f32, f32, f32, bool)> = match self.gesture {
            ActiveGesture::EdgeBack {
                right_edge,
                y,
                pull,
                committed,
                ..
            } => Some((!right_edge, self.press_origin[1], y, pull, committed)),
            _ => self.preview_edge_back.map(|(right_edge, y_frac, pull)| {
                let sy = phone.y + phone.height * y_frac;
                (!right_edge, sy, sy, pull, pull >= 44.0)
            }),
        };

        // Keep active app full-size during right/left edge-back swipe until release
        let pred_back: Option<(f32, f32, f32, f32)> = None;

        let hidden_home_app: Option<AppId> = match (self.app_transition, active_transition_t) {
            (
                AppTransition::Opening {
                    app,
                    from_icon: true,
                    from_shade: false,
                    ..
                },
                Some(t),
            ) if t < 0.95 => Some(app),
            (
                AppTransition::Closing {
                    app,
                    target_icon: Some(_),
                    ..
                },
                Some(t),
            ) if t < 0.96 || app == AppId::Settings => Some(app),
            _ => None,
        };
        let opening_from_shade = matches!(
            (self.app_transition, active_transition_t),
            (AppTransition::Opening { from_shade: true, .. }, Some(_))
        );

        // Clip entire phone display cleanly inside `phone`; suppress child pointer input whenever
        // our unified gesture engine has intercepted a drag (zero stray clicks on release!)
        f.within(phone, |f| {
            f.cancel_pointer_input(suppress_child_input, |f| {
                // === LAYER 1 / 2 / 3: Main Shell Content & App Open/Close Transitions ===
                f.without_input(top_overlay_blocks || active_transition_t.is_some(), |f| {
                    f.without_pointer_input(vol_hovered || ime_hovered, |f| {
                        if let Some(raw_t) = active_transition_t {
                            match self.app_transition {
                                AppTransition::Opening {
                                    app, origin, from_overview: true, ..
                                } => self.render_overview_reopen(
                                    f, LaunchTarget::Builtin(app), origin, raw_t, phone, app_rect, status_h, nav_h, now, gmail_text,
                                ),
                                AppTransition::Opening {
                                    app,
                                    origin,
                                    from_icon,
                                    from_shade: false,
                                    themed_icon,
                                    ..
                                } => {
                                    self.render_app_open_transition(
                                        f,
                                        LaunchTarget::Builtin(app),
                                        origin,
                                        from_icon,
                                        themed_icon,
                                        raw_t,
                                        phone,
                                        app_rect,
                                        status_h,
                                        nav_h,
                                        pointer,
                                        now,
                                        gmail_text,
                                        hidden_home_app,
                                    );
                                }
                                AppTransition::Opening { from_shade: true, .. } => {
                                    // Shade stays underneath while notification card expands over it (Layer 4b!)
                                    self.render_home_layer(
                                        f, phone, status_h, nav_h, pointer, now, gmail_text, None,
                                    );
                                }
                                AppTransition::OpeningDesktop { origin, from_icon, from_overview, .. } => {
                                    if let ShellMode::FullscreenDesktop(idx) = self.mode {
                                        let target = LaunchTarget::Desktop(idx);
                                        if from_overview {
                                            self.render_overview_reopen(
                                                f, target, origin, raw_t, phone, app_rect, status_h, nav_h, now, gmail_text,
                                            );
                                        } else {
                                            self.render_app_open_transition(
                                                f, target, origin, from_icon, false, raw_t, phone, app_rect,
                                                status_h, nav_h, pointer, now, gmail_text, None,
                                            );
                                        }
                                    }
                                }
                                AppTransition::Closing {
                                    app,
                                    from_rect,
                                    from_rad,
                                    target_icon,
                                    themed_icon,
                                    ..
                                } => {
                                    self.render_app_close_transition(
                                        f,
                                        app,
                                        from_rect,
                                        from_rad,
                                        target_icon,
                                        themed_icon,
                                        raw_t,
                                        phone,
                                        app_rect,
                                        status_h,
                                        nav_h,
                                        pointer,
                                        now,
                                        gmail_text,
                                        hidden_home_app,
                                    );
                                }
                                AppTransition::None => {}
                            }
                        } else if live_wayland {
                            // While the shade owns the display, keep the app's last view fixed.
                            if shade_visible && let Some(frame) = &self.wayland_shade_frame {
                                self.paint_wayland_window_frame(f, phone, app_rect, frame);
                            }
                            f.paint_rect(status_rect, 0.0, Color(0x0E, 0x11, 0x17, 245));
                            let nav_surface = self.active_wayland_app.map_or(
                                self.sys.palette().settings_bg,
                                |app| self.app_backdrop_color(app),
                            );
                            f.paint_rect(nav_rect, 0.0, nav_surface);
                        } else {
                            match self.mode {
                                ShellMode::Home => {
                                    if self.tasks.is_empty() && self.overview_anim.value > 0.0 {
                                        self.render_empty_overview(f, phone, status_h, nav_h, now, gmail_text);
                                    } else if self.overview_exiting {
                                        let home_alpha = smoothstep(0.10, 0.95, 1.0 - self.overview_anim.value);
                                        self.wallpapers.home.paint(f, phone, self.sys.wallpaper_idx);
                                        f.without_input(true, |f| {
                                            if home_alpha > 0.0 {
                                                f.push_layer(phone, home_alpha);
                                                self.render_home_layer(f, phone, status_h, nav_h, None, now, gmail_text, None);
                                                f.pop_layer();
                                            }
                                            self.render_overview(f, phone, app_rect, status_h, nav_h, None, None, gmail_text, now);
                                        });
                                    } else {
                                        self.render_home_layer(
                                            f, phone, status_h, nav_h, pointer, now, gmail_text, None,
                                        );
                                    }
                                }
                                ShellMode::Fullscreen(app) => {
                                    let s = 0.90 + 0.10 * self.app_anim.value;
                                    let up = (-nav_dy).max(0.0);
                                    let drag_p = (up / 300.0).clamp(0.0, 1.0);
                                    let (final_s, tx, ty, card_rad, show_home_under) =
                                        if let Some((ps, ptx, pty, prad)) = pred_back {
                                            (ps, ptx, pty, prad, true)
                                        } else {
                                            let fs = lerp(s, 0.67, drag_p).clamp(0.62, 1.0);
                                            let dx = nav_dx * 0.44;
                                            let dy = lerp(0.0, -44.0, drag_p);
                                            let moved = fs < 0.999 || dy.abs() > 0.5 || dx.abs() > 0.5;
                                            (fs, dx, dy, lerp(0.0, 26.0, drag_p), moved)
                                        };
                                    let mut painted_settings_snapshot = false;
                                    if show_home_under {
                                        f.without_input(true, |f| {
                                            self.render_home_layer(
                                                f, phone, status_h, nav_h, pointer, now, gmail_text, None,
                                            );
                                        });
                                        let p = self.sys.palette();
                                        let ov_peek = if pred_back.is_none() {
                                            smoothstep(0.14, 0.78, drag_p)
                                                .max((nav_hold as f32 / 0.18).clamp(0.0, 1.0))
                                        } else {
                                            0.0
                                        };
                                        if app != AppId::Settings {
                                            f.paint_rect(
                                                phone,
                                                0.0,
                                                Color(0, 0, 0, 42).mix(p.recents_bg, ov_peek * 0.75),
                                            );
                                        }
                                        let w = phone.width * final_s;
                                        let h = phone.height * final_s;
                                        let mut preview_r = Rect {
                                            x: phone.x + (phone.width - w) * 0.5 + tx,
                                            y: phone.y + (phone.height - h) * 0.5 + ty,
                                            width: w,
                                            height: h,
                                        };
                                        if app == AppId::Settings && pred_back.is_none() {
                                            let (bounds, radius) =
                                                self.settings_nav_preview(phone, [nav_dx, nav_dy], nav_hold);
                                            preview_r = bounds;
                                            if let Some(image) = &self.settings_return_snapshot {
                                                painted_settings_snapshot =
                                                    f.paint_image(bounds, image, ImageFit::Contain, radius);
                                            }
                                        }
                                        // Slide previous task preview card in from left edge when pulling up / holding (`strip-g4`)!
                                        if ov_peek > 0.04 && self.tasks.len() > 1 {
                                            let prev_app = match self.tasks[1] {
                                                TaskEntry::Single(a) => a,
                                                TaskEntry::SplitPair(a, _) => a,
                                                TaskEntry::Desktop(_) => AppId::Settings,
                                            };
                                            let gap = 20.0;
                                            let slide = smoothstep(0.04, 0.85, ov_peek);
                                            let prev_r = Rect {
                                                x: lerp(phone.x - w - 16.0, preview_r.x - w - gap, slide),
                                                y: preview_r.y,
                                                width: w,
                                                height: h,
                                            };
                                            if prev_r.x + prev_r.width > phone.x {
                                                if app != AppId::Settings {
                                                    f.elevation(prev_r, 26.0, 3);
                                                }
                                                f.within(prev_r, |f| {
                                                    self.builtin_apps.paint_preview(
                                                        f, prev_app, prev_r, 26.0, &self.sys,
                                                    );
                                                });
                                            }
                                        }
                                        if app != AppId::Settings {
                                            f.shadow(
                                                preview_r,
                                                card_rad,
                                                [0.0, 10.0],
                                                26.0,
                                                0.0,
                                                Color(0, 0, 0, 96),
                                            );
                                        }
                                        if !painted_settings_snapshot {
                                            let sx = preview_r.width / phone.width;
                                            let sy = preview_r.height / phone.height;
                                            f.push_transform(
                                                [phone.x + phone.width * 0.5, phone.y + phone.height * 0.5],
                                                [
                                                    preview_r.x + preview_r.width * 0.5 - (phone.x + phone.width * 0.5),
                                                    preview_r.y + preview_r.height * 0.5 - (phone.y + phone.height * 0.5),
                                                ],
                                                [sx, sy],
                                                0.0,
                                            );
                                        }
                                    }
                                    if !painted_settings_snapshot {
                                        self.paint_app_backdrop(f, app, phone);
                                        let action = self.render_live_app(
                                            f,
                                            app,
                                            app_rect,
                                            APP_ID_BASE + ((app as u64) << 28),
                                            pointer,
                                            gmail_text,
                                        );
                                        if show_home_under {
                                            f.pop_transform();
                                        }
                                        self.apply_shell_action(action, now);
                                    }
                                }
                                ShellMode::FullscreenDesktop(idx) => {
                                    self.paint_desktop_launch_splash(f, idx, phone);
                                }
                                ShellMode::Split { top, bottom } => {
                                    let up = (-nav_dy).max(0.0);
                                    let drag_p = (up / 300.0).clamp(0.0, 1.0);
                                    let (final_s, tx, ty) = if let Some((ps, ptx, pty, _)) = pred_back {
                                        (ps, ptx, pty)
                                    } else {
                                        (lerp(1.0, 0.67, drag_p).clamp(0.62, 1.0), nav_dx * 0.42, lerp(0.0, -40.0, drag_p))
                                    };
                                    if final_s < 0.999 {
                                        f.without_input(true, |f| {
                                            self.render_home_layer(
                                                f, phone, status_h, nav_h, pointer, now, gmail_text, None,
                                            );
                                        });
                                        f.paint_rect(phone, 0.0, Color(0, 0, 0, 42));
                                        f.push_transform(
                                            [phone.x + phone.width * 0.5, phone.y + phone.height * 0.5],
                                            [tx, ty],
                                            [final_s, final_s],
                                            0.0,
                                        );
                                    }
                                    self.render_split_screen(
                                        f, app_rect, phone, top, bottom, pointer, now, gmail_text,
                                    );
                                    if final_s < 0.999 {
                                        f.pop_transform();
                                    }
                                }
                                ShellMode::Overview => {
                                    self.render_overview(f, phone, app_rect, status_h, nav_h, None, pointer, gmail_text, now);
                                }
                                ShellMode::SplitSelect(first) => {
                                    self.render_overview(f, phone, app_rect, status_h, nav_h, Some(first), pointer, gmail_text, now);
                                }
                            }
                        }
                    });
                });

                // === LAYER 4: True Slow-Mo Morphing Notification Shade & Quick Settings ===
                if shade_visible {
                    let base_light_ink = self.underlying_status_bar_light_ink();
                    f.without_input(modal_over_shade || opening_from_shade, |f| {
                        f.without_pointer_input(vol_hovered || ime_hovered, |f| {
                            self.render_shade_and_qs(
                                f,
                                phone,
                                status_h,
                                base_light_ink,
                                pointer,
                                self.pointer_down,
                                now,
                            );
                        });
                    });
                }

                // === LAYER 4c: Google Keyboard (Gboard Material You) ===
                if self.ime_target.is_some() || self.ime_anim.value > 0.01 {
                    self.render_gboard(f, phone, nav_h, now);
                }

                // === LAYER 4b: Android 13 NotificationLaunchAnimator (expanding card OVER shade!) ===
                if let (
                    AppTransition::Opening {
                        app,
                        origin,
                        from_shade: true,
                        ..
                    },
                    Some(raw_t),
                ) = (self.app_transition, active_transition_t)
                {
                    self.render_notif_launch_transition(
                        f,
                        app,
                        origin,
                        raw_t,
                        phone,
                        app_rect,
                        pointer,
                        gmail_text,
                    );
                }

                // === Persistent Status Bar (when shade is closed or QS editor open) ===
                if (!shade_visible || self.qs_edit_open || self.qs_edit_anim.value > 0.35)
                    && !opening_from_shade
                {
                    let status_light_ink = self.status_bar_light_ink();
                    f.without_input(modal_over_shade, |f| {
                        if live_wayland {
                            self.render_wayland_status_bar(f, status_rect, now);
                        } else {
                            self.render_status_bar(f, status_rect, status_light_ink, now);
                        }
                    });
                }

                // === Bottom Navigation Bar (Gesture pill or 3-Button bar) ===
                let nav_light_ink = self.nav_bar_light_ink();
                if self.shade_anim.value < 0.45 && !self.power_open && !opening_from_shade {
                    f.without_input(modal_over_shade, |f| {
                        self.render_nav_bar(f, nav_rect, nav_light_ink, now);
                    });
                }

                // === LAYER 5: Volume Side Panel, Active Apps Dialog, Power Menu ===
                if self.vol_anim.value > 0.005 || self.vol_open {
                    self.render_volume_panel(f, phone, now);
                }
                if self.active_apps_open {
                    self.render_active_apps_dialog(f, phone, now);
                }
                if self.power_anim.value > 0.005 || self.power_open {
                    self.render_power_menu(f, phone, now);
                }
                if !shade_visible && !self.power_open {
                    self.render_heads_up_banner(f, phone, status_h + 6.0, now);
                }
            });

            // === LAYER 6: Android 13 Edge-Back Folding Chevron Indicator ===
            if let Some((from_left, _start_y, cur_y, pull, committed)) = active_edge_back {
                self.render_edge_back_indicator(f, phone, from_left, cur_y, pull, committed);
            }

            self.lockscreen.paint_unlock_reveal(f, phone, self.sys.palette(), self.sys.lock_method);
            self.paint_display_effects(f, phone, now);

            // Show taps developer feedback dot
            if self.sys.show_taps
                && let Some([px, py]) = pointer
                && phone.contains([px, py])
            {
                f.paint_rect(
                    Rect {
                        x: px - 10.0,
                        y: py - 10.0,
                        width: 20.0,
                        height: 20.0,
                    },
                    10.0,
                    Color(255, 255, 255, 130),
                );
            }

            // System Toast Pill
            if let Some((msg, until)) = &self.toast {
                if now < *until {
                    let tw = (f.text_width(msg, text_style(13.0, 500, 1)) + 36.0)
                        .clamp(120.0, phone.width - 40.0);
                    let tr = Rect {
                        x: phone.x + (phone.width - tw) * 0.5,
                        y: phone.y + phone.height - nav_h - 68.0,
                        width: tw,
                        height: 40.0,
                    };
                    f.elevation(tr, 20.0, 3);
                    f.paint_rect(tr, 20.0, Color(0x28, 0x2A, 0x30, 245));
                    f.styled_text(
                        tr,
                        msg,
                        Color(0xF5, 0xF5, 0xFA, 255),
                        text_style(13.0, 500, 1),
                    );
                    f.wake_at(*until);
                } else {
                    self.toast = None;
                }
            }
        });

        // Clean anti-aliased rounded screen corners inside the desktop phone bezel (ZERO corner overflow!)
        if desktop_frame {
            icons::paint_rounded_screen_mask(f, phone, 22.0, Color(0x0B, 0x0C, 0x10, 255));
        }

        if animating || self.has_pending_animation() {
            f.continuous(viewport);
        }
    }

    fn paint_display_effects(&self, f: &mut Frame<'_>, phone: Rect, now: f64) {
        if self.sys.night_light {
            f.paint_rect(phone, 0.0, Color(0xFF, 0x9C, 0x2A, 28));
        }
        let dim = ((0.70 - self.sys.brightness) / 0.55 * 140.0).clamp(0.0, 140.0) as u8;
        if dim > 0 {
            f.paint_rect(phone, 0.0, Color(0, 0, 0, dim));
        }
        if self.sys.invert_colors {
            f.paint_rect(phone, 0.0, Color(0xD8, 0xE2, 0xFF, 40));
        }
        if self.flash_until > now {
            let frac = ((self.flash_until - now) / 0.32).clamp(0.0, 1.0) as f32;
            f.paint_rect(phone, 0.0, Color(255, 255, 255, (frac * 230.0) as u8));
            f.continuous(phone);
        }
    }

    fn has_pending_animation(&self) -> bool {
        let active = |a: Anim| (a.value - a.target).abs() > 0.0005 && a.duration > 0.0;
        active(self.drawer_anim)
            || active(self.widgets_anim)
            || active(self.popup_anim)
            || active(self.launcher_sub_anim)
            || active(self.app_anim)
            || active(self.shade_anim)
            || active(self.notification_sheet_bounce)
            || active(self.qs_page_anim)
            || active(self.qs_edit_anim)
            || active(self.power_anim)
            || active(self.vol_anim)
            || active(self.vol_expand_anim)
            || active(self.overview_anim)
            || active(self.home_page_anim)
            || active(self.home_reorder_anim)
            || active(self.ime_anim)
            || !matches!(self.app_transition, AppTransition::None)
            || self.overview_glide.is_some()
            || self.scroll_fling.is_some()
            || !matches!(self.power_state, PowerState::Running)
    }

    fn underlying_status_bar_light_ink(&self) -> bool {
        match self.mode {
            ShellMode::Home => {
                if self.drawer_anim.value > 0.85
                    || self.widgets_anim.value > 0.85
                    || self.launcher_sub != LauncherSub::None
                {
                    self.sys.dark_theme
                } else {
                    true
                }
            }
            ShellMode::Fullscreen(app) => app.uses_light_status_ink(self.sys.dark_theme),
            ShellMode::FullscreenDesktop(_) => true,
            ShellMode::Split { top, .. } => top.uses_light_status_ink(self.sys.dark_theme),
            ShellMode::Overview => true,
            ShellMode::SplitSelect(_) => self.sys.dark_theme,
        }
    }

    fn status_bar_light_ink(&self) -> bool {
        if self.shade_anim.value > 0.25 || self.qs_edit_open {
            return self.sys.dark_theme;
        }
        self.underlying_status_bar_light_ink()
    }

    fn nav_bar_light_ink(&self) -> bool {
        match self.mode {
            ShellMode::Home => {
                if self.drawer_anim.value > 0.35
                    || self.widgets_anim.value > 0.35
                    || self.launcher_sub != LauncherSub::None
                {
                    self.sys.dark_theme
                } else {
                    true
                }
            }
            ShellMode::Fullscreen(app) => app.uses_light_status_ink(self.sys.dark_theme),
            ShellMode::FullscreenDesktop(_) => self.sys.dark_theme,
            ShellMode::Split { bottom, .. } => bottom.uses_light_status_ink(self.sys.dark_theme),
            ShellMode::Overview => true,
            ShellMode::SplitSelect(_) => self.sys.dark_theme,
        }
    }

    fn app_backdrop_color(&self, app: AppId) -> Color {
        let p = self.sys.palette();
        match app {
            AppId::Contacts => Color(0x2E, 0x4A, 0x7D, 255),
            AppId::Clock => Color(0x19, 0x19, 0x1D, 255),
            AppId::Camera => Color(0x0E, 0x0F, 0x12, 255),
            AppId::Gmail => Color(0xFF, 0xFF, 0xFF, 255),
            AppId::Recorder => Color(0x22, 0x11, 0x15, 255),
            _ => p.settings_bg,
        }
    }

    fn paint_app_backdrop(&self, f: &mut Frame<'_>, app: AppId, phone: Rect) {
        if app == AppId::Contacts {
            // Contacts has navy status/toolbar header and clean light list surface below!
            let hdr_h = 88.0_f32.min(phone.height);
            f.paint_rect(phone, 0.0, Color(0xF7, 0xF8, 0xFC, 255));
            f.paint_rect(
                Rect {
                    height: hdr_h,
                    ..phone
                },
                0.0,
                Color(0x2E, 0x4A, 0x7D, 255),
            );
        } else {
            f.paint_rect(phone, 0.0, self.app_backdrop_color(app));
        }
    }

    /// Android 13 Icon-Origin App Open Animation matching `strip-01-open-contacts.png` & `strip-05`:
    /// App card grows directly out of the tapped icon (`origin -> phone`), icon silhouette fades out
    #[allow(clippy::too_many_arguments)]
    fn paint_overview_app(
        &mut self,
        f: &mut Frame<'_>,
        app: AppId,
        phone: Rect,
        app_rect: Rect,
        card: Rect,
        radius: f32,
        gmail_text: &mut String,
    ) {
        f.paint_rect(card, radius, self.app_backdrop_color(app));
        if let Some(frame) = self.wayland_task_frame(TaskEntry::Single(app)) {
            f.paint_image(card, frame, ImageFit::Contain, radius);
            return;
        }
        if app == AppId::Settings && let Some(snapshot) = &self.settings_return_snapshot {
            f.paint_image(card, snapshot, ImageFit::Contain, radius);
            return;
        }

        // Scale the whole live frame into the rounded card's safe area.
        let safe = card.inset(radius * 0.30);
        let scale = (safe.width / phone.width).min(safe.height / phone.height);
        let center = [phone.x + phone.width * 0.5, phone.y + phone.height * 0.5];
        let translate = [
            card.x + card.width * 0.5 - center[0],
            card.y + card.height * 0.5 - center[1],
        ];
        f.without_input(true, |f| {
            f.push_transform(center, translate, [scale, scale], 0.0);
            f.within(phone, |f| {
                self.paint_app_backdrop(f, app, phone);
                let _ = self.render_live_app(f, app, app_rect,
                    APP_ID_BASE + ((app as u64) << 28), None, gmail_text);
            });
            f.pop_transform();
        });
    }

    fn render_overview_reopen(
        &mut self,
        f: &mut Frame<'_>,
        target: LaunchTarget,
        origin: Rect,
        raw_t: f32,
        phone: Rect,
        app_rect: Rect,
        status_h: f32,
        nav_h: f32,
        now: f64,
        gmail_text: &mut String,
    ) {
        self.wallpapers.home.paint(f, phone, self.sys.wallpaper_idx);
        let chrome_alpha = 1.0 - smoothstep(0.0, 0.60, raw_t);
        if chrome_alpha > 0.01 {
            f.push_layer(phone, chrome_alpha);
            self.render_overview(f, phone, app_rect, status_h, nav_h, None, None, gmail_text, now);
            f.pop_layer();
        }

        let progress = cubic_bezier(0.22, 0.0, 0.12, 1.0, raw_t);
        let card = lerp_rect(origin, phone, progress, progress);
        let radius = lerp(26.0 * (1.0 - self.overview_reopen_progress), 0.0, progress);
        match target {
            LaunchTarget::Builtin(app) if self.wayland_launch_frame.is_none() => {
                self.paint_overview_app(f, app, phone, app_rect, card, radius, gmail_text);
            }
            _ => {
                f.paint_rect(card, radius, self.sys.palette().settings_bg);
                f.push_transform(
                    [phone.x, phone.y],
                    [card.x - phone.x, card.y - phone.y],
                    [card.width / phone.width, card.height / phone.height],
                    0.0,
                );
                let inset = 0.30 * radius * (phone.width / card.width).max(phone.height / card.height);
                f.within(phone.inset(inset), |f| {
                    self.paint_launch_content(f, target, phone, app_rect, gmail_text);
                });
                f.pop_transform();
            }
        }
    }

    fn paint_desktop_launch_splash(&mut self, f: &mut Frame<'_>, idx: usize, phone: Rect) {
        let p = self.sys.palette();
        f.paint_rect(phone, 0.0, p.settings_bg);
        if let Some(entry) = self.desktop_registry.apps.get(idx).cloned()
        {
            let icon_img = self.desktop_registry.icon_for(&entry.id);
            let ir = Rect {
                x: phone.x + (phone.width - 76.0) * 0.5,
                y: phone.y + phone.height * 0.36,
                width: 76.0,
                height: 76.0,
            };
            freedesktop::paint_desktop_app_icon(
                f,
                ir,
                &entry.name,
                &entry.id,
                icon_img.as_ref(),
                &p,
            );
            f.styled_text(
                Rect {
                    x: phone.x + 24.0,
                    y: ir.y + 92.0,
                    width: phone.width - 48.0,
                    height: 28.0,
                },
                &entry.name,
                p.on_surface,
                text_style(20.0, 500, 1),
            );
            let sub = if entry.shiraz_package.is_some() {
                match self.sys.shiraz.state {
                    settings::ShirazRuntimeState::Warming => {
                        "Optimizing Shiraz runtime & launching…"
                    }
                    settings::ShirazRuntimeState::Starting => {
                        "Starting Shiraz Android runtime…"
                    }
                    _ => "Launching Android app via Shiraz…",
                }
            } else if self.native_compositor {
                "Launching Wayland application..."
            } else {
                entry.comment.as_deref().unwrap_or(&entry.id)
            };
            f.styled_text(
                Rect {
                    x: phone.x + 28.0,
                    y: ir.y + 124.0,
                    width: phone.width - 56.0,
                    height: 24.0,
                },
                sub,
                p.on_surface_variant,
                text_style(13.0, 400, 1),
            );
        }
    }

    fn paint_launch_content(
        &mut self,
        f: &mut Frame<'_>,
        target: LaunchTarget,
        phone: Rect,
        app_rect: Rect,
        gmail_text: &mut String,
    ) {
        if let Some(frame) = &self.wayland_launch_frame {
            self.paint_wayland_window_frame(f, phone, app_rect, frame);
        } else {
            match target {
                LaunchTarget::Builtin(app) => {
                    self.paint_app_backdrop(f, app, phone);
                    let _ = self.render_live_app(
                        f, app, app_rect, APP_ID_BASE + ((app as u64) << 28), None, gmail_text,
                    );
                }
                LaunchTarget::Desktop(idx) => self.paint_desktop_launch_splash(f, idx, phone),
            }
        }
    }

    fn paint_wayland_window_frame(&self, f: &mut Frame<'_>, phone: Rect, app_rect: Rect, frame: &Image) {
        f.paint_rect(phone, 0.0, self.sys.palette().settings_bg);
        let body = Rect {
            height: frame.height() as f32 * app_rect.width / frame.width() as f32,
            ..app_rect
        };
        f.paint_image(body, frame, ImageFit::Contain, 0.0);
    }

    /// during the first ~80ms (`0.04..0.28`), live app content expands & fades in (`0.10..0.44`),
    /// corners morph `origin_rad -> 24dp -> 0dp`, while Home gently zooms in (`1.0 -> 1.05`).
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn render_app_open_transition(
        &mut self,
        f: &mut Frame<'_>,
        target: LaunchTarget,
        origin: Rect,
        from_icon: bool,
        themed_icon: bool,
        raw_t: f32,
        phone: Rect,
        app_rect: Rect,
        status_h: f32,
        nav_h: f32,
        pointer: Option<[f32; 2]>,
        now: f64,
        gmail_text: &mut String,
        hidden_home_app: Option<AppId>,
    ) {
        let p = self.sys.palette();
        let settings_icon_launch = matches!(target, LaunchTarget::Builtin(AppId::Settings)) && from_icon;
        let e = if settings_icon_launch {
            cubic_bezier(0.27, 0.0, 0.03, 1.0, raw_t)
        } else {
            emphasized_decelerate(raw_t)
        };

        // 1. Underlying Home zooms subtly in (1.00 -> 1.05) + light scrim
        let home_s = if settings_icon_launch { 1.0 } else { lerp(1.0, 1.05, e) };
        f.push_transform(
            [phone.x + phone.width * 0.5, phone.y + phone.height * 0.5],
            [0.0, 0.0],
            [home_s, home_s],
            0.0,
        );
        self.render_home_layer(
            f,
            phone,
            status_h,
            nav_h,
            pointer,
            now,
            gmail_text,
            hidden_home_app,
        );
        f.pop_transform();
        if !settings_icon_launch {
            f.paint_rect(phone, 0.0, Color(0, 0, 0, (e * 34.0) as u8));
        }

        // 2. Expanding container rect & morphing corner radius
        let cur = lerp_rect(origin, phone, e, e);
        let start_rad = if from_icon {
            origin.width.min(origin.height) * 0.5
        } else {
            22.0
        };
        let rad = lerp(start_rad, 0.0, if settings_icon_launch {
            smoothstep(0.45, 1.0, raw_t)
        } else {
            smoothstep(0.32, 0.94, raw_t)
        });

        let (icon_bg, app_bg) = match target {
            LaunchTarget::Builtin(app) => (
                icons::app_icon_morph_bg(app, themed_icon, &p),
                if app == AppId::Contacts { Color(0xF7, 0xF8, 0xFC, 255) } else { self.app_backdrop_color(app) },
            ),
            LaunchTarget::Desktop(_) => (p.settings_bg, p.settings_bg),
        };
        let bg_mix = if settings_icon_launch {
            smoothstep(0.07, 0.21, raw_t)
        } else if from_icon {
            smoothstep(0.06, 0.34, raw_t)
        } else {
            1.0
        };
        let container_bg = icon_bg.mix(app_bg, bg_mix);
        f.paint_rect(cur, rad, container_bg);

        // 3. Live App UI scaling & fading cleanly across the ENTIRE growing container!
        // NOTE: Push transform directly (NO logical `f.within(cur)` before push_transform, which would
        // double-offset logical clipping and cull the AppBar!).
        let app_alpha = if settings_icon_launch {
            smoothstep(0.09, 0.23, raw_t)
        } else if from_icon {
            smoothstep(0.08, 0.42, raw_t)
        } else {
            smoothstep(0.0, 0.24, raw_t)
        };
        if app_alpha > 0.01 && cur.width > 12.0 && cur.height > 12.0 {
            let sx = (cur.width / phone.width).clamp(0.05, 1.0);
            let sy = (cur.height / phone.height).clamp(0.05, 1.0);
            f.push_transform(
                [phone.x, phone.y],
                [cur.x - phone.x, cur.y - phone.y],
                [sx, sy],
                0.0,
            );
            if app_alpha < 0.995 {
                f.push_layer(phone, app_alpha);
            }
            let mut paint_content = |f: &mut Frame<'_>| {
                self.paint_launch_content(f, target, phone, app_rect, gmail_text);
            };
            if settings_icon_launch || matches!(target, LaunchTarget::Desktop(_)) {
                // Keep the rectangular app content inside the rounded moving card.
                let inset_x = rad * 0.30 / sx;
                let inset_y = rad * 0.30 / sy;
                f.within(Rect {
                    x: phone.x + inset_x,
                    y: phone.y + inset_y,
                    width: phone.width - inset_x * 2.0,
                    height: phone.height - inset_y * 2.0,
                }, paint_content);
            } else {
                paint_content(f);
            }
            if app_alpha < 0.995 {
                f.pop_layer();
            }
            f.pop_transform();
        }

        // 4. Real launcher icon glyph fading rapidly out over the first ~80ms (0.04 -> 0.28)
        if from_icon {
            let icon_alpha = 1.0 - if settings_icon_launch {
                smoothstep(0.04, 0.18, raw_t)
            } else {
                smoothstep(0.04, 0.28, raw_t)
            };
            if icon_alpha > 0.01 {
                let gsz = lerp(
                    origin.width.min(origin.height),
                    (origin.width.min(origin.height) * 1.38).min(cur.width * 0.55),
                    raw_t,
                );
                let gr = Rect {
                    x: cur.x + (cur.width - gsz) * 0.5,
                    y: cur.y + (cur.height - gsz) * 0.5,
                    width: gsz,
                    height: gsz,
                };
                if icon_alpha < 0.995 {
                    f.push_layer(gr, icon_alpha);
                }
                match target {
                    LaunchTarget::Builtin(app) => icons::paint_real_app_morph_glyph(self.clock, f, gr, app, themed_icon, &p),
                    LaunchTarget::Desktop(idx) => {
                        if let Some(entry) = self.desktop_registry.apps.get(idx).cloned() {
                            let icon = self.desktop_registry.icon_for(&entry.id);
                            freedesktop::paint_desktop_app_icon(f, gr, &entry.name, &entry.id, icon.as_ref(), &p);
                        }
                    }
                }
                if icon_alpha < 0.995 {
                    f.pop_layer();
                }
            }
        }
    }

    /// Android 13 Return-to-Icon Close Animation matching `strip-02-close-contacts.png`!
    /// Window shrinks straight into its exact Home/Dock icon (`ex = emphasized_decelerate`,
    /// `ey = cubic_bezier(0.06, 0.62, 0.14, 1.0)`), morphing corners `from_rad -> circle` by `+233ms`,
    /// live app fading completely out (`0.12..0.46`) BEFORE circular clip, real icon glyph fading in
    /// (`0.22..0.52`) inside the lavender/icon disc, with subtle elastic settle (`1.04 -> 1.00`)!
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn render_app_close_transition(
        &mut self,
        f: &mut Frame<'_>,
        app: AppId,
        from_rect: Rect,
        from_rad: f32,
        target_icon: Option<Rect>,
        themed_icon: bool,
        raw_t: f32,
        phone: Rect,
        app_rect: Rect,
        status_h: f32,
        nav_h: f32,
        pointer: Option<[f32; 2]>,
        now: f64,
        gmail_text: &mut String,
        hidden_home_app: Option<AppId>,
    ) {
        let p = self.sys.palette();
        let settings_icon_return = app == AppId::Settings && target_icon.is_some();
        let morph_t = if settings_icon_return {
            (raw_t / SETTINGS_DOCK_FRACTION).min(1.0)
        } else {
            raw_t
        };
        let ex = if settings_icon_return {
            cubic_bezier(0.18, 0.0, 0.30, 1.0, morph_t)
        } else {
            emphasized_decelerate(raw_t)
        };

        // 1. Underlying Home zooms smoothly OUT 1.05 -> 1.00 (static target icon hidden until t >= 0.96!)
        let home_s = if settings_icon_return { 1.0 } else { lerp(1.05, 1.0, ex) };
        f.push_transform(
            [phone.x + phone.width * 0.5, phone.y + phone.height * 0.5],
            [0.0, 0.0],
            [home_s, home_s],
            0.0,
        );
        self.render_home_layer(
            f,
            phone,
            status_h,
            nav_h,
            pointer,
            now,
            gmail_text,
            hidden_home_app,
        );
        f.pop_transform();

        if let Some(target) = target_icon {
            let ey = if settings_icon_return { ex } else { cubic_bezier(0.06, 0.62, 0.14, 1.0, raw_t) };
            let mut cur = lerp_rect(from_rect, target, ex, ey);

            // Final subtle elastic settle (1.04 -> 1.00) over the last 22% as icon lands in slot
            if settings_icon_return {
                cur.y += motion::settle_bounce(raw_t, 0.48, 3.4);
            } else if raw_t > 0.78 {
                let bt = ((raw_t - 0.78) / 0.22).clamp(0.0, 1.0);
                let bounce = 1.0 + 0.042 * (bt * std::f32::consts::PI).sin();
                let cx = cur.x + cur.width * 0.5;
                let cy = cur.y + cur.height * 0.5;
                cur = Rect {
                    x: cx - cur.width * bounce * 0.5,
                    y: cy - cur.height * bounce * 0.5,
                    width: cur.width * bounce,
                    height: cur.height * bounce,
                };
            }

            let target_rad = target.width.min(target.height) * 0.5;
            let rad = lerp(from_rad, target_rad, smoothstep(0.0, 0.66, morph_t));

            // Soft elevation shadow fading smoothly down as the card docks into Home/Hotseat
            let sh_a = (1.0 - smoothstep(0.55, 0.96, raw_t)) * 78.0;
            if !settings_icon_return && sh_a > 2.0 {
                f.shadow(cur, rad, [0.0, 6.0], 20.0, 0.0, Color(0, 0, 0, sh_a as u8));
            }

            let app_bg = if app == AppId::Contacts {
                Color(0xF7, 0xF8, 0xFC, 255)
            } else {
                self.app_backdrop_color(app)
            };
            let icon_bg = icons::app_icon_morph_bg(app, themed_icon, &p);
            let bg_mix = if settings_icon_return {
                smoothstep(0.40, 0.76, morph_t)
            } else {
                smoothstep(0.14, 0.44, raw_t)
            };
            f.paint_rect(cur, rad, app_bg.mix(icon_bg, bg_mix));

            // Live app UI scales inside and fades COMPLETELY OUT by t=0.46 (zero square corner bleed!)
            let app_alpha = 1.0 - if settings_icon_return {
                smoothstep(0.38, 0.75, morph_t)
            } else {
                smoothstep(0.12, 0.46, raw_t)
            };
            let snapshot = if settings_icon_return {
                self.settings_return_snapshot.as_ref()
            } else {
                None
            };
            if let Some(image) = snapshot {
                if app_alpha > 0.01 {
                    f.paint_tinted_image(
                        cur, image, ImageFit::Cover, rad,
                        Color(255, 255, 255, 255).opacity(app_alpha),
                    );
                }
            } else if app_alpha > 0.01 && cur.width > 12.0 && cur.height > 12.0 {
                let sx = (cur.width / phone.width).clamp(0.05, 1.0);
                let sy = (cur.height / phone.height).clamp(0.05, 1.0);
                f.push_transform(
                    [phone.x, phone.y],
                    [cur.x - phone.x, cur.y - phone.y],
                    [sx, sy],
                    0.0,
                );
                if app_alpha < 0.995 {
                    f.push_layer(phone, app_alpha);
                }
                let mut paint_content = |f: &mut Frame<'_>| {
                    self.paint_app_backdrop(f, app, phone);
                    let _ = self.render_live_app(
                        f,
                        app,
                        app_rect,
                        APP_ID_BASE + ((app as u64) << 28),
                        None,
                        gmail_text,
                    );
                };
                if settings_icon_return {
                    // Keep the rectangular app content inside the rounded moving card.
                    let inset_x = rad * 0.30 / sx;
                    let inset_y = rad * 0.30 / sy;
                    f.within(Rect {
                        x: phone.x + inset_x,
                        y: phone.y + inset_y,
                        width: phone.width - inset_x * 2.0,
                        height: phone.height - inset_y * 2.0,
                    }, paint_content);
                } else {
                    paint_content(f);
                }
                if app_alpha < 0.995 {
                    f.pop_layer();
                }
                f.pop_transform();
            }

            // Real monochrome / full-color icon silhouette fades IN cleanly (0.22 -> 0.52)
            let glyph_alpha = if settings_icon_return {
                smoothstep(0.40, 0.82, morph_t)
            } else {
                smoothstep(0.22, 0.52, raw_t)
            };
            if glyph_alpha > 0.01 {
                let gsz = cur.width.min(cur.height);
                let gr = Rect {
                    x: cur.x + (cur.width - gsz) * 0.5,
                    y: cur.y + (cur.height - gsz) * 0.5,
                    width: gsz,
                    height: gsz,
                };
                if morph_t >= 0.88 {
                    icons::paint_real_app_icon(self.clock, f, gr, app, themed_icon, &p);
                } else {
                    if glyph_alpha < 0.995 {
                        f.push_layer(gr, glyph_alpha);
                    }
                    icons::paint_real_app_morph_glyph(self.clock, f, gr, app, themed_icon, &p);
                    if glyph_alpha < 0.995 {
                        f.pop_layer();
                    }
                }
            }
        } else {
            // App not on Home/Dock (`strip-swipe-up-home-clock.png`): shrinks smoothly toward
            // bottom-center (`from_rect -> 0.24x` centered at ~68% height) while fading out!
            let alpha = 1.0 - smoothstep(0.32, 0.92, raw_t);
            if alpha > 0.01 {
                let w = lerp(from_rect.width, phone.width * 0.24, ex);
                let h = lerp(from_rect.height, phone.height * 0.24, ex);
                let cur = Rect {
                    x: lerp(from_rect.x, phone.x + (phone.width - w) * 0.5, ex),
                    y: lerp(from_rect.y, phone.y + phone.height * 0.68, ex),
                    width: w,
                    height: h,
                };
                f.push_transform(
                    [phone.x, phone.y],
                    [cur.x - phone.x, cur.y - phone.y],
                    [cur.width / phone.width, cur.height / phone.height],
                    0.0,
                );
                f.push_layer(phone, alpha);
                self.paint_app_backdrop(f, app, phone);
                let _ = self.render_live_app(
                    f,
                    app,
                    app_rect,
                    APP_ID_BASE + ((app as u64) << 28),
                    None,
                    gmail_text,
                );
                f.pop_layer();
                f.pop_transform();
            }
        }
    }

    /// Android 13 `NotificationLaunchAnimator` matching `strip-06-notif-tap.png`!
    /// The tapped notification card breaks out over the Notification Shade with 24dp corners &
    /// shadow, expands in all 4 directions (`origin -> phone`), flattens corners `24dp -> 0dp`,
    /// and fades the launched app UI cleanly in over `0.24..0.82`.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn render_notif_launch_transition(
        &mut self,
        f: &mut Frame<'_>,
        app: AppId,
        origin: Rect,
        raw_t: f32,
        phone: Rect,
        app_rect: Rect,
        _pointer: Option<[f32; 2]>,
        gmail_text: &mut String,
    ) {
        let p = self.sys.palette();
        let e = emphasized_decelerate(raw_t);
        let cur = lerp_rect(origin, phone, e, e);
        let rad = lerp(24.0, 0.0, smoothstep(0.45, 0.96, raw_t));

        let sh_a = (1.0 - smoothstep(0.72, 1.0, raw_t)) * 96.0;
        if sh_a > 2.0 {
            f.shadow(cur, rad, [0.0, 10.0], 28.0, 0.0, Color(0, 0, 0, sh_a as u8));
        }
        let card_bg = p
            .card_elevated
            .mix(self.app_backdrop_color(app), smoothstep(0.20, 0.68, raw_t));
        f.paint_rect(cur, rad, card_bg);

        let app_alpha = smoothstep(0.18, 0.78, raw_t);
        if app_alpha > 0.01 && cur.width > 12.0 && cur.height > 12.0 {
            let sx = (cur.width / phone.width).clamp(0.08, 1.0);
            let sy = (cur.height / phone.height).clamp(0.08, 1.0);
            f.push_transform(
                [phone.x, phone.y],
                [cur.x - phone.x, cur.y - phone.y],
                [sx, sy],
                0.0,
            );
            if app_alpha < 0.995 {
                f.push_layer(phone, app_alpha);
            }
            self.paint_app_backdrop(f, app, phone);
            let _ = self.render_live_app(
                f,
                app,
                app_rect,
                APP_ID_BASE + ((app as u64) << 28),
                None,
                gmail_text,
            );
            if app_alpha < 0.995 {
                f.pop_layer();
            }
            f.pop_transform();
        }
    }

    /// Authentic Android 13 / LineageOS 20 `NavigationBarEdgePanel` folding chevron
    /// matching `strip-v4-edge-back-zoom.png`: bare folding `|` <-> `<` / `>` arrow floating
    /// ~58dp above the finger, unfolding smoothly as you pull inward and folding back on cancel!
    fn render_edge_back_indicator(
        &self,
        f: &mut Frame<'_>,
        phone: Rect,
        from_left: bool,
        cur_y: f32,
        pull: f32,
        committed: bool,
    ) {
        if pull <= 1.5 {
            return;
        }
        let appear = smoothstep(0.0, 14.0, pull);
        let bend = smoothstep(4.0, 46.0, pull);
        let glide = emphasized_decelerate((pull / 72.0).clamp(0.0, 1.0));
        let inward_x = lerp(6.0, 32.0, glide) + if committed { 4.5 } else { 0.0 };
        let cy = (cur_y - 58.0).clamp(phone.y + 64.0, phone.y + phone.height - 64.0);
        let cx = if from_left {
            phone.x + inward_x
        } else {
            phone.x + phone.width - inward_x
        };
        let dir = if from_left { 1.0 } else { -1.0 };
        let half_h = lerp(9.5, 11.5, bend);
        let span_x = lerp(0.8, 10.5, bend) * dir;
        let tip = [cx + span_x * 0.5, cy];
        let top_w = [cx - span_x * 0.5, cy - half_h];
        let bot_w = [cx - span_x * 0.5, cy + half_h];

        // Adapt chevron contrast cleanly to dark vs light underlying screen
        let dark_bg = self.sys.dark_theme
            || (self.shade_anim.value < 0.25
                && matches!(
                    self.mode,
                    ShellMode::Home
                        | ShellMode::Fullscreen(AppId::Clock | AppId::Camera | AppId::Recorder)
                )
                && self.drawer_anim.value < 0.45
                && self.widgets_anim.value < 0.45
                && self.launcher_sub == LauncherSub::None);
        let (fg, halo) = if dark_bg {
            (
                Color(0xF5, 0xF7, 0xFB, 255).opacity(appear),
                Color(0x00, 0x00, 0x00, 95).opacity(appear),
            )
        } else {
            (
                Color(0x1B, 0x1C, 0x20, 255).opacity(appear),
                Color(0xFF, 0xFF, 0xFF, 165).opacity(appear),
            )
        };
        let stroke = if committed { 3.2 } else { 2.8 };
        // Soft contrast halo behind bare folding chevron
        f.paint_line(top_w, tip, stroke + 2.4, halo);
        f.paint_line(bot_w, tip, stroke + 2.4, halo);
        // Crisp folding chevron arms + rounded apex cap
        f.paint_line(top_w, tip, stroke, fg);
        f.paint_line(bot_w, tip, stroke, fg);
        let r = stroke * 0.52;
        f.paint_rect(
            Rect {
                x: tip[0] - r,
                y: tip[1] - r,
                width: r * 2.0,
                height: r * 2.0,
            },
            r,
            fg,
        );
    }

    fn paint_desktop_bezel(&mut self, f: &mut Frame<'_>, phone: Rect, now: f64) {
        let outer = Rect {
            x: phone.x - 10.0,
            y: phone.y - 10.0,
            width: phone.width + 20.0,
            height: phone.height + 20.0,
        };
        f.shadow(outer, 34.0, [0.0, 14.0], 36.0, 0.0, Color(0, 0, 0, 130));
        f.paint_rect(outer, 32.0, Color(0x25, 0x28, 0x31, 255));
        f.paint_rect(
            Rect {
                x: phone.x - 2.0,
                y: phone.y - 2.0,
                width: phone.width + 4.0,
                height: phone.height + 4.0,
            },
            24.0,
            Color(0x0B, 0x0C, 0x10, 255),
        );
        // Top earpiece slit
        f.paint_rect(
            Rect {
                x: phone.x + phone.width * 0.5 - 24.0,
                y: outer.y + 3.0,
                width: 48.0,
                height: 3.0,
            },
            1.5,
            Color(0x3D, 0x41, 0x4E, 255),
        );

        // Tactile physical side hardware buttons on the right bezel edge
        let bx = outer.x + outer.width;
        let hw_buttons = [
            (SHELL_ID + 10, phone.y + 150.0, 44.0, "Vol +", 4_u32),
            (SHELL_ID + 11, phone.y + 200.0, 44.0, "Vol −", 5_u32),
            (SHELL_ID + 12, phone.y + 272.0, 56.0, "Power", 6_u32),
        ];
        for (id, by, bh, label, code) in hw_buttons {
            let btn = Rect {
                x: bx,
                y: by,
                width: 8.0,
                height: bh,
            };
            let hit = Rect {
                x: bx - 2.0,
                y: by - 2.0,
                width: 66.0,
                height: bh + 4.0,
            };
            let r = f.target(id, hit, label, true, Role::Button);
            let col = if code == 6 {
                if r.hovered || r.pressed {
                    self.sys.palette().primary
                } else {
                    Color(0x46, 0x52, 0x6B, 255)
                }
            } else if r.hovered || r.pressed {
                Color(0x6D, 0x73, 0x82, 255)
            } else {
                Color(0x38, 0x3C, 0x47, 255)
            };
            f.paint_rect(btn, 3.0, col);
            let txt_col = if r.hovered || r.pressed {
                Color(0xEC, 0xEE, 0xF5, 255)
            } else {
                Color(0x78, 0x7E, 0x8E, 255)
            };
            f.styled_text(
                Rect {
                    x: bx + 14.0,
                    y: by,
                    width: 52.0,
                    height: bh,
                },
                label,
                txt_col,
                text_style(11.0, 500, 0),
            );
            if r.clicked {
                self.system_key(code, now);
            }
        }

        // Subtle left-margin hardware/gesture helper rail ONLY when ample desktop margin exists
        if outer.x >= 140.0 {
            let rx = (outer.x - 122.0).max(16.0);
            let ry = phone.y + 42.0;
            f.styled_text(
                Rect {
                    x: rx,
                    y: ry,
                    width: 108.0,
                    height: 18.0,
                },
                "SYSTEM KEYS",
                Color(0x6E, 0x74, 0x85, 255),
                text_style(10.0, 600, 0),
            );
            let chips: &[(&str, u32)] = &[
                ("Shade / QS", 7),
                ("Home", 1),
                ("Recents", 2),
                ("Back", 3),
                ("Volume +", 4),
                ("Power", 6),
            ];
            for (i, (label, code)) in chips.iter().enumerate() {
                let cr = Rect {
                    x: rx,
                    y: ry + 26.0 + i as f32 * 38.0,
                    width: 102.0,
                    height: 30.0,
                };
                let r = f.target(SHELL_ID + 20 + i as u64, cr, label, true, Role::Button);
                f.paint_rect(
                    cr,
                    15.0,
                    if r.hovered || r.pressed {
                        Color(0x2E, 0x33, 0x42, 255)
                    } else {
                        Color(0x1E, 0x21, 0x2B, 255)
                    },
                );
                f.styled_text(
                    cr,
                    label,
                    Color(0xCB, 0xD0, 0xDF, 255),
                    text_style(12.0, 500, 1),
                );
                if r.clicked {
                    self.system_key(*code, now);
                }
            }
        }
    }

    fn render_wayland_status_bar(&mut self, f: &mut Frame<'_>, top_bar: Rect, now: f64) {
        if f.target(SHELL_ID + 1, top_bar, "Notification shade", true, Role::Button).clicked {
            self.tap_shade_header(now);
        }
        if let Some(idx) = self.heads_up_index(now) {
            self.render_heads_up_source(f, top_bar, idx, Color(242, 246, 255, 240));
        } else {
            let time_s = self.clock.hm();
            f.styled_text(
                Rect {
                    x: top_bar.x + 14.0,
                    y: top_bar.y + 5.0,
                    width: 68.0,
                    height: 20.0,
                },
                &time_s,
                Color(242, 246, 255, 240),
                text_style(12.5, 600, 0),
            );
            let active_title: Option<&str> = self
                .active_wayland_desktop
                .and_then(|idx| self.desktop_registry.apps.get(idx))
                .map(|a| a.name.as_str())
                .or_else(|| self.active_wayland_app.map(AppId::name));
            if let Some(title) = active_title {
                f.styled_text(
                    Rect {
                        x: top_bar.x + 84.0,
                        y: top_bar.y + 5.0,
                        width: top_bar.width - 168.0,
                        height: 20.0,
                    },
                    title,
                    Color(186, 198, 220, 220),
                    text_style(12.0, 500, 1),
                );
            }
        }
        let batt = format!("{}%", self.sys.battery_level);
        f.styled_text(
            Rect {
                x: top_bar.x + top_bar.width - 68.0,
                y: top_bar.y + 5.0,
                width: 54.0,
                height: 20.0,
            },
            &batt,
            Color(242, 246, 255, 240),
            text_style(12.0, 600, 2),
        );
    }

    fn render_status_bar(
        &mut self,
        f: &mut Frame<'_>,
        bar: Rect,
        light_ink: bool,
        now: f64,
    ) {
        let ink = if light_ink {
            Color(0xFF, 0xFF, 0xFF, 255)
        } else {
            Color(0x1B, 0x1B, 0x1F, 255)
        };
        let time_str = self.clock.hm();

        // Quick-tap right sound/ringer icon to open Volume panel
        let vol_hit = Rect {
            x: bar.x + bar.width - 108.0,
            y: bar.y,
            width: 30.0,
            height: bar.height,
        };
        let vol_r = f.target(SHELL_ID + 2, vol_hit, "Volume HUD", true, Role::Button);
        if vol_r.clicked {
            self.vol_open = !self.vol_open;
            self.vol_anim.set(f32::from(self.vol_open), now, 0.22, 0.18);
            self.vol_hide_at = now + 4.5;
        }

        // Main status bar tap trigger (dragging is handled continuously by process_pointer_gestures!)
        let shade_hit = Rect {
            x: bar.x,
            y: bar.y,
            width: bar.width,
            height: bar.height + 6.0,
        };
        let bar_r = f.without_pointer_input(vol_r.hovered || vol_r.pressed, |f| {
            f.target(SHELL_ID + 1, shade_hit, "Notification shade", true, Role::Button)
        });
        if bar_r.clicked {
            self.tap_shade_header(now);
        }

        // Clock (respects LineageOS Status Bar clock position!)
        let clock_rect = match self.sys.clock_pos {
            ClockPos::Left => Rect {
                x: bar.x + 18.0,
                y: bar.y,
                width: 44.0,
                height: bar.height,
            },
            ClockPos::Center => Rect {
                x: bar.x + bar.width * 0.5 - 26.0,
                y: bar.y,
                width: 52.0,
                height: bar.height,
            },
            ClockPos::Right => Rect {
                x: bar.x + bar.width - 154.0,
                y: bar.y,
                width: 44.0,
                height: bar.height,
            },
        };
        let clock_align = match self.sys.clock_pos {
            ClockPos::Left => 0,
            ClockPos::Center => 1,
            ClockPos::Right => 2,
        };
        let heads_up = self.heads_up_index(now);
        if let Some(idx) = heads_up {
            self.render_heads_up_source(f, bar, idx, ink);
        } else {
            f.styled_text(clock_rect, &time_str, ink, text_style(13.0, 600, clock_align));
        }

        if heads_up.is_none() && self.sys.clock_pos == ClockPos::Left {
            let mut ix = bar.x + 62.0;
            let unread = self
                .notifications
                .iter()
                .filter(|n| {
                    !n.snoozed
                        && !n.dismissing
                        && (!self.sys.hide_silent_in_status_bar || (!n.silent && n.urgency > 0))
                })
                .count();
            if self.sys.usb_debugging {
                paint_sys_icon(
                    f,
                    Rect {
                        x: ix,
                        y: bar.y + 8.5,
                        width: 14.0,
                        height: 14.0,
                    },
                    SysIcon::Usb,
                    ink,
                );
                ix += 18.0;
            }
            if unread > 0 {
                paint_sys_icon(
                    f,
                    Rect {
                        x: ix,
                        y: bar.y + 8.5,
                        width: 14.0,
                        height: 14.0,
                    },
                    SysIcon::Bell,
                    ink.opacity(0.92),
                );
                ix += 18.0;
            }
            if self.sys.dnd_enabled {
                paint_sys_icon(
                    f,
                    Rect {
                        x: ix,
                        y: bar.y + 8.5,
                        width: 14.0,
                        height: 14.0,
                    },
                    SysIcon::Dnd,
                    ink,
                );
            }
        }

        // Right cluster: Ringer icon, Wi-Fi / Airplane, Battery %
        let mut rx = bar.x + bar.width - 16.0;
        if self.sys.show_battery_pct {
            let pct = format!("{}%", self.sys.battery_level);
            rx -= 36.0;
            f.styled_text(
                Rect {
                    x: rx,
                    y: bar.y,
                    width: 38.0,
                    height: bar.height,
                },
                &pct,
                ink,
                text_style(13.0, 500, 2),
            );
            rx -= 6.0;
        } else {
            rx -= 18.0;
            paint_sys_icon(
                f,
                Rect {
                    x: rx,
                    y: bar.y + 8.0,
                    width: 15.0,
                    height: 15.0,
                },
                SysIcon::Battery,
                ink,
            );
            rx -= 6.0;
        }

        // Wi-Fi or Airplane
        rx -= 16.0;
        let net_icon = if self.sys.airplane_mode {
            SysIcon::Airplane
        } else if self.sys.wifi_enabled {
            SysIcon::Wifi
        } else {
            SysIcon::WifiOff
        };
        paint_sys_icon(
            f,
            Rect {
                x: rx,
                y: bar.y + 8.0,
                width: 15.0,
                height: 15.0,
            },
            net_icon,
            ink,
        );

        // Ringer mode icon
        rx -= 22.0;
        let ringer_icon = match self.sys.ringer_mode {
            RingerMode::Vibrate => SysIcon::Vibrate,
            RingerMode::Silent => SysIcon::Silent,
            RingerMode::Ring => SysIcon::Ring,
        };
        paint_sys_icon(
            f,
            Rect {
                x: rx,
                y: bar.y + 8.0,
                width: 15.0,
                height: 15.0,
            },
            ringer_icon,
            ink,
        );
    }

    fn render_nav_bar(
        &mut self,
        f: &mut Frame<'_>,
        bar: Rect,
        light_ink: bool,
        now: f64,
    ) {
        let ink = if light_ink {
            Color(0xFF, 0xFF, 0xFF, 225)
        } else {
            Color(0x1B, 0x1B, 0x1F, 210)
        };
        match self.sys.nav_mode {
            NavMode::Gestural => {
                // Subtle left-back & right-recents tap zones + center gesture drag pill
                let left_zone = Rect {
                    x: bar.x,
                    y: bar.y - 4.0,
                    width: bar.width * 0.26,
                    height: bar.height + 4.0,
                };
                let right_zone = Rect {
                    x: bar.x + bar.width * 0.74,
                    y: bar.y - 4.0,
                    width: bar.width * 0.26,
                    height: bar.height + 4.0,
                };
                let center_zone = Rect {
                    x: bar.x + bar.width * 0.26,
                    y: bar.y - 6.0,
                    width: bar.width * 0.48,
                    height: bar.height + 6.0,
                };

                let r_back = f.target(SHELL_ID + 30, left_zone, "Back gesture", true, Role::Button);
                if r_back.hovered {
                    f.styled_text(
                        left_zone,
                        "◁",
                        ink.opacity(0.65),
                        text_style(12.0, 500, 1),
                    );
                }
                if r_back.clicked {
                    self.go_back(now);
                }

                let r_rec =
                    f.target(SHELL_ID + 32, right_zone, "Overview gesture", true, Role::Button);
                if r_rec.hovered {
                    f.styled_text(
                        right_zone,
                        "□",
                        ink.opacity(0.65),
                        text_style(12.0, 500, 1),
                    );
                }
                if r_rec.clicked {
                    if self.mode == ShellMode::Overview {
                        self.go_home(now);
                    } else {
                        self.open_overview(now);
                    }
                }

                let r_home = f.target(SHELL_ID + 31, center_zone, "Home gesture", true, Role::Button);
                if r_home.clicked {
                    if self.mode == ShellMode::Home
                        && !self.drawer_open
                        && !self.widgets_open
                        && self.launcher_sub == LauncherSub::None
                        && self.freeform.is_none()
                    {
                        self.open_overview(now);
                    } else {
                        self.go_home(now);
                    }
                }

                let (pill_dx, pill_dy, nav_active) = match self.gesture {
                    ActiveGesture::NavGesture { delta, .. } => (
                        (delta[0] * 0.25).clamp(-36.0, 36.0),
                        (delta[1] * 0.20).clamp(-16.0, 2.0),
                        true,
                    ),
                    _ => (0.0, 0.0, false),
                };
                let pill_w = 108.0;
                let pill_h = 4.0;
                f.paint_rect(
                    Rect {
                        x: bar.x + (bar.width - pill_w) * 0.5 + pill_dx,
                        y: bar.y + (bar.height - pill_h) * 0.5 + pill_dy,
                        width: pill_w,
                        height: pill_h,
                    },
                    2.0,
                    ink.opacity(if nav_active || r_home.hovered || r_home.pressed {
                        0.96
                    } else {
                        0.72
                    }),
                );
            }
            NavMode::ThreeButton => {
                let bg = if light_ink {
                    Color(0x0A, 0x0C, 0x10, 150)
                } else {
                    self.sys.palette().card_elevated.opacity(0.96)
                };
                f.paint_rect(bar, 0.0, bg);
                let col_w = bar.width / 3.0;
                let btns = [
                    (SHELL_ID + 33, "Back", 0_usize),
                    (SHELL_ID + 34, "Home", 1_usize),
                    (SHELL_ID + 35, "Overview", 2_usize),
                ];
                for (id, label, idx) in btns {
                    let r_rect = Rect {
                        x: bar.x + idx as f32 * col_w,
                        y: bar.y,
                        width: col_w,
                        height: bar.height,
                    };
                    let r = f.target(id, r_rect, label, true, Role::Button);
                    f.state_layer(r, 20.0, ink);
                    let cx = r_rect.x + r_rect.width * 0.5;
                    let cy = r_rect.y + r_rect.height * 0.5;
                    match idx {
                        0 => {
                            // Classic left triangle
                            f.paint_line([cx + 5.0, cy - 7.0], [cx - 6.0, cy], 2.0, ink);
                            f.paint_line([cx - 6.0, cy], [cx + 5.0, cy + 7.0], 2.0, ink);
                            f.paint_line([cx + 5.0, cy - 7.0], [cx + 5.0, cy + 7.0], 2.0, ink);
                        }
                        1 => {
                            // Classic hollow circle
                            f.paint_arc(
                                Rect {
                                    x: cx - 7.5,
                                    y: cy - 7.5,
                                    width: 15.0,
                                    height: 15.0,
                                },
                                0.0,
                                std::f32::consts::TAU,
                                2.0,
                                ink,
                            );
                        }
                        _ => {
                            // Classic rounded square
                            f.paint_line([cx - 6.5, cy - 6.5], [cx + 6.5, cy - 6.5], 2.0, ink);
                            f.paint_line([cx + 6.5, cy - 6.5], [cx + 6.5, cy + 6.5], 2.0, ink);
                            f.paint_line([cx + 6.5, cy + 6.5], [cx - 6.5, cy + 6.5], 2.0, ink);
                            f.paint_line([cx - 6.5, cy + 6.5], [cx - 6.5, cy - 6.5], 2.0, ink);
                        }
                    }
                    if r.clicked {
                        match idx {
                            0 => self.go_back(now),
                            1 => self.go_home(now),
                            _ => {
                                if self.mode == ShellMode::Overview {
                                    self.go_home(now);
                                } else {
                                    self.open_overview(now);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // =========================================================================
    // SUBSYSTEM 1: TREBUCHET LAUNCHER (Home, Popups, App Drawer, Widgets, Style)
    // =========================================================================

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn render_home_layer(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
        pointer: Option<[f32; 2]>,
        now: f64,
        gmail_text: &mut String,
        hidden_app: Option<AppId>,
    ) {
        let p = self.sys.palette();

        // Wallpaper always forms the base of Layer 1
        self.wallpapers.home.paint(f, phone, self.sys.wallpaper_idx);

        let sub_blocks = self.drawer_open
            || self.drawer_anim.value > 0.05
            || self.widgets_open
            || self.widgets_anim.value > 0.05
            || self.launcher_sub != LauncherSub::None
            || self.freeform.is_some();
        let popup_blocks = self.home_popup != HomePopup::None;

        // Subtle Android Home workspace scale-back & fade as App Drawer glides up in slow-mo!
        let drawer_p = self.drawer_anim.value.clamp(0.0, 1.0);
        let empty_overview_p = if self.tasks.is_empty() {
            self.overview_anim.value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let wake_remaining = self.lockscreen.wake_reveal_remaining();
        let home_scale = lerp(1.0, 0.92, drawer_p.max(empty_overview_p)) * (1.0 - 0.025 * wake_remaining);
        let (settings_return_alpha, settings_return_y, settings_label_alpha) = match self.app_transition {
            AppTransition::Closing { app: AppId::Settings, target_icon: Some(_), start, duration, .. } => {
                let t = self.preview_transition_t.unwrap_or(((now - start) / duration) as f32);
                let p = ((t - 0.09) / 0.40).clamp(0.0, 1.0);
                let travel = -22.0 * (1.0 - p * p * (3.0 - 2.0 * p));
                (smoothstep(0.09, 0.49, t), travel + motion::settle_bounce(t, 0.46, 1.2), 1.0)
            }
            _ if matches!(self.mode, ShellMode::Fullscreen(AppId::Settings)) => {
                if let ActiveGesture::NavGesture { from_app: true, delta, hold_time, .. } = self.gesture {
                    let label = smoothstep(18.0, 64.0, -delta[1])
                        * (1.0 - smoothstep(0.02, 0.12, hold_time as f32));
                    (0.0, 0.0, label)
                } else {
                    (1.0, 0.0, 0.0)
                }
            }
            _ => (1.0, 0.0, 0.0),
        };
        let home_alpha = (1.0 - smoothstep(0.08, 0.72, drawer_p))
            * settings_return_alpha * self.lockscreen.home_reveal_alpha();
        let translate_y = -drawer_p * 18.0 + settings_return_y + 6.0 * wake_remaining;
        let transform_home = home_scale < 0.999 || translate_y.abs() > 0.01;

        self.last_home_icon_rects.clear();
        self.last_dock_icon_rects.clear();

        if home_alpha > 0.01 {
            if transform_home {
                f.push_transform(
                    [phone.x + phone.width * 0.5, phone.y + phone.height * 0.5],
                    [0.0, translate_y],
                    [home_scale, home_scale],
                    0.0,
                );
            }
            if home_alpha < 0.999 {
                f.push_layer(phone, home_alpha);
            }
            f.without_input(sub_blocks || popup_blocks, |f| {
                let ws_top = phone.y + status_h + 16.0;
                let dock_h = 84.0;
                let dock_y = phone.y + phone.height - nav_h - dock_h - 4.0;
                let ws_rect = Rect {
                    x: phone.x + 8.0,
                    y: ws_top,
                    width: phone.width - 16.0,
                    height: (dock_y - ws_top - 10.0).max(160.0),
                };
                self.last_ws_rect = ws_rect;
                self.last_ws_widget_rects.clear();

                let (cols, rows) = self.sys.grid_dims();
                let cell_w = ws_rect.width / cols as f32;
                let cell_h = ws_rect.height / rows as f32;

                // Optional added widgets from Widgets sheet!
                if self.sys.show_clock_widget {
                    let wr = Rect {
                        x: ws_rect.x + 14.0,
                        y: ws_rect.y + cell_h * 1.15,
                        width: ws_rect.width - 28.0,
                        height: 98.0,
                    };
                    self.last_ws_widget_rects.push(wr);
                    let rw = f.target(LAUNCHER_ID + 80, wr, "Clock widget", true, Role::Button);
                    f.paint_rect(wr, 24.0, Color(0x16, 0x19, 0x22, 165));
                    f.styled_text(
                        Rect {
                            x: wr.x + 20.0,
                            y: wr.y + 12.0,
                            width: wr.width - 40.0,
                            height: 48.0,
                        },
                        &self.clock.hm(),
                        Color(0xFF, 0xFF, 0xFF, 255),
                        text_style(42.0, 400, 0),
                    );
                    f.styled_text(
                        Rect {
                            x: wr.x + 22.0,
                            y: wr.y + 62.0,
                            width: wr.width - 44.0,
                            height: 22.0,
                        },
                        &format!("{} · LineageOS 20", self.clock.date()),
                        Color(0xD8, 0xE2, 0xFF, 235),
                        text_style(13.0, 500, 0),
                    );
                    if rw.clicked {
                        self.open_app(AppId::Clock, now);
                    }
                }
                if self.sys.show_weather_widget {
                    let wy = if self.sys.show_clock_widget {
                        ws_rect.y + cell_h * 1.15 + 110.0
                    } else {
                        ws_rect.y + cell_h * 1.3
                    };
                    let wr = Rect {
                        x: ws_rect.x + 14.0,
                        y: wy,
                        width: ws_rect.width - 28.0,
                        height: 76.0,
                    };
                    self.last_ws_widget_rects.push(wr);
                    let rc = f.target(LAUNCHER_ID + 81, wr, "Calendar widget", true, Role::Button);
                    let (event, time) = self.builtin_apps.calendar_upcoming()
                        .unwrap_or_else(|| ("No upcoming events".into(), "Calendar".into()));
                    f.paint_rect(wr, 22.0, Color(0xFA, 0xF9, 0xFC, 230));
                    f.styled_text(
                        Rect {
                            x: wr.x + 18.0,
                            y: wr.y + 12.0,
                            width: wr.width - 36.0,
                            height: 24.0,
                        },
                        &time,
                        p.primary,
                        text_style(12.0, 600, 0),
                    );
                    f.styled_text(
                        Rect {
                            x: wr.x + 18.0,
                            y: wr.y + 38.0,
                            width: wr.width - 36.0,
                            height: 24.0,
                        },
                        &event,
                        Color(0x1B, 0x1B, 0x1F, 255),
                        text_style(14.0, 500, 0),
                    );
                    if rc.clicked {
                        self.open_app(AppId::Calendar, now);
                    }
                }

                // Preview the exact cell that will receive the shortcut on release.
                let dragging_icon = match self.gesture {
                    ActiveGesture::HomeIconDrag { ws_idx, pos, .. } => {
                        let existing = ws_idx.or_else(|| {
                            let (app, _) = self.pressed_drawer_icon.as_ref()?;
                            self.home_shortcut_index(app)
                        });
                        if let Some((column, row)) = self.home_drop_slot(pos, existing, ws_idx.is_none()) {
                            let drop_cell = Rect {
                                x: ws_rect.x + column as f32 * cell_w,
                                y: ws_rect.y + row as f32 * cell_h,
                                width: cell_w,
                                height: cell_h,
                            }.inset(6.0);
                            f.paint_rect(drop_cell, 18.0, Color(255, 255, 255, 110));
                            f.paint_rect(drop_cell.inset(2.0), 16.0, Color(255, 255, 255, 36));
                        }
                        ws_idx.map(|index| (index, pos))
                    }
                    _ => None,
                };

                // Render mutable Workspace icons across all Home pages + horizontal edge-stretch overscroll!
                let icon_size = (cell_w * 0.72).clamp(44.0, 56.0);
                let ws_snapshot = self.home_icons.clone();
                let pages_snapshot = self.home_icon_pages.clone();
                let page_f = self.home_page_anim.value;
                let hs_dist = self.home_stretch.distance();
                let (hs_s, hs_sign) = if hs_dist.abs() > 0.0005 {
                    (1.0 + hs_dist.abs() * 0.14, hs_dist.signum())
                } else {
                    (1.0, 0.0)
                };
                // Android 13 Launcher3 SPRING_LOADED reorder mode (1:1 with `strip-home-reorder.png`!)
                let re_p = self.home_reorder_anim.value.clamp(0.0, 1.0);
                if re_p > 0.005 {
                    let bar_y = phone.y + status_h + 10.0 - 12.0 * (1.0 - re_p);
                    let pill_w = 112.0;
                    let pill_h = 34.0;
                    let pill_gap = 16.0;
                    let total_bar_w = pill_w * 2.0 + pill_gap;
                    let bar_x = phone.x + (phone.width - total_bar_w) * 0.5;
                    let can_uninstall = dragging_icon.is_some_and(|(idx, _)| matches!(ws_snapshot[idx].2, HomeApp::Builtin(_)));
                    let rem_x = if can_uninstall { bar_x } else { phone.x + (phone.width - pill_w) * 0.5 };
                    let rem_r = Rect { x: rem_x, y: bar_y, width: pill_w, height: pill_h };
                    let uni_r = Rect { x: bar_x + pill_w + pill_gap, y: bar_y, width: pill_w, height: pill_h };
                    let (over_rem, over_uni) = if let Some((_, pos)) = dragging_icon {
                        (
                            pos[1] <= phone.y + 84.0 && (!can_uninstall || pos[0] < phone.x + phone.width * 0.5),
                            pos[1] <= phone.y + 84.0 && pos[0] >= phone.x + phone.width * 0.5,
                        )
                    } else {
                        (false, false)
                    };

                    // Compact outlined Remove / Uninstall pills
                    let actions = [(rem_r, "×  Remove", over_rem), (uni_r, "Uninstall", over_uni)];
                    for (r, label, over) in actions.into_iter().take(if can_uninstall { 2 } else { 1 }) {
                        if over {
                            f.paint_rect(r, 17.0, Color(0xDC, 0x36, 0x2E, (225.0 * re_p) as u8));
                        } else {
                            // Crisp 1.5px outline pill with transparent/subtle dark scrim
                            f.paint_rect(r, 17.0, Color(255, 255, 255, (190.0 * re_p) as u8));
                            f.paint_rect(r.inset(1.5), 15.5, Color(0x16, 0x18, 0x1E, (145.0 * re_p) as u8));
                        }
                        f.styled_text(r, label, Color(255, 255, 255, (245.0 * re_p) as u8), text_style(12.5, 500, 1));
                    }

                    // Zoomed workspace page outline card + peeking next page card on right edge
                    let target_card_r = Rect {
                        x: phone.x + 28.0,
                        y: (phone.y + status_h + 10.0) + pill_h + 18.0,
                        width: phone.width - 56.0,
                        height: (ws_rect.y + ws_rect.height - ((phone.y + status_h + 10.0) + pill_h + 26.0)).max(200.0),
                    };
                    let card_r = Rect {
                        x: ws_rect.x + (target_card_r.x - ws_rect.x) * re_p,
                        y: ws_rect.y + (target_card_r.y - ws_rect.y) * re_p,
                        width: ws_rect.width + (target_card_r.width - ws_rect.width) * re_p,
                        height: ws_rect.height + (target_card_r.height - ws_rect.height) * re_p,
                    };
                    f.paint_rect(card_r, 22.0, Color(168, 192, 232, (165.0 * re_p) as u8));
                    f.paint_rect(card_r.inset(1.5), 20.5, Color(0x12, 0x15, 0x1C, (68.0 * re_p) as u8));
                    let peek_r = Rect {
                        x: card_r.x + card_r.width + 14.0 + 24.0 * (1.0 - re_p),
                        y: card_r.y,
                        width: card_r.width,
                        height: card_r.height,
                    };
                    f.paint_rect(peek_r, 22.0, Color(168, 192, 232, (115.0 * re_p) as u8));
                    f.paint_rect(peek_r.inset(1.5), 20.5, Color(0x12, 0x15, 0x1C, (42.0 * re_p) as u8));
                    // Smoothly scale the workspace icons into `target_card_r` (0.84x, matching `strip-home-reorder.png`)!
                    let sx = target_card_r.width / ws_rect.width.max(1.0);
                    let sy = target_card_r.height / ws_rect.height.max(1.0);
                    let ws_s = sx.min(sy).clamp(0.76, 0.88);
                    let tx = (target_card_r.x + target_card_r.width * 0.5) - (ws_rect.x + ws_rect.width * 0.5);
                    let ty = (target_card_r.y + target_card_r.height * 0.5) - (ws_rect.y + ws_rect.height * 0.5);
                    let cur_s = 1.0 + (ws_s - 1.0) * re_p;
                    f.push_transform(
                        [ws_rect.x + ws_rect.width * 0.5, ws_rect.y + ws_rect.height * 0.5],
                        [tx * re_p, ty * re_p],
                        [cur_s, cur_s],
                        0.0,
                    );
                }
                f.with_edge_stretch(ws_rect, true, hs_s, hs_sign, |f| {
                for (ws_idx, &(raw_c, raw_r, ref app)) in ws_snapshot.iter().enumerate() {
                    let entry = match app {
                        HomeApp::Builtin(app) if !self.is_app_hidden(*app) => TaskEntry::Single(*app),
                        HomeApp::Desktop(id) => {
                            let Some(index) = self.desktop_shortcut_index(id) else { continue };
                            TaskEntry::Desktop(index)
                        }
                        _ => continue,
                    };
                    let builtin = match entry {
                        TaskEntry::Single(app) => Some(app),
                        _ => None,
                    };
                    let name = match entry {
                        TaskEntry::Single(app) => std::borrow::Cow::Borrowed(app.name()),
                        TaskEntry::Desktop(index) => std::borrow::Cow::Owned(self.desktop_registry.apps[index].name.clone()),
                        _ => unreachable!(),
                    };
                    let icon_hidden = builtin.is_some() && hidden_app == builtin;
                    let icon_page = pages_snapshot.get(ws_idx).copied().unwrap_or(0);
                    let page_dx = (icon_page as f32 - page_f) * phone.width;
                    let c = raw_c.min(cols.saturating_sub(1));
                    let r = raw_r.min(rows.saturating_sub(1));
                    let cell = Rect {
                        x: ws_rect.x + c as f32 * cell_w + page_dx,
                        y: ws_rect.y + r as f32 * cell_h,
                        width: cell_w,
                        height: cell_h,
                    };
                    if cell.x + cell.width < phone.x - 16.0 || cell.x > phone.x + phone.width + 16.0 {
                        continue;
                    }
                    let ir = Rect {
                        x: cell.x + (cell.width - icon_size) * 0.5,
                        y: cell.y + (cell.height - icon_size - 20.0).max(4.0) * 0.5,
                        width: icon_size,
                        height: icon_size,
                    };
                    if icon_page == self.home_page {
                        let center = [ir.x + ir.width * 0.5, ir.y + ir.height * 0.5];
                        self.last_home_icon_rects.push((ws_idx, cell, center));
                    }
                    if dragging_icon.is_some_and(|(index, _)| index == ws_idx) {
                        continue;
                    }
                    let resp = f.target(
                        LAUNCHER_ID + 0x2000 + ws_idx as u64,
                        cell,
                        name.as_ref(),
                        true,
                        Role::Button,
                    );
                    let scale = if resp.pressed { 0.92 } else { 1.0 };
                    let drawn_ir = ir.inset((1.0 - scale) * icon_size * 0.5);
                    if !icon_hidden {
                        self.paint_home_shortcut(f, drawn_ir, app, &p);

                        // Notification dot badge when enabled
                        if self.sys.notification_dots
                            && matches!(builtin, Some(AppId::ArcaneChat | AppId::FDroid))
                            && self
                                .notifications
                                .iter()
                                .any(|n| n.target_app == builtin && !n.snoozed)
                        {
                            f.paint_rect(
                                Rect {
                                    x: drawn_ir.x + drawn_ir.width - 11.0,
                                    y: drawn_ir.y + 1.0,
                                    width: 10.0,
                                    height: 10.0,
                                },
                                5.0,
                                p.primary_container,
                            );
                        }
                    }

                    if self.sys.show_home_labels && !icon_hidden
                        && !(builtin == Some(AppId::Settings) && settings_label_alpha > 0.0) {
                        let lbl = Rect {
                            x: cell.x - 4.0,
                            y: ir.y + icon_size + 5.0,
                            width: cell.width + 8.0,
                            height: 18.0,
                        };
                        f.styled_text(
                            Rect {
                                y: lbl.y + 1.0,
                                ..lbl
                            },
                            name.as_ref(),
                            Color(0, 0, 0, 155),
                            wallpaper::customize_font(text_style(11.5, 500, 1), self.sys.font_style),
                        );
                        f.styled_text(
                            lbl,
                            name.as_ref(),
                            Color(0xFF, 0xFF, 0xFF, 250),
                            wallpaper::customize_font(text_style(11.5, 500, 1), self.sys.font_style),
                        );
                    }

                    if resp.clicked {
                        match entry {
                            TaskEntry::Single(app) => {
                                self.open_app_from(app, Some(drawn_ir), true, false, self.sys.themed_icons, now);
                            }
                            TaskEntry::Desktop(index) => {
                                self.open_desktop_app_from(index, None, Some(drawn_ir), false, now);
                            }
                            _ => {}
                        }
                    }
                }
                });
                if re_p > 0.005 {
                    f.pop_transform();
                }
                if self.home_page_count > 1 {
                    let dot_bar = Rect {
                        x: phone.x,
                        y: ws_rect.y + ws_rect.height - 9.0,
                        width: phone.width,
                        height: 12.0,
                    };
                    f.page_indicator(
                        0,
                        dot_bar,
                        self.home_page_count,
                        page_f,
                        PageIndicatorOptions {
                            dot_size: 6.0,
                            gap: 12.0,
                            active_color: Some(Color(255, 255, 255, 235)),
                            inactive_color: Some(Color(255, 255, 255, 105)),
                            interactive: false,
                        },
                    );
                }

                // Subtle App Drawer pull-up indicator pill right above the Hotseat dock (clickable!)
                let drawer_pill = Rect {
                    x: phone.x + phone.width * 0.5 - 32.0,
                    y: dock_y - 16.0,
                    width: 64.0,
                    height: 18.0,
                };
                let dp_r = f.target(LAUNCHER_ID + 2, drawer_pill, "All apps", true, Role::Button);
                let chev_c = Color(255, 255, 255, if dp_r.hovered { 235 } else { 150 });
                let cx = drawer_pill.x + drawer_pill.width * 0.5;
                let cy = drawer_pill.y + 9.0;
                f.paint_line([cx - 8.0, cy + 2.0], [cx, cy - 3.0], 2.0, chev_c);
                f.paint_line([cx, cy - 3.0], [cx + 8.0, cy + 2.0], 2.0, chev_c);
                if dp_r.clicked {
                    self.drawer_open = true;
                    self.drawer_anim.set(1.0, now, 0.32, 0.22);
                }

                // 5-Icon Hotseat Dock matching 01-home-main.png (Phone, Messaging, Contacts, Browser, Camera)
                let dock_apps = [
                    AppId::Phone,
                    AppId::Messaging,
                    AppId::Contacts,
                    AppId::Browser,
                    AppId::Camera,
                ];
                let d_cell_w = (phone.width - 16.0) / dock_apps.iter().filter(|&&a| !self.is_app_hidden(a)).count().max(1) as f32;
                let d_icon = (d_cell_w * 0.72).clamp(44.0, 56.0);
                let dock_apps: Vec<_> = dock_apps.into_iter().filter(|&a| !self.is_app_hidden(a)).collect();
                for (i, app) in dock_apps.into_iter().enumerate() {
                    let cell = Rect {
                        x: phone.x + 8.0 + i as f32 * d_cell_w,
                        y: dock_y,
                        width: d_cell_w,
                        height: dock_h,
                    };
                    self.last_dock_icon_rects.push((app, cell));
                    let ir = Rect {
                        x: cell.x + (cell.width - d_icon) * 0.5,
                        y: cell.y + (cell.height - d_icon) * 0.5,
                        width: d_icon,
                        height: d_icon,
                    };
                    let resp = f.target(
                        LAUNCHER_ID + 150 + i as u64,
                        cell,
                        app.name(),
                        true,
                        Role::Button,
                    );
                    let drawn_ir = ir.inset(if resp.pressed { 2.5 } else { 0.0 });
                    if hidden_app != Some(app) {
                        icons::paint_styled_app_icon(self.clock, f, drawn_ir, app, self.sys.themed_icons, &p, self.sys.icon_shape);
                    }
                    if resp.clicked {
                        self.open_app_from(app, Some(drawn_ir), true, false, self.sys.themed_icons, now);
                    }
                }

            });
            if home_alpha < 0.999 {
                f.pop_layer();
            }
            if transform_home {
                f.pop_transform();
            }
        }

        // Long-press popups (Blank Home Options & App Icon Shortcuts — animated enter & exit!)
        if self.home_popup != HomePopup::None
            || (self.last_home_popup != HomePopup::None && self.popup_anim.value > 0.01)
        {
            self.render_home_popup(f, phone, now);
        }

        // App Drawer Overlay (true 1:1 slow-mo slide-up)
        if self.drawer_anim.value > 0.005 || self.drawer_open {
            self.render_app_drawer(f, phone, status_h, nav_h, pointer, now);
        }

        if let ActiveGesture::HomeIconDrag { ws_idx, pos, offset } = self.gesture {
            let app = match ws_idx {
                Some(index) => self.home_icons.get(index).map(|(_, _, app)| app),
                None => self.pressed_drawer_icon.as_ref().map(|(app, _)| app),
            };
            if let Some(app) = app.cloned() {
                let (columns, _) = self.sys.grid_dims();
                let lifted = ((phone.width - 16.0) / columns as f32 * 0.72).clamp(44.0, 56.0) * 1.16;
                let rect = Rect {
                    x: pos[0] - offset[0] - lifted * 0.5,
                    y: pos[1] - offset[1] - lifted * 0.5,
                    width: lifted,
                    height: lifted,
                };
                self.paint_home_shortcut(f, rect, &app, &p);
            }
        }

        // Widgets Bottom Sheet (true 1:1 drag-to-dismiss)
        if self.widgets_anim.value > 0.005 || self.widgets_open {
            self.render_widgets_sheet(f, phone, status_h, nav_h, now);
        }

        // Keep the returning app name at its destination while the icons move.
        if settings_label_alpha > 0.0 && self.sys.show_home_labels
            && let Some(icon) = self.find_home_or_dock_icon_rect(AppId::Settings, phone, status_h, nav_h)
        {
            let label = Rect {
                x: icon.x + icon.width * 0.5 - 48.0,
                y: icon.y + icon.height + 5.0,
                width: 96.0,
                height: 18.0,
            };
            let style = text_style(11.5, 500, 1);
            f.styled_text(
                Rect { y: label.y + 1.0, ..label },
                AppId::Settings.name(),
                Color(0, 0, 0, 155).opacity(settings_label_alpha),
                style,
            );
            f.styled_text(
                label, AppId::Settings.name(),
                Color(255, 255, 255, 250).opacity(settings_label_alpha),
                style,
            );
        }

        // Wallpaper & Style / Home Settings sub-screens (smooth dual-page push/pop!)
        if self.launcher_sub != LauncherSub::None
            || (self.prev_launcher_sub != LauncherSub::None && self.launcher_sub_anim.value < 0.995)
        {
            self.render_launcher_subscreen(f, phone, status_h, nav_h, now);
        }

        // Freeform Floating Window over Home (matching 09-recents-freeform-window.png!)
        if let Some(win) = self.freeform {
            self.render_freeform_window(f, phone, status_h, nav_h, win, pointer, now, gmail_text);
        }
    }

    fn render_home_popup(&mut self, f: &mut Frame<'_>, phone: Rect, now: f64) {
        let p = self.sys.palette();
        let prog = self.popup_anim.value.clamp(0.0, 1.0);
        if prog <= 0.01 {
            return;
        }
        let closing = self.home_popup == HomePopup::None;
        let active_popup = if closing {
            self.last_home_popup
        } else {
            self.home_popup
        };
        let mut action_taken = false;
        let mut popup_box = Rect::default();

        match active_popup {
            HomePopup::BlankOptions([ax, ay]) => {
                // Exact match to LineageOS 20 `02-home-options-popup.png` & `strip-03-home-longpress.png`!
                let card_w = 216.0_f32.min(phone.width - 32.0);
                let row_h = 52.0;
                let gap = 2.0;
                let total_h = row_h * 3.0 + gap * 2.0;
                let px = (ax - card_w * 0.25)
                    .clamp(phone.x + 16.0, phone.x + phone.width - card_w - 16.0);
                let py = (ay - 12.0)
                    .clamp(phone.y + 84.0, phone.y + phone.height - total_h - 140.0);
                popup_box = Rect {
                    x: px,
                    y: py,
                    width: card_w,
                    height: total_h,
                };

                f.push_transform(
                    [px + card_w * 0.5, py],
                    [0.0, 0.0],
                    [lerp(0.95, 1.0, prog), lerp(0.84, 1.0, prog)],
                    0.0,
                );
                if prog < 0.995 {
                    f.push_layer(popup_box.inset(-18.0), prog);
                }
                let items = [
                    (SysIcon::Palette, "Wallpaper & style", 0_usize, 26.0_f32, 4.0_f32),
                    (SysIcon::Widgets, "Widgets", 1_usize, 4.0_f32, 4.0_f32),
                    (SysIcon::Settings, "Home settings", 2_usize, 4.0_f32, 26.0_f32),
                ];
                for (icon, label, idx, top_rad, bot_rad) in items {
                    let ry = py + idx as f32 * (row_h + gap);
                    let rr = Rect {
                        x: px,
                        y: ry,
                        width: card_w,
                        height: row_h,
                    };
                    let fill = match idx {
                        0 => p.card_pure,
                        1 => p.card_elevated.mix(p.primary_container, 0.24),
                        _ => p.card_elevated.mix(p.primary_container, 0.44),
                    };
                    let r = f.target(
                        LAUNCHER_ID + 210 + idx as u64,
                        rr,
                        label,
                        !closing,
                        Role::MenuItem,
                    );
                    paint_asym_rect(f, rr, top_rad, bot_rad, fill);
                    if r.hovered || r.pressed {
                        let hl_a = if r.pressed { 0.14 } else { 0.08 };
                        paint_asym_rect(f, rr, top_rad, bot_rad, p.on_surface.opacity(hl_a));
                    }
                    paint_sys_icon(
                        f,
                        Rect {
                            x: rr.x + 18.0,
                            y: rr.y + (row_h - 20.0) * 0.5,
                            width: 20.0,
                            height: 20.0,
                        },
                        icon,
                        p.on_surface,
                    );
                    f.styled_text(
                        Rect {
                            x: rr.x + 52.0,
                            y: rr.y,
                            width: rr.width - 64.0,
                            height: row_h,
                        },
                        label,
                        p.on_surface,
                        text_style(14.5, 500, 0),
                    );
                    if r.clicked {
                        action_taken = true;
                        self.last_home_popup = self.home_popup;
                        self.home_popup = HomePopup::None;
                        self.popup_anim.set(0.0, now, 0.16, 0.12);
                        match idx {
                            0 => self.set_launcher_sub(LauncherSub::WallpaperStyle, true, now),
                            1 => {
                                self.widgets_open = true;
                                self.widgets_anim.set(1.0, now, 0.30, 0.22);
                            }
                            _ => self.set_launcher_sub(LauncherSub::HomeSettings, true, now),
                        }
                    }
                }
                if prog < 0.995 {
                    f.pop_layer();
                }
                f.pop_transform();
            }
            HomePopup::AppShortcuts { app, anchor } => {
                // Exact match to 21-app-icon-popup-contacts.png!
                let card_w = 224.0_f32.min(phone.width - 28.0);
                let row_h = 52.0;
                let has_shortcut = app.shortcut_label().is_some();
                let count = if has_shortcut { 3 } else { 2 };
                let total_h = count as f32 * (row_h + 4.0);
                let px = (anchor[0] - 42.0).clamp(phone.x + 14.0, phone.x + phone.width - card_w - 14.0);
                let py = (anchor[1] - total_h - 14.0).clamp(phone.y + 44.0, phone.y + phone.height - total_h - 90.0);
                popup_box = Rect { x: px, y: py, width: card_w, height: total_h };

                f.push_transform(
                    [anchor[0], py + total_h],
                    [0.0, (1.0 - prog) * 12.0],
                    [0.88 + 0.12 * prog, 0.88 + 0.12 * prog],
                    0.0,
                );
                if prog < 0.995 {
                    f.push_layer(popup_box.inset(-18.0), prog);
                }
                for idx in 0..count {
                    let ry = py + idx as f32 * (row_h + 4.0);
                    let rr = Rect {
                        x: px,
                        y: ry,
                        width: card_w,
                        height: row_h,
                    };
                    let fill = match idx {
                        0 => p.card_pure,
                        1 => p.card_elevated.mix(p.primary_container, 0.20),
                        _ => p.card_elevated.mix(p.primary_container, 0.42),
                    };
                    f.shadow(rr, 26.0, [0.0, 4.0], 14.0, 0.0, Color(0, 0, 0, (prog * 50.0) as u8));
                    let label = match idx {
                        0 => "App info",
                        1 => "Widgets",
                        _ => app.shortcut_label().unwrap_or("Open"),
                    };
                    let r = f.target(
                        LAUNCHER_ID + 220 + idx as u64,
                        rr,
                        label,
                        !closing,
                        Role::MenuItem,
                    );
                    f.paint_rect(rr, 26.0, fill);
                    f.state_layer(r, 26.0, p.on_surface);
                    if idx == 2 {
                        // Blue + circle matching 21-app-icon-popup-contacts.png!
                        let circ = Rect {
                            x: rr.x + 12.0,
                            y: rr.y + 10.0,
                            width: 32.0,
                            height: 32.0,
                        };
                        f.paint_rect(circ, 16.0, p.card_pure);
                        f.paint_material_icon(
                            circ.inset(6.0),
                            MaterialIcon::Add,
                            Color(0x2D, 0x7A, 0xD8, 255),
                        );
                        // Trailing `=` drag grip
                        f.paint_line(
                            [rr.x + rr.width - 26.0, rr.y + 23.0],
                            [rr.x + rr.width - 15.0, rr.y + 23.0],
                            1.8,
                            p.on_surface_variant,
                        );
                        f.paint_line(
                            [rr.x + rr.width - 26.0, rr.y + 28.0],
                            [rr.x + rr.width - 15.0, rr.y + 28.0],
                            1.8,
                            p.on_surface_variant,
                        );
                    } else {
                        let ic = if idx == 0 {
                            SysIcon::Info
                        } else {
                            SysIcon::Widgets
                        };
                        paint_sys_icon(
                            f,
                            Rect {
                                x: rr.x + 18.0,
                                y: rr.y + 16.0,
                                width: 20.0,
                                height: 20.0,
                            },
                            ic,
                            p.on_surface,
                        );
                    }
                    f.styled_text(
                        Rect {
                            x: rr.x + 54.0,
                            y: rr.y,
                            width: rr.width - 84.0,
                            height: row_h,
                        },
                        label,
                        p.on_surface,
                        text_style(14.5, 500, 0),
                    );
                    if r.clicked {
                        action_taken = true;
                        self.last_home_popup = self.home_popup;
                        self.home_popup = HomePopup::None;
                        self.popup_anim.set(0.0, now, 0.16, 0.12);
                        match idx {
                            0 => {
                                self.settings_app
                                    .navigate(settings::SettingsPage::AppInfo(app.index()), now);
                                self.open_app(AppId::Settings, now);
                            }
                            1 => {
                                self.widgets_open = true;
                                self.widgets_anim.set(1.0, now, 0.30, 0.22);
                            }
                            _ => {
                                if app == AppId::Contacts {
                                    self.embedded.open_new_contact(f);
                                    self.open_app(AppId::Contacts, now);
                                } else if app == AppId::Clock {
                                    self.builtin_apps.add_quick_alarm();
                                    self.open_app(AppId::Clock, now);
                                } else if app == AppId::Settings {
                                    self.settings_app
                                        .navigate(settings::SettingsPage::Battery, now);
                                    self.open_app(AppId::Settings, now);
                                } else {
                                    self.open_app(app, now);
                                }
                            }
                        }
                    }
                }
                // Little downward arrow tip pointing at icon
                let tip_x = anchor[0].clamp(px + 24.0, px + card_w - 24.0);
                let tip_y = py + total_h - 4.0;
                f.paint_rect(
                    Rect {
                        x: tip_x - 5.0,
                        y: tip_y,
                        width: 10.0,
                        height: 8.0,
                    },
                    3.0,
                    p.card_elevated.mix(p.primary_container, 0.42),
                );
                if prog < 0.995 {
                    f.pop_layer();
                }
                f.pop_transform();
            }
            HomePopup::None => {}
        }

        if !closing {
            let over_popup = f.pointer_in(popup_box);
            let dismiss = f.without_pointer_input(over_popup, |f| {
                f.target(LAUNCHER_ID + 200, phone, "Dismiss popup", true, Role::Button)
            });
            if dismiss.clicked && !action_taken {
                self.last_home_popup = self.home_popup;
                self.home_popup = HomePopup::None;
                self.popup_anim.set(0.0, now, 0.16, 0.12);
            }
        }
    }

    fn render_app_drawer(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
        pointer: Option<[f32; 2]>,
        now: f64,
    ) {
        let p = self.sys.palette();
        let prog = self.drawer_anim.value.clamp(0.0, 1.0);
        let sheet_dy = (1.0 - prog) * phone.height * 0.28;
        self.last_drawer_icon_rects.clear();
        let top_rad = lerp(24.0, 0.0, smoothstep(0.65, 1.0, prog));
        let bg_alpha = smoothstep(0.0, 0.35, prog);
        let content_alpha = smoothstep(0.18, 0.62, prog);

        // Fast full-screen surface scrim matching `strip-home-app-drawer.png`
        if bg_alpha > 0.005 {
            f.paint_rect(phone, 0.0, p.settings_bg.opacity(bg_alpha));
        }

        f.push_transform([phone.x, phone.y], [0.0, sheet_dy], [1.0, 1.0], 0.0);
        if content_alpha < 0.999 {
            f.push_layer(phone, content_alpha);
        }
        f.paint_rect(phone, top_rad, p.settings_bg);

        // Top search bar pill (matches 17-app-drawer.png)
        let search_r = Rect {
            x: phone.x + 14.0,
            y: phone.y + status_h + 12.0,
            width: phone.width - 28.0,
            height: 48.0,
        };
        f.paint_rect(search_r, 24.0, p.surface_container);
        paint_sys_icon(
            f,
            Rect {
                x: search_r.x + 16.0,
                y: search_r.y + 14.0,
                width: 20.0,
                height: 20.0,
            },
            SysIcon::Search,
            p.on_surface,
        );
        let field_rect = Rect {
            x: search_r.x + 48.0,
            y: search_r.y + 2.0,
            width: search_r.width - 88.0,
            height: 44.0,
        };
        let sf = f.target(ID_DRAWER_SEARCH, field_rect, "Search apps", true, Role::TextField);
        let search_style = text_style(16.0, 400, 0);
        if sf.clicked || (sf.pressed && self.ime_target == Some(ID_DRAWER_SEARCH)) {
            self.ime_target = Some(ID_DRAWER_SEARCH);
            self.last_ime_field_rect = search_r;
            self.ime_anim.set(1.0, now, 0.24, 0.20);
            if let Some(pt) = pointer {
                let rel_x = (pt[0] - field_rect.x).max(0.0);
                self.ime_cursor = cursor_index_for_x(f, &self.drawer_search, search_style, rel_x);
            } else {
                self.ime_cursor = self.drawer_search.chars().count();
            }
        }
        if self.ime_target == Some(ID_DRAWER_SEARCH) {
            self.last_ime_field_rect = search_r;
            let max_c = self.drawer_search.chars().count();
            if self.ime_cursor > max_c {
                self.ime_cursor = max_c;
            }
        }
        let display_q = if self.drawer_search.is_empty() {
            "Search apps"
        } else {
            self.drawer_search.as_str()
        };
        f.styled_text(
            field_rect,
            display_q,
            if self.drawer_search.is_empty() {
                p.on_surface_variant
            } else {
                p.on_surface
            },
            search_style,
        );
        if self.ime_target == Some(ID_DRAWER_SEARCH) {
            f.continuous(phone);
            let prefix: String = self.drawer_search.chars().take(self.ime_cursor).collect();
            let caret_x = field_rect.x + f.text_width(&prefix, search_style).min(field_rect.width - 4.0) + 1.0;
            if sf.pressed || ((now * 2.0) as u64).is_multiple_of(2) {
                f.paint_rect(
                    Rect {
                        x: caret_x,
                        y: field_rect.y + 11.0,
                        width: 2.0,
                        height: 22.0,
                    },
                    1.0,
                    p.primary,
                );
            }
            // Android Editor.InsertionHandleView teardrop handle below cursor when dragging or text is non-empty
            if !self.drawer_search.is_empty() || sf.pressed {
                let hr = 7.0;
                f.paint_rect(
                    Rect {
                        x: caret_x + 1.0 - hr,
                        y: field_rect.y + 33.0,
                        width: hr * 2.0,
                        height: hr * 2.0,
                    },
                    hr,
                    p.primary,
                );
            }
        }
        if !self.drawer_search.is_empty() {
            let clr = Rect {
                x: search_r.x + search_r.width - 40.0,
                y: search_r.y + 8.0,
                width: 32.0,
                height: 32.0,
            };
            let rc = f.target(LAUNCHER_ID + 301, clr, "Clear search", true, Role::Button);
            paint_sys_icon(f, clr.inset(6.0), SysIcon::Close, p.on_surface_variant);
            if rc.clicked {
                self.drawer_search.clear();
            }
        }

        // Scrollable 5-column app grid
        let grid_area = Rect {
            x: phone.x + 10.0,
            y: search_r.y + search_r.height + 16.0,
            width: phone.width - 20.0,
            height: (phone.y + phone.height - nav_h - (search_r.y + search_r.height + 24.0))
                .max(120.0),
        };
        let q = self.drawer_search.trim().to_ascii_lowercase();

        #[derive(Clone)]
        enum DrawerItem {
            Builtin(AppId),
            Desktop {
                idx: usize,
                id: String,
                name: String,
                icon_name: Option<String>,
            },
        }

        // Unified alphabetical App Drawer: real Freedesktop `.desktop` apps (with XDG Icon Theme
        // PNG/SVG icons) alongside Wisp built-in apps (avoiding duplicates when a built-in app
        // maps to a discovered `.desktop` entry).
        let phone_backend = self.desktop_registry.find_for_builtin(AppId::Phone);
        let messages_backend = self.desktop_registry.find_for_builtin(AppId::Messaging);
        let mut drawer_items: Vec<DrawerItem> = Vec::new();
        for &a in AppId::ALL.iter() {
            if self.is_app_hidden(a) {
                continue;
            }
            // Phone and Messages share their launcher identities with the dock.
            let backend = match a {
                AppId::Phone => phone_backend,
                AppId::Messaging => messages_backend,
                AppId::Settings | AppId::Files | AppId::Calendar => None,
                _ if self.desktop_registry.find_for_builtin(a).is_some() => continue,
                _ => None,
            };
            if q.is_empty() || a.name().to_ascii_lowercase().contains(&q)
                || backend.is_some_and(|entry| entry.matches_query(&q))
            {
                drawer_items.push(DrawerItem::Builtin(a));
            }
        }
        let other_files_id = self.desktop_registry.find_for_builtin(AppId::Files).map(|app| app.id.as_str());
        for (idx, dapp) in self.desktop_registry.apps.iter().enumerate() {
            // Fingerprint Lab is a hardware diagnostic, separate from launcher apps.
            if ![phone_backend, messages_backend].into_iter().flatten().any(|entry| entry.id == dapp.id)
                && Some(dapp.id.as_str()) != other_files_id
                && dapp.id != "org.spectreos.Files.desktop"
                && dapp.id != "org.spectre.Fingerprint"
                && dapp.id != "org.spectre.Fingerprint.desktop"
                && dapp.matches_query(&q)
            {
                drawer_items.push(DrawerItem::Desktop {
                    idx,
                    id: dapp.id.clone(),
                    name: dapp.name.clone(),
                    icon_name: dapp.icon.clone(),
                });
            }
        }
        drawer_items.sort_by(|a, b| {
            let na = match a {
                DrawerItem::Builtin(app) => app.name(),
                DrawerItem::Desktop { name, .. } => name.as_str(),
            };
            let nb = match b {
                DrawerItem::Builtin(app) => app.name(),
                DrawerItem::Desktop { name, .. } => name.as_str(),
            };
            na.to_ascii_lowercase().cmp(&nb.to_ascii_lowercase())
        });

        let cols = 5_usize;
        let cell_w = grid_area.width / cols as f32;
        let cell_h = 98.0;
        let rows = drawer_items.len().div_ceil(cols);
        let content_h = (rows as f32 * cell_h + 32.0).max(grid_area.height);
        let max_s = (content_h - grid_area.height).max(0.0);
        self.last_scroll_max[0] = max_s;
        self.drawer_scroll = self.drawer_scroll.clamp(0.0, max_s);
        f.scroll_viewport(
            LAUNCHER_ID + 305,
            grid_area,
            &mut self.drawer_scroll,
            max_s,
        );

        let mut clicked_builtin: Option<(AppId, Rect)> = None;
        let mut clicked_desktop: Option<(usize, Rect)> = None;

        let (dr_s, dr_sign) = self.target_stretch(ScrollTarget::Drawer);
        f.within(grid_area, |f| {
            f.with_edge_stretch(grid_area, false, dr_s, dr_sign, |f| {
            if drawer_items.is_empty() {
                f.styled_text(
                    Rect {
                        x: grid_area.x,
                        y: grid_area.y + 64.0,
                        width: grid_area.width,
                        height: 28.0,
                    },
                    "No results found",
                    p.on_surface_variant,
                    text_style(16.0, 500, 1),
                );
                f.styled_text(
                    Rect {
                        x: grid_area.x,
                        y: grid_area.y + 96.0,
                        width: grid_area.width,
                        height: 22.0,
                    },
                    "Try searching for Clock or Settings",
                    p.on_surface_variant.opacity(0.8),
                    text_style(13.0, 400, 1),
                );
            } else {
                let icon_sz = (cell_w * 0.68).clamp(42.0, 52.0);
                for (i, item) in drawer_items.iter().enumerate() {
                    let c = i % cols;
                    let r = i / cols;
                    let cell = Rect {
                        x: grid_area.x + c as f32 * cell_w,
                        y: grid_area.y + r as f32 * cell_h - self.drawer_scroll,
                        width: cell_w,
                        height: cell_h,
                    };
                    if cell.y + cell.height < grid_area.y - 20.0
                        || cell.y > grid_area.y + grid_area.height + 20.0
                    {
                        continue;
                    }
                    let ir = Rect {
                        x: cell.x + (cell.width - icon_sz) * 0.5,
                        y: cell.y + 8.0,
                        width: icon_sz,
                        height: icon_sz,
                    };
                    let app_key = match item {
                        DrawerItem::Builtin(app) => HomeApp::Builtin(*app),
                        DrawerItem::Desktop { id, .. } => HomeApp::Desktop(id.clone()),
                    };
                    let mut hit = cell.intersect(grid_area);
                    hit.y += sheet_dy;
                    hit = hit.intersect(phone);
                    let center = [ir.x + ir.width * 0.5, ir.y + ir.height * 0.5 + sheet_dy];
                    let held = self.long_press_fired && self.pressed_drawer_icon.as_ref().is_some_and(|(app, _)| app == &app_key);
                    if hit.width > 0.0 && hit.height > 0.0 {
                        self.last_drawer_icon_rects.push((app_key, hit, center));
                    }
                    match item {
                        DrawerItem::Builtin(app) => {
                            let app = *app;
                            let resp = f.target(
                                LAUNCHER_ID + 320 + app as u64,
                                cell,
                                app.name(),
                                true,
                                Role::Button,
                            );
                            icons::paint_styled_app_icon(self.clock,
                                f,
                                ir.inset(if held { 3.5 } else if resp.pressed { 2.0 } else { 0.0 }),
                                app,
                                self.sys.themed_drawer_icons,
                                &p,
                                self.sys.icon_shape,
                            );
                            if self.sys.show_drawer_labels {
                                f.styled_text(
                                    Rect {
                                        x: cell.x - 2.0,
                                        y: ir.y + icon_sz + 8.0,
                                        width: cell.width + 4.0,
                                        height: 18.0,
                                    },
                                    app.name(),
                                    p.on_surface,
                                    wallpaper::customize_font(text_style(10.5, 400, 1), self.sys.font_style),
                                );
                            }
                            if resp.clicked {
                                clicked_builtin = Some((app, ir));
                            }
                        }
                        DrawerItem::Desktop {
                            idx,
                            id,
                            name,
                            icon_name,
                        } => {
                            let resp = f.target(
                                LAUNCHER_ID + 1024 + *idx as u64,
                                cell,
                                name,
                                true,
                                Role::Button,
                            );
                            let icon_img = icon_name
                                .as_deref()
                                .and_then(|n| self.desktop_registry.icon_resolver.load_icon_image(n));
                            freedesktop::paint_desktop_app_icon(
                                f,
                                ir.inset(if held { 3.5 } else if resp.pressed { 2.0 } else { 0.0 }),
                                name,
                                id,
                                icon_img.as_ref(),
                                &p,
                            );
                            if self.sys.show_drawer_labels {
                                f.styled_text(
                                    Rect {
                                        x: cell.x - 2.0,
                                        y: ir.y + icon_sz + 8.0,
                                        width: cell.width + 4.0,
                                        height: 18.0,
                                    },
                                    name,
                                    p.on_surface,
                                    wallpaper::customize_font(text_style(10.5, 400, 1), self.sys.font_style),
                                );
                            }
                            if resp.clicked {
                                clicked_desktop = Some((*idx, ir));
                            }
                        }
                    }
                }
            }
            });
        });
        if let Some((app, ir)) = clicked_builtin {
            self.open_app_from(app, Some(ir), true, false, self.sys.themed_drawer_icons, now);
        } else if let Some((idx, ir)) = clicked_desktop {
            self.open_desktop_app_from(idx, None, Some(ir), false, now);
        }
        if content_alpha < 0.999 {
            f.pop_layer();
        }
        f.pop_transform();
    }

    fn render_widgets_sheet(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
        now: f64,
    ) {
        let p = self.sys.palette();
        let prog = self.widgets_anim.value.clamp(0.0, 1.0);
        f.paint_rect(phone, 0.0, Color(0, 0, 0, (prog * 110.0) as u8));

        let sheet_y = phone.y + status_h + 10.0 + (1.0 - prog) * phone.height * 0.85;
        let sheet = Rect {
            x: phone.x,
            y: sheet_y,
            width: phone.width,
            height: phone.y + phone.height - sheet_y,
        };
        f.paint_rect(sheet, 28.0, p.settings_bg);
        // Drag handle (tap closes)
        let handle_hit = Rect {
            x: sheet.x + sheet.width * 0.5 - 48.0,
            y: sheet.y + 4.0,
            width: 96.0,
            height: 22.0,
        };
        let rh = f.target(LAUNCHER_ID + 400, handle_hit, "Close widgets", true, Role::Button);
        f.paint_rect(
            Rect {
                x: sheet.x + sheet.width * 0.5 - 18.0,
                y: sheet.y + 12.0,
                width: 36.0,
                height: 4.0,
            },
            2.0,
            p.outline,
        );
        if rh.clicked {
            self.widgets_open = false;
            self.widgets_anim.set(0.0, now, 0.26, 0.20);
        }

        f.styled_text(
            Rect {
                x: sheet.x,
                y: sheet.y + 32.0,
                width: sheet.width,
                height: 30.0,
            },
            "Widgets",
            p.on_surface,
            text_style(22.0, 500, 1),
        );

        let search_r = Rect {
            x: sheet.x + 16.0,
            y: sheet.y + 76.0,
            width: sheet.width - 32.0,
            height: 46.0,
        };
        f.paint_rect(search_r, 23.0, p.card_pure);
        paint_sys_icon(
            f,
            Rect {
                x: search_r.x + 16.0,
                y: search_r.y + 13.0,
                width: 20.0,
                height: 20.0,
            },
            SysIcon::Search,
            p.on_surface,
        );
        let sf = Rect {
            x: search_r.x + 48.0,
            y: search_r.y + 2.0,
            width: search_r.width - 64.0,
            height: 42.0,
        };
        f.target(ID_WIDGETS_SEARCH, sf, "Search widgets", true, Role::TextField);
        f.styled_text(
            sf,
            if self.widgets_search.is_empty() {
                "Search"
            } else {
                &self.widgets_search
            },
            if self.widgets_search.is_empty() {
                p.on_surface_variant
            } else {
                p.on_surface
            },
            text_style(15.0, 400, 0),
        );

        let list_area = Rect {
            x: sheet.x + 14.0,
            y: search_r.y + search_r.height + 16.0,
            width: sheet.width - 28.0,
            height: (phone.y + phone.height - nav_h - (search_r.y + search_r.height + 24.0))
                .max(120.0),
        };
        let groups: &[(AppId, &str, &str, &str)] = &[
            (AppId::Calendar, "Calendar", "1 widget", "Agenda · 4×2"),
            (AppId::Clock, "Clock", "2 widgets", "Digital Clock · 4×2"),
            (AppId::Contacts, "Contacts", "3 shortcuts", "Quick Contact · 2×2"),
            (AppId::Music, "Music", "3 widgets", "Now Playing · 4×1"),
        ];
        let q = self.widgets_search.trim().to_ascii_lowercase();
        let mut total_h = 0.0_f32;
        for (i, (_, name, _, preview)) in groups.iter().enumerate() {
            if !q.is_empty() && !name.to_ascii_lowercase().contains(&q) && !preview.to_ascii_lowercase().contains(&q) {
                continue;
            }
            total_h += 76.0 + if self.widgets_expanded[i] { 94.0 } else { 0.0 };
        }
        let max_s = (total_h - list_area.height + 24.0).max(0.0);
        self.last_scroll_max[2] = max_s;
        self.widgets_scroll = self.widgets_scroll.clamp(0.0, max_s);
        f.scroll_viewport(
            LAUNCHER_ID + 405,
            list_area,
            &mut self.widgets_scroll,
            max_s,
        );

        f.within(list_area, |f| {
            let mut y = list_area.y - self.widgets_scroll;
            for (i, &(app, name, count_lbl, widget_lbl)) in groups.iter().enumerate() {
                if !q.is_empty()
                    && !name.to_ascii_lowercase().contains(&q)
                    && !widget_lbl.to_ascii_lowercase().contains(&q)
                {
                    continue;
                }
                let expanded = self.widgets_expanded[i];
                let card_h = 72.0 + if expanded { 90.0 } else { 0.0 };
                let card = Rect {
                    x: list_area.x,
                    y,
                    width: list_area.width,
                    height: card_h,
                };
                f.paint_rect(card, 18.0, p.card_elevated);
                let hdr = Rect {
                    height: 72.0,
                    ..card
                };
                let r = f.target(LAUNCHER_ID + 410 + i as u64, hdr, name, true, Role::Button);
                f.state_layer(r, 18.0, p.on_surface);
                icons::paint_real_app_icon(self.clock,
                    f,
                    Rect {
                        x: hdr.x + 14.0,
                        y: hdr.y + 16.0,
                        width: 40.0,
                        height: 40.0,
                    },
                    app,
                    false,
                    &p,
                );
                f.styled_text(
                    Rect {
                        x: hdr.x + 68.0,
                        y: hdr.y + 16.0,
                        width: hdr.width - 110.0,
                        height: 22.0,
                    },
                    name,
                    p.on_surface,
                    text_style(15.0, 500, 0),
                );
                f.styled_text(
                    Rect {
                        x: hdr.x + 68.0,
                        y: hdr.y + 38.0,
                        width: hdr.width - 110.0,
                        height: 18.0,
                    },
                    count_lbl,
                    p.on_surface_variant,
                    text_style(12.5, 400, 0),
                );
                paint_sys_icon(
                    f,
                    Rect {
                        x: hdr.x + hdr.width - 36.0,
                        y: hdr.y + 26.0,
                        width: 20.0,
                        height: 20.0,
                    },
                    if expanded {
                        SysIcon::ChevronUp
                    } else {
                        SysIcon::ChevronDown
                    },
                    p.on_surface_variant,
                );
                if r.clicked {
                    self.widgets_expanded[i] = !self.widgets_expanded[i];
                }
                if expanded {
                    let prev_r = Rect {
                        x: card.x + 16.0,
                        y: card.y + 72.0,
                        width: card.width - 32.0,
                        height: 74.0,
                    };
                    f.paint_rect(prev_r, 16.0, p.surface_container);
                    f.styled_text(
                        Rect {
                            x: prev_r.x + 14.0,
                            y: prev_r.y + 12.0,
                            width: prev_r.width - 124.0,
                            height: 24.0,
                        },
                        widget_lbl,
                        p.on_surface,
                        text_style(14.0, 500, 0),
                    );
                    let is_added = match i {
                        0 => self.sys.show_weather_widget,
                        1 => self.sys.show_clock_widget,
                        _ => false,
                    };
                    let add_btn = Rect {
                        x: prev_r.x + prev_r.width - 102.0,
                        y: prev_r.y + 19.0,
                        width: 90.0,
                        height: 36.0,
                    };
                    let rb = f.target(
                        LAUNCHER_ID + 430 + i as u64,
                        add_btn,
                        "Toggle widget",
                        true,
                        Role::Button,
                    );
                    f.paint_rect(
                        add_btn,
                        18.0,
                        if is_added { p.primary_container } else { p.primary },
                    );
                    f.styled_text(
                        add_btn,
                        if is_added { "Remove" } else { "Add" },
                        if is_added {
                            p.on_primary_container
                        } else {
                            p.on_primary
                        },
                        text_style(13.0, 600, 1),
                    );
                    if rb.clicked {
                        match i {
                            0 => {
                                self.sys.show_weather_widget = !self.sys.show_weather_widget;
                                self.show_toast(
                                    if self.sys.show_weather_widget {
                                        "Calendar widget added to Home"
                                    } else {
                                        "Calendar widget removed"
                                    },
                                    now,
                                );
                            }
                            1 => {
                                self.sys.show_clock_widget = !self.sys.show_clock_widget;
                                self.show_toast(
                                    if self.sys.show_clock_widget {
                                        "Clock widget added to Home"
                                    } else {
                                        "Clock widget removed"
                                    },
                                    now,
                                );
                            }
                            _ => self.show_toast(format!("{name} widget pinned"), now),
                        }
                        self.widgets_open = false;
                        self.widgets_anim.set(0.0, now, 0.24, 0.20);
                    }
                }
                y += card_h + 6.0;
            }
        });
    }

    fn render_launcher_subscreen(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
        now: f64,
    ) {
        let prog = self.launcher_sub_anim.value.clamp(0.0, 1.0);
        let prev = self.prev_launcher_sub;
        let curr = self.launcher_sub;
        let gutter = 12.0;

        if prog < 0.995 && prev != curr {
            if self.launcher_sub_push {
                // Pushing a Launcher sub-screen (Home -> WallpaperStyle / HomeSettings -> HiddenApps)
                if prev != LauncherSub::None {
                    let out_x = -phone.width * 0.32 * prog;
                    f.push_transform([phone.x, phone.y], [out_x, 0.0], [1.0, 1.0], 0.0);
                    f.without_input(true, |f| {
                        self.render_launcher_sub_page(f, prev, phone, status_h, nav_h, now);
                    });
                    f.pop_transform();
                    f.paint_rect(phone, 0.0, Color(0, 0, 0, (prog * 36.0) as u8));
                }
                if curr != LauncherSub::None {
                    let in_x = (phone.width + gutter) * (1.0 - prog);
                    if in_x > 1.0 {
                        f.paint_rect(
                            Rect {
                                x: phone.x + in_x - gutter,
                                y: phone.y,
                                width: gutter,
                                height: phone.height,
                            },
                            0.0,
                            Color(0, 0, 0, 210),
                        );
                    }
                    f.push_transform([phone.x, phone.y], [in_x, 0.0], [1.0, 1.0], 0.0);
                    self.render_launcher_sub_page(f, curr, phone, status_h, nav_h, now);
                    f.pop_transform();
                }
            } else {
                // Popping back (HiddenApps -> HomeSettings, or WallpaperStyle -> Home)
                if curr != LauncherSub::None {
                    let in_x = -phone.width * 0.32 * (1.0 - prog);
                    f.push_transform([phone.x, phone.y], [in_x, 0.0], [1.0, 1.0], 0.0);
                    self.render_launcher_sub_page(f, curr, phone, status_h, nav_h, now);
                    f.pop_transform();
                    f.paint_rect(phone, 0.0, Color(0, 0, 0, ((1.0 - prog) * 36.0) as u8));
                }
                if prev != LauncherSub::None {
                    let out_x = (phone.width + gutter) * prog;
                    if out_x < phone.width {
                        f.paint_rect(
                            Rect {
                                x: phone.x + out_x - gutter,
                                y: phone.y,
                                width: gutter,
                                height: phone.height,
                            },
                            0.0,
                            Color(0, 0, 0, 210),
                        );
                    }
                    f.push_transform([phone.x, phone.y], [out_x, 0.0], [1.0, 1.0], 0.0);
                    f.without_input(true, |f| {
                        self.render_launcher_sub_page(f, prev, phone, status_h, nav_h, now);
                    });
                    f.pop_transform();
                }
            }
        } else if curr != LauncherSub::None {
            if curr != LauncherSub::WallpaperPreview { self.wallpaper_pending = None; }
            self.render_launcher_sub_page(f, curr, phone, status_h, nav_h, now);
        }
    }

    fn render_launcher_sub_page(
        &mut self,
        f: &mut Frame<'_>,
        sub: LauncherSub,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
        now: f64,
    ) {
        let p = self.sys.palette();
        f.paint_rect(phone, 0.0, p.settings_bg);
        let top_y = phone.y + status_h;

        // Back button
        let back_r = Rect {
            x: phone.x + 8.0,
            y: top_y + 6.0,
            width: 44.0,
            height: 44.0,
        };
        let page_id = match sub {
            LauncherSub::None => 0,
            LauncherSub::WallpaperStyle => 1,
            LauncherSub::WallpaperPicker => 2,
            LauncherSub::WallpaperFiles => 3,
            LauncherSub::WallpaperPreview => 4,
            LauncherSub::WallpaperSettings(kind) => 5 + kind as u64,
            LauncherSub::HomeSettings => 10,
            LauncherSub::HiddenApps => 11,
        };
        let rb = f.target(LAUNCHER_ID + 8000 + page_id, back_r, "Back", true, Role::Button);
        f.state_layer(rb, 22.0, p.on_surface);
        paint_sys_icon(f, back_r.inset(11.0), SysIcon::Back, p.on_surface);
        if rb.clicked {
            self.go_back(now);
            return;
        }

        let content = Rect {
            x: phone.x,
            y: top_y + 52.0,
            width: phone.width,
            height: (phone.height - status_h - nav_h - 52.0).max(160.0),
        };

        match sub {
            LauncherSub::WallpaperPicker | LauncherSub::WallpaperStyle | LauncherSub::WallpaperFiles
            | LauncherSub::WallpaperPreview | LauncherSub::WallpaperSettings(_) => {
                let title = match sub {
                    LauncherSub::WallpaperPicker => "Change wallpaper",
                    LauncherSub::WallpaperStyle => "Wallpaper & style",
                    LauncherSub::WallpaperFiles => "My photos",
                    LauncherSub::WallpaperPreview => "Wallpaper preview",
                    LauncherSub::WallpaperSettings(kind) => kind.title(),
                    _ => unreachable!(),
                };
                f.styled_text(
                    Rect { x: phone.x + 60.0, y: top_y + 13.0, width: phone.width - 76.0, height: 30.0 },
                    title, p.on_surface, wallpaper::customize_font(text_style(21.0, 500, 0), self.sys.font_style),
                );
                self.paint_wallpaper_page(f, sub, content, phone, now);
            }
            LauncherSub::HomeSettings => {
                // Exact match to 13-home-settings-top.png + EdgeStretch + CollapsingToolbar!
                let max_s = (760.0 - content.height).max(0.0);
                self.last_scroll_max[4] = max_s;
                self.home_settings_scroll = self.home_settings_scroll.clamp(0.0, max_s);
                f.scroll_viewport(
                    LAUNCHER_ID + 550,
                    content,
                    &mut self.home_settings_scroll,
                    max_s,
                );
                let (st_s, st_sign) = self.target_stretch(ScrollTarget::HomeSettings);
                let collapse = (self.home_settings_scroll / 56.0).clamp(0.0, 1.0);
                let tx = lerp(content.x + 22.0, phone.x + 60.0, collapse);
                let ty = lerp(top_y + 56.0, top_y + 13.0, collapse);
                if collapse <= 0.005 {
                    f.styled_text(
                        Rect { x: tx, y: ty, width: phone.width - 76.0, height: 42.0 },
                        "Home settings",
                        p.on_surface,
                        text_style(30.0, 400, 0),
                    );
                } else if collapse >= 0.995 {
                    f.styled_text(
                        Rect { x: tx, y: top_y + 11.0, width: phone.width - 76.0, height: 28.0 },
                        "Home settings",
                        p.on_surface,
                        text_style(20.0, 500, 0),
                    );
                } else {
                    let tsc = lerp(1.0, 20.0 / 30.0, collapse);
                    f.push_transform([tx, ty], [0.0, 0.0], [tsc, tsc], 0.0);
                    f.styled_text(
                        Rect { x: tx, y: ty, width: (phone.width - 76.0) / tsc, height: 42.0 },
                        "Home settings",
                        p.on_surface,
                        text_style(30.0, 400, 0),
                    );
                    f.pop_transform();
                }
                f.within(content, |f| {
                    f.with_edge_stretch(content, false, st_s, st_sign, |f| {
                    let mut y = content.y + 58.0 - self.home_settings_scroll;

                    let row_w = content.width - 32.0;
                    let rx = content.x + 16.0;
                    paint_pref_switch(
                        f,
                        LAUNCHER_ID + 551,
                        Rect { x: rx, y, width: row_w, height: 64.0 },
                        "Lock layout",
                        Some("Prevent adding or moving icons on home"),
                        &mut self.sys.lock_layout,
                        &p,
                    );
                    y += 66.0;
                    paint_pref_switch(
                        f,
                        LAUNCHER_ID + 552,
                        Rect { x: rx, y, width: row_w, height: 64.0 },
                        "Notification dots",
                        Some("Show unread dots on app icons"),
                        &mut self.sys.notification_dots,
                        &p,
                    );
                    y += 66.0;
                    paint_pref_switch(
                        f,
                        LAUNCHER_ID + 553,
                        Rect { x: rx, y, width: row_w, height: 64.0 },
                        "Add app icons to home screen",
                        Some("For new apps"),
                        &mut self.sys.add_new_icons,
                        &p,
                    );
                    y += 66.0;
                    paint_pref_switch(
                        f,
                        LAUNCHER_ID + 554,
                        Rect { x: rx, y, width: row_w, height: 64.0 },
                        "Allow home screen rotation",
                        Some("When phone is rotated"),
                        &mut self.sys.home_rotation,
                        &p,
                    );
                    y += 66.0;

                    // Hidden & Protected apps row
                    let hid_r = Rect {
                        x: rx,
                        y,
                        width: row_w,
                        height: 62.0,
                    };
                    let rh = f.target(
                        LAUNCHER_ID + 555,
                        hid_r,
                        "Hidden & Protected apps",
                        true,
                        Role::Button,
                    );
                    f.state_layer(rh, 12.0, p.on_surface);
                    f.styled_text(
                        Rect {
                            x: hid_r.x + 6.0,
                            y: hid_r.y + 10.0,
                            width: hid_r.width - 40.0,
                            height: 22.0,
                        },
                        "Hidden & Protected apps",
                        p.on_surface,
                        text_style(16.0, 400, 0),
                    );
                    let hid_sub = if self.sys.hidden_apps.is_empty() {
                        "No apps hidden".to_owned()
                    } else {
                        format!("{} hidden", self.sys.hidden_apps.len())
                    };
                    f.styled_text(
                        Rect {
                            x: hid_r.x + 6.0,
                            y: hid_r.y + 34.0,
                            width: hid_r.width - 40.0,
                            height: 18.0,
                        },
                        &hid_sub,
                        p.on_surface_variant,
                        text_style(13.0, 400, 0),
                    );
                    paint_sys_icon(
                        f,
                        Rect {
                            x: hid_r.x + hid_r.width - 28.0,
                            y: hid_r.y + 21.0,
                            width: 18.0,
                            height: 18.0,
                        },
                        SysIcon::ChevronRight,
                        p.on_surface_variant,
                    );
                    if rh.clicked {
                        self.launcher_sub = LauncherSub::HiddenApps;
                        self.set_launcher_sub(LauncherSub::HiddenApps, true, now);
                    }
                    y += 66.0;

                    paint_pref_switch(
                        f,
                        LAUNCHER_ID + 556,
                        Rect { x: rx, y, width: row_w, height: 64.0 },
                        "Use themed icons in drawer",
                        Some("Follow themed icons used on home screen"),
                        &mut self.sys.themed_drawer_icons,
                        &p,
                    );
                    y += 66.0;
                    paint_pref_switch(
                        f,
                        LAUNCHER_ID + 557,
                        Rect { x: rx, y, width: row_w, height: 58.0 },
                        "Show icon labels on desktop",
                        None,
                        &mut self.sys.show_home_labels,
                        &p,
                    );
                    y += 60.0;
                    paint_pref_switch(
                        f,
                        LAUNCHER_ID + 558,
                        Rect { x: rx, y, width: row_w, height: 58.0 },
                        "Show icon labels in drawer",
                        None,
                        &mut self.sys.show_drawer_labels,
                        &p,
                    );
                    });
                });
            }
            LauncherSub::HiddenApps => {
                f.styled_text(
                    Rect {
                        x: content.x + 22.0,
                        y: content.y,
                        width: content.width - 44.0,
                        height: 36.0,
                    },
                    "Hidden & Protected apps",
                    p.on_surface,
                    text_style(24.0, 400, 0),
                );
                let list = Rect {
                    x: content.x + 16.0,
                    y: content.y + 44.0,
                    width: content.width - 32.0,
                    height: (content.height - 52.0).max(120.0),
                };
                let max_s = (AppId::ALL.len() as f32 * 56.0 - list.height).max(0.0);
                self.last_scroll_max[4] = max_s;
                self.home_settings_scroll = self.home_settings_scroll.clamp(0.0, max_s);
                f.scroll_viewport(LAUNCHER_ID + 560, list, &mut self.home_settings_scroll, max_s);
                f.within(list, |f| {
                    for (i, &app) in AppId::ALL.iter().enumerate() {
                        let ry = list.y + i as f32 * 56.0 - self.home_settings_scroll;
                        if ry + 56.0 < list.y || ry > list.y + list.height {
                            continue;
                        }
                        let mut hidden = self.is_app_hidden(app);
                        let changed = paint_pref_switch(
                            f,
                            LAUNCHER_ID + 570 + i as u64,
                            Rect {
                                x: list.x,
                                y: ry,
                                width: list.width,
                                height: 54.0,
                            },
                            app.name(),
                            None,
                            &mut hidden,
                            &p,
                        );
                        if changed {
                            let pkg = app.package().to_string();
                            if hidden {
                                if !self.sys.hidden_apps.contains(&pkg) {
                                    self.sys.hidden_apps.push(pkg);
                                }
                            } else {
                                self.sys
                                    .hidden_apps
                                    .retain(|a| a != &pkg && a != app.name());
                            }
                        }
                    }
                });
            }
            LauncherSub::None => {}
        }
    }

    // =========================================================================
    // SUBSYSTEM 2: NOTIFICATION SHADE, QUICK SETTINGS, VOLUME & POWER
    // =========================================================================

    #[inline(never)]
    fn render_shade_and_qs(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        status_h: f32,
        base_light_ink: bool,
        pointer: Option<[f32; 2]>,
        pointer_down: bool,
        now: f64,
    ) {
        let p = self.sys.palette();
        let stage = self.shade_anim.value.clamp(0.0, 2.0);
        let s1 = stage.clamp(0.0, 1.0); // 0.0 = Closed -> 1.0 = Notification Shade
        let s2 = (stage - 1.0).clamp(0.0, 1.0); // 0.0 = Shade -> 1.0 = Expanded Quick Settings

        let edit_prog = self.qs_edit_anim.value.clamp(0.0, 1.0);
        if self.qs_edit_open && edit_prog >= 0.995 {
            f.within(phone, |f| {
                self.render_qs_tile_editor(f, phone, status_h, pointer, pointer_down, now);
            });
            return;
        }

        let pad = 16.0;
        let col_gap = 8.0;
        let tile_w = (phone.width - pad * 2.0 - col_gap) * 0.5;
        let row_gap = QS_ROW_GAP;

        let phone_bottom = phone.y + phone.height;
        let compact_notif_y = Self::compact_notification_sheet_top(phone, status_h);
        let notif_y = lerp(lerp(phone.y, compact_notif_y, s1), phone_bottom, s2)
            + motion::settle_bounce(self.notification_sheet_bounce.value, 0.0, 4.0);
        let sheet_surface = if p.dark {
            Color(0x1C, 0x1E, 0x24, 255)
        } else {
            Color(0xEE, 0xEE, 0xF3, 255)
        };
        let first_pull = s1 < 1.0;
        let controls_clip = if first_pull {
            Rect { height: notif_y - phone.y, ..phone }
        } else {
            phone
        };
        f.paint_rect(controls_clip, 0.0, p.qs_backdrop);
        if first_pull {
            // The underlying screen keeps a rounded leading edge while it fades into the sheet.
            let sheet = Rect { y: notif_y, height: phone_bottom - notif_y, ..phone };
            f.paint_rect(sheet, 0.0, sheet_surface.opacity(smoothstep(0.0, 0.56, s1)));
            let radius = (controls_clip.height * 0.6).min(28.0);
            if radius >= 1.0 {
                f.within(Rect { height: radius.max(8.0), ..sheet }, |f| {
                    f.paint_rounded_screen_mask(sheet, radius, p.qs_backdrop);
                });
            }
        }

        // 2. Render Shade/QS interior clipped cleanly to `phone` (disable QS input if QS Edit is revealing over it!)
        f.without_input(self.qs_edit_open, |f| {
        f.within(phone, |f| {
        f.within(controls_clip, |f| {
            // === UNIFIED CONTINUOUSLY MORPHING HEADER (ZERO BINARY CUT!) ===
            let base_ink = if base_light_ink {
                Color(0xFF, 0xFF, 0xFF, 255)
            } else {
                Color(0x1B, 0x1B, 0x1F, 255)
            };
            let hdr_ink = base_ink.mix(p.on_surface, smoothstep(0.04, 0.35, s1));
            let clock_x0 = match self.sys.clock_pos {
                ClockPos::Left => phone.x + 18.0,
                ClockPos::Center => phone.x + phone.width * 0.5 - 26.0,
                ClockPos::Right => phone.x + phone.width - 154.0,
            };
            let clock_x = lerp(clock_x0, phone.x + 18.0, smoothstep(0.0, 0.45, s1));
            let clock_y = lerp(phone.y, phone.y + 36.0, s2);
            let clock_h = lerp(status_h, 44.0, s2);
            let clock_sz = lerp(13.0, 34.0, s2);
            let clock_weight = if s2 < 0.35 {
                600
            } else if s2 < 0.70 {
                500
            } else {
                400
            };
            let time_str = self.clock.hm();
            f.styled_text(
                Rect {
                    x: clock_x,
                    y: clock_y,
                    width: 140.0,
                    height: clock_h,
                },
                &time_str,
                hdr_ink,
                text_style(clock_sz, clock_weight, 0),
            );

            // Status bar notification icons fade out cleanly as shade opens (s1: 0.0 -> 0.25)
            let status_icons_a = 1.0 - smoothstep(0.0, 0.28, s1);
            if status_icons_a > 0.02 && self.sys.clock_pos == ClockPos::Left {
                let mut ix = phone.x + 62.0;
                let ic_col = hdr_ink.opacity(status_icons_a);
                let unread = self.notifications.iter().filter(|n| !n.snoozed).count();
                if self.sys.usb_debugging {
                    paint_sys_icon(
                        f,
                        Rect {
                            x: ix,
                            y: phone.y + 8.5,
                            width: 14.0,
                            height: 14.0,
                        },
                        SysIcon::Usb,
                        ic_col,
                    );
                    ix += 18.0;
                }
                if unread > 0 {
                    paint_sys_icon(
                        f,
                        Rect {
                            x: ix,
                            y: phone.y + 8.5,
                            width: 14.0,
                            height: 14.0,
                        },
                        SysIcon::Bell,
                        ic_col,
                    );
                }
            }

            // The local date fades in next to clock during s1, then glides cleanly beneath large clock during s2!
            let date_a = smoothstep(0.18, 0.52, s1);
            let date_y = clock_y + lerp(0.0, 46.0, smoothstep(0.0, 0.78, s2));
            let date_h = lerp(status_h, 20.0, s2);
            if date_a > 0.01 {
                let date_x = lerp(phone.x + 64.0, phone.x + 18.0, smoothstep(0.28, 0.90, s2));
                f.styled_text(
                    Rect {
                        x: date_x,
                        y: date_y,
                        width: 120.0,
                        height: date_h,
                    },
                    &self.clock.date(),
                    hdr_ink.opacity(date_a),
                    text_style(13.0, 400, 0),
                );
            }

            // "Emergency calls only" fades in right-aligned as QS expands (s2)
            let carrier_a = smoothstep(0.32, 0.88, s2);
            if carrier_a > 0.01 {
                f.styled_text(
                    Rect {
                        x: phone.x + phone.width - 170.0,
                        y: lerp(phone.y + 8.0, phone.y + 46.0, s2),
                        width: 152.0,
                        height: 22.0,
                    },
                    "Emergency calls only",
                    p.on_surface.opacity(carrier_a),
                    text_style(13.0, 400, 2),
                );
            }

            // Right status cluster (Battery %, Wi-Fi, Ringer) glides smoothly alongside date row!
            let right_y = date_y;
            let right_h = date_h;
            let right_ic_y = right_y + (right_h - 15.0) * 0.5;
            let mut rx = phone.x + phone.width - 18.0;
            let pct = format!("{}%", self.sys.battery_level);
            rx -= 36.0;
            f.styled_text(
                Rect {
                    x: rx,
                    y: right_y,
                    width: 36.0,
                    height: right_h,
                },
                &pct,
                hdr_ink,
                text_style(13.0, 500, 2),
            );
            rx -= 22.0;
            paint_sys_icon(
                f,
                Rect {
                    x: rx,
                    y: right_ic_y,
                    width: 15.0,
                    height: 15.0,
                },
                if self.sys.airplane_mode {
                    SysIcon::Airplane
                } else if self.sys.wifi_enabled {
                    SysIcon::Wifi
                } else {
                    SysIcon::WifiOff
                },
                hdr_ink,
            );
            rx -= 22.0;
            paint_sys_icon(
                f,
                Rect {
                    x: rx,
                    y: right_ic_y,
                    width: 15.0,
                    height: 15.0,
                },
                match self.sys.ringer_mode {
                    RingerMode::Vibrate => SysIcon::Vibrate,
                    RingerMode::Silent => SysIcon::Silent,
                    RingerMode::Ring => SysIcon::Ring,
                },
                hdr_ink,
            );

            // A header tap toggles the two open shade stages.
            let hdr_hit = Rect {
                x: phone.x,
                y: phone.y,
                width: phone.width,
                height: status_h + 4.0,
            };
            let hdr_r = f.target(SHADE_ID + 1, hdr_hit, "Notification shade header", true, Role::Button);
            if hdr_r.clicked {
                self.tap_shade_header(now);
            }

            // === UNIFIED MORPHING LAYOUT METRICS (ZERO OVERLAP AT EVERY s2 IN 0.0..=1.0!) ===
            let grid_y = lerp(phone.y + status_h + 12.0, phone.y + 182.0, s2);
            // Compact tiles are pills. Expanded tiles grow to fit the subtitle
            // while keeping nearly the same corner radius.
            let tile_h = lerp(QS_COMPACT_TILE_HEIGHT, 78.0, s2);
            let tile_radius = lerp(29.0, 28.0, s2);
            // === BRIGHTNESS PILL SLIDER + AUTO BUTTON (expands cleanly into the gap between Date & Grid!) ===
            let br_slot_top = date_y + date_h + 10.0;
            let br_slot_h = (grid_y - 14.0 - br_slot_top).max(0.0);
            let br_a = smoothstep(0.24, 0.78, s2);
            let auto_sz = 44.0_f32.min(br_slot_h);
            let br_h = br_slot_h.clamp(0.0, 48.0);
            let br_y = br_slot_top + (br_slot_h - br_h) * 0.5;
            let br_bar = Rect {
                x: phone.x + pad,
                y: br_y,
                width: phone.width - pad * 2.0 - 44.0 - 12.0,
                height: br_h,
            };
            // Paint through the fade even after the controls are too small to touch.
            if br_a > 0.0 && br_h > 0.0 {
                let interactive = s2 > 0.45 && br_h >= 12.0;
                if interactive {
                    self.last_brightness_rect = Rect {
                        x: br_bar.x,
                        y: br_bar.y - 4.0,
                        width: br_bar.width,
                        height: br_bar.height + 8.0,
                    };
                }
                let _br_r = f.target(
                    SHADE_ID + 10,
                    br_bar,
                    "Screen brightness",
                    interactive,
                    Role::Slider,
                );
                let br_rad = br_h * 0.5;
                // Thin inactive track
                let track_h = br_h.min(6.0);
                f.paint_rect(
                    Rect {
                        x: br_bar.x + br_rad,
                        y: br_bar.y + (br_h - track_h) * 0.5,
                        width: (br_bar.width - br_rad).max(0.0),
                        height: track_h,
                    },
                    track_h * 0.5,
                    p.card_pure.opacity(br_a),
                );
                // Active pill fill
                let fill_w = (br_bar.width * self.sys.brightness).clamp(br_h, br_bar.width);
                f.paint_rect(
                    Rect {
                        width: fill_w,
                        ..br_bar
                    },
                    br_rad,
                    p.primary.opacity(br_a),
                );
                let sun_sz = (br_h * 0.42).min(20.0);
                paint_sys_icon(
                    f,
                    Rect {
                        x: br_bar.x + fill_w - sun_sz - 12.0,
                        y: br_bar.y + (br_h - sun_sz) * 0.5,
                        width: sun_sz,
                        height: sun_sz,
                    },
                    SysIcon::Sun,
                    p.on_primary.opacity(br_a),
                );
                // Auto brightness button
                let auto_r = Rect {
                    x: br_bar.x + br_bar.width + 12.0 + (44.0 - auto_sz) * 0.5,
                    y: br_y + (br_h - auto_sz) * 0.5,
                    width: auto_sz,
                    height: auto_sz,
                };
                let ra = f.target(
                    SHADE_ID + 11,
                    auto_r,
                    "Adaptive brightness",
                    interactive,
                    Role::ToggleButton,
                );
                if self.sys.auto_brightness {
                    f.paint_rect(auto_r, auto_sz * 0.5, p.primary_container.opacity(0.9 * br_a));
                }
                f.state_layer(ra, auto_sz * 0.5, p.on_surface.opacity(br_a));
                let ab_ic = (auto_sz * 0.55).min(24.0);
                paint_sys_icon(
                    f,
                    Rect {
                        x: auto_r.x + (auto_r.width - ab_ic) * 0.5,
                        y: auto_r.y + (auto_r.height - ab_ic) * 0.5,
                        width: ab_ic,
                        height: ab_ic,
                    },
                    SysIcon::AutoBrightness,
                    if self.sys.auto_brightness {
                        p.on_primary_container.opacity(br_a)
                    } else {
                        p.on_surface_variant.opacity(0.6 * br_a)
                    },
                );
                if ra.clicked {
                    self.sys.auto_brightness = !self.sys.auto_brightness;
                    self.trigger_haptic(services::HapticEffect::Click);
                    let msg = if self.sys.auto_brightness {
                        if let Some(lux) = self.sys.ambient_lux {
                            format!("Adaptive brightness on ({:.0} lx)", lux)
                        } else {
                            "Adaptive brightness on".into()
                        }
                    } else {
                        "Adaptive brightness off".into()
                    };
                    self.show_toast(&msg, now);
                }
            }

            // === UNIFIED MORPHING QS TILE GRID (2x2 COMPACT -> 2x4 EXPANDED + SIDE-BY-SIDE PAGE SWIPE!) ===
            // Fit both compact rows to the same height. Fade rows out when
            // the shade edge can no longer fit their minimum height.
            let row0_y = lerp(phone.y + status_h - 4.0, grid_y, s1);
            let compact_gap = row_gap * s1;
            let available = notif_y - row0_y;
            let per_row = (available - compact_gap) * 0.5;
            let compact_h = if first_pull {
                per_row.clamp(QS_MIN_TILE_HEIGHT, tile_h)
            } else {
                tile_h
            };
            let row1_y = row0_y + compact_h + compact_gap;
            let (row0_alpha, row1_alpha) = if first_pull {
                let fade = |space| smoothstep(QS_MIN_TILE_HEIGHT, QS_MIN_TILE_HEIGHT + 8.0, space);
                (fade(available), fade(per_row))
            } else {
                (1.0, 1.0)
            };
            let sub_alpha = smoothstep(0.32, 0.88, s2);
            let bottom4_alpha = smoothstep(0.18, 0.82, s2);
            let page_count = self.qs_tiles.len().div_ceil(8);
            let page_f = self.qs_page_anim.value * smoothstep(0.0, 0.25, s2);
            let qs_covered = s2 > 0.0 && pointer.is_some_and(|[_, y]| y >= notif_y);

            f.without_pointer_input(qs_covered, |f| {
            let grid_full_h = 4.0 * tile_h + 3.0 * row_gap;
            let qs_grid_clip = Rect {
                x: phone.x,
                y: phone.y + status_h - 4.0,
                width: phone.width,
                height: (grid_y + grid_full_h + 10.0 - (phone.y + status_h - 4.0)).max(grid_full_h),
            };
            if s2 > 0.45 {
                self.last_qs_grid_rect = Rect {
                    height: qs_grid_clip.height.min((notif_y - qs_grid_clip.y).max(0.0)),
                    ..qs_grid_clip
                };
            }

            let all_tiles = self.qs_tiles.clone();
            f.within(qs_grid_clip, |f| {
                let p0_dx = -page_f * phone.width;
                if p0_dx + phone.width > -8.0 && p0_dx < phone.width + 8.0 {
                    for (idx, &kind) in all_tiles.iter().take(8).enumerate() {
                        let col = idx % 2;
                        let row = idx / 2;
                        let (tile_y, height, alpha) = match row {
                            0 => (row0_y, compact_h, row0_alpha),
                            1 => (row1_y, compact_h, row1_alpha),
                            _ => (grid_y + row as f32 * (tile_h + row_gap), tile_h, bottom4_alpha),
                        };
                        if alpha <= 0.01 || height <= 0.0 {
                            continue;
                        }
                        let tr = Rect {
                            x: phone.x + pad + col as f32 * (tile_w + col_gap) + p0_dx,
                            y: tile_y,
                            width: tile_w,
                            height,
                        };
                        let interactive = alpha > 0.45 && height >= 44.0 && page_f < 0.55;
                        self.render_qs_tile(
                            f,
                            SHADE_ID + 30 + idx as u64,
                            tr,
                            tile_radius.min(height * 0.5),
                            kind,
                            sub_alpha,
                            alpha,
                            interactive,
                            now,
                        );
                    }
                }

                for (page, tiles) in all_tiles.chunks(8).enumerate().skip(1) {
                    let dx = (page as f32 - page_f) * phone.width;
                    if s2 <= 0.15 || dx >= phone.width + 8.0 || dx + phone.width <= -8.0 {
                        continue;
                    }
                    for (idx, &kind) in tiles.iter().enumerate() {
                        let col = idx % 2;
                        let row = idx / 2;
                        let alpha = bottom4_alpha;
                        if alpha <= 0.01 {
                            continue;
                        }
                        let tr = Rect {
                            x: phone.x + pad + col as f32 * (tile_w + col_gap) + dx,
                            y: grid_y + row as f32 * (tile_h + row_gap),
                            width: tile_w,
                            height: tile_h,
                        };
                        let interactive = alpha > 0.45 && (page_f - page as f32).abs() <= 0.55;
                        self.render_qs_tile(
                            f,
                            SHADE_ID + 50 + idx as u64 + (page as u64 - 1) * 8,
                            tr,
                            tile_radius,
                            kind,
                            sub_alpha,
                            alpha,
                            interactive,
                            now,
                        );
                    }
                }
            });

            // === EXPANDED QS META ROW & FOOTER (smoothly rise & fade in as s2: 0.4 -> 1.0) ===
            let meta_a = smoothstep(0.42, 0.94, s2);
            if meta_a > 0.01 {
                let meta_dy = (1.0 - s2) * 20.0;
                let meta_y = grid_y + 4.0 * (tile_h + row_gap) + 10.0 + meta_dy;
                f.styled_text(
                    Rect {
                        x: phone.x + 18.0,
                        y: meta_y,
                        width: 170.0,
                        height: 24.0,
                    },
                    "13 (TQ3A.230901.001)",
                    p.on_surface_variant.opacity(meta_a),
                    text_style(12.5, 400, 0),
                );
                // Page switcher dots
                let dots_hit = Rect {
                    x: phone.x + phone.width * 0.5 - 30.0,
                    y: meta_y - 4.0,
                    width: 60.0,
                    height: 32.0,
                };
                if page_count > 1 && let Some(next_pg) = f.page_indicator(
                    SHADE_ID + 15,
                    dots_hit,
                    page_count,
                    page_f,
                    PageIndicatorOptions {
                        dot_size: 6.0,
                        gap: 14.0,
                        active_color: Some(p.primary.opacity(meta_a)),
                        inactive_color: Some(p.on_surface_variant.opacity(0.45 * meta_a)),
                        interactive: meta_a > 0.5,
                    },
                ) {
                    self.qs_page = next_pg;
                    self.qs_page_anim.set(self.qs_page as f32, now, 0.24, 0.24);
                }

                // Generous 44×44dp Edit Pencil button clear of right EdgeBack strip -> 280ms Circular Reveal!
                let edit_r = Rect {
                    x: phone.x + phone.width - 58.0,
                    y: meta_y - 10.0,
                    width: 44.0,
                    height: 44.0,
                };
                if meta_a > 0.5 {
                    self.last_qs_edit_btn = edit_r;
                }
                let re = f.target(
                    SHADE_ID + 16,
                    edit_r,
                    "Edit tiles",
                    meta_a > 0.5,
                    Role::Button,
                );
                f.state_layer(re, 22.0, p.on_surface.opacity(meta_a));
                paint_sys_icon(f, edit_r.inset(12.0), SysIcon::Edit, p.on_surface.opacity(meta_a));
                if re.clicked {
                    self.qs_edit_open = true;
                    self.qs_edit_scroll = 0.0;
                    self.qs_edit_last_frame = None;
                    self.qs_edit_origin = [
                        edit_r.x + edit_r.width * 0.5,
                        edit_r.y + edit_r.height * 0.5,
                    ];
                    self.qs_edit_anim.snap(0.0);
                    self.qs_edit_anim.set(1.0, now, 0.28, 0.24);
                }

                // Tap blank area between tiles and footer to collapse back to Shade
                let collapse_area = Rect {
                    x: phone.x,
                    y: meta_y + 32.0,
                    width: phone.width,
                    height: (phone.y + phone.height - 96.0 - (meta_y + 32.0)).max(24.0),
                };
                let rc = f.target(
                    SHADE_ID + 17,
                    collapse_area,
                    "Collapse QS",
                    meta_a > 0.6,
                    Role::Button,
                );
                if rc.clicked {
                    self.settle_shade(1, now);
                }

                // Bottom Footer Action Bar ("1 app is active >", Settings gear circle, #435E91 Power circle)
                let foot_y = phone.y + phone.height - 64.0 + (1.0 - s2) * 28.0;
                let foot_pill_w = phone.width - pad * 2.0 - 104.0;
                let active_pill = Rect {
                    x: phone.x + pad,
                    y: foot_y,
                    width: foot_pill_w.max(140.0),
                    height: 44.0,
                };
                let r_act = f.target(
                    SHADE_ID + 20,
                    active_pill,
                    "Active apps",
                    meta_a > 0.5,
                    Role::Button,
                );
                f.paint_rect(active_pill, 22.0, p.card_elevated.opacity(meta_a));
                f.state_layer(r_act, 22.0, p.on_surface.opacity(meta_a));
                paint_sys_icon(
                    f,
                    Rect {
                        x: active_pill.x + 14.0,
                        y: active_pill.y + 12.0,
                        width: 20.0,
                        height: 20.0,
                    },
                    SysIcon::Info,
                    p.on_surface.opacity(meta_a),
                );
                let shiraz_active_count = usize::from(self.sys.shiraz_enabled)
                    + self.sys.shiraz.running_packages.len();
                let total_active = self.active_bg_apps.len() + shiraz_active_count;
                let active_lbl = match total_active {
                    0 => "No apps active".to_owned(),
                    1 => "1 app is active".to_owned(),
                    n => format!("{n} apps are active"),
                };
                f.styled_text(
                    Rect {
                        x: active_pill.x + 44.0,
                        y: active_pill.y,
                        width: active_pill.width - 74.0,
                        height: 44.0,
                    },
                    &active_lbl,
                    p.on_surface.opacity(meta_a),
                    text_style(13.5, 400, 0),
                );
                paint_sys_icon(
                    f,
                    Rect {
                        x: active_pill.x + active_pill.width - 30.0,
                        y: active_pill.y + 14.0,
                        width: 16.0,
                        height: 16.0,
                    },
                    SysIcon::ChevronRight,
                    p.on_surface.opacity(meta_a),
                );
                if r_act.clicked {
                    self.active_apps_open = true;
                }

                // Settings gear circle
                let set_btn = Rect {
                    x: active_pill.x + active_pill.width + 10.0,
                    y: foot_y,
                    width: 44.0,
                    height: 44.0,
                };
                let r_set = f.target(
                    SHADE_ID + 21,
                    set_btn,
                    "Open Settings",
                    meta_a > 0.5,
                    Role::Button,
                );
                f.paint_rect(set_btn, 22.0, p.card_elevated.opacity(meta_a));
                f.state_layer(r_set, 22.0, p.on_surface.opacity(meta_a));
                paint_sys_icon(
                    f,
                    set_btn.inset(12.0),
                    SysIcon::Settings,
                    p.on_surface.opacity(meta_a),
                );
                if r_set.clicked {
                    self.settings_app
                        .navigate(settings::SettingsPage::Home, now);
                    self.open_app(AppId::Settings, now);
                }

                // Power button circle (#435E91)
                let pwr_btn = Rect {
                    x: set_btn.x + 52.0,
                    y: foot_y,
                    width: 44.0,
                    height: 44.0,
                };
                let r_pwr = f.target(
                    SHADE_ID + 22,
                    pwr_btn,
                    "Power menu",
                    meta_a > 0.5,
                    Role::Button,
                );
                f.paint_rect(pwr_btn, 22.0, p.primary.opacity(meta_a));
                f.state_layer(r_pwr, 22.0, p.on_primary.opacity(meta_a));
                paint_sys_icon(
                    f,
                    pwr_btn.inset(12.0),
                    SysIcon::Power,
                    p.on_primary.opacity(meta_a),
                );
                if r_pwr.clicked {
                    self.power_open = true;
                    self.power_anim.set(1.0, now, 0.24, 0.18);
                }
            }
            });
        });

            let notif_a = smoothstep(0.43, 0.88, s1);
            if notif_a > 0.01 && notif_y < phone_bottom {
                let notif_h = (phone_bottom - notif_y - 36.0).max(120.0);
                let notif_area = Rect {
                    x: phone.x,
                    y: notif_y,
                    width: phone.width,
                    height: notif_h,
                };
                if s2 < 0.45 {
                    self.last_notif_container = notif_area;
                }
                if notif_a < 0.999 {
                    f.push_layer(phone, notif_a);
                }
                if !first_pull {
                    // As expanded QS collapses, the opaque sheet covers the rows behind it.
                    paint_asym_rect(
                        f,
                        Rect { height: phone_bottom - notif_y, ..notif_area },
                        28.0,
                        0.0,
                        sheet_surface,
                    );
                }
                f.without_pointer_input(s2 > 0.35 || s1 < 0.65, |f| {
                    self.render_notifications_list(f, notif_area, now);
                });
                if notif_a < 0.999 {
                    f.pop_layer();
                }
            }

            // Bottom close-shade pill bar (fades smoothly during s2; dragging up ANYWHERE closes too!)
            let close_a = smoothstep(0.25, 0.75, s1) * (1.0 - smoothstep(0.0, 0.45, s2));
            if close_a > 0.02 {
                let close_hit = Rect {
                    x: phone.x,
                    y: phone.y + phone.height - 32.0,
                    width: phone.width,
                    height: 32.0,
                };
                let r_close = f.target(
                    SHADE_ID + 26,
                    close_hit,
                    "Close shade",
                    close_a > 0.5,
                    Role::Button,
                );
                f.paint_rect(
                    Rect {
                        x: phone.x + phone.width * 0.5 - 48.0,
                        y: phone.y + phone.height - 14.0,
                        width: 96.0,
                        height: 4.0,
                    },
                    2.0,
                    p.on_surface.opacity(0.65 * close_a),
                );
                if r_close.clicked {
                    self.shade_stage = 0;
                    self.shade_anim.set(0.0, now, 0.26, 0.20);
                }
            }
        });
        });

        // Circular reveal overlay for QS Tile Editor (`strip-notification-edit.png`), clipped to phone!
        if self.qs_edit_open || self.qs_edit_anim.value > 0.005 {
            f.within(phone, |f| {
                self.render_qs_tile_editor(f, phone, status_h, pointer, pointer_down, now);
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_qs_tile(
        &mut self,
        f: &mut Frame<'_>,
        id: u64,
        rect: Rect,
        radius: f32,
        kind: QsTileKind,
        sub_alpha: f32,
        tile_alpha: f32,
        interactive: bool,
        now: f64,
    ) {
        let p = self.sys.palette();
        let (title, subtitle, icon, active, unavailable, has_chevron): (
            String,
            String,
            SysIcon,
            bool,
            bool,
            bool,
        ) = match kind {
            QsTileKind::Wifi | QsTileKind::Bluetooth => {
                let unavailable = if kind == QsTileKind::Wifi { !self.sys.wifi_available }
                    else { !self.sys.bluetooth_available };
                let (title, subtitle, icon, active) = self.qs_tile_meta(kind);
                (title, subtitle, icon, active, unavailable, false)
            }
            QsTileKind::Flashlight => (
                "Flashlight".into(),
                if !self.sys.flashlight_available {
                    "Unavailable".into()
                } else if self.sys.flashlight_on {
                    "On".into()
                } else {
                    "Off".into()
                },
                SysIcon::Flashlight,
                self.sys.flashlight_on && self.sys.flashlight_available,
                !self.sys.flashlight_available,
                false,
            ),
            QsTileKind::Shiraz => {
                let subtitle = if !self.sys.shiraz_enabled {
                    "Enable Shiraz".to_owned()
                } else {
                    match self.sys.shiraz.state {
                        settings::ShirazRuntimeState::Ready => {
                            if self.sys.shiraz.warm_ram_mb > 0 {
                                format!("Warm · {} MB", self.sys.shiraz.warm_ram_mb)
                            } else {
                                "Ready".to_owned()
                            }
                        }
                        settings::ShirazRuntimeState::Warming => "Optimizing…".to_owned(),
                        settings::ShirazRuntimeState::Starting
                        | settings::ShirazRuntimeState::Disabled => "Starting…".to_owned(),
                        settings::ShirazRuntimeState::Error => "Error".to_owned(),
                    }
                };
                (
                    "Shiraz".into(),
                    subtitle,
                    SysIcon::System,
                    self.sys.shiraz_enabled,
                    false,
                    false,
                )
            }
            QsTileKind::Alarm => (
                "Alarm".into(),
                self.sys
                    .alarm_summary
                    .clone()
                    .unwrap_or_else(|| "No alarm set".into()),
                SysIcon::Alarm,
                self.sys.alarm_summary.is_some(),
                false,
                true,
            ),
            QsTileKind::Airplane => (
                "Airplane mode".into(),
                if self.sys.airplane_mode { "On" } else { "Off" }.into(),
                SysIcon::Airplane,
                self.sys.airplane_mode,
                false,
                false,
            ),
            QsTileKind::DeviceControls => (
                "Device controls".into(),
                "Unavailable".into(),
                SysIcon::DeviceControls,
                false,
                true,
                true,
            ),
            QsTileKind::AutoRotate => (
                "Auto-rotate".into(),
                if self.sys.auto_rotate { "On" } else { "Off" }.into(),
                SysIcon::AutoRotate,
                self.sys.auto_rotate,
                false,
                false,
            ),
            QsTileKind::BatterySaver => (
                "Battery Saver".into(),
                if self.sys.charging {
                    "Unavailable".into()
                } else if self.sys.battery_saver {
                    "On".into()
                } else {
                    "Off".into()
                },
                SysIcon::BatterySaver,
                self.sys.battery_saver && !self.sys.charging,
                self.sys.charging,
                false,
            ),
            QsTileKind::DarkTheme => (
                "Dark theme".into(),
                if self.sys.dark_theme { "On" } else { "Off" }.into(),
                SysIcon::DarkMode,
                self.sys.dark_theme,
                false,
                false,
            ),
            QsTileKind::Dnd => (
                "Do Not Disturb".into(),
                if self.sys.dnd_enabled { "On" } else { "Off" }.into(),
                SysIcon::Dnd,
                self.sys.dnd_enabled,
                false,
                false,
            ),
            QsTileKind::ScreenRecord => (
                "Screen record".into(),
                if self.sys.screen_record { "Recording" } else { "Start" }.into(),
                SysIcon::ScreenRecord,
                self.sys.screen_record,
                false,
                true,
            ),
            QsTileKind::NightLight => (
                "Night Light".into(),
                if self.sys.night_light { "On" } else { "Off" }.into(),
                SysIcon::NightLight,
                self.sys.night_light,
                false,
                false,
            ),
            QsTileKind::CameraAccess => (
                "Camera access".into(),
                if self.sys.camera_access { "Available" } else { "Blocked" }.into(),
                SysIcon::Camera,
                self.sys.camera_access,
                false,
                false,
            ),
            QsTileKind::MicAccess => (
                "Mic access".into(),
                if self.sys.mic_access { "Available" } else { "Blocked" }.into(),
                SysIcon::Mic,
                self.sys.mic_access,
                false,
                false,
            ),
            QsTileKind::Aod => (
                "AOD".into(),
                if self.sys.aod { "On" } else { "Off" }.into(),
                SysIcon::Aod,
                self.sys.aod,
                false,
                false,
            ),
            QsTileKind::Hotspot => (
                "Hotspot".into(),
                if self.sys.hotspot { "Active" } else { "Off" }.into(),
                SysIcon::Hotspot,
                self.sys.hotspot,
                false,
                false,
            ),
        };

        let bg = (if unavailable {
            p.unavailable
        } else if active {
            p.primary
        } else {
            p.card_elevated
        })
        .opacity(tile_alpha);
        let fg = (if unavailable {
            p.on_surface_variant.opacity(0.65)
        } else if active {
            p.on_primary
        } else {
            p.on_surface
        })
        .opacity(tile_alpha);
        let sub_fg = (if unavailable {
            p.on_surface_variant.opacity(0.55)
        } else if active {
            p.on_primary.opacity(0.82)
        } else {
            p.on_surface_variant
        })
        .opacity(tile_alpha);

        if interactive {
            match kind {
                QsTileKind::Wifi => self.last_radio_tile_rects[0] = Some(rect),
                QsTileKind::Bluetooth => self.last_radio_tile_rects[1] = Some(rect),
                _ => {}
            }
        }

        let r = f.target(id, rect, &title, interactive && !unavailable, Role::ToggleButton);
        f.paint_rect(rect, radius, bg);
        f.state_layer(r, radius, fg);

        let detail_alpha = smoothstep(20.0, 32.0, rect.height);
        let fg = fg.opacity(detail_alpha);
        wallpaper_ui::paint_wallpaper_system_icon(
            f,
            Rect {
                x: rect.x + 16.0,
                y: rect.y + (rect.height - 20.0) * 0.5,
                width: 20.0,
                height: 20.0,
            },
            icon,
            fg,
            self.sys.icon_style,
        );
        let text_w = rect.width - 46.0 - if has_chevron { 22.0 } else { 10.0 };
        let show_sub_frac = sub_alpha;
        let centered_y = rect.y + (rect.height - 18.0) * 0.5;
        let top_y = rect.y + rect.height * 0.5 - 17.0;
        f.styled_text(
            Rect {
                x: rect.x + 44.0,
                y: lerp(centered_y, top_y, show_sub_frac),
                width: text_w.max(20.0),
                height: 18.0,
            },
            &title,
            fg,
            text_style(13.5, 500, 0),
        );
        if show_sub_frac > 0.02 {
            f.styled_text(
                Rect {
                    x: rect.x + 44.0,
                    y: rect.y + rect.height * 0.5 + 1.0,
                    width: text_w.max(20.0),
                    height: 16.0,
                },
                &subtitle,
                sub_fg.opacity(show_sub_frac),
                text_style(12.0, 400, 0),
            );
        }
        if has_chevron {
            paint_sys_icon(
                f,
                Rect {
                    x: rect.x + rect.width - 24.0,
                    y: rect.y + (rect.height - 14.0) * 0.5,
                    width: 14.0,
                    height: 14.0,
                },
                SysIcon::ChevronRight,
                fg.opacity(0.8),
            );
        }

        if r.clicked {
            self.trigger_haptic(services::HapticEffect::Click);
            match kind {
                QsTileKind::Wifi => {
                    self.sys.wifi_enabled = !self.sys.wifi_enabled;
                    self.sys.wifi_error = None;
                    if !self.sys.wifi_enabled {
                        self.sys.wifi_connected = false;
                        self.sys.wifi_signal = 0;
                    }
                }
                QsTileKind::Bluetooth => {
                    self.sys.bluetooth_enabled = !self.sys.bluetooth_enabled;
                    self.sys.bluetooth_error = None;
                }
                QsTileKind::Flashlight => {
                    self.sys.flashlight_on = !self.sys.flashlight_on;
                    self.show_toast(
                        if self.sys.flashlight_on {
                            "Flashlight on"
                        } else {
                            "Flashlight off"
                        },
                        now,
                    );
                }
                QsTileKind::Shiraz => {
                    self.sys.shiraz_enabled = !self.sys.shiraz_enabled;
                    self.sys.save_persisted_settings();
                    self.show_toast(
                        if self.sys.shiraz_enabled {
                            "Enabling Shiraz & optimizing memory…"
                        } else {
                            "Disabling Shiraz"
                        },
                        now,
                    );
                }
                QsTileKind::Alarm => self.open_app(AppId::Clock, now),
                QsTileKind::Airplane => {
                    self.sys.set_airplane_mode(!self.sys.airplane_mode);
                }
                QsTileKind::AutoRotate => self.sys.auto_rotate = !self.sys.auto_rotate,
                QsTileKind::BatterySaver => self.sys.battery_saver = !self.sys.battery_saver,
                QsTileKind::DarkTheme => self.sys.dark_theme = !self.sys.dark_theme,
                QsTileKind::Dnd => self.sys.dnd_enabled = !self.sys.dnd_enabled,
                QsTileKind::ScreenRecord => {
                    self.sys.screen_record = !self.sys.screen_record;
                    self.show_toast(
                        if self.sys.screen_record {
                            "Screen recording started"
                        } else {
                            "Screen recording saved"
                        },
                        now,
                    );
                }
                QsTileKind::NightLight => self.sys.night_light = !self.sys.night_light,
                QsTileKind::CameraAccess => self.sys.camera_access = !self.sys.camera_access,
                QsTileKind::MicAccess => self.sys.mic_access = !self.sys.mic_access,
                QsTileKind::Aod => self.sys.aod = !self.sys.aod,
                QsTileKind::Hotspot => self.sys.hotspot = !self.sys.hotspot,
                QsTileKind::DeviceControls => {}
            }
        }
    }

    fn notif_swipe_offset(&self, idx: usize) -> f32 {
        let n = &self.notifications[idx];
        match self.gesture {
            ActiveGesture::NotifSwipe {
                notif_id,
                offset_x,
                ..
            } if notif_id == n.id => offset_x,
            _ => self
                .preview_notif_swipe
                .filter(|(id, _)| *id == n.id)
                .map_or(n.swipe_anim.value, |(_, dx)| dx),
        }
    }

    fn render_notifications_list(&mut self, f: &mut Frame<'_>, area: Rect, now: f64) {
        let p = self.sys.palette();
        let inner = Rect {
            x: area.x + 16.0,
            y: area.y + 16.0,
            width: area.width - 32.0,
            height: (area.height - 22.0).max(100.0),
        };

        // Measure total height (including smoothly collapsing dismissed slots!)
        let mut total_h = 0.0_f32;
        let active_notifs: Vec<usize> = self
            .notifications
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.snoozed && !n.silent)
            .map(|(i, _)| i)
            .collect();
        let silent_notifs: Vec<usize> = self
            .notifications
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.snoozed && n.silent)
            .map(|(i, _)| i)
            .collect();

        for (pos, &idx) in active_notifs.iter().enumerate() {
            let n = &self.notifications[idx];
            total_h += n.shade_card_advance(n.shade_card_height(pos == 0));
        }
        if !silent_notifs.is_empty() {
            total_h += 38.0;
            for &idx in &silent_notifs {
                let n = &self.notifications[idx];
                total_h += n.shade_card_advance(n.shade_card_height(false));
            }
        }
        total_h += 76.0; // Manage + Clear all pills

        let max_s = (total_h - inner.height).max(0.0);
        self.last_scroll_max[1] = max_s;
        self.shade_scroll = self.shade_scroll.clamp(0.0, max_s);
        f.scroll_viewport(
            SHADE_ID + 100,
            inner,
            &mut self.shade_scroll,
            max_s,
        );

        let mut remove_ids: Vec<u32> = Vec::new();
        let mut invoke_action: Option<(u32, String, Rect)> = None;
        let mut jump_app: Option<(AppId, Rect)> = None;
        let mut jump_manage = false;
        let mut clear_all = false;
        let mut toast_msg: Option<String> = None;

        // Precompute detach progress per group item so neighboring cards smoothly round touching corners (`strip-v4-notif-dismiss.png`)!
        let active_detach: Vec<f32> = active_notifs
            .iter()
            .map(|&idx| {
                if self.notifications[idx].dismissing {
                    1.0
                } else {
                    smoothstep(0.0, 24.0, self.notif_swipe_offset(idx).abs())
                }
            })
            .collect();
        let silent_detach: Vec<f32> = silent_notifs
            .iter()
            .map(|&idx| {
                if self.notifications[idx].dismissing {
                    1.0
                } else {
                    smoothstep(0.0, 24.0, self.notif_swipe_offset(idx).abs())
                }
            })
            .collect();

        let (ns_s, ns_sign) = self.target_stretch(ScrollTarget::ShadeNotifs);
        f.within(inner, |f| {
            f.with_edge_stretch(inner, false, ns_s, ns_sign, |f| {
            let mut y = inner.y - self.shade_scroll;
            if active_notifs.is_empty() && silent_notifs.is_empty() {
                f.styled_text(
                    Rect {
                        x: inner.x,
                        y: y + 48.0,
                        width: inner.width,
                        height: 26.0,
                    },
                    "No notifications",
                    p.on_surface_variant,
                    text_style(15.0, 500, 1),
                );
                y += 120.0;
            } else {
                let active_len = active_notifs.len();
                for (pos, &idx) in active_notifs.iter().enumerate() {
                    let prev_d = if pos > 0 { active_detach[pos - 1] } else { 0.0 };
                    let next_d = if pos + 1 < active_len {
                        active_detach[pos + 1]
                    } else {
                        0.0
                    };
                    let (top_r, bot_r) = bhoo::segmented_card_radii(
                        pos,
                        active_len,
                        28.0,
                        4.0,
                        prev_d,
                        active_detach[pos],
                        next_d,
                    );
                    let adv = self.render_single_notif(
                        f,
                        idx,
                        Rect {
                            x: inner.x,
                            y,
                            width: inner.width,
                            height: 0.0,
                        },
                        top_r,
                        bot_r,
                        pos == 0,
                        now,
                        &p,
                        &mut remove_ids,
                        &mut invoke_action,
                        &mut jump_app,
                        &mut toast_msg,
                    );
                    y += adv;
                }
                if !silent_notifs.is_empty() {
                    y += 10.0;
                    f.styled_text(
                        Rect {
                            x: inner.x + 8.0,
                            y,
                            width: 120.0,
                            height: 22.0,
                        },
                        "Silent",
                        p.on_surface,
                        text_style(13.5, 500, 0),
                    );
                    y += 28.0;
                    let silent_len = silent_notifs.len();
                    for (pos, &idx) in silent_notifs.iter().enumerate() {
                        let prev_d = if pos > 0 { silent_detach[pos - 1] } else { 0.0 };
                        let next_d = if pos + 1 < silent_len {
                            silent_detach[pos + 1]
                        } else {
                            0.0
                        };
                        let (top_r, bot_r) = bhoo::segmented_card_radii(
                            pos,
                            silent_len,
                            28.0,
                            4.0,
                            prev_d,
                            silent_detach[pos],
                            next_d,
                        );
                        let adv = self.render_single_notif(
                            f,
                            idx,
                            Rect {
                                x: inner.x,
                                y,
                                width: inner.width,
                                height: 0.0,
                            },
                            top_r,
                            bot_r,
                            false,
                            now,
                            &p,
                            &mut remove_ids,
                            &mut invoke_action,
                            &mut jump_app,
                            &mut toast_msg,
                        );
                        y += adv;
                    }
                }
            }

            // Bottom "Manage" and "Clear all" pills matching 02-shade-populated.png
            y += 14.0;
            let manage_r = Rect {
                x: inner.x + 8.0,
                y,
                width: 96.0,
                height: 40.0,
            };
            let rm = f.target(SHADE_ID + 101, manage_r, "Manage notifications", true, Role::Button);
            f.paint_rect(manage_r, 20.0, p.card_elevated);
            f.state_layer(rm, 20.0, p.on_surface);
            f.styled_text(manage_r, "Manage", p.on_surface, text_style(13.5, 500, 1));
            if rm.clicked {
                jump_manage = true;
            }

            let has_dismissible = self
                .notifications
                .iter()
                .any(|n| !n.ongoing && !n.snoozed && !n.dismissing);
            if has_dismissible {
                let clr_r = Rect {
                    x: inner.x + inner.width - 104.0,
                    y,
                    width: 96.0,
                    height: 40.0,
                };
                let rc = f.target(SHADE_ID + 102, clr_r, "Clear all", true, Role::Button);
                f.paint_rect(clr_r, 20.0, p.card_elevated);
                f.state_layer(rc, 20.0, p.on_surface);
                f.styled_text(clr_r, "Clear all", p.on_surface, text_style(13.5, 500, 1));
                if rc.clicked {
                    clear_all = true;
                }
            }
            });
        });

        for id in remove_ids {
            self.dismiss_notification(id, notifications::CLOSE_REASON_DISMISSED);
        }
        if clear_all {
            self.clear_all_dismissible_notifications(now);
        }
        if let Some(msg) = toast_msg {
            self.show_toast(msg, now);
        }
        if jump_manage {
            self.settings_app
                .navigate(settings::SettingsPage::Notifications, now);
            self.open_app(AppId::Settings, now);
        } else if let Some((id, key, card_rect)) = invoke_action {
            self.invoke_notification_action(id, &key, Some(card_rect), now);
        } else if let Some((app, card_rect)) = jump_app {
            if app == AppId::Settings {
                self.settings_app
                    .navigate(settings::SettingsPage::DevOptions, now);
            }
            // Authentic Android 13 NotificationLaunchAnimator: expands THIS card right over the shade!
            self.open_app_from(app, Some(card_rect), false, true, false, now);
        }
    }

    fn heads_up_index(&self, now: f64) -> Option<usize> {
        if self.sys.dnd_enabled || self.lockscreen.locked || self.shade_stage != 0 || self.power_open {
            return None;
        }
        self.notifications.iter().position(|n| {
            !n.snoozed && !n.dismissing && n.heads_up_until.is_some_and(|until| now < until + HEADS_UP_EXIT_SECONDS * self.sys.anim_scale() as f64)
        })
    }

    fn render_heads_up_source(&self, f: &mut Frame<'_>, bar: Rect, idx: usize, ink: Color) {
        let notification = &self.notifications[idx];
        let badge = Rect {
            x: bar.x + 22.0,
            y: bar.y + (bar.height - 14.0) * 0.5,
            width: 14.0,
            height: 14.0,
        };
        if let Some(image) = &notification.icon_image {
            f.paint_image(badge, image, ImageFit::Contain, 7.0);
        } else {
            f.paint_rect(badge, 7.0, ink);
            let detail = if ink.0 > 127 { Color(46, 46, 46, 255) } else { Color(250, 250, 250, 255) };
            f.paint_rect(Rect { x: badge.x + 6.5, y: badge.y + 4.0, width: 1.0, height: 1.0 }, 0.5, detail);
            f.paint_rect(Rect { x: badge.x + 6.5, y: badge.y + 6.0, width: 1.0, height: 4.0 }, 0.5, detail);
        }
        f.styled_text(
            Rect { x: badge.x + 20.0, width: (bar.width - 170.0).max(0.0), ..bar },
            &notification.app,
            ink,
            text_style(10.0, 400, 0),
        );
    }

    /// Paint the production heads-up banner alone in a desktop preview.
    #[cfg(any(feature = "wayland", target_arch = "wasm32"))]
    pub fn show_notification_preview(&mut self, f: &mut Frame<'_>, settled: bool, expanded: bool) {
        let now = f.input().time;
        self.lockscreen.locked = false;
        self.sys.dnd_enabled = false;
        if self.notifications.is_empty() {
            self.push_freedesktop_notification(
                notifications::IncomingNotification {
                    id: 1,
                    replaces_id: 0,
                    app_name: "SpectreOS Test".into(),
                    app_icon: String::new(),
                    summary: "SpectreOS test".into(),
                    body: "This is a heads-up notification".into(),
                    actions: Vec::new(),
                    urgency: 1,
                    category: None,
                    desktop_entry: None,
                    resident: false,
                    transient: false,
                    suppress_sound: false,
                    image_data: None,
                    image_path: None,
                    icon_data: None,
                    expire_timeout_ms: 0,
                },
                if settled { now - 0.5 * self.sys.anim_scale() as f64 } else { now },
            );
            self.notifications[0].expanded = expanded;
        }
        if let Some(notification) = self.notifications.first_mut() {
            notification.heads_up_until = Some(f64::INFINITY);
        }
        let viewport = f.viewport();
        let phone = Rect { height: viewport.width * 2960.0 / 1440.0, ..viewport };
        f.within(viewport, |f| self.wallpapers.home.paint(f, phone, self.sys.wallpaper_idx));
        if let Some(idx) = self.heads_up_index(now) {
            self.render_heads_up_source(f, Rect { height: 30.0, ..viewport }, idx, Color(255, 255, 255, 255));
        }
        self.render_heads_up_banner(f, viewport, 38.0, now);
    }

    /// Floating Android 13 Material 3 Heads-Up Notification Banner shown when unlocked and shade is closed.
    fn render_heads_up_banner(&mut self, f: &mut Frame<'_>, phone: Rect, top_y: f32, now: f64) {
        let Some(idx) = self.heads_up_index(now) else { return };
        let surface = Color(46, 46, 46, 255);
        let foreground = Color(246, 242, 250, 255);
        let secondary = Color(212, 210, 215, 255);
        let n = self.notifications[idx].clone();
        let exit_at = n.heads_up_until.unwrap_or(f64::INFINITY);
        if now < exit_at {
            f.wake_at(exit_at);
        }
        let visible_actions: Vec<(String, String)> = n
            .actions
            .iter()
            .filter(|(k, l)| k != "default" && !l.trim().is_empty())
            .take(2)
            .cloned()
            .collect();
        let h = if visible_actions.is_empty() { 86.0 } else { 118.0 };
        let extra_body_height = if n.expanded { 32.0 } else { 0.0 };
        let card = Rect {
            x: phone.x + 15.0,
            y: phone.y + top_y,
            width: phone.width - 30.0,
            height: h + extra_body_height,
        };
        // The banner grows from its top edge before its contents fade in.
        const EXPAND_SECONDS: f64 = 0.20;
        const FADE_START_SECONDS: f64 = 0.15;
        const ENTRANCE_SECONDS: f64 = 0.47;
        let elapsed = (now - n.arrived_at).max(0.0);
        let duration_scale = self.sys.anim_scale() as f64;
        let motion_off = f.reduced_motion() || duration_scale == 0.0;
        let elapsed = if motion_off { ENTRANCE_SECONDS } else { elapsed / duration_scale };
        if !motion_off && (elapsed < ENTRANCE_SECONDS || now >= exit_at) {
            // Keep this frame chain on DRM VBlank, rather than the idle watchdog.
            f.continuous(card);
        }
        let expansion = (elapsed / EXPAND_SECONDS).clamp(0.0, 1.0) as f32;
        let expansion = expansion * expansion * (3.0 - 2.0 * expansion);
        let fade = ((elapsed - FADE_START_SECONDS) / (ENTRANCE_SECONDS - FADE_START_SECONDS))
            .clamp(0.0, 1.0) as f32;
        // Android retracts sharply on exit and holds its last thin edge briefly.
        if motion_off && now >= exit_at {
            return;
        }
        let exit_elapsed = if motion_off { 0.0 } else { (now - exit_at).max(0.0) / duration_scale };
        let retract = (1.0 - (exit_elapsed / HEADS_UP_EXIT_SECONDS).clamp(0.0, 1.0)).powi(3) as f32;
        let content_exit = (1.0 - exit_elapsed / 0.06).clamp(0.0, 1.0) as f32;
        let fade = fade * content_exit;
        let revealed = Rect { height: card.height * expansion * retract, ..card };
        if revealed.height <= 0.0 {
            return;
        }

        self.last_heads_up_rect = Some(revealed);
        let radius = 27.0_f32.min(revealed.height * 0.5);
        f.paint_rect(revealed, radius, surface);
        if fade <= 0.0 {
            return;
        }

        let mut clicked_action: Option<String> = None;
        let mut toggle_clicked = false;
        let mut body_clicked = false;
        f.within(revealed, |f| {
            let interactive = elapsed >= EXPAND_SECONDS && now < exit_at;
            let foreground = foreground.opacity(fade);
            let secondary = secondary.opacity(fade);
            if n.urgency == 2 {
                f.paint_rect(
                    Rect {
                        x: card.x + 5.0,
                        y: card.y + 14.0,
                        width: 3.5,
                        height: card.height - 28.0,
                    },
                    2.0,
                    Color(0xD9, 0x30, 0x25, 255).opacity(fade),
                );
            }

            let chevron = Rect {
                x: card.x + card.width - 40.0,
                y: card.y + (86.0 - 24.0) * 0.5,
                width: 24.0,
                height: 24.0,
            };
            let chevron_label = if n.expanded { "Collapse notification" } else { "Expand notification" };
            let toggle = f.target(SHADE_ID + 9901, chevron.inset(-6.0), chevron_label, interactive, Role::Button);
            f.paint_rect(chevron, 12.0, Color(25, 25, 25, 255).opacity(fade));
            f.state_layer(toggle, 12.0, foreground);
            let icon = if n.expanded { SysIcon::ChevronUp } else { SysIcon::ChevronDown };
            paint_sys_icon(f, chevron, icon, foreground);

            let body_hit = Rect {
                x: card.x,
                y: card.y,
                width: card.width - 46.0,
                height: 86.0,
            };
            let body_label = format!("{}: {}", n.app, n.title);
            let r_body = f.target(SHADE_ID + 9900, body_hit, &body_label, interactive, Role::Button);
            f.state_layer(r_body, 27.0, foreground);

            let badge = Rect {
                x: card.x + 13.0,
                y: card.y + 28.0,
                width: 30.0,
                height: 30.0,
            };
            if let Some(ref img) = n.icon_image {
                f.paint_tinted_image(badge, img, ImageFit::Cover, 10.0, Color(255, 255, 255, 255).opacity(fade));
            } else {
                paint_sys_icon(f, badge, SysIcon::Info, Color(234, 225, 250, 255).opacity(fade));
            }

            let tx = card.x + 51.0;
            let tw = (card.width - 103.0).max(80.0);
            let title_style = text_style(12.0, 500, 0);
            let title_width = if n.silent { tw } else { (tw - 16.0).max(0.0) };
            f.styled_text(
                Rect { x: tx, y: card.y + 23.0, width: title_width, height: 18.0 },
                &n.title,
                foreground,
                title_style,
            );
            if !n.silent {
                let x = tx + f.text_width(&n.title, title_style).min(title_width) + 4.0;
                paint_sys_icon(f, Rect { x, y: card.y + 25.5, width: 12.0, height: 12.0 }, SysIcon::NotificationsActive, foreground);
            }
            let body_style = text_style(12.0, 400, 0);
            let mut lines = Vec::new();
            if n.expanded {
                let mut line = String::new();
                for word in n.big_text.split_whitespace() {
                    let candidate = if line.is_empty() { word.to_owned() } else { format!("{line} {word}") };
                    if !line.is_empty() && f.text_width(&candidate, body_style) > tw {
                        lines.push(std::mem::replace(&mut line, word.to_owned()));
                        if lines.len() == 3 {
                            let last = &mut lines[2];
                            while !last.is_empty() && f.text_width(&format!("{last}…"), body_style) > tw {
                                last.pop();
                            }
                            last.push('…');
                            break;
                        }
                    } else {
                        line = candidate;
                    }
                }
                if lines.len() < 3 && !line.is_empty() {
                    lines.push(line);
                }
            } else if !n.body.is_empty() {
                lines.push(n.body.clone());
            }
            for (i, line) in lines.iter().enumerate() {
                f.styled_text(
                    Rect { x: tx, y: card.y + 46.0 + i as f32 * 16.0, width: tw, height: 18.0 },
                    line,
                    secondary,
                    body_style,
                );
            }

            if !visible_actions.is_empty() {
                let mut ax = tx;
                let ay = card.y + 80.0 + extra_body_height;
                for (ai, (key, label)) in visible_actions.iter().enumerate() {
                    let bw = (f.text_width(label, text_style(12.5, 600, 1)) + 22.0).clamp(64.0, 132.0);
                    let br = Rect { x: ax, y: ay, width: bw, height: 28.0 };
                    let rb = f.target(SHADE_ID + 9910 + ai as u64, br, label, interactive, Role::Button);
                    f.paint_rect(br, 14.0, Color(73, 70, 78, 255).opacity(fade));
                    f.state_layer(rb, 14.0, foreground);
                    f.styled_text(br, label, foreground, text_style(12.5, 600, 1));
                    if rb.clicked {
                        clicked_action = Some(key.clone());
                    }
                    ax += bw + 8.0;
                }
            }

            toggle_clicked = toggle.clicked;
            body_clicked = r_body.clicked;
        });

        if let Some(key) = clicked_action {
            self.invoke_notification_action(n.id, &key, Some(card), now);
        } else if toggle_clicked {
            let slot = &mut self.notifications[idx];
            slot.expanded = !slot.expanded;
            slot.expand_anim.set(if slot.expanded { 1.0 } else { 0.0 }, now, 0.24, 0.20);
            let deadline = slot.heads_up_until.unwrap_or(now).max(now + 4.8);
            slot.heads_up_until = Some(slot.expires_at.map_or(deadline, |expiry| expiry.min(deadline)));
        } else if body_clicked {
            self.invoke_notification_action(n.id, "default", Some(card), now);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_single_notif(
        &mut self,
        f: &mut Frame<'_>,
        idx: usize,
        origin: Rect,
        top_rad: f32,
        bot_rad: f32,
        prominent: bool,
        now: f64,
        p: &Palette,
        remove_ids: &mut Vec<u32>,
        invoke_action: &mut Option<(u32, String, Rect)>,
        jump_app: &mut Option<(AppId, Rect)>,
        toast_msg: &mut Option<String>,
    ) -> f32 {
        let notif_id = self.notifications[idx].id;
        let notif_ongoing = self.notifications[idx].ongoing;
        let notif_dismissing = self.notifications[idx].dismissing;
        let swipe_x = self.notif_swipe_offset(idx);
        let n = &mut self.notifications[idx];
        let base = SHADE_ID + 200 + (n.id as u64) * 10;
        let exp_p = n.expand_anim.value.clamp(0.0, 1.0);
        let ctrl_p = n.controls_anim.value.clamp(0.0, 1.0);
        let col_p = if notif_dismissing {
            n.dismiss_anim.value.clamp(0.0, 1.0)
        } else {
            1.0
        };
        let h = n.shade_card_height(prominent);
        let adv = n.shade_card_advance(h);
        let slot = Rect { height: h, ..origin };
        if !notif_dismissing {
            self.last_notif_rects.push((notif_id, notif_ongoing, slot));
        }

        // Authentic Android 13 swipe-to-dismiss (`strip-v4-notif-dismiss.png`):
        // ZERO grey box or trash can behind swiped card — pure clean linear fade + rounded detachment,
        // followed by smooth slot height collapse (`col_p: 1 -> 0`) so lower cards glide seamlessly upward!
        let card_alpha = if notif_ongoing {
            1.0
        } else {
            (1.0 - (swipe_x.abs() / (slot.width.max(120.0) * 0.72)).clamp(0.0, 1.0)) * col_p
        };
        if card_alpha <= 0.01 {
            return adv;
        }
        let card = Rect {
            x: slot.x + swipe_x,
            ..slot
        };
        if card_alpha < 0.999 {
            f.push_layer(card, card_alpha);
        }
        paint_asym_rect(f, card, top_rad, bot_rad, p.card_elevated);

        // Full-card bounded Android Material radial ink ripple from exact touch down coordinate!
        if let Some(([rx, ry], t_start, held)) = n.ripple {
            let elapsed = (now - t_start).max(0.0) as f32;
            let grow = emphasized_decelerate((elapsed / 0.28).clamp(0.0, 1.0));
            let max_r = card.width.hypot(card.height);
            let rad = lerp(24.0, max_r, grow);
            let fade = if held {
                1.0
            } else {
                1.0 - smoothstep(0.08, 0.42, elapsed)
            };
            if fade > 0.01 {
                let ink_c = p.on_surface.opacity(0.14 * fade);
                f.paint_ink(card, top_rad.min(bot_rad).max(8.0), [rx, ry], rad, ink_c);
            }
        }

        if n.controls_open || ctrl_p > 0.45 {
            // Inline notification controls matching 04-notification-longpress-controls.png!
            f.within(card, |f| {
                if ctrl_p < 0.995 {
                    f.push_layer(card, ctrl_p);
                }
                f.paint_rect(
                    Rect {
                        x: card.x + 14.0,
                        y: card.y + 16.0,
                        width: 34.0,
                        height: 34.0,
                    },
                    17.0,
                    p.primary,
                );
                paint_sys_icon(
                    f,
                    Rect {
                        x: card.x + 22.0,
                        y: card.y + 24.0,
                        width: 18.0,
                        height: 18.0,
                    },
                    SysIcon::Bell,
                    p.on_primary,
                );
                f.styled_text(
                    Rect {
                        x: card.x + 58.0,
                        y: card.y + 14.0,
                        width: card.width - 74.0,
                        height: 20.0,
                    },
                    &n.title,
                    p.on_surface,
                    text_style(14.0, 600, 0),
                );
                f.styled_text(
                    Rect {
                        x: card.x + 58.0,
                        y: card.y + 34.0,
                        width: card.width - 74.0,
                        height: 18.0,
                    },
                    &n.app,
                    p.on_surface_variant,
                    text_style(12.0, 400, 0),
                );
                let btn_y = card.y + 74.0;
                let b_snooze = Rect {
                    x: card.x + 14.0,
                    y: btn_y,
                    width: 96.0,
                    height: 34.0,
                };
                let rs = f.target(base + 1, b_snooze, "Snooze 1h", !notif_dismissing, Role::Button);
                f.paint_rect(b_snooze, 17.0, p.surface_container);
                f.styled_text(b_snooze, "Snooze 1h", p.on_surface, text_style(12.5, 500, 1));
                if rs.clicked {
                    n.snoozed = true;
                    *toast_msg = Some("Snoozed for 1 hour".into());
                }

                let b_mute = Rect {
                    x: card.x + 118.0,
                    y: btn_y,
                    width: 92.0,
                    height: 34.0,
                };
                let rm = f.target(base + 2, b_mute, "Turn off", !notif_dismissing, Role::Button);
                f.paint_rect(b_mute, 17.0, p.surface_container);
                f.styled_text(b_mute, "Silent", p.on_surface, text_style(12.5, 500, 1));
                if rm.clicked {
                    n.silent = !n.silent;
                    n.controls_open = false;
                    n.controls_anim.set(0.0, now, 0.22, 0.18);
                }

                let b_done = Rect {
                    x: card.x + card.width - 76.0,
                    y: btn_y,
                    width: 64.0,
                    height: 34.0,
                };
                let rd = f.target(base + 3, b_done, "Done", !notif_dismissing, Role::Button);
                f.styled_text(b_done, "Done", p.primary, text_style(13.5, 600, 1));
                if rd.clicked {
                    n.controls_open = false;
                    n.controls_anim.set(0.0, now, 0.22, 0.18);
                }
                if ctrl_p < 0.995 {
                    f.pop_layer();
                }
            });
            if card_alpha < 0.999 {
                f.pop_layer();
            }
            return adv;
        }

        // Expand/collapse chevron + explicit dismiss × on right
        let chev_r = Rect {
            x: card.x + card.width - 38.0,
            y: card.y + 12.0,
            width: 28.0,
            height: 28.0,
        };
        let r_chev = f.target(base + 4, chev_r, "Expand notification", !notif_dismissing, Role::Button);
        f.paint_rect(chev_r, 14.0, p.surface_container);
        f.state_layer(r_chev, 14.0, p.on_surface);
        f.push_transform(
            [chev_r.x + chev_r.width * 0.5, chev_r.y + chev_r.height * 0.5],
            [0.0, 0.0],
            [1.0, 1.0],
            std::f32::consts::PI * exp_p,
        );
        paint_sys_icon(f, chev_r.inset(7.0), SysIcon::ChevronDown, p.on_surface);
        f.pop_transform();

        let mut quick_dismiss_clicked = false;
        if !notif_ongoing {
            let dis_x = Rect {
                x: card.x + card.width - 70.0,
                y: card.y + 12.0,
                width: 28.0,
                height: 28.0,
            };
            let r_dx = f.target(base + 9, dis_x, "Dismiss", !notif_dismissing, Role::Button);
            f.state_layer(r_dx, 14.0, p.on_surface);
            paint_sys_icon(f, dis_x.inset(7.0), SysIcon::Close, p.on_surface_variant);
            quick_dismiss_clicked = r_dx.clicked;
        }

        let body_hit = Rect {
            x: card.x,
            y: card.y,
            width: card.width - if notif_ongoing { 44.0 } else { 76.0 },
            height: if prominent { 108.0 } else { 86.0 },
        };
        let r_card = f.target(base, body_hit, &n.title, !notif_dismissing, Role::Button);
        if r_card.hovered || r_card.pressed {
            let hl_a = if r_card.pressed { 0.12 } else { 0.06 };
            paint_asym_rect(f, card, top_rad, bot_rad, p.on_surface.opacity(hl_a));
        }

        if n.urgency == 2 {
            f.paint_rect(
                Rect {
                    x: card.x + 4.0,
                    y: card.y + 14.0,
                    width: 3.5,
                    height: card.height - 28.0,
                },
                2.0,
                Color(0xD9, 0x30, 0x25, 255),
            );
        }

        // Leading icon badge: real Freedesktop image-data / icon-theme image OR crisp Material SystemIcon
        let badge = Rect {
            x: card.x + 14.0,
            y: card.y + 18.0,
            width: 32.0,
            height: 32.0,
        };
        if let Some(ref img) = n.icon_image {
            f.paint_image(badge, img, ImageFit::Cover, 10.0);
        } else {
            let badge_col = if n.urgency == 2 {
                Color(0xD9, 0x30, 0x25, 255)
            } else {
                p.primary
            };
            f.paint_rect(badge, 16.0, badge_col);
            paint_sys_icon(
                f,
                badge.inset(7.5),
                lockscreen::notif_category_icon(n),
                p.on_primary,
            );
        }

        // App header + timestamp
        let header_w = (card.width - if notif_ongoing { 104.0 } else { 134.0 }).max(80.0);
        let title_y = if prominent { 50.0 } else { 34.0 };
        f.styled_text(
            Rect {
                x: card.x + 54.0,
                y: card.y + 19.0,
                width: header_w,
                height: 16.0,
            },
            &format!("{} · {}", n.app, n.time_label),
            p.on_surface_variant,
            text_style(11.5, 500, 0),
        );
        f.styled_text(
            Rect {
                x: card.x + 54.0,
                y: card.y + title_y,
                width: header_w,
                height: 18.0,
            },
            &n.title,
            p.on_surface,
            text_style(13.5, 600, 0),
        );
        let shown_body = if exp_p > 0.35 { &n.big_text } else { &n.body };
        f.styled_text(
            Rect {
                x: card.x + 54.0,
                y: card.y + title_y + 21.0,
                width: card.width - 70.0,
                height: lerp(18.0, 32.0, exp_p),
            },
            shown_body,
            p.on_surface_variant,
            text_style(12.5, 400, 0),
        );

        if quick_dismiss_clicked {
            remove_ids.push(n.id);
        } else if r_chev.clicked {
            n.expanded = !n.expanded;
            n.expand_anim
                .set(if n.expanded { 1.0 } else { 0.0 }, now, 0.24, 0.20);
        } else if r_card.clicked {
            if n.from_dbus {
                *invoke_action = Some((n.id, "default".to_string(), card));
            } else if let Some(app) = n.target_app {
                *jump_app = Some((app, card));
            } else {
                n.expanded = !n.expanded;
                n.expand_anim
                    .set(if n.expanded { 1.0 } else { 0.0 }, now, 0.24, 0.20);
            }
        }

        if exp_p > 0.15 {
            f.within(card, |f| {
                let act_a = smoothstep(0.20, 0.85, exp_p);
                if act_a < 0.995 {
                    f.push_layer(card, act_a);
                }
                let act_y = card.y + 112.0;
                let mut ax = card.x + 54.0;
                // Render real Freedesktop Notification Actions (excluding "default")
                let named_actions: Vec<(String, String)> = n
                    .actions
                    .iter()
                    .filter(|(k, l)| k != "default" && !l.trim().is_empty())
                    .take(2)
                    .cloned()
                    .collect();
                if !named_actions.is_empty() {
                    for (ai, (key, label)) in named_actions.iter().enumerate() {
                        let bw = (f.text_width(label, text_style(12.5, 600, 0)) + 16.0)
                            .clamp(56.0, 118.0);
                        let rr = Rect {
                            x: ax,
                            y: act_y,
                            width: bw,
                            height: 28.0,
                        };
                        let b = f.target(
                            base + 100 + ai as u64,
                            rr,
                            label,
                            n.expanded && !notif_dismissing,
                            Role::Button,
                        );
                        f.state_layer(b, 14.0, p.primary);
                        f.styled_text(rr, label, p.primary, text_style(13.0, 600, 0));
                        if b.clicked {
                            *invoke_action = Some((n.id, key.clone(), card));
                        }
                        ax += bw + 6.0;
                    }
                } else if n.has_reply {
                    let rr = Rect {
                        x: ax,
                        y: act_y,
                        width: 66.0,
                        height: 28.0,
                    };
                    let b = f.target(base + 5, rr, "Reply", n.expanded && !notif_dismissing, Role::Button);
                    f.styled_text(rr, "Reply", p.primary, text_style(13.0, 600, 0));
                    if b.clicked {
                        *jump_app = Some((AppId::ArcaneChat, card));
                    }
                    ax += 72.0;
                }
                let r_opt = Rect {
                    x: ax,
                    y: act_y,
                    width: 72.0,
                    height: 28.0,
                };
                let bo = f.target(base + 6, r_opt, "Controls", n.expanded && !notif_dismissing, Role::Button);
                f.styled_text(r_opt, "Options", p.primary, text_style(13.0, 600, 0));
                if bo.clicked {
                    n.controls_open = true;
                    n.controls_anim.set(1.0, now, 0.24, 0.18);
                }
                if !n.ongoing {
                    let r_dis = Rect {
                        x: card.x + card.width - 80.0,
                        y: act_y,
                        width: 68.0,
                        height: 28.0,
                    };
                    let bd = f.target(base + 7, r_dis, "Dismiss", n.expanded && !notif_dismissing, Role::Button);
                    f.styled_text(r_dis, "Dismiss", p.on_surface_variant, text_style(13.0, 500, 2));
                    if bd.clicked {
                        remove_ids.push(n.id);
                    }
                }
                if act_a < 0.995 {
                    f.pop_layer();
                }
            });
        }
        if card_alpha < 0.999 {
            f.pop_layer();
        }
        adv
    }

    fn qs_tile_meta(&self, kind: QsTileKind) -> (String, String, SysIcon, bool) {
        match kind {
            QsTileKind::Wifi => {
                let on = self.sys.wifi_available && self.sys.wifi_enabled;
                let connected = on && self.sys.wifi_connected;
                let title = if connected { self.sys.wifi_name.clone() } else { "Wi-Fi".into() };
                let subtitle = if !self.sys.wifi_available { "Unavailable".into() }
                    else if connected && self.sys.wifi_signal > 0 { format!("Connected · {}%", self.sys.wifi_signal) }
                    else if connected { "Connected".into() }
                    else if on { "Not connected".into() } else { "Off".into() };
                (title, subtitle, if on { SysIcon::Wifi } else { SysIcon::WifiOff }, on)
            }
            QsTileKind::Bluetooth => {
                let on = self.sys.bluetooth_available && self.sys.bluetooth_enabled;
                let subtitle = if !self.sys.bluetooth_available { "Unavailable".into() }
                    else if on { self.sys.bluetooth_device.clone().unwrap_or_else(|| "On".into()) }
                    else { "Off".into() };
                ("Bluetooth".into(), subtitle, SysIcon::Bluetooth, on)
            }
            QsTileKind::Flashlight => (
                "Flashlight".into(),
                if self.sys.flashlight_on { "On" } else { "Off" }.into(),
                SysIcon::Flashlight,
                self.sys.flashlight_on,
            ),
            QsTileKind::Shiraz => (
                "Shiraz".into(),
                if !self.sys.shiraz_enabled {
                    "Enable Shiraz".into()
                } else {
                    self.sys.shiraz.state.label().into()
                },
                SysIcon::System,
                self.sys.shiraz_enabled,
            ),
            QsTileKind::Alarm => (
                "Alarm".into(),
                self.sys
                    .alarm_summary
                    .clone()
                    .unwrap_or_else(|| "No alarm set".into()),
                SysIcon::Alarm,
                self.sys.alarm_summary.is_some(),
            ),
            QsTileKind::Airplane => (
                "Airplane mode".into(),
                if self.sys.airplane_mode { "On" } else { "Off" }.into(),
                SysIcon::Airplane,
                self.sys.airplane_mode,
            ),
            QsTileKind::DeviceControls => (
                "Device controls".into(),
                "Unavailable".into(),
                SysIcon::DeviceControls,
                false,
            ),
            QsTileKind::AutoRotate => (
                "Auto-rotate".into(),
                if self.sys.auto_rotate { "On" } else { "Off" }.into(),
                SysIcon::AutoRotate,
                self.sys.auto_rotate,
            ),
            QsTileKind::BatterySaver => (
                "Battery Saver".into(),
                if self.sys.charging {
                    "Unavailable".into()
                } else if self.sys.battery_saver {
                    "On".into()
                } else {
                    "Off".into()
                },
                SysIcon::BatterySaver,
                self.sys.battery_saver && !self.sys.charging,
            ),
            QsTileKind::DarkTheme => (
                "Dark theme".into(),
                if self.sys.dark_theme { "On" } else { "Off" }.into(),
                SysIcon::DarkMode,
                self.sys.dark_theme,
            ),
            QsTileKind::Dnd => (
                "Do Not Disturb".into(),
                if self.sys.dnd_enabled { "On" } else { "Off" }.into(),
                SysIcon::Dnd,
                self.sys.dnd_enabled,
            ),
            QsTileKind::ScreenRecord => (
                "Screen record".into(),
                if self.sys.screen_record { "Recording" } else { "Start" }.into(),
                SysIcon::ScreenRecord,
                self.sys.screen_record,
            ),
            QsTileKind::NightLight => (
                "Night Light".into(),
                if self.sys.night_light { "On" } else { "Off" }.into(),
                SysIcon::NightLight,
                self.sys.night_light,
            ),
            QsTileKind::CameraAccess => (
                "Camera access".into(),
                if self.sys.camera_access { "Available" } else { "Blocked" }.into(),
                SysIcon::Camera,
                self.sys.camera_access,
            ),
            QsTileKind::MicAccess => (
                "Mic access".into(),
                if self.sys.mic_access { "Available" } else { "Blocked" }.into(),
                SysIcon::Mic,
                self.sys.mic_access,
            ),
            QsTileKind::Aod => (
                "AOD".into(),
                if self.sys.aod { "On" } else { "Off" }.into(),
                SysIcon::Aod,
                self.sys.aod,
            ),
            QsTileKind::Hotspot => (
                "Hotspot".into(),
                if self.sys.hotspot { "Active" } else { "Off" }.into(),
                SysIcon::Hotspot,
                self.sys.hotspot,
            ),
        }
    }

    fn render_active_apps_dialog(&mut self, f: &mut Frame<'_>, phone: Rect, now: f64) {
        let p = self.sys.palette();
        f.paint_rect(phone, 0.0, Color(0, 0, 0, 120));

        let shiraz_rows = usize::from(self.sys.shiraz_enabled)
            + self.sys.shiraz.running_packages.len();
        let total_rows = self.active_bg_apps.len() + shiraz_rows;
        let card_w = (phone.width - 36.0).min(360.0);
        let card_h = (190.0 + total_rows.max(1) as f32 * 58.0).min(phone.height - 96.0);
        let card = Rect {
            x: phone.x + (phone.width - card_w) * 0.5,
            y: phone.y + (phone.height - card_h) * 0.5,
            width: card_w,
            height: card_h,
        };
        f.elevation(card, 28.0, 4);
        f.paint_rect(card, 28.0, p.card_pure);

        f.styled_text(
            Rect {
                x: card.x + 24.0,
                y: card.y + 24.0,
                width: card.width - 48.0,
                height: 30.0,
            },
            "Active apps",
            p.on_surface,
            text_style(22.0, 400, 1),
        );
        f.paragraph(
            Rect {
                x: card.x + 24.0,
                y: card.y + 62.0,
                width: card.width - 48.0,
                height: 60.0,
            },
            "These apps are active and running, even when you're not using them. This improves their functionality, but it may also affect battery life.",
            p.on_surface_variant,
            text_style(13.0, 400, 1),
            18.0,
        );

        let mut stop_idx: Option<usize> = None;
        let mut stop_shiraz_daemon = false;
        let mut stop_shiraz_pkg: Option<String> = None;
        if total_rows == 0 {
            f.styled_text(
                Rect {
                    x: card.x + 24.0,
                    y: card.y + 148.0,
                    width: card.width - 48.0,
                    height: 28.0,
                },
                "No background apps running",
                p.on_surface_variant,
                text_style(14.0, 500, 1),
            );
        } else {
            let mut row_slot = 0usize;
            for (i, &(app, dur)) in self.active_bg_apps.iter().enumerate() {
                let ry = card.y + 140.0 + row_slot as f32 * 58.0;
                row_slot += 1;
                icons::paint_real_app_icon(self.clock,
                    f,
                    Rect {
                        x: card.x + 22.0,
                        y: ry + 8.0,
                        width: 36.0,
                        height: 36.0,
                    },
                    app,
                    false,
                    &p,
                );
                f.styled_text(
                    Rect {
                        x: card.x + 70.0,
                        y: ry + 8.0,
                        width: card.width - 168.0,
                        height: 20.0,
                    },
                    app.name(),
                    p.on_surface,
                    text_style(14.0, 600, 0),
                );
                f.styled_text(
                    Rect {
                        x: card.x + 70.0,
                        y: ry + 28.0,
                        width: card.width - 168.0,
                        height: 18.0,
                    },
                    dur,
                    p.on_surface_variant,
                    text_style(12.5, 400, 0),
                );
                let stop_r = Rect {
                    x: card.x + card.width - 88.0,
                    y: ry + 8.0,
                    width: 68.0,
                    height: 36.0,
                };
                let rs = f.target(SHADE_ID + 710 + i as u64, stop_r, "Stop app", true, Role::Button);
                f.paint_rect(stop_r, 18.0, p.primary);
                f.paint_rect(stop_r.inset(1.2), 17.0, p.card_pure);
                f.styled_text(stop_r, "Stop", p.on_surface, text_style(13.5, 500, 1));
                if rs.clicked {
                    stop_idx = Some(i);
                }
            }
            if self.sys.shiraz_enabled {
                let ry = card.y + 140.0 + row_slot as f32 * 58.0;
                row_slot += 1;
                let badge = Rect {
                    x: card.x + 22.0,
                    y: ry + 8.0,
                    width: 36.0,
                    height: 36.0,
                };
                f.paint_rect(badge, 18.0, p.primary_container);
                paint_sys_icon(f, badge.inset(8.0), SysIcon::System, p.on_primary_container);
                f.styled_text(
                    Rect {
                        x: card.x + 70.0,
                        y: ry + 8.0,
                        width: card.width - 168.0,
                        height: 20.0,
                    },
                    "Shiraz Runtime",
                    p.on_surface,
                    text_style(14.0, 600, 0),
                );
                let sub = if self.sys.shiraz.warm_ram_mb > 0 {
                    format!("Warm in memory · {} MB", self.sys.shiraz.warm_ram_mb)
                } else {
                    self.sys.shiraz.state.label().to_owned()
                };
                f.styled_text(
                    Rect {
                        x: card.x + 70.0,
                        y: ry + 28.0,
                        width: card.width - 168.0,
                        height: 18.0,
                    },
                    &sub,
                    p.on_surface_variant,
                    text_style(12.5, 400, 0),
                );
                let stop_r = Rect {
                    x: card.x + card.width - 88.0,
                    y: ry + 8.0,
                    width: 68.0,
                    height: 36.0,
                };
                let rs = f.target(SHADE_ID + 740, stop_r, "Disable Shiraz", true, Role::Button);
                f.paint_rect(stop_r, 18.0, p.primary);
                f.paint_rect(stop_r.inset(1.2), 17.0, p.card_pure);
                f.styled_text(stop_r, "Stop", p.on_surface, text_style(13.5, 500, 1));
                if rs.clicked {
                    stop_shiraz_daemon = true;
                }
            }
            let running_pkgs = self.sys.shiraz.running_packages.clone();
            for (j, pkg) in running_pkgs.iter().enumerate() {
                let ry = card.y + 140.0 + row_slot as f32 * 58.0;
                row_slot += 1;
                let label = self
                    .sys
                    .shiraz
                    .installed_apps
                    .iter()
                    .find(|a| a.package == *pkg)
                    .map(|a| a.label.clone())
                    .unwrap_or_else(|| pkg.clone());
                let badge = Rect {
                    x: card.x + 22.0,
                    y: ry + 8.0,
                    width: 36.0,
                    height: 36.0,
                };
                f.paint_rect(badge, 18.0, p.primary_container);
                paint_sys_icon(f, badge.inset(8.0), SysIcon::AppsGrid, p.on_primary_container);
                f.styled_text(
                    Rect {
                        x: card.x + 70.0,
                        y: ry + 8.0,
                        width: card.width - 168.0,
                        height: 20.0,
                    },
                    &label,
                    p.on_surface,
                    text_style(14.0, 600, 0),
                );
                f.styled_text(
                    Rect {
                        x: card.x + 70.0,
                        y: ry + 28.0,
                        width: card.width - 168.0,
                        height: 18.0,
                    },
                    "Android app (Shiraz)",
                    p.on_surface_variant,
                    text_style(12.5, 400, 0),
                );
                let stop_r = Rect {
                    x: card.x + card.width - 88.0,
                    y: ry + 8.0,
                    width: 68.0,
                    height: 36.0,
                };
                let rs = f.target(SHADE_ID + 745 + j as u64, stop_r, "Stop Android app", true, Role::Button);
                f.paint_rect(stop_r, 18.0, p.primary);
                f.paint_rect(stop_r.inset(1.2), 17.0, p.card_pure);
                f.styled_text(stop_r, "Stop", p.on_surface, text_style(13.5, 500, 1));
                if rs.clicked {
                    stop_shiraz_pkg = Some(pkg.clone());
                }
            }
        }
        if let Some(i) = stop_idx {
            let (app, _) = self.active_bg_apps.remove(i);
            self.notifications.retain(|n| n.target_app != Some(app) || !n.silent);
            self.show_toast(format!("Stopped {}", app.name()), now);
        }
        if stop_shiraz_daemon {
            self.sys.shiraz_enabled = false;
            self.sys.save_persisted_settings();
            self.show_toast("Disabling Shiraz", now);
        }
        if let Some(pkg) = stop_shiraz_pkg {
            self.host_commands.push(HostCommand::ShirazStopPackage(pkg.clone()));
            self.show_toast(format!("Stopped {pkg}"), now);
        }
        let over_card = f.pointer_in(card);
        let scrim = f.without_pointer_input(over_card, |f| {
            f.target(SHADE_ID + 700, phone, "Dismiss active apps", true, Role::Button)
        });
        if scrim.clicked {
            self.active_apps_open = false;
        }
    }

    fn render_power_menu(&mut self, f: &mut Frame<'_>, phone: Rect, now: f64) {
        let p = self.sys.palette();
        let prog = self.power_anim.value.clamp(0.0, 1.0);
        f.paint_rect(phone, 0.0, Color(0, 0, 0, (prog * 140.0) as u8));

        let card_w = 276.0_f32.min(phone.width - 44.0);
        let card_h = 304.0;
        let card = Rect {
            x: phone.x + (phone.width - card_w) * 0.5,
            y: phone.y + (phone.height - card_h) * 0.5,
            width: card_w,
            height: card_h,
        };
        f.push_transform(
            [card.x + card_w * 0.5, card.y + card_h * 0.5],
            [0.0, (1.0 - prog) * 14.0],
            [0.88 + 0.12 * prog, 0.88 + 0.12 * prog],
            0.0,
        );
        f.elevation(card, 28.0, 4);
        f.paint_rect(card, 28.0, p.card_elevated);

        // 2x2 circular action buttons matching 12-power-menu.png!
        let actions = [
            ("Emergency", SysIcon::Emergency, true),
            ("Power off", SysIcon::Power, false),
            ("Restart", SysIcon::Restart, false),
            ("Screenshot", SysIcon::Screenshot, false),
        ];
        let cell_w = card_w * 0.5;
        for (i, &(label, icon, is_red)) in actions.iter().enumerate() {
            let col = i % 2;
            let row = i / 2;
            let cx = card.x + col as f32 * cell_w + cell_w * 0.5;
            let cy = card.y + 74.0 + row as f32 * 136.0;
            let circ = Rect {
                x: cx - 42.0,
                y: cy - 42.0,
                width: 84.0,
                height: 84.0,
            };
            let r = f.target(SHADE_ID + 810 + i as u64, circ, label, true, Role::Button);
            let bg = if is_red {
                Color(0xF2, 0x5C, 0x4D, 255)
            } else {
                p.card_pure
            };
            let fg = if is_red {
                Color(0x1B, 0x1B, 0x1F, 255)
            } else {
                p.on_surface
            };
            f.paint_rect(circ, 42.0, bg);
            f.state_layer(r, 42.0, fg);
            wallpaper_ui::paint_wallpaper_system_icon(f, circ.inset(30.0), icon, fg, self.sys.icon_style);
            f.styled_text(
                Rect {
                    x: cx - 56.0,
                    y: cy + 48.0,
                    width: 112.0,
                    height: 22.0,
                },
                label,
                p.on_surface,
                text_style(13.5, 400, 1),
            );
            if r.clicked {
                match i {
                    0 => {
                        self.power_open = false;
                        self.power_anim.set(0.0, now, 0.18, 0.14);
                        self.open_app(AppId::Phone, now);
                        self.show_toast("Emergency dialer", now);
                    }
                    1 => {
                        self.power_open = false;
                        self.power_anim.snap(0.0);
                        self.power_state = PowerState::ShuttingDown((now * 1000.0) as u64);
                    }
                    2 => {
                        self.power_open = false;
                        self.power_anim.snap(0.0);
                        self.power_state = PowerState::Rebooting((now * 1000.0) as u64);
                    }
                    _ => self.take_screenshot(now),
                }
            }
        }
        f.pop_transform();

        let over_card = f.pointer_in(card);
        let scrim = f.without_pointer_input(over_card, |f| {
            f.target(SHADE_ID + 800, phone, "Dismiss power menu", true, Role::Button)
        });
        if scrim.clicked {
            self.power_open = false;
            self.power_anim.set(0.0, now, 0.20, 0.16);
        }
    }

    fn render_volume_panel(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        now: f64,
    ) {
        let p = self.sys.palette();
        let prog = self.vol_anim.value.clamp(0.0, 1.0);
        let exp = self.vol_expand_anim.value.clamp(0.0, 1.0);

        // Exact match to 13-volume-compact.png & 14-volume-expanded-dialog.png!
        let col_w = 56.0;
        let col_gap = 6.0;
        let main_h = 286.0;
        let slide_x = (1.0 - prog) * 80.0;
        let main_x = phone.x + phone.width - col_w - 8.0 + slide_x;
        let main_y = phone.y + (phone.height - main_h) * 0.48;

        // Expanded 3 extra columns sliding out to the left of Media: Alarm, Notification, Ring
        if exp > 0.02 {
            let sub_h = 190.0;
            let sub_y = main_y + 52.0;
            let extra_specs: &[(u64, &str, SysIcon, usize, u8)] = &[
                (SHADE_ID + 920, "Alarm volume", SysIcon::Alarm, 0, 1),
                (SHADE_ID + 921, "Notification volume", SysIcon::Bell, 1, 2),
                (SHADE_ID + 922, "Ring volume", SysIcon::Vibrate, 2, 3),
            ];
            for (idx, &(id, lbl, icon, slot, channel)) in extra_specs.iter().enumerate() {
                let offset = (3 - idx) as f32 * (col_w + col_gap) * exp;
                let pill = Rect {
                    x: main_x - offset,
                    y: sub_y,
                    width: col_w,
                    height: sub_h,
                };
                f.elevation(pill, 28.0, 3);
                f.paint_rect(pill, 28.0, p.card_pure.opacity(exp));
                let track = Rect {
                    x: pill.x + 8.0,
                    y: pill.y + 12.0,
                    width: pill.width - 16.0,
                    height: pill.height - 24.0,
                };
                self.last_vol_tracks[channel as usize] = Some((channel, track));
                let val = match slot {
                    0 => self.sys.vol_alarm,
                    1 => self.sys.vol_notif,
                    _ => self.sys.vol_ring,
                };
                let _r = f.target(id, track, lbl, true, Role::Slider);
                // Thin center vertical track line
                let tx = track.x + track.width * 0.5;
                f.paint_line(
                    [tx, track.y + 8.0],
                    [tx, track.y + track.height - 8.0],
                    3.0,
                    p.outline,
                );
                // Pill thumb fill from bottom
                let fill_h = (track.height * val).clamp(40.0, track.height);
                let fill_r = Rect {
                    x: track.x,
                    y: track.y + track.height - fill_h,
                    width: track.width,
                    height: fill_h,
                };
                f.paint_rect(fill_r, 20.0, p.primary_container);
                paint_sys_icon(
                    f,
                    Rect {
                        x: fill_r.x + (fill_r.width - 18.0) * 0.5,
                        y: fill_r.y + 11.0,
                        width: 18.0,
                        height: 18.0,
                    },
                    icon,
                    p.on_primary_container,
                );
            }
        }

        // Primary compact Media pillar on right edge
        let pillar = Rect {
            x: main_x,
            y: main_y,
            width: col_w,
            height: main_h,
        };
        f.elevation(pillar, 28.0, 4);
        f.paint_rect(pillar, 28.0, p.card_pure);

        // Top ringer mode toggle circle (#435E91)
        let ringer_r = Rect {
            x: pillar.x + 8.0,
            y: pillar.y + 8.0,
            width: 40.0,
            height: 40.0,
        };
        let rr = f.target(SHADE_ID + 900, ringer_r, "Ringer mode", true, Role::Button);
        f.paint_rect(ringer_r, 20.0, p.primary);
        f.state_layer(rr, 20.0, p.on_primary);
        let r_ic = match self.sys.ringer_mode {
            RingerMode::Vibrate => SysIcon::Vibrate,
            RingerMode::Silent => SysIcon::Silent,
            RingerMode::Ring => SysIcon::Ring,
        };
        paint_sys_icon(f, ringer_r.inset(10.0), r_ic, p.on_primary);
        if rr.clicked {
            self.sys.ringer_mode = match self.sys.ringer_mode {
                RingerMode::Vibrate => RingerMode::Silent,
                RingerMode::Silent => RingerMode::Ring,
                RingerMode::Ring => RingerMode::Vibrate,
            };
            self.vol_hide_at = now + 4.5;
            self.show_toast(
                match self.sys.ringer_mode {
                    RingerMode::Vibrate => "Vibrate mode",
                    RingerMode::Silent => "Silent mode",
                    RingerMode::Ring => "Ring mode",
                },
                now,
            );
        }

        // Media vertical slider
        let m_track = Rect {
            x: pillar.x + 8.0,
            y: pillar.y + 62.0,
            width: 40.0,
            height: 170.0,
        };
        self.last_vol_tracks[0] = Some((0, m_track));
        let _rm = f.target(SHADE_ID + 901, m_track, "Media volume", true, Role::Slider);
        let mx = m_track.x + m_track.width * 0.5;
        f.paint_line(
            [mx, m_track.y + 4.0],
            [mx, m_track.y + m_track.height - 4.0],
            3.5,
            p.primary.opacity(0.3),
        );
        let m_fill_h = (m_track.height * self.sys.vol_media).clamp(40.0, m_track.height);
        let m_fill = Rect {
            x: m_track.x,
            y: m_track.y + m_track.height - m_fill_h,
            width: m_track.width,
            height: m_fill_h,
        };
        f.paint_rect(m_fill, 20.0, p.primary);
        paint_sys_icon(
            f,
            Rect {
                x: m_fill.x + 11.0,
                y: m_fill.y + 11.0,
                width: 18.0,
                height: 18.0,
            },
            SysIcon::MusicNote,
            p.on_primary,
        );

        // Expand / Collapse chevron button at bottom (`<` / `>`)
        let exp_r = Rect {
            x: pillar.x + 8.0,
            y: pillar.y + pillar.height - 46.0,
            width: 40.0,
            height: 38.0,
        };
        let re = f.target(SHADE_ID + 902, exp_r, "Expand volume panel", true, Role::Button);
        f.state_layer(re, 19.0, p.primary);
        paint_sys_icon(
            f,
            exp_r.inset(11.0),
            if self.vol_expanded {
                SysIcon::ChevronRight
            } else {
                SysIcon::ChevronLeft
            },
            p.primary,
        );
        if re.clicked {
            self.vol_expanded = !self.vol_expanded;
            self.vol_expand_anim
                .set(f32::from(self.vol_expanded), now, 0.24, 0.20);
            self.vol_hide_at = now + 5.5;
        }
    }

    // =========================================================================
    // SUBSYSTEM 3: MULTITASKING (OVERVIEW, SPLIT-SCREEN, FREEFORM)
    // =========================================================================

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn render_empty_overview(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
        now: f64,
        gmail_text: &mut String,
    ) {
        let progress = self.overview_anim.value.clamp(0.0, 1.0);
        f.paint_rect(phone, 0.0, Color(79, 79, 79, 255));
        if progress < 0.999 {
            f.push_layer(phone, 1.0 - progress);
            f.without_input(true, |f| {
                self.render_home_layer(f, phone, status_h, nav_h, None, now, gmail_text, None);
            });
            f.pop_layer();
        }

        let alpha = smoothstep(0.08, 0.78, progress);
        let center_x = phone.x + phone.width * 0.5;
        let center_y = phone.y + phone.height * 0.456;
        let icon_ink = Color(189, 189, 189, 255).opacity(alpha);
        for (offset, width, height) in [(-34.0, 15.0, 62.0), (0.0, 37.0, 74.0), (34.0, 15.0, 62.0)] {
            let x = center_x + offset - width * 0.5;
            let y = center_y - height * 0.5;
            let corners = [[x, y], [x + width, y], [x + width, y + height], [x, y + height]];
            for (from, to) in corners.into_iter().zip(corners.into_iter().cycle().skip(1)) {
                f.paint_line(from, to, 3.5, icon_ink);
            }
        }
        f.styled_text(
            Rect { x: phone.x, y: center_y + 52.0, width: phone.width, height: 22.0 },
            "No recent items",
            Color(255, 255, 255, 255).opacity(alpha),
            text_style(13.0, 400, 1),
        );
        if self.mode == ShellMode::Overview
            && f.target(MULTI_ID + 2, phone, "Return home", true, Role::Button).clicked
        {
            self.go_home(now);
        }
    }

    fn render_overview(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        app_rect: Rect,
        status_h: f32,
        nav_h: f32,
        split_first: Option<AppId>,
        pointer: Option<[f32; 2]>,
        gmail_text: &mut String,
        now: f64,
    ) {
        if split_first.is_none() && self.tasks.is_empty() {
            self.render_empty_overview(f, phone, status_h, nav_h, now, gmail_text);
            return;
        }
        let p = self.sys.palette();
        let overview_ink = Color(255, 255, 255, 255);
        let overview_action_bg = Color(60, 60, 64, 255);
        let reopening = match self.app_transition {
            AppTransition::Opening { app, origin, from_overview: true, .. } => {
                Some((TaskEntry::Single(app), origin))
            }
            AppTransition::OpeningDesktop { origin, from_overview: true, .. } => {
                match self.mode {
                    ShellMode::FullscreenDesktop(idx) => Some((TaskEntry::Desktop(idx), origin)),
                    _ => None,
                }
            }
            _ => None,
        };
        let ov_prog = if reopening.is_some() { 1.0 } else { self.overview_anim.value.clamp(0.0, 1.0) };
        let vertical_offset = match self.gesture {
            ActiveGesture::OverviewVertical { offset_y, .. } => offset_y,
            _ => self.overview_vertical_settle.as_ref().map_or(0.0, |(_, slide, _)| slide.value),
        };
        let drag_progress = Self::overview_drag_progress(vertical_offset, phone);
        if split_first.is_none() {
            if !self.overview_exiting {
                self.wallpapers.home.paint(f, phone, self.sys.wallpaper_idx);
            }
            let tint = if reopening.is_some() {
                Self::overview_drag_tint(self.overview_reopen_progress)
            } else if self.tasks.len() == 1 && vertical_offset < 0.0 {
                1.0 - smoothstep(80.0, phone.height * 0.8, -vertical_offset)
            } else {
                Self::overview_drag_tint(drag_progress)
            };
            f.paint_rect(phone, 0.0,
                Color(44, 44, 44, 255).opacity(0.85 * smoothstep(0.13, 1.0, ov_prog) * tint));
        } else {
            f.paint_rect(phone, 0.0, p.recents_bg);
        }

        // Split-screen second app picker header strip (05-split-select-second-app.png)
        let mut content_top = phone.y + status_h + 22.0;
        if let Some(first_app) = split_first {
            let strip = Rect {
                x: phone.x + 16.0,
                y: phone.y + status_h + 4.0,
                width: phone.width - 32.0,
                height: 54.0,
            };
            f.paint_rect(strip, 14.0, p.card_elevated);
            icons::paint_real_app_icon(self.clock,
                f,
                Rect {
                    x: strip.x + strip.width * 0.5 - 20.0,
                    y: strip.y + 7.0,
                    width: 40.0,
                    height: 40.0,
                },
                first_app,
                false,
                &p,
            );
            let cancel_r = Rect {
                x: strip.x + strip.width - 38.0,
                y: strip.y + 11.0,
                width: 32.0,
                height: 32.0,
            };
            let rc = f.target(MULTI_ID + 1, cancel_r, "Cancel split", true, Role::Button);
            paint_sys_icon(f, cancel_r.inset(7.0), SysIcon::Close, p.on_surface_variant);
            if rc.clicked {
                self.mode = ShellMode::Overview;
        self.active_wayland_app = None;
        self.active_wayland_desktop = None;
        if self.native_compositor {
            self.host_commands.push(HostCommand::MinimizeAll);
        }
            }
            content_top += 56.0;
        }

        // Filter candidates when picking second app for split screen
        let entries: Vec<TaskEntry> = if let Some(first) = split_first {
            let mut list: Vec<TaskEntry> = self
                .tasks
                .iter()
                .copied()
                .filter(|t| match *t {
                    TaskEntry::Single(a) => a != first,
                    TaskEntry::SplitPair(_, _) | TaskEntry::Desktop(_) => false,
                })
                .collect();
            if list.is_empty() {
                for &fallback in &[AppId::Clock, AppId::Settings, AppId::Calculator, AppId::Gmail] {
                    if fallback != first {
                        list.push(TaskEntry::Single(fallback));
                    }
                }
            }
            list
        } else {
            self.tasks.clone()
        };

        // Exact real-Android 13 Overview proportions (`01-recents-main.png`):
        // Card preserves phone screen aspect ratio (~67% width × full phone aspect -> ~275×566dp on 411×846)!
        let aspect = phone.height / phone.width.max(1.0);
        let max_card_h = (phone.height - (content_top - phone.y) - nav_h - 116.0).max(280.0);
        let card_w = (phone.width * 0.67).clamp(220.0, 286.0).min(max_card_h / aspect);
        let card_h = card_w * aspect;
        let gap = 22.0;
        let step = card_w + gap;
        let clear_w = if split_first.is_none() { 116.0 } else { 0.0 };
        let center_offset = (phone.width - card_w) * 0.5;
        let max_scroll = ((entries.len().saturating_sub(1)) as f32 * step + clear_w).max(0.0);
        if !matches!(self.gesture, ActiveGesture::OverviewHorizontal { .. }) {
            self.overview_scroll = self.overview_scroll.clamp(0.0, max_scroll);
        }

        let carousel_rect = Rect {
            x: phone.x,
            y: content_top,
            width: phone.width,
            height: card_h + 76.0,
        };
        self.last_overview_rect = carousel_rect;
        self.last_overview_step = step;
        self.last_overview_max = max_scroll;

        let mut open_single: Option<(AppId, Rect)> = None;
        let mut open_desktop: Option<(usize, Rect)> = None;
        let mut open_pair: Option<(AppId, AppId)> = None;
        let mut clear_all_clicked = false;
        let mut toggle_menu_idx: Option<usize> = None;

        // Task cards are laid out newest at right/center (`i = 0` at `scroll = 0`), older to the left
        // matching Android 13 Recents (`01-recents-main.png`: Clock peeks on LEFT of Contacts!).
        // Dragging RIGHT moves cards RIGHT 1:1 with finger, sliding older left cards into center!
        // Translate the carousel as a rigid row. Focus, corner radius and card
        // dimensions are calculated before applying the edge displacement.
        let carousel_dx = self.overview_edge_spring.distance() * 32.0
            - if self.overview_exiting { (1.0 - ov_prog) * phone.width } else { 0.0 };
        let expanding_index = if drag_progress > 0.0 {
            match self.gesture {
                ActiveGesture::OverviewVertical { task_idx, .. } => Some(task_idx),
                _ => self.overview_vertical_settle.as_ref()
                    .and_then(|(task, _, _)| entries.iter().position(|entry| entry == task)),
            }
        } else {
            None
        };
        // The carousel rect is for gestures. Paint against the display so
        // live previews can transform their fullscreen clips into card space.
        f.within(phone, |f| {
            // The expanding task covers its neighbours as it approaches fullscreen.
            for i in (0..entries.len()).filter(|i| Some(*i) != expanding_index)
                .chain(expanding_index.filter(|&i| i < entries.len()))
            {
                let entry = entries[i];
                let card_x = phone.x + center_offset - i as f32 * step + self.overview_scroll;
                let dismiss_dy = match self.gesture {
                    ActiveGesture::OverviewVertical {
                        task_idx, offset_y, ..
                    } if task_idx == i => offset_y,
                    _ => self.overview_vertical_settle.as_ref()
                        .filter(|(task, _, _)| *task == entry)
                        .map_or(0.0, |(_, slide, _)| slide.value),
                };
                let dismiss_alpha = if dismiss_dy < -2.0 {
                    (1.0 - ((-dismiss_dy) / (card_h * 0.65)).clamp(0.0, 0.85)).clamp(0.15, 1.0)
                } else {
                    1.0
                };
                let card_y = content_top + 56.0 + dismiss_dy.min(0.0);
                if card_x + carousel_dx + card_w < phone.x - 40.0
                    || card_x + carousel_dx > phone.x + phone.width + 40.0
                {
                    continue;
                }
                let enter_dx = if i > 0 && ov_prog < 0.995 && !self.overview_exiting {
                    (1.0 - ov_prog) * -86.0
                } else {
                    0.0
                };
                let card = Rect {
                    x: card_x + enter_dx,
                    y: card_y,
                    width: card_w,
                    height: card_h,
                };
                let dist_center = ((card.x + card_w * 0.5) - (phone.x + phone.width * 0.5)).abs();
                let dismiss_shrink = ((-dismiss_dy) / (card_h * 2.5)).clamp(0.0, 0.08);
                let focus =
                    (1.0 - (dist_center / step).clamp(0.0, 1.0) * 0.08 - dismiss_shrink).clamp(0.84, 1.0);
                let inset_x = (1.0 - focus) * 12.0;
                let inset_y = inset_x * aspect;
                let base_drawn = Rect {
                    x: card.x + inset_x,
                    y: card.y + inset_y,
                    width: card.width - 2.0 * inset_x,
                    height: card.height - 2.0 * inset_y,
                };
                let (mut drawn, mut card_rad) = if i == 0
                    && ov_prog < 0.995
                    && let Some((source, radius)) = self.overview_enter_from
                {
                    (lerp_rect(source, base_drawn, ov_prog, ov_prog), lerp(radius, 26.0, ov_prog))
                } else {
                    (base_drawn, 26.0)
                };
                drawn.x += carousel_dx;
                if Some(i) == expanding_index {
                    let origin = match self.gesture {
                        ActiveGesture::OverviewVertical { original_card, .. } => original_card,
                        _ => drawn,
                    };
                    drawn = lerp_rect(origin, phone, drag_progress, drag_progress);
                    card_rad *= 1.0 - drag_progress;
                }
                self.last_overview_cards.push((i, drawn));

                if dismiss_alpha < 0.999 {
                    f.push_layer(phone, dismiss_alpha);
                }

                // Floating circular App Icon Badge centered above card (fades in smoothly during entry!)
                let badge_a = smoothstep(0.28, 0.92, ov_prog);
                let icon_card = reopening.filter(|(task, _)| entry == *task)
                    .map_or(drawn, |(_, origin)| origin);
                let icon_y = icon_card.y - 50.0;
                if badge_a > 0.02 {
                    if badge_a < 0.995 {
                        f.push_layer(phone, badge_a);
                    }
                    match entry {
                        TaskEntry::Single(app) => {
                            let badge = Rect {
                                x: icon_card.x + icon_card.width * 0.5 - 22.0,
                                y: icon_y,
                                width: 44.0,
                                height: 44.0,
                            };
                            let r_ic = f.target(
                                MULTI_ID + 100 + i as u64,
                                badge,
                                "Task menu",
                                split_first.is_none() && ov_prog > 0.5,
                                Role::Button,
                            );
                            let themed = if app == AppId::Settings {
                                self.settings_launch_themed.unwrap_or(self.sys.themed_icons)
                            } else {
                                self.sys.themed_icons
                            };
                            icons::paint_real_app_icon(self.clock, f, badge, app, themed, &p);
                            if r_ic.clicked {
                                toggle_menu_idx = Some(i);
                            }
                        }
                        TaskEntry::SplitPair(a, b) => {
                            // Paired icon badges matching 08-recents-split-pair.png!
                            let b1 = Rect {
                                x: drawn.x + drawn.width * 0.5 - 44.0,
                                y: icon_y,
                                width: 40.0,
                                height: 40.0,
                            };
                            let b2 = Rect {
                                x: drawn.x + drawn.width * 0.5 + 4.0,
                                y: icon_y,
                                width: 40.0,
                                height: 40.0,
                            };
                            icons::paint_real_app_icon(self.clock, f, b1, a, false, &p);
                            icons::paint_real_app_icon(self.clock, f, b2, b, false, &p);
                        }
                        TaskEntry::Desktop(idx) => {
                            if let Some(entry) = self.desktop_registry.apps.get(idx).cloned() {
                                let icon_img = self.desktop_registry.icon_for(&entry.id);
                                let badge = Rect {
                                    x: drawn.x + drawn.width * 0.5 - 22.0,
                                    y: icon_y,
                                    width: 44.0,
                                    height: 44.0,
                                };
                                freedesktop::paint_desktop_app_icon(
                                    f,
                                    badge,
                                    &entry.name,
                                    &entry.id,
                                    icon_img.as_ref(),
                                    &p,
                                );
                            }
                        }
                    }
                    if badge_a < 0.995 {
                        f.pop_layer();
                    }
                }

                // The reopening transition paints the selected card in front of Recents.
                if reopening.is_some_and(|(task, _)| entry == task) {
                    continue;
                }
                let r_card = f.target(
                    MULTI_ID + 200 + i as u64,
                    drawn,
                    "Switch to task",
                    self.task_menu_idx.is_none(),
                    Role::CarouselItem,
                );
                match entry {
                    TaskEntry::Single(app) => {
                        self.paint_overview_app(f, app, phone, app_rect, drawn, card_rad, gmail_text);
                    }
                    TaskEntry::SplitPair(top_a, bot_a) => {
                        let half = (drawn.height - 8.0) * 0.5;
                        let r_top = Rect {
                            height: half,
                            ..drawn
                        };
                        let r_bot = Rect {
                            y: drawn.y + half + 8.0,
                            height: half,
                            ..drawn
                        };
                        let sx = drawn.width / phone.width.max(1.0);
                        let sy = half / phone.height.max(1.0);
                        f.within(r_top, |f| {
                            f.without_input(true, |f| {
                                f.push_transform(
                                    [phone.x, phone.y],
                                    [r_top.x - phone.x, r_top.y - phone.y],
                                    [sx, sy],
                                    0.0,
                                );
                                self.paint_app_backdrop(f, top_a, phone);
                                let _ = self.render_live_app(
                                    f,
                                    top_a,
                                    app_rect,
                                    APP_ID_BASE + ((top_a as u64) << 28),
                                    pointer,
                                    gmail_text,
                                );
                                f.pop_transform();
                            });
                        });
                        f.within(r_bot, |f| {
                            f.without_input(true, |f| {
                                f.push_transform(
                                    [phone.x, phone.y],
                                    [r_bot.x - phone.x, r_bot.y - phone.y],
                                    [sx, sy],
                                    0.0,
                                );
                                self.paint_app_backdrop(f, bot_a, phone);
                                let _ = self.render_live_app(
                                    f,
                                    bot_a,
                                    app_rect,
                                    APP_ID_BASE + ((bot_a as u64) << 28),
                                    pointer,
                                    gmail_text,
                                );
                                f.pop_transform();
                            });
                        });
                    }
                    TaskEntry::Desktop(idx) => {
                        f.paint_rect(drawn, card_rad, p.card_elevated);
                        if let Some(frame) = self.wayland_task_frame(entry) {
                            f.paint_image(drawn, frame, ImageFit::Contain, card_rad);
                        } else if let Some(entry) = self.desktop_registry.apps.get(idx).cloned() {
                            let icon_img = self.desktop_registry.icon_for(&entry.id);
                            let ir = Rect {
                                x: drawn.x + (drawn.width - 64.0) * 0.5,
                                y: drawn.y + drawn.height * 0.34,
                                width: 64.0,
                                height: 64.0,
                            };
                            freedesktop::paint_desktop_app_icon(
                                f,
                                ir,
                                &entry.name,
                                &entry.id,
                                icon_img.as_ref(),
                                &p,
                            );
                            f.styled_text(
                                Rect {
                                    x: drawn.x + 16.0,
                                    y: ir.y + 78.0,
                                    width: drawn.width - 32.0,
                                    height: 26.0,
                                },
                                &entry.name,
                                p.on_surface,
                                text_style(16.0, 600, 1),
                            );
                            let sub = entry
                                .generic_name
                                .as_deref()
                                .or(entry.comment.as_deref())
                                .unwrap_or("Wayland application");
                            f.styled_text(
                                Rect {
                                    x: drawn.x + 18.0,
                                    y: ir.y + 106.0,
                                    width: drawn.width - 36.0,
                                    height: 22.0,
                                },
                                sub,
                                p.on_surface_variant,
                                text_style(12.0, 400, 1),
                            );
                        }
                    }
                }

                if r_card.clicked {
                    if let Some(first) = split_first {
                        if let TaskEntry::Single(second) = entry {
                            open_pair = Some((first, second));
                        }
                    } else if dist_center > card_w * 0.42 {
                        // Clicking peeking side card centers it smoothly with gradual deceleration!
                        let target = (i as f32 * step).clamp(0.0, max_scroll);
                        self.overview_glide = Some(OverviewGlide::start(
                            self.overview_scroll,
                            target,
                            0.0,
                            now,
                        ));
                    } else {
                        match entry {
                            TaskEntry::Single(a) => open_single = Some((a, drawn)),
                            TaskEntry::SplitPair(a, b) => open_pair = Some((a, b)),
                            TaskEntry::Desktop(idx) => open_desktop = Some((idx, drawn)),
                        }
                    }
                }

                // Task Menu Popup right over the card (04-recents-task-menu.png!)
                if self.task_menu_idx == Some(i)
                    && let TaskEntry::Single(app) = entry
                {
                    f.paint_rect(drawn, 22.0, Color(0, 0, 0, 88));
                    let menu_w = drawn.width - 16.0;
                    let mx = drawn.x + 8.0;
                    let mut my = drawn.y + 8.0;
                    // Header title pill
                    let hdr = Rect {
                        x: mx,
                        y: my,
                        width: menu_w,
                        height: 36.0,
                    };
                    f.paint_rect(hdr, 16.0, overview_action_bg);
                    f.styled_text(hdr, app.name(), overview_ink, text_style(14.5, 500, 1));
                    my += 39.0;

                    let menu_items = [
                        ("App info", SysIcon::Info, 0_usize),
                        ("Split screen", SysIcon::SplitScreen, 1_usize),
                        ("Freeform", SysIcon::Freeform, 2_usize),
                    ];
                    for (lbl, ic, k) in menu_items {
                        let mr = Rect {
                            x: mx,
                            y: my,
                            width: menu_w,
                            height: 48.0,
                        };
                        let rm = f.target(MULTI_ID + 300 + k as u64, mr, lbl, true, Role::MenuItem);
                        f.paint_rect(mr, if k == 2 { 16.0 } else { 6.0 }, overview_action_bg);
                        f.state_layer(rm, 10.0, overview_ink);
                        paint_sys_icon(
                            f,
                            Rect {
                                x: mr.x + 16.0,
                                y: mr.y + 14.0,
                                width: 20.0,
                                height: 20.0,
                            },
                            ic,
                            overview_ink,
                        );
                        f.styled_text(
                            Rect {
                                x: mr.x + 50.0,
                                y: mr.y,
                                width: mr.width - 62.0,
                                height: mr.height,
                            },
                            lbl,
                            overview_ink,
                            text_style(14.5, 500, 0),
                        );
                        if rm.clicked {
                            self.task_menu_idx = None;
                            match k {
                                0 => {
                                    self.settings_app
                                        .navigate(settings::SettingsPage::AppInfo(app.index()), now);
                                    self.open_app(AppId::Settings, now);
                                }
                                1 => {
                                    self.mode = ShellMode::SplitSelect(app);
                                    self.overview_scroll = 0.0;
                                }
                                _ => {
                                    self.open_freeform(app, phone, now);
                                }
                            }
                        }
                        my += 50.0;
                    }
                }

                if dismiss_alpha < 0.999 {
                    f.pop_layer();
                }
            }

            // "Clear all" pill sitting at the far left end of the carousel
            if split_first.is_none() && drag_progress < 0.35 {
                let clear_x = phone.x + center_offset - entries.len() as f32 * step
                    + self.overview_scroll + carousel_dx
                    + card_w
                    - 100.0;
                if clear_x + 100.0 > phone.x && clear_x < phone.x + phone.width {
                    let clr = Rect {
                        x: clear_x,
                        y: content_top + 56.0 + card_h * 0.5 - 22.0,
                        width: 98.0,
                        height: 44.0,
                    };
                    let rc = f.target(MULTI_ID + 400, clr, "Clear all tasks", true, Role::Button);
                    f.paint_rect(clr, 22.0, overview_action_bg);
                    f.state_layer(rc, 22.0, overview_ink);
                    f.styled_text(clr, "Clear all", overview_ink, text_style(14.0, 500, 1));
                    if rc.clicked {
                        clear_all_clicked = true;
                    }
                }
            }
        });

        // Bottom Overview Action Row (fades smoothly in below the tall phone-aspect card!)
        let act_a = smoothstep(0.40, 1.0, ov_prog) * (1.0 - smoothstep(0.0, 0.55, drag_progress));
        let act_y = (content_top + 56.0 + card_h + 36.0).min(phone.y + phone.height - nav_h - 52.0);
        if act_a < 0.995 {
            f.push_layer(phone, act_a.max(0.01));
        }
        if split_first.is_some() {
            let hint_w = (phone.width - 48.0).min(276.0);
            let hr = Rect {
                x: phone.x + (phone.width - hint_w) * 0.5,
                y: act_y,
                width: hint_w,
                height: 44.0,
            };
            f.paint_rect(hr, 22.0, p.primary_container);
            f.styled_text(
                hr,
                "Tap another app to use split screen",
                p.on_primary_container,
                text_style(13.0, 500, 1),
            );
        } else {
            // Centered Screenshot button + quick Clear all accessibility pill on right
            let shot_r = Rect {
                x: phone.x + phone.width * 0.5 - 66.0,
                y: act_y,
                width: 132.0,
                height: 40.0,
            };
            let rs = f.target(MULTI_ID + 401, shot_r, "Screenshot", true, Role::Button);
            f.state_layer(rs, 20.0, overview_ink);
            paint_sys_icon(
                f,
                Rect {
                    x: shot_r.x + 12.0,
                    y: shot_r.y + 11.0,
                    width: 18.0,
                    height: 18.0,
                },
                SysIcon::Screenshot,
                overview_ink,
            );
            f.styled_text(
                Rect {
                    x: shot_r.x + 36.0,
                    y: shot_r.y,
                    width: 90.0,
                    height: 40.0,
                },
                "Screenshot",
                overview_ink,
                text_style(14.0, 500, 0),
            );
            if rs.clicked {
                self.take_screenshot(now);
            }
        }
        if act_a < 0.995 {
            f.pop_layer();
        }

        // Register after the cards and actions so their touch targets get first refusal.
        let outside = Rect {
            y: phone.y + status_h,
            height: (phone.height - status_h - nav_h).max(0.0),
            ..phone
        };
        let return_home = self.mode == ShellMode::Overview && self.task_menu_idx.is_none()
            && f.target(MULTI_ID + 2, outside, "Return home", true, Role::Button).clicked
            && (!f.input().released || f.input().pointer.is_none_or(|[x, y]| {
                (x - self.press_origin[0]).hypot(y - self.press_origin[1]) < TOUCH_SLOP
            }));

        if let Some(idx) = toggle_menu_idx {
            self.task_menu_idx = if self.task_menu_idx == Some(idx) {
                None
            } else {
                Some(idx)
            };
        }
        if clear_all_clicked {
            self.tasks.clear();
            if self.native_compositor {
                self.host_commands.push(HostCommand::CloseAllApps);
            }
            self.go_home(now);
            self.show_toast("Cleared all recent apps", now);
        } else if let Some((a, b)) = open_pair {
            self.open_split(a, b, now);
        } else if let Some((app, card_r)) = open_single {
            self.reopen_overview_task(TaskEntry::Single(app), card_r, 0.0, now);
        } else if let Some((idx, card_r)) = open_desktop {
            self.reopen_overview_task(TaskEntry::Desktop(idx), card_r, 0.0, now);
        } else if return_home {
            self.go_home(now);
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn render_split_screen(
        &mut self,
        f: &mut Frame<'_>,
        app_rect: Rect,
        phone: Rect,
        top: AppId,
        bottom: AppId,
        pointer: Option<[f32; 2]>,
        now: f64,
        gmail_text: &mut String,
    ) {
        let p = self.sys.palette();
        let div_h = 16.0;
        let top_h = ((app_rect.height - div_h) * self.split_ratio).clamp(96.0, app_rect.height - 112.0);
        let top_rect = Rect {
            x: app_rect.x,
            y: app_rect.y,
            width: app_rect.width,
            height: top_h,
        };
        let div_rect = Rect {
            x: app_rect.x,
            y: app_rect.y + top_h,
            width: app_rect.width,
            height: div_h,
        };
        let bot_rect = Rect {
            x: app_rect.x,
            y: div_rect.y + div_h,
            width: app_rect.width,
            height: (app_rect.height - top_h - div_h).max(96.0),
        };

        // Top status backdrop matches top app
        self.paint_app_backdrop(
            f,
            top,
            Rect {
                x: phone.x,
                y: phone.y,
                width: phone.width,
                height: top_rect.y + top_rect.height - phone.y,
            },
        );
        self.paint_app_backdrop(
            f,
            bottom,
            Rect {
                x: phone.x,
                y: bot_rect.y,
                width: phone.width,
                height: phone.y + phone.height - bot_rect.y,
            },
        );

        // Render BOTH apps LIVE AND INTERACTIVE!
        let act1 = self.render_live_app(
            f,
            top,
            top_rect,
            APP_ID_BASE + ((top as u64) << 28),
            pointer,
            gmail_text,
        );
        let act2 = self.render_live_app(
            f,
            bottom,
            bot_rect,
            APP_ID_BASE + ((bottom as u64) << 28) + (1 << 24),
            pointer,
            gmail_text,
        );
        self.apply_shell_action(act1, now);
        self.apply_shell_action(act2, now);

        // Draggable #FAF9FC Split-Screen Divider (pointer-captured across entire phone by gesture engine!)
        let hit_div = Rect {
            x: div_rect.x,
            y: div_rect.y - 6.0,
            width: div_rect.width,
            height: div_rect.height + 12.0,
        };
        let _rd = f.target(MULTI_ID + 500, hit_div, "Resize split screen", true, Role::Slider);
        f.paint_rect(div_rect, 0.0, p.card_elevated);
        let handle_col = if p.dark {
            Color(0xFF, 0xFF, 0xFF, 235)
        } else {
            Color(0x00, 0x00, 0x00, 240)
        };
        let div_active = matches!(self.gesture, ActiveGesture::SplitDivider);
        f.paint_rect(
            Rect {
                x: div_rect.x + div_rect.width * 0.5 - if div_active { 40.0 } else { 32.0 },
                y: div_rect.y + 6.0,
                width: if div_active { 80.0 } else { 64.0 },
                height: 4.0,
            },
            2.0,
            handle_col,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_freeform_window(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        status_h: f32,
        nav_h: f32,
        mut win: FreeformWin,
        pointer: Option<[f32; 2]>,
        now: f64,
        gmail_text: &mut String,
    ) {
        let p = self.sys.palette();
        let cap_h = 36.0;
        win.x = win.x.clamp(phone.x + 6.0, phone.x + phone.width - win.w - 6.0);
        win.y = win.y.clamp(phone.y + status_h + 6.0, phone.y + phone.height - nav_h - 120.0);
        let frame = Rect {
            x: win.x,
            y: win.y,
            width: win.w,
            height: win.h,
        };
        let cap = Rect {
            height: cap_h,
            ..frame
        };
        let body = Rect {
            x: frame.x,
            y: frame.y + cap_h,
            width: frame.width,
            height: (frame.height - cap_h).max(120.0),
        };

        f.shadow(frame, 12.0, [0.0, 10.0], 26.0, 0.0, Color(0, 0, 0, 95));
        f.paint_rect(frame, 10.0, p.card_pure);

        // #2E4A7D Caption bar matching 09-recents-freeform-window.png!
        f.paint_rect(cap, 0.0, p.primary_dark);
        let max_btn = Rect {
            x: cap.x + cap.width - 72.0,
            y: cap.y + 2.0,
            width: 32.0,
            height: 32.0,
        };
        let close_btn = Rect {
            x: cap.x + cap.width - 36.0,
            y: cap.y + 2.0,
            width: 32.0,
            height: 32.0,
        };
        let drag_bar = Rect {
            x: cap.x,
            y: cap.y,
            width: (cap.width - 76.0).max(40.0),
            height: cap_h,
        };
        let _r_drag = f.target(MULTI_ID + 600, drag_bar, "Move freeform window", true, Role::Button);
        f.styled_text(
            Rect {
                x: cap.x + 12.0,
                y: cap.y,
                width: (cap.width - 92.0).max(20.0),
                height: cap_h,
            },
            win.app.name(),
            Color(255, 255, 255, 240),
            text_style(13.5, 500, 0),
        );

        let r_max = f.target(MULTI_ID + 601, max_btn, "Maximize window", true, Role::Button);
        f.state_layer(r_max, 16.0, Color(255, 255, 255, 255));
        paint_sys_icon(f, max_btn.inset(7.0), SysIcon::Maximize, Color(255, 255, 255, 240));

        let r_cls = f.target(MULTI_ID + 602, close_btn, "Close window", true, Role::Button);
        f.state_layer(r_cls, 16.0, Color(255, 255, 255, 255));
        paint_sys_icon(f, close_btn.inset(7.0), SysIcon::Close, Color(255, 255, 255, 240));

        // Live interactive app content inside freeform window!
        let act = self.render_live_app(
            f,
            win.app,
            body,
            APP_ID_BASE + ((win.app as u64) << 28) + (2 << 24),
            pointer,
            gmail_text,
        );

        // Corner resize grip (bottom-right, pointer-captured by gesture engine!)
        let grip = Rect {
            x: frame.x + frame.width - 24.0,
            y: frame.y + frame.height - 24.0,
            width: 24.0,
            height: 24.0,
        };
        let _r_res = f.target(MULTI_ID + 603, grip, "Resize window", true, Role::Button);
        f.paint_line(
            [grip.x + 8.0, grip.y + 19.0],
            [grip.x + 19.0, grip.y + 8.0],
            1.6,
            p.outline,
        );
        f.paint_line(
            [grip.x + 13.0, grip.y + 19.0],
            [grip.x + 19.0, grip.y + 13.0],
            1.6,
            p.outline,
        );

        if r_cls.clicked {
            self.freeform = None;
        } else if r_max.clicked {
            let app = win.app;
            self.freeform = None;
            self.open_app(app, now);
        } else {
            self.freeform = Some(win);
            self.apply_shell_action(act, now);
        }
    }

    #[inline(never)]
    fn render_gboard(&mut self, f: &mut Frame<'_>, phone: Rect, nav_h: f32, now: f64) {
        let prog = self.ime_anim.value.clamp(0.0, 1.0);
        if prog <= 0.005 {
            return;
        }
        let p = self.sys.palette();
        let kb_h = 236.0;
        let dy = (1.0 - prog) * (kb_h + nav_h);
        let kb = Rect {
            x: phone.x,
            y: phone.y + phone.height - nav_h - kb_h + dy,
            width: phone.width,
            height: kb_h + nav_h,
        };
        let kb_bg = if p.dark {
            Color(0x1B, 0x1D, 0x22, 255)
        } else {
            Color(0xEB, 0xEE, 0xF5, 255)
        };
        let key_bg = if p.dark {
            Color(0x2B, 0x2E, 0x36, 255)
        } else {
            Color(0xFF, 0xFF, 0xFF, 255)
        };
        let fn_bg = if p.dark {
            Color(0x23, 0x26, 0x2E, 255)
        } else {
            Color(0xD8, 0xDE, 0xEC, 255)
        };
        f.within(phone, |f| {
            f.paint_rect(kb, 0.0, kb_bg);
            // Top Gboard toolbar strip
            let bar = Rect { x: kb.x, y: kb.y, width: kb.width, height: 34.0 };
            f.styled_text(
                Rect { x: bar.x + 14.0, y: bar.y, width: 120.0, height: 34.0 },
                "Gboard",
                p.on_surface_variant,
                text_style(12.0, 600, 0),
            );
            let hide_r = Rect { x: bar.x + bar.width - 42.0, y: bar.y + 3.0, width: 34.0, height: 28.0 };
            let rh = f.target(LAUNCHER_ID + 9000, hide_r, "Hide keyboard", true, Role::Button);
            f.state_layer(rh, 14.0, p.on_surface);
            f.styled_text(hide_r, "⌄", p.on_surface_variant, text_style(16.0, 600, 1));
            if rh.clicked {
                self.ime_target = None;
                self.ime_anim.set(0.0, now, 0.20, 0.18);
                return;
            }

            let mut press_char: Option<char> = None;
            let mut do_backspace = false;
            let mut do_enter = false;
            let mut toggle_shift = false;
            let mut toggle_sym = false;

            let pad_x = 6.0;
            let gap = 5.0;
            let row_h = 42.0;
            let row_gap = 7.0;
            let y0 = kb.y + 38.0;

            let rows_alpha: [&[&str]; 3] = [
                &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
                &["a", "s", "d", "f", "g", "h", "j", "k", "l"],
                &["z", "x", "c", "v", "b", "n", "m"],
            ];
            let rows_sym: [&[&str]; 3] = [
                &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
                &["@", "#", "$", "_", "&", "-", "+", "(", ")"],
                &["*", "\"", "'", ":", ";", "!", "?"],
            ];
            let active_rows = if self.ime_symbols { rows_sym } else { rows_alpha };
            let mut key_id = LAUNCHER_ID + 9010;

            // Row 0 (10 keys)
            let w0 = (kb.width - pad_x * 2.0 - gap * 9.0) / 10.0;
            for (i, &k) in active_rows[0].iter().enumerate() {
                let lbl = if self.ime_shift && !self.ime_symbols { k.to_ascii_uppercase() } else { k.to_owned() };
                let kr = Rect { x: kb.x + pad_x + i as f32 * (w0 + gap), y: y0, width: w0, height: row_h };
                let r = f.target(key_id, kr, &lbl, true, Role::Button);
                key_id += 1;
                f.paint_rect(kr, 6.0, key_bg);
                f.state_layer(r, 6.0, p.on_surface);
                f.styled_text(kr, &lbl, p.on_surface, text_style(18.0, 400, 1));
                if r.clicked {
                    press_char = lbl.chars().next();
                }
            }

            // Row 1 (9 keys, indented)
            let y1 = y0 + row_h + row_gap;
            let total_w1 = 9.0 * w0 + 8.0 * gap;
            let x1 = kb.x + (kb.width - total_w1) * 0.5;
            for (i, &k) in active_rows[1].iter().enumerate() {
                let lbl = if self.ime_shift && !self.ime_symbols { k.to_ascii_uppercase() } else { k.to_owned() };
                let kr = Rect { x: x1 + i as f32 * (w0 + gap), y: y1, width: w0, height: row_h };
                let r = f.target(key_id, kr, &lbl, true, Role::Button);
                key_id += 1;
                f.paint_rect(kr, 6.0, key_bg);
                f.state_layer(r, 6.0, p.on_surface);
                f.styled_text(kr, &lbl, p.on_surface, text_style(18.0, 400, 1));
                if r.clicked {
                    press_char = lbl.chars().next();
                }
            }

            // Row 2: Shift + 7 keys + Backspace
            let y2 = y1 + row_h + row_gap;
            let side_w = (kb.width - pad_x * 2.0 - (7.0 * w0 + 8.0 * gap)) * 0.5;
            let sh_r = Rect { x: kb.x + pad_x, y: y2, width: side_w, height: row_h };
            let r_sh = f.target(key_id, sh_r, "Shift", true, Role::Button);
            key_id += 1;
            f.paint_rect(sh_r, 6.0, if self.ime_shift { p.primary_container } else { fn_bg });
            f.state_layer(r_sh, 6.0, p.on_surface);
            f.styled_text(sh_r, if self.ime_symbols { "=\\<" } else { "⇧" }, p.on_surface, text_style(16.0, 600, 1));
            if r_sh.clicked {
                toggle_shift = true;
            }
            for (i, &k) in active_rows[2].iter().enumerate() {
                let lbl = if self.ime_shift && !self.ime_symbols { k.to_ascii_uppercase() } else { k.to_owned() };
                let kr = Rect { x: sh_r.x + side_w + gap + i as f32 * (w0 + gap), y: y2, width: w0, height: row_h };
                let r = f.target(key_id, kr, &lbl, true, Role::Button);
                key_id += 1;
                f.paint_rect(kr, 6.0, key_bg);
                f.state_layer(r, 6.0, p.on_surface);
                f.styled_text(kr, &lbl, p.on_surface, text_style(18.0, 400, 1));
                if r.clicked {
                    press_char = lbl.chars().next();
                }
            }
            let bs_r = Rect { x: kb.x + kb.width - pad_x - side_w, y: y2, width: side_w, height: row_h };
            let r_bs = f.target(key_id, bs_r, "Backspace", true, Role::Button);
            key_id += 1;
            f.paint_rect(bs_r, 6.0, fn_bg);
            f.state_layer(r_bs, 6.0, p.on_surface);
            f.styled_text(bs_r, "⌫", p.on_surface, text_style(17.0, 500, 1));
            if r_bs.clicked {
                do_backspace = true;
            }

            // Row 3: ?123 | , | Space | . | Enter
            let y3 = y2 + row_h + row_gap;
            let sym_w = 54.0;
            let punc_w = 34.0;
            let ent_w = 58.0;
            let sp_w = (kb.width - pad_x * 2.0 - sym_w - punc_w * 2.0 - ent_w - gap * 4.0).max(80.0);
            let mut cx = kb.x + pad_x;
            let sym_r = Rect { x: cx, y: y3, width: sym_w, height: row_h };
            let r_sym = f.target(key_id, sym_r, "Symbols", true, Role::Button);
            key_id += 1;
            f.paint_rect(sym_r, 18.0, fn_bg);
            f.state_layer(r_sym, 18.0, p.on_surface);
            f.styled_text(sym_r, if self.ime_symbols { "ABC" } else { "?123" }, p.on_surface, text_style(13.0, 600, 1));
            if r_sym.clicked {
                toggle_sym = true;
            }
            cx += sym_w + gap;

            let com_r = Rect { x: cx, y: y3, width: punc_w, height: row_h };
            let r_com = f.target(key_id, com_r, ",", true, Role::Button);
            key_id += 1;
            f.paint_rect(com_r, 6.0, fn_bg);
            f.state_layer(r_com, 6.0, p.on_surface);
            f.styled_text(com_r, ",", p.on_surface, text_style(18.0, 500, 1));
            if r_com.clicked {
                press_char = Some(',');
            }
            cx += punc_w + gap;

            let sp_r = Rect { x: cx, y: y3, width: sp_w, height: row_h };
            let r_sp = f.target(key_id, sp_r, "Space", true, Role::Button);
            key_id += 1;
            f.paint_rect(sp_r, 6.0, key_bg);
            f.state_layer(r_sp, 6.0, p.on_surface);
            f.styled_text(sp_r, "English", p.on_surface_variant, text_style(12.0, 400, 1));
            if r_sp.clicked {
                press_char = Some(' ');
            }
            cx += sp_w + gap;

            let dot_r = Rect { x: cx, y: y3, width: punc_w, height: row_h };
            let r_dot = f.target(key_id, dot_r, ".", true, Role::Button);
            key_id += 1;
            f.paint_rect(dot_r, 6.0, fn_bg);
            f.state_layer(r_dot, 6.0, p.on_surface);
            f.styled_text(dot_r, ".", p.on_surface, text_style(18.0, 500, 1));
            if r_dot.clicked {
                press_char = Some('.');
            }
            cx += punc_w + gap;

            let ent_r = Rect { x: cx, y: y3, width: ent_w, height: row_h };
            let r_ent = f.target(key_id, ent_r, "Done", true, Role::Button);
            f.paint_rect(ent_r, 21.0, p.primary);
            f.state_layer(r_ent, 21.0, p.on_primary);
            f.styled_text(ent_r, "↵", p.on_primary, text_style(18.0, 600, 1));
            if r_ent.clicked {
                do_enter = true;
            }

            if toggle_shift {
                self.ime_shift = !self.ime_shift;
            }
            if toggle_sym {
                self.ime_symbols = !self.ime_symbols;
            }
            if let Some(id) = self.ime_target {
                if let Some(ch) = press_char {
                    let cur = self.edit_text_for(id).to_owned();
                    let mut chars: Vec<char> = cur.chars().collect();
                    let pos = self.ime_cursor.min(chars.len());
                    chars.insert(pos, ch);
                    let next: String = chars.into_iter().collect();
                    self.ime_cursor = pos + 1;
                    self.set_edit_text(id, &next);
                    if self.ime_shift && !self.ime_symbols {
                        self.ime_shift = false;
                    }
                } else if do_backspace {
                    let cur = self.edit_text_for(id).to_owned();
                    let mut chars: Vec<char> = cur.chars().collect();
                    let pos = self.ime_cursor.min(chars.len());
                    if pos > 0 {
                        chars.remove(pos - 1);
                        let next: String = chars.into_iter().collect();
                        self.ime_cursor = pos - 1;
                        self.set_edit_text(id, &next);
                    }
                } else if do_enter {
                    self.ime_target = None;
                    self.ime_anim.set(0.0, now, 0.20, 0.18);
                }
            }
        });
    }

    fn render_live_app(
        &mut self,
        f: &mut Frame<'_>,
        app: AppId,
        bounds: Rect,
        id_base: u64,
        pointer: Option<[f32; 2]>,
        gmail_text: &mut String,
    ) -> ShellAction {
        match app {
            AppId::Contacts => {
                self.embedded.render_contacts(f, bounds);
                ShellAction::None
            }
            AppId::Gmail => {
                self.embedded.render_gmail(f, bounds, gmail_text);
                ShellAction::None
            }
            AppId::Settings => {
                let unread = self.notifications.iter().filter(|n| !n.snoozed && !n.dismissing).count();
                let installed = &self.sys.shiraz.installed_apps;
                if self.settings_apps.as_ref().is_some_and(|apps| !apps.matches(installed)) {
                    self.settings_apps = None;
                }
                let catalog = self.settings_apps.get_or_insert_with(||
                    settings_apps::SettingsApps::new(&self.desktop_registry, installed));
                if catalog.poll() {
                    f.wake_at(f.time() + 0.05);
                }
                let builtin_len = AppId::ALL.len();
                let paint_icon = |f: &mut Frame<'_>, r: Rect, idx: usize, pal: &Palette| {
                    if let Some(a) = AppId::from_index(idx) {
                        icons::paint_real_app_icon(self.clock, f, r, a, false, pal);
                    } else if let Some(rel) = idx.checked_sub(builtin_len)
                        && let Some(app_meta) = catalog.apps.get(idx)
                    {
                        let img = catalog.icons.get(rel).and_then(|o| o.as_ref());
                        freedesktop::paint_desktop_app_icon(
                            f,
                            r,
                            &app_meta.name,
                            app_meta.desktop_id.as_deref().unwrap_or(&app_meta.id),
                            img,
                            pal,
                        );
                    }
                };
                let ctx = settings::SettingsHostContext {
                    clock: &self.clock.hm(),
                    date: &self.clock.date(),
                    apps: &catalog.apps,
                    active_notif_count: unread,
                    history: &self.notif_history,
                    device: &self.device_info,
                    paint_app_icon: Some(&paint_icon),
                };
                match self
                    .settings_app
                    .show(f, bounds, SETTINGS_ID, &mut self.sys, pointer, &ctx)
                {
                    settings::SettingsAction::None => ShellAction::None,
                    settings::SettingsAction::JoinWifi(network) => ShellAction::JoinWifi(network),
                    settings::SettingsAction::ConnectBluetooth(address, connect) => ShellAction::ConnectBluetooth(address, connect),
                    settings::SettingsAction::AdvancedRadioSettings(panel) => ShellAction::AdvancedRadioSettings(panel),
                    settings::SettingsAction::OpenApp(idx) => {
                        if let Some(a) = AppId::from_index(idx) {
                            ShellAction::OpenApp(a)
                        } else if let Some(rel) = idx.checked_sub(builtin_len)
                            && let Some(&di) = catalog.desktop_indices.get(rel)
                        {
                            ShellAction::OpenDesktopApp(di)
                        } else {
                            ShellAction::None
                        }
                    }
                    settings::SettingsAction::ShowToast(msg) => ShellAction::ShowToast(msg),
                    settings::SettingsAction::PostTestNotification { app, title, body } => {
                        ShellAction::PostNotification { app, title, body }
                    }
                    settings::SettingsAction::ClearNotificationHistory => {
                        ShellAction::ClearNotificationHistory
                    }
                    settings::SettingsAction::OpenVolume => ShellAction::OpenVolume,
                    settings::SettingsAction::LockNow => ShellAction::LockNow,
                    settings::SettingsAction::ShirazLaunchPackage(pkg) => {
                        ShellAction::ShirazLaunchPackage(pkg)
                    }
                    settings::SettingsAction::ShirazStopPackage(pkg) => {
                        ShellAction::ShirazStopPackage(pkg)
                    }
                    settings::SettingsAction::ShirazStopAll => ShellAction::ShirazStopAll,
                    settings::SettingsAction::ShirazRefresh => ShellAction::ShirazRefresh,
                }
            }
            _ => self
                .builtin_apps
                .show(f, app, bounds, id_base, &mut self.sys, pointer, self.clock),
        }
    }
}

// =============================================================================
// SHARED PAINTING HELPERS: WALLPAPER, APP ICONS, SYSTEM ICONS & CONTROLS
// =============================================================================

fn cursor_index_for_x(f: &Frame<'_>, text: &str, style: TextStyle, rel_x: f32) -> usize {
    let mut best_idx = 0;
    let mut best_dist = rel_x.abs();
    let mut prefix = String::new();
    for (i, ch) in text.chars().enumerate() {
        prefix.push(ch);
        let w = f.text_width(&prefix, style);
        let d = (rel_x - w).abs();
        if d <= best_dist {
            best_dist = d;
            best_idx = i + 1;
        }
    }
    best_idx
}

fn mask_rounded_corners(f: &mut Frame<'_>, rect: Rect, r: f32, bg: Color) {
    f.mask_rounded_corners(rect, r, bg);
}

pub fn paint_wallpaper(f: &mut Frame<'_>, b: Rect, idx: usize) {
    // Solid Material You color wallpapers (indices 4..=7, zero bundle size)
    if let Some(solid) = match idx % 8 {
        4 => Some(Color(0x1E, 0x3A, 0x5F, 255)), // Deep Ocean Blue
        5 => Some(Color(0x2B, 0x4A, 0x3B, 255)), // Forest Sage
        6 => Some(Color(0x6B, 0x3A, 0x2E, 255)), // Warm Terracotta
        7 => Some(Color(0x18, 0x1A, 0x20, 255)), // Midnight Charcoal
        _ => None,
    } {
        f.paint_rect(b, 0.0, solid);
        return;
    }
    let (base, dark, warm, slate, accent) = match idx % 8 {
        1 => (
            Color(0x2D, 0x4D, 0x3E, 255),
            Color(0x0B, 0x1D, 0x16, 255),
            Color(0xD3, 0xDF, 0xC8, 255),
            Color(0x46, 0x6B, 0x57, 255),
            Color(0x1E, 0x38, 0x2C, 255),
        ),
        2 => (
            Color(0x3F, 0x37, 0x61, 255),
            Color(0x12, 0x0E, 0x22, 255),
            Color(0xD5, 0xBE, 0xD9, 255),
            Color(0x5E, 0x52, 0x87, 255),
            Color(0x2A, 0x22, 0x47, 255),
        ),
        3 => (
            Color(0x5B, 0x44, 0x32, 255),
            Color(0x1A, 0x13, 0x0E, 255),
            Color(0xE6, 0xCE, 0xAE, 255),
            Color(0x88, 0x62, 0x44, 255),
            Color(0x3D, 0x2A, 0x1C, 255),
        ),
        _ => (
            // Exact obsidian + slate + warm sand composition matching 01-home-main.png!
            Color(0x5F, 0x67, 0x76, 255),
            Color(0x0B, 0x0E, 0x13, 255),
            Color(0xC7, 0xB7, 0xA6, 255),
            Color(0x6D, 0x75, 0x84, 255),
            Color(0x15, 0x1A, 0x22, 255),
        ),
    };
    f.within(b, |f| {
        f.paint_rect(b, 0.0, base);
        let w = b.width;
        let h = b.height;
        // Obsidian curved mountain mass left-to-bottom-right
        f.paint_rect(
            Rect {
                x: b.x - w * 0.42,
                y: b.y + h * 0.04,
                width: w * 1.12,
                height: h * 0.92,
            },
            w * 0.56,
            dark,
        );
        // Large warm sand hemisphere sweeping from upper-right
        f.paint_rect(
            Rect {
                x: b.x + w * 0.16,
                y: b.y + h * 0.08,
                width: w * 1.18,
                height: h * 0.52,
            },
            w * 0.55,
            warm,
        );
        // Subtle sand dune strata bands
        f.paint_rect(
            Rect {
                x: b.x + w * 0.28,
                y: b.y + h * 0.36,
                width: w * 0.92,
                height: h * 0.14,
            },
            w * 0.12,
            warm.mix(dark, 0.14).opacity(0.45),
        );
        // Slate sweeping upper-center organic curve (signature Material You loop)
        f.paint_rect(
            Rect {
                x: b.x + w * 0.16,
                y: b.y + h * 0.12,
                width: w * 0.52,
                height: h * 0.30,
            },
            w * 0.26,
            slate,
        );
        // Overlapping dark wedge cutout in center
        f.paint_rect(
            Rect {
                x: b.x + w * 0.12,
                y: b.y + h * 0.33,
                width: w * 0.54,
                height: h * 0.26,
            },
            w * 0.22,
            accent,
        );
        // Lower-right obsidian sweep
        f.paint_rect(
            Rect {
                x: b.x + w * 0.48,
                y: b.y + h * 0.60,
                width: w * 0.78,
                height: h * 0.42,
            },
            w * 0.34,
            dark,
        );
        // Center-right slate petal overlap
        f.paint_rect(
            Rect {
                x: b.x + w * 0.40,
                y: b.y + h * 0.40,
                width: w * 0.66,
                height: h * 0.19,
            },
            w * 0.18,
            slate.opacity(0.92),
        );
        // Bottom slate curve under dock
        f.paint_rect(
            Rect {
                x: b.x + w * 0.04,
                y: b.y + h * 0.57,
                width: w * 0.68,
                height: h * 0.48,
            },
            w * 0.34,
            slate.opacity(0.95),
        );
        // Subtle textured star/ridge flecks on obsidian region
        for &(fx, fy, sz) in &[
            (0.08, 0.28, 2.0),
            (0.14, 0.34, 2.5),
            (0.22, 0.42, 2.0),
            (0.06, 0.47, 2.5),
            (0.17, 0.62, 2.0),
            (0.84, 0.80, 2.5),
            (0.76, 0.85, 2.0),
            (0.91, 0.88, 2.0),
        ] {
            f.paint_rect(
                Rect {
                    x: b.x + w * fx,
                    y: b.y + h * fy,
                    width: sz,
                    height: sz,
                },
                sz * 0.5,
                Color(255, 255, 255, 75),
            );
        }
    });
}

/// Asymmetric vertical rounded rectangle (independent top and bottom corner radii) with zero seam.
pub(crate) fn paint_asym_rect(f: &mut Frame<'_>, r: Rect, top_rad: f32, bot_rad: f32, color: Color) {
    f.paint_asym_rect(r, top_rad, bot_rad, color);
}

/// Real launcher app icons extracted directly from the LineageOS 20 device (`icons.rs`).
pub fn paint_app_icon(f: &mut Frame<'_>, r: Rect, app: AppId, themed: bool, p: &Palette) {
    icons::paint_real_app_icon(TimeService::default().now(), f, r, app, themed, p);
}

/// Official Google Material Outlined 96px high-res system icons (`icons.rs`).
pub fn paint_sys_icon(f: &mut Frame<'_>, r: Rect, kind: SysIcon, c: Color) {
    icons::paint_real_sys_icon(f, r, kind, c);
}

#[cfg(target_arch = "wasm32")]
impl<E: EmbeddedApps> LauncherDemo<E> {
    /// Toggle Wisp's system dark theme from the browser debug workbench (compiled on `wasm32` only).
    pub fn set_dark_theme(&mut self, dark: bool) {
        self.sys.dark_theme = dark;
    }

    /// Clear all dismissible notifications (compiled on `wasm32` only).
    pub fn clear_all_debug_notifications(&mut self, now: f64) {
        self.clear_all_dismissible_notifications(now);
    }

    /// Render the production heads-up notification banner alone over the home wallpaper
    /// for interactive browser debugging (`wasm32` only; zero bytes in native/phone builds).
    pub fn show_heads_up_debug(&mut self, f: &mut Frame<'_>, pinned: bool) {
        let now = f.input().time;
        self.clock = self.time_service.now();
        self.lockscreen.locked = false;
        self.sys.dnd_enabled = false;
        self.shade_stage = 0;
        self.power_open = false;

        if pinned && let Some(idx) = self.heads_up_index(now) {
            self.notifications[idx].heads_up_until = Some(f64::INFINITY);
        }

        let viewport = f.viewport();
        let phone = Rect {
            height: viewport.width * 2960.0 / 1440.0,
            ..viewport
        };
        f.within(viewport, |f| {
            self.wallpapers.home.paint(f, phone, self.sys.wallpaper_idx);
        });
        if let Some(idx) = self.heads_up_index(now) {
            self.render_heads_up_source(
                f,
                Rect { height: 30.0, ..viewport },
                idx,
                Color(255, 255, 255, 255),
            );
        }
        self.render_heads_up_banner(f, viewport, 38.0, now);
    }

    /// Snapshot live notification & shade state as JSON for the browser debugger inspector.
    pub fn debug_notifications_state_json(&self, mode: u32, now: f64) -> String {
        fn esc(s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            for ch in s.chars() {
                match ch {
                    '\\' => out.push_str("\\\\"),
                    '"' => out.push_str("\\\""),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if (c as u32) < 0x20 => {}
                    c => out.push(c),
                }
            }
            out
        }

        let mode_label = match mode {
            0 => "heads_up_pinned",
            1 => "shade",
            2 => "home_heads_up",
            3 => "lockscreen",
            _ => "other",
        };
        let heads_up_id = self
            .heads_up_index(now)
            .map(|i| self.notifications[i].id.to_string())
            .unwrap_or_else(|| "null".into());
        let heads_up_rect = self
            .last_heads_up_rect
            .map(|r| {
                format!(
                    "[{},{},{},{}]",
                    r.x.round() as i32,
                    r.y.round() as i32,
                    r.width.round() as i32,
                    r.height.round() as i32
                )
            })
            .unwrap_or_else(|| "null".into());

        let mut items = String::from("[");
        for (i, n) in self.notifications.iter().enumerate() {
            if i > 0 {
                items.push(',');
            }
            items.push_str(&format!(
                "{{\"id\":{},\"app\":\"{}\",\"title\":\"{}\",\"urgency\":{},\"expanded\":{},\"silent\":{},\"ongoing\":{},\"dismissing\":{},\"actions\":{}}}",
                n.id,
                esc(&n.app),
                esc(&n.title),
                n.urgency,
                n.expanded,
                n.silent,
                n.ongoing,
                n.dismissing,
                n.actions.len(),
            ));
        }
        items.push(']');

        format!(
            "{{\"mode\":\"{}\",\"shadeStage\":{},\"locked\":{},\"dnd\":{},\"darkTheme\":{},\"activeHeadsUpId\":{},\"headsUpRect\":{},\"count\":{},\"historyCount\":{},\"notifications\":{}}}",
            mode_label,
            self.shade_stage,
            self.lockscreen.locked,
            self.sys.dnd_enabled,
            self.sys.dark_theme,
            heads_up_id,
            heads_up_rect,
            self.notifications.len(),
            self.notif_history.len(),
            items,
        )
    }
}

#[cfg(test)]
#[path = "lib.spec.rs"]
mod tests;

#[cfg(feature = "remote_control")]
pub mod remote_control;
