use bhoo::*;
const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 400.0,
    height: 300.0,
};
fn draw(ui: &mut Ui, time: f64, value: bool, filled: bool, required: bool) -> Vec<DrawCommand> {
    ui.frame(
        Input {
            time,
            ..Input::default()
        },
        VIEW,
        8.0,
        |f| {
            f.text_field_required(1, "Name", 56.0, value, false, filled, required);
        },
    )
}
fn label(commands: &[DrawCommand]) -> (Rect, TextStyle, Color) {
    commands
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text {
                bounds,
                text,
                style,
                color,
            } if &**text == "Name" => Some((*bounds, *style, *color)),
            _ => None,
        })
        .unwrap()
}
#[test]
fn label_focus_blur_value_and_reversal_follow_presentation_without_losing_state() {
    let mut ui = Ui::default();
    let initial = label(&draw(&mut ui, 0.0, false, false, true));
    assert_eq!(initial.1.size, 16.0);
    assert!(ui.semantics()[0].required);
    ui.adapter_focus(Some(1));
    assert_eq!(label(&draw(&mut ui, 1.0, false, false, true)).1.size, 16.0);
    let mid = label(&draw(&mut ui, 1.060, false, false, true));
    assert!(mid.1.size > 12.0 && mid.1.size < 16.0);
    assert_ne!(mid.2, initial.2);
    ui.adapter_focus(None);
    assert_eq!(
        label(&draw(&mut ui, 1.060, false, false, true)).1.size,
        mid.1.size,
        "Reversing focus must not jump"
    );
    assert_eq!(label(&draw(&mut ui, 1.3, false, false, true)).1.size, 16.0);
    assert!(!ui.needs_redraw());
    ui.adapter_focus(Some(1));
    draw(&mut ui, 2.0, false, false, true);
    assert_eq!(label(&draw(&mut ui, 2.2, true, false, true)).1.size, 12.0);
    ui.adapter_focus(None);
    assert_eq!(
        label(&draw(&mut ui, 2.4, true, false, true)).1.size,
        12.0,
        "A value keeps the label floated on blur"
    );
    draw(&mut ui, 2.6, true, false, true);
    assert!(!ui.needs_redraw());
}
#[test]
fn filled_underline_animates_from_center_and_reduced_motion_snaps() {
    let mut ui = Ui::default();
    draw(&mut ui, 0.0, false, true, false);
    ui.adapter_focus(Some(1));
    draw(&mut ui, 1.0, false, true, false);
    let commands = draw(&mut ui, 1.09, false, true, false);
    let underline = commands
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color, .. }
                if bounds.height == 2.0
                    && color.0 == ui.theme.primary.0
                    && color.1 == ui.theme.primary.1
                    && color.2 == ui.theme.primary.2 =>
            {
                Some((*bounds, *color))
            }
            _ => None,
        })
        .expect("Active underline");
    assert!(underline.0.width > 0.0 && underline.0.width < VIEW.width);
    assert!((underline.0.x + underline.0.width / 2.0 - VIEW.width / 2.0).abs() < 0.01);
    assert!(underline.1.3 > 0 && underline.1.3 < 255);
    ui.motion_preference = MotionPreference::Reduced;
    assert_eq!(label(&draw(&mut ui, 1.1, false, true, false)).1.size, 12.0);
    assert!(!ui.needs_redraw());
    ui.adapter_focus(None);
    assert_eq!(label(&draw(&mut ui, 1.2, false, true, false)).1.size, 16.0);
    assert!(!ui.needs_redraw());
}

#[test]
fn popup_field_reserves_space_for_its_trailing_icon() {
    for filled in [false,true] {
        let mut ui=Ui::default();
        let label="A very long popup field label";
        let value="A very long selected value";
        let mut response=Response::default();
        let commands=ui.frame(Input::default(),VIEW,8.0,|f|{response=f.select_field(1,label,value,filled);});
        for text in [label,value] {
            let bounds=commands.iter().find_map(|command|match command {DrawCommand::Text{bounds,text:drawn,..} if &**drawn==text=>Some(*bounds),_=>None}).unwrap();
            assert!(bounds.x+bounds.width<=response.bounds.x+response.bounds.width-40.0, "{text} must not overlap the trailing icon");
        }
    }
}

#[test]
fn selected_menu_item_draws_a_check_and_aligns_other_labels() {
    let mut ui=Ui::default();
    let commands=ui.frame(Input::default(),VIEW,0.0,|f|{
        f.menu_item_with_selection(1,"First",true,true,false);
        f.menu_item_with_selection(2,"Second",true,false,false);
    });
    let icons:Vec<_>=commands.iter().filter_map(|command| match command {DrawCommand::Shape{color,..}=>Some(*color),_=>None}).collect();
    assert_eq!(icons.len(),2);
    assert_ne!(icons[0].3,0);
    assert_eq!(icons[1].3,0);
    let labels:Vec<_>=commands.iter().filter_map(|command|match command {DrawCommand::Text{bounds,text,..} if &**text=="First"||&**text=="Second"=>Some(bounds.x),_=>None}).collect();
    assert_eq!(labels.len(),2);assert_eq!(labels[0],labels[1]);
}

#[test]
fn text_field_at_paints_value_inside_bounds() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 120.0 };
    let field_rect = Rect { x: 16.0, y: 20.0, width: 260.0, height: 56.0 };
    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        let r = f.text_field_at(99, field_rect, "Subject", "Hello Bhoo", false, true);
        assert_eq!(r.bounds.x, 16.0);
    });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Hello Bhoo")));
}
