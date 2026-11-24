use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    #[default]
    Filled,
    Outlined,
    Elevated,
}

#[derive(Clone, Copy, Debug)]
pub struct CardOptions {
    pub variant: CardVariant,
    pub interactive: bool,
    pub enabled: bool,
}
impl Default for CardOptions {
    fn default() -> Self {
        Self {
            variant: CardVariant::Filled,
            interactive: false,
            enabled: true,
        }
    }
}

/// Caller-owned visual states of a Material card.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardIconGravity { TopStart, #[default] TopEnd, BottomStart, BottomEnd }
#[derive(Clone, Copy, Debug)]
pub struct CardVisualState {
    pub dragged: bool,
    pub checked: bool,
    pub checkable: bool,
    pub rtl: bool,
    pub checked_icon_size: f32,
    pub checked_icon_margin: f32,
    pub checked_icon_gravity: CardIconGravity,
    pub checked_icon_tint: Option<Color>,
}
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CardAppearance {
    pub background:Option<Color>,
    pub foreground:Option<Color>,
    pub stroke_color:Option<Color>,
    pub stroke_width:Option<f32>,
    pub corner_radius:Option<f32>,
    pub elevation:Option<f32>,
    pub ripple_color:Option<Color>,
}
impl Default for CardVisualState {
    fn default()->Self { Self {dragged:false,checked:false,checkable:false,rtl:false,checked_icon_size:24.0,checked_icon_margin:8.0,checked_icon_gravity:CardIconGravity::TopEnd,checked_icon_tint:None} }
}

impl Frame<'_> {
    /// Paint a passive or actionable card around caller-provided content.
    /// Child targets are processed first, so nested action clicks do not
    /// also activate an enclosing actionable card.
    pub fn card(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        options: CardOptions,
        build: impl FnOnce(&mut Self),
    ) -> Option<Response> {
        self.card_with_dragged(id,bounds,label,options,false,build)
    }
    /// Paint a card with the caller-owned Material dragged state.
    pub fn card_with_dragged(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        options: CardOptions,
        dragged: bool,
        build: impl FnOnce(&mut Self),
    ) -> Option<Response> {
        self.card_with_state(id,bounds,label,options,CardVisualState {dragged,..CardVisualState::default()},build)
    }
    /// Paint caller-controlled checked and dragged card states.
    pub fn card_with_state(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        options: CardOptions,
        state: CardVisualState,
        build: impl FnOnce(&mut Self),
    ) -> Option<Response> {
        self.card_with_state_inset(id,bounds,label,options,state,16.0,CardAppearance::default(),build)
    }
    pub(crate) fn card_with_state_inset(
        &mut self,
        id:u64,
        bounds:Rect,
        label:&str,
        options:CardOptions,
        state:CardVisualState,
        inset:f32,
        appearance:CardAppearance,
        build:impl FnOnce(&mut Self),
    ) -> Option<Response> {
        if !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width <= 0.0
            || bounds.height <= 0.0
        {
            return None;
        }
        let t = self.ui.theme;
        let radius=appearance.corner_radius.filter(|v|v.is_finite()).unwrap_or(12.0).clamp(0.0,bounds.width.min(bounds.height)*0.5);
        let enabled = options.enabled && self.enabled;
        let hovered = options.interactive
            && enabled
            && self.input.pointer_kind != PointerKind::Touch
            && self
                .input
                .pointer
                .is_some_and(|p| bounds.intersect(self.clip).contains(p));
        let pressed = self.ui.active == Some(id);
        let drag_target=if state.dragged && enabled {1.0} else {0.0};
        let drag_progress=self.animate(id, 200, drag_target, drag_target).clamp(0.0,1.0);
        let checked_target=if state.checked {1.0} else {0.0};
        // Material Android fades the checked icon linearly using motion short2
        // (100 ms) on entry and short1 (50 ms) on exit; reversals shorten.
        let checked_progress=self.ui.motion.css_tween(id,201,checked_target,checked_target,
            crate::timing::TweenSpec::new(0.100,crate::timing::Easing::Linear),
            crate::timing::TweenSpec::new(0.050,crate::timing::Easing::Linear)).clamp(0.0,1.0);
        let elevation = match options.variant {
            CardVariant::Elevated => {
                if hovered && !pressed {
                    2
                } else {
                    1
                }
            }
            _ if hovered && !pressed => 1,
            _ => 0,
        };
        if let Some(dp)=appearance.elevation {
            self.elevation_dp(bounds,radius,dp);
        } else if elevation > 0 && (enabled || options.variant == CardVariant::Elevated) {
            self.elevation(bounds, radius, elevation);
        }
        if drag_progress>0.001 {
            self.shadow(bounds,radius,[0.0,3.0],5.0,-1.0,Color(0,0,0,255).opacity(0.2*drag_progress));
            self.shadow(bounds,radius,[0.0,6.0],10.0,0.0,Color(0,0,0,255).opacity(0.14*drag_progress));
            self.shadow(bounds,radius,[0.0,1.0],18.0,0.0,Color(0,0,0,255).opacity(0.12*drag_progress));
        }
        let surface = appearance.background.unwrap_or(match (options.variant, enabled) {
            (CardVariant::Filled, true) => t.surface_highest,
            (CardVariant::Filled, false) => t.surface_highest.mix(t.surface_variant,0.38),
            (CardVariant::Elevated, true) => t.surface_low,
            (CardVariant::Elevated, false) => t.background,
            (CardVariant::Outlined, _) => t.background,
        });
        let default_stroke=if !enabled {t.surface_low.mix(t.outline,0.12)} else if state.checked {t.secondary} else {t.outline_variant};
        let custom_stroke=appearance.stroke_width.is_some() || appearance.stroke_color.is_some();
        if custom_stroke {
            self.paint_rect(bounds,radius,surface);
            let width=appearance.stroke_width.unwrap_or(if options.variant==CardVariant::Outlined {1.0} else {0.0}).max(0.0).min(bounds.width.min(bounds.height)*0.5);
            let color=appearance.stroke_color.unwrap_or(default_stroke);
            if width>0.0 {self.commands.push(DrawCommand::Outline {bounds,radius,width,color});}
        } else if options.variant == CardVariant::Outlined {
            self.paint_rect(
                bounds,
                radius,
                default_stroke,
            );
            self.paint_rect(bounds.inset(1.0), (radius-1.0).max(0.0), surface);
        } else {
            self.paint_rect(bounds, radius, surface);
        }
        self.within(bounds.inset(inset), |f| {
            let previous=f.content_color;
            if !enabled {f.content_color=Some(previous.unwrap_or(t.text).opacity(if previous.is_some() {1.0} else {0.38}));}
            f.with_enabled(enabled, build);
            f.content_color=previous;
        });
        if let Some(color)=appearance.foreground {self.paint_rect(bounds,radius,color);}
        else if state.checked {self.paint_rect(bounds,radius,t.secondary.opacity(0.16));}
        else if state.dragged {self.paint_rect(bounds,radius,t.text.opacity(0.08));}
        if checked_progress>0.001 {
            let alpha=checked_progress;
            let size=if state.checked_icon_size.is_finite() {state.checked_icon_size.max(0.0)} else {0.0};
            let margin=if state.checked_icon_margin.is_finite() {state.checked_icon_margin.max(0.0)} else {0.0};
            if size>0.0 && bounds.width>=size+2.0*margin && bounds.height>=size+2.0*margin {
                let at_end=matches!(state.checked_icon_gravity,CardIconGravity::TopEnd|CardIconGravity::BottomEnd);
                let at_bottom=matches!(state.checked_icon_gravity,CardIconGravity::BottomStart|CardIconGravity::BottomEnd);
                let icon=Rect {x:if at_end != state.rtl {bounds.x+bounds.width-margin-size} else {bounds.x+margin},y:if at_bottom {bounds.y+bounds.height-margin-size} else {bounds.y+margin},width:size,height:size};
                let index=crate::beercss_shapes::SHAPES.len()+crate::icons::material_navigation::ICONS.len()+crate::icons::material_symbols::ICONS.len()+1+crate::shapes::android_clock_shapes::SHAPES.len();
                let tint=state.checked_icon_tint.unwrap_or(if enabled {t.secondary} else {t.outline.opacity(0.12)});
                self.commands.push(DrawCommand::Shape {bounds:icon,index:index as u8,color:tint.opacity(alpha),rotation:0.0});
            }
        }
        if options.interactive {
            let response = self.target(id, bounds, label, enabled, Role::Button);
            let default_ink=if state.checked && !response.pressed || state.checkable && !state.checked && response.pressed {t.secondary} else {t.on_surface_variant};
            self.state_layer(response, radius, appearance.ripple_color.unwrap_or(default_ink));
            self.focus_ring(response);
            Some(response)
        } else {
            None
        }
    }

    /// Horizontal swipe-to-dismiss card container with asymmetric top/bottom corner radii,
    /// opacity fade during drag, threshold commit, and smooth spring return when released early.
    /// Returns `true` on the frame the item is dismissed.
    #[allow(clippy::too_many_arguments)]
    pub fn swipe_dismiss_box(
        &mut self,
        id: u64,
        bounds: Rect,
        state: &mut SwipeDismissState,
        top_radius: f32,
        bottom_radius: f32,
        build: impl FnOnce(&mut Self, Rect),
    ) -> bool {
        if state.dismissed {
            return false;
        }
        let mut just_dismissed = false;
        let threshold = (bounds.width * 0.42).max(72.0);
        if let Some(p) = self.input.pointer {
            if self.input.pressed && bounds.contains(p) {
                state.dragging = true;
                state.start_x = p[0] - state.offset;
            }
            if state.dragging {
                state.offset = p[0] - state.start_x;
            }
        }
        if state.dragging && (self.input.released || self.input.cancelled || self.input.blurred) {
            state.dragging = false;
            if state.offset.abs() >= threshold {
                state.dismissed = true;
                just_dismissed = true;
            }
        }
        let display_offset = if state.dragging || state.dismissed {
            state.offset
        } else {
            let anim = self.spatial(id, 77, 0.0, state.offset);
            if anim.abs() < 0.25 {
                state.offset = 0.0;
                0.0
            } else {
                state.offset = anim;
                anim
            }
        };
        let alpha = (1.0 - (display_offset.abs() / (bounds.width.max(1.0) * 0.85))).clamp(0.15, 1.0);
        let shifted = Rect { x: bounds.x + display_offset, ..bounds };
        let t = self.theme();
        self.with_opacity(alpha, |f| {
            f.paint_asym_rect(shifted, top_radius, bottom_radius, t.surface_low);
            f.within(shifted, |f| build(f, shifted));
        });
        just_dismissed
    }
}

/// Caller-owned horizontal swipe-to-dismiss state for `Frame::swipe_dismiss_box`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SwipeDismissState {
    pub offset: f32,
    pub dragging: bool,
    pub dismissed: bool,
    start_x: f32,
}

impl SwipeDismissState {
    /// Normalized detach progress in `0.0..=1.0` for morphing adjacent segmented card corners.
    pub fn detach_progress(&self) -> f32 {
        if self.dismissed {
            1.0
        } else {
            (self.offset.abs() / 24.0).clamp(0.0, 1.0)
        }
    }
}

/// Compute `(top_radius, bottom_radius)` for an item at `index` inside a segmented stacked card group
/// (M3 Expressive pattern: `outer` at group ends, `inner` between touching cards, dynamically
/// rounding to `outer` as a card or its neighbor detaches).
pub fn segmented_card_radii(
    index: usize,
    count: usize,
    outer: f32,
    inner: f32,
    prev_detach: f32,
    self_detach: f32,
    next_detach: f32,
) -> (f32, f32) {
    let base_top = if index == 0 { outer } else { inner };
    let base_bot = if index + 1 >= count { outer } else { inner };
    let t_top = prev_detach.max(self_detach).clamp(0.0, 1.0);
    let t_bot = next_detach.max(self_detach).clamp(0.0, 1.0);
    (base_top + (outer - base_top) * t_top, base_bot + (outer - base_bot) * t_bot)
}

#[cfg(test)]
#[path = "card.spec.rs"]
mod tests;
