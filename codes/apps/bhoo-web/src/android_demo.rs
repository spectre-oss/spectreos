//! Offline demonstration of imported Android XML using Bhoo widgets.
use bhoo::*;
#[path = "android_chats.rs"]
mod chats;
#[path = "android_conversation.rs"]
mod conversation;
#[derive(Default)]
pub struct AndroidDemo {
    pub state: android_xml::LayoutState,
    conversation: bool,
    dialog: OverlayState,
}
impl AndroidDemo {
    pub fn show(&mut self, f: &mut Frame<'_>, width: f32, height: f32, text: &mut String) {
        self.state.text.insert("message".into(), text.clone());
        let bounds = Rect {
            x: 0.,
            y: 0.,
            width,
            height,
        };
        let layout = if self.conversation {
            &conversation::LAYOUT
        } else {
            &chats::LAYOUT
        };
        let action = layout.render(f, bounds, &mut self.state);
        if let Some(action) = action {
            match action {
                "back" => {
                    self.conversation = false;
                    text.clear();
                }
                "compose" => {
                    self.conversation = true;
                    self.state
                        .text
                        .insert("chat_title".into(), "New demo chat".into());
                }
                "send" => {
                    if !text.trim().is_empty() {
                        self.state.text.insert("last_message".into(), text.clone());
                        text.clear();
                    }
                }
                a if a.starts_with("chat_") => {
                    let names = [
                        "Alex Brown",
                        "SpectreOS team",
                        "Mina Lee",
                        "Saved messages",
                        "Book club",
                        "Family",
                        "Design group",
                        "Jamie Bell",
                        "Weekend plans",
                        "Bhoo lab",
                        "Nora Kim",
                        "Community",
                    ];
                    let i = a[5..].parse::<usize>().unwrap_or(0).min(names.len() - 1);
                    self.state.text.insert("chat_title".into(), names[i].into());
                    self.conversation = true;
                }
                _ => self.dialog.open = true,
            }
            f.wake_at(f.time() + 0.001);
        }
        self.dialog.kind = OverlayKind::Dialog;
        let w = (width - 48.).min(480.);
        f.overlay(
            99000,
            &mut self.dialog,
            Rect {
                x: (width - w) / 2.,
                y: (height - 260.) / 2.,
                width: w,
                height: 260.,
            },
            true,
            |f, open| {
                f.label("Delta Chat UI · Bhoo demo");
                f.label("Built from Android XML into Rust.");
                f.label("Fake data. Nothing is sent.");
                f.label("Menu/search are demo placeholders.");
                if f.button(99001, "Close").clicked {
                    *open = false;
                }
            },
        );
    }
}
