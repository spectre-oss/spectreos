use crate::*;

#[test]
fn pill_and_vertical_pill_sliders_respond_to_pointer_drag() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 240.0 };
    let h_rect = Rect { x: 10.0, y: 10.0, width: 200.0, height: 44.0 };
    let v_rect = Rect { x: 240.0, y: 10.0, width: 52.0, height: 200.0 };
    let mut brightness = 0.2;
    let mut volume = 0.2;

    ui.frame(Input { pointer: Some([160.0, 30.0]), pressed: true, time: 0.1, ..Input::default() }, bounds, 0.0, |f| {
        f.pill_slider(2300, h_rect, "Brightness", &mut brightness, 0.0..=1.0, Some(SystemIcon::Sun), true);
    });
    assert!((brightness - 0.75).abs() < 0.05);

    ui.frame(Input { pointer: Some([160.0, 30.0]), released: true, time: 0.15, ..Input::default() }, bounds, 0.0, |f| {
        f.pill_slider(2300, h_rect, "Brightness", &mut brightness, 0.0..=1.0, Some(SystemIcon::Sun), true);
    });

    ui.frame(Input { pointer: Some([260.0, 60.0]), pressed: true, time: 0.2, ..Input::default() }, bounds, 0.0, |f| {
        f.vertical_pill_slider(2301, v_rect, "Volume", &mut volume, 0.0..=1.0, Some(SystemIcon::Sound), true);
    });
    assert!((volume - 0.75).abs() < 0.05);
}
