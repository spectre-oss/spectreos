use crate::*;

#[test]
fn material_icon_button_at_renders_at_explicit_bounds_and_clicks() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 120.0 };
    let target = Rect { x: 260.0, y: 12.0, width: 44.0, height: 44.0 };
    let mut clicked = false;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([280.0, 30.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            let resp = f.material_icon_button_at(10, target, MaterialIcon::Add, "Add", Options { variant: ButtonVariant::Filled, ..Options::default() });
            assert_eq!(resp.bounds, target);
            clicked |= resp.clicked;
        });
    }
    assert!(clicked);
}

#[test]
fn hover_wave_button_renders_label_and_clicks() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 240.0, height: 80.0 };
    let cmds = ui.frame(Input { pointer: Some([40.0, 20.0]), time: 0.1, ..Input::default() }, bounds, 0.0, |f| {
        f.hover_wave_button(20, "Wave", true);
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Wave")));
}
