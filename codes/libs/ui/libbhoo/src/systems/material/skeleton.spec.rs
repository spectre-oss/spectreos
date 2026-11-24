use super::*;

#[test]
fn skeleton_pulse_and_wave_animate_and_stop_under_reduced_motion() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 200.0, height: 40.0 };
    let cmds = ui.frame(Input { time: 0.8, ..Input::default() }, bounds, 0.0, |f| {
        f.skeleton_with_options(
            Rect { x: 0.0, y: 0.0, width: 160.0, height: 24.0 },
            SkeletonOptions { animation: SkeletonAnimation::Wave, radius: 6.0 },
        );
    });
    assert!(ui.needs_redraw());
    // Base rect + clipped shimmer band
    assert!(cmds.iter().filter(|c| matches!(c, DrawCommand::RoundedRect { .. })).count() >= 2);

    let mut reduced_ui = Ui::default();
    reduced_ui.motion_preference = MotionPreference::Reduced;
    let rcmds = reduced_ui.frame(Input { time: 0.8, ..Input::default() }, bounds, 0.0, |f| {
        f.skeleton_with_options(
            Rect { x: 0.0, y: 0.0, width: 160.0, height: 24.0 },
            SkeletonOptions { animation: SkeletonAnimation::Wave, radius: 6.0 },
        );
    });
    assert!(!reduced_ui.needs_redraw());
    assert_eq!(rcmds.iter().filter(|c| matches!(c, DrawCommand::RoundedRect { .. })).count(), 1);
}
