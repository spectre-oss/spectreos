//! SpectreOS default on-screen keyboard (`spectre-keyboard`) core library.
//!
//! Provides the stateful keyboard engine, layout geometry, adaptive touch scoring,
//! `libbhoo` rendering, XKB keymap generation, Freedesktop D-Bus service (`org.spectreos.OSK0`),
//! and Wayland OSK client runtime (`zwlr_layer_shell_v1` + `zwp_input_method_v2` +
//! `zwp_virtual_keyboard_v1`).

pub mod layout;
pub mod touch;
pub mod ui;

#[cfg(feature = "osk")]
pub mod dbus;
#[cfg(feature = "osk")]
pub mod keymap;
#[cfg(feature = "osk")]
pub mod wayland;

use bhoo::{DrawCommand, Rect};
use layout::{
    EMOJI_PAGES, KeyDef, KeyIcon, KeyUse, KeyboardLayout, LayoutCatalog, LayoutId, PlacedKey,
    REF_KEY_AREA_HEIGHT, REF_WIDTH, keyboard_height_for_viewport, keyboard_height_for_width,
    place_level_keys, ref_total_height,
};
use touch::{TouchKey, adaptive_typing_env_enabled, note_prefix_symbol, prefix_from_text, touch_choose};

/// Shift / Caps Lock state for alphabetic layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftState {
    Off,
    Latched,
    Locked,
}

/// Text field semantic purpose (matches Wayland `zwp_text_input_v3::ContentPurpose`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputPurpose {
    Normal,
    Alpha,
    Digits,
    Number,
    Phone,
    Url,
    Email,
    Name,
    Password,
    Pin,
    Terminal,
}

impl InputPurpose {
    pub fn from_wayland_purpose(code: u32) -> Self {
        match code {
            1 => Self::Alpha,
            2 => Self::Digits,
            3 => Self::Number,
            4 => Self::Phone,
            5 => Self::Url,
            6 => Self::Email,
            7 => Self::Name,
            8 => Self::Password,
            9 => Self::Pin,
            13 => Self::Terminal,
            _ => Self::Normal,
        }
    }

    pub fn allows_adaptive_typing(self) -> bool {
        !matches!(self, Self::Password | Self::Pin)
    }
}

/// Control key action emitted by the keyboard engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialKey {
    Backspace,
    Enter,
    Tab,
    Escape,
    Left,
    Right,
    Up,
    Down,
}

/// Active modifier bitmask for virtual keyboard events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

/// High-level output action emitted by `KeyboardEngine`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyboardOutput {
    /// Commit UTF-8 text (English, Persian, ZWNJ, punctuation, emoji).
    CommitText { text: String, modifiers: Modifiers },
    /// Emit a control key (Backspace, Enter, Tab, Escape, arrows).
    SpecialKey { key: SpecialKey, modifiers: Modifiers },
    /// User tapped the dismiss chevron (`⌄`) on the footer bar.
    HideRequested,
    /// Active layout changed (notify D-Bus `ActiveLayout` property & resize surface if needed).
    LayoutChanged(LayoutId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FooterAction {
    ToggleLanguage,
    ToggleEmoji,
    ToggleTerminal,
    Dismiss,
}

#[derive(Clone, Debug)]
pub struct AlternatePopover {
    pub bounds: Rect,
    pub items: Vec<String>,
    pub selected: usize,
}

#[derive(Clone, Debug)]
pub struct KeyPreview {
    pub face_rect: Rect,
    pub label: String,
}

#[derive(Clone, Debug)]
struct ActivePress {
    row: usize,
    col: usize,
    pressed_at: f64,
    start_x: f32,
    last_scrub_x: f32,
    scrubbed_space: bool,
    repeat_count: u32,
    next_repeat_at: f64,
}

const LONG_PRESS_SECS: f64 = 0.34;
const REPEAT_INITIAL_SECS: f64 = 0.42;
const REPEAT_INTERVAL_SECS: f64 = 0.065;
const DOUBLE_TAP_SHIFT_SECS: f64 = 0.36;

/// Complete interactive state machine for SpectreOS Keyboard.
pub struct KeyboardEngine {
    pub catalog: LayoutCatalog,
    pub layout_id: LayoutId,
    pub preferred_language: LayoutId,
    pub level_idx: usize,
    pub shift_state: ShiftState,
    pub ctrl_latched: bool,
    pub alt_latched: bool,
    pub emoji_open: bool,
    pub emoji_page: usize,
    pub pressed_emoji: Option<usize>,
    pub pressed_footer: Option<FooterAction>,
    pub popover: Option<AlternatePopover>,
    pub purpose: InputPurpose,
    pub adaptive_enabled: bool,
    pub typing_prefix: String,
    active_press: Option<ActivePress>,
    last_shift_tap: Option<f64>,
}

impl KeyboardEngine {
    pub fn new(initial_layout: LayoutId) -> Self {
        let catalog = LayoutCatalog::load();
        let preferred_language = match initial_layout {
            LayoutId::Ir => LayoutId::Ir,
            _ => LayoutId::Us,
        };
        Self {
            catalog,
            layout_id: initial_layout,
            preferred_language,
            level_idx: 0,
            shift_state: ShiftState::Off,
            ctrl_latched: false,
            alt_latched: false,
            emoji_open: false,
            emoji_page: 0,
            pressed_emoji: None,
            pressed_footer: None,
            popover: None,
            purpose: InputPurpose::Normal,
            adaptive_enabled: adaptive_typing_env_enabled(),
            typing_prefix: String::new(),
            active_press: None,
            last_shift_tap: None,
        }
    }

    pub fn active_layout(&self) -> &KeyboardLayout {
        self.catalog.get(self.layout_id)
    }

    pub fn desired_height(&self, width: f32) -> f32 {
        keyboard_height_for_width(width, self.layout_id)
    }

    pub fn desired_height_for_viewport(&self, width: f32, viewport_height: f32) -> f32 {
        keyboard_height_for_viewport(width, viewport_height, self.layout_id)
    }

    pub fn footer_y(&self, width: f32, height: f32) -> f32 {
        let eff_h = if height > 0.0 { height } else { self.desired_height(width) };
        let scale_y = (eff_h / ref_total_height(self.layout_id)).max(0.05);
        let extra = if self.layout_id == LayoutId::Terminal {
            92.0 * scale_y
        } else {
            0.0
        };
        REF_KEY_AREA_HEIGHT * scale_y + extra
    }

    pub fn placed_keys(&self, width: f32, height: f32) -> Vec<PlacedKey> {
        place_level_keys(
            self.active_layout(),
            self.level_idx,
            &self.catalog.geometry,
            width,
            height,
        )
    }

    pub fn is_key_pressed(&self, row: usize, col: usize) -> bool {
        self.active_press
            .as_ref()
            .is_some_and(|p| p.row == row && p.col == col)
    }

    pub fn pressed_key_preview(&self, width: f32, height: f32) -> Option<KeyPreview> {
        let press = self.active_press.as_ref()?;
        if press.scrubbed_space {
            return None;
        }
        let level = self.active_layout().level(self.level_idx);
        let key = level.rows.get(press.row)?.get(press.col)?;
        if key.is_command() || key.icon.is_some() {
            return None;
        }
        let label = key.display_text();
        if label.is_empty() {
            return None;
        }
        let placed = self.placed_keys(width, height);
        let pk = ui::placed_key_for(&placed, press.row, press.col)?;
        Some(KeyPreview {
            face_rect: pk.face_rect,
            label: label.to_string(),
        })
    }

    /// Update surrounding text & semantic purpose from `zwp_input_method_v2`.
    pub fn set_input_context(&mut self, surrounding: Option<&str>, purpose: InputPurpose) -> Option<KeyboardOutput> {
        let prev_layout = self.layout_id;
        self.purpose = purpose;
        let allow_adaptive = self.adaptive_enabled && purpose.allows_adaptive_typing();
        if allow_adaptive {
            if let Some(text) = surrounding {
                self.typing_prefix = prefix_from_text(text);
            }
        } else {
            self.typing_prefix.clear();
        }

        match purpose {
            InputPurpose::Terminal => {
                self.layout_id = LayoutId::Terminal;
                self.level_idx = 0;
                self.shift_state = ShiftState::Off;
            }
            InputPurpose::Digits | InputPurpose::Number | InputPurpose::Phone | InputPurpose::Pin => {
                self.layout_id = self.preferred_language;
                if let Some(opt_idx) = self.active_layout().find_level("opt") {
                    self.level_idx = opt_idx;
                }
                self.shift_state = ShiftState::Off;
            }
            _ => {
                if self.layout_id == LayoutId::Terminal {
                    self.layout_id = self.preferred_language;
                }
                self.level_idx = 0;
                self.shift_state = ShiftState::Off;
            }
        }
        if self.layout_id != prev_layout {
            Some(KeyboardOutput::LayoutChanged(self.layout_id))
        } else {
            None
        }
    }

    /// Switch layout explicitly (e.g. from D-Bus `SetLayout` or footer toggle).
    pub fn set_layout(&mut self, id: LayoutId) -> KeyboardOutput {
        self.layout_id = id;
        if id != LayoutId::Terminal {
            self.preferred_language = id;
        }
        self.level_idx = 0;
        self.shift_state = ShiftState::Off;
        self.ctrl_latched = false;
        self.alt_latched = false;
        self.emoji_open = false;
        self.cancel_press();
        self.typing_prefix.clear();
        KeyboardOutput::LayoutChanged(id)
    }

    pub fn cancel_press(&mut self) {
        self.active_press = None;
        self.popover = None;
        self.pressed_footer = None;
        self.pressed_emoji = None;
    }

    /// Returns the next monotonic timestamp (seconds) when `tick()` needs to wake up
    /// for long-press alternate popovers or Backspace/arrow auto-repeat.
    pub fn next_timer_deadline(&self) -> Option<f64> {
        let press = self.active_press.as_ref()?;
        let level = self.active_layout().level(self.level_idx);
        let key = level.rows.get(press.row)?.get(press.col)?;
        if is_repeatable_key(key) {
            return Some(press.next_repeat_at);
        }
        if self.popover.is_none() && !key.alternates.is_empty() && !press.scrubbed_space {
            return Some(press.pressed_at + LONG_PRESS_SECS);
        }
        None
    }

    /// Advance timers (long-press popover activation and repeatable key emission).
    pub fn tick(&mut self, now: f64, width: f32, height: f32) -> Vec<KeyboardOutput> {
        let mut out = Vec::new();
        let Some(ref mut press) = self.active_press else {
            return out;
        };
        let row = press.row;
        let col = press.col;
        let pressed_at = press.pressed_at;
        let next_repeat_at = press.next_repeat_at;
        let scrubbed = press.scrubbed_space;

        let level = self.active_layout().level(self.level_idx);
        let Some(key) = level.rows.get(row).and_then(|r| r.get(col)).cloned() else {
            return out;
        };

        if is_repeatable_key(&key) {
            if now >= next_repeat_at {
                if let Some(ref mut p) = self.active_press {
                    p.repeat_count += 1;
                    p.next_repeat_at = now + REPEAT_INTERVAL_SECS;
                }
                self.emit_key(&key, now, &mut out);
            }
            return out;
        }

        if self.popover.is_none()
            && !scrubbed
            && !key.alternates.is_empty()
            && now >= pressed_at + LONG_PRESS_SECS
        {
            let eff_h = if height > 0.0 { height } else { self.desired_height(width) };
            let placed = self.placed_keys(width, eff_h);
            if let Some(pk) = ui::placed_key_for(&placed, row, col) {
                let mut items = Vec::with_capacity(key.alternates.len() + 1);
                items.push(key.symbol.clone());
                items.extend(key.alternates.iter().cloned());
                let scale = (width / REF_WIDTH).min(eff_h / ref_total_height(self.layout_id));
                let cell_w = (98.0 * scale).max(34.0);
                let pop_w = (cell_w * items.len() as f32).min(width - 16.0 * scale);
                let pop_h = (122.0 * scale).max(40.0);
                let pop_x = (pk.face_rect.x + (pk.face_rect.width - pop_w) * 0.5)
                    .clamp(8.0 * scale, (width - pop_w - 8.0 * scale).max(8.0 * scale));
                let pop_y = (pk.face_rect.y - pop_h - 10.0 * scale).max(4.0 * scale);
                self.popover = Some(AlternatePopover {
                    bounds: Rect {
                        x: pop_x,
                        y: pop_y,
                        width: pop_w,
                        height: pop_h,
                    },
                    items,
                    selected: 1.min(key.alternates.len()),
                });
            }
        }
        out
    }

    pub fn pointer_down(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        now: f64,
    ) -> Vec<KeyboardOutput> {
        let mut out = Vec::new();
        self.cancel_press();
        let eff_h = if height > 0.0 { height } else { self.desired_height(width) };

        if let Some(footer_action) = ui::footer_hit_test(self, x, y, width, eff_h) {
            self.pressed_footer = Some(footer_action);
            return out;
        }

        if self.emoji_open {
            self.handle_emoji_down(x, y, width, eff_h, &mut out);
            return out;
        }

        if let Some((row, col)) = self.hit_test_key(x, y, width, eff_h) {
            self.active_press = Some(ActivePress {
                row,
                col,
                pressed_at: now,
                start_x: x,
                last_scrub_x: x,
                scrubbed_space: false,
                repeat_count: 0,
                next_repeat_at: now + REPEAT_INITIAL_SECS,
            });
        }
        out
    }

    pub fn pointer_move(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        _height: f32,
        _now: f64,
    ) -> Vec<KeyboardOutput> {
        let mut out = Vec::new();

        // Update hovered item in long-press alternate popover.
        if let Some(ref mut pop) = self.popover {
            let count = pop.items.len().max(1);
            let rel = ((x - pop.bounds.x) / pop.bounds.width).clamp(0.0, 0.9999);
            pop.selected = (rel * count as f32) as usize;
            return out;
        }

        // Horizontal slide on the spacebar scrubs cursor Left / Right cleanly.
        let is_space = self.active_press.as_ref().is_some_and(|press| {
            let level = self.active_layout().level(self.level_idx);
            level
                .rows
                .get(press.row)
                .and_then(|r| r.get(press.col))
                .is_some_and(|key| key.style == layout::KeyStyle::Space)
        });
        if is_space && let Some(ref mut press) = self.active_press {
            let step = (width * 0.045).max(16.0);
            let dx = x - press.last_scrub_x;
            if (x - press.start_x).abs() >= step * 0.85 {
                press.scrubbed_space = true;
            }
            if press.scrubbed_space && dx.abs() >= step {
                let steps = (dx.abs() / step) as usize;
                let dir = if dx > 0.0 {
                    SpecialKey::Right
                } else {
                    SpecialKey::Left
                };
                press.last_scrub_x += (steps as f32) * step * dx.signum();
                for _ in 0..steps.min(6) {
                    out.push(KeyboardOutput::SpecialKey {
                        key: dir,
                        modifiers: Modifiers::default(),
                    });
                }
            }
            let _ = y;
        }
        out
    }

    pub fn pointer_up(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        now: f64,
    ) -> Vec<KeyboardOutput> {
        let mut out = Vec::new();
        let eff_h = if height > 0.0 { height } else { self.desired_height(width) };

        if let Some(footer_action) = self.pressed_footer.take() {
            if ui::footer_hit_test(self, x, y, width, eff_h) == Some(footer_action) {
                match footer_action {
                    FooterAction::ToggleLanguage => {
                        let next = self.layout_id.next_language();
                        out.push(self.set_layout(next));
                    }
                    FooterAction::ToggleEmoji => {
                        self.emoji_open = !self.emoji_open;
                    }
                    FooterAction::ToggleTerminal => {
                        let target = if self.layout_id == LayoutId::Terminal {
                            self.preferred_language
                        } else {
                            LayoutId::Terminal
                        };
                        out.push(self.set_layout(target));
                    }
                    FooterAction::Dismiss => {
                        out.push(KeyboardOutput::HideRequested);
                    }
                }
            }
            return out;
        }

        if self.emoji_open {
            if let Some(idx) = self.pressed_emoji.take() {
                let page = &EMOJI_PAGES[self.emoji_page.min(EMOJI_PAGES.len() - 1)];
                if let Some(&emoji) = page.items.get(idx) {
                    self.typing_prefix.clear();
                    out.push(KeyboardOutput::CommitText {
                        text: emoji.to_string(),
                        modifiers: Modifiers::default(),
                    });
                }
            }
            return out;
        }

        // Commit selected alternate if long-press popover is open.
        if let Some(pop) = self.popover.take() {
            self.active_press = None;
            if let Some(chosen) = pop.items.get(pop.selected) {
                if chosen == "☺" {
                    self.emoji_open = true;
                } else {
                    self.commit_text_and_unlatch(chosen, &mut out);
                }
            }
            return out;
        }

        let Some(press) = self.active_press.take() else {
            return out;
        };
        if press.scrubbed_space || press.repeat_count > 0 {
            return out;
        }

        let level = self.active_layout().level(self.level_idx);
        if let Some(key) = level.rows.get(press.row).and_then(|r| r.get(press.col)).cloned() {
            self.emit_key(&key, now, &mut out);
        }
        out
    }

    fn handle_emoji_down(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        out: &mut Vec<KeyboardOutput>,
    ) {
        let scale_x = width / REF_WIDTH;
        let scale_y = (height / ref_total_height(self.layout_id)).max(0.05);
        let tab_h = 92.0 * scale_y;
        let footer_y = self.footer_y(width, height);
        let pad = 16.0 * scale_x;

        if y < tab_h {
            let tab_count = EMOJI_PAGES.len() + 2;
            let tab_w = (width - pad * 2.0) / tab_count as f32;
            let idx = ((x - pad) / tab_w).floor() as isize;
            if idx == 0 {
                self.emoji_open = false;
            } else if idx == (tab_count as isize - 1) {
                out.push(KeyboardOutput::SpecialKey {
                    key: SpecialKey::Backspace,
                    modifiers: Modifiers::default(),
                });
            } else if idx >= 1 && (idx as usize) <= EMOJI_PAGES.len() {
                self.emoji_page = (idx as usize) - 1;
            }
            return;
        }

        let grid_y = tab_h + 6.0 * scale_y;
        let grid_h = (footer_y - tab_h - 12.0 * scale_y).max(40.0);
        if y >= grid_y && y < grid_y + grid_h {
            let cols = 8usize;
            let rows = 4usize;
            let cell_w = (width - pad * 2.0) / cols as f32;
            let cell_h = grid_h / rows as f32;
            let c = ((x - pad) / cell_w).floor() as isize;
            let r = ((y - grid_y) / cell_h).floor() as isize;
            if c >= 0 && (c as usize) < cols && r >= 0 && (r as usize) < rows {
                self.pressed_emoji = Some((r as usize) * cols + (c as usize));
            }
        }
    }

    fn hit_test_key(&self, x: f32, y: f32, width: f32, height: f32) -> Option<(usize, usize)> {
        if x < 0.0 || x > width || y < 0.0 || y >= self.footer_y(width, height) {
            return None;
        }
        let placed = self.placed_keys(width, height);
        let level = self.active_layout().level(self.level_idx);
        let mut candidates = Vec::with_capacity(placed.len());
        let mut coords = Vec::with_capacity(placed.len());
        for pk in &placed {
            let Some(key) = level.rows.get(pk.row).and_then(|r| r.get(pk.col)) else {
                continue;
            };
            candidates.push(TouchKey {
                x: pk.hit_rect.x as f64,
                y: pk.hit_rect.y as f64,
                w: pk.hit_rect.width as f64,
                h: pk.hit_rect.height as f64,
                symbol: key.symbol.as_str(),
                command: key.is_command(),
            });
            coords.push((pk.row, pk.col));
        }
        let adaptive = self.adaptive_enabled
            && self.purpose.allows_adaptive_typing()
            && !self.active_layout().is_persian()
            && self.level_idx <= 1;
        let chosen = touch_choose(&candidates, x as f64, y as f64, &self.typing_prefix, adaptive)?;
        Some(coords[chosen])
    }

    fn emit_key(&mut self, key: &KeyDef, now: f64, out: &mut Vec<KeyboardOutput>) {
        match key.key_use {
            KeyUse::LayerSwitch => {
                self.handle_layer_switch(key, now);
                return;
            }
            KeyUse::Language => {
                let next = self.layout_id.next_language();
                out.push(self.set_layout(next));
                return;
            }
            KeyUse::Modifier => {
                match key.symbol.as_str() {
                    "MOD_CTRL" => self.ctrl_latched = !self.ctrl_latched,
                    "MOD_ALT" => self.alt_latched = !self.alt_latched,
                    _ => {}
                }
                return;
            }
            KeyUse::Action | KeyUse::Character => {}
        }

        let mods = self.take_sticky_modifiers();
        match key.symbol.as_str() {
            "KEY_BACKSPACE" => {
                note_prefix_symbol(&mut self.typing_prefix, "KEY_BACKSPACE", self.adaptive_enabled);
                out.push(KeyboardOutput::SpecialKey {
                    key: SpecialKey::Backspace,
                    modifiers: mods,
                });
            }
            "KEY_ENTER" => {
                self.typing_prefix.clear();
                out.push(KeyboardOutput::SpecialKey {
                    key: SpecialKey::Enter,
                    modifiers: mods,
                });
            }
            "KEY_TAB" => {
                self.typing_prefix.clear();
                out.push(KeyboardOutput::SpecialKey {
                    key: SpecialKey::Tab,
                    modifiers: mods,
                });
            }
            "KEY_ESC" => {
                self.typing_prefix.clear();
                out.push(KeyboardOutput::SpecialKey {
                    key: SpecialKey::Escape,
                    modifiers: mods,
                });
            }
            "KEY_LEFT" => out.push(KeyboardOutput::SpecialKey {
                key: SpecialKey::Left,
                modifiers: mods,
            }),
            "KEY_RIGHT" => out.push(KeyboardOutput::SpecialKey {
                key: SpecialKey::Right,
                modifiers: mods,
            }),
            "KEY_UP" => out.push(KeyboardOutput::SpecialKey {
                key: SpecialKey::Up,
                modifiers: mods,
            }),
            "KEY_DOWN" => out.push(KeyboardOutput::SpecialKey {
                key: SpecialKey::Down,
                modifiers: mods,
            }),
            sym if !sym.is_empty() => {
                self.commit_text_with_mods(sym, mods, out);
            }
            _ => {}
        }
    }

    fn take_sticky_modifiers(&mut self) -> Modifiers {
        let mods = Modifiers {
            ctrl: self.ctrl_latched,
            alt: self.alt_latched,
            shift: false,
        };
        self.ctrl_latched = false;
        self.alt_latched = false;
        mods
    }

    fn commit_text_and_unlatch(&mut self, sym: &str, out: &mut Vec<KeyboardOutput>) {
        let mods = self.take_sticky_modifiers();
        self.commit_text_with_mods(sym, mods, out);
    }

    fn commit_text_with_mods(&mut self, sym: &str, modifiers: Modifiers, out: &mut Vec<KeyboardOutput>) {
        note_prefix_symbol(
            &mut self.typing_prefix,
            sym,
            self.adaptive_enabled && self.purpose.allows_adaptive_typing(),
        );
        out.push(KeyboardOutput::CommitText {
            text: sym.to_string(),
            modifiers,
        });
        if self.shift_state == ShiftState::Latched {
            self.shift_state = ShiftState::Off;
            self.level_idx = 0;
        }
    }

    fn handle_layer_switch(&mut self, key: &KeyDef, now: f64) {
        let layout = self.active_layout();
        let cur_name = layout.level(self.level_idx).name.as_str();
        let target_layer = key.target_layer.unwrap_or(0);

        // Shift key on alphabetic layouts (`layer == 1`, `icon == Shift`).
        if key.icon == Some(KeyIcon::Shift) {
            let shift_idx = layout.find_level("shift");
            match self.shift_state {
                ShiftState::Off => {
                    if let Some(idx) = shift_idx {
                        self.level_idx = idx;
                        self.shift_state = ShiftState::Latched;
                        self.last_shift_tap = Some(now);
                    }
                }
                ShiftState::Latched => {
                    if self.last_shift_tap.is_some_and(|t| (now - t) <= DOUBLE_TAP_SHIFT_SECS) {
                        self.shift_state = ShiftState::Locked;
                        self.last_shift_tap = None;
                    } else {
                        self.shift_state = ShiftState::Off;
                        self.level_idx = 0;
                        self.last_shift_tap = None;
                    }
                }
                ShiftState::Locked => {
                    self.shift_state = ShiftState::Off;
                    self.level_idx = 0;
                    self.last_shift_tap = None;
                }
            }
            return;
        }

        // `?123` <-> `ABC` / `؟۳۲۱` <-> `اب‌ج` (`layer == 2`).
        if target_layer == 2 {
            if cur_name == "opt" || cur_name == "opt+shift" {
                self.level_idx = 0;
            } else if let Some(idx) = layout.find_level("opt") {
                self.level_idx = idx;
            }
            self.shift_state = ShiftState::Off;
            return;
        }

        // `=\<` <-> `?123` (`layer == 1` inside symbol layers).
        if target_layer == 1 {
            if cur_name == "opt" {
                if let Some(idx) = layout.find_level("opt+shift") {
                    self.level_idx = idx;
                }
            } else if cur_name == "opt+shift"
                && let Some(idx) = layout.find_level("opt")
            {
                self.level_idx = idx;
            }
            self.shift_state = ShiftState::Off;
        }
    }

    /// Render the keyboard into `bhoo::DrawCommand`s.
    pub fn render(&self, bounds: Rect, commands: &mut Vec<DrawCommand>) {
        ui::render_keyboard(self, bounds, commands);
    }
}

fn is_repeatable_key(key: &KeyDef) -> bool {
    matches!(
        key.symbol.as_str(),
        "KEY_BACKSPACE" | "KEY_LEFT" | "KEY_RIGHT" | "KEY_UP" | "KEY_DOWN"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_layouts_and_reference_geometry() {
        let cat = LayoutCatalog::load();
        assert_eq!(cat.us.locale, "en-US");
        assert_eq!(cat.us.levels.len(), 4);
        assert_eq!(cat.ir.locale, "fa-IR");
        assert_eq!(cat.ir.levels.len(), 3);
        assert_eq!(cat.terminal.levels[0].rows.len(), 5);
        assert_eq!(cat.geometry.english.len(), 4);
        assert_eq!(cat.geometry.persian.len(), 4);
        assert_eq!(cat.geometry.persian[0].len(), 11);
    }

    #[test]
    fn adaptive_touch_prefers_expected_english_next_letter_and_disables_for_passwords() {
        let keys = [
            TouchKey { x: 0.0, y: 0.0, w: 100.0, h: 100.0, symbol: "h", command: false },
            TouchKey { x: 100.0, y: 0.0, w: 100.0, h: 100.0, symbol: "j", command: false },
        ];
        // Boundary point slightly toward 'j', but prefix "t" strongly predicts "th..." ("the", "this", ...).
        assert_eq!(touch::touch_choose(&keys, 102.0, 50.0, "t", true), Some(0));
        // Without adaptive priors (e.g. password field), exact geometry picks 'j'.
        assert_eq!(touch::touch_choose(&keys, 102.0, 50.0, "t", false), Some(1));
    }

    #[test]
    fn grapheme_erase_handles_zwj_flags_and_persian_diacritics() {
        // Family ZWJ emoji is one grapheme cluster.
        let family = "a👨‍👩‍👧";
        let (bytes, cps) = touch::erase_last_grapheme_bytes(family, family.len());
        assert_eq!(bytes, family.len() - 1);
        assert_eq!(cps, 5);

        // Regional indicator flag pair is one grapheme cluster.
        let flag = "hi🇮🇷";
        let (fbytes, fcps) = touch::erase_last_grapheme_bytes(flag, flag.len());
        assert_eq!(fbytes, flag.len() - 2);
        assert_eq!(fcps, 2);

        // Persian letter with combining fatha (`بَ`) erases together.
        let fa = "س\u{0628}\u{064e}";
        let (pbytes, pcps) = touch::erase_last_grapheme_bytes(fa, fa.len());
        assert_eq!(pbytes, "\u{0628}\u{064e}".len());
        assert_eq!(pcps, 2);
    }

    #[test]
    fn shift_latch_caps_lock_symbols_and_persian_zwnj_work() {
        let mut eng = KeyboardEngine::new(LayoutId::Us);
        let w = 400.0;
        let h = eng.desired_height(w);
        let placed = eng.placed_keys(w, h);

        // Find Shift key (row 2, col 0) and tap it once -> Latched Shift.
        let shift = ui::placed_key_for(&placed, 2, 0).unwrap();
        let sx = shift.face_rect.x + shift.face_rect.width * 0.5;
        let sy = shift.face_rect.y + shift.face_rect.height * 0.5;
        eng.pointer_down(sx, sy, w, h, 1.0);
        eng.pointer_up(sx, sy, w, h, 1.05);
        assert_eq!(eng.shift_state, ShiftState::Latched);
        assert_eq!(eng.level_idx, 1);

        // Tap 'Q' (row 0, col 0) -> emits "Q" and unlatches back to lowercase.
        let q = ui::placed_key_for(&placed, 0, 0).unwrap();
        let qx = q.face_rect.x + q.face_rect.width * 0.5;
        let qy = q.face_rect.y + q.face_rect.height * 0.5;
        eng.pointer_down(qx, qy, w, h, 1.2);
        let out = eng.pointer_up(qx, qy, w, h, 1.25);
        assert_eq!(
            out,
            vec![KeyboardOutput::CommitText {
                text: "Q".into(),
                modifiers: Modifiers::default()
            }]
        );
        assert_eq!(eng.shift_state, ShiftState::Off);
        assert_eq!(eng.level_idx, 0);

        // Double-tap Shift -> Caps Lock stays locked across characters.
        eng.pointer_down(sx, sy, w, h, 2.0);
        eng.pointer_up(sx, sy, w, h, 2.05);
        eng.pointer_down(sx, sy, w, h, 2.15);
        eng.pointer_up(sx, sy, w, h, 2.20);
        assert_eq!(eng.shift_state, ShiftState::Locked);
        eng.pointer_down(qx, qy, w, h, 2.4);
        let out2 = eng.pointer_up(qx, qy, w, h, 2.45);
        assert_eq!(
            out2,
            vec![KeyboardOutput::CommitText {
                text: "Q".into(),
                modifiers: Modifiers::default()
            }]
        );
        assert_eq!(eng.shift_state, ShiftState::Locked);

        // Switch to Persian and press ZWNJ half-space key (row 3, col 4).
        assert_eq!(eng.set_layout(LayoutId::Ir), KeyboardOutput::LayoutChanged(LayoutId::Ir));
        let fa_placed = eng.placed_keys(w, h);
        let zwnj = ui::placed_key_for(&fa_placed, 3, 4).unwrap();
        let zx = zwnj.face_rect.x + zwnj.face_rect.width * 0.5;
        let zy = zwnj.face_rect.y + zwnj.face_rect.height * 0.5;
        eng.pointer_down(zx, zy, w, h, 3.0);
        let fa_out = eng.pointer_up(zx, zy, w, h, 3.05);
        assert_eq!(
            fa_out,
            vec![KeyboardOutput::CommitText {
                text: "\u{200c}".into(),
                modifiers: Modifiers::default()
            }]
        );
    }

    #[test]
    fn long_press_opens_alternates_and_commits_digit() {
        let mut eng = KeyboardEngine::new(LayoutId::Us);
        let w = 400.0;
        let h = eng.desired_height(w);
        let placed = eng.placed_keys(w, h);
        let q = ui::placed_key_for(&placed, 0, 0).unwrap();
        let qx = q.face_rect.x + q.face_rect.width * 0.5;
        let qy = q.face_rect.y + q.face_rect.height * 0.5;
        eng.pointer_down(qx, qy, w, h, 10.0);
        assert!(eng.popover.is_none());
        let _ = eng.tick(10.4, w, h);
        assert!(eng.popover.is_some());
        let out = eng.pointer_up(qx, qy, w, h, 10.45);
        assert_eq!(
            out,
            vec![KeyboardOutput::CommitText {
                text: "1".into(),
                modifiers: Modifiers::default()
            }]
        );
    }

    #[test]
    fn landscape_mode_fits_all_rows_and_footer_inside_bounds() {
        for layout in [LayoutId::Us, LayoutId::Ir, LayoutId::Terminal] {
            let mut eng = KeyboardEngine::new(layout);
            let vp_w = 832.0;
            let vp_h = 400.0;
            let kb_h = eng.desired_height_for_viewport(vp_w, vp_h);
            assert!(
                kb_h <= vp_h * 0.70,
                "layout {:?} landscape height {kb_h} too tall for {vp_h}",
                layout
            );
            let fy = eng.footer_y(vp_w, kb_h);
            assert!(fy > 0.0 && fy < kb_h - 24.0, "footer must fit inside landscape deck");

            let placed = eng.placed_keys(vp_w, kb_h);
            for pk in &placed {
                assert!(
                    pk.face_rect.y >= 0.0 && pk.face_rect.y + pk.face_rect.height <= fy + 1.0,
                    "key ({},{}) overflows landscape deck: face={:?} footer_y={fy}",
                    pk.row,
                    pk.col,
                    pk.face_rect
                );
            }

            // Bottom row key tap + footer language toggle must both work accurately in landscape.
            let last_row = placed.iter().map(|p| p.row).max().unwrap();
            let space_or_bottom = placed.iter().find(|p| p.row == last_row && p.col == 3).unwrap();
            let bx = space_or_bottom.face_rect.x + space_or_bottom.face_rect.width * 0.5;
            let by = space_or_bottom.face_rect.y + space_or_bottom.face_rect.height * 0.5;
            eng.pointer_down(bx, by, vp_w, kb_h, 1.0);
            assert!(eng.is_key_pressed(space_or_bottom.row, space_or_bottom.col));
            eng.pointer_up(bx, by, vp_w, kb_h, 1.05);

            // Tap left footer pill (language toggle)
            let lang_y = fy + (kb_h - fy) * 0.5;
            eng.pointer_down(32.0, lang_y, vp_w, kb_h, 2.0);
            let out = eng.pointer_up(32.0, lang_y, vp_w, kb_h, 2.05);
            assert!(matches!(out.first(), Some(KeyboardOutput::LayoutChanged(_))));
        }
    }

    #[test]
    fn renders_libbhoo_draw_commands_for_english_persian_and_emoji() {
        let mut eng = KeyboardEngine::new(LayoutId::Us);
        let bounds = Rect { x: 0.0, y: 0.0, width: 400.0, height: eng.desired_height(400.0) };
        let mut cmds = Vec::new();
        eng.render(bounds, &mut cmds);
        assert!(cmds.len() > 80);

        eng.set_layout(LayoutId::Ir);
        cmds.clear();
        eng.render(bounds, &mut cmds);
        assert!(cmds.len() > 95);

        eng.emoji_open = true;
        cmds.clear();
        eng.render(bounds, &mut cmds);
        assert!(cmds.len() > 70);
    }
}
