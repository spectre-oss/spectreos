//! Battery & External Charger Telemetry Service (platform-agnostic).

use spectre_settings::SystemState;
use std::time::{Duration, Instant};

/// Live battery and external power supply snapshot from a [`BatteryBackend`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BatteryStatus {
    pub level: u8,
    pub charging: bool,
    pub plugged_in: bool,
}

/// Platform-agnostic backend contract for battery and charger telemetry.
pub trait BatteryBackend: Send {
    fn poll(&mut self) -> Option<BatteryStatus>;
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullBatteryBackend;

impl BatteryBackend for NullBatteryBackend {
    fn poll(&mut self) -> Option<BatteryStatus> {
        None
    }
}

/// Domain service polling battery level and external cable presence (runs even while screen is OFF).
pub struct BatteryService {
    backend: Box<dyn BatteryBackend>,
    plugged_in: bool,
    last_poll: Instant,
}

impl BatteryService {
    pub fn new(mut backend: Box<dyn BatteryBackend>, sys: &mut SystemState) -> Self {
        let mut plugged_in = false;
        if let Some(batt) = backend.poll() {
            sys.battery_level = batt.level;
            plugged_in = batt.plugged_in || batt.charging;
            sys.charging = plugged_in;
        }
        Self {
            backend,
            plugged_in,
            last_poll: Instant::now(),
        }
    }

    pub fn is_plugged_in(&self) -> bool {
        self.plugged_in
    }

    /// Polls every 2s so charging LED & status bar stay responsive even while screen is OFF.
    pub fn poll(&mut self, sys: &mut SystemState) -> bool {
        if self.last_poll.elapsed() < Duration::from_secs(2) {
            return false;
        }
        self.last_poll = Instant::now();
        let Some(batt) = self.backend.poll() else {
            return false;
        };
        let plugged = batt.plugged_in || batt.charging;
        self.plugged_in = plugged;
        if sys.battery_level != batt.level || sys.charging != plugged {
            sys.battery_level = batt.level;
            sys.charging = plugged;
            true
        } else {
            false
        }
    }
}
