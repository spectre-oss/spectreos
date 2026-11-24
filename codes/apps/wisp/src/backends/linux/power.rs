//! Linux systemd (`systemctl`) poweroff & reboot backend.

use super::{find_in_path, spawn_detached};
use crate::services::power::PowerBackend;

pub struct SystemdPowerBackend;

impl PowerBackend for SystemdPowerBackend {
    fn poweroff(&mut self) {
        tracing::info!("Wisp Power menu triggered systemctl poweroff");
        if let Some(sc) = find_in_path("systemctl") {
            spawn_detached(&sc, &["poweroff"]);
        }
    }

    fn reboot(&mut self) {
        tracing::info!("Wisp Power menu triggered systemctl reboot");
        if let Some(sc) = find_in_path("systemctl") {
            spawn_detached(&sc, &["reboot"]);
        }
    }
}
