use bhoo::{
    android_xml::{Kind, LayoutState, Node, RelativeRule, Size, TintState, Visibility},
    *,
};
#[path = "../../../../../apps/bhoo-web/src/android_textappearance.rs"]
mod android_textappearance;
#[path = "../../../../../apps/bhoo-web/src/android_compound_textappearance.rs"]
mod android_compound_textappearance;

#[test]
fn compound_button_text_appearance_applies_and_preserves_interaction() {
    let bounds=Rect{x:0.,y:0.,width:280.,height:168.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{android_compound_textappearance::LAYOUT.render(f,bounds,&mut state);});
    for (label,size) in [("STYLED CHECKBOX",20.),("STYLED RADIO",20.),("STYLED SWITCH",14.)] {
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,style,color:Color(23,105,170,255),..} if text.as_ref()==label && style.size==size && style.weight==700 && style.letter_spacing>0.0)));
    }
    for input in [Input{pointer:Some([20.,20.]),pressed:true,..Input::default()},Input{pointer:Some([20.,20.]),released:true,..Input::default()}] {ui.frame(input,bounds,0.,|f|{android_compound_textappearance::LAYOUT.render(f,bounds,&mut state);});}
    assert!(!state.checked["appearance_check"]);
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{android_compound_textappearance::LAYOUT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,style,..} if text.as_ref()=="STYLED CHECKBOX" && style.align==2)));
}

#[test]
fn textview_compound_drawable_tint_follows_state_and_multiplies_alpha() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:173,anchor:"16,16,-1,200,120,0,128"},RelativeRule{kind:176,anchor:"0,1,90,90,90,255"},RelativeRule{kind:176,anchor:"2,0,200,0,0,128"},RelativeRule{kind:176,anchor:"8,0,0,160,0,255"},RelativeRule{kind:176,anchor:"0,0,0,0,200,255"}];
    static VIEW:Node=Node{kind:Kind::Text,text:"Tint",clickable:true,relative_rules:RULES,..button("tint_text",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node{enabled:false,..VIEW};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Ellipse{color:Color(0,0,200,128),..})));
    let commands=ui.frame(Input{pointer:Some([50.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Ellipse{color:Color(200,0,0,64),..})));
    ui.frame(Input{pointer:Some([50.,20.]),released:true,..Input::default()},bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    state.view_selected.insert("tint_text".into(),true);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Ellipse{color:Color(0,160,0,128),..})));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Ellipse{color:Color(90,90,90,128),..})));
}

#[test]
fn textview_compound_tint_modes_apply_to_shape_not_text() {
    static ATOP:&[RelativeRule]=&[RelativeRule{kind:173,anchor:"16,16,-1,100,100,100,255"},RelativeRule{kind:176,anchor:"0,0,200,0,0,128"},RelativeRule{kind:177,anchor:"0"}];
    static IN:&[RelativeRule]=&[RelativeRule{kind:173,anchor:"16,16,-1,100,100,100,255"},RelativeRule{kind:176,anchor:"0,0,200,0,0,128"},RelativeRule{kind:177,anchor:"1"}];
    static VIEW:Node=Node{kind:Kind::Text,text:"Tint mode",relative_rules:ATOP,..button("mode",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let atop=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(atop.iter().any(|c|matches!(c,DrawCommand::Ellipse{color:Color(150,50,50,255),..})));
    static INPUT:Node=Node{relative_rules:IN,..VIEW};
    let inside=ui.frame(Input::default(),bounds,0.,|f|{INPUT.render(f,bounds,&mut state);});
    assert!(inside.iter().any(|c|matches!(c,DrawCommand::Ellipse{color:Color(200,0,0,128),..})));
    assert!(inside.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="Tint mode")));
}

#[test]
fn compound_selector_uses_installed_bounds_for_different_sized_states() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:173,anchor:"20,12,0,0,0,200,255"},RelativeRule{kind:182,anchor:"2,0,10,8,2,200,0,0,255"},RelativeRule{kind:182,anchor:"0,0,20,12,0,0,0,200,255"}];
    static VIEW:Node=Node{kind:Kind::Text,text:"Choice",clickable:true,relative_rules:RULES,..button("selector_bounds",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input{pointer:Some([50.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(200,0,0,255),..} if bounds.width==20. && bounds.height==12.)));
}

#[test]
fn compound_drawable_selector_responds_to_pressed_disabled_and_checked() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:173,anchor:"16,16,0,0,0,200,255"},RelativeRule{kind:182,anchor:"0,1,16,16,0,90,90,90,255"},RelativeRule{kind:182,anchor:"2,0,0,0,0,0,0,0,0"},RelativeRule{kind:182,anchor:"64,0,16,16,0,0,160,0,255"},RelativeRule{kind:182,anchor:"0,0,16,16,0,0,0,200,255"}];
    static VIEW:Node=Node{kind:Kind::CheckedText,text:"Choice",checked:true,clickable:true,relative_rules:RULES,..button("choice_selector",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node{enabled:false,..VIEW};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(0,160,0,255),..})));
    let commands=ui.frame(Input{pointer:Some([50.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(0,160,0,255),..}|DrawCommand::RoundedRect{color:Color(0,0,200,255),..})));
    ui.frame(Input{pointer:Some([50.,20.]),released:true,..Input::default()},bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    state.checked.insert("choice_selector".into(),false);state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(0,0,200,255),..} if bounds.x==184.)));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(90,90,90,255),..})));
}

#[test]
fn checked_text_compound_drawable_reserves_check_mark_space_and_follows_rtl() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:173,anchor:"16,16,-1,200,120,0,128"},RelativeRule{kind:175,anchor:"4"},RelativeRule{kind:176,anchor:"64,0,0,0,200,255"},RelativeRule{kind:176,anchor:"0,0,200,0,0,255"}];
    static VIEW:Node=Node{kind:Kind::CheckedText,text:"Choice",checked:true,relative_rules:RULES,..button("choice",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    for rtl in [false,true] {
        state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Ellipse{bounds,color:Color(0,0,200,128)} if bounds.x==if rtl {184.} else {0.})));
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="Choice" && bounds.x==if rtl {32.} else {20.} && bounds.width==148.)));
    }
    state.checked.insert("choice".into(),false);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Ellipse{color:Color(200,0,0,128),..})));
}

#[test]
fn textview_compound_drawables_reserve_space_and_follow_rtl() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:169,anchor:"20,12,2,200,0,0,255"},RelativeRule{kind:170,anchor:"10,8,0,0,200,0,255"},RelativeRule{kind:171,anchor:"20,12,2,0,0,200,255"},RelativeRule{kind:172,anchor:"10,8,0,200,200,0,255"},RelativeRule{kind:175,anchor:"4"}];
    static VIEW:Node=Node{kind:Kind::Text,text:"Hello",relative_rules:RULES,..button("compound",Visibility::Visible,Size::Dp(80.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:80.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    for (color,x,y) in [(Color(200,0,0,255),0.,34.),(Color(0,200,0,255),95.,0.),(Color(0,0,200,255),180.,34.),(Color(200,200,0,255),95.,72.)] {
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:c,..} if *c==color && bounds.x==x && bounds.y==y)));
    }
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="Hello" && bounds.x==24. && bounds.width==152.)));
    static RELATIVE:&[RelativeRule]=&[RelativeRule{kind:169,anchor:"20,12,2,200,0,0,255"},RelativeRule{kind:173,anchor:"12,12,-1,10,20,30,255"},RelativeRule{kind:175,anchor:"4"}];
    static REL:Node=Node{relative_rules:RELATIVE,..VIEW};
    for rtl in [false,true] {
        state.rtl=rtl;let commands=ui.frame(Input::default(),bounds,0.,|f|{REL.render(f,bounds,&mut state);});
        assert!(!commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(200,0,0,255),..})));
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Ellipse{bounds,color:Color(10,20,30,255)} if bounds.x==if rtl{188.}else{0.})));
    }
    static NULL_REL:&[RelativeRule]=&[RelativeRule{kind:169,anchor:"20,12,2,200,0,0,255"},RelativeRule{kind:173,anchor:"0,0,0,0,0,0,0"}];
    static NULL_VIEW:Node=Node{relative_rules:NULL_REL,..VIEW};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{NULL_VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(200,0,0,255),..} if bounds.x==0.)));
    static WRAP:Node=Node{height:Size::Wrap,..VIEW};
    static COLUMN:Node=Node{kind:Kind::Linear,children:&[WRAP,button("after_compound",Visibility::Visible,Size::Dp(40.),0.)],..button("column",Visibility::Visible,Size::Match,0.)};
    let tall=Rect{x:0.,y:0.,width:200.,height:160.};state.rtl=false;
    ui.frame(Input::default(),tall,0.,|f|{COLUMN.render(f,tall,&mut state);});
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after_compound").unwrap().bounds.y,48.);
}

#[test]
fn imported_text_appearance_renders_inherited_and_overridden_styles() {
    let bounds=Rect{x:0.,y:0.,width:240.,height:168.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{android_textappearance::LAYOUT.render(f,bounds,&mut state);});
    for (text,size,color) in [("INHERITED APPEARANCE",20.,Color(23,105,170,255)),("DIRECT SIZE OVERRIDE",14.,Color(23,105,170,255)),("DISABLED APPEARANCE",20.,Color(136,136,136,255))] {
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text:t,style,color:c,..} if t.as_ref()==text && style.size==size && style.weight==700 && *c==color)));
    }
    let commands=ui.frame(Input{pointer:Some([20.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{android_textappearance::LAYOUT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,color:Color(191,56,72,255),..} if text.as_ref()=="INHERITED APPEARANCE")));
}
const fn button(id: &'static str, visibility: Visibility, height: Size, weight: f32) -> Node {
    Node {
        kind: Kind::Button,
        id,
        width: Size::Match,
        height,
        min_width: 0., min_height: 0.,
        horizontal: false,measure_all_children:false,
        layout_direction: 0,
        fill_viewport: false, clip_to_padding_disabled:false,clip_children_disabled:false,
        weight, weight_sum:0., divider_color:None, divider_thickness:0., divider_flags:0, divider_padding:0.,
        padding_edges: [0.; 4],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0], padding_relative: [None; 2],
        margin: [0.; 4],
        margin_relative: [None, None],
        layout_gravity: [0, 0],absolute_position:[0.0,0.0],relative_rules:&[],
        layout_gravity_explicit: false, content_gravity: [0, 0],
        text: id, hint:"", chrono_format:"", count_down:false, clock_12:"", clock_24:"", input_type:0,
        text_on: "ON", text_off: "OFF",
        text_size: 16.,letter_spacing_em:0.0,line_spacing_extra:0.0,line_spacing_multiplier:1.0,
        text_bold: false, text_all_caps: false,
        max_lines: 0,min_lines:1, ellipsize_end: false, ellipsize_side:0, single_line:false,
        align: 0,
        color: None, text_color_states:&[],
        background: None,foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,
        radius: 0.,
        clickable: false,
        icon: MaterialIcon::Menu,
        image_src: "", image_scale: 0,image_adjust_bounds:false,image_max_width:None,image_max_height:None,image_tint:None,image_tint_states:&[],image_tint_mode:0,image_alpha:255,view_alpha:1.0, image_builtin: true,
        enabled: true,
        checked: false,
        checked_button: "",
        min: 0., max: 100., progress: 0., secondary_progress: 0., indeterminate: false, num_stars: 5, rating: 0., step_size: 0.5, indicator: false, mirror_for_rtl: false, animation_in:None, animation_out:None, translate_in:None, translate_out:None, auto_start:false, flip_interval:3.0,
        visibility,
        children: &[],
    }
}
static ROOT: Node = Node {
    kind: Kind::Linear,
    id: "root",
    width: Size::Match,
    height: Size::Match,
    min_width: 0., min_height: 0.,
    horizontal: false,measure_all_children:false,
    layout_direction: 0,
    fill_viewport: false, clip_to_padding_disabled:false,clip_children_disabled:false,
    weight: 0., weight_sum:0., divider_color:None, divider_thickness:0., divider_flags:0, divider_padding:0.,
    padding_edges: [0.; 4],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0], padding_relative: [None; 2],
    margin: [0.; 4],
    margin_relative: [None, None],
    layout_gravity: [0, 0],absolute_position:[0.0,0.0],relative_rules:&[],
    layout_gravity_explicit: false, content_gravity: [0, 0],
    text: "", hint:"", chrono_format:"", count_down:false, clock_12:"", clock_24:"", input_type:0,
    text_on: "ON", text_off: "OFF",
    text_size: 16.,letter_spacing_em:0.0,line_spacing_extra:0.0,line_spacing_multiplier:1.0,
    text_bold: false, text_all_caps: false,
    max_lines: 0,min_lines:1, ellipsize_end: false, ellipsize_side:0, single_line:false,
    align: 0,
    color: None, text_color_states:&[],
    background: None,foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,
    radius: 0.,
    clickable: false,
    icon: MaterialIcon::Menu,
    image_src: "", image_scale: 0,image_adjust_bounds:false,image_max_width:None,image_max_height:None,image_tint:None,image_tint_states:&[],image_tint_mode:0,image_alpha:255,view_alpha:1.0, image_builtin: true,
    enabled: true,
    checked: false,
    checked_button: "",
    min: 0., max: 100., progress: 0., secondary_progress: 0., indeterminate: false, num_stars: 5, rating: 0., step_size: 0.5, indicator: false, mirror_for_rtl: false, animation_in:None, animation_out:None, translate_in:None, translate_out:None, auto_start:false, flip_interval:3.0,
    visibility: Visibility::Visible,
    children: &[
        button("gone", Visibility::Gone, Size::Dp(0.), 1.),
        button("invisible", Visibility::Invisible, Size::Dp(40.), 0.),
        button("visible", Visibility::Visible, Size::Dp(0.), 1.),
    ],
};
#[test]
fn visibility_controls_layout_and_accessibility() {
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect {
        x: 0.,
        y: 0.,
        width: 200.,
        height: 100.,
    };
    ui.frame(Input::default(), bounds, 0., |f| {
        ROOT.render(f, bounds, &mut state);
    });
    let nodes = ui.semantics();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].label.as_ref(), "visible");
    assert_eq!(nodes[0].bounds.y, 40.);
    assert_eq!(nodes[0].bounds.height, 60.);
    state
        .visibility
        .insert("invisible".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| {
        ROOT.render(f, bounds, &mut state);
    });
    assert_eq!(ui.semantics()[0].bounds.y, 0.);
    assert_eq!(ui.semantics()[0].bounds.height, 100.);
    state.visibility.insert("gone".into(), Visibility::Visible);
    ui.frame(Input::default(), bounds, 0., |f| {
        ROOT.render(f, bounds, &mut state);
    });
    let nodes = ui.semantics();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].bounds.height, 50.);
    assert_eq!(nodes[1].bounds.y, 50.);
}

static MARGIN_ROOT: Node = Node {
    horizontal: true,measure_all_children:false,
    children: &[
        Node {
            margin: [10., 3., 5., 7.],
            width: Size::Dp(40.),
            ..button("fixed", Visibility::Visible, Size::Match, 0.)
        },
        Node {
            margin: [5., 3., 10., 7.],
            width: Size::Dp(0.),
            ..button("weighted", Visibility::Visible, Size::Match, 1.)
        },
    ],
    ..ROOT
};
#[test]
fn margins_are_included_in_weighted_layout() {
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let b = Rect {
        x: 0.,
        y: 0.,
        width: 200.,
        height: 100.,
    };
    ui.frame(Input::default(), b, 0., |f| {
        MARGIN_ROOT.render(f, b, &mut state);
    });
    let nodes = ui.semantics();
    assert_eq!(nodes.len(), 2);
    assert_eq!(
        nodes[0].bounds,
        Rect {
            x: 10.,
            y: 3.,
            width: 40.,
            height: 90.
        }
    );
    assert_eq!(
        nodes[1].bounds,
        Rect {
            x: 60.,
            y: 3.,
            width: 130.,
            height: 90.
        }
    );
}

#[test]
fn wrap_content_uses_current_unicode_text() {
    static TEXT: Node = Node {
        kind: Kind::Text,
        width: Size::Wrap,
        text: "a",
        clickable: true,
        ..button("label", Visibility::Visible, Size::Dp(40.), 0.)
    };
    static HORIZONTAL: Node = Node {
        horizontal: true,measure_all_children:false,
        children: &[TEXT],
        ..ROOT
    };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect { x: 0., y: 0., width: 200., height: 100. };
    state.text.insert("label".into(), "éé".into());
    ui.frame(Input::default(), bounds, 0., |f| {
        HORIZONTAL.render(f, bounds, &mut state);
    });
    assert!((ui.semantics()[0].bounds.width - ui.text_metrics.width("éé", TextStyle { size:16., ..TextStyle::default() })).abs() < 0.01);
}

#[path = "../../examples/android_xml/screen.rs"]
mod generated;
#[test]
fn generated_example_renders_includes_and_dynamic_visibility() {
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect {
        x: 0.,
        y: 0.,
        width: 320.,
        height: 600.,
    };
    let commands = ui.frame(Input::default(), bounds, 0., |f| {
        generated::LAYOUT.render(f, bounds, &mut state);
    });
    let text = |commands: &[DrawCommand], expected: &str| {
        commands
            .iter()
            .any(|c| matches!(c,DrawCommand::Text{text,..} if text.as_ref()==expected))
    };
    assert!(text(&commands, "Bhoo workspace"));
    assert!(text(&commands, "A reusable Android XML layout"));
    assert!(!text(&commands, "Ready"));
    assert!(
        ui.semantics()
            .iter()
            .any(|n| n.label.as_ref() == "Continue")
    );
    state
        .visibility
        .insert("status".into(), Visibility::Visible);
    let commands = ui.frame(Input::default(), bounds, 0., |f| {
        generated::LAYOUT.render(f, bounds, &mut state);
    });
    assert!(text(&commands, "Ready"));
}

#[test]
fn horizontal_scroll_moves_content_and_clamps_wheel() {
    static ITEM: Node = Node { width: Size::Dp(300.), ..button("wide", Visibility::Visible, Size::Dp(50.), 0.) };
    static SCROLL: Node = Node { kind: Kind::HorizontalScroll, id: "horizontal", width: Size::Match, height: Size::Match, children: &[ITEM], ..button("horizontal", Visibility::Visible, Size::Match, 0.) };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect { x: 0., y: 0., width: 100., height: 60. };
    let render = |ui: &mut Ui, state: &mut LayoutState, input: Input| ui.frame(input, bounds, 0., |f| { SCROLL.render(f, bounds, state); });
    render(&mut ui, &mut state, Input::default());
    render(&mut ui, &mut state, Input { pointer: Some([50., 20.]), scroll: [500., 0.], ..Input::default() });
    assert_eq!(state.offsets["horizontal"], 200.);
    let item = ui.semantics().iter().find(|n| n.label.as_ref() == "wide").unwrap();
    assert_eq!(item.bounds.width, 100.); // The visible part is clipped to the viewport.
    render(&mut ui, &mut state, Input { pointer: Some([50., 20.]), scroll: [-500., 0.], ..Input::default() });
    assert_eq!(state.offsets["horizontal"], 0.);
    render(&mut ui, &mut state, Input { pointer: Some([80., 20.]), pointer_kind: PointerKind::Touch, pressed: true, time: 1.0, ..Input::default() });
    render(&mut ui, &mut state, Input { pointer: Some([30., 20.]), pointer_kind: PointerKind::Touch, time: 1.1, ..Input::default() });
    assert!(state.offsets["horizontal"] > 0.);
}

#[test]
fn android_checkbox_preserves_initial_state_and_toggles() {
    static CHECK: Node = Node { kind: Kind::CheckBox, checked: true, ..button("remember", Visibility::Visible, Size::Dp(48.), 0.) };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect { x: 0., y: 0., width: 220., height: 48. };
    let render = |ui: &mut Ui, state: &mut LayoutState, input: Input| {
        let mut action = None;
        ui.frame(input, bounds, 0., |f| { action = CHECK.render(f, bounds, state); });
        action
    };
    render(&mut ui, &mut state, Input::default());
    assert!(state.checked["remember"]);
    render(&mut ui, &mut state, Input { pointer: Some([15., 15.]), pressed: true, ..Input::default() });
    let action = render(&mut ui, &mut state, Input { pointer: Some([15., 15.]), released: true, ..Input::default() });
    assert_eq!(action, Some("remember"));
    assert!(!state.checked["remember"]);
    render(&mut ui, &mut state, Input::default());
    assert!(!state.checked["remember"]);
}

#[test]
fn android_switch_toggles_and_disabled_switch_stays_unchanged() {
    static ENABLED: Node = Node { kind: Kind::Switch, checked: true, ..button("wifi", Visibility::Visible, Size::Dp(48.), 0.) };
    static DISABLED: Node = Node { kind: Kind::Switch, enabled: false, checked: false, ..button("disabled", Visibility::Visible, Size::Dp(48.), 0.) };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect { x: 0., y: 0., width: 240., height: 48. };
    let render = |ui: &mut Ui, state: &mut LayoutState, node: &'static Node, input: Input| {
        ui.frame(input, bounds, 0., |f| { node.render(f, bounds, state); });
    };
    render(&mut ui, &mut state, &ENABLED, Input::default());
    assert!(state.checked["wifi"]);
    for input in [Input { pointer: Some([220., 15.]), pressed: true, ..Input::default() }, Input { pointer: Some([220., 15.]), released: true, ..Input::default() }] {
        render(&mut ui, &mut state, &ENABLED, input);
    }
    assert!(!state.checked["wifi"]);
    for input in [Input { pointer: Some([220., 15.]), pressed: true, ..Input::default() }, Input { pointer: Some([220., 15.]), released: true, ..Input::default() }] {
        render(&mut ui, &mut state, &DISABLED, input);
    }
    assert!(!state.checked["disabled"]);
}

#[test]
fn seekbar_uses_xml_initial_value_and_drag_updates_state() {
    static SEEK: Node = Node { kind: Kind::SeekBar, min: 10., max: 90., progress: 25., ..button("volume", Visibility::Visible, Size::Dp(48.), 0.) };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect { x: 0., y: 0., width: 200., height: 48. };
    let render = |ui: &mut Ui, state: &mut LayoutState, input: Input| {
        ui.frame(input, bounds, 0., |f| { SEEK.render(f, bounds, state); });
    };
    render(&mut ui, &mut state, Input::default());
    assert_eq!(state.progress["volume"], 25.);
    render(&mut ui, &mut state, Input { pointer: Some([190., 20.]), pressed: true, ..Input::default() });
    render(&mut ui, &mut state, Input { pointer: Some([190., 20.]), released: true, ..Input::default() });
    assert_eq!(state.progress["volume"], 90.);
    render(&mut ui, &mut state, Input::default());
    assert_eq!(state.progress["volume"], 90.);
}

#[test]
fn seekbar_tick_drawable_selector_tracks_press_and_disabled() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:167,anchor:"0,1,6,6,3,90,90,90,255"},RelativeRule{kind:167,anchor:"2,0,8,8,0,200,10,20,255"},RelativeRule{kind:167,anchor:"0,0,6,6,3,10,20,30,255"}];
    static BAR:Node=Node{kind:Kind::SeekBar,min:0.,max:4.,progress:2.,relative_rules:RULES,..button("tick_selector",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node{enabled:false,..BAR};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let count=|commands:&[DrawCommand],color|commands.iter().filter(|c|matches!(c,DrawCommand::RoundedRect{color:c,..} if *c==color)).count();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BAR.render(f,bounds,&mut state);});assert_eq!(count(&commands,Color(10,20,30,255)),5);
    let commands=ui.frame(Input{pointer:Some([100.,24.]),pressed:true,..Input::default()},bounds,0.,|f|{BAR.render(f,bounds,&mut state);});assert_eq!(count(&commands,Color(200,10,20,255)),5);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});assert_eq!(count(&commands,Color(90,90,90,255)),5);
    static NONE:&[RelativeRule]=&[RelativeRule{kind:167,anchor:"0,0,0,0,0,0,0,0,0"}];
    static NULL:Node=Node{relative_rules:NONE,..BAR};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{NULL.render(f,bounds,&mut state);});assert_eq!(count(&commands,Color(10,20,30,255)),0);
    static TINT:&[RelativeRule]=&[RelativeRule{kind:167,anchor:"0,0,6,6,3,10,20,30,255"},RelativeRule{kind:154,anchor:"0,0,50,60,70,255"}];
    static TINTED:Node=Node{relative_rules:TINT,..BAR};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{TINTED.render(f,bounds,&mut state);});assert_eq!(count(&commands,Color(50,60,70,255)),5);
}

#[test]
fn seekbar_custom_thumb_and_tick_tint_modes_compose_independently() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:166,anchor:"0,0,20,12,3,100,100,100,255"},RelativeRule{kind:167,anchor:"0,0,6,6,3,100,100,100,255"},RelativeRule{kind:153,anchor:"0,0,200,0,0,128"},RelativeRule{kind:154,anchor:"0,0,0,0,200,128"},RelativeRule{kind:186,anchor:"0"},RelativeRule{kind:187,anchor:"1"}];
    static BAR:Node=Node{kind:Kind::SeekBar,min:0.,max:4.,progress:2.,relative_rules:RULES,..button("tint_modes",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BAR.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(150,50,50,255),..} if bounds.width==20. && bounds.height==12.)));
    assert_eq!(commands.iter().filter(|c|matches!(c,DrawCommand::RoundedRect{color:Color(0,0,200,128),..})).count(),5);
}

#[test]
fn seekbar_custom_thumb_and_tick_tints_multiply_drawable_alpha() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:166,anchor:"0,0,20,12,3,10,20,30,128"},RelativeRule{kind:167,anchor:"0,0,6,6,3,10,20,30,128"},RelativeRule{kind:153,anchor:"0,0,200,0,0,128"},RelativeRule{kind:154,anchor:"0,0,0,0,200,128"}];
    static BAR:Node=Node{kind:Kind::SeekBar,min:0.,max:4.,progress:2.,relative_rules:RULES,..button("tint_alpha",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BAR.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(200,0,0,64),..} if bounds.width==20. && bounds.height==12.)));
    assert_eq!(commands.iter().filter(|c|matches!(c,DrawCommand::RoundedRect{color:Color(0,0,200,64),..})).count(),5);
}

#[test]
fn seekbar_custom_thumb_selector_and_split_use_drawable_size() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:163,anchor:""},RelativeRule{kind:166,anchor:"2,0,30,12,2,200,10,20,255"},RelativeRule{kind:166,anchor:"0,0,24,12,6,10,20,30,255"}];
    static BAR:Node=Node{kind:Kind::SeekBar,progress:50.,relative_rules:RULES,..button("custom_thumb",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BAR.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(10,20,30,255),..} if bounds.width==24. && bounds.height==12.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::PushClip(r) if (r.x+r.width-88.).abs()<0.1)));
    let commands=ui.frame(Input{pointer:Some([100.,24.]),pressed:true,..Input::default()},bounds,0.,|f|{BAR.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(200,10,20,255),..} if bounds.width==30.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::PushClip(r) if (r.x+r.width-85.).abs()<0.1)));
    static NULL_RULES:&[RelativeRule]=&[RelativeRule{kind:163,anchor:""},RelativeRule{kind:166,anchor:"0,0,0,0,0,0,0,0,0"}];
    static NULL:Node=Node{relative_rules:NULL_RULES,..BAR};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{NULL.render(f,bounds,&mut state);});
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::PushClip(r) if r.width<bounds.width || r.height<bounds.height)));
    static PRESSED_ONLY:&[RelativeRule]=&[RelativeRule{kind:166,anchor:"2,0,24,12,6,10,20,30,255"}];
    static MISSING:Node=Node{relative_rules:PRESSED_ONLY,..BAR};
    ui.frame(Input{pointer:Some([100.,24.]),released:true,..Input::default()},bounds,0.,|f|{BAR.render(f,bounds,&mut state);});
    let commands=ui.frame(Input::default(),bounds,0.,|f|{MISSING.render(f,bounds,&mut state);});
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(10,20,30,255),..})));
}

#[test]
fn seekbar_split_track_excludes_thumb_area_and_mirrors() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:163,anchor:""}];
    static BAR:Node=Node{kind:Kind::SeekBar,min:0.,max:100.,progress:25.,mirror_for_rtl:true,relative_rules:RULES,..button("split",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    for rtl in [false,true] {
        state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f|{BAR.render(f,bounds,&mut state);});
        let gap=if rtl{11.+178.*0.75}else{11.+178.*0.25};
        let clip_regions:Vec<_>=commands.iter().filter_map(|c|match c{DrawCommand::PushClip(r) if r.width<bounds.width || r.height<bounds.height=>Some(*r),_=>None}).collect();
        assert!(clip_regions.iter().any(|r|(r.x+r.width-(gap-10.)).abs()<0.1));
        assert!(clip_regions.iter().any(|r|(r.x-(gap+10.)).abs()<0.1));
        assert!(!clip_regions.iter().any(|r|r.x<gap && r.x+r.width>gap && r.y<24. && r.y+r.height>24.));
    }
    static TALL_RULES:&[RelativeRule]=&[RelativeRule{kind:163,anchor:""},RelativeRule{kind:155,anchor:"4,30,0,10,20,30,255"}];
    static TALL:Node=Node{relative_rules:TALL_RULES,..BAR};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{TALL.render(f,bounds,&mut state);});
    assert!(commands.iter().filter(|c|matches!(c,DrawCommand::PushClip(r) if r.width==20. && r.height>0.)).count()>=2);
}

#[test]
fn seekbar_ticks_follow_integer_range_and_appear_behind_thumb() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:155,anchor:"6,6,3,10,20,30,255"},RelativeRule{kind:154,anchor:"0,0,200,30,40,255"}];
    static BAR:Node=Node{kind:Kind::SeekBar,min:2.,max:6.,progress:4.,relative_rules:RULES,..button("seek_ticks",Visibility::Visible,Size::Dp(48.),0.)};
    let mut ui=Ui::default();let mut state=LayoutState::default();let bounds=Rect{x:0.,y:0.,width:200.,height:48.};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BAR.render(f,bounds,&mut state);});
    let ticks:Vec<_>=commands.iter().filter_map(|c|match c{DrawCommand::RoundedRect{bounds,color:Color(200,30,40,255),..}=>Some(*bounds),_=>None}).collect();
    assert_eq!(ticks.len(),5);
    assert!((ticks[1].x-ticks[0].x-(ticks[4].x-ticks[0].x)/4.).abs()<0.01);
    let last_tick=commands.iter().rposition(|c|matches!(c,DrawCommand::RoundedRect{color:Color(200,30,40,255),..})).unwrap();
    assert!(commands.iter().skip(last_tick+1).any(|c|matches!(c,DrawCommand::RoundedRect{bounds,..} if bounds.width==20.)));
}

#[test]
fn ratingbar_xml_tints_color_fractional_secondary_and_preserve_drag() {
    static RULES:&[RelativeRule]=&[
        RelativeRule{kind:149,anchor:"2,0,200,10,20,255"},RelativeRule{kind:149,anchor:"0,0,10,20,30,255"},
        RelativeRule{kind:150,anchor:"0,0,40,50,60,255"},RelativeRule{kind:151,anchor:"0,0,70,80,90,255"},
    ];
    static RATING:Node=Node {kind:Kind::RatingBar,rating:2.5,relative_rules:RULES,..button("stars",Visibility::Visible,Size::Dp(48.),0.)};
    let mut ui=Ui::default();let mut state=LayoutState::default();let bounds=Rect{x:0.,y:0.,width:200.,height:48.};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{RATING.render(f,bounds,&mut state);});
    let count=|color|commands.iter().filter(|c|matches!(c,DrawCommand::Shape{color:c,..} if *c==color)).count();
    assert_eq!(count(Color(40,50,60,255)),5);
    assert_eq!(count(Color(70,80,90,255)),3);
    assert_eq!(count(Color(10,20,30,255)),3);
    let commands=ui.frame(Input{pointer:Some([180.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{RATING.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape{color:Color(200,10,20,255),..})));
    assert!(state.ratings["stars"]>4.);
}

#[test]
fn seekbar_xml_tints_preserve_interaction_and_disabled_selector() {
    static RULES:&[RelativeRule]=&[
        RelativeRule{kind:149,anchor:"0,1,90,80,70,255"}, RelativeRule{kind:149,anchor:"2,0,140,30,40,255"}, RelativeRule{kind:149,anchor:"0,0,10,20,30,255"},
        RelativeRule{kind:150,anchor:"0,0,40,50,60,255"}, RelativeRule{kind:151,anchor:"0,0,70,80,90,255"},
        RelativeRule{kind:153,anchor:"0,0,100,110,120,255"},
    ];
    static SEEKBAR:Node=Node {kind:Kind::SeekBar, progress:30., secondary_progress:70., relative_rules:RULES,..button("seek",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node {enabled:false,..SEEKBAR};
    let mut ui=Ui::default();let mut state=LayoutState::default();let bounds=Rect{x:0.,y:0.,width:200.,height:48.};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{SEEKBAR.render(f,bounds,&mut state);});
    for color in [Color(10,20,30,255),Color(40,50,60,255),Color(70,80,90,255),Color(100,110,120,255)] {
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:c,..} if *c==color)));
    }
    let commands=ui.frame(Input{pointer:Some([180.,24.]),pressed:true,..Input::default()},bounds,0.,|f|{SEEKBAR.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(140,30,40,255),..})));
    assert!(state.progress["seek"]>80.);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(90,80,70,255),..})));
}

#[test]
fn progressbar_xml_tints_apply_to_layers_and_selected_state() {
    static RULES:&[RelativeRule]=&[
        RelativeRule{kind:149,anchor:"8,0,10,20,30,255"},
        RelativeRule{kind:149,anchor:"0,0,40,50,60,255"},
        RelativeRule{kind:150,anchor:"0,0,70,80,90,255"},
        RelativeRule{kind:151,anchor:"0,0,100,110,120,255"},
        RelativeRule{kind:152,anchor:"0,0,130,140,150,255"},
    ];
    static LINEAR:Node=Node { kind:Kind::ProgressBar, horizontal:true, progress:30., secondary_progress:70., relative_rules:RULES, ..button("download",Visibility::Visible,Size::Dp(48.),0.) };
    static SPINNER:Node=Node { kind:Kind::ProgressBar, indeterminate:true, relative_rules:RULES, ..button("loading",Visibility::Visible,Size::Dp(48.),0.) };
    let mut ui=Ui::default(); let mut state=LayoutState::default();
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};
    let draw=|ui:&mut Ui,state:&mut LayoutState| ui.frame(Input::default(),bounds,0.,|f| { LINEAR.render(f,bounds,state); }).to_vec();
    let colors=|commands:&[DrawCommand],color| commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect{color:c,..} if *c==color));
    let commands=draw(&mut ui,&mut state);
    for color in [Color(40,50,60,255),Color(70,80,90,255),Color(100,110,120,255)] { assert!(colors(&commands,color)); }
    state.view_selected.insert("download".into(),true);
    assert!(colors(&draw(&mut ui,&mut state),Color(10,20,30,255)));
    let commands=ui.frame(Input::default(),bounds,0.,|f| {SPINNER.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Arc{color:Color(130,140,150,255),..})));
}

#[test]
fn progressbar_renders_determinate_and_animates_indeterminate() {
    static LINEAR: Node = Node { kind: Kind::ProgressBar, horizontal: true,measure_all_children:false, progress: 40., secondary_progress: 70., ..button("download", Visibility::Visible, Size::Dp(48.), 0.) };
    static SPINNER: Node = Node { kind: Kind::ProgressBar, indeterminate: true, ..button("loading", Visibility::Visible, Size::Dp(48.), 0.) };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect { x: 0., y: 0., width: 200., height: 48. };
    let commands = ui.frame(Input::default(), bounds, 0., |f| { LINEAR.render(f, bounds, &mut state); });
    assert_eq!(state.progress["download"], 40.);
    assert_eq!(state.secondary_progress["download"], 70.);
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, .. } if (bounds.width - 140.).abs() < 0.1)));
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, .. } if (bounds.width - 80.).abs() < 0.1)));
    let commands = ui.frame(Input::default(), bounds, 0., |f| { SPINNER.render(f, bounds, &mut state); });
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::Arc { .. })));
    assert!(ui.needs_redraw());
}

#[test]
fn radio_group_selects_one_choice_and_rejects_disabled_choice() {
    static GROUP: Node = Node { kind: Kind::RadioGroup, id: "group", children: &[
        Node { kind: Kind::RadioButton, checked: true, ..button("one", Visibility::Visible, Size::Dp(48.), 0.) },
        Node { kind: Kind::RadioButton, ..button("two", Visibility::Visible, Size::Dp(48.), 0.) },
        Node { kind: Kind::RadioButton, enabled: false, ..button("three", Visibility::Visible, Size::Dp(48.), 0.) },
    ], ..button("group", Visibility::Visible, Size::Match, 0.) };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let bounds = Rect { x: 0., y: 0., width: 200., height: 144. };
    let render = |ui: &mut Ui, state: &mut LayoutState, input: Input| ui.frame(input, bounds, 0., |f| { GROUP.render(f, bounds, state); });
    render(&mut ui, &mut state, Input::default());
    assert!(state.checked["one"]);assert!(!state.checked["two"]);
    for input in [Input { pointer: Some([20., 60.]), pressed: true, ..Input::default() }, Input { pointer: Some([20., 60.]), released: true, ..Input::default() }] { render(&mut ui, &mut state, input); }
    assert!(!state.checked["one"]);assert!(state.checked["two"]);
    for input in [Input { pointer: Some([20., 110.]), pressed: true, ..Input::default() }, Input { pointer: Some([20., 110.]), released: true, ..Input::default() }] { render(&mut ui, &mut state, input); }
    assert!(state.checked["two"]);assert!(!state.checked["three"]);
}

#[test]
fn radio_group_checked_button_sets_initial_selection() {
    static GROUP: Node = Node { kind: Kind::RadioGroup, id: "group", checked_button: "two", children: &[
        Node { kind: Kind::RadioButton, ..button("one", Visibility::Visible, Size::Dp(48.), 0.) },
        Node { kind: Kind::RadioButton, ..button("two", Visibility::Visible, Size::Dp(48.), 0.) },
    ], ..button("group", Visibility::Visible, Size::Match, 0.) };
    let mut ui = Ui::default();let mut state = LayoutState::default();
    let bounds = Rect { x: 0., y: 0., width: 200., height: 96. };
    ui.frame(Input::default(), bounds, 0., |f| { GROUP.render(f, bounds, &mut state); });
    assert!(!state.checked["one"]);assert!(state.checked["two"]);
}

#[test]
fn rating_bar_selects_half_stars_and_indicator_is_read_only() {
    static RATING: Node = Node { kind: Kind::RatingBar, rating: 2.5, ..button("rating", Visibility::Visible, Size::Dp(48.), 0.) };
    static INDICATOR: Node = Node { kind: Kind::RatingBar, indicator: true, rating: 3.5, ..button("indicator", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds = Rect { x: 0., y: 0., width: 180., height: 48. };
    let mut ui = Ui::default();let mut state = LayoutState::default();
    let render = |ui: &mut Ui, state: &mut LayoutState, node: &'static Node, input: Input| ui.frame(input, bounds, 0., |f| { node.render(f, bounds, state); });
    let commands=render(&mut ui, &mut state, &RATING, Input::default());
    assert_eq!(state.ratings["rating"], 2.5);
    assert!(commands.iter().any(|cmd| matches!(cmd, DrawCommand::Shape { index, .. } if shape_data(*index as usize).is_some_and(|shape| shape.name == "rating-star-filled"))));
    assert!(!commands.iter().any(|cmd| matches!(cmd, DrawCommand::Text { text, .. } if text.as_ref() == "★" || text.as_ref() == "☆")));
    assert!(commands.iter().any(|cmd| matches!(cmd, DrawCommand::PushClip(r) if (r.width - 18.0).abs() < 0.1)));
    for input in [Input { pointer: Some([113., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([113., 20.]), released: true, ..Input::default() }] { render(&mut ui, &mut state, &RATING, input); }
    assert_eq!(state.ratings["rating"], 3.5);
    for input in [Input { pointer: Some([10., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([10., 20.]), released: true, ..Input::default() }] { render(&mut ui, &mut state, &INDICATOR, input); }
    assert_eq!(state.ratings["indicator"], 3.5);
}

#[test]
fn toggle_button_applies_text_style_and_disabled_indicator_alpha() {
    static FULL:&[RelativeRule]=&[RelativeRule{kind:162,anchor:"1"}];
    static NONE:&[RelativeRule]=&[RelativeRule{kind:162,anchor:"0"}];
    static ENABLED:Node=Node{kind:Kind::ToggleButton,checked:true,text_on:"ready",text_all_caps:true,text_bold:true,..button("on",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED_FULL:Node=Node{enabled:false,relative_rules:FULL,..ENABLED};
    static DISABLED_NONE:Node=Node{enabled:false,relative_rules:NONE,..ENABLED};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();let primary=ui.theme.primary;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{ENABLED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,style,..} if text.as_ref()=="READY" && style.weight==700)));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED_FULL.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color,..} if bounds.height==2. && *color==primary)));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED_NONE.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(_,_,_,0),..} if bounds.height==2.)));
}

#[test]
fn toggle_button_displays_and_retains_changed_label() {
    static TOGGLE: Node = Node { kind: Kind::ToggleButton, text_on: "Enabled", text_off: "Paused", ..button("power", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds = Rect { x: 0., y: 0., width: 200., height: 48. };
    let mut ui = Ui::default();let mut state = LayoutState::default();
    let render = |ui: &mut Ui, state: &mut LayoutState, input: Input| ui.frame(input, bounds, 0., |f| { TOGGLE.render(f, bounds, state); });
    let commands=render(&mut ui, &mut state, Input::default());
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref()=="Paused")));
    for input in [Input { pointer: Some([20., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([20., 20.]), released: true, ..Input::default() }] { render(&mut ui, &mut state, input); }
    assert!(state.checked["power"]);
    let commands=render(&mut ui, &mut state, Input::default());
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref()=="Enabled")));
}

#[test]
fn frame_layout_positions_children_using_gravity_and_margins() {
    static FRAME: Node = Node { kind: Kind::Frame, id: "frame", children: &[
        Node { width: Size::Dp(40.), height: Size::Dp(20.), layout_gravity: [1,1],absolute_position:[0.0,0.0], margin: [4., 3., 2., 1.], ..button("center", Visibility::Visible, Size::Dp(20.), 0.) },
        Node { width: Size::Dp(30.), height: Size::Dp(20.), layout_gravity: [2,2],absolute_position:[0.0,0.0], margin: [0., 0., 5., 7.], ..button("end", Visibility::Visible, Size::Dp(20.), 0.) },
    ], ..button("frame", Visibility::Visible, Size::Match, 0.) };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    let bounds=Rect { x:0., y:0., width:200., height:100. };
    ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
    let center=ui.semantics().iter().find(|n| n.label.as_ref()=="center").unwrap();
    assert_eq!(center.bounds, Rect { x:82., y:42., width:40., height:20. });
    let end=ui.semantics().iter().find(|n| n.label.as_ref()=="end").unwrap();
    assert_eq!(end.bounds, Rect { x:165., y:73., width:30., height:20. });
}

#[test]
fn frame_gravity_resolves_relative_but_not_absolute_sides_in_rtl() {
    static FRAME: Node = Node { kind: Kind::Frame, children: &[
        Node { width: Size::Dp(40.), height: Size::Dp(20.), layout_gravity: [0,0],absolute_position:[0.0,0.0], ..button("start", Visibility::Visible, Size::Dp(20.), 0.) },
        Node { width: Size::Dp(40.), height: Size::Dp(20.), layout_gravity: [2,1],absolute_position:[0.0,0.0], ..button("end", Visibility::Visible, Size::Dp(20.), 0.) },
        Node { width: Size::Dp(40.), height: Size::Dp(20.), layout_gravity: [3,2],absolute_position:[0.0,0.0], ..button("left", Visibility::Visible, Size::Dp(20.), 0.) },
    ], ..button("frame", Visibility::Visible, Size::Match, 0.) };
    let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=true;
    let bounds=Rect { x:0., y:0., width:200., height:100. };
    ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
    let x=|label: &str| ui.semantics().iter().find(|n| n.label.as_ref()==label).unwrap().bounds.x;
    assert_eq!(x("start"),160.);assert_eq!(x("end"),0.);assert_eq!(x("left"),0.);
}

#[test]
fn frame_relative_margins_follow_direction_and_override_physical_side() {
    static FRAME: Node = Node { kind: Kind::Frame, children: &[
        Node { width: Size::Dp(40.), height: Size::Dp(20.), margin: [5.,0.,7.,0.], margin_relative: [Some(20.),None], layout_gravity: [0,0],absolute_position:[0.0,0.0], ..button("start", Visibility::Visible, Size::Dp(20.), 0.) },
        Node { width: Size::Dp(40.), height: Size::Dp(20.), margin: [5.,0.,7.,0.], margin_relative: [None,Some(10.)], layout_gravity: [2,2],absolute_position:[0.0,0.0], ..button("end", Visibility::Visible, Size::Dp(20.), 0.) },
    ], ..button("frame", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
    let x=|ui: &Ui,label: &str| ui.semantics().iter().find(|n| n.label.as_ref()==label).unwrap().bounds.x;
    assert_eq!(x(&ui,"start"),20.);assert_eq!(x(&ui,"end"),150.);
    state.rtl=true;
    ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
    assert_eq!(x(&ui,"start"),140.);assert_eq!(x(&ui,"end"),10.);
}

#[test]
fn horizontal_linear_layout_reverses_children_and_resolves_margins_in_rtl() {
    static ROW: Node = Node { kind: Kind::Linear, horizontal: true,measure_all_children:false, children: &[
        Node { width: Size::Dp(40.), margin_relative: [Some(10.), Some(5.)], ..button("first", Visibility::Visible, Size::Dp(40.), 0.) },
        Node { width: Size::Dp(0.), weight: 1., margin_relative: [Some(5.), Some(10.)], ..button("second", Visibility::Visible, Size::Dp(40.), 0.) },
    ], ..button("row", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:50. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { ROW.render(f,bounds,&mut state); });
    let geometry=|ui: &Ui,label: &str| ui.semantics().iter().find(|n| n.label.as_ref()==label).unwrap().bounds;
    assert_eq!(geometry(&ui,"first").x,10.);assert_eq!(geometry(&ui,"second").x,60.);
    assert_eq!(geometry(&ui,"second").width,130.);
    state.rtl=true;
    ui.frame(Input::default(),bounds,0.,|f| { ROW.render(f,bounds,&mut state); });
    assert_eq!(geometry(&ui,"first").x,150.);assert_eq!(geometry(&ui,"second").x,10.);
    assert_eq!(geometry(&ui,"second").width,130.);
}

#[test]
fn linear_layout_honors_main_and_cross_axis_gravity() {
    static ROW: Node = Node { kind: Kind::Linear, horizontal: true,measure_all_children:false, content_gravity: [1,2], children: &[
        Node { width: Size::Dp(40.), height: Size::Dp(20.), ..button("first", Visibility::Visible, Size::Dp(20.), 0.) },
        Node { width: Size::Dp(40.), height: Size::Dp(20.), layout_gravity: [0,1],absolute_position:[0.0,0.0], layout_gravity_explicit: true, ..button("second", Visibility::Visible, Size::Dp(20.), 0.) },
    ], ..button("row", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { ROW.render(f,bounds,&mut state); });
    let geometry=|ui: &Ui,label: &str| ui.semantics().iter().find(|n| n.label.as_ref()==label).unwrap().bounds;
    assert_eq!(geometry(&ui,"first"),Rect { x:60., y:80., width:40., height:20. });
    assert_eq!(geometry(&ui,"second"),Rect { x:100., y:40., width:40., height:20. });
    state.rtl=true;
    ui.frame(Input::default(),bounds,0.,|f| { ROW.render(f,bounds,&mut state); });
    assert_eq!(geometry(&ui,"first").x,100.);assert_eq!(geometry(&ui,"second").x,60.);
}

#[test]
fn weighted_linear_children_shrink_when_content_exceeds_available_space() {
    static ROW: Node = Node { kind: Kind::Linear, horizontal: true,measure_all_children:false, children: &[
        Node { width: Size::Dp(120.), weight: 1., ..button("first", Visibility::Visible, Size::Dp(40.), 0.) },
        Node { width: Size::Dp(120.), weight: 1., ..button("second", Visibility::Visible, Size::Dp(40.), 0.) },
    ], ..button("row", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:50. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { ROW.render(f,bounds,&mut state); });
    let geometry=|label: &str| ui.semantics().iter().find(|n| n.label.as_ref()==label).unwrap().bounds;
    assert_eq!(geometry("first").width,100.);
    assert_eq!(geometry("second").x,100.);
    assert_eq!(geometry("second").width,100.);
}

#[test]
fn nested_layout_direction_is_scoped_to_the_view() {
    static ROOT: Node = Node { kind: Kind::Linear, children: &[
        Node { kind: Kind::Linear, horizontal:true,measure_all_children:false, layout_direction:2, height:Size::Dp(40.), children: &[
            Node { width:Size::Dp(40.), ..button("rtl-first", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { width:Size::Dp(40.), ..button("rtl-second", Visibility::Visible, Size::Dp(40.), 0.) },
        ], ..button("rtl-row", Visibility::Visible, Size::Dp(40.), 0.) },
        Node { kind: Kind::Linear, horizontal:true,measure_all_children:false, height:Size::Dp(40.), children: &[
            Node { width:Size::Dp(40.), ..button("inherited-first", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { width:Size::Dp(40.), ..button("inherited-second", Visibility::Visible, Size::Dp(40.), 0.) },
        ], ..button("inherited-row", Visibility::Visible, Size::Dp(40.), 0.) },
    ], ..button("root", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:80. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let x=|label: &str| ui.semantics().iter().find(|n| n.label.as_ref()==label).unwrap().bounds.x;
    assert_eq!(x("rtl-first"),160.);assert_eq!(x("rtl-second"),120.);
    assert_eq!(x("inherited-first"),0.);assert_eq!(x("inherited-second"),40.);
    assert!(!state.rtl);
}

#[test]
fn image_view_fit_and_crop_use_supplied_drawable() {
    static FIT: Node = Node { kind: Kind::Image, image_src: "photo", ..button("fit", Visibility::Visible, Size::Match, 0.) };
    static CROP: Node = Node { kind: Kind::Image, image_src: "photo", image_scale:1,image_alpha:255,view_alpha:1.0, ..button("crop", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:100., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("photo".into(), Image::rgba(4,2,vec![255;32]).unwrap());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { FIT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Image { bounds, uv, .. } if bounds.width==100. && bounds.height==50. && *uv==[0.,0.,1.,1.])));
    let commands=ui.frame(Input::default(),bounds,0.,|f| { CROP.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Image { bounds, uv, .. } if bounds.width==100. && bounds.height==100. && *uv==[0.25,0.,0.75,1.])));
    state.images.clear();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { FIT.render(f,bounds,&mut state); });
    assert!(!commands.iter().any(|cmd| matches!(cmd,DrawCommand::Image { .. })));
}

#[test]
fn image_matrix_scale_translation_and_flip_follow_view_bounds() {
    static VIEW:Node=Node{kind:Kind::Image,image_src:"matrix_photo",image_scale:7,padding_edges:[10.;4],padding_explicit:[true;4],..button("matrix_view",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:100.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("matrix_photo".into(),Image::rgba(20,10,vec![255;20*10*4]).unwrap());
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image{bounds,uv,..} if bounds.x==10. && bounds.y==10. && bounds.width==20. && bounds.height==10. && *uv==[0.,0.,1.,1.])));
    state.image_matrices.insert("matrix_view".into(),[2.,3.,5.,7.]);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image{bounds,..} if bounds.x==15. && bounds.y==17. && bounds.width==40. && bounds.height==30.)));
    state.image_matrices.insert("matrix_view".into(),[-1.,1.,20.,0.]);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image{uv,..} if *uv==[1.,0.,0.,1.])));
    state.image_matrices.insert("matrix_view".into(),[0.,1.,0.,0.]);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::Image{..})));
}

#[test]
fn image_crop_to_padding_controls_center_and_center_crop_overflow() {
    static CROP_RULE:&[RelativeRule]=&[RelativeRule{kind:188,anchor:""}];
    static VIEW:Node=Node{kind:Kind::Image,image_src:"wide",image_scale:1,padding_edges:[20.;4],padding_explicit:[true;4],..button("image_crop",Visibility::Visible,Size::Match,0.)};
    static CLIPPED:Node=Node{relative_rules:CROP_RULE,..VIEW};
    static CENTER:Node=Node{image_scale:3,..VIEW};
    static CENTER_CLIPPED:Node=Node{relative_rules:CROP_RULE,..CENTER};
    let bounds=Rect{x:0.,y:0.,width:100.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("wide".into(),Image::rgba(200,100,vec![255;200*100*4]).unwrap());
    for (view,x,width) in [(&VIEW,0.,100.),(&CLIPPED,20.,60.),(&CENTER,0.,100.),(&CENTER_CLIPPED,20.,60.)] {
        let commands=ui.frame(Input::default(),bounds,0.,|f|{view.render(f,bounds,&mut state);});
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image{bounds,..} if bounds.x==x && bounds.width==width)));
    }
}

#[test]
fn image_view_additional_scale_modes_use_expected_geometry() {
    let bounds=Rect { x:0., y:0., width:100., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("photo".into(), Image::rgba(40,20,vec![255;40*20*4]).unwrap());
    for (scale, expected) in [
        (2, Rect { x:0.,y:0.,width:100.,height:100. }),
        (3, Rect { x:30.,y:40.,width:40.,height:20. }),
        (4, Rect { x:30.,y:40.,width:40.,height:20. }),
        (5, Rect { x:0.,y:0.,width:100.,height:50. }),
        (6, Rect { x:0.,y:50.,width:100.,height:50. }),
    ] {
        let node=Box::leak(Box::new(Node { kind: Kind::Image, image_src:"photo", image_scale:scale, ..button("image", Visibility::Visible, Size::Match, 0.) }));
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Image { bounds, .. } if *bounds==expected)),"scale {scale}");
    }
}

#[test]
fn image_button_renders_supplied_image_and_reports_click() {
    static BUTTON: Node = Node { kind: Kind::Icon, image_src: "photo", image_builtin:false, ..button("image-button", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:100., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("photo".into(), Image::rgba(4,2,vec![255;32]).unwrap());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { BUTTON.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Image { .. })));
    ui.frame(Input { pointer:Some([50.,50.]),pressed:true,..Input::default() },bounds,0.,|f| { BUTTON.render(f,bounds,&mut state); });
    let mut result=None;
    ui.frame(Input { pointer:Some([50.,50.]),released:true,..Input::default() },bounds,0.,|f| { result=BUTTON.render(f,bounds,&mut state); });
    assert_eq!(result,Some("image-button"));
}

#[test]
fn image_button_dims_disabled_image_and_draws_ripple_over_content() {
    static ENABLED: Node = Node { kind: Kind::Icon, image_src:"photo", image_builtin:false, ..button("enabled-image", Visibility::Visible, Size::Match, 0.) };
    static DISABLED: Node = Node { kind: Kind::Icon, image_src:"photo", image_builtin:false, enabled:false, ..button("disabled-image", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:100., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("photo".into(), Image::rgba(4,2,vec![255;32]).unwrap());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { DISABLED.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Image { color, .. } if color.3==97)));
    ui.frame(Input::default(),bounds,0.,|f| { ENABLED.render(f,bounds,&mut state); });
    ui.frame(Input { pointer:Some([50.,50.]),pressed:true,time:1.,..Input::default() },bounds,0.,|f| { ENABLED.render(f,bounds,&mut state); });
    let commands=ui.frame(Input { pointer:Some([50.,50.]),time:1.1,..Input::default() },bounds,0.,|f| { ENABLED.render(f,bounds,&mut state); });
    let image=commands.iter().position(|cmd| matches!(cmd,DrawCommand::Image { .. })).unwrap();
    let ripple=commands.iter().position(|cmd| matches!(cmd,DrawCommand::Ripple { .. })).unwrap();
    assert!(ripple>image);
}

#[test]
fn image_view_center_uses_drawable_density_for_intrinsic_size() {
    static IMAGE: Node = Node { kind: Kind::Image, image_src:"photo", image_scale:4,image_alpha:255,view_alpha:1.0, ..button("photo", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:100., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("photo".into(), Image::rgba(40,20,vec![255;40*20*4]).unwrap());
    state.image_scales.insert("photo".into(),0.5);
    let commands=ui.frame(Input::default(),bounds,0.,|f| { IMAGE.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Image { bounds, .. } if *bounds==Rect { x:40.,y:45.,width:20.,height:10. })));
}

#[test]
fn scroll_view_fill_viewport_expands_only_when_requested() {
    static CONTENT: Node = Node { kind:Kind::View, width:Size::Dp(40.), height:Size::Dp(30.), background:Some(Color(1,2,3,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("content", Visibility::Visible, Size::Dp(30.), 0.) };
    static HORIZONTAL: Node = Node { kind:Kind::HorizontalScroll, children:&[CONTENT], ..button("scroll", Visibility::Visible, Size::Match, 0.) };
    static HORIZONTAL_FILL: Node = Node { fill_viewport:true,clip_to_padding_disabled:false,clip_children_disabled:false, ..HORIZONTAL };
    static VERTICAL: Node = Node { kind:Kind::Scroll, children:&[CONTENT], ..button("vertical", Visibility::Visible, Size::Match, 0.) };
    static VERTICAL_FILL: Node = Node { fill_viewport:true,clip_to_padding_disabled:false,clip_children_disabled:false, ..VERTICAL };
    let bounds=Rect { x:0., y:0., width:200., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    for (node,expected) in [(&HORIZONTAL,(40.,30.)),(&HORIZONTAL_FILL,(200.,30.)),(&VERTICAL,(40.,30.)),(&VERTICAL_FILL,(40.,100.))] {
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if (bounds.width,bounds.height)==expected)),"{expected:?}");
    }
}

#[test]
fn scroll_child_margins_contribute_to_fill_and_scroll_range() {
    static CONTENT: Node = Node { kind:Kind::View, width:Size::Dp(240.), height:Size::Dp(30.), margin:[10.,4.,20.,6.], background:Some(Color(1,2,3,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("content", Visibility::Visible, Size::Dp(30.), 0.) };
    static SCROLL: Node = Node { kind:Kind::HorizontalScroll, children:&[CONTENT], ..button("scroll", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { SCROLL.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if *bounds==Rect { x:10.,y:4.,width:240.,height:30. })));
    ui.frame(Input { pointer:Some([50.,50.]),scroll:[500.,0.],..Input::default() },bounds,0.,|f| { SCROLL.render(f,bounds,&mut state); });
    assert_eq!(state.offsets["scroll"],70.);
    static SHORT: Node = Node { kind:Kind::View, width:Size::Dp(40.), height:Size::Dp(30.), margin:[10.,4.,20.,6.], background:Some(Color(1,2,3,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("short", Visibility::Visible, Size::Dp(30.), 0.) };
    static FILLED: Node = Node { kind:Kind::HorizontalScroll, fill_viewport:true,clip_to_padding_disabled:false,clip_children_disabled:false, children:&[SHORT], ..button("filled", Visibility::Visible, Size::Match, 0.) };
    let commands=ui.frame(Input::default(),bounds,0.,|f| { FILLED.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if *bounds==Rect { x:10.,y:4.,width:170.,height:30. })));
}

#[test]
fn anonymous_scroll_views_keep_independent_offsets_and_nested_ids() {
    static ROOT: Node = Node { kind:Kind::Linear, children:&[
        Node { kind:Kind::HorizontalScroll, id:"", height:Size::Dp(50.), children:&[
            Node { width:Size::Dp(300.), ..button("first-content", Visibility::Visible, Size::Dp(40.), 0.) }
        ], ..button("", Visibility::Visible, Size::Dp(50.), 0.) },
        Node { kind:Kind::HorizontalScroll, id:"", height:Size::Dp(50.), children:&[
            Node { width:Size::Dp(300.), ..button("second-content", Visibility::Visible, Size::Dp(40.), 0.) }
        ], ..button("", Visibility::Visible, Size::Dp(50.), 0.) },
    ], ..button("root", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    ui.frame(Input { pointer:Some([50.,25.]),scroll:[60.,0.],..Input::default() },bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let mut offsets:Vec<_>=state.offsets.values().copied().collect();offsets.sort_by(f32::total_cmp);
    assert_eq!(offsets,vec![0.,60.]);
    ui.frame(Input { pointer:Some([50.,75.]),scroll:[30.,0.],..Input::default() },bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let mut offsets:Vec<_>=state.offsets.values().copied().collect();offsets.sort_by(f32::total_cmp);
    assert_eq!(offsets,vec![30.,60.]);
}

#[test]
fn anonymous_nested_widgets_have_distinct_ids() {
    static ROOT: Node = Node { kind:Kind::Linear, children:&[
        Node { kind:Kind::Linear, height:Size::Dp(40.), children:&[
            Node { id:"", ..button("", Visibility::Visible, Size::Dp(40.), 0.) }
        ], ..button("", Visibility::Visible, Size::Dp(40.), 0.) },
        Node { id:"", ..button("", Visibility::Visible, Size::Dp(40.), 0.) },
    ], ..button("root", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let ids:Vec<_>=ui.semantics().iter().map(|n|n.id).collect();
    assert_eq!(ids.len(),2);assert_ne!(ids[0],ids[1]);
}

#[test]
fn text_view_wraps_and_sizes_for_available_width_and_newlines() {
    static ROOT: Node = Node { kind:Kind::Linear, children:&[
        Node { kind:Kind::Text, text:"alpha beta\nnext", height:Size::Wrap, ..button("text", Visibility::Visible, Size::Wrap, 0.) },
        Node { ..button("after", Visibility::Visible, Size::Dp(40.), 0.) },
    ], ..button("root", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:80.,height:180. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. } if text.as_ref()!="after" => Some(text.as_ref()),_=>None }).collect();
    assert_eq!(lines,vec!["alpha","beta","next"]);
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y,72.);
}

#[test]
fn text_view_limits_lines_and_ellipsizes_last_visible_line() {
    static ROOT: Node = Node { kind:Kind::Linear, children:&[
        Node { kind:Kind::Text, text:"alpha beta gamma delta", max_lines:2,min_lines:1, ellipsize_end:true, height:Size::Wrap, ..button("text", Visibility::Visible, Size::Wrap, 0.) },
        Node { ..button("after", Visibility::Visible, Size::Dp(40.), 0.) },
    ], ..button("root", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:80.,height:180. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. } if text.as_ref()!="after" => Some(text.as_ref()),_=>None }).collect();
    assert_eq!(lines,vec!["alpha","beta…"]);
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y,48.);
}

#[test]
fn text_view_applies_vertical_and_directional_gravity() {
    static TEXT: Node = Node { kind:Kind::Text, text:"Hello", content_gravity:[0,1], ..button("text", Visibility::Visible, Size::Match, 0.) };
    static BOTTOM: Node = Node { kind:Kind::Text, text:"Bottom", content_gravity:[2,2], align:2, ..button("bottom", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { TEXT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Text { bounds,style,text,.. } if text.as_ref()=="Hello" && bounds.y==38. && style.align==0)));
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f| { TEXT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Text { style,text,.. } if text.as_ref()=="Hello" && style.align==2)));
    let commands=ui.frame(Input::default(),bounds,0.,|f| { BOTTOM.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|cmd| matches!(cmd,DrawCommand::Text { bounds,style,text,.. } if text.as_ref()=="Bottom" && bounds.y==76. && style.align==0)));
}

#[test]
fn text_view_renders_bold_and_uppercase_without_mutating_value() {
    static TEXT: Node = Node { kind:Kind::Text, text:"Hello world", text_bold:true, text_all_caps:true, ..button("text", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.text.insert("text".into(),"Mixed case".into());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { TEXT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,style,.. } if text.as_ref()=="MIXED CASE" && style.weight==700)));
    assert_eq!(state.text["text"],"Mixed case");
}

#[test]
fn padding_resolves_relative_edges_and_insets_child_geometry() {
    static FRAME: Node = Node { kind: Kind::Frame, padding_edges:[5., 7., 9., 11.],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0], padding_relative:[Some(20.),None], children:&[
        Node { width:Size::Match, height:Size::Match, ..button("child", Visibility::Visible, Size::Match, 0.) },
    ], ..button("frame", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:100. };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
    let geometry=|ui: &Ui| ui.semantics().iter().find(|n| n.label.as_ref()=="child").unwrap().bounds;
    assert_eq!(geometry(&ui),Rect { x:20., y:7., width:171., height:82. });
    state.rtl=true;
    ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
    assert_eq!(geometry(&ui),Rect { x:5., y:7., width:175., height:82. });
}

#[test]
fn rating_bar_mirrors_fraction_and_touch_when_requested() {
    static RATING: Node = Node { kind: Kind::RatingBar, rating: 2.5, mirror_for_rtl:true, ..button("rating", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds=Rect { x:0., y:0., width:180., height:48. };
    let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=true;
    let render=|ui: &mut Ui, state: &mut LayoutState, input: Input| ui.frame(input,bounds,0.,|f| { RATING.render(f,bounds,state); });
    let commands=render(&mut ui,&mut state,Input::default());
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::PushClip(r) if (r.x-90.).abs()<0.1 && (r.width-18.).abs()<0.1)));
    for input in [Input { pointer:Some([160.,20.]), pressed:true, ..Input::default() },Input { pointer:Some([160.,20.]), released:true, ..Input::default() }] { render(&mut ui,&mut state,input); }
    assert_eq!(state.ratings["rating"],1.0);
}

#[test]
fn seekbar_mirrors_pointer_and_track_when_requested() {
    static SEEK: Node = Node { kind:Kind::SeekBar, mirror_for_rtl:true, ..button("seek", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };
    let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=true;
    let render=|ui:&mut Ui,state:&mut LayoutState,input:Input| ui.frame(input,bounds,0.,|f| { SEEK.render(f,bounds,state); });
    for input in [Input { pointer:Some([189.,20.]),pressed:true,..Input::default() },Input { pointer:Some([189.,20.]),released:true,..Input::default() }] { render(&mut ui,&mut state,input); }
    assert_eq!(state.progress["seek"],0.);
    for input in [Input { pointer:Some([11.,20.]),pressed:true,..Input::default() },Input { pointer:Some([11.,20.]),released:true,..Input::default() }] { render(&mut ui,&mut state,input); }
    assert_eq!(state.progress["seek"],100.);
}

#[test]
fn progressbar_mirrors_primary_and_secondary_fill() {
    static LINEAR: Node = Node { kind:Kind::ProgressBar, horizontal:true,measure_all_children:false, progress:40., secondary_progress:70., mirror_for_rtl:true, ..button("download", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. }; let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f| { LINEAR.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,.. } if (bounds.x-60.).abs()<0.1 && (bounds.width-140.).abs()<0.1)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,.. } if (bounds.x-120.).abs()<0.1 && (bounds.width-80.).abs()<0.1)));
}

#[test]
fn wrapped_text_height_is_measured_in_frame_and_horizontal_linear_layout() {
    static FRAME: Node = Node { kind:Kind::Frame, children:&[
        Node { kind:Kind::Text, text:"alpha beta\nnext", width:Size::Dp(80.), height:Size::Wrap, layout_gravity:[0,2],absolute_position:[0.0,0.0], ..button("text", Visibility::Visible, Size::Wrap, 0.) },
    ], ..button("frame", Visibility::Visible, Size::Match, 0.) };
    static ROW: Node = Node { kind:Kind::Linear, horizontal:true,measure_all_children:false, children:&[
        Node { kind:Kind::Text, text:"alpha beta\nnext", width:Size::Dp(80.), height:Size::Wrap, ..button("text", Visibility::Visible, Size::Wrap, 0.) },
    ], ..button("row", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:120. };let mut ui=Ui::default();let mut state=LayoutState::default();
    for (node, expected_y) in [(&FRAME,48.),(&ROW,0.)] {
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,bounds,.. } if ["alpha","beta","next"].contains(&text.as_ref()) => Some((text.as_ref(),bounds.y)),_=>None }).collect();
        assert_eq!(lines,vec![("alpha",expected_y),("beta",expected_y+24.),("next",expected_y+48.)]);
    }
}

#[test]
fn space_reserves_fixed_and_weighted_gaps_without_painting() {
    static COLUMN: Node = Node { kind:Kind::Linear, children:&[
        button("first", Visibility::Visible, Size::Dp(40.), 0.),
        Node { kind:Kind::Space, height:Size::Dp(30.), background:Some(Color(255,0,0,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("space",Visibility::Visible,Size::Dp(30.),0.) },
        Node { kind:Kind::Space, height:Size::Dp(0.), weight:1., ..button("weighted",Visibility::Visible,Size::Dp(0.),1.) },
        button("last", Visibility::Visible, Size::Dp(40.), 0.),
    ], ..button("column", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0., y:0., width:200., height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,&mut state); });
    assert_eq!(ui.semantics().iter().find(|n| n.label.as_ref()=="last").unwrap().bounds.y,160.);
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(255,0,0,255),.. })));
    state.visibility.insert("space".into(),Visibility::Gone);
    let commands=ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,&mut state); });
    assert_eq!(ui.semantics().iter().find(|n| n.label.as_ref()=="last").unwrap().bounds.y,160.);
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(255,0,0,255),.. })));
}

#[test]
fn space_background_size_reserves_space_but_never_paints() {
    static COLUMN: Node = Node { kind:Kind::Linear, children:&[
        Node { kind:Kind::Space, width:Size::Wrap, height:Size::Wrap, min_height:10., background:Some(Color(255,0,0,255)), background_min_size:[80.,40.], ..button("gap", Visibility::Visible, Size::Wrap, 0.) },
        button("after", Visibility::Visible, Size::Dp(30.), 0.),
    ], ..button("column", Visibility::Visible, Size::Match, 0.) };
    static ROW: Node = Node { kind:Kind::Linear, horizontal:true, children:&[
        Node { kind:Kind::Space, width:Size::Wrap, height:Size::Wrap, min_width:10., background:Some(Color(255,0,0,255)), background_min_size:[80.,40.], ..button("gap", Visibility::Visible, Size::Wrap, 0.) },
        Node { width:Size::Dp(30.), ..button("after", Visibility::Visible, Size::Dp(30.), 0.) },
    ], ..button("row", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:200. }; let mut ui=Ui::default(); let mut state=LayoutState::default();
    for (node, x, y) in [(&COLUMN,0.,40.),(&ROW,80.,0.)] {
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        let after=ui.semantics().iter().find(|n| n.label.as_ref()=="after").unwrap().bounds;
        assert_eq!((after.x,after.y),(x,y));
        assert!(!commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(255,0,0,255),.. })));
    }
}

#[test]
fn minimum_dimensions_affect_wrap_content_but_not_fixed_size() {
    static COLUMN: Node = Node { kind:Kind::Linear, children:&[
        Node { kind:Kind::Space, height:Size::Wrap, min_height:30., ..button("gap", Visibility::Visible, Size::Wrap, 0.) },
        Node { kind:Kind::Text, text:"one", height:Size::Wrap, min_height:60., ..button("text", Visibility::Visible, Size::Wrap, 0.) },
        button("after", Visibility::Visible, Size::Dp(40.), 0.),
    ], ..button("column", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,&mut state); });
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y,90.);
}

#[test]
fn switch_state_text_follows_toggle_and_fits_wide_labels() {
    static SHOW:&[RelativeRule]=&[RelativeRule{kind:161,anchor:""}];
    static SWITCH:Node=Node{kind:Kind::Switch,text:"Power",text_on:"READY",text_off:"OFF",relative_rules:SHOW,..button("switch_text",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:240.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="OFF" && bounds.width>=30.)));
    ui.frame(Input{pointer:Some([220.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});
    ui.frame(Input{pointer:Some([220.,20.]),released:true,..Input::default()},bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});
    assert!(state.checked["switch_text"]);
    state.rtl=true;let commands=ui.frame(Input{time:0.5,..Input::default()},bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="READY" && bounds.x<80.)));
}

#[test]
fn switch_thumb_and_track_tints_follow_checked_pressed_and_disabled() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:159,anchor:"0,1,90,80,70,255"},RelativeRule{kind:159,anchor:"2,0,140,30,40,255"},RelativeRule{kind:159,anchor:"64,0,10,20,30,255"},RelativeRule{kind:159,anchor:"0,0,40,50,60,255"},RelativeRule{kind:160,anchor:"64,0,70,80,90,255"},RelativeRule{kind:160,anchor:"0,0,100,110,120,255"}];
    static SWITCH:Node=Node{kind:Kind::Switch,relative_rules:RULES,..button("switch_tint",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node{enabled:false,..SWITCH};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let has=|commands:&[DrawCommand],color|commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:c,..} if *c==color));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});assert!(has(&commands,Color(40,50,60,255)));assert!(has(&commands,Color(100,110,120,255)));
    let commands=ui.frame(Input{pointer:Some([180.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});assert!(has(&commands,Color(140,30,40,255)));
    ui.frame(Input{pointer:Some([180.,20.]),released:true,..Input::default()},bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});assert!(state.checked["switch_tint"]);
    let commands=ui.frame(Input::default(),bounds,0.5,|f|{SWITCH.render(f,bounds,&mut state);});assert!(has(&commands,Color(10,20,30,255)));assert!(has(&commands,Color(70,80,90,255)));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});assert!(has(&commands,Color(90,80,70,255)));
}

#[test]
fn radiobutton_button_tint_tracks_checked_pressed_and_disabled_state() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:158,anchor:"0,1,90,80,70,255"},RelativeRule{kind:158,anchor:"2,0,140,30,40,255"},RelativeRule{kind:158,anchor:"64,0,10,20,30,255"},RelativeRule{kind:158,anchor:"0,0,40,50,60,255"}];
    static RADIO:Node=Node{kind:Kind::RadioButton,relative_rules:RULES,..button("radio_tint",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node{enabled:false,..RADIO};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let has=|commands:&[DrawCommand],color|commands.iter().any(|c|matches!(c,DrawCommand::Outline{color:c,..} if *c==color));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{RADIO.render(f,bounds,&mut state);});assert!(has(&commands,Color(40,50,60,255)));
    let commands=ui.frame(Input{pointer:Some([20.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{RADIO.render(f,bounds,&mut state);});assert!(has(&commands,Color(140,30,40,255)));
    ui.frame(Input{pointer:Some([20.,20.]),released:true,..Input::default()},bounds,0.,|f|{RADIO.render(f,bounds,&mut state);});assert!(state.checked["radio_tint"]);
    let commands=ui.frame(Input::default(),bounds,0.5,|f|{RADIO.render(f,bounds,&mut state);});assert!(has(&commands,Color(10,20,30,255)));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});assert!(has(&commands,Color(90,80,70,255)));
}

#[test]
fn checkbox_button_tint_tracks_checked_pressed_and_disabled_state() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:158,anchor:"0,1,90,80,70,255"},RelativeRule{kind:158,anchor:"2,0,140,30,40,255"},RelativeRule{kind:158,anchor:"64,0,10,20,30,255"},RelativeRule{kind:158,anchor:"0,0,40,50,60,255"}];
    static BOX:Node=Node{kind:Kind::CheckBox,relative_rules:RULES,..button("box",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node{enabled:false,..BOX};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let has=|commands:&[DrawCommand],color|commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:c,..} if *c==color));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BOX.render(f,bounds,&mut state);});assert!(has(&commands,Color(40,50,60,255)));
    let commands=ui.frame(Input{pointer:Some([20.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{BOX.render(f,bounds,&mut state);});assert!(has(&commands,Color(140,30,40,255)));
    ui.frame(Input{pointer:Some([20.,20.]),released:true,..Input::default()},bounds,0.,|f|{BOX.render(f,bounds,&mut state);});assert!(state.checked["box"]);
    let commands=ui.frame(Input::default(),bounds,0.5,|f|{BOX.render(f,bounds,&mut state);});assert!(has(&commands,Color(10,20,30,255)));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});assert!(has(&commands,Color(90,80,70,255)));
}

#[test]
fn custom_compound_button_tint_modes_compose_color() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:165,anchor:"0,0,16,16,8,100,100,100,255"},RelativeRule{kind:158,anchor:"0,0,200,0,0,128"},RelativeRule{kind:184,anchor:"0"}];
    static BOX:Node=Node{kind:Kind::CheckBox,relative_rules:RULES,..button("mode_box",Visibility::Visible,Size::Dp(48.),0.)};
    static RADIO:Node=Node{kind:Kind::RadioButton,relative_rules:RULES,..button("mode_radio",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    for view in [&BOX,&RADIO] {
        let commands=ui.frame(Input::default(),bounds,0.,|f|{view.render(f,bounds,&mut state);});
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(150,50,50,255),..})));
    }
}

#[test]
fn radio_custom_button_tint_preserves_alpha_and_selection() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:165,anchor:"0,0,16,16,8,10,20,30,128"},RelativeRule{kind:158,anchor:"64,0,200,0,0,128"},RelativeRule{kind:158,anchor:"0,0,0,0,200,128"}];
    static GROUP:Node=Node{kind:Kind::RadioGroup,id:"alpha_group",children:&[
        Node{kind:Kind::RadioButton,checked:true,relative_rules:RULES,..button("alpha_one",Visibility::Visible,Size::Dp(48.),0.)},
        Node{kind:Kind::RadioButton,relative_rules:RULES,..button("alpha_two",Visibility::Visible,Size::Dp(48.),0.)},
    ],..button("alpha_group",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:96.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{GROUP.render(f,bounds,&mut state);});
    for color in [Color(200,0,0,64),Color(0,0,200,64)] {assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:c,..} if *c==color)));}
    for input in [Input{pointer:Some([20.,60.]),pressed:true,..Input::default()},Input{pointer:Some([20.,60.]),released:true,..Input::default()}] {ui.frame(input,bounds,0.,|f|{GROUP.render(f,bounds,&mut state);});}
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{GROUP.render(f,bounds,&mut state);});
    assert!(!state.checked["alpha_one"] && state.checked["alpha_two"]);
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(200,0,0,64),..} if bounds.x==172. && bounds.y==60.)));
}

#[test]
fn radio_group_custom_button_selector_preserves_single_selection_and_rtl() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:165,anchor:"0,1,16,16,8,90,90,90,255"},RelativeRule{kind:165,anchor:"64,0,16,16,8,10,20,30,255"},RelativeRule{kind:165,anchor:"0,0,16,16,0,200,100,10,255"}];
    static GROUP:Node=Node{kind:Kind::RadioGroup,id:"custom_group",children:&[
        Node{kind:Kind::RadioButton,checked:true,relative_rules:RULES,..button("radio_one",Visibility::Visible,Size::Dp(48.),0.)},
        Node{kind:Kind::RadioButton,relative_rules:RULES,..button("radio_two",Visibility::Visible,Size::Dp(48.),0.)},
        Node{kind:Kind::RadioButton,enabled:false,relative_rules:RULES,..button("radio_disabled",Visibility::Visible,Size::Dp(48.),0.)},
    ],..button("custom_group",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:144.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{GROUP.render(f,bounds,&mut state);});
    for color in [Color(10,20,30,255),Color(200,100,10,255),Color(90,90,90,255)] {assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:c,..} if *c==color)));}
    for input in [Input{pointer:Some([20.,60.]),pressed:true,..Input::default()},Input{pointer:Some([20.,60.]),released:true,..Input::default()}] {ui.frame(input,bounds,0.,|f|{GROUP.render(f,bounds,&mut state);});}
    ui.frame(Input::default(),bounds,0.,|f|{GROUP.render(f,bounds,&mut state);});assert!(!state.checked["radio_one"] && state.checked["radio_two"]);
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{GROUP.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(10,20,30,255),..} if bounds.x==172.)));
}

#[test]
fn checkbox_custom_button_tint_preserves_drawable_alpha() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:165,anchor:"0,0,16,16,8,10,20,30,128"},RelativeRule{kind:158,anchor:"64,0,200,0,0,128"},RelativeRule{kind:158,anchor:"0,0,0,0,200,128"}];
    static BOX:Node=Node{kind:Kind::CheckBox,relative_rules:RULES,..button("checkbox_alpha",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BOX.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(0,0,200,64),..})));
    for input in [Input{pointer:Some([20.,20.]),pressed:true,..Input::default()},Input{pointer:Some([20.,20.]),released:true,..Input::default()}] {ui.frame(input,bounds,0.,|f|{BOX.render(f,bounds,&mut state);});}
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BOX.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(200,0,0,64),..} if bounds.x==172.)));
}

#[test]
fn checkbox_custom_button_selector_changes_with_state_and_rtl() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:165,anchor:"0,1,16,16,8,90,90,90,255"},RelativeRule{kind:165,anchor:"64,0,16,16,8,10,20,30,255"},RelativeRule{kind:165,anchor:"0,0,16,16,0,200,100,10,255"}];
    static BOX:Node=Node{kind:Kind::CheckBox,relative_rules:RULES,..button("custom_button",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node{enabled:false,..BOX};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BOX.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(200,100,10,255),..} if bounds.x==12. && bounds.width==16.)));
    for input in [Input{pointer:Some([20.,20.]),pressed:true,..Input::default()},Input{pointer:Some([20.,20.]),released:true,..Input::default()}] {ui.frame(input,bounds,0.,|f|{BOX.render(f,bounds,&mut state);});}
    assert!(state.checked["custom_button"]);
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{BOX.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(10,20,30,255),..} if bounds.x==172.)));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(90,90,90,255),..})));
    static TINT_RULES:&[RelativeRule]=&[RelativeRule{kind:165,anchor:"0,0,16,16,8,10,20,30,255"},RelativeRule{kind:158,anchor:"0,0,50,60,70,255"}];
    static TINTED:Node=Node{relative_rules:TINT_RULES,..BOX};
    let commands=ui.frame(Input::default(),bounds,0.,|f|{TINTED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(50,60,70,255),..})));
}

#[test]
fn checked_text_custom_mark_tint_mode_composes_color() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:164,anchor:"0,0,16,16,8,100,100,100,255"},RelativeRule{kind:156,anchor:"0,0,200,0,0,128"},RelativeRule{kind:185,anchor:"0"}];
    static VIEW:Node=Node{kind:Kind::CheckedText,text:"Choice",relative_rules:RULES,..button("mark_mode",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(150,50,50,255),..})));
}

#[test]
fn checked_text_custom_mark_tint_preserves_alpha_and_checked_state() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:164,anchor:"0,0,16,16,8,10,20,30,128"},RelativeRule{kind:156,anchor:"64,0,200,0,0,128"},RelativeRule{kind:156,anchor:"0,0,0,0,200,128"}];
    static VIEW:Node=Node{kind:Kind::CheckedText,text:"Choice",checked:true,relative_rules:RULES,..button("mark_alpha",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(200,0,0,64),..})));
    state.checked.insert("mark_alpha".into(),false);state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(0,0,200,64),..} if bounds.x==4.)));
}

#[test]
fn checked_text_custom_mark_selector_tracks_checked_and_reserves_width() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:157,anchor:"start"},RelativeRule{kind:164,anchor:"64,0,16,16,8,10,20,30,255"},RelativeRule{kind:164,anchor:"0,0,0,0,0,0,0,0,0"}];
    static TEXT:Node=Node{kind:Kind::CheckedText,text:"Choice",checked:true,relative_rules:RULES,..button("custom_mark",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{TEXT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(10,20,30,255),..} if bounds.x==4. && bounds.width==16.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{bounds,..} if bounds.x==24.)));
    state.checked.insert("custom_mark".into(),false);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{TEXT.render(f,bounds,&mut state);});
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(10,20,30,255),..})));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{bounds,..} if bounds.x==24.)));
    static COLORS:&[RelativeRule]=&[RelativeRule{kind:164,anchor:"0,1,16,16,8,90,90,90,255"},RelativeRule{kind:164,anchor:"64,0,16,16,8,10,20,30,255"},RelativeRule{kind:164,anchor:"0,0,16,16,0,200,100,10,255"}];
    static SELECTOR:Node=Node{relative_rules:COLORS,..TEXT};
    static DISABLED:Node=Node{enabled:false,..SELECTOR};
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{SELECTOR.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(200,100,10,255),..})));
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(90,90,90,255),..})));
}

#[test]
fn checked_text_mark_tint_and_logical_gravity_follow_state_and_rtl() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:157,anchor:"start"},RelativeRule{kind:156,anchor:"0,1,90,80,70,255"},RelativeRule{kind:156,anchor:"2,0,140,30,40,255"},RelativeRule{kind:156,anchor:"64,0,10,20,30,255"}];
    static TEXT:Node=Node{kind:Kind::CheckedText,text:"Choice",checked:true,clickable:true,relative_rules:RULES,..button("choice",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node{enabled:false,..TEXT};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{TEXT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape{bounds,color:Color(10,20,30,255),..} if bounds.x<32.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{bounds,..} if bounds.x==32.)));
    let commands=ui.frame(Input{pointer:Some([50.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{TEXT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape{color:Color(140,30,40,255),..})));
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape{bounds,color:Color(90,80,70,255),..} if bounds.x>160.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{bounds,..} if bounds.x==0.)));
}

#[test]
fn checked_text_view_keeps_host_controlled_state_and_reserves_mark_space() {
    static TEXT: Node = Node { kind:Kind::CheckedText, text:"Choice", checked:true, clickable:true, ..button("choice", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,input:Input| ui.frame(input,bounds,0.,|f| { TEXT.render(f,bounds,state); });
    let commands=render(&mut ui,&mut state,Input::default());
    assert!(state.checked["choice"]);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,bounds,.. } if text.as_ref()=="Choice" && bounds.width==168.)));
    for input in [Input { pointer:Some([10.,20.]),pressed:true,..Input::default() },Input { pointer:Some([10.,20.]),released:true,..Input::default() }] { render(&mut ui,&mut state,input); }
    assert!(state.checked["choice"]);
    state.rtl=true;
    let commands=render(&mut ui,&mut state,Input::default());
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,bounds,.. } if text.as_ref()=="Choice" && bounds.x==32.)));
}

#[test]
fn android_switch_positions_control_and_thumb_for_layout_direction() {
    static SWITCH: Node = Node { kind:Kind::Switch, text:"Setting", checked:true, ..button("setting", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { SWITCH.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,bounds,.. } if text.as_ref()=="Setting" && bounds.x==0.)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,.. } if bounds.width==24. && bounds.x==172.)));
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f| { SWITCH.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,bounds,.. } if text.as_ref()=="Setting" && bounds.x==56.)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,.. } if bounds.width==24. && bounds.x==4.)));
}

#[test]
fn scroll_view_measures_nested_wrapped_text_for_scroll_range() {
    static SCROLL: Node = Node { kind:Kind::Scroll, children:&[
        Node { kind:Kind::Linear, height:Size::Wrap, children:&[
            Node { kind:Kind::Text, text:"alpha beta\nnext\nmore\nlast", height:Size::Wrap, ..button("text", Visibility::Visible, Size::Wrap, 0.) },
            button("after", Visibility::Visible, Size::Dp(40.), 0.),
        ], ..button("column", Visibility::Visible, Size::Wrap, 0.) },
    ], ..button("scroll", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:80.,height:72. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { SCROLL.render(f,bounds,&mut state); });
    ui.frame(Input { pointer:Some([30.,30.]),scroll:[0.,500.],..Input::default() },bounds,0.,|f| { SCROLL.render(f,bounds,&mut state); });
    assert_eq!(state.offsets["scroll"],88.);
    assert!(ui.semantics().iter().any(|n| n.label.as_ref()=="after"));
}

#[test]
fn checkbox_and_radio_mirror_indicator_and_label_in_rtl() {
    static CHECK: Node = Node { kind:Kind::CheckBox, text:"Check", checked:true, ..button("check", Visibility::Visible, Size::Dp(48.), 0.) };
    static RADIO: Node = Node { kind:Kind::RadioButton, text:"Radio", checked:true, ..button("radio", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=true;
    for (node,label,indicator_width,indicator_x) in [(&CHECK,"Check",18.,171.),(&RADIO,"Radio",20.,170.)] {
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,bounds,style,.. } if text.as_ref()==label && bounds.x==0. && style.align==2)));
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,.. } | DrawCommand::Outline { bounds,.. } if bounds.width==indicator_width && bounds.x==indicator_x)));
    }
}

#[test]
fn edit_text_uses_initial_value_hint_and_disabled_state() {
    static EDIT: Node = Node { kind:Kind::Edit, text:"Initial", hint:"Your name", enabled:false, ..button("name", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { EDIT.render(f,bounds,&mut state); });
    assert_eq!(state.text["name"],"Initial");
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="Initial")));
    assert!(ui.semantics().iter().any(|n| n.label.as_ref()=="Your name" && !n.enabled));
    state.text.insert("name".into(),"Changed".into());
    ui.frame(Input::default(),bounds,0.,|f| { EDIT.render(f,bounds,&mut state); });
    assert_eq!(state.text["name"],"Changed");
}

#[test]
fn password_edit_text_masks_rendered_value() {
    static EDIT: Node = Node { kind:Kind::Edit, text:"secret", hint:"Password", input_type:4, ..button("password", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { EDIT.render(f,bounds,&mut state); });
    assert_eq!(state.text["password"],"secret");assert_eq!(state.input_types["password"],4);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="••••••")));
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="secret")));
}

#[test]
fn view_switcher_displays_one_host_selected_child() {
    static SWITCHER: Node = Node { kind:Kind::ViewSwitcher, children:&[
        button("first", Visibility::Visible, Size::Dp(40.), 0.),
        button("second", Visibility::Visible, Size::Dp(40.), 0.),
    ], ..button("switcher", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="first"));assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="second"));
    state.displayed_children.insert("switcher".into(),1);
    ui.frame(Input::default(),bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="first"));assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="second"));
    state.displayed_children.insert("switcher".into(),100);
    ui.frame(Input::default(),bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert_eq!(state.displayed_children["switcher"],0);
}

#[test]
fn view_switcher_crossfades_and_honors_reduced_motion() {
    use bhoo::android_xml::AlphaAnimation;
    static SWITCHER: Node = Node { kind:Kind::ViewSwitcher, animation_in:Some(AlphaAnimation { from:0.,to:1.,duration:0.3,start_offset:0.0,linear:true }), animation_out:Some(AlphaAnimation { from:1.,to:0.,duration:0.3,start_offset:0.0,linear:true }), children:&[
        Node { kind:Kind::View, background:Some(Color(255,0,0,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("first", Visibility::Visible, Size::Match, 0.) },
        Node { kind:Kind::View, background:Some(Color(0,0,255,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("second", Visibility::Visible, Size::Match, 0.) },
    ], ..button("switcher", Visibility::Visible, Size::Match, 0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input { time:1., ..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    state.displayed_children.insert("switcher".into(),1);
    ui.frame(Input { time:2., ..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    let commands=ui.frame(Input { time:2.15, ..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert!(ui.needs_redraw());
    for (r,b) in [(255,0),(0,255)] { assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(red,0,blue,alpha),.. } if *red==r && *blue==b && (126..=129).contains(alpha)))); }
    let commands=ui.frame(Input { time:2.31, ..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(255,0,0,_),.. })));
    ui.motion_preference=MotionPreference::Reduced;state.displayed_children.insert("switcher".into(),0);
    let commands=ui.frame(Input { time:3., ..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(255,0,0,255),.. })));
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(0,0,255,_),.. })));
}

#[test]
fn view_switcher_delays_alpha_and_translation_independently() {
    use bhoo::android_xml::{AlphaAnimation,AnimationDelta,TranslateAnimation};
    const ZERO:AnimationDelta=AnimationDelta{value:0.,mode:0};
    static SWITCHER:Node=Node{kind:Kind::ViewSwitcher,
        animation_in:Some(AlphaAnimation{from:0.,to:1.,duration:0.4,start_offset:0.2,linear:true}),
        translate_in:Some(TranslateAnimation{deltas:[AnimationDelta{value:100.,mode:0},ZERO,ZERO,ZERO],duration:0.4,start_offset:0.4,linear:true,y_timing:None}),
        children:&[button("first",Visibility::Visible,Size::Match,0.),
            Node{kind:Kind::View,background:Some(Color(0,0,255,255)),..button("second",Visibility::Visible,Size::Match,0.)}],
        ..button("switcher",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:80.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input{time:1.,..Input::default()},bounds,0.,|f|{SWITCHER.render(f,bounds,&mut state);});
    state.displayed_children.insert("switcher".into(),1);
    ui.frame(Input{time:2.,..Input::default()},bounds,0.,|f|{SWITCHER.render(f,bounds,&mut state);});
    let commands=ui.frame(Input{time:2.4,..Input::default()},bounds,0.,|f|{SWITCHER.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(0,0,255,a),..} if (bounds.x-100.).abs()<0.01 && (126..=129).contains(a))));
    let commands=ui.frame(Input{time:2.6,..Input::default()},bounds,0.,|f|{SWITCHER.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(0,0,255,255),..} if (bounds.x-50.).abs()<0.01)));
    assert!(ui.needs_redraw());
    let commands=ui.frame(Input{time:2.81,..Input::default()},bounds,0.,|f|{SWITCHER.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(0,0,255,255),..} if bounds.x.abs()<0.01)));
}

#[test]
fn view_switcher_slides_relative_to_child_and_parent() {
    use bhoo::android_xml::{AnimationDelta,TranslateAnimation};
    const ZERO:AnimationDelta=AnimationDelta { value:0.,mode:0 };
    static SWITCHER:Node=Node { kind:Kind::ViewSwitcher,
        translate_in:Some(TranslateAnimation { deltas:[AnimationDelta { value:1.,mode:2 },ZERO,ZERO,ZERO],duration:0.4,start_offset:0.0,linear:true,y_timing:None }),
        translate_out:Some(TranslateAnimation { deltas:[ZERO,AnimationDelta { value:-1.,mode:1 },ZERO,ZERO],duration:0.4,start_offset:0.0,linear:true,y_timing:None }),
        children:&[
            Node { kind:Kind::View, width:Size::Dp(80.),height:Size::Dp(40.),background:Some(Color(255,0,0,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("first",Visibility::Visible,Size::Dp(40.),0.) },
            Node { kind:Kind::View, width:Size::Dp(80.),height:Size::Dp(40.),background:Some(Color(0,0,255,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("second",Visibility::Visible,Size::Dp(40.),0.) },
        ], ..button("switcher",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input { time:1.,..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    state.displayed_children.insert("switcher".into(),1);
    ui.frame(Input { time:2.,..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    let commands=ui.frame(Input { time:2.2,..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(255,0,0,255),.. } if (bounds.x+40.).abs()<0.01)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(0,0,255,255),.. } if (bounds.x-100.).abs()<0.01)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::PushClip(rect) if rect.width>0.0 && rect.width<=80.0)));
}

#[test]
fn view_switcher_combines_alpha_and_translation() {
    use bhoo::android_xml::{AlphaAnimation,AnimationDelta,TranslateAnimation};
    const ZERO:AnimationDelta=AnimationDelta { value:0.,mode:0 };
    static SWITCHER:Node=Node { kind:Kind::ViewSwitcher,
        animation_in:Some(AlphaAnimation { from:0.,to:1.,duration:0.4,start_offset:0.0,linear:true }),
        translate_in:Some(TranslateAnimation { deltas:[AnimationDelta { value:1.,mode:2 },ZERO,ZERO,ZERO],duration:0.4,start_offset:0.0,linear:true,y_timing:None }),
        animation_out:Some(AlphaAnimation { from:1.,to:0.,duration:0.4,start_offset:0.0,linear:true }),
        children:&[
            Node { kind:Kind::View, background:Some(Color(255,0,0,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("first",Visibility::Visible,Size::Match,0.) },
            Node { kind:Kind::View, background:Some(Color(0,0,255,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0, ..button("second",Visibility::Visible,Size::Match,0.) },
        ], ..button("switcher",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input { time:1.,..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    state.displayed_children.insert("switcher".into(),1);
    ui.frame(Input { time:2.,..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    let commands=ui.frame(Input { time:2.2,..Input::default() },bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(0,0,255,a),.. } if (bounds.x-100.).abs()<0.01 && (126..=129).contains(a))));
}

#[test]
fn view_flipper_schedules_flips_and_can_be_paused() {
    static FLIPPER:Node=Node { kind:Kind::ViewFlipper, auto_start:true, flip_interval:1.0, children:&[
        button("first",Visibility::Visible,Size::Dp(40.),0.),button("second",Visibility::Visible,Size::Dp(40.),0.),
    ], ..button("flipper",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,time:f64| ui.frame(Input { time,..Input::default() },bounds,0.,|f| { FLIPPER.render(f,bounds,state); });
    render(&mut ui,&mut state,1.);assert_eq!(ui.next_wakeup(),Some(2.));assert_eq!(state.displayed_children["flipper"],0);
    render(&mut ui,&mut state,2.);assert_eq!(state.displayed_children["flipper"],1);assert_eq!(ui.next_wakeup(),Some(3.));
    state.flipper_running.insert("flipper".into(),false);render(&mut ui,&mut state,2.5);assert_eq!(ui.next_wakeup(),None);
    render(&mut ui,&mut state,9.);assert_eq!(state.displayed_children["flipper"],1);
    state.flipper_running.insert("flipper".into(),true);render(&mut ui,&mut state,10.);assert_eq!(ui.next_wakeup(),Some(11.));
    render(&mut ui,&mut state,11.);assert_eq!(state.displayed_children["flipper"],0);
}

#[test]
fn view_flipper_restarts_interval_after_app_pause() {
    static FLIPPER:Node=Node { kind:Kind::ViewFlipper,auto_start:true,flip_interval:1.,children:&[
        button("first",Visibility::Visible,Size::Dp(40.),0.),button("second",Visibility::Visible,Size::Dp(40.),0.),
    ], ..button("flipper",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input { time:1.,..Input::default() },bounds,0.,|f| { FLIPPER.render(f,bounds,&mut state); });
    ui.pause_animations();
    ui.frame(Input { time:20.,..Input::default() },bounds,0.,|f| { FLIPPER.render(f,bounds,&mut state); });
    assert_eq!(state.displayed_children["flipper"],0);assert_eq!(ui.next_wakeup(),Some(21.));
}

#[test]
fn text_switcher_sets_current_or_next_text_in_anonymous_children() {
    static SWITCHER:Node=Node { kind:Kind::TextSwitcher, children:&[
        Node { kind:Kind::Text,text:"initial", ..button("",Visibility::Visible,Size::Match,0.) },
        Node { kind:Kind::Text,text:"unused", ..button("",Visibility::Visible,Size::Match,0.) },
    ], ..button("switcher",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    assert!(SWITCHER.set_switcher_text(&mut state,"Current",false));
    let commands=ui.frame(Input::default(),bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="Current")));
    assert!(SWITCHER.set_switcher_text(&mut state,"Next",true));
    let commands=ui.frame(Input::default(),bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert_eq!(state.displayed_children["switcher"],1);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="Next")));
}

#[test]
fn image_switcher_updates_next_anonymous_child() {
    static SWITCHER:Node=Node { kind:Kind::ImageSwitcher, children:&[
        Node { kind:Kind::Image,image_src:"first", ..button("",Visibility::Visible,Size::Match,0.) },
        Node { kind:Kind::Image,image_src:"first", ..button("",Visibility::Visible,Size::Match,0.) },
    ], ..button("switcher",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:80. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("first".into(),Image::rgba(4,2,vec![255;32]).unwrap());
    state.images.insert("second".into(),Image::rgba(2,4,vec![255;32]).unwrap());
    assert!(!SWITCHER.set_switcher_image(&mut state,"missing"));
    assert!(SWITCHER.set_switcher_image(&mut state,"second"));
    let commands=ui.frame(Input::default(),bounds,0.,|f| { SWITCHER.render(f,bounds,&mut state); });
    assert_eq!(state.displayed_children["switcher"],1);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Image { image,.. } if image.width()==2 && image.height()==4)));
}

#[test]
fn chronometer_formats_countup_and_countdown_and_stops_updates() {
    use bhoo::android_xml::ChronometerState;
    static UP:Node=Node { kind:Kind::Chronometer, chrono_format:"Elapsed %s", ..button("up",Visibility::Visible,Size::Dp(48.),0.) };
    static DOWN:Node=Node { kind:Kind::Chronometer, count_down:true, ..button("down",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.chronometers.insert("up".into(),ChronometerState { base:10.,running:true });
    let commands=ui.frame(Input { time:75.,..Input::default() },bounds,0.,|f| { UP.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="Elapsed 01:05")));assert!((ui.next_wakeup().unwrap()-76.001).abs()<1e-9);
    state.chronometers.get_mut("up").unwrap().running=false;
    ui.frame(Input { time:80.,..Input::default() },bounds,0.,|f| { UP.render(f,bounds,&mut state); });
    assert_eq!(state.text["up"],"Elapsed 01:05");assert_eq!(ui.next_wakeup(),None);
    state.chronometers.insert("down".into(),ChronometerState { base:90.,running:true });
    let commands=ui.frame(Input { time:85.,..Input::default() },bounds,0.,|f| { DOWN.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="00:05")));
}

#[test]
fn text_clock_formats_host_wall_time_and_schedules_next_tick() {
    use bhoo::android_xml::WallClock;
    static CLOCK:Node=Node { kind:Kind::TextClock, clock_12:"h:mm:ss a", clock_24:"HH:mm:ss", ..button("clock",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();state.wall_clock=Some(WallClock { unix_seconds:43205.25,offset_seconds:3600,is_24_hour:false });
    let commands=ui.frame(Input { time:10.,..Input::default() },bounds,0.,|f| { CLOCK.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="1:00:05 PM")));assert_eq!(ui.next_wakeup(),Some(10.75));
    state.wall_clock.as_mut().unwrap().is_24_hour=true;
    let commands=ui.frame(Input { time:10.5,..Input::default() },bounds,0.,|f| { CLOCK.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="13:00:05")));
}

#[test]
fn text_clock_date_handles_leap_day_and_timezone_boundaries() {
    use bhoo::android_xml::WallClock;
    static CLOCK:Node=Node { kind:Kind::TextClock, clock_24:"yyyy-MM-dd HH:mm:ss", ..button("clock",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:300.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();state.wall_clock=Some(WallClock { unix_seconds:1709251199.,offset_seconds:0,is_24_hour:true });
    let render=|ui:&mut Ui,state:&mut LayoutState| ui.frame(Input::default(),bounds,0.,|f| { CLOCK.render(f,bounds,state); });
    let commands=render(&mut ui,&mut state);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="2024-02-29 23:59:59")));
    state.wall_clock.as_mut().unwrap().offset_seconds=3600;
    let commands=render(&mut ui,&mut state);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="2024-03-01 00:59:59")));
    state.wall_clock=Some(WallClock { unix_seconds:-1.,offset_seconds:0,is_24_hour:true });
    let commands=render(&mut ui,&mut state);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="1969-12-31 23:59:59")));
}

#[test]
fn analog_clock_hands_follow_host_time_and_schedule_minute() {
    use bhoo::android_xml::WallClock;
    static CLOCK:Node=Node { kind:Kind::AnalogClock, ..button("clock",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();state.wall_clock=Some(WallClock { unix_seconds:10800.,offset_seconds:0,is_24_hour:true });
    let commands=ui.frame(Input { time:5.,..Input::default() },bounds,0.,|f| { CLOCK.render(f,bounds,&mut state); });
    let hand=|name:&str| commands.iter().find_map(|c| match c { DrawCommand::Shape { index,rotation,.. } if shape_data(*index as usize).is_some_and(|s|s.name==name) => Some(*rotation), _=>None }).unwrap();
    assert!((hand("aosp-clock-hand-hour")-std::f32::consts::FRAC_PI_2).abs()<0.001);
    assert!(hand("aosp-clock-hand-minute").abs()<0.001);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Shape { index,.. } if shape_data(*index as usize).is_some_and(|s|s.name=="aosp-clock-dial"))));
    assert_eq!(ui.next_wakeup(),Some(65.));
}

#[test]
fn linear_layout_dividers_measure_and_follow_direction_and_visibility() {
    static VERTICAL:Node=Node { kind:Kind::Linear,divider_color:Some(Color(10,20,30,255)),divider_thickness:2.,divider_flags:7,divider_padding:5.,children:&[
        button("first",Visibility::Visible,Size::Dp(40.),0.),button("second",Visibility::Visible,Size::Dp(40.),0.),
    ], ..button("vertical",Visibility::Visible,Size::Match,0.) };
    static HORIZONTAL:Node=Node { kind:Kind::Linear,horizontal:true,measure_all_children:false,divider_color:Some(Color(10,20,30,255)),divider_thickness:2.,divider_flags:7,divider_padding:5.,children:&[
        Node { width:Size::Dp(40.),..button("first",Visibility::Visible,Size::Dp(40.),0.) },
        Node { width:Size::Dp(40.),..button("second",Visibility::Visible,Size::Dp(40.),0.) },
    ], ..button("horizontal",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { VERTICAL.render(f,bounds,&mut state); });
    let positions:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::RoundedRect { bounds,color:Color(10,20,30,255),.. } =>Some(*bounds),_=>None }).collect();
    assert_eq!(positions,vec![Rect { x:5.,y:0.,width:190.,height:2. },Rect { x:5.,y:42.,width:190.,height:2. },Rect { x:5.,y:84.,width:190.,height:2. }]);
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="second").unwrap().bounds.y,44.);
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f| { HORIZONTAL.render(f,bounds,&mut state); });
    let xs:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::RoundedRect { bounds,color:Color(10,20,30,255),.. }=>Some(bounds.x),_=>None }).collect();
    assert_eq!(xs,vec![198.,156.,114.]);
    state.visibility.insert("first".into(),Visibility::Gone);
    let commands=ui.frame(Input::default(),bounds,0.,|f| { HORIZONTAL.render(f,bounds,&mut state); });
    let xs:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::RoundedRect { bounds,color:Color(10,20,30,255),.. }=>Some(bounds.x),_=>None }).collect();
    assert_eq!(xs,vec![198.,156.]);
}

#[test]
fn linear_layout_explicit_weight_sum_leaves_space_and_applies_gravity() {
    static ROW:Node=Node { kind:Kind::Linear,horizontal:true,measure_all_children:false,weight_sum:4.,content_gravity:[1,0],relative_rules:&[RelativeRule{kind:36,anchor:""}],children:&[
        Node { width:Size::Dp(0.),weight:1., ..button("first",Visibility::Visible,Size::Dp(40.),1.) },
        Node { width:Size::Dp(0.),weight:1., ..button("second",Visibility::Visible,Size::Dp(40.),1.) },
    ], ..button("row",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f| { ROW.render(f,bounds,&mut state); });
    let geometry=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    assert_eq!(geometry(&ui,"first"),Rect { x:50.,y:0.,width:50.,height:40. });
    assert_eq!(geometry(&ui,"second"),Rect { x:100.,y:0.,width:50.,height:40. });
    state.rtl=true;
    ui.frame(Input::default(),bounds,0.,|f| { ROW.render(f,bounds,&mut state); });
    assert_eq!(geometry(&ui,"first").x,100.);assert_eq!(geometry(&ui,"second").x,50.);
}

#[test]
fn text_view_single_line_ellipsizes_start_middle_and_end() {
    static START:Node=Node { kind:Kind::Text,text:"abcdefghij.txt",single_line:true,max_lines:1,min_lines:1,ellipsize_side:1, ..button("start",Visibility::Visible,Size::Wrap,0.) };
    static MIDDLE:Node=Node { kind:Kind::Text,text:"abcdefghij.txt",single_line:true,max_lines:1,min_lines:1,ellipsize_side:2, ..button("middle",Visibility::Visible,Size::Wrap,0.) };
    static END:Node=Node { kind:Kind::Text,text:"abcdefghij.txt",single_line:true,max_lines:1,min_lines:1,ellipsize_end:true, ..button("end",Visibility::Visible,Size::Wrap,0.) };
    static NEWLINE:Node=Node { kind:Kind::Text,text:"Hello\nworld",single_line:true,max_lines:1,min_lines:1, ..button("newline",Visibility::Visible,Size::Wrap,0.) };
    let bounds=Rect { x:0.,y:0.,width:80.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,node:&'static Node| {
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,state); });
        commands.into_iter().find_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.to_string()),_=>None }).unwrap()
    };
    let start=render(&mut ui,&mut state,&START);assert!(start.starts_with('…') && start.ends_with(".txt"),"{start}");
    let middle=render(&mut ui,&mut state,&MIDDLE);assert!(middle.starts_with('a') && middle.contains('…') && middle.ends_with("txt"),"{middle}");
    let end=render(&mut ui,&mut state,&END);assert!(end.starts_with('a') && end.ends_with('…'),"{end}");
    let text=render(&mut ui,&mut state,&NEWLINE);assert_eq!(text,"Hello world");
}

#[test]
fn text_view_preserves_graphemes_when_wrapping_and_truncating() {
    use unicode_segmentation::UnicodeSegmentation;
    static TEXT: &str = "a\u{301}👨‍👩‍👧‍👦a\u{301}👨‍👩‍👧‍👦a\u{301}👨‍👩‍👧‍👦";
    let mut ui=Ui::default(); let mut state=LayoutState::default();
    for side in 1..=3 {
        let node=Box::leak(Box::new(Node { kind:Kind::Text,text:TEXT,single_line:true,ellipsize_side:side, ..button("unicode",Visibility::Visible,Size::Wrap,0.) }));
        let bounds=Rect { x:0.,y:0.,width:45.,height:48. };
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        let rendered=commands.iter().find_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).unwrap();
        assert!(rendered.contains('…'),"{rendered}");
        for part in rendered.graphemes(true).filter(|g| *g!="…") { assert!(TEXT.graphemes(true).any(|original| original==part),"broken grapheme {part:?} in {rendered:?}"); }
    }
    let node=Box::leak(Box::new(Node { kind:Kind::Text,text:"a\u{301}b\u{301}c\u{301}", ..button("wrap",Visibility::Visible,Size::Wrap,0.) }));
    let bounds=Rect { x:0.,y:0.,width:12.,height:100. };
    let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
    let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).collect();
    assert_eq!(lines,vec!["a\u{301}","b\u{301}","c\u{301}"]);
    let node=Box::leak(Box::new(Node { kind:Kind::Text,text:"a\u{301}b\u{301}",max_lines:1,min_lines:1,ellipsize_end:true, ..button("limit",Visibility::Visible,Size::Wrap,0.) }));
    let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="…")));
}

#[test]
fn frame_layout_keeps_oversized_fixed_children_and_positions_by_gravity() {
    static FRAME: Node = Node { kind:Kind::Frame,children:&[
        Node { width:Size::Dp(240.),height:Size::Dp(140.),layout_gravity:[1,1],absolute_position:[0.0,0.0],..button("large",Visibility::Visible,Size::Dp(140.),0.) },
    ],..button("frame",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { bounds:Rect { x,y,width,height },text,.. } if text.as_ref()=="large" && (*x,*y,*width,*height)==(-20.,-20.,240.,140.))));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::PushClip(r) if *r==bounds)));
}

#[test]
fn frame_layout_clips_to_padding_by_default_and_can_disable_it() {
    static CHILD:Node=Node { width:Size::Dp(300.),height:Size::Dp(100.),layout_gravity:[1,1],absolute_position:[0.0,0.0], ..button("child",Visibility::Visible,Size::Dp(100.),0.) };
    static PADDED:Node=Node { kind:Kind::Frame,padding_edges:[10.,20.,30.,10.],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0],children:&[CHILD],..button("frame",Visibility::Visible,Size::Match,0.) };
    static UNCLIPPED_PADDING:Node=Node { clip_to_padding_disabled:true,clip_children_disabled:false,..PADDED };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,node:&'static Node| ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,state); });
    let commands=render(&mut ui,&mut state,&PADDED);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::PushClip(r) if *r==Rect { x:10.,y:20.,width:160.,height:70. })));
    let commands=render(&mut ui,&mut state,&UNCLIPPED_PADDING);
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::PushClip(r) if *r==Rect { x:10.,y:20.,width:160.,height:70. })));
}

#[test]
fn linear_and_animator_clip_children_to_padding_and_respect_override() {
    static CHILD:Node=Node { height:Size::Dp(90.), ..button("child",Visibility::Visible,Size::Dp(90.),0.) };
    static LINEAR:Node=Node { kind:Kind::Linear,padding_edges:[10.,20.,30.,10.],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0],children:&[CHILD],..button("linear",Visibility::Visible,Size::Match,0.) };
    static LINEAR_EDGE:Node=Node { clip_to_padding_disabled:true,clip_children_disabled:false,..LINEAR };
    static ANIMATOR_EDGE:Node=Node { kind:Kind::ViewAnimator,clip_to_padding_disabled:true,clip_children_disabled:false,..LINEAR };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    for (node,padded) in [(&LINEAR,true),(&LINEAR_EDGE,false),(&ANIMATOR_EDGE,false)] {
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        assert_eq!(commands.iter().any(|c| matches!(c,DrawCommand::PushClip(r) if *r==Rect { x:10.,y:20.,width:160.,height:70. })),padded);
    }
}

#[test]
fn scroll_views_honor_clip_to_padding_for_scrolled_children() {
    static CONTENT:Node=Node { kind:Kind::Text,text:"Content",width:Size::Dp(300.),height:Size::Dp(300.),..button("content",Visibility::Visible,Size::Dp(300.),0.) };
    static VERTICAL:Node=Node { kind:Kind::Scroll,padding_edges:[10.,20.,30.,10.],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0],children:&[CONTENT],..button("vertical",Visibility::Visible,Size::Match,0.) };
    static VERTICAL_EDGE:Node=Node { clip_to_padding_disabled:true,clip_children_disabled:false,..VERTICAL };
    static HORIZONTAL_EDGE:Node=Node { kind:Kind::HorizontalScroll,clip_to_padding_disabled:true,clip_children_disabled:false,..VERTICAL };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    for (node,padded) in [(&VERTICAL,true),(&VERTICAL_EDGE,false),(&HORIZONTAL_EDGE,false)] {
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        assert_eq!(commands.iter().any(|c| matches!(c,DrawCommand::PushClip(r) if *r==Rect { x:10.,y:20.,width:160.,height:70. })),padded);
    }
}

#[test]
fn text_clock_respects_quoted_seconds_and_hour_cycles() {
    use bhoo::android_xml::WallClock;
    static LITERAL:Node=Node { kind:Kind::TextClock,clock_24:"HH:mm 'seconds' ''",..button("clock",Visibility::Visible,Size::Dp(48.),0.) };
    static SECONDS:Node=Node { kind:Kind::TextClock,clock_24:"k K 's' ss",..button("clock",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:240.,height:48. };let mut ui=Ui::default();
    let mut state=LayoutState::default();state.wall_clock=Some(WallClock { unix_seconds:5.25,offset_seconds:30,is_24_hour:true });
    let commands=ui.frame(Input { time:10.,..Input::default() },bounds,0.,|f| { LITERAL.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="00:00 seconds '")));
    assert_eq!(ui.next_wakeup(),Some(34.75));
    let commands=ui.frame(Input { time:11.,..Input::default() },bounds,0.,|f| { SECONDS.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="24 0 s 35")));
    assert_eq!(ui.next_wakeup(),Some(11.75));
}

#[test]
fn chronometer_matches_android_rounding_and_base_aligned_ticks() {
    use bhoo::android_xml::ChronometerState;
    static UP:Node=Node { kind:Kind::Chronometer,..button("up",Visibility::Visible,Size::Dp(48.),0.) };
    static DOWN:Node=Node { kind:Kind::Chronometer,count_down:true,..button("down",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.chronometers.insert("up".into(),ChronometerState { base:10.2,running:true });
    let commands=ui.frame(Input { time:10.8,..Input::default() },bounds,0.,|f| { UP.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="00:01")));
    assert!((ui.next_wakeup().unwrap()-11.201).abs()<1e-9);
    state.chronometers.insert("down".into(),ChronometerState { base:20.2,running:true });
    let commands=ui.frame(Input { time:19.8,..Input::default() },bounds,0.,|f| { DOWN.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="00:00")));
    assert!((ui.next_wakeup().unwrap()-20.201).abs()<1e-9);
    let commands=ui.frame(Input { time:20.201,..Input::default() },bounds,0.,|f| { DOWN.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="-00:01")));
}

#[test]
fn chronometer_formats_indexed_duration_and_literal_percent() {
    use bhoo::android_xml::ChronometerState;
    static CLOCK:Node=Node { kind:Kind::Chronometer,chrono_format:"%1$s / %1$s (100%%)",..button("clock",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:300.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.chronometers.insert("clock".into(),ChronometerState { base:10.,running:true });
    let commands=ui.frame(Input { time:15.,..Input::default() },bounds,0.,|f| { CLOCK.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="00:05 / 00:05 (100%)")));
}

#[test]
fn seekbar_draws_secondary_progress_and_mirrors_it_in_rtl() {
    static BAR:Node=Node { kind:Kind::SeekBar,min:0.,max:100.,progress:25.,secondary_progress:75.,mirror_for_rtl:true,..button("bar",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    for (rtl,expected_x) in [(false,8.),(true,54.)] {
        state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f| { BAR.render(f,bounds,&mut state); });
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,.. } if (bounds.width-138.).abs()<0.01 && (bounds.x-expected_x).abs()<0.01)));
    }
    state.secondary_progress.insert("bar".into(),50.);
    let commands=ui.frame(Input::default(),bounds,0.,|f| { BAR.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,.. } if (bounds.width-92.).abs()<0.01)));
}

#[test]
fn image_alpha_affects_drawable_but_not_background() {
    static IMAGE:Node=Node { kind:Kind::Image,image_src:"photo",image_alpha:96,view_alpha:1.0,background:Some(Color(10,20,30,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,..button("photo",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:100.,height:60. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("photo".into(),Image::rgba(1,1,vec![255,255,255,255]).unwrap());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { IMAGE.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Image { color:Color(255,255,255,96),.. })));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(10,20,30,255),.. })));
}

#[test]
fn image_view_wrap_content_remeasures_host_selected_drawable() {
    static LAYOUT:Node=Node { kind:Kind::Linear,children:&[
        Node { kind:Kind::Image,id:"photo",image_src:"small",width:Size::Wrap,height:Size::Wrap,..button("photo",Visibility::Visible,Size::Wrap,0.) },
    ],..button("layout",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:150. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("small".into(),Image::rgba(20,10,vec![255;20*10*4]).unwrap());
    state.images.insert("large".into(),Image::rgba(80,40,vec![255;80*40*4]).unwrap());
    state.image_scales.insert("large".into(),0.5);
    let render=|ui:&mut Ui,state:&mut LayoutState| ui.frame(Input::default(),bounds,0.,|f| { LAYOUT.render(f,bounds,state); });
    let commands=render(&mut ui,&mut state);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Image { bounds,.. } if bounds.width==20. && bounds.height==10.)));
    state.image_sources.insert("photo".into(),"large".into());
    let commands=render(&mut ui,&mut state);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Image { bounds,.. } if bounds.width==40. && bounds.height==20.)));
}

#[test]
fn anonymous_imageswitcher_children_remeasure_overridden_drawables() {
    static SWITCHER:Node=Node { kind:Kind::ImageSwitcher,height:Size::Wrap,children:&[
        Node { kind:Kind::Image,image_src:"small",width:Size::Wrap,height:Size::Wrap,..button("",Visibility::Visible,Size::Wrap,0.) },
        Node { kind:Kind::Image,image_src:"small",width:Size::Wrap,height:Size::Wrap,..button("",Visibility::Visible,Size::Wrap,0.) },
    ],..button("switcher",Visibility::Visible,Size::Wrap,0.) };
    static ROOT:Node=Node { kind:Kind::Linear,children:&[SWITCHER],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:150. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("small".into(),Image::rgba(20,10,vec![255;20*10*4]).unwrap());
    state.images.insert("large".into(),Image::rgba(80,40,vec![255;80*40*4]).unwrap());
    let render=|ui:&mut Ui,state:&mut LayoutState| ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,state); });
    render(&mut ui,&mut state);
    assert!(SWITCHER.set_switcher_image(&mut state,"large"));
    let commands=render(&mut ui,&mut state);
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Image { bounds,image,.. } if image.width()==80 && bounds.width==80. && bounds.height==40.)));
}

#[test]
fn text_switcher_remeasures_anonymous_wrapped_text() {
    static SWITCHER:Node=Node { kind:Kind::TextSwitcher,height:Size::Wrap,children:&[
        Node { kind:Kind::Text,text:"Hi",height:Size::Wrap,..button("",Visibility::Visible,Size::Wrap,0.) },
        Node { kind:Kind::Text,text:"Hi",height:Size::Wrap,..button("",Visibility::Visible,Size::Wrap,0.) },
    ],..button("switcher",Visibility::Visible,Size::Wrap,0.) };
    static ROOT:Node=Node { kind:Kind::Linear,children:&[SWITCHER],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:100.,height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState| ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,state); });
    render(&mut ui,&mut state);
    assert!(SWITCHER.set_switcher_text(&mut state,"One two three four five",true));
    let commands=render(&mut ui,&mut state);
    let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).collect();
    assert_eq!(lines,vec!["One two", "three four", "five"]);
}

#[test]
fn image_button_uses_replaced_drawable_and_density() {
    static BUTTON:Node=Node { kind:Kind::Icon,image_src:"small",image_builtin:false,image_scale:3,..button("picture",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:100.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    state.images.insert("small".into(),Image::rgba(10,10,vec![255;400]).unwrap());
    state.images.insert("large".into(),Image::rgba(40,20,vec![255;3200]).unwrap());
    state.image_scales.insert("large".into(),0.5);
    state.image_sources.insert("picture".into(),"large".into());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { BUTTON.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Image { image,bounds,.. } if image.width()==40 && bounds.width==20. && bounds.height==10.)));
}

#[test]
fn absolute_layout_positions_children_relative_to_padding_in_both_directions() {
    static LAYOUT:Node=Node { kind:Kind::Absolute,padding_edges:[10.,20.,10.,10.],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0],children:&[
        Node { width:Size::Dp(50.),height:Size::Dp(30.),absolute_position:[15.,25.],..button("child",Visibility::Visible,Size::Dp(30.),0.) },
    ],..button("layout",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:150. };let mut ui=Ui::default();let mut state=LayoutState::default();
    for rtl in [false,true] {
        state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f| { LAYOUT.render(f,bounds,&mut state); });
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,bounds,.. } if text.as_ref()=="child" && *bounds==Rect { x:25.,y:45.,width:50.,height:30. })));
    }
}

#[test]
fn absolute_layout_wrap_ignores_gone_children_and_margins() {
    static ABS:Node=Node { kind:Kind::Absolute,width:Size::Wrap,height:Size::Wrap,padding_edges:[10.,10.,10.,10.],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0],background:Some(Color(12,34,56,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,children:&[
        Node { width:Size::Dp(40.),height:Size::Dp(30.),absolute_position:[15.,25.],margin:[30.;4],..button("visible",Visibility::Visible,Size::Dp(30.),0.) },
        Node { width:Size::Dp(40.),height:Size::Dp(30.),absolute_position:[120.,140.],..button("gone",Visibility::Gone,Size::Dp(30.),0.) },
    ],..button("abs",Visibility::Visible,Size::Wrap,0.) };
    static ROOT:Node=Node { kind:Kind::Linear,children:&[ABS],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:300.,height:300. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(12,34,56,255),.. } if bounds.width==75. && bounds.height==75.)));
}

#[test]
fn viewgroup_clip_children_controls_nested_child_bounds() {
    static NESTED:Node=Node { kind:Kind::Frame,width:Size::Dp(80.),height:Size::Dp(60.),layout_gravity:[1,1],absolute_position:[0.,0.],clip_children_disabled:true,children:&[
        Node { kind:Kind::View,width:Size::Dp(160.),height:Size::Dp(40.),layout_gravity:[1,1],absolute_position:[0.,0.],background:Some(Color(5,6,7,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,..button("wide",Visibility::Visible,Size::Dp(40.),0.) },
    ],..button("nested",Visibility::Visible,Size::Dp(60.),0.) };
    static CLIPPED:Node=Node { kind:Kind::Frame,children:&[NESTED],..button("root",Visibility::Visible,Size::Match,0.) };
    static OPEN:Node=Node { clip_children_disabled:true,..CLIPPED };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let nested_bounds=Rect { x:60.,y:20.,width:80.,height:60. };
    for (node,clipped) in [(&CLIPPED,true),(&OPEN,false)] {
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        assert_eq!(commands.iter().any(|c| matches!(c,DrawCommand::PushClip(r) if *r==nested_bounds)),clipped);
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(5,6,7,255),.. } if bounds.width==160.)));
    }
}

#[test]
fn view_alpha_composes_with_children_and_keeps_clicks_enabled() {
    static ROOT:Node=Node { kind:Kind::Frame,view_alpha:0.5,background:Some(Color(100,100,100,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,children:&[
        Node { kind:Kind::Button,view_alpha:0.5,background:Some(Color(20,30,40,255)),foreground:None,foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,..button("button",Visibility::Visible,Size::Match,0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    static CLEAR:Node=Node { view_alpha:0.,..button("clear",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let layers:Vec<_>=commands.iter().enumerate().filter_map(|(i,c)| match c {
        DrawCommand::PushLayer { bounds:b,alpha } => Some((i,*b,*alpha)),
        _ => None,
    }).collect();
    let pops:Vec<_>=commands.iter().enumerate().filter_map(|(i,c)| matches!(c,DrawCommand::PopLayer).then_some(i)).collect();
    assert_eq!(layers.len(),2);
    assert_eq!(pops.len(),2);
    assert_eq!(layers[0].1,bounds);
    assert!((layers[0].2-0.5).abs()<1e-6);
    assert_eq!(layers[1].1,bounds);
    assert!((layers[1].2-0.5).abs()<1e-6);
    let root_bg=commands.iter().position(|c| matches!(c,DrawCommand::RoundedRect { color:Color(100,100,100,255),.. })).unwrap();
    let child_bg=commands.iter().position(|c| matches!(c,DrawCommand::RoundedRect { color:Color(20,30,40,255),.. })).unwrap();
    assert!(layers[0].0<root_bg && root_bg<layers[1].0 && layers[1].0<child_bg && child_bg<pops[0] && pops[0]<pops[1]);
    let mut action=None;
    let mut clear_cmds=Vec::new();
    for input in [Input { pointer:Some([20.,20.]),pressed:true,..Input::default() },Input { pointer:Some([20.,20.]),released:true,..Input::default() }] {
        clear_cmds=ui.frame(input,bounds,0.,|f| { action=CLEAR.render(f,bounds,&mut state); });
    }
    assert!(clear_cmds.iter().any(|c| matches!(c,DrawCommand::PushLayer { alpha,.. } if *alpha==0.0)));
    assert_eq!(action,Some("clear"));
}

#[test]
fn scroll_view_accepts_scroll_on_visible_padding_when_unclipped() {
    static CONTENT:Node=Node { kind:Kind::View,width:Size::Match,height:Size::Dp(300.),..button("content",Visibility::Visible,Size::Dp(300.),0.) };
    static SCROLL:Node=Node { kind:Kind::Scroll,padding_edges:[20.,20.,20.,20.],padding_explicit:[false;4],background_padding:None,background_min_size:[0.0,0.0],clip_to_padding_disabled:true,children:&[CONTENT],..button("scroll",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input { pointer:Some([10.,50.]),scroll:[0.,25.],..Input::default() },bounds,0.,|f| { SCROLL.render(f,bounds,&mut state); });
    assert_eq!(state.offsets["scroll"],25.);
}

#[test]
fn zoom_button_tap_and_long_press_repeat_until_release() {
    static ZOOM:Node=Node { kind:Kind::Zoom,..button("zoom",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:80.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,input:Input| { let mut action=None;ui.frame(input,bounds,0.,|f| { action=ZOOM.render(f,bounds,state); });action };
    assert_eq!(render(&mut ui,&mut state,Input { time:1.,pointer:Some([20.,20.]),pressed:true,..Input::default() }),None);
    assert_eq!(ui.next_wakeup(),Some(1.5));
    assert_eq!(render(&mut ui,&mut state,Input { time:1.5,pointer:Some([20.,20.]),..Input::default() }),Some("zoom"));
    assert_eq!(ui.next_wakeup(),Some(2.5));
    assert_eq!(render(&mut ui,&mut state,Input { time:2.5,pointer:Some([20.,20.]),..Input::default() }),Some("zoom"));
    assert_eq!(render(&mut ui,&mut state,Input { time:2.6,pointer:Some([20.,20.]),released:true,..Input::default() }),None);
    assert_eq!(render(&mut ui,&mut state,Input { time:3.,pointer:Some([20.,20.]),pressed:true,..Input::default() }),None);
    assert_eq!(render(&mut ui,&mut state,Input { time:3.1,pointer:Some([20.,20.]),released:true,..Input::default() }),Some("zoom"));
}

#[test]
fn zoom_button_repeat_interval_can_change_during_a_hold() {
    static ZOOM:Node=Node { kind:Kind::Zoom,..button("zoom",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:80.,height:48. }; let mut ui=Ui::default(); let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,input:Input| { let mut action=None;ui.frame(input,bounds,0.,|f| { action=ZOOM.render(f,bounds,state); });action };
    state.zoom_speed_ms.insert("zoom".into(),250);
    render(&mut ui,&mut state,Input { time:1.,pointer:Some([20.,20.]),pressed:true,..Input::default() });
    assert_eq!(render(&mut ui,&mut state,Input { time:1.5,pointer:Some([20.,20.]),..Input::default() }),Some("zoom"));
    assert_eq!(ui.next_wakeup(),Some(1.75));
    state.zoom_speed_ms.insert("zoom".into(),500);
    assert_eq!(render(&mut ui,&mut state,Input { time:1.75,pointer:Some([20.,20.]),..Input::default() }),Some("zoom"));
    assert_eq!(ui.next_wakeup(),Some(2.25));
    assert_eq!(render(&mut ui,&mut state,Input { time:2.,pointer:Some([20.,20.]),released:true,..Input::default() }),None);
    assert!(state.zoom_holds.is_empty());
}

#[test]
fn zoom_button_cancels_pending_hold_when_pointer_leaves_and_on_cancel() {
    static ZOOM:Node=Node { kind:Kind::Zoom,..button("zoom",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:80.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,input:Input| { let mut action=None;ui.frame(input,bounds,0.,|f| { action=ZOOM.render(f,bounds,state); });action };
    render(&mut ui,&mut state,Input { time:1.,pointer:Some([20.,20.]),pressed:true,..Input::default() });
    assert_eq!(render(&mut ui,&mut state,Input { time:1.2,pointer:Some([100.,20.]),..Input::default() }),None);
    assert_eq!(ui.next_wakeup(),None);
    assert_eq!(render(&mut ui,&mut state,Input { time:1.6,pointer:Some([20.,20.]),..Input::default() }),None);
    assert_eq!(render(&mut ui,&mut state,Input { time:1.7,pointer:Some([20.,20.]),released:true,..Input::default() }),None);
    render(&mut ui,&mut state,Input { time:2.,pointer:Some([20.,20.]),pressed:true,..Input::default() });
    assert_eq!(render(&mut ui,&mut state,Input { time:2.1,cancelled:true,..Input::default() }),None);
    assert!(state.zoom_holds.is_empty());
}

#[test]
fn view_foreground_draws_over_children_and_respects_view_opacity() {
    static ROOT:Node=Node { kind:Kind::Frame,background:Some(Color(10,20,30,255)),foreground:Some(Color(100,0,0,128)),foreground_radius:0.0,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,view_alpha:0.5,children:&[
        Node { kind:Kind::View,background:Some(Color(0,50,0,255)),..button("child",Visibility::Visible,Size::Match,0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:100.,height:60. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let push=commands.iter().position(|c| matches!(c,DrawCommand::PushLayer { alpha,.. } if (*alpha-0.5).abs()<1e-6)).unwrap();
    let child=commands.iter().position(|c| matches!(c,DrawCommand::RoundedRect { color:Color(0,50,0,255),.. })).unwrap();
    let foreground=commands.iter().position(|c| matches!(c,DrawCommand::RoundedRect { color:Color(100,0,0,128),.. })).unwrap();
    let pop=commands.iter().position(|c| matches!(c,DrawCommand::PopLayer)).unwrap();
    assert!(push<child && child<foreground && foreground<pop);
    state.visibility.insert("root".into(),Visibility::Gone);
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(100,0,0,_),.. })));
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::PushLayer { .. })));
}

#[test]
fn shape_foreground_renders_rounded_corners_and_oval() {
    static ROUNDED:Node=Node { foreground:Some(Color(5,6,7,128)),foreground_radius:12.,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,..button("round",Visibility::Visible,Size::Match,0.) };
    static OVAL:Node=Node { foreground_radius:-1.,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,background_stroke:None,background_stroke_width:0.0,foreground_stroke:None,foreground_stroke_width:0.0,..ROUNDED };
    let bounds=Rect { x:0.,y:0.,width:100.,height:40. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROUNDED.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { radius,color:Color(5,6,7,128),.. } if *radius==12.)));
    let commands=ui.frame(Input::default(),bounds,0.,|f| { OVAL.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Ellipse { bounds:r,color:Color(5,6,7,128) } if *r==bounds)));
}

#[test]
fn shape_strokes_paint_in_background_and_foreground_order() {
    static ROOT:Node=Node { kind:Kind::Frame,background:Some(Color(1,2,3,255)),radius:12.,background_stroke:Some(Color(4,5,6,255)),background_stroke_width:3.,foreground_stroke:Some(Color(7,8,9,255)),foreground_stroke_width:2.,children:&[
        Node { kind:Kind::View,background:Some(Color(10,20,30,255)),..button("child",Visibility::Visible,Size::Match,0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:100.,height:60. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let bg=commands.iter().position(|c| matches!(c,DrawCommand::Outline { radius,width,color:Color(4,5,6,255),.. } if *radius==12. && *width==3.)).unwrap();
    let child=commands.iter().position(|c| matches!(c,DrawCommand::RoundedRect { color:Color(10,20,30,255),.. })).unwrap();
    let fg=commands.iter().position(|c| matches!(c,DrawCommand::Outline { width,color:Color(7,8,9,255),.. } if *width==2.)).unwrap();
    assert!(bg<child && child<fg);
}

#[test]
fn oval_drawable_keeps_oval_stroke_geometry() {
    static OVAL:Node=Node { kind:Kind::View,background:Some(Color(1,2,3,255)),radius:-1.,background_stroke:Some(Color(4,5,6,255)),background_stroke_width:4.,..button("oval",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:120.,height:60. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { OVAL.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Ellipse { bounds:r,.. } if *r==bounds)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Outline { bounds:r,radius,width,color:Color(4,5,6,255) } if *r==bounds && *radius<0. && *width==4.)));
}

#[test]
fn outline_only_foreground_leaves_child_uncovered() {
    static ROOT:Node=Node { kind:Kind::Frame,foreground:None,foreground_radius:-1.,foreground_size:[0.0,0.0],foreground_padding:None,foreground_gravity:[5,5],foreground_excludes_padding:false,foreground_stroke:Some(Color(122,51,166,255)),foreground_stroke_width:4.,children:&[
        Node { kind:Kind::View,background:Some(Color(1,2,3,255)),..button("child",Visibility::Visible,Size::Match,0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:100.,height:60. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Outline { radius,color:Color(122,51,166,255),.. } if *radius<0.)));
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::Ellipse { .. })));
}

#[test]
fn drawable_padding_resolves_direction_and_explicit_overrides() {
    static FRAME:Node=Node { kind:Kind::Frame,background_padding:Some([40.,12.,8.,6.]),background_min_size:[0.0,0.0],padding_explicit:[true,false,false,false],padding_edges:[0.,0.,0.,0.],children:&[
        Node { kind:Kind::View,background:Some(Color(1,2,3,255)),..button("child",Visibility::Visible,Size::Match,0.) },
    ],..button("frame",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    for (rtl,expected) in [(false,Rect { x:0.,y:12.,width:192.,height:82. }),(true,Rect { x:0.,y:12.,width:160.,height:82. })] {
        state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if *bounds==expected)));
    }
}

#[test]
fn shape_background_size_affects_wrap_but_not_fixed_dimensions() {
    static FRAME:Node=Node { kind:Kind::Frame,children:&[
        Node { kind:Kind::View,id:"sized",width:Size::Wrap,height:Size::Wrap,background:Some(Color(1,2,3,255)),background_min_size:[160.,60.],..button("sized",Visibility::Visible,Size::Wrap,0.) },
        Node { kind:Kind::View,id:"fixed",width:Size::Dp(30.),height:Size::Dp(20.),background:Some(Color(3,2,1,255)),background_min_size:[160.,60.],..button("fixed",Visibility::Visible,Size::Dp(20.),0.) },
    ],..button("frame",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:300.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if bounds.width==160. && bounds.height==60.)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(3,2,1,255),.. } if bounds.width==30. && bounds.height==20.)));
}

#[test]
fn foreground_gravity_positions_intrinsic_size_with_padding_and_rtl() {
    static FRAME:Node=Node { kind:Kind::Frame,padding_edges:[10.,20.,30.,10.],foreground:Some(Color(1,2,3,255)),foreground_size:[40.,20.],foreground_padding:None,foreground_gravity:[2,2],foreground_excludes_padding:true,..button("frame",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:100. };let mut ui=Ui::default();let mut state=LayoutState::default();
    for (rtl,expected) in [(false,Rect { x:130.,y:70.,width:40.,height:20. }),(true,Rect { x:10.,y:70.,width:40.,height:20. })] {
        state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f| { FRAME.render(f,bounds,&mut state); });
        assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if *bounds==expected)));
    }
}

#[test]
fn frame_layout_measures_foreground_and_applies_its_fill_padding() {
    static FRAME:Node=Node { kind:Kind::Frame,width:Size::Wrap,height:Size::Wrap,background:Some(Color(10,20,30,255)),foreground_stroke:Some(Color(1,2,3,255)),foreground_stroke_width:2.,foreground_size:[180.,70.],foreground_padding:Some([30.,14.,12.,14.]),children:&[
        Node { kind:Kind::View,background:Some(Color(4,5,6,255)),..button("child",Visibility::Visible,Size::Match,0.) },
    ],..button("frame",Visibility::Visible,Size::Wrap,0.) };
    static ROOT:Node=Node { kind:Kind::Linear,children:&[FRAME],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:300.,height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(10,20,30,255),.. } if bounds.width==180. && bounds.height==70.)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(4,5,6,255),.. } if *bounds==Rect { x:30.,y:14.,width:138.,height:42. })));
}

#[test]
fn frame_measure_all_children_includes_gone_only_in_measurement() {
    static FRAME:Node=Node { kind:Kind::Frame,width:Size::Wrap,height:Size::Wrap,background:Some(Color(1,2,3,255)),children:&[
        Node { kind:Kind::View,width:Size::Dp(80.),height:Size::Dp(40.),..button("visible",Visibility::Visible,Size::Dp(40.),0.) },
        Node { kind:Kind::View,width:Size::Dp(180.),height:Size::Dp(70.),background:Some(Color(9,9,9,255)),..button("gone",Visibility::Gone,Size::Dp(70.),0.) },
    ],..button("frame",Visibility::Visible,Size::Wrap,0.) };
    static ALL:Node=Node { measure_all_children:true,..FRAME };
    static ROOT:Node=Node { kind:Kind::Linear,children:&[FRAME,ALL],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:300.,height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let dims:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. }=>Some((bounds.width,bounds.height)),_=>None }).collect();
    assert_eq!(dims,vec![(80.,40.),(180.,70.)]);
    assert!(!commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { color:Color(9,9,9,255),.. })));
}

#[test]
fn text_view_min_lines_reserves_height_for_short_content() {
    static ROOT:Node=Node { kind:Kind::Linear,children:&[
        Node { kind:Kind::Text,text:"Short",height:Size::Wrap,min_lines:3,background:Some(Color(1,2,3,255)),..button("text",Visibility::Visible,Size::Wrap,0.) },
        Node { kind:Kind::View,height:Size::Dp(10.),background:Some(Color(4,5,6,255)),..button("next",Visibility::Visible,Size::Dp(10.),0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if bounds.height==72.)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(4,5,6,255),.. } if bounds.y==72.)));
}

#[test]
fn text_view_exact_lines_reserves_and_limits_visible_lines() {
    static ROOT:Node=Node { kind:Kind::Linear,children:&[
        Node { kind:Kind::Text,text:"Short",height:Size::Wrap,min_lines:2,max_lines:2,background:Some(Color(1,2,3,255)),..button("short",Visibility::Visible,Size::Wrap,0.) },
        Node { kind:Kind::Text,text:"One two three four five six seven eight",height:Size::Wrap,min_lines:2,max_lines:2,background:Some(Color(4,5,6,255)),..button("long",Visibility::Visible,Size::Wrap,0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:100.,height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if bounds.height==48.)));
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(4,5,6,255),.. } if bounds.y==48. && bounds.height==48.)));
    let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).collect();
    assert_eq!(lines,vec!["Short","One two","three four"]);
}

#[test]
fn text_view_line_spacing_changes_baselines_and_wrap_height() {
    static ROOT:Node=Node { kind:Kind::Linear,children:&[
        Node { kind:Kind::Text,text:"One\nTwo\nThree",height:Size::Wrap,line_spacing_extra:6.,line_spacing_multiplier:1.5,background:Some(Color(1,2,3,255)),..button("text",Visibility::Visible,Size::Wrap,0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:200.,height:200. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if bounds.height==108.)));
    let ys:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { bounds,.. }=>Some(bounds.y),_=>None }).collect();
    assert_eq!(ys,vec![0.,42.,84.]);
}

#[test]
fn text_view_letter_spacing_changes_wrapping_and_draw_style() {
    static ROOT:Node=Node { kind:Kind::Linear,children:&[
        Node { kind:Kind::Text,text:"abcdefghij",height:Size::Wrap,letter_spacing_em:0.25,background:Some(Color(1,2,3,255)),..button("text",Visibility::Visible,Size::Wrap,0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:100.,height:150. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::RoundedRect { bounds,color:Color(1,2,3,255),.. } if bounds.height==48.)));
    let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,style,.. } if style.letter_spacing==4.=>Some(text.as_ref()),_=>None }).collect();
    assert_eq!(lines,vec!["abcdefgh","ij"]);
}

#[test]
fn multiline_ellipsis_respects_narrow_width_and_combined_characters() {
    static TEXT:Node=Node { kind:Kind::Text,text:"a\u{301}b\u{301}c\u{301}d\u{301}",max_lines:1,ellipsize_end:true,..button("text",Visibility::Visible,Size::Match,0.) };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    let tiny=Rect { x:0.,y:0.,width:5.,height:48. };
    let commands=ui.frame(Input::default(),tiny,0.,|f| { TEXT.render(f,tiny,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.is_empty())));
    let narrow=Rect { x:0.,y:0.,width:20.,height:48. };
    let commands=ui.frame(Input::default(),narrow,0.,|f| { TEXT.render(f,narrow,&mut state); });
    assert!(commands.iter().any(|c| matches!(c,DrawCommand::Text { text,.. } if text.as_ref()=="a\u{301}…")));
}

#[test]
fn text_view_preserves_non_breaking_spaces_when_wrapping() {
    static ROOT:Node=Node { kind:Kind::Linear,children:&[
        Node { kind:Kind::Text,text:"A B\u{a0}C",height:Size::Wrap,..button("text",Visibility::Visible,Size::Wrap,0.) },
    ],..button("root",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect { x:0.,y:0.,width:40.,height:120. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ROOT.render(f,bounds,&mut state); });
    let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).collect();
    assert_eq!(lines,vec!["A","B\u{a0}C"]);
}

#[test]
fn text_view_wraps_at_unicode_hyphens_cjk_and_zero_width_spaces() {
    let mut ui=Ui::default();let mut state=LayoutState::default();
    for (text,width,expected) in [
        ("ab-cdef",40.,vec!["ab-","cdef"]),
        ("你好，世界",32.,vec!["你","好，","世界"]),
        ("ab\u{200b}cd",20.,vec!["ab","cd"]),
    ] {
        let node=Box::leak(Box::new(Node { kind:Kind::Text,text,..button("text",Visibility::Visible,Size::Match,0.) }));
        let bounds=Rect { x:0.,y:0.,width,height:200. };
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).collect();
        assert_eq!(lines,expected,"{text:?}");
    }
}

#[test]
fn text_view_honors_unicode_mandatory_line_separators_and_crlf() {
    let mut ui=Ui::default();let mut state=LayoutState::default();
    for (text,expected) in [
        ("one\r\ntwo",vec!["one","two"]),
        ("one\rtwo\u{2028}three\u{2029}",vec!["one","two","three",""]),
        ("a\u{85}b\u{b}c\u{c}d",vec!["a","b","c","d"]),
        ("\n\n",vec!["","",""]),
    ] {
        let node=Box::leak(Box::new(Node { kind:Kind::Text,text,..button("text",Visibility::Visible,Size::Match,0.) }));
        let bounds=Rect { x:0.,y:0.,width:200.,height:200. };
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).collect();
        assert_eq!(lines,expected,"{text:?}");
    }
}

#[test]
fn text_view_shows_soft_hyphen_only_when_broken() {
    static TEXT:Node=Node { kind:Kind::Text,text:"ab\u{ad}cd",..button("text",Visibility::Visible,Size::Match,0.) };
    let mut ui=Ui::default();let mut state=LayoutState::default();
    for (width,expected) in [(27.,vec!["ab-","cd"]),(100.,vec!["abcd"])] {
        let bounds=Rect { x:0.,y:0.,width,height:100. };
        let commands=ui.frame(Input::default(),bounds,0.,|f| { TEXT.render(f,bounds,&mut state); });
        let lines:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).collect();
        assert_eq!(lines,expected);
    }
}

#[test]
fn ellipsized_line_drops_only_inserted_soft_hyphen() {
    let mut ui=Ui::default();let mut state=LayoutState::default();
    for (text,expected) in [("ab\u{ad}cdef","ab…"),("ab-cdef","ab-…")] {
        let node=Box::leak(Box::new(Node { kind:Kind::Text,text,max_lines:1,ellipsize_end:true,..button("text",Visibility::Visible,Size::Match,0.) }));
        let bounds=Rect { x:0.,y:0.,width:38.,height:48. };
        let commands=ui.frame(Input::default(),bounds,0.,|f| { node.render(f,bounds,&mut state); });
        let actual=commands.iter().find_map(|c| match c { DrawCommand::Text { text,.. }=>Some(text.as_ref()),_=>None }).unwrap();
        assert_eq!(actual,expected,"{text:?}");
    }
}

#[test]
fn zoom_controls_report_independent_actions_and_repeat() {
    static ZOOM:Node=Node { kind:Kind::ZoomControls, width:Size::Dp(158.), ..button("controls",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:158.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,input:Input| { let mut action=None;ui.frame(input,bounds,0.,|f| { action=ZOOM.render(f,bounds,state); });action };
    render(&mut ui,&mut state,Input { time:1.,pointer:Some([39.,24.]),pressed:true,..Input::default() });
    assert_eq!(render(&mut ui,&mut state,Input { time:1.1,pointer:Some([39.,24.]),released:true,..Input::default() }),Some("controls"));
    assert_eq!(state.zoom_controls_actions.get("controls"),Some(&-1));
    state.zoom_controls_enabled.insert("controls".into(),[true,false]);
    render(&mut ui,&mut state,Input { time:2.,pointer:Some([118.,24.]),pressed:true,..Input::default() });
    assert_eq!(render(&mut ui,&mut state,Input { time:2.1,pointer:Some([118.,24.]),released:true,..Input::default() }),None);
    state.zoom_controls_enabled.insert("controls".into(),[true,true]);
    state.zoom_speed_ms.insert("controls".into(),250);
    render(&mut ui,&mut state,Input { time:3.,pointer:Some([118.,24.]),pressed:true,..Input::default() });
    assert_eq!(render(&mut ui,&mut state,Input { time:3.5,pointer:Some([118.,24.]),..Input::default() }),Some("controls"));
    assert_eq!(state.zoom_controls_actions.get("controls"),Some(&1));assert_eq!(ui.next_wakeup(),Some(3.75));
    assert_eq!(render(&mut ui,&mut state,Input { time:3.6,pointer:Some([118.,24.]),released:true,..Input::default() }),None);
}

#[test]
fn zoom_controls_fade_can_reverse_and_finishes_hidden() {
    static ZOOM:Node=Node { kind:Kind::ZoomControls, width:Size::Dp(158.), ..button("controls",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:158.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,time:f64| ui.frame(Input { time,..Input::default() },bounds,0.,|f| { ZOOM.render(f,bounds,state); });
    state.set_zoom_controls_visible("controls",false,1.0);
    let half=render(&mut ui,&mut state,1.25);
    let alpha=half.iter().find_map(|c|match c { DrawCommand::Image { color:Color(_,_,_,a),.. }=>Some(*a),_=>None }).unwrap();
    assert!((126..=129).contains(&alpha));
    state.set_zoom_controls_visible("controls",true,1.25);
    let reversed=render(&mut ui,&mut state,1.25);
    let alpha2=reversed.iter().find_map(|c|match c { DrawCommand::Image { color:Color(_,_,_,a),.. }=>Some(*a),_=>None }).unwrap();
    assert_eq!(alpha,alpha2);
    render(&mut ui,&mut state,1.75);
    assert_eq!(state.visibility.get("controls"),Some(&Visibility::Visible));
    state.set_zoom_controls_visible("controls",false,2.0);
    render(&mut ui,&mut state,2.5);
    assert_eq!(state.visibility.get("controls"),Some(&Visibility::Gone));
    assert!(ui.next_wakeup().is_some());
    render(&mut ui,&mut state,2.51);
    assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="Zoom out"));
}

#[test]
fn zoom_controls_nine_patch_preserves_fixed_parts_when_constrained() {
    static ZOOM:Node=Node { kind:Kind::ZoomControls, width:Size::Dp(156.), ..button("controls",Visibility::Visible,Size::Dp(48.),0.) };
    let bounds=Rect { x:0.,y:0.,width:156.,height:48. };let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f| { ZOOM.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image { bounds,uv,.. } if bounds.x==0. && bounds.width==74. && (uv[2]-74.0/79.0).abs()<0.00001)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image { bounds,uv,.. } if bounds.x==74. && bounds.width==4. && (uv[0]-75.0/79.0).abs()<0.00001)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image { bounds,.. } if bounds.x==78. && bounds.width==4.)));
}

#[test]
fn image_view_adjust_bounds_uses_drawable_aspect_and_padding() {
    static COLUMN:Node=Node { kind:Kind::Linear,children:&[
        Node { kind:Kind::Image,id:"wide_fixed",width:Size::Dp(180.),height:Size::Wrap,image_src:"wide",image_adjust_bounds:true,padding_edges:[10.;4],padding_explicit:[true;4],..button("wide_fixed",Visibility::Visible,Size::Wrap,0.) },
        Node { kind:Kind::Image,id:"tall_fixed",width:Size::Wrap,height:Size::Dp(60.),image_src:"wide",image_adjust_bounds:true,padding_edges:[10.;4],padding_explicit:[true;4],..button("tall_fixed",Visibility::Visible,Size::Dp(60.),0.) },
        button("after",Visibility::Visible,Size::Dp(40.),0.),
    ],..button("column",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect{x:0.,y:0.,width:300.,height:240.};let mut state=LayoutState::default();let mut ui=Ui::default();
    state.images.insert("wide".into(),Image::rgba(80,40,vec![255;80*40*4]).unwrap());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,&mut state); });
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image { bounds,.. } if *bounds==Rect{x:10.,y:10.,width:160.,height:80.})));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image { bounds,.. } if *bounds==Rect{x:10.,y:110.,width:80.,height:40.})));
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y,160.);
}

#[test]
fn image_button_adjusts_after_drawable_replacement() {
    static COLUMN:Node=Node { kind:Kind::Linear,children:&[
        Node { kind:Kind::Icon,id:"image_button",width:Size::Dp(120.),height:Size::Wrap,image_src:"wide",image_adjust_bounds:true,image_builtin:false,..button("image_button",Visibility::Visible,Size::Wrap,0.) },
        button("after",Visibility::Visible,Size::Dp(40.),0.),
    ],..button("column",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect{x:0.,y:0.,width:180.,height:400.};let mut state=LayoutState::default();let mut ui=Ui::default();
    state.images.insert("wide".into(),Image::rgba(80,40,vec![255;80*40*4]).unwrap());
    state.images.insert("tall".into(),Image::rgba(40,80,vec![255;40*80*4]).unwrap());
    ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,&mut state); });
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y,60.);
    state.image_sources.insert("image_button".into(),"tall".into());
    ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,&mut state); });
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y,240.);
}

#[test]
fn image_adjust_bounds_honors_maximum_dimensions_only_on_wrapping_axes() {
    const fn image(id:&'static str,width:Size,height:Size,max_width:Option<f32>,max_height:Option<f32>) -> Node {
        Node { kind:Kind::Image,id,width,height,image_src:"wide",image_adjust_bounds:true,image_max_width:max_width,image_max_height:max_height,background:Some(Color(10,20,30,255)),..button(id,Visibility::Visible,height,0.) }
    }
    static COLUMN:Node=Node { kind:Kind::Linear,children:&[
        image("height_cap",Size::Dp(180.),Size::Wrap,None,Some(60.)),
        image("width_cap",Size::Wrap,Size::Dp(60.),Some(70.),None),
        image("both_wrap",Size::Wrap,Size::Wrap,Some(100.),Some(30.)),
        image("fixed",Size::Dp(180.),Size::Dp(60.),Some(10.),Some(10.)),
    ],..button("column",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect{x:0.,y:0.,width:300.,height:250.};let mut state=LayoutState::default();let mut ui=Ui::default();
    state.images.insert("wide".into(),Image::rgba(80,40,vec![255;80*40*4]).unwrap());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,&mut state); });
    let actual:Vec<_>=commands.iter().filter_map(|c| match c { DrawCommand::RoundedRect { bounds,color:Color(10,20,30,255),.. }=>Some(*bounds),_=>None }).collect();
    assert_eq!(actual,vec![Rect{x:0.,y:0.,width:180.,height:60.},Rect{x:0.,y:60.,width:70.,height:60.},Rect{x:0.,y:120.,width:60.,height:30.},Rect{x:0.,y:150.,width:180.,height:60.}]);
}

#[test]
fn image_tint_modes_preserve_alpha_and_cache_drawables() {
    const fn tinted(id:&'static str,mode:u8) -> Node {
        Node { kind:Kind::Image,id,width:Size::Dp(40.),height:Size::Dp(40.),image_src:"pixel",image_tint:Some(Color(200,0,100,128)),image_tint_mode:mode,image_alpha:128,..button(id,Visibility::Visible,Size::Dp(40.),0.) }
    }
    static COLUMN:Node=Node { kind:Kind::Linear,children:&[tinted("atop",0),tinted("in",1)],..button("column",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect{x:0.,y:0.,width:100.,height:100.};let mut state=LayoutState::default();let mut ui=Ui::default();
    state.images.insert("pixel".into(),Image::rgba(1,1,vec![20,100,200,128]).unwrap());
    let render=|ui:&mut Ui,state:&mut LayoutState| ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,state); });
    let commands=render(&mut ui,&mut state);
    let images:Vec<_>=commands.iter().filter_map(|c|match c { DrawCommand::Image { image,color,.. }=>Some((image.clone(),*color)),_=>None }).collect();
    assert_eq!(images.len(),2);
    assert_eq!(images[0].0.pixels(),&[110,50,150,128]);
    assert_eq!(images[1].0.pixels(),&[200,0,100,64]);
    assert_eq!(images[0].1,Color(255,255,255,128));
    let again=render(&mut ui,&mut state);
    let ids:Vec<_>=again.iter().filter_map(|c|match c {DrawCommand::Image { image,.. }=>Some(image.id()),_=>None}).collect();
    assert_eq!(ids,vec![images[0].0.id(),images[1].0.id()]);
    state.images.insert("pixel".into(),Image::rgba(1,1,vec![100,0,0,255]).unwrap());
    let replaced=render(&mut ui,&mut state);
    let first=replaced.iter().find_map(|c|match c {DrawCommand::Image { image,.. }=>Some(image),_=>None}).unwrap();
    assert_ne!(first.id(),ids[0]);
    assert_eq!(first.pixels(),&[150,0,50,255]);
}

#[test]
fn image_tint_remaining_modes_blend_premultiplied_channels() {
    const fn tinted(id:&'static str,mode:u8) -> Node {
        Node { kind:Kind::Image,id,width:Size::Dp(40.),height:Size::Dp(40.),image_src:"pixel",image_tint:Some(Color(200,0,100,128)),image_tint_mode:mode,..button(id,Visibility::Visible,Size::Dp(40.),0.) }
    }
    static COLUMN:Node=Node { kind:Kind::Linear,children:&[tinted("over",2),tinted("multiply",3),tinted("screen",4),tinted("add",5)],..button("column",Visibility::Visible,Size::Match,0.) };
    let bounds=Rect{x:0.,y:0.,width:100.,height:180.};let mut state=LayoutState::default();let mut ui=Ui::default();
    state.images.insert("pixel".into(),Image::rgba(1,1,vec![20,100,200,128]).unwrap());
    let commands=ui.frame(Input::default(),bounds,0.,|f| { COLUMN.render(f,bounds,&mut state); });
    let pixels:Vec<_>=commands.iter().filter_map(|c|match c {DrawCommand::Image { image,.. }=>Some(image.pixels().to_vec()),_=>None}).collect();
    assert_eq!(pixels,vec![vec![140,33,133,192],vec![16,0,78,64],vec![142,67,174,192],vec![110,50,151,255]]);
}

#[test]
fn image_tint_selector_matches_disabled_pressed_and_selected_in_order() {
    static RULES:&[TintState]=&[
        TintState { required:0,forbidden:1,color:Color(120,120,120,255) },
        TintState { required:3,forbidden:0,color:Color(255,0,0,255) },
        TintState { required:8,forbidden:0,color:Color(0,0,255,255) },
        TintState { required:0,forbidden:0,color:Color(0,180,0,255) },
    ];
    static IMAGE:Node=Node {kind:Kind::Image,id:"image",width:Size::Dp(40.),height:Size::Dp(40.),image_src:"pixel",image_tint_states:RULES,image_tint_mode:1,clickable:true,..button("image",Visibility::Visible,Size::Dp(40.),0.)};
    static DISABLED:Node=Node {enabled:false,..IMAGE};
    let bounds=Rect{x:0.,y:0.,width:40.,height:40.};let mut state=LayoutState::default();let mut ui=Ui::default();
    state.images.insert("pixel".into(),Image::rgba(1,1,vec![255;4]).unwrap());
    let render=|ui:&mut Ui,state:&mut LayoutState,node:&'static Node,input:Input| {
        let commands=ui.frame(input,bounds,0.,|f| { node.render(f,bounds,state); });
        commands.iter().find_map(|c|match c {DrawCommand::Image { image,.. }=>Some(image.pixels().to_vec()),_=>None}).unwrap()
    };
    assert_eq!(render(&mut ui,&mut state,&IMAGE,Input::default()),vec![0,180,0,255]);
    state.view_selected.insert("image".into(),true);
    assert_eq!(render(&mut ui,&mut state,&IMAGE,Input::default()),vec![0,0,255,255]);
    assert_eq!(render(&mut ui,&mut state,&IMAGE,Input { pointer:Some([20.,20.]),pressed:true,..Input::default() }),vec![255,0,0,255]);
    assert_eq!(render(&mut ui,&mut state,&IMAGE,Input { pointer:Some([20.,20.]),released:true,..Input::default() }),vec![0,0,255,255]);
    assert_eq!(render(&mut ui,&mut state,&DISABLED,Input::default()),vec![120,120,120,255]);
}

#[test]
fn text_color_selector_changes_with_view_and_button_states() {
    static RULES:&[TintState]=&[
        TintState {required:0,forbidden:1,color:Color(120,120,120,255)},
        TintState {required:2,forbidden:0,color:Color(255,0,0,255)},
        TintState {required:8,forbidden:0,color:Color(0,0,255,255)},
        TintState {required:0,forbidden:0,color:Color(0,180,0,255)},
    ];
    static TEXT:Node=Node {kind:Kind::Text,id:"sample",text:"Two lines\nof text",width:Size::Dp(120.),height:Size::Dp(48.),clickable:true,text_color_states:RULES,..button("sample",Visibility::Visible,Size::Dp(48.),0.)};
    static DISABLED:Node=Node {enabled:false,..TEXT};
    static BUTTON:Node=Node {kind:Kind::Button,text:"Action",text_color_states:RULES,..TEXT};
    let bounds=Rect{x:0.,y:0.,width:120.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let render=|ui:&mut Ui,state:&mut LayoutState,node:&'static Node,input:Input| {
        let commands=ui.frame(input,bounds,0.,|f| {node.render(f,bounds,state);});
        commands.iter().filter_map(|c|match c {DrawCommand::Text {color,..}=>Some(*color),_=>None}).collect::<Vec<_>>()
    };
    assert_eq!(render(&mut ui,&mut state,&TEXT,Input::default()),vec![Color(0,180,0,255);2]);
    state.view_selected.insert("sample".into(),true);
    assert_eq!(render(&mut ui,&mut state,&TEXT,Input::default()),vec![Color(0,0,255,255);2]);
    assert_eq!(render(&mut ui,&mut state,&TEXT,Input {pointer:Some([10.,10.]),pressed:true,..Input::default()}),vec![Color(255,0,0,255);2]);
    render(&mut ui,&mut state,&TEXT,Input {pointer:Some([10.,10.]),released:true,..Input::default()});
    assert_eq!(render(&mut ui,&mut state,&DISABLED,Input::default()),vec![Color(120,120,120,255);2]);
    state.view_selected.insert("sample".into(),false);
    assert_eq!(render(&mut ui,&mut state,&BUTTON,Input {pointer:Some([10.,10.]),pressed:true,..Input::default()}),vec![Color(255,0,0,255)]);
}

#[test]
fn choice_text_selectors_follow_checked_pressed_and_disabled_states() {
    static RULES:&[TintState]=&[
        TintState {required:0,forbidden:1,color:Color(120,120,120,255)},
        TintState {required:2,forbidden:0,color:Color(255,0,0,255)},
        TintState {required:64,forbidden:0,color:Color(100,0,180,255)},
        TintState {required:0,forbidden:0,color:Color(0,150,0,255)},
    ];
    const fn choice(kind:Kind) -> Node {Node {kind,text:"Choice",text_on:"On",text_off:"Off",text_color_states:RULES,..button("choice",Visibility::Visible,Size::Dp(48.),0.)}}
    static CHOICES:[Node;4]=[choice(Kind::CheckBox),choice(Kind::Switch),choice(Kind::RadioButton),choice(Kind::ToggleButton)];
    static DISABLED:Node=Node {enabled:false,checked:true,..choice(Kind::CheckBox)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};
    for node in &CHOICES {
        let mut ui=Ui::default();let mut state=LayoutState::default();
        let render=|ui:&mut Ui,state:&mut LayoutState,input:Input| {
            let commands=ui.frame(input,bounds,0.,|f|{node.render(f,bounds,state);});
            commands.iter().find_map(|c|match c {DrawCommand::Text {color,..}=>Some(*color),_=>None}).unwrap()
        };
        assert_eq!(render(&mut ui,&mut state,Input::default()),Color(0,150,0,255));
        assert_eq!(render(&mut ui,&mut state,Input {pointer:Some([20.,20.]),pressed:true,..Input::default()}),Color(255,0,0,255));
        assert_eq!(render(&mut ui,&mut state,Input {pointer:Some([20.,20.]),released:true,..Input::default()}),Color(100,0,180,255));
    }
    let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{DISABLED.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {color:Color(120,120,120,255),..})));
}

#[test]
fn relative_layout_resolves_forward_anchors_parent_edges_and_margins() {
    static ROOT:Node=Node {kind:Kind::Relative,padding_edges:[10.;4],padding_explicit:[true;4],children:&[
        Node {width:Size::Dp(100.),height:Size::Dp(30.),margin:[5.,8.,0.,0.],relative_rules:&[RelativeRule{kind:3,anchor:"anchor"},RelativeRule{kind:10,anchor:"anchor"}],..button("below",Visibility::Visible,Size::Dp(30.),0.)},
        Node {width:Size::Dp(80.),height:Size::Dp(40.),margin:[4.,3.,6.,7.],..button("anchor",Visibility::Visible,Size::Dp(40.),0.)},
        Node {width:Size::Dp(50.),height:Size::Dp(20.),margin:[0.,0.,3.,5.],relative_rules:&[RelativeRule{kind:17,anchor:""},RelativeRule{kind:15,anchor:""}],..button("end",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:300.,height:200.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("anchor"),Rect{x:14.,y:13.,width:80.,height:40.});
    assert_eq!(get("below"),Rect{x:19.,y:68.,width:100.,height:30.});
    assert_eq!(get("end"),Rect{x:237.,y:165.,width:50.,height:20.});
}

#[test]
fn relative_layout_follows_gone_anchors_and_wraps_height() {
    static RELATIVE:Node=Node {kind:Kind::Relative,height:Size::Wrap,children:&[
        Node {width:Size::Dp(80.),height:Size::Dp(20.),relative_rules:&[RelativeRule{kind:3,anchor:"gone"}],..button("last",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(80.),height:Size::Dp(30.),relative_rules:&[RelativeRule{kind:3,anchor:"first"}],..button("gone",Visibility::Gone,Size::Dp(30.),0.)},
        Node {width:Size::Dp(80.),height:Size::Dp(40.),..button("first",Visibility::Visible,Size::Dp(40.),0.)},
    ],..button("relative",Visibility::Visible,Size::Wrap,0.)};
    static COLUMN:Node=Node {kind:Kind::Linear,children:&[RELATIVE,button("after",Visibility::Visible,Size::Dp(30.),0.)],..button("column",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:200.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{COLUMN.render(f,bounds,&mut state);});
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="last").unwrap().bounds.y,40.);
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y,60.);
}

#[test]
fn relative_layout_start_end_follow_rtl_and_opposing_edges_fill() {
    static ROOT:Node=Node {kind:Kind::Relative,children:&[
        Node {width:Size::Dp(40.),height:Size::Dp(20.),relative_rules:&[RelativeRule{kind:9,anchor:"anchor"}],..button("end_of",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(60.),height:Size::Dp(20.),..button("anchor",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Wrap,height:Size::Dp(20.),relative_rules:&[RelativeRule{kind:16,anchor:""},RelativeRule{kind:17,anchor:""},RelativeRule{kind:15,anchor:""}],..button("fill",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:240.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=true;
    ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("anchor").x,180.);assert_eq!(get("end_of").x,140.);
    assert_eq!(get("fill"),Rect{x:0.,y:80.,width:240.,height:20.});
}

#[test]
fn relative_layout_aligns_baselines_and_ignores_conflicting_bottom() {
    static ROOT:Node=Node {kind:Kind::Relative,children:&[
        Node {text_size:16.,width:Size::Dp(80.),height:Size::Dp(40.),relative_rules:&[RelativeRule{kind:22,anchor:"large"},RelativeRule{kind:15,anchor:""}],..button("small",Visibility::Visible,Size::Dp(40.),0.)},
        Node {text_size:24.,width:Size::Dp(100.),height:Size::Dp(50.),margin:[0.,20.,0.,0.],..button("large",Visibility::Visible,Size::Dp(50.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:300.,height:200.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    let small=get("small");let large=get("large");
    assert!((small.y+small.height*0.5+16.*0.3 - (large.y+large.height*0.5+24.*0.3)).abs()<0.001);
    assert!(small.y<100.);
}

#[test]
fn relative_layout_gravity_moves_the_whole_group_with_margins() {
    static CENTER:Node=Node {kind:Kind::Relative,content_gravity:[1,1],children:&[
        Node {width:Size::Dp(40.),height:Size::Dp(20.),margin:[5.,2.,5.,2.],..button("first",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(30.),height:Size::Dp(20.),relative_rules:&[RelativeRule{kind:1,anchor:"first"}],..button("second",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{CENTER.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("first"),Rect{x:65.,y:40.,width:40.,height:20.});
    assert_eq!(get("second"),Rect{x:110.,y:38.,width:30.,height:20.});
}

#[test]
fn relative_layout_gravity_end_resolves_rtl() {
    static ROOT:Node=Node {kind:Kind::Relative,content_gravity:[2,2],children:&[
        Node {width:Size::Dp(40.),height:Size::Dp(20.),..button("child",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};
    for (rtl,x) in [(false,160.),(true,0.)] {
        let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
        assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="child").unwrap().bounds,Rect{x,y:80.,width:40.,height:20.});
    }
}

#[test]
fn relative_layout_ignore_gravity_keeps_target_fixed() {
    static ROOT:Node=Node {kind:Kind::Relative,content_gravity:[1,1],relative_rules:&[RelativeRule{kind:23,anchor:"fixed"}],children:&[
        Node {width:Size::Dp(40.),height:Size::Dp(20.),..button("fixed",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(60.),height:Size::Dp(20.),..button("moving",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("fixed"),Rect{x:0.,y:0.,width:40.,height:20.});
    assert_eq!(get("moving"),Rect{x:70.,y:40.,width:60.,height:20.});
}

#[test]
fn relative_layout_ignore_gravity_uses_android_single_axis_bounds() {
    static ROOT:Node=Node {kind:Kind::Relative,content_gravity:[1,0],relative_rules:&[RelativeRule{kind:23,anchor:"fixed"}],children:&[
        Node {width:Size::Dp(40.),height:Size::Dp(20.),margin:[100.,0.,0.,0.],..button("fixed",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(60.),height:Size::Dp(20.),..button("moving",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("fixed").x,100.);assert_eq!(get("moving").x,30.);
}

#[test]
fn relative_layout_center_rules_use_full_parent_with_asymmetric_padding() {
    static ROOT:Node=Node {kind:Kind::Relative,padding_edges:[20.,10.,0.,0.],padding_explicit:[true;4],children:&[
        Node {width:Size::Dp(40.),height:Size::Dp(20.),relative_rules:&[RelativeRule{kind:20,anchor:""}],..button("center",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="center").unwrap().bounds,Rect{x:80.,y:40.,width:40.,height:20.});
}

#[test]
fn relative_layout_wrap_height_centers_in_minimum_parent_bounds() {
    static WRAP:Node=Node {kind:Kind::Relative,height:Size::Wrap,min_height:64.,padding_edges:[20.,10.,0.,0.],padding_explicit:[true;4],children:&[
        Node {width:Size::Dp(40.),height:Size::Dp(20.),relative_rules:&[RelativeRule{kind:20,anchor:""}],..button("center",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("wrap",Visibility::Visible,Size::Wrap,0.)};
    static ROOT:Node=Node {kind:Kind::Linear,children:&[WRAP,button("next",Visibility::Visible,Size::Dp(20.),0.)],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("center"),Rect{x:80.,y:22.,width:40.,height:20.});assert_eq!(get("next").y,64.);
}

#[test]
fn relative_wrap_measurement_honors_nested_layout_direction_override() {
    const WRAP:Node=Node {kind:Kind::Relative,height:Size::Wrap,children:&[
        Node {width:Size::Dp(120.),height:Size::Dp(24.),relative_rules:&[RelativeRule{kind:13,anchor:""}],..button("anchor",Visibility::Visible,Size::Dp(24.),0.)},
        Node {kind:Kind::Text,width:Size::Match,height:Size::Wrap,text:"A long line of text wraps based on the space beside the anchor.",relative_rules:&[RelativeRule{kind:9,anchor:"anchor"},RelativeRule{kind:17,anchor:""}],..button("words",Visibility::Visible,Size::Wrap,0.)},
    ],..button("relative",Visibility::Visible,Size::Wrap,0.)};
    static OVERRIDE:Node=Node {kind:Kind::Linear,children:&[Node {layout_direction:2,..WRAP},button("after",Visibility::Visible,Size::Dp(20.),0.)],..button("root",Visibility::Visible,Size::Match,0.)};
    static INHERIT:Node=Node {kind:Kind::Linear,children:&[WRAP,button("after",Visibility::Visible,Size::Dp(20.),0.)],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:2000.};
    let measure=|root:&'static Node,rtl| {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;ui.frame(Input::default(),bounds,0.,|f|{root.render(f,bounds,&mut state);});assert_eq!(state.rtl,rtl);ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y};
    let explicit=measure(&OVERRIDE,false);let inherited=measure(&INHERIT,true);let ltr=measure(&INHERIT,false);
    assert_eq!(explicit,inherited);assert_ne!(explicit,ltr);
}

#[test]
fn relative_layout_preserves_fixed_size_beyond_parent_edge() {
    static RELATIVE:Node=Node {kind:Kind::Relative,width:Size::Dp(100.),height:Size::Dp(50.),clip_children_disabled:true,clip_to_padding_disabled:true,children:&[
        Node {width:Size::Dp(100.),height:Size::Dp(20.),..button("anchor",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(40.),height:Size::Dp(20.),margin:[20.,0.,0.,0.],relative_rules:&[RelativeRule{kind:1,anchor:"anchor"}],..button("overflow",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("relative",Visibility::Visible,Size::Dp(50.),0.)};
    static ROOT:Node=Node {kind:Kind::Linear,clip_children_disabled:true,children:&[RELATIVE],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="overflow").unwrap().bounds,Rect{x:120.,y:0.,width:40.,height:20.});
}

#[test]
fn table_layout_shares_widest_columns_and_mirrors_rtl() {
    static TABLE:Node=Node {kind:Kind::Table,children:&[
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(40.),..button("a",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(80.),..button("b",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row1",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(60.),..button("c",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(30.),..button("d",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row2",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};
    for rtl in [false,true] {
        let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
        let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
        assert_eq!(get("a").width,60.);assert_eq!(get("b").width,80.);
        assert_eq!(get("a").x,if rtl {140.} else {0.});assert_eq!(get("b").x,if rtl {60.} else {60.});
        assert_eq!(get("c").x,get("a").x);assert_eq!(get("d").x,get("b").x);assert_eq!(get("c").y,20.);
    }
}

#[test]
fn table_layout_spans_columns_and_skips_explicit_gaps() {
    static TABLE:Node=Node {kind:Kind::Table,children:&[
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(40.),..button("first",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(80.),..button("second",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row1",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(10.),relative_rules:&[RelativeRule {kind:31,anchor:"2"}],..button("spanning",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row2",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(30.),relative_rules:&[RelativeRule {kind:30,anchor:"1"}],..button("skip",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row3",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:120.};
    for rtl in [false,true] {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
        let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
        assert_eq!(get("spanning").width,120.);assert_eq!(get("spanning").x,if rtl {80.} else {0.});
        assert_eq!(get("skip").x,get("second").x);assert_eq!(get("skip").width,80.);
    }
}

#[test]
fn table_layout_stretches_selected_and_shrinks_all_columns() {
    static TABLE:Node=Node {kind:Kind::Table,relative_rules:&[RelativeRule {kind:32,anchor:"1"},RelativeRule {kind:33,anchor:"*"}],children:&[
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(60.),..button("first",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(40.),..button("second",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    for (width,first,second) in [(200.,60.,140.),(80.,50.,30.)] {
        let bounds=Rect{x:0.,y:0.,width,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
        ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
        let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
        assert_eq!(get("first").width,first);assert_eq!(get("second").width,second);
    }
}

#[test]
fn table_layout_collapse_hides_cells_and_spans_and_can_restore() {
    static TABLE:Node=Node {kind:Kind::Table,relative_rules:&[RelativeRule {kind:34,anchor:"1"}],children:&[
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(60.),..button("first",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(40.),..button("second",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row1",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(100.),relative_rules:&[RelativeRule {kind:31,anchor:"2"}],..button("span",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row2",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="first"));
    assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="second" || n.label.as_ref()=="span"));
    assert!(TABLE.set_table_column_collapsed(&mut state,1,false));
    ui.frame(Input::default(),bounds,1.,|f|{TABLE.render(f,bounds,&mut state);});
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="second"));assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="span"));
}

#[test]
fn table_cells_respect_explicit_horizontal_gravity_in_both_directions() {
    static TABLE:Node=Node {kind:Kind::Table,children:&[
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(100.),..button("wide",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row1",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(40.),layout_gravity:[2,0],layout_gravity_explicit:true,relative_rules:&[RelativeRule {kind:35,anchor:""}],..button("end",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row2",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(40.),layout_gravity:[1,0],layout_gravity_explicit:true,relative_rules:&[RelativeRule {kind:35,anchor:""}],..button("center",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row3",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:120.};
    for rtl in [false,true] {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
        let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
        assert_eq!(get("end").width,40.);assert_eq!(get("end").x,if rtl {100.} else {60.});
        assert_eq!(get("center").x,if rtl {130.} else {30.});
    }
}

#[test]
fn table_cells_align_vertically_and_inherit_row_gravity() {
    static TABLE:Node=Node {kind:Kind::Table,children:&[
        Node {kind:Kind::TableRow,content_gravity:[0,1],children:&[
            Node {width:Size::Dp(40.),..button("tall",Visibility::Visible,Size::Dp(80.),0.)},
            Node {width:Size::Dp(40.),..button("center",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(40.),layout_gravity:[0,2],layout_gravity_explicit:true,..button("bottom",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("tall").y,0.);assert_eq!(get("center").y,30.);assert_eq!(get("bottom").y,60.);
}

#[test]
fn table_layout_clips_or_exposes_overflowing_columns() {
    const TABLE:Node=Node {kind:Kind::Table,children:&[
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(60.),..button("first",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(60.),..button("overflow",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    static CLIPPED:Node=TABLE;
    static UNCLIPPED:Node=Node {clip_children_disabled:true,clip_to_padding_disabled:true,..TABLE};
    let viewport=Rect{x:0.,y:0.,width:200.,height:100.};let bounds=Rect{x:0.,y:0.,width:100.,height:50.};
    for (table,visible_width) in [(&CLIPPED,40.),(&UNCLIPPED,60.)] {
        let mut ui=Ui::default();let mut state=LayoutState::default();
        ui.frame(Input::default(),viewport,0.,|f|{table.render(f,bounds,&mut state);});
        assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="overflow").unwrap().bounds.width,visible_width);
    }
}

#[test]
fn table_row_aligns_baselines_and_respects_opt_out() {
    const ROW:Node=Node {kind:Kind::TableRow,children:&[
        Node {kind:Kind::Text,text:"Small",text_size:16.,width:Size::Dp(100.),height:Size::Wrap,..button("small",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::Text,text:"Large",text_size:24.,width:Size::Dp(100.),height:Size::Wrap,..button("large",Visibility::Visible,Size::Wrap,0.)},
    ],..button("row",Visibility::Visible,Size::Wrap,0.)};
    static ON:Node=Node {kind:Kind::Table,children:&[ROW],..button("table",Visibility::Visible,Size::Match,0.)};
    static OFF:Node=Node {kind:Kind::Table,children:&[Node {relative_rules:&[RelativeRule {kind:36,anchor:""}],..ROW}],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:250.,height:100.};
    let delta=|table:&'static Node| {let mut ui=Ui::default();let mut state=LayoutState::default();let commands=ui.frame(Input::default(),bounds,0.,|f|{table.render(f,bounds,&mut state);});
        let baseline=|label:&str|commands.iter().find_map(|c|match c {DrawCommand::Text{text,bounds,style,..} if text.as_ref()==label=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
        (baseline("Small")-baseline("Large")).abs()
    };
    assert!(delta(&ON)<0.001);assert!(delta(&OFF)>1.0);
}

#[test]
fn table_row_direction_override_reverses_only_that_row() {
    static TABLE:Node=Node {kind:Kind::Table,children:&[
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(40.),..button("normal first",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(60.),..button("normal second",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row1",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::TableRow,layout_direction:2,children:&[
            Node {width:Size::Dp(40.),..button("rtl first",Visibility::Visible,Size::Dp(20.),0.)},
            Node {width:Size::Dp(60.),..button("rtl second",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row2",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert!(get("normal first").x<get("normal second").x);
    assert!(get("rtl first").x>get("rtl second").x);assert!(!state.rtl);
}

#[test]
fn table_row_visibility_alpha_and_foreground_apply_to_children() {
    static TABLE:Node=Node {kind:Kind::Table,children:&[
        Node {kind:Kind::TableRow,id:"row",view_alpha:0.5,foreground:Some(Color(3,4,5,200)),children:&[
            Node {kind:Kind::Text,text:"Child",color:Some(Color(10,20,30,200)),width:Size::Dp(70.),..button("child",Visibility::Visible,Size::Dp(20.),0.)},
        ],..button("row",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::TableRow,children:&[Node {width:Size::Dp(70.),..button("after",Visibility::Visible,Size::Dp(20.),0.)}],..button("after_row",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
    let push=commands.iter().position(|c|matches!(c,DrawCommand::PushLayer{alpha,..} if (*alpha-0.5).abs()<1e-6)).unwrap();
    let child_text=commands.iter().position(|c|matches!(c,DrawCommand::Text{text,color:Color(10,20,30,200),..} if text.as_ref()=="Child")).unwrap();
    let fg=commands.iter().position(|c|matches!(c,DrawCommand::RoundedRect{color:Color(3,4,5,200),..})).unwrap();
    let pop=commands.iter().position(|c|matches!(c,DrawCommand::PopLayer)).unwrap();
    assert!(push<child_text && child_text<fg && fg<pop);
    let before=ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y;
    state.visibility.insert("row".into(),Visibility::Invisible);
    let commands=ui.frame(Input::default(),bounds,1.,|f|{TABLE.render(f,bounds,&mut state);});
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="Child")));
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::PushLayer{..})));
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="after").unwrap().bounds.y,before);
}

#[test]
fn table_row_match_parent_cell_fills_tallest_row() {
    static TABLE:Node=Node {kind:Kind::Table,children:&[
        Node {kind:Kind::TableRow,children:&[
            Node {width:Size::Dp(50.),..button("tall",Visibility::Visible,Size::Dp(80.),0.)},
            Node {width:Size::Dp(50.),height:Size::Match,..button("fill",Visibility::Visible,Size::Match,0.)},
        ],..button("row",Visibility::Visible,Size::Wrap,0.)},
    ],..button("table",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{TABLE.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("fill").height,get("tall").height);assert_eq!(get("fill").height,80.);
}

#[test]
fn standalone_table_row_lays_out_cells_with_margins_and_rtl() {
    static ROW:Node=Node {kind:Kind::TableRow,children:&[
        Node {width:Size::Dp(40.),margin:[3.,0.,5.,0.],..button("first",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(60.),margin:[7.,0.,0.,0.],..button("second",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("row",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:60.};
    for rtl in [false,true] {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        ui.frame(Input::default(),bounds,0.,|f|{ROW.render(f,bounds,&mut state);});
        let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
        if rtl {assert_eq!(get("first").x,155.);assert_eq!(get("second").x,92.);} else {assert_eq!(get("first").x,3.);assert_eq!(get("second").x,55.);}
        assert_eq!(get("first").width,40.);assert_eq!(get("second").width,60.);
    }
}

#[test]
fn standalone_table_row_aligns_text_baselines_and_supports_opt_out() {
    const ROW:Node=Node {kind:Kind::TableRow,children:&[
        Node {kind:Kind::Text,text:"Small",text_size:16.,width:Size::Dp(100.),height:Size::Wrap,..button("small",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::Text,text:"Large",text_size:24.,width:Size::Dp(100.),height:Size::Wrap,..button("large",Visibility::Visible,Size::Wrap,0.)},
    ],..button("row",Visibility::Visible,Size::Match,0.)};
    static ON:Node=ROW;static OFF:Node=Node {relative_rules:&[RelativeRule {kind:36,anchor:""}],..ROW};
    let bounds=Rect{x:0.,y:0.,width:250.,height:60.};
    let delta=|row:&'static Node| {let mut ui=Ui::default();let mut state=LayoutState::default();let commands=ui.frame(Input::default(),bounds,0.,|f|{row.render(f,bounds,&mut state);});
        let baseline=|label:&str|commands.iter().find_map(|c|match c {DrawCommand::Text{text,bounds,style,..} if text.as_ref()==label=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
        (baseline("Small")-baseline("Large")).abs()
    };
    assert!(delta(&ON)<0.001);assert!(delta(&OFF)>1.0);
}

#[test]
fn grid_layout_automatic_tracks_and_rtl() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"2"}],children:&[
        Node {width:Size::Dp(50.),..button("a",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(60.),..button("b",Visibility::Visible,Size::Dp(30.),0.)},
        Node {width:Size::Dp(80.),..button("c",Visibility::Visible,Size::Dp(25.),0.)},
        Node {width:Size::Dp(40.),..button("d",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};
    for rtl in [false,true] {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
        let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
        assert_eq!(get("a").x,if rtl {150.} else {0.});assert_eq!(get("b").x,if rtl {60.} else {80.});
        assert_eq!(get("c").y,30.);assert_eq!(get("d").y,32.5);
    }
}

#[test]
fn grid_layout_combines_explicit_positions_and_automatic_placement() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"2"}],children:&[
        Node {width:Size::Dp(50.),..button("a",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(50.),relative_rules:&[RelativeRule {kind:42,anchor:"1"},RelativeRule {kind:43,anchor:"1"}],..button("placed",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(50.),..button("after",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("a").x,0.);assert_eq!(get("placed").x,50.);assert_eq!(get("placed").y,20.);assert_eq!(get("after").x,0.);assert_eq!(get("after").y,40.);
}

#[test]
fn grid_layout_spans_tracks_and_automatic_placement_skips_occupied_cells() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"2"}],children:&[
        Node {width:Size::Dp(50.),relative_rules:&[RelativeRule {kind:45,anchor:"2"}],..button("tall",Visibility::Visible,Size::Dp(60.),0.)},
        Node {width:Size::Dp(50.),..button("top",Visibility::Visible,Size::Dp(30.),0.)},
        Node {width:Size::Dp(50.),..button("under",Visibility::Visible,Size::Dp(30.),0.)},
        Node {width:Size::Dp(100.),relative_rules:&[RelativeRule {kind:44,anchor:"2"}],..button("wide",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:120.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("tall"),Rect{x:0.,y:0.,width:50.,height:60.});
    assert_eq!(get("top").x,50.);assert_eq!(get("under").x,50.);assert_eq!(get("under").y,30.);
    assert_eq!(get("wide"),Rect{x:0.,y:60.,width:100.,height:20.});
}

#[test]
fn grid_overlapping_spans_share_required_space() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"3"}],children:&[
        Node {width:Size::Dp(20.),..button("first",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(20.),..button("middle",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(20.),..button("last",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(100.),relative_rules:&[RelativeRule {kind:42,anchor:"0"},RelativeRule {kind:43,anchor:"1"},RelativeRule {kind:44,anchor:"2"}],..button("span left",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(100.),relative_rules:&[RelativeRule {kind:42,anchor:"1"},RelativeRule {kind:43,anchor:"2"},RelativeRule {kind:44,anchor:"2"}],..button("span right",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:120.,height:80.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert!((get("last").x-100.).abs()<0.01);assert!((get("span right").x-20.).abs()<0.01);
    assert!((get("span right").width-100.).abs()<0.01);
}

#[test]
fn grid_layout_cell_gravity_aligns_and_fills_tracks() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"2"}],children:&[
        Node {width:Size::Dp(100.),..button("tall",Visibility::Visible,Size::Dp(80.),0.)},
        Node {width:Size::Dp(40.),layout_gravity:[2,2],layout_gravity_explicit:true,..button("end bottom",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(100.),..button("other",Visibility::Visible,Size::Dp(80.),0.)},
        Node {width:Size::Dp(100.),..button("wide",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(100.),..button("third",Visibility::Visible,Size::Dp(80.),0.)},
        Node {width:Size::Dp(40.),layout_gravity:[5,5],layout_gravity_explicit:true,..button("fill",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:250.};
    for rtl in [false,true] {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
        let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
        assert_eq!(get("end bottom"),Rect{x:if rtl {0.} else {160.},y:60.,width:40.,height:20.});
        assert_eq!(get("fill"),Rect{x:if rtl {0.} else {100.},y:160.,width:100.,height:80.});
    }
}

#[test]
fn grid_layout_weighted_tracks_share_extra_width_and_height() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"2"}],children:&[
        Node {width:Size::Dp(0.),relative_rules:&[RelativeRule {kind:46,anchor:"1"},RelativeRule {kind:47,anchor:"1"}],..button("a",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(0.),relative_rules:&[RelativeRule {kind:46,anchor:"2"},RelativeRule {kind:47,anchor:"1"}],..button("b",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(0.),relative_rules:&[RelativeRule {kind:46,anchor:"1"},RelativeRule {kind:47,anchor:"2"}],..button("c",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(0.),relative_rules:&[RelativeRule {kind:46,anchor:"2"},RelativeRule {kind:47,anchor:"2"}],..button("d",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:300.,height:300.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert!((get("a").width-100.).abs()<0.01);assert!((get("b").width-200.).abs()<0.01);
    assert!((get("a").height-(20.+260./3.)).abs()<0.01);assert!((get("c").height-(20.+520./3.)).abs()<0.01);
}

#[test]
fn grid_weighted_children_respect_a_wider_unweighted_child_in_the_same_column() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"2"}],children:&[
        Node {width:Size::Dp(100.),relative_rules:&[RelativeRule {kind:42,anchor:"0"},RelativeRule {kind:43,anchor:"0"}],..button("wide",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(20.),relative_rules:&[RelativeRule {kind:42,anchor:"0"},RelativeRule {kind:43,anchor:"1"},RelativeRule {kind:46,anchor:"1"}],..button("weighted left",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(20.),relative_rules:&[RelativeRule {kind:42,anchor:"1"},RelativeRule {kind:43,anchor:"1"},RelativeRule {kind:46,anchor:"1"}],..button("weighted right",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:80.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert!((get("weighted left").width-100.).abs()<0.01);assert!((get("weighted right").width-100.).abs()<0.01);assert!((get("weighted right").x-100.).abs()<0.01);
}

#[test]
fn grid_weighted_children_respect_a_taller_unweighted_child_in_the_same_row() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,height:Size::Dp(200.),relative_rules:&[RelativeRule {kind:40,anchor:"2"}],children:&[
        Node {width:Size::Dp(50.),relative_rules:&[RelativeRule {kind:42,anchor:"0"},RelativeRule {kind:43,anchor:"0"},RelativeRule {kind:49,anchor:""}],..button("tall",Visibility::Visible,Size::Dp(100.),0.)},
        Node {width:Size::Dp(50.),relative_rules:&[RelativeRule {kind:42,anchor:"1"},RelativeRule {kind:43,anchor:"0"},RelativeRule {kind:47,anchor:"1"}],..button("weighted top",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(50.),relative_rules:&[RelativeRule {kind:42,anchor:"1"},RelativeRule {kind:43,anchor:"1"},RelativeRule {kind:47,anchor:"1"}],..button("weighted bottom",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:200.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert!((get("weighted top").height-100.).abs()<0.01);assert!((get("weighted bottom").height-100.).abs()<0.01);assert!((get("weighted bottom").y-100.).abs()<0.01);
}

#[test]
fn grid_layout_aligns_default_and_explicit_text_baselines() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"2"}],children:&[
        Node {kind:Kind::Text,text:"Small",text_size:16.,width:Size::Dp(80.),height:Size::Wrap,..button("small",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::Text,text:"Large",text_size:24.,width:Size::Dp(80.),height:Size::Wrap,layout_gravity:[0,6],layout_gravity_explicit:true,relative_rules:&[RelativeRule {kind:49,anchor:""}],..button("large",Visibility::Visible,Size::Wrap,0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let baseline=|label:&str|commands.iter().find_map(|c|match c {DrawCommand::Text{text,bounds,style,..} if text.as_ref()==label=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
    assert!((baseline("Small")-baseline("Large")).abs()<0.001);
}

#[test]
fn grid_default_margins_respect_explicit_zero_and_space() {
    static GRID:Node=Node {kind:Kind::Grid,horizontal:true,relative_rules:&[RelativeRule {kind:40,anchor:"3"},RelativeRule {kind:53,anchor:""}],children:&[
        Node {width:Size::Dp(50.),..button("default",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(50.),relative_rules:&[RelativeRule {kind:50,anchor:"1"}],..button("zero",Visibility::Visible,Size::Dp(20.),0.)},
        Node {kind:Kind::Space,width:Size::Dp(10.),height:Size::Dp(20.),..button("space",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("default").x,4.);assert_eq!(get("zero").x,58.);
    state.grid_default_gap=Some(24.);
    ui.frame(Input{time:0.1,..Input::default()},bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
    assert_eq!(get("default").x,12.);assert_eq!(get("zero").x,74.);
    state.grid_default_gap=Some(f32::NAN);
    ui.frame(Input{time:0.2,..Input::default()},bounds,0.,|f|{GRID.render(f,bounds,&mut state);});
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="default").unwrap().bounds.x,4.);
}

#[test]
fn grid_alignment_mode_shares_boundary_margins() {
    const GRID:Node=Node {kind:Kind::Grid,horizontal:true,children:&[
        Node {width:Size::Dp(50.),margin:[10.,0.,0.,0.],relative_rules:&[RelativeRule {kind:42,anchor:"0"},RelativeRule {kind:43,anchor:"0"}],..button("offset",Visibility::Visible,Size::Dp(20.),0.)},
        Node {width:Size::Dp(50.),relative_rules:&[RelativeRule {kind:42,anchor:"0"},RelativeRule {kind:43,anchor:"1"}],..button("plain",Visibility::Visible,Size::Dp(20.),0.)},
    ],..button("grid",Visibility::Visible,Size::Match,0.)};
    static MARGINS:Node=GRID;static BOUNDS:Node=Node {relative_rules:&[RelativeRule {kind:54,anchor:""}],..GRID};
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};
    for (grid,plain_x) in [(&MARGINS,0.),(&BOUNDS,10.)] {let mut ui=Ui::default();let mut state=LayoutState::default();
        ui.frame(Input::default(),bounds,0.,|f|{grid.render(f,bounds,&mut state);});
        let get=|name:&str|ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds;
        assert_eq!(get("offset").x,10.);assert_eq!(get("plain").x,plain_x);
    }
}

#[test]
fn editable_controls_align_entered_text_baselines_in_horizontal_layout() {
    const LABEL:Node=Node{kind:Kind::Text,text:"Large",text_size:32.,width:Size::Dp(100.),height:Size::Wrap,..button("large_label",Visibility::Visible,Size::Wrap,0.)};
    const EDIT:Node=Node{kind:Kind::Edit,text:"Edit",width:Size::Dp(120.),height:Size::Dp(48.),..button("baseline_edit",Visibility::Visible,Size::Dp(48.),0.)};
    const AUTO:Node=Node{kind:Kind::AutoComplete,text:"Auto",width:Size::Dp(120.),height:Size::Dp(48.),..button("baseline_auto",Visibility::Visible,Size::Dp(48.),0.)};
    const MULTI:Node=Node{kind:Kind::MultiAutoComplete,text:"Multi",width:Size::Dp(120.),height:Size::Dp(48.),..button("baseline_multi",Visibility::Visible,Size::Dp(48.),0.)};
    static EDIT_ROW:Node=Node{kind:Kind::Linear,horizontal:true,children:&[EDIT,LABEL],..button("edit_row",Visibility::Visible,Size::Match,0.)};
    static AUTO_ROW:Node=Node{kind:Kind::Linear,horizontal:true,children:&[AUTO,LABEL],..button("auto_row",Visibility::Visible,Size::Match,0.)};
    static MULTI_ROW:Node=Node{kind:Kind::Linear,horizontal:true,children:&[MULTI,LABEL],..button("multi_row",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:260.,height:100.};
    for rtl in [false,true] {for (row,label) in [(&EDIT_ROW,"Edit"),(&AUTO_ROW,"Auto"),(&MULTI_ROW,"Multi")] {
        let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f|{row.render(f,bounds,&mut state);});
        let base=|wanted:&str|commands.iter().find_map(|c|match c{DrawCommand::Text{text,bounds,style,..} if text.as_ref()==wanted=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
        assert!((base(label)-base("Large")).abs()<0.001,"{label} must align; rtl={rtl}");
    }}
}

#[test]
fn compound_controls_align_rendered_label_baselines_with_text() {
    const LABEL:Node=Node{kind:Kind::Text,text:"Large",text_size:32.,width:Size::Dp(100.),height:Size::Wrap,..button("large_label",Visibility::Visible,Size::Wrap,0.)};
    const CHECK:Node=Node{kind:Kind::CheckBox,text:"Check",text_size:16.,width:Size::Dp(100.),height:Size::Dp(48.),..button("baseline_check",Visibility::Visible,Size::Dp(48.),0.)};
    const RADIO:Node=Node{kind:Kind::RadioButton,text:"Radio",text_size:16.,width:Size::Dp(100.),height:Size::Dp(48.),..button("baseline_radio",Visibility::Visible,Size::Dp(48.),0.)};
    const SWITCH:Node=Node{kind:Kind::Switch,text:"Switch",text_size:16.,width:Size::Dp(100.),height:Size::Dp(48.),..button("baseline_switch",Visibility::Visible,Size::Dp(48.),0.)};
    static CHECK_ROW:Node=Node{kind:Kind::Linear,horizontal:true,children:&[CHECK,LABEL],..button("check_row",Visibility::Visible,Size::Match,0.)};
    static RADIO_ROW:Node=Node{kind:Kind::Linear,horizontal:true,children:&[RADIO,LABEL],..button("radio_row",Visibility::Visible,Size::Match,0.)};
    static SWITCH_ROW:Node=Node{kind:Kind::Linear,horizontal:true,children:&[SWITCH,LABEL],..button("switch_row",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:240.,height:100.};
    for rtl in [false,true] {for (row,label) in [(&CHECK_ROW,"Check"),(&RADIO_ROW,"Radio"),(&SWITCH_ROW,"Switch")] {
        let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f|{row.render(f,bounds,&mut state);});
        let base=|wanted:&str|commands.iter().find_map(|c|match c{DrawCommand::Text{text,bounds,style,..} if text.as_ref()==wanted=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
        assert!((base(label)-base("Large")).abs()<0.001,"{label} must align; rtl={rtl}");
    }}
}

#[test]
fn image_view_explicit_and_bottom_baselines_align_with_text() {
    static BASE:&[RelativeRule]=&[RelativeRule{kind:189,anchor:"20"}];
    static BOTTOM:&[RelativeRule]=&[RelativeRule{kind:189,anchor:"4"},RelativeRule{kind:190,anchor:""}];
    const IMAGE:Node=Node{kind:Kind::Image,id:"baseline_image",height:Size::Dp(40.),width:Size::Dp(40.),background:Some(Color(10,20,30,255)),..button("baseline_image",Visibility::Visible,Size::Dp(40.),0.)};
    const TEXT:Node=Node{kind:Kind::Text,text:"Baseline label",text_size:16.,width:Size::Dp(140.),height:Size::Wrap,..button("baseline_text",Visibility::Visible,Size::Wrap,0.)};
    static EXPLICIT:Node=Node{kind:Kind::Linear,horizontal:true,children:&[Node{relative_rules:BASE,..IMAGE},TEXT],..button("baseline_row",Visibility::Visible,Size::Match,0.)};
    static AT_BOTTOM:Node=Node{kind:Kind::Linear,horizontal:true,children:&[Node{relative_rules:BOTTOM,..IMAGE},TEXT],..button("baseline_row",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:240.,height:100.};
    for rtl in [false,true] {for (view,offset) in [(&EXPLICIT,20.),(&AT_BOTTOM,40.)] {
        let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f|{view.render(f,bounds,&mut state);});
        let image_y=commands.iter().find_map(|c|match c{DrawCommand::RoundedRect{bounds,color:Color(10,20,30,255),..}=>Some(bounds.y),_=>None}).unwrap();
        let text_baseline=commands.iter().find_map(|c|match c{DrawCommand::Text{text,bounds,style,..} if text.as_ref()=="Baseline label"=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
        assert!((image_y+offset-text_baseline).abs()<0.001,"image and text baseline must align; rtl={rtl}");
    }}
}

#[test]
fn horizontal_linear_layout_aligns_baselines_and_supports_xml_opt_out() {
    const CHILDREN:&[Node]=&[
        Node {kind:Kind::Text,text:"Small",text_size:16.,width:Size::Dp(100.),height:Size::Wrap,..button("small",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::Text,text:"Large",text_size:32.,width:Size::Dp(100.),height:Size::Wrap,..button("large",Visibility::Visible,Size::Wrap,0.)},
    ];
    const ROW:Node=Node {kind:Kind::Linear,horizontal:true,children:CHILDREN,..button("row",Visibility::Visible,Size::Match,0.)};
    static ON:Node=ROW;
    static OFF:Node=Node {relative_rules:&[RelativeRule {kind:36,anchor:""}],..ROW};
    let bounds=Rect{x:0.,y:0.,width:250.,height:80.};
    for rtl in [false,true] {
        let delta=|node:&'static Node| {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
            let commands=ui.frame(Input::default(),bounds,0.,|f|{node.render(f,bounds,&mut state);});
            let baseline=|label:&str|commands.iter().find_map(|c|match c {DrawCommand::Text{text,bounds,style,..} if text.as_ref()==label=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
            (baseline("Small")-baseline("Large")).abs()
        };
        assert!(delta(&ON)<0.001,"default aligned in RTL={rtl}");
        assert!(delta(&OFF)>1.0,"opt-out preserves top placement in RTL={rtl}");
    }
}

#[test]
fn horizontal_wrap_content_reserves_space_for_combined_baselines() {
    static PARENT:Node=Node {kind:Kind::Linear,horizontal:false,children:&[
        Node {kind:Kind::Linear,horizontal:true,height:Size::Wrap,children:&[
            Node {kind:Kind::Text,text:"Padded",text_size:16.,width:Size::Dp(100.),height:Size::Wrap,padding_edges:[0.,0.,0.,20.],padding_explicit:[true,true,true,true],..button("small",Visibility::Visible,Size::Wrap,0.)},
            Node {kind:Kind::Text,text:"Large",text_size:32.,width:Size::Dp(100.),height:Size::Wrap,..button("large",Visibility::Visible,Size::Wrap,0.)},
        ],..button("row",Visibility::Visible,Size::Wrap,0.)},
        Node {kind:Kind::Button,text:"After row",width:Size::Dp(100.),height:Size::Dp(40.),..button("after",Visibility::Visible,Size::Dp(40.),0.)},
    ],..button("parent",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:250.,height:200.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{PARENT.render(f,bounds,&mut state);});
    let after=ui.semantics().iter().find(|n|n.label.as_ref()=="After row").unwrap().bounds.y;
    let baseline=|label:&str|commands.iter().find_map(|c|match c {DrawCommand::Text{text,bounds,style,..} if text.as_ref()==label=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
    assert!((baseline("Padded")-baseline("Large")).abs()<0.001);
    assert!(after>55.0,"row must reserve the extra descent, got {after}");
}

#[test]
fn linear_layout_exposes_selected_child_baseline_to_its_parent() {
    const INNER:Node=Node {kind:Kind::Linear,horizontal:false,width:Size::Dp(120.),height:Size::Dp(64.),children:&[
        Node {kind:Kind::Text,text:"Caption",text_size:12.,width:Size::Dp(100.),height:Size::Dp(24.),..button("caption",Visibility::Visible,Size::Dp(24.),0.)},
        Node {kind:Kind::Text,text:"Nested",text_size:24.,width:Size::Dp(100.),height:Size::Dp(36.),..button("nested",Visibility::Visible,Size::Dp(36.),0.)},
    ],..button("inner",Visibility::Visible,Size::Dp(64.),0.)};
    const PEER:Node=Node {kind:Kind::Text,text:"Peer",text_size:32.,width:Size::Dp(100.),height:Size::Wrap,..button("peer",Visibility::Visible,Size::Wrap,0.)};
    static ON:Node=Node {kind:Kind::Linear,horizontal:true,children:&[Node {relative_rules:&[RelativeRule{kind:148,anchor:"1"}],..INNER},PEER],..button("outer",Visibility::Visible,Size::Match,0.)};
    static OFF:Node=Node {kind:Kind::Linear,horizontal:true,children:&[INNER,PEER],..button("outer",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:250.,height:90.};
    for rtl in [false,true] {
        let difference=|outer:&'static Node| {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
            let commands=ui.frame(Input::default(),bounds,0.,|f|{outer.render(f,bounds,&mut state);});
            let baseline=|label:&str|commands.iter().find_map(|c|match c {DrawCommand::Text{text,bounds,style,..} if text.as_ref()==label=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
            (baseline("Nested")-baseline("Peer")).abs()
        };
        assert!(difference(&ON)<0.001,"nested child baseline should align, rtl={rtl}");
        assert!(difference(&OFF)>8.0,"without child baseline index, rows should differ");
    }
}

#[test]
fn relative_layout_can_align_to_a_nested_linear_child_baseline() {
    static ROOT:Node=Node {kind:Kind::Relative,children:&[
        Node {kind:Kind::Linear,id:"container",horizontal:false,width:Size::Dp(120.),height:Size::Dp(64.),relative_rules:&[RelativeRule{kind:148,anchor:"1"}],children:&[
            Node {kind:Kind::Text,text:"Caption",text_size:12.,width:Size::Dp(100.),height:Size::Dp(24.),..button("caption",Visibility::Visible,Size::Dp(24.),0.)},
            Node {kind:Kind::Text,text:"Nested",text_size:24.,width:Size::Dp(100.),height:Size::Dp(36.),..button("nested",Visibility::Visible,Size::Dp(36.),0.)},
        ],..button("container",Visibility::Visible,Size::Dp(64.),0.)},
        Node {kind:Kind::Text,text:"Peer",text_size:24.,width:Size::Dp(100.),height:Size::Dp(36.),relative_rules:&[RelativeRule{kind:22,anchor:"container"},RelativeRule{kind:1,anchor:"container"}],..button("peer",Visibility::Visible,Size::Dp(36.),0.)},
    ],..button("root",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:250.,height:110.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{ROOT.render(f,bounds,&mut state);});
    let base=|label:&str|commands.iter().find_map(|c|match c {DrawCommand::Text{text,bounds,style,..} if text.as_ref()==label=>Some(bounds.y+bounds.height*0.5+style.size*0.3),_=>None}).unwrap();
    assert!((base("Nested")-base("Peer")).abs()<0.001);
}

#[test]
fn button_label_defaults_to_center_and_explicit_start_follows_rtl() {
    static CENTER:Node=button("button_center",Visibility::Visible,Size::Dp(48.),0.);
    static EXPLICIT:Node=Node {relative_rules:&[RelativeRule{kind:168,anchor:""}],content_gravity:[0,0],..button("button_start",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:120.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    for rtl in [false,true] {
        state.rtl=rtl;
        let commands=ui.frame(Input::default(),bounds,0.,|f|{CENTER.render(f,bounds,&mut state);});
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,style,..} if text.as_ref()=="button_center" && style.align==1)));
        let commands=ui.frame(Input::default(),bounds,0.,|f|{EXPLICIT.render(f,bounds,&mut state);});
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,style,..} if text.as_ref()=="button_start" && style.align==if rtl {2}else{0})));
    }
}

#[test]
fn horizontal_scroll_direction_change_stops_old_direction_fling() {
    static CONTENT:Node=Node{kind:Kind::Linear,horizontal:true,width:Size::Dp(500.),height:Size::Dp(40.),..button("content",Visibility::Visible,Size::Dp(40.),0.)};
    static SCROLL:Node=Node{kind:Kind::HorizontalScroll,id:"scroll_direction",children:&[CONTENT],..button("scroll_direction",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:100.,height:50.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let draw=|ui:&mut Ui,state:&mut LayoutState,input:Input|ui.frame(input,bounds,0.,|f|{SCROLL.render(f,bounds,state);});
    draw(&mut ui,&mut state,Input{pointer:Some([90.,20.]),pointer_kind:PointerKind::Touch,pressed:true,time:1.,..Input::default()});
    draw(&mut ui,&mut state,Input{pointer:Some([40.,20.]),pointer_kind:PointerKind::Touch,time:1.05,..Input::default()});
    draw(&mut ui,&mut state,Input{pointer:Some([40.,20.]),pointer_kind:PointerKind::Touch,released:true,time:1.06,..Input::default()});
    let before=state.offsets["scroll_direction"];
    state.rtl=true;draw(&mut ui,&mut state,Input{time:1.08,..Input::default()});
    assert!((state.offsets["scroll_direction"]-before).abs()<0.01);
    draw(&mut ui,&mut state,Input{time:1.5,..Input::default()});
    assert!((state.offsets["scroll_direction"]-before).abs()<0.01);
}

#[test]
fn horizontal_scroll_starts_at_logical_start_and_swipes_in_both_directions() {
    static CONTENT:Node=Node {kind:Kind::Linear,horizontal:true,width:Size::Dp(240.),height:Size::Dp(40.),children:&[
        Node {width:Size::Dp(80.),..button("First",Visibility::Visible,Size::Dp(40.),0.)},
        Node {width:Size::Dp(80.),..button("Middle",Visibility::Visible,Size::Dp(40.),0.)},
        Node {width:Size::Dp(80.),..button("Last",Visibility::Visible,Size::Dp(40.),0.)},
    ],..button("content",Visibility::Visible,Size::Dp(40.),0.)};
    static SCROLL:Node=Node {kind:Kind::HorizontalScroll,id:"scroll",children:&[CONTENT],..button("scroll",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:100.,height:50.};
    for rtl in [false,true] {
        let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        let draw=|ui:&mut Ui,state:&mut LayoutState,input:Input|ui.frame(input,bounds,0.,|f|{SCROLL.render(f,bounds,state);});
        draw(&mut ui,&mut state,Input::default());
        let visible=|ui:&Ui,label:&str|ui.semantics().iter().any(|n|n.label.as_ref()==label && n.bounds.width>0.0);
        assert!(visible(&ui,"First"),"first item must start visible, rtl={rtl}");
        assert!(!visible(&ui,"Last"),"last item starts outside viewport, rtl={rtl}");
        let from=if rtl{20.}else{80.};let to=if rtl{80.}else{20.};
        draw(&mut ui,&mut state,Input{pointer:Some([from,20.]),pointer_kind:PointerKind::Touch,pressed:true,time:1.,..Input::default()});
        draw(&mut ui,&mut state,Input{pointer:Some([to,20.]),pointer_kind:PointerKind::Touch,time:1.1,..Input::default()});
        assert!(state.offsets["scroll"]>40.,"swipe should move away from the start, rtl={rtl}");
        draw(&mut ui,&mut state,Input{pointer:Some([to,20.]),pointer_kind:PointerKind::Touch,released:true,time:1.2,..Input::default()});
        draw(&mut ui,&mut state,Input{pointer:Some([50.,20.]),scroll:[if rtl {-1000.}else{1000.},0.],time:2.,..Input::default()});
        assert_eq!(state.offsets["scroll"],140.);
        assert!(visible(&ui,"Last"),"last item must be visible at end, rtl={rtl}");
    }
}

#[test]
fn horizontal_scroll_places_short_content_at_rtl_start() {
    static CHILD:Node=Node {width:Size::Dp(40.),height:Size::Dp(40.),..button("Short",Visibility::Visible,Size::Dp(40.),0.)};
    static SCROLL:Node=Node {kind:Kind::HorizontalScroll,id:"short_scroll",children:&[CHILD],..button("scroll",Visibility::Visible,Size::Match,0.)};
    let bounds=Rect{x:0.,y:0.,width:100.,height:50.};let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=true;
    ui.frame(Input::default(),bounds,0.,|f|{SCROLL.render(f,bounds,&mut state);});
    let child=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap();
    assert_eq!(child.bounds.x,60.);assert_eq!(child.bounds.width,40.);
}
#[test]
fn wrap_content_width_uses_longest_line_for_all_mandatory_separators(){
    static TEXT:Node=Node{kind:Kind::Text,width:Size::Wrap,clickable:true,..button("label",Visibility::Visible,Size::Dp(80.),0.)};
    static HORIZONTAL:Node=Node{horizontal:true,measure_all_children:false,children:&[TEXT],..ROOT};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());let bounds=Rect{x:0.,y:0.,width:200.,height:100.};
    let expected=ui.text_metrics.width("WWW",TextStyle{size:16.,..TextStyle::default()});
    for separator in ["\n","\r","\r\n","\u{b}","\u{c}","\u{85}","\u{2028}","\u{2029}"] {
        state.text.insert("label".into(),format!("WWW{separator}i"));
        let commands=ui.frame(Input::default(),bounds,0.,|f|{HORIZONTAL.render(f,bounds,&mut state);});
        assert!((ui.semantics()[0].bounds.width-expected).abs()<0.01,"separator {separator:?}");
        let lines:Vec<_>=commands.iter().filter_map(|c|if let DrawCommand::Text{text,..}=c{Some(text.as_ref())}else{None}).collect();
        assert_eq!(lines,["WWW","i"],"separator {separator:?}");
    }
}

#[test]
fn switch_xml_layout_dimensions_apply_in_both_directions() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:193,anchor:"120"},RelativeRule{kind:194,anchor:"20"}];
    static SWITCH:Node=Node{kind:Kind::Switch,text:"Wide",relative_rules:RULES,..button("wide",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:300.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds:rect,..} if rect.x==180. && rect.width==120. && rect.height==32.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="Wide" && bounds.width==160.)));
    state.rtl=true;
    let commands=ui.frame(Input::default(),bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds:rect,..} if rect.x==0. && rect.width==120. && rect.height==32.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="Wide" && bounds.x==140.)));
}

#[test]
fn switch_thumb_text_padding_expands_the_track() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:161,anchor:""},RelativeRule{kind:195,anchor:"20"}];
    static SWITCH:Node=Node{kind:Kind::Switch,text:"Power",text_on:"YES",text_off:"NO",relative_rules:RULES,..button("padded",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:300.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{SWITCH.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds:rect,..} if rect.width>84. && rect.height==32.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="NO" && bounds.width>=40.)));
}

#[test]
fn wrap_switch_in_vertical_layout_reserves_configured_track() {
    #[path = "../../../../../apps/bhoo-web/src/android_switch_layout.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:156.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let wrap=ui.semantics().iter().find(|n|n.label.as_ref()=="Wrap").unwrap().bounds;
    assert!(wrap.width>=108.0 && wrap.width<200.0,"wrap switch width: {}",wrap.width);
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,..} if bounds.y>=104. && bounds.width==96. && bounds.height==32.)));
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=ui.semantics().iter().find(|n|n.label.as_ref()=="Wrap").unwrap().bounds;
    assert!((rtl.x+rtl.width-300.).abs()<0.01);
}

#[test]
fn switch_runtime_label_remeasures_and_preserves_checked_state() {
    #[path = "../../../../../apps/bhoo-web/src/android_switch_layout.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:156.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let before=ui.semantics().iter().find(|n|n.label.as_ref()=="Wrap").unwrap().bounds.width;
    state.checked.insert("wrap_switch".into(),true);
    state.text.insert("wrap_switch".into(),"Longer switch label".into());
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="Longer switch label")));
    let changed=ui.semantics().iter().find(|n|n.label.as_ref()=="Longer switch label").unwrap();
    assert!(changed.bounds.width>before);assert!(state.checked["wrap_switch"]);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=ui.semantics().iter().find(|n|n.label.as_ref()=="Longer switch label").unwrap().bounds;
    assert!((rtl.x+rtl.width-300.).abs()<0.01);
    state.text.remove("wrap_switch");ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!((ui.semantics().iter().find(|n|n.label.as_ref()=="Wrap").unwrap().bounds.width-before).abs()<0.01);
}

#[test]
fn checkbox_runtime_label_remeasures_without_resetting_state() {
    #[path = "../../../../../apps/bhoo-web/src/android_checkbox_label.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let before=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds.width;
    state.checked.insert("checkbox_label".into(),true);state.text.insert("checkbox_label".into(),"Longer checkbox label".into());
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="Longer checkbox label")));
    let changed=ui.semantics().iter().find(|n|n.label.as_ref()=="Longer checkbox label").unwrap();assert!(changed.bounds.width>before);assert!(state.checked["checkbox_label"]);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=ui.semantics().iter().find(|n|n.label.as_ref()=="Longer checkbox label").unwrap().bounds;assert!((rtl.x+rtl.width-300.).abs()<0.01);
    state.text.remove("checkbox_label");ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!((ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds.width-before).abs()<0.01);
}

#[test]
fn radio_runtime_label_remeasures_and_preserves_group_selection() {
    #[path = "../../../../../apps/bhoo-web/src/android_radio_label.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:96.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let before=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds.width;assert!(state.checked["radio_label"]);
    state.text.insert("radio_label".into(),"Longer radio label".into());
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="Longer radio label")));
    assert!(ui.semantics().iter().find(|n|n.label.as_ref()=="Longer radio label").unwrap().bounds.width>before);assert!(state.checked["radio_label"]);assert!(!state.checked["radio_other"]);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=ui.semantics().iter().find(|n|n.label.as_ref()=="Longer radio label").unwrap().bounds;assert!((rtl.x+rtl.width-300.).abs()<0.01);
    state.text.remove("radio_label");ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!((ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds.width-before).abs()<0.01);assert!(state.checked["radio_label"]);
}

#[test]
fn toggle_wrap_width_follows_current_state_label() {
    #[path = "../../../../../apps/bhoo-web/src/android_togglebutton_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let before=ui.semantics().iter().find(|n|n.label.as_ref()=="Adaptive toggle").unwrap().bounds;
    assert!(before.width>=48. && before.width<100.);
    state.checked.insert("wrap_toggle".into(),true);
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let after=ui.semantics().iter().find(|n|n.label.as_ref()=="Adaptive toggle").unwrap().bounds;
    assert!(after.width>before.width+60. && after.width<=300.);
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="Enabled for notifications" && bounds.width>100.)));
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=ui.semantics().iter().find(|n|n.label.as_ref()=="Adaptive toggle").unwrap().bounds;assert!((rtl.x+rtl.width-300.).abs()<0.01);
    state.checked.insert("wrap_toggle".into(),false);ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!((ui.semantics().iter().find(|n|n.label.as_ref()=="Adaptive toggle").unwrap().bounds.width-before.width).abs()<0.01);
}

#[test]
fn button_wrap_width_tracks_runtime_label_and_preserves_activation() {
    #[path = "../../../../../apps/bhoo-web/src/android_button_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:104.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let before=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds;
    state.text.insert("wrap_button".into(),"A much longer button".into());
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let after=ui.semantics().iter().find(|n|n.label.as_ref()=="A much longer button").unwrap().bounds;
    assert!(after.width>before.width+60. && after.width<=300.);
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,bounds,..} if text.as_ref()=="A much longer button" && bounds.width>100.)));
    let mut action=None;
    for input in [Input{pointer:Some([20.,20.]),pressed:true,..Input::default()},Input{pointer:Some([20.,20.]),released:true,..Input::default()}] {ui.frame(input,bounds,0.,|f|{action=example::LAYOUT.render(f,bounds,&mut state);});}
    assert_eq!(action,Some("wrap_button"));
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=ui.semantics().iter().find(|n|n.label.as_ref()=="A much longer button").unwrap().bounds;assert!((rtl.x+rtl.width-300.).abs()<0.01);
    state.text.remove("wrap_button");ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!((ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds.width-before.width).abs()<0.01);
}

#[test]
fn relative_layout_remeasures_wrap_button_and_repositions_sibling() {
    #[path = "../../../../../apps/bhoo-web/src/android_relative_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let get=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    let short=get(&ui,"Short");let next=get(&ui,"Next");assert!((next.x-short.x-short.width-8.).abs()<0.01);
    state.text.insert("relative_wrap".into(),"Longer label".into());
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let long=get(&ui,"Longer label");let moved=get(&ui,"Next");assert!(long.width>short.width+30.);assert!((moved.x-long.x-long.width-8.).abs()<0.01);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=get(&ui,"Longer label");let next_rtl=get(&ui,"Next");assert!((rtl.x+rtl.width-300.).abs()<0.01);assert!((rtl.x-next_rtl.x-next_rtl.width-8.).abs()<0.01);
}

#[test]
fn grid_layout_remeasures_wrap_child_and_moves_next_column() {
    #[path = "../../../../../apps/bhoo-web/src/android_grid_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let get=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    let short=get(&ui,"Short");let next=get(&ui,"Next");assert!((next.x-short.x-short.width-8.).abs()<0.01);
    state.text.insert("grid_wrap".into(),"Longer label".into());
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let long=get(&ui,"Longer label");let moved=get(&ui,"Next");assert!(long.width>short.width+30.);assert!((moved.x-long.x-long.width-8.).abs()<0.01);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=get(&ui,"Longer label");let next_rtl=get(&ui,"Next");assert!((rtl.x+rtl.width-300.).abs()<0.01);assert!((rtl.x-next_rtl.x-next_rtl.width-8.).abs()<0.01);
}

#[test]
fn table_layout_remeasures_shared_column_after_child_label_change() {
    #[path = "../../../../../apps/bhoo-web/src/android_table_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:96.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let get=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    let short=get(&ui,"Short");let next=get(&ui,"Next");let two=get(&ui,"Two");assert_eq!(next.x,two.x);assert!((next.x-short.x-short.width).abs()<0.01);
    state.text.insert("table_wrap".into(),"Longer label".into());
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let long=get(&ui,"Longer label");let moved=get(&ui,"Next");assert!(long.width>short.width+30.);assert_eq!(moved.x,get(&ui,"Two").x);assert!((moved.x-long.x-long.width).abs()<0.01);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=get(&ui,"Longer label");let next_rtl=get(&ui,"Next");assert!((rtl.x+rtl.width-300.).abs()<0.01);assert_eq!(next_rtl.x,get(&ui,"Two").x);assert!((rtl.x-next_rtl.x-next_rtl.width).abs()<0.01);
}

#[test]
fn nested_linear_wrap_tracks_child_and_shifts_following_siblings() {
    #[path = "../../../../../apps/bhoo-web/src/android_linear_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let get=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    let short=get(&ui,"Short");let next=get(&ui,"Next");let after=get(&ui,"After");assert!((next.x-short.x-short.width).abs()<0.01);assert!((after.x-next.x-next.width).abs()<0.01);
    state.text.insert("nested_wrap".into(),"Longer label".into());
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let long=get(&ui,"Longer label");let moved=get(&ui,"Next");let following=get(&ui,"After");assert!(long.width>short.width+30.);assert!((moved.x-long.x-long.width).abs()<0.01);assert!((following.x-moved.x-moved.width).abs()<0.01);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=get(&ui,"Longer label");let next_rtl=get(&ui,"Next");let after_rtl=get(&ui,"After");assert!((rtl.x+rtl.width-300.).abs()<0.01);assert!((rtl.x-next_rtl.x-next_rtl.width).abs()<0.01);assert!((next_rtl.x-after_rtl.x-after_rtl.width).abs()<0.01);
}

#[test]
fn nested_frame_wrap_tracks_child_and_shifts_following_sibling() {
    #[path = "../../../../../apps/bhoo-web/src/android_frame_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let get=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    let short=get(&ui,"Short");let after=get(&ui,"After");assert!((after.x-short.x-short.width).abs()<0.01);
    state.text.insert("frame_wrap".into(),"Longer label".into());
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let long=get(&ui,"Longer label");let moved=get(&ui,"After");assert!(long.width>short.width+30.);assert!((moved.x-long.x-long.width).abs()<0.01);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=get(&ui,"Longer label");let after_rtl=get(&ui,"After");assert!((rtl.x+rtl.width-300.).abs()<0.01);assert!((rtl.x-after_rtl.x-after_rtl.width).abs()<0.01);
}

#[test]
fn nested_relative_wrap_tracks_child_and_following_sibling() {
    #[path = "../../../../../apps/bhoo-web/src/android_relative_nested_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let get=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    let small=get(&ui,"Small");let more=get(&ui,"More");let after=get(&ui,"After");assert!((more.x-small.x-small.width).abs()<0.01);assert!((after.x-more.x-more.width).abs()<0.01);
    state.text.insert("nested_relative_label".into(),"Longer nested".into());
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let long=get(&ui,"Longer nested");let moved=get(&ui,"More");let following=get(&ui,"After");assert!(long.width>small.width+30.);assert!((moved.x-long.x-long.width).abs()<0.01);assert!((following.x-moved.x-moved.width).abs()<0.01);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=get(&ui,"Longer nested");let more_rtl=get(&ui,"More");let after_rtl=get(&ui,"After");assert!((rtl.x+rtl.width-300.).abs()<0.01);assert!((rtl.x-more_rtl.x-more_rtl.width).abs()<0.01);assert!((more_rtl.x-after_rtl.x-after_rtl.width).abs()<0.01);
    state.rtl=false;ui.text_metrics.normal=[0.7;95];ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let wider=get(&ui,"Longer nested");let more=get(&ui,"More");let after=get(&ui,"After");assert!(wider.width>long.width);assert!((more.x-wider.x-wider.width).abs()<0.01);assert!((after.x-more.x-more.width).abs()<0.01);
}

#[test]
fn nested_grid_wrap_tracks_child_and_following_sibling_with_active_metrics() {
    #[path = "../../../../../apps/bhoo-web/src/android_grid_nested_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let get=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    let small=get(&ui,"Small");let more=get(&ui,"More");let after=get(&ui,"After");assert!((more.x-small.x-small.width).abs()<0.01);assert!((after.x-more.x-more.width).abs()<0.01);
    state.text.insert("nested_grid_label".into(),"Longer nested".into());
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let long=get(&ui,"Longer nested");let moved=get(&ui,"More");let following=get(&ui,"After");assert!(long.width>small.width+30.);assert!((moved.x-long.x-long.width).abs()<0.01);assert!((following.x-moved.x-moved.width).abs()<0.01);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=get(&ui,"Longer nested");let more_rtl=get(&ui,"More");let after_rtl=get(&ui,"After");assert!((rtl.x+rtl.width-300.).abs()<0.01);assert!((rtl.x-more_rtl.x-more_rtl.width).abs()<0.01);assert!((more_rtl.x-after_rtl.x-after_rtl.width).abs()<0.01);
    state.rtl=false;ui.text_metrics.normal=[0.7;95];ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let wider=get(&ui,"Longer nested");let more=get(&ui,"More");let after=get(&ui,"After");assert!(wider.width>long.width);assert!((more.x-wider.x-wider.width).abs()<0.01);assert!((after.x-more.x-more.width).abs()<0.01);
}

#[test]
fn nested_table_wrap_tracks_child_and_following_sibling_with_active_metrics() {
    #[path = "../../../../../apps/bhoo-web/src/android_table_nested_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let get=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    let small=get(&ui,"Small");let more=get(&ui,"More");let after=get(&ui,"After");assert!((more.x-small.x-small.width).abs()<0.01);assert!((after.x-more.x-more.width).abs()<0.01);
    state.text.insert("nested_table_label".into(),"Longer nested".into());
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let long=get(&ui,"Longer nested");let moved=get(&ui,"More");let following=get(&ui,"After");assert!(long.width>small.width+30.);assert!((moved.x-long.x-long.width).abs()<0.01);assert!((following.x-moved.x-moved.width).abs()<0.01);
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let rtl=get(&ui,"Longer nested");let more_rtl=get(&ui,"More");let after_rtl=get(&ui,"After");assert!((rtl.x+rtl.width-300.).abs()<0.01);assert!((rtl.x-more_rtl.x-more_rtl.width).abs()<0.01);assert!((more_rtl.x-after_rtl.x-after_rtl.width).abs()<0.01);
    state.rtl=false;ui.text_metrics.normal=[0.7;95];ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let wider=get(&ui,"Longer nested");let more=get(&ui,"More");let after=get(&ui,"After");assert!(wider.width>long.width);assert!((more.x-wider.x-wider.width).abs()<0.01);assert!((after.x-more.x-more.width).abs()<0.01);
}

#[test]
fn button_compound_drawables_measure_mirror_and_respond_to_press() {
    #[path = "../../../../../apps/bhoo-web/src/android_button_drawables.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:300.,height:104.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let blue=commands.iter().find_map(|c|if let DrawCommand::Ellipse{bounds,color:Color(23,105,170,255),..}=c {Some(*bounds)}else{None}).unwrap();
    let amber=commands.iter().find_map(|c|if let DrawCommand::RoundedRect{bounds,color:Color(227,155,22,255),..}=c {Some(*bounds)}else{None}).unwrap();
    let text=commands.iter().find_map(|c|if let DrawCommand::Text{text,bounds,..}=c {if text.as_ref()=="Send"{Some(*bounds)}else{None}}else{None}).unwrap();
    assert!(blue.x+blue.width<=text.x && text.x+text.width<=amber.x);
    assert!(ui.semantics().iter().find(|n|n.label.as_ref()=="Send").unwrap().bounds.width>100.);
    let pressed=ui.frame(Input{pointer:Some([30.,70.]),pressed:true,..Input::default()},bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!(!pressed.iter().any(|c|matches!(c,DrawCommand::Ellipse{bounds,color:Color(23,105,170,255),..} if bounds.y>=52.)));
    ui.frame(Input{pointer:Some([30.,70.]),released:true,..Input::default()},bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    state.rtl=true;let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let blue=commands.iter().find_map(|c|if let DrawCommand::Ellipse{bounds,color:Color(23,105,170,255),..}=c {Some(*bounds)}else{None}).unwrap();
    let amber=commands.iter().find_map(|c|if let DrawCommand::RoundedRect{bounds,color:Color(227,155,22,255),..}=c {Some(*bounds)}else{None}).unwrap();
    assert!(blue.x>amber.x);
}

#[test]
fn button_compound_drawable_tint_changes_with_press() {
    static RULES:&[RelativeRule]=&[RelativeRule{kind:173,anchor:"16,16,0,100,100,100,255"},RelativeRule{kind:176,anchor:"2,0,200,0,0,255"},RelativeRule{kind:176,anchor:"0,0,0,0,200,255"}];
    static VIEW:Node=Node{kind:Kind::Button,text:"Tint",relative_rules:RULES,..button("tint_button",Visibility::Visible,Size::Dp(48.),0.)};
    let bounds=Rect{x:0.,y:0.,width:200.,height:48.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let normal=ui.frame(Input::default(),bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(normal.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(0,0,200,255),..} if bounds.width==16.)));
    let pressed=ui.frame(Input{pointer:Some([50.,20.]),pressed:true,..Input::default()},bounds,0.,|f|{VIEW.render(f,bounds,&mut state);});
    assert!(pressed.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(200,0,0,255),..} if bounds.width==16.)));
}

#[test]
fn button_wrap_height_reserves_vertical_compound_drawables() {
    #[path = "../../../../../apps/bhoo-web/src/android_button_drawable_height.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:280.,height:112.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let green=commands.iter().find_map(|c|if let DrawCommand::RoundedRect{bounds,color:Color(41,115,79,255),..}=c{Some(*bounds)}else{None}).unwrap();
    let red=commands.iter().find_map(|c|if let DrawCommand::RoundedRect{bounds,color:Color(191,56,72,255),..}=c{Some(*bounds)}else{None}).unwrap();
    let text=commands.iter().find_map(|c|if let DrawCommand::Text{text,bounds,..}=c{if text.as_ref()=="Vertical"{Some(*bounds)}else{None}}else{None}).unwrap();
    assert!(green.y+green.height<=text.y && text.y+text.height<=red.y);
    let button=ui.semantics().iter().find(|n|n.label.as_ref()=="Vertical").unwrap().bounds;
    assert!(button.height>48. && red.y+red.height<=button.y+button.height);
    state.rtl=true;let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{color:Color(191,56,72,255),..})));
    let mut action=None;for input in [Input{pointer:Some([40.,20.]),pressed:true,..Input::default()},Input{pointer:Some([40.,20.]),released:true,..Input::default()}] {ui.frame(input,bounds,0.,|f|{action=example::LAYOUT.render(f,bounds,&mut state);});}
    assert_eq!(action,Some("height_button"));
}

#[test]
fn button_wraps_multiline_text_and_grows_for_drawable_and_padding() {
    #[path = "../../../../../apps/bhoo-web/src/android_button_multiline.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:280.,height:112.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let lines:Vec<_>=commands.iter().filter_map(|c|if let DrawCommand::Text{text,bounds,..}=c{Some((text.to_string(),*bounds))}else{None}).collect();
    assert!(lines.len()>=2,"lines: {lines:?}");assert_eq!(lines.iter().map(|(s,_)|s.as_str()).collect::<Vec<_>>().join(" "),"Send the weekly update now");
    assert!(lines.windows(2).all(|w|w[1].1.y>=w[0].1.y+w[0].1.height));
    let button=ui.semantics().iter().find(|n|n.label.as_ref()=="Send the weekly update now").unwrap().bounds;assert!(button.height>48.);
    let green=commands.iter().find_map(|c|if let DrawCommand::RoundedRect{bounds,color:Color(41,115,79,255),..}=c{Some(*bounds)}else{None}).unwrap();assert!(green.y+green.height<=lines[0].1.y);
    let mut action=None;for input in [Input{pointer:Some([40.,30.]),pressed:true,..Input::default()},Input{pointer:Some([40.,30.]),released:true,..Input::default()}] {ui.frame(input,bounds,0.,|f|{action=example::LAYOUT.render(f,bounds,&mut state);});}assert_eq!(action,Some("multiline_button"));
    state.rtl=true;ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});let rtl=ui.semantics().iter().find(|n|n.label.as_ref()=="Send the weekly update now").unwrap().bounds;assert!((rtl.x+rtl.width-280.).abs()<0.01);
}

#[test]
fn multiline_button_baseline_aligns_first_rendered_line_with_textview() {
    #[path = "../../../../../apps/bhoo-web/src/android_button_multiline_baseline.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:280.,height:112.};let mut ui=Ui::default();let mut state=LayoutState::default();
    for rtl in [false,true] {
        state.rtl=rtl;let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
        let lines:Vec<_>=commands.iter().filter_map(|c|if let DrawCommand::Text{text,bounds,..}=c{Some((text.to_string(),*bounds))}else{None}).collect();
        let first=lines.iter().find(|(s,_)|s.starts_with("Send")).unwrap().1;
        let peer=lines.iter().find(|(s,_)|s=="Baseline").unwrap().1;
        assert!((first.y-peer.y).abs()<0.01,"first {first:?} peer {peer:?}");
        assert!(lines.iter().filter(|(s,_)|s!="Baseline").count()>=2);
    }
}

#[test]
fn button_max_lines_one_ellipsizes_start_middle_and_end() {
    #[path = "../../../../../apps/bhoo-web/src/android_button_ellipsis.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:280.,height:144.};let mut ui=Ui::default();let mut state=LayoutState::default();
    for rtl in [false,true] {
        state.rtl=rtl;let commands=ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
        let labels:Vec<_>=commands.iter().filter_map(|c|if let DrawCommand::Text{text,..}=c{Some(text.to_string())}else{None}).collect();
        assert_eq!(labels.len(),3,"labels: {labels:?}");
        assert!(labels[0].starts_with('…')&&!labels[0].ends_with('…'),"{:?}",labels[0]);
        assert!(!labels[1].starts_with('…')&&labels[1].contains('…')&&!labels[1].ends_with('…'),"{:?}",labels[1]);
        assert!(labels[2].ends_with('…')&&!labels[2].starts_with('…'),"{:?}",labels[2]);
    }
}

#[test]
fn nested_grid_button_ripple_matches_button_radius() {
    #[path = "../../../../../apps/bhoo-web/src/android_grid_nested_wrap.rs"] mod example;
    let bounds=Rect{x:0.,y:0.,width:320.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    ui.frame(Input::default(),bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let small=ui.semantics().iter().find(|n|n.label.as_ref()=="Small").unwrap().bounds;
    let pointer=[small.x+small.width*0.5,small.y+small.height*0.5];
    ui.frame(Input{pointer:Some(pointer),pressed:true,time:1.0,..Input::default()},bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let commands=ui.frame(Input{pointer:Some(pointer),time:1.1,..Input::default()},bounds,0.,|f|{example::LAYOUT.render(f,bounds,&mut state);});
    let ripple=commands.iter().find_map(|c|if let DrawCommand::Ripple{bounds,corner_radius,..}=c{Some((*bounds,*corner_radius))}else{None}).expect("expected button ripple");
    assert_eq!(ripple.0,small);
    assert_eq!(ripple.1,0.0);
}

#[test]
fn view_elevation_emits_shadow_before_background() {
    use bhoo::android_xml::RULE_ELEVATION;
    static RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_ELEVATION, anchor: "4" }];
    static ELEVATED: Node = Node {
        kind: Kind::View,
        background: Some(Color(240, 240, 240, 255)),
        radius: 8.0,
        relative_rules: RULES,
        ..button("elevated", Visibility::Visible, Size::Dp(64.), 0.)
    };
    let bounds = Rect { x: 0., y: 0., width: 160., height: 64. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let commands = ui.frame(Input::default(), bounds, 0., |f| { ELEVATED.render(f, bounds, &mut state); });
    let first_shadow = commands.iter().position(|c| matches!(c, DrawCommand::Shadow { .. }));
    let bg_rect = commands.iter().position(|c| matches!(c, DrawCommand::RoundedRect { color: Color(240, 240, 240, 255), .. }));
    assert!(first_shadow.is_some(), "expected DrawCommand::Shadow for elevation > 0");
    assert!(bg_rect.is_some(), "expected background RoundedRect");
    assert!(first_shadow.unwrap() < bg_rect.unwrap(), "shadow must be emitted before background");
}

#[test]
fn view_background_tint_overrides_background_fill() {
    use bhoo::android_xml::RULE_BACKGROUND_TINT;
    static RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_BACKGROUND_TINT, anchor: "12,134,200,255" }];
    static TINTED: Node = Node {
        kind: Kind::View,
        background: Some(Color(200, 200, 200, 255)),
        radius: 4.0,
        relative_rules: RULES,
        ..button("tinted", Visibility::Visible, Size::Dp(48.), 0.)
    };
    let bounds = Rect { x: 0., y: 0., width: 160., height: 48. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let commands = ui.frame(Input::default(), bounds, 0., |f| { TINTED.render(f, bounds, &mut state); });
    assert!(commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(12, 134, 200, 255), .. })));
    assert!(!commands.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(200, 200, 200, 255), .. })));
}

#[test]
fn text_alignment_applies_to_text_view_and_button_including_rtl() {
    use bhoo::android_xml::RULE_TEXT_ALIGNMENT;
    static START: &[RelativeRule] = &[RelativeRule { kind: RULE_TEXT_ALIGNMENT, anchor: "start" }];
    static CENTER: &[RelativeRule] = &[RelativeRule { kind: RULE_TEXT_ALIGNMENT, anchor: "center" }];
    static END: &[RelativeRule] = &[RelativeRule { kind: RULE_TEXT_ALIGNMENT, anchor: "end" }];
    static TV_CENTER: Node = Node { kind: Kind::Text, text: "Centered", relative_rules: CENTER, ..button("tv_center", Visibility::Visible, Size::Dp(48.), 0.) };
    static TV_END: Node = Node { kind: Kind::Text, text: "Ended", relative_rules: END, ..button("tv_end", Visibility::Visible, Size::Dp(48.), 0.) };
    static BTN_START: Node = Node { kind: Kind::Button, text: "StartBtn", relative_rules: START, ..button("btn_start", Visibility::Visible, Size::Dp(48.), 0.) };
    let bounds = Rect { x: 0., y: 0., width: 200., height: 48. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { TV_CENTER.render(f, bounds, &mut state); });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == "Centered" && style.align == 1)));
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { TV_END.render(f, bounds, &mut state); });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == "Ended" && style.align == 2)));
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { BTN_START.render(f, bounds, &mut state); });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == "StartBtn" && style.align == 0)));
    state.rtl = true;
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { TV_END.render(f, bounds, &mut state); });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == "Ended" && style.align == 0)));
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { BTN_START.render(f, bounds, &mut state); });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == "StartBtn" && style.align == 2)));
}

#[test]
fn constraint_layout_centers_child_and_applies_bias() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, RULE_CONSTRAINT_END_TO_END,
        RULE_CONSTRAINT_HORIZONTAL_BIAS, RULE_CONSTRAINT_START_TO_START,
        RULE_CONSTRAINT_TOP_TO_TOP, RULE_CONSTRAINT_VERTICAL_BIAS,
    };
    static CENTER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
    ];
    static BIASED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_HORIZONTAL_BIAS, anchor: "0.25" },
        RelativeRule { kind: RULE_CONSTRAINT_VERTICAL_BIAS, anchor: "0.75" },
    ];
    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(60.), relative_rules: CENTER_RULES, ..button("centered", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { width: Size::Dp(60.), relative_rules: BIASED_RULES, ..button("biased", Visibility::Visible, Size::Dp(40.), 0.) },
        ],
        ..button("cl_root", Visibility::Visible, Size::Match, 0.)
    };
    let bounds = Rect { x: 0., y: 0., width: 260., height: 140. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    let centered = ui.semantics().iter().find(|n| n.label.as_ref() == "centered").unwrap().bounds;
    assert_eq!(centered, Rect { x: 100., y: 50., width: 60., height: 40. });
    let biased = ui.semantics().iter().find(|n| n.label.as_ref() == "biased").unwrap().bounds;
    assert_eq!(biased, Rect { x: 50., y: 75., width: 60., height: 40. });
}

#[test]
fn constraint_layout_sibling_chaining_and_zero_dp_match_constraint() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, RULE_CONSTRAINT_END_TO_END,
        RULE_CONSTRAINT_END_TO_START, RULE_CONSTRAINT_START_TO_END,
        RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_BOTTOM,
        RULE_CONSTRAINT_TOP_TO_TOP,
    };
    static HEADER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static LEFT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "header" },
    ];
    static RIGHT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "header" },
    ];
    static MIDDLE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "left_box" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "right_box" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "header" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
    ];
    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(0.), margin: [10., 10., 10., 0.], relative_rules: HEADER_RULES, ..button("header", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { width: Size::Dp(50.), margin: [10., 8., 0., 0.], relative_rules: LEFT_RULES, ..button("left_box", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(60.), margin: [0., 8., 10., 0.], relative_rules: RIGHT_RULES, ..button("right_box", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(0.), margin: [6., 8., 4., 12.], relative_rules: MIDDLE_RULES, ..button("middle_fill", Visibility::Visible, Size::Dp(0.), 0.) },
        ],
        ..button("cl_chain", Visibility::Visible, Size::Match, 0.)
    };
    let bounds = Rect { x: 0., y: 0., width: 300., height: 200. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    let header = ui.semantics().iter().find(|n| n.label.as_ref() == "header").unwrap().bounds;
    assert_eq!(header, Rect { x: 10., y: 10., width: 280., height: 40. });
    let left_box = ui.semantics().iter().find(|n| n.label.as_ref() == "left_box").unwrap().bounds;
    assert_eq!(left_box, Rect { x: 10., y: 58., width: 50., height: 30. });
    let right_box = ui.semantics().iter().find(|n| n.label.as_ref() == "right_box").unwrap().bounds;
    assert_eq!(right_box, Rect { x: 230., y: 58., width: 60., height: 30. });
    // middle_fill: start = left_box.right(60) + 6 = 66, end = right_box.left(230) - 4 = 226 => width = 160
    // top = header.bottom(50) + 8 = 58, bottom = parent.bottom(200) - 12 = 188 => height = 130
    let middle = ui.semantics().iter().find(|n| n.label.as_ref() == "middle_fill").unwrap().bounds;
    assert_eq!(middle, Rect { x: 66., y: 58., width: 160., height: 130. });
}

#[test]
fn view_per_corner_radii_renders_asymmetric_fill_and_outline() {
    use bhoo::android_xml::RULE_CORNER_RADII;
    static TOP_ONLY: &[RelativeRule] = &[RelativeRule { kind: RULE_CORNER_RADII, anchor: "16,16,0,0" }];
    static LEFT_RIGHT: &[RelativeRule] = &[RelativeRule { kind: RULE_CORNER_RADII, anchor: "20,4,4,20" }];
    static FOUR_CORNERS: &[RelativeRule] = &[RelativeRule { kind: RULE_CORNER_RADII, anchor: "24,12,8,0" }];
    static UNIFORM: &[RelativeRule] = &[RelativeRule { kind: RULE_CORNER_RADII, anchor: "10,10,10,10" }];
    static TOP_VIEW: Node = Node {
        kind: Kind::View,
        background: Some(Color(100, 150, 200, 255)),
        background_stroke: Some(Color(20, 30, 40, 255)),
        background_stroke_width: 2.0,
        relative_rules: TOP_ONLY,
        ..button("top_rounded", Visibility::Visible, Size::Dp(80.), 0.)
    };
    static LR_VIEW: Node = Node { relative_rules: LEFT_RIGHT, background_stroke: None, ..TOP_VIEW };
    static QUAD_VIEW: Node = Node { relative_rules: FOUR_CORNERS, background_stroke: None, ..TOP_VIEW };
    static UNIFORM_VIEW: Node = Node { relative_rules: UNIFORM, background_stroke: None, ..TOP_VIEW };
    let bounds = Rect { x: 0., y: 0., width: 120., height: 80. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    let cmds = ui.frame(Input::default(), bounds, 0., |f| { TOP_VIEW.render(f, bounds, &mut state); });
    let fills: Vec<f32> = cmds.iter().filter_map(|c| match c { DrawCommand::RoundedRect { radius, color: Color(100, 150, 200, 255), .. } => Some(*radius), _ => None }).collect();
    let outlines: Vec<f32> = cmds.iter().filter_map(|c| match c { DrawCommand::Outline { radius, color: Color(20, 30, 40, 255), .. } => Some(*radius), _ => None }).collect();
    assert_eq!(fills, vec![16.0, 0.0]);
    assert_eq!(outlines, vec![16.0, 0.0]);
    assert_eq!(
        cmds.iter().filter(|c| matches!(c, DrawCommand::PushClip(_))).count(),
        cmds.iter().filter(|c| matches!(c, DrawCommand::PopClip)).count()
    );

    let cmds = ui.frame(Input::default(), bounds, 0., |f| { LR_VIEW.render(f, bounds, &mut state); });
    let lr_fills: Vec<f32> = cmds.iter().filter_map(|c| match c { DrawCommand::RoundedRect { radius, color: Color(100, 150, 200, 255), .. } => Some(*radius), _ => None }).collect();
    assert_eq!(lr_fills, vec![20.0, 4.0]);

    let cmds = ui.frame(Input::default(), bounds, 0., |f| { QUAD_VIEW.render(f, bounds, &mut state); });
    let quad_fills: Vec<f32> = cmds.iter().filter_map(|c| match c { DrawCommand::RoundedRect { radius, color: Color(100, 150, 200, 255), .. } => Some(*radius), _ => None }).collect();
    assert_eq!(quad_fills, vec![24.0, 12.0, 8.0, 0.0]);

    let cmds = ui.frame(Input::default(), bounds, 0., |f| { UNIFORM_VIEW.render(f, bounds, &mut state); });
    let uni_fills: Vec<f32> = cmds.iter().filter_map(|c| match c { DrawCommand::RoundedRect { radius, color: Color(100, 150, 200, 255), .. } => Some(*radius), _ => None }).collect();
    assert_eq!(uni_fills, vec![10.0]);
    assert_eq!(cmds.iter().filter(|c| matches!(c, DrawCommand::PushClip(_))).count(), 0);
}

#[test]
fn gradient_background_renders_directional_and_three_stop_clipped_bands() {
    use bhoo::android_xml::RULE_GRADIENT;
    static VERT_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_GRADIENT, anchor: "270,255,0,0,255,0,0,255,255" }];
    static HORIZ_THREE: &[RelativeRule] = &[RelativeRule { kind: RULE_GRADIENT, anchor: "0,255,0,0,255,0,255,0,255,0,0,255,255" }];
    static REV_HORIZ: &[RelativeRule] = &[RelativeRule { kind: RULE_GRADIENT, anchor: "180,255,0,0,255,0,0,255,255" }];
    static DIAG_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_GRADIENT, anchor: "45,255,0,0,255,0,0,255,255" }];
    static VERT: Node = Node { kind: Kind::View, radius: 12.0, relative_rules: VERT_RULES, ..button("g_vert", Visibility::Visible, Size::Dp(96.), 0.) };
    static HORIZ3: Node = Node { kind: Kind::View, radius: 0.0, relative_rules: HORIZ_THREE, ..button("g_h3", Visibility::Visible, Size::Dp(48.), 0.) };
    static REV: Node = Node { kind: Kind::View, radius: 0.0, relative_rules: REV_HORIZ, ..button("g_rev", Visibility::Visible, Size::Dp(48.), 0.) };
    static DIAG: Node = Node { kind: Kind::View, radius: 8.0, relative_rules: DIAG_RULES, ..button("g_diag", Visibility::Visible, Size::Dp(80.), 0.) };

    let bounds = Rect { x: 0., y: 0., width: 120., height: 96. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    let cmds = ui.frame(Input::default(), bounds, 0., |f| { VERT.render(f, bounds, &mut state); });
    let bands: Vec<(Rect, f32, Color)> = cmds.iter().filter_map(|c| match c { DrawCommand::RoundedRect { bounds, radius, color } => Some((*bounds, *radius, *color)), _ => None }).collect();
    assert!(bands.len() >= 16, "expected smooth stepped bands, got {}", bands.len());
    // Top band (270 deg = TOP_BOTTOM) is red-dominant and clipped to rounded bounds; bottom band is blue-dominant.
    let first = bands.first().unwrap().2;
    let last = bands.last().unwrap().2;
    assert!(first.0 > 220 && first.2 < 40, "top band should start red: {first:?}");
    assert!(last.2 > 220 && last.0 < 40, "bottom band should end blue: {last:?}");
    assert!(bands.iter().any(|(_, r, _)| (*r - 12.0).abs() < 0.01), "corner bands should use view corner radius");
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::PushClip(_))), "rounded corner bands should be clipped");
    assert_eq!(
        cmds.iter().filter(|c| matches!(c, DrawCommand::PushClip(_))).count(),
        cmds.iter().filter(|c| matches!(c, DrawCommand::PopClip)).count()
    );

    let cmds = ui.frame(Input::default(), bounds, 0., |f| { HORIZ3.render(f, bounds, &mut state); });
    let h3: Vec<Color> = cmds.iter().filter_map(|c| match c { DrawCommand::RoundedRect { color, .. } => Some(*color), _ => None }).collect();
    assert!(h3.len() >= 16);
    assert!(h3[0].0 > 200 && h3[0].1 < 60, "start should be red: {:?}", h3[0]);
    let mid = h3[h3.len() / 2];
    assert!(mid.1 > 200 && mid.0 < 60 && mid.2 < 60, "center should be green: {mid:?}");
    let end = *h3.last().unwrap();
    assert!(end.2 > 200 && end.1 < 60, "end should be blue: {end:?}");

    let cmds = ui.frame(Input::default(), bounds, 0., |f| { REV.render(f, bounds, &mut state); });
    let rev: Vec<Color> = cmds.iter().filter_map(|c| match c { DrawCommand::RoundedRect { color, .. } => Some(*color), _ => None }).collect();
    assert!(rev.first().unwrap().2 > 220 && rev.last().unwrap().0 > 220, "180 deg should run right-to-left");

    let cmds = ui.frame(Input::default(), bounds, 0., |f| { DIAG.render(f, bounds, &mut state); });
    let diag: Vec<(Rect, Color)> = cmds.iter().filter_map(|c| match c { DrawCommand::RoundedRect { bounds, color, .. } => Some((*bounds, *color)), _ => None }).collect();
    assert!(diag.len() >= 64, "diagonal gradient should render 2D stepped tiles");
    // 45 deg = BL_TR: first drawn tile is top-left (near middle), last row first col is bottom-left (red), first row last col is top-right (blue).
    assert!(diag.iter().any(|(_, c)| c.0 > 200 && c.2 < 60), "diagonal gradient should include start red");
    assert!(diag.iter().any(|(_, c)| c.2 > 200 && c.0 < 60), "diagonal gradient should include end blue");
}

#[test]
fn drawable_layer_list_renders_stacked_inset_layers_in_order() {
    use bhoo::android_xml::RULE_DRAWABLE_LAYER;
    static LAYERS: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_LAYER, anchor: "0,0,0,0|12|30,40,50,255|2,10,20,30,255" },
        RelativeRule { kind: RULE_DRAWABLE_LAYER, anchor: "6,4,6,4|8|200,210,220,255" },
        RelativeRule { kind: RULE_DRAWABLE_LAYER, anchor: "-2,-2,-2,0|0|none|2,255,0,0,255" },
    ];
    static LAYERED: Node = Node {
        kind: Kind::Text,
        text: "Layered",
        relative_rules: LAYERS,
        ..button("layered_view", Visibility::Visible, Size::Dp(60.), 0.)
    };
    let bounds = Rect { x: 10., y: 20., width: 160., height: 60. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { LAYERED.render(f, bounds, &mut state); });

    let l0_fill = cmds.iter().position(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(30, 40, 50, 255) } if *r == bounds && (*radius - 12.0).abs() < 0.01));
    let l0_stroke = cmds.iter().position(|c| matches!(c, DrawCommand::Outline { bounds: r, radius, width, color: Color(10, 20, 30, 255) } if *r == bounds && (*radius - 12.0).abs() < 0.01 && (*width - 2.0).abs() < 0.01));
    let l1_fill = cmds.iter().position(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(200, 210, 220, 255) } if *r == Rect { x: 16., y: 24., width: 148., height: 52. } && (*radius - 8.0).abs() < 0.01));
    let l2_stroke = cmds.iter().position(|c| matches!(c, DrawCommand::Outline { bounds: r, width, color: Color(255, 0, 0, 255), .. } if *r == Rect { x: 8., y: 18., width: 164., height: 62. } && (*width - 2.0).abs() < 0.01));
    let text_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Layered"));

    assert!(l0_fill.is_some() && l0_stroke.is_some() && l1_fill.is_some() && l2_stroke.is_some() && text_idx.is_some());
    assert!(l0_fill.unwrap() < l0_stroke.unwrap());
    assert!(l0_stroke.unwrap() < l1_fill.unwrap());
    assert!(l1_fill.unwrap() < l2_stroke.unwrap());
    assert!(l2_stroke.unwrap() < text_idx.unwrap(), "all layers must render behind text content");
    assert!(cmds[l1_fill.unwrap()..l2_stroke.unwrap()].iter().any(|c| matches!(c, DrawCommand::PushClip(r) if *r == bounds)), "negative inset layer must clip to view bounds");
}

#[test]
fn custom_ripple_color_and_unbounded_mask_apply_to_clickable_views_and_buttons() {
    use bhoo::android_xml::RULE_RIPPLE_COLOR;
    static BOUNDED_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_RIPPLE_COLOR, anchor: "220,40,80,255" }];
    static UNBOUNDED_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_RIPPLE_COLOR, anchor: "20,180,90,255|unbounded" }];
    static ICON_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_RIPPLE_COLOR, anchor: "255,140,0,255|unbounded" }];
    static CLICKABLE_VIEW: Node = Node {
        kind: Kind::View,
        clickable: true,
        radius: 10.0,
        relative_rules: BOUNDED_RULES,
        ..button("bounded_view", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static UNBOUNDED_BTN: Node = Node {
        kind: Kind::Button,
        text: "Go",
        radius: 4.0,
        relative_rules: UNBOUNDED_RULES,
        ..button("unbounded_btn", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static UNBOUNDED_ICON: Node = Node {
        kind: Kind::Icon,
        relative_rules: ICON_RULES,
        ..button("unbounded_icon", Visibility::Visible, Size::Dp(48.), 0.)
    };
    let bounds = Rect { x: 20., y: 20., width: 120., height: 48. };
    let viewport = Rect { x: 0., y: 0., width: 200., height: 120. };
    let pointer = [bounds.x + 60., bounds.y + 24.];

    // Bounded custom color on clickable View
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input { pointer: Some(pointer), pressed: true, time: 1.0, ..Input::default() }, viewport, 0., |f| { CLICKABLE_VIEW.render(f, bounds, &mut state); });
    let cmds = ui.frame(Input { pointer: Some(pointer), time: 1.1, ..Input::default() }, viewport, 0., |f| { CLICKABLE_VIEW.render(f, bounds, &mut state); });
    let (rb, rc, col) = cmds.iter().find_map(|c| match c {
        DrawCommand::Ripple { bounds, corner_radius, color, .. } => Some((*bounds, *corner_radius, *color)),
        _ => None,
    }).expect("expected bounded ripple");
    assert_eq!(rb, bounds);
    assert_eq!(rc, 10.0);
    assert_eq!((col.0, col.1, col.2), (220, 40, 80));
    assert!(col.3 > 0);

    // Unbounded custom color on Button
    let mut ui = Ui::default();
    ui.frame(Input { pointer: Some(pointer), pressed: true, time: 1.0, ..Input::default() }, viewport, 0., |f| { UNBOUNDED_BTN.render(f, bounds, &mut state); });
    let cmds = ui.frame(Input { pointer: Some(pointer), time: 1.1, ..Input::default() }, viewport, 0., |f| { UNBOUNDED_BTN.render(f, bounds, &mut state); });
    let (rb, rc, col) = cmds.iter().find_map(|c| match c {
        DrawCommand::Ripple { bounds, corner_radius, color, .. } => Some((*bounds, *corner_radius, *color)),
        _ => None,
    }).expect("expected unbounded button ripple");
    let expected_side = bounds.width.hypot(bounds.height);
    assert!((rb.width - expected_side).abs() < 0.05 && (rb.height - expected_side).abs() < 0.05);
    assert!((rc - expected_side * 0.5).abs() < 0.05);
    assert_eq!((col.0, col.1, col.2), (20, 180, 90));

    // Unbounded custom color on IconButton
    let mut ui = Ui::default();
    ui.frame(Input { pointer: Some(pointer), pressed: true, time: 1.0, ..Input::default() }, viewport, 0., |f| { UNBOUNDED_ICON.render(f, bounds, &mut state); });
    let cmds = ui.frame(Input { pointer: Some(pointer), time: 1.1, ..Input::default() }, viewport, 0., |f| { UNBOUNDED_ICON.render(f, bounds, &mut state); });
    let (_, _, col) = cmds.iter().find_map(|c| match c {
        DrawCommand::Ripple { bounds, corner_radius, color, .. } => Some((*bounds, *corner_radius, *color)),
        _ => None,
    }).expect("expected unbounded icon ripple");
    assert_eq!((col.0, col.1, col.2), (255, 140, 0));
}

#[test]
fn constraint_layout_guidelines_position_siblings_by_begin_end_and_percent() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BOTTOM_TO_TOP, RULE_CONSTRAINT_END_TO_START,
        RULE_CONSTRAINT_START_TO_END, RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_BOTTOM,
        RULE_CONSTRAINT_TOP_TO_TOP, RULE_GUIDE_BEGIN, RULE_GUIDE_END, RULE_GUIDE_ORIENTATION,
        RULE_GUIDE_PERCENT,
    };
    static V_BEGIN: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "vertical" },
        RelativeRule { kind: RULE_GUIDE_BEGIN, anchor: "60" },
    ];
    static V_END: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "vertical" },
        RelativeRule { kind: RULE_GUIDE_END, anchor: "40" },
    ];
    static V_PCT: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "vertical" },
        RelativeRule { kind: RULE_GUIDE_PERCENT, anchor: "0.25" },
    ];
    static H_BEGIN: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "horizontal" },
        RelativeRule { kind: RULE_GUIDE_BEGIN, anchor: "30" },
    ];
    static H_END: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "horizontal" },
        RelativeRule { kind: RULE_GUIDE_END, anchor: "20" },
    ];
    static H_PCT: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "horizontal" },
        RelativeRule { kind: RULE_GUIDE_PERCENT, anchor: "0.5" },
    ];
    static SPAN_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "g_v_begin" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "g_v_end" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "g_h_begin" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_TOP, anchor: "g_h_end" },
    ];
    static PCT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "g_v_pct" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "g_h_pct" },
    ];
    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: V_BEGIN, ..button("g_v_begin", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: V_END, ..button("g_v_end", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: V_PCT, ..button("g_v_pct", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: H_BEGIN, ..button("g_h_begin", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: H_END, ..button("g_h_end", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: H_PCT, ..button("g_h_pct", Visibility::Visible, Size::Wrap, 0.) },
            Node { width: Size::Dp(0.), relative_rules: SPAN_RULES, ..button("span_box", Visibility::Visible, Size::Dp(0.), 0.) },
            Node { width: Size::Dp(50.), relative_rules: PCT_RULES, ..button("pct_box", Visibility::Visible, Size::Dp(24.), 0.) },
        ],
        ..button("cl_guides", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 400., height: 200. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    // Guidelines should not emit semantic nodes.
    assert_eq!(ui.semantics().len(), 2);
    let get = |ui: &Ui, name: &str| ui.semantics().iter().find(|n| n.label.as_ref() == name).unwrap().bounds;
    // LTR: v_begin=60, v_end=360 => width=300; h_begin=30, h_end=180 => height=150
    assert_eq!(get(&ui, "span_box"), Rect { x: 60., y: 30., width: 300., height: 150. });
    // LTR: v_pct=100, h_pct=100
    assert_eq!(get(&ui, "pct_box"), Rect { x: 100., y: 100., width: 50., height: 24. });

    // RTL: v_begin=340, v_end=40 => span_box=[40, 340], width=300; v_pct=300 => pct_box starts at right=300 => x=250
    state.rtl = true;
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "span_box"), Rect { x: 40., y: 30., width: 300., height: 150. });
    assert_eq!(get(&ui, "pct_box"), Rect { x: 250., y: 100., width: 50., height: 24. });
}

#[test]
fn constraint_layout_horizontal_and_vertical_chains_spread_inside_packed_and_weighted() {
    use bhoo::android_xml::{
        RULE_CHAIN_STYLE_H, RULE_CHAIN_STYLE_V, RULE_CONSTRAINT_BOTTOM_TO_BOTTOM,
        RULE_CONSTRAINT_BOTTOM_TO_TOP, RULE_CONSTRAINT_END_TO_END, RULE_CONSTRAINT_END_TO_START,
        RULE_CONSTRAINT_HORIZONTAL_BIAS, RULE_CONSTRAINT_H_WEIGHT, RULE_CONSTRAINT_START_TO_END,
        RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_BOTTOM, RULE_CONSTRAINT_TOP_TO_TOP,
        RULE_CONSTRAINT_V_WEIGHT, RULE_GUIDE_BEGIN, RULE_GUIDE_END, RULE_GUIDE_ORIENTATION,
    };
    let bounds = Rect { x: 0., y: 0., width: 300., height: 240. };
    let get = |ui: &Ui, name: &str| ui.semantics().iter().find(|n| n.label.as_ref() == name).unwrap().bounds;

    // 1. Horizontal spread (default): 3 items of 40dp across 300dp => gaps = (300 - 120) / 4 = 45
    static HS_A: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "b" },
        RelativeRule { kind: RULE_CHAIN_STYLE_H, anchor: "spread" },
    ];
    static HS_B: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "a" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "c" },
    ];
    static HS_C: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "b" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
    ];
    static H_SPREAD: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(40.), relative_rules: HS_A, ..button("a", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: HS_B, ..button("b", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: HS_C, ..button("c", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("h_spread", Visibility::Visible, Size::Match, 0.)
    };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { H_SPREAD.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "a").x, 45.0);
    assert_eq!(get(&ui, "b").x, 130.0);
    assert_eq!(get(&ui, "c").x, 215.0);

    // 2. Horizontal spread_inside: 3 items of 40dp across 300dp => a=0, b=130, c=260
    static HSI_A: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "b" },
        RelativeRule { kind: RULE_CHAIN_STYLE_H, anchor: "spread_inside" },
    ];
    static H_SPREAD_INSIDE: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(40.), relative_rules: HSI_A, ..button("a", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: HS_B, ..button("b", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: HS_C, ..button("c", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("h_spread_inside", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { H_SPREAD_INSIDE.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "a").x, 0.0);
    assert_eq!(get(&ui, "b").x, 130.0);
    assert_eq!(get(&ui, "c").x, 260.0);

    // 3. Horizontal packed with bias 0.25 (LTR and RTL): a=60dp, b=40dp across 300dp => extra=200
    static HP_A: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "b" },
        RelativeRule { kind: RULE_CHAIN_STYLE_H, anchor: "packed" },
        RelativeRule { kind: RULE_CONSTRAINT_HORIZONTAL_BIAS, anchor: "0.25" },
    ];
    static HP_B: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "a" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
    ];
    static H_PACKED: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(60.), relative_rules: HP_A, ..button("a", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: HP_B, ..button("b", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("h_packed", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { H_PACKED.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "a"), Rect { x: 50., y: 0., width: 60., height: 30. });
    assert_eq!(get(&ui, "b"), Rect { x: 110., y: 0., width: 40., height: 30. });
    state.rtl = true;
    ui.frame(Input::default(), bounds, 0., |f| { H_PACKED.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "a"), Rect { x: 190., y: 0., width: 60., height: 30. });
    assert_eq!(get(&ui, "b"), Rect { x: 150., y: 0., width: 40., height: 30. });
    state.rtl = false;

    // 4. Horizontal weighted 0dp chain anchored to Guidelines (40..260 = 220dp), mixed fixed(20dp) + weight(1) + weight(3)
    static GL: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "vertical" },
        RelativeRule { kind: RULE_GUIDE_BEGIN, anchor: "40" },
    ];
    static GR: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "vertical" },
        RelativeRule { kind: RULE_GUIDE_END, anchor: "40" },
    ];
    static HW_A: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "gl" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "b" },
    ];
    static HW_B: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "a" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "c" },
        RelativeRule { kind: RULE_CONSTRAINT_H_WEIGHT, anchor: "1" },
    ];
    static HW_C: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "b" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "gr" },
        RelativeRule { kind: RULE_CONSTRAINT_H_WEIGHT, anchor: "3" },
    ];
    static H_WEIGHTED: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: GL, ..button("gl", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: GR, ..button("gr", Visibility::Visible, Size::Wrap, 0.) },
            Node { width: Size::Dp(20.), relative_rules: HW_A, ..button("a", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(0.), relative_rules: HW_B, ..button("b", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(0.), relative_rules: HW_C, ..button("c", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("h_weighted", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { H_WEIGHTED.render(f, bounds, &mut state); });
    // distributable = 220 - 20 = 200 => b = 50, c = 150
    assert_eq!(get(&ui, "a"), Rect { x: 40., y: 0., width: 20., height: 30. });
    assert_eq!(get(&ui, "b"), Rect { x: 60., y: 0., width: 50., height: 30. });
    assert_eq!(get(&ui, "c"), Rect { x: 110., y: 0., width: 150., height: 30. });

    // 5. Vertical weighted 0dp chain with a Gone middle item: weights 1 and 2 across 240dp => 80 and 160
    static VW_A: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_TOP, anchor: "gone_mid" },
        RelativeRule { kind: RULE_CONSTRAINT_V_WEIGHT, anchor: "1" },
        RelativeRule { kind: RULE_CHAIN_STYLE_V, anchor: "spread" },
    ];
    static VW_MID: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "a" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_TOP, anchor: "c" },
        RelativeRule { kind: RULE_CONSTRAINT_V_WEIGHT, anchor: "5" },
    ];
    static VW_C: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "gone_mid" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_V_WEIGHT, anchor: "2" },
    ];
    static V_WEIGHTED: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(80.), relative_rules: VW_A, ..button("a", Visibility::Visible, Size::Dp(0.), 0.) },
            Node { width: Size::Dp(80.), relative_rules: VW_MID, ..button("gone_mid", Visibility::Gone, Size::Dp(0.), 0.) },
            Node { width: Size::Dp(80.), relative_rules: VW_C, ..button("c", Visibility::Visible, Size::Dp(0.), 0.) },
        ],
        ..button("v_weighted", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { V_WEIGHTED.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "a"), Rect { x: 0., y: 0., width: 80., height: 80. });
    assert_eq!(get(&ui, "c"), Rect { x: 0., y: 80., width: 80., height: 160. });

    // 6. Vertical spread_inside and packed chains (2 items of 40dp across 240dp)
    static VSI_A: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_TOP, anchor: "b" },
        RelativeRule { kind: RULE_CHAIN_STYLE_V, anchor: "spread_inside" },
    ];
    static VSI_B: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "a" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
    ];
    static V_SPREAD_INSIDE: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(80.), relative_rules: VSI_A, ..button("a", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { width: Size::Dp(80.), relative_rules: VSI_B, ..button("b", Visibility::Visible, Size::Dp(40.), 0.) },
        ],
        ..button("v_spread_inside", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { V_SPREAD_INSIDE.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "a").y, 0.0);
    assert_eq!(get(&ui, "b").y, 200.0);

    static VP_A: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_TOP, anchor: "b" },
        RelativeRule { kind: RULE_CHAIN_STYLE_V, anchor: "packed" },
    ];
    static V_PACKED: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(80.), relative_rules: VP_A, ..button("a", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { width: Size::Dp(80.), relative_rules: VSI_B, ..button("b", Visibility::Visible, Size::Dp(40.), 0.) },
        ],
        ..button("v_packed", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { V_PACKED.render(f, bounds, &mut state); });
    // Packed default vertical bias 0.5: (240 - 80) * 0.5 = 80
    assert_eq!(get(&ui, "a").y, 80.0);
    assert_eq!(get(&ui, "b").y, 120.0);
}

#[test]
fn relative_layout_rtl_wrap_content_preserves_to_end_of_sibling_width() {
    #[path = "../../../../../apps/bhoo-web/src/android_relative_nested_wrap.rs"]
    mod example;
    let bounds = Rect { x: 0., y: 0., width: 300., height: 56. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.rtl = true;
    ui.frame(Input::default(), bounds, 0., |f| { example::LAYOUT.render(f, bounds, &mut state); });
    let get = |ui: &Ui, label: &str| ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds;
    let small = get(&ui, "Small");
    let more = get(&ui, "More");
    let after = get(&ui, "After");
    assert_eq!(more.width, 56.0, "toEndOf sibling in RTL wrap RelativeLayout must not collapse to 0");
    assert!((small.x + small.width - 300.0).abs() < 0.01);
    assert!((small.x - (more.x + more.width)).abs() < 0.01);
    assert!((more.x - (after.x + after.width)).abs() < 0.01);
}

#[test]
fn frame_layout_subclass_wrap_width_uses_active_text_metrics() {
    static CARD: Node = Node {
        kind: Kind::MaterialCard,
        width: Size::Wrap,
        height: Size::Wrap,
        children: &[Node {
            kind: Kind::Text,
            width: Size::Wrap,
            height: Size::Wrap,
            text: "Card title",
            clickable: true,
            ..button("card_text", Visibility::Visible, Size::Wrap, 0.)
        }],
        ..button("wrap_card", Visibility::Visible, Size::Wrap, 0.)
    };
    static ROW: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        children: &[CARD, Node { width: Size::Dp(48.), ..button("after_card", Visibility::Visible, Size::Dp(48.), 0.) }],
        ..button("row", Visibility::Visible, Size::Match, 0.)
    };
    let bounds = Rect { x: 0., y: 0., width: 320., height: 80. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { ROW.render(f, bounds, &mut state); });
    let before = ui.semantics().iter().find(|n| n.label.as_ref() == "after_card").unwrap().bounds.x;
    ui.text_metrics.normal = [0.8; 95];
    ui.frame(Input::default(), bounds, 0., |f| { ROW.render(f, bounds, &mut state); });
    let after = ui.semantics().iter().find(|n| n.label.as_ref() == "after_card").unwrap().bounds.x;
    assert!(after > before + 15.0, "MaterialCard wrap width must respond to active TextMetrics: before={before}, after={after}");
}

#[test]
fn android_theme_material_light_and_dark_render_screen_and_raised_aosp_button() {
    use bhoo::android_xml::{AndroidTheme, RULE_BACKGROUND_TINT, RULE_TEXT_ALL_CAPS};
    let bounds = Rect { x: 0., y: 0., width: 320., height: 600. };
    let mut ui = Ui::default();
    let original_bg = ui.theme.background;
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::MaterialLight;

    // 1. Render generated screen.rs with MaterialLight
    let cmds = ui.frame(Input::default(), bounds, 0., |f| {
        generated::LAYOUT.render(f, bounds, &mut state);
    });
    assert_eq!(ui.theme.background, original_bg, "Node::render must restore Ui.theme");
    // Root window surface #FAFAFA
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xFA, 0xFA, 0xFA, 255) }
            if *r == bounds && *radius == 0.0
    )));
    // Explicit dark text colors from screen.rs (#1F2320) preserved
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(31, 35, 32, 255), .. } if text.as_ref() == "Bhoo workspace"
    )));
    // Raised CONTINUE button with 4dp H / 6dp V visual inset, 2dp corner radius, subtle 2dp shadow, #D6D7D7 fill, #DE000000 14sp medium text
    let confirm_bounds = ui.semantics().iter().find(|n| n.label.as_ref() == "Continue").unwrap().bounds;
    let visual_b = Rect {
        x: confirm_bounds.x + 4.0,
        y: confirm_bounds.y + 6.0,
        width: confirm_bounds.width - 8.0,
        height: confirm_bounds.height - 12.0,
    };
    let rect_close = |a: Rect, b: Rect| {
        (a.x - b.x).abs() < 0.01
            && (a.y - b.y).abs() < 0.01
            && (a.width - b.width).abs() < 0.01
            && (a.height - b.height).abs() < 0.01
    };
    let shadow_idx = cmds.iter().position(|c| matches!(
        c,
        DrawCommand::Shadow { bounds: r, radius, .. } if rect_close(*r, visual_b) && (*radius - 2.0).abs() < 0.01
    ));
    let fill_idx = cmds.iter().position(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xD6, 0xD7, 0xD7, 255) }
            if rect_close(*r, visual_b) && (*radius - 2.0).abs() < 0.01
    ));
    let label_idx = cmds.iter().position(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0, 0, 0, 0xDE), style, .. }
            if text.as_ref() == "CONTINUE"
                && (style.size - 14.0).abs() < 0.01
                && style.weight == 500
                && style.align == 1
    ));
    assert!(shadow_idx.is_some(), "MaterialLight button must emit elevation Shadow");
    assert!(fill_idx.is_some(), "MaterialLight button must emit #D6D7D7 2dp-rounded 4dp-H/6dp-V-inset fill");
    assert!(label_idx.is_some(), "MaterialLight button must emit centered 14sp medium uppercase CONTINUE in #DE000000");
    assert!(shadow_idx.unwrap() < fill_idx.unwrap() && fill_idx.unwrap() < label_idx.unwrap());

    // Ripple on press matches 2dp radius and visual_b
    let center = [confirm_bounds.x + confirm_bounds.width * 0.5, confirm_bounds.y + confirm_bounds.height * 0.5];
    ui.frame(Input { pointer: Some(center), pressed: true, time: 1.0, ..Input::default() }, bounds, 0., |f| {
        generated::LAYOUT.render(f, bounds, &mut state);
    });
    let ripple_cmds = ui.frame(Input { pointer: Some(center), time: 1.1, ..Input::default() }, bounds, 0., |f| {
        generated::LAYOUT.render(f, bounds, &mut state);
    });
    assert!(ripple_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Ripple { bounds: r, corner_radius, .. } if rect_close(*r, visual_b) && (*corner_radius - 2.0).abs() < 0.01
    )));

    // 2. Unstyled screen tree (color: None) verifies default #DE000000 (light) and #FFFFFFFF / #303030 / #5A595B (dark)
    static TINT_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_BACKGROUND_TINT, anchor: "25,118,210,255" }];
    static NOCAPS_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_TEXT_ALL_CAPS, anchor: "false" }];
    static SCREEN_UNSTYLED: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Text, text: "Bhoo workspace", text_size: 22.0, ..button("h", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Text, text: "A reusable Android XML layout", ..button("d", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Button, text: "Continue", ..button("c", Visibility::Visible, Size::Dp(48.), 0.) },
            Node { kind: Kind::Button, text: "Tinted", relative_rules: TINT_RULES, ..button("tinted", Visibility::Visible, Size::Dp(48.), 0.) },
            Node { kind: Kind::Button, text: "MixedCase", relative_rules: NOCAPS_RULES, ..button("nocaps", Visibility::Visible, Size::Dp(48.), 0.) },
            Node { kind: Kind::Button, text: "Disabled", enabled: false, ..button("dis", Visibility::Visible, Size::Dp(48.), 0.) },
        ],
        ..button("root", Visibility::Visible, Size::Match, 0.)
    };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::MaterialLight;
    let light = ui.frame(Input::default(), bounds, 0., |f| { SCREEN_UNSTYLED.render(f, bounds, &mut state); });
    assert!(light.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0, 0, 0, 0xDE), .. } if text.as_ref() == "Bhoo workspace")));
    assert!(light.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0, 0, 0, 0xDE), .. } if text.as_ref() == "A reusable Android XML layout")));
    assert!(light.iter().any(|c| matches!(c, DrawCommand::RoundedRect { radius, color: Color(25, 118, 210, 255), .. } if (*radius - 2.0).abs() < 0.01)));
    assert!(light.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "MixedCase")));
    assert!(light.iter().any(|c| matches!(c, DrawCommand::RoundedRect { radius, color: Color(0xD6, 0xD7, 0xD7, a), .. } if (*radius - 2.0).abs() < 0.01 && *a < 200)));

    // MaterialDark
    state.theme_style = AndroidTheme::MaterialDark;
    let dark = ui.frame(Input::default(), bounds, 0., |f| { SCREEN_UNSTYLED.render(f, bounds, &mut state); });
    assert!(dark.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x30, 0x30, 0x30, 255) } if *r == bounds && *radius == 0.0)));
    assert!(dark.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(255, 255, 255, 255), .. } if text.as_ref() == "Bhoo workspace")));
    assert!(dark.iter().any(|c| matches!(c, DrawCommand::RoundedRect { radius, color: Color(0x5A, 0x59, 0x5B, 255), .. } if (*radius - 2.0).abs() < 0.01)));
    assert!(dark.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(255, 255, 255, 255), .. } if text.as_ref() == "CONTINUE")));
}

#[test]
fn android_theme_material3_light_and_dark_emit_pill_buttons_surfaces_and_text() {
    use bhoo::android_xml::AndroidTheme;
    static ROOT: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Text, text: "M3 Heading", text_size: 22.0, ..button("m3_h", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Button, text: "Continue", ..button("m3_btn", Visibility::Visible, Size::Dp(48.), 0.) },
        ],
        ..button("m3_root", Visibility::Visible, Size::Match, 0.)
    };
    static EXPLICIT_BG_ROOT: Node = Node {
        background: Some(Color(12, 34, 56, 255)),
        ..ROOT
    };
    let bounds = Rect { x: 0., y: 0., width: 320., height: 200. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // Material3Light
    state.theme_style = AndroidTheme::Material3Light;
    let light = ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    let btn_b = ui.semantics().iter().find(|n| n.label.as_ref() == "Continue").unwrap().bounds;
    let pill_b = Rect { x: btn_b.x, y: btn_b.y + 4.0, width: btn_b.width, height: 40.0 };
    assert!(light.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xFE, 0xF7, 0xFF, 255) } if *r == bounds && *radius == 0.0)));
    assert!(light.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0x1D, 0x1B, 0x20, 255), .. } if text.as_ref() == "M3 Heading")));
    assert!(light.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) } if *r == pill_b && (*radius - 20.0).abs() < 0.01)));
    assert!(light.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0xFF, 0xFF, 0xFF, 255), style, .. } if text.as_ref() == "Continue" && (style.size - 14.0).abs() < 0.01 && style.weight == 500 && style.align == 1)));
    assert!(!light.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })), "M3 filled button is flat by default");

    // Material3Dark
    state.theme_style = AndroidTheme::Material3Dark;
    let dark = ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert!(dark.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x14, 0x12, 0x18, 255) } if *r == bounds && *radius == 0.0)));
    assert!(dark.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0xE6, 0xE0, 0xE9, 255), .. } if text.as_ref() == "M3 Heading")));
    assert!(dark.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xD0, 0xBC, 0xFF, 255) } if *r == pill_b && (*radius - 20.0).abs() < 0.01)));
    assert!(dark.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0x38, 0x1E, 0x72, 255), style, .. } if text.as_ref() == "Continue" && (style.size - 14.0).abs() < 0.01 && style.weight == 500)));

    // Explicit root background skips theme window surface fill
    state.theme_style = AndroidTheme::Material3Light;
    let explicit = ui.frame(Input::default(), bounds, 0., |f| { EXPLICIT_BG_ROOT.render(f, bounds, &mut state); });
    assert!(explicit.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(12, 34, 56, 255), .. } if *r == bounds)));
    assert!(!explicit.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0xFE, 0xF7, 0xFF, 255), .. } if *r == bounds)));
}

#[test]
fn android_theme_edit_text_underline_and_toolbar_surface_elevation() {
    use bhoo::android_xml::AndroidTheme;
    static SCREEN: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Toolbar, text: "Settings", hint: "Account", ..button("tb", Visibility::Visible, Size::Dp(56.), 0.) },
            Node { kind: Kind::Edit, text: "", hint: "Email", ..button("email", Visibility::Visible, Size::Dp(48.), 0.) },
        ],
        ..button("screen", Visibility::Visible, Size::Match, 0.)
    };
    let bounds = Rect { x: 0., y: 0., width: 320., height: 200. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // MaterialLight unfocused: Toolbar #F5F5F5 + shadow; EditText 1dp #6B000000 bottom underline
    state.theme_style = AndroidTheme::MaterialLight;
    let unfocused = ui.frame(Input::default(), bounds, 0., |f| { SCREEN.render(f, bounds, &mut state); });
    assert!(unfocused.iter().any(|c| matches!(c, DrawCommand::Shadow { bounds: r, .. } if r.height == 56.0)));
    assert!(unfocused.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0xF5, 0xF5, 0xF5, 255), .. } if r.height == 56.0)));
    assert!(unfocused.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0, 0, 0, 0xDE), .. } if text.as_ref() == "Settings")));
    assert!(unfocused.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0, 0, 0, 0x6B), .. } if (r.height - 1.0).abs() < 0.01 && (r.y - (56.0 + 47.0)).abs() < 0.01)));

    // Focus EditText -> 2dp accent #009688 underline
    let _ = ui.frame(Input { pointer: Some([40., 80.]), pressed: true, time: 0.1, ..Input::default() }, bounds, 0., |f| { SCREEN.render(f, bounds, &mut state); });
    let focused = ui.frame(Input { pointer: Some([40., 80.]), released: true, time: 0.2, ..Input::default() }, bounds, 0., |f| { SCREEN.render(f, bounds, &mut state); });
    assert!(focused.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0x00, 0x96, 0x88, 255), .. } if (r.height - 2.0).abs() < 0.01)));

    // MaterialDark: Toolbar #212121 + shadow; EditText unfocused 1dp #8AFFFFFF -> focused 2dp #80CBC4
    let mut ui = Ui::default();
    state.theme_style = AndroidTheme::MaterialDark;
    let dark_unfocused = ui.frame(Input::default(), bounds, 0., |f| { SCREEN.render(f, bounds, &mut state); });
    assert!(dark_unfocused.iter().any(|c| matches!(c, DrawCommand::Shadow { bounds: r, .. } if r.height == 56.0)));
    assert!(dark_unfocused.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0x21, 0x21, 0x21, 255), .. } if r.height == 56.0)));
    assert!(dark_unfocused.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(255, 255, 255, 0x8A), .. } if (r.height - 1.0).abs() < 0.01)));

    // Material3Light: flat Toolbar #FEF7FF (no shadow); EditText container + Outline stroke + bottom indicator
    let mut ui = Ui::default();
    state.theme_style = AndroidTheme::Material3Light;
    let m3 = ui.frame(Input::default(), bounds, 0., |f| { SCREEN.render(f, bounds, &mut state); });
    assert!(!m3.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })), "M3 Toolbar is flat by default");
    assert!(m3.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0xFE, 0xF7, 0xFF, 255), .. } if r.height == 56.0)));
    assert!(m3.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE6, 0xE0, 0xE9, 255) } if (r.height - 48.0).abs() < 0.01 && (*radius - 4.0).abs() < 0.01)));
    assert!(m3.iter().any(|c| matches!(c, DrawCommand::Outline { width, color: Color(0x79, 0x74, 0x7E, 255), .. } if (*width - 1.0).abs() < 0.01)));
    assert!(m3.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0x49, 0x45, 0x4F, 255), .. } if (r.height - 1.0).abs() < 0.01)));
}

#[test]
fn android_theme_controls_accent_colors_and_wrap_button_uppercase_width() {
    use bhoo::android_xml::{AndroidTheme, RULE_TEXT_ALL_CAPS};
    static CONTROLS: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::ProgressBar, horizontal: true, indeterminate: false, progress: 50.0, ..button("pb", Visibility::Visible, Size::Dp(16.), 0.) },
            Node { kind: Kind::SeekBar, progress: 50.0, ..button("sb", Visibility::Visible, Size::Dp(48.), 0.) },
            Node { kind: Kind::CheckBox, checked: true, text: "Check", ..button("cb", Visibility::Visible, Size::Dp(48.), 0.) },
            Node { kind: Kind::RadioButton, checked: true, text: "Radio", ..button("rb", Visibility::Visible, Size::Dp(48.), 0.) },
            Node { kind: Kind::Switch, checked: true, text: "Toggle", ..button("sw", Visibility::Visible, Size::Dp(48.), 0.) },
        ],
        ..button("ctrl_root", Visibility::Visible, Size::Match, 0.)
    };
    let bounds = Rect { x: 0., y: 0., width: 320., height: 260. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // MaterialLight controls use #009688 accent
    state.theme_style = AndroidTheme::MaterialLight;
    let light_cmds = ui.frame(Input::default(), bounds, 0., |f| { CONTROLS.render(f, bounds, &mut state); });
    let teal_hits = light_cmds.iter().filter(|c| matches!(
        c,
        DrawCommand::RoundedRect { color: Color(0x00, 0x96, 0x88, 255), .. }
            | DrawCommand::Outline { color: Color(0x00, 0x96, 0x88, 255), .. }
    )).count();
    assert!(teal_hits >= 5, "Expected MaterialLight #009688 accent across all 5 controls, got {teal_hits}");

    // Material3Light controls use #6750A4 primary and #E8DEF8 secondary container tracks
    state.theme_style = AndroidTheme::Material3Light;
    let m3_cmds = ui.frame(Input::default(), bounds, 0., |f| { CONTROLS.render(f, bounds, &mut state); });
    let m3_primary_hits = m3_cmds.iter().filter(|c| matches!(
        c,
        DrawCommand::RoundedRect { color: Color(0x67, 0x50, 0xA4, 255), .. }
            | DrawCommand::Outline { color: Color(0x67, 0x50, 0xA4, 255), .. }
    )).count();
    let m3_track_hits = m3_cmds.iter().filter(|c| matches!(
        c,
        DrawCommand::RoundedRect { color: Color(0xE8, 0xDE, 0xF8, 255), .. }
    )).count();
    assert!(m3_primary_hits >= 5, "Expected M3 #6750A4 primary across all 5 controls, got {m3_primary_hits}");
    assert!(m3_track_hits >= 2, "Expected M3 #E8DEF8 secondaryContainer track on ProgressBar & SeekBar, got {m3_track_hits}");

    // Wrap-content Button horizontal measurement accounts for MaterialLight uppercase default width
    static NOCAPS: &[RelativeRule] = &[RelativeRule { kind: RULE_TEXT_ALL_CAPS, anchor: "false" }];
    static WRAP_ROW: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        children: &[
            Node { kind: Kind::Button, width: Size::Wrap, height: Size::Wrap, text: "Save changes", ..button("upper_btn", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Button, width: Size::Wrap, height: Size::Wrap, text: "Save changes", relative_rules: NOCAPS, ..button("mixed_btn", Visibility::Visible, Size::Wrap, 0.) },
        ],
        ..button("wrap_row", Visibility::Visible, Size::Match, 0.)
    };
    let mut ui = Ui::default();
    for ch in b'A'..=b'Z' {
        ui.text_metrics.medium[(ch - 32) as usize] = 0.85;
        ui.text_metrics.normal[(ch - 32) as usize] = 0.85;
    }
    for ch in b'a'..=b'z' {
        ui.text_metrics.medium[(ch - 32) as usize] = 0.45;
        ui.text_metrics.normal[(ch - 32) as usize] = 0.45;
    }
    let row_bounds = Rect { x: 0., y: 0., width: 500., height: 80. };
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::MaterialLight;
    ui.frame(Input::default(), row_bounds, 0., |f| { WRAP_ROW.render(f, row_bounds, &mut state); });
    let upper_w = ui.semantics()[0].bounds.width;
    let mixed_w = ui.semantics()[1].bounds.width;
    assert!(
        upper_w > mixed_w + 20.0,
        "MaterialLight wrap-content Button must measure uppercase width: upper={upper_w}, mixed={mixed_w}"
    );
}

#[test]
fn constraint_layout_dimension_ratio_and_percent_sizing() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, RULE_CONSTRAINT_DIMENSION_RATIO,
        RULE_CONSTRAINT_END_TO_END, RULE_CONSTRAINT_HEIGHT_PERCENT, RULE_CONSTRAINT_START_TO_END,
        RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP, RULE_CONSTRAINT_WIDTH_PERCENT,
    };
    static R16_9: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_DIMENSION_RATIO, anchor: "16:9" },
    ];
    static RH_16_9: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_DIMENSION_RATIO, anchor: "H,16:9" },
    ];
    static RW_4_3: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_DIMENSION_RATIO, anchor: "W,4:3" },
    ];
    static AFTER_W: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "w_ratio" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static R_FLOAT: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_DIMENSION_RATIO, anchor: "1.5" },
    ];
    static PCT_CENTER: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_WIDTH_PERCENT, anchor: "0.5" },
        RelativeRule { kind: RULE_CONSTRAINT_HEIGHT_PERCENT, anchor: "0.25" },
    ];
    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(160.), relative_rules: R16_9, ..button("r16_9", Visibility::Visible, Size::Dp(0.), 0.) },
            Node { width: Size::Dp(0.), relative_rules: RH_16_9, ..button("rh_16_9", Visibility::Visible, Size::Dp(0.), 0.) },
            Node { width: Size::Dp(0.), relative_rules: RW_4_3, ..button("w_ratio", Visibility::Visible, Size::Dp(90.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: AFTER_W, ..button("after_w", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(150.), relative_rules: R_FLOAT, ..button("r_float", Visibility::Visible, Size::Dp(0.), 0.) },
            Node { width: Size::Dp(0.), relative_rules: PCT_CENTER, ..button("pct_box", Visibility::Visible, Size::Dp(0.), 0.) },
        ],
        ..button("cl_ratio", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 240. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    let get = |name: &str| ui.semantics().iter().find(|n| n.label.as_ref() == name).unwrap().bounds;

    assert_eq!(get("r16_9"), Rect { x: 0., y: 0., width: 160., height: 90. });
    assert_eq!(get("rh_16_9"), Rect { x: 0., y: 0., width: 320., height: 180. });
    assert_eq!(get("w_ratio"), Rect { x: 0., y: 0., width: 120., height: 90. });
    assert_eq!(get("after_w"), Rect { x: 120., y: 0., width: 40., height: 30. });
    assert_eq!(get("r_float"), Rect { x: 0., y: 0., width: 150., height: 100. });
    // Percent box: 50% of 320 = 160 (centered -> x=80), 25% of 240 = 60 (centered -> y=90)
    assert_eq!(get("pct_box"), Rect { x: 80., y: 90., width: 160., height: 60. });
}

#[test]
fn constraint_layout_baseline_to_baseline_alignment() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BASELINE_TO_BASELINE, RULE_CONSTRAINT_START_TO_END,
        RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP,
    };
    static TARGET_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static SMALL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "big" },
        RelativeRule { kind: RULE_CONSTRAINT_BASELINE_TO_BASELINE, anchor: "big" },
    ];
    static BTN_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "small" },
        RelativeRule { kind: RULE_CONSTRAINT_BASELINE_TO_BASELINE, anchor: "big" },
    ];
    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node {
                kind: Kind::Text,
                text: "Heading",
                text_size: 28.0,
                width: Size::Wrap,
                margin: [0., 20., 0., 0.],
                relative_rules: TARGET_RULES,
                ..button("big", Visibility::Visible, Size::Wrap, 0.)
            },
            Node {
                kind: Kind::Text,
                text: "caption",
                text_size: 12.0,
                width: Size::Wrap,
                relative_rules: SMALL_RULES,
                ..button("small", Visibility::Visible, Size::Wrap, 0.)
            },
            Node {
                kind: Kind::Button,
                text: "Action",
                text_size: 14.0,
                width: Size::Wrap,
                relative_rules: BTN_RULES,
                ..button("action_btn", Visibility::Visible, Size::Dp(48.), 0.)
            },
        ],
        ..button("cl_baseline", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 360., height: 120. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    let offset = ui.text_metrics.normal_baseline_offset;
    let baseline_of = |label: &str| {
        cmds.iter()
            .find_map(|c| match c {
                DrawCommand::Text { bounds, text, style, .. } if text.as_ref() == label => {
                    Some(bounds.y + bounds.height * 0.5 + style.size * offset)
                }
                _ => None,
            })
            .unwrap()
    };
    let big_b = baseline_of("Heading");
    let small_b = baseline_of("caption");
    let btn_b = baseline_of("Action");
    assert!((big_b - small_b).abs() < 0.05, "text baseline mismatch: big={big_b}, small={small_b}");
    assert!((big_b - btn_b).abs() < 0.05, "button baseline mismatch: big={big_b}, btn={btn_b}");
    let big_y = cmds.iter().find_map(|c| match c { DrawCommand::Text { bounds, text, .. } if text.as_ref() == "Heading" => Some(bounds.y), _ => None }).unwrap();
    let small_y = cmds.iter().find_map(|c| match c { DrawCommand::Text { bounds, text, .. } if text.as_ref() == "caption" => Some(bounds.y), _ => None }).unwrap();
    assert!(small_y > big_y, "smaller text must be shifted downward to align baselines");
}

#[test]
fn constraint_layout_gone_margins_ltr_and_rtl() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_GONE_MARGINS, RULE_CONSTRAINT_START_TO_END,
        RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_BOTTOM, RULE_CONSTRAINT_TOP_TO_TOP,
    };
    static LEAD_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static FOLLOW_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "lead" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_GONE_MARGINS, anchor: "6,-1,14,-1,24,-1" },
    ];
    static BELOW_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "lead" },
        RelativeRule { kind: RULE_CONSTRAINT_GONE_MARGINS, anchor: "-1,30,-1,-1,-1,-1" },
    ];
    static ZERO_GONE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "lead" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "below" },
        RelativeRule { kind: RULE_CONSTRAINT_GONE_MARGINS, anchor: "-1,-1,-1,-1,0,-1" },
    ];
    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(50.), margin_relative: [Some(10.), None], relative_rules: LEAD_RULES, ..button("lead", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { width: Size::Dp(60.), margin_relative: [Some(8.), None], relative_rules: FOLLOW_RULES, ..button("follow", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(60.), margin: [0., 5., 0., 0.], relative_rules: BELOW_RULES, ..button("below", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(40.), margin_relative: [Some(12.), None], relative_rules: ZERO_GONE_RULES, ..button("zero_gone", Visibility::Visible, Size::Dp(20.), 0.) },
        ],
        ..button("cl_gone", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 300., height: 200. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let get = |ui: &Ui, name: &str| ui.semantics().iter().find(|n| n.label.as_ref() == name).unwrap().bounds;

    // LTR visible: lead=[10..60], follow starts at 60+8=68, below starts at 40+5=45, zero_gone starts at 60+12=72
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "follow").x, 68.0);
    assert_eq!(get(&ui, "below").y, 45.0);
    assert_eq!(get(&ui, "zero_gone").x, 72.0);

    // LTR gone: lead collapses to 0, follow uses goneMarginStart(24) overriding left(6) -> x=24; below uses goneMarginTop(30) -> y=30; zero_gone uses goneMarginStart(0) -> x=0
    state.visibility.insert("lead".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "follow").x, 24.0);
    assert_eq!(get(&ui, "below").y, 30.0);
    assert_eq!(get(&ui, "zero_gone").x, 0.0);

    // RTL visible: lead=[240..290], follow (60dp) right=240-8=232 -> x=172
    state.rtl = true;
    state.visibility.insert("lead".into(), Visibility::Visible);
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "follow").x, 172.0);

    // RTL gone: lead collapses to 300, follow uses goneMarginStart(24) overriding right(14) -> right=276, x=216
    state.visibility.insert("lead".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "follow").x, 216.0);
}

#[test]
fn constraint_layout_barrier_tracks_dynamic_text_and_visibility() {
    use bhoo::android_xml::{
        RULE_BARRIER, RULE_CONSTRAINT_START_TO_END, RULE_CONSTRAINT_START_TO_START,
        RULE_CONSTRAINT_TOP_TO_BOTTOM, RULE_CONSTRAINT_TOP_TO_TOP,
    };
    static A_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static B_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "label_a" },
    ];
    static END_BARRIER: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BARRIER, anchor: "end:label_a,label_b" },
    ];
    static BOTTOM_BARRIER: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BARRIER, anchor: "bottom:label_a,label_b" },
    ];
    static VAL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "end_barrier" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static BELOW_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "bot_barrier" },
    ];
    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::Text, text: "Hi", width: Size::Wrap, clickable: true, relative_rules: A_RULES, ..button("label_a", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Text, text: "Longer label", width: Size::Wrap, clickable: true, relative_rules: B_RULES, ..button("label_b", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, visibility: Visibility::Gone, relative_rules: END_BARRIER, ..button("end_barrier", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, visibility: Visibility::Gone, relative_rules: BOTTOM_BARRIER, ..button("bot_barrier", Visibility::Gone, Size::Wrap, 0.) },
            Node { width: Size::Dp(60.), margin_relative: [Some(8.), None], relative_rules: VAL_RULES, ..button("value", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(80.), margin: [0., 6., 0., 0.], relative_rules: BELOW_RULES, ..button("below", Visibility::Visible, Size::Dp(24.), 0.) },
        ],
        ..button("cl_barrier", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 400., height: 200. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let get = |ui: &Ui, label: &str| ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds;

    // 1. Initial LTR: Barriers emit no semantics (4 clickable views only); value sits after label_b
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(ui.semantics().len(), 4);
    let b_rect = get(&ui, "Longer label");
    let v_initial = get(&ui, "value");
    assert!((v_initial.x - (b_rect.x + b_rect.width + 8.0)).abs() < 0.05);
    assert!((get(&ui, "below").y - (b_rect.y + b_rect.height + 6.0)).abs() < 0.05);

    // 2. Dynamic text growth on label_a pushes barrier & value rightward
    state.text.insert("label_a".into(), "Much much wider dynamic text".into());
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    let a_wide = get(&ui, "Hi");
    let v_wide = get(&ui, "value");
    assert!(a_wide.width > b_rect.width + 40.0);
    assert!((v_wide.x - (a_wide.x + a_wide.width + 8.0)).abs() < 0.05);

    // 3. Hiding label_a (Gone) pulls barrier back to label_b
    state.visibility.insert("label_a".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    let b_after = get(&ui, "Longer label");
    assert!((get(&ui, "value").x - (b_after.x + b_after.width + 8.0)).abs() < 0.05);

    // 4. Hiding both referenced siblings collapses barrier to origin -> value at x=8
    state.visibility.insert("label_b".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "value").x, 8.0);

    // 5. RTL: end barrier tracks min x (leftmost edge of visible referenced views)
    state.rtl = true;
    state.visibility.insert("label_a".into(), Visibility::Visible);
    state.visibility.insert("label_b".into(), Visibility::Visible);
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    let a_rtl = get(&ui, "Hi");
    let v_rtl = get(&ui, "value");
    assert!((v_rtl.x + v_rtl.width - (a_rtl.x - 8.0)).abs() < 0.05);
}

#[test]
fn constraint_layout_group_propagates_gone_and_invisible() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_START_TO_END, RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP,
        RULE_GROUP_REFS,
    };
    static A_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static B_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "a" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static C_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "b" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static GRP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GROUP_REFS, anchor: "a,b" },
    ];
    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(60.), relative_rules: A_RULES, ..button("a", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(50.), relative_rules: B_RULES, ..button("b", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: C_RULES, ..button("c", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, relative_rules: GRP_RULES, ..button("grp", Visibility::Visible, Size::Wrap, 0.) },
        ],
        ..button("cl_group", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 300., height: 100. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let get = |ui: &Ui, name: &str| ui.semantics().iter().find(|n| n.label.as_ref() == name).map(|n| n.bounds);

    // Visible group: all 3 buttons present, Group itself non-visual
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(ui.semantics().len(), 3);
    assert_eq!(get(&ui, "c").unwrap().x, 110.0);

    // Invisible group: a & b are hidden (no semantics/draw), but space is preserved so c stays at x=110
    state.visibility.insert("grp".into(), Visibility::Invisible);
    let inv_cmds = ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(ui.semantics().len(), 1);
    assert!(get(&ui, "a").is_none() && get(&ui, "b").is_none());
    assert_eq!(get(&ui, "c").unwrap().x, 110.0);
    assert!(!inv_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "a" || text.as_ref() == "b")));

    // Gone group: a & b are collapsed and hidden, so c shifts to x=0
    state.visibility.insert("grp".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert_eq!(ui.semantics().len(), 1);
    assert!(get(&ui, "a").is_none() && get(&ui, "b").is_none());
    assert_eq!(get(&ui, "c").unwrap().x, 0.0);
}

#[test]
fn layout_state_density_and_font_scale_scale_dp_and_sp() {
    let mut default_state = LayoutState::default();
    assert_eq!(default_state.density, 1.0);
    assert_eq!(default_state.font_scale, 1.0);
    assert_eq!(default_state.effective_density(), 1.0);
    assert_eq!(default_state.effective_font_scale(), 1.0);
    default_state.density = 0.0;
    default_state.font_scale = -2.0;
    assert_eq!(default_state.effective_density(), 1.0);
    assert_eq!(default_state.effective_font_scale(), 1.0);

    static ROOT: Node = Node {
        kind: Kind::Linear,
        padding_edges: [10., 8., 10., 8.],
        padding_explicit: [true; 4],
        children: &[
            Node {
                kind: Kind::View,
                width: Size::Dp(60.),
                margin: [5., 4., 0., 6.],
                radius: 8.0,
                background: Some(Color(80, 120, 200, 255)),
                background_stroke: Some(Color(10, 20, 30, 255)),
                background_stroke_width: 2.0,
                clickable: true,
                ..button("dp_box", Visibility::Visible, Size::Dp(30.), 0.)
            },
            Node {
                kind: Kind::Text,
                text: "Scale me",
                text_size: 16.0,
                width: Size::Wrap,
                clickable: true,
                ..button("sp_text", Visibility::Visible, Size::Wrap, 0.)
            },
        ],
        ..button("scale_root", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 600., height: 400. };
    let mut ui = Ui::default();
    let get = |ui: &Ui, label: &str| ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds;

    // 1. Baseline 1.0 / 1.0
    let mut s1 = LayoutState::default();
    let c1 = ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut s1); });
    let box1 = get(&ui, "dp_box");
    let txt1 = get(&ui, "Scale me");
    assert_eq!(box1, Rect { x: 15., y: 12., width: 60., height: 30. });
    let txt_size_1 = c1.iter().find_map(|c| match c { DrawCommand::Text { text, style, .. } if text.as_ref() == "Scale me" => Some(style.size), _ => None }).unwrap();
    assert_eq!(txt_size_1, 16.0);

    // 2. density = 2.0, font_scale = 1.0 -> doubles dp geometry, radii, stroke width, and text size
    let mut s2 = LayoutState::default();
    s2.density = 2.0;
    let c2 = ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut s2); });
    let box2 = get(&ui, "dp_box");
    let txt2 = get(&ui, "Scale me");
    assert_eq!(box2, Rect { x: 30., y: 24., width: 120., height: 60. });
    assert!(c2.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, radius, color: Color(80, 120, 200, 255) } if *bounds == box2 && (*radius - 16.0).abs() < 0.01)));
    assert!(c2.iter().any(|c| matches!(c, DrawCommand::Outline { bounds, radius, width, color: Color(10, 20, 30, 255) } if *bounds == box2 && (*radius - 16.0).abs() < 0.01 && (*width - 4.0).abs() < 0.01)));
    let txt_size_2 = c2.iter().find_map(|c| match c { DrawCommand::Text { text, style, .. } if text.as_ref() == "Scale me" => Some(style.size), _ => None }).unwrap();
    assert_eq!(txt_size_2, 32.0);
    assert!((txt2.width - txt1.width * 2.0).abs() < 0.1);
    assert!((txt2.height - txt1.height * 2.0).abs() < 0.1);

    // 3. density = 1.0, font_scale = 1.5 -> dp geometry unchanged, sp text scales by 1.5x
    let mut s3 = LayoutState::default();
    s3.font_scale = 1.5;
    let c3 = ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut s3); });
    let box3 = get(&ui, "dp_box");
    let txt3 = get(&ui, "Scale me");
    assert_eq!(box3, box1);
    let txt_size_3 = c3.iter().find_map(|c| match c { DrawCommand::Text { text, style, .. } if text.as_ref() == "Scale me" => Some(style.size), _ => None }).unwrap();
    assert_eq!(txt_size_3, 24.0);
    assert!((txt3.width - txt1.width * 1.5).abs() < 0.1);
    assert!((txt3.height - txt1.height * 1.5).abs() < 0.1);

    // 4. Combined density = 2.0, font_scale = 1.5 -> sp text = 16 * 2 * 1.5 = 48
    let mut s4 = LayoutState::default();
    s4.density = 2.0;
    s4.font_scale = 1.5;
    let c4 = ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut s4); });
    let txt_size_4 = c4.iter().find_map(|c| match c { DrawCommand::Text { text, style, .. } if text.as_ref() == "Scale me" => Some(style.size), _ => None }).unwrap();
    assert_eq!(txt_size_4, 48.0);
}

#[test]
fn material3_polish_card_fab_elevation_and_tab_widget_indicator() {
    use bhoo::android_xml::{AndroidTheme, RULE_ELEVATION};
    static TABS: [Node; 2] = [
        Node { kind: Kind::Text, text: "Home", width: Size::Match, ..button("tab0", Visibility::Visible, Size::Match, 0.) },
        Node { kind: Kind::Text, text: "Saved", width: Size::Match, ..button("tab1", Visibility::Visible, Size::Match, 0.) },
    ];
    static EXPLICIT_ELEV: &[RelativeRule] = &[RelativeRule { kind: RULE_ELEVATION, anchor: "1" }];
    static SCREEN: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::TabWidget, children: &TABS, ..button("m3_tabs", Visibility::Visible, Size::Dp(48.), 0.) },
            Node {
                kind: Kind::MaterialCard,
                children: &[Node { kind: Kind::Text, text: "Card body", ..button("card_txt", Visibility::Visible, Size::Wrap, 0.) }],
                ..button("m3_card", Visibility::Visible, Size::Dp(64.), 0.)
            },
            Node {
                kind: Kind::Icon,
                width: Size::Dp(56.),
                radius: -1.0,
                background: Some(Color(0xEA, 0xDD, 0xFF, 255)),
                image_builtin: true,
                icon: MaterialIcon::Add,
                ..button("m3_fab", Visibility::Visible, Size::Dp(56.), 0.)
            },
            Node {
                kind: Kind::Icon,
                width: Size::Dp(56.),
                radius: -1.0,
                enabled: false,
                background: Some(Color(0xEA, 0xDD, 0xFF, 255)),
                ..button("m3_fab_disabled", Visibility::Visible, Size::Dp(56.), 0.)
            },
            Node {
                kind: Kind::Icon,
                width: Size::Dp(56.),
                radius: -1.0,
                relative_rules: EXPLICIT_ELEV,
                background: Some(Color(0xEA, 0xDD, 0xFF, 255)),
                ..button("m3_fab_custom_elev", Visibility::Visible, Size::Dp(56.), 0.)
            },
        ],
        ..button("m3_polish_root", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 320. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    let light = ui.frame(Input::default(), bounds, 0., |f| { SCREEN.render(f, bounds, &mut state); });

    // 1. TabWidget under M3Light: M3 surface (#FEF7FF) + 3dp active bottom indicator in M3 primary (#6750A4)
    assert!(light.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0xFE, 0xF7, 0xFF, 255), .. }
            if r.y == 0.0 && r.height == 48.0 && r.width == 320.0
    )));
    assert!(light.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0x67, 0x50, 0xA4, 255), .. }
            if (r.x - 0.0).abs() < 0.01 && (r.width - 160.0).abs() < 0.01 && (r.y - 45.0).abs() < 0.01 && (r.height - 3.0).abs() < 0.01
    )), "M3 TabWidget must paint 3dp primary active indicator bar");

    // 2. MaterialCard under M3Light: default 12dp radius, M3 surface (#FEF7FF), M3 outlineVariant (#CAC4D0)
    assert!(light.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xFE, 0xF7, 0xFF, 255) }
            if r.y == 48.0 && r.height == 64.0 && (*radius - 12.0).abs() < 0.01
    )));
    assert!(light.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, color: Color(0xCA, 0xC4, 0xD0, 255) }
            if r.y == 48.0 && r.height == 64.0 && (*radius - 12.0).abs() < 0.01 && (*width - 1.0).abs() < 0.01
    )));

    // 3. Enabled FAB (ImageButton radius < 0) emits M3 level-3 shadow (radius 28, blur 5.0); disabled FAB emits none; explicit elev=1 emits level-1 shadow (blur 3.0)
    let fab_shadows: Vec<f32> = light.iter().filter_map(|c| match c {
        DrawCommand::Shadow { bounds: r, radius, blur, .. } if (r.y - 112.0).abs() < 0.01 && (*radius - 28.0).abs() < 0.01 => Some(*blur),
        _ => None,
    }).collect();
    assert_eq!(fab_shadows.len(), 3, "enabled M3 FAB must emit level-3 elevation shadow");
    assert!(fab_shadows.iter().any(|b| (*b - 5.0).abs() < 0.01));
    assert!(!light.iter().any(|c| matches!(c, DrawCommand::Shadow { bounds: r, .. } if (r.y - 168.0).abs() < 0.01)), "disabled M3 FAB must not emit default shadow");
    let custom_shadows: Vec<f32> = light.iter().filter_map(|c| match c {
        DrawCommand::Shadow { bounds: r, blur, .. } if (r.y - 224.0).abs() < 0.01 => Some(*blur),
        _ => None,
    }).collect();
    assert_eq!(custom_shadows.len(), 3);
    assert!(!custom_shadows.iter().any(|b| (*b - 5.0).abs() < 0.01), "explicit elevation=1 should use level 1 instead of default FAB level 3");

    // 4. Material3Dark: dark surface (#141218), dark primary (#D0BCFF) 3dp indicator on selected tab 1, dark outlineVariant (#49454F)
    state.theme_style = AndroidTheme::Material3Dark;
    state.displayed_children.insert("m3_tabs".into(), 1);
    let dark = ui.frame(Input::default(), bounds, 0., |f| { SCREEN.render(f, bounds, &mut state); });
    assert!(dark.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0xD0, 0xBC, 0xFF, 255), .. }
            if (r.x - 160.0).abs() < 0.01 && (r.width - 160.0).abs() < 0.01 && (r.y - 45.0).abs() < 0.01 && (r.height - 3.0).abs() < 0.01
    )));
    assert!(dark.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, color: Color(0x49, 0x45, 0x4F, 255), .. }
            if r.y == 48.0 && (*radius - 12.0).abs() < 0.01
    )));

    // 5. Bhoo theme: FAB does not get automatic M3 shadow, TabWidget keeps 2dp indicator
    state.theme_style = AndroidTheme::Bhoo;
    let bhoo_cmds = ui.frame(Input::default(), bounds, 0., |f| { SCREEN.render(f, bounds, &mut state); });
    assert!(!bhoo_cmds.iter().any(|c| matches!(c, DrawCommand::Shadow { bounds: r, .. } if (r.y - 112.0).abs() < 0.01)));
    assert!(bhoo_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, .. } if (r.y - 46.0).abs() < 0.01 && (r.height - 2.0).abs() < 0.01)));
}

#[test]
fn coordinator_layout_scroll_behavior_and_anchor_placement_ltr_and_rtl() {
    use bhoo::android_xml::{RULE_COORDINATOR_ANCHOR, RULE_COORDINATOR_SCROLL_BEHAVIOR};
    static SCROLL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_COORDINATOR_SCROLL_BEHAVIOR, anchor: "" }];
    static FAB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_COORDINATOR_ANCHOR, anchor: "appbar:bottom|end" }];
    static BADGE_TOP_START: &[RelativeRule] = &[RelativeRule { kind: RULE_COORDINATOR_ANCHOR, anchor: "card:top|start" }];
    static BADGE_CENTER: &[RelativeRule] = &[RelativeRule { kind: RULE_COORDINATOR_ANCHOR, anchor: "card:center" }];

    static APPBAR_CHILDREN: [Node; 1] = [
        Node { kind: Kind::Toolbar, text: "Feed", ..button("tb", Visibility::Visible, Size::Dp(56.), 0.) },
    ];
    static SCROLL_CONTENT: [Node; 1] = [
        Node { kind: Kind::Button, text: "First row", ..button("first_row", Visibility::Visible, Size::Dp(48.), 0.) },
    ];
    static COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, children: &APPBAR_CHILDREN, ..button("appbar", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Scroll, relative_rules: SCROLL_RULES, children: &SCROLL_CONTENT, ..button("scroll", Visibility::Visible, Size::Match, 0.) },
            Node {
                kind: Kind::Icon,
                width: Size::Dp(56.),
                margin_relative: [None, Some(16.)],
                radius: -1.0,
                text: "Add",
                icon: MaterialIcon::Add,
                relative_rules: FAB_RULES,
                ..button("fab", Visibility::Visible, Size::Dp(56.), 0.)
            },
        ],
        ..button("coord", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 360., height: 320. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let get = |ui: &Ui, label: &str| ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds;

    // 1. LTR: scroll content starts below 56dp appbar (y=56), FAB centered on appbar bottom (y=28) with 16dp end margin (x=288)
    ui.frame(Input::default(), bounds, 0., |f| { COORD.render(f, bounds, &mut state); });
    let row = get(&ui, "First row");
    assert_eq!(row.y, 56.0, "scrolling child must start below appbar instead of y=0");
    assert_eq!(row.height, 48.0);
    let fab = get(&ui, "Add");
    assert_eq!(fab, Rect { x: 288., y: 28., width: 56., height: 56. });

    // 2. RTL: FAB moves to start/left side with 16dp margin (x=16), still straddling appbar bottom (y=28)
    state.rtl = true;
    ui.frame(Input::default(), bounds, 0., |f| { COORD.render(f, bounds, &mut state); });
    let fab_rtl = get(&ui, "Add");
    assert_eq!(fab_rtl, Rect { x: 16., y: 28., width: 56., height: 56. });

    // 3. Hiding appbar (Gone) lets scroll content move to y=0
    state.rtl = false;
    state.visibility.insert("appbar".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| { COORD.render(f, bounds, &mut state); });
    assert_eq!(get(&ui, "First row").y, 0.0);

    // 4. Direct Toolbar sibling + interior anchor corners (top|start and center)
    static DIRECT_TB: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Toolbar, text: "Direct", ..button("direct_tb", Visibility::Visible, Size::Dp(64.), 0.) },
            Node { kind: Kind::Linear, relative_rules: SCROLL_RULES, children: &SCROLL_CONTENT, ..button("body", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::View, width: Size::Dp(120.), margin: [80., 100., 0., 0.], ..button("card", Visibility::Visible, Size::Dp(80.), 0.) },
            Node { kind: Kind::Button, width: Size::Dp(20.), text: "Corner", relative_rules: BADGE_TOP_START, ..button("badge_corner", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { kind: Kind::Button, width: Size::Dp(20.), text: "Mid", relative_rules: BADGE_CENTER, ..button("badge_mid", Visibility::Visible, Size::Dp(20.), 0.) },
        ],
        ..button("coord_direct", Visibility::Visible, Size::Match, 0.)
    };
    let mut state2 = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { DIRECT_TB.render(f, bounds, &mut state2); });
    assert_eq!(get(&ui, "First row").y, 64.0);
    // card is at x=80..200, y=100..180 -> top|start LTR=(80,100) => centered 20x20 at (70,90); center=(140,140) => (130,130)
    assert_eq!(get(&ui, "Corner"), Rect { x: 70., y: 90., width: 20., height: 20. });
    assert_eq!(get(&ui, "Mid"), Rect { x: 130., y: 130., width: 20., height: 20. });
    state2.rtl = true;
    ui.frame(Input::default(), bounds, 0., |f| { DIRECT_TB.render(f, bounds, &mut state2); });
    // In RTL, card has layout_gravity=[0,0] (start -> right edge): x = 360 - 120 = 240, right = 360, top = 100 -> top|start = (360,100) clamped to max_x=340
    let rtl_corner = get(&ui, "Corner");
    assert_eq!(rtl_corner.y, 90.0);
    assert_eq!(rtl_corner.x, 340.0);
}

#[test]
fn text_input_layout_outline_and_filled_modes_focus_error_and_end_icons() {
    use bhoo::android_xml::{AndroidTheme, RULE_TEXT_INPUT_BOX};

    static OUTLINE_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_TEXT_INPUT_BOX,
        anchor: "outline|8||Email|Enter work email|1|clear_text",
    }];
    static FILLED_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_TEXT_INPUT_BOX,
        anchor: "filled|4||Password|At least 8 chars|0|password_toggle",
    }];
    static EMAIL_CHILD: [Node; 1] = [
        Node { kind: Kind::Edit, text: "alice@example.com", hint: "Email", ..button("email_edit", Visibility::Visible, Size::Match, 0.) },
    ];
    static PASS_CHILD: [Node; 1] = [
        Node { kind: Kind::Edit, text: "secret", hint: "Password", input_type: 4, ..button("pass_edit", Visibility::Visible, Size::Match, 0.) },
    ];
    static OUTLINE_TIL: Node = Node {
        kind: Kind::Linear,
        relative_rules: OUTLINE_RULES,
        children: &EMAIL_CHILD,
        ..button("til_outline", Visibility::Visible, Size::Dp(84.), 0.)
    };
    static FILLED_TIL: Node = Node {
        kind: Kind::Linear,
        relative_rules: FILLED_RULES,
        children: &PASS_CHILD,
        ..button("til_filled", Visibility::Visible, Size::Dp(80.), 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 84. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Outline mode unfocused: 1dp outline (#79747E) radius 8, notch (#FEF7FF) behind floating "Email", helper text, Close icon
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { OUTLINE_TIL.render(f, bounds, &mut state); });
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { radius, width, color: Color(0x79, 0x74, 0x7E, 255), .. }
            if (*radius - 8.0).abs() < 0.01 && (*width - 1.0).abs() < 0.01
    )));
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xFE, 0xF7, 0xFF, 255) }
            if *radius == 0.0 && r.width > 10.0 && r.width < 120.0 && r.y <= 8.0
    )), "outline TextInputLayout must paint background notch behind floating label");
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == "Email" && (style.size - 12.0).abs() < 0.01)));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Enter work email")));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "alice@example.com")));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Shape { .. })), "clear_text must draw MaterialIcon::Close");

    // 2. Focus child EditText -> 2dp primary (#6750A4) stroke & primary floating label
    let edit_b = ui.semantics().iter().find(|n| n.role == Role::TextField).unwrap().bounds;
    let focus_pt = [edit_b.x + 20., edit_b.y + edit_b.height * 0.5];
    let focused_cmds = ui.frame(Input { pointer: Some(focus_pt), pressed: true, ..Input::default() }, bounds, 0., |f| {
        OUTLINE_TIL.render(f, bounds, &mut state);
    });
    assert!(focused_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { width, color: Color(0x67, 0x50, 0xA4, 255), .. } if (*width - 2.0).abs() < 0.01
    )));
    ui.frame(Input { pointer: Some(focus_pt), released: true, ..Input::default() }, bounds, 0., |f| {
        OUTLINE_TIL.render(f, bounds, &mut state);
    });

    // 3. Error active via state.text["til_outline:error"] -> error stroke (#B3261E) + error message replaces helper
    state.text.insert("til_outline:error".into(), "Invalid address".into());
    let err_cmds = ui.frame(Input::default(), bounds, 0., |f| { OUTLINE_TIL.render(f, bounds, &mut state); });
    assert!(err_cmds.iter().any(|c| matches!(c, DrawCommand::Outline { color: Color(0xB3, 0x26, 0x1E, 255), .. })));
    assert!(err_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0xB3, 0x26, 0x1E, 255), .. } if text.as_ref() == "Invalid address")));
    assert!(!err_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Enter work email")));

    // 4. Click clear_text trailing button -> clears child EditText state.text
    let clear_b = ui.semantics().iter().find(|n| n.label.as_ref() == "Clear text").unwrap().bounds;
    let clear_pt = [clear_b.x + clear_b.width * 0.5, clear_b.y + clear_b.height * 0.5];
    ui.frame(Input { pointer: Some(clear_pt), pressed: true, ..Input::default() }, bounds, 0., |f| {
        OUTLINE_TIL.render(f, bounds, &mut state);
    });
    let mut act = None;
    ui.frame(Input { pointer: Some(clear_pt), released: true, ..Input::default() }, bounds, 0., |f| {
        act = OUTLINE_TIL.render(f, bounds, &mut state);
    });
    assert_eq!(act, Some("til_outline"));
    assert_eq!(state.text.get("email_edit").map(String::as_str), Some(""));

    // 5. Filled mode + password_toggle + error_enabled=0 ignores :error key
    let f_bounds = Rect { x: 0., y: 0., width: 320., height: 80. };
    let mut ui2 = Ui::default();
    let mut state2 = LayoutState::default();
    state2.theme_style = AndroidTheme::Material3Light;
    state2.text.insert("til_filled:error".into(), "Should be ignored".into());
    let f_cmds = ui2.frame(Input::default(), f_bounds, 0., |f| { FILLED_TIL.render(f, f_bounds, &mut state2); });
    assert!(f_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { radius, color: Color(0xE6, 0xE0, 0xE9, 255), .. } if (*radius - 4.0).abs() < 0.01)));
    assert!(f_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0x49, 0x45, 0x4F, 255), .. } if (r.height - 1.0).abs() < 0.01)));
    assert!(f_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Password")));
    assert!(f_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "At least 8 chars")));
    assert!(!f_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Should be ignored")));
    assert!(f_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "••••••")), "password must start masked");
    assert!(f_cmds.iter().any(|c| matches!(c, DrawCommand::Line { .. })), "masked eye icon should include slash");

    // Click password_toggle -> unmasks text to "secret" and removes slash line
    let toggle_b = ui2.semantics().iter().find(|n| n.label.as_ref() == "Toggle password visibility").unwrap().bounds;
    let toggle_pt = [toggle_b.x + toggle_b.width * 0.5, toggle_b.y + toggle_b.height * 0.5];
    ui2.frame(Input { pointer: Some(toggle_pt), pressed: true, ..Input::default() }, f_bounds, 0., |f| {
        FILLED_TIL.render(f, f_bounds, &mut state2);
    });
    let unmasked = ui2.frame(Input { pointer: Some(toggle_pt), released: true, ..Input::default() }, f_bounds, 0., |f| {
        FILLED_TIL.render(f, f_bounds, &mut state2);
    });
    assert!(unmasked.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "secret")));
    assert!(!unmasked.iter().any(|c| matches!(c, DrawCommand::Line { .. })), "revealed eye icon should have no slash");

    // Focus filled EditText -> 2dp primary bottom indicator
    let f_edit_b = ui2.semantics().iter().find(|n| n.role == Role::TextField).unwrap().bounds;
    let f_focus_pt = [f_edit_b.x + 20., f_edit_b.y + f_edit_b.height * 0.5];
    let f_focused = ui2.frame(Input { pointer: Some(f_focus_pt), pressed: true, ..Input::default() }, f_bounds, 0., |f| {
        FILLED_TIL.render(f, f_bounds, &mut state2);
    });
    assert!(f_focused.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0x67, 0x50, 0xA4, 255), .. } if (r.height - 2.0).abs() < 0.01
    )));
}

#[test]
fn material_button_variants_and_custom_background_override() {
    use bhoo::android_xml::{AndroidTheme, RULE_BUTTON_VARIANT};

    static FILLED: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "filled" }];
    static OUTLINED: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "outlined" }];
    static TONAL: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "tonal" }];
    static TEXT_V: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "text" }];
    static ELEVATED: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "elevated" }];

    static BTN_FILLED: Node = Node { text: "Filled", relative_rules: FILLED, ..button("b_filled", Visibility::Visible, Size::Dp(48.), 0.) };
    static BTN_FILLED_CUSTOM_R: Node = Node { text: "Square", radius: 8.0, relative_rules: FILLED, ..button("b_sq", Visibility::Visible, Size::Dp(48.), 0.) };
    static BTN_OUTLINED: Node = Node { text: "Outlined", relative_rules: OUTLINED, ..button("b_outlined", Visibility::Visible, Size::Dp(48.), 0.) };
    static BTN_OUTLINED_CUSTOM: Node = Node { text: "CustomStroke", background_stroke: Some(Color(10, 20, 30, 255)), background_stroke_width: 2.0, relative_rules: OUTLINED, ..button("b_out_c", Visibility::Visible, Size::Dp(48.), 0.) };
    static BTN_TONAL: Node = Node { text: "Tonal", relative_rules: TONAL, ..button("b_tonal", Visibility::Visible, Size::Dp(48.), 0.) };
    static BTN_TEXT: Node = Node { text: "TextBtn", relative_rules: TEXT_V, ..button("b_text", Visibility::Visible, Size::Dp(48.), 0.) };
    static BTN_ELEVATED: Node = Node { text: "Elevated", relative_rules: ELEVATED, ..button("b_elev", Visibility::Visible, Size::Dp(48.), 0.) };
    static BTN_CUSTOM_BG: Node = Node { text: "CustomBg", background: Some(Color(12, 34, 56, 255)), relative_rules: FILLED, ..button("b_custom", Visibility::Visible, Size::Dp(48.), 0.) };

    let bounds = Rect { x: 0., y: 0., width: 160., height: 48. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // filled: M3 primary (#6750A4) pill (r=20) + on_primary (#FFFFFF) text, no shadow
    let c = ui.frame(Input::default(), bounds, 0., |f| { BTN_FILLED.render(f, bounds, &mut state); });
    assert!(c.iter().any(|d| matches!(d, DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) } if (r.height - 40.0).abs() < 0.01 && (*radius - 20.0).abs() < 0.01)));
    assert!(c.iter().any(|d| matches!(d, DrawCommand::Text { text, color: Color(0xFF, 0xFF, 0xFF, 255), .. } if text.as_ref() == "Filled")));
    assert!(!c.iter().any(|d| matches!(d, DrawCommand::Shadow { .. })));

    // filled with explicit radius=8 respects self.radius
    let c = ui.frame(Input::default(), bounds, 0., |f| { BTN_FILLED_CUSTOM_R.render(f, bounds, &mut state); });
    assert!(c.iter().any(|d| matches!(d, DrawCommand::RoundedRect { radius, color: Color(0x67, 0x50, 0xA4, 255), .. } if (*radius - 8.0).abs() < 0.01)));

    // outlined: transparent fill + 1dp outline (#79747E) + primary text
    let c = ui.frame(Input::default(), bounds, 0., |f| { BTN_OUTLINED.render(f, bounds, &mut state); });
    assert!(!c.iter().any(|d| matches!(d, DrawCommand::RoundedRect { bounds: r, .. } if (r.height - 40.0).abs() < 0.01)));
    assert!(c.iter().any(|d| matches!(d, DrawCommand::Outline { bounds: r, width, color: Color(0x79, 0x74, 0x7E, 255), .. } if (r.height - 40.0).abs() < 0.01 && (*width - 1.0).abs() < 0.01)));
    assert!(c.iter().any(|d| matches!(d, DrawCommand::Text { text, color: Color(0x67, 0x50, 0xA4, 255), .. } if text.as_ref() == "Outlined")));

    // outlined with explicit background_stroke uses custom color & width once (not duplicated on outer bounds)
    let c = ui.frame(Input::default(), bounds, 0., |f| { BTN_OUTLINED_CUSTOM.render(f, bounds, &mut state); });
    let outlines: Vec<_> = c.iter().filter(|d| matches!(d, DrawCommand::Outline { color: Color(10, 20, 30, 255), width, .. } if (*width - 2.0).abs() < 0.01)).collect();
    assert_eq!(outlines.len(), 1);

    // tonal: secondaryContainer (#E8DEF8) fill + onSecondaryContainer (#1D192B) text
    let c = ui.frame(Input::default(), bounds, 0., |f| { BTN_TONAL.render(f, bounds, &mut state); });
    assert!(c.iter().any(|d| matches!(d, DrawCommand::RoundedRect { color: Color(0xE8, 0xDE, 0xF8, 255), .. })));
    assert!(c.iter().any(|d| matches!(d, DrawCommand::Text { text, color: Color(0x1D, 0x19, 0x2B, 255), .. } if text.as_ref() == "Tonal")));

    // text: transparent fill + no shadow + primary text
    let c = ui.frame(Input::default(), bounds, 0., |f| { BTN_TEXT.render(f, bounds, &mut state); });
    assert!(!c.iter().any(|d| matches!(d, DrawCommand::RoundedRect { bounds: r, .. } if (r.height - 40.0).abs() < 0.01)));
    assert!(!c.iter().any(|d| matches!(d, DrawCommand::Shadow { .. })));
    assert!(c.iter().any(|d| matches!(d, DrawCommand::Text { text, color: Color(0x67, 0x50, 0xA4, 255), .. } if text.as_ref() == "TextBtn")));

    // elevated: surfaceContainerLow (#F7F2FA) fill + level-1 shadow (blur 3.0) + primary text
    let c = ui.frame(Input::default(), bounds, 0., |f| { BTN_ELEVATED.render(f, bounds, &mut state); });
    assert!(c.iter().any(|d| matches!(d, DrawCommand::RoundedRect { color: Color(0xF7, 0xF2, 0xFA, 255), .. })));
    assert!(c.iter().any(|d| matches!(d, DrawCommand::Shadow { blur, .. } if (*blur - 3.0).abs() < 0.01)));
    assert!(c.iter().any(|d| matches!(d, DrawCommand::Text { text, color: Color(0x67, 0x50, 0xA4, 255), .. } if text.as_ref() == "Elevated")));

    // explicit background overrides RULE_BUTTON_VARIANT
    let c = ui.frame(Input::default(), bounds, 0., |f| { BTN_CUSTOM_BG.render(f, bounds, &mut state); });
    assert!(c.iter().any(|d| matches!(d, DrawCommand::RoundedRect { bounds: r, color: Color(12, 34, 56, 255), .. } if *r == bounds)));
    assert!(!c.iter().any(|d| matches!(d, DrawCommand::RoundedRect { color: Color(0x67, 0x50, 0xA4, 255), .. })));
}

#[test]
fn material3_chip_rendering_filter_selection_close_icon_and_density() {
    use bhoo::android_xml::{AndroidTheme, RULE_CHIP_STYLE};

    static FILTER_RULE: &[RelativeRule] = &[RelativeRule { kind: RULE_CHIP_STYLE, anchor: "filter|0" }];
    static INPUT_CLOSE_RULE: &[RelativeRule] = &[RelativeRule { kind: RULE_CHIP_STYLE, anchor: "input|1" }];
    static FILTER_CHIP: Node = Node {
        kind: Kind::Button,
        text: "Design",
        width: Size::Dp(120.),
        relative_rules: FILTER_RULE,
        ..button("chip_filter", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static INPUT_CHIP: Node = Node {
        kind: Kind::Button,
        text: "Tag",
        width: Size::Dp(120.),
        radius: 16.0,
        relative_rules: INPUT_CLOSE_RULE,
        ..button("chip_input", Visibility::Visible, Size::Dp(48.), 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 120., height: 48. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Unselected filter chip: 32dp tall centered (y=8..40), 8dp radius, 1dp outline (#79747E), no secondaryContainer fill, no check icon
    let unselected = ui.frame(Input::default(), bounds, 0., |f| { FILTER_CHIP.render(f, bounds, &mut state); });
    assert!(unselected.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, color: Color(0x79, 0x74, 0x7E, 255) }
            if (r.y - 8.0).abs() < 0.01 && (r.height - 32.0).abs() < 0.01 && (*radius - 8.0).abs() < 0.01 && (*width - 1.0).abs() < 0.01
    )));
    assert!(!unselected.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0xE8, 0xDE, 0xF8, 255), .. })));
    assert!(!unselected.iter().any(|c| matches!(c, DrawCommand::Shape { .. })));

    // 2. Click filter chip -> checked! Paints secondaryContainer (#E8DEF8) fill + Check icon + onSecondaryContainer text, no outline border
    let pt = [60., 24.];
    ui.frame(Input { pointer: Some(pt), pressed: true, ..Input::default() }, bounds, 0., |f| { FILTER_CHIP.render(f, bounds, &mut state); });
    let selected = ui.frame(Input { pointer: Some(pt), released: true, ..Input::default() }, bounds, 0., |f| { FILTER_CHIP.render(f, bounds, &mut state); });
    assert_eq!(state.checked.get("chip_filter"), Some(&true));
    assert!(selected.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE8, 0xDE, 0xF8, 255) }
            if (r.y - 8.0).abs() < 0.01 && (r.height - 32.0).abs() < 0.01 && (*radius - 8.0).abs() < 0.01
    )));
    assert!(selected.iter().any(|c| matches!(c, DrawCommand::Shape { bounds: r, .. } if (r.width - 18.0).abs() < 0.01 && r.x < 30.0)));
    assert!(selected.iter().any(|c| matches!(c, DrawCommand::Text { text, color: Color(0x1D, 0x19, 0x2B, 255), .. } if text.as_ref() == "Design")));

    // 3. Input chip with close_visible=1 and custom radius=16: renders trailing Close icon on right in LTR, left in RTL
    let input_ltr = ui.frame(Input::default(), bounds, 0., |f| { INPUT_CHIP.render(f, bounds, &mut state); });
    assert!(input_ltr.iter().any(|c| matches!(c, DrawCommand::Outline { radius, .. } if (*radius - 16.0).abs() < 0.01)));
    assert!(input_ltr.iter().any(|c| matches!(c, DrawCommand::Shape { bounds: r, .. } if (r.width - 18.0).abs() < 0.01 && r.x > 80.0)));
    state.rtl = true;
    let input_rtl = ui.frame(Input::default(), bounds, 0., |f| { INPUT_CHIP.render(f, bounds, &mut state); });
    assert!(input_rtl.iter().any(|c| matches!(c, DrawCommand::Shape { bounds: r, .. } if (r.width - 18.0).abs() < 0.01 && r.x < 20.0)));

    // 4. Density 2.0 scales visual chip to 64px centered inside 96px touch target with 16px corner radius
    state.rtl = false;
    state.density = 2.0;
    state.checked.insert("chip_filter".into(), false);
    let bounds_2x = Rect { x: 0., y: 0., width: 240., height: 96. };
    let cmds_2x = ui.frame(Input::default(), bounds_2x, 0., |f| { FILTER_CHIP.render(f, bounds_2x, &mut state); });
    assert!(cmds_2x.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, .. }
            if (r.y - 16.0).abs() < 0.01 && (r.height - 64.0).abs() < 0.01 && (*radius - 16.0).abs() < 0.01 && (*width - 2.0).abs() < 0.01
    )));

    // 5. Imported chip carrying default surfaceContainerLow background still switches to secondaryContainer when checked
    static IMPORTED_CHIP: Node = Node {
        background: Some(Color(0xF7, 0xF2, 0xFA, 255)),
        checked: true,
        ..FILTER_CHIP
    };
    state.density = 1.0;
    state.checked.remove("chip_filter");
    let imported_sel = ui.frame(Input::default(), bounds, 0., |f| { IMPORTED_CHIP.render(f, bounds, &mut state); });
    assert!(imported_sel.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0xE8, 0xDE, 0xF8, 255), .. })));
}

#[test]
fn stateful_drawable_layers_evaluate_live_state_mask_and_geometry() {
    use bhoo::android_xml::RULE_STATEFUL_DRAWABLE_LAYER;

    static LAYERS: &[RelativeRule] = &[
        // disabled (req=0, forbid=1) -> gray
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "0,1|0,0,0,0|4|120,120,120,255|" },
        // pressed (req=2, forbid=0) -> inset 2dp red fill + 2dp dark-red stroke
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "2,0|2,2,2,2|6|220,20,20,255|2,140,0,0,255" },
        // focused (req=4, forbid=0) -> oval purple
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "4,0|0,0,0,0|-1|130,40,200,255|" },
        // selected (req=8, forbid=0) -> asymmetric 4-corner teal
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "8,0|0,0,0,0|10,0,10,0|0,160,150,255|" },
        // activated (req=16, forbid=0) -> amber
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "16,0|0,0,0,0|4|230,170,0,255|" },
        // checked (req=64, forbid=0) -> green with stroke
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "64,0|0,0,0,0|8|20,180,60,255|1,10,90,30,255" },
        // hovered (req=32, forbid=0) -> cyan
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "32,0|0,0,0,0|4|0,180,220,255|" },
        // default (req=0, forbid=0) -> 2 stacked layers (outer blue + 4dp inset light-blue)
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "0,0|0,0,0,0|8|30,60,200,255|" },
        RelativeRule { kind: RULE_STATEFUL_DRAWABLE_LAYER, anchor: "0,0|4,4,4,4|4|100,150,255,255|" },
    ];
    static BTN: Node = Node {
        kind: Kind::Button,
        text: "Stateful",
        relative_rules: LAYERS,
        ..button("stateful_btn", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static BTN_DISABLED: Node = Node { enabled: false, ..BTN };

    let bounds = Rect { x: 0., y: 0., width: 160., height: 48. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // 1. Default enabled -> both consecutive (0,0) layers paint, none of the specialized states
    let def = ui.frame(Input::default(), bounds, 0., |f| { BTN.render(f, bounds, &mut state); });
    assert!(def.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(30, 60, 200, 255) } if *r == bounds && (*radius - 8.0).abs() < 0.01)));
    assert!(def.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(100, 150, 255, 255) } if *r == Rect { x: 4., y: 4., width: 152., height: 40. } && (*radius - 4.0).abs() < 0.01)));
    assert!(!def.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(220, 20, 20, 255), .. })));

    // 2. Hovered -> cyan (and default does NOT overwrite)
    let hov = ui.frame(Input { pointer: Some([50., 24.]), ..Input::default() }, bounds, 0., |f| { BTN.render(f, bounds, &mut state); });
    assert!(hov.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0, 180, 220, 255), .. })));
    assert!(!hov.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(30, 60, 200, 255), .. })));

    // 3. Pressed -> inset red fill + dark-red stroke
    let prs = ui.frame(Input { pointer: Some([50., 24.]), pressed: true, ..Input::default() }, bounds, 0., |f| { BTN.render(f, bounds, &mut state); });
    assert!(prs.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, color: Color(220, 20, 20, 255) } if *r == Rect { x: 2., y: 2., width: 156., height: 44. } && (*radius - 6.0).abs() < 0.01)));
    assert!(prs.iter().any(|c| matches!(c, DrawCommand::Outline { width, color: Color(140, 0, 0, 255), .. } if (*width - 2.0).abs() < 0.01)));
    assert!(!prs.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(30, 60, 200, 255), .. })));

    // 4. Release -> now focused (no longer pressed) -> oval purple Ellipse!
    let foc = ui.frame(Input { pointer: Some([50., 24.]), released: true, ..Input::default() }, bounds, 0., |f| { BTN.render(f, bounds, &mut state); });
    assert!(foc.iter().any(|c| matches!(c, DrawCommand::Ellipse { bounds: r, color: Color(130, 40, 200, 255) } if *r == bounds)));
    assert!(!foc.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(220, 20, 20, 255), .. })));

    // Clear focus for remaining state checks
    let mut ui = Ui::default();

    // 5. Selected -> asymmetric 4-corner teal
    state.view_selected.insert("stateful_btn".into(), true);
    let sel = ui.frame(Input::default(), bounds, 0., |f| { BTN.render(f, bounds, &mut state); });
    assert!(sel.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0, 160, 150, 255), .. })));
    state.view_selected.clear();

    // 6. Activated -> amber
    state.view_activated.insert("stateful_btn".into(), true);
    let act = ui.frame(Input::default(), bounds, 0., |f| { BTN.render(f, bounds, &mut state); });
    assert!(act.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(230, 170, 0, 255), .. })));
    state.view_activated.clear();

    // 7. Checked -> green with 1dp stroke
    state.checked.insert("stateful_btn".into(), true);
    let chk = ui.frame(Input::default(), bounds, 0., |f| { BTN.render(f, bounds, &mut state); });
    assert!(chk.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(20, 180, 60, 255), .. })));
    assert!(chk.iter().any(|c| matches!(c, DrawCommand::Outline { width, color: Color(10, 90, 30, 255), .. } if (*width - 1.0).abs() < 0.01)));
    state.checked.clear();

    // 8. Disabled -> gray
    let dis = ui.frame(Input::default(), bounds, 0., |f| { BTN_DISABLED.render(f, bounds, &mut state); });
    assert!(dis.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(120, 120, 120, 255), .. })));
}

#[test]
fn wave5b_material_button_6dp_inset_android_line_height_and_density_elevation() {
    use bhoo::android_xml::{AndroidTheme, RULE_ELEVATION};

    // 1. Line height: Bhoo keeps 1.5x; Material themes use 1.34x -> at density=1.125,
    //    header (~69.2px) + description (~60.1px) places confirm near Waydroid's ~131px.
    let screen_bounds = Rect { x: 0., y: 0., width: 720., height: 1226. };
    let mut ui = Ui::default();

    let mut bhoo_state = LayoutState::default();
    bhoo_state.theme_style = AndroidTheme::Bhoo;
    bhoo_state.density = 1.0;
    ui.frame(Input::default(), screen_bounds, 0., |f| {
        generated::LAYOUT.render(f, screen_bounds, &mut bhoo_state);
    });
    let bhoo_confirm = ui.semantics().iter().find(|n| n.label.as_ref() == "Continue").unwrap().bounds;
    // Bhoo 1.5x: (22*1.5 + 32) + (16*1.5 + 32) = 65 + 56 = 121
    assert!((bhoo_confirm.y - 121.0).abs() < 0.05, "Bhoo theme must preserve 1.5x line height: y={}", bhoo_confirm.y);

    for theme in [
        AndroidTheme::MaterialLight,
        AndroidTheme::MaterialDark,
        AndroidTheme::Material3Light,
        AndroidTheme::Material3Dark,
    ] {
        let mut st = LayoutState::default();
        st.theme_style = theme;
        st.density = 1.0;
        ui.frame(Input::default(), screen_bounds, 0., |f| {
            generated::LAYOUT.render(f, screen_bounds, &mut st);
        });
        let c = ui.semantics().iter().find(|n| n.label.as_ref() == "Continue").unwrap().bounds;
        // Android 1.34x: (22*1.34 + 32) + (16*1.34 + 32) = 61.48 + 53.44 = 114.92
        assert!((c.y - 114.92).abs() < 0.05, "{theme:?} must use 1.34x line height: y={}", c.y);
    }

    // 2. Waydroid geometry at density = 1.125 under MaterialLight:
    //    6dp vertical inset (6.75px) + 4dp horizontal inset (4.5px) + subtle non-overflowing 2dp shadow.
    let mut ml_state = LayoutState::default();
    ml_state.theme_style = AndroidTheme::MaterialLight;
    ml_state.density = 1.125;
    let ml_cmds = ui.frame(Input::default(), screen_bounds, 0., |f| {
        generated::LAYOUT.render(f, screen_bounds, &mut ml_state);
    });
    let confirm = ui.semantics().iter().find(|n| n.label.as_ref() == "Continue").unwrap().bounds;
    assert!((confirm.y - 129.285).abs() < 0.1, "confirm.y at density=1.125 should match ~129-131px: got {}", confirm.y);
    assert!((confirm.height - 54.0).abs() < 0.01);
    let expected_visual = Rect {
        x: 4.5,
        y: confirm.y + 6.75,
        width: 711.0,
        height: 40.5,
    };
    assert!(ml_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xD6, 0xD7, 0xD7, 255) }
            if (r.x - expected_visual.x).abs() < 0.01
                && (r.y - expected_visual.y).abs() < 0.01
                && (r.width - expected_visual.width).abs() < 0.01
                && (r.height - expected_visual.height).abs() < 0.01
                && (*radius - 2.25).abs() < 0.01
    )), "MaterialLight button fill must apply 4dp H / 6dp V inset at density=1.125");
    let btn_shadows: Vec<_> = ml_cmds.iter().filter_map(|c| match c {
        DrawCommand::Shadow { bounds: r, offset, blur, spread, color, .. }
            if (r.y - expected_visual.y).abs() < 0.01 => Some((*offset, *blur, *spread, *color)),
        _ => None,
    }).collect();
    assert_eq!(btn_shadows.len(), 2, "MaterialLight 2dp button should emit ambient + key shadows");
    for (offset, blur, spread, color) in &btn_shadows {
        assert!(*spread + *blur < 4.5, "button shadow side extent must fit inside 4dp (4.5px) horizontal inset: blur={blur}, spread={spread}");
        assert!(offset[1] > 0.0, "button shadow y-offset must be positive");
        assert!(color.3 <= 24, "each 2dp button shadow layer must remain subtle: alpha={}", color.3);
    }

    // 3. MaterialLight default button padding [12, 10, 12, 10] affects wrap_content vertical sizing for tall/multiline labels.
    static TALL_BTN_WRAP: Node = Node {
        kind: Kind::Linear,
        children: &[Node {
            kind: Kind::Button,
            text: "Line one\nLine two\nLine three",
            text_size: 16.0,
            ..button("tall_btn", Visibility::Visible, Size::Wrap, 0.)
        }],
        ..button("tall_btn_root", Visibility::Visible, Size::Match, 0.)
    };
    let wrap_bounds = Rect { x: 0., y: 0., width: 320., height: 240. };
    let mut pad_state = LayoutState::default();
    pad_state.theme_style = AndroidTheme::MaterialLight;
    ui.frame(Input::default(), wrap_bounds, 0., |f| { TALL_BTN_WRAP.render(f, wrap_bounds, &mut pad_state); });
    let tall_h = ui.semantics().iter().find(|n| n.label.as_ref() == "Line one\nLine two\nLine three").unwrap().bounds.height;
    // 3 lines * (14sp * 1.34) + 10dp top + 10dp bottom = 56.28 + 20.0 = 76.28
    assert!((tall_h - (3.0 * 14.0 * 1.34 + 20.0)).abs() < 0.05, "MaterialLight wrap button height must include 10dp+10dp vertical padding: got {tall_h}");

    // 4. RULE_ELEVATION thresholds use unscaled dp and scale offset/blur/spread by density.
    static ELEV_1: &[RelativeRule] = &[RelativeRule { kind: RULE_ELEVATION, anchor: "1" }];
    static ELEV_2: &[RelativeRule] = &[RelativeRule { kind: RULE_ELEVATION, anchor: "2" }];
    static VIEW_E1: Node = Node {
        kind: Kind::View,
        background: Some(Color(240, 240, 240, 255)),
        radius: 4.0,
        relative_rules: ELEV_1,
        ..button("e1", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static VIEW_E2: Node = Node {
        relative_rules: ELEV_2,
        ..VIEW_E1
    };
    let e_bounds = Rect { x: 0., y: 0., width: 200., height: 120. };
    let mut s1 = LayoutState::default();
    s1.density = 1.0;
    let c1 = ui.frame(Input::default(), e_bounds, 0., |f| { VIEW_E1.render(f, e_bounds, &mut s1); });
    let sh1: Vec<_> = c1.iter().filter_map(|c| match c {
        DrawCommand::Shadow { offset, blur, spread, .. } => Some((*offset, *blur, *spread)),
        _ => None,
    }).collect();
    assert_eq!(sh1.len(), 3);
    assert!((sh1[2].1 - 3.0).abs() < 0.01, "1dp elevation at density=1 must use level-1 blur 3.0");

    let mut s2 = LayoutState::default();
    s2.density = 2.0;
    let c2 = ui.frame(Input::default(), e_bounds, 0., |f| { VIEW_E1.render(f, e_bounds, &mut s2); });
    let sh2: Vec<_> = c2.iter().filter_map(|c| match c {
        DrawCommand::Shadow { offset, blur, spread, .. } => Some((*offset, *blur, *spread)),
        _ => None,
    }).collect();
    assert_eq!(sh2.len(), 3);
    for (a, b) in sh1.iter().zip(sh2.iter()) {
        assert!((b.0[0] - a.0[0] * 2.0).abs() < 0.01 && (b.0[1] - a.0[1] * 2.0).abs() < 0.01, "shadow offset must scale 2x without bumping level: {a:?} vs {b:?}");
        assert!((b.1 - a.1 * 2.0).abs() < 0.01, "shadow blur must scale 2x without bumping level: {a:?} vs {b:?}");
        assert!((b.2 - a.2 * 2.0).abs() < 0.01, "shadow spread must scale 2x without bumping level: {a:?} vs {b:?}");
    }

    let c2_lvl2 = ui.frame(Input::default(), e_bounds, 0., |f| { VIEW_E2.render(f, e_bounds, &mut s2); });
    let sh2_lvl2: Vec<_> = c2_lvl2.iter().filter_map(|c| match c {
        DrawCommand::Shadow { offset, blur, spread, .. } => Some((*offset, *blur, *spread)),
        _ => None,
    }).collect();
    assert_eq!(sh2_lvl2.len(), 3);
    assert!((sh2_lvl2[2].1 - 10.0).abs() < 0.01, "2dp elevation at density=2 must use level-2 blur 5.0*2=10.0, got {}", sh2_lvl2[2].1);
    assert!((sh2_lvl2[0].0[1] - 6.0).abs() < 0.01 && (sh2_lvl2[0].2 - -4.0).abs() < 0.01);

    // 5. MaterialLight Toolbar default 4dp elevation scales shadow geometry by density (not unscaled 4dp * density into elevation_dp).
    static TB: Node = Node { kind: Kind::Toolbar, text: "Title", ..button("tb_elev", Visibility::Visible, Size::Dp(56.), 0.) };
    let mut tb1 = LayoutState::default();
    tb1.theme_style = AndroidTheme::MaterialLight;
    tb1.density = 1.0;
    let tbc1 = ui.frame(Input::default(), e_bounds, 0., |f| { TB.render(f, e_bounds, &mut tb1); });
    let tb_sh1: Vec<_> = tbc1.iter().filter_map(|c| match c {
        DrawCommand::Shadow { offset, blur, spread, .. } => Some((*offset, *blur, *spread)),
        _ => None,
    }).collect();
    let mut tb2 = LayoutState::default();
    tb2.theme_style = AndroidTheme::MaterialLight;
    tb2.density = 2.0;
    let tbc2 = ui.frame(Input::default(), e_bounds, 0., |f| { TB.render(f, e_bounds, &mut tb2); });
    let tb_sh2: Vec<_> = tbc2.iter().filter_map(|c| match c {
        DrawCommand::Shadow { offset, blur, spread, .. } => Some((*offset, *blur, *spread)),
        _ => None,
    }).collect();
    assert_eq!(tb_sh1.len(), 3);
    assert_eq!(tb_sh2.len(), 3);
    for (a, b) in tb_sh1.iter().zip(tb_sh2.iter()) {
        assert!((b.0[1] - a.0[1] * 2.0).abs() < 0.01 && (b.1 - a.1 * 2.0).abs() < 0.01 && (b.2 - a.2 * 2.0).abs() < 0.01);
    }
}

#[test]
fn constraint_layout_flow_horizontal_vertical_aligned_none_gone_and_dependents() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_END_TO_END, RULE_CONSTRAINT_START_TO_START,
        RULE_CONSTRAINT_TOP_TO_BOTTOM, RULE_CONSTRAINT_TOP_TO_TOP, RULE_FLOW,
    };

    // 1. Horizontal chain wrap with width overflow, h_gap=10, v_gap=8, Gone item skipped,
    //    and a dependent view constrained top_toBottomOf="@id/flow".
    static FLOW_CHAIN_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "0|chain|10|8|0|a,b,c,d" },
    ];
    static BELOW_FLOW_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "flow" },
    ];
    static FLOW_CHAIN_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node {
                kind: Kind::View,
                width: Size::Dp(200.),
                relative_rules: FLOW_CHAIN_RULES,
                ..button("flow", Visibility::Visible, Size::Wrap, 0.)
            },
            Node { width: Size::Dp(80.), text: "A", ..button("a", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(90.), text: "B", ..button("b", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(70.), text: "C", ..button("c", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(60.), text: "D", ..button("d", Visibility::Visible, Size::Dp(30.), 0.) },
            Node {
                width: Size::Dp(120.),
                text: "Below",
                margin: [0., 6., 0., 0.],
                relative_rules: BELOW_FLOW_RULES,
                ..button("below", Visibility::Visible, Size::Dp(24.), 0.)
            },
        ],
        ..button("cl_flow_root", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 240. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { FLOW_CHAIN_ROOT.render(f, bounds, &mut state); });

    let rect_of = |ui: &Ui, label: &str| {
        ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds
    };
    // Flow helper is non-visual (only 5 semantics: A, B, C, D, Below).
    assert_eq!(ui.semantics().len(), 5);
    // Row 0: A (0..80), gap 10, B (90..180) <= 200. C (70) would make 180+10+70=260 > 200 -> Row 1!
    // Row 1 (y = 30 + 8 = 38): C (0..70), gap 10, D (80..140). Total Flow height = 68.
    assert_eq!(rect_of(&ui, "A"), Rect { x: 0., y: 0., width: 80., height: 30. });
    assert_eq!(rect_of(&ui, "B"), Rect { x: 90., y: 0., width: 90., height: 30. });
    assert_eq!(rect_of(&ui, "C"), Rect { x: 0., y: 38., width: 70., height: 30. });
    assert_eq!(rect_of(&ui, "D"), Rect { x: 80., y: 38., width: 60., height: 30. });
    // Dependent below Flow: y = 68 + 6 = 74!
    assert_eq!(rect_of(&ui, "Below"), Rect { x: 0., y: 74., width: 120., height: 24. });

    // Hiding B (Gone) lets A (80) + 10 + C (70) = 160 fit on Row 0, D wraps to Row 1 (160+10+60=230>200)
    state.visibility.insert("b".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| { FLOW_CHAIN_ROOT.render(f, bounds, &mut state); });
    assert_eq!(ui.semantics().len(), 4);
    assert_eq!(rect_of(&ui, "A"), Rect { x: 0., y: 0., width: 80., height: 30. });
    assert_eq!(rect_of(&ui, "C"), Rect { x: 90., y: 0., width: 70., height: 30. });
    assert_eq!(rect_of(&ui, "D"), Rect { x: 0., y: 38., width: 60., height: 30. });
    state.visibility.clear();

    // 2. Horizontal wrap=aligned with max_wrap=2 aligns columns across rows even with differing item widths
    static FLOW_ALIGNED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "0|aligned|10|6|2|a,b,c,d" },
    ];
    static FLOW_ALIGNED_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node {
                kind: Kind::View,
                width: Size::Dp(0.),
                relative_rules: FLOW_ALIGNED_RULES,
                ..button("flow_al", Visibility::Visible, Size::Wrap, 0.)
            },
            Node { width: Size::Dp(60.), text: "A", ..button("a", Visibility::Visible, Size::Dp(24.), 0.) },
            Node { width: Size::Dp(70.), text: "B", ..button("b", Visibility::Visible, Size::Dp(24.), 0.) },
            Node { width: Size::Dp(100.), text: "C", ..button("c", Visibility::Visible, Size::Dp(24.), 0.) },
            Node { width: Size::Dp(50.), text: "D", ..button("d", Visibility::Visible, Size::Dp(24.), 0.) },
        ],
        ..button("cl_flow_aligned", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { FLOW_ALIGNED_ROOT.render(f, bounds, &mut state); });
    // Col 0 max width = max(60, 100) = 100 -> Col 1 starts at x = 100 + 10 = 110 for BOTH rows!
    assert_eq!(rect_of(&ui, "A").x, 0.0);
    assert_eq!(rect_of(&ui, "C").x, 0.0);
    assert_eq!(rect_of(&ui, "B").x, 110.0);
    assert_eq!(rect_of(&ui, "D").x, 110.0);
    assert_eq!(rect_of(&ui, "C").y, 30.0);
    assert_eq!(rect_of(&ui, "D").y, 30.0);

    // 3. Vertical Flow (orient=1) wraps into columns at max_wrap=2 with h_gap=12, v_gap=6
    static FLOW_VERT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "1|chain|12|6|2|a,b,c" },
    ];
    static FLOW_VERT_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node {
                kind: Kind::View,
                width: Size::Wrap,
                relative_rules: FLOW_VERT_RULES,
                ..button("flow_v", Visibility::Visible, Size::Dp(120.), 0.)
            },
            Node { width: Size::Dp(50.), text: "A", ..button("a", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(50.), text: "B", ..button("b", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(50.), text: "C", ..button("c", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_flow_vert", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { FLOW_VERT_ROOT.render(f, bounds, &mut state); });
    assert_eq!(rect_of(&ui, "A"), Rect { x: 0., y: 0., width: 50., height: 30. });
    assert_eq!(rect_of(&ui, "B"), Rect { x: 0., y: 36., width: 50., height: 30. });
    assert_eq!(rect_of(&ui, "C"), Rect { x: 62., y: 0., width: 50., height: 30. });

    // 4. wrap=none keeps all items on a single row even if wider than Flow width
    static FLOW_NONE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "0|none|10|8|1|a,b,c" },
    ];
    static FLOW_NONE_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node {
                kind: Kind::View,
                width: Size::Dp(100.),
                relative_rules: FLOW_NONE_RULES,
                ..button("flow_n", Visibility::Visible, Size::Wrap, 0.)
            },
            Node { width: Size::Dp(60.), text: "A", ..button("a", Visibility::Visible, Size::Dp(24.), 0.) },
            Node { width: Size::Dp(60.), text: "B", ..button("b", Visibility::Visible, Size::Dp(24.), 0.) },
            Node { width: Size::Dp(60.), text: "C", ..button("c", Visibility::Visible, Size::Dp(24.), 0.) },
        ],
        ..button("cl_flow_none", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), bounds, 0., |f| { FLOW_NONE_ROOT.render(f, bounds, &mut state); });
    assert_eq!(rect_of(&ui, "A").y, 0.0);
    assert_eq!(rect_of(&ui, "B"), Rect { x: 70., y: 0., width: 60., height: 24. });
    assert_eq!(rect_of(&ui, "C"), Rect { x: 140., y: 0., width: 60., height: 24. });
}

#[test]
fn constraint_layout_circle_positioning_cardinals_diagonals_chained_and_density() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, RULE_CONSTRAINT_CIRCLE, RULE_CONSTRAINT_END_TO_END,
        RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP,
    };

    static HUB_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
    ];
    static R0: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "hub,60,0" }];
    static R90: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "hub,60,90" }];
    static R180: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "hub,60,180" }];
    static R270: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "hub,60,270" }];
    static R45: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "hub,100,45" }];
    static CHAINED: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "n90,30,180" }];

    static CIRCLE_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(40.), text: "Hub", relative_rules: HUB_RULES, ..button("hub", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { width: Size::Dp(20.), text: "N0", relative_rules: R0, ..button("n0", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(20.), text: "N90", relative_rules: R90, ..button("n90", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(20.), text: "N180", relative_rules: R180, ..button("n180", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(20.), text: "N270", relative_rules: R270, ..button("n270", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(20.), text: "N45", relative_rules: R45, ..button("n45", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(20.), text: "Moon", relative_rules: CHAINED, ..button("moon", Visibility::Visible, Size::Dp(20.), 0.) },
        ],
        ..button("cl_circle", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 300., height: 300. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), bounds, 0., |f| { CIRCLE_ROOT.render(f, bounds, &mut state); });

    let center = |ui: &Ui, label: &str| {
        let r = ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds;
        (r.x + r.width * 0.5, r.y + r.height * 0.5)
    };
    let assert_near = |got: (f32, f32), exp: (f32, f32)| {
        assert!((got.0 - exp.0).abs() < 0.05 && (got.1 - exp.1).abs() < 0.05, "got {got:?}, expected {exp:?}");
    };
    // Hub center is at (150, 150)
    assert_near(center(&ui, "Hub"), (150., 150.));
    assert_near(center(&ui, "N0"), (150., 90.));    // 0° = 12 o'clock (top)
    assert_near(center(&ui, "N90"), (210., 150.));  // 90° = 3 o'clock (right)
    assert_near(center(&ui, "N180"), (150., 210.)); // 180° = 6 o'clock (bottom)
    assert_near(center(&ui, "N270"), (90., 150.));  // 270° = 9 o'clock (left)
    let diag = 100.0 * std::f32::consts::FRAC_1_SQRT_2;
    assert_near(center(&ui, "N45"), (150. + diag, 150. - diag));
    // Chained Moon: 30dp below N90 (210, 150) -> (210, 180)
    assert_near(center(&ui, "Moon"), (210., 180.));

    // Density = 1.5 scales radius 60dp -> 90px
    state.density = 1.5;
    let bounds_hi = Rect { x: 0., y: 0., width: 450., height: 450. };
    ui.frame(Input::default(), bounds_hi, 0., |f| { CIRCLE_ROOT.render(f, bounds_hi, &mut state); });
    assert_near(center(&ui, "Hub"), (225., 225.));
    assert_near(center(&ui, "N0"), (225., 135.));
    assert_near(center(&ui, "N90"), (315., 225.));
}

#[test]
fn clip_to_outline_clips_card_banner_view_children_and_oval_bounds() {
    use bhoo::android_xml::{AndroidTheme, RULE_CLIP_TO_OUTLINE};

    // 1. Outlined MaterialCardView with 16dp radius + RULE_CLIP_TO_OUTLINE + full-bleed banner
    static CARD_RULES: &[RelativeRule] = &[
        RelativeRule { kind: 57, anchor: "Outlined" },
        RelativeRule { kind: 66, anchor: "16" },
        RelativeRule { kind: RULE_CLIP_TO_OUTLINE, anchor: "true" },
    ];
    static CARD_NODE: Node = Node {
        kind: Kind::MaterialCard,
        relative_rules: CARD_RULES,
        children: &[Node {
            kind: Kind::View,
            background: Some(Color(0xEA, 0xDD, 0xFF, 255)),
            ..button("banner", Visibility::Visible, Size::Dp(48.), 0.)
        }],
        ..button("clipped_card", Visibility::Visible, Size::Dp(120.), 0.)
    };
    let bounds = Rect { x: 0., y: 0., width: 240., height: 120. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { CARD_NODE.render(f, bounds, &mut state); });

    // Banner is clipped against the 16dp card outline via PushClip(banner_rect) + RoundedRect(card_bounds, radius=16) + PopClip
    let push_count = cmds.iter().filter(|c| matches!(c, DrawCommand::PushClip(_))).count();
    let pop_count = cmds.iter().filter(|c| matches!(c, DrawCommand::PopClip)).count();
    assert_eq!(push_count, pop_count, "clip stack must stay balanced");
    let banner_idx = cmds.iter().position(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xEA, 0xDD, 0xFF, 255) }
            if *r == bounds && (*radius - 16.0).abs() < 0.01
    ));
    assert!(banner_idx.is_some(), "full-bleed banner inside clipToOutline card must adopt the card's 16dp rounded outline");
    // Outlined card stroke must be repainted over the clipped banner
    let last_outline = cmds.iter().rposition(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, .. } if *r == bounds && (*radius - 16.0).abs() < 0.01
    ));
    assert!(last_outline.is_some_and(|oi| oi > banner_idx.unwrap()), "card stroke must render cleanly over clipped banner");

    // 2. General oval Frame with clip_children_disabled=true still clips child rect into Ellipse inside PushClip(bounds)
    static OVAL_CLIP_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CLIP_TO_OUTLINE, anchor: "true" }];
    static OVAL_FRAME: Node = Node {
        kind: Kind::Frame,
        radius: -1.0,
        clip_children_disabled: true,
        relative_rules: OVAL_CLIP_RULES,
        children: &[Node {
            kind: Kind::View,
            width: Size::Dp(160.),
            background: Some(Color(30, 140, 220, 255)),
            ..button("child_bar", Visibility::Visible, Size::Dp(40.), 0.)
        }],
        ..button("oval_clip", Visibility::Visible, Size::Dp(80.), 0.)
    };
    let oval_bounds = Rect { x: 10., y: 10., width: 80., height: 80. };
    let oval_cmds = ui.frame(Input::default(), oval_bounds, 0., |f| { OVAL_FRAME.render(f, oval_bounds, &mut state); });
    assert_eq!(
        oval_cmds.iter().filter(|c| matches!(c, DrawCommand::PushClip(_))).count(),
        oval_cmds.iter().filter(|c| matches!(c, DrawCommand::PopClip)).count(),
    );
    assert!(oval_cmds.iter().any(|c| matches!(c, DrawCommand::PushClip(r) if *r == oval_bounds)));
    assert!(oval_cmds.iter().any(|c| matches!(c, DrawCommand::Ellipse { bounds: r, color: Color(30, 140, 220, 255) } if *r == oval_bounds)));
}

#[test]
fn material3_and_aosp_form_control_visual_parity_switch_checkbox_radio_spinner() {
    use bhoo::android_xml::AndroidTheme;

    static SPINNER_ITEMS: &[RelativeRule] = &[
        RelativeRule { kind: 82, anchor: "Standard" },
        RelativeRule { kind: 82, anchor: "Compact" },
    ];
    static SW_OFF: Node = Node { kind: Kind::Switch, checked: false, text: "Off", ..button("sw_off", Visibility::Visible, Size::Dp(48.), 0.) };
    static SW_ON: Node = Node { kind: Kind::Switch, checked: true, text: "On", ..button("sw_on", Visibility::Visible, Size::Dp(48.), 0.) };
    static CB_OFF: Node = Node { kind: Kind::CheckBox, checked: false, text: "CheckOff", ..button("cb_off", Visibility::Visible, Size::Dp(48.), 0.) };
    static CB_ON: Node = Node { kind: Kind::CheckBox, checked: true, text: "CheckOn", ..button("cb_on", Visibility::Visible, Size::Dp(48.), 0.) };
    static RB_OFF: Node = Node { kind: Kind::RadioButton, checked: false, text: "RadioOff", ..button("rb_off", Visibility::Visible, Size::Dp(48.), 0.) };
    static RB_ON: Node = Node { kind: Kind::RadioButton, checked: true, text: "RadioOn", ..button("rb_on", Visibility::Visible, Size::Dp(48.), 0.) };
    static SP: Node = Node { kind: Kind::Spinner, relative_rules: SPINNER_ITEMS, ..button("sp_parity", Visibility::Visible, Size::Dp(48.), 0.) };

    let bounds = Rect { x: 0., y: 0., width: 240., height: 48. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // --- 1. Material3Light Switch: 52x32 pill, 2dp outline unchecked, 16dp unchecked / 24dp checked thumb ---
    state.theme_style = AndroidTheme::Material3Light;
    let sw_off_cmds = ui.frame(Input::default(), bounds, 0., |f| { SW_OFF.render(f, bounds, &mut state); });
    assert!(sw_off_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE6, 0xE0, 0xE9, 255) }
            if (r.width - 52.0).abs() < 0.01 && (r.height - 32.0).abs() < 0.01 && (*radius - 16.0).abs() < 0.01
    )), "M3 unchecked Switch must draw 52x32 surfaceContainerHighest pill track");
    assert!(sw_off_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, color: Color(0x79, 0x74, 0x7E, 255) }
            if (r.width - 52.0).abs() < 0.01 && (r.height - 32.0).abs() < 0.01 && (*radius - 16.0).abs() < 0.01 && (*width - 2.0).abs() < 0.01
    )), "M3 unchecked Switch must draw 2dp outline border");
    assert!(sw_off_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x79, 0x74, 0x7E, 255) }
            if (r.width - 16.0).abs() < 0.01 && (r.height - 16.0).abs() < 0.01 && (*radius - 8.0).abs() < 0.01
    )), "M3 unchecked Switch must draw 16dp outline thumb");

    let sw_on_cmds = ui.frame(Input::default(), bounds, 0., |f| { SW_ON.render(f, bounds, &mut state); });
    assert!(sw_on_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
            if (r.width - 52.0).abs() < 0.01 && (r.height - 32.0).abs() < 0.01 && (*radius - 16.0).abs() < 0.01
    )), "M3 checked Switch must draw 52x32 primary pill track");
    assert!(!sw_on_cmds.iter().any(|c| matches!(c, DrawCommand::Outline { .. })), "M3 checked Switch must not draw unchecked outline border");
    assert!(sw_on_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xFF, 0xFF, 0xFF, 255) }
            if (r.width - 24.0).abs() < 0.01 && (r.height - 24.0).abs() < 0.01 && (*radius - 12.0).abs() < 0.01
    )), "M3 checked Switch must draw 24dp onPrimary thumb");

    // --- 2. Material3Light CheckBox: 18x18 box, 2dp radius, 2dp onSurfaceVariant stroke unchecked / primary fill checked ---
    let cb_off_cmds = ui.frame(Input::default(), bounds, 0., |f| { CB_OFF.render(f, bounds, &mut state); });
    assert!(cb_off_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, color: Color(0x49, 0x45, 0x4F, 255) }
            if (r.width - 18.0).abs() < 0.01 && (r.height - 18.0).abs() < 0.01 && (*radius - 2.0).abs() < 0.01 && (*width - 2.0).abs() < 0.01
    )), "M3 unchecked CheckBox must draw 18x18 2dp-radius 2dp onSurfaceVariant outline");
    let cb_on_cmds = ui.frame(Input::default(), bounds, 0., |f| { CB_ON.render(f, bounds, &mut state); });
    assert!(cb_on_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
            if (r.width - 18.0).abs() < 0.01 && (r.height - 18.0).abs() < 0.01 && (*radius - 2.0).abs() < 0.01
    )), "M3 checked CheckBox must draw 18x18 2dp-radius primary fill");
    assert!(cb_on_cmds.iter().any(|c| matches!(c, DrawCommand::Polyline { count: 3, color: Color(0xFF, 0xFF, 0xFF, 255), .. })));

    // --- 3. Material3Light RadioButton: 20x20 ring (2dp stroke) + 10dp inner dot when checked ---
    let rb_off_cmds = ui.frame(Input::default(), bounds, 0., |f| { RB_OFF.render(f, bounds, &mut state); });
    assert!(rb_off_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, color: Color(0x49, 0x45, 0x4F, 255) }
            if (r.width - 20.0).abs() < 0.01 && (r.height - 20.0).abs() < 0.01 && (*radius - 10.0).abs() < 0.01 && (*width - 2.0).abs() < 0.01
    )));
    assert!(!rb_off_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, .. } if (r.width - 10.0).abs() < 0.01)));
    let rb_on_cmds = ui.frame(Input::default(), bounds, 0., |f| { RB_ON.render(f, bounds, &mut state); });
    assert!(rb_on_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, color: Color(0x67, 0x50, 0xA4, 255) }
            if (r.width - 20.0).abs() < 0.01 && (r.height - 20.0).abs() < 0.01 && (*radius - 10.0).abs() < 0.01 && (*width - 2.0).abs() < 0.01
    )));
    assert!(rb_on_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
            if (r.width - 10.0).abs() < 0.01 && (r.height - 10.0).abs() < 0.01 && (*radius - 5.0).abs() < 0.01
    )));

    // --- 4. Density = 1.5 scaling on M3 Switch, CheckBox, RadioButton ---
    state.density = 1.5;
    let hi_bounds = Rect { x: 0., y: 0., width: 360., height: 72. };
    let sw_hi = ui.frame(Input::default(), hi_bounds, 0., |f| { SW_ON.render(f, hi_bounds, &mut state); });
    assert!(sw_hi.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, .. } if (r.width - 78.0).abs() < 0.01 && (r.height - 48.0).abs() < 0.01)));
    assert!(sw_hi.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, .. } if (r.width - 36.0).abs() < 0.01 && (r.height - 36.0).abs() < 0.01)));
    let cb_hi = ui.frame(Input::default(), hi_bounds, 0., |f| { CB_OFF.render(f, hi_bounds, &mut state); });
    assert!(cb_hi.iter().any(|c| matches!(c, DrawCommand::Outline { bounds: r, width, .. } if (r.width - 27.0).abs() < 0.01 && (*width - 3.0).abs() < 0.01)));
    let rb_hi = ui.frame(Input::default(), hi_bounds, 0., |f| { RB_ON.render(f, hi_bounds, &mut state); });
    assert!(rb_hi.iter().any(|c| matches!(c, DrawCommand::Outline { bounds: r, .. } if (r.width - 30.0).abs() < 0.01)));
    assert!(rb_hi.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, .. } if (r.width - 15.0).abs() < 0.01)));

    // --- 5. Spinner: MaterialLight underline + arrow, Material3Light 4dp outlined container + arrow, Bhoo legacy filled ---
    state.density = 1.0;
    state.theme_style = AndroidTheme::MaterialLight;
    let sp_ml = ui.frame(Input::default(), bounds, 0., |f| { SP.render(f, bounds, &mut state); });
    assert!(sp_ml.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0, 0, 0, 0x6B), .. } if (r.height - 1.0).abs() < 0.01 && r.y >= 46.5)));
    assert!(!sp_ml.iter().any(|c| matches!(c, DrawCommand::Outline { .. })), "MaterialLight Spinner should show bottom underline, not full outline");
    assert!(sp_ml.iter().any(|c| matches!(c, DrawCommand::Polyline { count: 3, .. })), "MaterialLight Spinner must draw trailing dropdown arrow");

    state.theme_style = AndroidTheme::Material3Light;
    let sp_m3 = ui.frame(Input::default(), bounds, 0., |f| { SP.render(f, bounds, &mut state); });
    assert!(sp_m3.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, color: Color(0x79, 0x74, 0x7E, 255) }
            if *r == bounds && (*radius - 4.0).abs() < 0.01 && (*width - 1.0).abs() < 0.01
    )), "Material3Light Spinner must show 4dp outlined container");
    assert!(!sp_m3.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0xE6, 0xE0, 0xE9, 255), .. } if *r == bounds)));
    assert!(sp_m3.iter().any(|c| matches!(c, DrawCommand::Polyline { count: 3, .. })), "Material3Light Spinner must draw trailing dropdown arrow");

    state.theme_style = AndroidTheme::Bhoo;
    let sp_bhoo = ui.frame(Input::default(), bounds, 0., |f| { SP.render(f, bounds, &mut state); });
    assert!(sp_bhoo.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, radius, .. } if *r == bounds && (*radius - 4.0).abs() < 0.01)));
}

#[test]
fn line_height_and_include_font_padding_rules_drive_measurement_baseline_and_rendering() {
    use bhoo::android_xml::{AndroidTheme, RULE_INCLUDE_FONT_PADDING, RULE_LINE_HEIGHT};

    static LH_28: &[RelativeRule] = &[RelativeRule { kind: RULE_LINE_HEIGHT, anchor: "28" }];
    static LH_TINY: &[RelativeRule] = &[RelativeRule { kind: RULE_LINE_HEIGHT, anchor: "8" }];
    static NO_PAD: &[RelativeRule] = &[RelativeRule { kind: RULE_INCLUDE_FONT_PADDING, anchor: "false" }];
    static WITH_PAD: &[RelativeRule] = &[RelativeRule { kind: RULE_INCLUDE_FONT_PADDING, anchor: "true" }];

    static WRAP_DEFAULT: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Text, text: "Line one\nLine two", text_size: 16., ..button("t_def", Visibility::Visible, Size::Wrap, 0.) },
            button("after_def", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("col_def", Visibility::Visible, Size::Match, 0.)
    };
    static WRAP_LH_28: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Text, text: "Line one\nLine two", text_size: 16., relative_rules: LH_28, ..button("t_lh", Visibility::Visible, Size::Wrap, 0.) },
            button("after_lh", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("col_lh", Visibility::Visible, Size::Match, 0.)
    };
    static WRAP_LH_TINY: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Text, text: "Clamp me", text_size: 16., relative_rules: LH_TINY, ..button("t_clamp", Visibility::Visible, Size::Wrap, 0.) },
            button("after_clamp", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("col_clamp", Visibility::Visible, Size::Match, 0.)
    };
    static WRAP_NO_PAD: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Text, text: "Tight one\nTight two", text_size: 16., relative_rules: NO_PAD, ..button("t_nopad", Visibility::Visible, Size::Wrap, 0.) },
            button("after_nopad", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("col_nopad", Visibility::Visible, Size::Match, 0.)
    };
    static WRAP_WITH_PAD: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Text, text: "Loose one\nLoose two", text_size: 16., relative_rules: WITH_PAD, ..button("t_pad", Visibility::Visible, Size::Wrap, 0.) },
            button("after_pad", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("col_pad", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 240. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Default Material3Light uses 1.34x (2 lines = 2 * 16 * 1.34 = 42.88)
    let cmds_def = ui.frame(Input::default(), bounds, 0., |f| { WRAP_DEFAULT.render(f, bounds, &mut state); });
    let y_def = ui.semantics().iter().find(|n| n.label.as_ref() == "after_def").unwrap().bounds.y;
    assert!((y_def - 42.88).abs() < 0.05, "expected 42.88, got {y_def}");
    assert!(cmds_def.iter().any(|c| matches!(c, DrawCommand::Text { text, bounds: r, .. } if text.as_ref() == "Line one" && (r.height - 21.44).abs() < 0.05)));

    // 2. RULE_LINE_HEIGHT="28" overrides line height to 28px (2 lines = 56px)
    let cmds_lh = ui.frame(Input::default(), bounds, 0., |f| { WRAP_LH_28.render(f, bounds, &mut state); });
    let y_lh = ui.semantics().iter().find(|n| n.label.as_ref() == "after_lh").unwrap().bounds.y;
    assert!((y_lh - 56.0).abs() < 0.05, "expected 56.0, got {y_lh}");
    assert!(cmds_lh.iter().any(|c| matches!(c, DrawCommand::Text { text, bounds: r, .. } if text.as_ref() == "Line two" && (r.y - 28.0).abs() < 0.05 && (r.height - 28.0).abs() < 0.05)));

    // 3. Density (1.5) and font_scale (1.25) scale RULE_LINE_HEIGHT: 28 * 1.5 * 1.25 = 52.5 per line -> 105.0 for 2 lines
    state.density = 1.5;
    state.font_scale = 1.25;
    let cmds_scaled = ui.frame(Input::default(), bounds, 0., |f| { WRAP_LH_28.render(f, bounds, &mut state); });
    let y_scaled = ui.semantics().iter().find(|n| n.label.as_ref() == "after_lh").unwrap().bounds.y;
    assert!((y_scaled - 105.0).abs() < 0.05, "expected 105.0, got {y_scaled}");
    assert!(cmds_scaled.iter().any(|c| matches!(c, DrawCommand::Text { text, bounds: r, .. } if text.as_ref() == "Line two" && (r.y - 52.5).abs() < 0.05 && (r.height - 52.5).abs() < 0.05)));

    // 4. RULE_LINE_HEIGHT smaller than font size clamps to scaled font size (16 * 1.5 * 1.25 = 30.0)
    ui.frame(Input::default(), bounds, 0., |f| { WRAP_LH_TINY.render(f, bounds, &mut state); });
    let y_clamp = ui.semantics().iter().find(|n| n.label.as_ref() == "after_clamp").unwrap().bounds.y;
    assert!((y_clamp - 30.0).abs() < 0.05, "expected clamped 30.0, got {y_clamp}");

    // 5. RULE_INCLUDE_FONT_PADDING="false" tightens factor from 1.34x (and 1.5x Bhoo) to 1.18x (2 * 16 * 1.18 = 37.76)
    state.density = 1.0;
    state.font_scale = 1.0;
    let cmds_nopad = ui.frame(Input::default(), bounds, 0., |f| { WRAP_NO_PAD.render(f, bounds, &mut state); });
    let y_nopad = ui.semantics().iter().find(|n| n.label.as_ref() == "after_nopad").unwrap().bounds.y;
    assert!((y_nopad - 37.76).abs() < 0.05, "expected 37.76, got {y_nopad}");
    assert!(cmds_nopad.iter().any(|c| matches!(c, DrawCommand::Text { text, bounds: r, .. } if text.as_ref() == "Tight two" && (r.y - 18.88).abs() < 0.05 && (r.height - 18.88).abs() < 0.05)));

    // Explicitly "true" preserves theme factor (1.34x in M3, 1.5x in Bhoo)
    ui.frame(Input::default(), bounds, 0., |f| { WRAP_WITH_PAD.render(f, bounds, &mut state); });
    let y_pad_m3 = ui.semantics().iter().find(|n| n.label.as_ref() == "after_pad").unwrap().bounds.y;
    assert!((y_pad_m3 - 42.88).abs() < 0.05, "expected 42.88, got {y_pad_m3}");
    state.theme_style = AndroidTheme::Bhoo;
    ui.frame(Input::default(), bounds, 0., |f| { WRAP_NO_PAD.render(f, bounds, &mut state); });
    let y_nopad_bhoo = ui.semantics().iter().find(|n| n.label.as_ref() == "after_nopad").unwrap().bounds.y;
    assert!((y_nopad_bhoo - 37.76).abs() < 0.05, "Bhoo includeFontPadding=false should also use 1.18x, got {y_nopad_bhoo}");

    // 6. Baseline alignment in a horizontal LinearLayout reflects RULE_LINE_HEIGHT
    static BASELINE_ROW: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        children: &[
            Node { kind: Kind::Text, width: Size::Wrap, text: "Tall", text_size: 16., relative_rules: LH_28, ..button("b_tall", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Text, width: Size::Wrap, text: "Tight", text_size: 16., relative_rules: NO_PAD, ..button("b_tight", Visibility::Visible, Size::Wrap, 0.) },
        ],
        ..button("base_row", Visibility::Visible, Size::Wrap, 0.)
    };
    state.theme_style = AndroidTheme::Material3Light;
    let base_cmds = ui.frame(Input::default(), bounds, 0., |f| { BASELINE_ROW.render(f, bounds, &mut state); });
    let tall_r = base_cmds.iter().find_map(|c| match c { DrawCommand::Text { text, bounds: r, .. } if text.as_ref() == "Tall" => Some(*r), _ => None }).unwrap();
    let tight_r = base_cmds.iter().find_map(|c| match c { DrawCommand::Text { text, bounds: r, .. } if text.as_ref() == "Tight" => Some(*r), _ => None }).unwrap();
    assert!((tall_r.height - 28.0).abs() < 0.05 && (tight_r.height - 18.88).abs() < 0.05);
    assert!(((tall_r.y + tall_r.height * 0.5) - (tight_r.y + tight_r.height * 0.5)).abs() < 0.1, "baselines must align: tall={tall_r:?}, tight={tight_r:?}");
}

#[test]
fn nav_bar_rail_and_drawer_rules_render_m3_indicators_and_update_selection() {
    use bhoo::android_xml::{AndroidTheme, RULE_NAV_BAR, RULE_NAV_ITEM};

    static BAR_LABELED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "bar|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_home|Home|home" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_search|Search|search" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_saved|Saved|star" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_settings|Settings|settings" },
    ];
    static BAR_SELECTED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "bar|selected" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_home|Home|home" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_search|Search|search" },
    ];
    static BAR_UNLABELED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "bar|unlabeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_home|Home|home" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_search|Search|search" },
    ];
    static BAR_AUTO_4_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "bar|auto" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "a|One|home" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "b|Two|search" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "c|Three|star" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "d|Four|settings" },
    ];
    static RAIL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "rail|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "r_inbox|Inbox|inbox" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "r_star|Starred|star" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "r_arch|Archive|archive" },
    ];
    static DRAWER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "drawer|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "d_mail|Inbox|mail" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "d_notes|Notes|" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "d_settings|Settings|settings" },
    ];

    static BAR_HOST: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Linear, relative_rules: BAR_LABELED_RULES, ..button("m3_bar", Visibility::Visible, Size::Wrap, 0.) },
            button("below_bar", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("bar_host", Visibility::Visible, Size::Match, 0.)
    };
    static BAR_SELECTED: Node = Node { kind: Kind::Linear, relative_rules: BAR_SELECTED_RULES, ..button("bar_sel", Visibility::Visible, Size::Dp(80.), 0.) };
    static BAR_UNLABELED: Node = Node { kind: Kind::Linear, relative_rules: BAR_UNLABELED_RULES, ..button("bar_unlab", Visibility::Visible, Size::Dp(80.), 0.) };
    static BAR_AUTO_4: Node = Node { kind: Kind::Linear, relative_rules: BAR_AUTO_4_RULES, ..button("bar_auto4", Visibility::Visible, Size::Dp(80.), 0.) };
    static RAIL_HOST: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        children: &[
            Node { kind: Kind::Linear, width: Size::Wrap, relative_rules: RAIL_RULES, ..button("m3_rail", Visibility::Visible, Size::Match, 0.) },
            button("beside_rail", Visibility::Visible, Size::Match, 1.),
        ],
        ..button("rail_host", Visibility::Visible, Size::Match, 0.)
    };
    static DRAWER_NODE: Node = Node { kind: Kind::Linear, relative_rules: DRAWER_RULES, ..button("m3_drawer", Visibility::Visible, Size::Match, 0.) };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    let bounds = Rect { x: 0., y: 0., width: 320., height: 200. };

    // 1. NavigationBar: 80dp wrap height, #F3EDF7 surface, 64x32 secondaryContainer pill on slot 0, all 4 labels visible
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { BAR_HOST.render(f, bounds, &mut state); });
    let below_y = ui.semantics().iter().find(|n| n.label.as_ref() == "below_bar").unwrap().bounds.y;
    assert!((below_y - 80.0).abs() < 0.05, "NavigationBar wrap height must be 80dp, got {below_y}");
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0xF3, 0xED, 0xF7, 255), .. } if (r.height - 80.0).abs() < 0.05)));
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE8, 0xDE, 0xF8, 255) }
            if (r.width - 64.0).abs() < 0.05 && (r.height - 32.0).abs() < 0.05 && (*radius - 16.0).abs() < 0.05 && (r.x - 8.0).abs() < 0.5
    )), "slot 0 (width 80) must paint centered 64x32 secondaryContainer pill at x=8");
    for label in ["Home", "Search", "Saved", "Settings"] {
        assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == label && (style.size - 12.0).abs() < 0.05)));
    }

    // Click slot 2 ("Saved", x=160..240) -> updates state.selected_index["m3_bar"] = 2 & returns Some("nav_saved")
    ui.frame(Input { pointer: Some([200., 40.]), pressed: true, ..Input::default() }, bounds, 0., |f| { BAR_HOST.render(f, bounds, &mut state); });
    let mut act = None;
    let cmds2 = ui.frame(Input { pointer: Some([200., 40.]), released: true, ..Input::default() }, bounds, 0., |f| { act = BAR_HOST.render(f, bounds, &mut state); });
    assert_eq!(act, Some("nav_saved"));
    assert_eq!(state.selected_index.get("m3_bar"), Some(&2));
    assert!(cmds2.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0xE8, 0xDE, 0xF8, 255), .. }
            if (r.width - 64.0).abs() < 0.05 && (r.x - 168.0).abs() < 0.5
    )), "pill must move immediately to slot 2 at x=168");

    // Dark theme surface (#211F26) and secondaryContainer (#4A4458)
    state.theme_style = AndroidTheme::Material3Dark;
    let dark_cmds = ui.frame(Input::default(), bounds, 0., |f| { BAR_HOST.render(f, bounds, &mut state); });
    assert!(dark_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0x21, 0x1F, 0x26, 255), .. } if (r.height - 80.0).abs() < 0.05)));
    assert!(dark_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds: r, color: Color(0x4A, 0x44, 0x58, 255), .. } if (r.width - 64.0).abs() < 0.05)));

    // 2. label_mode checks: selected, unlabeled, auto (>3 items)
    state.theme_style = AndroidTheme::Material3Light;
    let bar_b = Rect { x: 0., y: 0., width: 320., height: 80. };
    let sel_cmds = ui.frame(Input::default(), bar_b, 0., |f| { BAR_SELECTED.render(f, bar_b, &mut state); });
    assert!(sel_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Home")));
    assert!(!sel_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Search")));
    let unlab_cmds = ui.frame(Input::default(), bar_b, 0., |f| { BAR_UNLABELED.render(f, bar_b, &mut state); });
    assert!(!unlab_cmds.iter().any(|c| matches!(c, DrawCommand::Text { .. })));
    let auto_cmds = ui.frame(Input::default(), bar_b, 0., |f| { BAR_AUTO_4.render(f, bar_b, &mut state); });
    assert!(auto_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "One")));
    assert!(!auto_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Two")));

    // 3. NavigationRail: 80dp wrap width, 56x32 active pill, interactive selection
    let rail_b = Rect { x: 0., y: 0., width: 240., height: 260. };
    let rail_cmds = ui.frame(Input::default(), rail_b, 0., |f| { RAIL_HOST.render(f, rail_b, &mut state); });
    let beside_x = ui.semantics().iter().find(|n| n.label.as_ref() == "beside_rail").unwrap().bounds.x;
    assert!((beside_x - 80.0).abs() < 0.05, "NavigationRail wrap width must be 80dp, got {beside_x}");
    assert!(rail_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE8, 0xDE, 0xF8, 255) }
            if (r.width - 56.0).abs() < 0.05 && (r.height - 32.0).abs() < 0.05 && (*radius - 16.0).abs() < 0.05
    )));
    // Click second rail destination (top_pad=12, item 0=56, gap=12 -> item 1 at y=80..136)
    ui.frame(Input { pointer: Some([40., 100.]), pressed: true, ..Input::default() }, rail_b, 0., |f| { RAIL_HOST.render(f, rail_b, &mut state); });
    let mut rail_act = None;
    ui.frame(Input { pointer: Some([40., 100.]), released: true, ..Input::default() }, rail_b, 0., |f| { rail_act = RAIL_HOST.render(f, rail_b, &mut state); });
    assert_eq!(rail_act, Some("r_star"));
    assert_eq!(state.selected_index.get("m3_rail"), Some(&1));

    // 4. NavigationDrawer: 56dp-tall full-pill active row with 12dp horizontal inset, 14sp label, interactive selection
    let drawer_b = Rect { x: 0., y: 0., width: 300., height: 220. };
    let drawer_cmds = ui.frame(Input::default(), drawer_b, 0., |f| { DRAWER_NODE.render(f, drawer_b, &mut state); });
    assert!(drawer_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE8, 0xDE, 0xF8, 255) }
            if (r.x - 12.0).abs() < 0.05 && (r.width - 276.0).abs() < 0.05 && (r.height - 56.0).abs() < 0.05 && (*radius - 28.0).abs() < 0.05
    )));
    assert!(drawer_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == "Inbox" && (style.size - 14.0).abs() < 0.05)));
    // Row 2 ("Settings") is at y = 12 + 2*56 = 124..180
    ui.frame(Input { pointer: Some([100., 150.]), pressed: true, ..Input::default() }, drawer_b, 0., |f| { DRAWER_NODE.render(f, drawer_b, &mut state); });
    let mut drawer_act = None;
    ui.frame(Input { pointer: Some([100., 150.]), released: true, ..Input::default() }, drawer_b, 0., |f| { drawer_act = DRAWER_NODE.render(f, drawer_b, &mut state); });
    assert_eq!(drawer_act, Some("d_settings"));
    assert_eq!(state.selected_index.get("m3_drawer"), Some(&2));
}

#[test]
fn search_bar_rule_renders_m3_pill_surface_icon_hint_query_and_focus() {
    use bhoo::android_xml::{AndroidTheme, RULE_SEARCH_BAR};

    static SB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SEARCH_BAR, anchor: "Search mail" }];
    static SB_HOST: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::SearchView, relative_rules: SB_RULES, ..button("m3_sb", Visibility::Visible, Size::Wrap, 0.) },
            button("after_sb", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("sb_host", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 120. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Default wrap height = 56dp, full pill radius = 28dp, surfaceContainerHigh #ECE6F0, leading Search icon, 16sp onSurfaceVariant hint
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { SB_HOST.render(f, bounds, &mut state); });
    let after_y = ui.semantics().iter().find(|n| n.label.as_ref() == "after_sb").unwrap().bounds.y;
    assert!((after_y - 56.0).abs() < 0.05, "SearchBar default wrap height must be 56dp, got {after_y}");
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xEC, 0xE6, 0xF0, 255) }
            if (r.height - 56.0).abs() < 0.05 && (*radius - 28.0).abs() < 0.05
    )));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Shape { bounds: r, .. } if (r.x - 16.0).abs() < 0.05 && (r.width - 24.0).abs() < 0.05)));
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0x49, 0x45, 0x4F, 255), style, .. }
            if text.as_ref() == "Search mail" && (style.size - 16.0).abs() < 0.05
    )));

    // 2. Interactive click focuses SearchBar and returns its XML ID
    ui.frame(Input { pointer: Some([120., 28.]), pressed: true, ..Input::default() }, bounds, 0., |f| { SB_HOST.render(f, bounds, &mut state); });
    let mut act = None;
    ui.frame(Input { pointer: Some([120., 28.]), released: true, ..Input::default() }, bounds, 0., |f| { act = SB_HOST.render(f, bounds, &mut state); });
    assert_eq!(act, Some("m3_sb"));
    assert_eq!(state.search_open.get("m3_sb"), Some(&true));

    // 3. Query in state.text replaces hint with onSurface text, and dark theme uses #2B2930
    state.theme_style = AndroidTheme::Material3Dark;
    state.text.insert("m3_sb".into(), "invoices".into());
    let dark_cmds = ui.frame(Input::default(), bounds, 0., |f| { SB_HOST.render(f, bounds, &mut state); });
    assert!(dark_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x2B, 0x29, 0x30, 255) }
            if (r.height - 56.0).abs() < 0.05 && (*radius - 28.0).abs() < 0.05
    )));
    assert!(dark_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0xE6, 0xE0, 0xE9, 255), .. } if text.as_ref() == "invoices"
    )));

    // 4. RTL places leading search icon on the right (x = 320 - 16 - 24 = 280)
    state.rtl = true;
    let rtl_cmds = ui.frame(Input::default(), bounds, 0., |f| { SB_HOST.render(f, bounds, &mut state); });
    assert!(rtl_cmds.iter().any(|c| matches!(c, DrawCommand::Shape { bounds: r, .. } if (r.x - 280.0).abs() < 0.05)));
}

#[test]
fn toggle_group_rule_connects_corners_avoids_doubled_seams_and_handles_selection_modes() {
    use bhoo::android_xml::{AndroidTheme, RULE_BUTTON_VARIANT, RULE_TOGGLE_GROUP};

    static OUTLINED_BTN: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "outlined" }];
    static TG_SINGLE_REQ: &[RelativeRule] = &[RelativeRule { kind: RULE_TOGGLE_GROUP, anchor: "1|1|seg_mid" }];
    static TG_SINGLE_OPT: &[RelativeRule] = &[RelativeRule { kind: RULE_TOGGLE_GROUP, anchor: "1|0|seg_left" }];
    static TG_MULTI_REQ: &[RelativeRule] = &[RelativeRule { kind: RULE_TOGGLE_GROUP, anchor: "0|1|seg_left" }];

    static GROUP_SINGLE_REQ: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        relative_rules: TG_SINGLE_REQ,
        children: &[
            Node { weight: 1., text: "Day", relative_rules: OUTLINED_BTN, ..button("seg_left", Visibility::Visible, Size::Match, 1.) },
            Node { weight: 1., text: "Week", relative_rules: OUTLINED_BTN, ..button("seg_mid", Visibility::Visible, Size::Match, 1.) },
            Node { weight: 1., text: "Month", relative_rules: OUTLINED_BTN, ..button("seg_right", Visibility::Visible, Size::Match, 1.) },
        ],
        ..button("tg_single_req", Visibility::Visible, Size::Dp(40.), 0.)
    };
    static GROUP_SINGLE_OPT: Node = Node {
        id: "tg_single_opt",
        relative_rules: TG_SINGLE_OPT,
        ..GROUP_SINGLE_REQ
    };
    static GROUP_MULTI_REQ: Node = Node {
        id: "tg_multi_req",
        relative_rules: TG_MULTI_REQ,
        ..GROUP_SINGLE_REQ
    };

    let bounds = Rect { x: 0., y: 0., width: 300., height: 40. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Initial checked_id="seg_mid" fills middle button with secondaryContainer, square corners, and leading Check icon;
    //    outer left has [20,0,0,20], middle has [0,0,0,0], outer right has [0,20,20,0], and non-left outlines shift left by 1dp
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { GROUP_SINGLE_REQ.render(f, bounds, &mut state); });
    assert_eq!(state.checked.get("seg_left"), Some(&false));
    assert_eq!(state.checked.get("seg_mid"), Some(&true));
    assert_eq!(state.checked.get("seg_right"), Some(&false));
    // Checked middle segment: square fill (radius=0) at x=100..200 with secondaryContainer #E8DEF8 + Check icon
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE8, 0xDE, 0xF8, 255) }
            if (r.x - 100.0).abs() < 0.05 && (r.width - 100.0).abs() < 0.05 && *radius == 0.0
    )));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Shape { bounds: r, .. } if r.x >= 100.0 && r.x < 200.0 && (r.width - 18.0).abs() < 0.05)));
    // Outer left outline has rounded left cap (radius=20 at x=0) and flat right edge (radius=0)
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, width, color: Color(0x79, 0x74, 0x7E, 255) }
            if (r.x - 0.0).abs() < 0.05 && (*radius - 20.0).abs() < 0.05 && (*width - 1.0).abs() < 0.05
    )));
    // Middle outline is shifted left by sw=1 (x=99) inside clip x=100..200 so inner seam is NOT doubled
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, .. } if (r.x - 99.0).abs() < 0.05 && *radius == 0.0
    )));
    // Outer right outline has flat shifted-left side (x=199) and rounded right cap (radius=20 reaching x=300)
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds: r, radius, .. }
            if (r.x + r.width - 300.0).abs() < 0.05 && (*radius - 20.0).abs() < 0.05
    )));

    // 2. Single selection + selection_required: clicking already-checked seg_mid keeps it checked; clicking seg_left switches exclusively
    for input in [Input { pointer: Some([150., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([150., 20.]), released: true, ..Input::default() }] {
        ui.frame(input, bounds, 0., |f| { GROUP_SINGLE_REQ.render(f, bounds, &mut state); });
    }
    assert_eq!(state.checked.get("seg_mid"), Some(&true), "selection_required must prevent unchecking the sole selected button");

    for input in [Input { pointer: Some([50., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([50., 20.]), released: true, ..Input::default() }] {
        ui.frame(input, bounds, 0., |f| { GROUP_SINGLE_REQ.render(f, bounds, &mut state); });
    }
    assert_eq!(state.checked.get("seg_left"), Some(&true));
    assert_eq!(state.checked.get("seg_mid"), Some(&false));
    assert_eq!(state.checked.get("seg_right"), Some(&false));

    // Verify outer-left checked button renders rounded left cap fill (radius=20 at x=0) in LTR,
    // and mirrors to rounded right cap fill (radius=20 ending at x=300) in RTL!
    let ltr_cmds = ui.frame(Input::default(), bounds, 0., |f| { GROUP_SINGLE_REQ.render(f, bounds, &mut state); });
    assert!(ltr_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE8, 0xDE, 0xF8, 255) }
            if (r.x - 0.0).abs() < 0.05 && (*radius - 20.0).abs() < 0.05
    )));
    state.rtl = true;
    let rtl_cmds = ui.frame(Input::default(), bounds, 0., |f| { GROUP_SINGLE_REQ.render(f, bounds, &mut state); });
    assert!(rtl_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xE8, 0xDE, 0xF8, 255) }
            if (r.x + r.width - 300.0).abs() < 0.05 && (*radius - 20.0).abs() < 0.05
    )), "RTL must mirror outer-first button's rounded corners to physical right cap");

    // 3. Single selection + selection_required=0: clicking the active button unchecks it
    let mut state_opt = LayoutState::default();
    state_opt.theme_style = AndroidTheme::Material3Light;
    ui.frame(Input::default(), bounds, 0., |f| { GROUP_SINGLE_OPT.render(f, bounds, &mut state_opt); });
    assert_eq!(state_opt.checked.get("seg_left"), Some(&true));
    for input in [Input { pointer: Some([50., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([50., 20.]), released: true, ..Input::default() }] {
        ui.frame(input, bounds, 0., |f| { GROUP_SINGLE_OPT.render(f, bounds, &mut state_opt); });
    }
    assert_eq!(state_opt.checked.get("seg_left"), Some(&false));

    // 4. Multi-selection + selection_required=1: can check multiple, can uncheck one, cannot uncheck last
    let mut state_multi = LayoutState::default();
    state_multi.theme_style = AndroidTheme::Material3Light;
    ui.frame(Input::default(), bounds, 0., |f| { GROUP_MULTI_REQ.render(f, bounds, &mut state_multi); });
    assert_eq!(state_multi.checked.get("seg_left"), Some(&true));
    assert_eq!(state_multi.checked.get("seg_right"), Some(&false));
    // Add seg_right
    for input in [Input { pointer: Some([250., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([250., 20.]), released: true, ..Input::default() }] {
        ui.frame(input, bounds, 0., |f| { GROUP_MULTI_REQ.render(f, bounds, &mut state_multi); });
    }
    assert_eq!(state_multi.checked.get("seg_left"), Some(&true));
    assert_eq!(state_multi.checked.get("seg_right"), Some(&true));
    // Uncheck seg_left (allowed because seg_right stays checked)
    for input in [Input { pointer: Some([50., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([50., 20.]), released: true, ..Input::default() }] {
        ui.frame(input, bounds, 0., |f| { GROUP_MULTI_REQ.render(f, bounds, &mut state_multi); });
    }
    assert_eq!(state_multi.checked.get("seg_left"), Some(&false));
    assert_eq!(state_multi.checked.get("seg_right"), Some(&true));
    // Attempt to uncheck last remaining seg_right -> blocked by selection_required=1
    for input in [Input { pointer: Some([250., 20.]), pressed: true, ..Input::default() }, Input { pointer: Some([250., 20.]), released: true, ..Input::default() }] {
        ui.frame(input, bounds, 0., |f| { GROUP_MULTI_REQ.render(f, bounds, &mut state_multi); });
    }
    assert_eq!(state_multi.checked.get("seg_right"), Some(&true));
}

#[test]
fn extended_fab_rule_renders_m3_container_elevation_padding_overrides_and_collapsed() {
    use bhoo::android_xml::{AndroidTheme, RULE_CORNER_RADII, RULE_EXTENDED_FAB};

    static EFAB_EXT: &[RelativeRule] = &[RelativeRule { kind: RULE_EXTENDED_FAB, anchor: "1" }];
    static EFAB_COL: &[RelativeRule] = &[RelativeRule { kind: RULE_EXTENDED_FAB, anchor: "0" }];
    static EFAB_RADII: &[RelativeRule] = &[
        RelativeRule { kind: RULE_EXTENDED_FAB, anchor: "1" },
        RelativeRule { kind: RULE_CORNER_RADII, anchor: "4,12,12,4" },
    ];

    static EXT_HOST: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node {
                width: Size::Wrap,
                height: Size::Wrap,
                text: "Compose",
                image_src: "edit",
                relative_rules: EFAB_EXT,
                ..button("efab_ext", Visibility::Visible, Size::Wrap, 0.)
            },
            button("after_efab", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("efab_host", Visibility::Visible, Size::Match, 0.)
    };
    static EXT_TEXT_ONLY: Node = Node {
        width: Size::Wrap,
        height: Size::Wrap,
        text: "Create",
        relative_rules: EFAB_EXT,
        ..button("efab_text", Visibility::Visible, Size::Wrap, 0.)
    };
    static EXT_DISABLED: Node = Node {
        enabled: false,
        ..EXT_TEXT_ONLY
    };
    static EXT_CUSTOM_BG: Node = Node {
        background: Some(Color(12, 120, 90, 255)),
        radius: 28.0,
        ..EXT_TEXT_ONLY
    };
    static EXT_ASYM_RADII: Node = Node {
        relative_rules: EFAB_RADII,
        ..EXT_TEXT_ONLY
    };
    static COL_HOST: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        children: &[
            Node {
                width: Size::Wrap,
                height: Size::Wrap,
                text: "Compose",
                icon: MaterialIcon::Add,
                relative_rules: EFAB_COL,
                ..button("efab_col", Visibility::Visible, Size::Wrap, 0.)
            },
            button("beside_col", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("col_host", Visibility::Visible, Size::Match, 0.)
    };
    static COL_VERT_HOST: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node {
                width: Size::Wrap,
                height: Size::Wrap,
                text: "Compose",
                icon: MaterialIcon::Add,
                relative_rules: EFAB_COL,
                ..button("efab_col_v", Visibility::Visible, Size::Wrap, 0.)
            },
            button("below_col", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("col_v_host", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 160. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Extended FAB: 56dp wrap height, 16dp radius, elevation level 3 shadow, M3 primaryContainer #EADDFF & onPrimaryContainer #21005D
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { EXT_HOST.render(f, bounds, &mut state); });
    let after_y = ui.semantics().iter().find(|n| n.label.as_ref() == "after_efab").unwrap().bounds.y;
    assert!((after_y - 56.0).abs() < 0.05, "Extended FAB wrap height must be 56dp, got {after_y}");
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Shadow { radius, .. } if (*radius - 16.0).abs() < 0.05)));
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xEA, 0xDD, 0xFF, 255) }
            if (r.height - 56.0).abs() < 0.05 && (*radius - 16.0).abs() < 0.05
    )));
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0x21, 0x00, 0x5D, 255), .. } if text.as_ref() == "Compose"
    )));
    // Leading icon in onPrimaryContainer at x=16 (start padding 16dp)
    let ltr_icon_x = cmds.iter().find_map(|c| match c {
        DrawCommand::Shape { bounds: r, color: Color(0x21, 0x00, 0x5D, 255), .. } => Some(r.x),
        _ => None,
    }).expect("Extended FAB leading icon must render in onPrimaryContainer");
    assert!((ltr_icon_x - 16.0).abs() < 0.5, "Extended FAB leading icon should start at 16dp, got {ltr_icon_x}");

    // Click returns XML ID
    ui.frame(Input { pointer: Some([40., 28.]), pressed: true, ..Input::default() }, bounds, 0., |f| { EXT_HOST.render(f, bounds, &mut state); });
    let mut clicked = None;
    ui.frame(Input { pointer: Some([40., 28.]), released: true, ..Input::default() }, bounds, 0., |f| { clicked = EXT_HOST.render(f, bounds, &mut state); });
    assert_eq!(clicked, Some("efab_ext"));

    // 2. Dark theme: primaryContainer #4F378B & onPrimaryContainer #EADDFF; density 2.0 scales height to 112px and radius to 32px
    state.theme_style = AndroidTheme::Material3Dark;
    state.density = 2.0;
    let dark_cmds = ui.frame(Input::default(), bounds, 0., |f| { EXT_HOST.render(f, bounds, &mut state); });
    let after_y_2x = ui.semantics().iter().find(|n| n.label.as_ref() == "after_efab").unwrap().bounds.y;
    assert!((after_y_2x - 112.0).abs() < 0.05, "2x density Extended FAB height must be 112px, got {after_y_2x}");
    assert!(dark_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x4F, 0x37, 0x8B, 255) }
            if (r.height - 112.0).abs() < 0.05 && (*radius - 32.0).abs() < 0.05
    )));
    assert!(dark_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0xEA, 0xDD, 0xFF, 255), .. } if text.as_ref() == "Compose"
    )));

    // 3. Padding [16, 0, 20, 0] LTR vs [20, 0, 16, 0] RTL on text-only Extended FAB
    state.density = 1.0;
    state.theme_style = AndroidTheme::Material3Light;
    let fab_b = Rect { x: 0., y: 0., width: 160., height: 56. };
    let ltr_text_cmds = ui.frame(Input::default(), fab_b, 0., |f| { EXT_TEXT_ONLY.render(f, fab_b, &mut state); });
    assert!(ltr_text_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, bounds: r, .. }
            if text.as_ref() == "Create" && (r.x - 16.0).abs() < 0.05 && (r.width - 124.0).abs() < 0.05
    )));
    state.rtl = true;
    let rtl_text_cmds = ui.frame(Input::default(), fab_b, 0., |f| { EXT_TEXT_ONLY.render(f, fab_b, &mut state); });
    assert!(rtl_text_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, bounds: r, .. }
            if text.as_ref() == "Create" && (r.x - 20.0).abs() < 0.05 && (r.width - 124.0).abs() < 0.05
    )));
    state.rtl = false;

    // 4. Disabled Extended FAB omits default elevation shadow
    let dis_cmds = ui.frame(Input::default(), fab_b, 0., |f| { EXT_DISABLED.render(f, fab_b, &mut state); });
    assert!(!dis_cmds.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })));

    // 5. Explicit background, explicit radius, and RULE_CORNER_RADII override defaults
    let custom_cmds = ui.frame(Input::default(), fab_b, 0., |f| { EXT_CUSTOM_BG.render(f, fab_b, &mut state); });
    assert!(custom_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { radius, color: Color(12, 120, 90, 255), .. } if (*radius - 28.0).abs() < 0.05
    )));
    let asym_cmds = ui.frame(Input::default(), fab_b, 0., |f| { EXT_ASYM_RADII.render(f, fab_b, &mut state); });
    assert!(asym_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { radius, .. } if (*radius - 4.0).abs() < 0.05)));
    assert!(asym_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { radius, .. } if (*radius - 12.0).abs() < 0.05)));

    // 6. Collapsed Extended FAB (anchor="0"): 56dp x 56dp square-rounded FAB, hides text, centers icon
    let col_h_cmds = ui.frame(Input::default(), bounds, 0., |f| { COL_HOST.render(f, bounds, &mut state); });
    let beside_x = ui.semantics().iter().find(|n| n.label.as_ref() == "beside_col").unwrap().bounds.x;
    assert!((beside_x - 56.0).abs() < 0.05, "Collapsed Extended FAB wrap width must be 56dp, got {beside_x}");
    assert!(!col_h_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Compose")));
    assert!(col_h_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Shape { bounds: r, color: Color(0x21, 0x00, 0x5D, 255), .. }
            if (r.x - 16.0).abs() < 0.5 && (r.y - 16.0).abs() < 0.5 && (r.width - 24.0).abs() < 0.05
    )));
    ui.frame(Input::default(), bounds, 0., |f| { COL_VERT_HOST.render(f, bounds, &mut state); });
    let below_col_y = ui.semantics().iter().find(|n| n.label.as_ref() == "below_col").unwrap().bounds.y;
    assert!((below_col_y - 56.0).abs() < 0.05, "Collapsed Extended FAB wrap height must be 56dp, got {below_col_y}");
}

#[test]
fn bottom_app_bar_rule_renders_surface_leading_actions_rtl_and_clicks() {
    use bhoo::android_xml::{AndroidTheme, RULE_BOTTOM_APP_BAR, RULE_NAV_ITEM};

    static BAB_END_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BOTTOM_APP_BAR, anchor: "end" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "bab_check|Done|check" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "bab_edit|Edit|edit" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "bab_search|Search|search" },
    ];
    static BAB_CENTER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BOTTOM_APP_BAR, anchor: "center" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "bab_home|Home|home" },
    ];

    static BAB_HOST: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node {
                kind: Kind::Toolbar,
                relative_rules: BAB_END_RULES,
                ..button("m3_bab", Visibility::Visible, Size::Wrap, 0.)
            },
            button("below_bab", Visibility::Visible, Size::Dp(20.), 0.),
        ],
        ..button("bab_host", Visibility::Visible, Size::Match, 0.)
    };
    static BAB_WITH_FAB: Node = Node {
        kind: Kind::Toolbar,
        relative_rules: BAB_CENTER_RULES,
        children: &[Node {
            width: Size::Dp(56.),
            height: Size::Dp(56.),
            ..button("center_fab", Visibility::Visible, Size::Dp(56.), 0.)
        }],
        ..button("bab_center", Visibility::Visible, Size::Dp(80.), 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 160. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Default wrap height = 80dp, surfaceContainer #F3EDF7 fill, 3 leading action icons
    let cmds = ui.frame(Input::default(), bounds, 0., |f| { BAB_HOST.render(f, bounds, &mut state); });
    let below_y = ui.semantics().iter().find(|n| n.label.as_ref() == "below_bab").unwrap().bounds.y;
    assert!((below_y - 80.0).abs() < 0.05, "BottomAppBar wrap height must be 80dp, got {below_y}");
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0xF3, 0xED, 0xF7, 255), .. }
            if (r.height - 80.0).abs() < 0.05
    )));
    let ltr_xs: Vec<f32> = cmds.iter().filter_map(|c| match c {
        DrawCommand::Shape { bounds: r, .. } if (r.width - 24.0).abs() < 0.05 => Some(r.x),
        _ => None,
    }).collect();
    assert_eq!(ltr_xs.len(), 3);
    assert!((ltr_xs[0] - 16.0).abs() < 0.5 && (ltr_xs[1] - 64.0).abs() < 0.5 && (ltr_xs[2] - 112.0).abs() < 0.5);

    // 2. Click 2nd action icon ("bab_edit", slot x = 52..100, y = 16..64)
    ui.frame(Input { pointer: Some([76., 40.]), pressed: true, ..Input::default() }, bounds, 0., |f| { BAB_HOST.render(f, bounds, &mut state); });
    let mut act = None;
    ui.frame(Input { pointer: Some([76., 40.]), released: true, ..Input::default() }, bounds, 0., |f| { act = BAB_HOST.render(f, bounds, &mut state); });
    assert_eq!(act, Some("bab_edit"));

    // 3. Dark theme surfaceContainer #211F26 & RTL mirrors action icons to right edge and clicks work there
    state.theme_style = AndroidTheme::Material3Dark;
    state.rtl = true;
    let rtl_cmds = ui.frame(Input::default(), bounds, 0., |f| { BAB_HOST.render(f, bounds, &mut state); });
    assert!(rtl_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0x21, 0x1F, 0x26, 255), .. }
            if (r.height - 80.0).abs() < 0.05
    )));
    let rtl_xs: Vec<f32> = rtl_cmds.iter().filter_map(|c| match c {
        DrawCommand::Shape { bounds: r, .. } if (r.width - 24.0).abs() < 0.05 => Some(r.x),
        _ => None,
    }).collect();
    assert_eq!(rtl_xs.len(), 3);
    assert!((rtl_xs[0] - 280.0).abs() < 0.5 && (rtl_xs[1] - 232.0).abs() < 0.5 && (rtl_xs[2] - 184.0).abs() < 0.5);
    // Click 2nd mirrored action icon ("bab_edit", center at x=244)
    ui.frame(Input { pointer: Some([244., 40.]), pressed: true, ..Input::default() }, bounds, 0., |f| { BAB_HOST.render(f, bounds, &mut state); });
    let mut rtl_act = None;
    ui.frame(Input { pointer: Some([244., 40.]), released: true, ..Input::default() }, bounds, 0., |f| { rtl_act = BAB_HOST.render(f, bounds, &mut state); });
    assert_eq!(rtl_act, Some("bab_edit"));

    // 4. Center FAB alignment positions embedded child centered horizontally
    state.rtl = false;
    let bar_b = Rect { x: 0., y: 0., width: 320., height: 80. };
    ui.frame(Input::default(), bar_b, 0., |f| { BAB_WITH_FAB.render(f, bar_b, &mut state); });
    let fab_sem = ui.semantics().iter().find(|n| n.label.as_ref() == "center_fab").unwrap().bounds;
    assert!((fab_sem.x - 132.0).abs() < 0.5, "Centered FAB in 320dp bar should sit at x=132, got {}", fab_sem.x);
}

#[test]
fn nav_bar_and_rail_badges_render_dot_pill_precedence_and_rtl() {
    use bhoo::android_xml::{AndroidTheme, RULE_NAV_BADGE, RULE_NAV_BAR, RULE_NAV_ITEM};

    static BAR_BADGE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "bar|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_home|Home|home" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_mail|Mail|mail" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_saved|Saved|star" },
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "nav_home:" },
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "nav_mail:99+" },
    ];
    static RAIL_BADGE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "rail|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "r_inbox|Inbox|inbox" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "r_star|Starred|star" },
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "r_inbox:" },
    ];

    static BAR_NODE: Node = Node { kind: Kind::Linear, relative_rules: BAR_BADGE_RULES, ..button("badge_bar", Visibility::Visible, Size::Dp(80.), 0.) };
    static RAIL_NODE: Node = Node { kind: Kind::Linear, width: Size::Dp(80.), relative_rules: RAIL_BADGE_RULES, ..button("badge_rail", Visibility::Visible, Size::Match, 0.) };

    let bar_b = Rect { x: 0., y: 0., width: 300., height: 80. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Declarative RULE_NAV_BADGE: 6dp red dot on nav_home + 16dp red pill "99+" with 11sp bold white text on nav_mail
    let cmds = ui.frame(Input::default(), bar_b, 0., |f| { BAR_NODE.render(f, bar_b, &mut state); });
    let dot_ltr = cmds.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xB3, 0x26, 0x1E, 255) }
            if (r.width - 6.0).abs() < 0.05 && (r.height - 6.0).abs() < 0.05 && (*radius - 3.0).abs() < 0.05 => Some(*r),
        _ => None,
    }).expect("Empty badge text must paint a 6dp circular M3 error dot");
    let pill_ltr = cmds.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xB3, 0x26, 0x1E, 255) }
            if (r.height - 16.0).abs() < 0.05 && (*radius - 8.0).abs() < 0.05 && r.width > 16.0 => Some(*r),
        _ => None,
    }).expect("Non-empty badge text '99+' must paint a 16dp-tall M3 error pill");
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0xFF, 0xFF, 0xFF, 255), style, .. }
            if text.as_ref() == "99+" && (style.size - 11.0).abs() < 0.05 && style.weight == 700
    )));
    // LTR anchoring: slot 0 center is x=50 -> dot center > 50; slot 1 center is x=150 -> pill center > 150
    assert!(dot_ltr.x + dot_ltr.width * 0.5 > 50.0);
    assert!(pill_ltr.x + pill_ltr.width * 0.5 > 150.0);

    // 2. RTL flips top-end anchoring to top-left of each destination icon
    state.rtl = true;
    let rtl_cmds = ui.frame(Input::default(), bar_b, 0., |f| { BAR_NODE.render(f, bar_b, &mut state); });
    // In RTL, slot 0 (nav_home) sits at x=200..300 (center 250), slot 1 (nav_mail) sits at x=100..200 (center 150)
    let dot_rtl = rtl_cmds.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xB3, 0x26, 0x1E, 255) }
            if (r.width - 6.0).abs() < 0.05 && (r.height - 6.0).abs() < 0.05 && (*radius - 3.0).abs() < 0.05 => Some(*r),
        _ => None,
    }).expect("RTL dot badge missing");
    let pill_rtl = rtl_cmds.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xB3, 0x26, 0x1E, 255) }
            if (r.height - 16.0).abs() < 0.05 && (*radius - 8.0).abs() < 0.05 && r.width > 16.0 => Some(*r),
        _ => None,
    }).expect("RTL pill badge missing");
    assert!(dot_rtl.x + dot_rtl.width * 0.5 < 250.0, "RTL dot must anchor at top-left of icon");
    assert!(pill_rtl.x + pill_rtl.width * 0.5 < 150.0, "RTL pill must anchor at top-left of icon");

    // 3. Runtime state.nav_badges overrides declarative RULE_NAV_BADGE + dark theme M3 error #F2B8B5 / onError #601410
    state.rtl = false;
    state.theme_style = AndroidTheme::Material3Dark;
    state.nav_badges.insert("nav_mail".into(), "4".into());
    state.nav_badges.insert("nav_saved".into(), String::new());
    let rt_cmds = ui.frame(Input::default(), bar_b, 0., |f| { BAR_NODE.render(f, bar_b, &mut state); });
    assert!(!rt_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "99+")));
    assert!(rt_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0x60, 0x14, 0x10, 255), style, .. }
            if text.as_ref() == "4" && (style.size - 11.0).abs() < 0.05 && style.weight == 700
    )));
    let dark_dots = rt_cmds.iter().filter(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xF2, 0xB8, 0xB5, 255) }
            if (r.width - 6.0).abs() < 0.05 && (r.height - 6.0).abs() < 0.05 && (*radius - 3.0).abs() < 0.05
    )).count();
    assert_eq!(dark_dots, 2, "Expected declarative nav_home dot + runtime nav_saved dot");

    // 4. NavigationRail badges (declarative dot + runtime pill)
    let rail_b = Rect { x: 0., y: 0., width: 80., height: 200. };
    state.theme_style = AndroidTheme::Material3Light;
    state.nav_badges.insert("r_star".into(), "12".into());
    let rail_cmds = ui.frame(Input::default(), rail_b, 0., |f| { RAIL_NODE.render(f, rail_b, &mut state); });
    assert!(rail_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xB3, 0x26, 0x1E, 255) }
            if (r.width - 6.0).abs() < 0.05 && (r.height - 6.0).abs() < 0.05 && (*radius - 3.0).abs() < 0.05
    )));
    assert!(rail_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0xFF, 0xFF, 0xFF, 255), style, .. }
            if text.as_ref() == "12" && (style.size - 11.0).abs() < 0.05 && style.weight == 700
    )));
}

#[test]
fn swipe_refresh_rule_paints_elevated_indicator_over_content_when_refreshing() {
    use bhoo::android_xml::{AndroidTheme, RULE_SWIPE_REFRESH};

    static SRL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SWIPE_REFRESH, anchor: "" }];
    static SRL_NODE: Node = Node {
        kind: Kind::Frame,
        relative_rules: SRL_RULES,
        children: &[Node {
            kind: Kind::Text,
            text: "Feed content",
            ..button("feed_text", Visibility::Visible, Size::Match, 0.)
        }],
        ..button("srl", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 320., height: 240. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Not refreshing: child renders, no circular refresh indicator or arc
    let idle = ui.frame(Input::default(), bounds, 0., |f| { SRL_NODE.render(f, bounds, &mut state); });
    assert!(idle.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Feed content")));
    assert!(!idle.iter().any(|c| matches!(c, DrawCommand::Arc { .. })));
    assert!(!idle.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })));

    // 2. Refreshing = true: 40dp elevated surfaceContainerLow circle at top center (y=16) + primary Arc painted AFTER child content
    state.refreshing.insert("srl".into(), true);
    let active = ui.frame(Input::default(), bounds, 0., |f| { SRL_NODE.render(f, bounds, &mut state); });
    let text_idx = active.iter().position(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Feed content")).unwrap();
    let shadow_idx = active.iter().position(|c| matches!(
        c,
        DrawCommand::Shadow { bounds: r, radius, .. }
            if (r.x - 140.0).abs() < 0.05 && (r.y - 16.0).abs() < 0.05 && (r.width - 40.0).abs() < 0.05 && (*radius - 20.0).abs() < 0.05
    )).expect("SwipeRefreshLayout must paint level-2 elevation shadow for 40dp circle");
    let circle_idx = active.iter().position(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xF7, 0xF2, 0xFA, 255) }
            if (r.x - 140.0).abs() < 0.05 && (r.y - 16.0).abs() < 0.05 && (r.width - 40.0).abs() < 0.05 && (r.height - 40.0).abs() < 0.05 && (*radius - 20.0).abs() < 0.05
    )).expect("SwipeRefreshLayout must paint 40dp surfaceContainerLow circle at y=b.y+16dp");
    let arc_idx = active.iter().position(|c| matches!(
        c,
        DrawCommand::Arc { color: Color(0x67, 0x50, 0xA4, 255), sweep, rounded: true, .. } if *sweep > 0.1
    )).expect("SwipeRefreshLayout must paint primary circular progress arc");
    assert!(text_idx < shadow_idx && shadow_idx < circle_idx && circle_idx < arc_idx, "Refresh indicator must paint cleanly over child content");

    // 3. Dark theme & 2x density scaling
    state.theme_style = AndroidTheme::Material3Dark;
    state.density = 2.0;
    let dark_2x = ui.frame(Input::default(), bounds, 0., |f| { SRL_NODE.render(f, bounds, &mut state); });
    assert!(dark_2x.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x1D, 0x1B, 0x20, 255) }
            if (r.x - 120.0).abs() < 0.05 && (r.y - 32.0).abs() < 0.05 && (r.width - 80.0).abs() < 0.05 && (*radius - 40.0).abs() < 0.05
    )));
    assert!(dark_2x.iter().any(|c| matches!(
        c,
        DrawCommand::Arc { color: Color(0xD0, 0xBC, 0xFF, 255), .. }
    )));
}

#[test]
fn drawer_layout_rule_hides_closed_drawer_shows_scrim_corners_rtl_and_dismisses() {
    use bhoo::android_xml::{AndroidTheme, RULE_DRAWER_LAYOUT, RULE_NAV_BAR, RULE_NAV_ITEM};

    static DL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_DRAWER_LAYOUT, anchor: "" }];
    static NAV_DRAWER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "drawer|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "d_inbox|Inbox|mail" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "d_settings|Settings|settings" },
    ];

    static DRAWER_HOST: Node = Node {
        kind: Kind::Frame,
        relative_rules: DL_RULES,
        children: &[
            Node {
                text: "Main Action",
                width: Size::Match,
                height: Size::Dp(48.),
                ..button("main_btn", Visibility::Visible, Size::Dp(48.), 0.)
            },
            Node {
                kind: Kind::Linear,
                width: Size::Wrap,
                height: Size::Match,
                layout_gravity: [0, 0],
                layout_gravity_explicit: true,
                relative_rules: NAV_DRAWER_RULES,
                ..button("nav_drawer", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("dl", Visibility::Visible, Size::Match, 0.)
    };

    static GRAVITY_DRAWER_HOST: Node = Node {
        kind: Kind::Frame,
        relative_rules: DL_RULES,
        children: &[
            Node {
                kind: Kind::Text,
                text: "Content",
                width: Size::Match,
                height: Size::Match,
                ..button("g_main", Visibility::Visible, Size::Match, 0.)
            },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(240.),
                height: Size::Match,
                layout_gravity: [2, 0], // end
                layout_gravity_explicit: true,
                children: &[Node {
                    kind: Kind::Text,
                    text: "End Panel",
                    ..button("end_label", Visibility::Visible, Size::Dp(40.), 0.)
                }],
                ..button("end_panel", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("dl_end", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 360., height: 300. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Closed by default: ONLY main content renders (no drawer items, no scrim), main button is interactive
    let closed_cmds = ui.frame(Input::default(), bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    assert!(closed_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Main Action")));
    assert!(!closed_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Inbox")));
    assert!(!closed_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0, 0, 0, 82), .. })));
    ui.frame(Input { pointer: Some([100., 24.]), pressed: true, ..Input::default() }, bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    let mut main_act = None;
    ui.frame(Input { pointer: Some([100., 24.]), released: true, ..Input::default() }, bounds, 0., |f| { main_act = DRAWER_HOST.render(f, bounds, &mut state); });
    assert_eq!(main_act, Some("main_btn"));

    // 2. Open drawer: scrim Color(0,0,0,82) over full bounds, drawer width=min(300, 360-56)=300,
    //    surfaceContainerLow #F7F2FA with [0, 16, 16, 0] end-rounded corners, drawer items interactive
    state.drawer_open.insert("dl".into(), true);
    let open_cmds = ui.frame(Input::default(), bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    assert!(open_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0, 0, 0, 82), .. }
            if *r == bounds
    )), "Open DrawerLayout must paint translucent scrim Color(0,0,0,82) over full bounds");
    assert!(open_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xF7, 0xF2, 0xFA, 255) }
            if (r.x + r.width - 300.0).abs() < 0.05 && (*radius - 16.0).abs() < 0.05
    )), "LTR start drawer must have 16dp end-rounded corners reaching x=300");
    assert!(open_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Settings")));

    // Clicking inside drawer selects destination (row 1 "Settings" is at y = 12 + 56 = 68..124) without closing drawer via scrim
    ui.frame(Input { pointer: Some([100., 96.]), pressed: true, ..Input::default() }, bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    let mut drawer_act = None;
    ui.frame(Input { pointer: Some([100., 96.]), released: true, ..Input::default() }, bounds, 0., |f| { drawer_act = DRAWER_HOST.render(f, bounds, &mut state); });
    assert_eq!(drawer_act, Some("d_settings"));
    assert_eq!(state.drawer_open.get("dl"), Some(&true));

    // Narrow viewport (320dp): default width clamps to 320 - 56 = 264dp
    let narrow = Rect { x: 0., y: 0., width: 320., height: 300. };
    let narrow_cmds = ui.frame(Input::default(), narrow, 0., |f| { DRAWER_HOST.render(f, narrow, &mut state); });
    assert!(narrow_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xF7, 0xF2, 0xFA, 255) }
            if (r.x + r.width - 264.0).abs() < 0.05 && (*radius - 16.0).abs() < 0.05
    )));

    // Clicking the scrim outside the drawer (x=330, outside 0..300) closes drawer_open
    ui.frame(Input { pointer: Some([330., 150.]), pressed: true, ..Input::default() }, bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    ui.frame(Input { pointer: Some([330., 150.]), released: true, ..Input::default() }, bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    assert_eq!(state.drawer_open.get("dl"), Some(&false));

    // 3. RTL: start drawer aligns to physical right (x=60..360), mirrors corners to [16, 0, 0, 16], scrim click on left (x=30) closes it
    state.rtl = true;
    state.drawer_open.insert("dl".into(), true);
    let rtl_open = ui.frame(Input::default(), bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    assert!(rtl_open.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xF7, 0xF2, 0xFA, 255) }
            if (r.x - 60.0).abs() < 0.05 && (*radius - 16.0).abs() < 0.05
    )), "RTL start drawer must align to physical right with 16dp left-rounded corners starting at x=60");
    ui.frame(Input { pointer: Some([30., 150.]), pressed: true, ..Input::default() }, bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    ui.frame(Input { pointer: Some([30., 150.]), released: true, ..Input::default() }, bounds, 0., |f| { DRAWER_HOST.render(f, bounds, &mut state); });
    assert_eq!(state.drawer_open.get("dl"), Some(&false));

    // 4. Non-NavigationView drawer child identified by explicit horizontal layout_gravity (end) + non-Match width
    state.rtl = false;
    state.theme_style = AndroidTheme::Material3Dark;
    let end_closed = ui.frame(Input::default(), bounds, 0., |f| { GRAVITY_DRAWER_HOST.render(f, bounds, &mut state); });
    assert!(!end_closed.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "End Panel")));
    state.drawer_open.insert("dl_end".into(), true);
    let end_open = ui.frame(Input::default(), bounds, 0., |f| { GRAVITY_DRAWER_HOST.render(f, bounds, &mut state); });
    assert!(end_open.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "End Panel")));
    assert!(end_open.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x1D, 0x1B, 0x20, 255) }
            if (r.x - 120.0).abs() < 0.05 && (*radius - 16.0).abs() < 0.05
    )), "End drawer (width 240 in 360dp bounds) must align at x=120 with 16dp left corners and dark surfaceContainerLow");
}

#[test]
fn collapsing_toolbar_interpolates_height_title_scrim_and_medium_variant_on_scroll() {
    use bhoo::android_xml::{
        AndroidShowAsAction, AndroidTheme, AndroidToolbarMenuItem, RULE_COLLAPSING_TOOLBAR,
        RULE_COORDINATOR_SCROLL_BEHAVIOR,
    };

    static LARGE_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_COLLAPSING_TOOLBAR,
        anchor: "Explore|152|243,237,247,255",
    }];
    static MEDIUM_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_COLLAPSING_TOOLBAR,
        anchor: "Library|112|",
    }];
    static SCROLL_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_COORDINATOR_SCROLL_BEHAVIOR,
        anchor: "",
    }];

    static LARGE_COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Frame,
                width: Size::Match,
                height: Size::Dp(152.),
                relative_rules: LARGE_RULES,
                children: &[Node {
                    kind: Kind::Toolbar,
                    image_src: "arrow_back",
                    text: "Fallback",
                    ..button("ct_tb", Visibility::Visible, Size::Dp(64.), 0.)
                }],
                ..button("ct_large", Visibility::Visible, Size::Dp(152.), 0.)
            },
            Node {
                kind: Kind::Scroll,
                relative_rules: SCROLL_RULES,
                children: &[Node {
                    kind: Kind::View,
                    background: Some(Color(20, 30, 40, 255)),
                    ..button("scroll_body", Visibility::Visible, Size::Dp(600.), 0.)
                }],
                ..button("feed_scroll", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("coord_root", Visibility::Visible, Size::Match, 0.)
    };

    static MEDIUM_HOST: Node = Node {
        kind: Kind::Frame,
        width: Size::Match,
        height: Size::Dp(112.),
        relative_rules: MEDIUM_RULES,
        children: &[Node {
            kind: Kind::Toolbar,
            ..button("med_tb", Visibility::Visible, Size::Dp(64.), 0.)
        }],
        ..button("ct_med", Visibility::Visible, Size::Dp(112.), 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 360., height: 400. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    state.toolbar_menu_items.insert(
        "ct_tb".into(),
        vec![AndroidToolbarMenuItem {
            id: 1,
            group_id: 0,
            label: "Search".into(),
            icon: Some(MaterialIcon::Search),
            icon_drawable: None,
            show_text_with_icon: false,
            show_as_action: AndroidShowAsAction::Always,
            enabled: true,
            visible: true,
            checkable: false,
            checked: false,
            exclusive: false,
        }],
    );

    // 1. Expanded (scroll = 0): 28sp title bottom-aligned, sibling scroll body starts at y=152, no scrim
    let expanded_cmds = ui.frame(Input::default(), bounds, 0., |f| {
        LARGE_COORD.render(f, bounds, &mut state);
    });
    let exp_title = expanded_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { text, bounds: r, style, .. } if text.as_ref() == "Explore" => {
                Some((*r, *style))
            }
            _ => None,
        })
        .expect("Expanded CollapsingToolbarLayout must draw its title");
    assert!((exp_title.1.size - 28.0).abs() < 0.05, "Large expanded title should be 28sp, got {}", exp_title.1.size);
    assert!(exp_title.0.y > 80.0, "Expanded title should sit bottom-aligned below 64dp bar, got y={}", exp_title.0.y);
    assert!((exp_title.0.x - 16.0).abs() < 0.5, "Expanded title starts at 16dp, got x={}", exp_title.0.x);
    assert!(
        !expanded_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Fallback")),
        "Child Toolbar must not draw duplicate title inside CollapsingToolbarLayout"
    );
    let body_y_exp = expanded_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, color: Color(20, 30, 40, 255), .. } => Some(r.y),
            _ => None,
        })
        .unwrap();
    assert!((body_y_exp - 152.0).abs() < 0.5, "Scroll sibling should start below 152dp expanded bar, got {body_y_exp}");
    assert!(
        !expanded_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(243, 237, 247, 255), .. })),
        "No scrim overlay when scroll == 0"
    );

    // Clicking nav icon inside child Toolbar works
    ui.frame(Input { pointer: Some([24., 32.]), pressed: true, ..Input::default() }, bounds, 0., |f| {
        LARGE_COORD.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([24., 32.]), released: true, ..Input::default() }, bounds, 0., |f| {
        LARGE_COORD.render(f, bounds, &mut state);
    });
    assert!(state.toolbar_navigation_clicked.contains("ct_tb"));

    // 2. Mid-collapse (scroll = 44dp = 50% of 88dp range): height=108dp, title=25sp, x/y interpolated, ~50% scrim
    state.offsets.insert("feed_scroll".into(), 44.0);
    let mid_cmds = ui.frame(Input::default(), bounds, 0., |f| {
        LARGE_COORD.render(f, bounds, &mut state);
    });
    let mid_title = mid_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { text, bounds: r, style, .. } if text.as_ref() == "Explore" => {
                Some((*r, *style))
            }
            _ => None,
        })
        .expect("Mid-collapsed title missing");
    assert!((mid_title.1.size - 25.0).abs() < 0.1, "Mid-collapse title size should be 25sp, got {}", mid_title.1.size);
    assert!(mid_title.0.y < exp_title.0.y && mid_title.0.y > 20.0);
    assert!(mid_title.0.x > 16.0 && mid_title.0.x < 56.0);
    assert!(mid_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(243, 237, 247, a), .. }
            if (r.height - 108.0).abs() < 0.5 && *a >= 120 && *a <= 136
    )));
    let body_y_mid = mid_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, color: Color(20, 30, 40, 255), .. } => Some(r.y),
            _ => None,
        })
        .unwrap();
    // Scroll child itself is shifted by -44 inside a scroll viewport starting at y=108, so clip y is 108 and child y is 64
    assert!((body_y_mid - 64.0).abs() < 0.5);
    assert!(mid_cmds.iter().any(|c| matches!(c, DrawCommand::PushClip(r) if (r.y - 108.0).abs() < 0.5)));

    // 3. Fully collapsed (scroll = 120dp >= 88dp): height=64dp, title=22sp centered in 64dp bar at x=56dp, opaque scrim
    state.offsets.insert("feed_scroll".into(), 120.0);
    let col_cmds = ui.frame(Input::default(), bounds, 0., |f| {
        LARGE_COORD.render(f, bounds, &mut state);
    });
    let col_title = col_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { text, bounds: r, style, .. } if text.as_ref() == "Explore" => {
                Some((*r, *style))
            }
            _ => None,
        })
        .expect("Collapsed title missing");
    assert!((col_title.1.size - 22.0).abs() < 0.05, "Collapsed title should be 22sp, got {}", col_title.1.size);
    assert!((col_title.0.x - 56.0).abs() < 0.5, "Collapsed title with nav icon must sit at x=56dp, got {}", col_title.0.x);
    assert!(col_title.0.y >= 10.0 && col_title.0.y <= 24.0, "Collapsed title must center in 64dp bar, got y={}", col_title.0.y);
    assert!(col_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(243, 237, 247, 255), .. }
            if (r.height - 64.0).abs() < 0.5
    )));

    // 4. RTL right-aligns title
    state.rtl = true;
    let rtl_cmds = ui.frame(Input::default(), bounds, 0., |f| {
        LARGE_COORD.render(f, bounds, &mut state);
    });
    let rtl_title = rtl_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { text, style, .. } if text.as_ref() == "Explore" => Some(*style),
            _ => None,
        })
        .unwrap();
    assert_eq!(rtl_title.align, 2);

    // 5. Medium variant (expanded_h = 112dp in 100..140): expanded title is 24sp, collapses to 22sp at scroll=48dp
    state.rtl = false;
    let med_b = Rect { x: 0., y: 0., width: 360., height: 112. };
    let med_exp = ui.frame(Input::default(), med_b, 0., |f| {
        MEDIUM_HOST.render(f, med_b, &mut state);
    });
    assert!(med_exp.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, style, .. } if text.as_ref() == "Library" && (style.size - 24.0).abs() < 0.05
    )));
    state.offsets.insert("ct_med".into(), 48.0);
    let med_col = ui.frame(Input::default(), med_b, 0., |f| {
        MEDIUM_HOST.render(f, med_b, &mut state);
    });
    assert!(med_col.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, style, .. } if text.as_ref() == "Library" && (style.size - 22.0).abs() < 0.05
    )));
}

#[test]
fn view_pager_rule_supports_horizontal_vertical_swipe_rtl_state_sync_and_dots() {
    use bhoo::android_xml::{AndroidTheme, RULE_VIEW_PAGER};

    static H_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_VIEW_PAGER, anchor: "horizontal" }];
    static V_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_VIEW_PAGER, anchor: "vertical" }];
    static PAGES: [Node; 3] = [
        Node { kind: Kind::Text, text: "Page 1", ..button("p1", Visibility::Visible, Size::Match, 0.) },
        Node { kind: Kind::Text, text: "Page 2", ..button("p2", Visibility::Visible, Size::Match, 0.) },
        Node { kind: Kind::Text, text: "Page 3", ..button("p3", Visibility::Visible, Size::Match, 0.) },
    ];
    static H_PAGER: Node = Node {
        kind: Kind::ViewFlipper,
        relative_rules: H_RULES,
        children: &PAGES,
        ..button("pager_h", Visibility::Visible, Size::Match, 0.)
    };
    static V_PAGER: Node = Node {
        kind: Kind::ViewFlipper,
        relative_rules: V_RULES,
        children: &PAGES,
        ..button("pager_v", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 300., height: 200. };
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Initial render: Page 1 + 3 bottom indicator dots (active 8dp primary leftmost in LTR)
    let init_cmds = ui.frame(Input::default(), bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    assert!(init_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page 1")));
    assert_eq!(state.displayed_children.get("pager_h"), Some(&0));
    assert_eq!(state.selected_index.get("pager_h"), Some(&0));
    let active_dot_ltr = init_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
                if (r.width - 8.0).abs() < 0.05 && (*radius - 4.0).abs() < 0.05 => Some(*r),
            _ => None,
        })
        .expect("Active 8dp primary page indicator dot missing");
    let inactive_dots_ltr: Vec<Rect> = init_cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, radius, .. }
                if (r.width - 6.0).abs() < 0.05 && (*radius - 3.0).abs() < 0.05 => Some(*r),
            _ => None,
        })
        .collect();
    assert_eq!(inactive_dots_ltr.len(), 2);
    assert!(inactive_dots_ltr.iter().all(|d| active_dot_ltr.x < d.x), "LTR page 0 dot must sit leftmost");

    // 2. LTR swipe left advances page 0 -> 1
    ui.frame(Input { pointer: Some([240., 100.]), pressed: true, time: 0.1, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([80., 100.]), time: 0.15, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    let mut act = None;
    let after_swipe = ui.frame(Input { pointer: Some([80., 100.]), released: true, time: 0.2, ..Input::default() }, bounds, 0., |f| {
        act = H_PAGER.render(f, bounds, &mut state);
    });
    assert_eq!(act, Some("pager_h"));
    assert_eq!(state.displayed_children["pager_h"], 1);
    assert_eq!(state.selected_index["pager_h"], 1);
    assert!(after_swipe.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page 2")));

    // 3. Programmatic state.selected_index update switches directly to page 2 and syncs displayed_children
    state.selected_index.insert("pager_h".into(), 2);
    let jumped = ui.frame(Input { time: 0.3, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    assert_eq!(state.displayed_children["pager_h"], 2);
    assert!(jumped.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page 3")));

    // 4. RTL: page 0 active dot is rightmost, swiping RIGHT advances 0 -> 1, swiping LEFT returns 1 -> 0
    state.rtl = true;
    state.displayed_children.insert("pager_h".into(), 0);
    let rtl_cmds = ui.frame(Input { time: 0.4, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    assert_eq!(state.selected_index["pager_h"], 0);
    let active_dot_rtl = rtl_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
                if (r.width - 8.0).abs() < 0.05 && (*radius - 4.0).abs() < 0.05 => Some(*r),
            _ => None,
        })
        .unwrap();
    let inactive_dots_rtl: Vec<Rect> = rtl_cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, radius, .. }
                if (r.width - 6.0).abs() < 0.05 && (*radius - 3.0).abs() < 0.05 => Some(*r),
            _ => None,
        })
        .collect();
    assert!(inactive_dots_rtl.iter().all(|d| active_dot_rtl.x > d.x), "RTL page 0 dot must sit rightmost");
    // Swipe right in RTL -> advance to page 1
    ui.frame(Input { pointer: Some([60., 100.]), pressed: true, time: 0.5, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([220., 100.]), time: 0.55, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([220., 100.]), released: true, time: 0.6, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    assert_eq!(state.selected_index["pager_h"], 1, "RTL swipe right must advance page");
    // Swipe left in RTL -> return to page 0
    ui.frame(Input { pointer: Some([220., 100.]), pressed: true, time: 0.7, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([60., 100.]), released: true, time: 0.75, ..Input::default() }, bounds, 0., |f| {
        H_PAGER.render(f, bounds, &mut state);
    });
    assert_eq!(state.selected_index["pager_h"], 0, "RTL swipe left must retreat page");

    // 5. Vertical ViewPager: swipe up advances 0 -> 1, swipe down retreats 1 -> 0
    state.rtl = false;
    ui.frame(Input { time: 0.8, ..Input::default() }, bounds, 0., |f| {
        V_PAGER.render(f, bounds, &mut state);
    });
    assert_eq!(state.selected_index["pager_v"], 0);
    ui.frame(Input { pointer: Some([150., 170.]), pressed: true, time: 0.85, ..Input::default() }, bounds, 0., |f| {
        V_PAGER.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([150., 40.]), released: true, time: 0.9, ..Input::default() }, bounds, 0., |f| {
        V_PAGER.render(f, bounds, &mut state);
    });
    assert_eq!(state.selected_index["pager_v"], 1, "Vertical swipe up must advance page");
    ui.frame(Input { pointer: Some([150., 40.]), pressed: true, time: 0.95, ..Input::default() }, bounds, 0., |f| {
        V_PAGER.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([150., 170.]), released: true, time: 1.0, ..Input::default() }, bounds, 0., |f| {
        V_PAGER.render(f, bounds, &mut state);
    });
    assert_eq!(state.selected_index["pager_v"], 0, "Vertical swipe down must retreat page");
}

#[test]
fn range_slider_rule_drags_both_thumbs_clamps_order_and_mirrors_rtl_and_density() {
    use bhoo::android_xml::{AndroidTheme, RULE_RANGE_SLIDER};

    static RANGE_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_RANGE_SLIDER,
        anchor: "20,80",
    }];
    static RANGE: Node = Node {
        kind: Kind::SeekBar,
        min: 0.,
        max: 100.,
        relative_rules: RANGE_RULES,
        ..button("price_range", Visibility::Visible, Size::Dp(48.), 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 222., height: 48. };
    // With pad = 11 on both ends, span_w = 200 (x = 11..211 => 2px per unit!)
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Initial seed [20, 80], active track spans 51..171 (width 120), two 20dp primary thumbs
    let cmds = ui.frame(Input::default(), bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    assert_eq!(state.range_values.get("price_range"), Some(&[20.0, 80.0]));
    let active_seg = cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, color: Color(0x67, 0x50, 0xA4, 255), .. }
                if (r.height - 4.0).abs() < 0.05 => Some(*r),
            _ => None,
        })
        .expect("Active range track segment missing");
    assert!((active_seg.x - 51.0).abs() < 0.5 && (active_seg.width - 120.0).abs() < 0.5);
    let thumbs: Vec<Rect> = cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
                if (r.width - 20.0).abs() < 0.05 && (*radius - 10.0).abs() < 0.05 => Some(*r),
            _ => None,
        })
        .collect();
    assert_eq!(thumbs.len(), 2);
    assert!((thumbs[0].x + 10.0 - 51.0).abs() < 0.5);
    assert!((thumbs[1].x + 10.0 - 171.0).abs() < 0.5);

    // 2. Drag lower thumb from 20 (x=51) to 40 (x=91)
    ui.frame(Input { pointer: Some([51., 24.]), pressed: true, time: 0.1, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    let mut act = None;
    ui.frame(Input { pointer: Some([91., 24.]), time: 0.15, ..Input::default() }, bounds, 0., |f| {
        act = RANGE.render(f, bounds, &mut state);
    });
    assert_eq!(act, Some("price_range"));
    assert_eq!(state.range_values["price_range"], [40.0, 80.0]);
    // Dragging lower thumb past upper thumb (x=191 -> 90) clamps v0 <= v1 (80)
    ui.frame(Input { pointer: Some([191., 24.]), time: 0.2, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    assert_eq!(state.range_values["price_range"], [80.0, 80.0]);
    // Bring lower thumb back to 30 (x=71) and release
    ui.frame(Input { pointer: Some([71., 24.]), time: 0.25, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([71., 24.]), released: true, time: 0.3, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    assert_eq!(state.range_values["price_range"], [30.0, 80.0]);

    // 3. Drag upper thumb from 80 (x=171) to 60 (x=131)
    ui.frame(Input { pointer: Some([171., 24.]), pressed: true, time: 0.4, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([131., 24.]), time: 0.45, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([131., 24.]), released: true, time: 0.5, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    assert_eq!(state.range_values["price_range"], [30.0, 60.0]);

    // 4. RTL: v0=30 sits on the RIGHT (x = 11 + 200*0.70 = 151), v1=60 sits on the LEFT (x = 11 + 200*0.40 = 91)
    state.rtl = true;
    let rtl_cmds = ui.frame(Input { time: 0.6, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    let rtl_thumbs: Vec<Rect> = rtl_cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
                if (r.width - 20.0).abs() < 0.05 && (*radius - 10.0).abs() < 0.05 => Some(*r),
            _ => None,
        })
        .collect();
    assert_eq!(rtl_thumbs.len(), 2);
    assert!((rtl_thumbs[0].x + 10.0 - 151.0).abs() < 0.5, "RTL thumb 0 should sit at x=151, got {}", rtl_thumbs[0].x + 10.0);
    assert!((rtl_thumbs[1].x + 10.0 - 91.0).abs() < 0.5, "RTL thumb 1 should sit at x=91, got {}", rtl_thumbs[1].x + 10.0);
    // Drag RTL upper thumb (left, x=91) further left to x=51 => frac = 0.2 => 1 - frac = 0.8 => v1 = 80
    ui.frame(Input { pointer: Some([91., 24.]), pressed: true, time: 0.7, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([51., 24.]), time: 0.75, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    ui.frame(Input { pointer: Some([51., 24.]), released: true, time: 0.8, ..Input::default() }, bounds, 0., |f| {
        RANGE.render(f, bounds, &mut state);
    });
    assert_eq!(state.range_values["price_range"], [30.0, 80.0]);

    // 5. Density = 2.0 scales thumb diameter to 40px and track height to 8px
    state.rtl = false;
    state.density = 2.0;
    let hi_b = Rect { x: 0., y: 0., width: 444., height: 96. };
    let hi_cmds = ui.frame(Input { time: 0.9, ..Input::default() }, hi_b, 0., |f| {
        RANGE.render(f, hi_b, &mut state);
    });
    let hi_thumbs = hi_cmds
        .iter()
        .filter(|c| matches!(
            c,
            DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
                if (r.width - 40.0).abs() < 0.05 && (*radius - 20.0).abs() < 0.05
        ))
        .count();
    assert_eq!(hi_thumbs, 2, "RangeSlider thumbs must scale to 40px at density=2.0");
    assert!(hi_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0x67, 0x50, 0xA4, 255) }
            if (r.height - 8.0).abs() < 0.05 && (*radius - 4.0).abs() < 0.05
    )));
}

#[test]
fn asymmetric_corner_radii_clips_state_layer_ripple_and_foreground() {
    use bhoo::android_xml::{AndroidTheme, RULE_BUTTON_VARIANT, RULE_CORNER_RADII};

    static TOP_CORNERS: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CORNER_RADII, anchor: "16,16,0,0" },
        RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "tonal" },
    ];
    static ASYM_BTN: Node = Node {
        kind: Kind::Button,
        text: "Top Rounded",
        foreground: Some(Color(220, 40, 60, 80)),
        foreground_stroke: Some(Color(20, 140, 60, 255)),
        foreground_stroke_width: 2.0,
        relative_rules: TOP_CORNERS,
        ..button("asym_btn", Visibility::Visible, Size::Dp(60.), 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 160., height: 60. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Foreground fill and foreground stroke honor asymmetric Corner.Top [16, 16, 0, 0]
    let idle = ui.frame(Input::default(), bounds, 0.0, |f| {
        ASYM_BTN.render(f, bounds, &mut state);
    });
    let fg_radii: Vec<f32> = idle
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { radius, color: Color(220, 40, 60, 80), .. } => Some(*radius),
            _ => None,
        })
        .collect();
    assert_eq!(fg_radii, vec![16.0, 0.0], "Foreground fill must split into 16dp top and 0dp bottom corners");
    let fg_stroke_radii: Vec<f32> = idle
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Outline { radius, color: Color(20, 140, 60, 255), .. } => Some(*radius),
            _ => None,
        })
        .collect();
    assert_eq!(fg_stroke_radii, vec![16.0, 0.0], "Foreground stroke must split into 16dp top and 0dp bottom corners");

    // 2. Hover state layer honors asymmetric [16, 16, 0, 0] (not uniform average 8dp)
    ui.frame(Input { pointer: Some([80., 30.]), time: 0.0, ..Input::default() }, bounds, 0., |f| {
        ASYM_BTN.render(f, bounds, &mut state);
    });
    let hovered = ui.frame(Input { pointer: Some([80., 30.]), time: 0.5, ..Input::default() }, bounds, 0., |f| {
        ASYM_BTN.render(f, bounds, &mut state);
    });
    // Tonal state_color is on_secondary_container Color(0x1D, 0x19, 0x2B, alpha ~20)
    let hover_radii: Vec<f32> = hovered
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { radius, color: Color(0x1D, 0x19, 0x2B, a), .. } if *a > 0 && *a < 40 => Some(*radius),
            _ => None,
        })
        .collect();
    assert_eq!(hover_radii, vec![16.0, 0.0], "Hover state layer must respect asymmetric top=16 / bottom=0 corners");

    // 3. Press ripple honors asymmetric [16, 16, 0, 0] via disjoint clipped DrawCommand::Ripple
    ui.frame(Input { pointer: Some([80., 30.]), pressed: true, time: 0.6, ..Input::default() }, bounds, 0., |f| {
        ASYM_BTN.render(f, bounds, &mut state);
    });
    let pressed = ui.frame(Input { pointer: Some([80., 30.]), time: 0.75, ..Input::default() }, bounds, 0., |f| {
        ASYM_BTN.render(f, bounds, &mut state);
    });
    let ripple_radii: Vec<f32> = pressed
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Ripple { corner_radius, .. } => Some(*corner_radius),
            _ => None,
        })
        .collect();
    assert_eq!(ripple_radii, vec![16.0, 0.0], "Press ripple must clip to top=16 / bottom=0 asymmetric outline");

    // 4. Density = 1.5 scales asymmetric foreground and ripple corners to [24, 24, 0, 0]
    state.density = 1.5;
    let hi_b = Rect { x: 0., y: 0., width: 240., height: 90. };
    let hi_cmds = ui.frame(Input { pointer: Some([120., 45.]), time: 0.8, ..Input::default() }, hi_b, 0., |f| {
        ASYM_BTN.render(f, hi_b, &mut state);
    });
    let hi_fg_radii: Vec<f32> = hi_cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { radius, color: Color(220, 40, 60, 80), .. } => Some(*radius),
            _ => None,
        })
        .collect();
    assert_eq!(hi_fg_radii, vec![24.0, 0.0]);
}

#[test]
fn horizontal_linear_wrap_height_distributes_weight_to_zero_dp_text_children() {
    use bhoo::android_xml::AndroidTheme;

    static SUBJECT_ROW: [Node; 3] = [
        Node {
            kind: Kind::Text,
            width: Size::Dp(0.0),
            weight: 1.0,
            text: "Photos from my trip!",
            text_size: 21.0,
            ..button("detail_subject", Visibility::Visible, Size::Wrap, 1.0)
        },
        Node {
            kind: Kind::Text,
            width: Size::Wrap,
            margin_relative: [Some(8.0), None],
            padding_relative: [Some(8.0), Some(8.0)],
            text: "Inbox",
            text_size: 11.0,
            content_gravity: [1, 1],
            ..button("detail_inbox_chip", Visibility::Visible, Size::Dp(22.0), 0.0)
        },
        Node {
            kind: Kind::Icon,
            width: Size::Dp(40.0),
            margin_relative: [Some(4.0), None],
            text: "Star message",
            ..button("detail_star", Visibility::Visible, Size::Dp(40.0), 0.0)
        },
    ];
    static SENDER_COL: [Node; 2] = [
        Node {
            kind: Kind::Text,
            width: Size::Wrap,
            text: "Sam 10:35 AM",
            text_size: 15.0,
            ..button("detail_sender", Visibility::Visible, Size::Wrap, 0.0)
        },
        Node {
            kind: Kind::Text,
            width: Size::Wrap,
            text: "to me",
            text_size: 13.0,
            ..button("detail_to", Visibility::Visible, Size::Wrap, 0.0)
        },
    ];
    static SENDER_ROW: [Node; 2] = [
        Node {
            kind: Kind::Frame,
            width: Size::Dp(40.0),
            ..button("avatar", Visibility::Visible, Size::Dp(40.0), 0.0)
        },
        Node {
            kind: Kind::Linear,
            width: Size::Dp(0.0),
            weight: 1.0,
            margin_relative: [Some(12.0), None],
            children: &SENDER_COL,
            ..button("sender_col", Visibility::Visible, Size::Wrap, 1.0)
        },
    ];
    static LONG_WRAP_ROW: [Node; 2] = [
        Node {
            kind: Kind::Text,
            width: Size::Dp(0.0),
            weight: 1.0,
            text: "This longer weighted subject line should wrap naturally to two lines at the distributed width",
            text_size: 16.0,
            ..button("long_subject", Visibility::Visible, Size::Wrap, 1.0)
        },
        Node {
            kind: Kind::View,
            width: Size::Dp(80.0),
            ..button("side_badge", Visibility::Visible, Size::Dp(24.0), 0.0)
        },
    ];
    static ROOT: Node = Node {
        kind: Kind::Linear,
        padding_edges: [16.0, 8.0, 16.0, 8.0],
        padding_explicit: [true; 4],
        children: &[
            Node {
                kind: Kind::Linear,
                horizontal: true,
                content_gravity: [0, 1],
                padding_edges: [0.0, 0.0, 0.0, 16.0],
                padding_explicit: [false, false, false, true],
                children: &SUBJECT_ROW,
                ..button("subject_row", Visibility::Visible, Size::Wrap, 0.0)
            },
            Node {
                kind: Kind::Linear,
                horizontal: true,
                content_gravity: [0, 1],
                padding_edges: [0.0, 0.0, 0.0, 16.0],
                padding_explicit: [false, false, false, true],
                children: &SENDER_ROW,
                ..button("sender_row", Visibility::Visible, Size::Wrap, 0.0)
            },
            Node {
                kind: Kind::Linear,
                horizontal: true,
                children: &LONG_WRAP_ROW,
                ..button("long_row", Visibility::Visible, Size::Wrap, 0.0)
            },
            button("after", Visibility::Visible, Size::Dp(40.0), 0.0),
        ],
        ..button("root", Visibility::Visible, Size::Match, 0.0)
    };

    let bounds = Rect { x: 0.0, y: 0.0, width: 585.0, height: 1266.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    state.density = 1.5;

    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        ROOT.render(f, bounds, &mut state);
    });

    // 1. "Photos from my trip!" must stay on 1 line with its distributed weighted width (not 1-char-per-line ~760px!)
    let subject_lines: Vec<Rect> = cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { bounds, text, .. } if text.contains("Photos") || text.as_ref() == "P" => Some(*bounds),
            _ => None,
        })
        .collect();
    assert_eq!(subject_lines.len(), 1, "weighted 0dp TextView must render on a single line, got {subject_lines:?}");
    assert!(
        subject_lines[0].width > 300.0 && subject_lines[0].height < 60.0,
        "weighted 0dp TextView must receive distributed width and compact 1-line height, got {:?}",
        subject_lines[0]
    );

    // 2. Sibling chip and star must sit in the compact top row (not pushed down to ~380px)
    let star_b = ui.semantics().iter().find(|n| n.label.as_ref() == "Star message").unwrap().bounds;
    assert!(star_b.y < 40.0, "star button must stay in compact top subject row, got y={}", star_b.y);

    // 3. Nested weighted column ("Sam 10:35 AM" / "to me") must also stay compact (not 1-char-per-line!)
    let sender_lines: Vec<&str> = cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { text, .. } if text.as_ref() == "Sam 10:35 AM" || text.as_ref() == "to me" => Some(text.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(sender_lines, vec!["Sam 10:35 AM", "to me"]);

    // 4. Longer weighted TextView must wrap naturally across 2-4 lines (not 1 char/line, not single clipped line)
    let long_lines: Vec<Rect> = cmds
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { bounds, text, .. }
                if text.contains("longer") || text.contains("weighted") || text.contains("distributed") =>
            {
                Some(*bounds)
            }
            _ => None,
        })
        .collect();
    assert!(
        (2..=4).contains(&long_lines.len()),
        "long weighted TextView should wrap naturally to 2..4 lines at distributed width, got {}",
        long_lines.len()
    );

    // 5. Button after all 3 rows must sit compactly below them (well under 320px, nowhere near 760px!)
    let after_b = ui.semantics().iter().find(|n| n.label.as_ref() == "after").unwrap().bounds;
    assert!(
        after_b.y > 180.0 && after_b.y < 320.0,
        "content below weighted horizontal rows must not be inflated, got after.y={}",
        after_b.y
    );
}

#[test]
fn multiline_edit_text_and_text_input_layout_wrap_newlines_and_clip_to_bounds() {
    use bhoo::android_xml::{AndroidTheme, RULE_TEXT_INPUT_BOX};

    static SINGLE_TIL_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_TEXT_INPUT_BOX,
        anchor: "outline|8.0|218,220,224,255|Subject||0|clear_text",
    }];
    static BODY_TIL_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_TEXT_INPUT_BOX,
        anchor: "outline|8.0|218,220,224,255|Compose email||0|none",
    }];
    static SINGLE_EDIT: [Node; 1] = [Node {
        kind: Kind::Edit,
        text: "Catch up soon?",
        hint: "Subject",
        text_size: 15.0,
        ..button("til_subject_edit", Visibility::Visible, Size::Dp(48.0), 0.0)
    }];
    static BODY_EDIT: [Node; 1] = [Node {
        kind: Kind::Edit,
        text: "",
        hint: "Compose email",
        text_size: 15.0,
        ..button("til_body_edit", Visibility::Visible, Size::Dp(220.0), 0.0)
    }];
    static SCREEN: Node = Node {
        kind: Kind::Linear,
        padding_edges: [16.0; 4],
        padding_explicit: [true; 4],
        children: &[
            Node {
                kind: Kind::Linear,
                relative_rules: SINGLE_TIL_RULES,
                children: &SINGLE_EDIT,
                ..button("til_subject", Visibility::Visible, Size::Wrap, 0.0)
            },
            Node {
                kind: Kind::Linear,
                margin: [0.0, 12.0, 0.0, 0.0],
                relative_rules: BODY_TIL_RULES,
                children: &BODY_EDIT,
                ..button("til_body", Visibility::Visible, Size::Wrap, 0.0)
            },
            Node {
                kind: Kind::Edit,
                margin: [0.0, 12.0, 0.0, 0.0],
                text: "One\nTwo\nThree\nFour\nFive\nSix\nSeven\nEight",
                hint: "Notes",
                text_size: 15.0,
                min_lines: 3,
                ..button("standalone_notes", Visibility::Visible, Size::Dp(96.0), 0.0)
            },
        ],
        ..button("compose_root", Visibility::Visible, Size::Match, 0.0)
    };

    let bounds = Rect { x: 0.0, y: 0.0, width: 360.0, height: 540.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    state.text.insert(
        "til_body_edit".into(),
        "Hey Sam,\n\nHow's it going? I attached some pictures from my recent trip to Japan!\n\nBest,\nAlex".into(),
    );

    let cmds = ui.frame(Input::default(), bounds, 0.0, |f| {
        SCREEN.render(f, bounds, &mut state);
    });

    // 1. Single-line TextInputLayout keeps its single centered line
    let subj_bounds = cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { bounds, text, .. } if text.as_ref() == "Catch up soon?" => Some(*bounds),
            _ => None,
        })
        .expect("single-line TextInputLayout must render Subject text");
    assert!(
        (subj_bounds.height - 56.0).abs() < 1.0,
        "single-line TextInputLayout should vertically center in its 56dp box, got {subj_bounds:?}"
    );

    // 2. Multiline TextInputLayout preserves explicit newlines, blank lines, wrapping, and top alignment
    let body_field = ui.semantics().iter().find(|n| n.label.as_ref() == "Compose email").unwrap().bounds;
    let mut body_lines: Vec<(Rect, &str)> = Vec::new();
    let mut active_clips: Vec<Rect> = Vec::new();
    let mut body_clipped = false;
    for cmd in &cmds {
        match cmd {
            DrawCommand::PushClip(r) => active_clips.push(*r),
            DrawCommand::PopClip => {
                active_clips.pop();
            }
            DrawCommand::Text { bounds, text, .. }
                if bounds.y >= body_field.y
                    && bounds.y < body_field.y + body_field.height
                    && text.as_ref() != "Compose email" =>
            {
                body_lines.push((*bounds, text.as_ref()));
                if let Some(clip) = active_clips.last() {
                    if clip.x >= body_field.x
                        && clip.y >= body_field.y
                        && clip.x + clip.width <= body_field.x + body_field.width + 0.01
                        && clip.y + clip.height <= body_field.y + body_field.height + 0.01
                    {
                        body_clipped = true;
                    }
                }
            }
            _ => {}
        }
    }
    assert!(body_clipped, "multiline TextInputLayout text must be clipped within its box");
    assert!(
        body_lines.len() >= 5,
        "multiline TextInputLayout must split paragraphs and wrap lines, got {body_lines:?}"
    );
    assert_eq!(body_lines[0].1, "Hey Sam,");
    assert!(
        body_lines[0].0.y - body_field.y <= 20.0,
        "multiline TextInputLayout must be top-aligned, got offset {}",
        body_lines[0].0.y - body_field.y
    );
    // Blank line between "Hey Sam," and "How's it going?..." leaves a >1.5x line gap
    let line_h = body_lines[0].0.height;
    assert!(
        body_lines[1].0.y - body_lines[0].0.y >= line_h * 1.8,
        "explicit blank line must preserve vertical paragraph spacing: {:?} -> {:?}",
        body_lines[0],
        body_lines[1]
    );
    assert!(body_lines.iter().any(|(_, t)| *t == "Best,"));
    assert!(body_lines.iter().any(|(_, t)| *t == "Alex"));

    // 3. Standalone multiline EditText wraps across lines, uses full 96dp height (not clamped to 48dp), and clips overflow
    let notes_field = ui.semantics().iter().find(|n| n.label.as_ref() == "Notes").unwrap().bounds;
    assert!(
        (notes_field.height - 96.0).abs() < 0.5,
        "standalone multiline EditText must use its full 96dp height, got {}",
        notes_field.height
    );
    let mut notes_lines: Vec<(Rect, &str)> = Vec::new();
    let mut notes_clipped = false;
    active_clips.clear();
    for cmd in &cmds {
        match cmd {
            DrawCommand::PushClip(r) => active_clips.push(*r),
            DrawCommand::PopClip => {
                active_clips.pop();
            }
            DrawCommand::Text { bounds, text, .. }
                if bounds.y >= notes_field.y && bounds.y <= notes_field.y + notes_field.height + 20.0 && text.as_ref() != "Notes" =>
            {
                notes_lines.push((*bounds, text.as_ref()));
                if let Some(clip) = active_clips.last() {
                    if clip.y >= notes_field.y && clip.y + clip.height <= notes_field.y + notes_field.height + 0.01 {
                        notes_clipped = true;
                    }
                }
            }
            _ => {}
        }
    }
    assert!(notes_clipped, "standalone multiline EditText must push clip rect inside its bounds");
    assert!(
        notes_lines.len() >= 3 && notes_lines.len() < 8,
        "overflowing lines in 96dp EditText should stop at height limit rather than bleeding out, got {notes_lines:?}"
    );
    assert_eq!(notes_lines[0].1, "One");
    assert_eq!(notes_lines[1].1, "Two");
    assert_eq!(notes_lines[2].1, "Three");
}

#[test]
fn image_button_centers_small_icon_without_upscaling_past_intrinsic_size() {
    static ICON_BTN: Node = Node {
        kind: Kind::Icon,
        width: Size::Dp(44.0),
        image_src: "ic_24",
        image_builtin: false,
        image_scale: 0,
        ..button("icon_btn", Visibility::Visible, Size::Dp(44.0), 0.0)
    };
    static ICON_FIT_XY: Node = Node {
        kind: Kind::Icon,
        width: Size::Dp(44.0),
        image_src: "ic_24",
        image_builtin: false,
        image_scale: 2,
        ..button("icon_fitxy", Visibility::Visible, Size::Dp(44.0), 0.0)
    };
    static ICON_PADDED: Node = Node {
        kind: Kind::Icon,
        width: Size::Dp(44.0),
        image_src: "ic_24",
        image_builtin: false,
        image_scale: 0,
        padding_edges: [4.0; 4],
        padding_explicit: [true; 4],
        ..button("icon_padded", Visibility::Visible, Size::Dp(44.0), 0.0)
    };
    static IMAGE_VIEW: Node = Node {
        kind: Kind::Image,
        width: Size::Dp(44.0),
        image_src: "ic_24",
        image_builtin: false,
        image_scale: 0,
        ..button("img_view", Visibility::Visible, Size::Dp(44.0), 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.images.insert("ic_24".into(), Image::rgba(24, 24, vec![255; 24 * 24 * 4]).unwrap());
    state.image_scales.insert("ic_24".into(), 1.0);
    state.images.insert("ic_big".into(), Image::rgba(96, 96, vec![255; 96 * 96 * 4]).unwrap());
    state.image_scales.insert("ic_big".into(), 1.0);

    // 1. Default ImageButton at 1.0x centers 24x24 icon inside 44x44 touch target without upscaling
    let b1 = Rect { x: 0.0, y: 0.0, width: 44.0, height: 44.0 };
    let cmds = ui.frame(Input::default(), b1, 0.0, |f| { ICON_BTN.render(f, b1, &mut state); });
    assert!(
        cmds.iter().any(|c| matches!(c, DrawCommand::Image { bounds, .. } if *bounds == Rect { x: 10.0, y: 10.0, width: 24.0, height: 24.0 })),
        "24x24 icon in 44x44 ImageButton must stay 24x24 centered at (10,10), got {cmds:?}"
    );

    // 2. Default ImageButton at 1.5x density scales 24dp -> 36px centered inside 66x66 target
    state.density = 1.5;
    let b15 = Rect { x: 0.0, y: 0.0, width: 66.0, height: 66.0 };
    let cmds15 = ui.frame(Input::default(), b15, 0.0, |f| { ICON_BTN.render(f, b15, &mut state); });
    assert!(
        cmds15.iter().any(|c| matches!(c, DrawCommand::Image { bounds, .. } if *bounds == Rect { x: 15.0, y: 15.0, width: 36.0, height: 36.0 })),
        "24dp icon at 1.5x in 66x66 ImageButton must render 36x36 centered at (15,15), got {cmds15:?}"
    );

    // 3. Oversized 96x96 drawable inside 44x44 ImageButton still scales down cleanly (centerInside)
    state.density = 1.0;
    state.image_sources.insert("icon_btn".into(), "ic_big".into());
    let cmds_big = ui.frame(Input::default(), b1, 0.0, |f| { ICON_BTN.render(f, b1, &mut state); });
    assert!(
        cmds_big.iter().any(|c| matches!(c, DrawCommand::Image { bounds, .. } if *bounds == b1)),
        "oversized drawable in ImageButton must scale down to fit bounds"
    );
    state.image_sources.remove("icon_btn");

    // 4. Explicit fitXY (scale=2) on ImageButton, explicit padding on ImageButton, and ImageView (Kind::Image) remain unchanged
    let cmds_xy = ui.frame(Input::default(), b1, 0.0, |f| { ICON_FIT_XY.render(f, b1, &mut state); });
    assert!(cmds_xy.iter().any(|c| matches!(c, DrawCommand::Image { bounds, .. } if *bounds == b1)));

    let cmds_pad = ui.frame(Input::default(), b1, 0.0, |f| { ICON_PADDED.render(f, b1, &mut state); });
    assert!(cmds_pad.iter().any(|c| matches!(c, DrawCommand::Image { bounds, .. } if *bounds == Rect { x: 4.0, y: 4.0, width: 36.0, height: 36.0 })));

    let cmds_img = ui.frame(Input::default(), b1, 0.0, |f| { IMAGE_VIEW.render(f, b1, &mut state); });
    assert!(cmds_img.iter().any(|c| matches!(c, DrawCommand::Image { bounds, .. } if *bounds == b1)));
}

#[test]
fn wave14a_background_image_nine_patch_slicing_tint_and_corner_radii() {
    use bhoo::android_xml::{
        RULE_BACKGROUND_IMAGE, RULE_BACKGROUND_TINT, RULE_CORNER_RADII,
    };

    static NINE_PATCH_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BACKGROUND_IMAGE, anchor: "card_9p|4,16;6,14" },
    ];
    static NINE_PATCH_VIEW: Node = Node {
        kind: Kind::Text,
        text: "Bubble",
        relative_rules: NINE_PATCH_RULES,
        ..button("np_bg", Visibility::Visible, Size::Match, 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.images.insert("card_9p".into(), Image::rgba(20, 20, vec![255; 20 * 20 * 4]).unwrap());
    state.image_scales.insert("card_9p".into(), 1.0);

    // 1. Density 1.0x into 200x100 -> 9 tiles; corners stay 4x6, center fills 192x88.
    let dst = Rect { x: 10.0, y: 20.0, width: 200.0, height: 100.0 };
    let cmds = ui.frame(Input::default(), Rect { x: 0.0, y: 0.0, width: 300.0, height: 200.0 }, 0.0, |f| {
        NINE_PATCH_VIEW.render(f, dst, &mut state);
    });
    let tiles: Vec<(usize, Rect, [f32; 4])> = cmds.iter().enumerate().filter_map(|(i, c)| match c {
        DrawCommand::Image { bounds, uv, .. } => Some((i, *bounds, *uv)),
        _ => None,
    }).collect();
    assert_eq!(tiles.len(), 9, "expected 9 nine-patch tiles, got {tiles:?}");
    let text_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Bubble")).unwrap();
    assert!(tiles.iter().all(|(i, _, _)| *i < text_idx), "background 9-patch tiles must paint behind view text");
    // Top-left fixed corner: (10,20,4,6), uv [0,0,0.2,0.3]
    assert_eq!(tiles[0].1, Rect { x: 10.0, y: 20.0, width: 4.0, height: 6.0 });
    assert!((tiles[0].2[2] - 0.2).abs() < 1e-5 && (tiles[0].2[3] - 0.3).abs() < 1e-5);
    // Center stretch tile: (14,26,192,88), uv [0.2,0.3,0.8,0.7]
    assert_eq!(tiles[4].1, Rect { x: 14.0, y: 26.0, width: 192.0, height: 88.0 });
    assert!((tiles[4].2[0] - 0.2).abs() < 1e-5 && (tiles[4].2[1] - 0.3).abs() < 1e-5);
    assert!((tiles[4].2[2] - 0.8).abs() < 1e-5 && (tiles[4].2[3] - 0.7).abs() < 1e-5);
    // Bottom-right fixed corner: (206,114,4,6), uv [0.8,0.7,1.0,1.0]
    assert_eq!(tiles[8].1, Rect { x: 206.0, y: 114.0, width: 4.0, height: 6.0 });

    // 2. Density 2.0x -> fixed corners scale by density to 8x12, center shrinks to 184x76.
    state.density = 2.0;
    let cmds2 = ui.frame(Input::default(), Rect { x: 0.0, y: 0.0, width: 300.0, height: 200.0 }, 0.0, |f| {
        NINE_PATCH_VIEW.render(f, dst, &mut state);
    });
    let tiles2: Vec<Rect> = cmds2.iter().filter_map(|c| match c {
        DrawCommand::Image { bounds, .. } => Some(*bounds),
        _ => None,
    }).collect();
    assert_eq!(tiles2.len(), 9);
    assert_eq!(tiles2[0], Rect { x: 10.0, y: 20.0, width: 8.0, height: 12.0 });
    assert_eq!(tiles2[4], Rect { x: 18.0, y: 32.0, width: 184.0, height: 76.0 });
    assert_eq!(tiles2[8], Rect { x: 202.0, y: 108.0, width: 8.0, height: 12.0 });

    // 3. Constrained small destination (6x8 < fixed 8x12 at density 1.0) scales corners down without center overlap.
    state.density = 1.0;
    let tiny = Rect { x: 0.0, y: 0.0, width: 6.0, height: 8.0 };
    let cmds_tiny = ui.frame(Input::default(), Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 }, 0.0, |f| {
        NINE_PATCH_VIEW.render(f, tiny, &mut state);
    });
    let tiny_tiles: Vec<Rect> = cmds_tiny.iter().filter_map(|c| match c {
        DrawCommand::Image { bounds, .. } => Some(*bounds),
        _ => None,
    }).collect();
    assert_eq!(tiny_tiles.len(), 4, "zero-sized stretch strips should be skipped when dst < fixed borders");
    assert!((tiny_tiles[0].width - 3.0).abs() < 1e-5 && (tiny_tiles[0].height - 4.0).abs() < 1e-5);

    // 4. Plain bitmap background image with RULE_BACKGROUND_TINT + radius & asymmetric RULE_CORNER_RADII.
    static TINTED_ROUND_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BACKGROUND_IMAGE, anchor: "plain_bg|" },
        RelativeRule { kind: RULE_BACKGROUND_TINT, anchor: "220,40,60,255" },
    ];
    static TINTED_ROUND_VIEW: Node = Node {
        kind: Kind::View,
        radius: 12.0,
        relative_rules: TINTED_ROUND_RULES,
        ..button("bg_round", Visibility::Visible, Size::Match, 0.0)
    };
    static ASYM_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BACKGROUND_IMAGE, anchor: "plain_bg|" },
        RelativeRule { kind: RULE_CORNER_RADII, anchor: "16,16,0,0" },
    ];
    static ASYM_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: ASYM_RULES,
        ..button("bg_asym", Visibility::Visible, Size::Match, 0.0)
    };

    state.images.insert("plain_bg".into(), Image::rgba(8, 8, vec![255; 8 * 8 * 4]).unwrap());
    let b = Rect { x: 0.0, y: 0.0, width: 120.0, height: 60.0 };
    let cmds_r = ui.frame(Input::default(), b, 0.0, |f| { TINTED_ROUND_VIEW.render(f, b, &mut state); });
    // No solid RoundedRect painted behind the background image!
    assert!(!cmds_r.iter().any(|c| matches!(c, DrawCommand::RoundedRect { .. })));
    let img_cmd = cmds_r.iter().find_map(|c| match c {
        DrawCommand::Image { bounds, radius, image, .. } => Some((*bounds, *radius, image.clone())),
        _ => None,
    }).expect("expected background DrawCommand::Image");
    assert_eq!(img_cmd.0, b);
    assert!((img_cmd.1 - 12.0).abs() < 1e-4, "expected uniform corner radius 12.0 on background image, got {}", img_cmd.1);
    assert_eq!(&img_cmd.2.pixels()[..4], &[220, 40, 60, 255], "expected RULE_BACKGROUND_TINT applied via SRC_IN");

    let cmds_asym = ui.frame(Input::default(), b, 0.0, |f| { ASYM_VIEW.render(f, b, &mut state); });
    let radii_emitted: Vec<f32> = cmds_asym.iter().filter_map(|c| match c {
        DrawCommand::Image { radius, .. } => Some(*radius),
        _ => None,
    }).collect();
    assert_eq!(radii_emitted, vec![16.0, 0.0], "asymmetric RULE_CORNER_RADII must clip top vs bottom halves cleanly");
}

#[test]
fn wave14a_stateful_bg_image_evaluates_state_mask_and_falls_back() {
    use bhoo::android_xml::{RULE_BACKGROUND_IMAGE, RULE_STATEFUL_BG_IMAGE};

    static RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_STATEFUL_BG_IMAGE, anchor: "2,0|bg_pressed|2,14;2,14" },
        RelativeRule { kind: RULE_STATEFUL_BG_IMAGE, anchor: "64,0|bg_checked|" },
        RelativeRule { kind: RULE_STATEFUL_BG_IMAGE, anchor: "8,0|bg_selected|" },
        RelativeRule { kind: RULE_STATEFUL_BG_IMAGE, anchor: "0,1|bg_disabled|" },
        RelativeRule { kind: RULE_BACKGROUND_IMAGE, anchor: "bg_fallback|" },
    ];
    static BTN: Node = Node {
        kind: Kind::ToggleButton,
        text_on: "ON",
        text_off: "OFF",
        relative_rules: RULES,
        ..button("state_bg_btn", Visibility::Visible, Size::Match, 0.0)
    };
    static DISABLED_BTN: Node = Node { enabled: false, ..BTN };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let solid = |r, g, b| Image::rgba(16, 16, [r, g, b, 255].repeat(16 * 16)).unwrap();
    state.images.insert("bg_pressed".into(), solid(255, 0, 0));
    state.images.insert("bg_checked".into(), solid(0, 255, 0));
    state.images.insert("bg_selected".into(), solid(0, 0, 255));
    state.images.insert("bg_disabled".into(), solid(120, 120, 120));
    state.images.insert("bg_fallback".into(), solid(200, 200, 0));

    let b = Rect { x: 0.0, y: 0.0, width: 120.0, height: 48.0 };

    // Default enabled (none of pressed/checked/selected/disabled match) -> falls back to RULE_BACKGROUND_IMAGE (251)
    let cmds = ui.frame(Input::default(), b, 0.0, |f| { BTN.render(f, b, &mut state); });
    let imgs: Vec<&Image> = cmds.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image), _ => None }).collect();
    assert_eq!(imgs.len(), 1);
    assert_eq!(&imgs[0].pixels()[..4], &[200, 200, 0, 255]);

    // Pressed -> matches 2,0|bg_pressed|2,14;2,14 (9-patch -> 9 tiles of red!)
    let cmds_p = ui.frame(Input { pointer: Some([40.0, 20.0]), pressed: true, ..Input::default() }, b, 0.0, |f| {
        BTN.render(f, b, &mut state);
    });
    let imgs_p: Vec<&Image> = cmds_p.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image), _ => None }).collect();
    assert_eq!(imgs_p.len(), 9, "pressed 9-patch background should emit 9 sliced tiles");
    assert_eq!(&imgs_p[0].pixels()[..4], &[255, 0, 0, 255]);
    ui.frame(Input { pointer: Some([200.0, 200.0]), released: true, ..Input::default() }, b, 0.0, |f| {
        BTN.render(f, b, &mut state);
    });

    // Selected -> matches 8,0|bg_selected|
    state.view_selected.insert("state_bg_btn".into(), true);
    let cmds_s = ui.frame(Input::default(), b, 0.0, |f| { BTN.render(f, b, &mut state); });
    let imgs_s: Vec<&Image> = cmds_s.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image), _ => None }).collect();
    assert_eq!(imgs_s.len(), 1);
    assert_eq!(&imgs_s[0].pixels()[..4], &[0, 0, 255, 255]);

    // Checked beats selected because 64,0 appears earlier in selector order
    state.checked.insert("state_bg_btn".into(), true);
    let cmds_c = ui.frame(Input::default(), b, 0.0, |f| { BTN.render(f, b, &mut state); });
    let imgs_c: Vec<&Image> = cmds_c.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image), _ => None }).collect();
    assert_eq!(imgs_c.len(), 1);
    assert_eq!(&imgs_c[0].pixels()[..4], &[0, 255, 0, 255]);

    // Disabled (forbid enabled bit 1) -> matches 0,1|bg_disabled|
    state.checked.insert("state_bg_btn".into(), false);
    state.view_selected.insert("state_bg_btn".into(), false);
    let cmds_d = ui.frame(Input::default(), b, 0.0, |f| { DISABLED_BTN.render(f, b, &mut state); });
    let imgs_d: Vec<&Image> = cmds_d.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image), _ => None }).collect();
    assert_eq!(imgs_d.len(), 1);
    assert_eq!(&imgs_d[0].pixels()[..4], &[120, 120, 120, 255]);
}

#[test]
fn wave14a_image_nine_patch_and_stateful_src_on_image_and_icon() {
    use bhoo::android_xml::{RULE_IMAGE_NINE_PATCH, RULE_STATEFUL_IMAGE_SRC};

    static NP_IMG_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_IMAGE_NINE_PATCH, anchor: "3,13;5,11" },
    ];
    static NP_IMG: Node = Node {
        kind: Kind::Image,
        image_src: "badge_9p",
        image_alpha: 200,
        relative_rules: NP_IMG_RULES,
        ..button("np_img", Visibility::Visible, Size::Match, 0.0)
    };
    static STATEFUL_ICON_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_STATEFUL_IMAGE_SRC, anchor: "2,0|ic_pressed|2,14;2,14" },
        RelativeRule { kind: RULE_STATEFUL_IMAGE_SRC, anchor: "8,0|ic_selected|" },
        RelativeRule { kind: RULE_STATEFUL_IMAGE_SRC, anchor: "0,0|ic_normal|" },
    ];
    static STATEFUL_ICON: Node = Node {
        kind: Kind::Icon,
        image_src: "",
        image_builtin: false,
        relative_rules: STATEFUL_ICON_RULES,
        ..button("state_icon", Visibility::Visible, Size::Match, 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let solid = |r, g, b| Image::rgba(16, 16, [r, g, b, 255].repeat(16 * 16)).unwrap();
    state.images.insert("badge_9p".into(), solid(10, 20, 30));
    state.images.insert("ic_pressed".into(), solid(250, 10, 10));
    state.images.insert("ic_selected".into(), solid(10, 250, 10));
    state.images.insert("ic_normal".into(), solid(10, 10, 250));

    // 1. ImageView with RULE_IMAGE_NINE_PATCH renders 9 tiles with preserved corners and image_alpha
    let b = Rect { x: 0.0, y: 0.0, width: 100.0, height: 60.0 };
    let cmds = ui.frame(Input::default(), b, 0.0, |f| { NP_IMG.render(f, b, &mut state); });
    let tiles: Vec<(Rect, Color)> = cmds.iter().filter_map(|c| match c {
        DrawCommand::Image { bounds, color, .. } => Some((*bounds, *color)),
        _ => None,
    }).collect();
    assert_eq!(tiles.len(), 9);
    assert_eq!(tiles[0].0, Rect { x: 0.0, y: 0.0, width: 3.0, height: 5.0 });
    assert_eq!(tiles[4].0, Rect { x: 3.0, y: 5.0, width: 94.0, height: 50.0 });
    assert_eq!(tiles[0].1.3, 200);

    // 2. ImageButton with RULE_STATEFUL_IMAGE_SRC switches between normal, selected, and pressed 9-patch
    let ib = Rect { x: 0.0, y: 0.0, width: 48.0, height: 48.0 };
    let cmds_n = ui.frame(Input::default(), ib, 0.0, |f| { STATEFUL_ICON.render(f, ib, &mut state); });
    let imgs_n: Vec<&Image> = cmds_n.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image), _ => None }).collect();
    assert_eq!(imgs_n.len(), 1);
    assert_eq!(&imgs_n[0].pixels()[..4], &[10, 10, 250, 255]);

    state.view_selected.insert("state_icon".into(), true);
    let cmds_sel = ui.frame(Input::default(), ib, 0.0, |f| { STATEFUL_ICON.render(f, ib, &mut state); });
    let imgs_sel: Vec<&Image> = cmds_sel.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image), _ => None }).collect();
    assert_eq!(imgs_sel.len(), 1);
    assert_eq!(&imgs_sel[0].pixels()[..4], &[10, 250, 10, 255]);

    let cmds_pr = ui.frame(Input { pointer: Some([24.0, 24.0]), pressed: true, ..Input::default() }, ib, 0.0, |f| {
        STATEFUL_ICON.render(f, ib, &mut state);
    });
    let imgs_pr: Vec<&Image> = cmds_pr.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image), _ => None }).collect();
    assert_eq!(imgs_pr.len(), 9, "pressed stateful 9-patch src should render 9 sliced tiles");
    assert_eq!(&imgs_pr[0].pixels()[..4], &[250, 10, 10, 255]);
}

#[test]
fn wave14a_auto_mirrored_flips_image_icon_and_toolbar_nav_in_rtl_only() {
    use bhoo::android_xml::RULE_AUTO_MIRRORED;

    static MIRROR_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_AUTO_MIRRORED, anchor: "true" },
    ];
    static IMG: Node = Node {
        kind: Kind::Image,
        image_src: "arrow",
        relative_rules: MIRROR_RULES,
        ..button("am_img", Visibility::Visible, Size::Dp(40.0), 0.0)
    };
    static ICON: Node = Node {
        kind: Kind::Icon,
        image_src: "arrow",
        image_builtin: false,
        relative_rules: MIRROR_RULES,
        ..button("am_icon", Visibility::Visible, Size::Dp(48.0), 0.0)
    };
    static TOOLBAR: Node = Node {
        kind: Kind::Toolbar,
        text: "Title",
        image_src: "arrow",
        relative_rules: MIRROR_RULES,
        ..button("am_tb", Visibility::Visible, Size::Dp(56.0), 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.images.insert("arrow".into(), Image::rgba(24, 24, vec![255; 24 * 24 * 4]).unwrap());

    let b_img = Rect { x: 10.0, y: 20.0, width: 40.0, height: 40.0 };
    let b_tb = Rect { x: 0.0, y: 0.0, width: 320.0, height: 56.0 };

    // LTR: no PushTransform emitted for any of the 3
    for (node, rect) in [(&IMG, b_img), (&ICON, b_img), (&TOOLBAR, b_tb)] {
        let cmds = ui.frame(Input::default(), Rect { x: 0.0, y: 0.0, width: 320.0, height: 120.0 }, 0.0, |f| {
            node.render(f, rect, &mut state);
        });
        assert!(!cmds.iter().any(|c| matches!(c, DrawCommand::PushTransform { .. })), "LTR must not mirror");
    }

    // RTL: every one wraps its icon draw in PushTransform { scale: [-1.0, 1.0], rotation_rad: 0.0 } / PopTransform
    state.rtl = true;
    for (node, rect, name) in [(&IMG, b_img, "Image"), (&ICON, b_img, "Icon"), (&TOOLBAR, b_tb, "Toolbar")] {
        let cmds = ui.frame(Input::default(), Rect { x: 0.0, y: 0.0, width: 320.0, height: 120.0 }, 0.0, |f| {
            node.render(f, rect, &mut state);
        });
        let push_idx = cmds.iter().position(|c| matches!(
            c,
            DrawCommand::PushTransform { translate, scale, rotation_rad, .. }
                if *translate == [0.0, 0.0] && *scale == [-1.0, 1.0] && *rotation_rad == 0.0
        )).unwrap_or_else(|| panic!("{name} missing RTL autoMirrored PushTransform: {cmds:?}"));
        let img_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Image { .. })).unwrap();
        let pop_idx = cmds.iter().position(|c| matches!(c, DrawCommand::PopTransform)).unwrap();
        assert!(push_idx < img_idx && img_idx < pop_idx, "{name}: PushTransform must wrap icon draw");
    }
}

#[test]
fn wave14a_view_transform_and_push_layer_compositing() {
    use bhoo::android_xml::RULE_VIEW_TRANSFORM;

    static CENTER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_VIEW_TRANSFORM, anchor: "12,-6,1.5,0.5,45,center,center" },
    ];
    static DP_PIVOT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_VIEW_TRANSFORM, anchor: "0,0,2.0,2.0,90,10,5" },
    ];
    static PCT_PIVOT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_VIEW_TRANSFORM, anchor: "0,0,1.0,1.0,30,25%,75%" },
    ];
    static TRANSFORMED_GROUP: Node = Node {
        kind: Kind::Frame,
        view_alpha: 0.6,
        background: Some(Color(20, 40, 60, 255)),
        foreground: Some(Color(255, 255, 255, 128)),
        relative_rules: CENTER_RULES,
        children: &[Node {
            kind: Kind::View,
            background: Some(Color(200, 100, 50, 255)),
            ..button("child_v", Visibility::Visible, Size::Match, 0.0)
        }],
        ..button("tr_group", Visibility::Visible, Size::Match, 0.0)
    };
    static DP_PIVOT_VIEW: Node = Node {
        kind: Kind::View,
        background: Some(Color(1, 2, 3, 255)),
        relative_rules: DP_PIVOT_RULES,
        ..button("dp_piv", Visibility::Visible, Size::Match, 0.0)
    };
    static PCT_PIVOT_VIEW: Node = Node {
        kind: Kind::View,
        background: Some(Color(1, 2, 3, 255)),
        relative_rules: PCT_PIVOT_RULES,
        ..button("pct_piv", Visibility::Visible, Size::Match, 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.density = 2.0;
    let b = Rect { x: 20.0, y: 40.0, width: 100.0, height: 80.0 };
    let vp = Rect { x: 0.0, y: 0.0, width: 400.0, height: 300.0 };

    let cmds = ui.frame(Input::default(), vp, 0.0, |f| {
        TRANSFORMED_GROUP.render(f, b, &mut state);
    });
    let push_tr = cmds.iter().position(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, translate, scale, rotation_rad }
            if (*origin == [70.0, 80.0])
                && (*translate == [24.0, -12.0])
                && (*scale == [1.5, 0.5])
                && (*rotation_rad - 45.0_f32.to_radians()).abs() < 1e-5
    )).expect("expected PushTransform with density-scaled translation and centered origin");
    let push_layer = cmds.iter().position(|c| matches!(
        c,
        DrawCommand::PushLayer { bounds, alpha } if *bounds == b && (*alpha - 0.6).abs() < 1e-6
    )).expect("expected PushLayer for view_alpha=0.6");
    let bg_idx = cmds.iter().position(|c| matches!(c, DrawCommand::RoundedRect { color: Color(20, 40, 60, 255), .. })).unwrap();
    let child_idx = cmds.iter().position(|c| matches!(c, DrawCommand::RoundedRect { color: Color(200, 100, 50, 255), .. })).unwrap();
    let fg_idx = cmds.iter().position(|c| matches!(c, DrawCommand::RoundedRect { color: Color(255, 255, 255, 128), .. })).unwrap();
    let pop_layer = cmds.iter().position(|c| matches!(c, DrawCommand::PopLayer)).unwrap();
    let pop_tr = cmds.iter().position(|c| matches!(c, DrawCommand::PopTransform)).unwrap();
    assert!(
        push_tr < push_layer && push_layer < bg_idx && bg_idx < child_idx && child_idx < fg_idx && fg_idx < pop_layer && pop_layer < pop_tr,
        "expected PushTransform -> PushLayer -> bg -> child -> fg -> PopLayer -> PopTransform"
    );

    // Explicit dp pivot scaled by density=2.0: origin = (20 + 20, 40 + 10) = (40, 50)
    let cmds_dp = ui.frame(Input::default(), vp, 0.0, |f| { DP_PIVOT_VIEW.render(f, b, &mut state); });
    assert!(cmds_dp.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, scale, rotation_rad, .. }
            if *origin == [40.0, 50.0] && *scale == [2.0, 2.0] && (*rotation_rad - 90.0_f32.to_radians()).abs() < 1e-5
    )));

    // Percentage pivot: 25% of 100 = 25 -> 45, 75% of 80 = 60 -> 100
    let cmds_pct = ui.frame(Input::default(), vp, 0.0, |f| { PCT_PIVOT_VIEW.render(f, b, &mut state); });
    assert!(cmds_pct.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, rotation_rad, .. }
            if *origin == [45.0, 100.0] && (*rotation_rad - 30.0_f32.to_radians()).abs() < 1e-5
    )));
}

#[test]
fn wave14a_text_style_ext_italic_and_font_family_across_widgets() {
    use bhoo::android_xml::{AndroidTheme, RULE_TEXT_STYLE_EXT};

    static ITALIC_SERIF: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_STYLE_EXT, anchor: "1|serif" },
    ];
    static MONO_REG: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_STYLE_EXT, anchor: "0|mono" },
    ];
    static ITALIC_MONO: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_STYLE_EXT, anchor: "1|mono" },
    ];
    static ROOT: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node { kind: Kind::Text, text: "TvText", relative_rules: ITALIC_SERIF, ..button("w_tv", Visibility::Visible, Size::Dp(32.0), 0.0) },
            Node { kind: Kind::Button, text: "BtnText", relative_rules: MONO_REG, ..button("w_btn", Visibility::Visible, Size::Dp(48.0), 0.0) },
            Node { kind: Kind::Edit, text: "EdText", relative_rules: ITALIC_MONO, ..button("w_ed", Visibility::Visible, Size::Dp(48.0), 0.0) },
            Node { kind: Kind::CheckBox, text: "CbText", relative_rules: ITALIC_SERIF, ..button("w_cb", Visibility::Visible, Size::Dp(48.0), 0.0) },
            Node { kind: Kind::RadioButton, text: "RbText", relative_rules: MONO_REG, ..button("w_rb", Visibility::Visible, Size::Dp(48.0), 0.0) },
            Node { kind: Kind::Switch, text: "SwText", relative_rules: ITALIC_SERIF, ..button("w_sw", Visibility::Visible, Size::Dp(48.0), 0.0) },
            Node { kind: Kind::ToggleButton, text_on: "TgOn", text_off: "TgOff", relative_rules: ITALIC_MONO, ..button("w_tg", Visibility::Visible, Size::Dp(48.0), 0.0) },
            Node { kind: Kind::Toolbar, text: "TbTitle", hint: "TbSub", relative_rules: ITALIC_SERIF, ..button("w_tb", Visibility::Visible, Size::Dp(56.0), 0.0) },
        ],
        ..button("style_root", Visibility::Visible, Size::Match, 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 320.0, height: 500.0 };
    for theme in [AndroidTheme::Bhoo, AndroidTheme::Material3Light] {
        let mut ui = Ui::default();
        let mut state = LayoutState::default();
        state.theme_style = theme;
        let cmds = ui.frame(Input::default(), b, 0.0, |f| { ROOT.render(f, b, &mut state); });
        let find_style = |needle: &str| {
            cmds.iter()
                .find_map(|c| match c {
                    DrawCommand::Text { text, style, .. } if text.as_ref().eq_ignore_ascii_case(needle) => Some(*style),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing text '{needle}' under theme {theme:?}"))
        };

        let tv = find_style("TvText");
        assert!(tv.italic && tv.family == FontFamily::Serif);
        let btn = find_style("BtnText");
        assert!(!btn.italic && btn.family == FontFamily::Monospace);
        let ed = find_style("EdText");
        assert!(ed.italic && ed.family == FontFamily::Monospace);
        let cb = find_style("CbText");
        assert!(cb.italic && cb.family == FontFamily::Serif);
        let rb = find_style("RbText");
        assert!(!rb.italic && rb.family == FontFamily::Monospace);
        let sw = find_style("SwText");
        assert!(sw.italic && sw.family == FontFamily::Serif);
        let tg = find_style("TgOff");
        assert!(tg.italic && tg.family == FontFamily::Monospace);
        let tb_t = find_style("TbTitle");
        assert!(tb_t.italic && tb_t.family == FontFamily::Serif);
        let tb_s = find_style("TbSub");
        assert!(tb_s.italic && tb_s.family == FontFamily::Serif);
    }
}

#[test]
fn wave14a_tooltip_text_reveals_on_hover_delay_and_keyboard_focus() {
    use bhoo::android_xml::RULE_TOOLTIP_TEXT;

    static BTN_TIP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TOOLTIP_TEXT, anchor: "Save changes" },
    ];
    static BTN: Node = Node {
        kind: Kind::Button,
        text: "Save",
        relative_rules: BTN_TIP_RULES,
        ..button("tip_btn", Visibility::Visible, Size::Dp(48.0), 0.0)
    };
    static TV_TIP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TOOLTIP_TEXT, anchor: "Read-only info" },
    ];
    static TV: Node = Node {
        kind: Kind::Text,
        text: "Status",
        relative_rules: TV_TIP_RULES,
        ..button("tip_tv", Visibility::Visible, Size::Dp(40.0), 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 240.0, height: 120.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // 1. Button tooltip: hover at t=1.0 schedules wakeup at 1.5s, starts fading in at t=1.5s, visible by t=1.8s, hides on Escape.
    let btn_b = Rect { x: 10.0, y: 10.0, width: 120.0, height: 48.0 };
    let cmds0 = ui.frame(Input { time: 1.0, pointer: Some([40.0, 24.0]), ..Input::default() }, b, 0.0, |f| {
        BTN.render(f, btn_b, &mut state);
    });
    assert!(!cmds0.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Save changes")));
    assert_eq!(ui.next_wakeup(), Some(1.5));

    ui.frame(Input { time: 1.5, pointer: Some([40.0, 24.0]), ..Input::default() }, b, 0.0, |f| {
        BTN.render(f, btn_b, &mut state);
    });
    assert!(ui.needs_redraw(), "tooltip spring should begin revealing once hover deadline expires");

    let cmds1 = ui.frame(Input { time: 1.8, pointer: Some([40.0, 24.0]), ..Input::default() }, b, 0.0, |f| {
        BTN.render(f, btn_b, &mut state);
    });
    assert!(cmds1.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Save changes")));
    assert!(ui.semantics().iter().any(|n| n.role == Role::Tooltip && n.label.as_ref() == "Save changes"));

    // Escape suppresses tooltip while still hovered
    ui.frame(Input {
        time: 2.0,
        pointer: Some([40.0, 24.0]),
        key: Some(KeyInput { key: Key::Escape, pressed: true, shift: false, repeat: false }),
        ..Input::default()
    }, b, 0.0, |f| {
        BTN.render(f, btn_b, &mut state);
    });
    let cmds_esc = ui.frame(Input {
        time: 3.0,
        pointer: Some([40.0, 24.0]),
        ..Input::default()
    }, b, 0.0, |f| {
        BTN.render(f, btn_b, &mut state);
    });
    assert!(!cmds_esc.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Save changes")));

    // 2. Non-clickable TextView tooltip: Tab keyboard focus reveals immediately without waiting 0.5s hover delay
    let mut ui2 = Ui::default();
    let tv_b = Rect { x: 10.0, y: 10.0, width: 140.0, height: 40.0 };
    // First frame registers focus_order without hover
    ui2.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| { TV.render(f, tv_b, &mut state); });
    // Tab focuses the TextView via keyboard -> tooltip starts revealing immediately
    ui2.frame(Input {
        time: 1.05,
        key: Some(KeyInput { key: Key::Tab, pressed: true, shift: false, repeat: false }),
        ..Input::default()
    }, b, 0.0, |f| {
        TV.render(f, tv_b, &mut state);
    });
    assert!(ui2.needs_redraw());
    let cmds_tab = ui2.frame(Input { time: 1.35, ..Input::default() }, b, 0.0, |f| {
        TV.render(f, tv_b, &mut state);
    });
    assert!(cmds_tab.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Read-only info")));
    assert!(ui2.semantics().iter().any(|n| n.role == Role::Tooltip && n.label.as_ref() == "Read-only info"));
}

#[test]
fn wave15a_bottom_sheet_states_and_drag_handle() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_BOTTOM_SHEET, RULE_DRAG_HANDLE, SHEET_STATE_COLLAPSED,
        SHEET_STATE_EXPANDED, SHEET_STATE_HALF_EXPANDED, SHEET_STATE_HIDDEN,
    };

    static BS_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_BOTTOM_SHEET,
        anchor: "120|1|1|0.5|0",
    }];
    static DH_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_DRAG_HANDLE,
        anchor: "",
    }];
    static ROOT: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Text,
                text: "Backdrop",
                ..button("bs_bg", Visibility::Visible, Size::Match, 0.0)
            },
            Node {
                kind: Kind::Linear,
                height: Size::Dp(300.0),
                relative_rules: BS_RULES,
                children: &[
                    Node {
                        kind: Kind::View,
                        height: Size::Wrap,
                        relative_rules: DH_RULES,
                        ..button("bs_dh", Visibility::Visible, Size::Match, 0.0)
                    },
                    Node {
                        kind: Kind::Text,
                        text: "Sheet Body",
                        ..button("bs_body", Visibility::Visible, Size::Wrap, 0.0)
                    },
                ],
                ..button("bs_sheet", Visibility::Visible, Size::Match, 0.0)
            },
        ],
        ..button("bs_root", Visibility::Visible, Size::Match, 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 600.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // Helper: compute union y-range of clipped surfaceContainerLow sheet bands and verify top-only 28dp rounding
    let sheet_span = |cmds: &[DrawCommand]| -> Option<(f32, f32, bool)> {
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut has_top_r28 = false;
        let mut active_clip: Option<Rect> = None;
        for c in cmds {
            match c {
                DrawCommand::PushClip(r) => active_clip = Some(*r),
                DrawCommand::PopClip => active_clip = None,
                DrawCommand::RoundedRect { bounds, radius, color } if *color == Color(0xF7, 0xF2, 0xFA, 255) => {
                    let vis = active_clip.map_or(*bounds, |cl| bounds.intersect(cl));
                    if vis.height > 0.0 {
                        min_y = min_y.min(vis.y);
                        max_y = max_y.max(vis.y + vis.height);
                        if (*radius - 28.0).abs() < 0.5 {
                            has_top_r28 = true;
                        }
                    }
                }
                _ => {}
            }
        }
        (min_y < max_y).then_some((min_y, max_y - min_y, has_top_r28))
    };

    // Default (collapsed/peek = 120dp docked at bottom y = 480..600) + centered 32x4 drag handle pill
    let cmds = ui.frame(Input::default(), b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    let (y0, h0, top_r28) = sheet_span(&cmds).expect("expected surfaceContainerLow BottomSheet");
    assert!(top_r28, "BottomSheet must have 28dp top corners");
    assert!((y0 - 480.0).abs() < 1.0 && (h0 - 120.0).abs() < 1.0, "expected y=480, h=120, got y={y0}, h={h0}");
    assert!(
        cmds.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })),
        "BottomSheet must paint elevation level 2 shadow"
    );
    // Drag handle: centered 32x4 pill inside 48dp touch band at y = 480 + (48 - 4)/2 = 502
    assert!(
        cmds.iter().any(|c| matches!(
            c,
            DrawCommand::RoundedRect { bounds, radius, .. }
                if (bounds.width - 32.0).abs() < 0.5
                    && (bounds.height - 4.0).abs() < 0.5
                    && (bounds.x - 164.0).abs() < 0.5
                    && (bounds.y - 502.0).abs() < 0.5
                    && (*radius - 2.0).abs() < 0.2
        )),
        "expected centered 32x4 drag handle pill in 48dp touch height"
    );

    // Half-expanded (0.5 * 600 = 300 -> y = 300)
    state.sheet_state.insert("bs_sheet".into(), SHEET_STATE_HALF_EXPANDED);
    let cmds_half = ui.frame(Input::default(), b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    let (yh, hh, _) = sheet_span(&cmds_half).expect("expected half-expanded BottomSheet");
    assert!((yh - 300.0).abs() < 1.0 && (hh - 300.0).abs() < 1.0);

    // Expanded (fit_to_contents=1 -> natural 300dp height)
    state.sheet_state.insert("bs_sheet".into(), SHEET_STATE_EXPANDED);
    let cmds_exp = ui.frame(Input::default(), b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    let (_, he, _) = sheet_span(&cmds_exp).expect("expected expanded BottomSheet");
    assert!((he - 300.0).abs() < 1.0);

    // Collapsed explicitly returns to 120
    state.sheet_state.insert("bs_sheet".into(), SHEET_STATE_COLLAPSED);
    let cmds_col = ui.frame(Input::default(), b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    let (_, hc, _) = sheet_span(&cmds_col).expect("expected collapsed BottomSheet");
    assert!((hc - 120.0).abs() < 1.0);

    // Hidden (hideable=1 -> sheet skipped)
    state.sheet_state.insert("bs_sheet".into(), SHEET_STATE_HIDDEN);
    let cmds_hid = ui.frame(Input::default(), b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    assert!(!cmds_hid.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Sheet Body")));
}

#[test]
fn wave15a_side_sheet_modal_coplanar_and_rtl() {
    use bhoo::android_xml::{AndroidTheme, RULE_SIDE_SHEET, SHEET_STATE_HIDDEN};

    static COPLANAR_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SIDE_SHEET, anchor: "end|1" }];
    static MODAL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SIDE_SHEET, anchor: "end|0" }];

    static COPLANAR_ROOT: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::View,
                background: Some(Color(10, 20, 30, 255)),
                ..button("ss_main", Visibility::Visible, Size::Match, 0.0)
            },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(160.0),
                height: Size::Match,
                relative_rules: COPLANAR_RULES,
                children: &[Node {
                    kind: Kind::Text,
                    text: "Coplanar",
                    ..button("ss_txt", Visibility::Visible, Size::Wrap, 0.0)
                }],
                ..button("ss_coplanar", Visibility::Visible, Size::Dp(160.0), 0.0)
            },
        ],
        ..button("ss_root", Visibility::Visible, Size::Match, 0.0)
    };
    static MODAL_ROOT: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Text,
                text: "Main",
                ..button("ssm_main", Visibility::Visible, Size::Match, 0.0)
            },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(180.0),
                height: Size::Match,
                relative_rules: MODAL_RULES,
                children: &[Node {
                    kind: Kind::Text,
                    text: "ModalSide",
                    ..button("ssm_txt", Visibility::Visible, Size::Wrap, 0.0)
                }],
                ..button("ss_modal", Visibility::Visible, Size::Dp(180.0), 0.0)
            },
        ],
        ..button("ssm_root", Visibility::Visible, Size::Match, 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 500.0, height: 400.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // Coplanar LTR: main view shrinks to width 340 (x=0..340), side sheet sits at x=340..500 with 1dp divider at x=340
    let cmds = ui.frame(Input::default(), b, 0.0, |f| { COPLANAR_ROOT.render(f, b, &mut state); });
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds, color, .. }
            if *color == Color(10, 20, 30, 255) && (bounds.x - 0.0).abs() < 0.5 && (bounds.width - 340.0).abs() < 0.5
    )));
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds, color, .. }
            if *color == Color(0xCA, 0xC4, 0xD0, 255) && (bounds.x - 340.0).abs() < 0.5 && (bounds.width - 1.0).abs() < 0.5
    )), "coplanar side sheet should draw 1dp outline_variant inner divider");

    // Coplanar RTL: end edge mirrors to LEFT (sheet x=0..160, main content x=160..500, divider at x=159)
    state.rtl = true;
    let cmds_rtl = ui.frame(Input::default(), b, 0.0, |f| { COPLANAR_ROOT.render(f, b, &mut state); });
    assert!(cmds_rtl.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds, color, .. }
            if *color == Color(10, 20, 30, 255) && (bounds.x - 160.0).abs() < 0.5 && (bounds.width - 340.0).abs() < 0.5
    )));
    assert!(cmds_rtl.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds, color, .. }
            if *color == Color(0xCA, 0xC4, 0xD0, 255) && (bounds.x - 159.0).abs() < 0.5 && (bounds.width - 1.0).abs() < 0.5
    )));

    // Modal SideSheet: paints scrim (alpha 82) + shadow, and clicking scrim dismisses to SHEET_STATE_HIDDEN
    state.rtl = false;
    let cmds_m = ui.frame(Input::default(), b, 0.0, |f| { MODAL_ROOT.render(f, b, &mut state); });
    assert!(cmds_m.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0, 0, 0, 82), .. })));
    assert!(cmds_m.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })));
    ui.frame(Input { pointer: Some([50.0, 200.0]), pressed: true, ..Input::default() }, b, 0.0, |f| { MODAL_ROOT.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([50.0, 200.0]), released: true, ..Input::default() }, b, 0.0, |f| { MODAL_ROOT.render(f, b, &mut state); });
    assert_eq!(state.sheet_state.get("ss_modal"), Some(&SHEET_STATE_HIDDEN));
}

#[test]
fn wave15a_loading_indicator_and_wavy_progress() {
    use bhoo::android_xml::{AndroidTheme, RULE_LOADING_INDICATOR, RULE_WAVY_PROGRESS};

    static LI_CONT_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_LOADING_INDICATOR, anchor: "1" }];
    static LI_PLAIN_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_LOADING_INDICATOR, anchor: "0" }];
    static WAVY_LIN_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_WAVY_PROGRESS, anchor: "40|3|40|4|4" }];
    static FLAT_LIN_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_WAVY_PROGRESS, anchor: "0|0|0|4|6" }];
    static WAVY_CIRC_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_WAVY_PROGRESS, anchor: "16|3|40|0|4" }];

    static LI_CONT: Node = Node {
        kind: Kind::ProgressBar,
        relative_rules: LI_CONT_RULES,
        ..button("li_c", Visibility::Visible, Size::Dp(48.0), 0.0)
    };
    static LI_PLAIN: Node = Node {
        kind: Kind::ProgressBar,
        relative_rules: LI_PLAIN_RULES,
        ..button("li_p", Visibility::Visible, Size::Dp(48.0), 0.0)
    };
    static WAVY_LIN: Node = Node {
        kind: Kind::ProgressBar,
        horizontal: true,
        progress: 50.0,
        relative_rules: WAVY_LIN_RULES,
        ..button("wl_1", Visibility::Visible, Size::Dp(200.0), 0.0)
    };
    static FLAT_LIN: Node = Node {
        kind: Kind::ProgressBar,
        horizontal: true,
        progress: 50.0,
        relative_rules: FLAT_LIN_RULES,
        ..button("fl_1", Visibility::Visible, Size::Dp(200.0), 0.0)
    };
    static WAVY_CIRC: Node = Node {
        kind: Kind::ProgressBar,
        horizontal: false,
        progress: 60.0,
        relative_rules: WAVY_CIRC_RULES,
        ..button("wc_1", Visibility::Visible, Size::Dp(64.0), 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 240.0, height: 80.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // Contained LoadingIndicator: primary_container pill + DrawCommand::Loading(on_primary_container)
    let c_cont = ui.frame(Input::default(), b, 0.0, |f| { LI_CONT.render(f, Rect { x: 10.0, y: 10.0, width: 48.0, height: 48.0 }, &mut state); });
    assert!(c_cont.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0xEA, 0xDD, 0xFF, 255), .. })));
    assert!(c_cont.iter().any(|c| matches!(c, DrawCommand::Loading { color: Color(0x21, 0x00, 0x5D, 255), .. })));

    // Uncontained LoadingIndicator: no primary_container pill, DrawCommand::Loading(primary)
    let c_plain = ui.frame(Input::default(), b, 0.0, |f| { LI_PLAIN.render(f, Rect { x: 10.0, y: 10.0, width: 48.0, height: 48.0 }, &mut state); });
    assert!(!c_plain.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0xEA, 0xDD, 0xFF, 255), .. })));
    assert!(c_plain.iter().any(|c| matches!(c, DrawCommand::Loading { color: Color(0x67, 0x50, 0xA4, 255), .. })));

    // Wavy linear progress: sinusoidal DrawCommand::Line wave segments + trailing stop indicator dot
    let c_wl = ui.frame(Input::default(), b, 0.0, |f| { WAVY_LIN.render(f, Rect { x: 10.0, y: 20.0, width: 200.0, height: 24.0 }, &mut state); });
    let wave_lines = c_wl.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count();
    assert!(wave_lines >= 10, "expected sinusoidal wave Line segments, got {wave_lines}");
    assert!(c_wl.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color: Color(0x67, 0x50, 0xA4, 255), .. } if (bounds.width - 4.0).abs() < 0.5 && (bounds.height - 4.0).abs() < 0.5)));

    // Flat progress (wavelength=0): no wave Lines, active bar + gapped inactive track + stop dot
    let c_fl = ui.frame(Input::default(), b, 0.0, |f| { FLAT_LIN.render(f, Rect { x: 10.0, y: 20.0, width: 200.0, height: 24.0 }, &mut state); });
    assert_eq!(c_fl.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count(), 0);
    let active_bar = c_fl.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds, color: Color(0x67, 0x50, 0xA4, 255), .. } if bounds.width > 20.0 => Some(*bounds),
        _ => None,
    }).expect("expected flat active bar");
    let inactive_bar = c_fl.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds, color: Color(0xE8, 0xDE, 0xF8, 255), .. } => Some(*bounds),
        _ => None,
    }).expect("expected flat inactive track");
    assert!((inactive_bar.x - (active_bar.x + active_bar.width) - 6.0).abs() < 1.0, "expected 6dp track gap");

    // Wavy circular progress: sinusoidal wave Lines + inactive Arc
    let c_wc = ui.frame(Input::default(), b, 0.0, |f| { WAVY_CIRC.render(f, Rect { x: 10.0, y: 8.0, width: 64.0, height: 64.0 }, &mut state); });
    assert!(c_wc.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count() >= 12);
    assert!(c_wc.iter().any(|c| matches!(c, DrawCommand::Arc { .. })));
}

#[test]
fn wave15a_alert_dialog_and_snackbar_chrome() {
    use bhoo::android_xml::{AndroidTheme, RULE_ALERT_DIALOG, RULE_SNACKBAR};

    static DLG_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_ALERT_DIALOG, anchor: "delete" }];
    static SNACK_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SNACKBAR, anchor: "" }];

    static DIALOG: Node = Node {
        kind: Kind::Linear,
        relative_rules: DLG_RULES,
        children: &[Node {
            kind: Kind::Text,
            text: "Delete item?",
            ..button("dlg_t", Visibility::Visible, Size::Match, 0.0)
        }],
        ..button("dlg_box", Visibility::Visible, Size::Dp(280.0), 0.0)
    };
    static SNACKBAR: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        relative_rules: SNACK_RULES,
        children: &[
            Node {
                kind: Kind::Text,
                width: Size::Dp(0.0),
                weight: 1.0,
                text: "Message sent",
                ..button("sn_msg", Visibility::Visible, Size::Wrap, 0.0)
            },
            Node {
                kind: Kind::Button,
                width: Size::Wrap,
                text: "Undo",
                ..button("sn_btn", Visibility::Visible, Size::Wrap, 0.0)
            },
        ],
        ..button("sn_box", Visibility::Visible, Size::Dp(300.0), 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 340.0, height: 220.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    let c_dlg = ui.frame(Input::default(), b, 0.0, |f| {
        DIALOG.render(f, Rect { x: 20.0, y: 20.0, width: 280.0, height: 140.0 }, &mut state);
    });
    assert!(c_dlg.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { radius, color: Color(0xEC, 0xE6, 0xF0, 255), .. } if (*radius - 28.0).abs() < 0.5
    )), "AlertDialog must use 28dp rounded surfaceContainerHigh");
    assert!(c_dlg.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })));
    // Top 24dp icon at y = 20 + 24 (default padding) = 44, and title pushed down by 40dp to y = 84
    assert!(c_dlg.iter().any(|c| matches!(
        c,
        DrawCommand::Shape { bounds, .. } if (bounds.width - 24.0).abs() < 0.5 && (bounds.y - 44.0).abs() < 0.5
    )));
    assert!(c_dlg.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, bounds, .. } if text.as_ref() == "Delete item?" && bounds.y >= 83.0
    )));

    let c_sn = ui.frame(Input::default(), b, 0.0, |f| {
        SNACKBAR.render(f, Rect { x: 10.0, y: 10.0, width: 300.0, height: 48.0 }, &mut state);
    });
    assert!(c_sn.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { radius, color: Color(0x32, 0x2F, 0x35, 255), .. } if (*radius - 4.0).abs() < 0.5
    )), "Snackbar must use 4dp rounded inverseSurface");
    assert!(c_sn.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0xF5, 0xEF, 0xF7, 255), .. } if text.as_ref() == "Message sent"
    )), "Snackbar message must default to inverseOnSurface");
    assert!(c_sn.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0xD0, 0xBC, 0xFF, 255), .. } if text.as_ref() == "Undo"
    )), "Snackbar action button must default to inversePrimary");
}

#[test]
fn wave15a_tab_options_primary_secondary_custom_color_and_scrollable() {
    use bhoo::android_xml::{AndroidTheme, RULE_TAB_OPTIONS};

    static PRIMARY_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_TAB_OPTIONS, anchor: "fixed|primary|220,40,80,255" }];
    static SECONDARY_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_TAB_OPTIONS, anchor: "fixed|secondary|" }];
    static SCROLL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_TAB_OPTIONS, anchor: "scrollable|primary|" }];

    static TABS_PRIMARY: Node = Node {
        kind: Kind::TabWidget,
        relative_rules: PRIMARY_RULES,
        children: &[
            Node { kind: Kind::Text, text: "Hi", align: 1, content_gravity: [1, 1], ..button("tp1", Visibility::Visible, Size::Match, 0.0) },
            Node { kind: Kind::Text, text: "Longer", align: 1, content_gravity: [1, 1], ..button("tp2", Visibility::Visible, Size::Match, 0.0) },
        ],
        ..button("tabs_p", Visibility::Visible, Size::Match, 0.0)
    };
    static TABS_SECONDARY: Node = Node {
        kind: Kind::TabWidget,
        relative_rules: SECONDARY_RULES,
        children: &[
            Node { kind: Kind::Text, text: "One", align: 1, content_gravity: [1, 1], ..button("ts1", Visibility::Visible, Size::Match, 0.0) },
            Node { kind: Kind::Text, text: "Two", align: 1, content_gravity: [1, 1], ..button("ts2", Visibility::Visible, Size::Match, 0.0) },
        ],
        ..button("tabs_s", Visibility::Visible, Size::Match, 0.0)
    };
    static TABS_SCROLL: Node = Node {
        kind: Kind::TabWidget,
        relative_rules: SCROLL_RULES,
        children: &[
            Node { kind: Kind::Text, text: "Tab Alpha", align: 1, content_gravity: [1, 1], ..button("tsc1", Visibility::Visible, Size::Wrap, 0.0) },
            Node { kind: Kind::Text, text: "Tab Bravo", align: 1, content_gravity: [1, 1], ..button("tsc2", Visibility::Visible, Size::Wrap, 0.0) },
            Node { kind: Kind::Text, text: "Tab Charlie", align: 1, content_gravity: [1, 1], ..button("tsc3", Visibility::Visible, Size::Wrap, 0.0) },
            Node { kind: Kind::Text, text: "Tab Delta", align: 1, content_gravity: [1, 1], ..button("tsc4", Visibility::Visible, Size::Wrap, 0.0) },
        ],
        ..button("tabs_sc", Visibility::Visible, Size::Match, 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 300.0, height: 48.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // Primary: 3dp top-rounded indicator with custom RGBA (220,40,80,255) narrower than full 150dp tab
    let c_p = ui.frame(Input::default(), b, 0.0, |f| { TABS_PRIMARY.render(f, b, &mut state); });
    let mut active_clip: Option<Rect> = None;
    let mut found_primary_ind = false;
    for c in &c_p {
        match c {
            DrawCommand::PushClip(r) => active_clip = Some(*r),
            DrawCommand::PopClip => active_clip = None,
            DrawCommand::RoundedRect { bounds, radius, color: Color(220, 40, 80, 255) } => {
                let vis = active_clip.map_or(*bounds, |cl| bounds.intersect(cl));
                if (vis.height - 3.0).abs() < 0.5 && (*radius - 3.0).abs() < 0.5 && vis.width < 100.0 && vis.width >= 24.0 {
                    found_primary_ind = true;
                }
            }
            _ => {}
        }
    }
    assert!(found_primary_ind, "Primary tab indicator should be 3dp tall top-rounded, label-width, and use custom indicator_rgba");

    // Secondary: 2dp full-width indicator (width == 150dp)
    let c_s = ui.frame(Input::default(), b, 0.0, |f| { TABS_SECONDARY.render(f, b, &mut state); });
    assert!(c_s.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds, color: Color(0x67, 0x50, 0xA4, 255), .. }
            if (bounds.height - 2.0).abs() < 0.5 && (bounds.width - 150.0).abs() < 0.5
    )), "Secondary tab indicator should be 2dp tall and full tab width");

    // Scrollable: horizontal overflow in narrow 180px viewport scrolls via wheel input
    let narrow = Rect { x: 0.0, y: 0.0, width: 180.0, height: 48.0 };
    let c_sc0 = ui.frame(Input::default(), narrow, 0.0, |f| { TABS_SCROLL.render(f, narrow, &mut state); });
    let x0 = c_sc0.iter().find_map(|c| match c {
        DrawCommand::Text { text, bounds, .. } if text.as_ref() == "Tab Alpha" => Some(bounds.x),
        _ => None,
    }).unwrap();
    let c_sc1 = ui.frame(Input { pointer: Some([90.0, 24.0]), scroll: [60.0, 0.0], ..Input::default() }, narrow, 0.0, |f| {
        TABS_SCROLL.render(f, narrow, &mut state);
    });
    let x1 = c_sc1.iter().find_map(|c| match c {
        DrawCommand::Text { text, bounds, .. } if text.as_ref() == "Tab Alpha" => Some(bounds.x),
        _ => None,
    }).unwrap();
    assert!(x1 < x0, "Scrollable TabWidget should scroll horizontally: x0={x0}, x1={x1}");
}

#[test]
fn wave15a_slider_options_step_min_sep_expressive_and_bubbles() {
    use bhoo::android_xml::{AndroidTheme, RULE_RANGE_SLIDER, RULE_SLIDER_OPTIONS};

    static EXP_SINGLE_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_SLIDER_OPTIONS,
        anchor: "10|0|visible|1",
    }];
    static EXP_RANGE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_RANGE_SLIDER, anchor: "20,80" },
        RelativeRule { kind: RULE_SLIDER_OPTIONS, anchor: "10|20|visible|1" },
    ];
    static GONE_LABEL_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_SLIDER_OPTIONS,
        anchor: "0|0|gone|1",
    }];

    static EXP_SLIDER: Node = Node {
        kind: Kind::SeekBar,
        min: 0.0,
        max: 100.0,
        progress: 50.0,
        relative_rules: EXP_SINGLE_RULES,
        ..button("exp_s", Visibility::Visible, Size::Match, 0.0)
    };
    static EXP_RANGE: Node = Node {
        kind: Kind::SeekBar,
        min: 0.0,
        max: 100.0,
        relative_rules: EXP_RANGE_RULES,
        ..button("exp_r", Visibility::Visible, Size::Match, 0.0)
    };
    static GONE_SLIDER: Node = Node {
        kind: Kind::SeekBar,
        min: 0.0,
        max: 100.0,
        progress: 40.0,
        relative_rules: GONE_LABEL_RULES,
        ..button("gone_s", Visibility::Visible, Size::Match, 0.0)
    };

    let vp = Rect { x: 0.0, y: 0.0, width: 320.0, height: 140.0 };
    let b = Rect { x: 20.0, y: 60.0, width: 220.0, height: 48.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // Expressive single slider: 16dp track, 4x44dp handle, visible "50" bubble
    let cmds = ui.frame(Input::default(), vp, 0.0, |f| { EXP_SLIDER.render(f, b, &mut state); });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, .. } if (bounds.height - 16.0).abs() < 0.5)));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, .. } if (bounds.width - 4.0).abs() < 0.5 && (bounds.height - 44.0).abs() < 0.5)));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "50")), "visible labelBehavior must render value bubble");

    // Drag to ~33% -> snaps to step_size=10 -> 30
    let drag_x = b.x + 10.0 + (b.width - 20.0) * 0.33;
    ui.frame(Input { pointer: Some([drag_x, b.y + 24.0]), pressed: true, ..Input::default() }, vp, 0.0, |f| {
        EXP_SLIDER.render(f, b, &mut state);
    });
    assert_eq!(state.progress.get("exp_s").copied(), Some(30.0));

    // Gone labelBehavior suppresses bubble even while pressed
    let cmds_g = ui.frame(Input { pointer: Some([b.x + 80.0, b.y + 24.0]), pressed: true, ..Input::default() }, vp, 0.0, |f| {
        GONE_SLIDER.render(f, b, &mut state);
    });
    assert!(!cmds_g.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0x32, 0x2F, 0x35, 255), .. })));

    // Expressive RangeSlider: dual bubbles ("20" & "80") and min_separation=20 enforced on drag
    let cmds_r = ui.frame(Input::default(), vp, 0.0, |f| { EXP_RANGE.render(f, b, &mut state); });
    assert!(cmds_r.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "20")));
    assert!(cmds_r.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "80")));
    // Grab lower thumb (at 20) and drag all the way past 80 -> clamped to 80 - 20 = 60!
    let x_low = b.x + 10.0 + (b.width - 20.0) * 0.2;
    let x_far = b.x + 10.0 + (b.width - 20.0) * 0.95;
    ui.frame(Input { pointer: Some([x_low, b.y + 24.0]), pressed: true, ..Input::default() }, vp, 0.0, |f| {
        EXP_RANGE.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([x_far, b.y + 24.0]), ..Input::default() }, vp, 0.0, |f| {
        EXP_RANGE.render(f, b, &mut state);
    });
    assert_eq!(state.range_values.get("exp_r").copied(), Some([60.0, 80.0]));
}

#[test]
fn wave15a_text_input_ext_affixes_counter_and_exposed_dropdown() {
    use bhoo::android_xml::{AndroidTheme, RULE_TEXT_INPUT_BOX, RULE_TEXT_INPUT_EXT};

    static TIL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_INPUT_BOX, anchor: "outline|4||Plan||0|dropdown_menu" },
        RelativeRule { kind: RULE_TEXT_INPUT_EXT, anchor: "1|8|$|USD|search" },
    ];
    static TIL: Node = Node {
        kind: Kind::Linear,
        relative_rules: TIL_RULES,
        children: &[Node {
            kind: Kind::Edit,
            text: "Pro",
            hint: "Plan",
            ..button("plan_edit", Visibility::Visible, Size::Match, 0.0)
        }],
        ..button("plan_til", Visibility::Visible, Size::Match, 0.0)
    };

    let vp = Rect { x: 0.0, y: 0.0, width: 320.0, height: 320.0 };
    let b = Rect { x: 16.0, y: 16.0, width: 280.0, height: 82.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    state.entries.insert("plan_edit".into(), vec!["Basic".into(), "Pro".into(), "Enterprise".into()]);

    // Initial draw: start_icon Shape, prefix "$", suffix "USD", counter "3 / 8" in on_surface_variant
    let cmds = ui.frame(Input::default(), vp, 0.0, |f| { TIL.render(f, b, &mut state); });
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Shape { bounds, .. } if (bounds.width - 24.0).abs() < 0.5)));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "$")));
    assert!(cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "USD")));
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0x49, 0x45, 0x4F, 255), .. } if text.as_ref() == "3 / 8"
    )));

    // Overflow counter (> 8 chars) switches to M3 error color (#B3261E)
    state.text.insert("plan_edit".into(), "Enterprise".into());
    let cmds_over = ui.frame(Input::default(), vp, 0.0, |f| { TIL.render(f, b, &mut state); });
    assert!(cmds_over.iter().any(|c| matches!(
        c,
        DrawCommand::Text { text, color: Color(0xB3, 0x26, 0x1E, 255), .. } if text.as_ref() == "10 / 8"
    )));

    // Click dropdown_menu end icon (right edge of box) -> opens Exposed Dropdown popup listing state.entries["plan_edit"]
    let icon_x = b.x + b.width - 24.0;
    let icon_y = b.y + 34.0;
    ui.frame(Input { pointer: Some([icon_x, icon_y]), pressed: true, ..Input::default() }, vp, 0.0, |f| {
        TIL.render(f, b, &mut state);
    });
    let cmds_open = ui.frame(Input { pointer: Some([icon_x, icon_y]), released: true, ..Input::default() }, vp, 0.0, |f| {
        TIL.render(f, b, &mut state);
    });
    assert!(cmds_open.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Basic")));
    assert!(cmds_open.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Enterprise")));

    // Click first item ("Basic") below box -> selects it into state.text["plan_edit"] and closes popup
    let first_item_pos = [b.x + 60.0, b.y + 84.0];
    ui.frame(Input { pointer: Some(first_item_pos), pressed: true, ..Input::default() }, vp, 0.0, |f| {
        TIL.render(f, b, &mut state);
    });
    let mut act = None;
    ui.frame(Input { pointer: Some(first_item_pos), released: true, ..Input::default() }, vp, 0.0, |f| {
        act = TIL.render(f, b, &mut state);
    });
    assert_eq!(act, Some("plan_til"));
    assert_eq!(state.text.get("plan_edit").map(String::as_str), Some("Basic"));
}

#[test]
fn wave15a_chip_group_wrap_multiline_and_single_selection() {
    use bhoo::android_xml::{AndroidTheme, RULE_CHIP_GROUP_WRAP, RULE_CHIP_STYLE};

    static CG_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CHIP_GROUP_WRAP, anchor: "0|1|8|10" }];
    static FILTER_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CHIP_STYLE, anchor: "filter|0" }];

    static ROOT: Node = Node {
        kind: Kind::Linear,
        children: &[
            Node {
                kind: Kind::Linear,
                horizontal: true,
                height: Size::Wrap,
                relative_rules: CG_RULES,
                children: &[
                    Node { kind: Kind::Button, text: "Alpha", checked: true, width: Size::Dp(90.0), height: Size::Dp(32.0), relative_rules: FILTER_RULES, ..button("cg_1", Visibility::Visible, Size::Dp(90.0), 0.0) },
                    Node { kind: Kind::Button, text: "Beta", width: Size::Dp(90.0), height: Size::Dp(32.0), relative_rules: FILTER_RULES, ..button("cg_2", Visibility::Visible, Size::Dp(90.0), 0.0) },
                    Node { kind: Kind::Button, text: "Gamma", width: Size::Dp(90.0), height: Size::Dp(32.0), relative_rules: FILTER_RULES, ..button("cg_3", Visibility::Visible, Size::Dp(90.0), 0.0) },
                ],
                ..button("cg_group", Visibility::Visible, Size::Match, 0.0)
            },
            Node {
                kind: Kind::Text,
                text: "AfterGroup",
                ..button("cg_after", Visibility::Visible, Size::Wrap, 0.0)
            },
        ],
        ..button("cg_root", Visibility::Visible, Size::Match, 0.0)
    };

    // Width 200: row 1 fits Alpha (90) + 8 + Beta (90) = 188, row 2 wraps Gamma (90) at y = 32 + 10 = 42 -> total height 74
    let b = Rect { x: 0.0, y: 0.0, width: 200.0, height: 160.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    let cmds = ui.frame(Input::default(), b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    let find_y = |label: &str| {
        cmds.iter().find_map(|c| match c {
            DrawCommand::Text { text, bounds, .. } if text.as_ref() == label => Some(bounds.y),
            _ => None,
        }).unwrap()
    };
    assert!((find_y("Alpha") - find_y("Beta")).abs() < 1.0, "Alpha & Beta should share row 1");
    assert!(find_y("Gamma") >= find_y("Alpha") + 41.0, "Gamma should wrap to row 2 with 10dp v_spacing");
    assert!(find_y("AfterGroup") >= 73.0, "Wrap height of ChipGroup must account for both rows + v_spacing");

    // Single selection: clicking Beta checks Beta and unchecks Alpha
    let beta_pos = [140.0, 16.0];
    ui.frame(Input { pointer: Some(beta_pos), pressed: true, ..Input::default() }, b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some(beta_pos), released: true, ..Input::default() }, b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    assert_eq!(state.checked.get("cg_1"), Some(&false));
    assert_eq!(state.checked.get("cg_2"), Some(&true));
    assert_eq!(state.checked.get("cg_3"), Some(&false));
}

#[test]
fn wave15a_floating_toolbar_button_group_split_button_and_carousel_item() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_BUTTON_GROUP, RULE_BUTTON_VARIANT, RULE_CAROUSEL_ITEM,
        RULE_FLOATING_TOOLBAR, RULE_SPLIT_BUTTON,
    };

    static FT_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_FLOATING_TOOLBAR, anchor: "" }];
    static BG_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_GROUP, anchor: "14" }];
    static SB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SPLIT_BUTTON, anchor: "" }];
    static CAR_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CAROUSEL_ITEM, anchor: "" }];
    static FILLED: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "filled" }];

    static FLOATING_TB: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        height: Size::Wrap,
        relative_rules: FT_RULES,
        children: &[Node { kind: Kind::Icon, icon: MaterialIcon::Edit, ..button("ft_i", Visibility::Visible, Size::Dp(40.0), 0.0) }],
        ..button("ft_box", Visibility::Visible, Size::Dp(160.0), 0.0)
    };
    static BTN_GROUP: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        relative_rules: BG_RULES,
        children: &[
            Node { kind: Kind::Button, text: "A", width: Size::Dp(60.0), height: Size::Dp(40.0), relative_rules: FILLED, ..button("bg_a", Visibility::Visible, Size::Dp(60.0), 0.0) },
            Node { kind: Kind::Button, text: "B", width: Size::Dp(60.0), height: Size::Dp(40.0), relative_rules: FILLED, ..button("bg_b", Visibility::Visible, Size::Dp(60.0), 0.0) },
        ],
        ..button("bg_root", Visibility::Visible, Size::Match, 0.0)
    };
    static SPLIT_BTN: Node = Node {
        kind: Kind::Linear,
        horizontal: true,
        relative_rules: SB_RULES,
        children: &[
            Node { kind: Kind::Button, text: "Save", width: Size::Dp(90.0), height: Size::Dp(40.0), relative_rules: FILLED, ..button("sb_lead", Visibility::Visible, Size::Dp(90.0), 0.0) },
            Node { kind: Kind::Button, text: "", width: Size::Dp(40.0), height: Size::Dp(40.0), relative_rules: FILLED, ..button("sb_trail", Visibility::Visible, Size::Dp(40.0), 0.0) },
        ],
        ..button("sb_root", Visibility::Visible, Size::Wrap, 0.0)
    };
    static CAROUSEL_ITEM: Node = Node {
        kind: Kind::Frame,
        relative_rules: CAR_RULES,
        children: &[Node {
            kind: Kind::View,
            background: Some(Color(20, 120, 220, 255)),
            ..button("car_v", Visibility::Visible, Size::Match, 0.0)
        }],
        ..button("car_item", Visibility::Visible, Size::Dp(140.0), 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 260.0, height: 120.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // FloatingToolbar: 64dp pill radius=32, surfaceContainer (#F3EDF7), elevation shadow
    let c_ft = ui.frame(Input::default(), b, 0.0, |f| {
        FLOATING_TB.render(f, Rect { x: 10.0, y: 10.0, width: 160.0, height: 64.0 }, &mut state);
    });
    assert!(c_ft.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds, radius, color: Color(0xF3, 0xED, 0xF7, 255) }
            if (bounds.height - 64.0).abs() < 0.5 && (*radius - 32.0).abs() < 0.5
    )));
    assert!(c_ft.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })));

    // ButtonGroup: 14dp gap between A (x=0..60) and B (x=74..134)
    let c_bg = ui.frame(Input::default(), b, 0.0, |f| {
        BTN_GROUP.render(f, Rect { x: 0.0, y: 0.0, width: 200.0, height: 48.0 }, &mut state);
    });
    let ax = c_bg.iter().find_map(|c| match c { DrawCommand::Text { text, bounds, .. } if text.as_ref() == "A" => Some(bounds.x), _ => None }).unwrap();
    let bx = c_bg.iter().find_map(|c| match c { DrawCommand::Text { text, bounds, .. } if text.as_ref() == "B" => Some(bounds.x), _ => None }).unwrap();
    assert!((bx - ax - 74.0).abs() < 1.0, "expected 60+14=74dp offset between buttons, got {}", bx - ax);

    // SplitButton: 2dp gap + trailing chevron Polyline + RTL mirroring
    let c_sb = ui.frame(Input::default(), b, 0.0, |f| {
        SPLIT_BTN.render(f, Rect { x: 0.0, y: 0.0, width: 160.0, height: 40.0 }, &mut state);
    });
    let chev_ltr_x = c_sb.iter().find_map(|c| match c { DrawCommand::Polyline { points, count: 3, .. } => Some(points[1][0]), _ => None }).expect("expected trailing chevron");
    assert!((chev_ltr_x - 112.0).abs() < 1.5, "LTR trailing chevron center should be around 90+2+20=112, got {chev_ltr_x}");
    state.rtl = true;
    let c_sb_rtl = ui.frame(Input::default(), b, 0.0, |f| {
        SPLIT_BTN.render(f, Rect { x: 0.0, y: 0.0, width: 160.0, height: 40.0 }, &mut state);
    });
    let chev_rtl_x = c_sb_rtl.iter().find_map(|c| match c { DrawCommand::Polyline { points, count: 3, .. } => Some(points[1][0]), _ => None }).expect("expected RTL trailing chevron");
    assert!(chev_rtl_x < 60.0, "RTL trailing chevron should mirror to left side, got {chev_rtl_x}");
    state.rtl = false;

    // CarouselItem: clips child rect to 28dp rounded outline
    let c_car = ui.frame(Input::default(), b, 0.0, |f| {
        CAROUSEL_ITEM.render(f, Rect { x: 0.0, y: 0.0, width: 140.0, height: 100.0 }, &mut state);
    });
    assert!(c_car.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { radius, color: Color(20, 120, 220, 255), .. } if (*radius - 28.0).abs() < 0.5
    )), "CarouselItem must clip child draw commands to default 28dp rounded outline");
}

#[test]
fn wave15a_m3_date_picker_and_time_picker_chrome() {
    use bhoo::android_xml::{AndroidTheme, WallClock};

    static CLOCK_RULES: &[RelativeRule] = &[RelativeRule { kind: 105, anchor: "clock" }];
    static DATE_PICKER: Node = Node {
        kind: Kind::DatePicker,
        ..button("m3_dp", Visibility::Visible, Size::Dp(348.0), 0.0)
    };
    static TIME_PICKER: Node = Node {
        kind: Kind::TimePicker,
        relative_rules: CLOCK_RULES,
        ..button("m3_tp", Visibility::Visible, Size::Dp(328.0), 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 500.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    // 2026-06-15 09:30 UTC
    state.wall_clock = Some(WallClock { unix_seconds: 1_781_515_800.0, offset_seconds: 0, is_24_hour: false });

    // DatePicker M3 chrome: 28dp surfaceContainerHigh container, NO M2 solid primary header, M3 outlined today ring when another day is selected
    let _ = ui.frame(Input::default(), b, 0.0, |f| { DATE_PICKER.render(f, b, &mut state); });
    state.calendars.get_mut("m3_dp").unwrap().select_day(20);
    let c_dp = ui.frame(Input::default(), b, 0.0, |f| { DATE_PICKER.render(f, b, &mut state); });
    assert!(c_dp.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { radius, color: Color(0xEC, 0xE6, 0xF0, 255), .. } if (*radius - 28.0).abs() < 0.5
    )), "M3 DatePicker must use 28dp surfaceContainerHigh chrome");
    assert!(!c_dp.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds, color: Color(0x67, 0x50, 0xA4, 255), .. } if bounds.width > 100.0 && bounds.height > 60.0
    )), "M3 DatePicker must NOT paint Material 2 solid primary header");
    assert!(c_dp.iter().any(|c| matches!(
        c,
        DrawCommand::Outline { bounds, color: Color(0x67, 0x50, 0xA4, 255), .. } if (bounds.width - bounds.height).abs() < 1.0 && bounds.width <= 42.0
    )), "M3 DatePicker must paint outlined today ring");

    // TimePicker M3 chrome: 28dp surfaceContainerHigh, primaryContainer active hour box, tertiaryContainer AM/PM toggle, NO solid M2 header
    let c_tp = ui.frame(Input::default(), b, 0.0, |f| { TIME_PICKER.render(f, b, &mut state); });
    assert!(c_tp.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { radius, color: Color(0xEC, 0xE6, 0xF0, 255), .. } if (*radius - 28.0).abs() < 0.5
    )), "M3 TimePicker must use 28dp surfaceContainerHigh chrome");
    assert!(!c_tp.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds, color: Color(0x67, 0x50, 0xA4, 255), .. } if bounds.width > 200.0 && bounds.height > 60.0
    )), "M3 TimePicker must NOT paint Material 2 solid primary header");
    assert!(c_tp.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { color: Color(0xEA, 0xDD, 0xFF, 255), .. }
    )), "M3 TimePicker must paint primaryContainer active hour/minute display box");
    assert!(c_tp.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { color: Color(0xFF, 0xD8, 0xE4, 255), .. }
    )), "M3 TimePicker must paint tertiaryContainer AM/PM segmented toggle");
}

#[test]
fn overlapping_frame_sibling_and_clickable_container_child_hit_boundaries() {
    // 1. FrameLayout / CoordinatorLayout: floating FAB overlays clickable row underneath.
    // Clicking over the FAB MUST activate the FAB and NEVER leak hover/press/click to the row below.
    static ROW_CHILDREN: &[Node] = &[
        Node {
            kind: Kind::Text,
            width: Size::Dp(0.0),
            height: Size::Wrap,
            weight: 1.0,
            text: "Message Row",
            ..button("", Visibility::Visible, Size::Wrap, 0.0)
        },
        Node {
            kind: Kind::Icon,
            id: "star_btn",
            width: Size::Dp(36.0),
            height: Size::Dp(36.0),
            text: "Star",
            ..button("star_btn", Visibility::Visible, Size::Dp(36.0), 0.0)
        },
    ];
    static OVERLAID_ROW: Node = Node {
        kind: Kind::Linear,
        id: "email_row",
        width: Size::Match,
        height: Size::Dp(80.0),
        horizontal: true,
        clickable: true,
        padding_edges: [16.0, 12.0, 8.0, 12.0],
        padding_explicit: [true; 4],
        children: ROW_CHILDREN,
        ..button("email_row", Visibility::Visible, Size::Dp(80.0), 0.0)
    };
    static FAB_RULES: &[RelativeRule] = &[RelativeRule { kind: bhoo::android_xml::RULE_EXTENDED_FAB, anchor: "1" }];
    static FAB: Node = Node {
        kind: Kind::Button,
        id: "fab_compose",
        width: Size::Dp(120.0),
        height: Size::Dp(56.0),
        layout_gravity: [2, 2],
        layout_gravity_explicit: true,
        margin: [0.0, 0.0, 16.0, 12.0],
        radius: 28.0,
        text: "Compose",
        relative_rules: FAB_RULES,
        ..button("fab_compose", Visibility::Visible, Size::Dp(56.0), 0.0)
    };
    static COORD_CHILDREN: &[Node] = &[OVERLAID_ROW, FAB];
    static COORD: Node = Node {
        kind: Kind::Frame,
        id: "coord",
        width: Size::Match,
        height: Size::Match,
        children: COORD_CHILDREN,
        ..button("coord", Visibility::Visible, Size::Match, 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 80.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // Click inside FAB (x=280, y=40): overlaps email_row, MUST trigger fab_compose (NOT email_row!)
    let fab_pt = [280.0, 40.0];
    let mut down_action = None;
    let mut up_action = None;
    ui.frame(Input { pointer: Some(fab_pt), pressed: true, time: 1.0, ..Input::default() }, b, 0.0, |f| {
        down_action = COORD.render(f, b, &mut state);
    });
    let cmds = ui.frame(Input { pointer: Some(fab_pt), released: true, time: 1.05, ..Input::default() }, b, 0.0, |f| {
        up_action = COORD.render(f, b, &mut state);
    });
    assert_eq!(down_action, None);
    assert_eq!(up_action, Some("fab_compose"), "Topmost FAB must capture click instead of overlaid row");
    assert!(cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Ripple { bounds, corner_radius, .. }
            if (bounds.width - 120.0).abs() < 1.0 && (*corner_radius - 28.0).abs() < 0.5
    )), "FAB press must ripple with its 28dp pill bounds");

    // Click inside email_row's left edge padding (x=6, y=40): full edge-to-edge click boundary!
    let row_edge_pt = [6.0, 40.0];
    ui.frame(Input { pointer: Some(row_edge_pt), pressed: true, time: 2.0, ..Input::default() }, b, 0.0, |f| {
        COORD.render(f, b, &mut state);
    });
    let mut row_action = None;
    ui.frame(Input { pointer: Some(row_edge_pt), released: true, time: 2.05, ..Input::default() }, b, 0.0, |f| {
        row_action = COORD.render(f, b, &mut state);
    });
    assert_eq!(row_action, Some("email_row"), "Full edge-to-edge row padding must click email_row");

    // Standalone row (without FAB covering star_btn): clicking star_btn MUST hit star_btn with circular ripple, NOT email_row!
    let mut ui2 = Ui::default();
    let mut state2 = LayoutState::default();
    let star_pt = [334.0, 30.0]; // inside 36x36 star_btn at x=316..352, y=12..48
    ui2.frame(Input { pointer: Some(star_pt), pressed: true, time: 3.0, ..Input::default() }, b, 0.0, |f| {
        OVERLAID_ROW.render(f, b, &mut state2);
    });
    let mut star_action = None;
    let star_cmds = ui2.frame(Input { pointer: Some(star_pt), released: true, time: 3.05, ..Input::default() }, b, 0.0, |f| {
        star_action = OVERLAID_ROW.render(f, b, &mut state2);
    });
    assert_eq!(star_action, Some("star_btn"), "Clickable child inside clickable container must win click");
    assert!(star_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Ripple { bounds, corner_radius, .. }
            if (bounds.width - 36.0).abs() < 0.5 && (*corner_radius - 18.0).abs() < 0.5
    )), "Borderless ImageButton must paint circular (width/2) state-layer ripple");
    assert!(!star_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::Ripple { bounds, .. } if (bounds.width - 360.0).abs() < 1.0
    )), "Parent clickable container must NOT ripple when its child icon button is clicked");

    // ScrollView with zero padding (inner == b): clipped item below viewport MUST NOT receive clicks!
    static SCROLL_ROWS: &[Node] = &[
        Node { kind: Kind::Button, id: "top_btn", width: Size::Match, height: Size::Dp(120.0), text: "Top", ..button("top_btn", Visibility::Visible, Size::Dp(120.0), 0.0) },
        Node { kind: Kind::Button, id: "offscreen_btn", width: Size::Match, height: Size::Dp(120.0), text: "Offscreen", ..button("offscreen_btn", Visibility::Visible, Size::Dp(120.0), 0.0) },
    ];
    static SCROLL_COL: &[Node] = &[Node {
        kind: Kind::Linear,
        width: Size::Match,
        height: Size::Wrap,
        children: SCROLL_ROWS,
        ..button("", Visibility::Visible, Size::Wrap, 0.0)
    }];
    static ZERO_PAD_SCROLL: Node = Node {
        kind: Kind::Scroll,
        id: "zero_pad_scroll",
        width: Size::Match,
        height: Size::Match,
        children: SCROLL_COL,
        ..button("zero_pad_scroll", Visibility::Visible, Size::Match, 0.0)
    };
    let scroll_vp = Rect { x: 0.0, y: 0.0, width: 300.0, height: 100.0 };
    let full_window = Rect { x: 0.0, y: 0.0, width: 300.0, height: 300.0 };
    let mut ui3 = Ui::default();
    let mut state3 = LayoutState::default();
    let outside_pt = [150.0, 160.0]; // inside offscreen_btn (y=120..240) but BELOW scroll_vp (y=0..100)!
    ui3.frame(Input { pointer: Some(outside_pt), pressed: true, time: 4.0, ..Input::default() }, full_window, 0.0, |f| {
        ZERO_PAD_SCROLL.render(f, scroll_vp, &mut state3);
    });
    let mut leaked = None;
    ui3.frame(Input { pointer: Some(outside_pt), released: true, time: 4.05, ..Input::default() }, full_window, 0.0, |f| {
        leaked = ZERO_PAD_SCROLL.render(f, scroll_vp, &mut state3);
    });
    assert_eq!(leaked, None, "Zero-padding ScrollView must clip hit-testing to its viewport bounds");
}

#[test]
fn wave16a_bottom_sheet_drag_detents_settle_flick_handle_and_reduced_motion() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_BOTTOM_SHEET, RULE_DRAG_HANDLE, SHEET_STATE_COLLAPSED,
        SHEET_STATE_EXPANDED, SHEET_STATE_HALF_EXPANDED, SHEET_STATE_HIDDEN,
    };

    // Hideable, fit_to_contents=0, half_ratio=0.5, peek=120dp -> detents: HIDDEN(0), COLLAPSED(120), HALF_EXPANDED(300), EXPANDED(600)
    static BS_HIDE_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_BOTTOM_SHEET,
        anchor: "120|1|0|0.5|0",
    }];
    // Non-hideable, fit_to_contents=1, peek=120dp -> detents: COLLAPSED(120), EXPANDED(300)
    static BS_FIXED_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_BOTTOM_SHEET,
        anchor: "120|0|1|0.5|0",
    }];
    static DH_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_DRAG_HANDLE,
        anchor: "",
    }];

    static HIDEABLE_ROOT: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Text,
                text: "Backdrop",
                ..button("w16_bg", Visibility::Visible, Size::Match, 0.0)
            },
            Node {
                kind: Kind::Linear,
                height: Size::Match,
                relative_rules: BS_HIDE_RULES,
                children: &[
                    Node {
                        kind: Kind::View,
                        height: Size::Wrap,
                        relative_rules: DH_RULES,
                        ..button("w16_dh", Visibility::Visible, Size::Match, 0.0)
                    },
                    Node {
                        kind: Kind::Button,
                        text: "Inner Btn",
                        height: Size::Dp(48.0),
                        ..button("w16_btn", Visibility::Visible, Size::Match, 0.0)
                    },
                ],
                ..button("w16_bs", Visibility::Visible, Size::Match, 0.0)
            },
        ],
        ..button("w16_bs_root", Visibility::Visible, Size::Match, 0.0)
    };

    static FIXED_ROOT: Node = Node {
        kind: Kind::Frame,
        children: &[Node {
            kind: Kind::Linear,
            height: Size::Dp(300.0),
            relative_rules: BS_FIXED_RULES,
            children: &[Node {
                kind: Kind::View,
                height: Size::Wrap,
                relative_rules: DH_RULES,
                ..button("w16_dh2", Visibility::Visible, Size::Match, 0.0)
            }],
            ..button("w16_bs2", Visibility::Visible, Size::Match, 0.0)
        }],
        ..button("w16_bs2_root", Visibility::Visible, Size::Match, 0.0)
    };

    let sheet_height = |cmds: &[DrawCommand]| -> Option<f32> {
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut active_clip: Option<Rect> = None;
        for c in cmds {
            match c {
                DrawCommand::PushClip(r) => active_clip = Some(*r),
                DrawCommand::PopClip => active_clip = None,
                DrawCommand::RoundedRect { bounds, color, .. } if *color == Color(0xF7, 0xF2, 0xFA, 255) => {
                    let vis = active_clip.map_or(*bounds, |cl| bounds.intersect(cl));
                    if vis.height > 0.0 {
                        min_y = min_y.min(vis.y);
                        max_y = max_y.max(vis.y + vis.height);
                    }
                }
                _ => {}
            }
        }
        (min_y < max_y).then_some(max_y - min_y)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 600.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Starts collapsed at 120dp (y=480..600). Tap inner button without dragging -> fires click!
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_height(&c0).unwrap() - 120.0).abs() < 1.0);
    // Inner button is below 48dp drag handle -> y = 528..576
    ui.frame(Input { pointer: Some([180.0, 550.0]), pressed: true, time: 1.1, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    let mut act = None;
    ui.frame(Input { pointer: Some([180.0, 552.0]), released: true, time: 1.15, ..Input::default() }, b, 0.0, |f| {
        act = HIDEABLE_ROOT.render(f, b, &mut state);
    });
    assert_eq!(act, Some("w16_btn"), "Short tap inside bottom sheet below 8dp slop must click child button");

    // 2. Drag upward from inner button by 220dp (120 -> 340dp): cancels button tap, tracks pointer,
    //    and on release snaps to nearest detent HALF_EXPANDED (300dp) with 0.25s ease-out settle!
    ui.frame(Input { pointer: Some([180.0, 550.0]), pressed: true, time: 2.0, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    let c_drag = ui.frame(Input { pointer: Some([180.0, 330.0]), time: 2.1, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    let h_drag = sheet_height(&c_drag).unwrap();
    assert!((h_drag - 340.0).abs() < 1.0, "Sheet must smoothly track upward drag, got {h_drag}");
    let mut drag_act = None;
    ui.frame(Input { pointer: Some([180.0, 330.0]), released: true, time: 2.25, ..Input::default() }, b, 0.0, |f| {
        drag_act = HIDEABLE_ROOT.render(f, b, &mut state);
    });
    assert_eq!(drag_act, None, "Dragging bottom sheet past 8dp slop must cancel inner child tap");
    assert_eq!(state.sheet_state.get("w16_bs"), Some(&SHEET_STATE_HALF_EXPANDED));
    // Mid-settle (t = 2.25 + 0.125 = 2.375, ease-out p=0.5 -> ease=0.75 -> 340 + (300 - 340)*0.75 = 310)
    let c_mid = ui.frame(Input { time: 2.375, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    let h_mid = sheet_height(&c_mid).unwrap();
    assert!(h_mid > 301.0 && h_mid < 339.0, "Expected smooth mid-settle height between 300 and 340, got {h_mid}");
    assert!(ui.needs_redraw());
    // End of settle (t = 2.52 >= 2.25 + 0.25)
    let c_end = ui.frame(Input { time: 2.52, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_height(&c_end).unwrap() - 300.0).abs() < 1.0);

    // 3. Drag down from HALF_EXPANDED (300) past half of peek (to h=40dp <= 60dp) -> snaps to HIDDEN and settles out
    ui.frame(Input { pointer: Some([180.0, 320.0]), pressed: true, time: 3.0, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    let c_low = ui.frame(Input { pointer: Some([180.0, 580.0]), time: 3.2, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_height(&c_low).unwrap() - 40.0).abs() < 1.0);
    ui.frame(Input { pointer: Some([180.0, 580.0]), released: true, time: 3.35, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    assert_eq!(state.sheet_state.get("w16_bs"), Some(&SHEET_STATE_HIDDEN));
    let c_hid = ui.frame(Input { time: 3.65, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    assert!(sheet_height(&c_hid).is_none(), "Hidden bottom sheet must disappear after settle finishes");

    // 4. Downward flick velocity (> 500dp/s) from COLLAPSED (120 -> 90dp in 0.03s = 1000dp/s) hides even above half-peek!
    state.sheet_state.insert("w16_bs".into(), SHEET_STATE_COLLAPSED);
    ui.frame(Input { time: 4.0, ..Input::default() }, b, 0.0, |f| { HIDEABLE_ROOT.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([180.0, 500.0]), pressed: true, time: 4.10, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([180.0, 530.0]), time: 4.13, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([180.0, 530.0]), released: true, time: 4.14, ..Input::default() }, b, 0.0, |f| {
        HIDEABLE_ROOT.render(f, b, &mut state);
    });
    assert_eq!(state.sheet_state.get("w16_bs"), Some(&SHEET_STATE_HIDDEN), "Fast downward flick must hide hideable bottom sheet");

    // 5. Clicking RULE_DRAG_HANDLE toggles COLLAPSED <-> EXPANDED (and non-hideable clamps downward drag at peek_h)
    let mut ui2 = Ui::default();
    let mut state2 = LayoutState::default();
    state2.theme_style = AndroidTheme::Material3Light;
    ui2.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| { FIXED_ROOT.render(f, b, &mut state2); });
    // Attempt dragging downward past peek on non-hideable sheet: clamps at 120dp and stays COLLAPSED
    ui2.frame(Input { pointer: Some([180.0, 500.0]), pressed: true, time: 1.1, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    let c_clamped = ui2.frame(Input { pointer: Some([180.0, 590.0]), time: 1.15, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    assert!((sheet_height(&c_clamped).unwrap() - 120.0).abs() < 1.0, "Non-hideable sheet must clamp at peek_h");
    ui2.frame(Input { pointer: Some([180.0, 590.0]), released: true, time: 1.2, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    assert_eq!(state2.sheet_state.get("w16_bs2"), Some(&SHEET_STATE_COLLAPSED));

    // Click drag handle (y=480..528) without dragging -> toggles COLLAPSED -> EXPANDED with animated settle
    ui2.frame(Input { pointer: Some([180.0, 504.0]), pressed: true, time: 2.0, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    ui2.frame(Input { pointer: Some([180.0, 504.0]), released: true, time: 2.05, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    assert_eq!(state2.sheet_state.get("w16_bs2"), Some(&SHEET_STATE_EXPANDED));
    let c_dh_mid = ui2.frame(Input { time: 2.175, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    let h_dh_mid = sheet_height(&c_dh_mid).unwrap();
    assert!(h_dh_mid > 125.0 && h_dh_mid < 295.0, "Drag handle toggle should animate height smoothly, got {h_dh_mid}");
    let c_dh_done = ui2.frame(Input { time: 2.35, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    assert!((sheet_height(&c_dh_done).unwrap() - 300.0).abs() < 1.0);

    // Under reduced motion, clicking drag handle again toggles EXPANDED -> COLLAPSED immediately without settle
    ui2.motion_preference = MotionPreference::Reduced;
    // Expanded drag handle is at y = 300..348
    ui2.frame(Input { pointer: Some([180.0, 324.0]), pressed: true, time: 3.0, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    ui2.frame(Input { pointer: Some([180.0, 324.0]), released: true, time: 3.05, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    assert_eq!(state2.sheet_state.get("w16_bs2"), Some(&SHEET_STATE_COLLAPSED));
    let c_rm = ui2.frame(Input { time: 3.06, ..Input::default() }, b, 0.0, |f| {
        FIXED_ROOT.render(f, b, &mut state2);
    });
    assert!((sheet_height(&c_rm).unwrap() - 120.0).abs() < 1.0, "Reduced motion must snap immediately");
}

#[test]
fn wave16a_modal_side_sheet_drag_to_dismiss_settle_rtl_and_reduced_motion() {
    use bhoo::android_xml::{AndroidTheme, RULE_SIDE_SHEET, SHEET_STATE_EXPANDED, SHEET_STATE_HIDDEN};

    static MODAL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SIDE_SHEET, anchor: "end|0" }];
    static MODAL_ROOT: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Text,
                text: "Main",
                ..button("w16_ssm_main", Visibility::Visible, Size::Match, 0.0)
            },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(200.0),
                height: Size::Match,
                relative_rules: MODAL_RULES,
                children: &[Node {
                    kind: Kind::Button,
                    text: "Sheet Btn",
                    height: Size::Dp(48.0),
                    ..button("w16_ssm_btn", Visibility::Visible, Size::Match, 0.0)
                }],
                ..button("w16_ssm", Visibility::Visible, Size::Dp(200.0), 0.0)
            },
        ],
        ..button("w16_ssm_root", Visibility::Visible, Size::Match, 0.0)
    };

    let sheet_x_and_scrim = |cmds: &[DrawCommand]| -> (Option<f32>, Option<u8>) {
        let mut sx = None;
        let mut scrim = None;
        for c in cmds {
            if let DrawCommand::RoundedRect { bounds, color, .. } = c {
                if color.0 == 0 && color.1 == 0 && color.2 == 0 && color.3 > 0 && color.3 <= 82 {
                    scrim = Some(color.3);
                } else if *color == Color(0xF7, 0xF2, 0xFA, 255) {
                    sx = Some(sx.map_or(bounds.x, |prev: f32| prev.min(bounds.x)));
                }
            }
        }
        (sx, scrim)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 500.0, height: 400.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Initial LTR right-docked modal side sheet at x = 300..500, scrim alpha = 82
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    let (x0, a0) = sheet_x_and_scrim(&c0);
    assert!((x0.unwrap() - 300.0).abs() < 1.0 && a0 == Some(82));

    // 2. Short outward drag (40px < 30% of 200 = 60px) translates sheet to x=340, fades scrim, cancels button tap,
    //    and on slow release settles smoothly back to x=300 without dismissing!
    ui.frame(Input { pointer: Some([350.0, 24.0]), pressed: true, time: 1.1, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    let c_short = ui.frame(Input { pointer: Some([390.0, 24.0]), time: 1.25, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    let (x_s, a_s) = sheet_x_and_scrim(&c_short);
    assert!((x_s.unwrap() - 340.0).abs() < 1.0, "Sheet must translate right by 40dp, got {:?}", x_s);
    assert!(a_s.unwrap() < 82 && a_s.unwrap() > 50, "Scrim alpha must fade proportionally, got {:?}", a_s);
    let mut short_act = None;
    ui.frame(Input { pointer: Some([390.0, 24.0]), released: true, time: 1.4, ..Input::default() }, b, 0.0, |f| {
        short_act = MODAL_ROOT.render(f, b, &mut state);
    });
    assert_eq!(short_act, None, "Horizontal drag must cancel inner button tap");
    assert_ne!(state.sheet_state.get("w16_ssm"), Some(&SHEET_STATE_HIDDEN));
    let c_back_mid = ui.frame(Input { time: 1.525, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    let (x_bm, _) = sheet_x_and_scrim(&c_back_mid);
    assert!(x_bm.unwrap() > 301.0 && x_bm.unwrap() < 339.0, "Expected smooth settle back toward 300, got {:?}", x_bm);
    let c_back_done = ui.frame(Input { time: 1.7, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_x_and_scrim(&c_back_done).0.unwrap() - 300.0).abs() < 1.0);

    // 3. Drag past 30% (80px > 60px) dismisses to SHEET_STATE_HIDDEN and slides out over 0.25s
    ui.frame(Input { pointer: Some([350.0, 100.0]), pressed: true, time: 2.0, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([430.0, 100.0]), time: 2.2, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([430.0, 100.0]), released: true, time: 2.35, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    assert_eq!(state.sheet_state.get("w16_ssm"), Some(&SHEET_STATE_HIDDEN));
    let c_out = ui.frame(Input { time: 2.65, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    assert!(sheet_x_and_scrim(&c_out).0.is_none(), "Dismissed modal side sheet must disappear after settle");

    // 4. RTL: end-docked side sheet sits on physical LEFT (x=0..200); dragging LEFT dismisses under reduced motion immediately
    state.rtl = true;
    state.sheet_state.insert("w16_ssm".into(), SHEET_STATE_EXPANDED);
    ui.motion_preference = MotionPreference::Reduced;
    let c_rtl0 = ui.frame(Input { time: 3.0, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_x_and_scrim(&c_rtl0).0.unwrap() - 0.0).abs() < 1.0);
    ui.frame(Input { pointer: Some([150.0, 100.0]), pressed: true, time: 3.1, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    let c_rtl_drag = ui.frame(Input { pointer: Some([80.0, 100.0]), time: 3.2, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_x_and_scrim(&c_rtl_drag).0.unwrap() + 70.0).abs() < 1.0, "RTL left-docked sheet must slide leftward");
    let c_rtl_rel = ui.frame(Input { pointer: Some([80.0, 100.0]), released: true, time: 3.25, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    assert_eq!(state.sheet_state.get("w16_ssm"), Some(&SHEET_STATE_HIDDEN));
    assert!(sheet_x_and_scrim(&c_rtl_rel).0.is_none(), "Reduced motion must hide dismissed side sheet immediately");

    // 5. Fast outward flick velocity (> 500dp/s, only 30px < 60px threshold) dismisses immediately!
    state.rtl = false;
    state.sheet_state.insert("w16_ssm".into(), SHEET_STATE_EXPANDED);
    ui.frame(Input { time: 4.0, ..Input::default() }, b, 0.0, |f| { MODAL_ROOT.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([350.0, 100.0]), pressed: true, time: 4.10, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([380.0, 100.0]), time: 4.12, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([380.0, 100.0]), released: true, time: 4.13, ..Input::default() }, b, 0.0, |f| {
        MODAL_ROOT.render(f, b, &mut state);
    });
    assert_eq!(state.sheet_state.get("w16_ssm"), Some(&SHEET_STATE_HIDDEN), "Fast outward flick must dismiss modal side sheet");
}

#[test]
fn wave16a_drawer_layout_animated_slide_scrim_fade_drag_close_and_reduced_motion() {
    use bhoo::android_xml::{AndroidTheme, RULE_DRAWER_LAYOUT, RULE_NAV_BAR, RULE_NAV_ITEM};

    static DL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_DRAWER_LAYOUT, anchor: "" }];
    static NAV_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "drawer|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "w16_d_home|Home|home" },
    ];
    static DRAWER: Node = Node {
        kind: Kind::Frame,
        relative_rules: DL_RULES,
        children: &[
            Node { kind: Kind::Text, text: "Main", ..button("w16_dl_main", Visibility::Visible, Size::Match, 0.0) },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(240.0),
                height: Size::Match,
                layout_gravity: [0, 0],
                layout_gravity_explicit: true,
                relative_rules: NAV_RULES,
                ..button("w16_dl_panel", Visibility::Visible, Size::Dp(240.0), 0.0)
            },
        ],
        ..button("w16_dl", Visibility::Visible, Size::Match, 0.0)
    };

    let drawer_right_and_scrim = |cmds: &[DrawCommand]| -> (Option<f32>, Option<u8>) {
        let mut right_edge = None;
        let mut scrim = None;
        for c in cmds {
            if let DrawCommand::RoundedRect { bounds, color, .. } = c {
                if color.0 == 0 && color.1 == 0 && color.2 == 0 && color.3 > 0 && color.3 <= 82 {
                    scrim = Some(color.3);
                } else if *color == Color(0xF7, 0xF2, 0xFA, 255) {
                    right_edge = Some(right_edge.map_or(bounds.x + bounds.width, |p: f32| p.max(bounds.x + bounds.width)));
                }
            }
        }
        (right_edge, scrim)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 300.0 };

    // 1. Initial-open snapshot on frame 1 starts at progress = 1.0 immediately
    {
        let mut ui = Ui::default();
        let mut state = LayoutState::default();
        state.theme_style = AndroidTheme::Material3Light;
        state.drawer_open.insert("w16_dl".into(), true);
        let c = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| {
            DRAWER.render(f, b, &mut state);
        });
        let (re, sc) = drawer_right_and_scrim(&c);
        assert!((re.unwrap() - 240.0).abs() < 0.5 && sc == Some(82), "Initial-open drawer must start at full progress 1.0");
    }

    // 2. Subsequent toggle animates slide-in & scrim alpha fade over 0.25s
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    ui.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| { DRAWER.render(f, b, &mut state); });
    state.drawer_open.insert("w16_dl".into(), true);
    ui.frame(Input { time: 2.0, ..Input::default() }, b, 0.0, |f| { DRAWER.render(f, b, &mut state); });
    assert!(ui.needs_redraw());
    let c_mid = ui.frame(Input { time: 2.125, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    let (re_mid, sc_mid) = drawer_right_and_scrim(&c_mid);
    assert!(re_mid.unwrap() > 100.0 && re_mid.unwrap() < 235.0, "Mid-slide drawer right edge should interpolate, got {:?}", re_mid);
    assert!(sc_mid.unwrap() > 20 && sc_mid.unwrap() < 80, "Scrim alpha must fade proportionally, got {:?}", sc_mid);
    let c_open = ui.frame(Input { time: 2.30, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    let (re_open, sc_open) = drawer_right_and_scrim(&c_open);
    assert!((re_open.unwrap() - 240.0).abs() < 0.5 && sc_open == Some(82));

    // 3. Horizontal drag-to-close short (< 30% of 240 = 72dp): cancels nav click & settles back open
    ui.frame(Input { pointer: Some([180.0, 40.0]), pressed: true, time: 3.0, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    let c_dshort = ui.frame(Input { pointer: Some([130.0, 40.0]), time: 3.15, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    assert!((drawer_right_and_scrim(&c_dshort).0.unwrap() - 190.0).abs() < 1.0);
    let mut act = None;
    ui.frame(Input { pointer: Some([130.0, 40.0]), released: true, time: 3.3, ..Input::default() }, b, 0.0, |f| {
        act = DRAWER.render(f, b, &mut state);
    });
    assert_eq!(act, None, "Dragging drawer must cancel inner nav tap");
    assert_eq!(state.drawer_open.get("w16_dl"), Some(&true));
    let c_reopen = ui.frame(Input { time: 3.6, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    assert!((drawer_right_and_scrim(&c_reopen).0.unwrap() - 240.0).abs() < 0.5);

    // 4. Horizontal drag-to-close past 30% (100dp > 72dp): closes drawer_open=false and animates out
    ui.frame(Input { pointer: Some([200.0, 100.0]), pressed: true, time: 4.0, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([100.0, 100.0]), time: 4.15, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([100.0, 100.0]), released: true, time: 4.3, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    assert_eq!(state.drawer_open.get("w16_dl"), Some(&false));
    let c_closed = ui.frame(Input { time: 4.6, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    assert!(drawer_right_and_scrim(&c_closed).0.is_none());

    // 5. Reduced motion: programmatic open & close snap immediately on the same frame
    ui.motion_preference = MotionPreference::Reduced;
    state.drawer_open.insert("w16_dl".into(), true);
    let c_rm_open = ui.frame(Input { time: 5.0, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    assert!((drawer_right_and_scrim(&c_rm_open).0.unwrap() - 240.0).abs() < 0.5 && drawer_right_and_scrim(&c_rm_open).1 == Some(82));
    state.drawer_open.insert("w16_dl".into(), false);
    let c_rm_closed = ui.frame(Input { time: 5.1, ..Input::default() }, b, 0.0, |f| {
        DRAWER.render(f, b, &mut state);
    });
    assert!(drawer_right_and_scrim(&c_rm_closed).0.is_none());
}

#[test]
fn wave16a_coordinator_snackbar_aware_fab_dodge() {
    use bhoo::android_xml::{AndroidTheme, RULE_ELEVATION, RULE_EXTENDED_FAB, RULE_SNACKBAR};

    static SNACK_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SNACKBAR, anchor: "" }];
    static EFAB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_EXTENDED_FAB, anchor: "1" }];
    static ICON_FAB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_ELEVATION, anchor: "6" }];

    static COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Linear,
                horizontal: true,
                width: Size::Match,
                height: Size::Dp(48.0),
                margin: [16.0, 0.0, 16.0, 16.0],
                layout_gravity: [1, 2],
                layout_gravity_explicit: true,
                relative_rules: SNACK_RULES,
                children: &[Node {
                    kind: Kind::Text,
                    text: "Saved",
                    ..button("w16_sn_txt", Visibility::Visible, Size::Wrap, 0.0)
                }],
                ..button("w16_snack", Visibility::Visible, Size::Match, 0.0)
            },
            Node {
                kind: Kind::Button,
                text: "Compose",
                width: Size::Wrap,
                height: Size::Dp(56.0),
                margin: [0.0, 0.0, 16.0, 16.0],
                layout_gravity: [2, 2],
                layout_gravity_explicit: true,
                relative_rules: EFAB_RULES,
                ..button("w16_efab", Visibility::Visible, Size::Wrap, 0.0)
            },
            Node {
                kind: Kind::Icon,
                width: Size::Dp(56.0),
                height: Size::Dp(56.0),
                margin: [16.0, 0.0, 0.0, 16.0],
                layout_gravity: [0, 2],
                layout_gravity_explicit: true,
                background: Some(Color(100, 180, 240, 255)),
                relative_rules: ICON_FAB_RULES,
                ..button("w16_ifab", Visibility::Visible, Size::Dp(56.0), 0.0)
            },
            Node {
                kind: Kind::Button,
                text: "TopEfab",
                width: Size::Wrap,
                height: Size::Dp(56.0),
                margin: [0.0, 16.0, 16.0, 0.0],
                layout_gravity: [2, 0],
                layout_gravity_explicit: true,
                relative_rules: EFAB_RULES,
                ..button("w16_top_efab", Visibility::Visible, Size::Wrap, 0.0)
            },
            Node {
                kind: Kind::Button,
                text: "FlatBtn",
                width: Size::Dp(80.0),
                height: Size::Dp(40.0),
                margin: [0.0, 0.0, 0.0, 16.0],
                layout_gravity: [1, 2],
                layout_gravity_explicit: true,
                ..button("w16_flat", Visibility::Visible, Size::Dp(80.0), 0.0)
            },
        ],
        ..button("w16_coord", Visibility::Visible, Size::Match, 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 600.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. With Snackbar visible (y=536..584, h=48, margin_b=16 -> shift = 48+16+8 = 72dp):
    //    Bottom ExtendedFAB & Icon FAB shift upward from y=528 to y=456 (bottom=512 <= snack.y - 8 = 528)
    let cmds_vis = ui.frame(Input::default(), b, 0.0, |f| { COORD.render(f, b, &mut state); });
    let snack_rect = cmds_vis
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color: Color(0x32, 0x2F, 0x35, 255), .. } => Some(*bounds),
            _ => None,
        })
        .expect("Snackbar rect missing");
    let efab_y = ui.semantics().iter().find(|s| s.label.as_ref() == "Compose").unwrap().bounds.y;
    let ifab_rect = cmds_vis
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color: Color(100, 180, 240, 255), .. } => Some(*bounds),
            _ => None,
        })
        .expect("Icon FAB rect missing");
    let top_efab_y = ui.semantics().iter().find(|s| s.label.as_ref() == "TopEfab").unwrap().bounds.y;
    let flat_btn_y = ui.semantics().iter().find(|s| s.label.as_ref() == "FlatBtn").unwrap().bounds.y;

    assert!((snack_rect.y - 536.0).abs() < 0.5);
    assert!(efab_y + 56.0 <= snack_rect.y - 8.0 + 0.5, "ExtendedFAB must dodge above Snackbar without overlap, y={efab_y}");
    assert!((efab_y - 456.0).abs() < 1.0, "Expected ExtendedFAB shifted to y=456, got {efab_y}");
    assert!((ifab_rect.y - 456.0).abs() < 1.0, "Expected elevated Icon FAB shifted to y=456, got {}", ifab_rect.y);
    assert!((top_efab_y - 16.0).abs() < 0.5, "Top-aligned ExtendedFAB must not be shifted");
    assert!((flat_btn_y - 544.0).abs() < 0.5, "Non-FAB flat button must not dodge");

    // 2. When Snackbar is Gone: FABs return to normal bottom position y=528
    state.visibility.insert("w16_snack".into(), Visibility::Gone);
    let cmds_gone = ui.frame(Input::default(), b, 0.0, |f| { COORD.render(f, b, &mut state); });
    let efab_y_gone = ui.semantics().iter().find(|s| s.label.as_ref() == "Compose").unwrap().bounds.y;
    let ifab_y_gone = cmds_gone
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color: Color(100, 180, 240, 255), .. } => Some(bounds.y),
            _ => None,
        })
        .unwrap();
    assert!((efab_y_gone - 528.0).abs() < 0.5 && (ifab_y_gone - 528.0).abs() < 0.5);
}

#[test]
fn wave16a_tab_layout_view_pager_two_way_sync() {
    use bhoo::android_xml::{AndroidTheme, RULE_TAB_OPTIONS, RULE_VIEW_PAGER};

    static TAB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_TAB_OPTIONS, anchor: "fixed|primary|" }];
    static PAGER_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_VIEW_PAGER, anchor: "horizontal" }];

    static ROOT: Node = Node {
        kind: Kind::Linear,
        horizontal: false,
        children: &[
            Node {
                kind: Kind::TabWidget,
                width: Size::Match,
                height: Size::Dp(48.0),
                relative_rules: TAB_RULES,
                children: &[
                    Node { kind: Kind::Text, text: "Tab 0", align: 1, content_gravity: [1, 1], ..button("w16_t0", Visibility::Visible, Size::Match, 0.0) },
                    Node { kind: Kind::Text, text: "Tab 1", align: 1, content_gravity: [1, 1], ..button("w16_t1", Visibility::Visible, Size::Match, 0.0) },
                    Node { kind: Kind::Text, text: "Tab 2", align: 1, content_gravity: [1, 1], ..button("w16_t2", Visibility::Visible, Size::Match, 0.0) },
                ],
                ..button("w16_tabs", Visibility::Visible, Size::Match, 0.0)
            },
            Node {
                kind: Kind::ViewFlipper,
                width: Size::Match,
                height: Size::Dp(200.0),
                relative_rules: PAGER_RULES,
                children: &[
                    Node { kind: Kind::Text, text: "Page Zero", ..button("w16_p0", Visibility::Visible, Size::Match, 0.0) },
                    Node { kind: Kind::Text, text: "Page One", ..button("w16_p1", Visibility::Visible, Size::Match, 0.0) },
                    Node { kind: Kind::Text, text: "Page Two", ..button("w16_p2", Visibility::Visible, Size::Match, 0.0) },
                ],
                ..button("w16_pager", Visibility::Visible, Size::Match, 0.0)
            },
        ],
        ..button("w16_tp_root", Visibility::Visible, Size::Match, 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 300.0, height: 248.0 };
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    state.tab_pager_links.insert("w16_tabs".into(), "w16_pager".into());

    // 1. Initial render: Tab 0 / Page Zero
    let c0 = ui.frame(Input::default(), b, 0.0, |f| { ROOT.render(f, b, &mut state); });
    assert!(c0.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page Zero")));

    // 2. Tap Tab 2 (x = 250, y = 24): immediately updates ViewPager2's selected_index & displayed_children -> Page Two!
    ui.frame(Input { pointer: Some([250.0, 24.0]), pressed: true, time: 0.1, ..Input::default() }, b, 0.0, |f| {
        ROOT.render(f, b, &mut state);
    });
    let c_tab2 = ui.frame(Input { pointer: Some([250.0, 24.0]), released: true, time: 0.15, ..Input::default() }, b, 0.0, |f| {
        ROOT.render(f, b, &mut state);
    });
    assert_eq!(state.selected_index.get("w16_pager"), Some(&2));
    assert_eq!(state.displayed_children.get("w16_pager"), Some(&2));
    assert!(c_tab2.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page Two")));

    // 3. Swipe ViewPager2 right (retreat page 2 -> 1): immediately updates linked TabWidget's selected_index!
    ui.frame(Input { pointer: Some([60.0, 150.0]), pressed: true, time: 0.2, ..Input::default() }, b, 0.0, |f| {
        ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([240.0, 150.0]), time: 0.25, ..Input::default() }, b, 0.0, |f| {
        ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([240.0, 150.0]), released: true, time: 0.3, ..Input::default() }, b, 0.0, |f| {
        ROOT.render(f, b, &mut state);
    });
    assert_eq!(state.selected_index.get("w16_pager"), Some(&1));
    assert_eq!(state.selected_index.get("w16_tabs"), Some(&1));
    assert_eq!(state.displayed_children.get("w16_tabs"), Some(&1));

    // 4. Reverse link (pager -> tabs) + keyboard arrow key on focused ViewPager2 syncs TabWidget too!
    state.tab_pager_links.clear();
    state.tab_pager_links.insert("w16_pager".into(), "w16_tabs".into());
    ui.frame(Input { time: 0.4, ..Input::default() }, b, 0.0, |f| {
        f.request_focus(bhoo::android_xml::view_id("w16_pager"));
        ROOT.render(f, b, &mut state);
    });
    ui.frame(
        Input {
            key: Some(KeyInput { key: Key::Left, pressed: true, shift: false, repeat: false }),
            time: 0.45,
            ..Input::default()
        },
        b,
        0.0,
        |f| {
            ROOT.render(f, b, &mut state);
        },
    );
    assert_eq!(state.selected_index.get("w16_pager"), Some(&0));
    assert_eq!(state.selected_index.get("w16_tabs"), Some(&0));
    assert_eq!(state.displayed_children.get("w16_tabs"), Some(&0));
}

#[test]
fn wave16a_scale_and_rotate_transition_animations_and_reduced_motion() {
    use bhoo::android_xml::{RULE_ROTATE_IN, RULE_ROTATE_OUT, RULE_SCALE_IN, RULE_SCALE_OUT};

    // Same-pivot combined scale + rotate on incoming, scale + rotate on outgoing
    static SAME_PIVOT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_SCALE_IN, anchor: "0.5,1.0,0.5,1.0|center,center|0.4|0.0|1" },
        RelativeRule { kind: RULE_ROTATE_IN, anchor: "-90,0|50%,50%|0.4|0.0|1" },
        RelativeRule { kind: RULE_SCALE_OUT, anchor: "1.0,0.2,1.0,0.2|center,center|0.4|0.0|1" },
        RelativeRule { kind: RULE_ROTATE_OUT, anchor: "0,90|50%,50%|0.4|0.0|1" },
    ];
    // Different pivots (%p parent pivot for rotate, dp pivot for scale) + start_offset
    static DIFF_PIVOT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_SCALE_IN, anchor: "50%,100%,50%,100%|10dp,5dp|0.4|0.1|1" },
        RelativeRule { kind: RULE_ROTATE_IN, anchor: "0,180|50%p,50%p|0.4|0.0|1" },
    ];

    static SWITCHER_SAME: Node = Node {
        kind: Kind::ViewSwitcher,
        relative_rules: SAME_PIVOT_RULES,
        children: &[
            Node { kind: Kind::View, background: Some(Color(255, 0, 0, 255)), ..button("w16_s1", Visibility::Visible, Size::Match, 0.0) },
            Node { kind: Kind::View, background: Some(Color(0, 0, 255, 255)), ..button("w16_s2", Visibility::Visible, Size::Match, 0.0) },
        ],
        ..button("w16_sw_same", Visibility::Visible, Size::Match, 0.0)
    };
    static SWITCHER_DIFF: Node = Node {
        kind: Kind::ViewSwitcher,
        relative_rules: DIFF_PIVOT_RULES,
        children: &[
            Node { kind: Kind::View, background: Some(Color(10, 20, 30, 255)), ..button("w16_d1", Visibility::Visible, Size::Dp(100.0), 0.0) },
            Node { kind: Kind::View, width: Size::Dp(100.0), height: Size::Dp(40.0), background: Some(Color(30, 60, 90, 255)), ..button("w16_d2", Visibility::Visible, Size::Dp(100.0), 0.0) },
        ],
        ..button("w16_sw_diff", Visibility::Visible, Size::Match, 0.0)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 200.0, height: 80.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // 1. Initial child 0: no transition active -> no PushTransform
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| {
        SWITCHER_SAME.render(f, b, &mut state);
    });
    assert!(!c0.iter().any(|c| matches!(c, DrawCommand::PushTransform { .. })));

    // 2. Switch to child 1 at t=2.0, sample midway at t=2.2 (p=0.5 linear):
    //    Outgoing: scale=[0.6, 0.6], rot=+45deg around [100, 40]
    //    Incoming: scale=[0.75, 0.75], rot=-45deg around [100, 40]
    state.displayed_children.insert("w16_sw_same".into(), 1);
    ui.frame(Input { time: 2.0, ..Input::default() }, b, 0.0, |f| {
        SWITCHER_SAME.render(f, b, &mut state);
    });
    let c_mid = ui.frame(Input { time: 2.2, ..Input::default() }, b, 0.0, |f| {
        SWITCHER_SAME.render(f, b, &mut state);
    });
    assert!(ui.needs_redraw());
    let push_cnt = c_mid.iter().filter(|c| matches!(c, DrawCommand::PushTransform { .. })).count();
    let pop_cnt = c_mid.iter().filter(|c| matches!(c, DrawCommand::PopTransform)).count();
    assert_eq!(push_cnt, 2, "Expected 1 combined PushTransform for outgoing + 1 for incoming");
    assert_eq!(pop_cnt, 2, "PushTransform and PopTransform must stay balanced");
    assert!(c_mid.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, scale, rotation_rad, .. }
            if (*origin == [100.0, 40.0])
                && (scale[0] - 0.6).abs() < 1e-4
                && (*rotation_rad - 45.0_f32.to_radians()).abs() < 1e-4
    )), "Outgoing child must sample scale=0.6 and rot=+45deg at midpoint");
    assert!(c_mid.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, scale, rotation_rad, .. }
            if (*origin == [100.0, 40.0])
                && (scale[0] - 0.75).abs() < 1e-4
                && (*rotation_rad - (-45.0_f32).to_radians()).abs() < 1e-4
    )), "Incoming child must sample scale=0.75 and rot=-45deg at midpoint");

    // 3. After completion (t=2.45 >= 2.4): outgoing gone, no PushTransform emitted
    let c_done = ui.frame(Input { time: 2.45, ..Input::default() }, b, 0.0, |f| {
        SWITCHER_SAME.render(f, b, &mut state);
    });
    assert!(!c_done.iter().any(|c| matches!(c, DrawCommand::PushTransform { .. })));
    assert!(!c_done.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(255, 0, 0, _), .. })));

    // 4. Reduced motion: switching back to child 0 emits ZERO PushTransform
    ui.motion_preference = MotionPreference::Reduced;
    state.displayed_children.insert("w16_sw_same".into(), 0);
    let c_rm = ui.frame(Input { time: 3.0, ..Input::default() }, b, 0.0, |f| {
        SWITCHER_SAME.render(f, b, &mut state);
    });
    assert!(!c_rm.iter().any(|c| matches!(c, DrawCommand::PushTransform { .. })));
    assert!(c_rm.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(255, 0, 0, 255), .. })));

    // 5. Different pivots (parent %p vs child dp) + start_offset: emits 2 nested PushTransforms for incoming child
    ui.motion_preference = MotionPreference::Full;
    ui.frame(Input { time: 4.0, ..Input::default() }, b, 0.0, |f| { SWITCHER_DIFF.render(f, b, &mut state); });
    state.displayed_children.insert("w16_sw_diff".into(), 1);
    ui.frame(Input { time: 5.0, ..Input::default() }, b, 0.0, |f| { SWITCHER_DIFF.render(f, b, &mut state); });
    // At t=5.2: rotate is at 0.2/0.4=0.5 -> 90deg around parent center [100, 40];
    // scale has start_offset=0.1 -> (0.2-0.1)/0.4 = 0.25 -> 0.5 + 0.5*0.25 = 0.625 around [10, 5]
    let c_diff = ui.frame(Input { time: 5.2, ..Input::default() }, b, 0.0, |f| {
        SWITCHER_DIFF.render(f, b, &mut state);
    });
    assert!(c_diff.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, rotation_rad, .. }
            if (*origin == [100.0, 40.0]) && (*rotation_rad - 90.0_f32.to_radians()).abs() < 1e-4
    )));
    assert!(c_diff.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, scale, .. }
            if (*origin == [10.0, 5.0]) && (scale[0] - 0.625).abs() < 1e-4
    )));
    assert_eq!(
        c_diff.iter().filter(|c| matches!(c, DrawCommand::PushTransform { .. })).count(),
        c_diff.iter().filter(|c| matches!(c, DrawCommand::PopTransform)).count()
    );

    // 6. AdapterViewFlipper scale + rotate playback & reduced motion
    static AVF_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_SCALE_IN, anchor: "0.5,1.0,0.5,1.0|center,center|0.4|0.0|1" },
        RelativeRule { kind: RULE_ROTATE_IN, anchor: "0,90|center,center|0.4|0.0|1" },
    ];
    static AVF: Node = Node {
        kind: Kind::AdapterViewFlipper,
        relative_rules: AVF_RULES,
        ..button("w16_avf", Visibility::Visible, Size::Match, 0.0)
    };
    state.list_items.insert("w16_avf".into(), vec!["Item A".into(), "Item B".into()]);
    ui.frame(Input { time: 6.0, ..Input::default() }, b, 0.0, |f| { AVF.render(f, b, &mut state); });
    state.displayed_children.insert("w16_avf".into(), 1);
    ui.frame(Input { time: 7.0, ..Input::default() }, b, 0.0, |f| { AVF.render(f, b, &mut state); });
    let c_avf = ui.frame(Input { time: 7.2, ..Input::default() }, b, 0.0, |f| { AVF.render(f, b, &mut state); });
    assert!(c_avf.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, scale, rotation_rad, .. }
            if (*origin == [100.0, 40.0])
                && (scale[0] - 0.75).abs() < 1e-4
                && (*rotation_rad - 45.0_f32.to_radians()).abs() < 1e-4
    )));
    assert_eq!(
        c_avf.iter().filter(|c| matches!(c, DrawCommand::PushTransform { .. })).count(),
        c_avf.iter().filter(|c| matches!(c, DrawCommand::PopTransform)).count()
    );
}

#[test]
fn wave17a_navigation_drawer_header_layout_divider_and_scrolling() {
    use bhoo::android_xml::{AndroidTheme, RULE_NAV_BAR, RULE_NAV_ITEM};

    static HEADER_CHILDREN: &[Node] = &[
        Node {
            kind: Kind::Text,
            text: "Workspace",
            height: Size::Dp(40.0),
            ..button("hdr_title", Visibility::Visible, Size::Match, 0.0)
        },
        Node {
            kind: Kind::Button,
            text: "Switch",
            height: Size::Dp(48.0),
            clickable: true,
            ..button("hdr_btn", Visibility::Visible, Size::Match, 0.0)
        },
    ];
    static DRAWER_CHILDREN: &[Node] = &[
        Node {
            kind: Kind::Linear,
            width: Size::Match,
            height: Size::Dp(120.0),
            background: Some(Color(30, 40, 50, 255)),
            children: HEADER_CHILDREN,
            ..button("drawer_hdr", Visibility::Visible, Size::Match, 0.0)
        },
        Node {
            kind: Kind::View,
            height: Size::Dp(60.0),
            ..button("hdr_gone", Visibility::Gone, Size::Match, 0.0)
        },
    ];
    static DRAWER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "drawer|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_inbox|Inbox|home" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_sent|Sent|send" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_drafts|Drafts|edit" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_spam|Spam|info" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_trash|Trash|delete" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_settings|Settings|settings" },
    ];
    static DRAWER: Node = Node {
        kind: Kind::Linear,
        width: Size::Dp(300.0),
        height: Size::Match,
        relative_rules: DRAWER_RULES,
        children: DRAWER_CHILDREN,
        ..button("nav_drawer", Visibility::Visible, Size::Dp(300.0), 0.0)
    };
    static WRAP_DRAWER: Node = Node {
        height: Size::Wrap,
        ..DRAWER
    };
    static WRAP_HOST: Node = Node {
        kind: Kind::Linear,
        children: std::slice::from_ref(&WRAP_DRAWER),
        ..button("wrap_host", Visibility::Visible, Size::Match, 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Tall viewport: header at y=0..120, 1dp outline_variant divider at y=120..121, nav items start at y=133
    let tall = Rect { x: 0.0, y: 0.0, width: 300.0, height: 600.0 };
    let cmds = ui.frame(Input::default(), tall, 0.0, |f| {
        DRAWER.render(f, tall, &mut state);
    });
    let hdr_bg = cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color: Color(30, 40, 50, 255), .. } => Some(*bounds),
            _ => None,
        })
        .expect("drawer header background should render");
    assert_eq!(hdr_bg, Rect { x: 0.0, y: 0.0, width: 300.0, height: 120.0 });
    assert!(
        cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, bounds, .. } if text.as_ref() == "Workspace" && bounds.y < 120.0)),
        "header title text must render inside header"
    );
    // 1dp outline_variant (M3Light: 0xCA, 0xC4, 0xD0, 255) divider directly below header
    assert!(
        cmds.iter().any(|c| matches!(
            c,
            DrawCommand::RoundedRect { bounds, color: Color(0xCA, 0xC4, 0xD0, 255), .. }
                if (bounds.y - 120.0).abs() < 0.05 && (bounds.height - 1.0).abs() < 0.05 && (bounds.width - 300.0).abs() < 0.05
        )),
        "expected 1dp outline_variant divider at y=120 below drawer header"
    );
    let inbox_y = cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { text, bounds, .. } if text.as_ref() == "Inbox" => Some(bounds.y),
            _ => None,
        })
        .expect("Inbox nav item text missing");
    assert!(
        (inbox_y - 133.0).abs() < 0.1,
        "first nav item should start at y = 120 + 1 + 12 = 133, got {inbox_y}"
    );

    // 2. Clicking a button inside the header layout fires its action
    let mut act = None;
    ui.frame(Input { pointer: Some([100.0, 64.0]), pressed: true, ..Input::default() }, tall, 0.0, |f| {
        DRAWER.render(f, tall, &mut state);
    });
    ui.frame(Input { pointer: Some([100.0, 64.0]), released: true, ..Input::default() }, tall, 0.0, |f| {
        act = DRAWER.render(f, tall, &mut state);
    });
    assert_eq!(act, Some("hdr_btn"));

    // 3. Wrap-content height measurement accounts for header + 1dp divider + items
    let wrap_bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 800.0 };
    let wrap_cmds = ui.frame(Input::default(), wrap_bounds, 0.0, |f| {
        WRAP_HOST.render(f, wrap_bounds, &mut state);
    });
    // Expected height: 120 (header) + 1 (divider) + 6*56 (items) + 24 (top+bottom pad) = 481dp
    let drawer_bg_h = wrap_cmds
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color: Color(0xF7, 0xF2, 0xFA, 255), .. } => Some(bounds.height),
            _ => None,
        })
        .expect("wrap drawer surface missing");
    assert!((drawer_bg_h - 481.0).abs() < 0.5, "expected wrap drawer height ~481, got {drawer_bg_h}");

    // 4. Short viewport triggers vertical scrolling via state.offsets and clips to inner
    let short = Rect { x: 0.0, y: 0.0, width: 300.0, height: 240.0 };
    let cmds_scroll = ui.frame(
        Input { pointer: Some([150.0, 120.0]), scroll: [0.0, 80.0], ..Input::default() },
        short,
        0.0,
        |f| {
            DRAWER.render(f, short, &mut state);
        },
    );
    let scrolled_off = state.offsets.get("nav_drawer").copied().unwrap_or(0.0);
    assert!(scrolled_off > 0.0, "drawer should scroll when header + items exceed viewport height");
    assert!(
        cmds_scroll.iter().any(|c| matches!(c, DrawCommand::PushClip(r) if *r == short)),
        "scrolling drawer must clip content to inner bounds"
    );
    let scrolled_inbox_y = cmds_scroll
        .iter()
        .find_map(|c| match c {
            DrawCommand::Text { text, bounds, .. } if text.as_ref() == "Inbox" => Some(bounds.y),
            _ => None,
        })
        .unwrap();
    assert!(
        (scrolled_inbox_y - (133.0 - scrolled_off)).abs() < 0.5,
        "nav items must shift upward by scroll offset ({scrolled_off}), got y={scrolled_inbox_y}"
    );
}

#[test]
fn wave17a_radial_and_sweep_gradients_raster_cache_and_clipping() {
    use bhoo::android_xml::{RULE_BACKGROUND_TINT, RULE_GRADIENT};

    static RADIAL_2STOP: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GRADIENT, anchor: "radial|0.5,0.5,0|255,0,0,255||0,0,255,255" },
    ];
    static RADIAL_3STOP: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GRADIENT, anchor: "radial|0.5,0.5,50|255,0,0,255|0,255,0,255|0,0,255,255" },
    ];
    static RADIAL_TINTED: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GRADIENT, anchor: "radial|0.5,0.5,50|255,0,0,255||0,0,255,0" },
        RelativeRule { kind: RULE_BACKGROUND_TINT, anchor: "220,40,60,128" },
    ];
    static SWEEP_3STOP: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GRADIENT, anchor: "sweep|0.5,0.5|255,0,0,255|0,255,0,255|0,0,255,255" },
    ];
    static LINEAR_PIPE: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GRADIENT, anchor: "0|255,0,0,255||0,0,255,255" },
    ];

    static RADIAL_VIEW: Node = Node {
        kind: Kind::View,
        radius: 16.0,
        relative_rules: RADIAL_2STOP,
        ..button("rad_v", Visibility::Visible, Size::Match, 0.0)
    };
    static RADIAL_OVAL: Node = Node {
        kind: Kind::View,
        radius: -1.0,
        relative_rules: RADIAL_2STOP,
        ..button("rad_oval", Visibility::Visible, Size::Match, 0.0)
    };
    static RADIAL_3V: Node = Node {
        kind: Kind::View,
        relative_rules: RADIAL_3STOP,
        ..button("rad_3v", Visibility::Visible, Size::Match, 0.0)
    };
    static RADIAL_TINT_V: Node = Node {
        kind: Kind::View,
        relative_rules: RADIAL_TINTED,
        ..button("rad_tint_v", Visibility::Visible, Size::Match, 0.0)
    };
    static SWEEP_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: SWEEP_3STOP,
        ..button("swp_v", Visibility::Visible, Size::Match, 0.0)
    };
    static LINEAR_PIPE_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: LINEAR_PIPE,
        ..button("lin_pipe_v", Visibility::Visible, Size::Match, 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // 1. Radial 2-stop gradient in 400x400: capped at <= 256x256, center red, corner blue, rounded radius=16, cached across frames!
    let big = Rect { x: 0.0, y: 0.0, width: 400.0, height: 400.0 };
    let cmds1 = ui.frame(Input::default(), big, 0.0, |f| { RADIAL_VIEW.render(f, big, &mut state); });
    let (rad1, img1) = cmds1
        .iter()
        .find_map(|c| match c {
            DrawCommand::Image { radius, image, .. } => Some((*radius, image.clone())),
            _ => None,
        })
        .expect("radial gradient must emit DrawCommand::Image");
    assert!((rad1 - 16.0).abs() < 0.05, "radial gradient must clip to rounded radius 16, got {rad1}");
    assert!(img1.width() <= 256 && img1.height() <= 256, "gradient raster must be capped at 256x256");
    let px_at = |img: &Image, x: u32, y: u32| -> [u8; 4] {
        let i = ((y * img.width() + x) * 4) as usize;
        [img.pixels()[i], img.pixels()[i + 1], img.pixels()[i + 2], img.pixels()[i + 3]]
    };
    let center_px = px_at(&img1, img1.width() / 2, img1.height() / 2);
    let corner_px = px_at(&img1, 0, 0);
    assert!(center_px[0] > 240 && center_px[2] < 15, "radial center must be red, got {center_px:?}");
    assert!(corner_px[2] > 240 && corner_px[0] < 15, "radial outer edge must be blue, got {corner_px:?}");

    // Steady-state frame reuses identical cached Image ID (zero allocations)
    let cmds2 = ui.frame(Input::default(), big, 0.0, |f| { RADIAL_VIEW.render(f, big, &mut state); });
    let img2 = cmds2
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .unwrap();
    assert_eq!(img1.id(), img2.id(), "steady-state radial gradient frame must hit cache with same Image ID");

    // 2. Oval radial gradient masks corners to transparent alpha while keeping center opaque
    let sq = Rect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 };
    let cmds_oval = ui.frame(Input::default(), sq, 0.0, |f| { RADIAL_OVAL.render(f, sq, &mut state); });
    let img_oval = cmds_oval
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .unwrap();
    assert_eq!(px_at(&img_oval, 0, 0)[3], 0, "oval radial gradient corner must be masked transparent");
    assert_eq!(px_at(&img_oval, 50, 50)[3], 255, "oval radial gradient center must remain opaque");

    // 3. Radial 3-stop with explicit radius_dp=50 samples green at mid-radius, and SRC_IN background_tint modulates color & alpha
    let cmds_3 = ui.frame(Input::default(), sq, 0.0, |f| { RADIAL_3V.render(f, sq, &mut state); });
    let img_3 = cmds_3
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .unwrap();
    let mid_px = px_at(&img_3, 75, 50); // dist = 25 out of radius 50 -> t = 0.5 -> green center stop
    assert!(mid_px[1] > 220 && mid_px[0] < 30 && mid_px[2] < 30 && mid_px[3] == 255, "3-stop midpoint must sample green, got {mid_px:?}");

    let cmds_t = ui.frame(Input::default(), sq, 0.0, |f| { RADIAL_TINT_V.render(f, sq, &mut state); });
    let img_t = cmds_t
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .unwrap();
    let tint_center = px_at(&img_t, 50, 50);
    let tint_corner = px_at(&img_t, 0, 0);
    assert_eq!(tint_center[0..3], [220, 40, 60], "SRC_IN background_tint must tint radial gradient RGB, got {tint_center:?}");
    assert!((tint_center[3] as i32 - 128).abs() <= 2, "SRC_IN background_tint must modulate radial gradient alpha, got {tint_center:?}");
    assert_eq!(tint_corner[3], 0, "SRC_IN tint must keep 0-alpha gradient stops transparent");

    // 4. Sweep gradient: angle 0 = +X/East increasing clockwise (South=0.25, West=0.5 green, North=0.75, East-top~1.0 blue)
    let cmds_sw1 = ui.frame(Input::default(), sq, 0.0, |f| { SWEEP_VIEW.render(f, sq, &mut state); });
    let img_sw1 = cmds_sw1
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .expect("sweep gradient must emit DrawCommand::Image");
    let east_cw = px_at(&img_sw1, 90, 51);  // just past 0 deg CW -> start (red)
    let west = px_at(&img_sw1, 10, 50);     // 180 deg -> center (green)
    let east_ccw = px_at(&img_sw1, 90, 49); // just before 360 deg -> end (blue)
    assert!(east_cw[0] > 230 && east_cw[2] < 20, "sweep at +X/+eps must be start red, got {east_cw:?}");
    assert!(west[1] > 230 && west[0] < 20 && west[2] < 20, "sweep at -X (180deg) must be center green, got {west:?}");
    assert!(east_ccw[2] > 230 && east_ccw[0] < 20, "sweep at +X/-eps (~360deg) must be end blue, got {east_ccw:?}");

    let cmds_sw2 = ui.frame(Input::default(), sq, 0.0, |f| { SWEEP_VIEW.render(f, sq, &mut state); });
    let img_sw2 = cmds_sw2
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .unwrap();
    assert_eq!(img_sw1.id(), img_sw2.id(), "steady-state sweep gradient must hit cache");

    // 5. Pipe-separated linear format backwards-compat extension
    let cmds_lp = ui.frame(Input::default(), sq, 0.0, |f| { LINEAR_PIPE_VIEW.render(f, sq, &mut state); });
    let rects: Vec<Color> = cmds_lp
        .iter()
        .filter_map(|c| match c { DrawCommand::RoundedRect { color, .. } => Some(*color), _ => None })
        .collect();
    assert!(rects.len() >= 16, "pipe linear gradient should emit stepped strips");
    assert!(rects.first().unwrap().0 > 220 && rects.last().unwrap().2 > 220);
}

#[test]
fn wave17a_cut_corners_shape_appearance_fill_outline_and_widgets() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_BUTTON_VARIANT, RULE_CHIP_STYLE, RULE_CORNER_FAMILY, RULE_CORNER_RADII,
        RULE_GRADIENT,
    };

    static ALL_CUT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CORNER_RADII, anchor: "16,16,16,16" },
        RelativeRule { kind: RULE_CORNER_FAMILY, anchor: "c,c,c,c" },
    ];
    static MIXED_CUT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CORNER_RADII, anchor: "20,20,20,20" },
        RelativeRule { kind: RULE_CORNER_FAMILY, anchor: "c,r,r,c" },
    ];
    static CUT_VIEW: Node = Node {
        kind: Kind::View,
        background: Some(Color(40, 120, 220, 255)),
        background_stroke: Some(Color(10, 20, 30, 255)),
        background_stroke_width: 2.0,
        relative_rules: ALL_CUT_RULES,
        ..button("cut_v", Visibility::Visible, Size::Match, 0.0)
    };
    static MIXED_VIEW: Node = Node {
        kind: Kind::View,
        background: Some(Color(80, 180, 90, 255)),
        background_stroke: Some(Color(20, 40, 20, 255)),
        background_stroke_width: 2.0,
        relative_rules: MIXED_CUT_RULES,
        ..button("mix_v", Visibility::Visible, Size::Match, 0.0)
    };
    static CUT_BTN_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "outlined" },
        RelativeRule { kind: RULE_CORNER_RADII, anchor: "12,12,12,12" },
        RelativeRule { kind: RULE_CORNER_FAMILY, anchor: "c,c,c,c" },
    ];
    static CUT_BTN: Node = Node {
        kind: Kind::Button,
        text: "Cut Action",
        relative_rules: CUT_BTN_RULES,
        ..button("cut_btn", Visibility::Visible, Size::Match, 0.0)
    };
    static CUT_CHIP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CHIP_STYLE, anchor: "filter|0" },
        RelativeRule { kind: RULE_CORNER_FAMILY, anchor: "c,c,c,c" },
    ];
    static CUT_CHIP: Node = Node {
        kind: Kind::Button,
        text: "Chamfer",
        checked: true,
        relative_rules: CUT_CHIP_RULES,
        ..button("cut_chip", Visibility::Visible, Size::Match, 0.0)
    };
    static CUT_CARD_RULES: &[RelativeRule] = &[
        RelativeRule { kind: 57, anchor: "Outlined" },
        RelativeRule { kind: 66, anchor: "16" },
        RelativeRule { kind: RULE_CORNER_FAMILY, anchor: "c,c,c,c" },
    ];
    static CUT_CARD_CHILD: Node = Node {
        kind: Kind::Text,
        text: "Inside Cut Card",
        ..button("cut_card_txt", Visibility::Visible, Size::Match, 0.0)
    };
    static CUT_CARD: Node = Node {
        kind: Kind::MaterialCard,
        relative_rules: CUT_CARD_RULES,
        children: std::slice::from_ref(&CUT_CARD_CHILD),
        ..button("cut_card", Visibility::Visible, Size::Match, 0.0)
    };
    static CUT_GRAD_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GRADIENT, anchor: "0|255,0,0,255||0,0,255,255" },
        RelativeRule { kind: RULE_CORNER_RADII, anchor: "20,20,20,20" },
        RelativeRule { kind: RULE_CORNER_FAMILY, anchor: "c,c,c,c" },
    ];
    static CUT_GRAD_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: CUT_GRAD_RULES,
        ..button("cut_grad", Visibility::Visible, Size::Match, 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    let b = Rect { x: 0.0, y: 0.0, width: 100.0, height: 60.0 };
    let px_at = |img: &Image, x: u32, y: u32| -> [u8; 4] {
        let i = ((y * img.width() + x) * 4) as usize;
        [img.pixels()[i], img.pixels()[i + 1], img.pixels()[i + 2], img.pixels()[i + 3]]
    };

    // 1. General view with all-cut corners: cached Image fill with transparent corners + 8 Line outline segments
    let cmds = ui.frame(Input::default(), b, 0.0, |f| { CUT_VIEW.render(f, b, &mut state); });
    let fill_img = cmds
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .expect("cut-corner fill must emit cached DrawCommand::Image");
    assert_eq!(px_at(&fill_img, 0, 0)[3], 0, "top-left cut corner must be transparent");
    assert_eq!(px_at(&fill_img, 99, 59)[3], 0, "bottom-right cut corner must be transparent");
    assert_eq!(px_at(&fill_img, 50, 30), [40, 120, 220, 255], "interior must have exact fill color");
    let lines = cmds
        .iter()
        .filter(|c| matches!(c, DrawCommand::Line { color: Color(10, 20, 30, 255), .. }))
        .count();
    assert_eq!(lines, 8, "all-cut octagon outline must emit 8 Line segments, got {lines}");
    assert!(!cmds.iter().any(|c| matches!(c, DrawCommand::Outline { .. })), "must not emit rounded Outline");

    // Second frame hits cache with identical Image ID
    let cmds_repeat = ui.frame(Input::default(), b, 0.0, |f| { CUT_VIEW.render(f, b, &mut state); });
    let fill_img2 = cmds_repeat
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .unwrap();
    assert_eq!(fill_img.id(), fill_img2.id());

    // 2. Mixed cut + rounded corners ("c,r,r,c", radii=20): TL cut transparent at (8,8), TR rounded opaque at (92,8)
    let cmds_mix = ui.frame(Input::default(), b, 0.0, |f| { MIXED_VIEW.render(f, b, &mut state); });
    let mix_img = cmds_mix
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .unwrap();
    assert_eq!(px_at(&mix_img, 8, 8)[3], 0, "TL cut corner must exclude (8,8) where 8+8 < 20");
    assert_eq!(px_at(&mix_img, 92, 8)[3], 255, "TR rounded corner must include (92,8) inside r=20 arc");
    let mix_lines = cmds_mix.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count();
    let mix_arcs = cmds_mix.iter().filter(|c| matches!(c, DrawCommand::Arc { .. })).count();
    assert_eq!(mix_lines, 6, "mixed c,r,r,c must emit 4 edges + 2 cut chamfers = 6 lines, got {mix_lines}");
    assert_eq!(mix_arcs, 2, "mixed c,r,r,c must emit 2 rounded corner arcs, got {mix_arcs}");

    // 3. Outlined MaterialButton with cut corners emits chamfered Lines and no rounded Outline
    let btn_b = Rect { x: 0.0, y: 0.0, width: 140.0, height: 48.0 };
    let cmds_btn = ui.frame(Input::default(), btn_b, 0.0, |f| { CUT_BTN.render(f, btn_b, &mut state); });
    assert!(
        cmds_btn.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count() >= 6,
        "outlined MaterialButton with cut corners must stroke chamfered lines"
    );
    assert!(!cmds_btn.iter().any(|c| matches!(c, DrawCommand::Outline { .. })));

    // 4. Selected Filter Chip with default 8dp radius + cut corners renders chamfered fill image
    let chip_b = Rect { x: 0.0, y: 0.0, width: 120.0, height: 40.0 };
    let cmds_chip = ui.frame(Input::default(), chip_b, 0.0, |f| { CUT_CHIP.render(f, chip_b, &mut state); });
    let chip_img = cmds_chip
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .expect("cut-corner Chip must emit chamfered Image fill");
    assert_eq!(px_at(&chip_img, 0, 0)[3], 0, "cut Chip corner must be transparent");

    // 5. Outlined MaterialCard with cut corners emits chamfered fill, chamfered outline Lines, and renders child text
    let card_b = Rect { x: 0.0, y: 0.0, width: 180.0, height: 100.0 };
    let cmds_card = ui.frame(Input::default(), card_b, 0.0, |f| { CUT_CARD.render(f, card_b, &mut state); });
    let card_img = cmds_card
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .expect("cut-corner MaterialCard must emit chamfered Image fill");
    assert_eq!(px_at(&card_img, 0, 0)[3], 0);
    assert_eq!(cmds_card.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count(), 8);
    assert!(!cmds_card.iter().any(|c| matches!(c, DrawCommand::Outline { .. })));
    assert!(cmds_card.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Inside Cut Card")));

    // 6. Gradient combined with cut corners clips gradient pixels to chamfered bevel
    let cmds_cg = ui.frame(Input::default(), b, 0.0, |f| { CUT_GRAD_VIEW.render(f, b, &mut state); });
    let cg_img = cmds_cg
        .iter()
        .find_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.clone()), _ => None })
        .unwrap();
    assert_eq!(px_at(&cg_img, 0, 0)[3], 0, "gradient on cut-corner view must clip corners to transparent");
    assert_eq!(px_at(&cg_img, 50, 30)[3], 255);
}

#[test]
fn wave17a_multi_segment_nine_patch_stretch_bands() {
    use bhoo::android_xml::{RULE_BACKGROUND_IMAGE, RULE_IMAGE_NINE_PATCH};

    // 20x16 source image:
    // X stretch bands: [2,6) len 4 and [12,18) len 6 (fixed: [0,2) len 2, [6,12) len 6, [18,20) len 2 => total fixed = 10)
    // Y stretch bands: [2,5) len 3 and [9,14) len 5 (fixed: [0,2) len 2, [5,9) len 4, [14,16) len 2 => total fixed = 8)
    static MULTI_BG_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BACKGROUND_IMAGE, anchor: "multi_9p|2,6,12,18;2,5,9,14" },
    ];
    static MULTI_BG_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: MULTI_BG_RULES,
        ..button("m9p_bg", Visibility::Visible, Size::Match, 0.0)
    };
    static MULTI_IMG_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_IMAGE_NINE_PATCH, anchor: "2,6,12,18;2,5,9,14" },
    ];
    static MULTI_IMG_VIEW: Node = Node {
        kind: Kind::Image,
        image_src: "multi_9p",
        relative_rules: MULTI_IMG_RULES,
        ..button("m9p_img", Visibility::Visible, Size::Match, 0.0)
    };
    static INVALID_OVERLAP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_IMAGE_NINE_PATCH, anchor: "2,8,6,10;2,5" },
    ];
    static INVALID_OOB_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_IMAGE_NINE_PATCH, anchor: "2,25;2,5" },
    ];
    static INVALID_OVERLAP_VIEW: Node = Node {
        kind: Kind::Image,
        image_src: "multi_9p",
        relative_rules: INVALID_OVERLAP_RULES,
        ..button("m9p_bad1", Visibility::Visible, Size::Match, 0.0)
    };
    static INVALID_OOB_VIEW: Node = Node {
        kind: Kind::Image,
        image_src: "multi_9p",
        relative_rules: INVALID_OOB_RULES,
        ..button("m9p_bad2", Visibility::Visible, Size::Match, 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.images.insert("multi_9p".into(), Image::rgba(20, 16, vec![255; 20 * 16 * 4]).unwrap());
    state.image_scales.insert("multi_9p".into(), 1.0);

    // 1. Background 2-band x 2-band into 100x64 at density 1.0 -> 5 x 5 = 25 tiles
    let dst = Rect { x: 0.0, y: 0.0, width: 100.0, height: 64.0 };
    let cmds = ui.frame(Input::default(), dst, 0.0, |f| {
        MULTI_BG_VIEW.render(f, dst, &mut state);
    });
    let tiles: Vec<(Rect, [f32; 4])> = cmds
        .iter()
        .filter_map(|c| match c { DrawCommand::Image { bounds, uv, .. } => Some((*bounds, *uv)), _ => None })
        .collect();
    assert_eq!(tiles.len(), 25, "2x2 stretch bands must emit 5x5=25 tiles, got {}", tiles.len());

    // Row 0 X widths: fixed 2, stretch1 = 90*(4/10) = 36, mid fixed 6, stretch2 = 90*(6/10) = 54, right fixed 2
    let row0_widths: Vec<f32> = tiles[0..5].iter().map(|(r, _)| r.width).collect();
    for (got, exp) in row0_widths.iter().zip([2.0, 36.0, 6.0, 54.0, 2.0]) {
        assert!((got - exp).abs() < 1e-4, "X segment width mismatch: got {row0_widths:?}, expected [2, 36, 6, 54, 2]");
    }
    // Col 0 Y heights: fixed 2, stretch1 = 56*(3/8) = 21, mid fixed 4, stretch2 = 56*(5/8) = 35, bot fixed 2
    let col0_heights: Vec<f32> = (0..5).map(|row| tiles[row * 5].0.height).collect();
    for (got, exp) in col0_heights.iter().zip([2.0, 21.0, 4.0, 35.0, 2.0]) {
        assert!((got - exp).abs() < 1e-4, "Y segment height mismatch: got {col0_heights:?}, expected [2, 21, 4, 35, 2]");
    }
    // Middle fixed tile (row 2, col 2): dst (38, 23, 6, 4), uv [6/20, 5/16, 12/20, 9/16]
    let mid = tiles[2 * 5 + 2];
    assert!((mid.0.x - 38.0).abs() < 1e-4 && (mid.0.y - 23.0).abs() < 1e-4);
    assert!((mid.0.width - 6.0).abs() < 1e-4 && (mid.0.height - 4.0).abs() < 1e-4);
    assert!((mid.1[0] - 0.3).abs() < 1e-5 && (mid.1[1] - 0.3125).abs() < 1e-5);
    assert!((mid.1[2] - 0.6).abs() < 1e-5 && (mid.1[3] - 0.5625).abs() < 1e-5);

    // 2. ImageView with RULE_IMAGE_NINE_PATCH (252) at density 2.0: fixed segments double, remaining splits proportionally
    state.density = 2.0;
    let cmds_img = ui.frame(Input::default(), dst, 0.0, |f| {
        MULTI_IMG_VIEW.render(f, dst, &mut state);
    });
    let tiles_img: Vec<Rect> = cmds_img
        .iter()
        .filter_map(|c| match c { DrawCommand::Image { bounds, .. } => Some(*bounds), _ => None })
        .collect();
    assert_eq!(tiles_img.len(), 25);
    // Fixed X at 2x = 4, 12, 4 (sum 20), remaining 80 -> stretch1 = 32, stretch2 = 48
    let row0_w2: Vec<f32> = tiles_img[0..5].iter().map(|r| r.width).collect();
    for (got, exp) in row0_w2.iter().zip([4.0, 32.0, 12.0, 48.0, 4.0]) {
        assert!((got - exp).abs() < 1e-4, "2x density X widths mismatch: got {row0_w2:?}");
    }

    // 3. Overlapping bands fail parse -> falls back to plain image (1 command); out-of-bounds bands safely emit 0 tiles
    state.density = 1.0;
    let cmds_ov = ui.frame(Input::default(), dst, 0.0, |f| { INVALID_OVERLAP_VIEW.render(f, dst, &mut state); });
    assert_eq!(cmds_ov.iter().filter(|c| matches!(c, DrawCommand::Image { .. })).count(), 1);
    let cmds_oob = ui.frame(Input::default(), dst, 0.0, |f| { INVALID_OOB_VIEW.render(f, dst, &mut state); });
    assert_eq!(cmds_oob.iter().filter(|c| matches!(c, DrawCommand::Image { .. })).count(), 0);
}

#[test]
fn wave17a_app_bar_layout_lift_on_scroll() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_APP_BAR_LIFT_ON_SCROLL, RULE_COORDINATOR_SCROLL_BEHAVIOR,
    };

    static TB_IN_APPBAR: Node = Node {
        kind: Kind::Toolbar,
        text: "Inbox",
        height: Size::Dp(64.0),
        ..button("lift_tb", Visibility::Visible, Size::Match, 0.0)
    };
    static APPBAR_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_APP_BAR_LIFT_ON_SCROLL, anchor: "1" },
    ];
    static SCROLL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COORDINATOR_SCROLL_BEHAVIOR, anchor: "" },
    ];
    static SCROLL_BODY: Node = Node {
        kind: Kind::View,
        height: Size::Dp(1200.0),
        ..button("lift_body", Visibility::Visible, Size::Match, 0.0)
    };
    static COORD_CHILDREN: &[Node] = &[
        Node {
            kind: Kind::Linear,
            height: Size::Dp(64.0),
            relative_rules: APPBAR_RULES,
            children: std::slice::from_ref(&TB_IN_APPBAR),
            ..button("lift_appbar", Visibility::Visible, Size::Match, 0.0)
        },
        Node {
            kind: Kind::Scroll,
            relative_rules: SCROLL_RULES,
            children: std::slice::from_ref(&SCROLL_BODY),
            ..button("lift_scroll", Visibility::Visible, Size::Match, 0.0)
        },
    ];
    static COORD: Node = Node {
        kind: Kind::Frame,
        children: COORD_CHILDREN,
        ..button("lift_coord", Visibility::Visible, Size::Match, 0.0)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    let b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 640.0 };

    // 1. At scroll=0: rests flat at 0dp (no Shadow, no M3 surface_high tonal tint)
    let cmds0 = ui.frame(Input::default(), b, 0.0, |f| { COORD.render(f, b, &mut state); });
    assert!(
        !cmds0.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })),
        "AppBarLayout at scroll=0 must rest flat with zero shadows"
    );
    assert!(
        !cmds0.iter().any(|c| matches!(
            c,
            DrawCommand::RoundedRect { bounds, color: Color(0xEC, 0xE6, 0xF0, 255), .. }
                if (bounds.height - 64.0).abs() < 0.5
        )),
        "AppBarLayout at scroll=0 must not show lifted surface_high tint"
    );

    // 2. Scrolled sibling (offset = 40dp > 0.5): auto-detects scroll -> emits M3 level 2 shadow + surface_high tonal tint!
    state.offsets.insert("lift_scroll".into(), 40.0);
    let cmds1 = ui.frame(Input::default(), b, 0.0, |f| { COORD.render(f, b, &mut state); });
    let shadows = cmds1
        .iter()
        .filter(|c| matches!(c, DrawCommand::Shadow { bounds, .. } if (bounds.height - 64.0).abs() < 0.5))
        .count();
    assert_eq!(shadows, 3, "lifted AppBarLayout must emit M3 level 2 (3-layer) shadow, got {shadows}");
    assert!(
        cmds1.iter().any(|c| matches!(
            c,
            DrawCommand::RoundedRect { bounds, color: Color(0xEC, 0xE6, 0xF0, 255), .. }
                if (bounds.height - 64.0).abs() < 0.5
        )),
        "lifted AppBarLayout must paint M3 surface_high tonal surface"
    );
    // Child Toolbar inside lifted AppBarLayout must not overwrite surface_high with flat window surface
    assert!(
        !cmds1.iter().any(|c| matches!(
            c,
            DrawCommand::RoundedRect { bounds, color: Color(0xFE, 0xF7, 0xFF, 255), .. }
                if (bounds.height - 64.0).abs() < 0.5
        )),
        "child Toolbar should not cover lifted AppBarLayout tonal surface"
    );

    // 3. Explicit target ID ("1|custom_feed") on a Toolbar with opaque custom background -> subtle 8% primary surface-tint overlay + level 2 shadow
    static TARGET_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_APP_BAR_LIFT_ON_SCROLL, anchor: "1|custom_feed" },
    ];
    static TARGET_TB: Node = Node {
        kind: Kind::Toolbar,
        text: "Custom",
        height: Size::Dp(64.0),
        background: Some(Color(240, 240, 245, 255)),
        relative_rules: TARGET_RULES,
        ..button("target_tb", Visibility::Visible, Size::Match, 0.0)
    };
    let tb_b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 64.0 };
    let t0 = ui.frame(Input::default(), tb_b, 0.0, |f| { TARGET_TB.render(f, tb_b, &mut state); });
    assert!(!t0.iter().any(|c| matches!(c, DrawCommand::Shadow { .. })));

    state.offsets.insert("custom_feed".into(), 24.0);
    let t1 = ui.frame(Input::default(), tb_b, 0.0, |f| { TARGET_TB.render(f, tb_b, &mut state); });
    assert_eq!(t1.iter().filter(|c| matches!(c, DrawCommand::Shadow { .. })).count(), 3);
    // M3Light primary is Color(0x67, 0x50, 0xA4, 255); 8% opacity -> alpha ~= 20
    assert!(
        t1.iter().any(|c| matches!(
            c,
            DrawCommand::RoundedRect { color: Color(0x67, 0x50, 0xA4, a), .. } if *a >= 18 && *a <= 22
        )),
        "custom-background lifted Toolbar must apply subtle 8% primary surface-tint overlay"
    );

    // 4. Smooth timed animation when time advances (t=1.0 -> 1.075 -> 1.25)
    let mut ui_anim = Ui::default();
    let mut st_anim = LayoutState::default();
    st_anim.theme_style = AndroidTheme::Material3Light;
    ui_anim.frame(Input { time: 1.0, ..Input::default() }, tb_b, 0.0, |f| {
        TARGET_TB.render(f, tb_b, &mut st_anim);
    });
    st_anim.offsets.insert("custom_feed".into(), 30.0);
    ui_anim.frame(Input { time: 1.0, ..Input::default() }, tb_b, 0.0, |f| {
        TARGET_TB.render(f, tb_b, &mut st_anim);
    });
    assert!(ui_anim.needs_redraw(), "starting timed lift transition must request redraw");
    let mid = ui_anim.frame(Input { time: 1.075, ..Input::default() }, tb_b, 0.0, |f| {
        TARGET_TB.render(f, tb_b, &mut st_anim);
    });
    assert!(ui_anim.needs_redraw(), "mid-transition lift must continue animating");
    let mid_alpha = mid
        .iter()
        .find_map(|c| match c {
            DrawCommand::RoundedRect { color: Color(0x67, 0x50, 0xA4, a), .. } => Some(*a),
            _ => None,
        })
        .expect("mid-transition primary tint overlay missing");
    assert!(mid_alpha > 0 && mid_alpha < 20, "mid-transition tint should smoothly interpolate, got alpha={mid_alpha}");
    let done = ui_anim.frame(Input { time: 1.25, ..Input::default() }, tb_b, 0.0, |f| {
        TARGET_TB.render(f, tb_b, &mut st_anim);
    });
    assert!(
        done.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0x67, 0x50, 0xA4, a), .. } if *a >= 19 && *a <= 21)),
        "settled lift should reach full 8% primary tint"
    );
}

#[test]
fn wave18a_barrier_margin_allows_gone_pipe_format_rtl_and_density() {
    use bhoo::android_xml::{
        RULE_BARRIER, RULE_CONSTRAINT_BARRIER, RULE_CONSTRAINT_BOTTOM_TO_TOP,
        RULE_CONSTRAINT_END_TO_START, RULE_CONSTRAINT_START_TO_END, RULE_CONSTRAINT_START_TO_START,
        RULE_CONSTRAINT_TOP_TO_BOTTOM, RULE_CONSTRAINT_TOP_TO_TOP,
    };

    assert_eq!(RULE_CONSTRAINT_BARRIER, RULE_BARRIER);

    static ANCHOR_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static A_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "anchor" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static B_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "anchor" },
    ];
    static BAR_ALLOW_GONE: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_BARRIER, anchor: "end|w18_a,w18_b|12|1" },
    ];
    static BAR_NO_GONE: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_BARRIER, anchor: "end|w18_a,w18_b|12|0" },
    ];
    static BAR_START_MARGIN: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_BARRIER, anchor: "start|w18_a|10|0" },
    ];
    static BAR_BOTTOM_MARGIN: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_BARRIER, anchor: "bottom|w18_a,w18_b|8|0" },
    ];
    static BAR_TOP_MARGIN: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_BARRIER, anchor: "top|w18_b|6|0" },
    ];
    static AFTER_ALLOW: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "bar_allow" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static AFTER_NOGONE: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "bar_nogone" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static BEFORE_START: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "bar_start" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static BELOW_BOT: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "bar_bot" },
    ];
    static ABOVE_TOP: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_TOP, anchor: "bar_top" },
    ];

    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            // anchor: (0, 0, 180, 40)
            Node { width: Size::Dp(180.), relative_rules: ANCHOR_RULES, ..button("anchor", Visibility::Visible, Size::Dp(40.), 0.) },
            // w18_a: (180, 0, 50, 30) when visible; (180, 0, 0, 0) when Gone
            Node { width: Size::Dp(50.), relative_rules: A_RULES, ..button("w18_a", Visibility::Visible, Size::Dp(30.), 0.) },
            // w18_b: (0, 40, 80, 30)
            Node { width: Size::Dp(80.), relative_rules: B_RULES, ..button("w18_b", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: BAR_ALLOW_GONE, ..button("bar_allow", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: BAR_NO_GONE, ..button("bar_nogone", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: BAR_START_MARGIN, ..button("bar_start", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: BAR_BOTTOM_MARGIN, ..button("bar_bot", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: BAR_TOP_MARGIN, ..button("bar_top", Visibility::Gone, Size::Wrap, 0.) },
            Node { width: Size::Dp(40.), relative_rules: AFTER_ALLOW, ..button("after_allow", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: AFTER_NOGONE, ..button("after_nogone", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: BEFORE_START, ..button("before_start", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: BELOW_BOT, ..button("below_bot", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(40.), relative_rules: ABOVE_TOP, ..button("above_top", Visibility::Visible, Size::Dp(20.), 0.) },
        ],
        ..button("w18_bar_root", Visibility::Visible, Size::Match, 0.)
    };

    let bounds = Rect { x: 0., y: 0., width: 500., height: 200. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let rect_of = |ui: &Ui, label: &str| {
        ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds
    };

    // 1. Both visible: end barrier max(230, 80) + 12 = 242; start barrier 180 - 10 = 170 -> before_start x = 130;
    //    bottom barrier max(30, 70) + 8 = 78; top barrier 40 - 6 = 34 -> above_top y = 14.
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert!((rect_of(&ui, "after_allow").x - 242.0).abs() < 0.05);
    assert!((rect_of(&ui, "after_nogone").x - 242.0).abs() < 0.05);
    assert!((rect_of(&ui, "before_start").x - 130.0).abs() < 0.05);
    assert!((rect_of(&ui, "below_bot").y - 78.0).abs() < 0.05);
    assert!((rect_of(&ui, "above_top").y - 14.0).abs() < 0.05);

    // 2. Make w18_a Gone:
    //    - allows_gone=1 includes w18_a's collapsed position 180 -> max(180, 80) + 12 = 192
    //    - allows_gone=0 excludes w18_a -> only w18_b's end 80 -> 80 + 12 = 92!
    state.visibility.insert("w18_a".into(), Visibility::Gone);
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    assert!((rect_of(&ui, "after_allow").x - 192.0).abs() < 0.05, "allows_gone=1 must include gone widget's collapsed position");
    assert!((rect_of(&ui, "after_nogone").x - 92.0).abs() < 0.05, "allows_gone=0 must exclude gone widget");

    // 3. Density 2.0 scales margin_dp by state.dp()
    state.density = 2.0;
    let big = Rect { x: 0., y: 0., width: 1000., height: 400. };
    ui.frame(Input::default(), big, 0., |f| { ROOT.render(f, big, &mut state); });
    // w18_b end = 160px, margin = 12*2 = 24px -> 184px
    assert!((rect_of(&ui, "after_nogone").x - 184.0).abs() < 0.05);

    // 4. RTL: start barrier shifts RIGHT (+margin), end barrier shifts LEFT (-margin)
    state.density = 1.0;
    state.rtl = true;
    state.visibility.clear();
    ui.frame(Input::default(), bounds, 0., |f| { ROOT.render(f, bounds, &mut state); });
    // In RTL (width=500): anchor at [320..500], w18_a at [270..320], w18_b at [420..500].
    // end barrier (min x) = 270 - 12 = 258 -> after_nogone (start_toEndOf=bar_nogone, width 40) sits at [218..258]
    // start barrier on w18_a (max x) = 320 + 10 = 330 -> before_start (end_toStartOf=bar_start, width 40) sits at [330..370]
    let an_rtl = rect_of(&ui, "after_nogone");
    let bs_rtl = rect_of(&ui, "before_start");
    assert!((an_rtl.x + an_rtl.width - 258.0).abs() < 0.05, "RTL end barrier must subtract margin leftward, got {an_rtl:?}");
    assert!((bs_rtl.x - 330.0).abs() < 0.05, "RTL start barrier must add margin rightward, got {bs_rtl:?}");
}

#[test]
fn wave18a_constraint_size_bounds_constrained_wrap_and_match_constraint_minmax() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, RULE_CONSTRAINT_BOTTOM_TO_TOP,
        RULE_CONSTRAINT_END_TO_END, RULE_CONSTRAINT_END_TO_START, RULE_CONSTRAINT_SIZE_BOUNDS,
        RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP, RULE_GUIDE_BEGIN,
        RULE_GUIDE_ORIENTATION,
    };

    static GUIDE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "vertical" },
        RelativeRule { kind: RULE_GUIDE_BEGIN, anchor: "140" },
    ];
    static UNCONSTRAINED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "g140" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static CONSTRAINED_W_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "g140" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_SIZE_BOUNDS, anchor: "1,0|0,0,0,0" },
    ];
    static CONSTRAINED_W_MINMAX_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "g140" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_SIZE_BOUNDS, anchor: "1,0|60,100,0,0" },
    ];
    static CONSTRAINED_H_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_SIZE_BOUNDS, anchor: "0,1|0,0,30,65" },
    ];
    static MATCH_CONSTRAINT_BOUNDS_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_SIZE_BOUNDS, anchor: "0,0|80,150,50,90" },
    ];

    static LONG_TEXT: &str = "This is a deliberately very long text line that would overflow 140dp without layout_constrainedWidth";

    static ROOT_WRAP: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: GUIDE_RULES, ..button("g140", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::Text, text: LONG_TEXT, width: Size::Wrap, clickable: true, relative_rules: UNCONSTRAINED_RULES, ..button("uncon", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Text, text: LONG_TEXT, width: Size::Wrap, clickable: true, relative_rules: CONSTRAINED_W_RULES, ..button("con_w", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Text, text: "Hi", width: Size::Wrap, clickable: true, relative_rules: CONSTRAINED_W_MINMAX_RULES, ..button("con_min", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Text, text: LONG_TEXT, width: Size::Wrap, clickable: true, relative_rules: CONSTRAINED_W_MINMAX_RULES, ..button("con_max", Visibility::Visible, Size::Wrap, 0.) },
        ],
        ..button("sb_wrap_root", Visibility::Visible, Size::Match, 0.)
    };

    static ROOT_CH: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::Text, text: LONG_TEXT, width: Size::Dp(60.), clickable: true, relative_rules: CONSTRAINED_H_RULES, ..button("con_h", Visibility::Visible, Size::Wrap, 0.) },
        ],
        ..button("sb_ch_root", Visibility::Visible, Size::Match, 0.)
    };

    static ROOT_0DP: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(0.), height: Size::Dp(0.), relative_rules: MATCH_CONSTRAINT_BOUNDS_RULES, ..button("mc_box", Visibility::Visible, Size::Dp(0.), 0.) },
        ],
        ..button("sb_0dp_root", Visibility::Visible, Size::Match, 0.)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // 1. Constrained wrap width vs unconstrained overflow past 140dp end anchor, plus [wmin, wmax]
    let wide = Rect { x: 0., y: 0., width: 320., height: 240. };
    ui.frame(Input::default(), wide, 0., |f| { ROOT_WRAP.render(f, wide, &mut state); });
    let sems = ui.semantics();
    let uncon_r = sems[0].bounds;
    let con_w_r = sems[1].bounds;
    let con_min_r = sems[2].bounds;
    let con_max_r = sems[3].bounds;
    assert!(uncon_r.width > 140.0, "unconstrained wrap Text should overflow past 140dp end anchor, got {}", uncon_r.width);
    assert!((con_w_r.width - 140.0).abs() < 0.05, "cw=1 wrap Text must clamp to available span 140dp, got {}", con_w_r.width);
    assert!(con_w_r.height > uncon_r.height, "clamped wrap Text must wrap onto multiple lines and grow taller");
    assert!((con_min_r.width - 60.0).abs() < 0.05, "wmin=60 must expand short wrap text to 60dp, got {}", con_min_r.width);
    assert!((con_max_r.width - 100.0).abs() < 0.05, "wmax=100 must cap long wrap text to 100dp, got {}", con_max_r.width);

    // 2. Constrained wrap height (ch=1) with [30, 65]
    let tall = Rect { x: 0., y: 0., width: 200., height: 120. };
    ui.frame(Input::default(), tall, 0., |f| { ROOT_CH.render(f, tall, &mut state); });
    let ch_r = ui.semantics()[0].bounds;
    assert!((ch_r.height - 65.0).abs() < 0.05, "ch=1 with hmax=65 must cap tall wrap child to 65dp, got {}", ch_r.height);
    let short_box = Rect { x: 0., y: 0., width: 200., height: 48. };
    ui.frame(Input::default(), short_box, 0., |f| { ROOT_CH.render(f, short_box, &mut state); });
    assert!((ui.semantics()[0].bounds.height - 48.0).abs() < 0.05, "ch=1 must clamp to smaller span 48dp");

    // 3. 0dp match-constraint child respects [wmin, wmax] and [hmin, hmax]
    let big = Rect { x: 0., y: 0., width: 300., height: 300. };
    ui.frame(Input::default(), big, 0., |f| { ROOT_0DP.render(f, big, &mut state); });
    let mc_big = ui.semantics()[0].bounds;
    assert_eq!(mc_big, Rect { x: 75.0, y: 105.0, width: 150.0, height: 90.0 });

    static G_TINY_L: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "vertical" },
        RelativeRule { kind: RULE_GUIDE_BEGIN, anchor: "40" },
    ];
    static G_TINY_R: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "vertical" },
        RelativeRule { kind: RULE_GUIDE_BEGIN, anchor: "100" },
    ];
    static G_TINY_T: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "horizontal" },
        RelativeRule { kind: RULE_GUIDE_BEGIN, anchor: "40" },
    ];
    static G_TINY_B: &[RelativeRule] = &[
        RelativeRule { kind: RULE_GUIDE_ORIENTATION, anchor: "horizontal" },
        RelativeRule { kind: RULE_GUIDE_BEGIN, anchor: "70" },
    ];
    static MC_TINY_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "gt_l" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "gt_r" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "gt_t" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_TOP, anchor: "gt_b" },
        RelativeRule { kind: RULE_CONSTRAINT_SIZE_BOUNDS, anchor: "0,0|80,150,50,90" },
    ];
    static ROOT_0DP_MIN: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: G_TINY_L, ..button("gt_l", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: G_TINY_R, ..button("gt_r", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: G_TINY_T, ..button("gt_t", Visibility::Gone, Size::Wrap, 0.) },
            Node { kind: Kind::View, visibility: Visibility::Gone, relative_rules: G_TINY_B, ..button("gt_b", Visibility::Gone, Size::Wrap, 0.) },
            Node { width: Size::Dp(0.), height: Size::Dp(0.), relative_rules: MC_TINY_RULES, ..button("mc_box_min", Visibility::Visible, Size::Dp(0.), 0.) },
        ],
        ..button("sb_0dp_min_root", Visibility::Visible, Size::Match, 0.)
    };
    let canvas = Rect { x: 0., y: 0., width: 200., height: 200. };
    ui.frame(Input::default(), canvas, 0., |f| { ROOT_0DP_MIN.render(f, canvas, &mut state); });
    let mc_tiny = ui.semantics()[0].bounds;
    assert_eq!(mc_tiny, Rect { x: 30.0, y: 30.0, width: 80.0, height: 50.0 });
}

#[test]
fn wave18a_flow_extended_chain_styles_bias_alignment_max_wrap_and_rtl() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_END_TO_END, RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP,
        RULE_FLOW,
    };

    let rect_of = |ui: &Ui, label: &str| {
        ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds
    };

    // 1. Horizontal compact/extended 5-seg format: spread + valign=end + max_wrap=2
    static SPREAD_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "h|chain|0,10|fa,fb,fc,fd|spread,,0.5,0.5,center,end,2" },
    ];
    static SPREAD_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Dp(0.), relative_rules: SPREAD_RULES, ..button("fl_sp", Visibility::Visible, Size::Wrap, 0.) },
            Node { width: Size::Dp(60.), text: "A", ..button("fa", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(60.), text: "B", ..button("fb", Visibility::Visible, Size::Dp(50.), 0.) },
            Node { width: Size::Dp(60.), text: "C", ..button("fc", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(60.), text: "D", ..button("fd", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_fl_sp", Visibility::Visible, Size::Match, 0.)
    };

    let b300 = Rect { x: 0., y: 0., width: 300., height: 240. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), b300, 0., |f| { SPREAD_ROOT.render(f, b300, &mut state); });
    // max_wrap=2 breaks into 2 rows of 2 items; spread of two 60dp items in 300dp -> gap=(300-120)/3=60 -> x=60, x=180!
    // valign=end on row 0 (row_h=50): A (h=20) aligns to bottom y=30, B (h=50) at y=0!
    // Row 1 starts at y = 50 + 10 = 60.
    assert!((rect_of(&ui, "A").x - 60.0).abs() < 0.05);
    assert!((rect_of(&ui, "B").x - 180.0).abs() < 0.05);
    assert!((rect_of(&ui, "A").y - 30.0).abs() < 0.05, "valign=end must bottom-align shorter item A");
    assert!((rect_of(&ui, "B").y - 0.0).abs() < 0.05);
    assert!((rect_of(&ui, "C").x - 60.0).abs() < 0.05 && (rect_of(&ui, "C").y - 60.0).abs() < 0.05);
    assert!((rect_of(&ui, "D").x - 180.0).abs() < 0.05 && (rect_of(&ui, "D").y - 60.0).abs() < 0.05);

    // 2. spread_inside + valign=start
    static SI_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "h|none|0,0|fa,fb,fc|spread_inside,,0.5,0.5,center,start,0" },
    ];
    static SI_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Dp(0.), relative_rules: SI_RULES, ..button("fl_si", Visibility::Visible, Size::Wrap, 0.) },
            Node { width: Size::Dp(40.), text: "A", ..button("fa", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(40.), text: "B", ..button("fb", Visibility::Visible, Size::Dp(50.), 0.) },
            Node { width: Size::Dp(40.), text: "C", ..button("fc", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_fl_si", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), b300, 0., |f| { SI_ROOT.render(f, b300, &mut state); });
    // 3 items of 40dp in 300dp: spread_inside -> x = 0, 130, 260; valign=start -> all y = 0!
    assert!((rect_of(&ui, "A").x - 0.0).abs() < 0.05 && (rect_of(&ui, "A").y - 0.0).abs() < 0.05);
    assert!((rect_of(&ui, "B").x - 130.0).abs() < 0.05 && (rect_of(&ui, "B").y - 0.0).abs() < 0.05);
    assert!((rect_of(&ui, "C").x - 260.0).abs() < 0.05 && (rect_of(&ui, "C").y - 0.0).abs() < 0.05);

    // 3. packed + hbias=0.25 (and RTL packed bias=0.25)
    static PACKED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "h|none|10,0|fa,fb|packed,,0.25,0.5,center,center,0" },
    ];
    static PACKED_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Dp(0.), relative_rules: PACKED_RULES, ..button("fl_pk", Visibility::Visible, Size::Wrap, 0.) },
            Node { width: Size::Dp(50.), text: "A", ..button("fa", Visibility::Visible, Size::Dp(20.), 0.) },
            Node { width: Size::Dp(50.), text: "B", ..button("fb", Visibility::Visible, Size::Dp(40.), 0.) },
        ],
        ..button("cl_fl_pk", Visibility::Visible, Size::Match, 0.)
    };
    let b310 = Rect { x: 0., y: 0., width: 310., height: 100. };
    ui.frame(Input::default(), b310, 0., |f| { PACKED_ROOT.render(f, b310, &mut state); });
    // used = 50 + 10 + 50 = 110, extra = 200 -> bias 0.25 starts at x = 50, B at x = 110; valign=center -> A.y = 10
    assert!((rect_of(&ui, "A").x - 50.0).abs() < 0.05 && (rect_of(&ui, "A").y - 10.0).abs() < 0.05);
    assert!((rect_of(&ui, "B").x - 110.0).abs() < 0.05);

    // RTL packed bias 0.25: A starts at right edge 310 - 50 = 260 -> A=[210..260], B=[150..200]
    state.rtl = true;
    ui.frame(Input::default(), b310, 0., |f| { PACKED_ROOT.render(f, b310, &mut state); });
    assert!((rect_of(&ui, "A").x - 210.0).abs() < 0.05);
    assert!((rect_of(&ui, "B").x - 150.0).abs() < 0.05);
    state.rtl = false;

    // 4. Vertical Flow: v|chain|12,0|fa,fb,fc|,spread_inside,0.5,0.5,end,center,2
    static VERT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "v|chain|12,0|fa,fb,fc|,spread_inside,0.5,0.5,end,center,2" },
    ];
    static VERT_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Dp(160.), relative_rules: VERT_RULES, ..button("fl_v", Visibility::Visible, Size::Dp(160.), 0.) },
            Node { width: Size::Dp(40.), text: "A", ..button("fa", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(80.), text: "B", ..button("fb", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(50.), text: "C", ..button("fc", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_fl_v", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), b300, 0., |f| { VERT_ROOT.render(f, b300, &mut state); });
    // Col 0 (max_wrap=2): A (w=40) and B (w=80) -> col_w=80, halign=end -> A.x = 40, B.x = 0;
    // vstyle=spread_inside in height 160 -> A.y = 0, B.y = 130!
    // Col 1 starts at x = 80 + 12 = 92; single item C centered at vbias=0.5 -> C.y = (160 - 30)*0.5 = 65!
    assert!((rect_of(&ui, "A").x - 40.0).abs() < 0.05 && (rect_of(&ui, "A").y - 0.0).abs() < 0.05);
    assert!((rect_of(&ui, "B").x - 0.0).abs() < 0.05 && (rect_of(&ui, "B").y - 130.0).abs() < 0.05);
    assert!((rect_of(&ui, "C").x - 92.0).abs() < 0.05 && (rect_of(&ui, "C").y - 65.0).abs() < 0.05);
}

#[test]
fn wave18a_constraint_layer_union_chrome_shared_transform_and_alpha() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_LAYER, RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP,
        RULE_ELEVATION, RULE_VIEW_TRANSFORM,
    };

    static A_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static B_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static OTHER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static LAYER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_LAYER, anchor: "la,lb,lg" },
        RelativeRule { kind: RULE_VIEW_TRANSFORM, anchor: "10,5,1.5,1.5,30,center,center" },
        RelativeRule { kind: RULE_ELEVATION, anchor: "4" },
    ];

    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            // Other (unreferenced): (0, 120, 60, 30)
            Node { width: Size::Dp(60.), margin: [0., 120., 0., 0.], text: "Other", relative_rules: OTHER_RULES, ..button("lother", Visibility::Visible, Size::Dp(30.), 0.) },
            // A: (20, 10, 60, 40)
            Node { width: Size::Dp(60.), margin: [20., 10., 0., 0.], text: "Alpha", relative_rules: A_RULES, ..button("la", Visibility::Visible, Size::Dp(40.), 0.) },
            // B: (100, 30, 80, 50)
            Node { width: Size::Dp(80.), margin: [100., 30., 0., 0.], text: "Beta", relative_rules: B_RULES, ..button("lb", Visibility::Visible, Size::Dp(50.), 0.) },
            // Gone view G: must not affect union rect
            Node { width: Size::Dp(400.), margin: [0., 0., 0., 0.], text: "Gone", relative_rules: A_RULES, ..button("lg", Visibility::Gone, Size::Dp(400.), 0.) },
            // Layer declared after its referenced siblings
            Node {
                kind: Kind::View,
                background: Some(Color(220, 235, 255, 255)),
                view_alpha: 0.6,
                relative_rules: LAYER_RULES,
                ..button("my_layer", Visibility::Visible, Size::Wrap, 0.)
            },
        ],
        ..button("cl_layer_root", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 320., height: 220. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let cmds = ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });

    // Union of A(20..80, 10..50) and B(100..180, 30..80) = (20, 10, 160, 70), center = [100, 45]!
    let push_tr_idx = cmds
        .iter()
        .position(|c| matches!(
            c,
            DrawCommand::PushTransform { origin, translate, scale, rotation_rad }
                if (origin[0] - 100.0).abs() < 0.05
                    && (origin[1] - 45.0).abs() < 0.05
                    && (translate[0] - 10.0).abs() < 0.05
                    && (translate[1] - 5.0).abs() < 0.05
                    && (scale[0] - 1.5).abs() < 0.01
                    && (scale[1] - 1.5).abs() < 0.01
                    && (*rotation_rad - 30.0_f32.to_radians()).abs() < 1e-4
        ))
        .expect("Layer must emit PushTransform around union center");
    let pop_tr_idx = cmds.iter().rposition(|c| matches!(c, DrawCommand::PopTransform)).unwrap();
    let push_layer_idx = cmds
        .iter()
        .position(|c| matches!(c, DrawCommand::PushLayer { alpha, .. } if (*alpha - 0.6).abs() < 0.01))
        .expect("Layer must emit PushLayer with view_alpha=0.6");
    let pop_layer_idx = cmds.iter().rposition(|c| matches!(c, DrawCommand::PopLayer)).unwrap();
    let shadow_idx = cmds
        .iter()
        .position(|c| matches!(c, DrawCommand::Shadow { bounds, .. } if *bounds == Rect { x: 20., y: 10., width: 160., height: 70. }))
        .expect("Layer elevation shadow must cover union rect");
    let bg_idx = cmds
        .iter()
        .position(|c| matches!(c, DrawCommand::RoundedRect { bounds, color: Color(220, 235, 255, 255), .. } if *bounds == Rect { x: 20., y: 10., width: 160., height: 70. }))
        .expect("Layer background must cover union rect");
    let other_txt_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Other")).unwrap();
    let alpha_txt_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Alpha")).unwrap();
    let beta_txt_idx = cmds.iter().position(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Beta")).unwrap();

    // Verify nesting & draw order: Other outside; then PushTransform -> PushLayer -> Shadow -> Bg -> Alpha -> Beta -> PopLayer -> PopTransform
    assert!(other_txt_idx < push_tr_idx);
    assert!(push_tr_idx < push_layer_idx);
    assert!(push_layer_idx < shadow_idx);
    assert!(shadow_idx < bg_idx);
    assert!(bg_idx < alpha_txt_idx && alpha_txt_idx < beta_txt_idx);
    assert!(beta_txt_idx < pop_layer_idx && pop_layer_idx < pop_tr_idx);

    // Runtime state overrides on Layer ID combine with static transform/alpha
    state.view_alpha.insert("my_layer".into(), 0.5); // 0.6 * 0.5 = 0.3
    state.view_translation.insert("my_layer".into(), [4.0, -2.0]); // [14, 3]
    let cmds2 = ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });
    assert!(cmds2.iter().any(|c| matches!(c, DrawCommand::PushLayer { alpha, .. } if (*alpha - 0.3).abs() < 0.01)));
    assert!(cmds2.iter().any(|c| matches!(c, DrawCommand::PushTransform { translate, .. } if (translate[0] - 14.0).abs() < 0.05 && (translate[1] - 3.0).abs() < 0.05)));
}

#[test]
fn wave18a_constraint_placeholder_static_runtime_override_and_empty_visibility() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_PLACEHOLDER, RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_BOTTOM,
        RULE_CONSTRAINT_TOP_TO_TOP,
    };

    static A_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static B_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static PH_GONE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_PLACEHOLDER, anchor: "btn_a|gone" },
    ];
    static BELOW_PH_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "ph" },
    ];

    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(80.), text: "BtnA", relative_rules: A_RULES, ..button("btn_a", Visibility::Visible, Size::Dp(36.), 0.) },
            Node { width: Size::Dp(90.), margin: [0., 80., 0., 0.], text: "BtnB", relative_rules: B_RULES, ..button("btn_b", Visibility::Visible, Size::Dp(44.), 0.) },
            // Placeholder at (180, 20), Size::Wrap copies bound target's dimensions
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Wrap, margin: [180., 20., 0., 0.], relative_rules: PH_GONE_RULES, ..button("ph", Visibility::Visible, Size::Wrap, 0.) },
            Node { width: Size::Dp(70.), text: "Below", relative_rules: BELOW_PH_RULES, ..button("below_ph", Visibility::Visible, Size::Dp(24.), 0.) },
        ],
        ..button("cl_ph_root", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 360., height: 220. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    let rect_of = |ui: &Ui, label: &str| {
        ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds
    };

    // 1. Static content_id="btn_a": BtnA renders ONLY at Placeholder's bounds (180, 20, 80, 36), NOT at (0, 0)!
    ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });
    assert_eq!(ui.semantics().iter().filter(|n| n.label.as_ref() == "BtnA").count(), 1);
    assert_eq!(rect_of(&ui, "BtnA"), Rect { x: 180., y: 20., width: 80., height: 36. });
    assert_eq!(rect_of(&ui, "BtnB"), Rect { x: 0., y: 80., width: 90., height: 44. });
    assert!((rect_of(&ui, "Below").y - 56.0).abs() < 0.05);

    // Clicking BtnA at its Placeholder position triggers action Some("btn_a")
    ui.frame(Input { pointer: Some([200., 35.]), pressed: true, ..Input::default() }, b, 0., |f| { ROOT.render(f, b, &mut state); });
    let mut act = None;
    ui.frame(Input { pointer: Some([200., 35.]), released: true, ..Input::default() }, b, 0., |f| { act = ROOT.render(f, b, &mut state); });
    assert_eq!(act, Some("btn_a"));

    // 2. Runtime override: state.placeholder_content["ph"] = "btn_b" moves BtnB to Placeholder, restores BtnA to (0, 0)
    state.placeholder_content.insert("ph".into(), "btn_b".into());
    ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });
    assert_eq!(rect_of(&ui, "BtnA"), Rect { x: 0., y: 0., width: 80., height: 36. });
    assert_eq!(rect_of(&ui, "BtnB"), Rect { x: 180., y: 20., width: 90., height: 44. });
    assert!((rect_of(&ui, "Below").y - 64.0).abs() < 0.05);

    // 3. Clearing placeholder_content to "" triggers empty_vis=gone: Below collapses to y=0, both buttons at original positions
    state.placeholder_content.insert("ph".into(), "".into());
    ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });
    assert_eq!(rect_of(&ui, "BtnA"), Rect { x: 0., y: 0., width: 80., height: 36. });
    assert_eq!(rect_of(&ui, "BtnB"), Rect { x: 0., y: 80., width: 90., height: 44. });
    assert!((rect_of(&ui, "Below").y - 0.0).abs() < 0.05);

    // 4. empty_vis=invisible keeps fixed Placeholder size (100x50) while omitting Placeholder visuals;
    //    empty_vis=visible draws Placeholder's own background
    static PH_INV_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_PLACEHOLDER, anchor: "|invisible" },
    ];
    static PH_VIS_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_PLACEHOLDER, anchor: "|visible" },
    ];
    static ROOT_EMPTY_MODES: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Dp(100.), height: Size::Dp(50.), background: Some(Color(200, 100, 100, 255)), relative_rules: PH_INV_RULES, ..button("ph", Visibility::Visible, Size::Dp(50.), 0.) },
            Node { width: Size::Dp(70.), text: "BelowInv", relative_rules: BELOW_PH_RULES, ..button("below_inv", Visibility::Visible, Size::Dp(24.), 0.) },
            Node { kind: Kind::View, width: Size::Dp(80.), height: Size::Dp(40.), margin: [150., 0., 0., 0.], background: Some(Color(100, 200, 100, 255)), relative_rules: PH_VIS_RULES, ..button("ph_vis", Visibility::Visible, Size::Dp(40.), 0.) },
        ],
        ..button("cl_ph_empty", Visibility::Visible, Size::Match, 0.)
    };
    state.placeholder_content.clear();
    let cmds_e = ui.frame(Input::default(), b, 0., |f| { ROOT_EMPTY_MODES.render(f, b, &mut state); });
    assert!((rect_of(&ui, "BelowInv").y - 50.0).abs() < 0.05, "empty_vis=invisible must preserve Placeholder layout bounds");
    assert!(!cmds_e.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(200, 100, 100, 255), .. })));
    assert!(cmds_e.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(100, 200, 100, 255), .. })));
}

#[test]
fn wave18a_drawer_layout_closed_edge_swipe_and_dual_edge_independence() {
    use bhoo::android_xml::{AndroidTheme, RULE_DRAWER_LAYOUT, RULE_NAV_BAR, RULE_NAV_ITEM};

    static DL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_DRAWER_LAYOUT, anchor: "" }];
    static START_NAV: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "drawer|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "st_home|StartHome|home" },
    ];
    static DUAL_DRAWER: Node = Node {
        kind: Kind::Frame,
        relative_rules: DL_RULES,
        children: &[
            Node {
                text: "MainBtn",
                width: Size::Match,
                height: Size::Dp(48.),
                ..button("dual_main", Visibility::Visible, Size::Dp(48.), 0.)
            },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(240.),
                height: Size::Match,
                layout_gravity: [0, 0], // start
                layout_gravity_explicit: true,
                relative_rules: START_NAV,
                ..button("start_panel", Visibility::Visible, Size::Match, 0.)
            },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(220.),
                height: Size::Match,
                layout_gravity: [2, 0], // end
                layout_gravity_explicit: true,
                children: &[Node {
                    kind: Kind::Text,
                    text: "EndFilter",
                    ..button("end_filter_txt", Visibility::Visible, Size::Dp(40.), 0.)
                }],
                ..button("end_panel", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("dual_dl", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 360., height: 300. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    let has_text = |cmds: &[DrawCommand], needle: &str| {
        cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == needle))
    };

    // 1. Dual-edge programmatic independence: "dual_dl" opens ONLY start; "dual_dl:end" opens ONLY end
    state.drawer_open.insert("dual_dl".into(), true);
    let c_st = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    assert!(has_text(&c_st, "StartHome") && !has_text(&c_st, "EndFilter"));

    state.drawer_open.insert("dual_dl".into(), false);
    state.drawer_open.insert("dual_dl:end".into(), true);
    let c_end = ui.frame(Input { time: 1.5, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    // After 0.3s settle, start closes and end opens fully
    let c_end2 = ui.frame(Input { time: 1.85, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    assert!(has_text(&c_end, "EndFilter") || has_text(&c_end2, "EndFilter"));
    assert!(!has_text(&c_end2, "StartHome") && has_text(&c_end2, "EndFilter"));

    // Reset clean closed state
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });

    // 2. Drag starting outside 24dp edge (x=80 -> 200) does NOT open start drawer
    ui.frame(Input { pointer: Some([80., 100.]), pressed: true, time: 2.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([200., 100.]), time: 2.1, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([200., 100.]), released: true, time: 2.2, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    assert_ne!(state.drawer_open.get("dual_dl"), Some(&true));

    // 3. Closed-edge short swipe from left edge (x=10 -> 60: 50/240 = 0.208 < 0.30, slow):
    //    tracks interactively while dragging, cancels MainBtn tap, settles back closed!
    ui.frame(Input { pointer: Some([10., 24.]), pressed: true, time: 3.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    let c_drag_short = ui.frame(Input { pointer: Some([60., 24.]), time: 3.3, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    assert!(has_text(&c_drag_short, "StartHome"), "Start drawer must track interactively during closed-edge swipe");
    let mut main_act = None;
    ui.frame(Input { pointer: Some([60., 24.]), released: true, time: 3.6, ..Input::default() }, b, 0., |f| { main_act = DUAL_DRAWER.render(f, b, &mut state); });
    assert_eq!(main_act, None, "Edge swipe must cancel main content button click");
    assert_ne!(state.drawer_open.get("dual_dl"), Some(&true));
    let c_after_short = ui.frame(Input { time: 4.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    assert!(!has_text(&c_after_short, "StartHome"), "Short edge swipe must animate back closed");

    // 4. Closed-edge swipe past 30% from left edge (x=8 -> 108: 100/240 = 0.417 > 0.30): settles open!
    ui.frame(Input { pointer: Some([8., 100.]), pressed: true, time: 5.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([108., 100.]), time: 5.2, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([108., 100.]), released: true, time: 5.4, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    assert_eq!(state.drawer_open.get("dual_dl"), Some(&true));
    assert_ne!(state.drawer_open.get("dual_dl:end"), Some(&true));

    // Close start drawer via Escape
    ui.frame(Input { key: Some(KeyInput { key: Key::Escape, pressed: true, shift: false, repeat: false }), time: 5.8, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    ui.frame(Input { time: 6.2, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    assert_eq!(state.drawer_open.get("dual_dl"), Some(&false));

    // 5. Fast inward flick from right edge (x=352 -> 312 in 0.04s = 1000dp/s > 500dp/s, even with progress 40/220 < 0.30):
    //    opens ONLY end drawer ("dual_dl:end")!
    ui.frame(Input { pointer: Some([352., 120.]), pressed: true, time: 7.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([312., 120.]), time: 7.04, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([312., 120.]), released: true, time: 7.05, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut state); });
    assert_eq!(state.drawer_open.get("dual_dl:end"), Some(&true));
    assert_eq!(state.drawer_open.get("dual_dl"), Some(&false));

    // 6. RTL: swiping inward from RIGHT edge opens start drawer ("dual_dl"), from LEFT edge opens end drawer ("dual_dl:end")
    let mut ui_rtl = Ui::default();
    let mut st_rtl = LayoutState::default();
    st_rtl.theme_style = AndroidTheme::Material3Light;
    st_rtl.rtl = true;
    ui_rtl.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut st_rtl); });
    ui_rtl.frame(Input { pointer: Some([350., 100.]), pressed: true, time: 2.0, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut st_rtl); });
    ui_rtl.frame(Input { pointer: Some([240., 100.]), time: 2.2, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut st_rtl); });
    ui_rtl.frame(Input { pointer: Some([240., 100.]), released: true, time: 2.3, ..Input::default() }, b, 0., |f| { DUAL_DRAWER.render(f, b, &mut st_rtl); });
    assert_eq!(st_rtl.drawer_open.get("dual_dl"), Some(&true), "RTL right-edge inward swipe must open start drawer");
    assert_ne!(st_rtl.drawer_open.get("dual_dl:end"), Some(&true));
}

#[test]
fn wave18a_app_bar_scroll_flags_scroll_exit_until_collapsed_and_enter_always() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_APP_BAR_SCROLL_FLAGS, RULE_COORDINATOR_SCROLL_BEHAVIOR,
    };

    static SCROLL_ONLY: &[RelativeRule] = &[RelativeRule { kind: RULE_APP_BAR_SCROLL_FLAGS, anchor: "scroll" }];
    static EXIT_COLLAPSED: &[RelativeRule] = &[RelativeRule { kind: RULE_APP_BAR_SCROLL_FLAGS, anchor: "scroll,exitUntilCollapsed" }];
    static ENTER_ALWAYS: &[RelativeRule] = &[RelativeRule { kind: RULE_APP_BAR_SCROLL_FLAGS, anchor: "scroll,enterAlways" }];
    static ENTER_COLLAPSED: &[RelativeRule] = &[RelativeRule { kind: RULE_APP_BAR_SCROLL_FLAGS, anchor: "scroll,enterAlways,enterAlwaysCollapsed" }];
    static COORD_SCROLL: &[RelativeRule] = &[RelativeRule { kind: RULE_COORDINATOR_SCROLL_BEHAVIOR, anchor: "" }];

    static BODY: Node = Node {
        kind: Kind::View,
        height: Size::Dp(1200.),
        ..button("ab_body", Visibility::Visible, Size::Match, 0.)
    };
    static TABS_PINNED: Node = Node {
        kind: Kind::View,
        height: Size::Dp(48.),
        background: Some(Color(10, 120, 200, 255)),
        ..button("ab_tabs", Visibility::Visible, Size::Match, 0.)
    };

    // 1. scroll only (64dp toolbar + 48dp pinned tabs)
    static AB_SCROLL_CHILDREN: &[Node] = &[
        Node { kind: Kind::View, height: Size::Dp(64.), background: Some(Color(220, 60, 60, 255)), relative_rules: SCROLL_ONLY, ..button("ab_tb1", Visibility::Visible, Size::Match, 0.) },
        TABS_PINNED,
    ];
    static COORD_SCROLL_ONLY: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, height: Size::Wrap, children: AB_SCROLL_CHILDREN, ..button("ab1", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Scroll, relative_rules: COORD_SCROLL, children: std::slice::from_ref(&BODY), ..button("sc1", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("coord1", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 360., height: 640. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    let rect_by_color = |cmds: &[DrawCommand], color: Color| -> Option<Rect> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color: c0, .. } if *c0 == color => Some(*bounds),
            _ => None,
        })
    };

    let red = Color(220, 60, 60, 255);
    let blue = Color(10, 120, 200, 255);

    let c0 = ui.frame(Input::default(), b, 0., |f| { COORD_SCROLL_ONLY.render(f, b, &mut state); });
    assert_eq!(rect_by_color(&c0, red).unwrap().y, 0.0);
    assert_eq!(rect_by_color(&c0, blue).unwrap().y, 64.0);

    state.offsets.insert("sc1".into(), 30.0);
    let c30 = ui.frame(Input::default(), b, 0., |f| { COORD_SCROLL_ONLY.render(f, b, &mut state); });
    assert!((rect_by_color(&c30, red).unwrap().y - (-30.0)).abs() < 0.05);
    assert!((rect_by_color(&c30, blue).unwrap().y - 34.0).abs() < 0.05);

    state.offsets.insert("sc1".into(), 100.0);
    let c100 = ui.frame(Input::default(), b, 0., |f| { COORD_SCROLL_ONLY.render(f, b, &mut state); });
    assert!(rect_by_color(&c100, red).is_none(), "fully collapsed scroll child must not draw");
    assert!((rect_by_color(&c100, blue).unwrap().y - 0.0).abs() < 0.05);

    // 2. scroll,exitUntilCollapsed (100dp header, min_height=40dp -> stays pinned at 40dp!)
    static AB_EXIT_CHILDREN: &[Node] = &[
        Node { kind: Kind::View, height: Size::Dp(100.), min_height: 40.0, background: Some(Color(220, 60, 60, 255)), relative_rules: EXIT_COLLAPSED, ..button("ab_tb2", Visibility::Visible, Size::Match, 0.) },
        TABS_PINNED,
    ];
    static COORD_EXIT: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, height: Size::Wrap, children: AB_EXIT_CHILDREN, ..button("ab2", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Scroll, relative_rules: COORD_SCROLL, children: std::slice::from_ref(&BODY), ..button("sc2", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("coord2", Visibility::Visible, Size::Match, 0.)
    };
    state.offsets.insert("sc2".into(), 200.0);
    let c_exit = ui.frame(Input::default(), b, 0., |f| { COORD_EXIT.render(f, b, &mut state); });
    assert!((rect_by_color(&c_exit, red).unwrap().y - (-60.0)).abs() < 0.05);
    assert!((rect_by_color(&c_exit, blue).unwrap().y - 40.0).abs() < 0.05, "exitUntilCollapsed must pin min_height=40 at top");

    // 3. scroll,enterAlways (64dp toolbar): scrolling back up from 200 -> 170 -> 120 immediately reveals bar!
    static AB_ENTER_CHILDREN: &[Node] = &[
        Node { kind: Kind::View, height: Size::Dp(64.), background: Some(Color(220, 60, 60, 255)), relative_rules: ENTER_ALWAYS, ..button("ab_tb3", Visibility::Visible, Size::Match, 0.) },
        TABS_PINNED,
    ];
    static COORD_ENTER: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, height: Size::Wrap, children: AB_ENTER_CHILDREN, ..button("ab3", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Scroll, relative_rules: COORD_SCROLL, children: std::slice::from_ref(&BODY), ..button("sc3", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("coord3", Visibility::Visible, Size::Match, 0.)
    };
    state.offsets.insert("sc3".into(), 0.0);
    let ce0 = ui.frame(Input::default(), b, 0., |f| { COORD_ENTER.render(f, b, &mut state); });
    assert!((rect_by_color(&ce0, blue).unwrap().y - 64.0).abs() < 0.05);

    state.offsets.insert("sc3".into(), 200.0);
    let ce200 = ui.frame(Input::default(), b, 0., |f| { COORD_ENTER.render(f, b, &mut state); });
    assert!(rect_by_color(&ce200, red).is_none());
    assert!((rect_by_color(&ce200, blue).unwrap().y - 0.0).abs() < 0.05);

    // Scroll back UP 30dp (200 -> 170): immediately reveals 30dp of toolbar!
    state.offsets.insert("sc3".into(), 170.0);
    let ce170 = ui.frame(Input::default(), b, 0., |f| { COORD_ENTER.render(f, b, &mut state); });
    assert!((rect_by_color(&ce170, red).unwrap().y - (-34.0)).abs() < 0.05);
    assert!((rect_by_color(&ce170, blue).unwrap().y - 30.0).abs() < 0.05, "enterAlways must reveal 30dp immediately on upward scroll");

    // Scroll back UP another 50dp (170 -> 120): fully revealed (64dp) even while scroll=120!
    state.offsets.insert("sc3".into(), 120.0);
    let ce120 = ui.frame(Input::default(), b, 0., |f| { COORD_ENTER.render(f, b, &mut state); });
    assert!((rect_by_color(&ce120, red).unwrap().y - 0.0).abs() < 0.05);
    assert!((rect_by_color(&ce120, blue).unwrap().y - 64.0).abs() < 0.05);

    // Scroll back DOWN 20dp (120 -> 140): collapses 20dp -> 44dp visible
    state.offsets.insert("sc3".into(), 140.0);
    let ce140 = ui.frame(Input::default(), b, 0., |f| { COORD_ENTER.render(f, b, &mut state); });
    assert!((rect_by_color(&ce140, blue).unwrap().y - 44.0).abs() < 0.05);

    // 4. scroll,enterAlways,enterAlwaysCollapsed (min_height=24dp): reveals only 24dp while far from top, then full 64dp at top
    static AB_EC_CHILDREN: &[Node] = &[
        Node { kind: Kind::View, height: Size::Dp(64.), min_height: 24.0, background: Some(Color(220, 60, 60, 255)), relative_rules: ENTER_COLLAPSED, ..button("ab_tb4", Visibility::Visible, Size::Match, 0.) },
        TABS_PINNED,
    ];
    static COORD_EC: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, height: Size::Wrap, children: AB_EC_CHILDREN, ..button("ab4", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Scroll, relative_rules: COORD_SCROLL, children: std::slice::from_ref(&BODY), ..button("sc4", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("coord4", Visibility::Visible, Size::Match, 0.)
    };
    state.offsets.insert("sc4".into(), 200.0);
    ui.frame(Input::default(), b, 0., |f| { COORD_EC.render(f, b, &mut state); });
    state.offsets.insert("sc4".into(), 150.0);
    let cec150 = ui.frame(Input::default(), b, 0., |f| { COORD_EC.render(f, b, &mut state); });
    assert!((rect_by_color(&cec150, blue).unwrap().y - 24.0).abs() < 0.05, "enterAlwaysCollapsed should cap mid-scroll reveal at min_height=24");
    state.offsets.insert("sc4".into(), 0.0);
    let cec0 = ui.frame(Input::default(), b, 0., |f| { COORD_EC.render(f, b, &mut state); });
    assert!((rect_by_color(&cec0, blue).unwrap().y - 64.0).abs() < 0.05);
}

#[test]
fn wave18a_animation_list_looping_oneshot_wake_at_background_and_reduced_motion() {
    use bhoo::android_xml::RULE_ANIMATION_LIST;

    static LOOP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ANIMATION_LIST, anchor: "0|af_0:100,af_1:200,af_2:100" },
    ];
    static ONESHOT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ANIMATION_LIST, anchor: "1|af_0:100,af_1:150" },
    ];
    static BG_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ANIMATION_LIST, anchor: "0|af_0:100,af_1:100" },
    ];

    static LOOP_IMG: Node = Node {
        kind: Kind::Image,
        relative_rules: LOOP_RULES,
        ..button("anim_img", Visibility::Visible, Size::Match, 0.)
    };
    static ONESHOT_ICON: Node = Node {
        kind: Kind::Icon,
        relative_rules: ONESHOT_RULES,
        ..button("anim_icon", Visibility::Visible, Size::Match, 0.)
    };
    static BG_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: BG_RULES,
        ..button("anim_bg", Visibility::Visible, Size::Match, 0.)
    };

    let img0 = Image::rgba(8, 8, vec![255, 0, 0, 255].repeat(64)).unwrap();
    let img1 = Image::rgba(8, 8, vec![0, 255, 0, 255].repeat(64)).unwrap();
    let img2 = Image::rgba(8, 8, vec![0, 0, 255, 255].repeat(64)).unwrap();
    let (id0, id1, id2) = (img0.id(), img1.id(), img2.id());

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.images.insert("af_0".into(), img0);
    state.images.insert("af_1".into(), img1);
    state.images.insert("af_2".into(), img2);

    let b = Rect { x: 0., y: 0., width: 64., height: 64. };
    let drawn_img_id = |cmds: &[DrawCommand]| -> Option<u32> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::Image { image, .. } => Some(image.id()),
            _ => None,
        })
    };

    // 1. Looping ImageView (total 400ms: 0..100=af_0, 100..300=af_1, 300..400=af_2, wraps at 400ms)
    let c0 = ui.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| { LOOP_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&c0), Some(id0));
    assert!((ui.next_wakeup().unwrap() - 0.1).abs() < 1e-6);

    let c1 = ui.frame(Input { time: 0.15, ..Input::default() }, b, 0., |f| { LOOP_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&c1), Some(id1));
    assert!((ui.next_wakeup().unwrap() - 0.3).abs() < 1e-6);

    let c2 = ui.frame(Input { time: 0.35, ..Input::default() }, b, 0., |f| { LOOP_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&c2), Some(id2));
    assert!((ui.next_wakeup().unwrap() - 0.4).abs() < 1e-6);

    let c_wrap = ui.frame(Input { time: 0.45, ..Input::default() }, b, 0., |f| { LOOP_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&c_wrap), Some(id0), "looping animation-list must wrap modulo total duration");
    assert!((ui.next_wakeup().unwrap() - 0.5).abs() < 1e-6);

    // 2. Oneshot ImageButton (Kind::Icon): stops on last frame with no wake_at
    let mut ui_os = Ui::default();
    let co0 = ui_os.frame(Input { time: 0.05, ..Input::default() }, b, 0., |f| { ONESHOT_ICON.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&co0), Some(id0));
    assert!((ui_os.next_wakeup().unwrap() - 0.1).abs() < 1e-6);

    let co1 = ui_os.frame(Input { time: 0.12, ..Input::default() }, b, 0., |f| { ONESHOT_ICON.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&co1), Some(id1));
    assert_eq!(ui_os.next_wakeup(), None, "oneshot on final frame must stop scheduling wakes");

    let co_done = ui_os.frame(Input { time: 1.5, ..Input::default() }, b, 0., |f| { ONESHOT_ICON.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&co_done), Some(id1), "oneshot past total duration must clamp to last frame");
    assert_eq!(ui_os.next_wakeup(), None);

    // 3. View background animation-list
    let mut ui_bg = Ui::default();
    let cb0 = ui_bg.frame(Input { time: 0.02, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&cb0), Some(id0));
    assert!((ui_bg.next_wakeup().unwrap() - 0.1).abs() < 1e-6);

    let cb1 = ui_bg.frame(Input { time: 0.12, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&cb1), Some(id1));
    assert!((ui_bg.next_wakeup().unwrap() - 0.2).abs() < 1e-6);

    // 4. Reduced motion freezes strictly on frame 0 and schedules no wakeup
    let mut ui_rm = Ui::default();
    ui_rm.motion_preference = MotionPreference::Reduced;
    let crm_img = ui_rm.frame(Input { time: 0.25, ..Input::default() }, b, 0., |f| { LOOP_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&crm_img), Some(id0), "reduced_motion must freeze ImageView on frame 0");
    assert_eq!(ui_rm.next_wakeup(), None);

    let crm_bg = ui_rm.frame(Input { time: 0.35, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&crm_bg), Some(id0), "reduced_motion must freeze background animation-list on frame 0");
    assert_eq!(ui_rm.next_wakeup(), None);
}

#[test]
fn wave19a_dynamic_wrapper_drawables_level_clip_rotate_scale_and_bg_prefix() {
    use bhoo::android_xml::RULE_DRAWABLE_DYNAMIC;

    let img_lo = Image::rgba(16, 16, vec![255, 0, 0, 255].repeat(256)).unwrap();
    let img_mid = Image::rgba(16, 16, vec![0, 255, 0, 255].repeat(256)).unwrap();
    let img_hi = Image::rgba(16, 16, vec![0, 0, 255, 255].repeat(256)).unwrap();
    let img_bg = Image::rgba(16, 16, vec![255, 255, 0, 255].repeat(256)).unwrap();
    let (id_lo, id_mid, id_hi, id_bg) = (img_lo.id(), img_mid.id(), img_hi.id(), img_bg.id());

    let mut state = LayoutState::default();
    state.images.insert("lo".into(), img_lo);
    state.images.insert("mid".into(), img_mid);
    state.images.insert("hi".into(), img_hi);
    state.images.insert("bg_img".into(), img_bg);

    let b = Rect { x: 0., y: 0., width: 64., height: 64. };
    let drawn_ids = |cmds: &[DrawCommand]| -> Vec<u32> {
        cmds.iter()
            .filter_map(|c| match c {
                DrawCommand::Image { image, .. } => Some(image.id()),
                _ => None,
            })
            .collect()
    };

    // 1. level-list on ImageView (default src) + fallback to item 0 + bg: prefix on View & ImageView
    static LEVEL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "level|100:3000:lo,3001:7000:mid,7001:10000:hi" },
    ];
    static LEVEL_IMG: Node = Node {
        kind: Kind::Image,
        image_scale: 2, // fitXY
        relative_rules: LEVEL_RULES,
        ..button("lvl_img", Visibility::Visible, Size::Match, 0.)
    };
    let mut ui = Ui::default();
    // level 0 (not in any range) -> falls back to item 0 ("lo")
    let c_fb = ui.frame(Input::default(), b, 0., |f| { LEVEL_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&c_fb), vec![id_lo]);
    state.drawable_level.insert("lvl_img".into(), 5000);
    let c_mid = ui.frame(Input::default(), b, 0., |f| { LEVEL_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&c_mid), vec![id_mid]);
    state.drawable_level.insert("lvl_img".into(), 9500);
    let c_hi = ui.frame(Input::default(), b, 0., |f| { LEVEL_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&c_hi), vec![id_hi]);

    // Both bg: and src: on the same ImageView
    static DUAL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "bg:level|0:10000:bg_img" },
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "src:level|0:5000:lo,5001:10000:hi" },
    ];
    static DUAL_IMG: Node = Node {
        kind: Kind::Image,
        image_scale: 2,
        relative_rules: DUAL_RULES,
        ..button("dual_img", Visibility::Visible, Size::Match, 0.)
    };
    state.drawable_level.insert("dual_img".into(), 8000);
    let c_dual = ui.frame(Input::default(), b, 0., |f| { DUAL_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&c_dual), vec![id_bg, id_hi], "bg: must draw before src: on the same ImageView");

    // 2. clip (horizontal start/end with RTL + vertical bottom + center + level 0 & 10000)
    static CLIP_H_START_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "clip|h|start|lo" },
    ];
    static CLIP_V_BOT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "bg:clip|v|bottom|hi" },
    ];
    static CLIP_IMG: Node = Node {
        kind: Kind::Image,
        image_scale: 2,
        relative_rules: CLIP_H_START_RULES,
        ..button("clip_h", Visibility::Visible, Size::Match, 0.)
    };
    static CLIP_BG_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: CLIP_V_BOT_RULES,
        ..button("clip_v", Visibility::Visible, Size::Match, 0.)
    };
    // Level 0 -> nothing drawn
    state.drawable_level.insert("clip_h".into(), 0);
    let c_cl0 = ui.frame(Input::default(), b, 0., |f| { CLIP_IMG.render(f, b, &mut state); });
    assert!(drawn_ids(&c_cl0).is_empty(), "clip at level 0 must draw nothing");
    // Level 5000 LTR -> left half [0..32]
    state.drawable_level.insert("clip_h".into(), 5000);
    state.rtl = false;
    let c_cl_ltr = ui.frame(Input::default(), b, 0., |f| { CLIP_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&c_cl_ltr), vec![id_lo]);
    assert!(c_cl_ltr.iter().any(|c| matches!(c, DrawCommand::PushClip(r) if (r.x - 0.0).abs() < 0.05 && (r.width - 32.0).abs() < 0.05)));
    assert_eq!(
        c_cl_ltr.iter().filter(|c| matches!(c, DrawCommand::PushClip(_))).count(),
        c_cl_ltr.iter().filter(|c| matches!(c, DrawCommand::PopClip)).count()
    );
    // Level 5000 RTL -> right half [32..64]
    state.rtl = true;
    let c_cl_rtl = ui.frame(Input::default(), b, 0., |f| { CLIP_IMG.render(f, b, &mut state); });
    assert!(c_cl_rtl.iter().any(|c| matches!(c, DrawCommand::PushClip(r) if (r.x - 32.0).abs() < 0.05 && (r.width - 32.0).abs() < 0.05)));
    state.rtl = false;
    // Vertical bottom clip at 25% -> y = 48..64
    state.drawable_level.insert("clip_v".into(), 2500);
    let c_cl_v = ui.frame(Input::default(), b, 0., |f| { CLIP_BG_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&c_cl_v), vec![id_hi]);
    assert!(c_cl_v.iter().any(|c| matches!(c, DrawCommand::PushClip(r) if (r.y - 48.0).abs() < 0.05 && (r.height - 16.0).abs() < 0.05)));

    // 3. rotate (from_deg,to_deg | px,py | img_key)
    static ROT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "rotate|-90,270|0.25,0.75|mid" },
    ];
    static ROT_ICON: Node = Node {
        kind: Kind::Icon,
        image_scale: 2,
        relative_rules: ROT_RULES,
        ..button("rot_icon", Visibility::Visible, Size::Match, 0.)
    };
    state.drawable_level.insert("rot_icon".into(), 5000); // -90 + 360*0.5 = +90 deg around [16, 48]
    let c_rot = ui.frame(Input::default(), b, 0., |f| { ROT_ICON.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&c_rot), vec![id_mid]);
    assert!(c_rot.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, rotation_rad, .. }
            if (origin[0] - 16.0).abs() < 0.05
                && (origin[1] - 48.0).abs() < 0.05
                && (*rotation_rad - 90.0_f32.to_radians()).abs() < 1e-4
    )));
    assert_eq!(
        c_rot.iter().filter(|c| matches!(c, DrawCommand::PushTransform { .. })).count(),
        c_rot.iter().filter(|c| matches!(c, DrawCommand::PopTransform)).count()
    );

    // 4. scale (sw_frac,sh_frac | gravity | img_key)
    static SCALE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "scale|0.5,1.0|start|lo" },
    ];
    static SCALE_IMG: Node = Node {
        kind: Kind::Image,
        image_scale: 2,
        relative_rules: SCALE_RULES,
        ..button("scl_img", Visibility::Visible, Size::Match, 0.)
    };
    // Level 0: sy = 1.0 - 1.0 + 0.0 = 0.0 -> skipped
    state.drawable_level.insert("scl_img".into(), 0);
    let c_sc0 = ui.frame(Input::default(), b, 0., |f| { SCALE_IMG.render(f, b, &mut state); });
    assert!(drawn_ids(&c_sc0).is_empty());
    // Level 5000 LTR: sx = 0.75, sy = 0.5 around start pivot [0, 32]
    state.drawable_level.insert("scl_img".into(), 5000);
    let c_sc50 = ui.frame(Input::default(), b, 0., |f| { SCALE_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&c_sc50), vec![id_lo]);
    assert!(c_sc50.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, scale, .. }
            if (origin[0] - 0.0).abs() < 0.05
                && (origin[1] - 32.0).abs() < 0.05
                && (scale[0] - 0.75).abs() < 1e-4
                && (scale[1] - 0.5).abs() < 1e-4
    )));
    // Level 5000 RTL: start pivot mirrors to [64, 32]
    state.rtl = true;
    let c_sc50_rtl = ui.frame(Input::default(), b, 0., |f| { SCALE_IMG.render(f, b, &mut state); });
    assert!(c_sc50_rtl.iter().any(|c| matches!(
        c,
        DrawCommand::PushTransform { origin, scale, .. }
            if (origin[0] - 64.0).abs() < 0.05
                && (origin[1] - 32.0).abs() < 0.05
                && (scale[0] - 0.75).abs() < 1e-4
                && (scale[1] - 0.5).abs() < 1e-4
    )));
}

#[test]
fn wave19a_dynamic_drawable_transition_crossfade_and_reduced_motion() {
    use bhoo::android_xml::RULE_DRAWABLE_DYNAMIC;

    static TR_SRC_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "transition|200|tr_a,tr_b" },
    ];
    static TR_BG_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWABLE_DYNAMIC, anchor: "bg:transition|200|tr_a,tr_b" },
    ];
    static TR_IMG: Node = Node {
        kind: Kind::Image,
        image_scale: 2,
        relative_rules: TR_SRC_RULES,
        ..button("tr_img", Visibility::Visible, Size::Match, 0.)
    };
    static TR_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: TR_BG_RULES,
        ..button("tr_view", Visibility::Visible, Size::Match, 0.)
    };

    let img_a = Image::rgba(8, 8, vec![255, 0, 0, 255].repeat(64)).unwrap();
    let img_b = Image::rgba(8, 8, vec![0, 0, 255, 255].repeat(64)).unwrap();
    let (id_a, id_b) = (img_a.id(), img_b.id());

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.images.insert("tr_a".into(), img_a);
    state.images.insert("tr_b".into(), img_b);

    let b = Rect { x: 0., y: 0., width: 64., height: 64. };
    let img_ids = |cmds: &[DrawCommand]| -> Vec<u32> {
        cmds.iter().filter_map(|c| match c { DrawCommand::Image { image, .. } => Some(image.id()), _ => None }).collect()
    };
    let layer_alphas = |cmds: &[DrawCommand]| -> Vec<f32> {
        cmds.iter().filter_map(|c| match c { DrawCommand::PushLayer { alpha, .. } => Some(*alpha), _ => None }).collect()
    };

    // 1. Initial frame (false): only img_a, no PushLayer
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { TR_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&c0), vec![id_a]);
    assert!(layer_alphas(&c0).is_empty());
    assert!(!ui.needs_redraw());

    // 2. Toggle true at t=2.0 -> starts 200ms crossfade
    state.drawable_transition_active.insert("tr_img".into(), true);
    ui.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| { TR_IMG.render(f, b, &mut state); });
    assert!(ui.needs_redraw());

    // Midpoint at t=2.1 (t=0.5): PushLayer(0.5) around img_a + PushLayer(0.5) around img_b
    let c_mid = ui.frame(Input { time: 2.1, ..Input::default() }, b, 0., |f| { TR_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&c_mid), vec![id_a, id_b]);
    let alphas = layer_alphas(&c_mid);
    assert_eq!(alphas.len(), 2);
    assert!((alphas[0] - 0.5).abs() < 1e-4 && (alphas[1] - 0.5).abs() < 1e-4);
    assert_eq!(
        c_mid.iter().filter(|c| matches!(c, DrawCommand::PushLayer { .. })).count(),
        c_mid.iter().filter(|c| matches!(c, DrawCommand::PopLayer)).count()
    );
    assert!(ui.needs_redraw());

    // At t=2.15 (t=0.75): alpha 0.25 for img_a, 0.75 for img_b
    let c_75 = ui.frame(Input { time: 2.15, ..Input::default() }, b, 0., |f| { TR_IMG.render(f, b, &mut state); });
    let a75 = layer_alphas(&c_75);
    assert!((a75[0] - 0.25).abs() < 1e-4 && (a75[1] - 0.75).abs() < 1e-4);

    // After 200ms (t=2.25): settled on img_b, no PushLayer, no redraw
    let c_done = ui.frame(Input { time: 2.25, ..Input::default() }, b, 0., |f| { TR_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&c_done), vec![id_b]);
    assert!(layer_alphas(&c_done).is_empty());
    assert!(!ui.needs_redraw());

    // 3. Background transition + reduced motion instant snap
    let mut ui_rm = Ui::default();
    ui_rm.motion_preference = MotionPreference::Reduced;
    let cb0 = ui_rm.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { TR_VIEW.render(f, b, &mut state); });
    assert_eq!(img_ids(&cb0), vec![id_a]);
    state.drawable_transition_active.insert("tr_view".into(), true);
    let cb1 = ui_rm.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| { TR_VIEW.render(f, b, &mut state); });
    assert_eq!(img_ids(&cb1), vec![id_b], "reduced_motion must snap transition immediately");
    assert!(layer_alphas(&cb1).is_empty());
    assert!(!ui_rm.needs_redraw());
}

#[test]
fn wave19a_bottom_sheet_nested_scrolling_handoff_scroll_and_listview() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_BOTTOM_SHEET, SHEET_STATE_COLLAPSED, SHEET_STATE_EXPANDED,
        SHEET_STATE_HIDDEN,
    };

    static BS_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_BOTTOM_SHEET,
        anchor: "120|1|0|0.5|0",
    }];
    static TALL_CONTENT: Node = Node {
        kind: Kind::View,
        height: Size::Dp(1200.0),
        ..button("bs_tall_body", Visibility::Visible, Size::Match, 0.0)
    };
    static SCROLL_SHEET_ROOT: Node = Node {
        kind: Kind::Frame,
        children: &[Node {
            kind: Kind::Linear,
            height: Size::Match,
            relative_rules: BS_RULES,
            children: &[Node {
                kind: Kind::Scroll,
                height: Size::Dp(0.0),
                weight: 1.0,
                children: std::slice::from_ref(&TALL_CONTENT),
                ..button("bs_inner_scroll", Visibility::Visible, Size::Dp(0.0), 1.0)
            }],
            ..button("bs_nested", Visibility::Visible, Size::Match, 0.0)
        }],
        ..button("bs_nested_root", Visibility::Visible, Size::Match, 0.0)
    };

    static LIST_SHEET_ROOT: Node = Node {
        kind: Kind::Frame,
        children: &[Node {
            kind: Kind::Linear,
            height: Size::Match,
            relative_rules: BS_RULES,
            children: &[Node {
                kind: Kind::ListView,
                height: Size::Dp(0.0),
                weight: 1.0,
                ..button("bs_inner_list", Visibility::Visible, Size::Dp(0.0), 1.0)
            }],
            ..button("bs_list_sheet", Visibility::Visible, Size::Match, 0.0)
        }],
        ..button("bs_list_root", Visibility::Visible, Size::Match, 0.0)
    };

    let sheet_height = |cmds: &[DrawCommand]| -> Option<f32> {
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut active_clip: Option<Rect> = None;
        for c in cmds {
            match c {
                DrawCommand::PushClip(r) => active_clip = Some(*r),
                DrawCommand::PopClip => active_clip = None,
                DrawCommand::RoundedRect { bounds, color, .. } if *color == Color(0xF7, 0xF2, 0xFA, 255) => {
                    let vis = active_clip.map_or(*bounds, |cl| bounds.intersect(cl));
                    if vis.height > 0.0 {
                        min_y = min_y.min(vis.y);
                        max_y = max_y.max(vis.y + vis.height);
                    }
                }
                _ => {}
            }
        }
        (min_y < max_y).then_some(max_y - min_y)
    };

    let b = Rect { x: 0.0, y: 0.0, width: 360.0, height: 600.0 };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    state.sheet_state.insert("bs_nested".into(), SHEET_STATE_EXPANDED);

    // Initial expanded render
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_height(&c0).unwrap() - 600.0).abs() < 1.0);

    // 1. Dragging UPWARD inside Expanded sheet -> inner ScrollView consumes scroll, sheet stays Expanded!
    ui.frame(Input { pointer: Some([180.0, 400.0]), pointer_kind: PointerKind::Touch, pressed: true, time: 2.0, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    let c_up = ui.frame(Input { pointer: Some([180.0, 260.0]), pointer_kind: PointerKind::Touch, time: 2.1, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_height(&c_up).unwrap() - 600.0).abs() < 1.0, "Upward drag in expanded sheet must keep sheet expanded");
    assert!(state.offsets["bs_inner_scroll"] > 100.0, "Upward drag must scroll inner ScrollView");
    ui.frame(Input { pointer: Some([180.0, 260.0]), pointer_kind: PointerKind::Touch, released: true, time: 2.2, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    assert_eq!(state.sheet_state.get("bs_nested"), Some(&SHEET_STATE_EXPANDED));

    // 2. Dragging DOWNWARD while inner ScrollView is scrolled down (offset > 0.5) -> inner ScrollView consumes it!
    state.offsets.insert("bs_inner_scroll".into(), 120.0);
    ui.frame(Input { time: 3.0, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([180.0, 200.0]), pointer_kind: PointerKind::Touch, pressed: true, time: 3.1, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    let c_sc_down = ui.frame(Input { pointer: Some([180.0, 270.0]), pointer_kind: PointerKind::Touch, time: 3.2, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    assert!((sheet_height(&c_sc_down).unwrap() - 600.0).abs() < 1.0, "Downward drag while inner scroll > 0.5 must NOT collapse sheet");
    assert!(
        state.offsets["bs_inner_scroll"] < 70.0,
        "Downward drag must scroll inner ScrollView back toward top, got offset={}",
        state.offsets["bs_inner_scroll"]
    );
    ui.frame(Input { pointer: Some([180.0, 270.0]), pointer_kind: PointerKind::Touch, released: true, time: 3.3, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });

    // 3. Dragging DOWNWARD when inner ScrollView is at top (offset == 0.0) -> hands off to BottomSheet & collapses!
    state.offsets.insert("bs_inner_scroll".into(), 0.0);
    ui.frame(Input { time: 4.0, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([180.0, 120.0]), pointer_kind: PointerKind::Touch, pressed: true, time: 4.1, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    let c_ho = ui.frame(Input { pointer: Some([180.0, 320.0]), pointer_kind: PointerKind::Touch, time: 4.2, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    let h_ho = sheet_height(&c_ho).unwrap();
    assert!((h_ho - 400.0).abs() < 2.0, "Downward drag at top must hand off to BottomSheet, got height {h_ho}");
    assert_eq!(state.offsets["bs_inner_scroll"], 0.0, "Inner scroll must remain at 0 during sheet handoff");
    ui.frame(Input { pointer: Some([180.0, 580.0]), pointer_kind: PointerKind::Touch, released: true, time: 4.3, ..Input::default() }, b, 0.0, |f| {
        SCROLL_SHEET_ROOT.render(f, b, &mut state);
    });
    assert!(
        matches!(state.sheet_state.get("bs_nested"), Some(&SHEET_STATE_COLLAPSED) | Some(&SHEET_STATE_HIDDEN)),
        "Releasing downward drag at top must collapse/hide sheet, got {:?}",
        state.sheet_state.get("bs_nested")
    );

    // 4. Continuous single-gesture handoff on ListView inside Expanded BottomSheet:
    //    scroll from offset=25 -> 0, then continue dragging down to collapse sheet!
    let mut ui_lv = Ui::default();
    let mut st_lv = LayoutState::default();
    st_lv.theme_style = AndroidTheme::Material3Light;
    st_lv.sheet_state.insert("bs_list_sheet".into(), SHEET_STATE_EXPANDED);
    st_lv.list_items.insert("bs_inner_list".into(), (0..30).map(|i| format!("Row {i}")).collect());
    st_lv.offsets.insert("bs_inner_list".into(), 25.0);
    ui_lv.frame(Input { time: 1.0, ..Input::default() }, b, 0.0, |f| {
        LIST_SHEET_ROOT.render(f, b, &mut st_lv);
    });
    ui_lv.frame(Input { pointer: Some([180.0, 150.0]), pointer_kind: PointerKind::Touch, pressed: true, time: 1.1, ..Input::default() }, b, 0.0, |f| {
        LIST_SHEET_ROOT.render(f, b, &mut st_lv);
    });
    // First drag down 40dp -> ListView absorbs the 25dp scroll down to top
    let clv1 = ui_lv.frame(Input { pointer: Some([180.0, 190.0]), pointer_kind: PointerKind::Touch, time: 1.2, ..Input::default() }, b, 0.0, |f| {
        LIST_SHEET_ROOT.render(f, b, &mut st_lv);
    });
    assert!((sheet_height(&clv1).unwrap() - 600.0).abs() < 1.0);
    assert!(st_lv.offsets["bs_inner_list"] <= 0.5);
    // Continue pulling down 120dp without lifting finger -> hands off smoothly to BottomSheet!
    let clv2 = ui_lv.frame(Input { pointer: Some([180.0, 310.0]), pointer_kind: PointerKind::Touch, time: 1.3, ..Input::default() }, b, 0.0, |f| {
        LIST_SHEET_ROOT.render(f, b, &mut st_lv);
    });
    let h_lv2 = sheet_height(&clv2).unwrap();
    assert!((h_lv2 - 480.0).abs() < 2.0, "Continuous downward drag past ListView top must hand off smoothly, got {h_lv2}");
    assert_eq!(st_lv.offsets["bs_inner_list"], 0.0);
}

#[test]
fn wave19a_constraint_layout_physical_vs_logical_pinning_in_rtl() {
    use bhoo::android_xml::{
        RULE_CHAIN_STYLE_H, RULE_CONSTRAINT_END_TO_END, RULE_CONSTRAINT_END_TO_START,
        RULE_CONSTRAINT_HORIZONTAL_BIAS, RULE_CONSTRAINT_PHYS_FLAGS,
        RULE_CONSTRAINT_START_TO_END, RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP,
    };

    let rect_of = |ui: &Ui, label: &str| {
        ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds
    };

    static LOG_LEFT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static PHYS_LEFT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_PHYS_FLAGS, anchor: "1" },
    ];
    static PHYS_RIGHT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_PHYS_FLAGS, anchor: "2" },
    ];
    static PHYS_BOTH_LOG_BIAS_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_HORIZONTAL_BIAS, anchor: "0.25" },
        RelativeRule { kind: RULE_CONSTRAINT_PHYS_FLAGS, anchor: "3" },
    ];
    static PHYS_ALL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_HORIZONTAL_BIAS, anchor: "0.25" },
        RelativeRule { kind: RULE_CONSTRAINT_PHYS_FLAGS, anchor: "7" },
    ];
    static PHYS_TO_RIGHT_OF_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "p_left" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_PHYS_FLAGS, anchor: "1" },
    ];

    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(40.), margin: [10., 0., 0., 0.], text: "LogStart", relative_rules: LOG_LEFT_RULES, ..button("l_start", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), margin: [10., 30., 0., 0.], text: "PhysLeft", relative_rules: PHYS_LEFT_RULES, ..button("p_left", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), margin: [0., 60., 15., 0.], text: "PhysRight", relative_rules: PHYS_RIGHT_RULES, ..button("p_right", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), margin: [0., 90., 0., 0.], text: "PhysBothLogBias", relative_rules: PHYS_BOTH_LOG_BIAS_RULES, ..button("p_both3", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), margin: [20., 120., 40., 0.], margin_relative: [Some(20.), Some(40.)], text: "PhysAll7", relative_rules: PHYS_ALL_RULES, ..button("p_all7", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), margin: [5., 150., 0., 0.], text: "RightOfPhysLeft", relative_rules: PHYS_TO_RIGHT_OF_RULES, ..button("p_after", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_phys_root", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 300., height: 220. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // LTR baseline
    state.rtl = false;
    ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });
    let ltr_phys_left = rect_of(&ui, "PhysLeft").x;
    let ltr_phys_right = rect_of(&ui, "PhysRight").x;
    let ltr_phys_all7 = rect_of(&ui, "PhysAll7").x;
    let ltr_after = rect_of(&ui, "RightOfPhysLeft").x;
    assert!((rect_of(&ui, "LogStart").x - 10.0).abs() < 0.05);
    assert!((ltr_phys_left - 10.0).abs() < 0.05);
    assert!((ltr_phys_right - 245.0).abs() < 0.05);
    // PhysAll7: [20 .. 260], span=240, avail=200, bias=0.25 -> 20 + 50 = 70
    assert!((ltr_phys_all7 - 70.0).abs() < 0.05);
    assert!((ltr_after - 55.0).abs() < 0.05);

    // RTL: logical mirrors, physical stays pinned!
    state.rtl = true;
    ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });
    assert!((rect_of(&ui, "LogStart").x - 260.0).abs() < 0.05, "Logical Start_toStartOf must still mirror in RTL");
    assert!((rect_of(&ui, "PhysLeft").x - ltr_phys_left).abs() < 0.05, "bit 0 physical Left must NOT mirror in RTL");
    assert!((rect_of(&ui, "PhysRight").x - ltr_phys_right).abs() < 0.05, "bit 1 physical Right must NOT mirror in RTL");
    assert!((rect_of(&ui, "RightOfPhysLeft").x - ltr_after).abs() < 0.05, "bit 0 physical Left_toRightOf must stay right of anchor in RTL");
    // flags=3: both anchors physical [0..300], bit 2 = 0 -> bias 0.25 flips to 0.75 -> (300-40)*0.75 = 195
    assert!((rect_of(&ui, "PhysBothLogBias").x - 195.0).abs() < 0.05);
    // flags=7: anchors, bias, and margins all physical -> identical x = 70 in RTL!
    assert!((rect_of(&ui, "PhysAll7").x - ltr_phys_all7).abs() < 0.05, "bit 2 physical bias + margins must match LTR in RTL");

    // Physical horizontal chain vs logical horizontal chain in RTL
    static CHAIN_A_PHYS: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "ch_b" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CHAIN_STYLE_H, anchor: "packed" },
        RelativeRule { kind: RULE_CONSTRAINT_HORIZONTAL_BIAS, anchor: "0.0" },
        RelativeRule { kind: RULE_CONSTRAINT_PHYS_FLAGS, anchor: "7" },
    ];
    static CHAIN_B_PHYS: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "ch_a" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_PHYS_FLAGS, anchor: "7" },
    ];
    static CHAIN_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { width: Size::Dp(60.), text: "ChA", relative_rules: CHAIN_A_PHYS, ..button("ch_a", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(60.), text: "ChB", relative_rules: CHAIN_B_PHYS, ..button("ch_b", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_phys_chain", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), b, 0., |f| { CHAIN_ROOT.render(f, b, &mut state); });
    assert!((rect_of(&ui, "ChA").x - 0.0).abs() < 0.05 && (rect_of(&ui, "ChB").x - 60.0).abs() < 0.05, "Physical horizontal chain must stay left-to-right in RTL");
}

#[test]
fn wave19a_elevation_object_animator_transitions_and_reduced_motion() {
    use bhoo::android_xml::{RULE_ELEVATION_IN, RULE_ELEVATION_OUT};

    static SW_ELEV_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ELEVATION_IN, anchor: "0,8|400|0|linear" },
        RelativeRule { kind: RULE_ELEVATION_OUT, anchor: "8,0|400|0|linear" },
    ];
    static SW_OFFSET_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ELEVATION_IN, anchor: "2,10|400|100|decelerate" },
    ];
    static SWITCHER: Node = Node {
        kind: Kind::ViewSwitcher,
        relative_rules: SW_ELEV_RULES,
        children: &[
            Node { kind: Kind::View, width: Size::Dp(120.), height: Size::Dp(48.), background: Some(Color(220, 60, 60, 255)), ..button("el_c0", Visibility::Visible, Size::Dp(48.), 0.) },
            Node { kind: Kind::View, width: Size::Dp(120.), height: Size::Dp(48.), background: Some(Color(60, 60, 220, 255)), ..button("el_c1", Visibility::Visible, Size::Dp(48.), 0.) },
        ],
        ..button("w19_sw_elev", Visibility::Visible, Size::Match, 0.)
    };
    static ANIMATOR_OFFSET: Node = Node {
        kind: Kind::ViewAnimator,
        relative_rules: SW_OFFSET_RULES,
        children: &[
            Node { kind: Kind::View, width: Size::Dp(120.), height: Size::Dp(48.), background: Some(Color(10, 20, 30, 255)), ..button("el_o0", Visibility::Visible, Size::Dp(48.), 0.) },
            Node { kind: Kind::View, width: Size::Dp(120.), height: Size::Dp(48.), background: Some(Color(30, 60, 90, 255)), ..button("el_o1", Visibility::Visible, Size::Dp(48.), 0.) },
        ],
        ..button("w19_va_off", Visibility::Visible, Size::Match, 0.)
    };
    static AVF_ELEV_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ELEVATION_IN, anchor: "0,12|400|0|linear" },
        RelativeRule { kind: RULE_ELEVATION_OUT, anchor: "12,0|400|0|linear" },
    ];
    static AVF_ELEV: Node = Node {
        kind: Kind::AdapterViewFlipper,
        relative_rules: AVF_ELEV_RULES,
        ..button("w19_avf_elev", Visibility::Visible, Size::Match, 0.)
    };

    let max_shadow_blur = |cmds: &[DrawCommand]| -> f32 {
        cmds.iter()
            .filter_map(|c| match c {
                DrawCommand::Shadow { blur, .. } => Some(*blur),
                _ => None,
            })
            .fold(0.0_f32, f32::max)
    };
    let shadow_count = |cmds: &[DrawCommand]| -> usize {
        cmds.iter().filter(|c| matches!(c, DrawCommand::Shadow { .. })).count()
    };

    let b = Rect { x: 0., y: 0., width: 200., height: 100. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // 1. Initial child 0: no transition yet -> no shadows
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { SWITCHER.render(f, b, &mut state); });
    assert_eq!(shadow_count(&c0), 0);

    // 2. Switch to child 1 at t=2.0 -> active transition over 400ms
    state.displayed_children.insert("w19_sw_elev".into(), 1);
    let c_start = ui.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| { SWITCHER.render(f, b, &mut state); });
    assert!(ui.needs_redraw());
    // At t=2.0: outgoing is at 8dp (3 shadows), incoming is at 0dp (0 shadows) -> 3 shadows total
    assert_eq!(shadow_count(&c_start), 3);
    let blur_at_8 = max_shadow_blur(&c_start);
    assert!(blur_at_8 > 18.0, "8dp elevation should scale blur past 6dp baseline (18.0), got {blur_at_8}");

    // At t=2.2 (midpoint): BOTH outgoing (4dp) and incoming (4dp) paint 3 shadows each -> 6 shadows total
    let c_mid = ui.frame(Input { time: 2.2, ..Input::default() }, b, 0., |f| { SWITCHER.render(f, b, &mut state); });
    assert!(ui.needs_redraw());
    assert_eq!(shadow_count(&c_mid), 6, "Both outgoing and incoming children must emit shadows mid-transition");
    let blur_at_4 = max_shadow_blur(&c_mid);
    assert!(blur_at_4 > 5.0 && blur_at_4 < blur_at_8, "4dp midpoint blur ({blur_at_4}) must lie strictly between 2dp and 8dp ({blur_at_8})");

    // At t=2.45 (completed): outgoing gone, incoming settled at 8dp (3 shadows), no redraw
    let c_done = ui.frame(Input { time: 2.45, ..Input::default() }, b, 0., |f| { SWITCHER.render(f, b, &mut state); });
    assert!(!ui.needs_redraw());
    assert_eq!(shadow_count(&c_done), 3);
    assert!((max_shadow_blur(&c_done) - blur_at_8).abs() < 0.05);

    // 3. Reduced motion: switching back to child 0 snaps immediately to final 8dp shadow with no outgoing shadow and no redraw!
    ui.motion_preference = MotionPreference::Reduced;
    state.displayed_children.insert("w19_sw_elev".into(), 0);
    let c_rm = ui.frame(Input { time: 3.0, ..Input::default() }, b, 0., |f| { SWITCHER.render(f, b, &mut state); });
    assert!(!ui.needs_redraw(), "reduced_motion must snap elevation transition immediately");
    assert_eq!(shadow_count(&c_rm), 3, "Only incoming child at final elevation should paint under reduced_motion");
    assert!((max_shadow_blur(&c_rm) - blur_at_8).abs() < 0.05);

    // 4. Start offset (100ms) + decelerate interpolator on ViewAnimator
    ui.motion_preference = MotionPreference::Full;
    ui.frame(Input { time: 4.0, ..Input::default() }, b, 0., |f| { ANIMATOR_OFFSET.render(f, b, &mut state); });
    state.displayed_children.insert("w19_va_off".into(), 1);
    ui.frame(Input { time: 5.0, ..Input::default() }, b, 0., |f| { ANIMATOR_OFFSET.render(f, b, &mut state); });
    // t=5.05 (< 100ms offset): still at from_dp=2.0 (max blur = 5.0)
    let c_off = ui.frame(Input { time: 5.05, ..Input::default() }, b, 0., |f| { ANIMATOR_OFFSET.render(f, b, &mut state); });
    assert!((max_shadow_blur(&c_off) - 5.0).abs() < 0.05, "Before start_offset must hold from_dp=2.0");
    // t=5.30 (elapsed=0.3 -> (0.3-0.1)/0.4 = 0.5 -> decelerate(0.5) = 0.75 -> 2 + 8*0.75 = 8dp)
    let c_dec = ui.frame(Input { time: 5.30, ..Input::default() }, b, 0., |f| { ANIMATOR_OFFSET.render(f, b, &mut state); });
    assert!((max_shadow_blur(&c_dec) - blur_at_8).abs() < 0.05, "Decelerate at 0.5 must reach 8dp");

    // 5. AdapterViewFlipper elevation transition + reduced motion snap
    state.list_items.insert("w19_avf_elev".into(), vec!["First".into(), "Second".into()]);
    let c_avf0 = ui.frame(Input { time: 6.0, ..Input::default() }, b, 0., |f| { AVF_ELEV.render(f, b, &mut state); });
    assert_eq!(shadow_count(&c_avf0), 0);
    state.displayed_children.insert("w19_avf_elev".into(), 1);
    ui.frame(Input { time: 7.0, ..Input::default() }, b, 0., |f| { AVF_ELEV.render(f, b, &mut state); });
    let c_avf_mid = ui.frame(Input { time: 7.2, ..Input::default() }, b, 0., |f| { AVF_ELEV.render(f, b, &mut state); });
    assert!(ui.needs_redraw());
    assert_eq!(shadow_count(&c_avf_mid), 6, "AdapterViewFlipper must paint both outgoing and incoming shadows mid-transition");
    ui.motion_preference = MotionPreference::Reduced;
    state.displayed_children.insert("w19_avf_elev".into(), 0);
    let c_avf_rm = ui.frame(Input { time: 8.0, ..Input::default() }, b, 0., |f| { AVF_ELEV.render(f, b, &mut state); });
    assert!(!ui.needs_redraw());
    assert_eq!(shadow_count(&c_avf_rm), 3);
    assert!(max_shadow_blur(&c_avf_rm) > blur_at_8, "12dp snapped blur must exceed 8dp blur");
}

#[test]
fn wave20a_app_bar_ext_lift_color_animated_snap_margins_and_compress() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_APP_BAR_EXT, RULE_APP_BAR_LIFT_ON_SCROLL, RULE_APP_BAR_SCROLL_FLAGS,
        RULE_COORDINATOR_SCROLL_BEHAVIOR,
    };

    static COORD_SCROLL: &[RelativeRule] = &[RelativeRule {
        kind: RULE_COORDINATOR_SCROLL_BEHAVIOR,
        anchor: "",
    }];
    static BODY: Node = Node {
        kind: Kind::View,
        height: Size::Dp(1200.),
        ..button("w20_ab_body", Visibility::Visible, Size::Match, 0.)
    };
    static TABS_PINNED: Node = Node {
        kind: Kind::View,
        height: Size::Dp(48.),
        background: Some(Color(10, 120, 200, 255)),
        ..button("w20_ab_tabs", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 360., height: 640. };
    let red = Color(220, 60, 60, 255);
    let blue = Color(10, 120, 200, 255);
    let rect_by_color = |cmds: &[DrawCommand], color: Color| -> Option<Rect> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color: c0, .. } if *c0 == color => Some(*bounds),
            _ => None,
        })
    };

    // 1. liftOnScrollColor (RULE_APP_BAR_EXT "lift_color=230,224,233,255") smoothly interpolates via app_bar_lift_motion
    static LIFT_EXT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_APP_BAR_LIFT_ON_SCROLL, anchor: "1" },
        RelativeRule { kind: RULE_APP_BAR_EXT, anchor: "lift_color=230,224,233,255" },
    ];
    static LIFT_TB: Node = Node {
        kind: Kind::Toolbar,
        text: "CustomLift",
        height: Size::Dp(64.),
        ..button("w20_lift_tb", Visibility::Visible, Size::Match, 0.)
    };
    static LIFT_COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Linear,
                height: Size::Dp(64.),
                background: Some(Color(255, 255, 255, 255)),
                relative_rules: LIFT_EXT_RULES,
                children: std::slice::from_ref(&LIFT_TB),
                ..button("w20_lift_ab", Visibility::Visible, Size::Match, 0.)
            },
            Node {
                kind: Kind::Scroll,
                relative_rules: COORD_SCROLL,
                children: std::slice::from_ref(&BODY),
                ..button("w20_lift_sc", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("w20_lift_coord", Visibility::Visible, Size::Match, 0.)
    };

    let appbar_fill = |cmds: &[DrawCommand]| -> Option<Color> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color, .. }
                if (bounds.height - 64.0).abs() < 0.5 && (bounds.width - 360.0).abs() < 0.5 =>
            {
                Some(*color)
            }
            _ => None,
        })
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // Resting at scroll=0 -> white base
    let cl0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { LIFT_COORD.render(f, b, &mut state); });
    assert_eq!(appbar_fill(&cl0), Some(Color(255, 255, 255, 255)));
    assert!(!ui.needs_redraw());

    // Scrolled -> starts 0.15s lift interpolation toward Color(230, 224, 233, 255)
    state.offsets.insert("w20_lift_sc".into(), 40.0);
    ui.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| { LIFT_COORD.render(f, b, &mut state); });
    assert!(ui.needs_redraw());
    let cl_mid = ui.frame(Input { time: 2.075, ..Input::default() }, b, 0., |f| { LIFT_COORD.render(f, b, &mut state); });
    let mid_col = appbar_fill(&cl_mid).unwrap();
    assert!(mid_col.0 > 230 && mid_col.0 < 255 && mid_col.1 > 224 && mid_col.1 < 255, "mid lift color should interpolate cleanly, got {mid_col:?}");
    let cl_end = ui.frame(Input { time: 2.20, ..Input::default() }, b, 0., |f| { LIFT_COORD.render(f, b, &mut state); });
    assert_eq!(appbar_fill(&cl_end), Some(Color(230, 224, 233, 255)));
    assert!(!ui.needs_redraw());
    // Ensure no default 8% primary overlay was stacked on top
    assert!(!cl_end.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(0x67, 0x50, 0xA4, _), .. })));

    // Reduced motion snaps lift color back to white immediately
    ui.motion_preference = MotionPreference::Reduced;
    state.offsets.insert("w20_lift_sc".into(), 0.0);
    let cl_rm = ui.frame(Input { time: 3.0, ..Input::default() }, b, 0., |f| { LIFT_COORD.render(f, b, &mut state); });
    assert_eq!(appbar_fill(&cl_rm), Some(Color(255, 255, 255, 255)));
    assert!(!ui.needs_redraw());

    // 2. Animated snap settle over 0.22s quadratic ease-out + reduced_motion instant snap
    static SNAP_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_APP_BAR_SCROLL_FLAGS,
        anchor: "scroll,snap",
    }];
    static SNAP_CHILDREN: &[Node] = &[
        Node { kind: Kind::View, height: Size::Dp(64.), background: Some(Color(220, 60, 60, 255)), relative_rules: SNAP_RULES, ..button("w20_snap_tb", Visibility::Visible, Size::Match, 0.) },
        TABS_PINNED,
    ];
    static SNAP_COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, height: Size::Wrap, children: SNAP_CHILDREN, ..button("w20_snap_ab", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Scroll, relative_rules: COORD_SCROLL, children: std::slice::from_ref(&BODY), ..button("w20_snap_sc", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("w20_snap_coord", Visibility::Visible, Size::Match, 0.)
    };

    let mut ui_snap = Ui::default();
    state.offsets.insert("w20_snap_sc".into(), 0.0);
    let cs0 = ui_snap.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { SNAP_COORD.render(f, b, &mut state); });
    assert!((rect_by_color(&cs0, blue).unwrap().y - 64.0).abs() < 0.05);

    // Active scroll change to 40dp (> 50% of 64): holds 40dp collapse while active, schedules idle wakeup
    state.offsets.insert("w20_snap_sc".into(), 40.0);
    let cs_act = ui_snap.frame(Input { time: 1.05, ..Input::default() }, b, 0., |f| { SNAP_COORD.render(f, b, &mut state); });
    assert!((rect_by_color(&cs_act, blue).unwrap().y - 24.0).abs() < 0.05, "Active scroll must not snap prematurely");
    assert!(ui_snap.next_wakeup().is_some(), "Partial snap scroll must schedule idle wakeup");

    // Idle after 0.10s (scroll unchanged at 40dp): starts 0.22s ease-out settle toward 64dp
    ui_snap.frame(Input { time: 1.15, ..Input::default() }, b, 0., |f| { SNAP_COORD.render(f, b, &mut state); });
    assert!(ui_snap.needs_redraw(), "Snap settle animation must request continuous frames");

    // Midpoint (+0.11s -> p=0.5 -> quad ease-out = 0.75 -> collapse = 40 + 24*0.75 = 58 -> tabs.y = 6)
    let cs_mid = ui_snap.frame(Input { time: 1.26, ..Input::default() }, b, 0., |f| { SNAP_COORD.render(f, b, &mut state); });
    assert!((rect_by_color(&cs_mid, blue).unwrap().y - 6.0).abs() < 0.2, "Expected tabs.y ~= 6.0 at midpoint, got {}", rect_by_color(&cs_mid, blue).unwrap().y);
    assert!(ui_snap.needs_redraw());

    // Settled (+0.25s >= 0.22s): collapse = 64 -> tabs.y = 0, no redraw/wakeup
    let cs_done = ui_snap.frame(Input { time: 1.40, ..Input::default() }, b, 0., |f| { SNAP_COORD.render(f, b, &mut state); });
    assert!(rect_by_color(&cs_done, red).is_none());
    assert!((rect_by_color(&cs_done, blue).unwrap().y - 0.0).abs() < 0.05);
    assert!(!ui_snap.needs_redraw());
    assert!(ui_snap.next_wakeup().is_none());

    // Reduced motion: scroll to 20dp (< 50%), wait idle -> snaps immediately back to 0 collapse (tabs.y = 64) with no animation
    ui_snap.motion_preference = MotionPreference::Reduced;
    state.offsets.insert("w20_snap_sc".into(), 20.0);
    let cs_rm_act = ui_snap.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| { SNAP_COORD.render(f, b, &mut state); });
    assert!((rect_by_color(&cs_rm_act, blue).unwrap().y - 44.0).abs() < 0.05);
    let cs_rm_idle = ui_snap.frame(Input { time: 2.15, ..Input::default() }, b, 0., |f| { SNAP_COORD.render(f, b, &mut state); });
    assert!((rect_by_color(&cs_rm_idle, blue).unwrap().y - 64.0).abs() < 0.05, "reduced_motion must snap immediately to expanded endpoint");
    assert!(!ui_snap.needs_redraw());
    assert!(ui_snap.next_wakeup().is_none());

    // 3. snapMargins: 60dp child + top 12dp + bottom 8dp = 80dp range (midpoint = 40dp!)
    static SNAP_MARGINS_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_APP_BAR_SCROLL_FLAGS,
        anchor: "scroll,snap,snapMargins",
    }];
    static SNAP_M_CHILDREN: &[Node] = &[
        Node { kind: Kind::View, height: Size::Dp(60.), margin: [0., 12., 0., 8.], background: Some(Color(220, 60, 60, 255)), relative_rules: SNAP_MARGINS_RULES, ..button("w20_sm_tb", Visibility::Visible, Size::Match, 0.) },
        TABS_PINNED,
    ];
    static SNAP_M_COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, height: Size::Wrap, children: SNAP_M_CHILDREN, ..button("w20_sm_ab", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Scroll, relative_rules: COORD_SCROLL, children: std::slice::from_ref(&BODY), ..button("w20_sm_sc", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("w20_sm_coord", Visibility::Visible, Size::Match, 0.)
    };
    let mut ui_sm = Ui::default();
    ui_sm.motion_preference = MotionPreference::Reduced;
    // scroll = 35dp (< 40dp midpoint with margins, whereas > 30dp without margins) -> snaps back to 0 (tabs.y = 80)
    state.offsets.insert("w20_sm_sc".into(), 35.0);
    let csm35 = ui_sm.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| { SNAP_M_COORD.render(f, b, &mut state); });
    assert!((rect_by_color(&csm35, blue).unwrap().y - 80.0).abs() < 0.05, "snapMargins midpoint=40 must expand 35dp back to 80dp");
    // scroll = 45dp (>= 40dp midpoint) -> collapses BOTH child AND vertical margins completely (tabs.y = 0)
    state.offsets.insert("w20_sm_sc".into(), 45.0);
    ui_sm.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { SNAP_M_COORD.render(f, b, &mut state); });
    let csm45 = ui_sm.frame(Input { time: 1.2, ..Input::default() }, b, 0., |f| { SNAP_M_COORD.render(f, b, &mut state); });
    assert!((rect_by_color(&csm45, blue).unwrap().y - 0.0).abs() < 0.05, "snapMargins must collapse vertical margins down to tabs.y=0");

    // 4. layout_scrollEffect="compress" (RULE_APP_BAR_EXT "effect=compress") vs rigid scroll
    static COMPRESS_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_APP_BAR_SCROLL_FLAGS, anchor: "scroll" },
        RelativeRule { kind: RULE_APP_BAR_EXT, anchor: "effect=compress" },
    ];
    static RIGID_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_APP_BAR_SCROLL_FLAGS, anchor: "scroll" },
    ];
    static COMPRESS_CHILDREN: &[Node] = &[
        Node { kind: Kind::View, height: Size::Dp(64.), background: Some(Color(220, 60, 60, 255)), relative_rules: COMPRESS_RULES, ..button("w20_cmp_tb", Visibility::Visible, Size::Match, 0.) },
        TABS_PINNED,
    ];
    static RIGID_CHILDREN: &[Node] = &[
        Node { kind: Kind::View, height: Size::Dp(64.), background: Some(Color(220, 60, 60, 255)), relative_rules: RIGID_RULES, ..button("w20_rgd_tb", Visibility::Visible, Size::Match, 0.) },
        TABS_PINNED,
    ];
    static COMPRESS_COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, height: Size::Wrap, children: COMPRESS_CHILDREN, ..button("w20_cmp_ab", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Scroll, relative_rules: COORD_SCROLL, children: std::slice::from_ref(&BODY), ..button("w20_cmp_sc", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("w20_cmp_coord", Visibility::Visible, Size::Match, 0.)
    };
    static RIGID_COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node { kind: Kind::Linear, height: Size::Wrap, children: RIGID_CHILDREN, ..button("w20_rgd_ab", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Scroll, relative_rules: COORD_SCROLL, children: std::slice::from_ref(&BODY), ..button("w20_rgd_sc", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("w20_rgd_coord", Visibility::Visible, Size::Match, 0.)
    };

    state.offsets.insert("w20_cmp_sc".into(), 32.0);
    state.offsets.insert("w20_rgd_sc".into(), 32.0);
    let c_cmp = ui.frame(Input::default(), b, 0., |f| { COMPRESS_COORD.render(f, b, &mut state); });
    let c_rgd = ui.frame(Input::default(), b, 0., |f| { RIGID_COORD.render(f, b, &mut state); });
    // Pinned tabs both sit at y = 32.0
    assert!((rect_by_color(&c_cmp, blue).unwrap().y - 32.0).abs() < 0.05);
    assert!((rect_by_color(&c_rgd, blue).unwrap().y - 32.0).abs() < 0.05);
    // Rigid scroll shifts painted rect to y = -32.0; compress translates smoothly to y = -14.4 inside clip [0..32]
    let rgd_y = rect_by_color(&c_rgd, red).unwrap().y;
    let cmp_y = rect_by_color(&c_cmp, red).unwrap().y;
    assert!((rgd_y - (-32.0)).abs() < 0.05);
    assert!(cmp_y > rgd_y + 10.0 && cmp_y < 0.0, "compress child y ({cmp_y}) must smoothly compress rather than sliding rigidly to {rgd_y}");
    // Verify PushClip clips the compressed child to [0..32]
    assert!(c_cmp.iter().any(|c| matches!(c, DrawCommand::PushClip(r) if (r.y - 0.0).abs() < 0.05 && (r.height - 32.0).abs() < 0.05)));
}

#[test]
fn wave20a_drawer_lock_mode_static_runtime_dual_rtl_and_reduced_motion() {
    use bhoo::android_xml::{
        AndroidTheme, DRAWER_LOCK_MODE_LOCKED_CLOSED, DRAWER_LOCK_MODE_LOCKED_OPEN,
        DRAWER_LOCK_MODE_UNDEFINED, DRAWER_LOCK_MODE_UNLOCKED, RULE_DRAWER_LAYOUT,
        RULE_DRAWER_LOCK_MODE, RULE_NAV_BAR, RULE_NAV_ITEM,
    };

    static DL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_DRAWER_LAYOUT, anchor: "" }];
    static START_NAV: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "drawer|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "lm_home|StartLocked|home" },
    ];
    static DUAL_DL: Node = Node {
        kind: Kind::Frame,
        relative_rules: DL_RULES,
        children: &[
            Node { text: "BodyAction", width: Size::Match, height: Size::Dp(48.), ..button("lm_body", Visibility::Visible, Size::Dp(48.), 0.) },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(240.),
                height: Size::Match,
                layout_gravity: [0, 0],
                layout_gravity_explicit: true,
                relative_rules: START_NAV,
                ..button("lm_start", Visibility::Visible, Size::Match, 0.)
            },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(220.),
                height: Size::Match,
                layout_gravity: [2, 0],
                layout_gravity_explicit: true,
                children: &[Node { kind: Kind::Text, text: "EndLocked", ..button("lm_end_txt", Visibility::Visible, Size::Dp(40.), 0.) }],
                ..button("lm_end", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("w20_dl", Visibility::Visible, Size::Match, 0.)
    };

    static STATIC_LOCK_CHILD: &[RelativeRule] = &[
        RelativeRule { kind: RULE_DRAWER_LOCK_MODE, anchor: "locked_open" },
    ];
    static STATIC_DL: Node = Node {
        kind: Kind::Frame,
        relative_rules: DL_RULES,
        children: &[
            Node { text: "StaticBody", width: Size::Match, height: Size::Dp(48.), ..button("sl_body", Visibility::Visible, Size::Dp(48.), 0.) },
            Node {
                kind: Kind::Linear,
                width: Size::Dp(240.),
                height: Size::Match,
                layout_gravity: [0, 0],
                layout_gravity_explicit: true,
                relative_rules: STATIC_LOCK_CHILD,
                children: &[Node { kind: Kind::Text, text: "StaticPanel", ..button("sl_txt", Visibility::Visible, Size::Dp(40.), 0.) }],
                ..button("sl_panel", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("w20_static_dl", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 360., height: 300. };
    let has_text = |cmds: &[DrawCommand], needle: &str| {
        cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == needle))
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. LOCKED_CLOSED (1) on start drawer: forces closed even if drawer_open=true, blocks 24dp edge swipe
    state.drawer_open.insert("w20_dl".into(), true);
    state.drawer_lock_mode.insert("w20_dl".into(), DRAWER_LOCK_MODE_LOCKED_CLOSED);
    let c_lc = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    assert!(!has_text(&c_lc, "StartLocked"), "LOCKED_CLOSED must force start drawer closed");
    assert_eq!(state.drawer_open.get("w20_dl"), Some(&false));

    // Attempt 24dp closed-edge swipe on left edge (x=8 -> 120) -> blocked!
    ui.frame(Input { pointer: Some([8., 100.]), pressed: true, time: 2.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([120., 100.]), time: 2.1, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    let c_edge = ui.frame(Input { pointer: Some([120., 100.]), released: true, time: 2.2, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    assert!(!has_text(&c_edge, "StartLocked"), "LOCKED_CLOSED must block 24dp closed-edge swipe");
    assert_eq!(state.drawer_open.get("w20_dl"), Some(&false));

    // End drawer remains UNLOCKED -> right-edge swipe still opens it!
    ui.frame(Input { pointer: Some([352., 100.]), pressed: true, time: 3.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([230., 100.]), time: 3.1, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([230., 100.]), released: true, time: 3.2, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    assert_eq!(state.drawer_open.get("w20_dl:end"), Some(&true));
    state.drawer_open.insert("w20_dl:end".into(), false);
    ui.frame(Input { time: 3.6, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });

    // 2. LOCKED_OPEN (2) on start drawer: forces open, blocks drag-to-close, blocks scrim-tap AND Escape!
    state.drawer_lock_mode.insert("w20_dl".into(), DRAWER_LOCK_MODE_LOCKED_OPEN);
    ui.frame(Input { time: 4.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    let c_lo = ui.frame(Input { time: 4.35, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    assert!(has_text(&c_lo, "StartLocked"), "LOCKED_OPEN must force start drawer open");
    assert_eq!(state.drawer_open.get("w20_dl"), Some(&true));

    // Attempt horizontal drag-to-close (x=200 -> 20) -> blocked!
    ui.frame(Input { pointer: Some([200., 120.]), pressed: true, time: 5.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([20., 120.]), time: 5.1, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    let c_drag = ui.frame(Input { pointer: Some([20., 120.]), released: true, time: 5.2, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    assert!(has_text(&c_drag, "StartLocked"), "LOCKED_OPEN must block drag-to-close");
    assert_eq!(state.drawer_open.get("w20_dl"), Some(&true));

    // Attempt scrim tap (x=310, y=150) -> blocked!
    ui.frame(Input { pointer: Some([310., 150.]), pressed: true, time: 5.4, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    let c_scrim = ui.frame(Input { pointer: Some([310., 150.]), released: true, time: 5.5, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    assert!(has_text(&c_scrim, "StartLocked"), "LOCKED_OPEN must block scrim-tap dismissal");
    assert_eq!(state.drawer_open.get("w20_dl"), Some(&true));

    // Attempt Escape -> blocked!
    ui.frame(Input { key: Some(KeyInput { key: Key::Escape, pressed: true, shift: false, repeat: false }), time: 5.7, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    assert_eq!(state.drawer_open.get("w20_dl"), Some(&true));

    // 3. Reduced motion: LOCKED_OPEN -> LOCKED_CLOSED snaps closed immediately with no redraw
    ui.motion_preference = MotionPreference::Reduced;
    state.drawer_lock_mode.insert("w20_dl".into(), DRAWER_LOCK_MODE_LOCKED_CLOSED);
    let c_rm = ui.frame(Input { time: 6.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut state); });
    assert!(!has_text(&c_rm, "StartLocked"));
    assert!(!ui.needs_redraw(), "reduced_motion lock mode transition must snap immediately");

    // 4. Static RULE_DRAWER_LOCK_MODE="locked_open" overridden by runtime UNLOCKED (0) / UNDEFINED (3)
    let mut ui_st = Ui::default();
    let mut st_stat = LayoutState::default();
    st_stat.theme_style = AndroidTheme::Material3Light;
    let cs0 = ui_st.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { STATIC_DL.render(f, b, &mut st_stat); });
    assert!(has_text(&cs0, "StaticPanel"), "Static locked_open must open drawer");
    // Override with UNDEFINED (3) -> allows scrim tap to close!
    st_stat.drawer_lock_mode.insert("w20_static_dl".into(), DRAWER_LOCK_MODE_UNDEFINED);
    ui_st.frame(Input { pointer: Some([320., 150.]), pressed: true, time: 2.0, ..Input::default() }, b, 0., |f| { STATIC_DL.render(f, b, &mut st_stat); });
    ui_st.frame(Input { pointer: Some([320., 150.]), released: true, time: 2.1, ..Input::default() }, b, 0., |f| { STATIC_DL.render(f, b, &mut st_stat); });
    assert_eq!(st_stat.drawer_open.get("w20_static_dl"), Some(&false), "Runtime UNDEFINED/UNLOCKED must override static locked_open");
    // Override with UNLOCKED (0) -> closed-edge swipe opens again
    st_stat.drawer_lock_mode.insert("w20_static_dl".into(), DRAWER_LOCK_MODE_UNLOCKED);
    ui_st.frame(Input { time: 2.5, ..Input::default() }, b, 0., |f| { STATIC_DL.render(f, b, &mut st_stat); });
    ui_st.frame(Input { pointer: Some([8., 100.]), pressed: true, time: 3.0, ..Input::default() }, b, 0., |f| { STATIC_DL.render(f, b, &mut st_stat); });
    ui_st.frame(Input { pointer: Some([120., 100.]), time: 3.1, ..Input::default() }, b, 0., |f| { STATIC_DL.render(f, b, &mut st_stat); });
    ui_st.frame(Input { pointer: Some([120., 100.]), released: true, time: 3.2, ..Input::default() }, b, 0., |f| { STATIC_DL.render(f, b, &mut st_stat); });
    assert_eq!(st_stat.drawer_open.get("w20_static_dl"), Some(&true));

    // 5. RTL: LOCKED_CLOSED on start ("w20_dl") blocks RIGHT-edge swipe; LOCKED_OPEN on end ("w20_dl:end") stays open on LEFT edge
    let mut ui_rtl = Ui::default();
    let mut st_rtl = LayoutState::default();
    st_rtl.theme_style = AndroidTheme::Material3Light;
    st_rtl.rtl = true;
    st_rtl.drawer_lock_mode.insert("w20_dl".into(), DRAWER_LOCK_MODE_LOCKED_CLOSED);
    ui_rtl.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut st_rtl); });
    ui_rtl.frame(Input { pointer: Some([352., 100.]), pressed: true, time: 2.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut st_rtl); });
    ui_rtl.frame(Input { pointer: Some([220., 100.]), time: 2.1, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut st_rtl); });
    ui_rtl.frame(Input { pointer: Some([220., 100.]), released: true, time: 2.2, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut st_rtl); });
    assert_ne!(st_rtl.drawer_open.get("w20_dl"), Some(&true), "RTL LOCKED_CLOSED start drawer must block right-edge swipe");

    st_rtl.drawer_lock_mode.insert("w20_dl:end".into(), DRAWER_LOCK_MODE_LOCKED_OPEN);
    ui_rtl.frame(Input { time: 3.0, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut st_rtl); });
    let c_rtl_end = ui_rtl.frame(Input { time: 3.35, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut st_rtl); });
    assert!(has_text(&c_rtl_end, "EndLocked"), "RTL LOCKED_OPEN end drawer must open on left edge");
    ui_rtl.frame(Input { pointer: Some([310., 150.]), pressed: true, time: 3.5, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut st_rtl); });
    ui_rtl.frame(Input { pointer: Some([310., 150.]), released: true, time: 3.6, ..Input::default() }, b, 0., |f| { DUAL_DL.render(f, b, &mut st_rtl); });
    assert_eq!(st_rtl.drawer_open.get("w20_dl:end"), Some(&true), "RTL LOCKED_OPEN end drawer must resist scrim tap");
}

#[test]
fn wave20a_constraint_flow_ext_first_last_lines_and_gone_margin_baseline() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BASELINE_TO_BASELINE, RULE_CONSTRAINT_END_TO_END,
        RULE_CONSTRAINT_FLOW_EXT, RULE_CONSTRAINT_GONE_MARGIN, RULE_CONSTRAINT_GONE_MARGINS,
        RULE_CONSTRAINT_START_TO_END, RULE_CONSTRAINT_START_TO_START,
        RULE_CONSTRAINT_TOP_TO_BOTTOM, RULE_CONSTRAINT_TOP_TO_TOP, RULE_FLOW,
    };

    let rect_of = |ui: &Ui, label: &str| {
        ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds
    };

    // 1. Horizontal wrapped Flow (3 rows via max_wrap=2, 5 items of 40dp each, box_w=300, h_gap=10):
    //    default h_style=spread; first line=packed bias=0.0; last line=packed bias=1.0
    static H_FLOW_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "h|chain|10,10|f0,f1,f2,f3,f4|spread,,0.5,0.5,,,2" },
        RelativeRule { kind: RULE_CONSTRAINT_FLOW_EXT, anchor: "packed,packed,0.0,1.0,,,," },
    ];
    static H_FLOW_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Match, height: Size::Wrap, relative_rules: H_FLOW_RULES, ..button("hflow", Visibility::Visible, Size::Wrap, 0.) },
            Node { width: Size::Dp(40.), text: "F0", ..button("f0", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "F1", ..button("f1", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "F2", ..button("f2", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "F3", ..button("f3", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "F4", ..button("f4", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_hflow", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 300., height: 240. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // LTR:
    // Row 0 (F0, F1): packed bias=0.0 -> x = 0, 50
    // Row 1 (F2, F3): spread -> extra=300-80=220, gap=220/3=73.33 -> x = 73.33, 186.67
    // Row 2 (F4): packed bias=1.0 -> x = 300 - 40 = 260
    ui.frame(Input::default(), b, 0., |f| { H_FLOW_ROOT.render(f, b, &mut state); });
    assert!((rect_of(&ui, "F0").x - 0.0).abs() < 0.05);
    assert!((rect_of(&ui, "F1").x - 50.0).abs() < 0.05);
    assert!((rect_of(&ui, "F2").x - (220.0 / 3.0)).abs() < 0.1);
    assert!((rect_of(&ui, "F3").x - (220.0 / 3.0 * 2.0 + 40.0)).abs() < 0.1);
    assert!((rect_of(&ui, "F4").x - 260.0).abs() < 0.05);

    // RTL: Row 0 packed bias=0.0 pins to RIGHT edge (F0 at 260, F1 at 210); Row 2 packed bias=1.0 pins to LEFT edge (F4 at 0)
    state.rtl = true;
    ui.frame(Input::default(), b, 0., |f| { H_FLOW_ROOT.render(f, b, &mut state); });
    assert!((rect_of(&ui, "F0").x - 260.0).abs() < 0.05, "RTL first line packed bias=0.0 must pin to right edge");
    assert!((rect_of(&ui, "F1").x - 210.0).abs() < 0.05);
    assert!((rect_of(&ui, "F4").x - 0.0).abs() < 0.05, "RTL last line packed bias=1.0 must pin to left edge");
    state.rtl = false;

    // 2. Vertical wrapped Flow (3 columns via max_wrap=2, 5 items of 30dp height, box_h=200, v_gap=10)
    //    default v_style=spread; first col=packed bias=0.0; last col=packed bias=1.0
    static V_FLOW_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "v|chain|10,10|v0,v1,v2,v3,v4|,spread,0.5,0.5,,,2" },
        RelativeRule { kind: RULE_CONSTRAINT_FLOW_EXT, anchor: ",,,,packed,packed,0.0,1.0" },
    ];
    static V_FLOW_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Wrap, height: Size::Dp(200.), relative_rules: V_FLOW_RULES, ..button("vflow", Visibility::Visible, Size::Dp(200.), 0.) },
            Node { width: Size::Dp(40.), text: "V0", ..button("v0", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "V1", ..button("v1", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "V2", ..button("v2", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "V3", ..button("v3", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "V4", ..button("v4", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_vflow", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), b, 0., |f| { V_FLOW_ROOT.render(f, b, &mut state); });
    assert!((rect_of(&ui, "V0").y - 0.0).abs() < 0.05);
    assert!((rect_of(&ui, "V1").y - 40.0).abs() < 0.05);
    // Col 1 (V2, V3): spread over 200 -> extra=140, gap=140/3=46.67 -> V2.y = 46.67
    assert!((rect_of(&ui, "V2").y - (140.0 / 3.0)).abs() < 0.1);
    // Col 2 (V4): packed bias=1.0 -> y = 200 - 30 = 170
    assert!((rect_of(&ui, "V4").y - 170.0).abs() < 0.05);

    // 3. Single-line Flow (wrap=none): first_hstyle/first_hbias applies!
    static SINGLE_FLOW_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_FLOW, anchor: "h|none|10,0|s0,s1|spread,,0.5,0.5" },
        RelativeRule { kind: RULE_CONSTRAINT_FLOW_EXT, anchor: "packed,spread,1.0,0.0,,,," },
    ];
    static SINGLE_FLOW_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Dp(300.), height: Size::Wrap, relative_rules: SINGLE_FLOW_RULES, ..button("sflow", Visibility::Visible, Size::Wrap, 0.) },
            Node { width: Size::Dp(40.), text: "S0", ..button("s0", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { width: Size::Dp(40.), text: "S1", ..button("s1", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("cl_sflow", Visibility::Visible, Size::Match, 0.)
    };
    ui.frame(Input::default(), b, 0., |f| { SINGLE_FLOW_ROOT.render(f, b, &mut state); });
    // Packed bias=1.0: total=90, extra=210 -> S0 at 210, S1 at 260
    assert!((rect_of(&ui, "S0").x - 210.0).abs() < 0.05);
    assert!((rect_of(&ui, "S1").x - 260.0).abs() < 0.05);

    // 4. RULE_CONSTRAINT_GONE_MARGIN (223) baseline offset: BOTH 5-value ("s,t,e,b,baseline") and 7-value ("l,t,r,b,s,e,baseline")
    static HDR_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static ANCHOR_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "hdr" },
    ];
    static FOL5_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "anc" },
        RelativeRule { kind: RULE_CONSTRAINT_BASELINE_TO_BASELINE, anchor: "anc" },
        RelativeRule { kind: RULE_CONSTRAINT_GONE_MARGIN, anchor: "15,-1,30,-1,24" },
    ];
    static FOL7_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "fol5" },
        RelativeRule { kind: RULE_CONSTRAINT_BASELINE_TO_BASELINE, anchor: "anc" },
        RelativeRule { kind: RULE_CONSTRAINT_GONE_MARGINS, anchor: "-1,-1,-1,-1,-1,-1,36" },
    ];
    static GONE_BL_ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        children: &[
            Node { kind: Kind::View, width: Size::Dp(100.), height: Size::Dp(40.), relative_rules: HDR_RULES, ..button("hdr", Visibility::Visible, Size::Dp(40.), 0.) },
            Node { kind: Kind::Text, text: "Anchor", text_size: 28., width: Size::Wrap, relative_rules: ANCHOR_RULES, ..button("anc", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Text, text: "Fol5", text_size: 12., width: Size::Wrap, relative_rules: FOL5_RULES, ..button("fol5", Visibility::Visible, Size::Wrap, 0.) },
            Node { kind: Kind::Text, text: "Fol7", text_size: 12., width: Size::Wrap, relative_rules: FOL7_RULES, ..button("fol7", Visibility::Visible, Size::Wrap, 0.) },
        ],
        ..button("cl_gbl", Visibility::Visible, Size::Match, 0.)
    };

    let text_baseline = |cmds: &[DrawCommand], ui: &Ui, label: &str| -> (f32, f32) {
        let off = ui.text_metrics.normal_baseline_offset;
        cmds.iter()
            .find_map(|c| match c {
                DrawCommand::Text { bounds, text, style, .. } if text.as_ref() == label => {
                    Some((bounds.x, bounds.y + bounds.height * 0.5 + style.size * off))
                }
                _ => None,
            })
            .unwrap()
    };

    // Visible anchor: Fol5 & Fol7 share Anchor's baseline
    let c_vis = ui.frame(Input::default(), b, 0., |f| { GONE_BL_ROOT.render(f, b, &mut state); });
    let (_, bl_anc) = text_baseline(&c_vis, &ui, "Anchor");
    let (_, bl_f5_v) = text_baseline(&c_vis, &ui, "Fol5");
    let (_, bl_f7_v) = text_baseline(&c_vis, &ui, "Fol7");
    assert!((bl_anc - bl_f5_v).abs() < 0.05 && (bl_anc - bl_f7_v).abs() < 0.05);

    // Gone anchor: anc collapses to y=40 (below hdr); Fol5 baseline = 40+24=64 (x=15), Fol7 baseline = 40+36=76
    state.visibility.insert("anc".into(), Visibility::Gone);
    let c_gone = ui.frame(Input::default(), b, 0., |f| { GONE_BL_ROOT.render(f, b, &mut state); });
    let (x_f5_g, bl_f5_g) = text_baseline(&c_gone, &ui, "Fol5");
    let (_, bl_f7_g) = text_baseline(&c_gone, &ui, "Fol7");
    assert!((x_f5_g - 15.0).abs() < 0.1, "5-value gone start margin must apply in LTR, got x={x_f5_g}");
    assert!((bl_f5_g - 64.0).abs() < 0.1, "5-value gone baseline margin 24dp must align baseline to 64.0, got {bl_f5_g}");
    assert!((bl_f7_g - 76.0).abs() < 0.1, "7-value gone baseline margin 36dp must align baseline to 76.0, got {bl_f7_g}");

    // Density = 2.0 + RTL: 40*2 + 24*2 = 128.0 baseline, and 5-value start margin=15*2=30 applies from right edge!
    state.density = 2.0;
    state.rtl = true;
    let b2 = Rect { x: 0., y: 0., width: 600., height: 480. };
    let c_gone_2x = ui.frame(Input::default(), b2, 0., |f| { GONE_BL_ROOT.render(f, b2, &mut state); });
    let (_, bl_f5_2x) = text_baseline(&c_gone_2x, &ui, "Fol5");
    assert!((bl_f5_2x - 128.0).abs() < 0.2, "2x density gone baseline must scale to 128.0, got {bl_f5_2x}");
    let f5_box_rtl = c_gone_2x.iter().find_map(|c| match c { DrawCommand::Text { bounds, text, .. } if text.as_ref() == "Fol5" => Some(*bounds), _ => None }).unwrap();
    assert!(((f5_box_rtl.x + f5_box_rtl.width) - (600.0 - 30.0)).abs() < 0.2, "RTL 5-value start gone margin must inset from right edge");
}

#[test]
fn wave20a_animated_selector_src_bg_reverse_rtl_and_reduced_motion() {
    use bhoo::android_xml::{
        RULE_ANIMATED_SELECTOR, RULE_AUTO_MIRRORED, RULE_STATEFUL_BG_IMAGE,
        RULE_STATEFUL_IMAGE_SRC,
    };

    static SRC_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_STATEFUL_IMAGE_SRC, anchor: "64,0|ic_on|" },
        RelativeRule { kind: RULE_STATEFUL_IMAGE_SRC, anchor: "0,0|ic_off|" },
        RelativeRule { kind: RULE_ANIMATED_SELECTOR, anchor: "src:0:64:1|tr_0:100,tr_1:100,tr_2:100" },
        RelativeRule { kind: RULE_AUTO_MIRRORED, anchor: "true" },
    ];
    static BG_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_STATEFUL_BG_IMAGE, anchor: "8,0|bg_on|" },
        RelativeRule { kind: RULE_STATEFUL_BG_IMAGE, anchor: "0,0|bg_off|" },
        RelativeRule { kind: RULE_ANIMATED_SELECTOR, anchor: "bg:0:8:0|bg_t0:80,bg_t1:80" },
    ];
    static SRC_IMG: Node = Node {
        kind: Kind::Image,
        width: Size::Dp(48.),
        height: Size::Dp(48.),
        relative_rules: SRC_RULES,
        ..button("as_img", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static BG_VIEW: Node = Node {
        kind: Kind::View,
        width: Size::Dp(100.),
        height: Size::Dp(48.),
        relative_rules: BG_RULES,
        ..button("as_bg", Visibility::Visible, Size::Dp(48.), 0.)
    };

    let solid_img = |r: u8, g: u8, b: u8| {
        Image::rgba(4, 4, std::iter::repeat_n([r, g, b, 255], 16).flatten().collect()).unwrap()
    };
    let mut state = LayoutState::default();
    let img_off = solid_img(10, 10, 10);
    let img_on = solid_img(20, 200, 20);
    let img_t0 = solid_img(200, 10, 10);
    let img_t1 = solid_img(200, 100, 10);
    let img_t2 = solid_img(200, 200, 10);
    let bg_off = solid_img(40, 40, 80);
    let bg_on = solid_img(40, 180, 240);
    let bg_t0 = solid_img(120, 40, 200);
    let bg_t1 = solid_img(180, 80, 220);

    let (id_off, id_on, id_t0, id_t1, id_t2) = (img_off.id(), img_on.id(), img_t0.id(), img_t1.id(), img_t2.id());
    let (bid_off, bid_on, bid_t0, bid_t1) = (bg_off.id(), bg_on.id(), bg_t0.id(), bg_t1.id());
    state.images.insert("ic_off".into(), img_off);
    state.images.insert("ic_on".into(), img_on);
    state.images.insert("tr_0".into(), img_t0);
    state.images.insert("tr_1".into(), img_t1);
    state.images.insert("tr_2".into(), img_t2);
    state.images.insert("bg_off".into(), bg_off);
    state.images.insert("bg_on".into(), bg_on);
    state.images.insert("bg_t0".into(), bg_t0);
    state.images.insert("bg_t1".into(), bg_t1);

    let img_ids = |cmds: &[DrawCommand]| -> Vec<u32> {
        cmds.iter()
            .filter_map(|c| match c {
                DrawCommand::Image { image, .. } => Some(image.id()),
                _ => None,
            })
            .collect()
    };

    let b = Rect { x: 0., y: 0., width: 100., height: 60. };
    let mut ui = Ui::default();

    // 1. Initial unchecked src -> ic_off, no wakeup
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&c0), vec![id_off]);
    assert!(ui.next_wakeup().is_none());

    // 2. Check -> plays forward tr_0 (0..100ms), tr_1 (100..200ms), tr_2 (200..300ms), then settles on ic_on
    state.checked.insert("as_img".into(), true);
    let cf0 = ui.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&cf0), vec![id_t0]);
    assert!((ui.next_wakeup().unwrap() - 2.1).abs() < 1e-4);

    let cf1 = ui.frame(Input { time: 2.12, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&cf1), vec![id_t1]);
    assert!((ui.next_wakeup().unwrap() - 2.2).abs() < 1e-4);

    // Moving pointer over image (adding hover bit) mid-playback must NOT cancel checked transition
    let cf2 = ui.frame(Input { pointer: Some([24., 24.]), time: 2.25, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&cf2), vec![id_t2]);
    assert!((ui.next_wakeup().unwrap() - 2.3).abs() < 1e-4);

    // Settled at 2.31s -> destination stateful ic_on, no wakeup
    let cf_done = ui.frame(Input { time: 2.31, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&cf_done), vec![id_on]);
    assert!(ui.next_wakeup().is_none());
    assert!(!ui.needs_redraw());

    // 3. Uncheck (reversible rev=1) in RTL -> plays REVERSE tr_2 -> tr_1 -> tr_0 -> ic_off, with RTL mirror transform!
    state.rtl = true;
    state.checked.insert("as_img".into(), false);
    let cr0 = ui.frame(Input { time: 3.0, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&cr0), vec![id_t2], "Reversible transition must start with last frame tr_2");
    assert!(cr0.iter().any(|c| matches!(c, DrawCommand::PushTransform { scale, .. } if scale[0] < 0.0)), "RTL autoMirrored transition frame must mirror horizontally");
    let cr1 = ui.frame(Input { time: 3.12, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&cr1), vec![id_t1]);
    let cr2 = ui.frame(Input { time: 3.25, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&cr2), vec![id_t0]);
    let cr_done = ui.frame(Input { time: 3.31, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&cr_done), vec![id_off]);
    assert!(ui.next_wakeup().is_none());
    state.rtl = false;

    // 4. Background animated-selector (selected bit 3 = 8, non-reversible rev=0)
    let cb0 = ui.frame(Input { time: 4.0, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(img_ids(&cb0), vec![bid_off]);
    state.view_selected.insert("as_bg".into(), true);
    let cbt0 = ui.frame(Input { time: 4.5, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(img_ids(&cbt0), vec![bid_t0]);
    assert!((ui.next_wakeup().unwrap() - 4.58).abs() < 1e-4);
    let cbt1 = ui.frame(Input { time: 4.60, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(img_ids(&cbt1), vec![bid_t1]);
    let cb_done = ui.frame(Input { time: 4.70, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(img_ids(&cb_done), vec![bid_on]);
    assert!(ui.next_wakeup().is_none());

    // Deselect on non-reversible (rev=0) -> jumps straight to bg_off with no wakeup
    state.view_selected.insert("as_bg".into(), false);
    let cb_back = ui.frame(Input { time: 5.0, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(img_ids(&cb_back), vec![bid_off]);
    assert!(ui.next_wakeup().is_none());

    // 5. Reduced motion: skips playback immediately for both src and bg with no wakeup
    ui.motion_preference = MotionPreference::Reduced;
    state.checked.insert("as_img".into(), true);
    let crm_src = ui.frame(Input { time: 6.0, ..Input::default() }, b, 0., |f| { SRC_IMG.render(f, b, &mut state); });
    assert_eq!(img_ids(&crm_src), vec![id_on], "reduced_motion must skip animated-selector src frames");
    assert!(ui.next_wakeup().is_none());
    assert!(!ui.needs_redraw());

    state.view_selected.insert("as_bg".into(), true);
    let crm_bg = ui.frame(Input { time: 6.1, ..Input::default() }, b, 0., |f| { BG_VIEW.render(f, b, &mut state); });
    assert_eq!(img_ids(&crm_bg), vec![bid_on], "reduced_motion must skip animated-selector bg frames");
    assert!(ui.next_wakeup().is_none());
    assert!(!ui.needs_redraw());
}

#[test]
fn wave21a_constraint_default_size_spread_wrap_percent_and_rtl() {
    use bhoo::android_xml::{
        RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, RULE_CONSTRAINT_DEFAULT_SIZE,
        RULE_CONSTRAINT_END_TO_END, RULE_CONSTRAINT_END_TO_START,
        RULE_CONSTRAINT_HEIGHT_PERCENT, RULE_CONSTRAINT_HORIZONTAL_BIAS,
        RULE_CONSTRAINT_SIZE_BOUNDS, RULE_CONSTRAINT_START_TO_END,
        RULE_CONSTRAINT_START_TO_START, RULE_CONSTRAINT_TOP_TO_TOP,
        RULE_CONSTRAINT_VERTICAL_BIAS, RULE_CONSTRAINT_WIDTH_PERCENT,
    };

    static REF_L_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static REF_R_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
    ];
    static SPREAD_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_DEFAULT_SIZE, anchor: "spread,spread" },
        RelativeRule { kind: RULE_CONSTRAINT_SIZE_BOUNDS, anchor: "0,0|0,320,0,0" },
    ];
    static WRAP_SHORT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_DEFAULT_SIZE, anchor: "wrap,wrap" },
        RelativeRule { kind: RULE_CONSTRAINT_HORIZONTAL_BIAS, anchor: "0.25" },
        RelativeRule { kind: RULE_CONSTRAINT_VERTICAL_BIAS, anchor: "0.75" },
        RelativeRule { kind: RULE_CONSTRAINT_SIZE_BOUNDS, anchor: "0,0|60,180,30,100" },
    ];
    static WRAP_LONG_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "ref_l" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "ref_r" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_DEFAULT_SIZE, anchor: "wrap,wrap" },
    ];
    static PCT_DUAL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_CONSTRAINT_START_TO_END, anchor: "ref_l" },
        RelativeRule { kind: RULE_CONSTRAINT_END_TO_START, anchor: "ref_r" },
        RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_BOTTOM_TO_BOTTOM, anchor: "parent" },
        RelativeRule { kind: RULE_CONSTRAINT_DEFAULT_SIZE, anchor: "percent,percent" },
        RelativeRule { kind: RULE_CONSTRAINT_WIDTH_PERCENT, anchor: "0.4" },
        RelativeRule { kind: RULE_CONSTRAINT_HEIGHT_PERCENT, anchor: "0.3" },
        RelativeRule { kind: RULE_CONSTRAINT_HORIZONTAL_BIAS, anchor: "0.2" },
        RelativeRule { kind: RULE_CONSTRAINT_VERTICAL_BIAS, anchor: "0.5" },
        RelativeRule { kind: RULE_CONSTRAINT_SIZE_BOUNDS, anchor: "0,0|0,150,0,0" },
    ];

    static ROOT: Node = Node {
        kind: Kind::ConstraintLayout,
        padding_edges: [10., 10., 10., 10.],
        padding_explicit: [true; 4],
        children: &[
            // Inner bounds = x:[10..410] (w=400), y:[10..310] (h=300)
            // ref_l: margin_start=50, w=20 -> LTR x=60..80; RTL x=340..360
            Node {
                width: Size::Dp(20.),
                margin_relative: [Some(50.), None],
                relative_rules: REF_L_RULES,
                ..button("ref_l", Visibility::Visible, Size::Dp(20.), 0.)
            },
            // ref_r: margin_end=50, w=20 -> LTR x=340..360; RTL x=60..80
            // Span between ref_l and ref_r = [80..340], width = 260 in both LTR and RTL!
            Node {
                width: Size::Dp(20.),
                margin_relative: [None, Some(50.)],
                relative_rules: REF_R_RULES,
                ..button("ref_r", Visibility::Visible, Size::Dp(20.), 0.)
            },
            // spread with 20dp horizontal margins -> span 360 clamped by w_max=320 -> x=50..370
            Node {
                width: Size::Dp(0.),
                margin: [20., 0., 20., 0.],
                relative_rules: SPREAD_RULES,
                ..button("w_spread", Visibility::Visible, Size::Dp(40.), 0.)
            },
            // wrap short text ("Hi") -> clamped up by w_min=60, h_min=30, biased (0.25, 0.75)
            Node {
                kind: Kind::Text,
                id: "w_wrap_short",
                text: "Hi",
                clickable: true,
                width: Size::Dp(0.),
                height: Size::Dp(0.),
                relative_rules: WRAP_SHORT_RULES,
                ..button("w_wrap_short", Visibility::Visible, Size::Dp(0.), 0.)
            },
            // wrap long text between ref_l and ref_r (span=260) -> clamped to <= 260, wraps multi-line height
            Node {
                kind: Kind::Text,
                id: "w_wrap_long",
                text: "This is a much longer sentence that clearly exceeds two hundred sixty logical pixels so match constraint wrap must clamp it",
                clickable: true,
                width: Size::Dp(0.),
                height: Size::Dp(0.),
                relative_rules: WRAP_LONG_RULES,
                ..button("w_wrap_long", Visibility::Visible, Size::Dp(0.), 0.)
            },
            // percent with dual anchors: 40% of parent inner width (400*0.4=160) clamped by w_max=150 -> 150,
            // positioned in [80..340] (span=260) with h_bias=0.2 -> LTR x = 80 + 110*0.2 = 102;
            // height = 30% of 300 = 90, centered in [10..310] -> y = 115
            Node {
                width: Size::Dp(0.),
                relative_rules: PCT_DUAL_RULES,
                ..button("w_pct", Visibility::Visible, Size::Dp(0.), 0.)
            },
        ],
        ..button("cl_def", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 420., height: 320. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });
    let get = |ui: &Ui, label: &str| ui.semantics().iter().find(|n| n.label.as_ref() == label).unwrap().bounds;

    let spread_r = get(&ui, "w_spread");
    assert!((spread_r.width - 320.).abs() < 0.1, "spread clamped to max 320: {spread_r:?}");
    assert!((spread_r.x - 50.).abs() < 0.1, "spread centered in [30..390]: {spread_r:?}");

    let short_r = get(&ui, "Hi");
    assert!((short_r.width - 60.).abs() < 0.1, "wrap_short clamped to w_min 60: {short_r:?}");
    assert!((short_r.height - 30.).abs() < 0.1, "wrap_short clamped to h_min 30: {short_r:?}");
    assert!((short_r.x - 95.).abs() < 0.1, "wrap_short LTR h_bias 0.25 -> 10 + 340*0.25 = 95: {short_r:?}");
    assert!((short_r.y - 212.5).abs() < 0.1, "wrap_short v_bias 0.75 -> 10 + 270*0.75 = 212.5: {short_r:?}");

    let long_r = get(&ui, "This is a much longer sentence that clearly exceeds two hundred sixty logical pixels so match constraint wrap must clamp it");
    assert!((long_r.width - 260.).abs() < 0.1, "wrap_long clamped to anchor span 260: {long_r:?}");
    assert!(long_r.height > 40., "wrap_long wraps vertically to multiple lines: {long_r:?}");

    let pct_r = get(&ui, "w_pct");
    assert!((pct_r.width - 150.).abs() < 0.1, "percent 0.4*400=160 clamped by w_max=150: {pct_r:?}");
    assert!((pct_r.height - 90.).abs() < 0.1, "percent 0.3*300=90: {pct_r:?}");
    assert!((pct_r.x - 102.).abs() < 0.1, "percent LTR bias 0.2 in [80..340] -> 102: {pct_r:?}");
    assert!((pct_r.y - 115.).abs() < 0.1, "percent v_bias 0.5 in [10..310] -> 115: {pct_r:?}");

    // RTL: horizontal bias mirrors cleanly for both wrap and percent
    state.rtl = true;
    ui.frame(Input::default(), b, 0., |f| { ROOT.render(f, b, &mut state); });
    let short_rtl = get(&ui, "Hi");
    assert!((short_rtl.x - 265.).abs() < 0.1, "wrap_short RTL h_bias mirrors to 0.75 -> 10 + 340*0.75 = 265: {short_rtl:?}");
    let pct_rtl = get(&ui, "w_pct");
    assert!((pct_rtl.x - 168.).abs() < 0.1, "percent RTL bias mirrors to 0.8 in [80..340] -> 80 + 110*0.8 = 168: {pct_rtl:?}");
}

#[test]
fn wave21a_compound_animation_list_all_slots_rtl_clip_and_reduced_motion() {
    use bhoo::android_expandable_list::{ExpandableChild, ExpandableGroup, ExpandableListState};
    use bhoo::android_xml::{AndroidTheme, RULE_COMPOUND_ANIMATION_LIST};

    let solid = |r: u8, g: u8, b: u8, w: u32, h: u32| {
        Image::rgba(w, h, std::iter::repeat_n([r, g, b, 255], (w * h) as usize).flatten().collect()).unwrap()
    };

    static TEXT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COMPOUND_ANIMATION_LIST, anchor: "start:0|s0:100,s1:150" },
        RelativeRule { kind: RULE_COMPOUND_ANIMATION_LIST, anchor: "top:1|t0:80,t1:80" },
    ];
    static TEXT_NODE: Node = Node {
        kind: Kind::Text,
        id: "tv_anim",
        text: "Hello",
        width: Size::Dp(160.),
        height: Size::Dp(60.),
        relative_rules: TEXT_RULES,
        ..button("tv_anim", Visibility::Visible, Size::Dp(60.), 0.)
    };
    static EDIT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COMPOUND_ANIMATION_LIST, anchor: "start:0|s0:100,s1:150" },
    ];
    static EDIT_NODE: Node = Node {
        kind: Kind::Edit,
        id: "et_anim",
        text: "Typed",
        width: Size::Dp(160.),
        height: Size::Dp(48.),
        relative_rules: EDIT_RULES,
        ..button("et_anim", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static BTN_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COMPOUND_ANIMATION_LIST, anchor: "end:0|s0:100,s1:150" },
    ];
    static BTN_NODE: Node = Node {
        kind: Kind::Button,
        id: "btn_anim",
        text: "Go",
        width: Size::Dp(120.),
        height: Size::Dp(48.),
        relative_rules: BTN_RULES,
        ..button("btn_anim", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static CB_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COMPOUND_ANIMATION_LIST, anchor: "button:0|b0:100,b1:100" },
    ];
    static CB_NODE: Node = Node {
        kind: Kind::CheckBox,
        id: "cb_anim",
        text: "Agree",
        width: Size::Dp(160.),
        height: Size::Dp(48.),
        relative_rules: CB_RULES,
        ..button("cb_anim", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static RB_NODE: Node = Node {
        kind: Kind::RadioButton,
        id: "rb_anim",
        text: "Pick",
        width: Size::Dp(160.),
        height: Size::Dp(48.),
        relative_rules: CB_RULES,
        ..button("rb_anim", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static TICK_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COMPOUND_ANIMATION_LIST, anchor: "tick:1|k0:100,k1:100" },
    ];
    static SEEK_NODE: Node = Node {
        kind: Kind::SeekBar,
        id: "sb_anim",
        min: 0.,
        max: 4.,
        progress: 2.,
        width: Size::Dp(200.),
        height: Size::Dp(48.),
        relative_rules: TICK_RULES,
        ..button("sb_anim", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static IND_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COMPOUND_ANIMATION_LIST, anchor: "indicator:0|i0:100,i1:120" },
    ];
    static ELV_NODE: Node = Node {
        kind: Kind::ExpandableListView,
        id: "elv_anim",
        width: Size::Dp(220.),
        height: Size::Dp(120.),
        relative_rules: IND_RULES,
        ..button("elv_anim", Visibility::Visible, Size::Dp(120.), 0.)
    };

    let mut state = LayoutState::default();
    let (s0, s1) = (solid(200, 10, 10, 16, 16), solid(10, 200, 10, 16, 16));
    let (t0, t1) = (solid(10, 10, 200, 12, 12), solid(200, 200, 10, 12, 12));
    let (b0, b1) = (solid(180, 20, 180, 20, 20), solid(20, 180, 180, 20, 20));
    let (k0, k1) = (solid(240, 120, 0, 6, 10), solid(0, 120, 240, 6, 10));
    let (i0, i1) = (solid(100, 220, 100, 18, 18), solid(220, 100, 100, 18, 18));
    let (id_s0, id_s1) = (s0.id(), s1.id());
    let (id_t0, id_t1) = (t0.id(), t1.id());
    let (id_b0, id_b1) = (b0.id(), b1.id());
    let (id_k0, id_k1) = (k0.id(), k1.id());
    let (id_i0, id_i1) = (i0.id(), i1.id());
    state.images.insert("s0".into(), s0);
    state.images.insert("s1".into(), s1);
    state.images.insert("t0".into(), t0);
    state.images.insert("t1".into(), t1);
    state.images.insert("b0".into(), b0);
    state.images.insert("b1".into(), b1);
    state.images.insert("k0".into(), k0);
    state.images.insert("k1".into(), k1);
    state.images.insert("i0".into(), i0);
    state.images.insert("i1".into(), i1);
    let mut elv = ExpandableListState::default();
    elv.set_groups(vec![ExpandableGroup {
        id: 1,
        label: "Group 1".into(),
        children: vec![ExpandableChild { id: 11, label: "Child 1".into(), selectable: true }],
    }])
    .unwrap();
    state.expandable_lists.insert("elv_anim".into(), elv);

    let img_cmds = |cmds: &[DrawCommand]| -> Vec<(u32, Rect)> {
        cmds.iter()
            .filter_map(|c| match c {
                DrawCommand::Image { image, bounds, .. } => Some((image.id(), *bounds)),
                _ => None,
            })
            .collect()
    };

    let b = Rect { x: 0., y: 0., width: 220., height: 120. };
    let mut ui = Ui::default();

    // 1. TextView start (loop) + top (oneshot): LTR -> start is on physical left (x == 0)
    let c0 = ui.frame(Input { time: 0.05, ..Input::default() }, b, 0., |f| { TEXT_NODE.render(f, Rect { width: 160., height: 60., ..b }, &mut state); });
    let imgs0 = img_cmds(&c0);
    assert_eq!(imgs0.len(), 2);
    assert_eq!(imgs0[0].0, id_s0);
    assert!((imgs0[0].1.x - 0.0).abs() < 0.1, "LTR start compound at left edge: {:?}", imgs0[0].1);
    assert_eq!(imgs0[1].0, id_t0);
    assert!((ui.next_wakeup().unwrap() - 0.08).abs() < 1e-3);

    // At t=0.12: start is on s1 (100..250ms), top is on t1 (80..160ms, oneshot last frame)
    let c1 = ui.frame(Input { time: 0.12, ..Input::default() }, b, 0., |f| { TEXT_NODE.render(f, Rect { width: 160., height: 60., ..b }, &mut state); });
    let imgs1 = img_cmds(&c1);
    assert_eq!(imgs1[0].0, id_s1);
    assert_eq!(imgs1[1].0, id_t1);
    assert!((ui.next_wakeup().unwrap() - 0.25).abs() < 1e-3);

    // At t=0.26 in RTL: start loops back to s0 AND mirrors to physical right (i=2, painted after top i=1); top stays clamped at t1
    state.rtl = true;
    let c2 = ui.frame(Input { time: 0.26, ..Input::default() }, b, 0., |f| { TEXT_NODE.render(f, Rect { width: 160., height: 60., ..b }, &mut state); });
    let imgs2 = img_cmds(&c2);
    assert_eq!(imgs2.len(), 2);
    assert_eq!(imgs2[0].0, id_t1, "top compound (i=1) stays clamped at t1");
    assert_eq!(imgs2[1].0, id_s0, "RTL start compound (i=2) loops back to s0");
    assert!((imgs2[1].1.x + imgs2[1].1.width - 160.0).abs() < 0.1, "RTL start compound at right edge: {:?}", imgs2[1].1);

    // EditText & Button animated compound drawables (monotonic t=0.37 -> 370%250 = 120ms -> s1)
    let ce = ui.frame(Input { time: 0.37, ..Input::default() }, b, 0., |f| { EDIT_NODE.render(f, Rect { width: 160., height: 48., ..b }, &mut state); });
    assert_eq!(img_cmds(&ce).first().map(|x| x.0), Some(id_s1));
    assert!((img_cmds(&ce)[0].1.x + img_cmds(&ce)[0].1.width - 160.0).abs() < 0.1, "RTL EditText start drawable on right");
    let cb = ui.frame(Input { time: 0.37, ..Input::default() }, b, 0., |f| { BTN_NODE.render(f, Rect { width: 120., height: 48., ..b }, &mut state); });
    assert_eq!(img_cmds(&cb).first().map(|x| x.0), Some(id_s1));
    assert!((img_cmds(&cb)[0].1.x - 0.0).abs() < 0.5, "RTL Button end drawable on left: {:?}", img_cmds(&cb)[0].1);
    state.rtl = false;

    // 2. CheckBox & RadioButton animated button drawable (both Bhoo and Material3; period 200ms)
    state.theme_style = AndroidTheme::Material3Light;
    let ccb0 = ui.frame(Input { time: 0.45, ..Input::default() }, b, 0., |f| { CB_NODE.render(f, Rect { width: 160., height: 48., ..b }, &mut state); });
    assert_eq!(img_cmds(&ccb0).iter().map(|x| x.0).collect::<Vec<_>>(), vec![id_b0]);
    assert!((ui.next_wakeup().unwrap() - 0.50).abs() < 1e-3);
    let ccb1 = ui.frame(Input { time: 0.52, ..Input::default() }, b, 0., |f| { CB_NODE.render(f, Rect { width: 160., height: 48., ..b }, &mut state); });
    assert_eq!(img_cmds(&ccb1).iter().map(|x| x.0).collect::<Vec<_>>(), vec![id_b1]);

    state.theme_style = AndroidTheme::Bhoo;
    let crb1 = ui.frame(Input { time: 0.52, ..Input::default() }, b, 0., |f| { RB_NODE.render(f, Rect { width: 160., height: 48., ..b }, &mut state); });
    assert_eq!(img_cmds(&crb1).iter().map(|x| x.0).collect::<Vec<_>>(), vec![id_b1]);

    // 3. SeekBar animated tick marks (fresh Ui so oneshot starts at t=0.05; min=0..max=4 -> 5 ticks)
    let mut ui_sb = Ui::default();
    let csb0 = ui_sb.frame(Input { time: 0.05, ..Input::default() }, b, 0., |f| { SEEK_NODE.render(f, Rect { width: 200., height: 48., ..b }, &mut state); });
    assert_eq!(img_cmds(&csb0).iter().map(|x| x.0).collect::<Vec<_>>(), vec![id_k0; 5]);
    assert!((ui_sb.next_wakeup().unwrap() - 0.10).abs() < 1e-3);
    let csb1 = ui_sb.frame(Input { time: 0.15, ..Input::default() }, b, 0., |f| { SEEK_NODE.render(f, Rect { width: 200., height: 48., ..b }, &mut state); });
    assert_eq!(img_cmds(&csb1).iter().map(|x| x.0).collect::<Vec<_>>(), vec![id_k1; 5]);
    assert!(ui_sb.next_wakeup().is_none(), "oneshot tick animation settled on last frame");

    // 4. ExpandableListView animated group indicator (LTR & RTL; period 220ms: 250ms->30ms i0 wake 320ms, 350ms->130ms i1)
    let celv0 = ui_sb.frame(Input { time: 0.25, ..Input::default() }, b, 0., |f| { ELV_NODE.render(f, b, &mut state); });
    let e0 = img_cmds(&celv0);
    assert_eq!(e0.len(), 1);
    assert_eq!(e0[0].0, id_i0);
    assert!(e0[0].1.x < 30.0, "LTR group indicator on left: {:?}", e0[0].1);
    assert!((ui_sb.next_wakeup().unwrap() - 0.32).abs() < 1e-3);
    state.rtl = true;
    let celv1 = ui_sb.frame(Input { time: 0.35, ..Input::default() }, b, 0., |f| { ELV_NODE.render(f, b, &mut state); });
    let e1 = img_cmds(&celv1);
    assert_eq!(e1[0].0, id_i1);
    assert!(e1[0].1.x > 180.0, "RTL group indicator on right: {:?}", e1[0].1);
    state.rtl = false;

    // 5. Off-screen clip suppresses wakeup
    let clip_off = Rect { x: 500., y: 500., width: 160., height: 60. };
    ui_sb.frame(Input { time: 0.40, ..Input::default() }, b, 0., |f| { TEXT_NODE.render(f, clip_off, &mut state); });
    assert!(ui_sb.next_wakeup().is_none(), "off-screen animated compound must not schedule wake_at");

    // 6. Reduced motion freezes on frame 0 with no wakeup
    ui_sb.motion_preference = MotionPreference::Reduced;
    let crm = ui_sb.frame(Input { time: 0.45, ..Input::default() }, b, 0., |f| { TEXT_NODE.render(f, Rect { width: 160., height: 60., ..b }, &mut state); });
    assert_eq!(img_cmds(&crm).iter().map(|x| x.0).collect::<Vec<_>>(), vec![id_s0, id_t0]);
    assert!(ui_sb.next_wakeup().is_none());
    assert!(!ui_sb.needs_redraw());
}

#[test]
fn wave21a_text_layout_ext_autosize_balanced_hyphen_justify_rtl() {
    use bhoo::android_xml::RULE_TEXT_LAYOUT_EXT;

    let text_cmds = |cmds: &[DrawCommand]| -> Vec<(String, Rect, TextStyle)> {
        cmds.iter()
            .filter_map(|c| match c {
                DrawCommand::Text { text, bounds, style, .. } => {
                    Some((text.as_ref().to_owned(), *bounds, *style))
                }
                _ => None,
            })
            .collect()
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();

    // 1. Uniform auto-sizing on TextView & Button
    static AUTO_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LAYOUT_EXT, anchor: "simple|none|none|10,28,2" },
    ];
    static AUTO_SHORT: Node = Node {
        kind: Kind::Text,
        id: "auto_s",
        text: "OK",
        width: Size::Dp(120.),
        height: Size::Dp(48.),
        max_lines: 2,
        relative_rules: AUTO_RULES,
        ..button("auto_s", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static AUTO_LONG: Node = Node {
        kind: Kind::Text,
        id: "auto_l",
        text: "Uniform auto sizing scales down cleanly to fit inside box",
        width: Size::Dp(120.),
        height: Size::Dp(48.),
        max_lines: 2,
        relative_rules: AUTO_RULES,
        ..button("auto_l", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static AUTO_BTN: Node = Node {
        kind: Kind::Button,
        id: "auto_btn",
        text: "Supercalifragilistic",
        width: Size::Dp(120.),
        height: Size::Dp(48.),
        max_lines: 1,
        relative_rules: AUTO_RULES,
        ..button("auto_btn", Visibility::Visible, Size::Dp(48.), 0.)
    };
    let box_auto = Rect { x: 0., y: 0., width: 120., height: 48. };
    let cs = ui.frame(Input::default(), box_auto, 0., |f| { AUTO_SHORT.render(f, box_auto, &mut state); });
    let ts = text_cmds(&cs);
    assert_eq!(ts.len(), 1);
    assert!((ts[0].2.size - 28.0).abs() < 0.01, "short text uses auto_max 28sp: {:?}", ts[0].2.size);

    let cl = ui.frame(Input::default(), box_auto, 0., |f| { AUTO_LONG.render(f, box_auto, &mut state); });
    let tl = text_cmds(&cl);
    assert!(tl.len() <= 2);
    let sz_l = tl[0].2.size;
    assert!(sz_l >= 10.0 && sz_l < 28.0, "long text shrinks within [10..28): {sz_l}");
    assert!(((sz_l - 10.0) % 2.0).abs() < 0.01, "quantized to step 2sp: {sz_l}");

    let cb = ui.frame(Input::default(), box_auto, 0., |f| { AUTO_BTN.render(f, box_auto, &mut state); });
    let tb = text_cmds(&cb);
    assert_eq!(tb.len(), 1);
    assert!(tb[0].2.size >= 10.0 && tb[0].2.size <= 16.0, "Button long single word auto-sizes down: {}", tb[0].2.size);

    // 2. Balanced vs Simple vs HighQuality line breaking
    static SIMPLE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LAYOUT_EXT, anchor: "simple|none|none|" },
    ];
    static BALANCED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LAYOUT_EXT, anchor: "balanced|none|none|" },
    ];
    static HQ_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LAYOUT_EXT, anchor: "high_quality|none|none|" },
    ];
    // 4 equal-length words in a box wide enough for 3 words but not 4
    static TV_SIMPLE_BAL: Node = Node {
        kind: Kind::Text,
        id: "tv_sb",
        text: "Alpha Beta Gamma Delta",
        text_size: 14.,
        width: Size::Dp(135.),
        height: Size::Dp(100.),
        relative_rules: SIMPLE_RULES,
        ..button("tv_sb", Visibility::Visible, Size::Dp(100.), 0.)
    };
    static TV_BALANCED: Node = Node {
        relative_rules: BALANCED_RULES,
        ..TV_SIMPLE_BAL
    };
    let box_bal = Rect { x: 0., y: 0., width: 135., height: 100. };
    let sim_lines: Vec<String> = text_cmds(&ui.frame(Input::default(), box_bal, 0., |f| { TV_SIMPLE_BAL.render(f, box_bal, &mut state); }))
        .into_iter().map(|x| x.0).collect();
    let bal_lines: Vec<String> = text_cmds(&ui.frame(Input::default(), box_bal, 0., |f| { TV_BALANCED.render(f, box_bal, &mut state); }))
        .into_iter().map(|x| x.0).collect();
    assert_eq!(sim_lines, vec!["Alpha Beta Gamma", "Delta"], "simple packs greedily 3+1");
    assert_eq!(bal_lines, vec!["Alpha Beta", "Gamma Delta"], "balanced optimizes squared slack to 2+2");

    // HighQuality optimizes non-last lines while excluding last line slack:
    // Greedy packs "AAAAA BBBBB cc" (tiny slack) + "DDDDDDD" (huge slack) + "EEEEEEEE" (last);
    // HighQuality pulls "cc" to line 2 ("AAAAA BBBBB" + "cc DDDDDDD" + "EEEEEEEE") for strictly lower non-last squared slack!
    static TV_HQ_GREEDY: Node = Node {
        kind: Kind::Text,
        id: "tv_hq_g",
        text: "AAAAA BBBBB cc DDDDDDD EEEEEEEE",
        text_size: 14.,
        width: Size::Dp(118.),
        height: Size::Dp(120.),
        relative_rules: SIMPLE_RULES,
        ..button("tv_hq_g", Visibility::Visible, Size::Dp(120.), 0.)
    };
    static TV_HQ: Node = Node {
        relative_rules: HQ_RULES,
        ..TV_HQ_GREEDY
    };
    let box_hq = Rect { x: 0., y: 0., width: 118., height: 120. };
    let hqg_lines: Vec<String> = text_cmds(&ui.frame(Input::default(), box_hq, 0., |f| { TV_HQ_GREEDY.render(f, box_hq, &mut state); }))
        .into_iter().map(|x| x.0).collect();
    let hq_lines: Vec<String> = text_cmds(&ui.frame(Input::default(), box_hq, 0., |f| { TV_HQ.render(f, box_hq, &mut state); }))
        .into_iter().map(|x| x.0).collect();
    assert_eq!(hqg_lines, vec!["AAAAA BBBBB cc", "DDDDDDD", "EEEEEEEE"]);
    assert_eq!(hq_lines, vec!["AAAAA BBBBB", "cc DDDDDDD", "EEEEEEEE"]);

    // 3. Soft hyphenation prevents long word overflow with trailing '-'
    static HYPHEN_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LAYOUT_EXT, anchor: "simple|normal|none|" },
    ];
    static TV_HYPHEN: Node = Node {
        kind: Kind::Text,
        id: "tv_hyp",
        text: "Internationalization",
        text_size: 14.,
        width: Size::Dp(72.),
        height: Size::Dp(120.),
        relative_rules: HYPHEN_RULES,
        ..button("tv_hyp", Visibility::Visible, Size::Dp(120.), 0.)
    };
    let box_hyp = Rect { x: 0., y: 0., width: 72., height: 120. };
    let hyp_cmds = text_cmds(&ui.frame(Input::default(), box_hyp, 0., |f| { TV_HYPHEN.render(f, box_hyp, &mut state); }));
    assert!(hyp_cmds.len() >= 2, "oversized word splits across multiple lines: {hyp_cmds:?}");
    for (idx, (line, _, style)) in hyp_cmds.iter().enumerate() {
        let w = ui.text_metrics.width(line, *style);
        assert!(w <= 72.01, "hyphenated piece {line:?} must fit 72px (got {w})");
        if idx + 1 < hyp_cmds.len() {
            assert!(line.ends_with('-'), "non-final split piece must end with '-': {line:?}");
        }
    }
    let rejoined: String = hyp_cmds.iter().enumerate().map(|(i, (s, _, _))| {
        if i + 1 < hyp_cmds.len() { s.strip_suffix('-').unwrap_or(s) } else { s.as_str() }
    }).collect();
    assert_eq!(rejoined, "Internationalization");

    // 4. Inter-word justification (LTR & RTL): non-final line spreads words edge-to-edge; final line stays un-stretched
    static JUST_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LAYOUT_EXT, anchor: "simple|none|inter_word|" },
    ];
    static TV_JUST: Node = Node {
        kind: Kind::Text,
        id: "tv_just",
        text: "one two three fourfive",
        text_size: 14.,
        width: Size::Dp(140.),
        height: Size::Dp(100.),
        relative_rules: JUST_RULES,
        ..button("tv_just", Visibility::Visible, Size::Dp(100.), 0.)
    };
    let box_j = Rect { x: 10., y: 0., width: 140., height: 100. };
    let j_ltr = text_cmds(&ui.frame(Input::default(), Rect { x: 0., y: 0., width: 200., height: 100. }, 0., |f| { TV_JUST.render(f, box_j, &mut state); }));
    // Line 1: ["one", "two", "three"] painted as 3 justified word draws; Line 2: ["fourfive"] painted as 1 draw
    let line1_ltr: Vec<_> = j_ltr.iter().filter(|x| (x.1.y - j_ltr[0].1.y).abs() < 0.5).collect();
    let line2_ltr: Vec<_> = j_ltr.iter().filter(|x| (x.1.y - j_ltr[0].1.y).abs() >= 0.5).collect();
    assert_eq!(line1_ltr.len(), 3, "non-final line emits per-word justified draws: {line1_ltr:?}");
    assert_eq!(line2_ltr.len(), 1, "final line is not justified: {line2_ltr:?}");
    assert!((line1_ltr[0].1.x - 10.0).abs() < 0.1, "LTR first word flush left");
    let last_r = line1_ltr[2].1.x + line1_ltr[2].1.width;
    assert!((last_r - 150.0).abs() < 0.1, "LTR last word flush right: {last_r}");
    let gap0 = line1_ltr[1].1.x - (line1_ltr[0].1.x + line1_ltr[0].1.width);
    let gap1 = line1_ltr[2].1.x - (line1_ltr[1].1.x + line1_ltr[1].1.width);
    assert!((gap0 - gap1).abs() < 0.1 && gap0 > 10.0, "LTR inter-word gaps equal and expanded: {gap0} vs {gap1}");

    // RTL justification: word 0 ("one") flush right at x+w=150, word 2 ("three") flush left at x=10
    state.rtl = true;
    let j_rtl = text_cmds(&ui.frame(Input::default(), Rect { x: 0., y: 0., width: 200., height: 100. }, 0., |f| { TV_JUST.render(f, box_j, &mut state); }));
    let line1_rtl: Vec<_> = j_rtl.iter().filter(|x| (x.1.y - j_rtl[0].1.y).abs() < 0.5).collect();
    assert_eq!(line1_rtl.len(), 3);
    assert!((line1_rtl[0].1.x + line1_rtl[0].1.width - 150.0).abs() < 0.1, "RTL first word flush right: {:?}", line1_rtl[0].1);
    assert!((line1_rtl[2].1.x - 10.0).abs() < 0.1, "RTL last word flush left: {:?}", line1_rtl[2].1);
    assert!(line1_rtl[0].1.x > line1_rtl[1].1.x && line1_rtl[1].1.x > line1_rtl[2].1.x, "RTL words flow right-to-left");
}

#[test]
fn wave21a_material_badge_ext_truncation_gravity_offsets_and_rtl() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_BOTTOM_APP_BAR, RULE_MATERIAL_BADGE_EXT, RULE_NAV_BADGE, RULE_NAV_BAR,
        RULE_NAV_ITEM,
    };

    let find_text = |cmds: &[DrawCommand], label: &str| -> Option<Rect> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::Text { text, bounds, .. } if text.as_ref() == label => Some(*bounds),
            _ => None,
        })
    };

    // 1. NavigationBar: truncation (4->999+, 3->99+, 2->9+) + gravities + offsets + RTL mirror
    static NAV_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "bar|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "home|Home|home" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "mail|Mail|mail" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "chat|Chat|chat" },
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "home:1250" },
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "mail:105" },
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "chat:12" },
        RelativeRule { kind: RULE_MATERIAL_BADGE_EXT, anchor: "0|top_end|4|6,4" },
        RelativeRule { kind: RULE_MATERIAL_BADGE_EXT, anchor: "1|bottom_start|3|-2,3" },
        RelativeRule { kind: RULE_MATERIAL_BADGE_EXT, anchor: "2|top_start|2|0,0" },
    ];
    static NAV_BAR: Node = Node {
        kind: Kind::Linear,
        id: "nav_bar_badge",
        width: Size::Dp(300.),
        height: Size::Dp(80.),
        relative_rules: NAV_RULES,
        ..button("nav_bar_badge", Visibility::Visible, Size::Dp(80.), 0.)
    };

    // 2. NavigationDrawer, TabWidget, BottomAppBar, Standalone View (-1 / self)
    static DRAWER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BAR, anchor: "drawer|labeled" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "inbox|Inbox|mail" },
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "inbox:1000" },
        RelativeRule { kind: RULE_MATERIAL_BADGE_EXT, anchor: "0|bottom_end|4|2,-2" },
    ];
    static DRAWER: Node = Node {
        kind: Kind::Linear,
        id: "drawer_badge",
        width: Size::Dp(240.),
        height: Size::Dp(80.),
        relative_rules: DRAWER_RULES,
        ..button("drawer_badge", Visibility::Visible, Size::Dp(80.), 0.)
    };
    static TAB_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "0:450" },
        RelativeRule { kind: RULE_MATERIAL_BADGE_EXT, anchor: "0|top_end|3|0,2" },
    ];
    static TABS: Node = Node {
        kind: Kind::TabWidget,
        id: "tab_badge",
        width: Size::Dp(200.),
        height: Size::Dp(48.),
        relative_rules: TAB_RULES,
        children: &[
            Node { kind: Kind::Text, id: "t0", text: "One", ..button("t0", Visibility::Visible, Size::Match, 0.) },
            Node { kind: Kind::Text, id: "t1", text: "Two", ..button("t1", Visibility::Visible, Size::Match, 0.) },
        ],
        ..button("tab_badge", Visibility::Visible, Size::Dp(48.), 0.)
    };
    static BAB_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BOTTOM_APP_BAR, anchor: "end" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "act0|Search|search" },
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "act0:25" },
        RelativeRule { kind: RULE_MATERIAL_BADGE_EXT, anchor: "0|top_end|2|0,0" },
    ];
    static BAB: Node = Node {
        kind: Kind::Toolbar,
        id: "bab_badge",
        width: Size::Dp(240.),
        height: Size::Dp(64.),
        relative_rules: BAB_RULES,
        ..button("bab_badge", Visibility::Visible, Size::Dp(64.), 0.)
    };
    static SELF_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_NAV_BADGE, anchor: "self:5000" },
        RelativeRule { kind: RULE_MATERIAL_BADGE_EXT, anchor: "-1|bottom_end|4|4,-2" },
    ];
    static SELF_BTN: Node = Node {
        kind: Kind::Button,
        id: "solo_btn",
        text: "Inbox",
        width: Size::Dp(120.),
        height: Size::Dp(48.),
        relative_rules: SELF_RULES,
        ..button("solo_btn", Visibility::Visible, Size::Dp(48.), 0.)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    let nb = Rect { x: 0., y: 0., width: 300., height: 80. };
    let c_ltr = ui.frame(Input::default(), nb, 0., |f| { NAV_BAR.render(f, nb, &mut state); });
    let r999_ltr = find_text(&c_ltr, "999+").expect("max_chars=4 truncates 1250 to 999+");
    let r99_ltr = find_text(&c_ltr, "99+").expect("max_chars=3 truncates 105 to 99+");
    let r9_ltr = find_text(&c_ltr, "9+").expect("max_chars=2 truncates 12 to 9+");
    // Item 0 center in LTR is x=50; top_end + hoff=6 shifts well right of center
    assert!(r999_ltr.x > 50.0, "LTR top_end badge sits right of item 0 center: {r999_ltr:?}");
    // Item 1 (bottom_start) must sit lower than item 0 (top_end) and left of item 1 center (x=150)
    assert!(r99_ltr.y > r999_ltr.y + 8.0, "bottom_start sits below top_end: {r99_ltr:?} vs {r999_ltr:?}");
    assert!(r99_ltr.x + r99_ltr.width < 150.0, "LTR bottom_start sits left of item 1 center: {r99_ltr:?}");
    // Item 2 (top_start) sits left of item 2 center (x=250)
    assert!(r9_ltr.x + r9_ltr.width <= 250.5, "LTR top_start sits left of item 2 center: {r9_ltr:?}");

    // RTL: item 0 moves to right slot (center x=250), top_end mirrors to physical left of center AND hoff=+6 mirrors to -6!
    state.rtl = true;
    let c_rtl = ui.frame(Input::default(), nb, 0., |f| { NAV_BAR.render(f, nb, &mut state); });
    let r999_rtl = find_text(&c_rtl, "999+").unwrap();
    let r99_rtl = find_text(&c_rtl, "99+").unwrap();
    let r9_rtl = find_text(&c_rtl, "9+").unwrap();
    // Exact horizontal mirror symmetry around bar width 300: r_ltr.center + r_rtl.center == 300!
    let c_ltr_0 = r999_ltr.x + r999_ltr.width * 0.5;
    let c_rtl_0 = r999_rtl.x + r999_rtl.width * 0.5;
    assert!((c_ltr_0 + c_rtl_0 - 300.0).abs() < 0.2, "RTL top_end + hoff mirrors symmetrically: {c_ltr_0} + {c_rtl_0}");
    let c_ltr_1 = r99_ltr.x + r99_ltr.width * 0.5;
    let c_rtl_1 = r99_rtl.x + r99_rtl.width * 0.5;
    assert!((c_ltr_1 + c_rtl_1 - 300.0).abs() < 0.2, "RTL bottom_start + hoff mirrors symmetrically: {c_ltr_1} + {c_rtl_1}");
    assert!(r9_rtl.x >= 49.5, "RTL top_start on item 2 (slot center x=50) mirrors to physical right: {r9_rtl:?}");
    state.rtl = false;

    // NavigationDrawer badge
    let db = Rect { x: 0., y: 0., width: 240., height: 80. };
    let cd = ui.frame(Input::default(), db, 0., |f| { DRAWER.render(f, db, &mut state); });
    assert!(find_text(&cd, "999+").is_some(), "NavigationDrawer renders truncated badge");

    // TabWidget badge
    let tb = Rect { x: 0., y: 0., width: 200., height: 48. };
    let ct = ui.frame(Input::default(), tb, 0., |f| { TABS.render(f, tb, &mut state); });
    assert!(find_text(&ct, "99+").is_some(), "TabWidget renders truncated badge");

    // BottomAppBar badge
    let bb = Rect { x: 0., y: 0., width: 240., height: 64. };
    let cb = ui.frame(Input::default(), bb, 0., |f| { BAB.render(f, bb, &mut state); });
    assert!(find_text(&cb, "9+").is_some(), "BottomAppBar renders truncated badge");

    // Standalone view badge (item_index = -1 / "self", bottom_end, LTR & RTL)
    let sb = Rect { x: 20., y: 20., width: 120., height: 48. };
    let c_solo_ltr = ui.frame(Input::default(), Rect { x: 0., y: 0., width: 200., height: 100. }, 0., |f| { SELF_BTN.render(f, sb, &mut state); });
    let solo_ltr = find_text(&c_solo_ltr, "999+").expect("standalone badge truncates 5000 -> 999+");
    assert!(solo_ltr.x > sb.x + sb.width * 0.5, "LTR standalone bottom_end on right half: {solo_ltr:?}");
    assert!(solo_ltr.y > sb.y + sb.height * 0.5, "standalone bottom_end on bottom half: {solo_ltr:?}");

    state.rtl = true;
    let c_solo_rtl = ui.frame(Input::default(), Rect { x: 0., y: 0., width: 200., height: 100. }, 0., |f| { SELF_BTN.render(f, sb, &mut state); });
    let solo_rtl = find_text(&c_solo_rtl, "999+").unwrap();
    assert!(solo_rtl.x + solo_rtl.width < sb.x + sb.width * 0.5, "RTL standalone bottom_end mirrors to left half: {solo_rtl:?}");
}

#[test]
fn wave22a_collapsing_toolbar_ext_pin_parallax_fade_multiline_subtitle_and_scrim_motion() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_COLLAPSING_TOOLBAR, RULE_COLLAPSING_TOOLBAR_EXT,
        RULE_COORDINATOR_SCROLL_BEHAVIOR,
    };

    static CT_FADE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COLLAPSING_TOOLBAR, anchor: "Explore Beautiful Mountain Trails|164|10,20,30,255" },
        RelativeRule { kind: RULE_COLLAPSING_TOOLBAR_EXT, anchor: "fade|2|300|Updated today" },
    ];
    static PIN_CHILD_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COLLAPSING_TOOLBAR_EXT, anchor: "collapse:pin:0" },
    ];
    static PARALLAX_CHILD_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COLLAPSING_TOOLBAR_EXT, anchor: "collapse:parallax:0.5" },
    ];
    static SCROLL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COORDINATOR_SCROLL_BEHAVIOR, anchor: "" },
    ];

    static COORD: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Frame,
                width: Size::Match,
                height: Size::Dp(164.),
                relative_rules: CT_FADE_RULES,
                children: &[
                    Node {
                        kind: Kind::View,
                        width: Size::Match,
                        height: Size::Match,
                        background: Some(Color(40, 120, 200, 255)),
                        relative_rules: PARALLAX_CHILD_RULES,
                        ..button("hero_parallax", Visibility::Visible, Size::Match, 0.)
                    },
                    Node {
                        kind: Kind::View,
                        width: Size::Dp(40.),
                        height: Size::Dp(20.),
                        margin: [12., 8., 0., 0.],
                        background: Some(Color(220, 80, 40, 255)),
                        relative_rules: PIN_CHILD_RULES,
                        ..button("pinned_badge", Visibility::Visible, Size::Dp(20.), 0.)
                    },
                    Node {
                        kind: Kind::Toolbar,
                        image_src: "arrow_back",
                        ..button("ct22_tb", Visibility::Visible, Size::Dp(64.), 0.)
                    },
                ],
                ..button("ct22", Visibility::Visible, Size::Dp(164.), 0.)
            },
            Node {
                kind: Kind::Scroll,
                relative_rules: SCROLL_RULES,
                children: &[Node {
                    kind: Kind::View,
                    ..button("ct22_body", Visibility::Visible, Size::Dp(800.), 0.)
                }],
                ..button("ct22_scroll", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("ct22_root", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 220., height: 400. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Expanded (scroll = 0, collapse_range = 100px):
    //    - multi-line title (max_lines=2) wraps onto 2 lines at 28sp
    //    - subtitle "Updated today" renders beneath expanded title with full alpha
    //    - parallax child at y=0, pinned child at y=8
    //    - scrim (Color(10,20,30,_)) is absent because p=0 < 0.72
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| {
        COORD.render(f, b, &mut state);
    });
    let exp_lines: Vec<_> = c0.iter().filter_map(|c| match c {
        DrawCommand::Text { text, bounds, color, style }
            if text.as_ref() != "Updated today" && !text.is_empty() =>
        {
            Some((text.to_string(), *bounds, *color, *style))
        }
        _ => None,
    }).collect();
    assert_eq!(exp_lines.len(), 2, "max_lines=2 wraps long expanded title into 2 lines: {exp_lines:?}");
    assert!((exp_lines[0].3.size - 28.0).abs() < 0.05);
    assert_eq!(exp_lines[0].2.3, 255);
    let sub0 = c0.iter().find_map(|c| match c {
        DrawCommand::Text { text, bounds, color, style } if text.as_ref() == "Updated today" => {
            Some((*bounds, *color, *style))
        }
        _ => None,
    }).expect("expanded subtitle must render");
    assert!(sub0.0.y > exp_lines[1].1.y, "subtitle must sit beneath expanded title");
    assert_eq!(sub0.1.3, 255);
    assert!(!c0.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(10, 20, 30, a), .. } if *a > 0)));
    let pin_y0 = c0.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds, color: Color(220, 80, 40, 255), .. } => Some(bounds.y),
        _ => None,
    }).unwrap();
    let par_y0 = c0.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds, color: Color(40, 120, 200, 255), .. } => Some(bounds.y),
        _ => None,
    }).unwrap();
    assert!((pin_y0 - 8.0).abs() < 0.05);
    assert!((par_y0 - 0.0).abs() < 0.05);

    // 2. Scroll to p = 0.25 (25px of 100px collapse_range):
    //    - parallax child shifts by -25 * 0.5 = -12.5 and is clipped cleanly to bar_rect (0..139)
    //    - pinned child stays pinned at y = 8.0
    //    - fade mode: expanded title stays 28sp with alpha ~0.5 (128), subtitle fades to (1 - 0.4) = 0.6 (~153)
    state.offsets.insert("ct22_scroll".into(), 25.0);
    let c25 = ui.frame(Input { time: 1.05, ..Input::default() }, b, 0., |f| {
        COORD.render(f, b, &mut state);
    });
    let pin_y25 = c25.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds, color: Color(220, 80, 40, 255), .. } => Some(bounds.y),
        _ => None,
    }).unwrap();
    let par_r25 = c25.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds, color: Color(40, 120, 200, 255), .. } => Some(*bounds),
        _ => None,
    }).unwrap();
    assert!((pin_y25 - 8.0).abs() < 0.05, "collapse:pin keeps child pinned at y=8, got {pin_y25}");
    assert!((par_r25.y - (-12.5)).abs() < 0.1, "collapse:parallax:0.5 shifts child by -12.5, got {}", par_r25.y);
    assert!(
        c25.iter().any(|c| matches!(c, DrawCommand::PushClip(clip) if (clip.y - 0.0).abs() < 0.1 && (clip.height - 139.0).abs() < 0.2)),
        "parallax child must be clipped cleanly inside shrinking CollapsingToolbarLayout bounds"
    );
    let fade_exp = c25.iter().find_map(|c| match c {
        DrawCommand::Text { text, color, style, .. } if text.as_ref() != "Updated today" => Some((*color, *style)),
        _ => None,
    }).unwrap();
    assert!((fade_exp.1.size - 28.0).abs() < 0.05, "mode=fade keeps expanded font size without scaling");
    assert!((fade_exp.0.3 as i32 - 128).abs() <= 2, "mode=fade at p=0.25 has ~50% alpha, got {}", fade_exp.0.3);
    let sub25 = c25.iter().find_map(|c| match c {
        DrawCommand::Text { text, color, .. } if text.as_ref() == "Updated today" => Some(color.3),
        _ => None,
    }).unwrap();
    assert!((sub25 as i32 - 153).abs() <= 3, "subtitle alpha at p=0.25 should be ~0.6 (153), got {sub25}");

    // 3. Cross scrim threshold at p = 0.75 (75px >= 0.72) at t = 2.0:
    //    - fade mode: collapsed single-line title at 22sp with ~50% alpha; subtitle fully faded out
    //    - scrim animates over 300ms (mid-alpha at t=2.15, full alpha at t=2.35)
    state.offsets.insert("ct22_scroll".into(), 75.0);
    let c75_start = ui.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| {
        COORD.render(f, b, &mut state);
    });
    assert!(ui.needs_redraw(), "scrim animation must request continuous frames");
    let col_lines: Vec<_> = c75_start.iter().filter_map(|c| match c {
        DrawCommand::Text { text, color, style, .. } => Some((text.to_string(), color.3, style.size)),
        _ => None,
    }).collect();
    assert_eq!(col_lines.len(), 1, "collapsed state transitions to single-line title & hides subtitle: {col_lines:?}");
    assert!((col_lines[0].2 - 22.0).abs() < 0.05, "collapsed title uses 22sp");
    assert!((col_lines[0].1 as i32 - 128).abs() <= 2, "fade collapsed title alpha at p=0.75 is ~50%, got {}", col_lines[0].1);

    let c75_mid = ui.frame(Input { time: 2.15, ..Input::default() }, b, 0., |f| {
        COORD.render(f, b, &mut state);
    });
    let scrim_mid_a = c75_mid.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { color: Color(10, 20, 30, a), .. } => Some(*a),
        _ => None,
    }).expect("scrim should be fading in at t=2.15");
    assert!(scrim_mid_a > 20 && scrim_mid_a < 245, "scrim alpha mid-300ms must be strictly intermediate, got {scrim_mid_a}");
    assert!(ui.needs_redraw());

    let c75_done = ui.frame(Input { time: 2.35, ..Input::default() }, b, 0., |f| {
        COORD.render(f, b, &mut state);
    });
    let scrim_done_a = c75_done.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { color: Color(10, 20, 30, a), .. } => Some(*a),
        _ => None,
    }).expect("scrim must reach full opacity after 300ms");
    assert_eq!(scrim_done_a, 255);

    // 4. Reduced motion snaps scrim immediately when dropping below threshold (p = 0.0) & RTL aligns subtitle right
    ui.motion_preference = MotionPreference::Reduced;
    state.offsets.insert("ct22_scroll".into(), 0.0);
    state.rtl = true;
    let c_rm_rtl = ui.frame(Input { time: 2.5, ..Input::default() }, b, 0., |f| {
        COORD.render(f, b, &mut state);
    });
    assert!(!ui.needs_redraw(), "reduced_motion scrim snap must not request extra frames");
    assert!(
        !c_rm_rtl.iter().any(|c| matches!(c, DrawCommand::RoundedRect { color: Color(10, 20, 30, a), .. } if *a > 0)),
        "reduced_motion must snap scrim off immediately"
    );
    let sub_rtl = c_rm_rtl.iter().find_map(|c| match c {
        DrawCommand::Text { text, style, .. } if text.as_ref() == "Updated today" => Some(style.align),
        _ => None,
    }).unwrap();
    assert_eq!(sub_rtl, 2, "RTL subtitle must right-align");
}

#[test]
fn wave22a_extended_fab_ext_width_animation_strategies_state_override_rtl_and_reduced_motion() {
    use bhoo::android_xml::{AndroidTheme, RULE_EXTENDED_FAB_EXT};

    static WRAP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_EXTENDED_FAB_EXT, anchor: "1|48|wrap_content" },
    ];
    static MATCH_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_EXTENDED_FAB_EXT, anchor: "1|56|match_parent" },
    ];
    static WRAP_HOST: Node = Node {
        kind: Kind::Frame,
        children: &[Node {
            kind: Kind::Button,
            id: "efab_wrap",
            width: Size::Wrap,
            height: Size::Wrap,
            text: "Compose",
            image_src: "edit",
            layout_gravity: [2, 2],
            layout_gravity_explicit: true,
            relative_rules: WRAP_RULES,
            ..button("efab_wrap", Visibility::Visible, Size::Wrap, 0.)
        }],
        ..button("efab_wrap_host", Visibility::Visible, Size::Match, 0.)
    };
    static MATCH_HOST: Node = Node {
        kind: Kind::Frame,
        children: &[Node {
            kind: Kind::Button,
            id: "efab_match",
            width: Size::Wrap,
            height: Size::Wrap,
            text: "New Message",
            image_src: "add",
            layout_gravity: [2, 2],
            layout_gravity_explicit: true,
            relative_rules: MATCH_RULES,
            ..button("efab_match", Visibility::Visible, Size::Wrap, 0.)
        }],
        ..button("efab_match_host", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 300., height: 120. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    let fab_rect = |cmds: &[DrawCommand]| -> Rect {
        cmds.iter().find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color: Color(0xEA, 0xDD, 0xFF, 255), .. } => Some(*bounds),
            _ => None,
        }).expect("Extended FAB container must render")
    };
    let label_alpha = |cmds: &[DrawCommand], lbl: &str| -> Option<u8> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::Text { text, color, .. } if text.as_ref() == lbl => Some(color.3),
            _ => None,
        })
    };

    // 1. Wrap strategy starts extended (intrinsic pill width > 48dp, < 300dp), label alpha=255
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| {
        WRAP_HOST.render(f, b, &mut state);
    });
    let r_ext = fab_rect(&c0);
    assert!(r_ext.width > 90.0 && r_ext.width < 240.0, "wrap_content Extended FAB uses intrinsic pill width: {r_ext:?}");
    assert!((r_ext.height - 56.0).abs() < 0.1);
    assert_eq!(label_alpha(&c0, "Compose"), Some(255));

    // 2. Override via state.fab_extended -> false: smoothly shrinks over 0.22s ease-out to 48dp x 48dp and fades label
    state.fab_extended.insert("efab_wrap".into(), false);
    let _c_start = ui.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| {
        WRAP_HOST.render(f, b, &mut state);
    });
    assert!(ui.needs_redraw(), "Extended FAB transition must request continuous animation frames");

    let c_mid = ui.frame(Input { time: 2.11, ..Input::default() }, b, 0., |f| {
        WRAP_HOST.render(f, b, &mut state);
    });
    let r_mid = fab_rect(&c_mid);
    assert!(
        r_mid.width > 48.5 && r_mid.width < r_ext.width - 5.0,
        "mid-0.22s width must smoothly interpolate between 48 and {}: got {}",
        r_ext.width,
        r_mid.width
    );
    let mid_a = label_alpha(&c_mid, "Compose").expect("label still visible mid-fade");
    assert!(mid_a > 10 && mid_a < 240, "label opacity fades smoothly mid-transition, got {mid_a}");

    let c_done = ui.frame(Input { time: 2.25, ..Input::default() }, b, 0., |f| {
        WRAP_HOST.render(f, b, &mut state);
    });
    let r_col = fab_rect(&c_done);
    assert!((r_col.width - 48.0).abs() < 0.1, "collapsed width honors custom collapsed_size_dp=48: {r_col:?}");
    assert!((r_col.height - 48.0).abs() < 0.1, "collapsed height honors custom collapsed_size_dp=48: {r_col:?}");
    assert!(label_alpha(&c_done, "Compose").is_none(), "collapsed FAB omits label completely");

    // 3. Reduced motion snaps immediately back to extended when fab_extended = true
    ui.motion_preference = MotionPreference::Reduced;
    state.fab_extended.insert("efab_wrap".into(), true);
    let c_rm = ui.frame(Input { time: 2.3, ..Input::default() }, b, 0., |f| {
        WRAP_HOST.render(f, b, &mut state);
    });
    assert!(!ui.needs_redraw(), "reduced_motion FAB snap must not request extra frames");
    let r_rm = fab_rect(&c_rm);
    assert!((r_rm.width - r_ext.width).abs() < 0.1, "reduced_motion snaps width immediately: {r_rm:?}");
    assert_eq!(label_alpha(&c_rm, "Compose"), Some(255));

    // 4. match_parent strategy extends to full 300px, collapses to 56dp, and mirrors alignment in RTL
    let cm_ext = ui.frame(Input { time: 3.0, ..Input::default() }, b, 0., |f| {
        MATCH_HOST.render(f, b, &mut state);
    });
    assert!((fab_rect(&cm_ext).width - 300.0).abs() < 0.1, "match_parent extended fills parent width");

    state.fab_extended.insert("efab_match".into(), false);
    let cm_col_ltr = ui.frame(Input { time: 3.1, ..Input::default() }, b, 0., |f| {
        MATCH_HOST.render(f, b, &mut state);
    });
    let r_ltr = fab_rect(&cm_col_ltr);
    assert!((r_ltr.width - 56.0).abs() < 0.1);
    assert!((r_ltr.x - 244.0).abs() < 0.1, "LTR end-aligned collapsed FAB sits on right edge: {r_ltr:?}");

    state.rtl = true;
    let cm_col_rtl = ui.frame(Input { time: 3.2, ..Input::default() }, b, 0., |f| {
        MATCH_HOST.render(f, b, &mut state);
    });
    let r_rtl = fab_rect(&cm_col_rtl);
    assert!((r_rtl.width - 56.0).abs() < 0.1);
    assert!((r_rtl.x - 0.0).abs() < 0.1, "RTL end-aligned collapsed FAB mirrors to left edge: {r_rtl:?}");
}

#[test]
fn wave22a_bottom_app_bar_ext_cradle_embed_cutout_icons_anchor_rtl_and_hide_on_scroll() {
    use bhoo::android_xml::{
        AndroidTheme, RULE_BOTTOM_APP_BAR_EXT, RULE_COORDINATOR_ANCHOR,
        RULE_COORDINATOR_SCROLL_BEHAVIOR, RULE_NAV_ITEM,
    };

    static BAB_CRADLE_CENTER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BOTTOM_APP_BAR_EXT, anchor: "center|cradle|5,8,6|1" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "a0|Search|search" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "a1|Edit|edit" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "a2|Star|star" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "a3|Share|share" },
    ];
    static BAB_END_EMBED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_BOTTOM_APP_BAR_EXT, anchor: "end|embed|5,8,0|0" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "e0|Search|search" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "e1|Edit|edit" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "e2|Star|star" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "e3|Share|share" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "e4|Delete|delete" },
        RelativeRule { kind: RULE_NAV_ITEM, anchor: "e5|Settings|settings" },
    ];
    static SCROLL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COORDINATOR_SCROLL_BEHAVIOR, anchor: "" },
    ];
    static ANCHOR_CENTER_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COORDINATOR_ANCHOR, anchor: "bab_c" },
    ];
    static ANCHOR_END_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_COORDINATOR_ANCHOR, anchor: "bab_e" },
    ];

    static COORD_CRADLE: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Scroll,
                relative_rules: SCROLL_RULES,
                children: &[Node {
                    kind: Kind::View,
                    ..button("bab_feed_body", Visibility::Visible, Size::Dp(1200.), 0.)
                }],
                ..button("bab_feed", Visibility::Visible, Size::Match, 0.)
            },
            Node {
                kind: Kind::Toolbar,
                id: "bab_c",
                width: Size::Match,
                height: Size::Dp(80.),
                image_src: "menu",
                layout_gravity: [0, 2],
                layout_gravity_explicit: true,
                relative_rules: BAB_CRADLE_CENTER_RULES,
                ..button("bab_c", Visibility::Visible, Size::Dp(80.), 0.)
            },
            Node {
                kind: Kind::Icon,
                id: "fab_c",
                width: Size::Dp(56.),
                height: Size::Dp(56.),
                radius: -1.0,
                background: Some(Color(103, 80, 164, 255)),
                icon: MaterialIcon::Add,
                relative_rules: ANCHOR_CENTER_RULES,
                ..button("fab_c", Visibility::Visible, Size::Dp(56.), 0.)
            },
        ],
        ..button("coord_bab_c", Visibility::Visible, Size::Match, 0.)
    };

    static COORD_EMBED: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::Toolbar,
                id: "bab_e",
                width: Size::Match,
                height: Size::Dp(80.),
                image_src: "menu",
                layout_gravity: [0, 2],
                layout_gravity_explicit: true,
                relative_rules: BAB_END_EMBED_RULES,
                ..button("bab_e", Visibility::Visible, Size::Dp(80.), 0.)
            },
            Node {
                kind: Kind::Icon,
                id: "fab_e",
                width: Size::Dp(56.),
                height: Size::Dp(56.),
                radius: -1.0,
                background: Some(Color(103, 80, 164, 255)),
                icon: MaterialIcon::Add,
                relative_rules: ANCHOR_END_RULES,
                ..button("fab_e", Visibility::Visible, Size::Dp(56.), 0.)
            },
        ],
        ..button("coord_bab_e", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 360., height: 480. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Center cradle: transparent cutout pixels at center top, icons skip around center cutout,
    //    CoordinatorLayout anchors sibling FAB at (180, 400 + 6) = (180, 406) -> rect (152, 378, 56, 56)
    let c0 = ui.frame(Input { time: 1.0, ..Input::default() }, b, 0., |f| {
        COORD_CRADLE.render(f, b, &mut state);
    });
    let (bab_img_b, bab_img) = c0.iter().find_map(|c| match c {
        DrawCommand::Image { bounds, image, .. } if (bounds.height - 80.0).abs() < 0.5 => Some((*bounds, image.clone())),
        _ => None,
    }).expect("cradle BottomAppBar must emit cached SDF cutout image");
    assert!((bab_img_b.y - 400.0).abs() < 0.1);
    let sample_alpha = |img: &Image, fx: f32, fy: f32| -> u8 {
        let px = ((fx * img.width() as f32) as u32).min(img.width() - 1);
        let py = ((fy * img.height() as f32) as u32).min(img.height() - 1);
        img.pixels()[((py * img.width() + px) * 4 + 3) as usize]
    };
    assert_eq!(sample_alpha(&bab_img, 0.5, 0.08), 0, "top-center cradle interior must be transparent");
    assert_eq!(sample_alpha(&bab_img, 0.1, 0.5), 255, "bar surface away from cutout must be opaque");

    // Check anchored FAB center = (180, 406) respecting cradle_voff_dp=6
    let fab_c0 = c0.iter().find_map(|c| match c {
        DrawCommand::Ellipse { bounds, color: Color(103, 80, 164, 255) } => Some(*bounds),
        _ => None,
    }).expect("anchored FAB must render");
    assert!((fab_c0.x + fab_c0.width * 0.5 - 180.0).abs() < 0.5, "center FAB cx=180: {fab_c0:?}");
    assert!((fab_c0.y + fab_c0.height * 0.5 - 406.0).abs() < 0.5, "cradle FAB cy=400+6=406: {fab_c0:?}");

    // Check no nav/action icon collides with center cutout [180-33, 180+33] = [147, 213]
    let icon_rects: Vec<Rect> = c0.iter().filter_map(|c| match c {
        DrawCommand::Shape { bounds, .. }
            if (bounds.width - 24.0).abs() < 0.5 && bounds.y >= 400.0 => Some(*bounds),
        _ => None,
    }).collect();
    assert!(icon_rects.len() >= 4, "nav + action icons render around center cutout: {icon_rects:?}");
    for r in &icon_rects {
        let slot_lo = r.x - 12.0;
        let slot_hi = r.x + r.width + 12.0;
        assert!(
            slot_hi <= 147.5 || slot_lo >= 212.5,
            "icon slot [{slot_lo}, {slot_hi}] must never collide with center cutout [147, 213]"
        );
    }

    // 2. hide_on_scroll=1: downward scroll slides BottomAppBar & anchored FAB smoothly down (+80px) over 0.22s,
    //    upward scroll slides back to 0, reduced_motion snaps immediately!
    state.offsets.insert("bab_feed".into(), 60.0);
    let _c_hd_start = ui.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| {
        COORD_CRADLE.render(f, b, &mut state);
    });
    assert!(ui.needs_redraw(), "hide_on_scroll must request continuous frames");
    let c_hd_mid = ui.frame(Input { time: 2.11, ..Input::default() }, b, 0., |f| {
        COORD_CRADLE.render(f, b, &mut state);
    });
    let bab_mid_y = c_hd_mid.iter().find_map(|c| match c {
        DrawCommand::Image { bounds, .. } if (bounds.height - 80.0).abs() < 0.5 => Some(bounds.y),
        _ => None,
    }).unwrap();
    assert!(bab_mid_y > 410.0 && bab_mid_y < 478.0, "mid-0.22s hide slides bar smoothly down, got y={bab_mid_y}");

    let c_hd_done = ui.frame(Input { time: 2.25, ..Input::default() }, b, 0., |f| {
        COORD_CRADLE.render(f, b, &mut state);
    });
    let bab_done_y = c_hd_done.iter().find_map(|c| match c {
        DrawCommand::Image { bounds, .. } if (bounds.height - 80.0).abs() < 0.5 => Some(bounds.y),
        _ => None,
    }).unwrap();
    let fab_done_y = c_hd_done.iter().find_map(|c| match c {
        DrawCommand::Ellipse { bounds, color: Color(103, 80, 164, 255) } => Some(bounds.y),
        _ => None,
    }).unwrap();
    assert!((bab_done_y - 480.0).abs() < 0.1, "fully hidden bar shifts +bar_height to y=480, got {bab_done_y}");
    assert!((fab_done_y - (fab_c0.y + 80.0)).abs() < 0.5, "anchored FAB hides smoothly with BottomAppBar");

    // Upward scroll with reduced_motion snaps immediately back to y=400
    ui.motion_preference = MotionPreference::Reduced;
    state.offsets.insert("bab_feed".into(), 20.0);
    let c_show_rm = ui.frame(Input { time: 2.3, ..Input::default() }, b, 0., |f| {
        COORD_CRADLE.render(f, b, &mut state);
    });
    assert!(!ui.needs_redraw(), "reduced_motion show snap must not request extra frames");
    let bab_back_y = c_show_rm.iter().find_map(|c| match c {
        DrawCommand::Image { bounds, .. } if (bounds.height - 80.0).abs() < 0.5 => Some(bounds.y),
        _ => None,
    }).unwrap();
    assert!((bab_back_y - 400.0).abs() < 0.1, "reduced_motion snaps bar back to y=400 immediately: {bab_back_y}");

    // 3. End + embed mode in LTR & RTL: circular pocket + anchored FAB centered inside bar at end edge,
    //    and icons stop before colliding with end pocket!
    let ce_ltr = ui.frame(Input::default(), b, 0., |f| {
        COORD_EMBED.render(f, b, &mut state);
    });
    let fab_e_ltr = ce_ltr.iter().find_map(|c| match c {
        DrawCommand::Ellipse { bounds, color: Color(103, 80, 164, 255) } => Some(*bounds),
        _ => None,
    }).unwrap();
    // LTR end FAB cx = 360 - 16 - 28 = 316; embed cy = 400 + 40 = 440
    assert!((fab_e_ltr.x + fab_e_ltr.width * 0.5 - 316.0).abs() < 0.5, "LTR end embed FAB cx=316: {fab_e_ltr:?}");
    assert!((fab_e_ltr.y + fab_e_ltr.height * 0.5 - 440.0).abs() < 0.5, "embed FAB cy=440: {fab_e_ltr:?}");
    // Pocket ellipse centered at (316, 440) with diameter 66
    assert!(ce_ltr.iter().any(|c| matches!(
        c,
        DrawCommand::Ellipse { bounds, color }
            if *color != Color(103, 80, 164, 255)
            && (bounds.width - 66.0).abs() < 0.5
            && (bounds.x + bounds.width * 0.5 - 316.0).abs() < 0.5
    )), "LTR end embed pocket ellipse must render around FAB");
    for c in &ce_ltr {
        if let DrawCommand::Shape { bounds, .. } = c
            && (bounds.width - 24.0).abs() < 0.5
            && bounds.y >= 400.0
            && (bounds.x + bounds.width * 0.5 - 316.0).abs() > 2.0
        {
            assert!(bounds.x + bounds.width + 12.0 <= 316.0 - 33.0 + 0.5, "LTR action icon must not collide with end pocket: {bounds:?}");
        }
    }

    // RTL end + embed mirrors FAB & pocket to physical left (cx = 44), and icons sit on physical right!
    state.rtl = true;
    let ce_rtl = ui.frame(Input::default(), b, 0., |f| {
        COORD_EMBED.render(f, b, &mut state);
    });
    let fab_e_rtl = ce_rtl.iter().find_map(|c| match c {
        DrawCommand::Ellipse { bounds, color: Color(103, 80, 164, 255) } => Some(*bounds),
        _ => None,
    }).unwrap();
    assert!((fab_e_rtl.x + fab_e_rtl.width * 0.5 - 44.0).abs() < 0.5, "RTL end embed FAB mirrors to cx=44: {fab_e_rtl:?}");
    assert!(ce_rtl.iter().any(|c| matches!(
        c,
        DrawCommand::Ellipse { bounds, color }
            if *color != Color(103, 80, 164, 255)
            && (bounds.width - 66.0).abs() < 0.5
            && (bounds.x + bounds.width * 0.5 - 44.0).abs() < 0.5
    )), "RTL end embed pocket mirrors to cx=44");
    for c in &ce_rtl {
        if let DrawCommand::Shape { bounds, .. } = c
            && (bounds.width - 24.0).abs() < 0.5
            && bounds.y >= 400.0
            && (bounds.x + bounds.width * 0.5 - 44.0).abs() > 2.0
        {
            assert!(bounds.x - 12.0 >= 44.0 + 33.0 - 0.5, "RTL action icon must not collide with left-end pocket: {bounds:?}");
        }
    }
}

#[test]
fn wave22a_recycler_layout_ext_horizontal_reverse_stack_end_staggered_and_rtl() {
    use bhoo::android_xml::RULE_RECYCLER_LAYOUT_EXT;

    let find_text = |cmds: &[DrawCommand], prefix: &str| -> Option<Rect> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::Text { text, bounds, .. } if text.as_ref().starts_with(prefix) => Some(*bounds),
            _ => None,
        })
    };

    // 1. Horizontal Linear + scroll_viewport_horizontal_rtl + LTR vs RTL + reverse=1 + stack_end=1
    static HORIZ_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_RECYCLER_LAYOUT_EXT, anchor: "linear|h|1|0|0" },
    ];
    static HORIZ_REV_STACK_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_RECYCLER_LAYOUT_EXT, anchor: "linear|h|1|1|1" },
    ];
    static VERT_REV_STACK_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_RECYCLER_LAYOUT_EXT, anchor: "linear|v|1|1|1" },
    ];
    static STAG_V_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_RECYCLER_LAYOUT_EXT, anchor: "staggered|v|2|0|0" },
    ];
    static STAG_H_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_RECYCLER_LAYOUT_EXT, anchor: "staggered|h|2|0|0" },
    ];

    static RV_H: Node = Node {
        kind: Kind::ListView,
        id: "rv_h",
        relative_rules: HORIZ_RULES,
        ..button("rv_h", Visibility::Visible, Size::Match, 0.)
    };
    static RV_H_RS: Node = Node {
        kind: Kind::ListView,
        id: "rv_h_rs",
        relative_rules: HORIZ_REV_STACK_RULES,
        ..button("rv_h_rs", Visibility::Visible, Size::Match, 0.)
    };
    static RV_V_RS: Node = Node {
        kind: Kind::ListView,
        id: "rv_v_rs",
        relative_rules: VERT_REV_STACK_RULES,
        ..button("rv_v_rs", Visibility::Visible, Size::Match, 0.)
    };
    static RV_STAG_V: Node = Node {
        kind: Kind::GridView,
        id: "rv_stag_v",
        relative_rules: STAG_V_RULES,
        ..button("rv_stag_v", Visibility::Visible, Size::Match, 0.)
    };
    static RV_STAG_H: Node = Node {
        kind: Kind::GridView,
        id: "rv_stag_h",
        relative_rules: STAG_H_RULES,
        ..button("rv_stag_h", Visibility::Visible, Size::Match, 0.)
    };

    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.list_items.insert(
        "rv_h".into(),
        vec!["Alpha".into(), "Beta".into(), "Gamma".into(), "Delta".into(), "Epsilon".into()],
    );
    let hb = Rect { x: 0., y: 0., width: 200., height: 64. };
    let ch_ltr0 = ui.frame(Input::default(), hb, 0., |f| { RV_H.render(f, hb, &mut state); });
    let xa = find_text(&ch_ltr0, "Alpha").unwrap().x;
    let xb = find_text(&ch_ltr0, "Beta").unwrap().x;
    assert!(xa < xb, "horizontal LTR arranges items left-to-right: {xa} < {xb}");

    // Horizontal wheel scroll shifts items left
    let ch_ltr1 = ui.frame(
        Input { pointer: Some([100., 32.]), scroll: [60., 0.], ..Input::default() },
        hb,
        0.,
        |f| { RV_H.render(f, hb, &mut state); },
    );
    assert!(state.offsets["rv_h"] > 40.0, "scroll_viewport_horizontal_rtl must advance horizontal offset");
    let xb_after = find_text(&ch_ltr1, "Beta").unwrap().x;
    assert!(xb_after < xb - 40.0, "horizontal scroll shifts Beta left: {xb} -> {xb_after}");

    // Horizontal RTL arranges items right-to-left (Alpha on physical right, Beta to its left)
    state.offsets.insert("rv_h".into(), 0.0);
    state.rtl = true;
    let ch_rtl = ui.frame(Input::default(), hb, 0., |f| { RV_H.render(f, hb, &mut state); });
    let xa_rtl = find_text(&ch_rtl, "Alpha").unwrap().x;
    let xb_rtl = find_text(&ch_rtl, "Beta").unwrap().x;
    assert!(xa_rtl > xb_rtl, "horizontal RTL arranges items right-to-left: {xa_rtl} > {xb_rtl}");
    state.rtl = false;

    // 2. reverse=1 + stack_end=1 (horizontal LTR & RTL, and vertical)
    state.list_items.insert("rv_h_rs".into(), vec!["One".into(), "Two".into()]);
    let big_hb = Rect { x: 0., y: 0., width: 320., height: 64. };
    let crs_ltr = ui.frame(Input::default(), big_hb, 0., |f| { RV_H_RS.render(f, big_hb, &mut state); });
    let r_one_ltr = find_text(&crs_ltr, "One").unwrap();
    let r_two_ltr = find_text(&crs_ltr, "Two").unwrap();
    assert!(r_two_ltr.x < r_one_ltr.x, "horizontal reverse=1 puts Two left of One in LTR");
    assert!(r_one_ltr.x > 200.0, "horizontal LTR stack_end=1 aligns short content flush to right edge: {r_one_ltr:?}");

    state.rtl = true;
    let crs_rtl = ui.frame(Input::default(), big_hb, 0., |f| { RV_H_RS.render(f, big_hb, &mut state); });
    let r_one_rtl = find_text(&crs_rtl, "One").unwrap();
    let r_two_rtl = find_text(&crs_rtl, "Two").unwrap();
    assert!(r_one_rtl.x < r_two_rtl.x, "horizontal RTL reverse=1 flips visual order: {r_one_rtl:?} vs {r_two_rtl:?}");
    assert!(r_one_rtl.x < 40.0, "horizontal RTL stack_end=1 aligns short content flush to left (end) edge: {r_one_rtl:?}");
    state.rtl = false;

    state.list_items.insert("rv_v_rs".into(), vec!["First".into(), "Second".into()]);
    let vb = Rect { x: 0., y: 0., width: 200., height: 300. };
    let cv_rs = ui.frame(Input::default(), vb, 0., |f| { RV_V_RS.render(f, vb, &mut state); });
    let r_first = find_text(&cv_rs, "First").unwrap();
    let r_second = find_text(&cv_rs, "Second").unwrap();
    assert!(r_second.y < r_first.y, "vertical reverse=1 puts Second above First");
    assert!(r_first.y > 240.0, "vertical stack_end=1 aligns short content flush to bottom edge: {r_first:?}");
    // Clicking reversed bottom item ("First") still reports adapter index 0!
    let click_pt = [r_first.x + 10., r_first.y + 10.];
    ui.frame(Input { pointer: Some(click_pt), pressed: true, ..Input::default() }, vb, 0., |f| { RV_V_RS.render(f, vb, &mut state); });
    ui.frame(Input { pointer: Some(click_pt), released: true, ..Input::default() }, vb, 0., |f| { RV_V_RS.render(f, vb, &mut state); });
    assert_eq!(state.list_clicked.get("rv_v_rs"), Some(&0), "reverse=1 preserves original adapter index on click");

    // 3. StaggeredGridLayoutManager vertical (2 cols): tall item 0 in col 0, short items 1 & 2 BOTH land in col 1!
    state.list_items.insert(
        "rv_stag_v".into(),
        vec![
            "TallZero word1 word2 word3 word4 word5 word6 word7 word8 word9".into(),
            "ShortOne".into(),
            "ShortTwo".into(),
            "ItemThree".into(),
        ],
    );
    let svb = Rect { x: 0., y: 0., width: 240., height: 320. };
    let csv = ui.frame(Input::default(), svb, 0., |f| { RV_STAG_V.render(f, svb, &mut state); });
    let r_tall = find_text(&csv, "TallZero").unwrap();
    let r_s1 = find_text(&csv, "ShortOne").unwrap();
    let r_s2 = find_text(&csv, "ShortTwo").unwrap();
    assert!(r_tall.x < 120.0, "item 0 in col 0 (left half): {r_tall:?}");
    assert!(r_s1.x >= 120.0, "item 1 in col 1 (right half): {r_s1:?}");
    assert!(
        r_s2.x >= 120.0 && r_s2.y > r_s1.y,
        "staggered greedy shortest-track places item 2 into shorter col 1 below item 1: s1={r_s1:?}, s2={r_s2:?}"
    );

    // RTL vertical staggered mirrors col 0 to right half and col 1 to left half
    state.rtl = true;
    let csv_rtl = ui.frame(Input::default(), svb, 0., |f| { RV_STAG_V.render(f, svb, &mut state); });
    assert!(find_text(&csv_rtl, "TallZero").unwrap().x >= 120.0, "RTL staggered col 0 sits on right half");
    assert!(find_text(&csv_rtl, "ShortTwo").unwrap().x < 120.0, "RTL staggered col 1 sits on left half");
    state.rtl = false;

    // 4. StaggeredGridLayoutManager horizontal (2 rows): wide item 0 in row 0, short items 1 & 2 BOTH land in row 1!
    state.list_items.insert(
        "rv_stag_h".into(),
        vec![
            "WideZero_12345678901234567890".into(),
            "S1".into(),
            "S2".into(),
        ],
    );
    let shb = Rect { x: 0., y: 0., width: 400., height: 120. };
    let csh = ui.frame(Input::default(), shb, 0., |f| { RV_STAG_H.render(f, shb, &mut state); });
    let r_wide = find_text(&csh, "WideZero").unwrap();
    let r_hs1 = find_text(&csh, "S1").unwrap();
    let r_hs2 = find_text(&csh, "S2").unwrap();
    assert!(r_wide.y < 60.0, "horizontal staggered item 0 in row 0 (top half): {r_wide:?}");
    assert!(r_hs1.y >= 60.0, "horizontal staggered item 1 in row 1 (bottom half): {r_hs1:?}");
    assert!(
        r_hs2.y >= 60.0 && r_hs2.x > r_hs1.x,
        "horizontal staggered places item 2 greedily into shorter row 1 after S1: s1={r_hs1:?}, s2={r_hs2:?}"
    );
}

#[test]
fn wave23a_search_view_ext_bar_coupling_animation_controls_rtl_and_reduced_motion() {
    use bhoo::android_xml::{AndroidTheme, RULE_SEARCH_BAR, RULE_SEARCH_VIEW_EXT};

    static SB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_SEARCH_BAR, anchor: "Search mail" }];
    static SV_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_SEARCH_VIEW_EXT,
        anchor: "w23_bar|Find messages|0|1",
    }];
    static SV_HIDE_NAV_RULES: &[RelativeRule] = &[RelativeRule {
        kind: RULE_SEARCH_VIEW_EXT,
        anchor: "w23_bar2|Quick find|1|0",
    }];

    static HOST: Node = Node {
        kind: Kind::Frame,
        children: &[
            Node {
                kind: Kind::SearchView,
                width: Size::Match,
                height: Size::Dp(56.),
                relative_rules: SB_RULES,
                ..button("w23_bar", Visibility::Visible, Size::Dp(56.), 0.)
            },
            Node {
                kind: Kind::Button,
                text: "Behind",
                margin: [0., 120., 0., 0.],
                width: Size::Dp(120.),
                height: Size::Dp(48.),
                ..button("w23_behind", Visibility::Visible, Size::Dp(48.), 0.)
            },
            Node {
                kind: Kind::Frame,
                width: Size::Match,
                height: Size::Match,
                relative_rules: SV_RULES,
                children: &[
                    Node {
                        kind: Kind::Text,
                        text: "Result Item A",
                        margin: [16., 12., 16., 0.],
                        width: Size::Match,
                        height: Size::Dp(32.),
                        ..button("w23_res_txt", Visibility::Visible, Size::Dp(32.), 0.)
                    },
                    Node {
                        kind: Kind::Button,
                        text: "Pick",
                        margin: [16., 60., 0., 0.],
                        width: Size::Dp(100.),
                        height: Size::Dp(44.),
                        ..button("w23_res_btn", Visibility::Visible, Size::Dp(44.), 0.)
                    },
                ],
                ..button("w23_sv", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("w23_root", Visibility::Visible, Size::Match, 0.)
    };

    static HIDE_NAV_SV: Node = Node {
        kind: Kind::Linear,
        horizontal: false,
        width: Size::Match,
        height: Size::Match,
        relative_rules: SV_HIDE_NAV_RULES,
        children: &[
            Node { kind: Kind::Text, text: "Row 1", height: Size::Dp(30.), ..button("hn_r1", Visibility::Visible, Size::Dp(30.), 0.) },
            Node { kind: Kind::Text, text: "Row 2", height: Size::Dp(30.), ..button("hn_r2", Visibility::Visible, Size::Dp(30.), 0.) },
        ],
        ..button("w23_sv_hn", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 320., height: 300. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Closed SearchView paints nothing and blocks no pointer input to behind button
    let closed_cmds = ui.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| {
        HOST.render(f, b, &mut state);
    });
    assert!(!closed_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Find messages")));
    assert!(!closed_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Result Item A")));

    let mut behind_act = None;
    ui.frame(Input { pointer: Some([40., 140.]), pressed: true, time: 0.05, ..Input::default() }, b, 0., |f| {
        HOST.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([40., 140.]), released: true, time: 0.08, ..Input::default() }, b, 0., |f| {
        behind_act = HOST.render(f, b, &mut state);
    });
    assert_eq!(behind_act, Some("w23_behind"), "closed SearchView must not block behind pointer input");

    // 2. Clicking coupled SearchBar reports bar click AND opens SearchView automatically
    ui.frame(Input { pointer: Some([120., 28.]), pressed: true, time: 0.10, ..Input::default() }, b, 0., |f| {
        HOST.render(f, b, &mut state);
    });
    let mut bar_act = None;
    ui.frame(Input { pointer: Some([120., 28.]), released: true, time: 0.12, ..Input::default() }, b, 0., |f| {
        bar_act = HOST.render(f, b, &mut state);
    });
    assert_eq!(bar_act, Some("w23_bar"), "SearchBar click must still report bar ID");
    assert_eq!(state.search_open.get("w23_sv"), Some(&true), "SearchBar click must auto-open coupled SearchView");

    // 3. Mid-animation (0.24s ease-out at +0.12s): emits PushLayer with intermediate alpha (0 < alpha < 1)
    let mid_open = ui.frame(Input { time: 0.24, ..Input::default() }, b, 0., |f| {
        HOST.render(f, b, &mut state);
    });
    assert!(
        mid_open.iter().any(|c| matches!(c, DrawCommand::PushLayer { alpha, .. } if *alpha > 0.1 && *alpha < 0.95)),
        "SearchView must animate smoothly over 0.24s ease-out"
    );

    // 4. Settled open (+0.30s): surfaceContainerHigh, 56dp header, 1dp outline_variant divider, Role::TextField, clipped children
    let open_cmds = ui.frame(Input { time: 0.42, ..Input::default() }, b, 0., |f| {
        HOST.render(f, b, &mut state);
    });
    assert!(
        !open_cmds.iter().any(|c| matches!(c, DrawCommand::PushLayer { .. })),
        "settled open SearchView must not emit PushLayer"
    );
    assert!(open_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0xEC, 0xE6, 0xF0, 255), .. }
            if (r.width - 320.0).abs() < 0.1 && (r.height - 300.0).abs() < 0.1
    )));
    assert!(open_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, color: Color(0xCA, 0xC4, 0xD0, 255), .. }
            if (r.y - 56.0).abs() < 0.1 && (r.height - 1.0).abs() < 0.1
    )), "1dp outline_variant divider must sit directly below 56dp header");
    let res_rect = open_cmds.iter().find_map(|c| match c {
        DrawCommand::Text { bounds: r, text, .. } if text.as_ref() == "Result Item A" => Some(*r),
        _ => None,
    }).expect("SearchView child result must render below divider");
    assert!(res_rect.y >= 57.0, "SearchView child must sit below header+divider: {res_rect:?}");
    let tf_bounds = ui.semantics().iter().find(|s| s.role == Role::TextField && s.label.as_ref() == "Find messages")
        .expect("SearchView header must expose interactive Role::TextField").bounds;
    let back_bounds = ui.semantics().iter().find(|s| s.role == Role::Button && s.label.as_ref() == "Back")
        .expect("SearchView header must expose leading Back button").bounds;
    assert!(back_bounds.x < tf_bounds.x, "LTR Back button sits left of TextField");

    // Open overlay blocks pointer clicks to behind button while letting SearchView's own child button click
    let mut occluded_act = None;
    ui.frame(Input { pointer: Some([40., 140.]), pressed: true, time: 0.45, ..Input::default() }, b, 0., |f| {
        HOST.render(f, b, &mut state);
    });
    ui.frame(Input { pointer: Some([40., 140.]), released: true, time: 0.48, ..Input::default() }, b, 0., |f| {
        occluded_act = HOST.render(f, b, &mut state);
    });
    assert_eq!(occluded_act, Some("w23_res_btn"), "open SearchView must occlude behind button and dispatch own child button");

    // 5. Non-empty query shows text + trailing Clear button; clicking Clear clears state.text[sv_id]
    state.text.insert("w23_sv".into(), "invoices".into());
    let query_cmds = ui.frame(Input { time: 0.50, ..Input::default() }, b, 0., |f| {
        HOST.render(f, b, &mut state);
    });
    assert!(query_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "invoices")));
    let clear_ltr = ui.semantics().iter().find(|s| s.label.as_ref() == "Clear search")
        .expect("non-empty query must show trailing Clear button").bounds;
    assert!(clear_ltr.x > 250.0, "LTR Clear button sits on right: {clear_ltr:?}");

    // 6. RTL mirrors header controls (Back on right, Clear on left, text align=2)
    state.rtl = true;
    let rtl_cmds = ui.frame(Input { time: 0.52, ..Input::default() }, b, 0., |f| {
        HOST.render(f, b, &mut state);
    });
    let back_rtl = ui.semantics().iter().find(|s| s.label.as_ref() == "Back").unwrap().bounds;
    let clear_rtl = ui.semantics().iter().find(|s| s.label.as_ref() == "Clear search").unwrap().bounds;
    assert!(back_rtl.x > 250.0 && clear_rtl.x < 20.0, "RTL must mirror Back={back_rtl:?} and Clear={clear_rtl:?}");
    assert!(rtl_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, style, .. } if text.as_ref() == "invoices" && style.align == 2)));
    // Click RTL Clear button
    let cpt = [clear_rtl.x + clear_rtl.width * 0.5, clear_rtl.y + clear_rtl.height * 0.5];
    ui.frame(Input { pointer: Some(cpt), pressed: true, time: 0.54, ..Input::default() }, b, 0., |f| { HOST.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some(cpt), released: true, time: 0.56, ..Input::default() }, b, 0., |f| { HOST.render(f, b, &mut state); });
    assert_eq!(state.text.get("w23_sv").map(String::as_str), Some(""), "clicking Clear must empty SearchView text");
    state.rtl = false;

    // 7. Clicking Back closes SearchView and animates out over 0.24s
    let back_pt = [back_bounds.x + 24.0, back_bounds.y + 24.0];
    ui.frame(Input { pointer: Some(back_pt), pressed: true, time: 0.60, ..Input::default() }, b, 0., |f| { HOST.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some(back_pt), released: true, time: 0.62, ..Input::default() }, b, 0., |f| { HOST.render(f, b, &mut state); });
    assert_eq!(state.search_open.get("w23_sv"), Some(&false));
    let mid_close = ui.frame(Input { time: 0.74, ..Input::default() }, b, 0., |f| { HOST.render(f, b, &mut state); });
    assert!(mid_close.iter().any(|c| matches!(c, DrawCommand::PushLayer { alpha, .. } if *alpha > 0.05 && *alpha < 0.95)));
    let done_close = ui.frame(Input { time: 0.90, ..Input::default() }, b, 0., |f| { HOST.render(f, b, &mut state); });
    assert!(!done_close.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Result Item A")));

    // 8. Reduced motion opens & Escape closes instantly without PushLayer
    ui.motion_preference = MotionPreference::Reduced;
    state.search_open.insert("w23_sv".into(), true);
    let rm_open = ui.frame(Input { time: 1.00, ..Input::default() }, b, 0., |f| { HOST.render(f, b, &mut state); });
    assert!(!rm_open.iter().any(|c| matches!(c, DrawCommand::PushLayer { .. })));
    assert!(rm_open.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Result Item A")));
    ui.frame(
        Input { key: Some(KeyInput { key: Key::Escape, pressed: true, shift: false, repeat: false }), time: 1.05, ..Input::default() },
        b,
        0.,
        |f| { HOST.render(f, b, &mut state); },
    );
    assert_eq!(state.search_open.get("w23_sv"), Some(&false), "Escape must close SearchView");
    let rm_closed = ui.frame(Input { time: 1.06, ..Input::default() }, b, 0., |f| { HOST.render(f, b, &mut state); });
    assert!(!rm_closed.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Result Item A")));

    // 9. hide_nav=1 omits Back button and lays out Kind::Linear result rows stacked vertically
    state.search_open.insert("w23_sv_hn".into(), true);
    let hn_cmds = ui.frame(Input { time: 1.10, ..Input::default() }, b, 0., |f| { HIDE_NAV_SV.render(f, b, &mut state); });
    assert!(!ui.semantics().iter().any(|s| s.label.as_ref() == "Back"), "hide_nav=1 must omit Back button");
    let hn_tf = ui.semantics().iter().find(|s| s.role == Role::TextField).unwrap().bounds;
    assert!((hn_tf.x - 16.0).abs() < 0.5, "hide_nav=1 TextField starts at 16dp: {hn_tf:?}");
    let r1 = hn_cmds.iter().find_map(|c| match c { DrawCommand::Text { bounds: r, text, .. } if text.as_ref() == "Row 1" => Some(*r), _ => None }).unwrap();
    let r2 = hn_cmds.iter().find_map(|c| match c { DrawCommand::Text { bounds: r, text, .. } if text.as_ref() == "Row 2" => Some(*r), _ => None }).unwrap();
    assert!(r1.y >= 57.0 && r2.y > r1.y, "Kind::Linear SearchView stacks result rows below divider: r1={r1:?}, r2={r2:?}");
}

#[test]
fn wave23a_swipe_refresh_ext_pull_physics_colors_size_offsets_retract_and_reduced_motion() {
    use bhoo::android_xml::{AndroidTheme, RULE_SWIPE_REFRESH, RULE_SWIPE_REFRESH_EXT};

    static SRL_EXT_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_SWIPE_REFRESH, anchor: "" },
        RelativeRule {
            kind: RULE_SWIPE_REFRESH_EXT,
            anchor: "240,248,255,255|255,0,0,255;0,180,0,255;0,0,255,255|large|-56,72",
        },
    ];
    static SRL_DEF_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_SWIPE_REFRESH, anchor: "" },
        RelativeRule { kind: RULE_SWIPE_REFRESH_EXT, anchor: "||default|-40,64" },
    ];
    static SRL_CUSTOM: Node = Node {
        kind: Kind::Frame,
        relative_rules: SRL_EXT_RULES,
        children: &[Node {
            kind: Kind::Scroll,
            children: &[Node {
                kind: Kind::Linear,
                children: &[
                    Node {
                        kind: Kind::Button,
                        text: "Feed Row",
                        width: Size::Match,
                        height: Size::Dp(120.),
                        ..button("w23_feed_btn", Visibility::Visible, Size::Dp(120.), 0.)
                    },
                    Node {
                        kind: Kind::Text,
                        text: "More Feed",
                        width: Size::Match,
                        height: Size::Dp(400.),
                        ..button("w23_feed_more", Visibility::Visible, Size::Dp(400.), 0.)
                    },
                ],
                ..button("w23_feed_col", Visibility::Visible, Size::Wrap, 0.)
            }],
            ..button("w23_srl_scroll", Visibility::Visible, Size::Match, 0.)
        }],
        ..button("w23_srl", Visibility::Visible, Size::Match, 0.)
    };
    static SRL_DEFAULT: Node = Node {
        kind: Kind::Frame,
        relative_rules: SRL_DEF_RULES,
        children: &[Node { kind: Kind::Text, text: "Default Feed", ..button("w23_df", Visibility::Visible, Size::Match, 0.) }],
        ..button("w23_srl_def", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 320., height: 260. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. Idle paints no spinner
    let idle = ui.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    assert!(!idle.iter().any(|c| matches!(c, DrawCommand::Arc { .. })));

    // 2. When child scroll is NOT at top (> 0.5), downward drag does NOT activate pull-to-refresh
    state.offsets.insert("w23_srl_scroll".into(), 40.0);
    ui.frame(Input { pointer: Some([160., 40.]), pressed: true, time: 0.05, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    let not_top = ui.frame(Input { pointer: Some([160., 180.]), time: 0.10, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    assert!(!not_top.iter().any(|c| matches!(c, DrawCommand::Arc { .. })), "scrolled child must block pull-to-refresh");
    ui.frame(Input { pointer: Some([160., 180.]), released: true, time: 0.12, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });

    // 3. At top (offset=0): pulling downward 100dp (0.45x => 45dp < 72dp) shows 56dp large custom spinner & cancels child click
    state.offsets.insert("w23_srl_scroll".into(), 0.0);
    ui.frame(Input { pointer: Some([160., 30.]), pressed: true, time: 0.20, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    let pulling = ui.frame(Input { pointer: Some([160., 130.]), time: 0.25, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    let pull_circle = pulling.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(240, 248, 255, 255) }
            if (r.width - 56.0).abs() < 0.1 && (*radius - 28.0).abs() < 0.1 => Some(*r),
        _ => None,
    }).expect("large 56dp custom bg circle must appear during pull");
    assert!(pull_circle.y > -56.0 && pull_circle.y < 72.0, "live pull circle y must interpolate from start to end: {pull_circle:?}");
    assert!(pulling.iter().any(|c| matches!(c, DrawCommand::Arc { color: Color(255, 0, 0, 255), .. })));

    // Releasing below end_off_dp cancels child click and smoothly retracts over 0.22s ease-out
    let mut short_act = None;
    ui.frame(Input { pointer: Some([160., 130.]), released: true, time: 0.30, ..Input::default() }, b, 0., |f| {
        short_act = SRL_CUSTOM.render(f, b, &mut state);
    });
    assert_eq!(short_act, None, "dragging > 8dp slop must cancel child button click");
    assert_ne!(state.refreshing.get("w23_srl"), Some(&true));
    let retract_mid = ui.frame(Input { time: 0.41, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    let mid_y = retract_mid.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds: r, color: Color(240, 248, 255, 255), .. } if (r.width - 56.0).abs() < 0.1 => Some(r.y),
        _ => None,
    }).expect("spinner should still be retracting at 0.11s of 0.22s");
    assert!(mid_y < pull_circle.y && mid_y > -56.0, "retract mid y must move toward -56: pull={} mid={}", pull_circle.y, mid_y);
    let retract_done = ui.frame(Input { time: 0.55, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    assert!(!retract_done.iter().any(|c| matches!(c, DrawCommand::Arc { .. })), "settled retract must hide spinner");

    // 4. Reduced motion retracts instantly on release
    ui.motion_preference = MotionPreference::Reduced;
    ui.frame(Input { pointer: Some([160., 30.]), pressed: true, time: 0.60, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([160., 130.]), time: 0.65, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    let rm_rel = ui.frame(Input { pointer: Some([160., 130.]), released: true, time: 0.70, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    assert!(!rm_rel.iter().any(|c| matches!(c, DrawCommand::Arc { .. })), "reduced motion must retract spinner immediately on release");
    ui.motion_preference = MotionPreference::default();

    // 5. Pulling >= end_off_dp (dy=180 => pull=81 >= 72) triggers refreshing, settles at y=72, and cycles scheme colors over time!
    ui.frame(Input { pointer: Some([160., 20.]), pressed: true, time: 0.80, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([160., 200.]), time: 0.85, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    let mut trig_act = None;
    ui.frame(Input { pointer: Some([160., 200.]), released: true, time: 0.90, ..Input::default() }, b, 0., |f| {
        trig_act = SRL_CUSTOM.render(f, b, &mut state);
    });
    assert_eq!(trig_act, Some("w23_srl"));
    assert_eq!(state.refreshing.get("w23_srl"), Some(&true));

    let ref_c0 = ui.frame(Input { time: 1.20, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    let settled_c = ref_c0.iter().find_map(|c| match c {
        DrawCommand::RoundedRect { bounds: r, color: Color(240, 248, 255, 255), .. } if (r.width - 56.0).abs() < 0.1 => Some(*r),
        _ => None,
    }).expect("settled refreshing circle missing");
    assert!((settled_c.y - 72.0).abs() < 0.5, "refreshing circle must settle at end_off_dp=72: {settled_c:?}");
    assert!(ref_c0.iter().any(|c| matches!(c, DrawCommand::Arc { color: Color(255, 0, 0, 255), .. })));
    let ref_c1 = ui.frame(Input { time: 1.60, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    assert!(ref_c1.iter().any(|c| matches!(c, DrawCommand::Arc { color: Color(0, 180, 0, 255), .. })), "refreshing arc must cycle to 2nd scheme color");
    let ref_c2 = ui.frame(Input { time: 3.00, ..Input::default() }, b, 0., |f| { SRL_CUSTOM.render(f, b, &mut state); });
    assert!(ref_c2.iter().any(|c| matches!(c, DrawCommand::Arc { color: Color(0, 0, 255, 255), .. })), "refreshing arc must cycle to 3rd scheme color");

    // 6. Default RULE_SWIPE_REFRESH_EXT ("||default|-40,64") uses 40dp, surface_high (#ECE6F0), primary arc, y=64dp
    state.refreshing.insert("w23_srl_def".into(), true);
    let def_cmds = ui.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| { SRL_DEFAULT.render(f, b, &mut state); });
    assert!(def_cmds.iter().any(|c| matches!(
        c,
        DrawCommand::RoundedRect { bounds: r, radius, color: Color(0xEC, 0xE6, 0xF0, 255) }
            if (r.width - 40.0).abs() < 0.1 && (r.y - 64.0).abs() < 0.1 && (*radius - 20.0).abs() < 0.1
    )));
    assert!(def_cmds.iter().any(|c| matches!(c, DrawCommand::Arc { color: Color(0x67, 0x50, 0xA4, 255), .. })));
}

#[test]
fn wave23a_carousel_layout_strategies_interpolation_snap_mask_align_rtl_and_reduced_motion() {
    use bhoo::android_xml::{AndroidTheme, RULE_CAROUSEL_ITEM, RULE_CAROUSEL_LAYOUT};

    static CAR_ITEM_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CAROUSEL_ITEM, anchor: "" }];
    static MB_START_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CAROUSEL_LAYOUT, anchor: "multi_browse|8|start" }];
    static MB_CENTER_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CAROUSEL_LAYOUT, anchor: "multi_browse|8|center" }];
    static HERO_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CAROUSEL_LAYOUT, anchor: "hero|8|start" }];
    static UNC_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CAROUSEL_LAYOUT, anchor: "uncontained|8|start" }];
    static FS_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CAROUSEL_LAYOUT, anchor: "full_screen|0|start" }];

    static ITEMS: [Node; 4] = [
        Node { kind: Kind::Frame, background: Some(Color(210, 60, 60, 255)), relative_rules: CAR_ITEM_RULES, children: &[Node { kind: Kind::Text, text: "Card0", ..button("ct0", Visibility::Visible, Size::Match, 0.) }], ..button("ci0", Visibility::Visible, Size::Match, 0.) },
        Node { kind: Kind::Frame, background: Some(Color(60, 170, 80, 255)), relative_rules: CAR_ITEM_RULES, children: &[Node { kind: Kind::Text, text: "Card1", ..button("ct1", Visibility::Visible, Size::Match, 0.) }], ..button("ci1", Visibility::Visible, Size::Match, 0.) },
        Node { kind: Kind::Frame, background: Some(Color(60, 110, 220, 255)), relative_rules: CAR_ITEM_RULES, children: &[Node { kind: Kind::Text, text: "Card2", ..button("ct2", Visibility::Visible, Size::Match, 0.) }], ..button("ci2", Visibility::Visible, Size::Match, 0.) },
        Node { kind: Kind::Frame, background: Some(Color(200, 140, 40, 255)), relative_rules: CAR_ITEM_RULES, children: &[Node { kind: Kind::Text, text: "Card3", ..button("ct3", Visibility::Visible, Size::Match, 0.) }], ..button("ci3", Visibility::Visible, Size::Match, 0.) },
    ];
    static MB_START: Node = Node { kind: Kind::Linear, horizontal: true, relative_rules: MB_START_RULES, children: &ITEMS, ..button("car_mb", Visibility::Visible, Size::Match, 0.) };
    static MB_CENTER: Node = Node { kind: Kind::Linear, horizontal: true, relative_rules: MB_CENTER_RULES, children: &ITEMS, ..button("car_mbc", Visibility::Visible, Size::Match, 0.) };
    static HERO_CAR: Node = Node { kind: Kind::Linear, horizontal: true, relative_rules: HERO_RULES, children: &ITEMS, ..button("car_hero", Visibility::Visible, Size::Match, 0.) };
    static UNC_LIST: Node = Node { kind: Kind::ListView, relative_rules: UNC_RULES, ..button("car_unc", Visibility::Visible, Size::Match, 0.) };
    static FS_HSCROLL: Node = Node {
        kind: Kind::HorizontalScroll,
        relative_rules: FS_RULES,
        children: &[Node { kind: Kind::Linear, horizontal: true, children: &ITEMS, ..button("fs_lin", Visibility::Visible, Size::Wrap, 0.) }],
        ..button("car_fs", Visibility::Visible, Size::Match, 0.)
    };

    let find_card = |cmds: &[DrawCommand], col: Color| -> Option<(Rect, f32)> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::RoundedRect { bounds, radius, color } if *color == col => Some((*bounds, *radius)),
            _ => None,
        })
    };

    let b = Rect { x: 0., y: 0., width: 400., height: 180. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. multi_browse|8|start: focal ~55% (220), medium ~26% (104), small ~12% (48), 8dp gaps, 28dp mask
    let mb0 = ui.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    let (c0, r0) = find_card(&mb0, Color(210, 60, 60, 255)).expect("Card0 missing");
    let (c1, r1) = find_card(&mb0, Color(60, 170, 80, 255)).expect("Card1 missing");
    let (c2, _) = find_card(&mb0, Color(60, 110, 220, 255)).expect("Card2 missing");
    assert!((c0.width - 220.0).abs() < 1.0 && (c1.width - 104.0).abs() < 1.0 && (c2.width - 48.0).abs() < 1.0,
        "multi_browse keyline widths mismatch: c0={c0:?}, c1={c1:?}, c2={c2:?}");
    assert!((c0.x - 0.0).abs() < 0.5 && ((c1.x - (c0.x + c0.width)) - 8.0).abs() < 0.5 && ((c2.x - (c1.x + c1.width)) - 8.0).abs() < 0.5);
    assert!((r0 - 28.0).abs() < 0.5 && (r1 - 28.0).abs() < 0.5, "carousel items must use 28dp rounded mask");

    // 2. Smooth scroll-interpolated widths via horizontal wheel (half step = 114px)
    let mb_half = ui.frame(
        Input { pointer: Some([200., 90.]), scroll: [114., 0.], time: 0.10, ..Input::default() },
        b,
        0.,
        |f| { MB_START.render(f, b, &mut state); },
    );
    let (h0, _) = find_card(&mb_half, Color(210, 60, 60, 255)).unwrap();
    let (h1, _) = find_card(&mb_half, Color(60, 170, 80, 255)).unwrap();
    assert!(h0.width < c0.width - 30.0 && h1.width > c1.width + 30.0,
        "scroll must smoothly interpolate keyline widths: h0={} h1={}", h0.width, h1.width);

    // 3. Pointer drag + release smoothly snaps over 0.24s ease-out to focal item 1 (step = 228)
    state.offsets.insert("car_mb".into(), 0.0);
    ui.frame(Input { pointer: Some([300., 90.]), pressed: true, time: 0.20, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([140., 90.]), time: 0.25, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([140., 90.]), released: true, time: 0.30, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    let off_rel = state.offsets["car_mb"];
    assert!((off_rel - 160.0).abs() < 1.0, "release frame starts snap from drag offset: {off_rel}");
    ui.frame(Input { time: 0.42, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    let off_mid = state.offsets["car_mb"];
    assert!(off_mid > off_rel + 10.0 && off_mid < 228.0 - 1.0, "mid-snap offset must ease smoothly toward 228: {off_mid}");
    ui.frame(Input { time: 0.56, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    assert!((state.offsets["car_mb"] - 228.0).abs() < 0.5, "snap must settle at focal item 1 (228): {}", state.offsets["car_mb"]);

    // 4. Reduced motion snaps instantaneously on release
    ui.motion_preference = MotionPreference::Reduced;
    state.offsets.insert("car_mb".into(), 0.0);
    ui.frame(Input { pointer: Some([300., 90.]), pressed: true, time: 0.60, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([140., 90.]), time: 0.65, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([140., 90.]), released: true, time: 0.70, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    assert!((state.offsets["car_mb"] - 228.0).abs() < 0.5, "reduced motion must snap instantly to 228");
    ui.motion_preference = MotionPreference::default();

    // 5. align=center places focal item centered in viewport; RTL mirrors start-aligned items right-to-left
    let mbc = ui.frame(Input { time: 0.80, ..Input::default() }, b, 0., |f| { MB_CENTER.render(f, b, &mut state); });
    let (cc0, _) = find_card(&mbc, Color(210, 60, 60, 255)).unwrap();
    assert!(((cc0.x + cc0.width * 0.5) - 200.0).abs() < 1.0, "align=center must center focal item: {cc0:?}");

    state.offsets.insert("car_mb".into(), 0.0);
    state.rtl = true;
    let mb_rtl = ui.frame(Input { time: 0.85, ..Input::default() }, b, 0., |f| { MB_START.render(f, b, &mut state); });
    let (rc0, _) = find_card(&mb_rtl, Color(210, 60, 60, 255)).unwrap();
    let (rc1, _) = find_card(&mb_rtl, Color(60, 170, 80, 255)).unwrap();
    assert!((rc0.x + rc0.width - 400.0).abs() < 0.5 && rc1.x + rc1.width < rc0.x, "RTL must place Card0 flush right and Card1 to its left: rc0={rc0:?} rc1={rc1:?}");
    state.rtl = false;

    // 6. hero (78% / 18%), uncontained (60% adapter ListView), full_screen (100% HorizontalScroll)
    let hero_cmds = ui.frame(Input { time: 0.90, ..Input::default() }, b, 0., |f| { HERO_CAR.render(f, b, &mut state); });
    let (hc0, _) = find_card(&hero_cmds, Color(210, 60, 60, 255)).unwrap();
    let (hc1, _) = find_card(&hero_cmds, Color(60, 170, 80, 255)).unwrap();
    assert!((hc0.width - 312.0).abs() < 1.0 && (hc1.width - 72.0).abs() < 1.0, "hero widths must be 78%/18%: {hc0:?} {hc1:?}");

    state.list_items.insert("car_unc".into(), vec!["Item A".into(), "Item B".into(), "Item C".into()]);
    let unc_cmds = ui.frame(Input { time: 0.95, ..Input::default() }, b, 0., |f| { UNC_LIST.render(f, b, &mut state); });
    let unc_cards: Vec<Rect> = unc_cmds.iter().filter_map(|c| match c {
        DrawCommand::RoundedRect { bounds: r, radius, .. } if (*radius - 28.0).abs() < 0.5 && (r.width - 240.0).abs() < 1.0 => Some(*r),
        _ => None,
    }).collect();
    assert_eq!(unc_cards.len(), 2, "uncontained 60% viewport must show 1 full + 1 peeking trailing item: {unc_cards:?}");

    ui.motion_preference = MotionPreference::Reduced;
    let fs0 = ui.frame(Input { time: 1.00, ..Input::default() }, b, 0., |f| { FS_HSCROLL.render(f, b, &mut state); });
    let (fc0, _) = find_card(&fs0, Color(210, 60, 60, 255)).unwrap();
    assert!((fc0.width - 400.0).abs() < 0.5, "full_screen item must span 100% viewport width: {fc0:?}");
    ui.frame(Input { pointer: Some([350., 90.]), pressed: true, time: 1.05, ..Input::default() }, b, 0., |f| { FS_HSCROLL.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([100., 90.]), time: 1.10, ..Input::default() }, b, 0., |f| { FS_HSCROLL.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([100., 90.]), released: true, time: 1.15, ..Input::default() }, b, 0., |f| { FS_HSCROLL.render(f, b, &mut state); });
    assert!((state.offsets["car_fs"] - 400.0).abs() < 0.5, "full_screen drag release must snap to page 1 (400)");
}

#[test]
fn wave23a_view_pager_ext_user_input_margin_velocity_transformers_tabs_rtl_and_reduced_motion() {
    use bhoo::android_xml::{AndroidTheme, RULE_TAB_OPTIONS, RULE_VIEW_PAGER, RULE_VIEW_PAGER_EXT};

    static VP_ZOOM_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_VIEW_PAGER, anchor: "horizontal" },
        RelativeRule { kind: RULE_VIEW_PAGER_EXT, anchor: "h|1|16|zoom_out" },
    ];
    static VP_DEPTH_V_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_VIEW_PAGER, anchor: "vertical" },
        RelativeRule { kind: RULE_VIEW_PAGER_EXT, anchor: "v|1|12|depth" },
    ];
    static VP_LOCKED_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_VIEW_PAGER, anchor: "horizontal" },
        RelativeRule { kind: RULE_VIEW_PAGER_EXT, anchor: "h|0|0|none" },
    ];
    static TAB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_TAB_OPTIONS, anchor: "fixed|primary|" }];

    static PAGES: [Node; 3] = [
        Node { kind: Kind::Text, text: "Page A", background: Some(Color(230, 230, 250, 255)), ..button("vp_pa", Visibility::Visible, Size::Match, 0.) },
        Node { kind: Kind::Text, text: "Page B", background: Some(Color(230, 250, 230, 255)), ..button("vp_pb", Visibility::Visible, Size::Match, 0.) },
        Node { kind: Kind::Text, text: "Page C", background: Some(Color(250, 230, 230, 255)), ..button("vp_pc", Visibility::Visible, Size::Match, 0.) },
    ];
    static VP_ZOOM: Node = Node { kind: Kind::ViewFlipper, relative_rules: VP_ZOOM_RULES, children: &PAGES, ..button("w23_vp_zoom", Visibility::Visible, Size::Match, 0.) };
    static VP_DEPTH_V: Node = Node { kind: Kind::ViewFlipper, relative_rules: VP_DEPTH_V_RULES, children: &PAGES, ..button("w23_vp_depth", Visibility::Visible, Size::Match, 0.) };
    static LOCKED_HOST: Node = Node {
        kind: Kind::Linear,
        horizontal: false,
        children: &[
            Node {
                kind: Kind::TabWidget,
                width: Size::Match,
                height: Size::Dp(48.),
                relative_rules: TAB_RULES,
                children: &[
                    Node { kind: Kind::Text, text: "T0", align: 1, content_gravity: [1, 1], ..button("w23_t0", Visibility::Visible, Size::Match, 0.) },
                    Node { kind: Kind::Text, text: "T1", align: 1, content_gravity: [1, 1], ..button("w23_t1", Visibility::Visible, Size::Match, 0.) },
                    Node { kind: Kind::Text, text: "T2", align: 1, content_gravity: [1, 1], ..button("w23_t2", Visibility::Visible, Size::Match, 0.) },
                ],
                ..button("w23_tabs", Visibility::Visible, Size::Match, 0.)
            },
            Node {
                kind: Kind::ViewFlipper,
                width: Size::Match,
                height: Size::Dp(200.),
                relative_rules: VP_LOCKED_RULES,
                children: &PAGES,
                ..button("w23_vp_locked", Visibility::Visible, Size::Match, 0.)
            },
        ],
        ..button("w23_locked_root", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 300., height: 200. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. h|1|16|zoom_out: mid-drag applies 16dp pageMargin gap AND zoom_out scale/alpha via PushTransform/PushLayer
    ui.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([250., 100.]), pressed: true, time: 0.10, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    let drag_zoom = ui.frame(Input { pointer: Some([100., 100.]), time: 0.15, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    let pa = drag_zoom.iter().find_map(|c| match c { DrawCommand::RoundedRect { bounds: r, color: Color(230, 230, 250, 255), .. } => Some(*r), _ => None }).expect("Page A rect missing");
    let pb = drag_zoom.iter().find_map(|c| match c { DrawCommand::RoundedRect { bounds: r, color: Color(230, 250, 230, 255), .. } => Some(*r), _ => None }).expect("Page B rect missing");
    assert!(((pb.x - (pa.x + pa.width)) - 16.0).abs() < 0.5, "pageMargin=16dp gap must separate adjacent pages: pa={pa:?} pb={pb:?}");
    let zoom_scales: Vec<f32> = drag_zoom.iter().filter_map(|c| match c { DrawCommand::PushTransform { scale, .. } => Some(scale[0]), _ => None }).collect();
    let zoom_alphas: Vec<f32> = drag_zoom.iter().filter_map(|c| match c { DrawCommand::PushLayer { alpha, .. } => Some(*alpha), _ => None }).collect();
    assert_eq!(zoom_scales.len(), 2, "both pages receive zoom_out PushTransform during drag");
    assert_eq!(zoom_alphas.len(), 2, "both pages receive zoom_out PushLayer during drag");
    assert!(zoom_scales.iter().all(|&s| s >= 0.85 - 1e-4 && s < 0.99), "zoom_out scales must lie in [0.85, 1.0): {zoom_scales:?}");
    assert!(zoom_alphas.iter().all(|&a| a >= 0.50 - 1e-4 && a < 0.99), "zoom_out alphas must lie in [0.5, 1.0): {zoom_alphas:?}");
    // Release past 30% advances 0 -> 1 and animates 0.25s settle
    ui.frame(Input { pointer: Some([100., 100.]), released: true, time: 0.20, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    assert_eq!(state.displayed_children["w23_vp_zoom"], 1);
    let settle_mid = ui.frame(Input { time: 0.30, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    assert!(settle_mid.iter().any(|c| matches!(c, DrawCommand::PushTransform { .. })), "0.25s settle should still be active at +0.10s");
    let settle_done = ui.frame(Input { time: 0.50, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    assert!(!settle_done.iter().any(|c| matches!(c, DrawCommand::PushTransform { .. })), "settled ViewPager must emit no transform");

    // 2. Fast short flick (> 400 dp/s, < 30% distance) advances 1 -> 2; slow short drag stays on page 2
    ui.frame(Input { pointer: Some([200., 100.]), pressed: true, time: 0.60, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([150., 100.]), time: 0.65, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([150., 100.]), released: true, time: 0.66, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    assert_eq!(state.displayed_children["w23_vp_zoom"], 2, "fast short flick (>400 dp/s) must advance page");
    ui.frame(Input { time: 1.00, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });

    // RTL horizontal drag right retreats/advances with 16dp margin
    state.rtl = true;
    state.displayed_children.insert("w23_vp_zoom".into(), 0);
    ui.motion_preference = MotionPreference::Reduced;
    ui.frame(Input { time: 1.05, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([60., 100.]), pressed: true, time: 1.10, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    let rtl_drag = ui.frame(Input { pointer: Some([210., 100.]), time: 1.15, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    let pa_rtl = rtl_drag.iter().find_map(|c| match c { DrawCommand::RoundedRect { bounds: r, color: Color(230, 230, 250, 255), .. } => Some(*r), _ => None }).unwrap();
    let pb_rtl = rtl_drag.iter().find_map(|c| match c { DrawCommand::RoundedRect { bounds: r, color: Color(230, 250, 230, 255), .. } => Some(*r), _ => None }).unwrap();
    assert!(((pa_rtl.x - (pb_rtl.x + pb_rtl.width)) - 16.0).abs() < 0.5, "RTL pageMargin=16dp gap must sit between Page B and Page A: pa={pa_rtl:?} pb={pb_rtl:?}");
    let rtl_rel = ui.frame(Input { pointer: Some([210., 100.]), released: true, time: 1.20, ..Input::default() }, b, 0., |f| { VP_ZOOM.render(f, b, &mut state); });
    assert_eq!(state.displayed_children["w23_vp_zoom"], 1);
    assert!(!rtl_rel.iter().any(|c| matches!(c, DrawCommand::PushTransform { .. })), "reduced motion must settle ViewPager instantly on release");
    state.rtl = false;
    ui.motion_preference = MotionPreference::default();

    // 3. v|1|12|depth: behind page (p > 0) scales 0.75+0.25*(1-p) and fades (1-p) while outgoing stays scale 1.0 on top
    ui.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| { VP_DEPTH_V.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some([150., 180.]), pressed: true, time: 0.10, ..Input::default() }, b, 0., |f| { VP_DEPTH_V.render(f, b, &mut state); });
    let depth_drag = ui.frame(Input { pointer: Some([150., 80.]), time: 0.15, ..Input::default() }, b, 0., |f| { VP_DEPTH_V.render(f, b, &mut state); });
    let depth_scales: Vec<f32> = depth_drag.iter().filter_map(|c| match c { DrawCommand::PushTransform { scale, .. } => Some(scale[0]), _ => None }).collect();
    let depth_alphas: Vec<f32> = depth_drag.iter().filter_map(|c| match c { DrawCommand::PushLayer { alpha, .. } => Some(*alpha), _ => None }).collect();
    assert_eq!(depth_scales.len(), 1, "depth transformer must scale ONLY behind (p>0) page, keeping outgoing at 1.0");
    assert_eq!(depth_alphas.len(), 1, "depth transformer must fade ONLY behind (p>0) page");
    // dy = -100, page_stride = 200 + 12 = 212 => p_behind = (212 - 100)/212 ≈ 0.5283 => scale ≈ 0.8679, alpha ≈ 0.4717
    let expected_p = (212.0 - 100.0) / 212.0;
    assert!((depth_scales[0] - (0.75 + 0.25 * (1.0 - expected_p))).abs() < 0.02, "depth scale mismatch: {}", depth_scales[0]);
    assert!((depth_alphas[0] - (1.0 - expected_p)).abs() < 0.02, "depth alpha mismatch: {}", depth_alphas[0]);
    let idx_behind = depth_drag.iter().position(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page B")).unwrap();
    let idx_out = depth_drag.iter().position(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page A")).unwrap();
    assert!(idx_behind < idx_out, "depth behind page must paint underneath outgoing page");
    ui.frame(Input { pointer: Some([150., 80.]), released: true, time: 0.20, ..Input::default() }, b, 0., |f| { VP_DEPTH_V.render(f, b, &mut state); });
    assert_eq!(state.displayed_children["w23_vp_depth"], 1, "vertical drag past 30% must advance page");

    // 4. user_input=0 disables drag swiping while preserving programmatic displayed_children & TabLayout sync
    let lb = Rect { x: 0., y: 0., width: 300., height: 248. };
    state.tab_pager_links.insert("w23_tabs".into(), "w23_vp_locked".into());
    ui.motion_preference = MotionPreference::Reduced;
    ui.frame(Input { time: 0.0, ..Input::default() }, lb, 0., |f| { LOCKED_HOST.render(f, lb, &mut state); });
    ui.frame(Input { pointer: Some([260., 150.]), pressed: true, time: 0.10, ..Input::default() }, lb, 0., |f| { LOCKED_HOST.render(f, lb, &mut state); });
    ui.frame(Input { pointer: Some([40., 150.]), time: 0.15, ..Input::default() }, lb, 0., |f| { LOCKED_HOST.render(f, lb, &mut state); });
    ui.frame(Input { pointer: Some([40., 150.]), released: true, time: 0.20, ..Input::default() }, lb, 0., |f| { LOCKED_HOST.render(f, lb, &mut state); });
    assert_eq!(state.displayed_children["w23_vp_locked"], 0, "user_input=0 must disable interactive drag swiping");

    // Programmatic displayed_children still works
    state.displayed_children.insert("w23_vp_locked".into(), 2);
    let prog_cmds = ui.frame(Input { time: 0.25, ..Input::default() }, lb, 0., |f| { LOCKED_HOST.render(f, lb, &mut state); });
    assert!(prog_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page C")));

    // Linked TabLayout click (Tab 1 at x=150, y=24) still changes ViewPager page!
    ui.frame(Input { pointer: Some([150., 24.]), pressed: true, time: 0.30, ..Input::default() }, lb, 0., |f| { LOCKED_HOST.render(f, lb, &mut state); });
    let tab_cmds = ui.frame(Input { pointer: Some([150., 24.]), released: true, time: 0.35, ..Input::default() }, lb, 0., |f| { LOCKED_HOST.render(f, lb, &mut state); });
    assert_eq!(state.displayed_children["w23_vp_locked"], 1, "linked TabLayout click must update user_input=0 ViewPager");
    assert!(tab_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, .. } if text.as_ref() == "Page B")));
}

#[test]
fn wave24a_text_links_spans_web_email_phone_custom_color_clickable_and_non_clickable() {
    use bhoo::android_xml::{AndroidTheme, RULE_TEXT_LINKS_SPANS};

    static ALL_CLICK_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LINKS_SPANS, anchor: "all||1" },
    ];
    static CUSTOM_NOCLICK_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LINKS_SPANS, anchor: "web,email|25,118,210,255|0" },
    ];
    static PIPE_MASK_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LINKS_SPANS, anchor: "web|phone|180,40,90,255|1" },
    ];
    static NONE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LINKS_SPANS, anchor: "none||1" },
    ];

    static TV_ALL: Node = Node {
        kind: Kind::Text,
        text: "Go https://example.com/docs_(v2), mail alice.smith+tag@example.org! Call +1-555-123-4567 or (555) 123-4567 or 555-1234 (not 123-456).",
        text_size: 16.0,
        relative_rules: ALL_CLICK_RULES,
        ..button("tv_all", Visibility::Visible, Size::Match, 0.)
    };
    static TV_CUSTOM: Node = Node {
        kind: Kind::Text,
        text: "See www.rust-lang.org, dev@bhoo.dev, or 555-1234.",
        text_size: 16.0,
        relative_rules: CUSTOM_NOCLICK_RULES,
        ..button("tv_custom", Visibility::Visible, Size::Match, 0.)
    };
    static TV_PIPE: Node = Node {
        kind: Kind::Text,
        text: "Site http://bhoo.dev mail a@b.io tel 555-1234",
        text_size: 16.0,
        relative_rules: PIPE_MASK_RULES,
        ..button("tv_pipe", Visibility::Visible, Size::Match, 0.)
    };
    static TV_NONE: Node = Node {
        kind: Kind::Text,
        text: "https://example.com a@b.io 555-1234",
        text_size: 16.0,
        relative_rules: NONE_RULES,
        ..button("tv_none", Visibility::Visible, Size::Match, 0.)
    };

    let b = Rect { x: 0., y: 0., width: 420., height: 180. };
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;

    // 1. mask="all", default link color (t.primary), clickable=1
    let mut action = None;
    let cmds = ui.frame(Input::default(), b, 0., |f| { action = TV_ALL.render(f, b, &mut state); });
    assert_eq!(action, None);
    let primary = Color(0x67, 0x50, 0xA4, 255); // Material3Light primary
    let links: Vec<(String, Rect)> = ui
        .semantics()
        .iter()
        .filter(|s| s.role == Role::Link)
        .map(|s| (s.label.to_string(), s.bounds))
        .collect();
    let labels: Vec<&str> = links.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "https://example.com/docs_(v2)",
            "alice.smith+tag@example.org",
            "+1-555-123-4567",
            "(555) 123-4567",
            "555-1234",
        ],
        "all mask must match web/email/phone and ignore 6-digit 123-456 and trailing punctuation"
    );
    for (lbl, rect) in &links {
        assert!(
            cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, color, bounds, .. }
                if text.as_ref() == lbl.as_str() && *color == primary && (bounds.x - rect.x).abs() < 0.5 && (bounds.width - rect.width).abs() < 0.5)),
            "missing primary-colored link text for {lbl}"
        );
        assert!(
            cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color, .. }
                if *color == primary && (bounds.height - 1.0).abs() < 0.05 && (bounds.x - rect.x).abs() < 0.5 && (bounds.width - rect.width).abs() < 0.5 && bounds.y >= rect.y && bounds.y <= rect.y + rect.height)),
            "missing 1dp underline beneath link {lbl}"
        );
    }

    // Click first link -> sets state.last_clicked_link and returns Some("tv_all")
    let p0 = [links[0].1.x + links[0].1.width * 0.5, links[0].1.y + links[0].1.height * 0.5];
    ui.frame(Input { pointer: Some(p0), pressed: true, time: 0.1, ..Input::default() }, b, 0., |f| { action = TV_ALL.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some(p0), released: true, time: 0.15, ..Input::default() }, b, 0., |f| { action = TV_ALL.render(f, b, &mut state); });
    assert_eq!(action, Some("tv_all"));
    assert_eq!(state.last_clicked_link.as_deref(), Some("https://example.com/docs_(v2)"));

    // Click second link -> overwrites state.last_clicked_link
    let p1 = [links[1].1.x + links[1].1.width * 0.5, links[1].1.y + links[1].1.height * 0.5];
    ui.frame(Input { pointer: Some(p1), pressed: true, time: 0.2, ..Input::default() }, b, 0., |f| { action = TV_ALL.render(f, b, &mut state); });
    ui.frame(Input { pointer: Some(p1), released: true, time: 0.25, ..Input::default() }, b, 0., |f| { action = TV_ALL.render(f, b, &mut state); });
    assert_eq!(action, Some("tv_all"));
    assert_eq!(state.last_clicked_link.as_deref(), Some("alice.smith+tag@example.org"));

    // 2. mask="web,email", custom RGBA, clickable=0: highlights web+email only, zero Role::Link targets
    let custom_col = Color(25, 118, 210, 255);
    let mut ui2 = Ui::default();
    let cmds2 = ui2.frame(Input::default(), b, 0., |f| { TV_CUSTOM.render(f, b, &mut state); });
    assert!(ui2.semantics().iter().all(|s| s.role != Role::Link), "clickable=0 must not register Role::Link targets");
    assert!(cmds2.iter().any(|c| matches!(c, DrawCommand::Text { text, color, .. } if text.as_ref() == "www.rust-lang.org" && *color == custom_col)));
    assert!(cmds2.iter().any(|c| matches!(c, DrawCommand::Text { text, color, .. } if text.as_ref() == "dev@bhoo.dev" && *color == custom_col)));
    assert!(!cmds2.iter().any(|c| matches!(c, DrawCommand::Text { text, color, .. } if text.as_ref() == "555-1234" && *color == custom_col)));
    let underlines2 = cmds2.iter().filter(|c| matches!(c, DrawCommand::RoundedRect { bounds, color, .. } if *color == custom_col && (bounds.height - 1.0).abs() < 0.05)).count();
    assert_eq!(underlines2, 2, "only web and email should be underlined");

    // 3. Pipe-joined mask="web|phone" with clickable=1
    let pipe_col = Color(180, 40, 90, 255);
    let mut ui3 = Ui::default();
    let cmds3 = ui3.frame(Input::default(), b, 0., |f| { TV_PIPE.render(f, b, &mut state); });
    let pipe_links: Vec<String> = ui3.semantics().iter().filter(|s| s.role == Role::Link).map(|s| s.label.to_string()).collect();
    assert_eq!(pipe_links, vec!["http://bhoo.dev", "555-1234"]);
    assert!(cmds3.iter().any(|c| matches!(c, DrawCommand::Text { text, color, .. } if text.as_ref() == "http://bhoo.dev" && *color == pipe_col)));
    assert!(!cmds3.iter().any(|c| matches!(c, DrawCommand::Text { text, color, .. } if text.as_ref() == "a@b.io" && *color == pipe_col)));

    // 4. mask="none" emits zero links and zero underlines
    let mut ui4 = Ui::default();
    let cmds4 = ui4.frame(Input::default(), b, 0., |f| { TV_NONE.render(f, b, &mut state); });
    assert!(ui4.semantics().iter().all(|s| s.role != Role::Link));
    assert!(!cmds4.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, .. } if (bounds.height - 1.0).abs() < 0.05)));
}

#[test]
fn wave24a_text_links_spans_alignment_rtl_justified_clickable_text_and_edit_text() {
    use bhoo::android_xml::{AndroidTheme, RULE_TEXT_LAYOUT_EXT, RULE_TEXT_LINKS_SPANS};

    static LINK_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LINKS_SPANS, anchor: "all|30,100,220,255|1" },
    ];
    static JUSTIFY_LINK_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_TEXT_LAYOUT_EXT, anchor: "simple|none|inter_word|0,0,0" },
        RelativeRule { kind: RULE_TEXT_LINKS_SPANS, anchor: "web|30,100,220,255|1" },
    ];

    static TV_CENTER: Node = Node {
        kind: Kind::Text,
        text: "Visit https://bhoo.dev now",
        align: 1,
        content_gravity: [1, 0],
        text_size: 16.0,
        single_line: true,
        relative_rules: LINK_RULES,
        ..button("tv_center", Visibility::Visible, Size::Match, 0.)
    };
    static TV_DIR: Node = Node {
        kind: Kind::Text,
        text: "Go https://bhoo.dev",
        text_size: 16.0,
        single_line: true,
        clickable: true,
        relative_rules: LINK_RULES,
        ..button("tv_dir", Visibility::Visible, Size::Match, 0.)
    };
    static TV_JUST: Node = Node {
        kind: Kind::Text,
        text: "Check https://bhoo.dev today for details and more words on next line",
        text_size: 16.0,
        relative_rules: JUSTIFY_LINK_RULES,
        ..button("tv_just", Visibility::Visible, Size::Match, 0.)
    };
    static EDIT_LINK: Node = Node {
        kind: Kind::Edit,
        text: "Mail ops@bhoo.dev please",
        text_size: 16.0,
        relative_rules: LINK_RULES,
        ..button("ed_link", Visibility::Visible, Size::Match, 0.)
    };
    static EDIT_PASS: Node = Node {
        kind: Kind::Edit,
        text: "https://secret.example.com",
        input_type: 4,
        text_size: 16.0,
        relative_rules: LINK_RULES,
        ..button("ed_pass", Visibility::Visible, Size::Match, 0.)
    };

    let link_col = Color(30, 100, 220, 255);
    let b = Rect { x: 0., y: 0., width: 320., height: 100. };

    // 1. Center alignment places link near horizontal center
    let mut ui = Ui::default();
    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::Material3Light;
    let c_center = ui.frame(Input::default(), b, 0., |f| { TV_CENTER.render(f, b, &mut state); });
    let link_center = ui.semantics().iter().find(|s| s.role == Role::Link).expect("center link missing").bounds;
    let center_mid = link_center.x + link_center.width * 0.5;
    assert!((center_mid - 160.0).abs() < 35.0, "centered link should sit near middle of 320px box: {link_center:?}");
    assert!(c_center.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color, .. }
        if *color == link_col && (bounds.x - link_center.x).abs() < 0.5 && (bounds.width - link_center.width).abs() < 0.5)));

    // 2. LTR vs RTL alignment shifts link rect to right edge in RTL, and clickable TextView defers to link click
    let mut ui_ltr = Ui::default();
    ui_ltr.frame(Input::default(), b, 0., |f| { TV_DIR.render(f, b, &mut state); });
    let ltr_rect = ui_ltr.semantics().iter().find(|s| s.role == Role::Link).unwrap().bounds;

    state.rtl = true;
    let mut ui_rtl = Ui::default();
    ui_rtl.frame(Input::default(), b, 0., |f| { TV_DIR.render(f, b, &mut state); });
    let rtl_rect = ui_rtl.semantics().iter().find(|s| s.role == Role::Link).unwrap().bounds;
    assert!(rtl_rect.x > ltr_rect.x + 100.0, "RTL must align line to right edge: ltr={ltr_rect:?} rtl={rtl_rect:?}");
    assert!(((rtl_rect.x + rtl_rect.width) - 320.0).abs() < 1.0, "RTL trailing link must reach right edge: {rtl_rect:?}");

    let p_rtl = [rtl_rect.x + rtl_rect.width * 0.5, rtl_rect.y + rtl_rect.height * 0.5];
    let mut action = None;
    ui_rtl.frame(Input { pointer: Some(p_rtl), pressed: true, time: 0.1, ..Input::default() }, b, 0., |f| { action = TV_DIR.render(f, b, &mut state); });
    ui_rtl.frame(Input { pointer: Some(p_rtl), released: true, time: 0.15, ..Input::default() }, b, 0., |f| { action = TV_DIR.render(f, b, &mut state); });
    assert_eq!(action, Some("tv_dir"));
    assert_eq!(state.last_clicked_link.as_deref(), Some("https://bhoo.dev"));

    // 3. Inter-word justified multiline text with links in LTR and RTL
    for rtl in [false, true] {
        state.rtl = rtl;
        let mut ui_j = Ui::default();
        let cj = ui_j.frame(Input::default(), Rect { x: 0., y: 0., width: 220., height: 100. }, 0., |f| {
            TV_JUST.render(f, Rect { x: 0., y: 0., width: 220., height: 100. }, &mut state);
        });
        let j_link = ui_j.semantics().iter().find(|s| s.role == Role::Link && s.label.as_ref() == "https://bhoo.dev").expect("justified link missing").bounds;
        assert!(cj.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color, .. }
            if *color == link_col && (bounds.x - j_link.x).abs() < 0.5 && (bounds.width - j_link.width).abs() < 0.5)));
    }
    state.rtl = false;

    // 4. Kind::Edit highlights links, underlines them, clicks through to last_clicked_link; password field suppresses links
    let mut ui_ed = Ui::default();
    state.last_clicked_link = None;
    let ed_cmds = ui_ed.frame(Input::default(), b, 0., |f| { EDIT_LINK.render(f, b, &mut state); });
    let ed_link = ui_ed.semantics().iter().find(|s| s.role == Role::Link && s.label.as_ref() == "ops@bhoo.dev").expect("EditText link missing").bounds;
    assert!(ed_cmds.iter().any(|c| matches!(c, DrawCommand::Text { text, color, .. } if text.as_ref() == "ops@bhoo.dev" && *color == link_col)));
    assert!(ed_cmds.iter().any(|c| matches!(c, DrawCommand::RoundedRect { bounds, color, .. }
        if *color == link_col && (bounds.height - 1.0).abs() < 0.05 && (bounds.x - ed_link.x).abs() < 0.5)));
    let p_ed = [ed_link.x + ed_link.width * 0.5, ed_link.y + ed_link.height * 0.5];
    let mut ed_action = None;
    ui_ed.frame(Input { pointer: Some(p_ed), pressed: true, time: 0.1, ..Input::default() }, b, 0., |f| { ed_action = EDIT_LINK.render(f, b, &mut state); });
    ui_ed.frame(Input { pointer: Some(p_ed), released: true, time: 0.15, ..Input::default() }, b, 0., |f| { ed_action = EDIT_LINK.render(f, b, &mut state); });
    assert_eq!(ed_action, Some("ed_link"));
    assert_eq!(state.last_clicked_link.as_deref(), Some("ops@bhoo.dev"));

    let mut ui_pw = Ui::default();
    ui_pw.frame(Input::default(), b, 0., |f| { EDIT_PASS.render(f, b, &mut state); });
    assert!(ui_pw.semantics().iter().all(|s| s.role != Role::Link), "password EditText must not linkify masked bullets");
}

#[test]
fn wave24a_animated_vector_src_repeat_oneshot_clip_gating_rtl_mirror_and_reduced_motion() {
    use bhoo::android_xml::{RULE_ANIMATED_VECTOR, RULE_AUTO_MIRRORED};

    static REP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ANIMATED_VECTOR, anchor: "src:1:600|avd_0,avd_1,avd_2" },
        RelativeRule { kind: RULE_AUTO_MIRRORED, anchor: "true" },
    ];
    static ONCE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ANIMATED_VECTOR, anchor: "src:0:600|avd_0,avd_1,avd_2" },
    ];

    static AVD_IMG: Node = Node {
        kind: Kind::Image,
        relative_rules: REP_RULES,
        ..button("avd_img", Visibility::Visible, Size::Wrap, 0.)
    };
    static AVD_ICON: Node = Node {
        kind: Kind::Icon,
        relative_rules: ONCE_RULES,
        ..button("avd_icon", Visibility::Visible, Size::Match, 0.)
    };
    static AVD_ZOOM: Node = Node {
        kind: Kind::Zoom,
        relative_rules: ONCE_RULES,
        ..button("avd_zoom", Visibility::Visible, Size::Match, 0.)
    };

    let img0 = Image::rgba(16, 12, vec![255, 0, 0, 255].repeat(16 * 12)).unwrap();
    let img1 = Image::rgba(16, 12, vec![0, 255, 0, 255].repeat(16 * 12)).unwrap();
    let img2 = Image::rgba(16, 12, vec![0, 0, 255, 255].repeat(16 * 12)).unwrap();
    let (id0, id1, id2) = (img0.id(), img1.id(), img2.id());

    let mut state = LayoutState::default();
    state.images.insert("avd_0".into(), img0);
    state.images.insert("avd_1".into(), img1);
    state.images.insert("avd_2".into(), img2);

    let b = Rect { x: 0., y: 0., width: 64., height: 64. };
    let drawn_img_id = |cmds: &[DrawCommand]| -> Option<u32> {
        cmds.iter().find_map(|c| match c {
            DrawCommand::Image { image, .. } => Some(image.id()),
            _ => None,
        })
    };

    // 1. Repeating src AVD (600ms / 3 = 200ms per frame): wraps modulo 600ms
    let mut ui = Ui::default();
    let c0 = ui.frame(Input { time: 0.05, ..Input::default() }, b, 0., |f| { AVD_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&c0), Some(id0));
    assert!((ui.next_wakeup().unwrap() - 0.2).abs() < 1e-6);
    assert!(!c0.iter().any(|c| matches!(c, DrawCommand::PushTransform { scale, .. } if scale[0] < 0.0)), "LTR must not mirror");

    let c1 = ui.frame(Input { time: 0.25, ..Input::default() }, b, 0., |f| { AVD_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&c1), Some(id1));
    assert!((ui.next_wakeup().unwrap() - 0.4).abs() < 1e-6);

    let c2 = ui.frame(Input { time: 0.45, ..Input::default() }, b, 0., |f| { AVD_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&c2), Some(id2));
    assert!((ui.next_wakeup().unwrap() - 0.6).abs() < 1e-6);

    let c_wrap = ui.frame(Input { time: 0.65, ..Input::default() }, b, 0., |f| { AVD_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&c_wrap), Some(id0), "repeat=1 AVD must wrap modulo dur_ms");
    assert!((ui.next_wakeup().unwrap() - 0.8).abs() < 1e-6);

    // 2. RTL autoMirrored pushes horizontal flip [-1.0, 1.0]
    state.rtl = true;
    let c_rtl = ui.frame(Input { time: 0.05, ..Input::default() }, b, 0., |f| { AVD_IMG.render(f, b, &mut state); });
    assert!(c_rtl.iter().any(|c| matches!(c, DrawCommand::PushTransform { scale, .. } if (scale[0] + 1.0).abs() < 1e-5 && (scale[1] - 1.0).abs() < 1e-5)),
        "RTL autoMirrored AVD must emit horizontal flip transform");
    state.rtl = false;

    // 3. Clip-gated wakeup: off-screen AVD schedules NO wake_at
    let mut ui_clip = Ui::default();
    let offscreen = Rect { x: 500., y: 500., width: 64., height: 64. };
    ui_clip.frame(Input { time: 0.05, ..Input::default() }, b, 0., |f| { AVD_IMG.render(f, offscreen, &mut state); });
    assert_eq!(ui_clip.next_wakeup(), None, "off-screen clipped AVD must not schedule wakeup");

    // 4. Non-repeating src AVD on Kind::Icon and Kind::Zoom clamps on final frame with no wakeup
    let mut ui_once = Ui::default();
    let o0 = ui_once.frame(Input { time: 0.05, ..Input::default() }, b, 0., |f| { AVD_ICON.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&o0), Some(id0));
    assert!((ui_once.next_wakeup().unwrap() - 0.2).abs() < 1e-6);

    let o1 = ui_once.frame(Input { time: 0.25, ..Input::default() }, b, 0., |f| { AVD_ICON.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&o1), Some(id1));
    assert!((ui_once.next_wakeup().unwrap() - 0.4).abs() < 1e-6);

    let o2 = ui_once.frame(Input { time: 0.45, ..Input::default() }, b, 0., |f| { AVD_ICON.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&o2), Some(id2));
    assert_eq!(ui_once.next_wakeup(), None, "repeat=0 on final frame must stop scheduling wakes");

    let o_done = ui_once.frame(Input { time: 2.0, ..Input::default() }, b, 0., |f| { AVD_ZOOM.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&o_done), Some(id2), "Kind::Zoom repeat=0 past dur_ms must clamp on final frame");
    assert_eq!(ui_once.next_wakeup(), None);

    // 5. Reduced motion: repeat=0 settles on index len-1 (id2); repeat=1 settles on frame 0 (id0); zero wakeups
    let mut ui_rm = Ui::default();
    ui_rm.motion_preference = MotionPreference::Reduced;
    let rm_once = ui_rm.frame(Input { time: 0.01, ..Input::default() }, b, 0., |f| { AVD_ICON.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&rm_once), Some(id2), "reduced motion repeat=0 must settle directly on final frame");
    assert_eq!(ui_rm.next_wakeup(), None);

    let rm_rep = ui_rm.frame(Input { time: 0.45, ..Input::default() }, b, 0., |f| { AVD_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_img_id(&rm_rep), Some(id0), "reduced motion repeat=1 must settle directly on frame 0");
    assert_eq!(ui_rm.next_wakeup(), None);
}

#[test]
fn wave24a_animated_vector_bg_and_dual_src_bg_repeatable_rules() {
    use bhoo::android_xml::{RULE_ANIMATED_VECTOR, RULE_AUTO_MIRRORED};

    static BG_REP_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ANIMATED_VECTOR, anchor: "bg:1:400|bg_0,bg_1" },
        RelativeRule { kind: RULE_AUTO_MIRRORED, anchor: "true" },
    ];
    static BG_ONCE_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ANIMATED_VECTOR, anchor: "bg:0:400|bg_0,bg_1" },
    ];
    static DUAL_RULES: &[RelativeRule] = &[
        RelativeRule { kind: RULE_ANIMATED_VECTOR, anchor: "bg:1:400|bg_0,bg_1" },
        RelativeRule { kind: RULE_ANIMATED_VECTOR, anchor: "src:0:600|avd_0,avd_1,avd_2" },
    ];

    static BG_REP_VIEW: Node = Node {
        kind: Kind::View,
        relative_rules: BG_REP_RULES,
        ..button("avd_bg_rep", Visibility::Visible, Size::Match, 0.)
    };
    static BG_ONCE_VIEW: Node = Node {
        kind: Kind::Frame,
        relative_rules: BG_ONCE_RULES,
        ..button("avd_bg_once", Visibility::Visible, Size::Match, 0.)
    };
    static DUAL_IMG: Node = Node {
        kind: Kind::Image,
        relative_rules: DUAL_RULES,
        ..button("avd_dual", Visibility::Visible, Size::Match, 0.)
    };

    let bg0 = Image::rgba(8, 8, vec![10, 20, 30, 255].repeat(64)).unwrap();
    let bg1 = Image::rgba(8, 8, vec![40, 50, 60, 255].repeat(64)).unwrap();
    let avd0 = Image::rgba(8, 8, vec![200, 10, 10, 255].repeat(64)).unwrap();
    let avd1 = Image::rgba(8, 8, vec![10, 200, 10, 255].repeat(64)).unwrap();
    let avd2 = Image::rgba(8, 8, vec![10, 10, 200, 255].repeat(64)).unwrap();
    let (id_bg0, id_bg1, id_avd0, id_avd1, id_avd2) = (bg0.id(), bg1.id(), avd0.id(), avd1.id(), avd2.id());

    let mut state = LayoutState::default();
    state.images.insert("bg_0".into(), bg0);
    state.images.insert("bg_1".into(), bg1);
    state.images.insert("avd_0".into(), avd0);
    state.images.insert("avd_1".into(), avd1);
    state.images.insert("avd_2".into(), avd2);

    let b = Rect { x: 0., y: 0., width: 80., height: 80. };
    let drawn_ids = |cmds: &[DrawCommand]| -> Vec<u32> {
        cmds.iter().filter_map(|c| match c {
            DrawCommand::Image { image, .. } => Some(image.id()),
            _ => None,
        }).collect()
    };

    // 1. Background repeating AVD wraps modulo 400ms and mirrors in RTL
    let mut ui = Ui::default();
    let b0 = ui.frame(Input { time: 0.05, ..Input::default() }, b, 0., |f| { BG_REP_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&b0), vec![id_bg0]);
    assert!((ui.next_wakeup().unwrap() - 0.2).abs() < 1e-6);

    let b1 = ui.frame(Input { time: 0.25, ..Input::default() }, b, 0., |f| { BG_REP_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&b1), vec![id_bg1]);
    assert!((ui.next_wakeup().unwrap() - 0.4).abs() < 1e-6);

    state.rtl = true;
    let b_wrap_rtl = ui.frame(Input { time: 0.45, ..Input::default() }, b, 0., |f| { BG_REP_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&b_wrap_rtl), vec![id_bg0]);
    assert!(b_wrap_rtl.iter().any(|c| matches!(c, DrawCommand::PushTransform { scale, .. } if (scale[0] + 1.0).abs() < 1e-5)));
    state.rtl = false;

    // 2. Background non-repeating AVD clamps on final frame with no wakeup, and under reduced motion settles on final frame
    let mut ui_once = Ui::default();
    let bo = ui_once.frame(Input { time: 0.25, ..Input::default() }, b, 0., |f| { BG_ONCE_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&bo), vec![id_bg1]);
    assert_eq!(ui_once.next_wakeup(), None);

    ui_once.motion_preference = MotionPreference::Reduced;
    let bo_rm = ui_once.frame(Input { time: 0.0, ..Input::default() }, b, 0., |f| { BG_ONCE_VIEW.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&bo_rm), vec![id_bg1]);
    assert_eq!(ui_once.next_wakeup(), None);

    // 3. Dual repeatable bg + src RULE_ANIMATED_VECTOR on the same Image node
    let mut ui_dual = Ui::default();
    let d_mid = ui_dual.frame(Input { time: 0.25, ..Input::default() }, b, 0., |f| { DUAL_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&d_mid), vec![id_bg1, id_avd1], "both bg and src AVD frames must paint simultaneously");
    assert!((ui_dual.next_wakeup().unwrap() - 0.4).abs() < 1e-6);

    let d_late = ui_dual.frame(Input { time: 0.45, ..Input::default() }, b, 0., |f| { DUAL_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&d_late), vec![id_bg0, id_avd2], "bg wraps to frame 0 while src clamps on final frame");

    let mut ui_dual_rm = Ui::default();
    ui_dual_rm.motion_preference = MotionPreference::Reduced;
    let d_rm = ui_dual_rm.frame(Input { time: 0.25, ..Input::default() }, b, 0., |f| { DUAL_IMG.render(f, b, &mut state); });
    assert_eq!(drawn_ids(&d_rm), vec![id_bg0, id_avd2], "reduced motion settles repeating bg on frame 0 and non-repeating src on final frame");
    assert_eq!(ui_dual_rm.next_wakeup(), None);
    let _ = id_avd0;
}
