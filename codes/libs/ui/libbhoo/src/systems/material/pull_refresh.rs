//! Pull-to-refresh interaction. Threshold and drag resistance follow the
//! Apache-2.0 Compose Material 3 reference; see THIRD_PARTY_NOTICES.md.
use crate::{Frame, PointerKind, Rect};

/// Caller-owned state for one pull surface. Application data and refreshing state
/// remain owned by the caller. Coordinates and threshold are in logical pixels.
#[derive(Default, Debug)]
pub struct PullRefreshState {
    origin: Option<f32>,
    start_distance: f32,
    distance: f32,
    shown: f32,
    last_time: Option<f64>,
    dragging: bool,
}
impl PullRefreshState {
    /// Current visible indicator offset in logical pixels.
    pub fn offset(&self) -> f32 { self.shown }
    /// Whether the current pull has crossed the 80 dp refresh threshold.
    pub fn armed(&self) -> bool { self.distance * 0.5 > 80.0 }
    fn cancel(&mut self) { self.origin = None; self.dragging = false; self.distance = 0.0; }
}
impl Frame<'_> {
    /// A scroll viewport that transfers unconsumed touch movement at its top
    /// edge to pull-to-refresh. `content_height` is the total scrollable height.
    /// The closure paints viewport content using the supplied scroll offset.
    pub fn pull_refresh_scroll(&mut self, id:u64, bounds:Rect, offset:&mut f32, content_height:f32, state:&mut PullRefreshState, refreshing:bool, build:impl FnOnce(&mut Self, f32)) -> bool {
        let input=self.input;
        let inside=self.pointer_in(bounds);
        let reduced=self.reduced_motion();
        let max=(content_height-bounds.height).max(0.0);
        let wheel=self.scroll_y(bounds);
        let scroll=self.ui.scrolls.entry(id).or_default();
        if (scroll.offset-*offset).abs()>0.1 { scroll.cancel(); }
        scroll.offset=offset.clamp(0.0,max);
        if wheel!=0.0 { scroll.cancel(); scroll.offset=(scroll.offset+wheel).clamp(0.0,max); }
        if input.pointer_kind==PointerKind::Touch {
            if input.pressed && inside { scroll.begin(input.pointer.unwrap()[1],input.time);state.cancel(); }
            if let Some(pointer)=input.pointer {
                let mut distance=if refreshing { 0.0 } else { state.distance };
                scroll.drag_with_refresh(pointer[1],input.time,max,&mut distance);
                state.distance=if refreshing { 0.0 } else { distance };
                state.dragging=state.distance>0.0;
            }
        }
        let dragged=scroll.dragging;
        let triggered=input.released && !refreshing && state.dragging && state.armed();
        if input.released {
            if state.distance>0.0 { scroll.cancel(); } else { scroll.release(input.time,reduced); }
            state.cancel();
        }
        if input.cancelled || input.blurred || !self.enabled { scroll.cancel(); state.cancel(); }
        let active=scroll.advance(input.time,max,reduced);
        *offset=scroll.offset.clamp(0.0,max);
        if active { self.continuous(bounds); }
        if dragged { self.input.pressed=false;self.input.released=false;self.input.cancelled=true; }
        self.within(bounds,|f| build(f,*offset));
        self.paint_pull_refresh(bounds,state,refreshing);
        triggered && !input.cancelled && !input.blurred && self.enabled
    }
    /// Processes a touch gesture on a surface whose content is at its top edge.
    /// Returns true once when an armed pull is released. Callers set `refreshing`
    /// after starting work and clear it on completion. Nested scrolling must be
    /// coordinated by the host; pass `at_top` only when the child cannot scroll up.
    pub fn pull_to_refresh(&mut self, bounds: Rect, state: &mut PullRefreshState, at_top: bool, refreshing: bool) -> bool {
        let input = self.input;
        let mut triggered = false;
        if input.cancelled || input.blurred || !self.enabled || refreshing || !at_top {
            state.cancel();
        } else {
            if input.pointer_kind == PointerKind::Touch && input.pressed && self.pointer_in(bounds) {
                state.origin = input.pointer.map(|p| p[1]);
                state.start_distance = state.shown * 2.0;
            }
            if let (Some(origin), Some(pointer)) = (state.origin, input.pointer) {
                let delta = pointer[1] - origin;
                if delta > 6.0 { state.dragging = true; }
                if state.dragging { state.distance = (state.start_distance + delta).max(0.0); }
            }
            if input.released {
                triggered = state.dragging && state.armed();
                if state.dragging { self.input.released = false; self.input.cancelled = true; }
                state.cancel();
            } else if state.dragging {
                self.input.pressed = false;
                self.input.cancelled = true;
            }
        }
        self.paint_pull_refresh(bounds, state, refreshing);
        triggered
    }

    fn paint_pull_refresh(&mut self, bounds: Rect, state: &mut PullRefreshState, refreshing: bool) {
        let now = self.time();
        let dt = state.last_time.map_or(0.0, |t| (now-t).clamp(0.0,0.05) as f32);
        state.last_time = Some(now);
        let target = if refreshing { 80.0 } else if state.dragging {
            let adjusted = state.distance * 0.5;
            if adjusted <= 80.0 { adjusted } else {
                let tension = (adjusted / 80.0 - 1.0).clamp(0.0,2.0);
                80.0 + 80.0 * (tension - tension*tension/4.0)
            }
        } else { 0.0 };
        if state.dragging || self.reduced_motion() { state.shown = target; }
        else if (state.shown-target).abs() > 0.1 {
            state.shown += (target-state.shown) * (1.0-(-18.0*dt).exp());
            self.continuous(bounds);
        } else { state.shown = target; }
        if state.shown > 0.5 {
            let diameter = 40.0;
            let circle = Rect { x:bounds.x+(bounds.width-diameter)*0.5, y:bounds.y+state.shown-diameter, width:diameter,height:diameter };
            let theme = self.theme();
            let fraction=state.shown/80.0;
            let adjusted=(fraction.min(1.0)-0.4).max(0.0)*5.0/3.0;
            let overshoot=(fraction.abs()-1.0).clamp(0.0,2.0);
            let tension=overshoot-overshoot*overshoot/4.0;
            let rotation=(-0.25+0.4*adjusted+tension)*0.5*std::f32::consts::TAU;
            let sweep=adjusted*0.8*std::f32::consts::TAU;
            let color=theme.on_surface_variant.opacity(if fraction >= 1.0 || refreshing { 1.0 } else { 0.3 });
            let arc=Rect { x:circle.x+13.25,y:circle.y+13.25,width:13.5,height:13.5 };
            let (start, busy_sweep) = if refreshing && self.continuous(circle) { crate::progress::classic_circular_sample(now) } else { (rotation*2.0,4.0) };
            self.within(bounds, |f| {
                f.paint_rect(circle,20.0,theme.surface_high);
                if refreshing {
                    f.paint_arc(arc,start,busy_sweep,2.5,color);
                } else if adjusted > 0.0 {
                    f.paint_arc(arc,rotation*2.0,sweep,2.5,color);
                    let angle=rotation*2.0+sweep-2.5_f32.to_radians();
                    let scale=adjusted.min(1.0);
                    let cx=circle.x+20.0; let cy=circle.y+20.0;
                    let rotate=|x:f32,y:f32| [cx+x*angle.cos()-y*angle.sin(),cy+x*angle.sin()+y*angle.cos()];
                    let half=5.0*scale;
                    let left=rotate(6.75-half,-2.5);
                    let tip=rotate(6.75,-2.5+5.0*scale);
                    let right=rotate(6.75+half,-2.5);
                    f.paint_line(left,tip,2.5,color);
                    f.paint_line(tip,right,2.5,color);
                }
            });
        }
    }
}

#[cfg(test)]
#[path = "pull_refresh.spec.rs"]
mod tests;
