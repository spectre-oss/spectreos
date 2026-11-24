use crate::*;

#[test]
fn scaffold_switches_between_bottom_bar_and_navigation_rail_at_breakpoint() {
    let mut ui = Ui::default();
    let opts = ScaffoldOptions::default();

    let compact_bounds = Rect { x: 0.0, y: 0.0, width: 400.0, height: 800.0 };
    let mut compact_slots = None;
    ui.frame(Input::default(), compact_bounds, 0.0, |f| {
        compact_slots = Some(f.scaffold_slots(compact_bounds, opts));
    });
    let cs = compact_slots.unwrap();
    assert!(cs.compact);
    assert!(cs.nav_rail.is_none());
    assert!(cs.bottom_bar.is_some());
    assert_eq!(cs.body.height, 800.0 - 64.0 - 80.0);

    let wide_bounds = Rect { x: 0.0, y: 0.0, width: 900.0, height: 700.0 };
    let mut wide_slots = None;
    ui.frame(Input::default(), wide_bounds, 0.0, |f| {
        wide_slots = Some(f.scaffold_slots(wide_bounds, opts));
    });
    let ws = wide_slots.unwrap();
    assert!(!ws.compact);
    assert!(ws.nav_rail.is_some());
    assert!(ws.bottom_bar.is_none());
    assert_eq!(ws.body.width, 900.0 - 80.0);
}

#[test]
fn scaffold_adaptive_expands_rail_and_shows_mobile_drawer() {
    let mut ui = Ui::default();
    ui.motion_preference = MotionPreference::Reduced;
    let wide = Rect { x: 0.0, y: 0.0, width: 960.0, height: 640.0 };
    let mut drawer_open = false;
    let mut rail_w = 0.0;
    let _ = ui.frame(Input::default(), wide, 0.0, |f| {
        let slots = f.scaffold_adaptive(
            10,
            wide,
            ScaffoldAdaptiveOptions::default(),
            true,
            &mut drawer_open,
            |_, _| {},
            |_, r, exp, compact| {
                assert!(!compact && exp > 0.99);
                rail_w = r.width;
            },
            |_, _| {},
            |_, _| {},
        );
        assert_eq!(slots.body.width, 960.0 - 256.0);
    });
    assert!((rail_w - 256.0).abs() < 0.01);

    let mobile = Rect { x: 0.0, y: 0.0, width: 400.0, height: 700.0 };
    drawer_open = true;
    let mut saw_drawer = false;
    let _ = ui.frame(Input::default(), mobile, 0.0, |f| {
        f.scaffold_adaptive(
            11,
            mobile,
            ScaffoldAdaptiveOptions::default(),
            false,
            &mut drawer_open,
            |_, _| {},
            |_, r, exp, compact| {
                if !compact && exp > 0.5 && r.width >= 279.0 {
                    saw_drawer = true;
                }
            },
            |_, _| {},
            |_, _| {},
        );
    });
    assert!(saw_drawer);
}
