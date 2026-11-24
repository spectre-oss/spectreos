use bhoo::*;
const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 400.0,
    height: 400.0,
};
const ACTIONS: [ToolbarAction<'static>; 3] = [
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
fn frame(ui: &mut Ui, state: &mut ToolbarState, kind: ToolbarKind, rtl: bool, input: Input) {
    ui.frame(input, VIEW, 0.0, |f| {
        let bounds = if kind == ToolbarKind::FloatingVertical {
            Rect {
                x: 0.0,
                y: 0.0,
                width: 64.0,
                height: 160.0,
            }
        } else {
            Rect {
                x: 0.0,
                y: 0.0,
                width: 160.0,
                height: 64.0,
            }
        };
        f.toolbar_roving(100, bounds, kind, false, &ACTIONS, state, rtl);
        f.target(
            200,
            Rect {
                x: 250.0,
                y: 200.0,
                width: 50.0,
                height: 50.0,
            },
            "After",
            true,
            Role::Button,
        );
    });
}
fn key(k: Key, t: f64) -> Input {
    Input {
        time: t,
        key: Some(KeyInput {
            key: k,
            pressed: true,
            repeat: false,
            shift: false,
        }),
        ..Input::default()
    }
}
#[test]
fn toolbar_roves_skips_disabled_wraps_and_tabs_out() {
    for (kind, rtl, forward) in [
        (ToolbarKind::Docked, false, Key::Right),
        (ToolbarKind::Docked, true, Key::Left),
        (ToolbarKind::FloatingVertical, false, Key::Down),
    ] {
        let mut ui = Ui::default();
        let mut state = ToolbarState::default();
        frame(&mut ui, &mut state, kind, rtl, Input::default());
        frame(&mut ui, &mut state, kind, rtl, key(Key::Tab, 0.1));
        assert_eq!(ui.focused(), Some(100));
        frame(&mut ui, &mut state, kind, rtl, key(forward, 0.2));
        assert_eq!(ui.focused(), Some(102));
        frame(&mut ui, &mut state, kind, rtl, key(forward, 0.3));
        assert_eq!(ui.focused(), Some(100));
        frame(&mut ui, &mut state, kind, rtl, key(Key::End, 0.4));
        assert_eq!(ui.focused(), Some(102));
        frame(&mut ui, &mut state, kind, rtl, key(Key::Tab, 0.5));
        assert_eq!(ui.focused(), Some(200));
    }
}

#[test]
fn overflow_keeps_hidden_actions_reachable_and_disabled_items_inert() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ToolbarState::default();
    let bounds = Rect {
        x: 0.0,
        y: 120.0,
        width: 112.0,
        height: 64.0,
    };
    fn draw(ui: &mut Ui, state: &mut ToolbarState, bounds: Rect, input: Input) -> Option<usize> {
        let mut selected = None;
        ui.frame(input, VIEW, 0.0, |f| {
            selected = f.toolbar_with_overflow(
                100,
                bounds,
                ToolbarKind::Docked,
                false,
                &ACTIONS,
                state,
                false,
            );
        });
        selected
    }
    draw(&mut ui, &mut state, bounds, Input::default());
    let trigger = ui.semantics().iter().find(|n| n.id == 101).unwrap().bounds;
    let p = [trigger.x + 24.0, trigger.y + 24.0];
    for (pressed, released) in [(true, false), (false, true)] {
        draw(
            &mut ui,
            &mut state,
            bounds,
            Input {
                pointer: Some(p),
                pressed,
                released,
                ..Input::default()
            },
        );
    }
    assert!(
        ui.semantics()
            .iter()
            .any(|n| n.id == 166 && !n.enabled && n.label.as_ref() == "Copy")
    );
    assert_eq!(ui.focused(), Some(167));
    draw(&mut ui, &mut state, bounds, key(Key::Escape, 0.1));
    draw(
        &mut ui,
        &mut state,
        bounds,
        Input {
            time: 0.2,
            ..Input::default()
        },
    );
    assert_eq!(ui.focused(), Some(101));
    for (pressed, released) in [(true, false), (false, true)] {
        draw(
            &mut ui,
            &mut state,
            bounds,
            Input {
                pointer: Some(p),
                pressed,
                released,
                time: 0.3,
                ..Input::default()
            },
        );
    }
    let paste = ui
        .semantics()
        .iter()
        .find(|n| n.id == 167 && n.enabled)
        .unwrap()
        .bounds;
    let p = [paste.x + 20.0, paste.y + 24.0];
    for (pressed, released) in [(true, false), (false, true)] {
        let selected = draw(
            &mut ui,
            &mut state,
            bounds,
            Input {
                pointer: Some(p),
                pressed,
                released,
                ..Input::default()
            },
        );
        if released {
            assert_eq!(selected, Some(2));
        }
    }
}

#[test]
fn overflow_touch_drag_scrolls_without_selecting() {
    let actions = vec![
        ToolbarAction {
            label: "Action",
            enabled: true
        };
        10
    ];
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ToolbarState::default();
    let bounds = Rect {
        x: 0.0,
        y: 300.0,
        width: 112.0,
        height: 64.0,
    };
    fn draw(
        ui: &mut Ui,
        state: &mut ToolbarState,
        actions: &[ToolbarAction<'_>],
        bounds: Rect,
        input: Input,
    ) -> Option<usize> {
        let mut result = None;
        ui.frame(input, VIEW, 0.0, |f| {
            result = f.toolbar_with_overflow(
                100,
                bounds,
                ToolbarKind::Docked,
                false,
                actions,
                state,
                false,
            );
        });
        result
    }
    draw(&mut ui, &mut state, &actions, bounds, Input::default());
    let b = ui.semantics().iter().find(|n| n.id == 101).unwrap().bounds;
    let p = [b.x + 24.0, b.y + 24.0];
    for (pressed, released) in [(true, false), (false, true)] {
        draw(
            &mut ui,
            &mut state,
            &actions,
            bounds,
            Input {
                pointer: Some(p),
                pressed,
                released,
                ..Input::default()
            },
        );
    }
    let first = ui.semantics().iter().find(|n| n.id == 170).unwrap().bounds;
    let start = ui.semantics().iter().find(|n| n.id == 167).unwrap().bounds;
    let x = start.x + 20.0;
    let y = start.y + 24.0;
    for (y, pressed, released) in [
        (y, true, false),
        (y - 80.0, false, false),
        (y - 80.0, false, true),
    ] {
        assert_eq!(
            draw(
                &mut ui,
                &mut state,
                &actions,
                bounds,
                Input {
                    pointer: Some([x, y]),
                    pointer_kind: PointerKind::Touch,
                    pressed,
                    released,
                    ..Input::default()
                }
            ),
            None
        );
    }
    let after = ui.semantics().iter().find(|n| n.id == 170).unwrap().bounds;
    assert!((first.y - after.y - 80.0).abs() < 0.01);
    assert!(state.overflow_open());
}

#[test]
fn docked_toolbar_uses_weighted_material_icons_and_embedded_primary_action() {
    let actions = [
        ToolbarAction {
            label: "Back",
            enabled: true,
        },
        ToolbarAction {
            label: "Forward",
            enabled: true,
        },
        ToolbarAction {
            label: "Add",
            enabled: true,
        },
        ToolbarAction {
            label: "Folder",
            enabled: true,
        },
        ToolbarAction {
            label: "More options",
            enabled: true,
        },
    ];
    let icons = [
        AppBarIcon::Back,
        AppBarIcon::Forward,
        AppBarIcon::Add,
        AppBarIcon::Folder,
        AppBarIcon::More,
    ];
    for rtl in [false, true] {
        let mut ui = Ui::default();
        let mut state = ToolbarState::default();
        let bounds = Rect {
            x: 0.0,
            y: 10.0,
            width: 400.0,
            height: 64.0,
        };
        let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
            f.docked_toolbar_with_overflow(
                100,
                bounds,
                false,
                &actions,
                &icons,
                Some(2),
                &mut state,
                rtl,
            );
        });
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds:b,color,.. } if *b==bounds && *color==ui.theme.control)));
        let nodes = ui.semantics();
        assert_eq!(nodes.len(), 5);
        let center = nodes.iter().find(|n| n.id == 102).unwrap().bounds;
        assert_eq!((center.width, center.height), (56.0, 56.0));
        assert!((center.x + 28.0 - 200.0).abs() < 0.01);
        let back = nodes.iter().find(|n| n.id == 100).unwrap().bounds;
        let forward = nodes.iter().find(|n| n.id == 101).unwrap().bounds;
        assert_eq!(back.x > forward.x, rtl);
        let shapes: Vec<_> = commands
            .iter()
            .filter_map(|c| {
                if let DrawCommand::Shape { bounds, .. } = c {
                    Some(*bounds)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(shapes.len(), 5);
        assert!(shapes.iter().all(|b| b.width == 24.0 && b.height == 24.0));
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds:b,color,.. } if *b==center && *color==ui.theme.primary)));
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, DrawCommand::Text { .. }))
        );
    }
}

#[test]
fn docked_overflow_shows_icons_with_logical_text_alignment() {
    let actions = [
        ToolbarAction {
            label: "Back",
            enabled: true,
        },
        ToolbarAction {
            label: "Forward",
            enabled: true,
        },
        ToolbarAction {
            label: "Tabs",
            enabled: true,
        },
    ];
    let icons = [AppBarIcon::Back, AppBarIcon::Forward, AppBarIcon::Tab];
    for rtl in [false, true] {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut state = ToolbarState::default();
        let bounds = Rect {
            x: 20.0,
            y: 120.0,
            width: 128.0,
            height: 64.0,
        };
        fn draw(
            ui: &mut Ui,
            state: &mut ToolbarState,
            bounds: Rect,
            actions: &[ToolbarAction<'_>],
            icons: &[AppBarIcon],
            rtl: bool,
            input: Input,
        ) -> Vec<DrawCommand> {
            ui.frame(input, VIEW, 0.0, |f| {
                f.docked_toolbar_with_overflow(
                    100, bounds, false, actions, icons, None, state, rtl,
                );
            })
        }
        draw(
            &mut ui,
            &mut state,
            bounds,
            &actions,
            &icons,
            rtl,
            Input::default(),
        );
        let b = ui.semantics().iter().find(|n| n.id == 101).unwrap().bounds;
        let p = [b.x + b.width / 2.0, b.y + b.height / 2.0];
        for (pressed, released) in [(true, false), (false, true)] {
            draw(
                &mut ui,
                &mut state,
                bounds,
                &actions,
                &icons,
                rtl,
                Input {
                    pointer: Some(p),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        let commands = draw(
            &mut ui,
            &mut state,
            bounds,
            &actions,
            &icons,
            rtl,
            Input {
                time: 0.1,
                ..Input::default()
            },
        );
        let menu = ui.semantics().iter().find(|n| n.id == 166).unwrap().bounds;
        let icon = commands
            .iter()
            .find_map(|c| {
                if let DrawCommand::Shape { bounds, .. } = c {
                    (bounds.y >= menu.y && bounds.y < menu.y + 48.0).then_some(*bounds)
                } else {
                    None
                }
            })
            .unwrap();
        let (text, style) = commands
            .iter()
            .find_map(|c| {
                if let DrawCommand::Text {
                    bounds,
                    text,
                    style,
                    ..
                } = c
                {
                    (text.as_ref() == "Forward").then_some((*bounds, *style))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(
            icon.x,
            if rtl {
                menu.x + menu.width - 40.0
            } else {
                menu.x + 16.0
            }
        );
        assert_eq!(style.align, if rtl { 2 } else { 0 });
        assert!(if rtl {
            text.x + text.width < icon.x
        } else {
            text.x > icon.x + icon.width
        });
    }
}

#[test]
fn docked_toolbar_scroll_hides_reverses_and_respects_touch_exploration() {
    let mut ui = Ui::default();
    let mut state = DockedToolbarScrollState::default();
    fn sample(
        ui: &mut Ui,
        state: &mut DockedToolbarScrollState,
        time: f64,
        delta: f32,
        accessible: bool,
    ) -> f32 {
        let mut offset = 0.0;
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            VIEW,
            0.0,
            |f| {
                offset = f.docked_toolbar_scroll_offset(state, delta, 64.0, accessible);
            },
        );
        offset
    }
    assert_eq!(sample(&mut ui, &mut state, 0.0, 0.0, false), 0.0);
    assert_eq!(sample(&mut ui, &mut state, 0.01, 1.0, false), 0.0);
    let mid = sample(&mut ui, &mut state, 0.08, 0.0, false);
    assert!(mid > 0.0 && mid < 64.0);
    let reverse = sample(&mut ui, &mut state, 0.08, -1.0, false);
    assert!((reverse - mid).abs() < 0.001);
    assert_eq!(sample(&mut ui, &mut state, 0.4, 0.0, false), 0.0);
    sample(&mut ui, &mut state, 0.5, 1.0, false);
    assert_eq!(sample(&mut ui, &mut state, 0.8, 0.0, false), 64.0);
    assert_eq!(sample(&mut ui, &mut state, 0.9, 1.0, true), 0.0);
    ui.motion_preference = MotionPreference::Reduced;
    assert_eq!(sample(&mut ui, &mut state, 1.0, 1.0, false), 64.0);
    assert_eq!(sample(&mut ui, &mut state, 1.1, -1.0, false), 0.0);
}

#[test]
fn floating_toolbar_scroll_uses_full_distance_and_reverses() {
    let mut ui=Ui::default();let mut state=DockedToolbarScrollState::default();
    fn sample(ui:&mut Ui,state:&mut DockedToolbarScrollState,time:f64,delta:f32)->f32 {
        let mut offset=0.0;ui.frame(Input{time,..Input::default()},VIEW,0.0,|f|{offset=f.floating_toolbar_scroll_offset(state,delta,72.0);});offset
    }
    assert_eq!(sample(&mut ui,&mut state,0.0,1.0),0.0);
    let mid=sample(&mut ui,&mut state,0.08,0.0);assert!(mid>0.0 && mid<72.0);
    assert!((sample(&mut ui,&mut state,0.08,-1.0)-mid).abs()<0.001);
    assert_eq!(sample(&mut ui,&mut state,0.4,0.0),0.0);
    ui.motion_preference=MotionPreference::Reduced;
    assert_eq!(sample(&mut ui,&mut state,0.5,1.0),72.0);
    assert_eq!(sample(&mut ui,&mut state,0.6,-1.0),0.0);
}

#[test]
fn docked_insets_follow_gravity_and_explicit_overrides_without_accumulation() {
    let bounds = Rect {
        x: 0.0,
        y: 10.0,
        width: 320.0,
        height: 104.0,
    };
    let inset = DockedToolbarInsets {
        top: 16.0,
        bottom: 24.0,
        ..DockedToolbarInsets::default()
    };
    let bottom = inset.content_bounds(bounds).unwrap();
    assert_eq!((bottom.y, bottom.height), (10.0, 80.0));
    let top = DockedToolbarInsets {
        edge: DockedToolbarEdge::Top,
        ..inset
    }
    .content_bounds(bounds)
    .unwrap();
    assert_eq!((top.y, top.height), (26.0, 88.0));
    let both = DockedToolbarInsets {
        apply_top: Some(true),
        apply_bottom: Some(true),
        fits_system_windows: false,
        ..inset
    }
    .content_bounds(bounds)
    .unwrap();
    assert_eq!((both.y, both.height), (26.0, 64.0));
    let none = DockedToolbarInsets {
        apply_top: Some(false),
        apply_bottom: Some(false),
        ..inset
    }
    .content_bounds(bounds)
    .unwrap();
    assert_eq!(none, bounds);
    assert_eq!(inset.content_bounds(bounds), Some(bottom));
    assert!(
        DockedToolbarInsets {
            bottom: 50.0,
            ..inset
        }
        .content_bounds(bounds)
        .is_none()
    );
}

#[test]
fn docked_inset_surface_covers_padding_and_keeps_controls_out_of_it() {
    let mut ui = Ui::default();
    let mut state = ToolbarState::default();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 88.0,
    };
    let icons = [AppBarIcon::Back, AppBarIcon::Forward, AppBarIcon::Add];
    let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.docked_toolbar_with_insets(
            100,
            bounds,
            DockedToolbarInsets {
                bottom: 24.0,
                ..DockedToolbarInsets::default()
            },
            false,
            &ACTIONS,
            &icons,
            Some(2),
            &mut state,
            false,
        );
    });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect {bounds:b,color,..} if *b==bounds && *color==ui.theme.control)));
    assert!(
        ui.semantics()
            .iter()
            .all(|node| node.bounds.y + node.bounds.height <= 64.0)
    );
}

#[test]
fn overflow_typeahead_reaches_offscreen_enabled_item() {
    let actions: Vec<_> = (0..10)
        .map(|i| ToolbarAction {
            label: if i == 9 { "Zoom" } else { "Action" },
            enabled: i != 8,
        })
        .collect();
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ToolbarState::default();
    let bounds = Rect {
        x: 0.0,
        y: 300.0,
        width: 112.0,
        height: 64.0,
    };
    fn draw(
        ui: &mut Ui,
        state: &mut ToolbarState,
        actions: &[ToolbarAction<'_>],
        bounds: Rect,
        input: Input,
    ) {
        ui.frame(input, VIEW, 0.0, |f| {
            f.toolbar_with_overflow(
                100,
                bounds,
                ToolbarKind::Docked,
                false,
                actions,
                state,
                false,
            );
        });
    }
    draw(&mut ui, &mut state, &actions, bounds, Input::default());
    let b = ui.semantics().iter().find(|n| n.id == 101).unwrap().bounds;
    let p = [b.x + 24.0, b.y + 24.0];
    for (pressed, released) in [(true, false), (false, true)] {
        draw(
            &mut ui,
            &mut state,
            &actions,
            bounds,
            Input {
                pointer: Some(p),
                pressed,
                released,
                ..Input::default()
            },
        );
    }
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        key(Key::Character('z'), 0.1),
    );
    assert_eq!(ui.focused(), Some(174));
    assert!(
        ui.semantics()
            .iter()
            .any(|n| n.id == 174 && n.enabled && n.bounds.height > 0.0)
    );
    draw(&mut ui, &mut state, &actions, bounds, key(Key::Enter, 0.2));
    assert!(!state.overflow_open());
}

#[test]
fn overflow_fling_continues_stops_at_touch_and_respects_reduced_motion() {
    let actions = vec![
        ToolbarAction {
            label: "Action",
            enabled: true
        };
        12
    ];
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ToolbarState::default();
    let bounds = Rect {
        x: 0.0,
        y: 300.0,
        width: 112.0,
        height: 64.0,
    };
    fn draw(
        ui: &mut Ui,
        state: &mut ToolbarState,
        actions: &[ToolbarAction<'_>],
        bounds: Rect,
        input: Input,
    ) {
        ui.frame(input, VIEW, 0.0, |f| {
            f.toolbar_with_overflow(
                100,
                bounds,
                ToolbarKind::Docked,
                false,
                actions,
                state,
                false,
            );
        });
    }
    draw(&mut ui, &mut state, &actions, bounds, Input::default());
    let b = ui.semantics().iter().find(|n| n.id == 101).unwrap().bounds;
    let p = [b.x + 24.0, b.y + 24.0];
    for (pressed, released) in [(true, false), (false, true)] {
        draw(
            &mut ui,
            &mut state,
            &actions,
            bounds,
            Input {
                pointer: Some(p),
                pressed,
                released,
                ..Input::default()
            },
        );
    }
    ui.motion_preference = MotionPreference::Full;
    let touch = |time, y, pressed, released| Input {
        time,
        pointer: Some([80.0, y]),
        pointer_kind: PointerKind::Touch,
        pressed,
        released,
        ..Input::default()
    };
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        touch(1.0, 180.0, true, false),
    );
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        touch(1.05, 130.0, false, false),
    );
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        touch(1.06, 130.0, false, true),
    );
    let before = ui
        .semantics()
        .iter()
        .find(|n| n.id == 170)
        .unwrap()
        .bounds
        .y;
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        Input {
            time: 1.12,
            ..Input::default()
        },
    );
    let after = ui
        .semantics()
        .iter()
        .find(|n| n.id == 170)
        .unwrap()
        .bounds
        .y;
    assert!(after < before);
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        touch(1.13, 130.0, true, false),
    );
    let stopped = ui
        .semantics()
        .iter()
        .find(|n| n.id == 170)
        .unwrap()
        .bounds
        .y;
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        touch(1.3, 130.0, false, false),
    );
    assert_eq!(
        ui.semantics()
            .iter()
            .find(|n| n.id == 170)
            .unwrap()
            .bounds
            .y,
        stopped
    );
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        Input {
            time: 1.31,
            cancelled: true,
            ..Input::default()
        },
    );
    ui.motion_preference = MotionPreference::Reduced;
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        touch(2.0, 180.0, true, false),
    );
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        touch(2.05, 160.0, false, false),
    );
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        touch(2.06, 160.0, false, true),
    );
    let reduced = ui
        .semantics()
        .iter()
        .find(|n| n.id == 171)
        .unwrap()
        .bounds
        .y;
    draw(
        &mut ui,
        &mut state,
        &actions,
        bounds,
        Input {
            time: 2.12,
            ..Input::default()
        },
    );
    assert_eq!(
        ui.semantics()
            .iter()
            .find(|n| n.id == 171)
            .unwrap()
            .bounds
            .y,
        reduced
    );
}

#[test]
fn reusing_toolbar_state_does_not_leak_docked_icon_layout() {
    let mut ui = Ui::default();
    let mut state = ToolbarState::default();
    let bounds = Rect {
        x: 0.0,
        y: 20.0,
        width: 300.0,
        height: 64.0,
    };
    let icons = [AppBarIcon::Back, AppBarIcon::Forward, AppBarIcon::Add];
    ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.docked_toolbar_with_overflow(
            100,
            bounds,
            false,
            &ACTIONS,
            &icons,
            Some(2),
            &mut state,
            false,
        );
    });
    let commands = ui.frame(
        Input {
            time: 0.1,
            ..Input::default()
        },
        VIEW,
        0.0,
        |f| {
            f.toolbar_with_overflow(
                100,
                bounds,
                ToolbarKind::Docked,
                false,
                &ACTIONS,
                &mut state,
                false,
            );
        },
    );
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::Shape { .. }))
            .count(),
        0
    );
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::Text { .. }))
            .count(),
        3
    );
    ui.frame(
        Input {
            time: 0.2,
            ..Input::default()
        },
        VIEW,
        0.0,
        |f| {
            f.docked_toolbar_with_overflow(
                100,
                bounds,
                false,
                &ACTIONS,
                &icons,
                Some(2),
                &mut state,
                false,
            );
        },
    );
    let commands = ui.frame(
        Input {
            time: 0.3,
            ..Input::default()
        },
        VIEW,
        0.0,
        |f| {
            f.toolbar_roving(
                100,
                bounds,
                ToolbarKind::Docked,
                false,
                &ACTIONS,
                &mut state,
                false,
            );
        },
    );
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::Shape { .. }))
            .count(),
        0
    );
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::Text { .. }))
            .count(),
        3
    );
}

#[test]
fn switching_to_roving_toolbar_closes_open_overflow_and_restores_focus() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = ToolbarState::default();
    let narrow = Rect {
        x: 0.0,
        y: 120.0,
        width: 112.0,
        height: 64.0,
    };
    let draw = |ui: &mut Ui, state: &mut ToolbarState, input: Input| {
        ui.frame(input, VIEW, 0.0, |f| {
            f.toolbar_with_overflow(
                100,
                narrow,
                ToolbarKind::Docked,
                false,
                &ACTIONS,
                state,
                false,
            );
        });
    };
    draw(&mut ui, &mut state, Input::default());
    let b = ui.semantics().iter().find(|n| n.id == 101).unwrap().bounds;
    let p = [b.x + 24.0, b.y + 24.0];
    for (pressed, released) in [(true, false), (false, true)] {
        draw(
            &mut ui,
            &mut state,
            Input {
                pointer: Some(p),
                pressed,
                released,
                ..Input::default()
            },
        );
    }
    assert!(state.overflow_open());
    ui.frame(
        Input {
            time: 0.1,
            ..Input::default()
        },
        VIEW,
        0.0,
        |f| {
            f.toolbar_roving(
                100,
                Rect {
                    width: 300.0,
                    ..narrow
                },
                ToolbarKind::Docked,
                false,
                &ACTIONS,
                &mut state,
                false,
            );
        },
    );
    assert!(!state.overflow_open());
    assert_eq!(ui.focused(), Some(100));
    assert!(!ui.semantics().iter().any(|n| n.role == Role::Menu));
}

#[test]
fn toolbar_keyboard_navigation_skips_clipped_actions() {
    fn draw(ui: &mut Ui, state: &mut ToolbarState, height: f32, input: Input) {
        ui.frame(input, VIEW, 0.0, |f| {
            f.within(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 64.0,
                    height,
                },
                |f| {
                    f.toolbar_roving(
                        100,
                        Rect {
                            x: 0.0,
                            y: 0.0,
                            width: 64.0,
                            height: 160.0,
                        },
                        ToolbarKind::FloatingVertical,
                        false,
                        &ACTIONS,
                        state,
                        false,
                    );
                },
            );
        });
    }
    let mut ui = Ui::default();
    let mut state = ToolbarState::default();
    draw(&mut ui, &mut state, 100.0, Input::default());
    draw(&mut ui, &mut state, 100.0, key(Key::Tab, 0.1));
    assert_eq!(ui.focused(), Some(100));
    draw(&mut ui, &mut state, 100.0, key(Key::Down, 0.2));
    assert_eq!(ui.focused(), Some(100));
    draw(&mut ui, &mut state, 110.0, key(Key::Down, 0.3));
    assert_eq!(ui.focused(), Some(102));
}

#[test]
fn floating_toolbar_fab_placement_and_activation_follow_direction() {
    let fab=ToolbarAction{label:"Compose",enabled:true};
    for (kind,rtl,pos,expected) in [
        (ToolbarKind::FloatingHorizontal,false,ToolbarFabPosition::End,Rect{x:194.,y:10.,width:56.,height:56.}),
        (ToolbarKind::FloatingHorizontal,true,ToolbarFabPosition::End,Rect{x:10.,y:10.,width:56.,height:56.}),
        (ToolbarKind::FloatingHorizontal,false,ToolbarFabPosition::Start,Rect{x:10.,y:10.,width:56.,height:56.}),
        (ToolbarKind::FloatingVertical,false,ToolbarFabPosition::End,Rect{x:10.,y:194.,width:56.,height:56.}),
        (ToolbarKind::FloatingVertical,true,ToolbarFabPosition::Start,Rect{x:10.,y:10.,width:56.,height:56.}),
    ] {
        let mut ui=Ui::default();let mut state=ToolbarState::default();
        let bounds=if kind==ToolbarKind::FloatingVertical {Rect{x:10.,y:10.,width:56.,height:240.}}else{Rect{x:10.,y:10.,width:240.,height:56.}};
        let mut chosen=None;
        let mut draw=|ui:&mut Ui,input:Input| {ui.frame(input,VIEW,0.0,|f|{chosen=f.floating_toolbar_with_fab(1000,bounds,kind,false,&ACTIONS,&mut state,rtl,fab,AppBarIcon::Add,pos);});};
        draw(&mut ui,Input::default());
        assert_eq!(ui.semantics().iter().find(|n|n.id==1132).unwrap().bounds,expected);
        let point=[expected.x+28.,expected.y+28.];
        draw(&mut ui,Input{pointer:Some(point),pressed:true,..Input::default()});
        draw(&mut ui,Input{pointer:Some(point),released:true,..Input::default()});
        assert_eq!(chosen,Some(ToolbarFabSelection::Fab));
    }
}

#[test]
fn expandable_floating_toolbar_morphs_reverses_and_snaps() {
    let mut ui=Ui::default();let mut state=ToolbarState::default();
    let bounds=Rect{x:10.,y:10.,width:240.,height:80.};
    fn sample(ui:&mut Ui,state:&mut ToolbarState,bounds:Rect,time:f64,expanded:bool)->(Rect,bool) {
        ui.frame(Input{time,..Input::default()},VIEW,0.0,|f|{f.floating_toolbar_with_fab_expanded(2000,bounds,ToolbarKind::FloatingHorizontal,false,&ACTIONS,state,false,ToolbarAction{label:"Create",enabled:true},AppBarIcon::Add,ToolbarFabPosition::End,expanded);});
        (ui.semantics().iter().find(|n|n.id==2132).unwrap().bounds,ui.semantics().iter().any(|n|n.id==2000 && n.enabled))
    }
    assert_eq!(sample(&mut ui,&mut state,bounds,0.0,true).0.width,56.0);
    sample(&mut ui,&mut state,bounds,0.01,false);
    let middle=sample(&mut ui,&mut state,bounds,0.03,false).0.width;
    assert!(middle>56.0 && middle<80.0);
    let reversed=sample(&mut ui,&mut state,bounds,0.03,true).0.width;
    assert!((reversed-middle).abs()<0.001);
    assert_eq!(sample(&mut ui,&mut state,bounds,1.0,true).0.width,56.0);
    ui.motion_preference=MotionPreference::Reduced;
    let (fab,has_actions)=sample(&mut ui,&mut state,bounds,1.1,false);
    assert_eq!(fab.width,80.0);assert!(!has_actions);
    assert_eq!(sample(&mut ui,&mut state,bounds,1.2,true).0.width,56.0);
}

#[test]
fn floating_toolbar_expansion_scroll_uses_directional_thresholds() {
    let mut state=FloatingToolbarExpansionScrollState::default();let mut expanded=true;
    assert!(!state.update(&mut expanded,20.0,false,40.0,40.0));assert!(expanded);
    assert!(state.update(&mut expanded,20.0,false,40.0,40.0));assert!(!expanded);
    assert!(!state.update(&mut expanded,-39.0,false,40.0,40.0));assert!(!expanded);
    assert!(state.update(&mut expanded,-1.0,false,40.0,40.0));assert!(expanded);
    assert!(!state.update(&mut expanded,30.0,true,40.0,40.0));assert!(expanded);
    assert!(state.update(&mut expanded,-70.0,true,40.0,40.0));assert!(!expanded);
    expanded=true;
    assert!(!state.update(&mut expanded,0.0,false,40.0,40.0));
    assert!(!state.update(&mut expanded,f32::NAN,false,40.0,40.0));assert!(expanded);
    assert!(state.update(&mut expanded,40.0,false,40.0,40.0));assert!(!expanded);
}

#[test]
fn floating_toolbar_and_fab_follow_standard_and_vibrant_color_roles() {
    let bounds=Rect{x:10.,y:10.,width:240.,height:80.};
    for vibrant in [false,true] {
        let mut ui=Ui::default();let theme=ui.theme;let mut state=ToolbarState::default();
        let commands=ui.frame(Input::default(),VIEW,0.0,|f|{
            f.floating_toolbar_with_fab_expanded(2000,bounds,ToolbarKind::FloatingHorizontal,vibrant,&ACTIONS,&mut state,false,ToolbarAction{label:"Create",enabled:true},AppBarIcon::Add,ToolbarFabPosition::End,true);
        });
        let toolbar=if vibrant{theme.primary_container}else{theme.control};
        let fab=if vibrant{theme.tertiary_container}else{theme.primary_container};
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color,..} if bounds.width==176. && *color==toolbar)));
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color,..} if bounds.width==56. && *color==fab)));
    }
}

#[test]
fn paired_floating_toolbar_uses_toolbar_and_fab_elevation_roles() {
    let bounds=Rect{x:10.,y:10.,width:240.,height:80.};let mut ui=Ui::default();let mut state=ToolbarState::default();
    ui.motion_preference=MotionPreference::Reduced;
    fn render(ui:&mut Ui,state:&mut ToolbarState,bounds:Rect,input:Input,expanded:bool)->Vec<DrawCommand> {
        ui.frame(input,VIEW,0.0,|f|{f.floating_toolbar_with_fab_expanded(2000,bounds,ToolbarKind::FloatingHorizontal,false,&ACTIONS,state,false,ToolbarAction{label:"Create",enabled:true},AppBarIcon::Add,ToolbarFabPosition::End,expanded);})
    }
    let idle=render(&mut ui,&mut state,bounds,Input::default(),true);
    assert_eq!(idle.iter().filter(|c|matches!(c,DrawCommand::Shadow{bounds,..} if bounds.width==176.)).count(),3);
    let fab_offset=|commands:&[DrawCommand],width:f32|commands.iter().filter_map(|c|match c{DrawCommand::Shadow{bounds,offset,..} if bounds.width==width=>Some(offset[1]),_=>None}).fold(0.0_f32,f32::max);
    let before=fab_offset(&idle,56.0);assert!(before>0.0);
    let hovered=render(&mut ui,&mut state,bounds,Input{pointer:Some([220.,50.]),..Input::default()},true);
    assert!(fab_offset(&hovered,56.0)>before);
    let collapsed=render(&mut ui,&mut state,bounds,Input::default(),false);
    assert!(!collapsed.iter().any(|c|matches!(c,DrawCommand::Shadow{bounds,..} if bounds.width==176.)));
    assert!(fab_offset(&collapsed,80.0)>0.0);
}

#[test]
fn vertical_expandable_floating_toolbar_places_and_grows_fab() {
    let bounds=Rect{x:10.,y:10.,width:80.,height:200.};
    for rtl in [false,true] {for position in [ToolbarFabPosition::Start,ToolbarFabPosition::End] {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=ToolbarState::default();
        let mut draw=|ui:&mut Ui,expanded:bool|{ui.frame(Input::default(),VIEW,0.0,|f|{f.floating_toolbar_with_fab_expanded(2000,bounds,ToolbarKind::FloatingVertical,false,&ACTIONS,&mut state,rtl,ToolbarAction{label:"Create",enabled:true},AppBarIcon::Add,position,expanded);});};
        draw(&mut ui,true);
        let expanded=ui.semantics().iter().find(|n|n.id==2132).unwrap().bounds;
        assert_eq!(expanded.width,56.0);assert_eq!(expanded.y,if position==ToolbarFabPosition::Start {10.0}else{154.0});
        draw(&mut ui,false);
        let collapsed=ui.semantics().iter().find(|n|n.id==2132).unwrap().bounds;
        assert_eq!(collapsed.width,80.0);assert_eq!(collapsed.y,if position==ToolbarFabPosition::Start {10.0}else{130.0});
        assert!(!ui.semantics().iter().any(|n|n.id==2000 && n.enabled));
    }}
}

#[test]
fn vertical_floating_toolbar_expansion_reverses_without_jumping() {
    let bounds=Rect{x:10.,y:10.,width:80.,height:200.};
    for position in [ToolbarFabPosition::Start,ToolbarFabPosition::End] {
        let mut ui=Ui::default();let mut state=ToolbarState::default();
        let mut sample=|ui:&mut Ui,time:f64,expanded:bool| {
            ui.frame(Input{time,..Input::default()},VIEW,0.0,|f| {
                f.floating_toolbar_with_fab_expanded(2000,bounds,ToolbarKind::FloatingVertical,false,&ACTIONS,&mut state,false,ToolbarAction{label:"Create",enabled:true},AppBarIcon::Add,position,expanded);
            });
            ui.semantics().iter().find(|node|node.id==2132).unwrap().bounds
        };
        sample(&mut ui,0.0,true);
        sample(&mut ui,0.01,false);
        let middle=sample(&mut ui,0.03,false);
        assert!(middle.height>56.0 && middle.height<80.0);
        let reversed=sample(&mut ui,0.03,true);
        assert!((middle.height-reversed.height).abs()<0.001);
        assert!((middle.y-reversed.y).abs()<0.001);
        let restored=sample(&mut ui,1.0,true);
        assert_eq!(restored.height,56.0);
        assert_eq!(restored.y,if position==ToolbarFabPosition::Start {10.0}else{154.0});
    }
}
