//! Hierarchical Breadcrumb trail widget.
use crate::*;

impl Frame<'_> {
    /// Horizontal clickable breadcrumb trail (`Home › Settings › Display`).
    /// Returns the clicked segment index when any ancestor or leaf is clicked.
    pub fn breadcrumb(&mut self, id: u64, segments: &[&str], active: Option<usize>) -> Option<usize> {
        if segments.is_empty() {
            return None;
        }
        let t = self.theme();
        let bar = self.reserve(36.0);
        let active_idx = active.unwrap_or(segments.len() - 1).min(segments.len() - 1);
        let mut x = bar.x;
        let max_x = bar.x + bar.width;
        let mut clicked = None;

        for (i, &seg) in segments.iter().enumerate() {
            if i > 0 {
                let sep_w = 18.0;
                if x + sep_w > max_x {
                    break;
                }
                self.styled_text(
                    Rect { x, y: bar.y, width: sep_w, height: bar.height },
                    "›",
                    t.on_surface_variant,
                    TextStyle { size: 14.0, align: 1, ..TextStyle::default() },
                );
                x += sep_w;
            }
            let is_active = i == active_idx;
            let style = TextStyle {
                size: 14.0,
                weight: if is_active { 600 } else { 500 },
                align: 1,
                ..TextStyle::default()
            };
            let w = (self.text_width(seg, style) + 16.0).min((max_x - x).max(0.0));
            if w <= 1.0 {
                break;
            }
            let rect = Rect { x, y: bar.y, width: w, height: bar.height };
            let r = self.target(id.wrapping_add(i as u64), rect, seg, true, Role::Link);
            self.state_layer(r, 8.0, t.primary);
            self.focus_ring(r);
            self.styled_text(rect, seg, if is_active { t.text } else { t.primary }, style);
            if r.clicked {
                clicked = Some(i);
            }
            x += w;
        }
        clicked
    }
}

#[cfg(test)]
#[path = "breadcrumb.spec.rs"]
mod tests;
