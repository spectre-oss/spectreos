use crate::*;

#[test]
fn toast_renders_before_deadline_and_clears_after_expiry() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 640.0 };
    let mut state = ToastState::default();
    state.show("Saved", 1.0, 2.0);
    let cmds = ui.frame(Input { time: 1.5, ..Input::default() }, bounds, 0.0, |f| {
        f.toast(&mut state, bounds, 32.0);
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Saved")));
    assert_eq!(ui.next_wakeup(), Some(3.0));

    let expired = ui.frame(Input { time: 3.1, ..Input::default() }, bounds, 0.0, |f| {
        f.toast(&mut state, bounds, 32.0);
    });
    assert!(expired.iter().all(|c| !matches!(c, DrawCommand::Text { .. })));
    assert!(state.message.is_none());
}

#[test]
fn toast_with_options_applies_custom_background_and_foreground() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 200.0 };
    let mut state = ToastState::default();
    state.show("Custom", 0.0, 2.0);
    let bg = Color(12, 80, 40, 255);
    let fg = Color(250, 250, 250, 255);
    let cmds = ui.frame(Input { time: 0.5, ..Input::default() }, bounds, 0.0, |f| {
        f.toast_with_options(
            &mut state,
            bounds,
            16.0,
            ToastOptions { background: Some(bg), foreground: Some(fg), corner_radius: 12.0 },
        );
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color, radius, .. } if *color == bg && (*radius - 12.0).abs() < 0.01)));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { color, .. } if *color == fg)));
}
