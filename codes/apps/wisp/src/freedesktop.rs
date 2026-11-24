//! Freedesktop.org specification implementation for Wisp:
//! - XDG Base Directory Specification v0.8 (`docs/freedesktop/basedir/index.md`)
//! - Desktop Entry Specification v1.5 (`docs/freedesktop/desktop-entry/index.md`)
//! - Icon Theme Specification v0.13 (`docs/freedesktop/icon-theme/index.md`)
//!
//! Discovers installed Linux/NixOS `.desktop` applications, parses localized metadata,
//! expands `Exec=` command lines & field codes (`%i`, `%c`, `%k`, `%%`, `%f/%F/%u/%U`),
//! and resolves + decodes PNG and SVG icons from active XDG icon themes with `hicolor`
//! and `pixmaps` fallbacks.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use bhoo::{Color, Frame, Image, ImageFit, Rect};

use crate::{AppId, Palette, text_style};

/// Target square pixel size for Freedesktop icons cached into libbhoo's GPU mipmap atlas.
/// `128x128` fits ~130+ system icons alongside Wisp's built-in icons in the 2048x2048 atlas
/// while staying crisp on `1080x2246` (`2.7x` scale) with hardware trilinear mipmaps.
pub const XDG_ICON_TARGET_PX: u32 = 128;

#[derive(Clone, Debug)]
pub struct DesktopAction {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub exec_argv: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DesktopApp {
    /// Desktop File ID per Desktop Entry Spec §2.1 (e.g. `org.gnome.Nautilus.desktop`).
    pub id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub icon: Option<String>,
    pub exec_raw: Option<String>,
    pub exec_argv: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub terminal: bool,
    pub dbus_activatable: bool,
    pub startup_wm_class: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub desktop_path: PathBuf,
    pub actions: Vec<DesktopAction>,
    /// Android package name when this `.desktop` entry is backed by Shiraz (`X-Shiraz-Package`).
    pub shiraz_package: Option<String>,
}

impl DesktopApp {
    /// Match a search query against Name, GenericName, Comment, Keywords, Categories, and ID.
    pub fn matches_query(&self, query_lower: &str) -> bool {
        if query_lower.is_empty() {
            return true;
        }
        if self.name.to_ascii_lowercase().contains(query_lower)
            || self.id.to_ascii_lowercase().contains(query_lower)
        {
            return true;
        }
        if self
            .generic_name
            .as_deref()
            .is_some_and(|s| s.to_ascii_lowercase().contains(query_lower))
        {
            return true;
        }
        if self
            .comment
            .as_deref()
            .is_some_and(|s| s.to_ascii_lowercase().contains(query_lower))
        {
            return true;
        }
        self.keywords
            .iter()
            .chain(self.categories.iter())
            .any(|s| s.to_ascii_lowercase().contains(query_lower))
    }

    /// Best-effort executable binary basename (used to match running processes/built-ins).
    pub fn exec_basename(&self) -> Option<&str> {
        let first = self.exec_argv.first()?;
        Path::new(first).file_name().and_then(|s| s.to_str())
    }
}

/// XDG Icon Theme directory scaling mode per Icon Theme Spec §4 Table 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirType {
    Fixed,
    Scalable,
    Threshold,
}

#[derive(Clone, Debug)]
pub struct IconSubdir {
    pub path: String,
    pub size: u32,
    pub scale: u32,
    pub dir_type: DirType,
    pub min_size: u32,
    pub max_size: u32,
    pub threshold: u32,
}

impl IconSubdir {
    /// Icon Theme Spec §5 `DirectoryMatchesSize`.
    pub fn matches_size(&self, icon_size: u32, icon_scale: u32) -> bool {
        if self.scale != icon_scale {
            return false;
        }
        match self.dir_type {
            DirType::Fixed => self.size == icon_size,
            DirType::Scalable => self.min_size <= icon_size && icon_size <= self.max_size,
            DirType::Threshold => {
                icon_size >= self.size.saturating_sub(self.threshold)
                    && icon_size <= self.size.saturating_add(self.threshold)
            }
        }
    }

    /// Icon Theme Spec §5 `DirectorySizeDistance`.
    pub fn size_distance(&self, icon_size: u32, icon_scale: u32) -> u32 {
        let wanted = icon_size.saturating_mul(icon_scale);
        match self.dir_type {
            DirType::Fixed => self.size.saturating_mul(self.scale).abs_diff(wanted),
            DirType::Scalable => {
                let lo = self.min_size.saturating_mul(self.scale);
                let hi = self.max_size.saturating_mul(self.scale);
                if wanted < lo {
                    lo - wanted
                } else if wanted > hi {
                    wanted - hi
                } else {
                    0
                }
            }
            DirType::Threshold => {
                let lo = self
                    .size
                    .saturating_sub(self.threshold)
                    .saturating_mul(self.scale);
                let hi = self
                    .size
                    .saturating_add(self.threshold)
                    .saturating_mul(self.scale);
                if wanted < lo {
                    lo - wanted
                } else if wanted > hi {
                    wanted - hi
                } else {
                    0
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct IconTheme {
    name: String,
    inherits: Vec<String>,
    subdirs: Vec<IconSubdir>,
}

/// Icon Theme Spec v0.13 resolver across `$HOME/.icons`, `$XDG_DATA_HOME/icons`,
/// `$XDG_DATA_DIRS/icons`, and `/usr/share/pixmaps`.
#[derive(Clone, Debug, Default)]
pub struct IconThemeResolver {
    base_dirs: Vec<PathBuf>,
    pixmap_dirs: Vec<PathBuf>,
    active_themes: Vec<String>,
    themes: HashMap<String, IconTheme>,
    resolved_paths: HashMap<String, Option<PathBuf>>,
    decoded_images: HashMap<String, Option<Image>>,
}

impl IconThemeResolver {
    pub fn discover() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::default()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (base_dirs, pixmap_dirs) = xdg_icon_dirs();
            Self::from_dirs(base_dirs, pixmap_dirs, detect_active_icon_themes())
        }
    }

    pub fn from_dirs(
        base_dirs: Vec<PathBuf>,
        pixmap_dirs: Vec<PathBuf>,
        preferred_themes: Vec<String>,
    ) -> Self {
        let mut resolver = Self {
            base_dirs,
            pixmap_dirs,
            active_themes: Vec::new(),
            themes: HashMap::new(),
            resolved_paths: HashMap::new(),
            decoded_images: HashMap::new(),
        };
        resolver.index_themes();
        for t in preferred_themes {
            if !t.is_empty()
                && !resolver.active_themes.contains(&t)
                && resolver.themes.contains_key(&t)
            {
                resolver.active_themes.push(t);
            }
        }
        // Standard fallback themes before mandatory "hicolor" (Icon Theme Spec §5)
        for fallback in ["Adwaita", "Papirus", "breeze", "gnome"] {
            if resolver.themes.contains_key(fallback)
                && !resolver.active_themes.iter().any(|x| x == fallback)
            {
                resolver.active_themes.push(fallback.to_string());
            }
        }
        resolver
    }

    fn index_themes(&mut self) {
        let mut seen = HashSet::new();
        for base in &self.base_dirs {
            let Ok(entries) = std::fs::read_dir(base) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let Some(theme_id) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                if seen.contains(theme_id) {
                    continue;
                }
                let index_file = path.join("index.theme");
                if let Ok(content) = std::fs::read_to_string(&index_file)
                    && let Some(theme) = parse_icon_theme(theme_id, &content)
                {
                    seen.insert(theme_id.to_string());
                    self.themes.insert(theme_id.to_string(), theme);
                }
            }
        }
        // Icon Theme Spec §3: `hicolor` is the mandatory fallback theme. Even if no
        // system `hicolor/index.theme` is present (e.g. isolated XDG prefix or minimal
        // rootfs), provide standard hicolor app icon directories so per-user icons in
        // `$XDG_DATA_HOME/icons/hicolor/256x256/apps/` always resolve cleanly.
        if !self.themes.contains_key("hicolor") {
            let mut subdirs = Vec::new();
            for sz in [512, 256, 192, 128, 96, 64, 48, 32] {
                subdirs.push(IconSubdir {
                    path: format!("{sz}x{sz}/apps"),
                    size: sz,
                    scale: 1,
                    dir_type: DirType::Threshold,
                    min_size: sz,
                    max_size: sz,
                    threshold: 2,
                });
            }
            subdirs.push(IconSubdir {
                path: "scalable/apps".into(),
                size: 128,
                scale: 1,
                dir_type: DirType::Scalable,
                min_size: 16,
                max_size: 512,
                threshold: 2,
            });
            self.themes.insert(
                "hicolor".into(),
                IconTheme {
                    name: "hicolor".into(),
                    inherits: Vec::new(),
                    subdirs,
                },
            );
        }
    }

    /// Icon Theme Spec §5 `FindIcon(icon, size, scale)` + absolute path support (Desktop Entry §6).
    pub fn find_icon(&mut self, icon_name: &str, size: u32, scale: u32) -> Option<PathBuf> {
        let key = icon_name.trim();
        if key.is_empty() {
            return None;
        }
        if let Some(cached) = self.resolved_paths.get(key) {
            return cached.clone();
        }
        let found = self.find_icon_uncached(key, size, scale);
        self.resolved_paths.insert(key.to_string(), found.clone());
        found
    }

    fn find_icon_uncached(&self, icon_name: &str, size: u32, scale: u32) -> Option<PathBuf> {
        // Desktop Entry Spec §6: If Icon is an absolute path, use the given file directly.
        let direct = Path::new(icon_name);
        if direct.is_absolute() {
            if direct.is_file() {
                return Some(direct.to_path_buf());
            }
            return None;
        }

        // Strip trailing extension if a desktop entry mistakenly includes `.png` / `.svg`
        let bare = icon_name
            .strip_suffix(".png")
            .or_else(|| icon_name.strip_suffix(".svg"))
            .or_else(|| icon_name.strip_suffix(".xpm"))
            .unwrap_or(icon_name);

        let mut visited_themes = HashSet::new();
        for theme in &self.active_themes {
            if let Some(p) = self.find_icon_helper(bare, size, scale, theme, &mut visited_themes) {
                return Some(p);
            }
        }
        // Mandatory fallback to "hicolor" (Icon Theme Spec §3 & §5)
        if let Some(p) = self.find_icon_helper(bare, size, scale, "hicolor", &mut visited_themes) {
            return Some(p);
        }
        // Unthemed fallback across icon base directories and pixmaps (Icon Theme Spec §5)
        self.lookup_fallback_icon(bare)
    }

    fn find_icon_helper(
        &self,
        icon_name: &str,
        size: u32,
        scale: u32,
        theme_name: &str,
        visited: &mut HashSet<String>,
    ) -> Option<PathBuf> {
        if !visited.insert(theme_name.to_string()) {
            return None;
        }
        if let Some(found) = self.lookup_icon_in_theme(icon_name, size, scale, theme_name) {
            return Some(found);
        }
        if let Some(theme) = self.themes.get(theme_name) {
            for parent in &theme.inherits {
                if let Some(found) =
                    self.find_icon_helper(icon_name, size, scale, parent, visited)
                {
                    return Some(found);
                }
            }
        }
        None
    }

    fn lookup_icon_in_theme(
        &self,
        icon_name: &str,
        size: u32,
        scale: u32,
        theme_name: &str,
    ) -> Option<PathBuf> {
        let theme = self.themes.get(theme_name)?;
        let exts = supported_icon_extensions();

        // Phase 1: Exact size + scale match (prefer higher-res/scalable if exact)
        for subdir in &theme.subdirs {
            if !subdir.matches_size(size, scale) {
                continue;
            }
            for base in &self.base_dirs {
                let dir = base.join(&theme.name).join(&subdir.path);
                if !dir.is_dir() {
                    continue;
                }
                for ext in exts {
                    let candidate = dir.join(format!("{icon_name}.{ext}"));
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }

        // Phase 2: Closest directory by `DirectorySizeDistance`
        let mut best_dist = u32::MAX;
        let mut best_path: Option<PathBuf> = None;
        for subdir in &theme.subdirs {
            let dist = subdir.size_distance(size, scale);
            if dist >= best_dist {
                continue;
            }
            for base in &self.base_dirs {
                let dir = base.join(&theme.name).join(&subdir.path);
                if !dir.is_dir() {
                    continue;
                }
                for ext in exts {
                    let candidate = dir.join(format!("{icon_name}.{ext}"));
                    if candidate.is_file() {
                        best_dist = dist;
                        best_path = Some(candidate);
                        break;
                    }
                }
                if best_dist == 0 {
                    return best_path;
                }
            }
        }
        best_path
    }

    fn lookup_fallback_icon(&self, icon_name: &str) -> Option<PathBuf> {
        let exts = supported_icon_extensions();
        for base in self.base_dirs.iter().chain(self.pixmap_dirs.iter()) {
            if !base.is_dir() {
                continue;
            }
            for ext in exts {
                let candidate = base.join(format!("{icon_name}.{ext}"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// Resolve, decode (PNG or SVG), and cache a GPU-ready `bhoo::Image` for `icon_name`.
    pub fn load_icon_image(&mut self, icon_name: &str) -> Option<Image> {
        let key = icon_name.trim();
        if key.is_empty() {
            return None;
        }
        if let Some(cached) = self.decoded_images.get(key) {
            return cached.clone();
        }
        // Shiraz exports launcher art from the installed APK into hicolor/256x256.
        // Allow the original detail through, with a bounded texture size.
        let img = if key.ends_with(".Shiraz") || key.ends_with(".Shiraz.png") {
            self.find_icon(key, 256, 1)
                .and_then(|path| decode_icon_file(&path, 256))
        } else {
            self.find_icon(key, 64, 2)
                .or_else(|| self.find_icon(key, 128, 1))
                .and_then(|path| decode_icon_file(&path, XDG_ICON_TARGET_PX))
        };
        self.decoded_images.insert(key.to_string(), img.clone());
        img
    }
}

fn supported_icon_extensions() -> &'static [&'static str] {
    #[cfg(feature = "svg-icons")]
    {
        &["svg", "png"]
    }
    #[cfg(not(feature = "svg-icons"))]
    {
        &["png"]
    }
}

/// Unified registry of discovered Freedesktop `.desktop` apps + Icon Theme loader.
#[derive(Clone, Debug, Default)]
pub struct DesktopRegistry {
    pub apps: Vec<DesktopApp>,
    pub icon_resolver: IconThemeResolver,
    #[allow(dead_code)]
    dir_stamp: u64,
}

impl DesktopRegistry {
    /// Discover installed XDG `.desktop` applications and icon themes on native Linux.
    /// Returns an empty registry on WASM or in unit tests without touching the filesystem.
    pub fn discover() -> Self {
        #[cfg(any(target_arch = "wasm32", test))]
        {
            Self::default()
        }
        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            let app_dirs = xdg_application_dirs();
            let dir_stamp = compute_desktop_dirs_stamp(&app_dirs);
            let locale = detect_posix_locale();
            let desktops = detect_current_desktops();
            let apps = scan_desktop_dirs(&app_dirs, &locale, &desktops);
            let icon_resolver = IconThemeResolver::discover();
            Self {
                apps,
                icon_resolver,
                dir_stamp,
            }
        }
    }

    /// Hot-reload `.desktop` applications and icon caches when any XDG applications dir changes
    /// (for example after installing or removing an Android APK via Shiraz).
    pub fn reload_if_changed(&mut self) -> bool {
        #[cfg(any(target_arch = "wasm32", test))]
        {
            false
        }
        #[cfg(all(not(target_arch = "wasm32"), not(test)))]
        {
            let app_dirs = xdg_application_dirs();
            let stamp = compute_desktop_dirs_stamp(&app_dirs);
            if stamp == self.dir_stamp {
                return false;
            }
            let locale = detect_posix_locale();
            let desktops = detect_current_desktops();
            self.apps = scan_desktop_dirs(&app_dirs, &locale, &desktops);
            self.icon_resolver = IconThemeResolver::discover();
            self.dir_stamp = stamp;
            true
        }
    }

    pub fn get(&self, desktop_id: &str) -> Option<&DesktopApp> {
        self.apps.iter().find(|a| a.id == desktop_id)
    }

    /// Match a Wayland `xdg_toplevel` `app_id` against discovered `.desktop` entries
    /// (Desktop Entry Spec §2.1 & `StartupWMClass`).
    pub fn find_by_wayland_app_id(&self, wl_app_id: &str) -> Option<&DesktopApp> {
        let needle = wl_app_id.trim();
        if needle.is_empty() {
            return None;
        }
        let with_ext = if needle.ends_with(".desktop") {
            needle.to_string()
        } else {
            format!("{needle}.desktop")
        };
        let lower = needle.to_ascii_lowercase();
        self.apps.iter().find(|app| {
            app.id == with_ext
                || app.id.eq_ignore_ascii_case(&with_ext)
                || app
                    .startup_wm_class
                    .as_deref()
                    .is_some_and(|wm| wm.eq_ignore_ascii_case(needle))
                || app
                    .exec_basename()
                    .is_some_and(|bin| bin.eq_ignore_ascii_case(&lower))
        })
    }

    /// Map a built-in Wisp `AppId` to a real installed Freedesktop `.desktop` application, if any.
    pub fn find_for_builtin(&self, app: AppId) -> Option<&DesktopApp> {
        let candidates: &[&str] = match app {
            AppId::Browser => &[
                "org.gnome.epiphany.desktop",
                "firefox.desktop",
                "chromium-browser.desktop",
                "org.chromium.chromium.desktop",
            ],
            AppId::Files => &["org.gnome.nautilus.desktop", "thunar.desktop", "dolphin.desktop"],
            AppId::Camera => &[
                "spectre-camera.desktop",
                "org.postmarketos.megapixels.desktop",
                "megapixels.desktop",
                "org.gnome.snapshot.desktop",
            ],
            AppId::Phone => &["org.gnome.calls.desktop", "sm.puri.calls.desktop"],
            AppId::Messaging => &["sm.puri.chatty.desktop"],
            AppId::Calculator => &["org.gnome.calculator.desktop"],
            AppId::Calendar => &["org.gnome.calendar.desktop"],
            AppId::Clock => &["org.gnome.clocks.desktop"],
            AppId::Contacts => &["org.gnome.contacts.desktop"],
            AppId::Gallery => &["org.gnome.loupe.desktop", "org.gnome.eog.desktop"],
            AppId::Music => &["org.gnome.music.desktop", "lollypop.desktop"],
            AppId::Telegram => &[
                "org.telegram.desktop.desktop",
                "telegramdesktop.desktop",
            ],
            AppId::DeltaChat => &[
                "chat.delta.desktop.desktop",
                "deltachat.desktop",
                "chat.delta.shiraz.desktop",
            ],
            _ => &[],
        };
        for cand in candidates {
            if let Some(entry) = self.apps.iter().find(|a| a.id.eq_ignore_ascii_case(cand)) {
                return Some(entry);
            }
        }
        // Secondary match by executable basename
        let bins: &[&str] = match app {
            AppId::Browser => &["epiphany", "firefox", "chromium"],
            AppId::Files => &["nautilus", "thunar", "dolphin"],
            AppId::Camera => &["spectre-camera", "megapixels", "snapshot"],
            AppId::Phone => &["gnome-calls"],
            AppId::Messaging => &["chatty"],
            _ => &[],
        };
        for bin in bins {
            if let Some(entry) = self
                .apps
                .iter()
                .find(|a| a.exec_basename().is_some_and(|b| b.eq_ignore_ascii_case(bin)))
            {
                return Some(entry);
            }
        }
        None
    }

    /// Resolve & cache a GPU-ready icon `Image` for a discovered `.desktop` app.
    pub fn icon_for(&mut self, desktop_id: &str) -> Option<Image> {
        let icon_name = self.get(desktop_id)?.icon.clone()?;
        self.icon_resolver.load_icon_image(&icon_name)
    }

    /// Pre-warm up to `limit` Freedesktop app icons at compositor boot for zero-hitch 60 Hz scrolling.
    #[cfg(feature = "compositor")]
    pub fn prewarm_icons(&mut self, limit: usize) -> Vec<Image> {
        let names: Vec<String> = self
            .apps
            .iter()
            .filter_map(|a| a.icon.clone())
            .take(limit)
            .collect();
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            if let Some(img) = self.icon_resolver.load_icon_image(&name) {
                out.push(img);
            }
        }
        out
    }
}

/// Paint a Freedesktop `.desktop` app icon inside `r` — uses its real XDG icon theme PNG/SVG
/// when available, or a crisp Material You adaptive circular monogram badge as fallback.
pub fn paint_desktop_app_icon(
    f: &mut Frame<'_>,
    r: Rect,
    app_name: &str,
    desktop_id: &str,
    icon_img: Option<&Image>,
    p: &Palette,
) {
    let rad = r.width.min(r.height) * 0.5;
    if let Some(img) = icon_img {
        f.paint_image(r, img, ImageFit::Contain, rad * 0.45);
        return;
    }
    // Crisp deterministic Material You monogram badge when an app ships no icon
    let mut hash: u32 = 2166136261;
    for b in desktop_id.bytes() {
        hash = (hash ^ b as u32).wrapping_mul(16777619);
    }
    const PALETTE: [Color; 8] = [
        Color(0x3B, 0x69, 0xB8, 255),
        Color(0x2E, 0x7D, 0x5B, 255),
        Color(0x7C, 0x4D, 0xA8, 255),
        Color(0xB8, 0x5A, 0x3B, 255),
        Color(0x2B, 0x7A, 0x8B, 255),
        Color(0x9A, 0x5B, 0x25, 255),
        Color(0x4A, 0x5F, 0x9D, 255),
        Color(0xA6, 0x3E, 0x5E, 255),
    ];
    let bg = PALETTE[(hash as usize) % PALETTE.len()];
    f.paint_rect(r, rad, bg);
    let initial: String = app_name
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "A".into());
    let _ = p;
    f.styled_text(
        r,
        &initial,
        Color(255, 255, 255, 245),
        text_style((r.height * 0.46).clamp(16.0, 28.0), 600, 1),
    );
}

// ============================================================================
// XDG Base Directory Specification v0.8 (§3)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub fn xdg_data_home() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(&val);
        // BaseDir Spec §2: relative paths in XDG env vars must be ignored.
        if !val.is_empty() && p.is_absolute() {
            return Some(p);
        }
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn xdg_data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = xdg_data_home() {
        dirs.push(home);
    }
    let env_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    let mut added_env = false;
    for part in env_dirs.split(':') {
        if part.is_empty() {
            continue;
        }
        let p = PathBuf::from(part);
        if p.is_absolute() {
            added_env = true;
            if !dirs.contains(&p) {
                dirs.push(p);
            }
        }
    }
    if !added_env {
        for def in ["/usr/local/share", "/usr/share"] {
            let p = PathBuf::from(def);
            if !dirs.contains(&p) {
                dirs.push(p);
            }
        }
    }
    // NixOS & user profile share directories (ensures apps work even if PAM didn't export XDG_DATA_DIRS)
    if let Some(home) = std::env::var_os("HOME") {
        let nix_user = PathBuf::from(home).join(".nix-profile/share");
        if nix_user.is_dir() && !dirs.contains(&nix_user) {
            dirs.push(nix_user);
        }
    }
    for extra in ["/run/current-system/sw/share", "/usr/share"] {
        let p = PathBuf::from(extra);
        if p.is_dir() && !dirs.contains(&p) {
            dirs.push(p);
        }
    }
    dirs
}

#[cfg(not(target_arch = "wasm32"))]
pub fn xdg_application_dirs() -> Vec<PathBuf> {
    xdg_data_dirs()
        .into_iter()
        .map(|d| d.join("applications"))
        .filter(|d| d.is_dir())
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn xdg_icon_dirs() -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut base_dirs = Vec::new();
    // Icon Theme Spec §3: $HOME/.icons first for backwards compatibility
    if let Some(home) = std::env::var_os("HOME") {
        let legacy = PathBuf::from(home).join(".icons");
        if legacy.is_dir() {
            base_dirs.push(legacy);
        }
    }
    for data_dir in xdg_data_dirs() {
        let icons = data_dir.join("icons");
        if icons.is_dir() && !base_dirs.contains(&icons) {
            base_dirs.push(icons);
        }
    }
    let mut pixmap_dirs = Vec::new();
    for p in ["/run/current-system/sw/share/pixmaps", "/usr/share/pixmaps"] {
        let pb = PathBuf::from(p);
        if pb.is_dir() && !pixmap_dirs.contains(&pb) {
            pixmap_dirs.push(pb);
        }
    }
    (base_dirs, pixmap_dirs)
}

#[cfg(all(not(target_arch = "wasm32"), not(test)))]
fn detect_posix_locale() -> String {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var)
            && !val.is_empty()
            && val != "C"
            && val != "POSIX"
        {
            return val;
        }
    }
    String::new()
}

#[cfg(all(not(target_arch = "wasm32"), not(test)))]
fn detect_current_desktops() -> Vec<String> {
    if let Ok(val) = std::env::var("XDG_CURRENT_DESKTOP") {
        let list: Vec<String> = val
            .split(':')
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        if !list.is_empty() {
            return list;
        }
    }
    vec![
        "Wisp".into(),
        "SpectreOS".into(),
        "GNOME".into(),
        "Phosh".into(),
    ]
}

#[cfg(not(target_arch = "wasm32"))]
fn detect_active_icon_themes() -> Vec<String> {
    let mut themes = Vec::new();
    if let Ok(env_theme) = std::env::var("WISP_ICON_THEME")
        && !env_theme.trim().is_empty()
    {
        themes.push(env_theme.trim().to_string());
    }
    if let Some(home) = std::env::var_os("HOME") {
        let cfg_home = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .filter(|s| !s.is_empty() && Path::new(s).is_absolute())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(home).join(".config"));
        for rel in ["gtk-4.0/settings.ini", "gtk-3.0/settings.ini"] {
            if let Ok(text) = std::fs::read_to_string(cfg_home.join(rel)) {
                for line in text.lines() {
                    let trimmed = line.trim();
                    if let Some(rest) = trimmed.strip_prefix("gtk-icon-theme-name")
                        && let Some((_, val)) = rest.split_once('=')
                    {
                        let name = val.trim().trim_matches('"').to_string();
                        if !name.is_empty() && !themes.contains(&name) {
                            themes.push(name);
                        }
                    }
                }
            }
        }
    }
    themes
}

// ============================================================================
// Desktop Entry Specification v1.5 Parsing & Field Code Expansion
// ============================================================================

/// Scan application directories in XDG precedence order, deduplicating by Desktop File ID (§2.1).
pub fn scan_desktop_dirs(
    app_dirs: &[PathBuf],
    locale: &str,
    current_desktops: &[String],
) -> Vec<DesktopApp> {
    let mut seen_ids = HashSet::new();
    let mut apps = Vec::new();

    for root in app_dirs {
        let mut files = Vec::new();
        collect_desktop_files(root, root, &mut files);
        for (desktop_id, path) in files {
            // Desktop Entry Spec §2.1: first directory in precedence order wins (even if Hidden=true!)
            if !seen_ids.insert(desktop_id.clone()) {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Some(app) =
                parse_desktop_entry(&desktop_id, &path, &content, locale, current_desktops)
            {
                apps.push(app);
            }
        }
    }

    apps.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    apps
}

fn collect_desktop_files(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_desktop_files(root, &path, out);
        } else if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| e == "desktop")
            && let Ok(rel) = path.strip_prefix(root)
        {
            // Desktop Entry Spec §2.1: relative path with '/' turned into '-'
            let id = rel
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect::<Vec<_>>()
                .join("-");
            if !id.is_empty() {
                out.push((id, path));
            }
        }
    }
}

/// Parse a `.desktop` file string per Desktop Entry Spec v1.5 (§3, §4, §5, §6, §7, §11).
pub fn parse_desktop_entry(
    desktop_id: &str,
    desktop_path: &Path,
    content: &str,
    locale: &str,
    current_desktops: &[String],
) -> Option<DesktopApp> {
    let groups = parse_ini_groups(content);
    let main = groups.get("Desktop Entry")?;

    // §6: Type is required and must be "Application"
    if main.get("Type").map(String::as_str) != Some("Application") {
        return None;
    }
    // §6: Hidden=true means deleted; NoDisplay=true means omit from launcher menus
    if parse_bool(main.get("Hidden")) || parse_bool(main.get("NoDisplay")) {
        return None;
    }
    // §6: OnlyShowIn / NotShowIn matching against XDG_CURRENT_DESKTOP
    if !desktop_visibility_allowed(
        main.get("OnlyShowIn").map(|s| split_semicolon_list(s)),
        main.get("NotShowIn").map(|s| split_semicolon_list(s)),
        current_desktops,
    ) {
        return None;
    }
    // §6: TryExec presence & executable check
    if let Some(try_exec) = main.get("TryExec").map(|s| unescape_desktop_string(s))
        && !try_exec.is_empty()
        && !is_executable_available(&try_exec)
    {
        return None;
    }

    let name = lookup_localized(main, "Name", locale)?;
    if name.is_empty() {
        return None;
    }
    let generic_name = lookup_localized(main, "GenericName", locale);
    let comment = lookup_localized(main, "Comment", locale);
    let icon = lookup_localized(main, "Icon", locale).filter(|s| !s.is_empty());
    let dbus_activatable = parse_bool(main.get("DBusActivatable"));
    let exec_raw = main.get("Exec").map(|s| unescape_desktop_string(s));
    let exec_argv = exec_raw
        .as_deref()
        .map(|raw| expand_exec_line(raw, icon.as_deref(), &name, desktop_path))
        .unwrap_or_default();

    // §6: Exec is required unless DBusActivatable=true
    if exec_argv.is_empty() && !dbus_activatable {
        return None;
    }

    let working_dir = main
        .get("Path")
        .map(|s| unescape_desktop_string(s))
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    let terminal = parse_bool(main.get("Terminal"));
    let startup_wm_class = main
        .get("StartupWMClass")
        .map(|s| unescape_desktop_string(s))
        .filter(|s| !s.is_empty());
    let categories = main
        .get("Categories")
        .map(|s| split_semicolon_list(s))
        .unwrap_or_default();
    let keywords = lookup_localized(main, "Keywords", locale)
        .map(|s| split_semicolon_list(&s))
        .unwrap_or_default();
    let shiraz_package = main
        .get("X-Shiraz-Package")
        .map(|s| unescape_desktop_string(s))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // §11: Additional application actions (`Actions=` -> `[Desktop Action <id>]`)
    let mut actions = Vec::new();
    if let Some(action_ids) = main.get("Actions") {
        for aid in split_semicolon_list(action_ids) {
            let group_key = format!("Desktop Action {aid}");
            if let Some(ag) = groups.get(&group_key)
                && let Some(aname) = lookup_localized(ag, "Name", locale)
            {
                let aicon = lookup_localized(ag, "Icon", locale).filter(|s| !s.is_empty());
                let aexec = ag
                    .get("Exec")
                    .map(|s| unescape_desktop_string(s))
                    .map(|raw| expand_exec_line(&raw, aicon.as_deref(), &aname, desktop_path))
                    .unwrap_or_default();
                if !aexec.is_empty() || dbus_activatable {
                    actions.push(DesktopAction {
                        id: aid,
                        name: aname,
                        icon: aicon,
                        exec_argv: aexec,
                    });
                }
            }
        }
    }

    Some(DesktopApp {
        id: desktop_id.to_string(),
        name,
        generic_name,
        comment,
        icon,
        exec_raw,
        exec_argv,
        working_dir,
        terminal,
        dbus_activatable,
        startup_wm_class,
        categories,
        keywords,
        desktop_path: desktop_path.to_path_buf(),
        actions,
        shiraz_package,
    })
}

#[cfg(all(not(target_arch = "wasm32"), not(test)))]
fn compute_desktop_dirs_stamp(app_dirs: &[PathBuf]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut mix = |v: u64| {
        h ^= v;
        h = h.wrapping_mul(0x100000001b3);
    };
    for dir in app_dirs {
        if let Ok(meta) = std::fs::metadata(dir)
            && let Ok(mt) = meta.modified()
            && let Ok(dur) = mt.duration_since(std::time::UNIX_EPOCH)
        {
            mix(dur.as_secs());
            mix(dur.subsec_nanos() as u64);
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut count = 0u64;
            for entry in entries.flatten() {
                count += 1;
                if let Ok(meta) = entry.metadata()
                    && let Ok(mt) = meta.modified()
                    && let Ok(dur) = mt.duration_since(std::time::UNIX_EPOCH)
                {
                    mix(dur.as_secs() ^ (dur.subsec_nanos() as u64));
                }
            }
            mix(count);
        }
    }
    h
}

fn parse_ini_groups(content: &str) -> HashMap<String, HashMap<String, String>> {
    let mut groups: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current: Option<String> = None;
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') && line.len() > 2 {
            let gname = line[1..line.len() - 1].to_string();
            groups.entry(gname.clone()).or_default();
            current = Some(gname);
            continue;
        }
        if let Some(ref gname) = current
            && let Some((k, v)) = line.split_once('=')
        {
            let key = k.trim();
            // §3.3: space around '=' ignored
            let val = v.trim_start();
            if !key.is_empty() {
                groups
                    .entry(gname.clone())
                    .or_default()
                    .entry(key.to_string())
                    .or_insert_with(|| val.to_string());
            }
        }
    }
    groups
}

/// Desktop Entry Spec §4 string unescaping (`\s`, `\n`, `\t`, `\r`, `\\`).
pub fn unescape_desktop_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('s') => out.push(' '),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Desktop Entry Spec §4 plural string(s) splitter (semicolon-separated, `\;` escaped).
pub fn split_semicolon_list(raw: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(';') => cur.push(';'),
                Some('s') => cur.push(' '),
                Some('n') => cur.push('\n'),
                Some('t') => cur.push('\t'),
                Some('r') => cur.push('\r'),
                Some('\\') => cur.push('\\'),
                Some(other) => {
                    cur.push('\\');
                    cur.push(other);
                }
                None => cur.push('\\'),
            }
        } else if c == ';' {
            let trimmed = cur.trim();
            if !trimmed.is_empty() {
                items.push(trimmed.to_string());
            }
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    let trimmed = cur.trim();
    if !trimmed.is_empty() {
        items.push(trimmed.to_string());
    }
    items
}

fn parse_bool(val: Option<&String>) -> bool {
    matches!(val.map(|s| s.trim()), Some("true" | "1"))
}

/// Desktop Entry Spec §5 Table 1 localized key lookup.
pub fn lookup_localized(
    group: &HashMap<String, String>,
    key: &str,
    locale: &str,
) -> Option<String> {
    if !locale.is_empty() {
        // Strip `.ENCODING` from `lang_COUNTRY.ENCODING@MODIFIER`
        let (before_at, modifier) = match locale.split_once('@') {
            Some((a, m)) => (a, Some(m)),
            None => (locale, None),
        };
        let without_enc = before_at.split('.').next().unwrap_or(before_at);
        let (lang, country) = match without_enc.split_once('_') {
            Some((l, c)) => (l, Some(c)),
            None => (without_enc, None),
        };

        let mut candidates = Vec::with_capacity(4);
        if let (Some(c), Some(m)) = (country, modifier) {
            candidates.push(format!("{key}[{lang}_{c}@{m}]"));
        }
        if let Some(c) = country {
            candidates.push(format!("{key}[{lang}_{c}]"));
        }
        if let Some(m) = modifier {
            candidates.push(format!("{key}[{lang}@{m}]"));
        }
        if !lang.is_empty() {
            candidates.push(format!("{key}[{lang}]"));
        }
        for cand in candidates {
            if let Some(v) = group.get(&cand) {
                return Some(unescape_desktop_string(v));
            }
        }
    }
    group.get(key).map(|v| unescape_desktop_string(v))
}

/// Desktop Entry Spec §6 `OnlyShowIn` / `NotShowIn` precedence evaluation.
pub fn desktop_visibility_allowed(
    only_show_in: Option<Vec<String>>,
    not_show_in: Option<Vec<String>>,
    current_desktops: &[String],
) -> bool {
    for desk in current_desktops {
        if let Some(ref only) = only_show_in
            && only.iter().any(|x| x == desk)
        {
            return true;
        }
        if let Some(ref not_in) = not_show_in
            && not_in.iter().any(|x| x == desk)
        {
            return false;
        }
    }
    only_show_in.is_none()
}

/// Desktop Entry Spec §7 `Exec` unquoting and field-code expansion.
pub fn expand_exec_line(
    exec: &str,
    icon: Option<&str>,
    app_name: &str,
    desktop_path: &Path,
) -> Vec<String> {
    let raw_tokens = tokenize_exec(exec);
    let mut out = Vec::with_capacity(raw_tokens.len());

    for token in raw_tokens {
        // Standalone multi-arg / drop codes (§7)
        match token.as_str() {
            "%f" | "%F" | "%u" | "%U" | "%d" | "%D" | "%n" | "%N" | "%v" | "%m" => continue,
            "%i" => {
                if let Some(ic) = icon
                    && !ic.is_empty()
                {
                    out.push("--icon".into());
                    out.push(ic.to_string());
                }
                continue;
            }
            _ => {}
        }

        // Inline field-code expansion within a single argument
        let mut expanded = String::with_capacity(token.len());
        let mut chars = token.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%' {
                match chars.next() {
                    Some('%') => expanded.push('%'),
                    Some('c') => expanded.push_str(app_name),
                    Some('k') => expanded.push_str(&desktop_path.to_string_lossy()),
                    Some('f' | 'F' | 'u' | 'U' | 'd' | 'D' | 'n' | 'N' | 'v' | 'm' | 'i') => {}
                    Some(other) => {
                        expanded.push('%');
                        expanded.push(other);
                    }
                    None => expanded.push('%'),
                }
            } else {
                expanded.push(c);
            }
        }
        if !expanded.is_empty() {
            out.push(expanded);
        }
    }
    out
}

fn tokenize_exec(exec: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut has_token = false;
    let mut chars = exec.chars();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                in_quotes = false;
            } else if c == '\\' {
                match chars.next() {
                    Some(esc @ ('"' | '`' | '$' | '\\')) => cur.push(esc),
                    Some(other) => {
                        cur.push('\\');
                        cur.push(other);
                    }
                    None => cur.push('\\'),
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
            has_token = true;
        } else if c.is_ascii_whitespace() {
            if has_token {
                tokens.push(std::mem::take(&mut cur));
                has_token = false;
            }
        } else {
            cur.push(c);
            has_token = true;
        }
    }
    if has_token {
        tokens.push(cur);
    }
    tokens
}

fn is_executable_available(cmd: &str) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let check = |p: &Path| {
            p.is_file()
                && p.metadata()
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
        };
        let path = Path::new(cmd);
        if path.is_absolute() {
            return check(path);
        }
        if let Ok(path_env) = std::env::var("PATH") {
            for dir in path_env.split(':') {
                if !dir.is_empty() && check(&Path::new(dir).join(cmd)) {
                    return true;
                }
            }
        }
        for dir in [
            "/run/current-system/sw/bin",
            "/run/wrappers/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
        ] {
            if check(&Path::new(dir).join(cmd)) {
                return true;
            }
        }
        false
    }
    #[cfg(not(unix))]
    {
        let _ = cmd;
        true
    }
}

// ============================================================================
// Icon Theme Specification v0.13 `index.theme` Parser & PNG/SVG Decoder
// ============================================================================

fn parse_icon_theme(theme_id: &str, content: &str) -> Option<IconTheme> {
    let groups = parse_ini_groups(content);
    let header = groups.get("Icon Theme")?;

    let inherits: Vec<String> = header
        .get("Inherits")
        .map(|s| {
            s.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let mut dir_names: Vec<String> = Vec::new();
    for key in ["ScaledDirectories", "Directories"] {
        if let Some(val) = header.get(key) {
            for item in val.split(',') {
                let d = item.trim();
                if !d.is_empty() && !dir_names.iter().any(|x| x == d) {
                    dir_names.push(d.to_string());
                }
            }
        }
    }

    let mut subdirs = Vec::with_capacity(dir_names.len());
    for dname in dir_names {
        let Some(dg) = groups.get(&dname) else {
            continue;
        };
        let Some(size) = dg.get("Size").and_then(|s| s.trim().parse::<u32>().ok()) else {
            continue;
        };
        let scale = dg
            .get("Scale")
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(1)
            .max(1);
        let dir_type = match dg.get("Type").map(|s| s.trim()) {
            Some("Fixed") => DirType::Fixed,
            Some("Scalable") => DirType::Scalable,
            _ => DirType::Threshold,
        };
        let min_size = dg
            .get("MinSize")
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(size);
        let max_size = dg
            .get("MaxSize")
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(size);
        let threshold = dg
            .get("Threshold")
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(2);
        subdirs.push(IconSubdir {
            path: dname,
            size,
            scale,
            dir_type,
            min_size,
            max_size,
            threshold,
        });
    }

    Some(IconTheme {
        name: theme_id.to_string(),
        inherits,
        subdirs,
    })
}

fn decode_icon_file(path: &Path, target_px: u32) -> Option<Image> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())?;
    let bytes = std::fs::read(path).ok()?;
    match ext.as_str() {
        "png" => decode_png_to_image(&bytes, target_px),
        #[cfg(feature = "svg-icons")]
        "svg" => decode_svg_to_image(&bytes, target_px),
        _ => None,
    }
}

pub(crate) fn decode_png_to_image(bytes: &[u8], max_dim: u32) -> Option<Image> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());

    let (w, h) = (info.width, info.height);
    if w == 0 || h == 0 {
        return None;
    }
    let total = (w as usize).checked_mul(h as usize)?;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity(total * 4);
            for c in buf.chunks_exact(3) {
                out.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
            out
        }
        png::ColorType::GrayscaleAlpha => {
            let mut out = Vec::with_capacity(total * 4);
            for c in buf.chunks_exact(2) {
                out.extend_from_slice(&[c[0], c[0], c[0], c[1]]);
            }
            out
        }
        png::ColorType::Grayscale => {
            let mut out = Vec::with_capacity(total * 4);
            for &g in &buf {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        _ => return None,
    };

    if w > max_dim || h > max_dim {
        let (nw, nh, scaled) = downscale_rgba_bilinear(&rgba, w, h, max_dim);
        Image::rgba(nw, nh, scaled)
    } else {
        Image::rgba(w, h, rgba)
    }
}

fn downscale_rgba_bilinear(src: &[u8], sw: u32, sh: u32, max_dim: u32) -> (u32, u32, Vec<u8>) {
    let scale = (max_dim as f32 / sw.max(sh) as f32).min(1.0);
    let dw = ((sw as f32 * scale).round() as u32).clamp(1, max_dim);
    let dh = ((sh as f32 * scale).round() as u32).clamp(1, max_dim);
    let mut dst = vec![0u8; (dw * dh * 4) as usize];

    let x_ratio = sw as f32 / dw as f32;
    let y_ratio = sh as f32 / dh as f32;
    for dy in 0..dh {
        let sy0 = ((dy as f32) * y_ratio).floor() as u32;
        let sy1 = (((dy + 1) as f32) * y_ratio).ceil().min(sh as f32) as u32;
        let sy1 = sy1.max(sy0 + 1).min(sh);
        for dx in 0..dw {
            let sx0 = ((dx as f32) * x_ratio).floor() as u32;
            let sx1 = (((dx + 1) as f32) * x_ratio).ceil().min(sw as f32) as u32;
            let sx1 = sx1.max(sx0 + 1).min(sw);
            let mut r = 0.0f32;
            let mut g = 0.0f32;
            let mut b = 0.0f32;
            let mut a = 0.0f32;
            let mut count = 0.0f32;
            for sy in sy0..sy1 {
                for sx in sx0..sx1 {
                    let o = ((sy * sw + sx) * 4) as usize;
                    let pa = src[o + 3] as f32 / 255.0;
                    r += src[o] as f32 * pa;
                    g += src[o + 1] as f32 * pa;
                    b += src[o + 2] as f32 * pa;
                    a += pa;
                    count += 1.0;
                }
            }
            let out_o = ((dy * dw + dx) * 4) as usize;
            let avg_a = (a / count).clamp(0.0, 1.0);
            if a > 1e-4 {
                dst[out_o] = (r / a).round().clamp(0.0, 255.0) as u8;
                dst[out_o + 1] = (g / a).round().clamp(0.0, 255.0) as u8;
                dst[out_o + 2] = (b / a).round().clamp(0.0, 255.0) as u8;
                dst[out_o + 3] = (avg_a * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    (dw, dh, dst)
}

#[cfg(feature = "svg-icons")]
fn decode_svg_to_image(bytes: &[u8], target_px: u32) -> Option<Image> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &opt).ok()?;
    let size = tree.size();
    if size.width() <= 0.0 || size.height() <= 0.0 {
        return None;
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_px, target_px)?;
    let scale = (target_px as f32 / size.width()).min(target_px as f32 / size.height());
    let tx = (target_px as f32 - size.width() * scale) * 0.5;
    let ty = (target_px as f32 - size.height() * scale) * 0.5;
    let ts = resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);
    resvg::render(&tree, ts, &mut pixmap.as_mut());

    // tiny-skia stores premultiplied RGBA; libbhoo expects straight RGBA in Image::rgba.
    let mut rgba = pixmap.take();
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3];
        if a > 0 && a < 255 {
            let af = 255.0 / a as f32;
            px[0] = ((px[0] as f32 * af).round() as u32).min(255) as u8;
            px[1] = ((px[1] as f32 * af).round() as u32).min(255) as u8;
            px[2] = ((px[2] as f32 * af).round() as u32).min(255) as u8;
        }
    }
    Image::rgba(target_px, target_px, rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_entry_parses_localization_actions_and_exec_field_codes() {
        let sample = r#"
[Desktop Entry]
Version=1.5
Type=Application
Name=Foo Viewer
Name[sr_YU]=Foo Pregledač
Name[sr]=Foo SR
GenericName=Image Viewer
Comment=View\sFoo\nobjects
Icon=fooview
Exec="/usr/bin/foo viewer" --title=%c %i --desktop=%k %F --pct=100%%
Path=/tmp
Terminal=false
Categories=Graphics;Viewer;
Keywords=photo;image\;pic;
Actions=Gallery;

[Desktop Action Gallery]
Name=Browse Gallery
Exec=fooview --gallery
"#;
        let app = parse_desktop_entry(
            "org.example.FooViewer.desktop",
            Path::new("/usr/share/applications/org.example.FooViewer.desktop"),
            sample,
            "sr_YU.UTF-8@Latn",
            &["Wisp".into(), "GNOME".into()],
        )
        .expect("valid desktop entry");

        assert_eq!(app.name, "Foo Pregledač");
        assert_eq!(app.comment.as_deref(), Some("View Foo\nobjects"));
        assert_eq!(app.keywords, vec!["photo", "image;pic"]);
        assert_eq!(
            app.exec_argv,
            vec![
                "/usr/bin/foo viewer",
                "--title=Foo Pregledač",
                "--icon",
                "fooview",
                "--desktop=/usr/share/applications/org.example.FooViewer.desktop",
                "--pct=100%",
            ]
        );
        assert_eq!(app.actions.len(), 1);
        assert_eq!(app.actions[0].name, "Browse Gallery");
        assert_eq!(app.actions[0].exec_argv, vec!["fooview", "--gallery"]);
    }

    #[test]
    fn desktop_entry_respects_hidden_nodisplay_and_only_not_show_in() {
        let base = "[Desktop Entry]\nType=Application\nName=Test\nExec=test-bin\n";
        assert!(
            parse_desktop_entry("a.desktop", Path::new("/a.desktop"), &format!("{base}Hidden=true\n"), "", &["Wisp".into()])
                .is_none()
        );
        assert!(
            parse_desktop_entry("a.desktop", Path::new("/a.desktop"), &format!("{base}NoDisplay=true\n"), "", &["Wisp".into()])
                .is_none()
        );
        assert!(
            parse_desktop_entry("a.desktop", Path::new("/a.desktop"), &format!("{base}OnlyShowIn=KDE;\n"), "", &["Wisp".into(), "GNOME".into()])
                .is_none()
        );
        assert!(
            parse_desktop_entry("a.desktop", Path::new("/a.desktop"), &format!("{base}OnlyShowIn=GNOME;KDE;\n"), "", &["Wisp".into(), "GNOME".into()])
                .is_some()
        );
        assert!(
            parse_desktop_entry("a.desktop", Path::new("/a.desktop"), &format!("{base}NotShowIn=Wisp;\n"), "", &["Wisp".into(), "GNOME".into()])
                .is_none()
        );
    }

    #[test]
    fn icon_theme_size_matching_and_distance() {
        let scalable = IconSubdir {
            path: "scalable/apps".into(),
            size: 48,
            scale: 1,
            dir_type: DirType::Scalable,
            min_size: 16,
            max_size: 256,
            threshold: 2,
        };
        assert!(scalable.matches_size(64, 1));
        assert!(!scalable.matches_size(64, 2));
        assert_eq!(scalable.size_distance(128, 1), 0);

        let fixed_2x = IconSubdir {
            path: "64x64@2/apps".into(),
            size: 64,
            scale: 2,
            dir_type: DirType::Fixed,
            min_size: 64,
            max_size: 64,
            threshold: 2,
        };
        assert!(fixed_2x.matches_size(64, 2));
        assert_eq!(fixed_2x.size_distance(64, 2), 0);
        assert_eq!(fixed_2x.size_distance(48, 2), 32);
    }

    #[cfg(feature = "svg-icons")]
    #[test]
    fn svg_icon_rasterizes_to_straight_rgba_image() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64">
            <rect x="8" y="8" width="48" height="48" rx="12" fill="#2563eb"/>
        </svg>"##;
        let img = decode_svg_to_image(svg, 64).expect("svg decode must succeed");
        assert_eq!(img.width(), 64);
        assert_eq!(img.height(), 64);
        // Center pixel (32, 32) should match #2563eb (0x25, 0x63, 0xeb, 0xff)
        let o = ((32 * 64 + 32) * 4) as usize;
        let px = img.pixels();
        assert_eq!(px[o], 0x25);
        assert_eq!(px[o + 1], 0x63);
        assert_eq!(px[o + 2], 0xeb);
        assert_eq!(px[o + 3], 0xff);
    }

    #[test]
    fn hicolor_user_dir_without_index_theme_falls_back_to_system_or_builtin_hicolor() {
        let base = std::env::temp_dir().join(format!("wisp-hicolor-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let user_icons = base.join("user-icons");
        let sys_icons = base.join("sys-icons");
        let app_dir = user_icons.join("hicolor/256x256/apps");
        std::fs::create_dir_all(&app_dir).unwrap();
        let icon_file = app_dir.join("net.osmand.plus.Shiraz.png");
        std::fs::write(&icon_file, b"\x89PNG\r\n\x1a\n").unwrap();

        // System dir has index.theme, user dir has NO index.theme (exact real-world layout!)
        std::fs::create_dir_all(sys_icons.join("hicolor")).unwrap();
        std::fs::write(
            sys_icons.join("hicolor/index.theme"),
            "[Icon Theme]\nName=Hicolor\nDirectories=256x256/apps\n\n[256x256/apps]\nSize=256\nType=Threshold\n",
        )
        .unwrap();

        let mut resolver = IconThemeResolver::from_dirs(
            vec![user_icons.clone(), sys_icons.clone()],
            Vec::new(),
            Vec::new(),
        );
        let found = resolver.find_icon("net.osmand.plus.Shiraz", 64, 2);
        assert_eq!(found.as_deref(), Some(icon_file.as_path()));

        // Also works even when NO system index.theme exists at all (minimal fallback hicolor)
        let mut resolver_no_sys =
            IconThemeResolver::from_dirs(vec![user_icons], Vec::new(), Vec::new());
        let found2 = resolver_no_sys.find_icon("net.osmand.plus.Shiraz", 64, 2);
        assert_eq!(found2.as_deref(), Some(icon_file.as_path()));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn parses_shiraz_package_and_matches_wayland_wmclass() {
        let content = "[Desktop Entry]\nType=Application\nVersion=1.5\nName=OsmAnd~\nExec=shiraz-launch net.osmand.plus\nIcon=net.osmand.plus.Shiraz\nStartupWMClass=net.osmand.plus.Shiraz\nX-Shiraz-Package=net.osmand.plus\n";
        let app = parse_desktop_entry(
            "net.osmand.plus.Shiraz.desktop",
            Path::new("/home/user/.local/share/applications/net.osmand.plus.Shiraz.desktop"),
            content,
            "",
            &["Wisp".into()],
        )
        .expect("valid Shiraz desktop entry");
        assert_eq!(app.shiraz_package.as_deref(), Some("net.osmand.plus"));
        assert_eq!(app.startup_wm_class.as_deref(), Some("net.osmand.plus.Shiraz"));

        let reg = DesktopRegistry {
            apps: vec![app],
            ..DesktopRegistry::default()
        };
        let matched = reg
            .find_by_wayland_app_id("net.osmand.plus.Shiraz")
            .expect("must match SDL_VIDEO_WAYLAND_WMCLASS");
        assert_eq!(matched.id, "net.osmand.plus.Shiraz.desktop");
    }
}
