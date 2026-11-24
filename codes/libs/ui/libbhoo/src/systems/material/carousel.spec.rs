use bhoo::*;
const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 400.0,
    height: 200.0,
};
const B: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 300.0,
    height: 120.0,
};
const ITEMS: [CarouselItem<'static>; 5] = [
    CarouselItem {
        label: "One",
        image: None,
        enabled: true,
    },
    CarouselItem {
        label: "Two",
        image: None,
        enabled: false,
    },
    CarouselItem {
        label: "Three",
        image: None,
        enabled: true,
    },
    CarouselItem {
        label: "Four",
        image: None,
        enabled: true,
    },
    CarouselItem {
        label: "Five",
        image: None,
        enabled: true,
    },
];
fn draw(
    ui: &mut Ui,
    state: &mut CarouselState,
    input: Input,
    options: CarouselOptions,
) -> Option<usize> {
    let mut result = None;
    ui.frame(input, VIEW, 0.0, |f| {
        result = f.carousel(100, B, &ITEMS, state, options)
    });
    result
}
fn key(key: Key, time: f64) -> Input {
    Input {
        key: Some(KeyInput {
            key,
            pressed: true,
            repeat: false,
            shift: false,
        }),
        time,
        ..Input::default()
    }
}
#[test]
fn drag_cancels_activation_snaps_and_stops_when_idle() {
    let mut ui = Ui::default();
    let mut state = CarouselState::default();
    let opts = CarouselOptions {
        item_width: 180.0,
        ..CarouselOptions::default()
    };
    draw(&mut ui, &mut state, Input::default(), opts);
    assert!(state.can_next());
    assert!(!state.can_previous());
    assert!(!ui.needs_redraw());
    draw(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([220.0, 50.0]),
            pointer_kind: PointerKind::Touch,
            pressed: true,
            time: 0.1,
            ..Input::default()
        },
        opts,
    );
    draw(
        &mut ui,
        &mut state,
        Input {
            pointer: Some([50.0, 50.0]),
            pointer_kind: PointerKind::Touch,
            time: 0.2,
            ..Input::default()
        },
        opts,
    );
    assert_eq!(state.index(), 1);
    assert_eq!(
        draw(
            &mut ui,
            &mut state,
            Input {
                pointer: Some([50.0, 50.0]),
                pointer_kind: PointerKind::Touch,
                released: true,
                time: 0.3,
                ..Input::default()
            },
            opts
        ),
        None
    );
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 0.31,
            ..Input::default()
        },
        opts,
    );
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 1.0,
            ..Input::default()
        },
        opts,
    );
    assert_eq!(state.index(), 1);
    assert!(!ui.needs_redraw());
}
#[test]
fn keyboard_skips_disabled_items_and_rtl_mirrors_layout() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = CarouselState::default();
    let opts = CarouselOptions {
        item_width: 180.0,
        rtl: true,
        ..CarouselOptions::default()
    };
    draw(&mut ui, &mut state, Input::default(), opts);
    let first = ui.semantics().iter().find(|n| n.id == 101).unwrap();
    assert_eq!(first.bounds.x, 120.0);
    draw(&mut ui, &mut state, key(Key::Tab, 0.1), opts);
    assert!(ui.semantics().iter().any(|n| n.id == 101 && n.focused));
    draw(&mut ui, &mut state, key(Key::Left, 0.2), opts);
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 0.3,
            ..Input::default()
        },
        opts,
    );
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 0.4,
            ..Input::default()
        },
        opts,
    );
    assert!(ui.semantics().iter().any(|n| n.id == 103 && n.focused));
}
#[test]
fn fullscreen_and_large_sets_only_build_visible_items() {
    let mut ui = Ui::default();
    let mut state = CarouselState::default();
    let items = vec![
        CarouselItem {
            label: "Item",
            image: None,
            enabled: true
        };
        1000
    ];
    ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.carousel(
            100,
            B,
            &items,
            &mut state,
            CarouselOptions {
                layout: CarouselLayout::FullScreen,
                ..CarouselOptions::default()
            },
        );
    });
    assert_eq!(
        ui.semantics()
            .iter()
            .filter(|n| n.role == Role::CarouselItem && n.enabled)
            .count(),
        1
    );
    assert!(ui.semantics().len() < 6);
    assert!(state.can_next());
}
#[test]
fn resize_during_snap_keeps_target_visible_and_motion_settles() {
    let mut ui = Ui::default();
    let mut state = CarouselState::default();
    let opts = CarouselOptions {
        item_width: 180.0,
        ..CarouselOptions::default()
    };
    draw(&mut ui, &mut state, Input::default(), opts);
    state.go_to(2);
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 0.1,
            ..Input::default()
        },
        opts,
    );
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 0.2,
            ..Input::default()
        },
        opts,
    );
    let narrower = Rect { width: 180.0, ..B };
    for time in [0.21, 0.4, 0.6] {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            VIEW,
            0.0,
            |f| {
                f.carousel(100, narrower, &ITEMS, &mut state, opts);
            },
        );
    }
    assert_eq!(state.index(), 2);
    let target = ui
        .semantics()
        .iter()
        .find(|n| n.id == 103 && n.enabled)
        .unwrap();
    assert!(target.bounds.width > 0.0);
    assert!(
        target.bounds.x >= narrower.x
            && target.bounds.x + target.bounds.width <= narrower.x + narrower.width + 0.001
    );
    assert!(!ui.needs_redraw());
}

#[test]
fn centered_hero_exposes_neighbors_snaps_and_mirrors_rtl() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = CarouselState::default();
    let opts = CarouselOptions {
        layout: CarouselLayout::CenteredHero,
        ..CarouselOptions::default()
    };
    draw(&mut ui, &mut state, Input::default(), opts);
    state.go_to(2);
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 1.,
            ..Input::default()
        },
        opts,
    );
    let find = |ui: &Ui, id| ui.semantics().iter().find(|n| n.id == id).unwrap().bounds;
    let left = find(&ui, 102);
    let center = find(&ui, 103);
    let right = find(&ui, 104);
    assert!((center.x + center.width / 2. - (B.x + B.width / 2.)).abs() < 0.01);
    assert!(center.width > left.width && center.width > right.width);
    assert!(left.x + left.width <= center.x && center.x + center.width <= right.x);
    assert_eq!(state.index(), 2);
    let rtl = CarouselOptions { rtl: true, ..opts };
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 2.,
            ..Input::default()
        },
        rtl,
    );
    let mirrored = find(&ui, 102);
    assert!((mirrored.x - (B.x + B.width - (left.x - B.x) - left.width)).abs() < 0.01);
    state.go_to(4);
    draw(
        &mut ui,
        &mut state,
        Input {
            time: 3.,
            ..Input::default()
        },
        rtl,
    );
    assert_eq!(state.index(), 4);
    assert!(!state.can_next());
}

#[test]
fn hero_items_remain_ordered_and_separated_during_drag() {
    for rtl in [false, true] {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut state = CarouselState::default();
        let opts = CarouselOptions {
            layout: CarouselLayout::CenteredHero,
            rtl,
            ..CarouselOptions::default()
        };
        draw(&mut ui, &mut state, Input::default(), opts);
        state.go_to(2);
        draw(
            &mut ui,
            &mut state,
            Input {
                time: 1.,
                ..Input::default()
            },
            opts,
        );
        let origin = 150.;
        draw(
            &mut ui,
            &mut state,
            Input {
                time: 1.1,
                pointer: Some([origin, 60.]),
                pressed: true,
                ..Input::default()
            },
            opts,
        );
        let step = 192.; // 300px viewport, 50px neighbors, 8px spacing.
        for part in 1..10 {
            let delta = step * part as f32 / 10. * if rtl { 1. } else { -1. };
            draw(
                &mut ui,
                &mut state,
                Input {
                    time: 1.1 + part as f64 * 0.01,
                    pointer: Some([origin + delta, 60.]),
                    ..Input::default()
                },
                opts,
            );
            let mut items: Vec<_> = ui
                .semantics()
                .iter()
                .filter(|n| (101..=105).contains(&n.id) && n.bounds.width > 0.)
                .map(|n| n.bounds)
                .collect();
            items.sort_by(|a, b| a.x.total_cmp(&b.x));
            assert!(
                items
                    .windows(2)
                    .all(|pair| pair[0].x + pair[0].width <= pair[1].x + 0.01),
                "overlap at {part}/10, rtl={rtl}: {items:?}"
            );
        }
    }
}

#[test]
fn multi_browse_arranges_masks_and_reverses_edges_without_overlap() {
    let area = Rect { width: 400., ..B };
    for rtl in [false, true] {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut state = CarouselState::default();
        let opts = CarouselOptions {
            layout: CarouselLayout::MultiBrowse,
            rtl,
            ..CarouselOptions::default()
        };
        let render = |ui: &mut Ui, state: &mut CarouselState, input: Input| {
            ui.frame(input, VIEW, 0., |f| {
                f.carousel(100, area, &ITEMS, state, opts);
            });
        };
        render(&mut ui, &mut state, Input::default());
        let b = |ui: &Ui, id| ui.semantics().iter().find(|n| n.id == id).unwrap().bounds;
        assert!(b(&ui, 101).width > b(&ui, 102).width);
        assert!(b(&ui, 102).width > b(&ui, 103).width);
        state.go_to(4);
        render(
            &mut ui,
            &mut state,
            Input {
                time: 1.,
                ..Input::default()
            },
        );
        assert_eq!(state.index(), 4);
        let last = b(&ui, 105);
        assert!(
            (if rtl {
                last.x - area.x
            } else {
                area.x + area.width - last.x - last.width
            })
            .abs()
                < 0.01
        );
        assert!(!state.can_next());
        state.go_to(2);
        render(
            &mut ui,
            &mut state,
            Input {
                time: 2.,
                ..Input::default()
            },
        );
        render(
            &mut ui,
            &mut state,
            Input {
                time: 2.1,
                pointer: Some([200., 60.]),
                pressed: true,
                ..Input::default()
            },
        );
        for part in 1..10 {
            let delta = 228. * part as f32 / 10. * if rtl { 1. } else { -1. };
            render(
                &mut ui,
                &mut state,
                Input {
                    time: 2.1 + part as f64 * 0.01,
                    pointer: Some([200. + delta, 60.]),
                    ..Input::default()
                },
            );
            let mut cells: Vec<_> = ui
                .semantics()
                .iter()
                .filter(|n| (101..=105).contains(&n.id) && n.bounds.width > 0.)
                .map(|n| n.bounds)
                .collect();
            cells.sort_by(|a, b| a.x.total_cmp(&b.x));
            assert!(
                cells
                    .windows(2)
                    .all(|p| p[0].x + p[0].width <= p[1].x + 0.01),
                "overlap at {part} rtl={rtl}: {cells:?}"
            );
        }
    }
}

#[test]
fn masked_carousel_images_keep_large_item_scale() {
    let image = Image::rgba(200, 100, vec![255; 200 * 100 * 4]).unwrap();
    let items = [CarouselItem {
        label: "One",
        image: Some(&image),
        enabled: true,
    }; 3];
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = CarouselState::default();
    let options = CarouselOptions {
        layout: CarouselLayout::MultiBrowse,
        ..CarouselOptions::default()
    };
    let area = Rect { width: 400., ..B };
    let commands = ui.frame(Input::default(), VIEW, 0., |f| {
        f.carousel(100, area, &items, &mut state, options);
    });
    let images: Vec<_> = commands
        .iter()
        .filter_map(|cmd| match cmd {
            DrawCommand::Image { bounds, uv, .. } => Some((*bounds, *uv)),
            _ => None,
        })
        .collect();
    assert_eq!(images.len(), 3);
    let (large, large_uv) = images[0];
    for (mask, uv) in &images[1..] {
        assert!((uv[1] - large_uv[1]).abs() < 0.0001);
        assert!((uv[3] - large_uv[3]).abs() < 0.0001);
        let fraction = (uv[2] - uv[0]) / (large_uv[2] - large_uv[0]);
        assert!((fraction - mask.width / large.width).abs() < 0.0001);
        assert!(((uv[0] + uv[2]) - (large_uv[0] + large_uv[2])).abs() < 0.0001);
    }
}

#[test]
fn multi_browse_wide_viewport_fits_multiple_large_items() {
    let area = Rect {
        x: 0.,
        y: 0.,
        width: 800.,
        height: 160.,
    };
    let items = [CarouselItem {
        label: "Item",
        image: None,
        enabled: true,
    }; 8];
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = CarouselState::default();
    let options = CarouselOptions {
        layout: CarouselLayout::MultiBrowse,
        item_width: 220.,
        ..CarouselOptions::default()
    };
    let render = |ui: &mut Ui, state: &mut CarouselState, time| {
        ui.frame(
            Input {
                time,
                ..Input::default()
            },
            area,
            0.,
            |f| {
                f.carousel(100, area, &items, state, options);
            },
        );
    };
    render(&mut ui, &mut state, 0.);
    let bounds = |ui: &Ui, id| ui.semantics().iter().find(|n| n.id == id).unwrap().bounds;
    let large = bounds(&ui, 101);
    assert!((bounds(&ui, 102).width - large.width).abs() < 0.01);
    assert!((bounds(&ui, 103).width - large.width).abs() < 0.01);
    assert!(bounds(&ui, 104).width < large.width);
    assert!((bounds(&ui, 104).x + bounds(&ui, 104).width - area.width).abs() < 0.01);
    state.go_to(7);
    render(&mut ui, &mut state, 1.);
    assert_eq!(state.index(), 7);
    assert!((bounds(&ui, 108).x + bounds(&ui, 108).width - area.width).abs() < 0.01);
    assert!((bounds(&ui, 107).width - bounds(&ui, 108).width).abs() < 0.01);
    state.go_to(6);
    render(&mut ui, &mut state, 2.);
    ui.frame(
        Input {
            time: 2.1,
            pointer: Some([400., 80.]),
            pressed: true,
            ..Input::default()
        },
        area,
        0.,
        |f| {
            f.carousel(100, area, &items, &mut state, options);
        },
    );
    for part in 1..10 {
        ui.frame(
            Input {
                time: 2.1 + part as f64 * 0.01,
                pointer: Some([400. - 248. * part as f32 / 10., 80.]),
                ..Input::default()
            },
            area,
            0.,
            |f| {
                f.carousel(100, area, &items, &mut state, options);
            },
        );
        let mut cells: Vec<_> = ui
            .semantics()
            .iter()
            .filter(|n| (101..=108).contains(&n.id) && n.bounds.width > 0.)
            .map(|n| n.bounds)
            .collect();
        cells.sort_by(|a, b| a.x.total_cmp(&b.x));
        assert!(
            cells
                .windows(2)
                .all(|p| p[0].x + p[0].width <= p[1].x + 0.01),
            "overlap at {part}: {cells:?}"
        );
    }
}

#[test]
fn home_and_end_navigate_and_focus_first_and_last_enabled_items() {
    for rtl in [false, true] {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut state = CarouselState::default();
        let opts = CarouselOptions {
            item_width: 180.,
            rtl,
            ..CarouselOptions::default()
        };
        draw(&mut ui, &mut state, Input::default(), opts);
        draw(&mut ui, &mut state, key(Key::Tab, 0.1), opts);
        draw(&mut ui, &mut state, key(Key::End, 0.2), opts);
        draw(
            &mut ui,
            &mut state,
            Input {
                time: 0.3,
                ..Input::default()
            },
            opts,
        );
        draw(
            &mut ui,
            &mut state,
            Input {
                time: 0.4,
                ..Input::default()
            },
            opts,
        );
        assert!(ui.semantics().iter().any(|n| n.id == 105 && n.focused));
        assert!(
            !ui.semantics().iter().any(|n| n.id == 101),
            "fully hidden item must not be exposed"
        );
        draw(&mut ui, &mut state, key(Key::Home, 0.5), opts);
        draw(
            &mut ui,
            &mut state,
            Input {
                time: 0.6,
                ..Input::default()
            },
            opts,
        );
        draw(
            &mut ui,
            &mut state,
            Input {
                time: 0.7,
                ..Input::default()
            },
            opts,
        );
        assert!(ui.semantics().iter().any(|n| n.id == 101 && n.focused));
    }
}

#[test]
fn carousel_semantics_include_collection_and_visible_item_positions() {
    let mut ui = Ui::default();
    let mut state = CarouselState::default();
    let options = CarouselOptions {
        item_width: 180.,
        ..CarouselOptions::default()
    };
    draw(&mut ui, &mut state, Input::default(), options);
    let group = ui
        .semantics()
        .iter()
        .find(|n| n.role == Role::Carousel)
        .unwrap();
    assert_eq!(group.value, Some(1.));
    assert_eq!(group.range, Some([1., 5.]));
    let first = ui.semantics().iter().find(|n| n.id == 101).unwrap();
    assert_eq!(first.role, Role::CarouselItem);
    assert_eq!(first.value, Some(1.));
    assert_eq!(first.range, Some([1., 5.]));
    let second = ui.semantics().iter().find(|n| n.id == 102).unwrap();
    assert!(!second.enabled);
    assert_eq!(second.value, Some(2.));
    assert!(!ui.semantics().iter().any(|n| n.id == 105));
}

#[test]
fn accessible_focus_paints_keyboard_state_and_rejects_hidden_items() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = CarouselState::default();
    let options = CarouselOptions {
        item_width: 180.,
        ..CarouselOptions::default()
    };
    draw(&mut ui, &mut state, Input::default(), options);
    ui.accessibility_focus(Some(101));
    let commands = ui.frame(
        Input {
            time: 1.,
            ..Input::default()
        },
        VIEW,
        0.,
        |f| {
            f.carousel(100, B, &ITEMS, &mut state, options);
        },
    );
    let focused = ui.semantics().iter().find(|n| n.id == 101).unwrap();
    assert!(focused.focused);
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color: Color(_, _, _, 31), .. } if *bounds == focused.bounds)), "expected visible keyboard state layer");
    ui.accessibility_focus(Some(105));
    assert_eq!(ui.focused(), None, "fully hidden item cannot take focus");
}

#[test]
fn carousel_region_uses_caller_supplied_accessible_name() {
    let mut ui = Ui::default();
    let mut state = CarouselState::default();
    ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.carousel_named(100, B, &ITEMS, &mut state, CarouselOptions::default(), "Featured stories");
    });
    let region = ui.semantics().iter().find(|node| node.role == Role::Carousel).unwrap();
    assert_eq!(&*region.label, "Featured stories");
}
