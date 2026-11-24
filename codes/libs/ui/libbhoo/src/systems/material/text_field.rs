use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldAffordance {
    #[default]
    Arrow,
    Calendar,
    Clock,
}

impl Frame<'_> {
    pub fn select_field(&mut self, id: u64, label: &str, value: &str, filled: bool) -> Response {
        self.popup_field(id, label, value, filled, FieldAffordance::Arrow)
    }
    /// Expose the text portion of a calendar field to an editing backend.
    pub fn date_range_field(
        &mut self,
        id: u64,
        label: &str,
        value: &str,
        open: bool,
    ) -> (Response, Response) {
        let (response, toggle) = self.popup_field_inner(
            id,
            label,
            value,
            true,
            FieldAffordance::Calendar,
            Some((id + 1, open)),
        );
        (response, toggle.expect("calendar toggle"))
    }
    pub fn popup_field(
        &mut self,
        id: u64,
        label: &str,
        value: &str,
        filled: bool,
        icon: FieldAffordance,
    ) -> Response {
        self.popup_field_inner(id, label, value, filled, icon, None)
            .0
    }
    fn popup_field_inner(
        &mut self,
        id: u64,
        label: &str,
        value: &str,
        filled: bool,
        icon: FieldAffordance,
        calendar: Option<(u64, bool)>,
    ) -> (Response, Option<Response>) {
        let r = self.text_field_inner(
            id,
            label,
            56.0,
            !value.is_empty(),
            false,
            filled,
            false,
            if calendar.is_some() { 48.0 } else { 0.0 },
            40.0,
        );
        if let Some(node) = self.ui.semantics.last_mut() {
            node.role = if calendar.is_some() {
                Role::TextField
            } else {
                Role::Button
            };
            if calendar.is_some() {
                node.bounds.width = (node.bounds.width - 48.0).max(0.0);
            }
        }
        if !value.is_empty() {
            self.styled_text(
                Rect {
                    y: r.bounds.y + if filled { 16.0 } else { 0.0 },
                    height: if filled { 40.0 } else { 56.0 },
                    width: (r.bounds.width - 40.0).max(0.0),
                    ..r.bounds
                },
                value,
                self.ui
                    .theme
                    .text
                    .opacity(if r.enabled { 1.0 } else { 0.38 }),
                TextStyle {
                    padding: 16.0,
                    ..TextStyle::default()
                },
            );
        }
        let center = [r.bounds.x + r.bounds.width - 20.0, r.bounds.y + 28.0];
        let color = self
            .ui
            .theme
            .on_surface_variant
            .opacity(if r.enabled { 1.0 } else { 0.38 });
        let toggle = calendar.map(|(toggle_id, open)| {
            let bounds = Rect {
                x: center[0] - 20.0,
                y: center[1] - 20.0,
                width: 40.0,
                height: 40.0,
            };
            let button = self.target(toggle_id, bounds, "Open calendar", true, Role::Button);
            let tint = if open {
                self.ui.theme.primary
            } else {
                self.ui.theme.on_surface_variant
            };
            let alpha = if button.pressed || button.focused && self.ui.keyboard_focus {
                0.12
            } else if button.hovered {
                0.08
            } else {
                0.0
            };
            if button.enabled && alpha > 0.0 {
                self.rect(bounds, 20.0, tint.opacity(alpha));
            }
            self.ink(button, 20.0, tint);
            button
        });
        let color = if calendar.is_some_and(|(_, open)| open) {
            self.ui
                .theme
                .primary
                .opacity(if r.enabled { 1.0 } else { 0.38 })
        } else {
            color
        };
        if icon == FieldAffordance::Calendar {
            let b = Rect {
                x: center[0] - 9.0,
                y: center[1] - 8.0,
                width: 18.0,
                height: 18.0,
            };
            for rect in [
                Rect { height: 5.0, ..b },
                Rect {
                    x: b.x,
                    y: b.y + 5.0,
                    width: 2.0,
                    height: 13.0,
                },
                Rect {
                    x: b.x + 16.0,
                    y: b.y + 5.0,
                    width: 2.0,
                    height: 13.0,
                },
                Rect {
                    x: b.x,
                    y: b.y + 16.0,
                    width: 18.0,
                    height: 2.0,
                },
            ] {
                self.rect(rect, 0.0, color);
            }
            for x in [b.x + 4.0, b.x + 12.0] {
                self.rect(
                    Rect {
                        x,
                        y: b.y - 2.0,
                        width: 2.0,
                        height: 5.0,
                    },
                    0.0,
                    color,
                );
            }
            self.rect(
                Rect {
                    x: b.x + 4.0,
                    y: b.y + 7.0,
                    width: 4.0,
                    height: 4.0,
                },
                0.0,
                color,
            );
        } else if icon == FieldAffordance::Clock {
            let b = Rect {
                x: center[0] - 10.0,
                y: center[1] - 10.0,
                width: 20.0,
                height: 20.0,
            };
            self.rect(b, 10.0, color);
            self.rect(
                b.inset(2.0),
                8.0,
                if filled {
                    self.ui.theme.surface_variant
                } else {
                    self.ui.theme.background
                },
            );
            self.paint_line([center[0], center[1] - 6.0], center, 1.5, color);
            self.paint_line(center, [center[0] + 4.0, center[1] + 2.5], 1.5, color);
        } else {
            let x = center[0] - 6.0;
            let y = center[1];
            self.paint_line([x - 4.0, y - 2.0], [x, y + 2.0], 2.0, color);
            self.paint_line([x, y + 2.0], [x + 4.0, y - 2.0], 2.0, color);
        }
        (r, toggle)
    }
    /// Draws an editable-field boundary. Text editing/IME belongs to the host adapter.
    pub fn edit_boundary(&mut self, id: u64, label: &str, height: f32, invalid: bool) -> Response {
        let b = self.allocate(height);
        let r = self.interact(id, b, true);
        let t = self.ui.theme;
        self.rect(
            b,
            4.0,
            if invalid {
                t.error
            } else if r.focused {
                t.primary
            } else {
                t.outline
            }
            .opacity(if r.enabled { 1.0 } else { 0.38 }),
        );
        self.rect(b.inset(1.0), 3.0, t.background);
        self.semantic(r, Role::TextField, label, false, None, None, None);
        r
    }
    /// Labeled Material field geometry. Editing/IME remains a backend adapter responsibility.
    pub fn text_field(
        &mut self,
        id: u64,
        label: &str,
        height: f32,
        has_value: bool,
        invalid: bool,
        filled: bool,
    ) -> Response {
        self.text_field_required(id, label, height, has_value, invalid, filled, false)
    }
    /// Labeled field with an optional required marker and semantic required state.
    #[allow(clippy::too_many_arguments)]
    pub fn text_field_required(
        &mut self,
        id: u64,
        label: &str,
        height: f32,
        has_value: bool,
        invalid: bool,
        filled: bool,
        required: bool,
    ) -> Response {
        self.text_field_inner(id, label, height, has_value, invalid, filled, required, 0.0, 0.0)
    }
    #[allow(clippy::too_many_arguments)]
    fn text_field_inner(
        &mut self,
        id: u64,
        label: &str,
        height: f32,
        has_value: bool,
        invalid: bool,
        filled: bool,
        required: bool,
        trailing: f32,
        label_trailing: f32,
    ) -> Response {
        let total = self.allocate(height + 20.0);
        let b = Rect { height, ..total };
        let mut r = self.interact(
            id,
            Rect {
                width: (b.width - trailing).max(0.0),
                ..b
            },
            true,
        );
        r.bounds = b;
        if trailing > 0.0 {
            r.hovered =
                r.enabled && self.pointer_in(b) && self.input.pointer_kind != PointerKind::Touch;
        }
        let t = self.ui.theme;
        use crate::timing::{Easing, TweenSpec};
        let curve = Easing::Cubic([0.4, 0.0, 0.2, 1.0]);
        let label_spec = TweenSpec::new(0.150, curve);
        let floating = has_value || r.focused;
        let target = f32::from(floating);
        let amount = self
            .ui
            .motion
            .css_tween(id, 60, target, target, label_spec, label_spec);
        let label_target = if !r.enabled {
            t.text.opacity(0.38)
        } else if invalid {
            t.error
        } else if r.focused {
            t.primary
        } else if r.hovered && !filled {
            t.text
        } else {
            t.on_surface_variant
        };
        let target_channels = [
            label_target.0,
            label_target.1,
            label_target.2,
            label_target.3,
        ];
        let mut channels = [0u8; 4];
        for (index, value) in target_channels.into_iter().enumerate() {
            let target = f32::from(value) / 255.0;
            let sample = self.ui.motion.css_tween(
                id,
                61 + index as u8,
                target,
                target,
                label_spec,
                label_spec,
            );
            channels[index] = (sample * 255.0).round() as u8;
        }
        let foreground = Color(channels[0], channels[1], channels[2], channels[3]);
        let edge = if !r.enabled {
            t.text.opacity(0.12)
        } else if invalid {
            t.error
        } else if r.focused {
            t.primary
        } else if r.hovered {
            t.text
        } else {
            t.outline
        };
        if filled {
            self.rect(
                b,
                4.0,
                if r.enabled {
                    t.surface_variant
                } else {
                    t.text.opacity(0.04)
                },
            );
            if r.enabled && r.hovered && !r.focused {
                self.rect(b, 4.0, t.text.opacity(0.08));
            }
            let base = if !r.enabled {
                t.text.opacity(0.38)
            } else if invalid {
                t.error
            } else if r.hovered && !r.focused {
                t.text
            } else {
                t.on_surface_variant
            };
            self.rect(
                Rect {
                    y: b.y + b.height - 1.0,
                    height: 1.0,
                    ..b
                },
                0.0,
                base,
            );
            let line_spec = TweenSpec::new(0.180, curve);
            let focus = f32::from(r.focused && r.enabled);
            let line_alpha = self
                .ui
                .motion
                .css_tween(id, 65, focus, focus, line_spec, line_spec);
            let scale_target = f32::from(focus > 0.0 || line_alpha > 0.0);
            let line_scale =
                self.ui
                    .motion
                    .css_tween(id, 66, scale_target, scale_target, line_spec, line_spec);
            if line_alpha > 0.0 && line_scale > 0.0 {
                let width = b.width * line_scale;
                self.rect(
                    Rect {
                        x: b.x + (b.width - width) / 2.0,
                        y: b.y + b.height - 2.0,
                        width,
                        height: 2.0,
                    },
                    0.0,
                    if invalid {
                        t.error.opacity(line_alpha)
                    } else {
                        t.primary.opacity(line_alpha)
                    },
                );
            }
        } else {
            let stroke = if r.focused { 2.0 } else { 1.0 };
            self.rect(b, 4.0, edge);
            self.rect(b.inset(stroke), (4.0 - stroke).max(0.0), t.background);
            if floating {
                let style = TextStyle {
                    size: 12.0,
                    letter_spacing: 0.375,
                    ..TextStyle::default()
                };
                let width = (self.text_width(label, style)
                    + if required {
                        self.text_width("*", style) + 1.0
                    } else {
                        0.0
                    }
                    + 8.0).min((b.width - 24.0 - label_trailing).max(0.0));
                self.rect(
                    Rect {
                        x: b.x + 12.0,
                        y: b.y - 8.0,
                        width,
                        height: 16.0,
                    },
                    0.0,
                    t.background,
                );
            }
        }
        // Angular's floating label keeps the 56px label origin even for a textarea.
        // The transformed 16px label scales to 12px while moving to the notch.
        let label_bounds = Rect {
            x: b.x + 16.0,
            y: b.y
                + if filled {
                    6.05 * amount
                } else {
                    1.0 - 9.0 * amount
                },
            width: (b.width - 32.0 - label_trailing).max(0.0),
            height: 56.0 - 40.0 * amount,
        };
        let style = TextStyle {
            size: 16.0 - 4.0 * amount,
            letter_spacing: 0.5 * (1.0 - 0.25 * amount),
            ..TextStyle::default()
        };
        self.styled_text(label_bounds, label, foreground, style);
        if required {
            let marker_x = label_bounds.x + self.text_width(label, style) + (1.0 - 0.25 * amount);
            self.styled_text(
                Rect {
                    x: marker_x,
                    width: (label_bounds.x + label_bounds.width - marker_x).max(0.0),
                    ..label_bounds
                },
                "*",
                foreground,
                style,
            );
        }
        self.semantic(r, Role::TextField, label, false, None, None, None);
        if let Some(node) = self.ui.semantics.last_mut() {
            node.invalid = invalid;
            node.filled = filled;
            node.required = required;
        }
        r
    }

    /// Labeled Material field rendered at caller-supplied `bounds` that also paints `value` text.
    #[allow(clippy::too_many_arguments)]
    pub fn text_field_at(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        value: &str,
        invalid: bool,
        filled: bool,
    ) -> Response {
        let mut resp = Response::default();
        let field_h = (bounds.height - 1.0).max(40.0);
        self.within(bounds, |f| {
            resp = f.text_field(id, label, field_h, !value.is_empty(), invalid, filled);
            if !value.is_empty() {
                let text_top = if filled { bounds.y + 23.0 } else { bounds.y + 14.0 };
                f.paragraph(
                    Rect {
                        x: bounds.x + 16.0,
                        y: text_top,
                        width: (bounds.width - 32.0).max(0.0),
                        height: (bounds.y + bounds.height - text_top - 4.0).max(16.0),
                    },
                    value,
                    f.theme().text.opacity(if resp.enabled { 1.0 } else { 0.38 }),
                    TextStyle { size: 16.0, ..TextStyle::default() },
                    22.0,
                );
            }
        });
        resp
    }

    /// Material text field paired with supporting hint/error subscript line and optional character counter.
    #[allow(clippy::too_many_arguments)]
    pub fn text_field_with_subscript(
        &mut self,
        id: u64,
        label: &str,
        value: &str,
        filled: bool,
        invalid: bool,
        required: bool,
        supporting: &str,
        max_chars: Option<usize>,
    ) -> Response {
        let r = self.text_field_required(id, label, 56.0, !value.is_empty(), invalid, filled, required);
        let t = self.theme();
        let sub = self.reserve(20.0);
        let color = if invalid { t.error } else { t.on_surface_variant };
        let style = TextStyle { size: 12.0, padding: 16.0, ..TextStyle::default() };
        if !supporting.is_empty() {
            let w = if max_chars.is_some() { (sub.width - 72.0).max(0.0) } else { sub.width };
            self.styled_text(Rect { width: w, ..sub }, supporting, color, style);
        }
        if let Some(limit) = max_chars {
            let count = value.chars().count();
            let counter = format!("{count}/{limit}");
            self.styled_text(
                sub,
                &counter,
                if count > limit || invalid { t.error } else { t.on_surface_variant },
                TextStyle { align: 2, ..style },
            );
        }
        r
    }
}

#[cfg(test)]
#[path = "text_field.spec.rs"]
mod tests;
