use super::*;

#[test]
fn whole_star_rating_clicks_and_toggles_off_same_star() {
    let mut ui = Ui::default();
    let mut value = 0;
    let bounds = Rect { x: 0.0, y: 0.0, width: 240.0, height: 80.0 };
    let third_star = [90.0, 20.0];
    let _ = ui.frame(Input { pointer: Some(third_star), pressed: true, time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
        f.rating(10, &mut value, 5, true);
    });
    let _ = ui.frame(Input { pointer: Some(third_star), released: true, time: 0.05, ..Input::default() }, bounds, 0.0, |f| {
        f.rating(10, &mut value, 5, true);
    });
    assert_eq!(value, 3);

    let _ = ui.frame(Input { pointer: Some(third_star), pressed: true, time: 0.2, ..Input::default() }, bounds, 0.0, |f| {
        f.rating(10, &mut value, 5, true);
    });
    let _ = ui.frame(Input { pointer: Some(third_star), released: true, time: 0.25, ..Input::default() }, bounds, 0.0, |f| {
        f.rating(10, &mut value, 5, true);
    });
    assert_eq!(value, 0);
}

#[test]
fn fractional_rating_drags_to_half_star_step() {
    let mut ui = Ui::default();
    let mut value = 1.0_f32;
    let bounds = Rect { x: 0.0, y: 0.0, width: 240.0, height: 80.0 };
    // 5 stars across 240px -> width 48px/star. x = 110 -> ~2.29 -> ceil to 0.5 step -> 2.5
    let p = [110.0, 20.0];
    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
        let r = f.rating_fractional(30, &mut value, 5, 0.5, true);
        assert!(r.changed);
    });
    assert!((value - 2.5).abs() < 0.01);
}
