//! Material Command Palette (inline or modal searchable command launcher with shortcut hints).
use crate::*;

/// One command entry displayed inside `Frame::command_palette` or `Frame::command_palette_inline`.
#[derive(Clone, Copy, Debug)]
pub struct CommandItem<'a> {
    pub label: &'a str,
    pub shortcut: Option<&'a str>,
    pub icon: Option<MaterialIcon>,
}

impl Frame<'_> {
    /// Inline searchable command list with filter field, shortcut badges, and Clear button.
    /// Returns the original index of the activated command, if any.
    pub fn command_palette_inline(
        &mut self,
        id: u64,
        query: &mut String,
        items: &[CommandItem<'_>],
        placeholder: &str,
    ) -> Option<usize> {
        let t = self.theme();
        let r = self.text_field(id, placeholder, 56.0, !query.is_empty(), false, false);
        if !query.is_empty() {
            self.styled_text(
                Rect { x: r.bounds.x + 16.0, width: (r.bounds.width - 32.0).max(0.0), ..r.bounds },
                query,
                t.text,
                TextStyle { size: 16.0, ..TextStyle::default() },
            );
        }
        let needle = query.to_lowercase();
        let mut chosen = None;
        self.with_gap(0.0, |f| {
            for (i, item) in items.iter().enumerate() {
                if !needle.is_empty() && !item.label.to_lowercase().contains(&needle) {
                    continue;
                }
                let row = f.reserve(48.0);
                let resp = f.target(id.wrapping_add(100 + i as u64), row, item.label, true, Role::MenuItem);
                f.state_layer(resp, 8.0, t.text);
                f.focus_ring(resp);
                let mut text_x = row.x + 16.0;
                if let Some(icon) = item.icon {
                    f.paint_material_icon(
                        Rect { x: row.x + 12.0, y: row.y + 12.0, width: 24.0, height: 24.0 },
                        icon,
                        t.on_surface_variant,
                    );
                    text_x = row.x + 48.0;
                }
                let right_pad = if let Some(sc) = item.shortcut {
                    let sc_w = (f.text_width(sc, TextStyle { size: 12.0, ..TextStyle::default() }) + 16.0).max(36.0);
                    let badge = Rect {
                        x: (row.x + row.width - sc_w - 12.0).max(text_x),
                        y: row.y + 11.0,
                        width: sc_w,
                        height: 26.0,
                    };
                    f.paint_rect(badge, 6.0, t.surface_high);
                    f.styled_text(
                        badge,
                        sc,
                        t.on_surface_variant,
                        TextStyle { size: 12.0, weight: 500, align: 1, ..TextStyle::default() },
                    );
                    sc_w + 24.0
                } else {
                    16.0
                };
                f.styled_text(
                    Rect { x: text_x, width: (row.x + row.width - text_x - right_pad).max(0.0), ..row },
                    item.label,
                    t.text,
                    TextStyle { size: 14.0, ..TextStyle::default() },
                );
                if resp.clicked {
                    chosen = Some(i);
                }
            }
        });
        if !query.is_empty() && self.button(id.wrapping_add(50), "Clear search").clicked {
            query.clear();
        }
        chosen
    }

    /// Modal Command Palette dialog overlay centered near the top of the viewport.
    /// Returns the activated command index (and closes `overlay`) when an item is chosen.
    pub fn command_palette(
        &mut self,
        id: u64,
        overlay: &mut OverlayState,
        query: &mut String,
        items: &[CommandItem<'_>],
        placeholder: &str,
    ) -> Option<usize> {
        let vp = self.viewport();
        let w = 520.0_f32.min((vp.width - 32.0).max(200.0));
        let h = (items.len() as f32 * 48.0 + 140.0).min((vp.height - 80.0).max(160.0));
        let panel = Rect {
            x: vp.x + (vp.width - w) * 0.5,
            y: vp.y + 48.0_f32.min((vp.height - h).max(0.0) * 0.5),
            width: w,
            height: h,
        };
        overlay.kind = OverlayKind::Dialog;
        overlay.dismiss_policy = OverlayDismissPolicy::EscapeOrOutside;
        let mut chosen = None;
        self.overlay(id, overlay, panel, true, |f, open| {
            let inner = f.available().inset(16.0);
            f.within(inner, |f| {
                if let Some(idx) = f.command_palette_inline(id.wrapping_add(1), query, items, placeholder) {
                    chosen = Some(idx);
                    *open = false;
                }
            });
        });
        chosen
    }
}

#[cfg(test)]
#[path = "command_palette.spec.rs"]
mod tests;
