use bhoo::*;
#[test]
fn material_fab_icons_are_centered_and_extended_hugs_content() {
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 300.0,
    };
    let mut ui = Ui::default();
    let mut responses = Vec::new();
    let commands = ui.frame(Input::default(), view, 0.0, |f| {
        for (id, size) in [(10, 40.0), (11, 56.0), (12, 80.0), (14, 96.0)] {
            responses.push(f.fab_with_icon(id, "Add", AppBarIcon::Add, size, false, true));
        }
        responses.push(f.fab_with_icon(13, "Create item", AppBarIcon::Add, 56.0, true, true));
    });
    let shapes: Vec<_> = commands
        .iter()
        .filter_map(|c| {
            if let DrawCommand::Shape { bounds, .. } = c {
                Some(*bounds)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(shapes.len(), 5);
    for i in 0..4 {
        let b = responses[i].bounds;
        let icon = shapes[i];
        assert_eq!(icon.width, [24.0,24.0,28.0,36.0][i]);
        assert!((icon.x + icon.width / 2.0 - b.x - b.width / 2.0).abs() < 0.01);
        assert!((icon.y + icon.height / 2.0 - b.y - b.height / 2.0).abs() < 0.01);
    }
    assert!(responses[4].bounds.width < view.width);
    assert_eq!(shapes[4].x, responses[4].bounds.x + 16.0);
    assert!(
        ui.semantics()
            .iter()
            .any(|n| n.id == 13 && n.label.as_ref() == "Create item")
    );
}

#[test]
fn extended_fab_mirrors_icon_and_label_in_rtl() {
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 220.0,
    };
    for size in [56.0, 80.0, 96.0] {
        let mut ui = Ui::default();
        let mut response = Response::default();
        let commands = ui.frame(Input::default(), view, 0.0, |f| {
            response = f.fab_with_icon_direction(
                10,
                "Create item",
                AppBarIcon::Add,
                size,
                true,
                true,
                true,
            );
        });
        let icon = commands
            .iter()
            .find_map(|c| {
                if let DrawCommand::Shape { bounds, .. } = c {
                    Some(*bounds)
                } else {
                    None
                }
            })
            .unwrap();
        let (label, style) = commands
            .iter()
            .find_map(|c| {
                if let DrawCommand::Text {
                    bounds,
                    text,
                    style,
                    ..
                } = c
                {
                    (text.as_ref() == "Create item").then_some((*bounds, *style))
                } else {
                    None
                }
            })
            .unwrap();
        let leading=if size>=96.0{28.0}else if size>=80.0{26.0}else{16.0};
        let gap=if size>=96.0{16.0}else{12.0};
        assert!((response.bounds.x + response.bounds.width - icon.x - icon.width - leading).abs() < 0.01);
        assert!((icon.x - label.x - label.width - gap).abs() < 0.01);
        assert_eq!(style.size,if size>=96.0{24.0}else if size>=80.0{22.0}else{14.0});
        assert_eq!(style.align, 2);
        assert!(label.width > 0.0);
    }
}

#[test]
fn fab_hover_elevation_animates_and_reduced_motion_snaps() {
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    fn sample(ui: &mut Ui, view: Rect, input: Input, enabled: bool) -> Option<f32> {
        ui.frame(input, view, 0.0, |f| {
            f.fab_with_icon(10, "Add", AppBarIcon::Add, 56.0, false, enabled);
        })
        .into_iter()
        .find_map(|c| {
            if let DrawCommand::Shadow { offset, .. } = c {
                Some(offset[1])
            } else {
                None
            }
        })
    }
    let mut ui = Ui::default();
    assert_eq!(sample(&mut ui, view, Input::default(), true), Some(3.0));
    sample(
        &mut ui,
        view,
        Input {
            pointer: Some([28.0, 28.0]),
            time: 0.01,
            ..Input::default()
        },
        true,
    );
    let mid = sample(
        &mut ui,
        view,
        Input {
            pointer: Some([28.0, 28.0]),
            time: 0.08,
            ..Input::default()
        },
        true,
    )
    .unwrap();
    assert!(mid > 3.0 && mid < 5.0);
    ui.motion_preference = MotionPreference::Reduced;
    assert_eq!(
        sample(
            &mut ui,
            view,
            Input {
                pointer: Some([28.0, 28.0]),
                time: 0.09,
                ..Input::default()
            },
            true
        ),
        Some(5.0)
    );
    assert_eq!(
        sample(
            &mut ui,
            view,
            Input {
                pointer: Some([28.0, 28.0]),
                pressed: true,
                time: 0.1,
                ..Input::default()
            },
            true
        ),
        Some(3.0)
    );
    assert_eq!(
        sample(
            &mut ui,
            view,
            Input {
                pointer: Some([28.0, 28.0]),
                time: 0.2,
                ..Input::default()
            },
            false
        ),
        None
    );
}

#[test]
fn lowered_and_bottom_bar_elevation_follow_reference_states() {
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    fn shadow(
        ui: &mut Ui,
        view: Rect,
        elevation: FabElevation,
        input: Input,
    ) -> Option<(f32, f32)> {
        ui.frame(input, view, 0.0, |f| {
            f.fab_with_options(
                10,
                "Add",
                AppBarIcon::Add,
                FabOptions {
                    elevation,
                    ..FabOptions::default()
                },
            );
        })
        .into_iter()
        .find_map(|c| {
            if let DrawCommand::Shadow { offset, blur, .. } = c {
                Some((offset[1], blur))
            } else {
                None
            }
        })
    }
    assert_eq!(
        shadow(&mut ui, view, FabElevation::Lowered, Input::default()),
        Some((2.0, 1.0))
    );
    assert_eq!(
        shadow(
            &mut ui,
            view,
            FabElevation::Lowered,
            Input {
                pointer: Some([28.0, 28.0]),
                time: 0.1,
                ..Input::default()
            }
        ),
        Some((3.0, 3.0))
    );
    assert_eq!(
        shadow(
            &mut ui,
            view,
            FabElevation::Lowered,
            Input {
                pointer: Some([28.0, 28.0]),
                pressed: true,
                time: 0.2,
                ..Input::default()
            }
        ),
        Some((2.0, 1.0))
    );
    assert_eq!(
        shadow(
            &mut ui,
            view,
            FabElevation::BottomAppBar,
            Input {
                pointer: Some([28.0, 28.0]),
                time: 0.3,
                ..Input::default()
            }
        ),
        None
    );
}

#[test]
fn fab_color_variants_use_matching_container_and_icon_roles() {
    let view = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    let mut ui = Ui::default();
    for (color, container, icon) in [
        (
            FabColor::Primary,
            ui.theme.primary_container,
            ui.theme.on_primary_container,
        ),
        (
            FabColor::Secondary,
            ui.theme.secondary_container,
            ui.theme.on_secondary_container,
        ),
        (
            FabColor::Tertiary,
            ui.theme.tertiary_container,
            ui.theme.on_tertiary_container,
        ),
        (FabColor::Surface, ui.theme.surface_high, ui.theme.primary),
    ] {
        let commands = ui.frame(Input::default(), view, 0.0, |f| {
            f.fab_with_options(
                10,
                "Add",
                AppBarIcon::Add,
                FabOptions {
                    color,
                    ..FabOptions::default()
                },
            );
        });
        assert!(
            commands.iter().any(
                |c| matches!(c, DrawCommand::RoundedRect { color, .. } if *color == container)
            )
        );
        assert!(
            commands
                .iter()
                .any(|c| matches!(c, DrawCommand::Shape { color, .. } if *color == icon))
        );
    }
}

#[test]
fn secondary_fab_uses_container_roles_when_secondary_roles_differ() {
    let mut ui=Ui::default();ui.theme.secondary=Color(1,2,3,255);ui.theme.on_secondary=Color(4,5,6,255);ui.theme.secondary_container=Color(10,20,30,255);ui.theme.on_secondary_container=Color(40,50,60,255);
    let bounds=Rect{x:0.,y:0.,width:200.,height:100.};let commands=ui.frame(Input::default(),bounds,0.,|f|{f.fab_with_options(1,"Add",AppBarIcon::Add,FabOptions{color:FabColor::Secondary,..FabOptions::default()});});
    assert!(commands.iter().any(|cmd|matches!(cmd,DrawCommand::RoundedRect{color,..} if *color==Color(10,20,30,255))));
    assert!(commands.iter().any(|cmd|matches!(cmd,DrawCommand::Shape{color,..} if *color==Color(40,50,60,255))));
}
#[test]
fn extended_fab_animates_and_reverses_from_presented_width() {
    let mut ui=Ui::default();let view=Rect{x:0.,y:0.,width:360.,height:100.};
    fn sample(ui:&mut Ui,view:Rect,time:f64,extended:bool)->f32 {let mut width=0.;ui.frame(Input{time,..Input::default()},view,0.,|f|{width=f.fab_with_options(10,"Create item",AppBarIcon::Add,FabOptions{extended,..FabOptions::default()}).bounds.width;});width}
    assert_eq!(sample(&mut ui,view,0.,false),56.);assert_eq!(sample(&mut ui,view,1.,true),56.);
    let middle=sample(&mut ui,view,1.1,true);assert!(middle>56.);let reverse=sample(&mut ui,view,1.1,false);assert!((reverse-middle).abs()<0.01);let later=sample(&mut ui,view,1.25,false);assert!(later<middle);
    ui.motion_preference=MotionPreference::Reduced;let expanded=sample(&mut ui,view,1.3,true);assert!(expanded>56.);assert_eq!(sample(&mut ui,view,1.31,false),56.);
}
#[test]
fn narrow_fab_keeps_icon_inside_container_in_both_directions() {
    for width in [24.0,32.0,45.0,56.0] {for rtl in [false,true] {for extended in [false,true] {
        let mut ui=Ui::default();let view=Rect{x:0.,y:0.,width,height:80.};let mut button=Rect::default();
        let commands=ui.frame(Input::default(),view,0.,|f|{button=f.fab_with_options(9,"Create",AppBarIcon::Add,FabOptions{extended,rtl,..FabOptions::default()}).bounds;});
        let icon=commands.iter().find_map(|cmd|if let DrawCommand::Shape{bounds,..}=cmd{Some(*bounds)}else{None}).unwrap();
        assert!(icon.x>=button.x-0.01 && icon.x+icon.width<=button.x+button.width+0.01,"width={width} rtl={rtl} extended={extended}");
        assert!(icon.y>=button.y-0.01 && icon.y+icon.height<=button.y+button.height+0.01);
    }}}
}
#[test]
fn fab_corner_tokens_and_custom_radius_are_applied_to_container_and_shadow(){
    let view=Rect{x:0.,y:0.,width:200.,height:120.};
    for (size,override_radius,expected) in [(40.,None,12.),(56.,None,16.),(80.,None,20.),(96.,None,28.),(80.,Some(40.),40.),(80.,Some(f32::NAN),20.)] {
        let mut ui=Ui::default();let color=ui.theme.primary_container;let commands=ui.frame(Input::default(),view,0.,|f|{f.fab_with_options(11,"Add",AppBarIcon::Add,FabOptions{size,corner_radius:override_radius,..FabOptions::default()});});
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{radius,color:c,..} if *c==color&&(*radius-expected).abs()<0.01)));
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shadow{radius,..} if (*radius-expected).abs()<0.01)));
    }
}
#[test]
fn larger_extended_fabs_use_material_padding_and_type(){for (size,leading,gap,trailing,text_size) in [(80.,26.,12.,26.,22.),(96.,28.,16.,28.,24.)]{let mut ui=Ui::default();let view=Rect{x:0.,y:0.,width:400.,height:120.};let mut b=Rect::default();let commands=ui.frame(Input::default(),view,0.,|f|{b=f.fab_with_options(2,"Create",AppBarIcon::Add,FabOptions{size,extended:true,..FabOptions::default()}).bounds;});let icon=commands.iter().find_map(|c|if let DrawCommand::Shape{bounds,..}=c{Some(*bounds)}else{None}).unwrap();let (text,style)=commands.iter().find_map(|c|if let DrawCommand::Text{bounds,style,text,..}=c{(text.as_ref()=="Create").then_some((*bounds,*style))}else{None}).unwrap();assert_eq!(icon.x-b.x,leading);assert_eq!(text.x-(icon.x+icon.width),gap);assert!((b.x+b.width-text.x-text.width-trailing).abs()<0.01);assert_eq!(style.size,text_size);}}

#[test]
fn bounded_fab_layout_preserves_parent_clip_for_shadow() {
    let view = Rect { x: 0.0, y: 0.0, width: 240.0, height: 160.0 };
    let outer = Rect { x: 4.0, y: 4.0, width: 220.0, height: 140.0 };
    let layout = Rect { x: 24.0, y: 24.0, width: 100.0, height: 56.0 };
    let mut ui = Ui::default();
    let commands = ui.frame(Input::default(), view, 0.0, |f| {
        f.within(outer, |f| f.within_unclipped(layout, |f| {
            f.fab_with_icon(100, "Create item", AppBarIcon::Add, 56.0, true, true);
        }));
    });
    let shadow = commands.iter().position(|c| matches!(c, DrawCommand::Shadow { .. })).expect("FAB shadow");
    let clips: Vec<_> = commands[..shadow].iter().filter_map(|c| match c { DrawCommand::PushClip(rect) => Some(*rect), _ => None }).collect();
    assert!(clips.iter().any(|r| r.x == outer.x && r.width == outer.width), "parent clip must remain in force");
    assert!(!clips.iter().any(|r| r.x == layout.x && r.y == layout.y && r.width == layout.width && r.height == layout.height), "layout bounds must not clip the shadow");
}

#[test]
fn fab_at_animated_morphs_width_between_collapsed_and_extended() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 100.0 };
    let anchor = Rect { x: 80.0, y: 20.0, width: 220.0, height: 56.0 };
    let mut collapsed_w = 0.0;
    let mut extended_w = 0.0;
    let _ = ui.frame(Input { time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
        collapsed_w = f.fab_at_animated(40, anchor, MaterialIcon::Edit, "Compose", FabOptions { extended: false, ..FabOptions::default() }).bounds.width;
    });
    let _ = ui.frame(Input { time: 0.5, ..Input::default() }, bounds, 0.0, |f| {
        extended_w = f.fab_at_animated(40, anchor, MaterialIcon::Edit, "Compose", FabOptions { extended: true, ..FabOptions::default() }).bounds.width;
    });
    assert!((collapsed_w - 56.0).abs() < 0.01);
    assert!(extended_w > collapsed_w + 30.0);
}
