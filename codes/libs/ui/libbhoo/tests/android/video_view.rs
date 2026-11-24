use bhoo::*;
use bhoo::android_xml::{MediaControllerCapabilities,MediaPlayerSnapshot,MediaControllerAction};
mod video {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_video_view.rs"));}
mod controller {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_media_controller.rs"));}
#[test]
fn video_frame_fit_and_touch_controller_only_when_ready() {
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();controller::install_drawables(&mut state);
 state.video_frames.insert("video".into(),Image::rgba(4,2,vec![255;32]).unwrap());state.video_controllers.insert("video".into(),("controller".into(),100.));
 state.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:0,duration_ms:10000,buffered_percent:50,playing:false});state.media_controller_capabilities.insert("controller".into(),MediaControllerCapabilities{has_next:true,has_previous:true,..Default::default()});
 let b=Rect{x:0.,y:0.,width:300.,height:220.};let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input|ui.frame(input,b,0.,|f|{video::LAYOUT.render(f,b,state);controller::LAYOUT.render_floating_media_controller(f,b,100.,state);});
 let commands=draw(&mut ui,&mut state,Input::default());let frame=commands.iter().find_map(|c|match c{DrawCommand::Image{bounds,image,..} if image.width()==4=>Some(*bounds),_=>None}).unwrap();assert_eq!((frame.width,frame.height,frame.y),(300.,150.,35.));
 let p=Some([30.,40.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:1.,..Input::default()});assert!(!state.floating_media_controller_is_showing("controller"));draw(&mut ui,&mut state,Input{pointer:p,released:true,time:1.01,..Input::default()});
 state.video_ready.insert("video".into());draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:2.,..Input::default()});assert!(state.floating_media_controller_is_showing("controller"));draw(&mut ui,&mut state,Input{pointer:p,released:true,time:2.01,..Input::default()});
 let play=ui.semantics().iter().find(|node|node.label.as_ref()=="Play").unwrap().bounds;let p=Some([play.x+play.width/2.,play.y+play.height/2.]);draw(&mut ui,&mut state,Input{pointer:p,pressed:true,time:2.2,..Input::default()});draw(&mut ui,&mut state,Input{pointer:p,released:true,time:2.21,..Input::default()});assert!(state.floating_media_controller_is_showing("controller"));assert_eq!(state.media_controller_actions.remove("controller"),Some(MediaControllerAction::Play));
}
#[test]
fn framework_video_measurement_modes_and_unknown_size() {
 use android_xml::{measure_video_view as measure,VideoMeasureSpec::*};
 assert_eq!(measure((1920,1080),Exactly(300),Exactly(220)),(300,168));
 assert_eq!(measure((1920,1080),Exactly(300),AtMost(100)),(300,100));
 assert_eq!(measure((1920,1080),AtMost(100),Exactly(220)),(100,220));
 assert_eq!(measure((1920,1080),AtMost(300),AtMost(100)),(177,100));
 assert_eq!(measure((1920,1080),Unspecified,Unspecified),(1920,1080));
 assert_eq!(measure((0,0),AtMost(300),Exactly(220)),(300,220));
}
#[test]
fn frame_parent_places_video_using_measured_aspect_ratio() {
 static CHILDREN:[android_xml::Node;1]=[video::LAYOUT];
 static PARENT:android_xml::Node=android_xml::Node{kind:android_xml::Kind::Frame,id:"parent",width:android_xml::Size::Match,height:android_xml::Size::Match,children:&CHILDREN,..video::LAYOUT};
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.video_frames.insert("video".into(),Image::rgba(4,2,vec![255;32]).unwrap());
 let b=Rect{x:0.,y:0.,width:300.,height:220.};let commands=ui.frame(Input::default(),b,0.,|f|{PARENT.render(f,b,&mut state);});
 let frame=commands.iter().find_map(|c|match c{DrawCommand::Image{bounds,image,..} if image.width()==4=>Some(*bounds),_=>None}).unwrap();assert_eq!((frame.x,frame.y,frame.width,frame.height),(0.,0.,300.,150.));
}
#[test]
fn vertical_linear_positions_sibling_after_measured_video() {
 static AFTER:android_xml::Node=android_xml::Node{kind:android_xml::Kind::View,id:"after",width:android_xml::Size::Match,height:android_xml::Size::Dp(20.),background:Some(Color(1,2,3,255)),..video::LAYOUT};
 static CHILDREN:[android_xml::Node;2]=[video::LAYOUT,AFTER];
 static PARENT:android_xml::Node=android_xml::Node{kind:android_xml::Kind::Linear,id:"parent",width:android_xml::Size::Match,height:android_xml::Size::Match,horizontal:false,children:&CHILDREN,..video::LAYOUT};
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.video_frames.insert("video".into(),Image::rgba(4,2,vec![255;32]).unwrap());
 let b=Rect{x:0.,y:0.,width:300.,height:300.};let commands=ui.frame(Input::default(),b,0.,|f|{PARENT.render(f,b,&mut state);});
 let rect=commands.iter().find_map(|c|match c{DrawCommand::RoundedRect{bounds,color:Color(1,2,3,255),..}=>Some(*bounds),_=>None}).unwrap();assert_eq!(rect.y,150.);
}
#[test]
fn horizontal_linear_positions_sibling_after_measured_video() {
 static PORTRAIT:android_xml::Node=android_xml::Node{width:android_xml::Size::Dp(300.),height:android_xml::Size::Dp(220.),..video::LAYOUT};
 static AFTER:android_xml::Node=android_xml::Node{kind:android_xml::Kind::View,id:"after",width:android_xml::Size::Dp(20.),height:android_xml::Size::Match,background:Some(Color(1,2,3,255)),..video::LAYOUT};
 static CHILDREN:[android_xml::Node;2]=[PORTRAIT,AFTER];
 static PARENT:android_xml::Node=android_xml::Node{kind:android_xml::Kind::Linear,id:"parent",width:android_xml::Size::Match,height:android_xml::Size::Match,horizontal:true,children:&CHILDREN,..video::LAYOUT};
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.video_frames.insert("video".into(),Image::rgba(2,4,vec![255;32]).unwrap());
 let b=Rect{x:0.,y:0.,width:400.,height:220.};let commands=ui.frame(Input::default(),b,0.,|f|{PARENT.render(f,b,&mut state);});
 let rect=commands.iter().find_map(|c|match c{DrawCommand::RoundedRect{bounds,color:Color(1,2,3,255),..}=>Some(*bounds),_=>None}).unwrap();assert_eq!(rect.x,110.);
}
#[test]
fn weighted_vertical_video_remeasures_share_before_positioning_sibling() {
 static WEIGHTED:android_xml::Node=android_xml::Node{height:android_xml::Size::Dp(0.),weight:1.,..video::LAYOUT};
 static AFTER:android_xml::Node=android_xml::Node{kind:android_xml::Kind::View,id:"after",width:android_xml::Size::Match,height:android_xml::Size::Dp(20.),background:Some(Color(1,2,3,255)),..video::LAYOUT};
 static CHILDREN:[android_xml::Node;2]=[WEIGHTED,AFTER];static PARENT:android_xml::Node=android_xml::Node{kind:android_xml::Kind::Linear,id:"parent",width:android_xml::Size::Match,height:android_xml::Size::Match,horizontal:false,children:&CHILDREN,..video::LAYOUT};
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.video_frames.insert("video".into(),Image::rgba(4,2,vec![255;32]).unwrap());
 let b=Rect{x:0.,y:0.,width:300.,height:300.};let commands=ui.frame(Input::default(),b,0.,|f|{PARENT.render(f,b,&mut state);});
 let rect=commands.iter().find_map(|c|match c{DrawCommand::RoundedRect{bounds,color:Color(1,2,3,255),..}=>Some(*bounds),_=>None}).unwrap();assert_eq!(rect.y,150.);
}
#[test]
fn weighted_horizontal_video_remeasures_share_before_positioning_sibling() {
 static WEIGHTED:android_xml::Node=android_xml::Node{width:android_xml::Size::Dp(0.),height:android_xml::Size::Match,weight:1.,..video::LAYOUT};
 static AFTER:android_xml::Node=android_xml::Node{kind:android_xml::Kind::View,id:"after",width:android_xml::Size::Dp(20.),height:android_xml::Size::Match,background:Some(Color(1,2,3,255)),..video::LAYOUT};
 static CHILDREN:[android_xml::Node;2]=[WEIGHTED,AFTER];static PARENT:android_xml::Node=android_xml::Node{kind:android_xml::Kind::Linear,id:"parent",width:android_xml::Size::Match,height:android_xml::Size::Match,horizontal:true,children:&CHILDREN,..video::LAYOUT};
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.video_frames.insert("video".into(),Image::rgba(2,4,vec![255;32]).unwrap());
 let b=Rect{x:0.,y:0.,width:400.,height:220.};let commands=ui.frame(Input::default(),b,0.,|f|{PARENT.render(f,b,&mut state);});
 let rect=commands.iter().find_map(|c|match c{DrawCommand::RoundedRect{bounds,color:Color(1,2,3,255),..}=>Some(*bounds),_=>None}).unwrap();assert_eq!(rect.x,110.);
}
#[test]
fn weighted_video_does_not_redistribute_unused_share() {
 static WEIGHTED:android_xml::Node=android_xml::Node{height:android_xml::Size::Dp(0.),weight:1.,..video::LAYOUT};
 static AFTER:android_xml::Node=android_xml::Node{kind:android_xml::Kind::View,id:"after",width:android_xml::Size::Match,height:android_xml::Size::Dp(0.),weight:1.,background:Some(Color(1,2,3,255)),..video::LAYOUT};
 static CHILDREN:[android_xml::Node;2]=[WEIGHTED,AFTER];static PARENT:android_xml::Node=android_xml::Node{kind:android_xml::Kind::Linear,id:"parent",width:android_xml::Size::Match,height:android_xml::Size::Match,horizontal:false,children:&CHILDREN,..video::LAYOUT};
 let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();state.video_frames.insert("video".into(),Image::rgba(4,2,vec![255;32]).unwrap());let b=Rect{x:0.,y:0.,width:300.,height:400.};
 let commands=ui.frame(Input::default(),b,0.,|f|{PARENT.render(f,b,&mut state);});let rect=commands.iter().find_map(|c|match c{DrawCommand::RoundedRect{bounds,color:Color(1,2,3,255),..}=>Some(*bounds),_=>None}).unwrap();assert_eq!((rect.y,rect.height),(150.,200.));
}
