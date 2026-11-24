mod drag {
    use bhoo::*;

    #[test]
    fn thumb_drag_tracks_pointer_clamps_and_stops_after_release() {
        let mut ui = Ui::default();
        let mut drag = ScrollbarDragState::default();
        let mut offset = 0.0;
        let bounds = Rect { x: 0.0, y: 0.0, width: 100.0, height: 200.0 };
        let track = Rect { x: 80.0, y: 0.0, width: 10.0, height: 120.0 };
        let render = |ui: &mut Ui, drag: &mut ScrollbarDragState, offset: &mut f32, input: Input| {
            ui.frame(input, bounds, 0.0, |f| {
                let thumb = Rect { x: 80.0, y: *offset / 10.0, width: 10.0, height: 20.0 };
                f.scrollbar_thumb_drag(1, thumb, track, 1000.0, offset, drag);
            });
        };
        let pointer = |y, pressed, released| Input { pointer: Some([85.0,y]), pressed, released, ..Input::default() };
        render(&mut ui, &mut drag, &mut offset, pointer(10.0, true, false));
        render(&mut ui, &mut drag, &mut offset, pointer(60.0, false, false));
        assert_eq!(offset, 500.0);
        render(&mut ui, &mut drag, &mut offset, pointer(180.0, false, false));
        assert_eq!(offset, 1000.0);
        render(&mut ui, &mut drag, &mut offset, pointer(180.0, false, true));
        render(&mut ui, &mut drag, &mut offset, pointer(20.0, false, false));
        assert_eq!(offset, 1000.0);
    }

    #[test]
    fn scrollbar_pages_and_preserves_grab_position_while_dragging() {
        let mut ui=Ui::default();let mut drag=ScrollbarDragState::default();let mut offset=0.0;
        let bounds=Rect{x:0.0,y:0.0,width:100.0,height:200.0};let track=Rect{x:80.0,y:0.0,width:20.0,height:200.0};
        let draw=|ui:&mut Ui,state:&mut ScrollbarDragState,offset:&mut f32,input:Input| {
            let mut thumb=Rect::default();ui.frame(input,bounds,0.0,|f|{(_,thumb)=f.vertical_scrollbar(1,track,"Scroll",1000.0,200.0,offset,state);});thumb
        };
        let pointer=|y,pressed,released|Input{pointer:Some([90.0,y]),pressed,released,..Input::default()};
        draw(&mut ui,&mut drag,&mut offset,pointer(180.0,true,false));assert_eq!(offset,200.0);
        draw(&mut ui,&mut drag,&mut offset,pointer(190.0,false,false));assert_eq!(offset,200.0);
        draw(&mut ui,&mut drag,&mut offset,pointer(190.0,false,true));
        let thumb=draw(&mut ui,&mut drag,&mut offset,Input::default());let grab=thumb.y+4.0;
        draw(&mut ui,&mut drag,&mut offset,pointer(grab,true,false));assert_eq!(offset,200.0);
        draw(&mut ui,&mut drag,&mut offset,pointer(grab+20.0,false,false));assert_eq!(offset,300.0);
        draw(&mut ui,&mut drag,&mut offset,pointer(500.0,false,false));assert_eq!(offset,800.0);
        draw(&mut ui,&mut drag,&mut offset,Input{cancelled:true,..Input::default()});
        draw(&mut ui,&mut drag,&mut offset,pointer(10.0,false,false));assert_eq!(offset,800.0);
    }

    #[test]
    fn scrollbar_page_keys_clamp_to_content_bounds() {
        let mut ui=Ui::default();let mut drag=ScrollbarDragState::default();let mut offset=0.0;
        let bounds=Rect{x:0.,y:0.,width:100.,height:200.};let track=Rect{x:80.,y:0.,width:20.,height:200.};
        let draw=|ui:&mut Ui,state:&mut ScrollbarDragState,offset:&mut f32,input:Input| {ui.frame(input,bounds,0.,|f|{f.vertical_scrollbar(1,track,"Scroll",1000.,200.,offset,state);});};
        draw(&mut ui,&mut drag,&mut offset,Input{pointer:Some([90.,10.]),pressed:true,..Input::default()});
        draw(&mut ui,&mut drag,&mut offset,Input{pointer:Some([90.,10.]),released:true,..Input::default()});
        let key=|key|Input{key:Some(KeyInput{key,pressed:true,repeat:false,shift:false}),..Input::default()};
        draw(&mut ui,&mut drag,&mut offset,key(Key::PageDown));assert_eq!(offset,200.);
        draw(&mut ui,&mut drag,&mut offset,key(Key::End));assert_eq!(offset,800.);
        draw(&mut ui,&mut drag,&mut offset,key(Key::PageDown));assert_eq!(offset,800.);
        draw(&mut ui,&mut drag,&mut offset,key(Key::PageUp));assert_eq!(offset,600.);
        draw(&mut ui,&mut drag,&mut offset,key(Key::Home));assert_eq!(offset,0.);
    }

    #[test]
    fn external_scrollbar_control_stops_viewport_fling() {
        let mut ui=Ui::default();let mut offset=0.0;
        let bounds=Rect{x:0.0,y:0.0,width:100.0,height:200.0};
        let draw=|ui:&mut Ui,offset:&mut f32,input:Input,cancel:bool| {
            ui.frame(input,bounds,0.0,|f|{if cancel {f.cancel_scroll(77);}f.scroll_viewport(77,bounds,offset,800.0);});
        };
        let touch=|y,pressed,released,time|Input{pointer:Some([50.0,y]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()};
        draw(&mut ui,&mut offset,touch(150.0,true,false,1.0),false);
        draw(&mut ui,&mut offset,touch(80.0,false,false,1.03),false);
        draw(&mut ui,&mut offset,touch(40.0,false,false,1.06),false);
        draw(&mut ui,&mut offset,touch(40.0,false,true,1.07),false);
        let released=offset;
        draw(&mut ui,&mut offset,Input{time:1.12,..Input::default()},false);
        assert!(offset>released,"test requires an active fling");
        let before=offset;
        draw(&mut ui,&mut offset,Input{time:1.20,..Input::default()},true);
        draw(&mut ui,&mut offset,Input{time:1.30,..Input::default()},false);
        assert!((offset-before).abs()<0.01,"external control should cancel fling");
    }
}

mod indicator {
    use bhoo::*;

    fn draw(ui:&mut Ui, state:&mut ScrollbarIndicatorState, area:Rect, content:f32, offset:f32, orientation:ScrollbarOrientation, rtl:bool, fade:bool, time:f64)->(Option<Rect>,Vec<DrawCommand>) {
        let mut result=None;
        let view=Rect{x:0.0,y:0.0,width:500.0,height:500.0};
        let commands=ui.frame(Input{time,..Input::default()},view,0.0,|f|{
            result=f.scroll_indicator(area,content,offset,orientation,rtl,fade,state);
        });
        (result,commands)
    }

    #[test]
    fn indicator_places_vertical_and_horizontal_thumbs_in_both_directions() {
        let mut ui=Ui::default();let mut state=ScrollbarIndicatorState::default();
        let vertical=Rect{x:10.,y:20.,width:100.,height:200.};
        let start=draw(&mut ui,&mut state,vertical,1000.,0.,ScrollbarOrientation::Vertical,false,false,0.).0.unwrap();
        assert_eq!(start.x,106.);assert_eq!(start.y,22.);assert!((start.height-39.2).abs()<0.01);assert_eq!(start.width,4.);
        let end=draw(&mut ui,&mut state,vertical,1000.,800.,ScrollbarOrientation::Vertical,true,false,0.1).0.unwrap();
        assert_eq!(end.x,10.);assert!((end.y+end.height-218.).abs()<0.01);
        let horizontal=Rect{x:10.,y:20.,width:200.,height:100.};
        let ltr=draw(&mut ui,&mut state,horizontal,1000.,0.,ScrollbarOrientation::Horizontal,false,false,0.2).0.unwrap();
        let rtl=draw(&mut ui,&mut state,horizontal,1000.,0.,ScrollbarOrientation::Horizontal,true,false,0.3).0.unwrap();
        assert_eq!(ltr.x,12.);assert!((rtl.x+rtl.width-208.).abs()<0.01);assert_eq!(ltr.y,116.);
        assert!(draw(&mut ui,&mut state,vertical,200.,0.,ScrollbarOrientation::Vertical,false,false,0.4).0.is_none());
    }

    #[test]
    fn indicator_shows_on_scroll_holds_then_fades_and_restarts() {
        let mut ui=Ui::default();let mut state=ScrollbarIndicatorState::default();let area=Rect{x:0.,y:0.,width:100.,height:200.};
        let frame=|ui:&mut Ui,state:&mut ScrollbarIndicatorState,offset,time|draw(ui,state,area,1000.,offset,ScrollbarOrientation::Vertical,false,true,time);
        assert!(frame(&mut ui,&mut state,0.,0.).0.is_none());
        let (_,shown)=frame(&mut ui,&mut state,100.,0.1);let color=|v:&[DrawCommand]|v.iter().find_map(|c|if let DrawCommand::RoundedRect{color,..}=c {Some(*color)}else{None}).unwrap();
        let alpha=color(&shown).3;assert!(alpha>0);assert!((ui.next_wakeup().unwrap()-0.5).abs()<0.001);
        let (_,held)=frame(&mut ui,&mut state,100.,0.49);assert_eq!(color(&held).3,alpha);
        let (_,fading)=frame(&mut ui,&mut state,100.,0.6);assert!(color(&fading).3<alpha);
        assert!(frame(&mut ui,&mut state,100.,0.76).0.is_none());
        let (_,again)=frame(&mut ui,&mut state,140.,0.8);assert_eq!(color(&again).3,alpha);
        ui.motion_preference=MotionPreference::Reduced;
        assert!(frame(&mut ui,&mut state,140.,1.21).0.is_none());
    }

    #[test]
    fn horizontal_touch_scroll_direction_follows_layout_direction() {
        for rtl in [false,true] {
            let mut ui=Ui::default();let mut offset=0.;let area=Rect{x:0.,y:0.,width:200.,height:100.};
            let draw=|ui:&mut Ui,offset:&mut f32,input:Input|{ui.frame(input,area,0.,|f|{f.scroll_viewport_horizontal_rtl(1,area,offset,1000.,rtl);});};
            let start=if rtl{60.}else{140.};let end=if rtl{140.}else{60.};
            draw(&mut ui,&mut offset,Input{pointer:Some([start,50.]),pointer_kind:PointerKind::Touch,pressed:true,time:1.0,..Input::default()});
            draw(&mut ui,&mut offset,Input{pointer:Some([end,50.]),pointer_kind:PointerKind::Touch,time:1.1,..Input::default()});
            assert!(offset>50.,"touch should advance in the expected direction; rtl={rtl}, offset={offset}");
        }
    }

    #[test]
    fn horizontal_wheel_uses_physical_direction_and_vertical_wheel_advances_logically() {
        for rtl in [false,true] {
            let mut ui=Ui::default();let mut offset=100.;let area=Rect{x:0.,y:0.,width:200.,height:100.};
            let draw=|ui:&mut Ui,offset:&mut f32,input:Input|{ui.frame(input,area,0.,|f|{f.scroll_viewport_horizontal_rtl(1,area,offset,1000.,rtl);});};
            draw(&mut ui,&mut offset,Input{pointer:Some([100.,50.]),scroll:[40.,0.],..Input::default()});
            assert_eq!(offset,if rtl{60.}else{140.});
            draw(&mut ui,&mut offset,Input{pointer:Some([100.,50.]),scroll:[0.,30.],..Input::default()});
            assert_eq!(offset,if rtl{90.}else{170.});
        }
    }
}
