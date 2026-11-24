//! Linux sysfs display backlight backend (`/sys/class/backlight/*` + `login1` fallback).

use super::{find_in_path, spawn_detached};
use crate::services::display::BacklightBackend;
use std::path::PathBuf;

pub struct SysfsBacklightBackend {
    dir: Option<PathBuf>,
    max_brightness: u32,
}

impl SysfsBacklightBackend {
    pub fn discover() -> Self {
        let dir = std::fs::read_dir("/sys/class/backlight")
            .ok()
            .and_then(|mut it| it.find_map(|e| e.ok().map(|e| e.path())));
        let max_brightness = dir
            .as_ref()
            .and_then(|d| std::fs::read_to_string(d.join("max_brightness")).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(4095)
            .max(1);
        Self {
            dir,
            max_brightness,
        }
    }
}

impl BacklightBackend for SysfsBacklightBackend {
    fn max_raw(&self) -> u32 {
        self.max_brightness
    }

    fn read_raw(&self) -> Option<u32> {
        self.dir
            .as_ref()
            .and_then(|d| std::fs::read_to_string(d.join("brightness")).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
    }

    fn set_raw(&mut self, raw: u32, power_off: bool) {
        let Some(dir) = &self.dir else {
            return;
        };
        let val = raw.min(self.max_brightness);
        let wrote = std::fs::write(dir.join("brightness"), val.to_string()).is_ok();
        let _ = std::fs::write(dir.join("bl_power"), if power_off { "4" } else { "0" });
        if !wrote
            && let Some(name) = dir.file_name().and_then(|n| n.to_str())
            && let Some(busctl) = find_in_path("busctl")
        {
            let val_s = val.to_string();
            spawn_detached(
                &busctl,
                &[
                    "call",
                    "org.freedesktop.login1",
                    "/org/freedesktop/login1/session/auto",
                    "org.freedesktop.login1.Session",
                    "SetBrightness",
                    "ssu",
                    "backlight",
                    name,
                    &val_s,
                ],
            );
        }
    }
}
