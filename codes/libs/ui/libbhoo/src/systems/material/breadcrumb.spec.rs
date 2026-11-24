use crate::*;

#[test]
fn breadcrumb_activates_clicked_ancestor() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 100.0 };
    let path = ["Home", "Settings", "Display"];
    let mut clicked = None;
    ui.frame(Input::default(), bounds, 0.0, |f| {
        clicked = f.breadcrumb(1100, &path, None);
    });
    let home = ui.semantics().iter().find(|n| n.label.as_ref() == "Home").unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([home.x + 8.0, home.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            if let Some(i) = f.breadcrumb(1100, &path, None) {
                clicked = Some(i);
            }
        });
    }
    assert_eq!(clicked, Some(0));
}
