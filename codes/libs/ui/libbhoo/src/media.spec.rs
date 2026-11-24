use bhoo::*;
#[test]
fn image_validates_bytes_and_object_fit_geometry() {
    assert!(Image::rgba(0, 1, vec![]).is_none());
    assert!(Image::rgba(1023, 1, vec![0; 4092]).is_none());
    assert!(Image::rgba(2, 2, vec![0; 15]).is_none());
    let image = Image::rgba(4, 2, vec![255; 32]).unwrap();
    assert_eq!((image.width(), image.height()), (4, 2));
    assert_eq!(image.id(), image.clone().id());
    let b = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    assert_eq!(
        media::fit_image(b, 4, 2, ImageFit::Cover),
        Some((b, [0.25, 0.0, 0.75, 1.0]))
    );
    assert_eq!(
        media::fit_image(b, 4, 2, ImageFit::Contain),
        Some((
            Rect {
                x: 0.0,
                y: 25.0,
                width: 100.0,
                height: 50.0
            },
            [0.0, 0.0, 1.0, 1.0]
        ))
    );
    assert!(media::fit_image(Rect { width: 0.0, ..b }, 4, 2, ImageFit::Cover).is_none());
}
#[test]
fn image_commands_are_clipped_and_share_pixels() {
    let image = Image::rgba(2, 1, vec![255; 8]).unwrap();
    let b = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    let mut ui = Ui::default();
    let commands = ui.frame(Input::default(), b, 0.0, |f| {
        assert!(f.paint_image(b, &image, ImageFit::Cover, 8.0));
        assert!(!f.paint_image(Rect { x: 200.0, ..b }, &image, ImageFit::Cover, 0.0));
    });
    let command = commands
        .iter()
        .find_map(|c| {
            if let DrawCommand::Image {
                image: i,
                radius,
                uv,
                ..
            } = c
            {
                Some((i, *radius, *uv))
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(command.0.id(), image.id());
    assert_eq!(command.1, 8.0);
    assert_eq!(command.2, [0.25, 0.0, 0.75, 1.0]);
    assert!(ui.semantics().is_empty());
}
