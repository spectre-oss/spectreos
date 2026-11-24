use crate::*;

fn update_group_selection(selected:&mut u32,index:usize,total:usize,multiple:bool,required:bool) {
    let bit=1_u32<<index;
    if multiple {
        let valid=if total==32 {u32::MAX} else {(1_u32<<total)-1};
        if *selected&bit==0 || !required || (*selected&valid).count_ones()>1 { *selected^=bit; }
    } else if *selected&bit==0 { *selected=bit; }
    else if !required { *selected=0; }
}
/// Persistent hit geometry for a standard expressive button group.
#[derive(Default)]
pub struct ButtonGroupState {
    bounds: Rect,
    rtl: bool,
    hit_bounds: Vec<Rect>,
    overflow: OverlayState,
    overflow_offset: f32,
    overflow_typeahead: MenuTypeaheadState,
    drag_origin: Option<f32>,
    drag_last: Option<f32>,
    dragging: bool,
    drag_velocity: f32,
    drag_sample_time: f64,
    drag_motion_time: f64,
    fling_velocity: f32,
    fling_curve: crate::scroll::SplineFling,
}

impl Frame<'_> {
    /// Equal-width connected toggle buttons with 2dp spacing. Selection stays
    /// with the caller; returns the activated index. The caller supplies at
    /// least 48dp per target. A group may contain at most 32 items.
    pub fn connected_button_group(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[ToolbarAction<'_>],
        selected: &mut u32,
        multiple: bool,
        selection_required: bool,
    ) -> Option<usize> {
        self.connected_button_group_direction(
            id,
            bounds,
            actions,
            selected,
            multiple,
            selection_required,
            false,
        )
    }
    /// Connected selection group with logical order mirrored for right-to-left layouts.
    #[allow(clippy::too_many_arguments)]
    pub fn connected_button_group_direction(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[ToolbarAction<'_>],
        selected: &mut u32,
        multiple: bool,
        selection_required: bool,
        rtl: bool,
    ) -> Option<usize> {
        self.connected_button_group_inner(id,bounds,actions,selected,multiple,selection_required,rtl,None,actions.len())
    }
    #[allow(clippy::too_many_arguments)]
    fn connected_button_group_inner(
        &mut self, id:u64, bounds:Rect, actions:&[ToolbarAction<'_>], selected:&mut u32,
        multiple:bool, selection_required:bool, rtl:bool, overflow_trigger:Option<usize>, total:usize,
    ) -> Option<usize> {
        let n = actions.len();
        if n == 0
            || n > 32
            || id.checked_add((n - 1) as u64).is_none()
            || !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width < n as f32 * 48.0 + (n - 1) as f32 * 2.0
            || bounds.height < 48.0
        {
            return None;
        }
        let width = (bounds.width - (n - 1) as f32 * 2.0) / n as f32;
        let mut activated = None;
        let mut responses = [Response::default(); 32];
        for (i, action) in actions.iter().enumerate() {
            let hit = Rect {
                x: bounds.x + (if rtl { n - 1 - i } else { i }) as f32 * (width + 2.0),
                y: bounds.y,
                width,
                height: 48.0,
            };
            let r = self.target(
                id + i as u64,
                hit,
                action.label,
                action.enabled,
                if overflow_trigger==Some(i) { Role::Button } else { Role::ToggleButton },
            );
            if r.clicked {
                if overflow_trigger!=Some(i) { update_group_selection(selected,i,total,multiple,selection_required); }
                activated = Some(i);
            }
            responses[i] = r;
        }
        if !self.key_consumed
            && let Some(key) = self.input.key
            && key.pressed
            && matches!(key.key, Key::Left | Key::Right | Key::Home | Key::End)
            && let Some(current) = responses[..n].iter().position(|r| r.focused && r.enabled)
        {
            let next = match key.key {
                Key::Home => (0..n).find(|&i| responses[i].enabled),
                Key::End => (0..n).rev().find(|&i| responses[i].enabled),
                Key::Right | Key::Left => (1..=n)
                    .map(|d| {
                        if (key.key == Key::Right) != rtl {
                            (current + d) % n
                        } else {
                            (current + n - d) % n
                        }
                    })
                    .find(|&i| responses[i].enabled),
                _ => None,
            };
            if let Some(next) = next {
                let next_id = responses[next].id;
                self.request_focus(next_id);
                self.key_consumed = true;
                self.ui.continuous = true;
                for r in &mut responses[..n] {
                    r.focused = r.id == next_id;
                }
                for node in &mut self.ui.semantics {
                    if responses[..n].iter().any(|r| r.id == node.id) {
                        node.focused = node.id == next_id;
                    }
                }
            }
        }
        for (i, action) in actions.iter().enumerate() {
            let r = responses[i];
            let hit = r.bounds;
            let checked = overflow_trigger != Some(i) && *selected & (1_u32 << i) != 0;
            if let Some(node) = self
                .ui
                .semantics
                .iter_mut()
                .rev()
                .find(|node| node.id == r.id)
            {
                node.selected = checked;
                node.checked = Some(if checked {
                    CheckState::Checked
                } else {
                    CheckState::Unchecked
                });
            }
            let target = if r.pressed {
                4.0
            } else if checked {
                20.0
            } else {
                8.0
            };
            let inner = self
                .spatial(r.id, 57, target, if checked { 20.0 } else { 8.0 })
                .clamp(4.0, 20.0);
            let physical = if rtl { n - 1 - i } else { i };
            let left = if physical == 0 { 20.0 } else { inner };
            let right = if physical + 1 == n { 20.0 } else { inner };
            let visual = Rect {
                y: hit.y + 4.0,
                height: 40.0,
                ..hit
            };
            let t = self.ui.theme;
            let surface = if !r.enabled {
                t.text.opacity(0.12)
            } else if checked {
                t.secondary
            } else {
                t.surface_variant
            };
            let ink = if !r.enabled {
                t.text.opacity(0.38)
            } else if checked {
                t.on_secondary
            } else {
                t.text
            };
            self.paint_connected_halves(visual, left, right, surface);
            let hover = self.effect(r.id, 0, f32::from(r.hovered), 0.0);
            let layer = if r.pressed || (r.focused && self.ui.keyboard_focus) {
                0.12
            } else {
                hover * 0.08
            };
            if r.enabled && layer > 0.001 {
                self.paint_connected_halves(visual, left, right, ink.opacity(layer));
            }
            self.connected_ink(r, visual, left, right, ink);
            self.focus_ring(r);
            if overflow_trigger==Some(i) {
                self.commands.push(DrawCommand::Shape { bounds:Rect { x:visual.x+(visual.width-24.0)*0.5, y:visual.y+8.0, width:24.0,height:24.0 }, index:AppBarIcon::More.shape_index(rtl),color:ink,rotation:0.0 });
            } else {
                self.styled_text(visual,action.label,ink,TextStyle { align:1,..TextStyle::LABEL });
            }
        }
        activated
    }

    pub(super) fn connected_ink(&mut self, r: Response, visual: Rect, left: f32, right: f32, ink: Color) {
        let before = self.commands.len();
        self.ink(
            Response {
                bounds: visual,
                ..r
            },
            0.0,
            ink,
        );
        if self.commands.len() > before
            && let Some(DrawCommand::Ripple {
                bounds,
                center,
                radius,
                color,
                soft,
                ..
            }) = self.commands.pop()
        {
            for (x, corner_radius) in [(visual.x, left), (visual.x + visual.width / 2.0, right)] {
                self.commands.push(DrawCommand::PushClip(
                    Rect {
                        x,
                        width: visual.width / 2.0,
                        ..visual
                    }
                    .intersect(self.clip),
                ));
                self.commands.push(DrawCommand::Ripple {
                    bounds,
                    corner_radius,
                    center,
                    radius,
                    color,
                    soft,
                });
                self.commands.push(DrawCommand::PopClip);
            }
        }
    }

    pub(super) fn connected_elevation(&mut self, bounds: Rect, left: f32, right: f32, level: u8) {
        let middle = bounds.x + bounds.width / 2.0;
        for (start, end, radius) in [
            (self.clip.x, middle, left),
            (middle, self.clip.x + self.clip.width, right),
        ] {
            self.commands.push(DrawCommand::PushClip(
                Rect {
                    x: start,
                    y: self.clip.y,
                    width: (end - start).max(0.0),
                    height: self.clip.height,
                }
                .intersect(self.clip),
            ));
            self.elevation(bounds, radius, level);
            self.commands.push(DrawCommand::PopClip);
        }
    }

    pub(super) fn paint_connected_outline(&mut self, b: Rect, left: f32, right: f32, color: Color) {
        use std::f32::consts::{FRAC_PI_2, PI};
        let left = left.clamp(1.0, (b.width.min(b.height) / 2.0).max(1.0));
        let right = right.clamp(1.0, (b.width.min(b.height) / 2.0).max(1.0));
        let x0 = b.x;
        let x1 = b.x + b.width;
        let y0 = b.y;
        let y1 = b.y + b.height;
        for (x, y, r, start) in [
            (x0, y0, left, PI),
            (x1 - 2.0 * right, y0, right, 3.0 * FRAC_PI_2),
            (x1 - 2.0 * right, y1 - 2.0 * right, right, 0.0),
            (x0, y1 - 2.0 * left, left, FRAC_PI_2),
        ] {
            self.commands.push(DrawCommand::Arc {
                bounds: Rect {
                    x,
                    y,
                    width: 2.0 * r,
                    height: 2.0 * r,
                },
                start,
                sweep: FRAC_PI_2,
                stroke: 1.0,
                color,
                rounded: false,
            });
        }
        for (from, to) in [
            ([x0 + left, y0 + 0.5], [x1 - right, y0 + 0.5]),
            ([x1 - 0.5, y0 + right], [x1 - 0.5, y1 - right]),
            ([x1 - right, y1 - 0.5], [x0 + left, y1 - 0.5]),
            ([x0 + 0.5, y1 - left], [x0 + 0.5, y0 + left]),
        ] {
            self.paint_line(from, to, 1.0, color);
        }
    }

    pub(super) fn paint_connected_halves(&mut self, bounds: Rect, left: f32, right: f32, color: Color) {
        for (x, radius) in [(bounds.x, left), (bounds.x + bounds.width / 2.0, right)] {
            self.commands.push(DrawCommand::PushClip(
                Rect {
                    x,
                    width: bounds.width / 2.0,
                    ..bounds
                }
                .intersect(self.clip),
            ));
            self.paint_rect(bounds, radius, color);
            self.commands.push(DrawCommand::PopClip);
        }
    }

    /// Standard equal-width action group. Returns the activated child index.
    /// The caller must provide enough room for 48dp targets and 12dp gaps.
    pub fn button_group(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[ToolbarAction<'_>],
        state: &mut ButtonGroupState,
    ) -> Option<usize> {
        self.button_group_direction(id, bounds, actions, state, false)
    }
    /// Standard group with logical action order mirrored in right-to-left layouts.
    pub fn button_group_direction(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[ToolbarAction<'_>],
        state: &mut ButtonGroupState,
        rtl: bool,
    ) -> Option<usize> {
        self.button_group_inner(id, bounds, actions, state, rtl, None, &[])
    }
    /// Standard group with optional decorative 20dp Material icons preceding labels.
    /// Missing icon entries are treated as text-only actions.
    pub fn button_group_with_icons(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[ToolbarAction<'_>],
        icons: &[Option<AppBarIcon>],
        state: &mut ButtonGroupState,
        rtl: bool,
    ) -> Option<usize> {
        self.button_group_inner(id, bounds, actions, state, rtl, None, icons)
    }
    #[allow(clippy::too_many_arguments)]
    fn button_group_inner(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[ToolbarAction<'_>],
        state: &mut ButtonGroupState,
        rtl: bool,
        overflow_trigger: Option<usize>,
        icons: &[Option<AppBarIcon>],
    ) -> Option<usize> {
        let n = actions.len();
        if n == 0
            || n > 32
            || id.checked_add((n - 1) as u64).is_none()
            || !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width < n as f32 * 48.0 + (n - 1) as f32 * 12.0
            || bounds.height < 48.0
        {
            state.hit_bounds.clear();
            return None;
        }
        let width = (bounds.width - (n - 1) as f32 * 12.0) / n as f32;
        if state.bounds != bounds || state.hit_bounds.len() != n || state.rtl != rtl {
            state.bounds = bounds;
            state.rtl = rtl;
            state.hit_bounds = (0..n)
                .map(|i| Rect {
                    x: bounds.x + (if rtl { n - 1 - i } else { i }) as f32 * (width + 12.0),
                    y: bounds.y,
                    width,
                    height: 48.0,
                })
                .collect();
        }
        let mut responses = [Response::default(); 32];
        let mut progress = [0.0; 32];
        for (i, action) in actions.iter().enumerate() {
            let r = self.target(
                id + i as u64,
                state.hit_bounds[i],
                action.label,
                action.enabled,
                Role::Button,
            );
            progress[i] = self
                .spatial(r.id, 56, f32::from(r.pressed), 0.0)
                .clamp(0.0, 1.0);
            responses[i] = r;
        }
        let mut widths = [width; 32];
        if n > 1 {
            for i in 0..n {
                let requested = width * 0.15 * progress[i];
                let neighbors = usize::from(i > 0) + usize::from(i + 1 < n);
                let share = requested / neighbors as f32;
                for j in [i.checked_sub(1), (i + 1 < n).then_some(i + 1)]
                    .into_iter()
                    .flatten()
                {
                    let delta = share.min((widths[j] - 48.0).max(0.0));
                    widths[j] -= delta;
                    widths[i] += delta;
                }
            }
        }
        let mut x = if rtl {
            bounds.x + bounds.width
        } else {
            bounds.x
        };
        let mut activated = None;
        for (i, mut r) in responses.into_iter().take(n).enumerate() {
            if rtl {
                x -= widths[i];
            }
            let hit = Rect {
                x,
                y: bounds.y,
                width: widths[i],
                height: 48.0,
            };
            state.hit_bounds[i] = hit;
            if let Some(node) = self
                .ui
                .semantics
                .iter_mut()
                .rev()
                .find(|node| node.id == r.id)
            {
                node.bounds = hit.intersect(self.clip);
            }
            r.bounds = hit;
            let visual = Rect {
                x: if overflow_trigger == Some(i) {
                    hit.x + (hit.width - 40.0) / 2.0
                } else {
                    hit.x
                },
                y: hit.y + 4.0,
                width: if overflow_trigger == Some(i) {
                    40.0
                } else {
                    hit.width
                },
                height: 40.0,
            };
            let t = self.ui.theme;
            // Compose small expressive button: full round at rest, 8dp when pressed.
            let radius = 20.0 - 12.0 * progress[i];
            self.paint_rect(
                visual,
                radius,
                if r.enabled && overflow_trigger == Some(i) {
                    t.primary
                } else if r.enabled {
                    t.secondary
                } else {
                    t.text.opacity(0.12)
                },
            );
            self.state_layer(
                Response {
                    bounds: visual,
                    ..r
                },
                radius,
                if r.enabled && overflow_trigger == Some(i) {
                    t.on_primary
                } else if r.enabled {
                    t.on_secondary
                } else {
                    t.text
                },
            );
            self.focus_ring(r);
            let foreground = if r.enabled && overflow_trigger == Some(i) {
                t.on_primary
            } else if r.enabled {
                t.on_secondary
            } else {
                t.text.opacity(0.38)
            };
            if overflow_trigger == Some(i) {
                self.commands.push(DrawCommand::Shape {
                    bounds: visual.inset(8.0),
                    index: AppBarIcon::More.shape_index(rtl),
                    color: foreground,
                    rotation: 0.0,
                });
            } else if let Some(icon) = icons.get(i).copied().flatten() {
                let style = TextStyle::LABEL;
                let text_width = self.text_width(actions[i].label, style);
                let group_width = 20.0 + 8.0 + text_width;
                let start = visual.x + ((visual.width - group_width) / 2.0).max(0.0);
                let icon_x = if rtl { start + text_width + 8.0 } else { start };
                let text_x = if rtl { start } else { start + 28.0 };
                self.commands
                    .push(DrawCommand::PushClip(visual.intersect(self.clip)));
                self.commands.push(DrawCommand::Shape {
                    bounds: Rect {
                        x: icon_x,
                        y: visual.y + 10.0,
                        width: 20.0,
                        height: 20.0,
                    },
                    index: icon.shape_index(rtl),
                    color: foreground,
                    rotation: 0.0,
                });
                self.styled_text(
                    Rect {
                        x: text_x,
                        width: text_width,
                        ..visual
                    },
                    actions[i].label,
                    foreground,
                    style,
                );
                self.commands.push(DrawCommand::PopClip);
            } else {
                self.styled_text(
                    visual,
                    actions[i].label,
                    foreground,
                    TextStyle {
                        align: 1,
                        ..TextStyle::LABEL
                    },
                );
            }
            if r.clicked {
                activated = Some(i);
            }
            if rtl {
                x -= 12.0;
            } else {
                x += widths[i] + 12.0;
            }
        }
        activated
    }
    /// Standard group with an anchored overflow menu. Action IDs remain logical.
    /// Requires at least a 48dp target, at most 32 actions, and reserves 66 IDs.
    pub fn button_group_with_overflow(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[ToolbarAction<'_>],
        state: &mut ButtonGroupState,
        rtl: bool,
    ) -> Option<usize> {
        self.button_group_with_overflow_icons(id, bounds, actions, &[], state, rtl)
    }
    /// Standard overflow group retaining optional icons on visible and menu actions.
    pub fn button_group_with_overflow_icons(
        &mut self,
        id: u64,
        bounds: Rect,
        actions: &[ToolbarAction<'_>],
        icons: &[Option<AppBarIcon>],
        state: &mut ButtonGroupState,
        rtl: bool,
    ) -> Option<usize> {
        self.button_group_overflow_inner(id,bounds,actions,icons,state,rtl,None)
    }
    /// Connected single- or multi-selection group with an anchored overflow menu.
    #[allow(clippy::too_many_arguments)]
    pub fn connected_button_group_with_overflow(
        &mut self,id:u64,bounds:Rect,actions:&[ToolbarAction<'_>],selected:&mut u32,
        multiple:bool,required:bool,state:&mut ButtonGroupState,rtl:bool,
    ) -> Option<usize> {
        self.button_group_overflow_inner(id,bounds,actions,&[],state,rtl,Some((selected,multiple,required)))
    }
    #[allow(clippy::too_many_arguments)]
    fn button_group_overflow_inner(
        &mut self,id:u64,bounds:Rect,actions:&[ToolbarAction<'_>],icons:&[Option<AppBarIcon>],
        state:&mut ButtonGroupState,rtl:bool,mut connected:Option<(&mut u32,bool,bool)>,
    ) -> Option<usize> {
        if actions.is_empty()
            || actions.len() > 32
            || id.checked_add(66).is_none()
            || !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width < 48.0
            || bounds.height < 48.0
        {
            state.overflow.open = false;
            state.hit_bounds.clear();
            return None;
        }
        let gap=if connected.is_some() {2.0} else {12.0};
        let slots = (((bounds.width + gap) / (48.0+gap)).floor() as usize).min(actions.len());
        if slots >= actions.len() {
            if state.overflow.open {
                state.overflow = OverlayState::default();
                if let Some(i) = actions.iter().position(|action| action.enabled) {
                    self.request_focus(id + i as u64);
                }
            }
            state.overflow_typeahead.reset();
            return if let Some((selected,multiple,required))=connected {
                self.connected_button_group_direction(id,bounds,actions,selected,multiple,required,rtl)
            } else { self.button_group_with_icons(id,bounds,actions,icons,state,rtl) };
        }
        let visible = slots.saturating_sub(1);
        let mut shown = actions[..visible].to_vec();
        shown.push(ToolbarAction {
            label: "More",
            enabled: true,
        });
        let mut activated = None;
        self.without_input(state.overflow.open, |f| {
            if let Some((selected,multiple,required))=connected.as_mut() {
                activated = f.connected_button_group_inner(id,bounds,&shown,selected,*multiple,*required,rtl,Some(visible),actions.len());
                let n=shown.len();let width=(bounds.width-(n-1) as f32*2.0)/n as f32;
                state.hit_bounds=(0..n).map(|i| Rect { x:bounds.x+(if rtl {n-1-i} else {i}) as f32*(width+2.0),y:bounds.y,width,height:48.0 }).collect();
            } else { activated = f.button_group_inner(id, bounds, &shown, state, rtl, Some(visible), icons); }
        });
        if activated == Some(visible) {
            activated = None;
            state.overflow.open = true;
            state.overflow_offset = 0.0;
        }
        let trigger = state.hit_bounds[visible];
        let viewport = self.viewport();
        let remaining = &actions[visible..];
        let width = remaining
            .iter()
            .enumerate()
            .map(|(i, a)| self.text_width(a.label, TextStyle::LABEL) + if connected.is_some() || icons.get(visible+i).copied().flatten().is_some() { 80.0 } else { 48.0 })
            .fold(112.0_f32, f32::max)
            .min(280.0)
            .min(viewport.width);
        let below = (viewport.y + viewport.height - trigger.y - trigger.height).max(0.0);
        let above = (trigger.y - viewport.y).max(0.0);
        let down = below >= above;
        let height = (remaining.len() as f32 * 48.0 + 16.0).min(if down { below } else { above });
        let menu = Rect {
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
        state.overflow_offset = state.overflow_offset.clamp(0.0, max_offset);
        if !state.overflow.open
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
            let old = state.overflow_offset;
            state.overflow_offset = (old + movement).clamp(0.0, max_offset);
            if !moving { state.fling_velocity = 0.; }
            if state.fling_velocity.abs() < 20.0
                || (state.overflow_offset - old - movement).abs() > 0.5
            {
                state.fling_velocity = 0.0;
            } else {
                self.ui.continuous = true;
            }
        }
        let first_id = id + 33;
        if state.overflow.open {
            state.overflow_offset =
                (state.overflow_offset + self.scroll_y(menu)).clamp(0.0, max_offset);
            if max_offset > 0.0 && self.input.pointer_kind == PointerKind::Touch {
                if self.input.pressed && self.pointer_in(menu) {
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
                        state.overflow_offset =
                            (state.overflow_offset + movement).clamp(0.0, max_offset);
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
            if let Some(current) = self
                .focused_id()
                .and_then(|v| v.checked_sub(first_id))
                .and_then(|v| usize::try_from(v).ok())
                .filter(|i| *i < remaining.len())
            {
                let key = self.input.key.filter(|k| k.pressed).map(|k| k.key);
                if !self.key_consumed
                    && matches!(key, Some(Key::Up | Key::Down | Key::Home | Key::End))
                {
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
                        if top < state.overflow_offset {
                            state.overflow_offset = top;
                        } else if top + 48.0 > state.overflow_offset + body {
                            state.overflow_offset = (top + 48.0 - body).min(max_offset);
                        }
                        self.request_focus(first_id + next as u64);
                        self.key_consumed = true;
                    }
                }
            }
            let items: Vec<_> = remaining.iter().map(|a| (a.label, a.enabled)).collect();
            if let Some(next) = self.menu_typeahead(&mut state.overflow_typeahead, first_id, &items)
            {
                let top = next as f32 * 48.0;
                if top < state.overflow_offset {
                    state.overflow_offset = top;
                } else if top + 48.0 > state.overflow_offset + body {
                    state.overflow_offset = (top + 48.0 - body).min(max_offset);
                }
            }
        } else {
            state.overflow_typeahead.reset();
        }
        if self.input.released || self.input.cancelled || self.input.blurred || !state.overflow.open
        {
            if state.dragging {
                self.ui.active = None;
                if self.input.released
                    && !self.input.cancelled
                    && !self.input.blurred
                    && state.overflow.open
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
        state.overflow.kind = OverlayKind::Menu;
        state.overflow.style = OverlayStyle::AndroidMenu;
        state.overflow.initial_focus = remaining
            .iter()
            .position(|a| a.enabled)
            .map(|i| first_id + i as u64);
        let start = self.commands.len();
        self.overlay(id + 66, &mut state.overflow, menu, false, |f, open| {
            let content = f.available();
            f.within(content, |f| {
                f.within(
                    Rect {
                        y: content.y - state.overflow_offset,
                        height: remaining.len() as f32 * 48.0,
                        ..content
                    },
                    |f| {
                        f.with_gap(0.0, |f| {
                            for (i, action) in remaining.iter().enumerate() {
                                let response = if let Some((selected,_,_))=connected.as_ref() {
                                    f.menu_item_with_selection(first_id+i as u64,action.label,action.enabled,**selected & (1_u32 << (visible+i)) != 0,rtl)
                                } else if let Some(icon) = icons.get(visible+i).copied().flatten() {
                                    f.menu_item_with_icon(first_id+i as u64, action.label, action.enabled, icon, rtl)
                                } else {
                                    f.menu_item(first_id+i as u64, action.label, action.enabled)
                                };
                                if response.clicked {
                                    if let Some((selected,multiple,required))=connected.as_mut() { update_group_selection(selected,visible+i,actions.len(),*multiple,*required); }
                                    activated = Some(visible + i);
                                    *open = false;
                                }
                            }
                        });
                    },
                );
            });
        });
        self.ui.overlays.extend(self.commands.drain(start..));
        activated
    }
}

#[cfg(test)]
#[path = "button_group.spec.rs"]
mod tests;
