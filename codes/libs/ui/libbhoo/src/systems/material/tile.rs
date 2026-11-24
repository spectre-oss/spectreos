//! Material 3 Expressive two-line pill Action Tile (Quick Settings / control tile).
use crate::*;

/// Configuration for `Frame::action_tile`.
#[derive(Clone, Copy, Debug)]
pub struct ActionTileOptions {
    pub active: bool,
    pub enabled: bool,
    pub show_chevron: bool,
    /// Optional 0..1 collapse factor for the subtitle (`1.0` = visible, `0.0` = centered single-line title).
    pub subtitle_alpha: f32,
}

impl Default for ActionTileOptions {
    fn default() -> Self {
        Self {
            active: false,
            enabled: true,
            show_chevron: false,
            subtitle_alpha: 1.0,
        }
    }
}

impl Frame<'_> {
    /// Two-line Material 3 pill toggle/action tile (Quick Settings style) at explicit `bounds`
    /// with leading `SystemIcon`, title, collapsible subtitle, and optional trailing chevron.
    pub fn action_tile(
        &mut self,
        id: u64,
        bounds: Rect,
        icon: SystemIcon,
        title: &str,
        subtitle: &str,
        options: ActionTileOptions,
    ) -> Response {
        let t = self.theme();
        let rad = (bounds.height * 0.42).min(bounds.width * 0.5);
        let r = self.target(id, bounds, title, options.enabled, Role::ToggleButton);
        let (bg, fg, sub_fg) = if !r.enabled {
            (t.surface_low, t.text.opacity(0.38), t.on_surface_variant.opacity(0.38))
        } else if options.active {
            (t.primary, t.on_primary, t.on_primary.opacity(0.82))
        } else {
            (t.surface_high, t.text, t.on_surface_variant)
        };
        self.paint_rect(bounds, rad, bg);
        self.state_layer(r, rad, fg);
        self.focus_ring(r);

        let icon_sz = 22.0_f32.min(bounds.height - 12.0).max(0.0);
        let icon_rect = Rect {
            x: bounds.x + 14.0,
            y: bounds.y + (bounds.height - icon_sz) * 0.5,
            width: icon_sz,
            height: icon_sz,
        };
        self.paint_system_icon(icon_rect, icon, fg);

        let trail_pad = if options.show_chevron { 28.0 } else { 12.0 };
        let text_x = icon_rect.x + icon_sz + 10.0;
        let text_w = (bounds.x + bounds.width - text_x - trail_pad).max(0.0);
        let sa = if subtitle.is_empty() { 0.0 } else { options.subtitle_alpha.clamp(0.0, 1.0) };
        let title_y = bounds.y + (bounds.height - 18.0) * 0.5 - 8.5 * sa;
        self.styled_text(
            Rect { x: text_x, y: title_y, width: text_w, height: 18.0 },
            title,
            fg,
            TextStyle { size: 13.5, weight: 600, ..TextStyle::default() },
        );
        if sa > 0.02 {
            self.styled_text(
                Rect { x: text_x, y: title_y + 18.0, width: text_w, height: 16.0 },
                subtitle,
                sub_fg.opacity(sa),
                TextStyle { size: 11.5, ..TextStyle::default() },
            );
        }
        if options.show_chevron {
            let cx = bounds.x + bounds.width - 18.0;
            let cy = bounds.y + bounds.height * 0.5;
            self.paint_line([cx - 2.5, cy - 4.0], [cx + 2.0, cy], 1.6, sub_fg);
            self.paint_line([cx + 2.0, cy], [cx - 2.5, cy + 4.0], 1.6, sub_fg);
        }
        r
    }
}

#[cfg(test)]
#[path = "tile.spec.rs"]
mod tests;
