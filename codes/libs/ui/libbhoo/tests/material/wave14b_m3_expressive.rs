//! Wave 14B native Material 3 Expressive widget parity tests:
//! 1. Wavy linear & circular progress indicators (`WavyProgressOptions`)
//! 2. `LoadingIndicator` contained M3 tokens (`primary_container` / `on_primary_container`)
//! 3. Plain tooltip M3 token alignment + interactive `rich_tooltip`
//! 4. Expressive `Slider` & `RangeSlider` (16dp tracks, 4dp×44dp handle, 6dp gaps, stops, ticks, min_separation, teardrop bubble)
//! 5. M3 Chip variants (`ChipKind`, `ChipOptions`, close action) & Primary / Secondary / Scrollable Tabs (`TabStyle`, `TabOptions`)

use bhoo::progress::wavy_amplitude_ramp;
use bhoo::*;

const BOUNDS: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 400.0,
    height: 600.0,
};

fn key(key: Key, pressed: bool) -> Input {
    Input {
        key: Some(KeyInput {
            key,
            pressed,
            repeat: false,
            shift: false,
        }),
        ..Input::default()
    }
}

// ============================================================================
// 1. M3 Expressive Wavy Progress Indicators
// ============================================================================

#[test]
fn wavy_progress_options_match_pinned_m3_defaults_and_amplitude_ramp() {
    let linear = WavyProgressOptions::default();
    assert_eq!(linear, WavyProgressOptions::linear());
    assert_eq!(linear.wavelength, 40.0);
    assert_eq!(linear.amplitude, 3.0);
    assert_eq!(linear.wave_speed, 40.0);
    assert_eq!(linear.track_thickness, 4.0);
    assert_eq!(linear.stop_indicator_size, 4.0);
    assert_eq!(linear.track_gap, 4.0);

    let circular = WavyProgressOptions::circular();
    assert_eq!(circular.wavelength, 16.0);
    assert_eq!(circular.amplitude, 3.0);
    assert_eq!(circular.wave_speed, 40.0);
    assert_eq!(circular.track_thickness, 4.0);
    assert_eq!(circular.stop_indicator_size, 4.0);
    assert_eq!(circular.track_gap, 4.0);

    assert_eq!(wavy_amplitude_ramp(0.0), 0.0);
    assert_eq!(wavy_amplitude_ramp(1.0), 0.0);
    assert_eq!(wavy_amplitude_ramp(0.5), 1.0);
    assert!(wavy_amplitude_ramp(0.05) > 0.0 && wavy_amplitude_ramp(0.05) < 1.0);
    assert!(wavy_amplitude_ramp(0.95) > 0.0 && wavy_amplitude_ramp(0.95) < 1.0);
}

#[test]
fn wavy_linear_progress_draws_sinusoid_gap_stop_dot_and_respects_ramps_and_reduced_motion() {
    let mut ui = Ui::default();
    let theme = ui.theme;
    let opt = WavyProgressOptions::linear();

    // Determinate at 50%: smooth sinusoidal polyline, 4dp gap to inactive track, 4dp stop dot.
    let cmds_t0 = ui.frame(
        Input {
            time: 0.0,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_linear_progress(1, "Sync", Some(0.5), opt),
    );
    assert!(ui.needs_redraw(), "Active wave should animate traveling phase");
    let lines_t0: Vec<([f32; 2], [f32; 2], f32, Color)> = cmds_t0
        .iter()
        .filter_map(|c| match *c {
            DrawCommand::Line {
                from,
                to,
                width,
                color,
            } => Some((from, to, width, color)),
            _ => None,
        })
        .collect();
    assert!(lines_t0.len() >= 16, "Expected smooth multi-segment wave");
    assert!(lines_t0.iter().all(|(_, _, w, c)| *w == 4.0 && *c == theme.primary));
    let min_y = lines_t0
        .iter()
        .map(|(a, b, _, _)| a[1].min(b[1]))
        .fold(f32::INFINITY, f32::min);
    let max_y = lines_t0
        .iter()
        .map(|(a, b, _, _)| a[1].max(b[1]))
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_y - min_y > 4.5 && max_y - min_y <= 6.05,
        "Expected peak-to-peak ~6dp (2 * 3dp amplitude), got {}",
        max_y - min_y
    );

    // Inactive track separated by 4dp gap from 50% (x = 200 -> inactive starts at 204).
    let inactive = cmds_t0
        .iter()
        .find_map(|c| match *c {
            DrawCommand::RoundedRect { bounds, color, .. }
                if color == theme.secondary_container =>
            {
                Some(bounds)
            }
            _ => None,
        })
        .expect("inactive straight track");
    assert!((inactive.x - 204.0).abs() < 0.01, "Expected 4dp gap at x=204, got {}", inactive.x);
    assert_eq!(inactive.height, 4.0);

    // Trailing 4dp stop indicator dot at right end (x = 396).
    assert!(cmds_t0.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, radius, color }
            if color == theme.primary
                && (bounds.x - 396.0).abs() < 0.01
                && bounds.width == 4.0
                && bounds.height == 4.0
                && radius == 2.0
    )));

    // Advancing time translates traveling phase.
    let cmds_t1 = ui.frame(
        Input {
            time: 0.25,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_linear_progress(1, "Sync", Some(0.5), opt),
    );
    let first_y_t0 = lines_t0[0].0[1];
    let first_y_t1 = cmds_t1
        .iter()
        .find_map(|c| match *c {
            DrawCommand::Line { from, .. } => Some(from[1]),
            _ => None,
        })
        .unwrap();
    assert!(
        (first_y_t1 - first_y_t0).abs() > 0.5,
        "Traveling phase must move wave vertically at fixed x"
    );

    // Near 0% and 100%, amplitude ramps to 0 -> flat bar (no wave lines), no stop dot at 100%.
    let mut ui_flat = Ui::default();
    let cmds_zero = ui_flat.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.wavy_linear_progress(2, "Empty", Some(0.0), opt)
    });
    assert!(!cmds_zero.iter().any(|c| matches!(c, DrawCommand::Line { .. })));
    assert!(!ui_flat.needs_redraw());

    let cmds_full = ui_flat.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.wavy_linear_progress(3, "Done", Some(1.0), opt)
    });
    assert!(!cmds_full.iter().any(|c| matches!(c, DrawCommand::Line { .. })));
    assert!(cmds_full.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, color, .. }
            if color == theme.primary && (bounds.width - 400.0).abs() < 0.01
    )));
    assert!(!ui_flat.needs_redraw());

    // Reduced motion: static wave WITHOUT traveling phase, no continuous redraw.
    let mut ui_red = Ui::default();
    ui_red.motion_preference = MotionPreference::Reduced;
    let r0 = ui_red.frame(
        Input {
            time: 0.0,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_linear_progress(4, "Reduced", Some(0.5), opt),
    );
    assert!(!ui_red.needs_redraw());
    let r1 = ui_red.frame(
        Input {
            time: 0.5,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_linear_progress(4, "Reduced", Some(0.5), opt),
    );
    assert!(!ui_red.needs_redraw());
    let red_lines_0: Vec<[f32; 2]> = r0
        .iter()
        .filter_map(|c| match *c {
            DrawCommand::Line { from, .. } => Some(from),
            _ => None,
        })
        .collect();
    let red_lines_1: Vec<[f32; 2]> = r1
        .iter()
        .filter_map(|c| match *c {
            DrawCommand::Line { from, .. } => Some(from),
            _ => None,
        })
        .collect();
    assert!(red_lines_0.len() >= 16, "Reduced motion still shows static wave");
    assert_eq!(red_lines_0, red_lines_1, "Reduced motion wave phase must stay static");

    // Indeterminate wavy linear progress sets continuous redraw and draws wavy active segment.
    let mut ui_ind = Ui::default();
    ui_ind.frame(
        Input {
            time: 0.0,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_linear_progress(5, "Loading", None, opt),
    );
    let ind_cmds = ui_ind.frame(
        Input {
            time: 0.4,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_linear_progress(5, "Loading", None, opt),
    );
    assert!(ui_ind.needs_redraw());
    assert!(ind_cmds.iter().any(|c| matches!(c, DrawCommand::Line { .. })));
    assert_eq!(ui_ind.semantics()[0].role, Role::Progress);
    assert_eq!(ui_ind.semantics()[0].value, None);
}

#[test]
fn wavy_circular_progress_draws_sinusoidal_polar_wave_inactive_arc_and_respects_reduced_motion() {
    let mut ui = Ui::default();
    let theme = ui.theme;
    let opt = WavyProgressOptions::circular();

    let cmds_t0 = ui.frame(
        Input {
            time: 0.0,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_circular_progress(10, "Upload", Some(0.5), opt),
    );
    assert!(ui.needs_redraw());
    // Straight inactive arc present with track gap.
    let inactive_arc = cmds_t0
        .iter()
        .find_map(|c| match *c {
            DrawCommand::Arc {
                sweep,
                stroke,
                color,
                ..
            } if color == theme.secondary_container => Some((sweep, stroke)),
            _ => None,
        })
        .expect("inactive circular track arc");
    assert_eq!(inactive_arc.1, 4.0);
    assert!(
        inactive_arc.0 > 0.0 && inactive_arc.0 < std::f32::consts::PI,
        "Inactive arc should be shortened by track_gap on both ends"
    );

    // Active wavy segments oscillate radially around center (24, 24).
    let radii: Vec<f32> = cmds_t0
        .iter()
        .filter_map(|c| match *c {
            DrawCommand::Line { from, color, .. } if color == theme.primary => {
                Some((from[0] - 24.0).hypot(from[1] - 24.0))
            }
            _ => None,
        })
        .collect();
    assert!(radii.len() >= 12);
    let r_min = radii.iter().copied().fold(f32::INFINITY, f32::min);
    let r_max = radii.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        r_max - r_min > 4.0 && r_max - r_min <= 6.1,
        "Expected radial peak-to-peak ~6dp, got {}",
        r_max - r_min
    );

    // At 100% progress, amplitude ramps to 0 -> draws smooth flat Arc instead of wavy lines.
    let mut ui_full = Ui::default();
    let full_cmds = ui_full.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.wavy_circular_progress(11, "Complete", Some(1.0), opt)
    });
    assert!(!full_cmds.iter().any(|c| matches!(c, DrawCommand::Line { .. })));
    assert!(full_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::Arc { color, sweep, .. }
            if color == theme.primary && (sweep - std::f32::consts::TAU).abs() < 0.01
    )));
    assert!(!ui_full.needs_redraw());

    // Reduced motion keeps static wave without moving phase or scheduling continuous redraw.
    let mut ui_red = Ui::default();
    ui_red.motion_preference = MotionPreference::Reduced;
    let a = ui_red.frame(
        Input {
            time: 0.0,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_circular_progress(12, "Static", Some(0.5), opt),
    );
    let b = ui_red.frame(
        Input {
            time: 0.5,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| f.wavy_circular_progress(12, "Static", Some(0.5), opt),
    );
    assert!(!ui_red.needs_redraw());
    assert_eq!(a, b);
}

// ============================================================================
// 2. LoadingIndicator Contained Tokens
// ============================================================================

#[test]
fn loading_indicator_contained_uses_m3_primary_container_and_on_primary_container_tokens() {
    let mut ui = Ui::default();
    // Use distinct custom colors so accidental fallback to secondary/text is impossible.
    ui.theme.primary = Color(10, 20, 30, 255);
    ui.theme.secondary = Color(99, 88, 77, 255);
    ui.theme.text = Color(250, 240, 230, 255);
    ui.theme.primary_container = Color(111, 22, 133, 255);
    ui.theme.on_primary_container = Color(244, 211, 255, 255);

    let mut state = LoadingState::default();
    let contained_cmds = ui.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.loading_indicator(20, "Contained busy", &mut state, Some(0.5), true);
    });
    assert!(contained_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, radius, color }
            if bounds.width == 48.0
                && bounds.height == 48.0
                && radius == 24.0
                && color == Color(111, 22, 133, 255)
    )));
    assert!(contained_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::Loading { color, .. } if color == Color(244, 211, 255, 255)
    )));

    // Uncontained variant retains primary shape color and no container fill.
    let uncontained_cmds = ui.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.loading_indicator(21, "Uncontained busy", &mut state, Some(0.5), false);
    });
    assert!(!uncontained_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { .. })));
    assert!(uncontained_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::Loading { color, .. } if color == Color(10, 20, 30, 255)
    )));
}

// ============================================================================
// 3. M3 Plain Tooltip Token Alignment + Rich Tooltip
// ============================================================================

#[test]
fn plain_tooltip_uses_m3_inverse_surface_tokens_and_4dp_radius() {
    let mut ui = Ui::default();
    let theme = ui.theme;
    ui.motion_preference = MotionPreference::Reduced;

    let build = |ui: &mut Ui, input: Input| {
        ui.frame(input, BOUNDS, 8.0, |f| {
            let r = f.button(30, "Info");
            f.tooltip(31, r, "Quick tip");
        })
    };
    build(
        &mut ui,
        Input {
            pointer: Some([20.0, 20.0]),
            time: 0.0,
            ..Input::default()
        },
    );
    let cmds = build(
        &mut ui,
        Input {
            pointer: Some([20.0, 20.0]),
            time: 0.6,
            ..Input::default()
        },
    );
    assert!(cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { radius, color, .. }
            if radius == 4.0 && color == theme.inverse_surface
    )));
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color, .. }
            if text.as_ref() == "Quick tip" && *color == theme.inverse_on_surface
    )));
}

#[test]
fn rich_tooltip_renders_m3_spec_and_handles_interactive_action_click_and_escape() {
    let mut ui = Ui::default();
    let theme = ui.theme;
    ui.motion_preference = MotionPreference::Reduced;

    // 1. Hover anchor at t=0 -> schedules 0.5s wakeup, not yet shown.
    let mut clicked = false;
    let cmds_wait = ui.frame(
        Input {
            pointer: Some([20.0, 20.0]),
            time: 0.0,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            let anchor = f.button(40, "Storage");
            clicked = f.rich_tooltip(
                41,
                anchor,
                "Cloud sync",
                "Back up files automatically over Wi-Fi.",
                Some("Learn more"),
            );
        },
    );
    assert!(!clicked);
    assert_eq!(ui.next_wakeup(), Some(0.5));
    assert!(!cmds_wait.iter().any(|c| matches!(c, DrawCommand::Shadow { radius, .. } if *radius == 12.0)));

    // 2. At t=0.6 -> rich tooltip reveals with level-2 shadow, 12dp radius, surface_low, bold 14sp subhead, 14sp body, primary action.
    let cmds_shown = ui.frame(
        Input {
            pointer: Some([20.0, 20.0]),
            time: 0.6,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            let anchor = f.button(40, "Storage");
            clicked = f.rich_tooltip(
                41,
                anchor,
                "Cloud sync",
                "Back up files automatically over Wi-Fi.",
                Some("Learn more"),
            );
        },
    );
    assert!(!clicked);
    assert_eq!(
        cmds_shown
            .iter()
            .filter(|c| matches!(c, DrawCommand::Shadow { radius, .. } if *radius == 12.0))
            .count(),
        3,
        "Expected 3-layer level-2 elevation shadow"
    );
    assert!(cmds_shown.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { radius, color, .. }
            if radius == 12.0 && color == theme.surface_low
    )));
    assert!(cmds_shown.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, style, .. }
            if text.as_ref() == "Cloud sync" && style.size == 14.0 && style.weight >= 700
    )));
    assert!(cmds_shown.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, style, .. }
            if text.as_ref().contains("Back up") && style.size == 14.0 && style.weight == 400
    )));
    let action_bounds = cmds_shown
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text {
                bounds,
                text,
                color,
                ..
            } if text.as_ref() == "Learn more" && *color == theme.primary => Some(*bounds),
            _ => None,
        })
        .expect("interactive primary action button");
    assert!(ui.semantics().iter().any(|n| n.role == Role::Tooltip));

    // 3. Move pointer onto the action button inside the tooltip and click -> stays open and returns true!
    let action_center = [
        action_bounds.x + action_bounds.width * 0.5,
        action_bounds.y + action_bounds.height * 0.5,
    ];
    ui.frame(
        Input {
            pointer: Some(action_center),
            pressed: true,
            time: 0.7,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            let anchor = f.button(40, "Storage");
            clicked = f.rich_tooltip(
                41,
                anchor,
                "Cloud sync",
                "Back up files automatically over Wi-Fi.",
                Some("Learn more"),
            );
        },
    );
    assert!(!clicked);
    ui.frame(
        Input {
            pointer: Some(action_center),
            released: true,
            time: 0.75,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            let anchor = f.button(40, "Storage");
            clicked = f.rich_tooltip(
                41,
                anchor,
                "Cloud sync",
                "Back up files automatically over Wi-Fi.",
                Some("Learn more"),
            );
        },
    );
    assert!(clicked, "Clicking rich tooltip action button must return true");

    // 4. Escape dismisses the tooltip.
    ui.frame(
        Input {
            pointer: Some([20.0, 20.0]),
            time: 0.8,
            key: Some(KeyInput {
                key: Key::Escape,
                pressed: true,
                repeat: false,
                shift: false,
            }),
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |f| {
            let anchor = f.button(40, "Storage");
            f.rich_tooltip(
                41,
                anchor,
                "Cloud sync",
                "Back up files automatically over Wi-Fi.",
                None,
            );
        },
    );
    assert!(!ui.semantics().iter().any(|n| n.role == Role::Tooltip));
}

// ============================================================================
// 4. M3 Expressive Slider & RangeSlider
// ============================================================================

#[test]
fn expressive_slider_renders_16dp_tracks_4x44_handle_6dp_gaps_stop_dot_ticks_and_value_bubble() {
    let mut ui = Ui::default();
    let theme = ui.theme;
    let mut value = 50.0;
    let bounds = Rect {
        x: 0.0,
        y: 80.0,
        width: 400.0,
        height: 400.0,
    };

    // Idle continuous slider: 16dp tracks, 4x44 handle, 6dp gap on both sides, 4dp stop dot, NO bubble.
    let idle = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.expressive_slider(
            50,
            "Brightness",
            &mut value,
            0.0..=100.0,
            SliderExpressiveOptions::default(),
        );
    });
    let handle = idle
        .iter()
        .find_map(|c| match *c {
            DrawCommand::RoundedRect { bounds, radius, color }
                if bounds.width == 4.0 && bounds.height == 44.0 && radius == 2.0 && color == theme.primary =>
            {
                Some(bounds)
            }
            _ => None,
        })
        .expect("4dp x 44dp vertical pill handle");
    let active_track = idle
        .iter()
        .find_map(|c| match *c {
            DrawCommand::RoundedRect { bounds, color, .. }
                if bounds.height == 16.0 && color == theme.primary =>
            {
                Some(bounds)
            }
            _ => None,
        })
        .expect("16dp active track");
    let inactive_track = idle
        .iter()
        .find_map(|c| match *c {
            DrawCommand::RoundedRect { bounds, color, .. }
                if bounds.height == 16.0 && color == theme.secondary_container =>
            {
                Some(bounds)
            }
            _ => None,
        })
        .expect("16dp inactive track");

    // Verify 6dp gap on both sides of the 4dp handle.
    assert!(
        ((handle.x - (active_track.x + active_track.width)) - 6.0).abs() < 0.01,
        "Expected 6dp leading gap between active track and handle"
    );
    assert!(
        ((inactive_track.x - (handle.x + handle.width)) - 6.0).abs() < 0.01,
        "Expected 6dp trailing gap between handle and inactive track"
    );
    // Verify 4dp trailing stop indicator dot.
    assert!(idle.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { bounds, radius, color }
            if bounds.width == 4.0
                && bounds.height == 4.0
                && radius == 2.0
                && color == theme.on_secondary_container
    )));
    // No value bubble while idle.
    assert!(!idle.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { radius, color, .. }
            if radius == 16.0 && color == theme.inverse_surface
    )));

    // Press slider with discrete step_size=25 -> snaps value and shows floating teardrop value bubble above handle.
    let opts = SliderExpressiveOptions {
        step_size: 25.0,
        show_value_bubble: true,
        ..SliderExpressiveOptions::default()
    };
    let pressed = ui.frame(
        Input {
            pointer: Some([305.0, 104.0]),
            pressed: true,
            ..Input::default()
        },
        bounds,
        0.0,
        |f| {
            f.expressive_slider(50, "Brightness", &mut value, 0.0..=100.0, opts);
        },
    );
    assert_eq!(value, 75.0, "Discrete slider must snap to nearest 25.0 step");
    // Discrete tick marks rendered (4dp dots).
    let tick_count = pressed
        .iter()
        .filter(|c| matches!(
            **c,
            DrawCommand::RoundedRect { bounds, radius, .. }
                if bounds.width == 4.0 && bounds.height == 4.0 && radius == 2.0
        ))
        .count();
    assert!(tick_count >= 3, "Expected discrete tick dots along track, got {tick_count}");
    // Floating teardrop value bubble above handle with value text "75".
    let bubble = pressed
        .iter()
        .find_map(|c| match *c {
            DrawCommand::RoundedRect { bounds, radius, color }
                if radius == 16.0 && color == theme.inverse_surface =>
            {
                Some(bounds)
            }
            _ => None,
        })
        .expect("teardrop value-indicator pill bubble");
    assert!(
        bubble.y + bubble.height <= handle.y,
        "Value bubble must float above the active handle"
    );
    assert!(pressed.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color, .. }
            if text.as_ref() == "75" && *color == theme.inverse_on_surface
    )));
}

#[test]
fn expressive_range_slider_enforces_min_separation_and_renders_two_handles_with_gaps_and_bubble() {
    let mut ui = Ui::default();
    let theme = ui.theme;
    let bounds = Rect {
        x: 0.0,
        y: 80.0,
        width: 400.0,
        height: 400.0,
    };
    // Initial values violate min_separation=20 -> sanitized automatically.
    let mut values = [45.0, 50.0];
    let opts = SliderExpressiveOptions {
        step_size: 5.0,
        min_separation: 20.0,
        show_value_bubble: true,
        enabled: true,
    };
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.expressive_range_slider([60, 61], "Price", &mut values, 0.0..=100.0, opts);
    });
    assert!(
        values[1] - values[0] >= 20.0 - 1e-4,
        "Initial range must enforce min_separation: {:?}",
        values
    );
    // Two 4dp x 44dp handles present.
    let handles: Vec<Rect> = cmds
        .iter()
        .filter_map(|c| match *c {
            DrawCommand::RoundedRect { bounds, radius, color }
                if bounds.width == 4.0 && bounds.height == 44.0 && radius == 2.0 && color == theme.primary =>
            {
                Some(bounds)
            }
            _ => None,
        })
        .collect();
    assert_eq!(handles.len(), 2);

    // Focus handle 0 with Tab and press End -> clamped to values[1] - min_separation.
    ui.frame(key(Key::Tab, true), bounds, 0.0, |f| {
        f.expressive_range_slider([60, 61], "Price", &mut values, 0.0..=100.0, opts);
    });
    let after_end = ui.frame(key(Key::End, true), bounds, 0.0, |f| {
        f.expressive_range_slider([60, 61], "Price", &mut values, 0.0..=100.0, opts);
    });
    assert!(
        (values[1] - values[0] - 20.0).abs() < 0.01,
        "Handle 0 End must clamp at values[1] - min_separation, got {:?}",
        values
    );
    // Focused handle displays value bubble.
    assert!(after_end.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { radius, color, .. }
            if radius == 16.0 && color == theme.inverse_surface
    )));

    // Tab to handle 1 and press Home -> clamped to values[0] + min_separation.
    ui.frame(key(Key::Tab, true), bounds, 0.0, |f| {
        f.expressive_range_slider([60, 61], "Price", &mut values, 0.0..=100.0, opts);
    });
    ui.frame(key(Key::Home, true), bounds, 0.0, |f| {
        f.expressive_range_slider([60, 61], "Price", &mut values, 0.0..=100.0, opts);
    });
    assert!(
        (values[1] - values[0] - 20.0).abs() < 0.01,
        "Handle 1 Home must clamp at values[0] + min_separation, got {:?}",
        values
    );
}

// ============================================================================
// 5. M3 Chip Variants & Primary/Secondary/Scrollable Tabs
// ============================================================================

#[test]
fn chip_with_options_supports_all_four_kinds_elevation_and_close_button_click() {
    let mut ui = Ui::default();
    let theme = ui.theme;
    ui.motion_preference = MotionPreference::Reduced;

    // 1. Assist chip ignores selected=true, no checkmark, uses text color; elevated uses surface_low + shadows.
    let assist_cmds = ui.frame(Input::default(), BOUNDS, 0.0, |f| {
        let (r, close) = f.chip_with_options(
            70,
            "Directions",
            ChipOptions {
                kind: ChipKind::Assist,
                selected: true,
                elevated: true,
                show_close: false,
                enabled: true,
            },
        );
        assert!(!close);
        assert_eq!(r.bounds.height, 32.0);
    });
    assert!(!ui.semantics()[0].selected, "Assist chip is never checkable");
    assert!(assist_cmds.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })));
    assert!(assist_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { radius, color, .. }
            if radius == 8.0 && color == theme.surface_low
    )));
    assert!(!assist_cmds.iter().any(|c| matches!(c, DrawCommand::Line { .. })));

    // 2. Filter chip selected draws secondary_container fill + checkmark lines.
    let filter_cmds = ui.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.chip_with_options(
            71,
            "Unread",
            ChipOptions {
                kind: ChipKind::Filter,
                selected: true,
                elevated: false,
                show_close: false,
                enabled: true,
            },
        );
    });
    assert!(ui.semantics()[0].selected);
    assert!(filter_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::RoundedRect { radius, color, .. }
            if radius == 8.0 && color == theme.secondary_container
    )));
    assert!(
        filter_cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Line { .. }))
            .count()
            >= 2,
        "Selected Filter chip must draw leading checkmark"
    );

    // 3. Suggestion chip selected uses secondary_container without checkmark.
    let suggestion_cmds = ui.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.chip_with_options(
            72,
            "Try today",
            ChipOptions {
                kind: ChipKind::Suggestion,
                selected: true,
                elevated: false,
                show_close: false,
                enabled: true,
            },
        );
    });
    assert!(ui.semantics()[0].selected);
    assert!(!suggestion_cmds.iter().any(|c| matches!(c, DrawCommand::Line { .. })));

    // 4. Input chip with show_close=true distinguishes body click from trailing close click.
    let input_opts = ChipOptions {
        kind: ChipKind::Input,
        selected: false,
        elevated: false,
        show_close: true,
        enabled: true,
    };
    let mut body_clicked = false;
    let mut close_clicked = false;
    // Click on chip body (left side, x = 40, y = 24).
    ui.frame(
        Input {
            pointer: Some([40.0, 24.0]),
            pressed: true,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| {
            f.chip_with_options(73, "alice@example.com", input_opts);
        },
    );
    ui.frame(
        Input {
            pointer: Some([40.0, 24.0]),
            released: true,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| {
            let (r, c) = f.chip_with_options(73, "alice@example.com", input_opts);
            body_clicked = r.clicked;
            close_clicked = c;
        },
    );
    assert!(body_clicked && !close_clicked);

    // Click on trailing close button (right end, x = 384, y = 24).
    ui.frame(
        Input {
            pointer: Some([384.0, 24.0]),
            pressed: true,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| {
            f.chip_with_options(73, "alice@example.com", input_opts);
        },
    );
    let close_cmds = ui.frame(
        Input {
            pointer: Some([384.0, 24.0]),
            released: true,
            ..Input::default()
        },
        BOUNDS,
        0.0,
        |f| {
            let (r, c) = f.chip_with_options(73, "alice@example.com", input_opts);
            body_clicked = r.clicked;
            close_clicked = c;
        },
    );
    assert!(!body_clicked && close_clicked, "Clicking close zone must return close_clicked=true");
    assert_eq!(
        close_cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::Line { .. }))
            .count(),
        2,
        "Close X icon draws two crossed lines"
    );
}

#[test]
fn tabs_with_options_differentiates_primary_vs_secondary_and_supports_horizontal_scroll() {
    let mut ui = Ui::default();
    let theme = ui.theme;
    ui.motion_preference = MotionPreference::Reduced;
    let mut selected = 0usize;

    // 1. Primary tabs: 3dp top-rounded label-width indicator, primary active label color.
    let primary_cmds = ui.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.tabs_with_options(
            80,
            &["Home", "Explore"],
            &mut selected,
            TabOptions {
                style: TabStyle::Primary,
                scrollable: false,
                enabled: true,
            },
        );
    });
    let primary_ind = primary_cmds
        .iter()
        .find_map(|c| match *c {
            DrawCommand::RoundedRect {
                bounds,
                radius,
                color,
            } if color == theme.primary && radius == 3.0 => Some(bounds),
            _ => None,
        })
        .expect("Primary 3dp top-rounded indicator");
    assert!(
        primary_ind.width >= 24.0 && primary_ind.width < 200.0,
        "Primary indicator must be label-width (< full 200dp tab width), got {}",
        primary_ind.width
    );
    // Indicator clip restricts visible height to 3dp.
    assert!(primary_cmds.iter().any(|c| matches!(
        *c,
        DrawCommand::PushClip(clip) if (clip.height - 3.0).abs() < 0.01 && (clip.y - 45.0).abs() < 0.01
    )));
    assert!(primary_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color, .. }
            if text.as_ref() == "Home" && *color == theme.primary
    )));

    // 2. Secondary tabs: 2dp full-tab-width sharp indicator, onSurface (text) active label color.
    let secondary_cmds = ui.frame(Input::default(), BOUNDS, 0.0, |f| {
        f.tabs_with_options(
            81,
            &["Home", "Explore"],
            &mut selected,
            TabOptions {
                style: TabStyle::Secondary,
                scrollable: false,
                enabled: true,
            },
        );
    });
    let secondary_ind = secondary_cmds
        .iter()
        .find_map(|c| match *c {
            DrawCommand::RoundedRect {
                bounds,
                radius,
                color,
            } if color == theme.primary && radius == 0.0 && bounds.height == 2.0 => Some(bounds),
            _ => None,
        })
        .expect("Secondary 2dp sharp full-width indicator");
    assert!(
        (secondary_ind.width - 200.0).abs() < 0.01,
        "Secondary indicator must span full 200dp tab width, got {}",
        secondary_ind.width
    );
    assert!(secondary_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color, .. }
            if text.as_ref() == "Home" && *color == theme.text
    )));

    // 3. Scrollable tabs: horizontal scroll shifts tab positions and pointer hit-testing uses offset.
    let narrow = Rect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 120.0,
    };
    let many_tabs = ["Overview", "Analytics", "Deployments", "Settings", "Security"];
    let mut scroll_sel = 0usize;
    let scroll_opts = TabOptions {
        style: TabStyle::Primary,
        scrollable: true,
        enabled: true,
    };
    let before_scroll = ui.frame(Input::default(), narrow, 0.0, |f| {
        f.tabs_with_options(82, &many_tabs, &mut scroll_sel, scroll_opts);
    });
    let x_before = before_scroll
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { bounds, text, .. } if text.as_ref() == "Overview" => Some(bounds.x),
            _ => None,
        })
        .unwrap();

    let after_scroll = ui.frame(
        Input {
            pointer: Some([120.0, 24.0]),
            scroll: [120.0, 0.0],
            ..Input::default()
        },
        narrow,
        0.0,
        |f| {
            f.tabs_with_options(82, &many_tabs, &mut scroll_sel, scroll_opts);
        },
    );
    let x_after = after_scroll
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { bounds, text, .. } if text.as_ref() == "Overview" => Some(bounds.x),
            _ => None,
        })
        .unwrap();
    assert!(
        (x_before - x_after - 120.0).abs() < 0.1,
        "Horizontal wheel scroll should shift tab X by 120dp (before={x_before}, after={x_after})"
    );

    // Click inside viewport at x=200 after scrolling 120dp -> selects a later tab (> 1).
    ui.frame(
        Input {
            pointer: Some([200.0, 24.0]),
            pressed: true,
            ..Input::default()
        },
        narrow,
        0.0,
        |f| {
            f.tabs_with_options(82, &many_tabs, &mut scroll_sel, scroll_opts);
        },
    );
    ui.frame(
        Input {
            pointer: Some([200.0, 24.0]),
            released: true,
            ..Input::default()
        },
        narrow,
        0.0,
        |f| {
            f.tabs_with_options(82, &many_tabs, &mut scroll_sel, scroll_opts);
        },
    );
    assert!(
        scroll_sel >= 2,
        "Clicking scrolled tab bar at x=200 with 120dp offset must select scrolled tab, got {scroll_sel}"
    );
}
