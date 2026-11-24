use bhoo::{
    DrawCommand, Input, Rect, Ui,
    snackbar::{Duration, Host, Notice, Outcome, Presentation},
};

fn notice(id: u64, duration: Duration, action: bool) -> Notice {
    Notice {
        id,
        message: format!("Notice {id}"),
        action: action.then(|| "Undo".into()),
        dismiss_label: None,
        duration,
    }
}

#[test]
fn queues_timeouts_actions_and_dismissals_in_order() {
    let mut host = Host::default();
    assert!(host.enqueue(notice(1, Duration::Short, false), 0.));
    assert!(host.enqueue(notice(2, Duration::Indefinite, true), 1.));
    assert!(host.enqueue(notice(3, Duration::Long, false), 1.));
    assert!(!host.enqueue(notice(2, Duration::Short, false), 1.));
    assert!(!host.action(2, 1.));
    assert!(!host.action(1, 1.));
    assert_eq!(host.update(1., false), Some(4.));
    assert_eq!(host.update(4., false), None);
    assert_eq!(host.take_result().unwrap().outcome, Outcome::TimedOut);
    assert_eq!(host.current().unwrap().id, 2);
    assert!(host.action(2, 9.));
    assert_eq!(host.take_result().unwrap().outcome, Outcome::Action);
    assert_eq!(host.update(9., false), Some(19.));
    assert!(host.dismiss(3, 10.));
    assert_eq!(host.take_result().unwrap().outcome, Outcome::Dismissed);
    assert!(host.current().is_none());
}

#[test]
fn hover_or_focus_preserves_remaining_time_and_queued_removal() {
    let mut host = Host::default();
    host.enqueue(notice(1, Duration::Seconds(6.), true), 0.);
    host.enqueue(notice(2, Duration::Short, false), 0.);
    assert!(host.dismiss(2, 1.));
    assert_eq!(host.take_result().unwrap().id, 2);
    assert_eq!(host.update(5., true), None);
    assert_eq!(host.update(100., true), None);
    assert_eq!(host.update(101., false), Some(102.));
    assert_eq!(host.update(102., false), None);
    assert_eq!(host.take_result().unwrap().outcome, Outcome::TimedOut);
}

#[test]
fn visual_host_draws_action_and_drives_queue_from_pointer_input() {
    let mut host = Host::default();
    let mut presentation = Presentation::default();
    let mut ui = Ui::default();
    let bounds = Rect {
        x: 0.,
        y: 0.,
        width: 320.,
        height: 240.,
    };
    host.enqueue(notice(1, Duration::Short, true), 0.);
    host.enqueue(notice(2, Duration::Short, false), 0.);
    let mut render = |ui: &mut Ui, time, pointer, pressed, released| {
        ui.frame(
            Input {
                time,
                pointer,
                pressed,
                released,
                ..Input::default()
            },
            bounds,
            0.,
            |f| {
                f.snackbar_host(42, bounds, &mut host, &mut presentation);
            },
        )
    };
    render(&mut ui, 0., None, false, false);
    assert!(ui.needs_redraw());
    let commands = render(&mut ui, 0.2, None, false, false);
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Notice 1"))
    );
    assert_eq!(ui.semantics()[0].label.as_ref(), "Undo");
    let status = ui
        .semantics()
        .iter()
        .find(|node| node.role == bhoo::Role::Status)
        .unwrap();
    assert_eq!(status.label.as_ref(), "Notice 1");
    assert!(!status.focused);
    assert_eq!(ui.next_wakeup(), Some(4.));
    render(&mut ui, 1., Some([280., 210.]), true, false);
    assert_eq!(ui.next_wakeup(), None);
    render(&mut ui, 1.1, Some([280., 210.]), false, true);
    drop(render);
    assert_eq!(host.take_result().unwrap().outcome, Outcome::Action);
    assert_eq!(host.current().unwrap().id, 2);
}

#[test]
fn reduced_motion_removes_dismissed_notice_immediately() {
    let mut host = Host::default();
    let mut presentation = Presentation::default();
    let mut ui = Ui::default();
    ui.motion_preference = bhoo::MotionPreference::Reduced;
    let bounds = Rect {
        x: 0.,
        y: 0.,
        width: 320.,
        height: 240.,
    };
    host.enqueue(notice(1, Duration::Indefinite, true), 0.);
    let commands = ui.frame(Input::default(), bounds, 0., |f| {
        f.snackbar_host(1, bounds, &mut host, &mut presentation)
    });
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Notice 1"))
    );
    assert!(!ui.needs_redraw());
    host.dismiss(1, 1.);
    let commands = ui.frame(
        Input {
            time: 1.,
            ..Input::default()
        },
        bounds,
        0.,
        |f| f.snackbar_host(1, bounds, &mut host, &mut presentation),
    );
    assert!(
        !commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Notice 1"))
    );
    assert!(ui.semantics().is_empty());
    assert!(!ui.needs_redraw());
}

#[test]
fn long_messages_wrap_above_the_action_without_overlapping_it() {
    let mut host = Host::default();
    let mut presentation = Presentation::default();
    let mut ui = Ui::default();
    ui.motion_preference = bhoo::MotionPreference::Reduced;
    let bounds = Rect {
        x: 0.,
        y: 0.,
        width: 220.,
        height: 240.,
    };
    host.enqueue(
        Notice {
            id: 1,
            message: "Message archived in the project folder".into(),
            action: Some("Undo".into()),
            dismiss_label: None,
            duration: Duration::Indefinite,
        },
        0.,
    );
    let commands = ui.frame(Input::default(), bounds, 0., |f| {
        f.snackbar_host(10, bounds, &mut host, &mut presentation)
    });
    let action = ui.semantics()[0].bounds;
    let lines: Vec<_> = commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { bounds, text, .. } if text.as_ref() != "Undo" => {
                Some((*bounds, text.as_ref()))
            }
            _ => None,
        })
        .collect();
    assert!(lines.len() >= 2);
    assert!(lines.iter().all(|(b, _)| b.x + b.width <= action.x));
    assert!(lines.windows(2).all(|w| w[0].0.y < w[1].0.y));
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, .. } if bounds.height >= 68. && bounds.y + bounds.height <= 232.)));
}

#[test]
fn narrow_and_explicit_stacked_actions_are_below_the_message() {
    for (width, explicit) in [(180., false), (400., true)] {
        let mut host = Host::default();
        let mut presentation = Presentation::default();
        presentation.action_on_new_line = explicit;
        let mut ui = Ui::default();
        ui.motion_preference = bhoo::MotionPreference::Reduced;
        let bounds = Rect {
            x: 0.,
            y: 0.,
            width,
            height: 240.,
        };
        host.enqueue(
            Notice {
                id: 1,
                message: "Message archived".into(),
                action: Some("Restore message".into()),
                dismiss_label: None,
                duration: Duration::Indefinite,
            },
            0.,
        );
        let commands = ui.frame(Input::default(), bounds, 0., |f| {
            f.snackbar_host(10, bounds, &mut host, &mut presentation)
        });
        let action = ui.semantics()[0].bounds;
        let message = commands
            .iter()
            .find_map(|c| match c {
                DrawCommand::Text { bounds, text, .. } if text.as_ref() == "Message archived" => {
                    Some(*bounds)
                }
                _ => None,
            })
            .unwrap();
        assert!(message.y + message.height <= action.y);
        assert!(action.x >= bounds.x && action.x + action.width <= bounds.x + bounds.width);
    }
}

#[test]
fn dismiss_control_closes_indefinite_notice_without_firing_action() {
    let mut host = Host::default();
    let mut presentation = Presentation::default();
    let mut ui = Ui::default();
    ui.motion_preference = bhoo::MotionPreference::Reduced;
    let bounds = Rect {
        x: 0.,
        y: 0.,
        width: 320.,
        height: 240.,
    };
    let mut first = notice(1, Duration::Indefinite, true);
    first.dismiss_label = Some("Dismiss notice".into());
    host.enqueue(first, 0.);
    host.enqueue(notice(2, Duration::Short, false), 0.);
    ui.frame(Input::default(), bounds, 0., |f| {
        f.snackbar_host(42, bounds, &mut host, &mut presentation)
    });
    let close = ui
        .semantics()
        .iter()
        .find(|n| n.label.as_ref() == "Dismiss notice")
        .unwrap()
        .bounds;
    let undo = ui
        .semantics()
        .iter()
        .find(|n| n.label.as_ref() == "Undo")
        .unwrap()
        .bounds;
    assert!(undo.x + undo.width <= close.x);
    let pointer = Some([close.x + close.width * 0.5, close.y + close.height * 0.5]);
    ui.frame(
        Input {
            time: 1.,
            pointer,
            pressed: true,
            ..Input::default()
        },
        bounds,
        0.,
        |f| f.snackbar_host(42, bounds, &mut host, &mut presentation),
    );
    ui.frame(
        Input {
            time: 1.1,
            pointer,
            released: true,
            ..Input::default()
        },
        bounds,
        0.,
        |f| f.snackbar_host(42, bounds, &mut host, &mut presentation),
    );
    assert_eq!(host.take_result().unwrap().outcome, Outcome::Dismissed);
    assert_eq!(host.current().unwrap().id, 2);
}

#[test]
fn rtl_mirrors_action_and_dismiss_controls_and_aligns_message() {
    let mut host = Host::default();
    let mut presentation = Presentation::default();
    presentation.right_to_left = true;
    let mut ui = Ui::default();
    ui.motion_preference = bhoo::MotionPreference::Reduced;
    let bounds = Rect {
        x: 0.,
        y: 0.,
        width: 400.,
        height: 240.,
    };
    let mut item = notice(1, Duration::Indefinite, true);
    item.dismiss_label = Some("Dismiss".into());
    host.enqueue(item, 0.);
    let commands = ui.frame(Input::default(), bounds, 0., |f| {
        f.snackbar_host(42, bounds, &mut host, &mut presentation)
    });
    let close = ui
        .semantics()
        .iter()
        .find(|n| n.label.as_ref() == "Dismiss")
        .unwrap()
        .bounds;
    let action = ui
        .semantics()
        .iter()
        .find(|n| n.label.as_ref() == "Undo")
        .unwrap()
        .bounds;
    let (message, align) = commands
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text {
                bounds,
                text,
                style,
                ..
            } if text.as_ref() == "Notice 1" => Some((*bounds, style.align)),
            _ => None,
        })
        .unwrap();
    assert!(close.x + close.width <= action.x);
    assert!(action.x + action.width <= message.x);
    assert_eq!(align, 2);
}
