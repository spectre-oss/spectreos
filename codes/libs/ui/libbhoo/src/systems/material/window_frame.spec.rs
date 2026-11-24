use super::*;

#[test]
fn floating_window_drags_resizes_and_triggers_caption_buttons() {
    let mut ui = Ui::default();
    let vp = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
    let mut win = WindowFrameState::new(Rect { x: 100.0, y: 100.0, width: 300.0, height: 240.0 });
    let opts = WindowFrameOptions { clamp_to: Some(vp), ..WindowFrameOptions::default() };

    // Drag caption bar by (+40, +25)
    let _ = ui.frame(Input { pointer: Some([150.0, 115.0]), pressed: true, time: 0.0, ..Input::default() }, vp, 0.0, |f| {
        f.floating_window(10, "Notes", &mut win, opts, |_, _| {});
    });
    let mut act = WindowFrameAction::default();
    let _ = ui.frame(Input { pointer: Some([190.0, 140.0]), time: 0.05, ..Input::default() }, vp, 0.0, |f| {
        act = f.floating_window(10, "Notes", &mut win, opts, |_, _| {});
    });
    assert!(act.moved);
    assert!((win.bounds.x - 140.0).abs() < 0.01);
    assert!((win.bounds.y - 125.0).abs() < 0.01);

    // Release then resize via bottom-right grip
    let _ = ui.frame(Input { pointer: Some([190.0, 140.0]), released: true, time: 0.1, ..Input::default() }, vp, 0.0, |f| {
        f.floating_window(10, "Notes", &mut win, opts, |_, _| {});
    });
    let grip_pt = [win.bounds.x + win.bounds.width - 6.0, win.bounds.y + win.bounds.height - 6.0];
    let _ = ui.frame(Input { pointer: Some(grip_pt), pressed: true, time: 0.15, ..Input::default() }, vp, 0.0, |f| {
        f.floating_window(10, "Notes", &mut win, opts, |_, _| {});
    });
    let _ = ui.frame(Input { pointer: Some([grip_pt[0] + 50.0, grip_pt[1] + 30.0]), time: 0.2, ..Input::default() }, vp, 0.0, |f| {
        act = f.floating_window(10, "Notes", &mut win, opts, |_, _| {});
    });
    assert!(act.resized);
    assert!((win.bounds.width - 350.0).abs() < 0.01);
    assert!((win.bounds.height - 270.0).abs() < 0.01);

    // Release and click close button
    let _ = ui.frame(Input { pointer: Some([0.0, 0.0]), released: true, time: 0.25, ..Input::default() }, vp, 0.0, |f| {
        f.floating_window(10, "Notes", &mut win, opts, |_, _| {});
    });
    let close_pt = [win.bounds.x + win.bounds.width - 20.0, win.bounds.y + 18.0];
    let _ = ui.frame(Input { pointer: Some(close_pt), pressed: true, time: 0.3, ..Input::default() }, vp, 0.0, |f| {
        f.floating_window(10, "Notes", &mut win, opts, |_, _| {});
    });
    let _ = ui.frame(Input { pointer: Some(close_pt), released: true, time: 0.35, ..Input::default() }, vp, 0.0, |f| {
        act = f.floating_window(10, "Notes", &mut win, opts, |_, _| {});
    });
    assert!(act.close_clicked);
}
