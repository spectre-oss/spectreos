use super::*;

#[test]
fn command_palette_inline_filters_and_returns_original_index() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 320.0 };
    let items = [
        CommandItem { label: "Browser", shortcut: Some("Ctrl+B"), icon: Some(MaterialIcon::Search) },
        CommandItem { label: "Settings", shortcut: Some("Ctrl+,"), icon: Some(MaterialIcon::Settings) },
        CommandItem { label: "Terminal", shortcut: None, icon: None },
    ];
    let mut query = "set".to_owned();
    let _ = ui.frame(Input { time: 0.0, ..Input::default() }, bounds, 8.0, |f| {
        f.command_palette_inline(10, &mut query, &items, "Search commands");
    });
    let settings_node = ui
        .semantics()
        .iter()
        .find(|n| n.role == Role::MenuItem && n.label.as_ref() == "Settings")
        .expect("Settings visible");
    assert!(!ui.semantics().iter().any(|n| n.role == Role::MenuItem && n.label.as_ref() == "Browser"));
    let p = [settings_node.bounds.x + 20.0, settings_node.bounds.y + 20.0];
    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.1, ..Input::default() }, bounds, 8.0, |f| {
        f.command_palette_inline(10, &mut query, &items, "Search commands");
    });
    let mut activated = None;
    let _ = ui.frame(Input { pointer: Some(p), released: true, time: 0.15, ..Input::default() }, bounds, 8.0, |f| {
        activated = f.command_palette_inline(10, &mut query, &items, "Search commands");
    });
    assert_eq!(activated, Some(1));
}
