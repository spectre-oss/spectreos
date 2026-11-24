//! Draggable and resizable floating Material window frame (freeform/desktop window chrome).
use crate::*;

/// Caller-owned geometry and drag/resize state for `Frame::floating_window`.
#[derive(Clone, Copy, Debug)]
pub struct WindowFrameState {
    pub bounds: Rect,
    pub min_size: [f32; 2],
    dragging: bool,
    resizing: bool,
    grab_offset: [f32; 2],
}

impl Default for WindowFrameState {
    fn default() -> Self {
        Self {
            bounds: Rect { x: 40.0, y: 40.0, width: 320.0, height: 360.0 },
            min_size: [200.0, 160.0],
            dragging: false,
            resizing: false,
            grab_offset: [0.0, 0.0],
        }
    }
}

impl WindowFrameState {
    pub fn new(bounds: Rect) -> Self {
        Self { bounds, ..Self::default() }
    }
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }
    pub fn is_resizing(&self) -> bool {
        self.resizing
    }
}

/// Visual and behavioral options for `Frame::floating_window`.
#[derive(Clone, Copy, Debug)]
pub struct WindowFrameOptions {
    pub caption_height: f32,
    pub corner_radius: f32,
    pub caption_color: Option<Color>,
    pub caption_text_color: Option<Color>,
    pub surface_color: Option<Color>,
    pub clamp_to: Option<Rect>,
    pub show_maximize: bool,
    pub show_close: bool,
    pub resizable: bool,
}

impl Default for WindowFrameOptions {
    fn default() -> Self {
        Self {
            caption_height: 36.0,
            corner_radius: 10.0,
            caption_color: None,
            caption_text_color: None,
            surface_color: None,
            clamp_to: None,
            show_maximize: true,
            show_close: true,
            resizable: true,
        }
    }
}

/// Events produced by `Frame::floating_window` during the current frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowFrameAction {
    pub moved: bool,
    pub resized: bool,
    pub maximize_clicked: bool,
    pub close_clicked: bool,
}

impl Frame<'_> {
    /// Paint a floating elevated window with draggable caption bar, maximize/close actions,
    /// bottom-right resize grip, and clipped live `body` content.
    pub fn floating_window(
        &mut self,
        id: u64,
        title: &str,
        state: &mut WindowFrameState,
        options: WindowFrameOptions,
        body: impl FnOnce(&mut Self, Rect),
    ) -> WindowFrameAction {
        let t = self.theme();
        let cap_h = options.caption_height.max(28.0);
        let min_w = state.min_size[0].max(140.0);
        let min_h = state.min_size[1].max(cap_h + 64.0);
        state.bounds.width = state.bounds.width.max(min_w);
        state.bounds.height = state.bounds.height.max(min_h);

        let mut action = WindowFrameAction::default();
        let btn_count = usize::from(options.show_maximize) + usize::from(options.show_close);
        let btn_area = btn_count as f32 * 36.0 + 4.0;
        let drag_bar = Rect {
            x: state.bounds.x,
            y: state.bounds.y,
            width: (state.bounds.width - btn_area).max(40.0),
            height: cap_h,
        };
        let grip = Rect {
            x: state.bounds.x + state.bounds.width - 24.0,
            y: state.bounds.y + state.bounds.height - 24.0,
            width: 24.0,
            height: 24.0,
        };

        if self.input.released || self.input.cancelled || self.input.blurred {
            state.dragging = false;
            state.resizing = false;
        }
        if let Some(p) = self.input.pointer {
            if self.input.pressed {
                if options.resizable && grip.contains(p) {
                    state.resizing = true;
                    state.grab_offset = [
                        state.bounds.x + state.bounds.width - p[0],
                        state.bounds.y + state.bounds.height - p[1],
                    ];
                } else if drag_bar.contains(p) {
                    state.dragging = true;
                    state.grab_offset = [p[0] - state.bounds.x, p[1] - state.bounds.y];
                }
            }
            if state.dragging {
                let nx = p[0] - state.grab_offset[0];
                let ny = p[1] - state.grab_offset[1];
                if (nx - state.bounds.x).abs() > 0.01 || (ny - state.bounds.y).abs() > 0.01 {
                    state.bounds.x = nx;
                    state.bounds.y = ny;
                    action.moved = true;
                }
            } else if state.resizing {
                let nw = (p[0] + state.grab_offset[0] - state.bounds.x).max(min_w);
                let nh = (p[1] + state.grab_offset[1] - state.bounds.y).max(min_h);
                if (nw - state.bounds.width).abs() > 0.01 || (nh - state.bounds.height).abs() > 0.01 {
                    state.bounds.width = nw;
                    state.bounds.height = nh;
                    action.resized = true;
                }
            }
        }
        if let Some(limit) = options.clamp_to {
            state.bounds.width = state.bounds.width.min(limit.width.max(min_w));
            state.bounds.height = state.bounds.height.min(limit.height.max(min_h));
            state.bounds.x = state.bounds.x.clamp(limit.x, (limit.x + limit.width - state.bounds.width).max(limit.x));
            state.bounds.y = state.bounds.y.clamp(limit.y, (limit.y + limit.height - cap_h).max(limit.y));
        }

        let frame = state.bounds;
        let cap = Rect { height: cap_h, ..frame };
        let body_rect = Rect {
            x: frame.x,
            y: frame.y + cap_h,
            width: frame.width,
            height: (frame.height - cap_h).max(0.0),
        };
        let cap_bg = options.caption_color.unwrap_or(t.primary_container);
        let cap_fg = options.caption_text_color.unwrap_or(t.on_primary_container);
        let surf_bg = options.surface_color.unwrap_or(t.surface_low);

        self.shadow(frame, options.corner_radius + 2.0, [0.0, 10.0], 26.0, 0.0, Color(0, 0, 0, 95));
        self.paint_rect(frame, options.corner_radius, surf_bg);
        self.paint_asym_rect(cap, options.corner_radius, 0.0, cap_bg);

        let drag_bar = Rect { x: cap.x, y: cap.y, width: (cap.width - btn_area).max(40.0), height: cap_h };
        let _ = self.target(id, drag_bar, "Move window", true, Role::Button);
        self.styled_text(
            Rect { x: cap.x + 12.0, y: cap.y, width: (cap.width - btn_area - 16.0).max(0.0), height: cap_h },
            title,
            cap_fg,
            TextStyle { size: 13.5, weight: 500, ..TextStyle::default() },
        );

        let mut btn_x = cap.x + cap.width - 36.0;
        if options.show_close {
            let close_btn = Rect { x: btn_x, y: cap.y + (cap_h - 32.0) * 0.5, width: 32.0, height: 32.0 };
            let r_cls = self.target(id.wrapping_add(2), close_btn, "Close window", true, Role::Button);
            self.state_layer(r_cls, 16.0, cap_fg);
            self.paint_system_icon(close_btn.inset(7.0), SystemIcon::Close, cap_fg);
            action.close_clicked = r_cls.clicked;
            btn_x -= 36.0;
        }
        if options.show_maximize {
            let max_btn = Rect { x: btn_x, y: cap.y + (cap_h - 32.0) * 0.5, width: 32.0, height: 32.0 };
            let r_max = self.target(id.wrapping_add(1), max_btn, "Maximize window", true, Role::Button);
            self.state_layer(r_max, 16.0, cap_fg);
            self.paint_system_icon(max_btn.inset(7.0), SystemIcon::Maximize, cap_fg);
            action.maximize_clicked = r_max.clicked;
        }

        self.within(body_rect, |f| body(f, body_rect));

        if options.resizable {
            let grip = Rect {
                x: frame.x + frame.width - 24.0,
                y: frame.y + frame.height - 24.0,
                width: 24.0,
                height: 24.0,
            };
            let _ = self.target(id.wrapping_add(3), grip, "Resize window", true, Role::Button);
            self.paint_line([grip.x + 8.0, grip.y + 19.0], [grip.x + 19.0, grip.y + 8.0], 1.6, t.outline);
            self.paint_line([grip.x + 13.0, grip.y + 19.0], [grip.x + 19.0, grip.y + 13.0], 1.6, t.outline);
        }

        action
    }
}

#[cfg(test)]
#[path = "window_frame.spec.rs"]
mod tests;
