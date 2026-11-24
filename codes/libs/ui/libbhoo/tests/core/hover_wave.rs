use bhoo::*;
const B: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 200.0,
    height: 40.0,
};
fn sample(ui: &mut Ui, time: f64, pointer: Option<[f32; 2]>, pressed: bool) -> (f32, bool) {
    let mut progress = 0.0;
    let commands = ui.frame(
        Input {
            time,
            pointer,
            pressed,
            ..Input::default()
        },
        B,
        0.0,
        |f| {
            let r = f.target(1, B, "Wave", true, Role::Button);
            progress = f.wave(r, 20.0, Color(255, 255, 255, 255));
        },
    );
    (
        progress,
        commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Ripple { soft: false, .. })),
    )
}
#[test]
fn hover_wave_expands_linearly_and_disappears_when_pressed_or_left() {
    let mut ui = Ui::default();
    assert_eq!(sample(&mut ui, 0.0, None, false), (0.0, false));
    assert_eq!(sample(&mut ui, 1.0, Some([50.0, 20.0]), false), (0.0, true));
    let p = sample(&mut ui, 1.1, Some([50.0, 20.0]), false);
    assert!((p.0 - 0.5).abs() < 0.001 && p.1);
    assert_eq!(
        sample(&mut ui, 1.201, Some([50.0, 20.0]), false),
        (1.0, true)
    );
    assert_eq!(sample(&mut ui, 1.3, Some([50.0, 20.0]), true), (0.0, false));
    assert_eq!(sample(&mut ui, 1.4, None, false), (0.0, false));
}
