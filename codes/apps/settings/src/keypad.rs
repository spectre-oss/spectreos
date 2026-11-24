//! Reusable Android 13 Material You PIN keypad & QWERTY soft keyboard widgets built on `libbhoo`.
//! Shared between SpectreOS Settings (Screen Lock setup) and Wisp's Lock Screen bouncer.

use bhoo::{Color, Frame, Input, Rect, Role, TextStyle};

const KEYPAD_ID_BASE: u64 = 0x5749_5350_0100_0000;
const ID_PIN_BASE: u64 = KEYPAD_ID_BASE + 100;
const ID_KBD_BASE: u64 = KEYPAD_ID_BASE + 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyPadEvent {
    None,
    Digit(char),
    Backspace,
    Submit,
}

/// Reusable Android 13 Material You circular numeric PIN keypad built with `libbhoo`.
pub fn paint_pin_keypad(
    f: &mut Frame<'_>,
    area: Rect,
    _input: Input,
    primary: Color,
    key_bg: Color,
    key_fg: Color,
) -> KeyPadEvent {
    let gap = 14.0;
    let btn_size = ((area.width - gap * 2.0) / 3.0)
        .min((area.height - gap * 3.0) / 4.0)
        .clamp(48.0, 72.0);
    let grid_w = btn_size * 3.0 + gap * 2.0;
    let start_x = area.x + (area.width - grid_w) * 0.5;
    let start_y = area.y;

    let mut event = KeyPadEvent::None;
    let digits = [
        ('1', 0, 0),
        ('2', 1, 0),
        ('3', 2, 0),
        ('4', 0, 1),
        ('5', 1, 1),
        ('6', 2, 1),
        ('7', 0, 2),
        ('8', 1, 2),
        ('9', 2, 2),
        ('0', 1, 3),
    ];

    for (idx, (ch, col, row)) in digits.into_iter().enumerate() {
        let rect = Rect {
            x: start_x + (col as f32) * (btn_size + gap),
            y: start_y + (row as f32) * (btn_size + gap),
            width: btn_size,
            height: btn_size,
        };
        let mut buf = [0u8; 4];
        let label = ch.encode_utf8(&mut buf);
        let r = f.target(ID_PIN_BASE + idx as u64, rect, label, true, Role::Button);
        f.paint_rect(rect, btn_size * 0.5, key_bg);
        f.state_layer(r, btn_size * 0.5, primary);
        f.styled_text(
            rect,
            label,
            key_fg,
            TextStyle {
                size: 26.0,
                weight: 500,
                align: 1,
                ..TextStyle::default()
            },
        );
        if r.clicked {
            event = KeyPadEvent::Digit(ch);
        }
    }

    // Backspace (row 3, col 0)
    let bs_rect = Rect {
        x: start_x,
        y: start_y + 3.0 * (btn_size + gap),
        width: btn_size,
        height: btn_size,
    };
    let r_bs = f.target(ID_PIN_BASE + 20, bs_rect, "Backspace", true, Role::Button);
    f.paint_rect(bs_rect, btn_size * 0.5, Color(key_bg.0, key_bg.1, key_bg.2, 200));
    f.state_layer(r_bs, btn_size * 0.5, key_fg);
    f.styled_text(
        bs_rect,
        "⌫",
        key_fg,
        TextStyle {
            size: 20.0,
            weight: 500,
            align: 1,
            ..TextStyle::default()
        },
    );
    if r_bs.clicked {
        event = KeyPadEvent::Backspace;
    }

    // Submit / Enter (row 3, col 2)
    let ok_rect = Rect {
        x: start_x + 2.0 * (btn_size + gap),
        y: start_y + 3.0 * (btn_size + gap),
        width: btn_size,
        height: btn_size,
    };
    let r_ok = f.target(ID_PIN_BASE + 21, ok_rect, "Confirm PIN", true, Role::Button);
    f.paint_rect(
        ok_rect,
        btn_size * 0.5,
        Color(primary.0, primary.1, primary.2, 250),
    );
    f.state_layer(r_ok, btn_size * 0.5, Color(16, 20, 28, 255));
    f.styled_text(
        ok_rect,
        "OK",
        Color(16, 20, 28, 255),
        TextStyle {
            size: 15.0,
            weight: 700,
            align: 1,
            ..TextStyle::default()
        },
    );
    if r_ok.clicked {
        event = KeyPadEvent::Submit;
    }

    event
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SoftKeyboardState {
    pub shift: bool,
    pub symbols: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardEvent {
    None,
    Char(char),
    Backspace,
    Submit,
}

/// Reusable Android 13 Material You on-screen QWERTY keyboard built natively with `libbhoo`.
pub fn paint_qwerty_keyboard(
    f: &mut Frame<'_>,
    area: Rect,
    input: Input,
    state: &mut SoftKeyboardState,
    primary: Color,
) -> KeyboardEvent {
    paint_qwerty_keyboard_themed(f, area, input, state, primary, true)
}

pub fn paint_qwerty_keyboard_themed(
    f: &mut Frame<'_>,
    area: Rect,
    _input: Input,
    state: &mut SoftKeyboardState,
    primary: Color,
    dark: bool,
) -> KeyboardEvent {
    let (background, key_bg, mod_bg, key_fg) = if dark {
        (Color(24, 28, 36, 248), Color(44, 50, 62, 255),
            Color(58, 66, 82, 255), Color(242, 246, 255, 255))
    } else {
        (Color(235, 234, 239, 255), Color(250, 249, 252, 255),
            Color(235, 231, 250, 255), Color(27, 26, 34, 255))
    };
    f.paint_rect(area, 16.0, background);

    let pad_x = 6.0;
    let pad_y = 8.0;
    let row_gap = 7.0;
    let key_gap = 4.5;
    let inner_w = (area.width - pad_x * 2.0).max(120.0);
    let row_h = ((area.height - pad_y * 2.0 - row_gap * 3.0) / 4.0).clamp(38.0, 50.0);

    let action_bg = if dark { primary } else { Color(229, 223, 253, 255).mix(primary, 0.20) };

    let mut event = KeyboardEvent::None;
    let mut kid = ID_KBD_BASE;

    let row1: &[char] = if state.symbols {
        &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']
    } else if state.shift {
        &['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P']
    } else {
        &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p']
    };
    let row2: &[char] = if state.symbols {
        &['@', '#', '$', '_', '&', '-', '+', '(', ')']
    } else if state.shift {
        &['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L']
    } else {
        &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l']
    };
    let row3: &[char] = if state.symbols {
        &['*', '"', '\'', ':', ';', '!', '?']
    } else if state.shift {
        &['Z', 'X', 'C', 'V', 'B', 'N', 'M']
    } else {
        &['z', 'x', 'c', 'v', 'b', 'n', 'm']
    };

    let mut paint_key = |f: &mut Frame<'_>, rect: Rect, label: &str, bg: Color, fg: Color, size: f32, radius: f32| -> bool {
        let r = f.target(kid, rect, label, true, Role::Button);
        kid += 1;
        f.paint_rect(rect, radius, bg);
        f.state_layer(r, radius, fg);
        f.styled_text(
            rect,
            label,
            fg,
            TextStyle {
                size,
                weight: 500,
                align: 1,
                ..TextStyle::default()
            },
        );
        r.clicked
    };

    // Row 1 (10 keys)
    let r1_y = area.y + pad_y;
    let kw1 = (inner_w - key_gap * 9.0) / 10.0;
    for (i, ch) in row1.iter().enumerate() {
        let rect = Rect {
            x: area.x + pad_x + (i as f32) * (kw1 + key_gap),
            y: r1_y,
            width: kw1,
            height: row_h,
        };
        let mut buf = [0u8; 4];
        if paint_key(f, rect, ch.encode_utf8(&mut buf), key_bg, key_fg, 17.5, 8.0) {
            event = KeyboardEvent::Char(*ch);
            if state.shift && !state.symbols {
                state.shift = false;
            }
        }
    }

    // Row 2 (9 keys centered)
    let r2_y = r1_y + row_h + row_gap;
    let r2_w = kw1 * 9.0 + key_gap * 8.0;
    let r2_x = area.x + (area.width - r2_w) * 0.5;
    for (i, ch) in row2.iter().enumerate() {
        let rect = Rect {
            x: r2_x + (i as f32) * (kw1 + key_gap),
            y: r2_y,
            width: kw1,
            height: row_h,
        };
        let mut buf = [0u8; 4];
        if paint_key(f, rect, ch.encode_utf8(&mut buf), key_bg, key_fg, 17.5, 8.0) {
            event = KeyboardEvent::Char(*ch);
            if state.shift && !state.symbols {
                state.shift = false;
            }
        }
    }

    // Row 3 (Shift + 7 letters + Backspace)
    let r3_y = r2_y + row_h + row_gap;
    let mod_w = kw1 * 1.45;
    let shift_rect = Rect {
        x: area.x + pad_x,
        y: r3_y,
        width: mod_w,
        height: row_h,
    };
    let shift_bg = if state.shift && !state.symbols {
        if dark { Color(primary.0, primary.1, primary.2, 240) } else { action_bg }
    } else {
        mod_bg
    };
    let shift_fg = if state.shift && !state.symbols {
        Color(16, 20, 28, 255)
    } else {
        key_fg
    };
    if paint_key(
        f,
        shift_rect,
        if state.symbols { "SYM" } else { "⇧" },
        shift_bg,
        shift_fg,
        15.0,
        8.0,
    ) && !state.symbols
    {
        state.shift = !state.shift;
    }

    let letters_w = kw1 * 7.0 + key_gap * 6.0;
    let letters_x = area.x + (area.width - letters_w) * 0.5;
    for (i, ch) in row3.iter().enumerate() {
        let rect = Rect {
            x: letters_x + (i as f32) * (kw1 + key_gap),
            y: r3_y,
            width: kw1,
            height: row_h,
        };
        let mut buf = [0u8; 4];
        if paint_key(f, rect, ch.encode_utf8(&mut buf), key_bg, key_fg, 17.5, 8.0) {
            event = KeyboardEvent::Char(*ch);
            if state.shift && !state.symbols {
                state.shift = false;
            }
        }
    }

    let bs_rect = Rect {
        x: area.x + area.width - pad_x - mod_w,
        y: r3_y,
        width: mod_w,
        height: row_h,
    };
    if paint_key(f, bs_rect, "⌫", mod_bg, key_fg, 16.5, 8.0) {
        event = KeyboardEvent::Backspace;
    }

    // Row 4 (?123 | , | Space | . | DONE)
    let r4_y = r3_y + row_h + row_gap;
    let sym_w = kw1 * 1.65;
    let sym_rect = Rect {
        x: area.x + pad_x,
        y: r4_y,
        width: sym_w,
        height: row_h,
    };
    if paint_key(
        f,
        sym_rect,
        if state.symbols { "ABC" } else { "?123" },
        mod_bg,
        key_fg,
        12.5,
        18.0,
    ) {
        state.symbols = !state.symbols;
    }

    let ok_w = kw1 * 1.85;
    let ok_rect = Rect {
        x: area.x + area.width - pad_x - ok_w,
        y: r4_y,
        width: ok_w,
        height: row_h,
    };
    if paint_key(
        f,
        ok_rect,
        if dark { "DONE" } else { "✓" },
        if dark { Color(primary.0, primary.1, primary.2, 250) } else { action_bg },
        Color(16, 20, 28, 255),
        12.5,
        18.0,
    ) {
        event = KeyboardEvent::Submit;
    }

    let dot_w = kw1 * 0.95;
    let comma_rect = Rect {
        x: sym_rect.x + sym_rect.width + key_gap,
        y: r4_y,
        width: dot_w,
        height: row_h,
    };
    if paint_key(f, comma_rect, ",", key_bg, key_fg, 16.0, 8.0) {
        event = KeyboardEvent::Char(',');
    }

    let dot_rect = Rect {
        x: ok_rect.x - key_gap - dot_w,
        y: r4_y,
        width: dot_w,
        height: row_h,
    };
    if paint_key(f, dot_rect, ".", key_bg, key_fg, 16.0, 8.0) {
        event = KeyboardEvent::Char('.');
    }

    let space_x = comma_rect.x + comma_rect.width + key_gap;
    let space_w = (dot_rect.x - key_gap - space_x).max(40.0);
    let space_rect = Rect {
        x: space_x,
        y: r4_y,
        width: space_w,
        height: row_h,
    };
    let space_fg = if dark { Color(195, 205, 224, 220) } else { key_fg.opacity(0.65) };
    if paint_key(f, space_rect, "space", key_bg, space_fg, 13.0, 8.0) {
        event = KeyboardEvent::Char(' ');
    }

    event
}
