//! Rear Camera Flashlight / Torch Service (platform-agnostic).
//!
//! Strictly controls dedicated camera flash/torch hardware — never hijacks
//! chassis notification or charging LEDs (see [`super::notification_led`]).

use spectre_settings::SystemState;

/// Live camera flashlight / torch state reported by a [`FlashlightBackend`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlashlightStatus {
    pub available: bool,
    pub on: bool,
    pub backend_label: String,
}

/// Platform-agnostic backend contract for a camera torch / flash driver.
pub trait FlashlightBackend: Send {
    fn status(&self) -> FlashlightStatus;
    fn set_enabled(&mut self, on: bool) -> bool;
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullFlashlightBackend;

impl FlashlightBackend for NullFlashlightBackend {
    fn status(&self) -> FlashlightStatus {
        FlashlightStatus {
            available: false,
            on: false,
            backend_label: "none".into(),
        }
    }

    fn set_enabled(&mut self, _on: bool) -> bool {
        false
    }
}

/// Domain service owning flashlight availability and toggle synchronization.
pub struct FlashlightService {
    backend: Box<dyn FlashlightBackend>,
    last_on: bool,
}

impl FlashlightService {
    pub fn new(backend: Box<dyn FlashlightBackend>, sys: &mut SystemState) -> Self {
        let st = backend.status();
        sys.flashlight_available = st.available;
        sys.flashlight_on = st.on;
        Self {
            backend,
            last_on: st.on,
        }
    }

    pub fn status(&self) -> FlashlightStatus {
        self.backend.status()
    }

    pub fn is_available(&self) -> bool {
        self.backend.status().available
    }

    pub fn backend_label(&self) -> String {
        self.backend.status().backend_label
    }

    pub fn set_enabled(&mut self, on: bool, sys: &mut SystemState) -> bool {
        let actual = self.backend.set_enabled(on);
        self.last_on = actual;
        sys.flashlight_on = actual;
        actual
    }

    pub fn sync_from_state(&mut self, sys: &mut SystemState) {
        if sys.flashlight_on != self.last_on {
            self.set_enabled(sys.flashlight_on, sys);
        }
    }
}
