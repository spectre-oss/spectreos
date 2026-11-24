use crate::*;

#[test]
fn spin_box_increments_decrements_and_clamps() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 100.0 };
    let mut val = 5.0;
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.spin_box(1200, "Count", &mut val, 0.0..=10.0, 2.0, true);
    });
    let inc = ui.semantics().iter().find(|n| n.id == 1202).unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([inc.x + 8.0, inc.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            f.spin_box(1200, "Count", &mut val, 0.0..=10.0, 2.0, true);
        });
    }
    assert_eq!(val, 7.0);
}
