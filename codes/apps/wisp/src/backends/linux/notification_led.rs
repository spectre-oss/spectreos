//! Linux sysfs notification & charging status LED backend (e.g. `/sys/class/leds/white:status`).

use crate::services::notification_led::{NotificationLedBackend, NotificationLedMode};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct SysfsNotificationLedBackend {
    tx: Option<mpsc::Sender<NotificationLedMode>>,
    label: String,
    available: bool,
}

impl SysfsNotificationLedBackend {
    fn discover_node() -> Option<(PathBuf, u32, String)> {
        let entries = std::fs::read_dir("/sys/class/leds").ok()?;
        let mut candidates: Vec<(i32, PathBuf, String)> = Vec::new();
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let lower = name.to_ascii_lowercase();
            if lower.contains("flash")
                || lower.contains("torch")
                || lower.starts_with("mmc")
                || lower.contains("kbd")
                || lower.contains("backlight")
            {
                continue;
            }
            let prio = if lower == "white:status" || lower.contains("notification") {
                0
            } else if lower.contains("status") || lower.contains("indicator") {
                1
            } else if lower.contains("white") || lower.contains("green") || lower.contains("blue") {
                2
            } else {
                continue;
            };
            candidates.push((prio, p, name));
        }
        candidates.sort_by_key(|(prio, _, _)| *prio);
        let (_, dir, name) = candidates.into_iter().next()?;
        let max_b = std::fs::read_to_string(dir.join("max_brightness"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(255)
            .max(1);
        Some((dir, max_b, name))
    }

    pub fn start() -> Self {
        let Some((dir, max_b, name)) = Self::discover_node() else {
            return Self {
                tx: None,
                label: "none".into(),
                available: false,
            };
        };

        let _ = std::fs::write(dir.join("trigger"), "none");
        let brightness_path = dir.join("brightness");
        let (tx, rx) = mpsc::channel::<NotificationLedMode>();

        let _ = thread::Builder::new()
            .name("wisp-notif-led".into())
            .spawn(move || {
                let mut mode = NotificationLedMode::Off;
                let write_level = |v: u32| {
                    let _ = std::fs::write(&brightness_path, v.min(max_b).to_string());
                };
                write_level(0);

                loop {
                    match mode {
                        NotificationLedMode::Off => {
                            write_level(0);
                            match rx.recv() {
                                Ok(m) => mode = m,
                                Err(_) => break,
                            }
                        }
                        NotificationLedMode::Charging => {
                            write_level(((max_b as f32 * 0.45).round() as u32).max(1));
                            match rx.recv() {
                                Ok(m) => mode = m,
                                Err(_) => break,
                            }
                        }
                        NotificationLedMode::Charged => {
                            write_level(((max_b as f32 * 0.15).round() as u32).max(1));
                            match rx.recv() {
                                Ok(m) => mode = m,
                                Err(_) => break,
                            }
                        }
                        NotificationLedMode::NotificationPulse { charging } => {
                            let floor = if charging {
                                (max_b as f32 * 0.14).round()
                            } else {
                                0.0
                            };
                            let peak = (max_b as f32 * 0.85).round();
                            let mut interrupted = false;

                            for step in 0..=24 {
                                let phase = (step as f32 / 24.0) * std::f32::consts::TAU;
                                let wave = 0.5 - 0.5 * phase.cos();
                                let val = (floor + (peak - floor) * wave).round() as u32;
                                write_level(val);
                                match rx.recv_timeout(Duration::from_millis(50)) {
                                    Ok(m) => {
                                        mode = m;
                                        interrupted = true;
                                        break;
                                    }
                                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                                }
                            }
                            if interrupted {
                                continue;
                            }
                            write_level(floor as u32);
                            match rx.recv_timeout(Duration::from_millis(1800)) {
                                Ok(m) => mode = m,
                                Err(mpsc::RecvTimeoutError::Timeout) => {}
                                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                            }
                        }
                    }
                }
            });

        Self {
            tx: Some(tx),
            label: format!("led:{name}"),
            available: true,
        }
    }
}

impl NotificationLedBackend for SysfsNotificationLedBackend {
    fn is_available(&self) -> bool {
        self.available
    }

    fn backend_label(&self) -> &str {
        &self.label
    }

    fn set_mode(&mut self, mode: NotificationLedMode) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(mode);
        }
    }
}
