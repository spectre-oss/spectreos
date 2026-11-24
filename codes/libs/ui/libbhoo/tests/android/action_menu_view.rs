use bhoo::*;
use bhoo::android_xml::{AndroidToolbarMenuItem as Item,AndroidShowAsAction as Policy,LayoutState};
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_action_menu_view.rs"));}
fn item(id:u64,label:&str,policy:Policy,enabled:bool)->Item {Item{id,group_id:0,label:label.into(),icon:None,show_text_with_icon:false,icon_drawable:None,show_as_action:policy,enabled,visible:true,checkable:false,checked:false,exclusive:false}}
fn draw(ui:&mut Ui,state:&mut LayoutState,input:Input,bounds:Rect)->Option<&'static str>{let mut action=None;ui.frame(input,Rect{x:0.,y:0.,width:320.,height:400.},0.,|f|action=fixture::LAYOUT.render(f,bounds,state));action}
fn click(ui:&mut Ui,state:&mut LayoutState,bounds:Rect,label:&str,time:f64)->Option<&'static str>{let r=ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;let pointer=Some([r.x+r.width/2.,r.y+r.height/2.]);draw(ui,state,Input{pointer,pressed:true,time,..Input::default()},bounds);draw(ui,state,Input{pointer,released:true,time:time+0.01,..Input::default()},bounds)}
#[test] fn actions_use_56dp_cells_and_report_clicks(){let mut ui=Ui::default();let mut state=LayoutState::default();let mut first=item(1,"First",Policy::IfRoom,true);first.icon=Some(MaterialIcon::Search);let mut second=item(2,"Second",Policy::IfRoom,true);second.icon=Some(MaterialIcon::Search);state.toolbar_menu_items.insert("actions".into(),vec![first,second]);let b=Rect{x:0.,y:0.,width:112.,height:56.};draw(&mut ui,&mut state,Input::default(),b);assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="More options"));assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="First").unwrap().bounds.width,56.);assert_eq!(click(&mut ui,&mut state,b,"First",1.),Some("actions"));assert_eq!(state.toolbar_menu_clicked["actions"],1);}
#[test] fn overflow_reports_selection_and_disabled_items_do_not_activate(){let mut ui=Ui::default();let mut state=LayoutState::default();state.toolbar_menu_items.insert("actions".into(),vec![item(1,"First",Policy::IfRoom,true),item(2,"More",Policy::Never,true),item(3,"Disabled",Policy::Never,false)]);let b=Rect{x:0.,y:0.,width:168.,height:56.};draw(&mut ui,&mut state,Input::default(),b);click(&mut ui,&mut state,b,"More options",1.);draw(&mut ui,&mut state,Input{time:1.1,..Input::default()},b);assert!(!ui.semantics().iter().find(|n|n.label.as_ref()=="Disabled").unwrap().enabled);assert_eq!(click(&mut ui,&mut state,b,"More",1.2),Some("actions"));assert_eq!(state.toolbar_menu_clicked["actions"],2);}
#[test] fn rtl_places_overflow_on_the_left(){let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=true;state.toolbar_menu_items.insert("actions".into(),vec![item(1,"First",Policy::IfRoom,true),item(2,"More",Policy::Never,true)]);let b=Rect{x:0.,y:0.,width:168.,height:56.};draw(&mut ui,&mut state,Input::default(),b);assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="More options").unwrap().bounds.x,0.);}
#[test] fn action_items_spread_across_available_width_and_mirror_in_rtl(){
    let b=Rect{x:0.,y:0.,width:280.,height:56.};
    for rtl in [false,true] {let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;
        let mut first=item(1,"First",Policy::IfRoom,true);first.icon=Some(MaterialIcon::Search);
        let mut second=item(2,"Second",Policy::IfRoom,true);second.icon=Some(MaterialIcon::Search);
        state.toolbar_menu_items.insert("actions".into(),vec![first,second,item(3,"Overflow",Policy::Never,true)]);
        draw(&mut ui,&mut state,Input::default(),b);
        let rect=|label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
        assert_eq!(rect("First").x,if rtl{224.}else{0.});assert_eq!(rect("Second").x,112.);assert_eq!(rect("More options").x,if rtl{0.}else{224.});
    }
}
#[test] fn single_action_is_centered_and_cells_absorb_remainder(){
    let mut ui=Ui::default();let mut state=LayoutState::default();let mut entry=item(1,"Only",Policy::IfRoom,true);entry.icon=Some(MaterialIcon::Search);state.toolbar_menu_items.insert("actions".into(),vec![entry]);
    let b=Rect{x:0.,y:0.,width:230.,height:56.};draw(&mut ui,&mut state,Input::default(),b);
    let r=ui.semantics().iter().find(|n|n.label.as_ref()=="Only").unwrap().bounds;
    assert!((r.width-230.).abs()<0.01);assert!((r.x+r.width/2.-115.).abs()<0.01);
}
#[test] fn text_action_needs_two_cells_and_moves_to_overflow_when_narrow(){
    let mut ui=Ui::default();let mut state=LayoutState::default();let mut icon=item(2,"Icon",Policy::IfRoom,true);icon.icon=Some(MaterialIcon::Search);
    state.toolbar_menu_items.insert("actions".into(),vec![item(1,"Save",Policy::IfRoom,true),icon]);
    let wide=Rect{x:0.,y:0.,width:168.,height:56.};draw(&mut ui,&mut state,Input::default(),wide);
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="Save").unwrap().bounds.width,112.);
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Icon"));
    let narrow=Rect{width:112.,..wide};draw(&mut ui,&mut state,Input{time:1.,..Input::default()},narrow);
    assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="Save"));assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="More options"));
    click(&mut ui,&mut state,narrow,"More options",1.1);draw(&mut ui,&mut state,Input{time:1.2,..Input::default()},narrow);
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Save"));
}
#[test] fn with_text_respects_configuration_and_rtl_icon_order(){
    let b=Rect{x:0.,y:0.,width:280.,height:56.};
    for (allowed,rtl) in [(false,false),(true,false),(true,true)] {
        let mut ui=Ui::default();let mut state=LayoutState::default();state.rtl=rtl;state.menu_allow_text_with_icon=Some(allowed);
        let mut entry=item(1,"Search",Policy::IfRoom,true);entry.icon=Some(MaterialIcon::Search);entry.show_text_with_icon=true;state.toolbar_menu_items.insert("actions".into(),vec![entry]);
        let commands=ui.frame(Input::default(),Rect{x:0.,y:0.,width:320.,height:400.},0.,|f|{fixture::LAYOUT.render(f,b,&mut state);});
        let text=commands.iter().find_map(|c|match c {DrawCommand::Text{text,bounds,..} if text.as_ref()=="Search"=>Some(*bounds),_=>None});
        assert_eq!(text.is_some(),allowed);let target=ui.semantics().iter().find(|n|n.label.as_ref()=="Search").unwrap().bounds;assert_eq!(target.width,280.);
        if let Some(text)=text {let icon=commands.iter().find_map(|c|match c {DrawCommand::Shape{bounds,..}=>Some(*bounds),_=>None}).unwrap();assert_eq!(icon.x>text.x,rtl);}
    }
}
#[test] fn expandable_text_actions_share_remaining_whole_cells(){
    let b=Rect{x:0.,y:0.,width:392.,height:56.};let mut ui=Ui::default();let mut state=LayoutState::default();
    let mut icon=item(3,"Icon",Policy::IfRoom,true);icon.icon=Some(MaterialIcon::Search);
    state.toolbar_menu_items.insert("actions".into(),vec![item(1,"Save",Policy::IfRoom,true),item(2,"Edit",Policy::IfRoom,true),icon]);ui.frame(Input::default(),Rect{x:0.,y:0.,width:392.,height:400.},0.,|f|{fixture::LAYOUT.render(f,b,&mut state);});
    let get=|label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
    assert_eq!(get("Save").width,168.);assert_eq!(get("Edit").width,168.);assert_eq!(get("Icon").width,56.);
}
#[test] fn mixed_required_and_requested_group_moves_together_to_overflow(){
    let mut ui=Ui::default();let mut state=LayoutState::default();let mut first=item(1,"Required",Policy::Always,true);first.group_id=12;first.icon=Some(MaterialIcon::Search);
    let mut second=item(2,"Save",Policy::IfRoom,true);second.group_id=12;
    state.toolbar_menu_items.insert("actions".into(),vec![first,second,item(3,"Other",Policy::Never,true)]);
    let wide=Rect{x:0.,y:0.,width:224.,height:56.};draw(&mut ui,&mut state,Input::default(),wide);assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Required"));assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Save"));
    let narrow=Rect{width:168.,..wide};draw(&mut ui,&mut state,Input{time:1.,..Input::default()},narrow);assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="Required"));assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="Save"));
    click(&mut ui,&mut state,narrow,"More options",1.1);draw(&mut ui,&mut state,Input{time:1.2,..Input::default()},narrow);assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Required"));assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Save"));
}
