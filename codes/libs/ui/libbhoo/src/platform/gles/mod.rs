//! Private GLES 3 renderer. All GL calls stay on the thread with the current context.
mod atlas;
mod commands;
pub(crate) mod fonts;
mod geometry;
mod shaders;

use atlas::{Glyph, GlyphKey, ShapeKey};
use fonts::{FontSet, ShapedLine};
use geometry::Affine2D;
use shaders::{FRAGMENT, VERTEX};
use crate::{Color, DrawCommand, Rect};
use glow::HasContext;
use std::{collections::HashMap, error::Error, path::Path, sync::Arc};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const GLYPH_ATLAS: usize = 2048;
const IMAGE_ATLAS: usize = 2048;
const STRIDE: i32 = 14 * 4;
const ITALIC_SHEAR: f32 = 0.212_556_56; // tan(12 deg) oblique fallback
const MAX_LAYER_DEPTH: usize = 16;


struct LayerTarget {
    fbo: glow::Framebuffer,
    texture: glow::Texture,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub quads: u32,
    pub flushes: u32,
    pub layers: u32,
    pub glyph_rows: u32,
    pub image_rows: u32,
}

struct ActiveLayer {
    pool_index: usize,
    alpha: f32,
    composite_rect: Rect,
    prev_screen_clip: Rect,
    prev_local_clip: Option<Rect>,
    skipped: bool,
    passthrough: bool,
}

pub struct Renderer {
    gl: glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    gpu_vertex_bytes: usize,
    gpu_index_quads: usize,
    vbo_quad_offset: usize,
    frame_vbo_orphaned: bool,
    stats: RenderStats,
    indices: Vec<u32>,
    screen_clip: Rect,
    local_clip: Option<Rect>,
    clips: Vec<(Rect, Option<Rect>)>,
    transform: Affine2D,
    transforms: Vec<(Affine2D, Option<Rect>, bool)>,
    layer_pool: Vec<LayerTarget>,
    active_layers: Vec<ActiveLayer>,
    target_fbo: Option<glow::Framebuffer>,
    texture: glow::Texture,
    image_texture: glow::Texture,
    image_atlas: Vec<u8>,
    image_shelf: [usize; 3],
    images: HashMap<u32, [f32; 4]>,
    viewport: glow::UniformLocation,
    fonts: FontSet,
    glyphs: HashMap<GlyphKey, Glyph>,
    shape_cache: HashMap<ShapeKey, Arc<ShapedLine>>,
    shaper_buf: harfrust::Buffer,
    zeno_scratch: zeno::Scratch,
    atlas: Vec<u8>,
    shelf: [usize; 3],
    scale: f32,
    vertices: Vec<f32>,
    path_points: Vec<[f32; 2]>,
}

impl Renderer {
    pub fn new(gl: glow::Context, font_path: &Path) -> Result<Self> {
        let fonts = FontSet::load(font_path)?;
        // SAFETY: the caller keeps the GLES context current for creation, rendering, and Drop.
        unsafe {
            let program = gl.create_program()?;
            let mut shaders = Vec::new();
            for (kind, source) in [
                (glow::VERTEX_SHADER, VERTEX),
                (glow::FRAGMENT_SHADER, FRAGMENT),
            ] {
                let shader = match gl.create_shader(kind) {
                    Ok(shader) => shader,
                    Err(e) => {
                        for shader in shaders {
                            gl.delete_shader(shader);
                        }
                        gl.delete_program(program);
                        return Err(e.into());
                    }
                };
                gl.shader_source(shader, source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    let log = gl.get_shader_info_log(shader);
                    gl.delete_shader(shader);
                    for shader in shaders {
                        gl.delete_shader(shader);
                    }
                    gl.delete_program(program);
                    return Err(format!("GLES shader compilation: {log}").into());
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }
            gl.link_program(program);
            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }
            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                gl.delete_program(program);
                return Err(format!("GLES program link: {log}").into());
            }
            let viewport = gl
                .get_uniform_location(program, "viewport")
                .ok_or("Missing viewport uniform")?;
            let vao = gl.create_vertex_array()?;
            let vbo = gl.create_buffer()?;
            let ebo = gl.create_buffer()?;
            let texture = gl.create_texture()?;
            let image_texture = gl.create_texture()?;
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            let mut offset = 0;
            for (index, size) in [2, 2, 2, 4, 2, 1, 1].into_iter().enumerate() {
                gl.enable_vertex_attrib_array(index as u32);
                gl.vertex_attrib_pointer_f32(
                    index as u32,
                    size,
                    glow::FLOAT,
                    false,
                    STRIDE,
                    offset,
                );
                offset += size * 4;
            }
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            let atlas = vec![0; GLYPH_ATLAS * GLYPH_ATLAS];
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R8 as i32,
                GLYPH_ATLAS as i32,
                GLYPH_ATLAS as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&atlas)),
            );
            let image_atlas = vec![0; IMAGE_ATLAS * IMAGE_ATLAS * 4];
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(image_texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAX_LEVEL,
                4,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                IMAGE_ATLAS as i32,
                IMAGE_ATLAS as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&image_atlas)),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.use_program(Some(program));
            gl.uniform_1_i32(gl.get_uniform_location(program, "atlas").as_ref(), 0);
            gl.uniform_1_i32(gl.get_uniform_location(program, "image_atlas").as_ref(), 1);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.use_program(None);
            eprintln!(
                "Bhoo: Wayland / {} / {}",
                gl.get_parameter_string(glow::VERSION),
                gl.get_parameter_string(glow::RENDERER)
            );
            Ok(Self {
                gl,
                program,
                vao,
                vbo,
                ebo,
                gpu_vertex_bytes: 0,
                gpu_index_quads: 0,
                vbo_quad_offset: 0,
                frame_vbo_orphaned: false,
                stats: RenderStats::default(),
                indices: Vec::new(),
                screen_clip: Rect::default(),
                local_clip: None,
                clips: Vec::new(),
                transform: Affine2D::IDENTITY,
                transforms: Vec::new(),
                layer_pool: Vec::new(),
                active_layers: Vec::new(),
                target_fbo: None,
                texture,
                image_texture,
                image_atlas,
                image_shelf: [16, 16, 0],
                images: HashMap::new(),
                viewport,
                fonts,
                glyphs: HashMap::new(),
                shape_cache: HashMap::new(),
                shaper_buf: harfrust::Buffer::new(),
                zeno_scratch: zeno::Scratch::new(),
                atlas,
                shelf: [1, 1, 0],
                scale: 0.0,
                vertices: Vec::new(),
                path_points: Vec::new(),
            })
        }
    }

    fn active_local_clip(&self) -> Option<Rect> {
        if self.transform.is_identity() {
            Some(self.screen_clip)
        } else {
            self.local_clip
        }
    }

    fn current_gpu_scissor(&self) -> Option<Rect> {
        if self.transform.is_identity() {
            None
        } else {
            Some(self.screen_clip)
        }
    }

    pub(super) fn active_fbo(&self) -> Option<glow::Framebuffer> {
        self.active_layers
            .iter()
            .rev()
            .find(|layer| !layer.skipped)
            .map(|layer| self.layer_pool[layer.pool_index].fbo)
            .or(self.target_fbo)
    }

    pub(super) fn is_y_flipped(&self) -> bool {
        self.target_fbo.is_some() && !self.active_layers.iter().any(|l| !l.skipped)
    }

    pub(super) fn sync_viewport(&self, logical_w: f32, logical_h: f32) {
        let signed_h = if self.is_y_flipped() {
            -logical_h
        } else {
            logical_h
        };
        unsafe {
            self.gl
                .uniform_2_f32(Some(&self.viewport), logical_w, signed_h);
        }
    }

    fn ensure_layer_target(&mut self, index: usize, size: [u32; 2]) -> Result<()> {
        let width = size[0].max(1);
        let height = size[1].max(1);
        // SAFETY: GLES context is current during render.
        unsafe {
            while self.layer_pool.len() <= index {
                let texture = self.gl.create_texture()?;
                self.gl.active_texture(glow::TEXTURE1);
                self.gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                self.gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::NEAREST as i32,
                );
                self.gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::NEAREST as i32,
                );
                self.gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_S,
                    glow::CLAMP_TO_EDGE as i32,
                );
                self.gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_T,
                    glow::CLAMP_TO_EDGE as i32,
                );
                self.gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA8 as i32,
                    width as i32,
                    height as i32,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(None),
                );
                let fbo = self.gl.create_framebuffer()?;
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
                self.gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::TEXTURE_2D,
                    Some(texture),
                    0,
                );
                let status = self.gl.check_framebuffer_status(glow::FRAMEBUFFER);
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, self.active_fbo());
                self.gl
                    .bind_texture(glow::TEXTURE_2D, Some(self.image_texture));
                self.gl.active_texture(glow::TEXTURE0);
                if status != glow::FRAMEBUFFER_COMPLETE {
                    self.gl.delete_framebuffer(fbo);
                    self.gl.delete_texture(texture);
                    return Err(format!("Bhoo offscreen FBO incomplete: {status:#x}").into());
                }
                self.layer_pool.push(LayerTarget {
                    fbo,
                    texture,
                    width,
                    height,
                });
            }
            let target = &mut self.layer_pool[index];
            if target.width != width || target.height != height {
                self.gl.active_texture(glow::TEXTURE1);
                self.gl.bind_texture(glow::TEXTURE_2D, Some(target.texture));
                self.gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA8 as i32,
                    width as i32,
                    height as i32,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(None),
                );
                self.gl
                    .bind_texture(glow::TEXTURE_2D, Some(self.image_texture));
                self.gl.active_texture(glow::TEXTURE0);
                target.width = width;
                target.height = height;
            }
        }
        Ok(())
    }

    pub fn last_stats(&self) -> RenderStats {
        self.stats
    }

    /// Pre-upload static images (e.g. launcher app icons) into the GPU image atlas and generate
    /// trilinear mipmaps once at startup so the first App Drawer / Overview open has zero hitch.
    pub fn prewarm_images(&mut self, images: &[crate::Image]) -> Result<()> {
        let mut cmds = Vec::with_capacity(images.len());
        for img in images {
            cmds.push(DrawCommand::Image {
                bounds: Rect::default(),
                uv: [0.0, 0.0, 1.0, 1.0],
                radius: 0.0,
                color: Color(255, 255, 255, 255),
                image: img.clone(),
            });
        }
        if let Some((y0, y1)) = self.prepare_images(&cmds)? {
            let rows = y1 - y0;
            // SAFETY: caller keeps the GLES context current during prewarm.
            unsafe {
                self.gl.active_texture(glow::TEXTURE1);
                self.gl
                    .bind_texture(glow::TEXTURE_2D, Some(self.image_texture));
                self.gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    y0 as i32,
                    IMAGE_ATLAS as i32,
                    rows as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(
                        &self.image_atlas[y0 * IMAGE_ATLAS * 4..y1 * IMAGE_ATLAS * 4],
                    )),
                );
                self.gl.generate_mipmap(glow::TEXTURE_2D);
                self.gl.active_texture(glow::TEXTURE0);
            }
        }
        Ok(())
    }

    /// Pre-allocate `count` offscreen layer FBOs at `size` during startup so the first
    /// translucent App Drawer / Shade / App Open transition never stalls allocating GPU memory.
    pub fn prewarm_layers(&mut self, size: [u32; 2], count: usize) -> Result<()> {
        for i in 0..count.min(MAX_LAYER_DEPTH) {
            self.ensure_layer_target(i, size)?;
        }
        Ok(())
    }

    fn flush_vertices(
        &mut self,
        scissor: Option<Rect>,
        size: [u32; 2],
        scale: f32,
    ) -> Result<()> {
        if self.vertices.is_empty() {
            return Ok(());
        }
        let batch_quads = self.vertices.len() / (14 * 4);
        if batch_quads == 0 {
            self.vertices.clear();
            return Ok(());
        }
        let quad_bytes = 14 * 4 * 4;
        // SAFETY: all objects belong to the context kept current by the window runner.
        unsafe {
            if let Some(clip) = scissor {
                let pw = size[0] as f32;
                let ph = size[1] as f32;
                let x0 = (clip.x * scale).floor().clamp(0.0, pw) as i32;
                let x1 = ((clip.x + clip.width) * scale).ceil().clamp(0.0, pw) as i32;
                let y_top = (clip.y * scale).floor().clamp(0.0, ph) as i32;
                let y_bot = ((clip.y + clip.height) * scale).ceil().clamp(0.0, ph) as i32;
                let scissor_y = if self.is_y_flipped() {
                    y_top
                } else {
                    size[1] as i32 - y_bot
                };
                self.gl.enable(glow::SCISSOR_TEST);
                self.gl.scissor(
                    x0,
                    scissor_y,
                    (x1 - x0).max(0),
                    (y_bot - y_top).max(0),
                );
            } else {
                self.gl.disable(glow::SCISSOR_TEST);
            }

            let bytes = bytemuck::cast_slice::<f32, u8>(&self.vertices);
            let needed = (self.vbo_quad_offset + batch_quads) * quad_bytes;
            if needed > self.gpu_vertex_bytes {
                self.gpu_vertex_bytes = needed.next_power_of_two().max(256 * 1024);
                self.vbo_quad_offset = 0;
                self.gl.buffer_data_size(
                    glow::ARRAY_BUFFER,
                    i32::try_from(self.gpu_vertex_bytes)
                        .map_err(|_| "Bhoo vertex buffer too large")?,
                    glow::STREAM_DRAW,
                );
                self.frame_vbo_orphaned = true;
            } else if !self.frame_vbo_orphaned {
                // Orphan the previous frame's VBO once per frame so Freedreno/Adreno never stalls
                // waiting for in-flight draws, then stream every batch into non-overlapping slices.
                self.gl.buffer_data_size(
                    glow::ARRAY_BUFFER,
                    i32::try_from(self.gpu_vertex_bytes)
                        .map_err(|_| "Bhoo vertex buffer too large")?,
                    glow::STREAM_DRAW,
                );
                self.frame_vbo_orphaned = true;
            }
            let byte_offset = i32::try_from(self.vbo_quad_offset * quad_bytes)
                .map_err(|_| "Bhoo vertex offset too large")?;
            self.gl
                .buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, byte_offset, bytes);

            let total_quads = self.vbo_quad_offset + batch_quads;
            if total_quads > self.gpu_index_quads {
                self.gpu_index_quads = total_quads.next_power_of_two();
                self.indices.clear();
                for quad in 0..self.gpu_index_quads {
                    let base =
                        u32::try_from(quad * 4).map_err(|_| "Bhoo index buffer too large")?;
                    self.indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
                self.gl
                    .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
                self.gl.buffer_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    bytemuck::cast_slice(&self.indices),
                    glow::STATIC_DRAW,
                );
            }
            let ebo_byte_offset = i32::try_from(self.vbo_quad_offset * 6 * 4)
                .map_err(|_| "Bhoo index offset too large")?;
            self.gl.draw_elements(
                glow::TRIANGLES,
                i32::try_from(batch_quads * 6).map_err(|_| "Bhoo draw count too large")?,
                glow::UNSIGNED_INT,
                ebo_byte_offset,
            );
            self.vbo_quad_offset += batch_quads;
            self.stats.quads += batch_quads as u32;
            self.stats.flushes += 1;
            if scissor.is_some() {
                self.gl.disable(glow::SCISSOR_TEST);
            }
        }
        self.vertices.clear();
        Ok(())
    }

    pub fn render(
        &mut self,
        commands: &[DrawCommand],
        size: [u32; 2],
        scale: f32,
    ) -> Result<()> {
        let image_dirty = self.prepare_images(commands)?;
        let glyph_dirty = self.prepare_glyphs(commands, scale)?;
        let logical_w = size[0] as f32 / scale;
        let logical_h = size[1] as f32 / scale;
        let clear = commands
            .iter()
            .rev()
            .find_map(|cmd| match cmd {
                DrawCommand::Clear(color) => Some(*color),
                _ => None,
            })
            .unwrap_or(Color(0, 0, 0, 255));

        self.vertices.clear();
        self.clips.clear();
        self.transforms.clear();
        self.active_layers.clear();
        self.vbo_quad_offset = 0;
        self.frame_vbo_orphaned = false;
        self.stats = RenderStats {
            glyph_rows: glyph_dirty.map_or(0, |(y0, y1)| (y1 - y0) as u32),
            image_rows: image_dirty.map_or(0, |(y0, y1)| (y1 - y0) as u32),
            ..RenderStats::default()
        };
        self.transform = Affine2D::IDENTITY;
        self.local_clip = None;
        self.screen_clip = Rect {
            x: 0.0,
            y: 0.0,
            width: logical_w,
            height: logical_h,
        };

        // SAFETY: all objects belong to the context kept current by the window runner.
        let target_fbo = unsafe {
            let id = self.gl.get_parameter_i32(glow::DRAW_FRAMEBUFFER_BINDING);
            std::num::NonZeroU32::new(id as u32).map(glow::NativeFramebuffer)
        };
        self.target_fbo = target_fbo;
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, target_fbo);
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.enable(glow::BLEND);
            self.gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.viewport(0, 0, size[0] as i32, size[1] as i32);
            if clear.3 > 0 || target_fbo.is_none() {
                self.gl.clear_color(
                    clear.0 as f32 / 255.0,
                    clear.1 as f32 / 255.0,
                    clear.2 as f32 / 255.0,
                    clear.3 as f32 / 255.0,
                );
                self.gl.clear(glow::COLOR_BUFFER_BIT);
            }
            self.gl.use_program(Some(self.program));
            self.sync_viewport(logical_w, logical_h);
            self.gl.active_texture(glow::TEXTURE0);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            if let Some((y0, y1)) = glyph_dirty {
                let rows = y1 - y0;
                self.gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    y0 as i32,
                    GLYPH_ATLAS as i32,
                    rows as i32,
                    glow::RED,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(
                        &self.atlas[y0 * GLYPH_ATLAS..y1 * GLYPH_ATLAS],
                    )),
                );
            }
            self.gl.active_texture(glow::TEXTURE1);
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.image_texture));
            if let Some((y0, y1)) = image_dirty {
                let rows = y1 - y0;
                self.gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    y0 as i32,
                    IMAGE_ATLAS as i32,
                    rows as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(Some(
                        &self.image_atlas[y0 * IMAGE_ATLAS * 4..y1 * IMAGE_ATLAS * 4],
                    )),
                );
                self.gl.generate_mipmap(glow::TEXTURE_2D);
            }
            self.gl.active_texture(glow::TEXTURE0);
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        }

        let result = self.execute_commands(commands, size, scale, logical_w, logical_h);

        // SAFETY: always restore the caller's framebuffer and leave GL state clean.
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, target_fbo);
            self.gl.bind_vertex_array(None);
            self.gl.bind_buffer(glow::ARRAY_BUFFER, None);
            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            self.gl.use_program(None);
            self.gl.active_texture(glow::TEXTURE1);
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.image_texture));
            self.gl.active_texture(glow::TEXTURE0);
        }

        result?;

        #[cfg(debug_assertions)]
        {
            // SAFETY: GL error check on current context (debug builds only to avoid GPU pipeline stalls).
            let error = unsafe { self.gl.get_error() };
            if error != glow::NO_ERROR {
                return Err(format!("GLES rendering error: {error:#x}").into());
            }
        }
        Ok(())
    }

    pub(crate) fn capture(&self, path: &Path, size: [u32; 2]) -> Result<()> {
        use std::io::Write;
        let mut rgba = vec![0; size[0] as usize * size[1] as usize * 4];
        // SAFETY: reads the current default framebuffer before swapping; buffer is sized above.
        unsafe {
            self.gl.read_pixels(
                0,
                0,
                size[0] as i32,
                size[1] as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut rgba)),
            );
        }
        let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
        write!(file, "P6\n{} {}\n255\n", size[0], size[1])?;
        for row in rgba.chunks_exact(size[0] as usize * 4).rev() {
            for pixel in row.as_chunks::<4>().0 {
                file.write_all(&pixel[..3])?;
            }
        }
        file.flush()?;
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // SAFETY: the runner drops the renderer before releasing its current EGL context.
        unsafe {
            for target in &self.layer_pool {
                self.gl.delete_framebuffer(target.fbo);
                self.gl.delete_texture(target.texture);
            }
            self.gl.delete_texture(self.texture);
            self.gl.delete_texture(self.image_texture);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.ebo);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_program(self.program);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::atlas::clip_glyph;
    use super::fonts::text_glyph_params;
    use crate::{FontFamily, TextStyle};

    #[test]
    fn clipping_adjusts_texture_coordinates() {
        let (rect, uv) = clip_glyph(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 10.0,
            },
            [0.0, 0.0, 1.0, 1.0],
            Rect {
                x: 10.0,
                y: 0.0,
                width: 10.0,
                height: 5.0,
            },
        )
        .unwrap();
        assert_eq!(rect.width, 10.0);
        assert_eq!(uv, [0.5, 0.0, 1.0, 0.5]);
    }

    #[test]
    fn affine2d_matches_canvas_order_and_composition() {
        let m = Affine2D::from_parts(
            [10.0, 20.0],
            [5.0, -3.0],
            [2.0, 3.0],
            std::f32::consts::FRAC_PI_2,
        );
        // Origin maps to origin + translate.
        let o = m.apply([10.0, 20.0]);
        assert!((o[0] - 15.0).abs() < 1e-4 && (o[1] - 17.0).abs() < 1e-4);
        // (11, 20) -> local (1,0) -> scale (2,0) -> rot +90deg (0,2) -> (15, 19).
        let p = m.apply([11.0, 20.0]);
        assert!((p[0] - 15.0).abs() < 1e-4 && (p[1] - 19.0).abs() < 1e-4);
        // Nesting: parent translate(10,0) * local scale(2,2) on (5,0) -> (20,0).
        let parent = Affine2D::from_parts([0.0, 0.0], [10.0, 0.0], [1.0, 1.0], 0.0);
        let local = Affine2D::from_parts([0.0, 0.0], [0.0, 0.0], [2.0, 2.0], 0.0);
        let composed = parent.then(local);
        let q = composed.apply([5.0, 0.0]);
        assert!((q[0] - 20.0).abs() < 1e-4 && q[1].abs() < 1e-4);
    }

    #[test]
    fn text_glyph_params_quantizes_and_clamps() {
        let (fam, wt, ital, px, factor) = text_glyph_params(
            TextStyle {
                size: 42.0,
                weight: 700,
                italic: true,
                family: FontFamily::Serif,
                ..TextStyle::default()
            },
            1.0,
        );
        assert_eq!((fam, wt, ital, px), (1, 3, true, 42));
        assert!((factor - 1.0).abs() < 1e-5);
        assert_eq!(
            text_glyph_params(TextStyle { weight: 300, ..TextStyle::default() }, 1.0).1,
            0
        );
        assert_eq!(
            text_glyph_params(TextStyle { weight: 400, ..TextStyle::default() }, 1.0).1,
            1
        );
        assert_eq!(
            text_glyph_params(TextStyle { weight: 500, ..TextStyle::default() }, 1.0).1,
            2
        );

        let (_, _, _, clamped_hi, factor_hi) = text_glyph_params(
            TextStyle {
                size: 160.0,
                ..TextStyle::default()
            },
            2.0,
        );
        assert_eq!(clamped_hi, 256);
        assert!((factor_hi - 1.25).abs() < 1e-5);

        let (_, _, _, clamped_lo, _) = text_glyph_params(
            TextStyle {
                size: 4.0,
                ..TextStyle::default()
            },
            1.0,
        );
        assert_eq!(clamped_lo, 8);
    }
}
