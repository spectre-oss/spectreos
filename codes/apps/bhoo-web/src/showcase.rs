//! Bhoo-rendered chrome for the continuous Beer CSS-inspired showcase.
use bhoo::*;
pub const NAV: &[&str] = &[
    "App bars",
    "Badges",
    "Buttons",
    "Cards",
    "Checkboxes",
    "Chips",
    "Colors",
    "Containers",
    "Dialogs",
    "Directions",
    "Dividers",
    "Expansions",
    "Fieldsets",
    "Grids",
    "Icons",
    "Inputs",
    "Layouts",
    "Lists",
    "Media",
    "Menus",
    "Navigations",
    "Pages",
    "Progress",
    "Radios",
    "Responsive",
    "Scrolls",
    "Search",
    "Selects",
    "Shapes",
    "Sliders",
    "Snackbars",
    "Spaces",
    "Steppers",
    "Switches",
    "Tables",
    "Tabs",
    "Textareas",
    "Toolbars",
    "Tooltips",
    "Typography",
    "Waves",
    "More components",
];
#[derive(Default)]
pub struct Showcase {
    pub title: String,
    pub description: String,
    pub action: u32,
    pub selected: usize,
    pub expanded: bool,
    offset: f32,
    bottom_selected: usize,
    dragging: bool,
    pub dialog: OverlayState,
    logo: Option<Image>,
}
fn text(f: &mut Frame<'_>, bounds: Rect, value: &str, size: f32, color: Color, weight: u16) {
    f.styled_text(
        bounds,
        value,
        color,
        TextStyle {
            size,
            weight,
            ..TextStyle::default()
        },
    );
}
fn code(f: &mut Frame<'_>, x: f32, y: f32, color: Color) {
    f.paint_material_icon(
        Rect {
            x,
            y,
            width: 24.,
            height: 24.,
        },
        MaterialIcon::Code,
        color,
    );
}
impl Showcase {
    fn paint_logo(&mut self, f: &mut Frame<'_>, bounds: Rect) {
        let image = self.logo.get_or_insert_with(|| {
            let alpha = include_bytes!("../assets/branding/ghost-alpha.bin");
            let mut pixels = Vec::with_capacity(alpha.len() * 4);
            for &a in alpha {
                pixels.extend_from_slice(&[255, 255, 255, a]);
            }
            Image::rgba(96, 96, pixels).expect("bundled branding dimensions")
        });
        f.paint_rect(bounds, bounds.width * 0.22, Color(103, 80, 164, 255));
        f.paint_image(
            bounds.inset(bounds.width * 0.125),
            image,
            ImageFit::Contain,
            0.,
        );
    }

    fn scroll(&mut self, f: &mut Frame<'_>, _input: Input, area: Rect, max: f32) {
        self.dragging = f.scroll_viewport(9000, area, &mut self.offset, max);
    }
    pub fn show(&mut self, f: &mut Frame<'_>, scene: u32, width: f32, height: f32, input: Input) {
        self.action = 0;
        let t = f.theme();
        let full = Rect {
            x: 0.,
            y: 0.,
            width,
            height,
        };
        match scene {
            10000 => {
                let expanded = self.expanded || width > 120.;
                f.paint_rect(full, 0., t.surface_low);
                let toggle = Rect {
                    x: 16.,
                    y: 12.,
                    width: 48.,
                    height: 48.,
                };
                if f.target(1, toggle, "Toggle navigation width", true, Role::Button)
                    .clicked
                {
                    self.action = 1002;
                }
                f.paint_material_icon(toggle.inset(12.), MaterialIcon::Menu, t.text);
                let start = Rect {
                    x: 12.,
                    y: 76.,
                    width: (width - 24.).max(48.),
                    height: 56.,
                };
                let r = f.target(2, start, "Get started", true, Role::Button);
                f.paint_rect(start, 16., t.primary_container);
                code(f, start.x + 16., start.y + 16., t.on_primary_container);
                if expanded {
                    text(
                        f,
                        Rect {
                            x: 64.,
                            width: (width - 76.).max(0.),
                            ..start
                        },
                        "Get started",
                        14.,
                        t.on_primary_container,
                        500,
                    );
                }
                if r.clicked {
                    self.action = 1000;
                }
                let area = Rect {
                    y: 148.,
                    height: (height - 148.).max(0.),
                    ..full
                };
                self.scroll(
                    f,
                    input,
                    area,
                    (NAV.len() as f32 * 56. - area.height).max(0.),
                );
                f.within(area, |f| {
                    for (i, name) in NAV.iter().enumerate() {
                        let b = Rect {
                            x: 12.,
                            y: 148. + i as f32 * 56. - self.offset,
                            width: (width - 24.).max(0.),
                            height: 52.,
                        };
                        let expansion = ((width - 88.) / 192.).clamp(0., 1.);
                        let r = f.material_navigation_item(
                            100 + i as u64,
                            b,
                            name,
                            MaterialIcon::Widgets,
                            self.selected == i + 1,
                            expansion,
                            true,
                        );
                        if r.clicked && !self.dragging {
                            self.action = i as u32 + 1;
                        }
                    }
                });
            }
            10001 => {
                let b = Rect {
                    x: 4.,
                    y: 4.,
                    width: (width - 8.).max(0.),
                    height: 56.,
                };
                let theme_icon = if t.background.0 < 128 {
                    MaterialIcon::LightMode
                } else {
                    MaterialIcon::DarkMode
                };
                let slots = [
                    FloatingPillSlot { label: "Explore components", icon: Some(MaterialIcon::Menu) },
                    FloatingPillSlot { label: "Bhoo", icon: None },
                    FloatingPillSlot { label: "Theme colors", icon: Some(MaterialIcon::Palette) },
                    FloatingPillSlot { label: "Reduce motion", icon: Some(MaterialIcon::Pause) },
                    FloatingPillSlot { label: "Switch theme", icon: Some(theme_icon) },
                ];
                if let Some(i) = f.floating_pill_app_bar(10, b, &slots, |f, _, logo_rect| {
                    self.paint_logo(f, logo_rect);
                }) {
                    self.action = [1003, 1000, 1011, 1004, 1001][i];
                }
            }
            10002 => {
                f.paint_rect(full, 0., Color(255, 235, 59, 255));
                let ink = Color(28, 27, 20, 255);
                self.paint_logo(
                    f,
                    Rect {
                        x: width / 2. - 36.,
                        y: height * 0.25 - 80.,
                        width: 72.,
                        height: 72.,
                    },
                );
                let title_size = if width < 500. { 84. } else { 132. };
                f.styled_text(
                    Rect {
                        x: 0.,
                        y: height * 0.25,
                        width,
                        height: 160.,
                    },
                    "bhoo.",
                    ink,
                    TextStyle {
                        size: title_size,
                        weight: 700,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
                // Centered typography follows the source site's full-height introduction.
                for (j, line) in [
                    "Build material design",
                    "with a little spirit",
                    "for the whole system",
                ]
                .iter()
                .enumerate()
                {
                    f.styled_text(
                        Rect {
                            x: 20.,
                            y: height * 0.55 + j as f32 * 40.,
                            width: (width - 40.).max(0.),
                            height: 40.,
                        },
                        line,
                        ink,
                        TextStyle {
                            size: if width < 500. { 23. } else { 30. },
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                }
                let b = Rect {
                    x: (width - 200.) / 2.,
                    y: height - if width < 500. { 160. } else { 90. },
                    width: 200.,
                    height: 48.,
                };
                if f.target(30, b, "Explore components", true, Role::Button)
                    .clicked
                {
                    self.action = 1005;
                }
                f.paint_rect(b, 24., ink);
                f.styled_text(
                    b,
                    "Explore components",
                    Color(255, 235, 59, 255),
                    TextStyle {
                        size: 14.,
                        align: 1,
                        ..TextStyle::LABEL
                    },
                );
            }
            10003 => {
                text(
                    f,
                    Rect {
                        x: 24.,
                        y: 12.,
                        width: (width - 104.).max(0.),
                        height: 42.,
                    },
                    &self.title,
                    26.,
                    t.text,
                    500,
                );
                text(
                    f,
                    Rect {
                        x: 24.,
                        y: 53.,
                        width: (width - 48.).max(0.),
                        height: 28.,
                    },
                    &self.description,
                    14.,
                    t.on_surface_variant,
                    400,
                );
                let b = Rect {
                    x: width - 64.,
                    y: 12.,
                    width: 40.,
                    height: 40.,
                };
                f.paint_rect(b, 20., t.surface_low);
                code(f, b.x + 8., b.y + 8., t.text);
                if f.target(40, b, "Show Rust source", true, Role::Button)
                    .clicked
                {
                    self.action = 1006;
                }
            }
            10004 => {
                let mut dialog = std::mem::take(&mut self.dialog);
                dialog.kind = OverlayKind::Dialog;
                dialog.initial_focus = Some(50);
                let dialog_height = (152. + self.description.lines().count() as f32 * 24.)
                    .max(200.)
                    .min(height * 0.76);
                let bounds = Rect {
                    x: (width - (width - 48.).min(720.)) / 2.,
                    y: (height - dialog_height) / 2.,
                    width: (width - 48.).clamp(0., 720.),
                    height: dialog_height,
                };
                f.overlay(800, &mut dialog, bounds, true, |f, open| {
                    let header = f.reserve(56.);
                    text(
                        f,
                        Rect {
                            width: (header.width - 48.).max(0.),
                            ..header
                        },
                        &self.title,
                        24.,
                        t.text,
                        400,
                    );
                    let close = Rect {
                        x: header.x + header.width - 40.,
                        y: header.y,
                        width: 40.,
                        height: 40.,
                    };
                    if f.target(50, close, "Close source", true, Role::Button)
                        .clicked
                    {
                        *open = false;
                    }
                    f.paint_material_icon(close.inset(8.), MaterialIcon::Close, t.text);
                    let area = f.reserve((bounds.height - 96.).max(0.));
                    let line_count = self.description.lines().count();
                    self.scroll(
                        f,
                        input,
                        area,
                        (line_count as f32 * 24. - area.height).max(0.),
                    );
                    f.overlay_section(OverlaySection::Content, |f| {
                        f.within(area, |f| {
                            for (i, line) in self.description.lines().enumerate() {
                                text(
                                    f,
                                    Rect {
                                        x: area.x + 8.,
                                        y: area.y + i as f32 * 24. - self.offset,
                                        width: (area.width - 16.).max(0.),
                                        height: 24.,
                                    },
                                    line,
                                    14.,
                                    t.text,
                                    400,
                                );
                            }
                        });
                    });
                });
                if !dialog.open {
                    self.action = 1007;
                }
                self.dialog = dialog;
            }

            10005 => {
                f.paint_rect(full, 0., t.surface_low);
                for (i, label) in ["Install", "Explore", "Top", "Bottom", "Code"]
                    .iter()
                    .enumerate()
                {
                    let b = Rect {
                        x: i as f32 * width / 5.,
                        y: 0.,
                        width: width / 5.,
                        height,
                    };
                    let icon = [
                        MaterialIcon::Download,
                        MaterialIcon::Menu,
                        MaterialIcon::VerticalAlignTop,
                        MaterialIcon::VerticalAlignBottom,
                        MaterialIcon::Code,
                    ][i];
                    if f.material_navigation_bar_item(
                        60 + i as u64,
                        b,
                        label,
                        icon,
                        self.bottom_selected == i,
                        true,
                    )
                    .clicked
                    {
                        self.bottom_selected = i;
                        self.action = [1000, 1003, 1008, 1009, 1010][i];
                    }
                }
            }
            _ => {}
        }
        if input.released {
            self.dragging = false;
        }
    }
}
