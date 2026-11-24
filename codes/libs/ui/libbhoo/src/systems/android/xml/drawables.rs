use super::*;

// Android 15 framework group indicators (Apache-2.0); see asset SOURCE.md.
pub(super) fn expandable_indicator(expanded: bool) -> &'static Image {
    static IMAGES: std::sync::OnceLock<[Image; 2]> = std::sync::OnceLock::new();
    &IMAGES.get_or_init(|| [
        Image::rgba(31, 31, include_bytes!("../../../../assets/android-expandable-list/open.rgba").to_vec()).expect("valid collapsed indicator"),
        Image::rgba(31, 31, include_bytes!("../../../../assets/android-expandable-list/close.rgba").to_vec()).expect("valid expanded indicator"),
    ])[usize::from(expanded)]
}
// Android 15 framework mdpi drawables (Apache-2.0); see asset SOURCE.md.
pub(super) fn zoom_control_image(direction:usize, visual:usize) -> &'static Image {
    static IMAGES:std::sync::OnceLock<[Image;10]>=std::sync::OnceLock::new();
    let sources:[&[u8];10]=[
        include_bytes!("../../../../assets/android-zoomcontrols/down_normal.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/down_pressed.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/down_disabled.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/down_selected.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/down_disabled_focused.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/up_normal.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/up_pressed.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/up_disabled.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/up_selected.rgba"),
        include_bytes!("../../../../assets/android-zoomcontrols/up_disabled_focused.rgba"),
    ];
    &IMAGES.get_or_init(|| std::array::from_fn(|i| Image::rgba(79,48,sources[i].to_vec()).expect("valid ZoomControls asset")))[direction*5+visual]
}
/// Draw the framework nine-patch using its single marked stretch run per axis.
pub(super) fn paint_zoom_nine_patch(f:&mut Frame<'_>, bounds:Rect, image:&Image, direction:usize) {
    if (bounds.width-79.0).abs()<0.01 && (bounds.height-48.0).abs()<0.01 {
        f.paint_image(bounds,image,ImageFit::Contain,0.0);
        return;
    }
    let (x,y)=if direction==0 { ([0.0,74.0,75.0,79.0],[0.0,0.0,1.0,48.0]) }
        else { ([0.0,4.0,5.0,79.0],[0.0,7.0,37.0,48.0]) };
    pub(super) fn destination(source:[f32;4], size:f32) -> [f32;4] {
        let fixed=source[1]+source[3]-source[2];
        let factor=if fixed>0.0 { (size/fixed).clamp(0.0,1.0) } else {1.0};
        [0.0,source[1]*factor,(size-(source[3]-source[2])*factor).max(0.0),size.max(0.0)]
    }
    let dx=destination(x,bounds.width);let dy=destination(y,bounds.height);
    for row in 0..3 { for col in 0..3 {
        let w=dx[col+1]-dx[col];let h=dy[row+1]-dy[row];
        if w<=0.0 || h<=0.0 || x[col+1]<=x[col] || y[row+1]<=y[row] { continue; }
        let rect=Rect { x:bounds.x+dx[col],y:bounds.y+dy[row],width:w,height:h };
        if rect.intersect(f.clip).width<=0.0 || rect.intersect(f.clip).height<=0.0 { continue; }
        f.commands.push(DrawCommand::Image { bounds:rect,uv:[x[col]/79.0,y[row]/48.0,x[col+1]/79.0,y[row+1]/48.0],radius:0.0,color:Color(255,255,255,255),image:image.clone() });
    }}
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct NinePatchBands {
    pub(super) x: Vec<(u32, u32)>,
    pub(super) y: Vec<(u32, u32)>,
}
pub(super) fn parse_nine_patch_axis(part: &str) -> Option<Vec<(u32, u32)>> {
    let coords: Vec<u32> = part
        .split(',')
        .map(|s| s.trim().parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    if coords.len() < 2 || !coords.len().is_multiple_of(2) {
        return None;
    }
    let mut bands = Vec::with_capacity(coords.len() / 2);
    let mut prev_end = 0u32;
    for pair in coords.chunks_exact(2) {
        let (start, end) = (pair[0], pair[1]);
        if start > end || start < prev_end {
            return None;
        }
        bands.push((start, end));
        prev_end = end;
    }
    Some(bands)
}
pub(super) fn parse_nine_patch_stretch(spec: &str) -> Option<NinePatchBands> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    let (x_part, y_part) = spec.split_once(';')?;
    let x = parse_nine_patch_axis(x_part)?;
    let y = parse_nine_patch_axis(y_part)?;
    Some(NinePatchBands { x, y })
}
pub(super) fn parse_bg_image_rule(anchor: &str) -> Option<(&str, &str)> {
    let (key, stretch) = anchor.split_once('|').unwrap_or((anchor, ""));
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key, stretch.trim()))
}
pub(super) fn parse_stateful_image_rule(anchor: &str) -> Option<(u8, u8, &str, &str)> {
    let (mask_part, rest) = anchor.split_once('|')?;
    let (req_s, forbid_s) = mask_part.split_once(',')?;
    let required = req_s.trim().parse::<u8>().ok()?;
    let forbidden = forbid_s.trim().parse::<u8>().ok()?;
    let (key, stretch) = rest.split_once('|').unwrap_or((rest, ""));
    Some((required, forbidden, key.trim(), stretch.trim()))
}
pub(super) fn validate_nine_patch_axis(bands: &[(u32, u32)], limit: u32) -> bool {
    if bands.is_empty() {
        return false;
    }
    let mut prev_end = 0u32;
    for &(start, end) in bands {
        if start > end || start < prev_end || end > limit {
            return false;
        }
        prev_end = end;
    }
    true
}
/// Split one nine-patch axis into alternating fixed and stretchable `(src0, src1, dst0, dst1)` segments.
/// Remaining `(dst_size - total_fixed)` pixels are distributed proportionally across all stretchable bands,
/// matching Android Skia `NinePatch`.
pub(super) fn nine_patch_axis_segments(
    bands: &[(u32, u32)],
    total: u32,
    dst_size: f32,
    density: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let mut raw: Vec<(f32, f32, bool)> = Vec::with_capacity(bands.len() * 2 + 1);
    let mut cursor = 0u32;
    for &(b0, b1) in bands {
        if b0 > cursor {
            raw.push((cursor as f32, b0 as f32, false));
        }
        if b1 > b0 {
            raw.push((b0 as f32, b1 as f32, true));
        }
        cursor = b1;
    }
    if cursor < total {
        raw.push((cursor as f32, total as f32, false));
    }
    let total_fixed_src: f32 = raw.iter().filter(|s| !s.2).map(|s| s.1 - s.0).sum();
    let total_stretch_src: f32 = raw.iter().filter(|s| s.2).map(|s| s.1 - s.0).sum();
    let total_fixed_dst = total_fixed_src * density;
    let mut out = Vec::with_capacity(raw.len());
    let mut dst_pos = 0.0_f32;
    if total_stretch_src <= 0.0 {
        let scale = if total_fixed_dst > 0.0 { dst_size / total_fixed_dst } else { 0.0 };
        for (s0, s1, _) in raw {
            let len = (s1 - s0) * density * scale;
            if len > 0.0 {
                let next = (dst_pos + len).min(dst_size);
                out.push((s0, s1, dst_pos, next));
                dst_pos = next;
            }
        }
    } else if total_fixed_dst >= dst_size && total_fixed_dst > 0.0 {
        let factor = dst_size / total_fixed_dst;
        for (s0, s1, is_stretch) in raw {
            if is_stretch {
                continue;
            }
            let len = (s1 - s0) * density * factor;
            if len > 0.0 {
                let next = (dst_pos + len).min(dst_size);
                out.push((s0, s1, dst_pos, next));
                dst_pos = next;
            }
        }
    } else {
        let rem = (dst_size - total_fixed_dst).max(0.0);
        for (s0, s1, is_stretch) in raw {
            let len = if is_stretch {
                rem * ((s1 - s0) / total_stretch_src)
            } else {
                (s1 - s0) * density
            };
            if len > 0.0 {
                let next = (dst_pos + len).min(dst_size);
                out.push((s0, s1, dst_pos, next));
                dst_pos = next;
            }
        }
    }
    if let Some(last) = out.last_mut()
        && (dst_size - last.3).abs() < 0.05
    {
        last.3 = dst_size;
    }
    out
}
/// Draw an arbitrary Android `.9.png` (single-band or multi-segment) by slicing into UV-mapped tiles
/// so fixed borders/dividers scale cleanly by `density` while stretch bands divide remaining space proportionally.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_nine_patch_image(
    f: &mut Frame<'_>,
    state: &mut LayoutState,
    dst: Rect,
    img: &Image,
    bands: &NinePatchBands,
    tint: Option<Color>,
    tint_mode: u8,
    alpha: u8,
    density: f32,
) {
    let iw_u = img.width();
    let ih_u = img.height();
    let iw = iw_u as f32;
    let ih = ih_u as f32;
    if iw <= 0.0
        || ih <= 0.0
        || dst.width <= 0.0
        || dst.height <= 0.0
        || !validate_nine_patch_axis(&bands.x, iw_u)
        || !validate_nine_patch_axis(&bands.y, ih_u)
    {
        return;
    }
    let density = if density.is_finite() && density > 0.0 { density } else { 1.0 };
    let x_segs = nine_patch_axis_segments(&bands.x, iw_u, dst.width, density);
    let y_segs = nine_patch_axis_segments(&bands.y, ih_u, dst.height, density);
    let image = tint_image(state, img, tint, tint_mode);
    let color = Color(255, 255, 255, alpha);
    for &(sy0, sy1, dy0, dy1) in &y_segs {
        let h = dy1 - dy0;
        if h <= 0.0 || sy1 <= sy0 {
            continue;
        }
        for &(sx0, sx1, dx0, dx1) in &x_segs {
            let w = dx1 - dx0;
            if w <= 0.0 || sx1 <= sx0 {
                continue;
            }
            let rect = Rect {
                x: dst.x + dx0,
                y: dst.y + dy0,
                width: w,
                height: h,
            };
            f.commands.push(DrawCommand::Image {
                bounds: rect,
                uv: [sx0 / iw, sy0 / ih, sx1 / iw, sy1 / ih],
                radius: 0.0,
                color,
                image: image.clone(),
            });
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum TransformPivot {
    Center,
    Percent(f32),
    Dp(f32),
}
impl TransformPivot {
    pub(super) fn resolve(self, extent: f32, state: &LayoutState) -> f32 {
        match self {
            Self::Center => extent * 0.5,
            Self::Percent(pct) => extent * (pct * 0.01),
            Self::Dp(dp) => state.dp(dp),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ViewTransformSpec {
    pub(super) tx_dp: f32,
    pub(super) ty_dp: f32,
    pub(super) sx: f32,
    pub(super) sy: f32,
    pub(super) rot_deg: f32,
    pub(super) px: TransformPivot,
    pub(super) py: TransformPivot,
}
pub(super) fn parse_transform_pivot(raw: &str) -> Option<TransformPivot> {
    let s = raw.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("center") {
        return Some(TransformPivot::Center);
    }
    if let Some(pct_str) = s.strip_suffix('%') {
        let v = pct_str.trim().parse::<f32>().ok()?;
        return v.is_finite().then_some(TransformPivot::Percent(v));
    }
    let v = s.parse::<f32>().ok()?;
    v.is_finite().then_some(TransformPivot::Dp(v))
}
pub(super) fn parse_view_transform(anchor: &str) -> Option<ViewTransformSpec> {
    let mut parts = anchor.split(',');
    let tx_dp = parts.next()?.trim().parse::<f32>().ok()?;
    let ty_dp = parts.next()?.trim().parse::<f32>().ok()?;
    let sx = parts.next()?.trim().parse::<f32>().ok()?;
    let sy = parts.next()?.trim().parse::<f32>().ok()?;
    let rot_deg = parts.next()?.trim().parse::<f32>().ok()?;
    let px = parse_transform_pivot(parts.next().unwrap_or("center"))?;
    let py = parse_transform_pivot(parts.next().unwrap_or("center"))?;
    if parts.next().is_some() {
        return None;
    }
    if ![tx_dp, ty_dp, sx, sy, rot_deg].iter().all(|v| v.is_finite()) {
        return None;
    }
    Some(ViewTransformSpec { tx_dp, ty_dp, sx, sy, rot_deg, px, py })
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum AnimPivot {
    Center,
    Percent(f32),
    ParentPercent(f32),
    Dp(f32),
}
impl AnimPivot {
    pub(super) fn resolve(
        self,
        own_origin: f32,
        own_extent: f32,
        parent_origin: f32,
        parent_extent: f32,
        state: &LayoutState,
    ) -> f32 {
        match self {
            Self::Center => own_origin + own_extent * 0.5,
            Self::Percent(pct) => own_origin + own_extent * (pct * 0.01),
            Self::ParentPercent(pct) => parent_origin + parent_extent * (pct * 0.01),
            Self::Dp(dp) => own_origin + state.dp(dp),
        }
    }
}
pub(super) fn parse_anim_pivot(raw: &str) -> Option<AnimPivot> {
    let s = raw.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("center") {
        return Some(AnimPivot::Center);
    }
    if let Some(pct_str) = s.strip_suffix("%p").or_else(|| s.strip_suffix("%P")) {
        let v = pct_str.trim().parse::<f32>().ok()?;
        return v.is_finite().then_some(AnimPivot::ParentPercent(v));
    }
    if let Some(pct_str) = s.strip_suffix('%') {
        let v = pct_str.trim().parse::<f32>().ok()?;
        return v.is_finite().then_some(AnimPivot::Percent(v));
    }
    let trimmed = s
        .strip_suffix("dp")
        .or_else(|| s.strip_suffix("dip"))
        .unwrap_or(s)
        .trim();
    let v = trimmed.parse::<f32>().ok()?;
    v.is_finite().then_some(AnimPivot::Dp(v))
}
pub(super) fn parse_scale_factor(raw: &str) -> Option<f32> {
    let s = raw.trim();
    if let Some(pct) = s.strip_suffix('%') {
        let v = pct.trim().parse::<f32>().ok()?;
        return v.is_finite().then_some(v * 0.01);
    }
    let v = s.parse::<f32>().ok()?;
    v.is_finite().then_some(v)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ScaleAnimationSpec {
    pub(super) from_sx: f32,
    pub(super) to_sx: f32,
    pub(super) from_sy: f32,
    pub(super) to_sy: f32,
    pub(super) px: AnimPivot,
    pub(super) py: AnimPivot,
    pub(super) duration: f64,
    pub(super) start_offset: f64,
    pub(super) linear: bool,
}
impl ScaleAnimationSpec {
    pub(super) fn total_duration(self) -> f64 {
        self.start_offset + self.duration
    }
    pub(super) fn sample(self, elapsed: f64, object: bool) -> [f32; 2] {
        if object && elapsed < self.start_offset {
            return [1.0, 1.0];
        }
        let raw = if elapsed < self.start_offset {
            0.0
        } else if self.duration <= 0.0 {
            1.0
        } else {
            ((elapsed - self.start_offset) / self.duration).clamp(0.0, 1.0) as f32
        };
        let t = if self.linear {
            raw
        } else {
            (1.0 - (std::f32::consts::PI * raw).cos()) * 0.5
        };
        [
            self.from_sx + (self.to_sx - self.from_sx) * t,
            self.from_sy + (self.to_sy - self.from_sy) * t,
        ]
    }
    pub(super) fn resolve_origin(self, child: Rect, parent: Rect, state: &LayoutState) -> [f32; 2] {
        [
            self.px.resolve(child.x, child.width, parent.x, parent.width, state),
            self.py.resolve(child.y, child.height, parent.y, parent.height, state),
        ]
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RotateAnimationSpec {
    pub(super) from_deg: f32,
    pub(super) to_deg: f32,
    pub(super) px: AnimPivot,
    pub(super) py: AnimPivot,
    pub(super) duration: f64,
    pub(super) start_offset: f64,
    pub(super) linear: bool,
}
impl RotateAnimationSpec {
    pub(super) fn total_duration(self) -> f64 {
        self.start_offset + self.duration
    }
    pub(super) fn sample(self, elapsed: f64, object: bool) -> f32 {
        if object && elapsed < self.start_offset {
            return 0.0;
        }
        let raw = if elapsed < self.start_offset {
            0.0
        } else if self.duration <= 0.0 {
            1.0
        } else {
            ((elapsed - self.start_offset) / self.duration).clamp(0.0, 1.0) as f32
        };
        let t = if self.linear {
            raw
        } else {
            (1.0 - (std::f32::consts::PI * raw).cos()) * 0.5
        };
        self.from_deg + (self.to_deg - self.from_deg) * t
    }
    pub(super) fn resolve_origin(self, child: Rect, parent: Rect, state: &LayoutState) -> [f32; 2] {
        [
            self.px.resolve(child.x, child.width, parent.x, parent.width, state),
            self.py.resolve(child.y, child.height, parent.y, parent.height, state),
        ]
    }
}
pub(super) fn parse_anim_pivot_and_timing(
    rest: &[&str],
) -> Option<(AnimPivot, AnimPivot, f64, f64, bool)> {
    let mut idx = 0usize;
    let (px, py) = if let Some(&seg) = rest.get(idx) {
        if seg.contains(',') || seg.is_empty() || seg.eq_ignore_ascii_case("center") || seg.contains('%') {
            idx += 1;
            let (sx, sy) = seg.split_once(',').unwrap_or((seg, "center"));
            (parse_anim_pivot(sx)?, parse_anim_pivot(sy)?)
        } else {
            (AnimPivot::Center, AnimPivot::Center)
        }
    } else {
        (AnimPivot::Center, AnimPivot::Center)
    };
    let duration = rest
        .get(idx)
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.3);
    let start_offset = rest
        .get(idx + 1)
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.0);
    let linear = rest
        .get(idx + 2)
        .is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true"));
    Some((px, py, duration, start_offset, linear))
}
pub(super) fn parse_scale_animation(anchor: &str) -> Option<ScaleAnimationSpec> {
    let parts: Vec<&str> = anchor.split('|').collect();
    let vals: Vec<&str> = parts.first()?.split(',').collect();
    let (from_sx, to_sx, from_sy, to_sy) = match vals.as_slice() {
        [fsx, tsx, fsy, tsy] => (
            parse_scale_factor(fsx)?,
            parse_scale_factor(tsx)?,
            parse_scale_factor(fsy)?,
            parse_scale_factor(tsy)?,
        ),
        [fs, ts] => {
            let f = parse_scale_factor(fs)?;
            let t = parse_scale_factor(ts)?;
            (f, t, f, t)
        }
        _ => return None,
    };
    let (px, py, duration, start_offset, linear) = parse_anim_pivot_and_timing(&parts[1..])?;
    Some(ScaleAnimationSpec {
        from_sx,
        to_sx,
        from_sy,
        to_sy,
        px,
        py,
        duration,
        start_offset,
        linear,
    })
}
pub(super) fn parse_rotate_animation(anchor: &str) -> Option<RotateAnimationSpec> {
    let parts: Vec<&str> = anchor.split('|').collect();
    let vals: Vec<&str> = parts.first()?.split(',').collect();
    let [fd, td] = vals.as_slice() else { return None };
    let from_deg = fd.trim().parse::<f32>().ok().filter(|v| v.is_finite())?;
    let to_deg = td.trim().parse::<f32>().ok().filter(|v| v.is_finite())?;
    let (px, py, duration, start_offset, linear) = parse_anim_pivot_and_timing(&parts[1..])?;
    Some(RotateAnimationSpec {
        from_deg,
        to_deg,
        px,
        py,
        duration,
        start_offset,
        linear,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AnimInterp {
    Linear,
    AccelerateDecelerate,
    Accelerate,
    Decelerate,
    FastOutSlowIn,
    Overshoot,
    Anticipate,
}
impl AnimInterp {
    pub(super) fn parse(raw: &str) -> Self {
        let lower = raw.trim().to_ascii_lowercase();
        let name = lower
            .strip_prefix("@android:anim/")
            .or_else(|| lower.strip_prefix("@android:interpolator/"))
            .or_else(|| lower.strip_prefix("@interpolator/"))
            .unwrap_or(&lower);
        match name {
            "linear" | "linear_interpolator" | "1" | "true" => Self::Linear,
            "accelerate" | "accelerate_interpolator" | "accelerate_quad" => Self::Accelerate,
            "decelerate" | "decelerate_interpolator" | "decelerate_quad" => Self::Decelerate,
            "fast_out_slow_in" | "emphasized" => Self::FastOutSlowIn,
            "overshoot" | "overshoot_interpolator" => Self::Overshoot,
            "anticipate" | "anticipate_interpolator" => Self::Anticipate,
            _ => Self::AccelerateDecelerate,
        }
    }
    pub(super) fn eval(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::AccelerateDecelerate => (1.0 - (std::f32::consts::PI * t).cos()) * 0.5,
            Self::Accelerate => t * t,
            Self::Decelerate => 1.0 - (1.0 - t) * (1.0 - t),
            Self::FastOutSlowIn => {
                // Material cubic-bezier(0.4, 0.0, 0.2, 1.0)
                let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
                for _ in 0..14 {
                    let u = (lo + hi) * 0.5;
                    let inv = 1.0 - u;
                    let x = 3.0 * inv * inv * u * 0.4 + 3.0 * inv * u * u * 0.2 + u * u * u;
                    if x < t { lo = u; } else { hi = u; }
                }
                let u = (lo + hi) * 0.5;
                let inv = 1.0 - u;
                3.0 * inv * u * u + u * u * u
            }
            Self::Overshoot => {
                let u = t - 1.0;
                u * u * (3.0 * u + 2.0) + 1.0
            }
            Self::Anticipate => t * t * (3.0 * t - 2.0),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ElevationAnimationSpec {
    pub(super) from_dp: f32,
    pub(super) to_dp: f32,
    pub(super) duration: f64,
    pub(super) start_offset: f64,
    pub(super) interp: AnimInterp,
}
impl ElevationAnimationSpec {
    pub(super) fn total_duration(self) -> f64 {
        self.start_offset + self.duration
    }
    pub(super) fn sample(self, elapsed: f64) -> f32 {
        let raw = if elapsed < self.start_offset {
            0.0
        } else if self.duration <= 0.0 {
            1.0
        } else {
            ((elapsed - self.start_offset) / self.duration).clamp(0.0, 1.0) as f32
        };
        let eased = self.interp.eval(raw);
        (self.from_dp + (self.to_dp - self.from_dp) * eased).max(0.0)
    }
}
pub(super) fn parse_elevation_animation(anchor: &str) -> Option<ElevationAnimationSpec> {
    let parts: Vec<&str> = anchor.split('|').collect();
    let vals: Vec<&str> = parts.first()?.split(',').collect();
    let [fd, td] = vals.as_slice() else { return None };
    let parse_dp = |s: &str| {
        s.trim()
            .strip_suffix("dp")
            .or_else(|| s.trim().strip_suffix("dip"))
            .unwrap_or(s.trim())
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|v| v.is_finite())
    };
    let from_dp = parse_dp(fd)?;
    let to_dp = parse_dp(td)?;
    let dur_ms = parts
        .get(1)
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(300.0);
    let offset_ms = parts
        .get(2)
        .and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.0);
    let interp = parts.get(3).map_or(AnimInterp::AccelerateDecelerate, |s| AnimInterp::parse(s));
    Some(ElevationAnimationSpec {
        from_dp,
        to_dp,
        duration: dur_ms / 1000.0,
        start_offset: offset_ms / 1000.0,
        interp,
    })
}
pub(super) fn push_transition_transforms(
    f: &mut Frame<'_>,
    scale_spec: Option<ScaleAnimationSpec>,
    rotate_spec: Option<RotateAnimationSpec>,
    elapsed: f64,
    child: Rect,
    parent: Rect,
    state: &LayoutState,
    object: bool,
) -> u8 {
    match (scale_spec, rotate_spec) {
        (Some(s), Some(r)) => {
            let scale = s.sample(elapsed, object);
            let deg = r.sample(elapsed, object);
            let so = s.resolve_origin(child, parent, state);
            let ro = r.resolve_origin(child, parent, state);
            if (so[0] - ro[0]).abs() < 0.01 && (so[1] - ro[1]).abs() < 0.01 {
                f.push_transform(so, [0.0, 0.0], scale, deg.to_radians());
                1
            } else {
                f.push_transform(ro, [0.0, 0.0], [1.0, 1.0], deg.to_radians());
                f.push_transform(so, [0.0, 0.0], scale, 0.0);
                2
            }
        }
        (Some(s), None) => {
            let scale = s.sample(elapsed, object);
            let so = s.resolve_origin(child, parent, state);
            f.push_transform(so, [0.0, 0.0], scale, 0.0);
            1
        }
        (None, Some(r)) => {
            let deg = r.sample(elapsed, object);
            let ro = r.resolve_origin(child, parent, state);
            f.push_transform(ro, [0.0, 0.0], [1.0, 1.0], deg.to_radians());
            1
        }
        (None, None) => 0,
    }
}

pub(super) fn tint_pixel(source:Color, tint:Color, mode:u8) -> Color {
    let Color(r,g,b,a)=source; let Color(tr,tg,tb,ta)=tint;
    if mode==1 { Color(tr,tg,tb,((a as u32*ta as u32+127)/255) as u8) }
    else if mode==0 {
        let blend=|s:u8,t:u8| ((t as u32*ta as u32+s as u32*(255-ta as u32)+127)/255) as u8;
        Color(blend(r,tr),blend(g,tg),blend(b,tb),a)
    } else {
        // Porter-Duff operates on premultiplied channels. Convert back to
        // straight RGBA for Bhoo's shared image representation.
        let sa=ta as f32/255.0;let da=a as f32/255.0;
        let alpha=match mode { 2=>sa+da*(1.0-sa), 3=>sa*da, 4=>sa+da-sa*da, 5=>(sa+da).min(1.0), _=>da };
        let channel=|src:u8,dst:u8| {
            let s=src as f32/255.0*sa;let d=dst as f32/255.0*da;
            let premul=match mode { 2=>s+(1.0-sa)*d,3=>s*d,4=>s+d-s*d,5=>(s+d).min(1.0),_=>d };
            if alpha<=0.0 {0} else { ((premul/alpha).clamp(0.0,1.0)*255.0).round() as u8 }
        };
        Color(channel(tr,r),channel(tg,g),channel(tb,b),(alpha*255.0).round() as u8)
    }
}
pub(super) fn tint_image(state:&mut LayoutState, image:&Image, tint:Option<Color>, mode:u8) -> Image {
    let Some(color)=tint else { return image.clone(); };
    let key=(image.id(),[color.0,color.1,color.2,color.3],mode);
    if let Some(cached)=state.tinted_images.get(&key) { return cached.clone(); }
    let bytes=image.pixels().len();
    if state.tinted_image_bytes.saturating_add(bytes)>8*1024*1024 {
        state.tinted_images.clear();state.tinted_image_bytes=0;
    }
    let mut pixels=Vec::with_capacity(bytes);
    for px in image.pixels().chunks_exact(4) {
        let Color(r,g,b,a)=tint_pixel(Color(px[0],px[1],px[2],px[3]),color,mode);
        pixels.extend_from_slice(&[r,g,b,a]);
    }
    let result=Image::rgba(image.width(),image.height(),pixels).expect("tint preserves validated image dimensions");
    state.tinted_image_bytes+=bytes;
    state.tinted_images.insert(key,result.clone());
    result
}

/// Draw a filled oval within its actual bounds.
pub(super) fn paint_oval(f:&mut Frame<'_>, bounds:Rect, color:Color) {
    if bounds.width>0.0 && bounds.height>0.0 { f.commands.push(DrawCommand::Ellipse { bounds,color }); }
}

pub(super) fn parse_corner_radii(anchor: &str) -> Option<[f32; 4]> {
    let mut parts = anchor.split(',').map(|p| p.trim().parse::<f32>().ok());
    let tl = parts.next()??;
    let tr = parts.next()??;
    let br = parts.next()??;
    let bl = parts.next()??;
    if parts.next().is_some() || ![tl, tr, br, bl].iter().all(|v| v.is_finite() && *v >= 0.0) {
        return None;
    }
    Some([tl, tr, br, bl])
}

pub(super) fn parse_corner_family(anchor: &str) -> Option<[bool; 4]> {
    let mut parts = anchor.split(',').map(str::trim);
    let parse_tok = |s: &str| -> Option<bool> {
        if s.eq_ignore_ascii_case("c") || s.eq_ignore_ascii_case("cut") {
            Some(true)
        } else if s.eq_ignore_ascii_case("r") || s.eq_ignore_ascii_case("rounded") {
            Some(false)
        } else {
            None
        }
    };
    let tl = parse_tok(parts.next()?)?;
    let tr = parse_tok(parts.next()?)?;
    let br = parse_tok(parts.next()?)?;
    let bl = parse_tok(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some([tl, tr, br, bl])
}

pub(super) fn has_active_cut_corner(cut: [bool; 4], radii: [f32; 4]) -> bool {
    (0..4).any(|i| cut[i] && radii[i] > 0.01)
}

pub(super) fn raster_dimensions_capped(width: f32, height: f32, cap: u32) -> Option<(u32, u32)> {
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let cap_f = cap as f32;
    let max_dim = width.max(height);
    let scale = if max_dim > cap_f { cap_f / max_dim } else { 1.0 };
    let w = (width * scale).round().clamp(2.0, cap_f) as u32;
    let h = (height * scale).round().clamp(2.0, cap_f) as u32;
    Some((w, h))
}

/// Exact Euclidean signed distance (in raster pixels) to a rectangle with per-corner cut or rounded corners.
/// Negative inside, positive outside.
pub(super) fn corner_shape_sdf(x: f32, y: f32, w: f32, h: f32, r: [f32; 4], cut: [bool; 4]) -> f32 {
    let mut d = (-x).max(x - w).max(-y).max(y - h);
    let corners = [
        (x, y, r[0], cut[0]),
        (w - x, y, r[1], cut[1]),
        (w - x, h - y, r[2], cut[2]),
        (x, h - y, r[3], cut[3]),
    ];
    for (dx, dy, ri, is_cut) in corners {
        if ri > 0.0 && dx < ri && dy < ri {
            let dc = if is_cut {
                (ri - dx - dy) * std::f32::consts::FRAC_1_SQRT_2
            } else {
                (ri - dx).hypot(ri - dy) - ri
            };
            d = d.max(dc);
        }
    }
    d
}

/// Paint a filled rectangle with one or more chamfered (`cut`) corners using a bounded cached RGBA `Image`
/// so both GLES and Canvas 2D render anti-aliased bevels natively with zero new packet tags.
pub(super) fn paint_cut_shape_fill(
    f: &mut Frame<'_>,
    state: &mut LayoutState,
    b: Rect,
    radii: [f32; 4],
    cut: [bool; 4],
    color: Color,
) {
    if b.width <= 0.0 || b.height <= 0.0 || color.3 == 0 {
        return;
    }
    let r_norm = normalize_corner_radii(b, radii);
    if !has_active_cut_corner(cut, r_norm) {
        paint_rounded_rect_radii(f, b, r_norm, color);
        return;
    }
    let Some((img_w, img_h)) = raster_dimensions_capped(b.width, b.height, 256) else {
        return;
    };
    let sx = img_w as f32 / b.width;
    let sy = img_h as f32 / b.height;
    let r_px = [
        r_norm[0] * sx.min(sy),
        r_norm[1] * sx.min(sy),
        r_norm[2] * sx.min(sy),
        r_norm[3] * sx.min(sy),
    ];
    let key = CutShapeCacheKey {
        w: img_w as u16,
        h: img_h as u16,
        radii_q: r_px.map(|v| (v * 16.0).round().clamp(0.0, u16::MAX as f32) as u16),
        cut,
        color: [color.0, color.1, color.2, color.3],
    };
    let image = if let Some(cached) = state.cut_shape_images.get(&key) {
        cached.clone()
    } else {
        let bytes = (img_w as usize) * (img_h as usize) * 4;
        if state.cut_shape_images.len() >= 48
            || state.cut_shape_image_bytes.saturating_add(bytes) > 4 * 1024 * 1024
        {
            state.cut_shape_images.clear();
            state.cut_shape_image_bytes = 0;
        }
        let mut pixels = Vec::with_capacity(bytes);
        let wf = img_w as f32;
        let hf = img_h as f32;
        for py in 0..img_h {
            let y = py as f32 + 0.5;
            for px in 0..img_w {
                let x = px as f32 + 0.5;
                let d = corner_shape_sdf(x, y, wf, hf, r_px, cut);
                let cov = (0.5 - d).clamp(0.0, 1.0);
                let a = (color.3 as f32 * cov).round() as u8;
                pixels.extend_from_slice(&[color.0, color.1, color.2, a]);
            }
        }
        let img = Image::rgba(img_w, img_h, pixels).expect("cut shape raster within bounded size");
        state.cut_shape_image_bytes += bytes;
        state.cut_shape_images.insert(key, img.clone());
        img
    };
    f.commands.push(DrawCommand::Image {
        bounds: b,
        uv: [0.0, 0.0, 1.0, 1.0],
        radius: 0.0,
        color: Color(255, 255, 255, 255),
        image,
    });
}

/// Signed distance to a Material `BottomAppBar` top edge with a circular FAB cradle cutout
/// (radius `cradle_r`, center `(cx, voff)`) and rounded shoulder fillets (`fillet_r`).
/// Negative inside the bar, positive outside/inside the transparent cutout.
pub(super) fn bottom_app_bar_cradle_sdf(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    cx: f32,
    voff: f32,
    cradle_r: f32,
    fillet_r: f32,
) -> f32 {
    let dx = (x - cx).abs();
    let r_big = cradle_r.max(0.0);
    let r_fil = fillet_r.max(0.0);
    let d_top = if r_big <= 0.001 {
        -y
    } else if r_fil <= 0.001 {
        (-y).max(r_big - dx.hypot(y - voff))
    } else {
        let sum = r_big + r_fil;
        let dy_f = r_fil - voff;
        let disc = sum * sum - dy_f * dy_f;
        if disc <= 0.0 {
            -y
        } else {
            let dx_f = disc.sqrt();
            if dx >= dx_f {
                -y
            } else {
                let cross = dx * dy_f - (y - voff) * dx_f;
                if cross >= 0.0 {
                    (dx - dx_f).hypot(y - r_fil) - r_fil
                } else {
                    r_big - dx.hypot(y - voff)
                }
            }
        }
    };
    d_top.max(-x).max(x - w).max(y - h)
}

/// Paint a `BottomAppBar` surface with an anti-aliased transparent semicircular FAB cradle cutout
/// using a bounded cached RGBA `Image`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_bottom_app_bar_cradle(
    f: &mut Frame<'_>,
    state: &mut LayoutState,
    b: Rect,
    fab_cx_local: f32,
    cradle_voff_px: f32,
    cradle_r_px: f32,
    fillet_r_px: f32,
    color: Color,
) {
    if b.width <= 0.0 || b.height <= 0.0 || color.3 == 0 {
        return;
    }
    let Some((img_w, img_h)) = raster_dimensions_capped(b.width, b.height, 512) else {
        return;
    };
    let sx = img_w as f32 / b.width;
    let sy = img_h as f32 / b.height;
    let cx = fab_cx_local * sx;
    let voff = cradle_voff_px * sy;
    let scale = sx.min(sy);
    let cradle_r = cradle_r_px.max(0.0) * scale;
    let fillet_r = fillet_r_px.max(0.0) * scale;
    let key = BottomAppBarCradleCacheKey {
        w: img_w as u16,
        h: img_h as u16,
        cx_q: (cx * 8.0).round().clamp(0.0, u16::MAX as f32) as u16,
        voff_q: (voff * 8.0).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16,
        cradle_r_q: (cradle_r * 8.0).round().clamp(0.0, u16::MAX as f32) as u16,
        fillet_r_q: (fillet_r * 8.0).round().clamp(0.0, u16::MAX as f32) as u16,
        color: [color.0, color.1, color.2, color.3],
    };
    let image = if let Some(cached) = state.bottom_app_bar_images.get(&key) {
        cached.clone()
    } else {
        let bytes = (img_w as usize) * (img_h as usize) * 4;
        if state.bottom_app_bar_images.len() >= 32
            || state.bottom_app_bar_image_bytes.saturating_add(bytes) > 4 * 1024 * 1024
        {
            state.bottom_app_bar_images.clear();
            state.bottom_app_bar_image_bytes = 0;
        }
        let mut pixels = Vec::with_capacity(bytes);
        let wf = img_w as f32;
        let hf = img_h as f32;
        for py in 0..img_h {
            let y = py as f32 + 0.5;
            for px in 0..img_w {
                let x = px as f32 + 0.5;
                let d = bottom_app_bar_cradle_sdf(x, y, wf, hf, cx, voff, cradle_r, fillet_r);
                let cov = (0.5 - d).clamp(0.0, 1.0);
                let a = (color.3 as f32 * cov).round() as u8;
                pixels.extend_from_slice(&[color.0, color.1, color.2, a]);
            }
        }
        let img = Image::rgba(img_w, img_h, pixels).expect("bottom app bar cradle within bounded size");
        state.bottom_app_bar_image_bytes += bytes;
        state.bottom_app_bar_images.insert(key, img.clone());
        img
    };
    f.commands.push(DrawCommand::Image {
        bounds: b,
        uv: [0.0, 0.0, 1.0, 1.0],
        radius: 0.0,
        color: Color(255, 255, 255, 255),
        image,
    });
}

/// Paint a crisp stroke outline around a rectangle with chamfered (`cut`) and/or rounded corners
/// using native `DrawCommand::Line` (and `DrawCommand::Arc` for mixed rounded corners).
pub(super) fn paint_cut_shape_outline(
    f: &mut Frame<'_>,
    b: Rect,
    radii: [f32; 4],
    cut: [bool; 4],
    width: f32,
    color: Color,
) {
    if b.width <= 0.0 || b.height <= 0.0 || width <= 0.0 || color.3 == 0 {
        return;
    }
    let r = normalize_corner_radii(b, radii);
    if !has_active_cut_corner(cut, r) {
        paint_outline_radii(f, b, r, width, color);
        return;
    }
    let sw = width.min(b.width * 0.5).min(b.height * 0.5);
    let half = sw * 0.5;
    let x0 = b.x + half;
    let y0 = b.y + half;
    let x1 = b.x + b.width - half;
    let y1 = b.y + b.height - half;
    let iw = (x1 - x0).max(0.0);
    let ih = (y1 - y0).max(0.0);
    if iw <= 0.0 || ih <= 0.0 {
        return;
    }
    let inset_rect = Rect { x: x0, y: y0, width: iw, height: ih };
    let miter_shrink = half * (2.0 - std::f32::consts::SQRT_2);
    let raw_off = [
        if cut[0] { (r[0] - miter_shrink).max(0.0) } else { (r[0] - half).max(0.0) },
        if cut[1] { (r[1] - miter_shrink).max(0.0) } else { (r[1] - half).max(0.0) },
        if cut[2] { (r[2] - miter_shrink).max(0.0) } else { (r[2] - half).max(0.0) },
        if cut[3] { (r[3] - miter_shrink).max(0.0) } else { (r[3] - half).max(0.0) },
    ];
    let off = normalize_corner_radii(inset_rect, raw_off);
    let emit_line = |cmds: &mut Vec<DrawCommand>, from: [f32; 2], to: [f32; 2]| {
        if (to[0] - from[0]).hypot(to[1] - from[1]) > 0.05 {
            cmds.push(DrawCommand::Line { from, to, width: sw, color });
        }
    };
    let emit_corner_arc = |cmds: &mut Vec<DrawCommand>, cx: f32, cy: f32, ar: f32, start: f32| {
        if ar > 0.05 {
            cmds.push(DrawCommand::Arc {
                bounds: Rect {
                    x: cx - ar - half,
                    y: cy - ar - half,
                    width: 2.0 * ar + sw,
                    height: 2.0 * ar + sw,
                },
                start,
                sweep: std::f32::consts::FRAC_PI_2,
                stroke: sw,
                color,
                rounded: true,
            });
        }
    };
    let cmds = &mut f.commands;
    // Top edge
    emit_line(cmds, [x0 + off[0], y0], [x1 - off[1], y0]);
    // Top-right corner
    if cut[1] {
        emit_line(cmds, [x1 - off[1], y0], [x1, y0 + off[1]]);
    } else {
        emit_corner_arc(cmds, x1 - off[1], y0 + off[1], off[1], -std::f32::consts::FRAC_PI_2);
    }
    // Right edge
    emit_line(cmds, [x1, y0 + off[1]], [x1, y1 - off[2]]);
    // Bottom-right corner
    if cut[2] {
        emit_line(cmds, [x1, y1 - off[2]], [x1 - off[2], y1]);
    } else {
        emit_corner_arc(cmds, x1 - off[2], y1 - off[2], off[2], 0.0);
    }
    // Bottom edge
    emit_line(cmds, [x1 - off[2], y1], [x0 + off[3], y1]);
    // Bottom-left corner
    if cut[3] {
        emit_line(cmds, [x0 + off[3], y1], [x0, y1 - off[3]]);
    } else {
        emit_corner_arc(cmds, x0 + off[3], y1 - off[3], off[3], std::f32::consts::FRAC_PI_2);
    }
    // Left edge
    emit_line(cmds, [x0, y1 - off[3]], [x0, y0 + off[0]]);
    // Top-left corner
    if cut[0] {
        emit_line(cmds, [x0, y0 + off[0]], [x0 + off[0], y0]);
    } else {
        emit_corner_arc(cmds, x0 + off[0], y0 + off[0], off[0], std::f32::consts::PI);
    }
}

/// Paint a solid fill honoring uniform radius, asymmetric corner radii, oval, and chamfered cut corners.
pub(super) fn paint_shape_fill_with_family(
    f: &mut Frame<'_>,
    state: &mut LayoutState,
    b: Rect,
    is_oval: bool,
    corner_radii: Option<[f32; 4]>,
    uniform_radius: f32,
    cut: [bool; 4],
    color: Color,
) {
    if b.width <= 0.0 || b.height <= 0.0 || color.3 == 0 {
        return;
    }
    if is_oval {
        paint_oval(f, b, color);
        return;
    }
    let eff_radii = corner_radii.unwrap_or([uniform_radius.max(0.0); 4]);
    if has_active_cut_corner(cut, eff_radii) {
        paint_cut_shape_fill(f, state, b, eff_radii, cut, color);
    } else if let Some(radii) = corner_radii {
        paint_rounded_rect_radii(f, b, radii, color);
    } else {
        f.paint_rect(b, uniform_radius.max(0.0), color);
    }
}

/// Paint a stroke outline honoring uniform radius, asymmetric corner radii, oval, and chamfered cut corners.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_shape_outline_with_family(
    f: &mut Frame<'_>,
    b: Rect,
    is_oval: bool,
    corner_radii: Option<[f32; 4]>,
    uniform_radius: f32,
    cut: [bool; 4],
    width: f32,
    color: Color,
) {
    if b.width <= 0.0 || b.height <= 0.0 || width <= 0.0 || color.3 == 0 {
        return;
    }
    if is_oval {
        f.commands.push(DrawCommand::Outline { bounds: b, radius: -1.0, width, color });
        return;
    }
    let eff_radii = corner_radii.unwrap_or([uniform_radius.max(0.0); 4]);
    if has_active_cut_corner(cut, eff_radii) {
        paint_cut_shape_outline(f, b, eff_radii, cut, width, color);
    } else if let Some(radii) = corner_radii {
        paint_outline_radii(f, b, radii, width, color);
    } else {
        f.commands.push(DrawCommand::Outline { bounds: b, radius: uniform_radius, width, color });
    }
}

pub(super) fn normalize_corner_radii(b: Rect, raw: [f32; 4]) -> [f32; 4] {
    if b.width <= 0.0 || b.height <= 0.0 {
        return [0.0; 4];
    }
    let mut r = [raw[0].max(0.0), raw[1].max(0.0), raw[2].max(0.0), raw[3].max(0.0)];
    let mut scale = 1.0_f32;
    for (sum, limit) in [
        (r[0] + r[1], b.width),
        (r[3] + r[2], b.width),
        (r[0] + r[3], b.height),
        (r[1] + r[2], b.height),
    ] {
        if sum > limit && sum > 0.0 {
            scale = scale.min(limit / sum);
        }
    }
    for v in &mut r {
        *v *= scale;
    }
    let max_h = r[0].max(r[3]) + r[1].max(r[2]);
    let max_v = r[0].max(r[1]) + r[3].max(r[2]);
    let mut q_scale = 1.0_f32;
    if max_h > b.width && max_h > 0.0 {
        q_scale = q_scale.min(b.width / max_h);
    }
    if max_v > b.height && max_v > 0.0 {
        q_scale = q_scale.min(b.height / max_v);
    }
    for v in &mut r {
        *v *= q_scale;
    }
    r
}

/// Paint an asymmetric rounded rectangle using disjoint clipped RoundedRects (zero ABI break).
pub(super) fn paint_rounded_rect_radii(f: &mut Frame<'_>, b: Rect, radii: [f32; 4], color: Color) {
    if b.width <= 0.0 || b.height <= 0.0 || color.3 == 0 {
        return;
    }
    let r = normalize_corner_radii(b, radii);
    if (r[0] - r[1]).abs() < 0.01 && (r[1] - r[2]).abs() < 0.01 && (r[2] - r[3]).abs() < 0.01 {
        f.paint_rect(b, r[0], color);
        return;
    }
    if (r[0] - r[1]).abs() < 0.01 && (r[3] - r[2]).abs() < 0.01 {
        let min_y = b.y + r[0];
        let max_y = b.y + b.height - r[3];
        let my = ((min_y + max_y) * 0.5).round().clamp(min_y, max_y);
        let top_h = (my - b.y).max(0.0);
        let bot_h = (b.y + b.height - my).max(0.0);
        let ext_t = r[0].max(2.0);
        let ext_b = r[3].max(2.0);
        if top_h > 0.0 {
            f.within(Rect { x: b.x, y: b.y, width: b.width, height: top_h }, |f| {
                f.paint_rect(Rect { x: b.x, y: b.y, width: b.width, height: top_h + ext_t }, r[0], color);
            });
        }
        if bot_h > 0.0 {
            f.within(Rect { x: b.x, y: my, width: b.width, height: bot_h }, |f| {
                f.paint_rect(Rect { x: b.x, y: my - ext_b, width: b.width, height: bot_h + ext_b }, r[3], color);
            });
        }
        return;
    }
    if (r[0] - r[3]).abs() < 0.01 && (r[1] - r[2]).abs() < 0.01 {
        let min_x = b.x + r[0];
        let max_x = b.x + b.width - r[1];
        let mx = ((min_x + max_x) * 0.5).round().clamp(min_x, max_x);
        let left_w = (mx - b.x).max(0.0);
        let right_w = (b.x + b.width - mx).max(0.0);
        let ext_l = r[0].max(2.0);
        let ext_r = r[1].max(2.0);
        if left_w > 0.0 {
            f.within(Rect { x: b.x, y: b.y, width: left_w, height: b.height }, |f| {
                f.paint_rect(Rect { x: b.x, y: b.y, width: left_w + ext_l, height: b.height }, r[0], color);
            });
        }
        if right_w > 0.0 {
            f.within(Rect { x: mx, y: b.y, width: right_w, height: b.height }, |f| {
                f.paint_rect(Rect { x: mx - ext_r, y: b.y, width: right_w + ext_r, height: b.height }, r[1], color);
            });
        }
        return;
    }
    let min_x = b.x + r[0].max(r[3]);
    let max_x = b.x + b.width - r[1].max(r[2]);
    let mx = ((min_x + max_x) * 0.5).round().clamp(min_x, max_x);
    let min_y = b.y + r[0].max(r[1]);
    let max_y = b.y + b.height - r[3].max(r[2]);
    let my = ((min_y + max_y) * 0.5).round().clamp(min_y, max_y);
    let lw = (mx - b.x).max(0.0);
    let rw = (b.x + b.width - mx).max(0.0);
    let th = (my - b.y).max(0.0);
    let bh = (b.y + b.height - my).max(0.0);
    let e = [r[0].max(2.0), r[1].max(2.0), r[2].max(2.0), r[3].max(2.0)];
    if lw > 0.0 && th > 0.0 {
        f.within(Rect { x: b.x, y: b.y, width: lw, height: th }, |f| {
            f.paint_rect(Rect { x: b.x, y: b.y, width: lw + e[0], height: th + e[0] }, r[0], color);
        });
    }
    if rw > 0.0 && th > 0.0 {
        f.within(Rect { x: mx, y: b.y, width: rw, height: th }, |f| {
            f.paint_rect(Rect { x: mx - e[1], y: b.y, width: rw + e[1], height: th + e[1] }, r[1], color);
        });
    }
    if rw > 0.0 && bh > 0.0 {
        f.within(Rect { x: mx, y: my, width: rw, height: bh }, |f| {
            f.paint_rect(Rect { x: mx - e[2], y: my - e[2], width: rw + e[2], height: bh + e[2] }, r[2], color);
        });
    }
    if lw > 0.0 && bh > 0.0 {
        f.within(Rect { x: b.x, y: my, width: lw, height: bh }, |f| {
            f.paint_rect(Rect { x: b.x, y: my - e[3], width: lw + e[3], height: bh + e[3] }, r[3], color);
        });
    }
}

/// Paint an asymmetric rounded rectangle outline using disjoint clipped Outlines.
pub(super) fn paint_outline_radii(f: &mut Frame<'_>, b: Rect, radii: [f32; 4], width: f32, color: Color) {
    if b.width <= 0.0 || b.height <= 0.0 || width <= 0.0 || color.3 == 0 {
        return;
    }
    let r = normalize_corner_radii(b, radii);
    if (r[0] - r[1]).abs() < 0.01 && (r[1] - r[2]).abs() < 0.01 && (r[2] - r[3]).abs() < 0.01 {
        f.commands.push(DrawCommand::Outline { bounds: b, radius: r[0], width, color });
        return;
    }
    let d = width.min(b.width * 0.5).min(b.height * 0.5);
    if (r[0] - r[1]).abs() < 0.01 && (r[3] - r[2]).abs() < 0.01 {
        let min_y = b.y + r[0];
        let max_y = b.y + b.height - r[3];
        let my = ((min_y + max_y) * 0.5).clamp(min_y, max_y);
        let top_h = (my - b.y).max(0.0);
        let bot_h = (b.y + b.height - my).max(0.0);
        let pad_t = r[0].max(d) + d;
        let pad_b = r[3].max(d) + d;
        if top_h > 0.0 {
            f.within(Rect { x: b.x, y: b.y, width: b.width, height: top_h }, |f| {
                f.commands.push(DrawCommand::Outline {
                    bounds: Rect { x: b.x, y: b.y, width: b.width, height: top_h + pad_t },
                    radius: r[0], width, color,
                });
            });
        }
        if bot_h > 0.0 {
            f.within(Rect { x: b.x, y: my, width: b.width, height: bot_h }, |f| {
                f.commands.push(DrawCommand::Outline {
                    bounds: Rect { x: b.x, y: my - pad_b, width: b.width, height: bot_h + pad_b },
                    radius: r[3], width, color,
                });
            });
        }
        return;
    }
    if (r[0] - r[3]).abs() < 0.01 && (r[1] - r[2]).abs() < 0.01 {
        let min_x = b.x + r[0];
        let max_x = b.x + b.width - r[1];
        let mx = ((min_x + max_x) * 0.5).clamp(min_x, max_x);
        let left_w = (mx - b.x).max(0.0);
        let right_w = (b.x + b.width - mx).max(0.0);
        let pad_l = r[0].max(d) + d;
        let pad_r = r[1].max(d) + d;
        if left_w > 0.0 {
            f.within(Rect { x: b.x, y: b.y, width: left_w, height: b.height }, |f| {
                f.commands.push(DrawCommand::Outline {
                    bounds: Rect { x: b.x, y: b.y, width: left_w + pad_l, height: b.height },
                    radius: r[0], width, color,
                });
            });
        }
        if right_w > 0.0 {
            f.within(Rect { x: mx, y: b.y, width: right_w, height: b.height }, |f| {
                f.commands.push(DrawCommand::Outline {
                    bounds: Rect { x: mx - pad_r, y: b.y, width: right_w + pad_r, height: b.height },
                    radius: r[1], width, color,
                });
            });
        }
        return;
    }
    let min_x = b.x + r[0].max(r[3]);
    let max_x = b.x + b.width - r[1].max(r[2]);
    let mx = ((min_x + max_x) * 0.5).clamp(min_x, max_x);
    let min_y = b.y + r[0].max(r[1]);
    let max_y = b.y + b.height - r[3].max(r[2]);
    let my = ((min_y + max_y) * 0.5).clamp(min_y, max_y);
    let lw = (mx - b.x).max(0.0);
    let rw = (b.x + b.width - mx).max(0.0);
    let th = (my - b.y).max(0.0);
    let bh = (b.y + b.height - my).max(0.0);
    let p = [r[0].max(d) + d, r[1].max(d) + d, r[2].max(d) + d, r[3].max(d) + d];
    if lw > 0.0 && th > 0.0 {
        f.within(Rect { x: b.x, y: b.y, width: lw, height: th }, |f| {
            f.commands.push(DrawCommand::Outline {
                bounds: Rect { x: b.x, y: b.y, width: lw + p[0], height: th + p[0] },
                radius: r[0], width, color,
            });
        });
    }
    if rw > 0.0 && th > 0.0 {
        f.within(Rect { x: mx, y: b.y, width: rw, height: th }, |f| {
            f.commands.push(DrawCommand::Outline {
                bounds: Rect { x: mx - p[1], y: b.y, width: rw + p[1], height: th + p[1] },
                radius: r[1], width, color,
            });
        });
    }
    if rw > 0.0 && bh > 0.0 {
        f.within(Rect { x: mx, y: my, width: rw, height: bh }, |f| {
            f.commands.push(DrawCommand::Outline {
                bounds: Rect { x: mx - p[2], y: my - p[2], width: rw + p[2], height: bh + p[2] },
                radius: r[2], width, color,
            });
        });
    }
    if lw > 0.0 && bh > 0.0 {
        f.within(Rect { x: b.x, y: my, width: lw, height: bh }, |f| {
            f.commands.push(DrawCommand::Outline {
                bounds: Rect { x: b.x, y: my - p[3], width: lw + p[3], height: bh + p[3] },
                radius: r[3], width, color,
            });
        });
    }
}

/// Paint an asymmetric rounded ripple using disjoint clipped Ripple commands (zero ABI break).
pub(super) fn paint_ripple_radii(
    f: &mut Frame<'_>,
    b: Rect,
    radii: [f32; 4],
    center: [f32; 2],
    radius: f32,
    color: Color,
    soft: bool,
) {
    if b.width <= 0.0 || b.height <= 0.0 || color.3 == 0 {
        return;
    }
    let r = normalize_corner_radii(b, radii);
    if (r[0] - r[1]).abs() < 0.01 && (r[1] - r[2]).abs() < 0.01 && (r[2] - r[3]).abs() < 0.01 {
        f.commands.push(DrawCommand::Ripple {
            bounds: b,
            corner_radius: r[0],
            center,
            radius,
            color,
            soft,
        });
        return;
    }
    if (r[0] - r[1]).abs() < 0.01 && (r[3] - r[2]).abs() < 0.01 {
        let min_y = b.y + r[0];
        let max_y = b.y + b.height - r[3];
        let my = ((min_y + max_y) * 0.5).round().clamp(min_y, max_y);
        let top_h = (my - b.y).max(0.0);
        let bot_h = (b.y + b.height - my).max(0.0);
        let ext_t = r[0].max(2.0);
        let ext_b = r[3].max(2.0);
        if top_h > 0.0 {
            f.within(Rect { x: b.x, y: b.y, width: b.width, height: top_h }, |f| {
                f.commands.push(DrawCommand::Ripple {
                    bounds: Rect { x: b.x, y: b.y, width: b.width, height: top_h + ext_t },
                    corner_radius: r[0],
                    center,
                    radius,
                    color,
                    soft,
                });
            });
        }
        if bot_h > 0.0 {
            f.within(Rect { x: b.x, y: my, width: b.width, height: bot_h }, |f| {
                f.commands.push(DrawCommand::Ripple {
                    bounds: Rect { x: b.x, y: my - ext_b, width: b.width, height: bot_h + ext_b },
                    corner_radius: r[3],
                    center,
                    radius,
                    color,
                    soft,
                });
            });
        }
        return;
    }
    if (r[0] - r[3]).abs() < 0.01 && (r[1] - r[2]).abs() < 0.01 {
        let min_x = b.x + r[0];
        let max_x = b.x + b.width - r[1];
        let mx = ((min_x + max_x) * 0.5).round().clamp(min_x, max_x);
        let left_w = (mx - b.x).max(0.0);
        let right_w = (b.x + b.width - mx).max(0.0);
        let ext_l = r[0].max(2.0);
        let ext_r = r[1].max(2.0);
        if left_w > 0.0 {
            f.within(Rect { x: b.x, y: b.y, width: left_w, height: b.height }, |f| {
                f.commands.push(DrawCommand::Ripple {
                    bounds: Rect { x: b.x, y: b.y, width: left_w + ext_l, height: b.height },
                    corner_radius: r[0],
                    center,
                    radius,
                    color,
                    soft,
                });
            });
        }
        if right_w > 0.0 {
            f.within(Rect { x: mx, y: b.y, width: right_w, height: b.height }, |f| {
                f.commands.push(DrawCommand::Ripple {
                    bounds: Rect { x: mx - ext_r, y: b.y, width: right_w + ext_r, height: b.height },
                    corner_radius: r[1],
                    center,
                    radius,
                    color,
                    soft,
                });
            });
        }
        return;
    }
    let min_x = b.x + r[0].max(r[3]);
    let max_x = b.x + b.width - r[1].max(r[2]);
    let mx = ((min_x + max_x) * 0.5).round().clamp(min_x, max_x);
    let min_y = b.y + r[0].max(r[1]);
    let max_y = b.y + b.height - r[3].max(r[2]);
    let my = ((min_y + max_y) * 0.5).round().clamp(min_y, max_y);
    let lw = (mx - b.x).max(0.0);
    let rw = (b.x + b.width - mx).max(0.0);
    let th = (my - b.y).max(0.0);
    let bh = (b.y + b.height - my).max(0.0);
    let e = [r[0].max(2.0), r[1].max(2.0), r[2].max(2.0), r[3].max(2.0)];
    if lw > 0.0 && th > 0.0 {
        f.within(Rect { x: b.x, y: b.y, width: lw, height: th }, |f| {
            f.commands.push(DrawCommand::Ripple {
                bounds: Rect { x: b.x, y: b.y, width: lw + e[0], height: th + e[0] },
                corner_radius: r[0],
                center,
                radius,
                color,
                soft,
            });
        });
    }
    if rw > 0.0 && th > 0.0 {
        f.within(Rect { x: mx, y: b.y, width: rw, height: th }, |f| {
            f.commands.push(DrawCommand::Ripple {
                bounds: Rect { x: mx - e[1], y: b.y, width: rw + e[1], height: th + e[1] },
                corner_radius: r[1],
                center,
                radius,
                color,
                soft,
            });
        });
    }
    if rw > 0.0 && bh > 0.0 {
        f.within(Rect { x: mx, y: my, width: rw, height: bh }, |f| {
            f.commands.push(DrawCommand::Ripple {
                bounds: Rect { x: mx - e[2], y: my - e[2], width: rw + e[2], height: bh + e[2] },
                corner_radius: r[2],
                center,
                radius,
                color,
                soft,
            });
        });
    }
    if lw > 0.0 && bh > 0.0 {
        f.within(Rect { x: b.x, y: my, width: lw, height: bh }, |f| {
            f.commands.push(DrawCommand::Ripple {
                bounds: Rect { x: b.x, y: my - e[3], width: lw + e[3], height: bh + e[3] },
                corner_radius: r[3],
                center,
                radius,
                color,
                soft,
            });
        });
    }
}

pub(super) fn paint_bg_shape(f: &mut Frame<'_>, b: Rect, is_oval: bool, radii: [f32; 4], color: Color) {
    if is_oval {
        paint_oval(f, b, color);
    } else {
        paint_rounded_rect_radii(f, b, radii, color);
    }
}

pub(super) fn outline_corner_intrusion(cb: Rect, ob: Rect, is_oval: bool, r: [f32; 4]) -> [bool; 4] {
    let clipped = cb.intersect(ob);
    if clipped.width <= 0.0 || clipped.height <= 0.0 {
        return [false; 4];
    }
    if is_oval {
        let cx = ob.x + ob.width * 0.5;
        let cy = ob.y + ob.height * 0.5;
        let rx = (ob.width * 0.5).max(0.001);
        let ry = (ob.height * 0.5).max(0.001);
        let outside = |px: f32, py: f32| {
            let nx = (px - cx) / rx;
            let ny = (py - cy) / ry;
            nx * nx + ny * ny > 1.0 - 1e-4
        };
        return [
            outside(clipped.x, clipped.y),
            outside(clipped.x + clipped.width, clipped.y),
            outside(clipped.x + clipped.width, clipped.y + clipped.height),
            outside(clipped.x, clipped.y + clipped.height),
        ];
    }
    [
        r[0] > 0.01 && clipped.x < ob.x + r[0] && clipped.y < ob.y + r[0],
        r[1] > 0.01 && clipped.x + clipped.width > ob.x + ob.width - r[1] && clipped.y < ob.y + r[1],
        r[2] > 0.01
            && clipped.x + clipped.width > ob.x + ob.width - r[2]
            && clipped.y + clipped.height > ob.y + ob.height - r[2],
        r[3] > 0.01
            && clipped.x < ob.x + r[3]
            && clipped.y + clipped.height > ob.y + ob.height - r[3],
    ]
}

/// Post-process child draw commands in `start..` to clip solid rects and images cleanly
/// against a parent view's rounded or oval outline without altering `DrawCommand` ABI.
pub(super) fn clip_commands_to_rounded_outline(
    f: &mut Frame<'_>,
    start: usize,
    outline: Rect,
    is_oval: bool,
    radii: [f32; 4],
) {
    if outline.width <= 0.0 || outline.height <= 0.0 || start >= f.commands.len() {
        return;
    }
    let r = if is_oval { [0.0; 4] } else { normalize_corner_radii(outline, radii) };
    if !is_oval && r.iter().all(|&v| v <= 0.01) {
        return;
    }
    let needs_rewrite = f.commands[start..].iter().any(|cmd| match cmd {
        DrawCommand::RoundedRect { bounds, .. } | DrawCommand::Image { bounds, .. } => {
            outline_corner_intrusion(*bounds, outline, is_oval, r).into_iter().any(|v| v)
        }
        _ => false,
    });
    if !needs_rewrite {
        return;
    }
    let orig: Vec<DrawCommand> = f.commands.drain(start..).collect();
    for cmd in orig {
        match cmd {
            DrawCommand::RoundedRect { bounds: cb, radius: cr, color } => {
                let intr = outline_corner_intrusion(cb, outline, is_oval, r);
                if !intr.into_iter().any(|v| v) {
                    f.commands.push(DrawCommand::RoundedRect { bounds: cb, radius: cr, color });
                    continue;
                }
                let clipped_cb = cb.intersect(outline);
                if clipped_cb.width <= 0.0 || clipped_cb.height <= 0.0 {
                    continue;
                }
                if cr <= 0.01 || is_oval {
                    f.within(clipped_cb, |f| paint_bg_shape(f, outline, is_oval, r, color));
                } else {
                    let eff = [
                        if intr[0] { cr.max(r[0]) } else { cr },
                        if intr[1] { cr.max(r[1]) } else { cr },
                        if intr[2] { cr.max(r[2]) } else { cr },
                        if intr[3] { cr.max(r[3]) } else { cr },
                    ];
                    paint_rounded_rect_radii(f, clipped_cb, eff, color);
                }
            }
            DrawCommand::Image { bounds: cb, uv, radius: cr, color, image } => {
                let intr = outline_corner_intrusion(cb, outline, is_oval, r);
                if !intr.into_iter().any(|v| v) {
                    f.commands.push(DrawCommand::Image { bounds: cb, uv, radius: cr, color, image });
                    continue;
                }
                let oval_r = outline.width.min(outline.height) * 0.5;
                let eff = [
                    if intr[0] { cr.max(if is_oval { oval_r } else { r[0] }) } else { cr },
                    if intr[1] { cr.max(if is_oval { oval_r } else { r[1] }) } else { cr },
                    if intr[2] { cr.max(if is_oval { oval_r } else { r[2] }) } else { cr },
                    if intr[3] { cr.max(if is_oval { oval_r } else { r[3] }) } else { cr },
                ];
                if (eff[0] - eff[1]).abs() < 0.01
                    && (eff[1] - eff[2]).abs() < 0.01
                    && (eff[2] - eff[3]).abs() < 0.01
                {
                    f.commands.push(DrawCommand::Image { bounds: cb, uv, radius: eff[0], color, image });
                } else if (eff[0] - eff[1]).abs() < 0.01 && (eff[2] - eff[3]).abs() < 0.01 {
                    let min_y = cb.y + eff[0].min(cb.height * 0.5);
                    let max_y = cb.y + cb.height - eff[2].min(cb.height * 0.5);
                    let my = ((min_y + max_y) * 0.5).clamp(cb.y, cb.y + cb.height);
                    let top_h = (my - cb.y).max(0.0);
                    let bot_h = (cb.y + cb.height - my).max(0.0);
                    if top_h > 0.0 {
                        f.within(Rect { x: cb.x, y: cb.y, width: cb.width, height: top_h }, |f| {
                            f.commands.push(DrawCommand::Image { bounds: cb, uv, radius: eff[0], color, image: image.clone() });
                        });
                    }
                    if bot_h > 0.0 {
                        f.within(Rect { x: cb.x, y: my, width: cb.width, height: bot_h }, |f| {
                            f.commands.push(DrawCommand::Image { bounds: cb, uv, radius: eff[2], color, image });
                        });
                    }
                } else {
                    let mx = cb.x + cb.width * 0.5;
                    let my = cb.y + cb.height * 0.5;
                    let lw = (mx - cb.x).max(0.0);
                    let rw = (cb.x + cb.width - mx).max(0.0);
                    let th = (my - cb.y).max(0.0);
                    let bh = (cb.y + cb.height - my).max(0.0);
                    let quads = [
                        (Rect { x: cb.x, y: cb.y, width: lw, height: th }, eff[0]),
                        (Rect { x: mx, y: cb.y, width: rw, height: th }, eff[1]),
                        (Rect { x: mx, y: my, width: rw, height: bh }, eff[2]),
                        (Rect { x: cb.x, y: my, width: lw, height: bh }, eff[3]),
                    ];
                    for (qrect, qrad) in quads {
                        if qrect.width > 0.0 && qrect.height > 0.0 {
                            f.within(qrect, |f| {
                                f.commands.push(DrawCommand::Image { bounds: cb, uv, radius: qrad, color, image: image.clone() });
                            });
                        }
                    }
                }
            }
            other => f.commands.push(other),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum GradientKind {
    Linear { angle: f32 },
    Radial { cx: f32, cy: f32, radius_dp: f32 },
    Sweep { cx: f32, cy: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GradientSpec {
    pub(super) kind: GradientKind,
    pub(super) start: Color,
    pub(super) center: Option<(Color, f32)>,
    pub(super) end: Color,
}

impl GradientSpec {
    pub(super) fn sample(self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        if let Some((mid, pos)) = self.center {
            let pos = pos.clamp(0.0, 1.0);
            if t <= pos {
                self.start.mix(mid, if pos > 0.0 { t / pos } else { 1.0 })
            } else {
                mid.mix(self.end, if pos < 1.0 { (t - pos) / (1.0 - pos) } else { 1.0 })
            }
        } else {
            self.start.mix(self.end, t)
        }
    }
}

pub(super) fn parse_rgba_csv(s: &str) -> Option<Color> {
    let mut parts = s.split(',').map(|p| p.trim().parse::<u8>().ok());
    let c = Color(parts.next()??, parts.next()??, parts.next()??, parts.next()??);
    if parts.next().is_some() {
        return None;
    }
    Some(c)
}

pub(super) fn parse_opt_center_rgba(s: &str) -> Option<Option<(Color, f32)>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Some(None);
    }
    let parts: Vec<&str> = trimmed.split(',').map(str::trim).collect();
    if parts.len() != 4 && parts.len() != 5 {
        return None;
    }
    let c = Color(
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
        parts[3].parse().ok()?,
    );
    let pos = if parts.len() == 5 {
        parts[4].parse::<f32>().ok().filter(|v| v.is_finite())?.clamp(0.0, 1.0)
    } else {
        0.5
    };
    Some(Some((c, pos)))
}

pub(super) fn parse_gradient(anchor: &str) -> Option<GradientSpec> {
    let anchor = anchor.trim();
    if anchor.contains('|') {
        let parts: Vec<&str> = anchor.split('|').map(str::trim).collect();
        if parts.len() == 5 && parts[0].eq_ignore_ascii_case("radial") {
            let mut cx = 0.5_f32;
            let mut cy = 0.5_f32;
            let mut radius_dp = 0.0_f32;
            if !parts[1].is_empty() {
                let p: Vec<&str> = parts[1].split(',').map(str::trim).collect();
                if p.len() > 3 {
                    return None;
                }
                if !p[0].is_empty() {
                    cx = p[0].parse::<f32>().ok().filter(|v| v.is_finite())?;
                }
                if p.len() >= 2 && !p[1].is_empty() {
                    cy = p[1].parse::<f32>().ok().filter(|v| v.is_finite())?;
                }
                if p.len() >= 3 && !p[2].is_empty() {
                    radius_dp = p[2].parse::<f32>().ok().filter(|v| v.is_finite())?;
                }
            }
            return Some(GradientSpec {
                kind: GradientKind::Radial { cx, cy, radius_dp },
                start: parse_rgba_csv(parts[2])?,
                center: parse_opt_center_rgba(parts[3])?,
                end: parse_rgba_csv(parts[4])?,
            });
        }
        if parts.len() == 5 && parts[0].eq_ignore_ascii_case("sweep") {
            let mut cx = 0.5_f32;
            let mut cy = 0.5_f32;
            if !parts[1].is_empty() {
                let p: Vec<&str> = parts[1].split(',').map(str::trim).collect();
                if p.len() > 2 {
                    return None;
                }
                if !p[0].is_empty() {
                    cx = p[0].parse::<f32>().ok().filter(|v| v.is_finite())?;
                }
                if p.len() >= 2 && !p[1].is_empty() {
                    cy = p[1].parse::<f32>().ok().filter(|v| v.is_finite())?;
                }
            }
            return Some(GradientSpec {
                kind: GradientKind::Sweep { cx, cy },
                start: parse_rgba_csv(parts[2])?,
                center: parse_opt_center_rgba(parts[3])?,
                end: parse_rgba_csv(parts[4])?,
            });
        }
        if parts.len() == 4 {
            let angle = parts[0].parse::<f32>().ok().filter(|v| v.is_finite())?;
            return Some(GradientSpec {
                kind: GradientKind::Linear { angle },
                start: parse_rgba_csv(parts[1])?,
                center: parse_opt_center_rgba(parts[2])?,
                end: parse_rgba_csv(parts[3])?,
            });
        }
        if parts.len() == 5 && parts[0].eq_ignore_ascii_case("linear") {
            let angle = parts[1].parse::<f32>().ok().filter(|v| v.is_finite())?;
            return Some(GradientSpec {
                kind: GradientKind::Linear { angle },
                start: parse_rgba_csv(parts[2])?,
                center: parse_opt_center_rgba(parts[3])?,
                end: parse_rgba_csv(parts[4])?,
            });
        }
        return None;
    }
    let parts: Vec<&str> = anchor.split(',').map(str::trim).collect();
    let angle = parts.first()?.parse::<f32>().ok().filter(|v| v.is_finite())?;
    let color_at = |idx: usize| -> Option<Color> {
        Some(Color(
            parts.get(idx)?.parse().ok()?,
            parts.get(idx + 1)?.parse().ok()?,
            parts.get(idx + 2)?.parse().ok()?,
            parts.get(idx + 3)?.parse().ok()?,
        ))
    };
    match parts.len() {
        9 => Some(GradientSpec {
            kind: GradientKind::Linear { angle },
            start: color_at(1)?,
            center: None,
            end: color_at(5)?,
        }),
        13 => Some(GradientSpec {
            kind: GradientKind::Linear { angle },
            start: color_at(1)?,
            center: Some((color_at(5)?, 0.5)),
            end: color_at(9)?,
        }),
        14 => {
            let pos = parts[13].parse::<f32>().ok().filter(|v| v.is_finite())?.clamp(0.0, 1.0);
            Some(GradientSpec {
                kind: GradientKind::Linear { angle },
                start: color_at(1)?,
                center: Some((color_at(5)?, pos)),
                end: color_at(9)?,
            })
        }
        _ => None,
    }
}

/// Rasterize radial, sweep, or cut-corner gradients into a bounded cached `Image` (capped at 256x256)
/// and clip to the view's rounded/oval/cut outline.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_raster_gradient(
    f: &mut Frame<'_>,
    state: &mut LayoutState,
    b: Rect,
    spec: GradientSpec,
    is_oval: bool,
    r: [f32; 4],
    cut: [bool; 4],
    tint: Option<Color>,
) {
    let Some((img_w, img_h)) = raster_dimensions_capped(b.width, b.height, 256) else {
        return;
    };
    let has_cut = !is_oval && has_active_cut_corner(cut, r);
    let mask_tag = if is_oval {
        1u8
    } else if has_cut {
        2u8
    } else {
        0u8
    };
    let sx = img_w as f32 / b.width;
    let sy = img_h as f32 / b.height;
    let r_px = if has_cut {
        let s = sx.min(sy);
        [r[0] * s, r[1] * s, r[2] * s, r[3] * s]
    } else {
        [0.0; 4]
    };
    let mask_radii = r_px.map(|v| (v * 16.0).round().clamp(0.0, u16::MAX as f32) as u16);
    let max_dim = b.width.max(b.height).max(0.001);
    let (kind_tag, p0, p1, p2, resolved_radius_px) = match spec.kind {
        GradientKind::Linear { angle } => (0u8, (angle * 1000.0).round() as i32, 0, 0, 0.0),
        GradientKind::Radial { cx, cy, radius_dp } => {
            let rad_px = if radius_dp > 0.0 {
                state.dp(radius_dp)
            } else {
                b.width.min(b.height) * 0.5
            }
            .max(0.001);
            (
                1u8,
                (cx * 1000.0).round() as i32,
                (cy * 1000.0).round() as i32,
                ((rad_px / max_dim) * 10000.0).round() as i32,
                rad_px,
            )
        }
        GradientKind::Sweep { cx, cy } => (
            2u8,
            (cx * 1000.0).round() as i32,
            (cy * 1000.0).round() as i32,
            0,
            0.0,
        ),
    };
    let key = GradientCacheKey {
        kind_tag,
        w: img_w as u16,
        h: img_h as u16,
        p0,
        p1,
        p2,
        start: [spec.start.0, spec.start.1, spec.start.2, spec.start.3],
        center: spec
            .center
            .map(|(c, pos)| ([c.0, c.1, c.2, c.3], (pos * 1000.0).round().clamp(0.0, 1000.0) as u16)),
        end: [spec.end.0, spec.end.1, spec.end.2, spec.end.3],
        tint: tint.map(|c| [c.0, c.1, c.2, c.3]),
        mask_tag,
        mask_radii,
        mask_cut: if has_cut { cut } else { [false; 4] },
    };
    let image = if let Some(cached) = state.gradient_images.get(&key) {
        cached.clone()
    } else {
        let bytes = (img_w as usize) * (img_h as usize) * 4;
        if state.gradient_images.len() >= 32
            || state.gradient_image_bytes.saturating_add(bytes) > 4 * 1024 * 1024
        {
            state.gradient_images.clear();
            state.gradient_image_bytes = 0;
        }
        let sample_color = |t: f32| {
            let c = spec.sample(t);
            tint.map_or(c, |tc| tint_pixel(c, tc, 1))
        };
        let mut pixels = Vec::with_capacity(bytes);
        let wf = img_w as f32;
        let hf = img_h as f32;
        for py in 0..img_h {
            let py_c = py as f32 + 0.5;
            let v = py_c / hf;
            let ly = v * b.height;
            for px in 0..img_w {
                let px_c = px as f32 + 0.5;
                let u = px_c / wf;
                let lx = u * b.width;
                let t = match spec.kind {
                    GradientKind::Linear { angle } => {
                        let rad = angle.to_radians();
                        let dx = rad.cos();
                        let dy = -rad.sin();
                        let span = (dx.abs() + dy.abs()).max(1e-4);
                        (((u - 0.5) * dx + (v - 0.5) * dy) / span + 0.5).clamp(0.0, 1.0)
                    }
                    GradientKind::Radial { cx, cy, .. } => {
                        let dist = (lx - cx * b.width).hypot(ly - cy * b.height);
                        (dist / resolved_radius_px).clamp(0.0, 1.0)
                    }
                    GradientKind::Sweep { cx, cy } => {
                        let dx = lx - cx * b.width;
                        let dy = ly - cy * b.height;
                        dy.atan2(dx).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU
                    }
                };
                let mut c = sample_color(t);
                if mask_tag == 1 {
                    let rx = (wf * 0.5).max(0.5);
                    let ry = (hf * 0.5).max(0.5);
                    let nx = (px_c - rx) / rx;
                    let ny = (py_c - ry) / ry;
                    let dist_norm = nx.hypot(ny);
                    let grad_scale = (nx / rx).hypot(ny / ry).max(1e-4);
                    let sdf = (dist_norm - 1.0) / grad_scale;
                    let cov = (0.5 - sdf).clamp(0.0, 1.0);
                    c.3 = (c.3 as f32 * cov).round() as u8;
                } else if mask_tag == 2 {
                    let sdf = corner_shape_sdf(px_c, py_c, wf, hf, r_px, cut);
                    let cov = (0.5 - sdf).clamp(0.0, 1.0);
                    c.3 = (c.3 as f32 * cov).round() as u8;
                }
                pixels.extend_from_slice(&[c.0, c.1, c.2, c.3]);
            }
        }
        let img = Image::rgba(img_w, img_h, pixels).expect("gradient raster within bounded size");
        state.gradient_image_bytes += bytes;
        state.gradient_images.insert(key, img.clone());
        img
    };
    let grad_start = f.commands.len();
    f.commands.push(DrawCommand::Image {
        bounds: b,
        uv: [0.0, 0.0, 1.0, 1.0],
        radius: 0.0,
        color: Color(255, 255, 255, 255),
        image,
    });
    if !has_cut {
        clip_commands_to_rounded_outline(f, grad_start, b, is_oval, r);
    }
}

/// Render linear, radial, or sweep `<shape>` gradients clipped to the view's rounded/oval/cut outline.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_gradient(
    f: &mut Frame<'_>,
    state: &mut LayoutState,
    b: Rect,
    spec: GradientSpec,
    is_oval: bool,
    radii: [f32; 4],
    cut: [bool; 4],
    tint: Option<Color>,
) {
    if b.width <= 0.0 || b.height <= 0.0 {
        return;
    }
    let r = if is_oval { [0.0; 4] } else { normalize_corner_radii(b, radii) };
    let angle = match spec.kind {
        GradientKind::Linear { angle } if is_oval || !has_active_cut_corner(cut, r) => angle,
        _ => {
            paint_raster_gradient(f, state, b, spec, is_oval, r, cut, tint);
            return;
        }
    };
    let sample_color = |t: f32| {
        let c = spec.sample(t);
        tint.map_or(c, |tc| tint_pixel(c, tc, 1))
    };
    // Match AOSP GradientDrawable angle convention: 0=L->R, 90=B->T, 180=R->L, 270=T->B.
    let rad = angle.to_radians();
    let dx = rad.cos();
    let dy = -rad.sin();
    if dy.abs() < 1e-4 {
        let steps = (b.width / 3.0).ceil().clamp(16.0, 48.0).min(b.width.ceil().max(1.0)) as usize;
        let left_r = r[0].max(r[3]);
        let right_r = r[1].max(r[2]);
        for i in 0..steps {
            let x0 = b.x + b.width * (i as f32 / steps as f32);
            let x1 = b.x + b.width * ((i + 1) as f32 / steps as f32);
            let u = (i as f32 + 0.5) / steps as f32;
            let t = if dx >= 0.0 { u } else { 1.0 - u };
            let color = sample_color(t);
            if color.3 == 0 {
                continue;
            }
            let strip = Rect { x: x0, y: b.y, width: x1 - x0, height: b.height };
            if is_oval || x0 < b.x + left_r || x1 > b.x + b.width - right_r {
                f.within(strip, |f| paint_bg_shape(f, b, is_oval, r, color));
            } else {
                f.paint_rect(strip, 0.0, color);
            }
        }
    } else if dx.abs() < 1e-4 {
        let steps = (b.height / 3.0).ceil().clamp(16.0, 48.0).min(b.height.ceil().max(1.0)) as usize;
        let top_r = r[0].max(r[1]);
        let bot_r = r[3].max(r[2]);
        for i in 0..steps {
            let y0 = b.y + b.height * (i as f32 / steps as f32);
            let y1 = b.y + b.height * ((i + 1) as f32 / steps as f32);
            let v = (i as f32 + 0.5) / steps as f32;
            let t = if dy >= 0.0 { v } else { 1.0 - v };
            let color = sample_color(t);
            if color.3 == 0 {
                continue;
            }
            let strip = Rect { x: b.x, y: y0, width: b.width, height: y1 - y0 };
            if is_oval || y0 < b.y + top_r || y1 > b.y + b.height - bot_r {
                f.within(strip, |f| paint_bg_shape(f, b, is_oval, r, color));
            } else {
                f.paint_rect(strip, 0.0, color);
            }
        }
    } else {
        let nx = (b.width / 8.0).ceil().clamp(8.0, 16.0).min(b.width.ceil().max(1.0)) as usize;
        let ny = (b.height / 8.0).ceil().clamp(8.0, 16.0).min(b.height.ceil().max(1.0)) as usize;
        let span = dx.abs() + dy.abs();
        for iy in 0..ny {
            let y0 = b.y + b.height * (iy as f32 / ny as f32);
            let y1 = b.y + b.height * ((iy + 1) as f32 / ny as f32);
            let v = (iy as f32 + 0.5) / ny as f32;
            for ix in 0..nx {
                let x0 = b.x + b.width * (ix as f32 / nx as f32);
                let x1 = b.x + b.width * ((ix + 1) as f32 / nx as f32);
                let u = (ix as f32 + 0.5) / nx as f32;
                let t = (((u - 0.5) * dx + (v - 0.5) * dy) / span + 0.5).clamp(0.0, 1.0);
                let color = sample_color(t);
                if color.3 == 0 {
                    continue;
                }
                let tile = Rect { x: x0, y: y0, width: x1 - x0, height: y1 - y0 };
                let in_tl = r[0] > 0.0 && x0 < b.x + r[0] && y0 < b.y + r[0];
                let in_tr = r[1] > 0.0 && x1 > b.x + b.width - r[1] && y0 < b.y + r[1];
                let in_br = r[2] > 0.0 && x1 > b.x + b.width - r[2] && y1 > b.y + b.height - r[2];
                let in_bl = r[3] > 0.0 && x0 < b.x + r[3] && y1 > b.y + b.height - r[3];
                if is_oval || in_tl || in_tr || in_br || in_bl {
                    f.within(tile, |f| paint_bg_shape(f, b, is_oval, r, color));
                } else {
                    f.paint_rect(tile, 0.0, color);
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum LayerRadius {
    Uniform(f32),
    Corners([f32; 4]),
}

#[derive(Clone, Copy)]
pub(super) struct DrawableLayerSpec {
    pub(super) inset: [f32; 4],
    pub(super) radius: LayerRadius,
    pub(super) fill: Option<Color>,
    pub(super) stroke: Option<(f32, Color)>,
}

pub(super) fn parse_drawable_layer(anchor: &str) -> Option<DrawableLayerSpec> {
    let parts: Vec<&str> = anchor.split('|').map(str::trim).collect();
    if parts.len() < 3 || parts.len() > 4 {
        return None;
    }
    let mut inset_iter = parts[0].split(',').map(|p| p.trim().parse::<f32>().ok());
    let l = inset_iter.next()??;
    let t = inset_iter.next()??;
    let r = inset_iter.next()??;
    let b = inset_iter.next()??;
    if inset_iter.next().is_some() || ![l, t, r, b].iter().all(|v| v.is_finite()) {
        return None;
    }
    let radius = if parts[1].contains(',') {
        LayerRadius::Corners(parse_corner_radii(parts[1])?)
    } else {
        let rv = parts[1].parse::<f32>().ok().filter(|v| v.is_finite())?;
        LayerRadius::Uniform(rv)
    };
    let fill = if parts[2].is_empty() || parts[2].eq_ignore_ascii_case("none") {
        None
    } else {
        let mut rgba = parts[2].split(',').map(|p| p.trim().parse::<u8>().ok());
        let c = Color(rgba.next()??, rgba.next()??, rgba.next()??, rgba.next()??);
        if rgba.next().is_some() {
            return None;
        }
        (c.3 > 0).then_some(c)
    };
    let stroke = if let Some(sp) = parts.get(3).filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("none")) {
        let mut iter = sp.split(',').map(str::trim);
        let sw = iter.next()?.parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0.0)?;
        let sc = Color(
            iter.next()?.parse().ok()?,
            iter.next()?.parse().ok()?,
            iter.next()?.parse().ok()?,
            iter.next()?.parse().ok()?,
        );
        if iter.next().is_some() {
            return None;
        }
        (sw > 0.0 && sc.3 > 0).then_some((sw, sc))
    } else {
        None
    };
    Some(DrawableLayerSpec { inset: [l, t, r, b], radius, fill, stroke })
}

pub(super) fn paint_drawable_layer(f: &mut Frame<'_>, b: Rect, layer: DrawableLayerSpec, tint: Option<Color>, density: f32) {
    let [l, t, r, bottom] = layer.inset.map(|v| v * density);
    let rect = Rect {
        x: b.x + l,
        y: b.y + t,
        width: (b.width - l - r).max(0.0),
        height: (b.height - t - bottom).max(0.0),
    };
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let radius = match layer.radius {
        LayerRadius::Uniform(rad) => LayerRadius::Uniform(if rad < 0.0 { rad } else { rad * density }),
        LayerRadius::Corners(radii) => LayerRadius::Corners(radii.map(|v| v * density)),
    };
    let draw_layer = |f: &mut Frame<'_>| {
        if let Some(fill) = layer.fill {
            let color = tint.map_or(fill, |tc| tint_pixel(fill, tc, 1));
            match radius {
                LayerRadius::Uniform(rad) if rad < 0.0 => paint_oval(f, rect, color),
                LayerRadius::Uniform(rad) => f.paint_rect(rect, rad, color),
                LayerRadius::Corners(radii) => paint_rounded_rect_radii(f, rect, radii, color),
            }
        }
        if let Some((sw, sc)) = layer.stroke {
            let sw = sw * density;
            match radius {
                LayerRadius::Uniform(rad) => {
                    f.commands.push(DrawCommand::Outline { bounds: rect, radius: rad, width: sw, color: sc });
                }
                LayerRadius::Corners(radii) => {
                    paint_outline_radii(f, rect, radii, sw, sc);
                }
            }
        }
    };
    if l < 0.0 || t < 0.0 || r < 0.0 || bottom < 0.0 {
        f.within(b, draw_layer);
    } else {
        draw_layer(f);
    }
}

#[derive(Clone, Copy)]
pub(super) struct StatefulDrawableLayerSpec {
    pub(super) required: u8,
    pub(super) forbidden: u8,
    pub(super) layer: DrawableLayerSpec,
}

pub(super) fn parse_stateful_drawable_layer(anchor: &str) -> Option<StatefulDrawableLayerSpec> {
    let (mask_part, layer_part) = anchor.split_once('|')?;
    let (req_str, forbid_str) = mask_part.split_once(',')?;
    let required = req_str.trim().parse::<u8>().ok()?;
    let forbidden = forbid_str.trim().parse::<u8>().ok()?;
    let layer = parse_drawable_layer(layer_part)?;
    Some(StatefulDrawableLayerSpec { required, forbidden, layer })
}

pub(super) struct AnimationListFrame<'a> {
    pub(super) img_key: &'a str,
    pub(super) duration_ms: u64,
}

pub(super) struct AnimationListSpec<'a> {
    pub(super) oneshot: bool,
    pub(super) frames: Vec<AnimationListFrame<'a>>,
    pub(super) total_ms: u64,
}

pub(super) fn parse_animation_list(anchor: &str) -> Option<AnimationListSpec<'_>> {
    let (oneshot_raw, frames_raw) = anchor.split_once('|')?;
    let oneshot = matches!(oneshot_raw.trim().to_ascii_lowercase().as_str(), "1" | "true");
    let mut frames = Vec::new();
    let mut total_ms = 0u64;
    for item in frames_raw.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (key_raw, dur_raw) = item.rsplit_once(':')?;
        let img_key = key_raw.trim();
        let duration_ms = dur_raw.trim().parse::<u64>().ok().filter(|&d| d > 0)?;
        if img_key.is_empty() {
            continue;
        }
        frames.push(AnimationListFrame { img_key, duration_ms });
        total_ms = total_ms.saturating_add(duration_ms);
    }
    if frames.is_empty() || total_ms == 0 {
        return None;
    }
    Some(AnimationListSpec { oneshot, frames, total_ms })
}

pub(super) fn select_animation_list_frame<'a>(
    spec: &AnimationListSpec<'a>,
    now_sec: f64,
    reduced_motion: bool,
) -> (&'a str, Option<f64>) {
    if reduced_motion || spec.frames.len() <= 1 {
        return (spec.frames[0].img_key, None);
    }
    let now = if now_sec.is_finite() { now_sec.max(0.0) } else { 0.0 };
    let elapsed_ms = (now * 1000.0 + 1e-6).floor() as u64;
    if spec.oneshot {
        if elapsed_ms >= spec.total_ms {
            return (spec.frames.last().unwrap().img_key, None);
        }
        let mut acc = 0u64;
        let last_idx = spec.frames.len() - 1;
        for (i, frame) in spec.frames.iter().enumerate() {
            acc = acc.saturating_add(frame.duration_ms);
            if elapsed_ms < acc {
                let next_wake = if i == last_idx {
                    None
                } else {
                    Some((acc as f64 / 1000.0).max(now + 0.001))
                };
                return (frame.img_key, next_wake);
            }
        }
        return (spec.frames[last_idx].img_key, None);
    }
    let cycle = elapsed_ms / spec.total_ms;
    let in_cycle = elapsed_ms % spec.total_ms;
    let mut acc = 0u64;
    for frame in &spec.frames {
        acc = acc.saturating_add(frame.duration_ms);
        if in_cycle < acc {
            let boundary_ms = cycle.saturating_mul(spec.total_ms).saturating_add(acc);
            let next_wake = (boundary_ms as f64 / 1000.0).max(now + 0.001);
            return (frame.img_key, Some(next_wake));
        }
    }
    (spec.frames[0].img_key, Some(now + spec.frames[0].duration_ms as f64 / 1000.0))
}

pub(super) struct AnimatedVectorSpec<'a> {
    pub(super) is_src: bool,
    pub(super) repeat: bool,
    pub(super) dur_ms: u64,
    pub(super) frames: Vec<&'a str>,
}

pub(super) fn parse_animated_vector(anchor: &str) -> Option<AnimatedVectorSpec<'_>> {
    let (header, frames_raw) = anchor.split_once('|')?;
    let mut parts = header.split(':');
    let ctx_raw = parts.next()?.trim().to_ascii_lowercase();
    let is_src = match ctx_raw.as_str() {
        "src" => true,
        "bg" => false,
        _ => return None,
    };
    let repeat = match parts.next()?.trim().to_ascii_lowercase().as_str() {
        "1" | "true" => true,
        "0" | "false" => false,
        _ => return None,
    };
    let dur_raw = parts.next()?.trim();
    let dur_ms = dur_raw
        .parse::<u64>()
        .ok()
        .or_else(|| {
            dur_raw
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v > 0.0)
                .map(|v| v.round().max(1.0) as u64)
        })
        .filter(|&d| d > 0)?;
    if parts.next().is_some() {
        return None;
    }
    let frames: Vec<&str> = frames_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if frames.is_empty() {
        return None;
    }
    Some(AnimatedVectorSpec { is_src, repeat, dur_ms, frames })
}

pub(super) fn select_animated_vector_frame<'a>(
    spec: &AnimatedVectorSpec<'a>,
    now_sec: f64,
    reduced_motion: bool,
) -> (&'a str, Option<f64>) {
    let count = spec.frames.len();
    if count <= 1 {
        return (spec.frames[0], None);
    }
    if reduced_motion {
        let settled = if spec.repeat { 0 } else { count - 1 };
        return (spec.frames[settled], None);
    }
    let now = if now_sec.is_finite() { now_sec.max(0.0) } else { 0.0 };
    let dur_ms_f = spec.dur_ms as f64;
    let elapsed_ms_f = now * 1000.0 + 1e-6;
    if !spec.repeat {
        if elapsed_ms_f >= dur_ms_f {
            return (spec.frames[count - 1], None);
        }
        let idx = ((elapsed_ms_f / dur_ms_f) * count as f64).floor() as usize;
        let idx = idx.min(count - 1);
        let next_wake = if idx == count - 1 {
            None
        } else {
            let next_ms = ((idx + 1) as f64 * dur_ms_f) / count as f64;
            Some((next_ms / 1000.0).max(now + 0.001))
        };
        return (spec.frames[idx], next_wake);
    }
    let cycle = (elapsed_ms_f / dur_ms_f).floor();
    let in_cycle = (elapsed_ms_f - cycle * dur_ms_f).max(0.0);
    let idx = ((in_cycle / dur_ms_f) * count as f64).floor() as usize;
    let idx = idx.min(count - 1);
    let next_in_cycle_ms = ((idx + 1) as f64 * dur_ms_f) / count as f64;
    let boundary_sec = (cycle * dur_ms_f + next_in_cycle_ms) / 1000.0;
    (spec.frames[idx], Some(boundary_sec.max(now + 0.001)))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CompoundAnimSlot {
    Start,
    Top,
    End,
    Bottom,
    Left,
    Right,
    Button,
    Tick,
    Indicator,
}

pub(super) fn parse_compound_animation_list(anchor: &str) -> Option<(CompoundAnimSlot, AnimationListSpec<'_>)> {
    let (slot_raw, rest) = anchor.split_once(':')?;
    let slot = match slot_raw.trim().to_ascii_lowercase().as_str() {
        "start" => CompoundAnimSlot::Start,
        "top" => CompoundAnimSlot::Top,
        "end" => CompoundAnimSlot::End,
        "bottom" => CompoundAnimSlot::Bottom,
        "left" => CompoundAnimSlot::Left,
        "right" => CompoundAnimSlot::Right,
        "button" => CompoundAnimSlot::Button,
        "tick" => CompoundAnimSlot::Tick,
        "indicator" => CompoundAnimSlot::Indicator,
        _ => return None,
    };
    let spec = parse_animation_list(rest)?;
    Some((slot, spec))
}

pub(super) struct AnimatedSelectorTransition<'a> {
    pub(super) is_src: bool,
    pub(super) from_mask: u8,
    pub(super) to_mask: u8,
    pub(super) reversible: bool,
    pub(super) frames: Vec<AnimationListFrame<'a>>,
    pub(super) total_ms: u64,
}

pub(super) fn parse_animated_selector_transition(anchor: &str) -> Option<AnimatedSelectorTransition<'_>> {
    let (header, frames_raw) = anchor.split_once('|')?;
    let mut parts = header.split(':');
    let ctx_raw = parts.next()?.trim().to_ascii_lowercase();
    let is_src = match ctx_raw.as_str() {
        "src" => true,
        "bg" => false,
        _ => return None,
    };
    let from_mask = parts.next()?.trim().parse::<u8>().ok()? & 0x7F;
    let to_mask = parts.next()?.trim().parse::<u8>().ok()? & 0x7F;
    let reversible = matches!(parts.next()?.trim().to_ascii_lowercase().as_str(), "1" | "true");
    let mut frames = Vec::new();
    let mut total_ms = 0u64;
    for item in frames_raw.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (key_raw, dur_raw) = item.rsplit_once(':')?;
        let img_key = key_raw.trim();
        let duration_ms = dur_raw.trim().parse::<u64>().ok().filter(|&d| d > 0)?;
        if img_key.is_empty() {
            continue;
        }
        frames.push(AnimationListFrame { img_key, duration_ms });
        total_ms = total_ms.saturating_add(duration_ms);
    }
    if frames.is_empty() || total_ms == 0 {
        return None;
    }
    Some(AnimatedSelectorTransition {
        is_src,
        from_mask,
        to_mask,
        reversible,
        frames,
        total_ms,
    })
}

pub(super) fn matches_animated_selector_transition(old: u8, new: u8, from_mask: u8, to_mask: u8) -> bool {
    if old == new || from_mask == to_mask {
        return false;
    }
    if from_mask == 0 {
        (old & to_mask) != to_mask && (new & to_mask) == to_mask
    } else if to_mask == 0 {
        (old & from_mask) == from_mask && (new & from_mask) != from_mask
    } else if (from_mask & to_mask) == to_mask {
        (old & from_mask) == from_mask && (new & from_mask) != from_mask && (new & to_mask) == to_mask
    } else {
        (old & from_mask) == from_mask && (old & to_mask) != to_mask && (new & to_mask) == to_mask
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DynamicDrawableTarget {
    Bg,
    Src,
}

pub(super) struct LevelDrawableItem<'a> {
    pub(super) min: u32,
    pub(super) max: u32,
    pub(super) img_key: &'a str,
}

pub(super) enum DynamicDrawableSpec<'a> {
    Level(Vec<LevelDrawableItem<'a>>),
    Clip {
        vertical: bool,
        gravity: &'a str,
        img_key: &'a str,
    },
    Rotate {
        from_deg: f32,
        to_deg: f32,
        px: f32,
        py: f32,
        img_key: &'a str,
    },
    Scale {
        sw_frac: f32,
        sh_frac: f32,
        gravity: &'a str,
        img_key: &'a str,
    },
    Transition {
        dur_ms: f64,
        img_key_0: &'a str,
        img_key_1: &'a str,
    },
}

pub(super) fn parse_dynamic_fraction(raw: &str) -> Option<f32> {
    let s = raw.trim();
    if s.eq_ignore_ascii_case("center") {
        return Some(0.5);
    }
    if let Some(pct) = s.strip_suffix('%') {
        let v = pct.trim().parse::<f32>().ok()?;
        return v.is_finite().then_some(v * 0.01);
    }
    let v = s.parse::<f32>().ok()?;
    v.is_finite().then_some(v)
}

pub(super) fn parse_dynamic_drawable(
    anchor: &str,
    default_target: DynamicDrawableTarget,
) -> Option<(DynamicDrawableTarget, DynamicDrawableSpec<'_>)> {
    let trimmed = anchor.trim();
    let (target, payload) = if let Some(rest) = trimmed.strip_prefix("bg:") {
        (DynamicDrawableTarget::Bg, rest)
    } else if let Some(rest) = trimmed.strip_prefix("src:") {
        (DynamicDrawableTarget::Src, rest)
    } else {
        (default_target, trimmed)
    };
    let (kind_raw, rest) = payload.split_once('|')?;
    let spec = match kind_raw.trim().to_ascii_lowercase().as_str() {
        "level" => {
            let mut items = Vec::new();
            for entry in rest.split(',') {
                let entry = entry.trim();
                if entry.is_empty() {
                    continue;
                }
                let mut parts = entry.splitn(3, ':');
                let min = parts.next()?.trim().parse::<u32>().ok()?;
                let max = parts.next()?.trim().parse::<u32>().ok()?;
                let img_key = parts.next()?.trim();
                if img_key.is_empty() {
                    continue;
                }
                items.push(LevelDrawableItem { min, max, img_key });
            }
            if items.is_empty() {
                return None;
            }
            DynamicDrawableSpec::Level(items)
        }
        "clip" => {
            let mut parts = rest.splitn(3, '|');
            let orient = parts.next()?.trim();
            let gravity = parts.next()?.trim();
            let img_key = parts.next()?.trim();
            if img_key.is_empty() {
                return None;
            }
            let vertical = matches!(orient.to_ascii_lowercase().as_str(), "v" | "vertical" | "2");
            DynamicDrawableSpec::Clip { vertical, gravity, img_key }
        }
        "rotate" => {
            let mut parts = rest.splitn(3, '|');
            let degs = parts.next()?;
            let pivot = parts.next()?;
            let img_key = parts.next()?.trim();
            if img_key.is_empty() {
                return None;
            }
            let (fd, td) = degs.split_once(',')?;
            let from_deg = fd.trim().parse::<f32>().ok().filter(|v| v.is_finite())?;
            let to_deg = td.trim().parse::<f32>().ok().filter(|v| v.is_finite())?;
            let (px_s, py_s) = pivot.split_once(',').unwrap_or((pivot, "0.5"));
            let px = parse_dynamic_fraction(px_s).unwrap_or(0.5);
            let py = parse_dynamic_fraction(py_s).unwrap_or(0.5);
            DynamicDrawableSpec::Rotate { from_deg, to_deg, px, py, img_key }
        }
        "scale" => {
            let mut parts = rest.splitn(3, '|');
            let fracs = parts.next()?;
            let gravity = parts.next()?.trim();
            let img_key = parts.next()?.trim();
            if img_key.is_empty() {
                return None;
            }
            let (sw_s, sh_s) = fracs.split_once(',').unwrap_or((fracs, fracs));
            let sw_frac = parse_dynamic_fraction(sw_s)?.max(0.0);
            let sh_frac = parse_dynamic_fraction(sh_s)?.max(0.0);
            DynamicDrawableSpec::Scale { sw_frac, sh_frac, gravity, img_key }
        }
        "transition" => {
            let (dur_raw, imgs_raw) = rest.split_once('|')?;
            let dur_ms = dur_raw.trim().parse::<f64>().ok().filter(|v| v.is_finite() && *v >= 0.0)?;
            let (k0, k1) = imgs_raw.split_once(',')?;
            let img_key_0 = k0.trim();
            let img_key_1 = k1.trim();
            if img_key_0.is_empty() && img_key_1.is_empty() {
                return None;
            }
            DynamicDrawableSpec::Transition { dur_ms, img_key_0, img_key_1 }
        }
        _ => return None,
    };
    Some((target, spec))
}

pub(super) fn select_level_drawable_key<'a>(items: &[LevelDrawableItem<'a>], level: u32) -> &'a str {
    items
        .iter()
        .find(|item| item.min <= level && level <= item.max)
        .unwrap_or(&items[0])
        .img_key
}

pub(super) fn dynamic_clip_rect(dst: Rect, vertical: bool, gravity: &str, frac: f32, rtl: bool) -> Rect {
    let frac = frac.clamp(0.0, 1.0);
    let g = gravity.trim().to_ascii_lowercase();
    if vertical {
        let ch = dst.height * frac;
        let y = if g.contains("bottom") || g == "end" {
            dst.y + dst.height - ch
        } else if g.contains("center") {
            dst.y + (dst.height - ch) * 0.5
        } else {
            dst.y
        };
        Rect { x: dst.x, y, width: dst.width, height: ch }
    } else {
        let cw = dst.width * frac;
        let x = if g == "start" {
            if rtl { dst.x + dst.width - cw } else { dst.x }
        } else if g == "end" {
            if rtl { dst.x } else { dst.x + dst.width - cw }
        } else if g.contains("right") {
            dst.x + dst.width - cw
        } else if g.contains("left") {
            dst.x
        } else if g.contains("center") {
            dst.x + (dst.width - cw) * 0.5
        } else if rtl {
            dst.x + dst.width - cw
        } else {
            dst.x
        };
        Rect { x, y: dst.y, width: cw, height: dst.height }
    }
}

pub(super) fn dynamic_scale_pivot(gravity: &str, rtl: bool) -> [f32; 2] {
    let mut px = 0.5_f32;
    let mut py = 0.5_f32;
    for tok in gravity.split(['|', ',', ' ']).map(str::trim).filter(|s| !s.is_empty()) {
        match tok.to_ascii_lowercase().as_str() {
            "start" => px = if rtl { 1.0 } else { 0.0 },
            "end" => px = if rtl { 0.0 } else { 1.0 },
            "left" => px = 0.0,
            "right" => px = 1.0,
            "top" => py = 0.0,
            "bottom" => py = 1.0,
            "center_horizontal" => px = 0.5,
            "center_vertical" => py = 0.5,
            "center" => {
                px = 0.5;
                py = 0.5;
            }
            _ => {}
        }
    }
    [px, py]
}

pub(super) fn sample_drawable_transition(
    f: &mut Frame<'_>,
    b: Rect,
    state: &mut LayoutState,
    view_key: &str,
    slot: &str,
    dur_ms: f64,
) -> f32 {
    let active_now = state.drawable_transition_active.get(view_key).copied().unwrap_or(false);
    let dur_sec = (dur_ms / 1000.0).max(0.0);
    let now = f.time();
    let motion_key = format!("{view_key}:{slot}");
    let mut tm = state.drawable_transition_motion.get(&motion_key).copied().unwrap_or_default();
    let instant = f.reduced_motion() || dur_sec <= 0.0;
    let target = if active_now { 1.0_f32 } else { 0.0_f32 };
    if !tm.initialized {
        tm.initialized = true;
        tm.last_active = active_now;
        tm.progress = target;
        tm.anim = None;
    } else {
        if let Some((start, from, to, dur)) = tm.anim {
            if instant || dur <= 0.0 {
                tm.progress = to;
                tm.anim = None;
            } else {
                let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                tm.progress = (from + (to - from) * p).clamp(0.0, 1.0);
                if p >= 1.0 {
                    tm.progress = to;
                    tm.anim = None;
                } else {
                    f.continuous(b);
                }
            }
        }
        if active_now != tm.last_active {
            tm.last_active = active_now;
            if instant || (tm.progress - target).abs() <= 0.001 {
                tm.progress = target;
                tm.anim = None;
            } else {
                let rem = (dur_sec * (target - tm.progress).abs() as f64).max(0.001);
                tm.anim = Some((now, tm.progress, target, rem));
                f.continuous(b);
            }
        }
    }
    state.drawable_transition_motion.insert(motion_key, tm);
    tm.progress
}

pub(super) fn paint_dynamic_bg_drawable(
    f: &mut Frame<'_>,
    state: &mut LayoutState,
    b: Rect,
    view_key: &str,
    spec: &DynamicDrawableSpec<'_>,
    background_tint: Option<Color>,
) {
    if b.width <= 0.0 || b.height <= 0.0 {
        return;
    }
    let level = state.drawable_level.get(view_key).copied().unwrap_or(0);
    let rtl = state.rtl;
    let draw_key = |f: &mut Frame<'_>, state: &mut LayoutState, img_key: &str| {
        if img_key.is_empty() {
            return;
        }
        let Some(img) = state.images.get(img_key).cloned() else { return };
        let tinted = tint_image(state, &img, background_tint, 1);
        f.commands.push(DrawCommand::Image {
            bounds: b,
            uv: [0.0, 0.0, 1.0, 1.0],
            radius: 0.0,
            color: Color(255, 255, 255, 255),
            image: tinted,
        });
    };
    match spec {
        DynamicDrawableSpec::Level(items) => {
            draw_key(f, state, select_level_drawable_key(items, level));
        }
        DynamicDrawableSpec::Clip { vertical, gravity, img_key } => {
            let frac = level.min(10000) as f32 / 10000.0;
            if frac <= 0.0 {
                return;
            }
            if frac < 1.0 {
                let clip = dynamic_clip_rect(b, *vertical, gravity, frac, rtl).intersect(f.clip);
                if clip.width > 0.0 && clip.height > 0.0 {
                    f.commands.push(DrawCommand::PushClip(clip));
                    draw_key(f, state, img_key);
                    f.commands.push(DrawCommand::PopClip);
                }
            } else {
                draw_key(f, state, img_key);
            }
        }
        DynamicDrawableSpec::Rotate { from_deg, to_deg, px, py, img_key } => {
            let t = level.min(10000) as f32 / 10000.0;
            let deg = from_deg + (to_deg - from_deg) * t;
            let origin = [b.x + b.width * px, b.y + b.height * py];
            f.push_transform(origin, [0.0, 0.0], [1.0, 1.0], deg.to_radians());
            draw_key(f, state, img_key);
            f.pop_transform();
        }
        DynamicDrawableSpec::Scale { sw_frac, sh_frac, gravity, img_key } => {
            let t = level.min(10000) as f32 / 10000.0;
            let sx = 1.0 - sw_frac + sw_frac * t;
            let sy = 1.0 - sh_frac + sh_frac * t;
            if sx <= 1e-4 || sy <= 1e-4 {
                return;
            }
            let [px, py] = dynamic_scale_pivot(gravity, rtl);
            let origin = [b.x + b.width * px, b.y + b.height * py];
            f.push_transform(origin, [0.0, 0.0], [sx, sy], 0.0);
            draw_key(f, state, img_key);
            f.pop_transform();
        }
        DynamicDrawableSpec::Transition { dur_ms, img_key_0, img_key_1 } => {
            let t = sample_drawable_transition(f, b, state, view_key, "bg", *dur_ms);
            if t <= 0.001 {
                draw_key(f, state, img_key_0);
            } else if t >= 0.999 {
                draw_key(f, state, img_key_1);
            } else {
                f.push_layer(b, 1.0 - t);
                draw_key(f, state, img_key_0);
                f.pop_layer();
                f.push_layer(b, t);
                draw_key(f, state, img_key_1);
                f.pop_layer();
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_dynamic_src_drawable(
    f: &mut Frame<'_>,
    state: &mut LayoutState,
    inner: Rect,
    clip_box: Rect,
    view_key: &str,
    spec: &DynamicDrawableSpec<'_>,
    effective_scale: u8,
    tint: Option<Color>,
    tint_mode: u8,
    alpha_color: Color,
    mirror: bool,
) {
    let level = state.drawable_level.get(view_key).copied().unwrap_or(0);
    let rtl = state.rtl;
    let matrix = state.image_matrices.get(view_key).copied();
    let resolve_geom = |state: &LayoutState, img_key: &str| -> Option<(Image, Rect, [f32; 4])> {
        if img_key.is_empty() {
            return None;
        }
        let img = state.images.get(img_key).cloned()?;
        let density = state.image_scales.get(img_key).copied().unwrap_or(1.0) * state.effective_density();
        let (bounds, uv) = image_geometry(inner, clip_box, &img, effective_scale, density, matrix)?;
        Some((img, bounds, uv))
    };
    let draw_img = |f: &mut Frame<'_>, state: &mut LayoutState, img: &Image, bounds: Rect, uv: [f32; 4]| {
        let tinted = tint_image(state, img, tint, tint_mode);
        if mirror {
            f.push_transform(
                [bounds.x + bounds.width * 0.5, bounds.y + bounds.height * 0.5],
                [0.0, 0.0],
                [-1.0, 1.0],
                0.0,
            );
        }
        f.commands.push(DrawCommand::Image { bounds, uv, radius: 0.0, color: alpha_color, image: tinted });
        if mirror {
            f.pop_transform();
        }
    };
    match spec {
        DynamicDrawableSpec::Level(items) => {
            let key = select_level_drawable_key(items, level);
            if let Some((img, bounds, uv)) = resolve_geom(state, key)
                && bounds.intersect(f.clip).width > 0.0
                && bounds.intersect(f.clip).height > 0.0
            {
                draw_img(f, state, &img, bounds, uv);
            }
        }
        DynamicDrawableSpec::Clip { vertical, gravity, img_key } => {
            let frac = level.min(10000) as f32 / 10000.0;
            if frac <= 0.0 {
                return;
            }
            let Some((img, bounds, uv)) = resolve_geom(state, img_key) else { return };
            if frac < 1.0 {
                let clip = dynamic_clip_rect(bounds, *vertical, gravity, frac, rtl).intersect(f.clip);
                if clip.width > 0.0 && clip.height > 0.0 {
                    f.commands.push(DrawCommand::PushClip(clip));
                    draw_img(f, state, &img, bounds, uv);
                    f.commands.push(DrawCommand::PopClip);
                }
            } else if bounds.intersect(f.clip).width > 0.0 && bounds.intersect(f.clip).height > 0.0 {
                draw_img(f, state, &img, bounds, uv);
            }
        }
        DynamicDrawableSpec::Rotate { from_deg, to_deg, px, py, img_key } => {
            let Some((img, bounds, uv)) = resolve_geom(state, img_key) else { return };
            let t = level.min(10000) as f32 / 10000.0;
            let deg = from_deg + (to_deg - from_deg) * t;
            let origin = [bounds.x + bounds.width * px, bounds.y + bounds.height * py];
            f.push_transform(origin, [0.0, 0.0], [1.0, 1.0], deg.to_radians());
            draw_img(f, state, &img, bounds, uv);
            f.pop_transform();
        }
        DynamicDrawableSpec::Scale { sw_frac, sh_frac, gravity, img_key } => {
            let t = level.min(10000) as f32 / 10000.0;
            let sx = 1.0 - sw_frac + sw_frac * t;
            let sy = 1.0 - sh_frac + sh_frac * t;
            if sx <= 1e-4 || sy <= 1e-4 {
                return;
            }
            let Some((img, bounds, uv)) = resolve_geom(state, img_key) else { return };
            let [px, py] = dynamic_scale_pivot(gravity, rtl);
            let origin = [bounds.x + bounds.width * px, bounds.y + bounds.height * py];
            f.push_transform(origin, [0.0, 0.0], [sx, sy], 0.0);
            draw_img(f, state, &img, bounds, uv);
            f.pop_transform();
        }
        DynamicDrawableSpec::Transition { dur_ms, img_key_0, img_key_1 } => {
            let t = sample_drawable_transition(f, clip_box, state, view_key, "src", *dur_ms);
            if t <= 0.001 {
                if let Some((img, bounds, uv)) = resolve_geom(state, img_key_0)
                    && bounds.intersect(f.clip).width > 0.0
                    && bounds.intersect(f.clip).height > 0.0
                {
                    draw_img(f, state, &img, bounds, uv);
                }
            } else if t >= 0.999 {
                if let Some((img, bounds, uv)) = resolve_geom(state, img_key_1)
                    && bounds.intersect(f.clip).width > 0.0
                    && bounds.intersect(f.clip).height > 0.0
                {
                    draw_img(f, state, &img, bounds, uv);
                }
            } else {
                let g0 = resolve_geom(state, img_key_0);
                let g1 = resolve_geom(state, img_key_1);
                if let Some((img, bounds, uv)) = g0 {
                    f.push_layer(bounds, 1.0 - t);
                    draw_img(f, state, &img, bounds, uv);
                    f.pop_layer();
                }
                if let Some((img, bounds, uv)) = g1 {
                    f.push_layer(bounds, t);
                    draw_img(f, state, &img, bounds, uv);
                    f.pop_layer();
                }
            }
        }
    }
}

pub(super) fn image_geometry(bounds: Rect, clip: Rect, image: &Image, scale: u8, density_scale: f32, matrix: Option<[f32;4]>) -> Option<(Rect, [f32; 4])> {
    let density_scale = if density_scale.is_finite() && density_scale > 0.0 { density_scale } else { 1.0 };
    let w = image.width() as f32 * density_scale; let h = image.height() as f32 * density_scale;
    if bounds.width <= 0.0 || bounds.height <= 0.0 { return None; }
    if scale == 7 {
        let [sx,sy,tx,ty]=matrix.unwrap_or([1.0,1.0,0.0,0.0]);
        if ![sx,sy,tx,ty].iter().all(|v|v.is_finite()) || sx==0.0 || sy==0.0 {return None;}
        let full=Rect{x:bounds.x+tx+(w*sx).min(0.0),y:bounds.y+ty+(h*sy).min(0.0),width:(w*sx).abs(),height:(h*sy).abs()};
        let visible=full.intersect(clip);
        if visible.width<=0.0 || visible.height<=0.0 {return None;}
        let mut uv=[(visible.x-full.x)/full.width,(visible.y-full.y)/full.height,(visible.x+visible.width-full.x)/full.width,(visible.y+visible.height-full.y)/full.height];
        if sx<0.0 {uv[0]=1.0-uv[0];uv[2]=1.0-uv[2];}
        if sy<0.0 {uv[1]=1.0-uv[1];uv[3]=1.0-uv[3];}
        return Some((visible,uv));
    }
    if scale == 1 || scale == 3 {
        let factor=if scale==1 {(bounds.width/w).max(bounds.height/h)} else {1.0};
        let width=w*factor;let height=h*factor;
        let full=Rect{x:bounds.x+(bounds.width-width)*0.5,y:bounds.y+(bounds.height-height)*0.5,width,height};
        let visible=full.intersect(clip);
        if visible.width<=0.0 || visible.height<=0.0 {return None;}
        return Some((visible,[(visible.x-full.x)/width,(visible.y-full.y)/height,(visible.x+visible.width-full.x)/width,(visible.y+visible.height-full.y)/height]));
    }
    if scale == 2 { return Some((bounds, [0.0, 0.0, 1.0, 1.0])); }
    let mut factor = (bounds.width / w).min(bounds.height / h);
    if scale == 4 { factor = factor.min(1.0); }
    let width = w * factor; let height = h * factor;
    let align = match scale { 5 => 0.0, 6 => 1.0, _ => 0.5 };
    Some((Rect { x: bounds.x + (bounds.width-width)*align, y: bounds.y + (bounds.height-height)*align, width, height }, [0.0,0.0,1.0,1.0]))
}
