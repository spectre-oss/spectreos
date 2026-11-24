//! Android 13 / Material You Lock Screen (Keyguard) built natively with `libbhoo`.
//!
//! Uses `libbhoo` interaction targets (`Frame::target`), Material 3 state layers & ink ripples
//! (`Frame::state_layer`), AndroidX spatial spring transitions (`Frame::spatial`), and crisp
//! high-res Material `SystemIcon`s (`paint_sys_icon`).
//!
//! Enforces Android/SpectreOS Lock Screen notification privacy policies (`LockScreenNotifMode`:
//! ShowAll / HideSilent / HideSensitive / DontShow + `show_sensitive_on_lock` + DND) and supports
//! interactive horizontal swipe-to-dismiss, explicit `×` dismiss, Clear all, and tap-to-open/unlock.

use bhoo::{Color, Frame, ImageFit, Input, Key, Rect, Role, TextStyle};

pub use spectre_settings::{
    KeyPadEvent, KeyboardEvent, LockConfig, LockMethod, LockScreenNotifMode, SoftKeyboardState,
    load_lock_config, paint_pin_keypad, paint_qwerty_keyboard, save_lock_config,
};

use crate::services::{FingerprintStatus, HapticEffect, TouchIdState};
use crate::{NotificationItem, Palette, RingerMode, SysIcon, SystemState, paint_sys_icon, wallpaper};

const LOCK_ID_BASE: u64 = 0x5749_5350_0100_0000;
const ID_FLASHLIGHT: u64 = LOCK_ID_BASE + 2;
const ID_CAMERA: u64 = LOCK_ID_BASE + 3;
const ID_UNLOCK_PILL: u64 = LOCK_ID_BASE + 4;
const ID_BOUNCER_CANCEL: u64 = LOCK_ID_BASE + 5;
const ID_SHOW_PASS: u64 = LOCK_ID_BASE + 6;
const ID_CLEAR_ALL_NOTIFS: u64 = LOCK_ID_BASE + 7;
const ID_TOUCH_ID: u64 = LOCK_ID_BASE + 8;
const ID_BOUNCER_ANIM: u64 = LOCK_ID_BASE + 10;
const ID_SWIPE_ANIM: u64 = LOCK_ID_BASE + 11;
const ID_NOTIF_CARD_BASE: u64 = LOCK_ID_BASE + 40;
const ID_NOTIF_DISMISS_BASE: u64 = LOCK_ID_BASE + 70;
const UNLOCK_DURATION: f64 = 0.32;

fn bouncer_colors(p: Palette) -> (Color, Color) {
    if p.dark {
        (Color(24, 23, 29, 255), Color(244, 241, 250, 255))
    } else {
        (Color(240, 239, 241, 255), Color(30, 28, 35, 255))
    }
}

fn password_field(phone: Rect, slide_y: f32) -> Rect {
    Rect {
        x: phone.x + phone.width * 0.17,
        y: phone.y + phone.height * 0.49 - 44.0 + slide_y,
        width: phone.width * 0.66,
        height: 44.0,
    }
}

#[derive(Clone, Debug)]
struct UnlockReveal {
    start: Option<f64>,
    progress: f32,
    from_bouncer: bool,
    wake_from_dark: bool,
}

#[derive(Clone, Debug)]
pub struct LockScreen {
    pub locked: bool,
    pub bouncer_open: bool,
    pub entry: String,
    pub error: Option<String>,
    pub show_password: bool,
    pub kbd: SoftKeyboardState,
    pub drag_start_y: Option<f32>,
    pub drag_active: bool,
    pub drag_offset_y: f32,
    unlock_reveal: Option<UnlockReveal>,
    pub pending_notif_open: Option<u32>,
    pending_shortcut: Option<usize>,
    /// Live Touch ID / fingerprint sensor status surfaced from `FingerprintConnector`.
    pub touch_id_state: TouchIdState,
    pub touch_id_message: Option<String>,
    /// Queued haptic effects emitted by lock-screen buttons, keypad, and unlock events.
    pub pending_haptics: Vec<HapticEffect>,
    /// Active horizontal swipe-to-dismiss state on a lock-screen notification card:
    /// `(notif_id, start_x, start_y, offset_x, locked_horizontal)`.
    notif_drag: Option<(u32, f32, f32, f32, bool)>,
    /// Last-frame dismissible notification card bounds `(notif_id, rect)` for gesture hit-testing.
    last_notif_cards: Vec<(u32, Rect)>,
}

impl Default for LockScreen {
    fn default() -> Self {
        Self {
            locked: false,
            bouncer_open: false,
            entry: String::new(),
            error: None,
            show_password: false,
            kbd: SoftKeyboardState::default(),
            drag_start_y: None,
            drag_active: false,
            drag_offset_y: 0.0,
            unlock_reveal: None,
            pending_notif_open: None,
            pending_shortcut: None,
            touch_id_state: TouchIdState::Unavailable,
            touch_id_message: None,
            pending_haptics: Vec::new(),
            notif_drag: None,
            last_notif_cards: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockAction {
    None,
    Unlocked,
    OpenShortcut(usize),
    DismissNotification(u32),
    ClearAllNotifications,
    OpenNotification(u32),
    RetryTouchId,
}

pub(super) fn shortcut_icon(shortcut: usize) -> Option<SysIcon> {
    match shortcut {
        1 => Some(SysIcon::Silent),
        2 => Some(SysIcon::ScreenRecord),
        3 => Some(SysIcon::Camera),
        4 => Some(SysIcon::Dnd),
        5 => Some(SysIcon::Flashlight),
        6 => Some(SysIcon::DeviceControls),
        _ => None,
    }
}

impl LockScreen {
    pub fn lock(&mut self) {
        self.pending_shortcut = None;
        self.locked = true;
        self.bouncer_open = false;
        self.entry.clear();
        self.show_password = false;
        self.unlock_reveal = None;
        self.error = None;
        self.drag_start_y = None;
        self.drag_active = false;
        self.drag_offset_y = 0.0;
        self.pending_notif_open = None;
        self.notif_drag = None;
        if self.touch_id_state == TouchIdState::Matched || self.touch_id_state == TouchIdState::Failed {
            self.touch_id_state = TouchIdState::Idle;
            self.touch_id_message = None;
        }
    }

    pub fn unlock(&mut self) {
        self.locked = false;
        self.bouncer_open = false;
        self.entry.clear();
        self.show_password = false;
        self.error = None;
        self.drag_start_y = None;
        self.drag_active = false;
        self.drag_offset_y = 0.0;
        self.notif_drag = None;
    }

    /// Update visual Touch ID state from the platform-agnostic `FingerprintConnector`.
    pub fn on_touch_id_status(&mut self, status: FingerprintStatus) {
        self.touch_id_state = if status.available && status.enrolled_count > 0 {
            status.state
        } else {
            TouchIdState::Unavailable
        };
        self.touch_id_message = status.message;
    }

    /// Unlock immediately when the fingerprint connector confirms a biometric match.
    pub fn on_touch_id_matched(&mut self) -> LockAction {
        self.fingerprint_unlock(false)
    }

    pub(crate) fn fingerprint_unlock(&mut self, screen_was_off: bool) -> LockAction {
        if !self.locked {
            return LockAction::None;
        }
        self.touch_id_state = TouchIdState::Matched;
        self.touch_id_message = Some("Unlocked".into());
        self.pending_haptics.push(HapticEffect::Confirm);
        let action = self.finish_unlock();
        if let Some(reveal) = &mut self.unlock_reveal {
            reveal.wake_from_dark = screen_was_off;
        }
        action
    }

    /// Show biometric mismatch feedback without dismissing the lock screen.
    pub fn on_touch_id_rejected(&mut self, message: String) {
        if !self.locked {
            return;
        }
        self.touch_id_state = TouchIdState::Failed;
        self.touch_id_message = Some(message.clone());
        if self.bouncer_open {
            self.error = Some(message);
        }
        self.pending_haptics.push(HapticEffect::Reject);
    }

    fn finish_unlock(&mut self) -> LockAction {
        let pending = self.pending_notif_open.take();
        let shortcut = self.pending_shortcut.take();
        let from_bouncer = self.bouncer_open;
        self.unlock();
        self.unlock_reveal = Some(UnlockReveal { start: None, progress: 0.0, from_bouncer, wake_from_dark: false });
        if let Some(id) = pending {
            LockAction::OpenNotification(id)
        } else if let Some(shortcut) = shortcut {
            LockAction::OpenShortcut(shortcut)
        } else {
            LockAction::Unlocked
        }
    }

    pub(crate) fn advance_unlock(&mut self, now: f64, scale: f32, reduced: bool) {
        if let Some(reveal) = &mut self.unlock_reveal {
            if reduced || scale <= 0.0 {
                self.unlock_reveal = None;
                return;
            }
            let start = *reveal.start.get_or_insert(now);
            let duration = if reveal.wake_from_dark { 0.22 } else { UNLOCK_DURATION };
            reveal.progress = ((now - start) / (duration * f64::from(scale))) as f32;
            if reveal.progress >= 1.0 {
                self.unlock_reveal = None;
            }
        }
    }

    pub(crate) fn is_unlocking(&self) -> bool {
        self.unlock_reveal.is_some()
    }

    // Remaining distance of the screen-off reveal. A cubic ease-out gives
    // immediate feedback and settles gently over the final frames.
    pub(crate) fn wake_reveal_remaining(&self) -> f32 {
        self.unlock_reveal.as_ref().filter(|r| r.wake_from_dark)
            .map_or(0.0, |r| (1.0 - r.progress.clamp(0.0, 1.0)).powi(3))
    }

    pub(crate) fn home_reveal_alpha(&self) -> f32 {
        self.unlock_reveal.as_ref().map_or(1.0, |reveal| {
            if reveal.wake_from_dark { 1.0 } else { crate::smoothstep(0.30, 0.94, reveal.progress) }
        })
    }

    pub(crate) fn paint_unlock_reveal(&self, f: &mut Frame<'_>, phone: Rect, p: Palette, method: LockMethod) {
        let Some(reveal) = &self.unlock_reveal else { return };
        let t = reveal.progress;
        let alpha = 1.0 - crate::smoothstep(0.0, 0.52, t);
        if reveal.wake_from_dark {
            f.paint_rect(phone, 0.0, Color(0, 0, 0, 255).opacity(self.wake_reveal_remaining()));
        } else if reveal.from_bouncer {
            let (surface, ink) = if method == LockMethod::Password {
                bouncer_colors(p)
            } else {
                (Color(0x0B, 0x0E, 0x15, 255), Color(0, 0, 0, 0))
            };
            f.paint_rect(phone, 0.0, surface.opacity(alpha));
            // Leave only the underline of the departing credential UI. Input
            // and reveal state have already been cleared by authentication.
            let field = password_field(phone, 0.0);
            f.paint_rect(
                Rect { y: field.y + field.height - 1.5, height: 1.5, ..field },
                0.75,
                ink.opacity(1.0 - crate::smoothstep(0.24, 0.75, t)),
            );
        } else {
            f.paint_rect(phone, 0.0, Color(0, 0, 0, 57).opacity(alpha));
        }
        f.continuous(phone);
    }

    fn attempt_unlock(&mut self, sys: &SystemState) -> LockAction {
        if sys.lock_method == LockMethod::Swipe
            || self.entry == sys.lock_secret
            || sys.lock_secret.is_empty()
        {
            self.pending_haptics.push(HapticEffect::Confirm);
            self.finish_unlock()
        } else {
            self.entry.clear();
            self.show_password = false;
            self.error = Some(match sys.lock_method {
                LockMethod::Pin => "Wrong PIN".into(),
                LockMethod::Password => "Wrong password".into(),
                LockMethod::Swipe => String::new(),
            });
            self.pending_haptics.push(HapticEffect::Reject);
            LockAction::None
        }
    }

    /// Paint the Android 13 / Material You lock screen inside `phone` using `libbhoo`.
    pub(crate) fn show(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        input: Input,
        sys: &mut SystemState,
        notifications: &[NotificationItem],
        p: Palette,
        wallpaper: &wallpaper::Wallpaper,
        clock: super::ClockReading,
    ) -> LockAction {
        let primary = p.primary;
        let mut pending_action = LockAction::None;

        // 1. Handle horizontal swipe-to-dismiss on lock-screen notification cards OR vertical swipe-up to unlock.
        if !self.bouncer_open {
            if let Some([px, py]) = input.pointer {
                let inside = px >= phone.x
                    && px <= phone.x + phone.width
                    && py >= phone.y
                    && py <= phone.y + phone.height;
                if input.pressed && inside {
                    self.drag_start_y = Some(py);
                    self.drag_active = false;
                    self.drag_offset_y = 0.0;
                    self.notif_drag = self
                        .last_notif_cards
                        .iter()
                        .find(|(_, r)| r.contains([px, py]))
                        .map(|(id, _)| (*id, px, py, 0.0, false));
                } else {
                    // Check horizontal notification card swipe first
                    if let Some((nid, sx, sy, _, mut horiz)) = self.notif_drag {
                        let dx = px - sx;
                        let dy = py - sy;
                        if !horiz && !self.drag_active && dx.abs() > 14.0 && dx.abs() > dy.abs() * 1.15 {
                            horiz = true;
                            self.drag_start_y = None;
                        }
                        if horiz {
                            let off_x = dx.clamp(-phone.width, phone.width);
                            if input.released || input.cancelled {
                                self.notif_drag = None;
                                if !input.cancelled && off_x.abs() >= (phone.width * 0.24).clamp(76.0, 120.0) {
                                    pending_action = LockAction::DismissNotification(nid);
                                }
                                f.continuous(phone);
                            } else {
                                self.notif_drag = Some((nid, sx, sy, off_x, true));
                                f.continuous(phone);
                            }
                        } else if input.released || input.cancelled {
                            self.notif_drag = None;
                        }
                    }

                    // Vertical swipe-up to unlock (only when not horizontally swiping a notification)
                    let horiz_active = self.notif_drag.is_some_and(|d| d.4);
                    if !horiz_active
                        && let Some(start_y) = self.drag_start_y
                    {
                        let dy = py - start_y;
                        if !self.drag_active && dy < -18.0 {
                            self.drag_active = true;
                            self.notif_drag = None;
                        }
                        if input.released || input.cancelled {
                            let lifted = self.drag_active && self.drag_offset_y < -72.0;
                            self.drag_start_y = None;
                            self.drag_active = false;
                            self.drag_offset_y = 0.0;
                            if lifted {
                                if sys.lock_method == LockMethod::Swipe {
                                    f.continuous(phone);
                                    return self.finish_unlock();
                                } else {
                                    self.bouncer_open = true;
                                    self.entry.clear();
                                    self.error = None;
                                    f.continuous(phone);
                                }
                            }
                        } else if self.drag_active {
                            self.drag_offset_y = dy.clamp(-240.0, 12.0);
                        }
                    }
                }
            } else if input.released || input.cancelled {
                self.drag_start_y = None;
                self.drag_active = false;
                self.drag_offset_y = 0.0;
                self.notif_drag = None;
            }
        }

        // Smooth libbhoo spatial spring for swipe bounce-back and bouncer reveal
        let visual_drag = if self.drag_active {
            self.drag_offset_y
        } else {
            f.spatial(ID_SWIPE_ANIM, 0, 0.0, self.drag_offset_y)
        };
        let bouncer_t = f.spatial(
            ID_BOUNCER_ANIM,
            0,
            if self.bouncer_open { 1.0 } else { 0.0 },
            0.0,
        );

        let swipe_progress = (-visual_drag / 180.0).clamp(0.0, 1.0);
        wallpaper.paint(f, phone, sys.wallpaper_idx);
        f.paint_rect(phone, 0.0, Color(0, 0, 0, (57.0 - 24.0 * swipe_progress) as u8));
        if self.bouncer_open {
            let surface = if sys.lock_method == LockMethod::Password {
                bouncer_colors(p).0
            } else {
                Color(0x0B, 0x0E, 0x15, 255)
            };
            f.paint_rect(phone, 0.0, surface.opacity(0.82 + 0.18 * bouncer_t));
            return self.paint_bouncer(f, phone, input, sys, p, bouncer_t);
        }

        let cx = phone.x + phone.width * 0.5;
        let y_shift = visual_drag * 0.45;

        // Top status bar using high-res Material SystemIcons
        let status_y = phone.y + 8.0;
        f.styled_text(
            Rect {
                x: phone.x + 16.0,
                y: status_y,
                width: 112.0,
                height: 20.0,
            },
            "SpectreOS",
            Color(235, 240, 248, 225),
            TextStyle {
                size: 12.5,
                weight: 500,
                align: 0,
                ..TextStyle::default()
            },
        );

        // Status bar notification/DND indicators on lock screen (honors hide_silent_in_status_bar)
        let status_notif_count = notifications
            .iter()
            .filter(|n| {
                !n.snoozed
                    && !n.dismissing
                    && (!sys.hide_silent_in_status_bar || (!n.silent && n.urgency > 0))
            })
            .count();
        let mut status_rx = phone.x + phone.width - 16.0;
        if sys.show_battery_pct {
            let batt = format!("{}%", sys.battery_level);
            status_rx -= 44.0;
            f.styled_text(
                Rect {
                    x: status_rx,
                    y: status_y,
                    width: 44.0,
                    height: 20.0,
                },
                &batt,
                Color(235, 240, 248, 225),
                TextStyle {
                    size: 12.5,
                    weight: 500,
                    align: 2,
                    ..TextStyle::default()
                },
            );
            status_rx -= 6.0;
        }
        status_rx -= 16.0;
        paint_sys_icon(
            f,
            Rect {
                x: status_rx,
                y: status_y + 2.0,
                width: 15.0,
                height: 15.0,
            },
            if sys.wifi_enabled {
                SysIcon::Wifi
            } else {
                SysIcon::WifiOff
            },
            Color(235, 240, 248, 220),
        );
        if sys.dnd_enabled {
            status_rx -= 20.0;
            paint_sys_icon(
                f,
                Rect {
                    x: status_rx,
                    y: status_y + 2.0,
                    width: 15.0,
                    height: 15.0,
                },
                SysIcon::Dnd,
                Color(235, 240, 248, 220),
            );
        }
        if status_notif_count > 0 && sys.lock_screen_notifs != LockScreenNotifMode::DontShow {
            status_rx -= 20.0;
            paint_sys_icon(
                f,
                Rect {
                    x: status_rx,
                    y: status_y + 2.0,
                    width: 15.0,
                    height: 15.0,
                },
                SysIcon::Bell,
                Color(235, 240, 248, 220),
            );
        }

        // Determine visible lock screen notifications based on user privacy & notification settings
        let visible_notifs: Vec<&NotificationItem> = match sys.lock_screen_notifs {
            LockScreenNotifMode::DontShow => Vec::new(),
            LockScreenNotifMode::HideSilent => notifications
                .iter()
                .filter(|n| !n.snoozed && !n.dismissing && !n.silent && n.urgency > 0)
                .collect(),
            LockScreenNotifMode::ShowAll | LockScreenNotifMode::HideSensitive => notifications
                .iter()
                .filter(|n| !n.snoozed && !n.dismissing)
                .collect(),
        };
        let total_active = notifications
            .iter()
            .filter(|n| !n.snoozed && !n.dismissing)
            .count();
        let compact_clock = !visible_notifs.is_empty();

        // Android 13 dynamic clock: compact single-line when notifications are present,
        // expressive stacked two-line clock when lock screen is empty!
        use chrono::Timelike;
        let (hh, mm) = (clock.0.hour(), clock.0.minute());
        let hh_s = format!("{hh:02}");
        let mm_s = format!("{mm:02}");
        let clock_ink = Color(255, 255, 255, 255).mix(primary, 0.14);
        {
            f.styled_text(
                Rect { x: phone.x + 30.0, y: phone.y + 56.0 + y_shift, width: phone.width - 60.0, height: 34.0 },
                &clock.date(),
                clock_ink,
                TextStyle { size: 15.5, weight: 500, align: 0, ..TextStyle::default() },
            );
        }
        let notif_start_y = if compact_clock {
            let clock_y = phone.y + 91.0 + y_shift;
            f.styled_text(
                Rect { x: phone.x + 28.0, y: clock_y, width: phone.width - 56.0, height: 74.0 },
                &format!("{hh:02}:{mm:02}"),
                clock_ink,
                TextStyle { size: 59.0, weight: 400, align: 0, ..TextStyle::default() },
            );
            clock_y + 91.0
        } else {
            let line_h = phone.width * 0.38;
            let clock_y = phone.y + phone.height * 0.266 + y_shift;
            for (row, line) in [hh_s, mm_s].iter().enumerate() {
                f.styled_text(
                    Rect { x: phone.x, y: clock_y + row as f32 * line_h, width: phone.width, height: line_h },
                    line,
                    clock_ink,
                    TextStyle { size: phone.width * 0.46, weight: 300, align: 1, ..TextStyle::default() },
                );
            }
            clock_y + line_h * 2.0
        };

        // Interactive Material 3 Lock Screen Notifications (with privacy redaction, swipe-to-dismiss, × & Clear all)
        self.last_notif_cards.clear();
        let hide_sensitive = sys.lock_screen_notifs == LockScreenNotifMode::HideSensitive
            || !sys.show_sensitive_on_lock;
        let has_touch_id = sys.touch_id_available && sys.touch_id_enrolled > 0;
        let max_ny = phone.y + phone.height - if has_touch_id { 226.0 } else { 144.0 };

        if sys.lock_screen_notifs == LockScreenNotifMode::DontShow && total_active > 0 {
            let pill_w = 224.0_f32.min(phone.width - 48.0);
            let pill = Rect {
                x: cx - pill_w * 0.5,
                y: notif_start_y + 8.0,
                width: pill_w,
                height: 36.0,
            };
            f.paint_rect(pill, 18.0, Color(28, 33, 44, 210));
            paint_sys_icon(
                f,
                Rect {
                    x: pill.x + 14.0,
                    y: pill.y + 10.0,
                    width: 16.0,
                    height: 16.0,
                },
                SysIcon::VisibilityOff,
                Color(195, 206, 226, 220),
            );
            let msg = if total_active == 1 {
                "1 notification hidden".to_string()
            } else {
                format!("{total_active} notifications hidden")
            };
            f.styled_text(
                Rect {
                    x: pill.x + 36.0,
                    y: pill.y,
                    width: pill.width - 48.0,
                    height: pill.height,
                },
                &msg,
                Color(208, 216, 232, 225),
                TextStyle {
                    size: 12.5,
                    weight: 500,
                    align: 0,
                    ..TextStyle::default()
                },
            );
        } else if !visible_notifs.is_empty() {
            let mut ny = notif_start_y;
            let max_cards = 4usize;
            let mut shown = 0usize;
            let mut dismissible_shown = 0usize;
            let drag_horiz = self.notif_drag.is_some_and(|d| d.4);

            for (i, notif) in visible_notifs.iter().take(max_cards).enumerate() {
                let card_h = 68.0;
                if ny + card_h > max_ny {
                    break;
                }
                shown += 1;
                if !notif.ongoing {
                    dismissible_shown += 1;
                }

                let base_card = Rect {
                    x: phone.x + 16.0,
                    y: ny,
                    width: phone.width - 32.0,
                    height: card_h,
                };
                if !notif.ongoing {
                    self.last_notif_cards.push((notif.id, base_card));
                }

                let swipe_dx = match self.notif_drag {
                    Some((nid, _, _, off_x, true)) if nid == notif.id => off_x,
                    _ => 0.0,
                };
                let alpha = (1.0 - (swipe_dx.abs() / (phone.width * 0.65)).clamp(0.0, 0.92)).max(0.08);
                let card = Rect {
                    x: base_card.x + swipe_dx,
                    ..base_card
                };

                if alpha < 0.995 {
                    f.push_layer(card, alpha);
                }

                let bg = if notif.urgency == 2 {
                    Color(44, 30, 38, 236)
                } else {
                    Color(26, 31, 42, 232)
                };
                f.paint_rect(card, 20.0, bg);
                if notif.urgency == 2 {
                    f.paint_rect(
                        Rect {
                            x: card.x + 6.0,
                            y: card.y + 14.0,
                            width: 3.5,
                            height: card.height - 28.0,
                        },
                        2.0,
                        Color(255, 118, 128, 240),
                    );
                }

                // Dismiss × button on right of non-ongoing cards
                let dismiss_hit = Rect {
                    x: card.x + card.width - 38.0,
                    y: card.y + 18.0,
                    width: 30.0,
                    height: 30.0,
                };
                let mut dismiss_clicked = false;
                if !notif.ongoing {
                    let r_dis = f.target(
                        ID_NOTIF_DISMISS_BASE + i as u64,
                        dismiss_hit,
                        "Dismiss notification",
                        !drag_horiz,
                        Role::Button,
                    );
                    f.paint_rect(dismiss_hit, 15.0, Color(255, 255, 255, 18));
                    f.state_layer(r_dis, 15.0, Color(255, 255, 255, 220));
                    paint_sys_icon(
                        f,
                        dismiss_hit.inset(7.5),
                        SysIcon::Close,
                        Color(204, 212, 228, 225),
                    );
                    dismiss_clicked = r_dis.clicked && !drag_horiz;
                }

                // Card tap target (opens/unlocks to invoke notification action)
                let tap_rect = Rect {
                    x: card.x,
                    y: card.y,
                    width: if notif.ongoing {
                        card.width
                    } else {
                        card.width - 42.0
                    },
                    height: card.height,
                };
                let r_card = f.target(
                    ID_NOTIF_CARD_BASE + i as u64,
                    tap_rect,
                    &notif.title,
                    !drag_horiz,
                    Role::Button,
                );
                f.state_layer(r_card, 20.0, primary);

                // Leading icon: real Freedesktop image-data / icon-theme image OR crisp Material SystemIcon
                let badge = Rect {
                    x: card.x + 14.0,
                    y: card.y + 17.0,
                    width: 34.0,
                    height: 34.0,
                };
                if !hide_sensitive
                    && let Some(ref img) = notif.icon_image
                {
                    f.paint_rect(badge, 12.0, Color(255, 255, 255, 24));
                    f.paint_image(badge.inset(2.0), img, ImageFit::Cover, 10.0);
                } else {
                    let badge_bg = if notif.urgency == 2 {
                        Color(235, 92, 108, 240)
                    } else {
                        Color(primary.0, primary.1, primary.2, 235)
                    };
                    f.paint_rect(badge, 17.0, badge_bg);
                    let lead_ic = if hide_sensitive {
                        SysIcon::VisibilityOff
                    } else {
                        notif_category_icon(notif)
                    };
                    paint_sys_icon(f, badge.inset(8.0), lead_ic, Color(16, 20, 28, 255));
                }

                // Text column
                let text_x = card.x + 58.0;
                let text_w = (card.width - if notif.ongoing { 74.0 } else { 104.0 }).max(80.0);
                let meta = format!("{} · {}", notif.app, notif.time_label);
                f.styled_text(
                    Rect {
                        x: text_x,
                        y: card.y + 8.0,
                        width: text_w,
                        height: 15.0,
                    },
                    &meta,
                    Color(178, 190, 212, 220),
                    TextStyle {
                        size: 11.0,
                        weight: 500,
                        align: 0,
                        ..TextStyle::default()
                    },
                );

                let (disp_title, disp_body) = if hide_sensitive {
                    (
                        "Sensitive notification",
                        "Content hidden until unlocked",
                    )
                } else {
                    (notif.title.as_str(), notif.body.as_str())
                };
                f.styled_text(
                    Rect {
                        x: text_x,
                        y: card.y + 24.0,
                        width: text_w,
                        height: 19.0,
                    },
                    disp_title,
                    Color(244, 247, 255, 250),
                    TextStyle {
                        size: 13.5,
                        weight: 600,
                        align: 0,
                        ..TextStyle::default()
                    },
                );
                if !disp_body.is_empty() {
                    f.styled_text(
                        Rect {
                            x: text_x,
                            y: card.y + 44.0,
                            width: text_w,
                            height: 17.0,
                        },
                        disp_body,
                        Color(192, 202, 220, 218),
                        TextStyle {
                            size: 12.0,
                            weight: 400,
                            align: 0,
                            ..TextStyle::default()
                        },
                    );
                }

                if alpha < 0.995 {
                    f.pop_layer();
                }

                if dismiss_clicked {
                    pending_action = LockAction::DismissNotification(notif.id);
                } else if r_card.clicked && !drag_horiz && !self.drag_active {
                    if sys.lock_method == LockMethod::Swipe {
                        self.unlock();
                        f.continuous(phone);
                        return LockAction::OpenNotification(notif.id);
                    }
                    self.pending_notif_open = Some(notif.id);
                    self.bouncer_open = true;
                    self.entry.clear();
                    self.error = None;
                    f.continuous(phone);
                }

                ny += card_h + 8.0;
            }

            // Overflow summary + "Clear all" pill below lock screen cards
            let remaining = visible_notifs.len().saturating_sub(shown);
            let total_dismissible = visible_notifs.iter().filter(|n| !n.ongoing).count();
            if ny + 34.0 <= max_ny + 18.0 && (remaining > 0 || total_dismissible >= 2 || dismissible_shown >= 2) {
                let row_y = ny + 2.0;
                if remaining > 0 {
                    let more_r = Rect {
                        x: phone.x + 20.0,
                        y: row_y,
                        width: 112.0,
                        height: 30.0,
                    };
                    f.paint_rect(more_r, 15.0, Color(255, 255, 255, 24));
                    f.styled_text(
                        more_r,
                        &format!("+{remaining} more"),
                        Color(225, 232, 246, 230),
                        TextStyle {
                            size: 12.0,
                            weight: 500,
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                }
                if total_dismissible >= 2 {
                    let clr_r = Rect {
                        x: phone.x + phone.width - 108.0,
                        y: row_y,
                        width: 92.0,
                        height: 30.0,
                    };
                    let rc = f.target(ID_CLEAR_ALL_NOTIFS, clr_r, "Clear all notifications", true, Role::Button);
                    f.paint_rect(clr_r, 15.0, Color(255, 255, 255, 26));
                    f.state_layer(rc, 15.0, primary);
                    f.styled_text(
                        clr_r,
                        "Clear all",
                        Color(238, 244, 255, 240),
                        TextStyle {
                            size: 12.0,
                            weight: 500,
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                    if rc.clicked {
                        pending_action = LockAction::ClearAllNotifications;
                    }
                }
            }
        }

        // Touch ID / Rear Fingerprint live indicator above the bottom quick-action row
        let bottom_y = phone.y + phone.height - 88.0;
        if has_touch_id {
            let tid_y = bottom_y - 84.0;
            let tid_rect = Rect {
                x: cx - 28.0,
                y: tid_y,
                width: 56.0,
                height: 56.0,
            };
            let pulse = if self.touch_id_state == TouchIdState::Listening {
                0.5 + 0.5 * ((f.time() * 3.2).sin() as f32)
            } else {
                0.0
            };
            let (ring_col, icon_col, label_col, default_hint) = match self.touch_id_state {
                TouchIdState::Listening => (
                    Color(primary.0, primary.1, primary.2, (88.0 + 95.0 * pulse) as u8),
                    Color(primary.0, primary.1, primary.2, 245),
                    Color(215, 224, 242, 220),
                    "Touch sensor to unlock",
                ),
                TouchIdState::Failed => (
                    Color(255, 110, 120, 225),
                    Color(255, 134, 142, 255),
                    Color(255, 152, 158, 245),
                    "Fingerprint not recognized",
                ),
                TouchIdState::Matched => (
                    Color(110, 232, 165, 240),
                    Color(140, 248, 190, 255),
                    Color(180, 248, 210, 240),
                    "Unlocked",
                ),
                TouchIdState::Idle | TouchIdState::Unavailable => (
                    Color(255, 255, 255, 42),
                    Color(225, 234, 248, 220),
                    Color(195, 206, 226, 200),
                    "Tap or touch rear sensor",
                ),
            };
            if self.touch_id_state == TouchIdState::Listening {
                let glow = tid_rect.inset(-(3.0 + 3.0 * pulse));
                f.paint_rect(
                    glow,
                    glow.width * 0.5,
                    Color(primary.0, primary.1, primary.2, (28.0 + 26.0 * pulse) as u8),
                );
            }
            let r_tid = f.target(ID_TOUCH_ID, tid_rect, "Touch ID", true, Role::Button);
            f.paint_rect(tid_rect.inset(-2.0), 30.0, ring_col);
            f.paint_rect(tid_rect, 28.0, Color(24, 30, 42, 238));
            f.state_layer(r_tid, 28.0, primary);
            paint_sys_icon(f, tid_rect.inset(14.0), SysIcon::Fingerprint, icon_col);
            let hint_text = self.touch_id_message.as_deref().unwrap_or(default_hint);
            f.styled_text(
                Rect {
                    x: phone.x + 24.0,
                    y: tid_rect.y + tid_rect.height + 4.0,
                    width: phone.width - 48.0,
                    height: 18.0,
                },
                hint_text,
                label_col,
                TextStyle {
                    size: 11.5,
                    weight: 500,
                    align: 1,
                    ..TextStyle::default()
                },
            );
            if r_tid.clicked {
                self.pending_haptics.push(HapticEffect::Tick);
                self.touch_id_message = Some("Touch rear sensor".into());
                pending_action = LockAction::RetryTouchId;
                f.continuous(phone);
            }
        }

        for (slot, shortcut) in sys.lock_shortcuts.into_iter().enumerate() {
            let Some(icon) = shortcut_icon(shortcut) else { continue };
            let bounds = Rect { x: if slot == 0 { phone.x + 24.0 } else { phone.x + phone.width - 76.0 },
                y: bottom_y, width: 52.0, height: 52.0 };
            let enabled = shortcut != 5 || sys.flashlight_available;
            let active = match shortcut {
                1 => sys.ringer_mode == RingerMode::Silent,
                4 => sys.dnd_enabled,
                5 => sys.flashlight_on,
                _ => false,
            };
            let response = f.target(if slot == 0 { ID_FLASHLIGHT } else { ID_CAMERA }, bounds,
                wallpaper::SHORTCUTS[shortcut], enabled, Role::Button);
            let (background, ink) = if active { (primary, p.on_primary) } else {
                (Color(30, 35, 46, if enabled { 225 } else { 140 }), Color(242, 246, 255, if enabled { 245 } else { 120 }))
            };
            f.paint_rect(bounds, 26.0, background);
            f.state_layer(response, 26.0, ink);
            super::wallpaper_ui::paint_wallpaper_system_icon(f, bounds.inset(14.0), icon, ink, sys.icon_style);
            if response.clicked {
                self.pending_haptics.push(HapticEffect::Click);
                f.continuous(phone);
                match shortcut {
                    1 => sys.ringer_mode = if active { RingerMode::Ring } else { RingerMode::Silent },
                    4 => sys.dnd_enabled = !active,
                    5 => sys.flashlight_on = !active,
                    2 | 3 | 6 => {
                        self.pending_shortcut = Some(shortcut);
                        if sys.lock_method == LockMethod::Swipe { return self.finish_unlock(); }
                        self.bouncer_open = true;
                        self.entry.clear();
                        self.error = None;
                    }
                    _ => {}
                }
                if matches!(shortcut, 1 | 4) { sys.save_persisted_settings(); }
            }
        }

        let unlock_rect = Rect {
            x: cx - 26.0,
            y: bottom_y + 4.0,
            width: 52.0,
            height: 48.0,
        };
        let unlock_label = match sys.lock_method {
            LockMethod::Swipe => "Swipe up to unlock",
            LockMethod::Pin => "Enter PIN to unlock",
            LockMethod::Password => "Enter password",
        };
        let r_unlock = f.target(ID_UNLOCK_PILL, unlock_rect, unlock_label, true, Role::Button);
        paint_sys_icon(
            f, unlock_rect.inset(9.0),
            if sys.lock_method == LockMethod::Swipe { SysIcon::LockOpen } else { SysIcon::Security },
            Color(242, 239, 255, 245),
        );
        if r_unlock.clicked && !self.drag_active {
            if sys.lock_method == LockMethod::Swipe {
                f.continuous(phone);
                return self.finish_unlock();
            }
            self.bouncer_open = true;
            self.entry.clear();
            self.error = None;
            f.continuous(phone);
        }

        // Bottom gesture pill
        f.paint_rect(
            Rect {
                x: cx - 54.0,
                y: phone.y + phone.height - 14.0,
                width: 108.0,
                height: 4.0,
            },
            2.0,
            Color(230, 236, 248, 195),
        );

        pending_action
    }

    fn paint_bouncer(
        &mut self,
        f: &mut Frame<'_>,
        phone: Rect,
        input: Input,
        sys: &SystemState,
        p: Palette,
        bouncer_t: f32,
    ) -> LockAction {
        let primary = p.primary;

        // Also accept hardware keyboard input when bouncer is open
        if let Some(k) = input.key
            && k.pressed
        {
            match k.key {
                Key::Character('\u{8}') | Key::Character('\u{7f}') => {
                    self.entry.pop();
                    self.error = None;
                    f.continuous(phone);
                }
                Key::Space if sys.lock_method == LockMethod::Password && self.entry.len() < 32 => {
                    self.entry.push(' ');
                    self.error = None;
                    f.continuous(phone);
                }
                Key::Enter => {
                    let act = self.attempt_unlock(sys);
                    f.continuous(phone);
                    return act;
                }
                Key::Escape => {
                    self.bouncer_open = false;
                    self.pending_notif_open = None;
                    self.pending_shortcut = None;
                    self.entry.clear();
                    self.show_password = false;
                    self.error = None;
                    f.continuous(phone);
                    return LockAction::None;
                }
                Key::Character(ch) if !ch.is_control() && self.entry.len() < 32 => {
                    if sys.lock_method != LockMethod::Pin || ch.is_ascii_digit() {
                        self.entry.push(ch);
                        self.error = None;
                        f.continuous(phone);
                    }
                }
                _ => {}
            }
        }

        let slide_y = (1.0 - bouncer_t) * 36.0;
        let cx = phone.x + phone.width * 0.5;
        let password = sys.lock_method == LockMethod::Password;
        let header_y = phone.y + if password { phone.height * 0.115 } else { 108.0 } + slide_y;
        let (_, password_ink) = bouncer_colors(p);

        let prompt = match (&self.error, sys.lock_method, self.pending_notif_open.is_some()) {
            (Some(err), _, _) => err.as_str(),
            (None, LockMethod::Pin, true) => "Enter PIN to open notification",
            (None, LockMethod::Password, true) => "Enter password to open notification",
            (None, LockMethod::Pin, false) => "Enter PIN",
            (None, LockMethod::Password, false) => "Enter your password",
            (None, LockMethod::Swipe, _) => "Unlocked",
        };
        f.styled_text(
            Rect {
                x: phone.x + 20.0,
                y: header_y,
                width: phone.width - 40.0,
                height: 26.0,
            },
            prompt,
            if self.error.is_some() {
                Color(255, 134, 134, 255)
            } else if password {
                password_ink
            } else {
                Color(225, 232, 245, 240)
            },
            TextStyle {
                size: if password { 13.0 } else { 16.0 },
                weight: 500,
                align: 1,
                ..TextStyle::default()
            },
        );

        match sys.lock_method {
            LockMethod::Pin | LockMethod::Swipe => {
                let dots = if self.entry.is_empty() {
                    "○  ○  ○  ○".to_string()
                } else {
                    "● ".repeat(self.entry.len()).trim_end().to_string()
                };
                f.styled_text(
                    Rect {
                        x: phone.x + 20.0,
                        y: header_y + 42.0,
                        width: phone.width - 40.0,
                        height: 36.0,
                    },
                    &dots,
                    Color(245, 248, 255, 255),
                    TextStyle {
                        size: 24.0,
                        weight: 700,
                        align: 1,
                        ..TextStyle::default()
                    },
                );

                let pad_area = Rect {
                    x: phone.x + 32.0,
                    y: header_y + 104.0,
                    width: phone.width - 64.0,
                    height: 332.0,
                };
                match paint_pin_keypad(
                    f,
                    pad_area,
                    input,
                    primary,
                    Color(34, 40, 52, 238),
                    Color(244, 247, 255, 255),
                ) {
                    KeyPadEvent::Digit(ch) if self.entry.len() < 12 => {
                        self.entry.push(ch);
                        self.error = None;
                        self.pending_haptics.push(HapticEffect::Tick);
                        f.continuous(phone);
                        if !sys.lock_secret.is_empty() && self.entry == sys.lock_secret {
                            self.pending_haptics.push(HapticEffect::Confirm);
                            return self.finish_unlock();
                        }
                    }
                    KeyPadEvent::Backspace => {
                        self.entry.pop();
                        self.error = None;
                        self.pending_haptics.push(HapticEffect::Tick);
                        f.continuous(phone);
                    }
                    KeyPadEvent::Submit => {
                        let act = self.attempt_unlock(sys);
                        f.continuous(phone);
                        return act;
                    }
                    _ => {}
                }

                let cancel_rect = Rect {
                    x: cx - 64.0,
                    y: phone.y + phone.height - 68.0,
                    width: 128.0,
                    height: 38.0,
                };
                let r_cancel = f.target(ID_BOUNCER_CANCEL, cancel_rect, "Cancel", true, Role::Button);
                f.paint_rect(cancel_rect, 19.0, Color(255, 255, 255, 20));
                f.state_layer(r_cancel, 19.0, primary);
                f.styled_text(
                    cancel_rect,
                    "Cancel",
                    Color(215, 224, 240, 235),
                    TextStyle {
                        size: 13.5,
                        weight: 500,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
                if r_cancel.clicked {
                    self.bouncer_open = false;
                    self.pending_notif_open = None;
                    self.pending_shortcut = None;
                    self.entry.clear();
                    self.error = None;
                    f.continuous(phone);
                }
            }
            LockMethod::Password => {
                let field_rect = password_field(phone, slide_y);
                f.paint_rect(Rect {
                    y: field_rect.y + field_rect.height - 1.5,
                    height: 1.5,
                    ..field_rect
                }, 0.75, password_ink);
                let display = if self.show_password {
                    self.entry.clone()
                } else {
                    "•".repeat(self.entry.chars().count())
                };
                if display.is_empty() {
                    f.paint_rect(Rect {
                        x: cx,
                        y: field_rect.y + 12.0,
                        width: 1.0,
                        height: 20.0,
                    }, 0.5, password_ink.opacity(0.35));
                }
                f.styled_text(
                    Rect {
                        x: field_rect.x + 44.0,
                        y: field_rect.y,
                        width: field_rect.width - 88.0,
                        height: field_rect.height,
                    },
                    &display,
                    password_ink,
                    TextStyle {
                        size: 17.0,
                        weight: 400,
                        align: 1,
                        ..TextStyle::default()
                    },
                );

                let eye_rect = Rect {
                    x: field_rect.x + field_rect.width - 44.0,
                    y: field_rect.y,
                    width: 44.0,
                    height: 42.0,
                };
                let r_eye = f.target(ID_SHOW_PASS, eye_rect, "Toggle visibility", true, Role::Button);
                f.state_layer(r_eye, 10.0, primary);
                f.styled_text(
                    eye_rect,
                    if self.show_password { "HIDE" } else { "SHOW" },
                    password_ink,
                    TextStyle {
                        size: 10.5,
                        weight: 600,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
                if r_eye.clicked {
                    self.show_password = !self.show_password;
                    f.continuous(phone);
                }

                let cancel_rect = Rect {
                    x: cx - 64.0,
                    y: phone.y + phone.height * 0.602 + slide_y,
                    width: 128.0,
                    height: 36.0,
                };
                let r_cancel = f.target(ID_BOUNCER_CANCEL, cancel_rect, "Cancel", true, Role::Button);
                f.paint_rect(cancel_rect, 18.0, p.primary_container);
                f.state_layer(r_cancel, 18.0, primary);
                f.styled_text(
                    cancel_rect,
                    "Cancel",
                    p.on_primary_container,
                    TextStyle {
                        size: 13.5,
                        weight: 500,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
                if r_cancel.clicked {
                    self.bouncer_open = false;
                    self.pending_notif_open = None;
                    self.pending_shortcut = None;
                    self.entry.clear();
                    self.show_password = false;
                    self.error = None;
                    f.continuous(phone);
                }

                let kbd_h = (phone.height * 0.31).clamp(208.0, 260.0);
                let kbd_area = Rect {
                    x: phone.x + 4.0,
                    y: phone.y + phone.height - kbd_h - 14.0 + slide_y,
                    width: phone.width - 8.0,
                    height: kbd_h,
                };
                match spectre_settings::keypad::paint_qwerty_keyboard_themed(
                    f, kbd_area, input, &mut self.kbd, primary, p.dark,
                ) {
                    KeyboardEvent::Char(ch) if self.entry.len() < 32 => {
                        self.entry.push(ch);
                        self.error = None;
                        self.pending_haptics.push(HapticEffect::Tick);
                        f.continuous(phone);
                    }
                    KeyboardEvent::Backspace => {
                        self.entry.pop();
                        self.error = None;
                        self.pending_haptics.push(HapticEffect::Tick);
                        f.continuous(phone);
                    }
                    KeyboardEvent::Submit => {
                        let act = self.attempt_unlock(sys);
                        f.continuous(phone);
                        return act;
                    }
                    _ => {}
                }
            }
        }

        LockAction::None
    }
}

pub(crate) fn notif_category_icon(n: &NotificationItem) -> SysIcon {
    if let Some(ref cat) = n.category {
        if cat.starts_with("email") || cat.starts_with("im") {
            return SysIcon::NotificationsActive;
        }
        if cat.starts_with("call") {
            return SysIcon::Sound;
        }
        if cat.starts_with("transfer") {
            return SysIcon::Storage;
        }
        if cat.starts_with("device") {
            return SysIcon::Usb;
        }
        if cat.starts_with("network") {
            return SysIcon::Wifi;
        }
    }
    if n.app == "Android System" || n.app.contains("USB") {
        return SysIcon::Usb;
    }
    if n.urgency == 2 {
        return SysIcon::NotificationsActive;
    }
    SysIcon::Bell
}
