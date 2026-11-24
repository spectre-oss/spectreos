//! Stateful H.264 decoding and explicit reference-chain recovery.
use crate::{
    protocol,
    video::{self, VideoFrame},
};
use openh264::{decoder::Decoder, formats::YUVSource};
use serde_json::{Value, json};

pub struct FrameDecoder {
    decoder: Option<Decoder>,
    epoch: Option<u64>,
    sequence: Option<u64>,
    pub needs_keyframe: bool,
}
impl Default for FrameDecoder {
    fn default() -> Self {
        Self {
            decoder: None,
            epoch: None,
            sequence: None,
            needs_keyframe: true,
        }
    }
}
impl FrameDecoder {
    pub fn decode(
        &mut self,
        metadata: &Value,
        bytes: Vec<u8>,
    ) -> Result<Option<VideoFrame>, String> {
        if metadata["codec"].as_str().unwrap_or("png") == "png" {
            return video::decode(bytes).map(Some);
        }
        if metadata["codec"] != "h264" {
            return Err("unsupported video codec".into());
        }
        let width = metadata["width"].as_u64().ok_or("missing encoded width")?;
        let height = metadata["height"]
            .as_u64()
            .ok_or("missing encoded height")?;
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || width * height > protocol::MAX_BINARY as u64 / 4
        {
            return Err("H.264 dimensions exceed limits".into());
        }
        let epoch = metadata["stream_id"].as_u64().ok_or("missing stream id")?;
        let sequence = metadata["sequence"]
            .as_u64()
            .ok_or("missing frame sequence")?;
        if self.epoch != Some(epoch)
            || self
                .sequence
                .is_some_and(|previous| sequence != previous + 1)
        {
            self.decoder = None;
            self.needs_keyframe = true;
        }
        self.epoch = Some(epoch);
        self.sequence = Some(sequence);
        if self.needs_keyframe && metadata["keyframe"] != true {
            return Ok(None);
        }
        if self.decoder.is_none() {
            self.decoder = Some(Decoder::new().map_err(|e| e.to_string())?);
        }
        let decoded = match self.decoder.as_mut().unwrap().decode(&bytes) {
            Ok(Some(decoded)) => decoded,
            Ok(None) => return Ok(None),
            Err(_) => {
                self.decoder = None;
                self.needs_keyframe = true;
                return Ok(None);
            }
        };
        let (w, h) = decoded.dimensions();
        if w != width as usize || h != height as usize {
            self.needs_keyframe = true;
            return Err("decoded dimensions do not match the access-unit header".into());
        }
        let mut rgb = vec![0; w * h * 3];
        decoded.write_rgb8(&mut rgb);
        let pixels = rgb
            .chunks_exact(3)
            .map(|p| ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | p[2] as u32)
            .collect();
        self.needs_keyframe = false;
        Ok(Some(VideoFrame {
            width: w as u32,
            height: h as u32,
            pixels,
            png: Vec::new(),
        }))
    }
}

/// Standard-I/O bridge for the existing Python CLI. Wire input remains H.264;
/// only the host's decoded-frame cache is PNG for `frame` and legacy viewers.
pub fn decode_pipe() -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();
    let mut decoder = FrameDecoder::default();
    loop {
        let (mut metadata, bytes) = match protocol::read_frame(&mut input) {
            Ok(frame) => frame,
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(error) => return Err(error.to_string()),
        };
        match decoder.decode(&metadata, bytes)? {
            Some(frame) => {
                let bytes = frame.encoded_png()?;
                metadata["wire_codec"] = metadata["codec"].clone();
                metadata["codec"] = "png".into();
                protocol::write_packet(&mut output, &metadata, &bytes)
                    .map_err(|e| e.to_string())?;
            }
            None if decoder.needs_keyframe => {
                protocol::write_packet(&mut output, &json!({"event":"need_keyframe"}), &[])
                    .map_err(|e| e.to_string())?
            }
            None => {}
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use openh264::{
        encoder::Encoder,
        formats::{RgbaSliceU8, YUVBuffer},
    };
    #[test]
    fn sequence_gaps_wait_for_idr_and_recover() {
        let rgba = [30, 80, 170, 255].repeat(64 * 64);
        let yuv = YUVBuffer::from_rgba8_source(RgbaSliceU8::new(&rgba, (64, 64)));
        let mut encoder = Encoder::new().unwrap();
        let mut decoder = FrameDecoder::default();
        let meta = |sequence, keyframe| json!({"codec":"h264","width":64,"height":64,"stream_id":1,"sequence":sequence,"keyframe":keyframe});
        let first = encoder.encode(&yuv).unwrap().to_vec();
        assert!(decoder.decode(&meta(1, true), first).unwrap().is_some());
        let delta = encoder.encode(&yuv).unwrap().to_vec();
        assert!(decoder.decode(&meta(3, false), delta).unwrap().is_none());
        assert!(decoder.needs_keyframe);
        encoder.force_intra_frame();
        let idr = encoder.encode(&yuv).unwrap().to_vec();
        assert!(decoder.decode(&meta(4, true), idr).unwrap().is_some());
        assert!(!decoder.needs_keyframe);
    }
}
