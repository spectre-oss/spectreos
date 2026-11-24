use crate::*;

#[test]
fn empty_state_renders_title_and_triggers_cta() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 240.0 };
    let opts = EmptyStateOptions {
        icon: Some(MaterialIcon::Inbox),
        action_label: Some("Refresh"),
        error: false,
    };
    ui.frame(Input::default(), bounds, 0.0, |f| {
        f.empty_state(1400, "No messages", "Everything is up to date.", opts);
    });
    let btn = ui.semantics().iter().find(|n| n.id == 1400).unwrap().bounds;
    let mut clicked = false;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([btn.x + 8.0, btn.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 0.0, |f| {
            if let Some(resp) = f.empty_state(1400, "No messages", "Everything is up to date.", opts) {
                clicked |= resp.clicked;
            }
        });
    }
    assert!(clicked);
}
