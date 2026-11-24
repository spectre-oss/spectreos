//! Reusable Material You Settings widgets and high-resolution Android 13 vector/icon figures.

use crate::state::{LockScreenNotifMode, NavMode, Palette};
use bhoo::{Color, Frame, Rect, Role, SystemIcon, TextStyle};

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn emphasized_decelerate(t: f32) -> f32 {
    let u = t.clamp(0.0, 1.0);
    1.0 - (1.0 - u).powi(3)
}

pub fn text_style(size: f32, weight: u16, align: u8) -> TextStyle {
    TextStyle {
        size,
        weight,
        align,
        ..TextStyle::default()
    }
}

pub fn paint_switch(
    f: &mut Frame<'_>,
    id: u64,
    bounds: Rect,
    on: &mut bool,
    enabled: bool,
    p: &Palette,
) -> bool {
    let w = 50.0;
    let h = 28.0;
    let pill = Rect {
        x: bounds.x + (bounds.width - w) * 0.5,
        y: bounds.y + (bounds.height - h) * 0.5,
        width: w,
        height: h,
    };
    let r = f.target(id, pill.inset(-4.0), "Switch", enabled, Role::Switch);
    let mut changed = false;
    if r.clicked && enabled {
        *on = !*on;
        changed = true;
    }
    let track = if *on {
        p.primary
    } else if p.dark {
        Color(0x49, 0x4C, 0x56, 255)
    } else {
        Color(0x63, 0x65, 0x6E, 255)
    };
    f.paint_rect(pill, 14.0, track.opacity(if enabled { 1.0 } else { 0.4 }));
    let thumb_sz = if *on { 22.0 } else { 18.0 };
    let thumb_x = if *on {
        pill.x + w - thumb_sz - 3.0
    } else {
        pill.x + 5.0
    };
    let thumb_col = if *on {
        p.primary_container
    } else {
        Color(0xE8, 0xEA, 0xF2, 255)
    };
    f.paint_rect(
        Rect {
            x: thumb_x,
            y: pill.y + (h - thumb_sz) * 0.5,
            width: thumb_sz,
            height: thumb_sz,
        },
        thumb_sz * 0.5,
        thumb_col,
    );
    changed
}

pub fn paint_pref_switch(
    f: &mut Frame<'_>,
    id: u64,
    bounds: Rect,
    title: &str,
    subtitle: Option<&str>,
    on: &mut bool,
    p: &Palette,
) -> bool {
    let full = Rect {
        x: bounds.x - 16.0,
        y: bounds.y,
        width: bounds.width + 32.0,
        height: bounds.height,
    };
    let r = f.target(id, full, title, true, Role::Switch);
    f.state_layer(r, 0.0, p.on_surface);
    let mut changed = false;
    if r.clicked {
        *on = !*on;
        changed = true;
    }
    let text_w = (bounds.width - 72.0).max(40.0);
    if let Some(sub) = subtitle {
        f.styled_text(
            Rect {
                x: bounds.x + 6.0,
                y: bounds.y + 10.0,
                width: text_w,
                height: 22.0,
            },
            title,
            p.on_surface,
            text_style(16.0, 400, 0),
        );
        f.styled_text(
            Rect {
                x: bounds.x + 6.0,
                y: bounds.y + 34.0,
                width: text_w,
                height: 18.0,
            },
            sub,
            p.on_surface_variant,
            text_style(12.5, 400, 0),
        );
    } else {
        f.styled_text(
            Rect {
                x: bounds.x + 6.0,
                y: bounds.y,
                width: text_w,
                height: bounds.height,
            },
            title,
            p.on_surface,
            text_style(16.0, 400, 0),
        );
    }
    let sw_w = 50.0;
    let sw_h = 28.0;
    let pill = Rect {
        x: bounds.x + bounds.width - sw_w - 6.0,
        y: bounds.y + (bounds.height - sw_h) * 0.5,
        width: sw_w,
        height: sw_h,
    };
    let track = if *on {
        p.primary
    } else if p.dark {
        Color(0x49, 0x4C, 0x56, 255)
    } else {
        Color(0x63, 0x65, 0x6E, 255)
    };
    f.paint_rect(pill, 14.0, track);
    let thumb_sz = if *on { 22.0 } else { 18.0 };
    let thumb_x = if *on {
        pill.x + sw_w - thumb_sz - 3.0
    } else {
        pill.x + 5.0
    };
    f.paint_rect(
        Rect {
            x: thumb_x,
            y: pill.y + (sw_h - thumb_sz) * 0.5,
            width: thumb_sz,
            height: thumb_sz,
        },
        thumb_sz * 0.5,
        if *on {
            p.primary_container
        } else {
            Color(0xE8, 0xEA, 0xF2, 255)
        },
    );
    changed
}

pub fn paint_h_slider(
    f: &mut Frame<'_>,
    id: u64,
    bounds: Rect,
    value: &mut f32,
    pointer: Option<[f32; 2]>,
    p: &Palette,
) -> bool {
    let r = f.target(id, bounds, "Slider", true, Role::Slider);
    let mut changed = false;
    if r.pressed
        && let Some([px, _]) = pointer
    {
        let nv = ((px - bounds.x - 10.0) / (bounds.width - 20.0).max(1.0)).clamp(0.0, 1.0);
        if (nv - *value).abs() > 0.001 {
            *value = nv;
            changed = true;
        }
    }
    let track_y = bounds.y + bounds.height * 0.5 - 3.0;
    let track_w = (bounds.width - 20.0).max(10.0);
    let tx = bounds.x + 10.0;
    f.paint_rect(
        Rect {
            x: tx,
            y: track_y,
            width: track_w,
            height: 6.0,
        },
        3.0,
        p.surface_container,
    );
    let active_w = (track_w * value.clamp(0.0, 1.0)).max(6.0);
    f.paint_rect(
        Rect {
            x: tx,
            y: track_y,
            width: active_w,
            height: 6.0,
        },
        3.0,
        p.primary,
    );
    let knob = 18.0;
    f.paint_rect(
        Rect {
            x: tx + track_w * value.clamp(0.0, 1.0) - knob * 0.5,
            y: bounds.y + (bounds.height - knob) * 0.5,
            width: knob,
            height: knob,
        },
        knob * 0.5,
        p.primary,
    );
    changed
}

pub fn paint_radio(f: &mut Frame<'_>, bounds: Rect, selected: bool, p: &Palette) {
    let s = 20.0;
    let r = Rect {
        x: bounds.x + (bounds.width - s) * 0.5,
        y: bounds.y + (bounds.height - s) * 0.5,
        width: s,
        height: s,
    };
    let ring_col = if selected {
        p.primary
    } else {
        p.on_surface_variant
    };
    f.paint_arc(r, 0.0, std::f32::consts::TAU, 2.0, ring_col);
    if selected {
        f.paint_rect(r.inset(5.0), 5.0, p.primary);
    }
}

pub fn paint_pref_row(
    f: &mut Frame<'_>,
    id: u64,
    bounds: Rect,
    title: &str,
    summary: &str,
    p: &Palette,
) -> bool {
    let full = Rect {
        x: bounds.x - 16.0,
        y: bounds.y,
        width: bounds.width + 32.0,
        height: bounds.height,
    };
    let r = f.target(id, full, title, true, Role::ListItem);
    f.state_layer(r, 0.0, p.on_surface);
    f.styled_text(
        Rect {
            x: bounds.x + 6.0,
            y: bounds.y + 10.0,
            width: bounds.width - 32.0,
            height: 22.0,
        },
        title,
        p.on_surface,
        text_style(16.0, 400, 0),
    );
    f.styled_text(
        Rect {
            x: bounds.x + 6.0,
            y: bounds.y + 34.0,
            width: bounds.width - 32.0,
            height: 18.0,
        },
        summary,
        p.on_surface_variant,
        text_style(13.0, 400, 0),
    );
    r.clicked
}

/// High-res Android 13 illustrated System Navigation hero figure (uses crisp 96x96 icons + vector bezel).
pub fn paint_navigation_figure(f: &mut Frame<'_>, card: Rect, mode: NavMode, p: &Palette) {
    f.paint_rect(card, 24.0, p.card_elevated);
    let phone_w = 104.0;
    let phone_h = 112.0;
    let px = card.x + (card.width - phone_w) * 0.5;
    let py = card.y + 16.0;
    let frame = Rect {
        x: px,
        y: py,
        width: phone_w,
        height: phone_h,
    };
    let bezel_col = if p.dark {
        Color(0x42, 0x46, 0x52, 255)
    } else {
        Color(0x36, 0x3A, 0x45, 255)
    };
    f.paint_rect(frame, 18.0, bezel_col);
    let screen = frame.inset(4.5);
    f.paint_rect(screen, 13.5, p.surface_container);

    // Subtle miniature content cards inside phone preview
    f.paint_rect(
        Rect {
            x: screen.x + 9.0,
            y: screen.y + 14.0,
            width: screen.width - 18.0,
            height: 22.0,
        },
        8.0,
        p.primary_container,
    );
    f.paint_rect(
        Rect {
            x: screen.x + 9.0,
            y: screen.y + 42.0,
            width: (screen.width - 18.0) * 0.72,
            height: 10.0,
        },
        5.0,
        p.outline.opacity(0.45),
    );
    f.paint_rect(
        Rect {
            x: screen.x + 9.0,
            y: screen.y + 58.0,
            width: (screen.width - 18.0) * 0.55,
            height: 10.0,
        },
        5.0,
        p.outline.opacity(0.32),
    );

    if mode == NavMode::Gestural {
        // Left edge-back bubble with crisp 96x96 ChevronRight
        let bubble = Rect {
            x: screen.x - 2.0,
            y: screen.y + 38.0,
            width: 24.0,
            height: 28.0,
        };
        f.paint_rect(bubble, 12.0, p.primary);
        f.paint_system_icon(bubble.inset(5.0), SystemIcon::ChevronRight, p.on_primary);
        // Gesture home pill
        f.paint_rect(
            Rect {
                x: screen.x + (screen.width - 36.0) * 0.5,
                y: screen.y + screen.height - 9.0,
                width: 36.0,
                height: 3.5,
            },
            1.75,
            p.primary,
        );
    } else {
        // 3-button Material bar using crisp vector shapes
        let bar = Rect {
            x: screen.x,
            y: screen.y + screen.height - 20.0,
            width: screen.width,
            height: 20.0,
        };
        f.paint_rect(bar, 6.0, p.card_pure.opacity(0.85));
        let y_mid = bar.y + 4.0;
        f.paint_system_icon(
            Rect {
                x: bar.x + 12.0,
                y: y_mid,
                width: 12.0,
                height: 12.0,
            },
            SystemIcon::ChevronLeft,
            p.primary,
        );
        f.paint_arc(
            Rect {
                x: bar.x + bar.width * 0.5 - 5.5,
                y: y_mid + 0.5,
                width: 11.0,
                height: 11.0,
            },
            0.0,
            std::f32::consts::TAU,
            1.8,
            p.primary,
        );
        let sq = Rect {
            x: bar.x + bar.width - 23.0,
            y: y_mid + 1.0,
            width: 10.0,
            height: 10.0,
        };
        f.paint_rect(sq, 2.5, p.primary);
        f.paint_rect(sq.inset(1.8), 1.0, p.card_pure);
    }

    f.styled_text(
        Rect {
            x: card.x + 16.0,
            y: card.y + card.height - 32.0,
            width: card.width - 32.0,
            height: 22.0,
        },
        match mode {
            NavMode::Gestural => "Gesture pill + left/right Edge-Back active",
            NavMode::ThreeButton => "3-button navigation bar active",
        },
        p.primary,
        text_style(13.5, 500, 1),
    );
}

/// High-res Lock Screen Notification preview figure inside Settings -> Notifications.
pub fn paint_lock_notif_figure(
    f: &mut Frame<'_>,
    card: Rect,
    mode: LockScreenNotifMode,
    show_sensitive: bool,
    clock: &str,
    p: &Palette,
) {
    f.paint_rect(card, 24.0, p.card_elevated);
    let phone_w = 136.0;
    let phone_h = 114.0;
    let px = card.x + (card.width - phone_w) * 0.5;
    let py = card.y + 14.0;
    let frame = Rect {
        x: px,
        y: py,
        width: phone_w,
        height: phone_h,
    };
    f.paint_rect(frame, 18.0, Color(24, 28, 38, 255));
    let screen = frame.inset(4.0);
    f.paint_rect(screen, 14.0, Color(18, 24, 36, 255));

    // Tiny lock icon + clock
    f.paint_system_icon(
        Rect {
            x: screen.x + screen.width * 0.5 - 6.0,
            y: screen.y + 6.0,
            width: 12.0,
            height: 12.0,
        },
        SystemIcon::Security,
        Color(210, 224, 250, 220),
    );
    f.styled_text(
        Rect {
            x: screen.x,
            y: screen.y + 19.0,
            width: screen.width,
            height: 22.0,
        },
        clock,
        Color(240, 245, 255, 250),
        text_style(17.0, 600, 1),
    );

    match mode {
        LockScreenNotifMode::DontShow => {
            f.paint_system_icon(
                Rect {
                    x: screen.x + screen.width * 0.5 - 11.0,
                    y: screen.y + 54.0,
                    width: 22.0,
                    height: 22.0,
                },
                SystemIcon::Silent,
                Color(160, 174, 198, 170),
            );
            f.styled_text(
                Rect {
                    x: screen.x + 8.0,
                    y: screen.y + 78.0,
                    width: screen.width - 16.0,
                    height: 16.0,
                },
                "Hidden on lock",
                Color(168, 180, 202, 190),
                text_style(10.0, 400, 1),
            );
        }
        LockScreenNotifMode::HideSensitive => {
            let nc = Rect {
                x: screen.x + 8.0,
                y: screen.y + 46.0,
                width: screen.width - 16.0,
                height: 36.0,
            };
            f.paint_rect(nc, 10.0, Color(36, 42, 56, 240));
            f.paint_system_icon(
                Rect {
                    x: nc.x + 7.0,
                    y: nc.y + 10.0,
                    width: 16.0,
                    height: 16.0,
                },
                SystemIcon::VisibilityOff,
                p.primary,
            );
            f.styled_text(
                Rect {
                    x: nc.x + 28.0,
                    y: nc.y + 5.0,
                    width: nc.width - 34.0,
                    height: 13.0,
                },
                "Messages",
                Color(238, 242, 252, 245),
                text_style(10.0, 600, 0),
            );
            f.styled_text(
                Rect {
                    x: nc.x + 28.0,
                    y: nc.y + 18.0,
                    width: nc.width - 34.0,
                    height: 13.0,
                },
                "Content hidden",
                Color(175, 186, 206, 210),
                text_style(9.0, 400, 0),
            );
        }
        LockScreenNotifMode::ShowAll | LockScreenNotifMode::HideSilent => {
            let nc = Rect {
                x: screen.x + 8.0,
                y: screen.y + 44.0,
                width: screen.width - 16.0,
                height: 32.0,
            };
            f.paint_rect(nc, 10.0, Color(36, 42, 56, 240));
            f.paint_system_icon(
                Rect {
                    x: nc.x + 7.0,
                    y: nc.y + 8.0,
                    width: 15.0,
                    height: 15.0,
                },
                if show_sensitive {
                    SystemIcon::NotificationsActive
                } else {
                    SystemIcon::VisibilityOff
                },
                p.primary,
            );
            f.styled_text(
                Rect {
                    x: nc.x + 26.0,
                    y: nc.y + 4.0,
                    width: nc.width - 32.0,
                    height: 12.0,
                },
                "Messages",
                Color(238, 242, 252, 245),
                text_style(9.5, 600, 0),
            );
            f.styled_text(
                Rect {
                    x: nc.x + 26.0,
                    y: nc.y + 16.0,
                    width: nc.width - 32.0,
                    height: 12.0,
                },
                if show_sensitive {
                    "New message"
                } else {
                    "Content hidden"
                },
                Color(180, 192, 214, 215),
                text_style(8.5, 400, 0),
            );
            if mode == LockScreenNotifMode::ShowAll {
                let nc2 = Rect {
                    x: screen.x + 8.0,
                    y: screen.y + 80.0,
                    width: screen.width - 16.0,
                    height: 20.0,
                };
                f.paint_rect(nc2, 8.0, Color(30, 35, 47, 215));
                f.styled_text(
                    Rect {
                        x: nc2.x + 8.0,
                        y: nc2.y + 3.0,
                        width: nc2.width - 16.0,
                        height: 14.0,
                    },
                    "Silent alert",
                    Color(164, 176, 198, 200),
                    text_style(8.5, 400, 0),
                );
            }
        }
    }

    f.styled_text(
        Rect {
            x: card.x + 14.0,
            y: card.y + card.height - 28.0,
            width: card.width - 28.0,
            height: 20.0,
        },
        mode.short_label(),
        p.primary,
        text_style(13.0, 500, 1),
    );
}
