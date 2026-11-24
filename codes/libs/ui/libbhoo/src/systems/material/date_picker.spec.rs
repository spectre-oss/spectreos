use crate::*;

#[test]
fn gregorian_month_days_and_weekday_handle_leap_years() {
    assert_eq!(month_days(2024, 2), 29);
    assert_eq!(month_days(2025, 2), 28);
    assert_eq!(month_days(1900, 2), 28);
    assert_eq!(month_days(2000, 2), 29);
    assert_eq!(month_days(2026, 11), 30);
    // 2026-11-09 is Monday -> 0
    assert_eq!(weekday(2026, 11, 9), 0);
}

#[test]
fn calendar_grid_selects_date_and_range() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 420.0 };
    let mut state = DatePickerState {
        year: 2026,
        month: 11,
        selected: None,
        range_end: None,
        today: [2026, 11, 9],
        ..DatePickerState::default()
    };
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.calendar_grid(1000, &mut state, true);
    });
    let day10 = ui.semantics().iter().find(|n| n.label.as_ref() == "10").unwrap().bounds;
    let day15 = ui.semantics().iter().find(|n| n.label.as_ref() == "15").unwrap().bounds;

    // Click day 10 (start)
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([day10.x + 4.0, day10.y + 4.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            f.calendar_grid(1000, &mut state, true);
        });
    }
    assert_eq!(state.selected, Some([2026, 11, 10]));
    assert!(state.picking_end);

    // Click day 15 (end)
    for (p, r, t) in [(true, false, 0.3), (false, true, 0.4)] {
        ui.frame(Input { pointer: Some([day15.x + 4.0, day15.y + 4.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            f.calendar_grid(1000, &mut state, true);
        });
    }
    assert_eq!(state.range_end, Some([2026, 11, 15]));
    assert!(!state.picking_end);
    assert_eq!(state.formatted_range(), "11/10/2026 – 11/15/2026");
}

#[test]
fn date_picker_disabled_dates_text_input_mode_and_dual_range_work() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 640.0, height: 420.0 };
    let mut state = DatePickerState::default();
    let opts = DatePickerOptions {
        text_input_mode: true,
        min_date: Some([2026, 11, 5]),
        max_date: Some([2026, 11, 20]),
    };
    assert!(opts.is_disabled([2026, 11, 4]));
    assert!(!opts.is_disabled([2026, 11, 10]));
    assert!(opts.is_disabled([2026, 11, 21]));
    let cmds = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.calendar_grid_with_options(50, &mut state, false, opts, |_| false);
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "11/9/2026")));

    let dual_opts = DatePickerOptions::default();
    let cmds2 = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.date_range_picker_dual(80, &mut state, dual_opts, |_| false);
    });
    assert!(cmds2.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref().contains("NOV 2026"))));
    assert!(cmds2.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref().contains("DEC 2026"))));
}
