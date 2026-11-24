use super::*;

const BOUNDS: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 200.0,
    height: 100.0,
};

fn button(ui: &mut Ui, input: Input) -> Response {
    let mut response = Response::default();
    ui.frame(input, BOUNDS, 8.0, |frame| {
        response = frame.button(1, "Open")
    });
    response
}

#[test]
fn click_requires_press_then_release_inside() {
    let mut ui = Ui::default();
    let inside = Input {
        pointer: Some([10.0, 10.0]),
        ..Input::default()
    };
    assert!(
        !button(
            &mut ui,
            Input {
                released: true,
                ..inside
            }
        )
        .clicked
    );
    assert!(
        button(
            &mut ui,
            Input {
                pressed: true,
                ..inside
            }
        )
        .pressed
    );
    assert!(
        button(
            &mut ui,
            Input {
                released: true,
                ..inside
            }
        )
        .clicked
    );
    assert!(
        !button(
            &mut ui,
            Input {
                released: true,
                ..inside
            }
        )
        .clicked
    );
}

#[test]
fn releasing_outside_or_cancelling_does_not_click() {
    for end in [
        Input {
            pointer: Some([250.0, 10.0]),
            released: true,
            ..Input::default()
        },
        Input {
            pointer: Some([10.0, 10.0]),
            released: true,
            cancelled: true,
            ..Input::default()
        },
    ] {
        let mut ui = Ui::default();
        button(
            &mut ui,
            Input {
                pointer: Some([10.0, 10.0]),
                pressed: true,
                ..Input::default()
            },
        );
        assert!(!button(&mut ui, end).clicked);
        assert_eq!(ui.active, None);
    }
}

#[test]
fn removed_widget_does_not_keep_capture() {
    let mut ui = Ui::default();
    button(
        &mut ui,
        Input {
            pointer: Some([10.0, 10.0]),
            pressed: true,
            ..Input::default()
        },
    );
    ui.frame(
        Input {
            released: true,
            ..Input::default()
        },
        BOUNDS,
        8.0,
        |_| {},
    );
    assert_eq!(ui.active, None);
}

#[test]
fn column_places_button_after_label() {
    let mut ui = Ui::default();
    ui.frame(Input::default(), BOUNDS, 8.0, |frame| {
        frame.label("Bhoo");
        frame.button(1, "Open");
    });
    let b = ui.semantics().iter().find(|n| n.id == 1).unwrap().bounds;
    assert_eq!(b.y, 36.0);
    assert_eq!(b.height, 40.0);
}

#[test]
fn theme_mix_interpolates_between_dark_and_light() {
    let dark = Theme::default();
    let light = Theme::light();
    assert_eq!(dark.mix(light, 0.0).background, dark.background);
    assert_eq!(dark.mix(light, 1.0).background, light.background);
    let mid = dark.mix(light, 0.5);
    assert!(mid.background.0 > dark.background.0 && mid.background.0 < light.background.0);
    assert!(mid.primary.1 != dark.primary.1 || mid.primary.0 != dark.primary.0);
}
