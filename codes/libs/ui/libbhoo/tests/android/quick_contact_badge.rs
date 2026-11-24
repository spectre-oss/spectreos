use bhoo::*;
use bhoo::android_xml::QuickContactTarget;
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_quick_contact_badge.rs"));}
#[test]
fn assigned_contact_activates_and_cleared_target_does_not() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let b=Rect{x:0.,y:0.,width:80.,height:80.};let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{let mut action=None;ui.frame(input,b,0.,|f|action=fixture::LAYOUT.render(f,b,state));action};
    let p=Some([40.,40.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});assert_eq!(draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()}),None);
    state.quick_contact_targets.insert("contact_badge".into(),QuickContactTarget::Email("alex@example.test".into()));
    draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:2.,..Input::default()});assert_eq!(draw(&mut ui,&mut state,Input{pointer:p,released:true,time:2.01,..Input::default()}),Some("contact_badge"));
    assert_eq!(state.quick_contact_clicked.remove("contact_badge"),Some(QuickContactTarget::Email("alex@example.test".into())));
    state.quick_contact_targets.remove("contact_badge");draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:3.,..Input::default()});assert_eq!(draw(&mut ui,&mut state,Input{pointer:p,released:true,time:3.01,..Input::default()}),None);
}
#[test]
fn changing_contact_during_press_does_not_open_replacement() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let b=Rect{x:0.,y:0.,width:80.,height:80.};let p=Some([40.,40.]);
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{let mut action=None;ui.frame(input,b,0.,|f|action=fixture::LAYOUT.render(f,b,state));action};
    state.quick_contact_targets.insert("contact_badge".into(),QuickContactTarget::Email("first@example.test".into()));draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});
    state.quick_contact_targets.insert("contact_badge".into(),QuickContactTarget::Email("second@example.test".into()));assert_eq!(draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()}),None);assert!(state.quick_contact_clicked.is_empty());
}
#[test]
fn framework_overlay_changes_on_press_and_is_hidden_without_contact() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let b=Rect{x:0.,y:0.,width:80.,height:80.};
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|ui.frame(input,b,0.,|f|{fixture::LAYOUT.render(f,b,state);});
    let overlays=|commands:Vec<DrawCommand>|commands.into_iter().filter_map(|c|match c{DrawCommand::Image{image,..}=>Some(image.pixels().to_vec()),_=>None}).collect::<Vec<_>>();
    assert!(overlays(draw(&mut ui,&mut state,Input::default())).is_empty());
    state.quick_contact_targets.insert("contact_badge".into(),QuickContactTarget::Phone("123".into()));
    let normal=overlays(draw(&mut ui,&mut state,Input{time:1.,..Input::default()}));assert_eq!(normal.len(),9);
    let pressed=overlays(draw(&mut ui,&mut state,Input{pointer:Some([40.,40.]),pressed:true,time:1.1,..Input::default()}));assert_eq!(pressed.len(),9);assert_ne!(normal[0],pressed[0]);
    let avatar=android_xml::quick_contact_default_avatar();assert_eq!((avatar.width(),avatar.height()),(32,32));
}
#[test]
fn lazy_lookup_produces_show_or_create_and_ignores_stale_responses() {
    use android_xml::QuickContactAction;
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();let b=Rect{x:0.,y:0.,width:80.,height:80.};
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{ui.frame(input,b,0.,|f|{fixture::LAYOUT.render(f,b,state);});};
    let click=|ui:&mut Ui,state:&mut android_xml::LayoutState,time|{let p=Some([40.,40.]);draw(ui,state,Input{pointer:p,pressed:true,time,..Input::default()});draw(ui,state,Input{pointer:p,released:true,time:time+0.01,..Input::default()});};
    state.assign_quick_contact("contact_badge",Some(QuickContactTarget::Email("alex@example.test".into())),true);assert!(state.quick_contact_lookups.is_empty());
    click(&mut ui,&mut state,1.);let first=state.quick_contact_lookups["contact_badge"].clone();assert!(first.open_after_lookup);
    state.assign_quick_contact("contact_badge",Some(QuickContactTarget::Phone("123".into())),false);let second=state.quick_contact_lookups["contact_badge"].clone();assert!(!second.open_after_lookup);
    assert!(!state.resolve_quick_contact("contact_badge",first.token,Some("old".into())));assert!(state.resolve_quick_contact("contact_badge",second.token,Some("content://contact/1".into())));assert!(state.quick_contact_actions.is_empty());
    click(&mut ui,&mut state,2.);assert_eq!(state.quick_contact_actions.remove("contact_badge"),Some(QuickContactAction::Show("content://contact/1".into())));
    state.assign_quick_contact("contact_badge",Some(QuickContactTarget::Email("new@example.test".into())),true);click(&mut ui,&mut state,3.);let request=state.quick_contact_lookups["contact_badge"].clone();assert!(state.resolve_quick_contact("contact_badge",request.token,None));
    assert_eq!(state.quick_contact_actions.remove("contact_badge"),Some(QuickContactAction::Create(QuickContactTarget::Email("new@example.test".into()))));
}
