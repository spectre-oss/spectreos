//! Material ExpansionPanel and Accordion container with rotating chevron and split-corner shadow.
use crate::*;

/// Configuration for `Frame::expansion_panel`.
#[derive(Clone, Copy, Debug)]
pub struct ExpansionPanelOptions {
    pub round_top: bool,
    pub round_bottom: bool,
    pub enabled: bool,
    pub header_height: f32,
}

impl Default for ExpansionPanelOptions {
    fn default() -> Self {
        Self {
            round_top: true,
            round_bottom: true,
            enabled: true,
            header_height: 48.0,
        }
    }
}

fn expansion_panel_shadow(f: &mut Frame<'_>, panel: Rect, round_top: bool, round_bottom: bool) {
    if round_top == round_bottom {
        f.elevation(panel, if round_top { 12.0 } else { 0.0 }, 2);
        return;
    }
    let half = panel.height * 0.5;
    let margin = 24.0;
    let top = Rect {
        x: panel.x - margin,
        y: panel.y - margin,
        width: panel.width + 2.0 * margin,
        height: half + margin,
    };
    let bottom = Rect {
        x: panel.x - margin,
        y: panel.y + half,
        width: panel.width + 2.0 * margin,
        height: panel.height - half + margin,
    };
    f.within(top, |f| f.elevation(panel, if round_top { 12.0 } else { 0.0 }, 2));
    f.within(bottom, |f| f.elevation(panel, if round_bottom { 12.0 } else { 0.0 }, 2));
}

impl Frame<'_> {
    /// Material ExpansionPanel with animated collapse/expand height, rotating chevron, and
    /// independent top/bottom corner rounding for stacked accordions.
    pub fn expansion_panel(
        &mut self,
        id: u64,
        title: &str,
        description: &str,
        expanded: &mut bool,
        body_height: f32,
        options: ExpansionPanelOptions,
        body: impl FnOnce(&mut Self),
    ) -> Response {
        let t = self.theme();
        let progress = self
            .transition_css(id, 40, f32::from(*expanded), f32::from(*expanded), 0.225, timing::Easing::Standard)
            .clamp(0.0, 1.0);
        let header_h = options.header_height.max(40.0) + 16.0 * progress;
        let vis_body_h = (body_height.max(0.0) * progress).max(0.0);
        let total_h = header_h + vis_body_h;
        let panel = self.reserve(total_h);
        let top_r = if options.round_top { 12.0 } else { 0.0 };
        let bot_r = if options.round_bottom { 12.0 } else { 0.0 };
        expansion_panel_shadow(self, panel, options.round_top, options.round_bottom);
        self.paint_asym_rect(panel, top_r, bot_r, t.surface_low);

        let header = Rect { height: header_h, ..panel };
        let r = self.target(id, header, title, options.enabled, Role::Button);
        self.state_layer(r, top_r.max(bot_r), t.text);
        self.focus_ring(r);
        let mut resp = r;
        if r.clicked {
            *expanded = !*expanded;
            resp.changed = true;
        }
        let title_col = if r.enabled { t.text } else { t.text.opacity(0.38) };
        let title_rect = Rect {
            x: header.x + 20.0,
            width: (header.width - 56.0).max(0.0),
            ..header
        };
        if description.is_empty() {
            self.styled_text(title_rect, title, title_col, TextStyle { size: 15.0, weight: 500, ..TextStyle::default() });
        } else {
            let split = title_rect.width * 0.45;
            self.styled_text(
                Rect { width: split, ..title_rect },
                title,
                title_col,
                TextStyle { size: 15.0, weight: 500, ..TextStyle::default() },
            );
            self.styled_text(
                Rect { x: title_rect.x + split + 8.0, width: (title_rect.width - split - 8.0).max(0.0), ..title_rect },
                description,
                t.on_surface_variant.opacity(if r.enabled { 1.0 } else { 0.38 }),
                TextStyle { size: 14.0, ..TextStyle::default() },
            );
        }
        // Rotating chevron indicator (0 deg down -> 180 deg up)
        let cx = header.x + header.width - 24.0;
        let cy = header.y + header.height * 0.5;
        let flip = 1.0 - 2.0 * progress;
        self.paint_line([cx - 5.0, cy - 2.0 * flip], [cx, cy + 3.0 * flip], 1.8, t.on_surface_variant);
        self.paint_line([cx, cy + 3.0 * flip], [cx + 5.0, cy - 2.0 * flip], 1.8, t.on_surface_variant);

        if vis_body_h > 0.5 {
            let body_bounds = Rect {
                x: panel.x + 20.0,
                y: panel.y + header_h,
                width: (panel.width - 40.0).max(0.0),
                height: vis_body_h,
            };
            self.within(body_bounds, |f| {
                f.with_opacity(progress, body);
            });
        }
        resp
    }

    /// Stacked single- or multi-expand Material accordion over a slice of `(title, description)` panels.
    pub fn accordion(
        &mut self,
        id: u64,
        panels: &[(&str, &str)],
        expanded_mask: &mut u64,
        multi: bool,
        body_height: f32,
        mut build_body: impl FnMut(&mut Self, usize),
    ) {
        let count = panels.len();
        self.with_gap(1.0, |f| {
            for (i, &(title, desc)) in panels.iter().enumerate() {
                let bit = 1u64 << (i.min(63));
                let mut open = (*expanded_mask & bit) != 0;
                let prev_open = i > 0 && (*expanded_mask & (1u64 << (i - 1).min(63))) != 0;
                let next_open = i + 1 < count && (*expanded_mask & (1u64 << (i + 1).min(63))) != 0;
                let opts = ExpansionPanelOptions {
                    round_top: i == 0 || open || prev_open,
                    round_bottom: i + 1 == count || open || next_open,
                    ..ExpansionPanelOptions::default()
                };
                let resp = f.expansion_panel(id.wrapping_add(i as u64), title, desc, &mut open, body_height, opts, |f| {
                    build_body(f, i);
                });
                if resp.changed {
                    if open {
                        if multi {
                            *expanded_mask |= bit;
                        } else {
                            *expanded_mask = bit;
                        }
                    } else {
                        *expanded_mask &= !bit;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
#[path = "expansion_panel.spec.rs"]
mod tests;
