use crate::*;

#[test]
fn action_tile_toggles_and_renders_icon_title_subtitle() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 220.0, height: 72.0 };
    let mut active = false;
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.action_tile(
            1500,
            bounds,
            SystemIcon::Wifi,
            "Internet",
            "Spectre-5G",
            ActionTileOptions { active, show_chevron: true, ..ActionTileOptions::default() },
        );
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Image { .. })));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Internet")));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Spectre-5G")));

    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([40.0, 30.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            let resp = f.action_tile(
                1500,
                bounds,
                SystemIcon::Wifi,
                "Internet",
                "Spectre-5G",
                ActionTileOptions { active, show_chevron: true, ..ActionTileOptions::default() },
            );
            if resp.clicked {
                active = !active;
            }
        });
    }
    assert!(active);
}
