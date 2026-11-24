use crate::*;

#[test]
fn accordion_single_mode_expands_one_panel_at_a_time() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 400.0 };
    let panels = [("General", "Display & sound"), ("Privacy", "Permissions")];
    let mut mask = 1u64;
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.accordion(200, &panels, &mut mask, false, 64.0, |f, i| {
            f.label(if i == 0 { "Body 0" } else { "Body 1" });
        });
    });
    let second = ui.semantics().iter().find(|n| n.id == 201).unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([second.x + 10.0, second.y + 10.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            f.accordion(200, &panels, &mut mask, false, 64.0, |f, i| {
                f.label(if i == 0 { "Body 0" } else { "Body 1" });
            });
        });
    }
    assert_eq!(mask, 2);
}
