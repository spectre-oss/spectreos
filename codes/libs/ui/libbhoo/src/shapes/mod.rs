//! Auto-generated tessellated vector shapes (see `tools/generate-*.py`).
//! Bundled icon catalogs live in `crate::icons`.
pub mod beercss_shapes;
pub(crate) mod loading_shapes;
pub(crate) mod android_clock_shapes;

/// Shared shape lookup for Beer CSS shapes and bundled vector icons from `crate::icons`.
pub fn shape_data(index: usize) -> Option<&'static beercss_shapes::ShapeData> {
    use crate::icons;
    beercss_shapes::SHAPES.get(index).or_else(|| {
        index.checked_sub(beercss_shapes::SHAPES.len()).and_then(|i| {
            icons::material_navigation::ICONS.get(i).or_else(|| {
                i.checked_sub(icons::material_navigation::ICONS.len()).and_then(|j| {
                    icons::material_symbols::ICONS.get(j).or_else(|| {
                        let extra = j.checked_sub(icons::material_symbols::ICONS.len())?;
                        if extra == 0 {
                            Some(&icons::material_rating::STAR)
                        } else if extra == 1 + android_clock_shapes::SHAPES.len() {
                            Some(&icons::material_card::CHECKED)
                        } else if extra > 1 + android_clock_shapes::SHAPES.len() {
                            let index = extra - 2 - android_clock_shapes::SHAPES.len();
                            icons::android_time_picker::ICONS.get(index).or_else(|| {
                                index
                                    .checked_sub(icons::android_time_picker::ICONS.len())
                                    .and_then(|i| icons::android_search::ICONS.get(i))
                            })
                        } else {
                            android_clock_shapes::SHAPES.get(extra - 1)
                        }
                    })
                })
            })
        })
    })
}

#[cfg(test)]
#[path = "beercss_shapes.spec.rs"]
mod beercss_shapes_tests;
