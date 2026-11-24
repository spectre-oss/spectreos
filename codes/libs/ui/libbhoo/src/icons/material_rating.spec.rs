use super::*;

#[test]
fn rating_star_geometry_and_shape_index_match() {
    assert_eq!(STAR.name, "rating-star-filled");
    assert_eq!(STAR.points.len(), 11);
    assert_eq!(STAR.triangles.len(), 30);
    for &idx in STAR.triangles {
        assert!((idx as usize) < STAR.points.len());
    }
    let rating_idx = crate::beercss_shapes::SHAPES.len()
        + crate::icons::material_navigation::ICONS.len()
        + crate::icons::material_symbols::ICONS.len();
    let looked_up = crate::shape_data(rating_idx).expect("rating star index registered");
    assert_eq!(looked_up.name, STAR.name);
}
