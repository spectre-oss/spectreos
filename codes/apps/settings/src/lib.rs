//! SpectreOS Settings — Android 13 / AOSP-style System Settings application built on `libbhoo`.
//!
//! Can run either:
//! - As a standalone native Wayland application (`spectre-settings`, `--features wayland`), syncing
//!   preferences live via `~/.config/wisp/{settings,lock}.conf` and Linux hardware interfaces; OR
//! - Embedded directly inside Wisp / WebAssembly hosts via `SettingsApp::show(...)`.

pub mod keypad;
pub mod state;
pub mod widgets;
mod time_zone;

use bhoo::{Color, Frame, Rect, Role, SystemIcon};
pub use keypad::{KeyPadEvent, KeyboardEvent, SoftKeyboardState, paint_pin_keypad, paint_qwerty_keyboard};
pub use state::{
    ClockPos, DeviceRuntimeInfo, DnsMode, ID_DNS_HOSTNAME, ID_SETTINGS_SEARCH, InstalledAppInfo,
    LockConfig, LockMethod, LockScreenNotifMode, NavMode, NotificationHistoryEntry, Palette,
    RingerMode, ShirazInstalledApp, ShirazRuntimeInfo, ShirazRuntimeState, SystemState,
    load_lock_config, load_shiraz_runtime_info, save_lock_config,
};
#[cfg(not(target_arch = "wasm32"))]
pub use state::{
    detect_shiraz_root, parse_shiraz_status_json, xdg_config_home, xdg_data_home, xdg_runtime_dir,
};
pub use widgets::{
    emphasized_decelerate, lerp, paint_h_slider, paint_lock_notif_figure, paint_navigation_figure,
    paint_pref_row, paint_pref_switch, paint_radio, paint_switch, smoothstep, text_style,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsPage {
    Home,
    Search,
    Network,
    Internet,
    Connected,
    Apps,
    Shiraz,
    AppInfo(usize),
    Notifications,
    NotificationHistory,
    Battery,
    Storage,
    Sound,
    Display,
    Accessibility,
    Security,
    ScreenLock,
    Privacy,
    Location,
    Safety,
    Accounts,
    System,
    DateTime,
    TimeZoneRegion(String),
    Gestures,
    Navigation,
    LineageButtons,
    LineageStatusBar,
    DevOptions,
    About,
}

impl SettingsPage {
    pub fn from_cli_arg(arg: &str) -> Self {
        match arg.trim().to_ascii_lowercase().as_str() {
            "search" => Self::Search,
            "network" => Self::Network,
            "internet" | "wifi" => Self::Internet,
            "connected" | "bluetooth" => Self::Connected,
            "apps" => Self::Apps,
            "shiraz" | "android" | "android-apps" => Self::Shiraz,
            "notifications" | "notif" => Self::Notifications,
            "notification-history" | "history" => Self::NotificationHistory,
            "battery" => Self::Battery,
            "storage" => Self::Storage,
            "sound" | "volume" => Self::Sound,
            "display" => Self::Display,
            "accessibility" => Self::Accessibility,
            "security" => Self::Security,
            "screen-lock" | "lock" => Self::ScreenLock,
            "privacy" => Self::Privacy,
            "location" => Self::Location,
            "safety" => Self::Safety,
            "accounts" => Self::Accounts,
            "system" => Self::System,
            "date-time" | "timezone" => Self::DateTime,
            "gestures" => Self::Gestures,
            "navigation" => Self::Navigation,
            "buttons" => Self::LineageButtons,
            "statusbar" => Self::LineageStatusBar,
            "dev" | "developer" => Self::DevOptions,
            "about" => Self::About,
            _ => Self::Home,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SettingsAction {
    None,
    OpenApp(usize),
    JoinWifi(state::WifiNetwork),
    ConnectBluetooth(String, bool),
    AdvancedRadioSettings(&'static str),
    ShowToast(String),
    PostTestNotification {
        app: String,
        title: String,
        body: String,
    },
    ClearNotificationHistory,
    OpenVolume,
    LockNow,
    ShirazLaunchPackage(String),
    ShirazStopPackage(String),
    ShirazStopAll,
    ShirazRefresh,
}

/// Host-supplied live context for rendering Settings pages (apps list, active notifications,
/// notification history, and optional high-res app icon painter).
pub struct SettingsHostContext<'a> {
    pub clock: &'a str,
    pub date: &'a str,
    pub apps: &'a [InstalledAppInfo],
    pub active_notif_count: usize,
    pub history: &'a [NotificationHistoryEntry],
    pub device: &'a DeviceRuntimeInfo,
    pub paint_app_icon: Option<&'a dyn Fn(&mut Frame<'_>, Rect, usize, &Palette)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsDialog {
    None,
    PrivateDns,
    ScreenTimeout,
    AnimScale,
    LockScreenNotifs,
}

#[derive(Clone, Debug)]
struct SettingsTransition {
    from_page: SettingsPage,
    from_scroll: f32,
    is_push: bool,
    is_search: bool,
    start: f64,
}

struct SearchIndexEntry {
    title: &'static str,
    summary: Option<&'static str>,
    breadcrumb: &'static str,
    keywords: &'static str,
    target: fn() -> SettingsPage,
}

const SEARCH_INDEX: &[SearchIndexEntry] = &[
    SearchIndexEntry {
        title: "Date & time",
        summary: Some("Choose the system time zone"),
        breadcrumb: "System",
        keywords: "date time timezone time zone city region clock Vienna",
        target: || SettingsPage::DateTime,
    },
    SearchIndexEntry {
        title: "Display cutout",
        summary: None,
        breadcrumb: "System > Developer options",
        keywords: "display notch cutout developer",
        target: || SettingsPage::DevOptions,
    },
    SearchIndexEntry {
        title: "Display size",
        summary: Some("Make everything bigger or smaller"),
        breadcrumb: "Display > Display size and text",
        keywords: "display size font text scale dpi",
        target: || SettingsPage::Accessibility,
    },
    SearchIndexEntry {
        title: "Display",
        summary: Some("Brightness, dark theme, Night Light, timeout"),
        breadcrumb: "Display",
        keywords: "display brightness screen dark theme",
        target: || SettingsPage::Display,
    },
    SearchIndexEntry {
        title: "Notifications on lock screen",
        summary: Some("Show all, hide silent, hide sensitive, or don't show"),
        breadcrumb: "Notifications",
        keywords: "notifications lock screen hide sensitive silent privacy",
        target: || SettingsPage::Notifications,
    },
    SearchIndexEntry {
        title: "Sensitive notifications",
        summary: Some("Show or hide private content when locked"),
        breadcrumb: "Notifications",
        keywords: "sensitive lock screen private hidden notifications",
        target: || SettingsPage::Notifications,
    },
    SearchIndexEntry {
        title: "Notification history",
        summary: Some("Recent and dismissed notifications"),
        breadcrumb: "Notifications",
        keywords: "notifications history alerts dismissed recent",
        target: || SettingsPage::NotificationHistory,
    },
    SearchIndexEntry {
        title: "Notification dots on app icons",
        summary: None,
        breadcrumb: "Notifications",
        keywords: "dots badges icon notifications",
        target: || SettingsPage::Notifications,
    },
    SearchIndexEntry {
        title: "Do Not Disturb",
        summary: Some("Silence interruptions and heads-up popups"),
        breadcrumb: "Notifications",
        keywords: "dnd silent disturb quiet notifications",
        target: || SettingsPage::Notifications,
    },
    SearchIndexEntry {
        title: "Dark theme",
        summary: Some("Use dark background across system UI"),
        breadcrumb: "Display",
        keywords: "dark theme night mode black display",
        target: || SettingsPage::Display,
    },
    SearchIndexEntry {
        title: "Night Light",
        summary: Some("Tint screen amber for evening reading"),
        breadcrumb: "Display",
        keywords: "night light amber blue filter display",
        target: || SettingsPage::Display,
    },
    SearchIndexEntry {
        title: "Brightness level",
        summary: Some("Adaptive brightness and screen dimmer"),
        breadcrumb: "Display",
        keywords: "brightness adaptive screen display",
        target: || SettingsPage::Display,
    },
    SearchIndexEntry {
        title: "Screen timeout",
        summary: Some("Turn off display after inactivity"),
        breadcrumb: "Display",
        keywords: "timeout sleep lock screen display",
        target: || SettingsPage::Display,
    },
    SearchIndexEntry {
        title: "Wi-Fi",
        summary: Some("Manage saved wireless networks"),
        breadcrumb: "Network & internet",
        keywords: "wifi internet wireless lte",
        target: || SettingsPage::Internet,
    },
    SearchIndexEntry {
        title: "Private DNS",
        summary: Some("Encrypted DNS provider hostname"),
        breadcrumb: "Network & internet",
        keywords: "dns private cloudflare quad9 hostname",
        target: || SettingsPage::Network,
    },
    SearchIndexEntry {
        title: "Airplane mode",
        summary: Some("Disable all wireless radios"),
        breadcrumb: "Network & internet",
        keywords: "airplane flight offline radio",
        target: || SettingsPage::Network,
    },
    SearchIndexEntry {
        title: "Bluetooth",
        summary: Some("Pair headphones and accessories"),
        breadcrumb: "Connected devices",
        keywords: "bluetooth pair buds audio device",
        target: || SettingsPage::Connected,
    },
    SearchIndexEntry {
        title: "All apps",
        summary: Some("App info, storage & force stop"),
        breadcrumb: "Apps",
        keywords: "apps installed contacts gmail clock manage",
        target: || SettingsPage::Apps,
    },
    SearchIndexEntry {
        title: "Enable Shiraz",
        summary: Some("Android app compatibility runtime & memory optimization"),
        breadcrumb: "Shiraz (Android apps)",
        keywords: "shiraz android apk enable runtime warm memory art dalvik osmand deltachat",
        target: || SettingsPage::Shiraz,
    },
    SearchIndexEntry {
        title: "Pre-warm Android runtime",
        summary: Some("Keep ART, framework & app libraries hot in memory"),
        breadcrumb: "Shiraz (Android apps)",
        keywords: "shiraz prewarm memory cache fast launch aot dex2oat android",
        target: || SettingsPage::Shiraz,
    },
    SearchIndexEntry {
        title: "AOT boot image & app optimization",
        summary: Some("Pre-compile Android DEX bytecode with dex2oat"),
        breadcrumb: "Shiraz (Android apps)",
        keywords: "shiraz aot dex2oat odex boot image optimize android",
        target: || SettingsPage::Shiraz,
    },
    SearchIndexEntry {
        title: "Battery Saver",
        summary: Some("Reduce background activity"),
        breadcrumb: "Battery",
        keywords: "battery saver power charge",
        target: || SettingsPage::Battery,
    },
    SearchIndexEntry {
        title: "Storage",
        summary: Some("Internal storage usage breakdown"),
        breadcrumb: "Storage",
        keywords: "storage disk space memory files",
        target: || SettingsPage::Storage,
    },
    SearchIndexEntry {
        title: "Volume & vibration",
        summary: Some("Media, ring, notification & alarm"),
        breadcrumb: "Sound & vibration",
        keywords: "sound volume ring media alarm vibrate silent",
        target: || SettingsPage::Sound,
    },
    SearchIndexEntry {
        title: "Screen lock & passwords",
        summary: Some("Swipe, PIN, password, auto-lock"),
        breadcrumb: "Security",
        keywords: "password pass pin lock security screen",
        target: || SettingsPage::Security,
    },
    SearchIndexEntry {
        title: "Camera & microphone access",
        summary: Some("Privacy permissions dashboard"),
        breadcrumb: "Privacy",
        keywords: "privacy camera mic microphone permissions",
        target: || SettingsPage::Privacy,
    },
    SearchIndexEntry {
        title: "System navigation",
        summary: Some("Gesture navigation or 3-button navigation"),
        breadcrumb: "System > Gestures",
        keywords: "navigation gesture back swipe 3 button pill",
        target: || SettingsPage::Navigation,
    },
    SearchIndexEntry {
        title: "Status bar",
        summary: Some("Clock position, battery percentage"),
        breadcrumb: "System",
        keywords: "status bar clock position left center right",
        target: || SettingsPage::LineageStatusBar,
    },
    SearchIndexEntry {
        title: "Developer options",
        summary: Some("USB debugging, animation scale, show taps"),
        breadcrumb: "System",
        keywords: "developer usb debugging adb animation scale taps",
        target: || SettingsPage::DevOptions,
    },
    SearchIndexEntry {
        title: "About phone",
        summary: Some("SpectreOS · Poco F1 · Kernel & Build number"),
        breadcrumb: "About phone",
        keywords: "about phone android spectreos kernel version build number",
        target: || SettingsPage::About,
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
enum ScreenLockStep {
    VerifyCurrent,
    Choose,
    EnterNew(LockMethod),
    ConfirmNew(LockMethod, String),
}

pub struct SettingsApp {
    time_zones: time_zone::TimeZones,
    zone_page_active: bool,
    page: SettingsPage,
    stack: Vec<SettingsPage>,
    search: String,
    recent_searches: Vec<String>,
    search_menu_open: bool,
    focus_search_pending: bool,
    dns_draft: String,
    dns_draft_mode: DnsMode,
    lock_step: ScreenLockStep,
    lock_entry: String,
    lock_error: Option<String>,
    lock_kbd: SoftKeyboardState,
    scroll: f32,
    sub_scroll: f32,
    search_scroll: f32,
    last_max_scroll: f32,
    dialog: SettingsDialog,
    dialog_start: f64,
    transition: Option<SettingsTransition>,
    edge_stretch: (f32, f32),
}

impl Default for SettingsApp {
    fn default() -> Self {
        Self {
            time_zones: Default::default(),
            zone_page_active: false,
            page: SettingsPage::Home,
            stack: Vec::new(),
            search: String::new(),
            recent_searches: vec![
                "notifications".into(),
                "lock".into(),
                "display".into(),
                "timeout".into(),
                "wifi".into(),
            ],
            search_menu_open: false,
            focus_search_pending: false,
            dns_draft: "dns.quad9.net".into(),
            dns_draft_mode: DnsMode::Automatic,
            lock_step: ScreenLockStep::Choose,
            lock_entry: String::new(),
            lock_error: None,
            lock_kbd: SoftKeyboardState::default(),
            scroll: 0.0,
            sub_scroll: 0.0,
            search_scroll: 0.0,
            last_max_scroll: 0.0,
            dialog: SettingsDialog::None,
            dialog_start: 0.0,
            transition: None,
            edge_stretch: (1.0, 0.0),
        }
    }
}

pub const TIMEOUT_LABELS: &[&str] = &[
    "15 seconds",
    "30 seconds",
    "1 minute",
    "2 minutes",
    "5 minutes",
    "10 minutes",
];
const ANIM_LABELS: &[&str] = &[
    "Animation off",
    "Animation scale .5x",
    "Animation scale 1x",
    "Animation scale 1.5x",
    "Animation scale 2x",
    "Animation scale 5x",
    "Animation scale 10x",
];
impl SettingsApp {
    pub fn is_dialog_open(&self) -> bool {
        self.dialog != SettingsDialog::None
    }

    pub fn current_page(&self) -> SettingsPage {
        self.page.clone()
    }

    pub fn scroll_state_mut(&mut self) -> (&mut f32, f32) {
        match self.page {
            SettingsPage::Home => (&mut self.scroll, self.last_max_scroll),
            SettingsPage::Search => (&mut self.search_scroll, self.last_max_scroll),
            _ => (&mut self.sub_scroll, self.last_max_scroll),
        }
    }

    fn current_scroll_value(&self, page: &SettingsPage) -> f32 {
        match page {
            SettingsPage::Home => self.scroll,
            SettingsPage::Search => self.search_scroll,
            _ => self.sub_scroll,
        }
    }

    pub fn open_search(&mut self, now: f64) {
        if self.page == SettingsPage::Search {
            self.focus_search_pending = true;
            return;
        }
        let prev = self.page.clone();
        let prev_scroll = self.current_scroll_value(&prev);
        self.stack.push(prev.clone());
        self.page = SettingsPage::Search;
        self.search_scroll = 0.0;
        self.search_menu_open = false;
        self.focus_search_pending = true;
        self.dialog = SettingsDialog::None;
        self.transition = Some(SettingsTransition {
            from_page: prev,
            from_scroll: prev_scroll,
            is_push: true,
            is_search: true,
            start: now,
        });
    }

    pub fn set_search_preview(&mut self, query: &str) {
        self.page = SettingsPage::Search;
        self.stack.clear();
        self.stack.push(SettingsPage::Home);
        self.search = query.to_owned();
        self.search_scroll = 0.0;
        self.search_menu_open = false;
        self.transition = None;
    }

    pub fn set_edge_stretch(&mut self, stretch: (f32, f32)) {
        self.edge_stretch = stretch;
    }

    pub fn set_home_scroll(&mut self, scroll_y: f32) {
        self.page = SettingsPage::Home;
        self.stack.clear();
        self.scroll = scroll_y.max(0.0);
        self.transition = None;
    }

    pub fn set_subpage_preview(&mut self, page: SettingsPage, scroll_y: f32) {
        self.page = page;
        self.stack.clear();
        self.stack.push(SettingsPage::Home);
        self.sub_scroll = scroll_y.max(0.0);
        self.transition = None;
    }

    pub fn navigate(&mut self, next: SettingsPage, now: f64) {
        if next == SettingsPage::Home {
            self.stack.clear();
            self.page = SettingsPage::Home;
            self.dialog = SettingsDialog::None;
            self.search_menu_open = false;
            self.transition = None;
            return;
        }
        if next == SettingsPage::Search {
            self.open_search(now);
            return;
        }
        if self.page != next {
            let prev = self.page.clone();
            let prev_scroll = self.current_scroll_value(&prev);
            self.stack.push(prev.clone());
            self.page = next;
            self.sub_scroll = 0.0;
            self.search_menu_open = false;
            self.dialog = SettingsDialog::None;
            self.transition = Some(SettingsTransition {
                from_page: prev,
                from_scroll: prev_scroll,
                is_push: true,
                is_search: false,
                start: now,
            });
        }
    }

    pub fn go_back(&mut self, now: f64) -> bool {
        if self.dialog != SettingsDialog::None {
            self.dialog = SettingsDialog::None;
            return true;
        }
        if self.search_menu_open {
            self.search_menu_open = false;
            return true;
        }
        if let Some(prev) = self.stack.pop() {
            let cur = self.page.clone();
            let cur_scroll = self.current_scroll_value(&cur);
            let is_search = cur == SettingsPage::Search;
            self.page = prev.clone();
            if is_search {
                self.search.clear();
                self.focus_search_pending = false;
            }
            if prev != SettingsPage::Home && prev != SettingsPage::Search {
                self.sub_scroll = 0.0;
            }
            self.transition = Some(SettingsTransition {
                from_page: cur,
                from_scroll: cur_scroll,
                is_push: false,
                is_search,
                start: now,
            });
            true
        } else if self.page != SettingsPage::Home {
            let cur = self.page.clone();
            let cur_scroll = self.current_scroll_value(&cur);
            let is_search = cur == SettingsPage::Search;
            self.page = SettingsPage::Home;
            if is_search {
                self.search.clear();
                self.focus_search_pending = false;
            }
            self.sub_scroll = 0.0;
            self.transition = Some(SettingsTransition {
                from_page: cur,
                from_scroll: cur_scroll,
                is_push: false,
                is_search,
                start: now,
            });
            true
        } else {
            false
        }
    }

    fn record_recent_search(&mut self, term: &str) {
        let trimmed = term.trim();
        if trimmed.is_empty() {
            return;
        }
        let lower = trimmed.to_ascii_lowercase();
        self.recent_searches
            .retain(|s| s.to_ascii_lowercase() != lower);
        self.recent_searches.insert(0, trimmed.to_owned());
        if self.recent_searches.len() > 8 {
            self.recent_searches.truncate(8);
        }
    }

    pub fn edit_text_for(&self, id: u64) -> &str {
        match id {
            ID_SETTINGS_SEARCH => &self.search,
            ID_DNS_HOSTNAME => &self.dns_draft,
            _ => "",
        }
    }

    pub fn set_edit_text(&mut self, id: u64, value: &str) {
        match id {
            ID_SETTINGS_SEARCH => {
                self.search = value.to_owned();
                self.search_scroll = 0.0;
            }
            ID_DNS_HOSTNAME => {
                self.dns_draft = value.to_owned();
            }
            _ => {}
        }
    }

    pub fn show(
        &mut self,
        f: &mut Frame<'_>,
        bounds: Rect,
        id_base: u64,
        sys: &mut SystemState,
        pointer: Option<[f32; 2]>,
        ctx: &SettingsHostContext<'_>,
    ) -> SettingsAction {
        let p = sys.palette();
        let now = f.time();
        let scale = sys.anim_scale();
        let reduced = f.reduced_motion() || scale <= 0.01;
        f.paint_rect(bounds, 0.0, p.settings_bg);

        let zone_page = matches!(self.page, SettingsPage::DateTime | SettingsPage::TimeZoneRegion(_));
        self.time_zones.poll();
        if zone_page && !self.zone_page_active {
            self.time_zones.request(None);
        }
        self.zone_page_active = zone_page;
        if zone_page && self.time_zones.busy() {
            f.wake_at(now + 0.05);
        }

        let before_fingerprint = settings_fingerprint(sys);

        let mut active_trans: Option<(SettingsTransition, f32)> = None;
        if let Some(ref tr) = self.transition {
            let base_dur = if tr.is_search {
                if tr.is_push { 0.28 } else { 0.22 }
            } else if tr.is_push {
                0.30
            } else {
                0.26
            };
            let dur = base_dur * scale as f64;
            let raw = if reduced || dur <= 0.001 {
                1.0
            } else {
                ((now - tr.start) / dur).clamp(0.0, 1.0) as f32
            };
            if raw >= 1.0 {
                self.transition = None;
            } else {
                f.continuous(bounds);
                active_trans = Some((tr.clone(), raw));
            }
        }

        let mut action = SettingsAction::None;
        let dialog_open = self.dialog != SettingsDialog::None;
        let cur_page = self.page.clone();

        f.within(bounds, |f| {
            if let Some((tr, raw)) = active_trans {
                let e = emphasized_decelerate(raw);
                if tr.is_search {
                    let home_collapse = (self.scroll / 80.0).clamp(0.0, 1.0);
                    let home_pill_y = (152.0 - 44.0 * home_collapse) - 60.0;
                    let search_pill_y = 10.0;
                    if tr.is_push {
                        let home_a = (1.0 - smoothstep(0.0, 0.50, raw)).clamp(0.0, 1.0);
                        if home_a > 0.01 {
                            f.without_input(true, |f| {
                                f.push_layer(bounds, home_a);
                                let _ = self.render_page(
                                    f,
                                    &tr.from_page,
                                    tr.from_scroll,
                                    bounds,
                                    id_base + 5000,
                                    sys,
                                    &p,
                                    pointer,
                                    ctx,
                                    now,
                                );
                                f.pop_layer();
                            });
                        }
                        let search_a = smoothstep(0.06, 0.72, raw);
                        let dy = lerp(home_pill_y - search_pill_y, 0.0, e);
                        let sc = lerp(0.96, 1.0, e);
                        f.push_transform(
                            [bounds.x + bounds.width * 0.5, bounds.y + 36.0],
                            [0.0, dy * 0.55],
                            [sc, 1.0],
                            0.0,
                        );
                        f.push_layer(bounds, search_a);
                        action = self.render_page(
                            f,
                            &cur_page,
                            self.current_scroll_value(&cur_page),
                            bounds,
                            id_base,
                            sys,
                            &p,
                            pointer,
                            ctx,
                            now,
                        );
                        f.pop_layer();
                        f.pop_transform();
                    } else {
                        let home_a = smoothstep(0.18, 0.90, raw);
                        if home_a > 0.01 {
                            f.push_layer(bounds, home_a);
                            action = self.render_page(
                                f,
                                &cur_page,
                                self.current_scroll_value(&cur_page),
                                bounds,
                                id_base,
                                sys,
                                &p,
                                pointer,
                                ctx,
                                now,
                            );
                            f.pop_layer();
                        }
                        let search_a = (1.0 - smoothstep(0.0, 0.65, raw)).clamp(0.0, 1.0);
                        if search_a > 0.01 {
                            let dy = lerp(0.0, (home_pill_y - search_pill_y) * 0.55, e);
                            f.without_input(true, |f| {
                                f.push_transform([bounds.x, bounds.y], [0.0, dy], [1.0, 1.0], 0.0);
                                f.push_layer(bounds, search_a);
                                let _ = self.render_page(
                                    f,
                                    &tr.from_page,
                                    tr.from_scroll,
                                    bounds,
                                    id_base + 5000,
                                    sys,
                                    &p,
                                    pointer,
                                    ctx,
                                    now,
                                );
                                f.pop_layer();
                                f.pop_transform();
                            });
                        }
                    }
                } else {
                    let gutter_w = 12.0;
                    let gutter_col = if p.dark {
                        Color(0x0B, 0x0C, 0x10, 255)
                    } else {
                        Color(0xCC, 0xCE, 0xD8, 255)
                    };
                    if tr.is_push {
                        let out_dx = -bounds.width * 0.32 * e;
                        f.without_input(true, |f| {
                            f.push_transform([bounds.x, bounds.y], [out_dx, 0.0], [1.0, 1.0], 0.0);
                            f.paint_rect(bounds, 0.0, p.settings_bg);
                            let _ = self.render_page(
                                f,
                                &tr.from_page,
                                tr.from_scroll,
                                bounds,
                                id_base + 5000,
                                sys,
                                &p,
                                pointer,
                                ctx,
                                now,
                            );
                            f.paint_rect(bounds, 0.0, Color(0, 0, 0, (e * 34.0) as u8));
                            f.pop_transform();
                        });

                        let in_dx = (bounds.width + gutter_w) * (1.0 - e);
                        f.paint_rect(
                            Rect {
                                x: bounds.x + in_dx - gutter_w,
                                y: bounds.y,
                                width: gutter_w,
                                height: bounds.height,
                            },
                            0.0,
                            gutter_col,
                        );
                        f.push_transform([bounds.x, bounds.y], [in_dx, 0.0], [1.0, 1.0], 0.0);
                        f.paint_rect(bounds, 0.0, p.settings_bg);
                        let in_a = smoothstep(0.0, 0.24, raw);
                        if in_a < 0.999 {
                            f.push_layer(bounds, in_a);
                        }
                        action = self.render_page(
                            f,
                            &cur_page,
                            self.current_scroll_value(&cur_page),
                            bounds,
                            id_base,
                            sys,
                            &p,
                            pointer,
                            ctx,
                            now,
                        );
                        if in_a < 0.999 {
                            f.pop_layer();
                        }
                        f.pop_transform();
                    } else {
                        let in_dx = -bounds.width * 0.32 * (1.0 - e);
                        f.push_transform([bounds.x, bounds.y], [in_dx, 0.0], [1.0, 1.0], 0.0);
                        f.paint_rect(bounds, 0.0, p.settings_bg);
                        action = self.render_page(
                            f,
                            &cur_page,
                            self.current_scroll_value(&cur_page),
                            bounds,
                            id_base,
                            sys,
                            &p,
                            pointer,
                            ctx,
                            now,
                        );
                        f.paint_rect(bounds, 0.0, Color(0, 0, 0, ((1.0 - e) * 34.0) as u8));
                        f.pop_transform();

                        let out_dx = (bounds.width + gutter_w) * e;
                        f.paint_rect(
                            Rect {
                                x: bounds.x + out_dx - gutter_w,
                                y: bounds.y,
                                width: gutter_w,
                                height: bounds.height,
                            },
                            0.0,
                            gutter_col,
                        );
                        f.without_input(true, |f| {
                            f.push_transform([bounds.x, bounds.y], [out_dx, 0.0], [1.0, 1.0], 0.0);
                            f.paint_rect(bounds, 0.0, p.settings_bg);
                            let _ = self.render_page(
                                f,
                                &tr.from_page,
                                tr.from_scroll,
                                bounds,
                                id_base + 5000,
                                sys,
                                &p,
                                pointer,
                                ctx,
                                now,
                            );
                            f.pop_transform();
                        });
                    }
                }
            } else {
                let (st_s, st_sign) = self.edge_stretch;
                let whole_s = if st_sign > 0.0 { st_s } else { 1.0 };
                let whole_sign = if st_sign > 0.0 { st_sign } else { 0.0 };
                f.without_input(dialog_open, |f| {
                    f.with_edge_stretch(bounds, false, whole_s, whole_sign, |f| {
                        action = self.render_page(
                            f,
                            &cur_page,
                            self.current_scroll_value(&cur_page),
                            bounds,
                            id_base,
                            sys,
                            &p,
                            pointer,
                            ctx,
                            now,
                        );
                    });
                });
            }

            if dialog_open {
                let d_act = self.render_dialog(f, bounds, id_base + 900, sys, &p, now);
                if !matches!(d_act, SettingsAction::None) {
                    action = d_act;
                }
            }
        });

        if settings_fingerprint(sys) != before_fingerprint {
            sys.save_persisted_settings();
        }
        action
    }

    #[allow(clippy::too_many_arguments)]
    fn render_page(
        &mut self,
        f: &mut Frame<'_>,
        page: &SettingsPage,
        scroll_override: f32,
        bounds: Rect,
        id_base: u64,
        sys: &mut SystemState,
        p: &Palette,
        pointer: Option<[f32; 2]>,
        ctx: &SettingsHostContext<'_>,
        now: f64,
    ) -> SettingsAction {
        let is_primary = *page == self.page && id_base < 0x4C04_0000_0000_1000;
        match page {
            SettingsPage::Home => self.render_home(
                f,
                bounds,
                id_base,
                sys,
                p,
                ctx,
                now,
                if is_primary { None } else { Some(scroll_override) },
            ),
            SettingsPage::Search => self.render_search_page(f, bounds, id_base, p, now),
            _ => self.render_subpage(
                f,
                page,
                bounds,
                id_base,
                sys,
                p,
                pointer,
                ctx,
                now,
                if is_primary { None } else { Some(scroll_override) },
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_home(
        &mut self,
        f: &mut Frame<'_>,
        bounds: Rect,
        id_base: u64,
        sys: &SystemState,
        p: &Palette,
        ctx: &SettingsHostContext<'_>,
        now: f64,
        scroll_override: Option<f32>,
    ) -> SettingsAction {
        let title_travel = 88.0;
        let row_h = 74.0;
        let top_list_y = title_travel + 72.0;

        let net_sub = if sys.airplane_mode {
            "Airplane mode on".to_owned()
        } else if sys.wifi_enabled {
            format!("Mobile, {}, hotspot", sys.wifi_name)
        } else {
            "Wi-Fi off, hotspot".to_owned()
        };
        let bt_sub = if sys.bluetooth_enabled {
            "Bluetooth on · Connected".to_owned()
        } else {
            "Bluetooth, pairing".to_owned()
        };
        let disp_sub = format!(
            "{} · {}%",
            if sys.dark_theme { "Dark theme" } else { "Light theme" },
            (sys.brightness * 100.0).round() as u32
        );
        let notif_sub = format!(
            "{} active · {}",
            ctx.active_notif_count,
            sys.lock_screen_notifs.short_label()
        );
        let bat_sub = format!(
            "{}%{}",
            sys.battery_level,
            if sys.charging { " · Charging" } else { "" }
        );
        let free_gb = (ctx.device.storage_total_gb - ctx.device.storage_used_gb).max(0.0);
        let used_pct = if ctx.device.storage_total_gb > 0.1 {
            ((ctx.device.storage_used_gb / ctx.device.storage_total_gb) * 100.0).round() as u32
        } else {
            0
        };
        let storage_sub = format!("{used_pct}% used · {free_gb:.1} GB free");
        let sys_sub = match sys.nav_mode {
            NavMode::Gestural => "Languages, gestures, Buttons, Status bar",
            NavMode::ThreeButton => "3-button nav, Buttons, Status bar",
        };
        let sec_sub = format!("Screen lock: {}", sys.lock_method.label());
        let shiraz_sub = if sys.shiraz_enabled {
            match sys.shiraz.state {
                ShirazRuntimeState::Ready => {
                    if sys.shiraz.warm_ram_mb > 0 {
                        format!(
                            "On · Warm ({} MB) · {} apps",
                            sys.shiraz.warm_ram_mb,
                            sys.shiraz.installed_apps.len()
                        )
                    } else {
                        format!("On · Ready · {} apps", sys.shiraz.installed_apps.len())
                    }
                }
                ShirazRuntimeState::Warming => "On · Optimizing memory & AOT…".to_owned(),
                ShirazRuntimeState::Starting => "On · Starting runtime…".to_owned(),
                ShirazRuntimeState::Error => sys
                    .shiraz
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "Error starting runtime".to_owned()),
                ShirazRuntimeState::Disabled => "On · Initializing…".to_owned(),
            }
        } else if !sys.shiraz.installed_apps.is_empty() {
            format!("Off · {} Android apps installed", sys.shiraz.installed_apps.len())
        } else {
            "Off · Enable Android app compatibility".to_owned()
        };

        // Authentic Android 13 / Material You category tint accents for circular icon badges
        let categories: Vec<(SettingsPage, SystemIcon, Color, &str, String)> = vec![
            (SettingsPage::Network, SystemIcon::Wifi, Color(0x2B, 0x6C, 0xB0, 255), "Network & internet", net_sub),
            (SettingsPage::Connected, SystemIcon::Devices, Color(0x0D, 0x94, 0x88, 255), "Connected devices", bt_sub),
            (SettingsPage::Apps, SystemIcon::AppsGrid, Color(0xD9, 0x77, 0x06, 255), "Apps", format!("{} installed · Recent & default apps", ctx.apps.len())),
            (SettingsPage::Shiraz, SystemIcon::System, Color(0x10, 0xB9, 0x81, 255), "Shiraz (Android apps)", shiraz_sub),
            (SettingsPage::Notifications, SystemIcon::NotificationsActive, Color(0xDC, 0x26, 0x26, 255), "Notifications", notif_sub),
            (SettingsPage::Battery, SystemIcon::Battery, Color(0x16, 0xA3, 0x4A, 255), "Battery", bat_sub),
            (SettingsPage::Storage, SystemIcon::Storage, Color(0x7C, 0x3A, 0xED, 255), "Storage", storage_sub),
            (SettingsPage::Sound, SystemIcon::Sound, Color(0x02, 0x84, 0xC7, 255), "Sound & vibration", "Volume, vibration, Do Not Disturb".into()),
            (SettingsPage::Display, SystemIcon::Display, Color(0xEA, 0x58, 0x0C, 255), "Display", disp_sub),
            (SettingsPage::Accessibility, SystemIcon::Accessibility, Color(0x4F, 0x46, 0xE5, 255), "Accessibility", "Display, interaction, audio".into()),
            (SettingsPage::Security, SystemIcon::Fingerprint, Color(0x05, 0x96, 0x69, 255), "Security", sec_sub),
            (SettingsPage::Privacy, SystemIcon::Privacy, Color(0x25, 0x63, 0xEB, 255), "Privacy", "Permissions, camera & mic access".into()),
            (SettingsPage::Location, SystemIcon::Location, Color(0x0D, 0x94, 0x88, 255), "Location", if sys.location { "On · App access enabled" } else { "Off" }.into()),
            (SettingsPage::Safety, SystemIcon::Emergency, Color(0xE1, 0x1D, 0x48, 255), "Safety & emergency", "Emergency SOS, medical info".into()),
            (SettingsPage::Accounts, SystemIcon::Accounts, Color(0x63, 0x66, 0xF1, 255), "Passwords & accounts", "spectre@spectreos.org".into()),
            (SettingsPage::System, SystemIcon::Tune, Color(0x47, 0x55, 0x69, 255), "System", sys_sub.into()),
            (SettingsPage::About, SystemIcon::PhoneInfo, Color(0x43, 0x5E, 0x91, 255), "About phone", format!("{} · {}", ctx.device.device_name, ctx.device.os_version)),
        ];

        let max_s = (top_list_y + categories.len() as f32 * row_h + 24.0 - bounds.height).max(0.0);
        if scroll_override.is_none() {
            self.last_max_scroll = max_s;
            self.scroll = self.scroll.clamp(0.0, max_s);
            f.scroll_viewport(id_base + 5, bounds, &mut self.scroll, max_s);
        }
        let scroll_y = scroll_override.unwrap_or(self.scroll);
        let collapse = (scroll_y / title_travel).clamp(0.0, 1.0);
        let search_y = bounds.y + lerp(title_travel + 10.0, 10.0, collapse);
        let pinned_clip_top = search_y + 62.0;
        let list_clip = Rect {
            x: bounds.x,
            y: pinned_clip_top,
            width: bounds.width,
            height: (bounds.y + bounds.height - pinned_clip_top).max(60.0),
        };

        let mut next_page: Option<SettingsPage> = None;
        let (bot_s, bot_sign) = if self.edge_stretch.1 < 0.0 {
            self.edge_stretch
        } else {
            (1.0, 0.0)
        };
        f.within(list_clip, |f| {
            f.with_edge_stretch(list_clip, false, bot_s, bot_sign, |f| {
                for (i, (page, icon, tint, title, sub)) in categories.iter().enumerate() {
                    let ry = bounds.y + top_list_y + i as f32 * row_h - scroll_y;
                    if ry + row_h < list_clip.y - 10.0
                        || ry > list_clip.y + list_clip.height + 10.0
                    {
                        continue;
                    }
                    let row = Rect {
                        x: bounds.x,
                        y: ry,
                        width: bounds.width,
                        height: row_h,
                    };
                    let r = f.target(id_base + 20 + i as u64, row, title, true, Role::ListItem);
                    f.state_layer(r, 0.0, p.on_surface);

                    // High-res 96x96 Material Symbols icon inside authentic Android 13 circular badge
                    let badge = Rect {
                        x: row.x + 18.0,
                        y: row.y + (row_h - 40.0) * 0.5,
                        width: 40.0,
                        height: 40.0,
                    };
                    let badge_bg = if p.dark {
                        Color(tint.0, tint.1, tint.2, 54)
                    } else {
                        Color(tint.0, tint.1, tint.2, 30)
                    };
                    let icon_fg = if p.dark {
                        Color(
                            tint.0.saturating_add(72),
                            tint.1.saturating_add(72),
                            tint.2.saturating_add(72),
                            255,
                        )
                    } else {
                        *tint
                    };
                    f.paint_rect(badge, 20.0, badge_bg);
                    f.paint_system_icon(badge.inset(9.0), *icon, icon_fg);

                    f.styled_text(
                        Rect {
                            x: row.x + 72.0,
                            y: row.y + 15.0,
                            width: row.width - 88.0,
                            height: 22.0,
                        },
                        title,
                        p.on_surface,
                        text_style(16.5, 500, 0),
                    );
                    f.styled_text(
                        Rect {
                            x: row.x + 72.0,
                            y: row.y + 40.0,
                            width: row.width - 88.0,
                            height: 18.0,
                        },
                        sub,
                        p.on_surface_variant,
                        text_style(13.0, 400, 0),
                    );
                    if r.clicked {
                        next_page = Some(page.clone());
                    }
                }
            });
        });

        f.paint_rect(
            Rect {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: (pinned_clip_top - bounds.y).max(0.0),
            },
            0.0,
            p.settings_bg,
        );
        let title_alpha = 1.0 - smoothstep(0.15, 0.78, collapse);
        if title_alpha > 0.01 {
            f.within(bounds, |f| {
                f.styled_text(
                    Rect {
                        x: bounds.x + 22.0,
                        y: bounds.y + 36.0 - scroll_y,
                        width: bounds.width - 44.0,
                        height: 46.0,
                    },
                    "Settings",
                    p.on_surface.opacity(title_alpha),
                    text_style(34.0, 400, 0),
                );
            });
        }

        let search_r = Rect {
            x: bounds.x + 16.0,
            y: search_y,
            width: bounds.width - 32.0,
            height: 52.0,
        };
        let r_search = f.target(id_base + 1, search_r, "Search settings", true, Role::Button);
        f.paint_rect(search_r, 26.0, p.card_pure);
        f.state_layer(r_search, 26.0, p.on_surface);
        f.paint_system_icon(
            Rect {
                x: search_r.x + 16.0,
                y: search_r.y + 15.0,
                width: 22.0,
                height: 22.0,
            },
            SystemIcon::Search,
            p.on_surface,
        );
        f.styled_text(
            Rect {
                x: search_r.x + 50.0,
                y: search_r.y + 4.0,
                width: search_r.width - 66.0,
                height: 44.0,
            },
            "Search settings",
            p.on_surface_variant,
            text_style(16.0, 400, 0),
        );

        if r_search.clicked {
            self.open_search(now);
        } else if let Some(pg) = next_page {
            self.navigate(pg, now);
        }
        SettingsAction::None
    }

    fn render_search_page(
        &mut self,
        f: &mut Frame<'_>,
        bounds: Rect,
        id_base: u64,
        p: &Palette,
        now: f64,
    ) -> SettingsAction {
        let mut action = SettingsAction::None;

        let pill = Rect {
            x: bounds.x + 14.0,
            y: bounds.y + 10.0,
            width: bounds.width - 28.0,
            height: 52.0,
        };
        f.shadow(pill, 26.0, [0.0, 2.0], 8.0, 0.0, Color(0, 0, 0, 18));
        f.paint_rect(pill, 26.0, p.card_pure);

        let back_r = Rect {
            x: pill.x + 4.0,
            y: pill.y + 4.0,
            width: 44.0,
            height: 44.0,
        };
        let rb = f.target(id_base + 700, back_r, "Close search", true, Role::Button);
        f.state_layer(rb, 22.0, p.on_surface);
        f.paint_system_icon(back_r.inset(11.0), SystemIcon::Back, p.on_surface);

        let more_r = Rect {
            x: pill.x + pill.width - 46.0,
            y: pill.y + 6.0,
            width: 40.0,
            height: 40.0,
        };
        let rm = f.target(id_base + 701, more_r, "Search options", true, Role::Button);
        f.state_layer(rm, 20.0, p.on_surface);
        f.paint_system_icon(more_r.inset(10.0), SystemIcon::MoreVert, p.on_surface);

        let has_query = !self.search.is_empty();
        let clear_w = if has_query { 40.0 } else { 0.0 };
        if has_query {
            let clear_r = Rect {
                x: more_r.x - 40.0,
                y: pill.y + 6.0,
                width: 40.0,
                height: 40.0,
            };
            let rc = f.target(id_base + 702, clear_r, "Clear search", true, Role::Button);
            f.state_layer(rc, 20.0, p.on_surface);
            f.paint_system_icon(clear_r.inset(11.0), SystemIcon::Close, p.on_surface);
            if rc.clicked {
                self.search.clear();
                self.search_scroll = 0.0;
                self.focus_search_pending = true;
            }
        }

        let field = Rect {
            x: pill.x + 52.0,
            y: pill.y + 6.0,
            width: (pill.width - 102.0 - clear_w).max(60.0),
            height: 40.0,
        };
        let rf = f.target(ID_SETTINGS_SEARCH, field, "Search settings", true, Role::TextField);
        if self.focus_search_pending {
            self.focus_search_pending = false;
            f.request_focus(ID_SETTINGS_SEARCH);
        }
        let field_focused = rf.focused;
        if field_focused
            && let Some(k) = f.input().key
            && k.pressed
        {
            match k.key {
                bhoo::Key::Character('\u{8}') | bhoo::Key::Character('\u{7f}') => {
                    self.search.pop();
                    self.search_scroll = 0.0;
                }
                bhoo::Key::Space => {
                    self.search.push(' ');
                    self.search_scroll = 0.0;
                }
                bhoo::Key::Character(ch) if !ch.is_control() => {
                    self.search.push(ch);
                    self.search_scroll = 0.0;
                }
                _ => {}
            }
        }
        f.styled_text(
            field,
            if self.search.is_empty() {
                "Search settings"
            } else {
                &self.search
            },
            if self.search.is_empty() {
                p.on_surface_variant
            } else {
                p.on_surface
            },
            text_style(16.0, 400, 0),
        );
        if field_focused && ((now * 2.0) as u64).is_multiple_of(2) {
            let caret_x = (field.x + self.search.len() as f32 * 8.4 + 2.0)
                .min(field.x + field.width - 2.0);
            f.paint_rect(
                Rect {
                    x: caret_x,
                    y: field.y + 9.0,
                    width: 2.0,
                    height: 22.0,
                },
                1.0,
                p.primary,
            );
        }

        if rb.clicked {
            self.go_back(now);
            return SettingsAction::None;
        }
        if rm.clicked {
            self.search_menu_open = !self.search_menu_open;
        }

        let list_area = Rect {
            x: bounds.x,
            y: pill.y + pill.height + 10.0,
            width: bounds.width,
            height: (bounds.y + bounds.height - (pill.y + pill.height + 10.0)).max(80.0),
        };
        let q = self.search.trim().to_ascii_lowercase();
        let mut pick_recent: Option<String> = None;
        let mut jump_target: Option<SettingsPage> = None;

        if q.is_empty() {
            let row_h = 56.0;
            let total_h = self.recent_searches.len() as f32 * row_h + 24.0;
            let max_s = (total_h - list_area.height).max(0.0);
            if id_base < 0x4C04_0000_0000_1000 {
                self.last_max_scroll = max_s;
                self.search_scroll = self.search_scroll.clamp(0.0, max_s);
                f.scroll_viewport(id_base + 705, list_area, &mut self.search_scroll, max_s);
            }
            f.within(list_area, |f| {
                if self.recent_searches.is_empty() {
                    f.styled_text(
                        Rect {
                            x: list_area.x,
                            y: list_area.y + 44.0,
                            width: list_area.width,
                            height: 24.0,
                        },
                        "Type to search all SpectreOS settings",
                        p.on_surface_variant,
                        text_style(14.0, 400, 1),
                    );
                } else {
                    for (i, term) in self.recent_searches.iter().enumerate() {
                        let ry = list_area.y + 6.0 + i as f32 * row_h - self.search_scroll;
                        let row = Rect {
                            x: list_area.x,
                            y: ry,
                            width: list_area.width,
                            height: row_h,
                        };
                        let r = f.target(id_base + 710 + i as u64, row, term, true, Role::ListItem);
                        f.state_layer(r, 8.0, p.on_surface);
                        f.paint_system_icon(
                            Rect {
                                x: row.x + 24.0,
                                y: row.y + 17.0,
                                width: 22.0,
                                height: 22.0,
                            },
                            SystemIcon::History,
                            p.on_surface_variant,
                        );
                        f.styled_text(
                            Rect {
                                x: row.x + 68.0,
                                y: row.y,
                                width: row.width - 84.0,
                                height: row_h,
                            },
                            term,
                            p.on_surface,
                            text_style(15.5, 400, 0),
                        );
                        if r.clicked {
                            pick_recent = Some(term.clone());
                        }
                    }
                }
            });
        } else {
            let matches: Vec<&SearchIndexEntry> = SEARCH_INDEX
                .iter()
                .filter(|e| {
                    e.title.to_ascii_lowercase().contains(&q)
                        || e.summary
                            .is_some_and(|s| s.to_ascii_lowercase().contains(&q))
                        || e.breadcrumb.to_ascii_lowercase().contains(&q)
                        || e.keywords.contains(&q)
                })
                .collect();

            let mut total_h = 12.0_f32;
            for e in &matches {
                total_h += if e.summary.is_some() { 94.0 } else { 76.0 };
            }
            let max_s = (total_h + 24.0 - list_area.height).max(0.0);
            if id_base < 0x4C04_0000_0000_1000 {
                self.last_max_scroll = max_s;
                self.search_scroll = self.search_scroll.clamp(0.0, max_s);
                f.scroll_viewport(id_base + 706, list_area, &mut self.search_scroll, max_s);
            }

            f.within(list_area, |f| {
                if matches.is_empty() {
                    f.styled_text(
                        Rect {
                            x: list_area.x,
                            y: list_area.y + 56.0,
                            width: list_area.width,
                            height: 26.0,
                        },
                        "No results found",
                        p.on_surface_variant,
                        text_style(15.5, 500, 1),
                    );
                } else {
                    let mut y = list_area.y + 8.0 - self.search_scroll;
                    for (i, e) in matches.iter().enumerate() {
                        let rh = if e.summary.is_some() { 94.0 } else { 76.0 };
                        if y + rh >= list_area.y - 10.0 && y <= list_area.y + list_area.height + 10.0 {
                            let row = Rect {
                                x: list_area.x,
                                y,
                                width: list_area.width,
                                height: rh,
                            };
                            let r = f.target(id_base + 750 + i as u64, row, e.title, true, Role::ListItem);
                            f.state_layer(r, 8.0, p.on_surface);
                            f.styled_text(
                                Rect {
                                    x: row.x + 24.0,
                                    y: row.y + 12.0,
                                    width: row.width - 44.0,
                                    height: 22.0,
                                },
                                e.title,
                                p.on_surface,
                                text_style(16.0, 500, 0),
                            );
                            let mut sub_y = row.y + 36.0;
                            if let Some(sum) = e.summary {
                                f.styled_text(
                                    Rect {
                                        x: row.x + 24.0,
                                        y: sub_y,
                                        width: row.width - 44.0,
                                        height: 18.0,
                                    },
                                    sum,
                                    p.on_surface,
                                    text_style(13.5, 400, 0),
                                );
                                sub_y += 22.0;
                            }
                            f.styled_text(
                                Rect {
                                    x: row.x + 24.0,
                                    y: sub_y,
                                    width: row.width - 44.0,
                                    height: 18.0,
                                },
                                e.breadcrumb,
                                p.on_surface_variant,
                                text_style(12.5, 400, 0),
                            );
                            if r.clicked {
                                jump_target = Some((e.target)());
                            }
                        }
                        y += rh;
                    }
                }
            });
        }

        if self.search_menu_open {
            let menu = Rect {
                x: bounds.x + bounds.width - 196.0,
                y: pill.y + 46.0,
                width: 182.0,
                height: 48.0,
            };
            f.elevation(menu, 12.0, 3);
            f.paint_rect(menu, 12.0, p.card_pure);
            let r_clr = f.target(id_base + 790, menu, "Clear search history", true, Role::MenuItem);
            f.state_layer(r_clr, 12.0, p.on_surface);
            f.styled_text(
                Rect {
                    x: menu.x + 16.0,
                    y: menu.y,
                    width: menu.width - 24.0,
                    height: menu.height,
                },
                "Clear search history",
                p.on_surface,
                text_style(14.0, 400, 0),
            );
            if r_clr.clicked {
                self.recent_searches.clear();
                self.search_menu_open = false;
                action = SettingsAction::ShowToast("Search history cleared".into());
            }
        }

        if let Some(term) = pick_recent {
            self.search = term;
            self.search_scroll = 0.0;
        } else if let Some(target) = jump_target {
            let q_save = self.search.clone();
            self.record_recent_search(&q_save);
            self.navigate(target, now);
        }

        action
    }

    #[allow(clippy::too_many_arguments)]
    fn render_subpage(
        &mut self,
        f: &mut Frame<'_>,
        page: &SettingsPage,
        bounds: Rect,
        id_base: u64,
        sys: &mut SystemState,
        p: &Palette,
        pointer: Option<[f32; 2]>,
        ctx: &SettingsHostContext<'_>,
        now: f64,
        scroll_override: Option<f32>,
    ) -> SettingsAction {
        let mut action = SettingsAction::None;

        let title = match page {
            SettingsPage::Home => "Settings",
            SettingsPage::Search => "Search",
            SettingsPage::Network => "Network & internet",
            SettingsPage::Internet => "Internet",
            SettingsPage::Connected => "Connected devices",
            SettingsPage::Apps => "Apps",
            SettingsPage::Shiraz => "Shiraz",
            SettingsPage::AppInfo(_) => "App info",
            SettingsPage::Notifications => "Notifications",
            SettingsPage::NotificationHistory => "Notification history",
            SettingsPage::Battery => "Battery",
            SettingsPage::Storage => "Storage",
            SettingsPage::Sound => "Sound & vibration",
            SettingsPage::Display => "Display",
            SettingsPage::Accessibility => "Accessibility",
            SettingsPage::Security => "Security",
            SettingsPage::ScreenLock => "Screen lock",
            SettingsPage::Privacy => "Privacy",
            SettingsPage::Location => "Location",
            SettingsPage::Safety => "Safety & emergency",
            SettingsPage::Accounts => "Passwords & accounts",
            SettingsPage::System => "System",
            SettingsPage::DateTime => "Date & time",
            SettingsPage::TimeZoneRegion(region) => region.as_str(),
            SettingsPage::Gestures => "Gestures",
            SettingsPage::Navigation => "System navigation",
            SettingsPage::LineageButtons => "Buttons",
            SettingsPage::LineageStatusBar => "Status bar",
            SettingsPage::DevOptions => "Developer options",
            SettingsPage::About => "About phone",
        };

        let expanded_h = 136.0_f32;
        let collapsed_h = 56.0_f32;
        let collapse_range = expanded_h - collapsed_h;
        let content_h = match page {
            SettingsPage::Apps => ctx.apps.len() as f32 * 64.0 + 72.0,
            SettingsPage::Shiraz => {
                860.0 + (sys.shiraz.installed_apps.len().max(1) as f32) * 66.0
            }
            SettingsPage::NotificationHistory => (ctx.history.len().max(2) as f32) * 76.0 + 160.0,
            SettingsPage::DateTime => 260.0 + self.time_zones.regions.len() as f32 * 60.0,
            SettingsPage::TimeZoneRegion(region) => 94.0 + self.time_zones.zones.iter().filter(|z| time_zone::region(z) == region).count() as f32 * 60.0,
            SettingsPage::Notifications => 790.0,
            SettingsPage::About => 13.0 * 62.0 + 168.0,
            SettingsPage::Network | SettingsPage::Display | SettingsPage::Sound => 720.0,
            SettingsPage::Internet => 250.0 + sys.wifi_networks.len() as f32 * 59.0,
            SettingsPage::Connected => 280.0 + sys.bluetooth_devices.len() as f32 * 59.0,
            SettingsPage::System | SettingsPage::DevOptions => 640.0,
            _ => 580.0,
        };
        let max_sub = (expanded_h + content_h - bounds.height).max(0.0);
        if scroll_override.is_none() {
            self.last_max_scroll = max_sub;
            self.sub_scroll = self.sub_scroll.clamp(0.0, max_sub);
            f.scroll_viewport(id_base + 101, bounds, &mut self.sub_scroll, max_sub);
        }
        let scroll_y = scroll_override.unwrap_or(self.sub_scroll);
        let collapse = (scroll_y / collapse_range).clamp(0.0, 1.0);
        let bar_h = lerp(expanded_h, collapsed_h, collapse);
        let body = Rect {
            x: bounds.x,
            y: bounds.y + bar_h,
            width: bounds.width,
            height: (bounds.height - bar_h).max(80.0),
        };

        let mut push_page: Option<SettingsPage> = None;
        let (bot_s, bot_sign) = if self.edge_stretch.1 < 0.0 {
            self.edge_stretch
        } else {
            (1.0, 0.0)
        };

        f.within(body, |f| {
            f.with_edge_stretch(body, false, bot_s, bot_sign, |f| {
                let mut y = bounds.y + expanded_h - scroll_y;
                let x = bounds.x + 16.0;
                let w = bounds.width - 32.0;

                match page {
                    SettingsPage::Network => {
                        let summary = if sys.wifi_connected { sys.wifi_name.as_str() }
                            else if sys.wifi_enabled { "Wi-Fi on" } else { "Wi-Fi off" };
                        if paint_pref_row(f, id_base + 200, Rect { x, y, width: w, height: 62.0 },
                            "Internet", summary, p)
                        {
                            push_page = Some(SettingsPage::Internet);
                        }
                        y += 64.0;
                        let mut airplane = sys.airplane_mode;
                        if paint_pref_switch(f, id_base + 220, Rect { x, y, width: w, height: 62.0 },
                            "Airplane mode", None, &mut airplane, p)
                        {
                            sys.set_airplane_mode(airplane);
                        }
                        y += 64.0;
                        paint_pref_switch(
                            f,
                            id_base + 221,
                            Rect { x, y, width: w, height: 62.0 },
                            "Hotspot & tethering",
                            Some(if sys.hotspot { "Active" } else { "Off" }),
                            &mut sys.hotspot,
                            p,
                        );
                        y += 64.0;
                        paint_pref_switch(
                            f,
                            id_base + 222,
                            Rect { x, y, width: w, height: 62.0 },
                            "Data Saver",
                            Some(if sys.data_saver { "On" } else { "Off" }),
                            &mut sys.data_saver,
                            p,
                        );
                        y += 64.0;
                        let dns_sub = match sys.dns_mode {
                            DnsMode::Off => "Off".to_owned(),
                            DnsMode::Cloudflare => "Cloudflare DNS".to_owned(),
                            DnsMode::Automatic => "Automatic".to_owned(),
                            DnsMode::Custom => sys.dns_host.clone(),
                        };
                        if paint_pref_row(f, id_base + 223, Rect { x, y, width: w, height: 62.0 }, "Private DNS", &dns_sub, p) {
                            self.dns_draft = sys.dns_host.clone();
                            self.dns_draft_mode = sys.dns_mode;
                            self.dialog = SettingsDialog::PrivateDns;
                            self.dialog_start = now;
                        }
                    }
                    SettingsPage::Internet => {
                        if sys.wifi_available {
                            paint_pref_switch(f, id_base + 4000, Rect { x, y, width: w, height: 62.0 },
                                "Wi-Fi", None, &mut sys.wifi_enabled, p);
                        } else {
                            f.styled_text(Rect { x, y, width: w, height: 50.0 }, "Wi-Fi unavailable",
                                p.on_surface_variant, text_style(15.0, 500, 0));
                        }
                        y += 64.0;
                        if let Some(error) = &sys.wifi_error {
                            f.styled_text(Rect { x, y, width: w, height: 36.0 }, error, p.on_surface_variant, text_style(13.0, 500, 0));
                            y += 38.0;
                        }
                        if sys.wifi_available && sys.wifi_enabled {
                            let message = if sys.wifi_networks.is_empty() { "No networks found" } else { "Available networks" };
                            f.styled_text(Rect { x, y, width: w, height: 34.0 }, message, p.primary, text_style(13.0, 600, 0));
                            y += 36.0;
                            for (i, network) in sys.wifi_networks.iter().enumerate() {
                                let state = if network.connected { "Connected" }
                                    else if network.profile.is_some() { "Saved" }
                                    else if network.secured { "Secured" } else { "Open network" };
                                let subtitle = format!("{state}  ·  {}%", network.signal);
                                if paint_pref_row(f, id_base + 4010 + i as u64, Rect { x, y, width: w, height: 58.0 }, &network.ssid, &subtitle, p) {
                                    if network.connected || (network.secured && network.profile.is_none()) {
                                        action = SettingsAction::AdvancedRadioSettings("wifi");
                                    } else {
                                        action = SettingsAction::JoinWifi(network.clone());
                                        sys.wifi_error = None;
                                    }
                                }
                                y += 59.0;
                            }
                            if paint_pref_row(f, id_base + 4002, Rect { x, y, width: w, height: 58.0 },
                                "Add network", "Join a secured or hidden network", p)
                            {
                                action = SettingsAction::AdvancedRadioSettings("wifi");
                            }
                        }
                    }
                    SettingsPage::Connected => {
                        if sys.bluetooth_available {
                            paint_pref_switch(f, id_base + 4100, Rect { x, y, width: w, height: 62.0 },
                                "Bluetooth", None, &mut sys.bluetooth_enabled, p);
                        } else {
                            f.styled_text(Rect { x, y, width: w, height: 50.0 }, "Bluetooth unavailable",
                                p.on_surface_variant, text_style(15.0, 500, 0));
                        }
                        y += 66.0;
                        if let Some(error) = &sys.bluetooth_error {
                            f.styled_text(Rect { x, y, width: w, height: 36.0 }, error, p.on_surface_variant, text_style(13.0, 500, 0));
                            y += 38.0;
                        }
                        if sys.bluetooth_available && sys.bluetooth_enabled {
                            if paint_pref_row(f, id_base + 4101, Rect { x, y, width: w, height: 58.0 },
                                "Pair new device", "Find nearby Bluetooth devices", p)
                            {
                                action = SettingsAction::AdvancedRadioSettings("bluetooth");
                            }
                            y += 66.0;
                            let label = if sys.bluetooth_devices.is_empty() { "No saved devices" } else { "Saved devices" };
                            f.styled_text(Rect { x, y, width: w, height: 34.0 }, label, p.primary, text_style(13.0, 600, 0));
                            y += 36.0;
                            for (i, device) in sys.bluetooth_devices.iter().enumerate() {
                                if paint_pref_row(f, id_base + 4110 + i as u64, Rect { x, y, width: w, height: 58.0 },
                                    &device.name, if device.connected { "Connected · Tap to disconnect" } else { "Saved · Tap to connect" }, p)
                                {
                                    action = SettingsAction::ConnectBluetooth(device.address.clone(), !device.connected);
                                    sys.bluetooth_error = None;
                                }
                                y += 59.0;
                            }
                        }
                    }
                    SettingsPage::Apps => {
                        for (i, app) in ctx.apps.iter().enumerate() {
                            let rr = Rect { x, y, width: w, height: 62.0 };
                            if rr.y + rr.height >= body.y - 20.0 && rr.y <= body.y + body.height + 20.0 {
                                let full_rr = Rect { x: bounds.x, y: rr.y, width: bounds.width, height: rr.height };
                                let r = f.target(id_base + 240 + i as u64, full_rr, &app.name, true, Role::ListItem);
                                f.state_layer(r, 0.0, p.on_surface);
                                let icon_r = Rect { x: rr.x + 8.0, y: rr.y + 10.0, width: 42.0, height: 42.0 };
                                if let Some(painter) = ctx.paint_app_icon {
                                    painter(f, icon_r, i, p);
                                } else {
                                    f.paint_rect(icon_r, 21.0, p.primary_container);
                                    f.paint_system_icon(icon_r.inset(9.0), SystemIcon::AppsGrid, p.on_primary_container);
                                }
                                f.styled_text(
                                    Rect { x: rr.x + 64.0, y: rr.y + 10.0, width: rr.width - 78.0, height: 22.0 },
                                    &app.name,
                                    p.on_surface,
                                    text_style(15.5, 500, 0),
                                );
                                f.styled_text(
                                    Rect { x: rr.x + 64.0, y: rr.y + 33.0, width: rr.width - 78.0, height: 18.0 },
                                    &if app.is_android {
                                        format!("Android (Shiraz) · v{}", app.version)
                                    } else {
                                        format!("{} MB · v{}", app.storage_mb, app.version)
                                    },
                                    p.on_surface_variant,
                                    text_style(12.5, 400, 0),
                                );
                                if r.clicked {
                                    push_page = Some(SettingsPage::AppInfo(i));
                                }
                            }
                            y += 64.0;
                        }
                    }
                    SettingsPage::Shiraz => {
                        // 1. Hero Runtime & Memory Telemetry Card
                        let hero = Rect { x, y, width: w, height: 132.0 };
                        f.paint_rect(hero, 24.0, p.card_elevated);
                        let badge = Rect { x: hero.x + 16.0, y: hero.y + 18.0, width: 46.0, height: 46.0 };
                        let (pill_bg, pill_fg, status_label) = if !sys.shiraz_enabled {
                            (p.surface_container, p.on_surface_variant, "Off")
                        } else {
                            match sys.shiraz.state {
                                ShirazRuntimeState::Ready => (
                                    Color(0x10, 0xB9, 0x81, if p.dark { 64 } else { 38 }),
                                    if p.dark { Color(0x6E, 0xE7, 0xB7, 255) } else { Color(0x04, 0x78, 0x57, 255) },
                                    "Ready · Warm",
                                ),
                                ShirazRuntimeState::Warming => (
                                    p.primary_container,
                                    p.on_primary_container,
                                    "Optimizing…",
                                ),
                                ShirazRuntimeState::Starting | ShirazRuntimeState::Disabled => (
                                    p.primary_container,
                                    p.on_primary_container,
                                    "Starting…",
                                ),
                                ShirazRuntimeState::Error => (
                                    Color(0xDC, 0x26, 0x26, if p.dark { 64 } else { 34 }),
                                    if p.dark { Color(0xFC, 0xA5, 0xA5, 255) } else { Color(0xB9, 0x1C, 0x1C, 255) },
                                    "Error",
                                ),
                            }
                        };
                        f.paint_rect(badge, 23.0, pill_bg);
                        f.paint_system_icon(badge.inset(10.0), SystemIcon::System, pill_fg);

                        let pill_w = 104.0;
                        let pill_r = Rect {
                            x: hero.x + hero.width - pill_w - 14.0,
                            y: hero.y + 18.0,
                            width: pill_w,
                            height: 26.0,
                        };
                        f.paint_rect(pill_r, 13.0, pill_bg);
                        f.styled_text(pill_r, status_label, pill_fg, text_style(11.5, 600, 1));

                        f.styled_text(
                            Rect {
                                x: hero.x + 74.0,
                                y: hero.y + 16.0,
                                width: hero.width - 74.0 - pill_w - 22.0,
                                height: 22.0,
                            },
                            "Shiraz Runtime",
                            p.on_surface,
                            text_style(17.0, 600, 0),
                        );
                        let sub_line_1 = if let Some(ref err) = sys.shiraz.error_message {
                            err.clone()
                        } else if sys.shiraz_enabled {
                            let aot_lbl = if sys.shiraz.aot_boot_ready && sys.shiraz.aot_framework_ready {
                                "AOT Ready"
                            } else if sys.shiraz_aot_enabled {
                                "AOT Preparing"
                            } else {
                                "JIT"
                            };
                            format!(
                                "Warm: {} MB ({} files) · {}",
                                sys.shiraz.warm_ram_mb, sys.shiraz.warm_files, aot_lbl
                            )
                        } else if !sys.shiraz.available {
                            "Runtime not detected (set SHIRAZ_HOME or install shiraz)".to_owned()
                        } else {
                            "Enable to pre-warm ART & framework in memory".to_owned()
                        };
                        f.styled_text(
                            Rect { x: hero.x + 74.0, y: hero.y + 42.0, width: hero.width - 90.0, height: 18.0 },
                            &sub_line_1,
                            p.on_surface_variant,
                            text_style(12.5, 400, 0),
                        );
                        let launch_suffix = sys
                            .shiraz
                            .last_launch_ms
                            .map(|ms| format!(" · Last launch {ms} ms"))
                            .unwrap_or_default();
                        let sub_line_2 = format!(
                            "{} Android apps · {} running{}",
                            sys.shiraz.installed_apps.len(),
                            sys.shiraz.running_packages.len(),
                            launch_suffix
                        );
                        f.styled_text(
                            Rect { x: hero.x + 18.0, y: hero.y + 78.0, width: hero.width - 36.0, height: 18.0 },
                            &sub_line_2,
                            p.on_surface,
                            text_style(13.0, 500, 0),
                        );
                        f.styled_text(
                            Rect { x: hero.x + 18.0, y: hero.y + 100.0, width: hero.width - 36.0, height: 16.0 },
                            "Wayland native · Freedesktop Desktop, Icons & Notifications",
                            p.primary,
                            text_style(11.5, 500, 0),
                        );
                        y += 146.0;

                        // 2. Primary Master Switch Card ("Enable Shiraz")
                        let master_r = Rect { x, y, width: w, height: 72.0 };
                        let master_bg = if sys.shiraz_enabled {
                            p.primary_container
                        } else {
                            p.surface_container
                        };
                        let master_fg = if sys.shiraz_enabled {
                            p.on_primary_container
                        } else {
                            p.on_surface
                        };
                        f.paint_rect(master_r, 24.0, master_bg);
                        let rm = f.target(id_base + 500, master_r, "Enable Shiraz", true, Role::Switch);
                        f.state_layer(rm, 24.0, master_fg);
                        f.styled_text(
                            Rect { x: master_r.x + 20.0, y: master_r.y + 13.0, width: master_r.width - 96.0, height: 24.0 },
                            "Enable Shiraz",
                            master_fg,
                            text_style(17.5, 600, 0),
                        );
                        f.styled_text(
                            Rect { x: master_r.x + 20.0, y: master_r.y + 39.0, width: master_r.width - 96.0, height: 18.0 },
                            if sys.shiraz_enabled {
                                "Pre-warmed in memory for instant Android app launches"
                            } else {
                                "Pre-warm ART, framework & service broker in memory"
                            },
                            master_fg.opacity(0.82),
                            text_style(12.0, 400, 0),
                        );
                        f.without_input(true, |f| {
                            let mut sw = sys.shiraz_enabled;
                            paint_switch(
                                f,
                                id_base + 509,
                                Rect {
                                    x: master_r.x + master_r.width - 68.0,
                                    y: master_r.y + 20.0,
                                    width: 52.0,
                                    height: 32.0,
                                },
                                &mut sw,
                                true,
                                p,
                            );
                        });
                        if rm.clicked {
                            sys.shiraz_enabled = !sys.shiraz_enabled;
                            action = SettingsAction::ShowToast(if sys.shiraz_enabled {
                                "Enabling Shiraz & optimizing memory…".into()
                            } else {
                                "Disabling Shiraz & releasing warm memory".into()
                            });
                        }
                        y += 88.0;

                        // 3. Performance & Optimization Section
                        f.styled_text(
                            Rect { x: x + 6.0, y, width: w - 12.0, height: 22.0 },
                            "Performance & optimization",
                            p.primary,
                            text_style(13.5, 600, 0),
                        );
                        y += 28.0;
                        paint_pref_switch(
                            f,
                            id_base + 501,
                            Rect { x, y, width: w, height: 64.0 },
                            "Pre-warm runtime in memory",
                            Some("Keep ART, framework & libraries hot in page cache"),
                            &mut sys.shiraz_prewarm_memory,
                            p,
                        );
                        y += 66.0;
                        paint_pref_switch(
                            f,
                            id_base + 502,
                            Rect { x, y, width: w, height: 64.0 },
                            "AOT boot image & app optimization",
                            Some("Pre-compile DEX bytecode via dex2oat for fast launches"),
                            &mut sys.shiraz_aot_enabled,
                            p,
                        );
                        y += 66.0;
                        paint_pref_switch(
                            f,
                            id_base + 503,
                            Rect { x, y, width: w, height: 64.0 },
                            "Start automatically on login",
                            Some("XDG Autostart (org.spectreos.Shiraz.desktop)"),
                            &mut sys.shiraz_autostart,
                            p,
                        );
                        y += 66.0;
                        paint_pref_switch(
                            f,
                            id_base + 504,
                            Rect { x, y, width: w, height: 64.0 },
                            "Isolate apps with sandbox",
                            Some("Confine Android apps with bubblewrap when available"),
                            &mut sys.shiraz_sandbox,
                            p,
                        );
                        y += 74.0;

                        // Action buttons: Optimize & refresh / Stop all apps
                        let bw = (w - 12.0) * 0.5;
                        let b_opt = Rect { x, y, width: bw, height: 44.0 };
                        let ro = f.target(id_base + 505, b_opt, "Optimize & refresh", true, Role::Button);
                        f.paint_rect(b_opt, 22.0, p.primary);
                        f.state_layer(ro, 22.0, p.on_primary);
                        f.styled_text(b_opt, "Optimize now", p.on_primary, text_style(13.5, 600, 1));
                        if ro.clicked {
                            action = SettingsAction::ShirazRefresh;
                        }

                        let b_stop = Rect { x: x + bw + 12.0, y, width: bw, height: 44.0 };
                        let rs = f.target(id_base + 506, b_stop, "Stop all Android apps", true, Role::Button);
                        f.paint_rect(b_stop, 22.0, p.card_elevated);
                        f.state_layer(rs, 22.0, p.on_surface);
                        f.styled_text(b_stop, "Stop all apps", p.on_surface, text_style(13.5, 500, 1));
                        if rs.clicked {
                            action = SettingsAction::ShirazStopAll;
                        }
                        y += 62.0;

                        // 4. Installed Android Apps Section
                        f.styled_text(
                            Rect { x: x + 6.0, y, width: w - 12.0, height: 22.0 },
                            "Installed Android apps",
                            p.primary,
                            text_style(13.5, 600, 0),
                        );
                        y += 28.0;
                        if sys.shiraz.installed_apps.is_empty() {
                            let empty = Rect { x, y, width: w, height: 86.0 };
                            f.paint_rect(empty, 18.0, p.card_elevated);
                            f.styled_text(
                                Rect { x: empty.x + 16.0, y: empty.y + 14.0, width: empty.width - 32.0, height: 22.0 },
                                "No Android apps installed yet",
                                p.on_surface,
                                text_style(14.5, 500, 0),
                            );
                            f.styled_text(
                                Rect { x: empty.x + 16.0, y: empty.y + 40.0, width: empty.width - 32.0, height: 34.0 },
                                "Open any .apk file or run shiraz-install-apk <app.apk> to register it in the Launcher.",
                                p.on_surface_variant,
                                text_style(12.0, 400, 0),
                            );
                            y += 98.0;
                        } else {
                            let apps_snapshot = sys.shiraz.installed_apps.clone();
                            for (i, aapp) in apps_snapshot.iter().enumerate() {
                                let row = Rect { x, y, width: w, height: 62.0 };
                                f.paint_rect(row, 16.0, p.card_elevated);
                                let ic_r = Rect { x: row.x + 12.0, y: row.y + 11.0, width: 40.0, height: 40.0 };
                                f.paint_rect(ic_r, 20.0, p.primary_container);
                                f.paint_system_icon(ic_r.inset(9.0), SystemIcon::AppsGrid, p.on_primary_container);

                                let open_w = 62.0;
                                let stop_w = if aapp.running { 58.0 } else { 0.0 };
                                let btns_w = open_w + if aapp.running { stop_w + 6.0 } else { 0.0 } + 12.0;

                                f.styled_text(
                                    Rect { x: row.x + 62.0, y: row.y + 10.0, width: (row.width - 68.0 - btns_w).max(60.0), height: 20.0 },
                                    &aapp.label,
                                    p.on_surface,
                                    text_style(14.5, 500, 0),
                                );
                                let meta = format!(
                                    "{} · {}{}",
                                    aapp.package,
                                    if aapp.aot_compiled { "AOT" } else { "JIT" },
                                    if aapp.running { " · Running" } else { "" }
                                );
                                f.styled_text(
                                    Rect { x: row.x + 62.0, y: row.y + 33.0, width: (row.width - 68.0 - btns_w).max(60.0), height: 16.0 },
                                    &meta,
                                    p.on_surface_variant,
                                    text_style(11.5, 400, 0),
                                );

                                let open_r = Rect {
                                    x: row.x + row.width - open_w - 10.0,
                                    y: row.y + 14.0,
                                    width: open_w,
                                    height: 34.0,
                                };
                                let r_open = f.target(id_base + 520 + (i as u64) * 2, open_r, "Open Android app", true, Role::Button);
                                f.paint_rect(open_r, 17.0, p.primary_container);
                                f.state_layer(r_open, 17.0, p.on_primary_container);
                                f.styled_text(open_r, "Open", p.on_primary_container, text_style(12.5, 600, 1));
                                if r_open.clicked {
                                    if !sys.shiraz_enabled {
                                        sys.shiraz_enabled = true;
                                    }
                                    action = SettingsAction::ShirazLaunchPackage(aapp.package.clone());
                                }

                                if aapp.running {
                                    let stop_r = Rect {
                                        x: open_r.x - stop_w - 6.0,
                                        y: row.y + 14.0,
                                        width: stop_w,
                                        height: 34.0,
                                    };
                                    let r_stop = f.target(id_base + 521 + (i as u64) * 2, stop_r, "Stop Android app", true, Role::Button);
                                    f.paint_rect(stop_r, 17.0, p.surface_container);
                                    f.state_layer(r_stop, 17.0, p.on_surface);
                                    f.styled_text(stop_r, "Stop", p.on_surface, text_style(12.5, 500, 1));
                                    if r_stop.clicked {
                                        action = SettingsAction::ShirazStopPackage(aapp.package.clone());
                                    }
                                }
                                y += 68.0;
                            }
                        }
                        y += 10.0;

                        // 5. Ecosystem & Freedesktop Integration Summary
                        f.styled_text(
                            Rect { x: x + 6.0, y, width: w - 12.0, height: 22.0 },
                            "Ecosystem & Freedesktop",
                            p.primary,
                            text_style(13.5, 600, 0),
                        );
                        y += 28.0;
                        paint_pref_row(
                            f,
                            id_base + 580,
                            Rect { x, y, width: w, height: 58.0 },
                            "Wayland & Wisp shell",
                            "Direct xdg_toplevel · Spectre Keyboard · Edge Back · Multi-touch",
                            p,
                        );
                        y += 60.0;
                        paint_pref_row(
                            f,
                            id_base + 581,
                            Rect { x, y, width: w, height: 58.0 },
                            "Freedesktop standards",
                            "XDG BaseDir · Desktop Entry v1.5 · Icon Theme · Notifications v1.3",
                            p,
                        );
                        y += 60.0;
                        paint_pref_row(
                            f,
                            id_base + 582,
                            Rect { x, y, width: w, height: 58.0 },
                            "APK MIME association",
                            "application/vnd.android.package-archive → Shiraz Installer",
                            p,
                        );
                    }
                    SettingsPage::AppInfo(idx) => {
                        if let Some(app) = ctx.apps.get(*idx) {
                            let icon_r = Rect { x: body.x + body.width * 0.5 - 34.0, y, width: 68.0, height: 68.0 };
                            if let Some(painter) = ctx.paint_app_icon {
                                painter(f, icon_r, *idx, p);
                            } else {
                                f.paint_rect(icon_r, 34.0, p.primary_container);
                                f.paint_system_icon(icon_r.inset(14.0), SystemIcon::AppsGrid, p.on_primary_container);
                            }
                            y += 78.0;
                            f.styled_text(Rect { x, y, width: w, height: 28.0 }, &app.name, p.on_surface, text_style(22.0, 500, 1));
                            y += 28.0;
                            f.styled_text(Rect { x, y, width: w, height: 20.0 }, &format!("{} · v{}", app.package, app.version), p.on_surface_variant, text_style(12.5, 400, 1));
                            y += 32.0;
                            let bw = (w - 12.0) * 0.5;
                            let b_open = Rect { x, y, width: bw, height: 44.0 };
                            let ro = f.target(id_base + 270, b_open, "Open app", true, Role::Button);
                            f.paint_rect(b_open, 22.0, p.primary);
                            f.state_layer(ro, 22.0, p.on_primary);
                            f.styled_text(b_open, "Open", p.on_primary, text_style(14.0, 600, 1));
                            if ro.clicked {
                                action = SettingsAction::OpenApp(*idx);
                            }
                            let b_stop = Rect { x: x + bw + 12.0, y, width: bw, height: 44.0 };
                            let rs = f.target(id_base + 271, b_stop, "Force stop", true, Role::Button);
                            f.paint_rect(b_stop, 22.0, p.card_elevated);
                            f.state_layer(rs, 22.0, p.on_surface);
                            f.styled_text(b_stop, "Force stop", p.on_surface, text_style(14.0, 500, 1));
                            if rs.clicked {
                                if app.is_android {
                                    action = SettingsAction::ShirazStopPackage(app.package.clone());
                                } else {
                                    action = SettingsAction::ShowToast(format!("Force stopped {}", app.name));
                                }
                            }
                            y += 62.0;
                            paint_pref_row(f, id_base + 272, Rect { x, y, width: w, height: 58.0 }, "Notifications", "Allowed", p);
                            y += 60.0;
                            paint_pref_row(f, id_base + 273, Rect { x, y, width: w, height: 58.0 }, "Permissions", "Storage, Network", p);
                            y += 60.0;
                            paint_pref_row(f, id_base + 274, Rect { x, y, width: w, height: 58.0 }, "Storage & cache", &format!("{} MB used in internal storage", app.storage_mb), p);
                            y += 60.0;
                            paint_pref_row(f, id_base + 275, Rect { x, y, width: w, height: 58.0 }, "Battery", "Optimized", p);
                        }
                    }
                    SettingsPage::Notifications => {
                        // Live Lock Screen Notification Preview illustration
                        let preview_card = Rect { x, y, width: w, height: 168.0 };
                        paint_lock_notif_figure(
                            f,
                            preview_card,
                            sys.lock_screen_notifs,
                            sys.show_sensitive_on_lock,
                            ctx.clock,
                            p,
                        );
                        y += 182.0;

                        // Section: Privacy & Lock Screen
                        f.styled_text(
                            Rect { x: x + 6.0, y, width: w - 12.0, height: 22.0 },
                            "Privacy & lock screen",
                            p.primary,
                            text_style(13.5, 600, 0),
                        );
                        y += 28.0;
                        if paint_pref_row(
                            f,
                            id_base + 278,
                            Rect { x, y, width: w, height: 64.0 },
                            "Notifications on lock screen",
                            sys.lock_screen_notifs.label(),
                            p,
                        ) {
                            self.dialog = SettingsDialog::LockScreenNotifs;
                            self.dialog_start = now;
                        }
                        y += 66.0;
                        paint_pref_switch(
                            f,
                            id_base + 279,
                            Rect { x, y, width: w, height: 64.0 },
                            "Sensitive notifications",
                            Some("Show sensitive content when device is locked"),
                            &mut sys.show_sensitive_on_lock,
                            p,
                        );
                        y += 66.0;
                        paint_pref_switch(
                            f,
                            id_base + 284,
                            Rect { x, y, width: w, height: 64.0 },
                            "Wake screen for notifications",
                            Some("Turn on display briefly when new alerts arrive"),
                            &mut sys.wake_screen_for_notifs,
                            p,
                        );
                        y += 72.0;

                        // Section: General & Freedesktop Service
                        f.styled_text(
                            Rect { x: x + 6.0, y, width: w - 12.0, height: 22.0 },
                            "General",
                            p.primary,
                            text_style(13.5, 600, 0),
                        );
                        y += 28.0;
                        if paint_pref_row(
                            f,
                            id_base + 282,
                            Rect { x, y, width: w, height: 62.0 },
                            "Notification history",
                            &format!(
                                "{} active · {} in history",
                                ctx.active_notif_count,
                                ctx.history.len()
                            ),
                            p,
                        ) {
                            push_page = Some(SettingsPage::NotificationHistory);
                        }
                        y += 64.0;
                        paint_pref_switch(
                            f,
                            id_base + 280,
                            Rect { x, y, width: w, height: 62.0 },
                            "Notification dots on app icons",
                            Some("Show unread badges on home & drawer icons"),
                            &mut sys.notification_dots,
                            p,
                        );
                        y += 64.0;
                        paint_pref_switch(
                            f,
                            id_base + 285,
                            Rect { x, y, width: w, height: 62.0 },
                            "Hide silent notifications in status bar",
                            Some("Only show alerting badges in top bar"),
                            &mut sys.hide_silent_in_status_bar,
                            p,
                        );
                        y += 64.0;
                        paint_pref_switch(
                            f,
                            id_base + 281,
                            Rect { x, y, width: w, height: 62.0 },
                            "Do Not Disturb",
                            Some("Silence heads-up popups, wake, and badges"),
                            &mut sys.dnd_enabled,
                            p,
                        );
                        y += 72.0;

                        let test_btn = Rect { x, y, width: w, height: 48.0 };
                        let rt = f.target(id_base + 283, test_btn, "Send test notification", true, Role::Button);
                        f.paint_rect(test_btn, 24.0, p.primary_container);
                        f.state_layer(rt, 24.0, p.on_primary_container);
                        f.paint_system_icon(
                            Rect { x: test_btn.x + 18.0, y: test_btn.y + 13.0, width: 22.0, height: 22.0 },
                            SystemIcon::NotificationsActive,
                            p.on_primary_container,
                        );
                        f.styled_text(
                            test_btn,
                            "Send test notification",
                            p.on_primary_container,
                            text_style(14.5, 600, 1),
                        );
                        if rt.clicked {
                            action = SettingsAction::PostTestNotification {
                                app: "SpectreOS Settings".into(),
                                title: "Freedesktop Notification Test".into(),
                                body: "Delivered via org.freedesktop.Notifications v1.3".into(),
                            };
                        }
                    }
                    SettingsPage::NotificationHistory => {
                        paint_pref_switch(
                            f,
                            id_base + 286,
                            Rect { x, y, width: w, height: 62.0 },
                            "Use notification history",
                            Some("Record recent and dismissed notifications"),
                            &mut sys.notification_history_enabled,
                            p,
                        );
                        y += 72.0;
                        if ctx.history.is_empty() {
                            let empty = Rect { x, y, width: w, height: 112.0 };
                            f.paint_rect(empty, 20.0, p.card_elevated);
                            f.paint_system_icon(
                                Rect { x: empty.x + empty.width * 0.5 - 16.0, y: empty.y + 22.0, width: 32.0, height: 32.0 },
                                SystemIcon::History,
                                p.on_surface_variant,
                            );
                            f.styled_text(
                                Rect { x: empty.x + 16.0, y: empty.y + 64.0, width: empty.width - 32.0, height: 24.0 },
                                "No recent notifications recorded yet",
                                p.on_surface_variant,
                                text_style(14.0, 400, 1),
                            );
                        } else {
                            for entry in ctx.history.iter().take(24) {
                                let card = Rect { x, y, width: w, height: 68.0 };
                                f.paint_rect(card, 18.0, p.card_elevated);
                                let badge = Rect { x: card.x + 12.0, y: card.y + 17.0, width: 34.0, height: 34.0 };
                                f.paint_rect(badge, 17.0, if entry.urgency >= 2 { Color(0xDC, 0x26, 0x26, 255) } else { p.primary_container });
                                f.paint_system_icon(
                                    badge.inset(7.0),
                                    if entry.dismissed { SystemIcon::History } else { SystemIcon::NotificationsActive },
                                    if entry.urgency >= 2 { Color(255, 255, 255, 255) } else { p.on_primary_container },
                                );
                                f.styled_text(
                                    Rect { x: card.x + 56.0, y: card.y + 8.0, width: card.width - 70.0, height: 15.0 },
                                    &format!(
                                        "{} · {}{}",
                                        entry.app,
                                        entry.time_label,
                                        if entry.dismissed { " · Dismissed" } else { " · Active" }
                                    ),
                                    p.on_surface_variant,
                                    text_style(11.5, 400, 0),
                                );
                                f.styled_text(
                                    Rect { x: card.x + 56.0, y: card.y + 25.0, width: card.width - 70.0, height: 18.0 },
                                    &entry.title,
                                    p.on_surface,
                                    text_style(14.0, 600, 0),
                                );
                                f.styled_text(
                                    Rect { x: card.x + 56.0, y: card.y + 44.0, width: card.width - 70.0, height: 16.0 },
                                    &entry.body,
                                    p.on_surface_variant,
                                    text_style(12.0, 400, 0),
                                );
                                y += 74.0;
                            }
                            y += 8.0;
                            let clr = Rect { x, y, width: w, height: 44.0 };
                            let rc = f.target(id_base + 287, clr, "Clear notification history", true, Role::Button);
                            f.paint_rect(clr, 22.0, p.surface_container);
                            f.state_layer(rc, 22.0, p.on_surface);
                            f.styled_text(clr, "Clear history", p.on_surface, text_style(14.0, 500, 1));
                            if rc.clicked {
                                action = SettingsAction::ClearNotificationHistory;
                            }
                        }
                    }
                    SettingsPage::Battery => {
                        let card = Rect { x, y, width: w, height: 124.0 };
                        f.paint_rect(card, 24.0, p.card_elevated);
                        // High-res battery icon badge + percentage figure
                        let ic_r = Rect { x: card.x + card.width - 62.0, y: card.y + 18.0, width: 44.0, height: 44.0 };
                        f.paint_rect(ic_r, 22.0, p.primary_container);
                        f.paint_system_icon(ic_r.inset(10.0), SystemIcon::Battery, p.on_primary_container);
                        f.styled_text(
                            Rect { x: card.x + 20.0, y: card.y + 14.0, width: card.width - 96.0, height: 44.0 },
                            &format!("{}%", sys.battery_level),
                            p.primary,
                            text_style(38.0, 500, 0),
                        );
                        f.styled_text(
                            Rect { x: card.x + 20.0, y: card.y + 60.0, width: card.width - 40.0, height: 20.0 },
                            if sys.charging { "Charging via USB" } else { "Discharging · Normal usage" },
                            p.on_surface_variant,
                            text_style(13.5, 400, 0),
                        );
                        let bar = Rect { x: card.x + 20.0, y: card.y + 92.0, width: card.width - 40.0, height: 12.0 };
                        f.paint_rect(bar, 6.0, p.surface_container);
                        f.paint_rect(Rect { width: (bar.width * (sys.battery_level as f32 / 100.0)).max(12.0), ..bar }, 6.0, p.primary);
                        y += 138.0;
                        paint_pref_switch(f, id_base + 291, Rect { x, y, width: w, height: 62.0 }, "Battery Saver", Some("Reduce background activity & visual effects"), &mut sys.battery_saver, p);
                        y += 64.0;
                        paint_pref_switch(f, id_base + 292, Rect { x, y, width: w, height: 62.0 }, "Battery percentage", Some("Show battery percentage in status bar"), &mut sys.show_battery_pct, p);
                    }
                    SettingsPage::Storage => {
                        let used = ctx.device.storage_used_gb;
                        let total = ctx.device.storage_total_gb.max(1.0);
                        let free = (total - used).max(0.0);
                        let frac = (used / total).clamp(0.03, 1.0);
                        let pct = (frac * 100.0).round() as u32;
                        let card = Rect { x, y, width: w, height: 124.0 };
                        f.paint_rect(card, 24.0, p.card_elevated);
                        let ic_r = Rect { x: card.x + card.width - 62.0, y: card.y + 18.0, width: 44.0, height: 44.0 };
                        f.paint_rect(ic_r, 22.0, p.primary_container);
                        f.paint_system_icon(ic_r.inset(10.0), SystemIcon::Storage, p.on_primary_container);
                        f.styled_text(
                            Rect { x: card.x + 20.0, y: card.y + 16.0, width: card.width - 96.0, height: 34.0 },
                            &format!("{used:.1} GB used of {total:.0} GB"),
                            p.on_surface,
                            text_style(20.0, 500, 0),
                        );
                        f.styled_text(
                            Rect { x: card.x + 20.0, y: card.y + 52.0, width: card.width - 40.0, height: 18.0 },
                            &format!("{pct}% used · {free:.1} GB free"),
                            p.on_surface_variant,
                            text_style(13.5, 400, 0),
                        );
                        let bar = Rect { x: card.x + 20.0, y: card.y + 88.0, width: card.width - 40.0, height: 14.0 };
                        f.paint_rect(bar, 7.0, p.surface_container);
                        f.paint_rect(Rect { width: (bar.width * frac).max(14.0), ..bar }, 7.0, p.primary);
                        y += 138.0;
                        let rows = [
                            (SystemIcon::System, "System & Nix store", format!("{:.1} GB", used * 0.72)),
                            (SystemIcon::AppsGrid, "Apps & data", format!("{:.1} GB", used * 0.16)),
                            (SystemIcon::Camera, "Images & Videos", format!("{:.1} GB", used * 0.06)),
                            (SystemIcon::MusicNote, "Audio & Music", format!("{:.1} GB", used * 0.03)),
                            (SystemIcon::Storage, "Documents & Downloads", format!("{:.1} GB", used * 0.03)),
                        ];
                        for (i, (ic, lbl, sz)) in rows.iter().enumerate() {
                            let rr = Rect { x, y, width: w, height: 58.0 };
                            let r = f.target(id_base + 300 + i as u64, rr, lbl, true, Role::ListItem);
                            f.state_layer(r, 12.0, p.on_surface);
                            f.paint_system_icon(Rect { x: rr.x + 10.0, y: rr.y + 18.0, width: 22.0, height: 22.0 }, *ic, p.primary);
                            f.styled_text(Rect { x: rr.x + 48.0, y: rr.y + 8.0, width: rr.width - 64.0, height: 22.0 }, lbl, p.on_surface, text_style(15.5, 500, 0));
                            f.styled_text(Rect { x: rr.x + 48.0, y: rr.y + 32.0, width: rr.width - 64.0, height: 18.0 }, sz, p.on_surface_variant, text_style(12.5, 400, 0));
                            y += 60.0;
                        }
                    }
                    SettingsPage::Sound => {
                        let sliders: &[(u64, SystemIcon, &str, usize)] = &[
                            (id_base + 310, SystemIcon::MusicNote, "Media volume", 0),
                            (id_base + 311, SystemIcon::Ring, "Ring volume", 1),
                            (id_base + 312, SystemIcon::NotificationsActive, "Notification volume", 2),
                            (id_base + 313, SystemIcon::Alarm, "Alarm volume", 3),
                        ];
                        for &(sid, ic, lbl, idx) in sliders {
                            let val = match idx {
                                0 => &mut sys.vol_media,
                                1 => &mut sys.vol_ring,
                                2 => &mut sys.vol_notif,
                                _ => &mut sys.vol_alarm,
                            };
                            f.paint_system_icon(Rect { x: x + 6.0, y: y + 1.0, width: 20.0, height: 20.0 }, ic, p.primary);
                            f.styled_text(Rect { x: x + 34.0, y, width: w - 88.0, height: 22.0 }, lbl, p.on_surface, text_style(15.0, 500, 0));
                            f.styled_text(Rect { x: x + w - 50.0, y, width: 44.0, height: 22.0 }, &format!("{}%", (*val * 100.0).round() as u32), p.on_surface_variant, text_style(13.0, 400, 2));
                            paint_h_slider(f, sid, Rect { x, y: y + 24.0, width: w, height: 32.0 }, val, pointer, p);
                            y += 68.0;
                        }
                        f.styled_text(Rect { x: x + 6.0, y, width: w, height: 22.0 }, "Ringer mode", p.primary, text_style(13.5, 600, 0));
                        y += 28.0;
                        let modes = [(RingerMode::Vibrate, "Vibrate"), (RingerMode::Silent, "Silent"), (RingerMode::Ring, "Ring")];
                        let mw = (w - 16.0) / 3.0;
                        for (i, (m, lbl)) in modes.iter().enumerate() {
                            let mr = Rect { x: x + i as f32 * (mw + 8.0), y, width: mw, height: 42.0 };
                            let active = sys.ringer_mode == *m;
                            let r = f.target(id_base + 320 + i as u64, mr, lbl, true, Role::Radio);
                            f.paint_rect(mr, 21.0, if active { p.primary } else { p.card_elevated });
                            f.styled_text(mr, lbl, if active { p.on_primary } else { p.on_surface }, text_style(13.5, 500, 1));
                            if r.clicked {
                                sys.ringer_mode = *m;
                            }
                        }
                        y += 56.0;
                        paint_pref_switch(f, id_base + 325, Rect { x, y, width: w, height: 60.0 }, "Do Not Disturb", Some(if sys.dnd_enabled { "On" } else { "Off" }), &mut sys.dnd_enabled, p);
                    }
                    SettingsPage::Display => {
                        f.paint_system_icon(Rect { x: x + 6.0, y: y + 1.0, width: 20.0, height: 20.0 }, SystemIcon::Sun, p.primary);
                        f.styled_text(Rect { x: x + 34.0, y, width: w - 88.0, height: 22.0 }, "Brightness level", p.on_surface, text_style(15.0, 500, 0));
                        f.styled_text(Rect { x: x + w - 50.0, y, width: 44.0, height: 22.0 }, &format!("{}%", (sys.brightness * 100.0).round() as u32), p.on_surface_variant, text_style(13.0, 400, 2));
                        let mut br = ((sys.brightness - 0.15) / 0.85).clamp(0.0, 1.0);
                        if paint_h_slider(f, id_base + 330, Rect { x, y: y + 24.0, width: w, height: 32.0 }, &mut br, pointer, p) {
                            sys.brightness = 0.15 + br * 0.85;
                        }
                        y += 66.0;
                        paint_pref_switch(f, id_base + 331, Rect { x, y, width: w, height: 60.0 }, "Adaptive brightness", Some("Optimize brightness for available light"), &mut sys.auto_brightness, p);
                        y += 62.0;
                        paint_pref_switch(f, id_base + 332, Rect { x, y, width: w, height: 60.0 }, "Dark theme", Some("Use dark background across system UI"), &mut sys.dark_theme, p);
                        y += 62.0;
                        paint_pref_switch(f, id_base + 333, Rect { x, y, width: w, height: 60.0 }, "Night Light", Some("Tint screen amber for evening reading"), &mut sys.night_light, p);
                        y += 62.0;
                        if paint_pref_row(f, id_base + 334, Rect { x, y, width: w, height: 60.0 }, "Screen timeout", &format!("After {} of inactivity", TIMEOUT_LABELS[sys.screen_timeout_idx.min(5)]), p) {
                            self.dialog = SettingsDialog::ScreenTimeout;
                            self.dialog_start = now;
                        }
                        y += 62.0;
                        paint_pref_switch(f, id_base + 335, Rect { x, y, width: w, height: 60.0 }, "Auto-rotate screen", None, &mut sys.auto_rotate, p);
                        y += 62.0;
                        paint_pref_switch(f, id_base + 336, Rect { x, y, width: w, height: 60.0 }, "Always show time and info (AOD)", None, &mut sys.aod, p);
                    }
                    SettingsPage::Accessibility => {
                        paint_pref_switch(f, id_base + 340, Rect { x, y, width: w, height: 62.0 }, "Color inversion", Some("Subtle high-contrast color filter"), &mut sys.invert_colors, p);
                        y += 64.0;
                        paint_pref_row(f, id_base + 341, Rect { x, y, width: w, height: 60.0 }, "Display size and text", "Roboto 3 · 400×832 dp @ 2.7x", p);
                        y += 62.0;
                        paint_pref_row(f, id_base + 342, Rect { x, y, width: w, height: 60.0 }, "System controls", "Navigation & power button behavior", p);
                    }
                    SettingsPage::Security => {
                        if paint_pref_row(
                            f,
                            id_base + 350,
                            Rect { x, y, width: w, height: 62.0 },
                            "Screen lock",
                            sys.lock_method.label(),
                            p,
                        ) {
                            self.lock_entry.clear();
                            self.lock_error = None;
                            self.lock_step = if sys.lock_method == LockMethod::Swipe || sys.lock_secret.is_empty() {
                                ScreenLockStep::Choose
                            } else {
                                ScreenLockStep::VerifyCurrent
                            };
                            push_page = Some(SettingsPage::ScreenLock);
                        }
                        y += 64.0;
                        let auto_sub = format!(
                            "Sleep & lock after {} of inactivity",
                            TIMEOUT_LABELS[sys.screen_timeout_idx.min(5)]
                        );
                        let prev_auto = sys.auto_lock_enabled;
                        paint_pref_switch(
                            f,
                            id_base + 354,
                            Rect { x, y, width: w, height: 64.0 },
                            "Auto-lock after timeout",
                            Some(&auto_sub),
                            &mut sys.auto_lock_enabled,
                            p,
                        );
                        if sys.auto_lock_enabled != prev_auto {
                            save_lock_config(sys.lock_method, &sys.lock_secret, sys.auto_lock_enabled, sys.screen_timeout_idx);
                        }
                        y += 66.0;
                        if paint_pref_row(
                            f,
                            id_base + 359,
                            Rect { x, y, width: w, height: 60.0 },
                            "Screen timeout",
                            TIMEOUT_LABELS[sys.screen_timeout_idx.min(5)],
                            p,
                        ) {
                            self.dialog = SettingsDialog::ScreenTimeout;
                            self.dialog_start = now;
                        }
                        y += 62.0;
                        if paint_pref_row(
                            f,
                            id_base + 353,
                            Rect { x, y, width: w, height: 60.0 },
                            "Lock screen now",
                            "Test your lock screen & notifications immediately",
                            p,
                        ) {
                            action = SettingsAction::LockNow;
                        }
                        y += 62.0;
                        paint_pref_row(f, id_base + 351, Rect { x, y, width: w, height: 60.0 }, "Encryption & credentials", "Device storage protected", p);
                        y += 62.0;
                        paint_pref_row(f, id_base + 352, Rect { x, y, width: w, height: 60.0 }, "Security update", "November 2026 · SpectreOS", p);
                    }
                    SettingsPage::ScreenLock => {
                        let inp = f.input();
                        let step = self.lock_step.clone();
                        match step {
                            ScreenLockStep::VerifyCurrent => {
                                let label = match sys.lock_method {
                                    LockMethod::Pin => "Confirm your current PIN",
                                    LockMethod::Password => "Confirm your current password",
                                    LockMethod::Swipe => "Verified",
                                };
                                f.styled_text(
                                    Rect { x: x + 6.0, y, width: w - 12.0, height: 24.0 },
                                    self.lock_error.as_deref().unwrap_or(label),
                                    if self.lock_error.is_some() { Color(240, 105, 105, 255) } else { p.on_surface },
                                    text_style(16.0, 500, 1),
                                );
                                y += 34.0;
                                let field = Rect { x: x + 16.0, y, width: w - 32.0, height: 46.0 };
                                f.paint_rect(field, 14.0, p.surface_container);
                                let masked = if self.lock_entry.is_empty() {
                                    "Enter current secret".to_string()
                                } else {
                                    "• ".repeat(self.lock_entry.len()).trim_end().to_string()
                                };
                                f.styled_text(
                                    field,
                                    &masked,
                                    if self.lock_entry.is_empty() { p.on_surface_variant } else { p.on_surface },
                                    text_style(18.0, 600, 1),
                                );
                                y += 60.0;
                                if sys.lock_method == LockMethod::Pin {
                                    match paint_pin_keypad(
                                        f,
                                        Rect { x: x + 16.0, y, width: w - 32.0, height: 300.0 },
                                        inp,
                                        p.primary,
                                        p.surface_container,
                                        p.on_surface,
                                    ) {
                                        KeyPadEvent::Digit(ch) if self.lock_entry.len() < 12 => {
                                            self.lock_entry.push(ch);
                                            self.lock_error = None;
                                        }
                                        KeyPadEvent::Backspace => {
                                            self.lock_entry.pop();
                                            self.lock_error = None;
                                        }
                                        KeyPadEvent::Submit => {
                                            if self.lock_entry == sys.lock_secret {
                                                self.lock_entry.clear();
                                                self.lock_error = None;
                                                self.lock_step = ScreenLockStep::Choose;
                                            } else {
                                                self.lock_entry.clear();
                                                self.lock_error = Some("Wrong PIN — try again".into());
                                            }
                                        }
                                        _ => {}
                                    }
                                } else {
                                    match paint_qwerty_keyboard(
                                        f,
                                        Rect { x: bounds.x + 4.0, y: y + 12.0, width: bounds.width - 8.0, height: 226.0 },
                                        inp,
                                        &mut self.lock_kbd,
                                        p.primary,
                                    ) {
                                        KeyboardEvent::Char(ch) if self.lock_entry.len() < 32 => {
                                            self.lock_entry.push(ch);
                                            self.lock_error = None;
                                        }
                                        KeyboardEvent::Backspace => {
                                            self.lock_entry.pop();
                                            self.lock_error = None;
                                        }
                                        KeyboardEvent::Submit => {
                                            if self.lock_entry == sys.lock_secret {
                                                self.lock_entry.clear();
                                                self.lock_error = None;
                                                self.lock_step = ScreenLockStep::Choose;
                                            } else {
                                                self.lock_entry.clear();
                                                self.lock_error = Some("Wrong password — try again".into());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            ScreenLockStep::Choose => {
                                f.styled_text(
                                    Rect { x: x + 6.0, y, width: w - 12.0, height: 22.0 },
                                    &format!("Current screen lock: {}", sys.lock_method.label()),
                                    p.primary,
                                    text_style(13.5, 600, 0),
                                );
                                y += 30.0;
                                if paint_pref_row(f, id_base + 355, Rect { x, y, width: w, height: 64.0 }, "Swipe", "No screen lock protection", p) {
                                    sys.lock_method = LockMethod::Swipe;
                                    sys.lock_secret.clear();
                                    save_lock_config(sys.lock_method, &sys.lock_secret, sys.auto_lock_enabled, sys.screen_timeout_idx);
                                    action = SettingsAction::ShowToast("Screen lock set to Swipe".into());
                                }
                                y += 66.0;
                                if paint_pref_row(f, id_base + 356, Rect { x, y, width: w, height: 64.0 }, "PIN", "Unlock with 4 or more digits", p) {
                                    self.lock_entry.clear();
                                    self.lock_error = None;
                                    self.lock_step = ScreenLockStep::EnterNew(LockMethod::Pin);
                                }
                                y += 66.0;
                                if paint_pref_row(f, id_base + 357, Rect { x, y, width: w, height: 64.0 }, "Password", "Unlock with letters, numbers & symbols", p) {
                                    self.lock_entry.clear();
                                    self.lock_error = None;
                                    self.lock_step = ScreenLockStep::EnterNew(LockMethod::Password);
                                }
                                y += 78.0;
                                if paint_pref_row(f, id_base + 358, Rect { x, y, width: w, height: 60.0 }, "Lock screen now", "Preview the lock screen immediately", p) {
                                    action = SettingsAction::LockNow;
                                }
                            }
                            ScreenLockStep::EnterNew(target) | ScreenLockStep::ConfirmNew(target, _) => {
                                let is_confirm = matches!(step, ScreenLockStep::ConfirmNew(_, _));
                                let heading = match (target, is_confirm) {
                                    (LockMethod::Pin, false) => "Set a new PIN (at least 4 digits)",
                                    (LockMethod::Pin, true) => "Re-enter your new PIN to confirm",
                                    (LockMethod::Password, false) => "Set a new password (4+ chars)",
                                    (LockMethod::Password, true) => "Re-enter your password to confirm",
                                    _ => "Set screen lock",
                                };
                                f.styled_text(
                                    Rect { x: x + 6.0, y, width: w - 12.0, height: 24.0 },
                                    self.lock_error.as_deref().unwrap_or(heading),
                                    if self.lock_error.is_some() { Color(240, 105, 105, 255) } else { p.on_surface },
                                    text_style(15.0, 500, 1),
                                );
                                y += 34.0;
                                let field = Rect { x: x + 16.0, y, width: w - 32.0, height: 46.0 };
                                f.paint_rect(field, 14.0, p.surface_container);
                                let shown = if self.lock_entry.is_empty() {
                                    if target == LockMethod::Pin { "New PIN" } else { "New password" }.to_string()
                                } else if target == LockMethod::Password {
                                    self.lock_entry.clone()
                                } else {
                                    "• ".repeat(self.lock_entry.len()).trim_end().to_string()
                                };
                                f.styled_text(
                                    field,
                                    &shown,
                                    if self.lock_entry.is_empty() { p.on_surface_variant } else { p.on_surface },
                                    text_style(17.5, 600, 1),
                                );
                                y += 58.0;

                                let mut submitted = false;
                                if target == LockMethod::Pin {
                                    match paint_pin_keypad(
                                        f,
                                        Rect { x: x + 16.0, y, width: w - 32.0, height: 300.0 },
                                        inp,
                                        p.primary,
                                        p.surface_container,
                                        p.on_surface,
                                    ) {
                                        KeyPadEvent::Digit(ch) if self.lock_entry.len() < 12 => {
                                            self.lock_entry.push(ch);
                                            self.lock_error = None;
                                        }
                                        KeyPadEvent::Backspace => {
                                            self.lock_entry.pop();
                                            self.lock_error = None;
                                        }
                                        KeyPadEvent::Submit => submitted = true,
                                        _ => {}
                                    }
                                } else {
                                    match paint_qwerty_keyboard(
                                        f,
                                        Rect { x: bounds.x + 4.0, y: y + 10.0, width: bounds.width - 8.0, height: 226.0 },
                                        inp,
                                        &mut self.lock_kbd,
                                        p.primary,
                                    ) {
                                        KeyboardEvent::Char(ch) if self.lock_entry.len() < 32 => {
                                            self.lock_entry.push(ch);
                                            self.lock_error = None;
                                        }
                                        KeyboardEvent::Backspace => {
                                            self.lock_entry.pop();
                                            self.lock_error = None;
                                        }
                                        KeyboardEvent::Submit => submitted = true,
                                        _ => {}
                                    }
                                }

                                if submitted {
                                    match step {
                                        ScreenLockStep::EnterNew(m) => {
                                            if self.lock_entry.len() < 4 {
                                                self.lock_error = Some("Must be at least 4 characters".into());
                                            } else {
                                                let first = std::mem::take(&mut self.lock_entry);
                                                self.lock_error = None;
                                                self.lock_step = ScreenLockStep::ConfirmNew(m, first);
                                            }
                                        }
                                        ScreenLockStep::ConfirmNew(m, first) => {
                                            if self.lock_entry == first {
                                                sys.lock_method = m;
                                                sys.lock_secret = first;
                                                save_lock_config(sys.lock_method, &sys.lock_secret, sys.auto_lock_enabled, sys.screen_timeout_idx);
                                                self.lock_entry.clear();
                                                self.lock_error = None;
                                                self.lock_step = ScreenLockStep::Choose;
                                                action = SettingsAction::ShowToast(format!("Screen lock updated to {}", m.label()));
                                            } else {
                                                self.lock_entry.clear();
                                                self.lock_error = Some("Doesn't match — try again".into());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    SettingsPage::Privacy => {
                        paint_pref_switch(f, id_base + 360, Rect { x, y, width: w, height: 62.0 }, "Camera access", Some("Synced with Quick Settings"), &mut sys.camera_access, p);
                        y += 64.0;
                        paint_pref_switch(f, id_base + 361, Rect { x, y, width: w, height: 62.0 }, "Microphone access", Some("Synced with Quick Settings"), &mut sys.mic_access, p);
                        y += 64.0;
                        paint_pref_row(f, id_base + 362, Rect { x, y, width: w, height: 60.0 }, "Privacy dashboard", "Camera, Microphone, Location", p);
                    }
                    SettingsPage::Location => {
                        paint_pref_switch(f, id_base + 370, Rect { x, y, width: w, height: 62.0 }, "Use location", Some("Allow apps with permission to request location"), &mut sys.location, p);
                        y += 66.0;
                        paint_pref_row(f, id_base + 371, Rect { x, y, width: w, height: 58.0 }, "Recent access", "Weather · Browser", p);
                    }
                    SettingsPage::Safety => {
                        paint_pref_row(f, id_base + 380, Rect { x, y, width: w, height: 60.0 }, "Emergency SOS", "Press Power button 5 times quickly", p);
                        y += 62.0;
                        paint_pref_row(f, id_base + 381, Rect { x, y, width: w, height: 60.0 }, "Medical information", "Add blood type & emergency contacts", p);
                    }
                    SettingsPage::Accounts => {
                        paint_pref_row(f, id_base + 390, Rect { x, y, width: w, height: 60.0 }, "spectre@spectreos.org", "SpectreOS · Sync on", p);
                        y += 62.0;
                        paint_pref_switch(f, id_base + 391, Rect { x, y, width: w, height: 62.0 }, "Automatically sync app data", None, &mut sys.auto_sync, p);
                    }
                    SettingsPage::System => {
                        let nav_summary = match sys.nav_mode {
                            NavMode::Gestural => "Gesture navigation",
                            NavMode::ThreeButton => "3-button navigation",
                        };
                        if paint_pref_row(f, id_base + 400, Rect { x, y, width: w, height: 62.0 }, "Gestures", nav_summary, p) {
                            push_page = Some(SettingsPage::Gestures);
                        }
                        y += 64.0;
                        if paint_pref_row(f, id_base + 401, Rect { x, y, width: w, height: 62.0 }, "Buttons", "Power menu, torch, double-tap sleep", p) {
                            push_page = Some(SettingsPage::LineageButtons);
                        }
                        y += 64.0;
                        if paint_pref_row(f, id_base + 402, Rect { x, y, width: w, height: 62.0 }, "Status bar", "Clock position, battery percentage", p) {
                            push_page = Some(SettingsPage::LineageStatusBar);
                        }
                        y += 64.0;
                        paint_pref_row(f, id_base + 403, Rect { x, y, width: w, height: 62.0 }, "Languages & input", "English (US) · فارسی (Spectre Keyboard)", p);
                        y += 64.0;
                        if paint_pref_row(f, id_base + 404, Rect { x, y, width: w, height: 62.0 }, "Date & time", "Time zone and local time", p) {
                            push_page = Some(SettingsPage::DateTime);
                        }
                        y += 64.0;
                        if sys.dev_unlocked
                            && paint_pref_row(f, id_base + 405, Rect { x, y, width: w, height: 62.0 }, "Developer options", if sys.usb_debugging { "USB debugging on" } else { "Enabled" }, p)
                        {
                            push_page = Some(SettingsPage::DevOptions);
                        }
                    }
                    SettingsPage::DateTime | SettingsPage::TimeZoneRegion(_) => {
                        let zones = &mut self.time_zones;
                        f.styled_text(Rect { x, y, width: w, height: 35.0 }, &format!("{} · {}", ctx.date, ctx.clock), p.on_surface, text_style(21.0, 500, 0));
                        y += 38.0;
                        let status = if zones.busy() { "Updating time zone…" } else { &zones.current };
                        f.styled_text(Rect { x, y, width: w, height: 30.0 }, status, p.on_surface_variant, text_style(15.0, 400, 0));
                        y += 36.0;
                        if let Some(error) = &zones.error {
                            if paint_pref_row(f, id_base + 1200, Rect { x, y, width: w, height: 62.0 }, "Could not update time zone · Retry", error, p) {
                                zones.request(None);
                            }
                            y += 64.0;
                        }
                        let mut choice = None;
                        if let SettingsPage::TimeZoneRegion(area) = page {
                            for (i, zone) in zones.zones.iter().filter(|z| time_zone::region(z) == area).enumerate() {
                                let row = Rect { x, y, width: w, height: 60.0 };
                                if y + 60.0 >= body.y && y < body.y + body.height {
                                    let city = zone.split_once('/').map_or(zone.as_str(), |(_, city)| city).replace('_', " ");
                                    let summary = if *zone == zones.current { "Selected" } else { zone.as_str() };
                                    if paint_pref_row(f, id_base + 1300 + i as u64, row, &city, summary, p) {
                                        choice = Some(zone.clone());
                                    }
                                }
                                y += 60.0;
                            }
                        } else {
                            for (i, zone) in zones.recent.iter().map(String::as_str).chain(std::iter::once("UTC"))
                                .filter(|z| *z != zones.current && zones.zones.iter().any(|item| item == z))
                                .enumerate() {
                                if i == 1 && zones.recent.as_deref() == Some("UTC") { continue; }
                                if paint_pref_row(f, id_base + 1210 + i as u64, Rect { x, y, width: w, height: 60.0 }, zone, "Use this time zone", p) {
                                    choice = Some(zone.to_owned());
                                }
                                y += 60.0;
                            }
                            for (i, area) in zones.regions.iter().enumerate() {
                                if paint_pref_row(f, id_base + 1230 + i as u64, Rect { x, y, width: w, height: 60.0 }, area, "Choose a time zone", p) {
                                    push_page = Some(SettingsPage::TimeZoneRegion(area.clone()));
                                }
                                y += 60.0;
                            }
                        }
                        if let Some(zone) = choice {
                            zones.request(Some(&zone));
                            f.wake_at(now + 0.05);
                        }
                    }
                    SettingsPage::Gestures => {
                        let nav_lbl = match sys.nav_mode {
                            NavMode::Gestural => "Gesture navigation",
                            NavMode::ThreeButton => "3-button navigation",
                        };
                        if paint_pref_row(f, id_base + 410, Rect { x, y, width: w, height: 64.0 }, "System navigation", nav_lbl, p) {
                            push_page = Some(SettingsPage::Navigation);
                        }
                        y += 66.0;
                        paint_pref_switch(f, id_base + 411, Rect { x, y, width: w, height: 62.0 }, "Double-tap to sleep", Some("Double-tap status bar to lock"), &mut sys.double_tap_sleep, p);
                    }
                    SettingsPage::Navigation => {
                        let card = Rect { x, y, width: w, height: 170.0 };
                        paint_navigation_figure(f, card, sys.nav_mode, p);
                        y += 186.0;

                        for (i, (mode, title, desc)) in [
                            (NavMode::Gestural, "Gesture navigation", "Drag from left/right edge for Back, swipe up for Home"),
                            (NavMode::ThreeButton, "3-button navigation", "Go Back, Home, and switch apps with bottom buttons"),
                        ]
                        .iter()
                        .enumerate()
                        {
                            let rr = Rect { x, y, width: w, height: 72.0 };
                            let full_rr = Rect { x: bounds.x, y: rr.y, width: bounds.width, height: rr.height };
                            let r = f.target(id_base + 420 + i as u64, full_rr, title, true, Role::Radio);
                            f.state_layer(r, 0.0, p.on_surface);
                            paint_radio(f, Rect { x: rr.x + 8.0, y: rr.y + 22.0, width: 28.0, height: 28.0 }, sys.nav_mode == *mode, p);
                            f.styled_text(Rect { x: rr.x + 48.0, y: rr.y + 12.0, width: rr.width - 56.0, height: 22.0 }, title, p.on_surface, text_style(16.0, 500, 0));
                            f.styled_text(Rect { x: rr.x + 48.0, y: rr.y + 36.0, width: rr.width - 56.0, height: 28.0 }, desc, p.on_surface_variant, text_style(12.5, 400, 0));
                            if r.clicked {
                                sys.nav_mode = *mode;
                                action = SettingsAction::ShowToast(format!("Switched to {title}"));
                            }
                            y += 76.0;
                        }
                    }
                    SettingsPage::LineageButtons => {
                        paint_pref_switch(f, id_base + 430, Rect { x, y, width: w, height: 64.0 }, "Long-press power for torch", Some("Toggle flashlight when screen is off"), &mut sys.torch_power_longpress, p);
                        y += 66.0;
                        paint_pref_switch(f, id_base + 431, Rect { x, y, width: w, height: 64.0 }, "Double-tap status bar to sleep", None, &mut sys.double_tap_sleep, p);
                        y += 66.0;
                        if paint_pref_row(f, id_base + 432, Rect { x, y, width: w, height: 60.0 }, "Volume panel", "Open side-by-side volume HUD", p) {
                            action = SettingsAction::OpenVolume;
                        }
                    }
                    SettingsPage::LineageStatusBar => {
                        f.styled_text(Rect { x: x + 6.0, y, width: w, height: 22.0 }, "Clock position", p.primary, text_style(13.5, 600, 0));
                        y += 28.0;
                        let cw = (w - 16.0) / 3.0;
                        for (i, (pos, lbl)) in [(ClockPos::Left, "Left"), (ClockPos::Center, "Center"), (ClockPos::Right, "Right")].iter().enumerate() {
                            let cr = Rect { x: x + i as f32 * (cw + 8.0), y, width: cw, height: 40.0 };
                            let active = sys.clock_pos == *pos;
                            let r = f.target(id_base + 440 + i as u64, cr, lbl, true, Role::Radio);
                            f.paint_rect(cr, 20.0, if active { p.primary } else { p.card_elevated });
                            f.styled_text(cr, lbl, if active { p.on_primary } else { p.on_surface }, text_style(13.5, 500, 1));
                            if r.clicked {
                                sys.clock_pos = *pos;
                            }
                        }
                        y += 56.0;
                        paint_pref_switch(f, id_base + 445, Rect { x, y, width: w, height: 62.0 }, "Battery percentage", Some("Show exact percentage in status bar"), &mut sys.show_battery_pct, p);
                    }
                    SettingsPage::DevOptions => {
                        paint_pref_switch(f, id_base + 450, Rect { x, y, width: w, height: 64.0 }, "USB debugging", Some("Show USB diagnostics state"), &mut sys.usb_debugging, p);
                        y += 66.0;
                        paint_pref_switch(f, id_base + 451, Rect { x, y, width: w, height: 64.0 }, "Stay awake", Some("Screen will never sleep while charging"), &mut sys.stay_awake, p);
                        y += 66.0;
                        paint_pref_switch(f, id_base + 452, Rect { x, y, width: w, height: 64.0 }, "Show taps", Some("Show visual pointer dot for touch feedback"), &mut sys.show_taps, p);
                        y += 66.0;
                        if paint_pref_row(f, id_base + 453, Rect { x, y, width: w, height: 62.0 }, "Window animation scale", ANIM_LABELS[sys.anim_scale_idx.min(ANIM_LABELS.len() - 1)], p) {
                            self.dialog = SettingsDialog::AnimScale;
                            self.dialog_start = now;
                        }
                    }
                    SettingsPage::About => {
                        // Crisp SpectreOS / Poco F1 device hero card
                        let hero = Rect { x, y, width: w, height: 106.0 };
                        f.paint_rect(hero, 24.0, p.card_elevated);
                        let badge = Rect { x: hero.x + 18.0, y: hero.y + 25.0, width: 56.0, height: 56.0 };
                        f.paint_rect(badge, 28.0, p.primary_container);
                        f.paint_system_icon(badge.inset(12.0), SystemIcon::PhoneInfo, p.on_primary_container);
                        f.styled_text(Rect { x: hero.x + 88.0, y: hero.y + 20.0, width: hero.width - 104.0, height: 26.0 }, &ctx.device.device_name, p.on_surface, text_style(20.0, 600, 0));
                        f.styled_text(Rect { x: hero.x + 88.0, y: hero.y + 48.0, width: hero.width - 104.0, height: 18.0 }, &ctx.device.model, p.on_surface_variant, text_style(13.0, 400, 0));
                        f.styled_text(Rect { x: hero.x + 88.0, y: hero.y + 68.0, width: hero.width - 104.0, height: 18.0 }, &format!("{:.1} GB RAM · {:.0} GB Storage", ctx.device.ram_total_gb, ctx.device.storage_total_gb), p.primary, text_style(12.5, 500, 0));
                        y += 120.0;

                        let specs = [
                            ("Device name", ctx.device.device_name.as_str()),
                            ("Model", ctx.device.model.as_str()),
                            ("OS version", ctx.device.os_version.as_str()),
                            ("Compositor", "Wisp (Smithay + libbhoo GLES3)"),
                            ("Notifications", "org.freedesktop.Notifications v1.3"),
                            ("Kernel version", ctx.device.kernel.as_str()),
                            ("IP address", ctx.device.ip_address.as_str()),
                            ("Wi-Fi MAC address", ctx.device.wifi_mac.as_str()),
                            ("Legal information", "SpectreOS & open-source notices (GPL-3.0 / Apache-2.0)"),
                            ("Build number", "spectreos-beryllium-26.05-wisp"),
                        ];
                        let build_idx = specs.len() - 1;
                        for (i, (k, v)) in specs.iter().enumerate() {
                            if paint_pref_row(f, id_base + 460 + i as u64, Rect { x, y, width: w, height: 60.0 }, k, v, p)
                                && i == build_idx
                            {
                                sys.tap_build_count = sys.tap_build_count.saturating_add(1);
                                if sys.tap_build_count >= 3 {
                                    sys.dev_unlocked = true;
                                    action = SettingsAction::ShowToast("You are now a developer!".into());
                                } else {
                                    action = SettingsAction::ShowToast(format!(
                                        "{} more taps to unlock Developer options",
                                        3 - sys.tap_build_count
                                    ));
                                }
                            }
                            y += 62.0;
                        }
                    }
                    SettingsPage::Home | SettingsPage::Search => {}
                }
            });
        });

        let appbar = Rect {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bar_h,
        };
        let elev_t = smoothstep(0.55, 1.0, collapse);
        f.paint_rect(appbar, 0.0, p.settings_bg.mix(p.surface_container, elev_t));
        let back_r = Rect {
            x: bounds.x + 8.0,
            y: bounds.y + 6.0,
            width: 44.0,
            height: 44.0,
        };
        let rb = f.target(id_base + 100, back_r, "Navigate up", true, Role::Button);
        f.state_layer(rb, 22.0, p.on_surface);
        f.paint_system_icon(back_r.inset(11.0), SystemIcon::Back, p.on_surface);

        let title_x = lerp(bounds.x + 24.0, bounds.x + 60.0, collapse);
        let title_y = lerp(bounds.y + 76.0, bounds.y + 15.0, collapse);
        let avail_w = (bounds.width - (title_x - bounds.x) - 16.0).max(80.0);
        if collapse <= 0.005 {
            f.styled_text(
                Rect {
                    x: title_x,
                    y: title_y,
                    width: avail_w,
                    height: 40.0,
                },
                title,
                p.on_surface,
                text_style(32.0, 400, 0),
            );
        } else if collapse >= 0.995 {
            f.styled_text(
                Rect {
                    x: title_x,
                    y: bounds.y + 13.0,
                    width: avail_w,
                    height: 28.0,
                },
                title,
                p.on_surface,
                text_style(20.0, 500, 0),
            );
        } else {
            let title_scale = lerp(1.0, 20.0 / 32.0, collapse);
            f.push_transform([title_x, title_y], [0.0, 0.0], [title_scale, title_scale], 0.0);
            f.styled_text(
                Rect {
                    x: title_x,
                    y: title_y,
                    width: avail_w / title_scale,
                    height: 40.0,
                },
                title,
                p.on_surface,
                text_style(32.0, 400, 0),
            );
            f.pop_transform();
        }
        if rb.clicked {
            self.go_back(now);
            return SettingsAction::None;
        }

        if let Some(pg) = push_page {
            self.navigate(pg, now);
        }
        action
    }

    fn render_dialog(
        &mut self,
        f: &mut Frame<'_>,
        bounds: Rect,
        id_base: u64,
        sys: &mut SystemState,
        p: &Palette,
        now: f64,
    ) -> SettingsAction {
        let mut action = SettingsAction::None;
        let dur = 0.22 * sys.anim_scale() as f64;
        let raw = if f.reduced_motion() || dur <= 0.001 {
            1.0
        } else {
            ((now - self.dialog_start) / dur).clamp(0.0, 1.0) as f32
        };
        if raw < 1.0 {
            f.continuous(bounds);
        }
        let e = emphasized_decelerate(raw);
        f.paint_rect(bounds, 0.0, Color(0, 0, 0, (128.0 * e) as u8));

        let card_w = (bounds.width - 40.0).min(340.0);
        let card_h = match self.dialog {
            SettingsDialog::PrivateDns => 348.0,
            SettingsDialog::ScreenTimeout => 364.0,
            SettingsDialog::AnimScale => 78.0 + ANIM_LABELS.len() as f32 * 44.0,
            SettingsDialog::LockScreenNotifs => 316.0,
            SettingsDialog::None => 0.0,
        };
        let card = Rect {
            x: bounds.x + (bounds.width - card_w) * 0.5,
            y: bounds.y + (bounds.height - card_h) * 0.5,
            width: card_w,
            height: card_h,
        };
        let sc = lerp(0.90, 1.0, e);
        if sc < 0.999 {
            f.push_transform(
                [card.x + card.width * 0.5, card.y + card.height * 0.5],
                [0.0, (1.0 - e) * 10.0],
                [sc, sc],
                0.0,
            );
            f.push_layer(bounds, e);
        }
        f.elevation(card, 28.0, 4);
        f.paint_rect(card, 28.0, p.card_pure);

        match self.dialog {
            SettingsDialog::PrivateDns => {
                f.styled_text(
                    Rect { x: card.x + 24.0, y: card.y + 22.0, width: card.width - 48.0, height: 28.0 },
                    "Select Private DNS Mode",
                    p.on_surface,
                    text_style(19.0, 500, 0),
                );
                let opts = [
                    (DnsMode::Off, "Off"),
                    (DnsMode::Cloudflare, "Cloudflare DNS"),
                    (DnsMode::Automatic, "Automatic"),
                    (DnsMode::Custom, "Private DNS provider hostname"),
                ];
                let mut y = card.y + 62.0;
                for (i, (m, lbl)) in opts.iter().enumerate() {
                    let rr = Rect { x: card.x + 16.0, y, width: card.width - 32.0, height: 42.0 };
                    let r = f.target(id_base + 10 + i as u64, rr, lbl, true, Role::Radio);
                    paint_radio(f, Rect { x: rr.x + 4.0, y: rr.y + 9.0, width: 24.0, height: 24.0 }, self.dns_draft_mode == *m, p);
                    f.styled_text(Rect { x: rr.x + 40.0, y: rr.y, width: rr.width - 48.0, height: 42.0 }, lbl, p.on_surface, text_style(14.5, 400, 0));
                    if r.clicked {
                        self.dns_draft_mode = *m;
                    }
                    y += 44.0;
                }
                let fld = Rect { x: card.x + 24.0, y: y + 4.0, width: card.width - 48.0, height: 38.0 };
                f.target(ID_DNS_HOSTNAME, fld, "DNS provider hostname", true, Role::TextField);
                f.styled_text(
                    fld,
                    if self.dns_draft.is_empty() { "Enter hostname of DNS provider" } else { &self.dns_draft },
                    if self.dns_draft.is_empty() { p.on_surface_variant } else { p.on_surface },
                    text_style(14.0, 400, 0),
                );
                f.paint_line([fld.x, fld.y + fld.height], [fld.x + fld.width, fld.y + fld.height], 1.2, p.outline);

                let cancel_r = Rect { x: card.x + card.width - 156.0, y: card.y + card.height - 52.0, width: 72.0, height: 36.0 };
                let save_r = Rect { x: card.x + card.width - 80.0, y: card.y + card.height - 52.0, width: 64.0, height: 36.0 };
                if f.target(id_base + 20, cancel_r, "Cancel", true, Role::Button).clicked {
                    self.dialog = SettingsDialog::None;
                }
                f.styled_text(cancel_r, "Cancel", p.primary, text_style(14.0, 500, 1));
                if f.target(id_base + 21, save_r, "Save", true, Role::Button).clicked {
                    sys.dns_mode = self.dns_draft_mode;
                    if !self.dns_draft.trim().is_empty() {
                        sys.dns_host = self.dns_draft.trim().to_owned();
                    }
                    self.dialog = SettingsDialog::None;
                    action = SettingsAction::ShowToast("Private DNS updated".into());
                }
                f.styled_text(save_r, "Save", p.primary, text_style(14.0, 600, 1));
            }
            SettingsDialog::ScreenTimeout => {
                f.styled_text(
                    Rect { x: card.x + 24.0, y: card.y + 22.0, width: card.width - 48.0, height: 28.0 },
                    "Screen timeout",
                    p.on_surface,
                    text_style(20.0, 500, 0),
                );
                for (i, &lbl) in TIMEOUT_LABELS.iter().enumerate() {
                    let rr = Rect { x: card.x + 16.0, y: card.y + 62.0 + i as f32 * 42.0, width: card.width - 32.0, height: 40.0 };
                    let r = f.target(id_base + 30 + i as u64, rr, lbl, true, Role::Radio);
                    paint_radio(f, Rect { x: rr.x + 6.0, y: rr.y + 8.0, width: 24.0, height: 24.0 }, sys.screen_timeout_idx == i, p);
                    f.styled_text(Rect { x: rr.x + 42.0, y: rr.y, width: rr.width - 48.0, height: 40.0 }, lbl, p.on_surface, text_style(15.0, 400, 0));
                    if r.clicked {
                        sys.screen_timeout_idx = i;
                        save_lock_config(sys.lock_method, &sys.lock_secret, sys.auto_lock_enabled, sys.screen_timeout_idx);
                        self.dialog = SettingsDialog::None;
                    }
                }
                let cancel_r = Rect { x: card.x + card.width - 96.0, y: card.y + card.height - 48.0, width: 76.0, height: 34.0 };
                if f.target(id_base + 39, cancel_r, "Cancel", true, Role::Button).clicked {
                    self.dialog = SettingsDialog::None;
                }
                f.styled_text(cancel_r, "Cancel", p.primary, text_style(14.0, 500, 1));
            }
            SettingsDialog::AnimScale => {
                f.styled_text(
                    Rect { x: card.x + 24.0, y: card.y + 22.0, width: card.width - 48.0, height: 28.0 },
                    "Window animation scale",
                    p.on_surface,
                    text_style(19.0, 500, 0),
                );
                for (i, &lbl) in ANIM_LABELS.iter().enumerate() {
                    let rr = Rect { x: card.x + 16.0, y: card.y + 62.0 + i as f32 * 44.0, width: card.width - 32.0, height: 42.0 };
                    let r = f.target(id_base + 40 + i as u64, rr, lbl, true, Role::Radio);
                    paint_radio(f, Rect { x: rr.x + 6.0, y: rr.y + 9.0, width: 24.0, height: 24.0 }, sys.anim_scale_idx == i, p);
                    f.styled_text(Rect { x: rr.x + 42.0, y: rr.y, width: rr.width - 48.0, height: 42.0 }, lbl, p.on_surface, text_style(14.5, 400, 0));
                    if r.clicked {
                        sys.anim_scale_idx = i;
                        self.dialog = SettingsDialog::None;
                    }
                }
            }
            SettingsDialog::LockScreenNotifs => {
                f.styled_text(
                    Rect { x: card.x + 24.0, y: card.y + 22.0, width: card.width - 48.0, height: 28.0 },
                    "Notifications on lock screen",
                    p.on_surface,
                    text_style(18.5, 500, 0),
                );
                for (i, &mode) in LockScreenNotifMode::ALL.iter().enumerate() {
                    let rr = Rect { x: card.x + 16.0, y: card.y + 62.0 + i as f32 * 48.0, width: card.width - 32.0, height: 46.0 };
                    let r = f.target(id_base + 50 + i as u64, rr, mode.label(), true, Role::Radio);
                    paint_radio(f, Rect { x: rr.x + 6.0, y: rr.y + 11.0, width: 24.0, height: 24.0 }, sys.lock_screen_notifs == mode, p);
                    f.styled_text(Rect { x: rr.x + 42.0, y: rr.y, width: rr.width - 48.0, height: 46.0 }, mode.label(), p.on_surface, text_style(13.5, 400, 0));
                    if r.clicked {
                        sys.lock_screen_notifs = mode;
                        if mode == LockScreenNotifMode::HideSensitive {
                            sys.show_sensitive_on_lock = false;
                        } else if mode == LockScreenNotifMode::ShowAll {
                            sys.show_sensitive_on_lock = true;
                        }
                        self.dialog = SettingsDialog::None;
                        action = SettingsAction::ShowToast(mode.short_label().into());
                    }
                }
                let cancel_r = Rect { x: card.x + card.width - 96.0, y: card.y + card.height - 48.0, width: 76.0, height: 34.0 };
                if f.target(id_base + 59, cancel_r, "Cancel", true, Role::Button).clicked {
                    self.dialog = SettingsDialog::None;
                }
                f.styled_text(cancel_r, "Cancel", p.primary, text_style(14.0, 500, 1));
            }
            SettingsDialog::None => {}
        }

        if sc < 0.999 {
            f.pop_layer();
            f.pop_transform();
        }

        let over_card = f.pointer_in(card);
        let scrim = f.without_pointer_input(over_card, |f| {
            f.target(id_base, bounds, "Dismiss dialog", true, Role::Button)
        });
        if scrim.clicked {
            self.dialog = SettingsDialog::None;
        }
        action
    }
}

fn settings_fingerprint(s: &SystemState) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut mix = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(0x100000001b3);
    };
    mix(s.dark_theme as u64);
    mix(s.night_light as u64);
    mix(s.auto_brightness as u64);
    mix((s.brightness * 1000.0) as u64);
    mix(s.auto_rotate as u64);
    mix(s.aod as u64);
    mix(s.invert_colors as u64);
    mix(s.accent_idx as u64);
    mix(s.dnd_enabled as u64);
    mix(s.notification_dots as u64);
    mix(s.lock_screen_notifs as u64);
    mix(s.show_sensitive_on_lock as u64);
    mix(s.wake_screen_for_notifs as u64);
    mix(s.hide_silent_in_status_bar as u64);
    mix(s.notification_history_enabled as u64);
    mix(s.battery_saver as u64);
    mix(s.show_battery_pct as u64);
    mix(s.nav_mode as u64);
    mix(s.clock_pos as u64);
    mix(s.double_tap_sleep as u64);
    mix(s.torch_power_longpress as u64);
    mix(s.ringer_mode as u64);
    mix((s.vol_media * 100.0) as u64);
    mix((s.vol_ring * 100.0) as u64);
    mix((s.vol_notif * 100.0) as u64);
    mix((s.vol_alarm * 100.0) as u64);
    mix(s.dns_mode as u64);
    mix(s.anim_scale_idx as u64);
    mix(s.stay_awake as u64);
    mix(s.show_taps as u64);
    mix(s.usb_debugging as u64);
    mix(s.camera_access as u64);
    mix(s.mic_access as u64);
    mix(s.location as u64);
    mix(s.shiraz_enabled as u64);
    mix(s.shiraz_prewarm_memory as u64);
    mix(s.shiraz_aot_enabled as u64);
    mix(s.shiraz_autostart as u64);
    mix(s.shiraz_sandbox as u64);
    h
}
