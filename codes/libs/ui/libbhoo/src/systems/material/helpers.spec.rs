use crate::*;

#[test]
fn paint_asym_rect_uses_single_rect_when_equal_and_two_clipped_halves_when_different() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 200.0, height: 100.0 };
    let eq_cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.paint_asym_rect(Rect { x: 0.0, y: 0.0, width: 100.0, height: 60.0 }, 16.0, 16.0, Color(255, 0, 0, 255));
    });
    assert_eq!(
        eq_cmds.iter().filter(|c| matches!(c, DrawCommand::RoundedRect { .. })).count(),
        1
    );

    let asym_cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.paint_asym_rect(Rect { x: 0.0, y: 0.0, width: 100.0, height: 60.0 }, 24.0, 4.0, Color(255, 0, 0, 255));
    });
    let radii: Vec<f32> = asym_cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { radius, .. } => Some(*radius),
            _ => None,
        })
        .collect();
    assert_eq!(radii, vec![24.0, 4.0]);
}

#[test]
fn paint_rounded_screen_mask_emits_four_corner_images() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 720.0 };
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.paint_rounded_screen_mask(bounds, 24.0, Color(0, 0, 0, 255));
    });
    assert_eq!(
        cmds.iter().filter(|c| matches!(c, DrawCommand::Image { .. })).count(),
        4
    );
}

#[test]
fn surface_applies_tonal_tint_and_mask_rounded_corners_draws_four_arcs() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 200.0, height: 120.0 };
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.surface(Rect { x: 10.0, y: 10.0, width: 80.0, height: 40.0 }, 12.0, 2);
        f.mask_rounded_corners(Rect { x: 100.0, y: 10.0, width: 80.0, height: 80.0 }, 16.0, Color(0, 0, 0, 255));
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })));
    assert_eq!(cmds.iter().filter(|c| matches!(c, DrawCommand::Arc { .. })).count(), 4);
}
