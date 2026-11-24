use bhoo::*;
use bhoo::android_xml::{MediaPlayerSnapshot,MediaControllerAction};
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_media_controller.rs"));}
#[test]
fn xml_controller_reports_player_actions_and_times() {
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();fixture::install_drawables(&mut state);state.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:12000,duration_ms:125000,buffered_percent:50,playing:false});
 let b=Rect{x:0.,y:0.,width:400.,height:120.};let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{let mut a=None;let commands=ui.frame(input,b,0.,|f|a=fixture::LAYOUT.render(f,b,state));(a,commands)};
 let (_,commands)=draw(&mut ui,&mut state,Input::default());assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="0:12")));assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="2:05")));
 let bounds=ui.semantics().iter().find(|n|n.label.as_ref()=="Play").unwrap().bounds;let p=Some([bounds.x+bounds.width/2.,bounds.y+bounds.height/2.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});assert_eq!(draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()}).0,Some("controller"));assert_eq!(state.media_controller_actions.remove("controller"),Some(MediaControllerAction::Play));
 let bounds=ui.semantics().iter().find(|n|n.label.as_ref()=="Fast forward 15 seconds").unwrap().bounds;let p=Some([bounds.x+bounds.width/2.,bounds.y+bounds.height/2.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:2.,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,released:true,time:2.01,..Input::default()});assert_eq!(state.media_controller_actions.remove("controller"),Some(MediaControllerAction::SeekTo(27000)));
}
#[test]
fn seekbar_and_rewind_report_bounded_positions() {
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();fixture::install_drawables(&mut state);state.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:3000,duration_ms:100000,buffered_percent:30,playing:true});
 let b=Rect{x:0.,y:0.,width:400.,height:120.};let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{ui.frame(input,b,0.,|f|{fixture::LAYOUT.render(f,b,state);});};draw(&mut ui,&mut state,Input::default());
 let r=ui.semantics().iter().find(|n|n.label.as_ref()=="Rewind 5 seconds").unwrap().bounds;let p=Some([r.x+r.width/2.,r.y+r.height/2.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()});assert_eq!(state.media_controller_actions.remove("controller"),Some(MediaControllerAction::SeekTo(0)));
 let r=ui.semantics().iter().find(|n|n.role==Role::Slider).unwrap().bounds;let p=Some([r.x+r.width*0.75,r.y+r.height/2.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:2.,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,released:true,time:2.01,..Input::default()});
 match state.media_controller_actions.remove("controller") {Some(MediaControllerAction::SeekTo(value))=>assert!(value>50000&&value<100000),other=>panic!("unexpected {other:?}")}
}
#[test]
fn multiple_controllers_scope_child_ids_and_restore_host_values() {
 static SECOND:android_xml::Node=android_xml::Node{id:"second",..fixture::LAYOUT};
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();fixture::install_drawables(&mut state);
 state.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:12000,duration_ms:125000,buffered_percent:0,playing:false});
 state.media_players.insert("second".into(),MediaPlayerSnapshot{position_ms:61000,duration_ms:240000,buffered_percent:0,playing:true});
 state.text.insert("pause".into(),"host label".into());state.progress.insert("mediacontroller_progress".into(),42.0);
 let viewport=Rect{x:0.,y:0.,width:400.,height:240.};let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{ui.frame(input,viewport,0.,|f|{fixture::LAYOUT.render(f,Rect{x:0.,y:0.,width:400.,height:100.},state);SECOND.render(f,Rect{x:0.,y:120.,width:400.,height:100.},state);});};
 draw(&mut ui,&mut state,Input::default());assert_eq!(state.text["pause"],"host label");assert_eq!(state.progress["mediacontroller_progress"],42.0);
 let r=ui.semantics().iter().find(|node|node.label.as_ref()=="Play"&&node.bounds.y>100.).unwrap().bounds;let p=Some([r.x+r.width/2.,r.y+r.height/2.]);
 draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()});
 assert_eq!(state.media_controller_actions.remove("second"),Some(MediaControllerAction::Pause));assert!(!state.media_controller_actions.contains_key("controller"));
}
#[test]
fn unavailable_player_capabilities_disable_only_their_controls() {
 use android_xml::MediaControllerCapabilities;
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();fixture::install_drawables(&mut state);let b=Rect{x:0.,y:0.,width:400.,height:120.};
 state.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:12000,duration_ms:125000,buffered_percent:50,playing:true});
 state.media_controller_capabilities.insert("controller".into(),MediaControllerCapabilities{can_pause:false,can_seek_backward:false,can_seek_forward:false,has_previous:false,has_next:true});
 let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{ui.frame(input,b,0.,|f|{fixture::LAYOUT.render(f,b,state);});};draw(&mut ui,&mut state,Input::default());
 let enabled=|label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().enabled;
 assert!(!enabled("Play"));assert!(!enabled("Rewind 5 seconds"));assert!(!enabled("Fast forward 15 seconds"));assert!(!enabled("Previous"));assert!(enabled("Next"));assert!(!ui.semantics().iter().find(|n|n.role==Role::Slider).unwrap().enabled);
 let r=ui.semantics().iter().find(|n|n.label.as_ref()=="Play").unwrap().bounds;let p=Some([r.x+r.width/2.,r.y+r.height/2.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()});assert!(state.media_controller_actions.is_empty());
 state.media_controller_capabilities.get_mut("controller").unwrap().can_seek_forward=true;draw(&mut ui,&mut state,Input{time:2.,..Input::default()});assert!(ui.semantics().iter().find(|n|n.role==Role::Slider).unwrap().enabled);assert!(ui.semantics().iter().find(|n|n.label.as_ref()=="Fast forward 15 seconds").unwrap().enabled);
}
#[test]
fn play_image_tracks_player_state_without_mutating_host_image_binding() {
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();fixture::install_drawables(&mut state);
 state.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:0,duration_ms:100000,buffered_percent:0,playing:false});state.image_sources.insert("pause".into(),"host_icon".into());
 let b=Rect{x:0.,y:0.,width:400.,height:120.};let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState|ui.frame(Input::default(),b,0.,|f|{fixture::LAYOUT.render(f,b,state);});
 let images=|commands:Vec<DrawCommand>|commands.into_iter().filter_map(|c|match c{DrawCommand::Image{image,..}=>Some(image.id()),_=>None}).collect::<Vec<_>>();
 let paused=images(draw(&mut ui,&mut state));assert!(paused.contains(&state.images["android_media_play"].id()));assert_eq!(state.image_sources["pause"],"host_icon");
 state.media_players.get_mut("controller").unwrap().playing=true;let playing=images(draw(&mut ui,&mut state));assert!(playing.contains(&state.images["android_media_pause"].id()));assert_eq!(state.image_sources["pause"],"host_icon");
}
#[test]
fn floating_controller_timeout_and_held_seek() {
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();fixture::install_drawables(&mut state);state.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:12000,duration_ms:125000,buffered_percent:50,playing:false});
 let anchor=Rect{x:0.,y:0.,width:400.,height:240.};let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|{ui.frame(input,anchor,0.,|f|{fixture::LAYOUT.render_floating_media_controller(f,anchor,100.,state);});};
 state.show_floating_media_controller("controller",0.,3000);draw(&mut ui,&mut state,Input{time:0.,..Input::default()});assert!(state.floating_media_controller_is_showing("controller"));
 let r=ui.semantics().iter().find(|node|node.role==Role::Slider).unwrap().bounds;let p=Some([r.x+r.width*0.5,r.y+r.height/2.]);
 draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,time:4.,..Input::default()});assert!(state.floating_media_controller_is_showing("controller"));
 draw(&mut ui,&mut state,Input{pointer:p,released:true,time:4.1,..Input::default()});draw(&mut ui,&mut state,Input{time:7.2,..Input::default()});assert!(!state.floating_media_controller_is_showing("controller"));
 state.show_floating_media_controller("controller",10.,0);draw(&mut ui,&mut state,Input{time:100.,..Input::default()});assert!(state.floating_media_controller_is_showing("controller"));state.hide_floating_media_controller("controller");assert!(!state.floating_media_controller_is_showing("controller"));
}
#[test]
fn seek_drag_keeps_preview_when_player_snapshot_is_stale() {
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();fixture::install_drawables(&mut state);state.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:10000,duration_ms:100000,buffered_percent:30,playing:false});
 let b=Rect{x:0.,y:0.,width:400.,height:120.};let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|ui.frame(input,b,0.,|f|{fixture::LAYOUT.render(f,b,state);});
 draw(&mut ui,&mut state,Input::default());let r=ui.semantics().iter().find(|node|node.role==Role::Slider).unwrap().bounds;let p=Some([r.x+r.width*0.75,r.y+r.height/2.]);
 draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});let seek=match state.media_controller_actions.remove("controller"){Some(MediaControllerAction::SeekTo(value))=>value,other=>panic!("unexpected {other:?}")};assert!(seek>50000);
 let commands=draw(&mut ui,&mut state,Input{pointer:p,time:1.1,..Input::default()});assert!(!state.media_controller_actions.contains_key("controller"));let expected=format!("{}:{:02}",seek/60000,(seek/1000)%60);assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()==expected)));
 draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.2,..Input::default()});let commands=draw(&mut ui,&mut state,Input{time:1.3,..Input::default()});assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text{text,..} if text.as_ref()=="0:10")));
}
