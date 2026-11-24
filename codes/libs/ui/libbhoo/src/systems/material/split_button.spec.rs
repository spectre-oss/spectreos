use bhoo::*;
const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 300.0,
    height: 100.0,
};
const B: Rect = Rect {
    x: 10.0,
    y: 10.0,
    width: 200.0,
    height: 48.0,
};
fn render(
    ui: &mut Ui,
    input: Input,
    expanded: bool,
    enabled: bool,
    variant: ButtonVariant,
) -> (Response, Response) {
    let mut out = None;
    ui.frame(input, VIEW, 0.0, |f| {
        out = f.split_button(
            10,
            B,
            "Save",
            "More save options",
            Options {
                enabled,
                selected: expanded,
                variant,
            },
        )
    });
    out.unwrap()
}
fn click(ui: &mut Ui, x: f32, time: f64, expanded: bool, enabled: bool) -> (Response, Response) {
    render(
        ui,
        Input {
            pointer: Some([x, 34.0]),
            pressed: true,
            time,
            ..Input::default()
        },
        expanded,
        enabled,
        ButtonVariant::Filled,
    );
    render(
        ui,
        Input {
            pointer: Some([x, 34.0]),
            released: true,
            time: time + 0.01,
            ..Input::default()
        },
        expanded,
        enabled,
        ButtonVariant::Filled,
    )
}
#[test]
fn split_button_has_separate_actions_and_menu_state() {
    let mut ui = Ui::default();
    let (primary, menu) = render(
        &mut ui,
        Input::default(),
        false,
        true,
        ButtonVariant::Filled,
    );
    assert_eq!(primary.bounds.width, 150.0);
    assert_eq!(menu.bounds.x, 162.0);
    assert_eq!(menu.bounds.width, 48.0);
    assert_eq!(&*ui.semantics()[1].label, "More save options, collapsed");
    let (primary, menu) = click(&mut ui, 40.0, 0.1, false, true);
    assert!(primary.clicked);
    assert!(!menu.clicked);
    let (primary, menu) = click(&mut ui, 180.0, 0.3, false, true);
    assert!(!primary.clicked);
    assert!(menu.clicked);
    render(
        &mut ui,
        Input {
            time: 0.5,
            ..Input::default()
        },
        true,
        true,
        ButtonVariant::Outlined,
    );
    assert_eq!(&*ui.semantics()[1].label, "More save options, expanded");
    let (primary, menu) = click(&mut ui, 180.0, 0.7, false, false);
    assert!(!primary.clicked && !menu.clicked);
}
#[test]
fn keyboard_focuses_both_actions_and_activates_the_focused_trigger() {
    let mut ui = Ui::default();
    render(&mut ui, Input::default(), false, true, ButtonVariant::Tonal);
    let key = |key, time| Input {
        key: Some(KeyInput {
            key,
            pressed: true,
            repeat: false,
            shift: false,
        }),
        time,
        ..Input::default()
    };
    render(
        &mut ui,
        key(Key::Tab, 0.1),
        false,
        true,
        ButtonVariant::Tonal,
    );
    assert!(ui.semantics().iter().any(|n| n.id == 10 && n.focused));
    render(
        &mut ui,
        key(Key::Tab, 0.2),
        false,
        true,
        ButtonVariant::Tonal,
    );
    assert!(ui.semantics().iter().any(|n| n.id == 11 && n.focused));
    let (primary, menu) = render(
        &mut ui,
        key(Key::Enter, 0.3),
        false,
        true,
        ButtonVariant::Tonal,
    );
    assert!(!primary.clicked && menu.clicked);
}
#[test]
fn expanded_chevron_rotates_and_reduced_motion_snaps() {
    let mut ui = Ui::default();
    let sample = |ui: &mut Ui, time: f64, expanded: bool| {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            VIEW,
            0.0,
            |f| {
                f.split_button(
                    10,
                    B,
                    "Save",
                    "More",
                    Options {
                        selected: expanded,
                        variant: ButtonVariant::Filled,
                        ..Options::default()
                    },
                );
            },
        )
        .iter()
        .find_map(|c| {
            if let DrawCommand::Line { from, to, .. } = c {
                Some((from[1], to[1]))
            } else {
                None
            }
        })
        .unwrap()
    };
    let down = sample(&mut ui, 0.0, false);
    assert!(down.0 < down.1);
    sample(&mut ui, 0.1, true);
    let up = sample(&mut ui, 1.0, true);
    assert!(up.0 > up.1);
    ui.motion_preference = MotionPreference::Reduced;
    let back = sample(&mut ui, 1.1, false);
    assert!(back.0 < back.1);
    assert!(!ui.needs_redraw());
}
#[test]
fn split_sizes_rtl_and_independent_disabled_states() {
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 500.0,
        height: 300.0,
    };
    for (size, height) in [
        (SplitButtonSize::ExtraSmall, 32.0),
        (SplitButtonSize::Small, 40.0),
        (SplitButtonSize::Medium, 56.0),
        (SplitButtonSize::Large, 96.0),
        (SplitButtonSize::ExtraLarge, 136.0),
    ] {
        let mut ui = Ui::default();
        let hit = f32::max(height, 48.0);
        let bounds = Rect {
            x: 10.0,
            y: 10.0,
            width: 400.0,
            height: hit,
        };
        let mut out = None;
        let commands = ui.frame(Input::default(), view, 0.0, |f| {
            out = f.split_button_with(
                10,
                bounds,
                "Save",
                "More",
                SplitButtonOptions {
                    size,
                    rtl: true,
                    primary_enabled: true,
                    menu_enabled: false,
                    ..SplitButtonOptions::default()
                },
            )
        });
        let (primary, menu) = out.unwrap();
        assert_eq!(menu.bounds.x, bounds.x);
        assert_eq!(menu.bounds.width, hit);
        assert_eq!(primary.bounds.x, bounds.x + hit + 2.0);
        assert!(primary.enabled);
        assert!(!menu.enabled);
        assert!(
            commands
                .iter()
                .any(|c| matches!(c,DrawCommand::RoundedRect{bounds,..} if bounds.height==height))
        );
    }
}

#[test]
fn disabled_outlined_halves_keep_the_outline_and_transparent_interior() {
    let mut ui = Ui::default();
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 100.0,
    };
    let mut out = None;
    let commands = ui.frame(Input::default(), view, 0.0, |f| {
        f.paint_rect(view, 0.0, f.theme().primary_container);
        out = f.split_button_with(
            10,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 48.0,
            },
            "Save",
            "Options",
            SplitButtonOptions {
                variant: ButtonVariant::Outlined,
                primary_enabled: false,
                menu_enabled: false,
                ..SplitButtonOptions::default()
            },
        );
    });
    let (primary, menu) = out.unwrap();
    assert!(!primary.enabled && !menu.enabled);
    assert_eq!(commands.iter().filter(|c| matches!(c, DrawCommand::Arc { color, .. } if *color == ui.theme.outline.opacity(0.1))).count(), 8);
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::RoundedRect { .. }))
            .count(),
        1
    );
}

#[test]
fn elevated_split_shadows_follow_inner_and_outer_corner_shapes() {
    let mut ui = Ui::default();
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 200.0,
    };
    let commands = ui.frame(Input::default(), view, 0.0, |f| {
        f.split_button_with(
            10,
            Rect {
                x: 20.0,
                y: 20.0,
                width: 300.0,
                height: 136.0,
            },
            "Save",
            "More",
            SplitButtonOptions {
                size: SplitButtonSize::ExtraLarge,
                variant: ButtonVariant::Elevated,
                ..SplitButtonOptions::default()
            },
        );
    });
    let radii: Vec<_> = commands
        .iter()
        .filter_map(|c| {
            if let DrawCommand::Shadow { radius, .. } = c {
                Some(*radius)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        radii,
        vec![
            68.0, 68.0, 68.0, 12.0, 12.0, 12.0, 12.0, 12.0, 12.0, 68.0, 68.0, 68.0
        ]
    );
}

#[test]
fn split_inner_corners_follow_size_and_interaction_tokens() {
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 500.0,
        height: 200.0,
    };
    fn radii(
        ui: &mut Ui,
        view: Rect,
        size: SplitButtonSize,
        height: f32,
        input: Input,
        expanded: bool,
        rtl: bool,
    ) -> Vec<f32> {
        ui.frame(input, view, 0.0, |f| {
            f.split_button_with(
                10,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 400.0,
                    height,
                },
                "Save",
                "More",
                SplitButtonOptions {
                    size,
                    expanded,
                    rtl,
                    ..SplitButtonOptions::default()
                },
            );
        })
        .iter()
        .filter_map(|c| {
            if let DrawCommand::RoundedRect { radius, color, .. } = c {
                (*color == ui.theme.primary).then_some(*radius)
            } else {
                None
            }
        })
        .collect()
    }
    for (size, height, inner, active) in [
        (SplitButtonSize::ExtraSmall, 48.0, 4.0, 8.0),
        (SplitButtonSize::Small, 48.0, 4.0, 12.0),
        (SplitButtonSize::Medium, 56.0, 4.0, 12.0),
        (SplitButtonSize::Large, 96.0, 8.0, 20.0),
        (SplitButtonSize::ExtraLarge, 136.0, 12.0, 20.0),
    ] {
        for rtl in [false, true] {
            let mut ui = Ui::default();
            ui.motion_preference = MotionPreference::Reduced;
            let base = radii(&mut ui, view, size, height, Input::default(), false, rtl);
            assert!((base[if rtl { 0 } else { 1 }] - inner).abs() < 0.001);
            let x = if rtl { height + 12.0 } else { 10.0 };
            let hover = radii(
                &mut ui,
                view,
                size,
                height,
                Input {
                    pointer: Some([x, height / 2.0]),
                    time: 0.1,
                    ..Input::default()
                },
                false,
                rtl,
            );
            assert!((hover[if rtl { 0 } else { 1 }] - active).abs() < 0.001);
            let press = radii(
                &mut ui,
                view,
                size,
                height,
                Input {
                    pointer: Some([x, height / 2.0]),
                    pressed: true,
                    time: 0.2,
                    ..Input::default()
                },
                false,
                rtl,
            );
            assert!((press[if rtl { 0 } else { 1 }] - active).abs() < 0.001);
            let expanded = radii(
                &mut ui,
                view,
                size,
                height,
                Input {
                    time: 0.3,
                    ..Input::default()
                },
                true,
                rtl,
            );
            let outer = match size {
                SplitButtonSize::ExtraSmall => 16.0,
                SplitButtonSize::Small => 20.0,
                SplitButtonSize::Medium => 28.0,
                SplitButtonSize::Large => 48.0,
                SplitButtonSize::ExtraLarge => 68.0,
            };
            assert!((expanded[if rtl { 3 } else { 2 }] - outer).abs() < 0.001);
        }
    }
}

#[test]
fn split_corner_transition_reverses_without_jumping() {
    let mut ui = Ui::default();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 56.0,
    };
    fn sample(ui: &mut Ui, bounds: Rect, time: f64, hover: bool) -> f32 {
        ui.frame(
            Input {
                time,
                pointer: hover.then_some([30.0, 28.0]),
                ..Input::default()
            },
            bounds,
            0.0,
            |f| {
                f.split_button_with(
                    10,
                    bounds,
                    "Save",
                    "More",
                    SplitButtonOptions {
                        size: SplitButtonSize::Medium,
                        ..SplitButtonOptions::default()
                    },
                );
            },
        )
        .iter()
        .filter_map(|c| {
            if let DrawCommand::RoundedRect { radius, color, .. } = c {
                (*color == ui.theme.primary).then_some(*radius)
            } else {
                None
            }
        })
        .nth(1)
        .unwrap()
    }
    assert!((sample(&mut ui, bounds, 0.0, false) - 4.0).abs() < 0.001);
    sample(&mut ui, bounds, 0.01, true);
    let mid = sample(&mut ui, bounds, 0.06, true);
    assert!(mid > 4.0 && mid < 12.0);
    let reversal = sample(&mut ui, bounds, 0.06, false);
    assert!((reversal - mid).abs() < 0.001);
    let end = sample(&mut ui, bounds, 1.0, false);
    assert!((end - 4.0).abs() < 0.05);
}

#[test]
fn keyboard_focus_does_not_morph_split_inner_corner() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let draw = |ui: &mut Ui, input: Input| {
        ui.frame(input, VIEW, 0.0, |f| {
            f.split_button_with(10, B, "Save", "More", SplitButtonOptions::default());
        })
    };
    draw(&mut ui, Input::default());
    let commands = draw(
        &mut ui,
        Input {
            time: 0.1,
            key: Some(KeyInput {
                key: Key::Tab,
                pressed: true,
                repeat: false,
                shift: false,
            }),
            ..Input::default()
        },
    );
    assert!(ui.semantics().iter().any(|n| n.id == 10 && n.focused));
    let radii: Vec<_> = commands
        .iter()
        .filter_map(|c| {
            if let DrawCommand::RoundedRect { radius, color, .. } = c {
                (*color == ui.theme.primary).then_some(*radius)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(radii[1], 4.0);
}

#[test]
fn pressing_expanded_trailing_button_uses_pressed_corner() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let commands = ui.frame(
        Input {
            pointer: Some([180.0, 34.0]),
            pressed: true,
            ..Input::default()
        },
        VIEW,
        0.0,
        |f| {
            f.split_button_with(
                10,
                B,
                "Save",
                "More",
                SplitButtonOptions {
                    expanded: true,
                    ..SplitButtonOptions::default()
                },
            );
        },
    );
    let radii: Vec<_> = commands
        .iter()
        .filter_map(|c| {
            if let DrawCommand::RoundedRect { radius, color, .. } = c {
                (*color == ui.theme.primary).then_some(*radius)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(radii[2], 12.0);
}

#[test]
fn expanded_trailing_button_has_persistent_checked_state_layer() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    for enabled in [true, false] {
        for expanded in [false, true] {
            let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
                f.split_button_with(
                    10,
                    B,
                    "Save",
                    "More",
                    SplitButtonOptions {
                        expanded,
                        menu_enabled: enabled,
                        ..SplitButtonOptions::default()
                    },
                );
            });
            let foreground = if enabled {
                ui.theme.on_primary
            } else {
                ui.theme.text.opacity(0.38)
            };
            let overlay = foreground.opacity(0.12);
            let count = commands
                .iter()
                .filter(
                    |c| matches!(c, DrawCommand::RoundedRect { color, .. } if *color == overlay),
                )
                .count();
            assert_eq!(count, if expanded { 2 } else { 0 });
        }
    }
}

mod leading_icon_tests {
    use crate::*;

    #[test]
    fn split_button_leading_icon_paints_shape() {
        let mut ui = Ui::default();
        let bounds = Rect { x: 0.0, y: 0.0, width: 260.0, height: 80.0 };
        let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
            f.split_button_with(
                10,
                Rect { x: 10.0, y: 10.0, width: 200.0, height: 56.0 },
                "Edit",
                "More",
                SplitButtonOptions { leading_icon: Some(MaterialIcon::Edit), ..SplitButtonOptions::default() },
            );
        });
        assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Shape { .. })));
    }
}
