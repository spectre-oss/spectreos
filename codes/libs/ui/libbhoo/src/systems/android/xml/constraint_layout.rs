use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConstraintChainStyle {
    Spread,
    SpreadInside,
    Packed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RatioTarget {
    Width,
    Height,
}

pub(super) fn parse_dimension_ratio(anchor: &str) -> Option<(Option<RatioTarget>, f32)> {
    let trimmed = anchor.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (target, expr) = if let Some((prefix, rest)) = trimmed.split_once(',') {
        let side = match prefix.trim() {
            "W" | "w" => Some(RatioTarget::Width),
            "H" | "h" => Some(RatioTarget::Height),
            _ => return None,
        };
        (side, rest.trim())
    } else {
        (None, trimmed)
    };
    let ratio = if let Some((num, den)) = expr.split_once(':') {
        let w = num.trim().parse::<f32>().ok()?;
        let h = den.trim().parse::<f32>().ok()?;
        if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
            return None;
        }
        w / h
    } else {
        let r = expr.parse::<f32>().ok()?;
        if !r.is_finite() || r <= 0.0 {
            return None;
        }
        r
    };
    Some((target, ratio))
}

pub(super) fn resolved_ratio_target(node: Node) -> Option<(RatioTarget, f32)> {
    let raw = ConstraintAxis::rule(node, RULE_CONSTRAINT_DIMENSION_RATIO)?;
    let (explicit, ratio) = parse_dimension_ratio(raw)?;
    let w_zero = matches!(node.width, Size::Dp(v) if v <= 0.0);
    let h_zero = matches!(node.height, Size::Dp(v) if v <= 0.0);
    if !w_zero && !h_zero {
        return None;
    }
    let has_start = ConstraintAxis::rule(node, RULE_CONSTRAINT_START_TO_START).is_some()
        || ConstraintAxis::rule(node, RULE_CONSTRAINT_START_TO_END).is_some();
    let has_end = ConstraintAxis::rule(node, RULE_CONSTRAINT_END_TO_END).is_some()
        || ConstraintAxis::rule(node, RULE_CONSTRAINT_END_TO_START).is_some();
    let has_top = ConstraintAxis::rule(node, RULE_CONSTRAINT_TOP_TO_TOP).is_some()
        || ConstraintAxis::rule(node, RULE_CONSTRAINT_TOP_TO_BOTTOM).is_some();
    let has_bottom = ConstraintAxis::rule(node, RULE_CONSTRAINT_BOTTOM_TO_BOTTOM).is_some()
        || ConstraintAxis::rule(node, RULE_CONSTRAINT_BOTTOM_TO_TOP).is_some();
    let h_constrained = (has_start && has_end)
        || ConstraintAxis::rule(node, RULE_CONSTRAINT_WIDTH_PERCENT).is_some();
    let v_constrained = (has_top && has_bottom)
        || ConstraintAxis::rule(node, RULE_CONSTRAINT_HEIGHT_PERCENT).is_some();
    let target = match explicit {
        Some(RatioTarget::Height) => h_zero.then_some(RatioTarget::Height)?,
        Some(RatioTarget::Width) => w_zero.then_some(RatioTarget::Width)?,
        None => {
            if h_zero && !w_zero {
                RatioTarget::Height
            } else if w_zero && !h_zero {
                RatioTarget::Width
            } else if h_constrained && !v_constrained {
                RatioTarget::Height
            } else if v_constrained && !h_constrained {
                RatioTarget::Width
            } else if h_constrained {
                RatioTarget::Height
            } else {
                return None;
            }
        }
    };
    Some((target, ratio))
}

pub(super) fn parse_constraint_phys_flags(node: Node) -> u8 {
    ConstraintAxis::rule(node, RULE_CONSTRAINT_PHYS_FLAGS)
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0)
}

pub(super) fn constraint_resolved_margin(node: Node, state: &LayoutState, phys_margins: bool) -> [f32; 4] {
    if !phys_margins || !state.rtl {
        return node.resolved_margin(state);
    }
    let mut margin = node.margin;
    if let Some(value) = node.margin_relative[0] {
        margin[0] = value;
    }
    if let Some(value) = node.margin_relative[1] {
        margin[2] = value;
    }
    let d = state.effective_density();
    margin.map(|v| v * d)
}

pub(super) fn parse_gone_margins_full(
    node: Node,
    state: &LayoutState,
    phys_margins: bool,
) -> ([Option<f32>; 4], Option<f32>) {
    let Some(raw) = ConstraintAxis::rule(node, RULE_CONSTRAINT_GONE_MARGINS) else {
        return ([None; 4], None);
    };
    let parts: Vec<&str> = raw.split(',').collect();
    let parse_dp = |s: &str| {
        s.trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .map(|v| state.dp(v))
    };
    if parts.len() == 5 {
        // 5-value format: "s,t,e,b,baseline"
        let s = parse_dp(parts[0]);
        let t = parse_dp(parts[1]);
        let e = parse_dp(parts[2]);
        let b = parse_dp(parts[3]);
        let baseline = parse_dp(parts[4]);
        let (left, right) = if state.rtl && !phys_margins { (e, s) } else { (s, e) };
        return ([left, t, right, b], baseline);
    }
    let mut vals = [None; 7];
    for (i, part) in parts.iter().take(7).enumerate() {
        vals[i] = parse_dp(part);
    }
    let [l, t, r, b, s, e, baseline] = vals;
    let (left, right) = if state.rtl && !phys_margins {
        (e.or(l), s.or(r))
    } else {
        (s.or(l), e.or(r))
    };
    ([left, t, right, b], baseline)
}

pub(super) fn parse_gone_margins_phys(node: Node, state: &LayoutState, phys_margins: bool) -> [Option<f32>; 4] {
    parse_gone_margins_full(node, state, phys_margins).0
}

#[derive(Clone, Copy)]
pub(super) struct PlaceholderSpec<'a> {
    pub(super) content_id: &'a str,
    pub(super) empty_visibility: Visibility,
}

pub(super) fn parse_placeholder_spec(anchor: &str) -> PlaceholderSpec<'_> {
    let (content_part, vis_part) = anchor.split_once('|').unwrap_or((anchor, "gone"));
    let empty_visibility = match vis_part.trim().to_ascii_lowercase().as_str() {
        "visible" | "0" => Visibility::Visible,
        "invisible" | "4" => Visibility::Invisible,
        _ => Visibility::Gone,
    };
    PlaceholderSpec {
        content_id: content_part.trim(),
        empty_visibility,
    }
}

pub(super) fn resolve_constraint_placeholders(
    children: &[Node],
    state: &LayoutState,
) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
    let n = children.len();
    let mut ph_to_target = vec![None; n];
    let mut target_to_ph = vec![None; n];
    if !children.iter().any(is_constraint_placeholder) {
        return (ph_to_target, target_to_ph);
    }
    let ids: HashMap<&'static str, usize> = children
        .iter()
        .enumerate()
        .filter(|(_, node)| !node.id.is_empty())
        .map(|(i, node)| (node.id, i))
        .collect();
    for (i, child) in children.iter().enumerate() {
        let Some(spec) = child.placeholder_spec() else {
            continue;
        };
        let target_id = if !child.id.is_empty()
            && let Some(rt_id) = state.placeholder_content.get(child.id)
        {
            rt_id.trim()
        } else {
            spec.content_id
        };
        if target_id.is_empty() {
            continue;
        }
        if let Some(&t_idx) = ids.get(target_id)
            && t_idx != i
            && !is_constraint_helper(&children[t_idx])
            && !is_constraint_placeholder(&children[t_idx])
            && target_to_ph[t_idx].is_none()
        {
            ph_to_target[i] = Some(t_idx);
            target_to_ph[t_idx] = Some(i);
        }
    }
    (ph_to_target, target_to_ph)
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum ConstraintDefaultSize {
    #[default]
    Spread,
    Wrap,
    Percent,
}

pub(super) fn parse_constraint_default_size(anchor: &str) -> (ConstraintDefaultSize, ConstraintDefaultSize) {
    let parse_one = |raw: Option<&str>| -> ConstraintDefaultSize {
        match raw.unwrap_or("").trim().to_ascii_lowercase().as_str() {
            "wrap" | "1" => ConstraintDefaultSize::Wrap,
            "percent" | "2" => ConstraintDefaultSize::Percent,
            _ => ConstraintDefaultSize::Spread,
        }
    };
    let mut parts = anchor.split(',');
    (parse_one(parts.next()), parse_one(parts.next()))
}

#[derive(Clone, Copy, Default)]
pub(super) struct ConstraintSizeBounds {
    pub(super) constrained_w: bool,
    pub(super) constrained_h: bool,
    pub(super) w_min_dp: Option<f32>,
    pub(super) w_max_dp: Option<f32>,
    pub(super) h_min_dp: Option<f32>,
    pub(super) h_max_dp: Option<f32>,
}

pub(super) fn parse_constraint_size_bounds(anchor: &str) -> ConstraintSizeBounds {
    let mut out = ConstraintSizeBounds::default();
    let (flags_part, bounds_part) = anchor.split_once('|').unwrap_or((anchor, ""));
    let mut flags = flags_part.split(',');
    let parse_flag = |s: Option<&str>| {
        s.is_some_and(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true"))
    };
    out.constrained_w = parse_flag(flags.next());
    out.constrained_h = parse_flag(flags.next());
    let parse_opt_dp = |s: Option<&str>| -> Option<f32> {
        s.and_then(|v| v.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
    };
    if !bounds_part.is_empty() {
        let mut b = bounds_part.split(',');
        out.w_min_dp = parse_opt_dp(b.next());
        out.w_max_dp = parse_opt_dp(b.next());
        out.h_min_dp = parse_opt_dp(b.next());
        out.h_max_dp = parse_opt_dp(b.next());
    }
    out
}

pub(super) fn apply_constraint_size_bounds(val: f32, min_px: Option<f32>, max_px: Option<f32>) -> f32 {
    let mut s = val;
    if let Some(mn) = min_px {
        s = s.max(mn);
    }
    if let Some(mx) = max_px {
        s = s.min(mx);
    }
    s.max(0.0)
}

pub(super) struct BarrierSpec<'a> {
    pub(super) dir: &'a str,
    pub(super) refs_raw: &'a str,
    pub(super) margin_dp: f32,
    pub(super) allows_gone: bool,
}

pub(super) fn parse_barrier_spec(raw: &str) -> Option<BarrierSpec<'_>> {
    let mut segs = raw.split('|');
    let first = segs.next()?;
    let (dir, refs_raw, margin_seg, gone_seg) = if let Some((d, r)) = first.split_once(':') {
        (d.trim(), r.trim(), segs.next(), segs.next())
    } else {
        let r = segs.next()?;
        (first.trim(), r.trim(), segs.next(), segs.next())
    };
    if dir.is_empty() {
        return None;
    }
    let margin_dp = margin_seg
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(0.0);
    let allows_gone = gone_seg
        .map(|s| !matches!(s.trim().to_ascii_lowercase().as_str(), "0" | "false"))
        .unwrap_or(true);
    Some(BarrierSpec { dir, refs_raw, margin_dp, allows_gone })
}

pub(super) fn constraint_group_visibilities(children: &[Node], state: &LayoutState) -> Vec<Visibility> {
    let mut vis: Vec<Visibility> = children.iter().map(|n| n.effective_visibility(state)).collect();
    let has_group = children.iter().any(is_constraint_group);
    if has_group {
        let ids: HashMap<&'static str, usize> = children
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.id.is_empty())
            .map(|(i, n)| (n.id, i))
            .collect();
        for group in children {
            if !is_constraint_group(group) {
                continue;
            }
            let g_vis = group.effective_visibility(state);
            if !matches!(g_vis, Visibility::Gone | Visibility::Invisible) {
                continue;
            }
            for rule in group.relative_rules.iter().filter(|r| r.kind == RULE_GROUP_REFS) {
                for raw_id in rule.anchor.split(',') {
                    let id = raw_id.trim();
                    if let Some(&idx) = ids.get(id)
                        && !is_constraint_helper(&children[idx])
                    {
                        match (vis[idx], g_vis) {
                            (_, Visibility::Gone) => vis[idx] = Visibility::Gone,
                            (Visibility::Visible, Visibility::Invisible) => vis[idx] = Visibility::Invisible,
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    if children.iter().any(is_constraint_placeholder) {
        let (ph_to_target, _) = resolve_constraint_placeholders(children, state);
        for (i, child) in children.iter().enumerate() {
            let Some(spec) = child.placeholder_spec() else {
                continue;
            };
            let has_rt_vis = !child.id.is_empty() && state.visibility.contains_key(child.id);
            if let Some(t_idx) = ph_to_target[i] {
                if vis[t_idx] == Visibility::Gone {
                    vis[i] = Visibility::Gone;
                } else if !has_rt_vis {
                    vis[i] = Visibility::Visible;
                }
            } else if !has_rt_vis {
                vis[i] = spec.empty_visibility;
            }
        }
    }
    vis
}

pub(super) struct ConstraintAxis<'a> {
    pub(super) nodes: &'a [Node],
    pub(super) ids: HashMap<&'static str, usize>,
    pub(super) state: &'a LayoutState,
    pub(super) visibilities: &'a [Visibility],
    pub(super) ph_to_target: &'a [Option<usize>],
    pub(super) parent: u64,
    pub(super) origin: f32,
    pub(super) extent: f32,
    pub(super) horizontal: bool,
    pub(super) wrap_parent: bool,
    pub(super) other_axis: Option<&'a [(f32, f32)]>,
    pub(super) metrics: TextMetrics,
    pub(super) chains: Vec<Vec<usize>>,
    pub(super) chain_of: Vec<Option<usize>>,
    pub(super) result: Vec<Option<(f32, f32)>>,
    pub(super) visiting: Vec<bool>,
}

impl ConstraintAxis<'_> {
    pub(super) fn rule(node: Node, kind: u16) -> Option<&'static str> {
        node.relative_rules.iter().find(|r| r.kind == kind).map(|r| r.anchor)
    }

    pub(super) fn parse_chain_style(node: Node, horizontal: bool) -> ConstraintChainStyle {
        let kind = if horizontal { RULE_CHAIN_STYLE_H } else { RULE_CHAIN_STYLE_V };
        match Self::rule(node, kind).map(|s| s.trim().to_ascii_lowercase()).as_deref() {
            Some("spread_inside" | "1") => ConstraintChainStyle::SpreadInside,
            Some("packed" | "2") => ConstraintChainStyle::Packed,
            _ => ConstraintChainStyle::Spread,
        }
    }

    pub(super) fn parse_chain_weight(node: Node, horizontal: bool) -> Option<f32> {
        let kind = if horizontal { RULE_CONSTRAINT_H_WEIGHT } else { RULE_CONSTRAINT_V_WEIGHT };
        Self::rule(node, kind)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
    }

    pub(super) fn guideline_position(&self, n: Node) -> Option<(f32, f32)> {
        if !is_constraint_guideline(&n) {
            return None;
        }
        let vertical_guide = Self::rule(n, RULE_GUIDE_ORIENTATION)
            .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "vertical" | "1"));
        if vertical_guide != self.horizontal {
            return Some((self.origin, 0.0));
        }
        let begin = Self::rule(n, RULE_GUIDE_BEGIN)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map(|v| self.state.dp(v));
        let end = Self::rule(n, RULE_GUIDE_END)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map(|v| self.state.dp(v));
        let percent = Self::rule(n, RULE_GUIDE_PERCENT)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite());
        let rtl_h = self.horizontal && self.state.rtl;
        let pos = if let Some(v) = begin {
            if rtl_h { self.origin + self.extent - v } else { self.origin + v }
        } else if let Some(v) = end {
            if self.wrap_parent {
                if rtl_h { self.origin + self.extent } else { self.origin }
            } else if rtl_h {
                self.origin + v
            } else {
                self.origin + self.extent - v
            }
        } else if let Some(p) = percent {
            if self.wrap_parent {
                if rtl_h { self.origin + self.extent } else { self.origin }
            } else if rtl_h {
                self.origin + self.extent * (1.0 - p)
            } else {
                self.origin + self.extent * p
            }
        } else if rtl_h {
            self.origin + self.extent
        } else {
            self.origin
        };
        Some((pos, 0.0))
    }

    pub(super) fn barrier_position(&mut self, index: usize, n: Node) -> Option<(f32, f32)> {
        let raw = Self::rule(n, RULE_BARRIER)?;
        let spec = parse_barrier_spec(raw)?;
        let dir = spec.dir.to_ascii_lowercase();
        let rtl = self.state.rtl;
        let is_max = match (self.horizontal, dir.as_str()) {
            (true, "left") => false,
            (true, "right") => true,
            (true, "start") => rtl,
            (true, "end") => !rtl,
            (false, "top") => false,
            (false, "bottom") => true,
            _ => return Some((self.origin, 0.0)),
        };
        if self.visiting[index] {
            return Some((self.origin, 0.0));
        }
        self.visiting[index] = true;
        let mut best: Option<f32> = None;
        for ref_id in spec.refs_raw.split(',') {
            let ref_id = ref_id.trim();
            if let Some(&ref_idx) = self.ids.get(ref_id) {
                if ref_idx == index || self.visiting[ref_idx] {
                    continue;
                }
                let ref_helper = is_constraint_helper(&self.nodes[ref_idx]);
                let ref_gone = !ref_helper && self.visibilities[ref_idx] == Visibility::Gone;
                if ref_gone && !spec.allows_gone {
                    continue;
                }
                let (pos, size) = self.position(ref_idx);
                let eff_size = if ref_gone || is_zero_size_constraint_helper(&self.nodes[ref_idx]) {
                    0.0
                } else {
                    size
                };
                let edge = if is_max { pos + eff_size } else { pos };
                best = Some(match best {
                    Some(cur) => if is_max { cur.max(edge) } else { cur.min(edge) },
                    None => edge,
                });
            }
        }
        self.visiting[index] = false;
        let margin_px = self.state.dp(spec.margin_dp);
        let signed_margin = if is_max { margin_px } else { -margin_px };
        let fallback = if self.horizontal && rtl { self.origin + self.extent } else { self.origin };
        Some((best.map_or(fallback, |edge| edge + signed_margin), 0.0))
    }

    pub(super) fn resolve_edge_info_phys(&mut self, anchor: &str, target_high: bool, physical: bool) -> Option<(f32, bool)> {
        if anchor == "parent" {
            let rtl_h = self.horizontal && self.state.rtl && !physical;
            if self.wrap_parent && ((!rtl_h && target_high) || (rtl_h && !target_high)) {
                return None;
            }
            return Some((if target_high { self.origin + self.extent } else { self.origin }, false));
        }
        let &idx = self.ids.get(anchor)?;
        if self.visiting[idx] {
            return None;
        }
        let (pos, size) = self.position(idx);
        let is_helper = is_constraint_helper(&self.nodes[idx]);
        let target_gone = !is_helper && self.visibilities[idx] == Visibility::Gone;
        let eff_size = if target_gone || is_zero_size_constraint_helper(&self.nodes[idx]) { 0.0 } else { size };
        Some((if target_high { pos + eff_size } else { pos }, target_gone))
    }

    pub(super) fn resolve_edge_info(&mut self, anchor: &str, target_high: bool) -> Option<(f32, bool)> {
        self.resolve_edge_info_phys(anchor, target_high, false)
    }

    pub(super) fn measure_child(&self, index: usize, available: f32) -> f32 {
        let n = self.nodes[index];
        if self.horizontal {
            n.layout_intrinsic(true, self.state, available, self.metrics, child_view_id(self.parent, index, &n))
        } else {
            let w = self.other_axis.map_or(0.0, |widths| widths[index].1);
            n.layout_intrinsic_for_width(false, self.state, w, matches!(n.width, Size::Dp(v) if v == 0.0), self.metrics, child_view_id(self.parent, index, &n))
        }
    }

    pub(super) fn measure_child_wrap(&self, index: usize, available: f32) -> f32 {
        let mut n = self.nodes[index];
        if self.horizontal {
            n.width = Size::Wrap;
            n.layout_intrinsic(true, self.state, available, self.metrics, child_view_id(self.parent, index, &n))
        } else {
            n.height = Size::Wrap;
            let w = self.other_axis.map_or(0.0, |widths| widths[index].1);
            n.layout_intrinsic_for_width(false, self.state, w, true, self.metrics, child_view_id(self.parent, index, &n))
        }
    }

    pub(super) fn resolve_chain(&mut self, chain_idx: usize) {
        let members = self.chains[chain_idx].clone();
        for &m in &members {
            self.visiting[m] = true;
        }
        let head = members[0];
        let tail = *members.last().unwrap();
        let head_node = self.nodes[head];
        let tail_node = self.nodes[tail];
        let head_flags = if self.horizontal { parse_constraint_phys_flags(head_node) } else { 0 };
        let tail_flags = if self.horizontal { parse_constraint_phys_flags(tail_node) } else { 0 };
        let chain_phys = self.horizontal
            && ((head_flags & 2) != 0
                || (tail_flags & 1) != 0
                || ((head_flags & 1) != 0 && (tail_flags & 2) != 0));
        let rtl_h = self.horizontal && self.state.rtl && !chain_phys;
        let phys_chain_bias = self.horizontal
            && (head_flags & 4) != 0
            && (head_flags & 1) != 0
            && (tail_flags & 2) != 0;
        let style = Self::parse_chain_style(head_node, self.horizontal);
        let raw_bias = Self::rule(
            head_node,
            if self.horizontal { RULE_CONSTRAINT_HORIZONTAL_BIAS } else { RULE_CONSTRAINT_VERTICAL_BIAS },
        )
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .map_or(0.5, |v| v.clamp(0.0, 1.0));
        let bias = if self.horizontal && self.state.rtl && !phys_chain_bias {
            1.0 - raw_bias
        } else {
            raw_bias
        };

        let (phys_members, low_outer_info, high_outer_info) = if self.horizontal {
            if !rtl_h {
                let low_phys = (head_flags & 1) != 0;
                let high_phys = (tail_flags & 2) != 0;
                let low = Self::rule(head_node, RULE_CONSTRAINT_START_TO_START)
                    .and_then(|a| self.resolve_edge_info_phys(a, false, low_phys))
                    .or_else(|| Self::rule(head_node, RULE_CONSTRAINT_START_TO_END).and_then(|a| self.resolve_edge_info_phys(a, true, low_phys)));
                let high = Self::rule(tail_node, RULE_CONSTRAINT_END_TO_END)
                    .and_then(|a| self.resolve_edge_info_phys(a, true, high_phys))
                    .or_else(|| Self::rule(tail_node, RULE_CONSTRAINT_END_TO_START).and_then(|a| self.resolve_edge_info_phys(a, false, high_phys)));
                (members.clone(), low, high)
            } else {
                let low = Self::rule(tail_node, RULE_CONSTRAINT_END_TO_END)
                    .and_then(|a| self.resolve_edge_info(a, false))
                    .or_else(|| Self::rule(tail_node, RULE_CONSTRAINT_END_TO_START).and_then(|a| self.resolve_edge_info(a, true)));
                let high = Self::rule(head_node, RULE_CONSTRAINT_START_TO_START)
                    .and_then(|a| self.resolve_edge_info(a, true))
                    .or_else(|| Self::rule(head_node, RULE_CONSTRAINT_START_TO_END).and_then(|a| self.resolve_edge_info(a, false)));
                let mut rev = members.clone();
                rev.reverse();
                (rev, low, high)
            }
        } else {
            let low = Self::rule(head_node, RULE_CONSTRAINT_TOP_TO_TOP)
                .and_then(|a| self.resolve_edge_info(a, false))
                .or_else(|| Self::rule(head_node, RULE_CONSTRAINT_TOP_TO_BOTTOM).and_then(|a| self.resolve_edge_info(a, true)));
            let high = Self::rule(tail_node, RULE_CONSTRAINT_BOTTOM_TO_BOTTOM)
                .and_then(|a| self.resolve_edge_info(a, true))
                .or_else(|| Self::rule(tail_node, RULE_CONSTRAINT_BOTTOM_TO_TOP).and_then(|a| self.resolve_edge_info(a, false)));
            (members.clone(), low, high)
        };
        let low_outer = low_outer_info.map(|(v, _)| v);
        let high_outer = high_outer_info.map(|(v, _)| v);
        let low_outer_gone = low_outer_info.is_some_and(|(_, g)| g);
        let high_outer_gone = high_outer_info.is_some_and(|(_, g)| g);

        let active: Vec<usize> = phys_members
            .iter()
            .copied()
            .filter(|&i| self.visibilities[i] != Visibility::Gone)
            .collect();

        if active.is_empty() {
            let anchor = low_outer.or(high_outer).unwrap_or(self.origin);
            for &m in &members {
                self.visiting[m] = false;
                self.result[m] = Some((anchor, 0.0));
            }
            return;
        }

        let span = match (low_outer, high_outer) {
            (Some(l), Some(h)) => Some((h - l).max(0.0)),
            _ => None,
        };
        let avail = span.unwrap_or(self.extent);

        let mut margins = Vec::with_capacity(active.len());
        let mut sizes = Vec::with_capacity(active.len());
        let mut weights: Vec<Option<f32>> = Vec::with_capacity(active.len());
        let mut is_match_constraint = Vec::with_capacity(active.len());

        for &idx in &active {
            let n = self.nodes[idx];
            let n_flags = if self.horizontal { parse_constraint_phys_flags(n) } else { 0 };
            let n_phys_margins = (n_flags & 4) != 0 && (n_flags & 1) != 0 && (n_flags & 2) != 0;
            let m = constraint_resolved_margin(n, self.state, n_phys_margins);
            let gm = parse_gone_margins_phys(n, self.state, n_phys_margins);
            let (mut before, mut after) = if self.horizontal { (m[0], m[2]) } else { (m[1], m[3]) };
            let (g_before, g_after) = if self.horizontal { (gm[0], gm[2]) } else { (gm[1], gm[3]) };
            let phys_i = phys_members.iter().position(|&m_idx| m_idx == idx).unwrap_or(0);
            let prev_gone = if phys_i == 0 {
                low_outer_gone
            } else {
                self.visibilities[phys_members[phys_i - 1]] == Visibility::Gone
            };
            let next_gone = if phys_i + 1 == phys_members.len() {
                high_outer_gone
            } else {
                self.visibilities[phys_members[phys_i + 1]] == Visibility::Gone
            };
            if prev_gone && let Some(v) = g_before { before = v; }
            if next_gone && let Some(v) = g_after { after = v; }
            margins.push((before, after));
            let spec = if self.horizontal { n.width } else { n.height };
            let default_size = Self::rule(n, RULE_CONSTRAINT_DEFAULT_SIZE)
                .map(parse_constraint_default_size)
                .map(|(w, h)| if self.horizontal { w } else { h });
            let is_zero = matches!(spec, Size::Dp(v) if v <= 0.0);
            let mc = (is_zero && matches!(default_size, None | Some(ConstraintDefaultSize::Spread)))
                || (!self.wrap_parent && matches!(spec, Size::Match));
            is_match_constraint.push(mc);
            weights.push(Self::parse_chain_weight(n, self.horizontal));
            let sz = if mc {
                0.0
            } else if is_zero {
                let sb = Self::rule(n, RULE_CONSTRAINT_SIZE_BOUNDS)
                    .map(parse_constraint_size_bounds)
                    .unwrap_or_default();
                let min_px = (if self.horizontal { sb.w_min_dp } else { sb.h_min_dp }).map(|v| self.state.dp(v));
                let max_px = (if self.horizontal { sb.w_max_dp } else { sb.h_max_dp }).map(|v| self.state.dp(v));
                let raw = match default_size {
                    Some(ConstraintDefaultSize::Wrap) => {
                        let span_avail = (avail - before - after).max(0.0);
                        self.measure_child_wrap(idx, span_avail).min(span_avail)
                    }
                    Some(ConstraintDefaultSize::Percent) => {
                        let pct = Self::rule(
                            n,
                            if self.horizontal { RULE_CONSTRAINT_WIDTH_PERCENT } else { RULE_CONSTRAINT_HEIGHT_PERCENT },
                        )
                        .and_then(|s| s.trim().parse::<f32>().ok())
                        .filter(|v| v.is_finite())
                        .map_or(1.0, |v| v.clamp(0.0, 1.0));
                        if self.wrap_parent {
                            0.0
                        } else {
                            (self.extent * pct).max(0.0)
                        }
                    }
                    _ => 0.0,
                };
                apply_constraint_size_bounds(raw, min_px, max_px)
            } else {
                match spec {
                    Size::Dp(v) => self.state.dp(v),
                    Size::Wrap | Size::Match => self.measure_child(idx, (avail - before - after).max(0.0)),
                }
            };
            sizes.push(sz);
        }

        let mut resolved_weights = vec![0.0_f32; active.len()];
        let mut total_weight = 0.0_f32;
        if span.is_some() {
            for i in 0..active.len() {
                if is_match_constraint[i] {
                    let w = weights[i].unwrap_or(1.0);
                    resolved_weights[i] = w;
                    total_weight += w;
                }
            }
        }

        let sum_margins: f32 = margins.iter().map(|(b, a)| b + a).sum();
        if let Some(total_span) = span && total_weight > 0.0 {
            let fixed_total: f32 = sizes.iter().sum::<f32>() + sum_margins;
            let distributable = (total_span - fixed_total).max(0.0);
            for i in 0..active.len() {
                if resolved_weights[i] > 0.0 {
                    let mut sz = distributable * (resolved_weights[i] / total_weight);
                    if let Some(sb) = Self::rule(self.nodes[active[i]], RULE_CONSTRAINT_SIZE_BOUNDS)
                        .map(parse_constraint_size_bounds)
                    {
                        let min_px = (if self.horizontal { sb.w_min_dp } else { sb.h_min_dp }).map(|v| self.state.dp(v));
                        let max_px = (if self.horizontal { sb.w_max_dp } else { sb.h_max_dp }).map(|v| self.state.dp(v));
                        sz = apply_constraint_size_bounds(sz, min_px, max_px);
                    }
                    sizes[i] = sz;
                }
            }
        }

        let used: f32 = sizes.iter().sum::<f32>() + sum_margins;
        let mut positions = vec![0.0_f32; active.len()];

        match (low_outer, high_outer) {
            (Some(l), Some(h)) => {
                let total_span = (h - l).max(0.0);
                if total_weight > 0.0 {
                    let mut cursor = l;
                    for i in 0..active.len() {
                        positions[i] = cursor + margins[i].0;
                        cursor = positions[i] + sizes[i] + margins[i].1;
                    }
                } else {
                    let extra = (total_span - used).max(0.0);
                    match style {
                        ConstraintChainStyle::Spread => {
                            let gap = extra / (active.len() as f32 + 1.0);
                            let mut cursor = l + gap;
                            for i in 0..active.len() {
                                positions[i] = cursor + margins[i].0;
                                cursor = positions[i] + sizes[i] + margins[i].1 + gap;
                            }
                        }
                        ConstraintChainStyle::SpreadInside => {
                            if active.len() == 1 {
                                positions[0] = l + margins[0].0 + extra * bias;
                            } else {
                                let gap = extra / (active.len() as f32 - 1.0);
                                let mut cursor = l;
                                for i in 0..active.len() {
                                    positions[i] = cursor + margins[i].0;
                                    cursor = positions[i] + sizes[i] + margins[i].1 + gap;
                                }
                            }
                        }
                        ConstraintChainStyle::Packed => {
                            let mut cursor = l + extra * bias;
                            for i in 0..active.len() {
                                positions[i] = cursor + margins[i].0;
                                cursor = positions[i] + sizes[i] + margins[i].1;
                            }
                        }
                    }
                }
            }
            (Some(l), None) => {
                let mut cursor = l;
                for i in 0..active.len() {
                    positions[i] = cursor + margins[i].0;
                    cursor = positions[i] + sizes[i] + margins[i].1;
                }
            }
            (None, Some(h)) => {
                let mut cursor = h - used;
                for i in 0..active.len() {
                    positions[i] = cursor + margins[i].0;
                    cursor = positions[i] + sizes[i] + margins[i].1;
                }
            }
            (None, None) => {
                let mut cursor = if rtl_h { self.origin + self.extent - used } else { self.origin };
                for i in 0..active.len() {
                    positions[i] = cursor + margins[i].0;
                    cursor = positions[i] + sizes[i] + margins[i].1;
                }
            }
        }

        let mut last_pos = low_outer.unwrap_or(self.origin);
        let mut active_pos = 0usize;
        for &m in &phys_members {
            self.visiting[m] = false;
            if active_pos < active.len() && active[active_pos] == m {
                let res = (positions[active_pos], sizes[active_pos]);
                self.result[m] = Some(res);
                last_pos = res.0 + res.1;
                active_pos += 1;
            } else {
                self.result[m] = Some((last_pos, 0.0));
            }
        }
    }

    pub(super) fn position(&mut self, index: usize) -> (f32, f32) {
        if let Some(val) = self.result[index] {
            return val;
        }
        let n = self.nodes[index];
        if let Some(res) = self.guideline_position(n) {
            self.result[index] = Some(res);
            return res;
        }
        if is_constraint_barrier(&n)
            && let Some(res) = self.barrier_position(index, n)
        {
            self.result[index] = Some(res);
            return res;
        }
        if is_constraint_group(&n) || is_constraint_layer(&n) {
            let res = (self.origin, 0.0);
            self.result[index] = Some(res);
            return res;
        }
        if let Some(chain_idx) = self.chain_of[index] {
            self.resolve_chain(chain_idx);
            return self.result[index].unwrap_or((self.origin, 0.0));
        }
        if self.visiting[index] {
            return (self.origin, 0.0);
        }
        self.visiting[index] = true;
        let is_gone = self.visibilities[index] == Visibility::Gone;
        let phys_flags = if self.horizontal { parse_constraint_phys_flags(n) } else { 0 };
        let phys_left = (phys_flags & 1) != 0;
        let phys_right = (phys_flags & 2) != 0;
        let phys_bias_margins = (phys_flags & 4) != 0 && phys_left && phys_right;
        let m = if is_gone { [0.0; 4] } else { constraint_resolved_margin(n, self.state, phys_bias_margins) };
        let (gone_m, gone_baseline_px) = if is_gone {
            ([None; 4], None)
        } else {
            parse_gone_margins_full(n, self.state, phys_bias_margins)
        };
        let (before, after) = if self.horizontal { (m[0], m[2]) } else { (m[1], m[3]) };
        let (gone_before, gone_after) = if self.horizontal { (gone_m[0], gone_m[2]) } else { (gone_m[1], gone_m[3]) };

        let apply_low = |edge: f32, target_gone: bool| {
            edge + if target_gone { gone_before.unwrap_or(before) } else { before }
        };
        let apply_high = |edge: f32, target_gone: bool| {
            edge - if target_gone { gone_after.unwrap_or(after) } else { after }
        };

        let (low_edge, high_edge, bias) = if self.horizontal {
            let raw_bias = Self::rule(n, RULE_CONSTRAINT_HORIZONTAL_BIAS)
                .and_then(|s| s.trim().parse::<f32>().ok())
                .filter(|v| v.is_finite())
                .map_or(0.5, |v| v.clamp(0.0, 1.0));
            if !self.state.rtl {
                let low = Self::rule(n, RULE_CONSTRAINT_START_TO_START)
                    .and_then(|a| self.resolve_edge_info_phys(a, false, phys_left))
                    .or_else(|| Self::rule(n, RULE_CONSTRAINT_START_TO_END).and_then(|a| self.resolve_edge_info_phys(a, true, phys_left)))
                    .map(|(v, g)| apply_low(v, g));
                let high = Self::rule(n, RULE_CONSTRAINT_END_TO_END)
                    .and_then(|a| self.resolve_edge_info_phys(a, true, phys_right))
                    .or_else(|| Self::rule(n, RULE_CONSTRAINT_END_TO_START).and_then(|a| self.resolve_edge_info_phys(a, false, phys_right)))
                    .map(|(v, g)| apply_high(v, g));
                (low, high, raw_bias)
            } else {
                let phys_low = if phys_left {
                    Self::rule(n, RULE_CONSTRAINT_START_TO_START)
                        .and_then(|a| self.resolve_edge_info_phys(a, false, true))
                        .or_else(|| Self::rule(n, RULE_CONSTRAINT_START_TO_END).and_then(|a| self.resolve_edge_info_phys(a, true, true)))
                } else {
                    None
                };
                let log_low = if !phys_right {
                    Self::rule(n, RULE_CONSTRAINT_END_TO_END)
                        .and_then(|a| self.resolve_edge_info_phys(a, false, false))
                        .or_else(|| Self::rule(n, RULE_CONSTRAINT_END_TO_START).and_then(|a| self.resolve_edge_info_phys(a, true, false)))
                } else {
                    None
                };
                let low = phys_low.or(log_low).map(|(v, g)| apply_low(v, g));

                let phys_high = if phys_right {
                    Self::rule(n, RULE_CONSTRAINT_END_TO_END)
                        .and_then(|a| self.resolve_edge_info_phys(a, true, true))
                        .or_else(|| Self::rule(n, RULE_CONSTRAINT_END_TO_START).and_then(|a| self.resolve_edge_info_phys(a, false, true)))
                } else {
                    None
                };
                let log_high = if !phys_left {
                    Self::rule(n, RULE_CONSTRAINT_START_TO_START)
                        .and_then(|a| self.resolve_edge_info_phys(a, true, false))
                        .or_else(|| Self::rule(n, RULE_CONSTRAINT_START_TO_END).and_then(|a| self.resolve_edge_info_phys(a, false, false)))
                } else {
                    None
                };
                let high = phys_high.or(log_high).map(|(v, g)| apply_high(v, g));

                let eff_bias = if phys_bias_margins { raw_bias } else { 1.0 - raw_bias };
                (low, high, eff_bias)
            }
        } else {
            let raw_bias = Self::rule(n, RULE_CONSTRAINT_VERTICAL_BIAS)
                .and_then(|s| s.trim().parse::<f32>().ok())
                .filter(|v| v.is_finite())
                .map_or(0.5, |v| v.clamp(0.0, 1.0));
            let low = Self::rule(n, RULE_CONSTRAINT_TOP_TO_TOP)
                .and_then(|a| self.resolve_edge_info(a, false))
                .or_else(|| Self::rule(n, RULE_CONSTRAINT_TOP_TO_BOTTOM).and_then(|a| self.resolve_edge_info(a, true)))
                .map(|(v, g)| apply_low(v, g));
            let high = Self::rule(n, RULE_CONSTRAINT_BOTTOM_TO_BOTTOM)
                .and_then(|a| self.resolve_edge_info(a, true))
                .or_else(|| Self::rule(n, RULE_CONSTRAINT_BOTTOM_TO_TOP).and_then(|a| self.resolve_edge_info(a, false)))
                .map(|(v, g)| apply_high(v, g));
            (low, high, raw_bias)
        };

        let baseline_target = if !self.horizontal && !is_gone {
            Self::rule(n, RULE_CONSTRAINT_BASELINE_TO_BASELINE).and_then(|anchor| {
                let &target_idx = self.ids.get(anchor)?;
                if target_idx == index
                    || self.visiting[target_idx]
                    || is_constraint_helper(&self.nodes[target_idx])
                {
                    return None;
                }
                let target_gone = self.visibilities[target_idx] == Visibility::Gone;
                let (target_pos, target_height) = self.position(target_idx);
                if target_gone {
                    return Some(target_pos + gone_baseline_px.unwrap_or(0.0));
                }
                let target_width = self.other_axis.map_or(0.0, |w| w[target_idx].1);
                self.nodes[target_idx]
                    .relative_baseline(
                        target_width,
                        target_height,
                        self.state,
                        self.metrics,
                        child_view_id(self.parent, target_idx, &self.nodes[target_idx]),
                    )
                    .map(|offset| target_pos + offset)
            })
        } else {
            None
        };

        let available = (high_edge.unwrap_or(self.origin + self.extent - after)
            - low_edge.unwrap_or(self.origin + before))
            .max(0.0);
        let size_bounds = Self::rule(n, RULE_CONSTRAINT_SIZE_BOUNDS)
            .map(parse_constraint_size_bounds)
            .unwrap_or_default();
        let constrained_axis = if self.horizontal {
            size_bounds.constrained_w
        } else {
            size_bounds.constrained_h
        };
        let axis_min_px = (if self.horizontal {
            size_bounds.w_min_dp
        } else {
            size_bounds.h_min_dp
        })
        .map(|v| self.state.dp(v));
        let axis_max_px = (if self.horizontal {
            size_bounds.w_max_dp
        } else {
            size_bounds.h_max_dp
        })
        .map(|v| self.state.dp(v));
        let spec = if self.horizontal { n.width } else { n.height };
        let size = if is_gone {
            0.0
        } else {
            match spec {
                Size::Dp(v) if v <= 0.0 => {
                    let default_size = Self::rule(n, RULE_CONSTRAINT_DEFAULT_SIZE)
                        .map(parse_constraint_default_size)
                        .map(|(w, h)| if self.horizontal { w } else { h });
                    let ratio_size = resolved_ratio_target(n).and_then(|(target, ratio)| {
                        let matches_axis = (self.horizontal && target == RatioTarget::Width)
                            || (!self.horizontal && target == RatioTarget::Height);
                        if !matches_axis {
                            return None;
                        }
                        let other = self.other_axis?[index].1;
                        if other <= 0.0 {
                            return None;
                        }
                        Some(if self.horizontal { other * ratio } else { other / ratio })
                    });
                    let raw_sz = if let Some(rs) = ratio_size {
                        rs
                    } else if matches!(default_size, Some(ConstraintDefaultSize::Wrap)) {
                        let span = match (low_edge, high_edge) {
                            (Some(l), Some(h)) => (h - l).max(0.0),
                            _ => available,
                        };
                        let measured = if let Some(t_idx) = self.ph_to_target[index] {
                            let t_node = self.nodes[t_idx];
                            let t_spec = if self.horizontal { t_node.width } else { t_node.height };
                            match t_spec {
                                Size::Dp(tv) if tv > 0.0 => self.state.dp(tv),
                                _ => self.measure_child_wrap(t_idx, span),
                            }
                        } else {
                            self.measure_child_wrap(index, span)
                        };
                        measured.min(span)
                    } else if matches!(default_size, Some(ConstraintDefaultSize::Percent)) {
                        let pct = Self::rule(
                            n,
                            if self.horizontal { RULE_CONSTRAINT_WIDTH_PERCENT } else { RULE_CONSTRAINT_HEIGHT_PERCENT },
                        )
                        .and_then(|s| s.trim().parse::<f32>().ok())
                        .filter(|v| v.is_finite())
                        .map_or(1.0, |v| v.clamp(0.0, 1.0));
                        if self.wrap_parent {
                            if let (Some(l), Some(h)) = (low_edge, high_edge) {
                                ((h - l).max(0.0) * pct).max(0.0)
                            } else {
                                0.0
                            }
                        } else {
                            (self.extent * pct).max(0.0)
                        }
                    } else if default_size.is_none()
                        && let Some(pct) = Self::rule(
                            n,
                            if self.horizontal { RULE_CONSTRAINT_WIDTH_PERCENT } else { RULE_CONSTRAINT_HEIGHT_PERCENT },
                        )
                        .and_then(|s| s.trim().parse::<f32>().ok())
                        .filter(|v| v.is_finite())
                    {
                        let pct = pct.clamp(0.0, 1.0);
                        if self.wrap_parent && (low_edge.is_none() || high_edge.is_none()) {
                            0.0
                        } else {
                            (self.extent * pct).max(0.0)
                        }
                    } else if let (Some(l), Some(h)) = (low_edge, high_edge) {
                        (h - l).max(0.0)
                    } else {
                        0.0
                    };
                    apply_constraint_size_bounds(raw_sz, axis_min_px, axis_max_px)
                }
                Size::Dp(v) => self.state.dp(v),
                Size::Wrap => {
                    let mut sz = if let Some(t_idx) = self.ph_to_target[index] {
                        let t_node = self.nodes[t_idx];
                        let t_spec = if self.horizontal { t_node.width } else { t_node.height };
                        match t_spec {
                            Size::Dp(v) if v > 0.0 => self.state.dp(v),
                            _ => self.measure_child(t_idx, available),
                        }
                    } else {
                        self.measure_child(index, available)
                    };
                    if constrained_axis
                        && let (Some(l), Some(h)) = (low_edge, high_edge)
                    {
                        let span = (h - l).max(0.0);
                        sz = sz.min(span);
                        sz = apply_constraint_size_bounds(sz, axis_min_px, axis_max_px);
                    }
                    sz
                }
                Size::Match => {
                    if self.wrap_parent {
                        self.measure_child(index, available)
                    } else {
                        available
                    }
                }
            }
        };

        let pos = if let Some(target_baseline) = baseline_target {
            let own_width = self.other_axis.map_or(0.0, |w| w[index].1);
            if let Some(own_baseline) = n.relative_baseline(
                own_width,
                size,
                self.state,
                self.metrics,
                child_view_id(self.parent, index, &n),
            ) {
                target_baseline - own_baseline
            } else {
                match (low_edge, high_edge) {
                    (Some(l), Some(h)) => l + ((h - l) - size) * bias,
                    (Some(l), None) => l,
                    (None, Some(h)) => h - size,
                    (None, None) => self.origin + before,
                }
            }
        } else {
            match (low_edge, high_edge) {
                (Some(l), Some(h)) => l + ((h - l) - size) * bias,
                (Some(l), None) => l,
                (None, Some(h)) => h - size,
                (None, None) => {
                    if self.horizontal && self.state.rtl {
                        self.origin + self.extent - after - size
                    } else {
                        self.origin + before
                    }
                }
            }
        };

        let res = (pos, size);
        self.visiting[index] = false;
        self.result[index] = Some(res);
        res
    }
}

pub(super) fn detect_constraint_chains(
    nodes: &[Node],
    ids: &HashMap<&'static str, usize>,
    horizontal: bool,
) -> (Vec<Vec<usize>>, Vec<Option<usize>>) {
    let n = nodes.len();
    let (next_rule, prev_rule) = if horizontal {
        (RULE_CONSTRAINT_END_TO_START, RULE_CONSTRAINT_START_TO_END)
    } else {
        (RULE_CONSTRAINT_BOTTOM_TO_TOP, RULE_CONSTRAINT_TOP_TO_BOTTOM)
    };
    let mut next_link = vec![None; n];
    let mut prev_link = vec![None; n];
    for i in 0..n {
        if nodes[i].id.is_empty() || is_constraint_helper(&nodes[i]) {
            continue;
        }
        if let Some(target_id) = ConstraintAxis::rule(nodes[i], next_rule)
            && let Some(&j) = ids.get(target_id)
            && j != i
            && !is_constraint_helper(&nodes[j])
            && ConstraintAxis::rule(nodes[j], prev_rule) == Some(nodes[i].id)
            && (!horizontal
                || ((parse_constraint_phys_flags(nodes[i]) & 2) != 0)
                    == ((parse_constraint_phys_flags(nodes[j]) & 1) != 0))
            && next_link[i].is_none()
            && prev_link[j].is_none()
        {
            next_link[i] = Some(j);
            prev_link[j] = Some(i);
        }
    }
    let mut chains = Vec::new();
    let mut chain_of = vec![None; n];
    for head in 0..n {
        if prev_link[head].is_none() && next_link[head].is_some() {
            let chain_idx = chains.len();
            let mut members = Vec::new();
            let mut cur = Some(head);
            let mut seen = vec![false; n];
            while let Some(idx) = cur {
                if seen[idx] {
                    break;
                }
                seen[idx] = true;
                members.push(idx);
                cur = next_link[idx];
            }
            if members.len() >= 2 {
                for &m in &members {
                    chain_of[m] = Some(chain_idx);
                }
                chains.push(members);
            }
        }
    }
    (chains, chain_of)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FlowWrapMode {
    None,
    Chain,
    Aligned,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FlowAlign {
    Start,
    Center,
    End,
    Baseline,
}

pub(super) struct FlowSpec<'a> {
    pub(super) horizontal: bool,
    pub(super) wrap: FlowWrapMode,
    pub(super) h_gap_dp: f32,
    pub(super) v_gap_dp: f32,
    pub(super) max_wrap: usize,
    pub(super) refs: Vec<&'a str>,
    pub(super) h_style: Option<ConstraintChainStyle>,
    pub(super) v_style: Option<ConstraintChainStyle>,
    pub(super) h_bias: f32,
    pub(super) v_bias: f32,
    pub(super) h_align: Option<FlowAlign>,
    pub(super) v_align: Option<FlowAlign>,
}

pub(super) fn parse_flow_chain_style(raw: &str) -> Option<ConstraintChainStyle> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "spread" | "0" => Some(ConstraintChainStyle::Spread),
        "spread_inside" | "spreadinside" | "1" => Some(ConstraintChainStyle::SpreadInside),
        "packed" | "2" => Some(ConstraintChainStyle::Packed),
        _ => None,
    }
}

pub(super) fn parse_flow_align(raw: &str) -> Option<FlowAlign> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "start" | "top" | "left" | "0" => Some(FlowAlign::Start),
        "center" | "middle" | "1" => Some(FlowAlign::Center),
        "end" | "bottom" | "right" | "2" => Some(FlowAlign::End),
        "baseline" | "3" => Some(FlowAlign::Baseline),
        _ => None,
    }
}

pub(super) fn parse_flow_spec(anchor: &str) -> Option<FlowSpec<'_>> {
    let segs: Vec<&str> = anchor.split('|').collect();
    if segs.len() < 4 {
        return None;
    }
    let orient_raw = segs[0].trim().to_ascii_lowercase();
    let horizontal = !matches!(orient_raw.as_str(), "1" | "vertical" | "v");
    let wrap_raw = segs[1].trim().to_ascii_lowercase();
    let wrap = match wrap_raw.as_str() {
        "chain" | "1" => FlowWrapMode::Chain,
        "aligned" | "2" => FlowWrapMode::Aligned,
        _ => FlowWrapMode::None,
    };
    let parse_gap = |s: &str| {
        s.trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(0.0)
    };
    let (h_gap_dp, v_gap_dp, mut max_wrap, refs_raw, ext_seg) = if segs.len() >= 6 && !segs[2].contains(',') {
        let hg = parse_gap(segs[2]);
        let vg = parse_gap(segs[3]);
        let mw = segs[4].trim().parse::<usize>().unwrap_or(0);
        let refs_s = segs[5];
        let ext = segs.get(6).copied();
        (hg, vg, mw, refs_s, ext)
    } else {
        let (hg_s, vg_s) = segs[2].split_once(',').unwrap_or((segs[2], "0"));
        let hg = parse_gap(hg_s);
        let vg = parse_gap(vg_s);
        let refs_s = segs[3];
        let ext = segs.get(4).copied();
        (hg, vg, 0usize, refs_s, ext)
    };
    let refs: Vec<&str> = refs_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let mut h_style = None;
    let mut v_style = None;
    let mut h_bias = 0.5_f32;
    let mut v_bias = 0.5_f32;
    let mut h_align = None;
    let mut v_align = None;
    if let Some(ext) = ext_seg {
        let mut p = ext.split(',');
        if let Some(s) = p.next() {
            h_style = parse_flow_chain_style(s);
        }
        if let Some(s) = p.next() {
            v_style = parse_flow_chain_style(s);
        }
        if let Some(s) = p.next()
            && let Ok(v) = s.trim().parse::<f32>()
            && v.is_finite()
        {
            h_bias = v.clamp(0.0, 1.0);
        }
        if let Some(s) = p.next()
            && let Ok(v) = s.trim().parse::<f32>()
            && v.is_finite()
        {
            v_bias = v.clamp(0.0, 1.0);
        }
        if let Some(s) = p.next() {
            h_align = parse_flow_align(s);
        }
        if let Some(s) = p.next() {
            v_align = parse_flow_align(s);
        }
        if let Some(s) = p.next()
            && let Ok(mw) = s.trim().parse::<usize>()
            && mw > 0
        {
            max_wrap = mw;
        }
    }
    Some(FlowSpec {
        horizontal,
        wrap,
        h_gap_dp,
        v_gap_dp,
        max_wrap,
        refs,
        h_style,
        v_style,
        h_bias,
        v_bias,
        h_align,
        v_align,
    })
}

#[derive(Clone, Copy, Default)]
pub(super) struct FlowExtSpec {
    pub(super) first_h_style: Option<ConstraintChainStyle>,
    pub(super) last_h_style: Option<ConstraintChainStyle>,
    pub(super) first_h_bias: Option<f32>,
    pub(super) last_h_bias: Option<f32>,
    pub(super) first_v_style: Option<ConstraintChainStyle>,
    pub(super) last_v_style: Option<ConstraintChainStyle>,
    pub(super) first_v_bias: Option<f32>,
    pub(super) last_v_bias: Option<f32>,
}

pub(super) fn parse_flow_ext_spec(anchor: &str) -> FlowExtSpec {
    let mut parts = anchor.split(',');
    let parse_style = |s: Option<&str>| s.and_then(parse_flow_chain_style);
    let parse_bias = |s: Option<&str>| {
        s.and_then(|raw| raw.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 1.0))
    };
    FlowExtSpec {
        first_h_style: parse_style(parts.next()),
        last_h_style: parse_style(parts.next()),
        first_h_bias: parse_bias(parts.next()),
        last_h_bias: parse_bias(parts.next()),
        first_v_style: parse_style(parts.next()),
        last_v_style: parse_style(parts.next()),
        first_v_bias: parse_bias(parts.next()),
        last_v_bias: parse_bias(parts.next()),
    }
}

pub(super) fn distribute_flow_line(
    sizes: &[f32],
    gap: f32,
    span: f32,
    style: Option<ConstraintChainStyle>,
    raw_bias: f32,
    rtl_reverse: bool,
) -> Vec<f32> {
    let n = sizes.len();
    if n == 0 {
        return Vec::new();
    }
    let sum_items: f32 = sizes.iter().sum();
    let used_with_gaps = sum_items + n.saturating_sub(1) as f32 * gap;
    let extra_no_gaps = (span - sum_items).max(0.0);
    let extra_after_gaps = (span - used_with_gaps).max(0.0);
    let mut out = Vec::with_capacity(n);
    match style {
        None => {
            if rtl_reverse {
                let mut cursor = span;
                for &sz in sizes {
                    out.push(cursor - sz);
                    cursor -= sz + gap;
                }
            } else {
                let mut cursor = 0.0_f32;
                for &sz in sizes {
                    out.push(cursor);
                    cursor += sz + gap;
                }
            }
        }
        Some(ConstraintChainStyle::Packed) => {
            if rtl_reverse {
                let mut cursor = span - extra_after_gaps * raw_bias;
                for &sz in sizes {
                    out.push(cursor - sz);
                    cursor -= sz + gap;
                }
            } else {
                let mut cursor = extra_after_gaps * raw_bias;
                for &sz in sizes {
                    out.push(cursor);
                    cursor += sz + gap;
                }
            }
        }
        Some(ConstraintChainStyle::Spread) => {
            let ideal = extra_no_gaps / (n as f32 + 1.0);
            let (outer, inner_gap) = if ideal >= gap {
                (ideal, ideal)
            } else {
                (extra_after_gaps * 0.5, gap)
            };
            if rtl_reverse {
                let mut cursor = span - outer;
                for &sz in sizes {
                    out.push(cursor - sz);
                    cursor -= sz + inner_gap;
                }
            } else {
                let mut cursor = outer;
                for &sz in sizes {
                    out.push(cursor);
                    cursor += sz + inner_gap;
                }
            }
        }
        Some(ConstraintChainStyle::SpreadInside) => {
            if n == 1 {
                let pos = if rtl_reverse {
                    span - sizes[0] - extra_no_gaps * raw_bias
                } else {
                    extra_no_gaps * raw_bias
                };
                out.push(pos);
            } else {
                let inner_gap = (extra_no_gaps / (n - 1) as f32).max(gap);
                if rtl_reverse {
                    let mut cursor = span;
                    for &sz in sizes {
                        out.push(cursor - sz);
                        cursor -= sz + inner_gap;
                    }
                } else {
                    let mut cursor = 0.0_f32;
                    for &sz in sizes {
                        out.push(cursor);
                        cursor += sz + inner_gap;
                    }
                }
            }
        }
    }
    out
}

pub(super) fn flow_cross_offset(cell: f32, item: f32, align: FlowAlign, rtl_horizontal: bool) -> f32 {
    match align {
        FlowAlign::Start => if rtl_horizontal { cell - item } else { 0.0 },
        FlowAlign::Center | FlowAlign::Baseline => (cell - item) * 0.5,
        FlowAlign::End => if rtl_horizontal { 0.0 } else { cell - item },
    }
}

#[derive(Clone, Copy)]
pub(super) struct ConstraintCircleSpec<'a> {
    pub(super) target: &'a str,
    pub(super) radius_dp: f32,
    pub(super) angle_deg: f32,
}

pub(super) fn parse_constraint_circle(anchor: &str) -> Option<ConstraintCircleSpec<'_>> {
    let mut parts = anchor.split(',');
    let target = parts.next()?.trim();
    if target.is_empty() {
        return None;
    }
    let radius_dp = parts.next()?.trim().parse::<f32>().ok().filter(|v| v.is_finite())?;
    let angle_deg = parts.next()?.trim().parse::<f32>().ok().filter(|v| v.is_finite())?;
    Some(ConstraintCircleSpec { target, radius_dp, angle_deg })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn constraint_layout_axis(
    nodes: &[Node],
    parent: u64,
    bounds: Rect,
    state: &LayoutState,
    visibilities: &[Visibility],
    ph_to_target: &[Option<usize>],
    metrics: TextMetrics,
    horizontal: bool,
    wrap_parent: bool,
    other_axis: Option<&[(f32, f32)]>,
    preset: Option<&[Option<(f32, f32)>]>,
) -> Vec<(f32, f32)> {
    let ids: HashMap<&'static str, usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !n.id.is_empty())
        .map(|(i, n)| (n.id, i))
        .collect();
    let (chains, chain_of) = detect_constraint_chains(nodes, &ids, horizontal);
    let result = preset.map_or_else(|| vec![None; nodes.len()], |p| p.to_vec());
    let mut axis = ConstraintAxis {
        nodes,
        ids,
        state,
        visibilities,
        ph_to_target,
        parent,
        origin: if horizontal { bounds.x } else { bounds.y },
        extent: if horizontal { bounds.width } else { bounds.height },
        horizontal,
        wrap_parent,
        other_axis,
        metrics,
        chains,
        chain_of,
        result,
        visiting: vec![false; nodes.len()],
    };
    for i in 0..nodes.len() {
        axis.position(i);
    }
    axis.result.into_iter().map(|x| x.unwrap_or((axis.origin, 0.0))).collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_flow_helpers(
    nodes: &[Node],
    ids: &HashMap<&'static str, usize>,
    parent: u64,
    bounds: Rect,
    state: &LayoutState,
    visibilities: &[Visibility],
    metrics: TextMetrics,
    wrap_width: bool,
    wrap_height: bool,
    widths: &mut [(f32, f32)],
    heights: &mut [(f32, f32)],
    preset_x: &mut [Option<(f32, f32)>],
    preset_y: &mut [Option<(f32, f32)>],
) -> bool {
    let mut changed = false;
    let resolve_axis_edge = |anchor: &str, horizontal: bool, target_high: bool, wrap_parent: bool, physical: bool, axis: &[(f32, f32)]| -> Option<(f32, bool)> {
        let origin = if horizontal { bounds.x } else { bounds.y };
        let extent = if horizontal { bounds.width } else { bounds.height };
        if anchor == "parent" {
            let rtl_h = horizontal && state.rtl && !physical;
            if wrap_parent && ((!rtl_h && target_high) || (rtl_h && !target_high)) {
                return None;
            }
            return Some((if target_high { origin + extent } else { origin }, false));
        }
        let &idx = ids.get(anchor)?;
        let (pos, size) = axis[idx];
        let is_helper = is_constraint_helper(&nodes[idx]);
        let target_gone = !is_helper && visibilities[idx] == Visibility::Gone;
        let eff_size = if target_gone || is_zero_size_constraint_helper(&nodes[idx]) { 0.0 } else { size };
        Some((if target_high { pos + eff_size } else { pos }, target_gone))
    };
    let row_vertical_metrics = |row: &[(usize, f32, f32)], valign: FlowAlign| -> (f32, Vec<f32>) {
        if valign == FlowAlign::Baseline {
            let baselines: Vec<Option<f32>> = row
                .iter()
                .map(|&(idx, iw, ih)| {
                    nodes[idx].relative_baseline(iw, ih, state, metrics, child_view_id(parent, idx, &nodes[idx]))
                })
                .collect();
            if baselines.iter().any(Option::is_some) {
                let mut max_ascent = 0.0_f32;
                let mut max_descent = 0.0_f32;
                let mut max_h = 0.0_f32;
                for (k, &(_, _, ih)) in row.iter().enumerate() {
                    max_h = max_h.max(ih);
                    if let Some(bl) = baselines[k] {
                        max_ascent = max_ascent.max(bl);
                        max_descent = max_descent.max((ih - bl).max(0.0));
                    }
                }
                let row_h = max_h.max(max_ascent + max_descent);
                let offsets: Vec<f32> = row
                    .iter()
                    .enumerate()
                    .map(|(k, &(_, _, ih))| {
                        if let Some(bl) = baselines[k] {
                            (max_ascent - bl).max(0.0)
                        } else {
                            (row_h - ih) * 0.5
                        }
                    })
                    .collect();
                return (row_h, offsets);
            }
        }
        let row_h = row.iter().map(|&(_, _, h)| h).fold(0.0_f32, f32::max);
        let offsets = row
            .iter()
            .map(|&(_, _, ih)| flow_cross_offset(row_h, ih, valign, false))
            .collect();
        (row_h, offsets)
    };

    for (flow_idx, flow_node) in nodes.iter().enumerate() {
        let Some(raw) = ConstraintAxis::rule(*flow_node, RULE_FLOW) else {
            continue;
        };
        let Some(spec) = parse_flow_spec(raw) else {
            continue;
        };
        let ext = ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_FLOW_EXT)
            .map(parse_flow_ext_spec)
            .unwrap_or_default();
        let resolve_line_h = |line_idx: usize, line_count: usize| -> (Option<ConstraintChainStyle>, f32) {
            if line_count <= 1 || line_idx == 0 {
                (
                    ext.first_h_style.or(spec.h_style),
                    ext.first_h_bias.unwrap_or(spec.h_bias),
                )
            } else if line_idx + 1 == line_count {
                (
                    ext.last_h_style.or(spec.h_style),
                    ext.last_h_bias.unwrap_or(spec.h_bias),
                )
            } else {
                (spec.h_style, spec.h_bias)
            }
        };
        let resolve_line_v = |line_idx: usize, line_count: usize| -> (Option<ConstraintChainStyle>, f32) {
            if line_count <= 1 || line_idx == 0 {
                (
                    ext.first_v_style.or(spec.v_style),
                    ext.first_v_bias.unwrap_or(spec.v_bias),
                )
            } else if line_idx + 1 == line_count {
                (
                    ext.last_v_style.or(spec.v_style),
                    ext.last_v_bias.unwrap_or(spec.v_bias),
                )
            } else {
                (spec.v_style, spec.v_bias)
            }
        };
        let flow_phys_flags = parse_constraint_phys_flags(*flow_node);
        let flow_phys_left = (flow_phys_flags & 1) != 0;
        let flow_phys_right = (flow_phys_flags & 2) != 0;
        let flow_phys_bias_margins = (flow_phys_flags & 4) != 0 && flow_phys_left && flow_phys_right;
        let m = constraint_resolved_margin(*flow_node, state, flow_phys_bias_margins);
        let gm = parse_gone_margins_phys(*flow_node, state, flow_phys_bias_margins);
        let apply_low = |edge: f32, gone: bool, before: f32, g_before: Option<f32>| {
            edge + if gone { g_before.unwrap_or(before) } else { before }
        };
        let apply_high = |edge: f32, gone: bool, after: f32, g_after: Option<f32>| {
            edge - if gone { g_after.unwrap_or(after) } else { after }
        };

        let raw_h_bias = ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_HORIZONTAL_BIAS)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map_or(0.5, |v| v.clamp(0.0, 1.0));
        let (low_x, high_x, bias_x) = if !state.rtl {
            let low = ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_START_TO_START)
                .and_then(|a| resolve_axis_edge(a, true, false, wrap_width, flow_phys_left, widths))
                .or_else(|| ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_START_TO_END).and_then(|a| resolve_axis_edge(a, true, true, wrap_width, flow_phys_left, widths)))
                .map(|(v, g)| apply_low(v, g, m[0], gm[0]));
            let high = ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_END_TO_END)
                .and_then(|a| resolve_axis_edge(a, true, true, wrap_width, flow_phys_right, widths))
                .or_else(|| ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_END_TO_START).and_then(|a| resolve_axis_edge(a, true, false, wrap_width, flow_phys_right, widths)))
                .map(|(v, g)| apply_high(v, g, m[2], gm[2]));
            (low, high, raw_h_bias)
        } else {
            let phys_low = if flow_phys_left {
                ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_START_TO_START)
                    .and_then(|a| resolve_axis_edge(a, true, false, wrap_width, true, widths))
                    .or_else(|| ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_START_TO_END).and_then(|a| resolve_axis_edge(a, true, true, wrap_width, true, widths)))
            } else {
                None
            };
            let log_low = if !flow_phys_right {
                ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_END_TO_END)
                    .and_then(|a| resolve_axis_edge(a, true, false, wrap_width, false, widths))
                    .or_else(|| ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_END_TO_START).and_then(|a| resolve_axis_edge(a, true, true, wrap_width, false, widths)))
            } else {
                None
            };
            let low = phys_low.or(log_low).map(|(v, g)| apply_low(v, g, m[0], gm[0]));

            let phys_high = if flow_phys_right {
                ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_END_TO_END)
                    .and_then(|a| resolve_axis_edge(a, true, true, wrap_width, true, widths))
                    .or_else(|| ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_END_TO_START).and_then(|a| resolve_axis_edge(a, true, false, wrap_width, true, widths)))
            } else {
                None
            };
            let log_high = if !flow_phys_left {
                ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_START_TO_START)
                    .and_then(|a| resolve_axis_edge(a, true, true, wrap_width, false, widths))
                    .or_else(|| ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_START_TO_END).and_then(|a| resolve_axis_edge(a, true, false, wrap_width, false, widths)))
            } else {
                None
            };
            let high = phys_high.or(log_high).map(|(v, g)| apply_high(v, g, m[2], gm[2]));
            let eff_bias = if flow_phys_bias_margins { raw_h_bias } else { 1.0 - raw_h_bias };
            (low, high, eff_bias)
        };
        let bias_y = ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_VERTICAL_BIAS)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map_or(0.5, |v| v.clamp(0.0, 1.0));
        let low_y = ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_TOP_TO_TOP)
            .and_then(|a| resolve_axis_edge(a, false, false, wrap_height, false, heights))
            .or_else(|| ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_TOP_TO_BOTTOM).and_then(|a| resolve_axis_edge(a, false, true, wrap_height, false, heights)))
            .map(|(v, g)| apply_low(v, g, m[1], gm[1]));
        let high_y = ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_BOTTOM_TO_BOTTOM)
            .and_then(|a| resolve_axis_edge(a, false, true, wrap_height, false, heights))
            .or_else(|| ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_BOTTOM_TO_TOP).and_then(|a| resolve_axis_edge(a, false, false, wrap_height, false, heights)))
            .map(|(v, g)| apply_high(v, g, m[3], gm[3]));

        let avail_w = (high_x.unwrap_or(bounds.x + bounds.width - m[2]) - low_x.unwrap_or(bounds.x + m[0])).max(0.0);
        let avail_h = (high_y.unwrap_or(bounds.y + bounds.height - m[3]) - low_y.unwrap_or(bounds.y + m[1])).max(0.0);

        let explicit_box_w = match flow_node.width {
            Size::Dp(v) if v > 0.0 => Some(state.dp(v)),
            Size::Dp(_) => ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_WIDTH_PERCENT)
                .and_then(|s| s.trim().parse::<f32>().ok())
                .filter(|v| v.is_finite())
                .map(|p| (avail_w * p.clamp(0.0, 1.0)).clamp(0.0, avail_w))
                .or_else(|| match (low_x, high_x) {
                    (Some(l), Some(h)) => Some((h - l).max(0.0)),
                    _ => None,
                }),
            Size::Match if !wrap_width => Some(avail_w),
            _ => None,
        };
        let explicit_box_h = match flow_node.height {
            Size::Dp(v) if v > 0.0 => Some(state.dp(v)),
            Size::Dp(_) => ConstraintAxis::rule(*flow_node, RULE_CONSTRAINT_HEIGHT_PERCENT)
                .and_then(|s| s.trim().parse::<f32>().ok())
                .filter(|v| v.is_finite())
                .map(|p| (avail_h * p.clamp(0.0, 1.0)).clamp(0.0, avail_h))
                .or_else(|| match (low_y, high_y) {
                    (Some(l), Some(h)) => Some((h - l).max(0.0)),
                    _ => None,
                }),
            Size::Match if !wrap_height => Some(avail_h),
            _ => None,
        };
        let wrap_limit_w = explicit_box_w.unwrap_or(avail_w);
        let wrap_limit_h = explicit_box_h.unwrap_or(avail_h);

        let mut seen = vec![false; nodes.len()];
        let mut items: Vec<(usize, f32, f32)> = Vec::new();
        for ref_id in &spec.refs {
            if let Some(&idx) = ids.get(*ref_id)
                && idx != flow_idx
                && !seen[idx]
                && !is_constraint_helper(&nodes[idx])
                && visibilities[idx] != Visibility::Gone
            {
                seen[idx] = true;
                let item = &nodes[idx];
                let cid = child_view_id(parent, idx, item);
                let iw = match item.width {
                    Size::Dp(v) if v > 0.0 => state.dp(v),
                    Size::Match => wrap_limit_w,
                    _ => item.layout_intrinsic(true, state, wrap_limit_w, metrics, cid),
                };
                let ih = match item.height {
                    Size::Dp(v) if v > 0.0 => state.dp(v),
                    Size::Match => wrap_limit_h,
                    _ => item.layout_intrinsic(false, state, iw, metrics, cid),
                };
                items.push((idx, iw, ih));
            }
        }

        let h_gap = state.dp(spec.h_gap_dp);
        let v_gap = state.dp(spec.v_gap_dp);
        let mut placed: Vec<(usize, f32, f32, f32, f32)> = Vec::with_capacity(items.len());
        let (content_w, content_h) = if items.is_empty() {
            (0.0_f32, 0.0_f32)
        } else if spec.horizontal {
            let valign = spec.v_align.unwrap_or(FlowAlign::Center);
            match spec.wrap {
                FlowWrapMode::None => {
                    let (row_h, y_offs) = row_vertical_metrics(&items, valign);
                    let ws: Vec<f32> = items.iter().map(|&(_, w, _)| w).collect();
                    let total_w = ws.iter().sum::<f32>() + items.len().saturating_sub(1) as f32 * h_gap;
                    let box_w = explicit_box_w.unwrap_or(total_w);
                    let box_h = explicit_box_h.unwrap_or(row_h);
                    let (hs, hb) = resolve_line_h(0, 1);
                    let xs = distribute_flow_line(&ws, h_gap, box_w, hs, hb, state.rtl);
                    let row_y = distribute_flow_line(&[row_h], v_gap, box_h, spec.v_style, spec.v_bias, false)[0];
                    for (k, &(idx, iw, ih)) in items.iter().enumerate() {
                        placed.push((idx, xs[k], row_y + y_offs[k], iw, ih));
                    }
                    (total_w, row_h)
                }
                FlowWrapMode::Chain => {
                    let mut rows: Vec<(Vec<(usize, f32, f32)>, f32)> = Vec::new();
                    let mut cur: Vec<(usize, f32, f32)> = Vec::new();
                    let mut cur_w = 0.0_f32;
                    for item in items {
                        let (idx, iw, ih) = item;
                        let exceed_count = spec.max_wrap > 0 && cur.len() >= spec.max_wrap;
                        let exceed_width = wrap_limit_w > 0.0 && cur_w + h_gap + iw > wrap_limit_w + 0.01;
                        if !cur.is_empty() && (exceed_count || exceed_width) {
                            rows.push((std::mem::take(&mut cur), cur_w));
                            cur_w = 0.0;
                        }
                        cur_w = if cur.is_empty() { iw } else { cur_w + h_gap + iw };
                        cur.push((idx, iw, ih));
                    }
                    if !cur.is_empty() {
                        rows.push((cur, cur_w));
                    }
                    let row_info: Vec<(f32, Vec<f32>)> = rows
                        .iter()
                        .map(|(row, _)| row_vertical_metrics(row, valign))
                        .collect();
                    let row_hs: Vec<f32> = row_info.iter().map(|(h, _)| *h).collect();
                    let max_row_w = rows.iter().map(|(_, w)| *w).fold(0.0_f32, f32::max);
                    let total_h = row_hs.iter().sum::<f32>() + rows.len().saturating_sub(1) as f32 * v_gap;
                    let box_w = explicit_box_w.unwrap_or(max_row_w);
                    let box_h = explicit_box_h.unwrap_or(total_h);
                    let row_ys = distribute_flow_line(&row_hs, v_gap, box_h, spec.v_style, spec.v_bias, false);
                    let row_count = rows.len();
                    for (r, ((row, _), (_, y_offs))) in rows.into_iter().zip(row_info).enumerate() {
                        let ws: Vec<f32> = row.iter().map(|&(_, w, _)| w).collect();
                        let (hs, hb) = resolve_line_h(r, row_count);
                        let xs = distribute_flow_line(&ws, h_gap, box_w, hs, hb, state.rtl);
                        for (k, (idx, iw, ih)) in row.into_iter().enumerate() {
                            placed.push((idx, xs[k], row_ys[r] + y_offs[k], iw, ih));
                        }
                    }
                    (max_row_w, total_h)
                }
                FlowWrapMode::Aligned => {
                    let halign = spec.h_align.unwrap_or(FlowAlign::Start);
                    let max_cols = if spec.max_wrap > 0 { spec.max_wrap.min(items.len()) } else { items.len() }.max(1);
                    let mut chosen_cols = 1usize;
                    let mut col_widths = vec![0.0_f32; 1];
                    for c in (1..=max_cols).rev() {
                        let mut cw = vec![0.0_f32; c];
                        for (k, &(_, iw, _)) in items.iter().enumerate() {
                            cw[k % c] = cw[k % c].max(iw);
                        }
                        let grid_w = cw.iter().sum::<f32>() + c.saturating_sub(1) as f32 * h_gap;
                        if c == 1 || wrap_limit_w <= 0.0 || grid_w <= wrap_limit_w + 0.01 {
                            chosen_cols = c;
                            col_widths = cw;
                            break;
                        }
                    }
                    let row_count = items.len().div_ceil(chosen_cols);
                    let mut row_info: Vec<(f32, Vec<f32>)> = Vec::with_capacity(row_count);
                    for r in 0..row_count {
                        let start = r * chosen_cols;
                        let end = (start + chosen_cols).min(items.len());
                        row_info.push(row_vertical_metrics(&items[start..end], valign));
                    }
                    let row_heights: Vec<f32> = row_info.iter().map(|(h, _)| *h).collect();
                    let total_w = col_widths.iter().sum::<f32>() + chosen_cols.saturating_sub(1) as f32 * h_gap;
                    let total_h = row_heights.iter().sum::<f32>() + row_count.saturating_sub(1) as f32 * v_gap;
                    let box_w = explicit_box_w.unwrap_or(total_w);
                    let box_h = explicit_box_h.unwrap_or(total_h);
                    let (hs, hb) = if row_count <= 1 { resolve_line_h(0, 1) } else { (spec.h_style, spec.h_bias) };
                    let col_xs = distribute_flow_line(&col_widths, h_gap, box_w, hs, hb, state.rtl);
                    let row_ys = distribute_flow_line(&row_heights, v_gap, box_h, spec.v_style, spec.v_bias, false);
                    for (k, &(idx, iw, ih)) in items.iter().enumerate() {
                        let r = k / chosen_cols;
                        let c = k % chosen_cols;
                        let rx = col_xs[c] + flow_cross_offset(col_widths[c], iw, halign, state.rtl);
                        let ry = row_ys[r] + row_info[r].1[c];
                        placed.push((idx, rx, ry, iw, ih));
                    }
                    (total_w, total_h)
                }
            }
        } else {
            // Vertical orientation (columns)
            match spec.wrap {
                FlowWrapMode::None => {
                    let halign = spec.h_align.unwrap_or(FlowAlign::Center);
                    let col_w = items.iter().map(|&(_, w, _)| w).fold(0.0_f32, f32::max);
                    let hs: Vec<f32> = items.iter().map(|&(_, _, h)| h).collect();
                    let total_h = hs.iter().sum::<f32>() + items.len().saturating_sub(1) as f32 * v_gap;
                    let box_w = explicit_box_w.unwrap_or(col_w);
                    let box_h = explicit_box_h.unwrap_or(total_h);
                    let col_x = distribute_flow_line(&[col_w], h_gap, box_w, spec.h_style, spec.h_bias, state.rtl)[0];
                    let (vs, vb) = resolve_line_v(0, 1);
                    let ys = distribute_flow_line(&hs, v_gap, box_h, vs, vb, false);
                    for (k, &(idx, iw, ih)) in items.iter().enumerate() {
                        let rx = col_x + flow_cross_offset(col_w, iw, halign, state.rtl);
                        placed.push((idx, rx, ys[k], iw, ih));
                    }
                    (col_w, total_h)
                }
                FlowWrapMode::Chain => {
                    let halign = spec.h_align.unwrap_or(FlowAlign::Center);
                    let mut cols: Vec<(Vec<(usize, f32, f32)>, f32, f32)> = Vec::new();
                    let mut cur: Vec<(usize, f32, f32)> = Vec::new();
                    let mut cur_w = 0.0_f32;
                    let mut cur_h = 0.0_f32;
                    for item in items {
                        let (idx, iw, ih) = item;
                        let exceed_count = spec.max_wrap > 0 && cur.len() >= spec.max_wrap;
                        let exceed_height = wrap_limit_h > 0.0 && cur_h + v_gap + ih > wrap_limit_h + 0.01;
                        if !cur.is_empty() && (exceed_count || exceed_height) {
                            cols.push((std::mem::take(&mut cur), cur_w, cur_h));
                            cur_w = 0.0;
                            cur_h = 0.0;
                        }
                        cur_h = if cur.is_empty() { ih } else { cur_h + v_gap + ih };
                        cur_w = cur_w.max(iw);
                        cur.push((idx, iw, ih));
                    }
                    if !cur.is_empty() {
                        cols.push((cur, cur_w, cur_h));
                    }
                    let col_ws: Vec<f32> = cols.iter().map(|(_, w, _)| *w).collect();
                    let total_w = col_ws.iter().sum::<f32>() + cols.len().saturating_sub(1) as f32 * h_gap;
                    let max_col_h = cols.iter().map(|(_, _, h)| *h).fold(0.0_f32, f32::max);
                    let box_w = explicit_box_w.unwrap_or(total_w);
                    let box_h = explicit_box_h.unwrap_or(max_col_h);
                    let col_xs = distribute_flow_line(&col_ws, h_gap, box_w, spec.h_style, spec.h_bias, state.rtl);
                    let col_count = cols.len();
                    for (c, (col, col_w, _)) in cols.into_iter().enumerate() {
                        let hs: Vec<f32> = col.iter().map(|&(_, _, h)| h).collect();
                        let (vs, vb) = resolve_line_v(c, col_count);
                        let ys = distribute_flow_line(&hs, v_gap, box_h, vs, vb, false);
                        for (k, (idx, iw, ih)) in col.into_iter().enumerate() {
                            let rx = col_xs[c] + flow_cross_offset(col_w, iw, halign, state.rtl);
                            placed.push((idx, rx, ys[k], iw, ih));
                        }
                    }
                    (total_w, max_col_h)
                }
                FlowWrapMode::Aligned => {
                    let halign = spec.h_align.unwrap_or(FlowAlign::Start);
                    let valign = spec.v_align.unwrap_or(FlowAlign::Start);
                    let max_rows = if spec.max_wrap > 0 { spec.max_wrap.min(items.len()) } else { items.len() }.max(1);
                    let mut chosen_rows = 1usize;
                    let mut row_heights = vec![0.0_f32; 1];
                    for r in (1..=max_rows).rev() {
                        let mut rh = vec![0.0_f32; r];
                        for (k, &(_, _, ih)) in items.iter().enumerate() {
                            rh[k % r] = rh[k % r].max(ih);
                        }
                        let grid_h = rh.iter().sum::<f32>() + r.saturating_sub(1) as f32 * v_gap;
                        if r == 1 || wrap_limit_h <= 0.0 || grid_h <= wrap_limit_h + 0.01 {
                            chosen_rows = r;
                            row_heights = rh;
                            break;
                        }
                    }
                    let col_count = items.len().div_ceil(chosen_rows);
                    let mut col_widths = vec![0.0_f32; col_count];
                    for (k, &(_, iw, _)) in items.iter().enumerate() {
                        let c = k / chosen_rows;
                        col_widths[c] = col_widths[c].max(iw);
                    }
                    let total_w = col_widths.iter().sum::<f32>() + col_count.saturating_sub(1) as f32 * h_gap;
                    let total_h = row_heights.iter().sum::<f32>() + chosen_rows.saturating_sub(1) as f32 * v_gap;
                    let box_w = explicit_box_w.unwrap_or(total_w);
                    let box_h = explicit_box_h.unwrap_or(total_h);
                    let col_xs = distribute_flow_line(&col_widths, h_gap, box_w, spec.h_style, spec.h_bias, state.rtl);
                    let (vs, vb) = if col_count <= 1 { resolve_line_v(0, 1) } else { (spec.v_style, spec.v_bias) };
                    let row_ys = distribute_flow_line(&row_heights, v_gap, box_h, vs, vb, false);
                    for (k, &(idx, iw, ih)) in items.iter().enumerate() {
                        let c = k / chosen_rows;
                        let r = k % chosen_rows;
                        let rx = col_xs[c] + flow_cross_offset(col_widths[c], iw, halign, state.rtl);
                        let ry = row_ys[r] + flow_cross_offset(row_heights[r], ih, valign, false);
                        placed.push((idx, rx, ry, iw, ih));
                    }
                    (total_w, total_h)
                }
            }
        };

        let box_w = explicit_box_w.unwrap_or(content_w);
        let box_h = explicit_box_h.unwrap_or(content_h);
        let flow_x = match (low_x, high_x) {
            (Some(l), Some(h)) => l + ((h - l) - box_w) * bias_x,
            (Some(l), None) => l,
            (None, Some(h)) => h - box_w,
            (None, None) => {
                if state.rtl { bounds.x + bounds.width - m[2] - box_w } else { bounds.x + m[0] }
            }
        };
        let flow_y = match (low_y, high_y) {
            (Some(l), Some(h)) => l + ((h - l) - box_h) * bias_y,
            (Some(l), None) => l,
            (None, Some(h)) => h - box_h,
            (None, None) => bounds.y + m[1],
        };
        widths[flow_idx] = (flow_x, box_w);
        heights[flow_idx] = (flow_y, box_h);
        preset_x[flow_idx] = Some((flow_x, box_w));
        preset_y[flow_idx] = Some((flow_y, box_h));
        for (idx, rx, ry, iw, ih) in placed {
            widths[idx] = (flow_x + rx, iw);
            heights[idx] = (flow_y + ry, ih);
            preset_x[idx] = Some((flow_x + rx, iw));
            preset_y[idx] = Some((flow_y + ry, ih));
        }
        changed = true;
    }
    changed
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_circle_constraints(
    nodes: &[Node],
    ids: &HashMap<&'static str, usize>,
    bounds: Rect,
    state: &LayoutState,
    visibilities: &[Visibility],
    widths: &mut [(f32, f32)],
    heights: &mut [(f32, f32)],
    preset_x: &mut [Option<(f32, f32)>],
    preset_y: &mut [Option<(f32, f32)>],
) -> bool {
    let mut resolved = vec![false; nodes.len()];
    let mut visiting = vec![false; nodes.len()];
    let mut any = false;

    pub(super) fn solve(
        idx: usize,
        nodes: &[Node],
        ids: &HashMap<&'static str, usize>,
        bounds: Rect,
        state: &LayoutState,
        visibilities: &[Visibility],
        widths: &mut [(f32, f32)],
        heights: &mut [(f32, f32)],
        preset_x: &mut [Option<(f32, f32)>],
        preset_y: &mut [Option<(f32, f32)>],
        resolved: &mut [bool],
        visiting: &mut [bool],
        any: &mut bool,
    ) {
        if resolved[idx] || visiting[idx] {
            return;
        }
        let n = nodes[idx];
        if visibilities[idx] == Visibility::Gone || is_constraint_helper(&n) {
            resolved[idx] = true;
            return;
        }
        let Some(raw) = ConstraintAxis::rule(n, RULE_CONSTRAINT_CIRCLE) else {
            resolved[idx] = true;
            return;
        };
        let Some(spec) = parse_constraint_circle(raw) else {
            resolved[idx] = true;
            return;
        };
        visiting[idx] = true;
        let target_center = if spec.target == "parent" {
            Some((bounds.x + bounds.width * 0.5, bounds.y + bounds.height * 0.5))
        } else if let Some(&t_idx) = ids.get(spec.target) {
            if t_idx != idx && !visiting[t_idx] {
                solve(t_idx, nodes, ids, bounds, state, visibilities, widths, heights, preset_x, preset_y, resolved, visiting, any);
                let (tx, tw) = widths[t_idx];
                let (ty, th) = heights[t_idx];
                let is_zero = is_zero_size_constraint_helper(&nodes[t_idx]) || visibilities[t_idx] == Visibility::Gone;
                let cx = if is_zero { tx } else { tx + tw * 0.5 };
                let cy = if is_zero { ty } else { ty + th * 0.5 };
                Some((cx, cy))
            } else {
                None
            }
        } else {
            None
        };
        if let Some((tcx, tcy)) = target_center {
            let r = state.dp(spec.radius_dp);
            let rad = spec.angle_deg.to_radians();
            let cx = tcx + r * rad.sin();
            let cy = tcy - r * rad.cos();
            let w = widths[idx].1;
            let h = heights[idx].1;
            widths[idx] = (cx - w * 0.5, w);
            heights[idx] = (cy - h * 0.5, h);
            preset_x[idx] = Some(widths[idx]);
            preset_y[idx] = Some(heights[idx]);
            *any = true;
        }
        visiting[idx] = false;
        resolved[idx] = true;
    }

    for i in 0..nodes.len() {
        solve(i, nodes, ids, bounds, state, visibilities, widths, heights, preset_x, preset_y, &mut resolved, &mut visiting, &mut any);
    }
    any
}

pub(super) fn constraint_layout_positions(
    node: &Node,
    parent: u64,
    bounds: Rect,
    state: &LayoutState,
    metrics: TextMetrics,
    wrap_width: bool,
    wrap_height: bool,
) -> Vec<Rect> {
    let (ph_to_target, _) = resolve_constraint_placeholders(node.children, state);
    let visibilities = constraint_group_visibilities(node.children, state);
    let mut widths = constraint_layout_axis(node.children, parent, bounds, state, &visibilities, &ph_to_target, metrics, true, wrap_width, None, None);
    let mut heights = constraint_layout_axis(node.children, parent, bounds, state, &visibilities, &ph_to_target, metrics, false, wrap_height, Some(&widths), None);
    let has_width_ratio = node.children.iter().enumerate().any(|(i, n)| {
        visibilities[i] != Visibility::Gone
            && !is_constraint_helper(n)
            && matches!(resolved_ratio_target(*n), Some((RatioTarget::Width, _)))
    });
    if has_width_ratio {
        let updated_widths = constraint_layout_axis(node.children, parent, bounds, state, &visibilities, &ph_to_target, metrics, true, wrap_width, Some(&heights), None);
        let changed = widths.iter().zip(&updated_widths).any(|(a, b)| (a.0 - b.0).abs() > 0.01 || (a.1 - b.1).abs() > 0.01);
        widths = updated_widths;
        if changed {
            heights = constraint_layout_axis(node.children, parent, bounds, state, &visibilities, &ph_to_target, metrics, false, wrap_height, Some(&widths), None);
        }
    }
    let has_flow = node.children.iter().any(is_constraint_flow);
    let has_circle = node.children.iter().any(|n| ConstraintAxis::rule(*n, RULE_CONSTRAINT_CIRCLE).is_some());
    if has_flow || has_circle {
        let ids: HashMap<&'static str, usize> = node
            .children
            .iter()
            .enumerate()
            .filter(|(_, n)| !n.id.is_empty())
            .map(|(i, n)| (n.id, i))
            .collect();
        let mut preset_x: Vec<Option<(f32, f32)>> = vec![None; node.children.len()];
        let mut preset_y: Vec<Option<(f32, f32)>> = vec![None; node.children.len()];
        let mut needs_resolve = false;
        if has_flow {
            needs_resolve |= resolve_flow_helpers(
                node.children, &ids, parent, bounds, state, &visibilities, metrics,
                wrap_width, wrap_height, &mut widths, &mut heights, &mut preset_x, &mut preset_y,
            );
        }
        if has_circle {
            needs_resolve |= resolve_circle_constraints(
                node.children, &ids, bounds, state, &visibilities,
                &mut widths, &mut heights, &mut preset_x, &mut preset_y,
            );
        }
        if needs_resolve {
            widths = constraint_layout_axis(node.children, parent, bounds, state, &visibilities, &ph_to_target, metrics, true, wrap_width, Some(&heights), Some(&preset_x));
            heights = constraint_layout_axis(node.children, parent, bounds, state, &visibilities, &ph_to_target, metrics, false, wrap_height, Some(&widths), Some(&preset_y));
        }
    }
    widths.into_iter().zip(heights).map(|((x, width), (y, height))| Rect { x, y, width, height }).collect()
}

