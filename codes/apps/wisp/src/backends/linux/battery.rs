//! Linux sysfs battery & external power supply backend (`/sys/class/power_supply/*`).

use crate::services::battery::{BatteryBackend, BatteryStatus};
use std::path::PathBuf;

pub struct SysfsBatteryBackend {
    battery_dir: Option<PathBuf>,
    charger_dirs: Vec<PathBuf>,
}

impl SysfsBatteryBackend {
    pub fn discover() -> Self {
        let mut battery_dir = None;
        let mut charger_dirs = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for p in entries.filter_map(|e| e.ok().map(|e| e.path())) {
                let kind = std::fs::read_to_string(p.join("type"))
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase();
                if kind == "battery" && battery_dir.is_none() {
                    battery_dir = Some(p);
                } else if matches!(kind.as_str(), "usb" | "mains" | "wireless") {
                    charger_dirs.push(p);
                }
            }
        }
        Self {
            battery_dir,
            charger_dirs,
        }
    }
}

impl BatteryBackend for SysfsBatteryBackend {
    fn poll(&mut self) -> Option<BatteryStatus> {
        let dir = self.battery_dir.as_ref()?;
        let level = std::fs::read_to_string(dir.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())?
            .min(100);
        let batt_status = std::fs::read_to_string(dir.join("status")).unwrap_or_default();
        let batt_charging = matches!(batt_status.trim(), "Charging" | "Full");
        let ext_online = self.charger_dirs.iter().any(|c| {
            std::fs::read_to_string(c.join("online"))
                .map(|s| s.trim() == "1")
                .unwrap_or(false)
        });
        Some(BatteryStatus {
            level,
            charging: batt_charging,
            plugged_in: ext_online || batt_charging,
        })
    }
}
