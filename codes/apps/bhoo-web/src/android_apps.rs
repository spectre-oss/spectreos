//! Four offline integration fixtures. Original row resources are compiled separately;
//! activities, adapters and data stores are explicitly replaced by this Rust controller.
use bhoo::*;
#[path = "android_app_rows.rs"]
mod rows;
use bhoo::android_reference::NativeState;
#[derive(Clone)]
struct Entry {
    name: String,
    subject: String,
    body: String,
    archived: bool,
    starred: bool,
    unread: bool,
    garden: bool,
}
#[derive(Default)]
pub struct AndroidApps {
    app: usize,
    initialized: bool,
    entries: Vec<Entry>,
    selected: Option<usize>,
    folder: usize,
    search: bool,
    compose: bool,
    contacts: bool,
    pub fields: [String; 4],
    pub editing: usize,
    offset: f32,
    detail_offset: f32,
    dialog: OverlayState,
    drawer: OverlayState,
    overflow: OverlayState,
    overflow_sort: bool,
    sort_by: u8,
    page_key: Option<(bool, bool, Option<usize>, usize)>,
    page_started: f64,
    page_direction: f32,
    notice: String,
    native: NativeState,
    pub revision: u32,
}
fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width: width.max(0.),
        height: height.max(0.),
    }
}
fn label(f: &mut Frame<'_>, b: Rect, s: &str, size: f32, bold: bool) {
    f.styled_text(
        b,
        s,
        f.theme().text,
        TextStyle {
            size,
            weight: if bold { 600 } else { 400 },
            ..TextStyle::default()
        },
    );
}
fn icon(f: &mut Frame<'_>, id: u64, b: Rect, name: &str, icon: MaterialIcon) -> bool {
    f.material_icon_button_at(
        id,
        b,
        icon,
        name,
        Options { variant: ButtonVariant::Text, ..Options::default() },
    )
    .clicked
}
fn button(f: &mut Frame<'_>, id: u64, b: Rect, name: &str) -> bool {
    f.action_at(
        id,
        b,
        name,
        Options {
            variant: ButtonVariant::Tonal,
            ..Options::default()
        },
    )
    .clicked
}
impl AndroidApps {
    fn init(&mut self, app: usize) {
        if self.initialized {
            return;
        }
        self.app = app;
        self.initialized = true;
        let names = if app == 3 {
            vec![
                "Apple",
                "Avocado",
                "Beet",
                "Bougainvillea",
                "Cilantro",
                "Eggplant",
                "Grape",
                "Hibiscus",
                "Lettuce",
                "Mango",
                "Orange",
                "Pepper",
                "Potato",
                "Sunflower",
                "Tomato",
                "Watermelon",
            ]
        } else {
            vec![
                "Alex Morgan",
                "Bhoo team",
                "Mina Lee",
                "Family",
                "Nora Kim",
                "Design studio",
                "Book club",
                "Jamie Bell",
                "Travel plans",
                "Community",
                "Garden club",
                "Robin Chen",
                "Sam Taylor",
                "Project updates",
                "Weekend plans",
                "Avery Lane",
                "Workshop",
                "Kai Patel",
                "Friends",
                "Local library",
                "Dana Park",
                "Studio notes",
                "Chris Reed",
                "SpectreOS",
            ]
        };
        for (i, name) in names.into_iter().enumerate() {
            let subject = if app == 2 {
                [
                    "A fresh start for the project",
                    "Your weekly update",
                    "Design review on Thursday",
                    "Notes from our last meeting",
                ][i % 4]
            } else if app == 3 {
                "Growing guide"
            } else {
                [
                    "See you tomorrow!",
                    "The new build is ready to try.",
                    "Thanks for the update",
                    "Could you share your thoughts?",
                ][i % 4]
            };
            let body = if app == 3 {
                format!(
                    "{name} grows best with adequate sunlight and well-drained soil.\n\nThis is fictional gardening data for testing the interface.\n\nWater when the top layer of soil is dry. Inspect the leaves regularly and give the plant room to grow."
                )
            } else if app == 2 {
                format!(
                    "Hello,\n\nHere are the notes for our next review. We have been testing scrolling, input, navigation and local persistence in Bhoo.\n\nPlease take a look and share your thoughts. There is no live email account connected to this demo.\n\nThanks,\n{name}"
                )
            } else {
                format!(
                    "{name}: Hello! How is your day going?\nYou: Good, thanks. I am trying the new interface.\n{name}: {subject}"
                )
            };
            self.entries.push(Entry {
                name: name.into(),
                subject: subject.into(),
                body,
                archived: false,
                starred: false,
                unread: i % 3 == 0,
                garden: app == 3 && i == 13,
            });
        }
    }
    pub fn edit(&mut self, text: &str) {
        self.fields[self.editing] = prefix(text, [128, 2048, 128, 256][self.editing]);
        self.offset = 0.;
        self.revision = self.revision.wrapping_add(1);
    }
    pub fn focus(&mut self, id: u64) -> String {
        self.editing = (id.saturating_sub(9100) as usize).min(3);
        self.fields[self.editing].clone()
    }
    fn field(&mut self, f: &mut Frame<'_>, index: usize, b: Rect, name: &str) {
        f.text_field_at(9100 + index as u64, b, name, &self.fields[index], false, true);
    }
    fn back(&mut self) {
        self.selected = None;
        self.compose = false;
        self.contacts = false;
        self.search = false;
        self.fields[0].clear();
        self.offset = 0.;
        self.detail_offset = 0.;
    }
    fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
    pub fn show(&mut self, f: &mut Frame<'_>, w: f32, h: f32, app: usize) {
        self.init(app);
        let t = f.theme();
        f.paint_rect(rect(0., 0., w, h), 0., t.background);
        let width = w.min(900.);
        let x = (w - width) / 2.;
        let list = self.selected.is_none() && !self.compose && !self.contacts;
        let title = if self.compose {
            "Compose message".to_owned()
        } else if self.contacts {
            "New conversation".into()
        } else if let Some(i) = self.selected {
            self.entries[i].name.clone()
        } else {
            match app {
                0 => "Messages",
                1 => "Delta Chat",
                2 => ["Inbox", "Starred", "Archive"][self.folder.min(2)],
                _ => "Sunflower",
            }
            .into()
        };
        let page = (self.compose, self.contacts, self.selected, self.folder);
        if let Some(previous) = self.page_key {
            if previous != page {
                self.page_started = f.time();
                self.page_direction = if page.2.is_none() && previous.2.is_some()
                    || (page.2.is_none() && previous.2.is_none() && page.3 < previous.3)
                {
                    -1.
                } else {
                    1.
                };
                self.page_key = Some(page);
            }
        } else {
            self.page_key = Some(page);
            self.page_started = f.time() - 1.;
        }
        let progress = if f.reduced_motion() {
            1.
        } else {
            ((f.time() - self.page_started) / 0.2).clamp(0., 1.) as f32
        };
        let motion = bhoo::timing::Easing::Standard.sample(progress);
        if progress < 1. {
            f.continuous(rect(x, 64., width, h - 64.));
        }
        let mut offset = self.page_direction * (1. - motion) * if app == 3 { width } else { 32. };
        f.without_input(
            self.drawer.open || self.overflow.open || self.dialog.open,
            |f| {
                f.paint_rect(rect(x, 0., width, 64.), 0., t.surface_low);
                let toolbar_search = app == 2 && list && self.search;
                if toolbar_search {
                    if icon(
                        f,
                        9000,
                        rect(x + 4., 8., 48., 48.),
                        "Close search",
                        MaterialIcon::ArrowBack,
                    ) || f.pressed_key(Key::Escape)
                    {
                        self.search = false;
                        self.fields[0].clear();
                        self.offset = 0.;
                    }
                    let field = rect(x + 60., 8., (width - 120.).max(0.), 48.);
                    f.target(9100, field, "Search locally", true, Role::TextField);
                    let query = &self.fields[0];
                    label(
                        f,
                        rect(field.x + 8., field.y, field.width - 16., field.height),
                        if query.is_empty() { "Search" } else { query },
                        16.,
                        false,
                    );
                    if !query.is_empty()
                        && icon(
                            f,
                            9003,
                            rect(x + width - 52., 8., 48., 48.),
                            "Clear search",
                            MaterialIcon::Close,
                        )
                    {
                        self.fields[0].clear();
                        self.offset = 0.;
                    }
                } else {
                    if !(app == 3 && list)
                        && icon(
                            f,
                            9000,
                            rect(x + 4., 8., 48., 48.),
                            if list && app == 2 {
                                "Open navigation"
                            } else if list {
                                "About this demo"
                            } else {
                                "Back"
                            },
                            if list {
                                MaterialIcon::Menu
                            } else {
                                MaterialIcon::ArrowBack
                            },
                        )
                    {
                        if list && app == 2 {
                            self.drawer.open = true
                        } else if list {
                            self.dialog.open = true
                        } else {
                            self.back();
                        }
                    }
                    if app == 3 && list {
                        f.styled_text(
                            rect(x + 60., 8., width - 120., 48.),
                            &title,
                            t.text,
                            TextStyle {
                                size: 20.,
                                weight: 600,
                                align: 1,
                                ..TextStyle::default()
                            },
                        );
                    } else {
                        label(f, rect(x + 64., 8., width - 176., 48.), &title, 20., true);
                    }
                    if list
                        && icon(
                            f,
                            9001,
                            rect(x + width - 104., 8., 48., 48.),
                            "Search",
                            MaterialIcon::Search,
                        )
                    {
                        self.search = !self.search;
                        self.fields[0].clear();
                        self.offset = 0.;
                    }
                    if icon(
                        f,
                        9002,
                        rect(x + width - 52., 8., 48., 48.),
                        if app == 2 {
                            "More options"
                        } else {
                            "Demo information"
                        },
                        MaterialIcon::MoreVert,
                    ) {
                        if app == 2 {
                            self.overflow.open = true;
                            self.overflow_sort = false;
                        } else {
                            self.dialog.open = true;
                        }
                    }
                }
                let mut top = 64.;
                if self.search && list && app != 2 {
                    self.field(
                        f,
                        0,
                        rect(x + 12., top + 8., width - 24., 56.),
                        "Search locally",
                    );
                    top += 72.;
                }
                if list {
                    if app == 3 {
                        for (i, title) in ["My garden", "Plant list"].iter().enumerate() {
                            let b = rect(x + i as f32 * width / 2., top, width / 2., 56.);
                            let response = f.target(9020 + i as u64, b, title, true, Role::Tab);
                            f.state_layer(response, 0., t.primary);
                            f.paint_material_icon(
                                rect(b.x + b.width / 2. - 12., b.y + 5., 24., 24.),
                                if i == 0 {
                                    MaterialIcon::Home
                                } else {
                                    MaterialIcon::Widgets
                                },
                                t.text,
                            );
                            f.styled_text(
                                rect(b.x, b.y + 30., b.width, 22.),
                                title,
                                t.text,
                                TextStyle {
                                    size: 12.,
                                    align: 1,
                                    ..TextStyle::default()
                                },
                            );
                            f.focus_ring(response);
                            if response.clicked && self.folder != i {
                                self.page_direction = if i > self.folder { 1. } else { -1. };
                                self.folder = i;
                                self.offset = 0.;
                                self.page_key =
                                    Some((self.compose, self.contacts, self.selected, self.folder));
                                self.page_started = f.time();
                                offset = if f.reduced_motion() {
                                    0.
                                } else {
                                    self.page_direction * width
                                };
                                f.continuous(rect(x, top, width, h - top));
                            }
                        }
                        let tab_pos = f.transition(
                            9029,
                            1,
                            self.folder as f32,
                            self.folder as f32,
                            0.2,
                            bhoo::timing::Easing::Standard,
                        );
                        f.paint_rect(
                            rect(
                                x + tab_pos * width / 2. + width / 8.,
                                top + 53.,
                                width / 4.,
                                3.,
                            ),
                            1.5,
                            t.primary,
                        );
                        top += 56.;
                    }
                    let bottom = if app == 2 { 72. } else { 28. };
                    let viewport = rect(x, top, width, h - top - bottom);
                    let fab_bounds = rect(x + width - 76., h - bottom - 76., 56., 56.);
                    let fab_response = if app != 3 {
                        Some(f.target(
                            9040,
                            fab_bounds,
                            if app == 2 {
                                "Compose"
                            } else {
                                "New conversation"
                            },
                            true,
                            Role::Button,
                        ))
                    } else {
                        None
                    };
                    f.within(viewport, |f| {
                        let shifted = rect(
                            viewport.x + offset,
                            viewport.y,
                            viewport.width,
                            viewport.height,
                        );
                        if app == 3 {
                            self.plants(f, shifted);
                        } else {
                            self.inbox(f, shifted);
                        }
                    });
                    if app == 2 {
                        f.paint_rect(rect(x, h - 72., width, 72.), 0., t.surface_low);
                        for (i, title) in ["Inbox", "Starred", "Archive"].iter().enumerate() {
                            if f.material_navigation_bar_item(
                                9030 + i as u64,
                                rect(x + i as f32 * width / 3., h - 72., width / 3., 72.),
                                title,
                                [
                                    MaterialIcon::Inbox,
                                    MaterialIcon::Star,
                                    MaterialIcon::Archive,
                                ][i],
                                self.folder == i,
                                true,
                            )
                            .clicked
                            {
                                self.folder = i;
                                self.offset = 0.;
                            }
                        }
                    }
                    if let Some(response) = fab_response {
                        f.paint_rect(fab_bounds, 16., t.primary_container);
                        f.state_layer(response, 16., t.on_primary_container);
                        f.paint_material_icon(
                            rect(fab_bounds.x + 16., fab_bounds.y + 16., 24., 24.),
                            MaterialIcon::Add,
                            t.on_primary_container,
                        );
                        f.focus_ring(response);
                        if response.clicked {
                            if app == 2 {
                                self.start_compose(None);
                            } else {
                                self.contacts = true;
                                self.offset = 0.;
                            }
                        }
                    }
                    if app != 2 {
                        label(
                            f,
                            rect(x + 16., h - 26., width - 32., 24.),
                            "Offline Bhoo integration demo · fictional data",
                            11.,
                            false,
                        );
                    }
                } else if self.contacts {
                    label(
                        f,
                        rect(x + 16., top + 8., width - 32., 32.),
                        "Choose a demo contact",
                        16.,
                        false,
                    );
                    let b = rect(x, top + 48., width, h - top - 48.);
                    f.scroll_viewport(
                        9050,
                        b,
                        &mut self.offset,
                        (self.entries.len() as f32 * 64. - b.height).max(0.),
                    );
                    let offset = self.offset;
                    f.within(b, |f| {
                        for i in 0..self.entries.len() {
                            let row =
                                rect(x + 12., b.y + i as f32 * 64. - offset, width - 24., 56.);
                            if row.y + row.height < b.y || row.y > b.y + b.height {
                                continue;
                            }
                            if button(f, 9060 + i as u64, row, &self.entries[i].name) {
                                self.selected = Some(i);
                                self.contacts = false;
                                self.detail_offset = 0.;
                            }
                        }
                    });
                } else if self.compose {
                    let viewport = rect(x, top, width, h - top);
                    f.within(viewport, |f| {
                        self.compose_view(f, rect(x + offset, top, width, h - top))
                    });
                } else if let Some(i) = self.selected {
                    let viewport = rect(x, top, width, h - top);
                    f.within(viewport, |f| {
                        let b = rect(x + offset, top, width, h - top);
                        if app == 3 {
                            self.plant_detail(f, b, i)
                        } else if app == 2 {
                            self.mail_detail(f, b, i)
                        } else {
                            self.conversation(f, b, i)
                        }
                    });
                }
            },
        );
        self.drawer.kind = OverlayKind::LeftSideSheet;
        self.drawer.style = OverlayStyle::NavigationDrawer;
        // NavigationView's pinned M3 token is 360dp; DrawerLayout reserves
        // the Android minimum 64dp edge for the modal scrim.
        let drawer_width = (width - 64.).clamp(0., 360.);
        f.overlay(
            9980,
            &mut self.drawer,
            rect(x, 0., drawer_width, h),
            true,
            |f, open| {
                f.with_gap(0., |f| {
                    f.navigation_drawer_header("Thunderbird", Some("you@example.test"));
                    f.navigation_drawer_section_header("Mail");
                    for (i, name) in ["Inbox", "Starred", "Archive"].iter().enumerate() {
                        let b = f.reserve(56.);
                        if f.material_drawer_item(
                            9981 + i as u64,
                            b,
                            name,
                            [
                                MaterialIcon::Inbox,
                                MaterialIcon::Star,
                                MaterialIcon::Archive,
                            ][i],
                            self.folder == i,
                            true,
                        )
                        .clicked
                        {
                            self.folder = i;
                            self.offset = 0.;
                            *open = false;
                        }
                    }
                });
            },
        );
        self.overflow.kind = OverlayKind::Menu;
        self.overflow.style = OverlayStyle::AndroidMenu;
        let labels: &[&str] = if self.overflow_sort {
            &["Date", "Subject", "Sender", "Back to menu"]
        } else if list {
            &["Sort by", "Mark all as read", "About demo"]
        } else {
            &["About demo"]
        };
        self.overflow.initial_focus = Some(9941);
        if self.overflow.open
            && let Some(id) = f
                .focused_id()
                .filter(|id| (9941..9941 + labels.len() as u64).contains(id))
        {
            let index = (id - 9941) as usize;
            let next = if f.pressed_key(Key::Down) {
                Some((index + 1) % labels.len())
            } else if f.pressed_key(Key::Up) {
                Some((index + labels.len() - 1) % labels.len())
            } else if f.pressed_key(Key::Home) {
                Some(0)
            } else if f.pressed_key(Key::End) {
                Some(labels.len() - 1)
            } else {
                None
            };
            if let Some(next) = next {
                f.request_focus(9941 + next as u64);
            }
        }
        let menu_w = (width - 16.).clamp(0., 224.);
        let menu_h = labels.len() as f32 * 48. + 16.;
        let menu_y = 56_f32.min((h - menu_h - 8.).max(0.));
        self.overflow.transform_origin = Some([x + width - 8., menu_y]);
        let mut selected_action = None;
        f.overlay(
            9940,
            &mut self.overflow,
            rect(x + width - menu_w - 8., menu_y, menu_w, menu_h),
            false,
            |f, open| {
                f.with_gap(0., |f| {
                    for (i, title) in labels.iter().enumerate() {
                        let trailing = if list && !self.overflow_sort && i == 0 {
                            Some(MaterialIcon::ChevronRight)
                        } else if self.overflow_sort && i < 3 && self.sort_by as usize == i {
                            Some(MaterialIcon::Check)
                        } else {
                            None
                        };
                        if f.android_popup_item(9941 + i as u64, title, true, trailing)
                            .clicked
                        {
                            selected_action = Some(i);
                            if !(list && !self.overflow_sort && i == 0)
                                && !(self.overflow_sort && i == 3)
                            {
                                *open = false;
                            }
                        }
                    }
                });
            },
        );
        if let Some(i) = selected_action {
            if self.overflow_sort {
                if i < 3 {
                    self.sort_by = i as u8;
                    self.offset = 0.;
                    self.overflow_sort = false;
                } else {
                    self.overflow_sort = false;
                }
            } else if list {
                match i {
                    0 => self.overflow_sort = true,
                    1 => {
                        for entry in &mut self.entries {
                            let visible = match self.folder {
                                1 => entry.starred && !entry.archived,
                                2 => entry.archived,
                                _ => !entry.archived,
                            };
                            if visible {
                                entry.unread = false;
                            }
                        }
                        self.changed();
                    }
                    _ => self.dialog.open = true,
                }
            } else {
                self.dialog.open = true;
            }
        }
        self.dialog.kind = OverlayKind::Dialog;
        let dw = (width - 32.).min(460.);
        let dh = (h - 32.).min(380.);
        f.overlay(
            9990,
            &mut self.dialog,
            rect((w - dw) / 2., (h - dh) / 2., dw, dh),
            true,
            |f, open| {
                f.label(
                    [
                        "Fossify Messages · Bhoo",
                        "Delta Chat · Bhoo",
                        "Thunderbird · Bhoo",
                        "Sunflower · Bhoo",
                    ][app],
                );
                f.label("Unofficial offline integration example.");
                f.label("Original XML rows + local Rust controllers.");
                f.label("Data is saved only on this device.");
                f.label("No account, network or real delivery.");
                f.label("Source, changes and licenses are bundled.");
                if f.button(9991, "Close").clicked {
                    *open = false;
                }
            },
        );
    }
    fn inbox(&mut self, f: &mut Frame<'_>, b: Rect) {
        let query = self.fields[0].to_lowercase();
        let mut indices: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let folder = if self.app == 2 {
                    match self.folder {
                        1 => e.starred && !e.archived,
                        2 => e.archived,
                        _ => !e.archived,
                    }
                } else {
                    !e.archived
                };
                folder
                    && (query.is_empty()
                        || format!("{} {} {}", e.name, e.subject, e.body)
                            .to_lowercase()
                            .contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
        if self.app == 2 {
            match self.sort_by {
                1 => indices.sort_by_cached_key(|&i| self.entries[i].subject.to_lowercase()),
                2 => indices.sort_by_cached_key(|&i| self.entries[i].name.to_lowercase()),
                _ => {}
            }
        }
        let rh = if self.app == 2 { 88. } else { 76. };
        f.scroll_viewport(
            9200,
            b,
            &mut self.offset,
            (indices.len() as f32 * rh - b.height).max(0.),
        );
        if indices.is_empty() {
            label(
                f,
                rect(b.x + 24., b.y + 48., b.width - 48., 48.),
                "No conversations found",
                18.,
                false,
            );
        }
        let mut chosen = None;
        let offset = self.offset;
        f.within(b, |f| {
            for (row, &i) in indices.iter().enumerate() {
                let r = rect(b.x, b.y + row as f32 * rh - offset, b.width, rh);
                if r.y + rh < b.y || r.y > b.y + b.height {
                    continue;
                }
                let e = &self.entries[i];
                let ids = match self.app {
                    0 => [
                        "conversation_address",
                        "conversation_body_short",
                        "conversation_date",
                        "conversation_image",
                    ],
                    1 => ["from_text", "subject", "date", "avatar"],
                    _ => ["subject", "preview", "date", "contact_picture"],
                };
                self.native.text.insert(
                    ids[0].into(),
                    if self.app == 2 {
                        e.subject.clone()
                    } else {
                        e.name.clone()
                    },
                );
                self.native.text.insert(
                    ids[1].into(),
                    if self.app == 2 {
                        format!("{} — {}", e.name, e.body.replace('\n', " "))
                    } else {
                        e.subject.clone()
                    },
                );
                self.native.text.insert(
                    ids[2].into(),
                    if i == 0 {
                        "10:42".into()
                    } else {
                        "Yesterday".into()
                    },
                );
                self.native.text.insert(ids[3].into(), e.name.clone());
                if e.unread {
                    f.paint_rect(r, 0., f.theme().primary.opacity(0.035));
                }
                f.within(r, |f| {
                    [&rows::FOSSIFY, &rows::DELTACHAT, &rows::THUNDERBIRD][self.app].render(
                        f,
                        r,
                        &mut self.native,
                        10000 + i as u64 * 1000,
                    );
                });
                let target = f.target(9300 + i as u64, r, &e.name, true, Role::ListItem);
                f.state_layer(target, 0., f.theme().text);
                f.focus_ring(target);
                if target.clicked {
                    chosen = Some(i);
                }
            }
        });
        if let Some(i) = chosen {
            self.entries[i].unread = false;
            self.selected = Some(i);
            self.detail_offset = 0.;
            self.changed();
        }
    }
    fn plants(&mut self, f: &mut Frame<'_>, b: Rect) {
        let query = self.fields[0].to_lowercase();
        let indices: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                (self.folder == 1 || e.garden) && e.name.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        if indices.is_empty() {
            label(
                f,
                rect(b.x + 24., b.y + 48., b.width - 48., 48.),
                "Your garden is waiting",
                20.,
                true,
            );
            if button(
                f,
                9400,
                rect(b.x + 24., b.y + 112., b.width - 48., 48.),
                "Browse plants",
            ) {
                self.folder = 1;
            }
            return;
        }
        let cols = if b.width >= 680. { 3 } else { 2 };
        let cw = (b.width - 24.) / cols as f32;
        let rh = 170.;
        f.scroll_viewport(
            9401,
            b,
            &mut self.offset,
            ((indices.len().div_ceil(cols)) as f32 * rh + 24. - b.height).max(0.),
        );
        let offset = self.offset;
        f.within(b, |f| {
            for (n, &i) in indices.iter().enumerate() {
                let r = rect(
                    b.x + 12. + (n % cols) as f32 * cw + 6.,
                    b.y + 24. + (n / cols) as f32 * rh - offset,
                    cw - 12.,
                    144.,
                );
                if r.y + r.height < b.y || r.y > b.y + b.height {
                    continue;
                }
                self.native
                    .text
                    .insert("plant_item_title".into(), self.entries[i].name.clone());
                f.within(r, |f| {
                    rows::SUNFLOWER.render(f, r, &mut self.native, 11000 + i as u64);
                });
                let response = f.target(
                    9500 + i as u64,
                    r,
                    &self.entries[i].name,
                    true,
                    Role::Button,
                );
                f.state_layer(response, 12., f.theme().primary);
                if response.clicked {
                    self.selected = Some(i);
                    self.detail_offset = 0.;
                }
            }
        });
    }
    fn conversation(&mut self, f: &mut Frame<'_>, b: Rect, i: usize) {
        let area = rect(b.x, b.y, b.width, b.height - 88.);
        let lines: Vec<String> = self.entries[i].body.lines().map(str::to_owned).collect();
        let heights: Vec<f32> = lines
            .iter()
            .map(|s| {
                ((s.chars().count() as f32 / ((b.width - 96.) / 8.).max(10.)).ceil() * 22. + 32.)
                    .max(56.)
            })
            .collect();
        let total: f32 = heights.iter().sum::<f32>() + 48.;
        f.scroll_viewport(
            9600,
            area,
            &mut self.detail_offset,
            (total - area.height).max(0.),
        );
        let mut y = area.y + 24. - self.detail_offset;
        f.within(area, |f| {
            for (line, height) in lines.iter().zip(heights) {
                let own = line.starts_with("You:");
                let rw = (b.width - 72.).min(460.);
                let r = rect(
                    if own {
                        b.x + b.width - rw - 16.
                    } else {
                        b.x + 16.
                    },
                    y,
                    rw,
                    height - 8.,
                );
                if y + height >= area.y && y < area.y + area.height {
                    f.paint_rect(
                        r,
                        16.,
                        if own {
                            f.theme().primary_container
                        } else {
                            f.theme().surface_high
                        },
                    );
                    f.paragraph(
                        rect(r.x + 14., r.y + 12., r.width - 28., r.height - 20.),
                        line,
                        f.theme().text,
                        TextStyle {
                            size: 16.,
                            ..TextStyle::default()
                        },
                        22.,
                    );
                }
                y += height;
            }
        });
        f.paint_rect(
            rect(b.x, b.y + b.height - 88., b.width, 88.),
            0.,
            f.theme().surface_low,
        );
        self.field(
            f,
            1,
            rect(b.x + 12., b.y + b.height - 76., b.width - 84., 56.),
            "Message",
        );
        if icon(
            f,
            9601,
            rect(b.x + b.width - 64., b.y + b.height - 76., 52., 56.),
            "Send locally",
            MaterialIcon::ArrowForward,
        ) && !self.fields[1].trim().is_empty()
        {
            self.entries[i].body.push_str(&format!(
                "\nYou: {}",
                self.fields[1].trim().replace('\n', " ")
            ));
            let body = &mut self.entries[i].body;
            if body.len() > 2048 {
                let mut start = body.len() - 2048;
                while !body.is_char_boundary(start) {
                    start += 1;
                }
                *body = body[start..].to_owned();
            }
            self.entries[i].subject = prefix(&self.fields[1], 256);
            self.fields[1].clear();
            self.detail_offset = (self.entries[i]
                .body
                .lines()
                .map(|line| {
                    ((line.chars().count() as f32 / ((b.width - 96.) / 8.).max(10.)).ceil() * 22.
                        + 32.)
                        .max(56.)
                })
                .sum::<f32>()
                + 48.
                - area.height)
                .max(0.);
            self.changed();
        }
    }
    fn start_compose(&mut self, reply: Option<usize>) {
        self.compose = true;
        self.fields[1].clear();
        self.fields[2] = reply
            .map(|i| {
                format!(
                    "{}@example.test",
                    self.entries[i].name.to_lowercase().replace(' ', ".")
                )
            })
            .unwrap_or_else(|| "team@example.test".into());
        self.fields[3] = reply
            .map(|i| format!("Re: {}", self.entries[i].subject))
            .unwrap_or_else(|| "Hello from Bhoo".into());
    }
    fn compose_view(&mut self, f: &mut Frame<'_>, b: Rect) {
        self.field(f, 2, rect(b.x + 16., b.y + 12., b.width - 32., 56.), "To");
        self.field(
            f,
            3,
            rect(b.x + 16., b.y + 80., b.width - 32., 56.),
            "Subject",
        );
        self.field(
            f,
            1,
            rect(
                b.x + 16.,
                b.y + 148.,
                b.width - 32.,
                (b.height - 228.).max(56.),
            ),
            "Message body",
        );
        if button(
            f,
            9700,
            rect(b.x + 16., b.y + b.height - 64., b.width - 32., 48.),
            "Send locally",
        ) {
            if self.fields[1].trim().is_empty() || !self.fields[2].contains('@') {
                self.notice = "Enter a recipient and message first".into();
            } else {
                self.entries.insert(
                    0,
                    Entry {
                        name: self.fields[2].clone(),
                        subject: self.fields[3].clone(),
                        body: self.fields[1].clone(),
                        archived: false,
                        starred: false,
                        unread: false,
                        garden: false,
                    },
                );
                self.entries.truncate(32);
                self.fields[1].clear();
                self.compose = false;
                self.selected = None;
                self.folder = 0;
                self.offset = 0.;
                self.notice.clear();
                self.changed();
            }
        }
        if !self.notice.is_empty() {
            label(
                f,
                rect(b.x + 16., b.y + b.height - 94., b.width - 32., 24.),
                &self.notice,
                12.,
                false,
            );
        }
    }
    fn mail_detail(&mut self, f: &mut Frame<'_>, b: Rect, i: usize) {
        let e = self.entries[i].clone();
        let area = rect(b.x, b.y, b.width, b.height - 76.);
        f.scroll_viewport(
            9710,
            area,
            &mut self.detail_offset,
            (620. - area.height).max(0.),
        );
        f.within(area, |f| {
            let y = b.y - self.detail_offset;
            f.paragraph(
                rect(b.x + 24., y + 24., b.width - 48., 72.),
                &e.subject,
                f.theme().text,
                TextStyle {
                    size: 24.,
                    weight: 600,
                    ..TextStyle::default()
                },
                30.,
            );
            label(
                f,
                rect(b.x + 24., y + 104., b.width - 48., 28.),
                &format!("From: {}", e.name),
                14.,
                true,
            );
            label(
                f,
                rect(b.x + 24., y + 138., b.width - 48., 28.),
                "To: you@example.test · Today 10:42",
                12.,
                false,
            );
            f.paragraph(
                rect(b.x + 24., y + 190., b.width - 48., 400.),
                &e.body,
                f.theme().text,
                TextStyle {
                    size: 16.,
                    ..TextStyle::default()
                },
                25.,
            );
        });
        let bw = (b.width - 48.) / 3.;
        if button(
            f,
            9711,
            rect(b.x + 12., b.y + b.height - 60., bw, 44.),
            "Reply",
        ) {
            self.start_compose(Some(i));
        }
        if button(
            f,
            9712,
            rect(b.x + 24. + bw, b.y + b.height - 60., bw, 44.),
            if e.starred { "Unstar" } else { "Star" },
        ) {
            self.entries[i].starred = !e.starred;
            self.changed();
        }
        if button(
            f,
            9713,
            rect(b.x + 36. + bw * 2., b.y + b.height - 60., bw, 44.),
            if e.archived { "Restore" } else { "Archive" },
        ) {
            self.entries[i].archived = !e.archived;
            self.back();
            self.changed();
        }
    }
    fn plant_detail(&mut self, f: &mut Frame<'_>, b: Rect, i: usize) {
        let area = rect(b.x, b.y, b.width, b.height - 76.);
        f.scroll_viewport(
            9800,
            area,
            &mut self.detail_offset,
            (760. - area.height).max(0.),
        );
        let e = self.entries[i].clone();
        f.within(area, |f| {
            let y = b.y - self.detail_offset;
            f.paint_rect(rect(b.x, y, b.width, 220.), 0., Color(223, 237, 212, 255));
            f.paint_rect(
                rect(b.x + b.width / 2. - 3., y + 48., 6., 146.),
                3.,
                Color(65, 112, 58, 255),
            );
            for n in 0..4 {
                let left = n % 2 == 0;
                f.paint_rect(
                    rect(
                        b.x + b.width / 2. + if left { -70. } else { 0. },
                        y + 62. + n as f32 * 28.,
                        70.,
                        34.,
                    ),
                    25.,
                    Color(82, 133, 65, 255),
                );
            }
            label(
                f,
                rect(b.x + 24., y + 232., b.width - 48., 48.),
                &e.name,
                28.,
                true,
            );
            label(
                f,
                rect(b.x + 24., y + 290., b.width - 48., 32.),
                "Watering needs",
                18.,
                true,
            );
            label(
                f,
                rect(b.x + 24., y + 328., b.width - 48., 28.),
                "Every 7 days · sample schedule",
                14.,
                false,
            );
            f.paragraph(
                rect(b.x + 24., y + 382., b.width - 48., 320.),
                &e.body,
                f.theme().text,
                TextStyle {
                    size: 16.,
                    ..TextStyle::default()
                },
                25.,
            );
        });
        if button(
            f,
            9801,
            rect(b.x + 16., b.y + b.height - 60., b.width - 32., 48.),
            if e.garden {
                "Remove from garden"
            } else {
                "Add to garden"
            },
        ) {
            self.entries[i].garden = !e.garden;
            self.changed();
        }
    }
    pub fn save(&self) -> String {
        let mut out = format!("BHOO1\t{}\n", self.app);
        for e in &self.entries {
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                hex(&e.name),
                hex(&e.subject),
                hex(&e.body),
                u8::from(e.archived)
                    | (u8::from(e.starred) << 1)
                    | (u8::from(e.unread) << 2)
                    | (u8::from(e.garden) << 3)
            ));
        }
        out.push_str(&format!(
            "DRAFT\t{}\t{}\t{}\n",
            hex(&self.fields[1]),
            hex(&self.fields[2]),
            hex(&self.fields[3])
        ));
        out
    }
    pub fn restore(&mut self, app: usize, s: &str) -> bool {
        if !s.starts_with(&format!("BHOO1\t{app}\n")) {
            return false;
        }
        let mut entries = Vec::new();
        let mut drafts = None;
        for line in s.lines().skip(1) {
            let p: Vec<&str> = line.split('\t').collect();
            if p.len() != 4 {
                return false;
            }
            let Some(a) = unhex(p[1]) else { return false };
            let Some(b) = unhex(p[2]) else { return false };
            if p[0] == "DRAFT" {
                let Some(c) = unhex(p[3]) else { return false };
                if a.len() > 2048 || b.len() > 128 || c.len() > 256 {
                    return false;
                }
                drafts = Some([a, b, c]);
                continue;
            }
            let Some(name) = unhex(p[0]) else {
                return false;
            };
            let Ok(bits) = p[3].parse::<u8>() else {
                return false;
            };
            if entries.len() >= 32
                || name.len() > 128
                || a.len() > 256
                || b.len() > 2048
                || bits > 15
            {
                return false;
            }
            entries.push(Entry {
                name,
                subject: a,
                body: b,
                archived: bits & 1 != 0,
                starred: bits & 2 != 0,
                unread: bits & 4 != 0,
                garden: bits & 8 != 0,
            });
        }
        if entries.is_empty() {
            return false;
        }
        self.entries = entries;
        self.app = app;
        self.initialized = true;
        if let Some(d) = drafts {
            self.fields[1] = d[0].clone();
            self.fields[2] = d[1].clone();
            self.fields[3] = d[2].clone();
        }
        true
    }
}
fn prefix(s: &str, max: usize) -> String {
    let mut end = s.len().min(max);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}
fn hex(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(s: &str) -> Option<String> {
    if s.len() > 32768 || !s.len().is_multiple_of(2) || !s.is_ascii() {
        return None;
    }
    String::from_utf8(
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
            .collect::<Option<Vec<_>>>()?,
    )
    .ok()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_constraint_rows_keep_avatar_and_labels_visible() {
        let mut ui = Ui::default();
        let mut commands = Vec::new();
        let mut state = NativeState::default();
        for (id, value) in [
            ("conversation_address", "Alex Morgan"),
            ("conversation_body_short", "A short preview"),
            ("conversation_date", "10:42"),
            ("conversation_image", "Alex Morgan"),
        ] {
            state.text.insert(id.into(), value.into());
        }
        ui.frame_into(
            Input::default(),
            rect(0., 0., 390., 76.),
            0.,
            &mut commands,
            |f| {
                rows::FOSSIFY.render(f, rect(0., 0., 390., 76.), &mut state, 1);
            },
        );
        assert_eq!(state.slots["conversation_image"].height, 48.);
        assert!(state.slots["conversation_address"].height >= 20.);
        assert!(state.slots["conversation_body_short"].y > state.slots["conversation_address"].y);
        assert!(state.slots["conversation_address"].x >= state.slots["conversation_image"].x + 48.);
    }
    #[test]
    fn form_limits_preserve_utf8_and_fit_snapshot_buffer() {
        let mut a = AndroidApps::default();
        a.init(2);
        for index in 0..4 {
            a.editing = index;
            a.edit(&"👻".repeat(4096));
            assert!(a.fields[index].len() <= [128, 2048, 128, 256][index]);
        }
        let mut e = a.entries[0].clone();
        e.name = a.fields[2].clone();
        e.subject = a.fields[3].clone();
        e.body = a.fields[1].clone();
        a.entries = vec![e; 32];
        let s = a.save();
        assert!(s.len() < 262144);
        let mut b = AndroidApps::default();
        assert!(b.restore(2, &s));
        let before = b.save();
        assert!(!b.restore(2, &(s + "bad\n")));
        assert_eq!(b.save(), before);
    }
    #[test]
    fn persistence_is_lossless_and_app_scoped() {
        for app in 0..4 {
            let mut a = AndroidApps::default();
            a.init(app);
            a.entries[0].body = "hello\n世界\t👻".into();
            a.entries[0].starred = true;
            a.fields[1] = "a draft".into();
            let s = a.save();
            let mut b = AndroidApps::default();
            assert!(b.restore(app, &s));
            assert_eq!(b.save(), s);
            assert!(!b.restore((app + 1) % 4, &s));
            assert!(!b.restore(app, &format!("BHOO1\t{app}\nbroken")));
        }
    }
}
