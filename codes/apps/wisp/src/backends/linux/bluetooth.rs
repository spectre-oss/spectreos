//! BlueZ Bluetooth operations run on a worker so connecting cannot block the compositor.

use super::{find_in_path, run_cmd_capture};
use crate::services::bluetooth::{BluetoothBackend, BluetoothStatus};
use spectre_settings::state::BluetoothDevice;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

enum BtCmd {
    SetPowered(bool),
    Connect(String, bool),
}

pub struct BluezBluetoothBackend {
    tx: Option<mpsc::Sender<BtCmd>>,
    latest: Arc<Mutex<Option<BluetoothStatus>>>,
}

impl BluezBluetoothBackend {
    pub fn start() -> Self {
        let latest = Arc::new(Mutex::new(None));
        let Some(btctl) = find_in_path("bluetoothctl") else {
            return Self { tx: None, latest };
        };
        let (tx, rx) = mpsc::channel();
        let status = Arc::clone(&latest);
        std::thread::spawn(move || {
            let mut error = None;
            loop {
                let mut snapshot = query_bluetooth(&btctl);
                snapshot.error.clone_from(&error);
                *status.lock().unwrap() = Some(snapshot);
                let (output, message) = match rx.recv_timeout(Duration::from_secs(5)) {
                    Ok(BtCmd::SetPowered(on)) => {
                        if on && let Some(rfkill) = find_in_path("rfkill") {
                            let _ = run_cmd_capture(&rfkill, &["unblock", "bluetooth"]);
                        }
                        (
                            run_cmd_capture(&btctl, &["power", if on { "on" } else { "off" }]),
                            "Could not change Bluetooth",
                        )
                    }
                    Ok(BtCmd::Connect(address, on)) => (
                        run_cmd_capture(
                            &btctl,
                            &[
                                "--timeout",
                                "18",
                                if on { "connect" } else { "disconnect" },
                                &address,
                            ],
                        ),
                        if on {
                            "Could not connect to this device"
                        } else {
                            "Could not disconnect this device"
                        },
                    ),
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };
                error = (!output.is_some_and(|text| {
                    !text.contains("Failed") && !text.contains("No default controller")
                }))
                .then(|| message.to_owned());
            }
        });
        Self {
            tx: Some(tx),
            latest,
        }
    }

    fn send(&self, command: BtCmd) {
        if let Some(tx) = &self.tx {
            if let Ok(mut latest) = self.latest.lock() {
                *latest = None;
            }
            let _ = tx.send(command);
        }
    }
}

fn devices(output: &str) -> impl Iterator<Item = (&str, &str)> {
    output
        .lines()
        .filter_map(|line| line.strip_prefix("Device ")?.split_once(' '))
        .filter(|(address, name)| address.len() == 17 && !name.trim().is_empty())
}

fn query_bluetooth(btctl: &Path) -> BluetoothStatus {
    let Some(controller) =
        run_cmd_capture(btctl, &["show"]).filter(|out| out.contains("Controller "))
    else {
        return BluetoothStatus::default();
    };
    let powered = controller.lines().any(|line| line.trim() == "Powered: yes");
    let paired = run_cmd_capture(btctl, &["devices", "Paired"]).unwrap_or_default();
    let connected = if powered {
        run_cmd_capture(btctl, &["devices", "Connected"]).unwrap_or_default()
    } else {
        String::new()
    };
    let mut entries: BTreeMap<_, _> = devices(&paired)
        .map(|(address, name)| (address, (name, false)))
        .collect();
    entries.extend(devices(&connected).map(|(address, name)| (address, (name, true))));
    let mut devices: Vec<_> = entries
        .into_iter()
        .map(|(address, (name, connected))| BluetoothDevice {
            address: address.into(),
            name: name.trim().into(),
            connected,
        })
        .collect();
    devices.sort_by(|a, b| {
        (std::cmp::Reverse(a.connected), &a.name).cmp(&(std::cmp::Reverse(b.connected), &b.name))
    });
    BluetoothStatus {
        available: true,
        powered,
        connected_device: devices
            .iter()
            .find(|device| device.connected)
            .map(|device| device.name.clone()),
        connected_count: devices.iter().filter(|device| device.connected).count(),
        devices,
        error: None,
    }
}

impl BluetoothBackend for BluezBluetoothBackend {
    fn poll(&mut self) -> Option<BluetoothStatus> {
        self.latest.lock().ok()?.take()
    }
    fn set_powered(&mut self, on: bool) {
        self.send(BtCmd::SetPowered(on));
    }
    fn connect(&mut self, address: String, enabled: bool) {
        self.send(BtCmd::Connect(address, enabled));
    }
}
