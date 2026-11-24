use bhoo::*;
const B: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 300.0,
    height: 300.0,
};
fn draw(ui: &mut Ui, input: Input, enabled: bool) -> Vec<DrawCommand> {
    ui.frame(input, B, 8.0, |f| {
        f.with_enabled(enabled, |f| {
            f.button(1, "Action");
        });
    })
}
fn ink(commands: &[DrawCommand]) -> Option<(Rect, [f32; 2], f32, Color)> {
    commands.iter().find_map(|c| {
        if let DrawCommand::Ripple {
            bounds,
            center,
            radius,
            color,
            ..
        } = c
        {
            Some((*bounds, *center, *radius, *color))
        } else {
            None
        }
    })
}
fn chip_frame(ui: &mut Ui, time: f64, selected: bool) -> (f32, usize) {
    let commands = ui.frame(
        Input {
            time,
            ..Input::default()
        },
        B,
        8.0,
        |f| {
            f.chip(1, "Filter", selected, true);
        },
    );
    let padding = commands
        .iter()
        .find_map(|command| match command {
            DrawCommand::Text { text, style, .. } if &**text == "Filter" => Some(style.padding),
            _ => None,
        })
        .unwrap();
    let strokes = commands
        .iter()
        .filter(|c| matches!(c, DrawCommand::Line { .. }))
        .count();
    (padding, strokes)
}
#[test]
fn chip_selection_slides_before_drawing_tick_and_reduced_motion_snaps() {
    let mut ui = Ui::default();
    assert_eq!(chip_frame(&mut ui, 0.0, false), (12.0, 0));
    assert_eq!(chip_frame(&mut ui, 1.0, true), (12.0, 0));
    assert!(
        ui.semantics()[0].selected,
        "Logical state must commit before motion"
    );
    let before_tick = chip_frame(&mut ui, 1.030, true);
    assert!(before_tick.0 > 12.0 && before_tick.0 < 36.0);
    assert_eq!(before_tick.1, 0, "The tick must wait for its 45ms delay");
    assert!(chip_frame(&mut ui, 1.100, true).1 > 0);
    assert_eq!(chip_frame(&mut ui, 1.200, true), (36.0, 2));
    assert!(!ui.needs_redraw());
    let start = chip_frame(&mut ui, 2.0, false);
    assert_eq!(start.1, 0, "Deselection hides the tick immediately");
    assert!(chip_frame(&mut ui, 2.040, false).0 < 36.0);
    ui.motion_preference = MotionPreference::Reduced;
    assert_eq!(chip_frame(&mut ui, 2.050, false), (12.0, 0));
    assert!(!ui.needs_redraw());
    assert_eq!(chip_frame(&mut ui, 2.060, true), (36.0, 2));
    assert!(!ui.needs_redraw());
}
#[test]
fn chip_reverses_without_jumping_and_initial_selection_does_not_animate() {
    let mut ui = Ui::default();
    assert_eq!(chip_frame(&mut ui, 0.0, true), (36.0, 2));
    assert!(!ui.needs_redraw());
    chip_frame(&mut ui, 1.0, false);
    let halfway = chip_frame(&mut ui, 1.075, false).0;
    assert_eq!(chip_frame(&mut ui, 1.075, true).0, halfway);
    assert_eq!(chip_frame(&mut ui, 1.200, true).0, 36.0);
}
#[test]
fn pointer_ripple_expands_from_press_to_center_and_fades_after_release() {
    let mut ui = Ui::default();
    draw(
        &mut ui,
        Input {
            pressed: true,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    );
    let first = ink(&draw(
        &mut ui,
        Input {
            time: 0.020,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    ))
    .unwrap();
    assert!(first.1[0] > 20.0 && first.1[0] < 150.0);
    let grown = ink(&draw(
        &mut ui,
        Input {
            time: 0.5,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    ))
    .unwrap();
    assert!((grown.1[0] - 150.0).abs() < 0.01);
    assert!(grown.2 > first.2);
    assert!(!ui.needs_redraw());
    draw(
        &mut ui,
        Input {
            released: true,
            time: 0.6,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    );
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                time: 0.7,
                ..Input::default()
            },
            true
        ))
        .is_some()
    );
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                time: 1.1,
                ..Input::default()
            },
            true
        ))
        .is_none()
    );
}
#[test]
fn keyboard_ink_is_centered_and_disable_or_reduce_removes_it() {
    let mut ui = Ui::default();
    draw(&mut ui, Input::default(), true);
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
        true,
    );
    draw(
        &mut ui,
        Input {
            key: Some(KeyInput {
                key: Key::Space,
                pressed: true,
                repeat: false,
                shift: false,
            }),
            pointer: Some([1.0, 1.0]),
            time: 0.01,
            ..Input::default()
        },
        true,
    );
    let c = ink(&draw(
        &mut ui,
        Input {
            time: 0.05,
            ..Input::default()
        },
        true,
    ))
    .unwrap();
    assert_eq!(c.1, [150.0, 20.0]);
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                time: 0.06,
                ..Input::default()
            },
            false
        ))
        .is_none()
    );
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                time: 0.07,
                ..Input::default()
            },
            true
        ))
        .is_none()
    );
    draw(
        &mut ui,
        Input {
            pressed: true,
            time: 1.0,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    );
    ui.motion_preference = MotionPreference::Reduced;
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                time: 1.01,
                ..Input::default()
            },
            true
        ))
        .is_none()
    );
}
#[test]
fn tabs_have_one_ink_part_and_keyboard_does_not_select_stale_pointer_position() {
    let mut ui = Ui::default();
    let mut selected = 0;
    let mut build = |ui: &mut Ui, input| {
        ui.frame(input, B, 8.0, |f| {
            f.tabs(1, &["One", "Two", "Three"], &mut selected, true);
        })
    };
    build(
        &mut ui,
        Input {
            pressed: true,
            pointer: Some([150.0, 18.0]),
            ..Input::default()
        },
    );
    let commands = build(
        &mut ui,
        Input {
            time: 0.05,
            pointer: Some([150.0, 18.0]),
            ..Input::default()
        },
    );
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::Ripple { .. }))
            .count(),
        1
    );
    assert_eq!(ink(&commands).unwrap().0.width, 100.0);
    build(
        &mut ui,
        Input {
            released: true,
            time: 0.1,
            pointer: Some([150.0, 18.0]),
            ..Input::default()
        },
    );
    build(
        &mut ui,
        Input {
            key: Some(KeyInput {
                key: Key::Enter,
                pressed: true,
                repeat: false,
                shift: false,
            }),
            time: 0.2,
            pointer: Some([250.0, 18.0]),
            ..Input::default()
        },
    );
    assert_eq!(selected, 1);
}
#[test]
fn loading_reduced_motion_is_static_and_determinate_values_remain_semantic() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = LoadingState::default();
    let commands = ui.frame(Input::default(), B, 0.0, |f| {
        f.loading_indicator(1, "Busy", &mut state, None, false)
    });
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Loading { .. }))
    );
    assert!(!ui.needs_redraw());
    ui.frame(Input::default(), B, 0.0, |f| {
        f.loading_indicator(1, "Download", &mut state, Some(0.4), true)
    });
    assert_eq!(ui.semantics()[0].value, Some(0.4));
    assert!(!ui.needs_redraw());
}
#[test]
fn web_menu_and_dialog_settle_and_reject_input_while_exiting() {
    for kind in [OverlayKind::Menu, OverlayKind::Dialog] {
        let mut ui = Ui::default();
        let mut state = OverlayState::default();
        state.kind = kind;
        state.open = true;
        let mut actions = 0;
        let mut build = |ui: &mut Ui, time: f64, key| {
            ui.frame(
                Input {
                    time,
                    key,
                    ..Input::default()
                },
                B,
                8.0,
                |f| {
                    f.overlay(100, &mut state, B, true, |f, _| {
                        f.overlay_section(OverlaySection::Actions, |f| {
                            f.overlay_item(0, 1, |f| {
                                if f.button(200, "Action").clicked {
                                    actions += 1;
                                }
                            })
                        });
                    })
                },
            )
        };
        build(&mut ui, 0.0, None);
        assert!(ui.needs_redraw());
        build(&mut ui, 1.0, None);
        assert!(!ui.needs_redraw());
        build(
            &mut ui,
            1.1,
            Some(KeyInput {
                key: Key::Escape,
                pressed: true,
                repeat: false,
                shift: false,
            }),
        );
        assert!(!ui.semantics().iter().any(|n| n.id == 200));
        build(&mut ui, 2.0, None);
        assert!(!ui.needs_redraw());
        assert_eq!(actions, 0);
    }
}

#[test]
fn touch_waits_on_a_deadline_but_a_quick_tap_still_gets_ink_and_cancel_does_not() {
    let mut ui = Ui::default();
    draw(
        &mut ui,
        Input {
            pressed: true,
            pointer_kind: PointerKind::Touch,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    );
    assert!(!ui.needs_redraw());
    assert_eq!(ui.next_wakeup(), Some(0.150));
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                time: 0.05,
                pointer_kind: PointerKind::Touch,
                pointer: Some([20.0, 18.0]),
                ..Input::default()
            },
            true
        ))
        .is_none()
    );
    draw(
        &mut ui,
        Input {
            released: true,
            time: 0.06,
            pointer_kind: PointerKind::Touch,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    );
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                time: 0.09,
                pointer_kind: PointerKind::Touch,
                ..Input::default()
            },
            true
        ))
        .is_some()
    );
    let mut ui = Ui::default();
    draw(
        &mut ui,
        Input {
            pressed: true,
            pointer_kind: PointerKind::Touch,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    );
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                cancelled: true,
                time: 0.06,
                pointer_kind: PointerKind::Touch,
                ..Input::default()
            },
            true
        ))
        .is_none()
    );
    assert!(ui.next_wakeup().is_none());
}

#[test]
fn tooltip_escape_suppresses_until_leave_and_does_not_poll_an_expired_deadline() {
    let mut ui = Ui::default();
    let mut draw = |input| {
        ui.frame(input, B, 8.0, |f| {
            let owner = f.button(1, "Hover");
            f.tooltip(2, owner, "Hint");
        });
    };
    draw(Input {
        pointer: Some([20.0, 18.0]),
        ..Input::default()
    });
    draw(Input {
        pointer: Some([20.0, 18.0]),
        time: 0.6,
        ..Input::default()
    });
    draw(Input {
        pointer: Some([20.0, 18.0]),
        time: 0.7,
        key: Some(KeyInput {
            key: Key::Escape,
            pressed: true,
            repeat: false,
            shift: false,
        }),
        ..Input::default()
    });
    draw(Input {
        pointer: Some([20.0, 18.0]),
        time: 2.0,
        ..Input::default()
    });
    assert!(!ui.has_escape_target());
    assert!(ui.next_wakeup().is_none());
    assert!(!ui.semantics().iter().any(|n| n.role == Role::Tooltip));
}

#[test]
fn hiding_cancels_ink_and_loading_resume_excludes_hidden_time() {
    let mut ui = Ui::default();
    draw(
        &mut ui,
        Input {
            pressed: true,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    );
    draw(
        &mut ui,
        Input {
            time: 0.05,
            pointer: Some([20.0, 18.0]),
            ..Input::default()
        },
        true,
    );
    ui.pause_animations();
    assert!(
        ink(&draw(
            &mut ui,
            Input {
                time: 50.0,
                ..Input::default()
            },
            true
        ))
        .is_none()
    );
    let mut state = LoadingState::default();
    let mut build = |ui: &mut Ui, time| {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            B,
            0.0,
            |f| f.loading_indicator(2, "Busy", &mut state, None, false),
        )
    };
    build(&mut ui, 51.0);
    let a = build(&mut ui, 51.2);
    ui.pause_animations();
    let b = build(&mut ui, 150.0);
    let sample = |commands: Vec<DrawCommand>| {
        commands
            .into_iter()
            .find_map(|c| {
                if let DrawCommand::Loading { sample, .. } = c {
                    Some(sample)
                } else {
                    None
                }
            })
            .unwrap()
    };
    assert_eq!(sample(a), sample(b));
}
