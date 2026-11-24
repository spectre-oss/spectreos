use super::*;

#[test]
fn alert_standard_filled_outlined_and_custom_action_work() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 200.0 };
    let cmds = ui.frame(Input { time: 0.0, ..Input::default() }, bounds, 8.0, |f| {
        f.alert(1, "Saved", AlertSeverity::Success);
        f.alert_with_options(
            10,
            "Offline",
            AlertSeverity::Warning,
            AlertOptions { variant: AlertVariant::Filled, action_label: Some("Retry"), dismissible: true },
        );
        f.alert_with_options(
            20,
            "Bordered",
            AlertSeverity::Info,
            AlertOptions { variant: AlertVariant::Outlined, action_label: None, dismissible: false },
        );
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Outline { .. })));
    let retry = ui
        .semantics()
        .iter()
        .find(|n| n.role == Role::Button && n.label.as_ref() == "Retry")
        .expect("Retry action button");
    let p = [retry.bounds.x + 12.0, retry.bounds.y + 12.0];
    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.1, ..Input::default() }, bounds, 8.0, |f| {
        f.alert(1, "Saved", AlertSeverity::Success);
        f.alert_with_options(
            10,
            "Offline",
            AlertSeverity::Warning,
            AlertOptions { variant: AlertVariant::Filled, action_label: Some("Retry"), dismissible: true },
        );
    });
    let mut action_clicked = false;
    let _ = ui.frame(Input { pointer: Some(p), released: true, time: 0.15, ..Input::default() }, bounds, 8.0, |f| {
        f.alert(1, "Saved", AlertSeverity::Success);
        let (_, act) = f.alert_with_options(
            10,
            "Offline",
            AlertSeverity::Warning,
            AlertOptions { variant: AlertVariant::Filled, action_label: Some("Retry"), dismissible: true },
        );
        action_clicked = act;
    });
    assert!(action_clicked);
}
