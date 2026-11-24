use bhoo::{pull_refresh::PullRefreshState, Input, PointerKind, Rect, Ui};

#[test]
fn pull_threshold_release_cancel_and_top_edge() {
    let bounds=Rect{x:0.,y:0.,width:300.,height:300.};
    let mut ui=Ui::default();let mut state=PullRefreshState::default();
    let frame=|ui:&mut Ui,state:&mut PullRefreshState,y:f32,time:f64,pressed:bool,released:bool,cancelled:bool,at_top:bool| {
        let mut result=false;
        ui.frame(Input { pointer:Some([30.,y]),pointer_kind:PointerKind::Touch,time,pressed,released,cancelled,..Input::default() },bounds,0.,|f| { result=f.pull_to_refresh(bounds,state,at_top,false); });
        result
    };
    frame(&mut ui,&mut state,20.,1.,true,false,false,true);
    frame(&mut ui,&mut state,160.,1.1,false,false,false,true);
    assert!(!state.armed());
    assert!(!frame(&mut ui,&mut state,160.,1.2,false,true,false,true));
    frame(&mut ui,&mut state,20.,2.,true,false,false,true);
    frame(&mut ui,&mut state,200.,2.1,false,false,false,true);
    assert!(state.armed());
    assert!(frame(&mut ui,&mut state,200.,2.2,false,true,false,true));
    assert!(!frame(&mut ui,&mut state,200.,2.3,false,true,false,true));
    frame(&mut ui,&mut state,20.,3.,true,false,false,true);
    frame(&mut ui,&mut state,200.,3.1,false,false,true,true);
    assert!(!frame(&mut ui,&mut state,200.,3.2,false,true,false,true));
    frame(&mut ui,&mut state,20.,4.,true,false,false,false);
    frame(&mut ui,&mut state,200.,4.1,false,false,false,false);
    assert!(!frame(&mut ui,&mut state,200.,4.2,false,true,false,false));
}

#[test]
fn scroll_hands_remaining_drag_to_refresh_and_reverses() {
    let bounds=Rect{x:0.,y:0.,width:300.,height:200.};
    let mut ui=Ui::default();let mut state=PullRefreshState::default();let mut offset=100.;
    let frame=|ui:&mut Ui,state:&mut PullRefreshState,offset:&mut f32,y:f32,time:f64,pressed:bool,released:bool,cancelled:bool| {
        let mut result=false;
        ui.frame(Input { pointer:Some([30.,y]),pointer_kind:PointerKind::Touch,time,pressed,released,cancelled,..Input::default() },bounds,0.,|f| {
            result=f.pull_refresh_scroll(1,bounds,offset,600.,state,false,|_,_|{});
        });result
    };
    frame(&mut ui,&mut state,&mut offset,20.,1.,true,false,false);
    frame(&mut ui,&mut state,&mut offset,170.,1.1,false,false,false);
    assert_eq!(offset,0.); assert!((state.offset()-25.).abs()<0.01);
    frame(&mut ui,&mut state,&mut offset,130.,1.2,false,false,false);
    assert_eq!(offset,0.); assert!((state.offset()-5.).abs()<0.01);
    frame(&mut ui,&mut state,&mut offset,100.,1.3,false,false,false);
    assert_eq!(offset,20.); assert!(!state.armed());
    frame(&mut ui,&mut state,&mut offset,300.,1.4,false,false,false);
    assert_eq!(offset,0.); assert!(state.armed());
    assert!(frame(&mut ui,&mut state,&mut offset,300.,1.5,false,true,false));
    assert!(!frame(&mut ui,&mut state,&mut offset,300.,1.6,false,true,false));
}

#[test]
fn pull_indicator_reveals_small_arc_and_arrow_after_initial_pull() {
    use bhoo::DrawCommand;
    let bounds=Rect{x:0.,y:0.,width:300.,height:200.};let mut ui=Ui::default();let mut state=PullRefreshState::default();let mut offset=0.;
    let frame=|ui:&mut Ui,state:&mut PullRefreshState,offset:&mut f32,y:f32,time:f64,pressed:bool| {
        ui.frame(Input { pointer:Some([30.,y]),pointer_kind:PointerKind::Touch,time,pressed,..Input::default() },bounds,0.,|f| {
            f.pull_refresh_scroll(8,bounds,offset,400.,state,false,|_,_|{});
        })
    };
    frame(&mut ui,&mut state,&mut offset,10.,1.,true);
    let early=frame(&mut ui,&mut state,&mut offset,50.,1.1,false);
    assert!(!early.iter().any(|c|matches!(c,DrawCommand::Arc { .. } | DrawCommand::Line { .. })));
    let later=frame(&mut ui,&mut state,&mut offset,130.,1.2,false);
    assert!(later.iter().any(|c|matches!(c,DrawCommand::Arc { bounds, stroke,.. } if (bounds.width-13.5).abs()<0.01 && (*stroke-2.5).abs()<0.01)));
    assert_eq!(later.iter().filter(|c|matches!(c,DrawCommand::Line { .. })).count(),2);
}
