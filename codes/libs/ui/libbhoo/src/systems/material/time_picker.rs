//! Material / Angular-compatible popover time picker (dropdown slot list or clock dial, 12h or 24h).
use crate::*;

/// Presentation mode for `Frame::time_picker_with_options`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimePickerMode {
    #[default]
    Dropdown,
    Dial,
}

/// Configuration for `Frame::time_picker_with_options`.
#[derive(Clone, Copy, Debug, Default)]
pub struct TimePickerOptions {
    pub mode: TimePickerMode,
    pub use_24h: bool,
}

/// Caller-owned state for `Frame::time_picker` and `Frame::time_picker_with_options`.
pub struct TimePickerState {
    pub selected: usize,
    pub overlay: OverlayState,
    pub anchor: Rect,
    pub list: ListState,
    pub scrollbar: ScrollbarDragState,
    pub step_minutes: u16,
}

impl Default for TimePickerState {
    fn default() -> Self {
        Self {
            selected: 18, // 9:00 AM
            overlay: OverlayState::default(),
            anchor: Rect::default(),
            list: ListState { offset: 0.0, selected: Some(18) },
            scrollbar: ScrollbarDragState::default(),
            step_minutes: 30,
        }
    }
}

impl TimePickerState {
    pub fn slot_count(&self) -> usize {
        let step = self.step_minutes.clamp(1, 720) as usize;
        (24 * 60) / step
    }

    pub fn formatted_value(&self) -> String {
        time_picker_label_step(self.selected, self.step_minutes)
    }

    pub fn formatted_value_24h(&self) -> String {
        time_picker_label_24h(self.selected, self.step_minutes)
    }
}

/// Format a half-hour time slot (`0..48`) as `H:MM AM/PM`.
pub fn time_picker_label(index: usize) -> String {
    time_picker_label_step(index, 30)
}

/// Format a time slot with a configurable minute interval as `H:MM AM/PM`.
pub fn time_picker_label_step(index: usize, step_minutes: u16) -> String {
    let step = step_minutes.clamp(1, 720) as usize;
    let total_minutes = (index * step) % (24 * 60);
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    let h12 = if hours.is_multiple_of(12) { 12 } else { hours % 12 };
    let period = if hours < 12 { "AM" } else { "PM" };
    format!("{h12}:{minutes:02} {period}")
}

/// Format a time slot with a configurable minute interval in 24-hour `HH:MM` format.
pub fn time_picker_label_24h(index: usize, step_minutes: u16) -> String {
    let step = step_minutes.clamp(1, 720) as usize;
    let total_minutes = (index * step) % (24 * 60);
    format!("{:02}:{:02}", total_minutes / 60, total_minutes % 60)
}

impl Frame<'_> {
    /// Time picker trigger field + scrollable popover slot list (12-hour dropdown default).
    /// Returns `true` when the selected time slot changed.
    pub fn time_picker(&mut self, id: u64, label: &str, state: &mut TimePickerState, filled: bool) -> bool {
        self.time_picker_with_options(id, label, state, filled, TimePickerOptions::default())
    }

    /// Time picker supporting `Dropdown` slot list or interactive Material `Dial` face in 12h or 24h format.
    pub fn time_picker_with_options(
        &mut self,
        id: u64,
        label: &str,
        state: &mut TimePickerState,
        filled: bool,
        options: TimePickerOptions,
    ) -> bool {
        let count = state.slot_count().max(1);
        state.selected = state.selected.min(count - 1);
        let value = if options.use_24h {
            state.formatted_value_24h()
        } else {
            state.formatted_value()
        };
        let trigger = self.popup_field(id, label, &value, filled, FieldAffordance::Clock);
        if trigger.clicked {
            state.anchor = trigger.bounds;
            state.overlay.open = !state.overlay.open;
            state.list.selected = Some(state.selected);
        }
        let mut changed = false;
        state.overlay.kind = OverlayKind::Menu;
        state.overlay.style = OverlayStyle::AngularTimepicker;
        state.overlay.dismiss_policy = OverlayDismissPolicy::EscapeOrOutside;
        let step = state.step_minutes.clamp(1, 720);
        let (popup_w, popup_h) = match options.mode {
            TimePickerMode::Dropdown => (state.anchor.width.max(180.0), (count as f32 * 48.0 + 16.0).min(256.0)),
            TimePickerMode::Dial => (280.0_f32.min(self.viewport().width.max(220.0)), 320.0),
        };
        let popup = Rect {
            x: state.anchor.x,
            y: state.anchor.y + state.anchor.height + 4.0,
            width: popup_w,
            height: popup_h,
        };
        let mut overlay = std::mem::take(&mut state.overlay);
        self.overlay(id.wrapping_add(50), &mut overlay, popup, false, |f, open| {
            let area = f.available();
            match options.mode {
                TimePickerMode::Dropdown => {
                    let list_area = Rect { width: (area.width - 16.0).max(0.0), ..area };
                    f.within(list_area, |f| {
                        let mut picked = None;
                        let _ = f.virtual_list_rows(
                            id.wrapping_add(100),
                            list_area.height,
                            count,
                            &mut state.list,
                            48.0,
                            |f, index, selected| {
                                let text = if options.use_24h {
                                    time_picker_label_24h(index, step)
                                } else {
                                    time_picker_label_step(index, step)
                                };
                                let r = f.menu_item_with_selection(
                                    id.wrapping_add(200 + index as u64),
                                    &text,
                                    true,
                                    selected,
                                    false,
                                );
                                if r.clicked {
                                    picked = Some(index);
                                }
                                r
                            },
                        );
                        if let Some(idx) = picked {
                            state.selected = idx;
                            state.list.selected = Some(idx);
                            *open = false;
                            changed = true;
                        }
                    });
                    let track = Rect {
                        x: area.x + area.width - 15.0,
                        y: area.y,
                        width: 15.0,
                        height: area.height,
                    };
                    let content_h = count as f32 * 48.0 + 16.0;
                    f.vertical_scrollbar_painted(
                        id.wrapping_add(150),
                        track,
                        &mut state.list.offset,
                        area.height,
                        content_h,
                        &mut state.scrollbar,
                    );
                }
                TimePickerMode::Dial => {
                    let t = f.theme();
                    let total_min = (state.selected * step as usize) % (24 * 60);
                    let cur_hour = total_min / 60;
                    let cur_min = total_min % 60;
                    let header_h = 44.0;
                    let header = Rect { x: area.x + 12.0, y: area.y + 8.0, width: area.width - 24.0, height: header_h };
                    let display = if options.use_24h {
                        format!("{cur_hour:02}:{cur_min:02}")
                    } else {
                        let h12 = if cur_hour.is_multiple_of(12) { 12 } else { cur_hour % 12 };
                        format!("{h12}:{cur_min:02} {}", if cur_hour < 12 { "AM" } else { "PM" })
                    };
                    f.styled_text(
                        header,
                        &display,
                        t.text,
                        TextStyle { size: 22.0, weight: 600, align: 0, ..TextStyle::default() },
                    );
                    if !options.use_24h {
                        let toggle_rect = Rect {
                            x: header.x + header.width - 76.0,
                            y: header.y + 4.0,
                            width: 76.0,
                            height: 36.0,
                        };
                        let tr = f.action_at(
                            id.wrapping_add(160),
                            toggle_rect,
                            if cur_hour < 12 { "AM" } else { "PM" },
                            Options { variant: ButtonVariant::Tonal, ..Options::default() },
                        );
                        if tr.clicked {
                            let new_hour = (cur_hour + 12) % 24;
                            let new_min = new_hour * 60 + cur_min;
                            state.selected = (new_min / step as usize).min(count - 1);
                            changed = true;
                        }
                    }
                    let dial_sz = (area.width - 32.0).min(area.height - header_h - 32.0).max(120.0);
                    let dial = Rect {
                        x: area.x + (area.width - dial_sz) * 0.5,
                        y: area.y + header_h + 16.0,
                        width: dial_sz,
                        height: dial_sz,
                    };
                    let cx = dial.x + dial_sz * 0.5;
                    let cy = dial.y + dial_sz * 0.5;
                    let r_outer = dial_sz * 0.5;
                    f.paint_rect(dial, r_outer, t.surface_high);
                    let ring_r = r_outer - 22.0;
                    let sel_idx = cur_hour % 12;
                    let sel_angle = (sel_idx as f32 / 12.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                    let is_inner_24 = options.use_24h && cur_hour >= 12;
                    let hand_r = if is_inner_24 { ring_r * 0.62 } else { ring_r };
                    let tip = [cx + hand_r * sel_angle.cos(), cy + hand_r * sel_angle.sin()];
                    f.paint_line([cx, cy], tip, 2.0, t.primary);
                    f.paint_rect(Rect { x: cx - 4.0, y: cy - 4.0, width: 8.0, height: 8.0 }, 4.0, t.primary);

                    let total_hours = if options.use_24h { 24 } else { 12 };
                    for h in 0..total_hours {
                        let inner = options.use_24h && h >= 12;
                        let pos_idx = h % 12;
                        let angle = (pos_idx as f32 / 12.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                        let rad = if inner { ring_r * 0.62 } else { ring_r };
                        let hx = cx + rad * angle.cos();
                        let hy = cy + rad * angle.sin();
                        let hit_sz = if inner { 28.0 } else { 32.0 };
                        let cell = Rect { x: hx - hit_sz * 0.5, y: hy - hit_sz * 0.5, width: hit_sz, height: hit_sz };
                        let is_sel = if options.use_24h { cur_hour == h } else { cur_hour % 12 == h };
                        let lbl = if options.use_24h {
                            format!("{h:02}")
                        } else if h == 0 {
                            "12".to_owned()
                        } else {
                            h.to_string()
                        };
                        let r = f.target(id.wrapping_add(300 + h as u64), cell, &lbl, true, Role::Button);
                        if is_sel {
                            f.paint_rect(cell, hit_sz * 0.5, t.primary);
                        }
                        f.state_layer(r, hit_sz * 0.5, if is_sel { t.on_primary } else { t.text });
                        f.styled_text(
                            cell,
                            &lbl,
                            if is_sel { t.on_primary } else { t.text },
                            TextStyle { size: if inner { 11.0 } else { 13.0 }, weight: 500, align: 1, ..TextStyle::default() },
                        );
                        if r.clicked {
                            let new_hour = if options.use_24h {
                                h
                            } else if cur_hour >= 12 {
                                h + 12
                            } else {
                                h
                            };
                            let new_min = new_hour * 60 + cur_min;
                            state.selected = (new_min / step as usize).min(count - 1);
                            state.list.selected = Some(state.selected);
                            changed = true;
                        }
                    }
                }
            }
        });
        state.overlay = overlay;
        changed
    }
}

#[cfg(test)]
#[path = "time_picker.spec.rs"]
mod tests;
