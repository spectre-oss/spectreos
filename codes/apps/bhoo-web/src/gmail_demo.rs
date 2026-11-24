//! Interactive Gmail UI mockup rendered from pure imported Android Material 3 XML.
use bhoo::android_xml::{AndroidTheme, LayoutState, Visibility};
use bhoo::*;

#[path = "android_gmail_inbox.rs"]
mod inbox;
#[path = "android_gmail_detail.rs"]
mod detail;
#[path = "android_gmail_compose.rs"]
mod compose;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Inbox,
    Detail(usize),
    Compose,
}

struct EmailMeta {
    sender: &'static str,
    subject: &'static str,
    time: &'static str,
}

const EMAILS: &[EmailMeta] = &[
    EmailMeta {
        sender: "Sam",
        subject: "Photos from my trip!",
        time: "6:11 PM",
    },
    EmailMeta {
        sender: "Promotions",
        subject: "MasterClass - Congrats on finishing your course",
        time: "4 new",
    },
    EmailMeta {
        sender: "Square",
        subject: "Your Square account has been deactivated",
        time: "5:43 PM",
    },
    EmailMeta {
        sender: "Under Armour",
        subject: "Order confirmation for UA US-31884",
        time: "1:23 PM",
    },
    EmailMeta {
        sender: "Pinterest",
        subject: "Pinterest account security alert: Did you log in?",
        time: "10:47 AM",
    },
    EmailMeta {
        sender: "Vercel",
        subject: "Vercel December Product Update",
        time: "9:53 AM",
    },
    EmailMeta {
        sender: "Google One",
        subject: "Your month with Google One",
        time: "9:11 AM",
    },
];

pub struct GmailDemo {
    state: LayoutState,
    initialized: bool,
    screen: Screen,
    active_tab: usize,
    starred: [bool; 7],
    compose_initialized: bool,
    refresh_until: Option<f64>,
    status_toast: Option<(String, f64)>,
}

impl Default for GmailDemo {
    fn default() -> Self {
        Self {
            state: LayoutState::default(),
            initialized: false,
            screen: Screen::Inbox,
            active_tab: 0,
            starred: [false; 7],
            compose_initialized: false,
            refresh_until: None,
            status_toast: None,
        }
    }
}

impl GmailDemo {
    fn ensure_initialized(&mut self) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        self.state.theme_style = AndroidTheme::Material3Light;
        inbox::install_drawables(&mut self.state);
        detail::install_drawables(&mut self.state);
        compose::install_drawables(&mut self.state);
        self.state.selected_index.insert("bottom_nav".into(), 0);
        self.state.selected_index.insert("nav_drawer".into(), 0);
        self.sync_star_icons();
        self.sync_tab_visibility();
    }

    fn sync_star_icons(&mut self) {
        for (i, &is_starred) in self.starred.iter().enumerate() {
            if i == 1 {
                continue;
            }
            let src = if is_starred {
                "ic_star_filled"
            } else {
                "ic_star_outline"
            };
            self.state
                .image_sources
                .insert(format!("star_{i}"), src.into());
        }
        if let Screen::Detail(idx) = self.screen {
            let src = if self.starred[idx.min(self.starred.len() - 1)] {
                "ic_star_filled"
            } else {
                "ic_star_outline"
            };
            self.state
                .image_sources
                .insert("detail_star".into(), src.into());
        }
    }

    fn sync_tab_visibility(&mut self) {
        let mail_vis = if self.active_tab == 0 {
            Visibility::Visible
        } else {
            Visibility::Gone
        };
        let chat_vis = if self.active_tab == 1 {
            Visibility::Visible
        } else {
            Visibility::Gone
        };
        let meet_vis = if self.active_tab == 2 {
            Visibility::Visible
        } else {
            Visibility::Gone
        };
        self.state
            .visibility
            .insert("inbox_swipe_refresh".into(), mail_vis);
        self.state
            .visibility
            .insert("fab_compose".into(), mail_vis);
        self.state
            .visibility
            .insert("tab_chat_view".into(), chat_vis);
        self.state
            .visibility
            .insert("tab_meet_view".into(), meet_vis);
        self.state
            .selected_index
            .insert("bottom_nav".into(), self.active_tab);
    }

    fn open_detail(&mut self, idx: usize) {
        let idx = idx.min(EMAILS.len() - 1);
        let meta = &EMAILS[idx];
        self.screen = Screen::Detail(idx);
        self.state
            .drawer_open
            .insert("drawer_layout".into(), false);
        self.state.offsets.insert("detail_scroll".into(), 0.0);
        self.state
            .text
            .insert("detail_subject".into(), meta.subject.into());
        self.state
            .text
            .insert("detail_sender".into(), meta.sender.into());
        self.state
            .text
            .insert("detail_time".into(), meta.time.into());
        self.state
            .text
            .insert("detail_signoff".into(), meta.sender.into());
        self.sync_star_icons();
    }

    fn open_compose(&mut self, text: &mut String, quick_reply: Option<&str>) {
        if let Some(reply) = quick_reply {
            *text = format!("Hey Sam,\n\n{reply}\n\nBest,\nAlex");
            self.compose_initialized = true;
            self.state
                .text
                .insert("compose_subject".into(), "Re: Photos from my trip!".into());
        } else if !self.compose_initialized {
            *text = "Hey Sam,\n\nHow's it going? I attached some pictures from my recent trip to Japan! Can't wait to visit you soon.\n\nBest,\nAlex".into();
            self.compose_initialized = true;
            self.state
                .text
                .entry("compose_to".into())
                .or_insert_with(|| "samlee.mobbin@gmail.com & 1 more".into());
            self.state
                .text
                .entry("compose_subject".into())
                .or_insert_with(|| "Catch up soon?".into());
        }
        self.state
            .drawer_open
            .insert("drawer_layout".into(), false);
        self.state.text.insert("compose_body".into(), text.clone());
        self.screen = Screen::Compose;
    }

    fn edit_key_for_id(id: u64) -> Option<&'static str> {
        ["compose_to", "compose_subject", "compose_body"]
            .into_iter()
            .find(|name| bhoo::android_xml::view_id(name) == id)
    }

    pub fn edit_text_for(&self, id: u64) -> &str {
        Self::edit_key_for_id(id)
            .and_then(|key| self.state.text.get(key).map(String::as_str))
            .unwrap_or("")
    }

    pub fn set_edit_text(&mut self, id: u64, value: &str, body_fallback: &mut String) {
        if let Some(key) = Self::edit_key_for_id(id) {
            self.state.text.insert(key.into(), value.into());
            if key == "compose_body" {
                body_fallback.clear();
                body_fallback.push_str(value);
            }
        }
    }

    fn show_toast(&mut self, f: &mut Frame<'_>, message: &str, duration_s: f64) {
        let until = f.time() + duration_s;
        self.status_toast = Some((message.into(), until));
        f.wake_at(until);
    }

    pub fn show(&mut self, f: &mut Frame<'_>, width: f32, height: f32, text: &mut String) {
        self.show_in(
            f,
            Rect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            text,
        );
    }

    pub fn show_in(&mut self, f: &mut Frame<'_>, bounds: Rect, text: &mut String) {
        self.ensure_initialized();
        let width = bounds.width;
        let height = bounds.height;

        if let Some(until) = self.refresh_until {
            if f.time() >= until {
                self.refresh_until = None;
                self.state
                    .refreshing
                    .insert("inbox_swipe_refresh".into(), false);
            } else {
                f.wake_at(until);
            }
        }

        if self.screen == Screen::Compose {
            self.state.text.insert("compose_body".into(), text.clone());
        }

        let mut action = None;
        f.within(bounds, |f| {
            action = match self.screen {
                Screen::Inbox => inbox::LAYOUT.render(f, bounds, &mut self.state),
                Screen::Detail(_) => detail::LAYOUT.render(f, bounds, &mut self.state),
                Screen::Compose => compose::LAYOUT.render(f, bounds, &mut self.state),
            };
        });

        if let Some(action) = action {
            self.handle_action(f, action, text);
            f.wake_at(f.time() + 0.001);
        }

        if let Some((msg, until)) = &self.status_toast {
            if f.time() < *until {
                let toast_w = (width - 32.0).min(320.0);
                let toast_y = if self.screen == Screen::Inbox {
                    bounds.y + (height - 134.0).max(16.0)
                } else {
                    bounds.y + (height - 64.0).max(16.0)
                };
                let toast_rect = Rect {
                    x: bounds.x + (width - toast_w) * 0.5,
                    y: toast_y,
                    width: toast_w,
                    height: 44.0,
                };
                f.elevation(toast_rect, 8.0, 3);
                f.paint_rect(toast_rect, 8.0, Color(50, 47, 53, 255));
                f.styled_text(
                    Rect {
                        x: toast_rect.x + 16.0,
                        y: toast_rect.y,
                        width: (toast_rect.width - 32.0).max(0.0),
                        height: toast_rect.height,
                    },
                    msg,
                    Color(245, 239, 247, 255),
                    TextStyle {
                        size: 14.0,
                        weight: 500,
                        ..TextStyle::default()
                    },
                );
                f.wake_at(*until);
            } else {
                self.status_toast = None;
            }
        }
    }

    fn handle_action(&mut self, f: &mut Frame<'_>, action: &str, text: &mut String) {
        match self.screen {
            Screen::Inbox => match action {
                "menu_button" | "search_bar" => {
                    let open = self
                        .state
                        .drawer_open
                        .get("drawer_layout")
                        .copied()
                        .unwrap_or(false);
                    self.state
                        .drawer_open
                        .insert("drawer_layout".into(), !open);
                }
                "account_avatar" => {
                    self.state
                        .refreshing
                        .insert("inbox_swipe_refresh".into(), true);
                    let until = f.time() + 0.9;
                    self.refresh_until = Some(until);
                    f.wake_at(until);
                    self.show_toast(f, "Syncing aallexsmiith@gmail.com…", 1.6);
                }
                "fab_compose" => {
                    self.open_compose(text, None);
                }
                "nav_mail" => {
                    self.active_tab = 0;
                    self.sync_tab_visibility();
                }
                "nav_chat" => {
                    self.active_tab = 1;
                    self.sync_tab_visibility();
                }
                "nav_meet" => {
                    self.active_tab = 2;
                    self.sync_tab_visibility();
                }
                "drawer_primary" | "drawer_promotions" | "drawer_starred" | "drawer_sent"
                | "drawer_drafts" | "drawer_trash" | "drawer_settings" => {
                    let label = match action {
                        "drawer_primary" => "Primary",
                        "drawer_promotions" => "Promotions",
                        "drawer_starred" => "Starred",
                        "drawer_sent" => "Sent",
                        "drawer_drafts" => "Drafts",
                        "drawer_trash" => "Trash",
                        _ => "Settings",
                    };
                    self.state
                        .drawer_open
                        .insert("drawer_layout".into(), false);
                    self.active_tab = 0;
                    self.sync_tab_visibility();
                    self.state
                        .text
                        .insert("section_header".into(), label.into());
                    self.show_toast(f, &format!("Folder: {label}"), 1.6);
                }
                a if a.starts_with("email_row_") => {
                    if let Ok(idx) = a["email_row_".len()..].parse::<usize>() {
                        self.open_detail(idx);
                    }
                }
                a if a.starts_with("star_") => {
                    if let Ok(idx) = a["star_".len()..].parse::<usize>()
                        && idx < self.starred.len()
                    {
                        self.starred[idx] = !self.starred[idx];
                        self.sync_star_icons();
                    }
                }
                _ => {}
            },
            Screen::Detail(idx) => match action {
                "detail_toolbar" => {
                    if self.state.toolbar_navigation_clicked.remove("detail_toolbar") {
                        self.screen = Screen::Inbox;
                    }
                }
                "detail_archive" => {
                    self.screen = Screen::Inbox;
                    self.show_toast(f, "Conversation archived", 1.8);
                }
                "detail_delete" => {
                    self.screen = Screen::Inbox;
                    self.show_toast(f, "Conversation moved to Trash", 1.8);
                }
                "detail_mark_unread" => {
                    self.screen = Screen::Inbox;
                    self.show_toast(f, "Marked as unread", 1.6);
                }
                "detail_more" => {
                    self.show_toast(f, "More message actions", 1.5);
                }
                "detail_star" => {
                    if idx < self.starred.len() {
                        self.starred[idx] = !self.starred[idx];
                        self.sync_star_icons();
                    }
                }
                "detail_react" => {
                    self.show_toast(f, "Reactions (demo)", 1.5);
                }
                "detail_reply" => {
                    self.open_compose(text, None);
                }
                "detail_sender_more" => {
                    self.show_toast(f, "Message options", 1.5);
                }
                "photo_card_1" => {
                    self.show_toast(f, "Previewing Sakura sky", 1.5);
                }
                "photo_card_2" => {
                    self.show_toast(f, "Previewing Kyoto canal", 1.5);
                }
                "quick_reply_great" => {
                    self.open_compose(text, Some("Great pictures!"));
                }
                "quick_reply_amazing" => {
                    self.open_compose(text, Some("Amazing photos!"));
                }
                _ => {}
            },
            Screen::Compose => match action {
                "compose_toolbar" => {
                    if self
                        .state
                        .toolbar_navigation_clicked
                        .remove("compose_toolbar")
                    {
                        self.screen = Screen::Inbox;
                    }
                }
                "compose_attach" => {
                    self.show_toast(f, "Attachment picker (demo)", 1.5);
                }
                "compose_send" => {
                    self.screen = Screen::Inbox;
                    self.show_toast(f, "Message sent", 2.0);
                }
                "compose_more" => {
                    self.show_toast(f, "Draft options", 1.5);
                }
                _ => {}
            },
        }
    }
}
