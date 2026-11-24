use bhoo::*;
mod fixture { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_spinner.rs")); }
mod dialog_fixture { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_spinner_dialog.rs")); }
mod wide_fixture { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_spinner_wide.rs")); }
mod selected_template { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_spinner_selected_row.rs")); }
mod dropdown_template { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_spinner_dropdown_row.rs")); }
const VIEW:Rect=Rect{x:0.,y:0.,width:300.,height:400.};
const BOUNDS:Rect=Rect{x:0.,y:0.,width:300.,height:48.};
fn draw(ui:&mut Ui,state:&mut android_xml::LayoutState,input:Input)->Option<&'static str> {
    let mut action=None;ui.frame(input,VIEW,0.,|f|{action=fixture::LAYOUT.render(f,BOUNDS,state);});action
}
fn click(ui:&mut Ui,state:&mut android_xml::LayoutState,x:f32,y:f32,time:f64)->Option<&'static str> {
    draw(ui,state,Input {pointer:Some([x,y]),pressed:true,time,..Input::default()});
    draw(ui,state,Input {pointer:Some([x,y]),released:true,time:time+0.01,..Input::default()})
}
#[test]
fn xml_spinner_dropdown_selects_entries_and_dismisses() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    draw(&mut ui,&mut state,Input::default());assert_eq!(state.spinner_items["workspace"].len(),13);
    click(&mut ui,&mut state,30.,24.,0.1);assert_eq!(state.spinner_open.as_deref(),Some("workspace"));
    assert_eq!(click(&mut ui,&mut state,30.,120.,0.3),Some("workspace"));
    assert_eq!(state.spinner_selected.get("workspace"),Some(&1));assert!(state.spinner_open.is_none());
    click(&mut ui,&mut state,30.,24.,0.5);assert!(state.spinner_open.is_some());
    draw(&mut ui,&mut state,Input {pointer:Some([295.,10.]),pressed:true,time:0.7,..Input::default()});
    assert!(state.spinner_open.is_none());assert_eq!(state.spinner_selected.get("workspace"),Some(&1));
}
#[test]
fn spinner_keyboard_highlight_requires_commit_and_escape_cancels() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,30.,24.,0.1);
    let key=|key,time|Input {time,key:Some(KeyInput {key,pressed:true,repeat:false,shift:false}),..Input::default()};
    draw(&mut ui,&mut state,key(Key::Down,0.2));assert_eq!(state.spinner_selected.get("workspace"),None);
    draw(&mut ui,&mut state,key(Key::Escape,0.3));assert!(state.spinner_open.is_none());assert_eq!(state.spinner_selected.get("workspace"),None);
    click(&mut ui,&mut state,30.,24.,0.4);draw(&mut ui,&mut state,key(Key::Down,0.5));
    assert_eq!(draw(&mut ui,&mut state,key(Key::Enter,0.6)),Some("workspace"));
    assert_eq!(state.spinner_selected.get("workspace"),Some(&1));
}
#[test]
fn touch_scroll_shows_later_items_without_selecting_during_drag() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,30.,24.,0.1);
    let touch=|y,pressed,released,time|Input {pointer:Some([100.,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(300.,true,false,0.3));
    draw(&mut ui,&mut state,touch(100.,false,false,0.4));
    assert_eq!(draw(&mut ui,&mut state,touch(100.,false,true,0.6)),None);
    assert!(state.spinner_open.is_some());assert_eq!(state.spinner_selected.get("workspace"),None);
    assert!(ui.semantics().iter().any(|node|node.label.as_ref()=="Operations"));
    let pos=ui.semantics().iter().find(|node|node.label.as_ref()=="Operations").unwrap().bounds;
    assert_eq!(click(&mut ui,&mut state,pos.x+10.,pos.y+20.,0.8),Some("workspace"));
    assert_eq!(state.spinner_selected.get("workspace"),Some(&7));
}
#[test]
fn spinner_fling_moves_after_touch_release() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,30.,24.,0.1);
    let touch=|y,pressed,released,time|Input {pointer:Some([100.,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(280.,true,false,0.3));draw(&mut ui,&mut state,touch(180.,false,false,0.32));draw(&mut ui,&mut state,touch(180.,false,true,0.33));
    let first=ui.semantics().iter().find(|node|node.label.as_ref()=="Research").map(|n|n.bounds.y);
    draw(&mut ui,&mut state,Input {time:0.37,..Input::default()});
    let second=ui.semantics().iter().find(|node|node.label.as_ref()=="Research").map(|n|n.bounds.y);
    assert!(first.zip(second).is_some_and(|(a,b)|b<a));assert!(state.spinner_open.is_some());
}
#[test]
fn dialog_spinner_uses_modal_and_commits_a_choice() {
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=android_xml::LayoutState::default();
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input| {let mut action=None;let commands=ui.frame(input,VIEW,0.,|f| {action=dialog_fixture::LAYOUT.render(f,BOUNDS,state);});(commands,action)};
    draw(&mut ui,&mut state,Input::default());
    draw(&mut ui,&mut state,Input {pointer:Some([30.,24.]),pressed:true,time:0.1,..Input::default()});
    let (commands,_)=draw(&mut ui,&mut state,Input {pointer:Some([30.,24.]),released:true,time:0.11,..Input::default()});
    assert_eq!(state.spinner_open.as_deref(),Some("dialog_workspace"));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,..} if text.as_ref()=="Choose workspace")));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect {bounds,color,..} if *bounds==VIEW && color.3>0 && color.3<255)));
    let item=ui.semantics().iter().find(|n|n.label.as_ref()=="Projects").unwrap().bounds;
    let point=[item.x+item.width*0.5,item.y+item.height*0.5];
    draw(&mut ui,&mut state,Input {pointer:Some(point),pressed:true,time:0.2,..Input::default()});
    let (_,action)=draw(&mut ui,&mut state,Input {pointer:Some(point),released:true,time:0.21,..Input::default()});
    assert_eq!(action,Some("dialog_workspace"));assert_eq!(state.spinner_selected.get("dialog_workspace"),Some(&1));assert!(state.spinner_open.is_none());
}
#[test]
fn dialog_spinner_escape_keeps_selection_and_finishes_exit() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input| {ui.frame(input,VIEW,0.,|f| {dialog_fixture::LAYOUT.render(f,BOUNDS,state);})};
    draw(&mut ui,&mut state,Input::default());
    draw(&mut ui,&mut state,Input {pointer:Some([30.,24.]),pressed:true,time:0.1,..Input::default()});
    draw(&mut ui,&mut state,Input {pointer:Some([30.,24.]),released:true,time:0.11,..Input::default()});
    draw(&mut ui,&mut state,Input {time:0.6,..Input::default()});
    let down=KeyInput {key:Key::Down,pressed:true,repeat:false,shift:false};draw(&mut ui,&mut state,Input {time:0.7,key:Some(down),..Input::default()});
    let escape=KeyInput {key:Key::Escape,pressed:true,repeat:false,shift:false};draw(&mut ui,&mut state,Input {time:0.8,key:Some(escape),..Input::default()});
    assert!(state.spinner_open.is_none());assert_eq!(state.spinner_selected.get("dialog_workspace"),None);
    draw(&mut ui,&mut state,Input {time:1.3,..Input::default()});
    draw(&mut ui,&mut state,Input {time:1.9,..Input::default()});assert!(!ui.needs_redraw());
}
#[test]
fn spinner_dropdown_width_offsets_and_screen_edges_follow_direction() {
    for (rtl,expected_x) in [(false,80.0),(true,48.0)] {
        let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.rtl=rtl;
        let anchor=Rect{x:180.,y:330.,width:100.,height:48.};
        let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input| {ui.frame(input,VIEW,0.,|f| {wide_fixture::LAYOUT.render(f,anchor,state);})};
        draw(&mut ui,&mut state,Input::default());
        draw(&mut ui,&mut state,Input {pointer:Some([230.,354.]),pressed:true,time:0.1,..Input::default()});
        draw(&mut ui,&mut state,Input {pointer:Some([230.,354.]),released:true,time:0.11,..Input::default()});
        let item=ui.semantics().iter().find(|n|n.label.as_ref()=="Personal" && n.bounds.width>100.0).unwrap();
        assert_eq!(item.bounds,Rect{x:expected_x,y:0.,width:220.,height:48.},"rtl={rtl}");
    }
}
#[test]
fn spinner_closes_when_anchor_leaves_viewport() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,30.,24.,0.1);
    assert!(state.spinner_open.is_some());
    ui.frame(Input {time:0.3,..Input::default()},VIEW,0.,|f| {fixture::LAYOUT.render(f,Rect{y:500.,..BOUNDS},&mut state);});
    assert!(state.spinner_open.is_none());
}
#[test]
fn disabled_spinner_items_do_not_activate_and_keyboard_skips_them() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.spinner_enabled.insert("workspace".into(),vec![true,false,true]);
    draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,30.,24.,0.1);
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Projects" && !n.enabled));
    assert_eq!(click(&mut ui,&mut state,30.,120.,0.2),None);assert!(state.spinner_open.is_some());
    let key=|key,time|Input {time,key:Some(KeyInput {key,pressed:true,repeat:false,shift:false}),..Input::default()};
    draw(&mut ui,&mut state,key(Key::Down,0.3));assert_eq!(draw(&mut ui,&mut state,key(Key::Enter,0.4)),Some("workspace"));
    assert_eq!(state.spinner_selected.get("workspace"),Some(&2));
}
#[test]
fn dialog_spinner_disabled_item_is_not_selectable() {
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=android_xml::LayoutState::default();
    state.spinner_enabled.insert("dialog_workspace".into(),vec![true,false,true]);
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input| {let mut action=None;ui.frame(input,VIEW,0.,|f| {action=dialog_fixture::LAYOUT.render(f,BOUNDS,state);});action};
    draw(&mut ui,&mut state,Input::default());draw(&mut ui,&mut state,Input {pointer:Some([30.,24.]),pressed:true,time:0.1,..Input::default()});draw(&mut ui,&mut state,Input {pointer:Some([30.,24.]),released:true,time:0.11,..Input::default()});
    let item=ui.semantics().iter().find(|n|n.label.as_ref()=="Projects").unwrap();assert!(!item.enabled);let b=item.bounds;let p=[b.x+b.width*0.5,b.y+b.height*0.5];
    draw(&mut ui,&mut state,Input {pointer:Some(p),pressed:true,time:0.2,..Input::default()});assert_eq!(draw(&mut ui,&mut state,Input {pointer:Some(p),released:true,time:0.21,..Input::default()}),None);
    assert!(state.spinner_open.is_some());assert!(state.spinner_selected.get("dialog_workspace").is_none());
}
#[test]
fn replacing_spinner_entries_reconciles_selection_and_empty_list() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();draw(&mut ui,&mut state,Input::default());
    state.spinner_selected.insert("workspace".into(),9);state.spinner_items.insert("workspace".into(),vec!["One".into(),"Two".into()]);
    draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});assert_eq!(state.spinner_selected.get("workspace"),Some(&1));
    click(&mut ui,&mut state,30.,24.,0.2);assert!(state.spinner_open.is_some());state.spinner_items.insert("workspace".into(),vec![]);
    draw(&mut ui,&mut state,Input {time:0.3,..Input::default()});assert!(state.spinner_open.is_none());assert!(state.spinner_selected.get("workspace").is_none());
}
#[test]
fn stable_adapter_ids_preserve_selection_and_validate_updates() {
    let mut state=android_xml::LayoutState::default();
    let names=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
    state.set_spinner_adapter("workspace",names(&["A","B","C"]),Some(vec![10,20,30]),Some(vec![true,false,true])).unwrap();
    state.spinner_selected.insert("workspace".into(),1);
    state.set_spinner_adapter("workspace",names(&["C","A","B"]),Some(vec![30,10,20]),Some(vec![true,true,false])).unwrap();
    assert_eq!(state.spinner_selected["workspace"],2);assert_eq!(state.spinner_items["workspace"][2],"B");assert!(!state.spinner_enabled["workspace"][2]);
    assert!(state.set_spinner_adapter("workspace",names(&["X","Y"]),Some(vec![10,10]),None).is_err());
    assert!(state.set_spinner_adapter("workspace",names(&["X"]),Some(vec![1,2]),None).is_err());
    assert_eq!(state.spinner_items["workspace"],names(&["C","A","B"]));assert_eq!(state.spinner_selected["workspace"],2);
    state.set_spinner_adapter("workspace",names(&["C","A"]),Some(vec![30,10]),None).unwrap();assert_eq!(state.spinner_selected["workspace"],1);
}
#[test]
fn open_spinner_keeps_highlighted_stable_item_after_reorder() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let names=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
    state.set_spinner_adapter("workspace",names(&["A","B","C"]),Some(vec![10,20,30]),None).unwrap();
    draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,30.,24.,0.1);
    let key=|key,time|Input {time,key:Some(KeyInput {key,pressed:true,repeat:false,shift:false}),..Input::default()};
    draw(&mut ui,&mut state,key(Key::Down,0.2));
    state.set_spinner_adapter("workspace",names(&["C","A","B"]),Some(vec![30,10,20]),None).unwrap();
    assert_eq!(state.spinner_selected["workspace"],1);
    assert_eq!(draw(&mut ui,&mut state,key(Key::Enter,0.3)),Some("workspace"));
    assert_eq!(state.spinner_selected["workspace"],2);
}
#[test]
fn xml_spinner_templates_render_separate_selected_and_dropdown_views() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.set_spinner_item_templates("workspace",Some(android_xml::SpinnerItemTemplates {selected:&selected_template::LAYOUT,dropdown:&dropdown_template::LAYOUT,text_id:"spinner_title"})).unwrap();
    let mut commands=ui.frame(Input::default(),VIEW,0.,|f| {fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,style,..} if text.as_ref()=="Personal" && style.weight==700)));
    click(&mut ui,&mut state,30.,24.,0.1);
    commands=ui.frame(Input {time:0.2,..Input::default()},VIEW,0.,|f| {fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,style,..} if text.as_ref()=="Projects" && style.weight==700)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,style,..} if text.as_ref()=="Workspace" && style.size==10.0)));
    assert_eq!(click(&mut ui,&mut state,30.,120.,0.3),Some("workspace"));assert_eq!(state.spinner_selected["workspace"],1);
}
#[test]
fn spinner_templates_reject_interactive_views_and_missing_bindings() {
    static INTERACTIVE:android_xml::Node=android_xml::Node {clickable:true,..selected_template::LAYOUT};
    let mut state=android_xml::LayoutState::default();
    assert!(state.set_spinner_item_templates("workspace",Some(android_xml::SpinnerItemTemplates {selected:&INTERACTIVE,dropdown:&dropdown_template::LAYOUT,text_id:"spinner_title"})).is_err());
    assert!(state.set_spinner_item_templates("workspace",Some(android_xml::SpinnerItemTemplates {selected:&selected_template::LAYOUT,dropdown:&dropdown_template::LAYOUT,text_id:"missing"})).is_err());
}
#[test]
fn dialog_spinner_accepts_custom_xml_rows() {
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=android_xml::LayoutState::default();
    state.set_spinner_item_templates("dialog_workspace",Some(android_xml::SpinnerItemTemplates {selected:&selected_template::LAYOUT,dropdown:&dropdown_template::LAYOUT,text_id:"spinner_title"})).unwrap();
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input| {let mut action=None;let commands=ui.frame(input,VIEW,0.,|f| {action=dialog_fixture::LAYOUT.render(f,BOUNDS,state);});(commands,action)};
    draw(&mut ui,&mut state,Input::default());draw(&mut ui,&mut state,Input {pointer:Some([30.,24.]),pressed:true,time:0.1,..Input::default()});
    let (commands,_)=draw(&mut ui,&mut state,Input {pointer:Some([30.,24.]),released:true,time:0.11,..Input::default()});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,style,..} if text.as_ref()=="Workspace" && style.size==10.0)));
    let item=ui.semantics().iter().find(|n|n.label.as_ref()=="Projects").unwrap().bounds;let p=[item.x+item.width*0.5,item.y+item.height*0.5];
    draw(&mut ui,&mut state,Input {pointer:Some(p),pressed:true,time:0.2,..Input::default()});
    assert_eq!(draw(&mut ui,&mut state,Input {pointer:Some(p),released:true,time:0.21,..Input::default()}).1,Some("dialog_workspace"));
}

#[test]
fn spinner_xml_rows_measure_wrapped_text_and_keyboard_scrolls_to_them() {
    static TALL_ROW:android_xml::Node=android_xml::Node {height:android_xml::Size::Wrap,min_height:64.0,..dropdown_template::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let long="A long workspace name with enough words to wrap across several lines in the popup and demonstrate independently measured rows";
    state.set_spinner_adapter("workspace",vec!["Short".into(),long.into(),"Last".into()],None,None).unwrap();
    state.set_spinner_item_templates("workspace",Some(android_xml::SpinnerItemTemplates {selected:&selected_template::LAYOUT,dropdown:&TALL_ROW,text_id:"spinner_title"})).unwrap();
    draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,30.,24.,0.1);
    draw(&mut ui,&mut state,Input {time:0.2,..Input::default()});
    let short=ui.semantics().iter().find(|n|n.label.as_ref()=="Short" && n.bounds.y>=48.0).unwrap().bounds;
    let wrapped=ui.semantics().iter().find(|n|n.label.as_ref()==long).unwrap().bounds;
    assert!(short.height>=64.0);assert!(wrapped.height>short.height,"wrapped: {}, short: {}",wrapped.height,short.height);
    let key=|key,time|Input {time,key:Some(KeyInput {key,pressed:true,repeat:false,shift:false}),..Input::default()};
    draw(&mut ui,&mut state,key(Key::Down,0.3));draw(&mut ui,&mut state,key(Key::Down,0.4));
    assert_eq!(draw(&mut ui,&mut state,key(Key::Enter,0.5)),Some("workspace"));assert_eq!(state.spinner_selected["workspace"],2);
}

#[test]
fn spinner_wraps_selected_xml_view_and_keeps_it_clear_of_rtl_arrow() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let long="A long workspace name with enough words to require multiple lines when shown in the selected view";
    state.set_spinner_adapter("workspace",vec!["Short".into(),long.into()],None,None).unwrap();
    state.set_spinner_item_templates("workspace",Some(android_xml::SpinnerItemTemplates {selected:&selected_template::LAYOUT,dropdown:&dropdown_template::LAYOUT,text_id:"spinner_title"})).unwrap();
    let bounds=Rect {height:200.0,..BOUNDS};
    let render=|ui:&mut Ui,state:&mut android_xml::LayoutState| {ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,bounds,state);})};
    render(&mut ui,&mut state);
    let short=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds;
    state.spinner_selected.insert("workspace".into(),1);state.rtl=true;
    let commands=render(&mut ui,&mut state);
    let long_bounds=ui.semantics().iter().find(|n|n.label.as_ref()==long).unwrap().bounds;
    assert!(long_bounds.height>short.height,"long {}, short {}",long_bounds.height,short.height);
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {bounds,text,..} if long.contains(text.as_ref()) && bounds.x>=40.0)),"selected text must not overlap the RTL arrow");
    state.spinner_selected.insert("workspace".into(),0);render(&mut ui,&mut state);
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds.height,short.height);
}

#[test]
fn spinner_dropdown_uses_available_space_and_measures_xml_selected_views() {
    static WIDE_SELECTED:android_xml::Node=android_xml::Node {width:android_xml::Size::Dp(250.0),..selected_template::LAYOUT};
    static ROW:android_xml::Node=android_xml::Node {height:android_xml::Size::Dp(48.0),min_height:0.0,..dropdown_template::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.set_spinner_adapter("workspace",(0..20).map(|i|format!("Item {i}")).collect(),None,None).unwrap();
    state.set_spinner_item_templates("workspace",Some(android_xml::SpinnerItemTemplates {selected:&WIDE_SELECTED,dropdown:&ROW,text_id:"spinner_title"})).unwrap();
    let narrow=Rect {width:100.0,..BOUNDS};
    let render=|ui:&mut Ui,state:&mut android_xml::LayoutState,input| {ui.frame(input,VIEW,0.,|f|{fixture::LAYOUT.render(f,narrow,state);})};
    render(&mut ui,&mut state,Input::default());
    render(&mut ui,&mut state,Input {pointer:Some([40.,24.]),pressed:true,time:0.1,..Input::default()});
    render(&mut ui,&mut state,Input {pointer:Some([40.,24.]),released:true,time:0.11,..Input::default()});
    let row=ui.semantics().iter().find(|n|n.label.as_ref()=="Item 0" && n.bounds.width>100.0).unwrap();
    assert_eq!(row.bounds.width,250.0);
    assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Item 6"),"popup should use available height beyond six 48dp rows");
    render(&mut ui,&mut state,Input {key:Some(KeyInput {key:Key::Escape,pressed:true,repeat:false,shift:false}),time:0.2,..Input::default()});
    state.spinner_selected.insert("workspace".into(),19);
    render(&mut ui,&mut state,Input {pointer:Some([40.,24.]),pressed:true,time:0.3,..Input::default()});
    render(&mut ui,&mut state,Input {pointer:Some([40.,24.]),released:true,time:0.31,..Input::default()});
    let row=ui.semantics().iter().find(|n|n.label.as_ref()=="Item 19" && n.bounds.width>100.0).unwrap();
    assert!(row.bounds.y>=48.0 && row.bounds.y+row.bounds.height<=VIEW.height,"selected row must be visible when reopened: {:?}",row.bounds);
}

#[test]
fn spinner_popup_remeasures_when_drawable_source_size_or_density_changes() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    dropdown_template::install_drawables(&mut state);
    state.images.insert("switcher_day".into(),Image::rgba(10,10,vec![255;10*10*4]).unwrap());
    state.images.insert("tall".into(),Image::rgba(10,80,vec![255;10*80*4]).unwrap());
    state.set_spinner_adapter("workspace",vec!["Alpha".into(),"Beta".into()],None,None).unwrap();
    state.set_spinner_item_templates("workspace",Some(android_xml::SpinnerItemTemplates {selected:&selected_template::LAYOUT,dropdown:&dropdown_template::LAYOUT,text_id:"spinner_title"})).unwrap();
    draw(&mut ui,&mut state,Input::default());click(&mut ui,&mut state,30.,24.,0.1);
    let row_height=|ui:&Ui|ui.semantics().iter().find(|n|n.label.as_ref()=="Beta").unwrap().bounds.height;
    let small=row_height(&ui);
    state.image_sources.insert("spinner_image".into(),"tall".into());
    draw(&mut ui,&mut state,Input {time:0.2,..Input::default()});let tall=row_height(&ui);
    assert!(tall>small,"source change must remeasure: {small} -> {tall}");
    state.image_scales.insert("tall".into(),1.5);
    draw(&mut ui,&mut state,Input {time:0.3,..Input::default()});let scaled=row_height(&ui);
    assert!(scaled>tall,"density change must remeasure: {tall} -> {scaled}");
    state.images.insert("tall".into(),Image::rgba(10,30,vec![255;10*30*4]).unwrap());
    draw(&mut ui,&mut state,Input {time:0.4,..Input::default()});
    assert!(row_height(&ui)<scaled,"replacing a drawable must invalidate row measurement");
}
