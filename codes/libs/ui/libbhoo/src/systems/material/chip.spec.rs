use super::*;

fn bounds() -> Rect {
    Rect { x: 0.0, y: 0.0, width: 240.0, height: 100.0 }
}

#[test]
fn chip_emits_selected_semantic_and_responds_to_click() {
    let mut ui = Ui::default();
    let mut clicked = false;
    let p = [40.0, 24.0];
    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.0, ..Input::default() }, bounds(), 0.0, |f| {
        f.chip(10, "Unread", true, true);
    });
    let _ = ui.frame(Input { pointer: Some(p), released: true, time: 0.05, ..Input::default() }, bounds(), 0.0, |f| {
        clicked = f.chip(10, "Unread", true, true).clicked;
    });
    assert!(clicked);
    assert!(ui.semantics().iter().any(|n| n.id == 10 && n.selected));
}

#[test]
fn chip_with_options_routes_close_button_independently_from_body() {
    let mut ui = Ui::default();
    let opts = ChipOptions {
        kind: ChipKind::Input,
        selected: true,
        elevated: true,
        show_close: true,
        enabled: true,
    };
    let close_pt = [228.0, 24.0];
    let mut body_clicked = false;
    let mut close_clicked = false;
    let _ = ui.frame(Input { pointer: Some(close_pt), pressed: true, time: 0.0, ..Input::default() }, bounds(), 0.0, |f| {
        f.chip_with_options(20, "Tag", opts);
    });
    let _ = ui.frame(Input { pointer: Some(close_pt), released: true, time: 0.05, ..Input::default() }, bounds(), 0.0, |f| {
        let (r, c) = f.chip_with_options(20, "Tag", opts);
        body_clicked = r.clicked;
        close_clicked = c;
    });
    assert!(!body_clicked);
    assert!(close_clicked);
}
