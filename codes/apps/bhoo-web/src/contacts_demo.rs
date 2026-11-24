//! Interactive Android Contacts (com.android.contacts v1.7.34) dummy app rendered from pure imported Android XML.
use bhoo::android_xml::{AndroidTheme, LayoutState, Visibility, view_id};
use bhoo::*;

#[path = "android_contacts_home.rs"]
mod home;
#[path = "android_contacts_detail.rs"]
mod detail;
#[path = "android_contacts_editor.rs"]
mod editor;
#[path = "android_contacts_settings.rs"]
mod settings;

const PHONE_TYPES: &[&str] = &[
    "Mobile", "Work", "Home", "Main", "Work Fax", "Home Fax", "Pager", "Other", "Custom",
];
const EMAIL_TYPES: &[&str] = &["Home", "Work", "Other"];
const RINGTONES: &[&str] = &["Default", "Argon", "Carbon", "Helium", "Krypton"];
const MAX_ROWS: usize = 10;
const MAX_DRAWER_LABELS: usize = 4;
const MAX_PICKER_ROWS: usize = 6;

#[derive(Clone, Debug)]
struct Contact {
    id: u32,
    prefix: String,
    first_name: String,
    middle_name: String,
    last_name: String,
    suffix: String,
    phone: String,
    phone_type: usize,
    email: String,
    email_type: usize,
    org: String,
    title: String,
    address: String,
    notes: String,
    starred: bool,
    color_idx: usize,
    ringtone: usize,
    labels: Vec<u32>,
}

impl Contact {
    fn display_name(&self, last_first: bool) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if last_first {
            if !self.last_name.trim().is_empty() {
                parts.push(self.last_name.trim());
            }
            if !self.first_name.trim().is_empty() {
                parts.push(self.first_name.trim());
            }
            if parts.len() == 2 {
                return format!("{}, {}", parts[0], parts[1]);
            }
        } else {
            if !self.prefix.trim().is_empty() {
                parts.push(self.prefix.trim());
            }
            if !self.first_name.trim().is_empty() {
                parts.push(self.first_name.trim());
            }
            if !self.middle_name.trim().is_empty() {
                parts.push(self.middle_name.trim());
            }
            if !self.last_name.trim().is_empty() {
                parts.push(self.last_name.trim());
            }
            if !self.suffix.trim().is_empty() {
                parts.push(self.suffix.trim());
            }
        }
        let joined = parts.join(" ");
        if !joined.is_empty() {
            joined
        } else if !self.org.trim().is_empty() {
            self.org.trim().to_owned()
        } else if !self.phone.trim().is_empty() {
            self.phone.trim().to_owned()
        } else if !self.email.trim().is_empty() {
            self.email.trim().to_owned()
        } else {
            "(No name)".to_owned()
        }
    }

    fn sort_key(&self, sort_by_last: bool) -> String {
        if sort_by_last {
            format!(
                "{} {}",
                self.last_name.trim().to_ascii_lowercase(),
                self.first_name.trim().to_ascii_lowercase()
            )
            .trim()
            .to_owned()
        } else {
            format!(
                "{} {}",
                self.first_name.trim().to_ascii_lowercase(),
                self.last_name.trim().to_ascii_lowercase()
            )
            .trim()
            .to_owned()
        }
    }

    fn initial(&self, sort_by_last: bool) -> String {
        let key = self.sort_key(sort_by_last);
        key.chars()
            .next()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_else(|| "#".into())
    }

    fn first_or_display(&self) -> String {
        if !self.first_name.trim().is_empty() {
            self.first_name.trim().to_owned()
        } else {
            self.display_name(false)
        }
    }
}

#[derive(Clone, Debug)]
struct Label {
    id: u32,
    name: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Detail(u32),
    Editor,
    Settings(SettingsSub),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsSub {
    Main,
    About,
    Emergency,
    Blocked,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorTarget {
    NewContact,
    EditContact(u32),
    MyInfo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HomeDialog {
    None,
    CreateLabel,
    RenameLabel(u32),
    DeleteLabel(u32),
    DeleteSelected,
    AddToLabel(u32),
    RemoveFromLabel(u32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DetailDialog {
    None,
    Delete,
    Ringtone,
    Link,
    Share,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorDialog {
    None,
    ChangePhoto,
    Discard,
    PhoneType,
    EmailType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsDialog {
    None,
    DefaultAccount,
    ContactsToDisplay,
    SortBy,
    NameFormat,
    PhoneticName,
    Import,
    Export,
}

#[derive(Clone, Copy)]
struct DisplayedRow {
    contact_id: u32,
    is_fav_section: bool,
    show_star_icon: bool,
    letter: char,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TransitionKind {
    Push,
    Pop,
    ModalUp,
    ModalDown,
}

#[derive(Clone, Copy)]
struct ScreenTransition {
    from: Screen,
    to: Screen,
    kind: TransitionKind,
    start: f64,
    duration: f64,
}

#[derive(Clone, Copy, Default)]
struct OverlayAnim {
    target_open: bool,
    progress: f32,
    anim: Option<(f64, f32, f32, f64, bool)>,
}

fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0 || x >= 1.0 {
        return x;
    }
    let mut t = x;
    for _ in 0..6 {
        let u = 1.0 - t;
        let bx = 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t;
        let dbx = 3.0 * u * u * x1 + 6.0 * u * t * (x2 - x1) + 3.0 * t * t * (1.0 - x2);
        if dbx.abs() < 1e-6 {
            break;
        }
        t = (t - (bx - x) / dbx).clamp(0.0, 1.0);
    }
    let u = 1.0 - t;
    (3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t).clamp(0.0, 1.0)
}

fn emphasized_decelerate(t: f32) -> f32 {
    cubic_bezier(0.05, 0.7, 0.1, 1.0, t)
}

fn emphasized_accelerate(t: f32) -> f32 {
    cubic_bezier(0.3, 0.0, 0.8, 0.15, t)
}

impl OverlayAnim {
    fn step(&mut self, want_open: bool, now: f64, reduced: bool, enter_dur: f64, exit_dur: f64) -> (bool, bool, f32) {
        if want_open != self.target_open {
            self.target_open = want_open;
            let to = if want_open { 1.0 } else { 0.0 };
            if reduced || now == 0.0 {
                self.progress = to;
                self.anim = None;
            } else {
                let dur = if want_open { enter_dur } else { exit_dur };
                self.anim = Some((now, self.progress, to, dur, want_open));
            }
        }
        let mut animating = false;
        if let Some((start, from, to, dur, entering)) = self.anim {
            if reduced || dur <= 0.0 {
                self.progress = to;
                self.anim = None;
            } else {
                let raw = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                let eased = if entering {
                    emphasized_decelerate(raw)
                } else {
                    emphasized_accelerate(raw)
                };
                self.progress = from + (to - from) * eased;
                if raw >= 1.0 {
                    self.progress = to;
                    self.anim = None;
                } else {
                    animating = true;
                }
            }
        }
        let visible = self.target_open || self.progress > 0.002;
        (visible, animating, self.progress.clamp(0.0, 1.0))
    }
}

fn apply_dialog_anim(
    state: &mut LayoutState,
    anim: &mut OverlayAnim,
    want_open: bool,
    scrim_id: &str,
    card_id: &str,
    now: f64,
    reduced: bool,
) -> bool {
    let (visible, animating, p) = anim.step(want_open, now, reduced, 0.22, 0.15);
    state.visibility.insert(scrim_id.into(), vis(visible));
    if !visible || p >= 0.998 {
        state.view_alpha.remove(scrim_id);
        state.view_alpha.remove(card_id);
        state.view_scale.remove(card_id);
        state.view_translation.remove(card_id);
    } else {
        state.view_alpha.insert(scrim_id.into(), p);
        state.view_alpha.insert(card_id.into(), (p * 1.35).min(1.0));
        let s = 0.88 + 0.12 * p;
        state.view_scale.insert(card_id.into(), [s, s]);
        state.view_translation.insert(card_id.into(), [0.0, 10.0 * (1.0 - p)]);
    }
    animating
}

fn apply_popup_anim(
    state: &mut LayoutState,
    anim: &mut OverlayAnim,
    want_open: bool,
    popup_id: &str,
    now: f64,
    reduced: bool,
) -> bool {
    let (visible, animating, p) = anim.step(want_open, now, reduced, 0.18, 0.13);
    state.visibility.insert(popup_id.into(), vis(visible));
    if !visible || p >= 0.998 {
        state.view_alpha.remove(popup_id);
        state.view_scale.remove(popup_id);
        state.view_translation.remove(popup_id);
    } else {
        state.view_alpha.insert(popup_id.into(), p);
        let s = 0.90 + 0.10 * p;
        state.view_scale.insert(popup_id.into(), [s, s]);
        state.view_translation.insert(popup_id.into(), [0.0, -8.0 * (1.0 - p)]);
    }
    animating
}

fn paint_page_edge_shadow(f: &mut Frame<'_>, edge_x: f32, top_y: f32, height: f32, strength: f32) {
    if strength <= 0.01 {
        return;
    }
    let strips: [(f32, f32); 4] = [(12.0, 0.04), (8.0, 0.07), (4.0, 0.11), (1.5, 0.16)];
    for (w, a) in strips {
        let alpha = (a * strength * 255.0).round().clamp(0.0, 255.0) as u8;
        if alpha > 0 {
            f.paint_rect(
                Rect {
                    x: edge_x - w,
                    y: top_y,
                    width: w,
                    height,
                },
                0.0,
                Color(0, 0, 0, alpha),
            );
        }
    }
}

pub struct ContactsDemo {
    state: LayoutState,
    initialized: bool,
    screen: Screen,
    transition: Option<ScreenTransition>,
    home_dialog_anim: OverlayAnim,
    detail_dialog_anim: OverlayAnim,
    editor_dialog_anim: OverlayAnim,
    settings_dialog_anim: OverlayAnim,
    label_popup_anim: OverlayAnim,
    detail_popup_anim: OverlayAnim,
    contacts: Vec<Contact>,
    next_contact_id: u32,
    labels: Vec<Label>,
    next_label_id: u32,
    active_label: Option<u32>,
    my_info_name: String,
    my_info_phone: String,
    my_info_email: String,
    // Home modes
    search_mode: bool,
    cab_mode: bool,
    selected_ids: Vec<u32>,
    displayed_rows: Vec<DisplayedRow>,
    label_overflow_open: bool,
    home_dialog: HomeDialog,
    picker_contact_ids: Vec<u32>,
    picker_checked: [bool; MAX_PICKER_ROWS],
    // Detail state
    detail_overflow_open: bool,
    detail_dialog: DetailDialog,
    detail_link_ids: Vec<u32>,
    // Editor state
    editor_target: EditorTarget,
    editor_name_expanded: bool,
    editor_more_fields: bool,
    editor_phone_type: usize,
    editor_email_type: usize,
    editor_color_idx: usize,
    editor_dirty: bool,
    editor_dialog: EditorDialog,
    // Settings state
    default_account: usize,
    display_mode: usize,
    sort_by_last: bool,
    last_name_first: bool,
    phonetic_always: bool,
    settings_dialog: SettingsDialog,
    settings_radio_choice: usize,
    emerg_info_count: u32,
    emerg_contact_count: u32,
    blocked_numbers: Vec<String>,
    // Toast
    status_toast: Option<(String, f64)>,
}

impl Default for ContactsDemo {
    fn default() -> Self {
        Self {
            state: LayoutState::default(),
            initialized: false,
            screen: Screen::Home,
            transition: None,
            home_dialog_anim: OverlayAnim::default(),
            detail_dialog_anim: OverlayAnim::default(),
            editor_dialog_anim: OverlayAnim::default(),
            settings_dialog_anim: OverlayAnim::default(),
            label_popup_anim: OverlayAnim::default(),
            detail_popup_anim: OverlayAnim::default(),
            contacts: vec![
                Contact {
                    id: 1,
                    prefix: String::new(),
                    first_name: "Alice".into(),
                    middle_name: String::new(),
                    last_name: "Johnson".into(),
                    suffix: String::new(),
                    phone: "+1 555-010-1234".into(),
                    phone_type: 0,
                    email: "alice@example.com".into(),
                    email_type: 0,
                    org: "Acme Corp".into(),
                    title: String::new(),
                    address: String::new(),
                    notes: String::new(),
                    starred: true,
                    color_idx: 0,
                    ringtone: 0,
                    labels: vec![1],
                },
                Contact {
                    id: 2,
                    prefix: String::new(),
                    first_name: "Bob".into(),
                    middle_name: String::new(),
                    last_name: "Smith".into(),
                    suffix: String::new(),
                    phone: "+1 555-010-5678".into(),
                    phone_type: 1,
                    email: "bob@example.com".into(),
                    email_type: 1,
                    org: "Northwind".into(),
                    title: "Engineer".into(),
                    address: String::new(),
                    notes: String::new(),
                    starred: false,
                    color_idx: 1,
                    ringtone: 0,
                    labels: Vec::new(),
                },
                Contact {
                    id: 3,
                    prefix: String::new(),
                    first_name: "Charlie".into(),
                    middle_name: String::new(),
                    last_name: "Lee".into(),
                    suffix: String::new(),
                    phone: "+1 555-010-9012".into(),
                    phone_type: 0,
                    email: "charlie@example.com".into(),
                    email_type: 0,
                    org: String::new(),
                    title: String::new(),
                    address: String::new(),
                    notes: String::new(),
                    starred: false,
                    color_idx: 2,
                    ringtone: 0,
                    labels: Vec::new(),
                },
            ],
            next_contact_id: 4,
            labels: vec![Label {
                id: 1,
                name: "Work".into(),
            }],
            next_label_id: 2,
            active_label: None,
            my_info_name: String::new(),
            my_info_phone: String::new(),
            my_info_email: String::new(),
            search_mode: false,
            cab_mode: false,
            selected_ids: Vec::new(),
            displayed_rows: Vec::new(),
            label_overflow_open: false,
            home_dialog: HomeDialog::None,
            picker_contact_ids: Vec::new(),
            picker_checked: [false; MAX_PICKER_ROWS],
            detail_overflow_open: false,
            detail_dialog: DetailDialog::None,
            detail_link_ids: Vec::new(),
            editor_target: EditorTarget::NewContact,
            editor_name_expanded: false,
            editor_more_fields: false,
            editor_phone_type: 0,
            editor_email_type: 0,
            editor_color_idx: 3,
            editor_dirty: false,
            editor_dialog: EditorDialog::None,
            default_account: 0,
            display_mode: 0,
            sort_by_last: false,
            last_name_first: false,
            phonetic_always: false,
            settings_dialog: SettingsDialog::None,
            settings_radio_choice: 0,
            emerg_info_count: 0,
            emerg_contact_count: 0,
            blocked_numbers: Vec::new(),
            status_toast: None,
        }
    }
}

const EDIT_FIELD_KEYS: &[&str] = &[
    "search_input",
    "dialog_label_input",
    "ed_prefix",
    "ed_first_name",
    "ed_middle_name",
    "ed_last_name",
    "ed_suffix",
    "ed_phone",
    "ed_email",
    "ed_org",
    "ed_title",
    "ed_address",
    "ed_notes",
];

fn vis(show: bool) -> Visibility {
    if show {
        Visibility::Visible
    } else {
        Visibility::Gone
    }
}

impl ContactsDemo {
    fn ensure_initialized(&mut self) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        self.state.theme_style = AndroidTheme::MaterialLight;
        home::install_drawables(&mut self.state);
        detail::install_drawables(&mut self.state);
        editor::install_drawables(&mut self.state);
        settings::install_drawables(&mut self.state);
        self.sync_all();
    }

    fn show_toast(&mut self, f: &mut Frame<'_>, msg: impl Into<String>, dur: f64) {
        let until = f.time() + dur;
        self.status_toast = Some((msg.into(), until));
        f.wake_at(until);
    }

    pub fn edit_text_for(&self, id: u64) -> &str {
        EDIT_FIELD_KEYS
            .iter()
            .copied()
            .find(|k| view_id(k) == id)
            .and_then(|key| self.state.text.get(key).map(String::as_str))
            .unwrap_or("")
    }

    pub fn set_edit_text(&mut self, id: u64, value: &str) {
        if let Some(key) = EDIT_FIELD_KEYS.iter().copied().find(|k| view_id(k) == id) {
            self.state.text.insert(key.into(), value.into());
            if key.starts_with("ed_") {
                self.editor_dirty = true;
            }
            self.sync_all();
        }
    }

    fn contact_by_id(&self, id: u32) -> Option<&Contact> {
        self.contacts.iter().find(|c| c.id == id)
    }

    fn contact_by_id_mut(&mut self, id: u32) -> Option<&mut Contact> {
        self.contacts.iter_mut().find(|c| c.id == id)
    }

    fn label_name(&self, id: u32) -> String {
        self.labels
            .iter()
            .find(|l| l.id == id)
            .map_or_else(|| "Label".into(), |l| l.name.clone())
    }

    fn sync_all(&mut self) {
        self.sync_home();
        self.sync_detail();
        self.sync_editor();
        self.sync_settings();
    }

    fn sync_home(&mut self) {
        let query = self
            .state
            .text
            .get("search_input")
            .map(|s| s.trim().to_ascii_lowercase())
            .unwrap_or_default();

        // Toolbars
        let in_label = self.active_label.is_some();
        let show_cab = self.cab_mode;
        let show_search = !show_cab && self.search_mode;
        let show_label_bar = !show_cab && !show_search && in_label;
        let show_home_bar = !show_cab && !show_search && !show_label_bar;

        self.state.visibility.insert("home_toolbar".into(), vis(show_home_bar));
        self.state.visibility.insert("search_toolbar".into(), vis(show_search));
        self.state.visibility.insert("cab_toolbar".into(), vis(show_cab));
        self.state.visibility.insert("label_toolbar".into(), vis(show_label_bar));
        self.state
            .visibility
            .insert("btn_search_clear".into(), vis(!query.is_empty()));

        if show_cab {
            self.state
                .text
                .insert("cab_count".into(), self.selected_ids.len().to_string());
            self.state
                .visibility
                .insert("btn_cab_link".into(), vis(self.selected_ids.len() >= 2));
        }

        if let Some(lid) = self.active_label {
            let lname = self.label_name(lid);
            self.state.text.insert("label_toolbar_title".into(), lname);
        }

        // Build filtered + sorted rows
        let mut filtered: Vec<&Contact> = self
            .contacts
            .iter()
            .filter(|c| {
                if let Some(lid) = self.active_label
                    && !c.labels.contains(&lid)
                {
                    return false;
                }
                if show_search && !query.is_empty() {
                    let hay = format!(
                        "{} {} {} {}",
                        c.display_name(false),
                        c.phone,
                        c.email,
                        c.org
                    )
                    .to_ascii_lowercase();
                    return hay.contains(&query);
                }
                true
            })
            .collect();
        filtered.sort_by(|a, b| {
            a.sort_key(self.sort_by_last)
                .cmp(&b.sort_key(self.sort_by_last))
                .then(a.id.cmp(&b.id))
        });

        let mut rows: Vec<DisplayedRow> = Vec::new();
        // Favorites section on main all-contacts view (not search, not label)
        if !show_search && !in_label {
            let mut first_fav = true;
            for c in &filtered {
                if c.starred && rows.len() < MAX_ROWS {
                    rows.push(DisplayedRow {
                        contact_id: c.id,
                        is_fav_section: true,
                        show_star_icon: first_fav,
                        letter: '\0',
                    });
                    first_fav = false;
                }
            }
        }
        let mut last_letter = '\0';
        for c in &filtered {
            if rows.len() >= MAX_ROWS {
                break;
            }
            let ch = c.initial(self.sort_by_last).chars().next().unwrap_or('#');
            let show_letter = if show_search {
                '\0'
            } else if ch != last_letter {
                last_letter = ch;
                ch
            } else {
                '\0'
            };
            rows.push(DisplayedRow {
                contact_id: c.id,
                is_fav_section: false,
                show_star_icon: false,
                letter: show_letter,
            });
        }
        self.displayed_rows = rows;

        // Empty vs list state
        let is_empty = self.displayed_rows.is_empty();
        self.state
            .visibility
            .insert("empty_state".into(), vis(is_empty && !show_search && !in_label));
        self.state
            .visibility
            .insert("search_empty_state".into(), vis(is_empty && show_search));
        self.state
            .visibility
            .insert("label_empty_state".into(), vis(is_empty && !show_search && in_label));
        self.state
            .visibility
            .insert("contacts_scroll".into(), vis(!is_empty));

        // Label summary row
        let show_label_count = in_label && !show_search && !is_empty;
        self.state
            .visibility
            .insert("label_count_row".into(), vis(show_label_count));
        if show_label_count {
            let n = filtered.len();
            let noun = if n == 1 { "contact" } else { "contacts" };
            self.state
                .text
                .insert("label_count_text".into(), format!("{n} {noun} · DEVICE"));
        }

        // FAB visibility (hidden in CAB, search, or label screen)
        self.state
            .visibility
            .insert("fab_add".into(), vis(!show_cab && !show_search && !in_label));

        // Bind row slots 0..MAX_ROWS
        for i in 0..MAX_ROWS {
            let row_id = format!("contact_row_{i}");
            if let Some(row) = self.displayed_rows.get(i).copied()
                && let Some(c) = self.contact_by_id(row.contact_id).cloned()
            {
                self.state.visibility.insert(row_id, Visibility::Visible);
                self.state
                    .visibility
                    .insert(format!("row_star_{i}"), vis(row.show_star_icon));
                let has_letter = !row.is_fav_section && row.letter != '\0';
                self.state
                    .visibility
                    .insert(format!("row_letter_{i}"), vis(has_letter));
                self.state.text.insert(
                    format!("row_letter_{i}"),
                    if has_letter { row.letter.to_string() } else { String::new() },
                );
                self.state.image_sources.insert(
                    format!("row_avatar_bg_{i}"),
                    format!("avatar_circle_{}", c.color_idx % 6),
                );
                self.state
                    .text
                    .insert(format!("row_avatar_letter_{i}"), c.initial(false));
                self.state.text.insert(
                    format!("row_name_{i}"),
                    c.display_name(self.last_name_first),
                );
                self.state
                    .visibility
                    .insert(format!("row_check_{i}"), vis(show_cab));
                self.state
                    .checked
                    .insert(format!("row_check_{i}"), self.selected_ids.contains(&c.id));
            } else {
                self.state.visibility.insert(row_id, Visibility::Gone);
            }
        }

        // Drawer sync
        let profile_sub = if self.my_info_name.trim().is_empty() {
            "Set up your profile".to_owned()
        } else {
            self.my_info_name.clone()
        };
        self.state
            .text
            .insert("drawer_my_info_sub".into(), profile_sub);
        let contacts_active = self.active_label.is_none();
        self.state.image_sources.insert(
            "drawer_contacts_bg".into(),
            if contacts_active {
                "drawer_row_active".into()
            } else {
                "drawer_row_inactive".into()
            },
        );
        self.state.image_sources.insert(
            "drawer_contacts_icon".into(),
            if contacts_active {
                "ic_contacts_blue".into()
            } else {
                "ic_contacts_muted".into()
            },
        );
        for i in 0..MAX_DRAWER_LABELS {
            let row_key = format!("drawer_label_{i}");
            if let Some(lbl) = self.labels.get(i) {
                let active = self.active_label == Some(lbl.id);
                self.state.visibility.insert(row_key, Visibility::Visible);
                self.state
                    .text
                    .insert(format!("drawer_label_text_{i}"), lbl.name.clone());
                self.state.image_sources.insert(
                    format!("drawer_label_bg_{i}"),
                    if active {
                        "drawer_row_active".into()
                    } else {
                        "drawer_row_inactive".into()
                    },
                );
                self.state.image_sources.insert(
                    format!("drawer_label_icon_{i}"),
                    if active {
                        "ic_label_blue".into()
                    } else {
                        "ic_label_muted".into()
                    },
                );
            } else {
                self.state.visibility.insert(row_key, Visibility::Gone);
            }
        }

        // Label overflow popup & Home modal dialog visibility is animated in `step_overlay_anims`.
        match self.home_dialog {
            HomeDialog::None => {}
            HomeDialog::CreateLabel => {
                self.state
                    .text
                    .insert("home_dialog_title".into(), "Create label".into());
                self.state
                    .visibility
                    .insert("home_dialog_body".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("home_dialog_input_box".into(), Visibility::Visible);
                self.state
                    .visibility
                    .insert("home_dialog_picker_list".into(), Visibility::Gone);
                self.state.text.insert("home_dialog_cancel".into(), "CANCEL".into());
                self.state.text.insert("home_dialog_ok".into(), "OK".into());
            }
            HomeDialog::RenameLabel(_) => {
                self.state
                    .text
                    .insert("home_dialog_title".into(), "Rename label".into());
                self.state
                    .visibility
                    .insert("home_dialog_body".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("home_dialog_input_box".into(), Visibility::Visible);
                self.state
                    .visibility
                    .insert("home_dialog_picker_list".into(), Visibility::Gone);
                self.state.text.insert("home_dialog_cancel".into(), "CANCEL".into());
                self.state.text.insert("home_dialog_ok".into(), "OK".into());
            }
            HomeDialog::DeleteLabel(lid) => {
                let lname = self.label_name(lid);
                self.state
                    .text
                    .insert("home_dialog_title".into(), format!("Delete the label \"{lname}\"?"));
                self.state.text.insert(
                    "home_dialog_body".into(),
                    "(Contacts themselves will not be deleted.)".into(),
                );
                self.state
                    .visibility
                    .insert("home_dialog_body".into(), Visibility::Visible);
                self.state
                    .visibility
                    .insert("home_dialog_input_box".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("home_dialog_picker_list".into(), Visibility::Gone);
                self.state.text.insert("home_dialog_cancel".into(), "CANCEL".into());
                self.state.text.insert("home_dialog_ok".into(), "OK".into());
            }
            HomeDialog::DeleteSelected => {
                let n = self.selected_ids.len();
                let title = if n == 1 {
                    "Delete this contact?".into()
                } else {
                    format!("Delete {n} selected contacts?")
                };
                self.state.text.insert("home_dialog_title".into(), title);
                self.state
                    .visibility
                    .insert("home_dialog_body".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("home_dialog_input_box".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("home_dialog_picker_list".into(), Visibility::Gone);
                self.state.text.insert("home_dialog_cancel".into(), "CANCEL".into());
                self.state.text.insert("home_dialog_ok".into(), "DELETE".into());
            }
            HomeDialog::AddToLabel(_) | HomeDialog::RemoveFromLabel(_) => {
                let is_add = matches!(self.home_dialog, HomeDialog::AddToLabel(_));
                self.state.text.insert(
                    "home_dialog_title".into(),
                    if is_add { "Add contacts".into() } else { "Remove contacts".into() },
                );
                self.state
                    .visibility
                    .insert("home_dialog_body".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("home_dialog_input_box".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("home_dialog_picker_list".into(), Visibility::Visible);
                for i in 0..MAX_PICKER_ROWS {
                    if let Some(&cid) = self.picker_contact_ids.get(i)
                        && let Some(name) = self
                            .contact_by_id(cid)
                            .map(|c| c.display_name(self.last_name_first))
                    {
                        self.state
                            .visibility
                            .insert(format!("picker_row_{i}"), Visibility::Visible);
                        self.state
                            .text
                            .insert(format!("picker_name_{i}"), name);
                        self.state
                            .checked
                            .insert(format!("picker_check_{i}"), self.picker_checked[i]);
                    } else {
                        self.state
                            .visibility
                            .insert(format!("picker_row_{i}"), Visibility::Gone);
                    }
                }
                self.state.text.insert("home_dialog_cancel".into(), "CANCEL".into());
                self.state.text.insert(
                    "home_dialog_ok".into(),
                    if is_add { "ADD".into() } else { "REMOVE".into() },
                );
            }
        }
    }

    fn sync_detail(&mut self) {
        let Screen::Detail(cid) = self.screen else {
            return;
        };
        let Some(c) = self.contact_by_id(cid).cloned() else {
            self.screen = Screen::Home;
            return;
        };
        self.state
            .image_sources
            .insert("detail_hero_bg".into(), format!("hero_bg_{}", c.color_idx % 6));
        self.state.image_sources.insert(
            "detail_star".into(),
            if c.starred {
                "ic_star_filled_white".into()
            } else {
                "ic_star_outline_white".into()
            },
        );
        self.state
            .text
            .insert("detail_name".into(), c.display_name(self.last_name_first));

        let has_phone = !c.phone.trim().is_empty();
        let has_email = !c.email.trim().is_empty();
        self.state
            .visibility
            .insert("detail_comm_card".into(), vis(has_phone || has_email));
        self.state
            .visibility
            .insert("detail_phone_row".into(), vis(has_phone));
        self.state.text.insert("detail_phone".into(), c.phone.clone());
        self.state.text.insert(
            "detail_phone_type".into(),
            PHONE_TYPES[c.phone_type % PHONE_TYPES.len()].into(),
        );
        self.state
            .visibility
            .insert("detail_comm_divider".into(), vis(has_phone && has_email));
        self.state
            .visibility
            .insert("detail_email_row".into(), vis(has_email));
        self.state.text.insert("detail_email".into(), c.email.clone());
        self.state.text.insert(
            "detail_email_type".into(),
            EMAIL_TYPES[c.email_type % EMAIL_TYPES.len()].into(),
        );

        let has_org = !c.org.trim().is_empty() || !c.title.trim().is_empty();
        let has_notes = !c.notes.trim().is_empty();
        let label_names: Vec<String> = c
            .labels
            .iter()
            .filter_map(|lid| self.labels.iter().find(|l| l.id == *lid).map(|l| l.name.clone()))
            .collect();
        let has_labels = !label_names.is_empty();
        let has_ringtone = c.ringtone > 0;
        let show_about = has_org || has_notes || has_labels || has_ringtone;
        self.state
            .visibility
            .insert("detail_about_card".into(), vis(show_about));
        self.state.text.insert(
            "detail_about_title".into(),
            format!("About {}", c.first_or_display()),
        );
        self.state
            .visibility
            .insert("detail_org_row".into(), vis(has_org));
        let org_text = match (c.org.trim().is_empty(), c.title.trim().is_empty()) {
            (false, false) => format!("{} · {}", c.org.trim(), c.title.trim()),
            (false, true) => c.org.trim().to_owned(),
            (true, false) => c.title.trim().to_owned(),
            (true, true) => String::new(),
        };
        self.state.text.insert("detail_org".into(), org_text);
        self.state
            .visibility
            .insert("detail_notes_row".into(), vis(has_notes));
        self.state.text.insert("detail_notes".into(), c.notes.clone());
        self.state
            .visibility
            .insert("detail_labels_row".into(), vis(has_labels));
        self.state
            .text
            .insert("detail_labels".into(), label_names.join(", "));
        self.state
            .visibility
            .insert("detail_ringtone_row".into(), vis(has_ringtone));
        self.state.text.insert(
            "detail_ringtone".into(),
            RINGTONES[c.ringtone % RINGTONES.len()].into(),
        );

        // Detail overflow popup & modal dialog visibility is animated in `step_overlay_anims`.
        match self.detail_dialog {
            DetailDialog::None => {}
            DetailDialog::Delete => {
                self.state
                    .text
                    .insert("detail_dialog_title".into(), "Delete this contact?".into());
                self.state
                    .visibility
                    .insert("detail_dialog_body".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("detail_dialog_options".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("detail_dialog_confirm".into(), Visibility::Visible);
                self.state.text.insert("detail_dialog_cancel".into(), "CANCEL".into());
                self.state.text.insert("detail_dialog_confirm".into(), "DELETE".into());
            }
            DetailDialog::Ringtone => {
                self.state
                    .text
                    .insert("detail_dialog_title".into(), "Ringtones".into());
                self.state
                    .visibility
                    .insert("detail_dialog_body".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("detail_dialog_options".into(), Visibility::Visible);
                for (i, name) in RINGTONES.iter().enumerate() {
                    self.state
                        .visibility
                        .insert(format!("detail_opt_{i}"), Visibility::Visible);
                    let mark = if c.ringtone == i { "●  " } else { "○  " };
                    self.state
                        .text
                        .insert(format!("detail_opt_{i}"), format!("{mark}{name}"));
                }
                self.state.text.insert("detail_dialog_cancel".into(), "CANCEL".into());
                self.state
                    .visibility
                    .insert("detail_dialog_confirm".into(), Visibility::Gone);
            }
            DetailDialog::Link => {
                self.state
                    .text
                    .insert("detail_dialog_title".into(), "Link contacts".into());
                self.state.text.insert(
                    "detail_dialog_body".into(),
                    format!("Choose the contact you want to link with {}:", c.display_name(false)),
                );
                self.state
                    .visibility
                    .insert("detail_dialog_body".into(), Visibility::Visible);
                self.state
                    .visibility
                    .insert("detail_dialog_options".into(), Visibility::Visible);
                let others: Vec<(u32, String)> = self
                    .contacts
                    .iter()
                    .filter(|o| o.id != cid)
                    .take(5)
                    .map(|o| (o.id, o.display_name(self.last_name_first)))
                    .collect();
                self.detail_link_ids = others.iter().map(|(id, _)| *id).collect();
                for i in 0..5 {
                    if let Some((_, name)) = others.get(i) {
                        self.state
                            .visibility
                            .insert(format!("detail_opt_{i}"), Visibility::Visible);
                        self.state.text.insert(format!("detail_opt_{i}"), name.clone());
                    } else {
                        self.state
                            .visibility
                            .insert(format!("detail_opt_{i}"), Visibility::Gone);
                    }
                }
                self.state.text.insert("detail_dialog_cancel".into(), "CANCEL".into());
                self.state
                    .visibility
                    .insert("detail_dialog_confirm".into(), Visibility::Gone);
            }
            DetailDialog::Share => {
                self.state.text.insert(
                    "detail_dialog_title".into(),
                    format!("Share {}.vcf", c.display_name(false)),
                );
                self.state.text.insert(
                    "detail_dialog_body".into(),
                    format!(
                        "BEGIN:VCARD\nFN:{}\nTEL:{}\nEMAIL:{}\nEND:VCARD",
                        c.display_name(false),
                        c.phone,
                        c.email
                    ),
                );
                self.state
                    .visibility
                    .insert("detail_dialog_body".into(), Visibility::Visible);
                self.state
                    .visibility
                    .insert("detail_dialog_options".into(), Visibility::Gone);
                self.state.text.insert("detail_dialog_cancel".into(), "CLOSE".into());
                self.state.text.insert("detail_dialog_confirm".into(), "SHARE".into());
                self.state
                    .visibility
                    .insert("detail_dialog_confirm".into(), Visibility::Visible);
            }
        }
    }

    fn open_editor(&mut self, f: &mut Frame<'_>, target: EditorTarget) {
        self.editor_target = target;
        self.editor_name_expanded = false;
        self.editor_more_fields = false;
        self.editor_dirty = false;
        self.editor_dialog = EditorDialog::None;
        self.state.offsets.insert("editor_scroll".into(), 0.0);
        self.state
            .drawer_open
            .insert("drawer_layout".into(), false);

        match target {
            EditorTarget::NewContact => {
                for key in [
                    "ed_prefix",
                    "ed_first_name",
                    "ed_middle_name",
                    "ed_last_name",
                    "ed_suffix",
                    "ed_phone",
                    "ed_email",
                    "ed_org",
                    "ed_title",
                    "ed_address",
                    "ed_notes",
                ] {
                    self.state.text.insert(key.into(), String::new());
                }
                self.editor_phone_type = 0;
                self.editor_email_type = 0;
                self.editor_color_idx = (self.next_contact_id as usize) % 5;
            }
            EditorTarget::EditContact(cid) => {
                if let Some(c) = self.contact_by_id(cid).cloned() {
                    self.state.text.insert("ed_prefix".into(), c.prefix.clone());
                    self.state
                        .text
                        .insert("ed_first_name".into(), c.first_name.clone());
                    self.state
                        .text
                        .insert("ed_middle_name".into(), c.middle_name.clone());
                    self.state
                        .text
                        .insert("ed_last_name".into(), c.last_name.clone());
                    self.state.text.insert("ed_suffix".into(), c.suffix.clone());
                    self.state.text.insert("ed_phone".into(), c.phone.clone());
                    self.state.text.insert("ed_email".into(), c.email.clone());
                    self.state.text.insert("ed_org".into(), c.org.clone());
                    self.state.text.insert("ed_title".into(), c.title.clone());
                    self.state
                        .text
                        .insert("ed_address".into(), c.address.clone());
                    self.state.text.insert("ed_notes".into(), c.notes.clone());
                    self.editor_phone_type = c.phone_type;
                    self.editor_email_type = c.email_type;
                    self.editor_color_idx = c.color_idx;
                    self.editor_name_expanded = !c.prefix.is_empty()
                        || !c.middle_name.is_empty()
                        || !c.suffix.is_empty();
                    self.editor_more_fields = !c.org.is_empty()
                        || !c.title.is_empty()
                        || !c.address.is_empty()
                        || !c.notes.is_empty();
                }
            }
            EditorTarget::MyInfo => {
                let mut parts = self.my_info_name.splitn(2, ' ');
                let first = parts.next().unwrap_or("").to_owned();
                let last = parts.next().unwrap_or("").to_owned();
                for key in [
                    "ed_prefix",
                    "ed_middle_name",
                    "ed_suffix",
                    "ed_org",
                    "ed_title",
                    "ed_address",
                    "ed_notes",
                ] {
                    self.state.text.insert(key.into(), String::new());
                }
                self.state.text.insert("ed_first_name".into(), first);
                self.state.text.insert("ed_last_name".into(), last);
                self.state
                    .text
                    .insert("ed_phone".into(), self.my_info_phone.clone());
                self.state
                    .text
                    .insert("ed_email".into(), self.my_info_email.clone());
                self.editor_phone_type = 0;
                self.editor_email_type = 0;
                self.editor_color_idx = 5;
            }
        }
        self.navigate(f, Screen::Editor, TransitionKind::ModalUp);
        self.sync_editor();
    }

    fn sync_editor(&mut self) {
        let (title, saving_to) = match self.editor_target {
            EditorTarget::NewContact => ("Create new contact", "Device"),
            EditorTarget::EditContact(_) => ("Edit contact", "Device"),
            EditorTarget::MyInfo => ("My info", "My local profile"),
        };
        self.state.text.insert("editor_title".into(), title.into());
        self.state
            .text
            .insert("editor_saving_to".into(), saving_to.into());
        self.state
            .visibility
            .insert("editor_prefix_wrap".into(), vis(self.editor_name_expanded));
        self.state
            .visibility
            .insert("editor_middle_wrap".into(), vis(self.editor_name_expanded));
        self.state
            .visibility
            .insert("editor_suffix_wrap".into(), vis(self.editor_name_expanded));
        self.state.image_sources.insert(
            "editor_expand_name".into(),
            if self.editor_name_expanded {
                "ic_expand_less_muted".into()
            } else {
                "ic_expand_more_muted".into()
            },
        );
        self.state.text.insert(
            "editor_phone_type_text".into(),
            PHONE_TYPES[self.editor_phone_type % PHONE_TYPES.len()].into(),
        );
        self.state.text.insert(
            "editor_email_type_text".into(),
            EMAIL_TYPES[self.editor_email_type % EMAIL_TYPES.len()].into(),
        );
        self.state
            .visibility
            .insert("editor_extra_fields".into(), vis(self.editor_more_fields));
        self.state
            .visibility
            .insert("editor_more_fields_btn".into(), vis(!self.editor_more_fields));

        // Editor modal dialog visibility is animated in `step_overlay_anims`.
        match self.editor_dialog {
            EditorDialog::None => {}
            EditorDialog::ChangePhoto => {
                self.state
                    .text
                    .insert("editor_dialog_title".into(), "Change photo".into());
                self.state
                    .visibility
                    .insert("editor_dialog_options".into(), Visibility::Visible);
                let opts = ["Take photo", "Choose photo", "Cycle avatar color"];
                for i in 0..9 {
                    if let Some(lbl) = opts.get(i) {
                        self.state
                            .visibility
                            .insert(format!("ed_opt_{i}"), Visibility::Visible);
                        self.state.text.insert(format!("ed_opt_{i}"), (*lbl).into());
                    } else {
                        self.state
                            .visibility
                            .insert(format!("ed_opt_{i}"), Visibility::Gone);
                    }
                }
                self.state
                    .visibility
                    .insert("editor_dialog_actions".into(), Visibility::Visible);
                self.state
                    .text
                    .insert("editor_dialog_cancel".into(), "CANCEL".into());
                self.state
                    .visibility
                    .insert("editor_dialog_confirm".into(), Visibility::Gone);
            }
            EditorDialog::Discard => {
                self.state
                    .text
                    .insert("editor_dialog_title".into(), "Discard changes?".into());
                self.state
                    .visibility
                    .insert("editor_dialog_options".into(), Visibility::Gone);
                self.state
                    .visibility
                    .insert("editor_dialog_actions".into(), Visibility::Visible);
                self.state
                    .text
                    .insert("editor_dialog_cancel".into(), "CANCEL".into());
                self.state
                    .text
                    .insert("editor_dialog_confirm".into(), "DISCARD".into());
                self.state
                    .visibility
                    .insert("editor_dialog_confirm".into(), Visibility::Visible);
            }
            EditorDialog::PhoneType => {
                self.state
                    .text
                    .insert("editor_dialog_title".into(), "Phone type".into());
                self.state
                    .visibility
                    .insert("editor_dialog_options".into(), Visibility::Visible);
                for i in 0..9 {
                    self.state
                        .visibility
                        .insert(format!("ed_opt_{i}"), Visibility::Visible);
                    let mark = if self.editor_phone_type == i { "●  " } else { "○  " };
                    self.state
                        .text
                        .insert(format!("ed_opt_{i}"), format!("{mark}{}", PHONE_TYPES[i]));
                }
                self.state
                    .visibility
                    .insert("editor_dialog_actions".into(), Visibility::Gone);
            }
            EditorDialog::EmailType => {
                self.state
                    .text
                    .insert("editor_dialog_title".into(), "Email type".into());
                self.state
                    .visibility
                    .insert("editor_dialog_options".into(), Visibility::Visible);
                for i in 0..9 {
                    if let Some(lbl) = EMAIL_TYPES.get(i) {
                        self.state
                            .visibility
                            .insert(format!("ed_opt_{i}"), Visibility::Visible);
                        let mark = if self.editor_email_type == i { "●  " } else { "○  " };
                        self.state
                            .text
                            .insert(format!("ed_opt_{i}"), format!("{mark}{lbl}"));
                    } else {
                        self.state
                            .visibility
                            .insert(format!("ed_opt_{i}"), Visibility::Gone);
                    }
                }
                self.state
                    .visibility
                    .insert("editor_dialog_actions".into(), Visibility::Gone);
            }
        }
    }

    fn sync_settings(&mut self) {
        let sub = match self.screen {
            Screen::Settings(s) => s,
            _ => SettingsSub::Main,
        };
        self.sync_settings_for(sub);
    }

    fn sync_settings_for(&mut self, sub: SettingsSub) {
        let title = match sub {
            SettingsSub::Main => "Settings",
            SettingsSub::About => "About Contacts",
            SettingsSub::Emergency => "Emergency information",
            SettingsSub::Blocked => "Blocked numbers",
        };
        self.state.text.insert("settings_title".into(), title.into());
        self.state
            .visibility
            .insert("settings_main_list".into(), vis(sub == SettingsSub::Main));
        self.state
            .visibility
            .insert("settings_about_view".into(), vis(sub == SettingsSub::About));
        self.state
            .visibility
            .insert("settings_emergency_view".into(), vis(sub == SettingsSub::Emergency));
        self.state
            .visibility
            .insert("settings_blocked_view".into(), vis(sub == SettingsSub::Blocked));

        let profile_sub = if self.my_info_name.trim().is_empty() {
            "Set up your profile".to_owned()
        } else {
            self.my_info_name.clone()
        };
        self.state
            .text
            .insert("pref_my_info_sub".into(), profile_sub);
        self.state.text.insert(
            "pref_default_account_sub".into(),
            if self.default_account == 0 {
                "Device".into()
            } else {
                "SIM card".into()
            },
        );
        self.state.text.insert(
            "pref_display_sub".into(),
            if self.display_mode == 0 {
                "All contacts".into()
            } else {
                "Customize".into()
            },
        );
        self.state.text.insert(
            "pref_sort_sub".into(),
            if self.sort_by_last {
                "Last name".into()
            } else {
                "First name".into()
            },
        );
        self.state.text.insert(
            "pref_name_format_sub".into(),
            if self.last_name_first {
                "Last name first".into()
            } else {
                "First name first".into()
            },
        );
        self.state.text.insert(
            "pref_phonetic_sub".into(),
            if self.phonetic_always {
                "Always show".into()
            } else {
                "Hide if empty".into()
            },
        );
        self.state.text.insert(
            "emerg_owner_name".into(),
            if self.my_info_name.trim().is_empty() {
                "Owner".into()
            } else {
                self.my_info_name.clone()
            },
        );
        self.state.text.insert(
            "emerg_info_summary".into(),
            if self.emerg_info_count == 0 {
                "Add information".into()
            } else {
                format!("Medical notes saved ({})", self.emerg_info_count)
            },
        );
        self.state.text.insert(
            "emerg_contacts_summary".into(),
            if self.emerg_contact_count == 0 {
                "Add contact".into()
            } else {
                format!("{} emergency contact(s)", self.emerg_contact_count)
            },
        );
        self.state.text.insert(
            "blocked_list_text".into(),
            if self.blocked_numbers.is_empty() {
                "No blocked numbers".into()
            } else {
                self.blocked_numbers.join("\n")
            },
        );

        let dlg_open = self.settings_dialog != SettingsDialog::None;
        let (dtitle, opts): (&str, &[&str]) = match self.settings_dialog {
            SettingsDialog::None => ("", &[]),
            SettingsDialog::DefaultAccount => (
                "Default account for new contacts",
                &["Device", "SIM card"],
            ),
            SettingsDialog::ContactsToDisplay => (
                "Contacts to display",
                &["All contacts", "Customize"],
            ),
            SettingsDialog::SortBy => ("Sort by", &["First name", "Last name"]),
            SettingsDialog::NameFormat => (
                "Name format",
                &["First name first", "Last name first"],
            ),
            SettingsDialog::PhoneticName => (
                "Phonetic name",
                &["Always show", "Hide if empty"],
            ),
            SettingsDialog::Import => ("Import contacts from", &[".vcf file", "SIM card"]),
            SettingsDialog::Export => (
                "Export contacts",
                &["Export to .vcf file", "Share all contacts"],
            ),
        };
        if dlg_open {
            self.state
                .text
                .insert("settings_dialog_title".into(), dtitle.into());
            for i in 0..3 {
                let id = format!("set_opt_{i}");
                if let Some(lbl) = opts.get(i) {
                    self.state.visibility.insert(id.clone(), Visibility::Visible);
                    self.state.text.insert(id.clone(), (*lbl).into());
                    self.state
                        .checked
                        .insert(id, self.settings_radio_choice == i);
                } else {
                    self.state.visibility.insert(id, Visibility::Gone);
                }
            }
        }
    }

    fn navigate(&mut self, f: &mut Frame<'_>, target: Screen, kind: TransitionKind) {
        if target == self.screen {
            return;
        }
        let from = self.screen;
        self.screen = target;
        if f.reduced_motion() || f.time() == 0.0 {
            self.transition = None;
        } else {
            let duration = match kind {
                TransitionKind::Push => 0.28,
                TransitionKind::Pop => 0.26,
                TransitionKind::ModalUp => 0.26,
                TransitionKind::ModalDown => 0.20,
            };
            self.transition = Some(ScreenTransition {
                from,
                to: target,
                kind,
                start: f.time(),
                duration,
            });
            f.continuous(Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            });
        }
    }

    fn render_screen(
        &mut self,
        screen: Screen,
        f: &mut Frame<'_>,
        bounds: Rect,
    ) -> Option<&'static str> {
        match screen {
            Screen::Home => home::LAYOUT.render(f, bounds, &mut self.state),
            Screen::Detail(_) => detail::LAYOUT.render(f, bounds, &mut self.state),
            Screen::Editor => editor::LAYOUT.render(f, bounds, &mut self.state),
            Screen::Settings(sub) => {
                self.sync_settings_for(sub);
                settings::LAYOUT.render(f, bounds, &mut self.state)
            }
        }
    }

    fn step_overlay_anims(&mut self, now: f64, reduced: bool) -> bool {
        let mut animating = false;
        animating |= apply_dialog_anim(
            &mut self.state,
            &mut self.home_dialog_anim,
            self.home_dialog != HomeDialog::None,
            "home_dialog_scrim",
            "home_dialog_card",
            now,
            reduced,
        );
        animating |= apply_dialog_anim(
            &mut self.state,
            &mut self.detail_dialog_anim,
            self.detail_dialog != DetailDialog::None,
            "detail_dialog_scrim",
            "detail_dialog_card",
            now,
            reduced,
        );
        animating |= apply_dialog_anim(
            &mut self.state,
            &mut self.editor_dialog_anim,
            self.editor_dialog != EditorDialog::None,
            "editor_dialog_scrim",
            "editor_dialog_card",
            now,
            reduced,
        );
        animating |= apply_dialog_anim(
            &mut self.state,
            &mut self.settings_dialog_anim,
            self.settings_dialog != SettingsDialog::None,
            "settings_dialog_scrim",
            "settings_dialog_card",
            now,
            reduced,
        );
        animating |= apply_popup_anim(
            &mut self.state,
            &mut self.label_popup_anim,
            self.label_overflow_open,
            "label_overflow_popup",
            now,
            reduced,
        );
        animating |= apply_popup_anim(
            &mut self.state,
            &mut self.detail_popup_anim,
            self.detail_overflow_open,
            "detail_overflow_popup",
            now,
            reduced,
        );
        animating
    }

    pub fn open_new_contact(&mut self, f: &mut Frame<'_>) {
        self.ensure_initialized();
        self.open_editor(f, EditorTarget::NewContact);
        self.sync_all();
    }

    pub fn show(&mut self, f: &mut Frame<'_>, width: f32, height: f32) {
        self.show_in(
            f,
            Rect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
        );
    }

    pub fn show_in(&mut self, f: &mut Frame<'_>, bounds: Rect) {
        self.ensure_initialized();
        let width = bounds.width;
        let height = bounds.height;
        let now = f.time();
        let reduced = f.reduced_motion();
        if self.step_overlay_anims(now, reduced) {
            f.continuous(bounds);
        }

        let mut action = None;
        f.within(bounds, |f| {
            action = if let Some(tr) = self.transition {
                let raw = if reduced || tr.duration <= 0.0 {
                    1.0
                } else {
                    ((now - tr.start) / tr.duration).clamp(0.0, 1.0) as f32
                };
                if raw >= 1.0 {
                    self.transition = None;
                    self.render_screen(self.screen, f, bounds)
                } else {
                    f.continuous(bounds);
                    let mut act = None;
                    match tr.kind {
                        TransitionKind::Push => {
                            let e = emphasized_decelerate(raw);
                            let back_dx = -0.22 * width * e;
                            f.push_transform([bounds.x, bounds.y], [back_dx, 0.0], [1.0, 1.0], 0.0);
                            f.without_input(true, |f| {
                                self.render_screen(tr.from, f, bounds);
                            });
                            f.pop_transform();
                            let scrim_a = (42.0 * e).round().clamp(0.0, 255.0) as u8;
                            if scrim_a > 0 {
                                f.paint_rect(bounds, 0.0, Color(0, 0, 0, scrim_a));
                            }
                            let front_dx = width * (1.0 - e);
                            paint_page_edge_shadow(f, bounds.x + front_dx, bounds.y, height, 1.0 - e * 0.6);
                            f.push_transform([bounds.x, bounds.y], [front_dx, 0.0], [1.0, 1.0], 0.0);
                            f.without_input(raw < 0.75, |f| {
                                act = self.render_screen(tr.to, f, bounds);
                            });
                            f.pop_transform();
                        }
                        TransitionKind::Pop => {
                            let e = emphasized_decelerate(raw);
                            let back_dx = -0.22 * width * (1.0 - e);
                            f.push_transform([bounds.x, bounds.y], [back_dx, 0.0], [1.0, 1.0], 0.0);
                            f.without_input(raw < 0.75, |f| {
                                act = self.render_screen(tr.to, f, bounds);
                            });
                            f.pop_transform();
                            let scrim_a = (42.0 * (1.0 - e)).round().clamp(0.0, 255.0) as u8;
                            if scrim_a > 0 {
                                f.paint_rect(bounds, 0.0, Color(0, 0, 0, scrim_a));
                            }
                            let front_dx = width * e;
                            paint_page_edge_shadow(f, bounds.x + front_dx, bounds.y, height, 1.0 - e * 0.7);
                            f.push_transform([bounds.x, bounds.y], [front_dx, 0.0], [1.0, 1.0], 0.0);
                            f.without_input(true, |f| {
                                self.render_screen(tr.from, f, bounds);
                            });
                            f.pop_transform();
                        }
                        TransitionKind::ModalUp => {
                            let e = emphasized_decelerate(raw);
                            f.without_input(true, |f| {
                                self.render_screen(tr.from, f, bounds);
                            });
                            let scrim_a = (36.0 * e).round().clamp(0.0, 255.0) as u8;
                            if scrim_a > 0 {
                                f.paint_rect(bounds, 0.0, Color(0, 0, 0, scrim_a));
                            }
                            let dy = height * 0.14 * (1.0 - e);
                            let alpha = (e * 1.45).min(1.0);
                            f.push_transform([bounds.x, bounds.y], [0.0, dy], [1.0, 1.0], 0.0);
                            f.push_layer(bounds, alpha);
                            f.without_input(raw < 0.75, |f| {
                                act = self.render_screen(tr.to, f, bounds);
                            });
                            f.pop_layer();
                            f.pop_transform();
                        }
                        TransitionKind::ModalDown => {
                            let e = emphasized_accelerate(raw);
                            f.without_input(raw < 0.75, |f| {
                                act = self.render_screen(tr.to, f, bounds);
                            });
                            let scrim_a = (36.0 * (1.0 - e)).round().clamp(0.0, 255.0) as u8;
                            if scrim_a > 0 {
                                f.paint_rect(bounds, 0.0, Color(0, 0, 0, scrim_a));
                            }
                            let dy = height * 0.14 * e;
                            let alpha = (1.0 - e).clamp(0.0, 1.0);
                            f.push_transform([bounds.x, bounds.y], [0.0, dy], [1.0, 1.0], 0.0);
                            f.push_layer(bounds, alpha);
                            f.without_input(true, |f| {
                                self.render_screen(tr.from, f, bounds);
                            });
                            f.pop_layer();
                            f.pop_transform();
                        }
                    }
                    act
                }
            } else {
                self.render_screen(self.screen, f, bounds)
            };
        });
        if let Some(action) = action {
            self.handle_action(f, action);
            self.sync_all();
            if self.step_overlay_anims(f.time(), reduced) {
                f.continuous(bounds);
            }
            f.wake_at(f.time() + 0.001);
        }

        // Crisp pill toast matching `14-contact-detail-alice.png`
        if let Some((msg, until)) = &self.status_toast {
            if f.time() < *until {
                let style = TextStyle {
                    size: 13.5,
                    weight: 500,
                    align: 1,
                    ..TextStyle::default()
                };
                let tw = (f.text_width(msg, style) + 36.0).clamp(140.0, (width - 32.0).max(140.0));
                let th = 40.0;
                let toast_rect = Rect {
                    x: bounds.x + (width - tw) * 0.5,
                    y: bounds.y + (height - 84.0).max(16.0),
                    width: tw,
                    height: th,
                };
                f.elevation(toast_rect, 20.0, 3);
                f.paint_rect(toast_rect, 20.0, Color(244, 244, 248, 255));
                f.styled_text(toast_rect, msg, Color(31, 31, 31, 255), style);
                f.wake_at(*until);
            } else {
                self.status_toast = None;
            }
        }
    }

    fn handle_action(&mut self, f: &mut Frame<'_>, action: &str) {
        match self.screen {
            Screen::Home => self.handle_home_action(f, action),
            Screen::Detail(cid) => self.handle_detail_action(f, cid, action),
            Screen::Editor => self.handle_editor_action(f, action),
            Screen::Settings(sub) => self.handle_settings_action(f, sub, action),
        }
    }

    fn handle_home_action(&mut self, f: &mut Frame<'_>, action: &str) {
        match action {
            "btn_menu" | "btn_label_menu" => {
                self.label_overflow_open = false;
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
            "btn_search" => {
                self.search_mode = true;
                self.cab_mode = false;
                self.label_overflow_open = false;
                self.state.text.insert("search_input".into(), String::new());
            }
            "btn_search_back" => {
                self.search_mode = false;
                self.state.text.insert("search_input".into(), String::new());
            }
            "btn_search_clear" => {
                self.state.text.insert("search_input".into(), String::new());
            }
            "btn_select_mode" => {
                if self.contacts.is_empty() {
                    self.show_toast(f, "No contacts to select", 1.6);
                } else {
                    self.cab_mode = true;
                    self.selected_ids.clear();
                    if let Some(first) = self.displayed_rows.first() {
                        self.selected_ids.push(first.contact_id);
                    }
                }
            }
            "btn_cab_close" => {
                self.cab_mode = false;
                self.selected_ids.clear();
            }
            "btn_cab_share" => {
                let n = self.selected_ids.len();
                self.show_toast(f, format!("Exported {n} contact(s) as .vcf"), 2.0);
                self.cab_mode = false;
                self.selected_ids.clear();
            }
            "btn_cab_delete" => {
                if !self.selected_ids.is_empty() {
                    self.home_dialog = HomeDialog::DeleteSelected;
                }
            }
            "btn_cab_link" => {
                let n = self.selected_ids.len();
                self.show_toast(f, format!("Linked {n} contacts"), 2.0);
                self.cab_mode = false;
                self.selected_ids.clear();
            }
            "fab_add" | "btn_empty_add" => {
                self.open_editor(f, EditorTarget::NewContact);
            }
            "btn_label_add_member" | "btn_label_empty_add" => {
                if let Some(lid) = self.active_label {
                    self.label_overflow_open = false;
                    self.picker_contact_ids = self
                        .contacts
                        .iter()
                        .filter(|c| !c.labels.contains(&lid))
                        .take(MAX_PICKER_ROWS)
                        .map(|c| c.id)
                        .collect();
                    self.picker_checked = [false; MAX_PICKER_ROWS];
                    if self.picker_contact_ids.is_empty() {
                        self.show_toast(f, "All contacts are already in this label", 1.8);
                    } else {
                        self.home_dialog = HomeDialog::AddToLabel(lid);
                    }
                }
            }
            "btn_label_more" => {
                self.label_overflow_open = !self.label_overflow_open;
            }
            "menu_label_email" => {
                self.label_overflow_open = false;
                self.show_toast(f, "Composing group email…", 1.8);
            }
            "menu_label_sms" => {
                self.label_overflow_open = false;
                self.show_toast(f, "Composing group message…", 1.8);
            }
            "menu_label_remove" => {
                self.label_overflow_open = false;
                if let Some(lid) = self.active_label {
                    self.picker_contact_ids = self
                        .contacts
                        .iter()
                        .filter(|c| c.labels.contains(&lid))
                        .take(MAX_PICKER_ROWS)
                        .map(|c| c.id)
                        .collect();
                    self.picker_checked = [false; MAX_PICKER_ROWS];
                    if self.picker_contact_ids.is_empty() {
                        self.show_toast(f, "No contacts in this label", 1.6);
                    } else {
                        self.home_dialog = HomeDialog::RemoveFromLabel(lid);
                    }
                }
            }
            "menu_label_rename" => {
                self.label_overflow_open = false;
                if let Some(lid) = self.active_label {
                    let cur = self.label_name(lid);
                    self.state.text.insert("dialog_label_input".into(), cur);
                    self.home_dialog = HomeDialog::RenameLabel(lid);
                }
            }
            "menu_label_delete" => {
                self.label_overflow_open = false;
                if let Some(lid) = self.active_label {
                    self.home_dialog = HomeDialog::DeleteLabel(lid);
                }
            }
            "drawer_my_info" => {
                self.open_editor(f, EditorTarget::MyInfo);
            }
            "drawer_contacts" => {
                self.active_label = None;
                self.search_mode = false;
                self.cab_mode = false;
                self.label_overflow_open = false;
                self.state
                    .drawer_open
                    .insert("drawer_layout".into(), false);
            }
            "drawer_create_label" => {
                self.state
                    .drawer_open
                    .insert("drawer_layout".into(), false);
                self.state
                    .text
                    .insert("dialog_label_input".into(), String::new());
                self.home_dialog = HomeDialog::CreateLabel;
            }
            "drawer_emergency" => {
                self.state
                    .drawer_open
                    .insert("drawer_layout".into(), false);
                self.navigate(f, Screen::Settings(SettingsSub::Emergency), TransitionKind::Push);
            }
            "drawer_settings" => {
                self.state
                    .drawer_open
                    .insert("drawer_layout".into(), false);
                self.navigate(f, Screen::Settings(SettingsSub::Main), TransitionKind::Push);
            }
            "home_dialog_scrim" | "home_dialog_cancel" => {
                self.home_dialog = HomeDialog::None;
            }
            "home_dialog_ok" => match self.home_dialog {
                HomeDialog::CreateLabel => {
                    let name = self
                        .state
                        .text
                        .get("dialog_label_input")
                        .map(|s| s.trim().to_owned())
                        .unwrap_or_default();
                    if name.is_empty() {
                        self.show_toast(f, "Enter a label name", 1.5);
                    } else if self.labels.len() >= MAX_DRAWER_LABELS {
                        self.show_toast(f, "Max demo labels reached", 1.6);
                        self.home_dialog = HomeDialog::None;
                    } else {
                        let lid = self.next_label_id;
                        self.next_label_id += 1;
                        self.labels.push(Label { id: lid, name });
                        self.active_label = Some(lid);
                        self.home_dialog = HomeDialog::None;
                        self.show_toast(f, "Label created", 1.8);
                    }
                }
                HomeDialog::RenameLabel(lid) => {
                    let name = self
                        .state
                        .text
                        .get("dialog_label_input")
                        .map(|s| s.trim().to_owned())
                        .unwrap_or_default();
                    if !name.is_empty()
                        && let Some(lbl) = self.labels.iter_mut().find(|l| l.id == lid)
                    {
                        lbl.name = name;
                        self.show_toast(f, "Label renamed", 1.6);
                    }
                    self.home_dialog = HomeDialog::None;
                }
                HomeDialog::DeleteLabel(lid) => {
                    self.labels.retain(|l| l.id != lid);
                    for c in &mut self.contacts {
                        c.labels.retain(|id| *id != lid);
                    }
                    if self.active_label == Some(lid) {
                        self.active_label = None;
                    }
                    self.home_dialog = HomeDialog::None;
                    self.show_toast(f, "Label deleted", 1.8);
                }
                HomeDialog::DeleteSelected => {
                    let count = self.selected_ids.len();
                    let del = self.selected_ids.clone();
                    self.contacts.retain(|c| !del.contains(&c.id));
                    self.selected_ids.clear();
                    self.cab_mode = false;
                    self.home_dialog = HomeDialog::None;
                    self.show_toast(f, format!("Deleted {count} contact(s)"), 1.8);
                }
                HomeDialog::AddToLabel(lid) => {
                    let mut added = 0;
                    let ids = self.picker_contact_ids.clone();
                    for (i, cid) in ids.into_iter().enumerate() {
                        if self.picker_checked[i]
                            && let Some(c) = self.contact_by_id_mut(cid)
                            && !c.labels.contains(&lid)
                        {
                            c.labels.push(lid);
                            added += 1;
                        }
                    }
                    self.home_dialog = HomeDialog::None;
                    if added > 0 {
                        self.show_toast(f, format!("Added {added} contact(s)"), 1.8);
                    }
                }
                HomeDialog::RemoveFromLabel(lid) => {
                    let mut removed = 0;
                    let ids = self.picker_contact_ids.clone();
                    for (i, cid) in ids.into_iter().enumerate() {
                        if self.picker_checked[i]
                            && let Some(c) = self.contact_by_id_mut(cid)
                        {
                            c.labels.retain(|id| *id != lid);
                            removed += 1;
                        }
                    }
                    self.home_dialog = HomeDialog::None;
                    if removed > 0 {
                        self.show_toast(f, format!("Removed {removed} contact(s)"), 1.8);
                    }
                }
                HomeDialog::None => {}
            },
            a if a.starts_with("drawer_label_") => {
                if let Ok(idx) = a["drawer_label_".len()..].parse::<usize>()
                    && let Some(lbl) = self.labels.get(idx)
                {
                    self.active_label = Some(lbl.id);
                    self.search_mode = false;
                    self.cab_mode = false;
                    self.label_overflow_open = false;
                    self.state
                        .drawer_open
                        .insert("drawer_layout".into(), false);
                }
            }
            a if a.starts_with("picker_row_") || a.starts_with("picker_check_") => {
                let tail = a
                    .strip_prefix("picker_row_")
                    .or_else(|| a.strip_prefix("picker_check_"))
                    .unwrap_or("");
                if let Ok(idx) = tail.parse::<usize>()
                    && idx < MAX_PICKER_ROWS
                {
                    self.picker_checked[idx] = !self.picker_checked[idx];
                }
            }
            a if a.starts_with("contact_row_") || a.starts_with("row_check_") => {
                self.label_overflow_open = false;
                let tail = a
                    .strip_prefix("contact_row_")
                    .or_else(|| a.strip_prefix("row_check_"))
                    .unwrap_or("");
                if let Ok(idx) = tail.parse::<usize>()
                    && let Some(row) = self.displayed_rows.get(idx).copied()
                {
                    if self.cab_mode {
                        if let Some(pos) =
                            self.selected_ids.iter().position(|&id| id == row.contact_id)
                        {
                            self.selected_ids.remove(pos);
                            if self.selected_ids.is_empty() {
                                self.cab_mode = false;
                            }
                        } else {
                            self.selected_ids.push(row.contact_id);
                        }
                    } else {
                        self.detail_overflow_open = false;
                        self.detail_dialog = DetailDialog::None;
                        self.state.offsets.insert("detail_scroll".into(), 0.0);
                        self.navigate(f, Screen::Detail(row.contact_id), TransitionKind::Push);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_detail_action(&mut self, f: &mut Frame<'_>, cid: u32, action: &str) {
        match action {
            "detail_back" => {
                self.detail_overflow_open = false;
                self.detail_dialog = DetailDialog::None;
                self.navigate(f, Screen::Home, TransitionKind::Pop);
            }
            "detail_star" => {
                self.detail_overflow_open = false;
                if let Some(c) = self.contact_by_id_mut(cid) {
                    c.starred = !c.starred;
                    let msg = if c.starred {
                        "Added to favorites"
                    } else {
                        "Removed from favorites"
                    };
                    self.show_toast(f, msg, 1.5);
                }
            }
            "detail_edit" => {
                self.detail_overflow_open = false;
                self.open_editor(f, EditorTarget::EditContact(cid));
            }
            "detail_more" => {
                self.detail_overflow_open = !self.detail_overflow_open;
            }
            "detail_phone_row" => {
                self.detail_overflow_open = false;
                if let Some(c) = self.contact_by_id(cid) {
                    self.show_toast(f, format!("Calling {}…", c.phone), 1.8);
                }
            }
            "detail_sms" => {
                self.detail_overflow_open = false;
                if let Some(c) = self.contact_by_id(cid) {
                    self.show_toast(f, format!("Messaging {}…", c.phone), 1.8);
                }
            }
            "detail_email_row" => {
                self.detail_overflow_open = false;
                if let Some(c) = self.contact_by_id(cid) {
                    self.show_toast(f, format!("Emailing {}…", c.email), 1.8);
                }
            }
            "detail_menu_link" => {
                self.detail_overflow_open = false;
                if self.contacts.len() < 2 {
                    self.show_toast(f, "No other contacts to link", 1.6);
                } else {
                    self.detail_dialog = DetailDialog::Link;
                }
            }
            "detail_menu_delete" => {
                self.detail_overflow_open = false;
                self.detail_dialog = DetailDialog::Delete;
            }
            "detail_menu_share" => {
                self.detail_overflow_open = false;
                self.detail_dialog = DetailDialog::Share;
            }
            "detail_menu_shortcut" => {
                self.detail_overflow_open = false;
                self.show_toast(f, "Home screen shortcut created", 1.8);
            }
            "detail_menu_ringtone" => {
                self.detail_overflow_open = false;
                self.detail_dialog = DetailDialog::Ringtone;
            }
            "detail_dialog_scrim" | "detail_dialog_cancel" => {
                self.detail_dialog = DetailDialog::None;
            }
            "detail_dialog_confirm" => match self.detail_dialog {
                DetailDialog::Delete => {
                    self.contacts.retain(|c| c.id != cid);
                    self.detail_dialog = DetailDialog::None;
                    self.navigate(f, Screen::Home, TransitionKind::Pop);
                    self.show_toast(f, "Contact deleted", 1.8);
                }
                DetailDialog::Share => {
                    self.detail_dialog = DetailDialog::None;
                    self.show_toast(f, "vCard shared", 1.8);
                }
                _ => {
                    self.detail_dialog = DetailDialog::None;
                }
            },
            a if a.starts_with("detail_opt_") => {
                if let Ok(idx) = a["detail_opt_".len()..].parse::<usize>() {
                    match self.detail_dialog {
                        DetailDialog::Ringtone => {
                            if let Some(c) = self.contact_by_id_mut(cid) {
                                c.ringtone = idx % RINGTONES.len();
                            }
                            self.detail_dialog = DetailDialog::None;
                            self.show_toast(
                                f,
                                format!("Ringtone set to {}", RINGTONES[idx % RINGTONES.len()]),
                                1.6,
                            );
                        }
                        DetailDialog::Link => {
                            if let Some(&other_id) = self.detail_link_ids.get(idx)
                                && let Some(other) = self.contact_by_id(other_id)
                            {
                                let name = other.display_name(false);
                                self.show_toast(f, format!("Linked with {name}"), 1.8);
                            }
                            self.detail_dialog = DetailDialog::None;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_editor_action(&mut self, f: &mut Frame<'_>, action: &str) {
        let get = |s: &LayoutState, k: &str| {
            s.text.get(k).map(|v| v.trim().to_owned()).unwrap_or_default()
        };
        match action {
            "editor_cancel" => {
                if self.editor_dirty {
                    self.editor_dialog = EditorDialog::Discard;
                } else {
                    self.leave_editor(f);
                }
            }
            "editor_save" => {
                let first = get(&self.state, "ed_first_name");
                let last = get(&self.state, "ed_last_name");
                let phone = get(&self.state, "ed_phone");
                let email = get(&self.state, "ed_email");
                let org = get(&self.state, "ed_org");
                if first.is_empty() && last.is_empty() && phone.is_empty() && email.is_empty() && org.is_empty() {
                    self.show_toast(f, "Nothing to save — enter a name or phone", 1.8);
                    return;
                }
                match self.editor_target {
                    EditorTarget::MyInfo => {
                        self.my_info_name = format!("{first} {last}").trim().to_owned();
                        self.my_info_phone = phone;
                        self.my_info_email = email;
                        self.navigate(f, Screen::Home, TransitionKind::ModalDown);
                        self.show_toast(f, "Profile saved", 1.8);
                    }
                    EditorTarget::NewContact => {
                        let cid = self.next_contact_id;
                        self.next_contact_id += 1;
                        let mut labels = Vec::new();
                        if let Some(lid) = self.active_label {
                            labels.push(lid);
                        }
                        let contact = Contact {
                            id: cid,
                            prefix: get(&self.state, "ed_prefix"),
                            first_name: first,
                            middle_name: get(&self.state, "ed_middle_name"),
                            last_name: last,
                            suffix: get(&self.state, "ed_suffix"),
                            phone,
                            phone_type: self.editor_phone_type,
                            email,
                            email_type: self.editor_email_type,
                            org,
                            title: get(&self.state, "ed_title"),
                            address: get(&self.state, "ed_address"),
                            notes: get(&self.state, "ed_notes"),
                            starred: false,
                            color_idx: self.editor_color_idx,
                            ringtone: 0,
                            labels,
                        };
                        let name = contact.display_name(false);
                        self.contacts.push(contact);
                        self.detail_overflow_open = false;
                        self.detail_dialog = DetailDialog::None;
                        self.navigate(f, Screen::Detail(cid), TransitionKind::ModalDown);
                        self.show_toast(f, format!("{name} saved"), 2.0);
                    }
                    EditorTarget::EditContact(cid) => {
                        let prefix = get(&self.state, "ed_prefix");
                        let middle = get(&self.state, "ed_middle_name");
                        let suffix = get(&self.state, "ed_suffix");
                        let title = get(&self.state, "ed_title");
                        let address = get(&self.state, "ed_address");
                        let notes = get(&self.state, "ed_notes");
                        let ptype = self.editor_phone_type;
                        let etype = self.editor_email_type;
                        let color_idx = self.editor_color_idx;
                        let saved_name = if let Some(c) = self.contact_by_id_mut(cid) {
                            c.prefix = prefix;
                            c.first_name = first;
                            c.middle_name = middle;
                            c.last_name = last;
                            c.suffix = suffix;
                            c.phone = phone;
                            c.phone_type = ptype;
                            c.email = email;
                            c.email_type = etype;
                            c.org = org;
                            c.title = title;
                            c.address = address;
                            c.notes = notes;
                            c.color_idx = color_idx;
                            c.display_name(false)
                        } else {
                            "Contact".into()
                        };
                        self.navigate(f, Screen::Detail(cid), TransitionKind::ModalDown);
                        self.show_toast(f, format!("{saved_name} saved"), 2.0);
                    }
                }
            }
            "editor_photo_area" => {
                self.editor_dialog = EditorDialog::ChangePhoto;
            }
            "editor_expand_name" => {
                self.editor_name_expanded = !self.editor_name_expanded;
            }
            "editor_phone_type_btn" => {
                self.editor_dialog = EditorDialog::PhoneType;
            }
            "editor_email_type_btn" => {
                self.editor_dialog = EditorDialog::EmailType;
            }
            "editor_more_fields_btn" => {
                self.editor_more_fields = true;
            }
            "editor_dialog_scrim" | "editor_dialog_cancel" => {
                self.editor_dialog = EditorDialog::None;
            }
            "editor_dialog_confirm" => {
                self.editor_dialog = EditorDialog::None;
                self.leave_editor(f);
            }
            a if a.starts_with("ed_opt_") => {
                if let Ok(idx) = a["ed_opt_".len()..].parse::<usize>() {
                    match self.editor_dialog {
                        EditorDialog::ChangePhoto => {
                            if idx == 2 {
                                self.editor_color_idx = (self.editor_color_idx + 1) % 6;
                                self.editor_dirty = true;
                                self.show_toast(f, "Avatar color updated", 1.5);
                            } else {
                                let msg = if idx == 0 {
                                    "Camera capture (offline demo)"
                                } else {
                                    "Photo picker (offline demo)"
                                };
                                self.show_toast(f, msg, 1.6);
                            }
                            self.editor_dialog = EditorDialog::None;
                        }
                        EditorDialog::PhoneType => {
                            self.editor_phone_type = idx % PHONE_TYPES.len();
                            self.editor_dirty = true;
                            self.editor_dialog = EditorDialog::None;
                        }
                        EditorDialog::EmailType => {
                            self.editor_email_type = idx % EMAIL_TYPES.len();
                            self.editor_dirty = true;
                            self.editor_dialog = EditorDialog::None;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn leave_editor(&mut self, f: &mut Frame<'_>) {
        let target = match self.editor_target {
            EditorTarget::EditContact(cid) => Screen::Detail(cid),
            _ => Screen::Home,
        };
        self.navigate(f, target, TransitionKind::ModalDown);
    }

    fn handle_settings_action(&mut self, f: &mut Frame<'_>, sub: SettingsSub, action: &str) {
        match action {
            "settings_back" => {
                self.settings_dialog = SettingsDialog::None;
                let target = match sub {
                    SettingsSub::Main | SettingsSub::Emergency => Screen::Home,
                    SettingsSub::About | SettingsSub::Blocked => {
                        Screen::Settings(SettingsSub::Main)
                    }
                };
                self.navigate(f, target, TransitionKind::Pop);
            }
            "pref_my_info" => {
                self.open_editor(f, EditorTarget::MyInfo);
            }
            "pref_accounts" => {
                self.show_toast(f, "Local device account active", 1.8);
            }
            "pref_default_account" => {
                self.settings_radio_choice = self.default_account;
                self.settings_dialog = SettingsDialog::DefaultAccount;
            }
            "pref_contacts_to_display" => {
                self.settings_radio_choice = self.display_mode;
                self.settings_dialog = SettingsDialog::ContactsToDisplay;
            }
            "pref_sort_by" => {
                self.settings_radio_choice = usize::from(self.sort_by_last);
                self.settings_dialog = SettingsDialog::SortBy;
            }
            "pref_name_format" => {
                self.settings_radio_choice = usize::from(self.last_name_first);
                self.settings_dialog = SettingsDialog::NameFormat;
            }
            "pref_phonetic" => {
                self.settings_radio_choice = usize::from(!self.phonetic_always);
                self.settings_dialog = SettingsDialog::PhoneticName;
            }
            "pref_import" => {
                self.settings_radio_choice = 0;
                self.settings_dialog = SettingsDialog::Import;
            }
            "pref_export" => {
                self.settings_radio_choice = 0;
                self.settings_dialog = SettingsDialog::Export;
            }
            "pref_blocked" => {
                self.navigate(f, Screen::Settings(SettingsSub::Blocked), TransitionKind::Push);
            }
            "pref_about" => {
                self.navigate(f, Screen::Settings(SettingsSub::About), TransitionKind::Push);
            }
            "about_licenses" => self.show_toast(f, "Apache-2.0 / GPL-3.0 notices included", 1.8),
            "about_privacy" => self.show_toast(f, "Offline demo — zero network access", 1.8),
            "about_terms" => self.show_toast(f, "AOSP Contacts v1.7.34 reference flow", 1.8),
            "emerg_add_info" => {
                self.emerg_info_count += 1;
                self.show_toast(f, "Emergency medical note added", 1.6);
            }
            "emerg_add_contact" => {
                self.emerg_contact_count += 1;
                self.show_toast(f, "Emergency contact added", 1.6);
            }
            "blocked_add_number" => {
                let next = format!("+1 555-019-{:04}", 1000 + self.blocked_numbers.len() * 111);
                self.blocked_numbers.push(next.clone());
                self.show_toast(f, format!("Blocked {next}"), 1.8);
            }
            "settings_dialog_scrim" | "settings_dialog_cancel" => {
                self.settings_dialog = SettingsDialog::None;
            }
            "settings_dialog_ok" => {
                match self.settings_dialog {
                    SettingsDialog::DefaultAccount => {
                        self.default_account = self.settings_radio_choice;
                    }
                    SettingsDialog::ContactsToDisplay => {
                        self.display_mode = self.settings_radio_choice;
                    }
                    SettingsDialog::SortBy => {
                        self.sort_by_last = self.settings_radio_choice == 1;
                    }
                    SettingsDialog::NameFormat => {
                        self.last_name_first = self.settings_radio_choice == 1;
                    }
                    SettingsDialog::PhoneticName => {
                        self.phonetic_always = self.settings_radio_choice == 0;
                    }
                    SettingsDialog::Import => {
                        self.show_toast(f, "Imported contacts from .vcf (demo)", 1.8);
                    }
                    SettingsDialog::Export => {
                        self.show_toast(
                            f,
                            format!("Exported {} contacts to contacts.vcf", self.contacts.len()),
                            2.0,
                        );
                    }
                    SettingsDialog::None => {}
                }
                self.settings_dialog = SettingsDialog::None;
            }
            a if a.starts_with("set_opt_") => {
                if let Ok(idx) = a["set_opt_".len()..].parse::<usize>() {
                    self.settings_radio_choice = idx;
                    // Instant apply for single-choice radio dialogs matching Android AlertDialog
                    if matches!(
                        self.settings_dialog,
                        SettingsDialog::SortBy
                            | SettingsDialog::NameFormat
                            | SettingsDialog::PhoneticName
                            | SettingsDialog::DefaultAccount
                            | SettingsDialog::ContactsToDisplay
                    ) {
                        self.handle_settings_action(f, sub, "settings_dialog_ok");
                    }
                }
            }
            _ => {}
        }
    }
}
