//! System time-zone selection shared by the embedded and standalone Settings hosts.
use std::sync::mpsc::{self, Receiver, TryRecvError};

#[derive(Default)]
pub(crate) struct TimeZones {
    pub current: String,
    pub recent: Option<String>,
    pub zones: Vec<String>,
    pub regions: Vec<String>,
    pub error: Option<String>,
    pending: Option<Receiver<Result<ZoneData, String>>>,
}

struct ZoneData {
    current: String,
    zones: Option<Vec<String>>,
}

impl TimeZones {
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn request(&mut self, zone: Option<&str>) {
        if self.busy() {
            return;
        }
        if let Some(zone) = zone {
            if zone == self.current || !self.zones.iter().any(|z| z == zone) {
                return;
            }
        }
        self.error = None;
        #[cfg(target_os = "linux")]
        {
            let zone = zone.map(str::to_owned);
            let load_list = self.zones.is_empty();
            let (tx, rx) = mpsc::channel();
            match std::thread::Builder::new()
                .name("settings-time-zone".into())
                .spawn(move || {
                    let _ = tx.send(query(zone.as_deref(), load_list));
                }) {
                Ok(_) => self.pending = Some(rx),
                Err(e) => self.error = Some(e.to_string()),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.error = Some("System time zones can be changed on the phone.".into());
        }
    }

    pub fn poll(&mut self) {
        let Some(rx) = &self.pending else { return };
        let result = match rx.try_recv() {
            Ok(r) => r,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("Time zone service disconnected".into()),
        };
        self.pending = None;
        match result {
            Ok(data) => {
                if data.current != self.current && !self.current.is_empty() {
                    self.recent = Some(self.current.clone());
                }
                self.current = data.current;
                if let Some(zones) = data.zones {
                    self.regions = zones.iter().map(|z| region(z).to_owned()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
                    self.zones = zones;
                }
            }
            Err(error) => self.error = Some(error),
        }
    }
}

#[cfg(target_os = "linux")]
fn query(zone: Option<&str>, load_list: bool) -> Result<ZoneData, String> {
    fn run(args: &[&str]) -> Result<String, String> {
        let out = std::process::Command::new("timedatectl")
            .args(["--no-pager", "--no-ask-password"])
            .args(args)
            .output()
            .map_err(|e| format!("Cannot reach the time service: {e}"))?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    }
    if let Some(zone) = zone {
        run(&["set-timezone", zone])?;
    }
    let current = run(&["show", "-p", "Timezone", "--value"])?;
    if current.is_empty() {
        return Err("System returned an empty time zone".into());
    }
    let zones = if load_list {
        let mut zones: Vec<String> = run(&["list-timezones"])?
            .lines()
            .map(str::to_owned)
            .collect();
        zones.sort();
        zones.dedup();
        Some(zones)
    } else {
        None
    };
    Ok(ZoneData { current, zones })
}

pub(crate) fn region(zone: &str) -> &str {
    zone.split_once('/').map_or("Other", |(area, _)| area)
}
