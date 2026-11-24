use super::{Renderer, Result};
use crate::{FontFamily, TextStyle};
use harfrust::{Buffer, Direction, Font as HbFont, ShapeOptions, ShaperFont, shape};
use skrifa::{
    FontRef, MetadataProvider,
    instance::{LocationRef, Size},
    outline::{DrawSettings, HintingInstance, HintingMode, OutlinePen},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use unicode_bidi::BidiInfo;
use zeno::{Command as ZenoCmd, Mask, Origin, Point, Scratch, Vector};

#[derive(Clone, Debug)]
pub(super) struct ShapedGlyph {
    pub(super) font_id: u16,
    pub(super) glyph_id: u32,
    pub(super) x_advance: f32, // physical pixels at px_size
    pub(super) x_offset: f32,  // physical pixels at px_size
    pub(super) y_offset: f32,  // physical pixels at px_size (Y-up from baseline)
    pub(super) synthetic_italic: bool,
    pub(super) synthetic_bold: bool,
}

#[derive(Clone, Debug)]
pub(super) struct ShapedLine {
    pub(super) glyphs: Vec<ShapedGlyph>,
    pub(super) total_advance_px: f32,
    pub(super) baseline_offset_px: f32, // (ascent + descent) * 0.5 in physical px
}

pub(super) struct RasterizedGlyph {
    pub(super) left: i32,
    pub(super) top: i32,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) bitmap: Vec<u8>,
}

struct ZenoPen {
    cmds: Vec<ZenoCmd>,
}

impl OutlinePen for ZenoPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.cmds.push(ZenoCmd::MoveTo(Point::new(x, -y)));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.cmds.push(ZenoCmd::LineTo(Point::new(x, -y)));
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.cmds
            .push(ZenoCmd::QuadTo(Point::new(cx0, -cy0), Point::new(x, -y)));
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.cmds.push(ZenoCmd::CurveTo(
            Point::new(cx0, -cy0),
            Point::new(cx1, -cy1),
            Point::new(x, -y),
        ));
    }
    fn close(&mut self) {
        self.cmds.push(ZenoCmd::Close);
    }
}

pub(super) struct FontFace {
    data: Arc<Vec<u8>>,
    hb_font: HbFont,
    upem: f32,
    ascent_norm: f32,
    descent_norm: f32,
}

impl FontFace {
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        let data = Arc::new(bytes);
        let font_ref = FontRef::from_index(data.as_slice(), 0).ok()?;
        let shared_blob: Arc<dyn AsRef<[u8]> + Send + Sync> = data.clone();
        let hb_font = HbFont::new(shared_blob, 0)?;
        let upem = (hb_font.units_per_em() as f32).max(1.0);
        let m = font_ref.metrics(Size::unscaled(), LocationRef::default());
        Some(Self {
            data,
            hb_font,
            upem,
            ascent_norm: m.ascent / upem,
            descent_norm: m.descent / upem,
        })
    }

    pub(super) fn has_char(&self, ch: char) -> bool {
        self.hb_font.charmap().map_unicode(ch).is_some()
    }

    pub(super) fn baseline_offset_px(&self, px_size: u16) -> f32 {
        (self.ascent_norm + self.descent_norm) * 0.5 * px_size as f32
    }

    pub(super) fn rasterize_glyph(
        &self,
        glyph_id: u32,
        px_size: u16,
        sub_bin: u8,
        synthetic_bold: bool,
        scratch: &mut Scratch,
    ) -> Option<RasterizedGlyph> {
        let font_ref = FontRef::from_index(self.data.as_slice(), 0).ok()?;
        let outlines = font_ref.outline_glyphs();
        let glyph = outlines.get(skrifa::GlyphId::new(glyph_id))?;
        let size = Size::new(px_size as f32);

        let mut pen = ZenoPen {
            cmds: Vec::with_capacity(32),
        };

        // FreeType/Android-style smooth vertical-only hinting (preserves linear horizontal metrics
        // so 4x horizontal subpixel positioning stays completely jitter-free!).
        let drawn = if px_size <= 96 {
            let mode = HintingMode::Smooth {
                lcd_subpixel: None,
                preserve_linear_metrics: true,
            };
            if let Ok(hinter) =
                HintingInstance::new(&outlines, size, LocationRef::default(), mode)
            {
                glyph.draw(DrawSettings::hinted(&hinter, false), &mut pen).is_ok()
            } else {
                glyph
                    .draw(DrawSettings::unhinted(size, LocationRef::default()), &mut pen)
                    .is_ok()
            }
        } else {
            glyph
                .draw(DrawSettings::unhinted(size, LocationRef::default()), &mut pen)
                .is_ok()
        };

        if !drawn || pen.cmds.is_empty() {
            return Some(RasterizedGlyph {
                left: 0,
                top: 0,
                width: 0,
                height: 0,
                bitmap: Vec::new(),
            });
        }

        let sub_x = (sub_bin & 3) as f32 * 0.25;
        let (bitmap, placement) = Mask::with_scratch(&pen.cmds, scratch)
            .origin(Origin::TopLeft)
            .offset(Vector::new(sub_x, 0.0))
            .render();

        let (width, height, bitmap) = if synthetic_bold && placement.width > 0 && placement.height > 0 {
            dilate_bold_bitmap(
                placement.width as usize,
                placement.height as usize,
                bitmap,
                px_size,
            )
        } else {
            (placement.width as usize, placement.height as usize, bitmap)
        };

        Some(RasterizedGlyph {
            left: placement.left,
            top: placement.top,
            width,
            height,
            bitmap,
        })
    }
}

#[derive(Default)]
pub(super) struct FamilyFaces {
    light: Option<u16>,
    regular: Option<u16>,
    medium: Option<u16>,
    bold: Option<u16>,
    black: Option<u16>,
    italic: Option<u16>,
    bold_italic: Option<u16>,
}

impl FamilyFaces {
    pub(super) fn select(&self, weight_bucket: u8, italic: bool) -> Option<(u16, bool, bool)> {
        if italic && weight_bucket >= 3 {
            if let Some(f) = self.bold_italic {
                return Some((f, false, false));
            }
            if let Some(f) = self.bold {
                return Some((f, true, false));
            }
            if let Some(f) = self.italic {
                return Some((f, false, true));
            }
            return self.regular.map(|f| (f, true, true));
        }
        if italic {
            if let Some(f) = self.italic {
                return Some((f, false, false));
            }
            return self.regular.map(|f| (f, true, false));
        }
        if weight_bucket >= 4 {
            if let Some(f) = self.black {
                return Some((f, false, false));
            }
            if let Some(f) = self.bold {
                return Some((f, false, false));
            }
            return self.regular.map(|f| (f, false, true));
        }
        if weight_bucket >= 3 {
            if let Some(f) = self.bold {
                return Some((f, false, false));
            }
            if let Some(f) = self.medium {
                return Some((f, false, true));
            }
            return self.regular.map(|f| (f, false, true));
        }
        if weight_bucket == 2
            && let Some(f) = self.medium
        {
            return Some((f, false, false));
        }
        if weight_bucket == 0
            && let Some(f) = self.light
        {
            return Some((f, false, false));
        }
        self.regular.map(|f| (f, false, false))
    }
}

pub(super) struct FontSet {
    faces: Vec<FontFace>,
    sans: FamilyFaces,
    serif: FamilyFaces,
    mono: FamilyFaces,
    fallbacks: Vec<u16>,
}

/// Discover system font directories across NixOS (`/etc/fonts/conf.d/00-nixos-cache.conf`,
/// `/etc/fonts/fonts.conf`), `BHOO_FONT_DIRS`, and standard FHS font roots.
pub(crate) fn discover_system_font_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut push_dir = |p: PathBuf| {
        if p.is_dir() && !dirs.contains(&p) {
            dirs.push(p);
        }
    };
    let push_font_root = |root: &Path, push: &mut dyn FnMut(PathBuf)| {
        push(root.to_path_buf());
        let sf = root.join("share/fonts");
        if sf.is_dir() {
            push(sf.clone());
            if let Ok(entries) = std::fs::read_dir(&sf) {
                for entry in entries.flatten() {
                    let sub = entry.path();
                    if sub.is_dir() {
                        push(sub);
                    }
                }
            }
        }
    };

    if let Some(env_dirs) = std::env::var_os("BHOO_FONT_DIRS") {
        for part in std::env::split_paths(&env_dirs) {
            push_font_root(&part, &mut push_dir);
        }
    }

    for cfg in ["/etc/fonts/conf.d/00-nixos-cache.conf", "/etc/fonts/fonts.conf"] {
        if let Ok(text) = std::fs::read_to_string(cfg) {
            let mut rest = text.as_str();
            while let Some(start) = rest.find("<dir>") {
                let after = &rest[start + 5..];
                let Some(end) = after.find("</dir>") else {
                    break;
                };
                let raw = after[..end].trim();
                if raw.starts_with('/') {
                    push_font_root(Path::new(raw), &mut push_dir);
                }
                rest = &after[end + 6..];
            }
        }
    }

    for dir in [
        "/run/current-system/sw/share/fonts/truetype",
        "/run/current-system/sw/share/fonts/shabnam-fonts",
        "/run/current-system/sw/share/fonts/Adwaita",
        "/run/current-system/sw/share/fonts/noto",
        "/run/current-system/sw/share/fonts",
        "/usr/share/fonts/truetype/roboto",
        "/usr/share/fonts/roboto",
        "/usr/share/fonts/Adwaita",
        "/usr/share/fonts/truetype/dejavu",
        "/usr/share/fonts/TTF",
        "/usr/share/fonts/truetype/liberation",
        "/usr/share/fonts/liberation",
        "/usr/share/fonts/noto",
        "/usr/share/fonts/truetype/noto",
        "/system/fonts",
        "/var/lib/waydroid/rootfs/system/fonts",
        "/var/lib/waydroid/rootfs/system/product/fonts",
    ] {
        push_dir(PathBuf::from(dir));
    }
    dirs
}

/// Locate the best available primary UI font (preferring Roboto, then Adwaita/DejaVu/Liberation/Noto).
#[cfg_attr(not(feature = "wayland"), allow(dead_code))]
pub(crate) fn find_default_primary_font() -> Option<PathBuf> {
    let dirs = discover_system_font_dirs();
    for name in [
        "Roboto-Regular.ttf",
        "RobotoStatic-Regular.ttf",
        "AdwaitaSans-Regular.ttf",
        "DejaVuSans.ttf",
        "LiberationSans-Regular.ttf",
        "NotoSans-Regular.ttf",
    ] {
        for dir in &dirs {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

impl FontSet {
    pub(super) fn load(primary_path: &Path) -> Result<Self> {
        let bytes = std::fs::read(primary_path)
            .map_err(|e| format!("Font {}: {e}", primary_path.display()))?;
        let primary = FontFace::from_bytes(bytes).ok_or("Invalid primary font")?;

        let mut faces: Vec<FontFace> = vec![primary];
        let mut dirs: Vec<PathBuf> = Vec::new();
        if let Some(parent) = primary_path.parent()
            && !parent.as_os_str().is_empty()
        {
            dirs.push(parent.to_path_buf());
        }
        for p in discover_system_font_dirs() {
            if !dirs.contains(&p) {
                dirs.push(p);
            }
        }

        let sibling_dir = primary_path.parent();
        let stem = primary_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let base_stem = stem
            .strip_suffix("-Regular")
            .or_else(|| stem.strip_suffix("Regular"))
            .unwrap_or(stem);

        let try_add = |faces: &mut Vec<FontFace>, path: &Path| -> Option<u16> {
            if !path.is_file() {
                return None;
            }
            let bytes = std::fs::read(path).ok()?;
            let id = faces.len() as u16;
            let face = FontFace::from_bytes(bytes)?;
            faces.push(face);
            Some(id)
        };

        let load_sibling = |faces: &mut Vec<FontFace>, suffixes: &[&str]| -> Option<u16> {
            let dir = sibling_dir?;
            if base_stem.is_empty() {
                return None;
            }
            for suffix in suffixes {
                for ext in ["ttf", "otf"] {
                    let candidate = dir.join(format!("{base_stem}{suffix}.{ext}"));
                    if candidate != primary_path
                        && let Some(id) = try_add(faces, &candidate)
                    {
                        return Some(id);
                    }
                }
            }
            None
        };

        let load_from_dirs = |faces: &mut Vec<FontFace>, names: &[&str]| -> Option<u16> {
            for dir in &dirs {
                for name in names {
                    let candidate = dir.join(name);
                    if let Some(id) = try_add(faces, &candidate) {
                        return Some(id);
                    }
                }
            }
            None
        };

        let sans_light = load_sibling(&mut faces, &["-Light", "Light"]).or_else(|| {
            load_from_dirs(&mut faces, &["RobotoStatic-Light.ttf", "Roboto-Light.ttf", "NotoSans-Light.ttf"])
        });
        let sans_medium = load_sibling(&mut faces, &["-Medium", "Medium"]).or_else(|| {
            load_from_dirs(
                &mut faces,
                &[
                    "RobotoStatic-Medium.ttf",
                    "Roboto-Medium.ttf",
                    "Lato-Medium.ttf",
                    "NotoSans-Medium.ttf",
                ],
            )
        });
        let sans_bold = load_sibling(&mut faces, &["-Bold", "Bold"]).or_else(|| {
            load_from_dirs(
                &mut faces,
                &[
                    "RobotoStatic-Bold.ttf",
                    "Roboto-Bold.ttf",
                    "DejaVuSans-Bold.ttf",
                    "LiberationSans-Bold.ttf",
                    "NotoSans-Bold.ttf",
                    "SourceSansPro-Bold.ttf",
                    "Lato-Bold.ttf",
                ],
            )
        });
        let sans_black = load_sibling(&mut faces, &["-Black", "Black"]).or_else(|| {
            load_from_dirs(&mut faces, &["RobotoStatic-Black.ttf", "Roboto-Black.ttf"])
        });
        let sans_italic = load_sibling(&mut faces, &["-Italic", "-Oblique", "Italic", "Oblique"]).or_else(|| {
            load_from_dirs(
                &mut faces,
                &[
                    "RobotoStatic-Italic.ttf",
                    "Roboto-Italic.ttf",
                    "AdwaitaSans-Italic.ttf",
                    "DejaVuSans-Oblique.ttf",
                    "LiberationSans-Italic.ttf",
                    "NotoSans-Italic.ttf",
                    "SourceSansPro-Italic.ttf",
                    "Lato-Italic.ttf",
                ],
            )
        });
        let sans_bold_italic = load_sibling(
            &mut faces,
            &["-BoldItalic", "-BoldOblique", "BoldItalic", "BoldOblique"],
        )
        .or_else(|| {
            load_from_dirs(
                &mut faces,
                &[
                    "RobotoStatic-BoldItalic.ttf",
                    "Roboto-BoldItalic.ttf",
                    "DejaVuSans-BoldOblique.ttf",
                    "LiberationSans-BoldItalic.ttf",
                    "NotoSans-BoldItalic.ttf",
                    "SourceSansPro-BoldItalic.ttf",
                    "Lato-BoldItalic.ttf",
                ],
            )
        });

        let sans = FamilyFaces {
            light: sans_light,
            regular: Some(0),
            medium: sans_medium,
            bold: sans_bold,
            black: sans_black,
            italic: sans_italic,
            bold_italic: sans_bold_italic,
        };

        let serif = FamilyFaces {
            light: None,
            regular: load_from_dirs(
                &mut faces,
                &[
                    "NotoSerif-Regular.ttf",
                    "DejaVuSerif.ttf",
                    "LiberationSerif-Regular.ttf",
                    "DroidSerif-Regular.ttf",
                ],
            ),
            medium: None,
            bold: load_from_dirs(
                &mut faces,
                &[
                    "NotoSerif-Bold.ttf",
                    "DejaVuSerif-Bold.ttf",
                    "LiberationSerif-Bold.ttf",
                    "DroidSerif-Bold.ttf",
                ],
            ),
            black: None,
            italic: load_from_dirs(
                &mut faces,
                &[
                    "NotoSerif-Italic.ttf",
                    "DejaVuSerif-Italic.ttf",
                    "LiberationSerif-Italic.ttf",
                    "DroidSerif-Italic.ttf",
                ],
            ),
            bold_italic: load_from_dirs(
                &mut faces,
                &[
                    "NotoSerif-BoldItalic.ttf",
                    "DejaVuSerif-BoldItalic.ttf",
                    "LiberationSerif-BoldItalic.ttf",
                    "DroidSerif-BoldItalic.ttf",
                ],
            ),
        };

        let mono = FamilyFaces {
            light: None,
            regular: load_from_dirs(
                &mut faces,
                &[
                    "AdwaitaMono-Regular.ttf",
                    "DroidSansMono.ttf",
                    "DejaVuSansMono.ttf",
                    "LiberationMono-Regular.ttf",
                    "NotoSansMono-Regular.ttf",
                ],
            ),
            medium: None,
            bold: load_from_dirs(
                &mut faces,
                &[
                    "AdwaitaMono-Bold.ttf",
                    "DejaVuSansMono-Bold.ttf",
                    "LiberationMono-Bold.ttf",
                    "NotoSansMono-Bold.ttf",
                ],
            ),
            black: None,
            italic: load_from_dirs(
                &mut faces,
                &[
                    "AdwaitaMono-Italic.ttf",
                    "DejaVuSansMono-Oblique.ttf",
                    "LiberationMono-Italic.ttf",
                    "NotoSansMono-Italic.ttf",
                ],
            ),
            bold_italic: load_from_dirs(
                &mut faces,
                &[
                    "AdwaitaMono-BoldItalic.ttf",
                    "DejaVuSansMono-BoldOblique.ttf",
                    "LiberationMono-BoldItalic.ttf",
                    "NotoSansMono-BoldItalic.ttf",
                ],
            ),
        };

        // Fallback faces for Persian/Arabic scripts & UI symbols missing in primary Latin face.
        let mut fallbacks = Vec::new();
        for candidates in [
            &[
                "Shabnam-FD.ttf",
                "Shabnam.ttf",
                "Vazirmatn-Regular.ttf",
                "NotoSansArabic-Regular.ttf",
            ][..],
            &[
                "DejaVuSans.ttf",
                "NotoSansSymbols2-Regular.ttf",
                "AdwaitaSans-Regular.ttf",
            ][..],
        ] {
            if let Some(id) = load_from_dirs(&mut faces, candidates) {
                fallbacks.push(id);
            }
        }

        Ok(Self {
            faces,
            sans,
            serif,
            mono,
            fallbacks,
        })
    }

    pub(super) fn face(&self, id: u16) -> &FontFace {
        &self.faces[id as usize]
    }

    pub(super) fn select(&self, family: u8, weight_bucket: u8, italic: bool) -> (u16, bool, bool) {
        let faces = match family {
            1 => &self.serif,
            2 => &self.mono,
            _ => &self.sans,
        };
        faces
            .select(weight_bucket, italic)
            .or_else(|| self.sans.select(weight_bucket, italic))
            .unwrap_or((0, italic, weight_bucket >= 3))
    }

    fn resolve_face_for_char(
        &self,
        ch: char,
        primary_id: u16,
        syn_i: bool,
        syn_b: bool,
        italic: bool,
        weight_bucket: u8,
    ) -> (u16, bool, bool) {
        if self.faces[primary_id as usize].has_char(ch) {
            return (primary_id, syn_i, syn_b);
        }
        for &fb_id in &self.fallbacks {
            if self.faces[fb_id as usize].has_char(ch) {
                return (fb_id, italic, weight_bucket >= 3);
            }
        }
        (primary_id, syn_i, syn_b)
    }

    /// Shape a single line of UTF-8 text using `unicode-bidi` (for Arabic/Persian RTL + mixed BiDi)
    /// and `harfrust` (full HarfBuzz OpenType GSUB/GPOS shaping, kerning, ligatures, contextual joining).
    pub(super) fn shape_line(
        &self,
        text: &str,
        family: u8,
        weight_bucket: u8,
        italic: bool,
        px_size: u16,
        shaper_buf: &mut Buffer,
    ) -> ShapedLine {
        let (primary_id, syn_i, syn_b) = self.select(family, weight_bucket, italic);
        let baseline_offset_px = self.faces[primary_id as usize].baseline_offset_px(px_size);
        let mut glyphs = Vec::with_capacity(text.len());
        let mut total_advance_px = 0.0f32;

        // Fast-path pure ASCII LTR strings directly into HarfRust without BiDi allocation overhead
        if text.is_ascii() {
            let mut filtered = String::new();
            let src: &str = if text.bytes().any(|b| b < 0x20 || b == 0x7f) {
                filtered.extend(text.chars().filter(|c| is_visible_text_char(*c)));
                &filtered
            } else {
                text
            };
            self.shape_subrun(
                src,
                false,
                primary_id,
                syn_i,
                syn_b,
                px_size,
                shaper_buf,
                &mut glyphs,
                &mut total_advance_px,
            );
            return ShapedLine {
                glyphs,
                total_advance_px,
                baseline_offset_px,
            };
        }

        // Full Unicode BiDi visual run decomposition for Persian/Arabic & mixed text
        let bidi = BidiInfo::new(text, None);
        if bidi.paragraphs.is_empty() {
            return ShapedLine {
                glyphs,
                total_advance_px,
                baseline_offset_px,
            };
        }
        for para in &bidi.paragraphs {
            let line = para.range.clone();
            let (levels, runs) = bidi.visual_runs(para, line);
            for run in runs {
                let is_rtl = levels[run.start].is_rtl();
                let slice = &text[run];
                // Split the BiDi run into contiguous font-face sub-runs
                let mut subruns: Vec<(u16, bool, bool, usize, usize)> = Vec::new();
                for (byte_idx, ch) in slice.char_indices() {
                    if !is_visible_text_char(ch) {
                        continue;
                    }
                    let end = byte_idx + ch.len_utf8();
                    let (fid, si, sb) = if ch.is_whitespace() && !subruns.is_empty() {
                        let last = subruns.last().unwrap();
                        if self.faces[last.0 as usize].has_char(ch) {
                            (last.0, last.1, last.2)
                        } else {
                            self.resolve_face_for_char(ch, primary_id, syn_i, syn_b, italic, weight_bucket)
                        }
                    } else {
                        self.resolve_face_for_char(ch, primary_id, syn_i, syn_b, italic, weight_bucket)
                    };
                    if let Some(last) = subruns.last_mut()
                        && last.0 == fid
                        && last.1 == si
                        && last.2 == sb
                        && last.4 == byte_idx
                    {
                        last.4 = end;
                    } else {
                        subruns.push((fid, si, sb, byte_idx, end));
                    }
                }
                if is_rtl {
                    for &(fid, si, sb, s, e) in subruns.iter().rev() {
                        self.shape_subrun(
                            &slice[s..e],
                            true,
                            fid,
                            si,
                            sb,
                            px_size,
                            shaper_buf,
                            &mut glyphs,
                            &mut total_advance_px,
                        );
                    }
                } else {
                    for &(fid, si, sb, s, e) in &subruns {
                        self.shape_subrun(
                            &slice[s..e],
                            false,
                            fid,
                            si,
                            sb,
                            px_size,
                            shaper_buf,
                            &mut glyphs,
                            &mut total_advance_px,
                        );
                    }
                }
            }
        }

        ShapedLine {
            glyphs,
            total_advance_px,
            baseline_offset_px,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn shape_subrun(
        &self,
        slice: &str,
        is_rtl: bool,
        font_id: u16,
        synthetic_italic: bool,
        synthetic_bold: bool,
        px_size: u16,
        shaper_buf: &mut Buffer,
        out: &mut Vec<ShapedGlyph>,
        total_advance_px: &mut f32,
    ) {
        if slice.is_empty() {
            return;
        }
        let face = &self.faces[font_id as usize];
        shaper_buf.clear();
        shaper_buf.push_str(slice);
        shaper_buf.set_direction(if is_rtl {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        });
        shaper_buf.guess_segment_properties();
        let shaper_font = ShaperFont::new(&face.hb_font);
        if shape(&shaper_font, shaper_buf, ShapeOptions::new()).is_err() {
            return;
        }
        let unit_scale = px_size as f32 / face.upem;
        let infos = shaper_buf.glyph_infos();
        let positions = shaper_buf.glyph_positions();
        for (info, pos) in infos.iter().zip(positions.iter()) {
            let x_advance = pos.x_advance as f32 * unit_scale;
            let x_offset = pos.x_offset as f32 * unit_scale;
            let y_offset = pos.y_offset as f32 * unit_scale;
            *total_advance_px += x_advance;
            out.push(ShapedGlyph {
                font_id,
                glyph_id: info.glyph_id,
                x_advance,
                x_offset,
                y_offset,
                synthetic_italic,
                synthetic_bold,
            });
        }
    }
}

pub(super) fn family_index(family: FontFamily) -> u8 {
    match family {
        FontFamily::Sans => 0,
        FontFamily::Serif => 1,
        FontFamily::Monospace => 2,
    }
}

pub(super) fn weight_bucket(weight: u16) -> u8 {
    if weight >= 800 {
        4
    } else if weight >= 600 {
        3
    } else if weight >= 480 {
        2
    } else if weight <= 350 {
        0
    } else {
        1
    }
}

pub(super) fn text_glyph_params(style: TextStyle, scale: f32) -> (u8, u8, bool, u16, f32) {
    let target_px = (style.size * scale).max(1.0);
    let px_size = target_px.round().clamp(8.0, 256.0) as u16;
    let factor = target_px / px_size as f32;
    (
        family_index(style.family),
        weight_bucket(style.weight),
        style.italic,
        px_size,
        factor,
    )
}

pub(super) fn is_visible_text_char(c: char) -> bool {
    !c.is_control() && !matches!(c, '\u{00ad}' | '\u{200b}' | '\u{2060}' | '\u{feff}')
}

fn dilate_bold_bitmap(
    old_w: usize,
    h: usize,
    bitmap: Vec<u8>,
    px_size: u16,
) -> (usize, usize, Vec<u8>) {
    if old_w == 0 || h == 0 {
        return (old_w, h, bitmap);
    }
    let extra = ((px_size as f32) / 24.0).round().clamp(1.0, 3.0) as usize;
    let new_w = old_w + extra;
    let mut out = vec![0u8; new_w * h];
    for row in 0..h {
        let src = &bitmap[row * old_w..(row + 1) * old_w];
        let dst = &mut out[row * new_w..(row + 1) * new_w];
        for (col, slot) in dst.iter_mut().enumerate() {
            let mut best = 0u8;
            for dx in 0..=extra {
                if col >= dx && col - dx < old_w {
                    best = best.max(src[col - dx]);
                }
            }
            *slot = best;
        }
    }
    (new_w, h, out)
}

impl Renderer {
    pub fn text_metrics(&self) -> crate::TextMetrics {
        let mut metrics = crate::TextMetrics::default();
        let reg_id = self.fonts.sans.regular.unwrap_or(0);
        let med_id = self.fonts.sans.medium.unwrap_or(reg_id);
        let reg = self.fonts.face(reg_id);
        let med = self.fonts.face(med_id);

        let measure = |face: &FontFace, ch: char| -> f32 {
            let Ok(font_ref) = FontRef::from_index(face.data.as_slice(), 0) else {
                return 0.5;
            };
            let Some(gid) = face.hb_font.charmap().map_unicode(ch) else {
                return 0.5;
            };
            let gm = font_ref.glyph_metrics(Size::unscaled(), LocationRef::default());
            gm.advance_width(skrifa::GlyphId::new(gid.to_u32()))
                .unwrap_or(face.upem * 0.5)
                / face.upem
        };

        for code in 32..127 {
            let ch = char::from_u32(code).unwrap();
            metrics.normal[code as usize - 32] = measure(reg, ch);
            metrics.medium[code as usize - 32] = measure(med, ch);
        }
        metrics.normal_baseline_offset = (reg.ascent_norm + reg.descent_norm) * 0.5;
        metrics.medium_baseline_offset = (med.ascent_norm + med.descent_norm) * 0.5;
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harfrust_shapes_latin_and_bidi_and_skrifa_rasterizes_4_subpixel_phases() {
        let Some(font_path) = find_default_primary_font() else {
            return;
        };
        let fonts = FontSet::load(&font_path).expect("load system font");
        let mut buf = Buffer::new();
        let shaped = fonts.shape_line("AVATAR 12:30", 0, 1, false, 36, &mut buf);
        assert!(shaped.glyphs.len() >= 10);
        assert!(shaped.total_advance_px > 40.0);

        let mut scratch = Scratch::new();
        let first = &shaped.glyphs[0];
        for bin in 0..4u8 {
            let r = fonts
                .face(first.font_id)
                .rasterize_glyph(first.glyph_id, 36, bin, false, &mut scratch)
                .expect("rasterize hinted glyph");
            assert!(r.width > 0 && r.height > 0);
            assert!(r.bitmap.iter().any(|&a| a > 200));
        }

        // Verify BiDi + HarfRust handles mixed Latin + Persian/Arabic cleanly without panic
        let bidi = fonts.shape_line("12:30 سلام دنیا", 0, 1, false, 36, &mut buf);
        assert!(!bidi.glyphs.is_empty());
        assert!(bidi.total_advance_px > 20.0);
    }
}
