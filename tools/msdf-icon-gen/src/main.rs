//! Generates resolution-independent 48x48 MSDF (Multi-Channel Signed Distance Field)
//! data for all 68 bundled Google Material Symbols Outlined icons in `libbhoo`.
//!
//! Input:  `codes/libs/ui/libbhoo/assets/material-system/<name>_24px.svg`
//! Output: `codes/libs/ui/libbhoo/assets/material-system/icons.msdf`
//!
//! Binary format:
//! - Bytes 0..4:   magic `b"MSDF"`
//! - Bytes 4..6:   `count: u16` LE (68)
//! - Bytes 6..8:   `size: u16` LE (48)
//! - Bytes 8..12:  `px_range: f32` LE (4.0)
//! - Next `(count + 1) * 4` bytes: `u32` LE byte offsets into payload section
//! - Payload section: for each icon, RLE8-encoded planar `[R(size*size), G(size*size), B(size*size)]`
//!   using `libbhoo::icons::material_system::decode_rle8` format.

use fdsm::bezier::scanline::FillRule;
use fdsm::bezier::{Point, Segment};
use fdsm::correct_error::{correct_error_msdf, ErrorCorrectionConfig};
use fdsm::generate::generate_msdf;
use fdsm::render::correct_sign_msdf;
use fdsm::shape::{Contour, Shape};
use fdsm::transform::Transform;
use image::{ImageBuffer, Rgb};
use nalgebra::{Affine2, Matrix3};
use std::fs;
use std::path::PathBuf;
use svgtypes::{PathParser, PathSegment};

const SIZE: u32 = 48;
const PX_RANGE: f64 = 4.0;
/// Corner angle threshold (~3 radians, standard Chlumsky msdfgen setting).
const SIN_ALPHA: f64 = 0.03;

/// Exact 68-icon order matching `SystemIcon::mask_index` in `material_system.rs`.
const ICONS: &[&str] = &[
    "wifi",
    "wifi_off",
    "bluetooth",
    "flashlight_on",
    "alarm",
    "airplanemode_active",
    "home",
    "screen_rotation",
    "battery_saver",
    "dark_mode",
    "do_not_disturb_on",
    "cast",
    "fiber_manual_record",
    "bedtime",
    "videocam",
    "mic",
    "aod",
    "wifi_tethering",
    "location_on",
    "nfc",
    "invert_colors",
    "light_mode",
    "brightness_auto",
    "power_settings_new",
    "restart_alt",
    "screenshot",
    "emergency",
    "vibration",
    "notifications_off",
    "notifications",
    "music_note",
    "info",
    "splitscreen",
    "picture_in_picture",
    "open_in_full",
    "close",
    "settings",
    "widgets",
    "palette",
    "search",
    "arrow_back",
    "chevron_right",
    "chevron_left",
    "expand_more",
    "expand_less",
    "edit",
    "delete",
    "battery_full",
    "storage",
    "volume_up",
    "accessibility_new",
    "lock",
    "privacy_tip",
    "person",
    "smartphone",
    "apps",
    "usb",
    "history",
    "more_vert",
    "visibility_off",
    "lock_open",
    "fingerprint",
    "tune",
    "notifications_active",
    "gesture",
    "check_circle",
    "devices",
    "language",
];

fn extract_path_d(svg: &str) -> Result<String, String> {
    let needle = " d=\"";
    let start = svg.find(needle).ok_or("missing path d attribute")? + needle.len();
    let rest = &svg[start..];
    let end = rest.find('"').ok_or("unterminated path d attribute")?;
    Ok(rest[..end].to_string())
}

fn push_line(contour: &mut Contour, a: Point, b: Point) {
    if (b - a).norm_squared() > 1e-10 {
        contour.segments.push(Segment::line(a, b));
    }
}

fn parse_svg_path_to_shape(d: &str) -> Result<Shape<Contour>, String> {
    let mut contours: Vec<Contour> = Vec::new();
    let mut current = Contour::default();
    let mut cur = Point::new(0.0, 0.0);
    let mut start = Point::new(0.0, 0.0);
    let mut last_quad_ctrl: Option<Point> = None;
    let mut last_cubic_ctrl: Option<Point> = None;
    let mut has_subpath = false;

    for seg in PathParser::from(d) {
        let seg = seg.map_err(|e| format!("SVG path parse error: {e}"))?;
        match seg {
            PathSegment::MoveTo { abs, x, y } => {
                if has_subpath && !current.segments.is_empty() {
                    push_line(&mut current, cur, start);
                    contours.push(std::mem::take(&mut current));
                }
                let p = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(cur.x + x, cur.y + y)
                };
                cur = p;
                start = p;
                has_subpath = true;
                last_quad_ctrl = None;
                last_cubic_ctrl = None;
            }
            PathSegment::LineTo { abs, x, y } => {
                let p = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(cur.x + x, cur.y + y)
                };
                push_line(&mut current, cur, p);
                cur = p;
                last_quad_ctrl = None;
                last_cubic_ctrl = None;
            }
            PathSegment::HorizontalLineTo { abs, x } => {
                let p = if abs {
                    Point::new(x, cur.y)
                } else {
                    Point::new(cur.x + x, cur.y)
                };
                push_line(&mut current, cur, p);
                cur = p;
                last_quad_ctrl = None;
                last_cubic_ctrl = None;
            }
            PathSegment::VerticalLineTo { abs, y } => {
                let p = if abs {
                    Point::new(cur.x, y)
                } else {
                    Point::new(cur.x, cur.y + y)
                };
                push_line(&mut current, cur, p);
                cur = p;
                last_quad_ctrl = None;
                last_cubic_ctrl = None;
            }
            PathSegment::Quadratic { abs, x1, y1, x, y } => {
                let c = if abs {
                    Point::new(x1, y1)
                } else {
                    Point::new(cur.x + x1, cur.y + y1)
                };
                let p = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(cur.x + x, cur.y + y)
                };
                if (c - cur).norm_squared() > 1e-10 || (p - c).norm_squared() > 1e-10 {
                    current.segments.push(Segment::quad(cur, c, p));
                }
                cur = p;
                last_quad_ctrl = Some(c);
                last_cubic_ctrl = None;
            }
            PathSegment::SmoothQuadratic { abs, x, y } => {
                let c = match last_quad_ctrl {
                    Some(prev_c) => Point::new(2.0 * cur.x - prev_c.x, 2.0 * cur.y - prev_c.y),
                    None => cur,
                };
                let p = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(cur.x + x, cur.y + y)
                };
                if (c - cur).norm_squared() > 1e-10 || (p - c).norm_squared() > 1e-10 {
                    current.segments.push(Segment::quad(cur, c, p));
                }
                cur = p;
                last_quad_ctrl = Some(c);
                last_cubic_ctrl = None;
            }
            PathSegment::CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let c1 = if abs {
                    Point::new(x1, y1)
                } else {
                    Point::new(cur.x + x1, cur.y + y1)
                };
                let c2 = if abs {
                    Point::new(x2, y2)
                } else {
                    Point::new(cur.x + x2, cur.y + y2)
                };
                let p = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(cur.x + x, cur.y + y)
                };
                current.segments.push(Segment::cubic(cur, c1, c2, p));
                cur = p;
                last_cubic_ctrl = Some(c2);
                last_quad_ctrl = None;
            }
            PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                let c1 = match last_cubic_ctrl {
                    Some(prev_c) => Point::new(2.0 * cur.x - prev_c.x, 2.0 * cur.y - prev_c.y),
                    None => cur,
                };
                let c2 = if abs {
                    Point::new(x2, y2)
                } else {
                    Point::new(cur.x + x2, cur.y + y2)
                };
                let p = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(cur.x + x, cur.y + y)
                };
                current.segments.push(Segment::cubic(cur, c1, c2, p));
                cur = p;
                last_cubic_ctrl = Some(c2);
                last_quad_ctrl = None;
            }
            PathSegment::ClosePath { .. } => {
                if has_subpath {
                    push_line(&mut current, cur, start);
                    if !current.segments.is_empty() {
                        contours.push(std::mem::take(&mut current));
                    }
                    cur = start;
                }
                last_quad_ctrl = None;
                last_cubic_ctrl = None;
            }
            PathSegment::EllipticalArc { .. } => {
                return Err("unexpected elliptical arc in Material Symbols SVG".into());
            }
        }
    }
    if !current.segments.is_empty() {
        push_line(&mut current, cur, start);
        contours.push(current);
    }
    Ok(Shape { contours })
}

/// Lossless RLE8 encoder matching `libbhoo::icons::material_system::decode_rle8`.
/// - `0x00..=0x7F`: run of `(b + 1)` zeros
/// - `0x80..=0xBF`: run of `(b - 0x80 + 1)` 255s
/// - `0xC0..=0xFF`: `(b - 0xC0 + 1)` raw literal bytes
fn encode_rle8(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() / 2);
    let mut i = 0;
    let n = data.len();
    while i < n {
        let b = data[i];
        if b == 0 {
            let mut run = 1usize;
            while i + run < n && data[i + run] == 0 && run < 128 {
                run += 1;
            }
            out.push((run - 1) as u8);
            i += run;
        } else if b == 255 {
            let mut run = 1usize;
            while i + run < n && data[i + run] == 255 && run < 64 {
                run += 1;
            }
            out.push(0x80 + (run - 1) as u8);
            i += run;
        } else {
            let start = i;
            i += 1;
            while i < n && (i - start) < 64 {
                // Stop literal run if we hit a run of >=2 zeros or 255s.
                if (data[i] == 0 || data[i] == 255) && i + 1 < n && data[i + 1] == data[i] {
                    break;
                }
                i += 1;
            }
            let count = i - start;
            out.push(0xC0 + (count - 1) as u8);
            out.extend_from_slice(&data[start..i]);
        }
    }
    out
}

fn median_u8(r: u8, g: u8, b: u8) -> u8 {
    r.min(g).max(r.max(g).min(b))
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let assets_dir = root.join("codes/libs/ui/libbhoo/assets/material-system");
    let out_path = assets_dir.join("icons.msdf");

    // Map SVG viewBox="0 -960 960 960" exactly onto MSDF texel centers [0.5 .. SIZE - 0.5]
    // so atlas UVs inset by 0.5 texel map quad edges [0..1] to exact SVG [0..960] without clipping.
    let scale = (SIZE as f64 - 1.0) / 960.0;
    let tx = 0.5;
    let ty = 0.5 + 960.0 * scale;
    let transform = Affine2::from_matrix_unchecked(Matrix3::new(
        scale, 0.0, tx,
        0.0, scale, ty,
        0.0, 0.0, 1.0,
    ));

    let plane_len = (SIZE * SIZE) as usize;
    let mut offsets: Vec<u32> = Vec::with_capacity(ICONS.len() + 1);
    let mut payload: Vec<u8> = Vec::new();

    for (idx, name) in ICONS.iter().enumerate() {
        let svg_path = assets_dir.join(format!("{name}_24px.svg"));
        let svg = fs::read_to_string(&svg_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", svg_path.display()));
        let d = extract_path_d(&svg)
            .unwrap_or_else(|e| panic!("Failed to extract path from {name}: {e}"));
        let mut shape = parse_svg_path_to_shape(&d)
            .unwrap_or_else(|e| panic!("Failed to parse {name}: {e}"));
        assert!(
            !shape.contours.is_empty(),
            "Icon {name} produced zero contours"
        );
        shape.transform(&transform);

        let colored = Shape::edge_coloring_simple(shape, SIN_ALPHA, 69441337420u64 ^ (idx as u64));
        let prepared = colored.prepare();

        let mut msdf_f32: ImageBuffer<Rgb<f32>, Vec<f32>> = ImageBuffer::new(SIZE, SIZE);
        generate_msdf(&prepared, PX_RANGE, &mut msdf_f32);
        correct_error_msdf(
            &mut msdf_f32,
            &colored,
            &prepared,
            PX_RANGE,
            &ErrorCorrectionConfig::default(),
        );
        correct_sign_msdf(&mut msdf_f32, &prepared, FillRule::Nonzero);

        let mut r_plane = vec![0u8; plane_len];
        let mut g_plane = vec![0u8; plane_len];
        let mut b_plane = vec![0u8; plane_len];
        let mut inside = 0usize;
        let mut outside = 0usize;

        for y in 0..SIZE {
            for x in 0..SIZE {
                let p = msdf_f32.get_pixel(x, y).0;
                let r = (p[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                let g = (p[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                let b = (p[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                let o = (y * SIZE + x) as usize;
                r_plane[o] = r;
                g_plane[o] = g;
                b_plane[o] = b;
                if median_u8(r, g, b) >= 128 {
                    inside += 1;
                } else {
                    outside += 1;
                }
            }
        }
        assert!(
            inside > 10 && outside > 10,
            "Icon {name} has degenerate MSDF coverage (inside={inside}, outside={outside})"
        );

        // Print 24x24 ASCII check for wifi & arrow_back to confirm orientation & clarity.
        if *name == "wifi" || *name == "arrow_back" {
            println!("=== Preview {name} (inside={inside}/{plane_len}) ===");
            for y in (0..SIZE).step_by(2) {
                let mut row = String::with_capacity((SIZE / 2) as usize);
                for x in (0..SIZE).step_by(2) {
                    let o = (y * SIZE + x) as usize;
                    let m = median_u8(r_plane[o], g_plane[o], b_plane[o]);
                    row.push(if m >= 160 {
                        '#'
                    } else if m >= 128 {
                        '+'
                    } else if m >= 96 {
                        '.'
                    } else {
                        ' '
                    });
                }
                println!("|{row}|");
            }
        }

        let mut planar = Vec::with_capacity(plane_len * 3);
        planar.extend_from_slice(&r_plane);
        planar.extend_from_slice(&g_plane);
        planar.extend_from_slice(&b_plane);
        let encoded = encode_rle8(&planar);

        offsets.push(payload.len() as u32);
        payload.extend_from_slice(&encoded);
    }
    offsets.push(payload.len() as u32);

    let mut blob = Vec::with_capacity(12 + offsets.len() * 4 + payload.len());
    blob.extend_from_slice(b"MSDF");
    blob.extend_from_slice(&(ICONS.len() as u16).to_le_bytes());
    blob.extend_from_slice(&(SIZE as u16).to_le_bytes());
    blob.extend_from_slice(&(PX_RANGE as f32).to_le_bytes());
    for off in &offsets {
        blob.extend_from_slice(&off.to_le_bytes());
    }
    blob.extend_from_slice(&payload);

    fs::write(&out_path, &blob)
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", out_path.display()));
    println!(
        "Generated {} ({} icons, {}x{}, px_range={}, total={} bytes, avg={} bytes/icon)",
        out_path.display(),
        ICONS.len(),
        SIZE,
        SIZE,
        PX_RANGE,
        blob.len(),
        blob.len() / ICONS.len()
    );
}
