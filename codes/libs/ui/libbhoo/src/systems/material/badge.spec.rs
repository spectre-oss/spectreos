use bhoo::*;
const VIEW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 200.0,
    height: 100.0,
};
const ICON: Rect = Rect {
    x: 20.0,
    y: 20.0,
    width: 24.0,
    height: 24.0,
};
#[test]
fn badges_anchor_to_icons_mirror_and_augment_the_owner() {
    let mut ui = Ui::default();
    let commands = ui.frame(Input::default(), VIEW, 0.0, |f| {
        f.target(1, ICON, "Inbox", true, Role::Button);
        assert!(f.badge_at(1, ICON, BadgeContent::Dot, "New notification", false));
        f.target(2, ICON, "Inbox", true, Role::Button);
        assert!(f.badge_at(2, ICON, BadgeContent::Label("12"), "12 messages", true));
        assert!(!f.badge_at(2, ICON, BadgeContent::Label("12345"), "invalid", false));
    });
    let rects: Vec<_> = commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::RoundedRect { bounds, color, .. } => Some((*bounds, *color)),
            _ => None,
        })
        .collect();
    assert_eq!(rects.len(), 2);
    assert_eq!(
        rects[0].0,
        Rect {
            x: 38.0,
            y: 20.0,
            width: 6.0,
            height: 6.0
        }
    );
    assert_eq!(rects[1].0.y, 18.0);
    assert!((rects[1].0.x + rects[1].0.width - 32.0).abs() < 0.001);
    assert_eq!(rects[1].0.height, 16.0);
    assert_eq!(rects[0].1, ui.theme.error);
    assert_eq!(ui.semantics().len(), 2);
    assert_eq!(&*ui.semantics()[0].label, "Inbox, New notification");
    assert_eq!(&*ui.semantics()[1].label, "Inbox, 12 messages");
    assert!(!ui.needs_redraw());
}

#[test]
fn numeric_badge_formats_overflow_and_resizes_without_changing_owner() {
    let mut ui=Ui::default();
    for (value,max,expected) in [(0,999,"0"),(999,999,"999"),(1000,999,"999+"),(8,0,"0+"),(u32::MAX,u32::MAX,"4294967295")] {
        let commands=ui.frame(Input::default(),VIEW,0.0,|f|{
            f.target(1,ICON,"Inbox",true,Role::Button);
            assert!(f.badge_at(1,ICON,BadgeContent::Number{value,max},"",false));
        });
        let (label,bounds)=commands.iter().find_map(|c|if let DrawCommand::Text{text,bounds,..}=c{Some((text.to_string(),*bounds))}else{None}).unwrap();
        assert_eq!(label,expected);assert!(bounds.width>=16.);
        if value==u32::MAX {assert!(bounds.width>34.);}
        assert_eq!(ui.semantics()[0].bounds,ICON);
    }
    let commands=ui.frame(Input::default(),VIEW,0.0,|f|{f.target(1,ICON,"Inbox",true,Role::Button);f.badge_at(1,ICON,BadgeContent::Number{value:1000,max:999},"",true);});
    let bounds=commands.iter().find_map(|c|if let DrawCommand::Text{bounds,..}=c{Some(*bounds)}else{None}).unwrap();
    assert!((bounds.x+bounds.width-(ICON.x+12.)).abs()<0.01);
    assert!(!ui.needs_redraw());
}

#[test]
fn text_badge_truncates_at_grapheme_boundaries_and_mirrors() {
    let mut ui=Ui::default();
    for (text,max,expected) in [("New",4,"New"),("Updates",4,"Upd…"),("Updates",1,"…"),("a\u{301}bcde",4,"a\u{301}bc…"),("👩‍💻xyzq",3,"👩‍💻x…")] {
        let commands=ui.frame(Input::default(),VIEW,0.0,|f|{f.target(1,ICON,"Inbox",true,Role::Button);assert!(f.badge_at(1,ICON,BadgeContent::Text{text,max_graphemes:max},"",false));});
        let label=commands.iter().find_map(|c|if let DrawCommand::Text{text,..}=c{Some(text.to_string())}else{None}).unwrap();assert_eq!(label,expected);
    }
    ui.frame(Input::default(),VIEW,0.0,|f|{assert!(!f.badge_at(1,ICON,BadgeContent::Text{text:"",max_graphemes:4},"",false));assert!(!f.badge_at(1,ICON,BadgeContent::Text{text:"New",max_graphemes:0},"",false));});
    let commands=ui.frame(Input::default(),VIEW,0.0,|f|{f.badge_at(1,ICON,BadgeContent::Text{text:"Updates",max_graphemes:4},"",true);});
    let b=commands.iter().find_map(|c|if let DrawCommand::Text{bounds,..}=c{Some(*bounds)}else{None}).unwrap();assert!((b.x+b.width-(ICON.x+12.)).abs()<0.01);
}

#[test]
fn badge_gravity_places_all_logical_corners_in_both_directions() {
    let mut ui=Ui::default();
    for gravity in [BadgeGravity::TopStart,BadgeGravity::TopEnd,BadgeGravity::BottomStart,BadgeGravity::BottomEnd] {
        for rtl in [false,true] {
            let commands=ui.frame(Input::default(),VIEW,0.0,|f|{f.target(1,ICON,"Inbox",true,Role::Button);assert!(f.badge_at_gravity(1,ICON,BadgeContent::Label("3"),"",gravity,rtl));});
            let b=commands.iter().find_map(|c|if let DrawCommand::Text{bounds,..}=c{Some(*bounds)}else{None}).unwrap();
            let start=matches!(gravity,BadgeGravity::TopStart|BadgeGravity::BottomStart);let left=start!=rtl;
            if left {assert!((b.x+b.width-(ICON.x+12.)).abs()<0.01);}else{assert!((b.x-(ICON.x+ICON.width-12.)).abs()<0.01);}
            if matches!(gravity,BadgeGravity::BottomStart|BadgeGravity::BottomEnd) {assert_eq!(b.y,ICON.y+ICON.height-14.);}else{assert_eq!(b.y,ICON.y-16.+14.);}
            assert_eq!(ui.semantics()[0].bounds,ICON);
        }
    }
}

#[test]
fn badge_offsets_move_toward_center_for_every_corner_and_direction() {
    let mut ui=Ui::default();
    for gravity in [BadgeGravity::TopStart,BadgeGravity::TopEnd,BadgeGravity::BottomStart,BadgeGravity::BottomEnd] {for rtl in [false,true] {
        let mut position=|horizontal_offset,vertical_offset| {
            let commands=ui.frame(Input::default(),VIEW,0.0,|f|{f.target(1,ICON,"Inbox",true,Role::Button);assert!(f.badge_at_with_options(1,ICON,BadgeContent::Label("3"),"",BadgeOptions{gravity,rtl,horizontal_offset,vertical_offset}));});
            commands.iter().find_map(|c|if let DrawCommand::Text{bounds,..}=c{Some(*bounds)}else{None}).unwrap()
        };
        let base=position(0.,0.);let inward=position(4.,5.);let outward=position(-4.,-5.);
        let left=matches!(gravity,BadgeGravity::TopStart|BadgeGravity::BottomStart)!=rtl;
        let bottom=matches!(gravity,BadgeGravity::BottomStart|BadgeGravity::BottomEnd);
        assert_eq!(inward.x-base.x,if left {4.}else{-4.});assert_eq!(inward.y-base.y,if bottom {-5.}else{5.});
        assert_eq!(outward.x-base.x,if left {-4.}else{4.});assert_eq!(outward.y-base.y,if bottom {5.}else{-5.});
        assert_eq!(ui.semantics()[0].bounds,ICON);
    }}
    ui.frame(Input::default(),VIEW,0.0,|f|{assert!(!f.badge_at_with_options(1,ICON,BadgeContent::Dot,"",BadgeOptions{horizontal_offset:f32::NAN,..BadgeOptions::default()}));});
}
