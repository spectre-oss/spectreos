//! Material loading morphs: pinned AndroidX geometry, independent Rust sampling.
//! Copyright 2024 The Android Open Source Project (shape recipes and tokens),
//! Apache-2.0. See THIRD_PARTY_NOTICES.md and tools/BhooShapes.java.
use crate::*;
use crate::shapes::loading_shapes as shapes;

#[derive(Clone, Debug, Default)]
pub struct LoadingState {
    elapsed: f64,
    last: Option<f64>,
    generation: u64,
}
impl LoadingState {
    /// Call when presentation becomes hidden. Resume does not replay hidden time.
    pub fn pause(&mut self) {
        self.last = None;
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    fn advance(&mut self, time: f64) {
        if let Some(last) = self.last {
            self.elapsed += (time - last).max(0.0);
        }
        self.last = Some(time);
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoadingSample {
    pub shape: u8,
    pub progress: f32,
    pub rotation: f32,
}
/// 650ms morph cadence, .6/200 spring, and 4666ms global rotation from AndroidX.
/// The spring's 297ms duration is exported from FloatSpringSpec with threshold .1.
pub fn loading_sample(elapsed: f64) -> LoadingSample {
    let elapsed = if elapsed.is_finite() {
        elapsed.max(0.0)
    } else {
        0.0
    };
    let cycle = (elapsed / 0.650).floor() as u64;
    let t = elapsed % 0.650;
    let (shape, progress, quarter) = if t >= shapes::MORPH_SECONDS {
        (((cycle + 1) % 7) as u8, 0.0, cycle + 2)
    } else {
        ((cycle % 7) as u8, spring_progress(t), cycle + 1)
    };
    let rotation =
        (quarter % 4) as f32 * 90.0 + progress * 90.0 + ((elapsed % 4.666) / 4.666 * 360.0) as f32;
    LoadingSample {
        shape,
        progress,
        rotation,
    }
}
fn spring_progress(time: f64) -> f32 {
    let omega = 200.0_f64.sqrt();
    let decay = 0.6 * omega;
    let w = omega * 0.8;
    (1.0 - (-decay * time).exp() * ((w * time).cos() + 0.75 * (w * time).sin())) as f32
}
fn lerp(pair: &[[f32; 8]; 2], p: f32) -> [f32; 8] {
    std::array::from_fn(|i| pair[0][i] + (pair[1][i] - pair[0][i]) * p)
}
fn at(c: &[f32; 8], t: f32, axis: usize) -> f32 {
    let u = 1.0 - t;
    u * u * u * c[axis]
        + 3.0 * u * u * t * c[axis + 2]
        + 3.0 * u * t * t * c[axis + 4]
        + t * t * t * c[axis + 6]
}
/// Exact cubic extrema, so morph centering matches the upstream path bounds.
fn extrema(c: &[f32; 8], axis: usize) -> [f32; 2] {
    let mut min = c[axis].min(c[axis + 6]);
    let mut max = c[axis].max(c[axis + 6]);
    let (p0, p1, p2, p3) = (c[axis], c[axis + 2], c[axis + 4], c[axis + 6]);
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 2.0 * (p0 - 2.0 * p1 + p2);
    let d = p1 - p0;
    let roots = if a.abs() < 1e-7 {
        if b.abs() > 1e-7 {
            [-d / b, -1.0]
        } else {
            [-1.0; 2]
        }
    } else {
        let disc = b * b - 4.0 * a * d;
        if disc >= 0.0 {
            [
                (-b + disc.sqrt()) / (2.0 * a),
                (-b - disc.sqrt()) / (2.0 * a),
            ]
        } else {
            [-1.0; 2]
        }
    };
    for t in roots {
        if t > 0.0 && t < 1.0 {
            let v = at(c, t, axis);
            min = min.min(v);
            max = max.max(v);
        }
    }
    [min, max]
}
pub fn loading_curve_count(shape: u8) -> usize {
    shapes::SHAPES[shape.min(7) as usize].len()
}
/// Streams transformed cubic points without allocating a path or morph each frame.
pub fn loading_cubics(bounds: Rect, sample: LoadingSample, mut emit: impl FnMut([f32; 8])) {
    let pairs = shapes::SHAPES[sample.shape.min(7) as usize];
    let (mut min, mut max) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    for pair in pairs {
        let c = lerp(pair, sample.progress);
        for axis in 0..2 {
            let e = extrema(&c, axis);
            min[axis] = min[axis].min(e[0]);
            max[axis] = max[axis].max(e[1]);
        }
    }
    let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let scale = bounds.width.min(bounds.height) * shapes::SCALE * 38.0 / 48.0;
    let angle = sample.rotation.to_radians();
    let (sin, cos) = angle.sin_cos();
    for pair in pairs {
        let c = lerp(pair, sample.progress);
        let out = std::array::from_fn(|i| {
            let j = i / 2 * 2;
            let x = (c[j] - center[0]) * scale;
            let y = (c[j + 1] - center[1]) * scale;
            if i % 2 == 0 {
                bounds.x + bounds.width * 0.5 + x * cos - y * sin
            } else {
                bounds.y + bounds.height * 0.5 + x * sin + y * cos
            }
        });
        emit(out);
    }
}
impl Frame<'_> {
    pub fn loading_indicator(
        &mut self,
        id: u64,
        label: &str,
        state: &mut LoadingState,
        progress: Option<f32>,
        contained: bool,
    ) {
        if state.generation != self.ui.pause_generation {
            state.pause();
            state.generation = self.ui.pause_generation;
        }
        let mut b = self.allocate(48.0);
        b.width = b.width.min(48.0);
        let visible = b.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            state.pause();
            return;
        }
        let value = progress
            .filter(|p| p.is_finite())
            .map(|p| p.clamp(0.0, 1.0));
        let sample = if let Some(p) = value {
            let p = self.animate(id, 30, p, p);
            LoadingSample {
                shape: 7,
                progress: p,
                rotation: p * 180.0,
            }
        } else if self.ui.motion_preference == MotionPreference::Reduced {
            state.pause();
            LoadingSample {
                shape: 0,
                progress: 0.0,
                rotation: 90.0,
            }
        } else {
            state.advance(self.ui.time);
            self.ui.continuous = true;
            loading_sample(state.elapsed)
        };
        let t = self.ui.theme;
        if contained {
            self.rect(b, 24.0, t.primary_container);
        }
        self.commands.push(DrawCommand::Loading {
            bounds: b,
            sample,
            color: if contained {
                t.on_primary_container
            } else {
                t.primary
            },
        });
        self.semantic(
            Response {
                id,
                bounds: b,
                enabled: self.enabled,
                ..Response::default()
            },
            Role::Progress,
            label,
            false,
            None,
            value,
            value.map(|_| [0.0, 1.0]),
        );
    }
}
#[cfg(test)]
#[path = "loading.spec.rs"]
mod tests;
