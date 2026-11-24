//! Linux backend for the Shiraz Android Compatibility Subsystem.
//!
//! Manages the persistent `shiraz-daemon` process, communicates non-blockingly over
//! `$XDG_RUNTIME_DIR/shiraz/control.sock` (JSON lines IPC), syncs XDG Autostart v0.5
//! (`$XDG_CONFIG_HOME/autostart/org.spectreos.Shiraz.desktop`), and reads live telemetry
//! from `$XDG_RUNTIME_DIR/shiraz/status.json`.

use crate::backends::linux::find_in_path;
use crate::services::shiraz::{ShirazBackend, ShirazConfig};
use spectre_settings::{
    ShirazRuntimeInfo, detect_shiraz_root, load_shiraz_runtime_info, xdg_config_home,
    xdg_runtime_dir,
};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::sync::mpsc;

pub struct LinuxShirazBackend {
    shiraz_root: Option<PathBuf>,
    daemon_bin: Option<PathBuf>,
    launch_bin: Option<PathBuf>,
    child: Option<Child>,
    status_request: mpsc::SyncSender<()>,
    status_result: mpsc::Receiver<ShirazRuntimeInfo>,
    wayland_display: Option<String>,
    width: u32,
    height: u32,
    scale: f32,
    config: ShirazConfig,
}

impl LinuxShirazBackend {
    pub fn discover() -> Self {
        let shiraz_root = detect_shiraz_root();
        let daemon_bin = find_in_path("shiraz-daemon").or_else(|| {
            shiraz_root
                .as_ref()
                .map(|r| r.join("bin/shiraz-daemon"))
                .filter(|p| p.is_file())
        });
        let launch_bin = find_in_path("shiraz-launch").or_else(|| {
            shiraz_root
                .as_ref()
                .map(|r| r.join("bin/shiraz-launch"))
                .filter(|p| p.is_file())
        });
        let (status_request, requests) = mpsc::sync_channel(1);
        let (results, status_result) = mpsc::channel();
        let _ = std::thread::Builder::new().name("shiraz-status".into()).spawn(move || {
            while requests.recv().is_ok() {
                if results.send(load_shiraz_runtime_info()).is_err() {
                    break;
                }
            }
        });
        Self {
            shiraz_root,
            daemon_bin,
            launch_bin,
            child: None,
            status_request,
            status_result,
            wayland_display: std::env::var("WAYLAND_DISPLAY").ok(),
            width: 400,
            height: 765,
            scale: 1.0,
            config: ShirazConfig::default(),
        }
    }

    fn control_sock_path() -> Option<PathBuf> {
        xdg_runtime_dir().map(|d| d.join("shiraz/control.sock"))
    }

    fn send_ipc(&self, json_line: &str, timeout: Duration) -> Result<String, String> {
        let sock = Self::control_sock_path().ok_or_else(|| "XDG_RUNTIME_DIR unset".to_string())?;
        let mut stream =
            UnixStream::connect(&sock).map_err(|e| format!("connect {}: {e}", sock.display()))?;
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
        stream
            .write_all(json_line.as_bytes())
            .and_then(|_| stream.write_all(b"\n"))
            .and_then(|_| stream.flush())
            .map_err(|e| format!("write IPC: {e}"))?;
        let mut reader = BufReader::new(stream);
        let mut reply = String::new();
        reader
            .read_line(&mut reply)
            .map_err(|e| format!("read IPC: {e}"))?;
        Ok(reply)
    }

    fn daemon_alive(&mut self) -> bool {
        if let Some(ref mut ch) = self.child {
            match ch.try_wait() {
                Ok(None) => return true,
                _ => self.child = None,
            }
        }
        Self::control_sock_path().is_some_and(|p| p.exists())
            && self
                .send_ipc(r#"{"cmd":"status"}"#, Duration::from_millis(80))
                .is_ok()
    }

    fn apply_base_env(&self, cmd: &mut Command) {
        cmd.env_remove("LD_LIBRARY_PATH")
            .env_remove("BHOO_FONT")
            .env_remove("RUST_LOG")
            .env("SHIRAZ_FORM_FACTOR", "mobile")
            .env("XDG_SESSION_TYPE", "wayland")
            .env("SDL_VIDEODRIVER", "wayland")
            .env("XDG_CURRENT_DESKTOP", "Wisp:SpectreOS:GNOME");
        if let Some(ref wd) = self.wayland_display {
            cmd.env("WAYLAND_DISPLAY", wd);
        }
        if let Some(ref root) = self.shiraz_root {
            cmd.env("SHIRAZ_HOME", root);
            let py_src = root.join("src");
            let existing = std::env::var("PYTHONPATH").unwrap_or_default();
            let py_path = if existing.is_empty() {
                py_src.display().to_string()
            } else {
                format!("{}:{existing}", py_src.display())
            };
            cmd.env("PYTHONPATH", py_path);
        }
    }

    fn spawn_daemon(&mut self, cfg: ShirazConfig) {
        if self.daemon_alive() {
            return;
        }
        let mut cmd = if let Some(ref bin) = self.daemon_bin {
            Command::new(bin)
        } else if self.shiraz_root.is_some() {
            let mut c = Command::new("python3");
            c.args(["-m", "shiraz", "daemon"]);
            c
        } else {
            tracing::warn!("Shiraz runtime not found; cannot spawn shiraz-daemon");
            return;
        };
        if !cfg.prewarm_memory {
            cmd.arg("--no-prewarm");
        }
        if !cfg.aot_enabled {
            cmd.arg("--no-aot");
        }
        if cfg.sandbox {
            cmd.arg("--sandbox");
        }
        cmd.args([
            "--width",
            &self.width.to_string(),
            "--height",
            &self.height.to_string(),
            "--scale",
            &format!("{:.2}", self.scale),
        ]);
        self.apply_base_env(&mut cmd);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        match cmd.spawn() {
            Ok(child) => {
                tracing::info!("Started Shiraz warm runtime daemon (pid={})", child.id());
                self.child = Some(child);
            }
            Err(err) => {
                tracing::warn!("Failed to start shiraz-daemon: {err}");
            }
        }
    }

    fn push_configure(&self) {
        let wd = self.wayland_display.as_deref().unwrap_or("");
        let dbus = std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap_or_default();
        let payload = format!(
            r#"{{"cmd":"configure","prewarm_memory":{},"aot_enabled":{},"sandbox":{},"width":{},"height":{},"scale":{:.2},"wayland_display":"{}","dbus_address":"{}"}}"#,
            self.config.prewarm_memory,
            self.config.aot_enabled,
            self.config.sandbox,
            self.width,
            self.height,
            self.scale,
            escape_json(wd),
            escape_json(&dbus),
        );
        let _ = self.send_ipc(&payload, Duration::from_millis(120));
    }

    fn sync_autostart_file(&self, enabled: bool) {
        let Some(cfg_home) = xdg_config_home() else {
            return;
        };
        let autostart_dir = cfg_home.join("autostart");
        let path = autostart_dir.join("org.spectreos.Shiraz.desktop");
        if !enabled {
            let _ = std::fs::remove_file(&path);
            return;
        }
        let _ = std::fs::create_dir_all(&autostart_dir);
        let exec = if let Some(ref bin) = self.daemon_bin {
            bin.display().to_string()
        } else if let Some(ref root) = self.shiraz_root {
            format!(
                "env SHIRAZ_HOME={} PYTHONPATH={} python3 -m shiraz daemon",
                root.display(),
                root.join("src").display()
            )
        } else {
            "shiraz-daemon".to_string()
        };
        let body = format!(
            "[Desktop Entry]\nType=Application\nVersion=1.5\nName=Shiraz Android Runtime\nComment=Pre-warm Shiraz Android compatibility runtime in memory\nExec={exec}\nHidden=false\nX-GNOME-Autostart-enabled=true\nOnlyShowIn=Wisp;SpectreOS;Phosh;\n"
        );
        let tmp = path.with_extension("desktop.tmp");
        if std::fs::write(&tmp, body).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl ShirazBackend for LinuxShirazBackend {
    fn is_available(&self) -> bool {
        self.shiraz_root.is_some() || self.daemon_bin.is_some()
    }

    fn apply_config(&mut self, cfg: ShirazConfig) {
        self.config = cfg;
        self.sync_autostart_file(cfg.enabled && cfg.autostart);
        if cfg.enabled {
            if self.daemon_alive() {
                self.push_configure();
            } else {
                self.spawn_daemon(cfg);
            }
        } else {
            self.shutdown();
        }
    }

    fn set_wayland_env(&mut self, wayland_display: &str, width: u32, height: u32, scale: f32) {
        self.wayland_display = Some(wayland_display.to_string());
        self.width = width.max(120);
        self.height = height.max(120);
        self.scale = scale.max(0.5);
        if self.config.enabled && self.daemon_alive() {
            self.push_configure();
        }
    }

    fn poll_status(&mut self) -> Option<ShirazRuntimeInfo> {
        if let Some(ref mut ch) = self.child
            && let Ok(Some(_)) = ch.try_wait()
        {
            self.child = None;
        }
        let _ = self.status_request.try_send(());
        let mut info = self.status_result.try_iter().last()?;
        info.available = self.is_available();
        Some(info)
    }

    fn launch_package(&mut self, package: &str, width: u32, height: u32) -> Result<(), String> {
        self.width = width.max(120);
        self.height = height.max(120);
        // Fast path: dispatch over Unix IPC on a short background thread so the 60 Hz compositor
        // never stalls waiting for `HostSession.start()`, and give it a generous 3s timeout
        // before ever falling back to `shiraz-launch` (prevents duplicate launches on ARM64!).
        let sock = Self::control_sock_path();
        let pkg = package.to_string();
        let w = self.width;
        let h = self.height;
        let launch_bin = self.launch_bin.clone();
        let shiraz_root = self.shiraz_root.clone();
        let wayland_display = self.wayland_display.clone();
        let payload = format!(
            r#"{{"cmd":"launch","package":"{}","width":{},"height":{}}}"#,
            escape_json(package),
            w,
            h
        );
        if sock.as_ref().is_some_and(|p| p.exists()) {
            std::thread::spawn(move || {
                let ipc_ok = sock.as_ref().is_some_and(|sock_path| {
                    UnixStream::connect(sock_path)
                        .and_then(|mut stream| {
                            let _ = stream.set_read_timeout(Some(Duration::from_millis(3000)));
                            let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                            stream.write_all(payload.as_bytes())?;
                            stream.write_all(b"\n")?;
                            stream.flush()?;
                            let mut reader = BufReader::new(stream);
                            let mut reply = String::new();
                            reader.read_line(&mut reply)?;
                            Ok(reply.contains(r#""ok": true"#) || reply.contains(r#""ok":true"#))
                        })
                        .unwrap_or(false)
                });
                if ipc_ok {
                    tracing::info!("Launched Shiraz Android app '{pkg}' via warm daemon");
                    return;
                }
                tracing::warn!("Shiraz daemon IPC launch failed for '{pkg}', falling back to shiraz-launch");
                let mut cmd = if let Some(ref bin) = launch_bin {
                    let mut c = Command::new(bin);
                    c.arg(&pkg);
                    c
                } else if shiraz_root.is_some() {
                    let mut c = Command::new("python3");
                    c.args(["-m", "shiraz", "launch-app", &pkg]);
                    c
                } else {
                    return;
                };
                cmd.args(["--width", &w.to_string(), "--height", &h.to_string()])
                    .env_remove("LD_LIBRARY_PATH")
                    .env_remove("BHOO_FONT")
                    .env_remove("RUST_LOG")
                    .env("SHIRAZ_FORM_FACTOR", "mobile")
                    .env("XDG_SESSION_TYPE", "wayland")
                    .env("SDL_VIDEODRIVER", "wayland")
                    .env("XDG_CURRENT_DESKTOP", "Wisp:SpectreOS:GNOME");
                if let Some(ref wd) = wayland_display {
                    cmd.env("WAYLAND_DISPLAY", wd);
                }
                if let Some(ref root) = shiraz_root {
                    cmd.env("SHIRAZ_HOME", root);
                    cmd.env("PYTHONPATH", root.join("src"));
                }
                let _ = cmd
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            });
            return Ok(());
        }

        // Cold/warming fallback: spawn `shiraz-launch` detached so compositor stays 60Hz smooth
        let mut cmd = if let Some(ref bin) = self.launch_bin {
            let mut c = Command::new(bin);
            c.arg(package);
            c
        } else if self.shiraz_root.is_some() {
            let mut c = Command::new("python3");
            c.args(["-m", "shiraz", "launch-app", package]);
            c
        } else {
            return Err("Shiraz runtime is not installed".into());
        };
        cmd.args([
            "--width",
            &self.width.to_string(),
            "--height",
            &self.height.to_string(),
        ]);
        self.apply_base_env(&mut cmd);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd.spawn()
            .map(|_| {
                tracing::info!("Spawned shiraz-launch for '{package}'");
            })
            .map_err(|e| format!("spawn shiraz-launch failed: {e}"))
    }

    fn stop_package(&mut self, package: &str) {
        let payload = format!(r#"{{"cmd":"stop","package":"{}"}}"#, escape_json(package));
        let _ = self.send_ipc(&payload, Duration::from_millis(250));
    }

    fn stop_all(&mut self) {
        let _ = self.send_ipc(r#"{"cmd":"stop_all"}"#, Duration::from_millis(250));
    }

    fn refresh(&mut self) {
        let _ = self.send_ipc(r#"{"cmd":"refresh"}"#, Duration::from_millis(250));
    }

    fn shutdown(&mut self) {
        let _ = self.send_ipc(r#"{"cmd":"shutdown"}"#, Duration::from_millis(200));
        if let Some(mut ch) = self.child.take() {
            // Give daemon a brief grace window to munmap and remove control.sock
            for _ in 0..8 {
                if matches!(ch.try_wait(), Ok(Some(_))) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            let _ = ch.kill();
            let _ = ch.wait();
        }
    }
}

impl Drop for LinuxShirazBackend {
    fn drop(&mut self) {
        if self.child.is_some() {
            self.shutdown();
        }
    }
}
