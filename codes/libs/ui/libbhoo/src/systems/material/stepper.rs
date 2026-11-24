//! Material horizontal linear Stepper widget.
use crate::*;

/// Caller-owned state for `Frame::stepper`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepperState {
    pub active: usize,
    pub completed_mask: u64,
}

impl StepperState {
    pub fn is_completed(&self, index: usize) -> bool {
        index < 64 && (self.completed_mask & (1u64 << index)) != 0
    }

    pub fn set_completed(&mut self, index: usize, done: bool) {
        if index < 64 {
            if done {
                self.completed_mask |= 1u64 << index;
            } else {
                self.completed_mask &= !(1u64 << index);
            }
        }
    }
}

impl Frame<'_> {
    /// Material horizontal stepper with step circles, completed checkmarks, connector lines,
    /// and a scoped body closure for the active step. Returns `true` when `state.active` changed.
    pub fn stepper(
        &mut self,
        id: u64,
        steps: &[&str],
        state: &mut StepperState,
        linear: bool,
        body_height: f32,
        body: impl FnOnce(&mut Self, usize),
    ) -> bool {
        if steps.is_empty() {
            return false;
        }
        state.active = state.active.min(steps.len() - 1);
        let t = self.theme();
        let mut changed = false;
        let header = self.reserve(72.0);
        let n = steps.len();
        let slot_w = header.width / n as f32;
        for (i, &label) in steps.iter().enumerate() {
            let active = i == state.active;
            let done = state.is_completed(i);
            let allowed = !linear || i <= state.active || state.is_completed(i.saturating_sub(1));
            let cell = Rect { x: header.x + i as f32 * slot_w, width: slot_w, ..header };
            let r = self.target(id.wrapping_add(i as u64), cell, label, allowed, Role::Tab);
            self.state_layer(r, 8.0, t.on_surface_variant);
            self.focus_ring(r);
            if r.clicked && allowed && state.active != i {
                state.active = i;
                changed = true;
            }
            let badge = Rect { x: cell.x + 12.0, y: cell.y + (cell.height - 24.0) * 0.5, width: 24.0, height: 24.0 };
            let badge_bg = if active || done { t.primary } else { t.surface_highest };
            let badge_fg = if active || done { t.on_primary } else { t.on_surface_variant };
            self.paint_rect(badge, 12.0, if allowed { badge_bg } else { t.text.opacity(0.12) });
            if done {
                let cx = badge.x + 12.0;
                let cy = badge.y + 12.0;
                self.paint_line([cx - 4.5, cy], [cx - 1.0, cy + 3.5], 2.0, badge_fg);
                self.paint_line([cx - 1.0, cy + 3.5], [cx + 5.0, cy - 3.5], 2.0, badge_fg);
            } else {
                let num = (i + 1).to_string();
                self.styled_text(badge, &num, badge_fg, TextStyle { size: 12.0, weight: 500, align: 1, ..TextStyle::default() });
            }
            let text_x = badge.x + 32.0;
            let text_w = (cell.x + cell.width - text_x - 16.0).max(0.0);
            self.styled_text(
                Rect { x: text_x, width: text_w, ..cell },
                label,
                if !allowed {
                    t.text.opacity(0.38)
                } else if active {
                    t.text
                } else {
                    t.on_surface_variant
                },
                TextStyle { size: 14.0, weight: if active { 500 } else { 400 }, ..TextStyle::default() },
            );
            if i + 1 < n {
                let lw = self.text_width(label, TextStyle { size: 14.0, weight: 500, ..TextStyle::default() });
                let line_x0 = (text_x + lw + 10.0).min(cell.x + cell.width - 4.0);
                let line_x1 = header.x + (i + 1) as f32 * slot_w + 6.0;
                if line_x1 > line_x0 + 4.0 {
                    let ly = header.y + header.height * 0.5;
                    self.paint_line([line_x0, ly], [line_x1, ly], 1.0, t.outline_variant);
                }
            }
        }
        if body_height > 0.0 {
            let area = self.reserve(body_height);
            self.within(area, |f| body(f, state.active));
        }
        changed
    }
}

#[cfg(test)]
#[path = "stepper.spec.rs"]
mod tests;
