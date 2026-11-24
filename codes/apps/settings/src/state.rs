//! Shared SpectreOS system, lock-screen, and Freedesktop notification preferences
//! persisted under `~/.config/wisp/` so standalone `spectre-settings` and Wisp stay live-synced.

use bhoo::Color;
use std::time::SystemTime;

pub const ID_SETTINGS_SEARCH: u64 = 0x4C04_0000_0000_F003;
pub const ID_DNS_HOSTNAME: u64 = 0x4C04_0000_0000_F004;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavMode {
    Gestural,
    ThreeButton,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingerMode {
    Vibrate,
    Silent,
    Ring,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DnsMode {
    Off,
    Cloudflare,
    Automatic,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockPos {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockMethod {
    Swipe,
    Pin,
    Password,
}

impl LockMethod {
    pub fn label(self) -> &'static str {
        match self {
            Self::Swipe => "Swipe",
            Self::Pin => "PIN",
            Self::Password => "Password",
        }
    }

    pub fn as_config_str(self) -> &'static str {
        match self {
            Self::Swipe => "swipe",
            Self::Pin => "pin",
            Self::Password => "password",
        }
    }

    pub fn from_config_str(s: &str) -> Self {
        match s.trim() {
            "swipe" => Self::Swipe,
            "password" => Self::Password,
            _ => Self::Pin,
        }
    }
}

/// Android 13 / AOSP Lock Screen notification visibility policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockScreenNotifMode {
    /// Show conversations, alerting, and silent notifications with content.
    ShowAll,
    /// Hide low-urgency / silent notifications on the lock screen.
    HideSilent,
    /// Show notification cards but redact body/title content until unlocked.
    HideSensitive,
    /// Do not display any notification cards on the lock screen.
    DontShow,
}

impl LockScreenNotifMode {
    pub const ALL: &'static [Self] = &[
        Self::ShowAll,
        Self::HideSilent,
        Self::HideSensitive,
        Self::DontShow,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ShowAll => "Show conversations, default, and silent",
            Self::HideSilent => "Hide silent conversations and notifications",
            Self::HideSensitive => "Show notifications, hide sensitive content",
            Self::DontShow => "Don't show any notifications",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::ShowAll => "Show all notifications",
            Self::HideSilent => "Hide silent notifications",
            Self::HideSensitive => "Hide sensitive content",
            Self::DontShow => "Don't show notifications",
        }
    }

    pub fn as_config_str(self) -> &'static str {
        match self {
            Self::ShowAll => "show_all",
            Self::HideSilent => "hide_silent",
            Self::HideSensitive => "hide_sensitive",
            Self::DontShow => "dont_show",
        }
    }

    pub fn from_config_str(s: &str) -> Self {
        match s.trim() {
            "hide_silent" => Self::HideSilent,
            "hide_sensitive" => Self::HideSensitive,
            "dont_show" => Self::DontShow,
            _ => Self::ShowAll,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LockConfig {
    pub method: LockMethod,
    pub secret: String,
    pub auto_lock: bool,
    pub timeout_idx: usize,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            method: LockMethod::Pin,
            secret: "123456".to_string(),
            auto_lock: true,
            timeout_idx: 2, // 1 minute
        }
    }
}

/// Entry stored in the session Notification History log.
#[derive(Clone, Debug)]
pub struct NotificationHistoryEntry {
    pub id: u32,
    pub app: String,
    pub title: String,
    pub body: String,
    pub time_label: String,
    pub dismissed: bool,
    pub urgency: u8,
}

/// Lightweight app descriptor shown in Settings -> Apps & App Info.
#[derive(Clone, Debug)]
pub struct InstalledAppInfo {
    pub id: String,
    pub name: String,
    pub package: String,
    pub version: String,
    pub storage_mb: u32,
    pub builtin_idx: Option<usize>,
    pub desktop_id: Option<String>,
    pub is_android: bool,
}

/// Live lifecycle state of the Shiraz Android compatibility daemon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShirazRuntimeState {
    #[default]
    Disabled,
    Starting,
    Warming,
    Ready,
    Error,
}

impl ShirazRuntimeState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "Off",
            Self::Starting => "Starting…",
            Self::Warming => "Optimizing memory…",
            Self::Ready => "Ready · Warm",
            Self::Error => "Error",
        }
    }

    pub fn from_status_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "starting" => Self::Starting,
            "warming" => Self::Warming,
            "ready" => Self::Ready,
            "error" => Self::Error,
            _ => Self::Disabled,
        }
    }
}

/// Summary of an Android APK registered in Shiraz and exposed as a Freedesktop `.desktop` app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShirazInstalledApp {
    pub package: String,
    pub label: String,
    pub desktop_id: String,
    pub version: String,
    pub aot_compiled: bool,
    pub running: bool,
}

/// Live runtime telemetry for the Shiraz Android subsystem (`$XDG_RUNTIME_DIR/shiraz/status.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShirazRuntimeInfo {
    pub available: bool,
    pub state: ShirazRuntimeState,
    pub daemon_pid: Option<u32>,
    pub warm_ram_mb: u32,
    pub warm_files: u32,
    pub aot_boot_ready: bool,
    pub aot_framework_ready: bool,
    pub last_launch_ms: Option<u32>,
    pub error_message: Option<String>,
    pub installed_apps: Vec<ShirazInstalledApp>,
    pub running_packages: Vec<String>,
}

/// Live device telemetry displayed on Storage, Battery, and About Phone pages.
#[derive(Clone, Debug)]
pub struct DeviceRuntimeInfo {
    pub device_name: String,
    pub model: String,
    pub os_version: String,
    pub kernel: String,
    pub ip_address: String,
    pub wifi_mac: String,
    pub storage_used_gb: f32,
    pub storage_total_gb: f32,
    pub ram_total_gb: f32,
}

impl Default for DeviceRuntimeInfo {
    fn default() -> Self {
        Self::probe()
    }
}

impl DeviceRuntimeInfo {
    pub fn probe() -> Self {
        #[allow(unused_mut)]
        let mut info = Self {
            device_name: "SpectreOS Poco F1".into(),
            model: "Xiaomi Pocophone F1 (beryllium)".into(),
            os_version: "SpectreOS 26.05 · Wisp".into(),
            kernel: "7.1.0-rc1-spectreos".into(),
            ip_address: "172.16.42.1".into(),
            wifi_mac: "—".into(),
            storage_used_gb: 10.9,
            storage_total_gb: 64.0,
            ram_total_gb: 6.0,
        };
        #[cfg(target_os = "linux")]
        {
            if let Ok(host) = std::fs::read_to_string("/etc/hostname") {
                let h = host.trim();
                if !h.is_empty() {
                    info.device_name = h.to_string();
                }
            }
            if let Ok(model) = std::fs::read_to_string("/sys/firmware/devicetree/base/model") {
                let m = model.trim_matches(|c: char| c == '\0' || c.is_whitespace());
                if !m.is_empty() {
                    info.model = m.to_string();
                }
            }
            if let Ok(ver) = std::fs::read_to_string("/proc/version")
                && let Some(k) = ver.split_whitespace().nth(2)
            {
                info.kernel = k.to_string();
            }
            if let Ok(mem) = std::fs::read_to_string("/proc/meminfo") {
                for line in mem.lines() {
                    if let Some(rest) = line.strip_prefix("MemTotal:")
                        && let Some(kb_s) = rest.split_whitespace().next()
                        && let Ok(kb) = kb_s.parse::<f64>()
                    {
                        info.ram_total_gb = (kb / (1024.0 * 1024.0)) as f32;
                    }
                }
            }
            for iface in ["wlan0", "wlp0s20f3", "usb0"] {
                if let Ok(mac) = std::fs::read_to_string(format!("/sys/class/net/{iface}/address")) {
                    let m = mac.trim();
                    if !m.is_empty() && m != "00:00:00:00:00:00" {
                        info.wifi_mac = m.to_string();
                        break;
                    }
                }
            }
            if let Ok(out) = std::process::Command::new("df")
                .args(["-B1", "/"])
                .output()
                && out.status.success()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                if let Some(line) = text.lines().nth(1) {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    if cols.len() >= 4
                        && let (Ok(total), Ok(used)) = (cols[1].parse::<f64>(), cols[2].parse::<f64>())
                        && total > 0.0
                    {
                        info.storage_total_gb = (total / 1_000_000_000.0) as f32;
                        info.storage_used_gb = (used / 1_000_000_000.0) as f32;
                    }
                }
            }
            if let Ok(out) = std::process::Command::new("ip")
                .args(["-4", "-br", "addr"])
                .output()
                && out.status.success()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                for pref in ["wlan0", "tailscale0", "usb0"] {
                    if let Some(line) = text.lines().find(|l| l.starts_with(pref))
                        && let Some(addr) = line.split_whitespace().nth(2)
                    {
                        info.ip_address = addr.split('/').next().unwrap_or(addr).to_string();
                        break;
                    }
                }
            }
        }
        info
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub primary: Color,
    pub primary_dark: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    pub qs_backdrop: Color,
    pub settings_bg: Color,
    pub card_elevated: Color,
    pub card_pure: Color,
    pub unavailable: Color,
    pub surface_container: Color,
    pub on_surface: Color,
    pub on_surface_variant: Color,
    pub outline: Color,
    pub recents_bg: Color,
    pub divider: Color,
    pub dark: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub secured: bool,
    pub connected: bool,
    pub profile: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
    pub connected: bool,
}

#[derive(Clone, Debug)]
pub struct SystemState {
    pub wifi_available: bool,
    pub wifi_enabled: bool,
    pub wifi_networks: Vec<WifiNetwork>,
    pub wifi_error: Option<String>,
    pub bluetooth_devices: Vec<BluetoothDevice>,
    pub bluetooth_error: Option<String>,
    radios_before_airplane: Option<(bool, bool)>,
    pub wifi_connected: bool,
    pub wifi_signal: u8,
    pub wifi_name: String,
    pub wifi_selected: usize,
    pub bluetooth_enabled: bool,
    pub bluetooth_available: bool,
    pub bluetooth_device: Option<String>,
    pub flashlight_on: bool,
    pub flashlight_available: bool,
    pub ambient_lux: Option<f32>,
    pub haptic_enabled: bool,
    pub touch_id_available: bool,
    pub touch_id_enrolled: usize,
    pub airplane_mode: bool,
    pub auto_rotate: bool,
    pub battery_saver: bool,
    pub dark_theme: bool,
    pub night_light: bool,
    pub dnd_enabled: bool,
    pub hotspot: bool,
    pub nfc: bool,
    pub location: bool,
    pub camera_access: bool,
    pub mic_access: bool,
    pub aod: bool,
    pub screen_cast: bool,
    pub screen_record: bool,
    pub invert_colors: bool,
    pub auto_brightness: bool,
    pub brightness: f32,
    pub vol_media: f32,
    pub vol_ring: f32,
    pub vol_notif: f32,
    pub vol_alarm: f32,
    pub ringer_mode: RingerMode,
    pub battery_level: u8,
    pub charging: bool,
    pub dns_mode: DnsMode,
    pub dns_host: String,
    pub screen_timeout_idx: usize,
    pub nav_mode: NavMode,
    pub grid_idx: usize,
    pub themed_icons: bool,
    pub themed_drawer_icons: bool,
    pub show_home_labels: bool,
    pub show_drawer_labels: bool,
    pub lock_layout: bool,
    pub notification_dots: bool,
    pub lock_screen_notifs: LockScreenNotifMode,
    pub show_sensitive_on_lock: bool,
    pub wake_screen_for_notifs: bool,
    pub hide_silent_in_status_bar: bool,
    pub notification_history_enabled: bool,
    pub home_rotation: bool,
    pub add_new_icons: bool,
    pub accent_idx: usize,
    pub accent_rgb: Option<[u8; 3]>,
    pub wallpaper_color_slot: Option<usize>,
    pub lock_shortcuts: [usize; 2],
    pub icon_style: usize,
    pub icon_shape: usize,
    pub font_style: usize,
    pub wallpaper_idx: usize,
    pub show_clock_widget: bool,
    pub show_weather_widget: bool,
    pub show_battery_pct: bool,
    pub clock_pos: ClockPos,
    pub double_tap_sleep: bool,
    pub torch_power_longpress: bool,
    pub usb_debugging: bool,
    pub stay_awake: bool,
    pub show_taps: bool,
    pub anim_scale_idx: usize,
    pub data_saver: bool,
    pub auto_sync: bool,
    pub dev_unlocked: bool,
    pub tap_build_count: u8,
    pub alarm_summary: Option<String>,
    pub hidden_apps: Vec<String>,
    pub lock_method: LockMethod,
    pub lock_secret: String,
    pub auto_lock_enabled: bool,
    pub shiraz_enabled: bool,
    pub shiraz_prewarm_memory: bool,
    pub shiraz_aot_enabled: bool,
    pub shiraz_autostart: bool,
    pub shiraz_sandbox: bool,
    pub shiraz: ShirazRuntimeInfo,
    pub last_settings_mtime: Option<SystemTime>,
    pub last_lock_mtime: Option<SystemTime>,
}

impl Default for SystemState {
    fn default() -> Self {
        let lock_cfg = load_lock_config();
        let mut s = Self {
            wifi_available: false,
            wifi_enabled: false,
            wifi_networks: Vec::new(),
            wifi_error: None,
            bluetooth_devices: Vec::new(),
            bluetooth_error: None,
            radios_before_airplane: None,
            wifi_connected: false,
            wifi_signal: 0,
            wifi_name: "Wi-Fi".into(),
            wifi_selected: 0,
            bluetooth_enabled: false,
            bluetooth_available: true,
            bluetooth_device: None,
            flashlight_on: false,
            flashlight_available: true,
            ambient_lux: None,
            haptic_enabled: true,
            touch_id_available: false,
            touch_id_enrolled: 0,
            airplane_mode: false,
            auto_rotate: false,
            battery_saver: false,
            dark_theme: false,
            night_light: false,
            dnd_enabled: false,
            hotspot: false,
            nfc: true,
            location: true,
            camera_access: true,
            mic_access: true,
            aod: true,
            screen_cast: false,
            screen_record: false,
            invert_colors: false,
            auto_brightness: true,
            brightness: 0.72,
            vol_media: 0.76,
            vol_ring: 0.25,
            vol_notif: 0.22,
            vol_alarm: 0.48,
            ringer_mode: RingerMode::Vibrate,
            battery_level: 100,
            charging: true,
            dns_mode: DnsMode::Automatic,
            dns_host: "dns.quad9.net".into(),
            screen_timeout_idx: lock_cfg.timeout_idx,
            nav_mode: NavMode::Gestural,
            grid_idx: 0,
            themed_icons: true,
            themed_drawer_icons: false,
            show_home_labels: true,
            show_drawer_labels: true,
            lock_layout: false,
            notification_dots: true,
            lock_screen_notifs: LockScreenNotifMode::ShowAll,
            show_sensitive_on_lock: true,
            wake_screen_for_notifs: true,
            hide_silent_in_status_bar: false,
            notification_history_enabled: true,
            home_rotation: false,
            add_new_icons: true,
            accent_idx: 0,
            accent_rgb: None,
            wallpaper_color_slot: None,
            lock_shortcuts: [5, 3],
            icon_style: 0,
            icon_shape: 0,
            font_style: 0,
            wallpaper_idx: 0,
            show_clock_widget: false,
            show_weather_widget: false,
            show_battery_pct: true,
            clock_pos: ClockPos::Left,
            double_tap_sleep: true,
            torch_power_longpress: true,
            usb_debugging: true,
            stay_awake: false,
            show_taps: false,
            anim_scale_idx: 2,
            data_saver: false,
            auto_sync: true,
            dev_unlocked: true,
            tap_build_count: 0,
            alarm_summary: None,
            hidden_apps: Vec::new(),
            lock_method: lock_cfg.method,
            lock_secret: lock_cfg.secret,
            auto_lock_enabled: lock_cfg.auto_lock,
            shiraz_enabled: false,
            shiraz_prewarm_memory: true,
            shiraz_aot_enabled: true,
            shiraz_autostart: true,
            shiraz_sandbox: false,
            shiraz: load_shiraz_runtime_info(),
            last_settings_mtime: None,
            last_lock_mtime: None,
        };
        s.load_persisted_settings();
        s
    }
}

impl SystemState {
    pub fn set_airplane_mode(&mut self, enabled: bool) {
        if enabled == self.airplane_mode { return; }
        self.airplane_mode = enabled;
        if enabled {
            self.radios_before_airplane = Some((self.wifi_enabled, self.bluetooth_enabled));
            self.wifi_enabled = false;
            self.bluetooth_enabled = false;
        } else if let Some((wifi, bluetooth)) = self.radios_before_airplane.take() {
            self.wifi_enabled |= wifi;
            self.bluetooth_enabled |= bluetooth;
        }
    }

    pub fn palette(&self) -> Palette {
        let mut palette = if self.dark_theme {
            let (primary, primary_dark, on_primary, primary_container, on_primary_container) =
                match self.accent_idx {
                    1 => (
                        Color(0x8F, 0xD8, 0xAE, 255),
                        Color(0x23, 0x4C, 0x35, 255),
                        Color(0x09, 0x38, 0x20, 255),
                        Color(0x1F, 0x4E, 0x34, 255),
                        Color(0xC2, 0xF3, 0xD5, 255),
                    ),
                    2 => (
                        Color(0xCC, 0xC0, 0xFF, 255),
                        Color(0x46, 0x3B, 0x73, 255),
                        Color(0x2E, 0x22, 0x5A, 255),
                        Color(0x49, 0x3E, 0x76, 255),
                        Color(0xE8, 0xE0, 0xFF, 255),
                    ),
                    3 => (
                        Color(0xF6, 0xBE, 0x74, 255),
                        Color(0x63, 0x43, 0x16, 255),
                        Color(0x48, 0x2A, 0x00, 255),
                        Color(0x66, 0x44, 0x14, 255),
                        Color(0xFF, 0xE0, 0xB2, 255),
                    ),
                    _ => (
                        Color(0xA8, 0xC7, 0xFA, 255),
                        Color(0x2E, 0x4A, 0x7D, 255),
                        Color(0x0B, 0x2D, 0x63, 255),
                        Color(0x2B, 0x44, 0x70, 255),
                        Color(0xD8, 0xE2, 0xFF, 255),
                    ),
                };
            Palette {
                primary,
                primary_dark,
                on_primary,
                primary_container,
                on_primary_container,
                qs_backdrop: Color(0x14, 0x15, 0x19, 255),
                settings_bg: Color(0x1B, 0x1B, 0x1F, 255),
                card_elevated: Color(0x26, 0x28, 0x2E, 255),
                card_pure: Color(0x2E, 0x30, 0x38, 255),
                unavailable: Color(0x20, 0x22, 0x27, 255),
                surface_container: Color(0x2B, 0x2D, 0x35, 255),
                on_surface: Color(0xE4, 0xE2, 0xE6, 255),
                on_surface_variant: Color(0xA9, 0xAB, 0xB5, 255),
                outline: Color(0x4E, 0x50, 0x5B, 255),
                recents_bg: Color(0x19, 0x1B, 0x20, 255),
                divider: Color(0x34, 0x36, 0x40, 255),
                dark: true,
            }
        } else {
            let (primary, primary_dark, primary_container, on_primary_container) =
                match self.accent_idx {
                    1 => (
                        Color(0x38, 0x6A, 0x4D, 255),
                        Color(0x25, 0x4E, 0x36, 255),
                        Color(0xCA, 0xF0, 0xD8, 255),
                        Color(0x12, 0x38, 0x23, 255),
                    ),
                    2 => (
                        Color(0x65, 0x5A, 0x95, 255),
                        Color(0x4A, 0x3F, 0x76, 255),
                        Color(0xE7, 0xDE, 0xFF, 255),
                        Color(0x26, 0x1D, 0x4F, 255),
                    ),
                    3 => (
                        Color(0x8C, 0x62, 0x26, 255),
                        Color(0x69, 0x47, 0x16, 255),
                        Color(0xFF, 0xDF, 0xAE, 255),
                        Color(0x3C, 0x24, 0x04, 255),
                    ),
                    _ => (
                        Color(0x43, 0x5E, 0x91, 255),
                        Color(0x2E, 0x4A, 0x7D, 255),
                        Color(0xD8, 0xE2, 0xFF, 255),
                        Color(0x1B, 0x2B, 0x4C, 255),
                    ),
                };
            Palette {
                primary,
                primary_dark,
                on_primary: Color(0xFF, 0xFF, 0xFF, 255),
                primary_container,
                on_primary_container,
                qs_backdrop: Color(0xE3, 0xE2, 0xE6, 255),
                settings_bg: Color(0xF2, 0xF0, 0xF4, 255),
                card_elevated: Color(0xFA, 0xF9, 0xFC, 255),
                card_pure: Color(0xFF, 0xFF, 0xFF, 255),
                unavailable: Color(0xEA, 0xE9, 0xEC, 255),
                surface_container: Color(0xE7, 0xE6, 0xEC, 255),
                on_surface: Color(0x1B, 0x1B, 0x1F, 255),
                on_surface_variant: Color(0x5F, 0x63, 0x68, 255),
                outline: Color(0xC2, 0xC4, 0xCE, 255),
                recents_bg: Color(0xC5, 0xC8, 0xD2, 255),
                divider: Color(0xDC, 0xDB, 0xE1, 255),
                dark: false,
            }
        };
        if let Some([r, g, b]) = self.accent_rgb {
            let blend = |level: u8, weight: f32| {
                let channel = |v| (v as f32 * weight + level as f32 * (1.0 - weight)).round() as u8;
                Color(channel(r), channel(g), channel(b), 255)
            };
            palette.primary = if self.dark_theme { blend(255, 0.47) } else { blend(0, 0.48) };
            palette.primary_dark = blend(0, 0.41);
            palette.on_primary = if self.dark_theme { blend(0, 0.24) } else { Color(255, 255, 255, 255) };
            palette.primary_container = if self.dark_theme { blend(0, 0.42) } else { blend(255, 0.26) };
            palette.on_primary_container = if self.dark_theme { blend(255, 0.28) } else { blend(0, 0.31) };
        }
        palette
    }

    pub fn grid_dims(&self) -> (usize, usize) {
        match self.grid_idx {
            1 => (4, 5),
            2 => (4, 4),
            3 => (3, 3),
            4 => (2, 2),
            5 => (5, 6),
            _ => (5, 5),
        }
    }

    pub const ANIMATION_SCALES: [f32; 7] = [0.0, 0.5, 1.0, 1.5, 2.0, 5.0, 10.0];

    pub fn anim_scale(&self) -> f32 {
        Self::ANIMATION_SCALES.get(self.anim_scale_idx).copied().unwrap_or(1.0)
    }

    /// Persist user-modifiable settings cleanly to `~/.config/wisp/settings.conf` (atomic rename).
    pub fn save_persisted_settings(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(path) = settings_config_path() else {
                return;
            };
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let nav_str = match self.nav_mode {
                NavMode::Gestural => "gestural",
                NavMode::ThreeButton => "three_button",
            };
            let clock_str = match self.clock_pos {
                ClockPos::Left => "left",
                ClockPos::Center => "center",
                ClockPos::Right => "right",
            };
            let ringer_str = match self.ringer_mode {
                RingerMode::Vibrate => "vibrate",
                RingerMode::Silent => "silent",
                RingerMode::Ring => "ring",
            };
            let dns_str = match self.dns_mode {
                DnsMode::Off => "off",
                DnsMode::Cloudflare => "cloudflare",
                DnsMode::Automatic => "automatic",
                DnsMode::Custom => "custom",
            };
            let body = format!(
                concat!(
                    "dark_theme={}\n",
                    "night_light={}\n",
                    "auto_brightness={}\n",
                    "brightness={:.3}\n",
                    "auto_rotate={}\n",
                    "aod={}\n",
                    "invert_colors={}\n",
                    "accent_idx={}\n",
                    "accent_rgb={}\n",
                    "wallpaper_color_slot={}\n",
                    "grid_idx={}\n",
                    "themed_icons={}\n",
                    "lock_shortcuts={},{}\n",
                    "icon_style={}\n",
                    "icon_shape={}\n",
                    "font_style={}\n",
                    "dnd_enabled={}\n",
                    "notification_dots={}\n",
                    "lock_screen_notifs={}\n",
                    "show_sensitive_on_lock={}\n",
                    "wake_screen_for_notifs={}\n",
                    "hide_silent_in_status_bar={}\n",
                    "notification_history_enabled={}\n",
                    "battery_saver={}\n",
                    "show_battery_pct={}\n",
                    "nav_mode={}\n",
                    "clock_pos={}\n",
                    "double_tap_sleep={}\n",
                    "torch_power_longpress={}\n",
                    "haptic_enabled={}\n",
                    "ringer_mode={}\n",
                    "vol_media={:.3}\n",
                    "vol_ring={:.3}\n",
                    "vol_notif={:.3}\n",
                    "vol_alarm={:.3}\n",
                    "dns_mode={}\n",
                    "dns_host={}\n",
                    "anim_scale_idx={}\n",
                    "stay_awake={}\n",
                    "show_taps={}\n",
                    "usb_debugging={}\n",
                    "camera_access={}\n",
                    "mic_access={}\n",
                    "location={}\n",
                    "shiraz_enabled={}\n",
                    "shiraz_prewarm_memory={}\n",
                    "shiraz_aot_enabled={}\n",
                    "shiraz_autostart={}\n",
                    "shiraz_sandbox={}\n",
                ),
                self.dark_theme,
                self.night_light,
                self.auto_brightness,
                self.brightness,
                self.auto_rotate,
                self.aod,
                self.invert_colors,
                self.accent_idx,
                self.accent_rgb.map_or(String::new(), |[r, g, b]| format!("{r:02x}{g:02x}{b:02x}")),
                self.wallpaper_color_slot.map_or(String::new(), |slot| slot.to_string()),
                self.grid_idx, self.themed_icons, self.lock_shortcuts[0], self.lock_shortcuts[1],
                self.icon_style, self.icon_shape, self.font_style,
                self.dnd_enabled,
                self.notification_dots,
                self.lock_screen_notifs.as_config_str(),
                self.show_sensitive_on_lock,
                self.wake_screen_for_notifs,
                self.hide_silent_in_status_bar,
                self.notification_history_enabled,
                self.battery_saver,
                self.show_battery_pct,
                nav_str,
                clock_str,
                self.double_tap_sleep,
                self.torch_power_longpress,
                self.haptic_enabled,
                ringer_str,
                self.vol_media,
                self.vol_ring,
                self.vol_notif,
                self.vol_alarm,
                dns_str,
                self.dns_host,
                self.anim_scale_idx,
                self.stay_awake,
                self.show_taps,
                self.usb_debugging,
                self.camera_access,
                self.mic_access,
                self.location,
                self.shiraz_enabled,
                self.shiraz_prewarm_memory,
                self.shiraz_aot_enabled,
                self.shiraz_autostart,
                self.shiraz_sandbox,
            );
            let tmp = path.with_extension("conf.tmp");
            if std::fs::write(&tmp, body).is_ok() && std::fs::rename(&tmp, &path).is_ok() {
                self.last_settings_mtime = std::fs::metadata(&path).ok().and_then(|m| m.modified().ok());
            }
        }
    }

    pub fn load_persisted_settings(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = settings_config_path()
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                self.last_settings_mtime =
                    std::fs::metadata(&path).ok().and_then(|m| m.modified().ok());
                for line in text.lines() {
                    let Some((k, v)) = line.split_once('=') else {
                        continue;
                    };
                    let v = v.trim();
                    let b = v == "true" || v == "1";
                    match k.trim() {
                        "dark_theme" => self.dark_theme = b,
                        "night_light" => self.night_light = b,
                        "auto_brightness" => self.auto_brightness = b,
                        "brightness" => {
                            if let Ok(f) = v.parse::<f32>() {
                                self.brightness = f.clamp(0.15, 1.0);
                            }
                        }
                        "auto_rotate" => self.auto_rotate = b,
                        "aod" => self.aod = b,
                        "invert_colors" => self.invert_colors = b,
                        "accent_idx" => {
                            if let Ok(i) = v.parse::<usize>() {
                                self.accent_idx = i.min(3);
                            }
                        }
                        "accent_rgb" => {
                            self.accent_rgb = u32::from_str_radix(v, 16).ok().filter(|value| *value <= 0xffffff)
                                .map(|value| [(value >> 16) as u8, (value >> 8) as u8, value as u8]);
                        }
                        "wallpaper_color_slot" => self.wallpaper_color_slot = v.parse::<usize>().ok().filter(|slot| *slot < 4),
                        "grid_idx" => if let Ok(value) = v.parse::<usize>() { self.grid_idx = value.min(5); },
                        "themed_icons" => self.themed_icons = b,
                        "lock_shortcuts" => if let Some((left, right)) = v.split_once(',')
                            && let (Ok(left), Ok(right)) = (left.parse::<usize>(), right.parse::<usize>())
                        { self.lock_shortcuts = [left.min(6), right.min(6)]; },
                        "icon_style" => if let Ok(value) = v.parse::<usize>() { self.icon_style = value.min(6); },
                        "icon_shape" => if let Ok(value) = v.parse::<usize>() { self.icon_shape = value.min(6); },
                        "font_style" => if let Ok(value) = v.parse::<usize>() { self.font_style = value.min(3); },
                        "dnd_enabled" => self.dnd_enabled = b,
                        "notification_dots" => self.notification_dots = b,
                        "lock_screen_notifs" => {
                            self.lock_screen_notifs = LockScreenNotifMode::from_config_str(v);
                        }
                        "show_sensitive_on_lock" => self.show_sensitive_on_lock = b,
                        "wake_screen_for_notifs" => self.wake_screen_for_notifs = b,
                        "hide_silent_in_status_bar" => self.hide_silent_in_status_bar = b,
                        "notification_history_enabled" => self.notification_history_enabled = b,
                        "battery_saver" => self.battery_saver = b,
                        "show_battery_pct" => self.show_battery_pct = b,
                        "nav_mode" => {
                            self.nav_mode = if v == "three_button" {
                                NavMode::ThreeButton
                            } else {
                                NavMode::Gestural
                            };
                        }
                        "clock_pos" => {
                            self.clock_pos = match v {
                                "center" => ClockPos::Center,
                                "right" => ClockPos::Right,
                                _ => ClockPos::Left,
                            };
                        }
                        "double_tap_sleep" => self.double_tap_sleep = b,
                        "torch_power_longpress" => self.torch_power_longpress = b,
                        "haptic_enabled" => self.haptic_enabled = b,
                        "ringer_mode" => {
                            self.ringer_mode = match v {
                                "silent" => RingerMode::Silent,
                                "ring" => RingerMode::Ring,
                                _ => RingerMode::Vibrate,
                            };
                        }
                        "vol_media" => {
                            if let Ok(f) = v.parse::<f32>() {
                                self.vol_media = f.clamp(0.0, 1.0);
                            }
                        }
                        "vol_ring" => {
                            if let Ok(f) = v.parse::<f32>() {
                                self.vol_ring = f.clamp(0.0, 1.0);
                            }
                        }
                        "vol_notif" => {
                            if let Ok(f) = v.parse::<f32>() {
                                self.vol_notif = f.clamp(0.0, 1.0);
                            }
                        }
                        "vol_alarm" => {
                            if let Ok(f) = v.parse::<f32>() {
                                self.vol_alarm = f.clamp(0.0, 1.0);
                            }
                        }
                        "dns_mode" => {
                            self.dns_mode = match v {
                                "off" => DnsMode::Off,
                                "cloudflare" => DnsMode::Cloudflare,
                                "custom" => DnsMode::Custom,
                                _ => DnsMode::Automatic,
                            };
                        }
                        "dns_host" if !v.is_empty() => self.dns_host = v.to_string(),
                        "anim_scale_idx" => {
                            if let Ok(i) = v.parse::<usize>() {
                                self.anim_scale_idx = i.min(Self::ANIMATION_SCALES.len() - 1);
                            }
                        }
                        "stay_awake" => self.stay_awake = b,
                        "show_taps" => self.show_taps = b,
                        "usb_debugging" => self.usb_debugging = b,
                        "camera_access" => self.camera_access = b,
                        "mic_access" => self.mic_access = b,
                        "location" => self.location = b,
                        "shiraz_enabled" => self.shiraz_enabled = b,
                        "shiraz_prewarm_memory" => self.shiraz_prewarm_memory = b,
                        "shiraz_aot_enabled" => self.shiraz_aot_enabled = b,
                        "shiraz_autostart" => self.shiraz_autostart = b,
                        "shiraz_sandbox" => self.shiraz_sandbox = b,
                        _ => {}
                    }
                }
            }
            if let Some(lpath) = lock_config_path() {
                self.last_lock_mtime =
                    std::fs::metadata(&lpath).ok().and_then(|m| m.modified().ok());
            }
        }
    }

    /// Hot-reload settings & lock config if changed externally (e.g. by standalone `spectre-settings`).
    pub fn reload_persisted_if_changed(&mut self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut changed = false;
            if let Some(path) = settings_config_path()
                && let Ok(mt) = std::fs::metadata(&path).and_then(|m| m.modified())
                && self.last_settings_mtime != Some(mt)
            {
                self.load_persisted_settings();
                changed = true;
            }
            if let Some(lpath) = lock_config_path()
                && let Ok(mt) = std::fs::metadata(&lpath).and_then(|m| m.modified())
                && self.last_lock_mtime != Some(mt)
            {
                let cfg = load_lock_config();
                self.lock_method = cfg.method;
                self.lock_secret = cfg.secret;
                self.auto_lock_enabled = cfg.auto_lock;
                self.screen_timeout_idx = cfg.timeout_idx;
                self.last_lock_mtime = Some(mt);
                changed = true;
            }
            changed
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn xdg_config_home() -> Option<std::path::PathBuf> {
    if let Ok(v) = std::env::var("XDG_CONFIG_HOME") {
        let p = std::path::PathBuf::from(&v);
        if !v.is_empty() && p.is_absolute() {
            return Some(p);
        }
    }
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn xdg_data_home() -> Option<std::path::PathBuf> {
    if let Ok(v) = std::env::var("XDG_DATA_HOME") {
        let p = std::path::PathBuf::from(&v);
        if !v.is_empty() && p.is_absolute() {
            return Some(p);
        }
    }
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn xdg_runtime_dir() -> Option<std::path::PathBuf> {
    if let Ok(v) = std::env::var("XDG_RUNTIME_DIR") {
        let p = std::path::PathBuf::from(&v);
        if !v.is_empty() && p.is_absolute() {
            return Some(p);
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn settings_config_path() -> Option<std::path::PathBuf> {
    xdg_config_home().map(|c| c.join("wisp/settings.conf"))
}

#[cfg(not(target_arch = "wasm32"))]
fn lock_config_path() -> Option<std::path::PathBuf> {
    xdg_config_home().map(|c| c.join("wisp/lock.conf"))
}

/// Probe Shiraz runtime availability, live daemon telemetry (`$XDG_RUNTIME_DIR/shiraz/status.json`),
/// and installed Freedesktop `.Shiraz.desktop` Android applications.
pub fn load_shiraz_runtime_info() -> ShirazRuntimeInfo {
    #[cfg(any(target_arch = "wasm32", test))]
    {
        ShirazRuntimeInfo::default()
    }
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    {
        let mut info = ShirazRuntimeInfo::default();
        info.available = detect_shiraz_root().is_some();

        // 1. Read live daemon telemetry from `$XDG_RUNTIME_DIR/shiraz/status.json` (XDG BaseDir v0.8)
        if let Some(rt) = xdg_runtime_dir() {
            let status_path = rt.join("shiraz/status.json");
            if let Ok(text) = std::fs::read_to_string(&status_path) {
                parse_shiraz_status_json(&text, &mut info);
                // Guard against stale status files if daemon exited unexpectedly
                if let Some(pid) = info.daemon_pid
                    && info.state != ShirazRuntimeState::Disabled
                    && !std::path::Path::new(&format!("/proc/{pid}")).exists()
                {
                    info.state = ShirazRuntimeState::Disabled;
                    info.daemon_pid = None;
                    info.warm_ram_mb = 0;
                    info.warm_files = 0;
                    info.running_packages.clear();
                }
            }
        }

        // 2. Always discover installed Freedesktop `.Shiraz.desktop` apps so Settings shows them even when off
        let desktop_apps = scan_shiraz_desktop_apps(&info.running_packages);
        if info.installed_apps.is_empty() {
            info.installed_apps = desktop_apps;
        } else {
            for dapp in desktop_apps {
                if !info.installed_apps.iter().any(|a| a.package == dapp.package) {
                    info.installed_apps.push(dapp);
                }
            }
        }
        info.installed_apps
            .sort_by(|a, b| a.label.to_ascii_lowercase().cmp(&b.label.to_ascii_lowercase()));
        info
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn detect_shiraz_root() -> Option<std::path::PathBuf> {
    if let Ok(env_root) = std::env::var("SHIRAZ_HOME") {
        let p = std::path::PathBuf::from(env_root);
        if p.join("src/shiraz/spectre_daemon.py").is_file() || p.join("src/shiraz/cli.py").is_file()
        {
            return Some(p);
        }
    }
    let mut candidates = vec![
        std::path::PathBuf::from("/opt/shiraz"),
        std::path::PathBuf::from("/usr/share/shiraz"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        let h = std::path::PathBuf::from(home);
        candidates.push(h.join("spectreos/codes/shiraz-core"));
        candidates.push(h.join("shiraz/codes/shiraz"));
        candidates.push(h.join(".local/share/shiraz/runtime"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("codes/shiraz-core"));
    }
    candidates.into_iter().find(|p| {
        p.join("src/shiraz/spectre_daemon.py").is_file() || p.join("src/shiraz/cli.py").is_file()
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn scan_shiraz_desktop_apps(running: &[String]) -> Vec<ShirazInstalledApp> {
    let mut apps = Vec::new();
    let Some(data) = xdg_data_home() else {
        return apps;
    };
    let dir = data.join("applications");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return apps;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(fname) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !fname.ends_with(".desktop") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut pkg: Option<String> = None;
        let mut name: Option<String> = None;
        let mut version = String::from("Android");
        let mut aot = false;
        let mut in_main = false;
        for raw in text.lines() {
            let line = raw.trim();
            if line.starts_with('[') && line.ends_with(']') {
                in_main = line == "[Desktop Entry]";
                continue;
            }
            if !in_main {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "X-Shiraz-Package" => pkg = Some(v.trim().to_string()),
                    "Name" => name = Some(v.trim().to_string()),
                    "X-Shiraz-Version" => version = v.trim().to_string(),
                    "X-Shiraz-AOT" => aot = v.trim() == "true" || v.trim() == "1",
                    _ => {}
                }
            }
        }
        if let Some(package) = pkg {
            let is_running = running.iter().any(|p| p == &package);
            apps.push(ShirazInstalledApp {
                label: name.unwrap_or_else(|| package.clone()),
                desktop_id: fname.to_string(),
                version,
                aot_compiled: aot,
                running: is_running,
                package,
            });
        }
    }
    apps
}

#[cfg(not(target_arch = "wasm32"))]
pub fn parse_shiraz_status_json(text: &str, info: &mut ShirazRuntimeInfo) {
    if let Some(st) = json_extract_str(text, "state") {
        info.state = ShirazRuntimeState::from_status_str(&st);
    }
    if let Some(avail) = json_extract_bool(text, "available") {
        info.available = avail;
    }
    info.daemon_pid = json_extract_u32(text, "pid");
    if let Some(mb) = json_extract_u32(text, "warm_ram_mb") {
        info.warm_ram_mb = mb;
    }
    if let Some(wf) = json_extract_u32(text, "warm_files") {
        info.warm_files = wf;
    }
    if let Some(b) = json_extract_bool(text, "aot_boot_ready") {
        info.aot_boot_ready = b;
    }
    if let Some(b) = json_extract_bool(text, "aot_framework_ready") {
        info.aot_framework_ready = b;
    }
    info.last_launch_ms = json_extract_u32(text, "last_launch_ms");
    info.error_message = json_extract_str(text, "error").filter(|s| !s.is_empty());
    info.running_packages = json_extract_string_array(text, "running_packages");
    let parsed_apps = json_extract_installed_apps(text, &info.running_packages);
    if !parsed_apps.is_empty() {
        info.installed_apps = parsed_apps;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn json_extract_str(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let pos = json.find(&needle)?;
    let rest = &json[pos + needle.len()..];
    let colon = rest.find(':')?;
    let val = rest[colon + 1..].trim_start();
    if !val.starts_with('"') {
        return None;
    }
    let mut out = String::new();
    let mut chars = val[1..].chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(esc) = chars.next() {
                out.push(esc);
            }
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn json_extract_u32(json: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\"");
    let pos = json.find(&needle)?;
    let rest = &json[pos + needle.len()..];
    let colon = rest.find(':')?;
    let val = rest[colon + 1..].trim_start();
    let digits: String = val.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn json_extract_bool(json: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let pos = json.find(&needle)?;
    let rest = &json[pos + needle.len()..];
    let colon = rest.find(':')?;
    let val = rest[colon + 1..].trim_start();
    if val.starts_with("true") {
        Some(true)
    } else if val.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn json_extract_string_array(json: &str, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    let needle = format!("\"{key}\"");
    let Some(pos) = json.find(&needle) else {
        return out;
    };
    let rest = &json[pos + needle.len()..];
    let Some(lb) = rest.find('[') else {
        return out;
    };
    let Some(rb) = rest[lb..].find(']') else {
        return out;
    };
    let slice = &rest[lb + 1..lb + rb];
    let mut chars = slice.chars();
    while let Some(c) = chars.next() {
        if c == '"' {
            let mut s = String::new();
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    if let Some(esc) = chars.next() {
                        s.push(esc);
                    }
                } else if ch == '"' {
                    break;
                } else {
                    s.push(ch);
                }
            }
            if !s.is_empty() {
                out.push(s);
            }
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn json_extract_installed_apps(json: &str, running: &[String]) -> Vec<ShirazInstalledApp> {
    let mut out = Vec::new();
    let needle = "\"installed_apps\"";
    let Some(pos) = json.find(needle) else {
        return out;
    };
    let rest = &json[pos + needle.len()..];
    let Some(lb) = rest.find('[') else {
        return out;
    };
    let bytes = rest.as_bytes();
    let mut i = lb + 1;
    while i < bytes.len() {
        if bytes[i] == b']' {
            break;
        }
        if bytes[i] == b'{' {
            let start = i;
            let mut depth = 0i32;
            while i < bytes.len() {
                if bytes[i] == b'{' {
                    depth += 1;
                } else if bytes[i] == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        let obj = &rest[start..=i];
                        if let Some(package) = json_extract_str(obj, "package") {
                            let label =
                                json_extract_str(obj, "label").unwrap_or_else(|| package.clone());
                            let desktop_id = json_extract_str(obj, "desktop_id")
                                .unwrap_or_else(|| format!("{package}.Shiraz.desktop"));
                            let version = json_extract_str(obj, "version")
                                .unwrap_or_else(|| "Android".into());
                            let aot = json_extract_bool(obj, "aot_compiled").unwrap_or(false);
                            let is_run = json_extract_bool(obj, "running")
                                .unwrap_or_else(|| running.iter().any(|p| p == &package));
                            out.push(ShirazInstalledApp {
                                package,
                                label,
                                desktop_id,
                                version,
                                aot_compiled: aot,
                                running: is_run,
                            });
                        }
                        break;
                    }
                }
                i += 1;
            }
        }
        i += 1;
    }
    out
}

/// Load persisted screen lock configuration from `~/.config/wisp/lock.conf` (native Linux).
pub fn load_lock_config() -> LockConfig {
    let mut cfg = LockConfig::default();
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = lock_config_path()
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            for line in text.lines() {
                if let Some(v) = line.strip_prefix("method=") {
                    cfg.method = LockMethod::from_config_str(v);
                } else if let Some(v) = line.strip_prefix("secret=") {
                    cfg.secret = v.trim().to_string();
                } else if let Some(v) = line.strip_prefix("auto_lock=") {
                    cfg.auto_lock = v.trim() != "false" && v.trim() != "0";
                } else if let Some(v) = line.strip_prefix("timeout_idx=")
                    && let Ok(idx) = v.trim().parse::<usize>()
                {
                    cfg.timeout_idx = idx.min(5);
                }
            }
        }
    }
    cfg
}

/// Persist screen lock configuration to `~/.config/wisp/lock.conf` (native Linux).
pub fn save_lock_config(method: LockMethod, secret: &str, auto_lock: bool, timeout_idx: usize) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(path) = lock_config_path() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let body = format!(
                "method={}\nsecret={}\nauto_lock={}\ntimeout_idx={}\n",
                method.as_config_str(),
                secret,
                auto_lock,
                timeout_idx.min(5)
            );
            let tmp = path.with_extension("conf.tmp");
            if std::fs::write(&tmp, body).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (method, secret, auto_lock, timeout_idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_shiraz_status_json_fields_and_apps() {
        let json = r#"{
            "available": true,
            "state": "ready",
            "pid": 4242,
            "warm_ram_mb": 148,
            "warm_files": 23,
            "aot_boot_ready": true,
            "aot_framework_ready": true,
            "last_launch_ms": 612,
            "error": "",
            "running_packages": ["net.osmand.plus"],
            "installed_apps": [
                {
                    "package": "net.osmand.plus",
                    "label": "OsmAnd~",
                    "desktop_id": "net.osmand.plus.Shiraz.desktop",
                    "version": "5.4.1",
                    "aot_compiled": true,
                    "running": true
                },
                {
                    "package": "chat.delta",
                    "label": "Delta Chat",
                    "desktop_id": "chat.delta.Shiraz.desktop",
                    "version": "2.62.0",
                    "aot_compiled": false,
                    "running": false
                }
            ]
        }"#;
        let mut info = ShirazRuntimeInfo::default();
        parse_shiraz_status_json(json, &mut info);
        assert!(info.available);
        assert_eq!(info.state, ShirazRuntimeState::Ready);
        assert_eq!(info.daemon_pid, Some(4242));
        assert_eq!(info.warm_ram_mb, 148);
        assert_eq!(info.warm_files, 23);
        assert!(info.aot_boot_ready);
        assert!(info.aot_framework_ready);
        assert_eq!(info.last_launch_ms, Some(612));
        assert!(info.error_message.is_none());
        assert_eq!(info.running_packages, vec!["net.osmand.plus"]);
        assert_eq!(info.installed_apps.len(), 2);
        assert_eq!(info.installed_apps[0].label, "OsmAnd~");
        assert!(info.installed_apps[0].aot_compiled);
        assert!(info.installed_apps[0].running);
        assert_eq!(info.installed_apps[1].package, "chat.delta");
        assert!(!info.installed_apps[1].running);
    }

    #[test]
    fn shiraz_settings_persist_in_xdg_config_home() {
        let tmp = std::env::temp_dir().join(format!(
            "spectre-settings-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        // SAFETY: unit test runs isolated in this crate
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", &tmp);
        }
        let mut st = SystemState::default();
        st.shiraz_enabled = true;
        st.shiraz_prewarm_memory = false;
        st.shiraz_aot_enabled = true;
        st.shiraz_autostart = false;
        st.shiraz_sandbox = true;
        st.save_persisted_settings();

        let mut loaded = SystemState::default();
        loaded.load_persisted_settings();
        assert!(loaded.shiraz_enabled);
        assert!(!loaded.shiraz_prewarm_memory);
        assert!(loaded.shiraz_aot_enabled);
        assert!(!loaded.shiraz_autostart);
        assert!(loaded.shiraz_sandbox);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
