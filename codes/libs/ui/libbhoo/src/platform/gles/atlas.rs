use super::{
    GLYPH_ATLAS, IMAGE_ATLAS, Renderer, Result,
    fonts::{ShapedLine, text_glyph_params},
};
use crate::{DrawCommand, Rect};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct GlyphKey {
    pub(super) font_id: u16,
    pub(super) glyph_id: u32,
    pub(super) px_size: u16,
    pub(super) sub_bin: u8,
    pub(super) synthetic_bold: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ShapeKey {
    pub(super) family: u8,
    pub(super) weight: u8,
    pub(super) italic: bool,
    pub(super) px_size: u16,
    pub(super) text: Arc<str>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Glyph {
    pub(super) left: i32,
    pub(super) top: i32,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) uv: [f32; 4],
}

fn merge_rows(acc: &mut Option<(usize, usize)>, y0: usize, y1: usize) {
    if y1 <= y0 {
        return;
    }
    *acc = Some(match *acc {
        Some((a0, a1)) => (a0.min(y0), a1.max(y1)),
        None => (y0, y1),
    });
}

impl Renderer {
    pub(super) fn get_or_shape_line(
        &mut self,
        text: &Arc<str>,
        family: u8,
        weight: u8,
        italic: bool,
        px_size: u16,
    ) -> Arc<ShapedLine> {
        let key = ShapeKey {
            family,
            weight,
            italic,
            px_size,
            text: Arc::clone(text),
        };
        if let Some(cached) = self.shape_cache.get(&key) {
            return Arc::clone(cached);
        }
        if self.shape_cache.len() >= 1024 {
            self.shape_cache.clear();
        }
        let shaped = Arc::new(self.fonts.shape_line(
            text,
            family,
            weight,
            italic,
            px_size,
            &mut self.shaper_buf,
        ));
        self.shape_cache.insert(key, Arc::clone(&shaped));
        shaped
    }

    fn pack_glyph(&mut self, key: GlyphKey, dirty: &mut Option<(usize, usize)>) -> bool {
        if self.glyphs.contains_key(&key) {
            return true;
        }
        let Some(r) = self.fonts.face(key.font_id).rasterize_glyph(
            key.glyph_id,
            key.px_size,
            key.sub_bin,
            key.synthetic_bold,
            &mut self.zeno_scratch,
        ) else {
            self.glyphs.insert(
                key,
                Glyph {
                    left: 0,
                    top: 0,
                    width: 0,
                    height: 0,
                    uv: [0.0; 4],
                },
            );
            return true;
        };
        let (w, h) = (r.width, r.height);
        if w == 0 || h == 0 {
            self.glyphs.insert(
                key,
                Glyph {
                    left: r.left,
                    top: r.top,
                    width: 0,
                    height: 0,
                    uv: [0.0; 4],
                },
            );
            return true;
        }
        let [mut x, mut y, mut shelf_height] = self.shelf;
        if x + w + 2 >= GLYPH_ATLAS {
            x = 1;
            y += shelf_height + 2;
            shelf_height = 0;
        }
        if y + h + 2 >= GLYPH_ATLAS || w + 3 >= GLYPH_ATLAS {
            return false;
        }
        for row in 0..h {
            self.atlas[(y + row) * GLYPH_ATLAS + x..(y + row) * GLYPH_ATLAS + x + w]
                .copy_from_slice(&r.bitmap[row * w..row * w + w]);
        }
        merge_rows(dirty, y.saturating_sub(1), (y + h + 2).min(GLYPH_ATLAS));
        self.shelf = [x + w + 2, y, shelf_height.max(h)];
        self.glyphs.insert(
            key,
            Glyph {
                left: r.left,
                top: r.top,
                width: w,
                height: h,
                uv: [
                    x as f32 / GLYPH_ATLAS as f32,
                    y as f32 / GLYPH_ATLAS as f32,
                    (x + w) as f32 / GLYPH_ATLAS as f32,
                    (y + h) as f32 / GLYPH_ATLAS as f32,
                ],
            },
        );
        true
    }

    pub(super) fn prepare_glyphs(
        &mut self,
        commands: &[DrawCommand],
        scale: f32,
    ) -> Result<Option<(usize, usize)>> {
        let mut dirty: Option<(usize, usize)> = None;
        if self.scale != scale {
            self.scale = scale;
            self.glyphs.clear();
            self.shape_cache.clear();
            self.atlas.fill(0);
            self.shelf = [1, 1, 0];
            dirty = Some((0, GLYPH_ATLAS));
        }

        let pack_pass = |this: &mut Self, dirty: &mut Option<(usize, usize)>| -> bool {
            for command in commands {
                if let DrawCommand::Text {
                    bounds,
                    text,
                    style,
                    ..
                } = command
                {
                    if bounds.width <= 0.0
                        || bounds.height <= 0.0
                        || !style.size.is_finite()
                        || style.size <= 0.0
                    {
                        continue;
                    }
                    let (family, weight, italic, px_size, factor) =
                        text_glyph_params(*style, scale);
                    let shaped = this.get_or_shape_line(text, family, weight, italic, px_size);
                    let draw_factor = if (factor - 1.0).abs() < 0.04 {
                        1.0
                    } else {
                        factor
                    };
                    let n = shaped.glyphs.len();
                    let extra_ls = if n > 1 {
                        style.letter_spacing * (n - 1) as f32
                    } else {
                        0.0
                    };
                    let width = (shaped.total_advance_px / scale * draw_factor + extra_ls).max(0.0);
                    let mut x = match style.align {
                        1 => bounds.x + (bounds.width - width) * 0.5,
                        2 => bounds.x + bounds.width - style.padding - width,
                        _ => bounds.x + style.padding,
                    };
                    for sg in &shaped.glyphs {
                        let px_x = (x + sg.x_offset / scale * draw_factor) * scale;
                        let sub_bin = if px_size <= 48 {
                            ((px_x - px_x.floor()) * 4.0).floor().clamp(0.0, 3.0) as u8
                        } else {
                            0
                        };
                        let key = GlyphKey {
                            font_id: sg.font_id,
                            glyph_id: sg.glyph_id,
                            px_size,
                            sub_bin,
                            synthetic_bold: sg.synthetic_bold,
                        };
                        if !this.glyphs.contains_key(&key) && !this.pack_glyph(key, dirty) {
                            return false;
                        }
                        x += sg.x_advance / scale * draw_factor + style.letter_spacing;
                    }
                }
            }
            true
        };

        if !pack_pass(self, &mut dirty) {
            self.glyphs.clear();
            self.atlas.fill(0);
            self.shelf = [1, 1, 0];
            dirty = Some((0, GLYPH_ATLAS));
            if !pack_pass(self, &mut dirty) {
                return Err("Glyphs in this frame exceed the bounded Bhoo glyph atlas".into());
            }
        }
        Ok(dirty)
    }

    fn pack_image(&mut self, image: &crate::Image, dirty: &mut Option<(usize, usize)>) -> bool {
        if self.images.contains_key(&image.id) {
            return true;
        }
        // 16px zero-padding matches TEXTURE_MAX_LEVEL=4 (16x16 mip block) so mipmaps never bleed
        const PAD: usize = 16;
        let (w, h) = (image.width as usize, image.height as usize);
        let [mut x, mut y, mut shelf_height] = self.image_shelf;
        if x < PAD {
            x = PAD;
        }
        if y < PAD {
            y = PAD;
        }
        if x + w + PAD > IMAGE_ATLAS {
            x = PAD;
            y += shelf_height + PAD;
            shelf_height = 0;
        }
        if y + h + PAD > IMAGE_ATLAS {
            return false;
        }
        // MSDF images store raw 3-channel signed distances (do NOT premultiply);
        // normal RGBA images are premultiplied so glGenerateMipmap blends cleanly without halos.
        if image.msdf_px_range > 0.0 {
            for row in 0..h {
                let src = row * w * 4;
                let dst = ((y + row) * IMAGE_ATLAS + x) * 4;
                self.image_atlas[dst..dst + w * 4]
                    .copy_from_slice(&image.pixels[src..src + w * 4]);
            }
        } else {
            for row in 0..h {
                let src = row * w * 4;
                let dst = ((y + row) * IMAGE_ATLAS + x) * 4;
                let srow = &image.pixels[src..src + w * 4];
                let drow = &mut self.image_atlas[dst..dst + w * 4];
                for (sp, dp) in srow.chunks_exact(4).zip(drow.chunks_exact_mut(4)) {
                    let a = sp[3] as u16;
                    dp[0] = ((sp[0] as u16 * a + 127) / 255) as u8;
                    dp[1] = ((sp[1] as u16 * a + 127) / 255) as u8;
                    dp[2] = ((sp[2] as u16 * a + 127) / 255) as u8;
                    dp[3] = sp[3];
                }
            }
        }
        merge_rows(dirty, y.saturating_sub(1), (y + h + 1).min(IMAGE_ATLAS));
        self.images.insert(
            image.id,
            [
                (x as f32 + 0.5) / IMAGE_ATLAS as f32,
                (y as f32 + 0.5) / IMAGE_ATLAS as f32,
                (x as f32 + w as f32 - 0.5) / IMAGE_ATLAS as f32,
                (y as f32 + h as f32 - 0.5) / IMAGE_ATLAS as f32,
            ],
        );
        self.image_shelf = [x + w + PAD, y, shelf_height.max(h)];
        true
    }

    pub(super) fn prepare_images(
        &mut self,
        commands: &[DrawCommand],
    ) -> Result<Option<(usize, usize)>> {
        let mut dirty: Option<(usize, usize)> = None;
        let mut full = false;
        for command in commands {
            if let DrawCommand::Image { image, .. } = command
                && !self.pack_image(image, &mut dirty)
            {
                full = true;
                break;
            }
        }
        if full {
            self.images.clear();
            self.image_shelf = [16, 16, 0];
            self.image_atlas.fill(0);
            dirty = Some((0, IMAGE_ATLAS));
            for command in commands {
                if let DrawCommand::Image { image, .. } = command
                    && !self.pack_image(image, &mut dirty)
                {
                    return Err("Images in this frame exceed the bounded Bhoo image atlas".into());
                }
            }
        }
        Ok(dirty)
    }
}

pub(super) fn clip_glyph(rect: Rect, uv: [f32; 4], clip: Rect) -> Option<(Rect, [f32; 4])> {
    let x = rect.x.max(clip.x);
    let y = rect.y.max(clip.y);
    let right = (rect.x + rect.width).min(clip.x + clip.width);
    let bottom = (rect.y + rect.height).min(clip.y + clip.height);
    if right <= x || bottom <= y {
        return None;
    }
    Some((
        Rect {
            x,
            y,
            width: right - x,
            height: bottom - y,
        },
        [
            uv[0] + (x - rect.x) / rect.width * (uv[2] - uv[0]),
            uv[1] + (y - rect.y) / rect.height * (uv[3] - uv[1]),
            uv[0] + (right - rect.x) / rect.width * (uv[2] - uv[0]),
            uv[1] + (bottom - rect.y) / rect.height * (uv[3] - uv[1]),
        ],
    ))
}
