use crate::*;

#[test]
fn shortcut_hint_emits_keycap_and_description_text() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 64.0 };
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.shortcut_hint(&["Ctrl", "K"], "Search commands");
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Ctrl")));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "K")));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Search commands")));
}
