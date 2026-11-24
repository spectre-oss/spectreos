use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TopAppBarVariant {
    #[default]
    Small,
    CenterAligned,
    Medium,
    Large,
}
impl TopAppBarVariant {
    pub fn expanded_height(self) -> f32 {
        match self {
            Self::Small | Self::CenterAligned => 64.0,
            Self::Medium => 112.0,
            Self::Large => 152.0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TopAppBarScrollBehavior {
    #[default]
    Pinned,
    EnterAlways,
    ExitUntilCollapsed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BottomAppBarScrollBehavior {
    #[default]
    Pinned,
    ExitAlways,
}

/// Caller-owned bottom-bar offset, updated from content-consumed deltas.
#[derive(Clone, Copy, Debug, Default)]
pub struct BottomAppBarState {
    hidden: f32,
    settle: crate::timing::Tween,
    settle_target: Option<f32>,
    settle_height: f32,
}

/// Persistent menu and scroll position for a bottom bar's overflow actions.
#[derive(Default)]
pub struct BottomAppBarOverflowState {
    pub overlay: OverlayState,
    offset: f32,
    drag_origin: Option<f32>,
    drag_last: Option<f32>,
    dragging: bool,
    drag_velocity: f32,
    drag_sample_time: f64,
    drag_motion_time: f64,
    fling_velocity: f32,
    fling_curve: crate::scroll::SplineFling,
    typeahead: MenuTypeaheadState,
}

impl BottomAppBarState {
    pub fn hidden_offset(&self) -> f32 {
        self.hidden
    }
    /// Positive consumed scroll hides the bar; negative scroll reveals it.
    pub fn apply_scroll(
        &mut self,
        behavior: BottomAppBarScrollBehavior,
        consumed: f32,
        height: f32,
    ) {
        let height = if height.is_finite() {
            height.max(0.0)
        } else {
            0.0
        };
        if behavior == BottomAppBarScrollBehavior::Pinned {
            self.hidden = 0.0;
            self.settle_target = None;
        } else if consumed.is_finite() {
            if consumed != 0.0 {
                self.settle_target = None;
            }
            self.hidden = (self.hidden + consumed).clamp(0.0, height);
        } else {
            self.hidden = self.hidden.clamp(0.0, height);
        }
    }
}
/// Caller-owned scroll offset. The host routes nested scroll deltas explicitly.
#[derive(Clone, Copy, Debug, Default)]
pub struct TopAppBarState {
    offset: f32,
    drag_origin: Option<f32>,
    drag_last: Option<f32>,
    dragging: bool,
    drag_sample_time: f64,
    drag_motion_time: f64,
    drag_velocity: f32,
    fling_velocity: f32,
    fling_curve: crate::scroll::SplineFling,
    fling_variant: Option<TopAppBarVariant>,
    snap: crate::timing::Tween,
    snap_target: Option<f32>,
    snap_variant: Option<TopAppBarVariant>,
}
impl TopAppBarState {
    fn start_snap(
        &mut self,
        variant: TopAppBarVariant,
        behavior: TopAppBarScrollBehavior,
        time: f64,
        reduced: bool,
    ) -> bool {
        use crate::timing::{Easing, TweenSpec};
        if behavior == TopAppBarScrollBehavior::Pinned {
            return false;
        }
        let fraction = self.collapse_fraction(variant);
        if !(0.001..0.999).contains(&fraction) {
            return false;
        }
        let target = if fraction < 0.5 {
            0.0
        } else {
            variant.expanded_height() - 64.0
        };
        self.snap = crate::timing::Tween::at(self.offset);
        self.snap_target = Some(target);
        self.snap_variant = Some(variant);
        let spec = TweenSpec::new(0.25, Easing::Standard);
        self.offset = self.snap.sample(time, target, spec, spec, reduced);
        if !self.snap.moving() {
            self.snap_target = None;
        }
        self.snap.moving()
    }
    pub fn collapse_fraction(&self, variant: TopAppBarVariant) -> f32 {
        let range = variant.expanded_height() - 64.0;
        if range <= 0.0 {
            0.0
        } else {
            (self.offset / range).clamp(0.0, 1.0)
        }
    }
    pub fn set_collapse_fraction(&mut self, variant: TopAppBarVariant, fraction: f32) {
        self.snap_target = None;
        self.fling_velocity = 0.0;
        self.offset = if fraction.is_finite() {
            fraction.clamp(0.0, 1.0) * (variant.expanded_height() - 64.0)
        } else {
            0.0
        };
    }
    /// Positive deltas collapse; negative deltas expand. Returns consumed pixels.
    pub fn apply_scroll(
        &mut self,
        variant: TopAppBarVariant,
        behavior: TopAppBarScrollBehavior,
        delta: f32,
        content_at_start: bool,
    ) -> f32 {
        let max = variant.expanded_height() - 64.0;
        if delta.is_finite() && delta != 0.0 {
            self.snap_target = None;
        }
        self.offset = self.offset.clamp(0.0, max);
        if !delta.is_finite()
            || behavior == TopAppBarScrollBehavior::Pinned
            || (delta < 0.0
                && behavior == TopAppBarScrollBehavior::ExitUntilCollapsed
                && !content_at_start)
        {
            return 0.0;
        }
        let old = self.offset;
        self.offset = (self.offset + delta).clamp(0.0, max);
        self.offset - old
    }

    /// Route a wheel/drag delta between the bar and caller-owned content.
    /// Positive deltas move toward later content. The return value is the
    /// consumed delta, allowing the host to pass any remainder to a parent.
    pub fn route_scroll(
        &mut self,
        variant: TopAppBarVariant,
        behavior: TopAppBarScrollBehavior,
        delta: f32,
        content_offset: &mut f32,
        content_max: f32,
    ) -> f32 {
        if !delta.is_finite() {
            return 0.0;
        }
        let maximum = if content_max.is_finite() {
            content_max.max(0.0)
        } else {
            0.0
        };
        *content_offset = if content_offset.is_finite() {
            content_offset.clamp(0.0, maximum)
        } else {
            0.0
        };
        let mut remaining = delta;
        let expand_first = delta >= 0.0 || behavior == TopAppBarScrollBehavior::EnterAlways;
        if expand_first {
            remaining -= self.apply_scroll(variant, behavior, remaining, *content_offset <= 0.0);
        }
        let previous = *content_offset;
        *content_offset = (*content_offset + remaining).clamp(0.0, maximum);
        remaining -= *content_offset - previous;
        if !expand_first {
            remaining -= self.apply_scroll(variant, behavior, remaining, *content_offset <= 0.0);
        }
        delta - remaining
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AppBarAction<'a> {
    pub label: &'a str,
    pub glyph: &'a str,
    pub enabled: bool,
}

impl Frame<'_> {
    /// Route wheel and touch-drag input from a content viewport into a top app
    /// bar and its caller-owned scroll offset. A drag remains captured after
    /// leaving the viewport and is cancelled on blur or pointer cancellation.
    #[allow(clippy::too_many_arguments)]
    pub fn top_app_bar_scroll_input(
        &mut self,
        bounds: Rect,
        state: &mut TopAppBarState,
        variant: TopAppBarVariant,
        behavior: TopAppBarScrollBehavior,
        content_offset: &mut f32,
        content_max: f32,
    ) -> f32 {
        use crate::timing::{Easing, TweenSpec};
        if state.snap_variant != Some(variant) || behavior == TopAppBarScrollBehavior::Pinned {
            state.snap_target = None;
        }
        if let Some(target) = state.snap_target {
            let spec = TweenSpec::new(0.25, Easing::Standard);
            state.offset =
                state
                    .snap
                    .sample(self.ui.time, target, spec, spec, self.reduced_motion());
            if state.snap.moving() {
                self.ui.continuous = true;
            } else {
                state.snap_target = None;
            }
        }
        let mut delta = self.scroll_y(bounds);
        let mut fling_consumed = 0.0;
        if self.input.pressed
            || self.input.cancelled
            || self.input.blurred
            || !self.enabled
            || delta != 0.0
            || self.reduced_motion()
            || state.fling_variant != Some(variant)
        {
            state.fling_velocity = 0.0;
        }
        if state.fling_velocity != 0.0 {
            // AOSP spline feeds the nested app-bar/content scroll chain.
            let (movement, moving) = state.fling_curve.step(self.ui.time);
            fling_consumed =
                state.route_scroll(variant, behavior, movement, content_offset, content_max);
            if !moving { state.fling_velocity = 0.; }
            if state.fling_velocity.abs() < 20.0 || (movement - fling_consumed).abs() > 0.5 {
                state.fling_velocity = 0.0;
                self.ui.continuous |=
                    state.start_snap(variant, behavior, self.ui.time, self.reduced_motion());
            } else {
                self.ui.continuous = true;
            }
        }
        if self.enabled && self.input.pointer_kind == PointerKind::Touch {
            if self.input.pressed && self.pointer_in(bounds) {
                state.drag_origin = self.input.pointer.map(|p| p[1]);
                state.drag_last = state.drag_origin;
                state.dragging = false;
                state.drag_velocity = 0.0;
                state.drag_sample_time = self.ui.time;
                state.drag_motion_time = self.ui.time;
            }
            if let (Some(origin), Some(last), Some(pointer)) =
                (state.drag_origin, state.drag_last, self.input.pointer)
            {
                if !state.dragging && (pointer[1] - origin).abs() > 4.0 {
                    state.dragging = true;
                }
                if state.dragging {
                    let movement = last - pointer[1];
                    delta += movement;
                    if movement != 0.0 {
                        let elapsed = self.ui.time - state.drag_sample_time;
                        state.drag_velocity = if (0.005..=0.15).contains(&elapsed) {
                            (movement / elapsed as f32).clamp(-4000.0, 4000.0)
                        } else {
                            0.0
                        };
                        state.drag_sample_time = self.ui.time;
                        state.drag_motion_time = self.ui.time;
                    }
                    self.ui.active = None;
                }
                if state.dragging {
                    state.drag_last = Some(pointer[1]);
                }
            }
        }
        let finish_drag = state.dragging
            && self.input.released
            && !self.input.cancelled
            && !self.input.blurred
            && self.enabled;
        if self.input.pressed || delta != 0.0 {
            state.snap_target = None;
        }
        if self.input.released || self.input.cancelled || self.input.blurred || !self.enabled {
            state.drag_origin = None;
            state.drag_last = None;
            state.dragging = false;
        }
        let consumed = state.route_scroll(variant, behavior, delta, content_offset, content_max);
        if finish_drag {
            if !self.reduced_motion()
                && (self.ui.time - state.drag_motion_time) <= 0.1
                && state.drag_velocity.abs() >= 350.0
            {
                state.fling_velocity = state.drag_velocity;
                    state.fling_curve.start(state.fling_velocity,self.ui.time);
                    state.fling_variant = Some(variant);
                self.ui.continuous = true;
            } else {
                self.ui.continuous |=
                    state.start_snap(variant, behavior, self.ui.time, self.reduced_motion());
            }
        }
        if consumed != 0.0 {
            self.ui.continuous = true;
        }
        consumed + fling_consumed
    }

    /// Route arrow, Page Up/Down, Home and End keys while the scroll target has
    /// focus. Returns the delta consumed by the bar and content.
    #[allow(clippy::too_many_arguments)]
    pub fn top_app_bar_scroll_keyboard(
        &mut self,
        target_id: u64,
        bounds: Rect,
        state: &mut TopAppBarState,
        variant: TopAppBarVariant,
        behavior: TopAppBarScrollBehavior,
        content_offset: &mut f32,
        content_max: f32,
    ) -> f32 {
        let visible = bounds.intersect(self.clip);
        if !self.enabled
            || self.key_consumed
            || self.ui.focused != Some(target_id)
            || visible.width <= 0.0
            || visible.height <= 0.0
        {
            return 0.0;
        }
        let Some(key) = self.input.key.filter(|k| k.pressed) else {
            return 0.0;
        };
        let range = if content_max.is_finite() {
            content_max.max(0.0)
        } else {
            0.0
        };
        let full = range + variant.expanded_height() - 64.0;
        let delta = match key.key {
            Key::Down => 48.0,
            Key::Up => -48.0,
            Key::PageDown => visible.height,
            Key::PageUp => -visible.height,
            Key::Home => -full,
            Key::End => full,
            _ => return 0.0,
        };
        self.key_consumed = true;
        state.snap_target = None;
        state.fling_velocity = 0.0;
        let consumed = state.route_scroll(variant, behavior, delta, content_offset, content_max);
        if consumed != 0.0 {
            self.ui.continuous = true;
        }
        consumed
    }

    /// Material-inspired app bar. Action 0 is navigation; later actions are
    /// indexed from one. Scroll offsets are supplied by the caller.
    #[allow(clippy::too_many_arguments)]
    pub fn top_app_bar(
        &mut self,
        id: u64,
        title: &str,
        variant: TopAppBarVariant,
        state: &TopAppBarState,
        navigation: Option<AppBarAction<'_>>,
        actions: &[AppBarAction<'_>],
        rtl: bool,
    ) -> Option<usize> {
        self.top_app_bar_with_icons(id, title, variant, state, navigation, actions, rtl, &[])
    }

    /// Icon entries correspond to navigation followed by the trailing actions;
    /// missing entries fall back to the caller-provided glyph.
    #[allow(clippy::too_many_arguments)]
    pub fn top_app_bar_with_icons(
        &mut self,
        id: u64,
        title: &str,
        variant: TopAppBarVariant,
        state: &TopAppBarState,
        navigation: Option<AppBarAction<'_>>,
        actions: &[AppBarAction<'_>],
        rtl: bool,
        icons: &[Option<AppBarIcon>],
    ) -> Option<usize> {
        let fraction = state.collapse_fraction(variant);
        let expanded_height = variant.expanded_height();
        let height = expanded_height - (expanded_height - 64.0) * fraction;
        let bar = self.reserve(height);
        let t = self.ui.theme;
        self.paint_rect(
            bar,
            0.0,
            t.background.mix(t.surface_highest, 0.12 * fraction),
        );
        let mut selected = None;
        let nav_width = if navigation.is_some() && bar.width >= 48.0 {
            48.0
        } else {
            0.0
        };
        let count = actions
            .len()
            .min(((bar.width - nav_width).max(0.0) / 48.0) as usize);
        let paint_action = |f: &mut Self, index: usize, action: AppBarAction<'_>, x: f32| -> bool {
            let bounds = Rect {
                x,
                y: bar.y + 8.0,
                width: 48.0,
                height: 48.0,
            };
            let response = f.interact(id.wrapping_add(index as u64), bounds, action.enabled);
            let color = t
                .on_surface_variant
                .opacity(if response.enabled { 1.0 } else { 0.38 });
            f.state_layer(response, 24.0, color);
            if let Some(Some(icon)) = icons.get(index) {
                f.commands.push(DrawCommand::Shape {
                    bounds: bounds.inset(12.0),
                    index: icon.shape_index(rtl),
                    color,
                    rotation: 0.0,
                });
            } else {
                f.styled_text(
                    bounds,
                    action.glyph,
                    color,
                    TextStyle {
                        size: 22.0,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
            }
            f.semantic(
                response,
                Role::Button,
                action.label,
                false,
                None,
                None,
                None,
            );
            f.focus_ring(response);
            response.clicked
        };
        if let Some(nav) = navigation.filter(|_| nav_width > 0.0) {
            let x = if rtl { bar.x + bar.width - 48.0 } else { bar.x };
            if paint_action(self, 0, nav, x) {
                selected = Some(0);
            }
        }
        for (i, action) in actions.iter().take(count).enumerate() {
            let x = if rtl {
                bar.x + i as f32 * 48.0
            } else {
                bar.x + bar.width - (i + 1) as f32 * 48.0
            };
            if paint_action(self, i + 1, *action, x) {
                selected = Some(i + 1);
            }
        }
        let left = if rtl { count as f32 * 48.0 } else { nav_width };
        let right = if rtl { nav_width } else { count as f32 * 48.0 };
        let centered = variant == TopAppBarVariant::CenterAligned;
        let title_left = if centered { left.max(right) } else { left };
        let title_right = if centered { left.max(right) } else { right };
        let small_title = Rect {
            x: bar.x + title_left + 16.0,
            y: bar.y + 8.0,
            width: (bar.width - title_left - title_right - 32.0).max(0.0),
            height: 48.0,
        };
        let is_large = matches!(variant, TopAppBarVariant::Medium | TopAppBarVariant::Large);
        let small_alpha = if is_large { fraction } else { 1.0 };
        if small_alpha > 0.001 {
            self.styled_text(
                small_title,
                title,
                t.text.opacity(small_alpha),
                TextStyle {
                    size: 22.0,
                    align: if variant == TopAppBarVariant::CenterAligned {
                        1
                    } else if rtl {
                        2
                    } else {
                        0
                    },
                    ..TextStyle::default()
                },
            );
        }
        if is_large && fraction < 0.999 {
            let title_bounds = Rect {
                x: bar.x + 16.0,
                y: bar.y + height - 52.0,
                width: (bar.width - 32.0).max(0.0),
                height: 40.0,
            };
            self.styled_text(
                title_bounds,
                title,
                t.text.opacity(1.0 - fraction),
                TextStyle {
                    size: if variant == TopAppBarVariant::Large {
                        28.0
                    } else {
                        24.0
                    },
                    align: if rtl { 2 } else { 0 },
                    ..TextStyle::default()
                },
            );
        }
        selected
    }

    /// Route scrolling for a bottom bar and settle after a touch gesture or
    /// its content fling completes. Keyboard and wheel deltas remain direct.
    #[allow(clippy::too_many_arguments)]
    pub fn bottom_app_bar_scroll_input(
        &mut self,
        target_id: u64,
        bounds: Rect,
        bar: &mut BottomAppBarState,
        behavior: BottomAppBarScrollBehavior,
        scroll: &mut TopAppBarState,
        content_offset: &mut f32,
        content_max: f32,
        bar_height: f32,
    ) -> f32 {
        use crate::timing::{Easing, Tween, TweenSpec};
        let height = if bar_height.is_finite() {
            bar_height.max(0.0)
        } else {
            0.0
        };
        if behavior == BottomAppBarScrollBehavior::Pinned || bar.settle_height != height {
            bar.settle_target = None;
        }
        if let Some(target) = bar.settle_target {
            let spec = TweenSpec::new(0.25, Easing::Standard);
            bar.hidden = bar
                .settle
                .sample(self.ui.time, target, spec, spec, self.reduced_motion())
                .clamp(0.0, height);
            if bar.settle.moving() {
                self.ui.continuous = true;
            } else {
                bar.settle_target = None;
            }
        }
        let was_dragging = scroll.dragging;
        let was_flinging = scroll.fling_velocity != 0.0;
        let key_scroll = self.ui.focused == Some(target_id)
            && self.input.key.is_some_and(|k| {
                k.pressed
                    && matches!(
                        k.key,
                        Key::Up | Key::Down | Key::Home | Key::End | Key::PageUp | Key::PageDown
                    )
            });
        let direct_input = self.input.pressed
            || self.input.cancelled
            || self.input.blurred
            || key_scroll
            || self.scroll_y(bounds) != 0.0
            || !self.enabled;
        let mut consumed = self.top_app_bar_scroll_input(
            bounds,
            scroll,
            TopAppBarVariant::Small,
            TopAppBarScrollBehavior::Pinned,
            content_offset,
            content_max,
        );
        consumed += self.top_app_bar_scroll_keyboard(
            target_id,
            bounds,
            scroll,
            TopAppBarVariant::Small,
            TopAppBarScrollBehavior::Pinned,
            content_offset,
            content_max,
        );
        if direct_input {
            bar.settle_target = None;
        }
        bar.apply_scroll(behavior, consumed, height);
        let released =
            was_dragging && self.input.released && !self.input.cancelled && !self.input.blurred;
        let ended_fling = was_flinging && scroll.fling_velocity == 0.0 && !direct_input;
        if behavior == BottomAppBarScrollBehavior::ExitAlways
            && height > 0.0
            && (released || ended_fling)
            && scroll.fling_velocity == 0.0
            && bar.hidden > 0.001
            && bar.hidden < height - 0.001
        {
            let target = if bar.hidden / height < 0.5 {
                0.0
            } else {
                height
            };
            bar.settle = Tween::at(bar.hidden);
            bar.settle_target = Some(target);
            bar.settle_height = height;
            let spec = TweenSpec::new(0.25, Easing::Standard);
            bar.hidden = bar
                .settle
                .sample(self.ui.time, target, spec, spec, self.reduced_motion())
                .clamp(0.0, height);
            if bar.settle.moving() {
                self.ui.continuous = true;
            } else {
                bar.settle_target = None;
            }
        }
        consumed
    }

    /// Draw an anchored bar within a clipped viewport using the current
    /// scroll offset. Fully hidden actions are removed from the focus order.
    #[allow(clippy::too_many_arguments)]
    pub fn scrolling_bottom_app_bar(
        &mut self,
        id: u64,
        viewport: Rect,
        state: &BottomAppBarState,
        actions: &[AppBarAction<'_>],
        icons: &[Option<AppBarIcon>],
        fab: Option<(AppBarAction<'_>, Option<AppBarIcon>)>,
        rtl: bool,
    ) -> Option<usize> {
        let height = viewport.height.clamp(0.0, 80.0);
        if height < 56.0 || state.hidden >= height {
            return None;
        }
        let bar = Rect {
            y: viewport.y + viewport.height - height + state.hidden.clamp(0.0, height),
            height,
            ..viewport
        };
        let mut chosen = None;
        self.within(viewport, |f| {
            chosen = f.bottom_app_bar_at(id, bar, actions, icons, fab, rtl);
        });
        chosen
    }
    /// Bottom bar with a discoverable overflow menu. The returned index uses
    /// the original action order; the FAB, including when overflowed, is
    /// `actions.len()`. Supports up to 64 actions and a viewport at least
    /// 56 dp wide; the menu is independently scrollable when necessary.
    #[allow(clippy::too_many_arguments)]
    pub fn scrolling_bottom_app_bar_with_overflow(
        &mut self,
        id: u64,
        viewport: Rect,
        bar: &BottomAppBarState,
        menu: &mut BottomAppBarOverflowState,
        actions: &[AppBarAction<'_>],
        icons: &[Option<AppBarIcon>],
        fab: Option<(AppBarAction<'_>, Option<AppBarIcon>)>,
        rtl: bool,
    ) -> Option<usize> {
        if actions.len() > 64 || id.checked_add(actions.len() as u64 * 2 + 5).is_none() {
            return None;
        }
        let height = viewport.height.clamp(0.0, 80.0);
        let bar_bounds = Rect {
            y: viewport.y + viewport.height - height + bar.hidden.clamp(0.0, height),
            height,
            ..viewport
        };
        let mut layout = (None, 0, false, None);
        if height >= 56.0 && bar.hidden < height {
            self.without_input(menu.overlay.open, |f| {
                f.within(viewport, |f| {
                    layout = f.bottom_app_bar_inner(id, bar_bounds, actions, icons, fab, rtl, true);
                })
            });
        }
        let (mut chosen, visible, fab_visible, anchor) = layout;
        let total =
            actions.len().saturating_sub(visible) + usize::from(fab.is_some() && !fab_visible);
        if anchor.is_none() || total == 0 {
            menu.overlay.open = false;
        }
        if chosen == Some(actions.len() + 1) {
            menu.overlay.open = true;
            menu.offset = 0.0;
            chosen = None;
        }
        let anchor = anchor.unwrap_or(bar_bounds);
        let item_id = id + actions.len() as u64 + 2;
        let overlay_id = item_id + total as u64 + 1;
        let labels = |i: usize| -> AppBarAction<'_> {
            let index = visible + i;
            if index < actions.len() {
                actions[index]
            } else {
                fab.expect("overflowed FAB").0
            }
        };
        let width = (0..total)
            .map(|i| self.text_width(labels(i).label, TextStyle::LABEL) + 48.0)
            .fold(112.0_f32, f32::max)
            .min(280.0)
            .min(viewport.width);
        let height = (total as f32 * 48.0 + 16.0).min((anchor.y - viewport.y - 4.0).max(0.0));
        let bounds = Rect {
            x: if rtl {
                (anchor.x + anchor.width - width).max(viewport.x)
            } else {
                anchor.x.min(viewport.x + viewport.width - width)
            },
            y: anchor.y - height,
            width,
            height,
        };
        let body_height = (height - 16.0).max(0.0);
        let max_offset = (total as f32 * 48.0 - body_height).max(0.0);
        menu.offset = menu.offset.clamp(0.0, max_offset);
        if !menu.overlay.open
            || self.input.pressed
            || self.input.cancelled
            || self.input.blurred
            || self.reduced_motion()
            || self.input.scroll[1] != 0.0
            || self.input.key.is_some()
        {
            menu.fling_velocity = 0.0;
        }
        if menu.fling_velocity != 0.0 {
            let (movement, moving) = menu.fling_curve.step(self.ui.time);
            let old = menu.offset;
            menu.offset = (old + movement).clamp(0.0, max_offset);
            if !moving { menu.fling_velocity = 0.; }
            if menu.fling_velocity.abs() < 20.0 || (menu.offset - old - movement).abs() > 0.5 {
                menu.fling_velocity = 0.0;
            } else {
                self.ui.continuous = true;
            }
        }
        if menu.overlay.open {
            let scroll = self.scroll_y(bounds);
            if scroll.is_finite() {
                menu.offset = (menu.offset + scroll).clamp(0.0, max_offset);
            }
            if max_offset > 0.0 && self.input.pointer_kind == PointerKind::Touch {
                if self.input.pressed && self.pointer_in(bounds) {
                    menu.drag_origin = self.input.pointer.map(|p| p[1]);
                    menu.drag_last = menu.drag_origin;
                    menu.dragging = false;
                    menu.drag_velocity = 0.0;
                    menu.drag_sample_time = self.ui.time;
                    menu.drag_motion_time = self.ui.time;
                } else if let (Some(origin), Some(last), Some(pointer)) =
                    (menu.drag_origin, menu.drag_last, self.input.pointer)
                {
                    if (pointer[1] - origin).abs() > 4.0 {
                        menu.dragging = true;
                    }
                    if menu.dragging {
                        let movement = last - pointer[1];
                        menu.offset = (menu.offset + movement).clamp(0.0, max_offset);
                        if movement != 0.0 {
                            let elapsed = self.ui.time - menu.drag_sample_time;
                            menu.drag_velocity = if (0.005..=0.15).contains(&elapsed) {
                                (movement / elapsed as f32).clamp(-4000.0, 4000.0)
                            } else {
                                0.0
                            };
                            menu.drag_sample_time = self.ui.time;
                            menu.drag_motion_time = self.ui.time;
                        }
                        self.ui.active = None;
                        menu.drag_last = Some(pointer[1]);
                    }
                }
            }
            if self.input.key.is_some_and(|key| key.pressed)
                || self.input.blurred
                || self.input.cancelled
            {
                let typeahead_items: Vec<_> = (0..total)
                    .map(|i| {
                        let action = labels(i);
                        (action.label, action.enabled)
                    })
                    .collect();
                if let Some(next) =
                    self.menu_typeahead(&mut menu.typeahead, item_id, &typeahead_items)
                {
                    let top = next as f32 * 48.0;
                    if top < menu.offset {
                        menu.offset = top;
                    } else if top + 48.0 > menu.offset + body_height {
                        menu.offset = (top + 48.0 - body_height).min(max_offset);
                    }
                }
            }
            if let Some(current) = self
                .focused_id()
                .and_then(|v| v.checked_sub(item_id))
                .map(|v| v as usize)
                .filter(|i| *i < total)
            {
                let forward = self.pressed_key(Key::Down);
                let backward = self.pressed_key(Key::Up);
                let home = self.pressed_key(Key::Home);
                let end = self.pressed_key(Key::End);
                if forward || backward || home || end {
                    let candidate = (0..total)
                        .map(|step| {
                            if home {
                                step
                            } else if end {
                                total - 1 - step
                            } else if forward {
                                (current + step + 1) % total
                            } else {
                                (current + total - 1 - step) % total
                            }
                        })
                        .find(|i| labels(*i).enabled);
                    if let Some(next) = candidate {
                        let top = next as f32 * 48.0;
                        if top < menu.offset {
                            menu.offset = top;
                        } else if top + 48.0 > menu.offset + body_height {
                            menu.offset = (top + 48.0 - body_height).min(max_offset);
                        }
                        self.request_focus(item_id + next as u64);
                    }
                }
            }
        }
        if !menu.overlay.open {
            menu.typeahead.reset();
        }
        if self.input.released || self.input.cancelled || self.input.blurred || !menu.overlay.open {
            if menu.dragging {
                self.ui.active = None;
                if self.input.released
                    && !self.input.cancelled
                    && !self.input.blurred
                    && menu.overlay.open
                    && !self.reduced_motion()
                    && self.ui.time - menu.drag_motion_time <= 0.1
                    && menu.drag_velocity.abs() >= 350.0
                {
                    menu.fling_velocity = menu.drag_velocity;
                    menu.fling_curve.start(menu.fling_velocity,self.ui.time);
                            self.ui.continuous = true;
                }
            }
            menu.drag_origin = None;
            menu.drag_last = None;
            menu.dragging = false;
        }
        menu.overlay.kind = OverlayKind::Menu;
        menu.overlay.style = OverlayStyle::AngularMaterial;
        self.overlay(overlay_id, &mut menu.overlay, bounds, false, |f, open| {
            let content = f.available();
            f.within(content, |f| {
                let rows = Rect {
                    y: content.y - menu.offset,
                    height: total as f32 * 48.0,
                    ..content
                };
                f.within(rows, |f| {
                    f.with_gap(0.0, |f| {
                        for i in 0..total {
                            let action = labels(i);
                            if f.menu_item(item_id + i as u64, action.label, action.enabled)
                                .clicked
                            {
                                chosen = Some(visible + i);
                                *open = false;
                            }
                        }
                    })
                });
            });
        });
        chosen
    }

    /// Draw an 80 dp bottom app bar in caller-supplied bounds. The optional FAB
    /// sits at the logical end; the return index is `actions.len()` for it.
    /// Actions that cannot fit are omitted rather than overlapping the FAB.
    #[allow(clippy::too_many_arguments)]
    pub fn bottom_app_bar_at(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[AppBarAction<'_>],
        icons: &[Option<AppBarIcon>],
        fab: Option<(AppBarAction<'_>, Option<AppBarIcon>)>,
        rtl: bool,
    ) -> Option<usize> {
        self.bottom_app_bar_inner(id, bounds, actions, icons, fab, rtl, false)
            .0
    }

    #[allow(clippy::too_many_arguments)]
    fn bottom_app_bar_inner(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[AppBarAction<'_>],
        icons: &[Option<AppBarIcon>],
        fab: Option<(AppBarAction<'_>, Option<AppBarIcon>)>,
        rtl: bool,
        use_overflow: bool,
    ) -> (Option<usize>, usize, bool, Option<Rect>) {
        if !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width <= 0.0
            || bounds.height < 56.0
        {
            return (None, 0, false, None);
        }
        let t = self.ui.theme;
        self.paint_rect(bounds, 0.0, t.surface_low);
        let mut fab_visible = fab.is_some() && bounds.width >= 88.0;
        let mut slots =
            ((bounds.width - if fab_visible { 80.0 } else { 8.0 }).max(0.0) / 48.0) as usize;
        let mut needs_overflow =
            use_overflow && (actions.len() > slots || fab.is_some() && !fab_visible);
        if needs_overflow && slots == 0 && fab_visible {
            fab_visible = false;
            slots = ((bounds.width - 8.0).max(0.0) / 48.0) as usize;
        }
        needs_overflow &= slots > 0;
        let count = actions
            .len()
            .min(slots.saturating_sub(usize::from(needs_overflow)));
        let mut chosen = None;
        for (i, action) in actions.iter().take(count).enumerate() {
            let b = Rect {
                x: if rtl {
                    bounds.x + bounds.width - 4.0 - (i + 1) as f32 * 48.0
                } else {
                    bounds.x + 4.0 + i as f32 * 48.0
                },
                y: bounds.y + (bounds.height - 48.0) / 2.0,
                width: 48.0,
                height: 48.0,
            };
            let r = self.interact(id.wrapping_add(i as u64), b, action.enabled);
            let color = t
                .on_surface_variant
                .opacity(if r.enabled { 1.0 } else { 0.38 });
            self.state_layer(r, 24.0, color);
            if let Some(Some(icon)) = icons.get(i) {
                self.commands.push(DrawCommand::Shape {
                    bounds: b.inset(12.0),
                    index: icon.shape_index(rtl),
                    color,
                    rotation: 0.0,
                });
            } else {
                self.styled_text(
                    b,
                    action.glyph,
                    color,
                    TextStyle {
                        size: 22.0,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
            }
            self.semantic(r, Role::Button, action.label, false, None, None, None);
            self.focus_ring(r);
            if r.clicked {
                chosen = Some(i);
            }
        }
        let overflow_bounds = if needs_overflow {
            let b = Rect {
                x: if rtl {
                    bounds.x + bounds.width - 4.0 - (count + 1) as f32 * 48.0
                } else {
                    bounds.x + 4.0 + count as f32 * 48.0
                },
                y: bounds.y + (bounds.height - 48.0) / 2.0,
                width: 48.0,
                height: 48.0,
            };
            let r = self.interact(id.wrapping_add(actions.len() as u64 + 1), b, true);
            self.state_layer(r, 24.0, t.on_surface_variant);
            self.commands.push(DrawCommand::Shape {
                bounds: b.inset(12.0),
                index: AppBarIcon::More.shape_index(rtl),
                color: t
                    .on_surface_variant
                    .opacity(if r.enabled { 1.0 } else { 0.38 }),
                rotation: 0.0,
            });
            self.semantic(r, Role::Button, "More actions", false, None, None, None);
            self.focus_ring(r);
            if r.clicked {
                chosen = Some(actions.len() + 1);
            }
            Some(b)
        } else {
            None
        };
        if let Some((action, icon)) = fab.filter(|_| fab_visible) {
            let b = Rect {
                x: if rtl {
                    bounds.x + 16.0
                } else {
                    bounds.x + bounds.width - 72.0
                },
                y: bounds.y + (bounds.height - 56.0) / 2.0,
                width: 56.0,
                height: 56.0,
            };
            let r = self.interact(id.wrapping_add(actions.len() as u64), b, action.enabled);
            let fg = if r.enabled {
                t.on_secondary
            } else {
                t.text.opacity(0.38)
            };
            self.paint_rect(
                b,
                16.0,
                if r.enabled {
                    t.secondary
                } else {
                    t.text.opacity(0.12)
                },
            );
            self.state_layer(r, 16.0, fg);
            if let Some(icon) = icon {
                self.commands.push(DrawCommand::Shape {
                    bounds: b.inset(16.0),
                    index: icon.shape_index(rtl),
                    color: fg,
                    rotation: 0.0,
                });
            } else {
                self.styled_text(
                    b,
                    action.glyph,
                    fg,
                    TextStyle {
                        size: 24.0,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
            }
            self.semantic(r, Role::Button, action.label, false, None, None, None);
            self.focus_ring(r);
            if r.clicked {
                chosen = Some(actions.len());
            }
        }
        (chosen, count, fab_visible, overflow_bounds)
    }

    /// Positioned Top App Bar using official `MaterialIcon` buttons.
    /// Returns `Some(0)` if the navigation icon was clicked, or `Some(1 + i)` for trailing action `i`.
    #[allow(clippy::too_many_arguments)]
    pub fn top_app_bar_at(
        &mut self,
        id: u64,
        bounds: Rect,
        title: &str,
        centered: bool,
        navigation: Option<TopAppBarMaterialAction<'_>>,
        actions: &[TopAppBarMaterialAction<'_>],
    ) -> Option<usize> {
        let t = self.theme();
        self.paint_rect(bounds, 0.0, t.surface_low);
        let mut clicked = None;
        let btn_y = bounds.y + (bounds.height - 48.0) * 0.5;
        let nav_w: f32 = if let Some(nav) = navigation {
            let hit = Rect { x: bounds.x + 4.0, y: btn_y, width: 48.0, height: 48.0 };
            let r = self.material_icon_button_at(
                id,
                hit,
                nav.icon,
                nav.label,
                Options { enabled: nav.enabled, variant: ButtonVariant::Text, ..Options::default() },
            );
            if r.clicked {
                clicked = Some(0);
            }
            56.0
        } else {
            16.0
        };
        let count = actions.len();
        for (i, act) in actions.iter().enumerate() {
            let from_right = (count - i) as f32;
            let hit = Rect {
                x: bounds.x + bounds.width - 4.0 - from_right * 48.0,
                y: btn_y,
                width: 48.0,
                height: 48.0,
            };
            let r = self.material_icon_button_at(
                id.wrapping_add(1 + i as u64),
                hit,
                act.icon,
                act.label,
                Options { enabled: act.enabled, variant: ButtonVariant::Text, ..Options::default() },
            );
            if r.clicked {
                clicked = Some(1 + i);
            }
        }
        let trailing_w = if count > 0 { 8.0 + count as f32 * 48.0 } else { 16.0 };
        let (title_x, title_w, align) = if centered {
            let inset = nav_w.max(trailing_w);
            (bounds.x + inset, (bounds.width - inset * 2.0).max(0.0), 1)
        } else {
            (bounds.x + nav_w + 4.0, (bounds.width - nav_w - trailing_w - 8.0).max(0.0), 0)
        };
        self.styled_text(
            Rect { x: title_x, y: bounds.y, width: title_w, height: bounds.height },
            title,
            t.text,
            TextStyle { size: 20.0, weight: if centered { 600 } else { 500 }, align, ..TextStyle::default() },
        );
        clicked
    }

    /// Continuous scale-glide Collapsing Top App Bar (`CollapsingToolbarLayout` pattern):
    /// smoothly collapses from `expanded_h` to `collapsed_h` as `scroll_y` increases while a
    /// single title scales continuously (`32sp -> 20sp`) and glides diagonally beside the back button.
    /// Returns `(current_bar_height, back_clicked)`.
    #[allow(clippy::too_many_arguments)]
    pub fn top_app_bar_collapsing(
        &mut self,
        id: u64,
        bounds: Rect,
        title: &str,
        expanded_h: f32,
        collapsed_h: f32,
        scroll_y: f32,
        back_icon: Option<MaterialIcon>,
    ) -> (f32, bool) {
        let collapsed_h = collapsed_h.max(48.0);
        let expanded_h = expanded_h.max(collapsed_h);
        let range = (expanded_h - collapsed_h).max(1.0);
        let collapse = (scroll_y / range).clamp(0.0, 1.0);
        let bar_h = expanded_h + (collapsed_h - expanded_h) * collapse;
        let t = self.theme();
        let bg = t.background.mix(t.surface_high, collapse);
        let bar = Rect { height: bar_h, ..bounds };
        self.paint_rect(bar, 0.0, bg);
        let mut back_clicked = false;
        if let Some(icon) = back_icon {
            let hit = Rect { x: bar.x + 4.0, y: bar.y + (collapsed_h - 48.0) * 0.5, width: 48.0, height: 48.0 };
            let r = self.material_icon_button_at(
                id,
                hit,
                icon,
                "Navigate up",
                Options { variant: ButtonVariant::Text, ..Options::default() },
            );
            back_clicked = r.clicked;
        }
        let start_x = bar.x + 20.0;
        let end_x = bar.x + if back_icon.is_some() { 60.0 } else { 20.0 };
        let title_x = start_x + (end_x - start_x) * collapse;
        let start_y = bar.y + expanded_h - 48.0;
        let end_y = bar.y + (collapsed_h - 32.0) * 0.5;
        let title_y = start_y + (end_y - start_y) * collapse;
        let title_size = 32.0 + (20.0 - 32.0) * collapse;
        self.styled_text(
            Rect {
                x: title_x,
                y: title_y,
                width: (bar.x + bar.width - title_x - 16.0).max(0.0),
                height: 36.0,
            },
            title,
            t.text,
            TextStyle { size: title_size, weight: 500, ..TextStyle::default() },
        );
        (bar_h, back_clicked)
    }

    /// Floating primary-filled pill app bar with elevation shadow, icon buttons, and optional
    /// custom brand slot (invoked whenever `slot.icon` is `None`).
    /// Returns the clicked slot index, if any.
    pub fn floating_pill_app_bar(
        &mut self,
        id: u64,
        bounds: Rect,
        slots: &[FloatingPillSlot<'_>],
        mut custom_slot: impl FnMut(&mut Self, usize, Rect),
    ) -> Option<usize> {
        let t = self.theme();
        let radius = bounds.height * 0.5;
        self.shadow(bounds, radius, [0.0, 2.0], 5.0, 0.0, Color(0, 0, 0, 35));
        self.paint_rect(bounds, radius, t.primary);
        let mut clicked = None;
        let btn_sz = 48.0_f32.min(bounds.height);
        let pad_x = 4.0;
        let pad_y = (bounds.height - btn_sz) * 0.5;
        for (i, slot) in slots.iter().enumerate() {
            let hit = Rect {
                x: bounds.x + pad_x + i as f32 * btn_sz,
                y: bounds.y + pad_y,
                width: btn_sz,
                height: btn_sz,
            };
            let r = self.target(id.wrapping_add(i as u64), hit, slot.label, true, Role::Button);
            if r.hovered || (r.focused && self.ui.keyboard_focus) {
                self.paint_rect(hit, btn_sz * 0.5, t.on_primary.opacity(0.12));
            }
            self.ink(r, btn_sz * 0.5, t.on_primary);
            self.focus_ring(r);
            if let Some(icon) = slot.icon {
                self.paint_material_icon(hit.inset(12.0), icon, t.on_primary);
            } else {
                custom_slot(self, i, hit.inset(8.0));
            }
            if r.clicked {
                clicked = Some(i);
            }
        }
        clicked
    }
}

/// Action descriptor using official `MaterialIcon` for `Frame::top_app_bar_at`.
#[derive(Clone, Copy, Debug)]
pub struct TopAppBarMaterialAction<'a> {
    pub icon: MaterialIcon,
    pub label: &'a str,
    pub enabled: bool,
}

/// Slot descriptor for `Frame::floating_pill_app_bar` (`icon: None` delegates to `custom_slot`).
#[derive(Clone, Copy, Debug)]
pub struct FloatingPillSlot<'a> {
    pub label: &'a str,
    pub icon: Option<MaterialIcon>,
}

#[cfg(test)]
#[path = "app_bar.spec.rs"]
mod tests;
