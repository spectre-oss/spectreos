//! Material Hue ColorPicker and Circular SwatchPicker widgets.
use crate::*;

/// Convert a hue angle in degrees (`0.0..=360.0`) to a saturated opaque RGB `Color`.
pub fn hue_to_color(degrees: f32) -> Color {
    let deg = degrees.rem_euclid(360.0);
    let h = deg / 60.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    Color((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255)
}

impl Frame<'_> {
    /// Hue-based color picker combining a 0..=360 slider and live RGB swatch preview.
    /// Returns the currently selected `Color`.
    pub fn color_picker(&mut self, id: u64, label: &str, hue_degrees: &mut f32, enabled: bool) -> Color {
        *hue_degrees = hue_degrees.clamp(0.0, 360.0);
        self.slider(id, label, hue_degrees, 0.0..=360.0, 1.0, enabled);
        let color = hue_to_color(*hue_degrees);
        let swatch = self.reserve(40.0);
        self.paint_rect(swatch, 12.0, color);
        self.styled_text(
            swatch,
            &format!("#{:02X}{:02X}{:02X}", color.0, color.1, color.2),
            Color(255, 255, 255, 255),
            TextStyle { size: 13.0, weight: 600, align: 1, ..TextStyle::default() },
        );
        color
    }

    /// Row/grid of circular color swatch cards with active checkmark indicator.
    /// Returns `true` when `*selected` changed.
    pub fn swatch_picker(
        &mut self,
        id: u64,
        swatches: &[(&str, Color)],
        selected: &mut usize,
        columns: usize,
        enabled: bool,
    ) -> bool {
        if swatches.is_empty() {
            return false;
        }
        *selected = (*selected).min(swatches.len() - 1);
        let cols = columns.max(1);
        let rows = swatches.len().div_ceil(cols);
        let t = self.theme();
        let mut changed = false;

        for row in 0..rows {
            let b = self.reserve(64.0);
            let gap = 8.0;
            let cell_w = ((b.width - gap * (cols.saturating_sub(1)) as f32) / cols as f32).max(0.0);
            for col in 0..cols {
                let idx = row * cols + col;
                let Some(&(name, color)) = swatches.get(idx) else {
                    break;
                };
                let card = Rect { x: b.x + col as f32 * (cell_w + gap), y: b.y, width: cell_w, height: b.height };
                let is_sel = idx == *selected;
                let r = self.target(id.wrapping_add(idx as u64), card, name, enabled, Role::Radio);
                self.paint_rect(card, 16.0, if is_sel { t.surface_highest } else { t.surface_low });
                self.state_layer(r, 16.0, t.text);
                self.focus_ring(r);
                let sz = 36.0_f32.min(card.width - 8.0).min(card.height - 8.0).max(0.0);
                let dot = Rect {
                    x: card.x + (card.width - sz) * 0.5,
                    y: card.y + (card.height - sz) * 0.5,
                    width: sz,
                    height: sz,
                };
                self.paint_rect(dot, sz * 0.5, color);
                if is_sel {
                    let cx = dot.x + dot.width * 0.5;
                    let cy = dot.y + dot.height * 0.5;
                    let lum = 0.299 * color.0 as f32 + 0.587 * color.1 as f32 + 0.114 * color.2 as f32;
                    let check = if lum > 150.0 { Color(20, 20, 24, 255) } else { Color(255, 255, 255, 255) };
                    self.paint_line([cx - 5.0, cy], [cx - 1.5, cy + 3.5], 2.2, check);
                    self.paint_line([cx - 1.5, cy + 3.5], [cx + 5.5, cy - 4.0], 2.2, check);
                }
                if r.clicked && *selected != idx {
                    *selected = idx;
                    changed = true;
                }
            }
        }
        changed
    }
}

#[cfg(test)]
#[path = "color_picker.spec.rs"]
mod tests;
