//! Bluetooth Radio & Connected Devices Service (platform-agnostic).

use spectre_settings::{SystemState, state::BluetoothDevice};

/// Live Bluetooth adapter and connected device snapshot from a [`BluetoothBackend`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BluetoothStatus {
    pub available: bool,
    pub powered: bool,
    pub connected_device: Option<String>,
    pub connected_count: usize,
    pub devices: Vec<BluetoothDevice>,
    pub error: Option<String>,
}

/// Platform-agnostic backend contract for Bluetooth control (BlueZ, Fluoride/Android, mock).
pub trait BluetoothBackend: Send {
    fn poll(&mut self) -> Option<BluetoothStatus>;
    fn set_powered(&mut self, on: bool);
    fn connect(&mut self, address: String, enabled: bool) {
        let _ = (address, enabled);
    }
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullBluetoothBackend;

impl BluetoothBackend for NullBluetoothBackend {
    fn poll(&mut self) -> Option<BluetoothStatus> {
        None
    }

    fn set_powered(&mut self, _on: bool) {}
}

/// Domain service syncing Bluetooth radio state and connected device name with UI state.
pub struct BluetoothService {
    backend: Box<dyn BluetoothBackend>,
    last_enabled: Option<bool>,
}

impl BluetoothService {
    pub fn new(mut backend: Box<dyn BluetoothBackend>, sys: &mut SystemState) -> Self {
        if let Some(bt) = backend.poll() {
            apply_bluetooth_status(sys, &bt);
        }
        Self {
            backend,
            last_enabled: Some(sys.bluetooth_enabled),
        }
    }

    pub fn poll(&mut self, sys: &mut SystemState) -> bool {
        let Some(bt) = self.backend.poll() else {
            return false;
        };
        let prev_en = sys.bluetooth_enabled;
        let prev_dev = sys.bluetooth_device.clone();
        let prev_av = sys.bluetooth_available;
        let discovery_changed =
            sys.bluetooth_devices != bt.devices || sys.bluetooth_error != bt.error;
        apply_bluetooth_status(sys, &bt);
        self.last_enabled = Some(sys.bluetooth_enabled);
        discovery_changed
            || sys.bluetooth_enabled != prev_en
            || sys.bluetooth_device != prev_dev
            || sys.bluetooth_available != prev_av
    }

    pub fn connect(&mut self, address: String, enabled: bool) {
        self.backend.connect(address, enabled);
    }

    pub fn set_powered(&mut self, on: bool) {
        self.last_enabled = Some(on);
        self.backend.set_powered(on);
    }

    pub fn sync_from_state(&mut self, sys: &SystemState) {
        if self.last_enabled != Some(sys.bluetooth_enabled) {
            self.set_powered(sys.bluetooth_enabled);
        }
    }
}

fn apply_bluetooth_status(sys: &mut SystemState, bt: &BluetoothStatus) {
    sys.bluetooth_available = bt.available;
    sys.bluetooth_enabled = bt.available && bt.powered;
    sys.bluetooth_device.clone_from(&bt.connected_device);
    sys.bluetooth_devices.clone_from(&bt.devices);
    sys.bluetooth_error.clone_from(&bt.error);
}
