//! Adaptive Material Scaffold layout coordinating top bar, navigation rail/bar, body, and FAB.
use crate::*;

/// Configuration for `Frame::scaffold`.
#[derive(Clone, Copy, Debug)]
pub struct ScaffoldOptions {
    pub top_bar_height: f32,
    pub bottom_bar_height: f32,
    pub rail_width: f32,
    /// Viewport width threshold at or above which the navigation rail slot is shown instead of bottom bar.
    pub compact_breakpoint: f32,
}

impl Default for ScaffoldOptions {
    fn default() -> Self {
        Self {
            top_bar_height: 64.0,
            bottom_bar_height: 80.0,
            rail_width: 80.0,
            compact_breakpoint: 600.0,
        }
    }
}

/// Resolved slot rectangles computed by `Frame::scaffold_slots`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaffoldSlots {
    pub compact: bool,
    pub top_bar: Rect,
    pub nav_rail: Option<Rect>,
    pub bottom_bar: Option<Rect>,
    pub body: Rect,
    pub fab: Rect,
}

impl Frame<'_> {
    /// Compute adaptive Material 3 scaffold slot rectangles for `bounds`.
    pub fn scaffold_slots(&self, bounds: Rect, options: ScaffoldOptions) -> ScaffoldSlots {
        let compact = bounds.width < options.compact_breakpoint;
        let top_h = options.top_bar_height.max(0.0).min(bounds.height);
        let rail_w = if compact { 0.0 } else { options.rail_width.max(0.0).min(bounds.width) };
        let bottom_h = if compact {
            options.bottom_bar_height.max(0.0).min((bounds.height - top_h).max(0.0))
        } else {
            0.0
        };
        let top_bar = Rect { x: bounds.x + rail_w, y: bounds.y, width: (bounds.width - rail_w).max(0.0), height: top_h };
        let nav_rail = (!compact && rail_w > 0.0).then_some(Rect { x: bounds.x, y: bounds.y, width: rail_w, height: bounds.height });
        let bottom_bar = (compact && bottom_h > 0.0).then_some(Rect {
            x: bounds.x,
            y: bounds.y + bounds.height - bottom_h,
            width: bounds.width,
            height: bottom_h,
        });
        let body = Rect {
            x: bounds.x + rail_w,
            y: bounds.y + top_h,
            width: (bounds.width - rail_w).max(0.0),
            height: (bounds.height - top_h - bottom_h).max(0.0),
        };
        let fab = Rect {
            x: (body.x + body.width - 72.0).max(body.x),
            y: (body.y + body.height - 72.0).max(body.y),
            width: 56.0_f32.min(body.width),
            height: 56.0_f32.min(body.height),
        };
        ScaffoldSlots { compact, top_bar, nav_rail, bottom_bar, body, fab }
    }

    /// Adaptive Material 3 Scaffold rendering top bar, responsive navigation rail or bottom bar,
    /// main content body, and optional floating action button slot.
    pub fn scaffold(
        &mut self,
        bounds: Rect,
        options: ScaffoldOptions,
        top_bar: impl FnOnce(&mut Self, Rect),
        mut navigation: impl FnMut(&mut Self, Rect, bool),
        body: impl FnOnce(&mut Self, Rect),
        fab: impl FnOnce(&mut Self, Rect),
    ) -> ScaffoldSlots {
        let slots = self.scaffold_slots(bounds, options);
        if slots.top_bar.height > 0.0 {
            self.within(slots.top_bar, |f| top_bar(f, slots.top_bar));
        }
        if let Some(rail) = slots.nav_rail {
            self.within(rail, |f| navigation(f, rail, false));
        }
        self.within(slots.body, |f| body(f, slots.body));
        if let Some(bar) = slots.bottom_bar {
            self.within(bar, |f| navigation(f, bar, true));
        }
        fab(self, slots.fab);
        slots
    }

    /// Adaptive Material 3 Scaffold with animated rail expansion (`collapsed_rail_width` <-> `expanded_rail_width`)
    /// on wide viewports and a modal slide-over drawer slot (`drawer_open`) on compact viewports.
    #[allow(clippy::too_many_arguments)]
    pub fn scaffold_adaptive(
        &mut self,
        id: u64,
        bounds: Rect,
        options: ScaffoldAdaptiveOptions,
        rail_expanded: bool,
        drawer_open: &mut bool,
        top_bar: impl FnOnce(&mut Self, Rect),
        mut navigation: impl FnMut(&mut Self, Rect, f32, bool),
        body: impl FnOnce(&mut Self, Rect),
        fab: impl FnOnce(&mut Self, Rect),
    ) -> ScaffoldSlots {
        let compact = bounds.width < options.compact_breakpoint;
        let collapsed_w = options.collapsed_rail_width.max(56.0);
        let expanded_w = options.expanded_rail_width.max(collapsed_w);
        let target_exp = f32::from(!compact && rail_expanded);
        let expansion = self
            .transition(
                id,
                70,
                target_exp,
                target_exp,
                0.3,
                crate::timing::Easing::Standard,
            )
            .clamp(0.0, 1.0);
        let rail_w = if compact {
            0.0
        } else {
            collapsed_w + (expanded_w - collapsed_w) * expansion
        };
        let drawer_prog = self
            .transition(
                id,
                71,
                f32::from(compact && *drawer_open),
                0.0,
                0.3,
                crate::timing::Easing::Standard,
            )
            .clamp(0.0, 1.0);
        let slots = self.scaffold_slots(
            bounds,
            ScaffoldOptions {
                top_bar_height: options.top_bar_height,
                bottom_bar_height: options.bottom_bar_height,
                rail_width: rail_w,
                compact_breakpoint: options.compact_breakpoint,
            },
        );
        let blocked = compact && *drawer_open;
        self.without_input(blocked, |f| {
            if slots.top_bar.height > 0.0 {
                f.within(slots.top_bar, |f| top_bar(f, slots.top_bar));
            }
            if let Some(rail) = slots.nav_rail {
                f.within(rail, |f| navigation(f, rail, expansion, false));
            }
            f.within(slots.body, |f| body(f, slots.body));
            if let Some(bar) = slots.bottom_bar {
                f.within(bar, |f| navigation(f, bar, 0.0, true));
            }
            fab(f, slots.fab);
        });
        if compact && drawer_prog > 0.005 {
            let scrim_r = self.target(id.wrapping_add(2), bounds, "Close navigation drawer", *drawer_open, Role::Button);
            self.paint_rect(bounds, 0.0, Color(0, 0, 0, (0.38 * drawer_prog * 255.0) as u8));
            let dw = options.drawer_width.min(bounds.width);
            let drawer_rect = Rect {
                x: bounds.x - dw * (1.0 - drawer_prog),
                y: bounds.y,
                width: dw,
                height: bounds.height,
            };
            self.elevation(drawer_rect, 16.0, 6);
            self.paint_rect(drawer_rect, 16.0, self.theme().surface_low);
            self.within(drawer_rect, |f| navigation(f, drawer_rect, 1.0, false));
            if (scrim_r.clicked && !self.pointer_in(drawer_rect)) || self.pressed_key(Key::Escape) {
                *drawer_open = false;
            }
        }
        slots
    }
}

/// Configuration for `Frame::scaffold_adaptive`.
#[derive(Clone, Copy, Debug)]
pub struct ScaffoldAdaptiveOptions {
    pub top_bar_height: f32,
    pub bottom_bar_height: f32,
    pub collapsed_rail_width: f32,
    pub expanded_rail_width: f32,
    pub drawer_width: f32,
    pub compact_breakpoint: f32,
}

impl Default for ScaffoldAdaptiveOptions {
    fn default() -> Self {
        Self {
            top_bar_height: 64.0,
            bottom_bar_height: 80.0,
            collapsed_rail_width: 88.0,
            expanded_rail_width: 256.0,
            drawer_width: 280.0,
            compact_breakpoint: 600.0,
        }
    }
}

#[cfg(test)]
#[path = "scaffold.spec.rs"]
mod tests;
