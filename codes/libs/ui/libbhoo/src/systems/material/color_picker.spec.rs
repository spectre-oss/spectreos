use crate::*;

#[test]
fn hue_to_color_maps_primary_angles() {
    assert_eq!(hue_to_color(0.0), Color(255, 0, 0, 255));
    assert_eq!(hue_to_color(120.0), Color(0, 255, 0, 255));
    assert_eq!(hue_to_color(240.0), Color(0, 0, 255, 255));
}

#[test]
fn swatch_picker_selects_clicked_swatch() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 120.0 };
    let swatches = [
        ("Emerald", Color(43, 106, 77, 255)),
        ("Ocean", Color(40, 96, 178, 255)),
        ("Amber", Color(210, 140, 30, 255)),
    ];
    let mut sel = 0usize;
    ui.frame(Input::default(), bounds, 8.0, |f| {
        f.swatch_picker(1300, &swatches, &mut sel, 3, true);
    });
    let ocean = ui.semantics().iter().find(|n| n.label.as_ref() == "Ocean").unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([ocean.x + 12.0, ocean.y + 12.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 8.0, |f| {
            f.swatch_picker(1300, &swatches, &mut sel, 3, true);
        });
    }
    assert_eq!(sel, 1);
}
