# GLES Rendering and Text

Status: initial renderer implemented in `codes/libs/ui/libbhoo/src/platform/gles/mod.rs`.

## Draw contract

The core emits ordered `Clear`, `RoundedRect`, `Ellipse`, `Outline`, `Text`, `Shadow`, `Ripple`, `Loading`, `Shape`, `Image`, `Arc`, `Line`, `Polyline`, clip (`PushClip`/`PopClip`), scale/transform (`PushScaleY`/`PopScaleY`, `PushTransform`/`PopTransform`), and offscreen layer (`PushLayer`/`PopLayer`) commands in logical coordinates. The normal frame begins with one clear command. The renderer takes that background color and preserves primitive order when generating the triangle stream. Arbitrarily interleaving multiple clear operations is not a supported rendering contract.

Each nonempty rectangle or visible glyph becomes two triangles, using four vertices and six shared-pattern 32-bit indices. One vertex contains fourteen `f32` values (56 bytes):

| Attribute | Floats | Purpose |
| --- | --- | --- |
| Position | 2 | Logical position (after active `Affine2D` transform) converted to clip space in the vertex shader |
| Local position | 2 | Position relative to the primitive center |
| UV | 2 | Glyph/image/FBO atlas coordinates or analytic ink/arc/line/outline parameters |
| Color | 4 | Normalized RGBA |
| Half size | 2 | Rectangle extents |
| Radius | 1 | Rounded-corner radius, clamped to the extents |
| Mode | 1 | Select rectangle, ellipse, outline, glyph, image, offscreen FBO composite (`0.1`), ink, arc, segment, or mesh coverage |

The renderer reuses its CPU vertex vector and GPU buffer capacity, uploads vertex contents with sub-data updates, and submits one indexed draw batch per contiguous state run (one draw call for an ordinary untransformed, non-layered frame; flushing cleanly before GPU scissor transitions under active transforms or offscreen FBO bind/composite boundaries). Index data is rebuilt/uploaded only when its capacity grows. This representation costs 248 geometry bytes per quad before capacity overhead, compared with the previous 336. Instancing and packed colors remain options to evaluate using profiles.

`PushClip`/`PopClip` support nested rectangular clipping. Under identity transforms, geometry and glyph UVs are cropped on the CPU against the active clip while retaining original local coordinates for rounded-rectangle shading. Under active non-identity 2D affine transforms, clipping combines local-space cropping with an axis-aligned screen-space `glScissor` box derived from the transformed clip AABB. Unsupported or unbalanced clip, transform, or layer streams return an error. Clipping remains rectangular; it is not arbitrary path masking.

## Shader and blending

Shaders use GLSL ES 3.00. The vertex shader maps a top-left logical coordinate system to clip space. The fragment shader evaluates a signed-distance rounded rectangle, ellipse, or rounded/oval outline with derivative-based edge smoothing, samples the `R8` glyph atlas (`TEXTURE0`) for text coverage (with sRGB-aware luminance contrast tuning for dark/light ink), samples the `RGBA8` image atlas (`TEXTURE1`) for premultiplied mipmapped RGBA images (`mode = -0.25`) and resolution-independent MSDF vector icons (`mode = -0.58`, using `textureLod(..., 0.0)` + `median(r,g,b)` + `fwidth` screen-pixel range), composites premultiplied offscreen layer FBO textures (`mode = 0.1`, `texture(image_atlas, v_uv) * v_color.a`), or evaluates analytic radial/arc/segment coverage. Loading shapes and triangulated static shapes use the same ordered batch with tessellated fill/fringe geometry.

The fragment output is premultiplied RGBA, blended with `ONE, ONE_MINUS_SRC_ALPHA`. The current theme uses an opaque background. Transparent desktop surfaces and color-space handling need separate validation before becoming supported behavior.

There is no depth buffer, stencil-based clipping, offscreen shadow blur pipeline, or partial repaint. Every requested frame clears and redraws the window (using pooled offscreen color FBOs only when `PushLayer`/`PopLayer` is active). Batching and idle waiting reduce different costs: batching reduces submission work, while idle waiting avoids building and presenting unnecessary frames.

## Fonts and atlas

Font resolution follows this order:

1. `WindowOptions::font`.
2. The `BHOO_FONT` environment variable.
3. A short list of common Adwaita/DejaVu Linux font paths.

Explicit paths are loaded as supplied; an invalid path produces an error rather than silently selecting another font. Font discovery through fontconfig and bundled fonts are not implemented. NixOS callers should provide a reproducible font path explicitly when the defaults are unavailable.

Starting in Wave 13B, `Renderer` loads a multi-face `FontSet` (`sans`, `serif`, and `mono` `FamilyFaces`, each holding optional `regular`, `medium`, `bold`, `italic`, and `bold_italic` faces) from sibling files beside the primary font path (matching `-Medium`, `-Bold`, `-Italic`/`-Oblique`, `-BoldItalic`/`-BoldOblique`) and standard Linux/Android font directories (`/usr/share/fonts/{Adwaita,truetype/roboto,roboto,truetype/dejavu,TTF,truetype/liberation,liberation,noto,truetype/noto}`, `/system/fonts`, and Waydroid font paths when present). `DrawCommand::Text` carries `TextStyle.family: FontFamily` (`Sans`, `Serif`, `Monospace`) and `TextStyle.italic: bool` alongside `size`, `weight`, `align`, `padding`, and `letter_spacing`. When a requested face is unavailable, `FontSet::select` falls back cleanly within the family and then to `Sans`, applying a synthetic oblique shear (`ITALIC_SHEAR = tan(12°) ≈ 0.2126`) along glyph quad vertices when an italic face is missing and horizontal bitmap dilation (`dilate_bold_bitmap`, widening by `1..=3px` proportional to glyph size) when a bold face (`weight >= 600`) is missing.

Text shaping and glyph rasterization on GLES use `harfrust` (pure-Rust HarfBuzz OpenType shaping for GSUB/GPOS kerning, ligatures, and Persian/Arabic contextual joining), `unicode-bidi` (UAX #9 visual run ordering with an ASCII fast path), `skrifa` (`HintingMode::Smooth { lcd_subpixel: None, preserve_linear_metrics: true }` vertical-only stem hinting for `<= 96px`), and `zeno` (4x horizontal subpixel phase rasterization `0.0, 0.25, 0.50, 0.75` for `<= 48px`). Shaped lines are cached in a bounded line cache (`1024` entries) keyed by `(text, family, weight, italic, px_size)`, and glyphs are rasterized at physical pixel sizes (`px_size = round((style.size * scale).max(1.0)).clamp(8, 128)`) keyed by `GlyphKey { font_id, glyph_id, px_size, sub_bin, synthetic_bold }`. Missing codepoints automatically fall back across the loaded family, primary sans, Persian/Arabic (`Shabnam`/`Vazirmatn`/`NotoNaskhArabic`), and symbol/fallback faces (`DejaVuSans`/`AdwaitaSans`). Metrics and UV rectangles are cached per `GlyphKey` for the current display scale; a scale change clears the cache and atlas.

Glyph coverage is stored in a `2048×2048` `R8` texture (expanded from `1024×1024` in Wave 13B): approximately 4 MiB each for the CPU atlas and GPU texture, excluding font data, cache metadata, and driver overhead. Decoded bitmap images and 48x48 MSDF system icons share a `2048×2048` `RGBA8` `image_atlas` on `TEXTURE1` with `PAD = 16` texels between tiles (matching `TEXTURE_MAX_LEVEL = 4` so 16x16 mip blocks never bleed across neighbors). Normal RGBA images are premultiplied before upload and `glGenerateMipmap`; MSDF icons (`msdf_px_range > 0.0`) are uploaded raw without premultiplication and sampled at mip level 0 (`textureLod`). When glyphs or images change, the dirty rows are uploaded once before drawing; unchanged frames do not upload atlas data.

If inserting a new glyph overflows the `2048×2048` shelf, `prepare_glyphs` clears the atlas once and repacks only the glyphs referenced by the current frame; if a single frame's unique glyphs still exceed atlas capacity, it returns an explicit error and terminates the runner through its error path. This avoids silent corruption, though true LRU eviction, partial sub-rectangle atlas uploads, and cache budgets remain open work.

## Text placement and clipping

Text commands carry `size`, `weight`, `italic`, `family` (`FontFamily`), `align`, `padding`, and `letter_spacing`. Core body labels wrap at whitespace using backend-supplied advance metrics (`Renderer::text_metrics` populated from `Sans` regular and medium faces); complex-script line wrapping remains an estimate until shared shaping is wired into layout. The GLES renderer shapes each line through `unicode-bidi` + `harfrust` and rasterizes glyphs at physical sizes in `8..=128px` with 4x horizontal subpixel phases (`<= 48px`) across loaded sans/serif/monospace/fallback faces, using the selected face's horizontal line metrics for baselines. Shaped glyph advances and offsets place each glyph accurately. Glyph rectangles are intersected with their text bounds and the active clip, with UVs adjusted to match the cropped geometry and transformed through the active `Affine2D` matrix (with optional `ITALIC_SHEAR` around the baseline).

Control characters and zero-width format characters (`U+00AD`, `U+200B`, `U+2060`, `U+FEFF`) are handled cleanly during shaping. Remaining text limitations: sharing shaped line runs between core layout wrapping and GLES drawing, ellipsis truncation, text selection, and grapheme-aware editable text cursoring remain future work.

## 2D affine transforms (`PushTransform` / `PopTransform`)

Wave 13B adds `DrawCommand::PushTransform { origin, translate, scale, rotation_rad }` and `DrawCommand::PopTransform` (emitted via `Frame::push_transform` / `Frame::pop_transform`), generalizing `PushScaleY`/`PopScaleY` into a 2D affine transform stack (`Affine2D { a, b, c, d, e, f }`) with a 64-entry depth limit in `codes/libs/ui/libbhoo/src/platform/gles/mod.rs`.

- **Matrix order:** `Affine2D::from_parts(origin, translate, scale, rotation_rad)` evaluates `translate(origin + translate) * rotate(rotation_rad) * scale(scale) * translate(-origin)`, matching Canvas 2D's `ctx.translate(ox + tx, oy + ty); ctx.rotate(rot); ctx.scale(sx, sy); ctx.translate(-ox, -oy)` order (`affine2d_matches_canvas_order_and_composition`). Nested transforms compose via `self.transform.then(local)`.
- **Geometry & clipping:** All quad, glyph, mesh, polyline, static shape, and loading vertices are transformed in logical space via `Affine2D::apply`. When `self.transform` is non-identity, clip rectangles intersect `local_clip` in local space and update `screen_clip` from `transform_rect_aabb(*bounds)`, enforced by flushing pending vertices and enabling GPU `glScissor` on `screen_clip`.
- **Stack validation:** `PushScaleY`/`PopScaleY` and `PushTransform`/`PopTransform` share the stack with an explicit type tag; mismatched pops, depth overflow (`>= 64`), or unclosed transforms at end-of-frame return an error.
- **Limits:** GPU `glScissor` is axis-aligned in screen space; clipping rotated subtrees uses conservative screen-space AABB scissors rather than stencil masks, and transform-aware inverse hit testing remains the responsibility of callers/layout nodes.

## Offscreen layer compositing (`PushLayer` / `PopLayer`)

Wave 13B adds `DrawCommand::PushLayer { bounds, alpha }` and `DrawCommand::PopLayer` (emitted via `Frame::push_layer` / `Frame::pop_layer`, clamping `alpha` to `0.0..=1.0`) for true group opacity so overlapping opaque children fade uniformly as a single flattened surface with zero bleed-through.

- **Native GLES FBO pool (`codes/libs/ui/libbhoo/src/platform/gles/mod.rs`):** `Renderer` maintains a lazily allocated pool of offscreen `LayerTarget` framebuffers (`MAX_LAYER_DEPTH = 16`), each backed by a viewport-sized `RGBA8` texture (`NEAREST` filtering, `CLAMP_TO_EDGE`, verified with `FRAMEBUFFER_COMPLETE`, and resized automatically on window resize). On `PushLayer`:
  - Layers with `alpha <= 0.0` or an empty `composite_rect = transform.transform_rect_aabb(bounds).intersect(screen_clip)` are marked skipped without binding an FBO, setting empty clip bounds so descendants emit no pixels.
  - Active layers flush pending vertices to the current target, bind the next pooled FBO, clear it to transparent `(0, 0, 0, 0)`, and restrict clipping to `composite_rect` (and `local_clip` when transformed).
  - On `PopLayer`, pending layer vertices flush into the offscreen FBO, the parent FBO (or default framebuffer `None`) is rebound, and the offscreen texture is bound temporarily on `TEXTURE1` and composited over `composite_rect` as one textured quad (`mode = 0.1`, premultiplied `texture(image_atlas, v_uv) * v_color.a`) before restoring `image_texture` on `TEXTURE1`. Unbalanced layer stacks or depth overflow (`>= 16`) return an error.
- **Canvas 2D offscreen stack (`codes/apps/bhoo-web/site/canvas-painter.js`):** Web packet tags `22` (`PushLayer`) and `23` (`PopLayer`) compute the transformed device-pixel AABB clamped to the canvas, allocate an `OffscreenCanvas` (falling back to `document.createElement('canvas')`), copy the active transform shifted by `(-x0, -y0)` with a local clip rect, render descendants into the offscreen 2D context, and composite the buffer back onto the parent context with `ctx.setTransform(1, 0, 0, 1, 0, 0)` and `ctx.globalAlpha *= layer.alpha`. Zero-alpha or empty-area layers clip to `(0, 0, 0, 0)` and skip allocation; unbalanced layer stacks throw an error.
- **Limits:** GLES FBO pool attachments are full-viewport `RGBA8` textures (bounded by `MAX_LAYER_DEPTH = 16`), not tight-cropped scratch allocations; Canvas 2D allocates a per-layer offscreen surface on demand rather than pooling across frames.

## Ownership and safety

GL calls are private to the renderer and execute on the thread with the current EGL context. The runner owns the renderer and controls its destruction order. Normal renderer destruction deletes all pooled offscreen FBOs and layer textures, the glyph and image atlas textures, vertex/index buffers, vertex array, and shader program while the context is still current.

Shader compilation and linking failures include driver logs. Frame rendering checks the GL error state. Recovery from context loss and complete fault-injection coverage of partial initialization remain future work. Treat unsafe GL and native-handle boundaries as explicit review points.

## Allocation strategy

`Ui::frame_into` and the renderer retain command/vertex capacity. Text commands now use shared `Arc<str>` labels with a bounded text cache. Cache hits do not copy string contents; new/dynamic labels and glyph misses may allocate. GPU vertex/index buffers and pooled GLES layer FBOs grow on demand instead of being recreated for every upload. Driver costs are not covered by the core allocation test.

Profile these costs before introducing arenas or text handles. Any reuse scheme must keep command text valid until rendering finishes and invalidate font/scale-dependent cache entries correctly.

## Shared ink, vector and loading commands

`DrawCommand::Ripple` carries the control boundary, corner radius, moving ink center, radial extent and color. GLES combines rounded-rectangle and radial coverage in one existing-format quad; Canvas uses a clipped gradient with the same stops. No ripple texture or expanded per-vertex layout is required.

`DrawCommand::Arc` and `Line` implement rounded circular progress and rotating disclosure geometry. GLES computes distance to the arc/segment including end caps; Canvas strokes the matching primitive. CPU rectangular clipping continues to preserve local shader coordinates.

`DrawCommand::Loading` identifies the generated matched morph, progress and rotation. Canvas receives streamed transformed cubic coordinates. GLES uses 12 subdivisions per cubic and a physical-pixel coverage fringe, with stack-based triangle clipping. The common unclipped path packs adjacent fill triangles and fringe strips directly into indexed quads, halving vertex uploads relative to degenerate triangle quads. CPU point/vertex buffers and GPU capacities are retained. This specialized shape fill is tested for the star-shaped kernel of every generated morph, not offered as an arbitrary path tessellator.

See [effects implementation and provenance](effects-implementation.md) for timing, upstream sources, geometry generation, reduced motion and renderer validation boundaries.

## Web command extensions and comparison profiles

The version-one packet (`codes/apps/bhoo-web/src/lib.rs` -> `codes/apps/bhoo-web/site/canvas-painter.js`) includes these tagged records beyond basic clear/rect/text/clip:

- Tag `9` styled text: rectangle, RGBA, float size, packed `u32` flags (`bits 0..15` = `weight`, `bits 16..23` = `align`, `bit 24` = `italic`, `bits 25..26` = `FontFamily` `0 = Sans`, `1 = Serif`, `2 = Monospace`, decoded by `canvas-painter.js` into CSS `Arial, sans-serif` / `Georgia, serif` / `monospace`), padding, letter spacing, and UTF-8 string.
- Tag `10` shadow: rectangle, radius, offset, blur, spread, RGBA.
- Tags `5` / `11` soft/solid ripple, tags `7` / `12` rounded/butt-cap arc, tag `8` line, tag `15` static shape, tag `16` image, tag `17` polyline, tag `18` ellipse, tag `19` outline.
- Tags `13` / `14` `PushScaleY` / `PopScaleY`, tags `20` / `21` `PushTransform` (`origin`, `translate`, `scale`, `rotation_rad` as 7 floats) / `PopTransform`, and tags `22` / `23` `PushLayer` (bounds rect + `alpha`) / `PopLayer`.

Both renderers handle the corresponding core variants. Canvas shadows use native shadow rasterization; GLES `DrawCommand::Shadow` continues to use a bounded 8-layer approximation without an offscreen blur pass (even though `PushLayer`/`PopLayer` now uses a pooled FBO), so native shadow equality is not certified.

ASCII normal/medium advances are supplied once per browser engine initialization and stored in a fixed 760-byte table. Commands and semantic buffers retain capacity. Comparison palette bridging uses twenty roles and settings bit 32 selects Angular ink; neither requires a second UI framework in the Rust renderer. Subtree scale transforms affect draw commands, semantic bounds and inverse pointer coordinates together.
