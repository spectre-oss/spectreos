use super::*;

#[test]
fn plain_tooltip_schedules_wakeup_and_appears_after_hover_delay() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 160.0 };
    let p = [40.0, 20.0];
    let _ = ui.frame(Input { pointer: Some(p), time: 1.0, ..Input::default() }, bounds, 0.0, |f| {
        let r = f.button(1, "Save");
        f.tooltip(2, r, "Save document");
    });
    assert!(ui.next_wakeup().is_some());
    assert!(!ui.semantics().iter().any(|n| n.role == Role::Tooltip));

    let _ = ui.frame(Input { pointer: Some(p), time: 1.6, ..Input::default() }, bounds, 0.0, |f| {
        let r = f.button(1, "Save");
        f.tooltip(2, r, "Save document");
    });
    assert!(ui.semantics().iter().any(|n| n.role == Role::Tooltip && n.label.as_ref() == "Save document"));
}

#[test]
fn rich_tooltip_shows_after_delay_and_handles_action_click() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 260.0 };
    let p = [40.0, 20.0];
    let _ = ui.frame(Input { pointer: Some(p), time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
        let r = f.button(10, "Info");
        f.rich_tooltip(11, r, "Title", "Detailed explanation", Some("Learn more"));
    });
    let _ = ui.frame(Input { pointer: Some(p), time: 0.6, ..Input::default() }, bounds, 0.0, |f| {
        let r = f.button(10, "Info");
        f.rich_tooltip(11, r, "Title", "Detailed explanation", Some("Learn more"));
    });
    let action_node = ui
        .semantics()
        .iter()
        .find(|n| n.role == Role::Button && n.label.as_ref() == "Learn more")
        .expect("rich tooltip action semantic");
    let ap = [action_node.bounds.x + 12.0, action_node.bounds.y + 12.0];
    let _ = ui.frame(Input { pointer: Some(ap), pressed: true, time: 0.7, ..Input::default() }, bounds, 0.0, |f| {
        let r = f.button(10, "Info");
        f.rich_tooltip(11, r, "Title", "Detailed explanation", Some("Learn more"));
    });
    let mut clicked = false;
    let _ = ui.frame(Input { pointer: Some(ap), released: true, time: 0.75, ..Input::default() }, bounds, 0.0, |f| {
        let r = f.button(10, "Info");
        clicked = f.rich_tooltip(11, r, "Title", "Detailed explanation", Some("Learn more"));
    });
    assert!(clicked);
}
