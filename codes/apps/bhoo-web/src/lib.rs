//! Tiny dependency-free browser ABI around the actual Bhoo Rust core.
//! JavaScript paints these commands; state, hit testing, focus, and motion stay here.
mod android_apps;
mod android_demo;
mod android_material_divider;
mod contacts_demo;
mod gallery;
mod gmail_demo;
mod onepage;
mod showcase;
use bhoo::*;
use std::cell::RefCell;

#[derive(Default)]
struct DemoEmbeddedApps {
    contacts: Box<contacts_demo::ContactsDemo>,
    gmail: Box<gmail_demo::GmailDemo>,
}

impl wisp::EmbeddedApps for DemoEmbeddedApps {
    fn render_contacts(&mut self, f: &mut Frame<'_>, bounds: Rect) {
        self.contacts.show_in(f, bounds);
    }

    fn open_new_contact(&mut self, f: &mut Frame<'_>) {
        self.contacts.open_new_contact(f);
    }

    fn render_gmail(&mut self, f: &mut Frame<'_>, bounds: Rect, compose_body: &mut String) {
        self.gmail.show_in(f, bounds, compose_body);
    }

    fn edit_text_for(&self, id: u64) -> &str {
        let c = self.contacts.edit_text_for(id);
        if !c.is_empty() {
            c
        } else {
            self.gmail.edit_text_for(id)
        }
    }

    fn set_edit_text(&mut self, id: u64, value: &str) -> bool {
        self.contacts.set_edit_text(id, value);
        let mut body = String::new();
        self.gmail.set_edit_text(id, value, &mut body);
        id == bhoo::android_xml::view_id("compose_body")
    }
}

#[derive(Default)]
struct Demo {
    ui: Ui,
    gallery: gallery::Gallery,
    showcase: showcase::Showcase,
    android_demo: android_demo::AndroidDemo,
    gmail_demo: gmail_demo::GmailDemo,
    contacts_demo: contacts_demo::ContactsDemo,
    launcher_demo: wisp::WispShell<DemoEmbeddedApps>,
    android_apps: android_apps::AndroidApps,
    app_snapshot: Vec<u8>,
    app_restore: Vec<u8>,
    commands: Vec<DrawCommand>,
    packet: Vec<u8>,
    scene: u32,
    clicks: u32,
    check: CheckState,
    error_check: bool,
    checkbox_error: bool,
    binary: bool,
    switches: [bool; 3],
    value: f32,
    range: [f32; 2],
    tab: usize,
    radio: u64,
    radio_rtl: bool,
    list: ListState,
    local_scroll_remaining: f32,
    local_scroll_time: f64,
    video_buffer: Vec<u8>,
    video_action_arg: u64,
}
impl Demo {
    fn reset(&mut self, scene: u32) {
        let text_metrics = self.ui.text_metrics;
        *self = Self {
            scene,
            gallery: gallery::Gallery::new(),
            check: CheckState::Mixed,
            checkbox_error: true,
            switches: [true, false, true],
            value: 40.0,
            range: [25.0, 75.0],
            radio: 301,
            ..Self::default()
        };
        self.ui.text_metrics = text_metrics;
    }
}
thread_local! {static DEMO:RefCell<Demo>=RefCell::new(Demo::default());}
/// Temporary RGBA buffer for a decoded browser video frame.
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_video_frame_buffer(width:u32,height:u32)->u32 {
    if width==0||height==0||width>1022||height>1022{return 0;}
    let Some(length)=(width as usize).checked_mul(height as usize).and_then(|size|size.checked_mul(4)) else{return 0;};
    DEMO.with_borrow_mut(|d|{d.video_buffer.resize(length,0);d.video_buffer.as_mut_ptr() as usize as u32})
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_video_frame_commit(width:u32,height:u32)->u32 {
    DEMO.with_borrow_mut(|d|{if let Some(image)=Image::rgba(width,height,d.video_buffer.clone()){d.gallery.video_host_frame(image);1}else{0}})
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_video_host_status(ready:u32,width:u32,height:u32,position_ms:f64,duration_ms:f64,playing:u32,buffered:u32) {
    let value=|v:f64|if v.is_finite(){v.max(0.0).min(u64::MAX as f64) as u64}else{0};
    DEMO.with_borrow_mut(|d|d.gallery.video_host_status(ready!=0,width,height,value(position_ms),value(duration_ms),playing!=0,buffered.min(100) as u8));
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_video_take_action()->u32 {
    DEMO.with_borrow_mut(|d|match d.gallery.take_video_host_command(){Some(android_xml::MediaControllerAction::Play)=>1,Some(android_xml::MediaControllerAction::Pause)=>2,Some(android_xml::MediaControllerAction::SeekTo(position))=>{d.video_action_arg=position;3},Some(android_xml::MediaControllerAction::Previous)=>4,Some(android_xml::MediaControllerAction::Next)=>5,None=>0})
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_video_action_value()->f64 {DEMO.with_borrow(|d|d.video_action_arg as f64)}
// One WASM instance retains independent widget state for every canvas example.
thread_local! {static CONTEXTS:RefCell<(u32,std::collections::BTreeMap<u32,Demo>)>=const { RefCell::new((0,std::collections::BTreeMap::new())) };}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_context_select(id: u32) {
    CONTEXTS.with_borrow_mut(|c| {
        if c.0 != id {
            DEMO.with_borrow_mut(|d| {
                let next = c.1.remove(&id).unwrap_or_default();
                let old = std::mem::replace(d, next);
                c.1.insert(c.0, old);
                c.0 = id;
            });
        }
    });
}
// Comparison-only palette bridge. No changes to native defaults or widget geometry.
thread_local! {static REFERENCE_PALETTE:RefCell<[Option<Color>;22]>=const { RefCell::new([None;22]) };}

#[derive(Default)]
struct PaletteMotion {
    from: [Option<Color>; 22],
    current: [Option<Color>; 22],
    pending: bool,
    started: f64,
    initialized: bool,
    moving: bool,
}
thread_local! {static PALETTE_MOTION:RefCell<PaletteMotion>=RefCell::new(PaletteMotion::default());}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_palette_set(index: u32, rgba: u32) {
    REFERENCE_PALETTE.with_borrow_mut(|palette| {
        if let Some(slot) = palette.get_mut(index as usize) {
            let [r, g, b, a] = rgba.to_le_bytes();
            let color = Some(Color(r, g, b, a));
            if *slot != color {
                PALETTE_MOTION.with_borrow_mut(|p| {
                    if !p.pending {
                        p.from = p.current;
                        p.pending = true;
                    }
                });
                *slot = color;
            }
        }
    });
}
fn sample_palette(time: f64, reduced: bool) -> [Option<Color>; 22] {
    REFERENCE_PALETTE.with_borrow(|target| {
        PALETTE_MOTION.with_borrow_mut(|p| {
            if !p.initialized {
                p.current = *target;
                p.initialized = true;
                p.pending = false;
            }
            if p.pending {
                p.started = time;
                p.pending = false;
            }
            let progress = if reduced {
                1.
            } else {
                ((time - p.started) / 0.2).clamp(0., 1.) as f32
            };
            let amount = bhoo::timing::Easing::Standard.sample(progress);
            p.moving = progress < 1. && p.from != *target;
            for (i, target_color) in target.iter().enumerate() {
                p.current[i] = match (p.from[i], *target_color) {
                    (Some(a), Some(b)) => Some(a.mix(b, amount)),
                    (_, b) => b,
                };
            }
            p.current
        })
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_palette_moving() -> u32 {
    PALETTE_MOTION.with_borrow(|p| u32::from(p.moving))
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_palette_background() -> u32 {
    PALETTE_MOTION.with_borrow(|p| {
        let Color(r, g, b, a) = p.current[0].unwrap_or(Color(255, 251, 255, 255));
        u32::from_le_bytes([r, g, b, a])
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_reset(scene: u32) {
    DEMO.with_borrow_mut(|demo| demo.reset(scene));
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_font_advance(code: u32, normal: f32, medium: f32) {
    if (32..127).contains(&code)
        && normal.is_finite()
        && medium.is_finite()
        && (0.0..=2.0).contains(&normal)
        && (0.0..=2.0).contains(&medium)
    {
        DEMO.with_borrow_mut(|demo| {
            demo.ui.text_metrics.normal[code as usize - 32] = normal;
            demo.ui.text_metrics.medium[code as usize - 32] = medium;
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_font_baseline(normal:f32, medium:f32) {
    if normal.is_finite() && medium.is_finite() && (-1.0..=1.0).contains(&normal) && (-1.0..=1.0).contains(&medium) {
        DEMO.with_borrow_mut(|demo| {
            demo.ui.text_metrics.normal_baseline_offset=normal;
            demo.ui.text_metrics.medium_baseline_offset=medium;
        });
    }
}
/// Comparison fixture only: copy the Angular checkbox example's committed state.
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_comparison_checkbox_set(binary: u32, tri: u32, focused: u32) {
    DEMO.with_borrow_mut(|demo| {
        if demo.scene != 3 {
            return;
        }
        demo.binary = binary != 0;
        demo.check = match tri {
            0 => CheckState::Unchecked,
            1 => CheckState::Checked,
            _ => CheckState::Mixed,
        };
        demo.ui.adapter_focus(match focused {
            1 | 2 => Some(u64::from(focused)),
            _ => None,
        });
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_adapter_focus(id: u32, focused: u32) {
    DEMO.with_borrow_mut(|demo| {
        demo.ui.adapter_focus(if focused != 0 {
            Some(u64::from(id))
        } else {
            None
        });
        if focused == 0 {
            demo.gallery.touched = true;
        }
    });
}

/// Focus a full-width semantic ID from browser accessibility adapters.
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_adapter_focus64(id: u64, focused: u32) {
    DEMO.with_borrow_mut(|demo| {
        demo.ui.accessibility_focus(if focused != 0 { Some(id) } else { None });
        if focused != 0 {
            demo.launcher_demo.open_ime_for(id);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_showcase_config(selected: u32, expanded: u32) {
    DEMO.with_borrow_mut(|d| {
        d.showcase.selected = selected as usize;
        d.showcase.expanded = expanded != 0;
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_showcase_text(kind: u32, length: u32) {
    DEMO.with_borrow_mut(|d| {
        if let Ok(text) =
            std::str::from_utf8(&d.gallery.edit[..(length as usize).min(d.gallery.edit.len())])
        {
            if kind == 0 {
                d.showcase.title = text.to_owned();
            } else {
                d.showcase.description = text.to_owned();
            }
        }
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_showcase_dialog(open: u32) {
    DEMO.with_borrow_mut(|d| {
        d.showcase.dialog.open = open != 0;
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_showcase_action() -> u32 {
    DEMO.with_borrow(|d| d.showcase.action)
}

/// flags: press=1, release=2, cancel=4, blur=8, key-down=16, repeat=32, shift=64, touch=128, pen=256, horizontal-wheel=512.
/// settings: disabled=1, reduced=2, dark=4, expressive=8, reference palette=16;
/// bits 8..11 select a button variant. The reference palette is demo-only.
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_frame(
    scene: u32,
    width: f32,
    height: f32,
    time: f64,
    x: f32,
    y: f32,
    flags: u32,
    key: u32,
    scroll: f32,
    settings: u32,
) -> u32 {
    DEMO.with_borrow_mut(|demo| {
        if demo.scene != scene {
            demo.reset(scene);
        }
        demo.ui.ripple_style = if settings & 32 != 0 {
            RippleStyle::AngularMaterial
        } else {
            RippleStyle::MaterialWeb
        };
        demo.ui.motion_scheme = if settings & 8 != 0 {
            MotionScheme::Expressive
        } else {
            MotionScheme::Standard
        };
        demo.ui.motion_preference = if settings & 2 != 0 {
            MotionPreference::Reduced
        } else {
            MotionPreference::Full
        };
        demo.ui.theme = if settings & 4 != 0 {
            Theme::default()
        } else {
            Theme::light()
        };
        if settings & 16 != 0 {
            {
                let palette = sample_palette(time, settings & 2 != 0);
                let t = &mut demo.ui.theme;
                for (target, value) in [
                    &mut t.background,
                    &mut t.text,
                    &mut t.control,
                    &mut t.hovered,
                    &mut t.pressed,
                    &mut t.primary,
                    &mut t.on_primary,
                    &mut t.secondary,
                    &mut t.outline,
                    &mut t.error,
                    &mut t.on_secondary,
                    &mut t.surface_low,
                    &mut t.outline_variant,
                    &mut t.primary_container,
                    &mut t.on_primary_container,
                    &mut t.inverse_surface,
                    &mut t.inverse_on_surface,
                    &mut t.on_surface_variant,
                    &mut t.surface_variant,
                    &mut t.inverse_primary,
                    &mut t.tertiary_container,
                    &mut t.on_tertiary_container,
                ]
                .into_iter()
                .zip(&palette)
                {
                    if let Some(color) = value {
                        *target = *color;
                    }
                }
                if let Some(color) = palette[7] { t.secondary_container = color; }
                if let Some(color) = palette[10] { t.on_secondary_container = color; }
                if let Some(color) = palette[4] {
                    t.surface_highest = color;
                }
                if let Some(color) = palette[3] {
                    t.surface_high = color;
                }
            }
        }
        if (10010..=10013).contains(&scene) {
            let t = &mut demo.ui.theme;
            let (primary, container) = match scene {
                10010 => (Color(38, 113, 81, 255), Color(211, 239, 221, 255)),
                10011 => (Color(39, 99, 161, 255), Color(215, 234, 249, 255)),
                10012 => (Color(40, 94, 168, 255), Color(218, 231, 255, 255)),
                _ => (Color(61, 105, 41, 255), Color(218, 235, 198, 255)),
            };
            t.primary = primary;
            t.primary_container = container;
            t.on_primary_container = t.text;
            t.secondary = container;
            t.on_secondary = t.text;
            t.secondary_container = container;
            t.on_secondary_container = t.text;
            t.background = Color(255, 255, 255, 255);
            t.surface_low = if scene == 10012 {Color(246,249,255,255)} else {container.opacity(0.32)};
            t.surface_high = if scene == 10012 { Color(238,242,250,255) } else { Color(242,244,242,255) };
        }
        let key = match key {
            1 => Some(Key::Tab),
            2 => Some(Key::Enter),
            3 => Some(Key::Space),
            4 => Some(Key::Left),
            5 => Some(Key::Right),
            6 => Some(Key::Up),
            7 => Some(Key::Down),
            8 => Some(Key::Home),
            9 => Some(Key::End),
            10 => Some(Key::Escape),
            11 => Some(Key::Menu),
            12 => Some(Key::PageUp),
            13 => Some(Key::PageDown),
            256..=0x1100ff => char::from_u32(key - 256).map(Key::Character),
            _ => None,
        };
        let input = Input {
            pointer_kind: if flags & 128 != 0 {
                PointerKind::Touch
            } else if flags & 256 != 0 {
                PointerKind::Pen
            } else {
                PointerKind::Mouse
            },
            pointer: if x.is_finite() && y.is_finite() {
                Some([x, y])
            } else {
                None
            },
            pressed: flags & 1 != 0,
            released: flags & 2 != 0,
            cancelled: flags & 4 != 0,
            blurred: flags & 8 != 0,
            key: key.map(|key| KeyInput {
                key,
                pressed: flags & 16 != 0,
                repeat: flags & 32 != 0,
                shift: flags & 64 != 0,
            }),
            scroll: if flags & 512 != 0 {[if scroll.is_finite(){scroll}else{0.0},0.0]}else{[0.0,if scroll.is_finite(){scroll}else{0.0}]},
            time,
        };
        let bounds = if matches!(scene, 10006 | 10007 | 10008 | 10009) || (10010..=10013).contains(&scene) {
            Rect {
                x: 0.,
                y: 0.,
                width,
                height,
            }
        } else {
            Rect {
                x: 24.0,
                y: 24.0,
                width: (width - 48.0).max(0.0),
                height: (height - 48.0).max(0.0),
            }
        };
        let disabled = settings & 1 != 0;
        demo.ui.viewport = Some(Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        });
        let variant = match (settings >> 8) & 15 {
            1 => ButtonVariant::Filled,
            2 => ButtonVariant::Tonal,
            3 => ButtonVariant::Outlined,
            4 => ButtonVariant::Text,
            5 => ButtonVariant::Elevated,
            _ => ButtonVariant::Neutral,
        };
        let Demo {
            ui,
            gallery,
            commands,
            clicks,
            check,
            error_check,
            checkbox_error,
            binary,
            switches,
            value,
            range,
            tab,
            radio,
            radio_rtl,
            list,
            showcase,
            android_demo,
            gmail_demo,
            contacts_demo,
            launcher_demo,
            android_apps,
            ..
        } = demo;
        ui.frame_into(input, bounds, 12.0, commands, |f| {
            f.with_enabled(!disabled, |f| match scene {
                0 => {
                    f.label("Small toolkit. Real interactions.");
                    if f.button_with(
                        1,
                        "Summon a ghost",
                        Options {
                            variant: ButtonVariant::Filled,
                            ..Options::default()
                        },
                    )
                    .clicked
                    {
                        *clicks += 1;
                    }
                    f.label(format!("Ghosts summoned: {clicks}"));
                    f.switch(2, "Notifications", &mut switches[0], true);
                    f.checkbox(3, "Three possible states", check, true);
                    f.label(format!("Volume: {:.0}%", value));
                    f.slider(4, "Volume", value, 0.0..=100.0, 1.0, true);
                    f.tabs(5, &["Overview", "Activity", "Settings"], tab, true);
                    f.progress(6, "Volume", Some(*value / 100.0));
                }
                1 => {
                    for (i, (label, v)) in [
                        ("Filled button", ButtonVariant::Filled),
                        ("Tonal button", ButtonVariant::Tonal),
                        ("Outlined button", ButtonVariant::Outlined),
                        ("Text button", ButtonVariant::Text),
                        ("Elevated color treatment", ButtonVariant::Elevated),
                    ]
                    .iter()
                    .enumerate()
                    {
                        if f.button_with(
                            10 + i as u64,
                            label,
                            Options {
                                variant: *v,
                                ..Options::default()
                            },
                        )
                        .clicked
                        {
                            *clicks += 1;
                        }
                    }
                    if f.button_with(
                        20,
                        "Your chosen variant",
                        Options {
                            variant,
                            ..Options::default()
                        },
                    )
                    .clicked
                    {
                        *clicks += 1;
                    }
                    f.label(format!("Activations: {clicks}"));
                    f.button_with(
                        21,
                        "Disabled button",
                        Options {
                            enabled: false,
                            variant: ButtonVariant::Filled,
                            ..Options::default()
                        },
                    );
                }
                2 => {
                    f.label("Material Symbols / accessible action labels");
                    if f.material_icon_button(
                        1,
                        MaterialIcon::Add,
                        "Add item",
                        Options {
                            variant: ButtonVariant::Neutral,
                            ..Options::default()
                        },
                    )
                    .clicked
                    {
                        *clicks += 1;
                    }
                    f.material_icon_button(
                        2,
                        MaterialIcon::Close,
                        "Close",
                        Options {
                            variant: ButtonVariant::Neutral,
                            ..Options::default()
                        },
                    );
                    f.label(format!("Items added: {clicks}"));
                }
                3 => {
                    f.checkbox_bool(1, "Binary selection", binary, true);
                    f.checkbox(2, "Unchecked / checked / mixed", check, true);
                    let mut mixed = CheckState::Mixed;
                    f.checkbox(3, "Disabled mixed state", &mut mixed, false);
                    f.label(format!("Tri-state value: {check:?}"));
                    if f.button(6, "Toggle mixed / checked").clicked {
                        *check = if *check == CheckState::Mixed {
                            CheckState::Checked
                        } else {
                            CheckState::Mixed
                        };
                    }
                    f.label("Android error variant");
                    f.switch(5, "Show error", checkbox_error, true);
                    f.checkbox_bool_with_error(
                        4,
                        "Required choice",
                        error_check,
                        true,
                        *checkbox_error,
                    );
                    if *checkbox_error {
                        f.label("Please select this option.");
                    }
                }
                4 => {
                    f.switch(1, "Notifications", &mut switches[0], true);
                    f.switch_with_icons(2, "Selected icon", &mut switches[1], true,SwitchIconMode::Selected);
                    f.switch_with_icons(3, "Disabled icons", &mut switches[2], false,SwitchIconMode::Both);
                    f.switch_with_icons(4, "Both icons", binary, true,SwitchIconMode::Both);
                    f.label(format!("Notifications: {}", switches[0]));
                }
                5 => {
                    let choices = [
                        Choice {
                            id: 301,
                            label: "Balanced",
                            enabled: true,
                        },
                        Choice {
                            id: 302,
                            label: "Performance",
                            enabled: true,
                        },
                        Choice {
                            id: 303,
                            label: "Unavailable",
                            enabled: false,
                        },
                    ];
                    f.switch(304,"Right-to-left layout",radio_rtl,true);
                    f.radio_group_directional(300, "Power mode", &choices, radio, true,*radio_rtl);
                    f.label(format!("Selected option: {}", *radio - 300));
                }
                6 => {
                    f.label(format!("Volume: {:.0}%", value));
                    f.slider(1, "Volume", value, 0.0..=100.0, 1.0, true);
                    let mut d = 65.0;
                    f.slider(2, "Disabled slider", &mut d, 0.0..=100.0, 5.0, false);
                }
                7 => {
                    f.label(format!("Range: {:.0} – {:.0}", range[0], range[1]));
                    f.range_slider([1, 2], "Range", range, 0.0..=100.0, 1.0, true);
                    f.label("Tab between endpoints. Use arrows or drag.");
                }
                8 => {
                    f.tabs(1, &["Overview", "Activity", "Settings"], tab, true);
                    f.label(
                        [
                            "Overview selected",
                            "Activity selected",
                            "Settings selected",
                        ][*tab],
                    );
                }
                9 => {
                    f.row(48.0, 2, |f, i| {
                        if i == 0 {
                            if f.chip(1, "Filter: installed", *binary, true).clicked {
                                *binary = !*binary;
                            }
                        } else {
                            f.chip(2, "Disabled chip", false, false);
                        }
                    });
                    f.label(format!("Filter selected: {binary}"));
                }
                10 => {
                    f.with_gap(0.0, |f| {
                        f.reserve(8.0);
                        for (i, label) in ["Applications", "Appearance", "Keyboard", "Network"]
                            .iter()
                            .enumerate()
                        {
                            if f.selection_item(i as u64 + 1, label, *tab == i, true)
                                .clicked
                            {
                                *tab = i;
                            }
                        }
                        f.reserve(8.0);
                    });
                }
                11 => {
                    f.label("100,000 entries. Only visible rows are built.");
                    f.virtual_list_padded(1, 280.0, 100_000, list, |f, i, selected| {
                        f.selection_item(i as u64 + 100, "Application", selected, true)
                    });
                    f.label(format!(
                        "Selected: {} / offset: {:.0}px",
                        list.selected.map_or(0, |i| i + 1),
                        list.offset
                    ));
                }
                12 => {
                    f.label("Determinate / value-driven");
                    f.slider(1, "Progress", value, 0.0..=100.0, 1.0, true);
                    f.progress_classic(2, "Download", Some(*value / 100.0));
                    f.label("Indeterminate / visible-only motion");
                    f.progress_classic(3, "Working", None);
                    f.label("Circular / determinate and indeterminate");
                    let row = f.reserve(56.0);
                    for i in 0..2 {
                        f.within(
                            Rect {
                                x: row.x + i as f32 * 68.0,
                                width: 56.0,
                                ..row
                            },
                            |f| {
                                f.progress_circular_classic(
                                    10 + i as u64,
                                    if i == 0 { "Download" } else { "Working" },
                                    if i == 0 { Some(*value / 100.0) } else { None },
                                    56.0,
                                )
                            },
                        );
                    }
                }
                13 => {
                    f.label("Hover for half a second, or focus with Tab.");
                    let r = f.button_with(
                        1,
                        "Hover or focus me",
                        Options {
                            variant: ButtonVariant::Outlined,
                            ..Options::default()
                        },
                    );
                    f.tooltip(2, r, "A small, useful detail");
                }
                14 => {
                    f.switch(150,"Right-to-left layout",&mut switches[1],true);
                    f.label("Localized numeric badges");
                    let mut arabic = !switches[2];
                    f.switch(158,"Arabic-Indic digits",&mut arabic,true);
                    switches[2] = !arabic;
                    if f.button(159,"Increase localized count").clicked {*clicks=(*clicks+1)%4;}
                    let value=998+*clicks;
                    f.label(format!("Count: {value} · character limit: 4"));
                    let format=if arabic { BadgeNumberFormat{digits:["٠","١","٢","٣","٤","٥","٦","٧","٨","٩"],group_separator:"٬",..BadgeNumberFormat::default()} } else { BadgeNumberFormat::default() };
                    let r=f.icon_button(160,"","Localized inbox",Options::default());
                    let icon=Rect{x:r.bounds.x+8.0,y:r.bounds.y+8.0,width:24.0,height:24.0};
                    f.paint_material_icon(icon,MaterialIcon::Inbox,f.theme().on_surface_variant);
                    f.badge_at(160,icon,BadgeContent::FormattedNumber{value,limit:BadgeNumberLimit::MaxCharacters(4),format},"",switches[1]);
                    f.label("Grouped number: 1234");
                    let r=f.icon_button(161,"","Grouped inbox",Options::default());
                    let icon=Rect{x:r.bounds.x+8.0,y:r.bounds.y+8.0,width:24.0,height:24.0};
                    f.paint_material_icon(icon,MaterialIcon::Inbox,f.theme().on_surface_variant);
                    f.badge_at(161,icon,BadgeContent::FormattedNumber{value:1234,limit:BadgeNumberLimit::Unlimited,format},"",switches[1]);
                    f.label("Badge anchor corner");
                    if f.button(155,"Change badge corner").clicked {*radio=(*radio+1)%4;}
                    let (gravity,name)=match *radio%4 {1=>(BadgeGravity::TopEnd,"Top end"),2=>(BadgeGravity::BottomEnd,"Bottom end"),3=>(BadgeGravity::BottomStart,"Bottom start"),_=>(BadgeGravity::TopStart,"Top start")};
                    f.label(format!("Corner: {name}"));
                    f.switch(157,"Move badge inward",binary,true);
                    let r=f.icon_button(156,"","Corner inbox",Options::default());
                    let icon=Rect{x:r.bounds.x+8.0,y:r.bounds.y+8.0,width:24.0,height:24.0};
                    f.paint_material_icon(icon,MaterialIcon::Inbox,f.theme().on_surface_variant);
                    let offset=if *binary {6.0}else{0.0};
                    f.badge_at_with_options(156,icon,BadgeContent::Label("3"),"",BadgeOptions{gravity,rtl:switches[1],horizontal_offset:offset,vertical_offset:offset});
                    f.label("Text badge overflow");
                    if f.button(153,"Change badge text").clicked {switches[0]=!switches[0];}
                    let text=if switches[0] {"New"}else{"Updates"};
                    f.label(format!("Text: {text} · maximum: 4"));
                    let r=f.icon_button(154,"","Text inbox",Options::default());
                    let icon=Rect{x:r.bounds.x+8.0,y:r.bounds.y+8.0,width:24.0,height:24.0};
                    f.paint_material_icon(icon,MaterialIcon::Inbox,f.theme().on_surface_variant);
                    f.badge_at(154,icon,BadgeContent::Text{text,max_graphemes:4},"",switches[1]);
                    f.label("Numeric badge overflow");
                    if f.button(151,"Increase count").clicked {*clicks=(*clicks+1)%4;}
                    let count=998+*clicks;
                    f.label(format!("Count: {count} · maximum: 999"));
                    let r=f.icon_button(152,"","Numbered inbox",Options::default());
                    let icon=Rect{x:r.bounds.x+8.0,y:r.bounds.y+8.0,width:24.0,height:24.0};
                    f.paint_material_icon(icon,MaterialIcon::Inbox,f.theme().on_surface_variant);
                    f.badge_at(152,icon,BadgeContent::Number{value:count,max:999},"",switches[1]);
                    f.label("Small badge · unread notification");
                    let r = f.icon_button(140, "", "Notifications", Options::default());
                    let icon = Rect {
                        x: r.bounds.x + 8.0,
                        y: r.bounds.y + 8.0,
                        width: 24.0,
                        height: 24.0,
                    };
                    f.paint_material_icon(icon, MaterialIcon::Inbox, f.theme().on_surface_variant);
                    f.badge_at(140, icon, BadgeContent::Dot, "New notification", false);
                    f.label("Large badges · counts and labels");
                    for (i, value) in ["3", "12", "999+", "New"].iter().enumerate() {
                        let id = 141 + i as u64;
                        let r = f.icon_button(id, "", "Messages", Options::default());
                        let icon = Rect {
                            x: r.bounds.x + 8.0,
                            y: r.bounds.y + 8.0,
                            width: 24.0,
                            height: 24.0,
                        };
                        f.paint_material_icon(icon, MaterialIcon::Inbox, f.theme().on_surface_variant);
                        f.badge_at(id, icon, BadgeContent::Label(value), value, false);
                    }
                }
                15 => {
                f.switch(70,"Right-to-left layout",&mut switches[1],true);
                f.label("Full-width divider");
                let b=f.reserve(24.0);f.divider_styled_at(b,bhoo::DividerOptions::default());
                f.label("Inset divider: 72 dp start, 16 dp end");
                let b=f.reserve(24.0);f.divider_styled_at(b,bhoo::DividerOptions {inset_start:72.0,inset_end:16.0,rtl:switches[1],..bhoo::DividerOptions::default()});
                f.label("Heavy divider: 8 dp");
                let b=f.reserve(24.0);f.divider_styled_at(b,bhoo::DividerOptions {thickness:8.0,..bhoo::DividerOptions::default()});
                f.label("Vertical divider");
                let b=f.reserve(80.0);f.divider_styled_at(Rect{x:b.x,y:b.y,width:24.0,height:b.height},bhoo::DividerOptions {orientation:bhoo::DividerOrientation::Vertical,inset_start:8.0,inset_end:16.0,..bhoo::DividerOptions::default()});
                f.label("Android XML: Material heavy style with insets");
                let b=f.reserve(24.0);let mut state=bhoo::android_xml::LayoutState::default();state.rtl=switches[1];
                android_material_divider::LAYOUT.render(f,b,&mut state);
            }
                16 => {
                    for (id, label) in [(1, "First action"), (2, "Second action")] {
                        f.button_with(
                            id,
                            label,
                            Options {
                                variant: ButtonVariant::Filled,
                                ..Options::default()
                            },
                        );
                    }
                }
                17 => {
                    f.label("Bhoo / interface text");
                    f.label("System typography and Unicode: SpectreOS — روح");
                }
                18 => {
                    f.label("Touch or wheel to scroll this viewport");
                    let area = f.reserve(200.);
                    f.scroll_viewport(900, area, &mut list.offset, 6. * 52. - area.height);
                    f.within(area, |f| {
                        for i in 0..6 {
                            f.action_at(
                                i + 1,
                                Rect {
                                    x: area.x,
                                    y: area.y + i as f32 * 52. - list.offset,
                                    width: area.width,
                                    height: 40.,
                                },
                                "Scrollable content",
                                Options::default(),
                            );
                        }
                    });
                }
                10006 => android_demo.show(f, width, height, &mut gallery.text),
                10007 => gmail_demo.show(f, width, height, &mut gallery.text),
                10008 => contacts_demo.show(f, width, height),
                10009 => launcher_demo.show(f, width, height, input, &mut gallery.text),
                10010..=10013 => {
                    android_apps.show(f, width, height, (scene - 10010) as usize);
                    gallery
                        .text
                        .clone_from(&android_apps.fields[android_apps.editing]);
                }
                10000..=10005 => showcase.show(f, scene, width, height, input),
                _ => gallery.show(f, scene),
            });
        });
        if matches!(scene, 10001 | 10004)
            && let Some(DrawCommand::Clear(color)) = demo.commands.first_mut()
        {
            *color = Color(0, 0, 0, 0);
        }
        encode(demo);
        demo.packet.as_ptr() as u32
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_packet_len() -> u32 {
    DEMO.with_borrow(|d| d.packet.len() as u32)
}

fn u32v(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn f32v(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn rect(out: &mut Vec<u8>, b: Rect) {
    for v in [b.x, b.y, b.width, b.height] {
        f32v(out, v);
    }
}
fn color(out: &mut Vec<u8>, c: Color) {
    out.extend_from_slice(&[c.0, c.1, c.2, c.3]);
}
fn string(out: &mut Vec<u8>, s: &str) {
    u32v(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
}
fn encode(demo: &mut Demo) {
    let Demo {
        ui,
        commands,
        packet,
        scene,
        ..
    } = demo;
    packet.clear();
    u32v(packet, 0x42484f4f);
    u32v(packet, 1);
    u32v(packet, commands.len() as u32);
    u32v(packet, u32::from(ui.needs_redraw()));
    packet.extend_from_slice(&ui.next_wakeup().unwrap_or(-1.0).to_le_bytes());
    u32v(packet, ui.semantics().len() as u32);
    u32v(packet, *scene);
    for command in commands {
        match command {
            DrawCommand::Clear(c) => {
                u32v(packet, 0);
                color(packet, *c);
            }
            DrawCommand::Outline { bounds,radius,width,color:c } => {
                u32v(packet,19);rect(packet,*bounds);f32v(packet,*radius);f32v(packet,*width);color(packet,*c);
            }
            DrawCommand::Ellipse { bounds, color: c } => {
                u32v(packet,18);
                rect(packet,*bounds);
                color(packet,*c);
            }
            DrawCommand::RoundedRect {
                bounds,
                radius,
                color: c,
            } => {
                u32v(packet, 1);
                rect(packet, *bounds);
                f32v(packet, *radius);
                color(packet, *c);
            }
            DrawCommand::Text {
                bounds,
                text,
                color: c,
                style,
            } => {
                u32v(packet, 9);
                rect(packet, *bounds);
                color(packet, *c);
                f32v(packet, style.size);
                let family_bits = match style.family {
                    FontFamily::Sans => 0_u32,
                    FontFamily::Serif => 1_u32,
                    FontFamily::Monospace => 2_u32,
                };
                u32v(
                    packet,
                    u32::from(style.weight)
                        | (u32::from(style.align) << 16)
                        | (u32::from(style.italic) << 24)
                        | (family_bits << 25),
                );
                f32v(packet, style.padding);
                f32v(packet, style.letter_spacing);
                string(packet, text);
            }
            DrawCommand::Shadow {
                bounds,
                radius,
                offset,
                blur,
                spread,
                color: c,
            } => {
                u32v(packet, 10);
                rect(packet, *bounds);
                f32v(packet, *radius);
                f32v(packet, offset[0]);
                f32v(packet, offset[1]);
                f32v(packet, *blur);
                f32v(packet, *spread);
                color(packet, *c);
            }
            DrawCommand::Ripple {
                bounds,
                corner_radius,
                center,
                radius,
                color: c,
                soft,
            } => {
                u32v(packet, if *soft { 5 } else { 11 });
                rect(packet, *bounds);
                f32v(packet, *corner_radius);
                f32v(packet, center[0]);
                f32v(packet, center[1]);
                f32v(packet, *radius);
                color(packet, *c);
            }
            DrawCommand::Line {
                from,
                to,
                width,
                color: c,
            } => {
                u32v(packet, 8);
                for v in [from[0], from[1], to[0], to[1], *width] {
                    f32v(packet, v);
                }
                color(packet, *c);
            }
            DrawCommand::Polyline {
                points,
                count,
                width,
                color: c,
            } => {
                u32v(packet, 17);
                for p in points {
                    f32v(packet, p[0]);
                    f32v(packet, p[1]);
                }
                u32v(packet, u32::from(*count));
                f32v(packet, *width);
                color(packet, *c);
            }
            DrawCommand::Arc {
                bounds,
                start,
                sweep,
                stroke,
                color: c,
                rounded,
            } => {
                u32v(packet, if *rounded { 7 } else { 12 });
                rect(packet, *bounds);
                f32v(packet, *start);
                f32v(packet, *sweep);
                f32v(packet, *stroke);
                color(packet, *c);
            }
            DrawCommand::Loading {
                bounds,
                sample,
                color: c,
            } => {
                u32v(packet, 6);
                color(packet, *c);
                u32v(
                    packet,
                    bhoo::loading::loading_curve_count(sample.shape) as u32,
                );
                bhoo::loading::loading_cubics(*bounds, *sample, |curve| {
                    for p in curve {
                        f32v(packet, p);
                    }
                });
            }
            DrawCommand::Image {
                bounds,
                uv,
                radius,
                color: c,
                image,
            } => {
                u32v(packet, 16);
                rect(packet, *bounds);
                for p in uv {
                    f32v(packet, *p);
                }
                f32v(packet, *radius);
                color(packet, *c);
                u32v(packet, image.id());
                u32v(packet, image.width());
                u32v(packet, image.height());
                u32v(packet, image.pixels().as_ptr() as usize as u32);
            }
            DrawCommand::Shape {
                bounds,
                index,
                color: c,
                rotation,
            } => {
                let Some(shape) = bhoo::shape_data(*index as usize) else {
                    continue;
                };
                u32v(packet, 15);
                color(packet, *c);
                rect(packet, bhoo::shape_clip_bounds(*bounds, *rotation));
                u32v(packet, shape.contours.len().saturating_sub(1) as u32);
                let size = bounds.width.min(bounds.height);
                let x = bounds.x + (bounds.width - size) / 2.0;
                let y = bounds.y + (bounds.height - size) / 2.0;
                let (sin, cos) = rotation.sin_cos();
                for contour in shape.contours.windows(2) {
                    let points = &shape.points[contour[0] as usize..contour[1] as usize];
                    u32v(packet, points.len() as u32);
                    for p in points {
                        let px = p[0] - 0.5;
                        let py = p[1] - 0.5;
                        f32v(packet, x + (0.5 + px * cos - py * sin) * size);
                        f32v(packet, y + (0.5 + px * sin + py * cos) * size);
                    }
                }
            }
            DrawCommand::PushClip(b) => {
                u32v(packet, 3);
                rect(packet, *b);
            }
            DrawCommand::PopClip => u32v(packet, 4),
            DrawCommand::PushScaleY { origin, scale } => {
                u32v(packet, 13);
                f32v(packet, *origin);
                f32v(packet, *scale);
            }
            DrawCommand::PopScaleY => u32v(packet, 14),
            DrawCommand::PushTransform {
                origin,
                translate,
                scale,
                rotation_rad,
            } => {
                u32v(packet, 20);
                for v in [
                    origin[0],
                    origin[1],
                    translate[0],
                    translate[1],
                    scale[0],
                    scale[1],
                    *rotation_rad,
                ] {
                    f32v(packet, v);
                }
            }
            DrawCommand::PopTransform => u32v(packet, 21),
            DrawCommand::PushLayer { bounds, alpha } => {
                u32v(packet, 22);
                rect(packet, *bounds);
                f32v(packet, *alpha);
            }
            DrawCommand::PopLayer => u32v(packet, 23),
        }
    }
    for node in ui.semantics() {
        packet.extend_from_slice(&node.id.to_le_bytes());
        u32v(packet, node.part);
        u32v(packet, node.role as u32);
        rect(packet, node.bounds);
        u32v(
            packet,
            u32::from(node.enabled)
                | (u32::from(node.focused) << 1)
                | (u32::from(node.selected) << 2)
                | (u32::from(node.invalid) << 3)
                | (u32::from(node.filled) << 4)
                | (u32::from(node.required) << 5),
        );
        u32v(
            packet,
            match node.checked {
                None => 0,
                Some(CheckState::Unchecked) => 1,
                Some(CheckState::Checked) => 2,
                Some(CheckState::Mixed) => 3,
            },
        );
        f32v(packet, node.value.unwrap_or(f32::NAN));
        let range = node.range.unwrap_or([f32::NAN; 2]);
        f32v(packet, range[0]);
        f32v(packet, range[1]);
        string(packet, &node.label);
    }
}

/// Browser editing adapter: bounded UTF-8 buffer, no imported JavaScript functions.
/// Supply a browser wall-clock snapshot independently of the monotonic frame clock.
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_wall_clock(unix_seconds:f64, offset_seconds:i32, is_24_hour:u32) {
    DEMO.with_borrow_mut(|d| d.gallery.set_android_wall_clock(unix_seconds, offset_seconds, is_24_hour != 0));
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_calendar_locale_commit(length:u32) {DEMO.with_borrow_mut(|d|{let len=(length as usize).min(d.gallery.edit.len());if let Ok(data)=std::str::from_utf8(&d.gallery.edit[..len]) {let data=data.to_owned();d.gallery.set_android_calendar_locale(&data);}});}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_edit_buffer() -> u32 {
    DEMO.with_borrow_mut(|d| d.gallery.edit.as_mut_ptr() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_edit_commit(length: u32) {
    DEMO.with_borrow_mut(|d| {
        let length = (length as usize).min(d.gallery.edit.len());
        if let Ok(text) = std::str::from_utf8(&d.gallery.edit[..length]) {
            let text = text.to_owned();
            d.gallery.text.clone_from(&text);
            if (10010..=10013).contains(&d.scene) {
                d.android_apps.edit(&text);
            }
            if gallery::FAMILIES
                .lines()
                .nth(d.scene.saturating_sub(19) as usize)
                == Some("date-range-picker")
            {
                d.gallery.update_date_range_text(&text);
            }
        }
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_search_take_focus(id:u64)->u32 {DEMO.with_borrow_mut(|d|u32::from(d.gallery.take_android_search_focus(id)))}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_search_submit(id:u64)->u32 {DEMO.with_borrow_mut(|d|u32::from(d.gallery.submit_android_search(id)))}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_number_end(id:u64,commit:u32) {DEMO.with_borrow_mut(|d|d.gallery.finish_android_number_edit(id,commit!=0));}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_autocomplete_highlighted(id:u64)->u32 {DEMO.with_borrow(|d|u32::from(d.gallery.android_autocomplete_highlighted(id)))}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_autocomplete_open(id:u64)->u32 {DEMO.with_borrow(|d|u32::from(d.gallery.android_autocomplete_open(id)))}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_edit_selection(id:u64,end:u32) {DEMO.with_borrow_mut(|d|d.gallery.set_android_edit_selection(id,end as usize));}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_selection_end(id:u64)->u32 {DEMO.with_borrow(|d|d.gallery.android_edit_selection(id) as u32)}
/// Commit one named XML EditText independently of other fields.
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_edit_commit(id: u64, length: u32) {
    DEMO.with_borrow_mut(|d| {
        let length = (length as usize).min(d.gallery.edit.len());
        if let Ok(text) = std::str::from_utf8(&d.gallery.edit[..length]) {
            let text = text.to_owned();
            if d.scene == 10007 {
                d.gmail_demo.set_edit_text(id, &text, &mut d.gallery.text);
            } else if d.scene == 10008 {
                d.contacts_demo.set_edit_text(id, &text);
            } else if d.scene == 10009 {
                d.launcher_demo.set_edit_text(id, &text);
            } else {
                d.gallery.set_android_edit_text(id, &text);
            }
        }
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_launcher_key(code: u32, time: f64) {
    DEMO.with_borrow_mut(|d| {
        if d.scene == 10009 {
            d.launcher_demo.system_key(code, time);
        }
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_input_type(id: u64) -> u32 {
    DEMO.with_borrow(|d| u32::from(d.gallery.android_edit_input_type(id)))
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_text_pointer(id: u64) -> u32 {
    DEMO.with_borrow(|d| {
        if d.scene == 10007 {
            d.gmail_demo.edit_text_for(id).as_ptr() as u32
        } else if d.scene == 10008 {
            d.contacts_demo.edit_text_for(id).as_ptr() as u32
        } else if d.scene == 10009 {
            d.launcher_demo.edit_text_for(id).as_ptr() as u32
        } else {
            d.gallery.android_edit_text(id).as_ptr() as u32
        }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_xml_text_len(id: u64) -> u32 {
    DEMO.with_borrow(|d| {
        if d.scene == 10007 {
            d.gmail_demo.edit_text_for(id).len() as u32
        } else if d.scene == 10008 {
            d.contacts_demo.edit_text_for(id).len() as u32
        } else if d.scene == 10009 {
            d.launcher_demo.edit_text_for(id).len() as u32
        } else {
            d.gallery.android_edit_text(id).len() as u32
        }
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_text_pointer() -> u32 {
    DEMO.with_borrow(|d| d.gallery.text.as_ptr() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_text_len() -> u32 {
    DEMO.with_borrow(|d| d.gallery.text.len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_overlay_open() -> u32 {
    DEMO.with_borrow(|d| {
        let is_fab_menu = gallery::FAMILIES
            .lines()
            .nth(d.scene.saturating_sub(19) as usize)
            == Some("floating-action-menu")
            && d.scene >= 19;
        u32::from(
            d.gallery.overlay_open()
                || d.ui.has_escape_target()
                || (is_fab_menu && d.gallery.fab_menu_open()),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_pause() {
    DEMO.with_borrow_mut(|d| {
        d.gallery.pause();
        d.ui.pause_animations();
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_palette_tick(time: f64, reduced: u32) {
    sample_palette(time, reduced != 0);
}

#[cfg(test)]
mod palette_tests {
    use super::*;
    #[test]
    fn palette_effect_interpolates_and_retargets_from_presented_color() {
        PALETTE_MOTION.with_borrow_mut(|p| *p = PaletteMotion::default());
        REFERENCE_PALETTE.with_borrow_mut(|p| *p = [None; 22]);
        bhoo_palette_set(0, u32::from_le_bytes([255, 255, 255, 255]));
        sample_palette(0., false);
        bhoo_palette_set(0, u32::from_le_bytes([0, 0, 0, 255]));
        sample_palette(1., false);
        let middle = sample_palette(1.1, false)[0].unwrap();
        assert!(middle.0 > 0 && middle.0 < 255);
        bhoo_palette_set(0, u32::from_le_bytes([255, 255, 255, 255]));
        assert_eq!(sample_palette(1.1, false)[0], Some(middle));
        assert_eq!(
            sample_palette(1.5, false)[0],
            Some(Color(255, 255, 255, 255))
        );
        bhoo_palette_set(0, u32::from_le_bytes([0, 0, 0, 255]));
        assert_eq!(sample_palette(1.6, true)[0], Some(Color(0, 0, 0, 255)));
    }
}

/// Smooth nested scroll surfaces without changing pointer drag behavior.
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_local_scroll(delta: f32, time: f64, reduced: u32, active: u32) -> f32 {
    DEMO.with_borrow_mut(|d| {
        let dt = (time - d.local_scroll_time).clamp(0.001, 0.05) as f32;
        d.local_scroll_time = time;
        if active == 0 {
            d.local_scroll_remaining = 0.;
            return 0.;
        }
        if delta.is_finite() {
            d.local_scroll_remaining += delta;
        }
        let step = if reduced != 0 || d.local_scroll_remaining.abs() < 0.1 {
            d.local_scroll_remaining
        } else {
            d.local_scroll_remaining * (1. - (-18. * dt).exp())
        };
        d.local_scroll_remaining -= step;
        step
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_local_scroll_moving() -> u32 {
    DEMO.with_borrow(|d| u32::from(d.local_scroll_remaining != 0.))
}

#[cfg(test)]
mod smooth_scroll_tests {
    use super::*;
    #[test]
    fn nested_scroll_conserves_distance_and_cancels_on_pointer_exit() {
        bhoo_reset(1);
        let first = bhoo_local_scroll(240., 0.016, 0, 1);
        assert!(first > 0. && first < 240.);
        let mut total = first;
        for i in 2..180 {
            total += bhoo_local_scroll(0., i as f64 / 60., 0, 1);
        }
        assert!((total - 240.).abs() < 0.01);
        assert_eq!(bhoo_local_scroll_moving(), 0);
        bhoo_local_scroll(120., 4., 0, 1);
        assert_eq!(bhoo_local_scroll(0., 4.01, 0, 0), 0.);
        assert_eq!(bhoo_local_scroll_moving(), 0);
        assert_eq!(bhoo_local_scroll(120., 4.02, 1, 1), 120.);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn bhoo_app_focus(id: u64) {
    DEMO.with_borrow_mut(|d| {
        d.gallery.text = d.android_apps.focus(id);
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_app_revision() -> u32 {
    DEMO.with_borrow(|d| d.android_apps.revision)
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_app_snapshot() -> u32 {
    DEMO.with_borrow_mut(|d| {
        d.app_snapshot = d.android_apps.save().into_bytes();
        d.app_snapshot.as_ptr() as u32
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_app_snapshot_len() -> u32 {
    DEMO.with_borrow(|d| d.app_snapshot.len() as u32)
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_app_restore_buffer() -> u32 {
    DEMO.with_borrow_mut(|d| {
        d.app_restore.resize(262144, 0);
        d.app_restore.as_mut_ptr() as u32
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn bhoo_app_restore(length: u32) -> u32 {
    DEMO.with_borrow_mut(|d| {
        let Ok(s) =
            std::str::from_utf8(&d.app_restore[..(length as usize).min(d.app_restore.len())])
        else {
            return 0;
        };
        u32::from(
            d.android_apps
                .restore(d.scene.saturating_sub(10010) as usize, s),
        )
    })
}
