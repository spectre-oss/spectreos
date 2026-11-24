//! Display Backlight & Dimming Service (platform-agnostic).

use spectre_settings::SystemState;

/// Platform-agnostic backend contract for hardware panel backlight control.
pub trait BacklightBackend: Send {
    fn max_raw(&self) -> u32;
    fn read_raw(&self) -> Option<u32>;
    fn set_raw(&mut self, raw: u32, power_off: bool);
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullBacklightBackend;

impl BacklightBackend for NullBacklightBackend {
    fn max_raw(&self) -> u32 {
        4095
    }

    fn read_raw(&self) -> Option<u32> {
        None
    }

    fn set_raw(&mut self, _raw: u32, _power_off: bool) {}
}

/// Domain service owning saved brightness, dimming, panel sleep/wake, and slider mapping.
pub struct DisplayBacklightService {
    backend: Box<dyn BacklightBackend>,
    max_brightness: u32,
    saved_brightness: u32,
    powered_off: bool,
    last_raw: u32,
    last_norm: Option<f32>,
}

impl DisplayBacklightService {
    pub fn new(backend: Box<dyn BacklightBackend>, sys: &mut SystemState) -> Self {
        let max_brightness = backend.max_raw().max(1);
        let cur = backend
            .read_raw()
            .unwrap_or(max_brightness / 3)
            .max(max_brightness / 10);
        sys.brightness = (cur as f32 / max_brightness as f32).clamp(0.05, 1.0);
        Self {
            backend,
            max_brightness,
            saved_brightness: cur,
            powered_off: false,
            last_raw: cur,
            last_norm: Some(sys.brightness),
        }
    }

    fn set_raw(&mut self, value: u32, power_off: bool) {
        let value = value.min(self.max_brightness);
        if value > 0 {
            self.saved_brightness = value;
        }
        self.last_raw = value;
        self.backend.set_raw(value, power_off);
    }

    pub fn set_dimmed(&mut self) {
        if self.powered_off {
            return;
        }
        let dim = (self.saved_brightness / 5).max(16);
        self.last_raw = dim;
        self.backend.set_raw(dim, false);
    }

    pub fn set_off(&mut self) {
        self.powered_off = true;
        self.set_raw(0, true);
    }

    pub fn set_on(&mut self) {
        self.powered_off = false;
        let target = self.saved_brightness.max(self.max_brightness / 10);
        self.set_raw(target, false);
    }

    pub fn apply_normalized(&mut self, norm: f32) {
        let norm = norm.clamp(0.05, 1.0);
        self.last_norm = Some(norm);
        let min_on = (self.max_brightness / 50).max(16);
        let target =
            (min_on as f32 + norm * (self.max_brightness - min_on) as f32).round() as u32;
        if self.powered_off {
            self.saved_brightness = target;
        } else if target.abs_diff(self.last_raw) >= 8 {
            self.set_raw(target, false);
        }
    }

    /// Returns `Some(new_norm)` if the user changed `sys.brightness` on the UI slider.
    pub fn sync_slider_from_state(&mut self, sys: &SystemState, screen_on: bool) -> Option<f32> {
        if !screen_on {
            return None;
        }
        if let Some(prev) = self.last_norm
            && (sys.brightness - prev).abs() >= 0.005
        {
            self.apply_normalized(sys.brightness);
            return Some(sys.brightness);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct RecordingBacklight(Arc<Mutex<Vec<(u32, bool)>>>);

    impl BacklightBackend for RecordingBacklight {
        fn max_raw(&self) -> u32 { 1000 }
        fn read_raw(&self) -> Option<u32> { Some(300) }
        fn set_raw(&mut self, level: u32, off: bool) {
            self.0.lock().unwrap().push((level, off));
        }
    }

    #[test]
    fn brightness_changes_wait_for_explicit_wake() {
        let writes = Arc::new(Mutex::new(Vec::new()));
        let backend = Box::new(RecordingBacklight(writes.clone()));
        let mut display = DisplayBacklightService::new(backend, &mut SystemState::default());
        display.set_off();
        display.apply_normalized(0.5);
        display.set_dimmed();
        assert_eq!(*writes.lock().unwrap(), [(0, true)]);
        display.set_on();
        assert_eq!(*writes.lock().unwrap(), [(0, true), (510, false)]);
    }
}
