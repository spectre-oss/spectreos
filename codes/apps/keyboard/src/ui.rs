//! Pure `libbhoo` rendering for SpectreOS Keyboard.
//!
//! Draws the entire default keyboard (background deck, rounded keycaps, shadows, subtle
//! alternate hints, vector icons, floating key-press preview bubble, long-press alternate
//! popover, emoji picker, and footer bar) using `bhoo::DrawCommand` primitives.

use crate::layout::{
    EMOJI_PAGES, KeyDef, KeyIcon, KeyStyle, LayoutId, PlacedKey, REF_FOOTER_Y, REF_WIDTH,
    ref_total_height,
};
use crate::{FooterAction, KeyboardEngine, ShiftState};
use bhoo::{Color, DrawCommand, FontFamily, Rect, TextStyle};
use std::sync::Arc;

#[derive(Clone, Copy)]
struct DeckScale {
    x: f32,
    y: f32,
    glyph: f32,
    pill: f32,
}

fn deck_scale(width: f32, height: f32, layout: LayoutId) -> DeckScale {
    let x = (width / REF_WIDTH).max(0.05);
    let y = (height / ref_total_height(layout)).max(0.05);
    let glyph = x.min(y);
    let pill = glyph.max(x * 0.52);
    DeckScale { x, y, glyph, pill }
}

struct FooterLayout {
    footer_y: f32,
    lang: Rect,
    emoji: Rect,
    terminal: Rect,
    dismiss: Rect,
}

fn footer_layout(engine: &KeyboardEngine, bounds: Rect, sc: DeckScale) -> FooterLayout {
    let footer_y = bounds.y + engine.footer_y(bounds.width, bounds.height);
    let footer_h = (bounds.y + bounds.height - footer_y).max(20.0);
    let pill_h = (footer_h * 0.66).clamp(18.0, (88.0 * sc.glyph).max(24.0)).min(footer_h - 4.0).max(16.0);
    let pill_y = footer_y + (footer_h - pill_h) * 0.5;
    let side_pad = (28.0 * sc.pill).max(10.0);
    let gap = (20.0 * sc.pill).max(8.0);
    let lang_w = 152.0 * sc.pill;
    let emoji_w = 112.0 * sc.pill;
    let term_w = 152.0 * sc.pill;
    let hide_w = 148.0 * sc.pill;

    FooterLayout {
        footer_y,
        lang: Rect {
            x: bounds.x + side_pad,
            y: pill_y,
            width: lang_w,
            height: pill_h,
        },
        emoji: Rect {
            x: bounds.x + side_pad + lang_w + gap,
            y: pill_y,
            width: emoji_w,
            height: pill_h,
        },
        terminal: Rect {
            x: bounds.x + bounds.width * 0.5 - term_w * 0.5,
            y: pill_y,
            width: term_w,
            height: pill_h,
        },
        dismiss: Rect {
            x: bounds.x + bounds.width - side_pad - hide_w,
            y: pill_y,
            width: hide_w,
            height: pill_h,
        },
    }
}

// SpectreOS dark keyboard color tokens (matching adwaita-dark / SpectreOS palette).
pub const COLOR_BG: Color = Color(41, 41, 43, 255); // #29292b
pub const COLOR_KEY_BG: Color = Color(70, 70, 74, 255); // #46464a
pub const COLOR_KEY_FG: Color = Color(227, 227, 231, 255); // #e3e3e7
pub const COLOR_SYS_BG: Color = Color(62, 70, 95, 255); // #3e465f
pub const COLOR_SYS_FG: Color = Color(223, 227, 242, 255); // #dfe3f2
pub const COLOR_PRESSED_BG: Color = Color(89, 103, 129, 255); // #596781
pub const COLOR_ACTIVE_MOD: Color = Color(108, 129, 172, 255); // Shift locked / Ctrl active
pub const COLOR_HINT_FG: Color = Color(200, 200, 204, 195); // #c8c8cc
pub const COLOR_KEY_SHADOW: Color = Color(18, 18, 22, 145);
pub const COLOR_POPUP_BG: Color = Color(78, 80, 88, 255);
pub const COLOR_POPUP_BORDER: Color = Color(112, 114, 124, 255);
pub const COLOR_FOOTER_FG: Color = Color(236, 236, 242, 225);
pub const COLOR_FOOTER_PILL: Color = Color(58, 58, 64, 255);

fn text_style(size: f32, weight: u16, align: u8) -> TextStyle {
    TextStyle {
        size,
        weight,
        align,
        padding: 0.0,
        letter_spacing: 0.0,
        italic: false,
        family: FontFamily::Sans,
    }
}

/// Emit all `bhoo::DrawCommand`s for the current keyboard state inside `bounds`.
pub fn render_keyboard(
    engine: &KeyboardEngine,
    bounds: Rect,
    commands: &mut Vec<DrawCommand>,
) {
    let sc = deck_scale(bounds.width, bounds.height, engine.layout_id);
    commands.push(DrawCommand::RoundedRect {
        bounds,
        radius: 0.0,
        color: COLOR_BG,
    });
    // Subtle top hairline separating keyboard deck from application viewport.
    commands.push(DrawCommand::Line {
        from: [bounds.x, bounds.y + 0.5],
        to: [bounds.x + bounds.width, bounds.y + 0.5],
        width: 1.0,
        color: Color(64, 64, 70, 255),
    });

    if engine.emoji_open {
        render_emoji_picker(engine, bounds, sc, commands);
    } else {
        render_keys(engine, bounds, sc, commands);
    }

    render_footer(engine, bounds, sc, commands);

    if !engine.emoji_open {
        if let Some(ref popover) = engine.popover {
            render_alternates_popover(bounds, popover, sc.glyph, commands);
        } else if let Some(pressed) = engine.pressed_key_preview(bounds.width, bounds.height) {
            render_key_preview_bubble(bounds, &pressed.face_rect, &pressed.label, sc.glyph, commands);
        }
    }
}

fn key_colors(key: &KeyDef, pressed: bool, active_toggle: bool) -> (Color, Color) {
    let fg = match key.style {
        KeyStyle::Sys | KeyStyle::Toggle | KeyStyle::Symbols | KeyStyle::Return | KeyStyle::Punctuation => {
            COLOR_SYS_FG
        }
        _ => COLOR_KEY_FG,
    };
    if pressed {
        return (COLOR_PRESSED_BG, Color(255, 255, 255, 255));
    }
    if active_toggle {
        return (COLOR_ACTIVE_MOD, Color(255, 255, 255, 255));
    }
    let bg = match key.style {
        KeyStyle::Sys | KeyStyle::Toggle | KeyStyle::Symbols | KeyStyle::Return | KeyStyle::Punctuation => {
            COLOR_SYS_BG
        }
        _ => COLOR_KEY_BG,
    };
    (bg, fg)
}

fn render_keys(
    engine: &KeyboardEngine,
    bounds: Rect,
    sc: DeckScale,
    commands: &mut Vec<DrawCommand>,
) {
    let scale = sc.glyph;
    let layout = engine.active_layout();
    let level = layout.level(engine.level_idx);
    let placed = engine.placed_keys(bounds.width, bounds.height);

    for pk in &placed {
        let Some(key) = level.rows.get(pk.row).and_then(|r| r.get(pk.col)) else {
            continue;
        };
        let face = Rect {
            x: bounds.x + pk.face_rect.x,
            y: bounds.y + pk.face_rect.y,
            width: pk.face_rect.width,
            height: pk.face_rect.height,
        };
        let is_pressed = engine.is_key_pressed(pk.row, pk.col);
        let is_active_toggle = is_key_toggle_active(engine, key);
        let (bg, fg) = key_colors(key, is_pressed, is_active_toggle);

        let radius = match key.style {
            KeyStyle::Symbols | KeyStyle::Return => (face.height * 0.5).min(50.0 * scale),
            _ => (12.0 * scale).clamp(4.0, face.height * 0.3),
        };

        // Subtle 1px drop ledge for tactile keycap depth.
        commands.push(DrawCommand::RoundedRect {
            bounds: Rect {
                x: face.x,
                y: face.y + (2.5 * scale).max(1.0),
                width: face.width,
                height: face.height,
            },
            radius,
            color: COLOR_KEY_SHADOW,
        });
        commands.push(DrawCommand::RoundedRect {
            bounds: face,
            radius,
            color: bg,
        });

        // Subtle corner hint for keys with alternates (top-right digit / alternate mark).
        if let Some(hint) = key.alternates.first()
            && !hint.is_empty()
            && key.style != KeyStyle::Punctuation
        {
            let hint_size = (18.0 * scale).clamp(7.5, face.height * 0.30);
            let hint_rect = Rect {
                x: face.x + face.width - hint_size * 1.45,
                y: face.y + (3.5 * scale).max(1.0),
                width: hint_size * 1.25,
                height: hint_size * 1.15,
            };
            commands.push(DrawCommand::Text {
                bounds: hint_rect,
                text: Arc::from(hint.as_str()),
                color: COLOR_HINT_FG,
                style: text_style(hint_size, 400, 1),
            });
        }

        // Key foreground: vector icon or crisp centered glyph/label.
        if let Some(icon) = key.icon {
            draw_key_icon(icon, face, fg, engine.shift_state, scale, commands);
        } else {
            let label = key.display_text();
            if !label.is_empty() {
                let max_text_h = (face.height * 0.62).max(10.0);
                let font_size = match key.style {
                    KeyStyle::Space => (24.0 * scale).max(10.0),
                    KeyStyle::Symbols | KeyStyle::Toggle => (26.0 * scale).max(10.5),
                    KeyStyle::Sys => (23.0 * scale).max(10.0),
                    KeyStyle::HalfSpace => (26.0 * scale).max(10.5),
                    KeyStyle::Persian => (42.0 * scale).max(13.0),
                    _ => (40.0 * scale).max(13.0),
                }
                .min(max_text_h);
                let weight = match key.style {
                    KeyStyle::Symbols | KeyStyle::Toggle | KeyStyle::Sys => 500,
                    _ => 400,
                };
                commands.push(DrawCommand::Text {
                    bounds: face,
                    text: Arc::from(label),
                    color: fg,
                    style: text_style(font_size, weight, 1),
                });
            }
        }
    }
}

fn is_key_toggle_active(engine: &KeyboardEngine, key: &KeyDef) -> bool {
    if key.icon == Some(KeyIcon::Shift) {
        return engine.shift_state != ShiftState::Off;
    }
    if key.symbol == "MOD_CTRL" {
        return engine.ctrl_latched;
    }
    if key.symbol == "MOD_ALT" {
        return engine.alt_latched;
    }
    false
}

fn draw_key_icon(
    icon: KeyIcon,
    face: Rect,
    color: Color,
    shift_state: ShiftState,
    scale: f32,
    commands: &mut Vec<DrawCommand>,
) {
    let cx = face.x + face.width * 0.5;
    let cy = face.y + face.height * 0.5;
    let s = (face.width.min(face.height) * 0.24).max(5.5);
    let stroke = (5.0 * scale).max(1.7);

    match icon {
        KeyIcon::Shift => {
            // Upward arrow chevron + stem; underline bar indicates Caps Lock.
            let top = [cx, cy - s * 1.05];
            let left = [cx - s * 1.05, cy + s * 0.05];
            let right = [cx + s * 1.05, cy + s * 0.05];
            commands.push(DrawCommand::Polyline {
                points: [left, top, right],
                count: 3,
                width: stroke,
                color,
            });
            commands.push(DrawCommand::Line {
                from: [cx - s * 0.45, cy + s * 0.05],
                to: [cx - s * 0.45, cy + s * 0.9],
                width: stroke,
                color,
            });
            commands.push(DrawCommand::Line {
                from: [cx + s * 0.45, cy + s * 0.05],
                to: [cx + s * 0.45, cy + s * 0.9],
                width: stroke,
                color,
            });
            commands.push(DrawCommand::Line {
                from: [cx - s * 0.45, cy + s * 0.9],
                to: [cx + s * 0.45, cy + s * 0.9],
                width: stroke,
                color,
            });
            if shift_state == ShiftState::Locked {
                commands.push(DrawCommand::Line {
                    from: [cx - s * 0.75, cy + s * 1.32],
                    to: [cx + s * 0.75, cy + s * 1.32],
                    width: stroke,
                    color,
                });
            }
        }
        KeyIcon::Backspace => {
            // Pointed left tag outline + inner cross (`×`).
            let lx = cx - s * 1.15;
            let mid_x = cx - s * 0.5;
            let rx = cx + s * 1.1;
            let ty = cy - s * 0.75;
            let by = cy + s * 0.75;
            commands.push(DrawCommand::Polyline {
                points: [[rx, ty], [mid_x, ty], [lx, cy]],
                count: 3,
                width: stroke,
                color,
            });
            commands.push(DrawCommand::Polyline {
                points: [[lx, cy], [mid_x, by], [rx, by]],
                count: 3,
                width: stroke,
                color,
            });
            commands.push(DrawCommand::Line {
                from: [rx, ty],
                to: [rx, by],
                width: stroke,
                color,
            });
            let xs = s * 0.36;
            let xcx = cx + s * 0.22;
            commands.push(DrawCommand::Line {
                from: [xcx - xs, cy - xs],
                to: [xcx + xs, cy + xs],
                width: stroke,
                color,
            });
            commands.push(DrawCommand::Line {
                from: [xcx + xs, cy - xs],
                to: [xcx - xs, cy + xs],
                width: stroke,
                color,
            });
        }
        KeyIcon::Enter => {
            // Bent return arrow pointing left.
            let rx = cx + s * 0.95;
            let lx = cx - s * 0.85;
            let ty = cy - s * 0.65;
            let by = cy + s * 0.25;
            commands.push(DrawCommand::Polyline {
                points: [[rx, ty], [rx, by], [lx, by]],
                count: 3,
                width: stroke,
                color,
            });
            commands.push(DrawCommand::Polyline {
                points: [
                    [lx + s * 0.55, by - s * 0.5],
                    [lx, by],
                    [lx + s * 0.55, by + s * 0.5],
                ],
                count: 3,
                width: stroke,
                color,
            });
        }
        KeyIcon::Globe => {
            // Minimal globe icon (circle + equator + vertical meridian).
            let r = s * 0.95;
            let globe_rect = Rect {
                x: cx - r,
                y: cy - r,
                width: r * 2.0,
                height: r * 2.0,
            };
            commands.push(DrawCommand::Outline {
                bounds: globe_rect,
                radius: r,
                width: stroke * 0.85,
                color,
            });
            commands.push(DrawCommand::Outline {
                bounds: Rect {
                    x: cx - r * 0.48,
                    y: cy - r,
                    width: r * 0.96,
                    height: r * 2.0,
                },
                radius: r * 0.48,
                width: stroke * 0.72,
                color,
            });
            commands.push(DrawCommand::Line {
                from: [cx - r * 0.92, cy],
                to: [cx + r * 0.92, cy],
                width: stroke * 0.72,
                color,
            });
        }
        KeyIcon::Emoji => {
            let r = s * 0.92;
            commands.push(DrawCommand::Outline {
                bounds: Rect {
                    x: cx - r,
                    y: cy - r,
                    width: r * 2.0,
                    height: r * 2.0,
                },
                radius: r,
                width: stroke * 0.85,
                color,
            });
            let eye = (r * 0.16).max(1.4);
            commands.push(DrawCommand::Ellipse {
                bounds: Rect {
                    x: cx - r * 0.36 - eye,
                    y: cy - r * 0.25 - eye,
                    width: eye * 2.0,
                    height: eye * 2.0,
                },
                color,
            });
            commands.push(DrawCommand::Ellipse {
                bounds: Rect {
                    x: cx + r * 0.36 - eye,
                    y: cy - r * 0.25 - eye,
                    width: eye * 2.0,
                    height: eye * 2.0,
                },
                color,
            });
            commands.push(DrawCommand::Arc {
                bounds: Rect {
                    x: cx - r * 0.5,
                    y: cy - r * 0.4,
                    width: r,
                    height: r * 0.9,
                },
                start: 0.35,
                sweep: std::f32::consts::PI - 0.7,
                stroke: stroke * 0.75,
                color,
                rounded: true,
            });
        }
        KeyIcon::Dismiss => {
            // Downward chevron (`⌄`) to dismiss keyboard.
            let w = s * 0.95;
            let h = s * 0.5;
            commands.push(DrawCommand::Polyline {
                points: [[cx - w, cy - h * 0.5], [cx, cy + h * 0.8], [cx + w, cy - h * 0.5]],
                count: 3,
                width: stroke,
                color,
            });
        }
    }
}

fn render_emoji_picker(
    engine: &KeyboardEngine,
    bounds: Rect,
    sc: DeckScale,
    commands: &mut Vec<DrawCommand>,
) {
    let scale = sc.glyph;
    let tab_h = 92.0 * sc.y;
    let footer_y = engine.footer_y(bounds.width, bounds.height);
    let pad = 16.0 * sc.x;

    // Top category tab strip (`ABC`, Smileys, Gestures, Objects, Backspace).
    let tab_count = EMOJI_PAGES.len() + 2;
    let tab_w = (bounds.width - pad * 2.0) / tab_count as f32;
    for i in 0..tab_count {
        let rect = Rect {
            x: bounds.x + pad + i as f32 * tab_w + 4.0 * scale,
            y: bounds.y + 10.0 * sc.y,
            width: (tab_w - 8.0 * scale).max(8.0),
            height: (tab_h - 18.0 * sc.y).max(16.0),
        };
        let active = i >= 1 && i <= EMOJI_PAGES.len() && (i - 1) == engine.emoji_page;
        let bg = if active {
            COLOR_PRESSED_BG
        } else if i == 0 || i == tab_count - 1 {
            COLOR_SYS_BG
        } else {
            COLOR_KEY_BG
        };
        commands.push(DrawCommand::RoundedRect {
            bounds: rect,
            radius: (12.0 * scale).max(4.0),
            color: bg,
        });
        if i == 0 {
            commands.push(DrawCommand::Text {
                bounds: rect,
                text: Arc::from("ABC"),
                color: COLOR_SYS_FG,
                style: text_style((24.0 * scale).max(10.0), 500, 1),
            });
        } else if i == tab_count - 1 {
            draw_key_icon(KeyIcon::Backspace, rect, COLOR_SYS_FG, ShiftState::Off, scale, commands);
        } else {
            let page = &EMOJI_PAGES[i - 1];
            commands.push(DrawCommand::Text {
                bounds: rect,
                text: Arc::from(page.tab),
                color: COLOR_KEY_FG,
                style: text_style((32.0 * scale).max(12.0), 400, 1),
            });
        }
    }

    // 8×4 Emoji grid.
    let page = &EMOJI_PAGES[engine.emoji_page.min(EMOJI_PAGES.len() - 1)];
    let cols = 8usize;
    let rows = 4usize;
    let grid_y = bounds.y + tab_h + 6.0 * sc.y;
    let grid_h = (footer_y - tab_h - 12.0 * sc.y).max(40.0);
    let cell_w = (bounds.width - pad * 2.0) / cols as f32;
    let cell_h = grid_h / rows as f32;
    for (idx, emoji) in page.items.iter().enumerate().take(cols * rows) {
        let r = idx / cols;
        let c = idx % cols;
        let rect = Rect {
            x: bounds.x + pad + c as f32 * cell_w + 4.0 * scale,
            y: grid_y + r as f32 * cell_h + 3.0 * sc.y,
            width: (cell_w - 8.0 * scale).max(8.0),
            height: (cell_h - 6.0 * sc.y).max(8.0),
        };
        let bg = if engine.pressed_emoji == Some(idx) {
            COLOR_PRESSED_BG
        } else {
            COLOR_KEY_BG.opacity(0.65)
        };
        commands.push(DrawCommand::RoundedRect {
            bounds: rect,
            radius: (10.0 * scale).max(4.0),
            color: bg,
        });
        commands.push(DrawCommand::Text {
            bounds: rect,
            text: Arc::from(*emoji),
            color: COLOR_KEY_FG,
            style: text_style((40.0 * scale).clamp(13.0, rect.height * 0.72), 400, 1),
        });
    }
}

fn render_footer(
    engine: &KeyboardEngine,
    bounds: Rect,
    sc: DeckScale,
    commands: &mut Vec<DrawCommand>,
) {
    let fl = footer_layout(engine, bounds, sc);
    let scale = sc.glyph;
    let pill_r = fl.lang.height * 0.5;

    // Left: Language badge (`EN` / `فا` / `TERM`) — switches layout on tap.
    let lang_bg = if engine.pressed_footer == Some(FooterAction::ToggleLanguage) {
        COLOR_PRESSED_BG
    } else {
        COLOR_FOOTER_PILL
    };
    commands.push(DrawCommand::RoundedRect {
        bounds: fl.lang,
        radius: pill_r,
        color: lang_bg,
    });
    commands.push(DrawCommand::Text {
        bounds: fl.lang,
        text: Arc::from(engine.layout_id.display_badge()),
        color: COLOR_FOOTER_FG,
        style: text_style((26.0 * scale).clamp(10.0, fl.lang.height * 0.62), 600, 1),
    });

    // Center-left: Emoji toggle pill.
    let emoji_bg = if engine.emoji_open || engine.pressed_footer == Some(FooterAction::ToggleEmoji) {
        COLOR_PRESSED_BG
    } else {
        COLOR_FOOTER_PILL
    };
    commands.push(DrawCommand::RoundedRect {
        bounds: fl.emoji,
        radius: pill_r,
        color: emoji_bg,
    });
    draw_key_icon(KeyIcon::Emoji, fl.emoji, COLOR_FOOTER_FG, ShiftState::Off, scale, commands);

    // Center: Terminal mode quick toggle pill.
    let term_bg = if engine.layout_id == LayoutId::Terminal
        || engine.pressed_footer == Some(FooterAction::ToggleTerminal)
    {
        COLOR_PRESSED_BG
    } else {
        COLOR_FOOTER_PILL.opacity(0.65)
    };
    commands.push(DrawCommand::RoundedRect {
        bounds: fl.terminal,
        radius: pill_r,
        color: term_bg,
    });
    commands.push(DrawCommand::Text {
        bounds: fl.terminal,
        text: Arc::from(">_"),
        color: COLOR_FOOTER_FG,
        style: text_style((24.0 * scale).clamp(10.0, fl.terminal.height * 0.60), 600, 1),
    });

    // Right: Dismiss keyboard down-chevron (`⌄`).
    let hide_bg = if engine.pressed_footer == Some(FooterAction::Dismiss) {
        COLOR_PRESSED_BG
    } else {
        COLOR_FOOTER_PILL
    };
    commands.push(DrawCommand::RoundedRect {
        bounds: fl.dismiss,
        radius: pill_r,
        color: hide_bg,
    });
    draw_key_icon(KeyIcon::Dismiss, fl.dismiss, COLOR_FOOTER_FG, ShiftState::Off, scale, commands);

    let _ = (REF_FOOTER_Y, fl.footer_y);
}

fn render_key_preview_bubble(
    bounds: Rect,
    key_face: &Rect,
    label: &str,
    scale: f32,
    commands: &mut Vec<DrawCommand>,
) {
    let bubble_w = (key_face.width * 1.28).max(96.0 * scale);
    let bubble_h = (key_face.height * 1.12).max(118.0 * scale);
    let bx = (bounds.x + key_face.x + (key_face.width - bubble_w) * 0.5)
        .clamp(bounds.x + 8.0 * scale, bounds.x + bounds.width - bubble_w - 8.0 * scale);
    let by = (bounds.y + key_face.y - bubble_h - 10.0 * scale).max(bounds.y + 4.0 * scale);
    let rect = Rect {
        x: bx,
        y: by,
        width: bubble_w,
        height: bubble_h,
    };
    let radius = (14.0 * scale).max(5.0);
    commands.push(DrawCommand::Shadow {
        bounds: rect,
        radius,
        offset: [0.0, (4.0 * scale).max(1.5)],
        blur: (14.0 * scale).max(4.0),
        spread: 0.0,
        color: Color(0, 0, 0, 130),
    });
    commands.push(DrawCommand::RoundedRect {
        bounds: rect,
        radius,
        color: COLOR_POPUP_BG,
    });
    commands.push(DrawCommand::Outline {
        bounds: rect,
        radius,
        width: (2.0 * scale).max(1.0),
        color: COLOR_POPUP_BORDER,
    });
    commands.push(DrawCommand::Text {
        bounds: rect,
        text: Arc::from(label),
        color: Color(255, 255, 255, 255),
        style: text_style((52.0 * scale).max(18.0), 500, 1),
    });
}

fn render_alternates_popover(
    bounds: Rect,
    popover: &crate::AlternatePopover,
    scale: f32,
    commands: &mut Vec<DrawCommand>,
) {
    let rect = Rect {
        x: bounds.x + popover.bounds.x,
        y: bounds.y + popover.bounds.y,
        width: popover.bounds.width,
        height: popover.bounds.height,
    };
    let radius = (14.0 * scale).max(5.0);
    commands.push(DrawCommand::Shadow {
        bounds: rect,
        radius,
        offset: [0.0, (5.0 * scale).max(2.0)],
        blur: (18.0 * scale).max(5.0),
        spread: 0.0,
        color: Color(0, 0, 0, 150),
    });
    commands.push(DrawCommand::RoundedRect {
        bounds: rect,
        radius,
        color: COLOR_POPUP_BG,
    });
    commands.push(DrawCommand::Outline {
        bounds: rect,
        radius,
        width: (2.0 * scale).max(1.0),
        color: COLOR_POPUP_BORDER,
    });
    let count = popover.items.len().max(1);
    let cell_w = rect.width / count as f32;
    let pad = (6.0 * scale).max(2.0);
    for (idx, item) in popover.items.iter().enumerate() {
        let cell = Rect {
            x: rect.x + idx as f32 * cell_w + pad,
            y: rect.y + pad,
            width: (cell_w - pad * 2.0).max(6.0),
            height: (rect.height - pad * 2.0).max(6.0),
        };
        if popover.selected == idx {
            commands.push(DrawCommand::RoundedRect {
                bounds: cell,
                radius: (10.0 * scale).max(4.0),
                color: COLOR_PRESSED_BG,
            });
        }
        commands.push(DrawCommand::Text {
            bounds: cell,
            text: Arc::from(item.as_str()),
            color: Color(255, 255, 255, 255),
            style: text_style((42.0 * scale).max(15.0), 500, 1),
        });
    }
}

pub fn footer_hit_test(
    engine: &KeyboardEngine,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<FooterAction> {
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    let sc = deck_scale(width, height, engine.layout_id);
    let fl = footer_layout(engine, bounds, sc);
    if y < fl.footer_y {
        return None;
    }
    let slop = (12.0 * sc.pill).max(6.0);
    let hit = |r: Rect| x >= r.x - slop && x <= r.x + r.width + slop;
    if hit(fl.lang) {
        Some(FooterAction::ToggleLanguage)
    } else if hit(fl.emoji) {
        Some(FooterAction::ToggleEmoji)
    } else if hit(fl.terminal) {
        Some(FooterAction::ToggleTerminal)
    } else if hit(fl.dismiss) {
        Some(FooterAction::Dismiss)
    } else {
        None
    }
}

pub fn placed_key_for(placed: &[PlacedKey], row: usize, col: usize) -> Option<&PlacedKey> {
    placed.iter().find(|k| k.row == row && k.col == col)
}
