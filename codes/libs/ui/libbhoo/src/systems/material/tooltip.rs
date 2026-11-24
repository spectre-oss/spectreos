use crate::*;

impl Frame<'_> {
    fn angular_tooltip(&mut self, id: u64, response: Response, text: &str) {
        use crate::timing::{Easing, TweenSpec};
        let viewport = self.viewport();
        let style = TextStyle {
            size: 12.0,
            padding: 8.0,
            letter_spacing: 0.4,
            align: 1,
            ..TextStyle::default()
        };
        let width = (self.text_width(text, style) + style.letter_spacing + 16.0)
            .clamp(40.0, 200.0)
            .min((viewport.width - 16.0).max(0.0));
        let x = (response.bounds.x + (response.bounds.width - width) * 0.5).clamp(
            viewport.x + 8.0,
            (viewport.x + viewport.width - width - 8.0).max(viewport.x + 8.0),
        );
        let below = response.bounds.y + response.bounds.height + 8.0;
        let y = if below + 24.0 <= viewport.y + viewport.height - 8.0 {
            below
        } else {
            (response.bounds.y - 32.0).max(viewport.y + 8.0)
        };
        let bounds = Rect {
            x,
            y,
            width,
            height: 24.0,
        };
        let bridge = Rect {
            y: if y >= response.bounds.y { y - 8.0 } else { y },
            height: 32.0,
            ..bounds
        };
        let on_tooltip = self.ui.tooltip_showing == Some(response.id)
            && self.input.pointer.is_some_and(|p| bridge.contains(p));
        let eligible = response.enabled
            && (response.hovered || response.focused && self.ui.keyboard_focus || on_tooltip);
        if eligible && self.pressed_key(Key::Escape) {
            self.ui.tooltip_dismissed = Some(response.id);
        }
        if !eligible && self.ui.tooltip_dismissed == Some(response.id) {
            self.ui.tooltip_dismissed = None;
        }
        let suppressed = self.ui.tooltip_dismissed == Some(response.id);
        if eligible && !suppressed {
            if !self
                .ui
                .tooltip_started
                .is_some_and(|(owner, _)| owner == response.id)
            {
                self.ui.tooltip_started = Some((response.id, self.ui.time));
            }
        } else if self
            .ui
            .tooltip_started
            .is_some_and(|(owner, _)| owner == response.id)
        {
            self.ui.tooltip_started = None;
        }
        let deadline = self.ui.tooltip_started.map(|(_, start)| start + 0.5);
        let show =
            eligible && !suppressed && deadline.is_some_and(|deadline| self.ui.time >= deadline);
        if eligible
            && !suppressed
            && !show
            && let Some(deadline) = deadline
        {
            self.ui.wakeup = Some(self.ui.wakeup.map_or(deadline, |old| old.min(deadline)));
        }
        if show {
            self.ui.tooltip_showing = Some(response.id);
        } else if self.ui.tooltip_showing == Some(response.id) {
            self.ui.tooltip_showing = None;
        }
        self.ui.tooltip_visible |= show;
        let amount = self.ui.motion.tween(
            id,
            4,
            f32::from(show),
            0.0,
            TweenSpec::new(0.150, Easing::StandardDecelerate),
            TweenSpec::new(0.075, Easing::Cubic([0.4, 0.0, 1.0, 1.0])),
        );
        if amount <= 0.001 {
            return;
        }
        let scale = 0.8 + 0.2 * amount;
        let visual = Rect {
            x: bounds.x + bounds.width * (1.0 - scale) * 0.5,
            // CDK anchors a below tooltip at its top edge (and an above tooltip at its bottom edge).
            y: bounds.y
                + if y < response.bounds.y {
                    bounds.height * (1.0 - scale)
                } else {
                    0.0
                },
            width: bounds.width * scale,
            height: bounds.height * scale,
        };
        let label = self.ui.text(text);
        let theme = self.ui.theme;
        self.ui.overlays.push(DrawCommand::RoundedRect {
            bounds: visual,
            radius: 4.0 * scale,
            color: theme.inverse_surface.opacity(amount),
        });
        self.ui.overlays.push(DrawCommand::Text {
            bounds: visual,
            text: label,
            color: theme.inverse_on_surface.opacity(amount),
            style: TextStyle {
                size: style.size * scale,
                padding: style.padding * scale,
                letter_spacing: style.letter_spacing * scale,
                ..style
            },
        });
        if show {
            self.semantic(
                Response {
                    id,
                    bounds,
                    enabled: true,
                    ..Response::default()
                },
                Role::Tooltip,
                text,
                false,
                None,
                None,
                None,
            );
        }
    }

    /// Hover delay uses an idle deadline, not a spinning frame loop. Call after the owner.
    pub fn tooltip(&mut self, id: u64, response: Response, text: &str) {
        if self.ui.ripple_style == RippleStyle::AngularMaterial {
            self.angular_tooltip(id, response, text);
            return;
        }
        let eligible =
            response.enabled && (response.hovered || (response.focused && self.ui.keyboard_focus));
        if eligible && self.pressed_key(Key::Escape) {
            self.ui.tooltip_dismissed = Some(response.id);
        }
        if !eligible && self.ui.tooltip_dismissed == Some(response.id) {
            self.ui.tooltip_dismissed = None;
        }
        let suppressed = self.ui.tooltip_dismissed == Some(response.id);
        let deadline = self.ui.hover_start + 0.5;
        let show = eligible
            && !suppressed
            && (response.focused && self.ui.keyboard_focus || self.ui.time >= deadline);
        self.ui.tooltip_visible |= show;
        if eligible && !suppressed && !show && deadline > self.ui.time {
            self.ui.wakeup = Some(self.ui.wakeup.map_or(deadline, |old| old.min(deadline)));
        }
        let alpha = self.animate(id, 4, if show { 1.0 } else { 0.0 }, 0.0);
        if alpha <= 0.001 {
            return;
        }
        let width = (text.chars().count() as f32 * 10.0 + 24.0)
            .min(300.0)
            .min(self.bounds.width.max(0.0));
        let x = response
            .bounds
            .x
            .min((self.bounds.x + self.bounds.width - width).max(self.bounds.x));
        let y = (response.bounds.y + response.bounds.height + 4.0)
            .min((self.bounds.y + self.bounds.height - 32.0).max(self.bounds.y));
        let b = Rect {
            x,
            y,
            width,
            height: 32.0,
        };
        let label = self.ui.text(text);
        let t = self.ui.theme;
        self.ui.overlays.push(DrawCommand::RoundedRect {
            bounds: b,
            radius: 4.0,
            color: t.inverse_surface.opacity(alpha),
        });
        self.ui.overlays.push(DrawCommand::Text {
            bounds: b,
            text: Arc::clone(&label),
            color: t.inverse_on_surface.opacity(alpha),
            style: TextStyle {
                size: 12.0,
                padding: 8.0,
                letter_spacing: 0.4,
                ..TextStyle::default()
            },
        });
        if show {
            self.semantic(
                Response {
                    id,
                    bounds: b,
                    enabled: true,
                    ..Response::default()
                },
                Role::Tooltip,
                text,
                false,
                None,
                None,
                None,
            );
        }
    }

    /// Material 3 Rich Tooltip: 12dp radius, level-2 elevation, `surface_low` container,
    /// bold 14sp subhead, 14sp body text, and an optional interactive primary text action
    /// button returning `true` when clicked.
    pub fn rich_tooltip(
        &mut self,
        id: u64,
        anchor: Response,
        subhead: &str,
        body: &str,
        action_label: Option<&str>,
    ) -> bool {
        let viewport = self.viewport();
        let subhead_style = TextStyle {
            size: 14.0,
            weight: 700,
            align: 0,
            padding: 0.0,
            letter_spacing: 0.1,
            ..TextStyle::default()
        };
        let body_style = TextStyle {
            size: 14.0,
            weight: 400,
            align: 0,
            padding: 0.0,
            letter_spacing: 0.25,
            ..TextStyle::default()
        };
        let action_style = TextStyle {
            size: 14.0,
            weight: 500,
            align: 1,
            padding: 8.0,
            letter_spacing: 0.1,
            ..TextStyle::default()
        };
        let action = action_label.filter(|s| !s.is_empty());
        let pad_x = 16.0;
        let pad_top = 12.0;
        let pad_bottom = if action.is_some() { 8.0 } else { 12.0 };
        let max_w = (viewport.width - 16.0).clamp(0.0, 312.0);
        let subhead_w = if subhead.is_empty() {
            0.0
        } else {
            self.text_width(subhead, subhead_style)
        };
        let body_w = if body.is_empty() {
            0.0
        } else {
            self.text_width(body, body_style)
        };
        let action_w = action.map_or(0.0, |a| self.text_width(a, action_style) + 16.0);
        let natural_w = subhead_w.max(body_w).max(action_w) + pad_x * 2.0;
        let width = natural_w.clamp(200.0_f32.min(max_w), max_w);
        let content_w = (width - pad_x * 2.0).max(1.0);
        let metrics = self.ui.text_metrics;
        let body_lines = if body.is_empty() {
            0
        } else {
            metrics.lines(body, content_w, body_style, |_| {}).max(1)
        };
        let subhead_h = if subhead.is_empty() { 0.0 } else { 20.0 };
        let subhead_gap = if !subhead.is_empty() && body_lines > 0 {
            4.0
        } else {
            0.0
        };
        let body_h = body_lines as f32 * 20.0;
        let action_gap = if action.is_some() && (subhead_h > 0.0 || body_h > 0.0) {
            12.0
        } else {
            0.0
        };
        let action_h = if action.is_some() { 32.0 } else { 0.0 };
        let height = pad_top + subhead_h + subhead_gap + body_h + action_gap + action_h + pad_bottom;
        let x = anchor.bounds.x.clamp(
            viewport.x + 8.0,
            (viewport.x + viewport.width - width - 8.0).max(viewport.x + 8.0),
        );
        let below = anchor.bounds.y + anchor.bounds.height + 8.0;
        let y = if below + height <= viewport.y + viewport.height - 8.0 {
            below
        } else {
            (anchor.bounds.y - height - 8.0).max(viewport.y + 8.0)
        };
        let bounds = Rect {
            x,
            y,
            width,
            height,
        };
        let bridge = Rect {
            x: bounds.x.min(anchor.bounds.x),
            y: if y >= anchor.bounds.y {
                anchor.bounds.y + anchor.bounds.height
            } else {
                y + height
            },
            width: (bounds.x + bounds.width)
                .max(anchor.bounds.x + anchor.bounds.width)
                - bounds.x.min(anchor.bounds.x),
            height: 8.0,
        };
        let action_id = id.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x7269_6368_5f61_6374;
        let on_tooltip = self.ui.tooltip_showing == Some(anchor.id)
            && self
                .input
                .pointer
                .is_some_and(|p| bounds.contains(p) || bridge.contains(p));
        let action_engaged = self.ui.active == Some(action_id)
            || (self.ui.focused == Some(action_id) && self.ui.keyboard_focus);
        let eligible = anchor.enabled
            && (anchor.hovered
                || (anchor.focused && self.ui.keyboard_focus)
                || on_tooltip
                || action_engaged);
        if eligible && self.pressed_key(Key::Escape) {
            self.ui.tooltip_dismissed = Some(anchor.id);
        }
        if !eligible && self.ui.tooltip_dismissed == Some(anchor.id) {
            self.ui.tooltip_dismissed = None;
        }
        let suppressed = self.ui.tooltip_dismissed == Some(anchor.id);
        if eligible && !suppressed {
            if !self
                .ui
                .tooltip_started
                .is_some_and(|(owner, _)| owner == anchor.id)
            {
                self.ui.tooltip_started = Some((anchor.id, self.ui.time));
            }
        } else if self
            .ui
            .tooltip_started
            .is_some_and(|(owner, _)| owner == anchor.id)
        {
            self.ui.tooltip_started = None;
        }
        let deadline = self
            .ui
            .tooltip_started
            .filter(|(owner, _)| *owner == anchor.id)
            .map(|(_, start)| start + 0.5);
        let show = eligible
            && !suppressed
            && (self.ui.tooltip_showing == Some(anchor.id)
                || (anchor.focused && self.ui.keyboard_focus)
                || action_engaged
                || deadline.is_some_and(|d| self.ui.time >= d));
        if eligible
            && !suppressed
            && !show
            && let Some(deadline) = deadline
            && deadline > self.ui.time
        {
            self.ui.wakeup = Some(self.ui.wakeup.map_or(deadline, |old| old.min(deadline)));
        }
        if show {
            self.ui.tooltip_showing = Some(anchor.id);
        } else if self.ui.tooltip_showing == Some(anchor.id) {
            self.ui.tooltip_showing = None;
        }
        self.ui.tooltip_visible |= show;
        let alpha = self.animate(id, 4, f32::from(show), 0.0);
        if alpha <= 0.001 {
            return false;
        }
        let t = self.ui.theme;
        let black = Color(0, 0, 0, 255);
        // Elevation level 2 shadows on the overlay layer.
        for (ox, oy, blur, spread, sa) in [
            (0.0, 3.0, 1.0, -2.0, 0.2),
            (0.0, 2.0, 2.0, 0.0, 0.14),
            (0.0, 1.0, 5.0, 0.0, 0.12),
        ] {
            self.ui.overlays.push(DrawCommand::Shadow {
                bounds,
                radius: 12.0,
                offset: [ox, oy],
                blur,
                spread,
                color: black.opacity(sa * alpha),
            });
        }
        self.ui.overlays.push(DrawCommand::RoundedRect {
            bounds,
            radius: 12.0,
            color: t.surface_low.opacity(alpha),
        });
        let mut cursor_y = bounds.y + pad_top;
        if !subhead.is_empty() {
            let subhead_text = self.ui.text(subhead);
            self.ui.overlays.push(DrawCommand::Text {
                bounds: Rect {
                    x: bounds.x + pad_x,
                    y: cursor_y,
                    width: content_w,
                    height: subhead_h,
                },
                text: subhead_text,
                color: t.text.opacity(alpha),
                style: subhead_style,
            });
            cursor_y += subhead_h + subhead_gap;
        }
        if !body.is_empty() {
            let mut line_index = 0usize;
            let mut line_strings = Vec::new();
            metrics.lines(body, content_w, body_style, |line| {
                line_strings.push(line.to_owned());
            });
            for line in line_strings {
                let cached = self.ui.text(&line);
                self.ui.overlays.push(DrawCommand::Text {
                    bounds: Rect {
                        x: bounds.x + pad_x,
                        y: cursor_y + line_index as f32 * 20.0,
                        width: content_w,
                        height: 20.0,
                    },
                    text: cached,
                    color: t.on_surface_variant.opacity(alpha),
                    style: body_style,
                });
                line_index += 1;
            }
            cursor_y += body_h;
        }
        let mut action_clicked = false;
        if let Some(action_label) = action {
            cursor_y += action_gap;
            let btn_w = (self.text_width(action_label, action_style) + 16.0)
                .clamp(48.0_f32.min(content_w), content_w);
            let btn_rect = Rect {
                x: bounds.x + pad_x - 8.0,
                y: cursor_y,
                width: btn_w,
                height: action_h,
            };
            if show {
                let btn_resp = self.interact(action_id, btn_rect, true);
                action_clicked = btn_resp.clicked;
                let state_alpha = if btn_resp.pressed {
                    0.12
                } else if btn_resp.focused && self.ui.keyboard_focus {
                    0.12
                } else if btn_resp.hovered {
                    0.08
                } else {
                    0.0
                };
                if state_alpha > 0.0 {
                    self.ui.overlays.push(DrawCommand::RoundedRect {
                        bounds: btn_rect,
                        radius: 16.0,
                        color: t.primary.opacity(state_alpha * alpha),
                    });
                }
                self.semantic(
                    btn_resp,
                    Role::Button,
                    action_label,
                    false,
                    None,
                    None,
                    None,
                );
            }
            let cached_action = self.ui.text(action_label);
            self.ui.overlays.push(DrawCommand::Text {
                bounds: btn_rect,
                text: cached_action,
                color: t.primary.opacity(alpha),
                style: action_style,
            });
        }
        if show {
            let semantic_label = match (subhead.is_empty(), body.is_empty()) {
                (false, false) => format!("{subhead}: {body}"),
                (false, true) => subhead.to_owned(),
                (true, false) => body.to_owned(),
                (true, true) => String::new(),
            };
            self.semantic(
                Response {
                    id,
                    bounds,
                    enabled: true,
                    ..Response::default()
                },
                Role::Tooltip,
                &semantic_label,
                false,
                None,
                None,
                None,
            );
        }
        action_clicked
    }
}

#[cfg(test)]
#[path = "tooltip.spec.rs"]
mod tests;
