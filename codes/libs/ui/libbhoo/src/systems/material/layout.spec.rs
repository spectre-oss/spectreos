use bhoo::*;
#[test]
fn grid_positions_children_and_skips_unused_cells() {
    let mut ui = Ui::default();
    let mut cells = Vec::new();
    ui.frame(
        Input::default(),
        Rect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 400.0,
        },
        0.0,
        |f| {
            f.grid(5, 3, 40.0, 10.0, |f, i| cells.push((i, f.available())));
        },
    );
    assert_eq!(cells.len(), 5);
    assert_eq!(
        cells.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4]
    );
    assert_eq!(cells[0].1.x, 0.0);
    assert_eq!(cells[1].1.x, 110.0);
    assert_eq!(cells[2].1.x, 220.0);
    assert_eq!(cells[3].1.y, 50.0);
    assert_eq!(cells[0].1.width, 100.0);
}
#[test]
fn stack_spacing_and_container_width_are_bounded() {
    let mut ui = Ui::default();
    let mut stack = Vec::new();
    let mut child = Rect::default();
    ui.frame(
        Input::default(),
        Rect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 400.0,
        },
        0.0,
        |f| {
            f.stack(3, 8.0, |f, _| stack.push(f.reserve(40.0)));
            f.container(240.0, 60.0, |f| child = f.reserve(60.0));
        },
    );
    assert_eq!(
        stack.iter().map(|r| r.y).collect::<Vec<_>>(),
        [0.0, 48.0, 96.0]
    );
    assert_eq!(child.x, 40.0);
    assert_eq!(child.width, 240.0);
}
#[test]
fn boxed_region_applies_padding_and_clips_children() {
    let mut ui = Ui::default();
    let mut child = Rect::default();
    let commands = ui.frame(
        Input::default(),
        Rect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 400.0,
        },
        0.0,
        |f| {
            f.boxed(100.0, 12.0, 16.0, Color(1, 2, 3, 255), |f| {
                child = f.reserve(20.0)
            });
        },
    );
    assert_eq!((child.x, child.y, child.width), (16.0, 16.0, 288.0));
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, DrawCommand::PushClip(r) if r.x == 16.0 && r.y == 16.0))
    );
}
