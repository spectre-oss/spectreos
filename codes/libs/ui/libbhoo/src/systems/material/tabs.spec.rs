use crate::*;

#[test]
fn tabs_with_icons_switches_selection_on_click() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 300.0, height: 80.0 };
    let items = [
        TabItem { label: "Inbox", icon: Some(MaterialIcon::Inbox) },
        TabItem { label: "Starred", icon: Some(MaterialIcon::Star) },
    ];
    let mut selected = 0usize;
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.tabs_with_icons(2200, &items, &mut selected, true);
    });
    let tab1 = ui.semantics().iter().find(|n| n.id == 2201).unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([tab1.x + 10.0, tab1.y + 10.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            f.tabs_with_icons(2200, &items, &mut selected, true);
        });
    }
    assert_eq!(selected, 1);
}

#[test]
fn tabs_with_icons_emits_tablist_and_supports_arrow_navigation() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 100.0 };
    let items = [
        TabItem { label: "Home", icon: Some(MaterialIcon::Home) },
        TabItem { label: "Search", icon: Some(MaterialIcon::Search) },
        TabItem { label: "Settings", icon: Some(MaterialIcon::Settings) },
    ];
    let mut selected = 0usize;
    let _ = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.tabs_with_icons(50, &items, &mut selected, true);
    });
    assert!(ui.semantics().iter().any(|n| n.role == Role::TabList));
    ui.accessibility_focus(Some(50));
    let mut changed = false;
    let _ = ui.frame(
        Input {
            key: Some(KeyInput { key: Key::Right, pressed: true, repeat: false, shift: false }),
            time: 0.1,
            ..Input::default()
        },
        bounds,
        0.0,
        |f| {
            changed = f.tabs_with_icons(50, &items, &mut selected, true);
        },
    );
    assert!(changed && selected == 1);
}
