//! Tactile Haptic Feedback Service (platform-agnostic).

use spectre_settings::SystemState;

/// Tactile haptic feedback patterns emitted by launcher & lock-screen interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HapticEffect {
    /// Light, crisp 14ms tap (keypad digits, brightness ticks, subtle touch).
    Tick,
    /// Medium 22ms click (Quick Settings tiles, flashlight toggle, buttons).
    Click,
    /// Firm 36ms pulse (long-press actions, drag pickup).
    LongPress,
    /// Celebratory double-tap (Touch ID / PIN unlock success).
    Confirm,
    /// Distinct double-buzz (Touch ID mismatch, wrong PIN/password).
    Reject,
}

/// Platform-agnostic backend contract for haptic actuators (`evdev` FF, vibrator LED, HAL).
pub trait HapticsBackend: Send {
    fn is_available(&self) -> bool;
    fn play(&mut self, effect: HapticEffect);
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullHapticsBackend;

impl HapticsBackend for NullHapticsBackend {
    fn is_available(&self) -> bool {
        false
    }

    fn play(&mut self, _effect: HapticEffect) {}
}

/// Domain service gating haptic playback on user preferences (`sys.haptic_enabled`).
pub struct HapticsService {
    backend: Box<dyn HapticsBackend>,
}

impl HapticsService {
    pub fn new(backend: Box<dyn HapticsBackend>) -> Self {
        Self { backend }
    }

    pub fn is_available(&self) -> bool {
        self.backend.is_available()
    }

    pub fn play(&mut self, effect: HapticEffect, sys: &SystemState) {
        if sys.haptic_enabled && self.backend.is_available() {
            self.backend.play(effect);
        }
    }
}
