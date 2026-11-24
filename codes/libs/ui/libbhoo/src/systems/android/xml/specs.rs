use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum XmlButtonVariant {
    Filled,
    Outlined,
    Tonal,
    Text,
    Elevated,
}

pub(super) fn parse_button_variant(anchor: &str) -> Option<XmlButtonVariant> {
    match anchor.trim().to_ascii_lowercase().as_str() {
        "filled" => Some(XmlButtonVariant::Filled),
        "outlined" => Some(XmlButtonVariant::Outlined),
        "tonal" => Some(XmlButtonVariant::Tonal),
        "text" => Some(XmlButtonVariant::Text),
        "elevated" => Some(XmlButtonVariant::Elevated),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum XmlChipKind {
    Assist,
    Filter,
    Input,
    Suggestion,
}

#[derive(Clone, Copy)]
pub(super) struct XmlChipSpec {
    pub(super) kind: XmlChipKind,
    pub(super) close_visible: bool,
}

pub(super) fn parse_chip_style(anchor: &str) -> XmlChipSpec {
    let mut parts = anchor.split('|');
    let kind = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "filter" => XmlChipKind::Filter,
        "input" => XmlChipKind::Input,
        "suggestion" => XmlChipKind::Suggestion,
        _ => XmlChipKind::Assist,
    };
    let close_visible = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    XmlChipSpec { kind, close_visible }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TextInputBoxMode {
    Filled,
    Outline,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TextInputEndIcon {
    None,
    ClearText,
    PasswordToggle,
    DropdownMenu,
}

#[derive(Clone, Copy)]
pub(super) struct TextInputBoxSpec<'a> {
    pub(super) mode: TextInputBoxMode,
    pub(super) corner_radii_dp: [f32; 4],
    pub(super) stroke_color: Option<Color>,
    pub(super) hint: &'a str,
    pub(super) helper: &'a str,
    pub(super) error_enabled: bool,
    pub(super) end_icon: TextInputEndIcon,
}

#[derive(Clone, Copy)]
pub(super) struct TextInputChildBox {
    pub(super) filled: bool,
    pub(super) has_label: bool,
    pub(super) password_toggle: bool,
    pub(super) has_start_icon: bool,
    pub(super) prefix: &'static str,
    pub(super) suffix: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct BottomSheetSpec {
    pub(super) peek_h_dp: f32,
    pub(super) hideable: bool,
    pub(super) fit_to_contents: bool,
    pub(super) half_ratio: f32,
    pub(super) skip_collapsed: bool,
}

#[derive(Clone, Copy)]
pub(super) struct SideSheetSpec {
    pub(super) edge_start: bool,
    pub(super) coplanar: bool,
}

#[derive(Clone, Copy)]
pub(super) struct WavyProgressSpec {
    pub(super) wavelength_dp: f32,
    pub(super) amplitude_dp: f32,
    pub(super) wave_speed_dp: f32,
    pub(super) stop_size_dp: f32,
    pub(super) track_gap_dp: f32,
}

#[derive(Clone, Copy)]
pub(super) struct XmlTabOptionsSpec {
    pub(super) scrollable: bool,
    pub(super) style: crate::systems::material::TabStyle,
    pub(super) indicator_color: Option<Color>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SliderLabelBehavior {
    Floating,
    WithinBounds,
    Gone,
    Visible,
}

#[derive(Clone, Copy)]
pub(super) struct SliderOptionsSpec {
    pub(super) step_size: f32,
    pub(super) min_separation: f32,
    pub(super) label_behavior: SliderLabelBehavior,
    pub(super) expressive: bool,
}

#[derive(Clone, Copy)]
pub(super) struct TextInputExtSpec<'a> {
    pub(super) counter_enabled: bool,
    pub(super) counter_max: usize,
    pub(super) prefix: &'a str,
    pub(super) suffix: &'a str,
    pub(super) start_icon: &'a str,
}

#[derive(Clone, Copy)]
pub(super) struct ChipGroupWrapSpec {
    pub(super) single_line: bool,
    pub(super) single_selection: bool,
    pub(super) h_spacing_dp: f32,
    pub(super) v_spacing_dp: f32,
}

#[derive(Clone, Copy)]
pub(super) struct SplitButtonRole {
    pub(super) is_phys_left: bool,
    pub(super) is_trailing_chevron: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum NavBarKind {
    Bar,
    Rail,
    Drawer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum NavLabelMode {
    Auto,
    Labeled,
    Selected,
    Unlabeled,
}

#[derive(Clone, Copy)]
pub(super) struct NavBarSpec {
    pub(super) kind: NavBarKind,
    pub(super) label_mode: NavLabelMode,
}

#[derive(Clone, Copy)]
pub(super) struct NavItemSpec {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) icon_name: &'static str,
}

pub(super) fn parse_nav_bar(anchor: &str) -> NavBarSpec {
    let mut parts = anchor.split('|');
    let kind = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "rail" | "navigation_rail" | "navigationrail" => NavBarKind::Rail,
        "drawer" | "navigation_drawer" | "navigationview" => NavBarKind::Drawer,
        _ => NavBarKind::Bar,
    };
    let label_mode = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "labeled" => NavLabelMode::Labeled,
        "selected" => NavLabelMode::Selected,
        "unlabeled" => NavLabelMode::Unlabeled,
        _ => NavLabelMode::Auto,
    };
    NavBarSpec { kind, label_mode }
}

pub(super) fn parse_nav_item(anchor: &'static str) -> NavItemSpec {
    let mut parts = anchor.split('|');
    let id = parts.next().unwrap_or("").trim();
    let title = parts.next().unwrap_or("").trim();
    let icon_name = parts.next().unwrap_or("").trim();
    NavItemSpec { id, title, icon_name }
}

#[derive(Clone, Copy)]
pub(super) struct CollapsingToolbarSpec<'a> {
    pub(super) title: &'a str,
    pub(super) expanded_h_dp: f32,
    pub(super) scrim: Option<Color>,
}

pub(super) fn parse_collapsing_toolbar(anchor: &str, node_height: Size) -> CollapsingToolbarSpec<'_> {
    let mut parts = anchor.split('|');
    let title = parts.next().unwrap_or("").trim();
    let fallback_h = match node_height {
        Size::Dp(h) if h.is_finite() && h > 0.0 => h,
        _ => 152.0,
    };
    let expanded_h_dp = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
        .unwrap_or(fallback_h)
        .max(64.0);
    let scrim = parts.next().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        let mut it = trimmed.split(',').map(|p| p.trim().parse::<u8>().ok());
        let c = Color(it.next()??, it.next()??, it.next()??, it.next()??);
        it.next().is_none().then_some(c)
    });
    CollapsingToolbarSpec { title, expanded_h_dp, scrim }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CollapsingTitleMode {
    Scale,
    Fade,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CollapsingToolbarExt<'a> {
    pub(super) mode: CollapsingTitleMode,
    pub(super) max_lines: usize,
    pub(super) scrim_dur_ms: u32,
    pub(super) subtitle: &'a str,
}

pub(super) fn parse_collapsing_toolbar_ext(anchor: &str) -> Option<CollapsingToolbarExt<'_>> {
    let trimmed = anchor.trim();
    if trimmed.to_ascii_lowercase().starts_with("collapse:") {
        return None;
    }
    let mut parts = anchor.splitn(4, '|');
    let mode_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let mode = match mode_raw.as_str() {
        "fade" => CollapsingTitleMode::Fade,
        _ => CollapsingTitleMode::Scale,
    };
    let max_lines = parts
        .next()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let scrim_dur_ms = parts
        .next()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);
    let subtitle = parts.next().unwrap_or("").trim();
    Some(CollapsingToolbarExt { mode, max_lines, scrim_dur_ms, subtitle })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum CollapsingChildMode {
    None,
    Pin,
    Parallax(f32),
}

pub(super) fn parse_collapsing_child_mode(anchor: &str) -> Option<CollapsingChildMode> {
    let trimmed = anchor.trim();
    let rest = trimmed
        .strip_prefix("collapse:")
        .or_else(|| {
            trimmed
                .get(..9)
                .filter(|p| p.eq_ignore_ascii_case("collapse:"))
                .map(|_| &trimmed[9..])
        })?;
    let mut parts = rest.split(':');
    let mode = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let mult = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite());
    match mode.as_str() {
        "pin" => Some(CollapsingChildMode::Pin),
        "parallax" => Some(CollapsingChildMode::Parallax(mult.unwrap_or(0.5))),
        "none" => Some(CollapsingChildMode::None),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExtendedFabStrategy {
    Auto,
    WrapContent,
    MatchParent,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ExtendedFabSpec {
    pub(super) static_extended: bool,
    pub(super) collapsed_size_dp: f32,
    pub(super) strategy: ExtendedFabStrategy,
    pub(super) has_ext: bool,
}

pub(super) fn parse_extended_fab_ext(anchor: &str, legacy_extended: Option<bool>) -> ExtendedFabSpec {
    let mut parts = anchor.split('|');
    let ext_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let static_extended = if ext_raw.is_empty() {
        legacy_extended.unwrap_or(true)
    } else {
        !matches!(ext_raw.as_str(), "0" | "false")
    };
    let collapsed_size_dp = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
        .unwrap_or(56.0);
    let strategy = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "match_parent" => ExtendedFabStrategy::MatchParent,
        "wrap_content" => ExtendedFabStrategy::WrapContent,
        _ => ExtendedFabStrategy::Auto,
    };
    ExtendedFabSpec {
        static_extended,
        collapsed_size_dp,
        strategy,
        has_ext: true,
    }
}

/// Pure evaluation of a binary `0.0 <-> 1.0` ease-out transition using `state.frame_time` & `state.frame_reduced_motion`.
pub(super) fn eval_binary_ease_out_motion(
    prev: XmlLiftMotion,
    target_on: bool,
    dur_s: f64,
    now: f64,
    reduced_motion: bool,
) -> (XmlLiftMotion, bool) {
    let mut m = prev;
    let instant = reduced_motion || (now == 0.0 && m.last_time == 0.0) || dur_s <= 0.0;
    let target = if target_on { 1.0_f32 } else { 0.0_f32 };
    let mut animating = false;
    if !m.initialized {
        m.initialized = true;
        m.last_lifted = target_on;
        m.last_time = now;
        m.progress = target;
        m.anim = None;
    } else {
        if let Some((start, from, to, dur)) = m.anim {
            if instant || dur <= 0.0 {
                m.progress = to;
                m.anim = None;
            } else {
                let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                let ease = 1.0 - (1.0 - p) * (1.0 - p);
                m.progress = (from + (to - from) * ease).clamp(0.0, 1.0);
                if p >= 1.0 {
                    m.progress = to;
                    m.anim = None;
                } else {
                    animating = true;
                }
            }
        }
        if target_on != m.last_lifted {
            m.last_lifted = target_on;
            if instant || (m.progress - target).abs() <= 0.001 {
                m.progress = target;
                m.anim = None;
            } else {
                m.anim = Some((now, m.progress, target, dur_s));
                animating = true;
            }
        }
        m.last_time = now;
    }
    (m, animating)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecyclerLayoutManager {
    Linear,
    Grid,
    Staggered,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RecyclerLayoutExt {
    pub(super) manager: RecyclerLayoutManager,
    pub(super) horizontal: bool,
    pub(super) span_count: usize,
    pub(super) reverse: bool,
    pub(super) stack_end: bool,
}

pub(super) fn parse_recycler_layout_ext(anchor: &str, default_grid: bool) -> RecyclerLayoutExt {
    let mut parts = anchor.split('|');
    let mgr_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let manager = match mgr_raw.as_str() {
        "staggered" => RecyclerLayoutManager::Staggered,
        "grid" => RecyclerLayoutManager::Grid,
        "linear" => RecyclerLayoutManager::Linear,
        _ if default_grid => RecyclerLayoutManager::Grid,
        _ => RecyclerLayoutManager::Linear,
    };
    let orient_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let horizontal = matches!(orient_raw.as_str(), "h" | "horizontal" | "0");
    let raw_span = parts
        .next()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&v| v > 0);
    let span_count = match manager {
        RecyclerLayoutManager::Linear => 1,
        RecyclerLayoutManager::Grid | RecyclerLayoutManager::Staggered => {
            raw_span.unwrap_or(1).clamp(1, 256)
        }
    };
    let reverse = matches!(
        parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str(),
        "1" | "true"
    );
    let stack_end = matches!(
        parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str(),
        "1" | "true"
    );
    RecyclerLayoutExt {
        manager,
        horizontal,
        span_count,
        reverse,
        stack_end,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewPagerOrientation {
    Horizontal,
    Vertical,
}

pub(super) fn parse_view_pager_orientation(anchor: &str) -> ViewPagerOrientation {
    match anchor.trim().to_ascii_lowercase().as_str() {
        "vertical" | "v" | "1" => ViewPagerOrientation::Vertical,
        _ => ViewPagerOrientation::Horizontal,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewPagerTransformer {
    None,
    ZoomOut,
    Depth,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewPagerSpec {
    pub(super) orient: ViewPagerOrientation,
    pub(super) user_input_enabled: bool,
    pub(super) page_margin_dp: f32,
    pub(super) transformer: ViewPagerTransformer,
    pub(super) has_ext: bool,
}

pub(super) fn parse_view_pager_ext(anchor: &str, legacy: Option<ViewPagerOrientation>) -> ViewPagerSpec {
    let mut parts = anchor.split('|');
    let orient_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let orient = match orient_raw.as_str() {
        "v" | "vertical" | "1" => ViewPagerOrientation::Vertical,
        "h" | "horizontal" | "0" => ViewPagerOrientation::Horizontal,
        _ => legacy.unwrap_or(ViewPagerOrientation::Horizontal),
    };
    let user_input_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let user_input_enabled = !matches!(user_input_raw.as_str(), "0" | "false");
    let page_margin_dp = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.0);
    let transformer_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let transformer = match transformer_raw.as_str() {
        "zoom_out" | "zoomout" => ViewPagerTransformer::ZoomOut,
        "depth" => ViewPagerTransformer::Depth,
        _ => ViewPagerTransformer::None,
    };
    ViewPagerSpec {
        orient,
        user_input_enabled,
        page_margin_dp,
        transformer,
        has_ext: true,
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SearchViewExtSpec<'a> {
    pub(super) anchor_bar_id: &'a str,
    pub(super) hint: &'a str,
    pub(super) hide_nav: bool,
    pub(super) use_drawer_arrow: bool,
}

pub(super) fn parse_search_view_ext(anchor: &str) -> SearchViewExtSpec<'_> {
    let mut parts = anchor.split('|');
    let anchor_bar_id = parts.next().unwrap_or("").trim();
    let hint = parts.next().unwrap_or("").trim();
    let hide_nav = matches!(
        parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str(),
        "1" | "true"
    );
    let use_drawer_arrow = matches!(
        parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str(),
        "1" | "true"
    );
    SearchViewExtSpec {
        anchor_bar_id,
        hint,
        hide_nav,
        use_drawer_arrow,
    }
}

#[derive(Clone, Debug)]
pub(super) struct SwipeRefreshExtSpec {
    pub(super) bg_color: Option<Color>,
    pub(super) scheme_colors: Vec<Color>,
    pub(super) large: bool,
    pub(super) start_off_dp: f32,
    pub(super) end_off_dp: f32,
}

pub(super) fn parse_swipe_refresh_ext(anchor: &str) -> SwipeRefreshExtSpec {
    let mut parts = anchor.split('|');
    let bg_raw = parts.next().unwrap_or("").trim();
    let bg_color = if bg_raw.is_empty() {
        None
    } else {
        parse_rgba_csv(bg_raw)
    };
    let scheme_raw = parts.next().unwrap_or("").trim();
    let scheme_colors = if scheme_raw.is_empty() {
        Vec::new()
    } else {
        scheme_raw
            .split(';')
            .filter_map(|tok| parse_rgba_csv(tok.trim()))
            .collect()
    };
    let size_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let large = matches!(size_raw.as_str(), "large" | "1");
    let offs_raw = parts.next().unwrap_or("").trim();
    let (mut start_off_dp, mut end_off_dp) = (-40.0_f32, 64.0_f32);
    if !offs_raw.is_empty() {
        let mut oit = offs_raw.split(',');
        if let Some(s) = oit
            .next()
            .and_then(|v| v.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
        {
            start_off_dp = s;
        }
        if let Some(e) = oit
            .next()
            .and_then(|v| v.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
        {
            end_off_dp = e;
        }
    }
    SwipeRefreshExtSpec {
        bg_color,
        scheme_colors,
        large,
        start_off_dp,
        end_off_dp,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CarouselStrategy {
    MultiBrowse,
    Hero,
    Uncontained,
    FullScreen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CarouselAlign {
    Start,
    Center,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CarouselLayoutSpec {
    pub(super) strategy: CarouselStrategy,
    pub(super) spacing_dp: f32,
    pub(super) align: CarouselAlign,
}

pub(super) fn parse_carousel_layout(anchor: &str) -> CarouselLayoutSpec {
    let mut parts = anchor.split('|');
    let strat_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let strategy = match strat_raw.as_str() {
        "hero" => CarouselStrategy::Hero,
        "uncontained" => CarouselStrategy::Uncontained,
        "full_screen" | "fullscreen" => CarouselStrategy::FullScreen,
        _ => CarouselStrategy::MultiBrowse,
    };
    let spacing_raw = parts.next().unwrap_or("").trim();
    let default_spacing = if strategy == CarouselStrategy::FullScreen { 0.0 } else { 8.0 };
    let spacing_dp = if spacing_raw.is_empty() {
        default_spacing
    } else {
        spacing_raw
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(default_spacing)
    };
    let align_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let align = match align_raw.as_str() {
        "center" | "1" => CarouselAlign::Center,
        _ => CarouselAlign::Start,
    };
    CarouselLayoutSpec {
        strategy,
        spacing_dp,
        align,
    }
}

pub(super) fn collect_search_view_links(node: &Node, id: u64, out: &mut Vec<(String, String)>) {
    if let Some(sv) = node.search_view_ext()
        && !sv.anchor_bar_id.is_empty()
    {
        let sv_key = if node.id.is_empty() {
            format!("#{id}")
        } else {
            node.id.to_owned()
        };
        out.push((sv.anchor_bar_id.to_owned(), sv_key));
    }
    for (i, child) in node.children.iter().enumerate() {
        collect_search_view_links(child, child_view_id(id, i, child), out);
    }
}

pub(super) fn parse_range_slider_values(anchor: &str, min: f32, max: f32) -> [f32; 2] {
    let mut it = anchor.split(',').map(|p| p.trim().parse::<f32>().ok());
    let a = it.next().flatten().filter(|v| v.is_finite());
    let b = it.next().flatten().filter(|v| v.is_finite());
    let (mut v0, mut v1) = match (a, b) {
        (Some(x), Some(y)) => (x, y),
        (Some(x), None) => (x, max),
        _ => (min, max),
    };
    v0 = v0.clamp(min, max);
    v1 = v1.clamp(min, max);
    if v0 > v1 {
        std::mem::swap(&mut v0, &mut v1);
    }
    [v0, v1]
}

pub(super) fn find_scroll_view_offset(node: &Node, state: &LayoutState, id: u64) -> Option<f32> {
    if node.effective_visibility(state) == Visibility::Gone {
        return None;
    }
    if matches!(
        node.kind,
        Kind::Scroll | Kind::ListView | Kind::GridView | Kind::ExpandableListView
    ) && !node.recycler_layout_ext().is_some_and(|e| e.horizontal) {
        let key = if node.id.is_empty() { format!("#{id}") } else { node.id.to_owned() };
        return Some(
            state
                .offsets
                .get(&key)
                .copied()
                .filter(|v| v.is_finite())
                .unwrap_or(0.0)
                .max(0.0),
        );
    }
    for (i, child) in node.children.iter().enumerate() {
        if let Some(off) = find_scroll_view_offset(child, state, child_view_id(id, i, child)) {
            return Some(off);
        }
    }
    None
}

pub(super) fn collect_descendant_scrolls(
    node: &Node,
    state: &LayoutState,
    id: u64,
    out: &mut Vec<(u64, String, f32)>,
) {
    if node.effective_visibility(state) == Visibility::Gone {
        return;
    }
    if matches!(
        node.kind,
        Kind::Scroll | Kind::ListView | Kind::GridView | Kind::ExpandableListView
    ) {
        let key = if node.id.is_empty() {
            format!("#{id}")
        } else {
            node.id.to_owned()
        };
        let off = state
            .offsets
            .get(&key)
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(0.0);
        out.push((id, key, off));
    }
    for (i, child) in node.children.iter().enumerate() {
        collect_descendant_scrolls(child, state, child_view_id(id, i, child), out);
    }
}

pub(super) fn parse_material_icon(raw: &str) -> Option<MaterialIcon> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut s = trimmed
        .strip_prefix("@drawable/")
        .or_else(|| trimmed.strip_prefix("@android:drawable/"))
        .unwrap_or(trimmed)
        .to_ascii_lowercase();
    if let Some(rest) = s.strip_prefix("ic_") {
        s = rest.to_owned();
    }
    for suffix in [
        "_black_24dp",
        "_white_24dp",
        "_24dp",
        "_24",
        "_outlined",
        "_outline",
        "_filled",
        "_rounded",
        "_sharp",
    ] {
        if let Some(rest) = s.strip_suffix(suffix) {
            s = rest.to_owned();
        }
    }
    let compact: String = s.chars().filter(|c| *c != '_' && *c != '-').collect();
    let icon = match compact.as_str() {
        "widgets" | "apps" | "dashboard" | "explore" | "browse" | "grid" | "category" => MaterialIcon::Widgets,
        "code" | "developer" | "terminal" => MaterialIcon::Code,
        "palette" | "theme" | "color" | "style" | "brush" => MaterialIcon::Palette,
        "pause" => MaterialIcon::Pause,
        "darkmode" | "night" | "moon" => MaterialIcon::DarkMode,
        "lightmode" | "day" | "sun" => MaterialIcon::LightMode,
        "download" | "offline" | "save" => MaterialIcon::Download,
        "menu" | "hamburger" => MaterialIcon::Menu,
        "menuopen" => MaterialIcon::MenuOpen,
        "verticalaligntop" => MaterialIcon::VerticalAlignTop,
        "verticalalignbottom" => MaterialIcon::VerticalAlignBottom,
        "close" | "clear" | "cancel" => MaterialIcon::Close,
        "check" | "done" | "ok" => MaterialIcon::Check,
        "add" | "plus" | "new" => MaterialIcon::Add,
        "arrowback" | "back" => MaterialIcon::ArrowBack,
        "arrowforward" | "forward" => MaterialIcon::ArrowForward,
        "search" | "find" => MaterialIcon::Search,
        "morevert" | "more" => MaterialIcon::MoreVert,
        "home" | "house" => MaterialIcon::Home,
        "settings" | "tune" | "preferences" | "gear" => MaterialIcon::Settings,
        "inbox" => MaterialIcon::Inbox,
        "star" | "favorite" | "favorites" | "saved" | "heart" | "bookmark" => MaterialIcon::Star,
        "archive" | "folder" | "collections" | "library" => MaterialIcon::Archive,
        "chevronright" | "next" => MaterialIcon::ChevronRight,
        "person" | "profile" | "account" | "user" | "contacts" => MaterialIcon::Person,
        "delete" | "trash" | "remove" => MaterialIcon::Delete,
        "markemailunread" | "notifications" | "notification" | "bell" | "updates" => MaterialIcon::MarkEmailUnread,
        "mail" | "email" | "envelope" => MaterialIcon::Mail,
        "chatbubble" | "chat" | "message" | "messages" | "forum" | "comment" => MaterialIcon::ChatBubble,
        "videocam" | "video" | "meet" | "call" => MaterialIcon::Videocam,
        "edit" | "compose" | "create" | "pencil" => MaterialIcon::Edit,
        "sell" | "tag" | "label" | "shop" | "store" | "shopping" => MaterialIcon::Sell,
        "attachfile" | "attach" | "paperclip" => MaterialIcon::AttachFile,
        "send" => MaterialIcon::Send,
        "reply" => MaterialIcon::Reply,
        "morehoriz" => MaterialIcon::MoreHoriz,
        "chevronleft" | "prev" | "previous" => MaterialIcon::ChevronLeft,
        "image" | "photo" | "photos" | "gallery" | "media" => MaterialIcon::Image,
        "sentimentsatisfied" | "mood" | "smile" | "emoji" => MaterialIcon::SentimentSatisfied,
        "description" | "document" | "doc" | "file" | "files" | "article" | "notes" | "list" => MaterialIcon::Description,
        _ => MaterialIcon::Widgets,
    };
    Some(icon)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BottomAppBarFabAlignment {
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BottomAppBarAnchorMode {
    Cradle,
    Embed,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BottomAppBarSpec {
    pub(super) alignment: BottomAppBarFabAlignment,
    pub(super) anchor_mode: BottomAppBarAnchorMode,
    pub(super) cradle_margin_dp: f32,
    pub(super) cradle_rad_dp: f32,
    pub(super) cradle_voff_dp: f32,
    pub(super) hide_on_scroll: bool,
    pub(super) has_ext: bool,
}

pub(super) fn parse_bottom_app_bar_ext(
    anchor: &str,
    legacy_align: Option<BottomAppBarFabAlignment>,
) -> BottomAppBarSpec {
    let mut parts = anchor.split('|');
    let align_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let alignment = match align_raw.as_str() {
        "end" => BottomAppBarFabAlignment::End,
        "center" => BottomAppBarFabAlignment::Center,
        _ => legacy_align.unwrap_or(BottomAppBarFabAlignment::Center),
    };
    let anchor_mode = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "embed" => BottomAppBarAnchorMode::Embed,
        _ => BottomAppBarAnchorMode::Cradle,
    };
    let mut cradle_margin_dp = 5.0_f32;
    let mut cradle_rad_dp = 8.0_f32;
    let mut cradle_voff_dp = 0.0_f32;
    if let Some(cradle_raw) = parts.next() {
        let mut cparts = cradle_raw.split(',');
        if let Some(v) = cparts
            .next()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v >= 0.0)
        {
            cradle_margin_dp = v;
        }
        if let Some(v) = cparts
            .next()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v >= 0.0)
        {
            cradle_rad_dp = v;
        }
        if let Some(v) = cparts
            .next()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
        {
            cradle_voff_dp = v;
        }
    }
    let hide_on_scroll = matches!(
        parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str(),
        "1" | "true"
    );
    BottomAppBarSpec {
        alignment,
        anchor_mode,
        cradle_margin_dp,
        cradle_rad_dp,
        cradle_voff_dp,
        hide_on_scroll,
        has_ext: true,
    }
}

pub(super) fn bottom_app_bar_fab_center(
    bar_rect: Rect,
    spec: BottomAppBarSpec,
    fab_w: f32,
    state: &LayoutState,
) -> (f32, f32) {
    let end_margin = state.dp(16.0);
    let fab_half = if fab_w > 0.0 { fab_w * 0.5 } else { state.dp(28.0) };
    let cx = match spec.alignment {
        BottomAppBarFabAlignment::Center => bar_rect.x + bar_rect.width * 0.5,
        BottomAppBarFabAlignment::End => {
            if state.rtl {
                bar_rect.x + end_margin + fab_half
            } else {
                bar_rect.x + bar_rect.width - end_margin - fab_half
            }
        }
    };
    let voff = state.dp(spec.cradle_voff_dp);
    let cy = match spec.anchor_mode {
        BottomAppBarAnchorMode::Cradle => bar_rect.y + voff,
        BottomAppBarAnchorMode::Embed => bar_rect.y + bar_rect.height * 0.5 + voff,
    };
    (cx, cy)
}

pub(super) fn eval_bottom_app_bar_hide(
    key: &str,
    scroll_px: f32,
    state: &LayoutState,
) -> (BottomAppBarHideState, bool) {
    let now = if state.frame_time.is_finite() { state.frame_time.max(0.0) } else { 0.0 };
    let mut hs = state.bottom_app_bar_hide_motion.get(key).copied().unwrap_or_default();
    let instant = state.frame_reduced_motion || (now == 0.0 && hs.last_time == 0.0);
    let mut animating = false;
    if !hs.initialized {
        hs.initialized = true;
        hs.last_scroll = scroll_px;
        hs.hidden = false;
        hs.last_time = now;
        hs.progress = 0.0;
        hs.anim = None;
    } else {
        if let Some((start, from, to, dur)) = hs.anim {
            if instant || dur <= 0.0 {
                hs.progress = to;
                hs.anim = None;
            } else {
                let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                let ease = 1.0 - (1.0 - p) * (1.0 - p);
                hs.progress = (from + (to - from) * ease).clamp(0.0, 1.0);
                if p >= 1.0 {
                    hs.progress = to;
                    hs.anim = None;
                } else {
                    animating = true;
                }
            }
        }
        let delta = scroll_px - hs.last_scroll;
        let next_hidden = if delta > 0.5 {
            true
        } else if delta < -0.5 {
            false
        } else {
            hs.hidden
        };
        hs.last_scroll = scroll_px;
        if next_hidden != hs.hidden {
            hs.hidden = next_hidden;
            let target = if next_hidden { 1.0_f32 } else { 0.0_f32 };
            if instant || (hs.progress - target).abs() <= 0.001 {
                hs.progress = target;
                hs.anim = None;
            } else {
                hs.anim = Some((now, hs.progress, target, 0.22));
                animating = true;
            }
        }
        hs.last_time = now;
    }
    (hs, animating)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum BadgeGravity {
    #[default]
    TopEnd,
    TopStart,
    BottomEnd,
    BottomStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BadgeTarget<'a> {
    SelfView,
    Index(usize),
    Id(&'a str),
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MaterialBadgeExt {
    pub(super) gravity: BadgeGravity,
    pub(super) max_chars: usize,
    pub(super) hoff_dp: f32,
    pub(super) voff_dp: f32,
}

pub(super) fn parse_material_badge_ext(anchor: &str) -> Option<(BadgeTarget<'_>, MaterialBadgeExt)> {
    let mut parts = anchor.split('|');
    let target_raw = parts.next()?.trim();
    let target = if target_raw.is_empty()
        || target_raw == "-1"
        || target_raw.eq_ignore_ascii_case("self")
    {
        BadgeTarget::SelfView
    } else if let Ok(idx) = target_raw.parse::<usize>() {
        BadgeTarget::Index(idx)
    } else {
        BadgeTarget::Id(target_raw)
    };
    let gravity = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "top_start" | "topstart" | "8388659" => BadgeGravity::TopStart,
        "bottom_end" | "bottomend" | "8388693" => BadgeGravity::BottomEnd,
        "bottom_start" | "bottomstart" | "8388691" => BadgeGravity::BottomStart,
        _ => BadgeGravity::TopEnd,
    };
    let max_chars = parts
        .next()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let (hoff_dp, voff_dp) = parts
        .next()
        .map(|s| {
            let mut xy = s.split(',');
            let h = xy
                .next()
                .and_then(|v| v.trim().parse::<f32>().ok())
                .filter(|v| v.is_finite())
                .unwrap_or(0.0);
            let v = xy
                .next()
                .and_then(|v| v.trim().parse::<f32>().ok())
                .filter(|v| v.is_finite())
                .unwrap_or(0.0);
            (h, v)
        })
        .unwrap_or((0.0, 0.0));
    Some((
        target,
        MaterialBadgeExt {
            gravity,
            max_chars,
            hoff_dp,
            voff_dp,
        },
    ))
}

pub(super) fn format_badge_text<'a>(raw: &'a str, max_chars: usize) -> std::borrow::Cow<'a, str> {
    if max_chars == 0 || raw.is_empty() {
        return std::borrow::Cow::Borrowed(raw);
    }
    let trimmed = raw.trim();
    if let Ok(val) = trimmed.parse::<u64>() {
        if max_chars >= 2 {
            let exp = ((max_chars - 1) as u32).min(19);
            let max_num = 10u64.saturating_pow(exp).saturating_sub(1);
            if val > max_num {
                return std::borrow::Cow::Owned(format!("{max_num}+"));
            }
        } else if val >= 10 {
            return std::borrow::Cow::Borrowed("+");
        }
    }
    std::borrow::Cow::Borrowed(raw)
}

pub(super) fn paint_nav_badge_ext(
    f: &mut Frame<'_>,
    icon_rect: Rect,
    badge_text: &str,
    ext: Option<MaterialBadgeExt>,
    state: &LayoutState,
    disabled_alpha: f32,
) {
    let t = f.theme();
    let bg = t.error.opacity(disabled_alpha);
    let gravity = ext.map_or(BadgeGravity::TopEnd, |e| e.gravity);
    let is_start = matches!(gravity, BadgeGravity::TopStart | BadgeGravity::BottomStart);
    let is_bottom = matches!(gravity, BadgeGravity::BottomStart | BadgeGravity::BottomEnd);
    let phys_left = if state.rtl { !is_start } else { is_start };
    let dx = ext.map_or(0.0, |e| {
        let px = state.dp(e.hoff_dp);
        if state.rtl { -px } else { px }
    });
    let dy = ext.map_or(0.0, |e| state.dp(e.voff_dp));
    let display_text = format_badge_text(badge_text, ext.map_or(0, |e| e.max_chars));
    if display_text.is_empty() {
        let d = state.dp(6.0);
        let base_x = if phys_left {
            icon_rect.x - d * 0.5
        } else {
            icon_rect.x + icon_rect.width - d * 0.5
        };
        let base_y = if is_bottom {
            icon_rect.y + icon_rect.height - d
        } else {
            icon_rect.y
        };
        let dot = Rect {
            x: base_x + dx,
            y: base_y + dy,
            width: d,
            height: d,
        };
        f.paint_rect(dot, d * 0.5, bg);
    } else {
        let h = state.dp(16.0);
        let style = TextStyle {
            size: state.sp(11.0),
            weight: 700,
            align: 1,
            ..TextStyle::default()
        };
        let w = (f.text_width(&display_text, style) + state.dp(8.0)).max(h);
        let corner_inset = (icon_rect.width * 0.5).min(state.dp(12.0));
        let base_x = if phys_left {
            icon_rect.x + corner_inset - w
        } else {
            icon_rect.x + icon_rect.width - corner_inset
        };
        let base_y = if is_bottom {
            icon_rect.y + icon_rect.height - h + state.dp(4.0)
        } else {
            icon_rect.y - state.dp(4.0)
        };
        let pill = Rect {
            x: base_x + dx,
            y: base_y + dy,
            width: w,
            height: h,
        };
        f.paint_rect(pill, h * 0.5, bg);
        f.styled_text(pill, &display_text, t.on_error.opacity(disabled_alpha), style);
    }
}

#[derive(Clone, Copy)]
pub(super) struct ToggleGroupSpec {
    pub(super) single_selection: bool,
    pub(super) selection_required: bool,
    pub(super) checked_id: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct ToggleGroupSegment {
    pub(super) is_phys_left: bool,
    pub(super) is_phys_right: bool,
}

pub(super) fn parse_toggle_group(anchor: &'static str) -> ToggleGroupSpec {
    let mut parts = anchor.split('|');
    let single_selection = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    let selection_required = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    let checked_id = parts.next().unwrap_or("").trim();
    ToggleGroupSpec {
        single_selection,
        selection_required,
        checked_id,
    }
}

pub(super) fn parse_text_input_box(anchor: &str) -> TextInputBoxSpec<'_> {
    let mut parts = anchor.split('|');
    let mode_raw = parts.next().unwrap_or("").trim().to_ascii_lowercase();
    let mode = match mode_raw.as_str() {
        "outline" | "outlined" => TextInputBoxMode::Outline,
        "none" => TextInputBoxMode::None,
        _ => TextInputBoxMode::Filled,
    };
    let corner_raw = parts.next().unwrap_or("").trim();
    let corner_radii_dp = if corner_raw.contains(',') {
        parse_corner_radii(corner_raw).unwrap_or(match mode {
            TextInputBoxMode::Filled => [4.0, 4.0, 0.0, 0.0],
            _ => [4.0; 4],
        })
    } else {
        let c = corner_raw
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(4.0);
        match mode {
            TextInputBoxMode::Filled => [c, c, 0.0, 0.0],
            _ => [c; 4],
        }
    };
    let stroke_raw = parts.next().unwrap_or("").trim();
    let stroke_color = if stroke_raw.is_empty() || stroke_raw.eq_ignore_ascii_case("none") {
        None
    } else {
        let mut rgba = stroke_raw.split(',').map(|p| p.trim().parse::<u8>().ok());
        (|| {
            let c = Color(rgba.next()??, rgba.next()??, rgba.next()??, rgba.next()??);
            if rgba.next().is_some() {
                return None;
            }
            (c.3 > 0).then_some(c)
        })()
    };
    let hint = parts.next().unwrap_or("").trim();
    let helper = parts.next().unwrap_or("").trim();
    let error_enabled = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    let end_icon = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "clear_text" | "clear" | "2" => TextInputEndIcon::ClearText,
        "password_toggle" | "password" | "1" => TextInputEndIcon::PasswordToggle,
        "dropdown_menu" | "dropdown" | "3" => TextInputEndIcon::DropdownMenu,
        _ => TextInputEndIcon::None,
    };
    TextInputBoxSpec {
        mode,
        corner_radii_dp,
        stroke_color,
        hint,
        helper,
        error_enabled,
        end_icon,
    }
}

pub(super) fn parse_bottom_sheet(anchor: &str) -> BottomSheetSpec {
    let mut parts = anchor.split('|');
    let peek_h_dp = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(-1.0);
    let hideable = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    let fit_to_contents = parts
        .next()
        .map(|s| !matches!(s.trim().to_ascii_lowercase().as_str(), "0" | "false"))
        .unwrap_or(true);
    let half_ratio = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 1.0)
        .unwrap_or(0.5);
    let skip_collapsed = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    BottomSheetSpec {
        peek_h_dp,
        hideable,
        fit_to_contents,
        half_ratio,
        skip_collapsed,
    }
}

pub(super) fn parse_side_sheet(anchor: &str) -> SideSheetSpec {
    let mut parts = anchor.split('|');
    let edge_start = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "start" | "left"));
    let coplanar = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    SideSheetSpec {
        edge_start,
        coplanar,
    }
}

pub(super) fn parse_wavy_progress(anchor: &str) -> WavyProgressSpec {
    let mut parts = anchor.split('|');
    let parse_f = |s: Option<&str>, default: f32| {
        s.and_then(|v| v.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .unwrap_or(default)
    };
    WavyProgressSpec {
        wavelength_dp: parse_f(parts.next(), 0.0).max(0.0),
        amplitude_dp: parse_f(parts.next(), 0.0).max(0.0),
        wave_speed_dp: parse_f(parts.next(), 0.0),
        stop_size_dp: parse_f(parts.next(), 4.0).max(0.0),
        track_gap_dp: parse_f(parts.next(), 4.0).max(0.0),
    }
}

pub(super) fn parse_tab_options(anchor: &str) -> XmlTabOptionsSpec {
    let mut parts = anchor.split('|');
    let scrollable = parts
        .next()
        .is_some_and(|s| s.trim().eq_ignore_ascii_case("scrollable"));
    let style = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "secondary" => crate::systems::material::TabStyle::Secondary,
        _ => crate::systems::material::TabStyle::Primary,
    };
    let ind_raw = parts.next().unwrap_or("").trim();
    let indicator_color = if ind_raw.is_empty() || ind_raw.eq_ignore_ascii_case("none") {
        None
    } else {
        let mut rgba = ind_raw.split(',').map(|p| p.trim().parse::<u8>().ok());
        (|| {
            let c = Color(rgba.next()??, rgba.next()??, rgba.next()??, rgba.next()??);
            if rgba.next().is_some() {
                return None;
            }
            (c.3 > 0).then_some(c)
        })()
    };
    XmlTabOptionsSpec {
        scrollable,
        style,
        indicator_color,
    }
}

pub(super) fn parse_slider_options(anchor: &str) -> SliderOptionsSpec {
    let mut parts = anchor.split('|');
    let step_size = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.0);
    let min_separation = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.0);
    let label_behavior = match parts.next().unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "withinbounds" | "within_bounds" => SliderLabelBehavior::WithinBounds,
        "gone" => SliderLabelBehavior::Gone,
        "visible" => SliderLabelBehavior::Visible,
        _ => SliderLabelBehavior::Floating,
    };
    let expressive = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    SliderOptionsSpec {
        step_size,
        min_separation,
        label_behavior,
        expressive,
    }
}

pub(super) fn parse_text_input_ext(anchor: &str) -> TextInputExtSpec<'_> {
    let mut parts = anchor.split('|');
    let counter_enabled = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    let counter_max = parts
        .next()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let prefix = parts.next().unwrap_or("");
    let suffix = parts.next().unwrap_or("");
    let start_icon = parts.next().unwrap_or("").trim();
    TextInputExtSpec {
        counter_enabled,
        counter_max,
        prefix,
        suffix,
        start_icon,
    }
}

pub(super) fn parse_chip_group_wrap(anchor: &str) -> ChipGroupWrapSpec {
    let mut parts = anchor.split('|');
    let single_line = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    let single_selection = parts
        .next()
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    let h_spacing_dp = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(8.0);
    let v_spacing_dp = parts
        .next()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(8.0);
    ChipGroupWrapSpec {
        single_line,
        single_selection,
        h_spacing_dp,
        v_spacing_dp,
    }
}

pub(super) fn paint_xml_slider_value_bubble(
    f: &mut Frame<'_>,
    hx: f32,
    anchor_y: f32,
    value: f32,
    d: f32,
    within_bounds: Option<Rect>,
) {
    let viewport = f.viewport();
    let text = if (value - value.round()).abs() < 0.005 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.1}")
    };
    let style = TextStyle {
        size: 12.0 * d,
        weight: 500,
        align: 1,
        padding: 8.0 * d,
        letter_spacing: 0.4 * d,
        ..TextStyle::default()
    };
    let pill_w = (f.text_width(&text, style) + 20.0 * d).max(40.0 * d);
    let pill_h = 32.0 * d;
    let clamp_rect = within_bounds.unwrap_or(viewport);
    let pad = if within_bounds.is_some() { 0.0 } else { 4.0 * d };
    let min_x = clamp_rect.x + pad;
    let max_x = (clamp_rect.x + clamp_rect.width - pill_w - pad).max(min_x);
    let pill_x = (hx - pill_w * 0.5).clamp(min_x, max_x);
    let pill_y = (anchor_y - pill_h - 6.0 * d).max(viewport.y + 2.0 * d);
    let pill = Rect {
        x: pill_x,
        y: pill_y,
        width: pill_w,
        height: pill_h,
    };
    let t = f.theme();
    let tip_x = hx.clamp(pill.x + 8.0 * d, (pill.x + pill.width - 8.0 * d).max(pill.x));
    let tip_base_y = pill.y + pill.height - 2.0 * d;
    let tip_y = pill.y + pill.height + 4.0 * d;
    f.ui.overlays.push(DrawCommand::RoundedRect {
        bounds: pill,
        radius: pill_h * 0.5,
        color: t.inverse_surface,
    });
    f.ui.overlays.push(DrawCommand::Line {
        from: [tip_x - 4.0 * d, tip_base_y],
        to: [tip_x, tip_y],
        width: 4.0 * d,
        color: t.inverse_surface,
    });
    f.ui.overlays.push(DrawCommand::Line {
        from: [tip_x + 4.0 * d, tip_base_y],
        to: [tip_x, tip_y],
        width: 4.0 * d,
        color: t.inverse_surface,
    });
    let cached = f.ui.text(&text);
    f.ui.overlays.push(DrawCommand::Text {
        bounds: pill,
        text: cached,
        color: t.inverse_on_surface,
        style,
    });
}

pub(super) fn find_text_input_edit<'a>(node: &'a Node, parent_id: u64) -> Option<(&'a Node, u64, String)> {
    for (i, child) in node.children.iter().enumerate() {
        let cid = child_view_id(parent_id, i, child);
        if matches!(child.kind, Kind::Edit | Kind::AutoComplete | Kind::MultiAutoComplete) {
            let key = if child.id.is_empty() { format!("#{cid}") } else { child.id.to_owned() };
            return Some((child, cid, key));
        }
        if let Some(found) = find_text_input_edit(child, cid) {
            return Some(found);
        }
    }
    None
}

pub(super) fn paint_password_visibility_icon(f: &mut Frame<'_>, rect: Rect, revealed: bool, color: Color) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let cx = rect.x + rect.width * 0.5;
    let cy = rect.y + rect.height * 0.5;
    let rx = rect.width * 0.42;
    let ry = rect.height * 0.28;
    let stroke = (rect.width * 0.09).max(1.5);
    f.commands.push(DrawCommand::Outline {
        bounds: Rect { x: cx - rx, y: cy - ry, width: rx * 2.0, height: ry * 2.0 },
        radius: -1.0,
        width: stroke,
        color,
    });
    let pupil_r = (rect.width * 0.14).max(2.0);
    paint_oval(
        f,
        Rect { x: cx - pupil_r, y: cy - pupil_r, width: pupil_r * 2.0, height: pupil_r * 2.0 },
        color,
    );
    if !revealed {
        f.paint_line(
            [cx - rx * 0.8, cy - rx * 0.8],
            [cx + rx * 0.8, cy + rx * 0.8],
            stroke,
            color,
        );
    }
}

pub(super) fn resolve_coordinator_anchor_rect(
    anchor_spec: &str,
    target: Rect,
    child_size: (f32, f32),
    inner: Rect,
    margin: [f32; 4],
    rtl: bool,
) -> Rect {
    let tokens = anchor_spec.split_once(':').map_or("", |(_, t)| t);
    let mut h_pos: Option<f32> = None;
    let mut v_pos: Option<f32> = None;
    for raw in tokens.split('|') {
        match raw.trim().to_ascii_lowercase().as_str() {
            "left" => h_pos = Some(target.x),
            "right" => h_pos = Some(target.x + target.width),
            "start" => h_pos = Some(if rtl { target.x + target.width } else { target.x }),
            "end" => h_pos = Some(if rtl { target.x } else { target.x + target.width }),
            "center_horizontal" => h_pos = Some(target.x + target.width * 0.5),
            "top" => v_pos = Some(target.y),
            "bottom" => v_pos = Some(target.y + target.height),
            "center_vertical" => v_pos = Some(target.y + target.height * 0.5),
            "center" => {
                h_pos = Some(target.x + target.width * 0.5);
                v_pos = Some(target.y + target.height * 0.5);
            }
            _ => {}
        }
    }
    let ax = h_pos.unwrap_or(if rtl { target.x + target.width } else { target.x });
    let ay = v_pos.unwrap_or(target.y);
    let (w, h) = child_size;
    let min_x = inner.x + margin[0];
    let max_x = inner.x + inner.width - w - margin[2];
    let min_y = inner.y + margin[1];
    let max_y = inner.y + inner.height - h - margin[3];
    let x = (ax - w * 0.5).min(max_x).max(min_x);
    let y = (ay - h * 0.5).min(max_y).max(min_y);
    Rect { x, y, width: w, height: h }
}

pub(super) fn is_constraint_guideline(node: &Node) -> bool {
    node.relative_rules.iter().any(|r| {
        matches!(
            r.kind,
            RULE_GUIDE_ORIENTATION | RULE_GUIDE_BEGIN | RULE_GUIDE_END | RULE_GUIDE_PERCENT
        )
    })
}

pub(super) fn is_constraint_barrier(node: &Node) -> bool {
    node.relative_rules.iter().any(|r| r.kind == RULE_BARRIER)
}

pub(super) fn is_constraint_group(node: &Node) -> bool {
    node.relative_rules.iter().any(|r| r.kind == RULE_GROUP_REFS)
}

pub(super) fn is_constraint_flow(node: &Node) -> bool {
    node.relative_rules.iter().any(|r| r.kind == RULE_FLOW)
}

pub(super) fn is_constraint_layer(node: &Node) -> bool {
    node.relative_rules.iter().any(|r| r.kind == RULE_CONSTRAINT_LAYER)
}

pub(super) fn is_constraint_placeholder(node: &Node) -> bool {
    node.relative_rules.iter().any(|r| r.kind == RULE_CONSTRAINT_PLACEHOLDER)
}

pub(super) fn is_zero_size_constraint_helper(node: &Node) -> bool {
    is_constraint_guideline(node)
        || is_constraint_barrier(node)
        || is_constraint_group(node)
        || is_constraint_layer(node)
}

pub(super) fn is_constraint_helper(node: &Node) -> bool {
    is_zero_size_constraint_helper(node) || is_constraint_flow(node)
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) struct AppBarScrollFlags {
    pub(super) scroll: bool,
    pub(super) enter_always: bool,
    pub(super) enter_always_collapsed: bool,
    pub(super) exit_until_collapsed: bool,
    pub(super) snap: bool,
    pub(super) snap_margins: bool,
}

pub(super) fn parse_app_bar_scroll_flags(anchor: &str) -> AppBarScrollFlags {
    let mut flags = AppBarScrollFlags::default();
    for tok in anchor.split([',', '|']) {
        match tok.trim().to_ascii_lowercase().as_str() {
            "scroll" => flags.scroll = true,
            "enteralways" | "enter_always" => flags.enter_always = true,
            "enteralwayscollapsed" | "enter_always_collapsed" => flags.enter_always_collapsed = true,
            "exituntilcollapsed" | "exit_until_collapsed" => flags.exit_until_collapsed = true,
            "snap" => flags.snap = true,
            "snapmargins" | "snap_margins" => flags.snap_margins = true,
            _ => {}
        }
    }
    flags
}

pub(super) fn parse_app_bar_ext_lift_color(anchor: &str) -> Option<Color> {
    for seg in anchor.split(['|', ';']) {
        let seg = seg.trim();
        if let Some((k, v)) = seg.split_once('=')
            && k.trim().eq_ignore_ascii_case("lift_color")
        {
            let mut parts = v.split(',').filter_map(|p| p.trim().parse::<u8>().ok());
            return Some(Color(parts.next()?, parts.next()?, parts.next()?, parts.next()?));
        }
    }
    None
}

pub(super) fn parse_app_bar_ext_compress(anchor: &str) -> bool {
    for seg in anchor.split(['|', ';']) {
        let seg = seg.trim();
        if let Some((k, v)) = seg.split_once('=')
            && k.trim().eq_ignore_ascii_case("effect")
            && v.trim().eq_ignore_ascii_case("compress")
        {
            return true;
        }
    }
    false
}

pub(super) fn parse_drawer_lock_mode(anchor: &str) -> Option<u8> {
    match anchor.trim().to_ascii_lowercase().as_str() {
        "unlocked" | "lock_mode_unlocked" | "0" => Some(DRAWER_LOCK_MODE_UNLOCKED),
        "locked_closed" | "lock_mode_locked_closed" | "1" => Some(DRAWER_LOCK_MODE_LOCKED_CLOSED),
        "locked_open" | "lock_mode_locked_open" | "2" => Some(DRAWER_LOCK_MODE_LOCKED_OPEN),
        "undefined" | "lock_mode_undefined" | "3" => Some(DRAWER_LOCK_MODE_UNDEFINED),
        _ => None,
    }
}

pub(super) fn app_bar_child_collapsed_layout(
    natural_h: f32,
    m_top: f32,
    m_bot: f32,
    collapse_px: f32,
    snap_margins: bool,
) -> (f32, f32, f32, f32) {
    let natural_h = natural_h.max(0.0);
    let m_top = m_top.max(0.0);
    let m_bot = m_bot.max(0.0);
    let collapse_px = collapse_px.max(0.0);
    if snap_margins {
        let eff_top = (m_top - collapse_px).max(0.0);
        let body_collapse = (collapse_px - m_top).max(0.0).min(natural_h);
        let vis_h = (natural_h - body_collapse).max(0.0);
        let eff_bot = (m_bot - (collapse_px - m_top - natural_h).max(0.0)).max(0.0);
        (eff_top, vis_h, eff_bot, body_collapse)
    } else {
        let body_collapse = collapse_px.min(natural_h);
        let vis_h = (natural_h - body_collapse).max(0.0);
        (m_top, vis_h, m_bot, body_collapse)
    }
}

#[derive(Clone, Copy)]
pub(super) struct AppBarCollapseEval {
    pub(super) collapse_px: f32,
    pub(super) next_state: AppBarScrollState,
    pub(super) wake_at: Option<f64>,
    pub(super) continuous: bool,
}

pub(super) const APP_BAR_SNAP_IDLE_DELAY_S: f64 = 0.08;
pub(super) const APP_BAR_SNAP_SETTLE_DUR_S: f64 = 0.22;

pub(super) fn eval_app_bar_child_collapse(
    child: &Node,
    child_id: u64,
    child_h: f32,
    v_margins: f32,
    scroll_px: f32,
    state: &LayoutState,
) -> Option<AppBarCollapseEval> {
    if child.collapsing_toolbar_spec().is_some() {
        return None;
    }
    let flags = child.app_bar_scroll_flags()?;
    let scroll_px = if scroll_px.is_finite() { scroll_px.max(0.0) } else { 0.0 };
    let child_h = child_h.max(0.0);
    let v_margins = v_margins.max(0.0);
    if child_h <= 0.0 && (!flags.snap_margins || v_margins <= 0.0) {
        return Some(AppBarCollapseEval {
            collapse_px: 0.0,
            next_state: AppBarScrollState::default(),
            wake_at: None,
            continuous: false,
        });
    }
    let min_h = state.dp(child.min_height.max(0.0)).clamp(0.0, child_h);
    let margin_range = if flags.snap_margins { v_margins } else { 0.0 };
    let max_collapse = if flags.exit_until_collapsed {
        (child_h + margin_range - min_h).max(0.0)
    } else {
        (child_h + margin_range).max(0.0)
    };
    let child_key = if child.id.is_empty() {
        format!("#{child_id}")
    } else {
        child.id.to_owned()
    };
    let st = state
        .app_bar_scroll_motion
        .get(&child_key)
        .copied()
        .unwrap_or_default();
    let now = if state.frame_time.is_finite() { state.frame_time.max(0.0) } else { 0.0 };
    let reduced = state.frame_reduced_motion;
    let dragging = state.frame_scroll_dragging;

    let mut next = st;
    let mut wake_at: Option<f64> = None;
    let mut continuous = false;

    if !st.initialized {
        let mut collapse = scroll_px.clamp(0.0, max_collapse);
        next.initialized = true;
        next.last_scroll = scroll_px;
        next.last_scroll_time = now;
        next.last_time = now;
        next.was_dragging = dragging;
        next.snap_anim = None;
        if flags.snap && max_collapse > 0.0 && collapse > 0.001 && collapse < max_collapse - 0.001 {
            if reduced || now == 0.0 {
                collapse = if collapse >= max_collapse * 0.5 { max_collapse } else { 0.0 };
            } else if !dragging {
                wake_at = Some((now + APP_BAR_SNAP_IDLE_DELAY_S).max(now + 0.001));
            }
        }
        next.collapse_px = collapse.clamp(0.0, max_collapse);
        return Some(AppBarCollapseEval {
            collapse_px: next.collapse_px,
            next_state: next,
            wake_at,
            continuous,
        });
    }

    let delta = scroll_px - st.last_scroll;
    let scroll_changed = delta.abs() > 0.01;
    let mut collapse = if scroll_changed {
        next.snap_anim = None;
        next.last_scroll = scroll_px;
        next.last_scroll_time = now;
        if flags.enter_always {
            if delta > 0.0 {
                (st.collapse_px + delta).clamp(0.0, max_collapse)
            } else if flags.enter_always_collapsed && min_h > 0.0 {
                let min_collapse = (child_h + margin_range - min_h).min(scroll_px).max(0.0);
                (st.collapse_px + delta).clamp(min_collapse, max_collapse)
            } else {
                let c = (st.collapse_px + delta).clamp(0.0, max_collapse);
                if scroll_px <= 0.001 { 0.0 } else { c }
            }
        } else {
            scroll_px.clamp(0.0, max_collapse)
        }
    } else if flags.enter_always || flags.snap {
        if scroll_px <= 0.001 && !flags.snap {
            0.0
        } else {
            st.collapse_px.clamp(0.0, max_collapse)
        }
    } else {
        scroll_px.clamp(0.0, max_collapse)
    };

    if dragging {
        next.last_scroll_time = now;
        next.snap_anim = None;
    } else if flags.snap && max_collapse > 0.0 {
        if scroll_changed {
            if collapse > 0.001 && collapse < max_collapse - 0.001 {
                wake_at = Some((now + APP_BAR_SNAP_IDLE_DELAY_S).max(now + 0.001));
            }
        } else if let Some((start, from, to, dur)) = next.snap_anim {
            if reduced || dur <= 0.0 || (now == 0.0 && st.last_time == 0.0) {
                collapse = to;
                next.snap_anim = None;
            } else {
                let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                let ease = 1.0 - (1.0 - p) * (1.0 - p);
                collapse = (from + (to - from) * ease).clamp(0.0, max_collapse);
                if p >= 1.0 {
                    collapse = to;
                    next.snap_anim = None;
                } else {
                    continuous = true;
                }
            }
        } else if collapse > 0.001 && collapse < max_collapse - 0.001 {
            let idle_elapsed = (now - st.last_scroll_time).max(0.0);
            let instant_zero_clock = now == 0.0 && st.last_time == 0.0;
            let scroll_stopped = st.was_dragging || instant_zero_clock || idle_elapsed >= APP_BAR_SNAP_IDLE_DELAY_S - 1e-6;
            if scroll_stopped {
                let target = if collapse >= max_collapse * 0.5 { max_collapse } else { 0.0 };
                if reduced || instant_zero_clock || (collapse - target).abs() <= 0.001 {
                    collapse = target;
                    next.snap_anim = None;
                } else {
                    next.snap_anim = Some((now, collapse, target, APP_BAR_SNAP_SETTLE_DUR_S));
                    continuous = true;
                }
            } else {
                wake_at = Some((st.last_scroll_time + APP_BAR_SNAP_IDLE_DELAY_S).max(now + 0.001));
            }
        }
    }

    next.collapse_px = collapse.clamp(0.0, max_collapse);
    next.last_time = now;
    next.was_dragging = dragging;
    Some(AppBarCollapseEval {
        collapse_px: next.collapse_px,
        next_state: next,
        wake_at,
        continuous,
    })
}

pub(super) fn preview_app_bar_child_collapse(
    child: &Node,
    child_id: u64,
    child_h: f32,
    v_margins: f32,
    scroll_px: f32,
    state: &LayoutState,
) -> f32 {
    eval_app_bar_child_collapse(child, child_id, child_h, v_margins, scroll_px, state)
        .map_or(0.0, |ev| ev.collapse_px)
}

