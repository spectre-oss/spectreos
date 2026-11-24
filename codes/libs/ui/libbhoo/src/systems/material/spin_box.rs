//! Numeric SpinBox (bounded value with decrement/increment buttons and keyboard arrows).
use crate::*;

impl Frame<'_> {
    /// Bounded numeric SpinBox with `-` / `+` buttons and arrow-key stepping.
    /// Returns `true` when `*value` changed.
    pub fn spin_box(
        &mut self,
        id: u64,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        step: f32,
        enabled: bool,
    ) -> bool {
        let min = *range.start();
        let max = (*range.end()).max(min);
        let step = step.abs().max(1e-6);
        *value = value.clamp(min, max);
        let mut changed = false;
        let t = self.theme();

        let b = self.reserve(48.0);
        let btn_w = 48.0_f32.min(b.width * 0.25);
        let dec_rect = Rect { width: btn_w, ..b };
        let inc_rect = Rect { x: b.x + b.width - btn_w, width: btn_w, ..b };
        let val_rect = Rect { x: b.x + btn_w, width: (b.width - 2.0 * btn_w).max(0.0), ..b };

        let dec_r = self.action_at(
            id,
            dec_rect,
            "−",
            Options { enabled: enabled && *value > min, variant: ButtonVariant::Tonal, ..Options::default() },
        );
        if dec_r.clicked {
            *value = (*value - step).clamp(min, max);
            changed = true;
        }
        let val_r = self.target(id.wrapping_add(1), val_rect, label, enabled, Role::Slider);
        self.paint_rect(val_rect.inset(2.0), 8.0, t.surface_high);
        self.focus_ring(val_r);
        if val_r.focused && enabled {
            if self.pressed_key(Key::Up) || self.pressed_key(Key::Right) {
                let next = (*value + step).clamp(min, max);
                if (next - *value).abs() > 1e-6 {
                    *value = next;
                    changed = true;
                }
            } else if self.pressed_key(Key::Down) || self.pressed_key(Key::Left) {
                let next = (*value - step).clamp(min, max);
                if (next - *value).abs() > 1e-6 {
                    *value = next;
                    changed = true;
                }
            }
        }
        let display = if (step - step.round()).abs() < 1e-4 {
            format!("{label}: {:.0}", *value)
        } else {
            format!("{label}: {:.2}", *value)
        };
        self.styled_text(
            val_rect,
            &display,
            if enabled { t.text } else { t.text.opacity(0.38) },
            TextStyle { size: 14.0, weight: 500, align: 1, ..TextStyle::default() },
        );
        let inc_r = self.action_at(
            id.wrapping_add(2),
            inc_rect,
            "+",
            Options { enabled: enabled && *value < max, variant: ButtonVariant::Tonal, ..Options::default() },
        );
        if inc_r.clicked {
            *value = (*value + step).clamp(min, max);
            changed = true;
        }
        changed
    }
}

#[cfg(test)]
#[path = "spin_box.spec.rs"]
mod tests;
