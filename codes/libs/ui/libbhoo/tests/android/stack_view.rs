use bhoo::*;
use bhoo::android_xml::LayoutState;
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_stack_view.rs"));}
const B:Rect=Rect{x:0.,y:0.,width:300.,height:220.};
fn draw(ui:&mut Ui,s:&mut LayoutState,input:Input)->Option<&'static str>{let mut action=None;ui.frame(input,B,0.,|f|action=fixture::LAYOUT.render(f,B,s));action}
fn setup()->(Ui,LayoutState){let mut ui=Ui::default();let mut s=LayoutState::default();s.stack_looping.insert("stack".into(),true);s.set_list_adapter("stack",vec!["One".into(),"Two".into(),"Three".into()],None,None).unwrap();draw(&mut ui,&mut s,Input::default());(ui,s)}
fn pointer(y:f32,pressed:bool,released:bool,time:f64)->Input {Input{pointer:Some([150.,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()}}
#[test] fn downward_and_upward_swipes_wrap_and_do_not_click(){let(mut ui,mut s)=setup();draw(&mut ui,&mut s,pointer(60.,true,false,1.));draw(&mut ui,&mut s,pointer(125.,false,false,1.1));assert_eq!(draw(&mut ui,&mut s,pointer(125.,false,true,1.2)),Some("stack"));assert_eq!(s.displayed_children["stack"],1);assert!(!s.stack_clicked.contains_key("stack"));draw(&mut ui,&mut s,pointer(130.,true,false,2.));draw(&mut ui,&mut s,pointer(60.,false,false,2.1));draw(&mut ui,&mut s,pointer(60.,false,true,2.2));assert_eq!(s.displayed_children["stack"],0);assert!(s.stack_step("stack",-1));assert_eq!(s.displayed_children["stack"],2);}
#[test] fn short_swipe_and_cancel_keep_selection_and_click_reports_item(){let(mut ui,mut s)=setup();draw(&mut ui,&mut s,pointer(80.,true,false,1.));draw(&mut ui,&mut s,pointer(96.,false,false,1.1));draw(&mut ui,&mut s,pointer(96.,false,true,1.2));assert_eq!(s.displayed_children["stack"],0);assert!(!s.stack_clicked.contains_key("stack"));draw(&mut ui,&mut s,Input{time:2.,..Input::default()});draw(&mut ui,&mut s,pointer(80.,true,false,2.1));assert_eq!(draw(&mut ui,&mut s,pointer(80.,false,true,2.2)),Some("stack"));assert_eq!(s.stack_clicked["stack"],0);draw(&mut ui,&mut s,pointer(80.,true,false,3.));draw(&mut ui,&mut s,pointer(150.,false,false,3.1));draw(&mut ui,&mut s,Input{cancelled:true,time:3.2,..Input::default()});assert_eq!(s.displayed_children["stack"],0);}
#[test] fn stack_draws_multiple_adapter_items_and_empty_adapter_is_safe(){let(mut ui,mut s)=setup();let commands=ui.frame(Input{time:0.1,..Input::default()},B,0.,|f|{fixture::LAYOUT.render(f,B,&mut s);});for value in ["One","Two","Three"] {assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()==value)));}s.set_list_adapter("stack",vec![],None,None).unwrap();draw(&mut ui,&mut s,Input{time:0.2,..Input::default()});assert!(!s.stack_step("stack",1));assert!(!s.displayed_children.contains_key("stack"));}
#[test] fn upward_swipe_brings_previous_card_in_from_below(){
    let(mut ui,mut state)=setup();draw(&mut ui,&mut state,pointer(130.,true,false,1.));
    let drag=ui.frame(pointer(60.,false,false,1.1),B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});
    let y=|commands:&[DrawCommand],label:&str|commands.iter().rev().find_map(|c|match c {DrawCommand::Text{text,bounds,..} if text.as_ref()==label=>Some(bounds.y),_=>None}).unwrap();
    let stationary=y(&drag,"One");let incoming=y(&drag,"Three");assert!(incoming>stationary+60.);
    draw(&mut ui,&mut state,pointer(60.,false,true,1.2));assert_eq!(state.displayed_children["stack"],2);
    let during=ui.frame(Input{time:1.21,..Input::default()},B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});let after=ui.frame(Input{time:2.,..Input::default()},B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});assert!(y(&during,"Three")>y(&after,"Three")+40.);
}
#[test] fn programmatic_next_and_previous_animate_in_correct_directions(){
    let(mut ui,mut state)=setup();let text_y=|commands:&[DrawCommand],label:&str|commands.iter().rev().find_map(|c|match c {DrawCommand::Text{text,bounds,..} if text.as_ref()==label=>Some(bounds.y),_=>None}).unwrap();
    assert!(state.stack_step("stack",1));draw(&mut ui,&mut state,Input{time:1.,..Input::default()});let mid=ui.frame(Input{time:1.2,..Input::default()},B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});assert!(text_y(&mid,"One")>text_y(&mid,"Two")+50.);
    draw(&mut ui,&mut state,Input{time:1.5,..Input::default()});assert!(state.stack_step("stack",-1));draw(&mut ui,&mut state,Input{time:2.,..Input::default()});let mid=ui.frame(Input{time:2.2,..Input::default()},B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});let done=ui.frame(Input{time:2.5,..Input::default()},B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});assert!(text_y(&mid,"One")>text_y(&done,"One")+50.);
}
#[test] fn replacing_adapter_cancels_a_pending_press(){let(mut ui,mut state)=setup();draw(&mut ui,&mut state,pointer(80.,true,false,1.));state.set_list_adapter("stack",vec!["Replacement".into()],None,None).unwrap();draw(&mut ui,&mut state,pointer(80.,false,true,1.1));assert!(!state.stack_clicked.contains_key("stack"));}
#[test] fn moving_card_fades_as_it_leaves_and_does_not_jump_on_release(){
    let(mut ui,mut state)=setup();draw(&mut ui,&mut state,pointer(60.,true,false,1.));
    let before=ui.frame(pointer(122.,false,false,1.1),B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});
    let alpha=|commands:&[DrawCommand]|commands.iter().rev().find_map(|c|match c {DrawCommand::Text{text,color,..} if text.as_ref()=="One"=>Some(color.3),_=>None}).unwrap();
    let drag_alpha=alpha(&before);assert!((125..=130).contains(&drag_alpha),"drag alpha {drag_alpha}");
    let release=ui.frame(pointer(122.,false,true,1.2),B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});let release_alpha=alpha(&release);assert!((i16::from(drag_alpha)-i16::from(release_alpha)).abs()<=1);
    let later=ui.frame(Input{time:1.3,..Input::default()},B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});assert!(alpha(&later)<release_alpha);
}
#[test] fn sideways_drag_tracks_a_fraction_and_returns_to_center(){
    let(mut ui,mut state)=setup();let frame=|ui:&mut Ui,state:&mut LayoutState,input|ui.frame(input,B,0.,|f|{fixture::LAYOUT.render(f,B,state);});
    let x=|commands:&[DrawCommand]|commands.iter().rev().find_map(|c|match c {DrawCommand::Text{text,bounds,..} if text.as_ref()=="One"=>Some(bounds.x),_=>None}).unwrap();
    let initial=frame(&mut ui,&mut state,Input{time:0.5,..Input::default()});let baseline=x(&initial);
    frame(&mut ui,&mut state,Input{pointer:Some([100.,80.]),pressed:true,time:1.,..Input::default()});
    let dragged=frame(&mut ui,&mut state,Input{pointer:Some([150.,96.]),time:1.1,..Input::default()});assert!((x(&dragged)-baseline-10.).abs()<0.01);
    let released=frame(&mut ui,&mut state,Input{pointer:Some([150.,96.]),released:true,time:1.2,..Input::default()});assert!((x(&released)-baseline-10.).abs()<0.01);
    let finished=frame(&mut ui,&mut state,Input{time:2.,..Input::default()});assert!((x(&finished)-baseline).abs()<0.01);
}
#[test] fn swipe_tilt_follows_android_rotation_curve(){
    let(mut ui,mut state)=setup();draw(&mut ui,&mut state,pointer(60.,true,false,1.));
    let early=ui.frame(pointer(80.,false,false,1.1),B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});assert!(!early.iter().any(|c|matches!(c,DrawCommand::PushScaleY{..})));
    let tilted=ui.frame(pointer(122.,false,false,1.2),B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});let scale=tilted.iter().find_map(|c|match c{DrawCommand::PushScaleY{scale,..}=>Some(*scale),_=>None}).unwrap();
    let progress=54.0_f32/(220.0*0.7);let expected=(((progress-0.2)/0.8)*std::f32::consts::FRAC_PI_2).cos();assert!((scale-expected).abs()<0.0001);
    assert_eq!(tilted.iter().filter(|c|matches!(c,DrawCommand::PushScaleY{..})).count(),tilted.iter().filter(|c|matches!(c,DrawCommand::PopScaleY)).count());
}
#[test] fn default_stack_stops_at_ends_and_loop_option_wraps(){
    let mut ui=Ui::default();let mut state=LayoutState::default();state.set_list_adapter("stack",vec!["One".into(),"Two".into()],None,None).unwrap();draw(&mut ui,&mut state,Input::default());
    assert!(!state.stack_step("stack",-1));assert_eq!(state.displayed_children["stack"],0);
    draw(&mut ui,&mut state,pointer(130.,true,false,1.));draw(&mut ui,&mut state,pointer(40.,false,false,1.1));draw(&mut ui,&mut state,pointer(40.,false,true,1.2));assert_eq!(state.displayed_children["stack"],0);
    assert!(state.stack_step("stack",1));assert!(!state.stack_step("stack",1));assert_eq!(state.displayed_children["stack"],1);draw(&mut ui,&mut state,Input{time:2.,..Input::default()});
    draw(&mut ui,&mut state,pointer(50.,true,false,2.1));let edge=ui.frame(pointer(160.,false,false,2.2),B,0.,|f|{fixture::LAYOUT.render(f,B,&mut state);});let alpha=edge.iter().find_map(|c|match c{DrawCommand::Text{text,color,..} if text.as_ref()=="Two"=>Some(color.3),_=>None}).unwrap();assert_eq!(alpha,255);draw(&mut ui,&mut state,pointer(160.,false,true,2.3));assert_eq!(state.displayed_children["stack"],1);
    state.stack_looping.insert("stack".into(),true);assert!(state.stack_step("stack",1));assert_eq!(state.displayed_children["stack"],0);
}
#[test] fn xml_loop_views_enables_wrap(){
    use bhoo::android_xml::RelativeRule;
    static LOOPING:android_xml::Node=android_xml::Node {relative_rules:&[RelativeRule{kind:137,anchor:"true"}],..fixture::LAYOUT};
    let mut ui=Ui::default();let mut state=LayoutState::default();state.set_list_adapter("stack",vec!["One".into(),"Two".into()],None,None).unwrap();ui.frame(Input::default(),B,0.,|f|{LOOPING.render(f,B,&mut state);});assert!(state.stack_looping["stack"]);assert!(state.stack_step("stack",-1));assert_eq!(state.displayed_children["stack"],1);
}
#[test] fn mouse_wheel_advances_with_android_pacing_and_respects_bounds(){
    let(mut ui,mut state)=setup();state.stack_looping.insert("stack".into(),false);
    let wheel=|delta,time|Input{pointer:Some([100.,100.]),scroll:[0.,delta],time,..Input::default()};
    assert_eq!(draw(&mut ui,&mut state,wheel(1.,1.)),Some("stack"));assert_eq!(state.displayed_children["stack"],1);
    assert_eq!(draw(&mut ui,&mut state,wheel(1.,1.05)),None);assert_eq!(state.displayed_children["stack"],1);
    assert_eq!(draw(&mut ui,&mut state,wheel(1.,1.11)),Some("stack"));assert_eq!(state.displayed_children["stack"],2);
    assert_eq!(draw(&mut ui,&mut state,wheel(1.,1.22)),None);assert_eq!(state.displayed_children["stack"],2);
    assert_eq!(draw(&mut ui,&mut state,wheel(-1.,1.33)),Some("stack"));assert_eq!(state.displayed_children["stack"],1);
    assert_eq!(draw(&mut ui,&mut state,Input{pointer:Some([310.,100.]),scroll:[0.,1.],time:1.5,..Input::default()}),None);assert_eq!(state.displayed_children["stack"],1);
}
#[test] fn touch_slop_is_removed_and_gesture_direction_stays_locked(){
    let(mut ui,mut state)=setup();let frame=|ui:&mut Ui,state:&mut LayoutState,input|ui.frame(input,B,0.,|f|{fixture::LAYOUT.render(f,B,state);});
    let y=|commands:&[DrawCommand]|commands.iter().rev().find_map(|c|match c{DrawCommand::Text{text,bounds,..} if text.as_ref()=="One"=>Some(bounds.y),_=>None}).unwrap();
    let base=frame(&mut ui,&mut state,Input{time:0.5,..Input::default()});let base_y=y(&base);frame(&mut ui,&mut state,pointer(100.,true,false,1.));
    let start=frame(&mut ui,&mut state,pointer(109.,false,false,1.1));assert!((y(&start)-base_y-1.).abs()<0.01);
    frame(&mut ui,&mut state,pointer(40.,false,false,1.2));frame(&mut ui,&mut state,pointer(40.,false,true,1.3));assert_eq!(state.displayed_children["stack"],0);
}
#[test] fn release_position_is_used_for_distance_threshold(){let(mut ui,mut state)=setup();draw(&mut ui,&mut state,pointer(60.,true,false,1.));draw(&mut ui,&mut state,pointer(80.,false,false,1.1));draw(&mut ui,&mut state,pointer(95.,false,true,1.2));assert_eq!(state.displayed_children["stack"],1);}
#[test] fn taps_in_exposed_stack_margins_do_not_click_but_swipes_still_work(){
    let(mut ui,mut state)=setup();
    let point=|x,y,pressed,released,time|Input{pointer:Some([x,y]),pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,point(100.,10.,true,false,1.));assert_eq!(draw(&mut ui,&mut state,point(100.,10.,false,true,1.1)),None);
    draw(&mut ui,&mut state,point(290.,80.,true,false,1.2));assert_eq!(draw(&mut ui,&mut state,point(290.,80.,false,true,1.3)),None);assert!(!state.stack_clicked.contains_key("stack"));
    draw(&mut ui,&mut state,point(100.,10.,true,false,2.));draw(&mut ui,&mut state,point(100.,80.,false,false,2.1));assert_eq!(draw(&mut ui,&mut state,point(100.,80.,false,true,2.2)),Some("stack"));assert_eq!(state.displayed_children["stack"],1);assert!(!state.stack_clicked.contains_key("stack"));
}
#[test] fn keyboard_activation_still_clicks_current_item(){let(mut ui,mut state)=setup();ui.adapter_focus(Some(android_xml::view_id("stack")));let key=KeyInput{key:Key::Enter,pressed:true,repeat:false,shift:false};assert_eq!(draw(&mut ui,&mut state,Input{key:Some(key),time:1.,..Input::default()}),Some("stack"));assert_eq!(state.stack_clicked["stack"],0);}
