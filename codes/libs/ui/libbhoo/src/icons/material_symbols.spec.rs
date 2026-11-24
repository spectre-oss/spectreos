use bhoo::*;
#[test]
fn official_symbols_have_valid_meshes_and_preserve_the_design_box() {
    let mut ui=Ui::default();
    let commands=ui.frame(Input::default(),Rect{x:0.,y:0.,width:400.,height:400.},0.,|f| {
        for &icon in MaterialIcon::ALL {
            f.paint_material_icon(Rect{x:10.,y:20.,width:48.,height:24.},icon,Color(1,2,3,255));
        }
    });
    let mut count=0;
    for command in commands {
        if let DrawCommand::Shape{bounds,index,..}=command {
            let icon=MaterialIcon::ALL[count]; count+=1;
            assert_eq!(bounds,Rect{x:22.,y:20.,width:24.,height:24.});
            let mesh=shape_data(index as usize).unwrap();
            assert_eq!(mesh.name,icon.name());
            assert!(!mesh.triangles.is_empty());
            assert_eq!(mesh.triangles.len()%3,0);
            assert!(mesh.triangles.iter().all(|&i| (i as usize)<mesh.points.len()));
            assert!(mesh.points.iter().flatten().all(|&p| p.is_finite() && (-0.001..=1.001).contains(&p)));
        }
    }
    assert_eq!(count,MaterialIcon::ALL.len());
}
