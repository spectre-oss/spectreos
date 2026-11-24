use crate::*;

impl Frame<'_> {
    pub fn divider(&mut self) {
        let b = self.allocate(1.0);
        self.divider_at(b, DividerOrientation::Horizontal, 0.0, 0.0);
    }
    /// Draw a 1dp decorative divider centered on its short axis. Insets are
    /// physical distances along the long axis; callers handle RTL placement.
    pub fn divider_at(
        &mut self,
        bounds: Rect,
        orientation: DividerOrientation,
        inset_start: f32,
        inset_end: f32,
    ) {
        self.divider_styled_at(bounds, DividerOptions {orientation,inset_start,inset_end,..DividerOptions::default()});
    }
    /// Draw a divider with logical horizontal insets and an explicit thickness.
    /// Vertical insets run from top to bottom regardless of layout direction.
    pub fn divider_styled_at(&mut self, bounds: Rect, options: DividerOptions) {
        let DividerOptions {orientation,inset_start,inset_end,thickness,rtl,color}=options;
        if !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width <= 0.0
            || bounds.height <= 0.0
            || !thickness.is_finite()
            || thickness <= 0.0
        {
            return;
        }
        let long = if orientation == DividerOrientation::Horizontal {
            bounds.width
        } else {
            bounds.height
        };
        let (inset_start,inset_end)=if rtl && orientation==DividerOrientation::Horizontal {(inset_end,inset_start)} else {(inset_start,inset_end)};
        let start = if inset_start.is_finite() {
            inset_start.clamp(0.0, long)
        } else {
            0.0
        };
        let end = if inset_end.is_finite() {
            inset_end.clamp(0.0, long)
        } else {
            0.0
        };
        let length = (long - start - end).max(0.0);
        if length == 0.0 {
            return;
        }
        let line = match orientation {
            DividerOrientation::Horizontal => Rect {
                x: bounds.x + start,
                y: bounds.y + (bounds.height - bounds.height.min(thickness)) / 2.0,
                width: length,
                height: bounds.height.min(thickness),
            },
            DividerOrientation::Vertical => Rect {
                x: bounds.x + (bounds.width - bounds.width.min(thickness)) / 2.0,
                y: bounds.y + start,
                width: bounds.width.min(thickness),
                height: length,
            },
        };
        self.rect(line, 0.0, color.unwrap_or(self.ui.theme.outline_variant));
    }
}

#[cfg(test)]
#[path = "divider.spec.rs"]
mod tests;
