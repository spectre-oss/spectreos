//! Lightweight transient floating Toast pill with deadline auto-dismiss.
use crate::*;

/// Caller-owned state for `Frame::toast`.
#[derive(Clone, Debug, Default)]
pub struct ToastState {
    pub message: Option<String>,
    pub expires_at: f64,
}

impl ToastState {
    /// Show `message` for `duration_s` seconds starting at `now`.
    pub fn show(&mut self, message: impl Into<String>, now: f64, duration_s: f64) {
        self.message = Some(message.into());
        self.expires_at = now + duration_s.max(0.1);
    }

    pub fn dismiss(&mut self) {
        self.message = None;
    }

    pub fn is_visible(&self, now: f64) -> bool {
        self.message.is_some() && now < self.expires_at
    }
}

/// Visual options for `Frame::toast_with_options`.
#[derive(Clone, Copy, Debug)]
pub struct ToastOptions {
    pub background: Option<Color>,
    pub foreground: Option<Color>,
    pub corner_radius: f32,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            background: None,
            foreground: None,
            corner_radius: 20.0,
        }
    }
}

impl Frame<'_> {
    /// Paint an auto-sized centered floating Toast pill near the bottom of `container`
    /// and schedule `wake_at` for dismissal. Clears `state.message` once expired.
    pub fn toast(&mut self, state: &mut ToastState, container: Rect, bottom_margin: f32) {
        self.toast_with_options(state, container, bottom_margin, ToastOptions::default());
    }

    /// Paint a floating Toast pill with optional custom background/foreground tint and radius.
    pub fn toast_with_options(
        &mut self,
        state: &mut ToastState,
        container: Rect,
        bottom_margin: f32,
        options: ToastOptions,
    ) {
        let now = self.time();
        let Some(ref msg) = state.message else {
            return;
        };
        if now >= state.expires_at {
            state.message = None;
            return;
        }
        self.wake_at(state.expires_at);
        let t = self.theme();
        let bg = options.background.unwrap_or(t.inverse_surface);
        let fg = options.foreground.unwrap_or(t.inverse_on_surface);
        let radius = options.corner_radius.max(0.0);
        let style = TextStyle { size: 13.0, weight: 500, align: 1, ..TextStyle::default() };
        let max_w = (container.width - 32.0).max(80.0);
        let pill_w = (self.text_width(msg, style) + 36.0).clamp(96.0, max_w);
        let pill_h = 40.0;
        let rect = Rect {
            x: container.x + (container.width - pill_w) * 0.5,
            y: container.y + container.height - bottom_margin - pill_h,
            width: pill_w,
            height: pill_h,
        };
        self.elevation(rect, radius, 3);
        self.paint_rect(rect, radius, bg);
        self.styled_text(rect, msg, fg, style);
    }
}

#[cfg(test)]
#[path = "toast.spec.rs"]
mod tests;
