//! Linux Ambient Light Sensor backend (IIO sysfs -> Qualcomm SLPI `ssccli` -> `monitor-sensor`).

use super::find_in_path;
use crate::services::ambient_light::AmbientLightBackend;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

enum AlsBackendKind {
    IioSysfs {
        value_path: PathBuf,
        scale_path: Option<PathBuf>,
    },
    SscCli(PathBuf),
    MonitorSensor(PathBuf),
}

pub struct LinuxAmbientLightBackend {
    available: bool,
    active_flag: Arc<AtomicBool>,
    latest_lux: Arc<Mutex<Option<f32>>>,
}

impl LinuxAmbientLightBackend {
    fn discover_backend() -> Option<AlsBackendKind> {
        if let Ok(entries) = std::fs::read_dir("/sys/bus/iio/devices") {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.join("in_illuminance_input").is_file() {
                    return Some(AlsBackendKind::IioSysfs {
                        value_path: p.join("in_illuminance_input"),
                        scale_path: None,
                    });
                }
                if p.join("in_illuminance_raw").is_file() {
                    let scale = p.join("in_illuminance_scale");
                    return Some(AlsBackendKind::IioSysfs {
                        value_path: p.join("in_illuminance_raw"),
                        scale_path: scale.is_file().then_some(scale),
                    });
                }
            }
        }
        if let Some(ssc) = find_in_path("ssccli") {
            return Some(AlsBackendKind::SscCli(ssc));
        }
        if let Some(ms) = find_in_path("monitor-sensor") {
            return Some(AlsBackendKind::MonitorSensor(ms));
        }
        None
    }

    pub fn start() -> Self {
        let active_flag = Arc::new(AtomicBool::new(false));
        let latest_lux = Arc::new(Mutex::new(None));
        let Some(backend) = Self::discover_backend() else {
            return Self {
                available: false,
                active_flag,
                latest_lux,
            };
        };

        let active_bg = Arc::clone(&active_flag);
        let lux_bg = Arc::clone(&latest_lux);
        let _ = thread::Builder::new()
            .name("wisp-als".into())
            .spawn(move || match backend {
                AlsBackendKind::IioSysfs {
                    value_path,
                    scale_path,
                } => loop {
                    if active_bg.load(Ordering::Relaxed)
                        && let Ok(raw_s) = std::fs::read_to_string(&value_path)
                        && let Ok(val) = raw_s.trim().parse::<f32>()
                    {
                        let scale = scale_path
                            .as_ref()
                            .and_then(|sp| std::fs::read_to_string(sp).ok())
                            .and_then(|s| s.trim().parse::<f32>().ok())
                            .unwrap_or(1.0);
                        let lux = (val * scale).max(0.0);
                        *lux_bg.lock().unwrap() = Some(lux);
                    }
                    thread::sleep(Duration::from_millis(750));
                },
                AlsBackendKind::SscCli(bin) => loop {
                    if !active_bg.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_millis(400));
                        continue;
                    }
                    if let Ok(mut child) = Command::new(&bin)
                        .args(["--sensor", "light", "--timeout", "2"])
                        .env_remove("LD_LIBRARY_PATH")
                        .stdin(Stdio::null())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                    {
                        if let Some(stdout) = child.stdout.take() {
                            for line in BufReader::new(stdout).lines().flatten() {
                                if let Some(lux) = parse_lux_line(&line) {
                                    *lux_bg.lock().unwrap() = Some(lux);
                                }
                                if !active_bg.load(Ordering::Relaxed) {
                                    let _ = child.kill();
                                    break;
                                }
                            }
                        }
                        let _ = child.wait();
                    }
                    for _ in 0..6 {
                        if !active_bg.load(Ordering::Relaxed) {
                            break;
                        }
                        thread::sleep(Duration::from_millis(250));
                    }
                },
                AlsBackendKind::MonitorSensor(bin) => {
                    let mut child: Option<Child> = None;
                    loop {
                        if !active_bg.load(Ordering::Relaxed) {
                            if let Some(mut c) = child.take() {
                                let _ = c.kill();
                                let _ = c.wait();
                            }
                            thread::sleep(Duration::from_millis(400));
                            continue;
                        }
                        if let Ok(mut c) = Command::new(&bin)
                            .args(["--light"])
                            .env_remove("LD_LIBRARY_PATH")
                            .stdin(Stdio::null())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::null())
                            .spawn()
                        {
                            if let Some(stdout) = c.stdout.take() {
                                for line in BufReader::new(stdout).lines().flatten() {
                                    if let Some(lux) = parse_lux_line(&line) {
                                        *lux_bg.lock().unwrap() = Some(lux);
                                    }
                                    if !active_bg.load(Ordering::Relaxed) {
                                        let _ = c.kill();
                                        break;
                                    }
                                }
                            }
                            let _ = c.wait();
                        }
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            });

        Self {
            available: true,
            active_flag,
            latest_lux,
        }
    }
}

fn parse_lux_line(line: &str) -> Option<f32> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("lux") && !lower.contains("light") {
        return None;
    }
    let after_colon = line.rsplit_once(':')?.1;
    let tok = after_colon.split_whitespace().next()?;
    let val = tok.parse::<f32>().ok()?;
    if val.is_finite() && val >= 0.0 {
        Some(val)
    } else {
        None
    }
}

impl AmbientLightBackend for LinuxAmbientLightBackend {
    fn is_available(&self) -> bool {
        self.available
    }

    fn set_active(&mut self, active: bool) {
        self.active_flag.store(active, Ordering::Relaxed);
    }

    fn poll_lux(&mut self) -> Option<f32> {
        self.latest_lux.lock().ok().and_then(|mut g| g.take())
    }
}
