use bhoo::*;
const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 400.0,
    height: 360.0,
};
const MENU: Rect = Rect {
    x: 20.0,
    y: 20.0,
    width: 300.0,
    height: 300.0,
};
const ITEMS: [FabMenuAction<'static>; 3] = [
    FabMenuAction {
        label: "Document",
        glyph: "+",
        enabled: true,
    },
    FabMenuAction {
        label: "Folder",
        glyph: "+",
        enabled: false,
    },
    FabMenuAction {
        label: "Import",
        glyph: "v",
        enabled: true,
    },
];
#[test]
fn action_shadows_extend_beyond_the_pill() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = FabMenuState::default();
    let mut open = true;
    let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.fab_menu(100, MENU, &mut open, &ITEMS, &mut state);
    });
    let mut clips = vec![VIEW];
    let mut shadows = 0;
    for command in commands {
        match command {
            DrawCommand::PushClip(rect) => clips.push(rect),
            DrawCommand::PopClip => {
                clips.pop();
            }
            DrawCommand::Shadow { bounds, .. } if bounds.width > 56.0 && bounds.height == 56.0 => {
                let clip = clips.last().unwrap();
                assert!(clip.y + clip.height > bounds.y + bounds.height);
                assert!(clip.x < bounds.x);
                assert!(clip.x + clip.width > bounds.x + bounds.width);
                shadows += 1;
            }
            _ => {}
        }
    }
    assert!(shadows >= ITEMS.len());
}
fn frame(ui: &mut Ui, state: &mut FabMenuState, open: &mut bool, input: Input) -> Option<usize> {
    let mut result = None;
    ui.frame(input, VIEW, 0.0, |f| {
        result = f.fab_menu(100, MENU, open, &ITEMS, state)
    });
    result
}
fn k(key: Key, time: f64) -> Input {
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
fn opens_focuses_trigger_and_closes_without_focusable_exit_items() {
    let mut ui = Ui::default();
    let mut state = FabMenuState::default();
    let mut open = false;
    frame(&mut ui, &mut state, &mut open, Input::default());
    frame(&mut ui, &mut state, &mut open, k(Key::Tab, 0.01));
    frame(&mut ui, &mut state, &mut open, k(Key::Enter, 0.02));
    assert!(open);
    assert!(ui.semantics().iter().any(|n| n.id == 100 && n.focused));
    frame(&mut ui, &mut state, &mut open, k(Key::Down, 0.03));
    assert!(ui.semantics().iter().any(|n| n.id == 100 && n.focused));
    frame(
        &mut ui,
        &mut state,
        &mut open,
        Input {
            time: 0.3,
            ..Input::default()
        },
    );
    frame(
        &mut ui,
        &mut state,
        &mut open,
        Input {
            time: 0.35,
            ..Input::default()
        },
    );
    assert_eq!(
        ui.semantics()
            .iter()
            .filter(|n| (101..=103).contains(&n.id))
            .count(),
        3
    );
    assert!(!ui.semantics().iter().find(|n| n.id == 102).unwrap().enabled);
    assert!(ui.semantics().iter().any(|n| n.id == 101 && n.focused));
    frame(&mut ui, &mut state, &mut open, k(Key::Down, 0.36));
    assert!(ui.semantics().iter().any(|n| n.id == 103 && n.focused));
    frame(&mut ui, &mut state, &mut open, k(Key::Escape, 0.37));
    assert!(!open);
    assert!(
        ui.semantics()
            .iter()
            .filter(|n| (101..=103).contains(&n.id))
            .all(|n| !n.enabled)
    );
    frame(
        &mut ui,
        &mut state,
        &mut open,
        Input {
            time: 2.0,
            ..Input::default()
        },
    );
    frame(
        &mut ui,
        &mut state,
        &mut open,
        Input {
            time: 4.0,
            ..Input::default()
        },
    );
    assert!(!ui.needs_redraw());
}
#[test]
fn reduced_motion_shows_actions_immediately_and_action_closes() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = FabMenuState::default();
    let mut open = true;
    frame(&mut ui, &mut state, &mut open, Input::default());
    assert_eq!(
        ui.semantics()
            .iter()
            .filter(|n| (101..=103).contains(&n.id))
            .count(),
        3
    );
    assert!(!ui.needs_redraw());
    let b = ui.semantics().iter().find(|n| n.id == 103).unwrap().bounds;
    let p = [b.x + b.width / 2.0, b.y + b.height / 2.0];
    frame(
        &mut ui,
        &mut state,
        &mut open,
        Input {
            pointer: Some(p),
            pressed: true,
            time: 0.1,
            ..Input::default()
        },
    );
    assert_eq!(
        frame(
            &mut ui,
            &mut state,
            &mut open,
            Input {
                pointer: Some(p),
                released: true,
                time: 0.2,
                ..Input::default()
            }
        ),
        Some(2)
    );
    assert!(!open);
}
#[test]
fn compact_menu_scrolls_clips_and_mirrors_in_rtl() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = FabMenuState::default();
    let mut open = true;
    let short = Rect {
        x: 20.0,
        y: 20.0,
        width: 300.0,
        height: 156.0,
    };
    let mut selected = None;
    let mut draw = |ui: &mut Ui, state: &mut FabMenuState, open: &mut bool, input: Input| {
        ui.frame(input, VIEW, 0.0, |f| {
            selected = f.fab_menu_with(
                100,
                short,
                open,
                &ITEMS,
                state,
                FabMenuOptions { rtl: true },
            )
        });
    };
    draw(&mut ui, &mut state, &mut open, Input::default());
    assert!((state.scroll_offset() - 84.0).abs() < 0.001);
    let trigger = ui.semantics().iter().find(|n| n.id == 100).unwrap();
    assert_eq!(trigger.bounds.x, short.x);
    assert!(!ui.semantics().iter().any(|n| n.id == 101 && n.enabled));
    assert!(ui.semantics().iter().any(|n| n.id == 103 && n.enabled));
    draw(
        &mut ui,
        &mut state,
        &mut open,
        Input {
            pointer: Some([50.0, 40.0]),
            scroll: [0.0, -300.0],
            time: 0.1,
            ..Input::default()
        },
    );
    assert_eq!(state.scroll_offset(), 0.0);
    let first = ui.semantics().iter().find(|n| n.id == 101).unwrap();
    assert!(first.enabled);
    assert_eq!(first.bounds.x, short.x);
    assert!(first.bounds.y >= short.y);
    assert!(!ui.semantics().iter().any(|n| n.id == 103 && n.enabled));
}
#[test]
fn touch_drag_scrolls_without_activating_an_action() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = FabMenuState::default();
    let mut open = true;
    let short = Rect {
        x: 20.0,
        y: 20.0,
        width: 300.0,
        height: 156.0,
    };
    let draw = |ui: &mut Ui, state: &mut FabMenuState, open: &mut bool, input: Input| {
        let mut selected = None;
        ui.frame(input, VIEW, 0.0, |f| {
            selected = f.fab_menu_with(
                100,
                short,
                open,
                &ITEMS,
                state,
                FabMenuOptions { rtl: true },
            )
        });
        selected
    };
    draw(&mut ui, &mut state, &mut open, Input::default());
    assert_eq!(state.scroll_offset(), 84.0);
    draw(
        &mut ui,
        &mut state,
        &mut open,
        Input {
            pointer: Some([50.0, 80.0]),
            pointer_kind: PointerKind::Touch,
            pressed: true,
            time: 0.1,
            ..Input::default()
        },
    );
    draw(
        &mut ui,
        &mut state,
        &mut open,
        Input {
            pointer: Some([50.0, 110.0]),
            pointer_kind: PointerKind::Touch,
            time: 0.2,
            ..Input::default()
        },
    );
    assert_eq!(state.scroll_offset(), 54.0);
    assert_eq!(
        draw(
            &mut ui,
            &mut state,
            &mut open,
            Input {
                pointer: Some([50.0, 110.0]),
                pointer_kind: PointerKind::Touch,
                released: true,
                time: 0.3,
                ..Input::default()
            }
        ),
        None
    );
    assert!(open);
}
#[test]
fn compact_keyboard_moves_focus_and_scrolls_past_disabled_action() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = FabMenuState::default();
    let mut open = true;
    let short = Rect {
        x: 20.0,
        y: 20.0,
        width: 300.0,
        height: 156.0,
    };
    let draw = |ui: &mut Ui, state: &mut FabMenuState, open: &mut bool, input: Input| {
        ui.frame(input, VIEW, 0.0, |f| {
            f.fab_menu_with(100, short, open, &ITEMS, state, FabMenuOptions::default());
        });
    };
    draw(&mut ui, &mut state, &mut open, Input::default());
    draw(&mut ui, &mut state, &mut open, k(Key::Tab, 0.1));
    assert!(ui.semantics().iter().any(|n| n.id == 100 && n.focused));
    draw(&mut ui, &mut state, &mut open, k(Key::Down, 0.2));
    assert_eq!(state.scroll_offset(), 0.0);
    assert!(ui.semantics().iter().any(|n| n.id == 101 && n.focused));
    draw(&mut ui, &mut state, &mut open, k(Key::Down, 0.3));
    assert_eq!(state.scroll_offset(), 84.0);
    assert!(ui.semantics().iter().any(|n| n.id == 103 && n.focused));
}
