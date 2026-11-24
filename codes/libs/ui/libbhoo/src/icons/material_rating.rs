//! Filled star geometry from Google's Apache-2.0 Material Icons grade asset.
use crate::beercss_shapes::ShapeData;

pub static STAR: ShapeData = ShapeData {
    name: "rating-star-filled",
    points: &[
        [0.5, 0.7195833],
        [0.7575, 0.875],
        [0.6891667, 0.5820833],
        [0.9166667, 0.385],
        [0.6170833, 0.3595833],
        [0.5, 0.0833333],
        [0.3829167, 0.3595833],
        [0.0833333, 0.385],
        [0.3108333, 0.5820833],
        [0.2425, 0.875],
        [0.5, 0.5],
    ],
    contours: &[0, 10],
    triangles: &[
        10, 0, 1, 10, 1, 2, 10, 2, 3, 10, 3, 4, 10, 4, 5, 10, 5, 6, 10, 6, 7, 10, 7, 8, 10, 8, 9,
        10, 9, 0,
    ],
};

#[cfg(test)]
#[path = "material_rating.spec.rs"]
mod tests;
