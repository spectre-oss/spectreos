//! XKB keymap synthesis and Linux evdev keycode mapping for `zwp_virtual_keyboard_v1`.
//!
//! Used for control keys (`Backspace`, `Enter`, `Tab`, `Escape`, arrows), terminal Ctrl/Alt
//! combos, and dynamic single-codepoint Unicode fallback when emitting keys to Wayland clients
//! that do not have an active `zwp_text_input_v3` session.

use crate::SpecialKey;
use std::{
    ffi::CString,
    io::Write,
    os::fd::{FromRawFd, OwnedFd},
};

/// Dynamic evdev keycode slot reserved for single-character Unicode fallback (`<I248>`).
pub const UNICODE_FALLBACK_KEYCODE: u32 = 240;

// XKB modifier masks (standard evdev/pc105 layout).
pub const MOD_SHIFT: u32 = 1 << 0;
pub const MOD_CTRL: u32 = 1 << 2;
pub const MOD_ALT: u32 = 1 << 3;

/// Base US QWERTY + control-key XKB keymap string.
pub fn build_xkb_keymap(unicode_override: Option<char>) -> String {
    let extra_sym = match unicode_override {
        Some(ch) => format!("U{:04X}", ch as u32),
        None => "NoSymbol".to_string(),
    };
    format!(
        r#"xkb_keymap {{
xkb_keycodes "(spectre)" {{
    minimum = 8;
    maximum = 255;
    <ESC>  = 9;
    <AE01> = 10; <AE02> = 11; <AE03> = 12; <AE04> = 13; <AE05> = 14;
    <AE06> = 15; <AE07> = 16; <AE08> = 17; <AE09> = 18; <AE10> = 19;
    <AE11> = 20; <AE12> = 21; <BKSP> = 22;
    <TAB>  = 23;
    <AD01> = 24; <AD02> = 25; <AD03> = 26; <AD04> = 27; <AD05> = 28;
    <AD06> = 29; <AD07> = 30; <AD08> = 31; <AD09> = 32; <AD10> = 33;
    <AD11> = 34; <AD12> = 35; <RTRN> = 36;
    <LCTL> = 37;
    <AC01> = 38; <AC02> = 39; <AC03> = 40; <AC04> = 41; <AC05> = 42;
    <AC06> = 43; <AC07> = 44; <AC08> = 45; <AC09> = 46; <AC10> = 47;
    <AC11> = 48; <TLDE> = 49;
    <LFSH> = 50; <BKSL> = 51;
    <AB01> = 52; <AB02> = 53; <AB03> = 54; <AB04> = 55; <AB05> = 56;
    <AB06> = 57; <AB07> = 58; <AB08> = 59; <AB09> = 60; <AB10> = 61;
    <LALT> = 64; <SPCE> = 65;
    <UP>   = 111; <LEFT> = 113; <RGHT> = 114; <DOWN> = 116;
    <U248> = 248;
}};
xkb_types "complete" {{
    virtual_modifiers NumLock,Alt,LevelThree;
    type "ONE_LEVEL" {{
        modifiers = none;
        level_name[Level1] = "Any";
    }};
    type "TWO_LEVEL" {{
        modifiers = Shift;
        map[Shift] = Level2;
        level_name[Level1] = "Base";
        level_name[Level2] = "Shift";
    }};
    type "ALPHABETIC" {{
        modifiers = Shift+Lock;
        map[Shift] = Level2;
        map[Lock] = Level2;
        level_name[Level1] = "Base";
        level_name[Level2] = "Caps";
    }};
}};
xkb_compatibility "complete" {{
    interpret Any+AnyOf(all) {{ action = SetMods(modifiers=modMapMods,clearLocks); }};
}};
xkb_symbols "spectre_us" {{
    key <ESC>  {{ [ Escape ] }};
    key <AE01> {{ [ 1, exclam ] }};
    key <AE02> {{ [ 2, at ] }};
    key <AE03> {{ [ 3, numbersign ] }};
    key <AE04> {{ [ 4, dollar ] }};
    key <AE05> {{ [ 5, percent ] }};
    key <AE06> {{ [ 6, asciicircum ] }};
    key <AE07> {{ [ 7, ampersand ] }};
    key <AE08> {{ [ 8, asterisk ] }};
    key <AE09> {{ [ 9, parenleft ] }};
    key <AE10> {{ [ 0, parenright ] }};
    key <AE11> {{ [ minus, underscore ] }};
    key <AE12> {{ [ equal, plus ] }};
    key <BKSP> {{ [ BackSpace ] }};
    key <TAB>  {{ [ Tab ] }};
    key <AD01> {{ [ q, Q ] }};
    key <AD02> {{ [ w, W ] }};
    key <AD03> {{ [ e, E ] }};
    key <AD04> {{ [ r, R ] }};
    key <AD05> {{ [ t, T ] }};
    key <AD06> {{ [ y, Y ] }};
    key <AD07> {{ [ u, U ] }};
    key <AD08> {{ [ i, I ] }};
    key <AD09> {{ [ o, O ] }};
    key <AD10> {{ [ p, P ] }};
    key <AD11> {{ [ bracketleft, braceleft ] }};
    key <AD12> {{ [ bracketright, braceright ] }};
    key <RTRN> {{ [ Return ] }};
    key <LCTL> {{ [ Control_L ] }};
    key <AC01> {{ [ a, A ] }};
    key <AC02> {{ [ s, S ] }};
    key <AC03> {{ [ d, D ] }};
    key <AC04> {{ [ f, F ] }};
    key <AC05> {{ [ g, G ] }};
    key <AC06> {{ [ h, H ] }};
    key <AC07> {{ [ j, J ] }};
    key <AC08> {{ [ k, K ] }};
    key <AC09> {{ [ l, L ] }};
    key <AC10> {{ [ semicolon, colon ] }};
    key <AC11> {{ [ apostrophe, quotedbl ] }};
    key <TLDE> {{ [ grave, asciitilde ] }};
    key <LFSH> {{ [ Shift_L ] }};
    key <BKSL> {{ [ backslash, bar ] }};
    key <AB01> {{ [ z, Z ] }};
    key <AB02> {{ [ x, X ] }};
    key <AB03> {{ [ c, C ] }};
    key <AB04> {{ [ v, V ] }};
    key <AB05> {{ [ b, B ] }};
    key <AB06> {{ [ n, N ] }};
    key <AB07> {{ [ m, M ] }};
    key <AB08> {{ [ comma, less ] }};
    key <AB09> {{ [ period, greater ] }};
    key <AB10> {{ [ slash, question ] }};
    key <LALT> {{ [ Alt_L ] }};
    key <SPCE> {{ [ space ] }};
    key <UP>   {{ [ Up ] }};
    key <LEFT> {{ [ Left ] }};
    key <RGHT> {{ [ Right ] }};
    key <DOWN> {{ [ Down ] }};
    key <U248> {{ [ {extra_sym} ] }};
    modifier_map Shift   {{ <LFSH> }};
    modifier_map Control {{ <LCTL> }};
    modifier_map Mod1    {{ <LALT> }};
}};
}};
"#
    )
}

/// Write a NUL-terminated XKB keymap into a fresh `memfd` for `zwp_virtual_keyboard_v1::keymap`.
pub fn create_keymap_memfd(keymap: &str) -> Result<(OwnedFd, u32), String> {
    let name = CString::new("spectre-keyboard-keymap").unwrap();
    let raw_fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
    if raw_fd < 0 {
        return Err(format!("memfd_create failed: {}", std::io::Error::last_os_error()));
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(raw_fd) };
    let bytes = keymap.as_bytes();
    file.write_all(bytes)
        .and_then(|_| file.write_all(&[0u8]))
        .map_err(|e| format!("write keymap memfd: {e}"))?;
    let size = (bytes.len() + 1) as u32;
    let fd: OwnedFd = file.into();
    Ok((fd, size))
}

/// Map a `SpecialKey` to its Linux evdev keycode (Wayland `zwp_virtual_keyboard_v1::key` uses evdev codes).
pub fn evdev_for_special(key: SpecialKey) -> u32 {
    match key {
        SpecialKey::Escape => 1,
        SpecialKey::Backspace => 14,
        SpecialKey::Tab => 15,
        SpecialKey::Enter => 28,
        SpecialKey::Up => 103,
        SpecialKey::Left => 105,
        SpecialKey::Right => 106,
        SpecialKey::Down => 108,
    }
}

/// Map a printable ASCII character to `(evdev_keycode, needs_shift)`.
pub fn evdev_for_ascii(ch: char) -> Option<(u32, bool)> {
    let pair = match ch {
        '1' => (2, false),  '!' => (2, true),
        '2' => (3, false),  '@' => (3, true),
        '3' => (4, false),  '#' => (4, true),
        '4' => (5, false),  '$' => (5, true),
        '5' => (6, false),  '%' => (6, true),
        '6' => (7, false),  '^' => (7, true),
        '7' => (8, false),  '&' => (8, true),
        '8' => (9, false),  '*' => (9, true),
        '9' => (10, false), '(' => (10, true),
        '0' => (11, false), ')' => (11, true),
        '-' => (12, false), '_' => (12, true),
        '=' => (13, false), '+' => (13, true),
        'q' => (16, false), 'Q' => (16, true),
        'w' => (17, false), 'W' => (17, true),
        'e' => (18, false), 'E' => (18, true),
        'r' => (19, false), 'R' => (19, true),
        't' => (20, false), 'T' => (20, true),
        'y' => (21, false), 'Y' => (21, true),
        'u' => (22, false), 'U' => (22, true),
        'i' => (23, false), 'I' => (23, true),
        'o' => (24, false), 'O' => (24, true),
        'p' => (25, false), 'P' => (25, true),
        '[' => (26, false), '{' => (26, true),
        ']' => (27, false), '}' => (27, true),
        'a' => (30, false), 'A' => (30, true),
        's' => (31, false), 'S' => (31, true),
        'd' => (32, false), 'D' => (32, true),
        'f' => (33, false), 'F' => (33, true),
        'g' => (34, false), 'G' => (34, true),
        'h' => (35, false), 'H' => (35, true),
        'j' => (36, false), 'J' => (36, true),
        'k' => (37, false), 'K' => (37, true),
        'l' => (38, false), 'L' => (38, true),
        ';' => (39, false), ':' => (39, true),
        '\'' => (40, false), '"' => (40, true),
        '`' => (41, false), '~' => (41, true),
        '\\' => (43, false), '|' => (43, true),
        'z' => (44, false), 'Z' => (44, true),
        'x' => (45, false), 'X' => (45, true),
        'c' => (46, false), 'C' => (46, true),
        'v' => (47, false), 'V' => (47, true),
        'b' => (48, false), 'B' => (48, true),
        'n' => (49, false), 'N' => (49, true),
        'm' => (50, false), 'M' => (50, true),
        ',' => (51, false), '<' => (51, true),
        '.' => (52, false), '>' => (52, true),
        '/' => (53, false), '?' => (53, true),
        ' ' => (57, false),
        _ => return None,
    };
    Some(pair)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_xkb_keymap_with_and_without_unicode_override() {
        let base = build_xkb_keymap(None);
        assert!(base.contains("xkb_keymap"));
        assert!(base.contains("key <BKSP>"));
        assert!(base.contains("key <U248> { [ NoSymbol ] }"));

        let fa = build_xkb_keymap(Some('ش'));
        assert!(fa.contains("key <U248> { [ U0634 ] }"));
    }

    #[test]
    fn maps_ascii_and_special_evdev_codes() {
        assert_eq!(evdev_for_special(SpecialKey::Backspace), 14);
        assert_eq!(evdev_for_special(SpecialKey::Enter), 28);
        assert_eq!(evdev_for_ascii('a'), Some((30, false)));
        assert_eq!(evdev_for_ascii('A'), Some((30, true)));
        assert_eq!(evdev_for_ascii(' '), Some((57, false)));
        assert_eq!(evdev_for_ascii('ش'), None);
    }
}
