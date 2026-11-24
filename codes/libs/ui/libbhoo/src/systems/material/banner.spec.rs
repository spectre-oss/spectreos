use crate::*;

#[test]
fn banner_reveals_message_and_handles_action_click() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 240.0 };
    let mut clicked = None;
    ui.frame(Input::default(), bounds, 8.0, |f| {
        clicked = f.banner(1000, true, "Offline mode active", &["Dismiss", "Retry"]);
    });
    let retry = ui.semantics().iter().find(|n| n.label.as_ref() == "Retry").unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([retry.x + 8.0, retry.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 8.0, |f| {
            if let Some(idx) = f.banner(1000, true, "Offline mode active", &["Dismiss", "Retry"]) {
                clicked = Some(idx);
            }
        });
    }
    assert_eq!(clicked, Some(1));
}

#[test]
fn banner_with_leading_icon_paints_icon_shape() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 140.0 };
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.banner_with_options(
            50,
            true,
            "Sync paused",
            &["Dismiss", "Resume"],
            BannerOptions { leading_icon: Some(MaterialIcon::Settings) },
        );
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Shape { .. })));
}
