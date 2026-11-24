//! SpectreOS Default On-Screen Keyboard (`spectre-keyboard`).
//!
//! Runs natively as a standalone Wayland + Freedesktop D-Bus OSK daemon (`org.spectreos.OSK0`)
//! rendered via `libbhoo::GlesRenderer`, or in an interactive desktop preview window with
//! `--preview` (when built with `--features preview`).

use spectre_keyboard::layout::LayoutId;

fn print_help() {
    println!(
        "spectre-keyboard 0.1.0 — SpectreOS Default On-Screen Keyboard (Rust + libbhoo)\n\
         \n\
         USAGE:\n\
         \tspectre-keyboard [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \t--layout <us|ir|terminal>   Initial keyboard layout (default: us)\n\
         \t--show                      Map the keyboard immediately on startup\n\
         \t--no-compat-dbus            Only claim org.spectreos.OSK0 (do not alias sm.puri.OSK0)\n\
         \t--allow-replacement         Accepted for OSK service compatibility (always enabled)\n\
         \t--preview                   Open interactive desktop preview window (requires preview feature)\n\
         \t-h, --help                  Print this help message"
    );
}

fn main() {
    let mut initial_layout = LayoutId::Us;
    let mut start_visible = false;
    let mut compat_dbus = true;
    let mut preview = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "--show" => start_visible = true,
            "--no-compat-dbus" => compat_dbus = false,
            "--allow-replacement" => {}
            "--preview" => preview = true,
            "--layout" => {
                if let Some(val) = args.next() {
                    if let Some(id) = LayoutId::parse(&val) {
                        initial_layout = id;
                    } else {
                        eprintln!("spectre-keyboard: unknown layout '{val}' (expected us, ir, or terminal)");
                        std::process::exit(2);
                    }
                }
            }
            other => {
                eprintln!("spectre-keyboard: ignoring unknown argument '{other}'");
            }
        }
    }

    if preview {
        #[cfg(feature = "preview")]
        {
            if let Err(err) = run_preview(initial_layout) {
                eprintln!("spectre-keyboard preview error: {err}");
                std::process::exit(1);
            }
            return;
        }
        #[cfg(not(feature = "preview"))]
        {
            eprintln!("spectre-keyboard: rebuild with `--features preview` to run --preview mode");
            std::process::exit(2);
        }
    }

    #[cfg(feature = "osk")]
    {
        let opts = spectre_keyboard::wayland::DaemonOptions {
            initial_layout,
            start_visible,
            compat_dbus,
        };
        if let Err(err) = spectre_keyboard::wayland::run_daemon(opts) {
            eprintln!("spectre-keyboard: {err}");
            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "osk"))]
    {
        let _ = (initial_layout, start_visible, compat_dbus);
        eprintln!("spectre-keyboard: compiled without `osk` feature; pass --preview or enable `osk`");
        std::process::exit(2);
    }
}

#[cfg(feature = "preview")]
fn run_preview(initial_layout: LayoutId) -> Result<(), Box<dyn std::error::Error>> {
    use bhoo::{
        Color, DrawCommand, FontFamily, Rect, TextStyle,
        window::{self, FrameAction, WindowOptions},
    };
    use spectre_keyboard::{KeyboardEngine, KeyboardOutput, SpecialKey, touch::erase_last_grapheme_bytes};

    let mut engine = KeyboardEngine::new(initial_layout);
    let mut text = String::from("Salam / سلام ");
    let mut was_pressed = false;
    let width = 420u32;
    let scratch_h = 96.0_f32;
    let kb_h = engine.desired_height(width as f32).ceil() as u32;

    window::run(
        WindowOptions {
            title: "SpectreOS Keyboard Preview".into(),
            app_id: Some("org.spectreos.Keyboard".into()),
            size: [width, scratch_h as u32 + kb_h],
            decorations: true,
            ..WindowOptions::default()
        },
        move |f| {
            let vp = f.viewport();
            let now = f.input().time;
            let kb_h = engine.desired_height_for_viewport(vp.width, vp.height);
            let kb_y = (vp.height - kb_h).max(64.0);
            let kb_bounds = Rect {
                x: 0.0,
                y: kb_y,
                width: vp.width,
                height: (vp.height - kb_y).max(120.0),
            };

            let mut outputs = engine.tick(now, kb_bounds.width, kb_bounds.height);
            if let Some([px, py]) = f.input().pointer {
                let local_x = px - kb_bounds.x;
                let local_y = py - kb_bounds.y;
                if f.input().pressed && py >= kb_y {
                    was_pressed = true;
                    outputs.extend(engine.pointer_down(local_x, local_y, kb_bounds.width, kb_bounds.height, now));
                } else if f.input().released && was_pressed {
                    was_pressed = false;
                    outputs.extend(engine.pointer_up(local_x, local_y, kb_bounds.width, kb_bounds.height, now));
                } else if was_pressed {
                    outputs.extend(engine.pointer_move(local_x, local_y, kb_bounds.width, kb_bounds.height, now));
                }
            } else if f.input().cancelled && was_pressed {
                was_pressed = false;
                engine.cancel_press();
            }

            let mut exit = false;
            for ev in outputs {
                match ev {
                    KeyboardOutput::CommitText { text: s, .. } => text.push_str(&s),
                    KeyboardOutput::SpecialKey { key: SpecialKey::Backspace, .. } => {
                        let (bytes, _) = erase_last_grapheme_bytes(&text, text.len());
                        let new_len = text.len().saturating_sub(bytes);
                        text.truncate(new_len);
                    }
                    KeyboardOutput::SpecialKey { key: SpecialKey::Enter, .. } => text.push('\n'),
                    KeyboardOutput::SpecialKey { key: SpecialKey::Tab, .. } => text.push('\t'),
                    KeyboardOutput::HideRequested => exit = true,
                    _ => {}
                }
            }

            f.clear(Color(24, 24, 28, 255));
            f.paint_rect(
                Rect {
                    x: 12.0,
                    y: 12.0,
                    width: (vp.width - 24.0).max(40.0),
                    height: scratch_h - 24.0,
                },
                12.0,
                Color(38, 39, 46, 255),
            );
            f.styled_text(
                Rect {
                    x: 24.0,
                    y: 16.0,
                    width: (vp.width - 48.0).max(40.0),
                    height: scratch_h - 32.0,
                },
                &format!("{text}|"),
                Color(240, 240, 246, 255),
                TextStyle {
                    size: 18.0,
                    weight: 400,
                    align: 0,
                    padding: 0.0,
                    letter_spacing: 0.0,
                    italic: false,
                    family: FontFamily::Sans,
                },
            );

            let mut cmds = Vec::with_capacity(200);
            engine.render(kb_bounds, &mut cmds);
            for cmd in cmds {
                match cmd {
                    DrawCommand::Clear(c) => f.clear(c),
                    DrawCommand::RoundedRect { bounds, radius, color } => {
                        f.paint_rect(bounds, radius, color);
                    }
                    DrawCommand::Ellipse { bounds, color } => {
                        let r = bounds.width.min(bounds.height) * 0.5;
                        f.paint_rect(bounds, r, color);
                    }
                    DrawCommand::Outline { bounds, radius, width: w, color } => {
                        if radius >= bounds.width.min(bounds.height) * 0.45 {
                            f.paint_arc(bounds, 0.0, std::f32::consts::TAU, w, color);
                        } else {
                            let x0 = bounds.x;
                            let y0 = bounds.y;
                            let x1 = bounds.x + bounds.width;
                            let y1 = bounds.y + bounds.height;
                            f.paint_line([x0, y0], [x1, y0], w, color);
                            f.paint_line([x1, y0], [x1, y1], w, color);
                            f.paint_line([x1, y1], [x0, y1], w, color);
                            f.paint_line([x0, y1], [x0, y0], w, color);
                        }
                    }
                    DrawCommand::Text { bounds, text, color, style } => {
                        f.styled_text(bounds, &text, color, style);
                    }
                    DrawCommand::Shadow { bounds, radius, offset, blur, spread, color } => {
                        f.shadow(bounds, radius, offset, blur, spread, color);
                    }
                    DrawCommand::Line { from, to, width: w, color } => {
                        f.paint_line(from, to, w, color);
                    }
                    DrawCommand::Polyline { points, count, width: w, color } => {
                        let n = (count as usize).min(points.len());
                        for pair in points[..n].windows(2) {
                            f.paint_line(pair[0], pair[1], w, color);
                        }
                    }
                    DrawCommand::Arc { bounds, start, sweep, stroke, color, .. } => {
                        f.paint_arc(bounds, start, sweep, stroke, color);
                    }
                    _ => {}
                }
            }

            if exit {
                FrameAction::Exit
            } else if engine.next_timer_deadline().is_some() {
                FrameAction::Redraw
            } else {
                FrameAction::Wait
            }
        },
    )
}
