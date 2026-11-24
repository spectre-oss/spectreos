//! Persistent low-delay H.264 encoder. Raw frames are replaceable; encoded
//! reference frames are never dropped. Encoding never runs on the compositor.
use super::capture::{self, Pixels};
use openh264::{
    OpenH264API, Timestamp,
    encoder::{
        BitRate, Complexity, Encoder, EncoderConfig, FrameRate, FrameType, IntraFramePeriod,
        RateControlMode, UsageType,
    },
    formats::{RgbaSliceU8, YUVBuffer},
};
use serde_json::Value;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, Receiver},
};
use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Codec {
    H264,
    Png,
}
impl Codec {
    pub fn parse(value: Option<&str>) -> Result<Self, &'static str> {
        match value.unwrap_or("png") {
            "h264" => Ok(Self::H264),
            "png" => Ok(Self::Png),
            _ => Err("supported codecs: h264, png"),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::H264 => "h264",
            Self::Png => "png",
        }
    }
}
pub struct Packet {
    pub metadata: Value,
    pub bytes: Vec<u8>,
}
struct Job {
    pixels: Pixels,
    metadata: Value,
    timestamp_us: u64,
}
struct Slot {
    job: Mutex<Option<Job>>,
    ready: Condvar,
    stop: AtomicBool,
    force: AtomicBool,
    dropped: AtomicU64,
}
pub struct VideoWorker {
    slot: Arc<Slot>,
    pub output: Option<Receiver<Result<Packet, String>>>,
    thread: Option<thread::JoinHandle<()>>,
}
static EPOCH: AtomicU64 = AtomicU64::new(1);
impl VideoWorker {
    pub fn new(codec: Codec, fps: u8) -> Self {
        let slot = Arc::new(Slot {
            job: Mutex::new(None),
            ready: Condvar::new(),
            stop: AtomicBool::new(false),
            force: AtomicBool::new(true),
            dropped: AtomicU64::new(0),
        });
        let (tx, rx) = mpsc::sync_channel(1);
        let inner = slot.clone();
        let epoch = EPOCH.fetch_add(1, Ordering::Relaxed);
        let thread = thread::spawn(move || {
            let mut encoder: Option<Encoder> = None;
            let mut sequence = 0u64;
            let mut last_idr = Instant::now();
            loop {
                let job = {
                    let mut job = inner.job.lock().unwrap();
                    while job.is_none() && !inner.stop.load(Ordering::SeqCst) {
                        job = inner.ready.wait(job).unwrap();
                    }
                    if inner.stop.load(Ordering::SeqCst) {
                        break;
                    }
                    job.take().unwrap()
                };
                let result = (|| -> Result<Packet, String> {
                    let mut metadata = job.metadata;
                    let (pixels, bytes, keyframe) = if codec == Codec::H264 {
                        let pixels = scaled(job.pixels);
                        if encoder.is_none() {
                            let config = EncoderConfig::new()
                                .bitrate(BitRate::from_bps(3_000_000))
                                .max_frame_rate(FrameRate::from_hz(fps as f32))
                                .usage_type(UsageType::ScreenContentRealTime)
                                .complexity(Complexity::Low)
                                .num_threads(2)
                                .skip_frames(true)
                                .rate_control_mode(RateControlMode::Bitrate)
                                .adaptive_quantization(false)
                                .background_detection(false)
                                .intra_frame_period(IntraFramePeriod::from_num_frames(
                                    (fps as u32 * 2).max(1),
                                ));
                            encoder = Some(
                                Encoder::with_api_config(OpenH264API::from_source(), config)
                                    .map_err(|e| e.to_string())?,
                            );
                        }
                        let encoder = encoder.as_mut().unwrap();
                        if inner.force.swap(false, Ordering::SeqCst)
                            || last_idr.elapsed() >= Duration::from_secs(2)
                        {
                            encoder.force_intra_frame();
                        }
                        let source = RgbaSliceU8::new(
                            &pixels.rgba,
                            (pixels.width as usize, pixels.height as usize),
                        );
                        let yuv = YUVBuffer::from_rgba8_source(source);
                        let encoded = encoder
                            .encode_at(&yuv, Timestamp::from_millis(job.timestamp_us / 1000))
                            .map_err(|e| e.to_string())?;
                        let keyframe = matches!(encoded.frame_type(), FrameType::IDR);
                        let bytes = encoded.to_vec();
                        if keyframe {
                            last_idr = Instant::now();
                        }
                        (pixels, bytes, keyframe)
                    } else {
                        let w = job.pixels.width;
                        let h = job.pixels.height;
                        let bytes = capture::encode(job.pixels)?;
                        (
                            Pixels {
                                width: w,
                                height: h,
                                rgba: Vec::new(),
                            },
                            bytes,
                            true,
                        )
                    };
                    if bytes.is_empty() {
                        return Ok(Packet { metadata, bytes });
                    }
                    sequence += 1;
                    metadata["id"] = 0.into();
                    metadata["event"] = "frame".into();
                    metadata["codec"] = codec.name().into();
                    metadata["stream_id"] = epoch.into();
                    metadata["sequence"] = sequence.into();
                    metadata["timestamp_us"] = job.timestamp_us.into();
                    metadata["keyframe"] = keyframe.into();
                    metadata["source_width"] = metadata["width"].clone();
                    metadata["source_height"] = metadata["height"].clone();
                    metadata["width"] = pixels.width.into();
                    metadata["height"] = pixels.height.into();
                    metadata["dropped_raw_frames"] = inner.dropped.load(Ordering::Relaxed).into();
                    Ok(Packet { metadata, bytes })
                })();
                if matches!(&result, Ok(packet) if packet.bytes.is_empty()) {
                    continue;
                }
                let failed = result.is_err();
                if tx.send(result).is_err() || failed {
                    break;
                }
            }
        });
        Self {
            slot,
            output: Some(rx),
            thread: Some(thread),
        }
    }
    pub fn submit(&self, pixels: Pixels, metadata: Value, timestamp_us: u64) {
        let mut slot = self.slot.job.lock().unwrap();
        if slot
            .replace(Job {
                pixels,
                metadata,
                timestamp_us,
            })
            .is_some()
        {
            self.slot.dropped.fetch_add(1, Ordering::Relaxed);
        }
        self.slot.ready.notify_one();
    }
    pub fn keyframe(&self) {
        self.slot.force.store(true, Ordering::SeqCst);
    }
}
impl Drop for VideoWorker {
    fn drop(&mut self) {
        self.slot.stop.store(true, Ordering::SeqCst);
        self.slot.ready.notify_all();
        self.output.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn scaled(source: Pixels) -> Pixels {
    let factor = (720.0 / source.width as f64)
        .min(1600.0 / source.height as f64)
        .min(1.0);
    let w = ((source.width as f64 * factor) as u32 / 2 * 2).max(2);
    let h = ((source.height as f64 * factor) as u32 / 2 * 2).max(2);
    if [w, h] == [source.width, source.height] {
        return source;
    }
    let mut rgba = vec![0; w as usize * h as usize * 4];
    for y in 0..h {
        let sy = y as u64 * source.height as u64 / h as u64;
        for x in 0..w {
            let sx = x as u64 * source.width as u64 / w as u64;
            let from = (sy * source.width as u64 + sx) as usize * 4;
            let to = (y * w + x) as usize * 4;
            rgba[to..to + 4].copy_from_slice(&source.rgba[from..from + 4]);
        }
    }
    Pixels {
        width: w,
        height: h,
        rgba,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn h264_roundtrip_and_forced_keyframe_have_parameter_sets() {
        use openh264::{decoder::Decoder, formats::YUVSource};
        let worker = VideoWorker::new(Codec::H264, 15);
        let mut decoder = Decoder::new().unwrap();
        let frame = || Pixels {
            width: 64,
            height: 64,
            rgba: [40, 110, 190, 255].repeat(64 * 64),
        };
        worker.submit(frame(), json!({"ok":true,"width":64,"height":64}), 0);
        let first = worker
            .output
            .as_ref()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap();
        assert_eq!(first.metadata["keyframe"], true);
        assert!(
            first
                .bytes
                .windows(5)
                .any(|b| b[..4] == [0, 0, 0, 1] && b[4] & 31 == 7)
        );
        assert_eq!(
            decoder.decode(&first.bytes).unwrap().unwrap().dimensions(),
            (64, 64)
        );
        worker.submit(frame(), json!({"ok":true}), 100_000);
        let _ = worker
            .output
            .as_ref()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap();
        worker.keyframe();
        worker.submit(frame(), json!({"ok":true}), 200_000);
        let next = worker
            .output
            .as_ref()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap();
        assert_eq!(next.metadata["keyframe"], true);
        assert!(
            Decoder::new()
                .unwrap()
                .decode(&next.bytes)
                .unwrap()
                .is_some()
        );
    }
}
