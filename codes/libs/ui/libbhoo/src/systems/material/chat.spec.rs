use crate::*;

#[test]
fn chat_bubbles_align_incoming_left_and_outgoing_right_and_composer_sends() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 320.0 };
    let composer_rect = Rect { x: 0.0, y: 248.0, width: 360.0, height: 72.0 };
    let cmds = ui.frame(Input::default(), bounds, 8.0, |f| {
        f.chat_bubble(ChatMessage { body: "Hi!", timestamp: "10:00", outgoing: false });
        f.chat_bubble(ChatMessage { body: "Hello!", timestamp: "10:01", outgoing: true });
        f.message_composer(1600, composer_rect, "Message", "Ready", true);
    });
    let bubbles: Vec<Rect> = cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { bounds, radius, .. } if (*radius - 16.0).abs() < 0.1 => Some(*bounds),
            _ => None,
        })
        .collect();
    assert_eq!(bubbles.len(), 2);
    assert!(bubbles[0].x < bubbles[1].x);
    assert!(
        cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Ready")),
        "message_composer must paint the draft text"
    );

    let send = ui.semantics().iter().find(|n| n.id == 1601).unwrap().bounds;
    let mut sent = false;
    for (p, r, t) in [(true, false, 0.1), (false, true, 0.2)] {
        ui.frame(Input { pointer: Some([send.x + 8.0, send.y + 8.0]), pressed: p, released: r, time: t, ..Input::default() }, bounds, 8.0, |f| {
            let (_, s) = f.message_composer(1600, composer_rect, "Message", "Ready", true);
            sent |= s;
        });
    }
    assert!(sent);
}
