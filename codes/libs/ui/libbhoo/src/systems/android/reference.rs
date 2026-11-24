//! Static Android resource trees with explicit custom-view adapters.
//! This is a view/layout bridge, not an Android runtime or general ConstraintLayout solver.
use crate::*;
use std::collections::{HashMap, HashSet};
pub struct NativeNode {
    pub kind: &'static str,
    pub id: &'static str,
    pub attrs: &'static [(&'static str, &'static str)],
    pub children: &'static [NativeNode],
}
#[derive(Default)]
pub struct NativeState {
    pub text: HashMap<String, String>,
    pub hidden: HashSet<String>,
    pub slots: HashMap<String, Rect>,
}
fn hash(s: &str, seed: u64) -> u64 {
    s.bytes().fold(seed + 1469598103934665603, |h, b| {
        (h ^ b as u64).wrapping_mul(1099511628211)
    }) & ((1 << 52) - 1)
}
impl NativeNode {
    pub fn attr(&self, key: &str) -> &str {
        self.attrs
            .iter()
            .find(|(k, _)| *k == key)
            .map_or("", |(_, v)| v)
    }
    fn n(&self, key: &str, default: f32) -> f32 {
        self.attr(key)
            .trim_end_matches("dp")
            .trim_end_matches("sp")
            .trim_end_matches("px")
            .parse()
            .unwrap_or(default)
    }
    fn gone(&self, s: &NativeState) -> bool {
        self.attr("visibility") == "gone" || s.hidden.contains(self.id)
    }
    fn content<'a>(&'a self, s: &'a NativeState) -> &'a str {
        s.text
            .get(self.id)
            .map_or(self.attr("text"), String::as_str)
    }
    fn edges(&self, prefix: &str) -> [f32; 4] {
        let all = self.n(prefix, 0.);
        let horizontal = self.n(&format!("{prefix}Horizontal"), all);
        let vertical = self.n(&format!("{prefix}Vertical"), all);
        [
            self.n(
                &format!("{prefix}Start"),
                self.n(&format!("{prefix}Left"), horizontal),
            ),
            self.n(&format!("{prefix}Top"), vertical),
            self.n(
                &format!("{prefix}End"),
                self.n(&format!("{prefix}Right"), horizontal),
            ),
            self.n(&format!("{prefix}Bottom"), vertical),
        ]
    }
    fn measure(&self, axis: usize, available: f32, f: &Frame<'_>, s: &NativeState) -> f32 {
        if self.gone(s) {
            return 0.;
        }
        let key = if axis == 0 {
            "layout_width"
        } else {
            "layout_height"
        };
        let v = self.attr(key);
        if matches!(v, "match_parent" | "fill_parent") {
            return available;
        }
        if let Ok(n) = v
            .trim_end_matches("dp")
            .trim_end_matches("sp")
            .parse::<f32>()
        {
            return n.max(0.);
        }
        let padding = self.edges("padding");
        let extra = padding[axis] + padding[axis + 2];
        let intrinsic = match self.kind {
            "Text" => {
                if axis == 0 {
                    f.text_width(
                        self.content(s),
                        TextStyle {
                            size: self.n("textSize", 16.),
                            ..TextStyle::default()
                        },
                    )
                } else {
                    self.n("textSize", 16.) * 1.45
                }
            }
            "Avatar" | "Icon" => 48.,
            "Slot" => self.n(if axis == 0 { "minWidth" } else { "minHeight" }, 0.),
            _ => {
                let sum = self.kind == "Linear"
                    && ((self.attr("orientation") == "horizontal") == (axis == 0));
                let mut value: f32 = 0.;
                for c in self.children {
                    let m = c.edges("layout_margin");
                    let size = c.measure(axis, available, f, s) + m[axis] + m[axis + 2];
                    value = if sum { value + size } else { value.max(size) }
                }
                value
            }
        };
        (intrinsic + extra)
            .max(self.n(if axis == 0 { "minWidth" } else { "minHeight" }, 0.))
            .min(available)
    }
    pub fn render(
        &'static self,
        f: &mut Frame<'_>,
        b: Rect,
        s: &mut NativeState,
        instance: u64,
    ) -> bool {
        self.draw(f, b, s, instance)
    }
    fn draw(&'static self, f: &mut Frame<'_>, b: Rect, s: &mut NativeState, instance: u64) -> bool {
        if self.gone(s) || self.attr("visibility") == "invisible" || b.width <= 0. || b.height <= 0.
        {
            return false;
        }
        let id = hash(self.id, instance);
        s.slots.insert(self.id.into(), b);
        let theme = f.theme();
        let mut clicked = false;
        if let Some(color) = color(self.attr("background")) {
            f.paint_rect(b, self.n("cornerRadius", 0.), color)
        }
        if self.attr("clickable") == "true" {
            let r = f.target(id, b, self.content(s), true, Role::Button);
            f.state_layer(r, 12., theme.primary);
            clicked = r.clicked;
        }
        let p = self.edges("padding");
        let inner = Rect {
            x: b.x + p[0],
            y: b.y + p[1],
            width: (b.width - p[0] - p[2]).max(0.),
            height: (b.height - p[1] - p[3]).max(0.),
        };
        match self.kind {
            "Slot" => {}
            "Avatar" => {
                f.paint_rect(
                    inner,
                    inner.width.min(inner.height) / 2.,
                    color(self.attr("tint")).unwrap_or(theme.secondary),
                );
                let name = self.content(s);
                let initials = name
                    .split_whitespace()
                    .filter_map(|w| w.chars().next())
                    .take(2)
                    .collect::<String>()
                    .to_uppercase();
                f.styled_text(
                    inner,
                    &initials,
                    theme.on_secondary,
                    TextStyle {
                        size: 18.,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
            }
            "Text" => {
                let text = self.content(s);
                let style = TextStyle {
                    size: self.n("textSize", 16.),
                    weight: if self.attr("textStyle") == "bold" {
                        700
                    } else {
                        400
                    },
                    align: if self.attr("gravity").contains("center_horizontal")
                        || self.attr("gravity") == "center"
                    {
                        1
                    } else if self.attr("gravity").contains("end") {
                        2
                    } else {
                        0
                    },
                    ..TextStyle::default()
                };
                let ink = color(self.attr("textColor")).unwrap_or(theme.text);
                if self.attr("maxLines") == "1"
                    || self.attr("singleLine") == "true"
                    || inner.height < style.size * 2.
                {
                    let shortened;
                    let text = if f.text_width(text, style) > inner.width {
                        let chars: Vec<char> = text.chars().collect();
                        let mut lo = 0;
                        let mut hi = chars.len();
                        while lo < hi {
                            let mid = (lo + hi).div_ceil(2);
                            let candidate = chars[..mid].iter().collect::<String>() + "…";
                            if f.text_width(&candidate, style) <= inner.width {
                                lo = mid;
                            } else {
                                hi = mid - 1;
                            }
                        }
                        shortened = chars[..lo].iter().collect::<String>() + "…";
                        shortened.as_str()
                    } else {
                        text
                    };
                    f.styled_text(inner, text, ink, style);
                } else {
                    f.paragraph(inner, text, ink, style, style.size * 1.45);
                }
            }
            "Icon" => {
                let icon = match self.attr("icon") {
                    "search" => MaterialIcon::Search,
                    "add" => MaterialIcon::Add,
                    "back" => MaterialIcon::ArrowBack,
                    "send" => MaterialIcon::ArrowForward,
                    "more" => MaterialIcon::MoreVert,
                    "check" => MaterialIcon::Check,
                    _ => MaterialIcon::Menu,
                };
                f.paint_material_icon(
                    Rect {
                        x: inner.x + (inner.width - 24.) / 2.,
                        y: inner.y + (inner.height - 24.) / 2.,
                        width: 24.,
                        height: 24.,
                    },
                    icon,
                    color(self.attr("textColor")).unwrap_or(theme.text),
                );
            }
            "Picture" => {
                let shades = [
                    Color(71, 121, 72, 255),
                    Color(47, 111, 91, 255),
                    Color(96, 133, 60, 255),
                ];
                let ink = shades[(instance as usize) % 3];
                f.paint_rect(inner, 12., ink.opacity(0.14));
                let stem = Rect {
                    x: inner.x + inner.width / 2. - 2.,
                    y: inner.y + inner.height * 0.3,
                    width: 4.,
                    height: inner.height * 0.55,
                };
                f.paint_rect(stem, 2., ink);
                for j in 0..4 {
                    let left = j % 2 == 0;
                    let w = inner.width * 0.22;
                    f.paint_rect(
                        Rect {
                            x: stem.x + if left { -w } else { 0. },
                            y: stem.y + 8. + j as f32 * inner.height * 0.09,
                            width: w,
                            height: inner.height * 0.18,
                        },
                        inner.height * 0.09,
                        ink,
                    );
                }
            }
            "Linear" => {
                let horiz = self.attr("orientation") == "horizontal";
                let axis = usize::from(!horiz);
                let avail = if horiz { inner.width } else { inner.height };
                let mut fixed = 0.;
                let mut weights = 0.;
                for c in self.children {
                    if c.gone(s) {
                        continue;
                    }
                    let m = c.edges("layout_margin");
                    let weight = c.n("layout_weight", 0.);
                    weights += weight;
                    fixed += if weight > 0. {
                        m[axis] + m[axis + 2]
                    } else {
                        c.measure(axis, avail, f, s) + m[axis] + m[axis + 2]
                    };
                }
                let mut cursor = if horiz { inner.x } else { inner.y };
                let gravity = self.attr("gravity");
                if weights == 0.
                    && ((!horiz && gravity.contains("center_vertical")) || gravity == "center")
                {
                    cursor += (avail - fixed).max(0.) / 2.;
                }
                for (i, c) in self.children.iter().enumerate() {
                    if c.gone(s) {
                        continue;
                    }
                    let m = c.edges("layout_margin");
                    let len = if c.n("layout_weight", 0.) > 0. {
                        (avail - fixed).max(0.) * c.n("layout_weight", 0.) / weights
                    } else {
                        c.measure(axis, avail, f, s)
                    };
                    let cross = if horiz { inner.height } else { inner.width };
                    let crosslen =
                        c.measure(1 - axis, (cross - m[1 - axis] - m[3 - axis]).max(0.), f, s);
                    let align = if c.attr("layout_gravity").is_empty() {
                        gravity
                    } else {
                        c.attr("layout_gravity")
                    };
                    let shift = if align.contains("center") {
                        (cross - crosslen) / 2.
                    } else if align.contains("end") {
                        cross - crosslen - m[2]
                    } else {
                        m[1 - axis]
                    };
                    let r = if horiz {
                        Rect {
                            x: cursor + m[0],
                            y: inner.y + shift,
                            width: len,
                            height: crosslen,
                        }
                    } else {
                        Rect {
                            x: inner.x + shift,
                            y: cursor + m[1],
                            width: crosslen,
                            height: len,
                        }
                    };
                    clicked |= c.draw(f, r, s, instance.wrapping_add(i as u64 * 100));
                    cursor += len + m[axis] + m[axis + 2];
                }
            }
            _ => {
                let mut rects: Vec<Rect> = self
                    .children
                    .iter()
                    .map(|c| Rect {
                        x: inner.x,
                        y: inner.y,
                        width: c.measure(0, inner.width, f, s),
                        height: c.measure(1, inner.height, f, s),
                    })
                    .collect();
                // Bounded relaxation supports the sibling constraints in the imported templates.
                for _ in 0..10 {
                    for (i, c) in self.children.iter().enumerate() {
                        let lookup = |reference: &str| {
                            let name = reference.rsplit('/').next().unwrap_or(reference);
                            if name == "parent" {
                                inner
                            } else {
                                self.children
                                    .iter()
                                    .position(|n| n.id == name)
                                    .map_or(inner, |j| rects[j])
                            }
                        };
                        let mut r = rects[i];
                        let m = c.edges("layout_margin");
                        if c.kind == "Guideline" {
                            let pos = c.n("layout_constraintGuide_begin", 0.);
                            if c.attr("orientation") == "vertical" {
                                r.x = inner.x + pos;
                                r.width = 0.
                            } else {
                                r.y = if c.attr("layout_constraintGuide_end").is_empty() {
                                    inner.y + pos
                                } else {
                                    inner.y + inner.height - c.n("layout_constraintGuide_end", 0.)
                                };
                                r.height = 0.
                            }
                            rects[i] = r;
                            continue;
                        }
                        if c.kind == "Barrier" {
                            let refs = c.attr("constraint_referenced_ids");
                            let end = c.attr("barrierDirection") == "end";
                            let val = refs
                                .split(',')
                                .map(|v| lookup(v.trim()))
                                .map(|v| if end { v.x + v.width } else { v.y + v.height })
                                .fold(if end { inner.x } else { inner.y }, f32::max);
                            if end {
                                r.x = val;
                                r.width = 0.
                            } else {
                                r.y = val;
                                r.height = 0.
                            }
                            rects[i] = r;
                            continue;
                        }
                        let mut start = [None, None];
                        let mut end = [None, None];
                        for (key, axis, side, target_end) in [
                            ("layout_constraintStart_toStartOf", 0, 0, false),
                            ("layout_constraintStart_toEndOf", 0, 0, true),
                            ("layout_constraintEnd_toStartOf", 0, 1, false),
                            ("layout_constraintEnd_toEndOf", 0, 1, true),
                            ("layout_constraintTop_toTopOf", 1, 0, false),
                            ("layout_constraintTop_toBottomOf", 1, 0, true),
                            ("layout_constraintBottom_toTopOf", 1, 1, false),
                            ("layout_constraintBottom_toBottomOf", 1, 1, true),
                            ("layout_below", 1, 0, true),
                            ("layout_toEndOf", 0, 0, true),
                            ("layout_toRightOf", 0, 0, true),
                            ("layout_toStartOf", 0, 1, false),
                            ("layout_above", 1, 1, false),
                        ] {
                            let v = c.attr(key);
                            if !v.is_empty() {
                                let q = lookup(v);
                                let value = if axis == 0 {
                                    q.x + if target_end { q.width } else { 0. }
                                } else {
                                    q.y + if target_end { q.height } else { 0. }
                                };
                                if side == 0 {
                                    start[axis] = Some(value)
                                } else {
                                    end[axis] = Some(value)
                                }
                            }
                        }
                        if c.attr("layout_alignParentEnd") == "true"
                            || c.attr("layout_gravity").contains("end")
                        {
                            end[0] = Some(inner.x + inner.width)
                        }
                        if c.attr("layout_alignParentBottom") == "true"
                            || c.attr("layout_gravity").contains("bottom")
                        {
                            end[1] = Some(inner.y + inner.height)
                        }
                        for axis in 0..2 {
                            let key = if axis == 0 {
                                "layout_width"
                            } else {
                                "layout_height"
                            };
                            let mut len = c.measure(
                                axis,
                                if axis == 0 { inner.width } else { inner.height },
                                f,
                                s,
                            );
                            let origin = if axis == 0 { inner.x } else { inner.y };
                            let available = if axis == 0 { inner.width } else { inner.height };
                            let lo = start[axis].map(|v| v + m[axis]);
                            let hi = end[axis].map(|v| v - m[axis + 2]);
                            let matching =
                                matches!(c.attr(key), "match_parent" | "fill_parent" | "0dp");
                            if matching {
                                len = (hi.unwrap_or(origin + available - m[axis + 2])
                                    - lo.unwrap_or(origin + m[axis]))
                                .max(0.);
                            }
                            let pos = match (lo, hi) {
                                (Some(l), Some(h)) => {
                                    len = len.min((h - l).max(0.));
                                    l + (h - l - len)
                                        * c.n(
                                            if axis == 0 {
                                                "layout_constraintHorizontal_bias"
                                            } else {
                                                "layout_constraintVertical_bias"
                                            },
                                            0.5,
                                        )
                                }
                                (Some(l), _) => l,
                                (_, Some(h)) => h - len,
                                _ => origin + m[axis],
                            };
                            let center = c.attr(if axis == 0 {
                                "layout_centerHorizontal"
                            } else {
                                "layout_centerVertical"
                            }) == "true"
                                || c.attr("layout_centerInParent") == "true";
                            let pos = if center {
                                origin + (available - len) / 2.
                            } else {
                                pos
                            };
                            if axis == 0 {
                                r.x = pos;
                                r.width = len
                            } else {
                                r.y = pos;
                                r.height = len
                            }
                        }
                        if c.gone(s) {
                            r.width = 0.;
                            r.height = 0.;
                        }
                        rects[i] = r;
                    }
                }
                for (i, (c, r)) in self.children.iter().zip(rects).enumerate() {
                    clicked |= c.draw(f, r, s, instance.wrapping_add(i as u64 * 100));
                }
            }
        }
        clicked
    }
}
fn color(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#')?;
    let n = u32::from_str_radix(s, 16).ok()?;
    Some(Color(
        (n >> 16) as u8,
        (n >> 8) as u8,
        n as u8,
        if s.len() == 8 { (n >> 24) as u8 } else { 255 },
    ))
}
