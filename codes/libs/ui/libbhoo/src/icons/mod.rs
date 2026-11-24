//! Bundled vector & raster icon catalogs (Material Symbols, Material Outlined system icons,
//! Material Navigation icons, and Android widget icons).
pub(crate) mod android_search;
pub(crate) mod android_time_picker;
pub(crate) mod material_card;
pub(crate) mod material_navigation;
pub(crate) mod material_rating;
pub(crate) mod material_symbols;
pub mod material_system;

pub use material_system::{SystemIcon, decode_rle4, decode_rle8, system_icon_image, system_icon_msdf};

#[cfg(test)]
#[path = "material_symbols.spec.rs"]
mod material_symbols_tests;
