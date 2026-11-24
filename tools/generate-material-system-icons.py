#!/usr/bin/env python3
"""Fetch pinned Google Material Symbols Outlined SVGs and generate 96x96 8-bit
antialiased alpha masks (`material_system.rs`) for `libbhoo::SystemIcon`.

Source: https://github.com/google/material-design-icons (Apache-2.0)
Pinned revision: 10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7
"""

from __future__ import annotations

import json
import shutil
import struct
import subprocess
import urllib.request
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ASSETS_DIR = ROOT / "codes/libs/ui/libbhoo/assets/material-system"
OUT_RS = ROOT / "codes/libs/ui/libbhoo/src/icons/material_system.rs"
REV = "10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7"
BASE_URL = (
    f"https://raw.githubusercontent.com/google/material-design-icons/{REV}"
    "/symbols/web/{name}/materialsymbolsoutlined/{name}_24px.svg"
)
ICON_SIZE = 96

# Ordered list of (const_suffix, material_symbol_name, rust_variants)
# Preserves every existing SystemIcon variant and adds high-res icons for
# Settings figures and Freedesktop / Lock Screen notifications.
ICONS: list[tuple[str, str]] = [
    ("WIFI", "wifi"),
    ("WIFI_OFF", "wifi_off"),
    ("BLUETOOTH", "bluetooth"),
    ("FLASHLIGHT_ON", "flashlight_on"),
    ("ALARM", "alarm"),
    ("AIRPLANEMODE_ACTIVE", "airplanemode_active"),
    ("HOME", "home"),
    ("SCREEN_ROTATION", "screen_rotation"),
    ("BATTERY_SAVER", "battery_saver"),
    ("DARK_MODE", "dark_mode"),
    ("DO_NOT_DISTURB_ON", "do_not_disturb_on"),
    ("CAST", "cast"),
    ("FIBER_MANUAL_RECORD", "fiber_manual_record"),
    ("BEDTIME", "bedtime"),
    ("VIDEOCAM", "videocam"),
    ("MIC", "mic"),
    ("AOD", "aod"),
    ("WIFI_TETHERING", "wifi_tethering"),
    ("LOCATION_ON", "location_on"),
    ("NFC", "nfc"),
    ("INVERT_COLORS", "invert_colors"),
    ("LIGHT_MODE", "light_mode"),
    ("BRIGHTNESS_AUTO", "brightness_auto"),
    ("POWER_SETTINGS_NEW", "power_settings_new"),
    ("RESTART_ALT", "restart_alt"),
    ("SCREENSHOT", "screenshot"),
    ("EMERGENCY", "emergency"),
    ("VIBRATION", "vibration"),
    ("NOTIFICATIONS_OFF", "notifications_off"),
    ("NOTIFICATIONS", "notifications"),
    ("MUSIC_NOTE", "music_note"),
    ("INFO", "info"),
    ("SPLITSCREEN", "splitscreen"),
    ("PICTURE_IN_PICTURE", "picture_in_picture"),
    ("OPEN_IN_FULL", "open_in_full"),
    ("CLOSE", "close"),
    ("SETTINGS", "settings"),
    ("WIDGETS", "widgets"),
    ("PALETTE", "palette"),
    ("SEARCH", "search"),
    ("ARROW_BACK", "arrow_back"),
    ("CHEVRON_RIGHT", "chevron_right"),
    ("CHEVRON_LEFT", "chevron_left"),
    ("EXPAND_MORE", "expand_more"),
    ("EXPAND_LESS", "expand_less"),
    ("EDIT", "edit"),
    ("DELETE", "delete"),
    ("BATTERY_FULL", "battery_full"),
    ("STORAGE", "storage"),
    ("VOLUME_UP", "volume_up"),
    ("ACCESSIBILITY_NEW", "accessibility_new"),
    ("LOCK", "lock"),
    ("PRIVACY_TIP", "privacy_tip"),
    ("PERSON", "person"),
    ("SMARTPHONE", "smartphone"),
    ("APPS", "apps"),
    ("USB", "usb"),
    ("HISTORY", "history"),
    ("MORE_VERT", "more_vert"),
    # New crisp Material Symbols for Settings illustrations & Lock Screen notifications
    ("VISIBILITY_OFF", "visibility_off"),
    ("LOCK_OPEN", "lock_open"),
    ("FINGERPRINT", "fingerprint"),
    ("TUNE", "tune"),
    ("NOTIFICATIONS_ACTIVE", "notifications_active"),
    ("GESTURE", "gesture"),
    ("CHECK_CIRCLE", "check_circle"),
    ("DEVICES", "devices"),
    ("LANGUAGE", "language"),
]


def read_png_alpha(png_bytes: bytes) -> tuple[int, int, bytes]:
    """Minimal lossless PNG decoder extracting the 8-bit alpha channel."""
    assert png_bytes[:8] == b"\x89PNG\r\n\x1a\n"
    pos = 8
    width = height = bit_depth = color_type = 0
    idat = bytearray()
    while pos < len(png_bytes):
        length = struct.unpack(">I", png_bytes[pos : pos + 4])[0]
        ctype = png_bytes[pos + 4 : pos + 8]
        chunk = png_bytes[pos + 8 : pos + 8 + length]
        pos += 12 + length
        if ctype == b"IHDR":
            width, height, bit_depth, color_type = struct.unpack(">IIBB", chunk[:10])
        elif ctype == b"IDAT":
            idat.extend(chunk)
        elif ctype == b"IEND":
            break
    assert bit_depth == 8 and color_type in (4, 6), (bit_depth, color_type)
    bpp = 4 if color_type == 6 else 2
    stride = width * bpp
    raw = zlib.decompress(bytes(idat))
    prev = bytearray(stride)
    alpha = bytearray(width * height)
    off = 0
    for y in range(height):
        flt = raw[off]
        off += 1
        row = bytearray(raw[off : off + stride])
        off += stride
        if flt == 1:  # Sub
            for i in range(bpp, stride):
                row[i] = (row[i] + row[i - bpp]) & 0xFF
        elif flt == 2:  # Up
            for i in range(stride):
                row[i] = (row[i] + prev[i]) & 0xFF
        elif flt == 3:  # Average
            for i in range(stride):
                left = row[i - bpp] if i >= bpp else 0
                row[i] = (row[i] + ((left + prev[i]) >> 1)) & 0xFF
        elif flt == 4:  # Paeth
            for i in range(stride):
                a = row[i - bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i - bpp] if i >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                row[i] = (row[i] + pr) & 0xFF
        elif flt != 0:
            raise ValueError(f"unsupported PNG filter {flt}")
        prev = row
        a_off = bpp - 1
        row_dst = y * width
        for x in range(width):
            alpha[row_dst + x] = row[x * bpp + a_off]
    return width, height, bytes(alpha)


def encode_rle8(mask: bytes) -> bytes:
    """Lossless RLE8 for 8-bit alpha masks:
    - 0x00..=0x7F: (b + 1) zero bytes (transparent)
    - 0x80..=0xBF: (b - 0x80 + 1) 255 bytes (fully opaque)
    - 0xC0..=0xFF: (b - 0xC0 + 1) raw literal alpha bytes follow
    """
    out = bytearray()
    n = len(mask)
    i = 0
    while i < n:
        v = mask[i]
        if v == 0:
            run = 1
            while i + run < n and mask[i + run] == 0 and run < 128:
                run += 1
            out.append(run - 1)
            i += run
        elif v == 255:
            run = 1
            while i + run < n and mask[i + run] == 255 and run < 64:
                run += 1
            out.append(0x80 + (run - 1))
            i += run
        else:
            start = i
            while i < n and mask[i] not in (0, 255) and (i - start) < 64:
                i += 1
            lit = mask[start:i]
            out.append(0xC0 + (len(lit) - 1))
            out.extend(lit)
    return bytes(out)


def decode_rle8_py(data: bytes, total: int) -> bytes:
    out = bytearray()
    i = 0
    n = len(data)
    while i < n and len(out) < total:
        b = data[i]
        i += 1
        if b < 0x80:
            take = min(b + 1, total - len(out))
            out.extend(b"\x00" * take)
        elif b < 0xC0:
            take = min((b - 0x80) + 1, total - len(out))
            out.extend(b"\xff" * take)
        else:
            count = (b - 0xC0) + 1
            take = min(count, total - len(out))
            out.extend(data[i : i + take])
            i += count
    if len(out) < total:
        out.extend(b"\x00" * (total - len(out)))
    return bytes(out)


def ensure_svg(name: str) -> Path:
    dest = ASSETS_DIR / f"{name}_24px.svg"
    if dest.is_file():
        return dest
    # Reuse existing pinned SVG if already in material-symbols
    existing = ROOT / "codes/libs/ui/libbhoo/assets/material-symbols" / f"{name}_24px.svg"
    if existing.is_file():
        shutil.copy2(existing, dest)
        return dest
    url = BASE_URL.format(name=name)
    req = urllib.request.Request(url, headers={"User-Agent": "SpectreOS-libbhoo/1.0"})
    with urllib.request.urlopen(req, timeout=20) as resp:
        svg = resp.read()
    dest.write_bytes(svg)
    return dest


def rasterize_svg_alpha(svg_path: Path, size: int) -> bytes:
    png = subprocess.check_output(
        ["rsvg-convert", "-w", str(size), "-h", str(size), str(svg_path)]
    )
    w, h, alpha = read_png_alpha(png)
    assert (w, h) == (size, size), (w, h)
    assert any(a > 0 for a in alpha), f"empty alpha for {svg_path}"
    return alpha


def format_byte_array(data: bytes) -> str:
    lines = []
    for i in range(0, len(data), 16):
        chunk = ",".join(str(b) for b in data[i : i + 16])
        lines.append(f"    {chunk},")
    return "\n".join(lines)


def main() -> None:
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)
    lic_src = ROOT / "codes/libs/ui/libbhoo/assets/material-symbols/LICENSE.txt"
    if lic_src.is_file() and not (ASSETS_DIR / "LICENSE.txt").is_file():
        shutil.copy2(lic_src, ASSETS_DIR / "LICENSE.txt")

    (ASSETS_DIR / "SOURCE.json").write_text(
        json.dumps(
            {
                "repository": "https://github.com/google/material-design-icons",
                "revision": REV,
                "style": "Material Symbols Outlined",
                "size": 24,
                "raster_size": ICON_SIZE,
                "icons": [name for _, name in ICONS],
            },
            indent=2,
        )
        + "\n"
    )

    encoded: list[tuple[str, bytes]] = []
    total_bytes = 0
    for suffix, name in ICONS:
        svg = ensure_svg(name)
        alpha = rasterize_svg_alpha(svg, ICON_SIZE)
        rle = encode_rle8(alpha)
        assert decode_rle8_py(rle, ICON_SIZE * ICON_SIZE) == alpha, f"roundtrip failed for {name}"
        encoded.append((suffix, rle))
        total_bytes += len(rle)

    rust = [
        f"//! Official Google Material Symbols Outlined {ICON_SIZE}x{ICON_SIZE} lossless RLE8 system icons ({len(ICONS)} high-res 8-bit antialiased alpha masks).",
        "//! Generated by `tools/generate-material-system-icons.py` from pinned Apache-2.0 Google Material Symbols SVGs.",
        "//! Decoded lazily and cached by quantized alpha so animations never churn GPU textures.",
        "use crate::{Color, Frame, Image, ImageFit, Rect};",
        "use std::collections::HashMap;",
        "use std::sync::{Mutex, OnceLock};",
        "",
        f"const SYS_ICON_SIZE: u32 = {ICON_SIZE};",
        f"const SYS_SVG_COUNT: usize = {len(ICONS)};",
        "",
        "/// Bundled high-resolution Google Material Symbols Outlined system icons (Quick Settings, Settings, System UI).",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]",
        "pub enum SystemIcon {",
        "    Wifi,",
        "    WifiOff,",
        "    Bluetooth,",
        "    Flashlight,",
        "    Alarm,",
        "    Airplane,",
        "    DeviceControls,",
        "    AutoRotate,",
        "    BatterySaver,",
        "    DarkMode,",
        "    Dnd,",
        "    ScreenCast,",
        "    ScreenRecord,",
        "    NightLight,",
        "    Camera,",
        "    Mic,",
        "    Aod,",
        "    Hotspot,",
        "    Location,",
        "    Nfc,",
        "    InvertColors,",
        "    Sun,",
        "    AutoBrightness,",
        "    Power,",
        "    Restart,",
        "    Screenshot,",
        "    Emergency,",
        "    Vibrate,",
        "    Silent,",
        "    Ring,",
        "    MusicNote,",
        "    Bell,",
        "    Info,",
        "    SplitScreen,",
        "    Freeform,",
        "    Maximize,",
        "    Close,",
        "    Settings,",
        "    Widgets,",
        "    Palette,",
        "    Search,",
        "    Back,",
        "    ChevronRight,",
        "    ChevronLeft,",
        "    ChevronDown,",
        "    ChevronUp,",
        "    Edit,",
        "    Trash,",
        "    Battery,",
        "    Storage,",
        "    Sound,",
        "    Display,",
        "    Accessibility,",
        "    Security,",
        "    Privacy,",
        "    Accounts,",
        "    System,",
        "    PhoneInfo,",
        "    AppsGrid,",
        "    Usb,",
        "    History,",
        "    MoreVert,",
        "    VisibilityOff,",
        "    LockOpen,",
        "    Fingerprint,",
        "    Tune,",
        "    NotificationsActive,",
        "    Gesture,",
        "    CheckCircle,",
        "    Devices,",
        "    Language,",
        "}",
        "",
        "impl SystemIcon {",
        "    /// All distinct `SystemIcon` variants.",
        "    pub const ALL: &'static [Self] = &[",
        "        Self::Wifi, Self::WifiOff, Self::Bluetooth, Self::Flashlight, Self::Alarm, Self::Airplane,",
        "        Self::DeviceControls, Self::AutoRotate, Self::BatterySaver, Self::DarkMode, Self::Dnd,",
        "        Self::ScreenCast, Self::ScreenRecord, Self::NightLight, Self::Camera, Self::Mic, Self::Aod,",
        "        Self::Hotspot, Self::Location, Self::Nfc, Self::InvertColors, Self::Sun,",
        "        Self::AutoBrightness, Self::Power, Self::Restart, Self::Screenshot, Self::Emergency,",
        "        Self::Vibrate, Self::Silent, Self::Ring, Self::MusicNote, Self::Bell, Self::Info,",
        "        Self::SplitScreen, Self::Freeform, Self::Maximize, Self::Close, Self::Settings,",
        "        Self::Widgets, Self::Palette, Self::Search, Self::Back, Self::ChevronRight,",
        "        Self::ChevronLeft, Self::ChevronDown, Self::ChevronUp, Self::Edit, Self::Trash,",
        "        Self::Battery, Self::Storage, Self::Sound, Self::Display, Self::Accessibility,",
        "        Self::Security, Self::Privacy, Self::Accounts, Self::System, Self::PhoneInfo,",
        "        Self::AppsGrid, Self::Usb, Self::History, Self::MoreVert, Self::VisibilityOff,",
        "        Self::LockOpen, Self::Fingerprint, Self::Tune, Self::NotificationsActive,",
        "        Self::Gesture, Self::CheckCircle, Self::Devices, Self::Language,",
        "    ];",
        "",
        "    pub(crate) fn mask_index(self) -> usize {",
        "        match self {",
        "            Self::Wifi => 0,",
        "            Self::WifiOff => 1,",
        "            Self::Bluetooth => 2,",
        "            Self::Flashlight => 3,",
        "            Self::Alarm => 4,",
        "            Self::Airplane => 5,",
        "            Self::DeviceControls => 6,",
        "            Self::AutoRotate => 7,",
        "            Self::BatterySaver => 8,",
        "            Self::DarkMode => 9,",
        "            Self::Dnd => 10,",
        "            Self::ScreenCast => 11,",
        "            Self::ScreenRecord => 12,",
        "            Self::NightLight => 13,",
        "            Self::Camera => 14,",
        "            Self::Mic => 15,",
        "            Self::Aod => 16,",
        "            Self::Hotspot => 17,",
        "            Self::Location => 18,",
        "            Self::Nfc => 19,",
        "            Self::InvertColors => 20,",
        "            Self::Sun | Self::Display => 21,",
        "            Self::AutoBrightness => 22,",
        "            Self::Power => 23,",
        "            Self::Restart => 24,",
        "            Self::Screenshot => 25,",
        "            Self::Emergency => 26,",
        "            Self::Vibrate => 27,",
        "            Self::Silent => 28,",
        "            Self::Ring | Self::Bell => 29,",
        "            Self::MusicNote => 30,",
        "            Self::Info | Self::System => 31,",
        "            Self::SplitScreen => 32,",
        "            Self::Freeform => 33,",
        "            Self::Maximize => 34,",
        "            Self::Close => 35,",
        "            Self::Settings => 36,",
        "            Self::Widgets => 37,",
        "            Self::Palette => 38,",
        "            Self::Search => 39,",
        "            Self::Back => 40,",
        "            Self::ChevronRight => 41,",
        "            Self::ChevronLeft => 42,",
        "            Self::ChevronDown => 43,",
        "            Self::ChevronUp => 44,",
        "            Self::Edit => 45,",
        "            Self::Trash => 46,",
        "            Self::Battery => 47,",
        "            Self::Storage => 48,",
        "            Self::Sound => 49,",
        "            Self::Accessibility => 50,",
        "            Self::Security => 51,",
        "            Self::Privacy => 52,",
        "            Self::Accounts => 53,",
        "            Self::PhoneInfo => 54,",
        "            Self::AppsGrid => 55,",
        "            Self::Usb => 56,",
        "            Self::History => 57,",
        "            Self::MoreVert => 58,",
        "            Self::VisibilityOff => 59,",
        "            Self::LockOpen => 60,",
        "            Self::Fingerprint => 61,",
        "            Self::Tune => 62,",
        "            Self::NotificationsActive => 63,",
        "            Self::Gesture => 64,",
        "            Self::CheckCircle => 65,",
        "            Self::Devices => 66,",
        "            Self::Language => 67,",
        "        }",
        "    }",
        "}",
        "",
        "/// Decode a legacy 4-bit-value / 4-bit-run-length alpha mask into `total` bytes (`0..=255`).",
        "pub fn decode_rle4(rle: &[u8], total: usize) -> Vec<u8> {",
        "    let mut mask = Vec::with_capacity(total);",
        "    for &b in rle {",
        "        let v = (b >> 4) * 17;",
        "        let run = ((b & 0x0F) as usize) + 1;",
        "        let take = run.min(total - mask.len());",
        "        mask.resize(mask.len() + take, v);",
        "        if mask.len() >= total {",
        "            break;",
        "        }",
        "    }",
        "    mask.resize(total, 0);",
        "    mask",
        "}",
        "",
        "/// Decode a lossless 8-bit antialiased alpha mask into `total` bytes (`0..=255`).",
        "pub fn decode_rle8(rle: &[u8], total: usize) -> Vec<u8> {",
        "    let mut mask = Vec::with_capacity(total);",
        "    let mut i = 0;",
        "    while i < rle.len() && mask.len() < total {",
        "        let b = rle[i];",
        "        i += 1;",
        "        if b < 0x80 {",
        "            let take = ((b as usize) + 1).min(total - mask.len());",
        "            mask.resize(mask.len() + take, 0);",
        "        } else if b < 0xC0 {",
        "            let take = (((b - 0x80) as usize) + 1).min(total - mask.len());",
        "            mask.resize(mask.len() + take, 255);",
        "        } else {",
        "            let count = ((b - 0xC0) as usize) + 1;",
        "            let avail = count.min(rle.len().saturating_sub(i));",
        "            let take = avail.min(total - mask.len());",
        "            mask.extend_from_slice(&rle[i..i + take]);",
        "            i += count;",
        "        }",
        "    }",
        "    mask.resize(total, 0);",
        "    mask",
        "}",
        "",
        f"/// Decode a tinted {ICON_SIZE}x{ICON_SIZE} RGBA `Image` for `icon`, cached by quantized alpha.",
        "pub fn system_icon_image(icon: SystemIcon, color: Color) -> Image {",
        "    let idx = icon.mask_index();",
        "    let qa = if color.3 >= 248 {",
        "        255",
        "    } else {",
        "        ((color.3 as u16 + 8) / 17 * 17).min(255) as u8",
        "    };",
        "    let key = (idx as u16, color.0, color.1, color.2, qa);",
        "    static SYS_CACHE: OnceLock<Mutex<HashMap<(u16, u8, u8, u8, u8), Image>>> = OnceLock::new();",
        "    let lock = SYS_CACHE.get_or_init(|| Mutex::new(HashMap::new()));",
        "    let mut map = lock.lock().expect(\"sys icon cache lock\");",
        "    if let Some(existing) = map.get(&key) {",
        "        return existing.clone();",
        "    }",
        "    if map.len() > 280 {",
        "        map.clear();",
        "    }",
        "    let total = (SYS_ICON_SIZE * SYS_ICON_SIZE) as usize;",
        "    let mask = decode_rle8(SYS_MASKS[idx], total);",
        "    let mut rgba = vec![0u8; total * 4];",
        "    for pi in 0..total {",
        "        let m = mask[pi];",
        "        if m > 0 {",
        "            let o = pi * 4;",
        "            rgba[o] = color.0;",
        "            rgba[o + 1] = color.1;",
        "            rgba[o + 2] = color.2;",
        "            rgba[o + 3] = ((m as u16 * qa as u16 + 127) / 255) as u8;",
        "        }",
        "    }",
        f"    let created = Image::rgba(SYS_ICON_SIZE, SYS_ICON_SIZE, rgba).expect(\"valid {ICON_SIZE}x{ICON_SIZE} sys icon\");",
        "    map.insert(key, created.clone());",
        "    created",
        "}",
        "",
        "impl Frame<'_> {",
        "    /// Paint a bundled high-resolution Material Symbols Outlined system icon tinted to `color`.",
        "    pub fn paint_system_icon(&mut self, bounds: Rect, icon: SystemIcon, color: Color) {",
        "        if color.3 <= 2 || bounds.width <= 0.5 || bounds.height <= 0.5 {",
        "            return;",
        "        }",
        "        let img = system_icon_image(icon, color);",
        "        self.paint_image(bounds, &img, ImageFit::Contain, 0.0);",
        "    }",
        "}",
        "",
    ]

    for suffix, rle in encoded:
        rust.append(f"#[rustfmt::skip]\nconst SYS_{suffix}: &[u8] = &[")
        rust.append(format_byte_array(rle))
        rust.append("];")

    rust.append("const SYS_MASKS: [&[u8]; SYS_SVG_COUNT] = [")
    for suffix, _ in encoded:
        rust.append(f"    SYS_{suffix},")
    rust.append("];")
    rust.append("")
    rust.append("#[cfg(test)]")
    rust.append('#[path = "material_system.spec.rs"]')
    rust.append("mod tests;")
    rust.append("")

    OUT_RS.write_text("\n".join(rust))
    print(f"Generated {len(encoded)} {ICON_SIZE}x{ICON_SIZE} 8-bit RLE8 icons ({total_bytes} payload bytes) -> {OUT_RS}")


if __name__ == "__main__":
    main()
