use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckState {
    #[default]
    Unchecked,
    Checked,
    Mixed,
}

impl Frame<'_> {
    pub fn checkbox(
        &mut self,
        id: u64,
        label: &str,
        value: &mut CheckState,
        enabled: bool,
    ) -> Response {
        self.check_control(id, label, value, enabled, true, false, false, [None;2],false)
    }
    /// Tri-state checkbox with an independent error visual state.
    /// The caller supplies an accessible explanation alongside the control.
    pub fn checkbox_with_error(
        &mut self,
        id: u64,
        label: &str,
        value: &mut CheckState,
        enabled: bool,
        error: bool,
    ) -> Response {
        self.check_control(id, label, value, enabled, true, error, false, [None;2],false)
    }
    #[allow(clippy::too_many_arguments)]
    fn check_control(
        &mut self,
        id: u64,
        label: &str,
        value: &mut CheckState,
        enabled: bool,
        tristate: bool,
        error: bool,
        rtl: bool,
        tint: [Option<Color>;2],
        custom_indicator: bool,
    ) -> Response {
        let b = self.allocate(40.0);
        let mut r = self.interact(id, b, enabled);
        if r.clicked {
            *value = match value {
                CheckState::Unchecked => CheckState::Checked,
                CheckState::Checked if tristate => CheckState::Mixed,
                _ => CheckState::Unchecked,
            };
            r.changed = true;
        }
        let from = if self.ui.checkbox_transitions.contains_key(&id)
            || self.ui.checkbox_transitions.len() < 4096
        {
            let entry = self
                .ui
                .checkbox_transitions
                .entry(id)
                .or_insert((*value, *value));
            if entry.1 != *value {
                *entry = (entry.1, *value);
            }
            entry.0
        } else {
            *value
        };
        let from_mixed = from == CheckState::Mixed && *value == CheckState::Checked;
        let angular = self.ui.ripple_style == RippleStyle::AngularMaterial;
        let t = self.ui.theme;
        let selected = *value != CheckState::Unchecked;
        use crate::timing::{Easing, TweenSpec};
        let fill_in = TweenSpec::new(0.090, Easing::Cubic([0.0, 0.0, 0.2, 1.0]));
        let fill_out = TweenSpec::new(0.090, Easing::Cubic([0.4, 0.0, 0.6, 1.0]));
        let target = f32::from(selected);
        let amount = self
            .ui
            .motion
            .css_tween(id, 3, target, target, fill_in, fill_out);
        let check_target = f32::from(*value == CheckState::Checked);
        let check_in = if from_mixed && angular {
            TweenSpec::new(0.180, Easing::Cubic([0.0, 0.0, 0.2, 1.0]))
        } else if from_mixed {
            TweenSpec::new(0.500, Easing::Cubic([0.14, 0.0, 0.0, 1.0]))
        } else {
            TweenSpec {
                delay: 0.090,
                duration: 0.090,
                easing: Easing::Cubic([0.0, 0.0, 0.2, 1.0]),
            }
        };
        let check_out = if angular && *value == CheckState::Mixed {
            TweenSpec::new(0.090, Easing::Cubic([0.4, 0.0, 0.6, 1.0]))
        } else {
            TweenSpec::new(0.090, Easing::Cubic([0.4, 0.0, 1.0, 1.0]))
        };
        let checked_amount =
            self.ui
                .motion
                .css_tween(id, 4, check_target, check_target, check_in, check_out);
        let mixed_target = f32::from(*value == CheckState::Mixed);
        let mixed_amount = self.ui.motion.css_tween(
            id,
            5,
            mixed_target,
            mixed_target,
            if angular { fill_out } else { fill_in },
            if from_mixed && !angular {
                check_in
            } else {
                fill_out
            },
        );
        let box_rect = Rect {
            x: if rtl { b.x + b.width - 29.0 } else { b.x + 11.0 },
            y: b.y + 11.0,
            width: 18.0,
            height: 18.0,
        };
        let color = tint[usize::from(selected)].unwrap_or(if r.enabled && error {
            t.error
        } else if r.enabled {
            t.on_surface_variant.mix(t.primary, amount)
        } else {
            t.text.opacity(0.38)
        });
        let ink = Rect {
            x: if rtl { b.x + b.width - 40.0 } else { b.x },
            y: b.y,
            width: 40.0,
            height: 40.0,
        };
        let hovered = if angular {
            r.hovered && self.input.pointer.is_some_and(|p| ink.contains(p))
        } else {
            r.hovered
        };
        let hover = self.animate(id, 0, f32::from(hovered), 0.0);
        let focused = r.focused && (angular || self.ui.keyboard_focus);
        let state_color = if error {
            t.error
        } else if angular && *value != CheckState::Checked {
            t.text
        } else {
            t.primary
        };
        if r.enabled && (hover > 0.001 || focused) {
            self.rect(
                ink,
                20.0,
                state_color.opacity(if focused { 0.12 } else { 0.08 * hover }),
            );
        }
        self.ink(Response { bounds: ink, ..r }, 20.0, state_color);
        if !custom_indicator {
        if amount < 0.999 {
            // Leave the center transparent so the actual surface and state layer
            // remain visible, including on non-default and translucent surfaces.
            for edge in [
                Rect {
                    height: 2.0,
                    ..box_rect
                },
                Rect {
                    y: box_rect.y + 16.0,
                    height: 2.0,
                    ..box_rect
                },
                Rect {
                    y: box_rect.y + 2.0,
                    width: 2.0,
                    height: 14.0,
                    ..box_rect
                },
                Rect {
                    x: box_rect.x + 16.0,
                    y: box_rect.y + 2.0,
                    width: 2.0,
                    height: 14.0,
                },
            ] {
                self.rect(edge, 1.0, color);
            }
        }
        if amount > 0.001 {
            self.rect(box_rect, 2.0, color.opacity(amount));
        }
        let mark = if r.enabled && error {
            t.on_error
        } else if r.enabled {
            t.on_primary
        } else {
            t.background
        };
        if checked_amount > 0.001 {
            // Angular's SVG fills the 14px content box inside the 2px border.
            // Reveal the two path segments by length like its stroke-dashoffset.
            let scale = 14.0 / 24.0;
            let origin = [box_rect.x + 2.0, box_rect.y + 2.0];
            let points = [[1.73, 12.91], [8.1, 19.28], [22.79, 4.59]];
            let lengths = [9.0085_f32, 20.7748_f32];
            let distance = if *value == CheckState::Mixed || (from_mixed && !angular) {
                1.0
            } else {
                checked_amount
            } * (lengths[0] + lengths[1]);
            let p = |v: [f32; 2]| [origin[0] + v[0] * scale, origin[1] + v[1] * scale];
            let interpolate = |a: [f32; 2], b: [f32; 2], f: f32| {
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
            };
            let exiting = *value == CheckState::Unchecked;
            let (mut path, count) = if exiting {
                // Negative SVG dash offset erases from the beginning of the path.
                let start = lengths[0] + lengths[1] - distance;
                if start < lengths[0] {
                    (
                        [
                            p(interpolate(points[0], points[1], start / lengths[0])),
                            p(points[1]),
                            p(points[2]),
                        ],
                        3,
                    )
                } else {
                    (
                        [
                            p(interpolate(
                                points[1],
                                points[2],
                                ((start - lengths[0]) / lengths[1]).clamp(0.0, 1.0),
                            )),
                            p(points[2]),
                            [0.0; 2],
                        ],
                        2,
                    )
                }
            } else {
                let first = (distance / lengths[0]).clamp(0.0, 1.0);
                let second = ((distance - lengths[0]) / lengths[1]).clamp(0.0, 1.0);
                let mut path = [
                    p(points[0]),
                    p(interpolate(points[0], points[1], first)),
                    [0.0; 2],
                ];
                let count = if second > 0.0 {
                    path[2] = p(interpolate(points[1], points[2], second));
                    3
                } else {
                    2
                };
                (path, count)
            };
            if *value == CheckState::Mixed || from_mixed {
                let angle = if from_mixed && angular {
                    (1.0 - checked_amount) * std::f32::consts::FRAC_PI_4
                } else if from_mixed {
                    (1.0 + 7.0 * checked_amount) * std::f32::consts::FRAC_PI_4
                } else {
                    (1.0 - checked_amount) * std::f32::consts::FRAC_PI_4
                };
                let (sin, cos) = angle.sin_cos();
                let center = [box_rect.x + 9.0, box_rect.y + 9.0];
                for point in path.iter_mut().take(count as usize) {
                    let x = point[0] - center[0];
                    let y = point[1] - center[1];
                    *point = [center[0] + x * cos - y * sin, center[1] + x * sin + y * cos];
                }
            }
            self.commands.push(DrawCommand::Polyline {
                points: path,
                count,
                width: 3.12 * scale,
                color: if exiting || *value == CheckState::Mixed || from_mixed {
                    mark.opacity(checked_amount)
                } else {
                    mark
                },
            });
        }
        if mixed_amount > 0.001 {
            let angle = if from_mixed && angular {
                -(1.0 - mixed_amount) * std::f32::consts::FRAC_PI_4
            } else if from_mixed {
                (1.0 - mixed_amount) * 7.0 * std::f32::consts::FRAC_PI_4
            } else if selected
                && (checked_amount > 0.001 || (angular && from == CheckState::Checked))
            {
                -(1.0 - mixed_amount) * std::f32::consts::FRAC_PI_4
            } else {
                0.0
            };
            let (sin, cos) = angle.sin_cos();
            let center = [box_rect.x + 9.0, box_rect.y + 9.0];
            let scale = if angular
                && matches!(
                    (from, *value),
                    (CheckState::Unchecked, CheckState::Mixed)
                        | (CheckState::Mixed, CheckState::Unchecked)
                ) {
                mixed_amount
            } else {
                1.0
            };
            let offset = [7.0 * scale * cos, 7.0 * scale * sin];
            self.commands.push(DrawCommand::Polyline {
                points: [
                    [center[0] - offset[0], center[1] - offset[1]],
                    [center[0] + offset[0], center[1] + offset[1]],
                    [0.0; 2],
                ],
                count: 2,
                width: 2.0,
                color: mark.opacity(mixed_amount),
            });
        }
        }
        self.styled_text(
            Rect {
                x: if rtl { b.x } else { b.x + 44.0 },
                width: (b.width - 44.0).max(0.0),
                ..b
            },
            label,
            t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
            TextStyle {
                size: 14.0,
                align: if rtl { 2 } else { 0 },
                letter_spacing: 0.25,
                ..TextStyle::default()
            },
        );
        self.focus_ring(r);
        self.semantic(r, Role::Checkbox, label, selected, Some(*value), None, None);
        if let Some(node) = self.ui.semantics.last_mut() {
            node.invalid = error;
        }
        r
    }
    /// Binary checkbox; the mixed state is never introduced by this API.
    pub fn checkbox_bool(
        &mut self,
        id: u64,
        label: &str,
        value: &mut bool,
        enabled: bool,
    ) -> Response {
        self.checkbox_bool_with_error(id, label, value, enabled, false)
    }
    /// Binary checkbox with a caller-controlled, independent error state.
    pub fn checkbox_bool_with_error(
        &mut self,
        id: u64,
        label: &str,
        value: &mut bool,
        enabled: bool,
        error: bool,
    ) -> Response {
        let mut state = if *value {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        };
        let r = self.check_control(id, label, &mut state, enabled, false, error, false, [None;2],false);
        if r.changed {
            *value = state == CheckState::Checked;
        }
        r
    }
    /// Binary checkbox with layout-direction-aware indicator placement.
    pub fn checkbox_bool_directional(&mut self, id:u64, label:&str, value:&mut bool, enabled:bool, rtl:bool) -> Response {
        let mut state = if *value { CheckState::Checked } else { CheckState::Unchecked };
        let response = self.check_control(id, label, &mut state, enabled, false, false, rtl, [None;2],false);
        if response.changed { *value = state == CheckState::Checked; }
        response
    }
    pub(crate) fn checkbox_bool_directional_tinted(&mut self,id:u64,label:&str,value:&mut bool,enabled:bool,rtl:bool,tint:[Option<Color>;2],custom_indicator:bool)->Response {
        let mut state=if *value {CheckState::Checked}else{CheckState::Unchecked};
        let response=self.check_control(id,label,&mut state,enabled,false,false,rtl,tint,custom_indicator);
        if response.changed {*value=state==CheckState::Checked;}
        response
    }
}

#[cfg(test)]
#[path = "checkbox.spec.rs"]
mod tests;
