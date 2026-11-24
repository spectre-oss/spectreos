use crate::*;

/// Edge from which `Frame::slide` enters and exits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlideDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

/// Direction/style for dual-screen `Frame::route_transition`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteTransitionKind {
    Push,
    Pop,
    ModalUp,
    ModalDown,
}

/// Caller-owned animation state for `Frame::route_transition`.
#[derive(Clone, Copy, Debug)]
pub struct RouteTransitionState {
    pub kind: RouteTransitionKind,
    pub start: f64,
    pub duration: f64,
}

impl RouteTransitionState {
    pub fn start(kind: RouteTransitionKind, now: f64) -> Self {
        let duration = match kind {
            RouteTransitionKind::Push | RouteTransitionKind::ModalUp => 0.32,
            RouteTransitionKind::Pop | RouteTransitionKind::ModalDown => 0.24,
        };
        Self { kind, start: now, duration }
    }

    pub fn progress(&self, now: f64, reduced_motion: bool) -> f32 {
        if reduced_motion || self.duration <= 0.0 {
            return 1.0;
        }
        ((now - self.start) / self.duration).clamp(0.0, 1.0) as f32
    }
}

/// Direction of a page's entry relative to its final position.
#[derive(Clone, Copy, Debug, Default)]
pub enum PageDirection {
    #[default]
    None,
    Left,
    Right,
    Top,
    Bottom,
}

/// Caller-owned playback clock for a rotating shape.
#[derive(Default)]
pub struct ShapeRotationState {
    elapsed: f64,
    last: Option<f64>,
    generation: u64,
}
impl ShapeRotationState {
    pub fn pause(&mut self) {
        self.last = None;
    }
}

impl Frame<'_> {
    /// A caller-selected page with Beer CSS's 400ms fade and directional entry.
    pub fn page(
        &mut self,
        id: u64,
        active: bool,
        bounds: Rect,
        direction: PageDirection,
        build: impl FnOnce(&mut Self, Rect),
    ) -> f32 {
        use crate::timing::{Easing, TweenSpec};
        let spec = TweenSpec::new(0.4, Easing::Cubic([0.25, 0.1, 0.25, 1.0]));
        let amount = self.ui.motion.css_tween(
            id,
            86,
            f32::from(active),
            0.0,
            spec,
            TweenSpec::new(0.0, Easing::Linear),
        );
        if active && amount > 0.001 {
            let distance = 64.0 * (1.0 - amount);
            let offset = match direction {
                PageDirection::None => [0.0, 0.0],
                PageDirection::Left => [-distance, 0.0],
                PageDirection::Right => [distance, 0.0],
                PageDirection::Top => [0.0, -distance],
                PageDirection::Bottom => [0.0, distance],
            };
            let posed = Rect {
                x: bounds.x + offset[0],
                y: bounds.y + offset[1],
                ..bounds
            };
            self.within(posed, |f| f.with_opacity(amount, |f| build(f, posed)));
        }
        amount
    }
    /// Grow a bounded subtree using MUI's height-based timing and scale profile.
    pub fn grow(
        &mut self,
        id: u64,
        open: bool,
        bounds: Rect,
        build: impl FnOnce(&mut Self),
    ) -> (f32, f32) {
        use crate::timing::{Easing, TweenSpec};
        let factor = bounds.height.max(0.0) / 36.0;
        let duration = if factor == 0.0 {
            0.0
        } else {
            ((4.0 + 15.0 * factor.powf(0.25) + factor / 5.0) * 10.0)
                .round()
                .min(3000.0) as f64
                / 1000.0
        };
        let curve = Easing::Cubic([0.4, 0.0, 0.2, 1.0]);
        let target = f32::from(open);
        let opacity = self.ui.motion.css_tween(
            id,
            84,
            target,
            target,
            TweenSpec::new(duration, curve),
            TweenSpec::new(duration, curve),
        );
        let enter = TweenSpec::new(duration * 0.666, curve);
        let exit = TweenSpec {
            delay: duration * 0.333,
            duration: duration * 0.666,
            easing: curve,
        };
        let progress = self
            .ui
            .motion
            .css_tween(id, 85, target, target, enter, exit);
        let scale = 0.75 + 0.25 * progress;
        let vertical_ratio = (0.5625 + 0.4375 * progress) / scale;
        if opacity > 0.001 {
            let origin = [
                bounds.x + bounds.width / 2.0,
                bounds.y + bounds.height / 2.0,
            ];
            self.within(bounds, |f| {
                f.with_enabled(open, |f| {
                    let input = f.input;
                    if let Some(pointer) = &mut f.input.pointer {
                        pointer[1] = origin[1] + (pointer[1] - origin[1]) / vertical_ratio;
                    }
                    let nodes = f.ui.semantics.len();
                    f.commands.push(DrawCommand::PushScaleY {
                        origin: origin[1],
                        scale: vertical_ratio,
                    });
                    f.with_scale(origin, scale, |f| f.with_opacity(opacity, build));
                    f.commands.push(DrawCommand::PopScaleY);
                    for node in &mut f.ui.semantics[nodes..] {
                        node.bounds.y = origin[1] + (node.bounds.y - origin[1]) * vertical_ratio;
                        node.bounds.height *= vertical_ratio;
                    }
                    f.input = input;
                })
            });
        }
        (opacity, scale)
    }
    /// Slide a bounded subtree up from the lower viewport edge.
    pub fn slide_up(
        &mut self,
        id: u64,
        open: bool,
        bounds: Rect,
        build: impl FnOnce(&mut Self, Rect),
    ) -> f32 {
        self.slide(id, open, bounds, SlideDirection::Up, build)
    }
    /// Slide a bounded subtree in from any edge (`Up`, `Down`, `Left`, `Right`).
    pub fn slide(
        &mut self,
        id: u64,
        open: bool,
        bounds: Rect,
        direction: SlideDirection,
        build: impl FnOnce(&mut Self, Rect),
    ) -> f32 {
        use crate::timing::{Easing, TweenSpec};
        let target = f32::from(open);
        let amount = self.ui.motion.css_tween(
            id,
            83,
            target,
            target,
            TweenSpec::new(0.225, Easing::Cubic([0.0, 0.0, 0.2, 1.0])),
            TweenSpec::new(0.195, Easing::Cubic([0.4, 0.0, 0.6, 1.0])),
        );
        if amount > 0.001 {
            let viewport = self.viewport();
            let inv = 1.0 - amount;
            let posed = match direction {
                SlideDirection::Up => Rect {
                    y: bounds.y + (viewport.y + viewport.height - bounds.y) * inv,
                    ..bounds
                },
                SlideDirection::Down => Rect {
                    y: bounds.y + (viewport.y - bounds.height - bounds.y) * inv,
                    ..bounds
                },
                SlideDirection::Left => Rect {
                    x: bounds.x + (viewport.x + viewport.width - bounds.x) * inv,
                    ..bounds
                },
                SlideDirection::Right => Rect {
                    x: bounds.x + (viewport.x - bounds.width - bounds.x) * inv,
                    ..bounds
                },
            };
            self.within(posed, |f| f.with_enabled(open, |f| build(f, posed)));
        }
        amount
    }
    /// Collapse a subtree vertically using MUI's automatic height duration.
    pub fn collapse(
        &mut self,
        id: u64,
        open: bool,
        height: f32,
        build: impl FnOnce(&mut Self),
    ) -> f32 {
        use crate::timing::{Easing, TweenSpec};
        let height = height.max(0.0);
        let factor = height / 36.0;
        let duration = if height == 0.0 {
            0.0
        } else {
            ((4.0 + 15.0 * factor.powf(0.25) + factor / 5.0) * 10.0)
                .round()
                .min(3000.0) as f64
                / 1000.0
        };
        let spec = TweenSpec::new(duration, Easing::Cubic([0.4, 0.0, 0.2, 1.0]));
        let target = f32::from(open);
        let amount = self.ui.motion.css_tween(id, 82, target, target, spec, spec);
        let visible = self.reserve(height * amount);
        if amount > 0.001 {
            self.within(visible, |f| f.with_enabled(open, |f| build(f)));
        }
        amount
    }
    /// Scale a bounded subtree using MUI's default Zoom timings.
    pub fn zoom(
        &mut self,
        id: u64,
        open: bool,
        bounds: Rect,
        build: impl FnOnce(&mut Self),
    ) -> f32 {
        use crate::timing::{Easing, TweenSpec};
        let easing = Easing::Cubic([0.4, 0.0, 0.2, 1.0]);
        let target = f32::from(open);
        let amount = self.ui.motion.css_tween(
            id,
            81,
            target,
            target,
            TweenSpec::new(0.225, easing),
            TweenSpec::new(0.195, easing),
        );
        if amount > 0.001 {
            let origin = [
                bounds.x + bounds.width / 2.0,
                bounds.y + bounds.height / 2.0,
            ];
            self.within(bounds, |f| {
                f.with_enabled(open, |f| f.with_scale(origin, amount, build))
            });
        }
        amount
    }
    /// Fade a bounded subtree using MUI's default enter and exit timings.
    pub fn fade(
        &mut self,
        id: u64,
        open: bool,
        bounds: Rect,
        build: impl FnOnce(&mut Self),
    ) -> f32 {
        use crate::timing::{Easing, TweenSpec};
        let easing = Easing::Cubic([0.4, 0.0, 0.2, 1.0]);
        let target = f32::from(open);
        let amount = self.ui.motion.css_tween(
            id,
            80,
            target,
            target,
            TweenSpec::new(0.225, easing),
            TweenSpec::new(0.195, easing),
        );
        if amount > 0.001 {
            self.within(bounds, |f| {
                f.with_enabled(open, |f| f.with_opacity(amount, build))
            });
        }
        amount
    }
    /// Scale a subtree's geometry, text, clipping, semantics and pointer coordinates together.
    pub fn with_scale(&mut self, origin: [f32; 2], scale: f32, build: impl FnOnce(&mut Self)) {
        let scale = scale.clamp(0.001, 4.0);
        let start = self.commands.len();
        let nodes = self.ui.semantics.len();
        let input = self.input;
        if let Some(p) = self.input.pointer {
            self.input.pointer = Some([
                origin[0] + (p[0] - origin[0]) / scale,
                origin[1] + (p[1] - origin[1]) / scale,
            ]);
        }
        build(self);
        self.input = input;
        let point = |p: [f32; 2]| {
            [
                origin[0] + (p[0] - origin[0]) * scale,
                origin[1] + (p[1] - origin[1]) * scale,
            ]
        };
        let rect = |b: &mut Rect| {
            let p = point([b.x, b.y]);
            b.x = p[0];
            b.y = p[1];
            b.width *= scale;
            b.height *= scale;
        };
        for command in &mut self.commands[start..] {
            match command {
                DrawCommand::Ellipse { bounds,.. } => rect(bounds),
                DrawCommand::Outline { bounds,radius,width,.. } => { rect(bounds); *radius*=scale; *width*=scale; },
                DrawCommand::RoundedRect { bounds, radius, .. } => {
                    rect(bounds);
                    *radius *= scale;
                }
                DrawCommand::Text { bounds, style, .. } => {
                    rect(bounds);
                    style.size *= scale;
                    style.padding *= scale;
                    style.letter_spacing *= scale;
                }
                DrawCommand::Image { bounds, radius, .. } => {
                    rect(bounds);
                    *radius *= scale;
                }
                DrawCommand::Shadow {
                    bounds,
                    radius,
                    offset,
                    blur,
                    spread,
                    ..
                } => {
                    rect(bounds);
                    *radius *= scale;
                    offset[0] *= scale;
                    offset[1] *= scale;
                    *blur *= scale;
                    *spread *= scale;
                }
                DrawCommand::Ripple {
                    bounds,
                    corner_radius,
                    center,
                    radius,
                    ..
                } => {
                    rect(bounds);
                    *corner_radius *= scale;
                    *center = point(*center);
                    *radius *= scale;
                }
                DrawCommand::Arc { bounds, stroke, .. } => {
                    rect(bounds);
                    *stroke *= scale;
                }
                DrawCommand::Line {
                    from, to, width, ..
                } => {
                    *from = point(*from);
                    *to = point(*to);
                    *width *= scale;
                }
                DrawCommand::Polyline { points, width, .. } => {
                    for p in points {
                        *p = point(*p);
                    }
                    *width *= scale;
                }
                DrawCommand::Loading { bounds, .. }
                | DrawCommand::Shape { bounds, .. }
                | DrawCommand::PushClip(bounds) => rect(bounds),
                _ => {}
            }
        }
        for node in &mut self.ui.semantics[nodes..] {
            rect(&mut node.bounds);
        }
    }
    pub fn with_opacity(&mut self, alpha: f32, build: impl FnOnce(&mut Self)) {
        let alpha = alpha.clamp(0.0, 1.0);
        let start = self.commands.len();
        self.with_enabled(alpha > 0.001, build);
        for command in &mut self.commands[start..] {
            match command {
                DrawCommand::RoundedRect { color, .. }
                | DrawCommand::Ellipse { color, .. }
                | DrawCommand::Outline { color, .. }
                | DrawCommand::Text { color, .. }
                | DrawCommand::Shadow { color, .. }
                | DrawCommand::Ripple { color, .. }
                | DrawCommand::Loading { color, .. }
                | DrawCommand::Shape { color, .. }
                | DrawCommand::Image { color, .. }
                | DrawCommand::Arc { color, .. }
                | DrawCommand::Line { color, .. }
                | DrawCommand::Polyline { color, .. } => *color = color.opacity(alpha),
                _ => {}
            }
        }
    }
    /// Animate Beer CSS's two wavy shapes. Other shapes remain static.
    pub fn animated_wavy_shape(
        &mut self,
        bounds: Rect,
        index: usize,
        color: Color,
        state: &mut ShapeRotationState,
    ) -> bool {
        let Some(shape) = crate::beercss_shapes::SHAPES.get(index) else {
            return false;
        };
        let circle = shape.name == "wavy-circle";
        if !circle && shape.name != "wavy" {
            return self.paint_shape(bounds, index, color);
        }
        let visible = bounds.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            state.pause();
            return false;
        }
        if state.generation != self.ui.pause_generation {
            state.pause();
            state.generation = self.ui.pause_generation;
        }
        let period = if circle { 8.0 } else { 1.0 };
        let phase = if self.reduced_motion() {
            state.pause();
            0.0
        } else {
            if let Some(last) = state.last {
                state.elapsed += (self.ui.time - last).max(0.0);
            }
            state.last = Some(self.ui.time);
            self.ui.continuous = true;
            (state.elapsed.rem_euclid(period) / period) as f32
        };
        if circle {
            self.commands.push(DrawCommand::Shape {
                bounds,
                index: index as u8,
                color,
                rotation: -phase * std::f32::consts::TAU,
            });
        } else {
            // The source SVG translates 312.5 units in a 1220-unit viewport.
            let shift = phase * bounds.width.min(bounds.height) * (312.5 / 1220.0);
            self.within(bounds, |f| {
                for x in [bounds.x + shift - bounds.width, bounds.x + shift] {
                    f.paint_shape(Rect { x, ..bounds }, index, color);
                }
            });
        }
        true
    }
    /// Paint a continuously rotating shape; duration is seconds per revolution.
    pub fn rotating_shape(
        &mut self,
        bounds: Rect,
        index: usize,
        color: Color,
        seconds: f64,
        state: &mut ShapeRotationState,
    ) -> bool {
        if index >= crate::beercss_shapes::SHAPES.len() || !seconds.is_finite() || seconds <= 0.0 {
            return false;
        }
        let visible = bounds.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            state.pause();
            return false;
        }
        if state.generation != self.ui.pause_generation {
            state.pause();
            state.generation = self.ui.pause_generation;
        }
        let rotation = if self.reduced_motion() {
            state.pause();
            0.0
        } else {
            if let Some(last) = state.last {
                state.elapsed += (self.ui.time - last).max(0.0);
            }
            state.last = Some(self.ui.time);
            self.ui.continuous = true;
            (state.elapsed.rem_euclid(seconds) / seconds * std::f64::consts::TAU) as f32
        };
        self.commands.push(DrawCommand::Shape {
            bounds,
            index: index as u8,
            color,
            rotation,
        });
        true
    }
    /// Reversible measured reveal: full child layout, clipped presentation and no exit input.
    pub fn reveal(&mut self, id: u64, open: bool, height: f32, build: impl FnOnce(&mut Self)) {
        let amount = self.spatial(id, 21, f32::from(open), 0.0);
        let alpha = self.animate(id, 22, f32::from(open), 0.0);
        if amount <= 0.001 {
            return;
        }
        let b = self.allocate(height * amount);
        if amount > 0.001 {
            self.within(b, |f| {
                f.with_enabled(open, |f| {
                    f.with_opacity(alpha, |f| f.within(Rect { height, ..b }, build))
                })
            });
        }
    }
    pub fn transition(
        &mut self,
        id: u64,
        property: u8,
        target: f32,
        initial: f32,
        duration: f64,
        easing: crate::timing::Easing,
    ) -> f32 {
        let spec = crate::timing::TweenSpec::new(duration, easing);
        self.ui
            .motion
            .tween(id, property, target, initial, spec, spec)
    }
    /// CSS transition sampling, including shortened reversals.
    pub fn transition_css(
        &mut self,
        id: u64,
        property: u8,
        target: f32,
        initial: f32,
        duration: f64,
        easing: crate::timing::Easing,
    ) -> f32 {
        let spec = crate::timing::TweenSpec::new(duration, easing);
        self.ui
            .motion
            .css_tween(id, property, target, initial, spec, spec)
    }

    /// Dual-screen Android / Material route transition (`Push`/`Pop` Shared-Axis X parallax with
    /// scrim & page-edge shadow, or `ModalUp`/`ModalDown` vertical slide+fade).
    /// Returns `true` while the transition is still animating.
    pub fn route_transition(
        &mut self,
        bounds: Rect,
        state: RouteTransitionState,
        outgoing: impl FnOnce(&mut Self),
        incoming: impl FnOnce(&mut Self),
    ) -> bool {
        let raw = state.progress(self.time(), self.reduced_motion());
        let active = raw < 1.0;
        if active {
            self.continuous(bounds);
        }
        let ease = match state.kind {
            RouteTransitionKind::Push | RouteTransitionKind::ModalUp => {
                crate::timing::Easing::EmphasizedDecelerate.sample(raw)
            }
            RouteTransitionKind::Pop | RouteTransitionKind::ModalDown => {
                crate::timing::Easing::EmphasizedAccelerate.sample(raw)
            }
        };
        let origin = [bounds.x, bounds.y];
        let paint_edge_shadow = |f: &mut Self, edge_x: f32, strength: f32| {
            if strength <= 0.01 {
                return;
            }
            for (w, a) in [(12.0_f32, 0.04_f32), (8.0, 0.07), (4.0, 0.11), (1.5, 0.16)] {
                let alpha = (a * strength * 255.0).round().clamp(0.0, 255.0) as u8;
                if alpha > 0 {
                    f.paint_rect(
                        Rect { x: edge_x - w, y: bounds.y, width: w, height: bounds.height },
                        0.0,
                        Color(0, 0, 0, alpha),
                    );
                }
            }
        };
        match state.kind {
            RouteTransitionKind::Push => {
                let out_dx = -bounds.width * 0.28 * ease;
                let in_dx = bounds.width * (1.0 - ease);
                self.within(bounds, |f| {
                    f.without_input(true, |f| {
                        f.push_transform(origin, [out_dx, 0.0], [1.0, 1.0], 0.0);
                        outgoing(f);
                        f.pop_transform();
                    });
                    f.paint_rect(bounds, 0.0, Color(0, 0, 0, ((0.22 * ease) * 255.0) as u8));
                    paint_edge_shadow(f, bounds.x + in_dx, 1.0 - ease);
                    f.push_transform(origin, [in_dx, 0.0], [1.0, 1.0], 0.0);
                    incoming(f);
                    f.pop_transform();
                });
            }
            RouteTransitionKind::Pop => {
                let in_dx = -bounds.width * 0.28 * (1.0 - ease);
                let out_dx = bounds.width * ease;
                self.within(bounds, |f| {
                    f.push_transform(origin, [in_dx, 0.0], [1.0, 1.0], 0.0);
                    incoming(f);
                    f.pop_transform();
                    f.paint_rect(bounds, 0.0, Color(0, 0, 0, ((0.22 * (1.0 - ease)) * 255.0) as u8));
                    paint_edge_shadow(f, bounds.x + out_dx, 1.0 - ease);
                    f.without_input(true, |f| {
                        f.push_transform(origin, [out_dx, 0.0], [1.0, 1.0], 0.0);
                        outgoing(f);
                        f.pop_transform();
                    });
                });
            }
            RouteTransitionKind::ModalUp => {
                let in_dy = bounds.height * 0.12 * (1.0 - ease);
                self.within(bounds, |f| {
                    f.without_input(true, outgoing);
                    f.push_transform(origin, [0.0, in_dy], [1.0, 1.0], 0.0);
                    f.push_layer(bounds, ease);
                    incoming(f);
                    f.pop_layer();
                    f.pop_transform();
                });
            }
            RouteTransitionKind::ModalDown => {
                let out_dy = bounds.height * 0.12 * ease;
                self.within(bounds, |f| {
                    incoming(f);
                    f.without_input(true, |f| {
                        f.push_transform(origin, [0.0, out_dy], [1.0, 1.0], 0.0);
                        f.push_layer(bounds, 1.0 - ease);
                        outgoing(f);
                        f.pop_layer();
                        f.pop_transform();
                    });
                });
            }
        }
        active
    }

    /// Predictive edge-back folding chevron indicator (`|` -> `<` / `>`) floating at edge `y`.
    pub fn edge_back_indicator(
        &mut self,
        bounds: Rect,
        from_right_edge: bool,
        y: f32,
        progress: f32,
        committed: bool,
    ) {
        let p = progress.clamp(0.0, 1.0);
        if p <= 0.01 {
            return;
        }
        let t = self.theme();
        let dir = if from_right_edge { -1.0 } else { 1.0 };
        let inset = 14.0 + 18.0 * p;
        let cx = if from_right_edge { bounds.x + bounds.width - inset } else { bounds.x + inset };
        let cy = y.clamp(bounds.y + 40.0, bounds.y + bounds.height - 40.0);
        let pill = Rect { x: cx - 18.0, y: cy - 22.0, width: 36.0, height: 44.0 };
        self.elevation(pill, 18.0, 2);
        self.paint_rect(
            pill,
            18.0,
            if committed { t.primary_container } else { t.surface_high }.opacity(p),
        );
        let stroke = if committed { 2.4 } else { 1.8 };
        let fg = if committed { t.on_primary_container } else { t.on_surface_variant }.opacity(p);
        let fold = 5.0 * p;
        self.paint_line([cx - dir * fold * 0.5, cy - 7.0], [cx + dir * fold * 0.5, cy], stroke, fg);
        self.paint_line([cx + dir * fold * 0.5, cy], [cx - dir * fold * 0.5, cy + 7.0], stroke, fg);
    }

    /// Material circular reveal transition expanding from `origin` across `bounds`.
    /// Calls `build(frame, reveal_radius)` inside `bounds` once `progress > 0.005`.
    pub fn circular_reveal(
        &mut self,
        bounds: Rect,
        origin: [f32; 2],
        progress: f32,
        backdrop: Color,
        build: impl FnOnce(&mut Self, f32),
    ) {
        let p = progress.clamp(0.0, 1.0);
        if p <= 0.005 {
            return;
        }
        let max_r = bounds.width.hypot(bounds.height) * 1.04;
        let reveal_r = 16.0 + (max_r - 16.0) * p;
        self.within(bounds, |f| {
            if p >= 0.995 {
                f.paint_rect(bounds, 0.0, backdrop);
            } else {
                let disc = Rect {
                    x: origin[0] - reveal_r,
                    y: origin[1] - reveal_r,
                    width: reveal_r * 2.0,
                    height: reveal_r * 2.0,
                };
                f.paint_rect(disc, reveal_r, backdrop);
            }
            build(f, reveal_r);
        });
    }

    /// Generic Material container-transform morph interpolating rectangle bounds, corner radius,
    /// surface fill color, and cross-fading outgoing & incoming content.
    #[allow(clippy::too_many_arguments)]
    pub fn container_transform(
        &mut self,
        from: Rect,
        to: Rect,
        from_radius: f32,
        to_radius: f32,
        from_bg: Color,
        to_bg: Color,
        progress: f32,
        from_content: impl FnOnce(&mut Self, Rect),
        to_content: impl FnOnce(&mut Self, Rect),
    ) {
        let p = progress.clamp(0.0, 1.0);
        let lerp = |a: f32, b: f32| a + (b - a) * p;
        let current = Rect {
            x: lerp(from.x, to.x),
            y: lerp(from.y, to.y),
            width: lerp(from.width, to.width).max(0.0),
            height: lerp(from.height, to.height).max(0.0),
        };
        let radius = lerp(from_radius, to_radius).max(0.0);
        let bg = from_bg.mix(to_bg, p);
        self.paint_rect(current, radius, bg);
        let out_alpha = (1.0 - p * 3.0).clamp(0.0, 1.0);
        let in_alpha = ((p - 0.25) / 0.75).clamp(0.0, 1.0);
        self.within(current, |f| {
            if out_alpha > 0.01 {
                f.with_opacity(out_alpha, |f| from_content(f, current));
            }
            if in_alpha > 0.01 {
                f.with_opacity(in_alpha, |f| to_content(f, current));
            }
        });
    }
}

#[cfg(test)]
#[path = "transitions.spec.rs"]
mod tests;
