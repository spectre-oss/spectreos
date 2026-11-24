use crate::*;

#[test]
fn time_picker_labels_format_12_hour_clock_correctly() {
    assert_eq!(time_picker_label(0), "12:00 AM");
    assert_eq!(time_picker_label(1), "12:30 AM");
    assert_eq!(time_picker_label(18), "9:00 AM");
    assert_eq!(time_picker_label(24), "12:00 PM");
    assert_eq!(time_picker_label(25), "12:30 PM");
    assert_eq!(time_picker_label(47), "11:30 PM");
}

#[test]
fn time_picker_opens_popover_on_trigger_click() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 420.0 };
    let mut state = TimePickerState::default();
    ui.frame(Input::default(), bounds, 8.0, |f| {
        f.time_picker(500, "Start time", &mut state, true);
    });
    let trig = ui.semantics().iter().find(|n| n.id == 500).unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([trig.x + 10.0, trig.y + 10.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 8.0, |f| {
            f.time_picker(500, "Start time", &mut state, true);
        });
    }
    assert!(state.overlay.open);
}

#[test]
fn time_picker_supports_24h_formatting_and_dial_mode() {
    assert_eq!(time_picker_label_24h(0, 30), "00:00");
    assert_eq!(time_picker_label_24h(27, 30), "13:30");

    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 420.0 };
    let mut state = TimePickerState::default();
    state.overlay.open = true;
    state.anchor = Rect { x: 16.0, y: 16.0, width: 200.0, height: 56.0 };
    let cmds = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.time_picker_with_options(
            50,
            "Alarm",
            &mut state,
            true,
            TimePickerOptions { mode: TimePickerMode::Dial, use_24h: true },
        );
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Line { .. })));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "09:00")));
}
