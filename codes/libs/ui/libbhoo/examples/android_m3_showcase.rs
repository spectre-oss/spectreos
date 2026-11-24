//! Native Wayland + GLES Material 3 multi-widget showcase capture at `720 x 1280` (`density = 1.5`).
//! Supports both Material 3 Light (default) and Material 3 Dark (`BHOO_THEME=dark` or `--dark`).
//! Demonstrates Toolbar, M3 SearchBar, TextInputLayout (outlined + filled), MaterialButton (all 5 M3
//! variants), MaterialButtonToggleGroup (segmented buttons), Chips (assist, filter checked, input
//! with close), CheckBox, RadioButton, Switch, Spinner, SeekBar, RatingBar, ProgressBar (linear +
//! circular), MaterialCardView with `RULE_CLIP_TO_OUTLINE`, ConstraintLayout with `RULE_FLOW` +
//! `RULE_CONSTRAINT_CIRCLE`, Floating Action Button, and M3 bottom `NavigationBar`.
use bhoo::{
    android_xml::{
        AndroidShowAsAction, AndroidTheme, AndroidToolbarMenuItem, Kind, LayoutState, Node,
        RULE_ALERT_DIALOG, RULE_BOTTOM_APP_BAR, RULE_BOTTOM_SHEET, RULE_BUTTON_VARIANT,
        RULE_CHIP_GROUP_WRAP, RULE_CHIP_STYLE, RULE_CLIP_TO_OUTLINE, RULE_COLLAPSING_TOOLBAR,
        RULE_CONSTRAINT_CIRCLE, RULE_CONSTRAINT_END_TO_END, RULE_CONSTRAINT_START_TO_START,
        RULE_CONSTRAINT_TOP_TO_BOTTOM, RULE_CONSTRAINT_TOP_TO_TOP, RULE_CORNER_RADII,
        RULE_DRAG_HANDLE, RULE_DRAWER_LAYOUT, RULE_ELEVATION, RULE_EXTENDED_FAB,
        RULE_FLOATING_TOOLBAR, RULE_FLOW, RULE_INCLUDE_FONT_PADDING, RULE_LOADING_INDICATOR,
        RULE_NAV_BADGE, RULE_NAV_BAR, RULE_NAV_ITEM, RULE_RANGE_SLIDER, RULE_SEARCH_BAR,
        RULE_SLIDER_OPTIONS, RULE_SNACKBAR, RULE_SPLIT_BUTTON, RULE_SWIPE_REFRESH,
        RULE_TEXT_INPUT_BOX, RULE_TEXT_INPUT_EXT, RULE_TEXT_STYLE_EXT, RULE_TOGGLE_GROUP,
        RULE_VIEW_PAGER, RULE_VIEW_TRANSFORM, RULE_WAVY_PROGRESS, RelativeRule, Size, Visibility,
    },
    window::{self, FrameAction, WindowOptions},
    *,
};

#[derive(Clone, Copy)]
struct ShowcasePalette {
    on_surface_variant: Color,
    outline_variant: Color,
    surface_low: Color,
    primary: Color,
    on_primary: Color,
    primary_container: Color,
    on_primary_container: Color,
    secondary_container: Color,
    on_secondary_container: Color,
}

const LIGHT_PALETTE: ShowcasePalette = ShowcasePalette {
    on_surface_variant: Color(0x49, 0x45, 0x4F, 255),
    outline_variant: Color(0xCA, 0xC4, 0xD0, 255),
    surface_low: Color(0xF7, 0xF2, 0xFA, 255),
    primary: Color(0x67, 0x50, 0xA4, 255),
    on_primary: Color(0xFF, 0xFF, 0xFF, 255),
    primary_container: Color(0xEA, 0xDD, 0xFF, 255),
    on_primary_container: Color(0x21, 0x00, 0x5D, 255),
    secondary_container: Color(0xE8, 0xDE, 0xF8, 255),
    on_secondary_container: Color(0x1D, 0x19, 0x2B, 255),
};

const DARK_PALETTE: ShowcasePalette = ShowcasePalette {
    on_surface_variant: Color(0xCA, 0xC4, 0xD0, 255),
    outline_variant: Color(0x49, 0x45, 0x4F, 255),
    surface_low: Color(0x1D, 0x1B, 0x20, 255),
    primary: Color(0xD0, 0xBC, 0xFF, 255),
    on_primary: Color(0x38, 0x1E, 0x72, 255),
    primary_container: Color(0x4F, 0x37, 0x8B, 255),
    on_primary_container: Color(0xEA, 0xDD, 0xFF, 255),
    secondary_container: Color(0x4A, 0x44, 0x58, 255),
    on_secondary_container: Color(0xE8, 0xDE, 0xF8, 255),
};

const fn base_node(kind: Kind, id: &'static str, width: Size, height: Size) -> Node {
    Node {
        kind,
        id,
        width,
        height,
        min_width: 0.0,
        min_height: 0.0,
        horizontal: false,
        measure_all_children: false,
        layout_direction: 0,
        fill_viewport: false,
        clip_to_padding_disabled: false,
        clip_children_disabled: false,
        weight: 0.0,
        weight_sum: 0.0,
        divider_color: None,
        divider_thickness: 0.0,
        divider_flags: 0,
        divider_padding: 0.0,
        padding_edges: [0.0; 4],
        padding_explicit: [false; 4],
        background_padding: None,
        background_min_size: [0.0; 2],
        padding_relative: [None; 2],
        margin: [0.0; 4],
        margin_relative: [None; 2],
        layout_gravity: [0, 0],
        absolute_position: [0.0; 2],
        relative_rules: &[],
        layout_gravity_explicit: false,
        content_gravity: [0, 0],
        text: "",
        hint: "",
        chrono_format: "",
        count_down: false,
        clock_12: "",
        clock_24: "",
        input_type: 0,
        text_on: "ON",
        text_off: "OFF",
        text_size: 14.0,
        letter_spacing_em: 0.0,
        line_spacing_extra: 0.0,
        line_spacing_multiplier: 1.0,
        text_bold: false,
        text_all_caps: false,
        max_lines: 0,
        min_lines: 1,
        ellipsize_end: false,
        ellipsize_side: 0,
        single_line: false,
        align: 0,
        color: None,
        text_color_states: &[],
        background: None,
        foreground: None,
        foreground_radius: 0.0,
        foreground_size: [0.0; 2],
        foreground_padding: None,
        foreground_gravity: [5, 5],
        foreground_excludes_padding: false,
        background_stroke: None,
        background_stroke_width: 0.0,
        foreground_stroke: None,
        foreground_stroke_width: 0.0,
        radius: 0.0,
        clickable: false,
        icon: MaterialIcon::Add,
        image_src: "",
        image_scale: 0,
        image_adjust_bounds: false,
        image_max_width: None,
        image_max_height: None,
        image_tint: None,
        image_tint_states: &[],
        image_tint_mode: 0,
        image_alpha: 255,
        view_alpha: 1.0,
        image_builtin: true,
        enabled: true,
        checked: false,
        checked_button: "",
        min: 0.0,
        max: 100.0,
        progress: 0.0,
        secondary_progress: 0.0,
        indeterminate: false,
        num_stars: 5,
        rating: 0.0,
        step_size: 0.5,
        indicator: false,
        mirror_for_rtl: false,
        animation_in: None,
        animation_out: None,
        translate_in: None,
        translate_out: None,
        auto_start: false,
        flip_interval: 3.0,
        visibility: Visibility::Visible,
        children: &[],
    }
}

static TIGHT_TEXT_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_INCLUDE_FONT_PADDING,
    anchor: "false",
}];

const fn section_title(id: &'static str, text: &'static str, p: ShowcasePalette) -> Node {
    Node {
        text,
        text_size: 11.0,
        text_bold: true,
        color: Some(p.on_surface_variant),
        margin: [0.0, 3.0, 0.0, 1.0],
        relative_rules: TIGHT_TEXT_RULES,
        ..base_node(Kind::Text, id, Size::Match, Size::Wrap)
    }
}

static SEARCH_BAR_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_SEARCH_BAR,
    anchor: "Search Material 3 widgets...",
}];

static TIL_OUTLINE_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_TEXT_INPUT_BOX,
    anchor: "outline|4||Email|Work email address|1|clear_text",
}];
static TIL_FILLED_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_TEXT_INPUT_BOX,
    anchor: "filled|4||Password|At least 8 chars|0|password_toggle",
}];
static TIL_OUTLINE_CHILDREN: [Node; 1] = [Node {
    text: "alex@spectre.os",
    hint: "Email",
    text_size: 14.0,
    ..base_node(Kind::Edit, "email_edit", Size::Match, Size::Match)
}];
static TIL_FILLED_CHILDREN: [Node; 1] = [Node {
    text: "spectre9",
    hint: "Password",
    text_size: 14.0,
    input_type: 4,
    ..base_node(Kind::Edit, "pass_edit", Size::Match, Size::Match)
}];

static BTN_FILLED_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "filled" }];
static BTN_TONAL_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "tonal" }];
static BTN_ELEVATED_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "elevated" }];
static BTN_OUTLINED_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "outlined" }];
static BTN_TEXT_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "text" }];

static TOGGLE_GROUP_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_TOGGLE_GROUP,
    anchor: "1|1|seg_week",
}];

static CHIP_ASSIST_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CHIP_STYLE, anchor: "assist|0" }];
static CHIP_FILTER_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CHIP_STYLE, anchor: "filter|0" }];
static CHIP_INPUT_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CHIP_STYLE, anchor: "input|1" }];

static SPINNER_RULES: &[RelativeRule] = &[
    RelativeRule { kind: 82, anchor: "Standard" },
    RelativeRule { kind: 82, anchor: "Compact" },
    RelativeRule { kind: 82, anchor: "Expanded" },
];

static CARD_CLIP_RULES: &[RelativeRule] = &[
    RelativeRule { kind: 57, anchor: "Outlined" },
    RelativeRule { kind: 66, anchor: "16" },
    RelativeRule { kind: RULE_CLIP_TO_OUTLINE, anchor: "true" },
];

static CL_TITLE_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
    RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
];
static CL_FLOW_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_CONSTRAINT_START_TO_START, anchor: "parent" },
    RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_BOTTOM, anchor: "cl_flow_title" },
    RelativeRule { kind: RULE_FLOW, anchor: "0|chain|6|6|0|fl_1,fl_2,fl_3,fl_4,fl_5" },
];
static CL_HUB_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_CONSTRAINT_END_TO_END, anchor: "parent" },
    RelativeRule { kind: RULE_CONSTRAINT_TOP_TO_TOP, anchor: "parent" },
];
static ORBIT_0_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "orbit_hub,32,0" }];
static ORBIT_90_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "orbit_hub,32,90" }];
static ORBIT_180_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "orbit_hub,32,180" }];
static ORBIT_270_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_CONSTRAINT_CIRCLE, anchor: "orbit_hub,32,270" }];

static NAV_BAR_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_NAV_BAR, anchor: "bar|labeled" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_home|Home|home" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_widgets|Widgets|widgets" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_saved|Saved|star" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "nav_settings|Settings|settings" },
];

static FAB_RULES: &[RelativeRule] = &[RelativeRule { kind: RULE_ELEVATION, anchor: "6" }];

const fn flow_pill(id: &'static str, label: &'static str, w: f32, p: ShowcasePalette) -> Node {
    Node {
        text: label,
        text_size: 11.0,
        text_bold: true,
        align: 1,
        content_gravity: [1, 1],
        color: Some(p.on_secondary_container),
        background: Some(p.secondary_container),
        radius: 8.0,
        ..base_node(Kind::Text, id, Size::Dp(w), Size::Dp(24.0))
    }
}

const fn orbit_badge(
    id: &'static str,
    label: &'static str,
    rules: &'static [RelativeRule],
    p: ShowcasePalette,
) -> Node {
    Node {
        text: label,
        text_size: 9.0,
        text_bold: true,
        single_line: true,
        align: 1,
        content_gravity: [1, 1],
        color: Some(p.on_primary_container),
        background: Some(p.primary_container),
        background_stroke: Some(p.primary),
        background_stroke_width: 1.0,
        radius: -1.0,
        relative_rules: rules,
        ..base_node(Kind::Text, id, Size::Dp(26.0), Size::Dp(26.0))
    }
}

macro_rules! define_showcase {
    ($name:ident, $palette:expr, $is_dark:expr) => {
        static $name: Node = {
            const P: ShowcasePalette = $palette;
            Node {
                kind: Kind::Frame,
                id: "showcase_root",
                width: Size::Match,
                height: Size::Match,
                children: &[
                    // Main vertical content column + docked M3 NavigationBar
                    Node {
                        kind: Kind::Linear,
                        id: "main_col",
                        width: Size::Match,
                        height: Size::Match,
                        children: &[
                            // 1. M3 Top App Bar (Toolbar)
                            Node {
                                text: "Material 3 Showcase",
                                hint: "Android XML · Bhoo Runtime",
                                ..base_node(Kind::Toolbar, "top_bar", Size::Match, Size::Dp(52.0))
                            },
                            Node {
                                divider_color: Some(P.outline_variant),
                                divider_thickness: 1.0,
                                ..base_node(Kind::MaterialDivider, "bar_div", Size::Match, Size::Dp(1.0))
                            },
                            // Padded showcase body (weighted to sit cleanly above bottom NavigationBar)
                            Node {
                                kind: Kind::Linear,
                                id: "body",
                                weight: 1.0,
                                width: Size::Match,
                                height: Size::Dp(0.0),
                                padding_edges: [16.0, 6.0, 16.0, 4.0],
                                padding_explicit: [true; 4],
                                children: &[
                                    // 2. Material 3 SearchBar (RULE_SEARCH_BAR)
                                    Node {
                                        margin: [0.0, 2.0, 0.0, 2.0],
                                        relative_rules: SEARCH_BAR_RULES,
                                        ..base_node(Kind::SearchView, "m3_search", Size::Match, Size::Dp(48.0))
                                    },
                                    // 3. TextInputLayout: Outlined + Filled
                                    section_title("sec_fields", "TextInputLayout (Outlined & Filled)", P),
                                    Node {
                                        kind: Kind::Linear,
                                        id: "til_row",
                                        horizontal: true,
                                        width: Size::Match,
                                        height: Size::Dp(62.0),
                                        children: &[
                                            Node {
                                                weight: 1.0,
                                                margin: [0.0, 0.0, 10.0, 0.0],
                                                relative_rules: TIL_OUTLINE_RULES,
                                                children: &TIL_OUTLINE_CHILDREN,
                                                ..base_node(Kind::Linear, "til_outline", Size::Dp(0.0), Size::Match)
                                            },
                                            Node {
                                                weight: 1.0,
                                                relative_rules: TIL_FILLED_RULES,
                                                children: &TIL_FILLED_CHILDREN,
                                                ..base_node(Kind::Linear, "til_filled", Size::Dp(0.0), Size::Match)
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "til_row", Size::Match, Size::Dp(62.0))
                                    },
                                    // 4. MaterialButton: all 5 M3 variants
                                    section_title("sec_buttons", "MaterialButton Variants", P),
                                    Node {
                                        kind: Kind::Linear,
                                        id: "btn_row",
                                        horizontal: true,
                                        width: Size::Match,
                                        height: Size::Dp(40.0),
                                        children: &[
                                            Node {
                                                text: "Filled",
                                                padding_edges: [14.0, 6.0, 14.0, 6.0],
                                                padding_explicit: [true; 4],
                                                margin: [0.0, 0.0, 6.0, 0.0],
                                                relative_rules: BTN_FILLED_RULES,
                                                ..base_node(Kind::Button, "btn_filled", Size::Wrap, Size::Dp(40.0))
                                            },
                                            Node {
                                                text: "Tonal",
                                                padding_edges: [14.0, 6.0, 14.0, 6.0],
                                                padding_explicit: [true; 4],
                                                margin: [0.0, 0.0, 6.0, 0.0],
                                                relative_rules: BTN_TONAL_RULES,
                                                ..base_node(Kind::Button, "btn_tonal", Size::Wrap, Size::Dp(40.0))
                                            },
                                            Node {
                                                text: "Elevated",
                                                padding_edges: [14.0, 6.0, 14.0, 6.0],
                                                padding_explicit: [true; 4],
                                                margin: [0.0, 0.0, 6.0, 0.0],
                                                relative_rules: BTN_ELEVATED_RULES,
                                                ..base_node(Kind::Button, "btn_elevated", Size::Wrap, Size::Dp(40.0))
                                            },
                                            Node {
                                                text: "Outlined",
                                                padding_edges: [14.0, 6.0, 14.0, 6.0],
                                                padding_explicit: [true; 4],
                                                margin: [0.0, 0.0, 6.0, 0.0],
                                                relative_rules: BTN_OUTLINED_RULES,
                                                ..base_node(Kind::Button, "btn_outlined", Size::Wrap, Size::Dp(40.0))
                                            },
                                            Node {
                                                text: "Text",
                                                padding_edges: [10.0, 6.0, 10.0, 6.0],
                                                padding_explicit: [true; 4],
                                                relative_rules: BTN_TEXT_RULES,
                                                ..base_node(Kind::Button, "btn_text", Size::Wrap, Size::Dp(40.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "btn_row", Size::Match, Size::Dp(40.0))
                                    },
                                    // 5. MaterialButtonToggleGroup (Segmented Button)
                                    section_title("sec_toggle", "MaterialButtonToggleGroup (Segmented)", P),
                                    Node {
                                        kind: Kind::Linear,
                                        id: "toggle_group",
                                        horizontal: true,
                                        width: Size::Match,
                                        height: Size::Dp(38.0),
                                        relative_rules: TOGGLE_GROUP_RULES,
                                        children: &[
                                            Node {
                                                weight: 1.0,
                                                text: "Day",
                                                relative_rules: BTN_OUTLINED_RULES,
                                                ..base_node(Kind::Button, "seg_day", Size::Dp(0.0), Size::Match)
                                            },
                                            Node {
                                                weight: 1.0,
                                                text: "Week",
                                                relative_rules: BTN_OUTLINED_RULES,
                                                ..base_node(Kind::Button, "seg_week", Size::Dp(0.0), Size::Match)
                                            },
                                            Node {
                                                weight: 1.0,
                                                text: "Month",
                                                relative_rules: BTN_OUTLINED_RULES,
                                                ..base_node(Kind::Button, "seg_month", Size::Dp(0.0), Size::Match)
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "toggle_group", Size::Match, Size::Dp(38.0))
                                    },
                                    // 6. Material 3 Chips
                                    section_title("sec_chips", "Chips (Assist · Filter · Input)", P),
                                    Node {
                                        kind: Kind::Linear,
                                        id: "chip_row",
                                        horizontal: true,
                                        width: Size::Match,
                                        height: Size::Dp(34.0),
                                        children: &[
                                            Node {
                                                text: "Assist",
                                                margin: [0.0, 0.0, 8.0, 0.0],
                                                relative_rules: CHIP_ASSIST_RULES,
                                                ..base_node(Kind::Button, "chip_assist", Size::Wrap, Size::Dp(34.0))
                                            },
                                            Node {
                                                text: "Selected",
                                                checked: true,
                                                margin: [0.0, 0.0, 8.0, 0.0],
                                                relative_rules: CHIP_FILTER_RULES,
                                                ..base_node(Kind::Button, "chip_filter_on", Size::Wrap, Size::Dp(34.0))
                                            },
                                            Node {
                                                text: "Filter",
                                                checked: false,
                                                margin: [0.0, 0.0, 8.0, 0.0],
                                                relative_rules: CHIP_FILTER_RULES,
                                                ..base_node(Kind::Button, "chip_filter_off", Size::Wrap, Size::Dp(34.0))
                                            },
                                            Node {
                                                text: "Tag",
                                                relative_rules: CHIP_INPUT_RULES,
                                                ..base_node(Kind::Button, "chip_input", Size::Wrap, Size::Dp(34.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "chip_row", Size::Match, Size::Dp(34.0))
                                    },
                                    // 7. Form controls: CheckBox, RadioButton, Switch, Spinner
                                    section_title("sec_controls", "Selection & Input Controls", P),
                                    Node {
                                        kind: Kind::Linear,
                                        id: "controls_cols",
                                        horizontal: true,
                                        width: Size::Match,
                                        height: Size::Dp(96.0),
                                        children: &[
                                            Node {
                                                weight: 1.0,
                                                margin: [0.0, 0.0, 12.0, 0.0],
                                                children: &[
                                                    Node {
                                                        text: "Notifications",
                                                        checked: true,
                                                        ..base_node(Kind::CheckBox, "cb_on", Size::Match, Size::Dp(32.0))
                                                    },
                                                    Node {
                                                        text: "Auto-sync",
                                                        checked: false,
                                                        ..base_node(Kind::CheckBox, "cb_off", Size::Match, Size::Dp(32.0))
                                                    },
                                                    Node {
                                                        horizontal: true,
                                                        children: &[
                                                            Node {
                                                                weight: 1.0,
                                                                text: "Light",
                                                                checked: !$is_dark,
                                                                ..base_node(Kind::RadioButton, "rb_light", Size::Dp(0.0), Size::Dp(32.0))
                                                            },
                                                            Node {
                                                                weight: 1.0,
                                                                text: "Dark",
                                                                checked: $is_dark,
                                                                ..base_node(Kind::RadioButton, "rb_dark", Size::Dp(0.0), Size::Dp(32.0))
                                                            },
                                                        ],
                                                        ..base_node(Kind::RadioGroup, "rg_theme", Size::Match, Size::Dp(32.0))
                                                    },
                                                ],
                                                ..base_node(Kind::Linear, "controls_left", Size::Dp(0.0), Size::Match)
                                            },
                                            Node {
                                                weight: 1.0,
                                                children: &[
                                                    Node {
                                                        text: "Wi-Fi",
                                                        checked: true,
                                                        ..base_node(Kind::Switch, "sw_on", Size::Match, Size::Dp(32.0))
                                                    },
                                                    Node {
                                                        text: "Airplane",
                                                        checked: false,
                                                        ..base_node(Kind::Switch, "sw_off", Size::Match, Size::Dp(32.0))
                                                    },
                                                    Node {
                                                        margin: [0.0, 1.0, 0.0, 0.0],
                                                        relative_rules: SPINNER_RULES,
                                                        ..base_node(Kind::Spinner, "sp_density", Size::Match, Size::Dp(31.0))
                                                    },
                                                ],
                                                ..base_node(Kind::Linear, "controls_right", Size::Dp(0.0), Size::Match)
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "controls_cols", Size::Match, Size::Dp(96.0))
                                    },
                                    // 8. SeekBar, RatingBar, ProgressBar (linear + circular)
                                    section_title("sec_progress", "Slider, Rating & Progress", P),
                                    Node {
                                        kind: Kind::Linear,
                                        id: "slider_row",
                                        horizontal: true,
                                        width: Size::Match,
                                        height: Size::Dp(30.0),
                                        children: &[
                                            Node {
                                                weight: 1.15,
                                                progress: 65.0,
                                                secondary_progress: 85.0,
                                                margin: [0.0, 0.0, 12.0, 0.0],
                                                ..base_node(Kind::SeekBar, "seek_main", Size::Dp(0.0), Size::Dp(30.0))
                                            },
                                            Node {
                                                weight: 0.85,
                                                num_stars: 5,
                                                rating: 3.5,
                                                ..base_node(Kind::RatingBar, "rate_main", Size::Dp(0.0), Size::Dp(30.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "slider_row", Size::Match, Size::Dp(30.0))
                                    },
                                    Node {
                                        kind: Kind::Linear,
                                        id: "progress_row",
                                        horizontal: true,
                                        content_gravity: [0, 1],
                                        width: Size::Match,
                                        height: Size::Dp(30.0),
                                        children: &[
                                            Node {
                                                weight: 1.0,
                                                horizontal: true,
                                                indeterminate: false,
                                                progress: 68.0,
                                                secondary_progress: 86.0,
                                                margin: [0.0, 0.0, 16.0, 0.0],
                                                ..base_node(Kind::ProgressBar, "pb_linear", Size::Dp(0.0), Size::Dp(20.0))
                                            },
                                            Node {
                                                horizontal: false,
                                                indeterminate: false,
                                                progress: 72.0,
                                                ..base_node(Kind::ProgressBar, "pb_circular", Size::Dp(28.0), Size::Dp(28.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "progress_row", Size::Match, Size::Dp(30.0))
                                    },
                                    // 9. MaterialCardView with RULE_CLIP_TO_OUTLINE
                                    section_title("sec_card", "MaterialCardView (clipToOutline)", P),
                                    Node {
                                        relative_rules: CARD_CLIP_RULES,
                                        margin: [0.0, 0.0, 0.0, 1.0],
                                        children: &[Node {
                                            children: &[
                                                Node {
                                                    background: Some(P.primary_container),
                                                    padding_edges: [14.0, 6.0, 14.0, 5.0],
                                                    padding_explicit: [true; 4],
                                                    children: &[Node {
                                                        text: "Clipped Hero Banner",
                                                        text_size: 14.0,
                                                        text_bold: true,
                                                        color: Some(P.on_primary_container),
                                                        ..base_node(Kind::Text, "card_banner_title", Size::Match, Size::Wrap)
                                                    }],
                                                    ..base_node(Kind::Linear, "card_banner", Size::Match, Size::Dp(30.0))
                                                },
                                                Node {
                                                    padding_edges: [14.0, 5.0, 14.0, 5.0],
                                                    padding_explicit: [true; 4],
                                                    children: &[Node {
                                                        text: "Full-bleed banner clipped smoothly to the 16dp rounded card outline.",
                                                        text_size: 12.0,
                                                        color: Some(P.on_surface_variant),
                                                        ..base_node(Kind::Text, "card_body_text", Size::Match, Size::Wrap)
                                                    }],
                                                    ..base_node(Kind::Linear, "card_body", Size::Match, Size::Wrap)
                                                },
                                            ],
                                            ..base_node(Kind::Linear, "card_col", Size::Match, Size::Match)
                                        }],
                                        ..base_node(Kind::MaterialCard, "m3_card", Size::Match, Size::Dp(66.0))
                                    },
                                    // 10. ConstraintLayout with Flow + Circular Positioning
                                    section_title("sec_cl", "ConstraintLayout (Flow + Circular Orbit)", P),
                                    Node {
                                        background: Some(P.surface_low),
                                        background_stroke: Some(P.outline_variant),
                                        background_stroke_width: 1.0,
                                        radius: 12.0,
                                        padding_edges: [10.0, 6.0, 10.0, 6.0],
                                        padding_explicit: [true; 4],
                                        children: &[
                                            Node {
                                                text: "Flow (wrap=chain, 6dp gaps)",
                                                text_size: 11.0,
                                                color: Some(P.on_surface_variant),
                                                relative_rules: CL_TITLE_RULES,
                                                ..base_node(Kind::Text, "cl_flow_title", Size::Wrap, Size::Wrap)
                                            },
                                            Node {
                                                margin: [0.0, 6.0, 0.0, 0.0],
                                                relative_rules: CL_FLOW_RULES,
                                                ..base_node(Kind::View, "cl_flow", Size::Dp(208.0), Size::Wrap)
                                            },
                                            flow_pill("fl_1", "Rust", 50.0, P),
                                            flow_pill("fl_2", "GLES 3", 58.0, P),
                                            flow_pill("fl_3", "Wayland", 68.0, P),
                                            flow_pill("fl_4", "Material 3", 76.0, P),
                                            flow_pill("fl_5", "XML", 44.0, P),
                                            // Orbit hub + 4 circular badges (0°, 90°, 180°, 270°), clear of FAB
                                            Node {
                                                text: "Hub",
                                                text_size: 10.0,
                                                text_bold: true,
                                                align: 1,
                                                content_gravity: [1, 1],
                                                color: Some(P.on_primary),
                                                background: Some(P.primary),
                                                radius: -1.0,
                                                margin: [0.0, 32.0, 108.0, 0.0],
                                                relative_rules: CL_HUB_RULES,
                                                ..base_node(Kind::Text, "orbit_hub", Size::Dp(26.0), Size::Dp(26.0))
                                            },
                                            orbit_badge("orb_0", "0°", ORBIT_0_RULES, P),
                                            orbit_badge("orb_90", "90°", ORBIT_90_RULES, P),
                                            orbit_badge("orb_180", "180°", ORBIT_180_RULES, P),
                                            orbit_badge("orb_270", "270°", ORBIT_270_RULES, P),
                                        ],
                                        ..base_node(Kind::ConstraintLayout, "cl_demo", Size::Match, Size::Dp(104.0))
                                    },
                                ],
                                ..base_node(Kind::Linear, "body", Size::Match, Size::Dp(0.0))
                            },
                            // 11. Material 3 Bottom NavigationBar (RULE_NAV_BAR + RULE_NAV_ITEM)
                            Node {
                                relative_rules: NAV_BAR_RULES,
                                ..base_node(Kind::Linear, "bottom_nav", Size::Match, Size::Wrap)
                            },
                        ],
                        ..base_node(Kind::Linear, "main_col", Size::Match, Size::Match)
                    },
                    // 12. Material 3 Floating Action Button (bottom-end, floating cleanly above NavigationBar)
                    Node {
                        background: Some(P.primary_container),
                        color: Some(P.on_primary_container),
                        radius: 16.0,
                        icon: MaterialIcon::Add,
                        clickable: true,
                        layout_gravity: [2, 2],
                        layout_gravity_explicit: true,
                        margin: [0.0, 0.0, 16.0, 92.0],
                        relative_rules: FAB_RULES,
                        text: "Create",
                        ..base_node(Kind::Icon, "m3_fab", Size::Dp(52.0), Size::Dp(52.0))
                    },
                ],
                ..base_node(Kind::Frame, "showcase_root", Size::Match, Size::Match)
            }
        };
    };
}

define_showcase!(LAYOUT_LIGHT, LIGHT_PALETTE, false);
define_showcase!(LAYOUT_DARK, DARK_PALETTE, true);

// --- Page 2: Wave 8B + 9B Showcase (CollapsingToolbarLayout, DrawerLayout, SwipeRefreshLayout,
//     ViewPager, RangeSlider, ExtendedFloatingActionButton, BottomAppBar, NavigationBar + badges) ---

static P2_COLLAPSING_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_COLLAPSING_TOOLBAR,
    anchor: "Wave 8B & 9B|142|",
}];

static P2_DRAWER_LAYOUT_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_DRAWER_LAYOUT, anchor: "" },
    RelativeRule { kind: RULE_CLIP_TO_OUTLINE, anchor: "true" },
];

static P2_DRAWER_NAV_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_NAV_BAR, anchor: "drawer|labeled" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "dr_inbox|Inbox|inbox" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "dr_starred|Starred|star" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "dr_archive|Archive|archive" },
];

static P2_SRL_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_SWIPE_REFRESH, anchor: "" },
    RelativeRule { kind: RULE_CLIP_TO_OUTLINE, anchor: "true" },
];

static P2_PAGER_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_VIEW_PAGER, anchor: "horizontal" },
    RelativeRule { kind: RULE_CLIP_TO_OUTLINE, anchor: "true" },
];

static P2_RANGE_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_RANGE_SLIDER,
    anchor: "25,75",
}];

static P2_EXT_FAB_OPEN_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_EXTENDED_FAB,
    anchor: "1",
}];

static P2_EXT_FAB_COLLAPSED_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_EXTENDED_FAB,
    anchor: "0",
}];

static P2_CORNER_TOP_BTN_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_BUTTON_VARIANT, anchor: "tonal" },
    RelativeRule { kind: RULE_CORNER_RADII, anchor: "16,16,0,0" },
];

static P2_BAB_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_BOTTOM_APP_BAR, anchor: "end" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "bab_check|Done|check" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "bab_edit|Edit|edit" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "bab_search|Search|search" },
];

static P2_NAV_BAR_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_NAV_BAR, anchor: "bar|labeled" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "p2_home|Home|home" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "p2_mail|Mail|mail" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "p2_saved|Saved|star" },
    RelativeRule { kind: RULE_NAV_ITEM, anchor: "p2_settings|Settings|settings" },
    RelativeRule { kind: RULE_NAV_BADGE, anchor: "p2_home:" },
    RelativeRule { kind: RULE_NAV_BADGE, anchor: "p2_mail:99+" },
];

macro_rules! define_showcase_page2 {
    ($name:ident, $palette:expr) => {
        static $name: Node = {
            const P: ShowcasePalette = $palette;
            Node {
                kind: Kind::Linear,
                id: "p2_root",
                width: Size::Match,
                height: Size::Match,
                children: &[
                    // 1. Material 3 Large CollapsingToolbarLayout (28sp expanded headline + child Toolbar)
                    Node {
                        background: Some(P.surface_low),
                        relative_rules: P2_COLLAPSING_RULES,
                        children: &[Node {
                            kind: Kind::Toolbar,
                            image_src: "menu",
                            text: "Wave 8B & 9B",
                            ..base_node(Kind::Toolbar, "p2_tb", Size::Match, Size::Dp(64.0))
                        }],
                        ..base_node(Kind::Frame, "p2_collapsing", Size::Match, Size::Dp(142.0))
                    },
                    Node {
                        divider_color: Some(P.outline_variant),
                        divider_thickness: 1.0,
                        ..base_node(Kind::MaterialDivider, "p2_top_div", Size::Match, Size::Dp(1.0))
                    },
                    // Padded body showcasing DrawerLayout, SwipeRefreshLayout, ViewPager, RangeSlider, ExtendedFAB
                    Node {
                        kind: Kind::Linear,
                        id: "p2_body",
                        weight: 1.0,
                        width: Size::Match,
                        height: Size::Dp(0.0),
                        padding_edges: [16.0, 6.0, 16.0, 6.0],
                        padding_explicit: [true; 4],
                        children: &[
                            // 2. DrawerLayout (open modal NavigationView drawer + scrim)
                            section_title("p2_sec_drawer", "DrawerLayout (Open Modal NavigationView + Scrim)", P),
                            Node {
                                background: Some(P.surface_low),
                                background_stroke: Some(P.outline_variant),
                                background_stroke_width: 1.0,
                                radius: 14.0,
                                margin: [0.0, 2.0, 0.0, 4.0],
                                relative_rules: P2_DRAWER_LAYOUT_RULES,
                                children: &[
                                    // Underlying main content partially visible through dark scrim on right
                                    Node {
                                        padding_edges: [236.0, 18.0, 14.0, 14.0],
                                        padding_explicit: [true; 4],
                                        children: &[
                                            Node {
                                                text: "Inbox Feed",
                                                text_size: 15.0,
                                                text_bold: true,
                                                ..base_node(Kind::Text, "p2_dr_bg_h", Size::Match, Size::Wrap)
                                            },
                                            Node {
                                                text: "Tap scrim to dismiss modal drawer.",
                                                text_size: 12.0,
                                                color: Some(P.on_surface_variant),
                                                margin: [0.0, 6.0, 0.0, 10.0],
                                                ..base_node(Kind::Text, "p2_dr_bg_p", Size::Match, Size::Wrap)
                                            },
                                            Node {
                                                background: Some(P.primary_container),
                                                radius: 10.0,
                                                padding_edges: [10.0, 8.0, 10.0, 8.0],
                                                padding_explicit: [true; 4],
                                                children: &[Node {
                                                    text: "3 unread updates",
                                                    text_size: 11.0,
                                                    text_bold: true,
                                                    color: Some(P.on_primary_container),
                                                    ..base_node(Kind::Text, "p2_dr_bg_chip", Size::Match, Size::Wrap)
                                                }],
                                                ..base_node(Kind::Linear, "p2_dr_bg_card", Size::Match, Size::Wrap)
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "p2_dr_main", Size::Match, Size::Match)
                                    },
                                    // Modal NavigationView drawer panel (3 x 56dp rows + 24dp padding = 192dp)
                                    Node {
                                        width: Size::Dp(224.0),
                                        height: Size::Match,
                                        layout_gravity: [0, 0],
                                        layout_gravity_explicit: true,
                                        relative_rules: P2_DRAWER_NAV_RULES,
                                        ..base_node(Kind::Linear, "p2_nav_drawer", Size::Dp(224.0), Size::Match)
                                    },
                                ],
                                ..base_node(Kind::Frame, "p2_drawer", Size::Match, Size::Dp(194.0))
                            },
                            // 3. SwipeRefreshLayout (active elevated spinner) + ViewPager2 (dots)
                            section_title("p2_sec_mid", "SwipeRefreshLayout (Refreshing) & ViewPager2", P),
                            Node {
                                kind: Kind::Linear,
                                id: "p2_mid_row",
                                horizontal: true,
                                margin: [0.0, 2.0, 0.0, 4.0],
                                children: &[
                                    Node {
                                        weight: 1.0,
                                        margin: [0.0, 0.0, 10.0, 0.0],
                                        background: Some(P.surface_low),
                                        background_stroke: Some(P.outline_variant),
                                        background_stroke_width: 1.0,
                                        radius: 14.0,
                                        relative_rules: P2_SRL_RULES,
                                        children: &[Node {
                                            padding_edges: [12.0, 64.0, 12.0, 10.0],
                                            padding_explicit: [true; 4],
                                            children: &[
                                                Node {
                                                    text: "Refreshing feed...",
                                                    text_size: 13.0,
                                                    text_bold: true,
                                                    align: 1,
                                                    ..base_node(Kind::Text, "p2_srl_h", Size::Match, Size::Wrap)
                                                },
                                                Node {
                                                    text: "Elevated M3 progress spinner",
                                                    text_size: 11.0,
                                                    align: 1,
                                                    color: Some(P.on_surface_variant),
                                                    margin: [0.0, 2.0, 0.0, 0.0],
                                                    ..base_node(Kind::Text, "p2_srl_sub", Size::Match, Size::Wrap)
                                                },
                                            ],
                                            ..base_node(Kind::Linear, "p2_srl_col", Size::Match, Size::Match)
                                        }],
                                        ..base_node(Kind::Frame, "p2_srl", Size::Dp(0.0), Size::Match)
                                    },
                                    Node {
                                        weight: 1.0,
                                        background: Some(P.surface_low),
                                        background_stroke: Some(P.outline_variant),
                                        background_stroke_width: 1.0,
                                        radius: 14.0,
                                        relative_rules: P2_PAGER_RULES,
                                        children: &[
                                            Node {
                                                padding_edges: [12.0, 14.0, 12.0, 22.0],
                                                padding_explicit: [true; 4],
                                                children: &[Node {
                                                    text: "Page 1",
                                                    text_size: 14.0,
                                                    text_bold: true,
                                                    align: 1,
                                                    ..base_node(Kind::Text, "p2_vp_1", Size::Match, Size::Wrap)
                                                }],
                                                ..base_node(Kind::Linear, "p2_vp_c1", Size::Match, Size::Match)
                                            },
                                            Node {
                                                padding_edges: [12.0, 14.0, 12.0, 22.0],
                                                padding_explicit: [true; 4],
                                                children: &[
                                                    Node {
                                                        background: Some(P.secondary_container),
                                                        radius: 10.0,
                                                        padding_edges: [10.0, 8.0, 10.0, 8.0],
                                                        padding_explicit: [true; 4],
                                                        children: &[
                                                            Node {
                                                                text: "ViewPager2 · Page 2",
                                                                text_size: 13.0,
                                                                text_bold: true,
                                                                align: 1,
                                                                color: Some(P.on_secondary_container),
                                                                ..base_node(Kind::Text, "p2_vp_2h", Size::Match, Size::Wrap)
                                                            },
                                                            Node {
                                                                text: "Swipe left / right",
                                                                text_size: 11.0,
                                                                align: 1,
                                                                color: Some(P.on_secondary_container),
                                                                margin: [0.0, 2.0, 0.0, 0.0],
                                                                ..base_node(Kind::Text, "p2_vp_2s", Size::Match, Size::Wrap)
                                                            },
                                                        ],
                                                        ..base_node(Kind::Linear, "p2_vp_card", Size::Match, Size::Wrap)
                                                    },
                                                ],
                                                ..base_node(Kind::Linear, "p2_vp_c2", Size::Match, Size::Match)
                                            },
                                            Node {
                                                padding_edges: [12.0, 14.0, 12.0, 22.0],
                                                padding_explicit: [true; 4],
                                                children: &[Node {
                                                    text: "Page 3",
                                                    text_size: 14.0,
                                                    text_bold: true,
                                                    align: 1,
                                                    ..base_node(Kind::Text, "p2_vp_3", Size::Match, Size::Wrap)
                                                }],
                                                ..base_node(Kind::Linear, "p2_vp_c3", Size::Match, Size::Match)
                                            },
                                        ],
                                        ..base_node(Kind::ViewFlipper, "p2_pager", Size::Dp(0.0), Size::Match)
                                    },
                                ],
                                ..base_node(Kind::Linear, "p2_mid_row", Size::Match, Size::Dp(116.0))
                            },
                            // 4. Material 3 RangeSlider (two-thumb)
                            section_title("p2_sec_range", "RangeSlider (25 — 75)", P),
                            Node {
                                min: 0.0,
                                max: 100.0,
                                relative_rules: P2_RANGE_RULES,
                                ..base_node(Kind::SeekBar, "p2_range", Size::Match, Size::Dp(38.0))
                            },
                            // 5. ExtendedFloatingActionButton + Asymmetric ShapeAppearance
                            section_title("p2_sec_fab", "ExtendedFloatingActionButton & Corner.Top Shape", P),
                            Node {
                                kind: Kind::Linear,
                                id: "p2_fab_row",
                                horizontal: true,
                                clip_children_disabled: true,
                                clip_to_padding_disabled: true,
                                content_gravity: [0, 1],
                                padding_edges: [4.0, 4.0, 4.0, 12.0],
                                padding_explicit: [true; 4],
                                margin: [0.0, 4.0, 0.0, 0.0],
                                children: &[
                                    Node {
                                        text: "Compose",
                                        image_src: "edit",
                                        margin: [0.0, 0.0, 12.0, 0.0],
                                        relative_rules: P2_EXT_FAB_OPEN_RULES,
                                        ..base_node(Kind::Button, "p2_efab_ext", Size::Wrap, Size::Dp(56.0))
                                    },
                                    Node {
                                        text: "Add",
                                        image_src: "add",
                                        margin: [0.0, 0.0, 12.0, 0.0],
                                        relative_rules: P2_EXT_FAB_COLLAPSED_RULES,
                                        ..base_node(Kind::Button, "p2_efab_col", Size::Wrap, Size::Dp(56.0))
                                    },
                                    Node {
                                        weight: 1.0,
                                        text: "Corner.Top",
                                        padding_edges: [14.0, 6.0, 14.0, 6.0],
                                        padding_explicit: [true; 4],
                                        relative_rules: P2_CORNER_TOP_BTN_RULES,
                                        ..base_node(Kind::Button, "p2_asym_btn", Size::Dp(0.0), Size::Dp(48.0))
                                    },
                                ],
                                ..base_node(Kind::Linear, "p2_fab_row", Size::Match, Size::Dp(76.0))
                            },
                        ],
                        ..base_node(Kind::Linear, "p2_body", Size::Match, Size::Dp(0.0))
                    },
                    // 6. Material 3 BottomAppBar (with leading actions + end-aligned embedded FAB)
                    Node {
                        kind: Kind::Toolbar,
                        image_src: "menu",
                        relative_rules: P2_BAB_RULES,
                        children: &[Node {
                            background: Some(P.primary_container),
                            color: Some(P.on_primary_container),
                            radius: 16.0,
                            icon: MaterialIcon::Add,
                            clickable: true,
                            relative_rules: FAB_RULES,
                            text: "New",
                            ..base_node(Kind::Icon, "p2_bab_fab", Size::Dp(56.0), Size::Dp(56.0))
                        }],
                        ..base_node(Kind::Toolbar, "p2_bab", Size::Match, Size::Dp(80.0))
                    },
                    Node {
                        divider_color: Some(P.outline_variant),
                        divider_thickness: 1.0,
                        ..base_node(Kind::MaterialDivider, "p2_bot_div", Size::Match, Size::Dp(1.0))
                    },
                    // 7. Material 3 NavigationBar with 99+ red pill badge + red dot badge
                    Node {
                        relative_rules: P2_NAV_BAR_RULES,
                        ..base_node(Kind::Linear, "p2_bottom_nav", Size::Match, Size::Wrap)
                    },
                ],
                ..base_node(Kind::Linear, "p2_root", Size::Match, Size::Match)
            }
        };
    };
}

define_showcase_page2!(LAYOUT_PAGE2_LIGHT, LIGHT_PALETTE);
define_showcase_page2!(LAYOUT_PAGE2_DARK, DARK_PALETTE);

// --- Page 3: Wave 13B/14A/15A Material 3 Expressive Showcase ---
static P3_WAVY_LIN_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_WAVY_PROGRESS,
    anchor: "36|3|40|4|4",
}];
static P3_FLAT_GAP_LIN_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_WAVY_PROGRESS,
    anchor: "0|0|0|4|4",
}];
static P3_WAVY_CIRC_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_WAVY_PROGRESS,
    anchor: "15|2.2|40|0|4",
}];
static P3_LOADING_CONTAINED_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_LOADING_INDICATOR,
    anchor: "1",
}];
static P3_LOADING_PLAIN_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_LOADING_INDICATOR,
    anchor: "0",
}];
static P3_EXP_SLIDER_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_SLIDER_OPTIONS,
    anchor: "0|0|visible|1",
}];
static P3_EXP_RANGE_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_RANGE_SLIDER, anchor: "20,80" },
    RelativeRule { kind: RULE_SLIDER_OPTIONS, anchor: "10|20|visible|1" },
];
static P3_TIL_EXT_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_TEXT_INPUT_BOX, anchor: "outline|8||Budget|Monthly limit|0|clear_text" },
    RelativeRule { kind: RULE_TEXT_INPUT_EXT, anchor: "1|12|$|USD|search" },
];
static P3_TIL_EXT_CHILDREN: [Node; 1] = [Node {
    text: "2450",
    hint: "Budget",
    text_size: 14.0,
    ..base_node(Kind::Edit, "p3_budget_edit", Size::Match, Size::Match)
}];
static P3_CHIP_GROUP_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_CHIP_GROUP_WRAP,
    anchor: "0|1|6|6",
}];
static P3_SPLIT_BTN_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_SPLIT_BUTTON,
    anchor: "",
}];
static P3_FLOATING_TB_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_FLOATING_TOOLBAR,
    anchor: "",
}];
static P3_SNACKBAR_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_SNACKBAR,
    anchor: "",
}];
static P3_ALERT_DIALOG_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_ALERT_DIALOG,
    anchor: "delete",
}];
static P3_BOTTOM_SHEET_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_BOTTOM_SHEET,
    anchor: "92|1|1|0.5|0",
}];
static P3_DRAG_HANDLE_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_DRAG_HANDLE,
    anchor: "",
}];
static P3_SERIF_BI_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_TEXT_STYLE_EXT, anchor: "1|serif" },
    RelativeRule { kind: RULE_INCLUDE_FONT_PADDING, anchor: "false" },
];
static P3_MONO_RULES: &[RelativeRule] = &[
    RelativeRule { kind: RULE_TEXT_STYLE_EXT, anchor: "0|mono" },
    RelativeRule { kind: RULE_INCLUDE_FONT_PADDING, anchor: "false" },
];
static P3_ROT_BADGE_RULES: &[RelativeRule] = &[RelativeRule {
    kind: RULE_VIEW_TRANSFORM,
    anchor: "0,0,1,1,-10,center,center",
}];

macro_rules! define_showcase_page3 {
    ($name:ident, $palette:expr) => {
        static $name: Node = {
            const P: ShowcasePalette = $palette;
            Node {
                kind: Kind::Frame,
                id: "p3_root",
                width: Size::Match,
                height: Size::Match,
                children: &[
                    // Main vertical content above the docked BottomSheet
                    Node {
                        kind: Kind::Linear,
                        id: "p3_main_col",
                        padding_edges: [0.0, 0.0, 0.0, 96.0],
                        padding_explicit: [false, false, false, true],
                        children: &[
                            Node {
                                text: "M3 Expressive Showcase",
                                hint: "Wave 13B–15A · Bhoo Runtime",
                                ..base_node(Kind::Toolbar, "p3_top_bar", Size::Match, Size::Dp(48.0))
                            },
                            Node {
                                divider_color: Some(P.outline_variant),
                                divider_thickness: 1.0,
                                ..base_node(Kind::MaterialDivider, "p3_top_div", Size::Match, Size::Dp(1.0))
                            },
                            Node {
                                kind: Kind::Linear,
                                id: "p3_body",
                                weight: 1.0,
                                padding_edges: [14.0, 4.0, 14.0, 4.0],
                                padding_explicit: [true; 4],
                                children: &[
                                    // 1. Wavy Progress & Morphing LoadingIndicator
                                    section_title("p3_sec_prog", "Wavy Progress & Morphing LoadingIndicator", P),
                                    Node {
                                        horizontal: true,
                                        content_gravity: [0, 1],
                                        margin: [0.0, 2.0, 0.0, 2.0],
                                        children: &[
                                            Node {
                                                weight: 1.0,
                                                margin: [0.0, 0.0, 12.0, 0.0],
                                                children: &[
                                                    Node {
                                                        horizontal: true,
                                                        progress: 62.0,
                                                        relative_rules: P3_WAVY_LIN_RULES,
                                                        ..base_node(Kind::ProgressBar, "p3_wavy_lin", Size::Match, Size::Dp(20.0))
                                                    },
                                                    Node {
                                                        horizontal: true,
                                                        progress: 48.0,
                                                        margin: [0.0, 6.0, 0.0, 0.0],
                                                        relative_rules: P3_FLAT_GAP_LIN_RULES,
                                                        ..base_node(Kind::ProgressBar, "p3_flat_lin", Size::Match, Size::Dp(12.0))
                                                    },
                                                ],
                                                ..base_node(Kind::Linear, "p3_lin_col", Size::Dp(0.0), Size::Wrap)
                                            },
                                            Node {
                                                horizontal: false,
                                                progress: 70.0,
                                                margin: [0.0, 0.0, 8.0, 0.0],
                                                relative_rules: P3_WAVY_CIRC_RULES,
                                                ..base_node(Kind::ProgressBar, "p3_wavy_circ", Size::Dp(44.0), Size::Dp(44.0))
                                            },
                                            Node {
                                                horizontal: false,
                                                margin: [0.0, 0.0, 6.0, 0.0],
                                                relative_rules: P3_LOADING_CONTAINED_RULES,
                                                ..base_node(Kind::ProgressBar, "p3_load_c", Size::Dp(44.0), Size::Dp(44.0))
                                            },
                                            Node {
                                                horizontal: false,
                                                relative_rules: P3_LOADING_PLAIN_RULES,
                                                ..base_node(Kind::ProgressBar, "p3_load_u", Size::Dp(44.0), Size::Dp(44.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "p3_prog_row", Size::Match, Size::Dp(46.0))
                                    },
                                    // 2. M3 Expressive Slider & RangeSlider (with visible value bubbles!)
                                    section_title("p3_sec_sl", "Expressive Slider & RangeSlider (Visible Bubbles)", P),
                                    Node {
                                        horizontal: true,
                                        margin: [0.0, 40.0, 0.0, 2.0],
                                        children: &[
                                            Node {
                                                weight: 1.0,
                                                progress: 68.0,
                                                margin: [0.0, 0.0, 12.0, 0.0],
                                                relative_rules: P3_EXP_SLIDER_RULES,
                                                ..base_node(Kind::SeekBar, "p3_exp_sl", Size::Dp(0.0), Size::Dp(46.0))
                                            },
                                            Node {
                                                weight: 1.0,
                                                min: 0.0,
                                                max: 100.0,
                                                relative_rules: P3_EXP_RANGE_RULES,
                                                ..base_node(Kind::SeekBar, "p3_exp_rng", Size::Dp(0.0), Size::Dp(46.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "p3_sl_row", Size::Match, Size::Dp(46.0))
                                    },
                                    // 3. TextInputLayout with startIcon + prefix/suffix + counter
                                    section_title("p3_sec_til", "TextInputLayout (Start Icon · Prefix/Suffix · Counter)", P),
                                    Node {
                                        margin: [0.0, 2.0, 0.0, 2.0],
                                        relative_rules: P3_TIL_EXT_RULES,
                                        children: &P3_TIL_EXT_CHILDREN,
                                        ..base_node(Kind::Linear, "p3_til_ext", Size::Match, Size::Dp(74.0))
                                    },
                                    // 4. Multiline Wrapping ChipGroup
                                    section_title("p3_sec_chips", "ChipGroup (Multiline Wrap · Single Selection)", P),
                                    Node {
                                        horizontal: true,
                                        margin: [0.0, 2.0, 0.0, 2.0],
                                        relative_rules: P3_CHIP_GROUP_RULES,
                                        children: &[
                                            Node {
                                                text: "Expressive",
                                                checked: true,
                                                relative_rules: CHIP_FILTER_RULES,
                                                ..base_node(Kind::Button, "p3_ch1", Size::Wrap, Size::Dp(32.0))
                                            },
                                            Node {
                                                text: "Wavy",
                                                relative_rules: CHIP_FILTER_RULES,
                                                ..base_node(Kind::Button, "p3_ch2", Size::Wrap, Size::Dp(32.0))
                                            },
                                            Node {
                                                text: "Morphing",
                                                relative_rules: CHIP_FILTER_RULES,
                                                ..base_node(Kind::Button, "p3_ch3", Size::Wrap, Size::Dp(32.0))
                                            },
                                            Node {
                                                text: "FBO Alpha",
                                                relative_rules: CHIP_FILTER_RULES,
                                                ..base_node(Kind::Button, "p3_ch4", Size::Wrap, Size::Dp(32.0))
                                            },
                                            Node {
                                                text: "Transforms",
                                                relative_rules: CHIP_ASSIST_RULES,
                                                ..base_node(Kind::Button, "p3_ch5", Size::Wrap, Size::Dp(32.0))
                                            },
                                            Node {
                                                text: "Serif Italic",
                                                relative_rules: CHIP_ASSIST_RULES,
                                                ..base_node(Kind::Button, "p3_ch6", Size::Wrap, Size::Dp(32.0))
                                            },
                                            Node {
                                                text: "Monospace",
                                                relative_rules: CHIP_INPUT_RULES,
                                                ..base_node(Kind::Button, "p3_ch7", Size::Wrap, Size::Dp(32.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "p3_chip_group", Size::Match, Size::Wrap)
                                    },
                                    // 5. MaterialSplitButton & FloatingToolbar
                                    section_title("p3_sec_actions", "MaterialSplitButton & Expressive FloatingToolbar", P),
                                    Node {
                                        horizontal: true,
                                        content_gravity: [0, 1],
                                        clip_children_disabled: true,
                                        clip_to_padding_disabled: true,
                                        margin: [0.0, 2.0, 0.0, 2.0],
                                        children: &[
                                            Node {
                                                horizontal: true,
                                                margin: [0.0, 0.0, 12.0, 0.0],
                                                relative_rules: P3_SPLIT_BTN_RULES,
                                                children: &[
                                                    Node {
                                                        text: "Publish",
                                                        padding_edges: [16.0, 6.0, 14.0, 6.0],
                                                        padding_explicit: [true; 4],
                                                        relative_rules: BTN_FILLED_RULES,
                                                        ..base_node(Kind::Button, "p3_split_main", Size::Wrap, Size::Dp(42.0))
                                                    },
                                                    Node {
                                                        text: "",
                                                        padding_edges: [0.0; 4],
                                                        padding_explicit: [true; 4],
                                                        relative_rules: BTN_FILLED_RULES,
                                                        ..base_node(Kind::Button, "p3_split_more", Size::Dp(42.0), Size::Dp(42.0))
                                                    },
                                                ],
                                                ..base_node(Kind::Linear, "p3_split_btn", Size::Wrap, Size::Dp(42.0))
                                            },
                                            Node {
                                                weight: 1.0,
                                                horizontal: true,
                                                content_gravity: [1, 1],
                                                relative_rules: P3_FLOATING_TB_RULES,
                                                children: &[
                                                    Node {
                                                        icon: MaterialIcon::Edit,
                                                        color: Some(P.on_surface_variant),
                                                        margin: [0.0, 0.0, 4.0, 0.0],
                                                        ..base_node(Kind::Icon, "p3_ft_edit", Size::Dp(44.0), Size::Dp(44.0))
                                                    },
                                                    Node {
                                                        icon: MaterialIcon::Search,
                                                        color: Some(P.on_surface_variant),
                                                        margin: [0.0, 0.0, 4.0, 0.0],
                                                        ..base_node(Kind::Icon, "p3_ft_search", Size::Dp(44.0), Size::Dp(44.0))
                                                    },
                                                    Node {
                                                        icon: MaterialIcon::Star,
                                                        color: Some(P.primary),
                                                        margin: [0.0, 0.0, 4.0, 0.0],
                                                        ..base_node(Kind::Icon, "p3_ft_star", Size::Dp(44.0), Size::Dp(44.0))
                                                    },
                                                    Node {
                                                        icon: MaterialIcon::Settings,
                                                        color: Some(P.on_surface_variant),
                                                        ..base_node(Kind::Icon, "p3_ft_settings", Size::Dp(44.0), Size::Dp(44.0))
                                                    },
                                                ],
                                                ..base_node(Kind::Linear, "p3_float_tb", Size::Dp(0.0), Size::Dp(58.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "p3_action_row", Size::Match, Size::Dp(62.0))
                                    },
                                    // 6. Snackbar
                                    section_title("p3_sec_snack", "Snackbar (inverseSurface · inversePrimary)", P),
                                    Node {
                                        horizontal: true,
                                        margin: [0.0, 2.0, 0.0, 4.0],
                                        relative_rules: P3_SNACKBAR_RULES,
                                        children: &[
                                            Node {
                                                weight: 1.0,
                                                text: "3 layout layers synced",
                                                text_size: 13.0,
                                                ..base_node(Kind::Text, "p3_snack_msg", Size::Dp(0.0), Size::Wrap)
                                            },
                                            Node {
                                                text: "Undo",
                                                text_size: 13.0,
                                                text_bold: true,
                                                ..base_node(Kind::Button, "p3_snack_act", Size::Wrap, Size::Dp(34.0))
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "p3_snackbar", Size::Match, Size::Dp(48.0))
                                    },
                                    // 7. Side-by-side: MaterialAlertDialog + Wave 13B/14A Typography/Transform/FBO Card
                                    section_title("p3_sec_cards", "MaterialAlertDialog & Wave 13B/14A (Fonts · Transform · FBO α)", P),
                                    Node {
                                        horizontal: true,
                                        clip_children_disabled: true,
                                        clip_to_padding_disabled: true,
                                        margin: [0.0, 2.0, 0.0, 0.0],
                                        children: &[
                                            // MaterialAlertDialog with top icon
                                            Node {
                                                weight: 1.0,
                                                margin: [0.0, 0.0, 10.0, 0.0],
                                                padding_edges: [16.0, 14.0, 16.0, 10.0],
                                                padding_explicit: [true; 4],
                                                relative_rules: P3_ALERT_DIALOG_RULES,
                                                children: &[
                                                    Node {
                                                        text: "Discard draft?",
                                                        text_size: 15.0,
                                                        text_bold: true,
                                                        align: 1,
                                                        ..base_node(Kind::Text, "p3_dlg_title", Size::Match, Size::Wrap)
                                                    },
                                                    Node {
                                                        text: "Unsaved changes will be removed permanently.",
                                                        text_size: 11.5,
                                                        color: Some(P.on_surface_variant),
                                                        margin: [0.0, 4.0, 0.0, 6.0],
                                                        ..base_node(Kind::Text, "p3_dlg_body", Size::Match, Size::Wrap)
                                                    },
                                                    Node {
                                                        horizontal: true,
                                                        content_gravity: [2, 1],
                                                        children: &[
                                                            Node {
                                                                text: "Cancel",
                                                                text_size: 12.0,
                                                                padding_edges: [10.0, 4.0, 10.0, 4.0],
                                                                padding_explicit: [true; 4],
                                                                relative_rules: BTN_TEXT_RULES,
                                                                ..base_node(Kind::Button, "p3_dlg_cancel", Size::Wrap, Size::Dp(34.0))
                                                            },
                                                            Node {
                                                                text: "Delete",
                                                                text_size: 12.0,
                                                                padding_edges: [10.0, 4.0, 10.0, 4.0],
                                                                padding_explicit: [true; 4],
                                                                relative_rules: BTN_TEXT_RULES,
                                                                ..base_node(Kind::Button, "p3_dlg_ok", Size::Wrap, Size::Dp(34.0))
                                                            },
                                                        ],
                                                        ..base_node(Kind::Linear, "p3_dlg_btns", Size::Match, Size::Dp(36.0))
                                                    },
                                                ],
                                                ..base_node(Kind::Linear, "p3_dialog", Size::Dp(0.0), Size::Match)
                                            },
                                            // Wave 13B/14A Showcase Card
                                            Node {
                                                weight: 1.08,
                                                background: Some(P.surface_low),
                                                background_stroke: Some(P.outline_variant),
                                                background_stroke_width: 1.0,
                                                radius: 16.0,
                                                padding_edges: [12.0, 10.0, 12.0, 8.0],
                                                padding_explicit: [true; 4],
                                                clip_children_disabled: true,
                                                children: &[
                                                    Node {
                                                        text: "Serif Bold Italic",
                                                        text_size: 15.0,
                                                        text_bold: true,
                                                        relative_rules: P3_SERIF_BI_RULES,
                                                        ..base_node(Kind::Text, "p3_serif_bi", Size::Match, Size::Wrap)
                                                    },
                                                    Node {
                                                        text: "fn fbo_layer() -> u32",
                                                        text_size: 11.5,
                                                        color: Some(P.on_surface_variant),
                                                        margin: [0.0, 4.0, 0.0, 8.0],
                                                        relative_rules: P3_MONO_RULES,
                                                        ..base_node(Kind::Text, "p3_mono_txt", Size::Match, Size::Wrap)
                                                    },
                                                    Node {
                                                        horizontal: true,
                                                        content_gravity: [0, 1],
                                                        clip_children_disabled: true,
                                                        clip_to_padding_disabled: true,
                                                        children: &[
                                                            Node {
                                                                text: "ROT -10°",
                                                                text_size: 11.0,
                                                                text_bold: true,
                                                                align: 1,
                                                                content_gravity: [1, 1],
                                                                color: Some(P.on_primary_container),
                                                                background: Some(P.primary_container),
                                                                background_stroke: Some(P.primary),
                                                                background_stroke_width: 1.0,
                                                                radius: 10.0,
                                                                margin: [4.0, 4.0, 12.0, 4.0],
                                                                relative_rules: P3_ROT_BADGE_RULES,
                                                                ..base_node(Kind::Text, "p3_rot_badge", Size::Dp(76.0), Size::Dp(30.0))
                                                            },
                                                            // True Hardware FBO android:alpha="0.5" overlapping rects
                                                            Node {
                                                                view_alpha: 0.5,
                                                                children: &[
                                                                    Node {
                                                                        background: Some(Color(0xE5, 0x39, 0x35, 255)),
                                                                        radius: 8.0,
                                                                        ..base_node(Kind::View, "p3_fbo_r1", Size::Dp(54.0), Size::Dp(34.0))
                                                                    },
                                                                    Node {
                                                                        background: Some(Color(0x1E, 0x88, 0xE5, 255)),
                                                                        radius: 8.0,
                                                                        margin: [24.0, 14.0, 0.0, 0.0],
                                                                        ..base_node(Kind::View, "p3_fbo_r2", Size::Dp(54.0), Size::Dp(34.0))
                                                                    },
                                                                ],
                                                                ..base_node(Kind::Frame, "p3_fbo_group", Size::Dp(82.0), Size::Dp(50.0))
                                                            },
                                                        ],
                                                        ..base_node(Kind::Linear, "p3_fx_row", Size::Match, Size::Dp(54.0))
                                                    },
                                                    Node {
                                                        text: "FBO alpha=0.5 overlap · 2D rotate",
                                                        text_size: 10.0,
                                                        color: Some(P.on_surface_variant),
                                                        margin: [0.0, 4.0, 0.0, 0.0],
                                                        relative_rules: TIGHT_TEXT_RULES,
                                                        ..base_node(Kind::Text, "p3_fx_cap", Size::Match, Size::Wrap)
                                                    },
                                                ],
                                                ..base_node(Kind::Linear, "p3_fx_card", Size::Dp(0.0), Size::Match)
                                            },
                                        ],
                                        ..base_node(Kind::Linear, "p3_cards_row", Size::Match, Size::Dp(168.0))
                                    },
                                ],
                                ..base_node(Kind::Linear, "p3_body", Size::Match, Size::Dp(0.0))
                            },
                        ],
                        ..base_node(Kind::Linear, "p3_main_col", Size::Match, Size::Match)
                    },
                    // 8. BottomSheet with BottomSheetDragHandleView docked cleanly at bottom
                    Node {
                        kind: Kind::Linear,
                        id: "p3_bottom_sheet",
                        width: Size::Match,
                        height: Size::Wrap,
                        relative_rules: P3_BOTTOM_SHEET_RULES,
                        children: &[
                            Node {
                                relative_rules: P3_DRAG_HANDLE_RULES,
                                ..base_node(Kind::View, "p3_drag_handle", Size::Match, Size::Dp(26.0))
                            },
                            Node {
                                text: "Material 3 BottomSheet",
                                text_size: 14.0,
                                text_bold: true,
                                padding_edges: [20.0, 0.0, 20.0, 2.0],
                                padding_explicit: [true; 4],
                                ..base_node(Kind::Text, "p3_bs_title", Size::Match, Size::Wrap)
                            },
                            Node {
                                text: "Top 28dp corners · surfaceContainerLow · elevation 2 · drag handle",
                                text_size: 11.5,
                                color: Some(P.on_surface_variant),
                                padding_edges: [20.0, 0.0, 20.0, 10.0],
                                padding_explicit: [true; 4],
                                ..base_node(Kind::Text, "p3_bs_sub", Size::Match, Size::Wrap)
                            },
                        ],
                        ..base_node(Kind::Linear, "p3_bottom_sheet", Size::Match, Size::Wrap)
                    },
                ],
                ..base_node(Kind::Frame, "p3_root", Size::Match, Size::Match)
            }
        };
    };
}

define_showcase_page3!(LAYOUT_PAGE3_LIGHT, LIGHT_PALETTE);
define_showcase_page3!(LAYOUT_PAGE3_DARK, DARK_PALETTE);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let smoke = std::env::args().any(|x| x == "--smoke-test");
    let dark = std::env::args().any(|x| x == "--dark")
        || std::env::var("BHOO_THEME")
            .ok()
            .is_some_and(|v| v.trim().eq_ignore_ascii_case("dark"));
    let page: u8 = if std::env::args().any(|x| x == "--page3") {
        3
    } else if std::env::args().any(|x| x == "--page2") {
        2
    } else {
        match std::env::var("BHOO_SHOWCASE_PAGE").ok().as_deref().map(str::trim) {
            Some("3") => 3,
            Some("2") => 2,
            _ => 1,
        }
    };
    let mut state = LayoutState::default();
    state.theme_style = if dark {
        AndroidTheme::Material3Dark
    } else {
        AndroidTheme::Material3Light
    };
    state.density = 1.5;
    state.selected_index.insert("bottom_nav".into(), 1);
    state.selected_index.insert("p2_bottom_nav".into(), 1);
    state.selected_index.insert("p2_nav_drawer".into(), 0);
    state.selected_index.insert("p2_pager".into(), 1);
    state.drawer_open.insert("p2_drawer".into(), true);
    state.refreshing.insert("p2_srl".into(), true);
    let top_menu = vec![
        AndroidToolbarMenuItem {
            id: 1,
            group_id: 0,
            label: "Search".into(),
            icon: Some(MaterialIcon::Search),
            icon_drawable: None,
            show_text_with_icon: false,
            show_as_action: AndroidShowAsAction::Always,
            enabled: true,
            visible: true,
            checkable: false,
            checked: false,
            exclusive: false,
        },
        AndroidToolbarMenuItem {
            id: 2,
            group_id: 0,
            label: "Settings".into(),
            icon: None,
            icon_drawable: None,
            show_text_with_icon: false,
            show_as_action: AndroidShowAsAction::Never,
            enabled: true,
            visible: true,
            checkable: false,
            checked: false,
            exclusive: false,
        },
    ];
    state.toolbar_menu_items.insert("top_bar".into(), top_menu.clone());
    state.toolbar_menu_items.insert("p2_tb".into(), top_menu.clone());
    state.toolbar_menu_items.insert("p3_top_bar".into(), top_menu);
    let layout: &'static Node = match (page, dark) {
        (3, true) => &LAYOUT_PAGE3_DARK,
        (3, false) => &LAYOUT_PAGE3_LIGHT,
        (2, true) => &LAYOUT_PAGE2_DARK,
        (2, false) => &LAYOUT_PAGE2_LIGHT,
        (_, true) => &LAYOUT_DARK,
        (_, false) => &LAYOUT_LIGHT,
    };
    let mut frames = 0;
    window::run(
        WindowOptions {
            title: match (page, dark) {
                (3, true) => "Bhoo / Material 3 Expressive Showcase Page 3 (Dark)".into(),
                (3, false) => "Bhoo / Material 3 Expressive Showcase Page 3".into(),
                (2, true) => "Bhoo / Material 3 Showcase Page 2 (Dark)".into(),
                (2, false) => "Bhoo / Material 3 Showcase Page 2".into(),
                (_, true) => "Bhoo / Material 3 Showcase (Dark)".into(),
                (_, false) => "Bhoo / Material 3 Showcase".into(),
            },
            size: [720, 1280],
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
                    width: 720.0,
                    height: 1280.0,
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
