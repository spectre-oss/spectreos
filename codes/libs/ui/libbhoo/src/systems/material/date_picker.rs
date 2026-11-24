//! Material Design 3 & Angular-compatible calendar, date picker, and date-range picker.
use crate::*;

/// Active view inside a calendar / date picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerView {
    #[default]
    Days,
    Years,
    Months,
}

/// Optional constraints & mode configuration for `Frame::calendar_grid_with_options` and
/// `Frame::date_picker_with_options`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DatePickerOptions {
    /// When `true`, renders direct text-entry mode instead of the visual calendar grid.
    pub text_input_mode: bool,
    /// Optional inclusive minimum selectable date `[year, month, day]`.
    pub min_date: Option<[i32; 3]>,
    /// Optional inclusive maximum selectable date `[year, month, day]`.
    pub max_date: Option<[i32; 3]>,
}

impl DatePickerOptions {
    /// Returns `true` if `date` is disabled by `min_date`/`max_date`.
    pub fn is_disabled(&self, date: [i32; 3]) -> bool {
        self.min_date.is_some_and(|min| date < min) || self.max_date.is_some_and(|max| date > max)
    }
}

/// Caller-owned state for `Frame::calendar_grid`, `Frame::date_picker`, and `Frame::date_range_picker`.
#[derive(Clone)]
pub struct DatePickerState {
    pub year: i32,
    pub month: i32,
    pub selected: Option<[i32; 3]>,
    pub range_end: Option<[i32; 3]>,
    pub today: [i32; 3],
    pub focused_day: usize,
    pub view: DatePickerView,
    pub picking_end: bool,
    pub overlay: OverlayState,
    pub anchor: Rect,
}

impl Default for DatePickerState {
    fn default() -> Self {
        Self {
            year: 2026,
            month: 11,
            selected: Some([2026, 11, 9]),
            range_end: None,
            today: [2026, 11, 9],
            focused_day: 9,
            view: DatePickerView::Days,
            picking_end: false,
            overlay: OverlayState::default(),
            anchor: Rect::default(),
        }
    }
}

impl DatePickerState {
    /// Formatted label for the current single date (`M/D/YYYY`).
    pub fn formatted_date(&self) -> String {
        match self.selected {
            Some([y, m, d]) => format!("{m}/{d}/{y}"),
            None => String::new(),
        }
    }

    /// Formatted label for the current date range (`M/D/YYYY – M/D/YYYY`).
    pub fn formatted_range(&self) -> String {
        match (self.selected, self.range_end, self.picking_end) {
            (Some([y1, m1, d1]), Some([y2, m2, d2]), false) => {
                format!("{m1}/{d1}/{y1} – {m2}/{d2}/{y2}")
            }
            (Some([y1, m1, d1]), _, _) => format!("{m1}/{d1}/{y1} –"),
            _ => String::new(),
        }
    }
}

/// Number of days in a Gregorian `(year, month)` (`month` in `1..=12`).
pub fn month_days(year: i32, month: i32) -> usize {
    match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Day of week for a Gregorian date (`0 = Monday .. 6 = Sunday`).
pub fn weekday(year: i32, month: i32, day: i32) -> usize {
    let y = year - i32::from(month < 3);
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let m = month.clamp(1, 12) as usize;
    ((y + y / 4 - y / 100 + y / 400 + t[m - 1] + day + 6).rem_euclid(7)) as usize
}

fn paint_range_preview(f: &mut Frame<'_>, cell: Rect, inner: Rect, start: bool, end: bool, color: Color) {
    let x0 = if start { inner.x + inner.width * 0.5 } else { cell.x };
    let x1 = if end { inner.x + inner.width * 0.5 } else { cell.x + cell.width };
    let top = inner.y + 1.0;
    let bottom = inner.y + inner.height;
    let width = x1 - x0;
    let count = ((width + 2.0) / 5.0).floor().max(1.0) as usize;
    let spacing = if count > 1 { (width - 3.0) / (count - 1) as f32 } else { 0.0 };
    for i in 0..count {
        for y in [top, bottom] {
            let x = if end && !start {
                if y == top { x0 + 5.0 * i as f32 } else { x1 - 3.0 - 5.0 * i as f32 }
            } else {
                x0 + spacing * i as f32
            };
            let left = x.max(x0);
            let right = (x + 3.0).min(x1);
            if right > left {
                f.paint_rect(Rect { x: left, y, width: right - left, height: 1.0 }, 0.0, color);
            }
        }
    }
    if end && !start {
        f.paint_rect(Rect { x: x0, y: bottom, width: 2.0_f32.min(width), height: 1.0 }, 0.0, color);
    }
    let radius = (inner.height - 1.0) * 0.5;
    let cy = inner.y + inner.height * 0.5 + 1.0;
    for (enabled, cx, base) in [
        (start, x0, std::f32::consts::FRAC_PI_2),
        (end, x1, -std::f32::consts::FRAC_PI_2),
    ] {
        if enabled && radius > 0.5 {
            let length = std::f32::consts::PI * radius;
            let count = (length / 5.0).ceil().max(1.0) as usize;
            for i in 0..count {
                let angle = base + 5.0 * i as f32 / radius;
                f.paint_arc(
                    Rect {
                        x: cx - inner.width / 2.0,
                        y: cy - inner.height / 2.0,
                        width: inner.width,
                        height: inner.height,
                    },
                    angle,
                    (3.0 / radius).min(base + std::f32::consts::PI - angle),
                    1.0,
                    color,
                );
            }
        }
    }
}

impl Frame<'_> {
    /// Inline Material calendar grid supporting single-date or date-range selection,
    /// month/year/24-year drilldown, range hover preview, and roving arrow-key navigation.
    /// Returns `true` when the selected date/range changed.
    pub fn calendar_grid(&mut self, id: u64, state: &mut DatePickerState, range: bool) -> bool {
        self.calendar_grid_with_options(id, state, range, DatePickerOptions::default(), |_| false)
    }

    /// Inline Material calendar grid with optional text-input mode, `min_date`/`max_date` bounds,
    /// and a caller-supplied `is_disabled` predicate for blacked-out dates.
    pub fn calendar_grid_with_options(
        &mut self,
        id: u64,
        state: &mut DatePickerState,
        range: bool,
        options: DatePickerOptions,
        is_disabled: impl Fn([i32; 3]) -> bool,
    ) -> bool {
        if options.text_input_mode {
            let value = if range { state.formatted_range() } else { state.formatted_date() };
            let r = self.text_field(
                id.wrapping_add(400),
                if range { "Start – End (M/D/YYYY)" } else { "Date (M/D/YYYY)" },
                56.0,
                !value.is_empty(),
                false,
                false,
            );
            if !value.is_empty() {
                self.styled_text(
                    Rect { x: r.bounds.x + 16.0, width: (r.bounds.width - 32.0).max(0.0), ..r.bounds },
                    &value,
                    self.theme().text,
                    TextStyle { size: 16.0, ..TextStyle::default() },
                );
            }
            return false;
        }
        const MONTHS: [&str; 12] = [
            "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
        ];
        const DAYS: [&str; 31] = [
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
            "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30",
            "31",
        ];
        state.month = state.month.clamp(1, 12);
        let mut changed = false;
        let root = self.available();
        let t = self.theme();
        let width = (root.width - 16.0).max(0.0);
        let pitch = (width / 7.0).max(1.0);
        let margin = width * 0.05;
        let control_y = root.y + 8.0 + margin;
        let control_x = root.x + 8.0 + width * (4.714_286 / 100.0) - 16.0;
        let right = root.x + root.width - (control_x - root.x);
        let period = format!("{} {}", MONTHS[(state.month - 1) as usize], state.year);
        let period_width = (self.text_width(&period, TextStyle::LABEL) + 40.0).max(64.0);
        let period_box = Rect {
            x: control_x + 8.0,
            y: control_y,
            width: period_width,
            height: 40.0,
        };
        let r = self.target(id, period_box, "Choose year and month", true, Role::Button);
        self.state_layer(r, 20.0, t.on_surface_variant);
        if r.clicked {
            state.view = match state.view {
                DatePickerView::Days => DatePickerView::Years,
                _ => DatePickerView::Days,
            };
        }
        self.styled_text(
            period_box,
            &period,
            t.on_surface_variant,
            TextStyle { align: 0, padding: 12.0, ..TextStyle::LABEL },
        );
        let x = period_box.x + period_box.width - 16.0;
        let y = period_box.y + 20.0;
        for (i, half) in [4.0, 3.0, 2.0, 1.0].into_iter().enumerate() {
            self.paint_line([x - half, y - 1.5 + i as f32], [x + half, y - 1.5 + i as f32], 1.0, t.on_surface_variant);
        }
        for i in 0..2u64 {
            let b = Rect { x: right - 80.0 + i as f32 * 40.0, y: control_y, width: 40.0, height: 40.0 };
            let r = self.target(id.wrapping_add(1 + i), b, if i == 0 { "Previous month" } else { "Next month" }, true, Role::Button);
            self.state_layer(r, 20.0, t.on_surface_variant);
            let dir = if i == 0 { -1.0 } else { 1.0 };
            let c = [b.x + 20.0, b.y + 20.0];
            self.paint_line([c[0] - dir * 2.5, c[1] - 4.0], [c[0] + dir * 1.5, c[1]], 2.0, t.on_surface_variant);
            self.paint_line([c[0] + dir * 1.5, c[1]], [c[0] - dir * 2.5, c[1] + 4.0], 2.0, t.on_surface_variant);
            if r.clicked {
                match state.view {
                    DatePickerView::Years => state.year += if i == 0 { -24 } else { 24 },
                    DatePickerView::Months => state.year += if i == 0 { -1 } else { 1 },
                    DatePickerView::Days => {
                        state.month += if i == 0 { -1 } else { 1 };
                        if state.month == 0 {
                            state.month = 12;
                            state.year -= 1;
                        }
                        if state.month == 13 {
                            state.month = 1;
                            state.year += 1;
                        }
                        state.focused_day = state.focused_day.clamp(1, month_days(state.year, state.month));
                    }
                }
            }
        }
        let body_y = root.y + 8.0 + 2.0 * margin + 40.0;
        if state.view != DatePickerView::Days {
            let count = if state.view == DatePickerView::Years { 24 } else { 12 };
            let cols = 4;
            let row_height = 52.0;
            let start = state.year.div_euclid(24) * 24;
            for (i, month) in MONTHS.iter().cycle().take(count).enumerate() {
                let b = Rect {
                    x: root.x + 8.0 + (i % cols) as f32 * width / cols as f32,
                    y: body_y + (i / cols) as f32 * row_height,
                    width: width / cols as f32,
                    height: row_height,
                };
                let label = if state.view == DatePickerView::Years {
                    (start + i as i32).to_string()
                } else {
                    (*month).to_owned()
                };
                let r = self.target(id.wrapping_add(100 + i as u64), b, &label, true, Role::Button);
                self.state_layer(r, 26.0, t.on_surface_variant);
                self.styled_text(b, &label, t.text, TextStyle { size: 14.0, align: 1, ..TextStyle::default() });
                if r.clicked {
                    if state.view == DatePickerView::Years {
                        state.year = start + i as i32;
                        state.view = DatePickerView::Months;
                    } else {
                        state.month = i as i32 + 1;
                        state.view = DatePickerView::Days;
                    }
                }
            }
            self.reserve((body_y - root.y) + count.div_ceil(cols) as f32 * row_height);
            return false;
        }
        for (i, label) in ["S", "M", "T", "W", "T", "F", "S"].iter().enumerate() {
            self.styled_text(
                Rect { x: root.x + 8.0 + i as f32 * pitch, y: body_y, width: pitch, height: 18.0 },
                label,
                t.on_surface_variant,
                TextStyle { size: 14.0, weight: 500, align: 1, ..TextStyle::default() },
            );
        }
        let days = month_days(state.year, state.month);
        let offset = (weekday(state.year, state.month, 1) + 1) % 7;
        let extra = usize::from(offset == 0);
        let grid_y = body_y + 32.0;
        let month_label = Rect {
            x: root.x + 8.0 + 18.0,
            y: grid_y,
            width: if offset == 0 { width - 18.0 } else { pitch * offset as f32 - 18.0 },
            height: pitch,
        };
        self.styled_text(
            month_label,
            MONTHS[(state.month - 1) as usize],
            t.text,
            TextStyle { size: 14.0, weight: 500, ..TextStyle::default() },
        );
        let preview_end = if range && state.picking_end && let Some(start_d) = state.selected {
            (1..=days).find_map(|day| {
                let index = offset + day - 1;
                let cell = Rect {
                    x: root.x + 8.0 + (index % 7) as f32 * pitch,
                    y: grid_y + (extra + index / 7) as f32 * pitch,
                    width: pitch,
                    height: pitch,
                };
                let date = [state.year, state.month, day as i32];
                let allowed = !options.is_disabled(date) && !is_disabled(date);
                (allowed && date > start_d && self.pointer_in(cell)).then_some(date)
            })
        } else {
            None
        };
        state.focused_day = state.focused_day.clamp(1, days);
        for day in 1..=days {
            let index = offset + day - 1;
            let cell = Rect {
                x: root.x + 8.0 + (index % 7) as f32 * pitch,
                y: grid_y + (extra + index / 7) as f32 * pitch,
                width: pitch,
                height: pitch,
            };
            let b = cell.inset(pitch * 0.05);
            let date = [state.year, state.month, day as i32];
            let day_enabled = !options.is_disabled(date) && !is_disabled(date);
            let selected = if range {
                state.selected == Some(date) || (!state.picking_end && state.range_end == Some(date))
            } else {
                state.selected == Some(date)
            };
            let r = self.target_roving(id.wrapping_add(200 + day as u64), b, DAYS[day - 1], day_enabled, day == state.focused_day);
            if let (Some(start_d), Some(end_d)) = (state.selected, preview_end)
                && date >= start_d
                && date <= end_d
            {
                paint_range_preview(self, cell, b, date == start_d, date == end_d, t.primary);
            }
            if range
                && !state.picking_end
                && let (Some(s), Some(e)) = (state.selected, state.range_end)
                && date > s
                && date < e
            {
                self.paint_rect(Rect { y: b.y, height: b.height, ..cell }, 0.0, t.primary_container);
            }
            if selected
                && range
                && !state.picking_end
                && let (Some(s), Some(e)) = (state.selected, state.range_end)
                && s != e
            {
                let left = if date == s { b.x + b.width / 2.0 } else { cell.x };
                let right = if date == e { b.x + b.width / 2.0 } else { cell.x + cell.width };
                self.paint_rect(
                    Rect { x: left, y: b.y, width: (right - left).max(0.0), height: b.height },
                    0.0,
                    t.primary_container,
                );
            }
            if selected {
                self.paint_rect(b, b.width / 2.0, if r.enabled { t.primary } else { t.text.opacity(0.12) });
            } else if date == state.today {
                self.paint_rect(b, b.width / 2.0, t.text);
                self.paint_rect(b.inset(1.0), (b.width / 2.0 - 1.0).max(0.0), t.background);
            }
            self.state_layer(r, b.width / 2.0, if selected { t.on_primary } else { t.on_surface_variant });
            self.styled_text(
                b,
                DAYS[day - 1],
                if !r.enabled {
                    t.text.opacity(0.38)
                } else if selected {
                    t.on_primary
                } else {
                    t.text
                },
                TextStyle { size: 14.0, align: 1, ..TextStyle::default() },
            );
            if r.clicked {
                state.focused_day = day;
                changed = true;
                if range {
                    if state.picking_end && state.selected.is_some_and(|s| date >= s) {
                        state.range_end = Some(date);
                        state.picking_end = false;
                    } else {
                        state.selected = Some(date);
                        state.range_end = Some(date);
                        state.picking_end = true;
                    }
                } else {
                    state.selected = Some(date);
                }
            }
            if r.focused {
                let mut next = day as i32;
                if self.pressed_key(Key::Left) { next -= 1; }
                if self.pressed_key(Key::Right) { next += 1; }
                if self.pressed_key(Key::Up) { next -= 7; }
                if self.pressed_key(Key::Down) { next += 7; }
                if self.pressed_key(Key::Home) { next = 1; }
                if self.pressed_key(Key::End) { next = days as i32; }
                next = next.clamp(1, days as i32);
                if next != day as i32 {
                    state.focused_day = next as usize;
                    self.request_focus(id.wrapping_add(200 + next as u64));
                }
            }
        }
        self.reserve((grid_y - root.y) + (extra + (offset + days).div_ceil(7)) as f32 * pitch);
        changed
    }

    /// Dual-month side-by-side calendar range picker displaying `(state.year, state.month)`
    /// alongside the consecutive following month for cross-month range selection.
    pub fn date_range_picker_dual(
        &mut self,
        id: u64,
        state: &mut DatePickerState,
        options: DatePickerOptions,
        is_disabled: impl Fn([i32; 3]) -> bool + Copy,
    ) -> bool {
        let mut changed = false;
        let (next_year, next_month) = if state.month == 12 {
            (state.year + 1, 1)
        } else {
            (state.year, state.month + 1)
        };
        let mut second = DatePickerState {
            year: next_year,
            month: next_month,
            selected: state.selected,
            range_end: state.range_end,
            today: state.today,
            focused_day: 1,
            view: DatePickerView::Days,
            picking_end: state.picking_end,
            overlay: OverlayState::default(),
            anchor: Rect::default(),
        };
        let area = self.available();
        if area.width >= 520.0 {
            let half = (area.width - 16.0) * 0.5;
            let row = self.reserve(340.0);
            let left_rect = Rect { width: half, ..row };
            let right_rect = Rect { x: row.x + half + 16.0, width: half, ..row };
            self.within(left_rect, |f| {
                if f.calendar_grid_with_options(id, state, true, options, is_disabled) {
                    changed = true;
                }
            });
            second.selected = state.selected;
            second.range_end = state.range_end;
            second.picking_end = state.picking_end;
            self.within(right_rect, |f| {
                if f.calendar_grid_with_options(id.wrapping_add(500), &mut second, true, options, is_disabled) {
                    state.selected = second.selected;
                    state.range_end = second.range_end;
                    state.picking_end = second.picking_end;
                    changed = true;
                }
            });
        } else {
            changed |= self.calendar_grid_with_options(id, state, true, options, is_disabled);
            second.selected = state.selected;
            second.range_end = state.range_end;
            second.picking_end = state.picking_end;
            if self.calendar_grid_with_options(id.wrapping_add(500), &mut second, true, options, is_disabled) {
                state.selected = second.selected;
                state.range_end = second.range_end;
                state.picking_end = second.picking_end;
                changed = true;
            }
        }
        changed
    }

    /// Date picker field paired with an anchored calendar popup overlay.
    /// Returns `true` when the selected date changed.
    pub fn date_picker(&mut self, id: u64, label: &str, state: &mut DatePickerState, filled: bool) -> bool {
        let display = state.formatted_date();
        let trigger = self.popup_field(id, label, &display, filled, FieldAffordance::Calendar);
        if trigger.clicked {
            state.anchor = trigger.bounds;
            state.overlay.open = !state.overlay.open;
            state.view = DatePickerView::Days;
        }
        let mut changed = false;
        state.overlay.kind = OverlayKind::Menu;
        state.overlay.style = OverlayStyle::AngularDatepicker;
        state.overlay.dismiss_policy = OverlayDismissPolicy::EscapeOrOutside;
        let popup = Rect {
            x: state.anchor.x,
            y: state.anchor.y + state.anchor.height + 4.0,
            width: 296.0_f32.min(self.viewport().width.max(200.0)),
            height: 354.0,
        };
        let mut overlay = std::mem::take(&mut state.overlay);
        self.overlay(id.wrapping_add(50), &mut overlay, popup, false, |f, open| {
            if f.calendar_grid(id.wrapping_add(100), state, false) {
                changed = true;
                *open = false;
            }
        });
        state.overlay = overlay;
        changed
    }

    /// Date-range picker field paired with an anchored calendar popup overlay.
    /// Returns `true` when the selected range changed.
    pub fn date_range_picker(&mut self, id: u64, label: &str, state: &mut DatePickerState, filled: bool) -> bool {
        let display = state.formatted_range();
        let (field_r, icon_r) = self.date_range_field(id, label, &display, filled);
        if field_r.clicked || icon_r.clicked {
            state.anchor = field_r.bounds;
            state.overlay.open = !state.overlay.open;
            state.view = DatePickerView::Days;
        }
        let mut changed = false;
        state.overlay.kind = OverlayKind::Menu;
        state.overlay.style = OverlayStyle::AngularDatepicker;
        state.overlay.dismiss_policy = OverlayDismissPolicy::EscapeOrOutside;
        let popup = Rect {
            x: state.anchor.x,
            y: state.anchor.y + state.anchor.height + 4.0,
            width: 296.0_f32.min(self.viewport().width.max(200.0)),
            height: 354.0,
        };
        let mut overlay = std::mem::take(&mut state.overlay);
        self.overlay(id.wrapping_add(50), &mut overlay, popup, false, |f, open| {
            if f.calendar_grid(id.wrapping_add(100), state, true) {
                changed = true;
                if !state.picking_end {
                    *open = false;
                }
            }
        });
        state.overlay = overlay;
        changed
    }
}

#[cfg(test)]
#[path = "date_picker.spec.rs"]
mod tests;
