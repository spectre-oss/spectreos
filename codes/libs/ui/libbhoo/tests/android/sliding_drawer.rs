use bhoo::*;
use bhoo::android_xml::LayoutState;
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_sliding_drawer.rs"));}
const B:Rect=Rect{x:0.,y:0.,width:300.,height:240.};
fn draw(ui:&mut Ui,s:&mut LayoutState,node:&'static android_xml::Node,input:Input)->Option<&'static str>{let mut action=None;ui.frame(input,B,0.,|f|action=node.render(f,B,s));action}
fn touch(x:f32,y:f32,pressed:bool,released:bool,time:f64)->Input{Input{pointer:Some([x,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()}}
#[test] fn tapping_handle_opens_and_closes_vertical_drawer(){let mut ui=Ui::default();let mut s=LayoutState::default();let n=&fixture::LAYOUT;draw(&mut ui,&mut s,n,Input::default());let r=ui.semantics().iter().find(|v|v.label.as_ref()=="Open drawer").unwrap().bounds;assert_eq!(r.y,192.);draw(&mut ui,&mut s,n,touch(150.,216.,true,false,1.));assert_eq!(draw(&mut ui,&mut s,n,touch(150.,216.,false,true,1.1)),Some("drawer"));draw(&mut ui,&mut s,n,Input{time:2.,..Input::default()});assert!(s.sliding_drawers["drawer"].open);assert_eq!(ui.semantics().iter().find(|v|v.label.as_ref()=="Close drawer").unwrap().bounds.y,0.);draw(&mut ui,&mut s,n,touch(150.,24.,true,false,2.1));draw(&mut ui,&mut s,n,touch(150.,24.,false,true,2.2));draw(&mut ui,&mut s,n,Input{time:3.,..Input::default()});assert!(!s.sliding_drawers["drawer"].open);}
#[test] fn drag_opens_and_short_drag_returns_closed(){let mut ui=Ui::default();let mut s=LayoutState::default();let n=&fixture::LAYOUT;draw(&mut ui,&mut s,n,Input::default());draw(&mut ui,&mut s,n,touch(150.,216.,true,false,1.));draw(&mut ui,&mut s,n,touch(150.,175.,false,false,1.1));draw(&mut ui,&mut s,n,touch(150.,175.,false,true,1.4));draw(&mut ui,&mut s,n,Input{time:2.,..Input::default()});assert!(!s.sliding_drawers["drawer"].open);draw(&mut ui,&mut s,n,touch(150.,216.,true,false,2.1));draw(&mut ui,&mut s,n,touch(150.,70.,false,false,2.2));draw(&mut ui,&mut s,n,touch(150.,70.,false,true,2.3));draw(&mut ui,&mut s,n,Input{time:3.,..Input::default()});assert!(s.sliding_drawers["drawer"].open);}
#[test] fn horizontal_drawer_moves_from_right_to_left(){static H:android_xml::Node=android_xml::Node{horizontal:true,..fixture::LAYOUT};let mut ui=Ui::default();let mut s=LayoutState::default();draw(&mut ui,&mut s,&H,Input::default());assert_eq!(ui.semantics().iter().find(|v|v.label.as_ref()=="Open drawer").unwrap().bounds.x,228.);draw(&mut ui,&mut s,&H,touch(260.,120.,true,false,1.));draw(&mut ui,&mut s,&H,touch(260.,120.,false,true,1.1));draw(&mut ui,&mut s,&H,Input{time:2.,..Input::default()});assert!(s.sliding_drawers["drawer"].open);assert_eq!(ui.semantics().iter().find(|v|v.label.as_ref()=="Close drawer").unwrap().bounds.x,0.);}

#[test] fn short_fast_fling_opens_and_reversing_fling_closes(){let mut ui=Ui::default();let mut s=LayoutState::default();let n=&fixture::LAYOUT;draw(&mut ui,&mut s,n,touch(150.,216.,true,false,1.));draw(&mut ui,&mut s,n,touch(150.,190.,false,false,1.05));draw(&mut ui,&mut s,n,touch(150.,190.,false,true,1.06));draw(&mut ui,&mut s,n,Input{time:2.,..Input::default()});assert!(s.sliding_drawers["drawer"].open);draw(&mut ui,&mut s,n,touch(150.,24.,true,false,2.1));draw(&mut ui,&mut s,n,touch(150.,50.,false,false,2.15));draw(&mut ui,&mut s,n,touch(150.,50.,false,true,2.16));draw(&mut ui,&mut s,n,Input{time:3.,..Input::default()});assert!(!s.sliding_drawers["drawer"].open);}
#[test] fn cancelled_drag_returns_to_original_state(){let mut ui=Ui::default();let mut s=LayoutState::default();let n=&fixture::LAYOUT;draw(&mut ui,&mut s,n,touch(150.,216.,true,false,1.));draw(&mut ui,&mut s,n,touch(150.,60.,false,false,1.1));draw(&mut ui,&mut s,n,Input{pointer:Some([150.,60.]),cancelled:true,time:1.2,..Input::default()});draw(&mut ui,&mut s,n,Input{time:2.,..Input::default()});assert!(!s.sliding_drawers["drawer"].open);}
#[test] fn lock_blocks_touch_but_not_programmatic_open_and_close(){let mut ui=Ui::default();let mut s=LayoutState::default();let n=&fixture::LAYOUT;s.set_sliding_drawer_locked("drawer",true);draw(&mut ui,&mut s,n,touch(150.,216.,true,false,1.));draw(&mut ui,&mut s,n,touch(150.,216.,false,true,1.1));draw(&mut ui,&mut s,n,Input{time:2.,..Input::default()});assert!(!s.sliding_drawers["drawer"].open);s.set_sliding_drawer_open("drawer",true,false);draw(&mut ui,&mut s,n,Input{time:2.1,..Input::default()});assert!(s.sliding_drawers["drawer"].open);s.set_sliding_drawer_open("drawer",false,true);draw(&mut ui,&mut s,n,Input{time:2.2,..Input::default()});draw(&mut ui,&mut s,n,Input{time:3.,..Input::default()});assert!(!s.sliding_drawers["drawer"].open);s.set_sliding_drawer_locked("drawer",false);draw(&mut ui,&mut s,n,touch(150.,216.,true,false,3.1));draw(&mut ui,&mut s,n,touch(150.,216.,false,true,3.2));draw(&mut ui,&mut s,n,Input{time:4.,..Input::default()});assert!(s.sliding_drawers["drawer"].open);}
mod nested {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_sliding_drawer_nested.rs"));}
#[test] fn nested_handle_and_content_work_in_both_orientations(){
    static H:android_xml::Node=android_xml::Node{horizontal:true,..nested::LAYOUT};
    for (node,horizontal) in [(&nested::LAYOUT,false),(&H,true)] {
        let(mut ui,mut state)=(Ui::default(),LayoutState::default());ui.motion_preference=MotionPreference::Reduced;
        draw(&mut ui,&mut state,node,Input::default());
        let handle=ui.semantics().iter().find(|v|v.label.as_ref()=="Open drawer").unwrap().bounds;
        let(x,y)=(handle.x+handle.width/2.,handle.y+handle.height/2.);
        draw(&mut ui,&mut state,node,touch(x,y,true,false,1.));draw(&mut ui,&mut state,node,touch(x,y,false,true,1.1));
        assert!(state.sliding_drawers["drawer"].open);
        draw(&mut ui,&mut state,node,Input{time:1.2,..Input::default()});
        let action=ui.semantics().iter().find(|v|v.label.as_ref()=="Nested action").unwrap().bounds;
        let(x,y)=(action.x+action.width/2.,action.y+action.height/2.);
        draw(&mut ui,&mut state,node,touch(x,y,true,false,1.3));assert_eq!(draw(&mut ui,&mut state,node,touch(x,y,false,true,1.4)),Some("nested_action"));
        let close=ui.semantics().iter().find(|v|v.label.as_ref()=="Close drawer").unwrap().bounds;
        assert_eq!(if horizontal {close.x}else{close.y},0.);
    }
}
#[test] fn drawer_state_events_arrive_after_changes_and_are_drained_once(){
    use bhoo::android_xml::SlidingDrawerEvent::{Opened,Closed,ScrollStarted,ScrollEnded};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());let node=&fixture::LAYOUT;
    state.set_sliding_drawer_open("drawer",true,true);draw(&mut ui,&mut state,node,Input{time:1.,..Input::default()});
    assert_eq!(state.take_sliding_drawer_events("drawer"),vec![ScrollStarted,ScrollEnded]);
    draw(&mut ui,&mut state,node,Input{time:2.,..Input::default()});assert_eq!(state.take_sliding_drawer_events("drawer"),vec![Opened]);
    assert!(state.take_sliding_drawer_events("drawer").is_empty());
    state.set_sliding_drawer_open("drawer",true,false);draw(&mut ui,&mut state,node,Input{time:3.,..Input::default()});assert!(state.take_sliding_drawer_events("drawer").is_empty());
    state.set_sliding_drawer_locked("drawer",true);state.set_sliding_drawer_open("drawer",false,false);draw(&mut ui,&mut state,node,Input{time:4.,..Input::default()});
    assert_eq!(state.take_sliding_drawer_events("drawer"),vec![Closed]);
    for i in 0..36 {state.set_sliding_drawer_open("drawer",i%2==0,false);draw(&mut ui,&mut state,node,Input{time:5.+i as f64,..Input::default()});}
    let events=state.take_sliding_drawer_events("drawer");assert_eq!(events.len(),32);assert_eq!(events[0],Opened);assert_eq!(events[31],Closed);
}
#[test] fn interrupted_drawer_motion_does_not_report_an_unreached_state(){
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());let node=&fixture::LAYOUT;
    state.set_sliding_drawer_open("drawer",true,true);draw(&mut ui,&mut state,node,Input{time:1.,..Input::default()});
    state.set_sliding_drawer_open("drawer",false,true);draw(&mut ui,&mut state,node,Input{time:1.01,..Input::default()});
    draw(&mut ui,&mut state,node,Input{time:2.,..Input::default()});assert!(!state.sliding_drawers["drawer"].open);assert_eq!(state.take_sliding_drawer_events("drawer"),vec![bhoo::android_xml::SlidingDrawerEvent::ScrollStarted,bhoo::android_xml::SlidingDrawerEvent::ScrollEnded,bhoo::android_xml::SlidingDrawerEvent::ScrollStarted,bhoo::android_xml::SlidingDrawerEvent::ScrollEnded]);
}
#[test] fn tapping_handle_reports_completed_open_and_close(){
    use bhoo::android_xml::SlidingDrawerEvent::{Opened,Closed,ScrollStarted,ScrollEnded};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());let node=&fixture::LAYOUT;
    draw(&mut ui,&mut state,node,touch(150.,216.,true,false,1.));draw(&mut ui,&mut state,node,touch(150.,216.,false,true,1.1));
    assert_eq!(state.take_sliding_drawer_events("drawer"),vec![ScrollStarted,ScrollEnded]);draw(&mut ui,&mut state,node,Input{time:2.,..Input::default()});assert_eq!(state.take_sliding_drawer_events("drawer"),vec![Opened]);
    draw(&mut ui,&mut state,node,touch(150.,24.,true,false,2.1));draw(&mut ui,&mut state,node,touch(150.,24.,false,true,2.2));
    assert_eq!(state.take_sliding_drawer_events("drawer"),vec![ScrollStarted,ScrollEnded]);draw(&mut ui,&mut state,node,Input{time:3.,..Input::default()});assert_eq!(state.take_sliding_drawer_events("drawer"),vec![Closed]);
}
#[test] fn cancelling_or_locking_tracking_reports_one_end_event(){
    use bhoo::android_xml::SlidingDrawerEvent::{ScrollStarted,ScrollEnded};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());let node=&fixture::LAYOUT;
    draw(&mut ui,&mut state,node,touch(150.,216.,true,false,1.));assert_eq!(state.take_sliding_drawer_events("drawer"),vec![ScrollStarted]);
    draw(&mut ui,&mut state,node,Input{pointer:Some([150.,216.]),cancelled:true,time:1.1,..Input::default()});assert_eq!(state.take_sliding_drawer_events("drawer"),vec![ScrollEnded]);
    draw(&mut ui,&mut state,node,touch(150.,216.,true,false,2.));assert_eq!(state.take_sliding_drawer_events("drawer"),vec![ScrollStarted]);
    state.set_sliding_drawer_locked("drawer",true);assert_eq!(state.take_sliding_drawer_events("drawer"),vec![ScrollEnded]);
    draw(&mut ui,&mut state,node,touch(150.,216.,false,true,2.1));assert!(state.take_sliding_drawer_events("drawer").is_empty());
}
#[test] fn repeated_animated_open_reports_tracking_without_duplicate_open(){
    use bhoo::android_xml::SlidingDrawerEvent::{Opened,ScrollStarted,ScrollEnded};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());let node=&fixture::LAYOUT;
    state.set_sliding_drawer_open("drawer",true,false);draw(&mut ui,&mut state,node,Input{time:1.,..Input::default()});assert_eq!(state.take_sliding_drawer_events("drawer"),vec![Opened]);
    state.set_sliding_drawer_open("drawer",true,true);draw(&mut ui,&mut state,node,Input{time:2.,..Input::default()});assert_eq!(state.take_sliding_drawer_events("drawer"),vec![ScrollStarted,ScrollEnded]);
    draw(&mut ui,&mut state,node,Input{time:3.,..Input::default()});assert!(state.take_sliding_drawer_events("drawer").is_empty());
}
