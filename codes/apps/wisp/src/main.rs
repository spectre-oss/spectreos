//! Unified entrypoint for Wisp — SpectreOS Wayland Compositor & System UI Launcher.
//!
//! Modes:
//! - Default (with `--features compositor`): runs directly on DRM/KMS + libinput as the
//!   native Wayland compositor & mobile shell (replaces Phoc/Phosh).
//! - `--window` (or built with `--features wayland` only): runs inside a nested desktop
//!   Wayland preview window.
//!
//! Environment overrides:
//! - `WISP_SCALE` (compositor logical scale, default `2.7`)
//! - `WISP_UNLOCKED=1` (skip lock screen on boot)
//! - `WISP_SCREEN=home|drawer|shade|qs|power|volume|overview|split|freeform|settings|lock`
//! - `WISP_W` / `WISP_H` (window preview dimensions, default `411 x 846`)
//! - `BHOO_FONT=/path/to/Roboto-Regular.ttf`
//! - `BHOO_CAPTURE=/path/to/out.ppm` (optional screenshot capture in window mode)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--animation-scale") {
        let mut settings = wisp::SystemState::default();
        if let Some(value) = std::env::args().nth(2) {
            let scale: f32 = value.parse()?;
            settings.anim_scale_idx = wisp::SystemState::ANIMATION_SCALES
                .iter().position(|&option| option == scale)
                .ok_or("Expected an animation scale of 0, 0.5, 1, 1.5, 2, 5, or 10")?;
            settings.save_persisted_settings();
            if wisp::SystemState::default().anim_scale() != scale {
                return Err("Could not save the Wisp animation scale".into());
            }
        }
        println!("Wisp animation scale: {}x", settings.anim_scale());
        return Ok(());
    }

    #[allow(unused_variables)]
    let force_window = std::env::args().any(|a| matches!(a.as_str(), "--window" | "--smoke-test" | "--notification-preview"));

    #[cfg(feature = "compositor")]
    if !force_window {
        return wisp::compositor::run();
    }

    #[cfg(feature = "wayland")]
    {
        run_preview_window()
    }

    #[cfg(not(feature = "wayland"))]
    {
        let _ = force_window;
        Err("Wisp was built without `--features compositor` or `--features wayland`.".into())
    }
}

#[cfg(feature = "wayland")]
fn run_preview_window() -> Result<(), Box<dyn std::error::Error>> {
    use bhoo::window::{self, FrameAction, WindowOptions};
    use wisp::{NoopEmbeddedApps, WispShell};

    if std::env::args().any(|arg| arg == "--notification-preview") {
        let capture = std::env::var_os("BHOO_CAPTURE").map(Into::into);
        let capturing = capture.is_some();
        let expanded = std::env::args().any(|arg| arg == "--expanded");
        let font = std::env::var_os("BHOO_FONT").map(std::path::PathBuf::from).or_else(|| {
            wisp::freedesktop::xdg_data_home()
                .map(|data| data.join("wisp/notification-preview/Roboto-Regular.ttf"))
                .filter(|path| path.is_file())
        });
        let mut shell = WispShell::<NoopEmbeddedApps>::default();
        shell.set_native_compositor(true);
        return window::run(
            WindowOptions {
                title: "Wisp · Notification preview".into(),
                size: [400, 168],
                font,
                capture,
                ..WindowOptions::default()
            },
            move |f| {
                shell.show_notification_preview(f, capturing, expanded);
                if capturing { FrameAction::Exit } else { FrameAction::Wait }
            },
        );
    }

    let smoke = std::env::args().any(|x| x == "--smoke-test");
    let preset = std::env::var("WISP_SCREEN")
        .or_else(|_| std::env::var("BHOO_LAUNCHER_SCREEN"))
        .unwrap_or_else(|_| "home".into());
    let win_w: u32 = std::env::var("WISP_W")
        .or_else(|_| std::env::var("BHOO_CAPTURE_W"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(411);
    let win_h: u32 = std::env::var("WISP_H")
        .or_else(|_| std::env::var("BHOO_CAPTURE_H"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(846);

    let mut shell = Box::new(WispShell::<NoopEmbeddedApps>::default());
    shell.set_capture_preset(&preset);
    let mut compose_body = String::new();
    let mut frames = 0;

    window::run(
        WindowOptions {
            title: "Wisp · SpectreOS Launcher".into(),
            size: [win_w, win_h],
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            ..WindowOptions::default()
        },
        move |f| {
            frames += 1;
            let vp = f.viewport();
            let input = f.input();
            shell.show(f, vp.width, vp.height, input, &mut compose_body);
            if smoke && frames >= 4 {
                FrameAction::Exit
            } else if smoke {
                FrameAction::Redraw
            } else {
                FrameAction::Wait
            }
        },
    )
}
