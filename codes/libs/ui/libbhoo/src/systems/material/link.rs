use crate::*;

impl Frame<'_> {
    /// Text link; the caller handles navigation after activation.
    pub fn link(&mut self, id: u64, label: &str, enabled: bool) -> Response {
        let row = self.reserve(32.0);
        let style = TextStyle {
            size: 16.0,
            ..TextStyle::default()
        };
        let bounds = Rect {
            width: self.text_width(label, style).min(row.width),
            ..row
        };
        let r = self.target(id, bounds, label, enabled, Role::Link);
        let color = self
            .ui
            .theme
            .primary
            .opacity(if r.enabled { 1.0 } else { 0.38 });
        self.styled_text(bounds, label, color, style);
        self.paint_rect(
            Rect {
                x: bounds.x,
                y: bounds.y + 25.0,
                width: bounds.width,
                height: 1.0,
            },
            0.0,
            color.opacity(if r.hovered || r.focused { 1.0 } else { 0.4 }),
        );
        self.focus_ring(r);
        r
    }
}

#[cfg(test)]
#[path = "link.spec.rs"]
mod tests;
