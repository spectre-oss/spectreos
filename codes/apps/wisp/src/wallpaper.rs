use bhoo::{Color, Frame, Image, ImageFit, Rect};
use image::{RgbaImage, imageops};
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PRESETS: [&str; 8] = [
    "Obsidian Sand",
    "Forest Mist",
    "Twilight Plum",
    "Desert Ochre",
    "Deep Ocean",
    "Forest Sage",
    "Warm Terracotta",
    "Midnight Charcoal",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Home,
    Lock,
    Both,
}

#[derive(Clone)]
pub enum Wallpaper {
    Preset(usize),
    Photo { key: String, image: Image },
    Original(Option<Image>),
}

impl Wallpaper {
    pub fn paint(&self, f: &mut Frame<'_>, rect: Rect, default: usize) {
        match self {
            Self::Preset(index) => super::paint_wallpaper(f, rect, *index),
            Self::Photo { image, .. } | Self::Original(Some(image)) => {
                f.paint_image(rect, image, ImageFit::Cover, 0.0);
            }
            Self::Original(None) => super::paint_wallpaper(f, rect, default),
        }
    }

    fn entry(&self) -> String {
        match self {
            Self::Preset(index) => format!("preset:{index}"),
            Self::Photo { key, .. } => key.clone(),
            Self::Original(_) => "original".into(),
        }
    }

    pub fn uses_preset(&self, index: usize, default: usize) -> bool {
        match self {
            Self::Preset(selected) => *selected == index,
            Self::Original(None) => index == default,
            _ => false,
        }
    }

    pub fn colors(&self, default: usize) -> [Color; 4] {
        let image = match self {
            Self::Photo { image, .. } | Self::Original(Some(image)) => image,
            Self::Preset(index) => return preset_colors(*index),
            Self::Original(None) => return preset_colors(default),
        };
        let mut hues = [(0_u32, [0_u32; 3]); 24];
        for pixel in image.pixels().chunks_exact(4).step_by(61) {
            let [red, green, blue, opacity] = [pixel[0], pixel[1], pixel[2], pixel[3]];
            let hi = red.max(green).max(blue);
            let lo = red.min(green).min(blue);
            let delta = (hi - lo) as f32;
            if opacity < 192 || delta < 16.0 || hi < 30 {
                continue;
            }
            let hue = if hi == red {
                (green as f32 - blue as f32) / delta
            } else if hi == green {
                2.0 + (blue as f32 - red as f32) / delta
            } else {
                4.0 + (red as f32 - green as f32) / delta
            };
            let (count, channels) = &mut hues[(hue.rem_euclid(6.0) * 4.0) as usize % 24];
            *count += 1;
            for (sum, value) in channels.iter_mut().zip(pixel) {
                *sum += u32::from(*value);
            }
        }
        hues.sort_by_key(|(count, _)| std::cmp::Reverse(*count));
        let mut result = preset_colors(default);
        for (slot, (count, channels)) in result.iter_mut().zip(hues) {
            if count > 0 {
                let [r, g, b] = channels.map(|value| (value / count) as u8);
                *slot = Color(r, g, b, 255);
            }
        }
        result
    }
}

fn preset_colors(index: usize) -> [Color; 4] {
    let [r, g, b] = [
        [115, 113, 136],
        [69, 109, 88],
        [113, 94, 151],
        [141, 99, 61],
        [52, 90, 139],
        [68, 109, 94],
        [148, 83, 69],
        [103, 112, 133],
    ][index % PRESETS.len()];
    [
        Color(r, g, b, 255),
        Color(r + 64, g, b, 255),
        Color(r, g + 50, b, 255),
        Color(r, g, b + 56, 255),
    ]
}

pub struct Library {
    pub home: Wallpaper,
    pub lock: Wallpaper,
    dir: Option<PathBuf>,
}

impl Default for Library {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let root = super::freedesktop::xdg_data_home().map(|dir| dir.join("wisp"));
        #[cfg(target_arch = "wasm32")]
        let root = None;
        Self::open(root)
    }
}

impl Library {
    fn open(root: Option<PathBuf>) -> Self {
        let original = Wallpaper::Original(root.as_ref().and_then(|root| {
            let photo = spectre_files::read_image(&root.join("wallpaper.png"), 1022).ok()?;
            make_image(photo).ok()
        }));
        let dir = root.map(|root| root.join("wallpapers"));
        let entry = |label: &str| {
            dir.as_deref()
                .and_then(|dir| {
                    let config = fs::read_to_string(dir.join("active")).ok()?;
                    let value = config.lines().find_map(|line| {
                        let (key, value) = line.split_once('=')?;
                        (key == label).then_some(value)
                    })?;
                    if let Some(index) = value
                        .strip_prefix("preset:")
                        .and_then(|index| index.parse::<usize>().ok())
                    {
                        return (index < PRESETS.len()).then_some(Wallpaper::Preset(index));
                    }
                    if value.starts_with("photo-")
                        && value.ends_with(".png")
                        && value
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"-.".contains(&byte))
                    {
                        let pixels = spectre_files::read_image(&dir.join(value), 1022).ok()?;
                        return Some(Wallpaper::Photo {
                            key: value.into(),
                            image: make_image(pixels).ok()?,
                        });
                    }
                    None
                })
                .unwrap_or_else(|| original.clone())
        };
        Self {
            home: entry("home"),
            lock: entry("lock"),
            dir,
        }
    }

    pub fn apply(
        &mut self,
        pending: &Pending,
        destination: Destination,
        aspect: f32,
    ) -> io::Result<()> {
        let dir = self
            .dir
            .as_ref()
            .ok_or_else(|| io::Error::other("Wallpaper storage is unavailable"))?;
        fs::create_dir_all(dir)?;
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let wallpaper = match &pending.source {
            Source::Preset(index) => Wallpaper::Preset(*index),
            Source::Current(wallpaper) => wallpaper.clone(),
            Source::Photo { pixels, .. } => {
                let (x, y, width, height) = crop_area(
                    pixels.width(),
                    pixels.height(),
                    aspect,
                    pending.zoom,
                    pending.center,
                );
                let mut cropped = imageops::crop_imm(pixels, x, y, width, height).to_image();
                if width.max(height) > 1022 {
                    cropped = image::DynamicImage::ImageRgba8(cropped)
                        .resize(1022, 1022, imageops::FilterType::Lanczos3)
                        .into_rgba8();
                }
                for pixel in cropped.pixels_mut() {
                    for channel in 0..3 {
                        pixel[channel] =
                            (u16::from(pixel[channel]) * u16::from(pixel[3]) / 255) as u8;
                    }
                    pixel[3] = 255;
                }
                let key = format!("photo-{tick}-{}.png", std::process::id());
                let image = make_image(cropped.clone())?;
                cropped.save(dir.join(&key)).map_err(io::Error::other)?;
                Wallpaper::Photo { key, image }
            }
        };
        let home = if destination == Destination::Lock {
            &self.home
        } else {
            &wallpaper
        };
        let lock = if destination == Destination::Home {
            &self.lock
        } else {
            &wallpaper
        };
        let temp = dir.join(format!(".active-{tick}-{}", std::process::id()));
        if let Err(error) = fs::write(
            &temp,
            format!("home={}\nlock={}\n", home.entry(), lock.entry()),
        )
        .and_then(|()| fs::rename(&temp, dir.join("active")))
        {
            let _ = fs::remove_file(temp);
            return Err(error);
        }
        self.home = home.clone();
        self.lock = lock.clone();
        Ok(())
    }
}

pub enum Source {
    Preset(usize),
    Current(Wallpaper),
    Photo {
        pixels: RgbaImage,
        preview: Image,
        name: String,
    },
}

pub struct Pending {
    pub source: Source,
    pub return_to: super::LauncherSub,
    pub lock_tab: bool,
    pub show_info: bool,
    pub edit: bool,
    pub show_apply: bool,
    pub zoom: f32,
    pub center: [f32; 2],
    pub drag: Option<[f32; 2]>,
}

impl Pending {
    pub fn new(source: Source, return_to: super::LauncherSub, lock_tab: bool) -> Self {
        Self {
            source,
            return_to,
            lock_tab,
            show_info: false,
            edit: false,
            show_apply: false,
            zoom: 1.0,
            center: [0.5, 0.5],
            drag: None,
        }
    }

    pub fn load(path: &Path) -> io::Result<Source> {
        let pixels = spectre_files::read_image(path, 2560)?;
        let thumbnail = if pixels.width().max(pixels.height()) > 1022 {
            image::DynamicImage::ImageRgba8(pixels.clone())
                .resize(1022, 1022, imageops::FilterType::Triangle)
                .into_rgba8()
        } else {
            pixels.clone()
        };
        let name = path
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
            .into_owned();
        Ok(Source::Photo {
            pixels,
            preview: make_image(thumbnail)?,
            name,
        })
    }

    pub fn paint(&self, f: &mut Frame<'_>, rect: Rect, default: usize, aspect: f32) {
        match &self.source {
            Source::Preset(index) => super::paint_wallpaper(f, rect, *index),
            Source::Current(wallpaper) => wallpaper.paint(f, rect, default),
            Source::Photo { preview, .. } => {
                let (x, y, w, h) = crop_area(
                    preview.width(),
                    preview.height(),
                    aspect,
                    self.zoom,
                    self.center,
                );
                let sx = rect.width / w as f32;
                let sy = rect.height / h as f32;
                let full = Rect {
                    x: rect.x - x as f32 * sx,
                    y: rect.y - y as f32 * sy,
                    width: preview.width() as f32 * sx,
                    height: preview.height() as f32 * sy,
                };
                f.within(rect, |f| {
                    f.paint_image(full, preview, ImageFit::Contain, 0.0);
                });
            }
        }
    }

    pub fn pan(&mut self, distance: [f32; 2], screen: Rect, aspect: f32) {
        if let Source::Photo { preview, .. } = &self.source {
            let (_, _, w, h) = crop_area(
                preview.width(),
                preview.height(),
                aspect,
                self.zoom,
                self.center,
            );
            let visible = [
                w as f32 / preview.width() as f32,
                h as f32 / preview.height() as f32,
            ];
            for (axis, edge) in [screen.width, screen.height].into_iter().enumerate() {
                let inset = visible[axis] * 0.5;
                self.center[axis] = (self.center[axis]
                    + distance[axis] * visible[axis] / edge.max(1.0))
                .clamp(inset, 1.0 - inset);
            }
        }
    }

    pub fn info(&self) -> String {
        match &self.source {
            Source::Preset(index) => PRESETS[*index].into(),
            Source::Current(_) => "Current wallpaper".into(),
            Source::Photo { name, pixels, .. } => {
                format!("{name}  ·  {} × {}", pixels.width(), pixels.height())
            }
        }
    }
}

fn make_image(image: RgbaImage) -> io::Result<Image> {
    Image::rgba(image.width(), image.height(), image.into_raw())
        .ok_or_else(|| io::Error::other("Could not prepare image"))
}

fn crop_area(
    width: u32,
    height: u32,
    aspect: f32,
    zoom: f32,
    center: [f32; 2],
) -> (u32, u32, u32, u32) {
    let aspect = aspect.clamp(0.2, 5.0);
    let crop_w = ((width as f32).min(height as f32 * aspect) / zoom)
        .round()
        .max(1.0) as u32;
    let crop_h = ((height as f32).min(width as f32 / aspect) / zoom)
        .round()
        .max(1.0) as u32;
    let x = (center[0] * width as f32 - crop_w as f32 * 0.5)
        .round()
        .clamp(0.0, (width - crop_w) as f32);
    let y = (center[1] * height as f32 - crop_h as f32 * 0.5)
        .round()
        .clamp(0.0, (height - crop_h) as f32);
    (x as u32, y as u32, crop_w, crop_h)
}

pub const SHORTCUTS: [&str; 7] = [
    "None",
    "Mute",
    "Video camera",
    "Camera",
    "Do Not Disturb",
    "Flashlight",
    "Device controls",
];

pub const ICON_STYLES: [&str; 7] = [
    "Default", "Outline", "Sharp", "Rounded", "Circular", "Compact", "Filled",
];

pub const ICON_SHAPES: [&str; 7] = [
    "Default",
    "Pebble",
    "Square",
    "Vessel",
    "Tapered rectangle",
    "Teardrop",
    "Squircle",
];

pub const FONT_STYLES: [&str; 4] = ["Default", "Serif", "Monospace", "Compact sans"];

pub fn customize_font(mut style: bhoo::TextStyle, font: usize) -> bhoo::TextStyle {
    match font {
        1 => style.family = bhoo::FontFamily::Serif,
        2 => style.family = bhoo::FontFamily::Monospace,
        3 => style.letter_spacing = -0.38,
        _ => {}
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_is_bounded_and_centering_can_pan_to_both_edges() {
        assert_eq!(
            crop_area(1000, 500, 0.5, 1.0, [0.5, 0.5]),
            (375, 0, 250, 500)
        );
        assert_eq!(
            crop_area(1000, 500, 0.5, 2.0, [1.0, 1.0]),
            (875, 250, 125, 250)
        );
        assert_eq!(crop_area(1, 1, 0.48, 4.0, [0.5, 0.5]), (0, 0, 1, 1));
    }

    #[test]
    fn imports_persist_after_source_is_removed_and_keep_the_other_destination() {
        let path =
            std::env::temp_dir().join(format!("spectre-wallpaper-test-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        let mut lib = Library::open(Some(path.clone()));
        let preset = Pending::new(
            Source::Preset(3),
            super::super::LauncherSub::WallpaperPicker,
            true,
        );
        lib.apply(&preset, Destination::Lock, 0.5).unwrap();
        let source = path.join("local.png");
        RgbaImage::from_pixel(200, 160, image::Rgba([26, 174, 51, 255]))
            .save(&source)
            .unwrap();
        let pending = Pending::new(
            Pending::load(&source).unwrap(),
            super::super::LauncherSub::WallpaperFiles,
            false,
        );
        lib.apply(&pending, Destination::Home, 0.5).unwrap();
        fs::remove_file(source).unwrap();

        let reloaded = Library::open(Some(path.clone()));
        assert!(reloaded.lock.uses_preset(3, 0));
        assert!(matches!(reloaded.home, Wallpaper::Photo { ref image, .. }
            if (image.width(), image.height()) == (80, 160)));
        assert_eq!(reloaded.home.colors(0)[0], Color(26, 174, 51, 255));
        fs::remove_dir_all(path).unwrap();
    }
}
