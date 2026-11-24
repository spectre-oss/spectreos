//! Bounded, deterministic motion state. No threads, timers, dependencies or disk caches.
//!
//! Standard effects spring parameters are taken from AndroidX StandardMotionTokens,
//! Copyright 2024 The Android Open Source Project, Apache-2.0. See
//! docs/implementation-progress.md and reference-licenses/compose-material3/LICENSE.txt.
//! The analytic oscillator implementation below is independently written for Bhoo.
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MotionPreference {
    #[default]
    Full,
    Reduced,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MotionScheme {
    #[default]
    Standard,
    Expressive,
}
impl MotionScheme {
    pub fn spatial(self) -> Spring {
        match self {
            Self::Standard => Spring::SPATIAL,
            Self::Expressive => Spring {
                stiffness: 380.0,
                damping: 0.8,
            },
        }
    }
    pub fn fast_spatial(self) -> Spring {
        match self {
            Self::Standard => Spring {
                stiffness: 1400.0,
                damping: 0.9,
            },
            Self::Expressive => Spring {
                stiffness: 800.0,
                damping: 0.6,
            },
        }
    }
    pub fn slow_spatial(self) -> Spring {
        match self {
            Self::Standard => Spring {
                stiffness: 300.0,
                damping: 0.9,
            },
            Self::Expressive => Spring {
                stiffness: 200.0,
                damping: 0.8,
            },
        }
    }
}

/// Standard Material springs: critical effects and slightly underdamped spatial motion.
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    stiffness: f64,
    damping: f64,
}

impl Spring {
    pub const PROGRESS: Self = Self {
        stiffness: 50.0,
        damping: 1.0,
    };
    pub const EFFECTS: Self = Self {
        stiffness: 1600.0,
        damping: 1.0,
    };
    /// AndroidX StandardMotionTokens default spatial profile.
    pub const SPATIAL: Self = Self {
        stiffness: 700.0,
        damping: 0.9,
    };
    pub const FAST_EFFECTS: Self = Self {
        stiffness: 3800.0,
        damping: 1.0,
    };
}

#[derive(Clone, Copy, Debug)]
struct Track {
    value: f64,
    velocity: f64,
    target: f64,
    time: f64,
    seen: u64,
    spring: Spring,
}

impl Track {
    fn advance(&mut self, time: f64, spring: Spring) {
        let dt = (time - self.time).max(0.0);
        self.time = time;
        if dt == 0.0 {
            return;
        }
        // Effects are critically damped. Exact solution avoids frame-rate-dependent
        // Euler instability and catches up after long suspension without stepping a backlog.
        let omega = spring.stiffness.max(1.0).sqrt();
        let displacement = self.value - self.target;
        if spring.damping < 1.0 {
            let rate = spring.damping * omega;
            let frequency = omega * (1.0 - spring.damping * spring.damping).sqrt();
            let c = (self.velocity + rate * displacement) / frequency;
            let phase = frequency * dt;
            let wave = displacement * phase.cos() + c * phase.sin();
            let decay = (-rate * dt).exp();
            self.value = self.target + wave * decay;
            self.velocity =
                (frequency * (-displacement * phase.sin() + c * phase.cos()) - rate * wave) * decay;
        } else {
            let c = self.velocity + omega * displacement;
            let decay = (-omega * dt).exp();
            self.value = self.target + (displacement + c * dt) * decay;
            self.velocity = (self.velocity - omega * c * dt) * decay;
        }
        if (self.value - self.target).abs() < 0.0005 && self.velocity.abs() < 0.01 {
            self.value = self.target;
            self.velocity = 0.0;
        }
    }
    fn moving(self) -> bool {
        self.value != self.target || self.velocity != 0.0
    }
}

/// Retains only properties referenced during the current frame, up to `capacity`.
/// Targets are normalized [0,1] effects, not arbitrary geometry.
pub struct Motion {
    tracks: HashMap<(u64, u8), Track>,
    tweens: HashMap<(u64, u8), (crate::timing::Tween, u64)>,
    clocks: HashMap<(u64, u8), (f64, u64)>,
    capacity: usize,
    epoch: u64,
    time: f64,
    preference: MotionPreference,
}

impl Default for Motion {
    fn default() -> Self {
        Self::new(4096)
    }
}

impl Motion {
    pub fn new(capacity: usize) -> Self {
        Self {
            tracks: HashMap::new(),
            tweens: HashMap::new(),
            clocks: HashMap::new(),
            capacity,
            epoch: 0,
            time: 0.0,
            preference: MotionPreference::Full,
        }
    }
    pub fn begin(&mut self, time: f64, preference: MotionPreference) {
        if time.is_finite() {
            self.time = time.max(self.time);
        }
        self.preference = preference;
        self.epoch = self.epoch.wrapping_add(1);
    }
    /// First encounter initializes at `initial`; use target for no entrance effect.
    /// Retargeting preserves the sampled presentation and velocity.
    pub fn effect(
        &mut self,
        id: u64,
        property: u8,
        target: f32,
        initial: f32,
        spring: Spring,
    ) -> f32 {
        let target = if target.is_finite() {
            target.clamp(0.0, 1.0) as f64
        } else {
            0.0
        };
        let initial = if initial.is_finite() {
            initial.clamp(0.0, 1.0) as f64
        } else {
            target
        };
        let key = (id, property);
        if !self.tracks.contains_key(&key)
            && self.tracks.len() + self.tweens.len() + self.clocks.len() >= self.capacity
        {
            return target as f32;
        }
        let track = self.tracks.entry(key).or_insert(Track {
            value: initial,
            velocity: 0.0,
            target: initial,
            time: self.time,
            seen: self.epoch,
            spring,
        });
        track.advance(self.time, track.spring);
        track.spring = spring;
        track.target = target;
        track.seen = self.epoch;
        // Effects are critical; standard spatial geometry is slightly underdamped.
        // Bound normalized presentation values; reduced motion reconciles immediately.
        if self.preference == MotionPreference::Reduced {
            track.value = target;
            track.velocity = 0.0;
        }
        track.value.clamp(0.0, 1.0) as f32
    }
    #[allow(clippy::too_many_arguments)]
    pub fn tween(
        &mut self,
        id: u64,
        property: u8,
        target: f32,
        initial: f32,
        enter: crate::timing::TweenSpec,
        exit: crate::timing::TweenSpec,
    ) -> f32 {
        self.tween_inner(id, property, target, initial, enter, exit, false)
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn css_tween(
        &mut self,
        id: u64,
        property: u8,
        target: f32,
        initial: f32,
        enter: crate::timing::TweenSpec,
        exit: crate::timing::TweenSpec,
    ) -> f32 {
        self.tween_inner(id, property, target, initial, enter, exit, true)
    }
    #[allow(clippy::too_many_arguments)]
    fn tween_inner(
        &mut self,
        id: u64,
        property: u8,
        target: f32,
        initial: f32,
        enter: crate::timing::TweenSpec,
        exit: crate::timing::TweenSpec,
        css: bool,
    ) -> f32 {
        let key = (id, property);
        let target = if target.is_finite() {
            target.clamp(0.0, 1.0)
        } else {
            0.0
        };
        if !self.tweens.contains_key(&key)
            && self.tracks.len() + self.tweens.len() + self.clocks.len() >= self.capacity
        {
            return target;
        }
        let (track, seen) = self.tweens.entry(key).or_insert((
            crate::timing::Tween::at(if initial.is_finite() {
                initial.clamp(0.0, 1.0)
            } else {
                target
            }),
            self.epoch,
        ));
        *seen = self.epoch;
        let sample = if css {
            crate::timing::Tween::sample_css
        } else {
            crate::timing::Tween::sample
        };
        sample(
            track,
            self.time,
            target,
            enter,
            exit,
            self.preference == MotionPreference::Reduced,
        )
    }
    pub fn elapsed(&mut self, id: u64, property: u8) -> f64 {
        let key = (id, property);
        if !self.clocks.contains_key(&key) && self.retained_properties() >= self.capacity {
            return self.time;
        }
        let (start, seen) = self.clocks.entry(key).or_insert((self.time, self.epoch));
        *seen = self.epoch;
        (self.time - *start).max(0.0)
    }
    pub fn pause_clocks(&mut self) {
        self.clocks.clear();
    }
    pub fn finish(&mut self) {
        let epoch = self.epoch;
        self.tracks.retain(|_, track| track.seen == epoch);
        self.tweens.retain(|_, (_, seen)| *seen == epoch);
        self.clocks.retain(|_, (_, seen)| *seen == epoch);
    }
    pub fn is_animating(&self) -> bool {
        self.tracks.values().any(|track| track.moving())
            || self.tweens.values().any(|(track, _)| track.moving())
    }
    pub fn retained_properties(&self) -> usize {
        self.tracks.len() + self.tweens.len() + self.clocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_spatial_spring_is_animated_stable_and_frame_independent() {
        fn run(times: &[f64]) -> f32 {
            let mut m = Motion::default();
            m.begin(0.0, MotionPreference::Full);
            m.effect(1, 0, 1.0, 0.0, Spring::SPATIAL);
            let mut value = 0.0;
            for &time in times {
                m.begin(time, MotionPreference::Full);
                value = m.effect(1, 0, 1.0, 0.0, Spring::SPATIAL);
            }
            value
        }
        let a = run(&[0.1]);
        let b = run(&[0.02, 0.04, 0.06, 0.08, 0.1]);
        assert!(a > 0.0 && a < 1.0);
        assert!((a - b).abs() < 0.00001);
        assert_eq!(run(&[3.0]), 1.0);
    }
    #[test]
    fn springs_tweens_and_clocks_share_one_bound_and_removed_tracks_are_pruned() {
        let mut m = Motion::new(4);
        m.begin(0.0, MotionPreference::Full);
        let s = crate::timing::TweenSpec::new(0.5, crate::timing::Easing::Linear);
        for i in 0..20 {
            m.tween(i, 1, 1.0, 0.0, s, s);
            m.elapsed(i, 2);
            m.effect(i, 3, 1.0, 0.0, Spring::EFFECTS);
        }
        assert_eq!(m.retained_properties(), 4);
        m.finish();
        m.begin(1.0, MotionPreference::Full);
        m.finish();
        assert_eq!(m.retained_properties(), 0);
        assert!(!m.is_animating());
    }
    fn sample(motion: &mut Motion, time: f64, target: f32, preference: MotionPreference) -> f32 {
        motion.begin(time, preference);
        let value = motion.effect(1, 0, target, 0.0, Spring::EFFECTS);
        motion.finish();
        value
    }
    #[test]
    fn settles_exactly_and_stops_scheduling() {
        let mut motion = Motion::default();
        assert_eq!(sample(&mut motion, 0.0, 1.0, MotionPreference::Full), 0.0);
        assert!(motion.is_animating());
        let mid = sample(&mut motion, 0.05, 1.0, MotionPreference::Full);
        assert!(mid > 0.0 && mid < 1.0);
        assert_eq!(sample(&mut motion, 2.0, 1.0, MotionPreference::Full), 1.0);
        assert!(!motion.is_animating());
    }
    #[test]
    fn reversal_has_no_position_jump_and_eventually_settles() {
        let mut m = Motion::default();
        sample(&mut m, 0.0, 1.0, MotionPreference::Full);
        let before = sample(&mut m, 0.06, 1.0, MotionPreference::Full);
        assert_eq!(sample(&mut m, 0.06, 0.0, MotionPreference::Full), before);
        assert_eq!(sample(&mut m, 2.0, 0.0, MotionPreference::Full), 0.0);
        assert!(!m.is_animating());
    }
    #[test]
    fn analytic_sampling_is_independent_of_frame_count() {
        let mut a = Motion::default();
        let mut b = Motion::default();
        sample(&mut a, 0.0, 1.0, MotionPreference::Full);
        sample(&mut b, 0.0, 1.0, MotionPreference::Full);
        for i in 1..6 {
            sample(&mut a, i as f64 * 0.01, 1.0, MotionPreference::Full);
        }
        assert!(
            (sample(&mut a, 0.06, 1.0, MotionPreference::Full)
                - sample(&mut b, 0.06, 1.0, MotionPreference::Full))
            .abs()
                < 0.00001
        );
    }
    #[test]
    fn reduced_motion_finishes_in_flight_effects() {
        let mut m = Motion::default();
        sample(&mut m, 0.0, 1.0, MotionPreference::Full);
        assert_eq!(sample(&mut m, 0.01, 1.0, MotionPreference::Reduced), 1.0);
        assert!(!m.is_animating());
    }
    #[test]
    fn removed_properties_are_reclaimed_and_cache_is_bounded() {
        let mut m = Motion::new(2);
        m.begin(0.0, MotionPreference::Full);
        for id in 0..100 {
            m.effect(id, 0, 1.0, 0.0, Spring::EFFECTS);
        }
        m.finish();
        assert_eq!(m.retained_properties(), 2);
        m.begin(1.0, MotionPreference::Full);
        m.finish();
        assert_eq!(m.retained_properties(), 0);
        assert!(!m.is_animating());
    }
    #[test]
    fn nonfinite_and_backwards_time_does_not_corrupt_values() {
        let mut m = Motion::default();
        sample(&mut m, 1.0, 1.0, MotionPreference::Full);
        for t in [f64::NAN, -1.0, f64::INFINITY, 1e20] {
            let value = sample(&mut m, t, 1.0, MotionPreference::Full);
            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
        }
    }
}
