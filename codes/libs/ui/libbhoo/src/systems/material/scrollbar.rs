use crate::*;

/// Pointer drag state for a scrollbar thumb.
#[derive(Default)]
pub struct ScrollbarDragState {
    anchor: Option<(f32, f32)>,
}

/// Direction of a non-interactive scroll position indicator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarOrientation { Vertical, Horizontal }

/// Caller-owned fade state for a scroll indicator.
#[derive(Default)]
pub struct ScrollbarIndicatorState {
    last_offset: Option<f32>,
    last_scroll: Option<f64>,
}

impl Frame<'_> {
    /// Draw the default Compose Material 3 non-interactive scrollbar indicator.
    /// Geometry and timing informed by AndroidX Scrollbar.kt (Apache-2.0),
    /// revision 1448d64974931a04c9fba96ac630cbd6b96cda36.
    /// Returns its painted bounds when visible. Scroll offsets are clamped to the content range.
    #[allow(clippy::too_many_arguments)]
    pub fn scroll_indicator(
        &mut self, viewport: Rect, content_size: f32, offset: f32,
        orientation: ScrollbarOrientation, rtl: bool, fade: bool,
        state: &mut ScrollbarIndicatorState,
    ) -> Option<Rect> {
        let size = if orientation == ScrollbarOrientation::Vertical { viewport.height } else { viewport.width };
        if !size.is_finite() || !content_size.is_finite() || !offset.is_finite() || size <= 0.0 || content_size <= size {
            state.last_offset = None; state.last_scroll = None; return None;
        }
        let track = size - 4.0;
        if track < 24.0 { return None; }
        let value = offset.clamp(0.0, content_size - size);
        let now = self.time();
        if state.last_offset.is_some_and(|last| (last - value).abs() > 0.001) {
            state.last_scroll = Some(now);
        }
        state.last_offset = Some(value);
        let alpha = if !fade { 1.0 } else if let Some(last) = state.last_scroll {
            let elapsed = (now - last).max(0.0);
            if elapsed < 0.4 {
                if viewport.intersect(self.clip).width > 0.0 && viewport.intersect(self.clip).height > 0.0 { self.wake_at(last + 0.4); }
                1.0
            } else if self.reduced_motion() { 0.0 }
            else if elapsed < 0.65 {
                self.continuous(viewport);
                1.0 - crate::timing::Easing::Cubic([0.4, 0.0, 0.2, 1.0]).sample(((elapsed - 0.4) / 0.25) as f32)
            } else { 0.0 }
        } else { 0.0 };
        if alpha <= 0.001 { return None; }
        let length = (track * size / content_size).max(24.0).min((track * 0.9).max(24.0));
        let distance = value / (content_size - size) * (track - length);
        let bounds = match orientation {
            ScrollbarOrientation::Vertical => Rect { x: if rtl { viewport.x } else { viewport.x + viewport.width - 4.0 }, y: viewport.y + 2.0 + distance, width: 4.0, height: length },
            ScrollbarOrientation::Horizontal => Rect { x: if rtl { viewport.x + viewport.width - 2.0 - length - distance } else { viewport.x + 2.0 + distance }, y: viewport.y + viewport.height - 4.0, width: length, height: 4.0 },
        };
        self.paint_rect(bounds, 2.0, self.theme().outline.opacity(0.7 * alpha));
        Some(bounds)
    }
    /// Vertical scrollbar with an anchored thumb drag and page-sized track clicks.
    /// `content_height` and `viewport_height` are measured in the same units as `offset`.
    pub fn vertical_scrollbar(
        &mut self,
        id: u64,
        track: Rect,
        label: &str,
        content_height: f32,
        viewport_height: f32,
        offset: &mut f32,
        state: &mut ScrollbarDragState,
    ) -> (Response, Rect) {
        let content = if content_height.is_finite() { content_height.max(0.0) } else { 0.0 };
        let viewport = if viewport_height.is_finite() { viewport_height.max(0.0) } else { 0.0 };
        let maximum = (content - viewport).max(0.0);
        *offset = if offset.is_finite() { offset.clamp(0.0, maximum) } else { 0.0 };
        let height = track.height.max(0.0);
        let thumb_height = if content > 0.0 { (height * viewport / content).max(32.0).min(height) } else { height };
        let travel = (height - thumb_height).max(0.0);
        let thumb_at = |value: f32| Rect { y: track.y + if maximum > 0.0 { value / maximum * travel } else { 0.0 }, height: thumb_height, ..track };
        let before = *offset;
        let mut response = self.interact(id, track, maximum > 0.0 && travel > 0.0);
        if response.enabled && self.input.pressed && response.pressed {
            if let Some(point) = self.input.pointer {
                let thumb = thumb_at(*offset);
                if thumb.contains(point) { state.anchor = Some((point[1], *offset)); }
                else {
                    state.anchor = None;
                    let amount = if point[1] < thumb.y { -viewport } else { viewport };
                    *offset = (*offset + amount).clamp(0.0, maximum);
                }
            }
        }
        if response.pressed && let (Some((start, initial)), Some(point)) = (state.anchor, self.input.pointer) {
            *offset = (initial + (point[1] - start) * maximum / travel).clamp(0.0, maximum);
        }
        if self.input.released || self.input.cancelled || self.input.blurred || !response.enabled { state.anchor = None; }
        if let Some(key) = self.key_for(response) {
            *offset = match key {
                Key::Up | Key::Left => (*offset - 40.0).max(0.0),
                Key::Down | Key::Right => (*offset + 40.0).min(maximum),
                Key::PageUp => (*offset - viewport).max(0.0),
                Key::PageDown => (*offset + viewport).min(maximum),
                Key::Home => 0.0,
                Key::End => maximum,
                _ => *offset,
            };
        }
        response.changed = before != *offset;
        self.semantic(response, Role::Slider, label, false, None, Some(*offset), Some([0.0, maximum]));
        self.focus_ring(response);
        (response, thumb_at(*offset))
    }
    /// Drag a vertical scrollbar thumb without changing its appearance.
    pub fn scrollbar_thumb_drag(
        &mut self,
        id: u64,
        thumb: Rect,
        track: Rect,
        max_offset: f32,
        offset: &mut f32,
        state: &mut ScrollbarDragState,
    ) {
        let travel = (track.height - thumb.height).max(0.0);
        if !max_offset.is_finite() || max_offset <= 0.0 || travel <= 0.0 {
            state.anchor = None;
            return;
        }
        let response = self.interact(id, thumb, true);
        if self.input.pressed && response.pressed {
            state.anchor = self.input.pointer.map(|p| (p[1], *offset));
        }
        if response.pressed && let (Some((start, initial)), Some(point)) = (state.anchor, self.input.pointer) {
            *offset = (initial + (point[1] - start) * max_offset / travel).clamp(0.0, max_offset);
            self.ui.continuous = true;
        }
        if self.input.released || self.input.cancelled || self.input.blurred || !response.enabled {
            state.anchor = None;
        }
    }

    /// Self-painting interactive vertical scrollbar (computes drag/keyboard state, cancels
    /// scroll inertia on grab, and paints track + rounded thumb).
    pub fn vertical_scrollbar_painted(
        &mut self,
        id: u64,
        track: Rect,
        offset: &mut f32,
        viewport: f32,
        content: f32,
        state: &mut ScrollbarDragState,
    ) -> Response {
        let (r, thumb) = self.vertical_scrollbar(id, track, "Scroll", content, viewport, offset, state);
        if content <= viewport + 0.5 || track.width <= 0.0 || track.height <= 0.0 {
            return r;
        }
        if r.pressed || r.changed {
            self.cancel_scroll(id);
        }
        let t = self.theme();
        self.paint_rect(track, track.width * 0.5, t.surface_variant.opacity(0.45));
        let thumb_color = if r.pressed {
            t.primary
        } else if r.hovered {
            t.on_surface_variant.opacity(0.8)
        } else {
            t.on_surface_variant.opacity(0.55)
        };
        self.paint_rect(thumb, thumb.width * 0.5, thumb_color);
        r
    }
}

#[cfg(test)]
#[path = "scrollbar.spec.rs"]
mod tests;
