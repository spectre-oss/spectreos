//! Native Wayland + GLES capture for the pure Android XML Contacts screens
//! (`android_contacts_{home,detail,editor,settings}.rs`) under `AndroidTheme::MaterialLight`
//! at `540 x 1110` (`density = 1.5`, matching the `1440 x 2960` aspect ratio of `context/contacts-flow/`).
//!
//! Select screen via `BHOO_CONTACTS_SCREEN=home|empty|drawer|detail|editor|settings` (default: `home`).
//! Pass `--smoke-test` with `BHOO_CAPTURE=/path/to/out.ppm` to render and capture a frame.
use bhoo::{
    android_xml::{AndroidTheme, LayoutState, Node, Visibility},
    window::{self, FrameAction, WindowOptions},
    *,
};

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/android_contacts_home.rs"]
mod home;

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/android_contacts_detail.rs"]
mod detail;

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/android_contacts_editor.rs"]
mod editor;

#[allow(dead_code)]
#[path = "../../../../apps/bhoo-web/src/android_contacts_settings.rs"]
mod settings;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Empty,
    Drawer,
    Detail,
    Editor,
    Settings,
    SettingsDialog,
}

fn select_screen() -> Screen {
    match std::env::var("BHOO_CONTACTS_SCREEN")
        .ok()
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("empty") => Screen::Empty,
        Some("drawer") => Screen::Drawer,
        Some("detail") => Screen::Detail,
        Some("editor") => Screen::Editor,
        Some("settings") => Screen::Settings,
        Some("dialog") => Screen::SettingsDialog,
        _ => Screen::Home,
    }
}

fn vis(show: bool) -> Visibility {
    if show {
        Visibility::Visible
    } else {
        Visibility::Gone
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|x| x == "--smoke-test");
    let screen = select_screen();

    let mut state = LayoutState::default();
    state.theme_style = AndroidTheme::MaterialLight;
    state.density = 1.5;

    home::install_drawables(&mut state);
    detail::install_drawables(&mut state);
    editor::install_drawables(&mut state);
    settings::install_drawables(&mut state);

    // Home / Empty / Drawer state
    state.visibility.insert("home_toolbar".into(), Visibility::Visible);
    state.visibility.insert("search_toolbar".into(), Visibility::Gone);
    state.visibility.insert("cab_toolbar".into(), Visibility::Gone);
    state.visibility.insert("label_toolbar".into(), Visibility::Gone);
    state.visibility.insert("label_count_row".into(), Visibility::Gone);
    state.visibility.insert("search_empty_state".into(), Visibility::Gone);
    state.visibility.insert("label_empty_state".into(), Visibility::Gone);
    state.visibility.insert("label_overflow_popup".into(), Visibility::Gone);
    state.visibility.insert("home_dialog_scrim".into(), Visibility::Gone);
    state.visibility.insert("empty_state".into(), vis(screen == Screen::Empty));
    state.visibility.insert("contacts_scroll".into(), vis(screen != Screen::Empty));
    state.visibility.insert("fab_add".into(), Visibility::Visible);
    state.drawer_open.insert("drawer_layout".into(), screen == Screen::Drawer);
    state.image_sources.insert("drawer_contacts_bg".into(), "drawer_row_active".into());
    state.image_sources.insert("drawer_contacts_icon".into(), "ic_contacts_blue".into());
    state.visibility.insert("drawer_label_0".into(), Visibility::Visible);
    state.text.insert("drawer_label_text_0".into(), "Work".into());
    state.image_sources.insert("drawer_label_bg_0".into(), "drawer_row_inactive".into());
    state.image_sources.insert("drawer_label_icon_0".into(), "ic_label_muted".into());
    for i in 1..4 {
        state.visibility.insert(format!("drawer_label_{i}"), Visibility::Gone);
    }

    let sample_rows = [
        (true, "", 0usize, "A", "Alice Johnson"),
        (false, "A", 0usize, "A", "Alice Johnson"),
        (false, "B", 1usize, "B", "Bob Smith"),
        (false, "C", 2usize, "C", "Charlie Lee"),
    ];
    for i in 0..10 {
        if screen != Screen::Empty && let Some(&(star, letter, color, init, name)) = sample_rows.get(i) {
            state.visibility.insert(format!("contact_row_{i}"), Visibility::Visible);
            state.visibility.insert(format!("row_star_{i}"), vis(star));
            state.visibility.insert(format!("row_letter_{i}"), vis(!letter.is_empty()));
            state.text.insert(format!("row_letter_{i}"), letter.into());
            state.image_sources.insert(format!("row_avatar_bg_{i}"), format!("avatar_circle_{color}"));
            state.text.insert(format!("row_avatar_letter_{i}"), init.into());
            state.text.insert(format!("row_name_{i}"), name.into());
            state.visibility.insert(format!("row_check_{i}"), Visibility::Gone);
        } else {
            state.visibility.insert(format!("contact_row_{i}"), Visibility::Gone);
        }
    }

    // Detail state (Alice Johnson)
    state.image_sources.insert("detail_hero_bg".into(), "hero_bg_0".into());
    state.image_sources.insert("detail_star".into(), "ic_star_filled_white".into());
    state.text.insert("detail_name".into(), "Alice Johnson".into());
    state.text.insert("detail_phone".into(), "+1 555-010-1234".into());
    state.text.insert("detail_phone_type".into(), "Mobile".into());
    state.text.insert("detail_email".into(), "alice@example.com".into());
    state.text.insert("detail_email_type".into(), "Home".into());
    state.text.insert("detail_about_title".into(), "About Alice".into());
    state.text.insert("detail_org".into(), "Acme Corp".into());
    state.visibility.insert("detail_notes_row".into(), Visibility::Gone);
    state.visibility.insert("detail_labels_row".into(), Visibility::Visible);
    state.text.insert("detail_labels".into(), "Work".into());
    state.visibility.insert("detail_ringtone_row".into(), Visibility::Gone);
    state.visibility.insert("detail_overflow_popup".into(), Visibility::Gone);
    state.visibility.insert("detail_dialog_scrim".into(), Visibility::Gone);

    // Editor state
    state.text.insert("editor_title".into(), "Create new contact".into());
    state.text.insert("editor_saving_to".into(), "Device".into());
    state.text.insert("ed_first_name".into(), "Alice".into());
    state.text.insert("ed_last_name".into(), "Johnson".into());
    state.text.insert("ed_phone".into(), "+1 555-010-1234".into());
    state.text.insert("ed_email".into(), "alice@example.com".into());
    state.text.insert("editor_phone_type_text".into(), "Mobile".into());
    state.text.insert("editor_email_type_text".into(), "Home".into());
    state.visibility.insert("editor_prefix_wrap".into(), Visibility::Gone);
    state.visibility.insert("editor_middle_wrap".into(), Visibility::Gone);
    state.visibility.insert("editor_suffix_wrap".into(), Visibility::Gone);
    state.visibility.insert("editor_extra_fields".into(), Visibility::Gone);
    state.visibility.insert("editor_more_fields_btn".into(), Visibility::Visible);
    state.visibility.insert("editor_dialog_scrim".into(), Visibility::Gone);

    // Settings state
    state.text.insert("settings_title".into(), "Settings".into());
    state.visibility.insert("settings_main_list".into(), Visibility::Visible);
    state.visibility.insert("settings_about_view".into(), Visibility::Gone);
    state.visibility.insert("settings_emergency_view".into(), Visibility::Gone);
    state.visibility.insert("settings_blocked_view".into(), Visibility::Gone);
    state.visibility.insert("settings_dialog_scrim".into(), vis(screen == Screen::SettingsDialog));
    if screen == Screen::SettingsDialog {
        state.text.insert("settings_dialog_title".into(), "Import contacts from".into());
        state.visibility.insert("set_opt_0".into(), Visibility::Visible);
        state.text.insert("set_opt_0".into(), ".vcf file".into());
        state.checked.insert("set_opt_0".into(), true);
        state.visibility.insert("set_opt_1".into(), Visibility::Visible);
        state.text.insert("set_opt_1".into(), "SIM card".into());
        state.checked.insert("set_opt_1".into(), false);
        state.visibility.insert("set_opt_2".into(), Visibility::Gone);
    }

    let (title, layout): (&'static str, &'static Node) = match screen {
        Screen::Home | Screen::Empty | Screen::Drawer => ("Bhoo / Contacts XML (Home)", &home::LAYOUT),
        Screen::Detail => ("Bhoo / Contacts XML (Detail)", &detail::LAYOUT),
        Screen::Editor => ("Bhoo / Contacts XML (Editor)", &editor::LAYOUT),
        Screen::Settings | Screen::SettingsDialog => ("Bhoo / Contacts XML (Settings)", &settings::LAYOUT),
    };

    let mut frames = 0;
    window::run(
        WindowOptions {
            title: title.into(),
            size: [540, 1110],
            capture: std::env::var_os("BHOO_CAPTURE").map(Into::into),
            ..WindowOptions::default()
        },
        move |f| {
            frames += 1;
            layout.render(
                f,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 540.0,
                    height: 1110.0,
                },
                &mut state,
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

