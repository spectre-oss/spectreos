use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) const COLORS: &[(u8, u8, u8)] = &[
    (43, 124, 100),
    (71, 112, 190),
    (143, 91, 183),
    (178, 73, 103),
    (170, 100, 31),
    (99, 110, 125),
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Calendar {
    pub id: String,
    pub name: String,
    pub color: usize,
    pub visible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct EventId {
    pub uid: String,
    pub instance: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Event {
    pub id: EventId,
    pub calendar: String,
    pub title: String,
    pub location: String,
    pub notes: String,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub zone: Tz,
    pub all_day: bool,
    pub rule: Option<String>,
    pub added_dates: Vec<DateTime<Utc>>,
    pub exceptions: Vec<DateTime<Utc>>,
    pub reminders: Vec<u32>,
}

impl Event {
    pub fn start_utc(&self) -> Option<DateTime<Utc>> {
        self.zone
            .from_local_datetime(&self.start)
            .earliest()
            .map(|date| date.to_utc())
    }

    pub fn end_utc(&self) -> Option<DateTime<Utc>> {
        self.zone
            .from_local_datetime(&self.end)
            .earliest()
            .map(|date| date.to_utc())
    }

    pub fn length(&self) -> Option<Duration> {
        let duration = self.end_utc()? - self.start_utc()?;
        (duration > Duration::zero()).then_some(duration)
    }

    pub fn valid(&self) -> bool {
        self.length().is_some() && !self.title.trim().is_empty()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Occurrence {
    pub index: usize,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub all_day: bool,
}

impl Occurrence {
    pub fn display_start(&self) -> NaiveDateTime {
        if self.all_day {
            self.start.naive_utc()
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.start.with_timezone(&Local).naive_local()
            }
            #[cfg(target_arch = "wasm32")]
            {
                let off = i64::from(WASM_TZ_OFFSET.load(std::sync::atomic::Ordering::Relaxed));
                (self.start + Duration::seconds(off)).naive_utc()
            }
        }
    }
    pub fn display_end(&self) -> NaiveDateTime {
        if self.all_day {
            self.end.naive_utc()
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.end.with_timezone(&Local).naive_local()
            }
            #[cfg(target_arch = "wasm32")]
            {
                let off = i64::from(WASM_TZ_OFFSET.load(std::sync::atomic::Ordering::Relaxed));
                (self.end + Duration::seconds(off)).naive_utc()
            }
        }
    }
    pub fn covers(&self, day: NaiveDate) -> bool {
        let start = day.and_hms_opt(0, 0, 0).expect("midnight");
        let next = start + Duration::days(1);
        self.display_start() < next && self.display_end() > start
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Data {
    pub calendars: Vec<Calendar>,
    pub events: Vec<Event>,
    #[serde(default)]
    notified: BTreeMap<String, i64>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            calendars: vec![Calendar {
                id: "personal".into(),
                name: "Personal".into(),
                color: 0,
                visible: true,
            }],
            events: Vec::new(),
            notified: BTreeMap::new(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
static WASM_CLOCK_UNIX: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(1_795_435_200);
#[cfg(target_arch = "wasm32")]
static WASM_TZ_OFFSET: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

#[cfg(target_arch = "wasm32")]
pub fn set_wasm_clock(unix_secs: i64, tz_offset_secs: i32) {
    WASM_CLOCK_UNIX.store(unix_secs, std::sync::atomic::Ordering::Relaxed);
    WASM_TZ_OFFSET.store(tz_offset_secs, std::sync::atomic::Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn now_utc() -> DateTime<Utc> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Utc::now()
    }
    #[cfg(target_arch = "wasm32")]
    {
        let secs = WASM_CLOCK_UNIX.load(std::sync::atomic::Ordering::Relaxed);
        DateTime::<Utc>::from_timestamp(secs, 0).unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
    }
}

#[inline(always)]
pub(crate) fn now_local_naive() -> NaiveDateTime {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Local::now().naive_local()
    }
    #[cfg(target_arch = "wasm32")]
    {
        let secs = WASM_CLOCK_UNIX.load(std::sync::atomic::Ordering::Relaxed);
        let offset = i64::from(WASM_TZ_OFFSET.load(std::sync::atomic::Ordering::Relaxed));
        DateTime::<Utc>::from_timestamp(secs + offset, 0)
            .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
            .naive_utc()
    }
}

pub(crate) fn local_zone() -> Tz {
    #[cfg(target_arch = "wasm32")]
    {
        chrono_tz::UTC
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        iana_time_zone::get_timezone()
            .ok()
            .and_then(|name| name.parse().ok())
            .unwrap_or(chrono_tz::UTC)
    }
}

pub(crate) fn id() -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let timestamp = now_utc().timestamp_nanos_opt().unwrap_or_default();
    #[cfg(not(target_arch = "wasm32"))]
    let source = std::process::id();
    #[cfg(target_arch = "wasm32")]
    let source = "web";
    format!("{timestamp}-{source}-{count}@calendar.spectreos")
}

impl Data {
    pub fn put(&mut self, event: Event) {
        if let Some(old) = self.events.iter_mut().find(|item| item.id == event.id) {
            *old = event;
        } else {
            self.events.push(event);
        }
    }

    pub fn instances(
        &self,
        from: NaiveDate,
        until: NaiveDate,
        query: &str,
        visible_only: bool,
    ) -> Vec<Occurrence> {
        let below = from.and_hms_opt(0, 0, 0).unwrap();
        let above = until.and_hms_opt(0, 0, 0).unwrap();
        let from_utc = below.and_utc() - Duration::days(2);
        let until_utc = above.and_utc() + Duration::days(2);
        let mut result = Vec::new();
        let mut overrides = BTreeSet::new();
        for event in &self.events {
            if let Some(instance) = event.id.instance {
                overrides.insert((event.id.uid.as_str(), instance));
            }
        }
        for (index, event) in self.events.iter().enumerate() {
            if visible_only
                && !self
                    .calendars
                    .iter()
                    .any(|cal| cal.id == event.calendar && cal.visible)
            {
                continue;
            }
            if !query.is_empty()
                && [&event.title, &event.location, &event.notes]
                    .into_iter()
                    .all(|text| !text.to_lowercase().contains(query))
            {
                continue;
            }
            let (Some(start), Some(length)) = (event.start_utc(), event.length()) else {
                continue;
            };
            if start >= until_utc && event.added_dates.is_empty() {
                continue;
            }
            let mut starts: BTreeSet<_> = [start]
                .into_iter()
                .chain(event.added_dates.iter().copied())
                .collect();
            if let Some(rule) = &event.rule {
                let description = format!(
                    "DTSTART;TZID={}:{}\nRRULE:{rule}",
                    event.zone,
                    event.start.format("%Y%m%dT%H%M%S")
                );
                if let Ok(rule_set) = description.parse::<rrule::RRuleSet>() {
                    let timezone = rule_set.get_dt_start().timezone();
                    let start_limit = (from_utc - length).with_timezone(&timezone);
                    let end_limit = until_utc.with_timezone(&timezone);
                    starts.extend(
                        rule_set
                            .after(start_limit)
                            .before(end_limit)
                            .all(4096)
                            .dates
                            .into_iter()
                            .map(|date| date.to_utc()),
                    );
                }
            }
            for start in starts {
                if event.exceptions.contains(&start)
                    || (event.id.instance.is_none()
                        && overrides.contains(&(event.id.uid.as_str(), start)))
                {
                    continue;
                }
                let Some(end) = start.checked_add_signed(length) else {
                    continue;
                };
                let occurrence = Occurrence {
                    index,
                    start,
                    end,
                    all_day: event.all_day,
                };
                if occurrence.display_start() < above && occurrence.display_end() > below {
                    result.push(occurrence);
                }
            }
        }
        result.sort_by_key(|item| {
            (
                item.display_start().date(),
                !item.all_day,
                item.display_start(),
                item.index,
            )
        });
        result
    }

    pub fn remove_series(&mut self, id: &EventId) {
        self.events.retain(|event| event.id.uid != id.uid);
    }

    pub fn remove_instance(&mut self, id: &EventId, start: DateTime<Utc>) {
        let instance = id.instance.unwrap_or(start);
        self.events
            .retain(|event| event.id.uid != id.uid || event.id.instance != Some(instance));
        if let Some(series) = self
            .events
            .iter_mut()
            .find(|event| event.id.uid == id.uid && event.id.instance.is_none())
            && !series.exceptions.contains(&instance)
        {
            series.exceptions.push(instance);
        }
    }

    pub fn due(&mut self, now: DateTime<Utc>) -> Vec<(String, String)> {
        self.notified
            .retain(|_, stamp| *stamp > (now - Duration::days(7)).timestamp());
        let day = now.with_timezone(&Local).date_naive();
        let mut due = Vec::new();
        for item in self.instances(day - Duration::days(1), day + Duration::days(32), "", false) {
            let event = &self.events[item.index];
            let start = if item.all_day {
                Local
                    .from_local_datetime(&item.start.naive_utc())
                    .earliest()
                    .map(|dt| dt.to_utc())
                    .unwrap_or(item.start)
            } else {
                item.start
            };
            for minutes in event
                .reminders
                .iter()
                .copied()
                .filter(|minutes| *minutes <= 43_200)
            {
                let trigger = start - Duration::minutes(i64::from(minutes));
                if trigger > now || now - trigger > Duration::minutes(5) {
                    continue;
                }
                let key = format!("{}:{}:{minutes}", event.id.uid, item.start.timestamp());
                if self.notified.insert(key, now.timestamp()).is_none() {
                    let when = if item.all_day {
                        item.display_start().format("%A, %b %-d").to_string()
                    } else {
                        item.display_start()
                            .format("%A, %b %-d at %H:%M")
                            .to_string()
                    };
                    let body = if event.location.is_empty() {
                        when
                    } else {
                        format!("{when} · {}", event.location)
                    };
                    due.push((event.title.clone(), body));
                }
            }
        }
        due
    }
}

pub(crate) struct Store {
    path: Option<PathBuf>,
    modified: Option<std::time::SystemTime>,
    pub data: Data,
    pub warning: Option<String>,
}

impl Default for Store {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let path = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|path| path.join(".local/share"))
            })
            .map(|path| path.join("spectre-calendar/calendar.json"));
        #[cfg(target_arch = "wasm32")]
        let path = None;
        Self::open(path)
    }
}

impl Store {
    pub fn open(path: Option<PathBuf>) -> Self {
        let mut store = Self {
            path,
            modified: None,
            data: Data::default(),
            warning: None,
        };
        if let Some(path) = &store.path {
            match read(path) {
                Ok(data) => store.data = data,
                Err(error) => {
                    store.warning = Some(format!("Could not load calendar data: {error}"))
                }
            }
            store.modified = fs::metadata(path).and_then(|file| file.modified()).ok();
        }
        store
    }

    pub fn refresh(&mut self) -> bool {
        let Some(path) = &self.path else { return false };
        let modified = fs::metadata(path).and_then(|file| file.modified()).ok();
        if modified == self.modified {
            return false;
        }
        self.modified = modified;
        match read(path) {
            Ok(data) => {
                self.data = data;
                self.warning = None;
                true
            }
            Err(error) => {
                self.warning = Some(format!("Could not load calendar data: {error}"));
                false
            }
        }
    }

    pub fn update<T>(&mut self, edit: impl FnOnce(&mut Data) -> io::Result<T>) -> io::Result<T> {
        let Some(path) = &self.path else {
            return edit(&mut self.data);
        };
        let dir = path.parent().expect("calendar has a data directory");
        fs::create_dir_all(dir)?;
        #[cfg(unix)]
        let _lock = {
            use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
            let lock = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(false)
                .mode(0o600)
                .open(dir.join("calendar.lock"))?;
            // The owned descriptor holds this advisory lock until the transaction finishes.
            if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
                return Err(io::Error::last_os_error());
            }
            lock
        };
        let mut fresh = read(path)?;
        let result = edit(&mut fresh)?;
        let temp = dir.join(format!(".calendar-{}.json", std::process::id()));
        let write = (|| {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temp)?;
            serde_json::to_writer(&mut file, &fresh).map_err(io::Error::other)?;
            file.sync_all()?;
            fs::rename(&temp, path)
        })();
        if let Err(error) = write {
            let _ = fs::remove_file(temp);
            return Err(error);
        }
        self.data = fresh;
        self.modified = fs::metadata(path).and_then(|file| file.modified()).ok();
        self.warning = None;
        Ok(result)
    }
}

fn read(path: &Path) -> io::Result<Data> {
    match fs::File::open(path) {
        Ok(file) => serde_json::from_reader(io::BufReader::new(file)).map_err(io::Error::other),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Data::default()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(year: i32, month: u32, day: u32, hour: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap()
    }

    fn event(start: NaiveDateTime, tz: Tz) -> Event {
        Event {
            id: EventId {
                uid: id(),
                instance: None,
            },
            calendar: "personal".into(),
            title: "Design review".into(),
            location: "Studio".into(),
            notes: String::new(),
            start,
            end: start + Duration::hours(1),
            zone: tz,
            all_day: false,
            rule: None,
            added_dates: vec![],
            exceptions: vec![],
            reminders: vec![15],
        }
    }

    #[test]
    fn recurring_events_preserve_wall_time_and_let_an_occurrence_be_changed() {
        let zone = chrono_tz::Europe::Vienna;
        let mut data = Data::default();
        let mut source = event(at(2026, 10, 18, 9), zone);
        source.rule = Some("FREQ=WEEKLY;COUNT=3".into());
        data.put(source.clone());
        let from = source.start.date();
        let to = at(2026, 11, 4, 0).date();
        let items = data.instances(from, to, "", true);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].start.naive_utc(), at(2026, 10, 18, 7));
        assert_eq!(items[1].start.naive_utc(), at(2026, 10, 25, 8));
        let mut moved = source.clone();
        moved.id.instance = Some(items[1].start);
        moved.start = at(2026, 10, 26, 14);
        moved.end = at(2026, 10, 26, 15);
        moved.rule = None;
        data.put(moved.clone());
        let items = data.instances(from, to, "", true);
        assert_eq!(items.len(), 3);
        assert_eq!(items[1].start.naive_utc(), at(2026, 10, 26, 13));
        data.remove_instance(&moved.id, items[1].start);
        assert_eq!(data.instances(from, to, "", true).len(), 2);
    }

    #[test]
    fn storage_serializes_updates_and_persists_reminder_deduplication() {
        let dir = std::env::temp_dir().join(id());
        let path = dir.join("calendar.json");
        let mut first = Store::open(Some(path.clone()));
        let mut second = Store::open(Some(path.clone()));
        let now = at(2026, 11, 24, 18).and_utc();
        first
            .update(|data| {
                data.put(event(
                    (now + Duration::minutes(15)).naive_utc(),
                    chrono_tz::UTC,
                ));
                Ok(())
            })
            .unwrap();
        second
            .update(|data| {
                let new = event((now + Duration::days(1)).naive_utc(), chrono_tz::UTC);
                data.put(new);
                Ok(())
            })
            .unwrap();
        assert!(first.refresh());
        assert_eq!(first.data.events.len(), 2);
        assert_eq!(first.update(|data| Ok(data.due(now))).unwrap().len(), 1);
        assert!(Store::open(Some(path)).data.due(now).is_empty());
        fs::remove_dir_all(dir).unwrap();
    }
}
