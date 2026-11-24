use bhoo::*;
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_gridview.rs"));}
mod row {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_spinner_dropdown_row.rs"));}
const VIEW:Rect=Rect{x:0.,y:0.,width:300.,height:240.};
const BOUNDS:Rect=Rect{x:0.,y:0.,width:300.,height:200.};
fn draw(ui:&mut Ui,state:&mut android_xml::LayoutState,input:Input)->Option<&'static str>{let mut result=None;ui.frame(input,VIEW,0.,|f|{result=fixture::LAYOUT.render(f,BOUNDS,state);});result}
fn click(ui:&mut Ui,state:&mut android_xml::LayoutState,x:f32,y:f32,time:f64)->Option<&'static str>{draw(ui,state,Input {pointer:Some([x,y]),pressed:true,time,..Input::default()});draw(ui,state,Input {pointer:Some([x,y]),released:true,time:time+0.01,..Input::default()})}
#[test]
fn autofit_columns_rtl_selection_and_disabled_items() {
    for rtl in [false,true] {
        let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.rtl=rtl;
        state.set_list_adapter("workspaces_grid",vec!["A".into(),"B".into(),"C".into()],None,Some(vec![true,false,true])).unwrap();draw(&mut ui,&mut state,Input::default());
        let a=ui.semantics().iter().find(|n|n.label.as_ref()=="A").unwrap().bounds;let b=ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds;let c=ui.semantics().iter().find(|n|n.label.as_ref()=="C").unwrap().bounds;
        assert_eq!(a.width,146.0);assert_eq!(b.width,146.0);assert_eq!(a.y,0.0);assert_eq!(c.y,56.0);assert_eq!(a.x,if rtl {154.0}else{0.0});assert_eq!(b.x,if rtl {0.0}else{154.0});
        assert_eq!(click(&mut ui,&mut state,b.x+20.,b.y+20.,0.1),None);assert_eq!(click(&mut ui,&mut state,a.x+20.,a.y+20.,0.2),Some("workspaces_grid"));assert_eq!(state.list_checked["workspaces_grid"],vec![0]);
        assert_eq!(click(&mut ui,&mut state,a.x+20.,a.y+20.,0.3),Some("workspaces_grid"));assert!(state.list_checked["workspaces_grid"].is_empty());
    }
}
#[test]
fn xml_items_measure_variable_height_per_row() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let long="A workspace name that wraps over several lines within a narrow grid column";
    state.list_items.insert("workspaces_grid".into(),vec!["Short".into(),long.into(),"Next".into()]);
    row::install_drawables(&mut state);state.image_scales.insert("switcher_day".into(),0.5);
    state.set_list_item_template("workspaces_grid",Some(android_xml::ListItemTemplate {row:&row::LAYOUT,text_id:"spinner_title"})).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    let a=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds;let b=ui.semantics().iter().find(|n|n.label.as_ref()==long).unwrap().bounds;
    assert!(b.height>a.height);assert_eq!(a.height,64.0);assert!(commands.iter().any(|c|matches!(c,DrawCommand::Image {..})));
    state.image_scales.insert("switcher_day".into(),1.5);draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});
    let updated=ui.semantics().iter().find(|n|n.label.as_ref()=="Short").unwrap().bounds;assert!(updated.height>a.height);
}
#[test]
fn large_grid_touch_scroll_virtualizes_cells_without_selecting() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.list_items.insert("workspaces_grid".into(),(0..100_000).map(|i|format!("Item {i}")).collect());
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});assert!(ui.semantics().len()<20);assert!(commands.len()<150);
    let touch=|y,pressed,released,time|Input {pointer:Some([50.,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(180.,true,false,0.1));draw(&mut ui,&mut state,touch(40.,false,false,0.2));assert_eq!(draw(&mut ui,&mut state,touch(40.,false,true,0.3)),None);
    assert!(state.offsets["workspaces_grid"]>50.0);assert!(!state.list_clicked.contains_key("workspaces_grid"));assert!(ui.semantics().len()<20);
}

#[test]
fn fixed_column_stretch_modes_follow_requested_geometry() {
    static RULES_NONE:&[android_xml::RelativeRule]=&[android_xml::RelativeRule {kind:91,anchor:"2"},android_xml::RelativeRule {kind:92,anchor:"100"},android_xml::RelativeRule {kind:93,anchor:"8"},android_xml::RelativeRule {kind:95,anchor:"none"}];
    static RULES_SPACING:&[android_xml::RelativeRule]=&[android_xml::RelativeRule {kind:91,anchor:"2"},android_xml::RelativeRule {kind:92,anchor:"100"},android_xml::RelativeRule {kind:93,anchor:"8"},android_xml::RelativeRule {kind:95,anchor:"spacingWidth"}];
    static RULES_UNIFORM:&[android_xml::RelativeRule]=&[android_xml::RelativeRule {kind:91,anchor:"2"},android_xml::RelativeRule {kind:92,anchor:"100"},android_xml::RelativeRule {kind:93,anchor:"8"},android_xml::RelativeRule {kind:95,anchor:"spacingWidthUniform"}];
    static NONE:android_xml::Node=android_xml::Node {relative_rules:RULES_NONE,..fixture::LAYOUT};
    static SPACING:android_xml::Node=android_xml::Node {relative_rules:RULES_SPACING,..fixture::LAYOUT};
    static UNIFORM:android_xml::Node=android_xml::Node {relative_rules:RULES_UNIFORM,..fixture::LAYOUT};
    for (layout,x1,x2) in [(&NONE,0.0,108.0),(&SPACING,0.0,200.0),(&UNIFORM,38.666668,177.33334)] {
        let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.list_items.insert("workspaces_grid".into(),vec!["A".into(),"B".into()]);
        ui.frame(Input::default(),VIEW,0.,|f|{layout.render(f,BOUNDS,&mut state);});
        let a=ui.semantics().iter().find(|n|n.label.as_ref()=="A").unwrap().bounds;let b=ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds;
        assert!((a.x-x1).abs()<0.01 && (b.x-x2).abs()<0.01,"A: {a:?}, B: {b:?}");assert!((a.width-100.0).abs()<0.01);
    }
}

#[test]
fn wrap_content_grid_measures_all_rows_and_spacing() {
    static GRID:android_xml::Node=android_xml::Node {height:android_xml::Size::Wrap,..fixture::LAYOUT};
    static AFTER:android_xml::Node=android_xml::Node {id:"after",text:"After",height:android_xml::Size::Dp(20.0),..row::LAYOUT.children[0]};
    static ROOT:android_xml::Node=android_xml::Node {id:"root",kind:android_xml::Kind::Linear,height:android_xml::Size::Match,horizontal:false,children:&[GRID,AFTER],padding_edges:[0.;4],content_gravity:[0,0],..row::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.list_items.insert("workspaces_grid".into(),vec!["A".into(),"B".into(),"C".into()]);
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{ROOT.render(f,BOUNDS,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.as_ref()=="After" && (bounds.y-104.0).abs()<0.01)));
}
#[test]
fn large_xml_grid_only_builds_visible_cells() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.list_items.insert("workspaces_grid".into(),(0..100_000).map(|i|format!("Workspace {i}")).collect());
    state.set_list_item_template("workspaces_grid",Some(android_xml::ListItemTemplate {row:&row::LAYOUT,text_id:"spinner_title"})).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});assert!(commands.len()<300);assert!(ui.semantics().len()<20);
    state.offsets.insert("workspaces_grid".into(),2_000_000.0);draw(&mut ui,&mut state,Input {time:0.2,..Input::default()});assert!(ui.semantics().len()<20);assert!(ui.semantics().iter().any(|n|n.label.as_ref().starts_with("Workspace ")));
}

#[test]
fn grid_long_press_fires_once_without_selecting_and_disabled_cell_ignores_it() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_grid";
    state.set_list_adapter(id,vec!["A".into(),"B".into()],Some(vec![10,20]),Some(vec![true,false])).unwrap();state.list_long_click_enabled.insert(id.into(),true);
    draw(&mut ui,&mut state,Input::default());
    let touch=|x,pressed,released,time|Input {pointer:Some([x,24.]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(200.,true,false,0.1));assert_eq!(draw(&mut ui,&mut state,touch(200.,false,false,0.7)),None);draw(&mut ui,&mut state,touch(200.,false,true,0.8));assert!(!state.list_long_clicked.contains_key(id));
    draw(&mut ui,&mut state,touch(30.,true,false,1.0));assert_eq!(draw(&mut ui,&mut state,touch(30.,false,false,1.49)),None);
    assert_eq!(draw(&mut ui,&mut state,touch(30.,false,false,1.51)),Some(id));assert_eq!(state.list_long_clicked.remove(id),Some(0));
    assert_eq!(draw(&mut ui,&mut state,touch(30.,false,false,1.8)),None);assert_eq!(draw(&mut ui,&mut state,touch(30.,false,true,1.9)),None);
    assert!(!state.list_clicked.contains_key(id));assert!(state.list_checked[id].is_empty());
}
#[test]
fn grid_long_press_is_cancelled_by_scrolling_and_adapter_update() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_grid";
    state.set_list_adapter(id,(0..30).map(|i|format!("Item {i}")).collect(),None,None).unwrap();state.list_long_click_enabled.insert(id.into(),true);
    draw(&mut ui,&mut state,Input::default());
    let touch=|y,pressed,released,time|Input {pointer:Some([30.,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(160.,true,false,0.1));draw(&mut ui,&mut state,touch(80.,false,false,0.2));assert_eq!(draw(&mut ui,&mut state,touch(80.,false,false,0.7)),None);draw(&mut ui,&mut state,touch(80.,false,true,0.8));
    assert!(!state.list_long_clicked.contains_key(id));
    state.offsets.insert(id.into(),0.0);draw(&mut ui,&mut state,Input {time:1.0,..Input::default()});draw(&mut ui,&mut state,touch(24.,true,false,1.1));
    state.set_list_adapter(id,vec!["Replacement".into()],None,None).unwrap();assert_eq!(draw(&mut ui,&mut state,touch(24.,false,false,1.7)),None);assert!(!state.list_long_clicked.contains_key(id));
}

#[test]
fn fixed_xml_item_width_follows_horizontal_gravity_and_rtl() {
    static ITEM:android_xml::Node=android_xml::Node {width:android_xml::Size::Dp(72.0),height:android_xml::Size::Dp(64.0),..row::LAYOUT};
    static START:android_xml::Node=android_xml::Node {content_gravity:[0,0],..fixture::LAYOUT};
    static CENTER:android_xml::Node=android_xml::Node {content_gravity:[1,0],..fixture::LAYOUT};
    static END:android_xml::Node=android_xml::Node {content_gravity:[2,0],..fixture::LAYOUT};
    static LEFT:android_xml::Node=android_xml::Node {content_gravity:[3,0],..fixture::LAYOUT};
    static RIGHT:android_xml::Node=android_xml::Node {content_gravity:[4,0],..fixture::LAYOUT};
    for (layout,rtl,expected) in [(&START,false,0.0),(&START,true,228.0),(&CENTER,false,37.0),(&CENTER,true,191.0),(&END,false,74.0),(&END,true,154.0),(&LEFT,true,154.0),(&RIGHT,true,228.0)] {
        let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.rtl=rtl;state.list_items.insert("workspaces_grid".into(),vec!["A".into(),"B".into()]);
        state.set_list_item_template("workspaces_grid",Some(android_xml::ListItemTemplate {row:&ITEM,text_id:"spinner_title"})).unwrap();
        ui.frame(Input::default(),VIEW,0.,|f|{layout.render(f,BOUNDS,&mut state);});
        let a=ui.semantics().iter().find(|n|n.label.as_ref()=="A").unwrap().bounds;
        assert!((a.x-expected).abs()<0.01 && (a.width-72.0).abs()<0.01 && (a.height-64.0).abs()<0.01,"rtl={rtl}, {a:?}");
    }
}
#[test]
fn taps_outside_narrow_item_do_not_select_it() {
    static ITEM:android_xml::Node=android_xml::Node {width:android_xml::Size::Dp(72.0),height:android_xml::Size::Dp(64.0),..row::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.list_items.insert("workspaces_grid".into(),vec!["A".into()]);
    state.set_list_item_template("workspaces_grid",Some(android_xml::ListItemTemplate {row:&ITEM,text_id:"spinner_title"})).unwrap();draw(&mut ui,&mut state,Input::default());
    assert_eq!(click(&mut ui,&mut state,100.,20.,0.1),None);assert_eq!(click(&mut ui,&mut state,40.,20.,0.2),Some("workspaces_grid"));
}

#[test]
fn wrap_content_item_width_remeasures_per_label() {
    static ITEM:android_xml::Node=android_xml::Node {width:android_xml::Size::Wrap,..row::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.list_items.insert("workspaces_grid".into(),vec!["A".into(),"Longer item title".into()]);
    state.set_list_item_template("workspaces_grid",Some(android_xml::ListItemTemplate {row:&ITEM,text_id:"spinner_title"})).unwrap();draw(&mut ui,&mut state,Input::default());
    let a=ui.semantics().iter().find(|n|n.label.as_ref()=="A").unwrap().bounds;let b=ui.semantics().iter().find(|n|n.label.as_ref()=="Longer item title").unwrap().bounds;
    assert!(a.width<b.width && b.width<=146.0,"{a:?}, {b:?}");
}

mod stateful_item {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_gridview_item.rs"));}
#[test]
fn xml_item_background_tracks_pressed_activated_and_disabled_states() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_grid";
    state.list_items.insert(id.into(),vec!["A".into(),"B".into()]);stateful_item::install_drawables(&mut state);
    state.set_list_item_template(id,Some(android_xml::ListItemTemplate {row:&stateful_item::LAYOUT,text_id:"grid_item_title"})).unwrap();
    let render=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|ui.frame(input,VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,state);});
    let has=|commands:&[DrawCommand],r,g,b|commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect {color:Color(red,green,blue,_),..} if (*red,*green,*blue)==(r,g,b)));
    let commands=render(&mut ui,&mut state,Input::default());assert!(has(&commands,228,241,231));
    let commands=render(&mut ui,&mut state,Input {pointer:Some([30.,24.]),pressed:true,time:0.1,..Input::default()});assert!(has(&commands,143,185,155),"pressed XML background");
    render(&mut ui,&mut state,Input {pointer:Some([30.,24.]),released:true,time:0.11,..Input::default()});
    let commands=render(&mut ui,&mut state,Input {time:0.2,..Input::default()});assert!(has(&commands,168,213,178),"activated XML background");assert_eq!(state.list_checked[id],vec![0]);
    state.list_enabled.insert(id.into(),vec![false,true]);let commands=render(&mut ui,&mut state,Input {time:0.3,..Input::default()});assert!(has(&commands,198,206,200),"disabled XML background");
    state.list_enabled.remove(id);state.list_checked.remove(id);let commands=render(&mut ui,&mut state,Input {time:0.4,..Input::default()});assert!(!has(&commands,168,213,178),"activation must not leak between recycled rows");
}

#[test]
fn grid_binds_per_item_text_and_images_without_leaking_between_cells() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_grid";
    state.set_list_adapter(id,vec!["A".into(),"B".into(),"C".into()],Some(vec![10,20,30]),None).unwrap();stateful_item::install_drawables(&mut state);
    let alt=Image::rgba(2,2,vec![255;16]).unwrap();let alt_id=alt.id();state.images.insert("alt".into(),alt);
    state.set_list_item_template(id,Some(android_xml::ListItemTemplate {row:&stateful_item::LAYOUT,text_id:"grid_item_title"})).unwrap();
    let mut binding=android_xml::AdapterItemBinding::default();binding.text.insert("grid_item_subtitle".into(),"Team".into());binding.images.insert("grid_item_picture".into(),"alt".into());
    state.set_list_item_binding(id,1,Some(binding)).unwrap();assert!(state.set_list_item_binding(id,8,None).is_err());
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    assert_eq!(commands.iter().filter(|c|matches!(c,DrawCommand::Text {text,..} if text.as_ref()=="Team")).count(),1);
    assert_eq!(commands.iter().filter(|c|matches!(c,DrawCommand::Text {text,..} if text.as_ref()=="Workspace")).count(),2);
    assert_eq!(commands.iter().filter(|c|matches!(c,DrawCommand::Image {image,..} if image.id()==alt_id)).count(),1);
    state.set_list_adapter(id,vec!["C".into(),"B".into(),"A".into()],Some(vec![30,20,10]),None).unwrap();
    let commands=ui.frame(Input {time:0.1,..Input::default()},VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    let b=ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds;
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.as_ref()=="Team" && bounds.x>=b.x && bounds.x<b.x+b.width)));
    state.set_list_adapter(id,vec!["C".into(),"A".into(),"B".into()],Some(vec![30,10,20]),None).unwrap();
    let commands=ui.frame(Input {time:0.2,..Input::default()},VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    let b=ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds;
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.as_ref()=="Team" && bounds.x>=b.x && bounds.y>=b.y && bounds.y<b.y+b.height)));
    state.set_list_item_binding(id,2,None).unwrap();let commands=ui.frame(Input {time:0.3,..Input::default()},VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});assert!(!commands.iter().any(|c|matches!(c,DrawCommand::Text {text,..} if text.as_ref()=="Team")));
}
#[test]
fn grid_item_binding_edits_remeasure_variable_row_height() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_grid";
    state.list_items.insert(id.into(),vec!["A".into(),"B".into()]);stateful_item::install_drawables(&mut state);
    state.set_list_item_template(id,Some(android_xml::ListItemTemplate {row:&stateful_item::LAYOUT,text_id:"grid_item_title"})).unwrap();
    draw(&mut ui,&mut state,Input::default());let before=ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds.height;
    let mut binding=android_xml::AdapterItemBinding::default();binding.text.insert("grid_item_subtitle".into(),"This is a very long subtitle that wraps over multiple lines".into());state.set_list_item_binding(id,1,Some(binding)).unwrap();
    draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});let after=ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds.height;assert!(after>before,"{before} -> {after}");
}

mod team_item {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_gridview_team_item.rs"));}
#[test]
fn grid_mixes_xml_item_layouts_and_keeps_them_with_stable_ids() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_grid";
    state.set_list_adapter(id,vec!["A".into(),"B".into(),"C".into()],Some(vec![10,20,30]),None).unwrap();
    stateful_item::install_drawables(&mut state);team_item::install_drawables(&mut state);
    state.set_list_item_template(id,Some(android_xml::ListItemTemplate {row:&stateful_item::LAYOUT,text_id:"grid_item_title"})).unwrap();
    let mut binding=android_xml::AdapterItemBinding::default();binding.template=Some(android_xml::ListItemTemplate {row:&team_item::LAYOUT,text_id:"team_title"});binding.text.insert("team_subtitle".into(),"Shared team".into());state.set_list_item_binding(id,1,Some(binding)).unwrap();
    let commands=ui.frame(Input::default(),VIEW,0.,|f|{fixture::LAYOUT.render(f,BOUNDS,&mut state);});
    let a=ui.semantics().iter().find(|n|n.label.as_ref()=="A").unwrap().bounds;let b=ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds;
    assert_eq!(a.width,72.0);assert_eq!(b.width,104.0);assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,..} if text.as_ref()=="Shared team" || text.as_ref()=="Shared")));
    assert_eq!(click(&mut ui,&mut state,b.x+10.,b.y+10.,0.1),Some(id));assert_eq!(state.list_checked[id],vec![1]);
    state.set_list_adapter(id,vec!["B".into(),"A".into(),"C".into()],Some(vec![20,10,30]),None).unwrap();draw(&mut ui,&mut state,Input {time:0.2,..Input::default()});
    assert_eq!(state.list_checked[id],vec![0]);assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds.width,104.0);
    assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="A").unwrap().bounds.width,72.0);
}
#[test]
fn per_item_template_works_without_adapter_default_and_rejects_interactive_layout() {
    static INTERACTIVE:android_xml::Node=android_xml::Node {clickable:true,..team_item::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_grid";
    state.list_items.insert(id.into(),vec!["A".into(),"B".into()]);
    let mut binding=android_xml::AdapterItemBinding::default();binding.template=Some(android_xml::ListItemTemplate {row:&team_item::LAYOUT,text_id:"team_title"});state.set_list_item_binding(id,1,Some(binding.clone())).unwrap();
    binding.template=Some(android_xml::ListItemTemplate {row:&INTERACTIVE,text_id:"team_title"});assert!(state.set_list_item_binding(id,1,Some(binding)).is_err());
    draw(&mut ui,&mut state,Input::default());let a=ui.semantics().iter().find(|n|n.label.as_ref()=="A").unwrap().bounds;let b=ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds;
    assert_eq!(a.width,146.0);assert_eq!(b.width,104.0);
}

#[test]
fn changing_an_item_layout_updates_visible_geometry() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let id="workspaces_grid";
    state.list_items.insert(id.into(),vec!["A".into(),"B".into()]);
    state.set_list_item_template(id,Some(android_xml::ListItemTemplate {row:&stateful_item::LAYOUT,text_id:"grid_item_title"})).unwrap();
    draw(&mut ui,&mut state,Input::default());assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds.width,72.0);
    let mut binding=android_xml::AdapterItemBinding::default();binding.template=Some(android_xml::ListItemTemplate {row:&team_item::LAYOUT,text_id:"team_title"});state.set_list_item_binding(id,1,Some(binding)).unwrap();
    draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds.width,104.0);
    state.set_list_item_binding(id,1,None).unwrap();draw(&mut ui,&mut state,Input {time:0.2,..Input::default()});assert_eq!(ui.semantics().iter().find(|n|n.label.as_ref()=="B").unwrap().bounds.width,72.0);
}
mod interactive_item {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_list_interactive_row.rs"));}
#[test]
fn interactive_grid_checkbox_follows_stable_item_on_reorder() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.set_list_adapter("workspaces_grid",vec!["First".into(),"Second".into()],Some(vec![101,202]),None).unwrap();
    state.set_interactive_list_item_template("workspaces_grid",Some(android_xml::ListItemTemplate{row:&interactive_item::LAYOUT,text_id:"row_title"})).unwrap();
    draw(&mut ui,&mut state,Input::default());
    let r=ui.semantics().iter().find(|n|n.role==Role::Checkbox).unwrap().bounds;let p=Some([r.x+r.width/2.,r.y+r.height/2.]);
    draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});
    assert_eq!(draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()}),Some("workspaces_grid"));
    assert_eq!(state.list_control_clicked["workspaces_grid"],(0,"row_check"));
    assert_eq!(state.list_item_control_checked("workspaces_grid",101,"row_check"),Some(true));
    assert!(state.list_clicked.get("workspaces_grid").is_none());
    state.set_list_adapter("workspaces_grid",vec!["Second".into(),"First".into()],Some(vec![202,101]),None).unwrap();
    draw(&mut ui,&mut state,Input{time:1.2,..Input::default()});
    assert_eq!(state.list_item_control_checked("workspaces_grid",101,"row_check"),Some(true));
    assert_eq!(state.list_item_control_checked("workspaces_grid",202,"row_check"),Some(false));
}
#[test]
fn mixed_grid_item_controls_follow_stable_binding_on_reorder() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.set_list_adapter("workspaces_grid",vec!["First".into(),"Second".into()],Some(vec![101,202]),None).unwrap();
    let mut binding=android_xml::AdapterItemBinding::default();binding.template=Some(android_xml::ListItemTemplate{row:&interactive_item::LAYOUT,text_id:"row_title"});
    state.set_interactive_list_item_binding("workspaces_grid",1,Some(binding)).unwrap();draw(&mut ui,&mut state,Input::default());
    let checks=ui.semantics().iter().filter(|node|node.role==Role::Checkbox).collect::<Vec<_>>();assert_eq!(checks.len(),1);let r=checks[0].bounds;let p=Some([r.x+r.width/2.,r.y+r.height/2.]);
    draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()});
    assert_eq!(state.list_control_clicked["workspaces_grid"],(1,"row_check"));assert_eq!(state.list_item_control_checked("workspaces_grid",202,"row_check"),Some(true));
    state.set_list_adapter("workspaces_grid",vec!["Second".into(),"First".into()],Some(vec![202,101]),None).unwrap();draw(&mut ui,&mut state,Input{time:1.2,..Input::default()});
    assert_eq!(ui.semantics().iter().filter(|node|node.role==Role::Checkbox).count(),1);assert_eq!(state.list_item_control_checked("workspaces_grid",202,"row_check"),Some(true));
}
