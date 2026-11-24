use crate::*;

#[derive(Clone, Debug, Default)]
pub struct ListState {
    pub offset: f32,
    pub selected: Option<usize>,
}

impl Frame<'_> {
    pub fn list_item(&mut self, id: u64, label: &str, selected: bool, enabled: bool) -> Response {
        let b = self.allocate(48.0);
        let r = self.interact(id, b, enabled);
        let t = self.ui.theme;
        let amount = self.animate(
            id,
            3,
            if selected { 1.0 } else { 0.0 },
            if selected { 1.0 } else { 0.0 },
        );
        let base = t.background.mix(t.secondary, amount);
        let feedback = if self.ui.motion_preference == MotionPreference::Full {
            Response {
                pressed: false,
                ..r
            }
        } else {
            r
        };
        let color = self.control_color(feedback, base);
        self.rect(b, 0.0, color);
        self.ink(r, 0.0, t.text);
        self.styled_text(
            b,
            label,
            t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
            TextStyle {
                padding: 16.0,
                letter_spacing: 0.5,
                ..TextStyle::default()
            },
        );
        self.focus_ring(r);
        self.semantic(r, Role::ListItem, label, selected, None, None, None);
        r
    }
    pub fn selection_item(
        &mut self,
        id: u64,
        label: &str,
        selected: bool,
        enabled: bool,
    ) -> Response {
        let r = self.list_item(id, label, false, enabled);
        let t = self.ui.theme;
        let b = Rect {
            x: r.bounds.x + r.bounds.width - 38.0,
            y: r.bounds.y + (r.bounds.height - 20.0) * 0.5,
            width: 20.0,
            height: 20.0,
        };
        let color = if selected {
            t.primary
        } else {
            t.on_surface_variant
        }
        .opacity(if r.enabled { 1.0 } else { 0.38 });
        self.rect(b, 10.0, color);
        self.rect(b.inset(2.0), 8.0, t.background);
        if selected {
            self.rect(b.inset(5.0), 5.0, color);
        }
        if let Some(node) = self.ui.semantics.last_mut() {
            node.selected = selected;
        }
        r
    }
    /// Only calls build for visible rows. Use stable row IDs and one 40px list_item per row.
    pub fn virtual_list(
        &mut self,
        id: u64,
        height: f32,
        count: usize,
        state: &mut ListState,
        build: impl FnMut(&mut Self, usize, bool) -> Response,
    ) -> Response {
        self.virtual_list_inner(id, height, count, state, 0.0, 40.0, build)
    }
    pub fn virtual_list_padded(
        &mut self,
        id: u64,
        height: f32,
        count: usize,
        state: &mut ListState,
        build: impl FnMut(&mut Self, usize, bool) -> Response,
    ) -> Response {
        self.virtual_list_inner(id, height, count, state, 8.0, 40.0, build)
    }
    /// Virtual list with an explicit row height, for components such as option panels.
    pub fn virtual_list_rows(
        &mut self,
        id: u64,
        height: f32,
        count: usize,
        state: &mut ListState,
        row_height: f32,
        build: impl FnMut(&mut Self, usize, bool) -> Response,
    ) -> Response {
        self.virtual_list_inner(id, height, count, state, 0.0, row_height.max(1.0), build)
    }
    #[allow(clippy::too_many_arguments)]
    fn virtual_list_inner(
        &mut self,
        id: u64,
        height: f32,
        count: usize,
        state: &mut ListState,
        padding: f32,
        row_height: f32,
        mut build: impl FnMut(&mut Self, usize, bool) -> Response,
    ) -> Response {
        let b = self.allocate(height.max(0.0));
        if !state.offset.is_finite() {
            state.offset = 0.0;
        }
        let old_input = self.input;
        self.input.pressed = false;
        self.input.released = false;
        let mut r = self.interact(id, b, true);
        self.input = old_input;
        let previous_selection = state.selected;
        let max_offset = (count as f32 * row_height + 2.0 * padding - b.height).max(0.0);
        self.scroll_viewport(id, b, &mut state.offset, max_offset);
        if let Some(key) = self.key_for(r)
            && count > 0
        {
            let index = state.selected.unwrap_or(0).min(count - 1);
            state.selected = Some(match key {
                Key::Down => (index + 1).min(count - 1),
                Key::Up => index.saturating_sub(1),
                Key::Home => 0,
                Key::End => count - 1,
                _ => index,
            });
            if let Some(selected) = state.selected {
                let top = padding + selected as f32 * row_height;
                if top < state.offset {
                    state.offset = top;
                } else if top + row_height > state.offset + b.height {
                    state.offset = top + row_height - b.height;
                }
            }
        }
        // scroll_viewport permits a bounded elastic edge displacement.
        if state.selected.is_some_and(|i| i >= count) {
            state.selected = count.checked_sub(1);
        }
        self.semantic(r, Role::ListBox, "List", false, None, None, None);
        let first = ((state.offset - padding).max(0.0) / row_height).floor() as usize;
        let last = ((state.offset + b.height - padding).max(0.0) / row_height).ceil() as usize;
        let old_gap = self.gap;
        self.gap = 0.0;
        // Allocate each visible row from its true content position without building hidden rows.
        self.within(b, |frame| {
            let rows_start = frame.ui.semantics.len();
            for index in first..last.min(count) {
                frame.cursor = b.y + padding + index as f32 * row_height - state.offset;
                frame.bounds = Rect {
                    y: frame.cursor,
                    height: row_height,
                    ..b
                };
                let row_start = frame.ui.semantics.len();
                let row = build(frame, index, state.selected == Some(index));
                if row.clicked {
                    state.selected = Some(index);
                    r.clicked = true;
                    frame.ui.continuous = true;
                    for node in &mut frame.ui.semantics[rows_start..row_start] {
                        if node.role == Role::ListItem {
                            node.selected = false;
                        }
                    }
                }
                if let Some(node) = frame.ui.semantics[row_start..]
                    .iter_mut()
                    .find(|node| node.id == row.id && node.role == Role::ListItem)
                {
                    node.selected = state.selected == Some(index);
                }
            }
        });
        self.gap = old_gap;
        r.changed = state.selected != previous_selection;
        r
    }
}

impl Frame<'_> {
    /// Fixed-cell grid; builds only visible cells and keeps selection by logical index.
    pub fn virtual_grid(
        &mut self,
        id: u64,
        height: f32,
        columns: usize,
        count: usize,
        state: &mut ListState,
        mut build: impl FnMut(&mut Self, usize, bool) -> Response,
    ) -> Response {
        assert!(columns > 0);
        let b = self.allocate(height.max(0.0));
        let cell_height = 64.0;
        let rows = count.div_ceil(columns);
        let max = (rows as f32 * cell_height - b.height).max(0.0);
        let old = self.input;
        self.input.pressed = false;
        self.input.released = false;
        let mut r = self.interact(id, b, true);
        self.input = old;
        if r.enabled {
            self.scroll_viewport(id, b, &mut state.offset, max);
        }
        let previous = state.selected;
        if count > 0
            && let Some(key) = self.key_for(r)
        {
            let i = state.selected.unwrap_or(0).min(count - 1);
            state.selected = Some(match key {
                Key::Down => (i + columns).min(count - 1),
                Key::Up => i.saturating_sub(columns),
                Key::Right => (i + 1).min(count - 1),
                Key::Left => i.saturating_sub(1),
                Key::Home => 0,
                Key::End => count - 1,
                _ => i,
            });
            if let Some(i) = state.selected {
                let y = (i / columns) as f32 * cell_height;
                if y < state.offset {
                    state.offset = y;
                } else if y + cell_height > state.offset + b.height {
                    state.offset = y + cell_height - b.height;
                }
            }
        }
        state.offset = if state.offset.is_finite() {
            state.offset.clamp(-b.height*0.12, max+b.height*0.12)
        } else {
            0.0
        };
        if state.selected.is_some_and(|i| i >= count) {
            state.selected = count.checked_sub(1);
        }
        self.semantic(
            r,
            Role::ListBox,
            "Application grid",
            false,
            None,
            None,
            None,
        );
        let first = (state.offset / cell_height).floor() as usize;
        let last = ((state.offset + b.height) / cell_height).ceil() as usize;
        self.within(b, |f| {
            for row in first..last.min(rows) {
                for col in 0..columns {
                    let i = row * columns + col;
                    if i >= count {
                        break;
                    }
                    let cell = Rect {
                        x: b.x + col as f32 * b.width / columns as f32,
                        y: b.y + row as f32 * cell_height - state.offset,
                        width: b.width / columns as f32,
                        height: cell_height,
                    };
                    f.within(cell.inset(3.0), |f| {
                        if build(f, i, state.selected == Some(i)).clicked {
                            state.selected = Some(i);
                            f.ui.continuous = true;
                        }
                    });
                }
            }
        });
        r.changed = state.selected != previous;
        r
    }

    /// Two-line Material list/preference row with optional leading `MaterialIcon`, title,
    /// secondary subtitle, and optional trailing chevron.
    pub fn list_item_two_line(
        &mut self,
        id: u64,
        icon: Option<MaterialIcon>,
        title: &str,
        subtitle: &str,
        show_chevron: bool,
        enabled: bool,
    ) -> Response {
        let b = self.allocate(if subtitle.is_empty() { 56.0 } else { 68.0 });
        let r = self.target(id, b, title, enabled, Role::ListItem);
        let t = self.theme();
        self.state_layer(r, 8.0, t.text);
        self.focus_ring(r);
        let fg = if r.enabled { t.text } else { t.text.opacity(0.38) };
        let sub_fg = if r.enabled { t.on_surface_variant } else { t.text.opacity(0.38) };
        let mut x = b.x + 16.0;
        if let Some(ic) = icon {
            let icon_rect = Rect { x, y: b.y + (b.height - 24.0) * 0.5, width: 24.0, height: 24.0 };
            self.paint_material_icon(icon_rect, ic, sub_fg);
            x += 40.0;
        }
        let trail = if show_chevron { 36.0 } else { 16.0 };
        let text_w = (b.x + b.width - x - trail).max(0.0);
        if subtitle.is_empty() {
            self.styled_text(Rect { x, width: text_w, ..b }, title, fg, TextStyle { size: 16.0, ..TextStyle::default() });
        } else {
            self.styled_text(Rect { x, y: b.y + 12.0, width: text_w, height: 22.0 }, title, fg, TextStyle { size: 16.0, weight: 500, ..TextStyle::default() });
            self.styled_text(Rect { x, y: b.y + 36.0, width: text_w, height: 20.0 }, subtitle, sub_fg, TextStyle { size: 13.5, ..TextStyle::default() });
        }
        if show_chevron {
            let cx = b.x + b.width - 20.0;
            let cy = b.y + b.height * 0.5;
            self.paint_line([cx - 2.5, cy - 4.5], [cx + 2.0, cy], 1.8, sub_fg);
            self.paint_line([cx + 2.0, cy], [cx - 2.5, cy + 4.5], 1.8, sub_fg);
        }
        r
    }

    /// Two-line settings preference row with title, secondary summary, and trailing Material switch.
    pub fn preference_switch_row(
        &mut self,
        id: u64,
        title: &str,
        subtitle: &str,
        value: &mut bool,
        enabled: bool,
    ) -> Response {
        let b = self.allocate(if subtitle.is_empty() { 56.0 } else { 68.0 });
        let mut r = self.interact(id, b, enabled);
        if r.clicked {
            *value = !*value;
            r.changed = true;
        }
        let t = self.theme();
        self.state_layer(r, 8.0, t.text);
        self.focus_ring(r);
        let fg = if r.enabled { t.text } else { t.text.opacity(0.38) };
        let sub_fg = if r.enabled { t.on_surface_variant } else { t.text.opacity(0.38) };
        let sw_w = 52.0;
        let sw_h = 32.0;
        let text_w = (b.width - sw_w - 40.0).max(0.0);
        if subtitle.is_empty() {
            self.styled_text(Rect { x: b.x + 16.0, width: text_w, ..b }, title, fg, TextStyle { size: 16.0, ..TextStyle::default() });
        } else {
            self.styled_text(Rect { x: b.x + 16.0, y: b.y + 12.0, width: text_w, height: 22.0 }, title, fg, TextStyle { size: 16.0, weight: 500, ..TextStyle::default() });
            self.styled_text(Rect { x: b.x + 16.0, y: b.y + 36.0, width: text_w, height: 20.0 }, subtitle, sub_fg, TextStyle { size: 13.5, ..TextStyle::default() });
        }
        let pos = self.animate_geometry(id, 4, f32::from(*value), f32::from(*value)).clamp(0.0, 1.0);
        let track = Rect { x: b.x + b.width - sw_w - 16.0, y: b.y + (b.height - sw_h) * 0.5, width: sw_w, height: sw_h };
        let track_col = if !r.enabled {
            t.text.opacity(0.12)
        } else {
            t.surface_highest.mix(t.primary, pos)
        };
        self.paint_rect(track, sw_h * 0.5, track_col);
        if !*value {
            self.commands.push(DrawCommand::Outline {
                bounds: track,
                radius: sw_h * 0.5,
                width: 2.0,
                color: if r.enabled { t.outline } else { t.text.opacity(0.12) },
            });
        }
        let thumb_d = 16.0 + 8.0 * pos;
        let thumb_cx = track.x + 16.0 + pos * (sw_w - 32.0);
        let thumb = Rect {
            x: thumb_cx - thumb_d * 0.5,
            y: track.y + (sw_h - thumb_d) * 0.5,
            width: thumb_d,
            height: thumb_d,
        };
        let thumb_col = if !r.enabled {
            t.text.opacity(0.38)
        } else {
            t.outline.mix(t.on_primary, pos)
        };
        self.paint_rect(thumb, thumb_d * 0.5, thumb_col);
        self.semantic(
            r,
            Role::Switch,
            title,
            *value,
            Some(if *value { CheckState::Checked } else { CheckState::Unchecked }),
            None,
            None,
        );
        r
    }
}

#[cfg(test)]
#[path = "list.spec.rs"]
mod tests;
