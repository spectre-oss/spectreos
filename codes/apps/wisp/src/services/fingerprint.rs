//! Biometric Fingerprint / Touch ID Service (platform-agnostic).
//!
//! Coordinates always-warm sensor pre-initialization, screen-off wake/unlock listening,
//! retry signaling, and event delivery independently of whether the underlying device
//! uses `spectre-fingerprint`, `fprintd`, Android Biometric HAL, or a test mock.

use spectre_settings::SystemState;

/// Live Touch ID / fingerprint sensor state shown on the Lock Screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TouchIdState {
    #[default]
    Unavailable,
    Idle,
    Listening,
    Matched,
    Failed,
}

/// Live fingerprint subsystem snapshot from a [`FingerprintBackend`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FingerprintStatus {
    pub available: bool,
    pub enrolled_count: usize,
    pub state: TouchIdState,
    pub message: Option<String>,
}

/// Asynchronous biometric events delivered by a [`FingerprintBackend`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FingerprintEvent {
    StatusChanged(FingerprintStatus),
    Matched,
    Rejected(String),
}

/// Platform-agnostic backend contract for fingerprint verification hardware.
pub trait FingerprintBackend: Send {
    fn status(&self) -> FingerprintStatus;
    /// Enable or disable delivering match/reject unlock events (true while locked, screen ON or OFF).
    fn set_listening(&mut self, listening: bool);
    /// Keep the hardware sensor warm in low-power IRQ wait so unlock latency stays instant.
    fn set_keep_warm(&mut self, keep_warm: bool);
    /// Request an immediate re-arm if idle/failed while locked.
    fn request_retry(&mut self);
    /// Non-blocking drain of status updates and match/reject events.
    fn poll_events(&mut self) -> Vec<FingerprintEvent>;
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullFingerprintBackend;

impl FingerprintBackend for NullFingerprintBackend {
    fn status(&self) -> FingerprintStatus {
        FingerprintStatus::default()
    }

    fn set_listening(&mut self, _listening: bool) {}
    fn set_keep_warm(&mut self, _keep_warm: bool) {}
    fn request_retry(&mut self) {}
    fn poll_events(&mut self) -> Vec<FingerprintEvent> {
        Vec::new()
    }
}

/// Domain service managing always-warm Touch ID listening and status synchronization.
pub struct FingerprintService {
    backend: Box<dyn FingerprintBackend>,
    last_listening: bool,
    last_keep_warm: bool,
}

impl FingerprintService {
    pub fn new(mut backend: Box<dyn FingerprintBackend>, sys: &mut SystemState) -> Self {
        let st = backend.status();
        sys.touch_id_available = st.available;
        sys.touch_id_enrolled = st.enrolled_count;

        let initial_warm = st.available && st.enrolled_count > 0;
        if initial_warm {
            backend.set_keep_warm(true);
        }

        Self {
            backend,
            last_listening: false,
            last_keep_warm: initial_warm,
        }
    }

    pub fn status(&self) -> FingerprintStatus {
        self.backend.status()
    }

    pub fn is_available(&self) -> bool {
        self.backend.status().available
    }

    pub fn request_retry(&mut self) {
        self.backend.request_retry();
    }

    pub(crate) fn update_listening(&mut self, sys: &SystemState, accept_matches: bool, pause_for_diagnostic_app: bool) {
        let ready = sys.touch_id_available && sys.touch_id_enrolled > 0 && !pause_for_diagnostic_app;
        let want_listening = accept_matches && ready;
        let want_keep_warm = ready;

        if want_keep_warm != self.last_keep_warm {
            self.last_keep_warm = want_keep_warm;
            self.backend.set_keep_warm(want_keep_warm);
        }
        if want_listening != self.last_listening {
            self.last_listening = want_listening;
            self.backend.set_listening(want_listening);
        }
    }

    /// Update listening/warm state (keeps listening even while screen is OFF when locked!)
    /// and drain any pending biometric match/reject/status events.
    pub fn poll(
        &mut self,
        sys: &mut SystemState,
        locked: bool,
        pause_for_diagnostic_app: bool,
    ) -> Vec<FingerprintEvent> {
        self.update_listening(sys, locked, pause_for_diagnostic_app);

        let events = self.backend.poll_events();
        for ev in &events {
            if let FingerprintEvent::StatusChanged(st) = ev {
                sys.touch_id_available = st.available;
                sys.touch_id_enrolled = st.enrolled_count;
            }
        }
        events
    }
}

/// Minimal zero-dependency JSON field extractors for local IPC protocols
/// (e.g. `/run/spectre-fingerprint/control.sock` newline-delimited JSON responses).
pub fn json_get_bool(json: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let pos = json.find(&needle)? + needle.len();
    let rest = json[pos..].trim_start().strip_prefix(':')?.trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

pub fn json_get_usize(json: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\"");
    let pos = json.find(&needle)? + needle.len();
    let rest = json[pos..].trim_start().strip_prefix(':')?.trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

pub fn json_get_str(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let pos = json.find(&needle)? + needle.len();
    let rest = json[pos..].trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(esc) = chars.next() {
                out.push(match esc {
                    'n' => '\n',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                });
            }
        } else if c == '"' {
            return Some(out);
        } else {
            out.push(c);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_extractors_parse_fingerprint_protocol_messages() {
        let list_msg = r#"{"ok": true, "enrolled_count": 1, "title": "1 saved fingerprints"}"#;
        assert_eq!(json_get_bool(list_msg, "ok"), Some(true));
        assert_eq!(json_get_usize(list_msg, "enrolled_count"), Some(1));
        assert_eq!(
            json_get_str(list_msg, "title").as_deref(),
            Some("1 saved fingerprints")
        );

        let match_msg = r#"{"ok": true, "matched": true, "title": "Fingerprint matched"}"#;
        assert_eq!(json_get_bool(match_msg, "matched"), Some(true));

        let prog_msg = r#"{"event": "progress", "title": "Touch the sensor"}"#;
        assert_eq!(json_get_str(prog_msg, "event").as_deref(), Some("progress"));
        assert_eq!(
            json_get_str(prog_msg, "title").as_deref(),
            Some("Touch the sensor")
        );
    }
}
