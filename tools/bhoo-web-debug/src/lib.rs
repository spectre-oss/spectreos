//! Debug-only WebAssembly host for interactively debugging standalone `libbhoo` apps
//! (`files`, `settings`, `calendar`, `keyboard`, `notifications`) in a browser.
//!
//! Emits the exact `0x42484f4f` v1 binary render packet understood by
//! `codes/apps/bhoo-web/site/canvas-painter.js`.
//!
//! This crate lives under `tools/bhoo-web-debug/` and is never linked into native phone binaries.

use bhoo::*;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppKind {
    #[cfg(feature = "files")]
    Files,
    #[cfg(feature = "settings")]
    Settings,
    #[cfg(feature = "calendar")]
    Calendar,
    #[cfg(feature = "keyboard")]
    Keyboard,
    #[cfg(feature = "notifications")]
    Notifications,
}

impl AppKind {
    fn slug(self) -> &'static str {
        match self {
            #[cfg(feature = "files")]
            Self::Files => "files",
            #[cfg(feature = "settings")]
            Self::Settings => "settings",
            #[cfg(feature = "calendar")]
            Self::Calendar => "calendar",
            #[cfg(feature = "keyboard")]
            Self::Keyboard => "keyboard",
            #[cfg(feature = "notifications")]
            Self::Notifications => "notifications",
        }
    }

    fn all() -> Vec<Self> {
        let mut out = Vec::new();
        #[cfg(feature = "files")]
        out.push(Self::Files);
        #[cfg(feature = "settings")]
        out.push(Self::Settings);
        #[cfg(feature = "calendar")]
        out.push(Self::Calendar);
        #[cfg(feature = "keyboard")]
        out.push(Self::Keyboard);
        #[cfg(feature = "notifications")]
        out.push(Self::Notifications);
        out
    }
}

struct DebugHost {
    ui: Ui,
    commands: Vec<DrawCommand>,
    packet: Vec<u8>,
    apps: Vec<AppKind>,
    active_idx: usize,
    edit_buf: [u8; 512],
    inject_buf: [u8; 2048],
    state_json: String,
    logs: Vec<String>,
    logs_json: String,
    bus_events: Vec<String>,
    bus_json: String,
    sys_notif_seq: u32,
    apps_json: String,
    focus_request: u64,
    last_time: f64,

    #[cfg(feature = "files")]
    files: spectre_files::Browser,

    #[cfg(feature = "settings")]
    settings_app: spectre_settings::SettingsApp,
    #[cfg(feature = "settings")]
    settings_sys: spectre_settings::SystemState,
    #[cfg(feature = "settings")]
    settings_device: spectre_settings::DeviceRuntimeInfo,
    #[cfg(feature = "settings")]
    settings_apps: Vec<spectre_settings::InstalledAppInfo>,
    #[cfg(feature = "settings")]
    settings_history: Vec<spectre_settings::NotificationHistoryEntry>,

    #[cfg(feature = "calendar")]
    calendar: spectre_calendar::CalendarApp,

    #[cfg(feature = "keyboard")]
    keyboard: spectre_keyboard::KeyboardEngine,
    #[cfg(feature = "keyboard")]
    keyboard_text: String,
    #[cfg(feature = "keyboard")]
    keyboard_pressed: bool,

    #[cfg(feature = "notifications")]
    notif_shell: Box<wisp::WispShell>,
    #[cfg(feature = "notifications")]
    notif_mode: u32,
    #[cfg(feature = "notifications")]
    notif_next_id: u32,
    #[cfg(feature = "notifications")]
    notif_push_queue: Vec<u32>,
    #[cfg(feature = "notifications")]
    notif_gmail_buf: String,
}

impl DebugHost {
    fn new() -> Self {
        let apps = AppKind::all();
        let apps_json = format!(
            "[{}]",
            apps.iter()
                .map(|a| format!("\"{}\"", a.slug()))
                .collect::<Vec<_>>()
                .join(",")
        );

        #[cfg(feature = "settings")]
        let settings_apps = vec![
            spectre_settings::InstalledAppInfo {
                id: "org.spectreos.Files.desktop".into(),
                name: "Files".into(),
                package: "org.spectreos.Files".into(),
                version: "0.1.0".into(),
                storage_mb: 14,
                builtin_idx: None,
                desktop_id: Some("org.spectreos.Files.desktop".into()),
                is_android: false,
            },
            spectre_settings::InstalledAppInfo {
                id: "org.spectreos.Settings.desktop".into(),
                name: "Settings".into(),
                package: "org.spectreos.Settings".into(),
                version: "0.1.0".into(),
                storage_mb: 12,
                builtin_idx: None,
                desktop_id: Some("org.spectreos.Settings.desktop".into()),
                is_android: false,
            },
            spectre_settings::InstalledAppInfo {
                id: "org.spectreos.Calendar.desktop".into(),
                name: "Calendar".into(),
                package: "org.spectreos.Calendar".into(),
                version: "0.1.0".into(),
                storage_mb: 10,
                builtin_idx: None,
                desktop_id: Some("org.spectreos.Calendar.desktop".into()),
                is_android: false,
            },
        ];

        Self {
            ui: Ui::default(),
            commands: Vec::with_capacity(512),
            packet: Vec::with_capacity(16 * 1024),
            apps,
            active_idx: 0,
            edit_buf: [0; 512],
            inject_buf: [0; 2048],
            state_json: "{}".into(),
            logs: Vec::new(),
            logs_json: "[]".into(),
            bus_events: Vec::new(),
            bus_json: "[]".into(),
            sys_notif_seq: 500,
            apps_json,
            focus_request: 0,
            last_time: 0.0,

            #[cfg(feature = "files")]
            files: spectre_files::Browser::default(),

            #[cfg(feature = "settings")]
            settings_app: spectre_settings::SettingsApp::default(),
            #[cfg(feature = "settings")]
            settings_sys: spectre_settings::SystemState::default(),
            #[cfg(feature = "settings")]
            settings_device: spectre_settings::DeviceRuntimeInfo::probe(),
            #[cfg(feature = "settings")]
            settings_apps,
            #[cfg(feature = "settings")]
            settings_history: Vec::new(),

            #[cfg(feature = "calendar")]
            calendar: spectre_calendar::CalendarApp::default(),

            #[cfg(feature = "keyboard")]
            keyboard: spectre_keyboard::KeyboardEngine::new(spectre_keyboard::layout::LayoutId::Us),
            #[cfg(feature = "keyboard")]
            keyboard_text: String::from("Salam / سلام "),
            #[cfg(feature = "keyboard")]
            keyboard_pressed: false,

            #[cfg(feature = "notifications")]
            notif_shell: Box::default(),
            #[cfg(feature = "notifications")]
            notif_mode: 0,
            #[cfg(feature = "notifications")]
            notif_next_id: 100,
            #[cfg(feature = "notifications")]
            notif_push_queue: vec![1],
            #[cfg(feature = "notifications")]
            notif_gmail_buf: String::new(),
        }
    }

    #[cfg(feature = "notifications")]
    fn configure_notif_mode(&mut self, mode: u32) {
        self.notif_shell = Box::default();
        self.notif_mode = mode;
        self.notif_push_queue.clear();
        match mode {
            0 => {
                // Heads-Up Banner (Pinned): immediately trigger entrance animation with an interactive sample alert
                self.notif_push_queue.push(1);
            }
            1 => {
                // Full Android 13 Material 3 Notification Shade populated with sample notifications
                self.notif_shell.set_capture_preset("shade");
            }
            2 => {
                // Unlocked Home Screen + live auto-expiring heads-up notification
                self.notif_push_queue.push(1);
            }
            3 => {
                // Lock Screen notification cards + swipe-to-dismiss
                self.notif_shell.set_capture_preset("lock");
            }
            _ => {
                self.notif_mode = 0;
                self.notif_push_queue.push(1);
            }
        }
    }

    fn active_kind(&self) -> Option<AppKind> {
        self.apps.get(self.active_idx).copied()
    }

    fn push_log(&mut self, msg: impl Into<String>) {
        if self.logs.len() < 64 {
            self.logs.push(msg.into());
        }
    }

    fn push_bus_raw(&mut self, json_obj: String) {
        if self.bus_events.len() < 64 {
            self.bus_events.push(json_obj);
        }
    }

    fn next_sys_notif_id(&mut self) -> u32 {
        self.sys_notif_seq = self.sys_notif_seq.wrapping_add(1).max(500);
        self.sys_notif_seq
    }

    fn reset_active(&mut self) {
        let metrics = self.ui.text_metrics;
        self.ui = Ui::default();
        self.ui.text_metrics = metrics;
        self.focus_request = 1; // tell JS to blur any active editor
        match self.active_kind() {
            #[cfg(feature = "files")]
            Some(AppKind::Files) => {
                #[cfg(target_arch = "wasm32")]
                spectre_files::Browser::reset_wasm_demo_fs();
                self.files = spectre_files::Browser::default();
            }
            #[cfg(feature = "settings")]
            Some(AppKind::Settings) => {
                self.settings_app = spectre_settings::SettingsApp::default();
                self.settings_sys = spectre_settings::SystemState::default();
            }
            #[cfg(feature = "calendar")]
            Some(AppKind::Calendar) => {
                self.calendar = spectre_calendar::CalendarApp::default();
            }
            #[cfg(feature = "keyboard")]
            Some(AppKind::Keyboard) => {
                self.keyboard =
                    spectre_keyboard::KeyboardEngine::new(spectre_keyboard::layout::LayoutId::Us);
                self.keyboard_text = String::from("Salam / سلام ");
                self.keyboard_pressed = false;
            }
            #[cfg(feature = "notifications")]
            Some(AppKind::Notifications) => {
                let mode = self.notif_mode;
                self.configure_notif_mode(mode);
            }
            None => {}
        }
    }
}

thread_local! {
    static HOST: RefCell<DebugHost> = RefCell::new(DebugHost::new());
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_list_apps_ptr() -> u32 {
    HOST.with_borrow(|h| h.apps_json.as_ptr() as usize as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_list_apps_len() -> u32 {
    HOST.with_borrow(|h| h.apps_json.len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_select_app(index: u32) {
    HOST.with_borrow_mut(|h| {
        let idx = index as usize;
        if idx < h.apps.len() {
            h.active_idx = idx;
            let metrics = h.ui.text_metrics;
            h.ui = Ui::default();
            h.ui.text_metrics = metrics;
            h.focus_request = 1;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_reset() {
    HOST.with_borrow_mut(|h| h.reset_active());
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_back() -> u32 {
    HOST.with_borrow_mut(|h| {
        let handled = match h.active_kind() {
            #[cfg(feature = "files")]
            Some(AppKind::Files) => h.files.back(),
            #[cfg(feature = "settings")]
            Some(AppKind::Settings) => h.settings_app.go_back(h.last_time),
            #[cfg(feature = "calendar")]
            Some(AppKind::Calendar) => h.calendar.back(),
            #[cfg(feature = "keyboard")]
            Some(AppKind::Keyboard) => {
                if h.keyboard.emoji_open {
                    h.keyboard.emoji_open = false;
                    true
                } else {
                    false
                }
            }
            #[cfg(feature = "notifications")]
            Some(AppKind::Notifications) => {
                h.notif_shell.system_key(3, h.last_time);
                true
            }
            None => false,
        };
        u32::from(handled)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_files_mode(mode: u32) {
    #[cfg(feature = "files")]
    HOST.with_borrow_mut(|h| {
        use spectre_files::{Browser, Filter, PickerRequest};
        h.files = match mode {
            1 => Browser::picker(PickerRequest::Open {
                filter: Filter::Any,
                multiple: false,
            }),
            2 => Browser::picker(PickerRequest::Open {
                filter: Filter::Images,
                multiple: true,
            }),
            3 => Browser::picker(PickerRequest::Folder),
            4 => Browser::picker(PickerRequest::Save {
                suggested_name: "notes.txt".into(),
            }),
            _ => Browser::default(),
        };
        h.focus_request = 1;
    });
    #[cfg(not(feature = "files"))]
    let _ = mode;
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_notif_mode(mode: u32) {
    #[cfg(feature = "notifications")]
    HOST.with_borrow_mut(|h| {
        h.configure_notif_mode(mode);
        h.focus_request = 1;
    });
    #[cfg(not(feature = "notifications"))]
    let _ = mode;
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_notif_push(kind: u32) {
    #[cfg(feature = "notifications")]
    HOST.with_borrow_mut(|h| {
        h.notif_push_queue.push(kind);
    });
    #[cfg(not(feature = "notifications"))]
    let _ = kind;
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_notif_clear() {
    #[cfg(feature = "notifications")]
    HOST.with_borrow_mut(|h| {
        h.notif_push_queue.clear();
        #[cfg(target_arch = "wasm32")]
        h.notif_shell.clear_all_debug_notifications(h.last_time);
        h.push_log("Notifications::ClearAll");
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_set_clock(unix_secs: f64, tz_offset_secs: i32) {
    #[cfg(all(feature = "calendar", target_arch = "wasm32"))]
    {
        if unix_secs.is_finite() && unix_secs > 0.0 {
            spectre_calendar::set_wasm_clock(unix_secs as i64, tz_offset_secs);
        }
    }
    #[cfg(all(feature = "notifications", target_arch = "wasm32"))]
    {
        if unix_secs.is_finite() && unix_secs > 0.0 {
            wisp::services::time::wisp_wall_time(unix_secs * 1000.0, tz_offset_secs);
        }
    }
    #[cfg(not(any(
        all(feature = "calendar", target_arch = "wasm32"),
        all(feature = "notifications", target_arch = "wasm32")
    )))]
    let _ = (unix_secs, tz_offset_secs);
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_font_advance(code: u32, normal: f32, medium: f32) {
    if (32..127).contains(&code)
        && normal.is_finite()
        && medium.is_finite()
        && (0.0..=2.0).contains(&normal)
        && (0.0..=2.0).contains(&medium)
    {
        HOST.with_borrow_mut(|h| {
            h.ui.text_metrics.normal[code as usize - 32] = normal;
            h.ui.text_metrics.medium[code as usize - 32] = medium;
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_font_baseline(normal: f32, medium: f32) {
    if normal.is_finite()
        && medium.is_finite()
        && (-1.0..=1.0).contains(&normal)
        && (-1.0..=1.0).contains(&medium)
    {
        HOST.with_borrow_mut(|h| {
            h.ui.text_metrics.normal_baseline_offset = normal;
            h.ui.text_metrics.medium_baseline_offset = medium;
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_adapter_focus(id: u64, focused: u32) {
    HOST.with_borrow_mut(|h| {
        h.ui.adapter_focus(if focused != 0 { Some(id) } else { None });
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_edit_buffer() -> u32 {
    HOST.with_borrow_mut(|h| h.edit_buf.as_mut_ptr() as usize as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_edit_commit(id: u64, length: u32) {
    HOST.with_borrow_mut(|h| {
        let len = (length as usize).min(h.edit_buf.len());
        let Ok(text) = std::str::from_utf8(&h.edit_buf[..len]) else {
            return;
        };
        let value = text.to_owned();
        match h.active_kind() {
            #[cfg(feature = "files")]
            Some(AppKind::Files) => {
                h.files.set_edit_text(id, &value);
            }
            #[cfg(feature = "settings")]
            Some(AppKind::Settings) => {
                h.settings_app.set_edit_text(id, &value);
            }
            #[cfg(feature = "calendar")]
            Some(AppKind::Calendar) => {
                h.calendar.set_edit_text(id, &value);
            }
            #[cfg(feature = "notifications")]
            Some(AppKind::Notifications) => {
                h.notif_shell.set_edit_text(id, &value);
            }
            _ => {}
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_text_ptr(id: u64) -> u32 {
    HOST.with_borrow(|h| {
        let s: &str = match h.active_kind() {
            #[cfg(feature = "files")]
            Some(AppKind::Files) => h.files.edit_text(id).unwrap_or(""),
            #[cfg(feature = "settings")]
            Some(AppKind::Settings) => h.settings_app.edit_text_for(id),
            #[cfg(feature = "calendar")]
            Some(AppKind::Calendar) => h.calendar.edit_text(id).unwrap_or(""),
            #[cfg(feature = "notifications")]
            Some(AppKind::Notifications) => h.notif_shell.edit_text_for(id),
            _ => "",
        };
        s.as_ptr() as usize as u32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_text_len(id: u64) -> u32 {
    HOST.with_borrow(|h| {
        let s: &str = match h.active_kind() {
            #[cfg(feature = "files")]
            Some(AppKind::Files) => h.files.edit_text(id).unwrap_or(""),
            #[cfg(feature = "settings")]
            Some(AppKind::Settings) => h.settings_app.edit_text_for(id),
            #[cfg(feature = "calendar")]
            Some(AppKind::Calendar) => h.calendar.edit_text(id).unwrap_or(""),
            #[cfg(feature = "notifications")]
            Some(AppKind::Notifications) => h.notif_shell.edit_text_for(id),
            _ => "",
        };
        s.len() as u32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_take_focus_request() -> u64 {
    HOST.with_borrow_mut(|h| std::mem::take(&mut h.focus_request))
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_state_ptr() -> u32 {
    HOST.with_borrow(|h| h.state_json.as_ptr() as usize as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_state_len() -> u32 {
    HOST.with_borrow(|h| h.state_json.len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_take_logs_ptr() -> u32 {
    HOST.with_borrow_mut(|h| {
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        h.logs_json = format!(
            "[{}]",
            h.logs
                .drain(..)
                .map(|s| format!("\"{}\"", esc(&s)))
                .collect::<Vec<_>>()
                .join(",")
        );
        h.logs_json.as_ptr() as usize as u32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_take_logs_len() -> u32 {
    HOST.with_borrow(|h| h.logs_json.len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_take_bus_ptr() -> u32 {
    HOST.with_borrow_mut(|h| {
        h.bus_json = format!("[{}]", h.bus_events.drain(..).collect::<Vec<_>>().join(","));
        h.bus_json.as_ptr() as usize as u32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_take_bus_len() -> u32 {
    HOST.with_borrow(|h| h.bus_json.len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_inject_buffer() -> u32 {
    HOST.with_borrow_mut(|h| h.inject_buf.as_mut_ptr() as usize as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_keyboard_layout(layout: u32) {
    #[cfg(feature = "keyboard")]
    HOST.with_borrow_mut(|h| {
        let id = match layout {
            1 => spectre_keyboard::layout::LayoutId::Ir,
            2 => spectre_keyboard::layout::LayoutId::Terminal,
            _ => spectre_keyboard::layout::LayoutId::Us,
        };
        let _ = h.keyboard.set_layout(id);
        let slug = id.as_str();
        h.push_log(format!("D-Bus IN -> org.spectreos.OSK0.SetLayout(\"{slug}\")"));
        h.push_bus_raw(bus_event_json(
            "in",
            "dbus-call",
            "session",
            "org.spectreos.OSK0",
            "/org/spectreos/OSK0",
            "org.spectreos.OSK0",
            "SetLayout",
            &format!("{{\"layout\":\"{slug}\"}}"),
            None,
        ));
        h.push_bus_raw(bus_event_json(
            "out",
            "dbus-signal",
            "session",
            "org.spectreos.OSK0",
            "/org/spectreos/OSK0",
            "org.freedesktop.DBus.Properties",
            "PropertiesChanged",
            &format!("{{\"interface\":\"org.spectreos.OSK0\",\"changed\":{{\"Layout\":\"{slug}\"}}}}"),
            None,
        ));
    });
    #[cfg(not(feature = "keyboard"))]
    let _ = layout;
}

/// Inject a simulated D-Bus call or System Notification JSON payload into the active app.
/// Payload is written to `debug_inject_buffer()` as UTF-8 JSON with flat string/number fields:
/// `service`, `interface`, `member`, `path`, `app`, `title`, `body`, `urgency`, `layout`, `value`, `mode`, `id`.
#[unsafe(no_mangle)]
pub extern "C" fn debug_inject_dbus(length: u32) -> u32 {
    HOST.with_borrow_mut(|h| {
        let len = (length as usize).min(h.inject_buf.len());
        let Ok(raw) = std::str::from_utf8(&h.inject_buf[..len]) else {
            return 0;
        };
        let payload = raw.to_owned();
        let service = json_str(&payload, "service").unwrap_or_else(|| "org.freedesktop.Notifications".into());
        let interface = json_str(&payload, "interface").unwrap_or_else(|| service.clone());
        let member = json_str(&payload, "member").unwrap_or_else(|| "Notify".into());
        let path = json_str(&payload, "path").unwrap_or_else(|| {
            if service.contains("OSK0") {
                "/org/spectreos/OSK0".into()
            } else if service.contains("Notifications") {
                "/org/freedesktop/Notifications".into()
            } else if service.contains("NetworkManager") {
                "/org/freedesktop/NetworkManager".into()
            } else if service.contains("portal") {
                "/org/freedesktop/portal/desktop".into()
            } else {
                "/org/spectreos/Debug".into()
            }
        });

        // Dispatch known D-Bus / System Notification methods into the live apps.
        if interface == "org.freedesktop.Notifications" && member == "Notify" {
            let app = json_str(&payload, "app").unwrap_or_else(|| "SpectreOS".into());
            let title = json_str(&payload, "title").unwrap_or_else(|| "Debug Notification".into());
            let body = json_str(&payload, "body").unwrap_or_else(|| "Injected via org.freedesktop.Notifications.Notify".into());
            let urgency = json_u32(&payload, "urgency").unwrap_or(1).min(2) as u8;
            let nid = h.next_sys_notif_id();

            #[cfg(feature = "notifications")]
            if h.active_kind() == Some(AppKind::Notifications) {
                let t = h.last_time;
                h.notif_shell.push_freedesktop_notification(
                    wisp::notifications::IncomingNotification {
                        id: nid,
                        replaces_id: 0,
                        app_name: app.clone(),
                        app_icon: String::new(),
                        summary: title.clone(),
                        body: body.clone(),
                        actions: vec![
                            ("open".into(), "Open".into()),
                            ("dismiss".into(), "Dismiss".into()),
                        ],
                        urgency,
                        category: Some("x-spectre.debug".into()),
                        desktop_entry: None,
                        resident: false,
                        transient: false,
                        suppress_sound: urgency == 0,
                        image_data: if urgency >= 2 { Some(make_critical_badge_rgba()) } else { None },
                        image_path: None,
                        icon_data: None,
                        expire_timeout_ms: if urgency >= 2 { 0 } else { -1 },
                    },
                    t,
                );
            }

            #[cfg(feature = "settings")]
            {
                h.settings_history.insert(
                    0,
                    spectre_settings::NotificationHistoryEntry {
                        id: nid,
                        app: app.clone(),
                        title: title.clone(),
                        body: body.clone(),
                        time_label: "now".into(),
                        dismissed: false,
                        urgency,
                    },
                );
            }

            let notif_json = notif_payload_json(nid, &app, &title, &body, urgency, &["Open", "Dismiss"]);
            h.push_bus_raw(bus_event_json(
                "in",
                "notification",
                "session",
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
                "Notify",
                &format!(
                    "{{\"id\":{nid},\"app\":\"{}\",\"summary\":\"{}\",\"body\":\"{}\",\"urgency\":{urgency}}}",
                    json_esc(&app),
                    json_esc(&title),
                    json_esc(&body)
                ),
                Some(&notif_json),
            ));
            h.push_log(format!("System Notification #{nid}: {app} — {title}"));
            return 1;
        }

        if interface == "org.freedesktop.Notifications" && member == "CloseNotification" {
            let id = json_u32(&payload, "id").unwrap_or(0);
            #[cfg(feature = "notifications")]
            if id != 0 {
                h.notif_shell.close_freedesktop_notification(id);
            }
            #[cfg(feature = "settings")]
            for e in &mut h.settings_history {
                if e.id == id {
                    e.dismissed = true;
                }
            }
            h.push_bus_raw(bus_event_json(
                "in",
                "dbus-call",
                "session",
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
                "CloseNotification",
                &format!("{{\"id\":{id}}}"),
                None,
            ));
            h.push_bus_raw(bus_event_json(
                "out",
                "dbus-signal",
                "session",
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
                "NotificationClosed",
                &format!("{{\"id\":{id},\"reason\":3}}"),
                None,
            ));
            h.push_log(format!("D-Bus: CloseNotification({id})"));
            return 1;
        }

        if interface.contains("OSK0") && member == "SetLayout" {
            let layout_str = json_str(&payload, "layout")
                .or_else(|| json_str(&payload, "value"))
                .unwrap_or_else(|| "ir".into());
            #[cfg(feature = "keyboard")]
            if let Some(lid) = spectre_keyboard::layout::LayoutId::parse(&layout_str) {
                let _ = h.keyboard.set_layout(lid);
            }
            h.push_bus_raw(bus_event_json(
                "in",
                "dbus-call",
                "session",
                &service,
                &path,
                &interface,
                "SetLayout",
                &format!("{{\"layout\":\"{}\"}}", json_esc(&layout_str)),
                None,
            ));
            h.push_bus_raw(bus_event_json(
                "out",
                "dbus-signal",
                "session",
                &service,
                &path,
                "org.freedesktop.DBus.Properties",
                "PropertiesChanged",
                &format!("{{\"Layout\":\"{}\"}}", json_esc(&layout_str)),
                None,
            ));
            h.push_log(format!("D-Bus: {interface}.SetLayout({layout_str})"));
            return 1;
        }

        if interface.contains("NetworkManager") && member == "SetWifi" {
            let enabled = json_str(&payload, "value")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true);
            #[cfg(feature = "settings")]
            {
                h.settings_sys.wifi_enabled = enabled;
            }
            h.push_bus_raw(bus_event_json(
                "in",
                "dbus-call",
                "system",
                "org.freedesktop.NetworkManager",
                "/org/freedesktop/NetworkManager",
                "org.freedesktop.NetworkManager",
                "SetWifi",
                &format!("{{\"enabled\":{enabled}}}"),
                None,
            ));
            h.push_log(format!("D-Bus: NetworkManager.SetWifi({enabled})"));
            return 1;
        }

        if interface.contains("FileChooser") {
            let mode = match member.as_str() {
                "SaveFile" => 4,
                "OpenDirectory" => 3,
                "OpenImages" => 2,
                _ => json_u32(&payload, "mode").unwrap_or(1),
            };
            #[cfg(feature = "files")]
            {
                use spectre_files::{Browser, Filter, PickerRequest};
                h.files = match mode {
                    2 => Browser::picker(PickerRequest::Open {
                        filter: Filter::Images,
                        multiple: true,
                    }),
                    3 => Browser::picker(PickerRequest::Folder),
                    4 => Browser::picker(PickerRequest::Save {
                        suggested_name: json_str(&payload, "value")
                            .unwrap_or_else(|| "export.txt".into()),
                    }),
                    _ => Browser::picker(PickerRequest::Open {
                        filter: Filter::Any,
                        multiple: false,
                    }),
                };
            }
            h.push_bus_raw(bus_event_json(
                "in",
                "dbus-call",
                "session",
                "org.freedesktop.portal.Desktop",
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.FileChooser",
                &member,
                &format!("{{\"mode\":{mode}}}"),
                None,
            ));
            h.push_log(format!("D-Bus: FileChooser.{member} (mode={mode})"));
            return 1;
        }

        // Generic D-Bus injection logged onto the session/system bus monitor.
        let bus = if service.contains("NetworkManager") || service.contains("bluez") || service.contains("login1") {
            "system"
        } else {
            "session"
        };
        h.push_bus_raw(bus_event_json(
            "in",
            "dbus-call",
            bus,
            &service,
            &path,
            &interface,
            &member,
            &format!("{{\"raw\":\"{}\"}}", json_esc(&payload)),
            None,
        ));
        h.push_log(format!("D-Bus IN: {interface}.{member}"));
        1
    })
}

fn json_esc(s: &str) -> String {
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

fn json_str(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let idx = json.find(&needle)?;
    let rest = &json[idx + needle.len()..];
    let colon = rest.find(':')?;
    let after = rest[colon + 1..].trim_start();
    let after = after.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = after.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(esc) = chars.next() {
                match esc {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    other => out.push(other),
                }
            }
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None
}

fn json_u32(json: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\"");
    let idx = json.find(&needle)?;
    let rest = &json[idx + needle.len()..];
    let colon = rest.find(':')?;
    let after = rest[colon + 1..].trim_start();
    let digits: String = after
        .trim_start_matches('"')
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn notif_payload_json(
    id: u32,
    app: &str,
    title: &str,
    body: &str,
    urgency: u8,
    actions: &[&str],
) -> String {
    let acts = actions
        .iter()
        .map(|a| format!("\"{}\"", json_esc(a)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"id\":{id},\"app\":\"{}\",\"title\":\"{}\",\"body\":\"{}\",\"urgency\":{urgency},\"actions\":[{acts}]}}",
        json_esc(app),
        json_esc(title),
        json_esc(body),
    )
}

#[allow(clippy::too_many_arguments)]
fn bus_event_json(
    direction: &str,
    kind: &str,
    bus: &str,
    service: &str,
    path: &str,
    interface: &str,
    member: &str,
    args_json: &str,
    notif_json: Option<&str>,
) -> String {
    let notif_part = notif_json.unwrap_or("null");
    format!(
        "{{\"dir\":\"{direction}\",\"kind\":\"{kind}\",\"bus\":\"{bus}\",\"service\":\"{}\",\"path\":\"{}\",\"interface\":\"{}\",\"member\":\"{}\",\"args\":{args_json},\"notification\":{notif_part}}}",
        json_esc(service),
        json_esc(path),
        json_esc(interface),
        json_esc(member),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn debug_packet_len() -> u32 {
    HOST.with_borrow(|h| h.packet.len() as u32)
}

fn decode_key(code: u32) -> Option<Key> {
    match code {
        1 => Some(Key::Tab),
        2 => Some(Key::Enter),
        3 => Some(Key::Space),
        4 => Some(Key::Left),
        5 => Some(Key::Right),
        6 => Some(Key::Up),
        7 => Some(Key::Down),
        8 => Some(Key::Home),
        9 => Some(Key::End),
        10 => Some(Key::Escape),
        11 => Some(Key::Menu),
        12 => Some(Key::PageUp),
        13 => Some(Key::PageDown),
        _ => None,
    }
}

#[cfg(feature = "files")]
fn files_appearance(dark: bool) -> spectre_files::Appearance {
    if dark {
        spectre_files::Appearance {
            background: Color(20, 22, 27, 255),
            surface: Color(30, 33, 41, 255),
            raised: Color(42, 46, 58, 255),
            text: Color(235, 238, 245, 255),
            muted: Color(156, 165, 184, 255),
            primary: Color(154, 186, 255, 255),
            selected: Color(46, 62, 96, 255),
            on_primary: Color(16, 28, 52, 255),
            outline: Color(64, 71, 88, 255),
        }
    } else {
        spectre_files::Appearance::default()
    }
}

#[cfg(feature = "calendar")]
fn calendar_appearance(dark: bool) -> spectre_calendar::Appearance {
    if dark {
        spectre_calendar::Appearance {
            background: Color(20, 22, 27, 255),
            surface: Color(30, 33, 41, 255),
            text: Color(235, 238, 245, 255),
            muted: Color(156, 165, 184, 255),
            primary: Color(154, 186, 255, 255),
            on_primary: Color(16, 28, 52, 255),
            accent_surface: Color(46, 62, 96, 255),
            on_accent_surface: Color(224, 234, 255, 255),
            divider: Color(58, 63, 76, 255),
        }
    } else {
        spectre_calendar::Appearance {
            background: Color(247, 248, 250, 255),
            surface: Color(255, 255, 255, 255),
            text: Color(28, 34, 44, 255),
            muted: Color(89, 100, 116, 255),
            primary: Color(43, 78, 144, 255),
            on_primary: Color(255, 255, 255, 255),
            accent_surface: Color(218, 229, 253, 255),
            on_accent_surface: Color(26, 48, 92, 255),
            divider: Color(210, 218, 230, 255),
        }
    }
}

#[cfg(feature = "notifications")]
fn make_critical_badge_rgba() -> wisp::notifications::RawImageData {
    let w = 48u32;
    let h = 48u32;
    let mut data = vec![0u8; (w * h * 4) as usize];
    let cx = 23.5f32;
    let cy = 23.5f32;
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = dx.hypot(dy);
            let idx = ((y * w + x) * 4) as usize;
            if dist <= 22.0 {
                let is_excl = (x >= 21 && x <= 26 && y >= 11 && y <= 28)
                    || (x >= 21 && x <= 26 && y >= 32 && y <= 37);
                if is_excl {
                    data[idx..idx + 4].copy_from_slice(&[255, 250, 250, 255]);
                } else {
                    let shade = (1.0 - (dist / 24.0) * 0.18).clamp(0.75, 1.0);
                    data[idx] = (217.0 * shade) as u8;
                    data[idx + 1] = (48.0 * shade) as u8;
                    data[idx + 2] = (37.0 * shade) as u8;
                    data[idx + 3] = 255;
                }
            }
        }
    }
    wisp::notifications::RawImageData {
        width: w,
        height: h,
        rowstride: (w * 4) as usize,
        has_alpha: true,
        bits_per_sample: 8,
        channels: 4,
        data,
    }
}

#[cfg(feature = "notifications")]
fn build_sample_notification(id: u32, kind: u32) -> wisp::notifications::IncomingNotification {
    match kind {
        0 => wisp::notifications::IncomingNotification {
            id,
            replaces_id: 0,
            app_name: "SpectreOS".into(),
            app_icon: String::new(),
            summary: format!("Build #{id} ready"),
            body: "libbhoo WASM bundle compiled cleanly in 0.3s.".into(),
            actions: Vec::new(),
            urgency: 1,
            category: Some("transfer.complete".into()),
            desktop_entry: None,
            resident: false,
            transient: false,
            suppress_sound: false,
            image_data: None,
            image_path: None,
            icon_data: None,
            expire_timeout_ms: -1,
        },
        2 => wisp::notifications::IncomingNotification {
            id,
            replaces_id: 0,
            app_name: "System Security".into(),
            app_icon: String::new(),
            summary: "Critical thermal alert".into(),
            body: "Device temperature reached 46 °C during stress benchmark. Tap Inspect or pause background sync to cool down immediately.".into(),
            actions: vec![
                ("inspect".into(), "Inspect".into()),
                ("throttle".into(), "Pause sync".into()),
            ],
            urgency: 2,
            category: Some("device.error".into()),
            desktop_entry: None,
            resident: false,
            transient: false,
            suppress_sound: false,
            image_data: Some(make_critical_badge_rgba()),
            image_path: None,
            icon_data: None,
            expire_timeout_ms: 0,
        },
        _ => wisp::notifications::IncomingNotification {
            id,
            replaces_id: 0,
            app_name: "ArcaneChat".into(),
            app_icon: String::new(),
            summary: "Maya Lin".into(),
            body: "Hey! Testing the heads-up notification entrance, chevron expand/collapse, and action pills live inside the WASM browser debugger.".into(),
            actions: vec![
                ("reply".into(), "Reply".into()),
                ("mark_read".into(), "Mark read".into()),
            ],
            urgency: 1,
            category: Some("im.received".into()),
            desktop_entry: None,
            resident: false,
            transient: false,
            suppress_sound: false,
            image_data: None,
            image_path: None,
            icon_data: None,
            expire_timeout_ms: -1,
        },
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn debug_frame(
    width: f32,
    height: f32,
    time: f64,
    x: f32,
    y: f32,
    flags: u32,
    key: u32,
    scroll: f32,
    options: u32,
) -> u32 {
    HOST.with_borrow_mut(|h| {
        let dark = (options & 1) != 0;
        let reduced = (options & 2) != 0;
        h.last_time = time;
        h.ui.theme = if dark { Theme::default() } else { Theme::light() };
        h.ui.motion_preference = if reduced {
            MotionPreference::Reduced
        } else {
            MotionPreference::Full
        };
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: height.max(1.0),
        };
        h.ui.viewport = Some(bounds);

        let input = Input {
            pointer: if x.is_finite() && y.is_finite() {
                Some([x, y])
            } else {
                None
            },
            pointer_kind: if (flags & 128) != 0 {
                PointerKind::Touch
            } else if (flags & 256) != 0 {
                PointerKind::Pen
            } else {
                PointerKind::Mouse
            },
            pressed: (flags & 1) != 0,
            released: (flags & 2) != 0,
            cancelled: (flags & 4) != 0,
            blurred: (flags & 8) != 0,
            key: if (flags & 16) != 0 {
                decode_key(key).map(|k| KeyInput {
                    key: k,
                    pressed: true,
                    repeat: (flags & 32) != 0,
                    shift: (flags & 64) != 0,
                })
            } else {
                None
            },
            scroll: if (flags & 512) != 0 {
                [scroll, 0.0]
            } else {
                [0.0, scroll]
            },
            time,
        };

        let active = h.active_kind();
        let mut focus_req: u64 = 0;
        let mut frame_logs: Vec<String> = Vec::new();
        let mut frame_bus: Vec<String> = Vec::new();
        let mut state_json = String::new();

        #[cfg(feature = "keyboard")]
        let mut extra_kb_cmds: Vec<DrawCommand> = Vec::new();

        {
            let ui = &mut h.ui;
            let commands = &mut h.commands;
            let sys_notif_seq = &mut h.sys_notif_seq;
            let mut alloc_notif_id = || -> u32 {
                *sys_notif_seq = sys_notif_seq.wrapping_add(1).max(500);
                *sys_notif_seq
            };
            #[cfg(feature = "files")]
            let files = &mut h.files;
            #[cfg(feature = "settings")]
            let (settings_app, settings_sys, settings_device, settings_apps, settings_history) = (
                &mut h.settings_app,
                &mut h.settings_sys,
                &h.settings_device,
                &h.settings_apps,
                &mut h.settings_history,
            );
            #[cfg(feature = "calendar")]
            let calendar = &mut h.calendar;
            #[cfg(feature = "keyboard")]
            let (keyboard, keyboard_text, keyboard_pressed) = (
                &mut h.keyboard,
                &mut h.keyboard_text,
                &mut h.keyboard_pressed,
            );
            #[cfg(feature = "notifications")]
            let (notif_shell, notif_mode, notif_next_id, notif_push_queue, notif_gmail_buf) = (
                &mut h.notif_shell,
                &mut h.notif_mode,
                &mut h.notif_next_id,
                &mut h.notif_push_queue,
                &mut h.notif_gmail_buf,
            );

            ui.frame_into(input, bounds, 0.0, commands, |f| match active {
                #[cfg(feature = "files")]
                Some(AppKind::Files) => {
                    let act = files.show(f, bounds, files_appearance(dark));
                    match act {
                        spectre_files::Action::None => {}
                        spectre_files::Action::Focus(id) => {
                            focus_req = id;
                            frame_logs.push(format!("Files::Focus({id})"));
                        }
                        spectre_files::Action::Blur => {
                            focus_req = 1;
                            frame_logs.push("Files::Blur".into());
                        }
                        spectre_files::Action::Open(path) => {
                            let disp = path.display().to_string();
                            files.report(format!("Open requested: {disp}"));
                            frame_logs.push(format!("Files::Open({disp})"));
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "session",
                                "org.freedesktop.portal.Desktop",
                                "/org/freedesktop/portal/desktop",
                                "org.freedesktop.portal.OpenURI",
                                "OpenFile",
                                &format!("{{\"uri\":\"file://{}\"}}", json_esc(&disp)),
                                None,
                            ));
                        }
                        spectre_files::Action::Share(paths) => {
                            let count = paths.len();
                            files.report(format!("Shared {count} item(s) (debug)"));
                            frame_logs.push(format!("Files::Share({count} items)"));
                            let nid = alloc_notif_id();
                            let body = format!("Shared {count} item(s) via desktop portal.");
                            let notif_json = notif_payload_json(nid, "Files", "Items shared", &body, 1, &["View"]);
                            frame_bus.push(bus_event_json(
                                "out",
                                "notification",
                                "session",
                                "org.freedesktop.Notifications",
                                "/org/freedesktop/Notifications",
                                "org.freedesktop.Notifications",
                                "Notify",
                                &format!("{{\"id\":{nid},\"app\":\"Files\",\"count\":{count}}}"),
                                Some(&notif_json),
                            ));
                        }
                        spectre_files::Action::NewWindow(folder) => {
                            frame_logs.push(format!("Files::NewWindow({folder:?})"));
                            files.report("NewWindow requested (debug)");
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "session",
                                "org.spectreos.Files",
                                "/org/spectreos/Files",
                                "org.freedesktop.Application",
                                "Activate",
                                &format!("{{\"folder\":\"{}\"}}", json_esc(&format!("{folder:?}"))),
                                None,
                            ));
                        }
                        spectre_files::Action::Selected(paths) => {
                            let joined = paths
                                .iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            let uris = paths
                                .iter()
                                .map(|p| format!("\"file://{}\"", json_esc(&p.display().to_string())))
                                .collect::<Vec<_>>()
                                .join(",");
                            frame_logs.push(format!("Files::Selected([{joined}])"));
                            files.report(format!("Selected: {joined}"));
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-signal",
                                "session",
                                "org.freedesktop.portal.Desktop",
                                "/org/freedesktop/portal/desktop",
                                "org.freedesktop.portal.Request",
                                "Response",
                                &format!("{{\"response\":0,\"uris\":[{uris}]}}"),
                                None,
                            ));
                        }
                        spectre_files::Action::Cancelled => {
                            frame_logs.push("Files::Cancelled".into());
                            files.report("Picker cancelled");
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-signal",
                                "session",
                                "org.freedesktop.portal.Desktop",
                                "/org/freedesktop/portal/desktop",
                                "org.freedesktop.portal.Request",
                                "Response",
                                "{\"response\":1,\"uris\":[]}",
                                None,
                            ));
                        }
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        state_json = files.debug_state_json();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        state_json = "{\"app\":\"files\"}".into();
                    }
                }

                #[cfg(feature = "settings")]
                Some(AppKind::Settings) => {
                    settings_sys.dark_theme = dark;
                    let act = {
                        let ctx = spectre_settings::SettingsHostContext {
                            clock: "12:00",
                            date: "Tue, Nov 24",
                            apps: settings_apps,
                            active_notif_count: settings_history.iter().filter(|n| !n.dismissed).count(),
                            history: settings_history,
                            device: settings_device,
                            paint_app_icon: None,
                        };
                        settings_app.show(
                            f,
                            bounds,
                            0x4C04_0000_0000_0000,
                            settings_sys,
                            input.pointer,
                            &ctx,
                        )
                    };
                    match &act {
                        spectre_settings::SettingsAction::None => {}
                        spectre_settings::SettingsAction::PostTestNotification { app, title, body } => {
                            let nid = alloc_notif_id();
                            settings_history.insert(
                                0,
                                spectre_settings::NotificationHistoryEntry {
                                    id: nid,
                                    app: app.clone(),
                                    title: title.clone(),
                                    body: body.clone(),
                                    time_label: "now".into(),
                                    dismissed: false,
                                    urgency: 1,
                                },
                            );
                            let notif_json = notif_payload_json(nid, app, title, body, 1, &["Open", "Dismiss"]);
                            frame_bus.push(bus_event_json(
                                "out",
                                "notification",
                                "session",
                                "org.freedesktop.Notifications",
                                "/org/freedesktop/Notifications",
                                "org.freedesktop.Notifications",
                                "Notify",
                                &format!(
                                    "{{\"id\":{nid},\"app\":\"{}\",\"summary\":\"{}\",\"body\":\"{}\"}}",
                                    json_esc(app),
                                    json_esc(title),
                                    json_esc(body)
                                ),
                                Some(&notif_json),
                            ));
                        }
                        spectre_settings::SettingsAction::ClearNotificationHistory => {
                            settings_history.clear();
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "session",
                                "org.freedesktop.Notifications",
                                "/org/freedesktop/Notifications",
                                "org.spectreos.Notifications",
                                "ClearHistory",
                                "{}",
                                None,
                            ));
                        }
                        spectre_settings::SettingsAction::JoinWifi(net) => {
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "system",
                                "org.freedesktop.NetworkManager",
                                "/org/freedesktop/NetworkManager",
                                "org.freedesktop.NetworkManager",
                                "AddAndActivateConnection",
                                &format!("{{\"ssid\":\"{}\"}}", json_esc(&net.ssid)),
                                None,
                            ));
                        }
                        spectre_settings::SettingsAction::ConnectBluetooth(addr, connect) => {
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "system",
                                "org.bluez",
                                "/org/bluez/hci0",
                                "org.bluez.Device1",
                                if *connect { "Connect" } else { "Disconnect" },
                                &format!("{{\"device\":\"{}\"}}", json_esc(addr)),
                                None,
                            ));
                        }
                        spectre_settings::SettingsAction::LockNow => {
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "system",
                                "org.freedesktop.login1",
                                "/org/freedesktop/login1/session/self",
                                "org.freedesktop.login1.Session",
                                "Lock",
                                "{}",
                                None,
                            ));
                        }
                        spectre_settings::SettingsAction::ShirazLaunchPackage(pkg) => {
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "session",
                                "org.spectreos.Shiraz",
                                "/org/spectreos/Shiraz",
                                "org.spectreos.Shiraz1",
                                "LaunchPackage",
                                &format!("{{\"package\":\"{}\"}}", json_esc(pkg)),
                                None,
                            ));
                        }
                        spectre_settings::SettingsAction::ShirazStopPackage(pkg) => {
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "session",
                                "org.spectreos.Shiraz",
                                "/org/spectreos/Shiraz",
                                "org.spectreos.Shiraz1",
                                "StopPackage",
                                &format!("{{\"package\":\"{}\"}}", json_esc(pkg)),
                                None,
                            ));
                        }
                        spectre_settings::SettingsAction::ShirazStopAll => {
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "session",
                                "org.spectreos.Shiraz",
                                "/org/spectreos/Shiraz",
                                "org.spectreos.Shiraz1",
                                "StopAll",
                                "{}",
                                None,
                            ));
                        }
                        spectre_settings::SettingsAction::ShirazRefresh => {
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "session",
                                "org.spectreos.Shiraz",
                                "/org/spectreos/Shiraz",
                                "org.spectreos.Shiraz1",
                                "Refresh",
                                "{}",
                                None,
                            ));
                        }
                        spectre_settings::SettingsAction::ShowToast(msg) => {
                            let nid = alloc_notif_id();
                            let notif_json = notif_payload_json(nid, "Settings", "Status", msg, 0, &[]);
                            frame_bus.push(bus_event_json(
                                "out",
                                "notification",
                                "session",
                                "org.spectreos.Shell",
                                "/org/spectreos/Shell",
                                "org.spectreos.Shell",
                                "ShowToast",
                                &format!("{{\"message\":\"{}\"}}", json_esc(msg)),
                                Some(&notif_json),
                            ));
                        }
                        other => {
                            frame_bus.push(bus_event_json(
                                "out",
                                "dbus-call",
                                "session",
                                "org.spectreos.Settings",
                                "/org/spectreos/Settings",
                                "org.spectreos.Settings",
                                "Action",
                                &format!("{{\"action\":\"{}\"}}", json_esc(&format!("{other:?}"))),
                                None,
                            ));
                        }
                    }
                    if !matches!(act, spectre_settings::SettingsAction::None) {
                        frame_logs.push(format!("Settings::{act:?}"));
                    }
                    state_json = format!(
                        "{{\"page\":\"{:?}\",\"darkTheme\":{},\"wifiEnabled\":{},\"shirazEnabled\":{},\"devUnlocked\":{},\"dialogOpen\":{},\"notifHistoryCount\":{}}}",
                        settings_app.current_page(),
                        settings_sys.dark_theme,
                        settings_sys.wifi_enabled,
                        settings_sys.shiraz_enabled,
                        settings_sys.dev_unlocked,
                        settings_app.is_dialog_open(),
                        settings_history.len(),
                    );
                }

                #[cfg(feature = "calendar")]
                Some(AppKind::Calendar) => {
                    let act = calendar.show(f, bounds, 0x4341_4C00_0000_0000, calendar_appearance(dark));
                    for (title, body) in calendar.due_reminders() {
                        let nid = alloc_notif_id();
                        let notif_json = notif_payload_json(nid, "Calendar", &title, &body, 1, &["Snooze", "Open"]);
                        frame_logs.push(format!("Calendar::Reminder({title})"));
                        frame_bus.push(bus_event_json(
                            "out",
                            "notification",
                            "session",
                            "org.freedesktop.Notifications",
                            "/org/freedesktop/Notifications",
                            "org.freedesktop.Notifications",
                            "Notify",
                            &format!(
                                "{{\"id\":{nid},\"app\":\"Calendar\",\"summary\":\"{}\",\"body\":\"{}\"}}",
                                json_esc(&title),
                                json_esc(&body)
                            ),
                            Some(&notif_json),
                        ));
                    }
                    match act {
                        spectre_calendar::Action::None => {}
                        spectre_calendar::Action::Focus(id) => {
                            focus_req = id;
                            frame_logs.push(format!("Calendar::Focus({id})"));
                        }
                        spectre_calendar::Action::Blur => {
                            focus_req = 1;
                            frame_logs.push("Calendar::Blur".into());
                        }
                    }
                    state_json = format!(
                        "{{\"app\":\"calendar\",\"choosingFile\":{}}}",
                        calendar.choosing_file()
                    );
                }

                #[cfg(feature = "keyboard")]
                Some(AppKind::Keyboard) => {
                    let kb_h = keyboard.desired_height_for_viewport(bounds.width, bounds.height);
                    let kb_y = (bounds.height - kb_h).max(64.0);
                    let kb_bounds = Rect {
                        x: 0.0,
                        y: kb_y,
                        width: bounds.width,
                        height: (bounds.height - kb_y).max(120.0),
                    };

                    let mut outs = keyboard.tick(time, kb_bounds.width, kb_bounds.height);
                    if let Some([px, py]) = input.pointer {
                        let lx = px - kb_bounds.x;
                        let ly = py - kb_bounds.y;
                        if input.pressed && py >= kb_y {
                            *keyboard_pressed = true;
                            outs.extend(keyboard.pointer_down(
                                lx,
                                ly,
                                kb_bounds.width,
                                kb_bounds.height,
                                time,
                            ));
                        } else if input.released && *keyboard_pressed {
                            *keyboard_pressed = false;
                            outs.extend(keyboard.pointer_up(
                                lx,
                                ly,
                                kb_bounds.width,
                                kb_bounds.height,
                                time,
                            ));
                        } else if *keyboard_pressed {
                            outs.extend(keyboard.pointer_move(
                                lx,
                                ly,
                                kb_bounds.width,
                                kb_bounds.height,
                                time,
                            ));
                        }
                    } else if input.cancelled && *keyboard_pressed {
                        *keyboard_pressed = false;
                        keyboard.cancel_press();
                    }

                    for ev in outs {
                        frame_logs.push(format!("Keyboard::{ev:?}"));
                        match ev {
                            spectre_keyboard::KeyboardOutput::CommitText { text, .. } => {
                                keyboard_text.push_str(&text);
                            }
                            spectre_keyboard::KeyboardOutput::SpecialKey {
                                key: spectre_keyboard::SpecialKey::Backspace,
                                ..
                            } => {
                                let (bytes, _) = spectre_keyboard::touch::erase_last_grapheme_bytes(
                                    keyboard_text,
                                    keyboard_text.len(),
                                );
                                let next_len = keyboard_text.len().saturating_sub(bytes);
                                keyboard_text.truncate(next_len);
                            }
                            spectre_keyboard::KeyboardOutput::SpecialKey {
                                key: spectre_keyboard::SpecialKey::Enter,
                                ..
                            } => keyboard_text.push('\n'),
                            spectre_keyboard::KeyboardOutput::LayoutChanged(layout_id) => {
                                let slug = layout_id.as_str();
                                frame_bus.push(bus_event_json(
                                    "out",
                                    "dbus-signal",
                                    "session",
                                    "org.spectreos.OSK0",
                                    "/org/spectreos/OSK0",
                                    "org.freedesktop.DBus.Properties",
                                    "PropertiesChanged",
                                    &format!("{{\"interface\":\"org.spectreos.OSK0\",\"changed\":{{\"Layout\":\"{slug}\"}}}}"),
                                    None,
                                ));
                            }
                            spectre_keyboard::KeyboardOutput::HideRequested => {
                                frame_bus.push(bus_event_json(
                                    "out",
                                    "dbus-signal",
                                    "session",
                                    "org.spectreos.OSK0",
                                    "/org/spectreos/OSK0",
                                    "org.freedesktop.DBus.Properties",
                                    "PropertiesChanged",
                                    "{\"interface\":\"org.spectreos.OSK0\",\"changed\":{\"Visible\":false}}",
                                    None,
                                ));
                            }
                            _ => {}
                        }
                    }

                    f.clear(Color(22, 23, 28, 255));
                    f.paint_rect(
                        Rect {
                            x: 12.0,
                            y: 12.0,
                            width: (bounds.width - 24.0).max(40.0),
                            height: (kb_y - 24.0).max(48.0),
                        },
                        12.0,
                        Color(35, 37, 45, 255),
                    );
                    f.styled_text(
                        Rect {
                            x: 24.0,
                            y: 20.0,
                            width: (bounds.width - 48.0).max(40.0),
                            height: (kb_y - 40.0).max(32.0),
                        },
                        &format!("{keyboard_text}|"),
                        Color(240, 242, 248, 255),
                        TextStyle {
                            size: 18.0,
                            weight: 400,
                            align: 0,
                            ..TextStyle::default()
                        },
                    );
                    keyboard.render(kb_bounds, &mut extra_kb_cmds);
                    if let Some(deadline) = keyboard.next_timer_deadline() {
                        f.wake_at(deadline);
                    }
                    state_json = format!(
                        "{{\"layout\":\"{}\",\"level\":{},\"shift\":\"{:?}\",\"emojiOpen\":{},\"persianFont\":\"Shabnam\",\"prefix\":\"{}\",\"textLen\":{},\"text\":\"{}\"}}",
                        keyboard.layout_id.as_str(),
                        keyboard.level_idx,
                        keyboard.shift_state,
                        keyboard.emoji_open,
                        json_esc(&keyboard.typing_prefix),
                        keyboard_text.chars().count(),
                        json_esc(keyboard_text),
                    );
                }

                #[cfg(feature = "notifications")]
                Some(AppKind::Notifications) => {
                    for kind in notif_push_queue.drain(..) {
                        let id = *notif_next_id;
                        *notif_next_id = notif_next_id.wrapping_add(1).max(100);
                        let incoming = build_sample_notification(id, kind);
                        let summary = incoming.summary.clone();
                        let app_name = incoming.app_name.clone();
                        let body = incoming.body.clone();
                        let urgency = incoming.urgency;
                        notif_shell.push_freedesktop_notification(incoming, time);
                        frame_logs.push(format!("Notifications::Push(#{id} {app_name}: {summary})"));
                        frame_bus.push(bus_event_json(
                            "in",
                            "notification",
                            "session",
                            "org.freedesktop.Notifications",
                            "/org/freedesktop/Notifications",
                            "org.freedesktop.Notifications",
                            "Notify",
                            &format!(
                                "{{\"id\":{id},\"app\":\"{}\",\"summary\":\"{}\",\"body\":\"{}\",\"urgency\":{urgency}}}",
                                json_esc(&app_name),
                                json_esc(&summary),
                                json_esc(&body)
                            ),
                            None,
                        ));
                    }
                    notif_shell.tick_notifications(time);
                    #[cfg(target_arch = "wasm32")]
                    notif_shell.set_dark_theme(dark);

                    if *notif_mode == 0 {
                        #[cfg(target_arch = "wasm32")]
                        notif_shell.show_heads_up_debug(f, true);
                        #[cfg(not(target_arch = "wasm32"))]
                        notif_shell.show(f, bounds.width, bounds.height, input, notif_gmail_buf);
                    } else {
                        notif_shell.show(f, bounds.width, bounds.height, input, notif_gmail_buf);
                    }

                    for cmd in notif_shell.take_host_commands() {
                        frame_logs.push(format!("Wisp::{cmd:?}"));
                        match cmd {
                            wisp::HostCommand::NotifyClosed { id, reason } => {
                                frame_bus.push(bus_event_json(
                                    "out",
                                    "dbus-signal",
                                    "session",
                                    "org.freedesktop.Notifications",
                                    "/org/freedesktop/Notifications",
                                    "org.freedesktop.Notifications",
                                    "NotificationClosed",
                                    &format!("{{\"id\":{id},\"reason\":{reason}}}"),
                                    None,
                                ));
                            }
                            wisp::HostCommand::NotifyActionInvoked { id, action_key } => {
                                frame_bus.push(bus_event_json(
                                    "out",
                                    "dbus-signal",
                                    "session",
                                    "org.freedesktop.Notifications",
                                    "/org/freedesktop/Notifications",
                                    "org.freedesktop.Notifications",
                                    "ActionInvoked",
                                    &format!("{{\"id\":{id},\"action_key\":\"{}\"}}", json_esc(&action_key)),
                                    None,
                                ));
                            }
                            other => {
                                frame_bus.push(bus_event_json(
                                    "out",
                                    "dbus-call",
                                    "session",
                                    "org.spectreos.Wisp",
                                    "/org/spectreos/Wisp",
                                    "org.spectreos.Wisp",
                                    "HostCommand",
                                    &format!("{{\"command\":\"{}\"}}", json_esc(&format!("{other:?}"))),
                                    None,
                                ));
                            }
                        }
                    }
                    for hap in notif_shell.drain_haptics() {
                        frame_logs.push(format!("Haptic::{hap:?}"));
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        state_json = notif_shell.debug_notifications_state_json(*notif_mode, time);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        state_json = "{\"app\":\"notifications\"}".into();
                    }
                }

                None => {
                    f.label("No apps enabled in bhoo-web-debug Cargo features.");
                }
            });
        }

        #[cfg(feature = "keyboard")]
        if !extra_kb_cmds.is_empty() {
            h.commands.extend(extra_kb_cmds);
        }

        if focus_req != 0 {
            h.focus_request = focus_req;
        }
        for log in frame_logs {
            h.push_log(log);
        }
        for bus_ev in frame_bus {
            h.push_bus_raw(bus_ev);
        }
        h.state_json = state_json;

        encode_packet(&h.ui, &h.commands, h.active_idx as u32, &mut h.packet);
        h.packet.as_ptr() as usize as u32
    })
}

fn u32v(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn f32v(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn rect(out: &mut Vec<u8>, b: Rect) {
    for v in [b.x, b.y, b.width, b.height] {
        f32v(out, v);
    }
}
fn color(out: &mut Vec<u8>, c: Color) {
    out.extend_from_slice(&[c.0, c.1, c.2, c.3]);
}
fn string(out: &mut Vec<u8>, s: &str) {
    u32v(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
}

fn encode_packet(ui: &Ui, commands: &[DrawCommand], scene: u32, packet: &mut Vec<u8>) {
    packet.clear();
    u32v(packet, 0x42484f4f);
    u32v(packet, 1);
    u32v(packet, commands.len() as u32);
    u32v(packet, u32::from(ui.needs_redraw()));
    packet.extend_from_slice(&ui.next_wakeup().unwrap_or(-1.0).to_le_bytes());
    u32v(packet, ui.semantics().len() as u32);
    u32v(packet, scene);
    for command in commands {
        match command {
            DrawCommand::Clear(c) => {
                u32v(packet, 0);
                color(packet, *c);
            }
            DrawCommand::Outline {
                bounds,
                radius,
                width,
                color: c,
            } => {
                u32v(packet, 19);
                rect(packet, *bounds);
                f32v(packet, *radius);
                f32v(packet, *width);
                color(packet, *c);
            }
            DrawCommand::Ellipse { bounds, color: c } => {
                u32v(packet, 18);
                rect(packet, *bounds);
                color(packet, *c);
            }
            DrawCommand::RoundedRect {
                bounds,
                radius,
                color: c,
            } => {
                u32v(packet, 1);
                rect(packet, *bounds);
                f32v(packet, *radius);
                color(packet, *c);
            }
            DrawCommand::Text {
                bounds,
                text,
                color: c,
                style,
            } => {
                u32v(packet, 9);
                rect(packet, *bounds);
                color(packet, *c);
                f32v(packet, style.size);
                let family_bits = match style.family {
                    FontFamily::Sans => 0_u32,
                    FontFamily::Serif => 1_u32,
                    FontFamily::Monospace => 2_u32,
                };
                u32v(
                    packet,
                    u32::from(style.weight)
                        | (u32::from(style.align) << 16)
                        | (u32::from(style.italic) << 24)
                        | (family_bits << 25),
                );
                f32v(packet, style.padding);
                f32v(packet, style.letter_spacing);
                string(packet, text);
            }
            DrawCommand::Shadow {
                bounds,
                radius,
                offset,
                blur,
                spread,
                color: c,
            } => {
                u32v(packet, 10);
                rect(packet, *bounds);
                f32v(packet, *radius);
                f32v(packet, offset[0]);
                f32v(packet, offset[1]);
                f32v(packet, *blur);
                f32v(packet, *spread);
                color(packet, *c);
            }
            DrawCommand::Ripple {
                bounds,
                corner_radius,
                center,
                radius,
                color: c,
                soft,
            } => {
                u32v(packet, if *soft { 5 } else { 11 });
                rect(packet, *bounds);
                f32v(packet, *corner_radius);
                f32v(packet, center[0]);
                f32v(packet, center[1]);
                f32v(packet, *radius);
                color(packet, *c);
            }
            DrawCommand::Line {
                from,
                to,
                width,
                color: c,
            } => {
                u32v(packet, 8);
                for v in [from[0], from[1], to[0], to[1], *width] {
                    f32v(packet, v);
                }
                color(packet, *c);
            }
            DrawCommand::Polyline {
                points,
                count,
                width,
                color: c,
            } => {
                u32v(packet, 17);
                for p in points {
                    f32v(packet, p[0]);
                    f32v(packet, p[1]);
                }
                u32v(packet, u32::from(*count));
                f32v(packet, *width);
                color(packet, *c);
            }
            DrawCommand::Arc {
                bounds,
                start,
                sweep,
                stroke,
                color: c,
                rounded,
            } => {
                u32v(packet, if *rounded { 7 } else { 12 });
                rect(packet, *bounds);
                f32v(packet, *start);
                f32v(packet, *sweep);
                f32v(packet, *stroke);
                color(packet, *c);
            }
            DrawCommand::Loading {
                bounds,
                sample,
                color: c,
            } => {
                u32v(packet, 6);
                color(packet, *c);
                u32v(packet, bhoo::loading::loading_curve_count(sample.shape) as u32);
                bhoo::loading::loading_cubics(*bounds, *sample, |curve| {
                    for p in curve {
                        f32v(packet, p);
                    }
                });
            }
            DrawCommand::Image {
                bounds,
                uv,
                radius,
                color: c,
                image,
            } => {
                u32v(packet, 16);
                rect(packet, *bounds);
                for p in uv {
                    f32v(packet, *p);
                }
                f32v(packet, *radius);
                color(packet, *c);
                u32v(packet, image.id());
                u32v(packet, image.width());
                u32v(packet, image.height());
                u32v(packet, image.pixels().as_ptr() as usize as u32);
            }
            DrawCommand::Shape {
                bounds,
                index,
                color: c,
                rotation,
            } => {
                let Some(shape) = bhoo::shape_data(*index as usize) else {
                    continue;
                };
                u32v(packet, 15);
                color(packet, *c);
                rect(packet, bhoo::shape_clip_bounds(*bounds, *rotation));
                u32v(packet, shape.contours.len().saturating_sub(1) as u32);
                let size = bounds.width.min(bounds.height);
                let x = bounds.x + (bounds.width - size) / 2.0;
                let y = bounds.y + (bounds.height - size) / 2.0;
                let (sin, cos) = rotation.sin_cos();
                for contour in shape.contours.windows(2) {
                    let points = &shape.points[contour[0] as usize..contour[1] as usize];
                    u32v(packet, points.len() as u32);
                    for p in points {
                        let px = p[0] - 0.5;
                        let py = p[1] - 0.5;
                        f32v(packet, x + (0.5 + px * cos - py * sin) * size);
                        f32v(packet, y + (0.5 + px * sin + py * cos) * size);
                    }
                }
            }
            DrawCommand::PushClip(b) => {
                u32v(packet, 3);
                rect(packet, *b);
            }
            DrawCommand::PopClip => u32v(packet, 4),
            DrawCommand::PushScaleY { origin, scale } => {
                u32v(packet, 13);
                f32v(packet, *origin);
                f32v(packet, *scale);
            }
            DrawCommand::PopScaleY => u32v(packet, 14),
            DrawCommand::PushTransform {
                origin,
                translate,
                scale,
                rotation_rad,
            } => {
                u32v(packet, 20);
                for v in [
                    origin[0],
                    origin[1],
                    translate[0],
                    translate[1],
                    scale[0],
                    scale[1],
                    *rotation_rad,
                ] {
                    f32v(packet, v);
                }
            }
            DrawCommand::PopTransform => u32v(packet, 21),
            DrawCommand::PushLayer { bounds, alpha } => {
                u32v(packet, 22);
                rect(packet, *bounds);
                f32v(packet, *alpha);
            }
            DrawCommand::PopLayer => u32v(packet, 23),
        }
    }
    for node in ui.semantics() {
        packet.extend_from_slice(&node.id.to_le_bytes());
        u32v(packet, node.part);
        u32v(packet, node.role as u32);
        rect(packet, node.bounds);
        u32v(
            packet,
            u32::from(node.enabled)
                | (u32::from(node.focused) << 1)
                | (u32::from(node.selected) << 2)
                | (u32::from(node.invalid) << 3)
                | (u32::from(node.filled) << 4)
                | (u32::from(node.required) << 5),
        );
        u32v(
            packet,
            match node.checked {
                None => 0,
                Some(CheckState::Unchecked) => 1,
                Some(CheckState::Checked) => 2,
                Some(CheckState::Mixed) => 3,
            },
        );
        f32v(packet, node.value.unwrap_or(f32::NAN));
        let range = node.range.unwrap_or([f32::NAN; 2]);
        f32v(packet, range[0]);
        f32v(packet, range[1]);
        string(packet, &node.label);
    }
}
