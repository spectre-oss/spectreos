//! Native Wayland + GLES capture fixture for `examples/android_xml/screen.rs`
//! under `AndroidTheme::MaterialLight` at `720 x 1226` (`density = 1.125`).
use bhoo::{
    android_xml::{AndroidTheme, LayoutState},
    window::{self, FrameAction, WindowOptions},
    *,
};

#[allow(dead_code)]
#[path = "android_xml/screen.rs"]
mod screen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|x| x == "--smoke-test");
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::MaterialLight;
    state.density = 1.125;
    let mut frames = 0;
    window::run(
        WindowOptions {
            title: "Bhoo / android screen capture".into(),
            size: [720, 1226],
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            ..WindowOptions::default()
        },
        move |f| {
            frames += 1;
            screen::LAYOUT.render(
                f,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 720.0,
                    height: 1226.0,
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
