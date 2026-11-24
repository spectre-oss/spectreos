//! Runtime for the deliberately limited, build-time Android XML importer.
use crate::*;
mod relative_layout;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use unicode_linebreak::{linebreaks, BreakOpportunity};
#[derive(Clone, Copy)]
pub enum Kind {
    Linear,
    Frame,
    MediaController,
    VideoView,
    Relative,
    ConstraintLayout,
    Grid,
    Table,
    TableRow,
    Absolute,
    ViewAnimator,
    ViewSwitcher,
    TextSwitcher,
    ImageSwitcher,
    ViewFlipper,
    AdapterViewFlipper,
    StackView,
    Gallery,
    TabWidget,
    TabHost,
    DialerFilter,
    SlidingDrawer,
    Scroll,
    HorizontalScroll,
    Text,
    Chronometer,
    TextClock,
    AnalogClock,
    CheckedText,
    Button,
    CheckBox,
    RadioButton,
    RadioGroup,
    Switch,
    SeekBar,
    ProgressBar,
    RatingBar,
    ToggleButton,
    Icon,
    Zoom,
    ZoomControls,
    Image,
    QuickContactBadge,
    Edit,
    View,
    Space,
    MaterialDivider,
    MaterialCard,
    Spinner,
    ListView,
    GridView,
    ExpandableListView,
    SearchView,
    Toolbar,
    ActionMenuView,
    TimePicker,
    NumberPicker,
    DatePicker,
    CalendarView,
    AutoComplete,
    MultiAutoComplete,
}
#[derive(Clone, Copy)]
pub enum Size {
    Match,
    Wrap,
    Dp(f32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Invisible,
    Gone,
}
/// Visual theme preset for Android XML rendering.
/// `Bhoo` (default) preserves Bhoo's native dark theme and existing behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AndroidTheme {
    #[default]
    Bhoo,
    MaterialLight,
    MaterialDark,
    Material3Light,
    Material3Dark,
}
impl AndroidTheme {
    /// Active Android window background color for themes other than `Bhoo`.
    pub fn window_background(self) -> Option<Color> {
        match self {
            Self::Bhoo => None,
            Self::MaterialLight => Some(Color(0xFA, 0xFA, 0xFA, 255)),
            Self::MaterialDark => Some(Color(0x30, 0x30, 0x30, 255)),
            Self::Material3Light => Some(Color(0xFE, 0xF7, 0xFF, 255)),
            Self::Material3Dark => Some(Color(0x14, 0x12, 0x18, 255)),
        }
    }
    /// Default Toolbar surface color for themes other than `Bhoo`.
    fn toolbar_surface(self) -> Option<Color> {
        match self {
            Self::Bhoo => None,
            Self::MaterialLight => Some(Color(0xF5, 0xF5, 0xF5, 255)),
            Self::MaterialDark => Some(Color(0x21, 0x21, 0x21, 255)),
            Self::Material3Light => Some(Color(0xFE, 0xF7, 0xFF, 255)),
            Self::Material3Dark => Some(Color(0x14, 0x12, 0x18, 255)),
        }
    }
    /// Line-height multiplier relative to scaled font size.
    /// `Bhoo` keeps 1.5x; Android Material themes match AOSP Roboto `includeFontPadding=true` (~1.34x).
    fn text_line_height_factor(self) -> f32 {
        match self {
            Self::Bhoo => 1.5,
            Self::MaterialLight
            | Self::MaterialDark
            | Self::Material3Light
            | Self::Material3Dark => 1.34,
        }
    }
    /// Map an Android Material / Material 3 theme to a matching `bhoo::Theme`
    /// using pinned AOSP `themes_material.xml` and `material-android` tokens.
    fn bhoo_theme(self) -> Option<Theme> {
        match self {
            Self::Bhoo => None,
            Self::MaterialLight => Some(Theme {
                background: Color(0xFA, 0xFA, 0xFA, 255),
                text: Color(0x00, 0x00, 0x00, 0xDE),
                control: Color(0xD6, 0xD7, 0xD7, 255),
                hovered: Color(0xEA, 0xEA, 0xEA, 255),
                pressed: Color(0xC2, 0xC2, 0xC2, 255),
                radius: 2.0,
                primary: Color(0x00, 0x96, 0x88, 255),
                on_primary: Color(0xFF, 0xFF, 0xFF, 255),
                secondary: Color(0x00, 0x79, 0x6B, 255),
                outline: Color(0x00, 0x00, 0x00, 0x8A),
                error: Color(0xD3, 0x2F, 0x2F, 255),
                on_error: Color(0xFF, 0xFF, 0xFF, 255),
                on_secondary: Color(0xFF, 0xFF, 0xFF, 255),
                secondary_container: Color(0xB2, 0xDF, 0xDB, 255),
                on_secondary_container: Color(0x00, 0x4D, 0x40, 255),
                surface_low: Color(0xF5, 0xF5, 0xF5, 255),
                surface_high: Color(0xFF, 0xFF, 0xFF, 255),
                surface_highest: Color(0xE0, 0xE0, 0xE0, 255),
                outline_variant: Color(0x00, 0x00, 0x00, 0x1F),
                primary_container: Color(0xB2, 0xDF, 0xDB, 255),
                on_primary_container: Color(0x00, 0x4D, 0x40, 255),
                tertiary_container: Color(0xE0, 0xF2, 0xF1, 255),
                on_tertiary_container: Color(0x00, 0x4D, 0x40, 255),
                inverse_surface: Color(0x30, 0x30, 0x30, 255),
                inverse_on_surface: Color(0xFF, 0xFF, 0xFF, 255),
                inverse_primary: Color(0x80, 0xCB, 0xC4, 255),
                on_surface_variant: Color(0x00, 0x00, 0x00, 0x8A),
                surface_variant: Color(0xCC, 0xCC, 0xCC, 255),
            }),
            Self::MaterialDark => Some(Theme {
                background: Color(0x30, 0x30, 0x30, 255),
                text: Color(0xFF, 0xFF, 0xFF, 255),
                control: Color(0x5A, 0x59, 0x5B, 255),
                hovered: Color(0x6B, 0x6A, 0x6C, 255),
                pressed: Color(0x48, 0x47, 0x49, 255),
                radius: 2.0,
                primary: Color(0x80, 0xCB, 0xC4, 255),
                on_primary: Color(0x00, 0x00, 0x00, 255),
                secondary: Color(0x4D, 0xB6, 0xAC, 255),
                outline: Color(0xFF, 0xFF, 0xFF, 0xB3),
                error: Color(0xFF, 0x70, 0x43, 255),
                on_error: Color(0x00, 0x00, 0x00, 255),
                on_secondary: Color(0x00, 0x00, 0x00, 255),
                secondary_container: Color(0x00, 0x4D, 0x40, 255),
                on_secondary_container: Color(0xB2, 0xDF, 0xDB, 255),
                surface_low: Color(0x21, 0x21, 0x21, 255),
                surface_high: Color(0x42, 0x42, 0x42, 255),
                surface_highest: Color(0x55, 0x55, 0x55, 255),
                outline_variant: Color(0xFF, 0xFF, 0xFF, 0x1F),
                primary_container: Color(0x00, 0x4D, 0x40, 255),
                on_primary_container: Color(0xB2, 0xDF, 0xDB, 255),
                tertiary_container: Color(0x00, 0x4D, 0x40, 255),
                on_tertiary_container: Color(0xB2, 0xDF, 0xDB, 255),
                inverse_surface: Color(0xFA, 0xFA, 0xFA, 255),
                inverse_on_surface: Color(0x00, 0x00, 0x00, 0xDE),
                inverse_primary: Color(0x00, 0x96, 0x88, 255),
                on_surface_variant: Color(0xFF, 0xFF, 0xFF, 0xB3),
                surface_variant: Color(0x55, 0x55, 0x55, 255),
            }),
            Self::Material3Light => Some(Theme {
                background: Color(0xFE, 0xF7, 0xFF, 255),
                text: Color(0x1D, 0x1B, 0x20, 255),
                control: Color(0xF3, 0xED, 0xF7, 255),
                hovered: Color(0xEC, 0xE6, 0xF0, 255),
                pressed: Color(0xE6, 0xE0, 0xE9, 255),
                radius: 12.0,
                primary: Color(0x67, 0x50, 0xA4, 255),
                on_primary: Color(0xFF, 0xFF, 0xFF, 255),
                secondary: Color(0x62, 0x5B, 0x71, 255),
                outline: Color(0x79, 0x74, 0x7E, 255),
                error: Color(0xB3, 0x26, 0x1E, 255),
                on_error: Color(0xFF, 0xFF, 0xFF, 255),
                on_secondary: Color(0xFF, 0xFF, 0xFF, 255),
                secondary_container: Color(0xE8, 0xDE, 0xF8, 255),
                on_secondary_container: Color(0x1D, 0x19, 0x2B, 255),
                surface_low: Color(0xF7, 0xF2, 0xFA, 255),
                surface_high: Color(0xEC, 0xE6, 0xF0, 255),
                surface_highest: Color(0xE6, 0xE0, 0xE9, 255),
                outline_variant: Color(0xCA, 0xC4, 0xD0, 255),
                primary_container: Color(0xEA, 0xDD, 0xFF, 255),
                on_primary_container: Color(0x21, 0x00, 0x5D, 255),
                tertiary_container: Color(0xFF, 0xD8, 0xE4, 255),
                on_tertiary_container: Color(0x31, 0x11, 0x1D, 255),
                inverse_surface: Color(0x32, 0x2F, 0x35, 255),
                inverse_on_surface: Color(0xF5, 0xEF, 0xF7, 255),
                inverse_primary: Color(0xD0, 0xBC, 0xFF, 255),
                on_surface_variant: Color(0x49, 0x45, 0x4F, 255),
                surface_variant: Color(0xE7, 0xE0, 0xEC, 255),
            }),
            Self::Material3Dark => Some(Theme {
                background: Color(0x14, 0x12, 0x18, 255),
                text: Color(0xE6, 0xE0, 0xE9, 255),
                control: Color(0x21, 0x1F, 0x26, 255),
                hovered: Color(0x2B, 0x29, 0x30, 255),
                pressed: Color(0x36, 0x34, 0x3B, 255),
                radius: 12.0,
                primary: Color(0xD0, 0xBC, 0xFF, 255),
                on_primary: Color(0x38, 0x1E, 0x72, 255),
                secondary: Color(0xCC, 0xC2, 0xDC, 255),
                outline: Color(0x93, 0x8F, 0x99, 255),
                error: Color(0xF2, 0xB8, 0xB5, 255),
                on_error: Color(0x60, 0x14, 0x10, 255),
                on_secondary: Color(0x33, 0x2D, 0x41, 255),
                secondary_container: Color(0x4A, 0x44, 0x58, 255),
                on_secondary_container: Color(0xE8, 0xDE, 0xF8, 255),
                surface_low: Color(0x1D, 0x1B, 0x20, 255),
                surface_high: Color(0x2B, 0x29, 0x30, 255),
                surface_highest: Color(0x36, 0x34, 0x3B, 255),
                outline_variant: Color(0x49, 0x45, 0x4F, 255),
                primary_container: Color(0x4F, 0x37, 0x8B, 255),
                on_primary_container: Color(0xEA, 0xDD, 0xFF, 255),
                tertiary_container: Color(0x63, 0x3B, 0x48, 255),
                on_tertiary_container: Color(0xFF, 0xD8, 0xE4, 255),
                inverse_surface: Color(0xE6, 0xE0, 0xE9, 255),
                inverse_on_surface: Color(0x32, 0x2F, 0x35, 255),
                inverse_primary: Color(0x67, 0x50, 0xA4, 255),
                on_surface_variant: Color(0xCA, 0xC4, 0xD0, 255),
                surface_variant: Color(0x49, 0x45, 0x4F, 255),
            }),
        }
    }
}
/// Map an unscaled Android elevation in dp to a discrete Material elevation level.
fn elevation_level_from_dp(dp: f32) -> u8 {
    if dp <= 1.0 {
        1
    } else if dp <= 3.0 {
        2
    } else if dp <= 6.0 {
        3
    } else if dp <= 8.0 {
        4
    } else {
        5
    }
}
/// Paint a discrete Material elevation shadow with geometry scaled by `density`.
fn paint_elevation_level(f: &mut Frame<'_>, bounds: Rect, radius: f32, level: u8, density: f32) {
    paint_elevation_level_alpha(f, bounds, radius, level, density, 1.0);
}
/// Paint a discrete Material elevation shadow with alpha modulated by `alpha_scale`.
fn paint_elevation_level_alpha(
    f: &mut Frame<'_>,
    bounds: Rect,
    radius: f32,
    level: u8,
    density: f32,
    alpha_scale: f32,
) {
    let scale = alpha_scale.clamp(0.0, 1.0);
    if scale <= 0.001 {
        return;
    }
    let d = if density.is_finite() && density > 0.0 { density } else { 1.0 };
    let black = Color(0, 0, 0, 255);
    let specs = if level >= 6 {
        [
            (0.0, 3.0, 5.0, -1.0, 0.2),
            (0.0, 6.0, 10.0, 0.0, 0.14),
            (0.0, 1.0, 18.0, 0.0, 0.12),
        ]
    } else if level <= 1 {
        [
            (0.0, 2.0, 1.0, -1.0, 0.2),
            (0.0, 1.0, 1.0, 0.0, 0.14),
            (0.0, 1.0, 3.0, 0.0, 0.12),
        ]
    } else {
        [
            (0.0, 3.0, 1.0, -2.0, 0.2),
            (0.0, 2.0, 2.0, 0.0, 0.14),
            (0.0, 1.0, 5.0, 0.0, 0.12),
        ]
    };
    for (x, y, blur, spread, alpha) in specs {
        f.shadow(bounds, radius, [x * d, y * d], blur * d, spread * d, black.opacity(alpha * scale));
    }
}
/// Paint a continuous Android elevation (in unscaled dp) with shadow geometry scaled by `density`.
fn paint_elevation_dp(f: &mut Frame<'_>, bounds: Rect, radius: f32, dp: f32, density: f32) {
    if !dp.is_finite() || dp <= 0.0 {
        return;
    }
    let d = if density.is_finite() && density > 0.0 { density } else { 1.0 };
    const LOW: [[f32; 5]; 3] = [[0., 2., 1., -1., 0.2], [0., 1., 1., 0., 0.14], [0., 1., 3., 0., 0.12]];
    const MID: [[f32; 5]; 3] = [[0., 3., 1., -2., 0.2], [0., 2., 2., 0., 0.14], [0., 1., 5., 0., 0.12]];
    const HIGH: [[f32; 5]; 3] = [[0., 3., 5., -1., 0.2], [0., 6., 10., 0., 0.14], [0., 1., 18., 0., 0.12]];
    let (a, b_spec, t) = if dp <= 1.0 {
        (LOW, LOW, 0.0)
    } else if dp <= 2.0 {
        (LOW, MID, dp - 1.0)
    } else {
        (MID, HIGH, ((dp - 2.0) / 4.0).min(1.0))
    };
    let scale = (if dp > 6.0 { (dp / 6.0).sqrt().min(2.0) } else { 1.0 }) * d;
    for i in 0..3 {
        let p = std::array::from_fn::<_, 5, _>(|j| a[i][j] + (b_spec[i][j] - a[i][j]) * t);
        f.shadow(
            bounds,
            radius,
            [p[0] * scale, p[1] * scale],
            p[2] * scale,
            p[3] * scale,
            Color(0, 0, 0, 255).opacity(p[4] * dp.min(1.0)),
        );
    }
}
/// Subtle 2dp resting shadow for AOSP `Widget.Material.Button` (`btn_default_material`).
/// Scaled by `density` to match Android's ~2px top / ~3px bottom / <=4px side halo (~9-12% peak alpha)
/// without overflowing the 4dp horizontal button inset.
fn paint_material_button_shadow(f: &mut Frame<'_>, bounds: Rect, radius: f32, density: f32) {
    let d = if density.is_finite() && density > 0.0 { density } else { 1.0 };
    let black = Color(0, 0, 0, 255);
    f.shadow(bounds, radius, [0.0, 0.25 * d], 2.8 * d, 0.0, black.opacity(0.07));
    f.shadow(bounds, radius, [0.0, 1.0 * d], 2.0 * d, 0.0, black.opacity(0.07));
}
/// A ColorStateList entry: enabled, pressed, focused, selected, activated,
/// hovered and checked are encoded in bits 0 through 6 respectively.
#[derive(Clone, Copy)]
pub struct TintState { pub required:u8, pub forbidden:u8, pub color:Color }
/// One RelativeLayout rule; sibling anchors are Android XML IDs.
#[derive(Clone, Copy)]
pub struct RelativeRule { pub kind:u16, pub anchor:&'static str }
/// RelativeRule kind for `android:elevation` in dp (e.g. `"4"`).
pub const RULE_ELEVATION: u16 = 193;
/// RelativeRule kind for `android:backgroundTint` as `"r,g,b,a"` (e.g. `"255,0,0,255"`).
pub const RULE_BACKGROUND_TINT: u16 = 194;
/// RelativeRule kind for `android:textAlignment` (`"start"`, `"center"`, `"end"`, `"left"`, `"right"`).
pub const RULE_TEXT_ALIGNMENT: u16 = 195;
/// ConstraintLayout `app:layout_constraintStart_toStartOf` (`"parent"` or sibling XML ID).
pub const RULE_CONSTRAINT_START_TO_START: u16 = 196;
/// ConstraintLayout `app:layout_constraintStart_toEndOf` (`"parent"` or sibling XML ID).
pub const RULE_CONSTRAINT_START_TO_END: u16 = 197;
/// ConstraintLayout `app:layout_constraintEnd_toEndOf` (`"parent"` or sibling XML ID).
pub const RULE_CONSTRAINT_END_TO_END: u16 = 198;
/// ConstraintLayout `app:layout_constraintEnd_toStartOf` (`"parent"` or sibling XML ID).
pub const RULE_CONSTRAINT_END_TO_START: u16 = 199;
/// ConstraintLayout `app:layout_constraintTop_toTopOf` (`"parent"` or sibling XML ID).
pub const RULE_CONSTRAINT_TOP_TO_TOP: u16 = 200;
/// ConstraintLayout `app:layout_constraintTop_toBottomOf` (`"parent"` or sibling XML ID).
pub const RULE_CONSTRAINT_TOP_TO_BOTTOM: u16 = 201;
/// ConstraintLayout `app:layout_constraintBottom_toBottomOf` (`"parent"` or sibling XML ID).
pub const RULE_CONSTRAINT_BOTTOM_TO_BOTTOM: u16 = 202;
/// ConstraintLayout `app:layout_constraintBottom_toTopOf` (`"parent"` or sibling XML ID).
pub const RULE_CONSTRAINT_BOTTOM_TO_TOP: u16 = 203;
/// ConstraintLayout `app:layout_constraintHorizontal_bias` (float in `0.0..=1.0`, default `0.5`).
pub const RULE_CONSTRAINT_HORIZONTAL_BIAS: u16 = 204;
/// ConstraintLayout `app:layout_constraintVertical_bias` (float in `0.0..=1.0`, default `0.5`).
pub const RULE_CONSTRAINT_VERTICAL_BIAS: u16 = 205;
/// Per-corner radii `"tl,tr,br,bl"` in dp for asymmetric rounded rectangles.
pub const RULE_CORNER_RADII: u16 = 206;
/// Gradient background (`linear` comma/pipe, `radial|<cx>,<cy>,<radius_dp>|<start>|<center>|<end>`, or `sweep|<cx>,<cy>|<start>|<center>|<end>`).
pub const RULE_GRADIENT: u16 = 207;
/// Stacked layer-list / inset drawable layer (repeatable in draw order): `"l,t,r,b|radius|fill_r,g,b,a|stroke_w,sr,sg,sb,sa"`.
pub const RULE_DRAWABLE_LAYER: u16 = 208;
/// Custom ripple color and optional unbounded mask: `"r,g,b,a"` or `"r,g,b,a|unbounded"`.
pub const RULE_RIPPLE_COLOR: u16 = 209;
/// ConstraintLayout Guideline orientation (`"vertical"` or `"horizontal"`).
pub const RULE_GUIDE_ORIENTATION: u16 = 210;
/// ConstraintLayout Guideline begin offset in dp.
pub const RULE_GUIDE_BEGIN: u16 = 211;
/// ConstraintLayout Guideline end offset in dp.
pub const RULE_GUIDE_END: u16 = 212;
/// ConstraintLayout Guideline percent offset (`0.0..=1.0`).
pub const RULE_GUIDE_PERCENT: u16 = 213;
/// ConstraintLayout horizontal chain style (`"spread"`, `"spread_inside"`, `"packed"`).
pub const RULE_CHAIN_STYLE_H: u16 = 214;
/// ConstraintLayout vertical chain style (`"spread"`, `"spread_inside"`, `"packed"`).
pub const RULE_CHAIN_STYLE_V: u16 = 215;
/// ConstraintLayout horizontal chain weight (positive float).
pub const RULE_CONSTRAINT_H_WEIGHT: u16 = 216;
/// ConstraintLayout vertical chain weight (positive float).
pub const RULE_CONSTRAINT_V_WEIGHT: u16 = 217;
/// Explicit override for `android:textAllCaps` (`"true"` or `"false"`), allowing
/// buttons under Material themes to opt out of default uppercase styling.
pub const RULE_TEXT_ALL_CAPS: u16 = 218;
/// ConstraintLayout `app:layout_constraintDimensionRatio` (`"16:9"`, `"H,16:9"`, `"W,4:3"`, `"1.5"`).
pub const RULE_CONSTRAINT_DIMENSION_RATIO: u16 = 219;
/// ConstraintLayout `app:layout_constraintWidth_percent` (`0.0..=1.0`).
pub const RULE_CONSTRAINT_WIDTH_PERCENT: u16 = 220;
/// ConstraintLayout `app:layout_constraintHeight_percent` (`0.0..=1.0`).
pub const RULE_CONSTRAINT_HEIGHT_PERCENT: u16 = 221;
/// ConstraintLayout `app:layout_constraintBaseline_toBaselineOf` (sibling XML ID).
pub const RULE_CONSTRAINT_BASELINE_TO_BASELINE: u16 = 222;
/// ConstraintLayout gone margins `"l,t,r,b,s,e"` (6-value), `"l,t,r,b,s,e,baseline"` (7-value),
/// or `"s,t,e,b,baseline"` (5-value) in dp (`< 0.0` or empty means unset).
pub const RULE_CONSTRAINT_GONE_MARGINS: u16 = 223;
/// Alias for `RULE_CONSTRAINT_GONE_MARGINS` (`223`).
pub const RULE_CONSTRAINT_GONE_MARGIN: u16 = RULE_CONSTRAINT_GONE_MARGINS;
/// ConstraintLayout Barrier `"<dir>:<id1>,<id2>,..."` or `"<dir>|<id1>,<id2>,...|<margin_dp>|<allows_gone:0|1>"` (`start|end|left|right|top|bottom`).
pub const RULE_BARRIER: u16 = 224;
/// Alias for `RULE_BARRIER` (`224`).
pub const RULE_CONSTRAINT_BARRIER: u16 = RULE_BARRIER;
/// ConstraintLayout Group `"<id1>,<id2>,..."` (propagates `Gone`/`Invisible` to referenced siblings).
pub const RULE_GROUP_REFS: u16 = 225;
/// CoordinatorLayout scrolling view behavior (`app:layout_behavior` scrolling below top app bar).
pub const RULE_COORDINATOR_SCROLL_BEHAVIOR: u16 = 226;
/// CoordinatorLayout anchor placement `"<target_id>:<gravity_tokens>"` (e.g. `"appbar:bottom|end"`).
pub const RULE_COORDINATOR_ANCHOR: u16 = 227;
/// Material TextInputLayout box chrome `"<mode>|<corner_dp>|<stroke_rgba>|<hint>|<helper>|<error_enabled>|<end_icon>"`.
pub const RULE_TEXT_INPUT_BOX: u16 = 228;
/// MaterialButton / Button visual variant (`"filled"`, `"outlined"`, `"tonal"`, `"text"`, `"elevated"`).
pub const RULE_BUTTON_VARIANT: u16 = 229;
/// Material 3 Chip style `"<kind>|<close_visible>"` (e.g. `"filter|1"`).
pub const RULE_CHIP_STYLE: u16 = 230;
/// Stateful drawable layer (repeatable): `"<req_mask>,<forbid_mask>|<l,t,r,b>|<radius>|<fill_r,g,b,a>|<stroke_w,sr,sg,sb,sa>"`.
pub const RULE_STATEFUL_DRAWABLE_LAYER: u16 = 231;
/// ConstraintLayout Flow helper:
/// - legacy: `"<orient:0|1>|<wrap:none|chain|aligned>|<h_gap_dp>|<v_gap_dp>|<max_wrap>|<id1>,<id2>,..."`
/// - compact/extended: `"<orient:h|v>|<wrap:none|chain|aligned>|<hgap_dp>,<vgap_dp>|<id1>,<id2>,...[|<hstyle>,<vstyle>,<hbias>,<vbias>,<halign>,<valign>,<max_wrap>]"`.
pub const RULE_FLOW: u16 = 232;
/// ConstraintLayout circular positioning `"<target_id>,<radius_dp>,<angle_deg>"` (0 deg = 12 o'clock, clockwise).
pub const RULE_CONSTRAINT_CIRCLE: u16 = 233;
/// Clips child draw commands inside the view's rounded outline bounds (`"true"`).
pub const RULE_CLIP_TO_OUTLINE: u16 = 234;
/// Explicit text line height in dp/sp (positive float).
pub const RULE_LINE_HEIGHT: u16 = 235;
/// Android `includeFontPadding` override (`"true"` or `"false"`).
pub const RULE_INCLUDE_FONT_PADDING: u16 = 236;
/// Material 3 navigation container `"<kind:bar|rail|drawer>|<label_mode:auto|labeled|selected|unlabeled>"`.
pub const RULE_NAV_BAR: u16 = 237;
/// Material 3 navigation item (repeatable) `"<id>|<title>|<icon_name_or_empty>"`.
pub const RULE_NAV_ITEM: u16 = 238;
/// Material 3 SearchBar hint text `"<hint>"` on `Kind::SearchView`.
pub const RULE_SEARCH_BAR: u16 = 239;
/// MaterialButtonToggleGroup `"<single_selection:0|1>|<selection_required:0|1>|<checked_id>"`.
pub const RULE_TOGGLE_GROUP: u16 = 240;
/// Material 3 ExtendedFloatingActionButton `"<extended:0|1>"` on `Kind::Button`.
pub const RULE_EXTENDED_FAB: u16 = 242;
/// Material 3 BottomAppBar `"<fab_alignment:center|end>"` on `Kind::Toolbar`.
pub const RULE_BOTTOM_APP_BAR: u16 = 243;
/// Material 3 navigation destination badge (repeatable) `"<item_id>:<badge_text>"`.
pub const RULE_NAV_BADGE: u16 = 244;
/// Material 3 CollapsingToolbarLayout `"<title>|<expanded_h_dp>|<scrim_rgba>"` on `Kind::Frame`.
pub const RULE_COLLAPSING_TOOLBAR: u16 = 245;
/// AndroidX SwipeRefreshLayout container marker on `Kind::Frame`.
pub const RULE_SWIPE_REFRESH: u16 = 246;
/// AndroidX DrawerLayout container marker on `Kind::Frame`.
pub const RULE_DRAWER_LAYOUT: u16 = 247;
/// AndroidX ViewPager / ViewPager2 orientation `"<horizontal|vertical>"` on `Kind::ViewFlipper`.
pub const RULE_VIEW_PAGER: u16 = 248;
/// Material 3 RangeSlider initial two-thumb values `"<v0>,<v1>"` on `Kind::SeekBar`.
pub const RULE_RANGE_SLIDER: u16 = 250;
/// Static background bitmap/9-patch image `"<image_key>|<stretch_spec>"` (`stretch_spec` = `""` or `"sx0,sx1,...;sy0,sy1,..."`).
pub const RULE_BACKGROUND_IMAGE: u16 = 251;
/// Nine-patch stretch bands `"sx0,sx1,...;sy0,sy1,..."` (stripped image pixel coordinates, even count >= 2 per axis) on `ImageView`/`ImageButton`.
pub const RULE_IMAGE_NINE_PATCH: u16 = 252;
/// Stateful image `src` `<selector>` entry (repeatable in match order): `"<req_mask>,<forbid_mask>|<image_key>|<stretch_spec>"`.
pub const RULE_STATEFUL_IMAGE_SRC: u16 = 253;
/// Stateful background image `<selector>` entry (repeatable in match order): `"<req_mask>,<forbid_mask>|<image_key>|<stretch_spec>"`.
pub const RULE_STATEFUL_BG_IMAGE: u16 = 254;
/// RTL horizontal mirroring marker (`"true"`) from `android:autoMirrored="true"`.
pub const RULE_AUTO_MIRRORED: u16 = 255;
/// 2D view transform `"<tx_dp>,<ty_dp>,<sx>,<sy>,<rot_deg>,<px>,<py>"` (`px`/`py` `"center"`, `"<pct>%"`, or float dp).
pub const RULE_VIEW_TRANSFORM: u16 = 256;
/// Extended text styling `"<italic:0|1>|<family:sans|serif|mono>"`.
pub const RULE_TEXT_STYLE_EXT: u16 = 257;
/// Material tooltip text from `android:tooltipText`.
pub const RULE_TOOLTIP_TEXT: u16 = 258;
/// Material 3 BottomSheet behavior `"<peek_h_dp>|<hideable:0|1>|<fit_to_contents:0|1>|<half_ratio>|<skip_collapsed:0|1>"`.
pub const RULE_BOTTOM_SHEET: u16 = 259;
/// Material 3 BottomSheetDragHandleView pill marker (`""`).
pub const RULE_DRAG_HANDLE: u16 = 260;
/// Material 3 SideSheet behavior `"<edge:end|start>|<coplanar:0|1>"`.
pub const RULE_SIDE_SHEET: u16 = 261;
/// Material 3 Expressive LoadingIndicator `"<contained:0|1>"` on `Kind::ProgressBar`.
pub const RULE_LOADING_INDICATOR: u16 = 262;
/// Material 3 Expressive Wavy / flat-with-gap ProgressIndicator `"<wavelength_dp>|<amplitude_dp>|<wave_speed_dp>|<stop_size_dp>|<track_gap_dp>"` on `Kind::ProgressBar`.
pub const RULE_WAVY_PROGRESS: u16 = 263;
/// Material 3 AlertDialog container `"<icon_name_or_empty>"` on `Kind::Linear`.
pub const RULE_ALERT_DIALOG: u16 = 264;
/// Material 3 Snackbar container (`""`) on `Kind::Linear`.
pub const RULE_SNACKBAR: u16 = 265;
/// Material 3 TabLayout options `"<mode:fixed|scrollable>|<style:primary|secondary>|<indicator_rgba>"` on `Kind::TabWidget`.
pub const RULE_TAB_OPTIONS: u16 = 266;
/// Material 3 Slider / RangeSlider options `"<step_size>|<min_separation>|<label_behavior:floating|withinBounds|gone|visible>|<expressive:0|1>"` on `Kind::SeekBar`.
pub const RULE_SLIDER_OPTIONS: u16 = 267;
/// Material 3 TextInputLayout extensions `"<counter_enabled:0|1>|<counter_max>|<prefix>|<suffix>|<start_icon>"` on `Kind::Linear` with `RULE_TEXT_INPUT_BOX`.
pub const RULE_TEXT_INPUT_EXT: u16 = 268;
/// Material 3 ChipGroup wrap & selection `"<single_line:0|1>|<single_selection:0|1>|<h_spacing_dp>|<v_spacing_dp>"` on `Kind::Linear`.
pub const RULE_CHIP_GROUP_WRAP: u16 = 269;
/// Material 3 Expressive FloatingToolbar pill container (`""`) on `Kind::Linear`.
pub const RULE_FLOATING_TOOLBAR: u16 = 270;
/// Material 3 Expressive MaterialButtonGroup `"<spacing_dp>"` on `Kind::Linear`.
pub const RULE_BUTTON_GROUP: u16 = 271;
/// Material 3 Expressive MaterialSplitButton (`""`) on `Kind::Linear`.
pub const RULE_SPLIT_BUTTON: u16 = 272;
/// Material 3 Carousel MaskableFrameLayout item (`""`) on `Kind::Frame`.
pub const RULE_CAROUSEL_ITEM: u16 = 273;
/// Scale transition animation for incoming `ViewAnimator`/`ViewSwitcher`/`ViewFlipper`/`AdapterViewFlipper` child:
/// `"<from_sx>,<to_sx>,<from_sy>,<to_sy>|<px>,<py>|<duration_s>|<start_offset_s>|<linear:0|1>"`.
pub const RULE_SCALE_IN: u16 = 274;
/// Scale transition animation for outgoing `ViewAnimator`/`ViewSwitcher`/`ViewFlipper`/`AdapterViewFlipper` child.
pub const RULE_SCALE_OUT: u16 = 275;
/// Rotate transition animation for incoming `ViewAnimator`/`ViewSwitcher`/`ViewFlipper`/`AdapterViewFlipper` child:
/// `"<from_deg>,<to_deg>|<px>,<py>|<duration_s>|<start_offset_s>|<linear:0|1>"`.
pub const RULE_ROTATE_IN: u16 = 276;
/// Rotate transition animation for outgoing `ViewAnimator`/`ViewSwitcher`/`ViewFlipper`/`AdapterViewFlipper` child.
pub const RULE_ROTATE_OUT: u16 = 277;
/// Per-corner family `"<tl>,<tr_>,<br>,<bl>"` where each token is `"r"` (rounded) or `"c"` (cut/chamfered).
pub const RULE_CORNER_FAMILY: u16 = 278;
/// AppBarLayout dynamic `liftOnScroll`: `"1"` (auto-detect sibling scroll) or `"1|<target_xml_id>"`.
pub const RULE_APP_BAR_LIFT_ON_SCROLL: u16 = 279;
/// ConstraintLayout constrained wrap and min/max bounds `"<cw:0|1>,<ch:0|1>|<wmin_dp>,<wmax_dp>,<hmin_dp>,<hmax_dp>"`.
pub const RULE_CONSTRAINT_SIZE_BOUNDS: u16 = 280;
/// ConstraintLayout Layer helper `"<id1>,<id2>,..."` applying shared background, elevation, transform, and alpha.
pub const RULE_CONSTRAINT_LAYER: u16 = 281;
/// ConstraintLayout Placeholder helper `"<content_id>|<empty_vis:gone|invisible|visible>"`.
pub const RULE_CONSTRAINT_PLACEHOLDER: u16 = 282;
/// AppBarLayout child `app:layout_scrollFlags` comma-separated flags (`scroll`, `enterAlways`, `enterAlwaysCollapsed`, `exitUntilCollapsed`, `snap`).
pub const RULE_APP_BAR_SCROLL_FLAGS: u16 = 284;
/// Animated `<animation-list>` frame playback `"<oneshot:0|1>|<img_key_0>:<dur_ms_0>,<img_key_1>:<dur_ms_1>,..."`.
pub const RULE_ANIMATION_LIST: u16 = 285;
/// Dynamic wrapper drawables (`level-list`, `clip`, `rotate`, `scale`, `transition`) with optional `bg:`/`src:` prefix:
/// - `"level|<min0>:<max0>:<img_key0>,<min1>:<max1>:<img_key1>,..."`
/// - `"clip|<h|v>|<start|end|center|top|bottom>|<img_key>"`
/// - `"rotate|<from_deg>,<to_deg>|<px>,<py>|<img_key>"`
/// - `"scale|<sw_frac>,<sh_frac>|<center|start|end|top|bottom>|<img_key>"`
/// - `"transition|<dur_ms>|<img_key_0>,<img_key_1>"`
pub const RULE_DRAWABLE_DYNAMIC: u16 = 286;
/// ConstraintLayout physical vs. logical horizontal pinning bitmask `"<bitmask_u8>"`:
/// bit 0 (`1`) = left-edge (`196`/`197`) physical `Left`; bit 1 (`2`) = right-edge (`198`/`199`) physical `Right`;
/// bit 2 (`4`) = bias (`204`) and left/right margins physical when both anchors are physical.
pub const RULE_CONSTRAINT_PHYS_FLAGS: u16 = 287;
/// Incoming `elevation` / `translationZ` object animator transition on `ViewAnimator`/`ViewSwitcher`/
/// `TextSwitcher`/`ImageSwitcher`/`ViewFlipper`/`AdapterViewFlipper`:
/// `"<from_dp>,<to_dp>|<dur_ms>|<offset_ms>|<interp>"`.
pub const RULE_ELEVATION_IN: u16 = 288;
/// Outgoing `elevation` / `translationZ` object animator transition on `ViewAnimator`/`ViewSwitcher`/
/// `TextSwitcher`/`ImageSwitcher`/`ViewFlipper`/`AdapterViewFlipper`.
pub const RULE_ELEVATION_OUT: u16 = 289;
/// AppBarLayout extended options:
/// - on `AppBarLayout`: `"lift_color=<r>,<g>,<b>,<a>"` (custom `liftOnScrollColor`)
/// - on `AppBarLayout` direct scrolling child: `"effect=compress"` (`app:layout_scrollEffect="compress"`)
pub const RULE_APP_BAR_EXT: u16 = 290;
/// DrawerLayout lock mode (`"unlocked"`/`"0"`, `"locked_closed"`/`"1"`, `"locked_open"`/`"2"`, `"undefined"`/`"3"`).
pub const RULE_DRAWER_LOCK_MODE: u16 = 291;
/// ConstraintLayout Flow per-first/last-line chain style & bias overrides:
/// `"<first_hstyle>,<last_hstyle>,<first_hbias>,<last_hbias>,<first_vstyle>,<last_vstyle>,<first_vbias>,<last_vbias>"`.
pub const RULE_CONSTRAINT_FLOW_EXT: u16 = 292;
/// `<animated-selector>` transition frame playback (repeatable, one per `<transition>`):
/// `"<ctx:src|bg>:<from_mask>:<to_mask>:<rev:0|1>|<img_key_0>:<dur_ms_0>,..."`.
pub const RULE_ANIMATED_SELECTOR: u16 = 293;
/// ConstraintLayout `app:layout_constraintWidth_default` / `app:layout_constraintHeight_default`:
/// `"<w_def:spread|wrap|percent>,<h_def:spread|wrap|percent>"`.
pub const RULE_CONSTRAINT_DEFAULT_SIZE: u16 = 294;
/// Timed `<animation-list>` frame playback on compound drawables, checkable button drawables,
/// `SeekBar` tick marks, and `ExpandableListView` group indicators (repeatable):
/// `"<slot:start|top|end|bottom|left|right|button|tick|indicator>:<oneshot:0|1>|<img_key_0>:<dur_ms_0>,..."`.
pub const RULE_COMPOUND_ANIMATION_LIST: u16 = 295;
/// Extended `TextView`/`Button` text layout options:
/// `"<break:simple|high_quality|balanced>|<hyphen:none|normal|full>|<just:none|inter_word>|<auto_min_sp>,<auto_max_sp>,<auto_step_sp>"`.
pub const RULE_TEXT_LAYOUT_EXT: u16 = 296;
/// Material `BadgeDrawable` extended options (repeatable):
/// `"<item_index|-1|self>|<gravity:top_end|top_start|bottom_end|bottom_start>|<max_chars>|<hoff_dp>,<voff_dp>"`.
pub const RULE_MATERIAL_BADGE_EXT: u16 = 297;
/// CollapsingToolbarLayout extended options:
/// - on `CollapsingToolbarLayout`: `"<mode:scale|fade>|<max_lines>|<scrim_dur_ms>|<subtitle>"`
/// - on `CollapsingToolbarLayout` direct children: `"collapse:<none|pin|parallax>:<mult>"`.
pub const RULE_COLLAPSING_TOOLBAR_EXT: u16 = 298;
/// Material 3 ExtendedFloatingActionButton extended options:
/// `"<extended:0|1>|<collapsed_size_dp>|<strategy:auto|wrap_content|match_parent>"`.
pub const RULE_EXTENDED_FAB_EXT: u16 = 299;
/// Material BottomAppBar extended options:
/// `"<align:center|end>|<anchor:cradle|embed>|<cradle_margin_dp>,<cradle_rad_dp>,<cradle_voff_dp>|<hide_on_scroll:0|1>"`.
pub const RULE_BOTTOM_APP_BAR_EXT: u16 = 300;
/// RecyclerView layout manager extended options on `Kind::ListView` / `Kind::GridView`:
/// `"<mgr:linear|grid|staggered>|<orient:v|h>|<span_count>|<reverse:0|1>|<stack_end:0|1>"`.
pub const RULE_RECYCLER_LAYOUT_EXT: u16 = 301;
/// Material `SearchView` fullscreen/modal search overlay coupled to `SearchBar` on `Kind::Frame` / `Kind::Linear`:
/// `"<anchor_bar_id>|<hint>|<hide_nav:0|1>|<use_drawer_arrow:0|1>"`.
pub const RULE_SEARCH_VIEW_EXT: u16 = 302;
/// AndroidX `SwipeRefreshLayout` extended options (paired with `RULE_SWIPE_REFRESH` on `Kind::Frame`):
/// `"<bg_rgba>|<scheme_rgba_list>|<size:default|large>|<start_off_dp>,<end_off_dp>"`.
pub const RULE_SWIPE_REFRESH_EXT: u16 = 303;
/// Material 3 `CarouselLayoutManager` keyline focal sizing & snap scrolling on `Kind::ListView`/`Kind::GridView`/`Kind::HorizontalScroll`/`Kind::Linear`:
/// `"<strategy:multi_browse|hero|uncontained|full_screen>|<spacing_dp>|<align:start|center>"`.
pub const RULE_CAROUSEL_LAYOUT: u16 = 304;
/// AndroidX `ViewPager` / `ViewPager2` extended options (paired with `RULE_VIEW_PAGER` on `Kind::ViewFlipper`):
/// `"<orient:h|v>|<user_input:0|1>|<page_margin_dp>|<transformer:none|zoom_out|depth>"`.
pub const RULE_VIEW_PAGER_EXT: u16 = 305;
/// Extended `TextView` / `EditText` auto-link detection & styling:
/// `"<mask:all|web|email|phone|none>|<link_rgba>|<clickable:0|1>"`.
pub const RULE_TEXT_LINKS_SPANS: u16 = 306;
/// Sampled `<animated-vector>` frame sequence playback (repeatable, one per `src` / `bg`):
/// `"<ctx:src|bg>:<repeat:0|1>:<dur_ms>|<img_key_0>,<img_key_1>,..."`.
pub const RULE_ANIMATED_VECTOR: u16 = 309;
/// DrawerLayout lock mode: unlocked (normal open/close, edge swipe, scrim dismiss).
pub const DRAWER_LOCK_MODE_UNLOCKED: u8 = 0;
/// DrawerLayout lock mode: locked closed (forces panel closed, blocks edge swipe-to-open).
pub const DRAWER_LOCK_MODE_LOCKED_CLOSED: u8 = 1;
/// DrawerLayout lock mode: locked open (forces panel open, blocks drag-to-close and scrim dismissal).
pub const DRAWER_LOCK_MODE_LOCKED_OPEN: u8 = 2;
/// DrawerLayout lock mode: undefined (behaves as unlocked).
pub const DRAWER_LOCK_MODE_UNDEFINED: u8 = 3;
/// Material BottomSheet/SideSheet expanded state (`BottomSheetBehavior.STATE_EXPANDED`).
pub const SHEET_STATE_EXPANDED: u8 = 3;
/// Material BottomSheet collapsed/peeking state (`BottomSheetBehavior.STATE_COLLAPSED`).
pub const SHEET_STATE_COLLAPSED: u8 = 4;
/// Material BottomSheet/SideSheet hidden state (`BottomSheetBehavior.STATE_HIDDEN`).
pub const SHEET_STATE_HIDDEN: u8 = 5;
/// Material BottomSheet half-expanded state (`BottomSheetBehavior.STATE_HALF_EXPANDED`).
pub const SHEET_STATE_HALF_EXPANDED: u8 = 6;
#[derive(Clone)]
struct TilDropdownAnchor {
    key: String,
    til_key: String,
    id: u64,
    xml_id: &'static str,
    bounds: Rect,
    rtl: bool,
}
#[derive(Clone, Copy)]
pub struct AnimationDelta { pub value: f32, pub mode: u8 }
impl AnimationDelta { fn resolve(self, own:f32, parent:f32) -> f32 { self.value * match self.mode { 1 => own, 2 => parent, _ => 1.0 } } }
#[derive(Clone, Copy)]
pub struct TranslateAnimation { pub deltas: [AnimationDelta; 4], pub duration:f64, pub start_offset:f64, pub linear:bool, pub y_timing:Option<(f64,bool,f64)> }
#[derive(Clone)]
struct SpinnerAnchor { key:String, id:u64, bounds:Rect, xml_id:&'static str, dialog:bool, prompt:&'static str, rtl:bool, width:&'static str, offset_x:f32, offset_y:f32, padding:[f32;4] }
impl TranslateAnimation {
    fn sample(self, elapsed:f64, child:Rect, parent:Rect, object:bool) -> [f32;2] {
        let progress = |duration:f64,linear:bool,offset:f64| { let t=if elapsed<offset {0.0} else if duration<=0.0 {1.0} else {((elapsed-offset)/duration).clamp(0.0,1.0) as f32}; if linear {t} else {(1.0-(std::f32::consts::PI*t).cos())*0.5} };
        let tx=progress(self.duration,self.linear,self.start_offset);
        let (yd,yl,yo)=self.y_timing.unwrap_or((self.duration,self.linear,self.start_offset));
        let ty=progress(yd,yl,yo);
        let [a,b,c,d] = self.deltas;
        let mix = |x:f32,y:f32,t:f32| x+(y-x)*t;
        [if object && elapsed<self.start_offset {0.0} else {mix(a.resolve(child.width,parent.width), b.resolve(child.width,parent.width),tx)},if object && elapsed<yo {0.0} else {mix(c.resolve(child.height,parent.height), d.resolve(child.height,parent.height),ty)}]
    }
    fn total_duration(self)->f64 { (self.start_offset+self.duration).max(self.y_timing.map_or(0.0,|timing|timing.0+timing.2)) }
}
#[derive(Clone, Copy)]
pub struct AlphaAnimation { pub from: f32, pub to: f32, pub duration: f64, pub start_offset: f64, pub linear: bool }
impl AlphaAnimation {
    fn total_duration(self) -> f64 { self.start_offset + self.duration }
    fn sample(self, elapsed: f64, object:bool) -> f32 {
        if object && elapsed<self.start_offset {return 1.0;}
        let t = if elapsed < self.start_offset { 0.0 } else if self.duration <= 0.0 { 1.0 } else { ((elapsed-self.start_offset) / self.duration).clamp(0.0, 1.0) as f32 };
        let t = if self.linear { t } else { (1.0 - (std::f32::consts::PI * t).cos()) * 0.5 };
        self.from + (self.to - self.from) * t
    }
}
#[derive(Clone, Copy)]
pub struct ViewTransition { current: usize, previous: usize, started: f64, active: bool }
#[derive(Clone, Copy)]
pub struct FlipperClock { running: bool, next: f64, epoch:u64, pause_generation:u64, visible:bool }
#[derive(Clone, Copy)]
pub struct ChronometerState { pub base:f64, pub running:bool }
#[derive(Clone, Copy)]
pub struct WallClock { pub unix_seconds:f64, pub offset_seconds:i32, pub is_24_hour:bool }
#[derive(Clone, Copy, Debug)]
pub struct ZoomControlsFade { from:f32, target_visible:bool, start:f64 }
impl ZoomControlsFade {
    fn value(self, now:f64) -> f32 {
        let t=((now-self.start)/0.5).clamp(0.0,1.0) as f32;
        let eased=(1.0-(std::f32::consts::PI*t).cos())*0.5;
        self.from+(f32::from(self.target_visible)-self.from)*eased
    }
}
#[derive(Clone, Copy)]
pub struct Node {
    pub kind: Kind,
    pub id: &'static str,
    pub width: Size,
    pub height: Size,
    pub min_width: f32,
    pub min_height: f32,
    pub horizontal: bool,
    pub measure_all_children: bool,
    /// 0 inherits, 1 is left-to-right, and 2 is right-to-left.
    pub layout_direction: u8,
    pub fill_viewport: bool,
    /// Android ViewGroup clips children to its padding by default.
    pub clip_to_padding_disabled: bool,
    /// Android ViewGroup clips each child to its bounds by default.
    pub clip_children_disabled: bool,
    pub weight: f32,
    /// Zero uses the sum of the visible child weights.
    pub weight_sum: f32,
    pub divider_color: Option<Color>,
    pub divider_thickness: f32,
    pub divider_flags: u8,
    pub divider_padding: f32,
    /// Physical left, top, right and bottom padding and optional relative overrides.
    pub padding_edges: [f32; 4],
    pub padding_explicit: [bool; 4],
    pub background_padding: Option<[f32; 4]>,
    pub background_min_size: [f32; 2],
    pub padding_relative: [Option<f32>; 2],
    /// Physical left, top, right and bottom margins.
    pub margin: [f32; 4],
    /// Optional start and end margins override the physical side in the current direction.
    pub margin_relative: [Option<f32>; 2],
    /// FrameLayout gravity: horizontal start/center/end/left/right, vertical top/center/bottom.
    pub layout_gravity: [u8; 2],
    /// Physical coordinates for children of deprecated AbsoluteLayout.
    pub absolute_position: [f32; 2],
    pub relative_rules: &'static [RelativeRule],
    pub layout_gravity_explicit: bool,
    pub content_gravity: [u8; 2],
    pub text: &'static str,
    pub hint: &'static str,
    pub chrono_format: &'static str,
    pub count_down: bool,
    pub clock_12: &'static str,
    pub clock_24: &'static str,
    /// 0 text, 1 email, 2 telephone, 3 numeric, 4 password, 5 decimal.
    pub input_type: u8,
    pub text_on: &'static str,
    pub text_off: &'static str,
    pub text_size: f32,
    /// Android letter spacing in em units.
    pub letter_spacing_em: f32,
    pub line_spacing_extra: f32,
    pub line_spacing_multiplier: f32,
    pub text_bold: bool,
    pub text_all_caps: bool,
    /// Zero means no explicit line limit.
    pub max_lines: usize,
    pub min_lines: usize,
    pub ellipsize_end: bool,
    /// 1 truncates at the start, 2 in the middle.
    pub ellipsize_side: u8,
    pub single_line: bool,
    pub align: u8,
    pub color: Option<Color>,
    pub text_color_states: &'static [TintState],
    pub background: Option<Color>,
    /// Solid foreground painted over this view and its descendants.
    pub foreground: Option<Color>,
    /// Negative radius represents an oval foreground.
    pub foreground_radius: f32,
    pub foreground_size: [f32; 2],
    pub foreground_padding: Option<[f32; 4]>,
    /// Horizontal/vertical foreground gravity; 5 means fill.
    pub foreground_gravity: [u8; 2],
    pub foreground_excludes_padding: bool,
    pub background_stroke: Option<Color>,
    pub background_stroke_width: f32,
    pub foreground_stroke: Option<Color>,
    pub foreground_stroke_width: f32,
    pub radius: f32,
    pub clickable: bool,
    pub icon: MaterialIcon,
    pub image_src: &'static str,
    /// Image scale mode encoded by the Android XML importer.
    pub image_scale: u8,
    /// Preserve drawable aspect ratio when a view dimension can wrap.
    pub image_adjust_bounds: bool,
    pub image_max_width: Option<f32>,
    pub image_max_height: Option<f32>,
    pub image_tint: Option<Color>,
    pub image_tint_states: &'static [TintState],
    /// 0 SRC_ATOP (XML default), 1 SRC_IN, 2 SRC_OVER, 3 MULTIPLY,
    /// 4 SCREEN, 5 ADD.
    pub image_tint_mode: u8,
    /// Drawable opacity in the Android imageAlpha range.
    pub image_alpha: u8,
    /// Opacity applied to the view and its children.
    pub view_alpha: f32,
    pub image_builtin: bool,
    pub enabled: bool,
    pub checked: bool,
    pub checked_button: &'static str,
    pub min: f32,
    pub max: f32,
    pub progress: f32,
    pub secondary_progress: f32,
    pub indeterminate: bool,
    pub num_stars: usize,
    pub rating: f32,
    pub step_size: f32,
    pub indicator: bool,
    pub mirror_for_rtl: bool,
    pub animation_in: Option<AlphaAnimation>,
    pub animation_out: Option<AlphaAnimation>,
    pub translate_in: Option<TranslateAnimation>,
    pub translate_out: Option<TranslateAnimation>,
    pub auto_start: bool,
    pub flip_interval: f64,
    pub visibility: Visibility,
    pub children: &'static [Node],
}
/// A vector drawable tint resolved against the current theme when painted.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum AndroidDrawableTint {Fixed(Color),ControlNormal}
/// Contact information supplied by the embedding application.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum QuickContactTarget { Uri(String), Email(String), Phone(String) }
/// A contact lookup delegated to the host. Resolve using its token.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct QuickContactLookup { pub token:u64,pub target:QuickContactTarget,pub open_after_lookup:bool }
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum QuickContactAction { Show(String), Create(QuickContactTarget) }
/// Host player state shown by an XML MediaController.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct MediaPlayerSnapshot {pub position_ms:u64,pub duration_ms:u64,pub buffered_percent:u8,pub playing:bool}
/// Player capabilities and whether previous/next listeners are installed.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct MediaControllerCapabilities {pub can_pause:bool,pub can_seek_backward:bool,pub can_seek_forward:bool,pub has_previous:bool,pub has_next:bool}
impl Default for MediaControllerCapabilities {fn default()->Self{Self{can_pause:true,can_seek_backward:true,can_seek_forward:true,has_previous:false,has_next:false}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum MediaControllerAction {Play,Pause,SeekTo(u64),Previous,Next}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum VideoMeasureSpec {Exactly(u32),AtMost(u32),Unspecified}
/// Android VideoView.onMeasure, using integer pixel dimensions.
pub fn measure_video_view(video:(u32,u32),width:VideoMeasureSpec,height:VideoMeasureSpec)->(u32,u32) {
    use VideoMeasureSpec::*;
    let default=|natural,spec|match spec {Exactly(size)|AtMost(size)=>size,Unspecified=>natural};
    let (vw,vh)=video;let (mut w,mut h)=(default(vw,width),default(vh,height));
    if vw==0||vh==0{return (w,h);}
    let ratio=|size:u32,numerator:u32,denominator:u32|((size as u64*numerator as u64/denominator as u64).min(u32::MAX as u64)) as u32;
    match (width,height) {
        (Exactly(sw),Exactly(sh))=>{w=sw;h=sh;if (vw as u64)*(h as u64)<(w as u64)*(vh as u64){w=ratio(h,vw,vh);}else if (vw as u64)*(h as u64)>(w as u64)*(vh as u64){h=ratio(w,vh,vw);}},
        (Exactly(sw),other)=>{w=sw;h=ratio(w,vh,vw);if let AtMost(limit)=other{h=h.min(limit);}},
        (other,Exactly(sh))=>{h=sh;w=ratio(h,vw,vh);if let AtMost(limit)=other{w=w.min(limit);}},
        (ws,hs)=>{w=vw;h=vh;if let AtMost(limit)=hs && h>limit{h=limit;w=ratio(h,vw,vh);}if let AtMost(limit)=ws && w>limit{w=limit;h=ratio(w,vh,vw);}},
    }
    (w,h)
}
#[derive(Clone,Copy,Debug,Default)]
struct MediaControllerFloating {visible:bool,deadline:Option<f64>,dragging:bool}
/// Android menu item action placement policy.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum AndroidShowAsAction {Never,IfRoom,Always}
/// Host-provided Android Toolbar menu item.
#[derive(Clone,Debug)]
pub struct AndroidToolbarMenuItem {pub id:u64,pub group_id:u64,pub label:String,pub icon:Option<MaterialIcon>,pub icon_drawable:Option<String>,pub show_text_with_icon:bool,pub show_as_action:AndroidShowAsAction,pub enabled:bool,pub visible:bool,pub checkable:bool,pub checked:bool,pub exclusive:bool}
/// Allocate fixed-width actions using Android's group promotion and demotion rules.
/// Slot widths remain an approximation of ActionMenuPresenter's view measurement.
pub fn allocate_android_toolbar_actions(items:&[AndroidToolbarMenuItem],slots:usize)->(Vec<usize>,Vec<usize>) {
    let required=items.iter().filter(|x|x.visible&&x.show_as_action==AndroidShowAsAction::Always).count();
    let requested=items.iter().filter(|x|x.visible&&x.show_as_action==AndroidShowAsAction::IfRoom).count();
    let never=items.iter().any(|x|x.visible&&x.show_as_action==AndroidShowAsAction::Never);
    let reserve=usize::from(never||required+requested>slots);
    let mut remaining=slots as isize-reserve as isize-required as isize;
    let mut width=slots.saturating_sub(reserve) as isize;
    let mut groups=std::collections::HashSet::new();let mut selected=vec![false;items.len()];
    for (i,item) in items.iter().enumerate() {if !item.visible{continue;}match item.show_as_action {
        AndroidShowAsAction::Always=>{selected[i]=true;width-=1;if item.group_id!=0 {groups.insert(item.group_id);}},
        AndroidShowAsAction::IfRoom=>{
            let grouped=item.group_id!=0&&groups.contains(&item.group_id);
            if (remaining>0||grouped)&&width>0 {selected[i]=true;width-=1;remaining-=1;if item.group_id!=0 {groups.insert(item.group_id);}}
            else if grouped {groups.remove(&item.group_id);for j in 0..i {if items[j].group_id==item.group_id && selected[j] {selected[j]=false;remaining+=1;}}}
        },
        AndroidShowAsAction::Never=>{},
    }}
    let mut visible=Vec::new();let mut overflow=Vec::new();
    for (i,item) in items.iter().enumerate() {if item.visible {if selected[i] {visible.push(i)}else{overflow.push(i)}}}
    (visible,overflow)
}
// ActionMenuView text items need at least two cells; icon items need one.
fn allocate_action_menu_items(items:&[AndroidToolbarMenuItem],costs:&[usize],slots:usize)->(Vec<usize>,Vec<usize>) {
    if slots==0 {return (Vec::new(),items.iter().enumerate().filter_map(|(i,item)|item.visible.then_some(i)).collect());}
    let requested=items.iter().enumerate().filter(|(_,item)|item.visible&&item.show_as_action!=AndroidShowAsAction::Never).map(|(i,_)|costs[i]).sum::<usize>();
    let reserve=usize::from(requested>slots||items.iter().any(|item|item.visible&&item.show_as_action==AndroidShowAsAction::Never));
    let required=items.iter().enumerate().filter(|(_,item)|item.visible&&item.show_as_action==AndroidShowAsAction::Always).map(|(i,_)|costs[i]).sum::<usize>();
    let mut remaining=slots.saturating_sub(reserve) as isize-required as isize;let mut selected=vec![false;items.len()];let mut groups=std::collections::HashSet::new();
    for (i,item) in items.iter().enumerate().filter(|(_,item)|item.visible) {
        if item.show_as_action==AndroidShowAsAction::Always {selected[i]=true;if item.group_id!=0{groups.insert(item.group_id);}continue;}
        if item.show_as_action!=AndroidShowAsAction::IfRoom {continue;}
        let grouped=item.group_id!=0&&groups.contains(&item.group_id);
        if remaining>=costs[i] as isize {selected[i]=true;remaining-=costs[i] as isize;if item.group_id!=0{groups.insert(item.group_id);}}
        else if grouped {groups.remove(&item.group_id);for j in 0..i {if items[j].group_id==item.group_id && selected[j] {selected[j]=false;remaining+=costs[j] as isize;}}}
    }
    let mut visible=Vec::new();let mut overflow=Vec::new();for (i,item) in items.iter().enumerate() {if item.visible {if selected[i]{visible.push(i)}else{overflow.push(i)}}}(visible,overflow)
}
/// Static, noninteractive XML views used to display a Spinner adapter item.
#[derive(Clone,Copy)]
pub struct SpinnerItemTemplates {
    pub selected: &'static Node,
    pub dropdown: &'static Node,
    /// XML TextView ID receiving the item's text in both templates.
    pub text_id: &'static str,
}
/// Static noninteractive XML view used for each ListView adapter row.
#[derive(Clone,Copy)]
pub struct ListItemTemplate {
    pub row: &'static Node,
    pub text_id: &'static str,
}
/// Additional text and drawable bindings for one ListView or GridView item.
#[derive(Clone,Default)]
pub struct AdapterItemBinding {
    /// Optional XML layout override for this item; otherwise the adapter default is used.
    pub template:Option<ListItemTemplate>,
    pub text:HashMap<String,String>,
    pub images:HashMap<String,String>,
}
fn adapter_item_template(default:Option<ListItemTemplate>,binding:Option<&AdapterItemBinding>)->Option<ListItemTemplate> {
    binding.and_then(|binding|binding.template).or(default)
}
fn hash_item_binding<H:std::hash::Hasher>(hash:&mut H,binding:Option<&AdapterItemBinding>) {
    use std::hash::{Hash,Hasher};
    if let Some(binding)=binding {
        binding.template.map(|template|(template.row as *const Node as usize,template.text_id)).hash(hash);
        for map in [&binding.text,&binding.images] {
            let mut combined=0u64;
            for (key,value) in map {let mut h=std::collections::hash_map::DefaultHasher::new();key.hash(&mut h);value.hash(&mut h);combined^=h.finish();}
            map.len().hash(hash);combined.hash(hash);
        }
    } else {Option::<(usize,&str)>::None.hash(hash);0usize.hash(hash);0usize.hash(hash);}
}
fn with_adapter_binding<T>(state:&mut LayoutState,text_id:&str,label:&str,binding:Option<&AdapterItemBinding>,run:impl FnOnce(&mut LayoutState)->T)->T {
    state.text.clear();let mut saved=Vec::new();
    if let Some(binding)=binding {
        state.text.extend(binding.text.iter().map(|(key,value)|(key.clone(),value.clone())));
        for (key,value) in &binding.images {saved.push((key.clone(),state.image_sources.insert(key.clone(),value.clone())));}
    }
    state.text.insert(text_id.to_owned(),label.to_owned());
    let result=run(state);
    for (key,old) in saved {if let Some(value)=old {state.image_sources.insert(key,value);}else{state.image_sources.remove(&key);}}
    result
}
/// An XML view placed before or after ListView adapter rows.
#[derive(Clone,Copy)]
pub struct ListFixedView { pub view:&'static Node, pub selectable:bool, pub label:&'static str }
fn inspect_expandable_template(node:&Node,text_id:&str)->(bool,bool,bool) {
    let allowed=matches!(node.kind,Kind::Text|Kind::Image|Kind::Space|Kind::View|Kind::Linear|Kind::Frame|Kind::CheckBox|Kind::Switch|Kind::Button);
    let interactive=matches!(node.kind,Kind::CheckBox|Kind::Switch|Kind::Button);
    let safe=allowed && (!node.clickable||interactive) && (!interactive||!node.id.is_empty());
    node.children.iter().fold((safe,matches!(node.kind,Kind::Text)&&node.id==text_id,interactive),|(ok,found,has),child|{let (a,b,c)=inspect_expandable_template(child,text_id);(ok&&a,found||b,has||c)})
}
fn inspect_adapter_template(node:&Node,text_id:&str)->(bool,bool) {
    let safe=!node.clickable && matches!(node.kind,Kind::Text|Kind::Image|Kind::Space|Kind::View|Kind::Linear|Kind::Frame);
    node.children.iter().fold((safe,matches!(node.kind,Kind::Text) && node.id==text_id),|(ok,found),child| {let (child_ok,child_found)=inspect_adapter_template(child,text_id);(ok&&child_ok,found||child_found)})
}
// Estimated row heights are refined only when rows become visible. The Fenwick
// tree keeps scroll position lookup and individual height updates logarithmic.
#[derive(Clone)]
struct ListTextGeometry { signature:u64, heights:Vec<f32>, fingerprints:Vec<Option<u64>>, tree:Vec<f32>, divider:f32 }
impl ListTextGeometry {
    fn new(signature:u64,count:usize,divider:f32)->Self {
        let step=48.0+divider;let mut tree=vec![0.0;count+1];
        for i in 1..=count {tree[i]=step*(i & i.wrapping_neg()) as f32;}
        Self {signature,heights:vec![48.0;count],fingerprints:vec![None;count],tree,divider}
    }
    fn prefix(&self,mut i:usize)->f32 {let mut sum=0.0;while i>0 {sum+=self.tree[i];i&=i-1;}sum}
    fn index_at(&self,target:f32)->usize {
        let n=self.heights.len();if n==0{return 0;}
        let mut bit=1usize;while bit<=n/2 {bit*=2;}
        let (mut index,mut sum)=(0usize,0.0f32);
        while bit>0 {let next=index+bit;if next<=n && sum+self.tree[next]<=target {index=next;sum+=self.tree[next];}bit/=2;}
        index
    }
    fn measure(&mut self,index:usize,label:&str,width:f32,metrics:TextMetrics,template:Option<ListItemTemplate>,row_state:&mut Option<LayoutState>,id:u64,binding:Option<&AdapterItemBinding>)->bool {
        use std::hash::{Hash,Hasher};let mut hash=std::collections::hash_map::DefaultHasher::new();label.hash(&mut hash);hash_item_binding(&mut hash,binding);let fingerprint=hash.finish();
        if self.fingerprints[index]==Some(fingerprint) {return false;}
        self.fingerprints[index]=Some(fingerprint);
        let template=adapter_item_template(template,binding);
        let height=if let (Some(template),Some(row_state))=(template,row_state.as_mut()) {
            with_adapter_binding(row_state,template.text_id,label,binding,|state|match template.row.height {Size::Match=>48.0,_=>template.row.layout_intrinsic(false,state,width,metrics,anonymous_child_id(id,1024+index))})
        } else {let lines=wrapped_text_lines(label,(width-32.0).max(1.0),TextStyle::default(),metrics,|_|{}).max(1);24.0*lines as f32};
        let height=if height.is_finite() {height.clamp(if template.is_some(){1.0}else{48.0},4096.0)}else{48.0};
        let delta=height-self.heights[index];self.heights[index]=height;
        if delta.abs()<0.01 {return false;}
        let mut pos=index+1;while pos<self.tree.len(){self.tree[pos]+=delta;pos+=pos & pos.wrapping_neg();}true
    }
    fn measure_grid_row(&mut self,row:usize,entries:&[String],columns:usize,width:f32,metrics:TextMetrics,template:Option<ListItemTemplate>,row_state:&mut Option<LayoutState>,id:u64,bindings:Option<&HashMap<usize,AdapterItemBinding>>)->bool {
        use std::hash::{Hash,Hasher};let start=row*columns;let end=(start+columns).min(entries.len());
        let mut hash=std::collections::hash_map::DefaultHasher::new();entries[start..end].hash(&mut hash);for index in start..end {hash_item_binding(&mut hash,bindings.and_then(|items|items.get(&index)));}let fingerprint=hash.finish();
        if self.fingerprints[row]==Some(fingerprint) {return false;}self.fingerprints[row]=Some(fingerprint);
        let mut height=0.0_f32;
        for (offset,label) in entries[start..end].iter().enumerate() {
            let (_,measured)=grid_item_size(template,row_state,label,width,metrics,anonymous_child_id(id,1024+start+offset),bindings.and_then(|items|items.get(&(start+offset))));
            height=height.max(measured);
        }
        let delta=height-self.heights[row];self.heights[row]=height;if delta.abs()<0.01{return false;}
        let mut pos=row+1;while pos<self.tree.len(){self.tree[pos]+=delta;pos+=pos & pos.wrapping_neg();}true
    }
    fn measure_grid_window(&mut self,entries:&[String],columns:usize,scroll:f32,height:f32,width:f32,metrics:TextMetrics,template:Option<ListItemTemplate>,row_state:&mut Option<LayoutState>,id:u64,bindings:Option<&HashMap<usize,AdapterItemBinding>>)->bool {
        if scroll+height<=0.0{return false;}let mut row=self.index_at(scroll.max(0.0));let mut y=self.prefix(row);let mut changed=false;
        while row<self.heights.len() && y<scroll+height {changed|=self.measure_grid_row(row,entries,columns,width,metrics,template,row_state,id,bindings);y+=self.heights[row]+self.divider;row+=1;}changed
    }
    fn measure_window(&mut self,entries:&[String],scroll:f32,height:f32,width:f32,metrics:TextMetrics,template:Option<ListItemTemplate>,row_state:&mut Option<LayoutState>,id:u64,bindings:Option<&HashMap<usize,AdapterItemBinding>>)->bool {
        if scroll+height<=0.0 {return false;}
        let mut index=self.index_at(scroll.max(0.0));let mut y=self.prefix(index);let mut changed=false;
        while index<entries.len() && y<scroll+height {changed|=self.measure(index,&entries[index],width,metrics,template,row_state,id,bindings.and_then(|items|items.get(&index)));y+=self.heights[index]+self.divider;index+=1;}
        changed
    }
}
fn list_text_signature(entries:&[String],width:f32,divider:f32,metrics:TextMetrics,template:Option<ListItemTemplate>,row_state:Option<&LayoutState>)->u64 {
    use std::hash::{Hash,Hasher};let mut hash=std::collections::hash_map::DefaultHasher::new();
    (entries.as_ptr() as usize).hash(&mut hash);entries.len().hash(&mut hash);width.to_bits().hash(&mut hash);divider.to_bits().hash(&mut hash);
    for advance in metrics.normal.iter().chain(metrics.medium.iter()) {advance.to_bits().hash(&mut hash);}
    template.map(|t|(t.row as *const Node as usize,t.text_id)).hash(&mut hash);
    if let Some(state)=row_state {hash_adapter_environment(&mut hash,state);}
    hash.finish()
}
#[derive(Clone)]
struct AutoAnchor {key:String,id:u64,xml_id:&'static str,bounds:Rect,threshold:usize,width:&'static str,height:&'static str,offset_x:f32,offset_y:f32,rtl:bool,multi:bool,search:bool}
#[derive(Clone)]
struct AutoMatches {query:Option<String>,indices:Vec<usize>}
#[derive(Clone,Copy)]
struct ListHold { item_id:u64, origin:[f32;2], deadline:f64, fired:bool }
#[derive(Clone,Default)]
struct StackGesture {origin:Option<[f32;2]>,offset:f32,offset_x:f32,dragging:bool,tap_in_front:bool,direction:i8,drag_delta:f32,settle:Option<(f64,f32,f32,f64)>,outgoing:Option<(f64,f32,f32,f32,usize,f64)>,incoming:Option<(f64,f32,f32,f64)>,last_wheel:Option<f64>}
#[derive(Clone,Default)]
struct GalleryGesture { rtl:bool, origin:Option<[f32;2]>, offset:f32, drag_base:f32, dragging:bool, settle:Option<(f64,f32,f64)>, fling:Option<(f64,f32,f32,f64,usize)>, samples:Vec<(f64,f32)> }
#[derive(Clone, Default)]
struct ViewPagerGesture {
    origin: Option<[f32; 2]>,
    offset: f32,
    dragging: bool,
    settle: Option<(f64, f32, f64)>,
    last_page: usize,
    from_page: Option<usize>,
    last_time: f64,
    initialized: bool,
    samples: Vec<(f64, f32)>,
}
#[derive(Clone, Copy, Default)]
struct SwipeRefreshMotion {
    initialized: bool,
    last_refreshing: bool,
    last_time: f64,
    origin: Option<[f32; 2]>,
    dragging: bool,
    pull_px: f32,
    offset_px: f32,
    settle: Option<(f64, f32, f32, f64)>,
}
#[derive(Clone, Default)]
struct CarouselMotion {
    origin: Option<[f32; 2]>,
    drag_base: f32,
    dragging: bool,
    was_touch_dragging: bool,
    snap: Option<(f64, f32, f32, f64)>,
    samples: Vec<(f64, f32)>,
}
#[derive(Clone, Default)]
struct XmlSheetMotion {
    origin: Option<[f32; 2]>,
    drag_base: f32,
    drag_offset: f32,
    current_h: Option<f32>,
    dragging: bool,
    settle: Option<(f64, f32, f32, f64)>,
    last_state: u8,
    samples: Vec<(f64, f32)>,
}
#[derive(Clone, Default)]
struct XmlDrawerMotion {
    initialized: bool,
    last_open: bool,
    last_time: f64,
    progress: f32,
    anim: Option<(f64, f32, f32, f64)>,
    origin: Option<[f32; 2]>,
    drag_base_progress: f32,
    drag_align_right: bool,
    drag_panel_w: f32,
    dragging: bool,
    edge_opening: bool,
    samples: Vec<(f64, f32)>,
}
#[derive(Clone, Copy, Default)]
struct AppBarScrollState {
    initialized: bool,
    last_scroll: f32,
    collapse_px: f32,
    last_scroll_time: f64,
    last_time: f64,
    was_dragging: bool,
    snap_anim: Option<(f64, f32, f32, f64)>,
}
#[derive(Clone, Copy, Default)]
struct AnimatedSelectorState {
    initialized: bool,
    last_mask: u8,
    active: Option<(f64, bool, usize)>,
}
#[derive(Clone, Copy, Default)]
struct XmlLiftMotion {
    initialized: bool,
    last_lifted: bool,
    last_time: f64,
    progress: f32,
    anim: Option<(f64, f32, f32, f64)>,
}
#[derive(Clone, Copy, Default)]
struct DrawableTransitionMotion {
    initialized: bool,
    last_active: bool,
    progress: f32,
    anim: Option<(f64, f32, f32, f64)>,
}
#[derive(Clone, Copy, Default)]
struct BottomAppBarHideState {
    initialized: bool,
    last_scroll: f32,
    hidden: bool,
    last_time: f64,
    progress: f32,
    anim: Option<(f64, f32, f32, f64)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BottomAppBarCradleCacheKey {
    w: u16,
    h: u16,
    cx_q: u16,
    voff_q: i16,
    cradle_r_q: u16,
    fillet_r_q: u16,
    color: [u8; 4],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct GradientCacheKey {
    kind_tag: u8,
    w: u16,
    h: u16,
    p0: i32,
    p1: i32,
    p2: i32,
    start: [u8; 4],
    center: Option<([u8; 4], u16)>,
    end: [u8; 4],
    tint: Option<[u8; 4]>,
    mask_tag: u8,
    mask_radii: [u16; 4],
    mask_cut: [bool; 4],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct CutShapeCacheKey {
    w: u16,
    h: u16,
    radii_q: [u16; 4],
    cut: [bool; 4],
    color: [u8; 4],
}
fn pointer_velocity(samples: &[(f64, f32)], now: f64) -> f32 {
    samples
        .last()
        .and_then(|&(last_t, last_v)| {
            if now - last_t < 0.12 {
                samples
                    .iter()
                    .rev()
                    .find(|(t, _)| last_t - *t >= 0.005)
                    .map(|&(first_t, first_v)| (last_v - first_v) / (last_t - first_t) as f32)
            } else {
                None
            }
        })
        .unwrap_or(0.0)
}
// Android Scroller samples its fling spline at 100 intervals and linearly
// interpolates between those samples. The binary search solves its Bezier x.
fn gallery_spline_sample(index:usize)->f32 {
    if index==0{return 0.0;}if index>=100{return 1.0;}
    let target=index as f32/100.0;let(mut lo,mut hi)=(0.0,1.0);
    for _ in 0..20 {let x=(lo+hi)*0.5;let c=3.0*x*(1.0-x);let t=c*((1.0-x)*0.175+x*0.35)+x*x*x;if t<target{lo=x;}else{hi=x;}}
    let x=(lo+hi)*0.5;let c=3.0*x*(1.0-x);c*((1.0-x)*0.5+x)+x*x*x
}
fn gallery_spline_position(progress:f32)->f32 {
    let scaled=progress.clamp(0.0,1.0)*100.0;let index=scaled as usize;if index>=100{return 1.0;}
    let a=gallery_spline_sample(index);let b=gallery_spline_sample(index+1);a+(b-a)*(scaled-index as f32)
}
/// View-ID based tabs configured by a host for Android TabHost.
#[derive(Clone,Default)]
pub struct TabHostState {pub widget_id:String,pub content_ids:Vec<String>,pub labels:Vec<String>,pub tags:Vec<String>,pub selected:usize,pub factories:Vec<Option<fn(&str)-> &'static Node>>}
/// Content of a TabHost tab: an existing XML view or a lazy XML factory.
#[derive(Clone)]
pub enum TabHostContent { View(String), Factory(fn(&str)-> &'static Node) }
fn tab_host_find<'a>(node:&'a Node,id:&str)->Option<&'a Node> {
    if node.id==id {return Some(node);}node.children.iter().find_map(|child|tab_host_find(child,id))
}
fn radio_descendant_by_id(node:&Node,id:u64)->Option<&Node> {
    if !node.id.is_empty() && view_id(node.id)==id {return Some(node);}
    node.children.iter().find_map(|child|radio_descendant_by_id(child,id))
}
/// SlidingDrawer notifications corresponding to Android's listeners.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SlidingDrawerEvent { ScrollStarted, ScrollEnded, Opened, Closed }
#[derive(Clone,Default)]
pub struct SlidingDrawerState {pub open:bool,pub locked:bool,pending:Option<(bool,bool)>,position:f32,drag:Option<(f32,f32)>,dragging:bool,motion:Option<(f64,f32,f32,f64,f32,f32)>,samples:Vec<(f64,f32)>,events:Vec<SlidingDrawerEvent>}
impl SlidingDrawerState {
    fn record(&mut self,event:SlidingDrawerEvent) {
        if self.events.len()==32 {self.events.remove(0);}
        self.events.push(event);
    }
    fn complete(&mut self,open:bool) {
        if self.open!=open {
            self.open=open;
            self.record(if open{SlidingDrawerEvent::Opened}else{SlidingDrawerEvent::Closed});
        }
    }
}
#[derive(Clone)]
pub struct LayoutState {
    /// Active Android Material / Material 3 theme preset; defaults to `AndroidTheme::Bhoo`.
    pub theme_style: AndroidTheme,
    /// Runtime display density multiplier (`dp` -> logical px). Defaults to `1.0`.
    pub density: f32,
    /// Runtime font scale multiplier (`sp` = `dp * font_scale`). Defaults to `1.0`.
    pub font_scale: f32,
    pub dialer_filters: HashMap<String,crate::android_dialer_filter::DialerFilterState>,
    pub tab_widget_clicked: HashMap<String,usize>,
    pub tab_hosts: HashMap<String,TabHostState>,
    tab_host_factory_cache: HashMap<(String,usize),&'static Node>,
    active_tab_host: Option<String>,
    gallery_gestures: HashMap<String,GalleryGesture>,
    gallery_rendered_generation: HashMap<String,u64>,
    pub gallery_clicked: HashMap<String,usize>,
    /// State for Android SlidingDrawer views, keyed by XML ID.
    pub sliding_drawers: HashMap<String,SlidingDrawerState>,
    /// Most recently clicked adapter position in a StackView.
    pub stack_clicked: HashMap<String,usize>,
    /// Runtime loop policy for each StackView. XML defaults to non-looping.
    pub stack_looping: HashMap<String,bool>,
    stack_gestures: HashMap<String,StackGesture>,
    stack_pending_step: HashMap<String,(usize,isize)>,
    stack_rendered_generation: HashMap<String,u64>,
    /// Override the Android screen-configuration policy for labels beside action icons.
    pub menu_allow_text_with_icon: Option<bool>,
    /// Host theme's GridLayout default gap in logical pixels. None uses Android's 8dp default.
    pub grid_default_gap: Option<f32>,
    /// Decoded frames supplied by the host; VideoView does not decode media itself.
    pub video_frames: HashMap<String,Image>,
    /// Original source dimensions when different from the decoded frame size.
    pub video_dimensions: HashMap<String,(u32,u32)>,
    /// IDs of videos whose host player is prepared for interaction.
    pub video_ready: std::collections::HashSet<String>,
    /// Video ID to floating controller ID and height in logical pixels.
    pub video_controllers: HashMap<String,(String,f32)>,
    pub media_players: HashMap<String,MediaPlayerSnapshot>,
    pub media_controller_actions: HashMap<String,MediaControllerAction>,
    pub media_controller_capabilities: HashMap<String,MediaControllerCapabilities>,
    media_active_capabilities: Option<MediaControllerCapabilities>,
    media_seek_progress: HashMap<String,f32>,
    media_floating: HashMap<String,MediaControllerFloating>,
    /// Host-provided contact targets; no Android contacts provider is accessed.
    pub quick_contact_targets: HashMap<String,QuickContactTarget>,
    /// Last activated target for each QuickContactBadge.
    pub quick_contact_clicked: HashMap<String,QuickContactTarget>,
    quick_contact_pressed: HashMap<String,QuickContactTarget>,
    pub quick_contact_lookups: HashMap<String,QuickContactLookup>,
    pub quick_contact_actions: HashMap<String,QuickContactAction>,
    quick_contact_generation: u64,
    /// Layout direction for relative FrameLayout start/end gravity.
    pub rtl: bool,
    /// Per-table overrides for Android TableLayout.setColumnCollapsed.
    pub table_collapsed: HashMap<(u64,usize),bool>,
    pub text: HashMap<String, String>,
    pub input_types: HashMap<String, u8>,
    /// Calendar widget state keyed by Android view ID.
    pub calendars: HashMap<String,crate::android_calendar::CalendarState>,
    /// NumberPicker values and displayed labels supplied by the host.
    pub number_pickers: HashMap<String,crate::android_number_picker::NumberPickerState>,
    pub date_spinners: HashMap<String,crate::android_date_spinner::DateSpinnerState>,
    pub time_spinners: HashMap<String,crate::android_time_spinner::TimeSpinnerState>,
    /// Expanded state, submit-button policy and last submitted query per SearchView ID.
    /// Caller-supplied groups and most recent user event for each expandable list.
    pub expandable_lists: HashMap<String,crate::android_expandable_list::ExpandableListState>,
    pub expandable_events: HashMap<String,crate::android_expandable_list::ExpandableEvent>,
    /// Most recent Toolbar navigation action, keyed by XML view ID.
    pub toolbar_navigation_clicked: std::collections::HashSet<String>,
    pub toolbar_title_colors: HashMap<String,Color>,
    pub toolbar_subtitle_colors: HashMap<String,Color>,
    /// Toolbar menu entries supplied by the host, in display order.
    pub toolbar_menu_items: HashMap<String,Vec<AndroidToolbarMenuItem>>,
    /// The ID of the last activated menu item for each toolbar.
    pub toolbar_menu_clicked: HashMap<String,u64>,
    toolbar_menu_overlays: HashMap<String,OverlayState>,
    /// Group IDs whose clicks are handled by the host before expansion.
    pub expandable_intercepted_groups: HashMap<String,std::collections::HashSet<u64>>,
    /// Most recent group position clicked, including intercepted clicks.
    pub expandable_group_clicked: HashMap<String,usize>,
    expandable_templates: HashMap<String,(Option<ListItemTemplate>,Option<ListItemTemplate>)>,
    expandable_bindings: HashMap<String,HashMap<(u64,Option<u64>),AdapterItemBinding>>,
    expandable_control_states: HashMap<(String,u64,Option<u64>),HashMap<String,bool>>,
    adapter_namespace: Option<u64>,
    /// Last XML control activated within an expandable row.
    pub expandable_control_events: HashMap<String,(crate::android_expandable_list::FlatPosition,&'static str)>,
    pub search_open: HashMap<String,bool>,
    pub search_submit_enabled: std::collections::HashSet<String>,
    pub search_submitted: HashMap<String,String>,
    /// Last activated local suggestion index for each SearchView.
    pub search_suggestion_clicked: HashMap<String,usize>,
    /// Minimum query length before showing local suggestions; defaults to one.
    pub search_suggestion_threshold: HashMap<String,usize>,
    // Keep the original filter while keyboard selection previews suggestion text.
    search_suggestion_preview: HashMap<String,(String,String)>,
    search_pending: std::collections::HashSet<String>,
    search_focus_requested: std::collections::HashSet<String>,
    pub time_clocks: HashMap<String,crate::android_time_clock::TimeClockState>,
    time_clock_dragging: std::collections::HashSet<String>,
    time_clock_transition: HashMap<String,TimeClockTransition>,
    pub time_clock_text_mode: std::collections::HashSet<String>,
    time_clock_field_owner: HashMap<String,(String,crate::android_time_clock::DialPart)>,
    time_clock_invalid: std::collections::HashSet<String>,
    time_clock_changed: std::collections::HashSet<String>,
    number_picker_motion: HashMap<String,NumberPickerMotion>,
    /// Fields currently showing the host text editor.
    pub number_picker_editing: std::collections::HashSet<String>,
    number_picker_committed: std::collections::HashSet<String>,
    /// Localized Gregorian labels and default week start, supplied by the host.
    pub calendar_locale: crate::android_calendar::CalendarLocale,
    calendar_motion: HashMap<String,CalendarMotion>,
    datepicker_year_open: std::collections::HashSet<String>,
    datepicker_year_scroll: HashMap<String,f32>,
    datepicker_fades: HashMap<String,DatePickerFade>,
    autocomplete_items: HashMap<String,Vec<String>>,
    /// UTF-16 selection end supplied by the host text editor.
    pub text_selection_end: HashMap<String,usize>,
    /// Fields configured by the host with Android’s comma tokenizer.
    pub comma_tokenizers: std::collections::HashSet<String>,
    autocomplete_matches: HashMap<String,AutoMatches>,
    autocomplete_anchor: Option<AutoAnchor>,
    autocomplete_open: Option<String>,
    autocomplete_panel: Option<Rect>,
    autocomplete_dismissed: HashMap<String,String>,
    autocomplete_highlight: HashMap<String,usize>,
    autocomplete_scroll: HashMap<String,f32>,
    /// Most recently selected suggestion index, keyed by field ID.
    pub autocomplete_selected: HashMap<String,usize>,
    /// Decoded drawable images supplied by the application, keyed by resource name.
    pub images: HashMap<String, crate::Image>,
    /// Source pixel size in density-independent units per drawable.
    pub image_scales: HashMap<String, f32>,
    /// Axis-aligned matrix scale and translation (sx, sy, tx, ty), keyed by view ID.
    pub image_matrices: HashMap<String, [f32; 4]>,
    pub drawable_tints: HashMap<String,AndroidDrawableTint>,
    /// Runtime drawable-name overrides by ImageView ID.
    pub image_sources: HashMap<String, String>,
    /// Application-owned selected and activated state for Android views.
    pub view_selected: HashMap<String,bool>,
    pub view_activated: HashMap<String,bool>,
    /// Caller-owned MaterialCardView dragged states, keyed by XML ID.
    pub view_dragged: HashMap<String,bool>,
    #[doc(hidden)] pub tinted_images: HashMap<(u32,[u8;4],u8),Image>,
    #[doc(hidden)] pub tinted_image_bytes: usize,
    /// Per-view long-press repeat deadline and whether repetition started.
    pub zoom_holds: HashMap<u64,(f64,bool,bool)>,
    /// ZoomButton repeat intervals in milliseconds, keyed by XML view ID.
    /// The default is 1000 ms; zero is limited to 1 ms to avoid a busy loop.
    pub zoom_speed_ms: HashMap<String,u64>,
    /// Per-control enabled state for the zoom-out and zoom-in buttons.
    pub zoom_controls_enabled: HashMap<String,[bool;2]>,
    /// Most recent action for a rendered ZoomControls: -1 out or 1 in.
    pub zoom_controls_actions: HashMap<String,i8>,
    pub zoom_controls_fades: HashMap<String,ZoomControlsFade>,
    pub checked: HashMap<String, bool>,
    /// Adapter entries and selected index for Android Spinner views.
    pub spinner_items: HashMap<String,Vec<String>>,
    pub spinner_selected: HashMap<String,usize>,
    /// Optional stable IDs supplied alongside adapter entries.
    pub spinner_item_ids: HashMap<String,Vec<i64>>,
    /// Optional per-item enabled flags; missing positions are enabled.
    pub spinner_enabled: HashMap<String,Vec<bool>>,
    /// Host-supplied ListView adapter labels, checked indices, and last click.
    pub list_items: HashMap<String,Vec<String>>,
    /// Optional unique stable IDs corresponding to ListView adapter items.
    pub list_item_ids: HashMap<String,Vec<i64>>,
    pub list_checked: HashMap<String,Vec<usize>>,
    list_control_states: HashMap<(String,i64),HashMap<String,bool>>,
    /// Most recent ListView child-control activation, as adapter position and XML view ID.
    pub list_control_clicked: HashMap<String,(usize,&'static str)>,
    pub list_clicked: HashMap<String,usize>,
    list_item_bindings: HashMap<String,HashMap<usize,AdapterItemBinding>>,
    /// Enable and retrieve handled item long presses by ListView ID.
    pub list_long_click_enabled: HashMap<String,bool>,
    pub list_long_clicked: HashMap<String,usize>,
    list_holds: HashMap<String,Option<ListHold>>,
    /// Missing item enabled flags default to true.
    pub list_enabled: HashMap<String,Vec<bool>>,
    /// Most recent selectable fixed-view click: (is_footer, index within that group).
    pub list_fixed_clicked: HashMap<String,(bool,usize)>,
    /// Most recent control activation in a fixed view: footer flag, view index, XML ID.
    pub list_fixed_control_clicked: HashMap<String,(bool,usize,&'static str)>,
    list_fixed_control_states: HashMap<(String,bool,usize),HashMap<String,bool>>,
    list_fixed_views: HashMap<String,(Vec<ListFixedView>,Vec<ListFixedView>)>,
    adapter_root_response: Option<(u64,Response)>,
    list_templates: HashMap<String,ListItemTemplate>,
    list_text_geometry: HashMap<String,std::sync::Arc<ListTextGeometry>>,
    list_pending_focus: HashMap<String,usize>,
    spinner_templates: HashMap<String,SpinnerItemTemplates>,
    spinner_row_cache: HashMap<String,(u64,std::sync::Arc<Vec<f32>>)>,
    spinner_highlight: HashMap<String,usize>,
    spinner_scroll: HashMap<String,crate::scroll::ScrollPhysics>,
    pub spinner_open: Option<String>,
    spinner_anchor: Option<SpinnerAnchor>,
    spinner_dialogs: HashMap<String,OverlayState>,
    spinner_closing: Option<String>,
    pub radio_groups: HashMap<u64, Option<u64>>,
    radio_group_requests: HashMap<String, Option<String>>,
    radio_group_events: HashMap<String, Vec<Option<u64>>>,
    pub progress: HashMap<String, f32>,
    pub secondary_progress: HashMap<String, f32>,
    pub ratings: HashMap<String, f32>,
    pub offsets: HashMap<String, f32>,
    expandable_scrolls: HashMap<String, crate::android_expandable_list::ExpansionScroll>,
    expandable_pending_scroll: HashMap<String,u64>,
    expandable_geometry: HashMap<String,std::sync::Arc<ListTextGeometry>>,
    /// Displayed child index for ViewAnimator and ViewSwitcher, keyed by view ID.
    pub displayed_children: HashMap<String, usize>,
    pub transitions: HashMap<u64, ViewTransition>,
    pub flipper_running: HashMap<String, bool>,
    /// Most recent clicked adapter position for an AdapterViewFlipper.
    pub adapter_flipper_clicked: HashMap<String,usize>,
    /// Flippers advanced by their host rather than by their own timer.
    pub adapter_flipper_host_advanced: std::collections::HashSet<String>,
    adapter_flipper_data_generation: HashMap<String,u64>,
    adapter_flipper_rendered_generation: HashMap<String,u64>,
    adapter_flipper_animated: std::collections::HashSet<String>,
    pub flipper_clocks: HashMap<u64, FlipperClock>,
    pub chronometers: HashMap<String, ChronometerState>,
    /// Wall-clock snapshot supplied by the host, including local UTC offset.
    pub wall_clock: Option<WallClock>,
    /// Runtime overrides for views with Android IDs.
    pub visibility: HashMap<String, Visibility>,
    /// Per-field password visibility toggle state for TextInputLayout password_toggle end icon.
    text_input_password_visible: HashMap<String, bool>,
    /// Active TextInputLayout box context while rendering its EditText child.
    text_input_active_box: Option<TextInputChildBox>,
    /// Selected destination index for `RULE_NAV_BAR` containers (`bar`/`rail`/`drawer`), keyed by XML ID.
    pub selected_index: HashMap<String, usize>,
    /// Runtime badges for `RULE_NAV_BAR` destinations (`bar`/`rail`), keyed by nav item XML ID.
    /// Empty string `""` renders a 6dp circular dot badge; non-empty text renders a 16dp pill badge.
    pub nav_badges: HashMap<String, String>,
    /// Refreshing state for `RULE_SWIPE_REFRESH` (`SwipeRefreshLayout`) containers, keyed by XML ID.
    pub refreshing: HashMap<String, bool>,
    /// Open state for `RULE_DRAWER_LAYOUT` (`DrawerLayout`) containers, keyed by XML ID.
    pub drawer_open: HashMap<String, bool>,
    /// Lock mode for `RULE_DRAWER_LAYOUT` (`DrawerLayout`) drawers, keyed by `"{id}"` (start) and `"{id}:end"` (end).
    /// Values: `0 = UNLOCKED`, `1 = LOCKED_CLOSED`, `2 = LOCKED_OPEN`, `3 = UNDEFINED`.
    pub drawer_lock_mode: HashMap<String, u8>,
    /// Active DrawerLayout drawer panel context while rendering a drawer child.
    drawer_panel_active: bool,
    /// Active MaterialButtonToggleGroup segment context while rendering a child button.
    toggle_group_segment: Option<ToggleGroupSegment>,
    /// Toggle groups whose initial `checked_id` / child `checked` flags have been seeded into `state.checked`.
    toggle_groups_initialized: std::collections::HashSet<u64>,
    /// Two-thumb values `[start, end]` for `RULE_RANGE_SLIDER` (`RangeSlider`) views, keyed by XML ID.
    pub range_values: HashMap<String, [f32; 2]>,
    /// Active dragged thumb index (`0` or `1`) for `RULE_RANGE_SLIDER`, keyed by view ID.
    range_slider_active_thumb: HashMap<u64, usize>,
    /// Interactive swipe/settle gesture state for `RULE_VIEW_PAGER` views, keyed by XML ID.
    view_pager_gestures: HashMap<String, ViewPagerGesture>,
    /// Scroll offset from sibling scrolling view propagated to `RULE_COLLAPSING_TOOLBAR`.
    collapsing_scroll_offset: Option<f32>,
    /// Active CollapsingToolbarLayout context while rendering a child Toolbar.
    collapsing_toolbar_child: bool,
    /// Sheet state for `RULE_BOTTOM_SHEET` / `RULE_SIDE_SHEET` views, keyed by XML ID
    /// (`SHEET_STATE_EXPANDED=3`, `SHEET_STATE_COLLAPSED=4`, `SHEET_STATE_HIDDEN=5`, `SHEET_STATE_HALF_EXPANDED=6`).
    pub sheet_state: HashMap<String, u8>,
    /// Exposed dropdown menu entries for `TextInputLayout` (`end_icon == "dropdown_menu"`), keyed by child Edit ID.
    pub entries: HashMap<String, Vec<String>>,
    /// Per-node morph state for `RULE_LOADING_INDICATOR` (`LoadingIndicator`) views.
    loading_states: HashMap<u64, crate::loading::LoadingState>,
    /// Currently open `TextInputLayout` exposed dropdown key.
    til_dropdown_open: Option<String>,
    /// Anchor metadata for open `TextInputLayout` exposed dropdown popup.
    til_dropdown_anchor: Option<TilDropdownAnchor>,
    /// Rendered popup bounds for open `TextInputLayout` exposed dropdown.
    til_dropdown_panel: Option<Rect>,
    /// Horizontal scroll offsets for scrollable `RULE_TAB_OPTIONS` tab strips, keyed by XML ID.
    tab_scroll_offsets: HashMap<String, f32>,
    /// Active `RULE_SNACKBAR` context while rendering Snackbar children.
    snackbar_active: bool,
    /// Active `RULE_SPLIT_BUTTON` segment role while rendering split button child buttons.
    split_button_role: Option<SplitButtonRole>,
    /// `RULE_CHIP_GROUP_WRAP` groups whose initial single-selection exclusion has been seeded.
    chip_groups_initialized: std::collections::HashSet<u64>,
    /// Two-way synchronization links between `TabWidget` (`TabLayout`) and `RULE_VIEW_PAGER` (`ViewPager2`) XML IDs.
    pub tab_pager_links: HashMap<String, String>,
    /// Interactive drag and settle motion state for `RULE_BOTTOM_SHEET` and modal `RULE_SIDE_SHEET` views.
    sheet_motion: HashMap<String, XmlSheetMotion>,
    /// Animated slide-in/out and drag-to-close motion state for `RULE_DRAWER_LAYOUT` containers.
    drawer_motion: HashMap<String, XmlDrawerMotion>,
    /// Active `RULE_BOTTOM_SHEET` key while rendering its children (for `RULE_DRAG_HANDLE` click toggle).
    active_bottom_sheet: Option<String>,
    /// Set when a `RULE_DRAG_HANDLE` child inside an active bottom sheet is clicked without dragging.
    drag_handle_toggled: bool,
    /// Last observed tab selection per `TabWidget` key to synchronize `selected_index` and `displayed_children`.
    tab_last_index: HashMap<String, usize>,
    /// Runtime alpha multiplier overrides (`0.0..=1.0`, matching Android `View.setAlpha`), keyed by XML ID.
    pub view_alpha: HashMap<String, f32>,
    /// Runtime scale overrides `[sx, sy]` (matching Android `View.setScaleX`/`setScaleY`), keyed by XML ID.
    pub view_scale: HashMap<String, [f32; 2]>,
    /// Runtime translation overrides `[tx_dp, ty_dp]` (matching Android `View.setTranslationX`/`setTranslationY`), keyed by XML ID.
    pub view_translation: HashMap<String, [f32; 2]>,
    /// Bounded cache of rasterized radial/sweep (and cut-corner linear) gradient images.
    gradient_images: HashMap<GradientCacheKey, Image>,
    gradient_image_bytes: usize,
    /// Bounded cache of rasterized chamfered cut-corner shape fills.
    cut_shape_images: HashMap<CutShapeCacheKey, Image>,
    cut_shape_image_bytes: usize,
    /// Smooth liftOnScroll transition progress per AppBarLayout / Toolbar XML ID.
    app_bar_lift_motion: HashMap<String, XmlLiftMotion>,
    /// Active lifted AppBarLayout context while rendering its child Toolbar.
    lifted_appbar_active: bool,
    /// Dynamic target sibling XML ID override for `RULE_CONSTRAINT_PLACEHOLDER` views, keyed by placeholder XML ID.
    pub placeholder_content: HashMap<String, String>,
    /// Scroll-delta collapse tracking for `RULE_APP_BAR_SCROLL_FLAGS` (`enterAlways`) children, keyed by view key.
    app_bar_scroll_motion: HashMap<String, AppBarScrollState>,
    /// Runtime drawable level (`0..=10000`, Android `Drawable.setLevel`) for `RULE_DRAWABLE_DYNAMIC`, keyed by XML ID.
    pub drawable_level: HashMap<String, u32>,
    /// Runtime transition state (`false` = layer 0, `true` = layer 1) for `RULE_DRAWABLE_DYNAMIC` `"transition|..."`, keyed by XML ID.
    pub drawable_transition_active: HashMap<String, bool>,
    /// Internal crossfade animation state for `RULE_DRAWABLE_DYNAMIC` `"transition|..."` drawables.
    drawable_transition_motion: HashMap<String, DrawableTransitionMotion>,
    /// Suppresses direct-child static `RULE_ELEVATION` when parent `ViewAnimator`/`AdapterViewFlipper` renders an elevation transition.
    suppress_next_elevation: bool,
    /// Bounded internal state tracking previous 7-bit state masks and active `<animated-selector>` transitions.
    animated_selector_states: HashMap<String, AnimatedSelectorState>,
    /// Runtime extended/collapsed override for `ExtendedFloatingActionButton`, keyed by node `id`.
    pub fab_extended: HashMap<String, bool>,
    /// Bounded internal scrim alpha fade motion for `RULE_COLLAPSING_TOOLBAR_EXT`.
    collapsing_scrim_motion: HashMap<String, XmlLiftMotion>,
    /// Bounded internal expand/collapse width motion for `ExtendedFloatingActionButton`.
    extended_fab_motion: HashMap<String, XmlLiftMotion>,
    /// Bounded internal hide-on-scroll slide motion for `RULE_BOTTOM_APP_BAR_EXT`.
    bottom_app_bar_hide_motion: HashMap<String, BottomAppBarHideState>,
    /// Bounded cache of rasterized `BottomAppBar` cradle-cutout surfaces.
    bottom_app_bar_images: HashMap<BottomAppBarCradleCacheKey, Image>,
    bottom_app_bar_image_bytes: usize,
    /// True while entering top-level `Node::render_inner` root `draw` (before descending into children).
    at_root_draw: bool,
    /// Suppresses double-applying `hide_on_scroll` offset when CoordinatorLayout already shifted `BottomAppBar`.
    bab_hide_applied_by_parent: bool,
    /// Current frame clock & motion/scroll snapshot for pure layout evaluation (AppBarLayout snap settle).
    frame_time: f64,
    frame_reduced_motion: bool,
    frame_scroll_dragging: bool,
    /// Bounded open/close ease-out animation state for `RULE_SEARCH_VIEW_EXT` overlays.
    search_view_motion: HashMap<String, XmlLiftMotion>,
    /// Transient per-frame `(anchor_bar_id, search_view_key)` coupling table populated at root render.
    search_bar_links: Vec<(String, String)>,
    /// Bounded pull-to-refresh drag/retract physics state for `RULE_SWIPE_REFRESH` / `RULE_SWIPE_REFRESH_EXT`.
    swipe_refresh_motion: HashMap<String, SwipeRefreshMotion>,
    /// Bounded drag & focal snap settle state for `RULE_CAROUSEL_LAYOUT` containers.
    carousel_motion: HashMap<String, CarouselMotion>,
    /// Most recently clicked auto-link span text from `RULE_TEXT_LINKS_SPANS`, overwritten on each link click.
    pub last_clicked_link: Option<String>,
}

mod constraint_layout;
mod draw;
mod draw_adapter_views;
mod draw_containers;
mod draw_widgets;
mod drawables;
mod helpers;
mod measure;
mod render;
mod specs;
mod state;
mod text;

use constraint_layout::*;
use drawables::*;
use helpers::*;
use specs::*;
use text::*;
pub use helpers::{quick_contact_default_avatar, view_id};

#[cfg(test)]
mod object_animation_offset_tests {
    use super::*;
    #[test]
    fn object_delays_leave_properties_at_identity_and_each_axis_has_own_timing() {
        let a=AlphaAnimation{from:0.0,to:1.0,duration:0.4,start_offset:0.2,linear:true};
        assert_eq!(a.sample(0.1,true),1.0);
        assert_eq!(a.sample(0.1,false),0.0);
        assert!((a.sample(0.4,true)-0.5).abs()<0.001);
        let zero=AnimationDelta{value:0.0,mode:0};
        let t=TranslateAnimation{deltas:[AnimationDelta{value:40.0,mode:0},zero,AnimationDelta{value:20.0,mode:0},zero],duration:0.4,start_offset:0.2,linear:true,y_timing:Some((0.4,true,0.5))};
        let b=Rect{x:0.,y:0.,width:100.,height:100.};
        assert_eq!(t.sample(0.1,b,b,true),[0.0,0.0]);
        assert_eq!(t.sample(0.1,b,b,false),[40.0,20.0]);
        let v=t.sample(0.4,b,b,true);assert!((v[0]-20.0).abs()<0.001);assert_eq!(v[1],0.0);
        let v=t.sample(0.7,b,b,true);assert_eq!(v[0],0.0);assert!((v[1]-10.0).abs()<0.001);
        assert!((t.total_duration()-0.9).abs()<0.001);
    }
}
