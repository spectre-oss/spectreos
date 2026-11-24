use crate::*;

#[test]
fn stepper_respects_linear_completion_and_advances() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 420.0, height: 240.0 };
    let steps = ["Account", "Profile", "Confirm"];
    let mut state = StepperState::default();
    ui.frame(Input::default(), bounds, 8.0, |f| {
        f.stepper(400, &steps, &mut state, true, 80.0, |f, step| {
            f.label(format!("Step {step}"));
        });
    });
    // Step 2 (index 1) should be disabled until step 0 is completed in linear mode.
    let step1 = ui.semantics().iter().find(|n| n.id == 401).unwrap();
    assert!(!step1.enabled);

    state.set_completed(0, true);
    ui.frame(Input::default(), bounds, 8.0, |f| {
        f.stepper(400, &steps, &mut state, true, 80.0, |_, _| {});
    });
    let step1_b = ui.semantics().iter().find(|n| n.id == 401).unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([step1_b.x + 8.0, step1_b.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 8.0, |f| {
            f.stepper(400, &steps, &mut state, true, 80.0, |_, _| {});
        });
    }
    assert_eq!(state.active, 1);
}
