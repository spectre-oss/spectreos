use bhoo::*;
#[test]
fn static_shapes_have_valid_meshes_and_bounded_contours() {
    assert_eq!(beercss_shapes::SHAPES.len(), 37);
    assert!(
        beercss_shapes::SHAPES
            .iter()
            .any(|s| s.name == "wavy-circle" && s.contours.len() == 3)
    );
    for shape in beercss_shapes::SHAPES {
        assert!(shape.points.len() >= 3, "{}", shape.name);
        assert_eq!(shape.triangles.len() % 3, 0, "{}", shape.name);
        assert!(!shape.triangles.is_empty(), "{}", shape.name);
        assert!(
            shape
                .triangles
                .iter()
                .all(|i| (*i as usize) < shape.points.len()),
            "{}",
            shape.name
        );
        assert_eq!(shape.contours.first(), Some(&0));
        assert_eq!(
            shape.contours.last().copied(),
            Some(shape.points.len() as u16)
        );
        assert!(shape.contours.windows(2).all(|c| c[1] - c[0] >= 3));
        assert!(
            shape
                .points
                .iter()
                .flatten()
                .all(|x| x.is_finite() && *x >= -1.0 && *x <= 2.0),
            "{}",
            shape.name
        );
    }
}
#[test]
fn shape_drawing_rejects_invalid_inputs_and_has_no_focus_targets() {
    let mut ui = Ui::default();
    let mut accepted = [false; 3];
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    let commands = ui.frame(Input::default(), bounds, 0.0, |f| {
        accepted[0] = f.paint_shape(bounds, 0, Color(1, 2, 3, 255));
        accepted[1] = f.paint_shape(bounds, 37, Color(1, 2, 3, 255));
        accepted[2] = f.paint_shape(
            Rect {
                width: 0.0,
                ..bounds
            },
            0,
            Color(1, 2, 3, 255),
        );
    });
    assert_eq!(accepted, [true, false, false]);
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::Shape { .. }))
            .count(),
        1
    );
    assert!(ui.semantics().is_empty());
}
