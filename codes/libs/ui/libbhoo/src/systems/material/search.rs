use crate::*;

/// Caller-owned state for a Material 3 SearchBar -> SearchView container-transform overlay.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchOverlayState {
    pub open: bool,
    pub query: String,
    pub selected: Option<usize>,
}

impl Frame<'_> {
    /// Material 3 SearchBar -> SearchView container-transform overlay.
    /// Renders a 56dp pill SearchBar when closed, expanding into a 28dp `surfaceContainerHigh`
    /// SearchView overlay with back button, clear button, divider, and clickable filtered
    /// suggestion history rows when tapped. Returns the clicked suggestion index into `suggestions`.
    pub fn search_bar_overlay(
        &mut self,
        id: u64,
        state: &mut SearchOverlayState,
        placeholder: &str,
        suggestions: &[&str],
    ) -> Option<usize> {
        let bar_bounds = self.allocate(56.0);
        let t = self.ui.theme;
        let was_open = state.open;
        let mut just_opened = false;
        let sub_id = |slot: u64| {
            let v = id.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(slot);
            if v == id || v == 0 {
                v.wrapping_add(slot.max(1))
            } else {
                v
            }
        };

        let paint_closed_bar = |f: &mut Self, state: &mut SearchOverlayState, interactive: bool| -> bool {
            let radius = (bar_bounds.height * 0.5).min(28.0);
            let bar = f.interact(id, bar_bounds, interactive);
            f.rect(bar_bounds, radius, t.surface_high);
            f.state_layer(bar, radius, t.text);
            f.ink(bar, radius, t.text);
            let icon_rect = Rect {
                x: bar_bounds.x + 16.0,
                y: bar_bounds.y + (bar_bounds.height - 24.0) * 0.5,
                width: 24.0,
                height: 24.0,
            };
            f.paint_material_icon(icon_rect, MaterialIcon::Search, t.text);
            let text_rect = Rect {
                x: bar_bounds.x + 56.0,
                y: bar_bounds.y,
                width: (bar_bounds.width - 72.0).max(0.0),
                height: bar_bounds.height,
            };
            let show_query = !state.query.is_empty();
            f.styled_text(
                text_rect,
                if show_query {
                    state.query.as_str()
                } else {
                    placeholder
                },
                if show_query {
                    t.text
                } else {
                    t.on_surface_variant
                },
                TextStyle {
                    size: 16.0,
                    ..TextStyle::default()
                },
            );
            f.focus_ring(bar);
            f.semantic(bar, Role::Button, placeholder, false, None, None, None);
            bar.clicked
        };

        if !was_open {
            if paint_closed_bar(self, state, true) {
                state.open = true;
                just_opened = true;
                self.ui.active = None;
                self.ui.continuous = true;
                if let Some(node) = self.ui.semantics.last_mut() {
                    node.enabled = false;
                }
            }
        } else if !self.key_consumed && self.pressed_key(Key::Escape) {
            state.open = false;
            self.key_consumed = true;
            self.ui.active = None;
            self.ui.continuous = true;
        } else if !self.key_consumed && self.enabled {
            if let Some(k) = self.input.key
                && k.pressed
                && let Key::Character(ch) = k.key
            {
                if ch == '\u{8}' || ch == '\u{7f}' {
                    state.query.pop();
                    self.key_consumed = true;
                    self.ui.continuous = true;
                } else if !ch.is_control() {
                    state.query.push(ch);
                    self.key_consumed = true;
                    self.ui.continuous = true;
                }
            }
        }

        let mut progress = self
            .spatial(id, 21, f32::from(state.open), f32::from(was_open))
            .clamp(0.0, 1.0);

        if was_open && !state.open && progress <= 0.001 {
            paint_closed_bar(self, state, false);
            return None;
        }

        let mut picked = None;
        if state.open || progress > 0.001 {
            let needle = state.query.trim().to_ascii_lowercase();
            let filtered: Vec<(usize, &str)> = suggestions
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    needle.is_empty() || item.to_ascii_lowercase().contains(&needle)
                })
                .map(|(idx, item)| (idx, *item))
                .collect();

            let vp = self.viewport();
            let header_h = 56.0;
            let row_h = 48.0;
            let avail_h = (vp.y + vp.height - bar_bounds.y).max(bar_bounds.height);
            let rows_h = (filtered.len().max(1) as f32) * row_h + 8.0;
            let target_h = (header_h + 1.0 + rows_h)
                .min(avail_h)
                .max(bar_bounds.height);
            let current_rect = Rect {
                x: bar_bounds.x,
                y: bar_bounds.y,
                width: bar_bounds.width,
                height: bar_bounds.height + (target_h - bar_bounds.height) * progress,
            };

            if was_open
                && state.open
                && self.input.pressed
                && self.input.pointer.is_some_and(|p| !current_rect.contains(p))
            {
                state.open = false;
                self.ui.active = None;
                self.ui.continuous = true;
                progress = self.spatial(id, 21, 0.0, 1.0).clamp(0.0, 1.0);
                if progress <= 0.001 {
                    paint_closed_bar(self, state, false);
                    return None;
                }
            }

            let interactive = state.open && !just_opened;
            let saved_clip = self.clip;
            self.clip = vp;
            let overlay_start = self.commands.len();
            if progress > 0.001 {
                self.rect(vp, 0.0, Color(0, 0, 0, 255).opacity(0.32 * progress));
            }
            self.elevation(current_rect, 28.0, 3);
            self.rect(current_rect, 28.0, t.surface_high);
            self.within(current_rect, |f| {
                let back_rect = Rect {
                    x: current_rect.x + 8.0,
                    y: current_rect.y + 8.0,
                    width: 40.0,
                    height: 40.0,
                };
                let back = f.interact(sub_id(1), back_rect, interactive);
                f.state_layer(back, 20.0, t.text);
                f.ink(back, 20.0, t.text);
                f.paint_material_icon(back_rect.inset(8.0), MaterialIcon::ArrowBack, t.text);
                f.focus_ring(back);
                f.semantic(back, Role::Button, "Back", false, None, None, None);
                if back.clicked {
                    state.open = false;
                    f.ui.active = None;
                    f.ui.continuous = true;
                }

                let has_query = !state.query.is_empty();
                let mut cleared = false;
                if has_query {
                    let clear_rect = Rect {
                        x: (current_rect.x + current_rect.width - 48.0)
                            .max(back_rect.x + back_rect.width),
                        y: current_rect.y + 8.0,
                        width: 40.0,
                        height: 40.0,
                    };
                    let clear = f.interact(sub_id(3), clear_rect, interactive && state.open);
                    f.state_layer(clear, 20.0, t.on_surface_variant);
                    f.ink(clear, 20.0, t.on_surface_variant);
                    f.paint_material_icon(
                        clear_rect.inset(8.0),
                        MaterialIcon::Close,
                        t.on_surface_variant,
                    );
                    f.focus_ring(clear);
                    f.semantic(clear, Role::Button, "Clear", false, None, None, None);
                    if clear.clicked {
                        state.query.clear();
                        state.selected = None;
                        cleared = true;
                        f.ui.continuous = true;
                    }
                }

                let trailing_w = if has_query && !cleared { 56.0 } else { 16.0 };
                let input_rect = Rect {
                    x: current_rect.x + 56.0,
                    y: current_rect.y,
                    width: (current_rect.width - 56.0 - trailing_w).max(0.0),
                    height: header_h,
                };
                let field = f.interact(sub_id(2), input_rect, interactive && state.open);
                let show_query = !state.query.is_empty();
                f.styled_text(
                    input_rect,
                    if show_query {
                        state.query.as_str()
                    } else {
                        placeholder
                    },
                    if show_query {
                        t.text
                    } else {
                        t.on_surface_variant
                    },
                    TextStyle {
                        size: 16.0,
                        ..TextStyle::default()
                    },
                );
                f.focus_ring(field);
                f.semantic(field, Role::TextField, placeholder, false, None, None, None);

                if current_rect.height > header_h + 0.5 {
                    f.divider_at(
                        Rect {
                            x: current_rect.x,
                            y: current_rect.y + header_h,
                            width: current_rect.width,
                            height: 1.0,
                        },
                        DividerOrientation::Horizontal,
                        0.0,
                        0.0,
                    );

                    if filtered.is_empty() {
                        let empty_rect = Rect {
                            x: current_rect.x + 16.0,
                            y: current_rect.y + header_h + 1.0,
                            width: (current_rect.width - 32.0).max(0.0),
                            height: row_h,
                        };
                        f.styled_text(
                            empty_rect,
                            "No matching suggestions",
                            t.on_surface_variant,
                            TextStyle {
                                size: 14.0,
                                ..TextStyle::default()
                            },
                        );
                    } else {
                        for (row_idx, &(orig_idx, suggestion)) in filtered.iter().enumerate() {
                            let row_y = current_rect.y + header_h + 1.0 + row_idx as f32 * row_h;
                            if row_y >= current_rect.y + current_rect.height {
                                break;
                            }
                            let row_rect = Rect {
                                x: current_rect.x,
                                y: row_y,
                                width: current_rect.width,
                                height: row_h,
                            };
                            let row = f.interact(
                                sub_id(100 + orig_idx as u64),
                                row_rect,
                                interactive && state.open,
                            );
                            let is_selected = state.selected == Some(orig_idx);
                            if is_selected {
                                f.rect(row_rect, 0.0, t.secondary.opacity(0.35));
                            }
                            f.state_layer(row, 0.0, t.text);
                            f.ink(row, 0.0, t.text);
                            let icon_rect = Rect {
                                x: row_rect.x + 16.0,
                                y: row_rect.y + (row_h - 20.0) * 0.5,
                                width: 20.0,
                                height: 20.0,
                            };
                            f.paint_material_icon(
                                icon_rect,
                                MaterialIcon::Search,
                                t.on_surface_variant,
                            );
                            let label_rect = Rect {
                                x: row_rect.x + 56.0,
                                y: row_rect.y,
                                width: (row_rect.width - 72.0).max(0.0),
                                height: row_h,
                            };
                            f.styled_text(
                                label_rect,
                                suggestion,
                                t.text,
                                TextStyle {
                                    size: 16.0,
                                    ..TextStyle::default()
                                },
                            );
                            f.focus_ring(row);
                            f.semantic(
                                row,
                                Role::ListItem,
                                suggestion,
                                is_selected,
                                None,
                                None,
                                None,
                            );
                            if row.clicked {
                                picked = Some(orig_idx);
                                state.selected = Some(orig_idx);
                                state.query.clear();
                                state.query.push_str(suggestion);
                                state.open = false;
                                f.ui.active = None;
                                f.ui.continuous = true;
                            }
                        }
                    }
                }
            });
            self.ui.overlays.extend(self.commands.drain(overlay_start..));
            self.clip = saved_clip;
        }
        self.ui.overlay_active |= state.open;
        picked
    }

    /// Inline docked search bar (top app bar search mode or drawer search pill) with optional
    /// leading back button, query/placeholder display, blinking caret when focused, and clear button.
    /// Returns `(field_response, back_clicked, clear_clicked)`.
    pub fn inline_search_bar(
        &mut self,
        id: u64,
        bounds: Rect,
        placeholder: &str,
        query: &str,
        show_back: bool,
        enabled: bool,
    ) -> (Response, bool, bool) {
        let t = self.theme();
        let rad = (bounds.height * 0.5).min(28.0);
        self.paint_rect(bounds, rad, t.surface_high);
        let mut back_clicked = false;
        let mut clear_clicked = false;
        let btn_sz = 40.0_f32.min(bounds.height);
        let btn_y = bounds.y + (bounds.height - btn_sz) * 0.5;
        let lead_x = bounds.x + 8.0;
        if show_back {
            let br = self.material_icon_button_at(
                id.wrapping_add(1),
                Rect { x: lead_x, y: btn_y, width: btn_sz, height: btn_sz },
                MaterialIcon::ArrowBack,
                "Back",
                Options { enabled, ..Options::default() },
            );
            back_clicked = br.clicked;
        } else {
            let icon_rect = Rect { x: lead_x + 8.0, y: bounds.y + (bounds.height - 22.0) * 0.5, width: 22.0, height: 22.0 };
            self.paint_material_icon(icon_rect, MaterialIcon::Search, t.on_surface_variant);
        }
        let has_clear = !query.is_empty();
        let trail_space = if has_clear { btn_sz + 8.0 } else { 16.0 };
        let field_rect = Rect {
            x: lead_x + btn_sz + 4.0,
            y: bounds.y,
            width: (bounds.width - btn_sz - trail_space - 12.0).max(0.0),
            height: bounds.height,
        };
        let field_r = self.target(id, field_rect, placeholder, enabled, Role::TextField);
        let text_style = TextStyle { size: 15.0, ..TextStyle::default() };
        if query.is_empty() {
            self.styled_text(field_rect, placeholder, t.on_surface_variant, text_style);
        } else {
            self.styled_text(field_rect, query, t.text, text_style);
        }
        if field_r.focused && enabled {
            let blink_on = ((self.time() * 2.0) as u64).is_multiple_of(2) || self.reduced_motion();
            if blink_on {
                let cx = field_rect.x + self.text_width(query, text_style).min(field_rect.width - 2.0);
                self.paint_rect(
                    Rect { x: cx, y: field_rect.y + (field_rect.height - 20.0) * 0.5, width: 2.0, height: 20.0 },
                    1.0,
                    t.primary,
                );
            }
            if !self.reduced_motion() {
                self.wake_at((self.time() * 2.0).floor() * 0.5 + 0.5);
            }
        }
        if has_clear {
            let cr = self.material_icon_button_at(
                id.wrapping_add(2),
                Rect { x: bounds.x + bounds.width - btn_sz - 8.0, y: btn_y, width: btn_sz, height: btn_sz },
                MaterialIcon::Close,
                "Clear search",
                Options { enabled, ..Options::default() },
            );
            clear_clicked = cr.clicked;
        }
        (field_r, back_clicked, clear_clicked)
    }
}

#[cfg(test)]
#[path = "search.spec.rs"]
mod tests;
