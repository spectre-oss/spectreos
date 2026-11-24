//! Bounded caller-owned RGBA images and object-fit geometry.
use crate::Rect;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
static NEXT_IMAGE_ID: AtomicU32 = AtomicU32::new(1);
/// Immutable, shared RGBA8 pixels. Loading and decoding are the host application's responsibility.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub(crate) id: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) msdf_px_range: f32,
    pub(crate) pixels: Arc<[u8]>,
}
impl Image {
    /// Construct a bounded image. The native atlas currently supports dimensions up to 1022 pixels.
    pub fn rgba(width: u32, height: u32, pixels: Vec<u8>) -> Option<Self> {
        Self::new_inner(width, height, 0.0, pixels)
    }
    /// Construct a Multi-Channel Signed Distance Field (MSDF) RGBA8 image (`px_range > 0`)
    /// for resolution-independent vector rendering in the GLES fragment shader.
    pub fn msdf(width: u32, height: u32, px_range: f32, pixels: Vec<u8>) -> Option<Self> {
        if !px_range.is_finite() || px_range <= 0.0 {
            return None;
        }
        Self::new_inner(width, height, px_range, pixels)
    }
    fn new_inner(width: u32, height: u32, msdf_px_range: f32, pixels: Vec<u8>) -> Option<Self> {
        if width == 0 || height == 0 || width > 1022 || height > 1022 {
            return None;
        }
        let length = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;
        if pixels.len() != length {
            return None;
        }
        let id = NEXT_IMAGE_ID
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .ok()?;
        Some(Self {
            id,
            width,
            height,
            msdf_px_range,
            pixels: pixels.into(),
        })
    }
    pub fn msdf_px_range(&self) -> f32 {
        self.msdf_px_range
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub fn id(&self) -> u32 {
        self.id
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    #[default]
    Cover,
    Contain,
}
/// Destination and normalized source coordinates for centered object-fit.
pub fn fit_image(bounds: Rect, width: u32, height: u32, fit: ImageFit) -> Option<(Rect, [f32; 4])> {
    if width == 0
        || height == 0
        || !bounds.width.is_finite()
        || !bounds.height.is_finite()
        || bounds.width <= 0.0
        || bounds.height <= 0.0
    {
        return None;
    }
    let iw = width as f32;
    let ih = height as f32;
    match fit {
        ImageFit::Contain => {
            let scale = (bounds.width / iw).min(bounds.height / ih);
            let w = iw * scale;
            let h = ih * scale;
            Some((
                Rect {
                    x: bounds.x + (bounds.width - w) / 2.0,
                    y: bounds.y + (bounds.height - h) / 2.0,
                    width: w,
                    height: h,
                },
                [0.0, 0.0, 1.0, 1.0],
            ))
        }
        ImageFit::Cover => {
            let scale = (bounds.width / iw).max(bounds.height / ih);
            let u = bounds.width / (iw * scale);
            let v = bounds.height / (ih * scale);
            Some((
                bounds,
                [
                    (1.0 - u) / 2.0,
                    (1.0 - v) / 2.0,
                    (1.0 + u) / 2.0,
                    (1.0 + v) / 2.0,
                ],
            ))
        }
    }
}

#[cfg(test)]
#[path = "media.spec.rs"]
mod tests;
