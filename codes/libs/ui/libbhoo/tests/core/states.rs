use bhoo::*;
const BOUNDS: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 400.0,
    height: 600.0,
};
fn key(key: Key, pressed: bool) -> Input {
    Input {
        key: Some(KeyInput {
            key,
            pressed,
            repeat: false,
            shift: false,
        }),
        ..Input::default()
    }
}
fn pointer(pressed: bool, released: bool) -> Input {
    Input {
        pointer: Some([15.0, 15.0]),
        pressed,
        released,
        ..Input::default()
    }
}

#[test]
fn tab_skips_disabled_controls_and_shift_reverses() {
    let mut ui = Ui::default();
    fn build(f: &mut Frame<'_>) {
        f.button(1, "First");
        f.button_with(
            2,
            "Disabled",
            Options {
                enabled: false,
                ..Options::default()
            },
        );
        f.button(3, "Last");
    }
    ui.frame(Input::default(), BOUNDS, 8.0, build);
    ui.frame(key(Key::Tab, true), BOUNDS, 8.0, build);
    assert_eq!(ui.focused(), Some(1));
    ui.frame(key(Key::Tab, true), BOUNDS, 8.0, build);
    assert_eq!(ui.focused(), Some(3));
    let mut back = key(Key::Tab, true);
    back.key.as_mut().unwrap().shift = true;
    ui.frame(back, BOUNDS, 8.0, build);
    assert_eq!(ui.focused(), Some(1));
}
#[test]
fn keyboard_space_activates_once_on_release_enter_ignores_repeat() {
    let mut ui = Ui::default();
    let mut events = 0;
    for input in [
        Input::default(),
        key(Key::Tab, true),
        key(Key::Space, true),
        key(Key::Space, false),
        key(Key::Space, false),
        key(Key::Enter, true),
    ] {
        ui.frame(input, BOUNDS, 8.0, |f| {
            events += usize::from(f.button(1, "Open").clicked);
        });
    }
    assert_eq!(events, 2);
    let mut repeat = key(Key::Enter, true);
    repeat.key.as_mut().unwrap().repeat = true;
    ui.frame(repeat, BOUNDS, 8.0, |f| {
        assert!(!f.button(1, "Open").clicked);
    });
}
#[test]
fn disabling_or_blurring_a_captured_control_cancels_activation() {
    for cancel in [
        Input {
            blurred: true,
            ..Input::default()
        },
        Input {
            cancelled: true,
            ..Input::default()
        },
    ] {
        let mut ui = Ui::default();
        ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
            f.button(1, "Open");
        });
        ui.frame(cancel, BOUNDS, 8.0, |f| {
            assert!(!f.button(1, "Open").clicked);
        });
        ui.frame(pointer(false, true), BOUNDS, 8.0, |f| {
            assert!(!f.button(1, "Open").clicked);
        });
    }
    let mut ui = Ui::default();
    ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
        f.button(1, "Open");
    });
    ui.frame(Input::default(), BOUNDS, 8.0, |f| {
        f.with_enabled(false, |f| {
            let r = f.button(1, "Open");
            assert!(!r.enabled && !r.pressed);
        });
    });
    ui.frame(pointer(false, true), BOUNDS, 8.0, |f| {
        assert!(!f.button(1, "Open").clicked);
    });
}
#[test]
fn checkbox_has_three_states_and_binary_api_never_introduces_mixed() {
    let mut ui = Ui::default();
    let mut value = CheckState::Unchecked;
    for expected in [
        CheckState::Checked,
        CheckState::Mixed,
        CheckState::Unchecked,
    ] {
        ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
            f.checkbox(1, "Choice", &mut value, true);
        });
        ui.frame(pointer(false, true), BOUNDS, 8.0, |f| {
            assert!(f.checkbox(1, "Choice", &mut value, true).changed);
        });
        assert_eq!(value, expected);
        assert_eq!(ui.semantics()[0].checked, Some(expected));
    }
    let mut value = true;
    ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
        f.checkbox_bool(2, "Binary", &mut value, true);
    });
    ui.frame(pointer(false, true), BOUNDS, 8.0, |f| {
        f.checkbox_bool(2, "Binary", &mut value, true);
    });
    assert!(!value);
    assert_eq!(ui.semantics()[0].checked, Some(CheckState::Unchecked));
}
#[test]
fn switch_commits_before_motion_and_eventually_returns_to_idle() {
    let mut ui = Ui::default();
    let mut value = false;
    ui.frame(Input::default(), BOUNDS, 8.0, |f| {
        f.switch(1, "Wi-Fi", &mut value, true);
    });
    ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
        f.switch(1, "Wi-Fi", &mut value, true);
    });
    ui.frame(pointer(false, true), BOUNDS, 8.0, |f| {
        f.switch(1, "Wi-Fi", &mut value, true);
    });
    assert!(value);
    assert_eq!(ui.semantics()[0].checked, Some(CheckState::Checked));
    assert!(ui.needs_redraw());
    ui.frame(
        Input {
            time: 2.0,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            f.switch(1, "Wi-Fi", &mut value, true);
        },
    );
    ui.frame(
        Input {
            time: 4.0,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            f.switch(1, "Wi-Fi", &mut value, true);
        },
    );
    assert!(!ui.needs_redraw());
}
#[test]
fn reduced_motion_rebuilds_committed_state_once_then_idles() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut checked = false;
    ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
        f.switch(1, "State", &mut checked, true);
    });
    ui.frame(pointer(false, true), BOUNDS, 8.0, |f| {
        f.switch(1, "State", &mut checked, true);
    });
    assert!(checked);
    // A discrete action schedules one rebuild for caller-owned labels/layout that
    // were emitted before the action. Reduced motion itself has no running spring.
    assert!(ui.needs_redraw());
    ui.frame(Input::default(), BOUNDS, 8.0, |f| {
        f.label(if checked { "On" } else { "Off" });
        f.switch(1, "State", &mut checked, true);
    });
    assert!(!ui.needs_redraw());
}
#[test]
fn radio_group_skips_disabled_choices_and_uses_one_tab_stop() {
    let mut ui = Ui::default();
    let mut selected = 11;
    let choices = [
        Choice {
            id: 11,
            label: "One",
            enabled: true,
        },
        Choice {
            id: 12,
            label: "Two",
            enabled: false,
        },
        Choice {
            id: 13,
            label: "Three",
            enabled: true,
        },
    ];
    for input in [Input::default(), key(Key::Tab, true), key(Key::Right, true)] {
        ui.frame(input, BOUNDS, 8.0, |f| {
            f.radio_group(1, "Mode", &choices, &mut selected, true);
            f.button(2, "Next");
        });
    }
    assert_eq!(selected, 13);
    assert_eq!(ui.focused(), Some(1));
    ui.frame(key(Key::Tab, true), BOUNDS, 8.0, |f| {
        f.radio_group(1, "Mode", &choices, &mut selected, true);
        f.button(2, "Next");
    });
    assert_eq!(ui.focused(), Some(2));
}
#[test]
fn slider_keyboard_step_bounds_and_disable_are_respected() {
    let mut ui = Ui::default();
    let mut value = 50.0;
    for input in [Input::default(), key(Key::Tab, true), key(Key::Right, true)] {
        ui.frame(input, BOUNDS, 8.0, |f| {
            f.slider(1, "Volume", &mut value, 0.0..=100.0, 5.0, true);
        });
    }
    assert_eq!(value, 55.0);
    ui.frame(key(Key::End, true), BOUNDS, 8.0, |f| {
        f.slider(1, "Volume", &mut value, 0.0..=100.0, 5.0, true);
    });
    assert_eq!(value, 100.0);
    ui.frame(key(Key::Left, true), BOUNDS, 8.0, |f| {
        f.slider(1, "Volume", &mut value, 0.0..=100.0, 5.0, false);
    });
    assert_eq!(value, 100.0);
}
#[test]
fn range_slider_never_crosses_endpoints() {
    let mut ui = Ui::default();
    let mut values = [20.0, 40.0];
    for input in [Input::default(), key(Key::Tab, true), key(Key::End, true)] {
        ui.frame(input, BOUNDS, 8.0, |f| {
            f.range_slider([1, 2], "Range", &mut values, 0.0..=100.0, 1.0, true);
        });
    }
    assert_eq!(values, [40.0, 40.0]);
    for input in [key(Key::Tab, true), key(Key::Home, true)] {
        ui.frame(input, BOUNDS, 8.0, |f| {
            f.range_slider([1, 2], "Range", &mut values, 0.0..=100.0, 1.0, true);
        });
    }
    assert_eq!(values, [40.0, 40.0]);
}
#[test]
fn clipped_controls_cannot_capture_pointer() {
    let mut ui = Ui::default();
    ui.frame(
        Input {
            pointer: Some([80.0, 10.0]),
            pressed: true,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            f.within(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 40.0,
                    height: 40.0,
                },
                |f| {
                    let r = f.button(1, "Clipped");
                    assert!(!r.hovered && !r.pressed);
                },
            );
        },
    );
}
#[test]
fn virtual_list_builds_only_visible_rows_and_end_reveals_last() {
    let mut ui = Ui::default();
    let mut list = ListState::default();
    let mut rows = Vec::new();
    for input in [Input::default(), key(Key::Tab, true), key(Key::End, true)] {
        rows.clear();
        ui.frame(input, BOUNDS, 8.0, |f| {
            f.virtual_list(1, 80.0, 100_000, &mut list, |f, i, selected| {
                rows.push(i);
                f.list_item(i as u64 + 100, "App", selected, true)
            });
        });
        assert!(rows.len() <= 2);
    }
    assert_eq!(list.selected, Some(99_999));
    assert_eq!(rows, vec![99_998, 99_999]);
}
#[test]
fn virtual_list_exposes_keyboard_and_pointer_selection_on_custom_rows() {
    let mut ui = Ui::default();
    let mut list = ListState { selected: Some(0), ..ListState::default() };
    let render = |ui: &mut Ui, list: &mut ListState, input: Input| {
        ui.frame(input, BOUNDS, 0.0, |f| {
            f.virtual_list_rows(1, 120.0, 3, list, 40.0, |f, index, _| {
                let bounds = f.reserve(40.0);
                f.target(100 + index as u64, bounds, "Option", true, Role::ListItem)
            });
        });
    };
    render(&mut ui, &mut list, Input::default());
    assert!(ui.semantics().iter().any(|n| n.id == 100 && n.selected));
    render(&mut ui, &mut list, key(Key::Tab, true));
    render(&mut ui, &mut list, key(Key::Down, true));
    assert_eq!(list.selected, Some(1));
    assert!(ui.semantics().iter().any(|n| n.id == 101 && n.selected));
    assert!(!ui.semantics().iter().any(|n| n.id == 100 && n.selected));
    render(&mut ui, &mut list, Input { pointer: Some([20.0, 100.0]), pressed: true, ..Input::default() });
    render(&mut ui, &mut list, Input { pointer: Some([20.0, 100.0]), released: true, ..Input::default() });
    assert_eq!(list.selected, Some(2));
    assert!(ui.semantics().iter().any(|n| n.id == 102 && n.selected));
    assert!(!ui.semantics().iter().any(|n| n.id == 101 && n.selected));
}
#[test]
fn tooltip_waits_on_deadline_then_reveals_without_polling() {
    let mut ui = Ui::default();
    ui.frame(pointer(false, false), BOUNDS, 8.0, |f| {
        let r = f.button(1, "Help");
        f.tooltip(2, r, "Details");
    });
    assert_eq!(ui.next_wakeup(), Some(0.5));
    ui.frame(
        Input {
            time: 0.6,
            ..pointer(false, false)
        },
        BOUNDS,
        8.0,
        |f| {
            let r = f.button(1, "Help");
            f.tooltip(2, r, "Details");
        },
    );
    assert!(ui.needs_redraw());
    ui.frame(
        Input {
            time: 1.6,
            ..pointer(false, false)
        },
        BOUNDS,
        8.0,
        |f| {
            let r = f.button(1, "Help");
            f.tooltip(2, r, "Details");
        },
    );
    assert!(ui.semantics().iter().any(|n| n.role == Role::Tooltip));
    assert!(!ui.needs_redraw());
    assert_eq!(ui.next_wakeup(), None);
}
#[test]
fn tab_parts_have_distinct_semantic_identity_from_neighbor_controls() {
    let mut ui = Ui::default();
    let mut selected = 0;
    ui.frame(Input::default(), BOUNDS, 8.0, |f| {
        f.tabs(20, &["A", "B"], &mut selected, true);
        f.button(21, "Neighbor");
    });
    let ids: std::collections::HashSet<_> = ui.semantics().iter().map(|n| (n.id, n.part)).collect();
    assert_eq!(ids.len(), ui.semantics().len());
}
#[test]
fn cached_text_is_bounded_and_unused_entries_are_reclaimed() {
    let mut ui = Ui::default();
    ui.frame(Input::default(), BOUNDS, 8.0, |f| {
        for i in 0..2000 {
            f.label(format!("Label {i}"));
        }
    });
    assert!(ui.cached_text_bytes() <= 128 * 1024);
    for _ in 0..60 {
        ui.frame(Input::default(), BOUNDS, 8.0, |f| f.label("Only"));
    }
    assert_eq!(ui.cached_text_bytes(), 4);
}

#[test]
fn angular_checkbox_keeps_pointer_focus_layer_until_blur_and_disables_it() {
    let mut ui = Ui::default();
    ui.ripple_style = RippleStyle::AngularMaterial;
    let mut checked = false;
    for input in [pointer(true, false), pointer(false, true)] {
        ui.frame(input, BOUNDS, 8.0, |f| {
            f.checkbox_bool(1, "Choice", &mut checked, true);
        });
    }
    assert!(checked);
    let theme = ui.theme;
    let commands = ui.frame(
        Input {
            time: 1.0,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            f.checkbox_bool(1, "Choice", &mut checked, true);
        },
    );
    assert!(commands.iter().any(|command| matches!(command, DrawCommand::RoundedRect { bounds, radius, color } if bounds.width == 40.0 && *radius == 20.0 && *color == theme.primary.opacity(0.12))));
    let commands = ui.frame(
        Input {
            time: 2.0,
            blurred: true,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            f.checkbox_bool(1, "Choice", &mut checked, true);
        },
    );
    assert!(!commands.iter().any(|command| matches!(command, DrawCommand::RoundedRect { bounds, radius, color } if bounds.width == 40.0 && *radius == 20.0 && *color == theme.primary.opacity(0.12))));
    let commands = ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
        f.checkbox_bool(1, "Choice", &mut checked, false);
    });
    assert!(!commands.iter().any(|command| matches!(command, DrawCommand::RoundedRect { bounds, radius, .. } if bounds.width == 40.0 && *radius == 20.0)));
}

#[test]
fn checkbox_error_is_independent_of_value_and_disabled_state() {
    let mut ui = Ui::default();
    for initial in [
        CheckState::Unchecked,
        CheckState::Checked,
        CheckState::Mixed,
    ] {
        let mut value = initial;
        let error = ui.theme.error;
        let commands = ui.frame(Input::default(), BOUNDS, 8.0, |f| {
            f.checkbox_with_error(1, "Choice: required", &mut value, true, true);
        });
        assert_eq!(value, initial);
        assert_eq!(ui.semantics()[0].checked, Some(initial));
        assert!(ui.semantics()[0].invalid);
        assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color, .. } if bounds.width == 18.0 && *color == error)));
        let commands = ui.frame(Input::default(), BOUNDS, 8.0, |f| {
            f.checkbox_with_error(1, "Choice: required", &mut value, false, true);
        });
        assert!(!commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color, .. } if bounds.width == 18.0 && *color == error)));
    }
    let mut value = CheckState::Unchecked;
    ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
        f.checkbox_with_error(1, "Choice", &mut value, true, true);
    });
    ui.frame(pointer(false, true), BOUNDS, 8.0, |f| {
        f.checkbox_with_error(1, "Choice", &mut value, true, true);
    });
    assert_eq!(value, CheckState::Checked);
}

#[test]
fn binary_checkbox_error_can_change_without_affecting_value() {
    let mut ui = Ui::default();
    let mut value = false;
    for error in [true, false, true] {
        ui.frame(Input::default(), BOUNDS, 8.0, |f| {
            f.checkbox_bool_with_error(1, "Required choice", &mut value, true, error);
        });
        assert_eq!(ui.semantics()[0].invalid, error);
        assert_eq!(ui.semantics()[0].checked, Some(CheckState::Unchecked));
        assert!(!value);
    }
    for expected in [true, false] {
        ui.frame(pointer(true, false), BOUNDS, 8.0, |f| {
            f.checkbox_bool_with_error(1, "Required choice", &mut value, true, true);
        });
        ui.frame(pointer(false, true), BOUNDS, 8.0, |f| {
            f.checkbox_bool_with_error(1, "Required choice", &mut value, true, true);
        });
        assert_eq!(value, expected);
        assert!(ui.semantics()[0].invalid);
        assert_eq!(
            ui.semantics()[0].checked,
            Some(if expected {
                CheckState::Checked
            } else {
                CheckState::Unchecked
            })
        );
    }
}

#[test]
fn unchecked_checkbox_does_not_paint_over_its_surface() {
    let mut ui = Ui::default();
    for error in [false, true] {
        let mut value = false;
        let commands = ui.frame(Input::default(), BOUNDS, 8.0, |f| {
            f.checkbox_bool_with_error(1, "Choice", &mut value, true, error);
        });
        // The 18dp control starts at (11, 11); its center must remain unpainted.
        assert!(!commands.iter().any(|command| matches!(command,
            DrawCommand::RoundedRect { bounds, .. }
                if bounds.x <= 20.0 && bounds.y <= 20.0
                    && bounds.x + bounds.width > 20.0
                    && bounds.y + bounds.height > 20.0
        )));
    }
}

#[test]
fn checkbox_fill_transitions_reverse_and_respect_reduced_motion() {
    fn draw(ui: &mut Ui, value: &mut bool, time: f64) -> Vec<DrawCommand> {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            8.0,
            |f| {
                f.checkbox_bool(1, "Choice", value, true);
            },
        )
    }
    fn fill_alpha(commands: &[DrawCommand]) -> u8 {
        commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::RoundedRect { bounds, color, .. }
                    if bounds.width == 18.0 && bounds.height == 18.0 =>
                {
                    Some(color.3)
                }
                _ => None,
            })
            .unwrap_or(0)
    }
    let mut ui = Ui::default();
    let mut value = false;
    draw(&mut ui, &mut value, 0.0);
    value = true;
    draw(&mut ui, &mut value, 0.01);
    assert_eq!(ui.semantics()[0].checked, Some(CheckState::Checked));
    let midway = fill_alpha(&draw(&mut ui, &mut value, 0.04));
    assert!(midway > 0 && midway < 255);
    value = false;
    let reversed = fill_alpha(&draw(&mut ui, &mut value, 0.05));
    assert!(reversed > 0 && reversed < 255);
    assert_eq!(fill_alpha(&draw(&mut ui, &mut value, 1.0)), 0);
    ui.motion_preference = MotionPreference::Reduced;
    value = true;
    assert_eq!(fill_alpha(&draw(&mut ui, &mut value, 1.01)), 255);
    value = false;
    assert_eq!(fill_alpha(&draw(&mut ui, &mut value, 1.02)), 0);
}

#[test]
fn checkbox_checkmark_is_delayed_and_fits_the_inner_box() {
    let mut ui = Ui::default();
    let mut value = false;
    fn draw(ui: &mut Ui, value: &mut bool, time: f64) -> Vec<DrawCommand> {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            8.0,
            |f| {
                f.checkbox_bool(1, "Choice", value, true);
            },
        )
    }
    draw(&mut ui, &mut value, 0.0);
    value = true;
    draw(&mut ui, &mut value, 0.01);
    let early = draw(&mut ui, &mut value, 0.08);
    assert!(
        !early
            .iter()
            .any(|c| matches!(c, DrawCommand::Polyline { .. }))
    );
    let partial = draw(&mut ui, &mut value, 0.12);
    assert!(
        partial
            .iter()
            .any(|c| matches!(c, DrawCommand::Polyline { .. }))
    );
    let settled = draw(&mut ui, &mut value, 0.25);
    let (points, count, width) = settled
        .iter()
        .find_map(|c| match c {
            DrawCommand::Polyline {
                points,
                count,
                width,
                ..
            } => Some((*points, *count, *width)),
            _ => None,
        })
        .expect("checkmark");
    assert_eq!(count, 3);
    assert!(width > 1.8 && width < 1.83);
    for point in points {
        assert!((13.0..=27.0).contains(&point[0]));
        assert!((13.0..=27.0).contains(&point[1]));
    }
}

#[test]
fn checkbox_uncheck_erases_from_start_and_fades() {
    let mut ui = Ui::default();
    let mut value = true;
    let draw = |ui: &mut Ui, value: &mut bool, time: f64| {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            8.0,
            |f| {
                f.checkbox_bool(1, "Choice", value, true);
            },
        )
    };
    draw(&mut ui, &mut value, 0.0);
    value = false;
    draw(&mut ui, &mut value, 0.01);
    let commands = draw(&mut ui, &mut value, 0.06);
    let (points, count, alpha) = commands
        .iter()
        .find_map(|c| match c {
            DrawCommand::Polyline {
                points,
                count,
                color,
                ..
            } => Some((points, *count, color.3)),
            _ => None,
        })
        .expect("fading mark");
    assert!(alpha > 0 && alpha < 255);
    let end = points[usize::from(count) - 1];
    assert!((end[0] - (13.0 + 22.79 * 14.0 / 24.0)).abs() < 0.01);
    assert!((end[1] - (13.0 + 4.59 * 14.0 / 24.0)).abs() < 0.01);
}

#[test]
fn checkbox_mixed_to_checked_rotates_over_reference_duration() {
    let mut ui = Ui::default();
    let mut state = CheckState::Mixed;
    let draw = |ui: &mut Ui, state: &mut CheckState, time: f64| {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            8.0,
            |f| {
                f.checkbox(1, "Choice", state, true);
            },
        )
    };
    draw(&mut ui, &mut state, 0.0);
    state = CheckState::Checked;
    draw(&mut ui, &mut state, 0.01);
    let middle = draw(&mut ui, &mut state, 0.10);
    let paths: Vec<_> = middle
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Polyline { count, color, .. } => Some((*count, color.3)),
            _ => None,
        })
        .collect();
    assert!(paths.iter().any(|&(n, a)| n == 3 && a > 0 && a < 255));
    assert!(paths.iter().any(|&(n, a)| n == 2 && a > 0 && a < 255));
    let settled = draw(&mut ui, &mut state, 0.6);
    assert!(
        settled
            .iter()
            .any(|c| matches!(c, DrawCommand::Polyline { count: 3, color, .. } if color.3 == 255))
    );
    assert!(
        !settled
            .iter()
            .any(|c| matches!(c, DrawCommand::Polyline { count: 2, .. }))
    );
}

#[test]
fn angular_mixed_mark_rotates_without_shrinking_and_finishes_before_checkmark() {
    let mut ui = Ui::default();
    ui.ripple_style = RippleStyle::AngularMaterial;
    let mut value = CheckState::Mixed;
    let draw = |ui: &mut Ui, value: &mut CheckState, time: f64| {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            8.0,
            |f| {
                f.checkbox(1, "Choice", value, true);
            },
        )
    };
    draw(&mut ui, &mut value, 0.0);
    value = CheckState::Checked;
    draw(&mut ui, &mut value, 0.01);
    let middle = draw(&mut ui, &mut value, 0.05);
    let points = middle
        .iter()
        .find_map(|c| match c {
            DrawCommand::Polyline {
                points,
                count: 2,
                width,
                ..
            } if *width == 2.0 => Some(points),
            _ => None,
        })
        .expect("mixed mark");
    let length = (points[1][0] - points[0][0]).hypot(points[1][1] - points[0][1]);
    assert!((length - 14.0).abs() < 0.001);
    assert_ne!(points[0][1], points[1][1]);
    let late = draw(&mut ui, &mut value, 0.12);
    assert!(
        !late
            .iter()
            .any(|c| matches!(c, DrawCommand::Polyline { count: 2, width, .. } if *width == 2.0))
    );
    assert!(late.iter().any(|c| matches!(c, DrawCommand::Polyline { width, color, .. } if *width < 2.0 && color.3 > 0 && color.3 < 255)));
}

#[test]
fn angular_checked_to_mixed_uses_exit_curve_for_mixed_mark() {
    let mut ui = Ui::default();
    ui.ripple_style = RippleStyle::AngularMaterial;
    let mut value = CheckState::Checked;
    let draw = |ui: &mut Ui, value: &mut CheckState, time: f64| {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            8.0,
            |f| {
                f.checkbox(1, "Choice", value, true);
            },
        )
    };
    draw(&mut ui, &mut value, 0.0);
    value = CheckState::Mixed;
    draw(&mut ui, &mut value, 0.01);
    let mid = draw(&mut ui, &mut value, 0.055);
    let alpha = mid
        .iter()
        .find_map(|c| match c {
            DrawCommand::Polyline {
                count: 2,
                width,
                color,
                ..
            } if *width == 2.0 => Some(color.3),
            _ => None,
        })
        .expect("mixed mark");
    assert!((alpha as i32 - 128).abs() <= 2);
    let end = draw(&mut ui, &mut value, 0.11);
    assert!(end.iter().any(|c| matches!(c, DrawCommand::Polyline { count: 2, width, color, .. } if *width == 2.0 && color.3 == 255)));
}

#[test]
fn angular_unchecked_mixed_scales_and_reverses_the_mark() {
    let mut ui = Ui::default();
    ui.ripple_style = RippleStyle::AngularMaterial;
    let mut value = CheckState::Unchecked;
    let draw = |ui: &mut Ui, value: &mut CheckState, time: f64| {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            8.0,
            |f| {
                f.checkbox(1, "Choice", value, true);
            },
        )
    };
    draw(&mut ui, &mut value, 0.0);
    value = CheckState::Mixed;
    draw(&mut ui, &mut value, 0.01);
    let partial = draw(&mut ui, &mut value, 0.055);
    let length = |commands: &[DrawCommand]| {
        commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::Polyline {
                    points,
                    count: 2,
                    width,
                    ..
                } if *width == 2.0 => {
                    Some((points[1][0] - points[0][0]).hypot(points[1][1] - points[0][1]))
                }
                _ => None,
            })
            .unwrap_or(0.0)
    };
    assert!((length(&partial) - 7.0).abs() < 0.1);
    value = CheckState::Unchecked;
    let reversed = draw(&mut ui, &mut value, 0.06);
    assert!(length(&reversed) > 0.0 && length(&reversed) < 14.0);
    assert_eq!(length(&draw(&mut ui, &mut value, 1.0)), 0.0);
    ui.motion_preference = MotionPreference::Reduced;
    value = CheckState::Mixed;
    assert!((length(&draw(&mut ui, &mut value, 1.01)) - 14.0).abs() < 0.001);
}

#[test]
fn disabled_switch_label_is_dimmed_in_both_layouts() {
    let mut ui = Ui::default();
    let mut first = false;
    let mut second = false;
    let commands = ui.frame(Input::default(), BOUNDS, 8.0, |f| {
        f.switch(101, "Leading", &mut first, false);
        f.switch_trailing(102, "Trailing", &mut second, false);
    });
    for label in ["Leading", "Trailing"] {
        let color = commands.iter().find_map(|cmd| match cmd {
            DrawCommand::Text { text, color, .. } if text.as_ref() == label => Some(color),
            _ => None,
        }).unwrap();
        assert_eq!(color.3, 97);
    }
}
