use super::*;

fn bounds() -> Rect {
    Rect { x: 0.0, y: 0.0, width: 320.0, height: 120.0 }
}

#[test]
fn tristate_checkbox_cycles_unchecked_checked_mixed_unchecked() {
    let mut ui = Ui::default();
    let mut state = CheckState::Unchecked;
    let p = [40.0, 20.0];
    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.0, ..Input::default() }, bounds(), 0.0, |f| {
        f.checkbox(1, "Select all", &mut state, true);
    });
    let _ = ui.frame(Input { pointer: Some(p), released: true, time: 0.05, ..Input::default() }, bounds(), 0.0, |f| {
        let r = f.checkbox(1, "Select all", &mut state, true);
        assert!(r.changed);
    });
    assert_eq!(state, CheckState::Checked);

    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.2, ..Input::default() }, bounds(), 0.0, |f| {
        f.checkbox(1, "Select all", &mut state, true);
    });
    let _ = ui.frame(Input { pointer: Some(p), released: true, time: 0.25, ..Input::default() }, bounds(), 0.0, |f| {
        f.checkbox(1, "Select all", &mut state, true);
    });
    assert_eq!(state, CheckState::Mixed);

    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.4, ..Input::default() }, bounds(), 0.0, |f| {
        f.checkbox(1, "Select all", &mut state, true);
    });
    let _ = ui.frame(Input { pointer: Some(p), released: true, time: 0.45, ..Input::default() }, bounds(), 0.0, |f| {
        f.checkbox(1, "Select all", &mut state, true);
    });
    assert_eq!(state, CheckState::Unchecked);
}

#[test]
fn checkbox_bool_with_error_sets_invalid_semantic_and_toggles_binary() {
    let mut ui = Ui::default();
    let mut accepted = false;
    let p = [40.0, 20.0];
    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.0, ..Input::default() }, bounds(), 0.0, |f| {
        f.checkbox_bool_with_error(2, "Accept terms", &mut accepted, true, true);
    });
    let _ = ui.frame(Input { pointer: Some(p), released: true, time: 0.05, ..Input::default() }, bounds(), 0.0, |f| {
        f.checkbox_bool_with_error(2, "Accept terms", &mut accepted, true, true);
    });
    assert!(accepted);
    assert!(ui.semantics().iter().any(|n| n.role == Role::Checkbox && n.invalid));
}
