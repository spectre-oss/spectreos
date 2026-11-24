//! Wave 15B Material 3 overlay & composition parity tests:
//! 1. `OverlayKind::BottomSheet` top-only 28dp corners + flush 0dp bottom edge
//! 2. `OverlayKind::Dialog` M3 28dp radius & `surfaceContainerHigh` surface
//! 3. Modal `OverlayKind::SideSheet` / `LeftSideSheet` inner-edge-only 16dp corners & horizontal drag-to-dismiss
//! 4. Coplanar adaptive `Frame::standard_side_sheet` (shared bounds + 1dp vertical divider + animated width)
//! 5. `SearchOverlayState` & `Frame::search_bar_overlay` (56dp pill -> 28dp `surfaceContainerHigh` SearchView, filtering, selection, clear, back, Escape)

use bhoo::*;

const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 400.0,
    height: 600.0,
};

fn key(key: Key, time: f64) -> Input {
    Input {
        time,
        key: Some(KeyInput {
            key,
            pressed: true,
            repeat: false,
            shift: false,
        }),
        ..Input::default()
    }
}

fn find_clipped_surface(
    commands: &[DrawCommand],
    expected_radius: f32,
    expected_color: Color,
) -> Option<(Rect, Rect)> {
    let mut clip_stack = Vec::new();
    for cmd in commands {
        match *cmd {
            DrawCommand::PushClip(c) => clip_stack.push(c),
            DrawCommand::PopClip => {
                clip_stack.pop();
            }
            DrawCommand::RoundedRect {
                bounds,
                radius,
                color,
            } if (radius - expected_radius).abs() < 0.01 && color == expected_color => {
                if let Some(&clip) = clip_stack.last() {
                    return Some((clip, bounds));
                }
            }
            _ => {}
        }
    }
    None
}

// ============================================================================
// 1. OverlayKind::BottomSheet — Top-Only 28dp Corners & Flush Bottom Edge
// ============================================================================

#[test]
fn bottom_sheet_renders_top_only_28dp_corners_with_flush_bottom_edge() {
    let sheet = Rect {
        x: 0.0,
        y: 340.0,
        width: 400.0,
        height: 260.0,
    };

    for drag_to_dismiss in [false, true] {
        let mut ui = Ui::default();
        ui.motion_preference = MotionPreference::Reduced;
        let mut state = OverlayState::default();
        state.kind = OverlayKind::BottomSheet;
        state.drag_to_dismiss = drag_to_dismiss;
        state.open = true;

        let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
            f.overlay(500, &mut state, sheet, true, |f, _| {
                f.label("Sheet content");
            });
        });

        let (clip, rect) = find_clipped_surface(&commands, 28.0, ui.theme.surface_low)
            .expect("BottomSheet must clip a 28dp surfaceContainerLow RoundedRect");

        // Clip sits exactly on the sheet bounds and flush against the viewport floor.
        assert_eq!(clip, sheet, "drag_to_dismiss={drag_to_dismiss}");
        assert_eq!(clip.y + clip.height, VIEW.y + VIEW.height);

        // Top corners sit at sheet.y with 28dp radius; bottom corners extend >= 28dp past
        // the bottom clip edge so the visible bottom edge is strictly 0dp flat and flush.
        assert_eq!(rect.x, sheet.x);
        assert_eq!(rect.y, sheet.y);
        assert_eq!(rect.width, sheet.width);
        assert!(
            rect.y + rect.height >= sheet.y + sheet.height + 28.0 - 0.01,
            "Bottom corners must extend below clip floor so bottom edge is flat 0dp: rect={rect:?}, sheet={sheet:?}"
        );
    }
}

// ============================================================================
// 2. OverlayKind::Dialog — M3 28dp Radius & surfaceContainerHigh Surface
// ============================================================================

#[test]
fn dialog_uses_m3_28dp_radius_and_surface_container_high() {
    let panel = Rect {
        x: 40.0,
        y: 160.0,
        width: 320.0,
        height: 220.0,
    };
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = OverlayState::default();
    state.kind = OverlayKind::Dialog;
    state.open = true;

    let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.overlay(510, &mut state, panel, true, |f, _| {
            f.label("Discard draft?");
            f.button(511, "Discard");
        });
    });

    assert!(
        commands.iter().any(|c| matches!(
            *c,
            DrawCommand::RoundedRect { bounds, radius, color }
                if bounds == panel
                    && (radius - 28.0).abs() < 0.01
                    && color == ui.theme.surface_high
        )),
        "M3 Dialog must render 28dp radius with surfaceContainerHigh (surface_high)"
    );

    // Fullscreen dialog still uses square 0dp corners across the full viewport.
    state.fullscreen = true;
    let fs_commands = ui.frame(
        Input {
            time: 0.1,
            ..Input::default()
        },
        VIEW,
        0.0,
        |f| {
            f.overlay(510, &mut state, panel, true, |f, _| {
                f.button(512, "Close");
            });
        },
    );
    assert!(fs_commands.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, radius, color }
            if bounds == VIEW && radius == 0.0 && color == ui.theme.surface_high
    )));
}

// ============================================================================
// 3. Modal Side Sheets — Inner-Edge-Only 16dp Corners & Drag-to-Dismiss
// ============================================================================

#[test]
fn modal_right_and_left_side_sheets_round_only_inner_edge_16dp() {
    let right_sheet = Rect {
        x: 120.0,
        y: 0.0,
        width: 280.0,
        height: 600.0,
    };
    let left_sheet = Rect {
        x: 0.0,
        y: 0.0,
        width: 280.0,
        height: 600.0,
    };

    // Right modal SideSheet: left (inner) corners 16dp, right (outer) edge flush 0dp.
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = OverlayState::default();
    state.kind = OverlayKind::SideSheet;
    state.open = true;
    let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.overlay(600, &mut state, right_sheet, true, |f, _| {
            f.label("Details");
        });
    });
    let (clip, rect) = find_clipped_surface(&commands, 16.0, ui.theme.surface_low)
        .expect("Right SideSheet must clip a 16dp surfaceContainerLow RoundedRect");
    assert_eq!(clip, right_sheet);
    assert_eq!(rect.x, right_sheet.x, "Inner left corners must sit at sheet.x");
    assert!(
        rect.x + rect.width >= right_sheet.x + right_sheet.width + 16.0 - 0.01,
        "Outer right corners must extend past right viewport edge so right edge is flush 0dp"
    );

    // Left modal LeftSideSheet: right (inner) corners 16dp, left (outer) edge flush 0dp.
    let mut ui_left = Ui::default();
    ui_left.motion_preference = MotionPreference::Reduced;
    let mut left_state = OverlayState::default();
    left_state.kind = OverlayKind::LeftSideSheet;
    left_state.open = true;
    let left_commands = ui_left.frame(Input::default(), VIEW, 0.0, |f| {
        f.overlay(601, &mut left_state, left_sheet, true, |f, _| {
            f.label("Filters");
        });
    });
    let (lclip, lrect) = find_clipped_surface(&left_commands, 16.0, ui_left.theme.surface_low)
        .expect("LeftSideSheet must clip a 16dp surfaceContainerLow RoundedRect");
    assert_eq!(lclip, left_sheet);
    assert!(
        lrect.x <= left_sheet.x - 16.0 + 0.01,
        "Outer left corners must extend past left viewport edge so left edge is flush 0dp"
    );
    assert!(
        (lrect.x + lrect.width - (left_sheet.x + left_sheet.width)).abs() < 0.01,
        "Inner right corners must sit at sheet right edge"
    );
}

#[test]
fn modal_side_sheet_horizontal_drag_to_dismiss_and_short_drag_settle() {
    let right_sheet = Rect {
        x: 120.0,
        y: 0.0,
        width: 280.0,
        height: 600.0,
    };
    let draw = |ui: &mut Ui, state: &mut OverlayState, input: Input, clicked: &mut bool| -> Option<f32> {
        let commands = ui.frame(input, VIEW, 0.0, |f| {
            f.overlay(610, state, right_sheet, true, |f, _| {
                if f.button(611, "Apply").clicked {
                    *clicked = true;
                }
            });
        });
        find_clipped_surface(&commands, 16.0, ui.theme.surface_low).map(|(_, rect)| rect.x)
    };

    let mut ui = Ui::default();
    let mut state = OverlayState::default();
    state.kind = OverlayKind::SideSheet;
    state.open = true;
    let mut clicked = false;

    // Settle open.
    draw(&mut ui, &mut state, Input { time: 0.0, ..Input::default() }, &mut clicked);
    let open_x = draw(&mut ui, &mut state, Input { time: 0.6, ..Input::default() }, &mut clicked).unwrap();
    assert!((open_x - 120.0).abs() < 0.2);

    // Normal click on button inside the sheet still works without starting a drag.
    let btn = ui.semantics().iter().find(|n| n.id == 611).unwrap().bounds;
    let center = Some([btn.x + btn.width * 0.5, btn.y + btn.height * 0.5]);
    draw(&mut ui, &mut state, Input { pointer: center, pressed: true, time: 0.65, ..Input::default() }, &mut clicked);
    draw(&mut ui, &mut state, Input { pointer: center, released: true, time: 0.67, ..Input::default() }, &mut clicked);
    assert!(clicked, "Normal button tap inside modal side sheet must work");
    assert!(state.open);

    // Short drag right (40dp < 40% of 280dp): follows pointer, stays open, settles back.
    draw(&mut ui, &mut state, Input { pointer: Some([180.0, 200.0]), pressed: true, time: 0.7, ..Input::default() }, &mut clicked);
    let dragged_x = draw(&mut ui, &mut state, Input { pointer: Some([220.0, 200.0]), time: 0.85, ..Input::default() }, &mut clicked).unwrap();
    assert!((dragged_x - 160.0).abs() < 0.2, "Sheet should follow rightward drag: {dragged_x}");
    let released_x = draw(&mut ui, &mut state, Input { pointer: Some([220.0, 200.0]), released: true, time: 0.9, ..Input::default() }, &mut clicked).unwrap();
    assert!(state.open, "Short drag must not dismiss");
    assert!((released_x - dragged_x).abs() < 0.5);
    let settling_x = draw(&mut ui, &mut state, Input { time: 1.0, ..Input::default() }, &mut clicked).unwrap();
    assert!(settling_x < released_x && settling_x > 120.0);
    let settled_x = draw(&mut ui, &mut state, Input { time: 1.6, ..Input::default() }, &mut clicked).unwrap();
    assert!((settled_x - 120.0).abs() < 0.2);

    // Long drag right (150dp >= 40% of 280dp): dismisses toward outer right edge!
    draw(&mut ui, &mut state, Input { pointer: Some([150.0, 200.0]), pressed: true, time: 1.7, ..Input::default() }, &mut clicked);
    let far_x = draw(&mut ui, &mut state, Input { pointer: Some([300.0, 200.0]), time: 1.9, ..Input::default() }, &mut clicked).unwrap();
    assert!((far_x - 270.0).abs() < 0.2);
    draw(&mut ui, &mut state, Input { pointer: Some([300.0, 200.0]), released: true, time: 1.95, ..Input::default() }, &mut clicked);
    assert!(!state.open, "Long outward drag must dismiss right SideSheet");
}

#[test]
fn modal_left_side_sheet_drag_left_dismisses() {
    let left_sheet = Rect {
        x: 0.0,
        y: 0.0,
        width: 280.0,
        height: 600.0,
    };
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = OverlayState::default();
    state.kind = OverlayKind::LeftSideSheet;
    state.open = true;

    let run = |ui: &mut Ui, state: &mut OverlayState, input: Input| {
        ui.frame(input, VIEW, 0.0, |f| {
            f.overlay(620, state, left_sheet, true, |f, _| {
                f.label("Navigation");
            });
        })
    };

    run(&mut ui, &mut state, Input { time: 0.0, ..Input::default() });
    assert!(state.open);

    // Drag leftward (toward outer left edge) by 140dp (50% of 280dp) -> dismisses.
    run(&mut ui, &mut state, Input { pointer: Some([220.0, 200.0]), pressed: true, time: 0.1, ..Input::default() });
    let cmds = run(&mut ui, &mut state, Input { pointer: Some([80.0, 200.0]), time: 0.25, ..Input::default() });
    let (clip, _) = find_clipped_surface(&cmds, 16.0, ui.theme.surface_low).unwrap();
    assert!((clip.width - 140.0).abs() < 0.5, "Visible sheet width should shrink to 140dp as it slides left: {clip:?}");
    run(&mut ui, &mut state, Input { pointer: Some([80.0, 200.0]), released: true, time: 0.3, ..Input::default() });
    assert!(!state.open, "Dragging LeftSideSheet past 40% toward outer left edge must dismiss");
}

// ============================================================================
// 4. Standard Coplanar SideSheet (Shared Space + 1dp Divider + Animated Width)
// ============================================================================

#[test]
fn standard_side_sheet_shares_coplanar_space_with_divider_and_animates() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut open = false;
    let mut content_clicks = 0;
    let mut sheet_clicks = 0;
    let mut content_bounds = Rect::default();
    let mut sheet_bounds = Rect::default();

    let run = |ui: &mut Ui,
               open: &mut bool,
               right_edge: bool,
               input: Input,
               content_clicks: &mut usize,
               sheet_clicks: &mut usize,
               content_bounds: &mut Rect,
               sheet_bounds: &mut Rect| {
        ui.frame(input, VIEW, 0.0, |f| {
            f.standard_side_sheet(
                700,
                open,
                right_edge,
                160.0,
                |f| {
                    *content_bounds = f.available();
                    if f.button(701, "Main action").clicked {
                        *content_clicks += 1;
                    }
                },
                |f| {
                    *sheet_bounds = f.available();
                    if f.button(702, "Inspector action").clicked {
                        *sheet_clicks += 1;
                    }
                },
            );
        })
    };

    // Closed: main content receives full 400dp width, no divider, no sheet button.
    let closed_cmds = run(
        &mut ui,
        &mut open,
        true,
        Input { time: 0.0, ..Input::default() },
        &mut content_clicks,
        &mut sheet_clicks,
        &mut content_bounds,
        &mut sheet_bounds,
    );
    assert_eq!(content_bounds.x, 0.0);
    assert_eq!(content_bounds.width, 400.0);
    assert!(ui.semantics().iter().all(|n| n.id != 702));
    assert!(!closed_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, color, .. }
            if bounds.width == 1.0 && bounds.height == 600.0 && color == ui.theme.outline_variant
    )));

    // Open on right edge: content (239dp) + 1dp vertical divider + sheet (160dp) = 400dp.
    open = true;
    let open_cmds = run(
        &mut ui,
        &mut open,
        true,
        Input { time: 0.1, ..Input::default() },
        &mut content_clicks,
        &mut sheet_clicks,
        &mut content_bounds,
        &mut sheet_bounds,
    );
    assert!((content_bounds.x - 0.0).abs() < 0.01);
    assert!((content_bounds.width - 239.0).abs() < 0.01);
    assert!((sheet_bounds.x - 240.0).abs() < 0.01);
    assert!((sheet_bounds.width - 160.0).abs() < 0.01);
    assert!(open_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, radius, color }
            if (bounds.x - 239.0).abs() < 0.01
                && (bounds.width - 1.0).abs() < 0.01
                && (bounds.height - 600.0).abs() < 0.01
                && radius == 0.0
                && color == ui.theme.outline_variant
    )), "Standard SideSheet must draw a 1dp vertical outline_variant divider at the seam");

    // Coplanar interactivity: both main content and sheet buttons work simultaneously!
    let main_btn = ui.semantics().iter().find(|n| n.id == 701).unwrap();
    let sheet_btn = ui.semantics().iter().find(|n| n.id == 702).unwrap();
    assert!(main_btn.enabled && sheet_btn.enabled);
    let mp = Some([main_btn.bounds.x + 10.0, main_btn.bounds.y + 10.0]);
    let sp = Some([sheet_btn.bounds.x + 10.0, sheet_btn.bounds.y + 10.0]);
    run(&mut ui, &mut open, true, Input { pointer: mp, pressed: true, time: 0.2, ..Input::default() }, &mut content_clicks, &mut sheet_clicks, &mut content_bounds, &mut sheet_bounds);
    run(&mut ui, &mut open, true, Input { pointer: mp, released: true, time: 0.21, ..Input::default() }, &mut content_clicks, &mut sheet_clicks, &mut content_bounds, &mut sheet_bounds);
    run(&mut ui, &mut open, true, Input { pointer: sp, pressed: true, time: 0.22, ..Input::default() }, &mut content_clicks, &mut sheet_clicks, &mut content_bounds, &mut sheet_bounds);
    run(&mut ui, &mut open, true, Input { pointer: sp, released: true, time: 0.23, ..Input::default() }, &mut content_clicks, &mut sheet_clicks, &mut content_bounds, &mut sheet_bounds);
    assert_eq!(content_clicks, 1);
    assert_eq!(sheet_clicks, 1);

    // Left edge coplanar layout: sheet [0..160], divider at 160, content [161..400].
    let left_cmds = run(
        &mut ui,
        &mut open,
        false,
        Input { time: 0.3, ..Input::default() },
        &mut content_clicks,
        &mut sheet_clicks,
        &mut content_bounds,
        &mut sheet_bounds,
    );
    assert!((sheet_bounds.x - 0.0).abs() < 0.01);
    assert!((sheet_bounds.width - 160.0).abs() < 0.01);
    assert!((content_bounds.x - 161.0).abs() < 0.01);
    assert!((content_bounds.width - 239.0).abs() < 0.01);
    assert!(left_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, color, .. }
            if (bounds.x - 160.0).abs() < 0.01
                && (bounds.width - 1.0).abs() < 0.01
                && color == ui.theme.outline_variant
    )));

    // Animated open/close width under MotionPreference::Full.
    let mut anim_ui = Ui::default();
    let mut anim_open = false;
    let mut c_w = 0.0;
    let sample = |ui: &mut Ui, open: &mut bool, time: f64, c_w: &mut f32| {
        ui.frame(Input { time, ..Input::default() }, VIEW, 0.0, |f| {
            f.standard_side_sheet(
                710,
                open,
                true,
                160.0,
                |f| {
                    *c_w = f.available().width;
                },
                |f| {
                    f.label("Animated sheet");
                },
            );
        });
    };
    sample(&mut anim_ui, &mut anim_open, 0.0, &mut c_w);
    assert_eq!(c_w, 400.0);
    anim_open = true;
    sample(&mut anim_ui, &mut anim_open, 0.01, &mut c_w);
    sample(&mut anim_ui, &mut anim_open, 0.05, &mut c_w);
    assert!(anim_ui.needs_redraw(), "Standard side sheet open transition should animate");
    assert!(c_w < 399.0 && c_w > 240.0, "Content width should animate smoothly between 400 and 239, got {c_w}");
    sample(&mut anim_ui, &mut anim_open, 1.0, &mut c_w);
    assert!((c_w - 239.0).abs() < 0.5);
    assert!(!anim_ui.needs_redraw());
}

// ============================================================================
// 5. SearchBar -> SearchView Container-Transform Overlay
// ============================================================================

#[test]
fn search_bar_overlay_expands_filters_selects_clears_and_dismisses() {
    let suggestions = ["Wi-Fi", "Bluetooth", "Display", "Blocklist"];
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = SearchOverlayState::default();
    assert!(!state.open);
    assert!(state.query.is_empty());
    assert_eq!(state.selected, None);

    let render = |ui: &mut Ui, state: &mut SearchOverlayState, input: Input| -> (Vec<DrawCommand>, Option<usize>) {
        let mut picked = None;
        let cmds = ui.frame(input, VIEW, 8.0, |f| {
            picked = f.search_bar_overlay(800, state, "Search settings", &suggestions);
        });
        (cmds, picked)
    };

    // 1. Closed state: 56dp M3 pill SearchBar (radius 28dp, surfaceContainerHigh), no divider/rows.
    let (closed_cmds, picked) = render(&mut ui, &mut state, Input { time: 0.0, ..Input::default() });
    assert_eq!(picked, None);
    assert!(!state.open);
    assert!(closed_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, radius, color }
            if (bounds.height - 56.0).abs() < 0.01
                && (radius - 28.0).abs() < 0.01
                && color == ui.theme.surface_high
    )));
    assert!(!ui.semantics().iter().any(|n| n.role == Role::ListItem));

    // 2. Tap closed pill -> expands into 28dp surfaceContainerHigh SearchView with back, divider, and suggestions.
    let bar_bounds = ui.semantics().iter().find(|n| n.label.as_ref() == "Search settings").unwrap().bounds;
    let bar_pt = Some([bar_bounds.x + 80.0, bar_bounds.y + 28.0]);
    render(&mut ui, &mut state, Input { pointer: bar_pt, pressed: true, time: 0.1, ..Input::default() });
    let (open_cmds, _) = render(&mut ui, &mut state, Input { pointer: bar_pt, released: true, time: 0.12, ..Input::default() });
    assert!(state.open);
    assert!(ui.has_escape_target());
    assert!(open_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, radius, color }
            if bounds.height > 100.0
                && (radius - 28.0).abs() < 0.01
                && color == ui.theme.surface_high
    )), "Expanded SearchView must have 28dp radius and surfaceContainerHigh background");
    assert!(open_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, color, .. }
            if (bounds.y - 56.0).abs() < 0.01
                && (bounds.height - 1.0).abs() < 0.01
                && color == ui.theme.outline_variant
    )), "Expanded SearchView must draw 1dp divider below header");
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "Back"));
    assert_eq!(
        ui.semantics().iter().filter(|n| n.role == Role::ListItem).count(),
        4
    );

    // 3. Filter suggestions via query ("bl" -> "Bluetooth" [1] & "Blocklist" [3]) and verify Clear button appears.
    state.query = "bl".into();
    render(&mut ui, &mut state, Input { time: 0.2, ..Input::default() });
    let visible_items: Vec<&str> = ui
        .semantics()
        .iter()
        .filter(|n| n.role == Role::ListItem)
        .map(|n| n.label.as_ref())
        .collect();
    assert_eq!(visible_items, vec!["Bluetooth", "Blocklist"]);
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "Clear"));

    // 4. Click "Blocklist" -> returns ORIGINAL suggestion index 3, updates state, closes SearchView.
    let blocklist_rect = ui.semantics().iter().find(|n| n.label.as_ref() == "Blocklist").unwrap().bounds;
    let row_pt = Some([blocklist_rect.x + 80.0, blocklist_rect.y + 24.0]);
    render(&mut ui, &mut state, Input { pointer: row_pt, pressed: true, time: 0.25, ..Input::default() });
    let (_, selected_idx) = render(&mut ui, &mut state, Input { pointer: row_pt, released: true, time: 0.27, ..Input::default() });
    assert_eq!(selected_idx, Some(3));
    assert_eq!(state.selected, Some(3));
    assert_eq!(state.query, "Blocklist");
    assert!(!state.open);

    // 5. Reopen, click Clear -> empties query while staying open, then Escape dismisses!
    state.open = true;
    render(&mut ui, &mut state, Input { time: 0.3, ..Input::default() });
    let clear_rect = ui.semantics().iter().find(|n| n.label.as_ref() == "Clear").unwrap().bounds;
    let clear_pt = Some([clear_rect.x + 20.0, clear_rect.y + 20.0]);
    render(&mut ui, &mut state, Input { pointer: clear_pt, pressed: true, time: 0.35, ..Input::default() });
    render(&mut ui, &mut state, Input { pointer: clear_pt, released: true, time: 0.37, ..Input::default() });
    assert!(state.open);
    assert!(state.query.is_empty());
    assert_eq!(state.selected, None);

    // Type 'w' directly via keyboard -> filters to "Wi-Fi".
    render(&mut ui, &mut state, key(Key::Character('w'), 0.4));
    assert_eq!(state.query, "w");
    let after_type: Vec<&str> = ui
        .semantics()
        .iter()
        .filter(|n| n.role == Role::ListItem)
        .map(|n| n.label.as_ref())
        .collect();
    assert_eq!(after_type, vec!["Wi-Fi"]);

    // Escape dismisses SearchView cleanly.
    render(&mut ui, &mut state, key(Key::Escape, 0.45));
    assert!(!state.open);
    assert!(!ui.has_escape_target());
    assert_eq!(state.query, "w");

    // 6. Reopen and dismiss via Back button.
    state.open = true;
    render(&mut ui, &mut state, Input { time: 0.5, ..Input::default() });
    let back_rect = ui.semantics().iter().find(|n| n.label.as_ref() == "Back").unwrap().bounds;
    let back_pt = Some([back_rect.x + 20.0, back_rect.y + 20.0]);
    render(&mut ui, &mut state, Input { pointer: back_pt, pressed: true, time: 0.55, ..Input::default() });
    render(&mut ui, &mut state, Input { pointer: back_pt, released: true, time: 0.57, ..Input::default() });
    assert!(!state.open);
}

#[test]
fn search_bar_overlay_animates_container_transform_under_full_motion() {
    let suggestions = ["Alpha", "Beta", "Gamma"];
    let mut ui = Ui::default();
    let mut state = SearchOverlayState::default();

    let bar_h = |cmds: &[DrawCommand], color: Color| -> f32 {
        cmds.iter()
            .filter_map(|c| match *c {
                DrawCommand::RoundedRect { bounds, radius, color: c_col }
                    if (radius - 28.0).abs() < 0.01 && c_col == color =>
                {
                    Some(bounds.height)
                }
                _ => None,
            })
            .fold(0.0_f32, f32::max)
    };

    let c0 = ui.frame(Input { time: 0.0, ..Input::default() }, VIEW, 8.0, |f| {
        f.search_bar_overlay(850, &mut state, "Search", &suggestions);
    });
    assert!((bar_h(&c0, ui.theme.surface_high) - 56.0).abs() < 0.01);

    // Tap to open -> height expands smoothly between 56dp and full target height.
    let pt = Some([100.0, 28.0]);
    ui.frame(Input { pointer: pt, pressed: true, time: 0.05, ..Input::default() }, VIEW, 8.0, |f| {
        f.search_bar_overlay(850, &mut state, "Search", &suggestions);
    });
    ui.frame(Input { pointer: pt, released: true, time: 0.06, ..Input::default() }, VIEW, 8.0, |f| {
        f.search_bar_overlay(850, &mut state, "Search", &suggestions);
    });
    assert!(state.open);

    let mid = ui.frame(Input { time: 0.1, ..Input::default() }, VIEW, 8.0, |f| {
        f.search_bar_overlay(850, &mut state, "Search", &suggestions);
    });
    let mid_h = bar_h(&mid, ui.theme.surface_high);
    assert!(ui.needs_redraw());
    assert!(mid_h > 56.0 && mid_h < 208.0, "Expected intermediate container-transform height, got {mid_h}");

    let end = ui.frame(Input { time: 1.0, ..Input::default() }, VIEW, 8.0, |f| {
        f.search_bar_overlay(850, &mut state, "Search", &suggestions);
    });
    let end_h = bar_h(&end, ui.theme.surface_high);
    assert!((end_h - 209.0).abs() < 1.0, "Expected settled expanded height ~209dp, got {end_h}");
    assert!(!ui.needs_redraw());
}

#[test]
fn crate_root_reexports_wave14b_and_wave15b_types() {
    let _search = SearchOverlayState::default();
    let _wavy = WavyProgressOptions::default();
    let _chip = ChipOptions {
        kind: ChipKind::Assist,
        ..ChipOptions::default()
    };
    let _slider = SliderExpressiveOptions::default();
    let _tab = TabOptions {
        style: TabStyle::Secondary,
        ..TabOptions::default()
    };
}
