//! Android-style spline flings in logical pixels, with AOSP EdgeEffect stretch overscroll.
//! Spline & EdgeEffect stretch constants/formulas adapted from AOSP OverScroller & EdgeEffect (Apache-2.0).
//! Copyright (C) 2010, 2021 The Android Open Source Project. See THIRD_PARTY_NOTICES.md.

/// Android 12/13 `EdgeEffect` (`TYPE_STRETCH`) overscroll spring state machine.
/// Positive `distance` = pulling away from the start edge (top / left);
/// negative `distance` = pulling away from the end edge (bottom / right).
#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeStretch {
    distance: f32,
    velocity: f32,
    pulling: bool,
}

impl EdgeStretch {
    // Exact AOSP EdgeEffect.java stretch constants (Apache-2.0):
    const NATURAL_FREQUENCY: f64 = 24.657;
    const DAMPING_RATIO: f64 = 0.98;
    const LINEAR_STRETCH_INTENSITY: f32 = 0.016;
    const EXP_STRETCH_INTENSITY: f32 = 0.016;
    const SCROLL_DIST_AFFECTED_BY_EXP_STRETCH: f32 = 0.33;
    const LINEAR_VELOCITY_TAKE_OVER: f32 = 200.0;
    const LINEAR_DISTANCE_TAKE_OVER: f32 = 8.0;
    const VELOCITY_THRESHOLD: f32 = 0.01;
    const VALUE_THRESHOLD: f32 = 0.001;

    /// Pull the edge effect by `delta_dp` (+ = start edge pull, - = end edge pull).
    /// Returns the portion of `delta_dp` consumed by stretching or un-stretching.
    pub fn pull(&mut self, delta_dp: f32, extent_dp: f32) -> f32 {
        let extent = extent_dp.max(120.0);
        if delta_dp.abs() < 1e-5 {
            return 0.0;
        }
        let prev = self.distance;
        let next = (prev + delta_dp / extent).clamp(-1.0, 1.0);
        // Never cross directly from top stretch to bottom stretch in one pull step:
        // consume only up to equilibrium 0.0 so the rest can scroll content normally.
        if prev != 0.0 && prev.signum() != next.signum() && next != 0.0 {
            self.distance = 0.0;
            self.velocity = 0.0;
            self.pulling = false;
            return -prev * extent;
        }
        self.distance = next;
        self.velocity = 0.0;
        self.pulling = self.distance.abs() > 1e-5;
        (next - prev) * extent
    }

    /// Release an active stretch pull so the AOSP damped spring recoils smoothly to 0.
    pub fn release(&mut self) {
        self.pulling = false;
    }

    /// Absorb fling impact velocity (`impact_vel_dp_s`: + = top/start edge hit, - = bottom/end edge hit).
    pub fn absorb(&mut self, impact_vel_dp_s: f32, extent_dp: f32) {
        if impact_vel_dp_s.abs() < 100.0 {
            return;
        }
        let extent = extent_dp.max(120.0);
        self.pulling = false;
        // Scale impact velocity into AOSP spring momentum so fast flings visibly stretch (~3-7%) and recoil smoothly.
        let v = (impact_vel_dp_s * 1.65).clamp(-8000.0, 8000.0);
        self.velocity = v;
        if self.distance.abs() < 1e-4 {
            self.distance = (v * 0.002 / extent).clamp(-0.25, 0.25);
        }
    }

    /// Reset stretch immediately (e.g. reduced motion or page change).
    pub fn reset(&mut self) {
        self.distance = 0.0;
        self.velocity = 0.0;
        self.pulling = false;
    }

    /// Current signed normalized overscroll distance in `[-1.0, 1.0]`.
    pub fn distance(&self) -> f32 {
        self.distance
    }

    /// Advance the AOSP damped harmonic oscillator by `dt` seconds. Returns true while active.
    pub fn step(&mut self, dt: f32, extent_dp: f32, reduced: bool) -> bool {
        if reduced {
            self.reset();
            return false;
        }
        if self.pulling {
            return self.distance.abs() > 1e-4;
        }
        if self.distance.abs() <= 1e-5 && self.velocity.abs() <= Self::VELOCITY_THRESHOLD {
            self.reset();
            return false;
        }
        let dt = dt.clamp(0.0, 0.05);
        if dt < 0.0005 {
            return true;
        }
        let height = extent_dp.max(120.0);
        let sign = if self.distance != 0.0 {
            self.distance.signum()
        } else {
            self.velocity.signum()
        };
        let mut dist_pos = self.distance * sign;
        let mut vel_pos = self.velocity * sign;

        if vel_pos.abs() <= Self::LINEAR_VELOCITY_TAKE_OVER
            && (dist_pos * height).abs() < Self::LINEAR_DISTANCE_TAKE_OVER
            && vel_pos < 0.0
        {
            vel_pos = -Self::LINEAR_VELOCITY_TAKE_OVER;
            let target = dist_pos + (vel_pos * dt / height);
            if target <= 0.0 {
                self.reset();
                return false;
            }
            self.distance = target * sign;
            self.velocity = vel_pos * sign;
            return true;
        }

        let omega = Self::NATURAL_FREQUENCY;
        let zeta = Self::DAMPING_RATIO;
        let damped_freq = omega * (1.0 - zeta * zeta).sqrt();
        let dt64 = f64::from(dt);
        let h64 = f64::from(height);
        let cos_coeff = f64::from(dist_pos) * h64;
        let sin_coeff = (1.0 / damped_freq) * (zeta * omega * cos_coeff + f64::from(vel_pos));
        let exp_decay = (-zeta * omega * dt64).exp();
        let angle = damped_freq * dt64;
        let next_disp = exp_decay * (cos_coeff * angle.cos() + sin_coeff * angle.sin());
        let next_vel = next_disp * (-omega * zeta)
            + exp_decay * (-damped_freq * cos_coeff * angle.sin() + damped_freq * sin_coeff * angle.cos());

        dist_pos = (next_disp / h64) as f32;
        vel_pos = next_vel as f32;
        if dist_pos > 1.0 {
            dist_pos = 1.0;
            vel_pos = 0.0;
        }
        // AOSP EdgeEffect.isAtEquilibrium(): never cross 0 to stretch the opposite edge!
        if dist_pos <= 0.0
            || (vel_pos.abs() < Self::VELOCITY_THRESHOLD
                && (dist_pos * height) < Self::VALUE_THRESHOLD)
        {
            self.reset();
            return false;
        }
        self.distance = dist_pos * sign;
        self.velocity = vel_pos * sign;
        true
    }

    /// Returns `(scale_factor, edge_sign)` where `scale_factor >= 1.0` and
    /// `edge_sign` is `+1.0` (start/top edge), `-1.0` (end/bottom edge), or `0.0` (idle).
    pub fn scale_and_sign(&self) -> (f32, f32) {
        let overscroll = self.distance.abs().clamp(0.0, 1.0);
        if overscroll <= 1e-5 {
            return (1.0, 0.0);
        }
        let linear = Self::LINEAR_STRETCH_INTENSITY * overscroll;
        let scalar = std::f32::consts::E / Self::SCROLL_DIST_AFFECTED_BY_EXP_STRETCH;
        let exp_i = Self::EXP_STRETCH_INTENSITY * (1.0 - (-overscroll * scalar).exp());
        let stretch = (1.0 + (linear + exp_i) * 4.2).clamp(1.0, 1.18);
        (stretch, self.distance.signum())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScrollPhysics {
    pub offset: f32,
    pub dragging: bool,
    touch: Option<(f32, f32)>,
    sample_time: f64,
    velocity: f32,
    last: f64,
    edge_velocity: f32,
    flight: Option<(f64, f32, f32, f32)>,
    destination: Option<f32>,
    smooth_target: Option<(f64, f32, f32, f64)>,
    horizontal_rtl: Option<bool>,
    stretch: EdgeStretch,
    last_viewport: f32,
}
impl ScrollPhysics {
    pub fn begin(&mut self, y: f32, time: f64) {
        self.cancel();
        self.touch = Some((y, y));
        self.sample_time = time;
        self.last = time;
        self.edge_velocity = 0.;
    }
    pub fn drag(&mut self, y: f32, time: f64, max: f32, viewport: f32) {
        self.last_viewport = viewport.max(120.0);
        if let Some((origin, last)) = self.touch {
            self.dragging |= (y - origin).abs() > 6.;
            let delta = last - y;
            if self.dragging && delta.abs() > 0.01 {
                let dt = (time - self.sample_time).clamp(0.001, 0.15) as f32;
                self.velocity +=
                    ((delta / dt).clamp(-8000., 8000.) - self.velocity) * (1. - (-dt / 0.04).exp());
                let outside = self.offset < 0. || self.offset > max;
                if outside || (self.offset <= 0. && delta < 0.) || (self.offset >= max && delta > 0.) {
                    let _ = self.stretch.pull(-delta, self.last_viewport);
                }
                self.offset += delta * if outside { 0.3 } else { 1. };
                let limit = viewport * 0.12;
                self.offset = self.offset.clamp(-limit, max + limit);
                self.sample_time = time;
            } else if time - self.sample_time > 0.1 {
                self.velocity = 0.;
            }
            self.touch = Some((origin, y));
        }
    }
    /// Scroll first on a downward drag, then pass the remaining movement to
    /// the refresh parent. On reversal, consume the pull before scrolling.
    pub(crate) fn drag_with_refresh(&mut self, y: f32, time: f64, max: f32, pull: &mut f32) {
        if let Some((origin, last)) = self.touch {
            self.dragging |= (y-origin).abs() > 6.0;
            let mut delta = last-y;
            if self.dragging && delta.abs() > 0.01 {
                if delta > 0.0 && *pull > 0.0 {
                    let consumed=delta.min(*pull);
                    *pull-=consumed; delta-=consumed;
                }
                let old=self.offset.clamp(0.0,max);
                let new=(old+delta).clamp(0.0,max);
                if delta < 0.0 { *pull += (new-old-delta).max(0.0); }
                self.offset=new;
                let dt=(time-self.sample_time).clamp(0.001,0.15) as f32;
                if *pull > 0.0 { self.velocity=0.0; }
                else { self.velocity += (((new-old)/dt).clamp(-8000.0,8000.0)-self.velocity)*(1.0-(-dt/0.04).exp()); }
                self.sample_time=time;
            } else if time-self.sample_time > 0.1 { self.velocity=0.0; }
            self.touch=Some((origin,y));
        }
    }
    pub fn release(&mut self, time: f64, reduced: bool) {
        self.stretch.release();
        if self.touch.is_none() {
            return;
        }
        self.touch = None;
        self.dragging = false;
        if time - self.sample_time > 0.1 || reduced {
            self.velocity = 0.;
        }
        if self.velocity.abs() >= 50. {
            // AOSP physical coefficient at density=1: CSS/Bhoo coordinates are dp.
            let physical = 9.80665_f32 * 39.37 * 160. * 0.84;
            let friction = 0.015 * physical;
            let rate = 0.78_f32.ln() / 0.9_f32.ln();
            let l = (0.35 * self.velocity.abs() / friction).ln();
            let duration = (l / (rate - 1.)).exp();
            let distance = friction * (rate / (rate - 1.) * l).exp() * self.velocity.signum();
            self.flight = Some((time, self.offset, distance, duration));
        }
        self.last = time;
    }
    pub(crate) fn fling_from_velocity(&mut self, velocity:f32, time:f64, reduced:bool) {
        self.cancel();self.touch=Some((0.0,0.0));self.sample_time=time;self.velocity=velocity.clamp(-8000.0,8000.0);self.release(time,reduced);
    }
    pub fn cancel(&mut self) {
        self.touch = None;
        self.dragging = false;
        self.velocity = 0.;
        self.flight = None;
        self.destination = None;
        self.smooth_target = None;
        self.stretch.release();
    }
    /// Smoothly animate `offset` to `target` (clamped to `0..=max`) over `duration_s`
    /// using Material's Standard easing curve, or snap immediately under `reduced`.
    pub fn scroll_to(&mut self, target: f32, max: f32, time: f64, duration_s: f64, reduced: bool) {
        self.cancel();
        let clamped = target.clamp(0.0, max.max(0.0));
        if reduced || duration_s <= 0.0 || (clamped - self.offset).abs() < 0.1 {
            self.offset = clamped;
            self.smooth_target = None;
        } else {
            self.smooth_target = Some((time, self.offset, clamped, duration_s));
            self.last = time;
        }
    }
    /// Jump immediately to any active `scroll_to` destination, clearing the animation.
    pub fn finish_scroll_to(&mut self) {
        if let Some((_, _, target, _)) = self.smooth_target.take() {
            self.offset = target;
        }
    }
    /// Whether a programmatic `scroll_to` animation is currently active.
    pub fn is_scrolling_to(&self) -> bool {
        self.smooth_target.is_some()
    }
    pub fn touching(&self) -> bool {
        self.touch.is_some()
    }
    pub fn edge_stretch(&self) -> (f32, f32) {
        self.stretch.scale_and_sign()
    }
    pub fn wheel(&mut self, delta: f32, max: f32, reduced: bool) {
        if delta == 0. {
            return;
        }
        self.flight = None;
        self.smooth_target = None;
        let target = (self.destination.unwrap_or(self.offset) + delta).clamp(0., max);
        if reduced {
            self.offset = target;
            self.destination = None;
        } else {
            self.destination = Some(target);
        }
    }
    pub fn advance(&mut self, time: f64, max: f32, reduced: bool) -> bool {
        let dt = (time - self.last).clamp(0., 0.05) as f32;
        self.last = time;
        let vp = self.last_viewport.max(120.0);
        if self.touch.is_some() {
            return self.stretch.step(dt, vp, reduced);
        }
        if reduced {
            let final_target = self
                .smooth_target
                .map(|(_, _, t, _)| t)
                .or(self.destination)
                .unwrap_or(self.offset);
            self.offset = final_target.clamp(0., max);
            self.cancel();
            self.stretch.reset();
            return false;
        }
        if let Some((start, origin, target, duration)) = self.smooth_target {
            let clamped = target.clamp(0.0, max.max(0.0));
            let progress = ((time - start) / duration.max(1e-6)).clamp(0.0, 1.0) as f32;
            let eased = crate::timing::Easing::Standard.sample(progress);
            self.offset = origin + (clamped - origin) * eased;
            if progress >= 1.0 {
                self.offset = clamped;
                self.smooth_target = None;
            }
        }
        if let Some(target) = self.destination {
            self.offset += (target - self.offset) * (1. - (-18. * dt).exp());
            if (target - self.offset).abs() < 0.1 {
                self.offset = target;
                self.destination = None;
            }
        }
        if let Some((start, origin, distance, duration)) = self.flight {
            let prev = self.offset;
            let t = ((time - start) as f32 / duration).clamp(0., 1.);
            self.offset = origin + distance * spline(t);
            if t >= 1. || self.offset < 0. || self.offset > max {
                if self.offset < 0. || self.offset > max {
                    let impact_vel = if dt > 0.001 {
                        (self.offset - prev) / dt
                    } else {
                        distance / duration.max(0.05)
                    };
                    self.stretch.absorb(-impact_vel, vp);
                }
                self.flight = None;
                self.offset = self.offset.clamp(-48., max + 48.);
            }
        }
        let edge = self.offset.clamp(0., max);
        if self.offset != edge {
            // Analytic critically damped spring: mass=1, stiffness=324, damping=36.
            let displacement = self.offset - edge;
            let c = self.edge_velocity + 18. * displacement;
            let decay = (-18. * dt).exp();
            self.offset = edge + (displacement + c * dt) * decay;
            self.edge_velocity = (self.edge_velocity - 18. * c * dt) * decay;
            if (self.offset - edge).abs() < 0.1 {
                self.offset = edge;
                self.edge_velocity = 0.;
            }
        }
        let stretch_active = self.stretch.step(dt, vp, reduced);
        self.flight.is_some()
            || self.destination.is_some()
            || self.smooth_target.is_some()
            || self.offset != edge
            || stretch_active
    }
}
fn spline(t: f32) -> f32 {
    let (mut lo, mut hi) = (0., 1.);
    for _ in 0..16 {
        let x = (lo + hi) * 0.5;
        let coef = 3. * x * (1. - x);
        let tx = coef * ((1. - x) * 0.175 + x * 0.35) + x * x * x;
        if tx < t {
            lo = x;
        } else {
            hi = x;
        }
    }
    let x = (lo + hi) * 0.5;
    3. * x * (1. - x) * ((1. - x) * 0.5 + x) + x * x * x
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fling() -> ScrollPhysics {
        let mut s = ScrollPhysics::default();
        s.begin(500., 0.);
        s.drag(450., 0.02, 10000., 800.);
        s.drag(400., 0.04, 10000., 800.);
        s.release(0.04, false);
        s
    }
    #[test]
    fn refresh_rate_does_not_change_fling_distance() {
        let mut a = fling();
        let mut b = a.clone();
        for i in 1..=60 {
            a.advance(0.04 + i as f64 / 60., 10000., false);
        }
        for i in 1..=120 {
            b.advance(0.04 + i as f64 / 120., 10000., false);
        }
        assert!((a.offset - b.offset).abs() < 0.01);
    }
    #[test]
    fn edge_returns_and_reduced_motion_stops() {
        let mut a = ScrollPhysics::default();
        a.begin(100., 0.);
        a.drag(250., 0.03, 1000., 800.);
        assert!(a.offset < 0.);
        a.release(0.2, false);
        for i in 1..120 {
            a.advance(0.2 + i as f64 / 60., 1000., false);
        }
        assert_eq!(a.offset, 0.);
        let mut b = fling();
        let before = b.offset;
        b.advance(0.1, 10000., true);
        assert_eq!(b.offset, before);
    }
    #[test]
    fn edge_stretch_pull_release_and_absorb_spring_recoil() {
        let mut s = EdgeStretch::default();
        assert_eq!(s.scale_and_sign(), (1.0, 0.0));
        // Pulling down at top stretches > 1.0 with positive sign
        let consumed = s.pull(200.0, 800.0);
        assert!((consumed - 200.0).abs() < 0.01);
        let (sc, sign) = s.scale_and_sign();
        assert!(sc > 1.04 && sc < 1.16 && sign > 0.0);
        // Pulling back up consumes un-stretch before returning to 0
        let back = s.pull(-300.0, 800.0);
        assert!((back + 200.0).abs() < 0.01);
        assert_eq!(s.distance(), 0.0);
        // Pull bottom (-250dp) then release -> AOSP damped spring smoothly returns to 0 without crossing!
        s.pull(-250.0, 800.0);
        assert!(s.scale_and_sign().1 < 0.0);
        s.release();
        for _ in 0..60 {
            s.step(1.0 / 60.0, 800.0, false);
            assert!(s.distance() <= 0.0, "stretch must never bounce through 0 to opposite edge");
        }
        assert_eq!(s.distance(), 0.0);
        // Fling impact absorb stretches and springs back cleanly
        s.absorb(2400.0, 800.0);
        assert!(s.step(1.0 / 60.0, 800.0, false));
        assert!(s.scale_and_sign().0 > 1.005);
        for _ in 0..90 {
            s.step(1.0 / 60.0, 800.0, false);
        }
        assert_eq!(s.distance(), 0.0);
    }
    #[test]
    fn scroll_to_animates_smoothly_and_respects_reduced_motion() {
        let mut s = ScrollPhysics::default();
        s.scroll_to(600.0, 1000.0, 1.0, 0.5, false);
        assert!(s.is_scrolling_to());
        assert!(s.advance(1.25, 1000.0, false));
        assert!(s.offset > 100.0 && s.offset < 600.0);
        assert!(!s.advance(1.55, 1000.0, false));
        assert!((s.offset - 600.0).abs() < 0.01);

        let mut r = ScrollPhysics::default();
        r.scroll_to(1200.0, 1000.0, 0.0, 0.5, true);
        assert!(!r.is_scrolling_to());
        assert_eq!(r.offset, 1000.0);
    }
}

impl crate::Frame<'_> {
    /// Current Android 12/13 EdgeEffect stretch `(scale_factor, edge_sign)` for a scroll viewport.
    pub fn scroll_stretch(&self, id: u64) -> (f32, f32) {
        self.ui
            .scrolls
            .get(&id)
            .map_or((1.0, 0.0), ScrollPhysics::edge_stretch)
    }

    /// Apply Android 12/13 `EdgeEffect` (`TYPE_STRETCH`) vertical or horizontal stretch transform
    /// anchored at the start edge (`edge_sign > 0`) or end edge (`edge_sign < 0`).
    /// No-op when `stretch_scale <= 1.0005` (zero transform overhead at rest).
    pub fn with_edge_stretch(
        &mut self,
        bounds: crate::Rect,
        horizontal: bool,
        stretch_scale: f32,
        edge_sign: f32,
        build: impl FnOnce(&mut Self),
    ) {
        let s = stretch_scale.clamp(1.0, 1.20);
        if s > 1.0005 && edge_sign.abs() > 0.01 {
            let (pivot, scale) = if horizontal {
                let px = if edge_sign > 0.0 {
                    bounds.x
                } else {
                    bounds.x + bounds.width
                };
                ([px, bounds.y + bounds.height * 0.5], [s, 1.0])
            } else {
                let py = if edge_sign > 0.0 {
                    bounds.y
                } else {
                    bounds.y + bounds.height
                };
                ([bounds.x + bounds.width * 0.5, py], [1.0, s])
            };
            self.push_transform(pivot, [0.0, 0.0], scale, 0.0);
            build(self);
            self.pop_transform();
        } else {
            build(self);
        }
    }

    /// Touch/fling handling shared by clipped scroll containers. Wheel input is
    /// already normalized by the platform adapter; no second easing is applied.
    pub fn scroll_viewport(&mut self, id: u64, b: crate::Rect, offset: &mut f32, max: f32) -> bool {
        self.scroll_viewport_axis(id, b, offset, max, false, false)
    }

    /// Stop any touch drag or fling owned by this viewport before external scrolling.
    pub fn cancel_scroll(&mut self, id: u64) {
        if let Some(state) = self.ui.scrolls.get_mut(&id) { state.cancel(); }
    }

    /// Horizontal counterpart of `scroll_viewport`, using the same fling physics.
    pub fn scroll_viewport_horizontal(&mut self, id: u64, b: crate::Rect, offset: &mut f32, max: f32) -> bool {
        self.scroll_viewport_axis(id, b, offset, max, true, false)
    }

    /// Horizontal viewport whose touch direction follows the supplied layout direction.
    pub fn scroll_viewport_horizontal_rtl(&mut self, id: u64, b: crate::Rect, offset: &mut f32, max: f32, rtl: bool) -> bool {
        self.scroll_viewport_axis(id, b, offset, max, true, rtl)
    }

    fn scroll_viewport_axis(&mut self, id: u64, b: crate::Rect, offset: &mut f32, max: f32, horizontal: bool, rtl: bool) -> bool {
        let input = self.input;
        let inside = self.pointer_in(b);
        let reduced = self.reduced_motion();
        let wheel = if horizontal && inside { if input.scroll[0] != 0.0 { if rtl {-input.scroll[0]}else{input.scroll[0]} } else { input.scroll[1] } } else if horizontal { 0.0 } else { self.scroll_y(b) };
        let state = self.ui.scrolls.entry(id).or_default();
        if horizontal {
            if state.horizontal_rtl.is_some_and(|previous|previous!=rtl) {
                state.cancel();
                *offset=offset.clamp(0.0,max);
            }
            state.horizontal_rtl=Some(rtl);
        }
        if input.key.is_some() || (state.offset - *offset).abs() > 0.1 {
            state.cancel();
        }
        state.offset = *offset;
        if wheel != 0. {
            state.cancel();
            state.offset = (state.offset + wheel).clamp(0., max);
        }
        if input.pointer_kind == crate::PointerKind::Touch {
            let coordinate = |p: [f32; 2]| if horizontal && rtl { -p[0] } else { p[usize::from(!horizontal)] };
            if input.pressed && inside {
                state.begin(coordinate(input.pointer.unwrap()), input.time);
            }
            if let Some(p) = input.pointer {
                state.drag(coordinate(p), input.time, max, if horizontal { b.width } else { b.height });
            }
        }
        let dragged = state.dragging;
        if input.released {
            state.release(input.time, reduced);
        }
        if input.cancelled || input.blurred {
            state.cancel();
        }
        let active = state.advance(input.time, max, reduced);
        *offset = state.offset;
        if active {
            self.continuous(b);
        }
        if dragged {
            self.input.pressed = false;
            self.input.released = false;
            self.input.cancelled = true;
        }
        dragged
    }
}

/// Incremental AOSP spline for existing nested-scroll consumers (app bars/menus).
#[derive(Clone, Copy, Debug, Default)]
pub struct SplineFling {
    start: f64,
    duration: f32,
    distance: f32,
    previous: f32,
}
impl SplineFling {
    pub fn start(&mut self, velocity: f32, time: f64) {
        let friction = 0.015 * 9.80665_f32 * 39.37 * 160. * 0.84;
        let rate = 0.78_f32.ln() / 0.9_f32.ln();
        let l = (0.35 * velocity.abs().max(1.) / friction).ln();
        *self = Self {
            start: time,
            duration: (l / (rate - 1.)).exp(),
            distance: friction * (rate / (rate - 1.) * l).exp() * velocity.signum(),
            previous: 0.,
        };
    }
    pub fn step(&mut self, time: f64) -> (f32, bool) {
        let t = if self.duration > 0. {
            ((time - self.start) as f32 / self.duration).clamp(0., 1.)
        } else {
            1.
        };
        let position = self.distance * spline(t);
        let delta = position - self.previous;
        self.previous = position;
        (delta, t < 1.)
    }
}
