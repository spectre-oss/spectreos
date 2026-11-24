use bhoo::*;
use bhoo::android_xml::LayoutState;
mod fixture {include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_tab_widget.rs"));}
const B:Rect=Rect{x:0.,y:0.,width:300.,height:56.};
fn draw(ui:&mut Ui,s:&mut LayoutState,input:Input){ui.frame(input,B,0.,|f|{fixture::LAYOUT.render(f,B,s);});}
fn touch(x:f32,pressed:bool,released:bool,time:f64)->Input{Input{pointer:Some([x,28.]),pointer_kind:PointerKind::Touch,pressed,released,time,..Input::default()}}
#[test] fn tap_selects_second_tab(){let(mut ui,mut s)=(Ui::default(),LayoutState::default());draw(&mut ui,&mut s,touch(150.,true,false,1.));draw(&mut ui,&mut s,touch(150.,false,true,1.1));assert_eq!(s.displayed_children["tabs"],1);assert_eq!(s.tab_widget_clicked["tabs"],1);}
#[test] fn rtl_reverses_visual_order(){let(mut ui,mut s)=(Ui::default(),LayoutState::default());s.rtl=true;draw(&mut ui,&mut s,Input::default());let first=ui.semantics().iter().find(|v|v.label.as_ref()=="First").unwrap().bounds;let third=ui.semantics().iter().find(|v|v.label.as_ref()=="Third").unwrap().bounds;assert!(first.x>third.x);draw(&mut ui,&mut s,touch(50.,true,false,1.));draw(&mut ui,&mut s,touch(50.,false,true,1.1));assert_eq!(s.displayed_children["tabs"],2);}
#[test] fn disabled_tab_is_not_selected(){static CHILDREN:[android_xml::Node;3]=[fixture::LAYOUT.children[0],android_xml::Node{enabled:false,..fixture::LAYOUT.children[1]},fixture::LAYOUT.children[2]];static W:android_xml::Node=android_xml::Node{children:&CHILDREN,..fixture::LAYOUT};let(mut ui,mut s)=(Ui::default(),LayoutState::default());ui.frame(touch(150.,true,false,1.),B,0.,|f|{W.render(f,B,&mut s);});ui.frame(touch(150.,false,true,1.1),B,0.,|f|{W.render(f,B,&mut s);});assert!(!s.tab_widget_clicked.contains_key("tabs"));}
#[test] fn custom_strips_surround_selected_tab_and_follow_visual_position(){
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());
    state.displayed_children.insert("tabs".into(),1);
    let render=|ui:&mut Ui,state:&mut LayoutState|ui.frame(Input::default(),B,0.,|f|{fixture::LAYOUT.render(f,B,state);});
    let c=render(&mut ui,&mut state);
    assert!(c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(229,87,84,255),..} if bounds.x==0. && bounds.width==100. && bounds.y==50. && bounds.height==6.)));
    assert!(c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(78,123,229,255),..} if bounds.x==200. && bounds.width==100. && bounds.y==50. && bounds.height==6.)));
    state.displayed_children.insert("tabs".into(),0);state.rtl=true;
    let c=render(&mut ui,&mut state);
    assert!(c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(229,87,84,255),..} if bounds.x==0. && bounds.width==200.)));
    assert!(c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(78,123,229,255),..} if bounds.x==300. && bounds.width==16.)));
}
#[test] fn custom_null_and_disabled_strips_draw_nothing(){
    use bhoo::android_xml::{Node,RelativeRule};
    static NULL:Node=Node{relative_rules:&[RelativeRule{kind:191,anchor:"null"},RelativeRule{kind:192,anchor:"16,6,78,123,229,255"}],..fixture::LAYOUT};
    static HIDDEN:Node=Node{relative_rules:&[RelativeRule{kind:147,anchor:"false"},RelativeRule{kind:191,anchor:"16,6,229,87,84,255"},RelativeRule{kind:192,anchor:"16,6,78,123,229,255"}],..fixture::LAYOUT};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());state.displayed_children.insert("tabs".into(),1);
    let c=ui.frame(Input::default(),B,0.,|f|{NULL.render(f,B,&mut state);});
    assert!(!c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(229,87,84,255),..} if bounds.height==6.)));
    assert!(c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(78,123,229,255),..} if bounds.height==6.)));
    let c=ui.frame(Input::default(),B,0.,|f|{HIDDEN.render(f,B,&mut state);});
    assert!(!c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(229,87,84,255)|Color(78,123,229,255),..} if bounds.height==6.)));
}
#[test] fn custom_strips_align_to_widget_edges_even_with_padding(){
    static PADDED:android_xml::Node=android_xml::Node{padding_edges:[10.,8.,10.,8.],padding_explicit:[true;4],..fixture::LAYOUT};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());state.displayed_children.insert("tabs".into(),1);
    let c=ui.frame(Input::default(),B,0.,|f|{PADDED.render(f,B,&mut state);});
    assert!(c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(229,87,84,255),..} if bounds.x==0. && bounds.y==50. && bounds.height==6.)));
    assert!(c.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds,color:Color(78,123,229,255),..} if (bounds.x+bounds.width-300.).abs()<0.01 && bounds.y==50.)));
}
#[test] fn invisible_tab_reserves_space_without_drawing_or_accepting_taps(){
    use bhoo::android_xml::Visibility;
    for rtl in [false,true] {
        let(mut ui,mut state)=(Ui::default(),LayoutState::default());state.rtl=rtl;
        state.visibility.insert("second".into(),Visibility::Invisible);
        draw(&mut ui,&mut state,Input::default());
        assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="Second"));
        let first=ui.semantics().iter().find(|n|n.label.as_ref()=="First").unwrap().bounds;
        assert_eq!(first.width,100.0);
        draw(&mut ui,&mut state,touch(150.,true,false,1.));draw(&mut ui,&mut state,touch(150.,false,true,1.1));
        assert!(!state.tab_widget_clicked.contains_key("tabs"));
        state.visibility.insert("second".into(),Visibility::Visible);
        draw(&mut ui,&mut state,touch(150.,true,false,2.));draw(&mut ui,&mut state,touch(150.,false,true,2.1));
        assert_eq!(state.tab_widget_clicked.get("tabs"),Some(&1));
    }
}
#[test] fn hiding_selected_tab_preserves_selection_without_target(){
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());
    state.displayed_children.insert("tabs".into(),1);
    state.visibility.insert("second".into(),bhoo::android_xml::Visibility::Invisible);
    draw(&mut ui,&mut state,Input::default());
    assert_eq!(state.displayed_children["tabs"],1);
    assert!(!ui.semantics().iter().any(|n|n.label.as_ref()=="Second"));
    draw(&mut ui,&mut state,touch(150.,true,false,1.));draw(&mut ui,&mut state,touch(150.,false,true,1.1));
    assert_eq!(state.displayed_children["tabs"],1);
    assert!(!state.tab_widget_clicked.contains_key("tabs"));
}
#[test] fn fixed_tab_widths_remain_distinct_and_shrink_on_overflow(){
    use bhoo::android_xml::{Node,Size};
    static CHILDREN:[Node;3]=[Node{width:Size::Dp(48.),..fixture::LAYOUT.children[0]},Node{width:Size::Dp(96.),..fixture::LAYOUT.children[1]},Node{width:Size::Dp(96.),..fixture::LAYOUT.children[2]}];
    static FIXED:Node=Node{children:&CHILDREN,..fixture::LAYOUT};
    for rtl in [false,true] {
        let(mut ui,mut state)=(Ui::default(),LayoutState::default());state.rtl=rtl;
        ui.frame(Input::default(),B,0.,|f|{FIXED.render(f,B,&mut state);});
        let bounds=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
        let first=bounds(&ui,"First");let second=bounds(&ui,"Second");let third=bounds(&ui,"Third");
        assert_eq!([first.width,second.width,third.width],[48.,96.,96.]);
        assert_eq!(first.x,if rtl {252.}else{0.});assert_eq!(third.x,if rtl {60.}else{144.});
        let narrow=Rect{width:120.,..B};
        ui.frame(Input::default(),narrow,0.,|f|{FIXED.render(f,narrow,&mut state);});
        assert_eq!([bounds(&ui,"First").width,bounds(&ui,"Second").width,bounds(&ui,"Third").width],[8.,56.,56.]);
    }
}
#[test] fn fixed_tabs_respect_margins_and_reduce_widths_on_overflow(){
    use bhoo::android_xml::{Node,Size};
    static CHILDREN:[Node;3]=[
        Node{width:Size::Dp(48.),margin_relative:[Some(10.),Some(4.)],..fixture::LAYOUT.children[0]},
        Node{width:Size::Dp(96.),margin:[5.,2.,7.,3.],..fixture::LAYOUT.children[1]},
        Node{width:Size::Dp(96.),..fixture::LAYOUT.children[2]},
    ];
    static FIXED:Node=Node{children:&CHILDREN,..fixture::LAYOUT};
    for rtl in [false,true] {
        let(mut ui,mut state)=(Ui::default(),LayoutState::default());state.rtl=rtl;
        ui.frame(Input::default(),B,0.,|f|{FIXED.render(f,B,&mut state);});
        let bounds=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
        let first=bounds(&ui,"First");let second=bounds(&ui,"Second");
        assert_eq!(first.x,if rtl {242.}else{10.});assert_eq!(second.x,if rtl {135.}else{67.});
        assert_eq!(second.y,2.);assert_eq!(second.height,51.);
        let narrow=Rect{width:146.,..B};
        ui.frame(Input::default(),narrow,0.,|f|{FIXED.render(f,narrow,&mut state);});
        assert_eq!([bounds(&ui,"First").width,bounds(&ui,"Second").width,bounds(&ui,"Third").width],[8.,56.,56.]);
    }
}
#[test] fn weighted_fixed_tabs_distribute_remaining_space_and_follow_rtl(){
    use bhoo::android_xml::{Node,Size};
    static CHILDREN:[Node;3]=[
        Node{width:Size::Dp(0.),weight:1.,margin_relative:[Some(5.),Some(5.)],..fixture::LAYOUT.children[0]},
        Node{width:Size::Dp(0.),weight:2.,..fixture::LAYOUT.children[1]},
        Node{width:Size::Dp(0.),weight:1.,..fixture::LAYOUT.children[2]},
    ];
    static WEIGHTED:Node=Node{children:&CHILDREN,..fixture::LAYOUT};
    static PARTIAL:Node=Node{weight_sum:5.,..WEIGHTED};
    for rtl in [false,true] {
        let(mut ui,mut state)=(Ui::default(),LayoutState::default());state.rtl=rtl;
        let render=|ui:&mut Ui,state:&mut LayoutState,node:&'static Node|ui.frame(Input::default(),B,0.,|f|{node.render(f,B,state);});
        render(&mut ui,&mut state,&WEIGHTED);
        let bounds=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
        let a=bounds(&ui,"First");let b=bounds(&ui,"Second");let c=bounds(&ui,"Third");
        assert_eq!([a.width,b.width,c.width],[72.5,145.,72.5]);assert_eq!(a.x,if rtl {222.5}else{5.});
        render(&mut ui,&mut state,&PARTIAL);
        let a=bounds(&ui,"First");let b=bounds(&ui,"Second");let c=bounds(&ui,"Third");
        assert_eq!([a.width,b.width,c.width],[58.,116.,58.]);
    }
}
#[test] fn weights_add_space_to_nonzero_fixed_widths(){
    use bhoo::android_xml::{Node,Size};
    static CHILDREN:[Node;3]=[Node{width:Size::Dp(40.),weight:1.,..fixture::LAYOUT.children[0]},Node{width:Size::Dp(60.),weight:2.,..fixture::LAYOUT.children[1]},Node{width:Size::Dp(60.),..fixture::LAYOUT.children[2]}];
    static WEIGHTED:Node=Node{children:&CHILDREN,..fixture::LAYOUT};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());
    ui.frame(Input::default(),B,0.,|f|{WEIGHTED.render(f,B,&mut state);});
    let width=|label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds.width;
    assert!((width("First")-86.6667).abs()<0.01);
    assert!((width("Second")-153.3333).abs()<0.01);
    assert_eq!(width("Third"),60.);
}
#[test] fn wrap_content_tabs_measure_text_and_remeasure_after_changes(){
    use bhoo::android_xml::{Node,Size};
    static CHILDREN:[Node;3]=[Node{width:Size::Wrap,..fixture::LAYOUT.children[0]},Node{width:Size::Wrap,..fixture::LAYOUT.children[1]},Node{width:Size::Dp(50.),..fixture::LAYOUT.children[2]}];
    static WRAPPED:Node=Node{children:&CHILDREN,..fixture::LAYOUT};
    for rtl in [false,true] {
        let(mut ui,mut state)=(Ui::default(),LayoutState::default());state.rtl=rtl;
        let render=|ui:&mut Ui,state:&mut LayoutState,bounds:Rect|ui.frame(Input::default(),bounds,0.,|f|{WRAPPED.render(f,bounds,state);});
        render(&mut ui,&mut state,B);
        let bounds=|ui:&Ui,label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds;
        let first=bounds(&ui,"First");let second=bounds(&ui,"Second");let third=bounds(&ui,"Third");
        assert!(first.width>0. && second.width>first.width);assert_eq!(third.width,50.);
        if rtl{assert!(first.x>second.x)}else{assert!(first.x<second.x)}
        state.text.insert("second".into(),"A considerably longer title".into());
        render(&mut ui,&mut state,B);let longer=bounds(&ui,"A considerably longer title");assert!(longer.width>second.width);
        let narrow=Rect{width:120.,..B};render(&mut ui,&mut state,narrow);
        let a=bounds(&ui,"First");let b=bounds(&ui,"A considerably longer title");let c=bounds(&ui,"Third");
        assert!((a.width+b.width+c.width-120.).abs()<0.01);assert!(b.width<longer.width);
    }
}
#[test] fn wrap_content_tabs_use_renderer_glyph_widths_and_longest_line(){
    use bhoo::android_xml::{Node,Size};
    static CHILDREN:[Node;3]=[
        Node{width:Size::Wrap,text:"iii",..fixture::LAYOUT.children[0]},
        Node{width:Size::Wrap,text:"WWW",..fixture::LAYOUT.children[1]},
        Node{width:Size::Wrap,text:"W\nii",..fixture::LAYOUT.children[2]},
    ];
    static WRAPPED:Node=Node{children:&CHILDREN,..fixture::LAYOUT};
    let(mut ui,mut state)=(Ui::default(),LayoutState::default());
    ui.text_metrics.normal['i' as usize-32]=0.25;
    ui.text_metrics.normal['W' as usize-32]=1.0;
    ui.frame(Input::default(),B,0.,|f|{WRAPPED.render(f,B,&mut state);});
    let width=|label:&str|ui.semantics().iter().find(|n|n.label.as_ref()==label).unwrap().bounds.width;
    assert_eq!(width("iii"),12.);assert_eq!(width("WWW"),48.);assert_eq!(width("W\nii"),16.);
}
