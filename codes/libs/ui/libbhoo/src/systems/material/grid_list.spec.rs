use super::*;

#[test]
fn grid_list_lays_out_columns_and_reports_clicked_tile() {
    let mut ui = Ui::default();
    let bounds = Rect { x: 0.0, y: 0.0, width: 320.0, height: 320.0 };
    let items = [
        GridListItem { label: "Files", subtitle: Some("12 items"), icon: Some(MaterialIcon::Description), image: None },
        GridListItem { label: "Photos", subtitle: None, icon: Some(MaterialIcon::Image), image: None },
        GridListItem { label: "Settings", subtitle: None, icon: Some(MaterialIcon::Settings), image: None },
    ];
    let _ = ui.frame(Input { time: 0.0, ..Input::default() }, bounds, 8.0, |f| {
        f.grid_list(100, &items, Some(0), GridListOptions::default());
    });
    let photos = ui.semantics().iter().find(|n| n.label.as_ref() == "Photos").expect("Photos tile");
    let p = [photos.bounds.x + 20.0, photos.bounds.y + 20.0];
    let _ = ui.frame(Input { pointer: Some(p), pressed: true, time: 0.1, ..Input::default() }, bounds, 8.0, |f| {
        f.grid_list(100, &items, Some(0), GridListOptions::default());
    });
    let mut clicked = None;
    let _ = ui.frame(Input { pointer: Some(p), released: true, time: 0.15, ..Input::default() }, bounds, 8.0, |f| {
        clicked = f.grid_list(100, &items, Some(0), GridListOptions::default());
    });
    assert_eq!(clicked, Some(1));
}
