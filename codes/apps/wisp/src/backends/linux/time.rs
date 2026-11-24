//! CLOCK_REALTIME and the system's local time zone, including administrator/NTP adjustments.
use crate::services::time::{ClockReading, TimeBackend};
use chrono::{DateTime, Local};
use std::time::SystemTime;

pub struct LinuxTimeBackend;

impl TimeBackend for LinuxTimeBackend {
    fn now(&self) -> ClockReading {
        let local: DateTime<Local> = SystemTime::now().into();
        ClockReading(local.fixed_offset())
    }
}
