use super::*;
use crate::wallpaper::{self, Destination, Pending, Source};
use std::sync::mpsc;

const ID: u64 = LAUNCHER_ID + 6000;

fn rect(parent: Rect, x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x: parent.x + x,
        y: parent.y + y,
        width,
        height,
    }
}

fn button(f: &mut Frame<'_>, id: u64, b: Rect, label: &str, active: bool, p: Palette) -> bool {
    let response = f.target(ID + id, b, label, true, Role::Button);
    let foreground = if active {
        p.on_primary_container
    } else {
        p.on_surface
    };
    f.paint_rect(
        b,
        b.height.min(54.0) * 0.5,
        if active {
            p.primary_container
        } else {
            p.card_elevated
        },
    );
    f.state_layer(response, b.height.min(54.0) * 0.5, foreground);
    f.styled_text(b.inset(4.0), label, foreground, text_style(14.0, 600, 1));
    response.clicked
}

fn line(f: &mut Frame<'_>, b: Rect, label: &str, color: Color, size: f32, font: usize) {
    f.styled_text(
        b,
        label,
        color,
        wallpaper::customize_font(text_style(size, 500, 0), font),
    );
}

fn caption(f: &mut Frame<'_>, b: Rect, label: &str, color: Color) {
    f.styled_text(b, label, color, text_style(14.0, 600, 1));
}

fn overlay_preview(
    clock: ClockReading,
    f: &mut Frame<'_>,
    b: Rect,
    lock: bool,
    p: Palette,
    sys: &SystemState,
    font: usize,
) {
    let white = Color(255, 255, 255, 255);
    f.paint_rect(
        rect(b, 0.0, 0.0, b.width, b.height * 0.15),
        0.0,
        Color(0, 0, 0, 34),
    );
    let s = (b.width / 160.0).clamp(0.6, 2.0);
    if lock {
        line(
            f,
            rect(b, 10.0 * s, b.height * 0.087, b.width - 20.0 * s, 16.0 * s),
            "Lock screen",
            white,
            11.0 * s,
            font,
        );
        f.styled_text(
            rect(b, 0.0, b.height * 0.30, b.width, b.height * 0.20),
            &clock.hm(),
            white,
            wallpaper::customize_font(text_style(31.0 * s, 600, 1), font),
        );
        for (slot, short) in sys.lock_shortcuts.iter().copied().enumerate() {
            if let Some(icon) = super::lockscreen::shortcut_icon(short) {
                let d = 22.0 * s;
                let x = if slot == 0 {
                    12.0 * s
                } else {
                    b.width - 34.0 * s
                };
                let circle = rect(b, x, b.height - 39.0 * s, d, d);
                f.paint_rect(circle, d * 0.5, Color(28, 33, 43, 210));
                paint_sys_icon(f, circle.inset(d * 0.23), icon, white);
            }
        }
    } else {
        let (columns, rows) = sys.grid_dims();
        let cw = b.width / columns as f32;
        let ch = b.height * 0.65 / rows as f32;
        let apps = [
            AppId::Files,
            AppId::Clock,
            AppId::Settings,
            AppId::Calculator,
            AppId::Camera,
        ];
        let icon_size = cw.min(ch) * 0.62;
        for (i, app) in apps
            .iter()
            .cycle()
            .take((columns * 2).min(columns * rows))
            .enumerate()
        {
            let x = (i % columns) as f32 * cw + (cw - icon_size) * 0.5;
            let y = b.height * 0.76 - (i / columns + 1) as f32 * ch;
            icons::paint_styled_app_icon(
                clock,
                f,
                rect(b, x, y, icon_size, icon_size),
                *app,
                sys.themed_icons,
                &p,
                sys.icon_shape,
            );
        }
        let pill = rect(
            b,
            b.width * 0.31,
            b.height - 9.0 * s,
            b.width * 0.38,
            2.0 * s,
        );
        f.paint_rect(pill, 3.0, white);
    }
}

impl<E: EmbeddedApps> LauncherDemo<E> {
    fn preview_wallpaper(&mut self, source: Source, from: LauncherSub, lock: bool, now: f64) {
        self.wallpaper_pending = Some(Pending::new(source, from, lock));
        self.wallpaper_load = None;
        self.ime_target = None;
        self.ime_anim.set(0.0, now, 0.20, 0.18);
        self.set_launcher_sub(LauncherSub::WallpaperPreview, true, now);
    }

    pub(super) fn paint_wallpaper_page(
        &mut self,
        f: &mut Frame<'_>,
        sub: LauncherSub,
        content: Rect,
        phone: Rect,
        now: f64,
    ) {
        let p = self.sys.palette();
        match sub {
            LauncherSub::WallpaperStyle => self.paint_style(f, content, p, now),
            LauncherSub::WallpaperPicker => self.paint_wallpaper_picker(f, content, p, now),
            LauncherSub::WallpaperFiles => self.paint_wallpaper_files(f, content, p, now),
            LauncherSub::WallpaperPreview => {
                self.paint_wallpaper_preview(f, content, phone, p, now)
            }
            LauncherSub::WallpaperSettings(kind) => self.paint_style_choices(f, content, p, kind),
            _ => {}
        }
    }

    fn paint_style(&mut self, f: &mut Frame<'_>, content: Rect, p: Palette, now: f64) {
        let max_scroll = (840.0 - content.height).max(0.0);
        self.last_scroll_max[3] = max_scroll;
        self.wallpaper_scroll = self.wallpaper_scroll.clamp(0.0, max_scroll);
        f.scroll_viewport(ID + 1, content, &mut self.wallpaper_scroll, max_scroll);
        let canvas = Rect {
            y: content.y - self.wallpaper_scroll,
            ..content
        };
        f.within(content, |f| {
            let tw = (content.width - 60.0) * 0.5;
            let h = 232.0;
            for slot in 0..2 {
                let lock = slot == 0;
                let name = if lock { "Lock screen" } else { "Home screen" };
                let bounds = rect(canvas, 24.0 + slot as f32 * (tw + 12.0), 10.0, tw, h);
                let current = if lock {
                    &self.wallpapers.lock
                } else {
                    &self.wallpapers.home
                };
                let response = f.target(ID + 2 + slot as u64, bounds, name, true, Role::Button);
                f.within(bounds, |f| {
                    current.paint(f, bounds, self.sys.wallpaper_idx);
                    overlay_preview(self.clock, f, bounds, lock, p, &self.sys, self.sys.font_style);
                });
                mask_rounded_corners(f, bounds, 22.0, p.settings_bg);
                f.state_layer(response, 22.0, p.on_surface);
                caption(f, rect(bounds, 0.0, h + 1.0, tw, 30.0), name, p.on_surface);
                if response.clicked {
                    self.preview_wallpaper(
                        Source::Current(current.clone()),
                        LauncherSub::WallpaperStyle,
                        lock,
                        now,
                    );
                }
            }
            if button(
                f,
                4,
                rect(canvas, 24.0, 280.0, canvas.width - 48.0, 44.0),
                "Change wallpaper",
                true,
                p,
            ) {
                self.wallpaper_picker_scroll = 0.0;
                self.set_launcher_sub(LauncherSub::WallpaperPicker, true, now);
            }

            let tab_width = (canvas.width - 52.0) * 0.5;
            for (i, name) in ["Wallpaper colors", "Basic colors"].into_iter().enumerate() {
                if button(
                    f,
                    5 + i as u64,
                    rect(
                        canvas,
                        22.0 + i as f32 * (tab_width + 8.0),
                        342.0,
                        tab_width,
                        42.0,
                    ),
                    name,
                    self.wallpaper_tab_basic == (i == 1),
                    p,
                ) {
                    self.wallpaper_tab_basic = i == 1;
                    self.wallpaper_color_page = 0;
                }
            }
            const BASIC: [[u8; 3]; 12] = [
                [67, 94, 145],
                [56, 106, 77],
                [101, 90, 149],
                [140, 98, 38],
                [147, 75, 110],
                [155, 57, 58],
                [83, 109, 28],
                [12, 106, 112],
                [36, 104, 152],
                [104, 76, 56],
                [111, 88, 109],
                [83, 87, 100],
            ];
            let dynamic = self.wallpapers.home.colors(self.sys.wallpaper_idx);
            let page_w = (canvas.width - 52.0) / 4.0;
            for i in 0..4 {
                let index = self.wallpaper_color_page * 4 + i;
                let color = if self.wallpaper_tab_basic {
                    let [r, g, b] = BASIC[index];
                    Color(r, g, b, 255)
                } else {
                    dynamic[i]
                };
                let label = if self.wallpaper_tab_basic {
                    "Basic color"
                } else {
                    "Wallpaper color"
                };
                let slot = rect(
                    canvas,
                    23.0 + i as f32 * (page_w + 2.0),
                    392.0,
                    page_w,
                    62.0,
                );
                let response = f.target(ID + 9 + i as u64, slot, label, true, Role::Radio);
                let diameter = (page_w - 12.0).min(50.0);
                let dot = rect(
                    slot,
                    (page_w - diameter) * 0.5,
                    (62.0 - diameter) * 0.5,
                    diameter,
                    diameter,
                );
                let active = if self.wallpaper_tab_basic && index < 4 {
                    self.sys.wallpaper_color_slot.is_none()
                        && self.sys.accent_rgb.is_none()
                        && self.sys.accent_idx == index
                } else {
                    let matches = self.sys.accent_rgb == Some([color.0, color.1, color.2]);
                    matches
                        && (self.sys.wallpaper_color_slot
                            == (!self.wallpaper_tab_basic).then_some(i))
                };
                if active {
                    f.paint_rect(dot.inset(-3.0), (diameter + 6.0) * 0.5, p.on_surface);
                }
                f.paint_rect(dot, diameter * 0.5, color);
                if active {
                    f.paint_material_icon(
                        dot.inset(diameter * 0.28),
                        MaterialIcon::Check,
                        Color(255, 255, 255, 255),
                    );
                }
                if response.clicked {
                    self.sys.wallpaper_color_slot = (!self.wallpaper_tab_basic).then_some(i);
                    self.sys.accent_rgb = if self.wallpaper_tab_basic && index < 4 {
                        self.sys.accent_idx = index;
                        None
                    } else {
                        Some([color.0, color.1, color.2])
                    };
                    self.sys.save_persisted_settings();
                }
            }
            if self.wallpaper_tab_basic {
                for i in 0..3 {
                    let dot = rect(
                        canvas,
                        canvas.width * 0.5 - 41.0 + i as f32 * 28.0,
                        456.0,
                        26.0,
                        28.0,
                    );
                    let response = f.target(
                        ID + 16 + i as u64,
                        dot,
                        "Basic colors page",
                        true,
                        Role::Tab,
                    );
                    let ink = if self.wallpaper_color_page == i {
                        p.primary
                    } else {
                        p.outline
                    };
                    f.paint_rect(dot.inset(9.0), 5.0, ink);
                    if response.clicked {
                        self.wallpaper_color_page = i;
                    }
                }
            }
            for (i, name) in ["Dark theme", "Themed icons"].into_iter().enumerate() {
                let mut value = if i == 0 {
                    self.sys.dark_theme
                } else {
                    self.sys.themed_icons
                };
                if paint_pref_switch(
                    f,
                    ID + 24 + i as u64,
                    rect(
                        canvas,
                        20.0,
                        491.0 + 55.0 * i as f32,
                        canvas.width - 40.0,
                        50.0,
                    ),
                    name,
                    None,
                    &mut value,
                    &p,
                ) {
                    if i == 0 {
                        self.sys.dark_theme = value;
                    } else {
                        self.sys.themed_icons = value;
                    }
                    self.sys.save_persisted_settings();
                }
            }
            for (i, kind) in AppearancePage::ALL.into_iter().enumerate() {
                let row = rect(
                    canvas,
                    20.0,
                    606.0 + 44.0 * i as f32,
                    canvas.width - 40.0,
                    42.0,
                );
                let r = f.target(ID + 30 + i as u64, row, kind.title(), true, Role::Button);
                f.state_layer(r, 12.0, p.primary);
                line(
                    f,
                    row.inset(8.0),
                    kind.title(),
                    p.on_surface,
                    15.0,
                    self.sys.font_style,
                );
                paint_sys_icon(
                    f,
                    rect(row, row.width - 25.0, 12.0, 18.0, 18.0),
                    SysIcon::ChevronRight,
                    p.on_surface_variant,
                );
                if r.clicked {
                    self.wallpaper_shortcut_side = 0;
                    self.set_launcher_sub(LauncherSub::WallpaperSettings(kind), true, now);
                }
            }
        });
    }

    fn paint_wallpaper_picker(&mut self, f: &mut Frame<'_>, content: Rect, p: Palette, now: f64) {
        let max_scroll = (642.0 - content.height).max(0.0);
        self.last_scroll_max[3] = max_scroll;
        self.wallpaper_picker_scroll = self.wallpaper_picker_scroll.clamp(0.0, max_scroll);
        f.scroll_viewport(
            ID + 39,
            content,
            &mut self.wallpaper_picker_scroll,
            max_scroll,
        );
        let canvas = Rect {
            y: content.y - self.wallpaper_picker_scroll,
            ..content
        };
        f.within(content, |f| {
            if button(
                f,
                40,
                rect(canvas, 20.0, 8.0, canvas.width - 40.0, 60.0),
                "My photos  ·  Browse Files",
                true,
                p,
            ) {
                self.wallpaper_files
                    .set_picker(spectre_files::PickerRequest::Open {
                        filter: spectre_files::Filter::Images,
                        multiple: false,
                    });
                self.set_launcher_sub(LauncherSub::WallpaperFiles, true, now);
            }
            line(
                f,
                rect(canvas, 20.0, 78.0, canvas.width - 40.0, 32.0),
                "On-device wallpapers",
                p.on_surface,
                17.0,
                self.sys.font_style,
            );
            let width = (canvas.width - 56.0) / 3.0;
            for (index, name) in wallpaper::PRESETS.into_iter().enumerate() {
                let bounds = rect(
                    canvas,
                    18.0 + (index % 3) as f32 * (width + 10.0),
                    121.0 + (index / 3) as f32 * 170.0,
                    width,
                    142.0,
                );
                let r = f.target(ID + 41 + index as u64, bounds, name, true, Role::Button);
                f.within(bounds, |f| paint_wallpaper(f, bounds, index));
                mask_rounded_corners(f, bounds, 16.0, p.settings_bg);
                f.state_layer(r, 16.0, Color(255, 255, 255, 255));
                if self
                    .wallpapers
                    .home
                    .uses_preset(index, self.sys.wallpaper_idx)
                {
                    f.paint_material_icon(
                        rect(bounds, bounds.width - 26.0, 7.0, 19.0, 19.0),
                        MaterialIcon::Check,
                        Color(255, 255, 255, 255),
                    );
                }
                caption(f, rect(bounds, 0.0, 143.0, width, 23.0), name, p.on_surface);
                if r.clicked {
                    self.preview_wallpaper(
                        Source::Preset(index),
                        LauncherSub::WallpaperPicker,
                        false,
                        now,
                    );
                }
            }
        });
    }

    fn paint_wallpaper_files(&mut self, f: &mut Frame<'_>, content: Rect, p: Palette, now: f64) {
        if self.wallpaper_load.is_some() && self.launcher_sub == LauncherSub::WallpaperFiles {
            let ready = self.wallpaper_load.as_ref().unwrap().try_recv();
            match ready {
                Ok(Ok(source)) => {
                    self.preview_wallpaper(source, LauncherSub::WallpaperFiles, false, now);
                    return;
                }
                Ok(Err(error)) => {
                    self.wallpaper_load = None;
                    self.wallpaper_files
                        .report(format!("Could not open image: {error}"));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.wallpaper_load = None;
                    self.wallpaper_files.report("Could not prepare image");
                }
                Err(mpsc::TryRecvError::Empty) => {
                    caption(f, content, "Loading image…", p.on_surface);
                    f.continuous(content);
                    return;
                }
            }
        }
        match self
            .wallpaper_files
            .show(f, content, apps::files_appearance(&p))
        {
            spectre_files::Action::Selected(paths) => {
                if let Some(path) = paths.into_iter().next() {
                    let (send, receive) = mpsc::channel();
                    self.wallpaper_load = Some(receive);
                    std::thread::spawn(move || {
                        let _ = send.send(Pending::load(&path));
                    });
                    f.continuous(content);
                }
            }
            spectre_files::Action::Focus(id) => self.open_ime_for(id),
            spectre_files::Action::Blur => self.blur_app_text_field(now),
            spectre_files::Action::Cancelled => self.leave_wallpaper_files(now),
            _ => {}
        }
    }

    pub(super) fn leave_wallpaper_files(&mut self, now: f64) {
        self.wallpaper_load = None;
        self.set_launcher_sub(LauncherSub::WallpaperPicker, false, now);
    }

    fn paint_wallpaper_preview(
        &mut self,
        f: &mut Frame<'_>,
        content: Rect,
        phone: Rect,
        p: Palette,
        now: f64,
    ) {
        let Some(pending) = self.wallpaper_pending.as_mut() else {
            return;
        };
        let mut apply = None;
        let aspect = phone.width / phone.height.max(1.0);
        let w = (content.width - 36.0).min((content.height - 198.0).max(48.0) * aspect);
        let screen = rect(content, (content.width - w) / 2.0, 56.0, w, w / aspect);
        f.without_input(pending.show_apply, |f| {
            for (i, label) in ["Home screen", "Lock screen"].into_iter().enumerate() {
                let width = (content.width - 48.0) * 0.5;
                if button(
                    f,
                    51 + i as u64,
                    rect(content, 20.0 + i as f32 * (width + 8.0), 3.0, width, 42.0),
                    label,
                    pending.lock_tab == (i == 1),
                    p,
                ) {
                    pending.lock_tab = i == 1;
                }
            }
            f.within(screen, |f| {
                pending.paint(f, screen, self.sys.wallpaper_idx, aspect);
                overlay_preview(
                    self.clock,
                    f,
                    screen,
                    pending.lock_tab,
                    p,
                    &self.sys,
                    self.sys.font_style,
                );
            });
            mask_rounded_corners(f, screen, 22.0, p.settings_bg);
            let base = screen.y + screen.height - content.y;
            let control_width = (content.width - 56.0) / 3.0;
            let editable = matches!(pending.source, Source::Photo { .. });
            if button(
                f,
                53,
                rect(content, 20.0, base + 12.0, control_width, 40.0),
                "Info",
                pending.show_info,
                p,
            ) {
                pending.show_info = !pending.show_info;
                pending.edit = false;
            }
            if editable
                && button(
                    f,
                    54,
                    rect(
                        content,
                        28.0 + control_width,
                        base + 12.0,
                        control_width,
                        40.0,
                    ),
                    "Edit",
                    pending.edit,
                    p,
                )
            {
                pending.edit = !pending.edit;
                pending.show_info = false;
            }
            if button(
                f,
                55,
                rect(
                    content,
                    36.0 + 2.0 * control_width,
                    base + 12.0,
                    control_width,
                    40.0,
                ),
                "Apply",
                true,
                p,
            ) {
                pending.show_apply = true;
                pending.drag = None;
            }
            if pending.show_info {
                caption(
                    f,
                    rect(content, 10.0, base + 62.0, content.width - 20.0, 34.0),
                    &pending.info(),
                    p.on_surface,
                );
            }
            if pending.edit {
                let zone = f.target(
                    ID + 56,
                    screen,
                    "Move photo within crop",
                    true,
                    Role::Button,
                );
                let input = f.input();
                if zone.pressed && !pending.show_apply {
                    if let Some(position) = input.pointer {
                        if let Some(old) = pending.drag.replace(position) {
                            pending.pan(
                                [old[0] - position[0], old[1] - position[1]],
                                screen,
                                aspect,
                            );
                        }
                    }
                } else {
                    pending.drag = None;
                }
                for (i, (label, delta)) in [("Zoom out", -0.18_f32), ("Zoom in", 0.18)]
                    .into_iter()
                    .enumerate()
                {
                    let width = (content.width - 48.0) * 0.5;
                    if button(
                        f,
                        57 + i as u64,
                        rect(
                            content,
                            20.0 + i as f32 * (width + 8.0),
                            base + 61.0,
                            width,
                            42.0,
                        ),
                        label,
                        false,
                        p,
                    ) {
                        pending.zoom = (pending.zoom + delta).clamp(1.0, 4.0);
                    }
                }
            }
        });
        if pending.show_apply {
            f.paint_rect(content, 0.0, Color(0, 0, 0, 130));
            let dialog = rect(
                content,
                20.0,
                (content.height - 272.0) * 0.54,
                content.width - 40.0,
                266.0,
            );
            f.paint_rect(dialog, 26.0, p.settings_bg);
            line(
                f,
                rect(dialog, 21.0, 12.0, dialog.width - 42.0, 48.0),
                "Set wallpaper on",
                p.on_surface,
                21.0,
                self.sys.font_style,
            );
            for (i, (label, destination)) in [
                ("Home screen", Destination::Home),
                ("Lock screen", Destination::Lock),
                ("Home and lock screens", Destination::Both),
            ]
            .into_iter()
            .enumerate()
            {
                if button(
                    f,
                    59 + i as u64,
                    rect(
                        dialog,
                        18.0,
                        62.0 + 48.0 * i as f32,
                        dialog.width - 36.0,
                        43.0,
                    ),
                    label,
                    false,
                    p,
                ) {
                    apply = Some(destination);
                }
            }
            if button(
                f,
                62,
                rect(dialog, 18.0, 212.0, dialog.width - 36.0, 40.0),
                "Cancel",
                false,
                p,
            ) {
                pending.show_apply = false;
            }
        }
        if let Some(destination) = apply {
            match self.wallpapers.apply(pending, destination, aspect) {
                Ok(()) => {
                    if let Some(index) = self.sys.wallpaper_color_slot {
                        let Color(r, g, b, _) =
                            self.wallpapers.home.colors(self.sys.wallpaper_idx)[index];
                        self.sys.accent_rgb = Some([r, g, b]);
                        self.sys.save_persisted_settings();
                    }
                    self.wallpaper_pending = None;
                    self.set_launcher_sub(LauncherSub::WallpaperStyle, false, now);
                    self.show_toast("Wallpaper applied", now);
                }
                Err(error) => {
                    pending.show_apply = false;
                    self.show_toast(format!("Could not set wallpaper: {error}"), now);
                }
            }
        }
    }

    fn paint_style_choices(
        &mut self,
        f: &mut Frame<'_>,
        content: Rect,
        p: Palette,
        kind: AppearancePage,
    ) {
        let width = content.width - 40.0;
        let preview = rect(
            content,
            content.width * 0.28,
            8.0,
            content.width * 0.44,
            (content.height * 0.36).clamp(100.0, 246.0),
        );
        let lock = kind == AppearancePage::Shortcuts;
        let wallpaper = if lock {
            &self.wallpapers.lock
        } else {
            &self.wallpapers.home
        };
        f.within(preview, |f| {
            wallpaper.paint(f, preview, self.sys.wallpaper_idx);
            overlay_preview(self.clock, f, preview, lock, p, &self.sys, self.sys.font_style);
            if kind == AppearancePage::Icon {
                for (i, icon) in [SysIcon::Wifi, SysIcon::Bluetooth, SysIcon::Flashlight]
                    .into_iter()
                    .enumerate()
                {
                    let bounds = rect(
                        preview,
                        9.0 + (preview.width - 18.0) / 3.0 * i as f32,
                        preview.height * 0.18,
                        29.0,
                        29.0,
                    );
                    paint_wallpaper_system_icon(
                        f,
                        bounds,
                        icon,
                        Color(255, 255, 255, 255),
                        self.sys.icon_style,
                    );
                }
            }
        });
        mask_rounded_corners(f, preview, 19.0, p.settings_bg);
        let mut base = preview.y + preview.height - content.y + 18.0;
        if lock {
            let half = (width - 8.0) * 0.5;
            for (i, name) in ["Left shortcut", "Right shortcut"].into_iter().enumerate() {
                if button(
                    f,
                    63 + i as u64,
                    rect(content, 20.0 + (half + 8.0) * i as f32, base, half, 42.0),
                    name,
                    self.wallpaper_shortcut_side == i,
                    p,
                ) {
                    self.wallpaper_shortcut_side = i;
                }
            }
            base += 54.0;
        }
        const GRID: [&str; 6] = ["5 × 5", "4 × 5", "4 × 4", "3 × 3", "2 × 2", "5 × 6"];
        let (choices, selected): (&[&str], usize) = match kind {
            AppearancePage::Grid => (&GRID, self.sys.grid_idx),
            AppearancePage::Shortcuts => (
                &wallpaper::SHORTCUTS,
                self.sys.lock_shortcuts[self.wallpaper_shortcut_side],
            ),
            AppearancePage::Icon => (&wallpaper::ICON_STYLES, self.sys.icon_style),
            AppearancePage::Font => (&wallpaper::FONT_STYLES, self.sys.font_style),
            AppearancePage::Shape => (&wallpaper::ICON_SHAPES, self.sys.icon_shape),
        };
        let half = (width - 8.0) * 0.5;
        for (i, label) in choices.iter().enumerate() {
            let bounds = rect(
                content,
                20.0 + (half + 8.0) * (i % 2) as f32,
                base + 50.0 * (i / 2) as f32,
                half,
                44.0,
            );
            if button(f, 70 + i as u64, bounds, label, selected == i, p) {
                match kind {
                    AppearancePage::Grid => {
                        let previous = self.sys.grid_dims();
                        self.sys.grid_idx = i;
                        self.reflow_home_grid(previous);
                    }
                    AppearancePage::Shortcuts => {
                        self.sys.lock_shortcuts[self.wallpaper_shortcut_side] = i
                    }
                    AppearancePage::Icon => self.sys.icon_style = i,
                    AppearancePage::Font => self.sys.font_style = i,
                    AppearancePage::Shape => self.sys.icon_shape = i,
                }
                self.sys.save_persisted_settings();
            }
        }
    }
}

pub(super) fn paint_wallpaper_system_icon(
    f: &mut Frame<'_>,
    bounds: Rect,
    icon: SysIcon,
    ink: Color,
    style: usize,
) {
    match style {
        1 | 2 | 3 | 4 | 6 => {
            let radius = match style {
                2 => 1.0,
                3 => 6.0,
                4 | 6 => bounds.width.min(bounds.height) * 0.5,
                _ => 4.0,
            };
            f.paint_rect(
                bounds,
                radius,
                ink.opacity(if style == 6 { 0.30 } else { 0.42 }),
            );
            if style != 6 {
                f.paint_rect(
                    bounds.inset(1.3),
                    (radius - 1.0).max(0.0),
                    Color(25, 29, 37, 240),
                );
            }
            paint_sys_icon(f, bounds.inset(bounds.width * 0.19), icon, ink);
        }
        5 => paint_sys_icon(f, bounds.inset(bounds.width * 0.22), icon, ink),
        _ => paint_sys_icon(f, bounds, icon, ink),
    }
}
