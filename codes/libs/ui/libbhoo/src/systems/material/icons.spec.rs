use super::*;

#[test]
fn material_icons_and_app_bar_icons_resolve_valid_shape_data() {
    for &icon in MaterialIcon::ALL {
        assert!(!icon.name().is_empty());
        let idx = crate::beercss_shapes::SHAPES.len()
            + crate::icons::material_navigation::ICONS.len()
            + icon as usize;
        assert!(crate::shape_data(idx).is_some(), "missing shape for {:?}", icon);
    }
    for app_icon in [
        AppBarIcon::Back,
        AppBarIcon::Forward,
        AppBarIcon::Folder,
        AppBarIcon::Tab,
        AppBarIcon::Search,
        AppBarIcon::More,
        AppBarIcon::Menu,
        AppBarIcon::Add,
    ] {
        assert!(crate::shape_data(app_icon.shape_index(false) as usize).is_some());
        assert!(crate::shape_data(app_icon.shape_index(true) as usize).is_some());
    }
    assert_ne!(AppBarIcon::Back.shape_index(false), AppBarIcon::Back.shape_index(true));
}

#[test]
fn paint_material_icon_emits_centered_square_shape_command() {
    let mut ui = Ui::default();
    let cmds = ui.frame(Input::default(), Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 }, 0.0, |f| {
        f.paint_material_icon(
            Rect { x: 10.0, y: 20.0, width: 40.0, height: 24.0 },
            MaterialIcon::Search,
            Color(255, 255, 255, 255),
        );
    });
    let shape = cmds.iter().find_map(|c| match *c {
        DrawCommand::Shape { bounds, .. } => Some(bounds),
        _ => None,
    });
    let b = shape.expect("shape emitted");
    assert_eq!(b.width, 24.0);
    assert_eq!(b.height, 24.0);
    assert!((b.x - 18.0).abs() < 0.01);
}
