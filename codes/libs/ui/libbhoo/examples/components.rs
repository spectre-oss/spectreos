//! Interactive first-wave component gallery. Later catalog families remain in the TDD.
use bhoo::window::{self, FrameAction, WindowOptions};
use bhoo::{ButtonVariant, CheckState, ListState, MotionPreference, Options, Rect};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|x| x == "--smoke-test");
    let mut frames = 0;
    let mut clicks = 0;
    let mut checked = CheckState::Mixed;
    let mut enabled = true;
    let mut notifications = false;
    let mut volume = 40.0;
    let mut tab = 0;
    let mut list = ListState::default();
    let options = WindowOptions {
        title: "Bhoo components".into(),
        size: [720, 640],
        motion: if std::env::args().any(|x| x == "--reduced-motion") {
            MotionPreference::Reduced
        } else {
            MotionPreference::Full
        },
        capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
        ..WindowOptions::default()
    };
    window::run(options, move |frame| {
        frames += 1;
        frame.label("Bhoo / component foundations");
        frame.label("Tab to focus; Space / Enter to activate; arrows adjust sliders.");
        let area = frame.available();
        let width = (area.width - 24.0) * 0.5;
        frame.within(Rect { width, ..area }, |f| {
            let response = f.button_with(
                1,
                "Summon a ghost",
                Options {
                    variant: ButtonVariant::Filled,
                    ..Options::default()
                },
            );
            if response.clicked {
                clicks += 1;
            }
            f.tooltip(1001, response, "One ghost per activation");
            f.label(format!("Ghosts summoned: {clicks}"));
            f.checkbox(2, "Tri-state checkbox", &mut checked, true);
            f.switch(3, "Enable controls", &mut enabled, true);
            f.with_enabled(enabled, |f| {
                f.switch(4, "Notifications", &mut notifications, true);
                f.label(format!("Volume: {:.0}%", volume));
                f.slider(5, "Volume", &mut volume, 0.0..=100.0, 1.0, true);
                f.button_with(
                    6,
                    "Outlined action",
                    Options {
                        variant: ButtonVariant::Outlined,
                        ..Options::default()
                    },
                );
                f.chip(7, "Filter chip", notifications, true);
            });
            f.divider();
            f.label("Determinate progress");
            f.progress(8, "Volume", Some(volume / 100.0));
            f.badge("3");
        });
        frame.within(
            Rect {
                x: area.x + width + 24.0,
                width,
                ..area
            },
            |f| {
                f.tabs(20, &["Apps", "Recent"], &mut tab, true);
                f.label("100,000 rows / visible rows only");
                f.virtual_list(21, 320.0, 100_000, &mut list, |f, index, selected| {
                    f.list_item(
                        10_000 + index as u64,
                        &format!("Application {}", index + 1),
                        selected,
                        true,
                    )
                });
                f.button_with(
                    22,
                    "Disabled action",
                    Options {
                        enabled: false,
                        ..Options::default()
                    },
                );
                f.label("Escape closes this window");
            },
        );
        if smoke && frames >= 4 {
            FrameAction::Exit
        } else if smoke {
            FrameAction::Redraw
        } else {
            FrameAction::Wait
        }
    })
}
