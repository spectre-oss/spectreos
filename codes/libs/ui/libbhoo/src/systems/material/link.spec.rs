use super::*;

#[test]
fn link_emits_link_role_underline_and_handles_clicks() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 240.0, height: 80.0 };
    let mut clicked = false;
    let p = [20.0, 16.0];
    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
        f.link(5, "Learn more", true);
    });
    let cmds = ui.frame(Input { pointer: Some(p), released: true, time: 0.05, ..Input::default() }, bounds, 0.0, |f| {
        clicked = f.link(5, "Learn more", true).clicked;
    });
    assert!(clicked);
    assert!(ui.semantics().iter().any(|n| n.id == 5 && n.role == Role::Link));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, .. } if bounds.height == 1.0)));
}
