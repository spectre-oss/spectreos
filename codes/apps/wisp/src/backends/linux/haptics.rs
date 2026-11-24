//! Linux `evdev` Force-Feedback (`FF_RUMBLE`) haptics backend (e.g. `spmi_haptics`)
//! with `/sys/class/leds/vibrator` fallback.

use crate::services::haptics::{HapticEffect, HapticsBackend};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[repr(C)]
#[derive(Clone, Copy)]
struct FfTrigger {
    button: u16,
    interval: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FfReplay {
    length: u16,
    delay: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FfEffect {
    type_: u16,
    id: i16,
    direction: u16,
    trigger: FfTrigger,
    replay: FfReplay,
    _pad: u16,
    u: [u8; 32],
}

const EV_FF: u16 = 0x15;
const FF_RUMBLE: u16 = 0x50;
const EVIOCSFF: libc::c_ulong = 0x4030_4580;
const EVIOCRMFF: libc::c_ulong = 0x4004_4581;

enum HapticDevice {
    Evdev(File),
    Led(PathBuf, u32),
}

pub struct EvdevHapticsBackend {
    tx: Option<mpsc::Sender<HapticEffect>>,
    available: bool,
}

impl EvdevHapticsBackend {
    fn open_device() -> Option<HapticDevice> {
        let mut candidates: Vec<(i32, PathBuf, String)> = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/sys/class/input") {
            for entry in entries.flatten() {
                let fname = entry.file_name();
                let Some(ev_name) = fname.to_str() else {
                    continue;
                };
                if !ev_name.starts_with("event") {
                    continue;
                }
                let ff_caps = std::fs::read_to_string(entry.path().join("device/capabilities/ff"))
                    .unwrap_or_default();
                let has_ff = ff_caps
                    .split_whitespace()
                    .any(|word| u64::from_str_radix(word, 16).unwrap_or(0) != 0);
                if !has_ff {
                    continue;
                }
                let dev_name = std::fs::read_to_string(entry.path().join("device/name"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let lower = dev_name.to_ascii_lowercase();
                let priority = if lower.contains("haptic") || lower.contains("vibra") {
                    0
                } else {
                    1
                };
                candidates.push((priority, PathBuf::from("/dev/input").join(ev_name), dev_name));
            }
        }
        candidates.sort_by_key(|(prio, _, _)| *prio);
        for (_, dev_path, dev_name) in candidates {
            if let Ok(file) = OpenOptions::new().read(true).write(true).open(&dev_path) {
                tracing::info!(
                    "Haptics bound to evdev FF device {} ({dev_name})",
                    dev_path.display()
                );
                return Some(HapticDevice::Evdev(file));
            }
        }

        let vib = PathBuf::from("/sys/class/leds/vibrator");
        if vib.join("brightness").exists() {
            let max_b = std::fs::read_to_string(vib.join("max_brightness"))
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(255);
            return Some(HapticDevice::Led(vib, max_b));
        }
        None
    }

    pub fn start() -> Self {
        let Some(mut dev) = Self::open_device() else {
            return Self {
                tx: None,
                available: false,
            };
        };
        let (tx, rx) = mpsc::channel::<HapticEffect>();
        let _ = thread::Builder::new()
            .name("wisp-haptics".into())
            .spawn(move || {
                while let Ok(mut eff) = rx.recv() {
                    while let Ok(newer) = rx.try_recv() {
                        eff = newer;
                    }
                    play_effect_sync(&mut dev, eff);
                }
            });
        Self {
            tx: Some(tx),
            available: true,
        }
    }
}

fn pulse_evdev(file: &mut File, duration_ms: u16, strong: u16, weak: u16) {
    debug_assert_eq!(std::mem::size_of::<FfEffect>(), 48);
    let mut u = [0u8; 32];
    u[0..2].copy_from_slice(&strong.to_ne_bytes());
    u[2..4].copy_from_slice(&weak.to_ne_bytes());
    let mut effect = FfEffect {
        type_: FF_RUMBLE,
        id: -1,
        direction: 0,
        trigger: FfTrigger {
            button: 0,
            interval: 0,
        },
        replay: FfReplay {
            length: duration_ms,
            delay: 0,
        },
        _pad: 0,
        u,
    };
    let fd = file.as_raw_fd();
    let rc = unsafe { libc::ioctl(fd, EVIOCSFF as _, &mut effect as *mut FfEffect) };
    if rc < 0 || effect.id < 0 {
        return;
    }
    let play = libc::input_event {
        time: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        type_: EV_FF,
        code: effect.id as u16,
        value: 1,
    };
    let bytes = unsafe {
        std::slice::from_raw_parts(
            &play as *const libc::input_event as *const u8,
            std::mem::size_of::<libc::input_event>(),
        )
    };
    let _ = file.write_all(bytes);
    thread::sleep(Duration::from_millis(duration_ms as u64 + 8));
    unsafe {
        let _ = libc::ioctl(fd, EVIOCRMFF as _, effect.id as libc::c_int);
    }
}

fn pulse_led(dir: &Path, max_b: u32, duration_ms: u16) {
    let _ = std::fs::write(dir.join("brightness"), max_b.to_string());
    thread::sleep(Duration::from_millis(duration_ms as u64));
    let _ = std::fs::write(dir.join("brightness"), "0");
}

fn play_effect_sync(dev: &mut HapticDevice, eff: HapticEffect) {
    let steps: &[(u16, u16, u16, u64)] = match eff {
        HapticEffect::Tick => &[(14, 0x3800, 0x3800, 0)],
        HapticEffect::Click => &[(22, 0x6000, 0x5000, 0)],
        HapticEffect::LongPress => &[(36, 0x9000, 0x7000, 0)],
        HapticEffect::Confirm => &[(18, 0x6000, 0x5000, 45), (26, 0xA000, 0x8000, 0)],
        HapticEffect::Reject => &[(42, 0xD000, 0xA000, 55), (52, 0xE000, 0xB000, 0)],
    };
    for &(dur, strong, weak, gap) in steps {
        match dev {
            HapticDevice::Evdev(f) => pulse_evdev(f, dur, strong, weak),
            HapticDevice::Led(dir, max_b) => pulse_led(dir, *max_b, dur),
        }
        if gap > 0 {
            thread::sleep(Duration::from_millis(gap));
        }
    }
}

impl HapticsBackend for EvdevHapticsBackend {
    fn is_available(&self) -> bool {
        self.available
    }

    fn play(&mut self, effect: HapticEffect) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(effect);
        }
    }
}
