//! Resizable two-pane SplitPane container with pointer & keyboard drag divider.
use crate::*;

/// Orientation of a `SplitPane` divider split.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitPaneOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// Configuration for `Frame::split_pane`.
#[derive(Clone, Copy, Debug)]
pub struct SplitPaneOptions {
    pub orientation: SplitPaneOrientation,
    pub min_ratio: f32,
    pub max_ratio: f32,
    pub handle_thickness: f32,
    pub enabled: bool,
}

impl Default for SplitPaneOptions {
    fn default() -> Self {
        Self {
            orientation: SplitPaneOrientation::Horizontal,
            min_ratio: 0.2,
            max_ratio: 0.8,
            handle_thickness: 12.0,
            enabled: true,
        }
    }
}

impl Frame<'_> {
    /// Two-pane resizable split container with pointer-draggable and arrow-key-adjustable divider.
    /// `ratio` is clamped to `options.min_ratio..=options.max_ratio`.
    pub fn split_pane(
        &mut self,
        id: u64,
        height: f32,
        ratio: &mut f32,
        options: SplitPaneOptions,
        first: impl FnOnce(&mut Self),
        second: impl FnOnce(&mut Self),
    ) -> Response {
        let t = self.theme();
        let bounds = self.reserve(height.max(0.0));
        let lo = options.min_ratio.clamp(0.05, 0.95);
        let hi = options.max_ratio.clamp(lo, 0.95);
        *ratio = ratio.clamp(lo, hi);
        let handle_t = options.handle_thickness.max(4.0);

        let (first_rect, handle_rect, second_rect) = match options.orientation {
            SplitPaneOrientation::Horizontal => {
                let usable = (bounds.width - handle_t).max(0.0);
                let first_w = usable * *ratio;
                let second_w = (usable - first_w).max(0.0);
                (
                    Rect { width: first_w, ..bounds },
                    Rect { x: bounds.x + first_w, width: handle_t, ..bounds },
                    Rect { x: bounds.x + first_w + handle_t, width: second_w, ..bounds },
                )
            }
            SplitPaneOrientation::Vertical => {
                let usable = (bounds.height - handle_t).max(0.0);
                let first_h = usable * *ratio;
                let second_h = (usable - first_h).max(0.0);
                (
                    Rect { height: first_h, ..bounds },
                    Rect { y: bounds.y + first_h, height: handle_t, ..bounds },
                    Rect { y: bounds.y + first_h + handle_t, height: second_h, ..bounds },
                )
            }
        };

        self.within(first_rect, first);
        let mut pct = *ratio * 100.0;
        let vertical_drag = options.orientation == SplitPaneOrientation::Vertical;
        let mut r = self.drag_handle(
            id,
            handle_rect,
            bounds,
            "Resize split pane",
            &mut pct,
            lo * 100.0..=hi * 100.0,
            vertical_drag,
        );
        let next_ratio = (pct / 100.0).clamp(lo, hi);
        if (next_ratio - *ratio).abs() > 1e-4 {
            *ratio = next_ratio;
            r.changed = true;
        }
        let pill_color = if r.pressed {
            t.primary
        } else if r.hovered {
            t.on_surface_variant
        } else {
            t.outline_variant
        };
        let pill = match options.orientation {
            SplitPaneOrientation::Horizontal => Rect {
                x: handle_rect.x + (handle_rect.width - 4.0) * 0.5,
                y: handle_rect.y + (handle_rect.height - 32.0).max(0.0) * 0.5,
                width: 4.0,
                height: 32.0_f32.min(handle_rect.height),
            },
            SplitPaneOrientation::Vertical => Rect {
                x: handle_rect.x + (handle_rect.width - 32.0).max(0.0) * 0.5,
                y: handle_rect.y + (handle_rect.height - 4.0) * 0.5,
                width: 32.0_f32.min(handle_rect.width),
                height: 4.0,
            },
        };
        self.paint_rect(pill, 2.0, pill_color);
        self.within(second_rect, second);
        r
    }
}

#[cfg(test)]
#[path = "split_pane.spec.rs"]
mod tests;
