use crate::*;

impl Frame<'_> {
    /// Whole-star rating with caller-owned value, hover preview and keyboard navigation.
    pub fn rating(&mut self, id: u64, value: &mut usize, max: usize, enabled: bool) {
        let enabled = enabled && self.enabled;
        let max = max.clamp(1, 20);
        *value = (*value).min(max);
        let row = self.reserve(40.0);
        let width = (row.width / max as f32).min(36.0);
        let hovered = (0..max).find(|i| {
            self.pointer_in(Rect {
                x: row.x + *i as f32 * width,
                width,
                ..row
            })
        });
        let preview = if enabled {
            hovered.map_or(*value, |i| i + 1)
        } else {
            *value
        };
        let mut keyboard_handled = false;
        for i in 0..max {
            let bounds = Rect {
                x: row.x + i as f32 * width,
                width,
                ..row
            };
            let mut r = self.interact(id + i as u64, bounds, enabled);
            if r.clicked {
                *value = if *value == i + 1 { 0 } else { i + 1 };
                r.changed = true;
            }
            if r.focused && enabled && !keyboard_handled {
                let next = if self.pressed_key(Key::Right) || self.pressed_key(Key::Up) {
                    Some((i + 2).min(max))
                } else if self.pressed_key(Key::Left) || self.pressed_key(Key::Down) {
                    Some(i.max(1))
                } else if self.pressed_key(Key::Home) {
                    Some(1)
                } else if self.pressed_key(Key::End) {
                    Some(max)
                } else {
                    None
                };
                if let Some(next) = next {
                    *value = next;
                    self.request_focus(id + (next - 1) as u64);
                    keyboard_handled = true;
                }
            }
            self.focus_ring(r);
            self.styled_text(
                bounds,
                if i < preview { "★" } else { "☆" },
                if enabled {
                    self.ui.theme.primary
                } else {
                    self.ui.theme.text.opacity(0.38)
                },
                TextStyle {
                    size: 30.0,
                    align: 1,
                    ..TextStyle::default()
                },
            );
            self.semantic(
                r,
                Role::Radio,
                &format!("{} of {} stars", i + 1, max),
                *value == i + 1,
                Some(if *value == i + 1 {
                    CheckState::Checked
                } else {
                    CheckState::Unchecked
                }),
                None,
                None,
            );
        }
    }
    /// Fractional rating with direct drag and configurable step size.
    pub fn rating_fractional(&mut self, id: u64, value: &mut f32, stars: usize, step: f32, enabled: bool) -> Response {
        self.rating_fractional_with_indicator(id, value, stars, step, enabled, false)
    }
    pub fn rating_fractional_with_indicator(&mut self, id: u64, value: &mut f32, stars: usize, step: f32, enabled: bool, indicator: bool) -> Response {
        self.rating_fractional_directional(id, value, stars, step, enabled, indicator, false)
    }
    /// Fractional rating, optionally mirrored for right-to-left layouts.
    #[allow(clippy::too_many_arguments)]
    pub fn rating_fractional_directional(&mut self, id: u64, value: &mut f32, stars: usize, step: f32, enabled: bool, indicator: bool, rtl: bool) -> Response {
        self.rating_fractional_tinted(id,value,stars,step,enabled,indicator,rtl,[None;3])
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn rating_fractional_tinted(&mut self, id:u64,value:&mut f32,stars:usize,step:f32,enabled:bool,indicator:bool,rtl:bool,colors:[Option<Color>;3])->Response {
        let stars = stars.clamp(1, 20);
        let step = if step.is_finite() && step > 0.0 { step.min(stars as f32) } else { 0.5 };
        *value = if value.is_finite() { value.clamp(0.0, stars as f32) } else { 0.0 };
        let old = *value;
        let mut row = self.reserve(40.0);
        let width = (row.width / stars as f32).min(48.0);
        if rtl { row.x += row.width - width * stars as f32; }
        row.width = width * stars as f32;
        let mut response = self.interact(id, row, enabled && !indicator);
        if response.enabled && self.ui.active == Some(id) && self.ui.capture == Capture::Pointer
            && let Some(point) = self.input.pointer {
            let distance = if rtl { row.x + row.width - point[0] } else { point[0] - row.x };
            let raw = (distance / width.max(1.0)).clamp(0.0, stars as f32);
            *value = ((raw / step).ceil() * step).clamp(0.0, stars as f32);
        }
        if let Some(key) = self.key_for(response) {
            *value = match key {
                Key::Left if rtl => (*value + step).min(stars as f32),
                Key::Right if rtl => (*value - step).max(0.0),
                Key::Left | Key::Down => (*value - step).max(0.0),
                Key::Right | Key::Up => (*value + step).min(stars as f32),
                Key::Home => 0.0,
                Key::End => stars as f32,
                _ => *value,
            };
        }
        response.changed = (*value - old).abs() > 0.0001;
        let foreground = colors[0].unwrap_or(self.ui.theme.primary.opacity(if enabled && self.enabled { 1.0 } else { 0.38 }));
        let background = colors[1].unwrap_or(self.ui.theme.on_surface_variant.opacity(if enabled && self.enabled { 0.38 } else { 0.12 }));
        let index = (crate::beercss_shapes::SHAPES.len() + crate::icons::material_navigation::ICONS.len() + crate::icons::material_symbols::ICONS.len()) as u8;
        for i in 0..stars {
            let physical_index = if rtl { stars - 1 - i } else { i };
            let star = Rect { x: row.x + physical_index as f32 * width, width, ..row };
            let icon_size = star.width.min(star.height);
            let icon = Rect { x: star.x + (star.width - icon_size) / 2.0, y: star.y + (star.height - icon_size) / 2.0, width: icon_size, height: icon_size };
            self.commands.push(DrawCommand::Shape { bounds: icon, index, color: background, rotation: 0.0 });
            let fill = (*value - i as f32).clamp(0.0, 1.0);
            if fill > 0.0 && let Some(secondary)=colors[2] {
                self.commands.push(DrawCommand::Shape { bounds: icon, index, color: secondary, rotation: 0.0 });
            }
            if fill > 0.0 {
                let fill_width = star.width * fill;
                let fill_x = if rtl { star.x + star.width - fill_width } else { star.x };
                self.commands.push(DrawCommand::PushClip(Rect { x: fill_x, width: fill_width, ..star }.intersect(self.clip)));
                self.commands.push(DrawCommand::Shape { bounds: icon, index, color: foreground, rotation: 0.0 });
                self.commands.push(DrawCommand::PopClip);
            }
        }
        self.focus_ring(response);
        response
    }
}

#[cfg(test)]
#[path = "rating.spec.rs"]
mod tests;
