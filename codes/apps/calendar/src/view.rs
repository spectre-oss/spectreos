use crate::*;
use bhoo::android_calendar::{CalendarDate, CalendarState};
use bhoo::android_xml::{AndroidTheme, WallClock};
use bhoo::{Color, Frame, Key, MaterialIcon, Rect, Role, TextStyle};
use chrono::{Datelike, Timelike};

const HOUR: f32 = 62.0;
const TOOLBAR: f32 = 56.0;
const TIMELINE_GUTTER: f32 = 46.0;
const TIMELINE_END_INSET: f32 = 8.0;
const EVENT_SELECTION_STEP: i64 = 15;

impl TimelineSelection {
    fn minutes(&self) -> (i64, i64) {
        let start = self.anchor_minute.min(self.end_minute);
        let end = self.anchor_minute.max(self.end_minute);
        (start, end.max(start + EVENT_SELECTION_STEP))
    }
}

fn timeline_minute(y: f32, top: f32) -> i64 {
    let minute = ((y - top) * 60.0 / HOUR).floor() as i64;
    minute.clamp(0, 24 * 60 - 1) / EVENT_SELECTION_STEP * EVENT_SELECTION_STEP
}
const RULES: [(Option<&str>, &str); 6] = [
    (None, "Does not repeat"),
    (Some("FREQ=DAILY"), "Daily"),
    (Some("FREQ=WEEKLY"), "Weekly"),
    (Some("FREQ=MONTHLY"), "Monthly"),
    (Some("FREQ=YEARLY"), "Yearly"),
    (Some("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"), "Weekdays"),
];
const REMINDERS: [(Option<u32>, &str); 8] = [
    (None, "Off"),
    (Some(0), "At time of event"),
    (Some(5), "5 minutes before"),
    (Some(10), "10 minutes before"),
    (Some(15), "15 minutes before"),
    (Some(30), "30 minutes before"),
    (Some(60), "1 hour before"),
    (Some(1440), "1 day before"),
];

#[derive(Clone, Copy)]
pub struct Appearance {
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub muted: Color,
    pub primary: Color,
    pub on_primary: Color,
    pub accent_surface: Color,
    pub on_accent_surface: Color,
    pub divider: Color,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            background: Color(250, 250, 247, 255),
            surface: Color(239, 241, 237, 255),
            text: Color(28, 38, 33, 255),
            muted: Color(84, 98, 92, 255),
            primary: Color(36, 95, 77, 255),
            on_primary: Color(255, 255, 255, 255),
            accent_surface: Color(207, 234, 222, 255),
            on_accent_surface: Color(20, 68, 54, 255),
            divider: Color(218, 225, 220, 255),
        }
    }
}

pub(crate) fn clock_scroll(time: NaiveDateTime) -> f32 {
    (time.hour() as f32 - 3.0).clamp(0.0, 15.0) * HOUR
}

fn rect(b: Rect, x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x: b.x + x,
        y: b.y + y,
        width,
        height,
    }
}

fn style(size: f32, weight: u16, align: u8) -> TextStyle {
    TextStyle {
        size,
        weight,
        align,
        padding: 0.0,
        ..TextStyle::default()
    }
}

fn text(f: &mut Frame<'_>, b: Rect, value: &str, ink: Color, size: f32, weight: u16, align: u8) {
    f.styled_text(b, value, ink, style(size, weight, align));
}

fn icon(
    f: &mut Frame<'_>,
    id: u64,
    b: Rect,
    label: &str,
    glyph: MaterialIcon,
    foreground: Color,
) -> bool {
    let response = f.target(id, b, label, true, Role::Button);
    f.state_layer(response, 23.0, foreground);
    f.paint_material_icon(
        b.inset((b.height.min(b.width) - 21.0) * 0.5),
        glyph,
        foreground,
    );
    response.clicked
}

fn button(f: &mut Frame<'_>, id: u64, b: Rect, caption: &str, p: Appearance, filled: bool) -> bool {
    let response = f.target(id, b, caption, true, Role::Button);
    let (surface, ink) = if filled {
        (p.primary, p.on_primary)
    } else {
        (p.accent_surface, p.on_accent_surface)
    };
    f.paint_rect(b, b.height * 0.5, surface);
    f.state_layer(response, b.height * 0.5, ink);
    text(f, b.inset(4.0), caption, ink, 13.0, 600, 1);
    response.clicked
}

fn row(f: &mut Frame<'_>, id: u64, b: Rect, name: &str, sub: &str, p: Appearance) -> bool {
    let result = f.target(id, b, name, true, Role::Button);
    f.state_layer(result, 12.0, p.primary);
    text(
        f,
        rect(
            b,
            13.0,
            if sub.is_empty() { 0.0 } else { 5.0 },
            b.width - 22.0,
            if sub.is_empty() { b.height } else { 26.0 },
        ),
        name,
        p.text,
        15.0,
        500,
        0,
    );
    if !sub.is_empty() {
        text(
            f,
            rect(b, 13.0, 28.0, b.width - 22.0, 22.0),
            sub,
            p.muted,
            12.0,
            400,
            0,
        );
    }
    result.clicked
}

fn calendar_color(data: &model::Data, id: &str) -> Color {
    swatch(
        data.calendars
            .iter()
            .find(|calendar| calendar.id == id)
            .map_or(0, |item| item.color),
    )
}

fn swatch(index: usize) -> Color {
    let (r, g, b) = model::COLORS
        .get(index)
        .copied()
        .unwrap_or(model::COLORS[0]);
    Color(r, g, b, 255)
}

fn week_start(day: NaiveDate) -> NaiveDate {
    day - Days::new(day.weekday().num_days_from_sunday().into())
}

fn short_time(item: &Occurrence) -> String {
    if item.all_day {
        "All day".into()
    } else {
        format!(
            "{}–{}",
            item.display_start().format("%H:%M"),
            item.display_end().format("%H:%M")
        )
    }
}

fn layout_field(
    f: &mut Frame<'_>,
    b: Rect,
    id: u64,
    label: &str,
    value: &str,
    p: Appearance,
) -> bool {
    let result = f.target(id, b, label, true, Role::TextField);
    f.paint_rect(b, 12.0, p.surface);
    text(
        f,
        rect(b, 11.0, 3.0, b.width - 22.0, 19.0),
        label,
        p.muted,
        11.0,
        550,
        0,
    );
    text(
        f,
        rect(b, 11.0, 20.0, b.width - 22.0, b.height - 23.0),
        value,
        p.text,
        15.0,
        500,
        0,
    );
    if result.focused {
        f.paint_rect(
            rect(b, 8.0, b.height - 2.0, b.width - 16.0, 2.0),
            1.0,
            p.primary,
        );
    }
    result.clicked
}

impl CalendarApp {
    pub fn show(&mut self, f: &mut Frame<'_>, b: Rect, base: u64, p: Appearance) -> Action {
        self.refresh();
        let mut action = Action::None;
        f.within(b, |f| {
            f.paint_rect(b, 0.0, p.background);
            if self.file_task.is_some() {
                let theme = spectre_files::Appearance {
                    background: p.background,
                    surface: p.background,
                    raised: p.surface,
                    text: p.text,
                    muted: p.muted,
                    primary: p.primary,
                    selected: p.accent_surface,
                    on_primary: p.on_primary,
                    outline: p.divider,
                };
                action = match self.picker.show(f, b, theme) {
                    spectre_files::Action::Selected(mut files) => {
                        if let Some(path) = files.pop() {
                            self.file_selected(path);
                        }
                        Action::Blur
                    }
                    spectre_files::Action::Cancelled => {
                        self.file_task = None;
                        Action::Blur
                    }
                    spectre_files::Action::Focus(id) => Action::Focus(id),
                    spectre_files::Action::Blur => Action::Blur,
                    _ => Action::None,
                };
                return;
            }
            let dialog = !matches!(self.dialog, Dialog::None);
            if dialog
                || self.view_drawer.open
                || self.options_sheet.open
                || self.draft.is_some()
                || self.detail.is_some()
                || self.search
                || !matches!(self.mode, Mode::Day | Mode::Week)
            {
                self.timeline_selection = None;
            }
            f.without_input(
                dialog || self.view_drawer.open || self.options_sheet.open,
                |f| {
                    if self.draft.is_some() {
                        action = self.paint_entering_event_form(f, b, base, p);
                    } else if self.detail.is_some() {
                        self.paint_event_detail(f, b, base, p);
                    } else {
                        action = self.paint_calendar(f, b, base, p);
                    }
                },
            );
            if dialog {
                action = self.paint_dialog(f, b, base, p);
            }
            if let Some(message) = self.message.as_ref().or(self.store.warning.as_ref()) {
                let bounds = rect(b, 15.0, b.height - 49.0, b.width - 96.0, 38.0);
                f.paint_rect(bounds, 13.0, p.accent_surface);
                text(
                    f,
                    bounds.inset(8.0),
                    message,
                    p.on_accent_surface,
                    12.0,
                    500,
                    0,
                );
            }
            self.paint_view_drawer(f, b, base);
            self.paint_options_sheet(f, b, base, p);
        });
        action
    }

    fn paint_calendar(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: Appearance) -> Action {
        let bar = rect(b, 0.0, 0.0, b.width, TOOLBAR);
        f.paint_rect(bar, 0.0, p.surface);
        let mut action = Action::None;
        let input = f.input();
        if input.pressed || input.scroll != [0.0; 2] {
            self.today_transition = None;
            self.week_fling = None;
        }
        if self.search {
            if icon(
                f,
                id + 1,
                rect(bar, 6.0, 4.0, 44.0, 44.0),
                "Close search",
                MaterialIcon::ArrowBack,
                p.text,
            ) {
                self.back();
                action = Action::Blur;
            }
            self.apply_key(f, FIELD_BASE + 7);
            if layout_field(
                f,
                rect(bar, 54.0, 3.0, b.width - 105.0, 49.0),
                FIELD_BASE + 7,
                "Search events",
                &self.query,
                p,
            ) {
                action = Action::Focus(FIELD_BASE + 7);
            }
            if icon(
                f,
                id + 4,
                rect(bar, b.width - 48.0, 4.0, 44.0, 44.0),
                "Clear search",
                MaterialIcon::Close,
                p.text,
            ) {
                self.query.clear();
            }
            let area = rect(b, 0.0, 60.0, b.width, b.height - 60.0);
            let today = crate::model::now_local_naive().date();
            if self.query.trim().is_empty() {
                text(
                    f,
                    area.inset(20.0),
                    "Search by title, location, or notes",
                    p.muted,
                    15.0,
                    400,
                    1,
                );
            } else {
                self.paint_agenda(
                    f,
                    area,
                    id,
                    today - Days::new(366 * 10),
                    today + Days::new(366 * 10),
                    true,
                    p,
                );
            }
            return action;
        }

        if icon(
            f,
            id + 1,
            rect(bar, 6.0, 5.0, 44.0, 44.0),
            "Calendar views",
            MaterialIcon::Menu,
            p.text,
        ) {
            self.view_drawer.open = true;
            self.message = None;
        }
        if icon(
            f,
            id + 2,
            rect(bar, b.width - 101.0, 5.0, 44.0, 44.0),
            "Today",
            MaterialIcon::Inbox,
            p.primary,
        ) {
            self.go_to_today(f, id);
        }
        self.update_today_transition(f, b);
        let title = self
            .selected_day
            .format(if self.mode == Mode::Day {
                "%a, %b %-d"
            } else {
                "%B %Y"
            })
            .to_string();
        text(
            f,
            rect(bar, 60.0, 1.0, b.width - 200.0, 53.0),
            &title,
            p.text,
            20.0,
            600,
            0,
        );
        if icon(
            f,
            id + 3,
            rect(bar, b.width - 50.0, 5.0, 44.0, 44.0),
            "More calendar options",
            MaterialIcon::MoreVert,
            p.text,
        ) {
            self.options_sheet.open = true;
            self.message = None;
        }
        let nav = rect(b, 0.0, TOOLBAR, b.width, 36.0);
        if icon(
            f,
            id + 5,
            rect(nav, 6.0, 0.0, 40.0, 36.0),
            "Previous period",
            MaterialIcon::ChevronLeft,
            p.muted,
        ) {
            self.navigate(false);
        }
        if icon(
            f,
            id + 6,
            rect(nav, b.width - 46.0, 0.0, 40.0, 36.0),
            "Next period",
            MaterialIcon::ChevronRight,
            p.muted,
        ) {
            self.navigate(true);
        }
        let content = rect(
            b,
            0.0,
            TOOLBAR + 36.0,
            b.width,
            (b.height - TOOLBAR - 36.0).max(0.0),
        );
        let add = rect(b, b.width - 71.0, b.height - 70.0, 53.0, 53.0);
        let add_target = f.target(id + 7, add, "New event", true, Role::Button);
        let selecting_event = self
            .timeline_selection
            .as_ref()
            .is_some_and(|selection| selection.active);
        let dragging_week =
            self.mode == Mode::Week && !selecting_event && self.scroll_week(f, content, id);
        if selecting_event {
            self.week_drag = None;
            self.press_origin = None;
        }
        let current = self.selected_day.format("%A, %B %-d").to_string();
        text(
            f,
            rect(nav, 53.0, 0.0, b.width - 106.0, 36.0),
            &current,
            p.muted,
            12.0,
            550,
            1,
        );

        let swipe = (self.mode != Mode::Week && !selecting_event)
            .then(|| self.horizontal_swipe(f, content))
            .flatten();
        match self.mode {
            Mode::Week => {
                f.cancel_pointer_input(dragging_week, |f| {
                    self.paint_timeline(f, content, add, id, p)
                });
            }
            Mode::Day => self.paint_timeline(f, content, add, id, p),
            Mode::Month => self.paint_month(f, content, id, p),
            Mode::Agenda => self.paint_agenda(
                f,
                content,
                id,
                self.selected_day,
                self.selected_day + Days::new(90),
                false,
                p,
            ),
        }
        if let Some(next) = swipe {
            self.navigate(next);
        }
        f.paint_rect(add, 19.0, p.primary);
        f.state_layer(add_target, 19.0, p.on_primary);
        f.paint_material_icon(add.inset(15.0), MaterialIcon::Add, p.on_primary);
        if add_target.clicked {
            let now_dt = crate::model::now_local_naive();
            let now = now_dt.time();
            let hour = if self.selected_day == now_dt.date() {
                (now.hour() + 1).min(23)
            } else {
                9
            };
            self.new_event(self.selected_day.and_hms_opt(hour, 0, 0).unwrap(), false);
        }
        action
    }

    fn horizontal_swipe(&mut self, f: &Frame<'_>, area: Rect) -> Option<bool> {
        let input = f.input();
        if input.pressed {
            self.press_origin = input.pointer.filter(|point| area.contains(*point));
        }
        if input.cancelled || input.blurred {
            self.press_origin = None;
        }
        if input.released
            && let (Some(old), Some(point)) = (self.press_origin.take(), input.pointer)
        {
            let dx = point[0] - old[0];
            let dy = (point[1] - old[1]).abs();
            if dx.abs() > 70.0 && dx.abs() > dy * 1.45 {
                return Some(dx < 0.0);
            }
        }
        None
    }

    fn scroll_week(&mut self, f: &mut Frame<'_>, area: Rect, id: u64) -> bool {
        let input = f.input();
        let col = (area.width - TIMELINE_GUTTER - TIMELINE_END_INSET) / 7.0;
        if col <= 0.0 {
            return false;
        }
        let inside = input.pointer.is_some_and(|point| area.contains(point));
        if input.pressed {
            self.week_drag = input.pointer.filter(|_| inside).map(|origin| WeekDrag {
                origin,
                last_x: origin[0],
                last_motion: input.time,
                velocity: 0.0,
                horizontal: false,
            });
        }
        let mut delta = if inside { input.scroll[0] } else { 0.0 };
        let mut dragging = false;
        if let (Some(drag), Some([x, y])) = (&mut self.week_drag, input.pointer) {
            let dx = x - drag.origin[0];
            let dy = y - drag.origin[1];
            if !drag.horizontal && dx.abs().max(dy.abs()) >= 8.0 {
                if dx.abs() > dy.abs() {
                    drag.horizontal = true;
                    f.cancel_scroll(id + 55);
                } else {
                    self.week_drag = None;
                }
            }
            if let Some(drag) = self.week_drag.as_mut().filter(|drag| drag.horizontal) {
                let movement = drag.last_x - x;
                if movement != 0.0 {
                    let elapsed = (input.time - drag.last_motion).max(0.008) as f32;
                    drag.velocity = (movement / elapsed).clamp(-2200.0, 2200.0);
                    drag.last_motion = input.time;
                    drag.last_x = x;
                    delta += movement;
                }
                dragging = true;
            }
        }
        if input.pressed
            || input.cancelled
            || input.blurred
            || input.scroll != [0.0; 2]
            || f.reduced_motion()
        {
            self.week_fling = None;
        }
        if input.released || input.cancelled || input.blurred {
            if let Some(drag) = self.week_drag.take()
                && input.released
                && !input.cancelled
                && !input.blurred
                && !f.reduced_motion()
                && drag.horizontal
                && input.time - drag.last_motion <= 0.08
                && drag.velocity.abs() >= 200.0
            {
                let mut fling = bhoo::scroll::SplineFling::default();
                fling.start(drag.velocity, input.time);
                self.week_fling = Some(fling);
            }
        }
        if let Some(mut fling) = self.week_fling.take() {
            let (movement, active) = fling.step(input.time);
            delta += movement;
            if active {
                self.week_fling = Some(fling);
                f.continuous(area);
            }
        }
        if delta != 0.0 && delta.is_finite() {
            let first = self
                .week_first
                .unwrap_or_else(|| week_start(self.selected_day));
            let earliest = week_start(NaiveDate::from_ymd_opt(1, 1, 1).unwrap());
            let latest = week_start(NaiveDate::from_ymd_opt(9998, 12, 31).unwrap());
            let requested = self.week_fraction + delta / col;
            let position = requested.clamp(
                (earliest - first).num_days() as f32,
                (latest - first).num_days() as f32,
            );
            if position != requested {
                self.week_fling = None;
            }
            let whole_days = position.floor() as i64;
            self.week_first = Some(first + Duration::days(whole_days));
            self.week_fraction = position - whole_days as f32;
            self.selected_day = (self.selected_day + Duration::days(whole_days)).clamp(
                NaiveDate::from_ymd_opt(1, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(9998, 12, 31).unwrap(),
            );
        }
        dragging
    }

    fn go_to_today(&mut self, f: &mut Frame<'_>, id: u64) {
        let now = model::now_local_naive();
        let date = now.date();
        let to_scroll = clock_scroll(now);
        if f.reduced_motion() || !matches!(self.mode, Mode::Day | Mode::Week) {
            self.select_day(date);
            self.scroll = to_scroll;
            return;
        }
        f.cancel_scroll(id + 55);
        self.today_transition = Some(TodayTransition {
            started: f.time(),
            date,
            from_day: self.selected_day,
            from_week: self
                .week_first
                .unwrap_or_else(|| week_start(self.selected_day)),
            from_fraction: self.week_fraction,
            from_scroll: self.scroll,
            to_scroll,
        });
    }

    fn update_today_transition(&mut self, f: &mut Frame<'_>, bounds: Rect) {
        let Some(motion) = self.today_transition.as_ref() else {
            return;
        };
        let t = ((f.time() - motion.started) / 0.36).clamp(0.0, 1.0) as f32;
        let scroll = motion.to_scroll;
        if t >= 1.0 || f.reduced_motion() {
            self.select_day(motion.date);
            self.scroll = scroll;
            return;
        }
        let eased = t * t * (3.0 - 2.0 * t);
        if self.mode == Mode::Week {
            let delta = (week_start(motion.date) - motion.from_week).num_days() as f32;
            let position = motion.from_fraction + (delta - motion.from_fraction) * eased;
            let whole = position.floor();
            self.week_first = Some(motion.from_week + Duration::days(whole as i64));
            self.week_fraction = position - whole;
        }
        let days = (motion.date - motion.from_day).num_days() as f32;
        self.selected_day = motion.from_day + Duration::days((days * eased).round() as i64);
        self.scroll = motion.from_scroll + (scroll - motion.from_scroll) * eased;
        f.continuous(bounds);
    }

    fn select_day(&mut self, day: NaiveDate) {
        self.today_transition = None;
        self.selected_day = day;
        self.week_first = None;
        self.week_fraction = 0.0;
        self.week_drag = None;
        self.week_fling = None;
        self.list_scroll = 0.0;
        self.message = None;
    }

    fn navigate(&mut self, forward: bool) {
        let delta = match self.mode {
            Mode::Day => 1,
            Mode::Week => 7,
            Mode::Month => 0,
            Mode::Agenda => 30,
        };
        let next = if delta == 0 {
            let months = chrono::Months::new(1);
            if forward {
                self.selected_day.checked_add_months(months)
            } else {
                self.selected_day.checked_sub_months(months)
            }
        } else if forward {
            self.selected_day.checked_add_days(Days::new(delta))
        } else {
            self.selected_day.checked_sub_days(Days::new(delta))
        };
        if let Some(day) = next.filter(|day| (1..=9998).contains(&day.year())) {
            if self.mode == Mode::Week {
                let change = day - self.selected_day;
                let first = self.week_first.map(|first| first + change);
                let fraction = self.week_fraction;
                self.select_day(day);
                self.week_first = first;
                self.week_fraction = fraction;
            } else {
                self.select_day(day);
            }
        }
    }

    fn open_occurrence(&mut self, item: &Occurrence) {
        self.detail = Some(Selected {
            id: self.store.data.events[item.index].id.clone(),
            start: item.start,
        });
        self.message = None;
    }

    fn paint_timeline(&mut self, f: &mut Frame<'_>, b: Rect, add: Rect, id: u64, p: Appearance) {
        let week = self.mode == Mode::Week;
        let from = if week {
            self.week_first
                .unwrap_or_else(|| week_start(self.selected_day))
        } else {
            self.selected_day
        };
        let visible_days = if week { 7 } else { 1 };
        let fraction = if week { self.week_fraction } else { 0.0 };
        let days = visible_days + usize::from(fraction > 0.0);
        let items = self
            .occurrences(from, from + Days::new(days as u64), String::new())
            .to_vec();
        let gutter = TIMELINE_GUTTER;
        let columns_width = (b.width - gutter - TIMELINE_END_INSET).max(0.0);
        let col = columns_width / visible_days as f32;
        let column_x = |day: usize| gutter + (day as f32 - fraction) * col;
        let today = crate::model::now_local_naive();
        let mut requested_day = None;
        f.paint_rect(rect(b, 0.0, 0.0, b.width, 45.0), 0.0, p.background);
        f.within(rect(b, gutter, 0.0, columns_width, 45.0), |f| {
            for day in 0..days {
                let date = from + Days::new(day as u64);
                let bounds = rect(b, column_x(day), 0.0, col, 43.0);
                let response = f.target(
                    id + 40 + day as u64,
                    bounds,
                    &date.format("%A, %B %-d").to_string(),
                    true,
                    Role::Button,
                );
                let ink = if date == today.date() {
                    p.primary
                } else {
                    p.muted
                };
                if date == today.date() {
                    f.paint_rect(bounds.inset(3.0), 10.0, p.accent_surface);
                }
                text(
                    f,
                    rect(bounds, 0.0, 1.0, col, 18.0),
                    &date.format("%a").to_string(),
                    ink,
                    11.0,
                    550,
                    1,
                );
                text(
                    f,
                    rect(bounds, 0.0, 17.0, col, 23.0),
                    &date.day().to_string(),
                    ink,
                    16.0,
                    650,
                    1,
                );
                if response.clicked {
                    requested_day = Some(date);
                }
            }
        });

        let bands: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.all_day)
            .collect();
        let all_day_height = if bands.is_empty() { 0.0 } else { 63.0 };
        if !bands.is_empty() {
            let band = rect(b, 0.0, 45.0, b.width, all_day_height);
            f.paint_rect(band, 0.0, p.surface);
            text(
                f,
                rect(band, 4.0, 0.0, gutter - 5.0, 21.0),
                "All day",
                p.muted,
                9.5,
                500,
                0,
            );
            f.within(rect(band, gutter, 0.0, columns_width, band.height), |f| {
                for day in 0..days {
                    let date = from + Days::new(day as u64);
                    for (row, (index, item)) in bands
                        .iter()
                        .filter(|(_, item)| item.covers(date))
                        .take(3)
                        .enumerate()
                    {
                        let bounds = rect(
                            band,
                            column_x(day) + 1.0,
                            row as f32 * 21.0,
                            col - 2.0,
                            20.0,
                        );
                        let event = &self.store.data.events[item.index];
                        let color = calendar_color(&self.store.data, &event.calendar);
                        let response = f.target(
                            id + 1300 + day as u64 * 8192 + *index as u64,
                            bounds,
                            &event.title,
                            true,
                            Role::Button,
                        );
                        f.paint_rect(bounds, 4.0, color);
                        f.state_layer(response, 4.0, p.on_primary);
                        text(
                            f,
                            bounds.inset(3.0),
                            &event.title,
                            Color(255, 255, 255, 255),
                            10.0,
                            550,
                            0,
                        );
                        if response.clicked {
                            self.open_occurrence(item);
                        }
                    }
                }
            });
        }

        let grid = rect(
            b,
            0.0,
            45.0 + all_day_height,
            b.width,
            (b.height - 45.0 - all_day_height).max(0.0),
        );
        let maximum = (24.0 * HOUR - grid.height).max(0.0);
        self.scroll = self.scroll.clamp(0.0, maximum);
        let grid_columns = rect(grid, gutter, 0.0, columns_width, grid.height);
        let input = f.input();
        if input.pressed {
            self.timeline_selection = None;
            if let Some(point) = input
                .pointer
                .filter(|point| grid_columns.contains(*point) && !add.contains(*point) && col > 0.0)
            {
                let day =
                    (((point[0] - grid_columns.x) / col + fraction).floor() as usize).min(days - 1);
                let date = from + Days::new(day as u64);
                let time = date.and_time(NaiveTime::MIN)
                    + Duration::minutes(((point[1] - grid.y + self.scroll) * 60.0 / HOUR) as i64);
                if !items.iter().any(|item| {
                    !item.all_day && item.display_start() <= time && item.display_end() > time
                }) {
                    let minute = timeline_minute(point[1], grid.y - self.scroll);
                    self.timeline_selection = Some(TimelineSelection {
                        origin: point,
                        deadline: input.time + 0.45,
                        date,
                        anchor_minute: minute,
                        end_minute: minute,
                        active: false,
                    });
                }
            }
        }
        if input.cancelled || input.blurred {
            self.timeline_selection = None;
        }
        if let Some(selection) = self.timeline_selection.as_mut() {
            if let Some([x, y]) = input.pointer {
                if selection.active {
                    selection.end_minute = timeline_minute(y, grid.y - self.scroll);
                } else if !input.released {
                    let [start_x, start_y] = selection.origin;
                    if (x - start_x).hypot(y - start_y) > 8.0 {
                        self.timeline_selection = None;
                    } else if input.time >= selection.deadline {
                        selection.active = true;
                        f.cancel_scroll(id + 55);
                    } else {
                        f.wake_at(selection.deadline);
                    }
                }
            }
        }
        let selecting = self
            .timeline_selection
            .as_ref()
            .is_some_and(|selection| selection.active);
        let mut selected = None;
        let mut create = None;
        f.cancel_pointer_input(selecting, |f| {
            f.scroll_viewport(id + 55, grid, &mut self.scroll, maximum);
            let top = grid.y - self.scroll;
            f.within(grid, |f| {
                for hour in 0..24_u32 {
                    let y = top + hour as f32 * HOUR;
                    if y < grid.y - HOUR || y > grid.y + grid.height + HOUR {
                        continue;
                    }
                    text(
                        f,
                        Rect {
                            x: grid.x + 8.0,
                            y: y + 1.0,
                            width: gutter - 10.0,
                            height: 19.0,
                        },
                        &format!("{hour:02}:00"),
                        p.muted,
                        10.0,
                        450,
                        0,
                    );
                }

                f.within(rect(grid, gutter, 0.0, columns_width, grid.height), |f| {
                    for day in 0..days {
                        let date = from + Days::new(day as u64);
                        let column_x = column_x(day);
                        if date == today.date() {
                            f.paint_rect(
                                rect(grid, column_x, 0.0, col, grid.height),
                                0.0,
                                p.accent_surface.opacity(0.43),
                            );
                        }
                        f.paint_rect(rect(grid, column_x, 0.0, 1.0, grid.height), 0.0, p.divider);
                    }
                    for hour in 0..24_u32 {
                        let y = top + hour as f32 * HOUR;
                        if (grid.y..grid.y + grid.height).contains(&y) {
                            f.paint_rect(
                                Rect {
                                    x: grid.x + gutter,
                                    y,
                                    width: columns_width,
                                    height: 1.0,
                                },
                                0.0,
                                p.divider,
                            );
                        }
                    }
                    for day in 0..days {
                        let date = from + Days::new(day as u64);
                        let day_start = date.and_time(NaiveTime::MIN);
                        let next_day = day_start + Duration::days(1);
                        let minutes =
                            |point: NaiveDateTime| (point - day_start).num_minutes() as f32;
                        let entries: Vec<_> = items
                            .iter()
                            .enumerate()
                            .filter(|(_, item)| !item.all_day && item.covers(date))
                            .collect();
                        let mut lanes: Vec<NaiveDateTime> = Vec::new();
                        let mut placements = Vec::new();
                        for (index, item) in entries {
                            let start = item.display_start().max(day_start);
                            let end = item.display_end().min(next_day);
                            let lane = lanes
                                .iter()
                                .position(|until| *until <= start)
                                .unwrap_or(lanes.len());
                            if lane == lanes.len() {
                                lanes.push(end);
                            } else {
                                lanes[lane] = end;
                            }
                            placements.push((index, item, lane, start, end));
                        }
                        let lane_w = col / lanes.len().max(1) as f32;
                        for (index, item, lane, start, end) in placements {
                            let y = top + minutes(start) * HOUR / 60.0;
                            let h = ((minutes(end) - minutes(start)) * HOUR / 60.0).max(23.0);
                            let bounds = Rect {
                                x: grid.x + column_x(day) + lane_w * lane as f32 + 1.0,
                                y,
                                width: (lane_w - 2.0).max(1.0),
                                height: h,
                            };
                            if y + h <= grid.y || y >= grid.y + grid.height {
                                continue;
                            }
                            let event = &self.store.data.events[item.index];
                            let response = f.target(
                                id + 100_000 + day as u64 * 32768 + index as u64,
                                bounds,
                                &event.title,
                                true,
                                Role::Button,
                            );
                            f.paint_rect(
                                bounds,
                                5.0,
                                calendar_color(&self.store.data, &event.calendar),
                            );
                            f.state_layer(response, 5.0, p.on_primary);
                            f.within(bounds, |f| {
                                text(
                                    f,
                                    rect(bounds, 4.0, 1.0, (bounds.width - 8.0).max(0.0), 23.0),
                                    &event.title,
                                    Color(255, 255, 255, 255),
                                    if days == 1 { 14.0 } else { 10.0 },
                                    600,
                                    0,
                                );
                                if days == 1 && bounds.height > 42.0 {
                                    text(
                                        f,
                                        rect(bounds, 4.0, 22.0, bounds.width - 8.0, 18.0),
                                        &short_time(item),
                                        Color(255, 255, 255, 235),
                                        11.0,
                                        450,
                                        0,
                                    );
                                }
                            });
                            if response.clicked {
                                selected = Some(item.clone());
                            }
                        }
                        if date == today.date() {
                            let y =
                                top + (today.hour() as f32 + today.minute() as f32 / 60.0) * HOUR;
                            f.paint_rect(
                                Rect {
                                    x: grid.x + column_x(day),
                                    y,
                                    width: col,
                                    height: 1.5,
                                },
                                0.75,
                                p.primary,
                            );
                        }
                        for hour in 0..24_u32 {
                            let y = top + hour as f32 * HOUR;
                            if y + HOUR <= grid.y || y >= grid.y + grid.height {
                                continue;
                            }
                            let bounds = Rect {
                                x: grid.x + column_x(day),
                                y,
                                width: col,
                                height: HOUR,
                            };
                            let label =
                                format!("New event {}, {hour:02}:00", date.format("%a %-d"));
                            if f.target(
                                id + 100 + day as u64 * 24 + u64::from(hour),
                                bounds,
                                &label,
                                true,
                                Role::Button,
                            )
                            .clicked
                            {
                                create = date.and_hms_opt(hour, 0, 0);
                            }
                        }
                    }
                });
            });
        });
        if let Some(selection) = self
            .timeline_selection
            .as_ref()
            .filter(|selection| selection.active)
        {
            let (start, end) = selection.minutes();
            let column = (selection.date - from).num_days() as usize;
            let top = grid.y - self.scroll;
            let bounds = Rect {
                x: grid.x + column_x(column) + 1.0,
                y: top + start as f32 * HOUR / 60.0,
                width: (col - 2.0).max(0.0),
                height: (end - start) as f32 * HOUR / 60.0,
            };
            f.within(grid_columns, |f| {
                f.paint_rect(bounds, 4.0, p.primary.opacity(0.75));
            });
        }
        if input.released {
            if let Some(selection) = self
                .timeline_selection
                .take()
                .filter(|selection| selection.active)
            {
                let (start, end) = selection.minutes();
                let midnight = selection.date.and_time(NaiveTime::MIN);
                self.new_event_range(
                    midnight + Duration::minutes(start),
                    midnight + Duration::minutes(end),
                    false,
                );
                f.wake_at(f.time() + 0.001);
                return;
            }
        }
        if let Some(date) = requested_day {
            if week {
                self.selected_day = date;
                self.message = None;
            } else {
                self.select_day(date);
            }
        }
        if let Some(item) = selected {
            self.open_occurrence(&item);
        } else if let Some(start) = create {
            self.new_event(start, false);
        }
    }

    fn paint_month(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: Appearance) {
        let first = self.selected_day.with_day(1).unwrap();
        let from = week_start(first);
        let items = self
            .occurrences(from, from + Days::new(42), String::new())
            .to_vec();
        let width = (b.width - 16.0) / 7.0;
        let height = ((b.height - 150.0) / 6.0).clamp(38.0, 73.0);
        let mut go_to = None;
        for i in 0..7 {
            text(
                f,
                rect(b, 8.0 + i as f32 * width, 1.0, width, 25.0),
                ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][i],
                p.muted,
                11.0,
                550,
                1,
            );
        }
        for i in 0..42 {
            let day = from + Days::new(i as u64);
            let bounds = rect(
                b,
                8.0 + (i % 7) as f32 * width,
                27.0 + (i / 7) as f32 * height,
                width,
                height,
            );
            let response = f.target(
                id + 40 + i as u64,
                bounds,
                &day.format("%A, %B %-d").to_string(),
                true,
                Role::Button,
            );
            if day == self.selected_day {
                f.paint_rect(bounds.inset(3.0), 11.0, p.accent_surface);
            }
            f.state_layer(response, 10.0, p.primary);
            let today = day == crate::model::now_local_naive().date();
            let ink = if today {
                p.primary
            } else if day.month() == first.month() {
                p.text
            } else {
                p.muted.opacity(0.6)
            };
            text(
                f,
                rect(bounds, 0.0, 3.0, width, 25.0),
                &day.day().to_string(),
                ink,
                14.0,
                if today { 700 } else { 500 },
                1,
            );
            for (index, item) in items
                .iter()
                .filter(|item| item.covers(day))
                .take(4)
                .enumerate()
            {
                let event = &self.store.data.events[item.index];
                f.paint_rect(
                    rect(bounds, 7.0 + index as f32 * 8.0, 32.0, 5.0, 5.0),
                    2.5,
                    calendar_color(&self.store.data, &event.calendar),
                );
            }
            if response.clicked {
                go_to = Some(day);
            }
        }
        if let Some(day) = go_to {
            self.select_day(day);
        }
        let y = 27.0 + 6.0 * height + 4.0;
        let area = rect(b, 0.0, y, b.width, (b.height - y).max(0.0));
        self.paint_list(
            f,
            area,
            id,
            items
                .iter()
                .filter(|item| item.covers(self.selected_day))
                .cloned()
                .collect(),
            &self.selected_day.format("%A, %B %-d").to_string(),
            "No events scheduled",
            p,
        );
    }

    fn paint_agenda(
        &mut self,
        f: &mut Frame<'_>,
        b: Rect,
        id: u64,
        from: NaiveDate,
        to: NaiveDate,
        searching: bool,
        p: Appearance,
    ) {
        let query = if searching {
            self.query.trim().to_lowercase()
        } else {
            String::new()
        };
        let events = self.occurrences(from, to, query).to_vec();
        self.paint_list(
            f,
            b,
            id,
            events,
            if searching {
                "Search results"
            } else {
                "Upcoming events"
            },
            if searching {
                "No matching events"
            } else {
                "No upcoming events"
            },
            p,
        );
    }

    fn paint_list(
        &mut self,
        f: &mut Frame<'_>,
        b: Rect,
        id: u64,
        items: Vec<Occurrence>,
        title: &str,
        empty: &str,
        p: Appearance,
    ) {
        text(
            f,
            rect(b, 16.0, 2.0, b.width - 32.0, 36.0),
            title,
            p.primary,
            13.0,
            650,
            0,
        );
        let area = rect(b, 0.0, 40.0, b.width, (b.height - 40.0).max(0.0));
        if items.is_empty() {
            text(
                f,
                rect(area, 18.0, 0.0, area.width - 36.0, 50.0),
                empty,
                p.muted,
                14.0,
                450,
                0,
            );
            return;
        }
        let max = (items.len() as f32 * 68.0 + 72.0 - area.height).max(0.0);
        self.list_scroll = self.list_scroll.clamp(0.0, max);
        f.scroll_viewport(id + 58, area, &mut self.list_scroll, max);
        let mut selected = None;
        f.within(area, |f| {
            for (index, item) in items
                .iter()
                .enumerate()
                .skip((self.list_scroll / 68.0).floor() as usize)
            {
                let y = index as f32 * 68.0 - self.list_scroll;
                if y >= area.height {
                    break;
                }
                let event = &self.store.data.events[item.index];
                let line = format!(
                    "{}  ·  {}",
                    item.display_start().format("%a, %b %-d"),
                    short_time(item)
                );
                let bounds = rect(area, 12.0, y + 2.0, area.width - 24.0, 62.0);
                f.paint_rect(bounds, 14.0, p.surface);
                f.paint_rect(
                    rect(bounds, 6.0, 11.0, 3.0, 40.0),
                    1.5,
                    calendar_color(&self.store.data, &event.calendar),
                );
                if row(
                    f,
                    id + 500_000 + index as u64,
                    bounds,
                    &event.title,
                    &line,
                    p,
                ) {
                    selected = Some(item.clone());
                }
            }
        });
        if let Some(item) = selected {
            self.open_occurrence(&item);
        }
    }

    fn apply_key(&mut self, f: &Frame<'_>, id: u64) {
        if f.focused_id() != Some(id) {
            return;
        }
        let Some(key) = f.input().key.filter(|key| key.pressed) else {
            return;
        };
        let Some(value) = self.edit_text(id) else {
            return;
        };
        let mut text = value.to_owned();
        match key.key {
            Key::Character('\u{8}' | '\u{7f}') => {
                text.pop();
            }
            Key::Character(ch) if !ch.is_control() => text.push(ch),
            Key::Space => text.push(' '),
            _ => return,
        }
        self.set_edit_text(id, &text);
    }

    fn paint_entering_event_form(
        &mut self,
        f: &mut Frame<'_>,
        bounds: Rect,
        id: u64,
        p: Appearance,
    ) -> Action {
        let time = f.time();
        let started = *self.editor_opened_at.get_or_insert(time);
        let progress = ((time - started) / 0.22).clamp(0.0, 1.0) as f32;
        if progress >= 1.0 || f.reduced_motion() {
            return self.paint_event_form(f, bounds, id, p);
        }

        let eased = bhoo::timing::Easing::StandardDecelerate.sample(progress);
        let entering = Rect {
            y: bounds.y + 24.0 * (1.0 - eased),
            ..bounds
        };
        f.continuous(bounds);
        let mut action = Action::None;
        f.with_opacity(0.35 + 0.65 * eased, |f| {
            action = self.paint_event_form(f, entering, id, p);
        });
        action
    }

    fn paint_event_form(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: Appearance) -> Action {
        let header = rect(b, 0.0, 0.0, b.width, TOOLBAR);
        f.paint_rect(header, 0.0, p.surface);
        if icon(
            f,
            id + 20,
            rect(header, 8.0, 6.0, 44.0, 44.0),
            "Discard changes",
            MaterialIcon::Close,
            p.text,
        ) {
            self.back();
            return Action::Blur;
        }
        let title = if self.draft.as_ref().unwrap().kind == EditKind::New {
            "New event"
        } else {
            "Edit event"
        };
        text(
            f,
            rect(header, 60.0, 0.0, b.width - 166.0, TOOLBAR),
            title,
            p.text,
            19.0,
            600,
            0,
        );
        if button(
            f,
            id + 21,
            rect(header, b.width - 96.0, 9.0, 80.0, 38.0),
            "Save",
            p,
            true,
        ) {
            return self.save_draft();
        }

        let area = rect(b, 0.0, TOOLBAR, b.width, b.height - TOOLBAR);
        let max = (665.0 - area.height).max(0.0);
        self.form_scroll = self.form_scroll.clamp(0.0, max);
        f.scroll_viewport(id + 22, area, &mut self.form_scroll, max);
        let content = Rect {
            y: area.y - self.form_scroll,
            ..area
        };
        let w = b.width - 28.0;
        let half = (w - 10.0) * 0.55;
        let mut action = Action::None;
        f.within(area, |f| {
            for (field, name, x, y, width) in [
                (0, "Title", 14.0, 12.0, w),
                (1, "Start date · YYYY-MM-DD", 14.0, 132.0, half),
                (2, "Start time · HH:MM", 24.0 + half, 132.0, w - half - 10.0),
                (3, "End date · YYYY-MM-DD", 14.0, 194.0, half),
                (4, "End time · HH:MM", 24.0 + half, 194.0, w - half - 10.0),
                (5, "Location", 14.0, 394.0, w),
                (6, "Notes", 14.0, 456.0, w),
            ] {
                let all_day = self.draft.as_ref().unwrap().all_day;
                if all_day && matches!(field, 2 | 4) {
                    continue;
                }
                self.apply_key(f, FIELD_BASE + field);
                let width = if all_day && matches!(field, 1 | 3) {
                    w
                } else {
                    width
                };
                let bounds = rect(content, x, y, width, if field == 6 { 75.0 } else { 51.0 });
                let date = matches!(field, 1 | 3);
                let calendar_button = rect(bounds, bounds.width - 38.0, 6.0, 34.0, 38.0);
                // Register the date-picker control first so it owns taps in its hit area.
                let pick = if date {
                    Some(f.target(
                        id + 80 + field,
                        calendar_button,
                        "Choose date",
                        true,
                        Role::Button,
                    ))
                } else {
                    None
                };
                let entry = self.edit_text(FIELD_BASE + field).unwrap_or_default();
                if layout_field(f, bounds, FIELD_BASE + field, name, entry, p) {
                    action = Action::Focus(FIELD_BASE + field);
                }
                if let Some(response) = pick {
                    f.state_layer(response, 16.0, p.primary);
                    f.paint_material_icon(
                        calendar_button.inset(8.0),
                        MaterialIcon::Inbox,
                        p.primary,
                    );
                    if response.clicked {
                        let selected = parse_date(entry).unwrap_or(self.selected_day);
                        self.open_date_picker(FIELD_BASE + field, selected);
                        action = Action::Blur;
                    }
                }
            }

            let draft = self.draft.as_mut().unwrap();
            if button(
                f,
                id + 23,
                rect(content, 14.0, 73.0, 112.0, 43.0),
                if draft.all_day {
                    "✓  All day"
                } else {
                    "All day"
                },
                p,
                draft.all_day,
            ) {
                draft.all_day = !draft.all_day;
                action = Action::Blur;
            }
            let calendar = self
                .store
                .data
                .calendars
                .iter()
                .find(|item| item.id == draft.calendar);
            let label = calendar.map_or("Personal", |item| item.name.as_str());
            let calendar_button = rect(content, 138.0, 73.0, (w - 124.0).max(1.0), 43.0);
            if row(f, id + 24, calendar_button, label, "", p) {
                self.calendars_scroll = 0.0;
                self.dialog = Dialog::EventOptions(EventOption::Calendar);
                action = Action::Blur;
            }
            let recurrence = RULES
                .iter()
                .find(|(rule, _)| *rule == draft.recurrence.as_deref())
                .map_or("Imported recurrence", |(_, text)| *text);
            if draft.kind != EditKind::Instance
                && row(
                    f,
                    id + 25,
                    rect(content, 14.0, 261.0, w, 60.0),
                    "Repeat",
                    recurrence,
                    p,
                )
            {
                self.dialog = Dialog::EventOptions(EventOption::Repeat);
                self.calendars_scroll = 0.0;
                action = Action::Blur;
            }
            let reminder = draft.reminders.first().copied();
            let caption = match reminder {
                None => "Off".into(),
                Some(0) => "At time of event".into(),
                Some(1440) => "1 day before".into(),
                Some(minutes) => format!("{minutes} min before"),
            };
            if row(
                f,
                id + 26,
                rect(content, 14.0, 323.0, w, 60.0),
                "Reminder",
                &caption,
                p,
            ) {
                self.dialog = Dialog::EventOptions(EventOption::Reminder);
                self.calendars_scroll = 0.0;
                action = Action::Blur;
            }
            let zone = if draft.all_day {
                "All-day dates follow the calendar".into()
            } else {
                format!("Time zone  ·  {}", draft.original.zone)
            };
            text(
                f,
                rect(content, 20.0, 540.0, w - 12.0, 34.0),
                &zone,
                p.muted,
                12.0,
                400,
                0,
            );
        });
        action
    }

    fn paint_event_detail(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: Appearance) {
        let selection = self.detail.clone().unwrap();
        let Some(event) = self.selected_event(&selection).cloned() else {
            self.detail = None;
            return;
        };
        let h = rect(b, 0.0, 0.0, b.width, TOOLBAR);
        f.paint_rect(h, 0.0, p.surface);
        if icon(
            f,
            id + 30,
            rect(h, 8.0, 6.0, 44.0, 44.0),
            "Back to calendar",
            MaterialIcon::ArrowBack,
            p.text,
        ) {
            self.back();
            return;
        }
        text(
            f,
            rect(h, 64.0, 0.0, b.width - 80.0, TOOLBAR),
            "Event details",
            p.text,
            19.0,
            600,
            0,
        );
        let occurrence = Occurrence {
            index: 0,
            start: selection.start,
            end: selection.start + event.length().unwrap_or(Duration::hours(1)),
            all_day: event.all_day,
        };
        let recurring =
            event.rule.is_some() || event.id.instance.is_some() || !event.added_dates.is_empty();
        let w = b.width - 36.0;
        let color = calendar_color(&self.store.data, &event.calendar);
        f.paint_rect(rect(b, 18.0, 75.0, 6.0, 59.0), 3.0, color);
        text(
            f,
            rect(b, 34.0, 76.0, w - 16.0, 46.0),
            &event.title,
            p.text,
            25.0,
            650,
            0,
        );
        let day = occurrence
            .display_start()
            .format("%A, %B %-d, %Y")
            .to_string();
        text(
            f,
            rect(b, 34.0, 123.0, w - 16.0, 32.0),
            &format!("{day} · {}", short_time(&occurrence)),
            p.muted,
            13.0,
            500,
            0,
        );
        let mut y = 160.0;
        let calendar = self
            .store
            .data
            .calendars
            .iter()
            .find(|row| row.id == event.calendar);
        for (name, value) in [
            (
                "Calendar",
                calendar.map(|item| item.name.clone()).unwrap_or_default(),
            ),
            ("Location", event.location.clone()),
            ("Notes", event.notes.clone()),
            (
                "Repeat",
                if recurring {
                    event
                        .rule
                        .as_ref()
                        .map_or("Series occurrence", |rule| {
                            if rule.contains("FREQ=DAILY") {
                                "Daily"
                            } else if rule.contains("FREQ=WEEKLY") {
                                "Weekly"
                            } else if rule.contains("FREQ=MONTHLY") {
                                "Monthly"
                            } else if rule.contains("FREQ=YEARLY") {
                                "Yearly"
                            } else {
                                "Repeating"
                            }
                        })
                        .into()
                } else {
                    String::new()
                },
            ),
            (
                "Reminder",
                event
                    .reminders
                    .iter()
                    .map(|minutes| format!("{minutes} min before"))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ] {
            if value.is_empty() {
                continue;
            }
            text(f, rect(b, 22.0, y, w, 19.0), name, p.primary, 11.5, 650, 0);
            let content = rect(b, 22.0, y + 18.0, w, 28.0);
            text(f, content, &value, p.text, 14.0, 450, 0);
            y += 52.0;
        }
        let width = (w - 8.0) * 0.5;
        for (index, (label, enabled)) in [
            (if recurring { "Edit this event" } else { "Edit" }, true),
            ("Edit series", recurring),
            (
                if recurring {
                    "Delete this event"
                } else {
                    "Delete"
                },
                true,
            ),
            ("Delete series", recurring),
            ("Export event", true),
        ]
        .into_iter()
        .enumerate()
        {
            if !enabled {
                continue;
            }
            let bounds = rect(
                b,
                18.0 + (index % 2) as f32 * (width + 8.0),
                y + 48.0 * (index / 2) as f32,
                width,
                42.0,
            );
            if button(f, id + 31 + index as u64, bounds, label, p, index == 0) {
                match index {
                    0 => self.edit_selected(&selection, false),
                    1 => self.edit_selected(&selection, true),
                    2 | 3 => {
                        self.dialog = Dialog::ConfirmDelete {
                            selection: selection.clone(),
                            whole: index == 3 || !recurring,
                        }
                    }
                    _ => self.begin_file(FileTask::Export(Some(event.id.uid.clone()))),
                }
            }
        }
    }

    fn paint_view_drawer(&mut self, f: &mut Frame<'_>, b: Rect, id: u64) {
        let width = (b.width - 64.0).clamp(0.0, 360.0);
        f.overlay(
            id + 6999,
            &mut self.view_drawer,
            rect(b, 0.0, 0.0, width, b.height),
            true,
            |f, open| {
                f.with_gap(0.0, |f| {
                    f.navigation_drawer_header("Calendar", None);
                    for (index, (mode, icon)) in [
                        (Mode::Day, MaterialIcon::Description),
                        (Mode::Week, MaterialIcon::Widgets),
                        (Mode::Month, MaterialIcon::Widgets),
                        (Mode::Agenda, MaterialIcon::VerticalAlignTop),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let row = f.reserve(56.0);
                        if f.material_drawer_item(
                            id + 7000 + index as u64,
                            row,
                            mode.title(),
                            icon,
                            mode == self.mode,
                            true,
                        )
                        .clicked
                        {
                            self.mode = mode;
                            self.list_scroll = 0.0;
                            *open = false;
                        }
                    }
                });
            },
        );
    }

    fn paint_options_sheet(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: Appearance) {
        const OPTIONS: [&str; 5] = [
            "Search events",
            "Go to date",
            "Manage calendars",
            "Import calendar file",
            "Export calendar file",
        ];
        let height = b.height.min(362.0);
        let bounds = rect(b, 0.0, b.height - height, b.width, height);
        let mut selection = None;
        f.overlay(
            id + 7009,
            &mut self.options_sheet,
            bounds,
            true,
            |f, open| {
                f.with_gap(0.0, |f| {
                    f.bottom_sheet_drag_handle();
                    let row_height = ((height - 114.0) / OPTIONS.len() as f32).clamp(0.0, 48.0);
                    let title = f.reserve(38.0);
                    let theme = f.theme();
                    text(
                        f,
                        rect(title, 24.0, 0.0, title.width - 48.0, title.height),
                        "Calendar options",
                        theme.text,
                        21.0,
                        600,
                        0,
                    );
                    let sheet_colors = Appearance {
                        text: theme.text,
                        primary: theme.primary,
                        ..p
                    };
                    for (i, label) in OPTIONS.into_iter().enumerate() {
                        let slot = f.reserve(row_height);
                        let row_bounds = rect(slot, 12.0, 0.0, slot.width - 24.0, slot.height);
                        if row(f, id + 7010 + i as u64, row_bounds, label, "", sheet_colors) {
                            selection = Some(i);
                            *open = false;
                        }
                    }
                    let footer = f.reserve(48.0);
                    if button(
                        f,
                        id + 7015,
                        rect(footer, 24.0, 4.0, 88.0, 36.0),
                        "Close",
                        p,
                        false,
                    ) {
                        *open = false;
                    }
                });
            },
        );
        match selection {
            Some(0) => {
                self.search = true;
                self.list_scroll = 0.0;
            }
            Some(1) => {
                self.date_input = self.selected_day.format("%Y-%m-%d").to_string();
                self.dialog = Dialog::Date;
            }
            Some(2) => {
                self.calendars_scroll = 0.0;
                self.dialog = Dialog::Calendars;
            }
            Some(3) => self.begin_file(FileTask::Import),
            Some(4) => self.begin_file(FileTask::Export(None)),
            _ => {}
        }
    }

    fn paint_dialog(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: Appearance) -> Action {
        f.paint_rect(b, 0.0, Color(0, 0, 0, 110));
        let mut action = Action::None;
        let width = (b.width - 36.0).min(406.0);
        let height = match self.dialog {
            Dialog::Calendars => (b.height - 90.0).min(510.0),
            Dialog::CalendarForm(_) => 270.0,
            Dialog::Date => 206.0,
            Dialog::DatePicker { .. } => b.height.min(522.0),
            Dialog::EventOptions(_) => (b.height - 54.0).min(455.0),
            Dialog::ConfirmDelete { .. } => 175.0,
            Dialog::None => return action,
        };
        let panel = rect(
            b,
            (b.width - width) * 0.5,
            (b.height - height).max(0.0) * 0.32,
            width,
            height,
        );
        f.paint_rect(panel, 23.0, p.background);
        let title = match self.dialog {
            Dialog::Calendars => "Your calendars",
            Dialog::CalendarForm(Some(_)) => "Edit calendar",
            Dialog::CalendarForm(None) => "New calendar",
            Dialog::Date => "Go to date",
            Dialog::EventOptions(EventOption::Calendar) => "Choose calendar",
            Dialog::EventOptions(EventOption::Repeat) => "Repeat",
            Dialog::EventOptions(EventOption::Reminder) => "Reminder",
            Dialog::ConfirmDelete { whole: true, .. } => "Delete event?",
            Dialog::ConfirmDelete { .. } => "Delete this occurrence?",
            _ => "",
        };
        text(
            f,
            rect(panel, 20.0, 10.0, width - 40.0, 40.0),
            title,
            p.text,
            21.0,
            600,
            0,
        );

        let dialog = std::mem::replace(&mut self.dialog, Dialog::None);
        self.dialog = match dialog {
            Dialog::Calendars => {
                let area = rect(panel, 12.0, 49.0, width - 24.0, height - 103.0);
                let max = (self.store.data.calendars.len() as f32 * 55.0 - area.height).max(0.0);
                self.calendars_scroll = self.calendars_scroll.clamp(0.0, max);
                f.scroll_viewport(id + 7020, area, &mut self.calendars_scroll, max);
                let mut edit = None;
                let mut toggle = None;
                f.within(area, |f| {
                    for (i, item) in self.store.data.calendars.iter().enumerate() {
                        let bounds = rect(
                            area,
                            3.0,
                            i as f32 * 55.0 - self.calendars_scroll,
                            area.width - 6.0,
                            52.0,
                        );
                        if bounds.y + bounds.height < area.y || bounds.y >= area.y + area.height {
                            continue;
                        }
                        let label = if item.visible { "Shown" } else { "Hidden" };
                        f.paint_rect(rect(bounds, 4.0, 13.0, 14.0, 14.0), 7.0, swatch(item.color));
                        if row(
                            f,
                            id + 7100 + i as u64 * 2,
                            rect(bounds, 13.0, 0.0, bounds.width - 65.0, 52.0),
                            &item.name,
                            label,
                            p,
                        ) {
                            toggle = Some(item.id.clone());
                        }
                        if icon(
                            f,
                            id + 7101 + i as u64 * 2,
                            rect(bounds, bounds.width - 42.0, 5.0, 40.0, 40.0),
                            "Edit calendar",
                            MaterialIcon::Edit,
                            p.primary,
                        ) {
                            edit = Some(item.clone());
                        }
                    }
                });
                if let Some(key) = toggle {
                    if let Err(error) = self.commit(|data| {
                        let item = data
                            .calendars
                            .iter_mut()
                            .find(|row| row.id == key)
                            .ok_or_else(|| io::Error::other("Calendar was removed"))?;
                        item.visible = !item.visible;
                        Ok(())
                    }) {
                        self.message = Some(error.to_string());
                    }
                }
                let add = button(
                    f,
                    id + 7021,
                    rect(panel, width - 143.0, height - 47.0, 129.0, 35.0),
                    "New calendar",
                    p,
                    true,
                );
                if let Some(item) = edit {
                    self.new_name = item.name;
                    self.new_color = item.color;
                    Dialog::CalendarForm(Some(item.id))
                } else if add {
                    self.new_name.clear();
                    self.new_color = self.store.data.calendars.len() % model::COLORS.len();
                    Dialog::CalendarForm(None)
                } else {
                    Dialog::Calendars
                }
            }
            Dialog::CalendarForm(key) => {
                self.apply_key(f, FIELD_BASE + 8);
                if layout_field(
                    f,
                    rect(panel, 16.0, 58.0, width - 32.0, 53.0),
                    FIELD_BASE + 8,
                    "Calendar name",
                    &self.new_name,
                    p,
                ) {
                    action = Action::Focus(FIELD_BASE + 8);
                }
                let spacing = (width - 32.0) / model::COLORS.len() as f32;
                for i in 0..model::COLORS.len() {
                    let bounds = rect(
                        panel,
                        16.0 + i as f32 * spacing,
                        126.0,
                        spacing,
                        spacing.min(45.0),
                    );
                    let response = f.target(
                        id + 7030 + i as u64,
                        bounds,
                        "Calendar color",
                        true,
                        Role::Radio,
                    );
                    let circle = bounds.inset(7.0);
                    f.paint_rect(circle, 18.0, swatch(i));
                    if self.new_color == i {
                        f.paint_material_icon(
                            circle.inset(4.0),
                            MaterialIcon::Check,
                            Color(255, 255, 255, 255),
                        );
                    }
                    if response.clicked {
                        self.new_color = i;
                    }
                }
                if button(
                    f,
                    id + 7037,
                    rect(panel, width - 106.0, height - 48.0, 89.0, 36.0),
                    "Save",
                    p,
                    true,
                ) {
                    self.save_calendar(key.clone());
                    action = Action::Blur;
                    if matches!(self.dialog, Dialog::Calendars) {
                        Dialog::Calendars
                    } else {
                        Dialog::CalendarForm(key)
                    }
                } else {
                    Dialog::CalendarForm(key)
                }
            }
            Dialog::Date => {
                self.apply_key(f, FIELD_BASE + 9);
                if layout_field(
                    f,
                    rect(panel, 16.0, 59.0, width - 32.0, 54.0),
                    FIELD_BASE + 9,
                    "Date · YYYY-MM-DD",
                    &self.date_input,
                    p,
                ) {
                    action = Action::Focus(FIELD_BASE + 9);
                }
                if button(
                    f,
                    id + 7040,
                    rect(panel, width - 107.0, height - 48.0, 89.0, 36.0),
                    "Go",
                    p,
                    true,
                ) {
                    action = Action::Blur;
                    if let Some(date) = parse_date(&self.date_input) {
                        self.select_day(date);
                        Dialog::None
                    } else {
                        self.message = Some("Use YYYY-MM-DD".into());
                        Dialog::Date
                    }
                } else {
                    Dialog::Date
                }
            }
            Dialog::DatePicker { field } => self.paint_date_picker(f, panel, id, p, field),
            Dialog::EventOptions(kind) => self.paint_event_options(f, panel, id, p, kind),
            Dialog::ConfirmDelete { selection, whole } => {
                text(
                    f,
                    rect(panel, 20.0, 54.0, width - 40.0, 49.0),
                    if whole {
                        "This removes the event and any repeats."
                    } else {
                        "The rest of the series stays on your calendar."
                    },
                    p.muted,
                    13.0,
                    450,
                    0,
                );
                if button(
                    f,
                    id + 7041,
                    rect(panel, width - 106.0, height - 48.0, 89.0, 36.0),
                    "Delete",
                    p,
                    true,
                ) {
                    self.delete(selection, whole);
                    Dialog::None
                } else {
                    Dialog::ConfirmDelete { selection, whole }
                }
            }
            Dialog::None => Dialog::None,
        };
        if button(
            f,
            id + 7099,
            rect(panel, 16.0, height - 48.0, 88.0, 36.0),
            if matches!(self.dialog, Dialog::DatePicker { .. }) {
                "Cancel"
            } else {
                "Close"
            },
            p,
            false,
        ) {
            self.back();
            action = Action::Blur;
        }
        action
    }
    fn open_date_picker(&mut self, field: u64, date: NaiveDate) {
        let min = CalendarDate {
            year: 1,
            month: 1,
            day: 1,
        };
        let max = CalendarDate {
            year: 9998,
            month: 12,
            day: 31,
        };
        let selected = CalendarDate {
            year: date.year(),
            month: date.month() as u8,
            day: date.day() as u8,
        }
        .clamp(min, max);
        self.date_picker = Default::default();
        let first_day = self.date_picker.calendar_locale.first_day_of_week;
        self.date_picker.calendars.insert(
            event_date_picker::LAYOUT.id.into(),
            CalendarState::new(min, max, selected, first_day).expect("valid Calendar date range"),
        );
        self.dialog = Dialog::DatePicker { field };
    }

    fn paint_date_picker(
        &mut self,
        f: &mut Frame<'_>,
        panel: Rect,
        id: u64,
        p: Appearance,
        field: u64,
    ) -> Dialog {
        let background = f.theme().background;
        self.date_picker.theme_style =
            if u16::from(background.0) + u16::from(background.1) + u16::from(background.2) < 384 {
                AndroidTheme::Material3Dark
            } else {
                AndroidTheme::Material3Light
            };
        self.date_picker.wall_clock = Some(WallClock {
            unix_seconds: model::now_local_naive().and_utc().timestamp() as f64,
            offset_seconds: 0,
            is_24_hour: true,
        });
        let bounds = rect(
            panel,
            8.0,
            8.0,
            panel.width - 16.0,
            (panel.height - 60.0).max(0.0),
        );
        event_date_picker::LAYOUT.render(f, bounds, &mut self.date_picker);

        if button(
            f,
            id + 7210,
            rect(panel, panel.width - 104.0, panel.height - 48.0, 88.0, 36.0),
            "OK",
            p,
            true,
        ) {
            let selected = self.date_picker.calendars[event_date_picker::LAYOUT.id].selected();
            if let Some(day) =
                NaiveDate::from_ymd_opt(selected.year, selected.month.into(), selected.day.into())
            {
                self.set_edit_text(field, &day.format("%Y-%m-%d").to_string());
                if field == FIELD_BASE + 1
                    && let Some(draft) = self.draft.as_mut()
                    && parse_date(&draft.end_date).is_none_or(|end| end < day)
                {
                    draft.end_date.clone_from(&draft.start_date);
                }
            }
            return Dialog::None;
        }
        Dialog::DatePicker { field }
    }

    fn paint_event_options(
        &mut self,
        f: &mut Frame<'_>,
        panel: Rect,
        id: u64,
        p: Appearance,
        kind: EventOption,
    ) -> Dialog {
        let draft = self.draft.as_mut().unwrap();
        let (options, active): (Vec<&str>, _) = match kind {
            EventOption::Calendar => (
                self.store
                    .data
                    .calendars
                    .iter()
                    .map(|item| item.name.as_str())
                    .collect(),
                self.store
                    .data
                    .calendars
                    .iter()
                    .position(|item| item.id == draft.calendar),
            ),
            EventOption::Repeat => (
                RULES.iter().map(|(_, label)| *label).collect(),
                RULES
                    .iter()
                    .position(|(rule, _)| *rule == draft.recurrence.as_deref()),
            ),
            EventOption::Reminder => (
                REMINDERS.iter().map(|(_, label)| *label).collect(),
                REMINDERS
                    .iter()
                    .position(|(option, _)| *option == draft.reminders.first().copied()),
            ),
        };
        let area = rect(
            panel,
            16.0,
            52.0,
            panel.width - 32.0,
            (panel.height - 108.0).max(0.0),
        );
        let max = (options.len() as f32 * 45.0 - area.height).max(0.0);
        f.scroll_viewport(id + 7280, area, &mut self.calendars_scroll, max);
        let mut selected = None;
        f.within(area, |f| {
            for (i, label) in options.into_iter().enumerate() {
                let bounds = rect(
                    area,
                    0.0,
                    i as f32 * 45.0 - self.calendars_scroll,
                    area.width,
                    41.0,
                );
                if button(f, id + 7290 + i as u64, bounds, label, p, active == Some(i)) {
                    selected = Some(i);
                }
            }
        });
        if let Some(index) = selected {
            match kind {
                EventOption::Calendar => draft
                    .calendar
                    .clone_from(&self.store.data.calendars[index].id),
                EventOption::Repeat => draft.recurrence = RULES[index].0.map(str::to_owned),
                EventOption::Reminder => draft.reminders = REMINDERS[index].0.into_iter().collect(),
            }
            Dialog::None
        } else {
            Dialog::EventOptions(kind)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn views_and_editor_render_a_saved_event_at_phone_size() {
        let mut ui = bhoo::Ui::default();
        let mut app = CalendarApp::default();
        app.store = Store::open(None);
        let day = NaiveDate::from_ymd_opt(2026, 11, 24).unwrap();
        app.select_day(day);
        app.new_event(day.and_hms_opt(14, 0, 0).unwrap(), false);
        assert!(app.set_edit_text(FIELD_BASE, "Test workshop"));
        assert!(matches!(app.save_draft(), Action::Blur));
        assert_eq!(app.store.data.events.len(), 1);
        assert_eq!(app.store.data.events[0].title, "Test workshop");
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 777.0,
        };
        let id = 0x4341_4c00_1000_0000;
        for (i, mode) in [Mode::Day, Mode::Week, Mode::Month, Mode::Agenda]
            .into_iter()
            .enumerate()
        {
            app.mode = mode;
            let input = bhoo::Input {
                time: i as f64,
                ..Default::default()
            };
            ui.frame(input, bounds, 0.0, |f| {
                let _ = app.show(f, bounds, id, Appearance::default());
            });
        }
        app.new_event(day.and_hms_opt(14, 0, 0).unwrap(), false);
        app.open_date_picker(FIELD_BASE + 1, day);
        for (i, dialog) in [
            Dialog::DatePicker {
                field: FIELD_BASE + 1,
            },
            Dialog::EventOptions(EventOption::Reminder),
        ]
        .into_iter()
        .enumerate()
        {
            app.dialog = dialog;
            let input = bhoo::Input {
                time: i as f64 + 5.0,
                ..Default::default()
            };
            ui.frame(input, bounds, 0.0, |f| {
                let _ = app.show(f, bounds, id, Appearance::default());
            });
        }
    }
}
