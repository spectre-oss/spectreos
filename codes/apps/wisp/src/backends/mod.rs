//! Swappable Device / OS Hardware Backends for Wisp / SpectreOS.
//!
//! Each target platform lives in its own module (currently [`linux`] for standard Linux
//! mobile phones & Poco F1; future ports such as `android_hal`, `pinephone`, `pixel`, or
//! vendor daemons plug in here by returning a [`crate::services::DeviceBackends`]).

#[cfg(feature = "compositor")]
pub mod linux;

use crate::services::DeviceBackends;

/// Discover the active platform backend set (honors `WISP_HARDWARE_PROFILE=null` override).
#[cfg(feature = "compositor")]
pub fn discover() -> DeviceBackends {
    if std::env::var("WISP_HARDWARE_PROFILE")
        .map(|v| v.eq_ignore_ascii_case("null"))
        .unwrap_or(false)
    {
        return DeviceBackends::null();
    }
    linux::discover()
}

/// Non-compositor targets (desktop preview, WASM, unit tests) always use null backends.
#[cfg(not(feature = "compositor"))]
pub fn discover() -> DeviceBackends {
    DeviceBackends::null()
}
