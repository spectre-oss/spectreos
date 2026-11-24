//! Platform-agnostic System Services for Wisp / SpectreOS.
//!
//! Each hardware capability lives in its own focused service module (owning its domain
//! state, policies, backend trait, and null fallback), while concrete OS/device drivers
//! live under [`crate::backends`] (e.g. `backends::linux::*` today, or Android HAL /
//! other phone backends in the future).

pub mod ambient_light;
pub mod audio;
pub mod battery;
pub mod bluetooth;
pub mod display;
pub mod fingerprint;
pub mod flashlight;
pub mod fonts;
pub mod gpu;
pub mod haptics;
pub mod notification_led;
pub mod power;
pub mod shiraz;
pub mod wifi;
pub mod time;

pub use ambient_light::{AmbientLightBackend, AmbientLightService, NullAmbientLightBackend, lux_to_brightness};
pub use audio::{AudioBackend, AudioService, NullAudioBackend};
pub use battery::{BatteryBackend, BatteryService, BatteryStatus, NullBatteryBackend};
pub use bluetooth::{BluetoothBackend, BluetoothService, BluetoothStatus, NullBluetoothBackend};
pub use display::{BacklightBackend, DisplayBacklightService, NullBacklightBackend};
pub use fingerprint::{
    FingerprintBackend, FingerprintEvent, FingerprintService, FingerprintStatus,
    NullFingerprintBackend, TouchIdState, json_get_bool, json_get_str, json_get_usize,
};
pub use flashlight::{FlashlightBackend, FlashlightService, FlashlightStatus, NullFlashlightBackend};
pub use fonts::{
    FontBackend, FontCatalog, FontFaceRef, FontPolicy, FontRole, FontScript, FontService,
    FontStylePreset, FontWeight, NullFontBackend, contains_persian_or_arabic,
};
pub use gpu::{GpuBoostBackend, GpuBoostService, NullGpuBoostBackend};
pub use haptics::{HapticEffect, HapticsBackend, HapticsService, NullHapticsBackend};
pub use notification_led::{
    NotificationLedBackend, NotificationLedMode, NotificationLedService,
    NullNotificationLedBackend, compute_notification_led_mode,
};
pub use power::{NullPowerBackend, PowerBackend, PowerService};
pub use shiraz::{NullShirazBackend, ShirazBackend, ShirazConfig, ShirazService};
pub use wifi::{NullWifiBackend, WifiBackend, WifiService, WifiStatus};

use spectre_settings::SystemState;
use std::time::Duration;

/// Asynchronous hardware event surfaced from [`SystemServices::poll`] into the shell / compositor.
#[derive(Clone, Debug, PartialEq)]
pub enum HardwareEvent {
    FingerprintStatusChanged(FingerprintStatus),
    FingerprintMatched,
    FingerprintRejected(String),
    AmbientLightLux(f32),
}

/// Dependency-injected bundle of platform backends for any target device.
///
/// Porting Wisp to a new phone or OS only requires constructing a `DeviceBackends`
/// bundle with implementations matching that platform's driver stack.
pub struct DeviceBackends {
    pub time: Box<dyn time::TimeBackend>,
    pub profile_name: &'static str,
    pub flashlight: Box<dyn FlashlightBackend>,
    pub notification_led: Box<dyn NotificationLedBackend>,
    pub wifi: Box<dyn WifiBackend>,
    pub bluetooth: Box<dyn BluetoothBackend>,
    pub fingerprint: Box<dyn FingerprintBackend>,
    pub ambient_light: Box<dyn AmbientLightBackend>,
    pub haptics: Box<dyn HapticsBackend>,
    pub backlight: Box<dyn BacklightBackend>,
    pub battery: Box<dyn BatteryBackend>,
    pub audio: Box<dyn AudioBackend>,
    pub power: Box<dyn PowerBackend>,
    pub gpu: Box<dyn GpuBoostBackend>,
    pub shiraz: Box<dyn ShirazBackend>,
    pub fonts: Box<dyn FontBackend>,
}

impl DeviceBackends {
    /// Safe no-op backend bundle for desktop previews, WASM, and unit tests.
    pub fn null() -> Self {
        Self {
            time: Box::new(time::HostTimeBackend),
            profile_name: "null",
            flashlight: Box::new(NullFlashlightBackend),
            notification_led: Box::new(NullNotificationLedBackend),
            wifi: Box::new(NullWifiBackend),
            bluetooth: Box::new(NullBluetoothBackend),
            fingerprint: Box::new(NullFingerprintBackend),
            ambient_light: Box::new(NullAmbientLightBackend),
            haptics: Box::new(NullHapticsBackend),
            backlight: Box::new(NullBacklightBackend),
            battery: Box::new(NullBatteryBackend),
            audio: Box::new(NullAudioBackend),
            power: Box::new(NullPowerBackend),
            gpu: Box::new(NullGpuBoostBackend),
            shiraz: Box::new(NullShirazBackend),
            fonts: Box::new(NullFontBackend),
        }
    }
}

/// Unified coordinator owning all platform-agnostic system services.
pub struct SystemServices {
    pub time: time::TimeService,
    pub profile_name: &'static str,
    pub flashlight: FlashlightService,
    pub notification_led: NotificationLedService,
    pub wifi: WifiService,
    pub bluetooth: BluetoothService,
    pub fingerprint: FingerprintService,
    pub ambient_light: AmbientLightService,
    pub haptics: HapticsService,
    pub display: DisplayBacklightService,
    pub battery: BatteryService,
    pub audio: AudioService,
    pub power: PowerService,
    pub gpu: GpuBoostService,
    pub shiraz: ShirazService,
    pub fonts: FontService,
}

impl SystemServices {
    pub fn new(backends: DeviceBackends, sys: &mut SystemState) -> Self {
        let display = DisplayBacklightService::new(backends.backlight, sys);
        let flashlight = FlashlightService::new(backends.flashlight, sys);
        let fingerprint = FingerprintService::new(backends.fingerprint, sys);
        let battery = BatteryService::new(backends.battery, sys);
        let wifi = WifiService::new(backends.wifi, sys);
        let bluetooth = BluetoothService::new(backends.bluetooth, sys);
        let notification_led = NotificationLedService::new(
            backends.notification_led,
            battery.is_plugged_in(),
            sys.battery_level,
            sys.dnd_enabled,
        );
        let ambient_light = AmbientLightService::new(backends.ambient_light, sys.auto_brightness);
        let haptics = HapticsService::new(backends.haptics);
        let audio = AudioService::new(backends.audio, sys);
        let power = PowerService::new(backends.power);
        let gpu = GpuBoostService::new(backends.gpu);
        let shiraz = ShirazService::new(backends.shiraz, sys);
        let fonts = FontService::new(backends.fonts, sys);

        Self {
            time: time::TimeService(backends.time.into()),
            profile_name: backends.profile_name,
            flashlight,
            notification_led,
            wifi,
            bluetooth,
            fingerprint,
            ambient_light,
            haptics,
            display,
            battery,
            audio,
            power,
            gpu,
            shiraz,
            fonts,
        }
    }

    pub fn boost_gpu(&mut self, hold: Duration) {
        self.gpu.boost(hold);
    }

    pub fn release_gpu_boost(&mut self, force: bool) {
        self.gpu.release(force);
    }

    pub fn current_gpu_mhz(&self) -> u32 {
        self.gpu.current_mhz()
    }

    pub fn play_haptic(&mut self, effect: HapticEffect, sys: &SystemState) {
        self.haptics.play(effect, sys);
    }

    pub fn request_fingerprint_retry(&mut self) {
        self.fingerprint.request_retry();
    }

    pub fn set_backlight_dimmed(&mut self) {
        self.display.set_dimmed();
    }

    pub fn set_backlight_off(&mut self) {
        self.display.set_off();
    }

    pub fn set_backlight_on(&mut self) {
        self.display.set_on();
    }

    /// Non-blocking periodic poll across all services (Touch ID even with screen OFF,
    /// Notification/Charging LED, Adaptive Brightness ALS, Wi-Fi, Bluetooth, Battery).
    pub fn poll(
        &mut self,
        sys: &mut SystemState,
        screen_on: bool,
        locked: bool,
        has_unread_notifs: bool,
        pause_fingerprint: bool,
    ) -> (bool, Vec<HardwareEvent>) {
        let mut changed = false;
        let mut events = Vec::new();

        for ev in self.fingerprint.poll(sys, locked, pause_fingerprint) {
            changed = true;
            events.push(match ev {
                FingerprintEvent::StatusChanged(st) => HardwareEvent::FingerprintStatusChanged(st),
                FingerprintEvent::Matched => HardwareEvent::FingerprintMatched,
                FingerprintEvent::Rejected(msg) => HardwareEvent::FingerprintRejected(msg),
            });
        }

        let als = self.ambient_light.poll(sys, screen_on);
        if let Some(lux) = als.lux_event {
            events.push(HardwareEvent::AmbientLightLux(lux));
        }
        if let Some(norm) = als.new_brightness {
            self.display.apply_normalized(norm);
            changed = true;
        }

        if self.wifi.poll(sys) {
            changed = true;
        }
        if self.bluetooth.poll(sys) {
            changed = true;
        }
        if self.battery.poll(sys) {
            changed = true;
        }
        if self.shiraz.poll(sys) {
            changed = true;
        }

        self.notification_led.update(
            self.battery.is_plugged_in(),
            sys.battery_level,
            has_unread_notifs,
            screen_on,
            locked,
            sys.dnd_enabled,
        );

        (changed, events)
    }

    /// Push user changes made inside Wisp UI down through each platform-agnostic service.
    pub fn sync_from_shell(
        &mut self,
        sys: &mut SystemState,
        power_off_requested: bool,
        reboot_requested: bool,
        screen_on: bool,
    ) {
        if let Some(target) = self.ambient_light.sync_toggle(sys.auto_brightness) {
            sys.brightness = target;
            self.display.apply_normalized(target);
        }
        if let Some(manual_norm) = self.display.sync_slider_from_state(sys, screen_on) {
            self.ambient_light
                .record_manual_slider(manual_norm, sys.auto_brightness);
        }

        self.flashlight.sync_from_state(sys);
        self.wifi.sync_from_state(sys);
        self.bluetooth.sync_from_state(sys);
        self.audio.sync_from_state(sys);
        self.shiraz.sync_from_state(sys);
        self.fonts.sync_from_state(sys);
        self.power
            .sync_requests(power_off_requested, reboot_requested);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn system_services_keeps_touch_id_armed_when_screen_off_and_drives_led() {
        struct MockFp {
            listening: Arc<Mutex<bool>>,
            warm: Arc<Mutex<bool>>,
        }
        impl FingerprintBackend for MockFp {
            fn status(&self) -> FingerprintStatus {
                FingerprintStatus {
                    available: true,
                    enrolled_count: 1,
                    state: TouchIdState::Idle,
                    message: None,
                }
            }
            fn set_listening(&mut self, l: bool) {
                *self.listening.lock().unwrap() = l;
            }
            fn set_keep_warm(&mut self, w: bool) {
                *self.warm.lock().unwrap() = w;
            }
            fn request_retry(&mut self) {}
            fn poll_events(&mut self) -> Vec<FingerprintEvent> {
                Vec::new()
            }
        }

        struct MockLed(Arc<Mutex<NotificationLedMode>>);
        impl NotificationLedBackend for MockLed {
            fn is_available(&self) -> bool {
                true
            }
            fn backend_label(&self) -> &str {
                "mock:status"
            }
            fn set_mode(&mut self, mode: NotificationLedMode) {
                *self.0.lock().unwrap() = mode;
            }
        }

        struct MockBatt(BatteryStatus);
        impl BatteryBackend for MockBatt {
            fn poll(&mut self) -> Option<BatteryStatus> {
                Some(self.0)
            }
        }

        let fp_l = Arc::new(Mutex::new(false));
        let fp_w = Arc::new(Mutex::new(false));
        let led_m = Arc::new(Mutex::new(NotificationLedMode::Off));

        let mut backends = DeviceBackends::null();
        backends.fingerprint = Box::new(MockFp {
            listening: Arc::clone(&fp_l),
            warm: Arc::clone(&fp_w),
        });
        backends.notification_led = Box::new(MockLed(Arc::clone(&led_m)));
        backends.battery = Box::new(MockBatt(BatteryStatus {
            level: 65,
            charging: true,
            plugged_in: true,
        }));

        let mut sys = SystemState::default();
        let mut services = SystemServices::new(backends, &mut sys);
        assert_eq!(*led_m.lock().unwrap(), NotificationLedMode::Charging);
        assert!(*fp_w.lock().unwrap());

        let _ = services.poll(&mut sys, false, true, true, false);
        assert!(
            *fp_l.lock().unwrap(),
            "Touch ID must stay armed while screen is OFF"
        );
        assert_eq!(
            *led_m.lock().unwrap(),
            NotificationLedMode::NotificationPulse { charging: true }
        );
    }
}
