//! Platform-agnostic Shiraz Android Compatibility Subsystem service.
//!
//! Coordinates the persistent warm Shiraz runtime daemon (`shiraz-daemon`), memory
//! pre-warming, AOT compilation state, XDG Autostart synchronization, and live
//! telemetry synchronization into `SystemState::shiraz`.

use spectre_settings::{ShirazRuntimeInfo, ShirazRuntimeState, SystemState};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShirazConfig {
    pub enabled: bool,
    pub prewarm_memory: bool,
    pub aot_enabled: bool,
    pub autostart: bool,
    pub sandbox: bool,
}

impl ShirazConfig {
    pub fn from_state(sys: &SystemState) -> Self {
        Self {
            enabled: sys.shiraz_enabled,
            prewarm_memory: sys.shiraz_prewarm_memory,
            aot_enabled: sys.shiraz_aot_enabled,
            autostart: sys.shiraz_autostart,
            sandbox: sys.shiraz_sandbox,
        }
    }
}

pub trait ShirazBackend: Send {
    fn is_available(&self) -> bool;
    fn apply_config(&mut self, cfg: ShirazConfig);
    fn set_wayland_env(&mut self, wayland_display: &str, width: u32, height: u32, scale: f32);
    fn poll_status(&mut self) -> Option<ShirazRuntimeInfo>;
    fn launch_package(&mut self, package: &str, width: u32, height: u32) -> Result<(), String>;
    fn stop_package(&mut self, package: &str);
    fn stop_all(&mut self);
    fn refresh(&mut self);
    fn shutdown(&mut self);
}

pub struct NullShirazBackend;

impl ShirazBackend for NullShirazBackend {
    fn is_available(&self) -> bool {
        false
    }
    fn apply_config(&mut self, _cfg: ShirazConfig) {}
    fn set_wayland_env(&mut self, _wayland_display: &str, _width: u32, _height: u32, _scale: f32) {}
    fn poll_status(&mut self) -> Option<ShirazRuntimeInfo> {
        None
    }
    fn launch_package(&mut self, _package: &str, _width: u32, _height: u32) -> Result<(), String> {
        Ok(())
    }
    fn stop_package(&mut self, _package: &str) {}
    fn stop_all(&mut self) {}
    fn refresh(&mut self) {}
    fn shutdown(&mut self) {}
}

pub struct ShirazService {
    backend: Box<dyn ShirazBackend>,
    last_config: ShirazConfig,
    last_poll: Instant,
    applied_initial: bool,
}

impl ShirazService {
    pub fn new(mut backend: Box<dyn ShirazBackend>, sys: &mut SystemState) -> Self {
        sys.shiraz.available = backend.is_available();
        if let Some(info) = backend.poll_status() {
            sys.shiraz = info;
        }
        let cfg = ShirazConfig::from_state(sys);
        let mut applied_initial = false;
        if cfg.enabled && backend.is_available() {
            backend.apply_config(cfg);
            if sys.shiraz.state == ShirazRuntimeState::Disabled {
                sys.shiraz.state = ShirazRuntimeState::Starting;
            }
            applied_initial = true;
        }
        Self {
            backend,
            last_config: cfg,
            last_poll: Instant::now() - Duration::from_secs(2),
            applied_initial,
        }
    }

    pub fn set_wayland_env(&mut self, wayland_display: &str, width: u32, height: u32, scale: f32) {
        self.backend
            .set_wayland_env(wayland_display, width, height, scale);
    }

    pub fn sync_from_state(&mut self, sys: &mut SystemState) {
        let cfg = ShirazConfig::from_state(sys);
        if cfg != self.last_config || (cfg.enabled && !self.applied_initial) {
            self.last_config = cfg;
            self.applied_initial = true;
            self.backend.apply_config(cfg);
            if cfg.enabled {
                if sys.shiraz.state == ShirazRuntimeState::Disabled {
                    sys.shiraz.state = ShirazRuntimeState::Starting;
                }
            } else {
                sys.shiraz.state = ShirazRuntimeState::Disabled;
                sys.shiraz.warm_ram_mb = 0;
                sys.shiraz.warm_files = 0;
                sys.shiraz.running_packages.clear();
                for app in &mut sys.shiraz.installed_apps {
                    app.running = false;
                }
            }
            // Force fast next status poll
            self.last_poll = Instant::now() - Duration::from_secs(2);
        }
    }

    pub fn poll(&mut self, sys: &mut SystemState) -> bool {
        let interval = if sys.shiraz_enabled {
            Duration::from_millis(500)
        } else {
            Duration::from_millis(2000)
        };
        if self.last_poll.elapsed() < interval {
            return false;
        }
        self.last_poll = Instant::now();
        if let Some(mut info) = self.backend.poll_status() {
            if sys.shiraz_enabled
                && info.state == ShirazRuntimeState::Disabled
                && info.error_message.is_none()
            {
                // Daemon is still booting up after being enabled; avoid flickering back to "Off".
                info.state = ShirazRuntimeState::Starting;
            }
            if info != sys.shiraz {
                sys.shiraz = info;
                return true;
            }
        }
        false
    }

    pub fn launch_package(
        &mut self,
        sys: &mut SystemState,
        package: &str,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        if !sys.shiraz_enabled {
            sys.shiraz_enabled = true;
            self.sync_from_state(sys);
        }
        let res = self.backend.launch_package(package, width, height);
        self.last_poll = Instant::now() - Duration::from_secs(2);
        res
    }

    pub fn stop_package(&mut self, package: &str) {
        self.backend.stop_package(package);
        self.last_poll = Instant::now() - Duration::from_secs(2);
    }

    pub fn stop_all(&mut self) {
        self.backend.stop_all();
        self.last_poll = Instant::now() - Duration::from_secs(2);
    }

    pub fn refresh(&mut self) {
        self.backend.refresh();
        self.last_poll = Instant::now() - Duration::from_secs(2);
    }

    pub fn shutdown(&mut self) {
        self.backend.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct MockShiraz {
        configs: Arc<Mutex<Vec<ShirazConfig>>>,
        launched: Arc<Mutex<Vec<String>>>,
        stopped: Arc<Mutex<Vec<String>>>,
        status: Arc<Mutex<ShirazRuntimeInfo>>,
    }

    impl ShirazBackend for MockShiraz {
        fn is_available(&self) -> bool {
            true
        }
        fn apply_config(&mut self, cfg: ShirazConfig) {
            self.configs.lock().unwrap().push(cfg);
            let mut st = self.status.lock().unwrap();
            st.state = if cfg.enabled {
                ShirazRuntimeState::Ready
            } else {
                ShirazRuntimeState::Disabled
            };
            st.warm_ram_mb = if cfg.enabled && cfg.prewarm_memory { 142 } else { 0 };
        }
        fn set_wayland_env(&mut self, _: &str, _: u32, _: u32, _: f32) {}
        fn poll_status(&mut self) -> Option<ShirazRuntimeInfo> {
            Some(self.status.lock().unwrap().clone())
        }
        fn launch_package(&mut self, package: &str, _: u32, _: u32) -> Result<(), String> {
            self.launched.lock().unwrap().push(package.to_string());
            Ok(())
        }
        fn stop_package(&mut self, package: &str) {
            self.stopped.lock().unwrap().push(package.to_string());
        }
        fn stop_all(&mut self) {
            self.stopped.lock().unwrap().push("*".to_string());
        }
        fn refresh(&mut self) {}
        fn shutdown(&mut self) {}
    }

    #[test]
    fn shiraz_service_toggles_and_auto_enables_on_launch() {
        let configs = Arc::new(Mutex::new(Vec::new()));
        let launched = Arc::new(Mutex::new(Vec::new()));
        let stopped = Arc::new(Mutex::new(Vec::new()));
        let status = Arc::new(Mutex::new(ShirazRuntimeInfo {
            available: true,
            ..ShirazRuntimeInfo::default()
        }));

        let mock = MockShiraz {
            configs: Arc::clone(&configs),
            launched: Arc::clone(&launched),
            stopped: Arc::clone(&stopped),
            status: Arc::clone(&status),
        };
        let mut sys = SystemState::default();
        sys.shiraz_enabled = false;
        let mut svc = ShirazService::new(Box::new(mock), &mut sys);
        assert!(sys.shiraz.available);
        assert!(configs.lock().unwrap().is_empty());

        // Launch auto-enables Shiraz and warms memory
        svc.launch_package(&mut sys, "net.osmand.plus", 400, 765)
            .unwrap();
        assert!(sys.shiraz_enabled);
        assert_eq!(configs.lock().unwrap().len(), 1);
        assert!(configs.lock().unwrap()[0].enabled);
        assert_eq!(launched.lock().unwrap().as_slice(), &["net.osmand.plus"]);
        assert!(svc.poll(&mut sys));
        assert_eq!(sys.shiraz.state, ShirazRuntimeState::Ready);
        assert_eq!(sys.shiraz.warm_ram_mb, 142);

        // Disable releases warm memory
        sys.shiraz_enabled = false;
        svc.sync_from_state(&mut sys);
        assert_eq!(sys.shiraz.state, ShirazRuntimeState::Disabled);
        assert_eq!(sys.shiraz.warm_ram_mb, 0);

        svc.stop_package("chat.delta");
        assert_eq!(stopped.lock().unwrap().as_slice(), &["chat.delta"]);
    }
}
