//! Modular Linux Mobile Hardware Backends (standard Linux sysfs/evdev/NetworkManager/BlueZ
//! + Qualcomm/Poco F1 auto-discovery).
//!
//! Each hardware capability lives in its own focused file so adding or swapping a driver
//! for another Linux phone only touches the relevant subsystem.

pub mod ambient_light;
pub mod audio;
pub mod backlight;
pub mod battery;
pub mod bluetooth;
pub mod fingerprint;
pub mod flashlight;
pub mod fonts;
pub mod gpu;
pub mod haptics;
pub mod notification_led;
pub mod power;
pub mod shiraz;
pub mod wifi;
mod time;

use crate::services::{
    AmbientLightBackend, DeviceBackends, FingerprintBackend, FlashlightBackend, FontBackend,
    FontPolicy, FontScript, HapticsBackend, NotificationLedBackend, ShirazBackend,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn find_in_path(bin: &str) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let cand = dir.join(bin);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    for dir in ["/run/current-system/sw/bin", "/usr/local/bin", "/usr/bin", "/bin"] {
        let cand = Path::new(dir).join(bin);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

pub(crate) fn spawn_detached(bin: &Path, args: &[&str]) {
    let _ = Command::new(bin)
        .args(args)
        .env_remove("LD_LIBRARY_PATH")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

pub(crate) fn run_cmd_capture(bin: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new(bin)
        .args(args)
        .env_remove("LD_LIBRARY_PATH")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

/// Auto-discover all Linux backends for the current device.
pub fn discover() -> DeviceBackends {
    let profile_name: &'static str = if Path::new(fingerprint::SPECTRE_FP_SOCKET).exists() {
        "linux-poco-f1"
    } else {
        "linux-generic"
    };

    let flashlight_drv = flashlight::SysfsFlashlightBackend::discover();
    let notif_led_drv = notification_led::SysfsNotificationLedBackend::start();
    let wifi_drv = wifi::NmcliWifiBackend::start();
    let bt_drv = bluetooth::BluezBluetoothBackend::start();
    let fp_drv = fingerprint::SpectreFingerprintBackend::start();
    let als_drv = ambient_light::LinuxAmbientLightBackend::start();
    let haptics_drv = haptics::EvdevHapticsBackend::start();
    let backlight_drv = backlight::SysfsBacklightBackend::discover();
    let battery_drv = battery::SysfsBatteryBackend::discover();
    let audio_drv = audio::PipewireAudioBackend;
    let power_drv = power::SystemdPowerBackend;
    let gpu_drv = gpu::DevfreqGpuBackend::discover();
    let shiraz_drv = shiraz::LinuxShirazBackend::discover();
    let fonts_drv = fonts::LinuxFontBackend::discover();
    let default_font_policy = FontPolicy::default();

    tracing::info!(
        "Linux backends ready (profile={profile_name}, torch={} [{}], notif_led={} [{}], haptics={}, als={}, fp_avail={}, fp_enrolled={}, shiraz_avail={}, fonts={} [primary={:?}, persian={:?}])",
        flashlight_drv.status().available,
        flashlight_drv.status().backend_label,
        notif_led_drv.is_available(),
        notif_led_drv.backend_label(),
        haptics_drv.is_available(),
        als_drv.is_available(),
        fp_drv.status().available,
        fp_drv.status().enrolled_count,
        shiraz_drv.is_available(),
        fonts_drv.backend_label(),
        fonts_drv.resolve_primary_font(&default_font_policy),
        fonts_drv.resolve_script_font(FontScript::Persian, &default_font_policy),
    );

    DeviceBackends {
        profile_name,
        time: Box::new(time::LinuxTimeBackend),
        flashlight: Box::new(flashlight_drv),
        notification_led: Box::new(notif_led_drv),
        wifi: Box::new(wifi_drv),
        bluetooth: Box::new(bt_drv),
        fingerprint: Box::new(fp_drv),
        ambient_light: Box::new(als_drv),
        haptics: Box::new(haptics_drv),
        backlight: Box::new(backlight_drv),
        battery: Box::new(battery_drv),
        audio: Box::new(audio_drv),
        power: Box::new(power_drv),
        gpu: Box::new(gpu_drv),
        shiraz: Box::new(shiraz_drv),
        fonts: Box::new(fonts_drv),
    }
}
