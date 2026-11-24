use bhoo::*;
mod fixture { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_listview.rs")); }
const VIEW:Rect=Rect{x:0.,y:0.,width:300.,height:220.};
const BOUNDS:Rect=Rect{x:0.,y:0.,width:300.,height:200.};
fn draw(ui:&mut Ui,state:&mut android_xml::LayoutState,input:Input)->Option<&'static str>{let mut action=None;ui.frame(input,VIEW,0.,|f|{action=fixture::LAYOUT.render(f,BOUNDS,state);});action}
fn click(ui:&mut Ui,state:&mut android_xml::LayoutState,y:f32,t:f64)->Option<&'static str>{draw(ui,state,Input {pointer:Some([40.,y]),pressed:true,time:t,..Input::default()});draw(ui,state,Input {pointer:Some([40.,y]),released:true,time:t+0.01,..Input::default()})}
#[test]
fn xml_entries_multiple_choice_disabled_and_dividers() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();draw(&mut ui,&mut state,Input::default());
    assert_eq!(state.list_items["workspaces_list"].len(),13);
    assert_eq!(click(&mut ui,&mut state,24.,0.1),Some("workspaces_list"));
    assert_eq!(click(&mut ui,&mut state,73.,0.2),Some("workspaces_list"));
    assert_eq!(state.list_checked["workspaces_list"],vec![0,1]);
    assert_eq!(click(&mut ui,&mut state,24.,0.3),Some("workspaces_list"));assert_eq!(state.list_checked["workspaces_list"],vec![1]);
    state.list_enabled.insert("workspaces_list".into(),vec![true,false]);
    assert_eq!(click(&mut ui,&mut state,73.,0.4),None);assert_eq!(state.list_checked["workspaces_list"],vec![1]);
    assert_eq!(state.list_clicked["workspaces_list"],0);
}
#[test]
fn single_choice_replaces_old_check_and_none_does_not_check() {
    static SINGLE:android_xml::Node=android_xml::Node{relative_rules:&[android_xml::RelativeRule {kind:88,anchor:"singleChoice"}],..fixture::LAYOUT};
    static NONE:android_xml::Node=android_xml::Node{relative_rules:&[],..fixture::LAYOUT};
    for (layout,expected) in [(&SINGLE,vec![1]),(&NONE,vec![])] {
        let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.list_items.insert("workspaces_list".into(),vec!["One".into(),"Two".into()]);
        let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{ui.frame(input,VIEW,0.,|f|{layout.render(f,BOUNDS,state);})};
        draw(&mut ui,&mut state,Input::default());
        for (i,y) in [24.,73.].into_iter().enumerate(){let t=i as f64*0.1+0.1;draw(&mut ui,&mut state,Input {pointer:Some([40.,y]),pressed:true,time:t,..Input::default()});draw(&mut ui,&mut state,Input {pointer:Some([40.,y]),released:true,time:t+0.01,..Input::default()});}
        assert_eq!(state.list_checked["workspaces_list"],expected);
    }
}
#[test]
fn virtual_list_touch_scroll_does_not_activate_and_clamps_after_data_change() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.list_items.insert("workspaces_list".into(),(0..100_000).map(|i|format!("Item {i}")).collect());
    draw(&mut ui,&mut state,Input::default());assert!(ui.semantics().len()<10);
    let touch=|y,pressed,released,time|Input {pointer:Some([100.,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(180.,true,false,0.1));draw(&mut ui,&mut state,touch(40.,false,false,0.2));
    assert_eq!(draw(&mut ui,&mut state,touch(40.,false,true,0.3)),None);assert!(state.list_clicked.is_empty());assert!(state.offsets["workspaces_list"]>50.0);
    state.list_items.insert("workspaces_list".into(),vec!["Only".into()]);draw(&mut ui,&mut state,Input {time:1.0,..Input::default()});
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Only"));
}

mod row_template { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_spinner_dropdown_row.rs")); }
#[test]
fn xml_adapter_rows_wrap_resize_and_keep_item_selection() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let long="A much longer workspace name that wraps across several lines and makes the second XML list row taller than its neighbors";
    state.list_items.insert("workspaces_list".into(),vec!["Short".into(),long.into(),"Last".into()]);
    row_template::install_drawables(&mut state);state.image_scales.insert("switcher_day".into(),0.5);
    state.set_list_item_template("workspaces_list",Some(android_xml::ListItemTemplate {row:&row_template::LAYOUT,text_id:"spinner_title"})).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f| {fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image {..})));
    let short=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds;
    let long_row=ui.semantics().iter().find(|n|n.label.as_ref()==long).unwrap().bounds;
    assert!(long_row.height>short.height,"{} <= {}",long_row.height,short.height);
    assert!((long_row.y-short.y-short.height-1.0).abs()<0.01,"divider must separate rows");
    assert_eq!(click(&mut ui,&mut state,long_row.y+10.,0.1),Some("workspaces_list"));assert_eq!(state.list_checked["workspaces_list"],vec![1]);
    state.image_scales.insert("switcher_day".into(),1.5);draw(&mut ui,&mut state,Input {time:0.2,..Input::default()});
    let resized=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds;
    assert!(resized.height>short.height,"image density must invalidate cached row heights");
}
#[test]
fn list_template_rejects_interactive_children_or_missing_text_binding() {
    static INTERACTIVE:android_xml::Node=android_xml::Node {clickable:true,..row_template::LAYOUT};
    let mut state=android_xml::LayoutState::default();
    assert!(state.set_list_item_template("x",Some(android_xml::ListItemTemplate {row:&INTERACTIVE,text_id:"spinner_title"})).is_err());
    assert!(state.set_list_item_template("x",Some(android_xml::ListItemTemplate {row:&row_template::LAYOUT,text_id:"unknown"})).is_err());
}

#[test]
fn wrap_content_list_measures_xml_row_heights_and_dividers() {
    static LIST:android_xml::Node=android_xml::Node {height:android_xml::Size::Wrap,..fixture::LAYOUT};
    static AFTER:android_xml::Node=android_xml::Node {id:"after",text:"After",height:android_xml::Size::Dp(20.0),..row_template::LAYOUT.children[0]};
    static ROOT:android_xml::Node=android_xml::Node {id:"root",kind:android_xml::Kind::Linear,height:android_xml::Size::Match,horizontal:false,children:&[LIST,AFTER],padding_edges:[0.;4],content_gravity:[0,0],..row_template::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.list_items.insert("workspaces_list".into(),vec!["One".into(),"Two".into()]);
    state.set_list_item_template("workspaces_list",Some(android_xml::ListItemTemplate {row:&row_template::LAYOUT,text_id:"spinner_title"})).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{ROOT.render(f,BOUNDS,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.as_ref()=="After" && (bounds.y-129.0).abs()<0.01)),"following view should start after both 64dp rows and one divider");
}

#[test]
fn stable_list_adapter_ids_keep_choices_and_validate_updates() {
    let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    let names=|items:&[&str]|items.iter().map(|s|s.to_string()).collect::<Vec<_>>();
    state.set_list_adapter(id,names(&["A","B","C"]),Some(vec![10,20,30]),Some(vec![true,false,true])).unwrap();
    state.list_checked.insert(id.into(),vec![0,2]);state.list_clicked.insert(id.into(),2);
    state.set_list_adapter(id,names(&["C","A","B"]),Some(vec![30,10,20]),Some(vec![true,true,false])).unwrap();
    assert_eq!(state.list_checked[id],vec![1,0]);assert_eq!(state.list_clicked[id],0);
    assert!(!state.list_enabled[id][2]);
    assert!(state.set_list_adapter(id,names(&["X","Y"]),Some(vec![1,1]),None).is_err());
    assert!(state.set_list_adapter(id,names(&["X"]),Some(vec![1,2]),None).is_err());
    assert_eq!(state.list_items[id],names(&["C","A","B"]));assert_eq!(state.list_checked[id],vec![1,0]);
    state.set_list_adapter(id,names(&["A","B"]),Some(vec![10,20]),None).unwrap();
    assert_eq!(state.list_checked[id],vec![0]);assert!(!state.list_clicked.contains_key(id));
    state.set_list_adapter(id,names(&["B"]),None,None).unwrap();assert_eq!(state.list_checked[id],vec![0]);
    state.set_list_adapter(id,vec![],None,None).unwrap();assert!(!state.list_checked.contains_key(id));
}
#[test]
fn adapter_reorder_during_press_does_not_activate_another_item() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.set_list_adapter(id,vec!["A".into(),"B".into()],Some(vec![10,20]),None).unwrap();
    draw(&mut ui,&mut state,Input::default());draw(&mut ui,&mut state,Input {pointer:Some([40.,24.]),pressed:true,time:0.1,..Input::default()});
    state.set_list_adapter(id,vec!["B".into(),"A".into()],Some(vec![20,10]),None).unwrap();
    assert_eq!(draw(&mut ui,&mut state,Input {pointer:Some([40.,24.]),released:true,time:0.2,..Input::default()}),None);
    assert!(!state.list_clicked.contains_key(id));
}

#[test]
fn directional_navigation_skips_disabled_rows_scrolls_and_activates() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.list_items.insert("workspaces_list".into(),(0..20).map(|i|format!("Item {i}")).collect());
    state.list_enabled.insert("workspaces_list".into(),(0..20).map(|i|i!=1 && i!=19).collect());
    draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,24.,0.1);
    let key=|key,time|Input {time,key:Some(KeyInput {key,pressed:true,repeat:false,shift:false}),..Input::default()};
    draw(&mut ui,&mut state,key(Key::Down,0.2));draw(&mut ui,&mut state,Input {time:0.21,..Input::default()});
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Item 2" && n.focused));
    assert_eq!(draw(&mut ui,&mut state,key(Key::Enter,0.3)),Some("workspaces_list"));assert_eq!(state.list_checked["workspaces_list"],vec![0,2]);
    draw(&mut ui,&mut state,key(Key::End,0.4));draw(&mut ui,&mut state,Input {time:0.41,..Input::default()});
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Item 18" && n.focused));assert!(state.offsets["workspaces_list"]>0.0);
    draw(&mut ui,&mut state,key(Key::PageUp,0.5));draw(&mut ui,&mut state,Input {time:0.51,..Input::default()});
    let page=ui.semantics().iter().find(|n|n.focused).unwrap();assert_ne!(page.label.as_ref(),"Item 18");
    draw(&mut ui,&mut state,key(Key::Home,0.6));draw(&mut ui,&mut state,Input {time:0.61,..Input::default()});
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Item 0" && n.focused));assert_eq!(state.offsets["workspaces_list"],0.0);
}

#[test]
fn navigation_scrolls_to_variable_height_xml_row() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.list_items.insert("workspaces_list".into(),vec!["Start".into(),"A very long workspace name requiring many lines in this row to verify the list uses the actual measured height for movement".into(),"Final".into()]);
    state.set_list_item_template("workspaces_list",Some(android_xml::ListItemTemplate {row:&row_template::LAYOUT,text_id:"spinner_title"})).unwrap();
    draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,20.,0.1);
    let key=|key,time|Input {time,key:Some(KeyInput {key,pressed:true,repeat:false,shift:false}),..Input::default()};
    draw(&mut ui,&mut state,key(Key::End,0.2));draw(&mut ui,&mut state,Input {time:0.21,..Input::default()});
    let last=ui.semantics().iter().find(|n|n.label.as_ref()=="Final" && n.focused).unwrap().bounds;
    assert!(last.y>=0.0 && last.y+last.height<=BOUNDS.height);assert!(state.offsets["workspaces_list"]>0.0);
    assert_eq!(draw(&mut ui,&mut state,key(Key::Enter,0.3)),Some("workspaces_list"));assert_eq!(state.list_checked["workspaces_list"],vec![0,2]);
}

#[test]
fn long_press_fires_once_and_does_not_trigger_short_click() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.list_long_click_enabled.insert("workspaces_list".into(),true);
    draw(&mut ui,&mut state,Input::default());
    let touch=|pressed,released,time|Input {pointer:Some([40.,24.]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(true,false,0.1));
    assert_eq!(draw(&mut ui,&mut state,touch(false,false,0.59)),None);assert!(!state.list_long_clicked.contains_key("workspaces_list"));
    assert_eq!(draw(&mut ui,&mut state,touch(false,false,0.61)),Some("workspaces_list"));assert_eq!(state.list_long_clicked.remove("workspaces_list"),Some(0));
    assert_eq!(draw(&mut ui,&mut state,touch(false,false,1.0)),None);
    assert_eq!(draw(&mut ui,&mut state,touch(false,true,1.1)),None);assert!(!state.list_clicked.contains_key("workspaces_list"));assert!(state.list_checked["workspaces_list"].is_empty());
}
#[test]
fn long_press_is_cancelled_by_scroll_movement_disable_or_adapter_update() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.list_long_click_enabled.insert(id.into(),true);
    draw(&mut ui,&mut state,Input::default());
    let touch=|x,y,pressed,released,time|Input {pointer:Some([x,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(40.,100.,true,false,0.1));draw(&mut ui,&mut state,touch(40.,60.,false,false,0.2));
    assert_eq!(draw(&mut ui,&mut state,touch(40.,60.,false,false,0.7)),None);draw(&mut ui,&mut state,touch(40.,60.,false,true,0.8));assert!(!state.list_long_clicked.contains_key(id));
    state.offsets.insert(id.into(),0.0);state.list_enabled.insert(id.into(),vec![false]);draw(&mut ui,&mut state,Input {time:1.0,..Input::default()});
    draw(&mut ui,&mut state,touch(40.,24.,true,false,1.1));assert_eq!(draw(&mut ui,&mut state,touch(40.,24.,false,false,1.7)),None);draw(&mut ui,&mut state,touch(40.,24.,false,true,1.8));
    assert!(!state.list_long_clicked.contains_key(id));
    state.list_enabled.remove(id);draw(&mut ui,&mut state,touch(40.,24.,true,false,2.0));
    state.set_list_adapter(id,vec!["Replacement".into()],None,None).unwrap();
    assert_eq!(draw(&mut ui,&mut state,touch(40.,24.,false,false,2.6)),None);assert!(!state.list_long_clicked.contains_key(id));
}

mod header {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_list_header.rs"));}
mod footer {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_list_footer.rs"));}
fn fixed_views()->(Vec<android_xml::ListFixedView>,Vec<android_xml::ListFixedView>) {
    (vec![android_xml::ListFixedView {view:&header::LAYOUT,selectable:false,label:"Workspaces"}],vec![android_xml::ListFixedView {view:&footer::LAYOUT,selectable:true,label:"End of workspaces"}])
}
#[test]
fn fixed_views_scroll_and_select_without_changing_item_indices() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.list_items.insert(id.into(),vec!["One".into()]);let (headers,footers)=fixed_views();state.set_list_fixed_views(id,headers,footers).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,..} if text.as_ref()=="Workspaces")));
    let item=ui.semantics().iter().find(|n|n.label.as_ref()=="One").unwrap().bounds;assert_eq!(item.y,57.0);
    assert_eq!(click(&mut ui,&mut state,24.,0.1),None);
    assert_eq!(click(&mut ui,&mut state,80.,0.2),Some(id));assert_eq!(state.list_checked[id],vec![0]);
    let foot=ui.semantics().iter().find(|n|n.label.as_ref()=="End of workspaces").unwrap().bounds;assert_eq!(foot.y,106.0);
    assert_eq!(click(&mut ui,&mut state,foot.y+20.,0.3),Some(id));assert_eq!(state.list_fixed_clicked[id],(true,0));assert_eq!(state.list_checked[id],vec![0]);
}
#[test]
fn fixed_view_dividers_obey_xml_flags_and_hold_survives_same_layout_update() {
    static NO_DIVIDERS:android_xml::Node=android_xml::Node {relative_rules:&[android_xml::RelativeRule {kind:88,anchor:"multipleChoice"},android_xml::RelativeRule {kind:89,anchor:"false"},android_xml::RelativeRule {kind:90,anchor:"false"}],..fixture::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.list_items.insert(id.into(),vec!["One".into()]);state.list_long_click_enabled.insert(id.into(),true);
    let (headers,footers)=fixed_views();state.set_list_fixed_views(id,headers,footers).unwrap();
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{let mut action=None;ui.frame(input,VIEW,0.,|f|{action=NO_DIVIDERS.render(f,BOUNDS,state);});action};
    draw(&mut ui,&mut state,Input::default());assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="One").unwrap().bounds.y,56.0);assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="End of workspaces").unwrap().bounds.y,104.0);
    let touch=|pressed,time|Input {pointer:Some([30.,75.]),pointer_kind:PointerKind::Touch,pressed,time,..Input::default()};
    draw(&mut ui,&mut state,touch(true,0.1));let (headers,footers)=fixed_views();state.set_list_fixed_views(id,headers,footers).unwrap();
    assert_eq!(draw(&mut ui,&mut state,touch(false,0.61)),Some(id));assert_eq!(state.list_long_clicked[id],0);
}
#[test]
fn multiple_fixed_views_and_empty_adapter_are_measured_and_clickable() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.list_items.insert(id.into(),vec![]);let (mut headers,mut footers)=fixed_views();headers.push(headers[0]);footers.push(footers[0]);state.set_list_fixed_views(id,headers,footers).unwrap();
    draw(&mut ui,&mut state,Input::default());let labels:Vec<_>=ui.semantics().iter().filter(|n|n.label.as_ref()=="End of workspaces").collect();assert_eq!(labels.len(),2);assert_eq!(labels[0].bounds.y,114.0);assert_eq!(labels[1].bounds.y,171.0);
    assert_eq!(click(&mut ui,&mut state,190.,0.1),Some(id));assert_eq!(state.list_fixed_clicked[id],(true,1));
}

#[test]
fn footer_reached_by_touch_scroll_and_wrap_content_includes_fixed_views() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.list_items.insert(id.into(),(0..10).map(|i|format!("Item {i}")).collect());let (headers,footers)=fixed_views();state.set_list_fixed_views(id,headers,footers).unwrap();
    draw(&mut ui,&mut state,Input::default());
    let touch=|y,pressed,released,time|Input {pointer:Some([80.,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    for j in 0..4 {let t=0.1+j as f64*0.5;draw(&mut ui,&mut state,touch(180.,true,false,t));draw(&mut ui,&mut state,touch(30.,false,false,t+0.1));draw(&mut ui,&mut state,touch(30.,false,true,t+0.2));}
    let end=ui.semantics().iter().find(|n|n.label.as_ref()=="End of workspaces").unwrap().bounds;
    assert_eq!(click(&mut ui,&mut state,(end.y+end.height*0.5).min(195.),3.0),Some(id));assert_eq!(state.list_fixed_clicked[id],(true,0));
    static LIST:android_xml::Node=android_xml::Node {height:android_xml::Size::Wrap,..fixture::LAYOUT};
    static AFTER:android_xml::Node=android_xml::Node {id:"after",text:"After",height:android_xml::Size::Dp(20.0),..row_template::LAYOUT.children[0]};
    static ROOT:android_xml::Node=android_xml::Node {id:"root",kind:android_xml::Kind::Linear,height:android_xml::Size::Match,horizontal:false,children:&[LIST,AFTER],padding_edges:[0.;4],content_gravity:[0,0],..row_template::LAYOUT};
    state.list_items.insert(id.into(),vec!["One".into()]);state.offsets.insert(id.into(),0.0);
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{ROOT.render(f,BOUNDS,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.as_ref()=="After" && (bounds.y-162.0).abs()<0.01)));
}

#[test]
fn default_rows_wrap_text_and_update_visible_geometry_when_changed() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    let long="A long title which must wrap across multiple lines so the Android simple list item expands to show its complete text";
    state.list_items.insert(id.into(),vec!["Short".into(),long.into(),"Last".into()]);
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    let first=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds;let second=ui.semantics().iter().find(|n|n.label.as_ref()==long).unwrap().bounds;
    assert_eq!(first.height,48.0);assert!(second.height>48.0);assert!((second.y-first.y-49.0).abs()<0.01);
    assert!(commands.iter().filter(|c|matches!(c,DrawCommand::Text {text,..} if long.contains(text.as_ref()) && !text.is_empty())).count()>=2);
    state.list_items.get_mut(id).unwrap()[1]="Changed".into();draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="Changed").unwrap().bounds.height,48.0);
}
#[test]
fn navigation_measures_offscreen_wrapping_target_before_focusing() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    let long="The final workspace has a long title that wraps several times and must be brought fully into the visible list area after navigation";
    let mut items=(0..8).map(|i|format!("Item {i}")).collect::<Vec<_>>();items.push(long.into());state.list_items.insert(id.into(),items);
    draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,24.,0.1);
    draw(&mut ui,&mut state,Input {time:0.2,key:Some(KeyInput {key:Key::End,pressed:true,repeat:false,shift:false}),..Input::default()});
    draw(&mut ui,&mut state,Input {time:0.21,..Input::default()});
    let target=ui.semantics().iter().find(|n|n.label.as_ref()==long && n.focused).unwrap().bounds;
    assert!(target.height>48.0 && target.y>=0.0 && target.y+target.height<=BOUNDS.height,"{target:?}");
}

#[test]
fn large_default_list_measures_only_visible_wrapping_rows() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    let long="A long workspace name that wraps across multiple lines and expands this row";
    state.list_items.insert(id.into(),(0..100_000).map(|i|if i==99_999 {long.into()}else{format!("Item {i}")}).collect());
    state.offsets.insert(id.into(),(99_995.0_f32)*49.0);
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    assert!(ui.semantics().len()<10);assert!(commands.len()<120);
    // Scroll to the final item; the newly discovered taller row updates the extent.
    draw(&mut ui,&mut state,Input {time:0.2,pointer:Some([100.,100.]),scroll:[0.,400.],..Input::default()});
    let last=ui.semantics().iter().find(|n|n.label.as_ref()==long).unwrap().bounds;
    assert!(last.height>48.0);assert!(ui.semantics().len()<10);
}

#[test]
fn default_rows_remeasure_when_list_width_changes() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    let name="A workspace name that fits a wide list but wraps in a narrow list";state.list_items.insert(id.into(),vec![name.into()]);
    ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});let wide=ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds.height;
    ui.frame(Input {time:0.1,..Input::default()},VIEW,0.,|f|{fixture::LAYOUT.render(f,Rect {width:150.,..BOUNDS},&mut state);});let narrow=ui.semantics().iter().find(|n|n.label.as_ref()==name).unwrap().bounds.height;
    assert!(narrow>wide,"wide {wide}, narrow {narrow}");
}

#[test]
fn large_xml_adapter_measures_visible_rows_and_reacts_to_edits() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.list_items.insert(id.into(),(0..100_000).map(|i|format!("Workspace {i}")).collect());
    state.set_list_item_template(id,Some(android_xml::ListItemTemplate {row:&row_template::LAYOUT,text_id:"spinner_title"})).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    assert!(ui.semantics().len()<10);assert!(commands.len()<200);
    let initial=ui.semantics().iter().find(|n|n.label.as_ref()=="Workspace 0").unwrap().bounds.height;
    let long="A newly edited workspace with a much longer title that has to wrap over several lines";
    state.list_items.get_mut(id).unwrap()[0]=long.into();draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});
    let changed=ui.semantics().iter().find(|n|n.label.as_ref()==long).unwrap().bounds.height;assert!(changed>initial);
    state.offsets.insert(id.into(),99_996.0*49.0);
    draw(&mut ui,&mut state,Input {time:0.2,..Input::default()});
    assert!(ui.semantics().iter().any(|n|n.label.as_ref().starts_with("Workspace 9999")));assert!(ui.semantics().len()<10);
}

mod stateful_item {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_gridview_item.rs"));}
#[test]
fn list_xml_item_receives_activated_background_without_leaking_to_other_rows() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.list_items.insert(id.into(),vec!["A".into(),"B".into()]);stateful_item::install_drawables(&mut state);
    state.set_list_item_template(id,Some(android_xml::ListItemTemplate {row:&stateful_item::LAYOUT,text_id:"grid_item_title"})).unwrap();
    draw(&mut ui,&mut state,Input::default());assert_eq!(click(&mut ui,&mut state,24.,0.1),Some(id));
    let commands=ui.frame(Input {time:0.2,..Input::default()},VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    let activated=commands.iter().filter(|c|matches!(c,DrawCommand::RoundedRect {color:Color(168,213,178,255),..})).count();
    let normal=commands.iter().filter(|c|matches!(c,DrawCommand::RoundedRect {color:Color(228,241,231,255),..})).count();
    assert_eq!(activated,1);assert_eq!(normal,1);
}

#[test]
fn list_item_bindings_follow_stable_ids_and_do_not_leak() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.set_list_adapter(id,vec!["One".into(),"Two".into()],Some(vec![10,20]),None).unwrap();stateful_item::install_drawables(&mut state);
    state.set_list_item_template(id,Some(android_xml::ListItemTemplate {row:&stateful_item::LAYOUT,text_id:"grid_item_title"})).unwrap();
    let mut binding=android_xml::AdapterItemBinding::default();binding.text.insert("grid_item_subtitle".into(),"Private".into());state.set_list_item_binding(id,0,Some(binding)).unwrap();
    state.set_list_adapter(id,vec!["Two".into(),"One".into()],Some(vec![20,10]),None).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    let target=ui.semantics().iter().find(|n|n.label.as_ref()=="One").unwrap().bounds;
    assert_eq!(commands.iter().filter(|c|matches!(c,DrawCommand::Text {text,..} if text.as_ref()=="Private")).count(),1);
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.as_ref()=="Private" && bounds.y>=target.y && bounds.y<target.y+target.height)));
}

mod team_item {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_gridview_team_item.rs"));}
#[test]
fn list_supports_mixed_default_and_per_item_xml_layouts() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_list";
    state.set_list_adapter(id,vec!["One".into(),"Two".into()],Some(vec![10,20]),None).unwrap();team_item::install_drawables(&mut state);
    let mut binding=android_xml::AdapterItemBinding::default();binding.template=Some(android_xml::ListItemTemplate {row:&team_item::LAYOUT,text_id:"team_title"});binding.text.insert("team_subtitle".into(),"Team".into());state.set_list_item_binding(id,1,Some(binding)).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    let one=ui.semantics().iter().find(|n|n.label.as_ref()=="One").unwrap().bounds;let two=ui.semantics().iter().find(|n|n.label.as_ref()=="Two").unwrap().bounds;
    assert_eq!(one.height,48.0);assert_eq!(two.height,64.0);assert_eq!(two.y,49.0);
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,..} if text.as_ref()=="Team")));
}
mod interactive_row {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_list_interactive_row.rs"));}
#[test]
fn interactive_row_checkbox_retains_state_by_stable_id() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.set_list_adapter("workspaces_list",vec!["First".into(),"Second".into()],Some(vec![10,20]),None).unwrap();
    state.set_interactive_list_item_template("workspaces_list",Some(android_xml::ListItemTemplate{row:&interactive_row::LAYOUT,text_id:"row_title"})).unwrap();
    draw(&mut ui,&mut state,Input::default());
    let p=Some([264.,28.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});
    assert_eq!(draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()}),Some("workspaces_list"));
    assert_eq!(state.list_control_clicked["workspaces_list"],(0,"row_check"));
    assert_eq!(state.list_item_control_checked("workspaces_list",10,"row_check"),Some(true));
    assert!(state.list_clicked.get("workspaces_list").is_none());
    state.set_list_adapter("workspaces_list",vec!["Second".into(),"First".into()],Some(vec![20,10]),None).unwrap();
    draw(&mut ui,&mut state,Input{time:1.2,..Input::default()});
    assert_eq!(state.list_item_control_checked("workspaces_list",10,"row_check"),Some(true));
    assert_eq!(state.list_item_control_checked("workspaces_list",20,"row_check"),Some(false));
    state.set_list_adapter("workspaces_list",vec!["Second".into(),"First".into()],None,None).unwrap();
    assert_eq!(state.list_item_control_checked("workspaces_list",1,"row_check"),Some(true));
    state.set_list_adapter("workspaces_list",vec!["Second".into(),"First".into()],Some(vec![20,10]),None).unwrap();
    assert_eq!(state.list_item_control_checked("workspaces_list",10,"row_check"),Some(true));
}
mod interactive_header {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_list_interactive_header.rs"));}
#[test]
fn fixed_header_switch_keeps_state_without_activating_header() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.set_list_fixed_views("workspaces_list",vec![android_xml::ListFixedView{view:&interactive_header::LAYOUT,selectable:true,label:"Notifications"}],vec![]).unwrap();
    draw(&mut ui,&mut state,Input::default());let r=ui.semantics().iter().find(|node|node.role==Role::Switch).unwrap().bounds;let p=Some([r.x+r.width/2.,r.y+r.height/2.]);
    draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});
    assert_eq!(draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()}),Some("workspaces_list"));
    assert_eq!(state.list_fixed_control_clicked["workspaces_list"],(false,0,"header_switch"));
    assert_eq!(state.list_fixed_control_checked("workspaces_list",false,0,"header_switch"),Some(true));
    assert!(state.list_fixed_clicked.get("workspaces_list").is_none());assert!(state.list_clicked.get("workspaces_list").is_none());
    draw(&mut ui,&mut state,Input{time:1.1,..Input::default()});
    assert_eq!(state.list_fixed_control_checked("workspaces_list",false,0,"header_switch"),Some(true));
}
#[test]
fn per_item_interactive_template_follows_binding_and_state_on_reorder() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.set_list_adapter("workspaces_list",vec!["First".into(),"Second".into()],Some(vec![10,20]),None).unwrap();
    let mut binding=android_xml::AdapterItemBinding::default();binding.template=Some(android_xml::ListItemTemplate{row:&interactive_row::LAYOUT,text_id:"row_title"});
    assert!(state.set_list_item_binding("workspaces_list",1,Some(binding.clone())).is_err());
    state.set_interactive_list_item_binding("workspaces_list",1,Some(binding)).unwrap();
    draw(&mut ui,&mut state,Input::default());let switches=ui.semantics().iter().filter(|node|node.role==Role::Checkbox).collect::<Vec<_>>();assert_eq!(switches.len(),1);let r=switches[0].bounds;let p=Some([r.x+r.width/2.,r.y+r.height/2.]);
    draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()});
    assert_eq!(state.list_control_clicked["workspaces_list"],(1,"row_check"));assert_eq!(state.list_item_control_checked("workspaces_list",20,"row_check"),Some(true));
    state.set_list_adapter("workspaces_list",vec!["Second".into(),"First".into()],Some(vec![20,10]),None).unwrap();
    draw(&mut ui,&mut state,Input{time:1.2,..Input::default()});assert_eq!(state.list_item_control_checked("workspaces_list",20,"row_check"),Some(true));assert_eq!(ui.semantics().iter().filter(|node|node.role==Role::Checkbox).count(),1);
}
