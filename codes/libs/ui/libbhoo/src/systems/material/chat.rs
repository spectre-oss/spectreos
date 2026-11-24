//! Material chat message bubbles and docked message composer bar.
use crate::*;

/// One chat message for `Frame::chat_bubble`.
#[derive(Clone, Copy, Debug)]
pub struct ChatMessage<'a> {
    pub body: &'a str,
    pub timestamp: &'a str,
    pub outgoing: bool,
}

impl Frame<'_> {
    /// Render one incoming or outgoing Material chat message bubble with wrapped body text
    /// and optional timestamp.
    pub fn chat_bubble(&mut self, message: ChatMessage<'_>) {
        let t = self.theme();
        let avail = self.available();
        let max_w = (avail.width * 0.78).clamp(120.0, 460.0);
        let style = TextStyle { size: 14.0, ..TextStyle::default() };
        let inner_w = (max_w - 28.0).max(40.0);
        let lines = self.ui.text_metrics.lines(message.body, inner_w, style, |_| {}).max(1);
        let has_ts = !message.timestamp.is_empty();
        let bubble_h = lines as f32 * 22.0 + 20.0 + if has_ts { 16.0 } else { 0.0 };
        let row = self.reserve(bubble_h);
        let content_w = if lines == 1 {
            (self.text_width(message.body, style) + 32.0).clamp(72.0, max_w)
        } else {
            max_w
        };
        let bubble = Rect {
            x: if message.outgoing { row.x + row.width - content_w } else { row.x },
            y: row.y,
            width: content_w,
            height: row.height,
        };
        let (bg, fg) = if message.outgoing {
            (t.primary_container, t.on_primary_container)
        } else {
            (t.surface_high, t.text)
        };
        self.paint_rect(bubble, 16.0, bg);
        let text_area = Rect {
            x: bubble.x + 14.0,
            y: bubble.y + 10.0,
            width: (bubble.width - 28.0).max(0.0),
            height: (bubble.height - 20.0).max(0.0),
        };
        self.within(text_area, |f| {
            f.paragraph(text_area, message.body, fg, style, 22.0);
        });
        if has_ts {
            self.styled_text(
                Rect {
                    x: bubble.x + 14.0,
                    y: bubble.y + bubble.height - 18.0,
                    width: (bubble.width - 28.0).max(0.0),
                    height: 14.0,
                },
                message.timestamp,
                fg.opacity(0.65),
                TextStyle { size: 11.0, align: if message.outgoing { 2 } else { 0 }, ..TextStyle::default() },
            );
        }
    }

    /// Docked message composer bar combining a text input field and send icon button.
    /// Returns `(field_response, send_clicked)`.
    pub fn message_composer(
        &mut self,
        id: u64,
        bounds: Rect,
        placeholder: &str,
        draft: &str,
        enabled: bool,
    ) -> (Response, bool) {
        let t = self.theme();
        self.paint_rect(bounds, 0.0, t.surface_low);
        self.paint_rect(Rect { height: 1.0, ..bounds }, 0.0, t.outline_variant);
        let pad = 12.0;
        let send_sz = 44.0;
        let field_rect = Rect {
            x: bounds.x + pad,
            y: bounds.y + (bounds.height - 56.0).max(0.0) * 0.5,
            width: (bounds.width - pad * 3.0 - send_sz).max(0.0),
            height: 56.0_f32.min(bounds.height),
        };
        let field_resp = self.text_field_at(id, field_rect, placeholder, draft, false, true);
        let send_rect = Rect {
            x: bounds.x + bounds.width - pad - send_sz,
            y: bounds.y + (bounds.height - send_sz) * 0.5,
            width: send_sz,
            height: send_sz,
        };
        let can_send = enabled && !draft.trim().is_empty();
        let send_resp = self.material_icon_button_at(
            id.wrapping_add(1),
            send_rect,
            MaterialIcon::Send,
            "Send message",
            Options {
                enabled: can_send,
                variant: ButtonVariant::Filled,
                ..Options::default()
            },
        );
        (field_resp, send_resp.clicked)
    }
}

#[cfg(test)]
#[path = "chat.spec.rs"]
mod tests;
