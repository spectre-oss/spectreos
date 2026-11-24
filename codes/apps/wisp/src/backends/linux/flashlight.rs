//! Linux sysfs rear camera flashlight / torch backend (`/sys/class/leds/*flash*` / `*torch*`).
//!
//! Strictly binds dedicated camera torch LEDs and never hijacks chassis notification/status LEDs.

use crate::services::flashlight::{FlashlightBackend, FlashlightStatus};
use std::path::PathBuf;

struct LedNode {
    dir: PathBuf,
    max_brightness: u32,
}

pub struct SysfsFlashlightBackend {
    nodes: Vec<LedNode>,
    label: String,
    on: bool,
}

impl SysfsFlashlightBackend {
    pub fn discover() -> Self {
        let mut torch_nodes: Vec<LedNode> = Vec::new();

        if let Ok(entries) = std::fs::read_dir("/sys/class/leds") {
            let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
            paths.sort();
            for p in paths {
                let name = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if name.contains("torch") || name.contains("flash") {
                    let max_b = std::fs::read_to_string(p.join("max_brightness"))
                        .ok()
                        .and_then(|s| s.trim().parse::<u32>().ok())
                        .unwrap_or(255)
                        .max(1);
                    torch_nodes.push(LedNode {
                        dir: p,
                        max_brightness: max_b,
                    });
                }
            }
        }

        let label = if torch_nodes.is_empty() {
            "none (kernel torch driver not bound)".into()
        } else {
            let names: Vec<String> = torch_nodes
                .iter()
                .filter_map(|n| n.dir.file_name().and_then(|s| s.to_str()).map(String::from))
                .collect();
            for n in &torch_nodes {
                let _ = std::fs::write(n.dir.join("brightness"), "0");
            }
            format!("torch:{}", names.join("+"))
        };

        Self {
            nodes: torch_nodes,
            label,
            on: false,
        }
    }
}

impl FlashlightBackend for SysfsFlashlightBackend {
    fn status(&self) -> FlashlightStatus {
        FlashlightStatus {
            available: !self.nodes.is_empty(),
            on: self.on,
            backend_label: self.label.clone(),
        }
    }

    fn set_enabled(&mut self, on: bool) -> bool {
        if self.nodes.is_empty() {
            self.on = false;
            return false;
        }
        let mut any_ok = false;
        for n in &self.nodes {
            if on {
                let _ = std::fs::write(n.dir.join("trigger"), "none");
            }
            let val = if on { n.max_brightness } else { 0 };
            if std::fs::write(n.dir.join("brightness"), val.to_string()).is_ok() {
                any_ok = true;
            }
        }
        if any_ok {
            self.on = on;
        }
        self.on
    }
}
