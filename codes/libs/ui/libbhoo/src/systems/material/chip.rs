use crate::*;

/// Material 3 chip role variants (pinned to MDC-Android `Widget.Material3.Chip.*`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipKind {
    Assist,
    #[default]
    Filter,
    Input,
    Suggestion,
}

/// Configuration for Material 3 chips rendered via `Frame::chip_with_options`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChipOptions {
    pub kind: ChipKind,
    pub selected: bool,
    pub elevated: bool,
    pub show_close: bool,
    pub enabled: bool,
}
impl Default for ChipOptions {
    fn default() -> Self {
        Self {
            kind: ChipKind::Filter,
            selected: false,
            elevated: false,
            show_close: false,
            enabled: true,
        }
    }
}

impl Frame<'_> {
    pub fn chip(&mut self, id: u64, label: &str, selected: bool, enabled: bool) -> Response {
        use crate::timing::{Easing, TweenSpec};
        let row = self.allocate(48.0);
        let b = Rect {
            y: row.y + 8.0,
            height: 32.0,
            ..row
        };
        let r = self.interact(id, b, enabled);
        let t = self.ui.theme;
        let target = f32::from(selected);
        // Angular Material 22.2.6 chip.scss: graphic width and transform 150ms;
        // checkmark stroke-dashoffset 150ms after a 45ms delay, all with the MDC curve.
        let spec = TweenSpec::new(0.150, Easing::Cubic([0.4, 0.0, 0.2, 1.0]));
        let slide = self.ui.motion.css_tween(id, 3, target, target, spec, spec);
        let stroke_spec = TweenSpec {
            delay: 0.045,
            ..spec
        };
        let stroke = self
            .ui
            .motion
            .css_tween(id, 4, target, target, stroke_spec, stroke_spec);
        let color = if selected {
            t.on_secondary
        } else {
            t.on_surface_variant
        };
        if selected {
            self.rect(
                b,
                8.0,
                if r.enabled {
                    t.secondary
                } else {
                    t.text.opacity(0.12)
                },
            );
        } else {
            self.rect(
                b,
                8.0,
                if !r.enabled {
                    t.text.opacity(0.12)
                } else if r.focused {
                    t.on_surface_variant
                } else {
                    t.outline
                },
            );
            self.rect(b.inset(1.0), 7.0, t.background);
        }
        let overlay_target = if !r.enabled {
            0.0
        } else if r.focused {
            0.12
        } else if r.hovered {
            0.08
        } else {
            0.0
        };
        let overlay_spec = TweenSpec::new(0.150, Easing::Linear);
        let overlay =
            self.ui
                .motion
                .css_tween(id, 0, overlay_target, 0.0, overlay_spec, overlay_spec);
        if r.enabled && overlay > 0.0 {
            self.rect(b, 8.0, color.opacity(overlay));
        }
        self.ink(r, 8.0, color);
        if selected && stroke > 0.0 {
            // The reference SVG is 20px, viewBox="-2 -3 30 30". Reveal its
            // two-segment path by length, rather than fading a completed tick.
            let left = b.x - 9.0 + 17.0 * slide;
            let point = |x: f32, y: f32| {
                [
                    left + (x + 2.0) * (2.0 / 3.0),
                    b.y + 6.0 + (y + 3.0) * (2.0 / 3.0),
                ]
            };
            let p = [point(1.73, 12.91), point(8.1, 19.28), point(22.79, 4.59)];
            let length = [6.37_f32 * 2.0_f32.sqrt(), 14.69_f32 * 2.0_f32.sqrt()];
            let mut remaining = stroke * (length[0] + length[1]);
            let ink = if r.enabled {
                color
            } else {
                t.text.opacity(0.38)
            };
            self.commands
                .push(DrawCommand::PushClip(self.clip.intersect(Rect {
                    x: b.x,
                    y: b.y + 4.0,
                    width: 12.0 + 24.0 * slide,
                    height: 24.0,
                })));
            for i in 0..2 {
                let amount = (remaining / length[i]).clamp(0.0, 1.0);
                if amount > 0.0 {
                    let end = [
                        p[i][0] + (p[i + 1][0] - p[i][0]) * amount,
                        p[i][1] + (p[i + 1][1] - p[i][1]) * amount,
                    ];
                    self.paint_line(p[i], end, 4.0 / 3.0, ink);
                }
                remaining -= length[i];
            }
            self.commands.push(DrawCommand::PopClip);
        }
        self.styled_text(
            b,
            label,
            if r.enabled {
                color
            } else {
                t.text.opacity(0.38)
            },
            TextStyle {
                align: 0,
                padding: 12.0 + 24.0 * slide,
                ..TextStyle::LABEL
            },
        );
        self.focus_ring(r);
        self.semantic(r, Role::Button, label, selected, None, None, None);
        r
    }
    /// Material 3 chip supporting Assist, Filter, Input, and Suggestion variants,
    /// flat or elevated surfaces, and an optional interactive trailing close button.
    /// Returns `(chip_response, close_clicked)`.
    pub fn chip_with_options(
        &mut self,
        id: u64,
        label: &str,
        options: ChipOptions,
    ) -> (Response, bool) {
        use crate::timing::{Easing, TweenSpec};
        let row = self.allocate(48.0);
        let b = Rect {
            y: row.y + 8.0,
            height: 32.0,
            ..row
        };
        let effective_selected = options.selected && options.kind != ChipKind::Assist;
        let show_check = options.kind == ChipKind::Filter && effective_selected;
        let close_w = if options.show_close {
            32.0_f32.min(b.width)
        } else {
            0.0
        };
        let close_hit = Rect {
            x: (b.x + b.width - close_w).max(b.x),
            y: b.y,
            width: close_w,
            height: b.height,
        };
        let body_hit = Rect {
            width: (b.width - close_w).max(0.0),
            ..b
        };
        let mut r = self.interact(id, body_hit, options.enabled);
        r.bounds = b;
        let close_response = if options.show_close {
            let close_id = id.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x4348_4950_5f43_4c53;
            Some(self.interact(close_id, close_hit, options.enabled))
        } else {
            None
        };
        let t = self.ui.theme;
        let check_target = f32::from(show_check);
        let spec = TweenSpec::new(0.150, Easing::Cubic([0.4, 0.0, 0.2, 1.0]));
        let slide = if options.kind == ChipKind::Filter {
            self.ui
                .motion
                .css_tween(id, 3, check_target, check_target, spec, spec)
        } else {
            0.0
        };
        let stroke_spec = TweenSpec {
            delay: 0.045,
            ..spec
        };
        let stroke = if options.kind == ChipKind::Filter {
            self.ui
                .motion
                .css_tween(id, 4, check_target, check_target, stroke_spec, stroke_spec)
        } else {
            0.0
        };
        let foreground = if !r.enabled {
            t.text.opacity(0.38)
        } else if effective_selected {
            t.on_secondary_container
        } else if options.kind == ChipKind::Assist {
            t.text
        } else {
            t.on_surface_variant
        };
        if options.elevated {
            if r.enabled {
                self.elevation(b, 8.0, if r.hovered { 2 } else { 1 });
            }
            self.rect(
                b,
                8.0,
                if !r.enabled {
                    t.text.opacity(0.12)
                } else if effective_selected {
                    t.secondary_container
                } else {
                    t.surface_low
                },
            );
        } else if effective_selected {
            self.rect(
                b,
                8.0,
                if r.enabled {
                    t.secondary_container
                } else {
                    t.text.opacity(0.12)
                },
            );
        } else {
            self.rect(
                b,
                8.0,
                if !r.enabled {
                    t.text.opacity(0.12)
                } else if r.focused {
                    t.on_surface_variant
                } else {
                    t.outline
                },
            );
            self.rect(b.inset(1.0), 7.0, t.background);
        }
        let overlay_target = if !r.enabled {
            0.0
        } else if r.focused {
            0.12
        } else if r.hovered {
            0.08
        } else {
            0.0
        };
        let overlay_spec = TweenSpec::new(0.150, Easing::Linear);
        let overlay =
            self.ui
                .motion
                .css_tween(id, 0, overlay_target, 0.0, overlay_spec, overlay_spec);
        if r.enabled && overlay > 0.0 {
            self.rect(b, 8.0, foreground.opacity(overlay));
        }
        self.ink(r, 8.0, foreground);
        if show_check && stroke > 0.0 {
            let left = b.x - 9.0 + 17.0 * slide;
            let point = |x: f32, y: f32| {
                [
                    left + (x + 2.0) * (2.0 / 3.0),
                    b.y + 6.0 + (y + 3.0) * (2.0 / 3.0),
                ]
            };
            let p = [point(1.73, 12.91), point(8.1, 19.28), point(22.79, 4.59)];
            let length = [6.37_f32 * 2.0_f32.sqrt(), 14.69_f32 * 2.0_f32.sqrt()];
            let mut remaining = stroke * (length[0] + length[1]);
            self.commands
                .push(DrawCommand::PushClip(self.clip.intersect(Rect {
                    x: b.x,
                    y: b.y + 4.0,
                    width: 12.0 + 24.0 * slide,
                    height: 24.0,
                })));
            for i in 0..2 {
                let amount = (remaining / length[i]).clamp(0.0, 1.0);
                if amount > 0.0 {
                    let end = [
                        p[i][0] + (p[i + 1][0] - p[i][0]) * amount,
                        p[i][1] + (p[i + 1][1] - p[i][1]) * amount,
                    ];
                    self.paint_line(p[i], end, 4.0 / 3.0, foreground);
                }
                remaining -= length[i];
            }
            self.commands.push(DrawCommand::PopClip);
        }
        let text_rect = Rect {
            width: (b.width - if options.show_close { 28.0 } else { 0.0 }).max(0.0),
            ..b
        };
        self.styled_text(
            text_rect,
            label,
            foreground,
            TextStyle {
                align: 0,
                padding: 12.0 + 24.0 * slide,
                ..TextStyle::LABEL
            },
        );
        let mut close_clicked = false;
        if let Some(cr) = close_response {
            close_clicked = cr.clicked;
            let cx = close_hit.x + close_hit.width * 0.5;
            let cy = b.y + b.height * 0.5;
            let circle = Rect {
                x: cx - 12.0,
                y: cy - 12.0,
                width: 24.0,
                height: 24.0,
            };
            if cr.enabled {
                let alpha = if cr.focused && self.ui.keyboard_focus {
                    0.12
                } else if cr.hovered {
                    0.08
                } else {
                    0.0
                };
                if alpha > 0.0 {
                    self.rect(circle, 12.0, foreground.opacity(alpha));
                }
            }
            self.ink(Response { bounds: circle, ..cr }, 12.0, foreground);
            self.paint_line([cx - 4.5, cy - 4.5], [cx + 4.5, cy + 4.5], 1.75, foreground);
            self.paint_line([cx + 4.5, cy - 4.5], [cx - 4.5, cy + 4.5], 1.75, foreground);
            let remove_label = format!("Remove {label}");
            self.semantic(cr, Role::Button, &remove_label, false, None, None, None);
        }
        self.focus_ring(r);
        self.semantic(r, Role::Button, label, effective_selected, None, None, None);
        (r, close_clicked)
    }
}

#[cfg(test)]
#[path = "chip.spec.rs"]
mod tests;
