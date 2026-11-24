use super::*;
use crate::{DrawCommand, Input, Ui};

#[test]
fn all_system_icons_decode_to_nonempty_96x96_masks() {
    let white = Color(255, 255, 255, 255);
    for &icon in SystemIcon::ALL {
        let img = system_icon_image(icon, white);
        assert_eq!(img.width(), 96);
        assert_eq!(img.height(), 96);
        assert!(
            img.pixels().chunks_exact(4).any(|px| px[3] > 0),
            "expected visible pixels for {icon:?}"
        );
    }
}

#[test]
fn all_system_icons_decode_to_valid_48x48_msdf() {
    for &icon in SystemIcon::ALL {
        let img = system_icon_msdf(icon);
        assert_eq!(img.width(), 48);
        assert_eq!(img.height(), 48);
        assert_eq!(img.msdf_px_range(), 4.0);
        let mut inside = 0usize;
        let mut outside = 0usize;
        for px in img.pixels().chunks_exact(4) {
            let m = px[0].min(px[1]).max(px[0].max(px[1]).min(px[2]));
            if m >= 128 {
                inside += 1;
            } else {
                outside += 1;
            }
        }
        assert!(
            inside > 10 && outside > 10,
            "expected valid MSDF field for {icon:?} (inside={inside}, outside={outside})"
        );
    }
}

#[test]
fn paint_system_icon_skips_transparent_and_emits_image() {
    let mut ui = Ui::default();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    };
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.paint_system_icon(
            Rect {
                x: 4.0,
                y: 4.0,
                width: 24.0,
                height: 24.0,
            },
            SystemIcon::Wifi,
            Color(255, 255, 255, 0),
        );
        f.paint_system_icon(
            Rect {
                x: 4.0,
                y: 4.0,
                width: 24.0,
                height: 24.0,
            },
            SystemIcon::Wifi,
            Color(200, 220, 255, 255),
        );
    });
    let images = cmds
        .iter()
        .filter(|c| matches!(c, DrawCommand::Image { .. }))
        .count();
    assert_eq!(images, 1);
}
