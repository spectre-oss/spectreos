//! Native Wayland + GLES capture for the pure Android XML Gmail screens
//! (`android_gmail_inbox.rs`, `android_gmail_detail.rs`, `android_gmail_compose.rs`)
//! under `AndroidTheme::Material3Light` at `585 x 1266` (`density = 1.5`, i.e. `390 x 844` dp).
//!
//! Select screen via `BHOO_GMAIL_SCREEN=inbox|detail|compose` (default: `inbox`),
//! or via CLI flags `--inbox`, `--detail`, `--compose`.
//! Pass `--smoke-test` with `BHOO_CAPTURE=/path/to/out.ppm` to render and capture a frame.
use bhoo::{
    android_xml::{AndroidTheme, LayoutState, Node, Visibility},
    window::{self, FrameAction, WindowOptions},
    *,
};

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/android_gmail_inbox.rs"]
mod inbox;

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/android_gmail_detail.rs"]
mod detail;

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/android_gmail_compose.rs"]
mod compose;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Inbox,
    Detail,
    Compose,
}

fn select_screen() -> Screen {
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--inbox" => return Screen::Inbox,
            "--detail" => return Screen::Detail,
            "--compose" => return Screen::Compose,
            _ => {}
        }
    }
    match std::env::var("BHOO_GMAIL_SCREEN")
        .ok()
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("detail") => Screen::Detail,
        Some("compose") => Screen::Compose,
        _ => Screen::Inbox,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|x| x == "--smoke-test");
    let screen = select_screen();

    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    state.density = 1.5;

    inbox::install_drawables(&mut state);
    detail::install_drawables(&mut state);
    compose::install_drawables(&mut state);

    state.selected_index.insert("bottom_nav".into(), 0);
    state.selected_index.insert("nav_drawer".into(), 0);
    for i in [0usize, 2, 3, 4, 5, 6] {
        state
            .image_sources
            .insert(format!("star_{i}"), "ic_star_outline".into());
    }
    state
        .image_sources
        .insert("detail_star".into(), "ic_star_outline".into());
    state
        .visibility
        .insert("inbox_swipe_refresh".into(), Visibility::Visible);
    state
        .visibility
        .insert("fab_compose".into(), Visibility::Visible);
    state
        .visibility
        .insert("tab_chat_view".into(), Visibility::Gone);
    state
        .visibility
        .insert("tab_meet_view".into(), Visibility::Gone);
    state.drawer_open.insert("drawer_layout".into(), false);
    state.offsets.insert("detail_scroll".into(), 0.0);
    state.text.insert(
        "compose_body".into(),
        "Hey Sam,\n\nHow's it going? I attached some pictures from my recent trip to Japan! Can't wait to visit you soon.\n\nBest,\nAlex".into(),
    );

    let (title, layout): (&'static str, &'static Node) = match screen {
        Screen::Inbox => ("Bhoo / Gmail XML (Inbox)", &inbox::LAYOUT),
        Screen::Detail => ("Bhoo / Gmail XML (Detail)", &detail::LAYOUT),
        Screen::Compose => ("Bhoo / Gmail XML (Compose)", &compose::LAYOUT),
    };

    let mut frames = 0;
    window::run(
        WindowOptions {
            title: title.into(),
            size: [585, 1266],
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            ..WindowOptions::default()
        },
        move |f| {
            frames += 1;
            layout.render(
                f,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 585.0,
                    height: 1266.0,
                },
                &mut state,
            );
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
