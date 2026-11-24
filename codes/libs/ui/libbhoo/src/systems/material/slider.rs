use crate::*;

#[derive(Clone, Copy, Default)]
pub(crate) struct SliderTintColors {
    pub primary: Option<Color>, pub background: Option<Color>,
    pub secondary: Option<Color>, pub thumb: Option<Color>,
    pub tick: Option<(f32,f32,f32,Color)>,
    pub split_track: bool,
    pub custom_thumb: Option<Option<(f32,f32,f32,Color)>>,
}

/// Configuration for Material 3 Expressive sliders and range sliders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderExpressiveOptions {
    pub step_size: f32,
    pub min_separation: f32,
    pub show_value_bubble: bool,
    pub enabled: bool,
}
impl Default for SliderExpressiveOptions {
    fn default() -> Self {
        Self {
            step_size: 0.0,
            min_separation: 0.0,
            show_value_bubble: true,
            enabled: true,
        }
    }
}

impl Frame<'_> {
    pub fn slider(
        &mut self,
        id: u64,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        step: f32,
        enabled: bool,
    ) -> Response {
        self.slider_directional(id, label, value, range, step, enabled, false)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn slider_directional(
        &mut self, id: u64, label: &str, value: &mut f32,
        range: std::ops::RangeInclusive<f32>, step: f32, enabled: bool, rtl: bool,
    ) -> Response {
        self.slider_directional_with_secondary(id, label, value, range, step, enabled, rtl, None)
    }
    /// Directional slider with an optional buffered fraction behind the primary track.
    #[allow(clippy::too_many_arguments)]
    pub fn slider_directional_with_secondary(
        &mut self, id: u64, label: &str, value: &mut f32,
        range: std::ops::RangeInclusive<f32>, step: f32, enabled: bool, rtl: bool,
        secondary: Option<f32>,
    ) -> Response {
        self.slider_directional_tinted(id,label,value,range,step,enabled,rtl,secondary,SliderTintColors::default())
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn slider_directional_tinted(
        &mut self, id:u64, label:&str, value:&mut f32,
        range:std::ops::RangeInclusive<f32>, step:f32, enabled:bool, rtl:bool,
        secondary:Option<f32>, colors:SliderTintColors,
    )->Response {
        let b = self.allocate(48.0);
        let mut r = self.interact(id, b, enabled);
        let min = *range.start();
        let max = *range.end();
        assert!(
            min.is_finite() && max.is_finite() && max > min,
            "Slider range must be finite and increasing"
        );
        let old = *value;
        *value = if value.is_finite() {
            value.clamp(min, max)
        } else {
            min
        };
        let start = b.x + 11.0;
        let width = (b.width - 22.0).max(1.0);
        if r.enabled && !self.input.cancelled && !self.input.blurred {
            if self.ui.active == Some(id)
                && self.ui.capture == Capture::Pointer
                && let Some(p) = self.input.pointer
            {
                let fraction = ((p[0] - start) / width).clamp(0.0, 1.0);
                *value = min + (if rtl { 1.0 - fraction } else { fraction }) * (max - min);
            }
            let increment = if step.is_finite() && step > 0.0 {
                step
            } else {
                (max - min) / 100.0
            };
            if let Some(key) = self.key_for(r) {
                match key {
                    Key::Left if rtl => *value += increment,
                    Key::Right if rtl => *value -= increment,
                    Key::Left | Key::Down => *value -= increment,
                    Key::Right | Key::Up => *value += increment,
                    Key::Home => *value = min,
                    Key::End => *value = max,
                    _ => {}
                }
            }
            if step.is_finite() && step > 0.0 && *value != old {
                *value = min + ((*value - min) / step).round() * step;
            }
            *value = value.clamp(min, max);
        }
        r.changed = *value != old;
        let amount = (*value - min) / (max - min);
        let t = self.ui.theme;
        let track = Rect {
            x: b.x + 8.0,
            y: b.y + 22.0,
            width: (b.width - 16.0).max(0.0),
            height: 4.0,
        };
        let x = start + width * if rtl { 1.0 - amount } else { amount };
        let thumb=match colors.custom_thumb {
            Some(Some((w,h,r,color)))=>Some((w,h,r,colors.thumb.unwrap_or(color))),
            Some(None)=>None,
            None=>Some((20.0,20.0,10.0,colors.thumb.unwrap_or(if r.enabled{t.primary}else{t.text.opacity(0.38)}))),
        };
        let draw_track=|this:&mut Self| {
        this.rect(
            track,
            2.0,
            colors.background.unwrap_or(if r.enabled {
                t.surface_variant
            } else {
                t.text.opacity(0.12)
            }),
        );
        if let Some(secondary) = secondary.filter(|v| v.is_finite()) {
            let buffered = secondary.clamp(0.0, 1.0);
            let buffered_width = track.width * buffered;
            this.rect(Rect { x: if rtl { track.x + track.width - buffered_width } else { track.x }, width: buffered_width, ..track }, 2.0,
                colors.secondary.unwrap_or(if r.enabled { t.primary.opacity(0.38) } else { t.text.opacity(0.12) }));
        }
        this.rect(
            Rect {
                x: if rtl { x } else { track.x },
                width: if rtl { (track.x + track.width - x).max(0.0) } else { (x - track.x).max(0.0) },
                ..track
            },
            2.0,
            colors.primary.unwrap_or(if r.enabled {
                t.primary
            } else {
                t.text.opacity(0.38)
            }),
        );
        if let Some((tick_width,tick_height,radius,color))=colors.tick {
            let count=(max-min).round().clamp(0.0,1024.0) as usize;
            if count>1 {
                for i in 0..=count {
                    let center=start+width*i as f32/count as f32;
                    this.rect(Rect{x:center-tick_width/2.0,y:b.y+24.0-tick_height/2.0,width:tick_width,height:tick_height},radius,color);
                }
            }
        }
        };
        if colors.split_track && let Some((thumb_width,thumb_height,_,_))=thumb {
            let clip=self.clip;
            let left=x-thumb_width/2.0;let right=x+thumb_width/2.0;
            let top=b.y+24.0-thumb_height/2.0;let bottom=b.y+24.0+thumb_height/2.0;
            let regions=[
                Rect{x:clip.x,y:clip.y,width:(left-clip.x).max(0.0),height:clip.height},
                Rect{x:right,y:clip.y,width:(clip.x+clip.width-right).max(0.0),height:clip.height},
                Rect{x:left,y:clip.y,width:thumb_width,height:(top-clip.y).max(0.0)},
                Rect{x:left,y:bottom,width:thumb_width,height:(clip.y+clip.height-bottom).max(0.0)},
            ];
            for (index,region) in regions.into_iter().enumerate() {
                if index>=2 && thumb_height>=4.0 && colors.tick.is_none_or(|(_,height,_,_)|height<=thumb_height) {continue;}
                let region=region.intersect(clip);
                if region.width>0.0 && region.height>0.0 {
                    self.commands.push(DrawCommand::PushClip(region));draw_track(self);self.commands.push(DrawCommand::PopClip);
                }
            }
        } else {draw_track(self);}
        self.state_layer(
            Response {
                bounds: Rect {
                    x: x - 20.0,
                    y: b.y + 4.0,
                    width: 40.0,
                    height: 40.0,
                },
                ..r
            },
            20.0,
            t.primary,
        );
        if let Some((w,h,radius,color))=thumb {self.rect(Rect{x:x-w/2.0,y:b.y+24.0-h/2.0,width:w,height:h},radius,color);}
        self.focus_ring(r);
        self.semantic(
            r,
            Role::Slider,
            label,
            false,
            None,
            Some(*value),
            Some([min, max]),
        );
        r
    }
    /// Two independently focusable endpoints; drag follows input without animation.
    pub fn range_slider(
        &mut self,
        ids: [u64; 2],
        label: &str,
        values: &mut [f32; 2],
        range: std::ops::RangeInclusive<f32>,
        step: f32,
        enabled: bool,
    ) -> [Response; 2] {
        let min = *range.start();
        let max = *range.end();
        assert!(min.is_finite() && max.is_finite() && min < max);
        for value in values.iter_mut() {
            *value = if value.is_finite() {
                value.clamp(min, max)
            } else {
                min
            };
        }
        if values[0] > values[1] {
            values.swap(0, 1);
        }
        let previous = *values;
        let b = self.allocate(48.0);
        let width = (b.width - 22.0).max(1.0);
        let start = b.x + 11.0;
        let mut responses = [Response::default(); 2];
        for i in 0..2 {
            let x = start + (values[i] - min) / (max - min) * width;
            responses[i] = self.interact(
                ids[i],
                Rect {
                    x: x - 12.0,
                    width: 24.0,
                    ..b
                },
                enabled,
            );
        }
        if enabled
            && self.enabled
            && self.input.pressed
            && !self.input.cancelled
            && self.ui.active.is_none()
            && let Some(p) = self.input.pointer
            && b.intersect(self.clip).contains(p)
        {
            let value = min + ((p[0] - start) / width).clamp(0.0, 1.0) * (max - min);
            let i = usize::from((value - values[1]).abs() < (value - values[0]).abs());
            self.ui.active = Some(ids[i]);
            self.ui.capture = Capture::Pointer;
            self.ui.focused = Some(ids[i]);
            self.ui.keyboard_focus = false;
        }
        for i in 0..2 {
            let r = &mut responses[i];
            r.focused = self.ui.focused == Some(ids[i]);
            if r.enabled && !self.input.cancelled && !self.input.blurred {
                if self.ui.active == Some(ids[i])
                    && self.ui.capture == Capture::Pointer
                    && let Some(p) = self.input.pointer
                {
                    values[i] = min + ((p[0] - start) / width).clamp(0.0, 1.0) * (max - min);
                }
                let increment = if step.is_finite() && step > 0.0 {
                    step
                } else {
                    (max - min) / 100.0
                };
                if let Some(key) = self.key_for(*r) {
                    match key {
                        Key::Left | Key::Down => values[i] -= increment,
                        Key::Right | Key::Up => values[i] += increment,
                        Key::Home => values[i] = min,
                        Key::End => values[i] = max,
                        _ => {}
                    }
                }
                if step.is_finite() && step > 0.0 && values[i] != previous[i] {
                    values[i] = min + ((values[i] - min) / step).round() * step;
                }
                values[i] = if i == 0 {
                    values[i].clamp(min, values[1])
                } else {
                    values[i].clamp(values[0], max)
                };
            }
            r.changed = values[i] != previous[i];
        }
        let t = self.ui.theme;
        let x0 = start + (values[0] - min) / (max - min) * width;
        let x1 = start + (values[1] - min) / (max - min) * width;
        self.rect(
            Rect {
                x: b.x + 8.0,
                y: b.y + 22.0,
                width: (b.width - 16.0).max(0.0),
                height: 4.0,
            },
            2.0,
            t.surface_variant,
        );
        self.rect(
            Rect {
                x: x0,
                y: b.y + 22.0,
                width: x1 - x0,
                height: 4.0,
            },
            2.0,
            t.primary
                .opacity(if enabled && self.enabled { 1.0 } else { 0.38 }),
        );
        for i in 0..2 {
            let x = if i == 0 { x0 } else { x1 };
            self.state_layer(
                Response {
                    bounds: Rect {
                        x: x - 20.0,
                        y: b.y + 4.0,
                        width: 40.0,
                        height: 40.0,
                    },
                    ..responses[i]
                },
                20.0,
                t.primary,
            );
            self.rect(
                Rect {
                    x: x - 10.0,
                    y: b.y + 14.0,
                    width: 20.0,
                    height: 20.0,
                },
                10.0,
                t.primary
                    .opacity(if responses[i].enabled { 1.0 } else { 0.38 }),
            );
            responses[i].bounds = Rect {
                x: x - 12.0,
                width: 24.0,
                ..b
            };
            self.focus_ring(responses[i]);
            self.semantic(
                responses[i],
                Role::Slider,
                label,
                false,
                None,
                Some(values[i]),
                Some([min, max]),
            );
        }
        responses
    }

    fn paint_slider_value_bubble(&mut self, hx: f32, anchor_y: f32, value: f32) {
        let viewport = self.viewport();
        let text = if (value - value.round()).abs() < 0.005 {
            format!("{}", value.round() as i64)
        } else {
            format!("{value:.1}")
        };
        let style = TextStyle {
            size: 12.0,
            weight: 500,
            align: 1,
            padding: 8.0,
            letter_spacing: 0.4,
            ..TextStyle::default()
        };
        let pill_w = (self.text_width(&text, style) + 20.0).max(40.0);
        let pill_h = 32.0;
        let pill_x = (hx - pill_w * 0.5).clamp(
            viewport.x + 4.0,
            (viewport.x + viewport.width - pill_w - 4.0).max(viewport.x + 4.0),
        );
        let pill_y = anchor_y - pill_h - 6.0;
        let pill = Rect {
            x: pill_x,
            y: pill_y,
            width: pill_w,
            height: pill_h,
        };
        let t = self.ui.theme;
        let tip_x = hx.clamp(pill.x + 8.0, pill.x + pill.width - 8.0);
        let tip_base_y = pill.y + pill.height - 2.0;
        let tip_y = pill.y + pill.height + 4.0;
        self.ui.overlays.push(DrawCommand::RoundedRect {
            bounds: pill,
            radius: 16.0,
            color: t.inverse_surface,
        });
        self.ui.overlays.push(DrawCommand::Line {
            from: [tip_x - 4.0, tip_base_y],
            to: [tip_x, tip_y],
            width: 4.0,
            color: t.inverse_surface,
        });
        self.ui.overlays.push(DrawCommand::Line {
            from: [tip_x + 4.0, tip_base_y],
            to: [tip_x, tip_y],
            width: 4.0,
            color: t.inverse_surface,
        });
        let cached = self.ui.text(&text);
        self.ui.overlays.push(DrawCommand::Text {
            bounds: pill,
            text: cached,
            color: t.inverse_on_surface,
            style,
        });
    }

    /// Material 3 Expressive slider: 16dp rounded tracks, 4dp×44dp vertical pill handle
    /// with 6dp track gaps, 4dp stop indicator / discrete ticks, and teardrop value bubble.
    pub fn expressive_slider(
        &mut self,
        id: u64,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        options: SliderExpressiveOptions,
    ) -> Response {
        let min = *range.start();
        let max = *range.end();
        assert!(
            min.is_finite() && max.is_finite() && max > min,
            "Slider range must be finite and increasing"
        );
        let step = if options.step_size.is_finite() && options.step_size > 0.0 {
            options.step_size
        } else {
            0.0
        };
        let b = self.allocate(48.0);
        let mut r = self.interact(id, b, options.enabled);
        let old = *value;
        *value = if value.is_finite() {
            value.clamp(min, max)
        } else {
            min
        };
        let track_x = b.x + 8.0;
        let track_w = (b.width - 16.0).max(1.0);
        let track_right = track_x + track_w;
        let start = track_x + 2.0;
        let travel = (track_w - 4.0).max(1.0);
        if r.enabled && !self.input.cancelled && !self.input.blurred {
            if self.ui.active == Some(id)
                && self.ui.capture == Capture::Pointer
                && let Some(p) = self.input.pointer
            {
                let fraction = ((p[0] - start) / travel).clamp(0.0, 1.0);
                *value = min + fraction * (max - min);
            }
            let increment = if step > 0.0 {
                step
            } else {
                (max - min) / 100.0
            };
            if let Some(key) = self.key_for(r) {
                match key {
                    Key::Left | Key::Down => *value -= increment,
                    Key::Right | Key::Up => *value += increment,
                    Key::Home => *value = min,
                    Key::End => *value = max,
                    _ => {}
                }
            }
            if step > 0.0 && *value != old {
                *value = min + ((*value - min) / step).round() * step;
            }
            *value = value.clamp(min, max);
        }
        r.changed = *value != old;
        let amount = (*value - min) / (max - min);
        let hx = start + travel * amount;
        let cy = b.y + 24.0;
        let track_y = cy - 8.0;
        let gap_half = 8.0; // 2dp half-handle + 6dp handle-to-track gap
        let t = self.ui.theme;
        let active_color = if r.enabled {
            t.primary
        } else {
            t.text.opacity(0.38)
        };
        let inactive_color = if r.enabled {
            t.secondary_container
        } else {
            t.text.opacity(0.12)
        };
        let active_stop_color = if r.enabled {
            t.on_primary
        } else {
            t.inverse_on_surface.opacity(0.38)
        };
        let inactive_stop_color = if r.enabled {
            t.on_secondary_container
        } else {
            t.text.opacity(0.38)
        };
        let active_end = (hx - gap_half).clamp(track_x, track_right);
        if active_end > track_x {
            let w = active_end - track_x;
            self.rect(
                Rect {
                    x: track_x,
                    y: track_y,
                    width: w,
                    height: 16.0,
                },
                8.0_f32.min(w * 0.5),
                active_color,
            );
        }
        let inactive_start = (hx + gap_half).clamp(track_x, track_right);
        if track_right > inactive_start {
            let w = track_right - inactive_start;
            self.rect(
                Rect {
                    x: inactive_start,
                    y: track_y,
                    width: w,
                    height: 16.0,
                },
                8.0_f32.min(w * 0.5),
                inactive_color,
            );
        }
        if step > 0.0 {
            let steps = ((max - min) / step).round().clamp(1.0, 128.0) as usize;
            for i in 0..=steps {
                let tx = start + travel * (i as f32 / steps as f32);
                if (tx - hx).abs() < gap_half {
                    continue;
                }
                self.rect(
                    Rect {
                        x: tx - 2.0,
                        y: cy - 2.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    2.0,
                    if tx < hx {
                        active_stop_color
                    } else {
                        inactive_stop_color
                    },
                );
            }
        } else if inactive_start <= track_right - 8.0 {
            self.rect(
                Rect {
                    x: track_right - 8.0,
                    y: cy - 2.0,
                    width: 4.0,
                    height: 4.0,
                },
                2.0,
                inactive_stop_color,
            );
        }
        self.rect(
            Rect {
                x: hx - 2.0,
                y: cy - 22.0,
                width: 4.0,
                height: 44.0,
            },
            2.0,
            active_color,
        );
        if options.show_value_bubble && r.enabled && (r.pressed || r.focused) {
            self.paint_slider_value_bubble(hx, cy - 22.0, *value);
        }
        self.focus_ring(r);
        self.semantic(
            r,
            Role::Slider,
            label,
            false,
            None,
            Some(*value),
            Some([min, max]),
        );
        r
    }

    /// Material 3 Expressive range slider with two 4dp×44dp vertical pill handles,
    /// 6dp track gaps, `min_separation` clamping, discrete ticks, and teardrop value bubble.
    pub fn expressive_range_slider(
        &mut self,
        ids: [u64; 2],
        label: &str,
        values: &mut [f32; 2],
        range: std::ops::RangeInclusive<f32>,
        options: SliderExpressiveOptions,
    ) -> [Response; 2] {
        let min = *range.start();
        let max = *range.end();
        assert!(min.is_finite() && max.is_finite() && min < max);
        let step = if options.step_size.is_finite() && options.step_size > 0.0 {
            options.step_size
        } else {
            0.0
        };
        let min_sep = if options.min_separation.is_finite() && options.min_separation > 0.0 {
            options.min_separation.min(max - min)
        } else {
            0.0
        };
        for value in values.iter_mut() {
            *value = if value.is_finite() {
                value.clamp(min, max)
            } else {
                min
            };
        }
        if values[0] > values[1] {
            values.swap(0, 1);
        }
        if values[1] - values[0] < min_sep {
            values[1] = (values[0] + min_sep).min(max);
            if values[1] - values[0] < min_sep {
                values[0] = (values[1] - min_sep).max(min);
            }
        }
        let previous = *values;
        let b = self.allocate(48.0);
        let track_x = b.x + 8.0;
        let track_w = (b.width - 16.0).max(1.0);
        let track_right = track_x + track_w;
        let start = track_x + 2.0;
        let travel = (track_w - 4.0).max(1.0);
        let mut responses = [Response::default(); 2];
        for i in 0..2 {
            let x = start + (values[i] - min) / (max - min) * travel;
            responses[i] = self.interact(
                ids[i],
                Rect {
                    x: x - 12.0,
                    width: 24.0,
                    ..b
                },
                options.enabled,
            );
        }
        if options.enabled
            && self.enabled
            && self.input.pressed
            && !self.input.cancelled
            && self.ui.active.is_none()
            && let Some(p) = self.input.pointer
            && b.intersect(self.clip).contains(p)
        {
            let value = min + ((p[0] - start) / travel).clamp(0.0, 1.0) * (max - min);
            let i = usize::from((value - values[1]).abs() < (value - values[0]).abs());
            self.ui.active = Some(ids[i]);
            self.ui.capture = Capture::Pointer;
            self.ui.focused = Some(ids[i]);
            self.ui.keyboard_focus = false;
            responses[i].pressed = true;
        }
        for i in 0..2 {
            let r = &mut responses[i];
            r.focused = r.enabled && self.ui.focused == Some(ids[i]);
            r.pressed = r.enabled
                && self.ui.active == Some(ids[i])
                && !self.input.released
                && !self.input.cancelled;
            if r.enabled && !self.input.cancelled && !self.input.blurred {
                if self.ui.active == Some(ids[i])
                    && self.ui.capture == Capture::Pointer
                    && let Some(p) = self.input.pointer
                {
                    values[i] = min + ((p[0] - start) / travel).clamp(0.0, 1.0) * (max - min);
                }
                let increment = if step > 0.0 {
                    step
                } else {
                    (max - min) / 100.0
                };
                if let Some(key) = self.key_for(*r) {
                    match key {
                        Key::Left | Key::Down => values[i] -= increment,
                        Key::Right | Key::Up => values[i] += increment,
                        Key::Home => values[i] = min,
                        Key::End => values[i] = max,
                        _ => {}
                    }
                }
                if step > 0.0 && values[i] != previous[i] {
                    values[i] = min + ((values[i] - min) / step).round() * step;
                }
                if i == 0 {
                    let upper = (values[1] - min_sep).max(min);
                    if step > 0.0 && values[0] > upper {
                        let floored = min + ((upper - min) / step).floor() * step;
                        values[0] = floored.clamp(min, upper);
                    } else {
                        values[0] = values[0].clamp(min, upper);
                    }
                } else {
                    let lower = (values[0] + min_sep).min(max);
                    if step > 0.0 && values[1] < lower {
                        let ceiled = min + ((lower - min) / step).ceil() * step;
                        values[1] = ceiled.clamp(lower, max);
                    } else {
                        values[1] = values[1].clamp(lower, max);
                    }
                }
            }
            r.changed = values[i] != previous[i];
        }
        let hx0 = start + (values[0] - min) / (max - min) * travel;
        let hx1 = start + (values[1] - min) / (max - min) * travel;
        let cy = b.y + 24.0;
        let track_y = cy - 8.0;
        let gap_half = 8.0; // 2dp half-handle + 6dp handle-to-track gap
        let t = self.ui.theme;
        let is_enabled = options.enabled && self.enabled;
        let active_color = if is_enabled {
            t.primary
        } else {
            t.text.opacity(0.38)
        };
        let inactive_color = if is_enabled {
            t.secondary_container
        } else {
            t.text.opacity(0.12)
        };
        let active_stop_color = if is_enabled {
            t.on_primary
        } else {
            t.inverse_on_surface.opacity(0.38)
        };
        let inactive_stop_color = if is_enabled {
            t.on_secondary_container
        } else {
            t.text.opacity(0.38)
        };
        // Left inactive track
        let left_end = (hx0 - gap_half).clamp(track_x, track_right);
        if left_end > track_x {
            let w = left_end - track_x;
            self.rect(
                Rect {
                    x: track_x,
                    y: track_y,
                    width: w,
                    height: 16.0,
                },
                8.0_f32.min(w * 0.5),
                inactive_color,
            );
        }
        // Center active track
        let mid_start = (hx0 + gap_half).clamp(track_x, track_right);
        let mid_end = (hx1 - gap_half).clamp(track_x, track_right);
        if mid_end > mid_start {
            let w = mid_end - mid_start;
            self.rect(
                Rect {
                    x: mid_start,
                    y: track_y,
                    width: w,
                    height: 16.0,
                },
                8.0_f32.min(w * 0.5),
                active_color,
            );
        }
        // Right inactive track
        let right_start = (hx1 + gap_half).clamp(track_x, track_right);
        if track_right > right_start {
            let w = track_right - right_start;
            self.rect(
                Rect {
                    x: right_start,
                    y: track_y,
                    width: w,
                    height: 16.0,
                },
                8.0_f32.min(w * 0.5),
                inactive_color,
            );
        }
        if step > 0.0 {
            let steps = ((max - min) / step).round().clamp(1.0, 128.0) as usize;
            for i in 0..=steps {
                let tx = start + travel * (i as f32 / steps as f32);
                if (tx - hx0).abs() < gap_half || (tx - hx1).abs() < gap_half {
                    continue;
                }
                self.rect(
                    Rect {
                        x: tx - 2.0,
                        y: cy - 2.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    2.0,
                    if tx > hx0 && tx < hx1 {
                        active_stop_color
                    } else {
                        inactive_stop_color
                    },
                );
            }
        } else {
            if left_end >= track_x + 8.0 {
                self.rect(
                    Rect {
                        x: track_x + 4.0,
                        y: cy - 2.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    2.0,
                    inactive_stop_color,
                );
            }
            if right_start <= track_right - 8.0 {
                self.rect(
                    Rect {
                        x: track_right - 8.0,
                        y: cy - 2.0,
                        width: 4.0,
                        height: 4.0,
                    },
                    2.0,
                    inactive_stop_color,
                );
            }
        }
        for i in 0..2 {
            let hx = if i == 0 { hx0 } else { hx1 };
            self.rect(
                Rect {
                    x: hx - 2.0,
                    y: cy - 22.0,
                    width: 4.0,
                    height: 44.0,
                },
                2.0,
                active_color,
            );
            responses[i].bounds = Rect {
                x: hx - 12.0,
                width: 24.0,
                ..b
            };
            if options.show_value_bubble
                && responses[i].enabled
                && (responses[i].pressed || responses[i].focused)
            {
                self.paint_slider_value_bubble(hx, cy - 22.0, values[i]);
            }
            self.focus_ring(responses[i]);
            self.semantic(
                responses[i],
                Role::Slider,
                label,
                false,
                None,
                Some(values[i]),
                Some([min, max]),
            );
        }
        responses
    }

    /// Thick horizontal Material 3 Expressive pill-fill slider (brightness/media HUD style)
    /// with optional embedded `SystemIcon` inside the active pill.
    pub fn pill_slider(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        icon: Option<SystemIcon>,
        enabled: bool,
    ) -> Response {
        let min = *range.start();
        let max = (*range.end()).max(min + 1e-6);
        *value = value.clamp(min, max);
        let mut r = self.interact(id, bounds, enabled);
        let mut frac = ((*value - min) / (max - min)).clamp(0.0, 1.0);
        if r.pressed && let Some(p) = self.input.pointer {
            let next = ((p[0] - bounds.x) / bounds.width.max(1.0)).clamp(0.0, 1.0);
            let next_val = min + next * (max - min);
            if (next_val - *value).abs() > 1e-5 {
                *value = next_val;
                frac = next;
                r.changed = true;
            }
        }
        if r.focused && enabled {
            let step = (max - min) * 0.05;
            if self.pressed_key(Key::Right) || self.pressed_key(Key::Up) {
                *value = (*value + step).clamp(min, max);
                frac = ((*value - min) / (max - min)).clamp(0.0, 1.0);
                r.changed = true;
            } else if self.pressed_key(Key::Left) || self.pressed_key(Key::Down) {
                *value = (*value - step).clamp(min, max);
                frac = ((*value - min) / (max - min)).clamp(0.0, 1.0);
                r.changed = true;
            }
        }
        let t = self.theme();
        let track_h = (bounds.height * 0.25).clamp(4.0, 12.0);
        let track = Rect {
            x: bounds.x,
            y: bounds.y + (bounds.height - track_h) * 0.5,
            width: bounds.width,
            height: track_h,
        };
        self.paint_rect(track, track_h * 0.5, t.surface_highest);
        let fill_w = (bounds.height + (bounds.width - bounds.height).max(0.0) * frac).min(bounds.width);
        let pill = Rect { width: fill_w, ..bounds };
        let rad = (pill.height * 0.5).min(pill.width * 0.5);
        self.paint_rect(pill, rad, if r.enabled { t.primary } else { t.text.opacity(0.12) });
        if let Some(ic) = icon {
            let sz = (bounds.height * 0.52).clamp(16.0, 24.0);
            let icon_rect = Rect {
                x: pill.x + (pill.width - sz - 10.0).max(8.0),
                y: pill.y + (pill.height - sz) * 0.5,
                width: sz,
                height: sz,
            };
            self.paint_system_icon(icon_rect, ic, if r.enabled { t.on_primary } else { t.text.opacity(0.38) });
        }
        self.focus_ring(r);
        self.semantic(r, Role::Slider, label, false, None, Some(*value), Some([min, max]));
        r
    }

    /// Vertical Material 3 Expressive pill-fill slider (volume HUD style) with bottom-anchored
    /// active fill and optional embedded `SystemIcon`.
    pub fn vertical_pill_slider(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        icon: Option<SystemIcon>,
        enabled: bool,
    ) -> Response {
        let min = *range.start();
        let max = (*range.end()).max(min + 1e-6);
        *value = value.clamp(min, max);
        let mut r = self.interact(id, bounds, enabled);
        let mut frac = ((*value - min) / (max - min)).clamp(0.0, 1.0);
        if r.pressed && let Some(p) = self.input.pointer {
            let next = (1.0 - (p[1] - bounds.y) / bounds.height.max(1.0)).clamp(0.0, 1.0);
            let next_val = min + next * (max - min);
            if (next_val - *value).abs() > 1e-5 {
                *value = next_val;
                frac = next;
                r.changed = true;
            }
        }
        if r.focused && enabled {
            let step = (max - min) * 0.05;
            if self.pressed_key(Key::Up) || self.pressed_key(Key::Right) {
                *value = (*value + step).clamp(min, max);
                frac = ((*value - min) / (max - min)).clamp(0.0, 1.0);
                r.changed = true;
            } else if self.pressed_key(Key::Down) || self.pressed_key(Key::Left) {
                *value = (*value - step).clamp(min, max);
                frac = ((*value - min) / (max - min)).clamp(0.0, 1.0);
                r.changed = true;
            }
        }
        let t = self.theme();
        let rad = (bounds.width * 0.5).min(bounds.height * 0.5);
        self.paint_rect(bounds, rad, t.surface_highest);
        let min_fill = bounds.width.min(bounds.height);
        let fill_h = (min_fill + (bounds.height - min_fill).max(0.0) * frac).min(bounds.height);
        let fill = Rect {
            x: bounds.x,
            y: bounds.y + bounds.height - fill_h,
            width: bounds.width,
            height: fill_h,
        };
        self.paint_rect(fill, rad, if r.enabled { t.primary } else { t.text.opacity(0.12) });
        if let Some(ic) = icon {
            let sz = (bounds.width * 0.48).clamp(16.0, 24.0);
            let icon_rect = Rect {
                x: fill.x + (fill.width - sz) * 0.5,
                y: fill.y + fill.height - sz - 10.0,
                width: sz,
                height: sz,
            };
            self.paint_system_icon(icon_rect, ic, if r.enabled { t.on_primary } else { t.text.opacity(0.38) });
        }
        self.focus_ring(r);
        self.semantic(r, Role::Slider, label, false, None, Some(*value), Some([min, max]));
        r
    }
}

#[cfg(test)]
#[path = "slider.spec.rs"]
mod tests;
