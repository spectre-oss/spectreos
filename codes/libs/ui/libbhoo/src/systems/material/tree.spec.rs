use crate::*;

#[test]
fn tree_hides_collapsed_descendants_and_expands_on_click() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 300.0, height: 300.0 };
    let items = [
        TreeItem { id: 1, label: "Root", depth: 0, has_children: true, enabled: true },
        TreeItem { id: 2, label: "Child", depth: 1, has_children: false, enabled: true },
        TreeItem { id: 3, label: "Sibling", depth: 0, has_children: false, enabled: true },
    ];
    let mut state = TreeState::default();
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.tree(600, &items, &mut state);
    });
    assert!(ui.semantics().iter().all(|n| n.label.as_ref() != "Child"));
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "Sibling"));

    let root = ui.semantics().iter().find(|n| n.label.as_ref() == "Root").unwrap().bounds;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([root.x + 8.0, root.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            f.tree(600, &items, &mut state);
        });
    }
    assert!(state.expanded.contains(&1));
    ui.frame(Input { time: 0.3, ..Input::default() }, bounds, 0.0, |f| {
        f.tree(600, &items, &mut state);
    });
    assert!(ui.semantics().iter().any(|n| n.label.as_ref() == "Child"));
}
