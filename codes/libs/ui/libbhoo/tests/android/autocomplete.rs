use bhoo::*;
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_autocomplete.rs"));}
const VIEW:Rect=Rect{x:0.,y:0.,width:300.,height:400.};
const BOUNDS:Rect=Rect{x:0.,y:0.,width:300.,height:64.};
fn draw(ui:&mut Ui,state:&mut android_xml::LayoutState,input:Input)->Option<&'static str>{let mut action=None;ui.frame(input,VIEW,0.,|f|{action=fixture::LAYOUT.render(f,BOUNDS,state);});action}
fn setup()->(Ui,android_xml::LayoutState){let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.set_autocomplete_items("city",["San Diego","San Francisco","Santa Fe","New York","Yorkshire"].into_iter().map(str::to_owned).collect());draw(&mut ui,&mut state,Input::default());ui.adapter_focus(Some(android_xml::view_id("city")));(ui,state)}
#[test]
fn filters_after_threshold_by_case_insensitive_word_prefix() {
    let (mut ui,mut state)=setup();state.text.insert("city".into(),"s".into());draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="San Diego"));
    state.text.insert("city".into(),"sA".into());draw(&mut ui,&mut state,Input {time:0.2,..Input::default()});assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="San Diego"));assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Santa Fe"));assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="New York"));
    state.text.insert("city".into(),"yo".into());draw(&mut ui,&mut state,Input {time:0.3,..Input::default()});assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="New York"));assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="Yorkshire"));
}
#[test]
fn click_suggestion_commits_text_and_dismisses_until_edited() {
    let (mut ui,mut state)=setup();state.text.insert("city".into(),"sa".into());draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});
    let item=ui.semantics().iter().find(|n|n.label.as_ref()=="San Francisco").unwrap().bounds;let point=[item.x+10.,item.y+10.];
    draw(&mut ui,&mut state,Input {pointer:Some(point),pressed:true,time:0.2,..Input::default()});
    assert_eq!(draw(&mut ui,&mut state,Input {pointer:Some(point),released:true,time:0.21,..Input::default()}),Some("city"));
    assert_eq!(state.text["city"],"San Francisco");assert_eq!(state.autocomplete_selected["city"],1);
    draw(&mut ui,&mut state,Input {time:0.3,..Input::default()});assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="San Diego"));
    state.text.insert("city".into(),"san".into());draw(&mut ui,&mut state,Input {time:0.4,..Input::default()});assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="San Diego"));
}
#[test]
fn outside_and_escape_dismiss_without_committing() {
    let (mut ui,mut state)=setup();state.text.insert("city".into(),"sa".into());draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});
    draw(&mut ui,&mut state,Input {pointer:Some([290.,390.]),pressed:true,time:0.2,..Input::default()});assert_eq!(state.text["city"],"sa");
    draw(&mut ui,&mut state,Input {time:0.3,..Input::default()});assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="San Diego"));
    state.text.insert("city".into(),"san".into());draw(&mut ui,&mut state,Input {time:0.4,..Input::default()});
    draw(&mut ui,&mut state,Input {time:0.5,key:Some(KeyInput {key:Key::Escape,pressed:true,repeat:false,shift:false}),..Input::default()});
    draw(&mut ui,&mut state,Input {time:0.6,..Input::default()});assert_eq!(state.text["city"],"san");assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="San Diego"));
}
#[test]
fn arrows_and_enter_select_suggestion() {
    let (mut ui,mut state)=setup();state.text.insert("city".into(),"sa".into());draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});
    let key=|key,time|Input {time,key:Some(KeyInput {key,pressed:true,repeat:false,shift:false}),..Input::default()};
    draw(&mut ui,&mut state,key(Key::Down,0.2));draw(&mut ui,&mut state,key(Key::Down,0.3));assert_eq!(draw(&mut ui,&mut state,key(Key::Enter,0.4)),Some("city"));assert_eq!(state.text["city"],"San Francisco");
}
#[test]
fn long_suggestion_list_scrolls_without_selecting_on_drag() {
    let (mut ui,mut state)=setup();state.set_autocomplete_items("city",(0..50).map(|i|format!("City {i}")).collect());state.text.insert("city".into(),"ci".into());draw(&mut ui,&mut state,Input {time:0.1,..Input::default()});
    let touch=|y,pressed,released,time|Input {pointer:Some([100.,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
    draw(&mut ui,&mut state,touch(250.,true,false,0.2));draw(&mut ui,&mut state,touch(100.,false,false,0.3));assert_eq!(draw(&mut ui,&mut state,touch(100.,false,true,0.4)),None);assert_eq!(state.text["city"],"ci");assert!(ui.semantics().iter().any(|n|n.label.as_ref()=="City 6"));
}
