use crate::*;

#[test]
fn preference_switch_row_toggles_and_renders_subtitle() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 340.0, height: 140.0 };
    let mut enabled_flag = false;
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.list_item_two_line(2100, Some(MaterialIcon::Settings), "Display", "Brightness & theme", true, true);
        f.preference_switch_row(2101, "Adaptive brightness", "Optimize for ambient light", &mut enabled_flag, true);
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Brightness & theme")));
    let sw = ui.semantics().iter().find(|n| n.id == 2101).unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([sw.x + 20.0, sw.y + 20.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            f.list_item_two_line(2100, Some(MaterialIcon::Settings), "Display", "Brightness & theme", true, true);
            f.preference_switch_row(2101, "Adaptive brightness", "Optimize for ambient light", &mut enabled_flag, true);
        });
    }
    assert!(enabled_flag);
}
