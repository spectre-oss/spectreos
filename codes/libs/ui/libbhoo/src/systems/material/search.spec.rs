use crate::*;

#[test]
fn inline_search_bar_triggers_back_and_clear_actions() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 340.0, height: 56.0 };
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.inline_search_bar(2400, bounds, "Search apps", "calc", true, true);
    });
    let clear = ui.semantics().iter().find(|n| n.id == 2402).unwrap().bounds;
    let mut cleared = false;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([clear.x + 8.0, clear.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            let (_, _, c) = f.inline_search_bar(2400, bounds, "Search apps", "calc", true, true);
            cleared |= c;
        });
    }
    assert!(cleared);
}
