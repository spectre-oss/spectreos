use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayKind {
    #[default]
    Popover,
    Menu,
    Dialog,
    BottomSheet,
    SideSheet,
    /// A modal surface entering from the left edge.
    LeftSideSheet,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayStyle {
    #[default]
    Material3,
    AngularMaterial,
    AngularDatepicker,
    AngularTimepicker,
    /// Surface and content geometry for an expanded modal navigation rail.
    NavigationRail,
    /// Material modal navigation drawer, with leading edge attached to the viewport.
    NavigationDrawer,
    /// Android Material popup menu surface.
    AndroidMenu,
}

/// Which implicit gestures may dismiss an overlay. Explicit actions may still
/// close it regardless of this setting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayDismissPolicy {
    #[default]
    EscapeOrOutside,
    EscapeOnly,
    OutsideOnly,
    ExplicitOnly,
}
impl OverlayDismissPolicy {
    fn escape(self) -> bool { matches!(self, Self::EscapeOrOutside | Self::EscapeOnly) }
    fn outside(self) -> bool { matches!(self, Self::EscapeOrOutside | Self::OutsideOnly) }
}

#[derive(Clone, Copy)]
pub(crate) struct OverlayPhase {
    id: u64,
    open: bool,
    kind: OverlayKind,
}
#[derive(Clone, Copy)]
pub enum OverlaySection {
    Headline,
    Content,
    Actions,
}

#[derive(Clone,Copy)]
struct BottomSheetNestedTouch {origin:f32,last:f32,last_time:f64,velocity:f32,claimed:bool,moved_sheet:bool}
fn sheet_detent(current:f32,collapsed:f32,half:Option<f32>,xvel:f32,yvel:f32)->f32 {
    let mut detents=vec![0.0];if let Some(value)=half {detents.push(value);}detents.push(collapsed);
    if yvel < -500.0 && yvel.abs()>xvel.abs() {detents.iter().rev().copied().find(|&v|v<current-0.01).unwrap_or(0.0)}
    else if yvel>500.0 && yvel.abs()>xvel.abs() {detents.iter().copied().find(|&v|v>current+0.01).unwrap_or(collapsed)}
    else {detents.iter().copied().min_by(|a,b|(a-current).abs().total_cmp(&(b-current).abs())).unwrap_or(0.0)}
}
// Material Android distinguishes handle release from nested-scroll stop.
fn sheet_handle_detent(current:f32,collapsed:f32,half:Option<f32>,expanded_top:f32,xvel:f32,yvel:f32)->f32 {
    if yvel<0.0 {
        return half.filter(|&h|current>h).unwrap_or(0.0);
    }
    if yvel==0.0 || xvel.abs()>yvel.abs() {
        if let Some(h)=half {
            if current<h {
                return if expanded_top+current < (current-collapsed).abs() {0.0} else {h};
            }
            return if (current-h).abs() < (current-collapsed).abs() {h} else {collapsed};
        }
        return if current < (current-collapsed).abs() {0.0} else {collapsed};
    }
    half.filter(|&h|(current-h).abs()<(current-collapsed).abs()).unwrap_or(collapsed)
}
/// Caller-owned overlay lifetime and focus return point.
#[derive(Default,Clone)]
pub struct OverlayState {
    pub open: bool,
    pub kind: OverlayKind,
    pub style: OverlayStyle,
    pub initial_focus: Option<u64>,
    pub dismiss_policy: OverlayDismissPolicy,
    /// Accessible title and description for a modal dialog.
    pub accessible_title: Option<String>,
    pub accessible_description: Option<String>,
    /// Fill the host viewport when this overlay is a dialog.
    pub fullscreen: bool,
    /// Temporarily suspend this overlay while a separately rendered child modal
    /// owns input. The caller clears this when the child has closed.
    pub suspended: bool,
    pub transform_origin: Option<[f32; 2]>,
    /// Allow dragging a Material bottom sheet from its top 32 logical pixels.
    pub drag_to_dismiss: bool,
    /// When set, a non-modal sheet settles between this visible peek height and full height.
    pub bottom_sheet_peek: Option<f32>,
    /// Current target detent for a sheet with a peek height.
    pub bottom_sheet_expanded: bool,
    /// Optional visible viewport-height fraction for an intermediate detent.
    pub bottom_sheet_half_ratio: Option<f32>,
    /// Whether the current target is the intermediate detent.
    pub bottom_sheet_half_expanded: bool,
    /// Offset of the scrollable region below the sheet's top; None disables nested scrolling.
    pub bottom_sheet_content_top: Option<f32>,
    /// Total scrollable content height in logical pixels.
    pub bottom_sheet_content_height: f32,
    /// Current nested content scroll position.
    pub bottom_sheet_scroll_offset: f32,
    nested_touch: Option<BottomSheetNestedTouch>,
    nested_physics: crate::scroll::ScrollPhysics,
    drag_origin: Option<(f32,f32)>,
    side_drag_origin: Option<(f32,f32,f32,bool)>,
    drag_last_motion: Option<(f64,[f32;2])>,
    drag_velocity: [f32;2],
    drag_displacement: f32,
    drag_settle: Option<(f64,f32,f32)>,
    drag_close_offset: f32,
    geometry: crate::timing::Tween,
    surface: crate::timing::Tween,
    scrim: crate::timing::Tween,
    content: crate::timing::Tween,
    was_open: bool,
    sampled_open: bool,
    opened_with_keyboard: bool,
    restore: Option<u64>,
}

impl OverlayState {
    fn sample(&mut self, time: f64, reduced: bool) -> [f32; 4] {
        use crate::timing::{Easing, TweenSpec};
        let target = f32::from(self.open);
        if self.style == OverlayStyle::AngularMaterial && self.kind == OverlayKind::BottomSheet {
            let geometry = self.geometry.sample(
                time,
                target,
                TweenSpec::new(0.195, Easing::Cubic([0.0, 0.0, 0.2, 1.0])),
                TweenSpec::new(0.375, Easing::Cubic([0.4, 0.0, 1.0, 1.0])),
                reduced,
            );
            let scrim = self.scrim.sample(
                time,
                target,
                TweenSpec::new(0.4, Easing::Backdrop),
                TweenSpec::new(0.4, Easing::Backdrop),
                reduced,
            );
            let visible = f32::from(self.open || self.geometry.moving());
            return [geometry, visible, scrim, visible];
        }
        if matches!(
            self.style,
            OverlayStyle::AngularDatepicker | OverlayStyle::AngularTimepicker
        ) {
            let spec = TweenSpec::new(0.120, Easing::StandardDecelerate);
            if self.open != self.sampled_open {
                if !self.open {
                    self.geometry.sample(time, 1.0, spec, spec, reduced);
                }
                let interrupted = !self.open && self.geometry.moving();
                let initial = if self.open || interrupted { 0.0 } else { 1.0 };
                self.geometry = crate::timing::Tween::at(initial);
                self.surface = crate::timing::Tween::at(initial);
                self.sampled_open = self.open;
            }
            let geometry = self.geometry.sample(
                time,
                target,
                spec,
                TweenSpec::new(0.0, Easing::Linear),
                reduced,
            );
            let alpha = self.surface.sample(
                time,
                target,
                spec,
                TweenSpec::new(0.100, Easing::Linear),
                reduced,
            );
            return [if self.open { geometry } else { 1.0 }, alpha, 0.0, alpha];
        }
        if matches!(self.style, OverlayStyle::AngularMaterial | OverlayStyle::AndroidMenu) {
            let menu = self.kind == OverlayKind::Menu;
            if menu {
                if self.open != self.sampled_open {
                    let interrupted = if !self.open && self.sampled_open {
                        let spec = TweenSpec::new(0.120, Easing::StandardDecelerate);
                        self.geometry.sample(time, 1.0, spec, spec, reduced);
                        self.geometry.moving()
                    } else {
                        false
                    };
                    // Angular's animationcancel notification detaches a panel
                    // closed before its enter animation finishes.
                    let initial = if self.open || interrupted { 0.0 } else { 1.0 };
                    self.geometry = crate::timing::Tween::at(initial);
                    self.surface = crate::timing::Tween::at(initial);
                    self.sampled_open = self.open;
                }
                self.scrim = crate::timing::Tween::at(0.0);
                self.content = crate::timing::Tween::at(0.0);
            }
            let enter = if menu { 0.120 } else { 0.150 };
            let exit = if menu { 0.100 } else { 0.075 };
            let g = self.geometry.sample(
                time,
                target,
                TweenSpec::new(enter, Easing::StandardDecelerate),
                TweenSpec::new(0.0, Easing::Linear),
                reduced,
            );
            let a = self.surface.sample(
                time,
                target,
                TweenSpec::new(
                    enter,
                    if menu {
                        Easing::StandardDecelerate
                    } else {
                        Easing::Linear
                    },
                ),
                TweenSpec {
                    delay: if menu { 0.025 } else { 0.0 },
                    duration: exit,
                    easing: Easing::Linear,
                },
                reduced,
            );
            let scrim = if menu {
                0.0
            } else {
                self.scrim.sample(
                    time,
                    target,
                    TweenSpec::new(0.4, Easing::Backdrop),
                    TweenSpec::new(0.4, Easing::Backdrop),
                    reduced,
                )
            };
            return [if self.open { g } else { 1.0 }, a, scrim, a];
        }
        let geometry = self.geometry.sample(
            time,
            target,
            TweenSpec::new(0.5, Easing::Emphasized),
            TweenSpec::new(0.15, Easing::EmphasizedAccelerate),
            reduced,
        );
        let surface = self.surface.sample(
            time,
            target,
            TweenSpec::new(0.05, Easing::Linear),
            TweenSpec {
                delay: 0.1,
                duration: 0.05,
                easing: Easing::Linear,
            },
            reduced,
        );
        let scrim = self.scrim.sample(
            time,
            target,
            TweenSpec::new(0.5, Easing::Linear),
            TweenSpec::new(0.15, Easing::Linear),
            reduced,
        );
        let content = self.content.sample(
            time,
            target,
            TweenSpec {
                delay: 0.05,
                duration: 0.2,
                easing: Easing::Linear,
            },
            TweenSpec::new(0.1, Easing::Linear),
            reduced,
        );
        [geometry, surface, scrim, content]
    }
    pub(crate) fn moving(&self) -> bool {
        self.geometry.moving()
            || self.surface.moving()
            || self.scrim.moving()
            || self.content.moving()
    }
}

impl Frame<'_> {
    /// Source Web menu entry/exit stagger; each item retargets from its presentation.
    pub fn overlay_item(&mut self, index: usize, count: usize, build: impl FnOnce(&mut Self)) {
        use crate::timing::{Easing, TweenSpec};
        if let Some(phase) = self.overlay_phase.filter(|p| p.kind == OverlayKind::Menu) {
            let count = count.clamp(1, 64);
            let index = index.min(count - 1);
            let enter = TweenSpec {
                delay: 0.250 / count as f64 * index as f64,
                duration: 0.250,
                easing: Easing::Linear,
            };
            let exit = TweenSpec {
                delay: 0.050 + 0.050 / count as f64 * (count - 1 - index) as f64,
                duration: 0.050,
                easing: Easing::Linear,
            };
            let alpha = self.ui.motion.tween(
                phase.id,
                100 + index as u8,
                f32::from(phase.open),
                0.0,
                enter,
                exit,
            );
            self.with_opacity(alpha, build);
        } else {
            build(self);
        }
    }
    pub fn overlay_section(&mut self, section: OverlaySection, build: impl FnOnce(&mut Self)) {
        use crate::timing::{Easing, TweenSpec};
        if let Some(phase) = self.overlay_phase.filter(|p| p.kind == OverlayKind::Dialog) {
            let (property, delay, duration, exit) = match section {
                OverlaySection::Headline => (80, 0.050, 0.200, 0.100),
                OverlaySection::Content => (81, 0.050, 0.200, 0.100),
                OverlaySection::Actions => (82, 0.150, 0.150, 0.050),
            };
            let alpha = self.ui.motion.tween(
                phase.id,
                property,
                f32::from(phase.open),
                0.0,
                TweenSpec {
                    delay,
                    duration,
                    easing: Easing::Linear,
                },
                TweenSpec::new(exit, Easing::Linear),
            );
            self.with_opacity(alpha, build);
        } else {
            build(self);
        }
    }
    /// Modal surface: disable background while `open`, then call this after the trigger.
    /// Exit visuals never receive input; focus is restored immediately on dismissal.
    pub fn overlay(
        &mut self,
        id: u64,
        state: &mut OverlayState,
        bounds: Rect,
        modal: bool,
        mut build: impl FnMut(&mut Self, &mut bool),
    ) {
        if state.open && !state.was_open {
            state.opened_with_keyboard = self.ui.keyboard_focus || self.input.key.is_some();
            state.restore = self.ui.focused;
            self.ui.focused = None;
        }
        let timed = matches!(state.kind, OverlayKind::Menu | OverlayKind::Dialog)
            || state.style == OverlayStyle::AngularMaterial
                && state.kind == OverlayKind::BottomSheet
            || matches!(
                state.style,
                OverlayStyle::AngularDatepicker | OverlayStyle::AngularTimepicker
            );
        let mut values = if timed {
            state.sample(self.ui.time, self.reduced_motion())
        } else {
            let position = self.spatial(id, 21, f32::from(state.open), 0.0);
            let alpha = self.animate(id, 20, f32::from(state.open), 0.0);
            [position, alpha, alpha, alpha]
        };
        let kind = state.kind;
        let fullscreen = kind == OverlayKind::Dialog && state.fullscreen;
        let full = if fullscreen { self.viewport() } else { bounds };
        let angular = matches!(
            state.style,
            OverlayStyle::AngularMaterial
                | OverlayStyle::AndroidMenu
                | OverlayStyle::AngularDatepicker
                | OverlayStyle::AngularTimepicker
        );
        let datepicker = matches!(
            state.style,
            OverlayStyle::AngularDatepicker | OverlayStyle::AngularTimepicker
        );
        let timepicker = state.style == OverlayStyle::AngularTimepicker;
        let draggable=state.drag_to_dismiss && kind==OverlayKind::BottomSheet && !angular && !state.suspended;
        let side_draggable=modal && matches!(kind,OverlayKind::SideSheet | OverlayKind::LeftSideSheet) && !angular && !state.suspended && (state.drag_to_dismiss || state.dismiss_policy.outside());
        let side_dir=if kind==OverlayKind::SideSheet {1.0} else {-1.0};
        let collapsed_offset=state.bottom_sheet_peek.filter(|v|v.is_finite()).map(|peek|(full.height-peek.clamp(0.0,full.height)).max(0.0));
        let viewport=self.viewport();
        let half_offset=state.bottom_sheet_half_ratio.filter(|r|r.is_finite() && *r>0.0 && *r<1.0).and_then(|ratio| {
            let offset=viewport.y+viewport.height*(1.0-ratio)-full.y;
            collapsed_offset.filter(|&collapsed|offset>0.01 && offset<collapsed-0.01).map(|_|offset)
        });
        let target_detent=if state.bottom_sheet_expanded {0.0} else if state.bottom_sheet_half_expanded {half_offset.unwrap_or(collapsed_offset.unwrap_or(0.0))} else {collapsed_offset.unwrap_or(0.0)};
        if draggable && !state.open && state.was_open && collapsed_offset.is_some() {state.drag_close_offset=state.drag_displacement;}
        if draggable && state.open {
            if !state.was_open && state.drag_close_offset==0.0 && let Some(collapsed)=collapsed_offset {
                state.drag_displacement=target_detent.min(collapsed);
            }
            if state.drag_close_offset>0.0 {
                state.drag_displacement=state.drag_close_offset*values[0].clamp(0.0,1.0);
                state.drag_close_offset=0.0;state.drag_settle=Some((self.ui.time,state.drag_displacement,target_detent));
            }
            // The caller can change the target or sheet geometry while it is open.
            // Retarget from the current position without disturbing an active drag.
            if collapsed_offset.is_some() && state.was_open && state.drag_origin.is_none() && state.nested_touch.is_none() && !self.input.pressed {
                match state.drag_settle {
                    Some((_,_,target)) if (target-target_detent).abs()>0.01 => {
                        state.drag_settle=Some((self.ui.time,state.drag_displacement,target_detent));
                    }
                    None if (state.drag_displacement-target_detent).abs()>0.01 => {
                        state.drag_settle=Some((self.ui.time,state.drag_displacement,target_detent));
                    }
                    _=>{}
                }
            }
            if let Some((started,from,target))=state.drag_settle {
                let elapsed=(self.ui.time-started).max(0.0);let factor=if self.reduced_motion() {0.0} else {((1.0+18.0*elapsed)*(-18.0*elapsed).exp()) as f32};
                state.drag_displacement=target+(from-target)*factor;
                if factor<0.001 {state.drag_settle=None;state.drag_displacement=target;} else {self.ui.continuous=true;}
            }
            if self.input.pressed && values[0]>0.95 && let Some(point)=self.input.pointer {
                let top=full.y+full.height*(1.0-values[0])+state.drag_displacement;
                if point[0]>=full.x && point[0]<full.x+full.width && point[1]>=top && point[1]<top+32.0 {
                    state.drag_origin=Some((point[1],state.drag_displacement));state.drag_last_motion=Some((self.ui.time,point));state.drag_velocity=[0.0;2];state.drag_settle=None;self.ui.active=None;
                }
            }
            if let Some((origin,initial))=state.drag_origin {
                if let Some(point)=self.input.pointer {
                    state.drag_displacement=(initial+point[1]-origin).clamp(0.0,collapsed_offset.unwrap_or(full.height));
                    if let Some((last_time,last_point))=state.drag_last_motion && point!=last_point {
                        let dt=self.ui.time-last_time;
                        if dt>0.0 {state.drag_velocity=[((point[0]-last_point[0])/dt as f32).clamp(-10000.0,10000.0),((point[1]-last_point[1])/dt as f32).clamp(-10000.0,10000.0)];}
                        state.drag_last_motion=Some((self.ui.time,point));
                    }
                }
                if self.input.cancelled || self.input.blurred || self.input.released {
                    state.drag_origin=None;self.ui.active=None;
                    let [xvel,yvel]=if state.drag_last_motion.is_some_and(|(last,_)|self.ui.time-last<=0.12) {state.drag_velocity} else {[0.0;2]};
                    state.drag_last_motion=None;state.drag_velocity=[0.0;2];
                    // Material Android's default significant velocity is 500 px/s; its
                    // hide projection uses 0.1 s friction and a half-peek threshold.
                    let flick=yvel>500.0 && yvel.abs()>xvel.abs() && state.drag_displacement+yvel*0.1>full.height*0.5;
                    let distance=state.drag_displacement>=full.height*0.4 && yvel>=-50.0;
                    if let Some(collapsed)=collapsed_offset {
                        let current=state.drag_displacement;
                        let target=if self.input.cancelled || self.input.blurred {target_detent} else {sheet_handle_detent(current,collapsed,half_offset,full.y-viewport.y,xvel,yvel)};
                        state.bottom_sheet_expanded=target<=0.01;state.bottom_sheet_half_expanded=half_offset.is_some_and(|half|(target-half).abs()<0.01);
                        state.drag_settle=Some((self.ui.time,current,target));self.ui.continuous=true;
                    } else if !self.input.cancelled && !self.input.blurred && (distance || flick) {
                        state.open=false;state.drag_close_offset=state.drag_displacement;state.drag_displacement=0.0;state.drag_settle=None;
                        values=[self.spatial(id,21,0.0,0.0),self.animate(id,20,0.0,0.0),values[2],values[3]];self.ui.continuous=true;
                    } else {state.drag_settle=Some((self.ui.time,state.drag_displacement,0.0));self.ui.continuous=true;}
                }
            }
        }
        let mut side_drag_claimed=false;
        if side_draggable && state.open {
            if state.drag_close_offset>0.0 {
                state.drag_displacement=state.drag_close_offset*values[0].clamp(0.0,1.0);
                state.drag_close_offset=0.0;state.drag_settle=Some((self.ui.time,state.drag_displacement,0.0));
            }
            if let Some((started,from,target))=state.drag_settle {
                let elapsed=(self.ui.time-started).max(0.0);let factor=if self.reduced_motion() {0.0} else {((1.0+18.0*elapsed)*(-18.0*elapsed).exp()) as f32};
                state.drag_displacement=target+(from-target)*factor;
                if factor<0.001 {state.drag_settle=None;state.drag_displacement=target;} else {self.ui.continuous=true;}
            }
            if self.input.pressed && values[0]>0.95 && let Some(point)=self.input.pointer {
                let cur_x=if kind==OverlayKind::SideSheet {
                    full.x+full.width*(1.0-values[0])+state.drag_displacement
                } else {
                    full.x-full.width*(1.0-values[0])-state.drag_displacement
                };
                let cur_rect=Rect {x:cur_x,..full};
                if cur_rect.contains(point) {
                    let already_displaced=state.drag_displacement>0.01;
                    state.side_drag_origin=Some((point[0],point[1],state.drag_displacement,already_displaced));
                    state.drag_last_motion=Some((self.ui.time,point));state.drag_velocity=[0.0;2];state.drag_settle=None;
                    if already_displaced {self.ui.active=None;}
                }
            }
            if let Some((ox,oy,initial,mut claimed))=state.side_drag_origin {
                if let Some(point)=self.input.pointer {
                    let dx=point[0]-ox;let dy=point[1]-oy;
                    if !claimed && dx*side_dir>6.0 && dx.abs()>dy.abs() {claimed=true;self.ui.active=None;}
                    if claimed {
                        state.drag_displacement=(initial+dx*side_dir).clamp(0.0,full.width);
                        self.ui.continuous=true;
                    }
                    if let Some((last_time,last_point))=state.drag_last_motion && point!=last_point {
                        let dt=self.ui.time-last_time;
                        if dt>0.0 {state.drag_velocity=[((point[0]-last_point[0])/dt as f32).clamp(-10000.0,10000.0),((point[1]-last_point[1])/dt as f32).clamp(-10000.0,10000.0)];}
                        state.drag_last_motion=Some((self.ui.time,point));
                    }
                }
                side_drag_claimed=claimed;
                if self.input.cancelled || self.input.blurred || self.input.released {
                    state.side_drag_origin=None;
                    let [xvel,yvel]=if state.drag_last_motion.is_some_and(|(last,_)|self.ui.time-last<=0.12) {state.drag_velocity} else {[0.0;2]};
                    state.drag_last_motion=None;state.drag_velocity=[0.0;2];
                    if claimed {
                        self.ui.active=None;
                        let outward_vel=xvel*side_dir;
                        let flick=outward_vel>500.0 && xvel.abs()>yvel.abs() && state.drag_displacement+outward_vel*0.1>full.width*0.5;
                        let distance=state.drag_displacement>=full.width*0.4 && outward_vel>=-50.0;
                        if !self.input.cancelled && !self.input.blurred && (distance || flick) {
                            state.open=false;state.drag_close_offset=state.drag_displacement;state.drag_displacement=0.0;state.drag_settle=None;
                            values=[self.spatial(id,21,0.0,0.0),self.animate(id,20,0.0,0.0),values[2],values[3]];self.ui.continuous=true;
                        } else {
                            state.drag_settle=Some((self.ui.time,state.drag_displacement,0.0));self.ui.continuous=true;
                        }
                    }
                } else {
                    state.side_drag_origin=Some((ox,oy,initial,claimed));
                }
            }
        }
        let mut nested_claimed=false;
        if draggable && state.open && let (Some(collapsed),Some(content_top))=(collapsed_offset,state.bottom_sheet_content_top.filter(|v|v.is_finite() && *v>=32.0 && *v<full.height)) {
            let nested_height=(full.height-content_top).max(0.0);let max=(state.bottom_sheet_content_height-nested_height).max(0.0);
            if (state.nested_physics.offset-state.bottom_sheet_scroll_offset).abs()>0.1 && state.nested_touch.is_none() {state.nested_physics.cancel();state.nested_physics.offset=state.bottom_sheet_scroll_offset.clamp(0.0,max);}
            if state.nested_touch.is_none() && state.nested_physics.advance(self.ui.time,max,self.reduced_motion()) {self.ui.continuous=true;}
            state.bottom_sheet_scroll_offset=state.nested_physics.offset.clamp(0.0,max);
            let sheet_top=full.y+full.height*(1.0-values[0])+state.drag_displacement;
            if self.input.pressed && values[0]>0.95 && let Some(point)=self.input.pointer {
                if point[0]>=full.x && point[0]<full.x+full.width && point[1]>=sheet_top+content_top && point[1]<(sheet_top+full.height).min(viewport.y+viewport.height) {
                    state.nested_touch=Some(BottomSheetNestedTouch {origin:point[1],last:point[1],last_time:self.ui.time,velocity:0.0,claimed:false,moved_sheet:false});
                    state.nested_physics.cancel();state.drag_settle=None;
                }
            }
            if let Some(mut touch)=state.nested_touch {
                if let Some(point)=self.input.pointer {
                    let delta=touch.last-point[1];touch.claimed|=(point[1]-touch.origin).abs()>6.0;
                    if touch.claimed && delta.abs()>0.01 {
                        let dt=(self.ui.time-touch.last_time).clamp(0.001,0.15) as f32;touch.velocity=(delta/dt).clamp(-8000.0,8000.0);
                        if delta>0.0 {
                            let sheet=delta.min(state.drag_displacement);state.drag_displacement-=sheet;touch.moved_sheet|=sheet>0.0;
                            state.bottom_sheet_scroll_offset=(state.bottom_sheet_scroll_offset+delta-sheet).clamp(0.0,max);
                        } else {
                            let scroll=(-delta).min(state.bottom_sheet_scroll_offset);state.bottom_sheet_scroll_offset-=scroll;
                            let before=state.drag_displacement;state.drag_displacement=(before-delta-scroll).min(collapsed);touch.moved_sheet|=state.drag_displacement>before;
                        }
                        state.nested_physics.offset=state.bottom_sheet_scroll_offset;self.ui.active=None;
                    }
                    if delta.abs()>0.01 {touch.last_time=self.ui.time;}touch.last=point[1];
                }
                nested_claimed=touch.claimed;
                if self.input.released || self.input.cancelled || self.input.blurred {
                    if touch.claimed {
                        if touch.moved_sheet && (state.drag_displacement>0.01 || self.input.cancelled || self.input.blurred) {
                            // BottomSheetBehavior.onStopNestedScroll uses the last touch
                            // direction, not the child's later non-touch fling velocity.
                            let current=state.drag_displacement;
                            let target=if self.input.cancelled || self.input.blurred {target_detent}
                                else if current<=0.01 {0.0}
                                else if touch.velocity>0.0 {half_offset.filter(|half|current>*half).unwrap_or(0.0)}
                                else if touch.velocity<0.0 {half_offset.filter(|half|(current-half).abs()<(current-collapsed).abs()).unwrap_or(collapsed)}
                                else {sheet_detent(current,collapsed,half_offset,0.0,0.0)};
                            state.bottom_sheet_expanded=target<=0.01;state.bottom_sheet_half_expanded=half_offset.is_some_and(|half|(target-half).abs()<0.01);
                            state.drag_settle=Some((self.ui.time,state.drag_displacement,target));self.ui.continuous=true;
                        } else if !self.input.cancelled && !self.input.blurred && self.ui.time-touch.last_time<=0.12 {
                            state.bottom_sheet_expanded=true;state.bottom_sheet_half_expanded=false;
                            state.nested_physics.fling_from_velocity(touch.velocity,self.ui.time,self.reduced_motion());self.ui.continuous=true;
                        }
                    }
                    state.nested_touch=None;
                } else {state.nested_touch=Some(touch);}
            }
        } else if !state.open {state.nested_touch=None;state.nested_physics.cancel();}
        let drag_offset=if draggable && state.open {state.drag_displacement} else if draggable {state.drag_close_offset*values[0].clamp(0.0,1.0)} else {0.0};
        let side_drag_offset=if side_draggable && state.open {state.drag_displacement} else if side_draggable {state.drag_close_offset*values[0].clamp(0.0,1.0)} else {0.0};
        if !state.open && values[0]<0.001 {state.drag_close_offset=0.0;state.drag_displacement=0.0;state.drag_settle=None;state.drag_origin=None;state.side_drag_origin=None;state.drag_last_motion=None;state.drag_velocity=[0.0;2];}
        let posed = |position: f32| {
            if fullscreen {
                full
            } else if angular && kind == OverlayKind::BottomSheet {
                Rect {
                    y: full.y + full.height * (1.0 - position),
                    ..full
                }
            } else if angular {
                full
            } else {
                match kind {
                    OverlayKind::Menu => Rect {
                        height: full.height * (0.35 + 0.65 * position),
                        ..full
                    },
                    OverlayKind::Dialog => Rect {
                        y: full.y - 50.0 * (1.0 - position),
                        height: full.height * (0.35 + 0.65 * position),
                        ..full
                    },
                    OverlayKind::BottomSheet => Rect {
                        y: full.y + full.height * (1.0 - position) + drag_offset,
                        ..full
                    },
                    OverlayKind::SideSheet => Rect {
                        x: full.x + full.width * (1.0 - position) + side_drag_offset,
                        ..full
                    },
                    OverlayKind::LeftSideSheet => Rect {
                        x: full.x - full.width * (1.0 - position) - side_drag_offset,
                        ..full
                    },
                    OverlayKind::Popover => Rect {
                        y: full.y + 8.0 * (1.0 - position),
                        ..full
                    },
                }
            }
        };
        if state.open && !state.suspended
            && (state.dismiss_policy.escape() && self.pressed_key(Key::Escape)
                || angular && kind == OverlayKind::Menu && self.pressed_key(Key::Tab)
                || state.dismiss_policy.outside() && self.input.pressed
                    && self
                        .input
                        .pointer
                        .is_some_and(|p| !posed(values[0]).contains(p)))
        {
            state.open = false;
            self.ui.active = None;
            self.ui.continuous = true;
            values = if timed {
                state.sample(self.ui.time, self.reduced_motion())
            } else {
                let position = self.spatial(id, 21, 0.0, 0.0);
                let alpha = self.animate(id, 20, 0.0, 0.0);
                [position, alpha, alpha, alpha]
            };
        }
        if timed {
            self.ui.continuous |= state.moving();
        }
        let bounds = posed(values[0]);
        let alpha = values[1];
        if timed && !angular {
            values[3] = 1.0;
        }
        if state.open || state.was_open {
            self.ui.next_focus_order.clear();
        }
        if alpha > 0.001 || state.open {
            let t = self.ui.theme;
            if modal {
                self.rect(
                    self.viewport(),
                    0.0,
                    Color(0, 0, 0, 255).opacity(0.32 * values[2]),
                );
                for node in &mut self.ui.semantics {
                    node.enabled = false;
                }
            }
            let start = self.commands.len();
            let semantics_start = self.ui.semantics.len();
            let previous_phase = self.overlay_phase;
            self.overlay_phase = if angular {
                None
            } else {
                Some(OverlayPhase {
                    id,
                    open: state.open,
                    kind: state.kind,
                })
            };
            let origin = if let Some(origin) = state
                .transform_origin
                .filter(|p| p.iter().all(|v| v.is_finite()))
            {
                origin
            } else if kind == OverlayKind::Menu {
                [bounds.x, bounds.y]
            } else {
                [
                    bounds.x + bounds.width / 2.0,
                    bounds.y + bounds.height / 2.0,
                ]
            };
            let scale = if angular
                && !fullscreen
                && !timepicker
                && matches!(kind, OverlayKind::Menu | OverlayKind::Dialog)
            {
                0.8 + 0.2 * values[0]
            } else {
                1.0
            };
            if datepicker {
                self.commands.push(DrawCommand::PushScaleY {
                    origin: bounds.y,
                    scale: 0.8 + 0.2 * values[0],
                });
            }
            let previous_nested_scroll=self.overlay_nested_scroll;
            self.overlay_nested_scroll=state.bottom_sheet_scroll_offset;
            self.with_scale(origin, scale, |f| {
                let radius = if fullscreen {
                    0.0
                } else if angular {
                    if kind == OverlayKind::Menu || timepicker {
                        4.0
                    } else if kind == OverlayKind::Popover {
                        16.0
                    } else if kind == OverlayKind::BottomSheet {
                        0.0
                    } else {
                        28.0
                    }
                } else if matches!(kind, OverlayKind::BottomSheet | OverlayKind::Dialog) {
                    28.0
                } else if matches!(kind, OverlayKind::SideSheet | OverlayKind::LeftSideSheet) {
                    16.0
                } else {
                    20.0
                };
                if angular
                    && (kind == OverlayKind::Menu || timepicker || kind == OverlayKind::BottomSheet)
                {
                    f.elevation(bounds, radius, if state.style == OverlayStyle::AndroidMenu {3} else {2});
                }
                let surface_color = if state.style == OverlayStyle::AndroidMenu {
                    t.surface_high
                } else if angular {
                    if kind == OverlayKind::BottomSheet {
                        t.surface_low
                    } else if kind == OverlayKind::Menu || timepicker {
                        t.control
                    } else if kind == OverlayKind::Popover {
                        t.hovered
                    } else {
                        t.background
                    }
                } else if kind == OverlayKind::Dialog {
                    t.surface_high
                } else if state.style == OverlayStyle::NavigationRail {
                    t.background
                } else if state.style == OverlayStyle::NavigationDrawer
                    || matches!(
                        kind,
                        OverlayKind::BottomSheet | OverlayKind::SideSheet | OverlayKind::LeftSideSheet
                    )
                {
                    t.surface_low
                } else {
                    t.background.mix(t.secondary, 0.35)
                };
                if !angular && !fullscreen && kind == OverlayKind::BottomSheet {
                    // Extend the rounded shape past the bottom edge, then clip it so
                    // only the top two 28dp corners are rounded and the bottom sits flush.
                    f.within(bounds, |f| {
                        f.rect(
                            Rect {
                                height: (bounds.height + radius).max(radius * 2.0),
                                ..bounds
                            },
                            radius,
                            surface_color,
                        )
                    });
                } else if !angular
                    && !fullscreen
                    && state.style != OverlayStyle::NavigationRail
                    && kind == OverlayKind::SideSheet
                {
                    // Extend the rounded shape past the outer right edge, then clip it so
                    // only the inner (top-left & bottom-left) 16dp corners are rounded.
                    f.within(bounds, |f| {
                        f.rect(
                            Rect {
                                width: (bounds.width + 16.0).max(32.0),
                                ..bounds
                            },
                            16.0,
                            surface_color,
                        )
                    });
                } else if !angular
                    && !fullscreen
                    && state.style != OverlayStyle::NavigationRail
                    && kind == OverlayKind::LeftSideSheet
                {
                    // Extend the rounded shape past the outer left edge, then clip it so
                    // only the inner (top-right & bottom-right) 16dp corners are rounded.
                    f.within(bounds, |f| {
                        let extend = (bounds.width + 16.0).max(32.0) - bounds.width;
                        f.rect(
                            Rect {
                                x: bounds.x - extend,
                                width: bounds.width + extend,
                                ..bounds
                            },
                            16.0,
                            surface_color,
                        )
                    });
                } else {
                    f.rect(bounds, radius, surface_color);
                }
                let body = if fullscreen || (kind==OverlayKind::BottomSheet && state.drag_to_dismiss) || matches!(state.style, OverlayStyle::NavigationRail | OverlayStyle::NavigationDrawer) {
                    bounds
                } else if angular {
                    if kind == OverlayKind::Menu || timepicker {
                        Rect {
                            y: bounds.y + 8.0,
                            height: (bounds.height - 16.0).max(0.0),
                            ..bounds
                        }
                    } else {
                        bounds
                    }
                } else {
                    Rect {
                        height: full.height,
                        ..bounds
                    }
                    .inset(16.0)
                };
                if modal && kind == OverlayKind::Dialog && state.open {
                    if let Some(title) = state.accessible_title.as_deref() {
                        f.semantic(Response { id, bounds, enabled: true, ..Response::default() }, Role::Dialog, title, false, None, None, None);
                    }
                    if let Some(description) = state.accessible_description.as_deref() {
                        f.semantic(Response { id, bounds, enabled: true, ..Response::default() }, Role::DialogDescription, description, false, None, None, None);
                    }
                }
                if angular && kind == OverlayKind::Menu {
                    f.semantic(
                        Response {
                            id,
                            bounds,
                            enabled: state.open && f.enabled,
                            ..Response::default()
                        },
                        Role::Menu,
                        "Menu",
                        false,
                        None,
                        None,
                        None,
                    );
                }
                let entering_menu = angular && kind == OverlayKind::Menu && values[0] < 1.0;
                let input = f.input;
                if entering_menu {
                    f.input.pointer = None;
                    f.input.pressed = false;
                    f.input.released = false;
                    f.input.scroll = [0.0; 2];
                }
                f.without_input(state.suspended || nested_claimed || side_drag_claimed, |f| f.with_enabled(state.open && (alpha > 0.001 || entering_menu), |f| {
                    f.within(body, |f| build(f, &mut state.open))
                }));
                f.input = input;
            });
            self.overlay_nested_scroll=previous_nested_scroll;
            if datepicker {
                self.commands.push(DrawCommand::PopScaleY);
            }
            self.overlay_phase = previous_phase;
            for (index, command) in self.commands[start..].iter_mut().enumerate() {
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
                    | DrawCommand::Polyline { color, .. } => {
                        *color = color.opacity(if !angular && index == 0 {
                            alpha
                        } else {
                            values[3]
                        })
                    }
                    _ => {}
                }
            }
            if !state.open {
                self.ui.semantics.truncate(semantics_start);
                if state.was_open {
                    self.ui.next_focus_order.clear();
                }
            }
        }
        if state.open && !state.suspended && self.ui.focused.is_none() {
            self.ui.focused = state
                .initial_focus
                .filter(|id| self.ui.next_focus_order.contains(id))
                .or_else(|| self.ui.next_focus_order.first().copied());
            self.ui.keyboard_focus =
                state.opened_with_keyboard || angular && kind == OverlayKind::BottomSheet;
            self.ui.continuous |= angular
                && matches!(kind, OverlayKind::Menu | OverlayKind::BottomSheet)
                && self.ui.focused.is_some();
        }
        if !state.open && state.was_open {
            // The background was built disabled earlier in this frame. Restore only
            // once its owner is enabled again, never to a disabled/removed control.
            self.ui.pending_focus = state.restore;
            self.ui.pending_focus_keyboard =
                self.input.key.is_some() || (self.ui.keyboard_focus && !self.input.pressed);
            self.ui.keyboard_focus = self.ui.pending_focus_keyboard;
            self.ui.continuous = true;
        }
        self.ui.overlay_active |= state.open;
        state.was_open = state.open;
    }
    /// Direct manipulation handle: pointer movement has no animation lag.
    #[allow(clippy::too_many_arguments)]
    pub fn drag_handle(
        &mut self,
        id: u64,
        b: Rect,
        track: Rect,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        vertical: bool,
    ) -> Response {
        let min = *range.start();
        let max = *range.end();
        assert!(min.is_finite() && max.is_finite() && min < max);
        *value = if value.is_finite() {
            value.clamp(min, max)
        } else {
            min
        };
        let before = *value;
        let mut r = self.interact(id, b, true);
        if r.enabled
            && r.pressed
            && self.ui.capture == Capture::Pointer
            && let Some(p) = self.input.pointer
        {
            let (position, start, size) = if vertical {
                (p[1], track.y, track.height)
            } else {
                (p[0], track.x, track.width)
            };
            *value = min + ((position - start) / size.max(1.0)).clamp(0.0, 1.0) * (max - min);
        }
        if let Some(key) = self.key_for(r) {
            *value = match key {
                Key::Left | Key::Up => (*value - 1.0).max(min),
                Key::Right | Key::Down => (*value + 1.0).min(max),
                Key::Home => min,
                Key::End => max,
                _ => *value,
            };
        }
        r.changed = before != *value;
        self.semantic(
            r,
            Role::Slider,
            label,
            false,
            None,
            Some(*value),
            Some([min, max]),
        );
        self.focus_ring(r);
        r
    }
    /// Material 3 coplanar adaptive Standard Side Sheet sharing horizontal bounds
    /// with main content, separated by a 1dp vertical divider, with animated width.
    pub fn standard_side_sheet(
        &mut self,
        id: u64,
        open: &mut bool,
        right_edge: bool,
        width: f32,
        build_content: impl FnOnce(&mut Self),
        build_sheet: impl FnOnce(&mut Self),
    ) {
        let avail = self.available();
        let area = self.allocate(avail.height);
        if area.width <= 0.0 || area.height <= 0.0 {
            return;
        }
        let target_width = if width.is_finite() {
            width.clamp(0.0, area.width)
        } else {
            0.0
        };
        let progress = self
            .spatial(id, 21, f32::from(*open), f32::from(*open))
            .clamp(0.0, 1.0);
        let sheet_width = (target_width * progress).min(area.width);
        let divider_width = if sheet_width > 0.5 && area.width > sheet_width {
            1.0
        } else {
            0.0
        };
        let content_width = (area.width - sheet_width - divider_width).max(0.0);
        let (content_rect, divider_rect, sheet_rect, inner_sheet_rect) = if right_edge {
            let content = Rect {
                x: area.x,
                y: area.y,
                width: content_width,
                height: area.height,
            };
            let divider = Rect {
                x: area.x + content_width,
                y: area.y,
                width: divider_width,
                height: area.height,
            };
            let sheet = Rect {
                x: area.x + content_width + divider_width,
                y: area.y,
                width: sheet_width,
                height: area.height,
            };
            let inner = Rect {
                x: sheet.x,
                y: sheet.y,
                width: target_width.max(sheet_width),
                height: sheet.height,
            };
            (content, divider, sheet, inner)
        } else {
            let sheet = Rect {
                x: area.x,
                y: area.y,
                width: sheet_width,
                height: area.height,
            };
            let divider = Rect {
                x: area.x + sheet_width,
                y: area.y,
                width: divider_width,
                height: area.height,
            };
            let content = Rect {
                x: area.x + sheet_width + divider_width,
                y: area.y,
                width: content_width,
                height: area.height,
            };
            let inner = Rect {
                x: sheet.x + sheet.width - target_width.max(sheet_width),
                y: sheet.y,
                width: target_width.max(sheet_width),
                height: sheet.height,
            };
            (content, divider, sheet, inner)
        };

        if content_rect.width > 0.0 {
            self.within(content_rect, build_content);
        }
        if sheet_width > 0.5 {
            if divider_width > 0.0 {
                self.divider_at(divider_rect, DividerOrientation::Vertical, 0.0, 0.0);
            }
            let surface = self.ui.theme.background;
            self.rect(sheet_rect, 0.0, surface);
            let enabled = *open;
            self.within(sheet_rect, |f| {
                f.with_enabled(enabled, |f| {
                    f.within_unclipped(inner_sheet_rect, build_sheet);
                });
            });
        }
        if *open && !self.key_consumed && self.pressed_key(Key::Escape) {
            *open = false;
            self.key_consumed = true;
            self.ui.continuous = true;
        }
    }

    /// Standard Material 3 dialog content layout (24sp title, body paragraph, Cancel/Confirm text actions).
    /// Returns `(cancel_clicked, confirm_clicked)`.
    pub fn dialog_chrome(
        &mut self,
        id: u64,
        title: &str,
        body: &str,
        cancel_label: Option<&str>,
        confirm_label: &str,
    ) -> (bool, bool) {
        let t = self.theme();
        let b = self.available();
        self.styled_text(
            Rect { x: b.x + 24.0, y: b.y + 16.0, width: (b.width - 48.0).max(0.0), height: 36.0 },
            title,
            t.text,
            TextStyle { size: 24.0, ..TextStyle::default() },
        );
        self.paragraph(
            Rect {
                x: b.x + 24.0,
                y: b.y + 60.0,
                width: (b.width - 48.0).max(0.0),
                height: (b.height - 124.0).max(24.0),
            },
            body,
            t.on_surface_variant,
            TextStyle { size: 14.0, ..TextStyle::default() },
            20.0,
        );
        let btn_y = (b.y + b.height - 56.0).max(b.y + 84.0);
        let conf_w = (self.text_width(confirm_label, TextStyle::LABEL) + 24.0).max(64.0);
        let conf_rect = Rect {
            x: (b.x + b.width - 20.0 - conf_w).max(b.x + 16.0),
            y: btn_y,
            width: conf_w,
            height: 40.0,
        };
        let confirm = self
            .action_at(
                id.wrapping_add(1),
                conf_rect,
                confirm_label,
                Options { variant: ButtonVariant::Text, ..Options::default() },
            )
            .clicked;
        let cancel = if let Some(c_label) = cancel_label {
            let canc_w = (self.text_width(c_label, TextStyle::LABEL) + 24.0).max(64.0);
            let canc_rect = Rect {
                x: (conf_rect.x - 8.0 - canc_w).max(b.x + 16.0),
                y: btn_y,
                width: canc_w,
                height: 40.0,
            };
            self.action_at(
                id,
                canc_rect,
                c_label,
                Options { variant: ButtonVariant::Text, ..Options::default() },
            )
            .clicked
        } else {
            false
        };
        (cancel, confirm)
    }

    /// Material 3 Full-Screen Dialog header bar (Close X + 22sp title + Save action + divider).
    /// Returns `(close_clicked, action_clicked)`.
    pub fn fullscreen_dialog_header(&mut self, id: u64, title: &str, action_label: &str) -> (bool, bool) {
        let t = self.theme();
        let bar = self.reserve(56.0);
        let close_rect = Rect { x: bar.x + 4.0, y: bar.y + 4.0, width: 48.0, height: 48.0 };
        let close = self
            .material_icon_button_at(
                id,
                close_rect,
                MaterialIcon::Close,
                "Close",
                Options { variant: ButtonVariant::Text, ..Options::default() },
            )
            .clicked;
        let act_w = (self.text_width(action_label, TextStyle::LABEL) + 24.0).clamp(64.0, (bar.width * 0.4).max(64.0));
        let act_rect = Rect {
            x: (bar.x + bar.width - act_w - 8.0).max(bar.x + 56.0),
            y: bar.y + 8.0,
            width: act_w,
            height: 40.0,
        };
        let action = self
            .action_at(
                id.wrapping_add(1),
                act_rect,
                action_label,
                Options { variant: ButtonVariant::Text, ..Options::default() },
            )
            .clicked;
        self.styled_text(
            Rect {
                x: bar.x + 64.0,
                y: bar.y,
                width: (act_rect.x - (bar.x + 64.0) - 8.0).max(0.0),
                height: bar.height,
            },
            title,
            t.text,
            TextStyle { size: 22.0, ..TextStyle::default() },
        );
        self.paint_line([bar.x, bar.y + bar.height], [bar.x + bar.width, bar.y + bar.height], 1.0, t.outline_variant);
        (close, action)
    }

    /// Centered Material 3 bottom sheet drag handle pill (`32dp × 4dp`, 22dp vertical touch margin).
    pub fn bottom_sheet_drag_handle(&mut self) {
        let row = self.reserve(28.0);
        let pill = Rect {
            x: row.x + (row.width - 32.0) * 0.5,
            y: row.y + (row.height - 4.0) * 0.5,
            width: 32.0,
            height: 4.0,
        };
        self.paint_rect(pill, 2.0, self.theme().on_surface_variant.opacity(0.4));
    }

    /// Material 3 Side Sheet header row (22sp title + trailing Close icon button + bottom divider).
    /// Returns `true` when the close button is clicked.
    pub fn side_sheet_header(&mut self, id: u64, title: &str) -> bool {
        let t = self.theme();
        let bar = self.reserve(56.0);
        let close_rect = Rect {
            x: (bar.x + bar.width - 52.0).max(bar.x),
            y: bar.y + 4.0,
            width: 48.0,
            height: 48.0,
        };
        self.styled_text(
            Rect {
                x: bar.x + 24.0,
                y: bar.y,
                width: (close_rect.x - (bar.x + 24.0) - 8.0).max(0.0),
                height: bar.height,
            },
            title,
            t.text,
            TextStyle { size: 22.0, ..TextStyle::default() },
        );
        let closed = self
            .material_icon_button_at(
                id,
                close_rect,
                MaterialIcon::Close,
                "Close",
                Options { variant: ButtonVariant::Text, ..Options::default() },
            )
            .clicked;
        self.paint_line([bar.x, bar.y + bar.height], [bar.x + bar.width, bar.y + bar.height], 1.0, t.outline_variant);
        closed
    }
}

#[cfg(test)]
#[path = "overlay.spec.rs"]
mod tests;
