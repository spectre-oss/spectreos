//! Deterministic GLES coverage samples plus real interactive controls.
use bhoo::{
    window::{self, FrameAction, WindowOptions},
    *,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let smoke = args.iter().any(|a| a == "--smoke-test");
    let reduced = args.iter().any(|a| a == "--reduced-motion");
    let mut states = [LoadingState::default(), LoadingState::default()];
    let mut frames = 0;
    let image = Image::rgba(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ],
    )
    .unwrap();
    window::run(
        WindowOptions {
            title: "Bhoo / effects".into(),
            size: [440, 360],
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            motion: if reduced {
                MotionPreference::Reduced
            } else {
                MotionPreference::Full
            },
            ..WindowOptions::default()
        },
        move |f| {
            frames += 1;
            f.label("Bhoo effects");
            let t = f.theme();
            let bounds = Rect {
                x: 24.0,
                y: 72.0,
                width: 240.0,
                height: 72.0,
            };
            f.paint_rect(bounds, 24.0, t.control);
            f.paint_ink(
                bounds,
                24.0,
                [100.0, 108.0],
                60.0,
                Color(255, 255, 255, 102),
            );
            f.within(
                Rect {
                    x: 24.0,
                    y: 200.0,
                    width: 48.0,
                    height: 48.0,
                },
                |f| f.loading_indicator(10, "Determinate", &mut states[0], Some(0.4), true),
            );
            f.within(
                Rect {
                    x: 120.0,
                    y: 200.0,
                    width: 48.0,
                    height: 48.0,
                },
                |f| f.loading_indicator(11, "Busy", &mut states[1], None, false),
            );
            f.within(
                Rect {
                    x: 220.0,
                    y: 200.0,
                    width: 48.0,
                    height: 48.0,
                },
                |f| f.progress_circular(12, "Half", Some(0.5)),
            );
            f.paint_image(
                Rect {
                    x: 300.0,
                    y: 72.0,
                    width: 64.0,
                    height: 64.0,
                },
                &image,
                ImageFit::Cover,
                8.0,
            );
            for (x, name) in [(300.0, "ghost-ish"), (370.0, "wavy-circle")] {
                let shape = bhoo::beercss_shapes::SHAPES
                    .iter()
                    .position(|s| s.name == name)
                    .unwrap();
                f.paint_shape(
                    Rect {
                        x,
                        y: 200.0,
                        width: 48.0,
                        height: 48.0,
                    },
                    shape,
                    t.primary,
                );
            }
            f.within(
                Rect {
                    x: 24.0,
                    y: 285.0,
                    width: 260.0,
                    height: 48.0,
                },
                |f| {
                    f.button(20, "Press for a real ripple");
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
