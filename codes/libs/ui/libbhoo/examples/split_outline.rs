//! Headless renderer sample for a transparent outlined split button.
use bhoo::{
    window::{self, FrameAction, WindowOptions},
    *,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|arg| arg == "--smoke-test");
    let mut frames = 0;
    window::run(
        WindowOptions {
            title: "Bhoo / split outline".into(),
            size: [300, 180],
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            ..WindowOptions::default()
        },
        move |f| {
            frames += 1;
            f.paint_rect(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 300.0,
                    height: 180.0,
                },
                0.0,
                Color(41, 110, 75, 255),
            );
            f.split_button_with(
                10,
                Rect {
                    x: 30.0,
                    y: 60.0,
                    width: 200.0,
                    height: 56.0,
                },
                "Save",
                "More",
                SplitButtonOptions {
                    size: SplitButtonSize::Medium,
                    variant: ButtonVariant::Outlined,
                    ..SplitButtonOptions::default()
                },
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
