//! Always-warm SpectreOS Fingerprint Lab Unix socket backend
//! (`/run/spectre-fingerprint/control.sock`).

use crate::services::fingerprint::{
    FingerprintBackend, FingerprintEvent, FingerprintStatus, TouchIdState, json_get_bool,
    json_get_str, json_get_usize,
};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

pub const SPECTRE_FP_SOCKET: &str = "/run/spectre-fingerprint/control.sock";

enum FpCmd {
    Wake,
}

pub struct SpectreFingerprintBackend {
    status: FingerprintStatus,
    listening: Arc<AtomicBool>,
    keep_warm: Arc<AtomicBool>,
    tx: Option<mpsc::Sender<FpCmd>>,
    ev_rx: Option<mpsc::Receiver<FingerprintEvent>>,
}

impl SpectreFingerprintBackend {
    fn query_once(sock_path: &Path, action: &str, timeout: Duration) -> Option<String> {
        let mut stream = UnixStream::connect(sock_path).ok()?;
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
        writeln!(stream, "{action}").ok()?;
        let _ = stream.flush();
        let reader = BufReader::new(stream);
        let mut last = None;
        for line in reader.lines() {
            let Ok(line) = line else { break };
            if json_get_str(&line, "event").as_deref() == Some("progress") {
                continue;
            }
            last = Some(line);
            break;
        }
        last
    }

    pub fn start() -> Self {
        Self::start_at(PathBuf::from(SPECTRE_FP_SOCKET))
    }

    fn start_at(sock_path: PathBuf) -> Self {
        let listening = Arc::new(AtomicBool::new(false));
        let keep_warm = Arc::new(AtomicBool::new(false));
        let (cmd_tx, cmd_rx) = mpsc::channel::<FpCmd>();
        let (ev_tx, ev_rx) = mpsc::channel::<FingerprintEvent>();
        let listening_bg = Arc::clone(&listening);
        let keep_warm_bg = Arc::clone(&keep_warm);

        let _ = thread::Builder::new()
            .name("wisp-touch-id".into())
            .spawn(move || {
                // A previous compositor's verification can still own the sensor
                // during startup. Only a successful enumeration establishes enrollment.
                let mut last_status = None;
                let enrolled = loop {
                    let ready = Self::query_once(&sock_path, "status", Duration::from_secs(2))
                        .is_some_and(|s| json_get_bool(&s, "ready") == Some(true)
                            && json_get_bool(&s, "ok") == Some(true));
                    let count = if ready {
                        Self::query_once(&sock_path, "list", Duration::from_secs(12))
                            .filter(|s| json_get_bool(s, "ok") == Some(true))
                            .and_then(|s| json_get_usize(&s, "enrolled_count"))
                    } else {
                        None
                    };
                    if let Some(count) = count {
                        if last_status != Some(count) {
                            let status = FingerprintStatus {
                                available: true, enrolled_count: count,
                                state: TouchIdState::Idle, message: None,
                            };
                            if ev_tx.send(FingerprintEvent::StatusChanged(status)).is_err() { return; }
                            last_status = Some(count);
                        }
                        if count > 0 { break count; }
                    }
                    let delay = if count == Some(0) { 15 } else { 2 };
                    if let Err(mpsc::RecvTimeoutError::Disconnected) =
                        cmd_rx.recv_timeout(Duration::from_secs(delay)) { return; }
                };

                let mut retry_after: Option<Instant> = None;

                loop {
                    let want_active =
                        listening_bg.load(Ordering::Relaxed) || keep_warm_bg.load(Ordering::Relaxed);
                    if !want_active {
                        if let Err(mpsc::RecvTimeoutError::Disconnected) = cmd_rx.recv_timeout(Duration::from_secs(5)) { return; }
                        continue;
                    }
                    if let Some(until) = retry_after {
                        let rem = until.saturating_duration_since(Instant::now());
                        if !rem.is_zero() {
                            let _ = cmd_rx.recv_timeout(rem);
                        }
                        retry_after = None;
                        while cmd_rx.try_recv().is_ok() {}
                        if !listening_bg.load(Ordering::Relaxed)
                            && !keep_warm_bg.load(Ordering::Relaxed)
                        {
                            continue;
                        }
                    }

                    let Ok(mut stream) = UnixStream::connect(&sock_path) else {
                        retry_after = Some(Instant::now() + Duration::from_secs(2));
                        continue;
                    };
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(125)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
                    if writeln!(stream, "verify").is_err() || stream.flush().is_err() {
                        retry_after = Some(Instant::now() + Duration::from_secs(1));
                        continue;
                    }

                    let reader = BufReader::new(stream);
                    let mut final_line: Option<String> = None;
                    for line in reader.lines() {
                        let Ok(line) = line else { break };
                        if json_get_str(&line, "event").as_deref() == Some("progress") {
                            let msg = json_get_str(&line, "title")
                                .unwrap_or_else(|| "Touch rear sensor".into());
                            let _ = ev_tx.send(FingerprintEvent::StatusChanged(
                                FingerprintStatus {
                                    available: true,
                                    enrolled_count: enrolled,
                                    state: TouchIdState::Listening,
                                    message: Some(msg),
                                },
                            ));
                            continue;
                        }
                        final_line = Some(line);
                        break;
                    }

                    while cmd_rx.try_recv().is_ok() {}

                    let is_locked = listening_bg.load(Ordering::Relaxed);
                    if let Some(res) = final_line {
                        let ok = json_get_bool(&res, "ok").unwrap_or(false);
                        let matched = json_get_bool(&res, "matched");
                        if is_locked && ok && matched == Some(true) {
                            let _ = ev_tx.send(FingerprintEvent::StatusChanged(
                                FingerprintStatus {
                                    available: true,
                                    enrolled_count: enrolled,
                                    state: TouchIdState::Matched,
                                    message: Some("Unlocked".into()),
                                },
                            ));
                            let _ = ev_tx.send(FingerprintEvent::Matched);
                            retry_after = Some(Instant::now() + Duration::from_millis(800));
                        } else if is_locked && ok && matched == Some(false) {
                            let _ = ev_tx.send(FingerprintEvent::StatusChanged(
                                FingerprintStatus {
                                    available: true,
                                    enrolled_count: enrolled,
                                    state: TouchIdState::Failed,
                                    message: Some("Not recognized".into()),
                                },
                            ));
                            let _ = ev_tx.send(FingerprintEvent::Rejected(
                                "Fingerprint not recognized".into(),
                            ));
                            retry_after = Some(Instant::now() + Duration::from_millis(200));
                        } else if !ok
                            && json_get_str(&res, "title")
                                .as_deref()
                                .is_some_and(|t| t.contains("already running"))
                        {
                            retry_after = Some(Instant::now() + Duration::from_millis(750));
                        } else {
                            retry_after = Some(Instant::now() + Duration::from_millis(80));
                        }
                    } else {
                        retry_after = Some(Instant::now() + Duration::from_millis(300));
                    }
                }
            });

        Self {
            status: FingerprintStatus::default(),
            listening,
            keep_warm,
            tx: Some(cmd_tx),
            ev_rx: Some(ev_rx),
        }
    }
}

impl FingerprintBackend for SpectreFingerprintBackend {
    fn status(&self) -> FingerprintStatus {
        self.status.clone()
    }

    fn set_listening(&mut self, listening: bool) {
        self.listening.store(listening, Ordering::Relaxed);
        if listening && let Some(tx) = &self.tx {
            let _ = tx.send(FpCmd::Wake);
        }
    }

    fn set_keep_warm(&mut self, keep_warm: bool) {
        self.keep_warm.store(keep_warm, Ordering::Relaxed);
        if keep_warm && let Some(tx) = &self.tx {
            let _ = tx.send(FpCmd::Wake);
        }
    }

    fn request_retry(&mut self) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(FpCmd::Wake);
        }
    }

    fn poll_events(&mut self) -> Vec<FingerprintEvent> {
        let mut out = Vec::new();
        if let Some(rx) = &self.ev_rx {
            while let Ok(ev) = rx.try_recv() {
                if let FingerprintEvent::StatusChanged(ref st) = ev {
                    self.status = st.clone();
                }
                out.push(ev);
            }
        }
        out
    }
}
