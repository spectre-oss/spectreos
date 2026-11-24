//! Independently implemented bounded ink feedback, based on Material Web's pinned
//! ripple timing/geometry constants (Copyright 2022 Google LLC, Apache-2.0).
//! See THIRD_PARTY_NOTICES.md. No JavaScript, tasks, textures or per-ripple timers.
use crate::RippleStyle;
use std::collections::HashMap;

pub(crate) const GROW: f64 = 0.450;
pub(crate) const MIN_PRESS: f64 = 0.225;
pub(crate) const FADE_IN: f64 = 0.105;
pub(crate) const FADE_OUT: f64 = 0.375;
#[derive(Clone, Copy)]
struct Ink {
    start: f64,
    release: Option<f64>,
    origin: [f32; 2],
    seen: u64,
    style: RippleStyle,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sample {
    pub origin: [f32; 2],
    pub growth: f32,
    pub opacity: f32,
    pub moving: bool,
    pub wake: Option<f64>,
    pub style: RippleStyle,
}
#[derive(Default)]
pub(crate) struct Ripples {
    inks: HashMap<(u64, u32), Ink>,
    epoch: u64,
}
impl Ripples {
    pub fn pause(&mut self) {
        self.inks.clear();
    }
    pub fn begin(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }
    pub fn finish(&mut self) {
        let epoch = self.epoch;
        self.inks.retain(|_, ink| ink.seen == epoch);
    }
    pub fn clear(&mut self, id: (u64, u32)) {
        self.inks.remove(&id);
    }
    pub fn cancel_pending(&mut self, id: (u64, u32), time: f64) {
        if self.inks.get(&id).is_some_and(|ink| time < ink.start) {
            self.inks.remove(&id);
        }
    }
    #[cfg(test)]
    pub fn sample(
        &mut self,
        id: (u64, u32),
        time: f64,
        start: Option<[f32; 2]>,
        held: bool,
    ) -> Option<Sample> {
        self.sample_touch(id, time, start, held, false)
    }
    #[cfg(test)]
    pub fn sample_touch(
        &mut self,
        id: (u64, u32),
        time: f64,
        start: Option<[f32; 2]>,
        held: bool,
        touch: bool,
    ) -> Option<Sample> {
        self.sample_with_style(id, time, start, held, touch, RippleStyle::MaterialWeb)
    }
    pub fn sample_with_style(
        &mut self,
        id: (u64, u32),
        time: f64,
        start: Option<[f32; 2]>,
        held: bool,
        touch: bool,
        style: RippleStyle,
    ) -> Option<Sample> {
        if let Some(origin) = start
            && (self.inks.len() < 1024 || self.inks.contains_key(&id))
        {
            self.inks.insert(
                id,
                Ink {
                    start: time
                        + if touch && style == RippleStyle::MaterialWeb {
                            0.150
                        } else {
                            0.0
                        },
                    release: None,
                    origin,
                    seen: self.epoch,
                    style,
                },
            );
        }
        let ink = self.inks.get_mut(&id)?;
        ink.seen = self.epoch;
        let angular = ink.style == RippleStyle::AngularMaterial;
        let growth = if angular { 0.225 } else { GROW };
        let fade_out = if angular { 0.150 } else { FADE_OUT };
        if !held && ink.release.is_none() {
            ink.start = ink.start.min(time);
            ink.release = Some(time.max(ink.start + if angular { growth } else { MIN_PRESS }));
        }
        if time < ink.start {
            return Some(Sample {
                origin: ink.origin,
                growth: 0.0,
                opacity: 0.0,
                moving: false,
                wake: Some(ink.start),
                style: ink.style,
            });
        }
        let age = (time - ink.start).max(0.0);
        let fade = ink.release.map_or(1.0, |release| {
            (1.0 - (time - release).max(0.0) / fade_out).max(0.0)
        });
        if fade <= 0.0 {
            self.inks.remove(&id);
            return None;
        }
        let opacity = if angular {
            fade
        } else {
            (age / FADE_IN).min(1.0) * fade
        };
        let growing = age < growth;
        let fading = ink.release.is_some_and(|t| time >= t);
        Some(Sample {
            origin: ink.origin,
            growth: if angular {
                crate::timing::Easing::StandardDecelerate.sample((age / growth).min(1.0) as f32)
            } else {
                standard_ease((age / growth).min(1.0) as f32)
            },
            opacity: opacity as f32,
            moving: growing || !angular && age < FADE_IN || fading,
            wake: ink.release.filter(|&t| t > time),
            style: ink.style,
        })
    }
}
/// Invert x with bounded bisection: CSS cubic-bezier(.2,0,0,1).
/// A polynomial in elapsed time would not reproduce CSS timing.
pub(crate) fn standard_ease(x: f32) -> f32 {
    crate::timing::Easing::Standard.sample(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn angular_quick_tap_keeps_origin_and_minimum_feedback_then_reclaims() {
        let mut r = Ripples::default();
        r.begin();
        let style = RippleStyle::AngularMaterial;
        let a = r
            .sample_with_style((7, 0), 0.0, Some([0.2, 0.8]), true, true, style)
            .unwrap();
        assert_eq!(a.opacity, 1.0);
        r.sample_with_style((7, 0), 0.01, None, false, true, style);
        let held = r
            .sample_with_style((7, 0), 0.20, None, false, true, style)
            .unwrap();
        assert_eq!(held.origin, [0.2, 0.8]);
        assert_eq!(held.opacity, 1.0);
        let fading = r
            .sample_with_style((7, 0), 0.3, None, false, true, style)
            .unwrap();
        assert!(fading.opacity > 0.0 && fading.opacity < 1.0);
        assert!(
            r.sample_with_style((7, 0), 0.4, None, false, true, style)
                .is_none()
        );
    }
    #[test]
    fn short_tap_preserves_minimum_press_and_fades_to_removal() {
        let mut r = Ripples::default();
        r.begin();
        let s = r.sample((1, 0), 0.0, Some([0.1, 0.8]), true).unwrap();
        assert_eq!(s.origin, [0.1, 0.8]);
        assert_eq!(s.opacity, 0.0);
        r.sample((1, 0), 0.01, None, false);
        let s = r.sample((1, 0), 0.225, None, false).unwrap();
        assert_eq!(s.opacity, 1.0);
        assert!(r.sample((1, 0), 0.4, None, false).unwrap().opacity < 1.0);
        assert!(r.sample((1, 0), 0.601, None, false).is_none());
    }
    #[test]
    fn holding_settles_without_polling_and_release_resumes() {
        let mut r = Ripples::default();
        r.begin();
        r.sample((1, 0), 0.0, Some([0.5; 2]), true);
        let s = r.sample((1, 0), 1.0, None, true).unwrap();
        assert!(!s.moving);
        assert!(s.wake.is_none());
        assert_eq!(s.growth, 1.0);
        assert!(r.sample((1, 0), 1.1, None, false).unwrap().moving);
        assert!(r.sample((1, 0), 1.5, None, false).is_none());
    }
    #[test]
    fn repress_replaces_old_lifetime_and_removed_owners_are_reclaimed() {
        let mut r = Ripples::default();
        r.begin();
        r.sample((1, 0), 0.0, Some([0.1; 2]), true);
        r.sample((1, 0), 0.02, None, false);
        let s = r.sample((1, 0), 0.1, Some([0.9; 2]), true).unwrap();
        assert_eq!(s.origin, [0.9; 2]);
        assert_eq!(s.opacity, 0.0);
        r.finish();
        r.begin();
        r.finish();
        assert!(r.inks.is_empty());
    }
    #[test]
    fn css_curve_endpoints_and_monotonicity() {
        assert_eq!(standard_ease(0.0), 0.0);
        assert_eq!(standard_ease(1.0), 1.0);
        let mut old = 0.0;
        for i in 0..100 {
            let v = standard_ease(i as f32 / 100.0);
            assert!(v >= old);
            old = v;
        }
        assert!(standard_ease(0.5) > 0.8);
    }
    #[test]
    fn state_is_hard_bounded() {
        let mut r = Ripples::default();
        r.begin();
        for i in 0..2048 {
            r.sample((i, 0), 0.0, Some([0.5; 2]), true);
        }
        assert_eq!(r.inks.len(), 1024);
    }
}
