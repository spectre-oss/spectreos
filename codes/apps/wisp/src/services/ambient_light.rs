//! Ambient Light Sensor & Adaptive Brightness Service (platform-agnostic).
//!
//! Converts raw lux measurements from any [`AmbientLightBackend`] through a perceptual
//! piecewise-log brightness spline with low-pass smoothing, hysteresis, and user slider bias.

use spectre_settings::SystemState;
use std::time::{Duration, Instant};

/// Platform-agnostic backend contract for reading ambient illuminance in lux.
pub trait AmbientLightBackend: Send {
    fn is_available(&self) -> bool;
    /// Start or pause live sampling (paused while display is off or Adaptive Brightness is off).
    fn set_active(&mut self, active: bool);
    /// Non-blocking read of the newest lux measurement, if available.
    fn poll_lux(&mut self) -> Option<f32>;
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullAmbientLightBackend;

impl AmbientLightBackend for NullAmbientLightBackend {
    fn is_available(&self) -> bool {
        false
    }

    fn set_active(&mut self, _active: bool) {}

    fn poll_lux(&mut self) -> Option<f32> {
        None
    }
}

/// Android-style perceptual piecewise-log lux -> normalized brightness (`0.08..=1.00`) curve.
pub fn lux_to_brightness(lux: f32) -> f32 {
    if !lux.is_finite() || lux <= 0.0 {
        return 0.08;
    }
    const POINTS: &[(f32, f32)] = &[
        (0.0, 0.08),
        (2.0, 0.14),
        (10.0, 0.22),
        (40.0, 0.36),
        (120.0, 0.52),
        (350.0, 0.68),
        (1000.0, 0.82),
        (3500.0, 0.94),
        (10000.0, 1.00),
    ];
    if lux >= POINTS[POINTS.len() - 1].0 {
        return 1.0;
    }
    for win in POINTS.windows(2) {
        let (x0, y0) = win[0];
        let (x1, y1) = win[1];
        if lux <= x1 {
            let l0 = (x0 + 1.0).ln();
            let l1 = (x1 + 1.0).ln();
            let lx = (lux + 1.0).ln();
            let t = ((lx - l0) / (l1 - l0).max(1e-5)).clamp(0.0, 1.0);
            return (y0 + (y1 - y0) * t).clamp(0.08, 1.0);
        }
    }
    1.0
}

/// Output of one adaptive-brightness poll step.
#[derive(Clone, Copy, Debug, Default)]
pub struct AmbientLightUpdate {
    /// Set when ambient lux shifted noticeably enough to emit a telemetry event.
    pub lux_event: Option<f32>,
    /// Set when adaptive brightness computed a new smoothed normalized display brightness.
    pub new_brightness: Option<f32>,
}

/// Domain service running exponential lux filtering, hysteresis, and user bias adjustment.
pub struct AmbientLightService {
    backend: Box<dyn AmbientLightBackend>,
    filtered_lux: Option<f32>,
    user_bias: f32,
    last_apply: Instant,
    last_auto_enabled: bool,
    last_active: bool,
}

impl AmbientLightService {
    pub fn new(backend: Box<dyn AmbientLightBackend>, auto_enabled: bool) -> Self {
        Self {
            backend,
            filtered_lux: None,
            user_bias: 0.0,
            last_apply: Instant::now() - Duration::from_secs(5),
            last_auto_enabled: auto_enabled,
            last_active: false,
        }
    }

    pub fn is_available(&self) -> bool {
        self.backend.is_available()
    }

    /// Record a manual brightness slider offset while Adaptive Brightness is active.
    pub fn record_manual_slider(&mut self, manual_norm: f32, auto_enabled: bool) {
        if auto_enabled && let Some(lux) = self.filtered_lux {
            let base = lux_to_brightness(lux);
            self.user_bias = (manual_norm - base).clamp(-0.35, 0.35);
        }
    }

    /// Handle Adaptive Brightness toggle changes; returns immediate target brightness if re-enabled.
    pub fn sync_toggle(&mut self, auto_enabled: bool) -> Option<f32> {
        if auto_enabled == self.last_auto_enabled {
            return None;
        }
        self.last_auto_enabled = auto_enabled;
        if auto_enabled {
            self.user_bias = 0.0;
            return self.filtered_lux.map(lux_to_brightness);
        }
        None
    }

    /// Poll ambient light when screen is on and Adaptive Brightness is enabled.
    pub fn poll(&mut self, sys: &mut SystemState, screen_on: bool) -> AmbientLightUpdate {
        let want_active = screen_on && sys.auto_brightness && self.backend.is_available();
        if want_active != self.last_active {
            self.last_active = want_active;
            self.backend.set_active(want_active);
        }
        if !want_active {
            return AmbientLightUpdate::default();
        }
        let Some(lux) = self.backend.poll_lux() else {
            return AmbientLightUpdate::default();
        };
        let smoothed = match self.filtered_lux {
            Some(prev) => prev * 0.72 + lux * 0.28,
            None => lux,
        };
        self.filtered_lux = Some(smoothed);
        let prev_lux = sys.ambient_lux.unwrap_or(-100.0);
        sys.ambient_lux = Some(smoothed);

        let lux_event = ((smoothed - prev_lux).abs() >= 4.0).then_some(smoothed);
        let target = (lux_to_brightness(smoothed) + self.user_bias).clamp(0.08, 1.0);
        let delta = (target - sys.brightness).abs();
        let new_brightness = if delta >= 0.035 && self.last_apply.elapsed() >= Duration::from_millis(350)
        {
            self.last_apply = Instant::now();
            let next = (sys.brightness + (target - sys.brightness) * 0.45).clamp(0.08, 1.0);
            sys.brightness = next;
            Some(next)
        } else {
            None
        };

        AmbientLightUpdate {
            lux_event,
            new_brightness,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lux_to_brightness_is_monotonic_and_clamped() {
        let samples = [0.0, 2.0, 10.0, 50.0, 200.0, 424.0, 1000.0, 5000.0, 25000.0];
        let mut prev = 0.0_f32;
        for lux in samples {
            let b = lux_to_brightness(lux);
            assert!((0.08..=1.0).contains(&b), "out of bounds for lux={lux}: {b}");
            assert!(b >= prev, "non-monotonic at lux={lux}: {b} < {prev}");
            prev = b;
        }
        assert!((lux_to_brightness(0.0) - 0.08).abs() < 1e-4);
        assert!((lux_to_brightness(25000.0) - 1.0).abs() < 1e-4);
    }
}
