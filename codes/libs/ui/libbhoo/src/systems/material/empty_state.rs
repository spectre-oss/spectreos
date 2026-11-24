//! Empty and error state panel with optional leading icon and primary CTA button.
use crate::*;

/// Configuration for `Frame::empty_state`.
#[derive(Clone, Copy, Debug, Default)]
pub struct EmptyStateOptions<'a> {
    pub icon: Option<MaterialIcon>,
    pub action_label: Option<&'a str>,
    pub error: bool,
}

impl Frame<'_> {
    /// Centered empty or error state panel with heading, supporting message, optional icon,
    /// and optional action button. Returns `Some(Response)` when an action button is shown.
    pub fn empty_state(
        &mut self,
        id: u64,
        title: &str,
        message: &str,
        options: EmptyStateOptions<'_>,
    ) -> Option<Response> {
        let t = self.theme();
        let has_icon = options.icon.is_some();
        let has_action = options.action_label.is_some();
        let height = 96.0 + if has_icon { 48.0 } else { 0.0 } + if has_action { 56.0 } else { 0.0 };
        let b = self.reserve(height);
        self.paint_rect(b, 16.0, t.surface_low);

        let mut y = b.y + 18.0;
        if let Some(icon) = options.icon {
            let icon_rect = Rect { x: b.x + (b.width - 36.0) * 0.5, y, width: 36.0, height: 36.0 };
            let col = if options.error { t.error } else { t.on_surface_variant };
            self.paint_material_icon(icon_rect, icon, col);
            y += 44.0;
        }
        self.styled_text(
            Rect { x: b.x + 16.0, y, width: (b.width - 32.0).max(0.0), height: 28.0 },
            title,
            if options.error { t.error } else { t.text },
            TextStyle { size: 18.0, weight: 600, align: 1, ..TextStyle::default() },
        );
        y += 30.0;
        if !message.is_empty() {
            self.styled_text(
                Rect { x: b.x + 20.0, y, width: (b.width - 40.0).max(0.0), height: 24.0 },
                message,
                t.on_surface_variant,
                TextStyle { size: 14.0, align: 1, ..TextStyle::default() },
            );
            y += 34.0;
        }
        options.action_label.map(|label| {
            let bw = (self.text_width(label, TextStyle::LABEL) + 40.0).clamp(112.0, (b.width - 32.0).max(112.0));
            let btn = Rect { x: b.x + (b.width - bw) * 0.5, y, width: bw, height: 40.0 };
            self.action_at(
                id,
                btn,
                label,
                Options { variant: ButtonVariant::Filled, ..Options::default() },
            )
        })
    }
}

#[cfg(test)]
#[path = "empty_state.spec.rs"]
mod tests;
