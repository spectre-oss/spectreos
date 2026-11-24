use bhoo::{android_xml::{self,LayoutState},*};
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_radio_group_nested.rs"));}
const B:Rect=Rect{x:0.,y:0.,width:300.,height:96.};
fn draw(ui:&mut Ui,state:&mut LayoutState,input:Input){ui.frame(input,B,0.,|f|{fixture::LAYOUT.render(f,B,state);});}
fn tap(ui:&mut Ui,state:&mut LayoutState,label:&str,time:f64){
    let b=ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;let p=Some([b.x+b.width/2.,b.y+b.height/2.]);
    draw(ui,state,Input{pointer:p,pressed:true,time,..Input::default()});draw(ui,state,Input{pointer:p,released:true,time:time+0.1,..Input::default()});
}
#[test] fn nested_initial_reference_and_direct_child_tracking_follow_android(){
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());draw(&mut ui,&mut state,Input::default());
    assert_eq!(state.checked.get("nested_choice"),Some(&true));assert_eq!(state.checked.get("direct_choice"),Some(&false));
    assert_eq!(state.radio_groups[&android_xml::view_id("nested_group")],Some(android_xml::view_id("nested_choice")));
    tap(&mut ui,&mut state,"Direct choice",1.);
    assert_eq!(state.checked.get("nested_choice"),Some(&false));assert_eq!(state.checked.get("direct_choice"),Some(&true));
    assert_eq!(state.radio_groups[&android_xml::view_id("nested_group")],Some(android_xml::view_id("direct_choice")));
    tap(&mut ui,&mut state,"Nested choice",2.);
    assert_eq!(state.checked.get("nested_choice"),Some(&true));assert_eq!(state.checked.get("direct_choice"),Some(&true));
    assert_eq!(state.radio_groups[&android_xml::view_id("nested_group")],Some(android_xml::view_id("direct_choice")));
}
#[test] fn direct_checked_child_takes_precedence_over_nested_reference(){
    use android_xml::Node;
    static CHILDREN:[Node;2]=[fixture::LAYOUT.children[0],Node{checked:true,..fixture::LAYOUT.children[1]}];
    static GROUP:Node=Node{children:&CHILDREN,..fixture::LAYOUT};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());ui.frame(Input::default(),B,0.,|f|{GROUP.render(f,B,&mut state);});
    assert_eq!(state.checked.get("nested_choice"),Some(&false));assert_eq!(state.checked.get("direct_choice"),Some(&true));
}
#[test] fn host_can_check_clear_and_replace_nested_selection(){
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());
    state.set_radio_group_checked("nested_group",Some("direct_choice"));draw(&mut ui,&mut state,Input::default());
    assert_eq!(state.checked.get("nested_choice"),Some(&false));assert_eq!(state.checked.get("direct_choice"),Some(&true));
    state.set_radio_group_checked("nested_group",Some("nested_choice"));draw(&mut ui,&mut state,Input::default());
    assert_eq!(state.checked.get("nested_choice"),Some(&true));assert_eq!(state.checked.get("direct_choice"),Some(&false));
    assert_eq!(state.radio_groups[&android_xml::view_id("nested_group")],Some(android_xml::view_id("nested_choice")));
    state.set_radio_group_checked("nested_group",None);draw(&mut ui,&mut state,Input::default());
    assert_eq!(state.checked.get("nested_choice"),Some(&false));assert_eq!(state.checked.get("direct_choice"),Some(&false));assert_eq!(state.radio_groups[&android_xml::view_id("nested_group")],None);
    state.set_radio_group_checked("nested_group",Some("missing"));draw(&mut ui,&mut state,Input::default());
    assert_eq!(state.radio_groups[&android_xml::view_id("nested_group")],Some(android_xml::view_id("missing")));assert_eq!(state.checked.get("direct_choice"),Some(&false));
    tap(&mut ui,&mut state,"Direct choice",3.);assert_eq!(state.checked.get("direct_choice"),Some(&true));
}
#[test] fn direct_selection_clears_another_programmatically_selected_descendant(){
    use android_xml::Node;
    static RADIOS:[Node;2]=[fixture::LAYOUT.children[0].children[0],Node{id:"nested_other",text:"Other nested",..fixture::LAYOUT.children[0].children[0]}];
    static CHILDREN:[Node;2]=[Node{height:android_xml::Size::Dp(96.),children:&RADIOS,..fixture::LAYOUT.children[0]},fixture::LAYOUT.children[1]];
    static GROUP:Node=Node{height:android_xml::Size::Dp(144.),children:&CHILDREN,..fixture::LAYOUT};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());let bounds=Rect{height:144.,..B};
    let render=|ui:&mut Ui,state:&mut LayoutState,input:Input|{ui.frame(input,bounds,0.,|f|{GROUP.render(f,bounds,state);});};
    state.set_radio_group_checked("nested_group",Some("nested_other"));render(&mut ui,&mut state,Input::default());
    assert_eq!(state.checked.get("nested_other"),Some(&true));assert_eq!(state.checked.get("nested_choice"),Some(&false));
    let b=ui.semantics().iter().find(|n|n.label.as_ref()=="Direct choice").unwrap().bounds;let p=Some([b.x+b.width/2.,b.y+b.height/2.]);
    render(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});render(&mut ui,&mut state,Input{pointer:p,released:true,time:1.1,..Input::default()});
    assert_eq!(state.checked.get("nested_other"),Some(&false));assert_eq!(state.checked.get("direct_choice"),Some(&true));
}
#[test] fn clearing_group_leaves_independent_nested_choice_checked(){
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());draw(&mut ui,&mut state,Input::default());
    tap(&mut ui,&mut state,"Direct choice",1.);tap(&mut ui,&mut state,"Nested choice",2.);
    state.set_radio_group_checked("nested_group",None);draw(&mut ui,&mut state,Input{time:3.,..Input::default()});
    assert_eq!(state.checked.get("nested_choice"),Some(&true));assert_eq!(state.checked.get("direct_choice"),Some(&false));
    assert_eq!(state.radio_groups[&android_xml::view_id("nested_group")],None);
}
#[test] fn group_events_follow_android_check_and_clear_rules(){
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());draw(&mut ui,&mut state,Input::default());
    let group="nested_group";let nested=android_xml::view_id("nested_choice");let direct=android_xml::view_id("direct_choice");
    assert!(state.take_radio_group_events(group).is_empty());
    tap(&mut ui,&mut state,"Direct choice",1.);assert_eq!(state.take_radio_group_events(group),vec![Some(direct)]);
    tap(&mut ui,&mut state,"Direct choice",2.);assert!(state.take_radio_group_events(group).is_empty());
    tap(&mut ui,&mut state,"Nested choice",3.);assert!(state.take_radio_group_events(group).is_empty());
    state.set_radio_group_checked(group,Some("nested_choice"));draw(&mut ui,&mut state,Input{time:4.,..Input::default()});assert_eq!(state.take_radio_group_events(group),vec![Some(nested)]);
    state.set_radio_group_checked(group,Some("nested_choice"));draw(&mut ui,&mut state,Input{time:5.,..Input::default()});assert!(state.take_radio_group_events(group).is_empty());
    for time in [6.,7.] {state.set_radio_group_checked(group,None);draw(&mut ui,&mut state,Input{time,..Input::default()});assert_eq!(state.take_radio_group_events(group),vec![None]);}
    state.set_radio_group_checked(group,Some("missing"));draw(&mut ui,&mut state,Input{time:8.,..Input::default()});assert_eq!(state.take_radio_group_events(group),vec![Some(android_xml::view_id("missing"))]);
}
#[test] fn radio_group_event_queue_keeps_latest_32(){
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());draw(&mut ui,&mut state,Input::default());
    for i in 0..40 {state.set_radio_group_checked("nested_group",Some(if i%2==0{"direct_choice"}else{"nested_choice"}));draw(&mut ui,&mut state,Input{time:i as f64+1.,..Input::default()});}
    let events=state.take_radio_group_events("nested_group");assert_eq!(events.len(),32);assert_eq!(events[0],Some(android_xml::view_id("direct_choice")));assert_eq!(events[31],Some(android_xml::view_id("nested_choice")));assert!(state.take_radio_group_events("nested_group").is_empty());
}
