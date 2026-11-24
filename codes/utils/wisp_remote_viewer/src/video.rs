use std::{io::Cursor, path::Path};

#[derive(Debug)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
    pub png: Vec<u8>,
}

pub fn decode(png: Vec<u8>) -> Result<VideoFrame, String> {
    let mut decoder = png::Decoder::new(Cursor::new(&png));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let info = reader.info();
    let count = (info.width as usize)
        .checked_mul(info.height as usize)
        .ok_or("image dimensions overflow")?;
    if info.width == 0
        || info.height == 0
        || count > super::protocol::MAX_BINARY / 4
        || reader.output_buffer_size() > super::protocol::MAX_BINARY
    {
        return Err("PNG exceeds viewer image limits".into());
    }
    let mut bytes = vec![0; reader.output_buffer_size()];
    let output = reader.next_frame(&mut bytes).map_err(|e| e.to_string())?;
    let bytes = &bytes[..output.buffer_size()];
    let pixels = match output.color_type {
        png::ColorType::Rgba => bytes
            .chunks_exact(4)
            .map(|p| rgb(p[0], p[1], p[2]))
            .collect(),
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .map(|p| rgb(p[0], p[1], p[2]))
            .collect(),
        png::ColorType::Grayscale => bytes.iter().map(|&p| rgb(p, p, p)).collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .map(|p| rgb(p[0], p[0], p[0]))
            .collect(),
        _ => return Err("unsupported decoded PNG format".into()),
    };
    Ok(VideoFrame {
        width: output.width,
        height: output.height,
        pixels,
        png,
    })
}
fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}
impl VideoFrame {
    pub fn encoded_png(&self) -> Result<Vec<u8>, String> {
        if !self.png.is_empty() {
            return Ok(self.png.clone());
        }
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, self.width, self.height);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_compression(png::Compression::Fast);
            let rgb: Vec<u8> = self
                .pixels
                .iter()
                .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
                .collect();
            encoder
                .write_header()
                .map_err(|e| e.to_string())?
                .write_image_data(&rgb)
                .map_err(|e| e.to_string())?;
        }
        Ok(bytes)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.encoded_png().map_err(std::io::Error::other)?)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Layout {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
pub const TOOLBAR: u32 = 40;
impl Layout {
    pub fn fit(window: [u32; 2], image: [u32; 2]) -> Self {
        let available = window[1].saturating_sub(TOOLBAR);
        if window[0] == 0 || available == 0 || image[0] == 0 || image[1] == 0 {
            return Self::default();
        }
        let ratio = (window[0] as f64 / image[0] as f64).min(available as f64 / image[1] as f64);
        let width = (image[0] as f64 * ratio).round().max(1.0) as u32;
        let height = (image[1] as f64 * ratio).round().max(1.0) as u32;
        Self {
            x: (window[0] - width) / 2,
            y: TOOLBAR + (available - height) / 2,
            width,
            height,
        }
    }
    pub fn point(&self, p: [f64; 2], logical: [f64; 2], clamp: bool) -> Option<[f64; 2]> {
        if self.width == 0 || self.height == 0 {
            return None;
        }
        let x = (p[0] - self.x as f64) / self.width as f64;
        let y = (p[1] - self.y as f64) / self.height as f64;
        if !clamp && (!(0.0..1.0).contains(&x) || !(0.0..1.0).contains(&y)) {
            return None;
        }
        Some([
            (x * logical[0]).clamp(0.0, (logical[0] - 0.001).max(0.0)),
            (y * logical[1]).clamp(0.0, (logical[1] - 0.001).max(0.0)),
        ])
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn letterbox_coordinates_and_drag_clamping() {
        let layout = Layout::fit([800, 840], [400, 800]);
        assert_eq!(
            [layout.x, layout.y, layout.width, layout.height],
            [200, 40, 400, 800]
        );
        assert_eq!(
            layout.point([400.0, 440.0], [400.0, 800.0], false),
            Some([200.0, 400.0])
        );
        assert!(
            layout
                .point([100.0, 440.0], [400.0, 800.0], false)
                .is_none()
        );
        assert!(layout.point([400.0, 20.0], [400.0, 800.0], false).is_none());
        assert_eq!(
            layout.point([0.0, 0.0], [400.0, 800.0], true),
            Some([0.0, 0.0])
        );
    }
    #[test]
    fn decodes_real_png_and_preserves_colors() {
        let mut data = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut data, 2, 1);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()
                .unwrap()
                .write_image_data(&[255, 0, 0, 255, 0, 175, 255, 255])
                .unwrap();
        }
        assert_eq!(decode(data).unwrap().pixels, [0xff0000, 0x00afff]);
    }
}
