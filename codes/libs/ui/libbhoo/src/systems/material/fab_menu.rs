use crate::*;

/// A named FAB menu action with a presentation glyph.
#[derive(Clone, Copy, Debug)]
pub struct FabMenuAction<'a> {
    pub label: &'a str,
    pub glyph: &'a str,
    pub enabled: bool,
}

/// Caller-owned transition state for a floating action menu.
#[derive(Default)]
pub struct FabMenuState {
    was_open: bool,
    changed_at: f64,
    scroll: f32,
    pending_focus: Option<usize>,
    dragging: bool,
}
impl FabMenuState {
    pub fn scroll_offset(&self) -> f32 {
        self.scroll
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FabMenuOptions {
    pub rtl: bool,
}

impl Frame<'_> {
    /// FAB with up to eight labeled actions expanding above it.
    pub fn fab_menu(
        &mut self,
        id: u64,
        bounds: Rect,
        open: &mut bool,
        actions: &[FabMenuAction<'_>],
        state: &mut FabMenuState,
    ) -> Option<usize> {
        self.fab_menu_with(id, bounds, open, actions, state, FabMenuOptions::default())
    }
    /// FAB menu with logical direction and scrollable constrained-height items.
    pub fn fab_menu_with(
        &mut self,
        id: u64,
        bounds: Rect,
        open: &mut bool,
        actions: &[FabMenuAction<'_>],
        state: &mut FabMenuState,
        options: FabMenuOptions,
    ) -> Option<usize> {
        let count = actions.len();
        if count == 0
            || count > 8
            || id > u64::MAX - count as u64
            || !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width < 56.0
            || bounds.height < 120.0
        {
            return None;
        }
        let visible = bounds.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return None;
        }
        let trigger = Rect {
            x: if options.rtl {
                bounds.x
            } else {
                bounds.x + bounds.width - 56.0
            },
            y: bounds.y + bounds.height - 56.0,
            width: 56.0,
            height: 56.0,
        };
        if *open && self.input.pressed && self.input.pointer.is_some_and(|p| !bounds.contains(p)) {
            *open = false;
        }
        if *open
            && self.pressed_key(Key::Escape)
            && self
                .ui
                .focused
                .is_some_and(|focus| focus >= id && focus <= id + count as u64)
        {
            *open = false;
            self.request_focus(id);
            self.key_consumed = true;
        }
        let r = self.target(
            id,
            trigger,
            if *open {
                "Close actions"
            } else {
                "Open actions"
            },
            true,
            Role::Button,
        );
        if r.clicked {
            *open = !*open;
        }
        let focus_first = *open
            && r.focused
            && !self.key_consumed
            && self
                .input
                .key
                .is_some_and(|k| k.pressed && k.key == Key::Down);
        let items_window = Rect {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height - 64.0,
        };
        let content_height = count as f32 * 56.0 + (count - 1) as f32 * 4.0;
        let maximum_scroll = (content_height - items_window.height).max(0.0);
        if state.was_open != *open {
            state.was_open = *open;
            state.changed_at = self.ui.time;
            if *open {
                state.scroll = maximum_scroll;
            } else {
                state.pending_focus = None;
            }
        }
        if *open {
            state.dragging=self.scroll_viewport(id.wrapping_add(0x100000),items_window,&mut state.scroll,maximum_scroll);
            if state.dragging {self.ui.active=None;}
        }
        if focus_first && let Some(index) = actions.iter().position(|action| action.enabled) {
            state.scroll = ((index as f32 * 60.0).min(maximum_scroll)).max(0.0);
            state.pending_focus = Some(index);
            self.key_consumed = true;
            self.ui.continuous = true;
        }
        if *open
            && !self.key_consumed
            && let Some(key) = self.input.key
            && key.pressed
            && matches!(key.key, Key::Up | Key::Down)
            && let Some(current) = self
                .ui
                .focused
                .and_then(|focus| focus.checked_sub(id + 1))
                .filter(|index| *index < count as u64)
        {
            let current = current as usize;
            let next = (1..=count)
                .map(|step| {
                    if key.key == Key::Down {
                        (current + step) % count
                    } else {
                        (current + count - step) % count
                    }
                })
                .find(|&index| actions[index].enabled);
            if let Some(index) = next {
                let top = index as f32 * 60.0;
                if top < state.scroll {
                    state.scroll = top;
                } else if top + 56.0 > state.scroll + items_window.height {
                    state.scroll = (top + 56.0 - items_window.height).min(maximum_scroll);
                }
                state.pending_focus = Some(index);
                self.request_focus(id);
                self.key_consumed = true;
                self.ui.continuous = true;
            }
        }
        let toggle = self.spatial(id, 68, f32::from(*open), 0.0).clamp(0.0, 1.0);
        let t = self.ui.theme;
        let radius = 16.0 + 12.0 * toggle;
        if r.enabled {
            self.elevation(trigger, radius, 3);
        }
        self.paint_rect(trigger, radius, t.primary_container);
        self.state_layer(r, radius, t.on_primary_container);
        self.focus_ring(r);
        let angle = toggle * std::f32::consts::FRAC_PI_4;
        let (sin, cos) = angle.sin_cos();
        let cx = trigger.x + 28.0;
        let cy = trigger.y + 28.0;
        let point = |x: f32, y: f32| [cx + x * cos - y * sin, cy + x * sin + y * cos];
        self.paint_line(
            point(-9.0, 0.0),
            point(9.0, 0.0),
            2.0,
            t.on_primary_container,
        );
        self.paint_line(
            point(0.0, -9.0),
            point(0.0, 9.0),
            2.0,
            t.on_primary_container,
        );
        let mut chosen = None;
        let previous_clip = self.clip;
        let content_clip = previous_clip.intersect(items_window);
        // Shadows need room outside the action's bounds, including the gap
        // above the trigger. Keep the interactive content in the viewport.
        self.clip = previous_clip.intersect(Rect {
            x: items_window.x - 8.0,
            width: items_window.width + 16.0,
            height: items_window.height + 8.0,
            ..items_window
        });
        let shadow_clip = self.clip;
        self.commands.push(DrawCommand::PushClip(self.clip));
        for (i, action) in actions.iter().enumerate() {
            let delay = (count - i - 1) as f64 * 0.045;
            let pending =
                *open && !self.reduced_motion() && self.ui.time - state.changed_at < delay;
            if pending {
                self.ui.continuous = true;
            }
            let amount = self
                .spatial(id + i as u64 + 1, 69, f32::from(*open && !pending), 0.0)
                .clamp(0.0, 1.0);
            if amount <= 0.001 {
                continue;
            }
            let full_width = (self.text_width(
                action.label,
                TextStyle {
                    size: 16.0,
                    weight: 500,
                    ..TextStyle::default()
                },
            ) + 80.0)
                .max(112.0)
                .min(bounds.width);
            let item = Rect {
                x: if options.rtl {
                    bounds.x
                } else {
                    bounds.x + bounds.width - full_width
                },
                y: items_window.y
                    + (items_window.height - content_height).max(0.0)
                    + i as f32 * 60.0
                    - state.scroll,
                width: full_width,
                height: 56.0,
            };
            let item_visible = item.intersect(content_clip);
            if item_visible.width <= 0.0 || item_visible.height <= 0.0 {
                continue;
            }
            self.clip = content_clip;
            let response = self.target(
                id + i as u64 + 1,
                item,
                action.label,
                *open && action.enabled,
                Role::Button,
            );
            self.clip = shadow_clip;
            if response.clicked {
                chosen = Some(i);
                *open = false;
                self.request_focus(id);
            }
            let clip = Rect {
                x: if options.rtl {
                    item.x
                } else {
                    item.x + item.width * (1.0 - amount)
                },
                width: item.width * amount,
                ..item
            };
            let partially_scrolled = item.y < content_clip.y
                || item.y + item.height > content_clip.y + content_clip.height;
            let draw_shadow = |f: &mut Self| {
                f.with_opacity(amount, |f| {
                    f.elevation(clip, 28.0_f32.min(clip.width / 2.0), 3);
                });
            };
            if partially_scrolled {
                self.within(content_clip, draw_shadow);
            } else {
                draw_shadow(self);
            }
            self.within(clip.intersect(content_clip), |f| {
                f.with_opacity(amount, |f| {
                    f.paint_rect(item, 28.0, t.primary_container);
                    f.state_layer(response, 28.0, t.on_primary_container);
                    f.styled_text(
                        Rect {
                            x: if options.rtl {
                                item.x + item.width - 48.0
                            } else {
                                item.x + 16.0
                            },
                            width: 32.0,
                            ..item
                        },
                        action.glyph,
                        t.on_primary_container,
                        TextStyle {
                            size: 20.0,
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                    f.styled_text(
                        Rect {
                            x: if options.rtl {
                                item.x + 16.0
                            } else {
                                item.x + 56.0
                            },
                            width: (item.width - 72.0).max(0.0),
                            ..item
                        },
                        action.label,
                        t.on_primary_container,
                        TextStyle {
                            size: 16.0,
                            weight: 500,
                            ..TextStyle::default()
                        },
                    );
                })
            });
            self.within(content_clip, |f| f.focus_ring(response));
        }
        self.commands.push(DrawCommand::PopClip);
        self.clip = previous_clip;
        if let Some(next) = state
            .pending_focus
            .map(|index| id + index as u64 + 1)
            .filter(|next| {
                self.ui
                    .semantics
                    .iter()
                    .any(|node| node.id == *next && node.enabled)
            })
        {
            self.request_focus(next);
            state.pending_focus = None;
            self.key_consumed = true;
            self.ui.continuous = true;
            for node in &mut self.ui.semantics {
                if node.id >= id && node.id <= id + count as u64 {
                    node.focused = node.id == next;
                }
            }
        }
        chosen
    }
}

#[cfg(test)]
#[path = "fab_menu.spec.rs"]
mod tests;
