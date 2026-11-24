//! Live built-in Android 13 / LineageOS 20 apps & Overview task-card preview renderers.
//! Includes Dark Clock (matching `06-split-screen-50-50.png`), working Calculator, Phone dialer,
//! interactive Messaging/ArcaneChat/DeltaChat/Telegram, Browser, Gallery, Music, Camera,
//! Calendar, Files, F-Droid, and Recorder.

use super::{
    AppId, ID_BROWSER_URL, ID_CHAT_COMPOSER, Palette, ShellAction, SysIcon, SystemState,
     paint_switch, paint_sys_icon, text_style,
};
use bhoo::*;

#[derive(Clone)]
struct AlarmItem {
    hour: u8,
    minute: u8,
    am: bool,
    days: String,
    enabled: bool,
}

#[derive(Clone)]
struct ChatMessage {
    from_me: bool,
    text: String,
    time: &'static str,
}

pub struct BuiltInApps {
    // Clock
    clock_tab: usize, // 0=Alarm, 1=Clock, 2=Timer, 3=Stopwatch
    alarms: Vec<AlarmItem>,
    timer_secs: f64,
    timer_running: bool,
    timer_last: f64,
    sw_elapsed: f64,
    sw_running: bool,
    sw_last: f64,
    sw_laps: Vec<f64>,

    // Calculator
    calc_display: String,
    calc_acc: Option<f64>,
    calc_op: Option<char>,
    calc_fresh: bool,

    // Phone
    phone_tab: usize,
    phone_dialpad_open: bool,
    phone_muted: bool,
    phone_speaker: bool,
    dial_num: String,
    in_call: Option<(String, f64)>,

    // Chat apps
    chat_thread: Option<usize>,
    chat_input: String,
    threads: Vec<(&'static str, Vec<ChatMessage>)>,

    // Browser
    browser_url: String,
    browser_page: usize,

    // Gallery
    gallery_open: Option<usize>,

    // Music
    music_track: usize,
    music_playing: bool,
    music_progress: f32,
    music_last: f64,
    music_fav: [bool; 3],

    // Camera
    camera_mode: usize,
    camera_shots: u32,
    camera_flash_until: f64,

    // Calendar
    calendar: spectre_calendar::CalendarApp,

    // Files
    files: spectre_files::Browser,

    // F-Droid
    fdroid_installed: [bool; 5],

    // Recorder
    rec_active: bool,
    rec_elapsed: f64,
    rec_last: f64,
    rec_saved: Vec<String>,
}

impl Default for BuiltInApps {
    fn default() -> Self {
        Self {
            clock_tab: 0,
            alarms: vec![
                AlarmItem {
                    hour: 8,
                    minute: 30,
                    am: true,
                    days: "Mon, Tue, Wed, Thu, Fri".into(),
                    enabled: false,
                },
                AlarmItem {
                    hour: 9,
                    minute: 0,
                    am: true,
                    days: "Sun, Sat".into(),
                    enabled: false,
                },
            ],
            timer_secs: 300.0,
            timer_running: false,
            timer_last: 0.0,
            sw_elapsed: 0.0,
            sw_running: false,
            sw_last: 0.0,
            sw_laps: Vec::new(),
            calc_display: "0".into(),
            calc_acc: None,
            calc_op: None,
            calc_fresh: true,
            phone_tab: 0,
            phone_dialpad_open: false,
            phone_muted: false,
            phone_speaker: false,
            dial_num: String::new(),
            in_call: None,
            chat_thread: None,
            chat_input: String::new(),
            threads: vec![
                (
                    "Alice Johnson",
                    vec![
                        ChatMessage {
                            from_me: false,
                            text: "Hey! Are we still meeting at 3 PM to review the SpectreOS System UI layout and animations?".into(),
                            time: "1:12 PM",
                        },
                        ChatMessage {
                            from_me: true,
                            text: "Yes! Split-screen and Freeform windows are already running live in libbhoo.".into(),
                            time: "1:14 PM",
                        },
                    ],
                ),
                (
                    "SpectreOS Core",
                    vec![ChatMessage {
                        from_me: false,
                        text: "bhoo-demo-bundle.zip passed all GLES visual parity checks.".into(),
                        time: "11:40 AM",
                    }],
                ),
                (
                    "Bob Miller",
                    vec![ChatMessage {
                        from_me: false,
                        text: "Let's test Contacts + Clock in 50/50 split screen later!".into(),
                        time: "Yesterday",
                    }],
                ),
            ],
            browser_url: "https://spectreos.local/bhoo-guide".into(),
            browser_page: 0,
            gallery_open: None,
            music_track: 0,
            music_playing: false,
            music_progress: 0.34,
            music_last: 0.0,
            music_fav: [true, false, false],
            camera_mode: 1,
            camera_shots: 6,
            camera_flash_until: 0.0,
            calendar: spectre_calendar::CalendarApp::default(),
            files: spectre_files::Browser::default(),
            fdroid_installed: [false; 5],
            rec_active: false,
            rec_elapsed: 0.0,
            rec_last: 0.0,
            rec_saved: Vec::new(),
        }
    }
}

impl BuiltInApps {
    pub(super) fn set_camera_mode(&mut self, video: bool) {
        self.camera_mode = if video { 0 } else { 1 };
    }

    pub fn alarm_summary(&self) -> Option<String> {
        self.alarms.iter().find(|a| a.enabled).map(|a| {
            format!(
                "{}:{:02} {}",
                a.hour,
                a.minute,
                if a.am { "AM" } else { "PM" }
            )
        })
    }

    pub fn add_quick_alarm(&mut self) {
        let next_h = ((self.alarms.len() as u8 + 7) % 12).max(1);
        self.alarms.push(AlarmItem {
            hour: next_h,
            minute: 15,
            am: true,
            days: "Tomorrow".into(),
            enabled: true,
        });
        self.clock_tab = 0;
    }

    pub fn edit_text_for(&self, id: u64, calendar_active: bool) -> &str {
        if spectre_calendar::field_is_calendar(id) || (calendar_active && self.calendar.choosing_file()
            && matches!(id, super::ID_FILES_SEARCH | super::ID_FILES_EDIT))
        {
            return self.calendar.edit_text(id).unwrap_or_default();
        }
        match id {
            ID_BROWSER_URL => &self.browser_url,
            ID_CHAT_COMPOSER => &self.chat_input,
            super::ID_FILES_SEARCH | super::ID_FILES_EDIT => self.files.edit_text(id).unwrap_or_default(),
            _ => "",
        }
    }

    pub fn set_edit_text(&mut self, id: u64, value: &str, calendar_active: bool) {
        if spectre_calendar::field_is_calendar(id) || (calendar_active && self.calendar.choosing_file()
            && matches!(id, super::ID_FILES_SEARCH | super::ID_FILES_EDIT))
        {
            self.calendar.set_edit_text(id, value);
            return;
        }
        match id {
            ID_BROWSER_URL => self.browser_url = value.to_owned(),
            ID_CHAT_COMPOSER => self.chat_input = value.to_owned(),
            super::ID_FILES_SEARCH | super::ID_FILES_EDIT => {
                self.files.set_edit_text(id, value);
            }
            _ => {}
        }
    }

    pub fn handle_back(&mut self, app: AppId) -> bool {
        match app {
            AppId::ArcaneChat | AppId::Messaging | AppId::DeltaChat | AppId::Telegram => {
                if self.chat_thread.is_some() {
                    self.chat_thread = None;
                    return true;
                }
            }
            AppId::Gallery => {
                if self.gallery_open.is_some() {
                    self.gallery_open = None;
                    return true;
                }
            }
            AppId::Phone => {
                if self.in_call.is_some() {
                    self.in_call = None;
                    return true;
                }
                if self.phone_dialpad_open {
                    self.phone_dialpad_open = false;
                    return true;
                }
            }
            AppId::Files => return self.files.back(),
            AppId::Calendar => return self.calendar.back(),
            _ => {}
        }
        false
    }

    pub fn show(
        &mut self,
        f: &mut Frame<'_>,
        app: AppId,
        bounds: Rect,
        id_base: u64,
        sys: &mut SystemState,
        _pointer: Option<[f32; 2]>,
        clock: super::ClockReading,
    ) -> ShellAction {
        let p = sys.palette();
        let mut action = ShellAction::None;
        f.within(bounds, |f| {
            action = match app {
                AppId::Clock => self.render_clock(f, bounds, id_base, clock),
                AppId::Calculator => self.render_calculator(f, bounds, id_base, &p),
                AppId::Phone => self.render_phone(f, bounds, id_base, &p),
                AppId::ArcaneChat | AppId::Messaging | AppId::DeltaChat | AppId::Telegram => {
                    self.render_chat_app(f, app, bounds, id_base, &p)
                }
                AppId::Browser => self.render_browser(f, bounds, id_base, &p),
                AppId::Gallery => self.render_gallery(f, bounds, id_base, &p),
                AppId::Music => self.render_music(f, bounds, id_base, &p),
                AppId::Camera => self.render_camera(f, bounds, id_base),
                AppId::Calendar => {
                    let appearance = spectre_calendar::Appearance {
                        background: p.settings_bg, surface: p.card_elevated, text: p.on_surface,
                        muted: p.on_surface_variant, primary: p.primary, on_primary: p.on_primary,
                        accent_surface: p.primary_container, on_accent_surface: p.on_primary_container, divider: p.divider,
                    };
                    match self.calendar.show(f, bounds, id_base, appearance) {
                        spectre_calendar::Action::None => ShellAction::None,
                        spectre_calendar::Action::Focus(id) => ShellAction::FocusTextField(id),
                        spectre_calendar::Action::Blur => ShellAction::BlurTextField,
                    }
                }
                AppId::Files => match self.files.show(f, bounds, files_appearance(&p)) {
                    spectre_files::Action::Focus(id) => ShellAction::FocusTextField(id),
                    spectre_files::Action::Blur => ShellAction::BlurTextField,
                    spectre_files::Action::Open(path) => ShellAction::OpenFile(path),
                    spectre_files::Action::NewWindow(folder) => ShellAction::LaunchFilesWindow(folder),
                    spectre_files::Action::Share(paths) => ShellAction::ShareFiles(paths),
                    _ => ShellAction::None,
                },
                AppId::FDroid => self.render_fdroid(f, bounds, id_base, &p),
                AppId::Recorder => self.render_recorder(f, bounds, id_base),
                AppId::Contacts | AppId::Gmail | AppId::Settings => ShellAction::None,
            };
        });
        action
    }

    // =========================================================================
    // CLOCK (Exact match to 06-split-screen-50-50.png dark #19191D UI!)
    // =========================================================================

    fn render_clock(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, clock: super::ClockReading) -> ShellAction {
        let now = f.time();
        if self.timer_running {
            let dt = (now - self.timer_last).clamp(0.0, 0.25);
            self.timer_last = now;
            self.timer_secs = (self.timer_secs - dt).max(0.0);
            if self.timer_secs <= 0.0 {
                self.timer_running = false;
            } else {
                f.continuous(b);
            }
        }
        if self.sw_running {
            let dt = (now - self.sw_last).clamp(0.0, 0.25);
            self.sw_last = now;
            self.sw_elapsed += dt;
            f.continuous(b);
        }

        let bg = Color(0x19, 0x19, 0x1D, 255);
        let card = Color(0x28, 0x29, 0x30, 255);
        let nav_bg = Color(0x23, 0x24, 0x2A, 255);
        let pink = Color(0xDE, 0xC0, 0xE5, 255);
        let fg = Color(0xEC, 0xEA, 0xF0, 255);
        let sub = Color(0xA4, 0xA6, 0xB2, 255);
        let dark_p = Palette {
            primary: pink,
            primary_dark: pink,
            on_primary: Color(0x22, 0x16, 0x28, 255),
            primary_container: Color(0xF6, 0xE5, 0xFA, 255),
            on_primary_container: Color(0x22, 0x16, 0x28, 255),
            qs_backdrop: bg,
            settings_bg: bg,
            card_elevated: card,
            card_pure: card,
            unavailable: card,
            surface_container: card,
            on_surface: fg,
            on_surface_variant: sub,
            outline: sub,
            recents_bg: bg,
            divider: card,
            dark: true,
        };

        f.paint_rect(b, 0.0, bg);
        let titles = ["Alarm", "Clock", "Timer", "Stopwatch"];
        f.styled_text(
            Rect { x: b.x + 20.0, y: b.y + 12.0, width: b.width - 70.0, height: 32.0 },
            titles[self.clock_tab],
            fg,
            text_style(21.0, 500, 0),
        );
        f.paint_material_icon(
            Rect { x: b.x + b.width - 40.0, y: b.y + 16.0, width: 22.0, height: 22.0 },
            MaterialIcon::MoreVert,
            fg,
        );

        let nav_h = 60.0;
        let body = Rect {
            x: b.x,
            y: b.y + 52.0,
            width: b.width,
            height: (b.height - 52.0 - nav_h).max(100.0),
        };
        let mut action = ShellAction::None;

        match self.clock_tab {
            0 => {
                // Alarm cards matching 06-split-screen-50-50.png
                let mut y = body.y + 6.0;
                for (i, alarm) in self.alarms.iter_mut().enumerate().take(4) {
                    if y + 92.0 > body.y + body.height + 10.0 {
                        break;
                    }
                    let cr = Rect { x: b.x + 14.0, y, width: b.width - 28.0, height: 102.0 };
                    f.paint_rect(cr, 22.0, card);
                    let time_txt = format!("{}:{:02}", alarm.hour, alarm.minute);
                    f.styled_text(
                        Rect { x: cr.x + 18.0, y: cr.y + 14.0, width: 132.0, height: 46.0 },
                        &time_txt,
                        if alarm.enabled { fg } else { sub },
                        text_style(38.0, 400, 0),
                    );
                    f.styled_text(
                        Rect { x: cr.x + 126.0, y: cr.y + 30.0, width: 42.0, height: 24.0 },
                        if alarm.am { "AM" } else { "PM" },
                        if alarm.enabled { fg } else { sub },
                        text_style(14.0, 500, 0),
                    );
                    f.styled_text(
                        Rect { x: cr.x + 18.0, y: cr.y + 68.0, width: cr.width - 104.0, height: 20.0 },
                        &alarm.days,
                        sub,
                        text_style(12.5, 400, 0),
                    );
                    paint_switch(
                        f,
                        id + 10 + i as u64,
                        Rect { x: cr.x + cr.width - 72.0, y: cr.y + 54.0, width: 58.0, height: 34.0 },
                        &mut alarm.enabled,
                        true,
                        &dark_p,
                    );
                    y += 110.0;
                }
                // Centered pink `#DEC0E5` `+` FAB matching 06-split-screen-50-50.png!
                let fab_sz = 64.0;
                let fab = Rect {
                    x: b.x + (b.width - fab_sz) * 0.5,
                    y: body.y + body.height - fab_sz - 12.0,
                    width: fab_sz,
                    height: fab_sz,
                };
                let rf = f.target(id + 20, fab, "Add alarm", true, Role::Button);
                f.elevation(fab, 32.0, 3);
                f.paint_rect(fab, 32.0, pink);
                f.paint_material_icon(fab.inset(20.0), MaterialIcon::Add, Color(0x1B, 0x16, 0x20, 255));
                if rf.clicked {
                    self.add_quick_alarm();
                    action = ShellAction::ShowToast("Added alarm".into());
                }
            }
            1 => {
                // World Clock
                f.styled_text(
                    Rect { x: b.x + 20.0, y: body.y + 18.0, width: b.width - 40.0, height: 56.0 },
                    &clock.hm(),
                    fg,
                    text_style(48.0, 400, 0),
                );
                f.styled_text(
                    Rect { x: b.x + 22.0, y: body.y + 74.0, width: b.width - 44.0, height: 22.0 },
                    &format!("{} · Local time", clock.date()),
                    sub,
                    text_style(14.0, 400, 0),
                );
                for (i, (city, zone)) in [
                    ("London", chrono_tz::Europe::London),
                    ("New York", chrono_tz::America::New_York),
                    ("Tokyo", chrono_tz::Asia::Tokyo),
                ].into_iter().enumerate() {
                    let local = clock.0.with_timezone(&zone);
                    let tm = local.format("%H:%M").to_string();
                    let delta = local.format("UTC%:z").to_string();
                    let cr = Rect { x: b.x + 14.0, y: body.y + 114.0 + i as f32 * 70.0, width: b.width - 28.0, height: 62.0 };
                    f.paint_rect(cr, 18.0, card);
                    f.styled_text(Rect { x: cr.x + 16.0, y: cr.y + 10.0, width: 140.0, height: 22.0 }, city, fg, text_style(16.0, 500, 0));
                    f.styled_text(Rect { x: cr.x + 16.0, y: cr.y + 34.0, width: 120.0, height: 18.0 }, &delta, sub, text_style(12.5, 400, 0));
                    f.styled_text(Rect { x: cr.x + cr.width - 110.0, y: cr.y, width: 94.0, height: cr.height }, &tm, fg, text_style(20.0, 400, 2));
                }
            }
            2 => {
                // Countdown Timer
                let mins = (self.timer_secs as u64) / 60;
                let secs = (self.timer_secs as u64) % 60;
                let ring = Rect {
                    x: b.x + b.width * 0.5 - 82.0,
                    y: body.y + 24.0,
                    width: 164.0,
                    height: 164.0,
                };
                f.paint_arc(ring, 0.0, std::f32::consts::TAU, 6.0, card);
                let frac = (self.timer_secs / 300.0).clamp(0.0, 1.0) as f32;
                f.paint_arc(ring, -std::f32::consts::FRAC_PI_2, std::f32::consts::TAU * frac, 6.0, pink);
                f.styled_text(ring, &format!("{mins:02}:{secs:02}"), fg, text_style(36.0, 400, 1));

                let by = ring.y + ring.height + 24.0;
                let b_play = Rect { x: b.x + b.width * 0.5 - 48.0, y: by, width: 96.0, height: 42.0 };
                let rp = f.target(id + 30, b_play, "Start timer", true, Role::Button);
                f.paint_rect(b_play, 21.0, pink);
                f.styled_text(b_play, if self.timer_running { "Pause" } else { "Start" }, Color(0x1B, 0x16, 0x20, 255), text_style(14.0, 600, 1));
                if rp.clicked {
                    self.timer_running = !self.timer_running;
                    self.timer_last = now;
                }
                let b_add = Rect { x: b.x + 20.0, y: by, width: 72.0, height: 42.0 };
                let ra = f.target(id + 31, b_add, "+1 min", true, Role::Button);
                f.paint_rect(b_add, 21.0, card);
                f.styled_text(b_add, "+1:00", fg, text_style(13.5, 500, 1));
                if ra.clicked {
                    self.timer_secs += 60.0;
                }
                let b_rst = Rect { x: b.x + b.width - 92.0, y: by, width: 72.0, height: 42.0 };
                let rr = f.target(id + 32, b_rst, "Reset timer", true, Role::Button);
                f.paint_rect(b_rst, 21.0, card);
                f.styled_text(b_rst, "Reset", fg, text_style(13.5, 500, 1));
                if rr.clicked {
                    self.timer_running = false;
                    self.timer_secs = 300.0;
                }
            }
            _ => {
                // Stopwatch
                let mins = (self.sw_elapsed as u64) / 60;
                let secs = (self.sw_elapsed as u64) % 60;
                let cs = ((self.sw_elapsed * 100.0) as u64) % 100;
                f.styled_text(
                    Rect { x: b.x, y: body.y + 38.0, width: b.width, height: 56.0 },
                    &format!("{mins:02}:{secs:02}.{cs:02}"),
                    fg,
                    text_style(42.0, 400, 1),
                );
                let by = body.y + 114.0;
                let b_start = Rect { x: b.x + b.width * 0.5 - 48.0, y: by, width: 96.0, height: 42.0 };
                if f.target(id + 40, b_start, "Stopwatch toggle", true, Role::Button).clicked {
                    self.sw_running = !self.sw_running;
                    self.sw_last = now;
                }
                f.paint_rect(b_start, 21.0, pink);
                f.styled_text(b_start, if self.sw_running { "Pause" } else { "Start" }, Color(0x1B, 0x16, 0x20, 255), text_style(14.0, 600, 1));

                let b_lap = Rect { x: b.x + 20.0, y: by, width: 72.0, height: 42.0 };
                if f.target(id + 41, b_lap, "Lap", true, Role::Button).clicked && self.sw_elapsed > 0.0 {
                    self.sw_laps.insert(0, self.sw_elapsed);
                    self.sw_laps.truncate(4);
                }
                f.paint_rect(b_lap, 21.0, card);
                f.styled_text(b_lap, "Lap", fg, text_style(13.5, 500, 1));

                let b_rst = Rect { x: b.x + b.width - 92.0, y: by, width: 72.0, height: 42.0 };
                if f.target(id + 42, b_rst, "Reset stopwatch", true, Role::Button).clicked {
                    self.sw_running = false;
                    self.sw_elapsed = 0.0;
                    self.sw_laps.clear();
                }
                f.paint_rect(b_rst, 21.0, card);
                f.styled_text(b_rst, "Reset", fg, text_style(13.5, 500, 1));

                for (i, &lap) in self.sw_laps.iter().enumerate() {
                    let ly = by + 56.0 + i as f32 * 28.0;
                    f.styled_text(
                        Rect { x: b.x + 32.0, y: ly, width: b.width - 64.0, height: 24.0 },
                        &format!("#{}   {:02}:{:05.2}", self.sw_laps.len() - i, (lap as u64) / 60, lap % 60.0),
                        sub,
                        text_style(14.0, 400, 1),
                    );
                }
            }
        }

        // Bottom 4-Tab Bar (Alarm, Clock, Timer, Stopwatch)
        let bar = Rect { x: b.x, y: b.y + b.height - nav_h, width: b.width, height: nav_h };
        f.paint_rect(bar, 0.0, nav_bg);
        let tab_icons = [
            NavigationIcon::Alarm,
            NavigationIcon::Clock,
            NavigationIcon::Timer,
            NavigationIcon::Stopwatch,
        ];
        let tw = bar.width / 4.0;
        for i in 0..4 {
            let tr = Rect { x: bar.x + i as f32 * tw, y: bar.y, width: tw, height: nav_h };
            let r = f.target(id + 50 + i as u64, tr, titles[i], true, Role::Tab);
            let active = self.clock_tab == i;
            let col = if active { pink } else { sub };
            f.paint_navigation_icon(
                Rect { x: tr.x + (tw - 22.0) * 0.5, y: tr.y + 10.0, width: 22.0, height: 22.0 },
                tab_icons[i],
                active,
                col,
            );
            f.styled_text(
                Rect { x: tr.x, y: tr.y + 34.0, width: tw, height: 18.0 },
                titles[i],
                col,
                text_style(11.5, 500, 1),
            );
            if r.clicked {
                self.clock_tab = i;
            }
        }
        action
    }

    // =========================================================================
    // WORKING MATERIAL YOU CALCULATOR
    // =========================================================================

    fn render_calculator(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: &Palette) -> ShellAction {
        f.paint_rect(b, 0.0, p.settings_bg);
        let disp_h = (b.height * 0.30).clamp(96.0, 210.0);
        let disp = Rect { x: b.x, y: b.y, width: b.width, height: disp_h };
        f.paint_rect(disp, 24.0, p.primary_container.opacity(0.45));

        if let (Some(acc), Some(op)) = (self.calc_acc, self.calc_op) {
            f.styled_text(
                Rect { x: disp.x + 20.0, y: disp.y + 16.0, width: disp.width - 40.0, height: 24.0 },
                &format!("{} {}", trim_float(acc), op),
                p.on_surface_variant,
                text_style(18.0, 400, 2),
            );
        }
        let font_sz = if self.calc_display.len() > 10 { 30.0 } else { 44.0 };
        f.styled_text(
            Rect { x: disp.x + 20.0, y: disp.y + disp_h - 68.0, width: disp.width - 40.0, height: 56.0 },
            &self.calc_display,
            p.on_surface,
            text_style(font_sz, 400, 2),
        );

        let keys: &[&[&str]] = &[
            &["AC", "±", "%", "÷"],
            &["7", "8", "9", "×"],
            &["4", "5", "6", "−"],
            &["1", "2", "3", "+"],
            &["0", ".", "⌫", "="],
        ];
        let grid_y = disp.y + disp.height + 12.0;
        let grid_h = (b.y + b.height - grid_y - 10.0).max(160.0);
        let gap = 8.0;
        let cw = (b.width - 24.0 - gap * 3.0) / 4.0;
        let ch = (grid_h - gap * 4.0) / 5.0;

        for (r_i, row) in keys.iter().enumerate() {
            for (c_i, &k) in row.iter().enumerate() {
                let kr = Rect {
                    x: b.x + 12.0 + c_i as f32 * (cw + gap),
                    y: grid_y + r_i as f32 * (ch + gap),
                    width: cw,
                    height: ch,
                };
                let resp = f.target(id + (r_i * 4 + c_i) as u64, kr, k, true, Role::Button);
                let (bg, fg) = match k {
                    "AC" => (Color(0xD8, 0xE8, 0xCE, 255), Color(0x1B, 0x35, 0x16, 255)),
                    "=" => (p.primary, p.on_primary),
                    "±" | "%" | "÷" | "×" | "−" | "+" => (p.primary_container, p.on_primary_container),
                    _ => (p.card_elevated, p.on_surface),
                };
                let rad = ch.min(cw) * 0.45;
                f.paint_rect(kr, rad, bg);
                f.state_layer(resp, rad, fg);
                f.styled_text(kr, k, fg, text_style((ch * 0.38).clamp(16.0, 24.0), 500, 1));
                if resp.clicked {
                    self.calc_press(k);
                }
            }
        }
        ShellAction::None
    }

    fn calc_press(&mut self, key: &str) {
        match key {
            "AC" => {
                self.calc_display = "0".into();
                self.calc_acc = None;
                self.calc_op = None;
                self.calc_fresh = true;
            }
            "⌫" => {
                if !self.calc_fresh {
                    self.calc_display.pop();
                    if self.calc_display.is_empty() || self.calc_display == "-" {
                        self.calc_display = "0".into();
                        self.calc_fresh = true;
                    }
                }
            }
            "±" => {
                if let Ok(v) = self.calc_display.parse::<f64>() {
                    self.calc_display = trim_float(-v);
                }
            }
            "%" => {
                if let Ok(v) = self.calc_display.parse::<f64>() {
                    self.calc_display = trim_float(v / 100.0);
                    self.calc_fresh = true;
                }
            }
            "÷" | "×" | "−" | "+" => {
                let op = key.chars().next().unwrap_or('+');
                if !self.calc_fresh {
                    self.calc_eval();
                }
                self.calc_acc = self.calc_display.parse::<f64>().ok();
                self.calc_op = Some(op);
                self.calc_fresh = true;
            }
            "=" => {
                self.calc_eval();
                self.calc_acc = None;
                self.calc_op = None;
                self.calc_fresh = true;
            }
            "." => {
                if self.calc_fresh {
                    self.calc_display = "0.".into();
                    self.calc_fresh = false;
                } else if !self.calc_display.contains('.') {
                    self.calc_display.push('.');
                }
            }
            d => {
                if self.calc_fresh || self.calc_display == "0" {
                    self.calc_display = d.into();
                    self.calc_fresh = false;
                } else if self.calc_display.len() < 14 {
                    self.calc_display.push_str(d);
                }
            }
        }
    }

    fn calc_eval(&mut self) {
        if let (Some(a), Some(op), Ok(b)) = (self.calc_acc, self.calc_op, self.calc_display.parse::<f64>()) {
            let res = match op {
                '+' => a + b,
                '−' => a - b,
                '×' => a * b,
                '÷' => if b.abs() < 1e-12 { f64::NAN } else { a / b },
                _ => b,
            };
            self.calc_display = if res.is_finite() {
                trim_float(res)
            } else {
                "Error".into()
            };
        }
    }

    // =========================================================================
    // PHONE DIALER & ACTIVE CALL
    // =========================================================================

    fn render_phone(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: &Palette) -> ShellAction {
        let now = f.time();
        // 1. Active In-Call Screen
        if let Some((who, start)) = self.in_call.clone() {
            f.paint_rect(b, 0.0, Color(0x15, 0x22, 0x38, 255));
            let secs = (now - start).max(0.0) as u64;
            f.continuous(b);
            let av = Rect { x: b.x + b.width * 0.5 - 42.0, y: b.y + 76.0, width: 84.0, height: 84.0 };
            f.paint_rect(av, 42.0, Color(0x43, 0x5E, 0x91, 255));
            let init = who.chars().next().unwrap_or('?').to_ascii_uppercase().to_string();
            f.styled_text(av, &init, Color(255, 255, 255, 255), text_style(34.0, 500, 1));
            f.styled_text(
                Rect { x: b.x, y: b.y + 178.0, width: b.width, height: 34.0 },
                &who,
                Color(255, 255, 255, 255),
                text_style(26.0, 500, 1),
            );
            f.styled_text(
                Rect { x: b.x, y: b.y + 216.0, width: b.width, height: 24.0 },
                &format!("Calling · {:02}:{:02}", secs / 60, secs % 60),
                Color(0xA8, 0xC7, 0xFA, 255),
                text_style(15.0, 400, 1),
            );
            // In-call action controls: Mute | Keypad | Speaker
            let ctrl_y = b.y + b.height - 210.0;
            let btns = [
                ("Mute", if self.phone_muted { "🔇" } else { "🎙" }, self.phone_muted),
                ("Keypad", "⋮⋮⋮", false),
                ("Speaker", "🔊", self.phone_speaker),
            ];
            let cw = b.width / 3.0;
            for (i, (lbl, glyph, on)) in btns.iter().enumerate() {
                let cr = Rect { x: b.x + i as f32 * cw + (cw - 58.0) * 0.5, y: ctrl_y, width: 58.0, height: 58.0 };
                let r = f.target(id + 80 + i as u64, cr, lbl, true, Role::Button);
                f.paint_rect(cr, 29.0, if *on { Color(255, 255, 255, 235) } else { Color(255, 255, 255, 34) });
                f.styled_text(cr, glyph, if *on { Color(0x15, 0x22, 0x38, 255) } else { Color(255, 255, 255, 255) }, text_style(20.0, 500, 1));
                f.styled_text(Rect { x: b.x + i as f32 * cw, y: ctrl_y + 64.0, width: cw, height: 18.0 }, lbl, Color(255, 255, 255, 210), text_style(12.0, 400, 1));
                if r.clicked {
                    match i {
                        0 => self.phone_muted = !self.phone_muted,
                        2 => self.phone_speaker = !self.phone_speaker,
                        _ => {}
                    }
                }
            }
            let end_btn = Rect {
                x: b.x + b.width * 0.5 - 34.0,
                y: b.y + b.height - 104.0,
                width: 68.0,
                height: 68.0,
            };
            let re = f.target(id + 90, end_btn, "End call", true, Role::Button);
            f.paint_rect(end_btn, 34.0, Color(0xE0, 0x3E, 0x36, 255));
            paint_sys_icon(f, end_btn.inset(22.0), SysIcon::Close, Color(255, 255, 255, 255));
            if re.clicked {
                self.in_call = None;
                self.phone_muted = false;
                self.phone_speaker = false;
            }
            return ShellAction::None;
        }

        let bg = if p.dark { p.settings_bg } else { Color(0xFF, 0xFF, 0xFF, 255) };
        f.paint_rect(b, 0.0, bg);

        // 2. Top Search Pill (`Search contacts`) matching Android 13 Dialer
        let search_r = Rect { x: b.x + 14.0, y: b.y + 10.0, width: b.width - 28.0, height: 48.0 };
        f.paint_rect(search_r, 24.0, p.surface_container);
        f.styled_text(Rect { x: search_r.x + 14.0, y: search_r.y, width: 28.0, height: 48.0 }, "🔍", p.on_surface_variant, text_style(16.0, 400, 1));
        f.styled_text(Rect { x: search_r.x + 48.0, y: search_r.y, width: search_r.width - 110.0, height: 48.0 }, "Search contacts", p.on_surface_variant, text_style(15.5, 400, 0));
        f.styled_text(Rect { x: search_r.x + search_r.width - 42.0, y: search_r.y, width: 32.0, height: 48.0 }, "⋮", p.on_surface_variant, text_style(18.0, 600, 1));

        let nav_bar_h = 68.0;
        let body = Rect { x: b.x, y: b.y + 66.0, width: b.width, height: (b.height - 66.0 - nav_bar_h).max(120.0) };

        // 3. Tab Content (100% fake data!)
        match self.phone_tab {
            0 => {
                // Favorites tab
                let favs = [
                    ("Alice Johnson", "+1 (415) 555-0142", Color(0x43, 0x5E, 0x91, 255)),
                    ("Bob Miller", "+1 (650) 555-0199", Color(0x2E, 0x7D, 0x5B, 255)),
                ];
                let fw = (body.width - 40.0) * 0.5;
                for (i, (name, num, col)) in favs.iter().enumerate() {
                    let fr = Rect { x: body.x + 14.0 + i as f32 * (fw + 12.0), y: body.y + 12.0, width: fw, height: 112.0 };
                    let r = f.target(id + 10 + i as u64, fr, name, !self.phone_dialpad_open, Role::Button);
                    f.paint_rect(fr, 20.0, p.surface_container);
                    f.state_layer(r, 20.0, p.on_surface);
                    let av = Rect { x: fr.x + fr.width * 0.5 - 24.0, y: fr.y + 14.0, width: 48.0, height: 48.0 };
                    f.paint_rect(av, 24.0, *col);
                    f.styled_text(av, &name[..1], Color(255, 255, 255, 255), text_style(20.0, 500, 1));
                    f.styled_text(Rect { x: fr.x + 8.0, y: fr.y + 68.0, width: fr.width - 16.0, height: 18.0 }, name, p.on_surface, text_style(13.0, 500, 1));
                    f.styled_text(Rect { x: fr.x + 8.0, y: fr.y + 86.0, width: fr.width - 16.0, height: 16.0 }, num, p.on_surface_variant, text_style(10.5, 400, 1));
                    if r.clicked {
                        self.in_call = Some(((*name).into(), now));
                    }
                }
                let add_btn = Rect { x: body.x + body.width * 0.5 - 76.0, y: body.y + 150.0, width: 152.0, height: 42.0 };
                let r_add = f.target(id + 15, add_btn, "ADD FAVORITE", !self.phone_dialpad_open, Role::Button);
                f.paint_rect(add_btn, 21.0, p.primary);
                f.state_layer(r_add, 21.0, p.on_primary);
                f.styled_text(add_btn, "ADD FAVORITE", p.on_primary, text_style(12.5, 600, 1));
                if r_add.clicked {
                    self.phone_tab = 2;
                }
            }
            1 => {
                // Recents call log tab
                let recents = [
                    ("Alice Johnson", "↙ Incoming · Today, 1:05 PM", Color(0x43, 0x5E, 0x91, 255)),
                    ("Bob Miller", "↗ Outgoing · Yesterday", Color(0x2E, 0x7D, 0x5B, 255)),
                    ("Charlie Lee", "↙ Missed · Nov 19", Color(0xC6, 0x3A, 0x32, 255)),
                    ("SpectreOS Lab", "↗ Outgoing · Nov 18", Color(0x6D, 0x58, 0x99, 255)),
                ];
                for (i, (name, meta, col)) in recents.iter().enumerate() {
                    let rr = Rect { x: body.x, y: body.y + 8.0 + i as f32 * 66.0, width: body.width, height: 62.0 };
                    let r = f.target(id + 20 + i as u64, rr, name, !self.phone_dialpad_open, Role::ListItem);
                    f.state_layer(r, 0.0, p.on_surface);
                    let av = Rect { x: rr.x + 16.0, y: rr.y + 11.0, width: 40.0, height: 40.0 };
                    f.paint_rect(av, 20.0, *col);
                    f.styled_text(av, &name[..1], Color(255, 255, 255, 255), text_style(17.0, 500, 1));
                    f.styled_text(Rect { x: rr.x + 68.0, y: rr.y + 11.0, width: rr.width - 124.0, height: 22.0 }, name, p.on_surface, text_style(15.0, 500, 0));
                    f.styled_text(Rect { x: rr.x + 68.0, y: rr.y + 34.0, width: rr.width - 124.0, height: 18.0 }, meta, p.on_surface_variant, text_style(12.5, 400, 0));
                    f.styled_text(Rect { x: rr.x + rr.width - 48.0, y: rr.y + 14.0, width: 34.0, height: 34.0 }, "📞", p.primary, text_style(17.0, 400, 1));
                    if r.clicked {
                        self.in_call = Some(((*name).into(), now));
                    }
                }
            }
            _ => {
                // Contacts tab
                let contacts = [
                    ("Alice Johnson", "+1 (415) 555-0142", Color(0x43, 0x5E, 0x91, 255)),
                    ("Bob Miller", "+1 (650) 555-0199", Color(0x2E, 0x7D, 0x5B, 255)),
                    ("Charlie Lee", "+1 (212) 555-0174", Color(0x9C, 0x62, 0x8F, 255)),
                    ("Diana Prince", "+1 (310) 555-0123", Color(0xB0, 0x5B, 0x3B, 255)),
                    ("SpectreOS Lab", "+1 (800) 555-0100", Color(0x6D, 0x58, 0x99, 255)),
                ];
                for (i, (name, num, col)) in contacts.iter().enumerate() {
                    let rr = Rect { x: body.x, y: body.y + 8.0 + i as f32 * 64.0, width: body.width, height: 60.0 };
                    let r = f.target(id + 30 + i as u64, rr, name, !self.phone_dialpad_open, Role::ListItem);
                    f.state_layer(r, 0.0, p.on_surface);
                    let av = Rect { x: rr.x + 16.0, y: rr.y + 10.0, width: 40.0, height: 40.0 };
                    f.paint_rect(av, 20.0, *col);
                    f.styled_text(av, &name[..1], Color(255, 255, 255, 255), text_style(17.0, 500, 1));
                    f.styled_text(Rect { x: rr.x + 68.0, y: rr.y + 10.0, width: rr.width - 84.0, height: 22.0 }, name, p.on_surface, text_style(15.0, 500, 0));
                    f.styled_text(Rect { x: rr.x + 68.0, y: rr.y + 33.0, width: rr.width - 84.0, height: 18.0 }, num, p.on_surface_variant, text_style(12.5, 400, 0));
                    if r.clicked {
                        self.in_call = Some(((*name).into(), now));
                    }
                }
            }
        }

        // 4. Floating Dialpad FAB (bottom-right above nav bar)
        if !self.phone_dialpad_open {
            let fab = Rect { x: b.x + b.width - 74.0, y: b.y + b.height - nav_bar_h - 72.0, width: 56.0, height: 56.0 };
            let r_fab = f.target(id + 40, fab, "Open dialpad", true, Role::Button);
            f.elevation(fab, 28.0, 3);
            f.paint_rect(fab, 28.0, p.primary);
            f.state_layer(r_fab, 28.0, p.on_primary);
            f.styled_text(fab, "⋮⋮⋮", p.on_primary, text_style(18.0, 700, 1));
            if r_fab.clicked {
                self.phone_dialpad_open = true;
            }
        }

        // 5. Material 3 Bottom Navigation Bar (`Favorites`, `Recents`, `Contacts`)
        let nav_r = Rect { x: b.x, y: b.y + b.height - nav_bar_h, width: b.width, height: nav_bar_h };
        f.paint_rect(nav_r, 0.0, p.surface_container);
        let tabs = [("★", "Favorites"), ("🕒", "Recents"), ("👥", "Contacts")];
        let tw = nav_r.width / 3.0;
        for (i, (glyph, lbl)) in tabs.iter().enumerate() {
            let tr = Rect { x: nav_r.x + i as f32 * tw, y: nav_r.y, width: tw, height: nav_bar_h };
            let r = f.target(id + i as u64, tr, lbl, !self.phone_dialpad_open, Role::Tab);
            let active = self.phone_tab == i;
            let pill = Rect { x: tr.x + (tw - 60.0) * 0.5, y: tr.y + 10.0, width: 60.0, height: 30.0 };
            if active {
                f.paint_rect(pill, 15.0, p.primary_container);
            }
            f.styled_text(pill, glyph, if active { p.on_primary_container } else { p.on_surface_variant }, text_style(16.0, 500, 1));
            f.styled_text(
                Rect { x: tr.x, y: tr.y + 44.0, width: tw, height: 18.0 },
                lbl,
                if active { p.on_surface } else { p.on_surface_variant },
                text_style(11.5, if active { 600 } else { 400 }, 1),
            );
            if r.clicked {
                self.phone_tab = i;
            }
        }

        // 6. Rounded-top Dialpad Bottom Sheet matching Android 13 Dialpad!
        if self.phone_dialpad_open {
            let sheet_h = (b.height * 0.56).clamp(360.0, 440.0);
            let dismiss_r = Rect { x: b.x, y: b.y, width: b.width, height: (b.height - sheet_h).max(40.0) };
            if f.target(id + 48, dismiss_r, "Close dialpad", true, Role::Button).clicked {
                self.phone_dialpad_open = false;
            }
            let sheet = Rect { x: b.x, y: b.y + b.height - sheet_h, width: b.width, height: sheet_h };
            f.paint_rect(sheet, 26.0, p.surface_container);
            f.paint_rect(Rect { x: sheet.x, y: sheet.y + 26.0, width: sheet.width, height: sheet.height - 26.0 }, 0.0, p.surface_container);

            // Number readout + backspace
            f.styled_text(
                Rect { x: sheet.x + 24.0, y: sheet.y + 14.0, width: sheet.width - 78.0, height: 44.0 },
                &self.dial_num,
                p.on_surface,
                text_style(28.0, 400, 1),
            );
            let bs = Rect { x: sheet.x + sheet.width - 54.0, y: sheet.y + 18.0, width: 40.0, height: 36.0 };
            let r_bs = f.target(id + 49, bs, "Backspace", true, Role::Button);
            f.state_layer(r_bs, 18.0, p.primary);
            f.styled_text(bs, "⌫", p.primary, text_style(20.0, 500, 1));
            if r_bs.clicked {
                self.dial_num.pop();
            }

            // T9 Keypad with sub-labels
            let keys = [
                ("1", "∞"), ("2", "ABC"), ("3", "DEF"),
                ("4", "GHI"), ("5", "JKL"), ("6", "MNO"),
                ("7", "PQRS"), ("8", "TUV"), ("9", "WXYZ"),
                ("*", ""), ("0", "+"), ("#", ""),
            ];
            let cw = (sheet.width - 48.0) / 3.0;
            let ch = ((sheet.height - 154.0) / 4.0).clamp(48.0, 62.0);
            for (i, &(digit, sub)) in keys.iter().enumerate() {
                let kr = Rect {
                    x: sheet.x + 24.0 + (i % 3) as f32 * cw,
                    y: sheet.y + 64.0 + (i / 3) as f32 * (ch + 4.0),
                    width: cw,
                    height: ch,
                };
                let r = f.target(id + 50 + i as u64, kr, digit, true, Role::Button);
                f.state_layer(r, ch * 0.5, p.on_surface);
                f.styled_text(Rect { x: kr.x, y: kr.y + 2.0, width: kr.width, height: ch * 0.62 }, digit, p.primary, text_style(28.0, 400, 1));
                if !sub.is_empty() {
                    f.styled_text(Rect { x: kr.x, y: kr.y + ch * 0.60, width: kr.width, height: 16.0 }, sub, p.on_surface_variant, text_style(10.0, 500, 1));
                }
                if r.clicked && self.dial_num.len() < 16 {
                    self.dial_num.push_str(digit);
                }
            }

            // Green circular Call button
            let call_btn = Rect { x: sheet.x + sheet.width * 0.5 - 28.0, y: sheet.y + sheet.height - 72.0, width: 56.0, height: 56.0 };
            let rc = f.target(id + 70, call_btn, "Call", true, Role::Button);
            f.elevation(call_btn, 28.0, 3);
            f.paint_rect(call_btn, 28.0, Color(0x1E, 0x7E, 0x4E, 255));
            f.state_layer(rc, 28.0, Color(255, 255, 255, 255));
            f.styled_text(call_btn, "📞", Color(255, 255, 255, 255), text_style(22.0, 400, 1));
            if rc.clicked {
                let num = if self.dial_num.is_empty() {
                    "Alice Johnson".into()
                } else {
                    self.dial_num.clone()
                };
                self.in_call = Some((num, now));
                self.phone_dialpad_open = false;
            }
        }
        ShellAction::None
    }

    // =========================================================================
    // MESSAGING / ARCANECHAT / DELTA CHAT / TELEGRAM
    // =========================================================================

    fn render_chat_app(
        &mut self,
        f: &mut Frame<'_>,
        app: AppId,
        b: Rect,
        id: u64,
        p: &Palette,
    ) -> ShellAction {
        let accent = match app {
            AppId::ArcaneChat => Color(0x7B, 0x00, 0xC6, 255),
            AppId::Telegram => Color(0x2A, 0xA1, 0xE2, 255),
            AppId::DeltaChat => Color(0x3D, 0x56, 0x64, 255),
            AppId::Messaging => p.primary,
            _ => Color(0x2E, 0x8B, 0x47, 255),
        };
        f.paint_rect(b, 0.0, p.settings_bg);
        let hdr = Rect { height: 54.0, ..b };
        f.paint_rect(hdr, 0.0, accent);

        if let Some(ti) = self.chat_thread {
            let back = Rect { x: hdr.x + 8.0, y: hdr.y + 7.0, width: 40.0, height: 40.0 };
            if f.target(id + 1, back, "Back", true, Role::Button).clicked {
                self.chat_thread = None;
                return ShellAction::None;
            }
            paint_sys_icon(f, back.inset(10.0), SysIcon::Back, Color(255, 255, 255, 255));
            let name = self.threads[ti].0;
            f.styled_text(
                Rect { x: hdr.x + 56.0, y: hdr.y, width: hdr.width - 72.0, height: hdr.height },
                name,
                Color(255, 255, 255, 255),
                text_style(18.0, 500, 0),
            );

            let mut y = hdr.y + hdr.height + 14.0;
            for msg in &self.threads[ti].1 {
                let bw = (b.width * 0.76).min(290.0);
                let bx = if msg.from_me { b.x + b.width - bw - 14.0 } else { b.x + 14.0 };
                let lines = (msg.text.len() / 32).max(1) as f32;
                let bh = 38.0 + lines * 18.0;
                let br = Rect { x: bx, y, width: bw, height: bh };
                f.paint_rect(br, 18.0, if msg.from_me { accent } else { p.card_pure });
                f.paragraph(
                    Rect {
                        x: br.x + 12.0,
                        y: br.y + 8.0,
                        width: (br.width - 24.0).max(20.0),
                        height: (br.height - 24.0).max(16.0),
                    },
                    &msg.text,
                    if msg.from_me { Color(255, 255, 255, 255) } else { p.on_surface },
                    text_style(13.5, 400, 0),
                    17.0,
                );
                f.styled_text(
                    Rect {
                        x: br.x + 12.0,
                        y: br.y + br.height - 18.0,
                        width: (br.width - 24.0).max(20.0),
                        height: 14.0,
                    },
                    msg.time,
                    if msg.from_me {
                        Color(255, 255, 255, 185)
                    } else {
                        p.on_surface_variant
                    },
                    text_style(10.5, 400, 2),
                );
                y += bh + 10.0;
            }

            // Quick reply chips + live editable composer
            let chip_y = b.y + b.height - 96.0;
            for (ci, chip) in ["On my way!", "Sounds great!"].iter().enumerate() {
                let cr = Rect { x: b.x + 14.0 + ci as f32 * 116.0, y: chip_y, width: 108.0, height: 30.0 };
                if f.target(id + 20 + ci as u64, cr, chip, true, Role::Button).clicked {
                    self.threads[ti].1.push(ChatMessage {
                        from_me: true,
                        text: (*chip).into(),
                        time: "now",
                    });
                }
                f.paint_rect(cr, 15.0, p.primary_container);
                f.styled_text(cr, chip, p.on_primary_container, text_style(12.5, 500, 1));
            }

            let comp_y = b.y + b.height - 56.0;
            let inp = Rect { x: b.x + 12.0, y: comp_y, width: b.width - 78.0, height: 44.0 };
            f.paint_rect(inp, 22.0, p.card_pure);
            f.target(ID_CHAT_COMPOSER, inp, "Message", true, Role::TextField);
            f.styled_text(
                Rect { x: inp.x + 16.0, y: inp.y, width: inp.width - 28.0, height: inp.height },
                if self.chat_input.is_empty() { "Type a message..." } else { &self.chat_input },
                if self.chat_input.is_empty() { p.on_surface_variant } else { p.on_surface },
                text_style(14.0, 400, 0),
            );
            let send = Rect { x: b.x + b.width - 56.0, y: comp_y, width: 44.0, height: 44.0 };
            if f.target(id + 25, send, "Send", true, Role::Button).clicked && !self.chat_input.trim().is_empty() {
                let msg = std::mem::take(&mut self.chat_input);
                self.threads[ti].1.push(ChatMessage {
                    from_me: true,
                    text: msg,
                    time: "now",
                });
            }
            f.paint_rect(send, 22.0, accent);
            f.paint_material_icon(send.inset(11.0), MaterialIcon::Send, Color(255, 255, 255, 255));
        } else {
            f.styled_text(
                Rect { x: hdr.x + 20.0, y: hdr.y, width: hdr.width - 72.0, height: hdr.height },
                app.name(),
                Color(255, 255, 255, 255),
                text_style(20.0, 500, 0),
            );
            f.styled_text(
                Rect { x: hdr.x + hdr.width - 44.0, y: hdr.y, width: 36.0, height: hdr.height },
                "⋮",
                Color(255, 255, 255, 235),
                text_style(19.0, 600, 1),
            );
            let is_sms = app == AppId::Messaging;
            let row_h = if is_sms { 76.0 } else { 74.0 };
            for (i, (name, msgs)) in self.threads.iter().enumerate() {
                let rr = if is_sms {
                    Rect { x: b.x, y: hdr.y + hdr.height + 4.0 + i as f32 * row_h, width: b.width, height: row_h }
                } else {
                    Rect { x: b.x + 12.0, y: hdr.y + hdr.height + 12.0 + i as f32 * row_h, width: b.width - 24.0, height: 66.0 }
                };
                let r = f.target(id + 5 + i as u64, rr, name, true, Role::ListItem);
                if !is_sms {
                    f.paint_rect(rr, 16.0, p.card_elevated);
                }
                f.state_layer(r, if is_sms { 0.0 } else { 16.0 }, p.on_surface);
                let av = Rect { x: rr.x + 12.0, y: rr.y + 13.0, width: 40.0, height: 40.0 };
                f.paint_rect(av, 20.0, accent);
                f.styled_text(av, &name[..1], Color(255, 255, 255, 255), text_style(16.0, 600, 1));
                f.styled_text(Rect { x: rr.x + 64.0, y: rr.y + 12.0, width: rr.width - 80.0, height: 22.0 }, name, p.on_surface, text_style(15.0, 600, 0));
                let last = msgs.last().map_or("", |m| m.text.as_str());
                f.styled_text(Rect { x: rr.x + 64.0, y: rr.y + 34.0, width: rr.width - 80.0, height: 20.0 }, last, p.on_surface_variant, text_style(12.5, 400, 0));
                if r.clicked {
                    self.chat_thread = Some(i);
                }
            }
            if is_sms {
                let fab = Rect { x: b.x + b.width - 74.0, y: b.y + b.height - 74.0, width: 56.0, height: 56.0 };
                let r_fab = f.target(id + 40, fab, "Start new conversation", true, Role::Button);
                f.elevation(fab, 28.0, 3);
                f.paint_rect(fab, 28.0, accent);
                f.state_layer(r_fab, 28.0, Color(255, 255, 255, 255));
                f.styled_text(fab, "💬", Color(255, 255, 255, 255), text_style(20.0, 400, 1));
                if r_fab.clicked && !self.threads.is_empty() {
                    self.chat_thread = Some(0);
                }
            }
        }
        ShellAction::None
    }

    // =========================================================================
    // BROWSER, GALLERY, MUSIC, CAMERA, CALENDAR, FILES, F-DROID, RECORDER
    // =========================================================================

    fn render_browser(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: &Palette) -> ShellAction {
        f.paint_rect(b, 0.0, p.settings_bg);
        let bar = Rect { x: b.x + 12.0, y: b.y + 10.0, width: b.width - 24.0, height: 42.0 };
        f.paint_rect(bar, 21.0, p.card_pure);
        f.target(ID_BROWSER_URL, bar, "Address bar", true, Role::TextField);
        f.styled_text(
            Rect { x: bar.x + 16.0, y: bar.y, width: bar.width - 32.0, height: bar.height },
            &self.browser_url,
            p.on_surface,
            text_style(13.5, 400, 0),
        );
        let pages = [
            ("SpectreOS · libbhoo", "Offline Material 3 & Android 13 System UI Shell rendered natively in Rust/WASM and Wayland/GLES."),
            ("LineageOS 20 Notes", "Trebuchet Launcher, Notification Shade, Quick Settings, Split-Screen, and Freeform multi-window."),
            ("WebXDC Sandbox", "Zero external network requests. All icons, fonts, layouts and animations run locally."),
        ];
        let tabs_y = bar.y + bar.height + 10.0;
        let tw = (b.width - 36.0) / 3.0;
        for (i, lbl) in ["Overview", "System UI", "WebXDC"].iter().enumerate() {
            let tr = Rect { x: b.x + 12.0 + i as f32 * (tw + 6.0), y: tabs_y, width: tw, height: 32.0 };
            let r = f.target(id + i as u64, tr, lbl, true, Role::Tab);
            let active = self.browser_page == i;
            f.paint_rect(tr, 16.0, if active { p.primary } else { p.card_elevated });
            f.styled_text(tr, lbl, if active { p.on_primary } else { p.on_surface }, text_style(12.5, 500, 1));
            if r.clicked {
                self.browser_page = i;
                self.browser_url = format!("https://spectreos.local/{}", lbl.to_ascii_lowercase().replace(' ', "-"));
            }
        }
        let card = Rect { x: b.x + 14.0, y: tabs_y + 44.0, width: b.width - 28.0, height: 180.0 };
        f.paint_rect(card, 20.0, p.card_pure);
        f.styled_text(Rect { x: card.x + 18.0, y: card.y + 18.0, width: card.width - 36.0, height: 30.0 }, pages[self.browser_page].0, p.primary, text_style(20.0, 600, 0));
        f.paragraph(Rect { x: card.x + 18.0, y: card.y + 56.0, width: card.width - 36.0, height: 108.0 }, pages[self.browser_page].1, p.on_surface, text_style(14.5, 400, 0), 21.0);
        ShellAction::None
    }

    fn render_gallery(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: &Palette) -> ShellAction {
        let photos = [
            ("Mountain Dusk", Color(0x1E, 0x2B, 0x45, 255), Color(0xD8, 0x9C, 0x6E, 255)),
            ("Kyoto Canal", Color(0x2D, 0x52, 0x4A, 255), Color(0xE8, 0xC8, 0xD4, 255)),
            ("Obsidian Sand", Color(0x12, 0x16, 0x1D, 255), Color(0xC7, 0xB7, 0xA6, 255)),
            ("Alpine Ridge", Color(0x2B, 0x48, 0x6E, 255), Color(0xA8, 0xC7, 0xFA, 255)),
            ("Forest Mist", Color(0x14, 0x33, 0x26, 255), Color(0x9C, 0xC9, 0xAA, 255)),
            ("Night Skyline", Color(0x21, 0x1C, 0x38, 255), Color(0xC9, 0xA8, 0xE6, 255)),
        ];
        if let Some(idx) = self.gallery_open {
            let (title, sky, sun) = photos[idx];
            f.paint_rect(b, 0.0, Color(0x0B, 0x0C, 0x10, 255));
            let back = Rect { x: b.x + 10.0, y: b.y + 10.0, width: 42.0, height: 42.0 };
            if f.target(id + 90, back, "Close photo", true, Role::Button).clicked {
                self.gallery_open = None;
            }
            paint_sys_icon(f, back.inset(10.0), SysIcon::Back, Color(255, 255, 255, 255));
            f.styled_text(Rect { x: b.x + 60.0, y: b.y + 10.0, width: b.width - 76.0, height: 42.0 }, title, Color(255, 255, 255, 255), text_style(18.0, 500, 0));
            let art = Rect { x: b.x + 16.0, y: b.y + 76.0, width: b.width - 32.0, height: (b.height - 140.0).max(160.0) };
            paint_photo_card(f, art, sky, sun);
            return ShellAction::None;
        }
        f.paint_rect(b, 0.0, p.settings_bg);
        f.styled_text(Rect { x: b.x + 20.0, y: b.y + 14.0, width: b.width - 40.0, height: 34.0 }, "Gallery", p.on_surface, text_style(24.0, 500, 0));
        let cw = (b.width - 36.0) * 0.5;
        let ch = 128.0;
        for (i, &(title, sky, sun)) in photos.iter().enumerate() {
            let pr = Rect {
                x: b.x + 14.0 + (i % 2) as f32 * (cw + 8.0),
                y: b.y + 58.0 + (i / 2) as f32 * (ch + 10.0),
                width: cw,
                height: ch,
            };
            if pr.y > b.y + b.height {
                break;
            }
            let r = f.target(id + i as u64, pr, title, true, Role::Button);
            paint_photo_card(f, pr, sky, sun);
            f.styled_text(Rect { x: pr.x + 10.0, y: pr.y + pr.height - 26.0, width: pr.width - 20.0, height: 20.0 }, title, Color(255, 255, 255, 240), text_style(12.5, 500, 0));
            if r.clicked {
                self.gallery_open = Some(i);
            }
        }
        ShellAction::None
    }

    fn render_music(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: &Palette) -> ShellAction {
        let now = f.time();
        if self.music_playing {
            let dt = (now - self.music_last).clamp(0.0, 0.25) as f32;
            self.music_last = now;
            self.music_progress = (self.music_progress + dt * 0.03) % 1.0;
            f.continuous(b);
        }
        let tracks = [
            ("Midnight Slate", "SpectreOS Ensemble", Color(0x36, 0x4E, 0x7A, 255)),
            ("Bhoo Resonance", "Material You Sessions", Color(0x7A, 0x2F, 0x52, 255)),
            ("Lineage Pulse", "Open Source Waves", Color(0x25, 0x5E, 0x56, 255)),
        ];
        let (title, artist, art_col) = tracks[self.music_track];
        f.paint_rect(b, 0.0, p.settings_bg);
        let art_sz = (b.width - 72.0).clamp(120.0, 220.0);
        let art = Rect { x: b.x + (b.width - art_sz) * 0.5, y: b.y + 24.0, width: art_sz, height: art_sz };
        f.elevation(art, 24.0, 3);
        f.paint_rect(art, 24.0, art_col);
        paint_sys_icon(f, art.inset(art_sz * 0.30), SysIcon::MusicNote, Color(255, 255, 255, 220));
        let ty = art.y + art.height + 20.0;
        f.styled_text(Rect { x: b.x + 24.0, y: ty, width: b.width - 48.0, height: 28.0 }, title, p.on_surface, text_style(22.0, 600, 1));
        f.styled_text(Rect { x: b.x + 24.0, y: ty + 30.0, width: b.width - 48.0, height: 20.0 }, artist, p.on_surface_variant, text_style(14.0, 400, 1));

        let bar = Rect { x: b.x + 28.0, y: ty + 66.0, width: b.width - 56.0, height: 6.0 };
        f.paint_rect(bar, 3.0, p.surface_container);
        f.paint_rect(Rect { width: bar.width * self.music_progress, ..bar }, 3.0, p.primary);

        let cy = ty + 88.0;
        let cx = b.x + b.width * 0.5;
        let prev = Rect { x: cx - 104.0, y: cy + 8.0, width: 44.0, height: 44.0 };
        if f.target(id + 1, prev, "Previous", true, Role::Button).clicked {
            self.music_track = (self.music_track + 2) % 3;
            self.music_progress = 0.0;
        }
        paint_sys_icon(f, prev.inset(10.0), SysIcon::ChevronLeft, p.on_surface);

        let play = Rect { x: cx - 30.0, y: cy, width: 60.0, height: 60.0 };
        if f.target(id + 2, play, "Play/Pause", true, Role::Button).clicked {
            self.music_playing = !self.music_playing;
            self.music_last = now;
        }
        f.paint_rect(play, 30.0, p.primary);
        f.styled_text(play, if self.music_playing { "❚❚" } else { "▶" }, p.on_primary, text_style(20.0, 600, 1));

        let next = Rect { x: cx + 60.0, y: cy + 8.0, width: 44.0, height: 44.0 };
        if f.target(id + 3, next, "Next", true, Role::Button).clicked {
            self.music_track = (self.music_track + 1) % 3;
            self.music_progress = 0.0;
        }
        paint_sys_icon(f, next.inset(10.0), SysIcon::ChevronRight, p.on_surface);

        let fav = Rect { x: b.x + b.width - 52.0, y: ty, width: 34.0, height: 34.0 };
        if f.target(id + 4, fav, "Favorite", true, Role::ToggleButton).clicked {
            self.music_fav[self.music_track] = !self.music_fav[self.music_track];
        }
        f.paint_material_icon(fav.inset(6.0), MaterialIcon::Star, if self.music_fav[self.music_track] { Color(0xE9, 0x1E, 0x63, 255) } else { p.on_surface_variant });
        ShellAction::None
    }

    fn render_camera(&mut self, f: &mut Frame<'_>, b: Rect, id: u64) -> ShellAction {
        let now = f.time();
        f.paint_rect(b, 0.0, Color(0x0A, 0x0B, 0x0E, 255));
        let vf = Rect { x: b.x + 8.0, y: b.y + 12.0, width: b.width - 16.0, height: (b.height - 146.0).max(160.0) };
        paint_photo_card(f, vf, Color(0x1E, 0x2E, 0x45, 255), Color(0xE0, 0xB8, 0x92, 255));
        // Viewfinder reticle
        let cx = vf.x + vf.width * 0.5;
        let cy = vf.y + vf.height * 0.5;
        f.paint_arc(Rect { x: cx - 26.0, y: cy - 26.0, width: 52.0, height: 52.0 }, 0.0, std::f32::consts::TAU, 1.5, Color(255, 255, 255, 160));
        if self.camera_flash_until > now {
            let a = (((self.camera_flash_until - now) / 0.25).clamp(0.0, 1.0) * 230.0) as u8;
            f.paint_rect(vf, 14.0, Color(255, 255, 255, a));
            f.continuous(b);
        }

        let modes = ["Video", "Photo", "Portrait"];
        let my = vf.y + vf.height + 12.0;
        let mw = b.width / 3.0;
        for (i, lbl) in modes.iter().enumerate() {
            let mr = Rect { x: b.x + i as f32 * mw, y: my, width: mw, height: 28.0 };
            if f.target(id + i as u64, mr, lbl, true, Role::Tab).clicked {
                self.camera_mode = i;
            }
            f.styled_text(mr, lbl, if self.camera_mode == i { Color(255, 255, 255, 255) } else { Color(150, 154, 164, 255) }, text_style(13.5, 600, 1));
        }

        let sh = Rect { x: b.x + b.width * 0.5 - 32.0, y: my + 38.0, width: 64.0, height: 64.0 };
        let rs = f.target(id + 10, sh, "Shutter", true, Role::Button);
        f.paint_rect(sh, 32.0, Color(255, 255, 255, 255));
        f.paint_rect(sh.inset(6.0), 26.0, if self.camera_mode == 0 { Color(0xE0, 0x3E, 0x36, 255) } else { Color(0xEC, 0xEE, 0xF5, 255) });
        if rs.clicked {
            self.camera_shots += 1;
            self.camera_flash_until = now + 0.25;
            return ShellAction::ShowToast(format!("Captured photo #{}", self.camera_shots));
        }
        f.styled_text(Rect { x: b.x + 18.0, y: my + 56.0, width: 90.0, height: 24.0 }, &format!("{} shots", self.camera_shots), Color(180, 184, 194, 255), text_style(12.5, 500, 0));
        ShellAction::None
    }

    pub(super) fn calendar_upcoming(&mut self) -> Option<(String, String)> {
        self.calendar.upcoming()
    }

    pub(super) fn calendar_reminders(&mut self) -> Vec<(String, String)> {
        self.calendar.due_reminders()
    }

    fn render_fdroid(&mut self, f: &mut Frame<'_>, b: Rect, id: u64, p: &Palette) -> ShellAction {
        f.paint_rect(b, 0.0, p.settings_bg);
        f.styled_text(Rect { x: b.x + 20.0, y: b.y + 14.0, width: b.width - 40.0, height: 32.0 }, "F-Droid", p.on_surface, text_style(22.0, 600, 0));
        let catalog = [
            ("ArcaneChat", "Decentralized email-based chat"),
            ("Delta Chat", "Open-source WebXDC messenger"),
            ("NewPipe", "Lightweight media client"),
            ("OsmAnd~", "Offline vector maps & GPS"),
            ("K-9 Mail", "Open-source email client"),
        ];
        for (i, (name, desc)) in catalog.iter().enumerate() {
            let rr = Rect { x: b.x + 14.0, y: b.y + 56.0 + i as f32 * 70.0, width: b.width - 28.0, height: 62.0 };
            if rr.y + rr.height > b.y + b.height {
                break;
            }
            f.paint_rect(rr, 16.0, p.card_elevated);
            f.styled_text(Rect { x: rr.x + 16.0, y: rr.y + 10.0, width: rr.width - 110.0, height: 22.0 }, name, p.on_surface, text_style(15.0, 600, 0));
            f.styled_text(Rect { x: rr.x + 16.0, y: rr.y + 34.0, width: rr.width - 110.0, height: 18.0 }, desc, p.on_surface_variant, text_style(12.0, 400, 0));
            let btn = Rect { x: rr.x + rr.width - 92.0, y: rr.y + 14.0, width: 80.0, height: 34.0 };
            let inst = self.fdroid_installed[i];
            if f.target(id + i as u64, btn, name, true, Role::Button).clicked {
                self.fdroid_installed[i] = !inst;
            }
            f.paint_rect(btn, 17.0, if inst { p.surface_container } else { p.primary });
            f.styled_text(btn, if inst { "Installed" } else { "Install" }, if inst { p.on_surface_variant } else { p.on_primary }, text_style(12.0, 600, 1));
        }
        ShellAction::None
    }

    fn render_recorder(&mut self, f: &mut Frame<'_>, b: Rect, id: u64) -> ShellAction {
        let now = f.time();
        if self.rec_active {
            let dt = (now - self.rec_last).clamp(0.0, 0.25);
            self.rec_last = now;
            self.rec_elapsed += dt;
            f.continuous(b);
        }
        f.paint_rect(b, 0.0, Color(0x22, 0x11, 0x15, 255));
        f.styled_text(Rect { x: b.x + 20.0, y: b.y + 16.0, width: b.width - 40.0, height: 30.0 }, "Recorder", Color(255, 255, 255, 245), text_style(22.0, 500, 0));
        let secs = self.rec_elapsed as u64;
        f.styled_text(
            Rect { x: b.x, y: b.y + 72.0, width: b.width, height: 48.0 },
            &format!("{:02}:{:02}", secs / 60, secs % 60),
            Color(255, 255, 255, 255),
            text_style(42.0, 400, 1),
        );
        // Animated audio waveform bars
        for i in 0..16 {
            let phase = if self.rec_active { (now * 7.0 + i as f64 * 0.7).sin().abs() as f32 } else { 0.15 };
            let bh = 10.0 + phase * 44.0;
            let bx = b.x + b.width * 0.5 - 128.0 + i as f32 * 16.0;
            f.paint_rect(Rect { x: bx, y: b.y + 164.0 - bh * 0.5, width: 6.0, height: bh }, 3.0, Color(0xF0, 0x4E, 0x58, 255));
        }
        let rec_btn = Rect { x: b.x + b.width * 0.5 - 32.0, y: b.y + 216.0, width: 64.0, height: 64.0 };
        if f.target(id + 1, rec_btn, "Record audio", true, Role::Button).clicked {
            if self.rec_active {
                self.rec_active = false;
                self.rec_saved.insert(0, format!("Recording · {:02}:{:02}", secs / 60, secs % 60));
                self.rec_elapsed = 0.0;
            } else {
                self.rec_active = true;
                self.rec_last = now;
            }
        }
        f.paint_rect(rec_btn, 32.0, Color(0xF0, 0x3E, 0x47, 255));
        f.styled_text(rec_btn, if self.rec_active { "■" } else { "●" }, Color(255, 255, 255, 255), text_style(22.0, 600, 1));
        for (i, item) in self.rec_saved.iter().enumerate().take(3) {
            f.styled_text(Rect { x: b.x + 24.0, y: b.y + 300.0 + i as f32 * 30.0, width: b.width - 48.0, height: 24.0 }, item, Color(230, 210, 214, 220), text_style(13.5, 400, 0));
        }
        ShellAction::None
    }

    // =========================================================================
    // CRISP OVERVIEW MINIATURE TASK PREVIEW CARDS (01-recents-main.png / 08-split-pair.png)
    // =========================================================================

    pub fn paint_preview(&self, f: &mut Frame<'_>, app: AppId, b: Rect, rad: f32, sys: &SystemState) {
        let p = sys.palette();
        match app {
            AppId::Contacts => {
                // Exact miniature match to 01-recents-main.png with crisp rounded corners!
                f.paint_rect(b, rad, Color(0xF7, 0xF7, 0xF9, 255));
                let bar_h = 52.0_f32.min(b.height * 0.28);
                super::paint_asym_rect(
                    f,
                    Rect { height: bar_h, ..b },
                    rad,
                    0.0,
                    Color(0x36, 0x52, 0x85, 255),
                );
                let st_h = 18.0_f32.min(bar_h * 0.45);
                f.within(Rect { height: st_h, ..b }, |f| {
                    f.paint_rect(
                        Rect { height: (rad * 2.0 + 4.0).max(bar_h), ..b },
                        rad,
                        Color(0x2E, 0x4A, 0x7D, 255),
                    );
                });
                f.styled_text(
                    Rect { x: b.x + 38.0, y: b.y + 18.0, width: b.width - 76.0, height: 34.0 },
                    "Contacts",
                    Color(255, 255, 255, 255),
                    text_style(14.5, 500, 0),
                );
                let cx = b.x + b.width * 0.5;
                let cy = b.y + b.height * 0.48;
                let circ = Rect { x: cx - 34.0, y: cy - 34.0, width: 68.0, height: 68.0 };
                f.paint_rect(circ, 34.0, Color(0xE2, 0xE2, 0xE6, 255));
                f.paint_rect(circ.inset(16.0), 4.0, Color(0xFF, 0xFF, 0xFF, 255));
                f.styled_text(
                    Rect { x: b.x + 12.0, y: cy + 44.0, width: b.width - 24.0, height: 20.0 },
                    "Contacts · Tap to open",
                    Color(0x5F, 0x63, 0x68, 255),
                    text_style(11.5, 400, 1),
                );
                let fab = Rect { x: b.x + b.width - 48.0, y: b.y + b.height - 48.0, width: 34.0, height: 34.0 };
                f.paint_rect(fab, 17.0, Color(0x43, 0x5E, 0x91, 255));
                f.styled_text(fab, "+", Color(255, 255, 255, 255), text_style(18.0, 500, 1));
            }
            AppId::Clock => {
                // Exact miniature match to 01-recents-main.png left card & 08-recents-split-pair.png!
                f.paint_rect(b, rad, Color(0x19, 0x19, 0x1D, 255));
                f.styled_text(Rect { x: b.x + 16.0, y: b.y + 18.0, width: b.width - 32.0, height: 22.0 }, "Alarm", Color(240, 240, 245, 255), text_style(14.5, 500, 0));
                for (i, a) in self.alarms.iter().take(2).enumerate() {
                    let cr = Rect { x: b.x + 12.0, y: b.y + 52.0 + i as f32 * 76.0, width: b.width - 24.0, height: 64.0 };
                    if cr.y + cr.height > b.y + b.height - 64.0 {
                        break;
                    }
                    f.paint_rect(cr, 16.0, Color(0x28, 0x29, 0x30, 255));
                    f.styled_text(
                        Rect { x: cr.x + 14.0, y: cr.y + 8.0, width: 120.0, height: 28.0 },
                        &format!("{}:{:02} {}", a.hour, a.minute, if a.am { "AM" } else { "PM" }),
                        Color(225, 225, 232, 255),
                        text_style(20.0, 400, 0),
                    );
                    f.styled_text(Rect { x: cr.x + 14.0, y: cr.y + 38.0, width: cr.width - 60.0, height: 16.0 }, &a.days, Color(150, 152, 162, 255), text_style(10.0, 400, 0));
                }
                // Bottom dark navigation bar strip matching real Android Clock in 01-recents-main.png!
                let bar_h = 42.0_f32.min(b.height * 0.14);
                super::paint_asym_rect(
                    f,
                    Rect { x: b.x, y: b.y + b.height - bar_h, width: b.width, height: bar_h },
                    0.0,
                    rad,
                    Color(0x22, 0x23, 0x2A, 255),
                );
                let fab = Rect { x: b.x + b.width * 0.5 - 22.0, y: b.y + b.height - bar_h - 56.0, width: 44.0, height: 44.0 };
                f.paint_rect(fab, 22.0, Color(0xDE, 0xC0, 0xE5, 255));
                f.styled_text(fab, "+", Color(24, 20, 28, 255), text_style(20.0, 500, 1));
            }
            AppId::Settings => {
                f.paint_rect(b, rad, p.settings_bg);
                f.styled_text(Rect { x: b.x + 18.0, y: b.y + 28.0, width: b.width - 36.0, height: 30.0 }, "Settings", p.on_surface, text_style(22.0, 400, 0));
                f.paint_rect(Rect { x: b.x + 14.0, y: b.y + 68.0, width: b.width - 28.0, height: 34.0 }, 17.0, p.card_pure);
                for (i, lbl) in ["Network & internet", "Connected devices", "Apps", "Notifications", "Battery", "Storage", "Sound & vibration", "Display"].iter().enumerate() {
                    let y = b.y + 116.0 + i as f32 * 44.0;
                    if y + 32.0 > b.y + b.height - 12.0 {
                        break;
                    }
                    f.styled_text(Rect { x: b.x + 20.0, y, width: b.width - 40.0, height: 24.0 }, lbl, p.on_surface, text_style(12.5, 400, 0));
                }
            }
            AppId::Gmail => {
                f.paint_rect(b, rad, Color(0xFF, 0xFF, 0xFF, 255));
                f.paint_rect(Rect { x: b.x + 14.0, y: b.y + 18.0, width: b.width - 28.0, height: 34.0 }, 17.0, Color(0xEE, 0xF2, 0xF9, 255));
                f.styled_text(Rect { x: b.x + 26.0, y: b.y + 18.0, width: b.width - 52.0, height: 34.0 }, "Search in mail", Color(0x5F, 0x63, 0x68, 255), text_style(12.0, 400, 0));
                for (i, (who, subj)) in [("Sam", "Photos from my trip!"), ("Square", "Account update"), ("Vercel", "December Update"), ("GitHub", "PR #142 merged")].iter().enumerate() {
                    let y = b.y + 66.0 + i as f32 * 52.0;
                    if y + 40.0 > b.y + b.height - 16.0 {
                        break;
                    }
                    f.styled_text(Rect { x: b.x + 20.0, y, width: b.width - 40.0, height: 18.0 }, who, Color(0x1F, 0x1F, 0x1F, 255), text_style(12.5, 600, 0));
                    f.styled_text(Rect { x: b.x + 20.0, y: y + 18.0, width: b.width - 40.0, height: 16.0 }, subj, Color(0x5F, 0x63, 0x68, 255), text_style(11.0, 400, 0));
                }
            }
            AppId::Calculator => {
                f.paint_rect(b, rad, p.settings_bg);
                super::paint_asym_rect(
                    f,
                    Rect { x: b.x, y: b.y, width: b.width, height: b.height * 0.34 },
                    rad,
                    16.0,
                    p.primary_container.opacity(0.55),
                );
                f.styled_text(Rect { x: b.x + 14.0, y: b.y + b.height * 0.16, width: b.width - 28.0, height: 32.0 }, &self.calc_display, p.on_surface, text_style(26.0, 400, 2));
                let cw = (b.width - 30.0) / 4.0;
                for r in 0..4 {
                    for c in 0..4 {
                        let kr = Rect { x: b.x + 12.0 + c as f32 * (cw + 2.5), y: b.y + b.height * 0.38 + r as f32 * 44.0, width: cw, height: 36.0 };
                        if kr.y + kr.height > b.y + b.height - 16.0 {
                            break;
                        }
                        f.paint_rect(kr, 18.0, if c == 3 { p.primary_container } else { p.card_elevated });
                    }
                }
            }
            _ => {
                f.paint_rect(b, rad, p.settings_bg);
                super::paint_asym_rect(
                    f,
                    Rect { height: 48.0_f32.min(b.height * 0.25), ..b },
                    rad,
                    0.0,
                    p.primary,
                );
                f.styled_text(Rect { x: b.x + 18.0, y: b.y + 8.0, width: b.width - 36.0, height: 36.0 }, app.name(), p.on_primary, text_style(14.5, 500, 0));
                f.styled_text(Rect { x: b.x + 18.0, y: b.y + b.height * 0.45, width: b.width - 36.0, height: 24.0 }, "Tap to resume", p.on_surface_variant, text_style(13.0, 400, 1));
            }
        }
    }
}

fn paint_photo_card(f: &mut Frame<'_>, r: Rect, sky: Color, sun: Color) {
    f.within(r, |f| {
        f.paint_rect(r, 14.0, sky);
        f.paint_rect(
            Rect { x: r.x + r.width * 0.56, y: r.y + r.height * 0.14, width: r.width * 0.28, height: r.width * 0.28 },
            r.width * 0.14,
            sun,
        );
        f.paint_rect(
            Rect { x: r.x - r.width * 0.12, y: r.y + r.height * 0.54, width: r.width * 0.86, height: r.height * 0.7 },
            r.width * 0.22,
            sky.mix(Color(0, 0, 0, 255), 0.42),
        );
        f.paint_rect(
            Rect { x: r.x + r.width * 0.36, y: r.y + r.height * 0.62, width: r.width * 0.80, height: r.height * 0.6 },
            r.width * 0.20,
            sky.mix(Color(0, 0, 0, 255), 0.62),
        );
    });
}

fn trim_float(v: f64) -> String {
    let s = format!("{v:.8}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" || s.is_empty() {
        "0".into()
    } else {
        s.into()
    }
}

pub(super) fn files_appearance(p: &Palette) -> spectre_files::Appearance {
    spectre_files::Appearance {
        background: p.settings_bg, surface: p.card_pure, raised: p.card_elevated, text: p.on_surface,
        muted: p.on_surface_variant, primary: p.primary, selected: p.primary_container,
        on_primary: p.on_primary, outline: p.outline,
    }
}
