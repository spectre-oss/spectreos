mod bottom_app_bar {
    use bhoo::*;
    const VIEW: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 400.0,
    };
    const ACTIONS: [AppBarAction<'static>; 3] = [
        AppBarAction {
            label: "Menu",
            glyph: "",
            enabled: true,
        },
        AppBarAction {
            label: "Search",
            glyph: "",
            enabled: false,
        },
        AppBarAction {
            label: "More",
            glyph: "",
            enabled: true,
        },
    ];
    const FAB: AppBarAction<'static> = AppBarAction {
        label: "Add",
        glyph: "+",
        enabled: true,
    };
    const ICONS: [Option<AppBarIcon>; 3] = [
        Some(AppBarIcon::Menu),
        Some(AppBarIcon::Search),
        Some(AppBarIcon::More),
    ];
    fn draw(ui: &mut Ui, bounds: Rect, rtl: bool, fab: bool, input: Input) -> Option<usize> {
        let mut result = None;
        ui.frame(input, VIEW, 0.0, |f| {
            result = f.bottom_app_bar_at(
                100,
                bounds,
                &ACTIONS,
                &ICONS,
                fab.then_some((FAB, Some(AppBarIcon::Add))),
                rtl,
            );
        });
        result
    }
    #[test]
    fn bottom_bar_places_fab_at_logical_end_and_disabled_actions() {
        for rtl in [false, true] {
            let mut ui = Ui::default();
            let b = Rect {
                x: 0.0,
                y: 320.0,
                width: 400.0,
                height: 80.0,
            };
            draw(&mut ui, b, rtl, true, Input::default());
            let nodes = ui.semantics();
            let menu = nodes.iter().find(|n| n.id == 100).unwrap();
            let search = nodes.iter().find(|n| n.id == 101).unwrap();
            let fab = nodes.iter().find(|n| n.id == 103).unwrap();
            assert_eq!(
                (fab.bounds.width, fab.bounds.height, fab.bounds.y),
                (56.0, 56.0, 332.0)
            );
            assert_eq!(fab.bounds.x, if rtl { 16.0 } else { 328.0 });
            assert_eq!(menu.bounds.x, if rtl { 348.0 } else { 4.0 });
            assert!(!search.enabled && fab.enabled);
            for (pressed, released) in [(true, false), (false, true)] {
                let result = draw(
                    &mut ui,
                    b,
                    rtl,
                    true,
                    Input {
                        pointer: Some([if rtl { 44.0 } else { 356.0 }, 360.0]),
                        pressed,
                        released,
                        ..Input::default()
                    },
                );
                if released {
                    assert_eq!(result, Some(3));
                }
            }
        }
    }
    #[test]
    fn narrow_bar_omits_overflow_without_overlapping_fab() {
        let mut ui = Ui::default();
        let b = Rect {
            x: 0.0,
            y: 0.0,
            width: 176.0,
            height: 80.0,
        };
        draw(&mut ui, b, false, true, Input::default());
        let nodes = ui.semantics();
        assert_eq!(
            nodes.iter().map(|n| n.id).collect::<Vec<_>>(),
            vec![100, 101, 103]
        );
        let action = nodes.iter().find(|n| n.id == 101).unwrap().bounds;
        let fab = nodes.iter().find(|n| n.id == 103).unwrap().bounds;
        assert!(action.x + action.width <= fab.x);
        draw(&mut ui, b, false, false, Input::default());
        assert_eq!(ui.semantics().len(), 3);
        assert!(!ui.semantics().iter().any(|n| n.id == 103));
    }

    #[test]
    fn scroll_hide_clips_and_removes_fully_hidden_actions() {
        let mut ui = Ui::default();
        let mut state = BottomAppBarState::default();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 400.0,
        };
        fn draw_scrolled(ui: &mut Ui, state: &BottomAppBarState, viewport: Rect) -> Vec<DrawCommand> {
            ui.frame(Input::default(), VIEW, 0.0, |f| {
                f.scrolling_bottom_app_bar(
                    100,
                    viewport,
                    state,
                    &ACTIONS,
                    &ICONS,
                    Some((FAB, Some(AppBarIcon::Add))),
                    false,
                );
            })
        }
        state.apply_scroll(BottomAppBarScrollBehavior::ExitAlways, 40.0, 80.0);
        assert_eq!(state.hidden_offset(), 40.0);
        let commands = draw_scrolled(&mut ui, &state, viewport);
        let fab = ui.semantics().iter().find(|n| n.id == 103).unwrap();
        assert!(fab.bounds.y + fab.bounds.height <= viewport.height);
        assert!(fab.bounds.height < 56.0);
        assert!(
            commands
                .iter()
                .any(|c| matches!(c,DrawCommand::PushClip(b) if *b==viewport))
        );
        state.apply_scroll(BottomAppBarScrollBehavior::ExitAlways, 100.0, 80.0);
        assert_eq!(state.hidden_offset(), 80.0);
        draw_scrolled(&mut ui, &state, viewport);
        assert!(ui.semantics().is_empty());
        state.apply_scroll(BottomAppBarScrollBehavior::ExitAlways, -20.0, 80.0);
        assert_eq!(state.hidden_offset(), 60.0);
        draw_scrolled(&mut ui, &state, viewport);
        assert!(ui.semantics().iter().any(|n| n.id == 103));
        state.apply_scroll(BottomAppBarScrollBehavior::Pinned, 999.0, 80.0);
        assert_eq!(state.hidden_offset(), 0.0);
        state.apply_scroll(BottomAppBarScrollBehavior::ExitAlways, f32::NAN, 80.0);
        assert_eq!(state.hidden_offset(), 0.0);
    }

    #[test]
    fn touch_settles_to_nearest_edge_and_can_be_interrupted() {
        fn frame(
            ui: &mut Ui,
            bar: &mut BottomAppBarState,
            scroll: &mut TopAppBarState,
            offset: &mut f32,
            event: Input,
        ) {
            ui.frame(event, VIEW, 0.0, |f| {
                f.bottom_app_bar_scroll_input(
                    130,
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 300.0,
                        height: 300.0,
                    },
                    bar,
                    BottomAppBarScrollBehavior::ExitAlways,
                    scroll,
                    offset,
                    600.0,
                    80.0,
                );
            });
        }
        fn touch(time: f64, y: f32, pressed: bool, released: bool) -> Input {
            Input {
                time,
                pointer: Some([30.0, y]),
                pointer_kind: PointerKind::Touch,
                pressed,
                released,
                ..Input::default()
            }
        }
        let mut ui = Ui::default();
        let mut bar = BottomAppBarState::default();
        let mut scroll = TopAppBarState::default();
        let mut offset = 0.0;
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.0, 200.0, true, false),
        );
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.2, 175.0, false, false),
        );
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.4, 175.0, false, true),
        );
        assert_eq!(bar.hidden_offset(), 25.0);
        assert!(ui.needs_redraw());
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            Input {
                time: 0.5,
                ..Input::default()
            },
        );
        let middle = bar.hidden_offset();
        assert!(middle > 0.0 && middle < 25.0);
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.5, 180.0, true, false),
        );
        assert!((bar.hidden_offset() - middle).abs() < 0.001);
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.7, 120.0, false, false),
        );
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.9, 120.0, false, true),
        );
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            Input {
                time: 1.4,
                ..Input::default()
            },
        );
        assert_eq!(bar.hidden_offset(), 80.0);
        assert!(!ui.needs_redraw());
        ui.motion_preference = MotionPreference::Reduced;
        bar.apply_scroll(BottomAppBarScrollBehavior::ExitAlways, -60.0, 80.0);
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(2.0, 200.0, true, false),
        );
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(2.2, 190.0, false, false),
        );
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(2.4, 190.0, false, true),
        );
        assert_eq!(bar.hidden_offset(), 0.0);
    }

    #[test]
    fn bottom_bar_tracks_content_fling_and_stops_at_boundaries() {
        fn frame(
            ui: &mut Ui,
            bar: &mut BottomAppBarState,
            scroll: &mut TopAppBarState,
            offset: &mut f32,
            event: Input,
        ) {
            ui.frame(event, VIEW, 0.0, |f| {
                f.bottom_app_bar_scroll_input(
                    130,
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 300.0,
                        height: 300.0,
                    },
                    bar,
                    BottomAppBarScrollBehavior::ExitAlways,
                    scroll,
                    offset,
                    600.0,
                    80.0,
                );
            });
        }
        let touch = |time, y, pressed, released| Input {
            time,
            pointer: Some([30.0, y]),
            pointer_kind: PointerKind::Touch,
            pressed,
            released,
            ..Input::default()
        };
        let mut ui = Ui::default();
        let mut bar = BottomAppBarState::default();
        let mut scroll = TopAppBarState::default();
        let mut offset = 0.0;
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.0, 200.0, true, false),
        );
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.02, 180.0, false, false),
        );
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            touch(0.03, 180.0, false, true),
        );
        let before = bar.hidden_offset();
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            Input {
                time: 0.08,
                ..Input::default()
            },
        );
        assert!(bar.hidden_offset() > before && offset > 20.0);
        frame(
            &mut ui,
            &mut bar,
            &mut scroll,
            &mut offset,
            Input {
                time: 2.0,
                ..Input::default()
            },
        );
        assert_eq!(bar.hidden_offset(), 80.0);
        assert!(!ui.needs_redraw());
    }

    #[test]
    fn overflow_preserves_hidden_actions_and_activates_original_index() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let bar = BottomAppBarState::default();
        let mut menu = BottomAppBarOverflowState::default();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: 176.0,
            height: 300.0,
        };
        fn frame(
            ui: &mut Ui,
            bar: &BottomAppBarState,
            menu: &mut BottomAppBarOverflowState,
            viewport: Rect,
            input: Input,
        ) -> Option<usize> {
            let mut result = None;
            ui.frame(input, VIEW, 0.0, |f| {
                result = f.scrolling_bottom_app_bar_with_overflow(
                    100,
                    viewport,
                    bar,
                    menu,
                    &ACTIONS,
                    &ICONS,
                    Some((FAB, Some(AppBarIcon::Add))),
                    false,
                );
            });
            result
        }
        frame(&mut ui, &bar, &mut menu, viewport, Input::default());
        assert!(
            ui.semantics()
                .iter()
                .any(|n| n.id == 104 && n.label.as_ref() == "More actions")
        );
        assert!(!ui.semantics().iter().any(|n| n.id == 101));
        let trigger = ui.semantics().iter().find(|n| n.id == 104).unwrap().bounds;
        let p = [
            trigger.x + trigger.width / 2.0,
            trigger.y + trigger.height / 2.0,
        ];
        for (pressed, released) in [(true, false), (false, true)] {
            frame(
                &mut ui,
                &bar,
                &mut menu,
                viewport,
                Input {
                    pointer: Some(p),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        assert!(menu.overlay.open);
        assert!(
            ui.semantics()
                .iter()
                .any(|n| n.id == 105 && !n.enabled && n.label.as_ref() == "Search")
        );
        let more = ui
            .semantics()
            .iter()
            .find(|n| n.id == 106 && n.enabled)
            .unwrap()
            .bounds;
        let p = [more.x + more.width / 2.0, more.y + more.height / 2.0];
        for (pressed, released) in [(true, false), (false, true)] {
            let result = frame(
                &mut ui,
                &bar,
                &mut menu,
                viewport,
                Input {
                    pointer: Some(p),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
            if released {
                assert_eq!(result, Some(2));
            }
        }
        assert!(!menu.overlay.open);
    }
    #[test]
    fn very_narrow_bar_moves_fab_into_overflow() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let bar = BottomAppBarState::default();
        let mut menu = BottomAppBarOverflowState::default();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: 104.0,
            height: 300.0,
        };
        let mut result = None;
        let mut frame = |ui: &mut Ui, menu: &mut BottomAppBarOverflowState, input: Input| {
            ui.frame(input, VIEW, 0.0, |f| {
                result = f.scrolling_bottom_app_bar_with_overflow(
                    100,
                    viewport,
                    &bar,
                    menu,
                    &ACTIONS,
                    &ICONS,
                    Some((FAB, Some(AppBarIcon::Add))),
                    false,
                );
            });
        };
        frame(&mut ui, &mut menu, Input::default());
        assert!(!ui.semantics().iter().any(|n| n.id == 103));
        let b = ui.semantics().iter().find(|n| n.id == 104).unwrap().bounds;
        let p = [b.x + b.width / 2.0, b.y + b.height / 2.0];
        for (pressed, released) in [(true, false), (false, true)] {
            frame(
                &mut ui,
                &mut menu,
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
                .any(|n| n.id == 107 && n.label.as_ref() == "Add")
        );
        drop(frame);
        assert_eq!(result, None);
    }

    #[test]
    fn overflow_keyboard_reaches_offscreen_items_and_escape_restores_trigger() {
        let actions: Vec<_> = (0..10)
            .map(|i| AppBarAction {
                label: if i == 9 { "Zebra" } else { "Action" },
                glyph: "",
                enabled: true,
            })
            .collect();
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let bar = BottomAppBarState::default();
        let mut menu = BottomAppBarOverflowState::default();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: 176.0,
            height: 300.0,
        };
        fn frame(
            ui: &mut Ui,
            bar: &BottomAppBarState,
            menu: &mut BottomAppBarOverflowState,
            viewport: Rect,
            actions: &[AppBarAction<'_>],
            input: Input,
        ) {
            ui.frame(input, VIEW, 0.0, |f| {
                f.scrolling_bottom_app_bar_with_overflow(
                    100,
                    viewport,
                    bar,
                    menu,
                    actions,
                    &[],
                    Some((FAB, Some(AppBarIcon::Add))),
                    false,
                );
            });
        }
        frame(
            &mut ui,
            &bar,
            &mut menu,
            viewport,
            &actions,
            Input::default(),
        );
        let b = ui.semantics().iter().find(|n| n.id == 111).unwrap().bounds;
        let p = [b.x + b.width / 2.0, b.y + b.height / 2.0];
        for (pressed, released) in [(true, false), (false, true)] {
            frame(
                &mut ui,
                &bar,
                &mut menu,
                viewport,
                &actions,
                Input {
                    pointer: Some(p),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        assert!(menu.overlay.open);
        assert_eq!(ui.focused(), Some(112));
        frame(
            &mut ui,
            &bar,
            &mut menu,
            viewport,
            &actions,
            Input {
                time: 0.05,
                key: Some(KeyInput {
                    key: Key::Character('z'),
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..Input::default()
            },
        );
        assert_eq!(ui.focused(), Some(120));
        assert!(ui.semantics().iter().any(|n| n.id == 120 && n.enabled));
        frame(
            &mut ui,
            &bar,
            &mut menu,
            viewport,
            &actions,
            Input {
                time: 0.1,
                key: Some(KeyInput {
                    key: Key::End,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..Input::default()
            },
        );
        assert_eq!(ui.focused(), Some(120));
        assert!(ui.semantics().iter().any(|n| n.id == 120 && n.enabled));
        frame(
            &mut ui,
            &bar,
            &mut menu,
            viewport,
            &actions,
            Input {
                time: 0.2,
                key: Some(KeyInput {
                    key: Key::Escape,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..Input::default()
            },
        );
        assert!(!menu.overlay.open);
        frame(
            &mut ui,
            &bar,
            &mut menu,
            viewport,
            &actions,
            Input {
                time: 0.3,
                ..Input::default()
            },
        );
        assert_eq!(ui.focused(), Some(111));
    }

    #[test]
    fn overflow_touch_drag_scrolls_without_activating_an_item() {
        let actions = vec![
            AppBarAction {
                label: "Action",
                glyph: "",
                enabled: true
            };
            10
        ];
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let bar = BottomAppBarState::default();
        let mut menu = BottomAppBarOverflowState::default();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: 176.0,
            height: 300.0,
        };
        fn frame(
            ui: &mut Ui,
            bar: &BottomAppBarState,
            menu: &mut BottomAppBarOverflowState,
            actions: &[AppBarAction<'_>],
            viewport: Rect,
            input: Input,
        ) -> Option<usize> {
            let mut result = None;
            ui.frame(input, VIEW, 0.0, |f| {
                result = f.scrolling_bottom_app_bar_with_overflow(
                    100,
                    viewport,
                    bar,
                    menu,
                    actions,
                    &[],
                    Some((FAB, Some(AppBarIcon::Add))),
                    false,
                );
            });
            result
        }
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            Input::default(),
        );
        let trigger = ui.semantics().iter().find(|n| n.id == 111).unwrap().bounds;
        let p = [
            trigger.x + trigger.width / 2.0,
            trigger.y + trigger.height / 2.0,
        ];
        for (pressed, released) in [(true, false), (false, true)] {
            frame(
                &mut ui,
                &bar,
                &mut menu,
                &actions,
                viewport,
                Input {
                    pointer: Some(p),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        let first = ui.semantics().iter().find(|n| n.id == 112).unwrap().bounds;
        let x = first.x + first.width / 2.0;
        let y = first.y + 24.0;
        for (y, pressed, released) in [
            (y, true, false),
            (y - 3.0, false, false),
            (y - 100.0, false, false),
            (y - 100.0, false, true),
        ] {
            assert_eq!(
                frame(
                    &mut ui,
                    &bar,
                    &mut menu,
                    &actions,
                    viewport,
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
        assert!(menu.overlay.open);
        let after = ui.semantics().iter().find(|n| n.id == 115).unwrap().bounds;
        assert!((first.y + 3.0 * 48.0 - after.y - 100.0).abs() < 0.01);
    }

    #[test]
    fn overflow_touch_fling_continues_and_interrupts() {
        let actions = vec![
            AppBarAction {
                label: "Action",
                glyph: "",
                enabled: true
            };
            10
        ];
        let mut ui = Ui::default();
        let bar = BottomAppBarState::default();
        let mut menu = BottomAppBarOverflowState::default();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            width: 176.0,
            height: 300.0,
        };
        fn frame(
            ui: &mut Ui,
            bar: &BottomAppBarState,
            menu: &mut BottomAppBarOverflowState,
            actions: &[AppBarAction<'_>],
            viewport: Rect,
            input: Input,
        ) {
            ui.frame(input, VIEW, 0.0, |f| {
                f.scrolling_bottom_app_bar_with_overflow(
                    100,
                    viewport,
                    bar,
                    menu,
                    actions,
                    &[],
                    Some((FAB, Some(AppBarIcon::Add))),
                    false,
                );
            });
        }
        ui.motion_preference = MotionPreference::Reduced;
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            Input::default(),
        );
        let b = ui.semantics().iter().find(|n| n.id == 111).unwrap().bounds;
        let trigger = [b.x + b.width / 2.0, b.y + b.height / 2.0];
        for (pressed, released) in [(true, false), (false, true)] {
            frame(
                &mut ui,
                &bar,
                &mut menu,
                &actions,
                viewport,
                Input {
                    pointer: Some(trigger),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        ui.motion_preference = MotionPreference::Full;
        let touch = |time, y, pressed, released| Input {
            time,
            pointer: Some([100.0, y]),
            pointer_kind: PointerKind::Touch,
            pressed,
            released,
            ..Input::default()
        };
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            touch(1.0, 150.0, true, false),
        );
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            touch(1.05, 100.0, false, false),
        );
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            touch(1.06, 100.0, false, true),
        );
        let before = ui
            .semantics()
            .iter()
            .find(|n| n.id == 116)
            .unwrap()
            .bounds
            .y;
        assert!(ui.needs_redraw());
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            Input {
                time: 1.12,
                ..Input::default()
            },
        );
        let after = ui
            .semantics()
            .iter()
            .find(|n| n.id == 116)
            .unwrap()
            .bounds
            .y;
        assert!(after < before);
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            touch(1.13, 100.0, true, false),
        );
        let stopped = ui
            .semantics()
            .iter()
            .find(|n| n.id == 116)
            .unwrap()
            .bounds
            .y;
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            touch(1.3, 100.0, false, false),
        );
        assert_eq!(
            ui.semantics()
                .iter()
                .find(|n| n.id == 116)
                .unwrap()
                .bounds
                .y,
            stopped
        );
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            Input {
                time: 1.31,
                cancelled: true,
                ..Input::default()
            },
        );
        ui.motion_preference = MotionPreference::Reduced;
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            touch(2.0, 150.0, true, false),
        );
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            touch(2.05, 130.0, false, false),
        );
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            touch(2.06, 130.0, false, true),
        );
        let reduced = ui
            .semantics()
            .iter()
            .find(|n| n.id == 117)
            .unwrap()
            .bounds
            .y;
        frame(
            &mut ui,
            &bar,
            &mut menu,
            &actions,
            viewport,
            Input {
                time: 2.12,
                ..Input::default()
            },
        );
        assert_eq!(
            ui.semantics()
                .iter()
                .find(|n| n.id == 117)
                .unwrap()
                .bounds
                .y,
            reduced
        );
    }
}

mod top_app_bar {
    use bhoo::*;
    const VIEW: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 240.0,
    };
    const NAV: AppBarAction = AppBarAction {
        label: "Back",
        glyph: "‹",
        enabled: true,
    };
    const ACTIONS: [AppBarAction; 2] = [
        AppBarAction {
            label: "More",
            glyph: "⋮",
            enabled: true,
        },
        AppBarAction {
            label: "Search",
            glyph: "?",
            enabled: false,
        },
    ];
    #[test]
    fn scroll_behavior_consumes_only_available_range() {
        let mut s = TopAppBarState::default();
        let v = TopAppBarVariant::Large;
        assert_eq!(
            s.apply_scroll(v, TopAppBarScrollBehavior::Pinned, 30.0, true),
            0.0
        );
        assert_eq!(
            s.apply_scroll(v, TopAppBarScrollBehavior::ExitUntilCollapsed, 100.0, true),
            88.0
        );
        assert_eq!(s.collapse_fraction(v), 1.0);
        assert_eq!(
            s.apply_scroll(v, TopAppBarScrollBehavior::ExitUntilCollapsed, -20.0, false),
            0.0
        );
        assert_eq!(
            s.apply_scroll(v, TopAppBarScrollBehavior::ExitUntilCollapsed, -20.0, true),
            -20.0
        );
        assert_eq!(
            s.apply_scroll(v, TopAppBarScrollBehavior::EnterAlways, -100.0, false),
            -68.0
        );
        assert_eq!(s.collapse_fraction(v), 0.0);
        assert_eq!(
            s.apply_scroll(v, TopAppBarScrollBehavior::EnterAlways, f32::NAN, true),
            0.0
        );
        assert_eq!(
            s.apply_scroll(
                TopAppBarVariant::Small,
                TopAppBarScrollBehavior::EnterAlways,
                100.0,
                true
            ),
            0.0
        );
    }
    #[test]
    fn variants_collapse_to_small_height_and_mirror_actions() {
        for (variant, expected) in [
            (TopAppBarVariant::Small, 64.0),
            (TopAppBarVariant::CenterAligned, 64.0),
            (TopAppBarVariant::Medium, 112.0),
            (TopAppBarVariant::Large, 152.0),
        ] {
            let mut ui = Ui::default();
            let mut s = TopAppBarState::default();
            let draw = |ui: &mut Ui, s: &TopAppBarState, rtl| {
                ui.frame(Input::default(), VIEW, 0.0, |f| {
                    f.top_app_bar(100, "Page", variant, s, Some(NAV), &ACTIONS, rtl);
                })
            };
            let commands = draw(&mut ui, &s, false);
            assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect{bounds,color,..} if bounds.height==expected && *color==ui.theme.background)));
            let nodes = ui.semantics();
            assert_eq!(nodes.iter().find(|n| n.id == 100).unwrap().bounds.x, 0.0);
            assert!(!nodes.iter().find(|n| n.id == 102).unwrap().enabled);
            s.set_collapse_fraction(variant, 1.0);
            let commands = draw(&mut ui, &s, true);
            assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect{bounds,..} if bounds.width==400.0 && bounds.height==64.0)));
            assert_eq!(
                ui.semantics()
                    .iter()
                    .find(|n| n.id == 100)
                    .unwrap()
                    .bounds
                    .x,
                352.0
            );
        }
    }
    #[test]
    fn enabled_action_activates_but_disabled_does_not() {
        let mut ui = Ui::default();
        let s = TopAppBarState::default();
        let mut chosen = None;
        for (pressed, released) in [(true, false), (false, true)] {
            ui.frame(
                Input {
                    pointer: Some([375.0, 32.0]),
                    pressed,
                    released,
                    ..Input::default()
                },
                VIEW,
                0.0,
                |f| {
                    chosen = f.top_app_bar(
                        100,
                        "Page",
                        TopAppBarVariant::Small,
                        &s,
                        Some(NAV),
                        &ACTIONS,
                        false,
                    );
                },
            );
        }
        assert_eq!(chosen, Some(1));
        for (pressed, released) in [(true, false), (false, true)] {
            ui.frame(
                Input {
                    pointer: Some([325.0, 32.0]),
                    pressed,
                    released,
                    ..Input::default()
                },
                VIEW,
                0.0,
                |f| {
                    chosen = f.top_app_bar(
                        100,
                        "Page",
                        TopAppBarVariant::Small,
                        &s,
                        Some(NAV),
                        &ACTIONS,
                        false,
                    );
                },
            );
        }
        assert_eq!(chosen, None);
    }

    #[test]
    fn nested_scroll_routes_delta_in_the_correct_order() {
        let v = TopAppBarVariant::Large;
        let mut state = TopAppBarState::default();
        let mut content = 0.0;
        let exit = TopAppBarScrollBehavior::ExitUntilCollapsed;
        assert_eq!(
            state.route_scroll(v, exit, 120.0, &mut content, 300.0),
            120.0
        );
        assert_eq!(state.collapse_fraction(v), 1.0);
        assert_eq!(content, 32.0);
        assert_eq!(
            state.route_scroll(v, exit, -12.0, &mut content, 300.0),
            -12.0
        );
        assert_eq!(content, 20.0);
        assert_eq!(state.collapse_fraction(v), 1.0);
        assert_eq!(
            state.route_scroll(v, exit, -40.0, &mut content, 300.0),
            -40.0
        );
        assert_eq!(content, 0.0);
        assert!((state.collapse_fraction(v) - (68.0 / 88.0)).abs() < 0.0001);
        content = 70.0;
        let before = state.collapse_fraction(v);
        state.route_scroll(
            v,
            TopAppBarScrollBehavior::EnterAlways,
            -20.0,
            &mut content,
            300.0,
        );
        assert!(state.collapse_fraction(v) < before);
        assert_eq!(content, 70.0);
        let before = state.collapse_fraction(v);
        state.route_scroll(
            v,
            TopAppBarScrollBehavior::Pinned,
            25.0,
            &mut content,
            300.0,
        );
        assert_eq!(state.collapse_fraction(v), before);
        assert_eq!(content, 95.0);
        assert_eq!(
            state.route_scroll(
                v,
                TopAppBarScrollBehavior::Pinned,
                1000.0,
                &mut content,
                300.0
            ),
            205.0
        );
        assert_eq!(content, 300.0);
        assert_eq!(
            state.route_scroll(v, exit, f32::NAN, &mut content, 300.0),
            0.0
        );
    }

    #[test]
    fn touch_drag_routes_delta_captures_outside_and_cancels() {
        let mut ui = Ui::default();
        let mut state = TopAppBarState::default();
        let mut content = 0.0;
        let body = Rect {
            x: 0.0,
            y: 100.0,
            width: 200.0,
            height: 100.0,
        };
        fn input(
            ui: &mut Ui,
            state: &mut TopAppBarState,
            content: &mut f32,
            body: Rect,
            event: Input,
        ) -> f32 {
            let mut consumed = 0.0;
            ui.frame(event, VIEW, 0.0, |f| {
                consumed = f.top_app_bar_scroll_input(
                    body,
                    state,
                    TopAppBarVariant::Large,
                    TopAppBarScrollBehavior::ExitUntilCollapsed,
                    content,
                    300.0,
                );
            });
            consumed
        }
        let touch = |y, pressed, released, cancelled| Input {
            pointer: Some([50.0, y]),
            pointer_kind: PointerKind::Touch,
            pressed,
            released,
            cancelled,
            ..Input::default()
        };
        assert_eq!(
            input(
                &mut ui,
                &mut state,
                &mut content,
                body,
                touch(150.0, true, false, false)
            ),
            0.0
        );
        assert_eq!(
            input(
                &mut ui,
                &mut state,
                &mut content,
                body,
                touch(147.0, false, false, false)
            ),
            0.0
        );
        assert_eq!(
            input(
                &mut ui,
                &mut state,
                &mut content,
                body,
                touch(100.0, false, false, false)
            ),
            50.0
        );
        assert_eq!(
            input(
                &mut ui,
                &mut state,
                &mut content,
                body,
                touch(70.0, false, false, false)
            ),
            30.0
        );
        assert!((state.collapse_fraction(TopAppBarVariant::Large) - 80.0 / 88.0).abs() < 0.0001);
        input(
            &mut ui,
            &mut state,
            &mut content,
            body,
            touch(70.0, false, false, true),
        );
        assert_eq!(
            input(
                &mut ui,
                &mut state,
                &mut content,
                body,
                touch(30.0, false, false, false)
            ),
            0.0
        );
        assert_eq!(content, 0.0);
        assert_eq!(
            input(
                &mut ui,
                &mut state,
                &mut content,
                body,
                Input {
                    pointer: Some([50.0, 150.0]),
                    scroll: [0.0, 30.0],
                    ..Input::default()
                }
            ),
            30.0
        );
        assert_eq!(content, 22.0);
    }

    #[test]
    fn released_touch_snaps_to_nearest_endpoint_and_can_be_interrupted() {
        fn frame(ui: &mut Ui, s: &mut TopAppBarState, offset: &mut f32, event: Input) {
            ui.frame(event, VIEW, 0.0, |f| {
                f.top_app_bar_scroll_input(
                    Rect {
                        x: 0.0,
                        y: 100.0,
                        width: 300.0,
                        height: 100.0,
                    },
                    s,
                    TopAppBarVariant::Large,
                    TopAppBarScrollBehavior::ExitUntilCollapsed,
                    offset,
                    300.0,
                );
            });
        }
        let touch = |time, y, pressed, released| Input {
            time,
            pointer: Some([50.0, y]),
            pointer_kind: PointerKind::Touch,
            pressed,
            released,
            ..Input::default()
        };
        let mut ui = Ui::default();
        let mut state = TopAppBarState::default();
        let mut offset = 0.0;
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(0.0, 180.0, true, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(0.1, 150.0, false, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(0.11, 150.0, false, true),
        );
        let start = state.collapse_fraction(TopAppBarVariant::Large);
        assert!((start - 30.0 / 88.0).abs() < 0.001 && ui.needs_redraw());
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            Input {
                time: 0.2,
                ..Input::default()
            },
        );
        let middle = state.collapse_fraction(TopAppBarVariant::Large);
        assert!(middle > 0.0 && middle < start);
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(0.2, 150.0, true, false),
        );
        assert!((state.collapse_fraction(TopAppBarVariant::Large) - middle).abs() < 0.001);
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(0.21, 90.0, false, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(0.22, 90.0, false, true),
        );
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            Input {
                time: 1.0,
                ..Input::default()
            },
        );
        assert_eq!(state.collapse_fraction(TopAppBarVariant::Large), 1.0);
        assert!(!ui.needs_redraw());
        ui.motion_preference = MotionPreference::Reduced;
        state.set_collapse_fraction(TopAppBarVariant::Large, 0.3);
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(1.1, 180.0, true, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(1.2, 170.0, false, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            touch(1.21, 170.0, false, true),
        );
        assert_eq!(state.collapse_fraction(TopAppBarVariant::Large), 0.0);
    }

    #[test]
    fn touch_fling_continues_into_content_stops_and_is_interruptible() {
        fn frame(ui: &mut Ui, s: &mut TopAppBarState, content: &mut f32, event: Input) {
            ui.frame(event, VIEW, 0.0, |f| {
                f.top_app_bar_scroll_input(
                    Rect {
                        x: 0.0,
                        y: 100.0,
                        width: 300.0,
                        height: 100.0,
                    },
                    s,
                    TopAppBarVariant::Large,
                    TopAppBarScrollBehavior::ExitUntilCollapsed,
                    content,
                    300.0,
                );
            });
        }
        let touch = |time, y, pressed, released| Input {
            time,
            pointer: Some([50.0, y]),
            pointer_kind: PointerKind::Touch,
            pressed,
            released,
            ..Input::default()
        };
        let mut ui = Ui::default();
        let mut state = TopAppBarState::default();
        let mut content = 0.0;
        frame(
            &mut ui,
            &mut state,
            &mut content,
            touch(0.0, 180.0, true, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut content,
            touch(0.02, 160.0, false, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut content,
            touch(0.03, 160.0, false, true),
        );
        let before = state.collapse_fraction(TopAppBarVariant::Large);
        frame(
            &mut ui,
            &mut state,
            &mut content,
            Input {
                time: 0.1,
                ..Input::default()
            },
        );
        assert!(state.collapse_fraction(TopAppBarVariant::Large) > before && ui.needs_redraw());
        frame(
            &mut ui,
            &mut state,
            &mut content,
            Input {
                time: 0.3,
                ..Input::default()
            },
        );
        assert_eq!(state.collapse_fraction(TopAppBarVariant::Large), 1.0);
        assert!(content > 0.0);
        frame(
            &mut ui,
            &mut state,
            &mut content,
            Input {
                time: 2.0,
                ..Input::default()
            },
        );
        assert!(!ui.needs_redraw());
        let stopped = content;
        frame(
            &mut ui,
            &mut state,
            &mut content,
            Input {
                time: 3.0,
                ..Input::default()
            },
        );
        assert_eq!(content, stopped);
        state.set_collapse_fraction(TopAppBarVariant::Large, 0.0);
        content = 0.0;
        frame(
            &mut ui,
            &mut state,
            &mut content,
            touch(4.0, 180.0, true, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut content,
            touch(4.02, 160.0, false, false),
        );
        frame(
            &mut ui,
            &mut state,
            &mut content,
            touch(4.03, 160.0, false, true),
        );
        frame(
            &mut ui,
            &mut state,
            &mut content,
            touch(4.08, 150.0, true, false),
        );
        let held = state.collapse_fraction(TopAppBarVariant::Large);
        frame(
            &mut ui,
            &mut state,
            &mut content,
            Input {
                time: 4.5,
                ..Input::default()
            },
        );
        assert_eq!(state.collapse_fraction(TopAppBarVariant::Large), held);
        assert_eq!(content, 0.0);
    }

    #[test]
    fn focused_content_keyboard_routes_scroll_and_respects_boundaries() {
        fn frame(
            ui: &mut Ui,
            state: &mut TopAppBarState,
            offset: &mut f32,
            event: Input,
            focus: bool,
            enabled: bool,
        ) -> f32 {
            let mut used = 0.0;
            ui.frame(event, VIEW, 0.0, |f| {
                f.with_enabled(enabled, |f| {
                    if focus {
                        f.request_focus(130);
                    }
                    let bounds = Rect {
                        x: 0.0,
                        y: 100.0,
                        width: 300.0,
                        height: 100.0,
                    };
                    used = f.top_app_bar_scroll_keyboard(
                        130,
                        bounds,
                        state,
                        TopAppBarVariant::Large,
                        TopAppBarScrollBehavior::ExitUntilCollapsed,
                        offset,
                        300.0,
                    );
                    f.target(130, bounds, "Scrollable content", true, Role::Group);
                })
            });
            used
        }
        fn key(key: Key, time: f64, repeat: bool) -> Input {
            Input {
                time,
                key: Some(KeyInput {
                    key,
                    pressed: true,
                    repeat,
                    shift: false,
                }),
                ..Input::default()
            }
        }
        let mut ui = Ui::default();
        let mut state = TopAppBarState::default();
        let mut offset = 0.0;
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            Input::default(),
            true,
            true,
        );
        assert_eq!(
            frame(
                &mut ui,
                &mut state,
                &mut offset,
                key(Key::Down, 0.1, false),
                false,
                true
            ),
            48.0
        );
        assert_eq!(
            state.collapse_fraction(TopAppBarVariant::Large),
            48.0 / 88.0
        );
        assert_eq!(
            frame(
                &mut ui,
                &mut state,
                &mut offset,
                key(Key::Down, 0.2, true),
                false,
                true
            ),
            48.0
        );
        assert_eq!(state.collapse_fraction(TopAppBarVariant::Large), 1.0);
        assert_eq!(offset, 8.0);
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            key(Key::End, 0.3, false),
            false,
            true,
        );
        assert_eq!(offset, 300.0);
        assert_eq!(
            frame(
                &mut ui,
                &mut state,
                &mut offset,
                key(Key::Down, 0.4, false),
                false,
                true
            ),
            0.0
        );
        frame(
            &mut ui,
            &mut state,
            &mut offset,
            key(Key::Home, 0.5, false),
            false,
            true,
        );
        assert_eq!(offset, 0.0);
        assert_eq!(state.collapse_fraction(TopAppBarVariant::Large), 0.0);
        assert_eq!(
            frame(
                &mut ui,
                &mut state,
                &mut offset,
                key(Key::PageDown, 0.51, false),
                false,
                true
            ),
            100.0
        );
        assert_eq!(offset, 12.0);
        assert_eq!(
            frame(
                &mut ui,
                &mut state,
                &mut offset,
                key(Key::PageUp, 0.52, false),
                false,
                true
            ),
            -100.0
        );
        assert_eq!(offset, 0.0);
        assert_eq!(state.collapse_fraction(TopAppBarVariant::Large), 0.0);
        assert_eq!(
            frame(
                &mut ui,
                &mut state,
                &mut offset,
                key(Key::Down, 0.6, false),
                false,
                false
            ),
            0.0
        );
        assert_eq!(offset, 0.0);
    }

    #[test]
    fn material_icons_are_centered_and_back_mirrors_in_rtl() {
        let mut ui = Ui::default();
        let state = TopAppBarState::default();
        for rtl in [false, true] {
            let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
                f.top_app_bar_with_icons(
                    100,
                    "Page",
                    TopAppBarVariant::CenterAligned,
                    &state,
                    Some(NAV),
                    &ACTIONS,
                    rtl,
                    &[
                        Some(AppBarIcon::Back),
                        Some(AppBarIcon::More),
                        Some(AppBarIcon::Search),
                    ],
                );
            });
            let shapes: Vec<_> = commands
                .iter()
                .filter_map(|c| match c {
                    DrawCommand::Shape {
                        bounds,
                        index,
                        color,
                        ..
                    } => Some((*bounds, *index, *color)),
                    _ => None,
                })
                .collect();
            let base = bhoo::beercss_shapes::SHAPES.len();
            assert_eq!(shapes.len(), 3);
            assert_eq!(shapes[0].1 as usize, base + if rtl { 17 } else { 16 });
            assert_eq!(shapes[1].1 as usize, base + 19);
            assert_eq!(shapes[2].1 as usize, base + 18);
            assert!(
                shapes
                    .iter()
                    .all(|(b, _, _)| b.width == 24.0 && b.height == 24.0)
            );
            assert_eq!(shapes[2].2, ui.theme.on_surface_variant.opacity(0.38));
            let title = commands
                .iter()
                .find_map(|c| match c {
                    DrawCommand::Text { bounds, text, .. } if text.as_ref() == "Page" => Some(*bounds),
                    _ => None,
                })
                .unwrap();
            assert_eq!(title.x + title.width / 2.0, VIEW.width / 2.0);
            assert_eq!(
                ui.semantics()
                    .iter()
                    .find(|n| n.id == 100)
                    .unwrap()
                    .label
                    .as_ref(),
                "Back"
            );
        }
    }
}

mod extended_app_bar_tests {
    use crate::*;

    #[test]
    fn top_app_bar_at_collapsing_and_floating_pill_work() {
        let mut ui = Ui::default();
        let bounds = Rect { x: 0.0, y: 0.0, width: 400.0, height: 240.0 };
        let nav = TopAppBarMaterialAction { icon: MaterialIcon::Menu, label: "Menu", enabled: true };
        let acts = [TopAppBarMaterialAction { icon: MaterialIcon::Search, label: "Search", enabled: true }];
        let mut clicked = None;
        let _ = ui.frame(Input { pointer: Some([24.0, 32.0]), pressed: true, time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
            f.top_app_bar_at(10, Rect { x: 0.0, y: 0.0, width: 400.0, height: 64.0 }, "Inbox", false, Some(nav), &acts);
        });
        let _ = ui.frame(Input { pointer: Some([24.0, 32.0]), released: true, time: 0.05, ..Input::default() }, bounds, 0.0, |f| {
            clicked = f.top_app_bar_at(10, Rect { x: 0.0, y: 0.0, width: 400.0, height: 64.0 }, "Inbox", false, Some(nav), &acts);
        });
        assert_eq!(clicked, Some(0));

        let (mut h0, mut h1) = (0.0, 0.0);
        let _ = ui.frame(Input::default(), bounds, 0.0, |f| {
            h0 = f.top_app_bar_collapsing(20, bounds, "Settings", 136.0, 56.0, 0.0, Some(MaterialIcon::ArrowBack)).0;
            h1 = f.top_app_bar_collapsing(21, bounds, "Settings", 136.0, 56.0, 80.0, Some(MaterialIcon::ArrowBack)).0;
        });
        assert!((h0 - 136.0).abs() < 0.01 && (h1 - 56.0).abs() < 0.01);

        let slots = [
            FloatingPillSlot { label: "Explore", icon: Some(MaterialIcon::Menu) },
            FloatingPillSlot { label: "Brand", icon: None },
        ];
        let mut custom_called = false;
        let _ = ui.frame(Input::default(), bounds, 0.0, |f| {
            f.floating_pill_app_bar(30, Rect { x: 20.0, y: 12.0, width: 120.0, height: 56.0 }, &slots, |_, idx, _| {
                if idx == 1 { custom_called = true; }
            });
        });
        assert!(custom_called);
    }
}
