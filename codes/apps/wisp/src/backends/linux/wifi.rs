//! NetworkManager Wi-Fi operations run on a worker so association cannot block the compositor.

use super::{find_in_path, run_cmd_capture};
use crate::services::wifi::{WifiBackend, WifiStatus};
use spectre_settings::state::WifiNetwork;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

enum WifiCmd {
    SetEnabled(bool),
    SetAirplane(bool),
    Join(WifiNetwork),
}

pub struct NmcliWifiBackend {
    tx: Option<mpsc::Sender<WifiCmd>>,
    latest: Arc<Mutex<Option<WifiStatus>>>,
}

impl NmcliWifiBackend {
    pub fn start() -> Self {
        let latest = Arc::new(Mutex::new(None));
        let Some(nmcli) = find_in_path("nmcli") else {
            return Self { tx: None, latest };
        };
        let (tx, rx) = mpsc::channel();
        let status = Arc::clone(&latest);
        std::thread::spawn(move || {
            let mut error = None;
            loop {
                let mut snapshot = query_nmcli(&nmcli);
                snapshot.error.clone_from(&error);
                *status.lock().unwrap() = Some(snapshot);
                let (args, message): (Vec<String>, _) =
                    match rx.recv_timeout(Duration::from_secs(4)) {
                        Ok(WifiCmd::SetEnabled(on)) => (
                            vec![
                                "radio".into(),
                                "wifi".into(),
                                if on { "on" } else { "off" }.into(),
                            ],
                            "Could not change Wi-Fi",
                        ),
                        Ok(WifiCmd::SetAirplane(airplane)) => (
                            vec![
                                "radio".into(),
                                "wwan".into(),
                                if airplane { "off" } else { "on" }.into(),
                            ],
                            "Could not change cellular radio",
                        ),
                        Ok(WifiCmd::Join(network)) => {
                            let mut args = vec!["--wait".into(), "30".into()];
                            let mut join = match network.profile {
                                Some(uuid) => {
                                    vec!["connection".into(), "up".into(), "uuid".into(), uuid]
                                }
                                None => vec![
                                    "device".into(),
                                    "wifi".into(),
                                    "connect".into(),
                                    network.ssid,
                                ],
                            };
                            args.append(&mut join);
                            (args, "Could not connect to this network")
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                let args: Vec<_> = args.iter().map(String::as_str).collect();
                error = run_cmd_capture(&nmcli, &args)
                    .is_none()
                    .then(|| message.to_owned());
            }
        });
        Self {
            tx: Some(tx),
            latest,
        }
    }

    fn send(&self, command: WifiCmd) {
        if let Some(tx) = &self.tx {
            if let Ok(mut latest) = self.latest.lock() {
                *latest = None;
            }
            let _ = tx.send(command);
        }
    }
}

// nmcli terse output escapes both literal colons and backslashes.
fn fields(line: &str) -> Vec<String> {
    let mut result = vec![String::new()];
    let mut escaped = false;
    for ch in line.chars() {
        if !escaped && ch == ':' {
            result.push(String::new());
        } else if !escaped && ch == '\\' {
            escaped = true;
        } else {
            result.last_mut().unwrap().push(ch);
            escaped = false;
        }
    }
    result
}

fn networks(scan: &str, profiles: &str) -> Vec<WifiNetwork> {
    let profiles: BTreeMap<_, _> = profiles
        .lines()
        .filter_map(|line| {
            let [uuid, kind, name]: [String; 3] = fields(line).try_into().ok()?;
            (kind == "802-11-wireless" || kind == "wifi").then_some((name, uuid))
        })
        .collect();
    let mut discovered: BTreeMap<String, WifiNetwork> = BTreeMap::new();
    for line in scan.lines() {
        let Ok([active, ssid, signal, security]) = <[String; 4]>::try_from(fields(line)) else {
            continue;
        };
        if ssid.is_empty() {
            continue;
        }
        let network = WifiNetwork {
            profile: profiles.get(&ssid).cloned(),
            ssid: ssid.clone(),
            signal: signal.parse::<u8>().unwrap_or(0).min(100),
            secured: !matches!(security.trim(), "" | "--"),
            connected: active == "*",
        };
        let best = discovered.entry(ssid).or_insert_with(|| network.clone());
        if (network.connected, network.signal) > (best.connected, best.signal) {
            *best = network;
        }
    }
    let mut result: Vec<_> = discovered.into_values().collect();
    result.sort_by_key(|network| {
        (
            std::cmp::Reverse(network.connected),
            std::cmp::Reverse(network.signal),
        )
    });
    result.truncate(40);
    result
}

fn query_nmcli(nmcli: &Path) -> WifiStatus {
    let Some(radios) = run_cmd_capture(nmcli, &["-t", "-f", "WIFI-HW,WIFI", "radio"]) else {
        return WifiStatus::default();
    };
    let Some((hardware, radio)) = radios.trim().split_once(':') else {
        return WifiStatus::default();
    };
    let available = hardware == "enabled";
    let enabled = available && radio == "enabled";
    if !enabled {
        return WifiStatus {
            available,
            ..WifiStatus::default()
        };
    }
    let scan = run_cmd_capture(
        nmcli,
        &[
            "-t",
            "-f",
            "IN-USE,SSID,SIGNAL,SECURITY",
            "device",
            "wifi",
            "list",
            "--rescan",
            "no",
        ],
    )
    .unwrap_or_default();
    let profiles = run_cmd_capture(nmcli, &["-t", "-f", "UUID,TYPE,NAME", "connection", "show"])
        .unwrap_or_default();
    let networks = networks(&scan, &profiles);
    let active = networks.iter().find(|network| network.connected);
    WifiStatus {
        available,
        enabled,
        connected: active.is_some(),
        ssid: active.map(|network| network.ssid.clone()),
        signal: active.map_or(0, |network| network.signal),
        networks,
        error: None,
    }
}

impl WifiBackend for NmcliWifiBackend {
    fn poll(&mut self) -> Option<WifiStatus> {
        self.latest.lock().ok()?.take()
    }
    fn set_enabled(&mut self, on: bool) {
        self.send(WifiCmd::SetEnabled(on));
    }
    fn set_airplane(&mut self, airplane: bool) {
        self.send(WifiCmd::SetAirplane(airplane));
    }
    fn join(&mut self, network: WifiNetwork) {
        self.send(WifiCmd::Join(network));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_access_point_wins_and_profiles_match_escaped_names() {
        let scan = "*:Lab\\:East:40:WPA2\n :Lab\\:East:99:WPA2\n :Cafe:50:--";
        let entries = networks(scan, "saved-uuid:802-11-wireless:Lab\\:East");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].profile.as_deref(), Some("saved-uuid"));
        assert_eq!(entries[0].signal, 40);
        assert!(entries[0].connected && !entries[1].secured);
    }
}
