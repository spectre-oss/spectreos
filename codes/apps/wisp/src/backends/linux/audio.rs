//! Linux PipeWire / WirePlumber (`wpctl`) audio control backend.

use super::{find_in_path, spawn_detached};
use crate::services::audio::AudioBackend;

pub struct PipewireAudioBackend;

impl AudioBackend for PipewireAudioBackend {
    fn set_media_volume(&mut self, norm: f32) {
        if let Some(wpctl) = find_in_path("wpctl") {
            let pct = format!("{:.2}", norm.clamp(0.0, 1.0));
            spawn_detached(&wpctl, &["set-volume", "@DEFAULT_AUDIO_SINK@", &pct]);
        }
    }

    fn set_muted(&mut self, muted: bool) {
        if let Some(wpctl) = find_in_path("wpctl") {
            spawn_detached(
                &wpctl,
                &["set-mute", "@DEFAULT_AUDIO_SINK@", if muted { "1" } else { "0" }],
            );
        }
    }
}
