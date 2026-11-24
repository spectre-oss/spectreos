//! High-resolution (`176x176` xxxhdpi) Android 13 / LineageOS 20 adaptive launcher app icons
//! embedded as lossless RGBA PNGs, plus Material You themed monochrome silhouette derivation
//! and `libbhoo::SystemIcon` bridge.
//!
//! Decoded lazily once into GPU-cached `bhoo::Image` instances with trilinear mipmaps.

use super::{AppId, Palette, SysIcon};
use bhoo::{Color, Frame, Image, ImageFit, Rect};
use chrono::Timelike;
use std::f32::consts::TAU;
use std::sync::{Mutex, OnceLock};

const APP_COUNT: usize = 18;

struct EmbeddedAppIcon {
    png_bytes: &'static [u8],
    bg_rgb: (u8, u8, u8),
}

const APP_ICONS: [EmbeddedAppIcon; APP_COUNT] = [
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/arcanechat.png"),
        bg_rgb: (123, 0, 198),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/browser.png"),
        bg_rgb: (69, 130, 230),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/calculator.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/calendar.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/camera.png"),
        bg_rgb: (45, 48, 53),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/clock.png"),
        bg_rgb: (42, 45, 51),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/contacts.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/deltachat.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/fdroid.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/files.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/gallery.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/gmail.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/messaging.png"),
        bg_rgb: (46, 139, 71),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/music.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/phone.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/recorder.png"),
        bg_rgb: (82, 23, 29),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/settings.png"),
        bg_rgb: (255, 255, 255),
    },
    EmbeddedAppIcon {
        png_bytes: include_bytes!("../assets/icons/telegram.png"),
        bg_rgb: (42, 161, 226),
    },
];

fn app_index(app: AppId) -> usize {
    match app {
        AppId::ArcaneChat => 0,
        AppId::Browser => 1,
        AppId::Calculator => 2,
        AppId::Calendar => 3,
        AppId::Camera => 4,
        AppId::Clock => 5,
        AppId::Contacts => 6,
        AppId::DeltaChat => 7,
        AppId::FDroid => 8,
        AppId::Files => 9,
        AppId::Gallery => 10,
        AppId::Gmail => 11,
        AppId::Messaging => 12,
        AppId::Music => 13,
        AppId::Phone => 14,
        AppId::Recorder => 15,
        AppId::Settings => 16,
        AppId::Telegram => 17,
    }
}

struct DecodedIcon {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    fg_mask: Vec<u8>,
}

fn decode_png_rgba(bytes: &[u8], bg_rgb: (u8, u8, u8)) -> DecodedIcon {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().expect("valid embedded PNG icon");
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("valid PNG frame");
    buf.truncate(info.buffer_size());

    let (w, h) = (info.width, info.height);
    let total = (w * h) as usize;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity(total * 4);
            for chunk in buf.chunks_exact(3) {
                out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            out
        }
        _ => panic!("unexpected PNG color type {:?}", info.color_type),
    };

    // Derive high-resolution anti-aliased foreground mask for Material You themed icons
    // from Euclidean RGB distance against the icon's adaptive backdrop `bg_rgb`.
    let mut fg_mask = vec![0u8; total];
    let (br, bg, bb) = (bg_rgb.0 as f32, bg_rgb.1 as f32, bg_rgb.2 as f32);
    let half_w = w as f32 * 0.5;
    let half_h = h as f32 * 0.5;
    let max_r = half_w.min(half_h) * 0.88;
    for y in 0..h {
        for x in 0..w {
            let pi = (y * w + x) as usize;
            let o = pi * 4;
            let a = rgba[o + 3];
            if a == 0 {
                continue;
            }
            let dx = x as f32 + 0.5 - half_w;
            let dy = y as f32 + 0.5 - half_h;
            if dx * dx + dy * dy > max_r * max_r {
                continue;
            }
            let dr = rgba[o] as f32 - br;
            let dg = rgba[o + 1] as f32 - bg;
            let db = rgba[o + 2] as f32 - bb;
            let dist = (dr * dr + dg * dg + db * db).sqrt();
            // Smooth ramp from 22..56 RGB distance preserves crisp anti-aliased vector edges.
            let cov = ((dist - 22.0) / 34.0).clamp(0.0, 1.0);
            fg_mask[pi] = ((cov * (a as f32 / 255.0)) * 255.0).round() as u8;
        }
    }

    DecodedIcon {
        width: w,
        height: h,
        rgba,
        fg_mask,
    }
}

fn decoded_icons() -> &'static [DecodedIcon; APP_COUNT] {
    static DECODED: OnceLock<[DecodedIcon; APP_COUNT]> = OnceLock::new();
    DECODED.get_or_init(|| {
        std::array::from_fn(|i| decode_png_rgba(APP_ICONS[i].png_bytes, APP_ICONS[i].bg_rgb))
    })
}

fn full_color_cache() -> &'static [Image; APP_COUNT] {
    static CACHE: OnceLock<[Image; APP_COUNT]> = OnceLock::new();
    CACHE.get_or_init(|| {
        let dec = decoded_icons();
        std::array::from_fn(|i| {
            Image::rgba(dec[i].width, dec[i].height, dec[i].rgba.clone())
                .expect("valid high-res app icon")
        })
    })
}

fn build_themed(dec: &DecodedIcon, bg: Color, fg: Color, glyph_only: bool) -> Image {
    let total = (dec.width * dec.height) as usize;
    let mut rgba = vec![0u8; total * 4];
    for pi in 0..total {
        let o = pi * 4;
        let circle_a = dec.rgba[o + 3];
        if circle_a == 0 {
            continue;
        }
        let m = dec.fg_mask[pi] as u32;
        if glyph_only {
            if m > 0 {
                rgba[o] = fg.0;
                rgba[o + 1] = fg.1;
                rgba[o + 2] = fg.2;
                rgba[o + 3] = ((m * circle_a as u32 + 127) / 255) as u8;
            }
        } else {
            let inv = 255 - m;
            rgba[o] = ((bg.0 as u32 * inv + fg.0 as u32 * m + 127) / 255) as u8;
            rgba[o + 1] = ((bg.1 as u32 * inv + fg.1 as u32 * m + 127) / 255) as u8;
            rgba[o + 2] = ((bg.2 as u32 * inv + fg.2 as u32 * m + 127) / 255) as u8;
            rgba[o + 3] = circle_a;
        }
    }
    Image::rgba(dec.width, dec.height, rgba).expect("valid high-res themed app icon")
}

struct ThemedCache {
    key: (Color, Color),
    images: [Image; APP_COUNT],
    glyphs: [Image; APP_COUNT],
}

fn themed_cache_entry(app: AppId, bg: Color, fg: Color, glyph_only: bool) -> Image {
    static CACHE: OnceLock<Mutex<Option<ThemedCache>>> = OnceLock::new();
    let lock = CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().expect("themed icon cache lock");
    let idx = app_index(app);
    if let Some(ref c) = *guard
        && c.key == (bg, fg)
    {
        return if glyph_only {
            c.glyphs[idx].clone()
        } else {
            c.images[idx].clone()
        };
    }
    let dec = decoded_icons();
    let images = std::array::from_fn(|i| build_themed(&dec[i], bg, fg, false));
    let glyphs = std::array::from_fn(|i| build_themed(&dec[i], bg, fg, true));
    let out = if glyph_only {
        glyphs[idx].clone()
    } else {
        images[idx].clone()
    };
    *guard = Some(ThemedCache {
        key: (bg, fg),
        images,
        glyphs,
    });
    out
}

/// Pre-decode all 18 high-res app icons and return them for upfront GPU atlas & mipmap upload
/// at compositor boot, eliminating any first-open App Drawer / Overview hitch.
#[cfg(feature = "compositor")]
pub fn prewarm_all(p: &Palette) -> Vec<Image> {
    let mut out = Vec::with_capacity(APP_COUNT * 2);
    out.extend(full_color_cache().iter().cloned());
    for &app in AppId::ALL.iter() {
        out.push(themed_cache_entry(
            app,
            p.primary_container,
            p.on_primary_container,
            false,
        ));
    }
    out
}

/// Real outer circle backdrop color for `app` (used during Android 13 app open & return-to-icon morph!).
pub fn app_icon_morph_bg(app: AppId, themed: bool, p: &Palette) -> Color {
    if themed {
        p.primary_container
    } else {
        let (r, g, b) = APP_ICONS[app_index(app)].bg_rgb;
        Color(r, g, b, 255)
    }
}

fn clock_hand_angles(hour: u32, minute: u32, second: u32) -> [f32; 3] {
    let seconds = second as f32;
    let minutes = minute as f32 + seconds / 60.0;
    [
        TAU * (hour % 12) as f32 / 12.0 + TAU * minutes / 720.0,
        TAU * minutes / 60.0,
        TAU * seconds / 60.0,
    ]
}

fn paint_clock_icon(clock: super::ClockReading, f: &mut Frame<'_>, r: Rect, themed: bool, p: &Palette, face: bool) {
    let side = r.width.min(r.height);
    if side <= 0.0 {
        return;
    }
    let visible = r.intersect(f.viewport());
    if visible.width <= 0.0 || visible.height <= 0.0 {
        return;
    }
    let now = clock.0;
    let deadline = f.time() + clock.until_second();
    f.wake_at(deadline);

    let center = [r.x + r.width * 0.5, r.y + r.height * 0.5];
    if face {
        f.paint_rect(
            Rect { x: center[0] - side * 0.5, y: center[1] - side * 0.5, width: side, height: side },
            side * 0.5,
            app_icon_morph_bg(AppId::Clock, themed, p),
        );
    }

    let ink = if themed { p.on_primary_container } else { Color(250, 250, 246, 255) };
    let second_ink = if themed { p.on_primary_container.opacity(0.72) } else { Color(249, 186, 67, 255) };
    let angles = clock_hand_angles(now.hour(), now.minute(), now.second());
    for (angle, length, width, color) in [
        (angles[0], 0.255, 0.070, ink),
        (angles[1], 0.376, 0.051, ink),
        (angles[2], 0.385, 0.026, second_ink),
    ] {
        let to = [
            center[0] + angle.sin() * side * length,
            center[1] - angle.cos() * side * length,
        ];
        f.paint_line(center, to, side * width, color);
    }
    let radius = side * 0.068;
    f.paint_rect(
        Rect { x: center[0] - radius, y: center[1] - radius, width: radius * 2.0, height: radius * 2.0 },
        radius,
        second_ink,
    );
}

/// Paint the authentic high-resolution (`176x176` xxxhdpi) Android 13 / LineageOS 20 icon for `app`.
pub fn paint_real_app_icon(clock: super::ClockReading, f: &mut Frame<'_>, r: Rect, app: AppId, themed: bool, p: &Palette) {
    if app == AppId::Clock {
        paint_clock_icon(clock, f, r, themed, p, true);
        return;
    }
    let rad = r.width.min(r.height) * 0.5;
    if themed {
        let img = themed_cache_entry(app, p.primary_container, p.on_primary_container, false);
        f.paint_image(r, &img, ImageFit::Cover, rad);
    } else {
        let img = &full_color_cache()[app_index(app)];
        f.paint_image(r, img, ImageFit::Cover, rad);
    }
}

/// Paint the icon foreground centered inside an expanding/shrinking Android 13 transition container.
pub fn paint_real_app_morph_glyph(clock: super::ClockReading, f: &mut Frame<'_>, r: Rect, app: AppId, themed: bool, p: &Palette) {
    if app == AppId::Clock {
        paint_clock_icon(clock, f, r, themed, p, !themed);
        return;
    }
    if themed {
        let img = themed_cache_entry(app, p.primary_container, p.on_primary_container, true);
        f.paint_image(r, &img, ImageFit::Cover, 0.0);
    } else {
        let img = &full_color_cache()[app_index(app)];
        f.paint_image(r, img, ImageFit::Cover, r.width.min(r.height) * 0.5);
    }
}

/// Paint an official Google Material Outlined system icon tinted to `color`.
pub fn paint_real_sys_icon(f: &mut Frame<'_>, r: Rect, kind: SysIcon, color: Color) {
    f.paint_system_icon(r, kind, color);
}

/// Anti-aliased 4-corner inverse rounded screen mask for the desktop phone bezel.
pub fn paint_rounded_screen_mask(f: &mut Frame<'_>, phone: Rect, corner_rad: f32, outer_col: Color) {
    f.paint_rounded_screen_mask(phone, corner_rad, outer_col);
}

pub fn paint_styled_app_icon(clock: super::ClockReading, f: &mut Frame<'_>, r: Rect, app: AppId, themed: bool, p: &Palette, shape: usize) {
    if shape == 0 {
        paint_real_app_icon(clock, f, r, app, themed, p);
        return;
    }
    let side = r.width.min(r.height);
    let radius = side * match shape { 1 => 0.42, 2 => 0.045, 3 => 0.27, 4 => 0.15, 5 => 0.36, _ => 0.31 };
    let bounds = match shape {
        1 => Rect { y: r.y + side * 0.08, height: r.height - side * 0.16, ..r },
        3 => Rect { x: r.x + side * 0.09, width: r.width - side * 0.18, ..r },
        4 => Rect { y: r.y + side * 0.06, height: r.height - side * 0.12, ..r },
        _ => r,
    };
    let background = app_icon_morph_bg(app, themed, p);
    f.paint_rect(bounds, radius, background);
    if shape == 5 {
        let tip = Rect { x: r.x + side * 0.59, y: r.y + side * 0.59, width: side * 0.41, height: side * 0.41 };
        f.paint_rect(tip, side * 0.065, background);
    }
    let content = r.inset(side * 0.14);
    if app == AppId::Clock {
        paint_clock_icon(clock, f, content, themed, p, false);
    } else if themed {
        let glyph = themed_cache_entry(app, p.primary_container, p.on_primary_container, true);
        f.paint_image(content, &glyph, ImageFit::Contain, 0.0);
    } else {
        let image = &full_color_cache()[app_index(app)];
        f.paint_image(content, image, ImageFit::Contain, side * 0.36);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_clock_hands_include_elapsed_minutes_and_seconds() {
        assert_eq!(clock_hand_angles(0, 0, 0), [0.0; 3]);
        assert_eq!(clock_hand_angles(12, 0, 0), [0.0; 3]);
        let [hour, minute, second] = clock_hand_angles(6, 30, 15);
        assert!((hour / TAU - (6.0 + 30.25 / 60.0) / 12.0).abs() < 0.0001);
        assert!((minute / TAU - 30.25 / 60.0).abs() < 0.0001);
        assert!((second / TAU - 0.25).abs() < 0.0001);
    }

    #[test]
    fn live_clock_schedules_a_tick_without_requesting_continuous_frames() {
        let mut ui = bhoo::Ui::default();
        let bounds = Rect { x: 0.0, y: 0.0, width: 56.0, height: 56.0 };
        let input = bhoo::Input { time: 5.0, ..bhoo::Input::default() };
        ui.frame(input, bounds, 0.0, |f| {
            paint_real_app_icon(super::super::TimeService::default().now(), f, bounds, AppId::Clock, false, &super::super::SystemState::default().palette());
        });
        let deadline = ui.next_wakeup().expect("the next clock tick is scheduled");
        assert!(deadline > input.time && deadline <= input.time + 1.0);
        assert!(!ui.needs_redraw());
    }

}
