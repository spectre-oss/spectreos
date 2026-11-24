use super::*;

#[test]
fn segmented_single_and_multiple_selection_update_bitmask() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 300.0, height: 80.0 };
    let labels = ["Day", "Week", "Month"];
    let mut single = 1u32;
    let second = [150.0, 20.0];
    let _ = ui.frame(Input { pointer: Some(second), pressed: true, time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
        f.segmented(1, &labels, &mut single, false, true);
    });
    let _ = ui.frame(Input { pointer: Some(second), released: true, time: 0.05, ..Input::default() }, bounds, 0.0, |f| {
        let r = f.segmented(1, &labels, &mut single, false, true);
        assert!(r.changed);
    });
    assert_eq!(single, 0b10);

    let mut multi = 0b01u32;
    let third = [250.0, 20.0];
    let _ = ui.frame(Input { pointer: Some(third), pressed: true, time: 0.2, ..Input::default() }, bounds, 0.0, |f| {
        f.segmented(2, &labels, &mut multi, true, true);
    });
    let _ = ui.frame(Input { pointer: Some(third), released: true, time: 0.25, ..Input::default() }, bounds, 0.0, |f| {
        f.segmented(2, &labels, &mut multi, true, true);
    });
    assert_eq!(multi, 0b101);
}
