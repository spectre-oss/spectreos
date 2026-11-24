use crate::*;

/// Placement and surface style for a Material-inspired toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarKind {
    Docked,
    FloatingHorizontal,
    FloatingVertical,
}

#[derive(Clone, Copy, Debug)]
pub struct ToolbarAction<'a> {
    pub label: &'a str,
    pub enabled: bool,
}
/// Logical FAB position beside a floating toolbar (start/top or end/bottom).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarFabPosition { Start, End }

/// Action returned by a floating toolbar paired with a FAB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarFabSelection { Action(usize), Fab }

/// Persistent tab stop for an arrow-navigable toolbar.
#[derive(Default)]
pub struct ToolbarState {
    index: usize,
    overlay: OverlayState,
    offset: f32,
    drag_origin: Option<f32>,
    drag_last: Option<f32>,
    dragging: bool,
    drag_velocity: f32,
    drag_sample_time: f64,
    drag_motion_time: f64,
    fling_velocity: f32,
    fling_curve: crate::scroll::SplineFling,
    docked_icons: Vec<AppBarIcon>,
    primary: Option<usize>,
    overflow_trigger: Option<usize>,
    typeahead: MenuTypeaheadState,
}
impl ToolbarState {
    pub fn overflow_open(&self) -> bool {
        self.overlay.open
    }
}
/// Caller-owned Android-style hide-on-scroll transition for a bottom-docked toolbar.
#[derive(Default)]
pub struct DockedToolbarScrollState {
    hidden: bool,
    progress: crate::timing::Tween,
}

/// Hysteresis for expanding or collapsing a floating toolbar from consumed scroll.
#[derive(Default)]
pub struct FloatingToolbarExpansionScrollState {
    offset:f32,
    threshold:f32,
    previous:Option<bool>,
    thresholds:Option<(f32,f32)>,
}
impl FloatingToolbarExpansionScrollState {
    /// Positive consumed scroll means content moved down; reverse changes that policy.
    /// Thresholds are logical pixels; invalid values use the Material 40 dp default.
    pub fn update(&mut self, expanded:&mut bool, consumed:f32, reverse:bool, expand_threshold:f32, collapse_threshold:f32)->bool {
        let valid=|v:f32|if v.is_finite() && v>=0.0 {v} else {40.0};
        let expand_threshold=valid(expand_threshold);let collapse_threshold=valid(collapse_threshold);
        if self.previous!=Some(*expanded) || self.thresholds!=Some((expand_threshold,collapse_threshold)) {
            self.threshold=if *expanded {self.offset-collapse_threshold}else{self.offset+expand_threshold};
            self.previous=Some(*expanded);self.thresholds=Some((expand_threshold,collapse_threshold));
        }
        if !consumed.is_finite() || consumed==0.0 {return false;}
        let delta=if reverse {consumed}else{-consumed};
        self.offset=(self.offset+delta).clamp(-1_000_000.0,1_000_000.0);
        let before=*expanded;
        if delta<0.0 && self.offset<=self.threshold {*expanded=false;self.threshold=self.offset+expand_threshold;}
        else if delta>0.0 && self.offset>=self.threshold {*expanded=true;self.threshold=self.offset-collapse_threshold;}
        self.previous=Some(*expanded);
        if self.offset.abs()>500_000.0 {self.threshold-=self.offset;self.offset=0.0;}
        before!=*expanded
    }
}

/// Dock edge used to resolve automatic system-inset padding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DockedToolbarEdge {
    Top,
    #[default]
    Bottom,
}

/// Insets supplied by the host after combining system bars, cutouts, and IME.
#[derive(Clone, Copy, Debug)]
pub struct DockedToolbarInsets {
    pub top: f32,
    pub bottom: f32,
    pub apply_top: Option<bool>,
    pub apply_bottom: Option<bool>,
    pub edge: DockedToolbarEdge,
    pub fits_system_windows: bool,
}
impl Default for DockedToolbarInsets {
    fn default() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            apply_top: None,
            apply_bottom: None,
            edge: DockedToolbarEdge::Bottom,
            fits_system_windows: true,
        }
    }
}
impl DockedToolbarInsets {
    /// Split a caller-supplied total surface into background and 64 dp content.
    /// The bounds must include enough space for the requested insets.
    pub fn content_bounds(self, bounds: Rect) -> Option<Rect> {
        if !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width < 0.0
            || bounds.height < 0.0
        {
            return None;
        }
        let top = if self
            .apply_top
            .unwrap_or(self.fits_system_windows && self.edge == DockedToolbarEdge::Top)
            && self.top.is_finite()
        {
            self.top.max(0.0)
        } else {
            0.0
        };
        let bottom = if self
            .apply_bottom
            .unwrap_or(self.fits_system_windows && self.edge == DockedToolbarEdge::Bottom)
            && self.bottom.is_finite()
        {
            self.bottom.max(0.0)
        } else {
            0.0
        };
        let height = bounds.height - top - bottom;
        if height < 64.0 {
            return None;
        }
        Some(Rect {
            y: bounds.y + top,
            height,
            ..bounds
        })
    }
}

impl Frame<'_> {
    /// Draw a docked or floating action toolbar. The caller owns placement,
    /// actions, and any overflow policy; returns the activated action index.
    pub fn toolbar(
        &mut self,
        id: u64,
        bounds: Rect,
        kind: ToolbarKind,
        vibrant: bool,
        actions: &[ToolbarAction<'_>],
    ) -> Option<usize> {
        self.toolbar_inner(id, bounds, kind, vibrant, actions, None, false)
    }
    /// Toolbar with one tab stop and arrow-key navigation.
    #[allow(clippy::too_many_arguments)]
    pub fn toolbar_roving(
        &mut self,
        id: u64,
        bounds: Rect,
        kind: ToolbarKind,
        vibrant: bool,
        actions: &[ToolbarAction<'_>],
        state: &mut ToolbarState,
        rtl: bool,
    ) -> Option<usize> {
        if state.overlay.open {
            state.overlay = OverlayState::default();
            if let Some(index) = actions.iter().position(|action| action.enabled)
                && let Some(focus_id) = id.checked_add(index as u64)
            {
                state.index = index;
                self.request_focus(focus_id);
            }
        }
        state.typeahead.reset();
        state.fling_velocity = 0.0;
        state.docked_icons.clear();
        state.primary = None;
        state.overflow_trigger = None;
        self.toolbar_inner(id, bounds, kind, vibrant, actions, Some(state), rtl)
    }
    /// Sample bottom-edge hide-on-scroll. Positive consumed deltas hide; negative show.
    /// Touch exploration forces visibility; the caller must reserve content padding.
    pub fn docked_toolbar_scroll_offset(
        &mut self,
        state: &mut DockedToolbarScrollState,
        consumed: f32,
        height: f32,
        touch_exploration: bool,
    ) -> f32 {
        use crate::timing::{Easing, TweenSpec};
        if touch_exploration {
            state.hidden = false;
        } else if consumed.is_finite() && consumed != 0.0 {
            state.hidden = consumed > 0.0;
        }
        let height = if height.is_finite() {
            height.max(0.0)
        } else {
            0.0
        };
        let progress = state.progress.sample(
            self.ui.time,
            f32::from(state.hidden),
            TweenSpec::new(0.175, Easing::Emphasized),
            TweenSpec::new(0.225, Easing::Emphasized),
            self.reduced_motion() || touch_exploration,
        );
        self.ui.continuous |= state.progress.moving();
        progress.clamp(0.0, 1.0) * height
    }

    /// Bottom-anchored floating toolbar hide-on-scroll offset. `hide_distance`
    /// includes the toolbar height and its gap to the viewport's bottom edge.
    pub fn floating_toolbar_scroll_offset(
        &mut self,
        state: &mut DockedToolbarScrollState,
        consumed: f32,
        hide_distance: f32,
    ) -> f32 {
        self.docked_toolbar_scroll_offset(state, consumed, hide_distance, false)
    }

    /// Docked toolbar whose background also fills host-supplied system insets.
    #[allow(clippy::too_many_arguments)]
    pub fn docked_toolbar_with_insets(
        &mut self,
        id: u64,
        bounds: Rect,
        insets: DockedToolbarInsets,
        vibrant: bool,
        actions: &[ToolbarAction<'_>],
        icons: &[AppBarIcon],
        primary: Option<usize>,
        state: &mut ToolbarState,
        rtl: bool,
    ) -> Option<usize> {
        let content = insets.content_bounds(bounds)?;
        let t = self.ui.theme;
        self.paint_rect(
            bounds,
            0.0,
            if vibrant {
                t.primary_container
            } else {
                t.control
            },
        );
        self.docked_toolbar_with_overflow(id, content, vibrant, actions, icons, primary, state, rtl)
    }

    /// Android-style 64 dp docked toolbar with icon actions and optional primary action.
    #[allow(clippy::too_many_arguments)]
    pub fn docked_toolbar_with_overflow(
        &mut self,
        id: u64,
        bounds: Rect,
        vibrant: bool,
        actions: &[ToolbarAction<'_>],
        icons: &[AppBarIcon],
        primary: Option<usize>,
        state: &mut ToolbarState,
        rtl: bool,
    ) -> Option<usize> {
        if icons.len() != actions.len() {
            return None;
        }
        state.docked_icons.clear();
        state.docked_icons.extend_from_slice(icons);
        state.primary = primary.filter(|i| *i < actions.len());
        self.toolbar_overflow_inner(
            id,
            bounds,
            ToolbarKind::Docked,
            vibrant,
            actions,
            state,
            rtl,
        )
    }
    /// Floating toolbar and 56 dp FAB separated by the Material 8 dp gap.
    /// The FAB uses `id + 132`; toolbar and overflow reserve `id..=id + 131`.
    /// Bounds too small for a FAB, gap and one toolbar slot return no action.
    #[allow(clippy::too_many_arguments)]
    pub fn floating_toolbar_with_fab(
        &mut self, id:u64, bounds:Rect, kind:ToolbarKind, vibrant:bool,
        actions:&[ToolbarAction<'_>], state:&mut ToolbarState, rtl:bool,
        fab:ToolbarAction<'_>, icon:AppBarIcon, position:ToolbarFabPosition,
    )->Option<ToolbarFabSelection> {
        let fab_id=id.checked_add(132)?;
        if kind==ToolbarKind::Docked || ![bounds.x,bounds.y,bounds.width,bounds.height].iter().all(|v|v.is_finite()) {return None;}
        let vertical=kind==ToolbarKind::FloatingVertical;
        let (length,cross)=if vertical {(bounds.height,bounds.width)}else{(bounds.width,bounds.height)};
        if length<128.0 || cross<56.0 {return None;}
        let leading=if vertical {position==ToolbarFabPosition::Start}else{(position==ToolbarFabPosition::Start)^rtl};
        let fab_bounds=if vertical {Rect{x:bounds.x+(bounds.width-56.0)*0.5,y:if leading{bounds.y}else{bounds.y+bounds.height-56.0},width:56.0,height:56.0}}
            else {Rect{x:if leading{bounds.x}else{bounds.x+bounds.width-56.0},y:bounds.y+(bounds.height-56.0)*0.5,width:56.0,height:56.0}};
        let bar=if vertical {Rect{x:bounds.x,y:bounds.y+if leading{64.0}else{0.0},width:bounds.width,height:bounds.height-64.0}}
            else {Rect{x:bounds.x+if leading{64.0}else{0.0},y:bounds.y,width:bounds.width-64.0,height:bounds.height}};
        self.elevation_dp(bar,bar.width.min(bar.height)*0.5,1.0);
        let action=self.toolbar_with_overflow(id,bar,kind,vibrant,actions,state,rtl);
        let mut clicked=false;
        self.without_input(state.overflow_open(),|f|f.within_unclipped(fab_bounds,|f|{
            clicked=f.fab_with_options(fab_id,fab.label,icon,crate::FabOptions{size:56.0,enabled:fab.enabled,elevation:crate::FabElevation::FloatingToolbar,color:if vibrant{crate::FabColor::Tertiary}else{crate::FabColor::Primary},..crate::FabOptions::default()}).clicked;
        }));
        if clicked {Some(ToolbarFabSelection::Fab)} else {action.map(ToolbarFabSelection::Action)}
    }

    /// Expandable floating toolbar paired with a 56–80 dp FAB. The caller owns
    /// the expanded state and supplies stable bounds with at least 80 dp cross size.
    /// Toolbar actions stop accepting input immediately when collapse is requested.
    #[allow(clippy::too_many_arguments)]
    pub fn floating_toolbar_with_fab_expanded(
        &mut self, id:u64, bounds:Rect, kind:ToolbarKind, vibrant:bool,
        actions:&[ToolbarAction<'_>], state:&mut ToolbarState, rtl:bool,
        fab:ToolbarAction<'_>, icon:AppBarIcon, position:ToolbarFabPosition, expanded:bool,
    )->Option<ToolbarFabSelection> {
        let fab_id=id.checked_add(132)?;
        if kind==ToolbarKind::Docked || ![bounds.x,bounds.y,bounds.width,bounds.height].iter().all(|v|v.is_finite()) {return None;}
        let vertical=kind==ToolbarKind::FloatingVertical;
        let (length,cross)=if vertical {(bounds.height,bounds.width)}else{(bounds.width,bounds.height)};
        if length<144.0 || cross<80.0 {return None;}
        let target=f32::from(expanded);
        let progress=self.animate_geometry(id,239,target,target).clamp(0.0,1.0);
        let fab_size=80.0-24.0*progress;
        let bar_length=(length-64.0)*progress;
        let leading=if vertical {position==ToolbarFabPosition::Start}else{(position==ToolbarFabPosition::Start)^rtl};
        let fab_bounds=if vertical {Rect{x:bounds.x+(cross-fab_size)*0.5,y:if leading{bounds.y}else{bounds.y+length-fab_size},width:fab_size,height:fab_size}}
            else {Rect{x:if leading{bounds.x}else{bounds.x+length-fab_size},y:bounds.y+(cross-fab_size)*0.5,width:fab_size,height:fab_size}};
        let bar=if vertical {Rect{x:bounds.x+(cross-64.0)*0.5,y:if leading{bounds.y+64.0}else{bounds.y+length-64.0-bar_length},width:64.0,height:bar_length}}
            else {Rect{x:if leading{bounds.x+64.0}else{bounds.x+length-64.0-bar_length},y:bounds.y+(cross-64.0)*0.5,width:bar_length,height:64.0}};
        if !expanded {state.overlay=OverlayState::default();}
        self.elevation_dp(bar,bar.width.min(bar.height)*0.5,progress);
        let mut action=None;
        if bar_length>=64.0 {
            self.without_input(!expanded,|f|{action=f.toolbar_with_overflow(id,bar,kind,vibrant,actions,state,rtl);});
        } else if bar_length>0.0 {
            let t=self.ui.theme;self.paint_rect(bar,bar.width.min(bar.height)*0.5,if vibrant{t.primary_container}else{t.control});
        }
        let mut clicked=false;
        self.without_input(state.overflow_open(),|f|f.within_unclipped(fab_bounds,|f|{
            clicked=f.fab_with_options(fab_id,fab.label,icon,crate::FabOptions{size:fab_size,enabled:fab.enabled,elevation:crate::FabElevation::FloatingToolbar,color:if vibrant{crate::FabColor::Tertiary}else{crate::FabColor::Primary},..crate::FabOptions::default()}).clicked;
        }));
        if clicked {Some(ToolbarFabSelection::Fab)} else {action.map(ToolbarFabSelection::Action)}
    }

    /// Toolbar with a menu for actions that do not fit. Supports up to 64 actions.
    /// IDs `id..=id+131` are reserved; returns the original action index.
    #[allow(clippy::too_many_arguments)]
    pub fn toolbar_with_overflow(
        &mut self,
        id: u64,
        bounds: Rect,
        kind: ToolbarKind,
        vibrant: bool,
        actions: &[ToolbarAction<'_>],
        state: &mut ToolbarState,
        rtl: bool,
    ) -> Option<usize> {
        state.docked_icons.clear();
        state.primary = None;
        self.toolbar_overflow_inner(id, bounds, kind, vibrant, actions, state, rtl)
    }
    #[allow(clippy::too_many_arguments)]
    fn toolbar_overflow_inner(
        &mut self,
        id: u64,
        bounds: Rect,
        kind: ToolbarKind,
        vibrant: bool,
        actions: &[ToolbarAction<'_>],
        state: &mut ToolbarState,
        rtl: bool,
    ) -> Option<usize> {
        if actions.len() > 64 {
            return None;
        }
        id.checked_add(131)?;
        let vertical = kind == ToolbarKind::FloatingVertical;
        let available = if vertical {
            bounds.height
        } else {
            bounds.width
        };
        let docked_icons = kind == ToolbarKind::Docked && !state.docked_icons.is_empty();
        let padding = if docked_icons { 32.0 } else { 16.0 };
        let slots = ((available - padding).max(0.0) / 48.0) as usize;
        if slots == 0 || actions.len() <= slots {
            state.overflow_trigger = None;
            state.typeahead.reset();
            state.fling_velocity = 0.0;
            if state.overlay.open {
                state.overlay = OverlayState::default();
                if slots > 0
                    && let Some(index) = actions.iter().take(slots).position(|a| a.enabled)
                {
                    state.index = index;
                    self.request_focus(id + index as u64);
                }
            }
            return self.toolbar_inner(id, bounds, kind, vibrant, actions, Some(state), rtl);
        }
        let visible = slots - 1;
        state.overflow_trigger = Some(visible);
        let mut shown = actions[..visible].to_vec();
        shown.push(ToolbarAction {
            label: "More",
            enabled: true,
        });
        let mut selected = None;
        self.without_input(state.overlay.open, |f| {
            selected = f.toolbar_inner(id, bounds, kind, vibrant, &shown, Some(state), rtl);
        });
        if selected == Some(visible) {
            selected = None;
            state.overlay.open = true;
            state.offset = 0.0;
        }
        let viewport = self.viewport();
        let trigger = if docked_icons {
            let cell = (bounds.width - 32.0) / slots as f32;
            let slot = if rtl { slots - 1 - visible } else { visible };
            Rect {
                x: bounds.x + 16.0 + slot as f32 * cell + (cell - 48.0) / 2.0,
                y: bounds.y + (bounds.height - 48.0) / 2.0,
                width: 48.0,
                height: 48.0,
            }
        } else if vertical {
            Rect {
                x: bounds.x + (bounds.width - 48.0) / 2.0,
                y: bounds.y + 8.0 + visible as f32 * 48.0,
                width: 48.0,
                height: 48.0,
            }
        } else {
            Rect {
                x: if rtl {
                    bounds.x + bounds.width - 56.0 - visible as f32 * 48.0
                } else {
                    bounds.x + 8.0 + visible as f32 * 48.0
                },
                y: bounds.y + (bounds.height - 48.0) / 2.0,
                width: 48.0,
                height: 48.0,
            }
        };
        let remaining = &actions[visible..];
        let width = remaining
            .iter()
            .map(|a| {
                self.text_width(a.label, TextStyle::LABEL) + if docked_icons { 72.0 } else { 48.0 }
            })
            .fold(112.0_f32, f32::max)
            .min(280.0)
            .min(viewport.width);
        let below = (viewport.y + viewport.height - trigger.y - trigger.height).max(0.0);
        let above = (trigger.y - viewport.y).max(0.0);
        let down = below >= above;
        let height = (remaining.len() as f32 * 48.0 + 16.0).min(if down { below } else { above });
        let menu_bounds = Rect {
            x: if rtl {
                (trigger.x + trigger.width - width).max(viewport.x)
            } else {
                trigger.x.min(viewport.x + viewport.width - width)
            },
            y: if down {
                trigger.y + trigger.height
            } else {
                trigger.y - height
            },
            width,
            height,
        };
        let body = (height - 16.0).max(0.0);
        let max_offset = (remaining.len() as f32 * 48.0 - body).max(0.0);
        state.offset = state.offset.clamp(0.0, max_offset);
        if !state.overlay.open
            || self.input.pressed
            || self.input.cancelled
            || self.input.blurred
            || self.reduced_motion()
            || self.input.scroll[1] != 0.0
            || self.input.key.is_some()
        {
            state.fling_velocity = 0.0;
        }
        if state.fling_velocity != 0.0 {
            let (movement, moving) = state.fling_curve.step(self.ui.time);
            let old = state.offset;
            state.offset = (old + movement).clamp(0.0, max_offset);
            if !moving { state.fling_velocity = 0.; }
            if state.fling_velocity.abs() < 20.0 || (state.offset - old - movement).abs() > 0.5 {
                state.fling_velocity = 0.0;
            } else {
                self.ui.continuous = true;
            }
        }
        let first_id = id + 66;
        if state.overlay.open {
            let scroll = self.scroll_y(menu_bounds);
            if scroll.is_finite() {
                state.offset = (state.offset + scroll).clamp(0.0, max_offset);
            }
            if max_offset > 0.0 && self.input.pointer_kind == PointerKind::Touch {
                if self.input.pressed && self.pointer_in(menu_bounds) {
                    state.drag_origin = self.input.pointer.map(|p| p[1]);
                    state.drag_last = state.drag_origin;
                    state.dragging = false;
                    state.drag_velocity = 0.0;
                    state.drag_sample_time = self.ui.time;
                    state.drag_motion_time = self.ui.time;
                } else if let (Some(origin), Some(last), Some(pointer)) =
                    (state.drag_origin, state.drag_last, self.input.pointer)
                {
                    if (pointer[1] - origin).abs() > 4.0 {
                        state.dragging = true;
                    }
                    if state.dragging {
                        let movement = last - pointer[1];
                        state.offset = (state.offset + movement).clamp(0.0, max_offset);
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
                        state.drag_last = Some(pointer[1]);
                        self.ui.active = None;
                    }
                }
            }
            if self.input.key.is_some_and(|key| key.pressed)
                || self.input.blurred
                || self.input.cancelled
            {
                let items: Vec<_> = remaining
                    .iter()
                    .map(|action| (action.label, action.enabled))
                    .collect();
                if let Some(next) = self.menu_typeahead(&mut state.typeahead, first_id, &items) {
                    let top = next as f32 * 48.0;
                    if top < state.offset {
                        state.offset = top;
                    } else if top + 48.0 > state.offset + body {
                        state.offset = (top + 48.0 - body).min(max_offset);
                    }
                }
            }
            if let Some(current) = self
                .focused_id()
                .and_then(|v| v.checked_sub(first_id))
                .and_then(|v| usize::try_from(v).ok())
                .filter(|i| *i < remaining.len())
            {
                let key = self.input.key.filter(|k| k.pressed).map(|k| k.key);
                if matches!(key, Some(Key::Up | Key::Down | Key::Home | Key::End)) {
                    let next = (0..remaining.len())
                        .map(|step| match key {
                            Some(Key::Home) => step,
                            Some(Key::End) => remaining.len() - 1 - step,
                            Some(Key::Down) => (current + step + 1) % remaining.len(),
                            _ => (current + remaining.len() - 1 - step) % remaining.len(),
                        })
                        .find(|i| remaining[*i].enabled);
                    if let Some(next) = next {
                        let top = next as f32 * 48.0;
                        if top < state.offset {
                            state.offset = top;
                        } else if top + 48.0 > state.offset + body {
                            state.offset = (top + 48.0 - body).min(max_offset);
                        }
                        self.request_focus(first_id + next as u64);
                        self.key_consumed = true;
                    }
                }
            }
        }
        if !state.overlay.open {
            state.typeahead.reset();
        }
        if self.input.released || self.input.cancelled || self.input.blurred || !state.overlay.open
        {
            if state.dragging {
                self.ui.active = None;
                if self.input.released
                    && !self.input.cancelled
                    && !self.input.blurred
                    && state.overlay.open
                    && !self.reduced_motion()
                    && self.ui.time - state.drag_motion_time <= 0.1
                    && state.drag_velocity.abs() >= 350.0
                {
                    state.fling_velocity = state.drag_velocity;
                    state.fling_curve.start(state.fling_velocity,self.ui.time);
                            self.ui.continuous = true;
                }
            }
            state.drag_origin = None;
            state.drag_last = None;
            state.dragging = false;
        }
        state.overlay.kind = OverlayKind::Menu;
        state.overlay.style = OverlayStyle::AngularMaterial;
        state.overlay.initial_focus = remaining
            .iter()
            .position(|a| a.enabled)
            .map(|i| first_id + i as u64);
        let overlay_start = self.commands.len();
        self.overlay(
            id + 131,
            &mut state.overlay,
            menu_bounds,
            false,
            |f, open| {
                let content = f.available();
                f.within(content, |f| {
                    f.within(
                        Rect {
                            y: content.y - state.offset,
                            height: remaining.len() as f32 * 48.0,
                            ..content
                        },
                        |f| {
                            f.with_gap(0.0, |f| {
                                for (i, action) in remaining.iter().enumerate() {
                                    let response = if docked_icons {
                                        f.menu_item_with_icon(
                                            first_id + i as u64,
                                            action.label,
                                            action.enabled,
                                            state.docked_icons[visible + i],
                                            rtl,
                                        )
                                    } else {
                                        f.menu_item(
                                            first_id + i as u64,
                                            action.label,
                                            action.enabled,
                                        )
                                    };
                                    if response.clicked {
                                        selected = Some(visible + i);
                                        *open = false;
                                    }
                                }
                            })
                        },
                    );
                });
            },
        );
        self.ui
            .overlays
            .extend(self.commands.drain(overlay_start..));
        selected
    }
    #[allow(clippy::too_many_arguments)]
    fn toolbar_inner(
        &mut self,
        id: u64,
        bounds: Rect,
        kind: ToolbarKind,
        vibrant: bool,
        actions: &[ToolbarAction<'_>],
        mut state: Option<&mut ToolbarState>,
        rtl: bool,
    ) -> Option<usize> {
        id.checked_add(actions.len() as u64)?;
        let docked_icons = kind == ToolbarKind::Docked
            && state.as_ref().is_some_and(|s| !s.docked_icons.is_empty());
        let t = self.ui.theme;
        let background = if vibrant {
            t.primary_container
        } else if docked_icons || kind != ToolbarKind::Docked {
            t.control
        } else {
            t.surface_variant
        };
        let foreground = if vibrant {
            t.on_primary_container
        } else if docked_icons {
            t.on_surface_variant
        } else {
            t.text
        };
        let vertical = kind == ToolbarKind::FloatingVertical;
        let floating = kind != ToolbarKind::Docked;
        self.paint_rect(
            bounds,
            if floating {
                bounds.width.min(bounds.height) / 2.0
            } else {
                0.0
            },
            background,
        );
        let available = if vertical {
            bounds.height
        } else {
            bounds.width
        };
        let count = actions
            .len()
            .min(((available - if docked_icons { 32.0 } else { 16.0 }).max(0.0) / 48.0) as usize);
        let primary = state.as_ref().and_then(|s| s.primary);
        let overflow_trigger = state.as_ref().and_then(|s| s.overflow_trigger);
        let item_bounds = |i: usize| {
            if vertical {
                Rect {
                    x: bounds.x + (bounds.width - 48.0) / 2.0,
                    y: bounds.y + 8.0 + i as f32 * 48.0,
                    width: 48.0,
                    height: 48.0,
                }
            } else if docked_icons {
                let cell = (bounds.width - 32.0) / count.max(1) as f32;
                let index = if rtl { count - 1 - i } else { i };
                let size = if primary == Some(i) && overflow_trigger != Some(i) {
                    cell.min(56.0)
                } else {
                    48.0
                };
                Rect {
                    x: bounds.x + 16.0 + index as f32 * cell + (cell - size) / 2.0,
                    y: bounds.y + (bounds.height - size) / 2.0,
                    width: size,
                    height: size,
                }
            } else {
                Rect {
                    x: if rtl {
                        bounds.x + bounds.width - 56.0 - i as f32 * 48.0
                    } else {
                        bounds.x + 8.0 + i as f32 * 48.0
                    },
                    y: bounds.y + (bounds.height - 48.0) / 2.0,
                    width: 48.0,
                    height: 48.0,
                }
            }
        };
        let clip = self.clip;
        let visible = |i: usize| {
            let clipped = item_bounds(i).intersect(bounds).intersect(clip);
            actions[i].enabled && clipped.width > 0.0 && clipped.height > 0.0
        };
        if let Some(state) = state.as_deref_mut() {
            if state.index >= count || !visible(state.index) {
                state.index = (0..count).find(|&i| visible(i)).unwrap_or(0);
            }
            if let Some(current) = self
                .focused_id()
                .and_then(|v| v.checked_sub(id))
                .and_then(|v| usize::try_from(v).ok())
                .filter(|i| *i < count && visible(*i))
            {
                state.index = current;
                if !self.key_consumed
                    && let Some(key) = self.input.key.filter(|k| k.pressed)
                {
                    let forward = if vertical {
                        key.key == Key::Down
                    } else if rtl {
                        key.key == Key::Left
                    } else {
                        key.key == Key::Right
                    };
                    let backward = if vertical {
                        key.key == Key::Up
                    } else if rtl {
                        key.key == Key::Right
                    } else {
                        key.key == Key::Left
                    };
                    if forward || backward || matches!(key.key, Key::Home | Key::End) {
                        let next = (0..count)
                            .map(|step| {
                                if key.key == Key::Home {
                                    step
                                } else if key.key == Key::End {
                                    count - 1 - step
                                } else if forward {
                                    (current + step + 1) % count
                                } else {
                                    (current + count - 1 - step) % count
                                }
                            })
                            .find(|i| visible(*i));
                        if let Some(next) = next {
                            state.index = next;
                            self.request_focus(id + next as u64);
                            self.key_consumed = true;
                        }
                    }
                }
            }
        }
        let mut activated = None;
        self.within(bounds, |f| {
            for (i, action) in actions.iter().take(count).enumerate() {
                let b = item_bounds(i);
                let r = if let Some(state) = state.as_deref_mut() {
                    f.target_roving(
                        id + i as u64,
                        b,
                        action.label,
                        action.enabled,
                        i == state.index,
                    )
                } else {
                    f.target(id + i as u64, b, action.label, action.enabled, Role::Button)
                };
                if let Some(state) = state.as_deref_mut()
                    && r.clicked
                {
                    state.index = i;
                }
                let is_primary = docked_icons
                    && state
                        .as_ref()
                        .is_some_and(|s| s.primary == Some(i) && s.overflow_trigger != Some(i));
                let ink = if is_primary { t.on_primary } else { foreground };
                if is_primary {
                    f.paint_rect(
                        b,
                        16.0,
                        if r.enabled {
                            t.primary
                        } else {
                            t.text.opacity(0.12)
                        },
                    );
                }
                f.state_layer(r, if is_primary { 16.0 } else { 24.0 }, ink);
                f.focus_ring(r);
                if docked_icons {
                    let state = state.as_ref().unwrap();
                    let icon = if state.overflow_trigger == Some(i) {
                        AppBarIcon::More
                    } else {
                        state.docked_icons[i]
                    };
                    f.commands.push(DrawCommand::Shape {
                        bounds: Rect {
                            x: b.x + (b.width - 24.0) / 2.0,
                            y: b.y + (b.height - 24.0) / 2.0,
                            width: 24.0,
                            height: 24.0,
                        },
                        index: icon.shape_index(rtl),
                        color: if r.enabled { ink } else { t.text.opacity(0.38) },
                        rotation: 0.0,
                    });
                } else {
                    f.styled_text(
                        b,
                        action.label,
                        foreground.opacity(if r.enabled { 1.0 } else { 0.38 }),
                        TextStyle {
                            size: 12.0,
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                }
                if r.clicked {
                    activated = Some(i);
                }
            }
        });
        activated
    }
}

#[cfg(test)]
#[path = "toolbar.spec.rs"]
mod tests;
