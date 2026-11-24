//! Bhoo: small, event-driven UI foundations for SpectreOS.
//! Application values remain caller-owned. Coordinates are logical pixels.
#[cfg(test)]
extern crate self as bhoo;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub mod icons;
pub mod media;
pub mod motion;
pub mod platform;
mod ripple;
pub mod scroll;
pub mod shapes;
pub mod systems;
pub mod timing;

pub use icons::SystemIcon;
pub use media::{Image, ImageFit};
pub use scroll::EdgeStretch;
pub use shapes::{beercss_shapes, shape_data};
#[cfg(feature = "gles")]
pub use glow;
#[cfg(feature = "gles")]
pub use platform::gles::{RenderStats as GlesRenderStats, Renderer as GlesRenderer};
#[cfg(feature = "wayland")]
pub use platform::window;

// Backward-compatible top-level re-exports for Material widgets and Android compatibility.
pub use systems::android::{
    reference as android_reference,
    state::{
        calendar as android_calendar,
        date_spinner as android_date_spinner,
        dialer_filter as android_dialer_filter,
        expandable_list as android_expandable_list,
        number_picker as android_number_picker,
        time_clock as android_time_clock,
        time_spinner as android_time_spinner,
    },
    xml as android_xml,
};
pub use systems::material::{
    ActionTileOptions, AlertOptions, AlertSeverity, AlertVariant, AppBarAction, AppBarIcon,
    AutocompleteOptions, AvatarContent, AvatarGroupCounts, AvatarGroupOptions, AvatarShape,
    BadgeContent, BadgeGravity, BadgeNumberFormat, BadgeNumberLimit, BadgeOptions, BannerOptions,
    BottomAppBarOverflowState, BottomAppBarScrollBehavior, BottomAppBarState, ButtonGroupState,
    ButtonVariant, CardIconGravity, CardOptions, CardVariant, CardVisualState, CarouselItem,
    CarouselLayout, CarouselOptions, CarouselState, ChatMessage, CheckState, ChipKind,
    ChipOptions, Choice, CommandItem, DatePickerOptions, DatePickerState, DatePickerView,
    DockedToolbarEdge, DockedToolbarInsets, DockedToolbarScrollState, EmptyStateOptions,
    ExpansionPanelOptions, FabColor, FabElevation, FabMenuAction, FabMenuOptions, FabMenuState,
    FabOptions, FieldAffordance, FloatingPillSlot, FloatingToolbarExpansionScrollState,
    GridListItem, GridListOptions, ListState, MaterialIcon, MenuTypeaheadState,
    NavigationDestination, NavigationIcon, Options, OverlayDismissPolicy, OverlayKind,
    OverlaySection, OverlayState, OverlayStyle, PageDirection, PageIndicatorOptions,
    PaginationOptions, RouteTransitionKind, RouteTransitionState, ScaffoldAdaptiveOptions,
    ScaffoldOptions, ScaffoldSlots, ScrollbarDragState, ScrollbarIndicatorState,
    ScrollbarOrientation, SearchOverlayState, SelectState, ShapeRotationState, SkeletonAnimation,
    SkeletonOptions, SlideDirection, SliderExpressiveOptions, SortDirection, SplitButtonOptions,
    SplitButtonSize, SplitPaneOptions, SplitPaneOrientation, StepperState, SwipeDismissState,
    SwitchIconMode, TabItem, TabOptions, TabStyle, TableColumn, TableSortState, TimePickerMode,
    TimePickerOptions, TimePickerState, ToastOptions, ToastState, ToolbarAction,
    ToolbarFabPosition, ToolbarFabSelection, ToolbarKind, ToolbarState, TopAppBarMaterialAction,
    TopAppBarScrollBehavior, TopAppBarState, TopAppBarVariant, TreeItem, TreeState,
    WindowFrameAction, WindowFrameOptions, WindowFrameState, hue_to_color, loading, month_days,
    progress, pull_refresh, segmented_card_radii, snackbar, time_picker_label,
    time_picker_label_24h, time_picker_label_step, weekday,
};
pub use loading::LoadingState;
pub use progress::WavyProgressOptions;

/// Direction of a decorative divider's long axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DividerOrientation {
    Horizontal,
    Vertical,
}
/// Appearance and logical insets for a Material divider.
#[derive(Clone, Copy, Debug)]
pub struct DividerOptions {
    pub orientation: DividerOrientation,
    pub inset_start: f32,
    pub inset_end: f32,
    pub thickness: f32,
    pub rtl: bool,
    pub color: Option<Color>,
}
impl Default for DividerOptions {
    fn default() -> Self {
        Self { orientation: DividerOrientation::Horizontal, inset_start: 0.0, inset_end: 0.0, thickness: 1.0, rtl: false, color: None }
    }
}

use motion::{Motion, Spring};
pub use motion::{MotionPreference, MotionScheme};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rect {
    pub fn contains(self, p: [f32; 2]) -> bool {
        self.width > 0.0
            && self.height > 0.0
            && p[0] >= self.x
            && p[0] < self.x + self.width
            && p[1] >= self.y
            && p[1] < self.y + self.height
    }
    pub fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            width: ((self.x + self.width).min(other.x + other.width) - x).max(0.0),
            height: ((self.y + self.height).min(other.y + other.height) - y).max(0.0),
        }
    }
    pub fn inset(self, padding: f32) -> Self {
        Self {
            x: self.x + padding,
            y: self.y + padding,
            width: (self.width - padding * 2.0).max(0.0),
            height: (self.height - padding * 2.0).max(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);
impl Color {
    pub fn mix(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Self(
            lerp(self.0, other.0),
            lerp(self.1, other.1),
            lerp(self.2, other.2),
            lerp(self.3, other.3),
        )
    }
    pub fn opacity(self, opacity: f32) -> Self {
        Self(
            self.0,
            self.1,
            self.2,
            (self.3 as f32 * opacity.clamp(0.0, 1.0)).round() as u8,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub control: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub radius: f32,
    pub primary: Color,
    pub on_primary: Color,
    pub secondary: Color,
    pub outline: Color,
    pub error: Color,
    pub on_error: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
    pub surface_low: Color,
    pub surface_high: Color,
    pub surface_highest: Color,
    pub outline_variant: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub inverse_primary: Color,
    pub on_surface_variant: Color,
    pub surface_variant: Color,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color(18, 18, 22, 255),
            text: Color(240, 240, 245, 255),
            control: Color(42, 42, 50, 255),
            hovered: Color(58, 58, 68, 255),
            pressed: Color(76, 68, 100, 255),
            radius: 8.0,
            primary: Color(208, 188, 255, 255),
            on_primary: Color(56, 30, 114, 255),
            secondary: Color(74, 68, 88, 255),
            outline: Color(147, 143, 153, 255),
            error: Color(255, 180, 171, 255),
            on_error: Color(105, 0, 5, 255),
            on_secondary: Color(232, 222, 248, 255),
            secondary_container: Color(74, 68, 88, 255),
            on_secondary_container: Color(232, 222, 248, 255),
            surface_low: Color(29, 27, 32, 255),
            surface_high: Color(43, 41, 48, 255),
            surface_highest: Color(54, 52, 59, 255),
            outline_variant: Color(73, 69, 79, 255),
            primary_container: Color(79, 55, 139, 255),
            on_primary_container: Color(234, 221, 255, 255),
            tertiary_container: Color(99, 59, 72, 255),
            on_tertiary_container: Color(255, 216, 228, 255),
            inverse_surface: Color(230, 225, 229, 255),
            inverse_on_surface: Color(49, 48, 51, 255),
            inverse_primary: Color(103, 80, 164, 255),
            on_surface_variant: Color(202, 196, 208, 255),
            surface_variant: Color(73, 69, 79, 255),
        }
    }
}
impl Theme {
    /// Material Design 3 light theme preset (matching Bhoo's dark `Theme::default()`).
    pub fn light() -> Self {
        Self {
            background: Color(250, 250, 248, 255),
            text: Color(31, 35, 32, 255),
            control: Color(238, 240, 235, 255),
            hovered: Color(221, 231, 222, 255),
            pressed: Color(200, 221, 206, 255),
            radius: 8.0,
            primary: Color(43, 106, 77, 255),
            on_primary: Color(255, 255, 255, 255),
            secondary: Color(220, 235, 224, 255),
            outline: Color(121, 132, 123, 255),
            error: Color(183, 57, 57, 255),
            on_error: Color(255, 255, 255, 255),
            on_secondary: Color(31, 53, 40, 255),
            secondary_container: Color(220, 235, 224, 255),
            on_secondary_container: Color(31, 53, 40, 255),
            surface_low: Color(246, 246, 242, 255),
            surface_high: Color(232, 235, 229, 255),
            surface_highest: Color(226, 229, 221, 255),
            outline_variant: Color(192, 201, 192, 255),
            primary_container: Color(200, 235, 212, 255),
            on_primary_container: Color(12, 55, 32, 255),
            tertiary_container: Color(207, 231, 246, 255),
            on_tertiary_container: Color(20, 53, 70, 255),
            inverse_surface: Color(45, 49, 46, 255),
            inverse_on_surface: Color(242, 243, 238, 255),
            inverse_primary: Color(160, 210, 179, 255),
            on_surface_variant: Color(70, 79, 72, 255),
            surface_variant: Color(220, 229, 220, 255),
        }
    }

    /// Linearly interpolate every color token and corner radius between `self` and `other`.
    pub fn mix(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            background: self.background.mix(other.background, t),
            text: self.text.mix(other.text, t),
            control: self.control.mix(other.control, t),
            hovered: self.hovered.mix(other.hovered, t),
            pressed: self.pressed.mix(other.pressed, t),
            radius: self.radius + (other.radius - self.radius) * t,
            primary: self.primary.mix(other.primary, t),
            on_primary: self.on_primary.mix(other.on_primary, t),
            secondary: self.secondary.mix(other.secondary, t),
            outline: self.outline.mix(other.outline, t),
            error: self.error.mix(other.error, t),
            on_error: self.on_error.mix(other.on_error, t),
            on_secondary: self.on_secondary.mix(other.on_secondary, t),
            secondary_container: self.secondary_container.mix(other.secondary_container, t),
            on_secondary_container: self.on_secondary_container.mix(other.on_secondary_container, t),
            surface_low: self.surface_low.mix(other.surface_low, t),
            surface_high: self.surface_high.mix(other.surface_high, t),
            surface_highest: self.surface_highest.mix(other.surface_highest, t),
            outline_variant: self.outline_variant.mix(other.outline_variant, t),
            primary_container: self.primary_container.mix(other.primary_container, t),
            on_primary_container: self.on_primary_container.mix(other.on_primary_container, t),
            tertiary_container: self.tertiary_container.mix(other.tertiary_container, t),
            on_tertiary_container: self.on_tertiary_container.mix(other.on_tertiary_container, t),
            inverse_surface: self.inverse_surface.mix(other.inverse_surface, t),
            inverse_on_surface: self.inverse_on_surface.mix(other.inverse_on_surface, t),
            inverse_primary: self.inverse_primary.mix(other.inverse_primary, t),
            on_surface_variant: self.on_surface_variant.mix(other.on_surface_variant, t),
            surface_variant: self.surface_variant.mix(other.surface_variant, t),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Tab,
    Enter,
    Space,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Character(char),
    Escape,
    Menu,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyInput {
    pub key: Key,
    pub pressed: bool,
    pub repeat: bool,
    pub shift: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerKind {
    #[default]
    Mouse,
    Touch,
    Pen,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RippleStyle {
    #[default]
    MaterialWeb,
    AngularMaterial,
}

/// One input snapshot. Press/release are edges, time is monotonic seconds.
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub pointer: Option<[f32; 2]>,
    pub pointer_kind: PointerKind,
    pub pressed: bool,
    pub released: bool,
    pub cancelled: bool,
    pub blurred: bool,
    pub key: Option<KeyInput>,
    /// Logical pixel scroll delta, positive y scrolls down.
    pub scroll: [f32; 2],
    pub time: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontFamily {
    #[default]
    Sans,
    Serif,
    Monospace,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub weight: u16,
    /// 0 = start, 1 = center, 2 = end.
    pub align: u8,
    pub padding: f32,
    pub letter_spacing: f32,
    pub italic: bool,
    pub family: FontFamily,
}
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 16.0,
            weight: 400,
            align: 0,
            padding: 0.0,
            letter_spacing: 0.0,
            italic: false,
            family: FontFamily::Sans,
        }
    }
}
impl TextStyle {
    pub const LABEL: Self = Self {
        size: 14.0,
        weight: 500,
        align: 1,
        padding: 16.0,
        letter_spacing: 0.096,
        italic: false,
        family: FontFamily::Sans,
    };
}
/// Backend-supplied normalized ASCII advances. Fixed storage; no per-frame font queries.
#[derive(Clone, Copy, Debug)]
pub struct TextMetrics {
    pub normal: [f32; 95],
    pub medium: [f32; 95],
    /// Baseline displacement from the center of a text line, in em units.
    pub normal_baseline_offset: f32,
    pub medium_baseline_offset: f32,
}
impl Default for TextMetrics {
    fn default() -> Self {
        Self {
            normal: [0.55; 95],
            medium: [0.55; 95],
            normal_baseline_offset: 0.3,
            medium_baseline_offset: 0.3,
        }
    }
}
impl TextMetrics {
    pub fn lines(
        &self,
        text: &str,
        max_width: f32,
        style: TextStyle,
        mut line: impl FnMut(&str),
    ) -> usize {
        let mut start = 0;
        let mut end = 0;
        let mut width = 0.0;
        let mut count = 0;
        for word in text.split_whitespace() {
            let offset = word.as_ptr() as usize - text.as_ptr() as usize;
            let w = self.width(word, style);
            let gap = if end > start {
                self.width(&text[end..offset], style)
            } else {
                0.0
            };
            if end > start && (width + gap + w > max_width || text[end..offset].contains('\n')) {
                line(&text[start..end]);
                count += 1;
                start = offset;
                width = w;
            } else {
                if end == start {
                    start = offset;
                }
                width += gap + w;
            }
            end = offset + word.len();
        }
        if end > start {
            line(&text[start..end]);
            count += 1;
        } else if count == 0 {
            line("");
            count = 1;
        }
        count
    }
    pub fn width(&self, text: &str, style: TextStyle) -> f32 {
        let advances = if style.weight >= 500 {
            &self.medium
        } else {
            &self.normal
        };
        let mut count = 0;
        let mut width = 0.0;
        // ASCII is the common path. For other text use one fallback advance per
        // grapheme, so combining marks and joined emoji are not counted twice.
        let visible = |c:char| !c.is_control() && !matches!(c,'\u{00ad}'|'\u{200b}'|'\u{200c}'|'\u{200d}'|'\u{2060}'|'\u{feff}');
        let mut add = |ch:char,wide:bool| {
            let code = ch as usize;
            width += if wide {
                style.size
            } else if (32..127).contains(&code) {
                advances[code - 32] * style.size
            } else {
                style.size * 0.6
            };
            count += 1;
        };
        if text.is_ascii() {
            for ch in text.chars().filter(|c|visible(*c)) {add(ch,false);}
        } else {
            for grapheme in text.graphemes(true) {
                let columns=UnicodeWidthStr::width(grapheme);
                if columns>0 && let Some(ch)=grapheme.chars().find(|c|visible(*c)) {add(ch,columns>=2);}
            }
        }
        width + (count - 1).max(0) as f32 * style.letter_spacing
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    Clear(Color),
    RoundedRect {
        bounds: Rect,
        radius: f32,
        color: Color,
    },
    Ellipse { bounds: Rect, color: Color },
    Outline { bounds: Rect, radius:f32, width:f32, color:Color },
    Text {
        bounds: Rect,
        text: Arc<str>,
        color: Color,
        style: TextStyle,
    },
    Shadow {
        bounds: Rect,
        radius: f32,
        offset: [f32; 2],
        blur: f32,
        spread: f32,
        color: Color,
    },
    /// Soft radial ink, clipped to the control's rounded shape in every renderer.
    Ripple {
        bounds: Rect,
        corner_radius: f32,
        center: [f32; 2],
        radius: f32,
        color: Color,
        soft: bool,
    },
    Loading {
        bounds: Rect,
        sample: loading::LoadingSample,
        color: Color,
    },
    /// A caller-owned RGBA image with a normalized source crop.
    Image {
        bounds: Rect,
        uv: [f32; 4],
        radius: f32,
        color: Color,
        image: Image,
    },
    /// A static, tessellated Beer CSS shape.
    Shape {
        bounds: Rect,
        index: u8,
        color: Color,
        rotation: f32,
    },
    Arc {
        bounds: Rect,
        start: f32,
        sweep: f32,
        stroke: f32,
        color: Color,
        rounded: bool,
    },
    Line {
        from: [f32; 2],
        to: [f32; 2],
        width: f32,
        color: Color,
    },
    /// A two- or three-point stroke with flat caps and a miter join.
    Polyline {
        points: [[f32; 2]; 3],
        count: u8,
        width: f32,
        color: Color,
    },
    PushClip(Rect),
    PopClip,
    PushScaleY {
        origin: f32,
        scale: f32,
    },
    PopScaleY,
    PushTransform {
        origin: [f32; 2],
        translate: [f32; 2],
        scale: [f32; 2],
        rotation_rad: f32,
    },
    PopTransform,
    PushLayer {
        bounds: Rect,
        alpha: f32,
    },
    PopLayer,
}

/// Axis-aligned clip of the shape's square viewport after rotation.
pub fn shape_clip_bounds(bounds: Rect, rotation: f32) -> Rect {
    if rotation == 0.0 {
        return bounds;
    }
    let size = bounds.width.min(bounds.height);
    let (sin, cos) = rotation.sin_cos();
    let extent = size * (sin.abs() + cos.abs());
    Rect {
        x: bounds.x + (bounds.width - extent) / 2.0,
        y: bounds.y + (bounds.height - extent) / 2.0,
        width: extent,
        height: extent,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    #[default]
    Button,
    Checkbox,
    RadioGroup,
    Radio,
    Switch,
    Slider,
    TabList,
    Tab,
    ListBox,
    ListItem,
    Progress,
    Tooltip,
    Group,
    TextField,
    Menu,
    MenuItem,
    Link,
    /// Momentary activation toggles a persistent on/off state.
    ToggleButton,
    /// Non-interactive polite status message for platform accessibility bridges.
    Status,
    /// Non-focusable modal dialog container and its descriptive text.
    Dialog,
    DialogDescription,
    /// A carousel collection and an actionable item within it.
    Carousel,
    CarouselItem,
}

/// Backend-independent semantic snapshot. A native accessibility bridge is future work.
#[derive(Clone, Debug)]
pub struct SemanticNode {
    pub id: u64,
    pub part: u32,
    pub role: Role,
    pub label: Arc<str>,
    pub bounds: Rect,
    pub enabled: bool,
    pub focused: bool,
    pub selected: bool,
    pub checked: Option<CheckState>,
    pub invalid: bool,
    pub filled: bool,
    pub required: bool,
    pub value: Option<f32>,
    pub range: Option<[f32; 2]>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Response {
    pub id: u64,
    pub bounds: Rect,
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub focused: bool,
    pub changed: bool,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Capture {
    Pointer,
    Keyboard,
}

struct CachedText {
    text: Arc<str>,
    seen: u64,
}

pub struct Ui {
    pub theme: Theme,
    pub text_metrics: TextMetrics,
    /// Optional full render viewport, distinct from the padded content layout bounds.
    pub viewport: Option<Rect>,
    pub high_contrast: bool,
    pub ripple_style: RippleStyle,
    pub motion_preference: MotionPreference,
    pub motion_scheme: MotionScheme,
    active: Option<u64>,
    capture: Capture,
    focused: Option<u64>,
    pending_focus: Option<u64>,
    pending_focus_keyboard: bool,
    keyboard_focus: bool,
    focus_order: Vec<u64>,
    next_focus_order: Vec<u64>,
    part_focus: HashMap<u64, usize>,
    scrolls: HashMap<u64, scroll::ScrollPhysics>,
    checkbox_transitions: HashMap<u64, (CheckState, CheckState)>,
    seen: HashSet<u64>,
    motion: Motion,
    ripples: ripple::Ripples,
    pause_generation: u64,
    overlay_active: bool,
    tooltip_visible: bool,
    tooltip_dismissed: Option<u64>,
    tooltip_started: Option<(u64, f64)>,
    tooltip_showing: Option<u64>,
    semantics: Vec<SemanticNode>,
    text_cache: HashMap<Arc<str>, CachedText>,
    text_bytes: usize,
    epoch: u64,
    time: f64,
    continuous: bool,
    wakeup: Option<f64>,
    hover: Option<u64>,
    hover_start: f64,
    overlays: Vec<DrawCommand>,
}
impl Default for Ui {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            text_metrics: TextMetrics::default(),
            viewport: None,
            high_contrast: false,
            ripple_style: RippleStyle::default(),
            motion_preference: MotionPreference::Full,
            motion_scheme: MotionScheme::Standard,
            active: None,
            capture: Capture::Pointer,
            focused: None,
            pending_focus: None,
            pending_focus_keyboard: false,
            keyboard_focus: false,
            focus_order: Vec::new(),
            next_focus_order: Vec::new(),
            part_focus: HashMap::new(),
            scrolls: HashMap::new(),
            checkbox_transitions: HashMap::new(),
            seen: HashSet::new(),
            motion: Motion::default(),
            ripples: ripple::Ripples::default(),
            pause_generation: 0,
            overlay_active: false,
            tooltip_visible: false,
            tooltip_dismissed: None,
            tooltip_started: None,
            tooltip_showing: None,
            semantics: Vec::new(),
            text_cache: HashMap::new(),
            text_bytes: 0,
            epoch: 0,
            time: 0.0,
            continuous: false,
            wakeup: None,
            hover: None,
            hover_start: 0.0,
            overlays: Vec::new(),
        }
    }
}
impl Ui {
    pub fn needs_redraw(&self) -> bool {
        self.continuous || self.motion.is_animating()
    }
    pub fn next_wakeup(&self) -> Option<f64> {
        self.wakeup
    }
    pub fn semantics(&self) -> &[SemanticNode] {
        &self.semantics
    }
    pub fn pause_animations(&mut self) {
        self.pause_generation = self.pause_generation.wrapping_add(1);
        self.motion.pause_clocks();
        self.ripples.pause();
        self.tooltip_started = None;
        self.tooltip_showing = None;
        self.active = None;
        self.pending_focus = None;
        self.wakeup = None;
        self.continuous = false;
    }
    pub fn has_escape_target(&self) -> bool {
        self.overlay_active || self.tooltip_visible || self.active.is_some()
    }
    pub fn focused(&self) -> Option<u64> {
        self.focused
    }
    /// Transfer focus from a host editing adapter to an enabled control from the last frame.
    pub fn adapter_focus(&mut self, id: Option<u64>) {
        self.focused = id.filter(|id| self.focus_order.contains(id));
        self.keyboard_focus = false;
    }
    /// Transfer focus from a keyboard or assistive-technology control, retaining
    /// the visible keyboard focus treatment. Unknown or disabled IDs clear focus.
    pub fn accessibility_focus(&mut self, id: Option<u64>) {
        self.focused = id.filter(|id| self.focus_order.contains(id));
        self.keyboard_focus = self.focused.is_some();
    }
    pub fn cached_text_bytes(&self) -> usize {
        self.text_bytes
    }
    pub fn retained_motion_properties(&self) -> usize {
        self.motion.retained_properties()
    }
    pub fn frame(
        &mut self,
        input: Input,
        bounds: Rect,
        gap: f32,
        build: impl FnOnce(&mut Frame<'_>),
    ) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        self.frame_into(input, bounds, gap, &mut commands, build);
        commands
    }
    pub fn frame_into(
        &mut self,
        input: Input,
        bounds: Rect,
        gap: f32,
        commands: &mut Vec<DrawCommand>,
        build: impl FnOnce(&mut Frame<'_>),
    ) {
        self.epoch = self.epoch.wrapping_add(1);
        if input.time.is_finite() {
            self.time = input.time.max(self.time);
        }
        self.motion.begin(self.time, self.motion_preference);
        self.ripples.begin();
        self.continuous = false;
        self.wakeup = None;
        self.semantics.clear();
        self.seen.clear();
        self.next_focus_order.clear();
        self.overlays.clear();
        self.overlay_active = false;
        self.tooltip_visible = false;
        commands.clear();
        if input.cancelled || input.blurred {
            self.active = None;
        }
        if input.blurred {
            self.pending_focus = None;
            self.focused = None;
            self.keyboard_focus = false;
        }
        if let Some(key) = input.key
            && key.key == Key::Tab
            && key.pressed
        {
            self.active = None;
            self.keyboard_focus = true;
            let len = self.focus_order.len();
            if len > 0 {
                let previous = self
                    .focused
                    .and_then(|id| self.focus_order.iter().position(|x| *x == id));
                let index = match (previous, key.shift) {
                    (Some(i), false) => (i + 1) % len,
                    (Some(i), true) => (i + len - 1) % len,
                    (None, false) => 0,
                    (None, true) => len - 1,
                };
                self.focused = Some(self.focus_order[index]);
            }
        }
        let clip = self.viewport.unwrap_or(bounds);
        let mut frame = Frame {
            ui: self,
            input,
            bounds,
            clip,
            cursor: bounds.y,
            gap: gap.max(0.0),
            commands: std::mem::take(commands),
            enabled: true,
            key_consumed: false,
            feedback_blocked: false,
            overlay_phase: None,
            overlay_nested_scroll: 0.0,
            content_color: None,
        };
        frame
            .commands
            .push(DrawCommand::Clear(frame.ui.theme.background));
        build(&mut frame);
        if input.released
            || input.cancelled
            || input.blurred
            || frame
                .ui
                .active
                .is_some_and(|id| !frame.ui.seen.contains(&id))
        {
            frame.ui.active = None;
        }
        if let Some(id) = frame.ui.pending_focus {
            if frame.ui.next_focus_order.contains(&id) {
                frame.ui.focused = Some(id);
                frame.ui.keyboard_focus = frame.ui.pending_focus_keyboard;
                frame.ui.pending_focus = None;
                for node in &mut frame.ui.semantics {
                    node.focused = node.id == id && node.enabled;
                }
                frame.ui.continuous = true;
            } else if !frame.ui.seen.contains(&id) {
                frame.ui.pending_focus = None;
            }
        }
        if frame
            .ui
            .focused
            .is_some_and(|id| !frame.ui.next_focus_order.contains(&id))
        {
            frame.ui.focused = None;
        }
        if frame
            .ui
            .hover
            .is_some_and(|id| !frame.ui.seen.contains(&id))
        {
            frame.ui.hover = None;
        }
        if frame
            .ui
            .tooltip_dismissed
            .is_some_and(|id| !frame.ui.seen.contains(&id))
        {
            frame.ui.tooltip_dismissed = None;
        }
        if frame
            .ui
            .tooltip_started
            .is_some_and(|(id, _)| !frame.ui.seen.contains(&id))
        {
            frame.ui.tooltip_started = None;
            frame.ui.tooltip_showing = None;
        }
        frame.ui.motion.finish();
        frame.ui.ripples.finish();
        frame
            .ui
            .part_focus
            .retain(|id, _| frame.ui.seen.contains(id));
        frame
            .ui
            .checkbox_transitions
            .retain(|id, _| frame.ui.seen.contains(id));
        frame.commands.append(&mut frame.ui.overlays);
        *commands = frame.commands;
        std::mem::swap(&mut frame.ui.focus_order, &mut frame.ui.next_focus_order);
        // Reclaim unused labels at a low frequency. Cache insertions are hard-bounded.
        if frame.ui.epoch.is_multiple_of(60) {
            let epoch = frame.ui.epoch;
            frame.ui.text_cache.retain(|_, value| value.seen == epoch);
            frame.ui.text_bytes = frame.ui.text_cache.keys().map(|s| s.len()).sum();
        }
    }
    fn text(&mut self, text: &str) -> Arc<str> {
        if let Some(cached) = self.text_cache.get_mut(text) {
            cached.seen = self.epoch;
            return Arc::clone(&cached.text);
        }
        let value: Arc<str> = Arc::from(text);
        if self.text_cache.len() < 1024 && self.text_bytes.saturating_add(text.len()) <= 128 * 1024
        {
            self.text_bytes += text.len();
            self.text_cache.insert(
                Arc::clone(&value),
                CachedText {
                    text: Arc::clone(&value),
                    seen: self.epoch,
                },
            );
        }
        value
    }
}

pub struct Frame<'a> {
    ui: &'a mut Ui,
    input: Input,
    bounds: Rect,
    clip: Rect,
    cursor: f32,
    gap: f32,
    commands: Vec<DrawCommand>,
    enabled: bool,
    key_consumed: bool,
    feedback_blocked: bool,
    overlay_phase: Option<systems::material::OverlayPhase>,
    overlay_nested_scroll: f32,
    content_color: Option<Color>,
}
impl Frame<'_> {
    pub fn theme(&self) -> Theme {
        self.ui.theme
    }
    pub fn input(&self) -> Input {
        self.input
    }
    pub fn clear(&mut self, color: Color) {
        self.commands.push(DrawCommand::Clear(color));
    }
    pub fn available(&self) -> Rect {
        Rect {
            y: self.cursor,
            height: (self.bounds.y + self.bounds.height - self.cursor).max(0.0),
            ..self.bounds
        }
    }
    pub fn with_enabled(&mut self, enabled: bool, build: impl FnOnce(&mut Self)) {
        let previous = self.enabled;
        self.enabled &= enabled;
        build(self);
        self.enabled = previous;
    }
    fn allocate(&mut self, height: f32) -> Rect {
        let bounds = Rect {
            x: self.bounds.x,
            y: self.cursor,
            width: self.bounds.width.max(0.0),
            height: height
                .min((self.bounds.y + self.bounds.height - self.cursor).max(0.0))
                .max(0.0),
        };
        self.cursor += height + self.gap;
        bounds
    }
    fn rect(&mut self, bounds: Rect, radius: f32, color: Color) {
        self.commands.push(DrawCommand::RoundedRect {
            bounds,
            radius,
            color,
        });
    }
    fn text_at(&mut self, bounds: Rect, text: &str, color: Color) {
        self.styled_text(bounds, text, color, TextStyle::default());
    }
    pub fn styled_text(&mut self, bounds: Rect, text: &str, color: Color, style: TextStyle) {
        let text = self.ui.text(text);
        self.commands.push(DrawCommand::Text {
            bounds,
            text,
            color,
            style,
        });
    }
    pub fn text_width(&self, text: &str, style: TextStyle) -> f32 {
        self.ui.text_metrics.width(text, style)
    }
    pub fn shadow(
        &mut self,
        bounds: Rect,
        radius: f32,
        offset: [f32; 2],
        blur: f32,
        spread: f32,
        color: Color,
    ) {
        self.commands.push(DrawCommand::Shadow {
            bounds,
            radius,
            offset,
            blur,
            spread,
            color,
        });
    }
    pub fn push_transform(
        &mut self,
        origin: [f32; 2],
        translate: [f32; 2],
        scale: [f32; 2],
        rotation_rad: f32,
    ) {
        self.commands.push(DrawCommand::PushTransform {
            origin,
            translate,
            scale,
            rotation_rad,
        });
    }
    pub fn pop_transform(&mut self) {
        self.commands.push(DrawCommand::PopTransform);
    }
    pub fn push_layer(&mut self, bounds: Rect, alpha: f32) {
        self.commands.push(DrawCommand::PushLayer {
            bounds,
            alpha: alpha.clamp(0.0, 1.0),
        });
    }
    pub fn pop_layer(&mut self) {
        self.commands.push(DrawCommand::PopLayer);
    }
    fn animate(&mut self, id: u64, property: u8, target: f32, initial: f32) -> f32 {
        self.ui
            .motion
            .effect(id, property, target, initial, Spring::EFFECTS)
    }
    fn animate_geometry(&mut self, id: u64, property: u8, target: f32, initial: f32) -> f32 {
        self.ui.motion.effect(
            id,
            property,
            target,
            initial,
            self.ui.motion_scheme.fast_spatial(),
        )
    }
    fn interact(&mut self, id: u64, bounds: Rect, enabled: bool) -> Response {
        assert!(
            self.ui.seen.insert(id),
            "Bhoo widget IDs must be unique within a frame: {id}"
        );
        let hit = bounds.intersect(self.clip);
        let enabled = enabled && self.enabled && hit.width > 0.0 && hit.height > 0.0;
        if enabled {
            self.ui.next_focus_order.push(id);
        } else {
            if self.ui.active == Some(id) {
                self.ui.active = None;
            }
            if self.ui.focused == Some(id) {
                self.ui.focused = None;
            }
        }
        let hovered = enabled && self.input.pointer.is_some_and(|p| hit.contains(p));
        if hovered && self.ui.hover != Some(id) {
            self.ui.hover = Some(id);
            self.ui.hover_start = self.ui.time;
        }
        if self.ui.hover == Some(id) && !hovered {
            self.ui.hover = None;
        }
        let mut clicked = false;
        if enabled && !self.input.cancelled && !self.input.blurred {
            if hovered && self.input.pressed && self.ui.active.is_none() {
                self.ui.active = Some(id);
                self.ui.capture = Capture::Pointer;
                self.ui.focused = Some(id);
                self.ui.keyboard_focus = false;
            }
            if self.ui.active == Some(id)
                && self.ui.capture == Capture::Pointer
                && self.input.released
            {
                clicked = hovered;
            }
            if self.ui.focused == Some(id)
                && !self.key_consumed
                && let Some(key) = self.input.key
            {
                match (key.key, key.pressed, key.repeat) {
                    (Key::Enter, true, false) => {
                        clicked = true;
                        self.key_consumed = true;
                    }
                    (Key::Space, true, false) => {
                        self.ui.active = Some(id);
                        self.ui.capture = Capture::Keyboard;
                        self.key_consumed = true;
                    }
                    (Key::Space, false, _)
                        if self.ui.active == Some(id) && self.ui.capture == Capture::Keyboard =>
                    {
                        clicked = true;
                        self.ui.active = None;
                        self.key_consumed = true;
                    }
                    (Key::Escape, true, _) => {
                        self.ui.active = None;
                        self.key_consumed = true;
                    }
                    _ => {}
                }
            }
        }
        if clicked {
            self.ui.continuous = true;
        }
        Response {
            id,
            bounds,
            hovered: hovered && self.input.pointer_kind != PointerKind::Touch,
            pressed: enabled
                && self.ui.active == Some(id)
                && !self.input.released
                && !self.input.cancelled,
            clicked,
            focused: enabled && !self.feedback_blocked && self.ui.focused == Some(id),
            changed: false,
            enabled,
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn semantic(
        &mut self,
        response: Response,
        role: Role,
        label: &str,
        selected: bool,
        checked: Option<CheckState>,
        value: Option<f32>,
        range: Option<[f32; 2]>,
    ) {
        let label = self.ui.text(label);
        self.ui.semantics.push(SemanticNode {
            id: response.id,
            part: 0,
            role,
            label,
            bounds: response.bounds.intersect(self.clip),
            enabled: response.enabled,
            focused: response.focused,
            selected,
            checked,
            invalid: false,
            filled: false,
            required: false,
            value,
            range,
        });
    }
    /// Paint the shared keyboard focus indicator for a custom semantic target.
    pub fn focus_ring(&mut self, response: Response) {
        if !self.ui.high_contrast {
            return;
        }
        let visible = response.focused && self.ui.keyboard_focus;
        let alpha = self.animate(response.id, 2, f32::from(visible), f32::from(visible));
        if alpha > 0.001 {
            let b = response.bounds;
            let c = self.ui.theme.primary.opacity(alpha);
            for r in [
                Rect {
                    x: b.x,
                    y: b.y,
                    width: b.width,
                    height: 2.0,
                },
                Rect {
                    x: b.x,
                    y: b.y + b.height - 2.0,
                    width: b.width,
                    height: 2.0,
                },
                Rect {
                    x: b.x,
                    y: b.y,
                    width: 2.0,
                    height: b.height,
                },
                Rect {
                    x: b.x + b.width - 2.0,
                    y: b.y,
                    width: 2.0,
                    height: b.height,
                },
            ] {
                self.rect(r, 1.0, c);
            }
        }
    }
    /// Material ink is a draw operation, not a second input target.
    fn ink(&mut self, r: Response, corner_radius: f32, color: Color) {
        self.ink_part(r, 0, corner_radius, color, true);
    }
    fn ink_part(
        &mut self,
        r: Response,
        part: u32,
        corner_radius: f32,
        color: Color,
        allow_label: bool,
    ) {
        let visible = r.bounds.intersect(self.clip);
        if !r.enabled
            || self.ui.motion_preference == MotionPreference::Reduced
            || self.input.blurred
            || visible.width <= 0.0
            || visible.height <= 0.0
        {
            self.ui.ripples.clear((r.id, part));
            return;
        }
        let keyboard = r.focused
            && self.input.key.is_some_and(|k| {
                k.pressed && !k.repeat && matches!(k.key, Key::Space | Key::Enter)
            });
        let pointer = self.input.pressed
            && r.pressed
            && self.ui.capture == Capture::Pointer
            && (allow_label || self.input.pointer.is_some_and(|p| r.bounds.contains(p)));
        let origin = if pointer {
            self.input.pointer.map(|p| {
                if r.bounds.contains(p) {
                    [
                        (p[0] - r.bounds.x) / r.bounds.width.max(1.0),
                        (p[1] - r.bounds.y) / r.bounds.height.max(1.0),
                    ]
                } else {
                    [0.5; 2]
                }
            })
        } else if keyboard {
            Some([0.5; 2])
        } else {
            None
        };
        let held = r.pressed && !self.input.cancelled;
        if self.input.cancelled {
            self.ui.ripples.cancel_pending((r.id, part), self.ui.time);
        }
        if let Some(sample) = self.ui.ripples.sample_with_style(
            (r.id, part),
            self.ui.time,
            origin,
            held,
            pointer && self.input.pointer_kind == PointerKind::Touch,
            self.ui.ripple_style,
        ) {
            self.ui.continuous |= sample.moving;
            if let Some(wake) = sample.wake {
                self.ui.wakeup = Some(self.ui.wakeup.map_or(wake, |old| old.min(wake)));
            }
            let b = r.bounds;
            let max = b.width.max(b.height);
            let initial = (max * 0.2).floor().max(1.0);
            let diameter = b.width.hypot(b.height) + 10.0 + (max * 0.35).max(75.0);
            let mut radius = (initial + (diameter - initial) * sample.growth) * 0.5;
            let start = [
                b.x + sample.origin[0] * b.width,
                b.y + sample.origin[1] * b.height,
            ];
            let mut center = [
                start[0] + (b.x + b.width * 0.5 - start[0]) * sample.growth,
                start[1] + (b.y + b.height * 0.5 - start[1]) * sample.growth,
            ];
            let angular = sample.style == RippleStyle::AngularMaterial;
            if angular {
                center = start;
                radius = (b.width * sample.origin[0].max(1.0 - sample.origin[0]))
                    .hypot(b.height * sample.origin[1].max(1.0 - sample.origin[1]))
                    * sample.growth;
            }
            if sample.opacity > 0.0 {
                self.commands.push(DrawCommand::Ripple {
                    bounds: b,
                    corner_radius,
                    center,
                    radius,
                    color: color.opacity(0.12 * sample.opacity),
                    soft: !angular,
                });
            }
        }
    }
    fn key_for(&mut self, response: Response) -> Option<Key> {
        if response.enabled
            && response.focused
            && !self.key_consumed
            && let Some(key) = self.input.key
            && key.pressed
        {
            self.key_consumed = true;
            return Some(key.key);
        }
        None
    }
    pub fn label(&mut self, text: impl AsRef<str>) {
        let text = text.as_ref();
        let style = TextStyle::default();
        let metrics = self.ui.text_metrics;
        let width = self.bounds.width;
        let count = metrics.lines(text, width, style, |_| {}).max(1);
        let height = (count as f32 * 24.0).max(28.0);
        let b = self.allocate(height);
        let mut index = 0;
        metrics.lines(text, width, style, |line| {
            let y = b.y + (height - count as f32 * 24.0) * 0.5 + index as f32 * 24.0;
            self.styled_text(
                Rect {
                    y,
                    height: 24.0,
                    ..b
                },
                line,
                self.content_color.unwrap_or(self.ui.theme.text),
                style,
            );
            index += 1;
        });
    }
}

#[cfg(test)]
#[path = "lib.spec.rs"]
mod tests;
