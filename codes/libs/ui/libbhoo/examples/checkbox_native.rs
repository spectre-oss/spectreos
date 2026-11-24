//! Native renderer fixture for checked and mixed checkbox marks.
use bhoo::{
    window::{self, FrameAction, WindowOptions},
    *,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|x| x == "--smoke-test");
    let mut frames = 0;
    let mut checked = CheckState::Checked;
    let mut mixed = CheckState::Mixed;
    window::run(
        WindowOptions {
            title: "Bhoo / checkbox marks".into(),
            size: [300, 180],
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            ..WindowOptions::default()
        },
        move |f| {
            frames += 1;
            f.checkbox(1, "Checked", &mut checked, true);
            f.checkbox(2, "Mixed", &mut mixed, true);
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
