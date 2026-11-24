//! Material Grid List / Image List displaying uniform media or icon tiles with caption bar.
use crate::*;

/// One tile inside `Frame::grid_list`.
#[derive(Clone, Copy)]
pub struct GridListItem<'a> {
    pub label: &'a str,
    pub subtitle: Option<&'a str>,
    pub icon: Option<MaterialIcon>,
    pub image: Option<&'a Image>,
}

/// Layout and visual options for `Frame::grid_list`.
#[derive(Clone, Copy, Debug)]
pub struct GridListOptions {
    pub columns: usize,
    pub tile_height: f32,
    pub gap: f32,
    pub radius: f32,
    pub show_caption_scrim: bool,
}

impl Default for GridListOptions {
    fn default() -> Self {
        Self {
            columns: 2,
            tile_height: 120.0,
            gap: 12.0,
            radius: 12.0,
            show_caption_scrim: true,
        }
    }
}

impl Frame<'_> {
    /// Material Grid List / Image List rendering responsive multi-column tiles with optional
    /// cover image, centered icon, bottom caption scrim, and selection highlight.
    /// Returns the clicked tile index, if any.
    pub fn grid_list(
        &mut self,
        id: u64,
        items: &[GridListItem<'_>],
        selected: Option<usize>,
        options: GridListOptions,
    ) -> Option<usize> {
        if items.is_empty() {
            return None;
        }
        let cols = options.columns.max(1);
        let rows = items.len().div_ceil(cols);
        let tile_h = options.tile_height.max(48.0);
        let gap = options.gap.max(0.0);
        let t = self.theme();
        let mut clicked = None;

        for row in 0..rows {
            let row_rect = self.reserve(tile_h);
            let tile_w = ((row_rect.width - gap * (cols - 1) as f32) / cols as f32).max(0.0);
            for col in 0..cols {
                let idx = row * cols + col;
                let Some(item) = items.get(idx) else {
                    break;
                };
                let tile = Rect {
                    x: row_rect.x + col as f32 * (tile_w + gap),
                    y: row_rect.y,
                    width: tile_w,
                    height: tile_h,
                };
                let is_sel = selected == Some(idx);
                let r = self.interact(id.wrapping_add(idx as u64), tile, true);
                let bg = if is_sel { t.primary_container } else { t.surface_high };
                let fg = if is_sel { t.on_primary_container } else { t.text };
                self.paint_rect(tile, options.radius, bg);
                if let Some(img) = item.image {
                    self.paint_image(tile, img, ImageFit::Cover, options.radius);
                } else if let Some(icon) = item.icon {
                    let icon_sz = 32.0_f32.min(tile.width * 0.4).min(tile.height * 0.4);
                    let icon_rect = Rect {
                        x: tile.x + (tile.width - icon_sz) * 0.5,
                        y: tile.y + (tile.height - 36.0 - icon_sz).max(8.0) * 0.5,
                        width: icon_sz,
                        height: icon_sz,
                    };
                    self.paint_material_icon(icon_rect, icon, fg);
                }
                let caption_h = (if item.subtitle.is_some() { 48.0_f32 } else { 36.0 }).min(tile.height);
                let caption = Rect {
                    x: tile.x,
                    y: tile.y + tile.height - caption_h,
                    width: tile.width,
                    height: caption_h,
                };
                let (cap_fg, sub_fg) = if item.image.is_some() && options.show_caption_scrim {
                    self.paint_asym_rect(caption, 0.0, options.radius, Color(0, 0, 0, 148));
                    (Color(255, 255, 255, 255), Color(255, 255, 255, 200))
                } else {
                    (fg, if is_sel { t.on_primary_container.opacity(0.8) } else { t.on_surface_variant })
                };
                if let Some(sub) = item.subtitle {
                    self.styled_text(
                        Rect { x: caption.x + 12.0, y: caption.y + 6.0, width: (caption.width - 24.0).max(0.0), height: 18.0 },
                        item.label,
                        cap_fg,
                        TextStyle { size: 14.0, weight: 500, ..TextStyle::default() },
                    );
                    self.styled_text(
                        Rect { x: caption.x + 12.0, y: caption.y + 24.0, width: (caption.width - 24.0).max(0.0), height: 16.0 },
                        sub,
                        sub_fg,
                        TextStyle { size: 12.0, ..TextStyle::default() },
                    );
                } else {
                    self.styled_text(
                        Rect { x: caption.x + 12.0, width: (caption.width - 24.0).max(0.0), ..caption },
                        item.label,
                        cap_fg,
                        TextStyle { size: 14.0, weight: 500, align: if item.image.is_none() { 1 } else { 0 }, ..TextStyle::default() },
                    );
                }
                if is_sel {
                    self.commands.push(DrawCommand::Outline {
                        bounds: tile,
                        radius: options.radius,
                        width: 2.0,
                        color: t.primary,
                    });
                }
                self.state_layer(r, options.radius, fg);
                self.focus_ring(r);
                self.semantic(r, Role::Button, item.label, is_sel, None, None, None);
                if r.clicked {
                    clicked = Some(idx);
                }
            }
        }
        clicked
    }
}

#[cfg(test)]
#[path = "grid_list.spec.rs"]
mod tests;
