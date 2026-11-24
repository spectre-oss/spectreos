use super::{
    ActiveLayer, ITALIC_SHEAR, MAX_LAYER_DEPTH, Renderer, Result,
    atlas::{GlyphKey, clip_glyph},
    fonts::text_glyph_params,
    geometry::Affine2D,
};
use crate::{DrawCommand, Rect};
use glow::HasContext;

impl Renderer {
    pub(super) fn execute_commands(
        &mut self,
        commands: &[DrawCommand],
        size: [u32; 2],
        scale: f32,
        logical_w: f32,
        logical_h: f32,
    ) -> Result<()> {
        for command in commands {
            match command {
                DrawCommand::Clear(_) => {}
                DrawCommand::PushScaleY { origin, scale: sy } => {
                    if self.transforms.len() >= 64 {
                        return Err("Bhoo scale stack is full".into());
                    }
                    let prev_scissor = self.current_gpu_scissor();
                    let next_transform = self.transform.then(Affine2D::scale_y(*origin, *sy));
                    let next_scissor = if next_transform.is_identity() {
                        None
                    } else {
                        Some(self.screen_clip)
                    };
                    if next_scissor != prev_scissor {
                        self.flush_vertices(prev_scissor, size, scale)?;
                    }
                    self.transforms.push((self.transform, self.local_clip, true));
                    self.transform = next_transform;
                    self.local_clip = None;
                }
                DrawCommand::PopScaleY => {
                    let Some((prev_transform, prev_local_clip, is_scale_y)) =
                        self.transforms.pop()
                    else {
                        return Err("Unbalanced Bhoo scale stack".into());
                    };
                    if !is_scale_y {
                        return Err("Unbalanced Bhoo scale stack".into());
                    }
                    let prev_scissor = self.current_gpu_scissor();
                    let next_scissor = if prev_transform.is_identity() {
                        None
                    } else {
                        Some(self.screen_clip)
                    };
                    if next_scissor != prev_scissor {
                        self.flush_vertices(prev_scissor, size, scale)?;
                    }
                    self.transform = prev_transform;
                    self.local_clip = prev_local_clip;
                }
                DrawCommand::PushTransform {
                    origin,
                    translate,
                    scale: s,
                    rotation_rad,
                } => {
                    if self.transforms.len() >= 64 {
                        return Err("Bhoo transform stack is full".into());
                    }
                    let prev_scissor = self.current_gpu_scissor();
                    let local = Affine2D::from_parts(*origin, *translate, *s, *rotation_rad);
                    let next_transform = self.transform.then(local);
                    let next_scissor = if next_transform.is_identity() {
                        None
                    } else {
                        Some(self.screen_clip)
                    };
                    if next_scissor != prev_scissor {
                        self.flush_vertices(prev_scissor, size, scale)?;
                    }
                    self.transforms
                        .push((self.transform, self.local_clip, false));
                    self.transform = next_transform;
                    self.local_clip = None;
                }
                DrawCommand::PopTransform => {
                    let Some((prev_transform, prev_local_clip, is_scale_y)) =
                        self.transforms.pop()
                    else {
                        return Err("Unbalanced Bhoo transform stack".into());
                    };
                    if is_scale_y {
                        return Err("Unbalanced Bhoo transform stack".into());
                    }
                    let prev_scissor = self.current_gpu_scissor();
                    let next_scissor = if prev_transform.is_identity() {
                        None
                    } else {
                        Some(self.screen_clip)
                    };
                    if next_scissor != prev_scissor {
                        self.flush_vertices(prev_scissor, size, scale)?;
                    }
                    self.transform = prev_transform;
                    self.local_clip = prev_local_clip;
                }
                DrawCommand::PushClip(bounds) => {
                    let prev_scissor = self.current_gpu_scissor();
                    let next_screen_clip = self
                        .screen_clip
                        .intersect(self.transform.transform_rect_aabb(*bounds));
                    let next_scissor = if self.transform.is_identity() {
                        None
                    } else {
                        Some(next_screen_clip)
                    };
                    if next_scissor != prev_scissor {
                        self.flush_vertices(prev_scissor, size, scale)?;
                    }
                    self.clips.push((self.screen_clip, self.local_clip));
                    self.screen_clip = next_screen_clip;
                    if !self.transform.is_identity() {
                        self.local_clip =
                            Some(self.local_clip.map_or(*bounds, |c| c.intersect(*bounds)));
                    }
                }
                DrawCommand::PopClip => {
                    let Some((prev_screen_clip, prev_local_clip)) = self.clips.pop() else {
                        return Err("Unbalanced Bhoo clip stack".into());
                    };
                    let prev_scissor = self.current_gpu_scissor();
                    let next_scissor = if self.transform.is_identity() {
                        None
                    } else {
                        Some(prev_screen_clip)
                    };
                    if next_scissor != prev_scissor {
                        self.flush_vertices(prev_scissor, size, scale)?;
                    }
                    self.screen_clip = prev_screen_clip;
                    self.local_clip = prev_local_clip;
                }
                DrawCommand::PushLayer { bounds, alpha } => {
                    let alpha = alpha.clamp(0.0, 1.0);
                    let composite_rect = self
                        .transform
                        .transform_rect_aabb(*bounds)
                        .intersect(self.screen_clip);
                    if alpha <= 0.0 || composite_rect.width <= 0.0 || composite_rect.height <= 0.0 {
                        let prev_scissor = self.current_gpu_scissor();
                        self.flush_vertices(prev_scissor, size, scale)?;
                        self.active_layers.push(ActiveLayer {
                            pool_index: 0,
                            alpha: 0.0,
                            composite_rect: Rect::default(),
                            prev_screen_clip: self.screen_clip,
                            prev_local_clip: self.local_clip,
                            skipped: true,
                            passthrough: false,
                        });
                        self.screen_clip = Rect::default();
                        self.local_clip = Some(Rect::default());
                        continue;
                    }
                    // Fast path: fully opaque layers need no offscreen FBO or GMEM resolve.
                    if alpha >= 0.996 {
                        let prev_scissor = self.current_gpu_scissor();
                        let next_scissor = if self.transform.is_identity() {
                            None
                        } else {
                            Some(composite_rect)
                        };
                        if next_scissor != prev_scissor {
                            self.flush_vertices(prev_scissor, size, scale)?;
                        }
                        let prev_screen_clip = self.screen_clip;
                        let prev_local_clip = self.local_clip;
                        self.screen_clip = composite_rect;
                        if !self.transform.is_identity() {
                            self.local_clip =
                                Some(self.local_clip.map_or(*bounds, |c| c.intersect(*bounds)));
                        }
                        self.active_layers.push(ActiveLayer {
                            pool_index: 0,
                            alpha: 1.0,
                            composite_rect,
                            prev_screen_clip,
                            prev_local_clip,
                            skipped: true,
                            passthrough: true,
                        });
                        continue;
                    }
                    let depth = self
                        .active_layers
                        .iter()
                        .filter(|l| !l.skipped)
                        .count();
                    if depth >= MAX_LAYER_DEPTH {
                        return Err("Bhoo layer stack is full".into());
                    }
                    let prev_scissor = self.current_gpu_scissor();
                    self.flush_vertices(prev_scissor, size, scale)?;
                    self.ensure_layer_target(depth, size)?;
                    let fbo = self.layer_pool[depth].fbo;
                    self.stats.layers += 1;
                    // SAFETY: GLES context is current and `fbo` is complete.
                    // Invalidate dead tiles first so TBDR GPUs (Adreno/Mali) never load stale DRAM,
                    // and scissor-clear only `composite_rect` (padded by 2px for AA edges).
                    unsafe {
                        self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
                        self.gl.invalidate_framebuffer(
                            glow::FRAMEBUFFER,
                            &[glow::COLOR_ATTACHMENT0],
                        );
                        let pw = size[0] as f32;
                        let ph = size[1] as f32;
                        let x0 = ((composite_rect.x * scale).floor() - 2.0).clamp(0.0, pw) as i32;
                        let x1 = (((composite_rect.x + composite_rect.width) * scale).ceil() + 2.0)
                            .clamp(0.0, pw) as i32;
                        let y_top =
                            ((composite_rect.y * scale).floor() - 2.0).clamp(0.0, ph) as i32;
                        let y_bot = (((composite_rect.y + composite_rect.height) * scale).ceil()
                            + 2.0)
                            .clamp(0.0, ph) as i32;
                        self.gl.enable(glow::SCISSOR_TEST);
                        self.gl.scissor(
                            x0,
                            size[1] as i32 - y_bot,
                            (x1 - x0).max(0),
                            (y_bot - y_top).max(0),
                        );
                        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
                        self.gl.clear(glow::COLOR_BUFFER_BIT);
                        self.gl.disable(glow::SCISSOR_TEST);
                    }
                    let prev_screen_clip = self.screen_clip;
                    let prev_local_clip = self.local_clip;
                    self.screen_clip = composite_rect;
                    if !self.transform.is_identity() {
                        self.local_clip =
                            Some(self.local_clip.map_or(*bounds, |c| c.intersect(*bounds)));
                    }
                    self.active_layers.push(ActiveLayer {
                        pool_index: depth,
                        alpha,
                        composite_rect,
                        prev_screen_clip,
                        prev_local_clip,
                        skipped: false,
                        passthrough: false,
                    });
                    self.sync_viewport(logical_w, logical_h);
                }
                DrawCommand::PopLayer => {
                    let Some(layer) = self.active_layers.pop() else {
                        return Err("Unbalanced Bhoo layer stack".into());
                    };
                    if layer.skipped {
                        if layer.passthrough {
                            let prev_scissor = self.current_gpu_scissor();
                            let next_scissor = if self.transform.is_identity() {
                                None
                            } else {
                                Some(layer.prev_screen_clip)
                            };
                            if next_scissor != prev_scissor {
                                self.flush_vertices(prev_scissor, size, scale)?;
                            }
                        }
                        self.screen_clip = layer.prev_screen_clip;
                        self.local_clip = layer.prev_local_clip;
                        continue;
                    }
                    let prev_scissor = self.current_gpu_scissor();
                    self.flush_vertices(prev_scissor, size, scale)?;
                    let parent_fbo = self.active_fbo();
                    self.screen_clip = layer.prev_screen_clip;
                    self.local_clip = layer.prev_local_clip;
                    let layer_texture = self.layer_pool[layer.pool_index].texture;
                    // SAFETY: bind parent FBO and composite layer texture with premultiplied alpha.
                    unsafe {
                        self.gl.bind_framebuffer(glow::FRAMEBUFFER, parent_fbo);
                        self.gl.active_texture(glow::TEXTURE1);
                        self.gl.bind_texture(glow::TEXTURE_2D, Some(layer_texture));
                        self.gl.active_texture(glow::TEXTURE0);
                    }
                    self.sync_viewport(logical_w, logical_h);
                    let r = layer.composite_rect;
                    let u0 = r.x / logical_w;
                    let u1 = (r.x + r.width) / logical_w;
                    let v0 = 1.0 - r.y / logical_h;
                    let v1 = 1.0 - (r.y + r.height) / logical_h;
                    let half = [r.width * 0.5, r.height * 0.5];
                    for ([cx, cy], [u, v]) in [
                        ([r.x, r.y], [u0, v0]),
                        ([r.x + r.width, r.y], [u1, v0]),
                        ([r.x + r.width, r.y + r.height], [u1, v1]),
                        ([r.x, r.y + r.height], [u0, v1]),
                    ] {
                        self.vertices.extend_from_slice(&[
                            cx,
                            cy,
                            0.0,
                            0.0,
                            u,
                            v,
                            1.0,
                            1.0,
                            1.0,
                            layer.alpha,
                            half[0],
                            half[1],
                            0.0,
                            0.1,
                        ]);
                    }
                    self.flush_vertices(None, size, scale)?;
                    // SAFETY: restore image atlas binding on TEXTURE1.
                    unsafe {
                        self.gl.active_texture(glow::TEXTURE1);
                        self.gl
                            .bind_texture(glow::TEXTURE_2D, Some(self.image_texture));
                        self.gl.active_texture(glow::TEXTURE0);
                    }
                }
                DrawCommand::Ellipse { bounds, color } => {
                    self.quad(*bounds, [0.0; 4], *color, 0.0, 0.25)
                }
                DrawCommand::Outline {
                    bounds,
                    radius,
                    width,
                    color,
                } => self.quad(
                    *bounds,
                    [*width, 0.0, *width, 0.0],
                    *color,
                    radius.max(0.0),
                    if *radius < 0.0 { 0.48 } else { 0.4 },
                ),
                DrawCommand::RoundedRect {
                    bounds,
                    radius,
                    color,
                } => {
                    self.quad(*bounds, [0.0; 4], *color, *radius, 0.0);
                }
                DrawCommand::Ripple {
                    bounds,
                    corner_radius,
                    center,
                    radius,
                    color,
                    soft,
                } => {
                    let cx = center[0] - bounds.x - bounds.width * 0.5;
                    let cy = center[1] - bounds.y - bounds.height * 0.5;
                    self.quad(
                        *bounds,
                        [cx, cy, cx, cy],
                        *color,
                        *corner_radius,
                        if *soft {
                            2.0 + *radius
                        } else {
                            10002.0 + *radius
                        },
                    );
                }
                DrawCommand::Line {
                    from,
                    to,
                    width,
                    color,
                } => {
                    let pad = *width * 0.5 + 1.0;
                    let rect = Rect {
                        x: from[0].min(to[0]) - pad,
                        y: from[1].min(to[1]) - pad,
                        width: (to[0] - from[0]).abs() + 2.0 * pad,
                        height: (to[1] - from[1]).abs() + 2.0 * pad,
                    };
                    let x = (to[0] - from[0]) * 0.5;
                    let y = (to[1] - from[1]) * 0.5;
                    self.quad(rect, [x, y, x, y], *color, *width * 0.5, -1.25);
                }
                DrawCommand::Polyline {
                    points,
                    count,
                    width,
                    color,
                } => {
                    self.polyline(*points, *count, *width, *color);
                }
                DrawCommand::Arc {
                    bounds,
                    start,
                    sweep,
                    stroke,
                    color,
                    rounded,
                } => {
                    if *sweep > 0.0001 {
                        let radius = (bounds.width.min(bounds.height) - *stroke).max(0.0) * 0.5;
                        self.quad(
                            *bounds,
                            [*start, *sweep, *start, *sweep],
                            *color,
                            *stroke,
                            if *rounded {
                                -2.0 - radius
                            } else {
                                -10002.0 - radius
                            },
                        );
                    }
                }
                DrawCommand::Loading {
                    bounds,
                    sample,
                    color,
                } => {
                    self.loading(*bounds, *sample, *color);
                }
                DrawCommand::Shape {
                    bounds,
                    index,
                    color,
                    rotation,
                } => self.static_shape(*bounds, *index as usize, *color, *rotation),
                DrawCommand::Image {
                    bounds,
                    uv,
                    radius,
                    color,
                    image,
                } => {
                    let a = self.images[&image.id];
                    let crop = [
                        a[0] + (a[2] - a[0]) * uv[0],
                        a[1] + (a[3] - a[1]) * uv[1],
                        a[0] + (a[2] - a[0]) * uv[2],
                        a[1] + (a[3] - a[1]) * uv[3],
                    ];
                    if image.msdf_px_range > 0.0 {
                        self.quad(*bounds, crop, *color, image.msdf_px_range, -0.58);
                    } else {
                        self.quad(*bounds, crop, *color, *radius, -0.25);
                    }
                }
                DrawCommand::Text {
                    bounds,
                    text,
                    color,
                    style,
                } => {
                    if bounds.width <= 0.0
                        || bounds.height <= 0.0
                        || !style.size.is_finite()
                        || style.size <= 0.0
                    {
                        continue;
                    }
                    let screen_bounds = self
                        .transform
                        .transform_rect_aabb(*bounds)
                        .intersect(self.screen_clip);
                    if screen_bounds.width <= 0.0 || screen_bounds.height <= 0.0 {
                        continue;
                    }
                    let visible = match self.active_local_clip() {
                        Some(clip) => bounds.intersect(clip),
                        None => *bounds,
                    };
                    if visible.width <= 0.0 || visible.height <= 0.0 {
                        continue;
                    }
                    let (family, weight, italic, px_size, factor) =
                        text_glyph_params(*style, scale);
                    let shaped = self.get_or_shape_line(text, family, weight, italic, px_size);
                    let snap_1to1 = self.transform.is_identity() && (factor - 1.0).abs() < 0.04;
                    let draw_factor = if (factor - 1.0).abs() < 0.04 {
                        1.0
                    } else {
                        factor
                    };
                    let n = shaped.glyphs.len();
                    let extra_ls = if n > 1 {
                        style.letter_spacing * (n - 1) as f32
                    } else {
                        0.0
                    };
                    let width =
                        (shaped.total_advance_px / scale * draw_factor + extra_ls).max(0.0);
                    let mut x = match style.align {
                        1 => bounds.x + (bounds.width - width) * 0.5,
                        2 => bounds.x + bounds.width - style.padding - width,
                        _ => bounds.x + style.padding,
                    };
                    let offset = shaped.baseline_offset_px / scale * draw_factor;
                    let mut baseline = bounds.y + bounds.height * 0.5 + offset;
                    if snap_1to1 {
                        baseline = (baseline * scale).round() / scale;
                    }
                    for sg in &shaped.glyphs {
                        if x >= visible.x + visible.width && sg.x_offset >= 0.0 {
                            break;
                        }
                        let pen_x = x + sg.x_offset / scale * draw_factor;
                        let pen_y = baseline - sg.y_offset / scale * draw_factor;
                        let px_x = pen_x * scale;
                        let sub_bin = if px_size <= 48 {
                            ((px_x - px_x.floor()) * 4.0).floor().clamp(0.0, 3.0) as u8
                        } else {
                            0
                        };
                        let key = GlyphKey {
                            font_id: sg.font_id,
                            glyph_id: sg.glyph_id,
                            px_size,
                            sub_bin,
                            synthetic_bold: sg.synthetic_bold,
                        };
                        if let Some(glyph) = self.glyphs.get(&key).copied()
                            && glyph.width > 0
                            && glyph.height > 0
                        {
                            let shear = if sg.synthetic_italic {
                                ITALIC_SHEAR
                            } else {
                                0.0
                            };
                            let (gx, gy) = if snap_1to1 {
                                let ix = if px_size <= 48 {
                                    px_x.floor()
                                } else {
                                    px_x.round()
                                };
                                let iy = (pen_y * scale).round();
                                (
                                    (ix + glyph.left as f32) / scale,
                                    (iy + glyph.top as f32) / scale,
                                )
                            } else {
                                (
                                    pen_x + glyph.left as f32 / scale * draw_factor,
                                    pen_y + glyph.top as f32 / scale * draw_factor,
                                )
                            };
                            let gw = glyph.width as f32 / scale * draw_factor;
                            let gh = glyph.height as f32 / scale * draw_factor;
                            let rect = Rect {
                                x: gx,
                                y: gy,
                                width: gw,
                                height: gh,
                            };
                            if let Some((rect, uv)) = clip_glyph(rect, glyph.uv, visible) {
                                self.glyph_quad(rect, uv, *color, pen_y, shear);
                            }
                        }
                        x += sg.x_advance / scale * draw_factor + style.letter_spacing;
                    }
                }
                DrawCommand::Shadow {
                    bounds,
                    radius,
                    offset,
                    blur,
                    spread,
                    color,
                } => {
                    // Single-pass analytical soft rounded-box shadow (8x less fillrate than layered quads).
                    let blur_r = blur.max(0.5);
                    let pad = *spread + blur_r;
                    let b = Rect {
                        x: bounds.x + offset[0] - pad,
                        y: bounds.y + offset[1] - pad,
                        width: (bounds.width + 2.0 * pad).max(0.0),
                        height: (bounds.height + 2.0 * pad).max(0.0),
                    };
                    let inner_r = (*radius + *spread).max(0.0);
                    self.quad(b, [blur_r, inner_r, blur_r, inner_r], *color, inner_r, -0.75);
                }
            }
        }
        if !self.transforms.is_empty() {
            return Err("Unbalanced Bhoo transform/scale stack".into());
        }
        if !self.clips.is_empty() {
            return Err("Unbalanced Bhoo clip stack".into());
        }
        if !self.active_layers.is_empty() {
            return Err("Unbalanced Bhoo layer stack".into());
        }
        let final_scissor = self.current_gpu_scissor();
        self.flush_vertices(final_scissor, size, scale)
    }
}
