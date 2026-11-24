//! Adaptive touch target scorer and Unicode grapheme-cluster erase rule for SpectreOS Keyboard.
//!
//! Faithfully preserves the SpectreOS Gaussian plateau + English prefix prior model:
//! - Command keys (Backspace, Shift, Enter, Symbols, Language, Space, ZWNJ) use exact hit boxes.
//! - Character keys use a central 20% flat plateau with 38% Gaussian width and optional
//!   `0.7 * ln(prior)` English prefix scoring (disabled on passwords/PINs or when
//!   `SPECTRE_ADAPTIVE_TYPING=0`).
//! - Prefix tracking is bounded to 63 ASCII bytes in RAM with zero disk or log I/O.

const VOCAB_WORDS: &str = "the there their this that these those hello help keyboard key keys touch type typing text test world work word words with what when where which would you your yes have has how home good great google gboard number numbers new next not now one only on of open phone please probability project first from for fix and are all app as at after back backspace can change create click code could do done delete did each exact english enter example function get go has here input into is it its just keep layout letters like local make model more my no other out people press right same see select separate shift should show some space stable start than them then they time to two up use version very want was we were will with would";

/// Key candidate passed to the adaptive touch scorer.
#[derive(Clone, Debug)]
pub struct TouchKey<'a> {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub symbol: &'a str,
    pub command: bool,
}

/// Extract the trailing lowercase ASCII word (up to 63 bytes) from `text`.
/// Never writes to disk or logs user input.
pub fn prefix_from_text(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut start = bytes.len();
    while start > 0 && bytes[start - 1].is_ascii_alphabetic() {
        start -= 1;
    }
    let len = bytes.len() - start;
    if len == 0 || len >= 64 {
        return String::new();
    }
    bytes[start..]
        .iter()
        .map(|b| b.to_ascii_lowercase() as char)
        .collect()
}

/// Update an in-memory typing prefix after emitting a symbol or backspace.
pub fn note_prefix_symbol(prefix: &mut String, symbol: &str, adaptive: bool) {
    if !adaptive {
        prefix.clear();
        return;
    }
    if symbol == "KEY_BACKSPACE" {
        prefix.pop();
        return;
    }
    if symbol.len() == 1 {
        let b = symbol.as_bytes()[0];
        if b.is_ascii_alphabetic() {
            if prefix.len() < 63 {
                prefix.push(b.to_ascii_lowercase() as char);
            } else {
                prefix.clear();
            }
            return;
        }
    }
    prefix.clear();
}

/// Next-letter prior for an ASCII prefix and candidate 1-byte ASCII symbol.
pub fn next_prior(prefix: &str, symbol: &str) -> f64 {
    let mut prior = 1.0_f64;
    if prefix.is_empty() || symbol.len() != 1 {
        return prior;
    }
    let next = symbol.as_bytes()[0].to_ascii_lowercase();
    let pbytes = prefix.as_bytes();
    let n = pbytes.len();
    for word in VOCAB_WORDS.split(' ') {
        let wb = word.as_bytes();
        if wb.len() > n && &wb[..n] == pbytes && wb[n] == next {
            prior += 4.0;
        }
    }
    prior
}

#[inline]
fn axis_distance(value: f64, lo: f64, hi: f64) -> f64 {
    if value < lo {
        lo - value
    } else if value > hi {
        value - hi
    } else {
        0.0
    }
}

/// Score a single key candidate for touch coordinates `(x, y)`.
pub fn touch_score(key: &TouchKey<'_>, x: f64, y: f64, prefix: &str, adaptive: bool) -> f64 {
    if !(key.w > 0.0 && key.h > 0.0) {
        return f64::NEG_INFINITY;
    }
    let cx = key.x + key.w * 0.5;
    let cy = key.y + key.h * 0.5;
    let dx = axis_distance(x, cx - key.w * 0.1, cx + key.w * 0.1);
    let dy = axis_distance(y, cy - key.h * 0.1, cy + key.h * 0.1);
    let sx = key.w * 0.38;
    let sy = key.h * 0.38;
    let mut score = -0.5 * ((dx * dx) / (sx * sx) + (dy * dy) / (sy * sy));
    if adaptive {
        score += 0.7 * next_prior(prefix, key.symbol).ln();
    }
    score
}

/// Choose the winning key index for touch point `(x, y)`.
///
/// Command keys are tested first by exact hit rectangle; letter keys compete using the
/// Gaussian plateau + optional adaptive prefix prior within `|dy| <= h` and `|dx| <= 1.5w`.
pub fn touch_choose(
    keys: &[TouchKey<'_>],
    x: f64,
    y: f64,
    prefix: &str,
    adaptive: bool,
) -> Option<usize> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    for (i, k) in keys.iter().enumerate() {
        if k.command && x >= k.x && x < k.x + k.w && y >= k.y && y < k.y + k.h {
            return Some(i);
        }
    }
    let mut best: Option<usize> = None;
    let mut best_score = f64::NEG_INFINITY;
    for (i, k) in keys.iter().enumerate() {
        if k.command
            || (y - (k.y + k.h * 0.5)).abs() > k.h
            || (x - (k.x + k.w * 0.5)).abs() > k.w * 1.5
        {
            continue;
        }
        let value = touch_score(k, x, y, prefix, adaptive);
        if value > best_score {
            best_score = value;
            best = Some(i);
        }
    }
    best
}

/// True if `c` extends the preceding Unicode grapheme (variation selector, keycap,
/// emoji Fitzpatrick skin tone, Latin or Arabic/Persian combining diacritic).
#[inline]
pub fn is_grapheme_extension(c: u32) -> bool {
    c == 0xfe0f
        || c == 0xfe0e
        || c == 0x20e3
        || (0x1f3fb..=0x1f3ff).contains(&c)
        || (0x0300..=0x036f).contains(&c)
        || (0x064b..=0x065f).contains(&c)
        || c == 0x0670
}

/// Starting codepoint index of the last grapheme cluster in `points`.
pub fn erase_start(points: &[u32]) -> usize {
    let length = points.len();
    if length == 0 {
        return 0;
    }
    let mut n = length - 1;
    if (0x1f1e6..=0x1f1ff).contains(&points[n]) {
        let mut count = 0usize;
        let mut j = length;
        while j > 0 && (0x1f1e6..=0x1f1ff).contains(&points[j - 1]) {
            count += 1;
            j -= 1;
        }
        if count % 2 == 0 && n > 0 {
            n -= 1;
        }
    } else {
        while n > 0 && is_grapheme_extension(points[n]) {
            n -= 1;
        }
        while n > 1 && points[n - 1] == 0x200d {
            n -= 2;
            while n > 0 && is_grapheme_extension(points[n]) {
                n -= 1;
            }
        }
    }
    n
}

/// Byte length of the last grapheme cluster before `cursor_byte` in `text`.
/// Returns `(byte_len, codepoint_count)`.
pub fn erase_last_grapheme_bytes(text: &str, cursor_byte: usize) -> (usize, usize) {
    let clamped = cursor_byte.min(text.len());
    let mut boundary = clamped;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    let slice = &text[..boundary];
    let chars: Vec<(usize, u32)> = slice.char_indices().map(|(i, c)| (i, c as u32)).collect();
    if chars.is_empty() {
        return (0, 0);
    }
    let points: Vec<u32> = chars.iter().map(|&(_, cp)| cp).collect();
    let start_idx = erase_start(&points);
    let start_byte = chars[start_idx].0;
    (boundary - start_byte, chars.len() - start_idx)
}

/// Check `SPECTRE_ADAPTIVE_TYPING` environment override (enabled by default unless `"0"`).
pub fn adaptive_typing_env_enabled() -> bool {
    std::env::var("SPECTRE_ADAPTIVE_TYPING")
        .map(|v| v.trim() != "0")
        .unwrap_or(true)
}
