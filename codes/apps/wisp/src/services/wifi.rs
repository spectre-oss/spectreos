//! Wi-Fi & Airplane Radio Service (platform-agnostic).

use spectre_settings::{SystemState, state::WifiNetwork};

/// Live Wi-Fi radio and connection snapshot from a [`WifiBackend`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WifiStatus {
    pub available: bool,
    pub enabled: bool,
    pub connected: bool,
    pub ssid: Option<String>,
    /// Signal strength `0..=100` when connected.
    pub signal: u8,
    pub networks: Vec<WifiNetwork>,
    pub error: Option<String>,
}

/// Platform-agnostic backend contract for Wi-Fi radio management (NetworkManager, iwd, ConnMan, HAL).
pub trait WifiBackend: Send {
    fn poll(&mut self) -> Option<WifiStatus>;
    fn set_enabled(&mut self, on: bool);
    fn set_airplane(&mut self, airplane: bool);
    fn join(&mut self, network: WifiNetwork) {
        let _ = network;
    }
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullWifiBackend;

impl WifiBackend for NullWifiBackend {
    fn poll(&mut self) -> Option<WifiStatus> {
        None
    }

    fn set_enabled(&mut self, _on: bool) {}
    fn set_airplane(&mut self, _airplane: bool) {}
}

/// Domain service syncing Wi-Fi and Airplane Mode state between UI and backend.
pub struct WifiService {
    backend: Box<dyn WifiBackend>,
    last_enabled: Option<bool>,
    last_airplane: Option<bool>,
}

impl WifiService {
    pub fn new(mut backend: Box<dyn WifiBackend>, sys: &mut SystemState) -> Self {
        if let Some(wifi) = backend.poll() {
            apply_wifi_status(sys, &wifi);
        }
        Self {
            backend,
            last_enabled: Some(sys.wifi_enabled),
            last_airplane: Some(sys.airplane_mode),
        }
    }

    pub fn poll(&mut self, sys: &mut SystemState) -> bool {
        let Some(wifi) = self.backend.poll() else {
            return false;
        };
        let prev_connected = sys.wifi_connected;
        let prev_name = sys.wifi_name.clone();
        let prev_sig = sys.wifi_signal;
        let prev_en = sys.wifi_enabled;
        let discovery_changed = sys.wifi_available != wifi.available
            || sys.wifi_networks != wifi.networks
            || sys.wifi_error != wifi.error;
        apply_wifi_status(sys, &wifi);
        self.last_enabled = Some(sys.wifi_enabled);
        discovery_changed
            || sys.wifi_connected != prev_connected
            || sys.wifi_name != prev_name
            || sys.wifi_signal != prev_sig
            || sys.wifi_enabled != prev_en
    }

    pub fn join(&mut self, network: WifiNetwork) {
        self.backend.join(network);
    }

    pub fn sync_from_state(&mut self, sys: &SystemState) {
        if self.last_enabled != Some(sys.wifi_enabled) {
            self.last_enabled = Some(sys.wifi_enabled);
            self.backend.set_enabled(sys.wifi_enabled);
        }
        if self.last_airplane != Some(sys.airplane_mode) {
            self.last_airplane = Some(sys.airplane_mode);
            self.backend.set_airplane(sys.airplane_mode);
        }
    }
}

fn apply_wifi_status(sys: &mut SystemState, wifi: &WifiStatus) {
    sys.wifi_available = wifi.available;
    sys.wifi_networks.clone_from(&wifi.networks);
    sys.wifi_error.clone_from(&wifi.error);
    sys.wifi_enabled = wifi.enabled;
    sys.wifi_connected = wifi.connected;
    sys.wifi_signal = wifi.signal;
    if let Some(ref ssid) = wifi.ssid
        && !ssid.is_empty()
    {
        sys.wifi_name = ssid.clone();
    } else if !wifi.connected {
        sys.wifi_name = if wifi.enabled {
            "Wi-Fi".into()
        } else {
            "Internet".into()
        };
    }
}
