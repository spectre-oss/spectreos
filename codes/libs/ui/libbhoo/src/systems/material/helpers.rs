use crate::*;

impl Frame<'_> {
    pub fn viewport(&self) -> Rect {
        self.ui.viewport.unwrap_or(self.clip)
    }
    pub fn without_input(&mut self, blocked: bool, build: impl FnOnce(&mut Self)) {
        let previous = self.input;
        let previous_feedback = self.feedback_blocked;
        self.feedback_blocked |= blocked;
        if blocked {
            self.input = Input {
                time: previous.time,
                ..Input::default()
            };
        }
        build(self);
        self.input = previous;
        self.feedback_blocked = previous_feedback;
    }
    /// Block only pointer and wheel input (preserving keyboard focus and key events)
    /// when a higher-z sibling or clickable descendant owns the pointer location.
    pub fn without_pointer_input<R>(&mut self, blocked: bool, build: impl FnOnce(&mut Self) -> R) -> R {
        let previous = self.input;
        if blocked {
            self.input.pointer = None;
            self.input.pressed = false;
            self.input.released = false;
            self.input.scroll = [0.0, 0.0];
        }
        let result = build(self);
        self.input = previous;
        result
    }
    /// Cancel active child pointer capture when a parent gesture (e.g. scroll/drag) takes over:
    /// clears `active` so releasing cannot trigger a click, while letting already-started
    /// Material ripples fade out smoothly (`ACTION_CANCEL` semantics).
    pub fn cancel_pointer_input<R>(&mut self, cancelled: bool, build: impl FnOnce(&mut Self) -> R) -> R {
        let previous = self.input;
        if cancelled {
            self.ui.active = None;
            self.input.cancelled = true;
            self.input.pointer = None;
            self.input.pressed = false;
            self.input.released = false;
            self.input.scroll = [0.0, 0.0];
        }
        let result = build(self);
        self.input = previous;
        result
    }
    pub(crate) fn any_scroll_dragging(&self) -> bool {
        self.ui.scrolls.values().any(|s| s.dragging)
    }
    pub fn paint_rect(&mut self, bounds: Rect, radius: f32, color: Color) {
        self.rect(bounds, radius, color);
    }
    /// Draw a caller-owned RGBA image with centered cover/contain geometry.
    pub fn paint_image(&mut self, bounds: Rect, image: &Image, fit: ImageFit, radius: f32) -> bool {
        self.paint_tinted_image(bounds, image, fit, radius, Color(255, 255, 255, 255))
    }
    /// Draw a caller-owned RGBA or MSDF image tinted by `color`.
    pub fn paint_tinted_image(
        &mut self,
        bounds: Rect,
        image: &Image,
        fit: ImageFit,
        radius: f32,
        color: Color,
    ) -> bool {
        let Some((bounds, uv)) =
            crate::media::fit_image(bounds, image.width(), image.height(), fit)
        else {
            return false;
        };
        if bounds.intersect(self.clip).width <= 0.0 || bounds.intersect(self.clip).height <= 0.0 {
            return false;
        }
        self.commands.push(DrawCommand::Image {
            bounds,
            uv,
            radius: radius.max(0.0),
            color,
            image: image.clone(),
        });
        true
    }
    /// Paint one of the bundled static Beer CSS shapes, contained within bounds.
    pub fn paint_shape(&mut self, bounds: Rect, index: usize, color: Color) -> bool {
        if index >= crate::beercss_shapes::SHAPES.len()
            || bounds.width <= 0.0
            || bounds.height <= 0.0
        {
            return false;
        }
        self.commands.push(DrawCommand::Shape {
            bounds,
            index: index as u8,
            color,
            rotation: 0.0,
        });
        true
    }
    /// Renderer-independent soft radial primitive. Stateful controls use `ink` internally.
    pub fn paint_ink(
        &mut self,
        bounds: Rect,
        corner_radius: f32,
        center: [f32; 2],
        radius: f32,
        color: Color,
    ) {
        self.commands.push(DrawCommand::Ripple {
            bounds,
            corner_radius,
            center,
            radius,
            color,
            soft: true,
        });
    }
    /// Beer CSS's centered hover wave: a 200ms expanding hard-edged state layer.
    pub fn wave(&mut self, response: Response, radius: f32, color: Color) -> f32 {
        use crate::timing::{Easing, TweenSpec};
        let visible = response.enabled
            && !response.pressed
            && (response.hovered || response.focused && self.ui.keyboard_focus);
        let target = f32::from(visible);
        let progress = self.ui.motion.css_tween(
            response.id,
            87,
            target,
            0.0,
            TweenSpec::new(0.2, Easing::Linear),
            TweenSpec::new(0.0, Easing::Linear),
        );
        if visible {
            let b = response.bounds;
            let maximum = b.width.hypot(b.height) * 0.5;
            self.commands.push(DrawCommand::Ripple {
                bounds: b,
                corner_radius: radius,
                center: [b.x + b.width / 2.0, b.y + b.height / 2.0],
                radius: maximum * (0.01 + 2.24 * progress),
                color: color.opacity(0.1),
                soft: false,
            });
        }
        progress
    }
    pub fn paint_line(&mut self, from: [f32; 2], to: [f32; 2], width: f32, color: Color) {
        self.commands.push(DrawCommand::Line {
            from,
            to,
            width,
            color,
        });
    }
    /// Paint a circular arc with flat ends.
    pub fn paint_arc(&mut self, bounds: Rect, start: f32, sweep: f32, stroke: f32, color: Color) {
        self.commands.push(DrawCommand::Arc {
            bounds,
            start,
            sweep,
            stroke,
            color,
            rounded: false,
        });
    }
    pub fn paint_text(&mut self, bounds: Rect, text: &str, color: Color) {
        self.text_at(bounds, text, color);
    }
    pub fn action_at(&mut self, id: u64, bounds: Rect, label: &str, options: Options) -> Response {
        self.button_at(id, bounds, label, options)
    }
    /// An interaction/semantic target for a composition with custom presentation.
    pub fn target(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        enabled: bool,
        role: Role,
    ) -> Response {
        let r = self.interact(id, bounds, enabled);
        self.semantic(r, role, label, false, None, None, None);
        r
    }
    pub fn target_roving(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        enabled: bool,
        tab_stop: bool,
    ) -> Response {
        let r = self.target(id, bounds, label, enabled, Role::Button);
        if !tab_stop && !r.focused && !r.clicked && self.ui.next_focus_order.last() == Some(&id) {
            self.ui.next_focus_order.pop();
        }
        r
    }
    pub fn state_layer(&mut self, response: Response, radius: f32, color: Color) {
        let hover = self.animate(response.id, 0, f32::from(response.hovered), 0.0);
        let alpha = if response.focused && self.ui.keyboard_focus {
            0.12
        } else {
            0.08 * hover
        };
        if response.enabled && alpha > 0.001 {
            self.rect(response.bounds, radius, color.opacity(alpha));
        }
        self.ink(response, radius, color);
    }
    pub fn effect(&mut self, id: u64, property: u8, target: f32, initial: f32) -> f32 {
        self.animate(id, property, target, initial)
    }
    /// Bounded geometry transition using AndroidX Standard spatial spring tokens.
    pub fn spatial(&mut self, id: u64, property: u8, target: f32, initial: f32) -> f32 {
        self.ui.motion.effect(
            id,
            property,
            target,
            initial,
            self.ui.motion_scheme.spatial(),
        )
    }
    /// Scroll offset supplied while building a nested-scroll bottom sheet.
    pub fn bottom_sheet_scroll_offset(&self)->f32 {self.overlay_nested_scroll}
    pub fn request_focus(&mut self, id: u64) {
        if self.enabled {
            self.ui.focused = Some(id);
            self.ui.keyboard_focus = true;
        }
    }
    pub fn focused_id(&self) -> Option<u64> {
        self.ui.focused
    }
    /// Whether this response has a keyboard-visible focus state.
    pub fn keyboard_focused(&self, response: Response) -> bool {
        response.focused && self.ui.keyboard_focus
    }
    pub fn time(&self) -> f64 {
        self.ui.time
    }
    pub fn reduced_motion(&self) -> bool {
        self.ui.motion_preference == MotionPreference::Reduced
    }
    pub fn wake_at(&mut self, deadline: f64) {
        if deadline.is_finite() && deadline > self.ui.time {
            self.ui.wakeup = Some(self.ui.wakeup.map_or(deadline, |old| old.min(deadline)));
        }
    }
    pub fn continuous(&mut self, bounds: Rect) -> bool {
        let visible = bounds.intersect(self.clip);
        if !self.reduced_motion() && visible.width > 0.0 && visible.height > 0.0 {
            self.ui.continuous = true;
            true
        } else {
            false
        }
    }
    pub fn pressed_key(&self, key: Key) -> bool {
        self.enabled && self.input.key.is_some_and(|k| k.key == key && k.pressed)
    }
    pub fn pointer_in(&self, bounds: Rect) -> bool {
        self.input
            .pointer
            .is_some_and(|p| bounds.intersect(self.clip).contains(p))
    }
    pub fn scroll_y(&self, bounds: Rect) -> f32 {
        if self.enabled
            && self
                .input
                .pointer
                .is_some_and(|p| bounds.intersect(self.clip).contains(p))
        {
            self.input.scroll[1]
        } else {
            0.0
        }
    }
    pub fn compact_action_at(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        selected: bool,
    ) -> Response {
        let r = self.button_at(
            id,
            bounds,
            "",
            Options {
                selected,
                variant: ButtonVariant::Tonal,
                ..Options::default()
            },
        );
        let name = self.ui.text(label);
        if let Some(node) = self.ui.semantics.last_mut() {
            node.label = name;
        }
        self.text_at(
            Rect {
                x: bounds.x - 8.0,
                width: bounds.width + 8.0,
                ..bounds
            },
            label,
            self.theme()
                .text
                .opacity(if r.enabled { 1.0 } else { 0.38 }),
        );
        r
    }

    /// Paint a rectangle with independent top and bottom corner radii without a center seam.
    pub fn paint_asym_rect(&mut self, rect: Rect, top_radius: f32, bottom_radius: f32, color: Color) {
        if rect.width <= 0.0 || rect.height <= 0.0 || color.3 == 0 {
            return;
        }
        let max_r = (rect.width.min(rect.height) * 0.5).max(0.0);
        let tr = top_radius.clamp(0.0, max_r);
        let br = bottom_radius.clamp(0.0, max_r);
        if (tr - br).abs() < 0.25 {
            self.paint_rect(rect, tr, color);
            return;
        }
        let mid_y = (rect.y + rect.height * 0.5).floor();
        let top_h = (mid_y - rect.y).max(0.0);
        let bot_h = (rect.y + rect.height - mid_y).max(0.0);
        if top_h > 0.0 {
            self.within(Rect { height: top_h, ..rect }, |f| {
                f.paint_rect(
                    Rect {
                        height: rect.height.max(tr * 2.0),
                        ..rect
                    },
                    tr,
                    color,
                );
            });
        }
        if bot_h > 0.0 {
            self.within(
                Rect {
                    y: mid_y,
                    height: bot_h,
                    ..rect
                },
                |f| {
                    let h = rect.height.max(br * 2.0);
                    f.paint_rect(
                        Rect {
                            y: rect.y + rect.height - h,
                            height: h,
                            ..rect
                        },
                        br,
                        color,
                    );
                },
            );
        }
    }

    /// Paint an anti-aliased 4-corner inverse rounded mask over `bounds` using `outer_color`.
    /// Useful for clipping rectangular content to rounded viewport/bezel corners.
    pub fn paint_rounded_screen_mask(&mut self, bounds: Rect, corner_radius: f32, outer_color: Color) {
        use std::sync::{Mutex, OnceLock};
        let r_px = (corner_radius.round() as u32).clamp(8, 36);
        let key = (r_px, [outer_color.0, outer_color.1, outer_color.2, outer_color.3]);
        static MASK_CACHE: OnceLock<Mutex<HashMap<(u32, [u8; 4]), [Image; 4]>>> = OnceLock::new();
        let lock = MASK_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let corners: [Image; 4] = {
            let mut map = lock.lock().expect("corner mask cache lock");
            if let Some(existing) = map.get(&key) {
                existing.clone()
            } else {
                if map.len() > 32 {
                    map.clear();
                }
                let sz = r_px;
                let rf = r_px as f32;
                let make_corner = |cx: f32, cy: f32| -> Image {
                    let mut rgba = vec![0u8; (sz * sz * 4) as usize];
                    for y in 0..sz {
                        for x in 0..sz {
                            let dx = (x as f32 + 0.5) - cx;
                            let dy = (y as f32 + 0.5) - cy;
                            let dist = dx.hypot(dy);
                            let cov = (dist - (rf - 0.5)).clamp(0.0, 1.0);
                            let a = (cov * outer_color.3 as f32 + 0.5) as u8;
                            let o = ((y * sz + x) * 4) as usize;
                            rgba[o] = outer_color.0;
                            rgba[o + 1] = outer_color.1;
                            rgba[o + 2] = outer_color.2;
                            rgba[o + 3] = a;
                        }
                    }
                    Image::rgba(sz, sz, rgba).expect("valid corner mask")
                };
                let arr = [
                    make_corner(rf, rf),
                    make_corner(0.0, rf),
                    make_corner(rf, 0.0),
                    make_corner(0.0, 0.0),
                ];
                map.insert(key, arr.clone());
                arr
            }
        };
        let s = r_px as f32;
        let rects = [
            Rect { x: bounds.x, y: bounds.y, width: s, height: s },
            Rect { x: bounds.x + bounds.width - s, y: bounds.y, width: s, height: s },
            Rect { x: bounds.x, y: bounds.y + bounds.height - s, width: s, height: s },
            Rect { x: bounds.x + bounds.width - s, y: bounds.y + bounds.height - s, width: s, height: s },
        ];
        for i in 0..4 {
            self.paint_image(rects[i], &corners[i], ImageFit::Cover, 0.0);
        }
    }

    /// Paint a Material 3 tonal surface with elevation tint (`background.mix(primary, 0.03 * level)`)
    /// and drop shadow for `level > 0`.
    pub fn surface(&mut self, bounds: Rect, radius: f32, level: u8) {
        let t = self.theme();
        if level > 0 {
            self.elevation(bounds, radius, level);
        }
        let tint = (0.03 * f32::from(level.min(5))).clamp(0.0, 0.2);
        self.paint_rect(bounds, radius, t.background.mix(t.primary, tint));
    }

    /// Lightweight arc-based rounded corner clip mask using `bg` without allocating textures.
    /// Useful for clipping arbitrary sub-viewports (e.g. thumbnails or cards).
    pub fn mask_rounded_corners(&mut self, rect: Rect, radius: f32, bg: Color) {
        let r = radius.min(rect.width * 0.5).min(rect.height * 0.5);
        if r <= 0.5 {
            return;
        }
        let s = r * 0.48;
        let outer = r + s;
        let d = outer * 2.0;
        let half_pi = std::f32::consts::FRAC_PI_2;
        let pi = std::f32::consts::PI;
        self.within(rect, |f| {
            f.paint_arc(Rect { x: rect.x + r - outer, y: rect.y + r - outer, width: d, height: d }, pi, half_pi, s, bg);
            f.paint_arc(Rect { x: rect.x + rect.width - r - outer, y: rect.y + r - outer, width: d, height: d }, -half_pi, half_pi, s, bg);
            f.paint_arc(Rect { x: rect.x + rect.width - r - outer, y: rect.y + rect.height - r - outer, width: d, height: d }, 0.0, half_pi, s, bg);
            f.paint_arc(Rect { x: rect.x + r - outer, y: rect.y + rect.height - r - outer, width: d, height: d }, half_pi, half_pi, s, bg);
        });
    }
}

#[cfg(test)]
#[path = "helpers.spec.rs"]
mod tests;
