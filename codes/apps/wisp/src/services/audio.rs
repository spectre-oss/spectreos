//! Audio Volume & Ringer Mute Service (platform-agnostic).

use spectre_settings::{RingerMode, SystemState};

/// Platform-agnostic backend contract for media volume and ringer mute control.
pub trait AudioBackend: Send {
    fn set_media_volume(&mut self, norm: f32);
    fn set_muted(&mut self, muted: bool);
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullAudioBackend;

impl AudioBackend for NullAudioBackend {
    fn set_media_volume(&mut self, _norm: f32) {}
    fn set_muted(&mut self, _muted: bool) {}
}

/// Domain service syncing media volume slider and ringer mode with the audio server.
pub struct AudioService {
    backend: Box<dyn AudioBackend>,
    last_vol_media: Option<f32>,
    last_ringer: Option<RingerMode>,
}

impl AudioService {
    pub fn new(backend: Box<dyn AudioBackend>, sys: &SystemState) -> Self {
        Self {
            backend,
            last_vol_media: Some(sys.vol_media),
            last_ringer: Some(sys.ringer_mode),
        }
    }

    pub fn sync_from_state(&mut self, sys: &SystemState) {
        if self
            .last_vol_media
            .is_none_or(|v| (v - sys.vol_media).abs() >= 0.02)
        {
            self.last_vol_media = Some(sys.vol_media);
            self.backend.set_media_volume(sys.vol_media);
        }
        if self.last_ringer != Some(sys.ringer_mode) {
            self.last_ringer = Some(sys.ringer_mode);
            self.backend.set_muted(sys.ringer_mode == RingerMode::Silent);
        }
    }
}
