//! Native Wayland + GLES capture for Wisp (the SpectreOS System UI & Launcher in
//! `codes/apps/wisp`) at `411 x 846` (1:1 phone dp aspect ratio of `1440 x 2960`).
//!
//! Select preset via `BHOO_LAUNCHER_SCREEN=home|drawer|shade|qs|power|volume|overview|split|freeform|settings`.
//! Pass `--smoke-test` with `BHOO_CAPTURE=/path/to/out.ppm` to render and capture a frame.
use bhoo::window::{self, FrameAction, WindowOptions};

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/contacts_demo.rs"]
mod contacts_demo;

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/gmail_demo.rs"]
mod gmail_demo;

#[allow(dead_code)]
#[path = "../../../../apps/wisp/src/lib.rs"]
mod wisp;

#[derive(Default)]
struct CaptureEmbeddedApps {
    contacts: Box<contacts_demo::ContactsDemo>,
    gmail: Box<gmail_demo::GmailDemo>,
}

impl wisp::EmbeddedApps for CaptureEmbeddedApps {
    fn render_contacts(&mut self, f: &mut bhoo::Frame<'_>, bounds: bhoo::Rect) {
        self.contacts.show_in(f, bounds);
    }

    fn open_new_contact(&mut self, f: &mut bhoo::Frame<'_>) {
        self.contacts.open_new_contact(f);
    }

    fn render_gmail(&mut self, f: &mut bhoo::Frame<'_>, bounds: bhoo::Rect, compose_body: &mut String) {
        self.gmail.show_in(f, bounds, compose_body);
    }

    fn edit_text_for(&self, id: u64) -> &str {
        let c = self.contacts.edit_text_for(id);
        if !c.is_empty() { c } else { self.gmail.edit_text_for(id) }
    }

    fn set_edit_text(&mut self, id: u64, value: &str) -> bool {
        self.contacts.set_edit_text(id, value);
        let mut body = String::new();
        self.gmail.set_edit_text(id, value, &mut body);
        id == bhoo::android_xml::view_id("compose_body")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|x| x == "--smoke-test");
    let preset = std::env::var("BHOO_LAUNCHER_SCREEN").unwrap_or_else(|_| "home".into());
    let win_w: u32 = std::env::var("BHOO_CAPTURE_W")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(411);
    let win_h: u32 = std::env::var("BHOO_CAPTURE_H")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(846);
    let mut demo = Box::new(wisp::WispShell::<CaptureEmbeddedApps>::default());
    demo.set_capture_preset(&preset);
    let mut gmail_text = String::new();
    let mut frames = 0;

    window::run(
        WindowOptions {
            title: format!("Bhoo / Android Launcher ({preset})"),
            size: [win_w, win_h],
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            ..WindowOptions::default()
        },
        move |f| {
            frames += 1;
            demo.show(
                f,
                win_w as f32,
                win_h as f32,
                bhoo::Input::default(),
                &mut gmail_text,
            );
            if smoke && frames >= 4 {
                FrameAction::Exit
            } else if smoke {
                FrameAction::Redraw
            } else {
                FrameAction::Wait
            }
        },
    )
}
