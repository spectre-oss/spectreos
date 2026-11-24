use bhoo::*;

#[test]
fn avatars_crop_images_and_apply_each_shape() {
    let image=Image::rgba(4,2,vec![255;32]).unwrap();
    let viewport=Rect{x:0.,y:0.,width:300.,height:300.};let mut ui=Ui::default();
    for (shape,radius) in [(AvatarShape::Circle,20.),(AvatarShape::Rounded,4.),(AvatarShape::Square,0.)] {
        let commands=ui.frame(Input::default(),viewport,0.,|f|f.avatar_at(Rect{x:10.,y:20.,width:60.,height:40.},shape,AvatarContent::Image(&image)));
        let (bounds,uv,r)=commands.iter().find_map(|c|match c {DrawCommand::Image{bounds,uv,radius,..}=>Some((*bounds,*uv,*radius)),_=>None}).unwrap();
        assert_eq!(bounds,Rect{x:20.,y:20.,width:40.,height:40.});assert_eq!(uv,[0.25,0.,0.75,1.]);assert_eq!(r,radius);
        assert!(commands.iter().any(|c|matches!(c,DrawCommand::RoundedRect{bounds:b,radius:r,..} if *b==bounds && *r==radius)));
    }
}

#[test]
fn initials_are_centered_and_empty_content_uses_person_symbol() {
    let viewport=Rect{x:0.,y:0.,width:200.,height:200.};let mut ui=Ui::default();
    let commands=ui.frame(Input::default(),viewport,0.,|f|{
        f.avatar_at(Rect{x:10.,y:10.,width:40.,height:40.},AvatarShape::Circle,AvatarContent::Initials("BH"));
        f.avatar_at(Rect{x:60.,y:10.,width:40.,height:40.},AvatarShape::Circle,AvatarContent::Initials(""));
        f.avatar_at(Rect{x:110.,y:10.,width:40.,height:40.},AvatarShape::Circle,AvatarContent::Fallback);
    });
    let (b,style)=commands.iter().find_map(|c|match c {DrawCommand::Text{bounds,text,style,..} if text.as_ref()=="BH"=>Some((*bounds,*style)),_=>None}).unwrap();
    assert_eq!(b,Rect{x:10.,y:10.,width:40.,height:40.});assert_eq!(style.align,1);assert_eq!(style.size,20.);
    let symbols:Vec<_>=commands.iter().filter_map(|c|match c {DrawCommand::Shape{bounds,index,..}=>Some((*bounds,*index)),_=>None}).collect();
    assert_eq!(symbols.len(),2);assert!(symbols.iter().all(|(bounds,index)|bounds.width==30. && shape_data(*index as usize).unwrap().name=="person"));
}

#[test]
fn avatar_group_max_and_total_follow_reference_examples() {
    let viewport=Rect{x:0.,y:0.,width:400.,height:100.};let mut ui=Ui::default();
    let items=[AvatarContent::Initials("A"),AvatarContent::Initials("B"),AvatarContent::Initials("C"),AvatarContent::Initials("D")];
    for (count,max,total,shown,surplus) in [(3,3,None,3,0),(4,3,None,2,2),(3,5,Some(10),3,7),(2,2,Some(3),1,2),(2,5,Some(1),1,0),(2,10,Some(10),2,8),(4,4,Some(4),4,0)] {
        let mut result=AvatarGroupCounts{shown:0,surplus:0};let commands=ui.frame(Input::default(),viewport,0.,|f|{
            result=f.avatar_group_with_options(&items[..count],AvatarGroupOptions{max,total,..AvatarGroupOptions::default()});
        });
        assert_eq!(result,AvatarGroupCounts{shown,surplus});
        let labels:Vec<_>=commands.iter().filter_map(|c|if let DrawCommand::Text{text,..}=c {Some(text.as_ref())}else{None}).collect();
        assert_eq!(labels.len(),shown+usize::from(surplus>0));
        if surplus>0 {assert!(labels.contains(&format!("+{surplus}").as_str()));}
    }
}

#[test]
fn avatar_group_supports_images_shapes_spacing_and_rtl() {
    let image=Image::rgba(4,2,vec![255;32]).unwrap();let viewport=Rect{x:0.,y:0.,width:120.,height:100.};let mut ui=Ui::default();
    let items=[AvatarContent::Image(&image),AvatarContent::Initials("B"),AvatarContent::Fallback];
    let commands=ui.frame(Input::default(),viewport,0.,|f|{f.avatar_group_with_options(&items,AvatarGroupOptions{max:4,overlap:16.,shape:AvatarShape::Square,rtl:true,..AvatarGroupOptions::default()});});
    let image_bounds=commands.iter().find_map(|c|match c {DrawCommand::Image{bounds,radius,..}=>{assert_eq!(*radius,0.);Some(*bounds)},_=>None}).unwrap();
    let label_bounds=commands.iter().find_map(|c|match c {DrawCommand::Text{bounds,text,..} if text.as_ref()=="B"=>Some(*bounds),_=>None}).unwrap();
    assert_eq!(image_bounds.x,78.);assert_eq!(label_bounds.x,50.);
    assert!(commands.iter().any(|c|matches!(c,DrawCommand::Shape{index,..} if shape_data(*index as usize).unwrap().name=="person")));
}

#[test]
fn avatar_group_large_input_only_draws_visible_members() {
    let viewport=Rect{x:0.,y:0.,width:100.,height:100.};let mut ui=Ui::default();
    let items=vec![AvatarContent::Initials("A");100_000];let mut counts=AvatarGroupCounts{shown:0,surplus:0};
    let commands=ui.frame(Input::default(),viewport,0.,|f|{counts=f.avatar_group_with_options(&items,AvatarGroupOptions{max:100_001,..AvatarGroupOptions::default()});});
    assert_eq!(counts.shown,100_000);assert_eq!(counts.surplus,0);
    assert!(commands.iter().filter(|c|matches!(c,DrawCommand::Text{..})).count()<=4);
}
