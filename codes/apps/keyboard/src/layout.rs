//! SpectreOS Keyboard layout loader, 1080×777 reference geometry, and built-in emoji pages.

use bhoo::Rect;
use serde::Deserialize;
use std::path::Path;

pub const REF_WIDTH: f32 = 1080.0;
pub const REF_HEIGHT: f32 = 777.0;
pub const REF_KEY_AREA_HEIGHT: f32 = 630.0;
pub const REF_FOOTER_Y: f32 = 630.0;

const EMBED_US_JSON: &str = include_str!("../assets/layouts/us.json");
const EMBED_IR_JSON: &str = include_str!("../assets/layouts/ir.json");
const EMBED_TERMINAL_JSON: &str = include_str!("../assets/layouts/terminal.json");
const EMBED_GEOMETRY_JSON: &str = include_str!("../assets/reference-geometry.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutId {
    Us,
    Ir,
    Terminal,
}

impl LayoutId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Us => "us",
            Self::Ir => "ir",
            Self::Terminal => "terminal",
        }
    }

    pub fn display_badge(self) -> &'static str {
        match self {
            Self::Us => "EN",
            Self::Ir => "فا",
            Self::Terminal => "TERM",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "us" | "en" | "en-us" | "en_us" | "english" => Some(Self::Us),
            "ir" | "fa" | "fa-ir" | "fa_ir" | "persian" | "farsi" => Some(Self::Ir),
            "terminal" | "term" => Some(Self::Terminal),
            _ => None,
        }
    }

    /// Cycle primary user-facing language layouts (`us` <-> `ir`).
    pub fn next_language(self) -> Self {
        match self {
            Self::Us | Self::Terminal => Self::Ir,
            Self::Ir => Self::Us,
        }
    }
}

/// Functional kind of a keycap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyUse {
    /// Regular character / symbol key (`use = 0` or omitted).
    Character = 0,
    /// Action / delete / navigation key (`use = 1`, e.g. `KEY_BACKSPACE`, `KEY_ESC`, arrows).
    Action = 1,
    /// Layer switch toggle (`use = 2`, e.g. Shift, `?123`, `=\<`, `ABC`).
    LayerSwitch = 2,
    /// Language / layout menu switch (`use = 3`).
    Language = 3,
    /// Sticky modifier (`use = 4`, e.g. `MOD_CTRL`, `MOD_ALT` on terminal layout).
    Modifier = 4,
}

impl KeyUse {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Action,
            2 => Self::LayerSwitch,
            3 => Self::Language,
            4 => Self::Modifier,
            _ => Self::Character,
        }
    }
}

/// Visual style class matching SpectreOS keyboard design tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyStyle {
    Default,
    Persian,
    Sys,
    Toggle,
    Symbols,
    Punctuation,
    Language,
    Space,
    HalfSpace,
    Return,
}

impl KeyStyle {
    fn from_opt(s: Option<&str>) -> Self {
        match s {
            Some("persian") => Self::Persian,
            Some("sys") => Self::Sys,
            Some("toggle") => Self::Toggle,
            Some("symbols") => Self::Symbols,
            Some("punctuation") => Self::Punctuation,
            Some("language") => Self::Language,
            Some("space") => Self::Space,
            Some("halfspace") => Self::HalfSpace,
            Some("return") => Self::Return,
            _ => Self::Default,
        }
    }
}

/// Built-in vector icon drawn directly with `libbhoo` primitives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyIcon {
    Shift,
    Backspace,
    Enter,
    Globe,
    Emoji,
    Dismiss,
}

#[derive(Clone, Debug)]
pub struct KeyDef {
    pub symbol: String,
    pub label: Option<String>,
    pub width: f32,
    pub key_use: KeyUse,
    pub target_layer: Option<usize>,
    pub style: KeyStyle,
    pub icon: Option<KeyIcon>,
    /// Optional long-press alternate characters (first entry also shown as subtle corner hint).
    pub alternates: Vec<String>,
}

impl KeyDef {
    /// True if this key is a command/control key that should use exact hit boxes rather than
    /// adaptive Gaussian typing priors.
    pub fn is_command(&self) -> bool {
        self.key_use != KeyUse::Character
            || self.symbol.is_empty()
            || self.symbol.starts_with("KEY_")
            || self.symbol.starts_with("MOD_")
            || self.symbol == " "
            || self.symbol == "\u{200c}"
    }

    /// Primary display text for the keycap (empty when rendered purely as a vector icon).
    pub fn display_text(&self) -> &str {
        if let Some(ref lbl) = self.label {
            lbl.as_str()
        } else if self.icon.is_some() {
            ""
        } else {
            self.symbol.as_str()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelMode {
    Default,
    Latched,
    Locked,
}

#[derive(Clone, Debug)]
pub struct LayoutLevel {
    pub name: String,
    pub mode: LevelMode,
    pub rows: Vec<Vec<KeyDef>>,
}

#[derive(Clone, Debug)]
pub struct KeyboardLayout {
    pub id: LayoutId,
    pub name: String,
    pub locale: String,
    pub levels: Vec<LayoutLevel>,
}

impl KeyboardLayout {
    pub fn is_persian(&self) -> bool {
        self.locale.starts_with("fa")
    }

    pub fn level(&self, idx: usize) -> &LayoutLevel {
        self.levels
            .get(idx)
            .unwrap_or_else(|| &self.levels[0])
    }

    pub fn find_level(&self, name: &str) -> Option<usize> {
        self.levels.iter().position(|l| l.name == name)
    }
}

/// Measured 1080×777 reference face rectangles `[x, y, w, h]` per row/key.
#[derive(Clone, Debug, Deserialize)]
pub struct ReferenceGeometry {
    pub english: Vec<Vec<[f32; 4]>>,
    pub persian: Vec<Vec<[f32; 4]>>,
}

impl ReferenceGeometry {
    pub fn load_embedded() -> Self {
        serde_json::from_str(EMBED_GEOMETRY_JSON)
            .expect("embedded reference-geometry.json must be valid")
    }
}

#[derive(Debug, Deserialize)]
struct RawLayout {
    name: String,
    locale: String,
    levels: Vec<RawLevel>,
}

#[derive(Debug, Deserialize)]
struct RawLevel {
    level: String,
    mode: String,
    rows: Vec<Vec<RawKey>>,
}

#[derive(Debug, Deserialize)]
struct RawKey {
    #[serde(default)]
    symbol: String,
    #[serde(default = "default_key_width")]
    width: f32,
    #[serde(default, rename = "use")]
    key_use: u8,
    #[serde(default)]
    layer: Option<usize>,
    #[serde(default)]
    style: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    symbols: Vec<String>,
}

fn default_key_width() -> f32 {
    1.0
}

fn parse_icon(icon: Option<&str>, label: Option<&str>, symbol: &str) -> Option<KeyIcon> {
    match icon {
        Some("edit-clear-symbolic") => return Some(KeyIcon::Backspace),
        Some("keyboard-enter-symbolic") => return Some(KeyIcon::Enter),
        Some("layout-menu-symbolic") => return Some(KeyIcon::Globe),
        _ => {}
    }
    if label == Some("⇧") {
        return Some(KeyIcon::Shift);
    }
    if symbol == "KEY_BACKSPACE" {
        return Some(KeyIcon::Backspace);
    }
    if symbol == "KEY_ENTER" {
        return Some(KeyIcon::Enter);
    }
    None
}

fn parse_layout_json(id: LayoutId, raw_str: &str) -> Result<KeyboardLayout, String> {
    let raw: RawLayout =
        serde_json::from_str(raw_str).map_err(|e| format!("layout {}: {e}", id.as_str()))?;
    let levels = raw
        .levels
        .into_iter()
        .map(|lvl| {
            let mode = match lvl.mode.as_str() {
                "latched" => LevelMode::Latched,
                "locked" => LevelMode::Locked,
                _ => LevelMode::Default,
            };
            let rows = lvl
                .rows
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|k| {
                            let icon = parse_icon(k.icon.as_deref(), k.label.as_deref(), &k.symbol);
                            KeyDef {
                                symbol: k.symbol,
                                label: k.label,
                                width: k.width.max(0.25),
                                key_use: KeyUse::from_u8(k.key_use),
                                target_layer: k.layer,
                                style: KeyStyle::from_opt(k.style.as_deref()),
                                icon,
                                alternates: k.symbols,
                            }
                        })
                        .collect()
                })
                .collect();
            LayoutLevel {
                name: lvl.level,
                mode,
                rows,
            }
        })
        .collect();
    Ok(KeyboardLayout {
        id,
        name: raw.name,
        locale: raw.locale,
        levels,
    })
}

/// Collection of built-in (or disk-overridden) keyboard layouts plus reference geometry.
#[derive(Clone, Debug)]
pub struct LayoutCatalog {
    pub us: KeyboardLayout,
    pub ir: KeyboardLayout,
    pub terminal: KeyboardLayout,
    pub geometry: ReferenceGeometry,
}

impl LayoutCatalog {
    pub fn load() -> Self {
        let override_dir = std::env::var_os("SPECTRE_KEYBOARD_LAYOUTS_DIR")
            .map(std::path::PathBuf::from)
            .filter(|p| p.is_dir());
        let load_one = |id: LayoutId, file: &str, embedded: &str| {
            if let Some(ref dir) = override_dir {
                let path = dir.join(file);
                if let Ok(text) = std::fs::read_to_string(&path)
                    && let Ok(parsed) = parse_layout_json(id, &text)
                {
                    return parsed;
                }
            }
            parse_layout_json(id, embedded).expect("embedded layout json must be valid")
        };
        let geometry = override_dir
            .as_deref()
            .and_then(|d| {
                let p = d.join("reference-geometry.json");
                std::fs::read_to_string(&p)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
            })
            .unwrap_or_else(ReferenceGeometry::load_embedded);

        Self {
            us: load_one(LayoutId::Us, "us.json", EMBED_US_JSON),
            ir: load_one(LayoutId::Ir, "ir.json", EMBED_IR_JSON),
            terminal: load_one(LayoutId::Terminal, "terminal.json", EMBED_TERMINAL_JSON),
            geometry,
        }
    }

    pub fn load_from_dir(dir: &Path) -> Result<Self, String> {
        let read = |f: &str| {
            std::fs::read_to_string(dir.join(f))
                .map_err(|e| format!("read {}: {e}", dir.join(f).display()))
        };
        let us = parse_layout_json(LayoutId::Us, &read("us.json")?)?;
        let ir = parse_layout_json(LayoutId::Ir, &read("ir.json")?)?;
        let terminal = parse_layout_json(LayoutId::Terminal, &read("terminal.json")?)?;
        let geometry = ReferenceGeometry::load_embedded();
        Ok(Self {
            us,
            ir,
            terminal,
            geometry,
        })
    }

    pub fn get(&self, id: LayoutId) -> &KeyboardLayout {
        match id {
            LayoutId::Us => &self.us,
            LayoutId::Ir => &self.ir,
            LayoutId::Terminal => &self.terminal,
        }
    }
}

/// A laid-out keycap with both its touch hit-box and visual face rectangle in logical pixels.
#[derive(Clone, Debug)]
pub struct PlacedKey {
    pub row: usize,
    pub col: usize,
    /// Full touch allocation box (contiguous across gutters for accurate touch selection).
    pub hit_rect: Rect,
    /// Visual rounded keycap rectangle (inset slightly inside `hit_rect`).
    pub face_rect: Rect,
}

/// Total reference height (in 1080-wide reference coordinates) for a layout.
pub fn ref_total_height(layout: LayoutId) -> f32 {
    match layout {
        LayoutId::Terminal => REF_HEIGHT + 92.0,
        LayoutId::Us | LayoutId::Ir => REF_HEIGHT,
    }
}

/// Compute desired logical keyboard height for a given logical width.
/// Preserves the exact SpectreOS 1080×777 aspect ratio in portrait and caps gracefully
/// on wide / landscape screens so keys remain ergonomic rather than oversized.
pub fn keyboard_height_for_width(width: f32, layout: LayoutId) -> f32 {
    let base = (width * (ref_total_height(layout) / REF_WIDTH)).ceil();
    let max_h = match layout {
        LayoutId::Terminal => 368.0,
        LayoutId::Us | LayoutId::Ir => 332.0,
    };
    base.min(max_h).max(160.0)
}

/// Compute desired logical keyboard height inside a known screen/viewport `(width, viewport_height)`.
/// Keeps exact portrait proportions (`400×832 -> 288px`) while keeping landscape phones
/// (`832×400 -> 248px`) and tablets comfortably proportioned with room above for the active app.
pub fn keyboard_height_for_viewport(width: f32, viewport_height: f32, layout: LayoutId) -> f32 {
    let base = keyboard_height_for_width(width, layout);
    if viewport_height <= 0.0 {
        return base;
    }
    let landscape = width > viewport_height;
    let frac = match (landscape, layout) {
        (true, LayoutId::Terminal) => 0.68,
        (true, _) => 0.62,
        (false, LayoutId::Terminal) => 0.58,
        (false, _) => 0.52,
    };
    let cap = (viewport_height * frac).floor().max(150.0_f32.min(viewport_height));
    base.min(cap).max(120.0_f32.min(viewport_height))
}

/// Place every key in the active layout level for a logical keyboard deck `[width, height]`.
/// Uses independent horizontal (`width / REF_WIDTH`) and vertical (`height / ref_total_height`)
/// scaling so portrait matches the 1080×777 reference pixel-for-pixel while landscape and
/// tablet aspect ratios fit every row and footer cleanly without clipping.
pub fn place_level_keys(
    layout: &KeyboardLayout,
    level_idx: usize,
    geometry: &ReferenceGeometry,
    width: f32,
    height: f32,
) -> Vec<PlacedKey> {
    let level = layout.level(level_idx);
    let scale_x = (width / REF_WIDTH).max(0.05);
    let ref_h = ref_total_height(layout.id);
    let eff_h = if height > 0.0 { height } else { keyboard_height_for_width(width, layout.id) };
    let scale_y = (eff_h / ref_h).max(0.05);
    let mut out = Vec::with_capacity(48);

    // Terminal layout prepends a compact 92px-ref control row above the 4 measured QWERTY rows.
    let is_terminal = layout.id == LayoutId::Terminal && level.rows.len() == 5;
    let y_offset_ref = if is_terminal { 92.0 } else { 0.0 };

    if is_terminal {
        let term_row = &level.rows[0];
        let total_w: f32 = term_row.iter().map(|k| k.width).sum::<f32>().max(1.0);
        let pad_x = 14.0 * scale_x;
        let gap_x = 10.0 * scale_x;
        let usable_w = (width - pad_x * 2.0).max(1.0);
        let row_y = 14.0 * scale_y;
        let face_h = 74.0 * scale_y;
        let hit_h = 92.0 * scale_y;
        let mut cursor = pad_x;
        for (col, key) in term_row.iter().enumerate() {
            let slot_w = usable_w * (key.width / total_w);
            let face_x = cursor + gap_x * 0.5;
            let face_w = (slot_w - gap_x).max(4.0);
            out.push(PlacedKey {
                row: 0,
                col,
                hit_rect: Rect {
                    x: cursor,
                    y: 0.0,
                    width: slot_w,
                    height: hit_h,
                },
                face_rect: Rect {
                    x: face_x,
                    y: row_y,
                    width: face_w,
                    height: face_h,
                },
            });
            cursor += slot_w;
        }
    }

    let measured_rows = if layout.is_persian() {
        &geometry.persian
    } else {
        &geometry.english
    };

    let row_start = usize::from(is_terminal);
    for (local_r, row) in level.rows[row_start..].iter().enumerate() {
        let actual_r = row_start + local_r;
        let measured = measured_rows
            .get(local_r)
            .filter(|m| m.len() == row.len());

        if let Some(faces) = measured {
            for (col, face) in faces.iter().enumerate() {
                let fx = face[0] * scale_x;
                let fy = (face[1] + y_offset_ref) * scale_y;
                let fw = face[2] * scale_x;
                let fh = face[3] * scale_y;
                let hit_y = (if local_r == 3 { 464.0 } else { 10.0 + local_r as f32 * 151.0 }
                    + y_offset_ref)
                    * scale_y;
                out.push(PlacedKey {
                    row: actual_r,
                    col,
                    hit_rect: Rect {
                        x: ((face[0] - 7.0) * scale_x).max(0.0),
                        y: hit_y,
                        width: (face[2] + 14.0) * scale_x,
                        height: 151.0 * scale_y,
                    },
                    face_rect: Rect {
                        x: fx,
                        y: fy,
                        width: fw,
                        height: fh,
                    },
                });
            }
        } else {
            // Proportional fallback for extended symbol rows with different key counts.
            let row_units: f32 = row.iter().map(|k| k.width).sum::<f32>().max(1.0);
            let max_units = 10.0_f32.max(row_units);
            let margin_x = 14.0 * scale_x;
            let usable_w = (width - margin_x * 2.0).max(1.0);
            let unit_w = usable_w / max_units;
            let row_w = row_units * unit_w;
            let mut cursor = margin_x + (usable_w - row_w) * 0.5;
            let gap = 14.0 * scale_x;
            let hit_y = (10.0 + local_r as f32 * 151.0 + y_offset_ref) * scale_y;
            let face_y = (25.0 + local_r as f32 * 151.0 + y_offset_ref) * scale_y;
            let face_h = 121.0 * scale_y;
            let hit_h = 151.0 * scale_y;
            for (col, key) in row.iter().enumerate() {
                let slot_w = key.width * unit_w;
                let fx = cursor + gap * 0.5;
                let fw = (slot_w - gap).max(6.0);
                out.push(PlacedKey {
                    row: actual_r,
                    col,
                    hit_rect: Rect {
                        x: cursor,
                        y: hit_y,
                        width: slot_w,
                        height: hit_h,
                    },
                    face_rect: Rect {
                        x: fx,
                        y: face_y,
                        width: fw,
                        height: face_h,
                    },
                });
                cursor += slot_w;
            }
        }
    }

    out
}

/// Built-in emoji categories for the integrated SpectreOS emoji picker.
pub struct EmojiPage {
    pub name: &'static str,
    pub tab: &'static str,
    pub items: &'static [&'static str],
}

pub const EMOJI_PAGES: &[EmojiPage] = &[
    EmojiPage {
        name: "Smileys",
        tab: "😀",
        items: &[
            "😀", "😃", "😄", "😁", "😆", "😅", "🤣", "😂",
            "🙂", "😉", "😊", "😇", "🥰", "😍", "🤩", "😘",
            "😋", "😜", "🤪", "🤨", "🧐", "🤓", "😎", "🥳",
            "😏", "😒", "😞", "😔", "😟", "😕", "😢", "😭",
        ],
    },
    EmojiPage {
        name: "Gestures",
        tab: "👍",
        items: &[
            "👍", "👎", "👏", "🙌", "🤝", "🙏", "✌️", "🤞",
            "🤟", "🤘", "👌", "🤏", "👈", "👉", "👆", "👇",
            "👋", "🤚", "🖐️", "✋", "💪", "🫶", "❤️", "🧡",
            "💛", "💚", "💙", "💜", "🖤", "🤍", "🔥", "✨",
        ],
    },
    EmojiPage {
        name: "Objects",
        tab: "📱",
        items: &[
            "📱", "💻", "⌨️", "🖥️", "📷", "🎧", "🔋", "🔌",
            "💡", "🔒", "🔓", "🔑", "📌", "📎", "📝", "📅",
            "📁", "🔍", "⚙️", "🛠️", "🚀", "⚡", "☀️", "🌙",
            "⭐", "☁️", "🌧️", "❄️", "🎉", "✅", "❌", "⚠️",
        ],
    },
];
