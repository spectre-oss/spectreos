//! Standalone SpectreOS Settings Wayland application (`spectre-settings`).
//!
//! Launches natively inside Wisp (or any Wayland compositor) with `app_id = "org.spectreos.Settings"`,
//! reads real Linux hardware telemetry (battery, storage, kernel, IP/MAC), syncs preferences live
//! with Wisp via `~/.config/wisp/{settings,lock}.conf`, and invokes `org.freedesktop.Notifications`
//! over D-Bus for live notification testing.

use bhoo::{
    Color, Rect,
    window::{self, FrameAction, WindowOptions},
};
use spectre_settings::{
    DeviceRuntimeInfo, InstalledAppInfo, NotificationHistoryEntry, SettingsAction, SettingsApp,
    SettingsHostContext, SettingsPage, SystemState, detect_shiraz_root, load_shiraz_runtime_info,
    text_style, xdg_config_home,
};
use std::process::{Command, Stdio};

fn run_shiraz_cli(args: &[&str]) {
    if let Some(root) = detect_shiraz_root() {
        let py_src = root.join("src");
        let existing = std::env::var("PYTHONPATH").unwrap_or_default();
        let py_path = if existing.is_empty() {
            py_src.display().to_string()
        } else {
            format!("{}:{existing}", py_src.display())
        };
        let mut cmd = Command::new("python3");
        cmd.arg("-m")
            .arg("shiraz")
            .args(args)
            .env("PYTHONPATH", py_path)
            .env("SHIRAZ_HOME", &root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if cmd.spawn().is_ok() {
            return;
        }
    }
    let _ = Command::new("shiraz-ctl")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn build_installed_apps(sys: &SystemState) -> Vec<InstalledAppInfo> {
    let mut apps = vec![
        InstalledAppInfo {
            id: "org.spectreos.Settings.desktop".into(),
            name: "SpectreOS Settings".into(),
            package: "org.spectreos.Settings".into(),
            version: "0.1.0".into(),
            storage_mb: 12,
            builtin_idx: None,
            desktop_id: Some("org.spectreos.Settings.desktop".into()),
            is_android: false,
        },
        InstalledAppInfo {
            id: "org.gnome.Nautilus.desktop".into(),
            name: "Files".into(),
            package: "org.gnome.Nautilus".into(),
            version: "50.2".into(),
            storage_mb: 48,
            builtin_idx: None,
            desktop_id: Some("org.gnome.Nautilus.desktop".into()),
            is_android: false,
        },
        InstalledAppInfo {
            id: "org.gnome.Epiphany.desktop".into(),
            name: "Web".into(),
            package: "org.gnome.Epiphany".into(),
            version: "50.4".into(),
            storage_mb: 112,
            builtin_idx: None,
            desktop_id: Some("org.gnome.Epiphany.desktop".into()),
            is_android: false,
        },
        InstalledAppInfo {
            id: "org.gnome.Console.desktop".into(),
            name: "Console".into(),
            package: "org.gnome.Console".into(),
            version: "50.0".into(),
            storage_mb: 22,
            builtin_idx: None,
            desktop_id: Some("org.gnome.Console.desktop".into()),
            is_android: false,
        },
        InstalledAppInfo {
            id: "org.gnome.Calls.desktop".into(),
            name: "Calls".into(),
            package: "org.gnome.Calls".into(),
            version: "49.1".into(),
            storage_mb: 28,
            builtin_idx: None,
            desktop_id: Some("org.gnome.Calls.desktop".into()),
            is_android: false,
        },
    ];
    for aapp in &sys.shiraz.installed_apps {
        apps.push(InstalledAppInfo {
            id: aapp.desktop_id.clone(),
            name: aapp.label.clone(),
            package: aapp.package.clone(),
            version: aapp.version.clone(),
            storage_mb: 0,
            builtin_idx: None,
            desktop_id: Some(aapp.desktop_id.clone()),
            is_android: true,
        });
    }
    apps
}

fn send_freedesktop_test_notification(app: &str, title: &str, body: &str) {
    // Prefer standard `gdbus` call to `org.freedesktop.Notifications.Notify` (v1.3 spec)
    let _ = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.freedesktop.Notifications",
            "--object-path",
            "/org/freedesktop/Notifications",
            "--method",
            "org.freedesktop.Notifications.Notify",
            app,
            "0",
            "org.spectreos.Settings",
            title,
            body,
            "['default', 'Open Settings', 'dismiss', 'Dismiss']",
            "{'urgency': <byte 1>, 'desktop-entry': <'org.spectreos.Settings'>}",
            "6000",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn load_history_file() -> Vec<NotificationHistoryEntry> {
    let mut out = Vec::new();
    let Some(cfg) = xdg_config_home() else {
        return out;
    };
    let path = cfg.join("wisp/notifications-history.tsv");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return out;
    };
    for line in text.lines().rev() {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() >= 7 {
            out.push(NotificationHistoryEntry {
                id: cols[0].parse().unwrap_or(0),
                app: cols[1].to_string(),
                title: cols[2].to_string(),
                body: cols[3].to_string(),
                time_label: cols[4].to_string(),
                dismissed: cols[5] == "1",
                urgency: cols[6].parse().unwrap_or(1),
            });
        }
    }
    out
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut start_page = SettingsPage::Home;
    let args: Vec<String> = std::env::args().collect();
    for pair in args.windows(2) {
        if pair[0] == "--page" {
            start_page = SettingsPage::from_cli_arg(&pair[1]);
        }
    }

    let mut sys = SystemState::default();
    let device = DeviceRuntimeInfo::probe();
    let mut app = SettingsApp::default();
    if start_page != SettingsPage::Home {
        app.navigate(start_page, 0.0);
    }

    let mut installed_apps = build_installed_apps(&sys);
    let mut last_shiraz_poll = -10.0_f64;

    let mut history = load_history_file();
    let mut toast: Option<(String, f64)> = None;

    window::run(
        WindowOptions {
            title: "SpectreOS Settings".into(),
            app_id: Some("org.spectreos.Settings".into()),
            size: [400, 765],
            decorations: false,
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            ..WindowOptions::default()
        },
        move |f| {
            let now = f.time();
            if sys.reload_persisted_if_changed() {
                history = load_history_file();
            }
            if now - last_shiraz_poll >= 0.75 {
                last_shiraz_poll = now;
                let fresh = load_shiraz_runtime_info();
                if fresh != sys.shiraz {
                    sys.shiraz = fresh;
                    installed_apps = build_installed_apps(&sys);
                }
            }

            // Hardware / XF86Back / Escape back navigation inside standalone Settings
            if let Some(k) = f.input().key
                && k.pressed
                && matches!(k.key, bhoo::Key::Escape)
                && app.go_back(now)
            {
                return FrameAction::Redraw;
            }

            let vp = f.viewport();
            let pointer = f.input().pointer;
            let wall = chrono::Local::now();
            use chrono::Timelike;
            f.wake_at(now + 60.0 - f64::from(wall.second()) - f64::from(wall.nanosecond()) / 1e9);
            let ctx = SettingsHostContext {
                clock: &wall.format("%H:%M").to_string(),
                    date: &wall.format("%a, %b %-d").to_string(),
                apps: &installed_apps,
                active_notif_count: history.iter().filter(|h| !h.dismissed).count(),
                history: &history,
                device: &device,
                paint_app_icon: None,
            };

            let act = app.show(
                f,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: vp.width,
                    height: vp.height,
                },
                0x4C04_0000_0000_0000,
                &mut sys,
                pointer,
                &ctx,
            );

            match act {
                SettingsAction::ShowToast(msg) => {
                    toast = Some((msg, now + 2.4));
                }
                SettingsAction::PostTestNotification { app, title, body } => {
                    send_freedesktop_test_notification(&app, &title, &body);
                    toast = Some(("Test notification sent via D-Bus".into(), now + 2.4));
                }
                SettingsAction::ClearNotificationHistory => {
                    if let Some(cfg) = xdg_config_home() {
                        let _ = std::fs::remove_file(cfg.join("wisp/notifications-history.tsv"));
                    }
                    history.clear();
                    toast = Some(("Notification history cleared".into(), now + 2.0));
                }
                SettingsAction::OpenApp(idx) => {
                    if let Some(app_info) = installed_apps.get(idx)
                        && app_info.is_android
                    {
                        run_shiraz_cli(&["launch-app", &app_info.package]);
                        toast = Some((format!("Launching {}…", app_info.name), now + 2.4));
                    }
                }
                SettingsAction::ShirazLaunchPackage(pkg) => {
                    run_shiraz_cli(&["launch-app", &pkg]);
                    toast = Some((format!("Launching {pkg}…"), now + 2.4));
                }
                SettingsAction::ShirazStopPackage(pkg) => {
                    run_shiraz_cli(&["ctl", "stop", &pkg]);
                    toast = Some((format!("Stopping {pkg}"), now + 2.0));
                }
                SettingsAction::ShirazStopAll => {
                    run_shiraz_cli(&["ctl", "stop-all"]);
                    toast = Some(("Stopping all Android apps".into(), now + 2.0));
                }
                SettingsAction::ShirazRefresh => {
                    run_shiraz_cli(&["ctl", "refresh"]);
                    sys.shiraz = load_shiraz_runtime_info();
                    installed_apps = build_installed_apps(&sys);
                    toast = Some(("Optimizing Shiraz memory & AOT cache…".into(), now + 2.4));
                }
                _ => {}
            }

            if let Some((ref msg, until)) = toast {
                if now < until {
                    let p = sys.palette();
                    let tw = (vp.width - 48.0).min(300.0);
                    let tr = Rect {
                        x: (vp.width - tw) * 0.5,
                        y: vp.height - 64.0,
                        width: tw,
                        height: 40.0,
                    };
                    f.paint_rect(tr, 20.0, Color(28, 32, 42, 238));
                    f.styled_text(
                        tr,
                        msg,
                        Color(242, 246, 255, 255),
                        text_style(13.0, 500, 1),
                    );
                    let _ = p;
                } else {
                    toast = None;
                }
            }

            FrameAction::Wait
        },
    )
}
