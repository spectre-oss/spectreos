mod menu {
    use bhoo::*;
    const VIEW: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 300.0,
    };
    const PANEL: Rect = Rect {
        x: 20.0,
        y: 60.0,
        width: 112.0,
        height: 160.0,
    };
    fn draw(
        ui: &mut Ui,
        state: &mut OverlayState,
        input: Input,
        actions: &mut usize,
    ) -> Vec<DrawCommand> {
        ui.frame(input, VIEW, 8.0, |f| {
            f.without_input(state.open, |f| {
                if f.button(100, "Open").clicked {
                    state.open = true;
                }
            });
            f.overlay(500, state, PANEL, false, |f, open| {
                f.with_gap(0.0, |f| {
                    if f.menu_item(600, "Open file", true).clicked {
                        *actions += 1;
                        *open = false;
                    }
                    f.menu_item(601, "Unavailable", false);
                    f.menu_item(602, "Rename", true);
                });
            });
        })
    }
    fn state() -> OverlayState {
        let mut state = OverlayState::default();
        state.style = OverlayStyle::AngularMaterial;
        state.kind = OverlayKind::Menu;
        state
    }
    fn input(time: f64) -> Input {
        Input {
            time,
            ..Input::default()
        }
    }
    #[test]
    fn entering_menu_blocks_pointer_without_dimming_or_disabling_keyboard_targets() {
        let mut ui = Ui::default();
        let mut state = state();
        let mut actions = 0;
        state.open = true;
        draw(&mut ui, &mut state, input(0.0), &mut actions);
        let commands = draw(
            &mut ui,
            &mut state,
            Input {
                pointer: Some([50.0, 90.0]),
                pressed: true,
                ..input(0.04)
            },
            &mut actions,
        );
        draw(
            &mut ui,
            &mut state,
            Input {
                pointer: Some([50.0, 90.0]),
                released: true,
                ..input(0.05)
            },
            &mut actions,
        );
        assert_eq!(actions, 0);
        assert!(
            ui.semantics()
                .iter()
                .any(|n| n.id == 600 && n.enabled && n.role == Role::MenuItem)
        );
        assert!(ui.semantics().iter().any(|n| n.id == 601 && !n.enabled));
        assert!(commands.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if &**text == "Open file" && style.weight == 500)));
        draw(&mut ui, &mut state, input(0.2), &mut actions);
        assert!(
            !ui.needs_redraw(),
            "The nonmodal menu must not keep running a 400ms scrim animation"
        );
        draw(
            &mut ui,
            &mut state,
            Input {
                pointer: Some([50.0, 90.0]),
                pressed: true,
                ..input(0.21)
            },
            &mut actions,
        );
        draw(
            &mut ui,
            &mut state,
            Input {
                pointer: Some([50.0, 90.0]),
                released: true,
                ..input(0.22)
            },
            &mut actions,
        );
        assert_eq!(actions, 1);
        assert!(!state.open);
    }
    #[test]
    fn menu_preserves_keyboard_opening_origin_and_restores_trigger_on_escape() {
        let mut ui = Ui::default();
        let mut state = state();
        let mut actions = 0;
        draw(&mut ui, &mut state, input(0.0), &mut actions);
        draw(
            &mut ui,
            &mut state,
            Input {
                key: Some(KeyInput {
                    key: Key::Tab,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..input(0.01)
            },
            &mut actions,
        );
        draw(
            &mut ui,
            &mut state,
            Input {
                key: Some(KeyInput {
                    key: Key::Enter,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..input(0.02)
            },
            &mut actions,
        );
        let commands = draw(&mut ui, &mut state, input(0.2), &mut actions);
        assert!(ui.semantics().iter().any(|n| n.id == 600 && n.focused));
        assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color, .. } if *color == ui.theme.text.opacity(0.12))));
        draw(
            &mut ui,
            &mut state,
            Input {
                key: Some(KeyInput {
                    key: Key::Escape,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..input(0.3)
            },
            &mut actions,
        );
        draw(&mut ui, &mut state, input(0.301), &mut actions);
        assert!(!state.open);
        assert!(ui.semantics().iter().any(|n| n.id == 100 && n.focused));
        assert!(!ui.semantics().iter().any(|n| n.role == Role::MenuItem));
        draw(&mut ui, &mut state, input(1.0), &mut actions);
        assert!(!ui.needs_redraw());
    }
}

mod typeahead {
    use bhoo::*;

    #[test]
    fn typeahead_matches_prefix_skips_disabled_and_cycles_after_timeout() {
        let mut ui = Ui::default();
        let mut state = MenuTypeaheadState::default();
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 240.0,
        };
        let items = [
            ("Apple", true),
            ("Apricot", false),
            ("Avocado", true),
            ("Banana split", true),
        ];
        fn frame(
            ui: &mut Ui,
            state: &mut MenuTypeaheadState,
            bounds: Rect,
            items: &[(&str, bool)],
            key: Option<char>,
            time: f64,
            focus: bool,
        ) -> Option<usize> {
            let mut result = None;
            ui.frame(
                Input {
                    time,
                    key: key.map(|ch| KeyInput {
                        key: if ch == ' ' {
                            Key::Space
                        } else {
                            Key::Character(ch)
                        },
                        pressed: true,
                        repeat: false,
                        shift: false,
                    }),
                    ..Input::default()
                },
                bounds,
                0.0,
                |f| {
                    if focus {
                        f.request_focus(100);
                    }
                    result = f.menu_typeahead(state, 100, items);
                    for (i, (label, enabled)) in items.iter().enumerate() {
                        f.menu_item(100 + i as u64, label, *enabled);
                    }
                },
            );
            result
        }
        frame(&mut ui, &mut state, bounds, &items, None, 0.0, true);
        assert_eq!(
            frame(&mut ui, &mut state, bounds, &items, Some('a'), 0.1, false),
            Some(2)
        );
        assert_eq!(
            frame(&mut ui, &mut state, bounds, &items, Some('v'), 0.2, false),
            Some(2)
        );
        assert_eq!(
            frame(&mut ui, &mut state, bounds, &items, Some('a'), 1.5, false),
            Some(0)
        );
        assert_eq!(
            frame(&mut ui, &mut state, bounds, &items, Some('z'), 1.6, false),
            None
        );
        assert_eq!(
            frame(&mut ui, &mut state, bounds, &items, Some('b'), 1.7, false),
            Some(3)
        );
        for (i, ch) in ['a', 'n', 'a', 'n', 'a', ' ', 's'].into_iter().enumerate() {
            assert_eq!(
                frame(
                    &mut ui,
                    &mut state,
                    bounds,
                    &items,
                    Some(ch),
                    1.75 + i as f64 * 0.05,
                    false
                ),
                Some(3)
            );
        }
        assert_eq!(ui.focused(), Some(103));
        assert_eq!(
            frame(&mut ui, &mut state, bounds, &items, Some('a'), 2.11, true),
            Some(2)
        );
    }

    #[test]
    fn closed_select_typeahead_updates_without_moving_focus() {
        let mut ui=Ui::default();let mut state=MenuTypeaheadState::default();
        let bounds=Rect{x:0.,y:0.,width:220.,height:100.};
        let items=[("Balanced",true),("Battery saver",true),("Performance",true)];
        let mut current=0;
        for (time,ch,expected) in [(0.0,None,0),(0.1,Some('b'),1),(0.15,Some('a'),1),(1.0,Some('p'),2),(2.0,Some('b'),0)] {
            ui.frame(Input{time,key:ch.map(|c|KeyInput{key:Key::Character(c),pressed:true,repeat:false,shift:false}),..Input::default()},bounds,0.,|f|{
                if time==0.0 {f.request_focus(42);}
                if let Some(index)=f.select_typeahead(&mut state,42,current,&items){current=index;}
                f.select_field(42,"Profile",items[current].0,false);
            });
            assert_eq!(current,expected);assert_eq!(ui.focused(),Some(42));
        }
    }
}
