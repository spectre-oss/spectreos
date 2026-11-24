mod dialog_semantics {
    use bhoo::*;
    const VIEW: Rect = Rect {
        x: 0.,
        y: 0.,
        width: 400.,
        height: 300.,
    };
    const PANEL: Rect = Rect {
        x: 40.,
        y: 50.,
        width: 320.,
        height: 200.,
    };
    fn draw(ui: &mut Ui, state: &mut OverlayState, input: Input) {
        ui.frame(input, VIEW, 0., |f| {
            f.without_input(state.open, |f| {
                f.button(100, "Open");
            });
            f.overlay(500, state, PANEL, true, |f, open| {
                f.button(600, "Cancel");
                if f.button(601, "Save").clicked {
                    *open = false;
                }
            });
        });
    }
    #[test]
    fn modal_dialog_exposes_title_description_and_removes_them_on_close() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut state = OverlayState::default();
        state.kind = OverlayKind::Dialog;
        state.accessible_title = Some("Save changes?".into());
        state.accessible_description = Some("Unsaved changes".into());
        state.open = true;
        draw(&mut ui, &mut state, Input::default());
        assert!(
            ui.semantics()
                .iter()
                .any(|n| n.role == Role::Dialog && n.label.as_ref() == "Save changes?")
        );
        assert!(
            ui.semantics()
                .iter()
                .any(|n| n.role == Role::DialogDescription && n.label.as_ref() == "Unsaved changes")
        );
        assert!(!ui.semantics().iter().find(|n| n.id == 100).unwrap().enabled);
        assert!(ui.semantics().iter().find(|n| n.id == 600).unwrap().enabled);
        state.open = false;
        draw(
            &mut ui,
            &mut state,
            Input {
                time: 1.,
                ..Input::default()
            },
        );
        assert!(
            !ui.semantics()
                .iter()
                .any(|n| matches!(n.role, Role::Dialog | Role::DialogDescription))
        );
    }

    #[test]
    fn dialog_dismiss_policy_controls_escape_and_outside_independently() {
        for (policy, escape, outside) in [
            (OverlayDismissPolicy::EscapeOrOutside, true, true),
            (OverlayDismissPolicy::EscapeOnly, true, false),
            (OverlayDismissPolicy::OutsideOnly, false, true),
            (OverlayDismissPolicy::ExplicitOnly, false, false),
        ] {
            for (by_escape, expected) in [(true, escape), (false, outside)] {
                let mut ui = Ui::default();
                ui.motion_preference = MotionPreference::Reduced;
                let mut state = OverlayState::default();
                state.kind = OverlayKind::Dialog;
                state.dismiss_policy = policy;
                state.open = true;
                draw(&mut ui, &mut state, Input::default());
                let input = if by_escape {
                    Input {
                        time: 1.,
                        key: Some(KeyInput {
                            key: Key::Escape,
                            pressed: true,
                            repeat: false,
                            shift: false,
                        }),
                        ..Input::default()
                    }
                } else {
                    Input {
                        time: 1.,
                        pointer: Some([5., 5.]),
                        pressed: true,
                        ..Input::default()
                    }
                };
                draw(&mut ui, &mut state, input);
                assert_eq!(!state.open, expected, "{policy:?}, escape={by_escape}");
                if !expected {
                    assert!(ui.has_escape_target(), "modal remains active");
                    let save = ui.semantics().iter().find(|n| n.id == 601).unwrap().bounds;
                    let pointer = Some([save.x + save.width / 2., save.y + save.height / 2.]);
                    draw(
                        &mut ui,
                        &mut state,
                        Input {
                            time: 2.,
                            pointer,
                            pressed: true,
                            ..Input::default()
                        },
                    );
                    draw(
                        &mut ui,
                        &mut state,
                        Input {
                            time: 2.1,
                            pointer,
                            released: true,
                            ..Input::default()
                        },
                    );
                    assert!(!state.open, "caller may explicitly close");
                }
            }
        }
    }

    #[test]
    fn fullscreen_dialog_uses_viewport_and_square_surface() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut state = OverlayState::default();
        state.kind = OverlayKind::Dialog;
        state.accessible_title = Some("Edit".into());
        state.fullscreen = true;
        state.open = true;
        let commands = ui.frame(Input::default(), VIEW, 0., |f| {
            f.overlay(500, &mut state, PANEL, true, |f, _| {
                f.button(600, "Close");
            });
        });
        let dialog = ui
            .semantics()
            .iter()
            .find(|n| n.role == Role::Dialog)
            .unwrap();
        assert_eq!(dialog.bounds, VIEW);
        assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, radius, .. } if *bounds == VIEW && *radius == 0.)));
    }

    #[test]
    fn suspended_parent_preserves_child_ownership_and_restores_focus() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut parent = OverlayState::default();
        parent.kind = OverlayKind::Dialog;
        parent.fullscreen = true;
        parent.open = true;
        parent.accessible_title = Some("Parent".into());
        let mut child = OverlayState::default();
        child.kind = OverlayKind::Dialog;
        child.accessible_title = Some("Child".into());
        let render =
            |ui: &mut Ui, input: Input, parent: &mut OverlayState, child: &mut OverlayState| {
                parent.suspended = child.open;
                ui.frame(input, VIEW, 0., |f| {
                    f.overlay(500, parent, VIEW, true, |f, _| {
                        f.button(601, "Open confirmation");
                    });
                    f.overlay(700, child, PANEL, true, |f, _| {
                        f.button(701, "Back");
                    });
                });
            };
        render(&mut ui, Input::default(), &mut parent, &mut child);
        assert_eq!(ui.focused(), Some(601));
        child.open = true;
        render(
            &mut ui,
            Input {
                time: 1.,
                ..Input::default()
            },
            &mut parent,
            &mut child,
        );
        assert_eq!(ui.focused(), Some(701));
        assert!(!ui.semantics().iter().find(|n| n.id == 601).unwrap().enabled);
        assert!(ui.semantics().iter().find(|n| n.id == 701).unwrap().enabled);
        render(
            &mut ui,
            Input {
                time: 2.,
                key: Some(KeyInput {
                    key: Key::Escape,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..Input::default()
            },
            &mut parent,
            &mut child,
        );
        assert!(parent.open);
        assert!(!child.open);
        render(
            &mut ui,
            Input {
                time: 2.1,
                ..Input::default()
            },
            &mut parent,
            &mut child,
        );
        assert_eq!(ui.focused(), Some(601));
    }
}

mod bottom_sheet {
    use bhoo::*;
    const VIEW:Rect=Rect{x:0.,y:0.,width:400.,height:600.};
    const SHEET:Rect=Rect{x:0.,y:340.,width:400.,height:260.};
    fn draw(ui:&mut Ui,state:&mut OverlayState,input:Input)->f32 {
        let commands=ui.frame(input,VIEW,0.,|f|{f.overlay(500,state,SHEET,true,|_,_|{});});
        commands.iter().find_map(|c|match c {DrawCommand::RoundedRect{bounds,radius,..} if *radius==28. && bounds.width==400.=>Some(bounds.y),_=>None}).unwrap()
    }
    fn input(time:f64)->Input {Input {time,..Input::default()}}
    fn sheet()->OverlayState {let mut state=OverlayState::default();state.kind=OverlayKind::BottomSheet;state.drag_to_dismiss=true;state.open=true;state}
    #[test]
    fn short_drag_follows_pointer_then_settles_and_can_be_interrupted() {
        let mut ui=Ui::default();let mut state=sheet();draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(0.6)});
        let dragged=draw(&mut ui,&mut state,Input {pointer:Some([200.,375.]),..input(0.65)});assert!((dragged-365.).abs()<0.1);
        let released=draw(&mut ui,&mut state,Input {pointer:Some([200.,375.]),released:true,..input(0.7)});assert!((released-dragged).abs()<0.1);assert!(state.open);
        let settling=draw(&mut ui,&mut state,input(0.8));assert!(settling<released && settling>340.);
        let restart=draw(&mut ui,&mut state,Input {pointer:Some([200.,settling+10.]),pressed:true,..input(0.8)});assert!((restart-settling).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,settling+10.]),cancelled:true,..input(0.81)});assert!(state.open);
    }
    #[test]
    fn long_drag_dismisses_without_initial_jump() {
        let mut ui=Ui::default();let mut state=sheet();draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(0.6)});
        let dragged=draw(&mut ui,&mut state,Input {pointer:Some([200.,490.]),..input(0.65)});
        let released=draw(&mut ui,&mut state,Input {pointer:Some([200.,490.]),released:true,..input(0.7)});
        assert!(!state.open);assert!((released-dragged).abs()<0.5);
        assert!(draw(&mut ui,&mut state,input(0.75))>=released);
    }
    #[test]
    fn reopening_during_drag_dismissal_preserves_position() {
        let mut ui=Ui::default();let mut state=sheet();draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(0.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,490.]),..input(0.65)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,490.]),released:true,..input(0.7)});
        let closing=draw(&mut ui,&mut state,input(0.73));state.open=true;
        let reopened=draw(&mut ui,&mut state,input(0.73));assert!((closing-reopened).abs()<0.1);
    }
    #[test]
    fn reduced_motion_snaps_short_drag_back() {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=sheet();draw(&mut ui,&mut state,input(0.));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(0.1)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,400.]),..input(0.2)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,400.]),released:true,..input(0.3)});
        assert_eq!(draw(&mut ui,&mut state,input(0.31)),340.);assert!(state.open);
    }
    #[test]
    fn fast_downward_flick_dismisses_but_slow_and_horizontal_drags_do_not() {
        for (point,elapsed,expected) in [([200.,375.],0.01,false),([200.,375.],0.2,true),([270.,375.],0.01,true)] {
            let mut ui=Ui::default();let mut state=sheet();draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
            draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(0.6)});
            draw(&mut ui,&mut state,Input {pointer:Some(point),..input(0.6+elapsed)});
            draw(&mut ui,&mut state,Input {pointer:Some(point),released:true,..input(0.61+elapsed)});
            assert_eq!(state.open,expected,"point {point:?}, elapsed {elapsed}");
        }
    }
    #[test]
    fn upward_release_after_long_drag_settles_open() {
        let mut ui=Ui::default();let mut state=sheet();draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(0.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,500.]),..input(0.7)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,480.]),..input(0.72)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,480.]),released:true,..input(0.73)});
        assert!(state.open);
    }
    #[test]
    fn standard_sheet_settles_between_peek_and_expanded() {
        let mut ui=Ui::default();let mut state=sheet();state.bottom_sheet_peek=Some(96.);
        draw(&mut ui,&mut state,input(0.));let collapsed=draw(&mut ui,&mut state,input(0.5));assert!((collapsed-504.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,514.]),pressed:true,..input(0.6)});
        let moved=draw(&mut ui,&mut state,Input {pointer:Some([200.,404.]),..input(0.8)});assert!((moved-394.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,404.]),released:true,..input(0.81)});assert!(state.open && state.bottom_sheet_expanded);
        let expanded=draw(&mut ui,&mut state,input(1.5));assert!((expanded-340.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(1.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,460.]),..input(1.8)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,460.]),released:true,..input(1.81)});assert!(state.open && !state.bottom_sheet_expanded);
        assert!((draw(&mut ui,&mut state,input(2.5))-504.).abs()<0.1);
    }
    #[test]
    fn standard_sheet_cancel_returns_to_original_detent_with_reduced_motion() {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=sheet();state.bottom_sheet_peek=Some(96.);
        assert!((draw(&mut ui,&mut state,input(0.))-504.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,514.]),pressed:true,..input(0.1)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,370.]),..input(0.2)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,370.]),cancelled:true,..input(0.21)});
        assert!(!state.bottom_sheet_expanded);assert!((draw(&mut ui,&mut state,input(0.22))-504.).abs()<0.1);
    }
    #[test]
    fn slow_handle_motion_uses_direction_without_a_flick_threshold() {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=sheet();state.bottom_sheet_peek=Some(96.);
        draw(&mut ui,&mut state,input(0.));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,514.]),pressed:true,..input(0.1)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,494.]),..input(0.3)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,494.]),released:true,..input(0.31)});
        assert!(state.bottom_sheet_expanded);assert!((draw(&mut ui,&mut state,input(0.32))-340.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(0.4)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,370.]),..input(0.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,370.]),released:true,..input(0.61)});
        assert!(!state.bottom_sheet_expanded);assert!((draw(&mut ui,&mut state,input(0.62))-504.).abs()<0.1);
    }
    #[test]
    fn half_sheet_small_downward_handle_drag_settles_to_half() {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=sheet();state.bottom_sheet_peek=Some(96.);state.bottom_sheet_half_ratio=Some(0.3);state.bottom_sheet_expanded=true;
        draw(&mut ui,&mut state,input(0.));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,350.]),pressed:true,..input(0.1)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,370.]),..input(0.3)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,370.]),released:true,..input(0.31)});
        assert!(state.bottom_sheet_half_expanded);assert!((draw(&mut ui,&mut state,input(0.32))-420.).abs()<0.1);
    }
    #[test]
    fn changing_open_sheet_target_and_peek_retargets_without_jump() {
        let mut ui=Ui::default();let mut state=sheet();state.bottom_sheet_peek=Some(96.);
        draw(&mut ui,&mut state,input(0.));assert!((draw(&mut ui,&mut state,input(0.5))-504.).abs()<0.1);
        state.bottom_sheet_expanded=true;
        assert!((draw(&mut ui,&mut state,input(0.6))-504.).abs()<0.1);
        let moving=draw(&mut ui,&mut state,input(0.7));assert!(moving<504. && moving>340.);
        state.bottom_sheet_expanded=false;
        assert!((draw(&mut ui,&mut state,input(0.7))-moving).abs()<0.1);
        assert!(draw(&mut ui,&mut state,input(0.8))>moving);
        assert!((draw(&mut ui,&mut state,input(1.5))-504.).abs()<0.1);
        state.bottom_sheet_peek=Some(144.);
        assert!((draw(&mut ui,&mut state,input(1.6))-504.).abs()<0.1);
        assert!((draw(&mut ui,&mut state,input(2.3))-456.).abs()<0.1);
    }
    #[test]
    fn standard_sheet_visits_half_expanded_between_other_detents() {
        let mut ui=Ui::default();let mut state=sheet();state.bottom_sheet_peek=Some(96.);state.bottom_sheet_half_ratio=Some(0.3);
        draw(&mut ui,&mut state,input(0.));assert!((draw(&mut ui,&mut state,input(0.5))-504.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,514.]),pressed:true,..input(0.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,454.]),..input(0.7)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,454.]),released:true,..input(0.71)});
        assert!(state.bottom_sheet_half_expanded && !state.bottom_sheet_expanded);assert!((draw(&mut ui,&mut state,input(1.4))-420.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,430.]),pressed:true,..input(1.5)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,370.]),..input(1.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,370.]),released:true,..input(1.61)});
        assert!(state.bottom_sheet_expanded && !state.bottom_sheet_half_expanded);assert!((draw(&mut ui,&mut state,input(2.3))-340.).abs()<0.1);
    }
    #[test]
    fn nested_content_transfers_upward_movement_from_sheet_to_scroll() {
        let mut ui=Ui::default();let mut state=sheet();state.bottom_sheet_peek=Some(96.);state.bottom_sheet_content_top=Some(64.);state.bottom_sheet_content_height=500.;
        draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,580.]),pressed:true,..input(0.6)});
        let y=draw(&mut ui,&mut state,Input {pointer:Some([200.,400.]),..input(0.8)});
        assert!((y-340.).abs()<0.1);assert!((state.bottom_sheet_scroll_offset-16.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,400.]),released:true,..input(0.81)});assert!(state.bottom_sheet_expanded);
        draw(&mut ui,&mut state,input(0.9));assert!(state.bottom_sheet_scroll_offset>16.);
    }
    #[test]
    fn nested_content_scrolls_to_top_before_collapsing_sheet() {
        let mut ui=Ui::default();let mut state=sheet();state.bottom_sheet_peek=Some(96.);state.bottom_sheet_expanded=true;state.bottom_sheet_content_top=Some(64.);state.bottom_sheet_content_height=500.;state.bottom_sheet_scroll_offset=60.;
        draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,450.]),pressed:true,..input(0.6)});
        let y=draw(&mut ui,&mut state,Input {pointer:Some([200.,490.]),..input(0.7)});assert!((y-340.).abs()<0.1);assert!((state.bottom_sheet_scroll_offset-20.).abs()<0.1);
        let y=draw(&mut ui,&mut state,Input {pointer:Some([200.,530.]),..input(0.8)});assert!((y-360.).abs()<0.1);assert_eq!(state.bottom_sheet_scroll_offset,0.);
    }
    #[test]
    fn cancelled_nested_drag_restores_collapsed_detent_after_full_expansion() {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=sheet();state.bottom_sheet_peek=Some(96.);state.bottom_sheet_content_top=Some(64.);state.bottom_sheet_content_height=500.;
        draw(&mut ui,&mut state,input(0.));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,580.]),pressed:true,..input(0.1)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,400.]),..input(0.2)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,400.]),cancelled:true,..input(0.21)});
        assert!(!state.bottom_sheet_expanded);assert!((draw(&mut ui,&mut state,input(0.22))-504.).abs()<0.1);
    }
    #[test]
    fn nested_downward_fling_does_not_move_sheet_after_touch_ends() {
        let mut ui=Ui::default();let mut state=sheet();state.bottom_sheet_peek=Some(96.);state.bottom_sheet_half_ratio=Some(0.3);state.bottom_sheet_expanded=true;state.bottom_sheet_content_top=Some(64.);state.bottom_sheet_content_height=500.;state.bottom_sheet_scroll_offset=60.;
        draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,450.]),pressed:true,..input(0.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,490.]),..input(0.65)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,490.]),released:true,..input(0.66)});
        for i in 1..60 { draw(&mut ui,&mut state,input(0.66+i as f64*0.016)); }
        assert!(state.bottom_sheet_expanded);assert_eq!(state.bottom_sheet_scroll_offset,0.);
        assert!((draw(&mut ui,&mut state,input(2.))-340.).abs()<0.1);
    }
    #[test]
    fn slow_nested_drag_uses_last_direction_to_choose_detent() {
        let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=sheet();state.bottom_sheet_peek=Some(96.);state.bottom_sheet_content_top=Some(64.);state.bottom_sheet_content_height=500.;
        draw(&mut ui,&mut state,input(0.));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,580.]),pressed:true,..input(0.1)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,560.]),..input(0.3)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,560.]),released:true,..input(0.5)});
        assert!(state.bottom_sheet_expanded);
        assert!((draw(&mut ui,&mut state,input(0.51))-340.).abs()<0.1);
        draw(&mut ui,&mut state,Input {pointer:Some([200.,450.]),pressed:true,..input(0.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,470.]),..input(0.8)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,470.]),released:true,..input(1.0)});
        assert!(!state.bottom_sheet_expanded);
        assert!((draw(&mut ui,&mut state,input(1.01))-504.).abs()<0.1);
    }
    #[test]
    fn nested_slow_fling_stops_before_top_without_moving_sheet() {
        let mut ui=Ui::default();let mut state=sheet();state.bottom_sheet_peek=Some(96.);state.bottom_sheet_expanded=true;state.bottom_sheet_content_top=Some(64.);state.bottom_sheet_content_height=500.;state.bottom_sheet_scroll_offset=150.;
        draw(&mut ui,&mut state,input(0.));draw(&mut ui,&mut state,input(0.5));
        draw(&mut ui,&mut state,Input {pointer:Some([200.,450.]),pressed:true,..input(0.6)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,460.]),..input(0.7)});
        draw(&mut ui,&mut state,Input {pointer:Some([200.,460.]),released:true,..input(0.71)});
        for i in 1..60 { assert!((draw(&mut ui,&mut state,input(0.71+i as f64*0.016))-340.).abs()<0.1); }
        assert!(state.bottom_sheet_expanded && state.bottom_sheet_scroll_offset>0.);
    }
}

mod chrome_builder_tests {
    use crate::*;

    #[test]
    fn dialog_fullscreen_bottom_and_side_sheet_chrome_builders_work() {
        let mut ui = Ui::default();
        let bounds = Rect { x: 0.0, y: 0.0, width: 400.0, height: 320.0 };
        let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
            f.bottom_sheet_drag_handle();
            f.fullscreen_dialog_header(10, "Edit", "Save");
            f.side_sheet_header(20, "Filters");
            f.dialog_chrome(30, "Confirm?", "Proceed with action.", Some("Cancel"), "OK");
        });
        assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Confirm?")));
        assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "OK"));
        assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "Save"));
    }
}
