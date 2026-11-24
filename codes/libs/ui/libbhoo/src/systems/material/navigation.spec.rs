mod bar {
    use bhoo::*;

    const VIEW: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 100.0,
    };
    const DESTINATIONS: [NavigationDestination<'static>; 3] = [
        NavigationDestination {
            label: "Home",
            icon: "H",
            enabled: true,
            badge: None,
        },
        NavigationDestination {
            label: "Files",
            icon: "F",
            enabled: true,
            badge: Some((BadgeContent::Label("3"), "3 new files")),
        },
        NavigationDestination {
            label: "Settings",
            icon: "S",
            enabled: false,
            badge: None,
        },
    ];
    fn draw(ui: &mut Ui, selected: &mut usize, rtl: bool, input: Input) -> bool {
        let mut changed = false;
        ui.frame(input, VIEW, 0.0, |f| {
            changed = f.navigation_bar(10, &DESTINATIONS, selected, rtl)
        });
        changed
    }
    #[test]
    fn selection_is_immediate_and_disabled_destinations_do_not_activate() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut selected = 0;
        draw(&mut ui, &mut selected, false, Input::default());
        let nodes = ui.semantics();
        assert!(nodes.iter().find(|n| n.id == 10).unwrap().selected);
        assert!(
            nodes
                .iter()
                .find(|n| n.id == 11)
                .unwrap()
                .label
                .contains("3 new files")
        );
        assert!(!nodes.iter().find(|n| n.id == 12).unwrap().enabled);
        for (pressed, released) in [(true, false), (false, true)] {
            draw(
                &mut ui,
                &mut selected,
                false,
                Input {
                    pointer: Some([150.0, 40.0]),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        assert_eq!(selected, 1);
        assert!(ui.semantics().iter().find(|n| n.id == 11).unwrap().selected);
        for (pressed, released) in [(true, false), (false, true)] {
            draw(
                &mut ui,
                &mut selected,
                false,
                Input {
                    pointer: Some([250.0, 40.0]),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        assert_eq!(selected, 1);
        assert!(!ui.needs_redraw());
    }
    #[test]
    fn rtl_mirrors_destination_order() {
        let mut ui = Ui::default();
        let mut selected = 0;
        draw(&mut ui, &mut selected, true, Input::default());
        let nodes = ui.semantics();
        assert!(
            nodes.iter().find(|n| n.id == 10).unwrap().bounds.x
                > nodes.iter().find(|n| n.id == 12).unwrap().bounds.x
        );
    }

    #[test]
    fn selected_only_labels_center_unselected_icons_without_hiding_semantics() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut selected = 0;
        let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
            f.navigation_bar_with_labels(10, &DESTINATIONS, &mut selected, false, false);
        });
        let text = |label: &str| {
            commands.iter().find_map(|cmd| match cmd {
                DrawCommand::Text { text, bounds, .. } if text.as_ref() == label => Some(*bounds),
                _ => None,
            })
        };
        assert!(text("Home").is_some());
        assert!(text("Files").is_none());
        assert!(text("Settings").is_none());
        assert_eq!(text("H").unwrap().y, 16.0);
        assert_eq!(text("F").unwrap().y, 28.0);
        assert!(
            ui.semantics()
                .iter()
                .any(|node| node.label.contains("Files"))
        );
        assert!(!ui.needs_redraw());
    }

    #[test]
    fn indicator_and_labels_use_their_own_surface_colors() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let theme = ui.theme;
        let mut selected = 0;
        let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
            f.navigation_bar(10, &DESTINATIONS, &mut selected, false);
        });
        assert!(commands.iter().any(|cmd| matches!(cmd,
            DrawCommand::RoundedRect { bounds, color, .. }
            if bounds.width == 56.0 && bounds.height == 32.0 && *color == theme.secondary)));
        assert!(commands.iter().any(|cmd| matches!(cmd,
            DrawCommand::Text { text, color, .. } if text.as_ref() == "Home" && *color == theme.text)));
        assert!(commands.iter().any(|cmd| matches!(cmd,
            DrawCommand::Text { text, color, .. } if text.as_ref() == "Files" && *color == theme.on_surface_variant)));
        assert!(
            ui.semantics()
                .iter()
                .filter(|node| node.role == Role::Tab)
                .count()
                == 3
        );
    }

    #[test]
    fn keyboard_can_select_a_destination() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut selected = 0;
        draw(&mut ui, &mut selected, false, Input::default());
        for time in [0.1, 0.2] {
            draw(
                &mut ui,
                &mut selected,
                false,
                Input {
                    time,
                    key: Some(KeyInput {
                        key: Key::Tab,
                        pressed: true,
                        repeat: false,
                        shift: false,
                    }),
                    ..Input::default()
                },
            );
        }
        assert!(
            ui.semantics()
                .iter()
                .any(|node| node.id == 11 && node.focused)
        );
        let changed = draw(
            &mut ui,
            &mut selected,
            false,
            Input {
                time: 0.3,
                key: Some(KeyInput {
                    key: Key::Enter,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..Input::default()
            },
        );
        assert!(changed);
        assert_eq!(selected, 1);
        assert!(
            ui.semantics()
                .iter()
                .any(|node| node.id == 11 && node.selected)
        );
    }

    #[test]
    fn vector_icons_follow_selection_and_have_valid_meshes() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut selected = 1;
        let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
            f.navigation_bar_with_icons(
                10,
                &DESTINATIONS,
                &mut selected,
                false,
                true,
                &[
                    NavigationIcon::Home,
                    NavigationIcon::Folder,
                    NavigationIcon::Settings,
                ],
            );
        });
        let indexes: Vec<_> = commands
            .iter()
            .filter_map(|c| match c {
                DrawCommand::Shape { index, .. } => Some(*index as usize),
                _ => None,
            })
            .collect();
        let base = bhoo::beercss_shapes::SHAPES.len();
        assert_eq!(indexes, vec![base + 1, base + 2, base + 5]);
        for i in 0..21 {
            let shape = bhoo::shape_data(base + i).unwrap();
            assert!(shape.contours.len() >= 2 && !shape.triangles.is_empty());
            assert_eq!(shape.triangles.len() % 3, 0);
            assert!(
                shape
                    .triangles
                    .iter()
                    .all(|index| (*index as usize) < shape.points.len())
            );
            // Compare tessellated GLES area with Canvas's even-odd contours at
            // interior sample points, excluding pixels close to contour edges.
            for y in 0..40 {
                for x in 0..40 {
                    let p = [(x as f32 + 0.37) / 40.0, (y as f32 + 0.61) / 40.0];
                    let cross = |a: [f32; 2], b: [f32; 2], q: [f32; 2]| {
                        (b[0] - a[0]) * (q[1] - a[1]) - (b[1] - a[1]) * (q[0] - a[0])
                    };
                    let mut inside = false;
                    let mut near_edge = false;
                    for contour in shape.contours.windows(2) {
                        let pts = &shape.points[contour[0] as usize..contour[1] as usize];
                        for j in 0..pts.len() {
                            let a = pts[j];
                            let b = pts[(j + 1) % pts.len()];
                            let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
                            if len > 0.0 && cross(a, b, p).abs() / len < 0.0001 {
                                near_edge = true;
                            }
                            if (a[1] > p[1]) != (b[1] > p[1])
                                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
                            {
                                inside = !inside;
                            }
                        }
                    }
                    let covered = shape.triangles.chunks_exact(3).any(|t| {
                        let [a, b, c] = [
                            shape.points[t[0] as usize],
                            shape.points[t[1] as usize],
                            shape.points[t[2] as usize],
                        ];
                        if cross(a, b, c).abs() < 0.00000001 {
                            return false;
                        }
                        let s = [cross(a, b, p), cross(b, c, p), cross(c, a, p)];
                        s.iter().all(|v| *v >= -0.0000000001) || s.iter().all(|v| *v <= 0.0000000001)
                    });
                    if !near_edge {
                        assert_eq!(covered, inside, "{} at {p:?}", shape.name);
                    }
                }
            }
        }
    }
}

mod chrome {
    use bhoo::*;
    #[test]
    fn chrome_selection_is_semantic_and_motion_respects_preference() {
     let mut ui=Ui::default();let b=Rect{x:0.,y:0.,width:240.,height:80.};
     for (time,selected) in [(0.,false),(0.01,true),(0.1,true)] {
      ui.frame(Input{time,..Input::default()},b,0.,|f|{f.material_navigation_bar_item(1,b,"Explore",MaterialIcon::Menu,selected,true);});
     }
     assert!(ui.semantics().iter().find(|n|n.id==1).unwrap().selected);
     assert!(ui.needs_redraw());
     ui.motion_preference=MotionPreference::Reduced;
     ui.frame(Input{time:0.11,..Input::default()},b,0.,|f|{f.material_navigation_bar_item(1,b,"Explore",MaterialIcon::Menu,true,true);});
     assert!(!ui.needs_redraw());
    }
}

mod rail {
    use bhoo::*;
    const VIEW: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 300.0,
    };
    const ITEMS: [NavigationDestination<'static>; 3] = [
        NavigationDestination {
            label: "Home",
            icon: "H",
            enabled: true,
            badge: None,
        },
        NavigationDestination {
            label: "Files",
            icon: "F",
            enabled: true,
            badge: Some((BadgeContent::Dot, "new files")),
        },
        NavigationDestination {
            label: "Settings",
            icon: "S",
            enabled: false,
            badge: None,
        },
    ];
    fn draw(ui: &mut Ui, selected: &mut usize, input: Input) -> (bool, Vec<DrawCommand>) {
        let mut changed = false;
        let commands = ui.frame(input, VIEW, 0.0, |f| {
            changed = f.navigation_rail(
                100,
                &ITEMS,
                selected,
                false,
                false,
                &[
                    NavigationIcon::Home,
                    NavigationIcon::Folder,
                    NavigationIcon::Settings,
                ],
            )
        });
        (changed, commands)
    }
    #[test]
    fn rail_positions_destinations_and_hides_only_visual_labels() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut selected = 0;
        let (_, commands) = draw(&mut ui, &mut selected, Input::default());
        let nodes = ui.semantics();
        let first = nodes.iter().find(|n| n.id == 100).unwrap();
        let second = nodes.iter().find(|n| n.id == 101).unwrap();
        let third = nodes.iter().find(|n| n.id == 102).unwrap();
        assert_eq!((first.bounds.width, first.bounds.height), (80.0, 56.0));
        assert_eq!(second.bounds.y - first.bounds.y, 60.0);
        assert_eq!(third.bounds.y - second.bounds.y, 60.0);
        assert!(first.selected && !third.enabled && second.label.contains("new files"));
        assert!(
            commands
                .iter()
                .any(|c| matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="Home"))
        );
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="Files"))
        );
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect{bounds,color,..} if bounds.width==56.0 && bounds.height==32.0 && *color==ui.theme.secondary)));
    }
    #[test]
    fn rail_selection_and_disabled_destinations() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut selected = 0;
        draw(&mut ui, &mut selected, Input::default());
        for (pressed, released) in [(true, false), (false, true)] {
            draw(
                &mut ui,
                &mut selected,
                Input {
                    pointer: Some([40.0, 92.0]),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        assert_eq!(selected, 1);
        assert!(ui.semantics().iter().any(|n| n.id == 101 && n.selected));
        for (pressed, released) in [(true, false), (false, true)] {
            draw(
                &mut ui,
                &mut selected,
                Input {
                    pointer: Some([40.0, 152.0]),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        assert_eq!(selected, 1);
    }

    #[test]
    fn wide_rail_places_icons_and_labels_in_logical_order() {
        for rtl in [false, true] {
            let mut ui = Ui::default();
            ui.motion_preference = MotionPreference::Reduced;
            let mut selected = 0;
            let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
                f.wide_navigation_rail(
                    100,
                    &ITEMS,
                    &mut selected,
                    rtl,
                    &[
                        NavigationIcon::Home,
                        NavigationIcon::Folder,
                        NavigationIcon::Settings,
                    ],
                );
            });
            let item = ui.semantics().iter().find(|n| n.id == 100).unwrap();
            assert_eq!(
                (item.bounds.x, item.bounds.width, item.bounds.height),
                (16.0, 208.0, 56.0)
            );
            assert!(item.selected);
            let icon = commands
                .iter()
                .find_map(|c| match c {
                    DrawCommand::Shape { bounds, .. } => Some(*bounds),
                    _ => None,
                })
                .unwrap();
            let (label, align) = commands
                .iter()
                .find_map(|c| match c {
                    DrawCommand::Text {
                        bounds,
                        text,
                        style,
                        ..
                    } if text.as_ref() == "Home" => Some((*bounds, style.align)),
                    _ => None,
                })
                .unwrap();
            if rtl {
                assert!(icon.x > label.x && align == 2);
            } else {
                assert!(icon.x < label.x && align == 0);
            }
            assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect{bounds,color,..} if *bounds==item.bounds && *color==ui.theme.secondary)));
            assert!(
                ui.semantics()
                    .iter()
                    .find(|n| n.id == 101)
                    .unwrap()
                    .label
                    .contains("new files")
            );
        }
    }
    #[test]
    fn wide_rail_click_selects_enabled_destinations() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut selected = 0;
        for (pressed, released) in [(false, false), (true, false), (false, true)] {
            ui.frame(
                Input {
                    pointer: Some([100.0, 92.0]),
                    pressed,
                    released,
                    ..Input::default()
                },
                VIEW,
                0.0,
                |f| {
                    f.wide_navigation_rail(100, &ITEMS, &mut selected, false, &[]);
                },
            );
        }
        assert_eq!(selected, 1);
        assert!(ui.semantics().iter().any(|n| n.id == 101 && n.selected));
    }

    #[test]
    fn animated_rail_reverses_from_current_geometry_and_snaps_for_reduced_motion() {
        fn sample(ui: &mut Ui, selected: &mut usize, expanded: bool, time: f64) -> (f32, bool) {
            let commands = ui.frame(
                Input {
                    time,
                    ..Input::default()
                },
                VIEW,
                0.0,
                |f| {
                    f.animated_navigation_rail(100, &ITEMS, selected, false, true, &[], expanded);
                },
            );
            let width = commands
                .iter()
                .find_map(|c| match c {
                    DrawCommand::RoundedRect { bounds, color, .. }
                        if *color == ui.theme.surface_low =>
                    {
                        Some(bounds.width)
                    }
                    _ => None,
                })
                .unwrap();
            (width, ui.needs_redraw())
        }
        let mut ui = Ui::default();
        let mut selected = 0;
        assert_eq!(sample(&mut ui, &mut selected, false, 0.0).0, 80.0);
        sample(&mut ui, &mut selected, true, 0.01);
        let (in_flight, active) = sample(&mut ui, &mut selected, true, 0.06);
        assert!(active && in_flight > 80.0 && in_flight < 240.0);
        let (reversed, _) = sample(&mut ui, &mut selected, false, 0.06);
        assert!((reversed - in_flight).abs() < 0.01);
        let (settled, active) = sample(&mut ui, &mut selected, false, 2.0);
        assert!((settled - 80.0).abs() < 0.01 && !active);
        assert!(ui.semantics().iter().any(|n| n.id == 100 && n.selected));
        ui.motion_preference = MotionPreference::Reduced;
        let (wide, active) = sample(&mut ui, &mut selected, true, 2.1);
        assert!((wide - 240.0).abs() < 0.01 && !active);
    }

    #[test]
    fn modal_rail_traps_focus_and_dismisses_with_scrim_or_escape() {
        fn frame(ui: &mut Ui, state: &mut OverlayState, selected: &mut usize, input: Input) {
            ui.frame(input, VIEW, 0.0, |f| {
                f.with_enabled(!state.open, |f| {
                    if f.button(10, "Open rail").clicked {
                        state.open = true;
                    }
                });
                f.overlay(
                    20,
                    state,
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 240.0,
                        height: 300.0,
                    },
                    true,
                    |f, open| {
                        if f.button(30, "Close rail").clicked {
                            *open = false;
                        }
                        f.wide_navigation_rail(40, &ITEMS, selected, false, &[]);
                    },
                );
            });
        }
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut state = OverlayState::default();
        state.kind = OverlayKind::LeftSideSheet;
        let mut selected = 0;
        for (pressed, released) in [(true, false), (false, true)] {
            frame(
                &mut ui,
                &mut state,
                &mut selected,
                Input {
                    pointer: Some([15.0, 20.0]),
                    pressed,
                    released,
                    ..Input::default()
                },
            );
        }
        assert!(state.open);
        assert_eq!(ui.focused(), Some(30));
        assert!(!ui.semantics().iter().find(|n| n.id == 10).unwrap().enabled);
        for i in 1..5 {
            frame(
                &mut ui,
                &mut state,
                &mut selected,
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
            assert!(matches!(ui.focused(), Some(30 | 40 | 41)));
        }
        frame(
            &mut ui,
            &mut state,
            &mut selected,
            Input {
                time: 6.0,
                pointer: Some([280.0, 200.0]),
                pressed: true,
                ..Input::default()
            },
        );
        assert!(!state.open);
        frame(
            &mut ui,
            &mut state,
            &mut selected,
            Input {
                time: 7.0,
                ..Input::default()
            },
        );
        assert_eq!(ui.focused(), Some(10));
        state.open = true;
        frame(
            &mut ui,
            &mut state,
            &mut selected,
            Input {
                time: 8.0,
                ..Input::default()
            },
        );
        frame(
            &mut ui,
            &mut state,
            &mut selected,
            Input {
                time: 9.0,
                key: Some(KeyInput {
                    key: Key::Escape,
                    pressed: true,
                    repeat: false,
                    shift: false,
                }),
                ..Input::default()
            },
        );
        assert!(!state.open);
    }

    #[test]
    fn expressive_rail_uses_android_dimensions_and_content_width_indicator() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut selected = 0;
        let icons = [
            NavigationIcon::Alarm,
            NavigationIcon::Clock,
            NavigationIcon::Timer,
        ];
        let mut render = |ui: &mut Ui, expanded| {
            ui.frame(Input::default(), VIEW, 0.0, |f| {
                f.expressive_navigation_rail(100, &ITEMS, &mut selected, false, &icons, expanded);
            })
        };
        let commands = render(&mut ui, false);
        let nodes = ui.semantics();
        let first = nodes.iter().find(|n| n.id == 100).unwrap();
        let second = nodes.iter().find(|n| n.id == 101).unwrap();
        assert_eq!((first.bounds.width, first.bounds.height), (96.0, 64.0));
        assert_eq!(second.bounds.y - first.bounds.y, 68.0);
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color,..} if bounds.width==56.0 && bounds.height==32.0 && *color==ui.theme.secondary)));
        let commands = render(&mut ui, true);
        let item = ui.semantics().iter().find(|n| n.id == 100).unwrap().bounds;
        let indicator = commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::RoundedRect { bounds, color, .. }
                    if *color == ui.theme.secondary && bounds.height == 56.0 =>
                {
                    Some(*bounds)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(item.x, 16.0);
        assert_eq!(indicator.x, 20.0);
        assert!(indicator.width < item.width && indicator.width > 64.0);
        assert_eq!(indicator.y, item.y + 4.0);
    }

    #[test]
    fn expressive_header_can_share_the_rails_transition_sample() {
        fn sample(ui: &mut Ui, selected: &mut usize, expanded: bool, time: f64) -> (f32, f32) {
            let mut progress = 0.0;
            let commands = ui.frame(
                Input {
                    time,
                    ..Input::default()
                },
                VIEW,
                0.0,
                |f| {
                    progress = f.navigation_rail_expansion(100, expanded);
                    f.expressive_navigation_rail_at_progress(
                        100,
                        &ITEMS,
                        selected,
                        false,
                        &[],
                        progress,
                    );
                },
            );
            let width = commands
                .iter()
                .find_map(|c| match c {
                    DrawCommand::RoundedRect { bounds, color, .. }
                        if *color == ui.theme.background && bounds.height > 100.0 =>
                    {
                        Some(bounds.width)
                    }
                    _ => None,
                })
                .unwrap();
            (progress, width)
        }
        let mut ui = Ui::default();
        let mut selected = 0;
        assert_eq!(sample(&mut ui, &mut selected, false, 0.0), (0.0, 96.0));
        sample(&mut ui, &mut selected, true, 0.01);
        let (p, width) = sample(&mut ui, &mut selected, true, 0.05);
        assert!(p > 0.0 && p < 1.0);
        assert!((width - (96.0 + 144.0 * p)).abs() < 0.001);
        let (reverse, width2) = sample(&mut ui, &mut selected, false, 0.05);
        assert!((reverse - p).abs() < 0.0001 && (width2 - width).abs() < 0.0001);
        ui.motion_preference = MotionPreference::Reduced;
        assert_eq!(sample(&mut ui, &mut selected, true, 0.1), (1.0, 240.0));
    }

    #[test]
    fn modal_rail_surface_has_no_inset_or_mismatched_inner_background() {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut overlay = OverlayState::default();
        overlay.open = true;
        overlay.kind = OverlayKind::LeftSideSheet;
        overlay.style = OverlayStyle::NavigationRail;
        let mut selected = 0;
        let panel = Rect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 300.0,
        };
        let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
            f.overlay(20, &mut overlay, panel, true, |f, _| {
                f.expressive_navigation_rail_at_progress(40, &ITEMS, &mut selected, false, &[], 1.0);
            });
        });
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect{bounds,color,..} if *bounds==panel && *color==ui.theme.background)));
        let selected = ui.semantics().iter().find(|n| n.id == 40).unwrap();
        assert_eq!(selected.bounds.x, 16.0);
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect{bounds,color,..} if *color==ui.theme.secondary && bounds.x==20.0 && bounds.width<selected.bounds.width)));
    }
}

mod drawer_container_tests {
    use crate::*;

    #[test]
    fn navigation_drawer_and_headers_select_items() {
        let mut ui = Ui::default();
        let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 400.0 };
        let items = [
            ("Home", Some(MaterialIcon::Home)),
            ("Settings", Some(MaterialIcon::Settings)),
        ];
        let mut sel = 0usize;
        let _ = ui.frame(Input::default(), bounds, 0.0, |f| {
            f.navigation_drawer_header("App", Some("user@example.test"));
            f.navigation_drawer_section_header("General");
            f.navigation_drawer(50, Rect { x: 0.0, y: 160.0, width: 260.0, height: 200.0 }, &items, &mut sel);
        });
        let settings_b = ui.semantics().iter().find(|n| n.label.as_ref() == "Settings").unwrap().bounds;
        let p = [settings_b.x + 20.0, settings_b.y + 20.0];
        let mut changed = false;
        for (pressed, released, time) in [(true, false, 0.1), (false, true, 0.15)] {
            ui.frame(Input { pointer: Some(p), pressed, released, time, ..Input::default() }, bounds, 0.0, |f| {
                f.navigation_drawer_header("App", Some("user@example.test"));
                f.navigation_drawer_section_header("General");
                changed |= f.navigation_drawer(50, Rect { x: 0.0, y: 160.0, width: 260.0, height: 200.0 }, &items, &mut sel);
            });
        }
        assert!(changed && sel == 1);
    }
}
