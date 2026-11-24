use crate::*;

#[test]
fn select_opens_menu_and_picks_option() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 360.0 };
    let opts = ["Light", "Dark", "System"];
    let mut selected = 0usize;
    let mut state = SelectState::default();
    ui.frame(Input::default(), bounds, 8.0, |f| {
        f.select(700, "Theme", &opts, &mut selected, &mut state, true);
    });
    let trig = ui.semantics().iter().find(|n| n.id == 700).unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([trig.x + 12.0, trig.y + 12.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 8.0, |f| {
            f.select(700, "Theme", &opts, &mut selected, &mut state, true);
        });
    }
    assert!(state.overlay.open);
    ui.frame(Input { time: 0.3, ..Input::default() }, bounds, 8.0, |f| {
        f.select(700, "Theme", &opts, &mut selected, &mut state, true);
    });
    let item2 = ui.semantics().iter().find(|n| n.label.as_ref() == "System").unwrap().bounds;
    for (p, r, t) in [(true, false, 0.4), (false, true, 0.5)] {
        ui.frame(Input { pointer: Some([item2.x + 10.0, item2.y + 10.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 8.0, |f| {
            f.select(700, "Theme", &opts, &mut selected, &mut state, true);
        });
    }
    assert_eq!(selected, 2);
    assert!(!state.overlay.open);
}
