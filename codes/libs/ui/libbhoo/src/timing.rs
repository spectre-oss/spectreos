//! Duration curves adapted from Material Web internal/motion/animation.ts,
//! Copyright 2021 Google LLC, Apache-2.0. Independently implemented CSS sampling.
#[derive(Clone, Copy, Debug, Default)]
pub enum Easing {
    #[default]
    Linear,
    Standard,
    Emphasized,
    EmphasizedAccelerate,
    EmphasizedDecelerate,
    StandardDecelerate,
    Backdrop,
    Cubic([f32; 4]),
}
impl Easing {
    pub fn sample(self, x: f32) -> f32 {
        let (x1, y1, x2, y2) = match self {
            Self::Linear => return x.clamp(0.0, 1.0),
            Self::Standard => (0.2, 0.0, 0.0, 1.0),
            Self::Emphasized => (0.3, 0.0, 0.0, 1.0),
            Self::EmphasizedAccelerate => (0.3, 0.0, 0.8, 0.15),
            Self::EmphasizedDecelerate => (0.05, 0.7, 0.1, 1.0),
            Self::StandardDecelerate => (0.0, 0.0, 0.2, 1.0),
            Self::Backdrop => (0.25, 0.8, 0.25, 1.0),
            Self::Cubic([x1, y1, x2, y2]) => (x1, y1, x2, y2),
        };
        let x = x.clamp(0.0, 1.0);
        if x == 0.0 || x == 1.0 {
            return x;
        }
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..20 {
            let t = (lo + hi) * 0.5;
            let u = 1.0 - t;
            let bx = 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t;
            if bx < x {
                lo = t;
            } else {
                hi = t;
            }
        }
        let t = (lo + hi) * 0.5;
        let u = 1.0 - t;
        3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t
    }
}
#[derive(Clone, Copy, Debug)]
pub struct TweenSpec {
    pub delay: f64,
    pub duration: f64,
    pub easing: Easing,
}
impl TweenSpec {
    pub const fn new(duration: f64, easing: Easing) -> Self {
        Self {
            delay: 0.0,
            duration,
            easing,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
/// Interruptible duration-based animation using the Material easing curves.
pub struct Tween {
    value: f32,
    from: f32,
    target: f32,
    start: f64,
    delay: f64,
    duration: f64,
    easing: Easing,
    reverse_start: f32,
    reverse_factor: f64,
}
impl Tween {
    pub fn at(value: f32) -> Self {
        Self {
            value,
            from: value,
            target: value,
            reverse_start: value,
            reverse_factor: 1.0,
            ..Self::default()
        }
    }
    pub fn sample(
        &mut self,
        time: f64,
        target: f32,
        enter: TweenSpec,
        exit: TweenSpec,
        reduced: bool,
    ) -> f32 {
        let time = if time.is_finite() {
            time.max(self.start)
        } else {
            self.start
        };
        self.advance(time);
        if target != self.target {
            let spec = if target > self.target { enter } else { exit };
            self.from = self.value;
            self.target = target;
            self.start = time;
            self.delay = spec.delay.max(0.0);
            self.duration = spec.duration.max(0.0);
            self.easing = spec.easing;
        }
        if reduced {
            self.value = target;
            self.from = target;
            self.target = target;
            self.duration = 0.0;
            self.delay = 0.0;
        }
        self.advance(time);
        self.value
    }
    /// CSS transition reversal shortening, preserving the current presentation.
    pub fn sample_css(
        &mut self,
        time: f64,
        target: f32,
        mut enter: TweenSpec,
        mut exit: TweenSpec,
        reduced: bool,
    ) -> f32 {
        let time = if time.is_finite() {
            time.max(self.start)
        } else {
            self.start
        };
        self.advance(time);
        if target != self.target {
            let reversing = self.moving() && target == self.reverse_start;
            let factor = if reversing {
                let progress = ((self.value - self.from) / (self.target - self.from)) as f64;
                (progress * self.reverse_factor + 1.0 - self.reverse_factor)
                    .abs()
                    .clamp(0.0, 1.0)
            } else {
                1.0
            };
            self.reverse_start = if reversing { self.target } else { self.value };
            self.reverse_factor = factor;
            enter.duration *= factor;
            exit.duration *= factor;
        }
        self.sample(time, target, enter, exit, reduced)
    }
    fn advance(&mut self, time: f64) {
        let elapsed = (time - self.start - self.delay).max(0.0);
        if time >= self.start + self.delay {
            let progress = if self.duration > 0.0 {
                (elapsed / self.duration).min(1.0) as f32
            } else {
                1.0
            };
            self.value = self.from + (self.target - self.from) * self.easing.sample(progress);
        }
    }
    pub fn moving(&self) -> bool {
        self.value != self.target
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reversal_samples_old_presentation_before_retarget() {
        let mut t = Tween::default();
        let spec = TweenSpec::new(0.5, Easing::Emphasized);
        t.sample(0.0, 1.0, spec, spec, false);
        let mid = t.sample(0.1, 1.0, spec, spec, false);
        assert!(mid > 0.0 && mid < 1.0);
        assert_eq!(mid, t.sample(0.1, 0.0, spec, spec, false));
        assert_eq!(t.sample(1.0, 0.0, spec, spec, false), 0.0);
        assert!(!t.moving());
    }
    #[test]
    fn delay_zero_duration_and_reduced_motion_are_exact() {
        let mut t = Tween::default();
        let s = TweenSpec {
            delay: 0.1,
            duration: 0.0,
            easing: Easing::Linear,
        };
        assert_eq!(t.sample(0.0, 1.0, s, s, false), 0.0);
        assert_eq!(t.sample(0.1, 1.0, s, s, false), 1.0);
        assert_eq!(t.sample(0.1, 0.0, s, s, true), 0.0);
        assert!(!t.moving());
    }
}
