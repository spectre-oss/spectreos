use crate::*;

/// Material 3 navigation tab styles (`Primary`: 3dp top-rounded label-width; `Secondary`: 2dp full-width).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabStyle {
    #[default]
    Primary,
    Secondary,
}

/// Tab descriptor pairing a label with an optional Material icon for `Frame::tabs_with_icons`.
#[derive(Clone, Copy, Debug)]
pub struct TabItem<'a> {
    pub label: &'a str,
    pub icon: Option<MaterialIcon>,
}

/// Configuration for Material 3 tabs rendered via `Frame::tabs_with_options`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabOptions {
    pub style: TabStyle,
    pub scrollable: bool,
    pub enabled: bool,
}
impl Default for TabOptions {
    fn default() -> Self {
        Self {
            style: TabStyle::Primary,
            scrollable: false,
            enabled: true,
        }
    }
}

impl Frame<'_> {
    /// A horizontal single-selection group. Tab enters the group; arrows move selection.
    pub fn tabs(
        &mut self,
        id: u64,
        labels: &[&str],
        selected: &mut usize,
        enabled: bool,
    ) -> Response {
        let b = self.allocate(49.0);
        let mut r = self.interact(id, b, enabled && !labels.is_empty());
        if labels.is_empty() {
            return r;
        }
        *selected = (*selected).min(labels.len() - 1);
        let old = *selected;
        if r.clicked
            && self.input.released
            && self.ui.capture == Capture::Pointer
            && let Some(p) = self.input.pointer
        {
            *selected = (((p[0] - b.x) / b.width.max(1.0)) * labels.len() as f32)
                .floor()
                .max(0.0) as usize;
            *selected = (*selected).min(labels.len() - 1);
        }
        if let Some(key) = self.key_for(r) {
            match key {
                Key::Right | Key::Down => *selected = (*selected + 1) % labels.len(),
                Key::Left | Key::Up => *selected = (*selected + labels.len() - 1) % labels.len(),
                Key::Home => *selected = 0,
                Key::End => *selected = labels.len() - 1,
                _ => {}
            }
        }
        r.changed = old != *selected;
        let t = self.ui.theme;
        self.rect(
            Rect {
                y: b.y + 48.0,
                height: 1.0,
                ..b
            },
            0.0,
            t.outline_variant,
        );
        let width = b.width / labels.len() as f32;
        let target = if labels.len() > 1 {
            *selected as f32 / (labels.len() - 1) as f32
        } else {
            0.0
        };
        let a = self.animate_geometry(id, 3, target, target);
        self.rect(
            Rect {
                x: b.x + a * (b.width - width),
                y: b.y + 46.0,
                width,
                height: 2.0,
            },
            0.0,
            t.primary.opacity(if r.enabled { 1.0 } else { 0.38 }),
        );
        self.semantic(r, Role::TabList, "Tabs", false, None, None, None);
        for (index, label) in labels.iter().enumerate() {
            let bounds = Rect {
                x: b.x + width * index as f32,
                width,
                ..b
            };
            if r.enabled {
                let hovered = self.input.pointer.is_some_and(|p| bounds.contains(p));
                let focused = r.focused && self.ui.keyboard_focus && index == *selected;
                let alpha = if focused {
                    0.12
                } else if hovered {
                    0.08
                } else {
                    0.0
                };
                if alpha > 0.0 {
                    self.rect(bounds, 0.0, t.primary.opacity(alpha));
                }
            }
            self.ink_part(
                Response {
                    bounds,
                    focused: r.focused && index == *selected,
                    ..r
                },
                index as u32 + 1,
                0.0,
                t.primary,
                false,
            );
            self.styled_text(
                Rect {
                    height: 48.0,
                    ..bounds
                },
                label,
                t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
                TextStyle::LABEL,
            );
            let child = Response {
                id,
                bounds,
                focused: r.focused && index == *selected,
                ..r
            };
            self.semantic(
                child,
                Role::Tab,
                label,
                index == *selected,
                None,
                None,
                None,
            );
            if let Some(node) = self.ui.semantics.last_mut() {
                node.part = index as u32 + 1;
            }
        }
        self.focus_ring(r);
        r
    }
    /// Material 3 navigation tabs supporting Primary (3dp top-rounded label-width indicator),
    /// Secondary (2dp full-tab-width indicator), and horizontal scrollable overflow.
    pub fn tabs_with_options(
        &mut self,
        id: u64,
        labels: &[&str],
        selected: &mut usize,
        options: TabOptions,
    ) -> Response {
        let b = self.allocate(49.0);
        if labels.is_empty() {
            return self.interact(id, b, false);
        }
        let mut widths = Vec::with_capacity(labels.len());
        let mut offsets = Vec::with_capacity(labels.len());
        let mut cursor_x = 0.0_f32;
        if options.scrollable {
            for label in labels {
                let w = (self.text_width(label, TextStyle::LABEL) + 32.0).clamp(72.0, 264.0);
                offsets.push(cursor_x);
                widths.push(w);
                cursor_x += w;
            }
        } else {
            let w = b.width / labels.len() as f32;
            for _ in labels {
                offsets.push(cursor_x);
                widths.push(w);
                cursor_x += w;
            }
        }
        let total_width = cursor_x.max(1.0);
        let max_scroll = if options.scrollable {
            (total_width - b.width).max(0.0)
        } else {
            0.0
        };
        let mut scroll_offset = if options.scrollable {
            self.ui
                .scrolls
                .get(&id)
                .map_or(0.0, |s| s.offset)
                .clamp(0.0, max_scroll)
        } else {
            0.0
        };
        let saved_input = self.input;
        let dragged = if options.scrollable && options.enabled && self.enabled {
            self.scroll_viewport_horizontal(id, b, &mut scroll_offset, max_scroll)
        } else {
            false
        };
        let mut r = self.interact(id, b, options.enabled);
        if dragged {
            self.input = saved_input;
        }
        *selected = (*selected).min(labels.len() - 1);
        let old = *selected;
        if r.clicked
            && self.input.released
            && self.ui.capture == Capture::Pointer
            && let Some(p) = self.input.pointer
        {
            let local_x = p[0] - b.x + scroll_offset;
            if let Some(idx) = offsets
                .iter()
                .zip(widths.iter())
                .position(|(&x, &w)| local_x >= x && local_x < x + w)
            {
                *selected = idx;
            } else if local_x >= total_width {
                *selected = labels.len() - 1;
            } else {
                *selected = 0;
            }
        }
        if let Some(key) = self.key_for(r) {
            match key {
                Key::Right | Key::Down => *selected = (*selected + 1) % labels.len(),
                Key::Left | Key::Up => *selected = (*selected + labels.len() - 1) % labels.len(),
                Key::Home => *selected = 0,
                Key::End => *selected = labels.len() - 1,
                _ => {}
            }
        }
        r.changed = old != *selected;
        if options.scrollable {
            if r.changed && max_scroll > 0.0 {
                let tab_x = offsets[*selected];
                let tab_w = widths[*selected];
                if tab_x < scroll_offset {
                    scroll_offset = tab_x;
                } else if tab_x + tab_w > scroll_offset + b.width {
                    scroll_offset = (tab_x + tab_w - b.width).min(max_scroll);
                }
            }
            scroll_offset = scroll_offset.clamp(0.0, max_scroll);
            self.ui.scrolls.entry(id).or_default().offset = scroll_offset;
        }
        let t = self.ui.theme;
        self.rect(
            Rect {
                y: b.y + 48.0,
                height: 1.0,
                ..b
            },
            0.0,
            t.outline_variant,
        );
        let sel_tab_x = offsets[*selected];
        let sel_tab_w = widths[*selected];
        let (target_ind_x, target_ind_w) = match options.style {
            TabStyle::Primary => {
                let label_w = self.text_width(labels[*selected], TextStyle::LABEL);
                let ind_w = label_w
                    .max(24.0)
                    .min((sel_tab_w - 4.0).max(0.0));
                (sel_tab_x + (sel_tab_w - ind_w) * 0.5, ind_w)
            }
            TabStyle::Secondary => (sel_tab_x, sel_tab_w),
        };
        let norm_x = (target_ind_x / total_width).clamp(0.0, 1.0);
        let norm_w = (target_ind_w / total_width).clamp(0.0, 1.0);
        let anim_x = self.animate_geometry(id, 3, norm_x, norm_x) * total_width;
        let anim_w = self.animate_geometry(id, 4, norm_w, norm_w) * total_width;
        let indicator_color = t.primary.opacity(if r.enabled { 1.0 } else { 0.38 });
        let bar_clip = b.intersect(self.clip);
        self.commands.push(DrawCommand::PushClip(bar_clip));
        match options.style {
            TabStyle::Primary => {
                let ind_clip = bar_clip.intersect(Rect {
                    x: b.x,
                    y: b.y + 45.0,
                    width: b.width,
                    height: 3.0,
                });
                self.commands.push(DrawCommand::PushClip(ind_clip));
                self.rect(
                    Rect {
                        x: b.x + anim_x - scroll_offset,
                        y: b.y + 45.0,
                        width: anim_w,
                        height: 6.0,
                    },
                    3.0,
                    indicator_color,
                );
                self.commands.push(DrawCommand::PopClip);
            }
            TabStyle::Secondary => {
                self.rect(
                    Rect {
                        x: b.x + anim_x - scroll_offset,
                        y: b.y + 46.0,
                        width: anim_w,
                        height: 2.0,
                    },
                    0.0,
                    indicator_color,
                );
            }
        }
        self.semantic(r, Role::TabList, "Tabs", false, None, None, None);
        for (index, label) in labels.iter().enumerate() {
            let bounds = Rect {
                x: b.x + offsets[index] - scroll_offset,
                y: b.y,
                width: widths[index],
                height: 48.0,
            };
            let is_selected = index == *selected;
            let state_color = match options.style {
                TabStyle::Primary if is_selected => t.primary,
                _ => t.text,
            };
            if r.enabled {
                let hovered = self
                    .input
                    .pointer
                    .is_some_and(|p| bar_clip.contains(p) && bounds.contains(p));
                let focused = r.focused && self.ui.keyboard_focus && is_selected;
                let alpha = if focused {
                    0.12
                } else if hovered {
                    0.08
                } else {
                    0.0
                };
                if alpha > 0.0 {
                    self.rect(bounds, 0.0, state_color.opacity(alpha));
                }
            }
            self.ink_part(
                Response {
                    bounds,
                    focused: r.focused && is_selected,
                    ..r
                },
                index as u32 + 1,
                0.0,
                state_color,
                false,
            );
            let label_color = if !r.enabled {
                t.text.opacity(0.38)
            } else if is_selected {
                match options.style {
                    TabStyle::Primary => t.primary,
                    TabStyle::Secondary => t.text,
                }
            } else {
                t.on_surface_variant
            };
            self.styled_text(bounds, label, label_color, TextStyle::LABEL);
            let child = Response {
                id,
                bounds,
                focused: r.focused && is_selected,
                ..r
            };
            self.semantic(child, Role::Tab, label, is_selected, None, None, None);
            if let Some(node) = self.ui.semantics.last_mut() {
                node.part = index as u32 + 1;
            }
        }
        self.commands.push(DrawCommand::PopClip);
        self.focus_ring(r);
        r
    }

    /// Primary Material 3 tabs with stacked top icon + label, `Role::TabList` container semantics,
    /// arrow/Home/End keyboard navigation, and animated 3dp bottom indicator.
    pub fn tabs_with_icons(
        &mut self,
        id: u64,
        items: &[TabItem<'_>],
        selected: &mut usize,
        enabled: bool,
    ) -> bool {
        self.tabs_with_icons_options(id, items, selected, enabled, TabOptions::default())
    }

    /// Material 3 icon + label tabs with configurable `TabOptions` (primary/secondary indicator, RTL).
    pub fn tabs_with_icons_options(
        &mut self,
        id: u64,
        items: &[TabItem<'_>],
        selected: &mut usize,
        enabled: bool,
        options: TabOptions,
    ) -> bool {
        if items.is_empty() {
            return false;
        }
        let has_icons = items.iter().any(|it| it.icon.is_some());
        let bar = self.reserve(if has_icons { 64.0 } else { 48.0 });
        self.tabs_with_icons_at(id, bar, items, selected, enabled, options)
    }

    /// Positioned Material 3 icon + label tabs inside caller-supplied `bounds`, with `Role::TabList`
    /// semantics, roving arrow/Home/End navigation, `TabOptions` styling, and animated indicator.
    pub fn tabs_with_icons_at(
        &mut self,
        id: u64,
        bar: Rect,
        items: &[TabItem<'_>],
        selected: &mut usize,
        enabled: bool,
        options: TabOptions,
    ) -> bool {
        if items.is_empty() {
            return false;
        }
        let count = items.len();
        *selected = (*selected).min(count - 1);
        let t = self.theme();
        self.paint_rect(bar, 0.0, t.surface_low);
        self.paint_rect(Rect { y: bar.y + bar.height - 1.0, height: 1.0, ..bar }, 0.0, t.outline_variant);
        let cell_w = bar.width / count as f32;
        let mut changed = false;
        let mut nav_handled = false;

        let enabled = enabled && options.enabled;
        self.semantic(
            Response { id: id.wrapping_add(0x5441_424c), bounds: bar, enabled: enabled && self.enabled, ..Response::default() },
            Role::TabList,
            "Tabs",
            false,
            None,
            None,
            None,
        );

        for (i, item) in items.iter().enumerate() {
            let cell = Rect { x: bar.x + i as f32 * cell_w, width: cell_w, ..bar };
            let is_sel = i == *selected;
            let tab_id = id.wrapping_add(i as u64);
            let r = self.interact(tab_id, cell, enabled);
            if !is_sel && !r.focused && !r.clicked && self.ui.next_focus_order.last() == Some(&tab_id) {
                self.ui.next_focus_order.pop();
            }
            let fg = if !r.enabled {
                t.text.opacity(0.38)
            } else if is_sel {
                match options.style {
                    TabStyle::Primary => t.primary,
                    TabStyle::Secondary => t.text,
                }
            } else {
                t.on_surface_variant
            };
            self.state_layer(r, 0.0, fg);
            self.focus_ring(r);
            if let Some(icon) = item.icon {
                let icon_rect = Rect {
                    x: cell.x + (cell.width - 24.0) * 0.5,
                    y: cell.y + 10.0,
                    width: 24.0,
                    height: 24.0,
                };
                self.paint_material_icon(icon_rect, icon, fg);
                self.styled_text(
                    Rect { y: cell.y + 36.0, height: 20.0, ..cell },
                    item.label,
                    fg,
                    TextStyle { size: 12.0, weight: 500, align: 1, ..TextStyle::default() },
                );
            } else {
                self.styled_text(cell, item.label, fg, TextStyle::LABEL);
            }
            self.semantic(r, Role::Tab, item.label, is_sel, None, None, None);
            if r.clicked && *selected != i {
                *selected = i;
                changed = true;
            }
            if r.focused && r.enabled && !nav_handled {
                let next = if self.pressed_key(Key::Right) {
                    Some((i + 1) % count)
                } else if self.pressed_key(Key::Left) {
                    Some((i + count - 1) % count)
                } else if self.pressed_key(Key::Home) {
                    Some(0)
                } else if self.pressed_key(Key::End) {
                    Some(count - 1)
                } else {
                    None
                };
                if let Some(target_idx) = next {
                    if *selected != target_idx {
                        *selected = target_idx;
                        changed = true;
                    }
                    self.request_focus(id.wrapping_add(target_idx as u64));
                    nav_handled = true;
                }
            }
        }
        let anim_x = self.animate_geometry(id, 88, *selected as f32, *selected as f32);
        let ind_w = match options.style {
            TabStyle::Primary => (cell_w * 0.5).clamp(24.0, cell_w.max(24.0)),
            TabStyle::Secondary => cell_w,
        };
        let ind_h = match options.style {
            TabStyle::Primary => 3.0,
            TabStyle::Secondary => 2.0,
        };
        let ind_x = bar.x + anim_x * cell_w + (cell_w - ind_w) * 0.5;
        self.paint_rect(
            Rect { x: ind_x, y: bar.y + bar.height - ind_h, width: ind_w, height: ind_h },
            if options.style == TabStyle::Primary { 1.5 } else { 0.0 },
            t.primary,
        );
        changed
    }
}

#[cfg(test)]
#[path = "tabs.spec.rs"]
mod tests;
