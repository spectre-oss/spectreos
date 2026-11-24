mod collapse {
    use bhoo::*;
    const B: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 300.0,
    };
    fn sample(ui: &mut Ui, time: f64, open: bool) -> (f32, f32) {
        let mut amount = -1.0;
        let mut after = 0.0;
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            B,
            0.0,
            |f| {
                amount = f.collapse(10, open, 96.0, |f| {
                    f.button(11, "Action");
                });
                after = f.reserve(1.0).y;
            },
        );
        (amount, after)
    }
    #[test]
    fn collapse_moves_following_content_and_blocks_input_while_exiting() {
        let mut ui = Ui::default();
        assert_eq!(sample(&mut ui, 0.0, false), (0.0, 0.0));
        assert_eq!(sample(&mut ui, 1.0, true), (0.0, 0.0));
        let mid = sample(&mut ui, 1.1, true);
        assert!(mid.0 > 0.0 && mid.0 < 1.0);
        assert!((mid.1 - mid.0 * 96.0).abs() < 0.01);
        assert_eq!(sample(&mut ui, 1.4, true), (1.0, 96.0));
        assert_eq!(sample(&mut ui, 2.0, false), (1.0, 96.0));
        assert!(ui.semantics().iter().all(|node| !node.enabled));
        assert_eq!(sample(&mut ui, 2.4, false), (0.0, 0.0));
    }
}

mod fade {
    use bhoo::*;
    const BOUNDS: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    fn sample(ui: &mut Ui, time: f64, open: bool) -> f32 {
        let mut amount = -1.0;
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            0.0,
            |f| {
                amount = f.fade(10, open, BOUNDS, |f| {
                    f.button(11, "Action");
                });
            },
        );
        amount
    }
    #[test]
    fn fade_has_mui_durations_and_blocks_input_on_exit() {
        let mut ui = Ui::default();
        assert_eq!(sample(&mut ui, 0.0, false), 0.0);
        assert_eq!(sample(&mut ui, 1.0, true), 0.0);
        assert!(sample(&mut ui, 1.1125, true) > 0.5);
        assert_eq!(sample(&mut ui, 1.2251, true), 1.0);
        assert_eq!(sample(&mut ui, 2.0, false), 1.0);
        assert!(ui.semantics().iter().all(|node| !node.enabled));
        assert_eq!(sample(&mut ui, 2.196, false), 0.0);
    }
}

mod grow {
    use bhoo::*;
    const B: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    fn sample(ui: &mut Ui, time: f64, open: bool) -> (f32, f32) {
        let mut state = (-1.0, -1.0);
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            B,
            0.0,
            |f| {
                state = f.grow(10, open, B, |f| {
                    f.button(11, "Action");
                });
            },
        );
        state
    }
    #[test]
    fn grow_finishes_scale_before_opacity_and_delays_scale_on_exit() {
        let mut ui = Ui::default();
        assert_eq!(sample(&mut ui, 0.0, false), (0.0, 0.75));
        assert_eq!(sample(&mut ui, 1.0, true), (0.0, 0.75));
        let mid = sample(&mut ui, 1.2, true);
        assert!(mid.0 > 0.0 && mid.0 < 1.0);
        assert_eq!(mid.1, 1.0);
        assert_eq!(sample(&mut ui, 1.5, true), (1.0, 1.0));
        assert_eq!(sample(&mut ui, 2.0, false), (1.0, 1.0));
        let early = sample(&mut ui, 2.03, false);
        assert!(early.0 < 1.0);
        assert_eq!(early.1, 1.0);
        assert!(ui.semantics().iter().all(|node| !node.enabled));
        assert_eq!(sample(&mut ui, 2.5, false), (0.0, 0.75));
    }
}

mod page {
    use bhoo::*;
    const VIEW: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 300.0,
    };
    const PAGE: Rect = Rect {
        x: 64.0,
        y: 60.0,
        width: 120.0,
        height: 100.0,
    };
    fn sample(ui: &mut Ui, time: f64, active: bool, direction: PageDirection) -> (f32, Option<Rect>) {
        let mut progress = -1.0;
        let mut posed = None;
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            VIEW,
            0.0,
            |f| {
                progress = f.page(10, active, PAGE, direction, |f, b| {
                    posed = Some(b);
                    f.button(11, "Action");
                });
            },
        );
        (progress, posed)
    }
    #[test]
    fn page_entry_fades_and_moves_then_inactive_page_disappears() {
        let mut ui = Ui::default();
        assert_eq!(
            sample(&mut ui, 0.0, false, PageDirection::Left),
            (0.0, None)
        );
        assert_eq!(sample(&mut ui, 1.0, true, PageDirection::Left), (0.0, None));
        let mid = sample(&mut ui, 1.2, true, PageDirection::Left);
        assert!(mid.0 > 0.0 && mid.0 < 1.0);
        assert!(mid.1.unwrap().x > 0.0 && mid.1.unwrap().x < PAGE.x);
        assert_eq!(
            sample(&mut ui, 1.401, true, PageDirection::Left),
            (1.0, Some(PAGE))
        );
        assert_eq!(
            sample(&mut ui, 2.0, false, PageDirection::Left),
            (0.0, None)
        );
        assert!(ui.semantics().is_empty());
    }
}

mod slide {
    use bhoo::*;
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 300.0,
    };
    const TARGET: Rect = Rect {
        x: 0.0,
        y: 40.0,
        width: 100.0,
        height: 100.0,
    };
    fn sample(ui: &mut Ui, time: f64, open: bool) -> (f32, Option<f32>) {
        let mut amount = -1.0;
        let mut y = None;
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            VIEWPORT,
            0.0,
            |f| {
                amount = f.slide_up(10, open, TARGET, |f, posed| {
                    y = Some(posed.y);
                    f.button(11, "Action");
                });
            },
        );
        (amount, y)
    }
    #[test]
    fn slide_enters_from_viewport_edge_and_disables_exit_input() {
        let mut ui = Ui::default();
        assert_eq!(sample(&mut ui, 0.0, false), (0.0, None));
        assert_eq!(sample(&mut ui, 1.0, true), (0.0, None));
        let mid = sample(&mut ui, 1.1, true);
        assert!(mid.0 > 0.0 && mid.0 < 1.0);
        assert!(mid.1.unwrap() > 40.0 && mid.1.unwrap() < 300.0);
        assert_eq!(sample(&mut ui, 1.226, true), (1.0, Some(40.0)));
        assert_eq!(sample(&mut ui, 2.0, false), (1.0, Some(40.0)));
        assert!(ui.semantics().iter().all(|node| !node.enabled));
        assert_eq!(sample(&mut ui, 2.196, false), (0.0, None));
    }
}

mod zoom {
    use bhoo::*;
    const BOUNDS: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    fn sample(ui: &mut Ui, time: f64, open: bool) -> f32 {
        let mut amount = -1.0;
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            BOUNDS,
            0.0,
            |f| {
                amount = f.zoom(10, open, BOUNDS, |f| {
                    f.button(11, "Action");
                });
            },
        );
        amount
    }
    #[test]
    fn zoom_has_mui_durations_and_blocks_input_on_exit() {
        let mut ui = Ui::default();
        assert_eq!(sample(&mut ui, 0.0, false), 0.0);
        assert_eq!(sample(&mut ui, 1.0, true), 0.0);
        assert!(sample(&mut ui, 1.1125, true) > 0.5);
        assert_eq!(sample(&mut ui, 1.2251, true), 1.0);
        assert_eq!(sample(&mut ui, 2.0, false), 1.0);
        assert!(ui.semantics().iter().all(|node| !node.enabled));
        assert_eq!(sample(&mut ui, 2.196, false), 0.0);
    }
}

mod shape_rotation {
    use bhoo::*;
    const B: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    fn sample(ui: &mut Ui, state: &mut ShapeRotationState, time: f64, bounds: Rect) -> Option<f32> {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            B,
            0.0,
            |f| {
                f.rotating_shape(bounds, 1, Color(1, 2, 3, 255), 12.0, state);
            },
        )
        .iter()
        .find_map(|c| match c {
            DrawCommand::Shape { rotation, .. } => Some(*rotation),
            _ => None,
        })
    }
    #[test]
    fn shape_rotation_is_linear_pauses_offscreen_and_respects_reduced_motion() {
        let mut ui = Ui::default();
        let mut state = ShapeRotationState::default();
        assert_eq!(sample(&mut ui, &mut state, 0.0, B), Some(0.0));
        let angle = sample(&mut ui, &mut state, 3.0, B).unwrap();
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 0.0001);
        assert!(ui.needs_redraw());
        assert_eq!(
            sample(&mut ui, &mut state, 4.0, Rect { x: 200.0, ..B }),
            None
        );
        assert!(!ui.needs_redraw());
        let resumed = sample(&mut ui, &mut state, 100.0, B).unwrap();
        assert!((resumed - angle).abs() < 0.0001);
        ui.motion_preference = MotionPreference::Reduced;
        assert_eq!(sample(&mut ui, &mut state, 101.0, B), Some(0.0));
        assert!(!ui.needs_redraw());
    }

    #[test]
    fn rotated_shape_clip_contains_the_rotated_square() {
        let b = Rect {
            x: 10.0,
            y: 20.0,
            width: 56.0,
            height: 56.0,
        };
        assert_eq!(shape_clip_bounds(b, 0.0), b);
        for degrees in [15.0_f32, 30.0, 45.0, 60.0, 75.0] {
            let angle = degrees.to_radians();
            let clip = shape_clip_bounds(b, angle);
            let (sin, cos) = angle.sin_cos();
            for (x, y) in [(-28.0, -28.0), (28.0, -28.0), (28.0, 28.0), (-28.0, 28.0)] {
                let px = 38.0 + x * cos - y * sin;
                let py = 48.0 + x * sin + y * cos;
                assert!(px >= clip.x - 0.0001 && px <= clip.x + clip.width + 0.0001);
                assert!(py >= clip.y - 0.0001 && py <= clip.y + clip.height + 0.0001);
            }
        }
    }
}

mod wavy_shapes {
    use bhoo::*;
    const B: Rect = Rect {
        x: 10.0,
        y: 10.0,
        width: 44.0,
        height: 44.0,
    };
    const VIEW: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    fn draw(
        ui: &mut Ui,
        state: &mut ShapeRotationState,
        name: &str,
        t: f64,
        bounds: Rect,
    ) -> Vec<DrawCommand> {
        let index = bhoo::beercss_shapes::SHAPES
            .iter()
            .position(|s| s.name == name)
            .unwrap();
        ui.frame(
            Input {
                time: t,
                ..Input::default()
            },
            VIEW,
            0.0,
            |f| {
                f.animated_wavy_shape(bounds, index, Color(1, 2, 3, 255), state);
            },
        )
        .to_vec()
    }
    #[test]
    fn wavy_shapes_move_at_source_rates_and_pause() {
        let mut ui = Ui::default();
        let mut state = ShapeRotationState::default();
        let angle = |v: Vec<DrawCommand>| {
            v.iter().find_map(|c| {
                if let DrawCommand::Shape { rotation, .. } = c {
                    Some(*rotation)
                } else {
                    None
                }
            })
        };
        assert_eq!(
            angle(draw(&mut ui, &mut state, "wavy-circle", 0.0, B)),
            Some(0.0)
        );
        assert!(
            (angle(draw(&mut ui, &mut state, "wavy-circle", 2.0, B)).unwrap()
                + std::f32::consts::FRAC_PI_2)
                .abs()
                < 0.0001
        );
        assert_eq!(
            angle(draw(
                &mut ui,
                &mut state,
                "wavy-circle",
                3.0,
                Rect { x: 200.0, ..B }
            )),
            None
        );
        assert!(!ui.needs_redraw());
        let mut ui = Ui::default();
        let mut wave = ShapeRotationState::default();
        let x = |v: Vec<DrawCommand>| {
            v.iter()
                .find_map(|c| {
                    if let DrawCommand::Shape { bounds, .. } = c {
                        Some(bounds.x)
                    } else {
                        None
                    }
                })
                .unwrap()
        };
        let initial = x(draw(&mut ui, &mut wave, "wavy", 0.0, B));
        let quarter = x(draw(&mut ui, &mut wave, "wavy", 0.25, B));
        assert!((quarter - initial - 44.0 * (312.5 / 1220.0) * 0.25).abs() < 0.0001);
        ui.motion_preference = MotionPreference::Reduced;
        assert_eq!(x(draw(&mut ui, &mut wave, "wavy", 1.0, B)), initial);
        assert!(!ui.needs_redraw());
    }
}

mod directional_and_reveal_tests {
    use crate::*;

    #[test]
    fn slide_directions_circular_reveal_container_transform_and_route_shadow_work() {
        let mut ui = Ui::default();
        let bounds = Rect { x: 0.0, y: 0.0, width: 400.0, height: 300.0 };
        let panel = Rect { x: 100.0, y: 100.0, width: 100.0, height: 80.0 };
        let _ = ui.frame(Input { time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
            f.slide(1, false, panel, SlideDirection::Left, |_, _| {});
        });
        let _ = ui.frame(Input { time: 0.1, ..Input::default() }, bounds, 0.0, |f| {
            f.slide(1, true, panel, SlideDirection::Left, |_, _| {});
        });
        let mut posed = Rect::default();
        let _ = ui.frame(Input { time: 0.2, ..Input::default() }, bounds, 0.0, |f| {
            f.slide(1, true, panel, SlideDirection::Left, |_, r| posed = r);
        });
        assert!(posed.x > panel.x);

        let mut reveal_r = 0.0;
        let cmds = ui.frame(Input { time: 0.2, ..Input::default() }, bounds, 0.0, |f| {
            f.circular_reveal(bounds, [380.0, 20.0], 0.5, Color(20, 20, 20, 255), |_, r| reveal_r = r);
            f.container_transform(
                Rect { x: 20.0, y: 20.0, width: 48.0, height: 48.0 },
                bounds,
                24.0,
                0.0,
                Color(10, 10, 10, 255),
                Color(200, 200, 200, 255),
                0.5,
                |_, _| {},
                |_, _| {},
            );
            let st = RouteTransitionState::start(RouteTransitionKind::Push, 0.16);
            f.route_transition(bounds, st, |_| {}, |_| {});
        });
        assert!(reveal_r > 100.0);
        // Route push shadow emits semi-transparent black vertical strips
        assert!(cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: b, color, .. } if b.width == 12.0 && b.height == 300.0 && color.0 == 0 && color.3 > 0)));
    }
}
