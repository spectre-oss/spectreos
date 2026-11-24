use bhoo::*;
const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 400.0,
    height: 100.0,
};
const B: Rect = Rect {
    x: 10.0,
    y: 10.0,
    width: 360.0,
    height: 48.0,
};
const ACTIONS: [ToolbarAction<'static>; 3] = [
    ToolbarAction {
        label: "Cut",
        enabled: true,
    },
    ToolbarAction {
        label: "Copy",
        enabled: true,
    },
    ToolbarAction {
        label: "Paste",
        enabled: true,
    },
];
fn frame(ui: &mut Ui, state: &mut ButtonGroupState, input: Input) -> Option<usize> {
    let mut result = None;
    ui.frame(input, VIEW, 0.0, |f| {
        result = f.button_group(100, B, &ACTIONS, state)
    });
    result
}
#[test]
fn press_expands_active_button_and_preserves_actions_and_targets() {
    let mut ui = Ui::default();
    let mut state = ButtonGroupState::default();
    frame(&mut ui, &mut state, Input::default());
    let initial: Vec<_> = ui.semantics().iter().map(|n| n.bounds).collect();
    assert_eq!(initial.len(), 3);
    assert!(initial.iter().all(|b| b.width >= 48.0));
    frame(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([190.0, 34.0]),
            pressed: true,
            time: 0.01,
            ..Input::default()
        },
    );
    frame(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([190.0, 34.0]),
            time: 0.21,
            ..Input::default()
        },
    );
    let pressed: Vec<_> = ui.semantics().iter().map(|n| n.bounds).collect();
    assert!(pressed[1].width > initial[1].width);
    assert!(pressed[0].width < initial[0].width);
    assert!(pressed[2].width < initial[2].width);
    assert!(pressed.iter().all(|b| b.width >= 48.0));
    assert_eq!(
        frame(
            &mut ui,
            &mut state,
            Input {
                pointer: Some([190.0, 34.0]),
                released: true,
                time: 0.22,
                ..Input::default()
            }
        ),
        Some(1)
    );
    frame(
        &mut ui,
        &mut state,
        Input {
            time: 2.0,
            ..Input::default()
        },
    );
    frame(
        &mut ui,
        &mut state,
        Input {
            time: 4.0,
            ..Input::default()
        },
    );
    assert!(!ui.needs_redraw());
}

fn connected(
    ui: &mut Ui,
    selected: &mut u32,
    input: Input,
    multiple: bool,
    required: bool,
) -> Option<usize> {
    let mut result = None;
    let actions = [
        ToolbarAction {
            label: "One",
            enabled: true,
        },
        ToolbarAction {
            label: "Two",
            enabled: true,
        },
        ToolbarAction {
            label: "Disabled",
            enabled: false,
        },
    ];
    ui.frame(input, VIEW, 0.0, |f| {
        result = f.connected_button_group(200, B, &actions, selected, multiple, required)
    });
    result
}
fn click_connected(
    ui: &mut Ui,
    selected: &mut u32,
    x: f32,
    time: f64,
    multiple: bool,
    required: bool,
) -> Option<usize> {
    connected(
        ui,
        selected,
        Input {
            pointer: Some([x, 34.0]),
            pressed: true,
            time,
            ..Input::default()
        },
        multiple,
        required,
    );
    connected(
        ui,
        selected,
        Input {
            pointer: Some([x, 34.0]),
            released: true,
            time: time + 0.01,
            ..Input::default()
        },
        multiple,
        required,
    )
}
#[test]
fn connected_selection_required_multiple_disabled_and_reduced_motion() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut selected = 1;
    connected(&mut ui, &mut selected, Input::default(), false, true);
    let before: Vec<_> = ui.semantics().iter().map(|n| n.bounds).collect();
    assert_eq!(
        click_connected(&mut ui, &mut selected, 50.0, 0.1, false, true),
        Some(0)
    );
    assert_eq!(selected, 1);
    assert_eq!(
        click_connected(&mut ui, &mut selected, 180.0, 0.2, false, true),
        Some(1)
    );
    assert_eq!(selected, 2);
    assert_eq!(ui.semantics().iter().filter(|n| n.selected).count(), 1);
    assert_eq!(
        ui.semantics().iter().map(|n| n.bounds).collect::<Vec<_>>(),
        before
    );
    assert_eq!(
        click_connected(&mut ui, &mut selected, 320.0, 0.3, false, true),
        None
    );
    assert_eq!(selected, 2);
    assert_eq!(
        click_connected(&mut ui, &mut selected, 50.0, 0.4, true, false),
        Some(0)
    );
    assert_eq!(selected, 3);
    assert_eq!(
        click_connected(&mut ui, &mut selected, 180.0, 0.5, true, false),
        Some(1)
    );
    assert_eq!(selected, 1);
    connected(
        &mut ui,
        &mut selected,
        Input {
            time: 1.0,
            ..Input::default()
        },
        true,
        false,
    );
    assert!(!ui.needs_redraw());
}

fn key(key: Key, pressed: bool, time: f64) -> Input {
    Input {
        key: Some(KeyInput {
            key,
            pressed,
            repeat: false,
            shift: false,
        }),
        time,
        ..Input::default()
    }
}
#[test]
fn connected_keyboard_skips_disabled_and_exposes_toggle_state() {
    let mut ui = Ui::default();
    let mut selected = 1;
    connected(&mut ui, &mut selected, Input::default(), false, true);
    connected(
        &mut ui,
        &mut selected,
        key(Key::Tab, true, 0.1),
        false,
        true,
    );
    assert!(ui.semantics().iter().any(|n| n.id == 200 && n.focused));
    connected(
        &mut ui,
        &mut selected,
        key(Key::Left, true, 0.2),
        false,
        true,
    );
    assert!(ui.semantics().iter().any(|n| n.id == 201 && n.focused));
    assert!(!ui.semantics().iter().any(|n| n.id == 202 && n.focused));
    assert_eq!(selected, 1);
    connected(
        &mut ui,
        &mut selected,
        key(Key::Space, true, 0.3),
        false,
        true,
    );
    assert_eq!(
        connected(
            &mut ui,
            &mut selected,
            key(Key::Space, false, 0.4),
            false,
            true
        ),
        Some(1)
    );
    assert_eq!(selected, 2);
    assert!(ui.semantics().iter().all(|n| n.role == Role::ToggleButton));
    assert_eq!(
        ui.semantics().iter().find(|n| n.id == 201).unwrap().checked,
        Some(CheckState::Checked)
    );
    assert_eq!(
        ui.semantics().iter().find(|n| n.id == 200).unwrap().checked,
        Some(CheckState::Unchecked)
    );
}
#[test]
fn connected_ripple_uses_two_corner_clips_and_reduced_motion_suppresses_it() {
    let actions = [
        ToolbarAction {
            label: "A",
            enabled: true,
        },
        ToolbarAction {
            label: "B",
            enabled: true,
        },
    ];
    let mut ui = Ui::default();
    let mut selection = 0;
    let sample = |ui: &mut Ui, selection: &mut u32, input: Input| {
        ui.frame(input, VIEW, 0.0, |f| {
            f.connected_button_group(500, B, &actions, selection, true, false);
        })
        .to_vec()
    };
    sample(&mut ui, &mut selection, Input::default());
    sample(
        &mut ui,
        &mut selection,
        Input {
            pointer: Some([40.0, 34.0]),
            pressed: true,
            time: 0.1,
            ..Input::default()
        },
    );
    let active = sample(
        &mut ui,
        &mut selection,
        Input {
            pointer: Some([40.0, 34.0]),
            time: 0.2,
            ..Input::default()
        },
    );
    let ripples: Vec<_> = active
        .iter()
        .filter_map(|c| {
            if let DrawCommand::Ripple { corner_radius, .. } = c {
                Some(*corner_radius)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(ripples.len(), 2);
    assert!(ripples[0] > ripples[1]);
    ui.motion_preference = MotionPreference::Reduced;
    let reduced = sample(
        &mut ui,
        &mut selection,
        Input {
            pointer: Some([40.0, 34.0]),
            time: 0.3,
            ..Input::default()
        },
    );
    assert!(
        !reduced
            .iter()
            .any(|c| matches!(c, DrawCommand::Ripple { .. }))
    );
}

#[test]
fn groups_reject_wrapped_ids_and_clip_presentation_bounds() {
    let mut ui = Ui::default();
    let mut state = ButtonGroupState::default();
    let mut selected = 0;
    ui.frame(Input::default(), VIEW, 0.0, |f| {
        assert_eq!(f.button_group(u64::MAX, B, &ACTIONS, &mut state), None);
        assert_eq!(
            f.connected_button_group(u64::MAX, B, &ACTIONS, &mut selected, true, false),
            None
        );
        f.within(
            Rect {
                x: 10.0,
                y: 10.0,
                width: 40.0,
                height: 48.0,
            },
            |f| {
                f.button_group(100, B, &ACTIONS, &mut state);
            },
        );
    });
    assert_eq!(ui.semantics().len(), 3);
    assert_eq!(ui.semantics()[0].bounds.width, 40.0);
    assert_eq!(ui.semantics()[1].bounds.width, 0.0);
    assert!(!ui.semantics()[1].enabled);
}

#[test]
fn connected_rtl_mirrors_items_and_arrow_navigation() {
    let mut ui = Ui::default();
    let mut selected = 1;
    let draw = |ui: &mut Ui, selected: &mut u32, input: Input| {
        ui.frame(input, VIEW, 0.0, |f| {
            f.connected_button_group_direction(200, B, &ACTIONS, selected, false, true, true);
        });
    };
    draw(&mut ui, &mut selected, Input::default());
    let positions: Vec<_> = ui.semantics().iter().map(|n| n.bounds.x).collect();
    assert!(positions[0] > positions[1] && positions[1] > positions[2]);
    draw(&mut ui, &mut selected, key(Key::Tab, true, 0.1));
    assert_eq!(ui.focused(), Some(200));
    draw(&mut ui, &mut selected, key(Key::Left, true, 0.2));
    assert_eq!(ui.focused(), Some(201));
    draw(&mut ui, &mut selected, key(Key::Right, true, 0.3));
    assert_eq!(ui.focused(), Some(200));
    assert_eq!(selected, 1);
}

#[test]
fn standard_rtl_repositions_cached_hits_and_expands_the_pressed_action() {
    let mut ui = Ui::default();
    let mut state = ButtonGroupState::default();
    let mut result = None;
    let mut draw = |ui: &mut Ui, input: Input, rtl: bool| {
        ui.frame(input, VIEW, 0.0, |f| {
            result = f.button_group_direction(100, B, &ACTIONS, &mut state, rtl);
        });
        result
    };
    draw(&mut ui, Input::default(), false);
    let left = ui.semantics()[0].bounds.x;
    draw(
        &mut ui,
        Input {
            time: 0.01,
            ..Input::default()
        },
        true,
    );
    let before: Vec<_> = ui.semantics().iter().map(|n| n.bounds).collect();
    assert!(before[0].x > before[1].x && before[1].x > before[2].x);
    assert!(before[0].x > left);
    draw(
        &mut ui,
        Input {
            pointer: Some([330.0, 34.0]),
            pressed: true,
            time: 0.02,
            ..Input::default()
        },
        true,
    );
    draw(
        &mut ui,
        Input {
            pointer: Some([330.0, 34.0]),
            time: 0.22,
            ..Input::default()
        },
        true,
    );
    assert!(ui.semantics()[0].bounds.width > before[0].width);
    assert_eq!(
        draw(
            &mut ui,
            Input {
                pointer: Some([330.0, 34.0]),
                released: true,
                time: 0.23,
                ..Input::default()
            },
            true
        ),
        Some(0)
    );
}

#[test]
fn standard_group_pressed_corner_morphs_and_reduced_motion_snaps() {
    fn render(ui: &mut Ui, state: &mut ButtonGroupState, input: Input) -> Vec<DrawCommand> {
        ui.frame(input, VIEW, 0.0, |f| {
            f.button_group(100, B, &ACTIONS, state);
        })
    }
    fn button_radii(commands: &[DrawCommand]) -> Vec<f32> {
        commands
            .iter()
            .filter_map(|c| match c {
                DrawCommand::RoundedRect { bounds, radius, .. }
                    if bounds.height == 40.0 && bounds.width >= 48.0 =>
                {
                    Some(*radius)
                }
                _ => None,
            })
            .collect()
    }
    let mut ui = Ui::default();
    let mut state = ButtonGroupState::default();
    assert_eq!(
        button_radii(&render(&mut ui, &mut state, Input::default())),
        vec![20.0; 3]
    );
    render(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([190.0, 34.0]),
            pressed: true,
            time: 0.01,
            ..Input::default()
        },
    );
    let radii = button_radii(&render(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([190.0, 34.0]),
            time: 0.04,
            ..Input::default()
        },
    ));
    assert!(radii[1] < 20.0 && radii[1] > 8.0);
    assert_eq!(radii[0], 20.0);
    ui.motion_preference = MotionPreference::Reduced;
    let radii = button_radii(&render(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([190.0, 34.0]),
            time: 0.05,
            ..Input::default()
        },
    ));
    assert_eq!(radii[1], 8.0);
}

#[test]
fn standard_overflow_exposes_hidden_actions_and_restores_when_wide() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ButtonGroupState::default();
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
    };
    let narrow = Rect {
        x: 10.0,
        y: 10.0,
        width: 120.0,
        height: 48.0,
    };
    let mut draw = |ui: &mut Ui, bounds: Rect, input: Input| {
        let mut result = None;
        ui.frame(input, view, 0.0, |f| {
            result = f.button_group_with_overflow(100, bounds, &ACTIONS, &mut state, false);
        });
        result
    };
    draw(&mut ui, narrow, Input::default());
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "More"));
    for (pressed, released, time) in [(true, false, 0.1), (false, true, 0.11)] {
        draw(
            &mut ui,
            narrow,
            Input {
                pointer: Some([100.0, 34.0]),
                pressed,
                released,
                time,
                ..Input::default()
            },
        );
    }
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "Copy"));
    draw(
        &mut ui,
        B,
        Input {
            time: 0.2,
            ..Input::default()
        },
    );
    assert!(!ui.semantics().iter().any(|n| n.label.as_ref() == "More"));
    assert_eq!(
        ui.semantics()
            .iter()
            .filter(|n| n.role == Role::Button)
            .count(),
        3
    );
}

#[test]
fn standard_overflow_pointer_selects_hidden_action() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ButtonGroupState::default();
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
    };
    let narrow = Rect {
        x: 10.0,
        y: 10.0,
        width: 120.0,
        height: 48.0,
    };
    let mut draw = |ui: &mut Ui, input: Input| {
        let mut result = None;
        ui.frame(input, view, 0.0, |f| {
            result = f.button_group_with_overflow(100, narrow, &ACTIONS, &mut state, false);
        });
        result
    };
    draw(&mut ui, Input::default());
    draw(
        &mut ui,
        Input {
            pointer: Some([100.0, 34.0]),
            pressed: true,
            time: 0.1,
            ..Input::default()
        },
    );
    draw(
        &mut ui,
        Input {
            pointer: Some([100.0, 34.0]),
            released: true,
            time: 0.11,
            ..Input::default()
        },
    );
    let node = ui
        .semantics()
        .iter()
        .find(|n| n.label.as_ref() == "Copy")
        .unwrap();
    let p = [
        node.bounds.x + node.bounds.width / 2.0,
        node.bounds.y + node.bounds.height / 2.0,
    ];
    draw(
        &mut ui,
        Input {
            pointer: Some(p),
            pressed: true,
            time: 0.2,
            ..Input::default()
        },
    );
    let result = draw(
        &mut ui,
        Input {
            pointer: Some(p),
            released: true,
            time: 0.21,
            ..Input::default()
        },
    );
    assert_eq!(result, Some(1));
}

#[test]
fn standard_overflow_keyboard_selects_hidden_action() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ButtonGroupState::default();
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
    };
    let narrow = Rect {
        x: 10.0,
        y: 10.0,
        width: 120.0,
        height: 48.0,
    };
    let mut draw = |ui: &mut Ui, input: Input| {
        let mut result = None;
        ui.frame(input, view, 0.0, |f| {
            result = f.button_group_with_overflow(100, narrow, &ACTIONS, &mut state, false);
        });
        result
    };
    draw(&mut ui, Input::default());
    draw(&mut ui, key(Key::Tab, true, 0.1));
    draw(&mut ui, key(Key::Tab, true, 0.2));
    draw(&mut ui, key(Key::Enter, true, 0.3));
    draw(
        &mut ui,
        Input {
            time: 0.31,
            ..Input::default()
        },
    );
    assert_eq!(ui.focused(), Some(133));
    draw(&mut ui, key(Key::Down, true, 0.4));
    assert_eq!(ui.focused(), Some(134));
    assert_eq!(draw(&mut ui, key(Key::Enter, true, 0.5)), Some(2));
}

#[test]
fn overflow_touch_drag_scrolls_without_selecting_and_respects_reduced_motion() {
    for reduced in [false, true] {
        let mut ui = Ui::default();
        if reduced {
            ui.motion_preference = MotionPreference::Reduced;
        }
        let mut state = ButtonGroupState::default();
        let actions = [ToolbarAction {
            label: "Item",
            enabled: true,
        }; 8];
        let view = Rect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 200.0,
        };
        let bounds = Rect {
            x: 10.0,
            y: 10.0,
            width: 120.0,
            height: 48.0,
        };
        let mut draw = |ui: &mut Ui, input: Input| {
            let mut result = None;
            ui.frame(input, view, 0.0, |f| {
                result = f.button_group_with_overflow(100, bounds, &actions, &mut state, false);
            });
            result
        };
        draw(&mut ui, Input::default());
        draw(
            &mut ui,
            Input {
                pointer: Some([100.0, 34.0]),
                pressed: true,
                time: 0.1,
                ..Input::default()
            },
        );
        draw(
            &mut ui,
            Input {
                pointer: Some([100.0, 34.0]),
                released: true,
                time: 0.11,
                ..Input::default()
            },
        );
        draw(
            &mut ui,
            Input {
                time: 0.5,
                ..Input::default()
            },
        );
        let old = ui
            .semantics()
            .iter()
            .find(|n| n.id == 134)
            .unwrap()
            .bounds
            .y;
        draw(
            &mut ui,
            Input {
                pointer: Some([100.0, 150.0]),
                pointer_kind: PointerKind::Touch,
                pressed: true,
                time: 0.51,
                ..Input::default()
            },
        );
        draw(
            &mut ui,
            Input {
                pointer: Some([100.0, 100.0]),
                pointer_kind: PointerKind::Touch,
                time: 0.56,
                ..Input::default()
            },
        );
        assert!(
            ui.semantics()
                .iter()
                .find(|n| n.id == 134)
                .unwrap()
                .bounds
                .y
                < old
        );
        assert_eq!(
            draw(
                &mut ui,
                Input {
                    pointer: Some([100.0, 100.0]),
                    pointer_kind: PointerKind::Touch,
                    released: true,
                    time: 0.57,
                    ..Input::default()
                }
            ),
            None
        );
        let after = ui
            .semantics()
            .iter()
            .find(|n| n.id == 135)
            .unwrap()
            .bounds
            .y;
        draw(
            &mut ui,
            Input {
                time: 0.62,
                ..Input::default()
            },
        );
        let later = ui
            .semantics()
            .iter()
            .find(|n| n.id == 135)
            .unwrap()
            .bounds
            .y;
        if reduced {
            assert_eq!(later, after);
        } else {
            assert!(later < after);
        }
    }
}

#[test]
fn overflow_trigger_is_a_named_40dp_material_icon_button() {
    let mut ui = Ui::default();
    let mut state = ButtonGroupState::default();
    let narrow = Rect {
        x: 10.0,
        y: 10.0,
        width: 120.0,
        height: 48.0,
    };
    let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.button_group_with_overflow(100, narrow, &ACTIONS, &mut state, false);
    });
    let more = ui
        .semantics()
        .iter()
        .find(|n| n.label.as_ref() == "More")
        .unwrap();
    assert!(more.bounds.width >= 48.0 && more.bounds.height >= 48.0);
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color, .. } if bounds.width == 40.0 && bounds.height == 40.0 && *color == ui.theme.primary)));
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::Shape { bounds, .. } if bounds.width == 24.0 && bounds.height == 24.0)));
    assert!(
        !commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "More"))
    );
}

#[test]
fn overflow_skips_disabled_items_and_restores_focus_on_escape_and_resize() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ButtonGroupState::default();
    let actions = [
        ToolbarAction {
            label: "Cut",
            enabled: true,
        },
        ToolbarAction {
            label: "Copy",
            enabled: false,
        },
        ToolbarAction {
            label: "Paste",
            enabled: true,
        },
    ];
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
    };
    let narrow = Rect {
        x: 10.0,
        y: 10.0,
        width: 120.0,
        height: 48.0,
    };
    let mut draw = |ui: &mut Ui, bounds: Rect, input: Input| {
        ui.frame(input, view, 0.0, |f| {
            f.button_group_with_overflow(100, bounds, &actions, &mut state, false);
        });
    };
    draw(&mut ui, narrow, Input::default());
    draw(&mut ui, narrow, key(Key::Tab, true, 0.1));
    draw(&mut ui, narrow, key(Key::Tab, true, 0.2));
    assert_eq!(ui.focused(), Some(101));
    draw(&mut ui, narrow, key(Key::Enter, true, 0.3));
    draw(
        &mut ui,
        narrow,
        Input {
            time: 0.31,
            ..Input::default()
        },
    );
    assert_eq!(ui.focused(), Some(134));
    assert!(!ui.semantics().iter().find(|n| n.id == 133).unwrap().enabled);
    draw(&mut ui, narrow, key(Key::Escape, true, 0.4));
    draw(
        &mut ui,
        narrow,
        Input {
            time: 0.41,
            ..Input::default()
        },
    );
    assert_eq!(ui.focused(), Some(101));
    draw(&mut ui, narrow, key(Key::Enter, true, 0.5));
    draw(
        &mut ui,
        narrow,
        Input {
            time: 0.51,
            ..Input::default()
        },
    );
    assert_eq!(ui.focused(), Some(134));
    draw(
        &mut ui,
        B,
        Input {
            time: 0.6,
            ..Input::default()
        },
    );
    assert_eq!(ui.focused(), Some(100));
    assert!(!ui.semantics().iter().any(|n| n.id == 133));
}

#[test]
fn icon_group_mirrors_icon_and_label_and_exposes_one_target_per_action() {
    let mut ui = Ui::default();
    let mut state = ButtonGroupState::default();
    let icons = [Some(AppBarIcon::Back), None, Some(AppBarIcon::Add)];
    for rtl in [false, true] {
        let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
            f.button_group_with_icons(100, B, &ACTIONS, &icons, &mut state, rtl);
        });
        assert_eq!(ui.semantics().len(), 3);
        let mut shapes = commands.iter().filter_map(|c| match c {
            DrawCommand::Shape { bounds, .. } if bounds.width == 20.0 => Some(*bounds),
            _ => None,
        });
        let first = shapes.next().unwrap();
        assert!(shapes.next().is_some());
        let label = commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::Text { bounds, text, .. } if text.as_ref() == "Cut" => Some(*bounds),
                _ => None,
            })
            .unwrap();
        if rtl {
            assert!(first.x > label.x);
        } else {
            assert!(first.x < label.x);
        }
    }
}

#[test]
fn icon_group_overflow_keeps_icons_and_targets_in_both_directions() {
    let view=Rect { x:0.,y:0.,width:400.,height:400. };
    let narrow=Rect { x:10.,y:10.,width:120.,height:48. };
    let icons=[Some(AppBarIcon::Back),Some(AppBarIcon::Search),Some(AppBarIcon::Add)];
    for rtl in [false,true] {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;
        let mut state=ButtonGroupState::default();
        let render=|ui:&mut Ui,state:&mut ButtonGroupState,input:Input| ui.frame(input,view,0.,|f| { f.button_group_with_overflow_icons(100,narrow,&ACTIONS,&icons,state,rtl); });
        let commands=render(&mut ui,&mut state,Input::default());
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::Shape { bounds,.. } if bounds.width==20.)));
        let trigger=ui.semantics().iter().find(|n|n.label.as_ref()=="More").unwrap().bounds;
        let pointer=Some([trigger.x+trigger.width*0.5,trigger.y+trigger.height*0.5]);
        for input in [Input { pointer,pressed:true,time:0.1,..Input::default() },Input { pointer,released:true,time:0.11,..Input::default() }] { render(&mut ui,&mut state,input); }
        let commands=render(&mut ui,&mut state,Input { time:0.2,..Input::default() });
        for label in ["Copy","Paste"] { assert!(ui.semantics().iter().any(|n|n.label.as_ref()==label)); }
        for label in ["Copy","Paste"] {
            let label_bounds=commands.iter().find_map(|c| match c { DrawCommand::Text { bounds,text,.. } if text.as_ref()==label =>Some(*bounds),_=>None }).unwrap();
            assert!(commands.iter().any(|c| matches!(c,DrawCommand::Shape { bounds,.. } if bounds.width==24. && (bounds.y-label_bounds.y-13.).abs()<0.1 && if rtl { bounds.x>label_bounds.x } else { bounds.x<label_bounds.x })));
        }
    }
}

#[test]
fn connected_overflow_preserves_selection_across_visible_and_hidden_items() {
    let view=Rect { x:0.,y:0.,width:400.,height:400. };
    let narrow=Rect { x:10.,y:10.,width:100.,height:48. };
    for rtl in [false,true] {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;
        let mut state=ButtonGroupState::default();let mut selected=2u32;
        let render=|ui:&mut Ui,state:&mut ButtonGroupState,selected:&mut u32,input:Input| {
            let mut action=None;
            let commands=ui.frame(input,view,0.,|f| { action=f.connected_button_group_with_overflow(100,narrow,&ACTIONS,selected,true,true,state,rtl); });
            (action,commands)
        };
        render(&mut ui,&mut state,&mut selected,Input::default());
        let click=|label:&str,ui:&mut Ui,state:&mut ButtonGroupState,selected:&mut u32,time:f64| {
            let b=ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
            let pointer=Some([b.x+b.width/2.,b.y+b.height/2.]);
            render(ui,state,selected,Input { pointer,pressed:true,time,..Input::default() });
            render(ui,state,selected,Input { pointer,released:true,time:time+0.01,..Input::default() }).0
        };
        assert_eq!(click("More",&mut ui,&mut state,&mut selected,0.1),None);
        assert_eq!(selected,2);
        assert_eq!(click("Copy",&mut ui,&mut state,&mut selected,0.3),Some(1));
        assert_eq!(selected,2,"required selection must survive clicking the sole selection");
        click("Cut",&mut ui,&mut state,&mut selected,0.5);assert_eq!(selected,3);
        click("More",&mut ui,&mut state,&mut selected,0.7);
        click("Copy",&mut ui,&mut state,&mut selected,0.9);assert_eq!(selected,1);
        let (_,commands)=render(&mut ui,&mut state,&mut selected,Input { time:1.1,..Input::default() });
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="Cut")));
    }
}

#[test]
fn connected_overflow_single_selection_can_move_to_hidden_action() {
    let view=Rect { x:0.,y:0.,width:400.,height:400. };
    let narrow=Rect { x:10.,y:10.,width:100.,height:48. };
    let actions=[ACTIONS[0],ACTIONS[1],ToolbarAction { label:"Disabled",enabled:false }];
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;
    let mut state=ButtonGroupState::default();let mut selected=1u32;
    let render=|ui:&mut Ui,state:&mut ButtonGroupState,selected:&mut u32,input:Input| {
        let mut action=None;
        ui.frame(input,view,0.,|f| { action=f.connected_button_group_with_overflow(100,narrow,&actions,selected,false,true,state,false); });
        action
    };
    render(&mut ui,&mut state,&mut selected,Input::default());
    let b=ui.semantics().iter().find(|n|n.label.as_ref()=="More").unwrap().bounds;
    let pointer=Some([b.x+b.width/2.,b.y+b.height/2.]);
    for input in [Input { pointer,pressed:true,time:0.1,..Input::default() },Input { pointer,released:true,time:0.11,..Input::default() }] { render(&mut ui,&mut state,&mut selected,input); }
    let disabled=ui.semantics().iter().find(|n|n.label.as_ref()=="Disabled").unwrap();assert!(!disabled.enabled);
    let b=ui.semantics().iter().find(|n|n.label.as_ref()=="Copy").unwrap().bounds;
    let pointer=Some([b.x+b.width/2.,b.y+b.height/2.]);
    render(&mut ui,&mut state,&mut selected,Input { pointer,pressed:true,time:0.3,..Input::default() });
    assert_eq!(render(&mut ui,&mut state,&mut selected,Input { pointer,released:true,time:0.31,..Input::default() }),Some(1));
    assert_eq!(selected,2);
}
