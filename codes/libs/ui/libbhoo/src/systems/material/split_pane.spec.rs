use crate::*;

#[test]
fn split_pane_renders_both_panes_and_clamps_ratio() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 400.0, height: 200.0 };
    let mut ratio = 0.01;
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.split_pane(
            900,
            200.0,
            &mut ratio,
            SplitPaneOptions::default(),
            |f| f.label("Left"),
            |f| f.label("Right"),
        );
    });
    assert!((ratio - 0.2).abs() < 1e-4);
    assert!(ui.semantics().iter().any(|n| n.id == 900));
}
