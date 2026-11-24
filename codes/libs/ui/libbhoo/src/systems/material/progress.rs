//! Pinned Compose Material 3 progress timelines (Apache-2.0, Copyright 2022 The Android Open Source Project).
//! Independent elapsed-time evaluation; no per-indicator timer or texture.
use crate::timing::Easing;
use crate::*;

/// Material 3 Expressive wavy progress configuration (pinned to MDC-Android `BaseProgressIndicatorSpec`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WavyProgressOptions {
    pub wavelength: f32,
    pub amplitude: f32,
    pub wave_speed: f32,
    pub track_thickness: f32,
    pub stop_indicator_size: f32,
    pub track_gap: f32,
}
impl WavyProgressOptions {
    /// Pinned M3 Expressive linear wavy defaults (`wavelength: 40dp`, `amplitude: 3dp`).
    pub const fn linear() -> Self {
        Self {
            wavelength: 40.0,
            amplitude: 3.0,
            wave_speed: 40.0,
            track_thickness: 4.0,
            stop_indicator_size: 4.0,
            track_gap: 4.0,
        }
    }
    /// Pinned M3 Expressive circular wavy defaults (`wavelength: 16dp`, `amplitude: 3dp`).
    pub const fn circular() -> Self {
        Self {
            wavelength: 16.0,
            amplitude: 3.0,
            wave_speed: 40.0,
            track_thickness: 4.0,
            stop_indicator_size: 4.0,
            track_gap: 4.0,
        }
    }
}
impl Default for WavyProgressOptions {
    fn default() -> Self {
        Self::linear()
    }
}

/// Smooth M3 amplitude ramp in `[0, 1]`, tapering to `0` near `0%` and `100%`
/// (`DeterminateDrawable.FULL_AMPLITUDE_PROGRESS_MIN = 0.1`, `MAX = 0.9`).
pub fn wavy_amplitude_ramp(progress: f32) -> f32 {
    if !progress.is_finite() {
        return 0.0;
    }
    let p = progress.clamp(0.0, 1.0);
    const MIN_RAMP: f32 = 0.1;
    const MAX_RAMP: f32 = 0.9;
    let edge = if p < MIN_RAMP {
        p / MIN_RAMP
    } else if p > MAX_RAMP {
        (1.0 - p) / (1.0 - MAX_RAMP)
    } else {
        1.0
    };
    let t = edge.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn sanitize_wavy(options: WavyProgressOptions, default_wavelength: f32) -> WavyProgressOptions {
    let valid_pos = |v: f32, fallback: f32| if v.is_finite() && v > 0.0 { v } else { fallback };
    let valid_nonneg = |v: f32, fallback: f32| if v.is_finite() && v >= 0.0 { v } else { fallback };
    WavyProgressOptions {
        wavelength: valid_pos(options.wavelength, default_wavelength),
        amplitude: valid_nonneg(options.amplitude, 3.0),
        wave_speed: if options.wave_speed.is_finite() { options.wave_speed } else { 40.0 },
        track_thickness: valid_pos(options.track_thickness, 4.0),
        stop_indicator_size: valid_nonneg(options.stop_indicator_size, 4.0),
        track_gap: valid_nonneg(options.track_gap, 4.0),
    }
}
#[derive(Clone, Copy)]
pub(crate) struct ClassicProgressColors {
    pub primary: Color,
    pub background: Color,
    pub secondary: Color,
    pub indeterminate: Color,
}
impl ClassicProgressColors {
    pub(crate) fn from_theme(theme: Theme) -> Self {
        Self { primary: theme.primary, background: theme.surface_variant, secondary: theme.primary.opacity(0.38), indeterminate: theme.primary }
    }
}
pub fn linear_segments(elapsed: f64) -> [[f32; 2]; 2] {
    let ms = (elapsed.max(0.0) % 1.750) * 1000.0;
    let eased = |delay: f64, duration: f64| {
        Easing::EmphasizedAccelerate.sample(((ms - delay) / duration).clamp(0.0, 1.0) as f32)
    };
    [
        [eased(250.0, 1000.0), eased(0.0, 1000.0)],
        [eased(900.0, 850.0), eased(650.0, 850.0)],
    ]
}
pub fn circular_sample(elapsed: f64) -> (f32, f32) {
    let t = elapsed.max(0.0) % 6.0;
    let quadrant = (t / 1.5).floor();
    let local = t % 1.5;
    let extra = quadrant as f32 * 90.0
        + 90.0 * Easing::EmphasizedDecelerate.sample((local / 0.3).min(1.0) as f32);
    let p = if t < 3.0 {
        0.1 + 0.77 * Easing::Standard.sample((t / 3.0) as f32)
    } else {
        0.87 - 0.77 * Easing::Standard.sample(((t - 3.0) / 3.0) as f32)
    };
    (
        ((t / 6.0 * 1080.0) as f32 + extra).to_radians(),
        p * std::f32::consts::TAU,
    )
}
impl Frame<'_> {
    pub fn progress_circular(&mut self, id: u64, label: &str, value: Option<f32>) {
        let mut b = self.allocate(40.0);
        b.width = b.width.min(40.0);
        let t = self.ui.theme;
        let value = value.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0));
        let visible = b.intersect(self.clip);
        let moving = value.is_none()
            && visible.width > 0.0
            && visible.height > 0.0
            && !self.reduced_motion();
        let (start, sweep) = if let Some(value) = value {
            let v = self.ui.motion.effect(id, 3, value, value, Spring::PROGRESS);
            (-std::f32::consts::FRAC_PI_2, v * std::f32::consts::TAU)
        } else if moving {
            self.ui.continuous = true;
            circular_sample(self.ui.motion.elapsed(id, 31))
        } else {
            (-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2)
        };
        let gap =
            (8.0 / (std::f32::consts::PI * b.width.max(1.0)) * std::f32::consts::TAU).min(sweep);
        self.commands.push(DrawCommand::Arc {
            bounds: b,
            start: start + sweep + gap,
            sweep: (std::f32::consts::TAU - sweep - 2.0 * gap).max(0.0),
            stroke: 4.0,
            color: t.control,
            rounded: true,
        });
        self.commands.push(DrawCommand::Arc {
            bounds: b,
            start,
            sweep,
            stroke: 4.0,
            color: t.primary,
            rounded: true,
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
    /// Material 3 Expressive wavy circular progress indicator (`48dp` default container).
    /// Draws a smooth sinusoidal active wave separated by `track_gap` from the straight inactive arc.
    pub fn wavy_circular_progress(
        &mut self,
        id: u64,
        label: &str,
        value: Option<f32>,
        options: WavyProgressOptions,
    ) {
        let opt = sanitize_wavy(options, 16.0);
        let size = 48.0_f32.max(opt.track_thickness + opt.amplitude * 2.0 + 16.0);
        let mut b = self.allocate(size);
        b.width = b.width.min(size);
        let t = self.ui.theme;
        let value = value.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0));
        let visible = b.intersect(self.clip);
        let is_visible = visible.width > 0.0 && visible.height > 0.0;
        let reduced = self.reduced_motion();
        let (start, sweep, amp_factor, gap_scale) = if let Some(v) = value {
            let a = self.ui.motion.effect(id, 3, v, v, Spring::PROGRESS);
            (
                -std::f32::consts::FRAC_PI_2,
                a * std::f32::consts::TAU,
                wavy_amplitude_ramp(a),
                (a / 0.01).clamp(0.0, 1.0),
            )
        } else if is_visible && !reduced {
            self.ui.continuous = true;
            let (s, sw) = circular_sample(self.ui.motion.elapsed(id, 31));
            (s, sw, 1.0, 1.0)
        } else {
            (
                -std::f32::consts::FRAC_PI_2,
                std::f32::consts::FRAC_PI_2,
                1.0,
                1.0,
            )
        };
        let cx = b.x + b.width * 0.5;
        let cy = b.y + b.height * 0.5;
        let half_t = opt.track_thickness * 0.5;
        let base_r = (b.width.min(b.height) * 0.5 - half_t - opt.amplitude).max(4.0);
        let circumference = std::f32::consts::TAU * base_r;
        let cycles = (circumference / opt.wavelength).round().max(3.0);
        let adjusted_wavelength = circumference / cycles;
        let effective_amp = opt.amplitude * amp_factor;
        let phase = if is_visible && !reduced && effective_amp > 0.001 && opt.wave_speed.abs() > 0.001
        {
            self.ui.continuous = true;
            let elapsed = self.ui.motion.elapsed(id, 32) as f32;
            (elapsed * opt.wave_speed / adjusted_wavelength) * std::f32::consts::TAU
        } else {
            0.0
        };
        let gap_rad = (((opt.track_gap + opt.track_thickness) / base_r) * gap_scale).min(sweep);
        let arc_side = 2.0 * base_r + opt.track_thickness;
        let arc_bounds = Rect {
            x: cx - arc_side * 0.5,
            y: cy - arc_side * 0.5,
            width: arc_side,
            height: arc_side,
        };
        let inactive_sweep = (std::f32::consts::TAU - sweep - 2.0 * gap_rad).max(0.0);
        if inactive_sweep > 0.001 {
            self.commands.push(DrawCommand::Arc {
                bounds: arc_bounds,
                start: start + sweep + gap_rad,
                sweep: inactive_sweep,
                stroke: opt.track_thickness,
                color: t.secondary_container,
                rounded: true,
            });
        }
        if sweep > 0.001 {
            if effective_amp <= 0.05 {
                self.commands.push(DrawCommand::Arc {
                    bounds: arc_bounds,
                    start,
                    sweep,
                    stroke: opt.track_thickness,
                    color: t.primary,
                    rounded: true,
                });
            } else {
                let arc_len = sweep * base_r;
                let steps = ((arc_len / 2.0).ceil() as usize)
                    .max((cycles * (sweep / std::f32::consts::TAU) * 16.0).ceil() as usize)
                    .clamp(12, 240);
                let mut prev = [0.0_f32; 2];
                for i in 0..=steps {
                    let frac = i as f32 / steps as f32;
                    let theta = start + sweep * frac;
                    let r = base_r + effective_amp * (cycles * theta - phase).sin();
                    let curr = [cx + r * theta.cos(), cy + r * theta.sin()];
                    if i > 0 {
                        self.paint_line(prev, curr, opt.track_thickness, t.primary);
                    }
                    prev = curr;
                }
            }
        }
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
/// MDC's classic two-second CSS timeline, retained as a distinct progress variant.
/// Values are from Angular Material 22.2.6 / Google LLC (MIT).
pub fn classic_linear_segments(elapsed: f64) -> [[f32; 2]; 2] {
    let t = (elapsed.max(0.0) % 2.0) as f32 / 2.0;
    let segment = |a: f32, b: f32, from: f32, to: f32, curve: [f32; 4]| {
        from + (to - from) * Easing::Cubic(curve).sample(((t - a) / (b - a)).clamp(0.0, 1.0))
    };
    let primary = if t < 0.2 {
        0.0
    } else if t < 0.5915 {
        segment(
            0.2,
            0.5915,
            0.0,
            0.836_714_2,
            [0.5, 0.0, 0.701_732, 0.495_819],
        )
    } else {
        segment(
            0.5915,
            1.0,
            0.836_714_2,
            2.006_110_7,
            [0.302_435, 0.381_352, 0.55, 0.956_352],
        )
    } - 1.451_666_1;
    let pscale = if t < 0.3665 {
        0.08
    } else if t < 0.6915 {
        segment(
            0.3665,
            0.6915,
            0.08,
            0.661_479,
            [0.334_731, 0.124_82, 0.785_844, 1.0],
        )
    } else {
        segment(0.6915, 1.0, 0.661_479, 0.08, [0.06, 0.11, 0.6, 1.0])
    };
    let secondary = if t < 0.25 {
        segment(
            0.0,
            0.25,
            0.0,
            0.376_519_14,
            [0.15, 0.0, 0.515_058, 0.409_685],
        )
    } else if t < 0.4835 {
        segment(
            0.25,
            0.4835,
            0.376_519_14,
            0.843_861_64,
            [0.310_33, 0.284_058, 0.8, 0.733_712],
        )
    } else {
        segment(
            0.4835,
            1.0,
            0.843_861_64,
            1.602_777_8,
            [0.4, 0.627_035, 0.6, 0.902_026],
        )
    } - 0.548_888_9;
    let sscale = if t < 0.1915 {
        segment(
            0.0,
            0.1915,
            0.08,
            0.457_104,
            [0.205_028, 0.057_051, 0.576_61, 0.453_971],
        )
    } else if t < 0.4415 {
        segment(
            0.1915,
            0.4415,
            0.457_104,
            0.727_96,
            [0.152_313, 0.196_432, 0.648_374, 1.004_315],
        )
    } else {
        segment(
            0.4415,
            1.0,
            0.727_96,
            0.08,
            [0.257_759, -0.003_163, 0.211_762, 1.381_79],
        )
    };
    [
        [primary.clamp(0.0, 1.0), (primary + pscale).clamp(0.0, 1.0)],
        [
            secondary.clamp(0.0, 1.0),
            (secondary + sscale).clamp(0.0, 1.0),
        ],
    ]
}
/// Exact composite angles of MDC's rotating layer and clipped half-circle keyframes.
pub fn classic_circular_sample(elapsed: f64) -> (f32, f32) {
    let t = elapsed.max(0.0);
    let phase = (t % 1.333) / 1.333;
    let local = if phase < 0.5 {
        phase * 2.0
    } else {
        (1.0 - phase) * 2.0
    } as f32;
    let curve = Easing::Cubic([0.4, 0.0, 0.2, 1.0]);
    let left = 265.0 - 135.0 * curve.sample(local);
    let layer = t / 0.6665;
    let angle = 135.0 * (layer.floor() as f32 + curve.sample(layer.fract() as f32))
        + (t / 1.568_235_294_117_6 * 360.0) as f32;
    (
        (angle + left).to_radians(),
        (540.0 - 2.0 * left).to_radians(),
    )
}
impl Frame<'_> {
    pub fn progress_classic(&mut self, id: u64, label: &str, value: Option<f32>) {
        self.progress_classic_with_secondary(id, label, value, None);
    }
    /// Linear progress with an optional buffered portion behind the primary bar.
    pub fn progress_classic_with_secondary(&mut self, id: u64, label: &str, value: Option<f32>, secondary: Option<f32>) {
        self.progress_classic_directional(id, label, value, secondary, false);
    }
    /// Linear progress with optional right-to-left mirroring.
    pub fn progress_classic_directional(&mut self, id: u64, label: &str, value: Option<f32>, secondary: Option<f32>, rtl: bool) {
        self.progress_classic_tinted(id, label, value, secondary, rtl, ClassicProgressColors::from_theme(self.ui.theme));
    }
    pub(crate) fn progress_classic_tinted(&mut self, id: u64, label: &str, value: Option<f32>, secondary: Option<f32>, rtl: bool, colors: ClassicProgressColors) {
        use crate::timing::TweenSpec;
        let b = self.allocate(4.0);
        let value = value.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0));
        let segments = if let Some(v) = value {
            let v = self.ui.motion.tween(
                id,
                3,
                v,
                v,
                TweenSpec::new(0.250, Easing::Cubic([0.4, 0.0, 0.6, 1.0])),
                TweenSpec::new(0.250, Easing::Cubic([0.4, 0.0, 0.6, 1.0])),
            );
            [[0.0, v], [0.0, 0.0]]
        } else if self.continuous(b) {
            classic_linear_segments(self.ui.motion.elapsed(id, 31))
        } else {
            [[0.0, 0.4], [0.0, 0.0]]
        };
        self.rect(b, 0.0, colors.background);
        if value.is_some() && let Some(buffer) = secondary.filter(|v| v.is_finite()) {
            let width = b.width * buffer.clamp(0.0, 1.0);
            self.rect(Rect { x: if rtl { b.x + b.width - width } else { b.x }, width, ..b }, 0.0, colors.secondary);
        }
        for [tail, head] in segments {
            if head > tail {
                self.rect(
                    Rect {
                        x: b.x + b.width * if rtl { 1.0 - head } else { tail },
                        width: b.width * (head - tail),
                        ..b
                    },
                    0.0,
                    if value.is_some() { colors.primary } else { colors.indeterminate },
                );
            }
        }
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
    pub fn progress_circular_classic(
        &mut self,
        id: u64,
        label: &str,
        value: Option<f32>,
        diameter: f32,
    ) {
        self.progress_circular_classic_tinted(id, label, value, diameter, ClassicProgressColors::from_theme(self.ui.theme));
    }
    pub(crate) fn progress_circular_classic_tinted(&mut self, id: u64, label: &str, value: Option<f32>, diameter: f32, colors: ClassicProgressColors) {
        use crate::timing::TweenSpec;
        let mut b = self.allocate(diameter);
        b.width = b.width.min(diameter);
        let value = value.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0));
        let (start, sweep) = if let Some(v) = value {
            let a = self.ui.motion.tween(
                id,
                3,
                v,
                v,
                TweenSpec::new(0.5, Easing::StandardDecelerate),
                TweenSpec::new(0.5, Easing::StandardDecelerate),
            );
            (-std::f32::consts::FRAC_PI_2, a * std::f32::consts::TAU)
        } else if self.continuous(b) {
            classic_circular_sample(self.ui.motion.elapsed(id, 31))
        } else {
            (-std::f32::consts::FRAC_PI_2, std::f32::consts::TAU * 0.4)
        };
        let stroke = diameter / 10.0;
        let radius = (diameter - 10.0) * 0.5 * diameter / (diameter - 10.0 + stroke);
        let inset = (diameter - 2.0 * radius - stroke) * 0.5;
        self.commands.push(DrawCommand::Arc {
            bounds: b.inset(inset.max(0.0)),
            start,
            sweep,
            stroke,
            color: if value.is_some() { colors.primary } else { colors.indeterminate },
            rounded: false,
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
#[path = "progress.spec.rs"]
mod tests;

impl Frame<'_> {
    /// The caller owns semantics of the work. None means visible indeterminate progress.
    pub fn progress(&mut self, id: u64, label: &str, value: Option<f32>) {
        let b = self.allocate(12.0);
        let t = self.ui.theme;
        let track = Rect {
            y: b.y + 4.0,
            height: 4.0,
            ..b
        };
        let value = value.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0));
        let segments = if let Some(v) = value {
            let a = self.ui.motion.effect(id, 3, v, v, Spring::PROGRESS);
            [[0.0, a], [0.0, 0.0]]
        } else {
            let visible = self.clip.intersect(b);
            if visible.width > 0.0
                && visible.height > 0.0
                && self.ui.motion_preference == MotionPreference::Full
            {
                self.ui.continuous = true;
                crate::progress::linear_segments(self.ui.motion.elapsed(id, 31))
            } else {
                [[0.0, 0.3], [0.0, 0.0]]
            }
        };
        // Draw only inactive portions, leaving the 4dp gap plus round-cap space.
        let gap = (8.0 / track.width.max(1.0)).min(1.0);
        let mut edges = [segments[0], segments[1]];
        if edges[0][0] > edges[1][0] {
            edges.swap(0, 1);
        }
        let mut cursor = 0.0;
        for [tail, head] in edges {
            if head <= tail {
                continue;
            }
            let end = (tail - gap).max(cursor);
            if end > cursor {
                self.rect(
                    Rect {
                        x: track.x + cursor * track.width,
                        width: (end - cursor) * track.width,
                        ..track
                    },
                    2.0,
                    t.control,
                );
            }
            cursor = cursor.max((head + gap).min(1.0));
        }
        if cursor < 1.0 {
            self.rect(
                Rect {
                    x: track.x + cursor * track.width,
                    width: (1.0 - cursor) * track.width,
                    ..track
                },
                2.0,
                t.control,
            );
        }
        for [tail, head] in segments {
            if head > tail {
                self.rect(
                    Rect {
                        x: track.x + tail * track.width,
                        width: (head - tail) * track.width,
                        ..track
                    },
                    2.0,
                    t.primary,
                );
            }
        }
        if value.is_some_and(|v| v < 1.0) {
            self.rect(
                Rect {
                    x: track.x + track.width - 4.0,
                    width: 4.0,
                    ..track
                },
                2.0,
                t.primary,
            );
        }
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
    /// Material 3 Expressive wavy linear progress indicator.
    /// Draws a smooth sinusoidal active wave separated by `track_gap` from the straight
    /// inactive track and a trailing `stop_indicator_size` dot.
    pub fn wavy_linear_progress(
        &mut self,
        id: u64,
        label: &str,
        value: Option<f32>,
        options: crate::progress::WavyProgressOptions,
    ) {
        let opt = crate::progress::sanitize_wavy(options, 40.0);
        let height = (opt.track_thickness + opt.amplitude * 2.0).max(12.0);
        let b = self.allocate(height);
        let t = self.ui.theme;
        let cy = b.y + b.height * 0.5;
        let half_t = opt.track_thickness * 0.5;
        let value = value.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0));
        let visible = self.clip.intersect(b);
        let is_visible = visible.width > 0.0 && visible.height > 0.0;
        let reduced = self.reduced_motion();
        let (segments, amp_factor, gap_scale) = if let Some(v) = value {
            let a = self.ui.motion.effect(id, 3, v, v, Spring::PROGRESS);
            (
                [[0.0, a], [0.0, 0.0]],
                crate::progress::wavy_amplitude_ramp(a),
                (a / 0.01).clamp(0.0, 1.0),
            )
        } else if is_visible && !reduced {
            self.ui.continuous = true;
            (
                crate::progress::linear_segments(self.ui.motion.elapsed(id, 31)),
                1.0,
                1.0,
            )
        } else {
            ([[0.0, 0.35], [0.0, 0.0]], 1.0, 1.0)
        };
        let effective_amp = opt.amplitude * amp_factor;
        let phase = if is_visible && !reduced && effective_amp > 0.001 && opt.wave_speed.abs() > 0.001
        {
            self.ui.continuous = true;
            let elapsed = self.ui.motion.elapsed(id, 32) as f32;
            (elapsed * opt.wave_speed / opt.wavelength) * std::f32::consts::TAU
        } else {
            0.0
        };
        let gap_frac = ((opt.track_gap * gap_scale) / b.width.max(1.0)).min(1.0);
        let mut edges = segments;
        if edges[0][0] > edges[1][0] {
            edges.swap(0, 1);
        }
        let track_rect = |start_f: f32, end_f: f32| Rect {
            x: b.x + start_f * b.width,
            y: cy - half_t,
            width: ((end_f - start_f) * b.width).max(0.0),
            height: opt.track_thickness,
        };
        let mut cursor = 0.0_f32;
        for [tail, head] in edges {
            if head <= tail {
                continue;
            }
            let end = (tail - gap_frac).max(cursor);
            if end > cursor {
                let r = track_rect(cursor, end);
                if r.width > 0.0 {
                    self.rect(r, half_t.min(r.width * 0.5), t.secondary_container);
                }
            }
            cursor = cursor.max((head + gap_frac).min(1.0));
        }
        if cursor < 1.0 {
            let r = track_rect(cursor, 1.0);
            if r.width > 0.0 {
                self.rect(r, half_t.min(r.width * 0.5), t.secondary_container);
            }
        }
        for [tail, head] in segments {
            if head <= tail {
                continue;
            }
            let x0 = b.x + tail * b.width;
            let x1 = b.x + head * b.width;
            let seg_w = x1 - x0;
            if seg_w <= 0.0 {
                continue;
            }
            if seg_w <= opt.track_thickness || effective_amp <= 0.05 {
                self.rect(
                    Rect {
                        x: x0,
                        y: cy - half_t,
                        width: seg_w,
                        height: opt.track_thickness,
                    },
                    half_t.min(seg_w * 0.5),
                    t.primary,
                );
            } else {
                let sx0 = x0 + half_t;
                let sx1 = x1 - half_t;
                let span = (sx1 - sx0).max(0.0);
                let step = (opt.wavelength / 16.0).clamp(1.5, 4.0);
                let steps = ((span / step).ceil() as usize).clamp(4, 512);
                let mut prev = [0.0_f32; 2];
                for i in 0..=steps {
                    let frac = i as f32 / steps as f32;
                    let x = sx0 + span * frac;
                    let y = cy
                        + effective_amp
                            * (std::f32::consts::TAU * (x - b.x) / opt.wavelength - phase).sin();
                    let curr = [x, y];
                    if i > 0 {
                        self.paint_line(prev, curr, opt.track_thickness, t.primary);
                    }
                    prev = curr;
                }
            }
        }
        if value.is_some_and(|v| v < 1.0) && opt.stop_indicator_size > 0.0 {
            let stop = opt.stop_indicator_size.min(opt.track_thickness);
            self.rect(
                Rect {
                    x: (b.x + b.width - stop).max(b.x),
                    y: cy - stop * 0.5,
                    width: stop,
                    height: stop,
                },
                stop * 0.5,
                t.primary,
            );
        }
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
