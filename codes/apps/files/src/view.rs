use crate::browser::{
    Action, Browser, Entry, Filter, Location, Modal, Order, PickerRequest, file_name,
};
use bhoo::{Color, Frame, ImageFit, Key, MaterialIcon, Rect, Role, SystemIcon, TextStyle};
use std::path::Path;

pub const SEARCH_FIELD_ID: u64 = 0x4649_4c45_0000_0001;
pub const EDIT_FIELD_ID: u64 = 0x4649_4c45_0000_0002;
const BASE: u64 = 0x4649_4c45_1000_0000;

#[derive(Clone, Copy)]
pub struct Appearance {
    pub background: Color,
    pub surface: Color,
    pub raised: Color,
    pub text: Color,
    pub muted: Color,
    pub primary: Color,
    pub selected: Color,
    pub on_primary: Color,
    pub outline: Color,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            background: Color(247, 248, 250, 255),
            surface: Color(255, 255, 255, 255),
            raised: Color(233, 237, 244, 255),
            text: Color(28, 34, 44, 255),
            muted: Color(89, 100, 116, 255),
            primary: Color(43, 78, 144, 255),
            selected: Color(218, 229, 253, 255),
            on_primary: Color(255, 255, 255, 255),
            outline: Color(205, 215, 231, 255),
        }
    }
}

fn style(size: f32, weight: u16, align: u8) -> TextStyle {
    TextStyle {
        size,
        weight,
        align,
        padding: 0.0,
        ..TextStyle::default()
    }
}

fn label(f: &mut Frame<'_>, rect: Rect, text: &str, color: Color, size: f32, weight: u16) {
    f.styled_text(rect, text, color, style(size, weight, 0));
}

fn icon_button(
    f: &mut Frame<'_>,
    id: u64,
    rect: Rect,
    text: &str,
    icon: MaterialIcon,
    p: Appearance,
) -> bool {
    let response = f.target(id, rect, text, true, Role::Button);
    f.state_layer(response, rect.height * 0.5, p.primary);
    f.paint_material_icon(rect.inset((rect.height - 21.0) * 0.5), icon, p.text);
    response.clicked
}

fn button(
    f: &mut Frame<'_>,
    id: u64,
    rect: Rect,
    text: &str,
    p: Appearance,
    primary: bool,
) -> bool {
    let response = f.target(id, rect, text, true, Role::Button);
    let (background, ink) = if primary {
        (p.primary, p.on_primary)
    } else {
        (p.selected, p.primary)
    };
    f.paint_rect(rect, rect.height * 0.5, background);
    f.state_layer(response, rect.height * 0.5, ink);
    f.styled_text(rect, text, ink, style(13.0, 600, 1));
    response.clicked
}

fn text_field(
    f: &mut Frame<'_>,
    id: u64,
    rect: Rect,
    hint: &str,
    value: &str,
    p: Appearance,
) -> bool {
    let response = f.target(id, rect, hint, true, Role::TextField);
    f.paint_rect(rect, 14.0, p.raised);
    let text_bounds = Rect {
        x: rect.x + 13.0,
        width: rect.width - 26.0,
        ..rect
    };
    label(
        f,
        text_bounds,
        if value.is_empty() { hint } else { value },
        if value.is_empty() { p.muted } else { p.text },
        15.0,
        400,
    );
    if response.focused {
        f.paint_rect(
            Rect {
                y: rect.y + rect.height - 2.0,
                height: 2.0,
                ..rect
            },
            1.0,
            p.primary,
        );
    }
    response.clicked
}

fn bytes(size: u64) -> String {
    for (scale, units) in [(1_073_741_824, "GB"), (1_048_576, "MB"), (1024, "KB")] {
        if size >= scale {
            return format!("{:.1} {units}", size as f64 / scale as f64);
        }
    }
    format!("{size} B")
}

fn paint_folder(f: &mut Frame<'_>, r: Rect, color: Color) {
    f.paint_rect(
        Rect {
            x: r.x,
            y: r.y + r.height * 0.2,
            width: r.width * 0.52,
            height: r.height * 0.28,
        },
        2.0,
        color,
    );
    f.paint_rect(
        Rect {
            y: r.y + r.height * 0.34,
            height: r.height * 0.49,
            ..r
        },
        2.0,
        color,
    );
}

fn file_icon(f: &mut Frame<'_>, rect: Rect, entry: &Entry, p: Appearance) {
    if entry.directory {
        paint_folder(f, rect, p.primary);
    } else {
        let icon = if Filter::Images.accepts(&entry.path) {
            MaterialIcon::Image
        } else if Filter::Archives.accepts(&entry.path) {
            MaterialIcon::Archive
        } else if Filter::Videos.accepts(&entry.path) {
            MaterialIcon::Videocam
        } else {
            MaterialIcon::Description
        };
        f.paint_material_icon(rect, icon, p.primary);
    }
}

#[derive(Clone, Copy)]
enum MenuAction {
    NewFolder,
    NewWindow,
    Paste,
    Selection,
    SelectAll,
    Hidden,
    Sort(Order),
    Reverse,
    Refresh,
    Rename,
    Copy,
    Cut,
    Delete,
    Restore,
    Details,
    Open,
    Share,
    Compress,
    Extract,
}

impl Browser {
    fn accepts_editor_input(&self) -> bool {
        self.search_open
            || matches!(self.request, Some(PickerRequest::Save { .. }))
            || matches!(
                self.modal,
                Some(Modal::NewFolder | Modal::Rename(_) | Modal::Compress(_))
            )
    }

    pub fn show(&mut self, f: &mut Frame<'_>, outer: Rect, appearance: Appearance) -> Action {
        self.poll();
        let editing = self.accepts_editor_input();
        if !self.idle() {
            f.continuous(outer);
        }
        let mut action = Action::None;
        f.within(outer, |f| {
            f.paint_rect(outer, 0.0, appearance.background);
            let blocked = self.nav_open || self.menu_open || self.modal.is_some();
            f.without_input(blocked, |f| {
                action = self.paint_body(f, outer, appearance);
            });
            if self.nav_open {
                self.paint_places(f, outer, appearance);
            }
            if self.menu_open {
                action = self.paint_menu(f, outer, appearance);
            }
            if let Some(modal) = self.modal.clone() {
                action = self.paint_modal(f, outer, appearance, modal);
            }
        });
        if !self.idle() {
            f.continuous(outer);
        }
        if action == Action::None && editing && !self.accepts_editor_input() {
            Action::Blur
        } else {
            action
        }
    }

    fn edit_key(&mut self, f: &Frame<'_>, id: u64) {
        if f.focused_id() != Some(id) {
            return;
        }
        let Some(key) = f.input().key.filter(|key| key.pressed) else {
            return;
        };
        let value = self.edit_text(id).unwrap_or_default();
        let mut edit = value.to_owned();
        match key.key {
            Key::Character('\u{8}' | '\u{7f}') => {
                edit.pop();
            }
            Key::Character(character) if !character.is_control() => edit.push(character),
            Key::Space => edit.push(' '),
            _ => return,
        }
        self.set_edit_text(id, &edit);
    }

    fn paint_body(&mut self, f: &mut Frame<'_>, b: Rect, p: Appearance) -> Action {
        let selection = self.selection.len();
        let is_picker = self.request.is_some();
        let title = if selection > 0 {
            format!("{selection} selected")
        } else {
            match &self.request {
                Some(PickerRequest::Open {
                    filter: Filter::Images,
                    ..
                }) => "Choose an image".to_owned(),
                Some(PickerRequest::Open { .. }) => "Open files".to_owned(),
                Some(PickerRequest::Folder) => "Choose a folder".to_owned(),
                Some(PickerRequest::Save { .. }) => "Save file".to_owned(),
                None => "Files".to_owned(),
            }
        };
        let left = Rect {
            x: b.x + 10.0,
            y: b.y + 6.0,
            width: 42.0,
            height: 42.0,
        };
        if selection > 0 || self.select_mode {
            if icon_button(
                f,
                BASE + 10,
                left,
                "Clear selection",
                MaterialIcon::Close,
                p,
            ) {
                self.selection.clear();
                self.select_mode = false;
            }
        } else if icon_button(
            f,
            BASE + 10,
            left,
            "Storage locations",
            MaterialIcon::Menu,
            p,
        ) {
            self.nav_open = true;
        }
        let title_width = (b.width - 201.0).max(1.0);
        let title_size = 21.0
            * (title_width / f.text_width(&title, style(21.0, 600, 0)).max(1.0)).clamp(0.70, 1.0);
        label(
            f,
            Rect {
                x: b.x + 60.0,
                y: b.y + 5.0,
                width: title_width,
                height: 44.0,
            },
            &title,
            p.text,
            title_size,
            600,
        );
        let action_button = |offset: f32| Rect {
            x: b.x + b.width - offset,
            y: b.y + 6.0,
            width: 42.0,
            height: 42.0,
        };
        if icon_button(
            f,
            BASE + 11,
            action_button(50.0),
            "More options",
            MaterialIcon::MoreVert,
            p,
        ) {
            self.menu_open = true;
        }
        if icon_button(
            f,
            BASE + 12,
            action_button(94.0),
            "Search",
            MaterialIcon::Search,
            p,
        ) {
            self.search_open = !self.search_open;
            if !self.search_open {
                self.set_edit_text(SEARCH_FIELD_ID, "");
            }
        }
        let view = action_button(138.0);
        if icon_button(
            f,
            BASE + 13,
            view,
            if self.grid { "List view" } else { "Grid view" },
            if self.grid {
                MaterialIcon::Menu
            } else {
                MaterialIcon::Widgets
            },
            p,
        ) {
            self.grid = !self.grid;
            self.scroll = 0.0;
        }

        let mut action = Action::None;
        let mut y = b.y + 54.0;
        if self.search_open {
            self.edit_key(f, SEARCH_FIELD_ID);
            let rect = Rect {
                x: b.x + 14.0,
                y,
                width: b.width - 28.0,
                height: 46.0,
            };
            if text_field(
                f,
                SEARCH_FIELD_ID,
                rect,
                "Search this location",
                &self.search,
                p,
            ) {
                action = Action::Focus(SEARCH_FIELD_ID);
            }
            y += 54.0;
        }

        let location_name = match &self.location {
            Location::Recent => "Recent".to_owned(),
            Location::Category(kind) => match kind {
                Filter::Images => "Images",
                Filter::Videos => "Videos",
                Filter::Audio => "Audio",
                _ => "Documents",
            }
            .to_owned(),
            Location::Directory(path) if path == &self.home => "Internal storage".to_owned(),
            Location::Directory(path) => file_name(path),
            Location::Trash => "Trash".to_owned(),
        };
        let location_left =
            if let Some(parent) = self.directory().and_then(Path::parent).map(Path::to_owned) {
                let up = Rect {
                    x: b.x + 10.0,
                    y,
                    width: 34.0,
                    height: 34.0,
                };
                if icon_button(
                    f,
                    BASE + 14,
                    up,
                    "Parent folder",
                    MaterialIcon::ArrowBack,
                    p,
                ) {
                    self.go(Location::Directory(parent));
                }
                b.x + 48.0
            } else {
                b.x + 16.0
            };
        label(
            f,
            Rect {
                x: location_left,
                y,
                width: b.x + b.width - location_left - 14.0,
                height: 34.0,
            },
            &location_name,
            p.muted,
            14.0,
            600,
        );
        y += 38.0;

        if !is_picker && matches!(self.location, Location::Directory(_) | Location::Recent) {
            let width = (b.width - 48.0) * 0.25;
            for (index, (text, filter)) in [
                ("All", Filter::Any),
                ("Images", Filter::Images),
                ("Videos", Filter::Videos),
                ("Audio", Filter::Audio),
            ]
            .into_iter()
            .enumerate()
            {
                let rect = Rect {
                    x: b.x + 12.0 + index as f32 * (width + 8.0),
                    y,
                    width,
                    height: 34.0,
                };
                if button(
                    f,
                    BASE + 20 + index as u64,
                    rect,
                    text,
                    p,
                    self.filter == filter,
                ) {
                    self.filter = filter;
                    self.scroll = 0.0;
                    self.reload();
                }
            }
            y += 44.0;
        }

        let mut footer_height = if self.message.is_some() { 36.0 } else { 0.0 };
        if is_picker || self.can_paste() {
            footer_height += if matches!(self.request, Some(PickerRequest::Save { .. })) {
                111.0
            } else {
                60.0
            };
        }
        let list = Rect {
            x: b.x + 10.0,
            y,
            width: b.width - 20.0,
            height: (b.y + b.height - y - footer_height).max(0.0),
        };
        if let Some(entry) = self.paint_entries(f, list, p) {
            action = self.pick(entry);
        }

        let mut foot_y = list.y + list.height;
        if let Some(message) = &self.message {
            let message_rect = Rect {
                x: b.x + 14.0,
                y: foot_y,
                width: b.width - 28.0,
                height: 34.0,
            };
            label(f, message_rect, message, p.muted, 12.0, 500);
            foot_y += 36.0;
        }
        if is_picker {
            if matches!(self.request, Some(PickerRequest::Save { .. })) {
                self.edit_key(f, EDIT_FIELD_ID);
                let rect = Rect {
                    x: b.x + 14.0,
                    y: foot_y,
                    width: b.width - 28.0,
                    height: 46.0,
                };
                if text_field(f, EDIT_FIELD_ID, rect, "File name", &self.save_name, p) {
                    action = Action::Focus(EDIT_FIELD_ID);
                }
                foot_y += 51.0;
            }
            let cancel = Rect {
                x: b.x + 14.0,
                y: foot_y + 8.0,
                width: 102.0,
                height: 40.0,
            };
            if button(f, BASE + 31, cancel, "Cancel", p, false) {
                return Action::Cancelled;
            }
            let confirm = Rect {
                x: b.x + b.width - 142.0,
                width: 128.0,
                ..cancel
            };
            let confirm_label = match self.request {
                Some(PickerRequest::Save { .. }) if self.directory().is_some() => Some("Save"),
                Some(PickerRequest::Folder) if self.directory().is_some() => {
                    Some("Use this folder")
                }
                Some(PickerRequest::Open { .. }) if !self.selection.is_empty() => Some("Open"),
                _ => None,
            };
            if confirm_label.is_some_and(|text| button(f, BASE + 32, confirm, text, p, true)) {
                return self.confirm_picker();
            }
        } else if self.can_paste() {
            let paste = Rect {
                x: b.x + b.width - 152.0,
                y: foot_y + 8.0,
                width: 138.0,
                height: 40.0,
            };
            if button(f, BASE + 33, paste, "Paste here", p, true) {
                self.paste();
            }
        }
        action
    }

    fn paint_entries(&mut self, f: &mut Frame<'_>, area: Rect, p: Appearance) -> Option<Entry> {
        if self.entries.is_empty() {
            let message = if self.is_loading() {
                "Loading files…"
            } else if self.search.is_empty() {
                "No files here"
            } else {
                "No matching files"
            };
            f.styled_text(
                Rect {
                    y: area.y + 40.0,
                    height: 34.0,
                    ..area
                },
                message,
                p.muted,
                style(15.0, 500, 1),
            );
            return None;
        }
        let columns = if self.grid {
            ((area.width / 124.0) as usize).clamp(2, 6)
        } else {
            1
        };
        let gap = 8.0;
        let pitch = if self.grid { 132.0 } else { 60.0 };
        let width = (area.width - (columns - 1) as f32 * gap) / columns as f32;
        let rows = self.entries.len().div_ceil(columns);
        let max = (rows as f32 * pitch - area.height).max(0.0);
        self.scroll = self.scroll.clamp(0.0, max);
        f.scroll_viewport(BASE + 40, area, &mut self.scroll, max);
        let first = (self.scroll / pitch).floor() as usize * columns;
        let count = ((area.height / pitch).ceil() as usize + 2) * columns;
        let mut selected = None;
        let mut toggle = None;
        let mut previews = Vec::new();
        f.within(area, |f| {
            for (index, entry) in self.entries.iter().enumerate().skip(first).take(count) {
                let row = index / columns;
                let column = index % columns;
                let rect = Rect {
                    x: area.x + column as f32 * (width + gap),
                    y: area.y + row as f32 * pitch - self.scroll,
                    width,
                    height: pitch - 5.0,
                };
                let checked = self.selection.contains(&entry.path);
                let selectable = self.request.is_none()
                    || (self.multiple() && !entry.directory && self.allowed(entry));
                let row_id = BASE + 0x1000 + index as u64 * 2;
                let select_rect = Rect {
                    x: rect.x + rect.width - 40.0,
                    y: rect.y + 6.0,
                    width: 36.0,
                    height: 36.0,
                };
                if selectable {
                    let r_select =
                        f.target(row_id + 1, select_rect, "Select item", true, Role::Checkbox);
                    if r_select.clicked {
                        toggle = Some(entry.path.clone());
                    }
                }
                let row_response =
                    f.target(row_id, rect, &entry.name, self.allowed(entry), Role::Button);
                f.paint_rect(rect, 12.0, if checked { p.selected } else { p.surface });
                f.state_layer(row_response, 12.0, p.primary);
                let picture = if self.grid {
                    Rect {
                        x: rect.x + 8.0,
                        y: rect.y + 7.0,
                        width: rect.width - 16.0,
                        height: 78.0,
                    }
                } else {
                    Rect {
                        x: rect.x + 10.0,
                        y: rect.y + 8.0,
                        width: 38.0,
                        height: 38.0,
                    }
                };
                if let Some(image) = self.pictures.get(&entry.path) {
                    f.paint_image(picture, image, ImageFit::Cover, 8.0);
                } else {
                    let thumb = !entry.directory && Filter::Images.accepts(&entry.path);
                    if thumb {
                        previews.push(entry.path.clone());
                    }
                    let side = if self.grid { 41.0 } else { 26.0 };
                    let icon_rect = Rect {
                        x: picture.x + (picture.width - side) * 0.5,
                        y: picture.y + (picture.height - side) * 0.5,
                        width: side,
                        height: side,
                    };
                    file_icon(f, icon_rect, entry, p);
                }
                let title_x = if self.grid {
                    rect.x + 8.0
                } else {
                    picture.x + picture.width + 10.0
                };
                let title_y = if self.grid {
                    picture.y + picture.height + 3.0
                } else {
                    rect.y + 3.0
                };
                let title_w =
                    (rect.x + rect.width - title_x - if self.grid { 8.0 } else { 42.0 }).max(10.0);
                let name = Rect {
                    x: title_x,
                    y: title_y,
                    width: title_w,
                    height: 25.0,
                };
                label(f, name, &entry.name, p.text, 14.0, 500);
                label(
                    f,
                    Rect {
                        y: name.y + 23.0,
                        height: 16.0,
                        ..name
                    },
                    &if entry.directory {
                        "Folder".into()
                    } else {
                        bytes(entry.bytes)
                    },
                    p.muted,
                    11.0,
                    400,
                );
                if selectable {
                    f.paint_material_icon(
                        select_rect.inset(9.0),
                        if checked {
                            MaterialIcon::Check
                        } else {
                            MaterialIcon::MoreHoriz
                        },
                        if checked { p.primary } else { p.muted },
                    );
                }
                if row_response.clicked {
                    selected = Some(entry.clone());
                }
            }
        });
        self.thumbnails(previews);
        if let Some(path) = toggle {
            self.toggle(path);
            return None;
        }
        selected
    }

    fn paint_places(&mut self, f: &mut Frame<'_>, b: Rect, p: Appearance) {
        f.paint_rect(b, 0.0, Color(0, 0, 0, 100));
        let sheet = Rect {
            width: (b.width - 40.0).min(312.0),
            ..b
        };
        let dismiss = Rect {
            x: sheet.x + sheet.width,
            width: b.width - sheet.width,
            ..b
        };
        if f.target(BASE + 50, dismiss, "Close locations", true, Role::Button)
            .clicked
        {
            self.nav_open = false;
        }
        f.paint_rect(sheet, 18.0, p.surface);
        label(
            f,
            Rect {
                x: sheet.x + 20.0,
                y: sheet.y + 8.0,
                width: sheet.width - 38.0,
                height: 42.0,
            },
            "Locations",
            p.text,
            20.0,
            600,
        );
        let mut next = None;
        for (index, (name, location)) in self.roots.iter().enumerate() {
            if self.request.is_some() && matches!(location, Location::Trash) {
                continue;
            }
            if matches!(
                self.request,
                Some(PickerRequest::Save { .. } | PickerRequest::Folder)
            ) && !matches!(location, Location::Directory(_))
            {
                continue;
            }
            let rect = Rect {
                x: sheet.x + 10.0,
                y: sheet.y + 54.0 + index as f32 * 47.0,
                width: sheet.width - 20.0,
                height: 43.0,
            };
            let r = f.target(BASE + 100 + index as u64, rect, name, true, Role::Button);
            if &self.location == location {
                f.paint_rect(rect, 12.0, p.selected);
            }
            f.state_layer(r, 12.0, p.primary);
            let icon = match location {
                Location::Recent => SystemIcon::History,
                Location::Category(Filter::Images) => SystemIcon::Camera,
                Location::Trash => SystemIcon::Trash,
                _ => SystemIcon::Storage,
            };
            f.paint_system_icon(
                Rect {
                    x: rect.x + 10.0,
                    y: rect.y + 12.0,
                    width: 20.0,
                    height: 20.0,
                },
                icon,
                p.primary,
            );
            label(
                f,
                Rect {
                    x: rect.x + 40.0,
                    width: rect.width - 48.0,
                    ..rect
                },
                name,
                p.text,
                15.0,
                500,
            );
            if r.clicked {
                next = Some(location.clone());
            }
        }
        if let Some(location) = next {
            self.root(location);
        }
    }

    fn paint_menu(&mut self, f: &mut Frame<'_>, b: Rect, p: Appearance) -> Action {
        let selected = self.selected();
        let mut options = Vec::new();
        if selected.is_empty() {
            if self.directory().is_some() {
                options.push(("New folder", MenuAction::NewFolder));
            }
            if self.can_paste() {
                options.push(("Paste here", MenuAction::Paste));
            }
            if self.request.is_none() {
                options.push(("New window", MenuAction::NewWindow));
            }
            if self.multiple() {
                options.push(("Select items", MenuAction::Selection));
            }
            options.extend([
                (
                    if self.hidden {
                        "Hide hidden files"
                    } else {
                        "Show hidden files"
                    },
                    MenuAction::Hidden,
                ),
                ("Sort by name", MenuAction::Sort(Order::Name)),
                ("Sort by date", MenuAction::Sort(Order::Modified)),
                ("Sort by size", MenuAction::Sort(Order::Size)),
                ("Sort by type", MenuAction::Sort(Order::Kind)),
                (
                    if self.descending {
                        "Ascending"
                    } else {
                        "Descending"
                    },
                    MenuAction::Reverse,
                ),
                ("Refresh", MenuAction::Refresh),
            ]);
        } else if self.request.is_some() {
            options.push(("Select all", MenuAction::SelectAll));
        } else if self.location == Location::Trash {
            options.extend([
                ("Restore", MenuAction::Restore),
                ("Delete permanently", MenuAction::Delete),
                ("Select all", MenuAction::SelectAll),
            ]);
        } else {
            if selected.len() == 1 {
                options.extend([
                    ("Open", MenuAction::Open),
                    ("Rename", MenuAction::Rename),
                    ("Details", MenuAction::Details),
                ]);
            }
            options.extend([
                ("Copy", MenuAction::Copy),
                ("Cut or move", MenuAction::Cut),
                ("Move to Trash", MenuAction::Delete),
                ("Compress as ZIP", MenuAction::Compress),
            ]);
            if selected.iter().all(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
            }) {
                options.push(("Extract ZIP", MenuAction::Extract));
            }
            if selected.iter().all(|path| !path.is_dir()) {
                options.push(("Share", MenuAction::Share));
            }
            options.push(("Select all", MenuAction::SelectAll));
        }
        let mut choice = None;
        let panel = Rect {
            x: b.x + b.width - 249.0,
            y: b.y + 48.0,
            width: 237.0,
            height: 12.0 + options.len() as f32 * 43.0,
        };
        f.paint_rect(panel, 16.0, p.surface);
        for (index, (text, item)) in options.into_iter().enumerate() {
            let rect = Rect {
                x: panel.x + 5.0,
                y: panel.y + 6.0 + index as f32 * 43.0,
                width: panel.width - 10.0,
                height: 42.0,
            };
            let response = f.target(BASE + 190 + index as u64, rect, text, true, Role::Button);
            f.state_layer(response, 10.0, p.primary);
            label(
                f,
                Rect {
                    x: rect.x + 12.0,
                    width: rect.width - 24.0,
                    ..rect
                },
                text,
                p.text,
                14.0,
                500,
            );
            if response.clicked {
                choice = Some(item);
            }
        }
        if f.target(BASE + 220, b, "Close menu", true, Role::Button)
            .clicked
        {
            self.menu_open = false;
        }
        let Some(choice) = choice else {
            return Action::None;
        };
        self.menu_open = false;
        match choice {
            MenuAction::NewFolder => self.begin_modal(Modal::NewFolder),
            MenuAction::NewWindow => {
                return Action::NewWindow(self.directory().map(Path::to_owned));
            }
            MenuAction::Paste => self.paste(),
            MenuAction::Selection => self.select_mode = true,
            MenuAction::SelectAll => self.select_all(),
            MenuAction::Hidden => {
                self.hidden = !self.hidden;
                self.reload();
            }
            MenuAction::Sort(order) => {
                self.order = order;
                self.descending = order == Order::Modified;
                self.sort();
            }
            MenuAction::Reverse => {
                self.descending = !self.descending;
                self.sort();
            }
            MenuAction::Refresh => self.reload(),
            MenuAction::Rename => self.begin_modal(Modal::Rename(selected[0].clone())),
            MenuAction::Copy => self.set_clipboard(false),
            MenuAction::Cut => self.set_clipboard(true),
            MenuAction::Delete => self.begin_modal(Modal::Delete(selected)),
            MenuAction::Restore => self.restore(),
            MenuAction::Details => {
                if let Some(entry) = self
                    .entries
                    .iter()
                    .find(|entry| entry.path == selected[0])
                    .cloned()
                {
                    self.begin_modal(Modal::Details(entry));
                }
            }
            MenuAction::Open => {
                if selected[0].is_dir() {
                    self.go(Location::Directory(selected[0].clone()));
                } else {
                    return Action::Open(selected[0].clone());
                }
            }
            MenuAction::Share => return Action::Share(selected),
            MenuAction::Compress => self.begin_modal(Modal::Compress(selected)),
            MenuAction::Extract => self.extract(),
        }
        Action::None
    }

    fn paint_modal(&mut self, f: &mut Frame<'_>, b: Rect, p: Appearance, modal: Modal) -> Action {
        f.paint_rect(b, 0.0, Color(0, 0, 0, 110));
        let title = match &modal {
            Modal::NewFolder => "New folder",
            Modal::Rename(_) => "Rename",
            Modal::Replace(_) => "Replace existing file?",
            Modal::Compress(_) => "Compress as ZIP",
            Modal::Delete(_) if self.location == Location::Trash => "Delete permanently?",
            Modal::Delete(_) => "Move to Trash?",
            Modal::Details(_) => "File details",
            Modal::Preview(_) => "Image preview",
        };
        let picture = matches!(modal, Modal::Preview(_));
        let height = if picture {
            (b.height * 0.72).min(520.0)
        } else {
            224.0
        };
        let panel = Rect {
            x: b.x + 16.0,
            y: b.y + (b.height - height).max(0.0) * 0.32,
            width: b.width - 32.0,
            height,
        };
        f.paint_rect(panel, 22.0, p.surface);
        label(
            f,
            Rect {
                x: panel.x + 16.0,
                y: panel.y + 12.0,
                width: panel.width - 32.0,
                height: 35.0,
            },
            title,
            p.text,
            19.0,
            600,
        );
        let inner = Rect {
            x: panel.x + 16.0,
            y: panel.y + 58.0,
            width: panel.width - 32.0,
            height: 50.0,
        };
        let mut action = Action::None;
        match &modal {
            Modal::Preview(entry) => {
                self.thumbnails(vec![entry.path.clone()]);
                let image_area = Rect {
                    height: panel.height - 143.0,
                    ..inner
                };
                if let Some(image) = self.pictures.get(&entry.path) {
                    f.paint_image(image_area, image, ImageFit::Contain, 8.0);
                }
                label(
                    f,
                    Rect {
                        y: image_area.y + image_area.height + 5.0,
                        height: 25.0,
                        ..inner
                    },
                    &entry.name,
                    p.text,
                    13.0,
                    500,
                );
            }
            Modal::Details(entry) => {
                label(
                    f,
                    Rect {
                        height: 26.0,
                        ..inner
                    },
                    &entry.name,
                    p.text,
                    15.0,
                    600,
                );
                let summary = if entry.directory {
                    "Folder".into()
                } else {
                    bytes(entry.bytes)
                };
                let date = entry.modified.map(format_date).unwrap_or_default();
                label(
                    f,
                    Rect {
                        y: inner.y + 28.0,
                        height: 26.0,
                        ..inner
                    },
                    &format!("{summary}  {date}"),
                    p.muted,
                    13.0,
                    400,
                );
                label(
                    f,
                    Rect {
                        y: inner.y + 55.0,
                        height: 26.0,
                        ..inner
                    },
                    &entry.path.to_string_lossy(),
                    p.muted,
                    12.0,
                    400,
                );
            }
            Modal::Replace(path) => {
                label(f, inner, &file_name(path), p.text, 15.0, 600);
                label(
                    f,
                    Rect {
                        y: inner.y + 38.0,
                        height: 30.0,
                        ..inner
                    },
                    "Saving here will replace the file’s contents.",
                    p.muted,
                    13.0,
                    400,
                );
            }
            Modal::Delete(paths) => {
                label(
                    f,
                    inner,
                    &format!("{} selected item(s)", paths.len()),
                    p.muted,
                    14.0,
                    500,
                );
            }
            _ => {
                self.edit_key(f, EDIT_FIELD_ID);
                if text_field(f, EDIT_FIELD_ID, inner, "Name", &self.edit, p) {
                    action = Action::Focus(EDIT_FIELD_ID);
                }
            }
        }
        let cancel = Rect {
            x: panel.x + 16.0,
            y: panel.y + panel.height - 52.0,
            width: 100.0,
            height: 38.0,
        };
        if button(
            f,
            BASE + 231,
            cancel,
            if matches!(modal, Modal::Preview(_) | Modal::Details(_)) {
                "Close"
            } else {
                "Cancel"
            },
            p,
            false,
        ) {
            self.modal = None;
        }
        let ok = Rect {
            x: panel.x + panel.width - 118.0,
            width: 102.0,
            ..cancel
        };
        match modal {
            Modal::Details(_) => {}
            Modal::Replace(ref path) => {
                if button(f, BASE + 232, ok, "Replace", p, true) {
                    self.modal = None;
                    #[cfg(not(target_arch = "wasm32"))]
                    let still_file =
                        std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_file());
                    #[cfg(target_arch = "wasm32")]
                    let still_file = crate::wasm_fs::is_file(path);
                    if still_file {
                        return Action::Selected(vec![path.clone()]);
                    }
                    self.report("The file changed. Choose it again.");
                }
            }
            Modal::Preview(ref entry) => {
                if button(f, BASE + 232, ok, "Open", p, true) {
                    self.modal = None;
                    return Action::Open(entry.path.clone());
                }
            }
            _ => {
                let submit = f.focused_id() == Some(EDIT_FIELD_ID) && f.pressed_key(Key::Enter);
                if button(
                    f,
                    BASE + 232,
                    ok,
                    if matches!(modal, Modal::Delete(_)) {
                        "Confirm"
                    } else {
                        "Save"
                    },
                    p,
                    true,
                ) || submit
                {
                    self.finish_modal();
                }
            }
        }
        if let Some(message) = &self.message
            && !matches!(modal, Modal::Preview(_) | Modal::Details(_))
        {
            label(
                f,
                Rect {
                    y: inner.y + 54.0,
                    height: 27.0,
                    ..inner
                },
                message,
                p.muted,
                12.0,
                400,
            );
        }
        action
    }
}

fn format_date(time: std::time::SystemTime) -> String {
    #[cfg(unix)]
    {
        let seconds = time
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()) as libc::time_t;
        let mut time = std::mem::MaybeUninit::<libc::tm>::uninit();
        // SAFETY: localtime_r initializes the supplied struct on success.
        let value = unsafe {
            if libc::localtime_r(&seconds, time.as_mut_ptr()).is_null() {
                return String::new();
            }
            time.assume_init()
        };
        format!(
            "{:04}-{:02}-{:02}",
            value.tm_year + 1900,
            value.tm_mon + 1,
            value.tm_mday
        )
    }
    #[cfg(not(unix))]
    {
        let secs = time
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let days = (secs / 86_400) as i64 + 719_468;
        let era = days.div_euclid(146_097);
        let doe = days.rem_euclid(146_097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = y + i64::from(m <= 2);
        format!("{year:04}-{m:02}-{d:02}")
    }
}
