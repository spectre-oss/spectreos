use bhoo::window::{self, FrameAction, WindowOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke_test = std::env::args().any(|arg| arg == "--smoke-test");
    let mut clicks = 0;
    let mut frames = 0;
    let options = WindowOptions {
        title: "Bhoo · SpectreOS".into(),
        size: [440, 360],
        capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
        ..WindowOptions::default()
    };
    window::run(options, move |frame| {
        frames += 1;
        frame.label("Bhoo / SpectreOS");
        frame.label("Wayland + OpenGL ES 3");
        if frame.button(1, "Say bhoo!").clicked {
            clicks += 1;
        }
        frame.label(format!("Ghosts summoned: {clicks}"));
        if frame.button(2, "Close window").clicked || (smoke_test && frames >= 4) {
            FrameAction::Exit
        } else if smoke_test {
            FrameAction::Redraw
        } else {
            FrameAction::Wait
        }
    })
}
