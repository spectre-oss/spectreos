use bhoo::*;
fn bounds() -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
    }
}
fn modal(ui: &mut Ui, state: &mut OverlayState, input: Input) {
    ui.frame(input, bounds(), 8.0, |f| {
        f.with_enabled(!state.open, |f| {
            if f.button(100, "Open").clicked {
                state.open = true;
            }
        });
        f.overlay(
            500,
            state,
            Rect {
                x: 20.0,
                y: 60.0,
                width: 300.0,
                height: 200.0,
            },
            true,
            |f, open| {
                if f.button(600, "Close").clicked {
                    *open = false;
                }
                f.button(601, "Other action");
            },
        );
    });
}
#[test]
fn overlay_dismisses_without_waiting_for_exit_and_restores_focus() {
    let mut ui = Ui::default();
    let mut state = OverlayState::default();
    modal(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([30.0, 18.0]),
            pressed: true,
            ..Input::default()
        },
    );
    modal(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([30.0, 18.0]),
            released: true,
            time: 0.01,
            ..Input::default()
        },
    );
    assert!(state.open);
    modal(
        &mut ui,
        &mut state,
        Input {
            time: 1.0,
            ..Input::default()
        },
    );
    assert_eq!(ui.focused(), Some(600));
    modal(
        &mut ui,
        &mut state,
        Input {
            key: Some(KeyInput {
                key: Key::Escape,
                pressed: true,
                repeat: false,
                shift: false,
            }),
            time: 1.1,
            ..Input::default()
        },
    );
    assert!(!state.open);
    modal(
        &mut ui,
        &mut state,
        Input {
            time: 1.2,
            ..Input::default()
        },
    );
    assert_eq!(ui.focused(), Some(100));
    assert!(!ui.semantics().iter().any(|n| n.id == 600));
    modal(
        &mut ui,
        &mut state,
        Input {
            time: 3.0,
            ..Input::default()
        },
    );
    modal(
        &mut ui,
        &mut state,
        Input {
            time: 4.0,
            ..Input::default()
        },
    );
    assert!(!ui.needs_redraw());
}
#[test]
fn modal_tab_cannot_reach_background() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = OverlayState::default();
    state.open = true;
    modal(&mut ui, &mut state, Input::default());
    for i in 1..7 {
        modal(
            &mut ui,
            &mut state,
            Input {
                time: i as f64,
                key: Some(KeyInput {
                    key: Key::Tab,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..Input::default()
            },
        );
        assert!(matches!(ui.focused(), Some(600 | 601)));
    }
}
#[test]
fn grid_builds_visible_cells_and_end_reveals_last_item() {
    let mut ui = Ui::default();
    let mut state = ListState::default();
    let mut built = 0;
    let mut draw = |ui: &mut Ui, input| {
        built = 0;
        ui.frame(input, bounds(), 0.0, |f| {
            f.virtual_grid(1, 200.0, 3, 100_000, &mut state, |f, i, selected| {
                built += 1;
                f.list_item(i as u64 + 100, "App", selected, true)
            });
        });
        assert!(built <= 15);
    };
    draw(&mut ui, Input::default());
    draw(
        &mut ui,
        Input {
            key: Some(KeyInput {
                key: Key::Tab,
                pressed: true,
                repeat: false,
                shift: false,
            }),
            ..Input::default()
        },
    );
    draw(
        &mut ui,
        Input {
            key: Some(KeyInput {
                key: Key::End,
                pressed: true,
                repeat: false,
                shift: false,
            }),
            ..Input::default()
        },
    );
    assert_eq!(state.selected, Some(99_999));
    assert!(ui.semantics().iter().any(|n| n.id == 100_099));
}
