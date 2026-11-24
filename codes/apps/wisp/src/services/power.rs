//! System Power Control Service (poweroff & reboot, platform-agnostic).

/// Platform-agnostic backend contract for graceful system shutdown and reboot.
pub trait PowerBackend: Send {
    fn poweroff(&mut self);
    fn reboot(&mut self);
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullPowerBackend;

impl PowerBackend for NullPowerBackend {
    fn poweroff(&mut self) {}
    fn reboot(&mut self) {}
}

/// Domain service guarding one-shot shutdown and reboot execution.
pub struct PowerService {
    backend: Box<dyn PowerBackend>,
    cmd_sent: bool,
}

impl PowerService {
    pub fn new(backend: Box<dyn PowerBackend>) -> Self {
        Self {
            backend,
            cmd_sent: false,
        }
    }

    pub fn sync_requests(&mut self, power_off: bool, reboot: bool) {
        if self.cmd_sent {
            return;
        }
        if power_off {
            self.cmd_sent = true;
            self.backend.poweroff();
        } else if reboot {
            self.cmd_sent = true;
            self.backend.reboot();
        }
    }
}
