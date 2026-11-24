use super::*;

fn step(
    ui: &mut Ui,
    demo: &mut LauncherDemo,
    time: f64,
    pos: Option<[f32; 2]>,
    pressed: bool,
    released: bool,
    kind: PointerKind,
) {
    let mut gmail = String::new();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 411.0,
        height: 846.0,
    };
    let input = Input {
        pointer: pos,
        pointer_kind: kind,
        pressed,
        released,
        time,
        ..Input::default()
    };
    ui.frame(input, bounds, 0.0, |f| {
        demo.show(f, 411.0, 846.0, input, &mut gmail);
    });
}

#[test]
fn slow_mo_notification_bar_scrub_and_collapse() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    // Prime initial frame
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Mouse);
    assert_eq!(demo.shade_stage, 0);

    // Press top status bar and slowly pull down ("slow mo") through Stage 1 -> Stage 2!
    step(&mut ui, &mut demo, 0.05, Some([200.0, 12.0]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 0.15, Some([200.0, 145.0]), false, false, PointerKind::Mouse);
    let mid_s1 = demo.shade_anim.value;
    assert!(
        mid_s1 > 0.30 && mid_s1 < 0.70,
        "expected continuous slow-mo Stage 1 fraction, got {mid_s1}"
    );

    // Keep dragging slowly past Stage 1 into Stage 2 morph (~1.45)!
    step(&mut ui, &mut demo, 0.35, Some([200.0, 440.0]), false, false, PointerKind::Mouse);
    let mid_s2 = demo.shade_anim.value;
    assert!(
        mid_s2 > 1.25 && mid_s2 < 1.85,
        "expected continuous slow-mo Stage 1->2 morph fraction, got {mid_s2}"
    );

    // Drag BACK UP in slow-mo while still holding finger down!
    step(&mut ui, &mut demo, 0.55, Some([200.0, 260.0]), false, false, PointerKind::Mouse);
    assert!(
        demo.shade_anim.value < mid_s2 - 0.3,
        "expected dragging back up to reverse slow-mo morph smoothly"
    );

    // Release at ~0.85 -> settles to Stage 1
    step(&mut ui, &mut demo, 0.75, Some([200.0, 260.0]), false, true, PointerKind::Mouse);
    assert_eq!(demo.shade_stage, 1);

    // Wait for settle
    for i in 1..=25 {
        step(&mut ui, &mut demo, 0.75 + i as f64 * 0.02, None, false, false, PointerKind::Mouse);
    }
    assert!((demo.shade_anim.value - 1.0).abs() < 0.02);

    // Drag down from ANYWHERE on open Stage 1 Shade body -> expands smoothly to Stage 2!
    step(&mut ui, &mut demo, 1.40, Some([200.0, 110.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 1.44, Some([200.0, 260.0]), false, false, PointerKind::Touch);
    assert!(demo.shade_anim.value > 1.35);
    step(&mut ui, &mut demo, 1.48, Some([200.0, 380.0]), false, true, PointerKind::Touch);
    assert_eq!(demo.shade_stage, 2);

    // Settle to Stage 2
    for i in 1..=25 {
        step(&mut ui, &mut demo, 1.48 + i as f64 * 0.02, None, false, false, PointerKind::Touch);
    }
    assert!((demo.shade_anim.value - 2.0).abs() < 0.02);

    // Drag UP from open Stage 2 body -> smoothly retreats toward Stage 1!
    step(&mut ui, &mut demo, 2.20, Some([200.0, 620.0]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 2.35, Some([200.0, 420.0]), false, false, PointerKind::Mouse);
    assert!(demo.shade_anim.value > 1.0 && demo.shade_anim.value < 1.6);
    step(&mut ui, &mut demo, 2.55, Some([200.0, 350.0]), false, true, PointerKind::Mouse);
    assert_eq!(demo.shade_stage, 1);
}

#[test]
fn overview_drag_right_moves_older_cards_into_center_and_swipe_up_dismisses() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.set_capture_preset("overview");
    let initial_tasks = demo.tasks.len();
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Mouse);
    assert_eq!(demo.overview_scroll, 0.0);

    // Drag RIGHT (x: 110 -> 340) -> overview_scroll MUST INCREASE so Clock on the LEFT slides into center!
    step(&mut ui, &mut demo, 0.05, Some([110.0, 420.0]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 0.12, Some([220.0, 420.0]), false, false, PointerKind::Mouse);
    assert!(
        demo.overview_scroll > 80.0,
        "dragging RIGHT must increase overview_scroll, got {}",
        demo.overview_scroll
    );
    step(&mut ui, &mut demo, 0.20, Some([340.0, 420.0]), false, true, PointerKind::Mouse);
    for i in 1..=25 {
        step(&mut ui, &mut demo, 0.20 + i as f64 * 0.02, None, false, false, PointerKind::Mouse);
    }
    assert!(
        (demo.overview_scroll - demo.last_overview_step).abs() < 4.0,
        "expected snap to card 1 (~{}), got {}",
        demo.last_overview_step,
        demo.overview_scroll
    );

    // Vertical swipe UP on centered card dismisses it!
    step(&mut ui, &mut demo, 1.00, Some([205.0, 450.0]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 1.05, Some([205.0, 360.0]), false, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 1.10, Some([205.0, 220.0]), false, true, PointerKind::Mouse);
    assert_eq!(demo.tasks.len(), initial_tasks - 1);
}

#[test]
fn app_drawer_continuous_swipe_up_from_dock_and_swipe_down_close() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Mouse);

    // Swipe UP starting directly on Hotseat dock icon (y=775) -> scrubs drawer 1:1 without launching icon!
    step(&mut ui, &mut demo, 0.05, Some([205.0, 775.0]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 0.15, Some([205.0, 560.0]), false, false, PointerKind::Mouse);
    assert!(
        demo.drawer_anim.value > 0.35 && demo.drawer_anim.value < 0.75,
        "expected continuous drawer scrub, got {}",
        demo.drawer_anim.value
    );
    step(&mut ui, &mut demo, 0.20, Some([205.0, 460.0]), false, true, PointerKind::Mouse);
    assert!(demo.drawer_open);
    assert_eq!(demo.mode, ShellMode::Home); // Hotseat Contacts icon MUST NOT accidentally launch!

    for i in 1..=25 {
        step(&mut ui, &mut demo, 0.20 + i as f64 * 0.02, None, false, false, PointerKind::Mouse);
    }
    assert!((demo.drawer_anim.value - 1.0).abs() < 0.02);

    // Swipe DOWN from App Drawer -> scrubs back down & closes!
    step(&mut ui, &mut demo, 1.00, Some([205.0, 120.0]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 1.08, Some([205.0, 420.0]), false, true, PointerKind::Mouse);
    assert!(!demo.drawer_open);
}

#[test]
fn password_unlock_reveals_home_only_after_authentication() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.sys.anim_scale_idx = SystemState::ANIMATION_SCALES.iter().position(|&scale| scale == 1.0).unwrap();
    demo.set_capture_preset("lock_password");
    demo.sys.lock_secret = "test credential".into();
    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    let mut gmail = String::new();
    let enter = KeyInput { key: Key::Enter, pressed: true, repeat: false, shift: false };

    demo.lockscreen.entry = "incorrect".into();
    let input = Input { time: 0.1, key: Some(enter), ..Input::default() };
    ui.frame(input, bounds, 0.0, |f| {
        demo.show(f, bounds.width, bounds.height, input, &mut gmail);
    });
    assert!(demo.lockscreen.locked);
    assert!(!demo.lockscreen.is_unlocking());

    demo.lockscreen.entry = demo.sys.lock_secret.clone();
    demo.lockscreen.show_password = true;
    let input = Input { time: 0.2, ..input };
    let commands = ui.frame(input, bounds, 0.0, |f| {
        demo.show(f, bounds.width, bounds.height, input, &mut gmail);
    });
    assert!(!demo.lockscreen.locked);
    assert!(demo.lockscreen.is_unlocking());
    assert_eq!(demo.lockscreen.home_reveal_alpha(), 0.0);
    assert!(demo.lockscreen.entry.is_empty());
    assert!(!demo.lockscreen.show_password);
    assert!(!commands.iter().any(|command| matches!(command,
        DrawCommand::Text { text, .. } if &**text == "test credential")));

    step(&mut ui, &mut demo, 0.42, None, false, false, PointerKind::Touch);
    assert!(demo.lockscreen.home_reveal_alpha() > 0.0);
    step(&mut ui, &mut demo, 0.55, None, false, false, PointerKind::Touch);
    assert!(!demo.lockscreen.is_unlocking());
    assert_eq!(demo.lockscreen.home_reveal_alpha(), 1.0);
}

#[test]
fn fullscreen_wayland_shade_follows_the_swipe_over_the_app() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.set_active_wayland_app(Some(AppId::Clock));
    demo.wayland_surface_ready = true;
    let touch = PointerKind::Touch;
    step(&mut ui, &mut demo, 0.0, Some([200.0, 8.0]), true, false, touch);

    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    let input = Input {
        time: 0.1,
        pointer: Some([200.0, 160.0]),
        pointer_kind: touch,
        ..Input::default()
    };
    let commands = ui.frame(input, bounds, 0.0, |f| {
        demo.show(f, bounds.width, bounds.height, input, &mut String::new());
    });
    assert!(matches!(demo.gesture, ActiveGesture::ShadeScrub { .. }));
    assert!(demo.shade_anim.value > 0.0 && demo.shade_anim.value < 1.0);
    assert!(demo.shows_wayland_surface());
    assert!(!demo.is_wayland_passthrough());
    assert!(commands.iter().any(|command| matches!(command,
        DrawCommand::Clear(Color(0, 0, 0, 0)))));

    let saved_frame = Image::rgba(2, 4, vec![255; 32]).unwrap();
    demo.wayland_shade_frame = Some(saved_frame.clone());
    let input = Input { time: 0.2, ..input };
    let commands = ui.frame(input, bounds, 0.0, |f| {
        demo.show(f, bounds.width, bounds.height, input, &mut String::new());
    });
    assert!(demo.shows_wayland_surface());
    assert!(!demo.composites_wayland_surface());
    assert!(commands.iter().any(|command| matches!(command,
        DrawCommand::Image { image, .. } if image.width() == 2 && image.height() == 4)));

    step(&mut ui, &mut demo, 0.4, Some([200.0, 160.0]), false, true, touch);
    assert_eq!(demo.shade_stage, 1);
    assert_eq!(demo.mode, ShellMode::Fullscreen(AppId::Clock));
    assert!(demo.shows_wayland_surface());
    demo.go_back(1.0);
    step(&mut ui, &mut demo, 4.0, None, false, false, touch);
    assert!(demo.is_wayland_passthrough());
    assert!(demo.composites_wayland_surface());
}

#[test]
fn wayland_activation_preserves_the_icon_launch_until_the_surface_is_ready() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.sys.anim_scale_idx = SystemState::ANIMATION_SCALES.iter().position(|&scale| scale == 1.0).unwrap();
    let id = "org.example.Window.desktop";
    let desktop = freedesktop::parse_desktop_entry(
        id,
        std::path::Path::new("/tmp/org.example.Window.desktop"),
        "[Desktop Entry]\nType=Application\nName=Window\nExec=window\n",
        "",
        &["Wisp".into()],
    ).expect("valid desktop entry");
    demo.desktop_registry.apps.push(desktop);
    let icon = Rect { x: 16.0, y: 100.0, width: 48.0, height: 48.0 };

    demo.open_desktop_app_from(0, None, Some(icon), false, 0.0);
    demo.set_active_wayland_desktop(Some(id));
    demo.wayland_surface_ready = true;
    assert!(matches!(demo.app_transition, AppTransition::OpeningDesktop {
        origin, from_icon: true, .. } if origin == icon));
    assert!(!demo.is_wayland_passthrough());
    step(&mut ui, &mut demo, 0.2, None, false, false, PointerKind::Touch);
    assert!(!demo.is_wayland_passthrough());
    step(&mut ui, &mut demo, 0.4, None, false, false, PointerKind::Touch);
    assert!(demo.is_wayland_passthrough());

    demo.wayland_task_frames.insert(id.to_owned(), Image::rgba(1, 1, vec![1, 2, 3, 255]).unwrap());
    demo.open_desktop_app(0, 1.0);
    assert!(demo.wayland_launch_frame.is_some());
    demo.set_active_wayland_desktop(Some(id));
    step(&mut ui, &mut demo, 1.4, None, false, false, PointerKind::Touch);
    assert!(matches!(demo.app_transition, AppTransition::None));
    assert!(!demo.is_wayland_passthrough());
    demo.wayland_surface_ready = true;
    assert!(demo.is_wayland_passthrough());

    demo.sys.nav_mode = NavMode::Gestural;
    let touch = PointerKind::Touch;
    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    let input = Input { time: 1.5, ..Input::default() };
    let commands = ui.frame(input, bounds, 0.0, |f| {
        demo.show(f, bounds.width, bounds.height, input, &mut String::new());
    });
    assert!(commands.iter().any(|command| matches!(command,
        DrawCommand::RoundedRect { bounds, .. } if bounds.width == 108.0 && bounds.height == 4.0)));

    step(&mut ui, &mut demo, 1.6, Some([200.0, 835.0]), true, false, touch);
    step(&mut ui, &mut demo, 1.7, Some([200.0, 620.0]), false, false, touch);
    step(&mut ui, &mut demo, 2.0, Some([200.0, 620.0]), false, true, touch);
    assert_eq!(demo.mode, ShellMode::Overview);
}

#[test]
fn tapping_recents_background_slides_the_card_away_to_home() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.sys.anim_scale_idx = SystemState::ANIMATION_SCALES.iter().position(|&scale| scale == 1.0).unwrap();
    demo.tasks = vec![TaskEntry::Single(AppId::Settings)];
    demo.mode = ShellMode::Overview;
    demo.overview_anim.snap(1.0);
    let touch = PointerKind::Touch;

    step(&mut ui, &mut demo, 0.0, None, false, false, touch);
    let initial_card = demo.last_overview_cards[0].1;
    step(&mut ui, &mut demo, 0.1, Some([34.0, 260.0]), true, false, touch);
    step(&mut ui, &mut demo, 0.15, Some([34.0, 260.0]), false, true, touch);
    assert_eq!(demo.mode, ShellMode::Home);
    assert!(demo.overview_exiting);
    assert_eq!(demo.tasks, vec![TaskEntry::Single(AppId::Settings)]);

    step(&mut ui, &mut demo, 0.20, None, false, false, touch);
    let moving_card = demo.last_overview_cards[0].1;
    assert!(moving_card.x < initial_card.x);
    assert_eq!(moving_card.y, initial_card.y);
    assert_eq!(moving_card.width, initial_card.width);
    assert_eq!(moving_card.height, initial_card.height);

    step(&mut ui, &mut demo, 0.45, None, false, false, touch);
    assert!(!demo.overview_exiting);
    assert!(demo.last_overview_cards.is_empty());

    demo.open_overview(0.5);
    step(&mut ui, &mut demo, 0.9, None, false, false, touch);
    step(&mut ui, &mut demo, 1.0, Some([200.0, 260.0]), true, false, touch);
    step(&mut ui, &mut demo, 1.05, Some([200.0, 260.0]), false, true, touch);
    assert_eq!(demo.mode, ShellMode::Fullscreen(AppId::Settings));
    assert!(!demo.overview_exiting);
}

#[test]
fn home_bottom_swipe_previews_and_opens_empty_recents() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.tasks.clear();
    demo.sys.nav_mode = NavMode::Gestural;
    let touch = PointerKind::Touch;

    step(&mut ui, &mut demo, 0.0, Some([200.0, 836.0]), true, false, touch);
    step(&mut ui, &mut demo, 0.1, Some([200.0, 665.0]), false, false, touch);
    let preview = demo.overview_anim.value;
    assert!(preview > 0.0 && preview < 1.0);
    assert_eq!(demo.mode, ShellMode::Home);
    assert_eq!(demo.drawer_anim.value, 0.0);

    step(&mut ui, &mut demo, 0.4, Some([200.0, 665.0]), false, true, touch);
    assert_eq!(demo.mode, ShellMode::Overview);
    assert_eq!(demo.overview_anim.value, preview);
    assert_eq!(demo.overview_anim.target, 1.0);
    step(&mut ui, &mut demo, 4.0, None, false, false, touch);
    assert_eq!(demo.overview_anim.value, 1.0);

    demo.go_home(4.1);
    step(&mut ui, &mut demo, 4.1, None, false, false, touch);
    step(&mut ui, &mut demo, 7.0, None, false, false, touch);
    assert_eq!(demo.mode, ShellMode::Home);
    assert_eq!(demo.overview_anim.value, 0.0);
}

#[test]
fn compact_quick_settings_rows_shrink_without_overlapping() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.set_capture_preset("shade");
    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    let input = Input::default();
    let mut gmail = String::new();
    let mut rows = |stage| {
        demo.shade_anim.snap(stage);
        ui.frame(input, bounds, 0.0, |f| {
            demo.show(f, bounds.width, bounds.height, input, &mut gmail);
        }).into_iter().filter_map(|command| match command {
            DrawCommand::RoundedRect { bounds: tile, radius, .. }
                if tile.x == 16.0 && tile.width == (bounds.width - 40.0) * 0.5 && radius > 0.0 =>
            {
                Some(tile)
            }
            _ => None,
        }).collect::<Vec<_>>()
    };
    let full = rows(1.0);
    let mid = rows(0.65);
    assert_eq!(full.len(), 2);
    assert_eq!(mid.len(), 2);
    assert!(mid[0].height < full[0].height && mid[1].height < full[1].height);
    assert_eq!(mid[0].height, mid[1].height);
    assert!(mid[0].y + mid[0].height < mid[1].y);

    let late = rows(0.42);
    assert_eq!(late.len(), 1);
    assert_eq!(late[0].height, QS_MIN_TILE_HEIGHT);
    assert!(rows(0.28).is_empty());
}

#[test]
fn first_notification_pull_moves_the_rounded_edge_with_the_finger() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    let backdrop = demo.sys.palette().qs_backdrop;
    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    let mut gmail = String::new();
    let touch = PointerKind::Touch;
    step(&mut ui, &mut demo, 0.0, Some([200.0, 8.0]), true, false, touch);

    let mut edge = |time, y| {
        let input = Input {
            time,
            pointer: Some([200.0, y]),
            pointer_kind: touch,
            ..Input::default()
        };
        let commands = ui.frame(input, bounds, 0.0, |f| {
            demo.show(f, bounds.width, bounds.height, input, &mut gmail);
        });
        let cap = commands.iter().find_map(|command| match command {
            DrawCommand::RoundedRect { bounds: rect, color, .. }
                if rect.y == bounds.y && rect.width == bounds.width && *color == backdrop =>
            {
                Some(rect.height)
            }
            _ => None,
        }).expect("opaque top panel");
        let tint = commands.iter().find_map(|command| match command {
            DrawCommand::RoundedRect { bounds: rect, color, .. }
                if rect.y == cap && rect.width == bounds.width => Some(color.3),
            _ => None,
        }).expect("tint only below the panel");
        assert!(commands.iter().any(|command| matches!(command,
            DrawCommand::Image { bounds, .. } if bounds.x == 0.0 && bounds.y == cap)));
        (cap, tint)
    };
    let (early_edge, early_tint) = edge(0.1, 42.0);
    let (later_edge, later_tint) = edge(0.2, 105.0);
    let (returned_edge, returned_tint) = edge(0.3, 42.0);
    assert!(later_edge > early_edge && later_tint > early_tint && later_tint < 255);
    assert_eq!((early_edge, early_tint), (returned_edge, returned_tint));
}

#[test]
fn opening_notification_shade_bounces_and_settles_the_sheet() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.sys.anim_scale_idx = SystemState::ANIMATION_SCALES.iter().position(|&scale| scale == 1.0).unwrap();
    // The drag can already have reached the compact position when it releases.
    demo.shade_anim.snap(1.0);
    demo.settle_shade(1, 0.0);
    let touch = PointerKind::Touch;

    step(&mut ui, &mut demo, 0.0, None, false, false, touch);
    let rest_y = demo.last_notif_container.y;
    assert!(rest_y > 0.0);
    step(&mut ui, &mut demo, 0.08, None, false, false, touch);
    assert!((demo.last_notif_container.y - rest_y - 4.0).abs() < 0.01);
    step(&mut ui, &mut demo, 0.16, None, false, false, touch);
    assert_eq!(demo.last_notif_container.y, rest_y);
    assert_eq!(demo.shade_anim.value, 1.0);
}

#[test]
fn brightness_controls_fade_until_compact_shade() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.set_capture_preset("qs");
    demo.sys.auto_brightness = true;
    let p = demo.sys.palette();
    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    let input = Input::default();
    let mut gmail = String::new();
    let mut paint_opacities = |stage| {
        demo.shade_anim.snap(stage);
        let commands = ui.frame(input, bounds, 0.0, |f| {
            demo.show(f, bounds.width, bounds.height, input, &mut gmail);
        });
        let mut fill = 0;
        let mut auto = 0;
        for command in commands {
            if let DrawCommand::RoundedRect { bounds, color, .. } = command
                && bounds.y < 100.0 && bounds.height < 12.0
            {
                if bounds.x == 16.0 && color == Color(p.primary.0, p.primary.1, p.primary.2, color.3) {
                    fill = color.3;
                }
                if bounds.x > 350.0 && color == Color(p.primary_container.0, p.primary_container.1, p.primary_container.2, color.3) {
                    auto = color.3;
                }
            }
        }
        (fill, auto)
    };
    let (fill_early, auto_early) = paint_opacities(1.40);
    let (fill_late, auto_late) = paint_opacities(1.32);
    assert!(0 < fill_late && fill_late < fill_early);
    assert!(0 < auto_late && auto_late < auto_early);
    assert_eq!(paint_opacities(1.0), (0, 0));
}

#[test]
fn notification_sheet_tracks_upward_swipe_distance() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.set_capture_preset("qs");
    let touch = PointerKind::Touch;

    step(&mut ui, &mut demo, 0.0, None, false, false, touch);
    step(&mut ui, &mut demo, 0.1, Some([200.0, 790.0]), true, false, touch);
    step(&mut ui, &mut demo, 0.2, Some([200.0, 400.0]), false, false, touch);
    let sheet_y = demo.last_notif_container.y;
    assert!(demo.last_notif_container.width > 0.0);

    step(&mut ui, &mut demo, 0.3, Some([200.0, 330.0]), false, false, touch);
    assert!((sheet_y - demo.last_notif_container.y - 70.0).abs() < 0.01);

    step(&mut ui, &mut demo, 0.4, Some([200.0, 292.0]), false, false, touch);
    assert!((sheet_y - demo.last_notif_container.y - 108.0).abs() < 0.01);
}

#[test]
fn expanded_quick_settings_reopens_on_first_page() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.sys.anim_scale_idx = SystemState::ANIMATION_SCALES.iter().position(|&scale| scale == 1.0).unwrap();
    demo.set_capture_preset("qs");
    demo.qs_page = 1;
    demo.qs_page_anim.snap(1.0);

    demo.go_back(0.0);
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Touch);
    assert_eq!(demo.qs_page, 1);

    step(&mut ui, &mut demo, 3.0, None, false, false, PointerKind::Touch);
    assert_eq!(demo.shade_stage, 1);
    assert_eq!(demo.qs_page, 0);
    assert_eq!(demo.qs_page_anim.value, 0.0);

    demo.tap_shade_header(3.0);
    step(&mut ui, &mut demo, 6.0, None, false, false, PointerKind::Touch);
    assert_eq!(demo.shade_anim.value, 2.0);
    assert_eq!(demo.qs_page, 0);
    assert_eq!(demo.qs_page_anim.value, 0.0);
}

#[test]
fn qs_page_horizontal_swipe_and_notification_swipe_to_dismiss() {
    let mut ui = Ui::default();
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    // Test QS Page 1 -> Page 2 swipe
    demo.set_capture_preset("qs");
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Mouse);
    assert_eq!(demo.qs_page, 0);

    step(&mut ui, &mut demo, 0.05, Some([340.0, 320.0]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 0.12, Some([140.0, 320.0]), false, false, PointerKind::Mouse);
    assert!(demo.qs_page_anim.value > 0.35);
    step(&mut ui, &mut demo, 0.16, Some([80.0, 320.0]), false, true, PointerKind::Mouse);
    assert_eq!(demo.qs_page, 1);

    // Test Notification horizontal swipe-to-dismiss: smooth fly-off + lower-card collapse!
    let mut demo2 = LauncherDemo::<NoopEmbeddedApps>::default();
    demo2.set_capture_preset("shade");
    step(&mut ui, &mut demo2, 1.0, None, false, false, PointerKind::Mouse);
    let before = demo2.notifications.len();
    let first_card = demo2.last_notif_rects[0].2;
    let cy = first_card.y + first_card.height * 0.5;
    // Slow partial swipe (<30% width, low velocity) springs smoothly back to 0 without dismissing
    step(&mut ui, &mut demo2, 1.02, Some([80.0, cy]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo2, 1.24, Some([138.0, cy]), false, false, PointerKind::Mouse);
    step(&mut ui, &mut demo2, 1.32, Some([140.0, cy]), false, true, PointerKind::Mouse);
    assert!(!demo2.notifications[0].dismissing);
    for i in 1..=15 {
        step(&mut ui, &mut demo2, 1.32 + i as f64 * 0.02, None, false, false, PointerKind::Mouse);
    }
    assert!(demo2.notifications[0].swipe_anim.value.abs() < 1.0);
    // Full swipe (>30%) enters smooth dismissing collapse and removes item upon completion!
    step(&mut ui, &mut demo2, 1.80, Some([80.0, cy]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo2, 1.87, Some([280.0, cy]), false, true, PointerKind::Mouse);
    assert!(demo2.notifications[0].dismissing);
    for i in 1..=20 {
        step(&mut ui, &mut demo2, 1.87 + i as f64 * 0.02, None, false, false, PointerKind::Mouse);
    }
    assert_eq!(demo2.notifications.len(), before - 1);
}

#[test]
fn qs_edit_pencil_opens_circular_reveal_and_reorders_tiles() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo.set_capture_preset("qs");
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Touch);
    assert!(demo.last_qs_edit_btn.width >= 40.0);
    let cx = demo.last_qs_edit_btn.x + demo.last_qs_edit_btn.width * 0.5;
    let cy = demo.last_qs_edit_btn.y + demo.last_qs_edit_btn.height * 0.5;
    // Tapping Pencil opens QS Edit with 280ms circular reveal (never intercepted by EdgeBack!)
    step(&mut ui, &mut demo, 0.05, Some([cx, cy]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.10, Some([cx, cy]), false, true, PointerKind::Touch);
    assert!(demo.qs_edit_open);
    step(&mut ui, &mut demo, 0.24, None, false, false, PointerKind::Touch);
    assert!(demo.qs_edit_anim.value > 0.20 && demo.qs_edit_anim.value < 0.98);
    step(&mut ui, &mut demo, 0.45, None, false, false, PointerKind::Touch);
    assert!((demo.qs_edit_anim.value - 1.0).abs() < 0.02);
    // Back collapses circular reveal smoothly back to 0!
    demo.go_back(0.50);
    assert!(!demo.qs_edit_open);
    step(&mut ui, &mut demo, 0.62, None, false, false, PointerKind::Touch);
    assert!(demo.qs_edit_anim.value > 0.02 && demo.qs_edit_anim.value < 0.98);
    step(&mut ui, &mut demo, 0.85, None, false, false, PointerKind::Touch);
    assert!(demo.qs_edit_anim.value < 0.02);
}

#[test]
fn overview_card_phone_aspect_ratio_and_swipe_up_hold_morph() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo.open_app(AppId::Clock, 0.0);
    step(&mut ui, &mut demo, 0.50, None, false, false, PointerKind::Touch);
    // Bottom pill swipe up and hold -> enters Overview with overview_enter_from morph!
    step(&mut ui, &mut demo, 0.55, Some([205.0, 836.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.65, Some([205.0, 680.0]), false, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.85, Some([205.0, 678.0]), false, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.90, Some([205.0, 678.0]), false, true, PointerKind::Touch);
    assert_eq!(demo.mode, ShellMode::Overview);
    assert!(demo.overview_enter_from.is_some());
    for i in 1..=20 {
        step(&mut ui, &mut demo, 0.90 + i as f64 * 0.02, None, false, false, PointerKind::Touch);
    }
    assert!((demo.overview_anim.value - 1.0).abs() < 0.02);
    let (_, card0) = demo.last_overview_cards[0];
    let ratio = card0.height / card0.width;
    // Phone aspect ratio is 846/411 = 2.058; card height must closely match phone proportions!
    assert!(
        ratio > 1.85 && ratio < 2.15,
        "Overview card must match phone aspect ratio (~2.06), got {ratio:.2} ({}x{})",
        card0.width,
        card0.height
    );
}

#[test]
fn settings_home_scroll_and_desktop_bezel_render_cleanly() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo.set_capture_preset("settings_scrolled");
    let mut gmail = String::new();
    // Exercise desktop 1280x860 viewport (rounded bezel mask path) + scrolled Settings Home!
    let bounds = Rect { x: 0.0, y: 0.0, width: 1280.0, height: 860.0 };
    let input = Input { time: 0.1, ..Input::default() };
    ui.frame(input, bounds, 0.0, |f| {
        demo.show(f, 1280.0, 860.0, input, &mut gmail);
    });
    assert_eq!(demo.mode, ShellMode::Fullscreen(AppId::Settings));
    assert!(demo.last_phone_rect.x > 300.0);
    assert!((demo.last_phone_rect.width - 411.0).abs() < 1.0);
}

#[test]
fn home_icon_long_press_drag_repositions_on_grid_and_nav_pill_gestures() {
    // Box LauncherDemo in debug unit tests so thread stack stays small
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Mouse);
    let (_, _, [cx, cy]) = demo.last_home_icon_rects[0];

    // Long-press ArcaneChat icon (0.40s) then drag to cell (3, 2)
    step(&mut ui, &mut demo, 0.05, Some([cx, cy]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 0.48, Some([cx, cy]), false, false, PointerKind::Mouse);
    assert!(matches!(demo.home_popup, HomePopup::AppShortcuts { .. }));
    let target_x = demo.last_ws_rect.x + demo.last_ws_rect.width * 0.70; // col 3
    let target_y = demo.last_ws_rect.y + demo.last_ws_rect.height * 0.50; // row 2
    step(&mut ui, &mut demo, 0.56, Some([target_x, target_y]), false, false, PointerKind::Mouse);
    assert!(matches!(demo.gesture, ActiveGesture::HomeIconDrag { .. }));
    step(&mut ui, &mut demo, 0.64, Some([target_x, target_y]), false, true, PointerKind::Mouse);
    assert_eq!((demo.home_icons[0].0, demo.home_icons[0].1), (3, 2));

    // Bottom gesture pill: open Clock (lightweight app for debug stack), drag up & hold -> Overview
    demo.open_app(AppId::Clock, 1.0);
    step(&mut ui, &mut demo, 1.5, None, false, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 1.55, Some([205.0, 836.0]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 1.65, Some([205.0, 720.0]), false, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 1.85, Some([205.0, 718.0]), false, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 1.90, Some([205.0, 718.0]), false, true, PointerKind::Mouse);
    assert_eq!(demo.mode, ShellMode::Overview);
}

#[test]
fn edge_back_gesture_pops_settings_and_returns_app_to_icon() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Mouse);

    // Open Settings -> Display subpage, then swipe from RIGHT edge to pop Display -> Home
    demo.open_app(AppId::Settings, 0.1);
    demo.settings_app
        .navigate(settings::SettingsPage::Display, 0.2);
    step(&mut ui, &mut demo, 0.7, None, false, false, PointerKind::Touch);
    assert_eq!(
        demo.settings_app.current_page(),
        settings::SettingsPage::Display
    );

    // Right-edge inward drag (x: 406 -> 332, pull = 74dp >= 44dp commit)
    step(&mut ui, &mut demo, 0.80, Some([406.0, 420.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.88, Some([332.0, 420.0]), false, false, PointerKind::Touch);
    assert!(matches!(
        demo.gesture,
        ActiveGesture::EdgeBack {
            right_edge: true,
            ..
        }
    ));
    step(&mut ui, &mut demo, 0.94, Some([332.0, 420.0]), false, true, PointerKind::Touch);
    assert_eq!(demo.mode, ShellMode::Fullscreen(AppId::Settings));
    assert_eq!(demo.settings_app.current_page(), settings::SettingsPage::Home);

    // Open Phone (in Dock) and swipe from LEFT edge -> Predictive Back + Return-to-Icon Close!
    demo.open_app(AppId::Phone, 1.0);
    step(&mut ui, &mut demo, 1.5, None, false, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 1.60, Some([5.0, 440.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 1.70, Some([92.0, 445.0]), false, false, PointerKind::Touch);
    assert!(matches!(
        demo.gesture,
        ActiveGesture::EdgeBack {
            right_edge: false,
            ..
        }
    ));
    step(&mut ui, &mut demo, 1.75, Some([92.0, 445.0]), false, true, PointerKind::Touch);
    assert_eq!(demo.mode, ShellMode::Home);
    assert!(matches!(
        demo.app_transition,
        AppTransition::Closing {
            app: AppId::Phone,
            target_icon: Some(_),
            ..
        }
    ));
}

#[test]
fn app_open_from_dock_and_close_returning_to_icon_rect() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Mouse);

    // Tap Phone icon in Hotseat Dock (index 0 of 5)
    let (_, dock_cell) = demo.last_dock_icon_rects[0];
    let cx = dock_cell.x + dock_cell.width * 0.5;
    let cy = dock_cell.y + dock_cell.height * 0.5;
    step(&mut ui, &mut demo, 0.10, Some([cx, cy]), true, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 0.15, Some([cx, cy]), false, true, PointerKind::Mouse);
    assert_eq!(demo.mode, ShellMode::Fullscreen(AppId::Phone));
    assert!(matches!(
        demo.app_transition,
        AppTransition::Opening {
            app: AppId::Phone,
            from_icon: true,
            from_shade: false,
            ..
        }
    ));
    // Mid-open and settle
    step(&mut ui, &mut demo, 0.28, None, false, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 0.65, None, false, false, PointerKind::Mouse);
    assert!(matches!(demo.app_transition, AppTransition::None));

    // Exit to Home -> Return-to-Icon Closing animation targeting Phone's Dock icon!
    demo.go_home(0.80);
    assert_eq!(demo.mode, ShellMode::Home);
    let expected_dock = demo
        .find_home_or_dock_icon_rect(AppId::Phone, demo.last_phone_rect, 31.0, 24.0)
        .expect("Phone dock icon rect");
    match demo.app_transition {
        AppTransition::Closing {
            app: AppId::Phone,
            target_icon: Some(t),
            ..
        } => {
            assert!((t.x - expected_dock.x).abs() < 1.0);
            assert!((t.y - expected_dock.y).abs() < 1.0);
        }
        other => panic!("expected Closing->Dock icon transition, got {other:?}"),
    }
    // Step through mid-close and completion
    step(&mut ui, &mut demo, 0.96, None, false, false, PointerKind::Mouse);
    step(&mut ui, &mut demo, 1.30, None, false, false, PointerKind::Mouse);
    assert!(matches!(demo.app_transition, AppTransition::None));
}

#[test]
fn settings_search_page_and_notification_tap_launch_animation() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo.set_capture_preset("settings_search_results");
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Mouse);
    assert_eq!(
        demo.settings_app.current_page(),
        settings::SettingsPage::Search
    );
    // Back exits Settings Search -> Settings Home
    demo.go_back(0.2);
    assert_eq!(
        demo.settings_app.current_page(),
        settings::SettingsPage::Home
    );

    // Notification Shade: tap F-Droid card -> full-card ripple + expanding card launch over shade!
    let mut demo2 = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo2.set_capture_preset("shade");
    step(&mut ui, &mut demo2, 0.0, None, false, false, PointerKind::Touch);
    let first_card = demo2.last_notif_rects[0].2;
    let tx = first_card.x + 80.0;
    let ty = first_card.y + 30.0;
    step(&mut ui, &mut demo2, 0.10, Some([tx, ty]), true, false, PointerKind::Touch);
    assert!(demo2.notifications[0].ripple.is_some());
    step(&mut ui, &mut demo2, 0.16, Some([tx, ty]), false, true, PointerKind::Touch);
    assert!(matches!(
        demo2.app_transition,
        AppTransition::Opening {
            from_shade: true,
            ..
        }
    ));
    // Shade remains visible during NotificationLaunchAnimator and clears upon completion!
    step(&mut ui, &mut demo2, 0.28, None, false, false, PointerKind::Touch);
    assert_eq!(demo2.shade_stage, 1);
    step(&mut ui, &mut demo2, 0.65, None, false, false, PointerKind::Touch);
    assert!(matches!(demo2.app_transition, AppTransition::None));
    assert_eq!(demo2.shade_stage, 0);
}

#[test]
fn settings_about_phone_top_and_bottom_edge_stretch_overscroll() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo.open_app(AppId::Settings, 0.0);
    demo.settings_app
        .set_subpage_preview(settings::SettingsPage::About, 0.0);
    step(&mut ui, &mut demo, 0.5, None, false, false, PointerKind::Touch);

    // Pull DOWN at very top of About phone -> positive TOP stretch (> 1.04) & recoil back to 1.0!
    step(&mut ui, &mut demo, 0.55, Some([200.0, 260.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.70, Some([200.0, 580.0]), false, false, PointerKind::Touch);
    let (st_top, sign_top) = demo.target_stretch(ScrollTarget::Settings);
    assert!(
        st_top > 1.04 && sign_top > 0.0,
        "expected top EdgeStretch > 1.04, got ({st_top}, {sign_top})"
    );
    step(&mut ui, &mut demo, 0.75, Some([200.0, 580.0]), false, true, PointerKind::Touch);
    for i in 1..=35 {
        step(&mut ui, &mut demo, 0.75 + i as f64 * 0.02, None, false, false, PointerKind::Touch);
    }
    assert_eq!(demo.target_stretch(ScrollTarget::Settings), (1.0, 0.0));

    // Scroll to bottom of About phone, then pull UP -> negative BOTTOM stretch (> 1.04) & recoil!
    let max_s = demo.settings_app.scroll_state_mut().1;
    assert!(max_s > 80.0, "About phone should be scrollable, max_s={max_s}");
    demo.settings_app
        .set_subpage_preview(settings::SettingsPage::About, max_s);
    step(&mut ui, &mut demo, 1.6, None, false, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 1.65, Some([200.0, 620.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 1.80, Some([200.0, 280.0]), false, false, PointerKind::Touch);
    let (st_bot, sign_bot) = demo.target_stretch(ScrollTarget::Settings);
    assert!(
        st_bot > 1.04 && sign_bot < 0.0,
        "expected bottom EdgeStretch > 1.04, got ({st_bot}, {sign_bot})"
    );
    step(&mut ui, &mut demo, 1.85, Some([200.0, 280.0]), false, true, PointerKind::Touch);
    for i in 1..=35 {
        step(&mut ui, &mut demo, 1.85 + i as f64 * 0.02, None, false, false, PointerKind::Touch);
    }
    assert_eq!(demo.target_stretch(ScrollTarget::Settings), (1.0, 0.0));
}

#[test]
fn settings_one_second_hold_then_drag_down_shows_ripple_without_selecting() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo.open_app(AppId::Settings, 0.0);
    step(&mut ui, &mut demo, 0.5, None, false, false, PointerKind::Touch);
    assert_eq!(demo.settings_app.current_page(), settings::SettingsPage::Home);

    // Press & HOLD for 1.0s on a Settings list item (y = 380) -> Material ink ripple MUST appear & stay held!
    step(&mut ui, &mut demo, 0.60, Some([200.0, 380.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.85, Some([200.0, 380.0]), false, false, PointerKind::Touch);
    let mut gmail = String::new();
    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    let hold_input = Input { pointer: Some([200.0, 380.0]), pointer_kind: PointerKind::Touch, time: 1.60, ..Input::default() };
    let hold_cmds = ui.frame(hold_input, bounds, 0.0, |f| {
        demo.show(f, 411.0, 846.0, hold_input, &mut gmail);
    });
    assert!(
        hold_cmds.iter().any(|c| matches!(c, DrawCommand::Ripple { .. })),
        "holding 1s on Settings item must emit an active Material ink ripple"
    );
    assert_eq!(demo.settings_app.current_page(), settings::SettingsPage::Home);

    // Move touch DOWN (exceeding TOUCH_SLOP) -> cancels click, ripple STILL smoothly fades out while scrolling!
    let drag_input = Input { pointer: Some([200.0, 440.0]), pointer_kind: PointerKind::Touch, time: 1.68, ..Input::default() };
    let drag_cmds = ui.frame(drag_input, bounds, 0.0, |f| {
        demo.show(f, 411.0, 846.0, drag_input, &mut gmail);
    });
    assert!(matches!(demo.gesture, ActiveGesture::ListScroll { .. }));
    assert!(
        drag_cmds.iter().any(|c| matches!(c, DrawCommand::Ripple { .. })),
        "dragging down after 1s hold must smoothly fade out the ripple (not abruptly vanish)"
    );
    step(&mut ui, &mut demo, 1.76, Some([200.0, 520.0]), false, true, PointerKind::Touch);
    step(&mut ui, &mut demo, 1.80, None, false, false, PointerKind::Touch);
    assert_eq!(
        demo.settings_app.current_page(),
        settings::SettingsPage::Home,
        "1s hold followed by drag down must NOT select the Settings item!"
    );
}

#[test]
fn overview_variable_speed_gradual_deceleration_short_settles_back_fast_coasts_multicard() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo.set_capture_preset("overview");
    step(&mut ui, &mut demo, 0.0, None, false, false, PointerKind::Touch);
    let step_px = demo.last_overview_step;

    // 1. Short slow swipe right (~55dp over 300ms) -> settles smoothly BACK to card 0!
    step(&mut ui, &mut demo, 0.10, Some([160.0, 420.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.25, Some([195.0, 420.0]), false, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 0.42, Some([215.0, 420.0]), false, true, PointerKind::Touch);
    assert!(demo.overview_glide.is_some());
    for i in 1..=45 {
        step(&mut ui, &mut demo, 0.42 + i as f64 * 0.02, None, false, false, PointerKind::Touch);
    }
    assert!(
        demo.overview_scroll < 4.0,
        "slow short swipe must settle back to card 0, got {}",
        demo.overview_scroll
    );

    // 2. Fast flick right (40 -> 340 in 70ms -> ~4000dp/s!) -> coasts smoothly across >= 2 cards!
    step(&mut ui, &mut demo, 1.50, Some([40.0, 420.0]), true, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 1.535, Some([190.0, 420.0]), false, false, PointerKind::Touch);
    step(&mut ui, &mut demo, 1.57, Some([340.0, 420.0]), false, true, PointerKind::Touch);
    for i in 1..=50 {
        step(&mut ui, &mut demo, 1.57 + i as f64 * 0.02, None, false, false, PointerKind::Touch);
    }
    assert!(
        demo.overview_scroll >= step_px * 1.9,
        "fast flick must coast across >= 2 cards ({}), got {}",
        step_px * 2.0,
        demo.overview_scroll
    );
}

#[test]
fn freedesktop_notification_lifecycle_and_lockscreen_privacy_dismiss() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    demo.set_native_compositor(true);
    assert!(demo.notifications.is_empty());

    // 1. Push a normal D-Bus notification & a silent notification & a critical notification
    let wake1 = demo.push_freedesktop_notification(
        notifications::IncomingNotification {
            id: 101,
            replaces_id: 0,
            app_name: "Chat".into(),
            app_icon: String::new(),
            summary: "Secret message".into(),
            body: "OTP code is 492011".into(),
            actions: vec![("default".into(), "Open".into()), ("reply".into(), "Reply".into())],
            urgency: 1,
            category: Some("im.received".into()),
            desktop_entry: None,
            resident: false,
            transient: false,
            suppress_sound: false,
            image_data: None,
            image_path: None,
            icon_data: None,
            expire_timeout_ms: 5000,
        },
        1.0,
    );
    assert!(wake1);
    assert_eq!(demo.notifications.len(), 1);

    // Atomic replaces_id updates notification 101 in place
    demo.push_freedesktop_notification(
        notifications::IncomingNotification {
            id: 101,
            replaces_id: 101,
            app_name: "Chat".into(),
            app_icon: String::new(),
            summary: "Secret updated".into(),
            body: "Updated OTP 881122".into(),
            actions: vec![("default".into(), "Open".into())],
            urgency: 1,
            category: Some("im.received".into()),
            desktop_entry: None,
            resident: false,
            transient: false,
            suppress_sound: false,
            image_data: None,
            image_path: None,
            icon_data: None,
            expire_timeout_ms: 0,
        },
        1.2,
    );
    assert_eq!(demo.notifications.len(), 1);
    assert_eq!(demo.notifications[0].title, "Secret updated");

    // Critical notification MUST NOT auto-expire even if expire_timeout_ms > 0
    demo.push_freedesktop_notification(
        notifications::IncomingNotification {
            id: 102,
            replaces_id: 0,
            app_name: "Battery".into(),
            app_icon: String::new(),
            summary: "Battery critically low".into(),
            body: "2% remaining".into(),
            actions: Vec::new(),
            urgency: 2,
            category: Some("device".into()),
            desktop_entry: None,
            resident: false,
            transient: false,
            suppress_sound: false,
            image_data: None,
            image_path: None,
            icon_data: None,
            expire_timeout_ms: 1000,
        },
        1.5,
    );
    // Heads-up expires at t=8.5s (returning true for redraw), while critical notification MUST NOT auto-expire!
    let _ = demo.tick_notifications(10.0);
    assert_eq!(demo.notifications.len(), 2);
    assert!(demo.notifications.iter().any(|n| n.id == 102));

    // 2. Lock screen privacy modes: HideSensitive redacts body/title text; DontShow hides all cards!
    demo.lockscreen.lock();
    demo.sys.lock_screen_notifs = LockScreenNotifMode::HideSensitive;
    let mut gmail = String::new();
    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    let inp = Input { time: 2.0, ..Input::default() };
    let cmds_sensitive = ui.frame(inp, bounds, 0.0, |f| {
        demo.show(f, 411.0, 846.0, inp, &mut gmail);
    });
    let texts_sensitive: Vec<String> = cmds_sensitive
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { text, .. } => Some(text.to_string()),
            _ => None,
        })
        .collect();
    assert!(
        texts_sensitive.iter().any(|t| t.contains("Content hidden")),
        "HideSensitive must redact sensitive notification text on lock screen"
    );
    assert!(
        !texts_sensitive.iter().any(|t| t.contains("881122")),
        "HideSensitive must never leak notification body on lock screen"
    );

    demo.sys.lock_screen_notifs = LockScreenNotifMode::DontShow;
    let cmds_hidden = ui.frame(inp, bounds, 0.0, |f| {
        demo.show(f, 411.0, 846.0, inp, &mut gmail);
    });
    let texts_hidden: Vec<String> = cmds_hidden
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { text, .. } => Some(text.to_string()),
            _ => None,
        })
        .collect();
    assert!(
        texts_hidden.iter().any(|t| t.contains("notifications hidden")),
        "DontShow must hide notification cards on lock screen"
    );

    // 3. Dismiss emits Freedesktop NotificationClosed(id, 2)
    demo.sys.lock_screen_notifs = LockScreenNotifMode::ShowAll;
    demo.dismiss_notification(101, notifications::CLOSE_REASON_DISMISSED);
    let host_cmds = demo.take_host_commands();
    assert!(
        host_cmds.contains(&HostCommand::NotifyClosed {
            id: 101,
            reason: notifications::CLOSE_REASON_DISMISSED,
        }),
        "dismissing a D-Bus notification must emit NotificationClosed(id, 2)"
    );
}

#[test]
fn notification_center_shows_shiraz_tile_and_launcher_auto_enables_android_apps() {
    let mut demo: LauncherDemo = LauncherDemo::default();
    demo.native_compositor = true;
    demo.sys.shiraz_enabled = false;

    // Register a Freedesktop Shiraz Android app in the launcher registry
    let osmand = freedesktop::parse_desktop_entry(
        "net.osmand.plus.Shiraz.desktop",
        std::path::Path::new("/tmp/net.osmand.plus.Shiraz.desktop"),
        "[Desktop Entry]\nType=Application\nVersion=1.5\nName=OsmAnd~\nExec=shiraz-launch net.osmand.plus\nIcon=net.osmand.plus.Shiraz\nStartupWMClass=net.osmand.plus.Shiraz\nX-Shiraz-Package=net.osmand.plus\n",
        "",
        &["Wisp".into()],
    )
    .expect("valid Shiraz .desktop entry");
    demo.desktop_registry.apps.push(osmand);

    // 1. Pulling down Notification Center renders the Shiraz tile ("Enable Shiraz")
    demo.shade_stage = 1;
    demo.shade_anim.snap(1.0);
    let mut ui = Ui::default();
    let mut gmail = String::new();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 411.0,
        height: 846.0,
    };
    let cmds = ui.frame(Input::default(), bounds, 1.0, |f| {
        demo.show(f, 411.0, 846.0, Input::default(), &mut gmail);
    });
    let texts: Vec<String> = cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { text, .. } => Some(text.to_string()),
            _ => None,
        })
        .collect();
    assert!(
        texts.iter().any(|t| t == "Shiraz"),
        "Notification Center Stage 1 must expose the Shiraz tile, got: {texts:?}"
    );

    // 2. Opening a Shiraz Android app from the Launcher auto-enables Shiraz & emits ShirazLaunchPackage
    demo.open_desktop_app(0, 2.0);
    assert!(
        demo.sys.shiraz_enabled,
        "Launching an Android app while Shiraz is off must auto-enable Shiraz"
    );
    let host_cmds = demo.take_host_commands();
    assert!(
        host_cmds.contains(&HostCommand::ShirazLaunchPackage {
            package: "net.osmand.plus".into(),
            desktop_id: Some("net.osmand.plus.Shiraz.desktop".into()),
        }),
        "Expected ShirazLaunchPackage command, got: {host_cmds:?}"
    );
}

#[test]
fn wallpapers_can_paint_both_pages_during_forward_and_back_transitions() {
    let mut ui = Ui::default();
    let mut shell = LauncherDemo::<NoopEmbeddedApps>::default();
    let kind = PointerKind::Touch;
    shell.launcher_sub = LauncherSub::WallpaperStyle;
    shell.set_launcher_sub(LauncherSub::WallpaperPicker, true, 0.0);
    step(&mut ui, &mut shell, 0.01, None, false, false, kind);
    shell.wallpaper_files.set_picker(spectre_files::PickerRequest::Open {
        filter: spectre_files::Filter::Images, multiple: false,
    });
    shell.set_launcher_sub(LauncherSub::WallpaperFiles, true, 1.0);
    step(&mut ui, &mut shell, 1.01, None, false, false, kind);
    shell.set_launcher_sub(LauncherSub::WallpaperPicker, false, 2.0);
    step(&mut ui, &mut shell, 2.01, None, false, false, kind);
}

#[test]
fn radio_tiles_distinguish_enabled_from_connected_in_airplane_mode() {
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.sys.wifi_available = true;
    demo.sys.wifi_enabled = true;
    demo.sys.wifi_connected = false;
    demo.sys.bluetooth_available = true;
    demo.sys.bluetooth_enabled = true;
    demo.sys.set_airplane_mode(true);
    assert!(!demo.qs_tile_meta(QsTileKind::Wifi).3);
    assert!(!demo.qs_tile_meta(QsTileKind::Bluetooth).3);
    demo.sys.wifi_enabled = true;
    let (title, subtitle, _, enabled) = demo.qs_tile_meta(QsTileKind::Wifi);
    assert_eq!((title.as_str(), subtitle.as_str(), enabled), ("Wi-Fi", "Not connected", true));
    assert!(demo.sys.airplane_mode);
    demo.sys.set_airplane_mode(false);
    assert!(demo.sys.bluetooth_enabled);
}

#[test]
fn drawer_icon_hold_then_drag_places_and_launches_a_home_shortcut() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    let touch = PointerKind::Touch;
    step(&mut ui, &mut demo, 0.0, None, false, false, touch);
    demo.drawer_search = "Clock".into();
    demo.drawer_open = true;
    demo.drawer_anim.snap(1.0);
    step(&mut ui, &mut demo, 0.1, None, false, false, touch);

    let app = HomeApp::Builtin(AppId::Clock);
    let (_, _, origin) = demo.last_drawer_icon_rects.iter().find(|(icon, _, _)| *icon == app).unwrap();
    let origin = *origin;
    let workspace = demo.last_ws_rect;
    let drop = [workspace.x + workspace.width * 0.5, workspace.y + workspace.height * 0.5];

    step(&mut ui, &mut demo, 0.2, Some(origin), true, false, touch);
    step(&mut ui, &mut demo, 0.62, Some(origin), false, false, touch);
    assert!(demo.long_press_fired);
    assert!(demo.drawer_open);
    step(&mut ui, &mut demo, 0.75, Some(drop), false, false, touch);
    assert!(!demo.drawer_open);
    assert!(matches!(demo.gesture, ActiveGesture::HomeIconDrag { ws_idx: None, .. }));
    step(&mut ui, &mut demo, 1.2, Some(drop), false, true, touch);
    let index = demo.home_icons.iter().position(|(col, row, icon)| (*col, *row, icon) == (2, 2, &app)).unwrap();
    assert_eq!(demo.mode, ShellMode::Home);

    step(&mut ui, &mut demo, 1.6, None, false, false, touch);
    let (_, _, center) = *demo.last_home_icon_rects.iter().find(|(i, _, _)| *i == index).unwrap();
    step(&mut ui, &mut demo, 1.7, Some(center), true, false, touch);
    step(&mut ui, &mut demo, 1.8, Some(center), false, true, touch);
    assert_eq!(demo.mode, ShellMode::Fullscreen(AppId::Clock));
}

#[test]
fn desktop_home_shortcut_resolves_stable_id_when_registry_order_changes() {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    let touch = PointerKind::Touch;
    let id = "org.example.HomeShortcut.desktop";
    let entry = freedesktop::parse_desktop_entry(
        id,
        std::path::Path::new("/example/org.example.HomeShortcut.desktop"),
        "[Desktop Entry]\nType=Application\nName=Home Shortcut Example\nExec=example\n",
        "C",
        &[],
    ).unwrap();
    demo.desktop_registry.apps.push(entry);
    step(&mut ui, &mut demo, 0.0, None, false, false, touch);
    demo.drawer_search = "Home Shortcut Example".into();
    demo.drawer_open = true;
    demo.drawer_anim.snap(1.0);
    step(&mut ui, &mut demo, 0.1, None, false, false, touch);

    let app = HomeApp::Desktop(id.into());
    let (_, _, origin) = *demo.last_drawer_icon_rects.iter().find(|(icon, _, _)| *icon == app).unwrap();
    let workspace = demo.last_ws_rect;
    let drop = [workspace.x + workspace.width * 0.5, workspace.y + workspace.height * 0.5];
    step(&mut ui, &mut demo, 0.2, Some(origin), true, false, touch);
    step(&mut ui, &mut demo, 0.62, Some(origin), false, false, touch);
    step(&mut ui, &mut demo, 0.75, Some(drop), false, false, touch);
    step(&mut ui, &mut demo, 1.2, Some(drop), false, true, touch);
    let index = demo.home_icons.iter().position(|(_, _, icon)| *icon == app).unwrap();

    let decoy = freedesktop::parse_desktop_entry(
        "org.example.Decoy.desktop",
        std::path::Path::new("/example/org.example.Decoy.desktop"),
        "[Desktop Entry]\nType=Application\nName=Decoy\nExec=decoy\n",
        "C",
        &[],
    ).unwrap();
    demo.desktop_registry.apps.insert(0, decoy);
    // Enable host commands after placement so the test uses only an in-memory layout.
    demo.native_compositor = true;
    step(&mut ui, &mut demo, 1.6, None, false, false, touch);
    let (_, _, center) = *demo.last_home_icon_rects.iter().find(|(i, _, _)| *i == index).unwrap();
    step(&mut ui, &mut demo, 1.7, Some(center), true, false, touch);
    step(&mut ui, &mut demo, 1.8, Some(center), false, true, touch);
    assert!(demo.take_host_commands().contains(&HostCommand::LaunchDesktopApp(id.into(), None)));
}

#[test]
fn phone_and_messages_home_shortcuts_reuse_existing_pins() {
    check_native_app_home_shortcut(AppId::Phone, "org.gnome.Calls.desktop", "Calls", "gnome-calls");
    check_native_app_home_shortcut(AppId::Messaging, "sm.puri.Chatty.desktop", "Chatty", "chatty");
}

fn check_native_app_home_shortcut(app_id: AppId, desktop_id: &str, name: &str, executable: &str) {
    let mut ui = Ui::default();
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    let touch = PointerKind::Touch;
    let entry = freedesktop::parse_desktop_entry(
        desktop_id,
        &std::path::Path::new("/example").join(desktop_id),
        &format!("[Desktop Entry]\nType=Application\nName={name}\nExec={executable}\n"),
        "C",
        &[],
    ).unwrap();
    demo.desktop_registry.apps.insert(0, entry);
    let index = demo.home_icons.len();
    demo.home_icons.push((0, 0, HomeApp::Desktop(desktop_id.into())));
    demo.home_icon_pages.push(0);
    step(&mut ui, &mut demo, 0.0, None, false, false, touch);
    demo.drawer_search = name.into();
    demo.drawer_open = true;
    demo.drawer_anim.snap(1.0);
    step(&mut ui, &mut demo, 0.1, None, false, false, touch);
    let [(app, _, origin)] = demo.last_drawer_icon_rects.as_slice() else {
        panic!("{name} search must show one launcher entry");
    };
    assert_eq!(*app, HomeApp::Builtin(app_id));
    let origin = *origin;
    let workspace = demo.last_ws_rect;
    let drop = [workspace.x + workspace.width * 0.5, workspace.y + workspace.height * 0.5];
    step(&mut ui, &mut demo, 0.2, Some(origin), true, false, touch);
    step(&mut ui, &mut demo, 0.62, Some(origin), false, false, touch);
    step(&mut ui, &mut demo, 0.75, Some(drop), false, false, touch);
    step(&mut ui, &mut demo, 1.2, Some(drop), false, true, touch);
    assert_eq!(demo.home_icons.len(), index + 1);
    assert_eq!(demo.home_icons[index], (2, 2, HomeApp::Builtin(app_id)));

    // Enable the native launch command while keeping this test's layout in memory.
    demo.native_compositor = true;
    step(&mut ui, &mut demo, 1.6, None, false, false, touch);
    let (_, _, center) = *demo.last_home_icon_rects.iter().find(|(i, _, _)| *i == index).unwrap();
    step(&mut ui, &mut demo, 1.7, Some(center), true, false, touch);
    step(&mut ui, &mut demo, 1.8, Some(center), false, true, touch);
    assert!(demo.take_host_commands().contains(&HostCommand::LaunchOrFocus(app_id)));
}

#[test]
fn native_messages_stays_hidden_with_only_a_console_installed() {
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    let mut ui = Ui::default();
    demo.native_compositor = true;
    demo.desktop_registry.apps = vec![freedesktop::parse_desktop_entry(
        "org.gnome.Console.desktop",
        std::path::Path::new("/example/org.gnome.Console.desktop"),
        "[Desktop Entry]\nType=Application\nName=Console\nExec=kgx\n",
        "C",
        &[],
    ).unwrap()];
    assert!(demo.is_app_hidden(AppId::Messaging));
    demo.open_app(AppId::Messaging, 0.0);
    assert!(demo.take_host_commands().is_empty());

    demo.drawer_search = "Console".into();
    demo.drawer_open = true;
    demo.drawer_anim.snap(1.0);
    step(&mut ui, &mut demo, 0.1, None, false, false, PointerKind::Touch);
    assert_eq!(demo.last_drawer_icon_rects[0].0, HomeApp::Desktop("org.gnome.Console.desktop".into()));
}

#[test]
fn dark_fingerprint_unlock_reveals_immediately_and_respects_reduced_motion() {
    let mut demo = Box::new(LauncherDemo::<NoopEmbeddedApps>::default());
    let lock = &mut demo.lockscreen;
    lock.lock();
    lock.fingerprint_unlock(true);
    assert!(!lock.locked);
    lock.advance_unlock(1.0, 1.0, false);
    assert_eq!(lock.wake_reveal_remaining(), 1.0);
    assert_eq!(lock.home_reveal_alpha(), 1.0);
    lock.advance_unlock(1.11, 1.0, false);
    assert!((lock.wake_reveal_remaining() - 0.125).abs() < 0.001);
    lock.advance_unlock(1.23, 1.0, false);
    assert!(!lock.is_unlocking());
    lock.lock();
    lock.fingerprint_unlock(true);
    lock.advance_unlock(2.0, 1.0, true);
    assert_eq!(lock.wake_reveal_remaining(), 0.0);
    assert!(!lock.is_unlocking());
}

#[test]
fn clock_uses_device_date_and_minute_across_midnight_in_the_shade() {
    let mut demo = LauncherDemo::<NoopEmbeddedApps>::default();
    demo.lockscreen.unlock();
    demo.shade_stage = 1;
    demo.shade_anim.snap(1.0);
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 411.0, height: 846.0 };
    for (stamp, hm, date) in [
        ("2026-11-24T23:59:59.750+05:30", "23:59", "Tue, Nov 24"),
        ("2026-11-25T00:00:00.750+05:30", "00:00", "Wed, Nov 25"),
    ] {
        let reading = ClockReading(chrono::DateTime::parse_from_rfc3339(stamp).unwrap());
        demo.time_service = TimeService(std::sync::Arc::new(reading));
        let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
            demo.show(f, bounds.width, bounds.height, Input::default(), &mut String::new());
        });
        for expected in [hm, date] {
            assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if &**text == expected)), "{expected}");
        }
        assert_eq!(reading.until_second(), 0.25);
    }
}
