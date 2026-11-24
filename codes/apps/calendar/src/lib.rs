mod event_date_picker;
mod ics;
mod model;
mod view;

use bhoo::systems::material::{OverlayKind, OverlayState, OverlayStyle};
use chrono::{DateTime, Days, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use model::{Calendar, Event, EventId, Occurrence, Store, id, local_zone};
use std::{io, path::PathBuf};

#[cfg(target_arch = "wasm32")]
pub use model::set_wasm_clock;
pub use view::Appearance;
pub const FIELD_BASE: u64 = 0x4341_4C00_0000_1100;
pub const FIELD_COUNT: u64 = 12;

pub fn field_is_calendar(id: u64) -> bool {
    (FIELD_BASE..FIELD_BASE + FIELD_COUNT).contains(&id)
}

pub enum Action {
    None,
    Focus(u64),
    Blur,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Day,
    Week,
    Month,
    Agenda,
}

impl Mode {
    fn title(self) -> &'static str {
        match self {
            Self::Day => "Day",
            Self::Week => "Week",
            Self::Month => "Month",
            Self::Agenda => "Agenda",
        }
    }
}

#[derive(Clone, Debug)]
struct Selected {
    id: EventId,
    start: DateTime<Utc>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditKind {
    New,
    Series,
    Instance,
}

struct Draft {
    kind: EditKind,
    original: Event,
    title: String,
    location: String,
    notes: String,
    start_date: String,
    end_date: String,
    start_time: String,
    end_time: String,
    all_day: bool,
    calendar: String,
    recurrence: Option<String>,
    reminders: Vec<u32>,
}

impl Draft {
    fn from(event: Event, kind: EditKind) -> Self {
        let end = if event.all_day {
            event.end.date() - Days::new(1)
        } else {
            event.end.date()
        };
        Self {
            kind,
            title: event.title.clone(),
            location: event.location.clone(),
            notes: event.notes.clone(),
            start_date: event.start.date().format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
            start_time: event.start.format("%H:%M").to_string(),
            end_time: event.end.format("%H:%M").to_string(),
            all_day: event.all_day,
            recurrence: event.rule.clone(),
            reminders: event.reminders.clone(),
            calendar: event.calendar.clone(),
            original: event,
        }
    }

    fn validate(&self) -> io::Result<Event> {
        if self.title.trim().is_empty() {
            return Err(io::Error::other("Enter an event title"));
        }
        let date = |text: &str| {
            parse_date(text).ok_or_else(|| io::Error::other("Use YYYY-MM-DD for dates"))
        };
        let start = date(&self.start_date)?;
        let end = date(&self.end_date)?;
        let (start, end, zone) = if self.all_day {
            (
                start.and_time(NaiveTime::MIN),
                (end + Days::new(1)).and_time(NaiveTime::MIN),
                chrono_tz::UTC,
            )
        } else {
            let time = |text: &str| {
                NaiveTime::parse_from_str(text.trim(), "%H:%M")
                    .map_err(|_| io::Error::other("Use HH:MM for times"))
            };
            let zone = if self.original.all_day {
                local_zone()
            } else {
                self.original.zone
            };
            (
                start.and_time(time(&self.start_time)?),
                end.and_time(time(&self.end_time)?),
                zone,
            )
        };
        let mut event = self.original.clone();
        event.title = self.title.trim().into();
        event.location = self.location.trim().into();
        event.notes = self.notes.trim().into();
        event.start = start;
        event.end = end;
        event.zone = zone;
        event.calendar.clone_from(&self.calendar);
        event.all_day = self.all_day;
        event.rule.clone_from(&self.recurrence);
        event.reminders.clone_from(&self.reminders);
        if !event.valid() {
            return Err(io::Error::other(
                "End must be after start. Check the date, time and daylight-saving transition.",
            ));
        }
        if let Some(rule) = &event.rule {
            format!(
                "DTSTART;TZID={zone}:{}\nRRULE:{rule}",
                start.format("%Y%m%dT%H%M%S")
            )
            .parse::<rrule::RRuleSet>()
            .map_err(|_| io::Error::other("This start date is incompatible with the recurrence"))?;
        }
        Ok(event)
    }
}

enum FileTask {
    Import,
    Export(Option<String>),
}
#[derive(Clone, Copy)]
enum EventOption {
    Calendar,
    Repeat,
    Reminder,
}

enum Dialog {
    None,
    Calendars,
    CalendarForm(Option<String>),
    Date,
    DatePicker { field: u64 },
    EventOptions(EventOption),
    ConfirmDelete { selection: Selected, whole: bool },
}

struct RangeCache {
    from: NaiveDate,
    to: NaiveDate,
    query: String,
    events: Vec<Occurrence>,
}

struct WeekDrag {
    origin: [f32; 2],
    last_x: f32,
    last_motion: f64,
    velocity: f32,
    horizontal: bool,
}

struct TimelineSelection {
    origin: [f32; 2],
    deadline: f64,
    date: NaiveDate,
    anchor_minute: i64,
    end_minute: i64,
    active: bool,
}

struct TodayTransition {
    started: f64,
    date: NaiveDate,
    from_day: NaiveDate,
    from_week: NaiveDate,
    from_fraction: f32,
    from_scroll: f32,
    to_scroll: f32,
}

pub struct CalendarApp {
    store: Store,
    mode: Mode,
    selected_day: NaiveDate,
    draft: Option<Draft>,
    editor_opened_at: Option<f64>,
    detail: Option<Selected>,
    dialog: Dialog,
    view_drawer: OverlayState,
    options_sheet: OverlayState,
    date_picker: bhoo::android_xml::LayoutState,
    picker: spectre_files::Browser,
    file_task: Option<FileTask>,
    search: bool,
    query: String,
    new_name: String,
    new_color: usize,
    date_input: String,
    message: Option<String>,
    scroll: f32,
    list_scroll: f32,
    form_scroll: f32,
    calendars_scroll: f32,
    cache: Option<RangeCache>,
    #[cfg(not(target_arch = "wasm32"))]
    last_refresh: Option<std::time::Instant>,
    #[cfg(not(target_arch = "wasm32"))]
    last_reminder: Option<std::time::Instant>,
    press_origin: Option<[f32; 2]>,
    week_first: Option<NaiveDate>,
    week_fraction: f32,
    week_drag: Option<WeekDrag>,
    week_fling: Option<bhoo::scroll::SplineFling>,
    today_transition: Option<TodayTransition>,
    timeline_selection: Option<TimelineSelection>,
}

impl Default for CalendarApp {
    fn default() -> Self {
        let now = model::now_local_naive();
        Self {
            store: Store::default(),
            mode: Mode::Week,
            selected_day: now.date(),
            draft: None,
            editor_opened_at: None,
            detail: None,
            dialog: Dialog::None,
            view_drawer: {
                let mut drawer = OverlayState::default();
                drawer.kind = OverlayKind::LeftSideSheet;
                drawer.style = OverlayStyle::NavigationDrawer;
                drawer
            },
            options_sheet: {
                let mut sheet = OverlayState::default();
                sheet.kind = OverlayKind::BottomSheet;
                sheet.drag_to_dismiss = true;
                sheet
            },
            date_picker: Default::default(),
            picker: spectre_files::Browser::default(),
            file_task: None,
            search: false,
            query: String::new(),
            new_name: String::new(),
            new_color: 0,
            date_input: String::new(),
            message: None,
            scroll: view::clock_scroll(now),
            list_scroll: 0.0,
            form_scroll: 0.0,
            calendars_scroll: 0.0,
            cache: None,
            #[cfg(not(target_arch = "wasm32"))]
            last_refresh: None,
            #[cfg(not(target_arch = "wasm32"))]
            last_reminder: None,
            press_origin: None,
            week_first: None,
            week_fraction: 0.0,
            week_drag: None,
            week_fling: None,
            today_transition: None,
            timeline_selection: None,
        }
    }
}

fn parse_date(text: &str) -> Option<NaiveDate> {
    use chrono::Datelike;
    let date = NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()?;
    (date.year() >= 1 && date.year() <= 9998).then_some(date)
}

impl CalendarApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn refresh(&mut self) {
        if self
            .last_refresh
            .is_some_and(|stamp| stamp.elapsed().as_millis() < 750)
        {
            return;
        }
        self.last_refresh = Some(std::time::Instant::now());
        if self.store.refresh() {
            self.cache = None;
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn refresh(&mut self) {}

    fn commit<T>(&mut self, edit: impl FnOnce(&mut model::Data) -> io::Result<T>) -> io::Result<T> {
        let result = self.store.update(edit)?;
        self.cache = None;
        Ok(result)
    }

    pub fn back(&mut self) -> bool {
        self.message = None;
        if self.file_task.is_some() {
            if !self.picker.back() {
                self.file_task = None;
            }
        } else if self.options_sheet.open {
            self.options_sheet.open = false;
        } else if self.view_drawer.open {
            self.view_drawer.open = false;
        } else if !matches!(self.dialog, Dialog::None) {
            self.dialog = match self.dialog {
                Dialog::CalendarForm(_) => Dialog::Calendars,
                _ => Dialog::None,
            };
        } else if self.draft.take().is_some() || self.detail.take().is_some() {
        } else if self.search {
            self.search = false;
            self.query.clear();
            self.list_scroll = 0.0;
        } else {
            return false;
        }
        true
    }

    pub fn edit_text(&self, field: u64) -> Option<&str> {
        if self.file_task.is_some() {
            return self.picker.edit_text(field);
        }
        let field = field.checked_sub(FIELD_BASE)?;
        match field {
            7 => Some(&self.query),
            8 => Some(&self.new_name),
            9 => Some(&self.date_input),
            _ => {
                let draft = self.draft.as_ref()?;
                match field {
                    0 => Some(&draft.title),
                    1 => Some(&draft.start_date),
                    2 => Some(&draft.start_time),
                    3 => Some(&draft.end_date),
                    4 => Some(&draft.end_time),
                    5 => Some(&draft.location),
                    6 => Some(&draft.notes),
                    _ => None,
                }
            }
        }
    }

    pub fn set_edit_text(&mut self, field: u64, value: &str) -> bool {
        if self.file_task.is_some() {
            return self.picker.set_edit_text(field, value);
        }
        let Some(field) = field.checked_sub(FIELD_BASE) else {
            return false;
        };
        let entry = match field {
            7 => {
                self.list_scroll = 0.0;
                &mut self.query
            }
            8 => &mut self.new_name,
            9 => &mut self.date_input,
            _ => {
                let Some(draft) = self.draft.as_mut() else {
                    return false;
                };
                match field {
                    0 => &mut draft.title,
                    1 => &mut draft.start_date,
                    2 => &mut draft.start_time,
                    3 => &mut draft.end_date,
                    4 => &mut draft.end_time,
                    5 => &mut draft.location,
                    6 => &mut draft.notes,
                    _ => return false,
                }
            }
        };
        *entry = value.chars().take(4096).collect();
        true
    }

    pub fn choosing_file(&self) -> bool {
        self.file_task.is_some()
    }

    fn calendar_for_new(&self) -> String {
        self.store
            .data
            .calendars
            .iter()
            .find(|item| item.visible)
            .or_else(|| self.store.data.calendars.first())
            .map(|calendar| calendar.id.clone())
            .unwrap_or_else(|| "personal".into())
    }

    fn new_event(&mut self, start: NaiveDateTime, all_day: bool) {
        let start = if all_day {
            start.date().and_time(NaiveTime::MIN)
        } else {
            start
        };
        let end = start
            + if all_day {
                Duration::days(1)
            } else {
                Duration::hours(1)
            };
        self.new_event_range(start, end, all_day);
    }

    fn new_event_range(&mut self, start: NaiveDateTime, end: NaiveDateTime, all_day: bool) {
        let event = Event {
            id: EventId {
                uid: id(),
                instance: None,
            },
            title: String::new(),
            location: String::new(),
            notes: String::new(),
            calendar: self.calendar_for_new(),
            start,
            end,
            zone: if all_day {
                chrono_tz::UTC
            } else {
                local_zone()
            },
            all_day,
            rule: None,
            added_dates: vec![],
            exceptions: vec![],
            reminders: vec![15],
        };
        self.start_draft(event, EditKind::New);
    }

    fn start_draft(&mut self, event: Event, kind: EditKind) {
        self.dialog = Dialog::None;
        self.message = None;
        self.form_scroll = 0.0;
        self.editor_opened_at = None;
        self.draft = Some(Draft::from(event, kind));
    }

    fn selected_event(&self, selection: &Selected) -> Option<&Event> {
        self.store
            .data
            .events
            .iter()
            .find(|event| event.id == selection.id)
    }

    fn edit_selected(&mut self, selection: &Selected, whole: bool) {
        let Some(mut event) = self.selected_event(selection).cloned() else {
            return;
        };
        let recurring =
            event.rule.is_some() || event.id.instance.is_some() || !event.added_dates.is_empty();
        if whole {
            if event.id.instance.is_some()
                && let Some(parent) = self
                    .store
                    .data
                    .events
                    .iter()
                    .find(|row| row.id.uid == event.id.uid && row.id.instance.is_none())
            {
                event = parent.clone();
            }
            self.start_draft(event, EditKind::Series);
        } else if recurring {
            let Some(duration) = event.length() else {
                return;
            };
            event.id.instance = Some(event.id.instance.unwrap_or(selection.start));
            event.rule = None;
            event.added_dates.clear();
            event.exceptions.clear();
            event.start = selection.start.with_timezone(&event.zone).naive_local();
            event.end = (selection.start + duration)
                .with_timezone(&event.zone)
                .naive_local();
            self.start_draft(event, EditKind::Instance);
        } else {
            self.start_draft(event, EditKind::Series);
        }
    }

    fn save_draft(&mut self) -> Action {
        let Some(draft) = &self.draft else {
            return Action::None;
        };
        let event = match draft.validate() {
            Ok(event) => event,
            Err(error) => {
                self.message = Some(error.to_string());
                return Action::Blur;
            }
        };
        let kind = draft.kind;
        let date = if event.all_day {
            event.start.date()
        } else {
            event.start_utc().map_or(event.start.date(), |date| {
                date.with_timezone(&Local).date_naive()
            })
        };
        let result = self.commit(move |data| {
            if !data.calendars.iter().any(|cal| cal.id == event.calendar) {
                return Err(io::Error::other("This calendar was removed"));
            }
            if kind == EditKind::Series && !data.events.iter().any(|row| row.id == event.id) {
                return Err(io::Error::other("This event has already been removed"));
            }
            if kind == EditKind::Instance
                && !data.events.iter().any(|row| row.id.uid == event.id.uid)
            {
                return Err(io::Error::other("This series has already been removed"));
            }
            data.put(event);
            Ok(())
        });
        match result {
            Ok(()) => {
                self.selected_day = date;
                self.draft = None;
                self.detail = None;
                self.message = Some("Event saved".into());
            }
            Err(error) => self.message = Some(error.to_string()),
        }
        Action::Blur
    }

    fn delete(&mut self, selection: Selected, whole: bool) {
        let result = self.commit(move |data| {
            if whole {
                data.remove_series(&selection.id);
            } else {
                data.remove_instance(&selection.id, selection.start);
            }
            Ok(())
        });
        self.message = Some(match result {
            Ok(()) => "Event deleted".into(),
            Err(error) => error.to_string(),
        });
        self.detail = None;
        self.dialog = Dialog::None;
    }

    fn save_calendar(&mut self, old: Option<String>) {
        let name = self.new_name.trim().to_owned();
        if name.is_empty() {
            self.message = Some("Enter a calendar name".into());
            return;
        }
        let color = self.new_color;
        let result = self.commit(move |data| {
            if let Some(key) = old {
                let calendar = data
                    .calendars
                    .iter_mut()
                    .find(|row| row.id == key)
                    .ok_or_else(|| io::Error::other("This calendar was removed"))?;
                calendar.name = name;
                calendar.color = color;
            } else {
                data.calendars.push(Calendar {
                    id: id(),
                    name,
                    color,
                    visible: true,
                });
            }
            Ok(())
        });
        match result {
            Ok(()) => {
                self.dialog = Dialog::Calendars;
                self.message = None;
            }
            Err(error) => self.message = Some(error.to_string()),
        }
    }

    fn begin_file(&mut self, task: FileTask) {
        let request = match &task {
            FileTask::Import => spectre_files::PickerRequest::Open {
                filter: spectre_files::Filter::Extensions(vec!["ics".into(), "ical".into()]),
                multiple: false,
            },
            FileTask::Export(_) => spectre_files::PickerRequest::Save {
                suggested_name: "Calendar.ics".into(),
            },
        };
        self.picker.set_picker(request);
        self.dialog = Dialog::None;
        self.file_task = Some(task);
        self.message = None;
    }

    fn file_selected(&mut self, path: PathBuf) {
        match self.file_task.take() {
            Some(FileTask::Import) => {
                let calendar = self.calendar_for_new();
                let result = ics::read(&path, &calendar).and_then(|imported| {
                    let count = imported.events.len();
                    if count == 0 {
                        return Err(io::Error::other("No supported events in this file"));
                    }
                    let skipped = imported.skipped;
                    self.commit(move |data| {
                        for event in imported.events {
                            // An imported UID updates the previous copy while retaining its calendar.
                            let mut event = event;
                            if let Some(existing) =
                                data.events.iter().find(|row| row.id.uid == event.id.uid)
                            {
                                event.calendar.clone_from(&existing.calendar);
                            }
                            data.put(event);
                        }
                        Ok((count, skipped))
                    })
                });
                self.message = Some(match result {
                    Ok((count, 0)) => format!("Imported {count} event(s)"),
                    Ok((count, skipped)) => {
                        format!("Imported {count}, skipped {skipped} unsupported event(s)")
                    }
                    Err(error) => format!("Import failed: {error}"),
                });
            }
            Some(FileTask::Export(uid)) => {
                let temp = path.with_extension(format!("ics-tmp-{}", std::process::id()));
                let text = ics::encode(
                    self.store
                        .data
                        .events
                        .iter()
                        .filter(|event| uid.as_ref().is_none_or(|uid| event.id.uid == *uid)),
                );
                let result = (|| {
                    use std::io::Write;
                    let mut options = std::fs::OpenOptions::new();
                    options.create_new(true).write(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        options.mode(0o600);
                    }
                    let mut file = options.open(&temp)?;
                    file.write_all(text.as_bytes())?;
                    file.sync_all()?;
                    std::fs::rename(&temp, &path)
                })();
                if result.is_err() {
                    let _ = std::fs::remove_file(temp);
                }
                self.message = Some(match result {
                    Ok(()) => "Calendar file exported".into(),
                    Err(error) => format!("Export failed: {error}"),
                });
            }
            None => {}
        }
    }

    fn occurrences(&mut self, from: NaiveDate, to: NaiveDate, query: String) -> &[Occurrence] {
        if !self
            .cache
            .as_ref()
            .is_some_and(|cache| cache.from == from && cache.to == to && cache.query == query)
        {
            let events = self.store.data.instances(from, to, &query, true);
            self.cache = Some(RangeCache {
                from,
                to,
                query,
                events,
            });
        }
        &self.cache.as_ref().unwrap().events
    }

    pub fn upcoming(&mut self) -> Option<(String, String)> {
        self.refresh();
        let now = model::now_local_naive();
        let day = now.date();
        let upcoming = self
            .store
            .data
            .instances(day, day + Days::new(30), "", true);
        let next = upcoming.iter().find(|item| item.display_end() > now)?;
        let when = if next.all_day {
            next.display_start()
                .format("%a, %b %-d · All day")
                .to_string()
        } else {
            next.display_start()
                .format("%a, %b %-d · %H:%M")
                .to_string()
        };
        Some((self.store.data.events[next.index].title.clone(), when))
    }

    pub fn due_reminders(&mut self) -> Vec<(String, String)> {
        #[cfg(target_arch = "wasm32")]
        {
            return Vec::new();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self
                .last_reminder
                .is_some_and(|stamp| stamp.elapsed().as_secs() < 12)
            {
                return Vec::new();
            }
            self.last_reminder = Some(std::time::Instant::now());
            self.refresh();
            let now = model::now_utc();
            if !self
                .store
                .data
                .events
                .iter()
                .any(|event| !event.reminders.is_empty())
                || self.store.data.clone().due(now).is_empty()
            {
                return Vec::new();
            }
            self.commit(|data| Ok(data.due(now))).unwrap_or_default()
        }
    }
}
