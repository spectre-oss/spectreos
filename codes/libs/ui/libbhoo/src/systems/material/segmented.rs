use crate::*;

impl Frame<'_> {
    /// Single or multiple segmented selection, with roving keyboard interaction.
    pub fn segmented(
        &mut self,
        id: u64,
        labels: &[&str],
        mask: &mut u32,
        multiple: bool,
        enabled: bool,
    ) -> Response {
        assert!(
            labels.len() <= 32,
            "Segmented controls support at most 32 choices"
        );
        let b = self.allocate(42.0);
        let mut r = self.interact(id, b, enabled && !labels.is_empty());
        if labels.is_empty() {
            return r;
        }
        let old = *mask;
        let count = labels.len();
        let angular = self.ui.ripple_style == RippleStyle::AngularMaterial;
        let divider = if angular { 1.0 } else { 0.0 };
        let width = (b.width - 2.0 - divider * (count - 1) as f32).max(0.0) / count as f32;
        if !multiple && *mask == 0 {
            *mask = 1;
        }
        let mut selected = self
            .ui
            .part_focus
            .get(&id)
            .copied()
            .unwrap_or_else(|| {
                if multiple {
                    0
                } else {
                    mask.trailing_zeros() as usize
                }
            })
            .min(count - 1);
        if r.clicked {
            if self.ui.capture == Capture::Pointer
                && self.input.released
                && let Some(p) = self.input.pointer
            {
                selected = (((p[0] - b.x - 1.0) / (width + divider).max(1.0))
                    .floor()
                    .max(0.0) as usize)
                    .min(count - 1);
            }
            if self.ui.capture == Capture::Keyboard || self.input.released {
                if multiple {
                    *mask ^= 1 << selected;
                } else {
                    *mask = 1 << selected;
                }
            }
        }
        if let Some(key) = self.key_for(r) {
            let navigation = match key {
                Key::Right | Key::Down => {
                    selected = (selected + 1) % count;
                    true
                }
                Key::Left | Key::Up => {
                    selected = (selected + count - 1) % count;
                    true
                }
                Key::Home => {
                    selected = 0;
                    true
                }
                Key::End => {
                    selected = count - 1;
                    true
                }
                _ => false,
            };
            if navigation && !multiple {
                *mask = 1 << selected;
            }
        }
        if self.ui.part_focus.len() < 4096 || self.ui.part_focus.contains_key(&id) {
            self.ui.part_focus.insert(id, selected);
        }
        let t = self.ui.theme;
        self.rect(b, 21.0, t.outline);
        self.rect(b.inset(1.0), 20.0, t.background);
        for (i, label) in labels.iter().enumerate() {
            let cell = Rect {
                x: b.x + 1.0 + i as f32 * (width + divider),
                y: b.y + 1.0,
                width,
                height: 40.0,
            };
            let checked = *mask & (1 << i) != 0;
            let child = Response {
                bounds: cell,
                focused: r.focused && i == selected,
                hovered: r.hovered && self.input.pointer.is_some_and(|p| cell.contains(p)),
                ..r
            };
            let clip = cell.intersect(self.clip);
            self.commands.push(DrawCommand::PushClip(clip));
            if checked {
                self.rect(
                    b.inset(1.0),
                    20.0,
                    if angular && !r.enabled {
                        t.text.opacity(0.12)
                    } else {
                        t.secondary
                    },
                );
            }
            self.commands.push(DrawCommand::PopClip);
            if i > 0 {
                self.rect(
                    Rect {
                        x: cell.x - divider,
                        y: cell.y,
                        width: 1.0,
                        height: 40.0,
                    },
                    0.0,
                    t.outline,
                );
            }
            let color = if r.enabled {
                if checked {
                    t.on_secondary
                } else {
                    if angular {
                        t.text
                    } else {
                        t.on_surface_variant
                    }
                }
            } else {
                t.text.opacity(0.38)
            };
            let mark_amount = if angular {
                use crate::timing::{Easing, TweenSpec};
                let spec = TweenSpec {
                    delay: 0.045,
                    duration: 0.150,
                    easing: Easing::Cubic([0.4, 0.0, 0.2, 1.0]),
                };
                self.ui.motion.css_tween(
                    id,
                    80 + i as u8,
                    f32::from(checked),
                    f32::from(checked),
                    spec,
                    spec,
                )
            } else {
                f32::from(checked)
            };
            if angular && mark_amount > 0.001 {
                let x = cell.x + 12.0;
                let clip = Rect {
                    x,
                    y: cell.y + 11.0,
                    width: 18.0 * mark_amount,
                    height: 18.0,
                }
                .intersect(self.clip);
                self.commands.push(DrawCommand::PushClip(clip));
                self.paint_line(
                    [x + 2.5, cell.y + 20.0],
                    [x + 6.5, cell.y + 24.0],
                    2.0,
                    color,
                );
                self.paint_line(
                    [x + 6.5, cell.y + 24.0],
                    [x + 15.0, cell.y + 15.5],
                    2.0,
                    color,
                );
                self.commands.push(DrawCommand::PopClip);
            } else if !angular && checked {
                let x = cell.x + cell.width / 2.0
                    - (self.text_width(label, TextStyle::LABEL) + 24.0) / 2.0;
                self.paint_line(
                    [x + 1.0, cell.y + 20.0],
                    [x + 5.0, cell.y + 24.0],
                    1.5,
                    color,
                );
                self.paint_line(
                    [x + 5.0, cell.y + 24.0],
                    [x + 13.0, cell.y + 16.0],
                    1.5,
                    color,
                );
            }
            if child.enabled {
                let amount = if child.focused && self.ui.keyboard_focus {
                    0.12
                } else if child.hovered {
                    0.08
                } else {
                    0.0
                };
                if amount > 0.0 {
                    if angular {
                        // The state layer shares the group's rounded outside edge,
                        // while the rectangular clip keeps it inside this segment.
                        self.commands.push(DrawCommand::PushClip(clip));
                        self.rect(b.inset(1.0), 20.0, t.text.opacity(amount));
                        self.commands.push(DrawCommand::PopClip);
                    } else {
                        self.rect(cell, 0.0, color.opacity(amount));
                    }
                }
            }
            self.styled_text(
                Rect {
                    x: cell.x
                        + if angular {
                            30.0 * mark_amount
                        } else if checked {
                            24.0
                        } else {
                            0.0
                        },
                    width: cell.width
                        - if angular {
                            30.0 * mark_amount
                        } else if checked {
                            24.0
                        } else {
                            0.0
                        },
                    ..cell
                },
                label,
                color,
                TextStyle::LABEL,
            );
            if angular {
                // Keep the ripple's origin and lifetime relative to the segment,
                // but clip its drawn shape to the rounded group and segment.
                self.commands.push(DrawCommand::PushClip(clip));
                self.ink_part(child, i as u32 + 1, 0.0, t.text, false);
                if let Some(DrawCommand::Ripple {
                    bounds,
                    corner_radius,
                    ..
                }) = self.commands.last_mut()
                {
                    *bounds = b.inset(1.0);
                    *corner_radius = 20.0;
                    self.commands.push(DrawCommand::PopClip);
                } else {
                    // Avoid emitting an empty clip pair when there is no ripple.
                    self.commands.pop();
                }
            } else {
                self.ink_part(child, i as u32 + 1, 0.0, color, false);
            }
            self.semantic(
                child,
                if multiple {
                    Role::Checkbox
                } else {
                    Role::Radio
                },
                label,
                checked,
                Some(if checked {
                    CheckState::Checked
                } else {
                    CheckState::Unchecked
                }),
                None,
                None,
            );
            if let Some(node) = self.ui.semantics.last_mut() {
                node.part = i as u32 + 1;
            }
        }
        self.focus_ring(r);
        self.semantic(
            r,
            Role::Group,
            "Segmented selection",
            false,
            None,
            Some(*mask as f32),
            None,
        );
        r.changed = old != *mask;
        r
    }
}

#[cfg(test)]
#[path = "segmented.spec.rs"]
mod tests;
