//! Civil time for clocks and event timestamps. Animation and timeout deadlines use monotonic time.
use chrono::{DateTime, FixedOffset, Timelike};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct ClockReading(pub DateTime<FixedOffset>);

impl ClockReading {
    pub fn hm(self) -> String {
        self.0.format("%H:%M").to_string()
    }

    pub fn date(self) -> String {
        self.0.format("%a, %b %-d").to_string()
    }

    pub fn until_second(self) -> f64 {
        1.0 - f64::from(self.0.nanosecond()) / 1_000_000_000.0
    }

    pub fn until_minute(self) -> f64 {
        f64::from(59 - self.0.second()) + self.until_second()
    }
}

pub trait TimeBackend: Send + Sync {
    fn now(&self) -> ClockReading;
}

impl TimeBackend for ClockReading {
    fn now(&self) -> ClockReading {
        *self
    }
}

#[derive(Clone)]
pub struct TimeService(pub Arc<dyn TimeBackend>);

impl TimeService {
    pub fn now(&self) -> ClockReading {
        self.0.now()
    }
}

pub struct HostTimeBackend;
impl TimeBackend for HostTimeBackend {
    fn now(&self) -> ClockReading {
        #[cfg(not(target_arch = "wasm32"))]
        {
            ClockReading(chrono::Local::now().fixed_offset())
        }
        #[cfg(target_arch = "wasm32")]
        {
            PREVIEW_TIME.get()
        }
    }
}

impl Default for TimeService {
    fn default() -> Self {
        Self(Arc::new(HostTimeBackend))
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static PREVIEW_TIME: std::cell::Cell<ClockReading> =
        std::cell::Cell::new(ClockReading(chrono::DateTime::UNIX_EPOCH.fixed_offset()));
}

/// The standalone WebAssembly preview gets civil time from its browser host.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn wisp_wall_time(unix_millis: f64, offset_seconds: i32) {
    if let (Some(utc), Some(offset)) = (
        DateTime::from_timestamp_millis(unix_millis as i64),
        FixedOffset::east_opt(offset_seconds),
    ) {
        PREVIEW_TIME.set(ClockReading(utc.with_timezone(&offset)));
        spectre_calendar::set_wasm_clock(utc.timestamp(), offset_seconds);
    }
}
