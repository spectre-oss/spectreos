use bhoo::*;
mod xml_fixture { include!(concat!(env!("CARGO_MANIFEST_DIR"),"/../../../apps/bhoo-web/src/android_material_divider.rs")); }
#[test]
fn android_xml_divider_wrap_content_and_logical_insets() {
    let mut ui=Ui::default();let viewport=Rect{x:0.,y:0.,width:200.,height:100.};
    for (rtl,x) in [(false,72.),(true,16.)] {
        let mut state=android_xml::LayoutState::default();state.rtl=rtl;
        let cmds=ui.frame(Input::default(),viewport,0.,|f| {xml_fixture::LAYOUT.render(f,viewport,&mut state);});
        assert!(cmds.iter().any(|c|matches!(c,DrawCommand::RoundedRect {bounds,..} if *bounds==Rect{x,y:0.,width:112.,height:8.})),"rtl={rtl}");
    }
}
#[test]
fn divider_orientations_insets_and_idle_behavior() {
    let mut ui = Ui::default();
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    let commands = ui.frame(Input::default(), bounds, 0.0, |f| {
        f.divider_at(
            Rect {
                x: 10.0,
                y: 10.0,
                width: 100.0,
                height: 9.0,
            },
            DividerOrientation::Horizontal,
            12.0,
            8.0,
        );
        f.divider_at(
            Rect {
                x: 30.0,
                y: 20.0,
                width: 9.0,
                height: 60.0,
            },
            DividerOrientation::Vertical,
            5.0,
            15.0,
        );
        f.divider_at(bounds, DividerOrientation::Horizontal, 100.0, 100.0);
    });
    let lines: Vec<_> = commands
        .iter()
        .filter_map(|c| {
            if let DrawCommand::RoundedRect { bounds, .. } = c {
                Some(*bounds)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            Rect {
                x: 22.0,
                y: 14.0,
                width: 80.0,
                height: 1.0
            },
            Rect {
                x: 34.0,
                y: 25.0,
                width: 1.0,
                height: 40.0
            }
        ]
    );
    assert!(ui.semantics().is_empty());
    assert!(!ui.needs_redraw());
}
#[test]
fn material_divider_mirrors_horizontal_insets_and_supports_heavy_thickness() {
    let mut ui=Ui::default();let viewport=Rect{x:0.,y:0.,width:200.,height:100.};
    let cmds=ui.frame(Input::default(),viewport,0.,|f| {
        let bounds=Rect{x:10.,y:10.,width:100.,height:12.};
        f.divider_styled_at(bounds,DividerOptions {inset_start:20.,inset_end:4.,thickness:8.,rtl:true,..DividerOptions::default()});
        f.divider_styled_at(Rect{x:120.,y:10.,width:12.,height:80.},DividerOptions {orientation:DividerOrientation::Vertical,inset_start:20.,inset_end:4.,thickness:8.,rtl:true,..DividerOptions::default()});
    });
    let lines:Vec<_>=cmds.iter().filter_map(|c|if let DrawCommand::RoundedRect {bounds,..}=c {Some(*bounds)} else {None}).collect();
    assert_eq!(lines,vec![Rect{x:14.,y:12.,width:76.,height:8.},Rect{x:122.,y:30.,width:8.,height:56.}]);
    assert!(!ui.needs_redraw());
}
#[test]
fn invalid_divider_thickness_and_excess_insets_draw_nothing() {
    let mut ui=Ui::default();let b=Rect{x:0.,y:0.,width:50.,height:10.};
    let cmds=ui.frame(Input::default(),b,0.,|f| {
        for thickness in [0.,-1.,f32::NAN,f32::INFINITY] {f.divider_styled_at(b,DividerOptions {thickness,..DividerOptions::default()});}
        f.divider_styled_at(b,DividerOptions {inset_start:30.,inset_end:30.,..DividerOptions::default()});
    });
    assert!(!cmds.iter().any(|c|matches!(c,DrawCommand::RoundedRect {..})));
}
