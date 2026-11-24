use bhoo::*;
mod xml_card { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_material_card.rs")); }
mod xml_compat_card { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_material_card_compat.rs")); }
mod xml_selector_card { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_material_card_selector.rs")); }
mod xml_theme_card { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_material_card_theme.rs")); }
const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 300.0,
    height: 160.0,
};
const B: Rect = Rect {
    x: 10.0,
    y: 10.0,
    width: 200.0,
    height: 96.0,
};
#[test]
fn passive_cards_use_distinct_surface_tokens_without_focus_targets() {
    for variant in [
        CardVariant::Filled,
        CardVariant::Outlined,
        CardVariant::Elevated,
    ] {
        let mut ui = Ui::default();
        let theme = ui.theme;
        let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
            assert!(
                f.card(
                    1,
                    B,
                    "Card",
                    CardOptions {
                        variant,
                        ..CardOptions::default()
                    },
                    |_| {}
                )
                .is_none()
            );
        });
        let rects: Vec<_> = commands
            .iter()
            .filter_map(|c| {
                if let DrawCommand::RoundedRect { color, .. } = c {
                    Some(*color)
                } else {
                    None
                }
            })
            .collect();
        match variant {
            CardVariant::Filled => assert_eq!(rects, [theme.surface_highest]),
            CardVariant::Outlined => assert_eq!(rects, [theme.outline_variant, theme.background]),
            CardVariant::Elevated => {
                assert_eq!(rects, [theme.surface_low]);
                assert!(
                    commands
                        .iter()
                        .any(|c| matches!(c, DrawCommand::Shadow { .. }))
                );
            }
        }
        assert!(ui.semantics().is_empty());
        assert!(!ui.needs_redraw());
    }
}
fn draw(ui: &mut Ui, input: Input, enabled: bool) -> (bool, bool) {
    let mut parent = false;
    let mut child = false;
    ui.frame(input, VIEW, 0.0, |f| {
        parent = f
            .card(
                1,
                B,
                "Card",
                CardOptions {
                    interactive: true,
                    enabled,
                    ..CardOptions::default()
                },
                |f| child = f.button(2, "Child").clicked,
            )
            .unwrap()
            .clicked;
    });
    (parent, child)
}
fn click(ui: &mut Ui, p: [f32; 2], time: f64, enabled: bool) -> (bool, bool) {
    draw(
        ui,
        Input {
            pointer: Some(p),
            pressed: true,
            time,
            ..Input::default()
        },
        enabled,
    );
    draw(
        ui,
        Input {
            pointer: Some(p),
            released: true,
            time: time + 0.01,
            ..Input::default()
        },
        enabled,
    )
}
#[test]
fn nested_action_does_not_activate_card_and_disabled_blocks_both() {
    let mut ui = Ui::default();
    draw(&mut ui, Input::default(), true);
    assert_eq!(click(&mut ui, [50.0, 40.0], 0.1, true), (false, true));
    assert_eq!(click(&mut ui, [190.0, 95.0], 0.3, true), (true, false));
    assert_eq!(click(&mut ui, [50.0, 40.0], 0.5, false), (false, false));
    assert!(ui.semantics().iter().all(|n| !n.enabled));
}
#[test]
fn passive_hover_stays_static_and_disabled_elevated_keeps_base_shadow() {
    let mut ui = Ui::default();
    let commands = ui.frame(
        Input {
            pointer: Some([30.0, 30.0]),
            time: 0.1,
            ..Input::default()
        },
        VIEW,
        0.0,
        |f| {
            f.card(1, B, "Card", CardOptions::default(), |_| {});
        },
    );
    assert!(
        !commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Shadow { .. } | DrawCommand::Ripple { .. }))
    );
    assert!(!ui.needs_redraw());
    let commands = ui.frame(
        Input {
            time: 0.2,
            ..Input::default()
        },
        VIEW,
        0.0,
        |f| {
            f.card(
                1,
                B,
                "Card",
                CardOptions {
                    variant: CardVariant::Elevated,
                    interactive: true,
                    enabled: false,
                },
                |_| {},
            );
        },
    );
    assert!(
        commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Shadow { .. }))
    );
    assert!(!ui.semantics()[0].enabled);
}
#[test]
fn dragged_card_paints_foreground_and_raised_shadow_then_settles() {
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;
    let draw=|ui:&mut Ui,dragged,enabled,time| ui.frame(Input {time,..Input::default()},VIEW,0.0,|f| {
        f.card_with_dragged(1,B,"Card",CardOptions {enabled,..CardOptions::default()},dragged,|f|f.label("Content"));
    });
    let resting=draw(&mut ui,false,true,0.0);
    assert!(!resting.iter().any(|c|matches!(c,DrawCommand::Shadow {..})));
    let dragged=draw(&mut ui,true,true,0.1);
    assert_eq!(dragged.iter().filter(|c|matches!(c,DrawCommand::Shadow {..})).count(),3);
    assert!(dragged.iter().any(|c|matches!(c,DrawCommand::RoundedRect {color,..} if color.3>0 && color.3<50)));
    let disabled=draw(&mut ui,true,false,0.2);
    assert!(!disabled.iter().any(|c|matches!(c,DrawCommand::Shadow {..})));
    assert!(!ui.needs_redraw());
}
#[test]
fn checked_card_icon_appears_at_logical_end_and_can_be_cleared() {
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;
    for (rtl,x,time) in [(false,178.0,0.0),(true,18.0,0.1)] {
        let commands=ui.frame(Input {time,..Input::default()},VIEW,0.0,|f| {
            f.card_with_state(1,B,"Card",CardOptions::default(),CardVisualState {checked:true,rtl,..CardVisualState::default()}, |_| {});
        });
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape {bounds,index,..} if bounds.x==x && bounds.y==18.0 && bounds.width==24.0 && bhoo::shape_data(*index as usize).is_some_and(|s|s.name=="material-android-card-checked" && s.contours.len()==3))),"rtl={rtl}");
    }
    let commands=ui.frame(Input {time:0.2,..Input::default()},VIEW,0.0,|f| {
        f.card_with_state(1,B,"Card",CardOptions::default(),CardVisualState::default(), |_| {});
    });
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::Shape {..})));
    assert!(!ui.needs_redraw());
}
#[test]
fn checked_icon_uses_android_linear_fades_and_shortens_reversal() {
    let mut ui=Ui::default();
    let draw=|ui:&mut Ui,checked,time| ui.frame(Input {time,..Input::default()},VIEW,0.0,|f| {
        f.card_with_state(1,B,"Card",CardOptions::default(),CardVisualState {checked,..CardVisualState::default()}, |_| {});
    });
    let alpha=|commands:&[DrawCommand]| commands.iter().find_map(|c|if let DrawCommand::Shape {index,color,..}=c {bhoo::shape_data(*index as usize).filter(|s|s.name=="material-android-card-checked").map(|_|color.3)} else {None}).unwrap_or(0);
    assert_eq!(alpha(&draw(&mut ui,false,0.0)),0);
    assert_eq!(alpha(&draw(&mut ui,true,0.1)),0);
    let midway=alpha(&draw(&mut ui,true,0.15));assert!((126..=129).contains(&midway),"{midway}");
    assert_eq!(alpha(&draw(&mut ui,false,0.15)),midway);
    assert_eq!(alpha(&draw(&mut ui,false,0.176)),0);
    assert!(!ui.needs_redraw());
}
#[test]
fn disabled_card_uses_disabled_tokens_and_scoped_default_text() {
    for variant in [CardVariant::Filled,CardVariant::Outlined,CardVariant::Elevated] {
        let mut ui=Ui::default();let theme=ui.theme;
        let cmds=ui.frame(Input::default(),VIEW,0.,|f| {
            f.card(1,B,"Card",CardOptions {variant,enabled:false,..CardOptions::default()},|f|f.label("Inside"));
            f.label("Outside");
        });
        let texts:Vec<_>=cmds.iter().filter_map(|c|if let DrawCommand::Text {text,color,..}=c {Some((text.as_ref(),*color))} else {None}).collect();
        assert_eq!(texts,vec![("Inside",theme.text.opacity(0.38)),("Outside",theme.text)]);
        let expected=match variant {CardVariant::Filled=>theme.surface_highest.mix(theme.surface_variant,0.38),CardVariant::Outlined=>theme.surface_low.mix(theme.outline,0.12),CardVariant::Elevated=>theme.background};
        assert!(cmds.iter().any(|c|matches!(c,DrawCommand::RoundedRect {color,..} if *color==expected)),"{variant:?}");
    }
}
#[test]
fn android_xml_card_renders_children_and_caller_checked_state() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    state.checked.insert("example_card".into(),true);
    let commands=ui.frame(Input::default(),VIEW,0.,|f| {xml_card::LAYOUT.render(f,B,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,..} if text.contains("Card from"))));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.contains("Card from") && bounds.x==42.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect {bounds,radius,color} if *bounds==B && *radius==20. && *color==Color(231,234,249,255))));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Outline {bounds,radius,width,color} if *bounds==B && *radius==20. && *width==2. && *color==Color(82,97,168,255))));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shadow {blur,..} if *blur>10.0)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape {index,bounds,color,..} if bhoo::shape_data(*index as usize).is_some_and(|s|s.name=="material-android-card-checked") && *bounds==Rect{x:174.,y:70.,width:32.,height:32.} && *color==Color(82,97,168,255))));
    assert!(ui.semantics().iter().any(|node|node.enabled));
    state.rtl=true;
    let commands=ui.frame(Input {time:0.1,..Input::default()},VIEW,0.,|f| {xml_card::LAYOUT.render(f,B,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.contains("Card from") && bounds.x==42.)));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape {index,bounds,..} if bhoo::shape_data(*index as usize).is_some_and(|s|s.name=="material-android-card-checked") && bounds.x==14. && bounds.y==70.)));
}
#[test]
fn explicit_zero_card_elevation_removes_base_shadow() {
    static CARD:android_xml::Node=android_xml::Node {relative_rules:&[android_xml::RelativeRule {kind:57,anchor:"Elevated"},android_xml::RelativeRule {kind:71,anchor:"0.0"}],..xml_card::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let commands=ui.frame(Input::default(),VIEW,0.,|f| {CARD.render(f,B,&mut state);});
    assert!(!commands.iter().any(|c|matches!(c,DrawCommand::Shadow {..})));
}
#[test]
fn android_card_ignores_view_padding_in_favor_of_content_padding() {
    static CARD:android_xml::Node=android_xml::Node {padding_edges:[99.;4],padding_explicit:[true;4],..xml_card::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let commands=ui.frame(Input::default(),VIEW,0.,|f| {CARD.render(f,B,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.contains("Card from") && bounds.x==42.)));
}
#[test]
fn android_card_compat_padding_insets_body_and_children() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let commands=ui.frame(Input::default(),VIEW,0.,|f| {xml_compat_card::LAYOUT.render(f,B,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect {bounds,radius,color} if *bounds==Rect{x:24.,y:28.,width:172.,height:60.} && *radius==20. && *color==Color(231,234,249,255))));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Text {text,bounds,..} if text.contains("Card with") && bounds.x==56. && bounds.y>=44.)));
}
#[test]
fn android_card_compat_padding_respects_max_elevation_and_corner_overlap() {
    static CARD:android_xml::Node=android_xml::Node {relative_rules:&[android_xml::RelativeRule {kind:57,anchor:"Outlined"},android_xml::RelativeRule {kind:66,anchor:"20"},android_xml::RelativeRule {kind:71,anchor:"2"},android_xml::RelativeRule {kind:72,anchor:"8"},android_xml::RelativeRule {kind:73,anchor:"true"},android_xml::RelativeRule {kind:74,anchor:"false"}],..xml_card::LAYOUT};
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    let commands=ui.frame(Input::default(),VIEW,0.,|f| {CARD.render(f,B,&mut state);});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect {bounds,..} if *bounds==Rect{x:18.,y:22.,width:184.,height:72.})));
}
#[test]
fn checked_card_state_colors_follow_android_selector_priority() {
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let theme=ui.theme;
    let commands=ui.frame(Input::default(),VIEW,0.,|f| {
        f.card_with_state(1,B,"Card",CardOptions {variant:CardVariant::Outlined,..CardOptions::default()},CardVisualState {checked:true,dragged:true,..CardVisualState::default()}, |_| {});
    });
    let colors:Vec<_>=commands.iter().filter_map(|c|if let DrawCommand::RoundedRect {color,..}=c {Some(*color)} else {None}).collect();
    assert!(colors.contains(&theme.secondary));
    assert!(colors.contains(&theme.secondary.opacity(0.16)));
    assert!(!colors.contains(&theme.text.opacity(0.08)));
    let commands=ui.frame(Input {time:0.1,..Input::default()},VIEW,0.,|f| {
        f.card_with_state(1,B,"Card",CardOptions {variant:CardVariant::Outlined,enabled:false,..CardOptions::default()},CardVisualState {checked:true,..CardVisualState::default()}, |_| {});
    });
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape {color,..} if *color==theme.outline.opacity(0.12))));
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect {color,..} if *color==theme.secondary.opacity(0.16))));
}
#[test]
fn xml_card_color_selectors_respect_state_and_item_order() {
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=android_xml::LayoutState::default();
    state.view_dragged.insert("selector_card".into(),true);
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,enabled,time| ui.frame(Input {time,..Input::default()},VIEW,0.,|f| f.with_enabled(enabled,|f| {xml_selector_card::LAYOUT.render(f,B,state);}));
    let dragged=draw(&mut ui,&mut state,true,0.0);
    assert!(dragged.iter().any(|c|matches!(c,DrawCommand::Outline {color,..} if *color==Color(0,109,59,255))));
    state.checked.insert("selector_card".into(),true);
    let checked=draw(&mut ui,&mut state,true,0.1);
    assert!(checked.iter().any(|c|matches!(c,DrawCommand::Outline {color,..} if *color==Color(179,38,30,255))));
    assert!(checked.iter().any(|c|matches!(c,DrawCommand::RoundedRect {color,..} if *color==Color(179,38,30,64))));
    assert!(checked.iter().any(|c|matches!(c,DrawCommand::Shape {color,..} if *color==Color(179,38,30,255))));
    let disabled=draw(&mut ui,&mut state,false,0.2);
    assert!(disabled.iter().any(|c|matches!(c,DrawCommand::Outline {color,..} if *color==Color(136,136,136,255))));
    assert!(disabled.iter().any(|c|matches!(c,DrawCommand::Shape {color,..} if *color==Color(136,136,136,255))));
}
#[test]
fn xml_card_theme_color_selectors_follow_live_theme() {
    let mut ui=Ui::default();ui.motion_preference=MotionPreference::Reduced;let mut state=android_xml::LayoutState::default();
    state.checked.insert("theme_card".into(),true);
    for (time,color) in [(0.0,Color(50,100,150,255)),(0.1,Color(190,60,40,255))] {
        ui.theme.secondary=color;
        let commands=ui.frame(Input {time,..Input::default()},VIEW,0.,|f| {xml_theme_card::LAYOUT.render(f,B,&mut state);});
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Outline {color:actual,..} if *actual==color)));
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect {color:actual,..} if *actual==color.opacity(41.0/255.0))));
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape {color:actual,..} if *actual==color)));
    }
}
#[test]
fn xml_card_ripple_uses_pressed_checkable_selector_and_cancel_does_not_click() {
    let mut ui=Ui::default();let mut state=android_xml::LayoutState::default();
    ui.theme.primary=Color(19,80,145,255);
    let draw=|ui:&mut Ui,state:&mut android_xml::LayoutState,input| {
        let mut action=None;let commands=ui.frame(input,VIEW,0.,|f| {action=xml_theme_card::LAYOUT.render(f,B,state);});(commands,action)
    };
    draw(&mut ui,&mut state,Input {time:0.,..Input::default()});
    draw(&mut ui,&mut state,Input {time:0.1,pointer:Some([100.,80.]),pressed:true,..Input::default()});
    let (commands,_)=draw(&mut ui,&mut state,Input {time:0.2,pointer:Some([100.,80.]),..Input::default()});
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Ripple {color,..} if color.0==19 && color.1==80 && color.2==145)));
    let (_,action)=draw(&mut ui,&mut state,Input {time:0.21,pointer:Some([100.,80.]),cancelled:true,..Input::default()});
    assert!(action.is_none());
}

#[test]
fn segmented_card_radii_and_swipe_dismiss_box_work() {
    assert_eq!(segmented_card_radii(0, 3, 24.0, 4.0, 0.0, 0.0, 0.0), (24.0, 4.0));
    assert_eq!(segmented_card_radii(1, 3, 24.0, 4.0, 0.0, 0.0, 0.0), (4.0, 4.0));
    assert_eq!(segmented_card_radii(1, 3, 24.0, 4.0, 1.0, 0.0, 0.0), (24.0, 4.0));
    assert_eq!(segmented_card_radii(2, 3, 24.0, 4.0, 0.0, 0.0, 0.0), (4.0, 24.0));

    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 80.0 };
    let mut state = SwipeDismissState::default();
    let _ = ui.frame(Input { pointer: Some([40.0, 40.0]), pressed: true, time: 0.0, ..Input::default() }, bounds, 0.0, |f| {
        f.swipe_dismiss_box(99, bounds, &mut state, 24.0, 24.0, |_, _| {});
    });
    let _ = ui.frame(Input { pointer: Some([220.0, 40.0]), time: 0.05, ..Input::default() }, bounds, 0.0, |f| {
        f.swipe_dismiss_box(99, bounds, &mut state, 24.0, 24.0, |_, _| {});
    });
    assert!(state.detach_progress() > 0.99);
    let mut dismissed = false;
    let _ = ui.frame(Input { pointer: Some([220.0, 40.0]), released: true, time: 0.1, ..Input::default() }, bounds, 0.0, |f| {
        dismissed = f.swipe_dismiss_box(99, bounds, &mut state, 24.0, 24.0, |_, _| {});
    });
    assert!(dismissed && state.dismissed);
}
