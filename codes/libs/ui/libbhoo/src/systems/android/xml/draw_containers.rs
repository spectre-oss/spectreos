use super::draw::DrawCtx;
use super::*;

impl Node {
    pub(super) fn draw_containers(
        &'static self,
        f: &mut Frame<'_>,
        b: Rect,
        state: &mut LayoutState,
        id: u64,
        ctx: DrawCtx<'_>,
    ) {
        let DrawCtx {
            t,
            enabled,
            view_key,
            explicit_background,
            background_tint,
            scaled_radius,
            corner_radii,
            corner_family,
            gradient,
            alert_dialog_icon,
            is_snackbar,
            is_floating_toolbar,
            til_spec,
            nav_spec,
            collapsing_spec,
            view_pager_spec,
            inner,
            action: action_out,
            ..
        } = ctx;
        let mut action = *action_out;
        match self.kind {
            Kind::Linear | Kind::RadioGroup | Kind::TableRow | Kind::Frame | Kind::MediaController | Kind::MaterialCard | Kind::Absolute | Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper => {
                if let Some(spec) = collapsing_spec {
                    let ct_ext = self.collapsing_toolbar_ext();
                    let expanded_px = state.dp(spec.expanded_h_dp);
                    let collapsed_px = state.dp(64.0).min(expanded_px);
                    let scroll_px = self.collapsing_toolbar_scroll_px(state, id);
                    let collapse_range = (expanded_px - collapsed_px).max(0.0);
                    let collapse_t = if collapse_range > 0.001 {
                        (scroll_px / collapse_range).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let collapsed_delta = collapse_range * collapse_t;
                    let bar_h = (expanded_px - collapsed_delta).min(b.height).max(0.0);
                    let bar_rect = Rect { x: b.x, y: b.y, width: b.width, height: bar_h };
                    let top_h = collapsed_px.min(bar_h);
                    let top_bar_rect = Rect { x: bar_rect.x, y: bar_rect.y, width: bar_rect.width, height: top_h };
                    let scrim_color = spec.scrim.unwrap_or_else(|| match state.theme_style {
                        AndroidTheme::Material3Light | AndroidTheme::MaterialLight => Color(0xF3, 0xED, 0xF7, 255),
                        AndroidTheme::Material3Dark | AndroidTheme::MaterialDark => Color(0x21, 0x1F, 0x26, 255),
                        AndroidTheme::Bhoo => t.surface_high,
                    });
                    let scrim_dur_ms = ct_ext.map_or(0, |e| e.scrim_dur_ms);
                    let scrim_alpha = if scrim_dur_ms > 0 {
                        let scrim_on = collapse_t >= 0.72;
                        let prev = state.collapsing_scrim_motion.get(&view_key).copied().unwrap_or_default();
                        let (next_sm, anim) = eval_binary_ease_out_motion(
                            prev,
                            scrim_on,
                            scrim_dur_ms as f64 / 1000.0,
                            f.time(),
                            f.reduced_motion(),
                        );
                        if state.collapsing_scrim_motion.len() >= 64
                            && !state.collapsing_scrim_motion.contains_key(&view_key)
                        {
                            state.collapsing_scrim_motion.clear();
                        }
                        state.collapsing_scrim_motion.insert(view_key.clone(), next_sm);
                        if anim {
                            f.continuous(bar_rect);
                        }
                        next_sm.progress
                    } else {
                        collapse_t
                    };
                    f.within(bar_rect, |f| {
                        if let Some(gspec) = gradient {
                            let is_oval = self.radius < 0.0 && corner_radii.is_none();
                            let radii = corner_radii.unwrap_or([scaled_radius.max(0.0); 4]);
                            paint_gradient(f, state, bar_rect, gspec, is_oval, radii, corner_family, background_tint);
                        } else {
                            let base_surface = background_tint
                                .or(explicit_background)
                                .unwrap_or_else(|| state.theme_style.toolbar_surface().unwrap_or(t.background));
                            if let Some(radii) = corner_radii {
                                paint_rounded_rect_radii(f, bar_rect, radii, base_surface);
                            } else {
                                f.paint_rect(bar_rect, scaled_radius.max(0.0), base_surface);
                            }
                        }
                        let mut toolbar_idx: Option<usize> = None;
                        for (i, child) in self.children.iter().enumerate() {
                            if child.effective_visibility(state) == Visibility::Gone {
                                continue;
                            }
                            if matches!(child.kind, Kind::Toolbar) && toolbar_idx.is_none() {
                                toolbar_idx = Some(i);
                                continue;
                            }
                            let ckey = child_view_id(id, i, child);
                            let margin = child.resolved_margin(state);
                            let cmode = child.collapsing_child_mode();
                            let base_h = if matches!(
                                cmode,
                                Some(CollapsingChildMode::Parallax(_) | CollapsingChildMode::None)
                            ) {
                                (expanded_px - margin[1] - margin[3]).max(0.0)
                            } else {
                                (bar_rect.height - margin[1] - margin[3]).max(0.0)
                            };
                            let avail_w = (bar_rect.width - margin[0] - margin[2]).max(0.0);
                            let mut r = Rect {
                                x: bar_rect.x + margin[0],
                                y: bar_rect.y + margin[1],
                                width: avail_w,
                                height: base_h,
                            };
                            if let Size::Dp(w) = child.width { r.width = state.dp(w); }
                            if let Size::Dp(h) = child.height { r.height = state.dp(h); }
                            if matches!(child.width, Size::Wrap) {
                                r.width = child.layout_intrinsic(true, state, r.width, f.ui.text_metrics, ckey).min(r.width);
                            }
                            if matches!(child.height, Size::Wrap) {
                                r.height = child.layout_intrinsic(false, state, r.width, f.ui.text_metrics, ckey).min(r.height);
                            }
                            if child.layout_gravity_explicit {
                                let right = match child.layout_gravity[0] {
                                    0 => state.rtl,
                                    2 => !state.rtl,
                                    4 => true,
                                    _ => false,
                                };
                                r.x = if child.layout_gravity[0] == 1 {
                                    bar_rect.x + (bar_rect.width - r.width) * 0.5 + margin[0] - margin[2]
                                } else if right {
                                    bar_rect.x + bar_rect.width - r.width - margin[2]
                                } else {
                                    bar_rect.x + margin[0]
                                };
                            }
                            match cmode {
                                Some(CollapsingChildMode::Pin) => {
                                    // Pinned at top position inside the collapsing toolbar.
                                }
                                Some(CollapsingChildMode::Parallax(mult)) => {
                                    r.y -= collapsed_delta * mult;
                                }
                                Some(CollapsingChildMode::None) => {
                                    r.y -= collapsed_delta;
                                }
                                None => {
                                    if matches!(child.kind, Kind::Image) {
                                        r.y -= scroll_px * 0.5;
                                    }
                                }
                            }
                            let mut child_action = None;
                            f.within(r, |f| {
                                child_action = child.draw(f, r, state, ckey);
                            });
                            if child_action.is_some() {
                                action = child_action;
                            }
                        }
                        if scrim_alpha > 0.001 && scrim_color.3 > 0 {
                            let overlay = scrim_color.opacity(scrim_alpha);
                            if let Some(radii) = corner_radii {
                                paint_rounded_rect_radii(f, bar_rect, radii, overlay);
                            } else {
                                f.paint_rect(bar_rect, scaled_radius.max(0.0), overlay);
                            }
                        }
                        let mut has_nav = false;
                        let mut trailing_actions_px = 0.0_f32;
                        let mut toolbar_fallback_title = String::new();
                        if let Some(ti) = toolbar_idx {
                            let tb = &self.children[ti];
                            let tb_id = child_view_id(id, ti, tb);
                            let tb_key = if tb.id.is_empty() { format!("#{tb_id}") } else { tb.id.to_owned() };
                            has_nav = !tb.image_src.is_empty();
                            if let Some(items) = state.toolbar_menu_items.get(&tb_key) {
                                let action_count = items
                                    .iter()
                                    .filter(|m| m.visible && m.show_as_action != AndroidShowAsAction::Never)
                                    .count()
                                    + usize::from(items.iter().any(|m| m.visible && m.show_as_action == AndroidShowAsAction::Never));
                                trailing_actions_px = action_count as f32 * state.dp(48.0);
                            }
                            toolbar_fallback_title = state
                                .text
                                .get(&tb_key)
                                .map_or(tb.text, String::as_str)
                                .to_owned();
                            let mut tb_rect = top_bar_rect;
                            match tb.collapsing_child_mode() {
                                Some(CollapsingChildMode::Parallax(mult)) => {
                                    tb_rect.y -= collapsed_delta * mult;
                                }
                                Some(CollapsingChildMode::None) => {
                                    tb_rect.y -= collapsed_delta;
                                }
                                Some(CollapsingChildMode::Pin) | None => {}
                            }
                            let prev_flag = state.collapsing_toolbar_child;
                            state.collapsing_toolbar_child = true;
                            let mut tb_action = None;
                            f.within(tb_rect, |f| {
                                tb_action = tb.draw(f, tb_rect, state, tb_id);
                            });
                            state.collapsing_toolbar_child = prev_flag;
                            if tb_action.is_some() {
                                action = tb_action;
                            }
                        }
                        let title_raw = state
                            .text
                            .get(&view_key)
                            .filter(|s| !s.is_empty())
                            .map(String::as_str)
                            .or_else(|| (!spec.title.is_empty()).then_some(spec.title))
                            .or_else(|| (!self.text.is_empty()).then_some(self.text))
                            .unwrap_or(toolbar_fallback_title.as_str());
                        let subtitle_raw = ct_ext.map_or("", |e| e.subtitle);
                        let title_mode = ct_ext.map_or(CollapsingTitleMode::Scale, |e| e.mode);
                        let max_lines = ct_ext.map_or(1, |e| e.max_lines);
                        let (italic, family) = self.text_style_ext();
                        let expanded_sp = if spec.expanded_h_dp >= 140.0 {
                            28.0
                        } else if spec.expanded_h_dp >= 100.0 {
                            24.0
                        } else {
                            22.0
                        };
                        let collapsed_sp = 22.0;
                        let expanded_start = state.dp(16.0);
                        let collapsed_start = state.dp(if has_nav { 56.0 } else { 16.0 });
                        let expanded_end = state.dp(16.0);
                        let collapsed_end = state.dp(16.0) + trailing_actions_px;
                        let bottom_pad = state.dp(if spec.expanded_h_dp >= 140.0 { 20.0 } else { 16.0 });
                        let en_alpha = if enabled { 1.0_f32 } else { 0.38_f32 };
                        let base_title_color = self.color.unwrap_or(t.text).opacity(en_alpha);
                        let sub_alpha = if !subtitle_raw.is_empty() {
                            (1.0 - collapse_t * 1.6).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        let sub_size = state.sp(14.0);
                        let sub_h = sub_size * 1.3;
                        let sub_gap = state.dp(2.0);
                        let sub_reserve = if !subtitle_raw.is_empty() { sub_h + sub_gap } else { 0.0 };
                        let sub_style = TextStyle {
                            size: sub_size,
                            weight: 400,
                            align: if state.rtl { 2 } else { 0 },
                            italic,
                            family,
                            ..TextStyle::default()
                        };
                        let wrap_capped = |text: &str, w: f32, style: TextStyle, limit: usize, metrics: TextMetrics| -> Vec<String> {
                            if limit <= 1 {
                                return vec![ellipsize_single_line(text, w, style, metrics, 3)];
                            }
                            let mut raw_lines: Vec<String> = Vec::new();
                            wrapped_text_lines(text, w.max(1.0), style, metrics, |l| {
                                raw_lines.push(l.text.into_owned());
                            });
                            if raw_lines.is_empty() {
                                return vec![String::new()];
                            }
                            if raw_lines.len() <= limit {
                                return raw_lines;
                            }
                            let mut out: Vec<String> = raw_lines[..limit - 1].to_vec();
                            let tail = raw_lines[limit - 1..].join(" ");
                            out.push(ellipsize_single_line(&tail, w, style, metrics, 3));
                            out
                        };
                        if !title_raw.is_empty() {
                            match title_mode {
                                CollapsingTitleMode::Fade => {
                                    let exp_alpha = (1.0 - collapse_t * 2.0).clamp(0.0, 1.0);
                                    let col_alpha = ((collapse_t - 0.5) * 2.0).clamp(0.0, 1.0);
                                    let exp_avail_w = (bar_rect.width - expanded_start - expanded_end).max(0.0);
                                    let exp_x = if state.rtl {
                                        bar_rect.x + expanded_end
                                    } else {
                                        bar_rect.x + expanded_start
                                    };
                                    let exp_size = state.sp(expanded_sp);
                                    let exp_line_h = exp_size * 1.3;
                                    let exp_style = TextStyle {
                                        size: exp_size,
                                        weight: if self.text_bold { 700 } else { 400 },
                                        align: if state.rtl { 2 } else { 0 },
                                        italic,
                                        family,
                                        ..TextStyle::default()
                                    };
                                    let col_size = state.sp(collapsed_sp);
                                    let col_h = col_size * 1.3;
                                    let col_y = top_bar_rect.y + (top_bar_rect.height - col_h) * 0.5;
                                    if exp_alpha > 0.001 {
                                        let use_lines = if max_lines > 1 && collapse_t < 0.5 { max_lines } else { 1 };
                                        let lines = wrap_capped(title_raw, exp_avail_w, exp_style, use_lines, f.ui.text_metrics);
                                        let block_h = lines.len().max(1) as f32 * exp_line_h;
                                        let exp_y = (bar_rect.y + bar_rect.height - bottom_pad - sub_reserve - block_h).max(col_y);
                                        let color = base_title_color.opacity(exp_alpha);
                                        for (li, line) in lines.iter().enumerate() {
                                            f.styled_text(
                                                Rect {
                                                    x: exp_x,
                                                    y: exp_y + li as f32 * exp_line_h,
                                                    width: exp_avail_w,
                                                    height: exp_line_h,
                                                },
                                                line,
                                                color,
                                                exp_style,
                                            );
                                        }
                                        if !subtitle_raw.is_empty() && sub_alpha > 0.001 {
                                            let sub_shown = ellipsize_single_line(subtitle_raw, exp_avail_w, sub_style, f.ui.text_metrics, 3);
                                            f.styled_text(
                                                Rect {
                                                    x: exp_x,
                                                    y: exp_y + block_h + sub_gap,
                                                    width: exp_avail_w,
                                                    height: sub_h,
                                                },
                                                &sub_shown,
                                                t.on_surface_variant.opacity(en_alpha * sub_alpha),
                                                sub_style,
                                            );
                                        }
                                    } else if !subtitle_raw.is_empty() && sub_alpha > 0.001 {
                                        let sub_shown = ellipsize_single_line(subtitle_raw, exp_avail_w, sub_style, f.ui.text_metrics, 3);
                                        let sub_y = (bar_rect.y + bar_rect.height - bottom_pad - sub_h).max(col_y);
                                        f.styled_text(
                                            Rect { x: exp_x, y: sub_y, width: exp_avail_w, height: sub_h },
                                            &sub_shown,
                                            t.on_surface_variant.opacity(en_alpha * sub_alpha),
                                            sub_style,
                                        );
                                    }
                                    if col_alpha > 0.001 {
                                        let col_avail_w = (bar_rect.width - collapsed_start - collapsed_end).max(0.0);
                                        let col_x = if state.rtl {
                                            bar_rect.x + collapsed_end
                                        } else {
                                            bar_rect.x + collapsed_start
                                        };
                                        let col_style = TextStyle {
                                            size: col_size,
                                            weight: if self.text_bold { 700 } else { 400 },
                                            align: if state.rtl { 2 } else { 0 },
                                            italic,
                                            family,
                                            ..TextStyle::default()
                                        };
                                        let shown = ellipsize_single_line(title_raw, col_avail_w, col_style, f.ui.text_metrics, 3);
                                        f.styled_text(
                                            Rect { x: col_x, y: col_y, width: col_avail_w, height: col_h },
                                            &shown,
                                            base_title_color.opacity(col_alpha),
                                            col_style,
                                        );
                                    }
                                }
                                CollapsingTitleMode::Scale => {
                                    let title_sp = expanded_sp + (collapsed_sp - expanded_sp) * collapse_t;
                                    let title_size = state.sp(title_sp);
                                    let title_h = title_size * 1.3;
                                    let start_inset = expanded_start + (collapsed_start - expanded_start) * collapse_t;
                                    let end_inset = expanded_end + (collapsed_end - expanded_end) * collapse_t;
                                    let avail_w = (bar_rect.width - start_inset - end_inset).max(0.0);
                                    let title_x = if state.rtl {
                                        bar_rect.x + end_inset
                                    } else {
                                        bar_rect.x + start_inset
                                    };
                                    let use_lines = if max_lines > 1 && collapse_t < 0.5 { max_lines } else { 1 };
                                    // Smooth AOSP CollapsingTextHelper GPU scale interpolation during scroll
                                    // (zero discrete glyph-size stepping; bit-identical at collapse_t == 0 or 1!).
                                    if use_lines == 1 && collapse_t > 0.001 && collapse_t < 0.999 && expanded_sp > collapsed_sp + 0.5 {
                                        let exp_size = state.sp(expanded_sp);
                                        let exp_h = exp_size * 1.3;
                                        let scale = (title_sp / expanded_sp).clamp(0.4, 1.0);
                                        let exp_style = TextStyle {
                                            size: exp_size,
                                            weight: if self.text_bold { 700 } else { 400 },
                                            align: if state.rtl { 2 } else { 0 },
                                            italic,
                                            family,
                                            ..TextStyle::default()
                                        };
                                        let collapsed_y = top_bar_rect.y + (top_bar_rect.height - title_h) * 0.5;
                                        let expanded_y = (bar_rect.y + bar_rect.height - bottom_pad - sub_reserve - exp_h).max(collapsed_y);
                                        let title_y = expanded_y + (collapsed_y - expanded_y) * collapse_t;
                                        let pivot_x = if state.rtl { title_x + avail_w } else { title_x };
                                        let unscaled_w = (avail_w / scale).max(1.0);
                                        let shown = ellipsize_single_line(title_raw, unscaled_w, exp_style, f.ui.text_metrics, 3);
                                        f.push_transform([pivot_x, title_y], [0.0, 0.0], [scale, scale], 0.0);
                                        f.styled_text(
                                            Rect {
                                                x: if state.rtl { pivot_x - unscaled_w } else { title_x },
                                                y: title_y,
                                                width: unscaled_w,
                                                height: exp_h,
                                            },
                                            &shown,
                                            base_title_color,
                                            exp_style,
                                        );
                                        f.pop_transform();
                                        if !subtitle_raw.is_empty() && sub_alpha > 0.001 {
                                            let sub_shown = ellipsize_single_line(subtitle_raw, avail_w, sub_style, f.ui.text_metrics, 3);
                                            f.styled_text(
                                                Rect {
                                                    x: title_x,
                                                    y: title_y + title_h + sub_gap,
                                                    width: avail_w,
                                                    height: sub_h,
                                                },
                                                &sub_shown,
                                                t.on_surface_variant.opacity(en_alpha * sub_alpha),
                                                sub_style,
                                            );
                                        }
                                    } else {
                                        let style = TextStyle {
                                            size: title_size,
                                            weight: if self.text_bold { 700 } else { 400 },
                                            align: if state.rtl { 2 } else { 0 },
                                            italic,
                                            family,
                                            ..TextStyle::default()
                                        };
                                        let lines = wrap_capped(title_raw, avail_w, style, use_lines, f.ui.text_metrics);
                                        let block_h = lines.len().max(1) as f32 * title_h;
                                        let collapsed_y = top_bar_rect.y + (top_bar_rect.height - title_h) * 0.5;
                                        let expanded_y = (bar_rect.y + bar_rect.height - bottom_pad - sub_reserve - block_h).max(collapsed_y);
                                        let title_y = expanded_y + (collapsed_y - expanded_y) * collapse_t;
                                        for (li, line) in lines.iter().enumerate() {
                                            f.styled_text(
                                                Rect {
                                                    x: title_x,
                                                    y: title_y + li as f32 * title_h,
                                                    width: avail_w,
                                                    height: title_h,
                                                },
                                                line,
                                                base_title_color,
                                                style,
                                            );
                                        }
                                        if !subtitle_raw.is_empty() && sub_alpha > 0.001 {
                                            let sub_shown = ellipsize_single_line(subtitle_raw, avail_w, sub_style, f.ui.text_metrics, 3);
                                            f.styled_text(
                                                Rect {
                                                    x: title_x,
                                                    y: title_y + block_h + sub_gap,
                                                    width: avail_w,
                                                    height: sub_h,
                                                },
                                                &sub_shown,
                                                t.on_surface_variant.opacity(en_alpha * sub_alpha),
                                                sub_style,
                                            );
                                        }
                                    }
                                }
                            }
                        } else if !subtitle_raw.is_empty() && sub_alpha > 0.001 {
                            let avail_w = (bar_rect.width - expanded_start - expanded_end).max(0.0);
                            let sx = if state.rtl { bar_rect.x + expanded_end } else { bar_rect.x + expanded_start };
                            let sy = (bar_rect.y + bar_rect.height - bottom_pad - sub_h).max(top_bar_rect.y);
                            let sub_shown = ellipsize_single_line(subtitle_raw, avail_w, sub_style, f.ui.text_metrics, 3);
                            f.styled_text(
                                Rect { x: sx, y: sy, width: avail_w, height: sub_h },
                                &sub_shown,
                                t.on_surface_variant.opacity(en_alpha * sub_alpha),
                                sub_style,
                            );
                        }
                    });
                } else if let Some(car_spec) = self.carousel_layout_spec() {
                    action = self.draw_carousel_layout(f, inner, state, id, car_spec, t);
                } else if let Some(vp_spec) = view_pager_spec {
                    let count = self.children.len();
                    if count == 0 {
                        state.view_pager_gestures.remove(&view_key);
                    } else {
                        let mut gesture = state.view_pager_gestures.remove(&view_key).unwrap_or_default();
                        let is_vertical = vp_spec.orient == ViewPagerOrientation::Vertical;
                        let axis_len = (if is_vertical { inner.height } else { inner.width }).max(1.0);
                        let page_margin_px = state.dp(vp_spec.page_margin_dp.max(0.0));
                        let page_stride = axis_len + page_margin_px;
                        let dir = if !is_vertical && state.rtl { -1.0 } else { 1.0 };
                        let now = if f.time().is_finite() { f.time().max(0.0) } else { 0.0 };
                        let instant = f.reduced_motion() || (now == 0.0 && gesture.last_time == 0.0);
                        let mut selected = if !gesture.initialized {
                            gesture.initialized = true;
                            let init = state
                                .selected_index
                                .get(&view_key)
                                .copied()
                                .or_else(|| state.displayed_children.get(&view_key).copied())
                                .unwrap_or(0)
                                .min(count - 1);
                            gesture.last_page = init;
                            gesture.last_time = now;
                            init
                        } else {
                            let sel = state.selected_index.get(&view_key).copied().map(|v| v.min(count - 1));
                            let disp = state.displayed_children.get(&view_key).copied().map(|v| v.min(count - 1));
                            let prev_page = gesture.last_page.min(count - 1);
                            let external_next = sel
                                .filter(|&v| v != prev_page)
                                .or_else(|| disp.filter(|&v| v != prev_page));
                            if let Some(next_page) = external_next {
                                gesture.last_page = next_page;
                                gesture.origin = None;
                                gesture.dragging = false;
                                gesture.samples.clear();
                                if vp_spec.has_ext && !instant {
                                    let sign = if next_page > prev_page { 1.0 } else { -1.0 };
                                    let from = sign * page_stride;
                                    gesture.offset = from;
                                    gesture.from_page = Some(prev_page);
                                    gesture.settle = Some((now, from, 0.25));
                                    f.continuous(inner);
                                } else {
                                    gesture.offset = 0.0;
                                    gesture.settle = None;
                                    gesture.from_page = None;
                                }
                                next_page
                            } else {
                                prev_page
                            }
                        };
                        gesture.last_time = now;
                        if let Some((start, from, dur)) = gesture.settle {
                            let p = if dur > 0.0 { ((now - start) / dur).clamp(0.0, 1.0) as f32 } else { 1.0 };
                            let ease = 1.0 - (1.0 - p) * (1.0 - p);
                            gesture.offset = from * (1.0 - ease);
                            if p >= 1.0 || f.reduced_motion() {
                                gesture.settle = None;
                                gesture.offset = 0.0;
                                gesture.from_page = None;
                            } else {
                                f.continuous(inner);
                            }
                        }
                        if enabled {
                            f.ui.seen.insert(id);
                            if !f.ui.next_focus_order.contains(&id) {
                                f.ui.next_focus_order.push(id);
                            }
                        }
                        if enabled && vp_spec.user_input_enabled && f.focused_id() == Some(id) && count > 1 {
                            let step: isize = if is_vertical {
                                if f.pressed_key(Key::Down) {
                                    1
                                } else if f.pressed_key(Key::Up) {
                                    -1
                                } else {
                                    0
                                }
                            } else {
                                let raw = if f.pressed_key(Key::Right) {
                                    1
                                } else if f.pressed_key(Key::Left) {
                                    -1
                                } else {
                                    0
                                };
                                if state.rtl { -raw } else { raw }
                            };
                            if step != 0 {
                                let next = (selected as isize + step).clamp(0, count as isize - 1) as usize;
                                if next != selected {
                                    let old = selected;
                                    selected = next;
                                    action = Some(self.id);
                                    let partners = state.linked_tab_pager_partners(&view_key);
                                    if !partners.is_empty() {
                                        for partner in partners {
                                            state.selected_index.insert(partner.clone(), selected);
                                            state.displayed_children.insert(partner, selected);
                                        }
                                        f.continuous(b);
                                    }
                                    if !f.reduced_motion() {
                                        let from = step as f32 * page_stride;
                                        gesture.offset = from;
                                        gesture.from_page = Some(old);
                                        gesture.settle = Some((now, from, 0.25));
                                        f.continuous(inner);
                                    }
                                }
                            }
                        }
                        let slop = state.dp(8.0);
                        if !vp_spec.user_input_enabled {
                            gesture.origin = None;
                            gesture.dragging = false;
                            gesture.samples.clear();
                        }
                        if enabled && vp_spec.user_input_enabled && f.input.pressed && f.pointer_in(inner) {
                            gesture.origin = f.input.pointer;
                            gesture.dragging = false;
                            gesture.settle = None;
                            gesture.from_page = None;
                            gesture.offset = 0.0;
                            gesture.samples.clear();
                            if let Some(p) = f.input.pointer {
                                gesture.samples.push((now, if is_vertical { p[1] } else { p[0] }));
                            }
                        }
                        if vp_spec.user_input_enabled
                            && let (Some(origin), Some(pointer)) = (gesture.origin, f.input.pointer)
                            && !f.input.cancelled
                            && !f.input.blurred
                        {
                            let axis_coord = if is_vertical { pointer[1] } else { pointer[0] };
                            if gesture.samples.last().map_or(true, |&(_, v)| (v - axis_coord).abs() > 0.01) {
                                gesture.samples.push((now, axis_coord));
                                if gesture.samples.len() > 8 {
                                    gesture.samples.remove(0);
                                }
                            }
                            let dx = pointer[0] - origin[0];
                            let dy = pointer[1] - origin[1];
                            let (primary, cross) = if is_vertical { (dy, dx) } else { (dx, dy) };
                            let logical = primary * dir;
                            if !gesture.dragging && primary.abs() > slop && primary.abs() >= cross.abs() {
                                gesture.dragging = true;
                                f.ui.active = None;
                            }
                            if gesture.dragging {
                                let at_start = selected == 0 && logical > 0.0;
                                let at_end = selected + 1 >= count && logical < 0.0;
                                gesture.offset = if at_start || at_end {
                                    (logical * 0.3).clamp(-axis_len * 0.25, axis_len * 0.25)
                                } else {
                                    logical.clamp(-page_stride, page_stride)
                                };
                                if !f.input.released {
                                    f.continuous(inner);
                                }
                            }
                        }
                        let was_dragging = gesture.dragging;
                        if gesture.origin.is_some()
                            && (f.input.released || f.input.cancelled || f.input.blurred)
                        {
                            if gesture.dragging && !f.input.cancelled && !f.input.blurred && count > 1 {
                                let threshold = axis_len * 0.30;
                                let v_threshold = state.dp(400.0);
                                let v_logical = pointer_velocity(&gesture.samples, now) * dir;
                                let old = selected;
                                if selected + 1 < count
                                    && (gesture.offset <= -threshold
                                        || (gesture.offset < -slop && v_logical < -v_threshold))
                                {
                                    selected += 1;
                                } else if selected > 0
                                    && (gesture.offset >= threshold
                                        || (gesture.offset > slop && v_logical > v_threshold))
                                {
                                    selected -= 1;
                                }
                                if selected != old {
                                    action = Some(self.id);
                                    gesture.offset += (selected as f32 - old as f32) * page_stride;
                                    gesture.from_page = Some(old);
                                    let partners = state.linked_tab_pager_partners(&view_key);
                                    if !partners.is_empty() {
                                        for partner in partners {
                                            state.selected_index.insert(partner.clone(), selected);
                                            state.displayed_children.insert(partner, selected);
                                        }
                                        f.continuous(b);
                                    }
                                }
                                if !f.reduced_motion() && gesture.offset.abs() > 0.5 {
                                    gesture.settle = Some((now, gesture.offset, 0.25));
                                    f.continuous(inner);
                                } else {
                                    gesture.offset = 0.0;
                                    gesture.from_page = None;
                                }
                            } else {
                                gesture.offset = 0.0;
                                gesture.from_page = None;
                            }
                            gesture.origin = None;
                            gesture.dragging = false;
                            gesture.samples.clear();
                        }
                        gesture.last_page = selected;
                        state.displayed_children.insert(view_key.clone(), selected);
                        state.selected_index.insert(view_key.clone(), selected);
                        // Each entry: (page_idx, rel_slot, shift_logical, p)
                        let mut pages: Vec<(usize, f32, f32, f32)> = Vec::with_capacity(2);
                        if gesture.offset.abs() > 0.5 {
                            if let Some(fp) = gesture.from_page.filter(|&fp| fp != selected && fp < count) {
                                let rel = if gesture.offset > 0.0 { -1.0 } else { 1.0 };
                                let shift_l = rel * page_stride + gesture.offset;
                                let p = (shift_l / page_stride.max(1.0)).clamp(-1.0, 1.0);
                                pages.push((fp, rel, shift_l, p));
                            } else if gesture.offset > 0.5 && selected > 0 {
                                let shift_l = -page_stride + gesture.offset;
                                let p = (shift_l / page_stride.max(1.0)).clamp(-1.0, 1.0);
                                pages.push((selected - 1, -1.0, shift_l, p));
                            } else if gesture.offset < -0.5 && selected + 1 < count {
                                let shift_l = page_stride + gesture.offset;
                                let p = (shift_l / page_stride.max(1.0)).clamp(-1.0, 1.0);
                                pages.push((selected + 1, 1.0, shift_l, p));
                            }
                        }
                        let sel_p = (gesture.offset / page_stride.max(1.0)).clamp(-1.0, 1.0);
                        pages.push((selected, 0.0, gesture.offset, sel_p));
                        if vp_spec.transformer == ViewPagerTransformer::Depth {
                            // Paint behind (p > 0) page first, foreground/outgoing (p <= 0) page on top.
                            pages.sort_by(|a, b| {
                                b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal)
                            });
                        }
                        f.within(inner, |f| {
                            for (page_idx, _rel, shift_logical, p) in pages {
                                let child = &self.children[page_idx];
                                if child.effective_visibility(state) == Visibility::Gone {
                                    continue;
                                }
                                let margin = child.resolved_margin(state);
                                let depth_behind =
                                    vp_spec.transformer == ViewPagerTransformer::Depth && p > 1e-4;
                                let shift = if depth_behind { 0.0 } else { shift_logical * dir };
                                let mut r = Rect {
                                    x: inner.x + margin[0] + if is_vertical { 0.0 } else { shift },
                                    y: inner.y + margin[1] + if is_vertical { shift } else { 0.0 },
                                    width: (inner.width - margin[0] - margin[2]).max(0.0),
                                    height: (inner.height - margin[1] - margin[3]).max(0.0),
                                };
                                let ckey = child_view_id(id, page_idx, child);
                                if let Size::Dp(w) = child.width { r.width = state.dp(w); }
                                if let Size::Dp(h) = child.height { r.height = state.dp(h); }
                                if matches!(child.width, Size::Wrap) {
                                    r.width = child.layout_intrinsic(true, state, r.width, f.ui.text_metrics, ckey).min(r.width);
                                }
                                if matches!(child.height, Size::Wrap) {
                                    r.height = child.layout_intrinsic(false, state, r.width, f.ui.text_metrics, ckey).min(r.height);
                                }
                                let (tr_scale, tr_alpha): (Option<f32>, Option<f32>) =
                                    if gesture.offset.abs() > 0.5 {
                                        match vp_spec.transformer {
                                            ViewPagerTransformer::None => (None, None),
                                            ViewPagerTransformer::ZoomOut if p.abs() > 1e-4 => {
                                                let s = (1.0 - p.abs() * 0.15).max(0.85);
                                                let a = (1.0 - p.abs() * 0.5).max(0.5);
                                                (Some(s), Some(a))
                                            }
                                            ViewPagerTransformer::Depth if p > 1e-4 => {
                                                let s = (0.75 + 0.25 * (1.0 - p)).clamp(0.75, 1.0);
                                                let a = (1.0 - p).clamp(0.0, 1.0);
                                                (Some(s), Some(a))
                                            }
                                            _ => (None, None),
                                        }
                                    } else {
                                        (None, None)
                                    };
                                let pushed_tr = if let Some(s) = tr_scale {
                                    let origin = [r.x + r.width * 0.5, r.y + r.height * 0.5];
                                    f.push_transform(origin, [0.0, 0.0], [s, s], 0.0);
                                    true
                                } else {
                                    false
                                };
                                let pushed_layer = if let Some(a) = tr_alpha {
                                    f.push_layer(r, a);
                                    true
                                } else {
                                    false
                                };
                                let mut child_action = None;
                                let block_input = was_dragging || page_idx != selected;
                                f.without_input(block_input, |f| {
                                    f.within(r, |f| {
                                        child_action = child.draw(f, r, state, ckey);
                                    });
                                });
                                if pushed_layer {
                                    f.pop_layer();
                                }
                                if pushed_tr {
                                    f.pop_transform();
                                }
                                if child_action.is_some() {
                                    action = child_action;
                                }
                            }
                        });
                        if count > 1 {
                            let active_d = state.dp(8.0);
                            let inactive_d = state.dp(6.0);
                            let gap = state.dp(6.0);
                            let total_w = (count.saturating_sub(1)) as f32 * (inactive_d + gap) + active_d;
                            let mut cursor_x = b.x + (b.width - total_w) * 0.5;
                            let cy = b.y + b.height - state.dp(10.0);
                            let dot_indices: Vec<usize> = if !is_vertical && state.rtl {
                                (0..count).rev().collect()
                            } else {
                                (0..count).collect()
                            };
                            f.within(b, |f| {
                                for idx in dot_indices {
                                    let is_active = idx == selected;
                                    let diam = if is_active { active_d } else { inactive_d };
                                    let dot = Rect {
                                        x: cursor_x,
                                        y: cy - diam * 0.5,
                                        width: diam,
                                        height: diam,
                                    };
                                    let color = if is_active {
                                        t.primary.opacity(if enabled { 1.0 } else { 0.38 })
                                    } else {
                                        t.on_surface_variant.opacity(if enabled { 0.35 } else { 0.18 })
                                    };
                                    f.paint_rect(dot, diam * 0.5, color);
                                    cursor_x += diam + gap;
                                }
                            });
                        }
                        state.view_pager_gestures.insert(view_key, gesture);
                    }
                } else if let Some(nav) = nav_spec {
                    let items = self.nav_items();
                    let count = items.len();
                    let mut selected = state
                        .selected_index
                        .get(&view_key)
                        .copied()
                        .unwrap_or(0)
                        .min(count.saturating_sub(1));
                    let disabled_alpha = if enabled { 1.0 } else { 0.38 };
                    match nav.kind {
                        NavBarKind::Bar => {
                            let slot_w = inner.width / count.max(1) as f32;
                            let mut responses = Vec::with_capacity(count);
                            for (i, item) in items.iter().enumerate() {
                                let sx = if state.rtl {
                                    inner.x + inner.width - (i + 1) as f32 * slot_w
                                } else {
                                    inner.x + i as f32 * slot_w
                                };
                                let slot = Rect {
                                    x: sx,
                                    y: inner.y,
                                    width: slot_w,
                                    height: inner.height,
                                };
                                let r = f.interact(anonymous_child_id(id, 9900 + i), slot, enabled);
                                if r.clicked {
                                    selected = i;
                                    state.selected_index.insert(view_key.clone(), i);
                                    action = Some(if !item.id.is_empty() { item.id } else { self.id });
                                }
                                responses.push((slot, r));
                            }
                            let label_size = state.sp(12.0);
                            let label_h = (label_size * 1.34).max(state.dp(16.0));
                            let gap = state.dp(4.0);
                            for (i, (item, (slot, r))) in items.iter().zip(responses).enumerate() {
                                let is_selected = i == selected;
                                f.semantic(r, Role::Tab, item.title, is_selected, None, None, None);
                                let show_label = !item.title.is_empty()
                                    && match nav.label_mode {
                                        NavLabelMode::Labeled => true,
                                        NavLabelMode::Unlabeled => false,
                                        NavLabelMode::Selected => is_selected,
                                        NavLabelMode::Auto => count <= 3 || is_selected,
                                    };
                                let pill_w = state.dp(64.0).min((slot.width - state.dp(8.0)).max(0.0));
                                let pill_h = state.dp(32.0).min(slot.height);
                                let pill_y = if show_label {
                                    let top_pad = state.dp(12.0);
                                    if slot.height >= top_pad + pill_h + gap + label_h + state.dp(8.0) {
                                        slot.y + top_pad
                                    } else {
                                        slot.y + ((slot.height - pill_h - gap - label_h) * 0.5).max(state.dp(2.0))
                                    }
                                } else {
                                    slot.y + (slot.height - pill_h) * 0.5
                                };
                                let pill = Rect {
                                    x: slot.x + (slot.width - pill_w) * 0.5,
                                    y: pill_y,
                                    width: pill_w,
                                    height: pill_h,
                                };
                                if is_selected {
                                    f.paint_rect(
                                        pill,
                                        pill_h * 0.5,
                                        t.secondary_container.opacity(disabled_alpha),
                                    );
                                }
                                let icon_base = if is_selected {
                                    t.on_secondary_container
                                } else {
                                    t.on_surface_variant
                                };
                                f.state_layer(Response { bounds: pill, ..r }, pill_h * 0.5, icon_base);
                                let icon_s = state.dp(24.0).min(pill.height).min(pill.width);
                                let icon_rect = Rect {
                                    x: pill.x + (pill.width - icon_s) * 0.5,
                                    y: pill.y + (pill.height - icon_s) * 0.5,
                                    width: icon_s,
                                    height: icon_s,
                                };
                                let icon_color = icon_base.opacity(disabled_alpha);
                                if !item.icon_name.is_empty() && let Some(img) = state.images.get(item.icon_name).cloned() {
                                    let tinted = tint_image(state, &img, Some(icon_color), 1);
                                    f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                                } else {
                                    let icon = parse_material_icon(item.icon_name).unwrap_or(MaterialIcon::Widgets);
                                    f.paint_material_icon(icon_rect, icon, icon_color);
                                }
                                if let Some(badge) = self.nav_badge_for_item(i, item.id, state) {
                                    let ext = self.material_badge_ext(Some(i), item.id);
                                    paint_nav_badge_ext(f, icon_rect, badge, ext, state, disabled_alpha);
                                }
                                if show_label {
                                    let label_rect = Rect {
                                        x: slot.x + state.dp(2.0),
                                        y: pill.y + pill.height + gap,
                                        width: (slot.width - state.dp(4.0)).max(0.0),
                                        height: label_h,
                                    };
                                    let label_color = if is_selected {
                                        t.text
                                    } else {
                                        t.on_surface_variant
                                    }
                                    .opacity(disabled_alpha);
                                    f.within(slot, |f| {
                                        f.styled_text(
                                            label_rect,
                                            item.title,
                                            label_color,
                                            TextStyle {
                                                size: label_size,
                                                weight: if is_selected { 700 } else { 500 },
                                                align: 1,
                                                ..TextStyle::default()
                                            },
                                        );
                                    });
                                }
                                f.focus_ring(r);
                            }
                        }
                        NavBarKind::Rail => {
                            let item_h = state.dp(if nav.label_mode == NavLabelMode::Unlabeled { 48.0 } else { 56.0 });
                            let item_gap = state.dp(12.0);
                            let total_h = count as f32 * item_h + count.saturating_sub(1) as f32 * item_gap;
                            let top_pad = state.dp(12.0);
                            let start_y = match self.content_gravity[1] {
                                1 => inner.y + ((inner.height - total_h) * 0.5).max(top_pad),
                                2 => inner.y + (inner.height - total_h - top_pad).max(top_pad),
                                _ => inner.y + top_pad,
                            };
                            let mut responses = Vec::with_capacity(count);
                            for (i, item) in items.iter().enumerate() {
                                let slot = Rect {
                                    x: inner.x,
                                    y: start_y + i as f32 * (item_h + item_gap),
                                    width: inner.width,
                                    height: item_h,
                                };
                                let r = f.interact(anonymous_child_id(id, 9900 + i), slot, enabled);
                                if r.clicked {
                                    selected = i;
                                    state.selected_index.insert(view_key.clone(), i);
                                    action = Some(if !item.id.is_empty() { item.id } else { self.id });
                                }
                                responses.push((slot, r));
                            }
                            let label_size = state.sp(12.0);
                            let label_h = (label_size * 1.34).max(state.dp(16.0));
                            let gap = state.dp(4.0);
                            for (i, (item, (slot, r))) in items.iter().zip(responses).enumerate() {
                                let is_selected = i == selected;
                                f.semantic(r, Role::Tab, item.title, is_selected, None, None, None);
                                let show_label = !item.title.is_empty()
                                    && match nav.label_mode {
                                        NavLabelMode::Labeled => true,
                                        NavLabelMode::Unlabeled => false,
                                        NavLabelMode::Selected => is_selected,
                                        NavLabelMode::Auto => count <= 3 || is_selected,
                                    };
                                let pill_w = state.dp(56.0).min((slot.width - state.dp(8.0)).max(0.0));
                                let pill_h = state.dp(32.0).min(slot.height);
                                let pill_y = if show_label {
                                    slot.y + ((slot.height - pill_h - gap - label_h) * 0.5).max(0.0)
                                } else {
                                    slot.y + (slot.height - pill_h) * 0.5
                                };
                                let pill = Rect {
                                    x: slot.x + (slot.width - pill_w) * 0.5,
                                    y: pill_y,
                                    width: pill_w,
                                    height: pill_h,
                                };
                                if is_selected {
                                    f.paint_rect(
                                        pill,
                                        pill_h * 0.5,
                                        t.secondary_container.opacity(disabled_alpha),
                                    );
                                }
                                let icon_base = if is_selected {
                                    t.on_secondary_container
                                } else {
                                    t.on_surface_variant
                                };
                                f.state_layer(Response { bounds: pill, ..r }, pill_h * 0.5, icon_base);
                                let icon_s = state.dp(24.0).min(pill.height).min(pill.width);
                                let icon_rect = Rect {
                                    x: pill.x + (pill.width - icon_s) * 0.5,
                                    y: pill.y + (pill.height - icon_s) * 0.5,
                                    width: icon_s,
                                    height: icon_s,
                                };
                                let icon_color = icon_base.opacity(disabled_alpha);
                                if !item.icon_name.is_empty() && let Some(img) = state.images.get(item.icon_name).cloned() {
                                    let tinted = tint_image(state, &img, Some(icon_color), 1);
                                    f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                                } else {
                                    let icon = parse_material_icon(item.icon_name).unwrap_or(MaterialIcon::Widgets);
                                    f.paint_material_icon(icon_rect, icon, icon_color);
                                }
                                if let Some(badge) = self.nav_badge_for_item(i, item.id, state) {
                                    let ext = self.material_badge_ext(Some(i), item.id);
                                    paint_nav_badge_ext(f, icon_rect, badge, ext, state, disabled_alpha);
                                }
                                if show_label {
                                    let label_rect = Rect {
                                        x: slot.x + state.dp(2.0),
                                        y: pill.y + pill.height + gap,
                                        width: (slot.width - state.dp(4.0)).max(0.0),
                                        height: label_h,
                                    };
                                    let label_color = if is_selected {
                                        t.text
                                    } else {
                                        t.on_surface_variant
                                    }
                                    .opacity(disabled_alpha);
                                    f.within(slot, |f| {
                                        f.styled_text(
                                            label_rect,
                                            item.title,
                                            label_color,
                                            TextStyle {
                                                size: label_size,
                                                weight: if is_selected { 700 } else { 500 },
                                                align: 1,
                                                ..TextStyle::default()
                                            },
                                        );
                                    });
                                }
                                f.focus_ring(r);
                            }
                        }
                        NavBarKind::Drawer => {
                            let row_h = state.dp(56.0);
                            let inset_x = state.dp(12.0);
                            let top_pad = state.dp(12.0);
                            let bottom_pad = state.dp(12.0);
                            let mut header_entries: Vec<(usize, &'static Node, u64, f32, f32, f32, f32)> = Vec::new();
                            let mut header_cursor = 0.0_f32;
                            for (ci, child) in self.children.iter().enumerate() {
                                if child.effective_visibility(state) == Visibility::Gone {
                                    continue;
                                }
                                let cid = child_view_id(id, ci, child);
                                let m = child.resolved_margin(state);
                                let max_w = (inner.width - m[0] - m[2]).max(0.0);
                                let cw = match child.width {
                                    Size::Match => max_w,
                                    Size::Dp(w) => state.dp(w).min(max_w),
                                    Size::Wrap => child
                                        .layout_intrinsic(true, state, max_w, f.ui.text_metrics, cid)
                                        .min(max_w),
                                }
                                .max(0.0);
                                let ch = match child.height {
                                    Size::Dp(h) => state.dp(h),
                                    Size::Match => (inner.height - m[1] - m[3]).max(0.0),
                                    Size::Wrap => child.layout_intrinsic(false, state, cw, f.ui.text_metrics, cid),
                                }
                                .max(0.0);
                                let right = match child.layout_gravity[0] {
                                    0 => state.rtl,
                                    2 => !state.rtl,
                                    4 => true,
                                    _ => false,
                                };
                                let rx = if child.layout_gravity[0] == 1 {
                                    inner.x + (inner.width - cw) * 0.5 + m[0] - m[2]
                                } else if right {
                                    inner.x + inner.width - cw - m[2]
                                } else {
                                    inner.x + m[0]
                                };
                                let rel_y = header_cursor + m[1];
                                header_entries.push((ci, child, cid, rx, rel_y, cw, ch));
                                header_cursor += m[1] + ch + m[3];
                            }
                            let divider_h = if header_entries.is_empty() { 0.0 } else { state.dp(1.0) };
                            let divider_y_rel = header_cursor;
                            let nav_start_rel = header_cursor + divider_h;
                            let total_content_h = nav_start_rel + top_pad + count as f32 * row_h + bottom_pad;
                            let max_scroll = (total_content_h - inner.height).max(0.0);
                            let prev_input = f.input;
                            let scroll_y = if max_scroll > 0.5 || state.offsets.get(&view_key).copied().unwrap_or(0.0) > 0.0 {
                                let offset = state.offsets.entry(view_key.clone()).or_default();
                                f.scroll_viewport(anonymous_child_id(id, 9899), inner, offset, max_scroll);
                                *offset
                            } else {
                                0.0
                            };
                            let needs_clip = max_scroll > 0.5 || !header_entries.is_empty();
                            let mut draw_drawer_content = |f: &mut Frame<'_>| {
                                for &(_, child, cid, rx, rel_y, cw, ch) in &header_entries {
                                    let child_rect = Rect {
                                        x: rx,
                                        y: inner.y + rel_y - scroll_y,
                                        width: cw,
                                        height: ch,
                                    };
                                    let mut child_action = None;
                                    if self.clip_children_disabled {
                                        child_action = child.draw(f, child_rect, state, cid);
                                    } else {
                                        f.within(child_rect, |f| {
                                            child_action = child.draw(f, child_rect, state, cid);
                                        });
                                    }
                                    if child_action.is_some() {
                                        action = child_action;
                                    }
                                }
                                if divider_h > 0.0 {
                                    f.paint_rect(
                                        Rect {
                                            x: inner.x,
                                            y: inner.y + divider_y_rel - scroll_y,
                                            width: inner.width,
                                            height: divider_h,
                                        },
                                        0.0,
                                        t.outline_variant,
                                    );
                                }
                                let mut responses = Vec::with_capacity(count);
                                for (i, item) in items.iter().enumerate() {
                                    let row = Rect {
                                        x: inner.x,
                                        y: inner.y + nav_start_rel + top_pad + i as f32 * row_h - scroll_y,
                                        width: inner.width,
                                        height: row_h,
                                    };
                                    let r = f.interact(anonymous_child_id(id, 9900 + i), row, enabled);
                                    if r.clicked {
                                        selected = i;
                                        state.selected_index.insert(view_key.clone(), i);
                                        action = Some(if !item.id.is_empty() { item.id } else { self.id });
                                    }
                                    responses.push((row, r));
                                }
                                let label_size = state.sp(14.0);
                                for (i, (item, (row, r))) in items.iter().zip(responses).enumerate() {
                                    let is_selected = i == selected;
                                    f.semantic(r, Role::ListItem, item.title, is_selected, None, None, None);
                                    let pill = Rect {
                                        x: row.x + inset_x,
                                        y: row.y,
                                        width: (row.width - inset_x * 2.0).max(0.0),
                                        height: row.height,
                                    };
                                    let pill_r = pill.height * 0.5;
                                    if is_selected {
                                        f.paint_rect(
                                            pill,
                                            pill_r,
                                            t.secondary_container.opacity(disabled_alpha),
                                        );
                                    }
                                    let fg_base = if is_selected {
                                        t.on_secondary_container
                                    } else {
                                        t.on_surface_variant
                                    };
                                    f.state_layer(Response { bounds: pill, ..r }, pill_r, fg_base);
                                    let fg_color = fg_base.opacity(disabled_alpha);
                                    let has_icon = !item.icon_name.is_empty();
                                    let mut badge_anchor = pill;
                                    if has_icon {
                                        let icon_s = state.dp(24.0).min(pill.height).min(pill.width);
                                        let icon_x = if state.rtl {
                                            row.x + row.width - state.dp(28.0) - icon_s
                                        } else {
                                            row.x + state.dp(28.0)
                                        };
                                        let icon_rect = Rect {
                                            x: icon_x,
                                            y: row.y + (row.height - icon_s) * 0.5,
                                            width: icon_s,
                                            height: icon_s,
                                        };
                                        badge_anchor = icon_rect;
                                        if let Some(img) = state.images.get(item.icon_name).cloned() {
                                            let tinted = tint_image(state, &img, Some(fg_color), 1);
                                            f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                                        } else if let Some(icon) = parse_material_icon(item.icon_name) {
                                            f.paint_material_icon(icon_rect, icon, fg_color);
                                        }
                                    }
                                    if let Some(badge) = self.nav_badge_for_item(i, item.id, state) {
                                        let ext = self.material_badge_ext(Some(i), item.id);
                                        paint_nav_badge_ext(f, badge_anchor, badge, ext, state, disabled_alpha);
                                    }
                                    let lead = state.dp(if has_icon { 64.0 } else { 28.0 });
                                    let trail = state.dp(28.0);
                                    let label_x = row.x + if state.rtl { trail } else { lead };
                                    let label_w = (row.width - lead - trail).max(0.0);
                                    f.within(pill, |f| {
                                        f.styled_text(
                                            Rect {
                                                x: label_x,
                                                y: row.y,
                                                width: label_w,
                                                height: row.height,
                                            },
                                            item.title,
                                            fg_color,
                                            TextStyle {
                                                size: label_size,
                                                weight: if is_selected { 700 } else { 500 },
                                                align: if state.rtl { 2 } else { 0 },
                                                ..TextStyle::default()
                                            },
                                        );
                                    });
                                    f.focus_ring(r);
                                }
                            };
                            if needs_clip {
                                f.within(inner, &mut draw_drawer_content);
                            } else {
                                draw_drawer_content(f);
                            }
                            f.input = prev_input;
                        }
                    }
                } else if let Some(spec) = til_spec {
                    let til_ext = self.text_input_ext();
                    let edit_info = find_text_input_edit(self, id);
                    let effective_hint = if !spec.hint.is_empty() {
                        spec.hint
                    } else if !self.hint.is_empty() {
                        self.hint
                    } else {
                        edit_info.as_ref().map_or("", |(n, _, _)| n.hint)
                    };
                    let error_key = format!("{view_key}:error");
                    let error_text = if spec.error_enabled {
                        state
                            .text
                            .get(&error_key)
                            .map(String::as_str)
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                    } else {
                        None
                    };
                    let char_len = edit_info.as_ref().map_or(0, |(n, _, k)| {
                        state.text.get(k).map_or(n.text, String::as_str).chars().count()
                    });
                    let counter_enabled = til_ext.is_some_and(|e| e.counter_enabled);
                    let counter_max = til_ext.map_or(0, |e| e.counter_max);
                    let counter_overflow = counter_enabled && counter_max > 0 && char_len > counter_max;
                    let counter_text = counter_enabled.then(|| {
                        if counter_max > 0 {
                            format!("{char_len} / {counter_max}")
                        } else {
                            format!("{char_len}")
                        }
                    });
                    let has_error = error_text.is_some() || counter_overflow;
                    let supporting_text: Option<String> = error_text
                        .or_else(|| (!spec.helper.is_empty()).then(|| spec.helper.to_owned()));
                    let label_size = state.sp(12.0);
                    let label_h = label_size.max(state.dp(12.0)) + state.dp(2.0);
                    let top_inset = if spec.mode == TextInputBoxMode::Outline && !effective_hint.is_empty() {
                        (label_h * 0.5).min(inner.height * 0.25)
                    } else {
                        0.0
                    };
                    let supporting_h = if supporting_text.is_some() || counter_text.is_some() {
                        (state.dp(4.0) + (label_size * 1.3333).max(state.dp(16.0)))
                            .min((inner.height - top_inset).max(0.0))
                    } else {
                        0.0
                    };
                    let box_rect = Rect {
                        x: inner.x,
                        y: inner.y + top_inset,
                        width: inner.width,
                        height: (inner.height - top_inset - supporting_h).max(0.0),
                    };
                    let d = state.effective_density();
                    let box_radii = spec.corner_radii_dp.map(|v| v * d);
                    let start_icon_name = til_ext
                        .map(|e| e.start_icon)
                        .filter(|s| !s.is_empty() && (state.images.contains_key(*s) || parse_material_icon(s).is_some()));
                    let has_start_icon = start_icon_name.is_some();
                    let start_icon_reserve = if has_start_icon {
                        state.dp(44.0).min(box_rect.width * 0.4)
                    } else {
                        0.0
                    };
                    let has_end_icon = spec.end_icon != TextInputEndIcon::None;
                    let btn_size = if has_end_icon {
                        state.dp(40.0).min(box_rect.height).min(box_rect.width * 0.5)
                    } else {
                        0.0
                    };
                    let btn_margin = state.dp(4.0);
                    let end_icon_reserve = if has_end_icon && btn_size > 0.0 {
                        (btn_size + btn_margin * 2.0).min((box_rect.width - start_icon_reserve).max(0.0))
                    } else {
                        0.0
                    };
                    let btn_rect = if has_end_icon && btn_size > 0.0 {
                        Rect {
                            x: if state.rtl {
                                box_rect.x + btn_margin
                            } else {
                                box_rect.x + box_rect.width - btn_size - btn_margin
                            },
                            y: box_rect.y + (box_rect.height - btn_size) * 0.5,
                            width: btn_size,
                            height: btn_size,
                        }
                    } else {
                        Rect::default()
                    };
                    let dd_key = edit_info
                        .as_ref()
                        .map(|(_, _, k)| k.clone())
                        .unwrap_or_else(|| view_key.clone());
                    let end_btn_response = if has_end_icon && btn_rect.width > 0.0 && btn_rect.height > 0.0 {
                        let btn_label = match spec.end_icon {
                            TextInputEndIcon::ClearText => "Clear text",
                            TextInputEndIcon::PasswordToggle => "Toggle password visibility",
                            TextInputEndIcon::DropdownMenu => "Show dropdown menu",
                            TextInputEndIcon::None => "",
                        };
                        let resp = f.target(anonymous_child_id(id, 9850), btn_rect, btn_label, enabled, Role::Button);
                        if resp.clicked {
                            match spec.end_icon {
                                TextInputEndIcon::ClearText => {
                                    if let Some((_, _, ref edit_key)) = edit_info {
                                        state.text.insert(edit_key.clone(), String::new());
                                    }
                                }
                                TextInputEndIcon::PasswordToggle => {
                                    if let Some((_, _, ref edit_key)) = edit_info {
                                        let entry = state
                                            .text_input_password_visible
                                            .entry(edit_key.clone())
                                            .or_insert(false);
                                        *entry = !*entry;
                                    }
                                }
                                TextInputEndIcon::DropdownMenu => {
                                    if let Some((_, edit_id, _)) = edit_info {
                                        f.request_focus(edit_id);
                                    }
                                    if state.til_dropdown_open.as_deref() == Some(dd_key.as_str()) {
                                        state.til_dropdown_open = None;
                                    } else {
                                        state.til_dropdown_open = Some(dd_key.clone());
                                    }
                                }
                                TextInputEndIcon::None => {}
                            }
                            action = Some(self.id);
                        }
                        Some(resp)
                    } else {
                        None
                    };
                    if spec.end_icon == TextInputEndIcon::DropdownMenu
                        && enabled
                        && state.til_dropdown_open.as_deref() == Some(dd_key.as_str())
                    {
                        state.til_dropdown_anchor = Some(TilDropdownAnchor {
                            key: dd_key.clone(),
                            til_key: view_key.clone(),
                            id,
                            xml_id: self.id,
                            bounds: box_rect,
                            rtl: state.rtl,
                        });
                    }
                    if spec.mode == TextInputBoxMode::Filled {
                        let container_color = if enabled {
                            explicit_background.unwrap_or(t.surface_highest)
                        } else {
                            t.text.opacity(0.04)
                        };
                        paint_rounded_rect_radii(f, box_rect, box_radii, container_color);
                    }
                    let content_rect = Rect {
                        x: box_rect.x + if state.rtl { end_icon_reserve } else { start_icon_reserve },
                        y: box_rect.y,
                        width: (box_rect.width - start_icon_reserve - end_icon_reserve).max(0.0),
                        height: box_rect.height,
                    };
                    let prev_til = state.text_input_active_box.replace(TextInputChildBox {
                        filled: spec.mode == TextInputBoxMode::Filled,
                        has_label: !effective_hint.is_empty(),
                        password_toggle: spec.end_icon == TextInputEndIcon::PasswordToggle,
                        has_start_icon,
                        prefix: til_ext.map_or("", |e| e.prefix),
                        suffix: til_ext.map_or("", |e| e.suffix),
                    });
                    for (i, n) in self.children.iter().enumerate() {
                        if n.effective_visibility(state) == Visibility::Gone {
                            continue;
                        }
                        let m = n.resolved_margin(state);
                        let child_r = Rect {
                            x: content_rect.x + m[0],
                            y: content_rect.y + m[1],
                            width: (content_rect.width - m[0] - m[2]).max(0.0),
                            height: (content_rect.height - m[1] - m[3]).max(0.0),
                        };
                        let ckey = child_view_id(id, i, n);
                        let mut child_action = None;
                        f.within(child_r, |f| {
                            child_action = n.draw(f, child_r, state, ckey);
                        });
                        if child_action.is_some() {
                            action = child_action;
                        }
                    }
                    state.text_input_active_box = prev_til;
                    if let Some(icon_name) = start_icon_name {
                        let icon_s = state.dp(24.0).min(box_rect.height).min(start_icon_reserve);
                        let icon_x = if state.rtl {
                            box_rect.x + box_rect.width - state.dp(12.0) - icon_s
                        } else {
                            box_rect.x + state.dp(12.0)
                        };
                        let icon_rect = Rect {
                            x: icon_x,
                            y: box_rect.y + (box_rect.height - icon_s) * 0.5,
                            width: icon_s,
                            height: icon_s,
                        };
                        let icon_color = t.on_surface_variant.opacity(if enabled { 1.0 } else { 0.38 });
                        if let Some(img) = state.images.get(icon_name).cloned() {
                            let tinted = tint_image(state, &img, Some(icon_color), 1);
                            f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                        } else if let Some(icon) = parse_material_icon(icon_name) {
                            f.paint_material_icon(icon_rect, icon, icon_color);
                        }
                    }
                    let edit_focused = enabled
                        && edit_info
                            .as_ref()
                            .is_some_and(|(_, edit_id, _)| f.ui.focused == Some(*edit_id));
                    let label_color = if !enabled {
                        t.text.opacity(0.38)
                    } else if has_error {
                        t.error
                    } else if edit_focused {
                        t.primary
                    } else {
                        t.on_surface_variant
                    };
                    let label_style = TextStyle {
                        size: label_size,
                        weight: 400,
                        align: if state.rtl { 2 } else { 0 },
                        ..TextStyle::default()
                    };
                    match spec.mode {
                        TextInputBoxMode::Outline => {
                            let stroke_w = state.dp(if edit_focused { 2.0 } else { 1.0 });
                            let stroke_color = if !enabled {
                                t.text.opacity(0.12)
                            } else if has_error {
                                t.error
                            } else if edit_focused {
                                t.primary
                            } else {
                                spec.stroke_color.unwrap_or(t.outline)
                            };
                            paint_outline_radii(f, box_rect, box_radii, stroke_w, stroke_color);
                            if !effective_hint.is_empty() {
                                let notch_pad = state.dp(4.0);
                                let max_label_w = (box_rect.width - state.dp(24.0) - notch_pad * 2.0).max(0.0);
                                let label_w = f.text_width(effective_hint, label_style).min(max_label_w);
                                let notch_w = label_w + notch_pad * 2.0;
                                let notch_x = if state.rtl {
                                    box_rect.x + box_rect.width - state.dp(12.0) - notch_w
                                } else {
                                    box_rect.x + state.dp(12.0)
                                };
                                let notch_h = stroke_w.max(state.dp(2.0)) * 2.0 + state.dp(2.0);
                                let notch_rect = Rect {
                                    x: notch_x,
                                    y: (box_rect.y - notch_h * 0.5).max(inner.y),
                                    width: notch_w,
                                    height: notch_h,
                                };
                                let notch_bg = explicit_background.unwrap_or(t.background);
                                f.paint_rect(notch_rect, 0.0, notch_bg);
                                let label_rect = Rect {
                                    x: notch_x + notch_pad,
                                    y: (box_rect.y - label_h * 0.5).max(inner.y),
                                    width: label_w,
                                    height: label_h,
                                };
                                f.styled_text(label_rect, effective_hint, label_color, label_style);
                            }
                        }
                        TextInputBoxMode::Filled => {
                            let ind_h = state.dp(if edit_focused { 2.0 } else { 1.0 });
                            let ind_color = if !enabled {
                                t.text.opacity(0.38)
                            } else if has_error {
                                t.error
                            } else if edit_focused {
                                t.primary
                            } else {
                                spec.stroke_color.unwrap_or(t.on_surface_variant)
                            };
                            f.paint_rect(
                                Rect {
                                    x: box_rect.x,
                                    y: box_rect.y + box_rect.height - ind_h,
                                    width: box_rect.width,
                                    height: ind_h,
                                },
                                0.0,
                                ind_color,
                            );
                            if !effective_hint.is_empty() {
                                let pad_start = if has_start_icon {
                                    start_icon_reserve + state.dp(4.0)
                                } else {
                                    state.dp(16.0)
                                };
                                let pad_end = end_icon_reserve + state.dp(16.0);
                                let label_x = box_rect.x + if state.rtl { pad_end } else { pad_start };
                                let label_w = (box_rect.width - pad_start - pad_end).max(0.0);
                                let label_rect = Rect {
                                    x: label_x,
                                    y: box_rect.y + state.dp(6.0),
                                    width: label_w,
                                    height: label_h,
                                };
                                f.styled_text(label_rect, effective_hint, label_color, label_style);
                            }
                        }
                        TextInputBoxMode::None => {}
                    }
                    if let Some(resp) = end_btn_response {
                        f.state_layer(resp, btn_rect.width.min(btn_rect.height) * 0.5, t.on_surface_variant);
                        let icon_side = state.dp(20.0).min(btn_rect.width).min(btn_rect.height);
                        let icon_rect = Rect {
                            x: btn_rect.x + (btn_rect.width - icon_side) * 0.5,
                            y: btn_rect.y + (btn_rect.height - icon_side) * 0.5,
                            width: icon_side,
                            height: icon_side,
                        };
                        match spec.end_icon {
                            TextInputEndIcon::ClearText => {
                                let has_text = edit_info.as_ref().is_some_and(|(n, _, k)| {
                                    !state.text.get(k).map_or(n.text, String::as_str).is_empty()
                                });
                                let color = t.on_surface_variant.opacity(if !enabled {
                                    0.38
                                } else if has_text {
                                    1.0
                                } else {
                                    0.45
                                });
                                f.paint_material_icon(icon_rect, MaterialIcon::Close, color);
                            }
                            TextInputEndIcon::PasswordToggle => {
                                let revealed = edit_info.as_ref().is_some_and(|(_, _, k)| {
                                    state.text_input_password_visible.get(k).copied().unwrap_or(false)
                                });
                                let color = t.on_surface_variant.opacity(if enabled { 1.0 } else { 0.38 });
                                paint_password_visibility_icon(f, icon_rect, revealed, color);
                            }
                            TextInputEndIcon::DropdownMenu => {
                                let color = t.on_surface_variant.opacity(if enabled { 1.0 } else { 0.38 });
                                let cx = icon_rect.x + icon_rect.width * 0.5;
                                let cy = icon_rect.y + icon_rect.height * 0.5;
                                let dx = state.dp(5.0).min(icon_rect.width * 0.35);
                                let dy = state.dp(2.5).min(icon_rect.height * 0.25);
                                let open = state.til_dropdown_open.as_deref() == Some(dd_key.as_str());
                                let points = if open {
                                    [[cx - dx, cy + dy], [cx, cy - dy], [cx + dx, cy + dy]]
                                } else {
                                    [[cx - dx, cy - dy], [cx, cy + dy], [cx + dx, cy - dy]]
                                };
                                f.commands.push(DrawCommand::Polyline {
                                    points,
                                    count: 3,
                                    width: state.dp(2.0),
                                    color,
                                });
                            }
                            TextInputEndIcon::None => {}
                        }
                    }
                    if supporting_text.is_some() || counter_text.is_some() {
                        let pad_x = state.dp(16.0);
                        let row_x = inner.x + pad_x;
                        let row_y = box_rect.y + box_rect.height + state.dp(4.0);
                        let row_w = (inner.width - pad_x * 2.0).max(0.0);
                        let row_h = (supporting_h - state.dp(4.0)).max(0.0);
                        let counter_reserve = if let Some(ref cnt) = counter_text {
                            let cnt_style = TextStyle {
                                align: if state.rtl { 0 } else { 2 },
                                ..label_style
                            };
                            let cnt_color = if !enabled {
                                t.text.opacity(0.38)
                            } else if counter_overflow {
                                t.error
                            } else {
                                t.on_surface_variant
                            };
                            let cnt_w = f.text_width(cnt, cnt_style).min(row_w);
                            f.styled_text(Rect { x: row_x, y: row_y, width: row_w, height: row_h }, cnt, cnt_color, cnt_style);
                            cnt_w + state.dp(8.0)
                        } else {
                            0.0
                        };
                        if let Some(ref msg) = supporting_text {
                            let supp_w = (row_w - counter_reserve).max(0.0);
                            let supp_x = if state.rtl { row_x + counter_reserve } else { row_x };
                            let supp_rect = Rect {
                                x: supp_x,
                                y: row_y,
                                width: supp_w,
                                height: row_h,
                            };
                            let supp_color = if !enabled {
                                t.text.opacity(0.38)
                            } else if has_error {
                                t.error
                            } else {
                                t.on_surface_variant
                            };
                            f.styled_text(supp_rect, msg, supp_color, label_style);
                        }
                    }
                } else {
                let mut inner = inner;
                if let Some(icon_name) = alert_dialog_icon
                    && !icon_name.is_empty()
                    && (state.images.contains_key(icon_name) || parse_material_icon(icon_name).is_some())
                {
                    let icon_s = state.dp(24.0).min(inner.width).min(inner.height);
                    let icon_rect = Rect {
                        x: inner.x + (inner.width - icon_s) * 0.5,
                        y: inner.y,
                        width: icon_s,
                        height: icon_s,
                    };
                    let icon_color = t.secondary.opacity(if enabled { 1.0 } else { 0.38 });
                    if let Some(img) = state.images.get(icon_name).cloned() {
                        let tinted = tint_image(state, &img, Some(icon_color), 1);
                        f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                    } else if let Some(icon) = parse_material_icon(icon_name) {
                        f.paint_material_icon(icon_rect, icon, icon_color);
                    }
                    let shift_y = state.dp(40.0).min(inner.height);
                    inner.y += shift_y;
                    inner.height = (inner.height - shift_y).max(0.0);
                }
                let prev_snackbar = if is_snackbar {
                    let prev = state.snackbar_active;
                    state.snackbar_active = true;
                    Some(prev)
                } else {
                    None
                };
                if let Some(cg) = self.chip_group_wrap_spec() {
                    let chip_children: Vec<(usize, &'static Node, String)> = self
                        .children
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| {
                            matches!(n.kind, Kind::Button)
                                && n.effective_visibility(state) != Visibility::Gone
                        })
                        .map(|(i, n)| {
                            let ck = child_view_id(id, i, n);
                            let name = if n.id.is_empty() { format!("#{ck}") } else { n.id.to_owned() };
                            (i, n, name)
                        })
                        .collect();
                    if cg.single_selection
                        && !chip_children.is_empty()
                        && !state.chip_groups_initialized.contains(&id)
                    {
                        state.chip_groups_initialized.insert(id);
                        let mut found = false;
                        for &(_, n, ref ck) in &chip_children {
                            let is_on = *state.checked.entry(ck.clone()).or_insert(n.checked);
                            if is_on && !found {
                                found = true;
                            } else if is_on {
                                state.checked.insert(ck.clone(), false);
                            }
                        }
                    }
                    let (cg_rects, _, _) = self.chip_group_wrap_rects(inner, cg, state, f.ui.text_metrics, id);
                    for (i, n) in self.children.iter().enumerate() {
                        let Some(r) = cg_rects[i] else { continue; };
                        let key = child_view_id(id, i, n);
                        let child_key = if n.id.is_empty() { format!("#{key}") } else { n.id.to_owned() };
                        let was_checked = state.checked.get(&child_key).copied().unwrap_or(n.checked);
                        let mut child_action = None;
                        let mut draw_child = |f: &mut Frame<'_>| {
                            if self.clip_children_disabled {
                                child_action = n.draw(f, r, state, key);
                            } else {
                                f.within(r, |f| {
                                    child_action = n.draw(f, r, state, key);
                                });
                            }
                        };
                        if !self.clip_to_padding_disabled && inner != b {
                            f.within(inner, &mut draw_child);
                        } else {
                            draw_child(f);
                        }
                        if let Some(a) = child_action {
                            if cg.single_selection && matches!(n.kind, Kind::Button) {
                                let now_checked = state.checked.get(&child_key).copied().unwrap_or(n.checked);
                                let is_filter = n.chip_style().is_some_and(|c| c.kind == XmlChipKind::Filter);
                                let become_checked = if is_filter { now_checked } else { !was_checked };
                                if become_checked {
                                    for &(_, _, ref sib_key) in &chip_children {
                                        state.checked.insert(sib_key.clone(), *sib_key == child_key);
                                    }
                                } else {
                                    state.checked.insert(child_key, false);
                                }
                                f.continuous(b);
                            }
                            action = Some(a);
                        }
                    }
                    if let Some(prev) = prev_snackbar {
                        state.snackbar_active = prev;
                    }
                } else {
                let media_key=if matches!(self.kind,Kind::MediaController){Some(if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()})}else{None};
                let media_namespace=if media_key.is_some(){let previous=state.adapter_namespace;state.adapter_namespace=Some(id);Some(previous)}else{None};
                let media_previous_capabilities=media_key.as_ref().map(|key|{let previous=state.media_active_capabilities;let capabilities=if state.media_players.contains_key(key){state.media_controller_capabilities.get(key).copied().unwrap_or_default()}else{MediaControllerCapabilities{can_pause:false,can_seek_backward:false,can_seek_forward:false,has_previous:false,has_next:false}};state.media_active_capabilities=Some(capabilities);previous});
                let media_saved=media_key.as_ref().map(|_|(["time_current","time","pause"].map(|key|state.text.get(key).cloned()),state.progress.get("mediacontroller_progress").copied(),state.secondary_progress.get("mediacontroller_progress").copied(),state.image_sources.get("pause").cloned()));
                if let Some(snapshot)=media_key.as_ref().and_then(|key|state.media_players.get(key)).copied() {
                    let time=|ms:u64|{let seconds=ms/1000;let hours=seconds/3600;if hours>0{format!("{}:{:02}:{:02}",hours,(seconds/60)%60,seconds%60)}else{format!("{}:{:02}",seconds/60,seconds%60)}};
                    let slider_value=media_key.as_ref().and_then(|key|state.media_seek_progress.get(key)).copied().unwrap_or_else(||if snapshot.duration_ms>0{(snapshot.position_ms.min(snapshot.duration_ms) as f64*1000.0/snapshot.duration_ms as f64) as f32}else{0.0});
                    let displayed_position=if media_key.as_ref().is_some_and(|key|state.media_seek_progress.contains_key(key)){(slider_value as f64*snapshot.duration_ms as f64/1000.0) as u64}else{snapshot.position_ms};
                    state.text.insert("time_current".into(),time(displayed_position));state.text.insert("time".into(),time(snapshot.duration_ms));state.text.insert("pause".into(),if snapshot.playing{"Pause"}else{"Play"}.into());
                    let icon=if snapshot.playing{"android_media_pause"}else{"android_media_play"};if state.images.contains_key(icon){state.image_sources.insert("pause".into(),icon.into());}
                    state.progress.insert("mediacontroller_progress".into(),slider_value);
                    state.secondary_progress.insert("mediacontroller_progress".into(),snapshot.buffered_percent.min(100) as f32*10.0);
                }
                let radio_group = matches!(self.kind, Kind::RadioGroup);
                let selected_child = if matches!(self.kind, Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper) {
                    let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                    let selected = state.displayed_children.entry(key.clone()).or_default();
                    if *selected >= self.children.len() { *selected = 0; }
                    if matches!(self.kind, Kind::ViewFlipper) {
                        let now = f.time();
                        let running = *state.flipper_running.entry(key).or_insert(self.auto_start);
                        let viewport = b.intersect(f.clip);
                        let visible = viewport.width > 0.0 && viewport.height > 0.0;
                        let epoch = f.ui.epoch; let pause_generation = f.ui.pause_generation;
                        let clock = state.flipper_clocks.entry(id).or_insert(FlipperClock { running, next: now + self.flip_interval, epoch, pause_generation, visible });
                        if running != clock.running || (visible && (!clock.visible || epoch.saturating_sub(clock.epoch) > 1 || clock.pause_generation != pause_generation)) {
                            *clock = FlipperClock { running, next: now + self.flip_interval, epoch, pause_generation, visible };
                        }
                        clock.epoch = epoch; clock.pause_generation = pause_generation; clock.visible = visible;
                        if running && visible && self.children.len() > 1 && self.flip_interval > 0.0 {
                            if now >= clock.next { *selected = (*selected + 1) % self.children.len(); clock.next = now + self.flip_interval; }
                            f.wake_at(clock.next);
                        }
                    }
                    Some(*selected)
                } else { None };
                let scale_in = self.scale_anim(RULE_SCALE_IN);
                let scale_out = self.scale_anim(RULE_SCALE_OUT);
                let rotate_in = self.rotate_anim(RULE_ROTATE_IN);
                let rotate_out = self.rotate_anim(RULE_ROTATE_OUT);
                let elev_in = self.elevation_anim(RULE_ELEVATION_IN);
                let elev_out = self.elevation_anim(RULE_ELEVATION_OUT);
                let transition = selected_child.map(|selected| {
                    let now = f.time();
                    let transition = state.transitions.entry(id).or_insert(ViewTransition { current:selected, previous:selected, started:now, active:false });
                    if transition.current != selected {
                        *transition = ViewTransition { previous:transition.current, current:selected, started:now, active:true };
                    }
                    let duration = self.animation_in.map_or(0.0, |a| a.total_duration())
                        .max(self.animation_out.map_or(0.0, |a| a.total_duration()))
                        .max(self.translate_in.map_or(0.0, |a| a.total_duration()))
                        .max(self.translate_out.map_or(0.0, |a| a.total_duration()))
                        .max(scale_in.map_or(0.0, |a| a.total_duration()))
                        .max(scale_out.map_or(0.0, |a| a.total_duration()))
                        .max(rotate_in.map_or(0.0, |a| a.total_duration()))
                        .max(rotate_out.map_or(0.0, |a| a.total_duration()))
                        .max(elev_in.map_or(0.0, |a| a.total_duration()))
                        .max(elev_out.map_or(0.0, |a| a.total_duration()));
                    if f.ui.motion_preference == MotionPreference::Reduced || now - transition.started >= duration { transition.active = false; }
                    if transition.active { f.ui.continuous = true; }
                    *transition
                });
                if radio_group && !state.radio_groups.contains_key(&id) {
                    let direct = self.children.iter().enumerate().rev().find(|(_, child)| matches!(child.kind, Kind::RadioButton) && child.checked)
                        .map(|(i, child)| if child.id.is_empty() { anonymous_child_id(id, i) } else { view_id(child.id) });
                    let referenced=if self.checked_button.is_empty(){None}else{self.children.iter().find_map(|child|tab_host_find(child,self.checked_button)).filter(|child|matches!(child.kind,Kind::RadioButton))};
                    let initial=direct.or_else(||referenced.map(|child|view_id(child.id)));
                    if direct.is_none() && let Some(child)=referenced {state.checked.insert(child.id.to_owned(),true);}
                    state.radio_groups.insert(id, initial);
                }
                if radio_group {
                    let group=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                    if let Some(request)=state.radio_group_requests.remove(&group) {
                        let next=request.as_deref().map(view_id);
                        let old=state.radio_groups.get(&id).copied().flatten();
                        if old!=next {
                            if let Some(child)=old.and_then(|value|radio_descendant_by_id(self,value)).filter(|child|matches!(child.kind,Kind::RadioButton)) {state.checked.insert(child.id.to_owned(),false);}
                            if let Some(child)=next.and_then(|value|radio_descendant_by_id(self,value)).filter(|child|matches!(child.kind,Kind::RadioButton)) {state.checked.insert(child.id.to_owned(),true);}
                            state.radio_groups.insert(id,next);
                        }
                        // RadioGroup.check returns early for the same non-empty ID,
                        // but clearCheck notifies even when it is already clear.
                        if old!=next || next.is_none() {state.record_radio_group_event(&group,next);}
                    }
                }
                let horizontal = self.horizontal || matches!(self.kind,Kind::TableRow);
                let toggle_group = self.toggle_group_spec().filter(|_| horizontal);
                let toggle_buttons: Vec<(usize, &'static Node, String)> = if toggle_group.is_some() {
                    self.children
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| {
                            matches!(n.kind, Kind::Button)
                                && n.effective_visibility(state) != Visibility::Gone
                        })
                        .map(|(i, n)| {
                            let ck = child_view_id(id, i, n);
                            let name = if n.id.is_empty() {
                                format!("#{ck}")
                            } else {
                                n.id.to_owned()
                            };
                            (i, n, name)
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                if let Some(tg) = toggle_group
                    && !toggle_buttons.is_empty()
                    && !state.toggle_groups_initialized.contains(&id)
                {
                    state.toggle_groups_initialized.insert(id);
                    let any_preset = toggle_buttons
                        .iter()
                        .any(|(_, _, k)| state.checked.contains_key(k));
                    for &(_, n, ref child_key) in &toggle_buttons {
                        state.checked.entry(child_key.clone()).or_insert_with(|| {
                            if any_preset {
                                false
                            } else if !tg.checked_id.is_empty() {
                                n.id == tg.checked_id
                            } else {
                                n.checked
                            }
                        });
                    }
                    if tg.single_selection {
                        let mut found = false;
                        for &(_, _, ref child_key) in &toggle_buttons {
                            let is_on = state.checked.get(child_key).copied().unwrap_or(false);
                            if is_on && !found {
                                found = true;
                            } else if is_on {
                                state.checked.insert(child_key.clone(), false);
                            }
                        }
                    }
                    if tg.selection_required {
                        let any_on = toggle_buttons
                            .iter()
                            .any(|(_, _, k)| state.checked.get(k).copied().unwrap_or(false));
                        if !any_on && let Some((_, _, first_key)) = toggle_buttons.first() {
                            state.checked.insert(first_key.clone(), true);
                        }
                    }
                }
                let has_collapsing_child = (!horizontal || matches!(self.kind, Kind::Frame))
                    && self.children.iter().any(|c| {
                        c.contains_collapsing_toolbar()
                            || c.contains_lift_on_scroll()
                            || c.contains_app_bar_scroll_flags()
                            || c.contains_hide_on_scroll_bar()
                    });
                let collapsing_sib_before = if has_collapsing_child {
                    self.sibling_scroll_offset(state, id)
                } else {
                    None
                };
                let prev_collapsing_scroll = state.collapsing_scroll_offset;
                if let Some(off) = collapsing_sib_before {
                    state.collapsing_scroll_offset = Some(off);
                }
                struct AppBarScrollUpdate {
                    ci: usize,
                    natural_h: f32,
                    eff_top: f32,
                    eff_bot: f32,
                    body_collapse: f32,
                    compress: bool,
                    commit: Option<(String, AppBarScrollState)>,
                }
                let appbar_scroll_updates: Vec<AppBarScrollUpdate> =
                    if !horizontal
                        && matches!(self.kind, Kind::Linear)
                        && self.children.iter().any(|c| c.app_bar_scroll_flags().is_some())
                    {
                        let parent_scroll = self.app_bar_scroll_px(state, id);
                        let metrics_ab = f.ui.text_metrics;
                        let mut updates = Vec::new();
                        for (ci, cn) in self.children.iter().enumerate() {
                            if cn.effective_visibility(state) == Visibility::Gone
                                || cn.collapsing_toolbar_spec().is_some()
                            {
                                continue;
                            }
                            let Some(flags) = cn.app_bar_scroll_flags() else {
                                continue;
                            };
                            let cid = child_view_id(id, ci, cn);
                            let cm = cn.resolved_margin(state);
                            let avail_w = (inner.width - cm[0] - cm[2]).max(0.0);
                            let natural_h = cn.layout_intrinsic(false, state, avail_w, metrics_ab, cid);
                            let sp = cn.app_bar_child_scroll_px(parent_scroll, state, cid);
                            let Some(eval) = eval_app_bar_child_collapse(cn, cid, natural_h, cm[1] + cm[3], sp, state) else {
                                continue;
                            };
                            if eval.continuous {
                                f.continuous(b);
                            } else if let Some(wake) = eval.wake_at {
                                let vis_rect = b.intersect(f.clip);
                                if vis_rect.width > 0.0 && vis_rect.height > 0.0 {
                                    f.wake_at(wake);
                                }
                            }
                            let (eff_top, _, eff_bot, body_collapse) = app_bar_child_collapsed_layout(
                                natural_h,
                                cm[1],
                                cm[3],
                                eval.collapse_px,
                                flags.snap_margins,
                            );
                            let commit = (flags.enter_always || flags.snap).then(|| {
                                let child_key = if cn.id.is_empty() {
                                    format!("#{cid}")
                                } else {
                                    cn.id.to_owned()
                                };
                                (child_key, eval.next_state)
                            });
                            updates.push(AppBarScrollUpdate {
                                ci,
                                natural_h,
                                eff_top,
                                eff_bot,
                                body_collapse,
                                compress: cn.app_bar_scroll_effect_compress(),
                                commit,
                            });
                        }
                        updates
                    } else {
                        Vec::new()
                    };
                let size = if horizontal {
                    inner.width
                } else {
                    inner.height
                };
                let metrics = f.ui.text_metrics;
                let mut lengths = self.linear_main_lengths(horizontal, state, size, inner.width, metrics, id);
                // LinearLayout remeasures weighted children with an EXACTLY spec for
                // their allocated main-axis share, then uses the child's measured size.
                for (i,n) in self.children.iter().enumerate() {
                    if !matches!(self.kind,Kind::Linear|Kind::RadioGroup|Kind::TableRow)||!matches!(n.kind,Kind::VideoView)||n.weight<=0.0||n.effective_visibility(state)==Visibility::Gone {continue;}
                    let name=if n.id.is_empty(){format!("#{}",child_view_id(id,i,n))}else{n.id.to_owned()};let source=state.video_dimensions.get(&name).copied().or_else(||state.video_frames.get(&name).map(|frame|(frame.width(),frame.height()))).unwrap_or((0,0));
                    let margin=n.resolved_margin(state);let cross=if horizontal{(inner.height-margin[1]-margin[3]).max(0.0)}else{(inner.width-margin[0]-margin[2]).max(0.0)};
                    let cross_spec=|size:Size|match size {Size::Dp(value)=>VideoMeasureSpec::Exactly(state.dp(value).max(0.0) as u32),Size::Match=>VideoMeasureSpec::Exactly(cross as u32),Size::Wrap=>VideoMeasureSpec::AtMost(cross as u32)};
                    let main=VideoMeasureSpec::Exactly(lengths[i].max(0.0) as u32);let (width,height)=if horizontal{measure_video_view(source,main,cross_spec(n.height))}else{measure_video_view(source,cross_spec(n.width),main)};
                    lengths[i]=if horizontal{width as f32}else{height as f32};
                }
                let row_baselines=if horizontal && matches!(self.kind,Kind::TableRow|Kind::Linear|Kind::RadioGroup) {Some(self.standalone_row_metrics(state,metrics,id,&lengths))} else {None};
                let divider_thickness = state.dp(self.divider_thickness);
                let divider_padding = state.dp(self.divider_padding);
                let extra_gap = if matches!(self.kind, Kind::Linear) { self.linear_extra_gap(state) } else { 0.0 };
                let visible_child_count = self.children.iter().filter(|n| n.effective_visibility(state) != Visibility::Gone).count();
                let used: f32 = self.children.iter().enumerate().filter(|(_,n)|n.effective_visibility(state)!=Visibility::Gone).map(|(i,n)| {
                    let m_axis = appbar_scroll_updates
                        .iter()
                        .find(|u| u.ci == i)
                        .map_or_else(|| n.margin_for_axis(horizontal, state), |u| u.eff_top + u.eff_bot);
                    lengths[i] + m_axis
                }).sum::<f32>() + if matches!(self.kind,Kind::Linear|Kind::RadioGroup|Kind::TableRow) { self.divider_count(state) as f32*divider_thickness + visible_child_count.saturating_sub(1) as f32 * extra_gap } else { 0.0 };
                let remaining = (size-used).max(0.0);
                let main_gravity = if horizontal { self.content_gravity[0] } else { self.content_gravity[1] };
                let trailing = if horizontal { match main_gravity { 0 => state.rtl, 2 => !state.rtl, 4 => true, _ => false } } else { main_gravity == 2 };
                let shift = if main_gravity == 1 { remaining * 0.5 } else if trailing { remaining } else { 0.0 };
                let mut cursor = if horizontal && state.rtl { inner.x + inner.width - remaining + shift } else if horizontal { inner.x + shift } else { inner.y + shift };
                let order: Vec<usize> = if let Some(selected) = selected_child {
                    let mut result = Vec::with_capacity(2);
                    if let Some(t) = transition.filter(|t| {
                        t.active
                            && t.previous != selected
                            && (self.animation_out.is_some_and(|a| f.time() - t.started < a.total_duration())
                                || self.translate_out.is_some_and(|a| f.time() - t.started < a.total_duration())
                                || scale_out.is_some_and(|a| f.time() - t.started < a.total_duration())
                                || rotate_out.is_some_and(|a| f.time() - t.started < a.total_duration())
                                || elev_out.is_some_and(|a| f.time() - t.started < a.total_duration()))
                    }) {
                        result.push(t.previous);
                    }
                    if selected < self.children.len() { result.push(selected); }
                    result
                } else { (0..self.children.len()).collect() };
                struct DrawerSlot {
                    motion_key: String,
                    open_key: String,
                    end_only_keys: Option<(String, String)>,
                    open_now: bool,
                    lock_mode: u8,
                    instant: bool,
                    panels: Vec<(usize, &'static Node, Rect, bool)>,
                    dm: XmlDrawerMotion,
                    edge_was_dragging: bool,
                    was_dragging: bool,
                }
                let is_drawer = self.is_drawer_layout();
                let mut drawer_slots: Vec<DrawerSlot> = Vec::new();
                if is_drawer {
                    let now = f.time();
                    let max_panel_w = (b.width - state.dp(56.0)).max(0.0);
                    let default_w = state.dp(300.0).min(max_panel_w);
                    let mut start_panels: Vec<(usize, &'static Node, Rect, bool)> = Vec::new();
                    let mut end_panels: Vec<(usize, &'static Node, Rect, bool)> = Vec::new();
                    for (ci, cn) in self.children.iter().enumerate() {
                        if cn.effective_visibility(state) == Visibility::Gone || !cn.is_drawer_panel_child() {
                            continue;
                        }
                        let panel_w = match cn.width {
                            Size::Dp(w) => state.dp(w).min(max_panel_w),
                            Size::Wrap | Size::Match => default_w,
                        };
                        let panel_h = match cn.height {
                            Size::Dp(h) => state.dp(h).min(b.height),
                            Size::Wrap | Size::Match => b.height,
                        };
                        let align_right = match cn.layout_gravity[0] {
                            2 => !state.rtl,
                            3 => false,
                            4 => true,
                            _ => state.rtl,
                        };
                        let panel_x = if align_right { b.x + b.width - panel_w } else { b.x };
                        let panel_y = match cn.layout_gravity[1] {
                            1 => b.y + (b.height - panel_h) * 0.5,
                            2 => b.y + b.height - panel_h,
                            _ => b.y,
                        };
                        let entry = (
                            ci,
                            cn,
                            Rect { x: panel_x, y: panel_y, width: panel_w, height: panel_h },
                            align_right,
                        );
                        if cn.drawer_panel_is_logical_end(state.rtl) {
                            end_panels.push(entry);
                        } else {
                            start_panels.push(entry);
                        }
                    }
                    let end_key = format!("{view_key}:end");
                    let has_start = !start_panels.is_empty();
                    let mut raw_slots: Vec<(String, String, Option<(String, String)>, bool, u8, Vec<(usize, &'static Node, Rect, bool)>)> = Vec::new();
                    if has_start {
                        let child_lock = start_panels.iter().find_map(|(_, cn, _, _)| cn.static_drawer_lock_mode());
                        let lock_mode = self.resolved_drawer_lock_mode(false, has_start, child_lock, state, &view_key, &end_key);
                        let raw_open = state.drawer_open.get(&view_key).copied().unwrap_or(false);
                        let open_now = match lock_mode {
                            DRAWER_LOCK_MODE_LOCKED_CLOSED => {
                                if raw_open {
                                    state.drawer_open.insert(view_key.clone(), false);
                                }
                                false
                            }
                            DRAWER_LOCK_MODE_LOCKED_OPEN => {
                                if !raw_open {
                                    state.drawer_open.insert(view_key.clone(), true);
                                }
                                true
                            }
                            _ => raw_open,
                        };
                        raw_slots.push((view_key.clone(), view_key.clone(), None, open_now, lock_mode, start_panels));
                    }
                    if !end_panels.is_empty() {
                        let child_lock = end_panels.iter().find_map(|(_, cn, _, _)| cn.static_drawer_lock_mode());
                        let lock_mode = self.resolved_drawer_lock_mode(true, has_start, child_lock, state, &view_key, &end_key);
                        if has_start {
                            let raw_open = state.drawer_open.get(&end_key).copied().unwrap_or(false);
                            let open_now = match lock_mode {
                                DRAWER_LOCK_MODE_LOCKED_CLOSED => {
                                    if raw_open {
                                        state.drawer_open.insert(end_key.clone(), false);
                                    }
                                    false
                                }
                                DRAWER_LOCK_MODE_LOCKED_OPEN => {
                                    if !raw_open {
                                        state.drawer_open.insert(end_key.clone(), true);
                                    }
                                    true
                                }
                                _ => raw_open,
                            };
                            raw_slots.push((end_key.clone(), end_key.clone(), None, open_now, lock_mode, end_panels));
                        } else {
                            let end_open = state.drawer_open.get(&end_key).copied().unwrap_or(false);
                            let prim_open = state.drawer_open.get(&view_key).copied().unwrap_or(false);
                            let use_end_key = end_open
                                || (state.drawer_open.contains_key(&end_key)
                                    && !state.drawer_open.contains_key(&view_key));
                            let open_key = if use_end_key { end_key.clone() } else { view_key.clone() };
                            let raw_open = end_open || prim_open;
                            let open_now = match lock_mode {
                                DRAWER_LOCK_MODE_LOCKED_CLOSED => {
                                    if prim_open {
                                        state.drawer_open.insert(view_key.clone(), false);
                                    }
                                    if end_open {
                                        state.drawer_open.insert(end_key.clone(), false);
                                    }
                                    false
                                }
                                DRAWER_LOCK_MODE_LOCKED_OPEN => {
                                    if !raw_open {
                                        state.drawer_open.insert(open_key.clone(), true);
                                    }
                                    true
                                }
                                _ => raw_open,
                            };
                            raw_slots.push((
                                view_key.clone(),
                                open_key,
                                Some((view_key.clone(), end_key.clone())),
                                open_now,
                                lock_mode,
                                end_panels,
                            ));
                        }
                    }
                    for (motion_key, open_key, end_only_keys, open_now, lock_mode, panels) in raw_slots {
                        let mut dm = state.drawer_motion.remove(&motion_key).unwrap_or_default();
                        let instant = f.reduced_motion() || (now == 0.0 && dm.last_time == 0.0);
                        if !dm.initialized {
                            dm.initialized = true;
                            dm.last_open = open_now;
                            dm.last_time = now;
                            dm.progress = if open_now { 1.0 } else { 0.0 };
                        } else {
                            if let Some((start, from, to, dur)) = dm.anim {
                                if instant || dur <= 0.0 {
                                    dm.progress = to;
                                    dm.anim = None;
                                } else {
                                    let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                                    let ease = 1.0 - (1.0 - p) * (1.0 - p);
                                    dm.progress = (from + (to - from) * ease).clamp(0.0, 1.0);
                                    if p >= 1.0 {
                                        dm.progress = to;
                                        dm.anim = None;
                                    } else {
                                        f.continuous(b);
                                    }
                                }
                            }
                            if open_now != dm.last_open {
                                dm.last_open = open_now;
                                let target = if open_now { 1.0 } else { 0.0 };
                                dm.origin = None;
                                dm.dragging = false;
                                dm.edge_opening = false;
                                dm.samples.clear();
                                if instant || (dm.progress - target).abs() <= 0.001 {
                                    dm.progress = target;
                                    dm.anim = None;
                                } else {
                                    dm.anim = Some((now, dm.progress, target, 0.25));
                                    f.continuous(b);
                                }
                            }
                            dm.last_time = now;
                        }
                        drawer_slots.push(DrawerSlot {
                            motion_key,
                            open_key,
                            end_only_keys,
                            open_now,
                            lock_mode,
                            instant,
                            panels,
                            dm,
                            edge_was_dragging: false,
                            was_dragging: false,
                        });
                    }
                    let any_open_or_closing = drawer_slots.iter().any(|s| {
                        s.open_now
                            || s.dm.last_open
                            || (!s.dm.edge_opening && (s.dm.progress > 0.001 || s.dm.dragging))
                    });
                    if !any_open_or_closing {
                        let edge_slop = state.dp(24.0);
                        let drag_slop = state.dp(8.0);
                        let flick_v = state.dp(500.0);
                        for slot in &mut drawer_slots {
                            if slot.lock_mode == DRAWER_LOCK_MODE_LOCKED_CLOSED
                                || slot.lock_mode == DRAWER_LOCK_MODE_LOCKED_OPEN
                            {
                                slot.dm.origin = None;
                                slot.dm.dragging = false;
                                slot.dm.edge_opening = false;
                                slot.dm.samples.clear();
                                slot.edge_was_dragging = false;
                                continue;
                            }
                            if enabled && f.input.pressed && !slot.dm.last_open && f.pointer_in(b)
                                && let Some(ptr) = f.input.pointer
                            {
                                let edge_panel = slot.panels.iter().find(|&&(_, _, open_r, align_right)| {
                                    open_r.width > 0.0
                                        && if align_right {
                                            ptr[0] >= b.x + b.width - edge_slop && ptr[0] <= b.x + b.width
                                        } else {
                                            ptr[0] >= b.x && ptr[0] <= b.x + edge_slop
                                        }
                                });
                                if let Some(&(_, _, open_r, align_right)) = edge_panel {
                                    slot.dm.origin = Some(ptr);
                                    slot.dm.drag_base_progress = slot.dm.progress;
                                    slot.dm.drag_align_right = align_right;
                                    slot.dm.drag_panel_w = open_r.width.max(1.0);
                                    slot.dm.dragging = false;
                                    slot.dm.edge_opening = true;
                                    slot.dm.anim = None;
                                    slot.dm.samples.clear();
                                    slot.dm.samples.push((now, ptr[0]));
                                }
                            }
                            if slot.dm.edge_opening
                                && let (Some(origin), Some(pointer)) = (slot.dm.origin, f.input.pointer)
                                && !f.input.cancelled
                                && !f.input.blurred
                            {
                                if slot.dm.samples.last().map_or(true, |&(_, x)| (x - pointer[0]).abs() > 0.01) {
                                    slot.dm.samples.push((now, pointer[0]));
                                    if slot.dm.samples.len() > 8 {
                                        slot.dm.samples.remove(0);
                                    }
                                }
                                let dx = pointer[0] - origin[0];
                                let dy = pointer[1] - origin[1];
                                let inward_dx = if slot.dm.drag_align_right { -dx } else { dx };
                                if !slot.dm.dragging {
                                    if inward_dx > drag_slop && dx.abs() >= dy.abs() {
                                        slot.dm.dragging = true;
                                        f.ui.active = None;
                                    } else if dy.abs() > drag_slop && dy.abs() > dx.abs() {
                                        slot.dm.origin = None;
                                        slot.dm.edge_opening = false;
                                        slot.dm.samples.clear();
                                    }
                                }
                                if slot.dm.dragging {
                                    slot.dm.progress = (inward_dx / slot.dm.drag_panel_w.max(1.0)).clamp(0.0, 1.0);
                                    if !f.input.released {
                                        f.continuous(b);
                                    }
                                }
                            }
                            slot.edge_was_dragging = slot.dm.edge_opening && slot.dm.dragging;
                            if slot.dm.edge_opening
                                && slot.dm.origin.is_some()
                                && (f.input.released || f.input.cancelled || f.input.blurred)
                            {
                                if slot.dm.dragging && !f.input.cancelled && !f.input.blurred {
                                    let vx = pointer_velocity(&slot.dm.samples, now);
                                    let v_inward = if slot.dm.drag_align_right { -vx } else { vx };
                                    let should_open = slot.dm.progress > 0.30 || v_inward > flick_v;
                                    if should_open {
                                        state.drawer_open.insert(slot.open_key.clone(), true);
                                        slot.open_now = true;
                                        slot.dm.last_open = true;
                                        if slot.instant || (1.0 - slot.dm.progress).abs() <= 0.001 {
                                            slot.dm.progress = 1.0;
                                            slot.dm.anim = None;
                                        } else {
                                            slot.dm.anim = Some((now, slot.dm.progress, 1.0, 0.25));
                                            f.continuous(b);
                                        }
                                    } else {
                                        slot.dm.last_open = false;
                                        if slot.instant || slot.dm.progress <= 0.001 {
                                            slot.dm.progress = 0.0;
                                            slot.dm.anim = None;
                                        } else {
                                            slot.dm.anim = Some((now, slot.dm.progress, 0.0, 0.25));
                                            f.continuous(b);
                                        }
                                    }
                                } else if slot.dm.dragging {
                                    if slot.instant || slot.dm.progress <= 0.001 {
                                        slot.dm.progress = 0.0;
                                        slot.dm.anim = None;
                                    } else {
                                        slot.dm.anim = Some((now, slot.dm.progress, 0.0, 0.25));
                                        f.continuous(b);
                                    }
                                }
                                slot.dm.origin = None;
                                slot.dm.dragging = false;
                                slot.dm.edge_opening = false;
                                slot.dm.samples.clear();
                            }
                        }
                    }
                }
                let drawer_blocking = drawer_slots.iter().any(|s| {
                    s.open_now || s.dm.progress > 0.001 || s.dm.dragging || s.edge_was_dragging
                });
                let is_srl = self.is_swipe_refresh();
                let srl_ext = if is_srl { self.swipe_refresh_ext() } else { None };
                let mut srl_state: Option<(SwipeRefreshMotion, f32, f32, f32, bool)> = None;
                let mut srl_dragging = false;
                if is_srl {
                    let now = if f.time().is_finite() { f.time().max(0.0) } else { 0.0 };
                    let start_off_px = state.dp(srl_ext.as_ref().map_or(-40.0, |e| e.start_off_dp));
                    let end_off_px = state.dp(srl_ext.as_ref().map_or(64.0, |e| e.end_off_dp));
                    let settled_off_px = if srl_ext.is_some() { end_off_px } else { state.dp(16.0) };
                    let trigger_px = if end_off_px > 0.0 {
                        end_off_px
                    } else {
                        (end_off_px - start_off_px).max(state.dp(1.0))
                    };
                    let mut is_refreshing = state.refreshing.get(&view_key).copied().unwrap_or(false);
                    let mut srm = state.swipe_refresh_motion.remove(&view_key).unwrap_or_default();
                    let instant = f.reduced_motion() || (now == 0.0 && srm.last_time == 0.0);
                    if !srm.initialized {
                        srm.initialized = true;
                        srm.last_refreshing = is_refreshing;
                        srm.last_time = now;
                        srm.offset_px = if is_refreshing { settled_off_px } else { start_off_px };
                        srm.settle = None;
                    } else {
                        if let Some((start, from, to, dur)) = srm.settle {
                            if f.reduced_motion() || dur <= 0.0 {
                                srm.offset_px = to;
                                srm.settle = None;
                            } else {
                                let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                                let ease = 1.0 - (1.0 - p) * (1.0 - p);
                                srm.offset_px = from + (to - from) * ease;
                                if p >= 1.0 {
                                    srm.offset_px = to;
                                    srm.settle = None;
                                } else {
                                    f.continuous(b);
                                }
                            }
                        }
                        if is_refreshing != srm.last_refreshing {
                            srm.last_refreshing = is_refreshing;
                            let target = if is_refreshing { settled_off_px } else { start_off_px };
                            srm.origin = None;
                            srm.dragging = false;
                            srm.pull_px = 0.0;
                            if instant || (srm.offset_px - target).abs() <= 0.5 {
                                srm.offset_px = target;
                                srm.settle = None;
                            } else {
                                srm.settle = Some((now, srm.offset_px, target, 0.22));
                                f.continuous(b);
                            }
                        } else if is_refreshing && srm.settle.is_none() && !srm.dragging {
                            srm.offset_px = settled_off_px;
                        } else if !is_refreshing && srm.settle.is_none() && !srm.dragging {
                            srm.offset_px = start_off_px;
                        }
                        srm.last_time = now;
                    }
                    let mut nested_scrolls = Vec::new();
                    collect_descendant_scrolls(self, state, id, &mut nested_scrolls);
                    let at_top = nested_scrolls.iter().all(|(_, _, off)| *off <= 0.5);
                    if enabled && !is_refreshing && at_top && f.input.pressed && f.pointer_in(b) {
                        srm.origin = f.input.pointer;
                        srm.dragging = false;
                        srm.pull_px = 0.0;
                        srm.settle = None;
                    }
                    if let (Some(origin), Some(pointer)) = (srm.origin, f.input.pointer)
                        && !f.input.cancelled
                        && !f.input.blurred
                    {
                        let dx = pointer[0] - origin[0];
                        let dy = pointer[1] - origin[1];
                        let slop = state.dp(8.0);
                        if !srm.dragging {
                            if !at_top || dy < -slop || (dx.abs() > slop && dx.abs() > dy.abs()) {
                                srm.origin = None;
                            } else if dy > slop && dy.abs() >= dx.abs() {
                                srm.dragging = true;
                                f.ui.active = None;
                                for &(sid, ref skey, off) in &nested_scrolls {
                                    f.cancel_scroll(sid);
                                    if off < 0.0 {
                                        state.offsets.insert(skey.clone(), 0.0);
                                    }
                                }
                            }
                        }
                        if srm.dragging {
                            let pull_px = (dy * 0.45).max(0.0);
                            srm.pull_px = pull_px;
                            let t_pull = (pull_px / trigger_px.max(1.0)).clamp(0.0, 1.0);
                            let overshoot = ((pull_px - trigger_px).max(0.0) * 0.25).min(state.dp(32.0));
                            srm.offset_px = start_off_px + (end_off_px - start_off_px) * t_pull + overshoot;
                            if !f.input.released {
                                f.continuous(b);
                            }
                        }
                    }
                    srl_dragging = srm.dragging;
                    if srm.origin.is_some()
                        && (f.input.released || f.input.cancelled || f.input.blurred)
                    {
                        if srm.dragging && !f.input.cancelled && !f.input.blurred {
                            if srm.pull_px >= trigger_px {
                                is_refreshing = true;
                                state.refreshing.insert(view_key.clone(), true);
                                srm.last_refreshing = true;
                                action = Some(self.id);
                                if f.reduced_motion() || (srm.offset_px - settled_off_px).abs() <= 0.5 {
                                    srm.offset_px = settled_off_px;
                                    srm.settle = None;
                                } else {
                                    srm.settle = Some((now, srm.offset_px, settled_off_px, 0.22));
                                    f.continuous(b);
                                }
                            } else if f.reduced_motion() || (srm.offset_px - start_off_px).abs() <= 0.5 {
                                srm.offset_px = start_off_px;
                                srm.settle = None;
                            } else {
                                srm.settle = Some((now, srm.offset_px, start_off_px, 0.22));
                                f.continuous(b);
                            }
                        } else if srm.dragging {
                            if f.reduced_motion() || (srm.offset_px - start_off_px).abs() <= 0.5 {
                                srm.offset_px = start_off_px;
                                srm.settle = None;
                            } else {
                                srm.settle = Some((now, srm.offset_px, start_off_px, 0.22));
                                f.continuous(b);
                            }
                        }
                        srm.origin = None;
                        srm.dragging = false;
                        srm.pull_px = 0.0;
                    }
                    srl_state = Some((srm, start_off_px, end_off_px, settled_off_px, is_refreshing));
                }
                if matches!(self.kind, Kind::Frame) {
                    let now = f.time();
                    for (ci, c) in self.children.iter().enumerate() {
                        let is_bs = c.bottom_sheet_spec().is_some();
                        let is_modal_ss = c.side_sheet_spec().is_some_and(|ss| !ss.coplanar);
                        if !is_bs && !is_modal_ss {
                            continue;
                        }
                        let cid = child_view_id(id, ci, c);
                        let sk = if c.id.is_empty() { format!("#{cid}") } else { c.id.to_owned() };
                        let cur_state = state.sheet_state.get(&sk).copied().unwrap_or(0);
                        if let Some(m) = state.sheet_motion.get_mut(&sk) {
                            if m.last_state != cur_state {
                                m.origin = None;
                                m.dragging = false;
                                m.settle = None;
                                m.current_h = None;
                                m.drag_offset = 0.0;
                                m.samples.clear();
                                m.last_state = cur_state;
                            } else if let Some((start, from, to, dur)) = m.settle {
                                if f.reduced_motion() || dur <= 0.0 {
                                    m.settle = None;
                                    if is_bs {
                                        m.current_h = None;
                                    } else {
                                        m.drag_offset = to;
                                    }
                                } else {
                                    let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                                    let ease = 1.0 - (1.0 - p) * (1.0 - p);
                                    let val = from + (to - from) * ease;
                                    if p >= 1.0 {
                                        m.settle = None;
                                        if is_bs {
                                            m.current_h = None;
                                        } else {
                                            m.drag_offset = to;
                                        }
                                    } else {
                                        if is_bs {
                                            m.current_h = Some(val);
                                        } else {
                                            m.drag_offset = val;
                                        }
                                        f.continuous(inner);
                                    }
                                }
                            }
                        } else if cur_state != 0 {
                            state.sheet_motion.insert(
                                sk,
                                XmlSheetMotion {
                                    last_state: cur_state,
                                    ..XmlSheetMotion::default()
                                },
                            );
                        }
                    }
                }
                let has_coordinator = matches!(self.kind, Kind::Frame)
                    && self.children.iter().any(|c| {
                        c.bottom_sheet_spec().is_some()
                            || c.side_sheet_spec().is_some()
                            || c.bottom_app_bar_spec().is_some_and(|s| s.hide_on_scroll || s.has_ext)
                            || (self.children.len() > 1 && c.is_snackbar())
                            || c.relative_rules.iter().any(|r| {
                                matches!(r.kind, RULE_COORDINATOR_SCROLL_BEHAVIOR | RULE_COORDINATOR_ANCHOR)
                            })
                    });
                let mut coordinator_rects = if has_coordinator {
                    Some(self.compute_coordinator_rects(inner, state, metrics, id))
                } else {
                    None
                };
                if has_coordinator {
                    for (ci, cn) in self.children.iter().enumerate() {
                        if cn.effective_visibility(state) == Visibility::Gone
                            || !cn.bottom_app_bar_spec().is_some_and(|s| s.hide_on_scroll)
                        {
                            continue;
                        }
                        let cid = child_view_id(id, ci, cn);
                        let bab_key = if cn.id.is_empty() { format!("#{cid}") } else { cn.id.to_owned() };
                        let scroll_px = cn.app_bar_scroll_px(state, cid);
                        let (hs, anim) = eval_bottom_app_bar_hide(&bab_key, scroll_px, state);
                        if state.bottom_app_bar_hide_motion.len() >= 64
                            && !state.bottom_app_bar_hide_motion.contains_key(&bab_key)
                        {
                            state.bottom_app_bar_hide_motion.clear();
                        }
                        state.bottom_app_bar_hide_motion.insert(bab_key, hs);
                        if anim {
                            f.continuous(b);
                        }
                    }
                }
                let modal_side_sheets: Vec<(usize, &'static Node, Rect)> = if matches!(self.kind, Kind::Frame) {
                    self.children
                        .iter()
                        .enumerate()
                        .filter_map(|(i, n)| {
                            if n.effective_visibility(state) == Visibility::Gone {
                                return None;
                            }
                            let ss = n.side_sheet_spec()?;
                            if ss.coplanar {
                                return None;
                            }
                            let cid = child_view_id(id, i, n);
                            if n.is_sheet_hidden(state, cid) {
                                return None;
                            }
                            let r = coordinator_rects.as_ref().and_then(|rects| rects[i])?;
                            Some((i, n, r))
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let has_modal_side_sheet = !modal_side_sheets.is_empty();
                let mut bs_dragging_indices: Vec<usize> = Vec::new();
                let mut bs_geometry: Vec<(usize, String, BottomSheetSpec, f32, f32, f32)> = Vec::new();
                if has_coordinator && !drawer_blocking && !has_modal_side_sheet {
                    let now = f.time();
                    let mut bs_height_changed = false;
                    for (i, n) in self.children.iter().enumerate() {
                        if n.effective_visibility(state) == Visibility::Gone {
                            continue;
                        }
                        let Some(bs) = n.bottom_sheet_spec() else { continue };
                        let cid = child_view_id(id, i, n);
                        if n.is_sheet_hidden(state, cid) {
                            continue;
                        }
                        let Some(sheet_rect) = coordinator_rects.as_ref().and_then(|rs| rs[i]) else {
                            continue;
                        };
                        let sk = if n.id.is_empty() { format!("#{cid}") } else { n.id.to_owned() };
                        let margin = n.resolved_margin(state);
                        let max_h = (inner.height - margin[1] - margin[3]).max(0.0);
                        let natural_h = match n.height {
                            Size::Dp(dh) => state.dp(dh).min(max_h),
                            Size::Wrap => n.layout_intrinsic(false, state, sheet_rect.width, metrics, cid).min(max_h),
                            Size::Match => max_h,
                        };
                        let expanded_h = (if bs.fit_to_contents { natural_h } else { max_h }).clamp(0.0, max_h);
                        let peek_h = (if bs.peek_h_dp > 0.0 { state.dp(bs.peek_h_dp) } else { natural_h }).clamp(0.0, max_h);
                        let half_h = (inner.height * bs.half_ratio).clamp(0.0, max_h);
                        let min_h = if bs.hideable { 0.0 } else { peek_h.min(expanded_h) };
                        let max_drag_h = expanded_h.max(peek_h);
                        let prev_h = sheet_rect.height;
                        let cur_sheet_state = state.sheet_state.get(&sk).copied().unwrap_or(0);
                        let is_sheet_expanded = expanded_h > 0.0
                            && (prev_h >= expanded_h - 1.0)
                            && (cur_sheet_state == SHEET_STATE_EXPANDED
                                || (cur_sheet_state == 0 && (bs.skip_collapsed || bs.peek_h_dp <= 0.0)));
                        let mut nested_scrolls = Vec::new();
                        collect_descendant_scrolls(n, state, cid, &mut nested_scrolls);
                        let has_nested_scroll = !nested_scrolls.is_empty();
                        let nested_scrolled = nested_scrolls.iter().any(|(_, _, off)| *off > 0.5);
                        let mut motion = state.sheet_motion.remove(&sk).unwrap_or_default();
                        if enabled && f.input.pressed && f.pointer_in(sheet_rect) {
                            motion.origin = f.input.pointer;
                            motion.drag_base = prev_h;
                            motion.dragging = false;
                            motion.settle = None;
                            motion.samples.clear();
                            if let Some(p) = f.input.pointer {
                                motion.samples.push((now, p[1]));
                            }
                        }
                        if let (Some(origin), Some(pointer)) = (motion.origin, f.input.pointer)
                            && !f.input.cancelled
                            && !f.input.blurred
                        {
                            if motion.samples.last().map_or(true, |&(_, y)| (y - pointer[1]).abs() > 0.01) {
                                motion.samples.push((now, pointer[1]));
                                if motion.samples.len() > 8 {
                                    motion.samples.remove(0);
                                }
                            }
                            let dx = pointer[0] - origin[0];
                            let dy = pointer[1] - origin[1];
                            let slop = state.dp(8.0);
                            if !motion.dragging {
                                if is_sheet_expanded && has_nested_scroll {
                                    if nested_scrolled || dy < 0.0 {
                                        motion.origin = Some(pointer);
                                        motion.drag_base = prev_h;
                                        motion.samples.clear();
                                        motion.samples.push((now, pointer[1]));
                                    } else if dy > slop && dy.abs() >= dx.abs() {
                                        motion.dragging = true;
                                        f.ui.active = None;
                                        for &(sid, ref skey, off) in &nested_scrolls {
                                            f.cancel_scroll(sid);
                                            if off < 0.0 {
                                                state.offsets.insert(skey.clone(), 0.0);
                                            }
                                        }
                                    }
                                } else if dy.abs() > slop && dy.abs() >= dx.abs() {
                                    motion.dragging = true;
                                    f.ui.active = None;
                                    for &(sid, ref skey, off) in &nested_scrolls {
                                        f.cancel_scroll(sid);
                                        if off < 0.0 {
                                            state.offsets.insert(skey.clone(), 0.0);
                                        }
                                    }
                                }
                            }
                            if motion.dragging {
                                motion.drag_offset = dy;
                                let clamped_h = (motion.drag_base - dy).clamp(min_h, max_drag_h);
                                motion.current_h = Some(clamped_h);
                                if (clamped_h - prev_h).abs() > 0.01 {
                                    bs_height_changed = true;
                                }
                                if !f.input.released {
                                    f.continuous(inner);
                                }
                            }
                        }
                        if motion.dragging {
                            bs_dragging_indices.push(i);
                        }
                        if motion.origin.is_some()
                            && (f.input.released || f.input.cancelled || f.input.blurred)
                        {
                            if motion.dragging && !f.input.cancelled && !f.input.blurred {
                                let drag_h = motion.current_h.unwrap_or(motion.drag_base);
                                let vy = pointer_velocity(&motion.samples, now);
                                let flick_v = state.dp(500.0);
                                let should_hide = bs.hideable
                                    && (drag_h <= peek_h * 0.5
                                        || (vy > flick_v && drag_h <= peek_h)
                                        || (bs.skip_collapsed
                                            && (drag_h <= expanded_h * 0.5 || vy > flick_v)));
                                let (target_state, target_h) = if should_hide {
                                    (SHEET_STATE_HIDDEN, 0.0)
                                } else {
                                    let mut detents: Vec<(u8, f32)> = Vec::with_capacity(3);
                                    detents.push((SHEET_STATE_EXPANDED, expanded_h));
                                    if !bs.fit_to_contents
                                        && half_h > peek_h + 1.0
                                        && half_h < expanded_h - 1.0
                                    {
                                        detents.push((SHEET_STATE_HALF_EXPANDED, half_h));
                                    }
                                    if !bs.skip_collapsed {
                                        detents.push((SHEET_STATE_COLLAPSED, peek_h));
                                    }
                                    detents
                                        .into_iter()
                                        .min_by(|a, b| {
                                            (a.1 - drag_h)
                                                .abs()
                                                .partial_cmp(&(b.1 - drag_h).abs())
                                                .unwrap_or(std::cmp::Ordering::Equal)
                                        })
                                        .unwrap_or((SHEET_STATE_EXPANDED, expanded_h))
                                };
                                state.sheet_state.insert(sk.clone(), target_state);
                                motion.last_state = target_state;
                                if !f.reduced_motion() && (drag_h - target_h).abs() > 0.5 {
                                    motion.settle = Some((now, drag_h, target_h, 0.25));
                                    motion.current_h = Some(drag_h);
                                    f.continuous(inner);
                                } else {
                                    motion.settle = None;
                                    motion.current_h = None;
                                    bs_height_changed = true;
                                }
                            } else if motion.dragging {
                                let drag_h = motion.current_h.unwrap_or(motion.drag_base);
                                if !f.reduced_motion() && (drag_h - motion.drag_base).abs() > 0.5 {
                                    motion.settle = Some((now, drag_h, motion.drag_base, 0.25));
                                    motion.current_h = Some(drag_h);
                                    f.continuous(inner);
                                } else {
                                    motion.settle = None;
                                    motion.current_h = None;
                                    bs_height_changed = true;
                                }
                            }
                            motion.origin = None;
                            motion.dragging = false;
                            motion.samples.clear();
                        }
                        let cur_h_after = motion.current_h.unwrap_or_else(|| {
                            n.resolve_bottom_sheet_height(bs, inner.height, max_h, natural_h, state, cid)
                        });
                        bs_geometry.push((i, sk.clone(), bs, cur_h_after, peek_h, expanded_h));
                        state.sheet_motion.insert(sk, motion);
                    }
                    if bs_height_changed {
                        coordinator_rects = Some(self.compute_coordinator_rects(inner, state, metrics, id));
                    }
                }
                let is_split_btn = self.is_split_button();
                let split_button_indices: Vec<usize> = if is_split_btn {
                    self.children
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| matches!(n.kind, Kind::Button) && n.effective_visibility(state) != Visibility::Gone)
                        .map(|(i, _)| i)
                        .collect()
                } else {
                    Vec::new()
                };
                let frame_child_rects: Vec<Option<Rect>> = if matches!(self.kind, Kind::Frame)
                    && selected_child.is_none()
                    && order.len() > 1
                {
                    let mut rects = vec![None; self.children.len()];
                    for &ci in &order {
                        let cn = &self.children[ci];
                        if cn.effective_visibility(state) == Visibility::Gone
                            || (is_drawer && cn.is_drawer_panel_child())
                        {
                            continue;
                        }
                        let cid = child_view_id(id, ci, cn);
                        if cn.is_sheet_hidden(state, cid)
                            || cn.is_search_view_closed(state, cid)
                            || (has_modal_side_sheet
                                && cn.side_sheet_spec().is_some_and(|ss| !ss.coplanar))
                        {
                            continue;
                        }
                        let cm = cn.resolved_margin(state);
                        let mut cr = Rect {
                            x: inner.x + cm[0],
                            y: inner.y + cm[1],
                            width: (inner.width - cm[0] - cm[2]).max(0.),
                            height: (inner.height - cm[1] - cm[3]).max(0.),
                        };
                        if let Size::Dp(w) = cn.width {
                            cr.width = state.dp(w);
                        }
                        if let Some(ch) = cn.collapsing_toolbar_height(state, cid) {
                            cr.height = ch;
                        } else if let Size::Dp(h) = cn.height {
                            cr.height = state.dp(h);
                        }
                        if matches!(cn.kind, Kind::VideoView) {
                            let name = if cn.id.is_empty() {
                                format!("#{cid}")
                            } else {
                                cn.id.to_owned()
                            };
                            let size = state
                                .video_dimensions
                                .get(&name)
                                .copied()
                                .or_else(|| {
                                    state
                                        .video_frames
                                        .get(&name)
                                        .map(|frame| (frame.width(), frame.height()))
                                })
                                .unwrap_or((0, 0));
                            let spec = |dimension: Size, available: f32| {
                                if matches!(dimension, Size::Wrap) {
                                    VideoMeasureSpec::AtMost(available.max(0.0) as u32)
                                } else {
                                    VideoMeasureSpec::Exactly(available.max(0.0) as u32)
                                }
                            };
                            let (w, h) = measure_video_view(
                                size,
                                spec(cn.width, cr.width),
                                spec(cn.height, cr.height),
                            );
                            cr.width = w as f32;
                            cr.height = h as f32;
                        }
                        if matches!(cn.width, Size::Wrap) && !matches!(cn.kind, Kind::VideoView) {
                            cr.width = cn
                                .layout_intrinsic(true, state, cr.width, metrics, cid)
                                .min(cr.width);
                        }
                        if matches!(cn.height, Size::Wrap) && !matches!(cn.kind, Kind::VideoView) {
                            cr.height = cn
                                .layout_intrinsic(false, state, cr.width, metrics, cid)
                                .min(cr.height);
                        }
                        let right = match cn.layout_gravity[0] {
                            0 => state.rtl,
                            2 => !state.rtl,
                            4 => true,
                            _ => false,
                        };
                        cr.x = if cn.layout_gravity[0] == 1 {
                            inner.x + (inner.width - cr.width) * 0.5 + cm[0] - cm[2]
                        } else if right {
                            inner.x + inner.width - cr.width - cm[2]
                        } else {
                            inner.x + cm[0]
                        };
                        cr.y = match cn.layout_gravity[1] {
                            1 => inner.y + (inner.height - cr.height) * 0.5 + cm[1] - cm[3],
                            2 => inner.y + inner.height - cr.height - cm[3],
                            _ => inner.y + cm[1],
                        };
                        if let Some(coord_r) = coordinator_rects.as_ref().and_then(|rs| rs[ci]) {
                            cr = coord_r;
                        }
                        rects[ci] = Some(cr);
                    }
                    rects
                } else {
                    Vec::new()
                };
                let mut visible_index = 0usize;
                for (order_pos, &i) in order.iter().enumerate() {
                    let n = &self.children[i];
                    if n.effective_visibility(state) == Visibility::Gone {
                        continue;
                    }
                    if is_drawer && n.is_drawer_panel_child() {
                        continue;
                    }
                    if matches!(self.kind, Kind::Frame) {
                        let cid = child_view_id(id, i, n);
                        if n.is_sheet_hidden(state, cid) {
                            continue;
                        }
                        if has_modal_side_sheet && n.side_sheet_spec().is_some_and(|ss| !ss.coplanar) {
                            continue;
                        }
                    }
                    if visible_index > 0 && extra_gap > 0.0 && matches!(self.kind, Kind::Linear) {
                        if horizontal && state.rtl { cursor -= extra_gap; } else { cursor += extra_gap; }
                    }
                    let before = matches!(self.kind,Kind::Linear|Kind::RadioGroup|Kind::TableRow) && self.divider_color.is_some() && if visible_index==0 { self.divider_flags & 1 != 0 } else { self.divider_flags & 2 != 0 };
                    if before {
                        let thickness=divider_thickness;
                        let rect=if horizontal { Rect { x:if state.rtl { cursor-thickness } else { cursor }, y:inner.y+divider_padding, width:thickness, height:(inner.height-2.0*divider_padding).max(0.0) } } else { Rect { x:inner.x+divider_padding, y:cursor, width:(inner.width-2.0*divider_padding).max(0.0), height:thickness } };
                        if let Some(color)=self.divider_color { f.paint_rect(rect,0.0,color); }
                        if horizontal && state.rtl { cursor-=thickness; } else { cursor+=thickness; }
                    }
                    visible_index+=1;
                    let margin = if matches!(self.kind,Kind::Absolute) { [0.0;4] } else { n.resolved_margin(state) };
                    let mut r = Rect {
                        x: inner.x + margin[0],
                        y: inner.y + margin[1],
                        width: (inner.width - margin[0] - margin[2]).max(0.),
                        height: (inner.height - margin[1] - margin[3]).max(0.),
                    };
                    if matches!(self.kind, Kind::Linear | Kind::RadioGroup | Kind::TableRow) {
                        let len = lengths[i];
                        if horizontal {
                            r.x = if state.rtl { cursor - margin[2] - len } else { cursor + margin[0] };
                            r.width = len;
                            let extent = len + n.margin_for_axis(horizontal, state);
                            if state.rtl { cursor -= extent; } else { cursor += extent; }
                        } else {
                            let (eff_top, eff_bot) = appbar_scroll_updates
                                .iter()
                                .find(|u| u.ci == i)
                                .map_or((margin[1], margin[3]), |u| (u.eff_top, u.eff_bot));
                            r.y = cursor + eff_top;
                            r.height = len;
                            cursor += len + eff_top + eff_bot;
                        }
                    }
                    if let Size::Dp(w) = n.width
                        && (!horizontal || matches!(self.kind, Kind::Frame | Kind::MediaController | Kind::MaterialCard | Kind::Absolute | Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper))
                    {
                        let w = state.dp(w);
                        r.width = if matches!(self.kind, Kind::Frame | Kind::MediaController | Kind::MaterialCard | Kind::Absolute | Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper) { w } else { w.min(r.width) };
                    }
                    if let Some(ch) = n.collapsing_toolbar_height(state, child_view_id(id, i, n)) {
                        r.height = ch;
                    } else if let Size::Dp(h) = n.height
                        && (horizontal || matches!(self.kind, Kind::Frame | Kind::MediaController | Kind::MaterialCard | Kind::Absolute | Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper))
                    {
                        let h = state.dp(h);
                        r.height = if matches!(self.kind, Kind::Frame | Kind::MediaController | Kind::MaterialCard | Kind::Absolute | Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper) { h } else { h.min(r.height) };
                    }
                    if matches!(n.kind,Kind::VideoView) {
                        let name=if n.id.is_empty(){format!("#{}",child_view_id(id,i,n))}else{n.id.to_owned()};let size=state.video_dimensions.get(&name).copied().or_else(||state.video_frames.get(&name).map(|frame|(frame.width(),frame.height()))).unwrap_or((0,0));
                        let spec=|dimension:Size,available:f32|if matches!(dimension,Size::Wrap){VideoMeasureSpec::AtMost(available.max(0.0) as u32)}else{VideoMeasureSpec::Exactly(available.max(0.0) as u32)};
                        let (width,height)=measure_video_view(size,spec(n.width,r.width),spec(n.height,r.height));r.width=width as f32;r.height=height as f32;
                    }
                    if matches!(self.kind, Kind::Linear | Kind::RadioGroup | Kind::TableRow) {
                        let mut gravity = if n.layout_gravity_explicit { n.layout_gravity } else { self.content_gravity };
                        if horizontal && (is_snackbar || is_floating_toolbar) && !n.layout_gravity_explicit && gravity[1] == 0 {
                            gravity[1] = 1;
                        }
                        if horizontal {
                            if matches!(n.height, Size::Wrap) && !matches!(n.kind,Kind::VideoView) { r.height = n.layout_intrinsic_for_width(false, state, r.width, n.weight > 0.0, metrics,child_view_id(id,i,n)).min(r.height); }
                            r.y = match gravity[1] {
                                1 => inner.y + (inner.height - r.height) * 0.5 + margin[1] - margin[3],
                                2 => inner.y + inner.height - r.height - margin[3],
                                _ => inner.y + margin[1],
                            };
                            if let Some((_,ascents,descents))=row_baselines && !matches!(n.height,Size::Match) && let Some(base)=n.relative_baseline(r.width,r.height,state,metrics,child_view_id(id,i,n)) {
                                match gravity[1] {0 if ascents[0]>=0.0=>r.y+=ascents[0]-base,2 if descents[2]>=0.0=>r.y-=descents[2]-(r.height-base),_=>{}}
                            }
                        } else {
                            if matches!(n.width, Size::Wrap) && !matches!(n.kind,Kind::VideoView) { r.width = n.layout_intrinsic(true, state,r.width,metrics,child_view_id(id,i,n)).min(r.width); }
                            let right = match gravity[0] { 0 => state.rtl, 2 => !state.rtl, 4 => true, _ => false };
                            r.x = if gravity[0] == 1 { inner.x + (inner.width - r.width) * 0.5 + margin[0] - margin[2] }
                                else if right { inner.x + inner.width - r.width - margin[2] } else { inner.x + margin[0] };
                        }
                    }
                    if matches!(self.kind, Kind::Frame | Kind::MediaController | Kind::MaterialCard | Kind::Absolute | Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper) {
                        if matches!(n.width, Size::Wrap) && !matches!(n.kind,Kind::VideoView) { r.width = n.layout_intrinsic(true, state,r.width,metrics,child_view_id(id,i,n)).min(r.width); }
                        if matches!(n.height, Size::Wrap) && !matches!(n.kind,Kind::VideoView) { r.height = n.layout_intrinsic(false, state, r.width, metrics,child_view_id(id,i,n)).min(r.height); }
                        let right = match n.layout_gravity[0] {
                            0 => state.rtl,
                            2 => !state.rtl,
                            4 => true,
                            _ => false,
                        };
                        r.x = if n.layout_gravity[0] == 1 {
                            inner.x + (inner.width - r.width) * 0.5 + margin[0] - margin[2]
                        } else if right {
                            inner.x + inner.width - r.width - margin[2]
                        } else {
                            inner.x + margin[0]
                        };
                        r.y = match n.layout_gravity[1] {
                            1 => inner.y + (inner.height - r.height) * 0.5 + margin[1] - margin[3],
                            2 => inner.y + inner.height - r.height - margin[3],
                            _ => inner.y + margin[1],
                        };
                    }
                    if matches!(self.kind, Kind::Absolute) {
                        r.x = inner.x + state.dp(n.absolute_position[0]);
                        r.y = inner.y + state.dp(n.absolute_position[1]);
                    }
                    if let Some(coord_r) = coordinator_rects.as_ref().and_then(|rects| rects[i]) {
                        r = coord_r;
                    }
                    let key = if n.id.is_empty() {
                        anonymous_child_id(id, i)
                    } else if let Some(namespace)=state.adapter_namespace {
                        anonymous_child_id(namespace,view_id(n.id) as usize)
                    } else {
                        view_id(n.id)
                    };
                    if radio_group && matches!(n.kind, Kind::RadioButton) {
                        let name = if n.id.is_empty() { format!("#{key}") } else { n.id.to_string() };
                        state.checked.insert(name, state.radio_groups.get(&id).copied().flatten() == Some(key));
                    }
                    let outgoing = transition.is_some_and(|t| t.active && i == t.previous && i != t.current);
                    let animation = if outgoing { self.animation_out } else { self.animation_in };
                    let alpha = transition.filter(|t| t.active).and_then(|t| animation.map(|a| a.sample((f.time()-t.started).max(0.0),false))).unwrap_or(1.0);
                    let translate = if outgoing { self.translate_out } else { self.translate_in };
                    let delta = transition.filter(|t| t.active).and_then(|t| translate.map(|a| a.sample((f.time()-t.started).max(0.0),r,inner,false))).unwrap_or([0.0,0.0]);
                    r.x += delta[0]; r.y += delta[1];
                    let active_tr_elapsed = transition.filter(|t| t.active).map(|t| (f.time() - t.started).max(0.0));
                    let child_scale_anim = if active_tr_elapsed.is_some() { if outgoing { scale_out } else { scale_in } } else { None };
                    let child_rotate_anim = if active_tr_elapsed.is_some() { if outgoing { rotate_out } else { rotate_in } } else { None };
                    let child_elev_dp = if let Some(elapsed) = active_tr_elapsed {
                        if outgoing {
                            elev_out.map(|a| a.sample(elapsed))
                        } else {
                            elev_in.map(|a| a.sample(elapsed))
                        }
                    } else if !outgoing && transition.is_some_and(|t| t.previous != t.current) {
                        elev_in.map(|a| a.to_dp.max(0.0))
                    } else {
                        None
                    };
                    let mut child_action = None;
                    let child_enabled=state.media_active_capabilities.map_or(true,|c|match n.id {"pause"=>c.can_pause,"rew"=>c.can_seek_backward,"ffwd"=>c.can_seek_forward,"mediacontroller_progress"=>c.can_seek_backward||c.can_seek_forward,"prev"=>c.has_previous,"next"=>c.has_next,_=>true});
                    let prev_toggle_seg = if let Some(pos) = toggle_buttons.iter().position(|(idx, _, _)| *idx == i) {
                        let is_first = pos == 0;
                        let is_last = pos + 1 == toggle_buttons.len();
                        let is_phys_left = if state.rtl { is_last } else { is_first };
                        let is_phys_right = if state.rtl { is_first } else { is_last };
                        state.toggle_group_segment.replace(ToggleGroupSegment {
                            is_phys_left,
                            is_phys_right,
                        })
                    } else {
                        state.toggle_group_segment.take()
                    };
                    let prev_split_role = if let Some(pos) = split_button_indices.iter().position(|idx| *idx == i) {
                        let is_leading = pos == 0;
                        let is_phys_left = if state.rtl { !is_leading } else { is_leading };
                        state.split_button_role.replace(SplitButtonRole {
                            is_phys_left,
                            is_trailing_chevron: !is_leading,
                        })
                    } else {
                        state.split_button_role.take()
                    };
                    let pointer_occluded = !frame_child_rects.is_empty()
                        && !f.any_scroll_dragging()
                        && f.input.pointer.is_some_and(|ptr| {
                            order[order_pos + 1..].iter().any(|&later_i| {
                                let later = &self.children[later_i];
                                let later_id = child_view_id(id, later_i, later);
                                later.blocks_pointer_for_lower_siblings_in(state, later_id)
                                    && frame_child_rects[later_i]
                                        .is_some_and(|lr| lr.intersect(f.clip).contains(ptr))
                            })
                        });
                    let bs_info = bs_geometry.iter().find(|(idx, _, _, _, _, _)| *idx == i).cloned();
                    let bs_dragged = bs_dragging_indices.contains(&i);
                    let prev_active_bs = bs_info.as_ref().map(|(_, sk, _, _, _, _)| {
                        state.drag_handle_toggled = false;
                        state.active_bottom_sheet.replace(sk.clone())
                    });
                    let ab_scroll_entry = appbar_scroll_updates
                        .iter()
                        .find(|u| u.ci == i)
                        .map(|u| (u.natural_h, u.body_collapse, u.compress));
                    if let Some((_, body_collapse, _)) = ab_scroll_entry
                        && r.height <= 0.001
                        && body_collapse > 0.001
                    {
                        continue;
                    }
                    let (clip_slot_rect, draw_r) = if let Some((natural_h, body_collapse, compress)) = ab_scroll_entry
                        && body_collapse > 0.001
                    {
                        let shift_y = if compress {
                            let p = (body_collapse / natural_h.max(1.0)).clamp(0.0, 1.0);
                            let ease_out_quad = 1.0 - (1.0 - p) * (1.0 - p);
                            (natural_h * 0.3 * ease_out_quad).min(body_collapse)
                        } else {
                            body_collapse
                        };
                        (
                            Some(r),
                            Rect {
                                x: r.x,
                                y: r.y - shift_y,
                                width: r.width,
                                height: natural_h,
                            },
                        )
                    } else {
                        (None, r)
                    };
                    let mut draw = |f: &mut Frame<'_>| f.with_enabled(child_enabled,|f|f.with_opacity(alpha, |f| {
                        let pushed_tr = if let Some(elapsed) = active_tr_elapsed {
                            push_transition_transforms(f, child_scale_anim, child_rotate_anim, elapsed, draw_r, inner, state, false)
                        } else {
                            0
                        };
                        if let Some(elev_dp) = child_elev_dp {
                            if elev_dp > 0.0 {
                                let elev_r = if n.radius < 0.0 {
                                    draw_r.width.min(draw_r.height) * 0.5
                                } else if n.radius > 0.0 {
                                    state.dp(n.radius)
                                } else if let Some([tl, tr, br, bl]) = n.resolved_corner_radii(state) {
                                    tl.max(tr).max(br).max(bl)
                                } else {
                                    n.resolved_state_radius(state)
                                };
                                paint_elevation_dp(f, draw_r, elev_r, elev_dp, state.effective_density());
                            }
                            state.suppress_next_elevation = true;
                        }
                        let prev_bab_applied = state.bab_hide_applied_by_parent;
                        if has_coordinator && n.bottom_app_bar_spec().is_some_and(|s| s.hide_on_scroll) {
                            state.bab_hide_applied_by_parent = true;
                        }
                        if outgoing || drawer_blocking || has_modal_side_sheet || bs_dragged || srl_dragging { f.without_input(true, |f| { n.draw(f, draw_r, state, key); }); }
                        else if pointer_occluded { f.without_pointer_input(true, |f| { child_action = n.draw(f, draw_r, state, key); }); }
                        else { child_action = n.draw(f, draw_r, state, key); }
                        state.bab_hide_applied_by_parent = prev_bab_applied;
                        state.suppress_next_elevation = false;
                        for _ in 0..pushed_tr {
                            f.pop_transform();
                        }
                    }));
                    let padding_clip = !self.clip_to_padding_disabled && inner != b;
                    let mut draw_child = |f: &mut Frame<'_>| {
                        if let Some(slot_r) = clip_slot_rect {
                            f.within(slot_r, &mut draw);
                        } else if self.clip_children_disabled || child_elev_dp.is_some() {
                            draw(f);
                        } else {
                            f.within(r, &mut draw);
                        }
                    };
                    if padding_clip { f.within(inner, &mut draw_child); } else { draw_child(f); }
                    if let Some(prev) = prev_active_bs {
                        state.active_bottom_sheet = prev;
                    }
                    if let Some((_, sk, bs, cur_h_after, peek_h, expanded_h)) = bs_info
                        && state.drag_handle_toggled
                    {
                        state.drag_handle_toggled = false;
                        let cur_st = state.sheet_state.get(&sk).copied().unwrap_or(if bs.skip_collapsed {
                            SHEET_STATE_EXPANDED
                        } else {
                            SHEET_STATE_COLLAPSED
                        });
                        let next_st = if cur_st == SHEET_STATE_EXPANDED {
                            if !bs.skip_collapsed {
                                SHEET_STATE_COLLAPSED
                            } else if bs.hideable {
                                SHEET_STATE_HIDDEN
                            } else {
                                SHEET_STATE_COLLAPSED
                            }
                        } else {
                            SHEET_STATE_EXPANDED
                        };
                        let next_h = match next_st {
                            SHEET_STATE_HIDDEN => 0.0,
                            SHEET_STATE_EXPANDED => expanded_h,
                            _ => peek_h,
                        };
                        state.sheet_state.insert(sk.clone(), next_st);
                        let m = state.sheet_motion.entry(sk).or_default();
                        m.last_state = next_st;
                        if !f.reduced_motion() && (cur_h_after - next_h).abs() > 0.5 {
                            m.settle = Some((f.time(), cur_h_after, next_h, 0.25));
                            m.current_h = Some(cur_h_after);
                        } else {
                            m.settle = None;
                            m.current_h = None;
                        }
                        f.continuous(inner);
                    }
                    state.toggle_group_segment = prev_toggle_seg;
                    state.split_button_role = prev_split_role;
                    if let Some(a) = child_action {
                        if let Some(tg) = toggle_group
                            && matches!(n.kind, Kind::Button)
                            && !toggle_buttons.is_empty()
                        {
                            let clicked_key = if n.id.is_empty() {
                                format!("#{key}")
                            } else {
                                n.id.to_owned()
                            };
                            let was_checked = state.checked.get(&clicked_key).copied().unwrap_or(false);
                            if tg.single_selection {
                                if was_checked {
                                    if !tg.selection_required {
                                        state.checked.insert(clicked_key, false);
                                    }
                                } else {
                                    for &(_, _, ref sib_key) in &toggle_buttons {
                                        state.checked.insert(sib_key.clone(), *sib_key == clicked_key);
                                    }
                                }
                            } else if was_checked {
                                let checked_count = toggle_buttons
                                    .iter()
                                    .filter(|(_, _, k)| state.checked.get(k).copied().unwrap_or(false))
                                    .count();
                                if !tg.selection_required || checked_count > 1 {
                                    state.checked.insert(clicked_key, false);
                                }
                            } else {
                                state.checked.insert(clicked_key, true);
                            }
                            f.continuous(b);
                        }
                        if radio_group && matches!(n.kind, Kind::RadioButton) {
                            let changed=state.radio_groups.get(&id).copied().flatten()!=Some(key);
                            if let Some(child)=state.radio_groups.get(&id).copied().flatten().and_then(|old|radio_descendant_by_id(self,old)).filter(|child|matches!(child.kind,Kind::RadioButton)) {state.checked.insert(child.id.to_owned(),false);}
                            state.radio_groups.insert(id, Some(key));
                            if changed {let group=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};state.record_radio_group_event(&group,Some(key));}
                            for (j, sibling) in self.children.iter().enumerate() {
                                if matches!(sibling.kind, Kind::RadioButton) {
                                    let sibling_key = if sibling.id.is_empty() { anonymous_child_id(id, j) } else { view_id(sibling.id) };
                                    let name = if sibling.id.is_empty() { format!("#{sibling_key}") } else { sibling.id.to_string() };
                                    state.checked.insert(name, sibling_key == key);
                                }
                            }
                        }
                        action = Some(a)
                    }
                }
                for u in appbar_scroll_updates {
                    if let Some((child_key, next_st)) = u.commit {
                        state.app_bar_scroll_motion.insert(child_key, next_st);
                    }
                }
                state.collapsing_scroll_offset = prev_collapsing_scroll;
                if has_collapsing_child {
                    let collapsing_sib_after = self.sibling_scroll_offset(state, id);
                    if collapsing_sib_before != collapsing_sib_after {
                        f.continuous(b);
                    }
                }
                if is_drawer {
                    let now = f.time();
                    let shifted_rect = |open_rect: Rect, align_right: bool, prog: f32| -> Rect {
                        let slide = (1.0 - prog.clamp(0.0, 1.0)) * open_rect.width;
                        let x = if align_right { open_rect.x + slide } else { open_rect.x - slide };
                        Rect { x, ..open_rect }
                    };
                    let close_slot_keys = |slot: &DrawerSlot, state: &mut LayoutState| {
                        state.drawer_open.insert(slot.open_key.clone(), false);
                        if let Some((ref prim_k, ref end_k)) = slot.end_only_keys {
                            if state.drawer_open.get(prim_k) == Some(&true) {
                                state.drawer_open.insert(prim_k.clone(), false);
                            }
                            if state.drawer_open.get(end_k) == Some(&true) {
                                state.drawer_open.insert(end_k.clone(), false);
                            }
                        }
                    };
                    for slot in &mut drawer_slots {
                        if slot.edge_was_dragging || slot.dm.edge_opening {
                            slot.was_dragging = slot.edge_was_dragging;
                            continue;
                        }
                        if slot.lock_mode == DRAWER_LOCK_MODE_LOCKED_OPEN {
                            slot.dm.origin = None;
                            slot.dm.dragging = false;
                            slot.dm.samples.clear();
                            slot.was_dragging = false;
                            continue;
                        }
                        if !(slot.dm.progress > 0.001 || slot.dm.dragging || slot.open_now) {
                            continue;
                        }
                        if enabled && f.input.pressed && slot.dm.last_open && slot.dm.progress > 0.05 {
                            for &(_, _, open_r, align_right) in &slot.panels {
                                let vis_r = shifted_rect(open_r, align_right, slot.dm.progress);
                                if f.pointer_in(vis_r) {
                                    slot.dm.origin = f.input.pointer;
                                    slot.dm.drag_base_progress = slot.dm.progress;
                                    slot.dm.drag_align_right = align_right;
                                    slot.dm.drag_panel_w = open_r.width.max(1.0);
                                    slot.dm.dragging = false;
                                    slot.dm.edge_opening = false;
                                    slot.dm.anim = None;
                                    slot.dm.samples.clear();
                                    if let Some(p) = f.input.pointer {
                                        slot.dm.samples.push((now, p[0]));
                                    }
                                    break;
                                }
                            }
                        }
                        if let (Some(origin), Some(pointer)) = (slot.dm.origin, f.input.pointer)
                            && !f.input.cancelled
                            && !f.input.blurred
                        {
                            if slot.dm.samples.last().map_or(true, |&(_, x)| (x - pointer[0]).abs() > 0.01) {
                                slot.dm.samples.push((now, pointer[0]));
                                if slot.dm.samples.len() > 8 {
                                    slot.dm.samples.remove(0);
                                }
                            }
                            let dx = pointer[0] - origin[0];
                            let dy = pointer[1] - origin[1];
                            let close_dx = if slot.dm.drag_align_right { dx } else { -dx };
                            let slop = state.dp(8.0);
                            if !slot.dm.dragging && dx.abs() > slop && dx.abs() >= dy.abs() {
                                slot.dm.dragging = true;
                                f.ui.active = None;
                            }
                            if slot.dm.dragging {
                                let delta_p = close_dx / slot.dm.drag_panel_w.max(1.0);
                                slot.dm.progress = (slot.dm.drag_base_progress - delta_p).clamp(0.0, 1.0);
                                if !f.input.released {
                                    f.continuous(b);
                                }
                            }
                        }
                        slot.was_dragging = slot.dm.dragging;
                        if slot.dm.origin.is_some()
                            && (f.input.released || f.input.cancelled || f.input.blurred)
                        {
                            if slot.dm.dragging && !f.input.cancelled && !f.input.blurred {
                                let vx = pointer_velocity(&slot.dm.samples, now);
                                let v_close = if slot.dm.drag_align_right { vx } else { -vx };
                                let should_close =
                                    (1.0 - slot.dm.progress) > 0.30 || v_close > state.dp(500.0);
                                if should_close {
                                    close_slot_keys(slot, state);
                                    slot.dm.last_open = false;
                                    if slot.instant || slot.dm.progress <= 0.001 {
                                        slot.dm.progress = 0.0;
                                        slot.dm.anim = None;
                                    } else {
                                        slot.dm.anim = Some((now, slot.dm.progress, 0.0, 0.25));
                                        f.continuous(b);
                                    }
                                } else if slot.instant || (1.0 - slot.dm.progress).abs() <= 0.001 {
                                    slot.dm.progress = 1.0;
                                    slot.dm.anim = None;
                                } else {
                                    slot.dm.anim = Some((now, slot.dm.progress, 1.0, 0.25));
                                    f.continuous(b);
                                }
                            } else if slot.dm.dragging {
                                let target = if slot.dm.last_open { 1.0 } else { 0.0 };
                                if slot.instant {
                                    slot.dm.progress = target;
                                    slot.dm.anim = None;
                                } else {
                                    slot.dm.anim = Some((now, slot.dm.progress, target, 0.25));
                                    f.continuous(b);
                                }
                            }
                            slot.dm.origin = None;
                            slot.dm.dragging = false;
                            slot.dm.samples.clear();
                        }
                    }
                    let mut scrim_hit = b;
                    for slot in &drawer_slots {
                        if slot.dm.progress <= 0.001 {
                            continue;
                        }
                        for &(_, _, open_r, align_right) in &slot.panels {
                            let pr = shifted_rect(open_r, align_right, slot.dm.progress);
                            if align_right {
                                let right_edge = pr.x.min(scrim_hit.x + scrim_hit.width);
                                scrim_hit.width = (right_edge - scrim_hit.x).max(0.0);
                            } else {
                                let left_edge = (pr.x + pr.width).max(scrim_hit.x);
                                let right_edge = scrim_hit.x + scrim_hit.width;
                                scrim_hit.x = left_edge;
                                scrim_hit.width = (right_edge - left_edge).max(0.0);
                            }
                        }
                    }
                    let any_was_dragging = drawer_slots.iter().any(|s| s.was_dragging || s.edge_was_dragging);
                    let any_open_scrim = drawer_slots.iter().any(|s| {
                        s.dm.last_open
                            && s.dm.progress > 0.01
                            && s.lock_mode != DRAWER_LOCK_MODE_LOCKED_OPEN
                    });
                    if !any_was_dragging && any_open_scrim {
                        let scrim_resp = f.interact(anonymous_child_id(id, 9980), scrim_hit, enabled);
                        let dismissed = scrim_resp.clicked || (enabled && f.pressed_key(Key::Escape));
                        if dismissed {
                            for slot in &mut drawer_slots {
                                if slot.dm.last_open && slot.lock_mode != DRAWER_LOCK_MODE_LOCKED_OPEN {
                                    close_slot_keys(slot, state);
                                    slot.dm.last_open = false;
                                    if slot.instant {
                                        slot.dm.progress = 0.0;
                                        slot.dm.anim = None;
                                    } else {
                                        slot.dm.anim = Some((now, slot.dm.progress, 0.0, 0.25));
                                    }
                                }
                            }
                            f.continuous(b);
                        }
                    }
                    let max_progress = drawer_slots
                        .iter()
                        .map(|s| s.dm.progress)
                        .fold(0.0_f32, f32::max);
                    if max_progress > 0.001 {
                        let scrim_a = (82.0 * max_progress).round().clamp(0.0, 255.0) as u8;
                        if scrim_a > 0 {
                            f.paint_rect(b, 0.0, Color(0, 0, 0, scrim_a));
                        }
                        let corner_r = state.dp(16.0);
                        let prev_drawer_active = state.drawer_panel_active;
                        state.drawer_panel_active = true;
                        f.within(b, |f| {
                            for slot in &drawer_slots {
                                if slot.dm.progress <= 0.001 {
                                    continue;
                                }
                                let block_drawer_input = slot.was_dragging || slot.edge_was_dragging || !slot.dm.last_open;
                                for &(i, n, open_rect, align_right) in &slot.panels {
                                    let panel_rect = shifted_rect(open_rect, align_right, slot.dm.progress);
                                    let radii = if align_right {
                                        [corner_r, 0.0, 0.0, corner_r]
                                    } else {
                                        [0.0, corner_r, corner_r, 0.0]
                                    };
                                    paint_elevation_level(f, panel_rect, corner_r, 1, state.effective_density());
                                    if !n.has_explicit_custom_background() {
                                        paint_rounded_rect_radii(f, panel_rect, radii, t.surface_low);
                                    }
                                    let key = if n.id.is_empty() {
                                        anonymous_child_id(id, i)
                                    } else if let Some(namespace) = state.adapter_namespace {
                                        anonymous_child_id(namespace, view_id(n.id) as usize)
                                    } else {
                                        view_id(n.id)
                                    };
                                    let mut child_action = None;
                                    f.within(panel_rect, |f| {
                                        if block_drawer_input {
                                            f.without_input(true, |f| {
                                                n.draw(f, panel_rect, state, key);
                                            });
                                        } else {
                                            child_action = n.draw(f, panel_rect, state, key);
                                        }
                                    });
                                    if child_action.is_some() {
                                        action = child_action;
                                    }
                                }
                            }
                        });
                        state.drawer_panel_active = prev_drawer_active;
                    }
                    for slot in drawer_slots {
                        state.drawer_motion.insert(slot.motion_key, slot.dm);
                    }
                }
                if let Some((srm, start_off_px, end_off_px, _settled_off_px, is_refreshing)) = srl_state {
                    let is_large = srl_ext.as_ref().is_some_and(|e| e.large);
                    let diameter = state.dp(if is_large { 56.0 } else { 40.0 });
                    let circle = Rect {
                        x: b.x + (b.width - diameter) * 0.5,
                        y: b.y + srm.offset_px,
                        width: diameter,
                        height: diameter,
                    };
                    let show_spinner = (is_refreshing || srl_dragging || srm.settle.is_some())
                        && circle.y + circle.height > b.y + 0.01
                        && circle.y < b.y + b.height;
                    if show_spinner {
                        let radius = diameter * 0.5;
                        let fill_color = match srl_ext.as_ref() {
                            Some(ext) => ext.bg_color.unwrap_or(t.surface_high),
                            None => t.surface_low,
                        };
                        let scheme_colors = srl_ext.as_ref().map(|e| e.scheme_colors.as_slice()).unwrap_or(&[]);
                        let arc_color = if scheme_colors.is_empty() {
                            t.primary
                        } else if is_refreshing && scheme_colors.len() > 1 && !f.reduced_motion() {
                            let idx = ((f.time().max(0.0) / 1.332).floor() as usize) % scheme_colors.len();
                            scheme_colors[idx]
                        } else {
                            scheme_colors[0]
                        };
                        let (start_a, sweep) = if is_refreshing {
                            if f.continuous(circle) {
                                crate::progress::classic_circular_sample(f.time() + 0.4)
                            } else {
                                (-std::f32::consts::FRAC_PI_2, std::f32::consts::PI * 1.35)
                            }
                        } else {
                            let span = (end_off_px - start_off_px).max(1.0);
                            let pull_frac = ((srm.offset_px - start_off_px) / span).clamp(0.0, 1.0);
                            (
                                -std::f32::consts::FRAC_PI_2 + pull_frac * std::f32::consts::PI * 1.5,
                                (pull_frac * std::f32::consts::PI * 1.55).max(0.15),
                            )
                        };
                        let arc_inset = state.dp(if is_large { 14.0 } else { 10.0 });
                        let arc_rect = Rect {
                            x: circle.x + arc_inset,
                            y: circle.y + arc_inset,
                            width: (circle.width - arc_inset * 2.0).max(0.0),
                            height: (circle.height - arc_inset * 2.0).max(0.0),
                        };
                        let stroke = state.dp(if is_large { 3.0 } else { 2.5 });
                        f.within(b, |f| {
                            paint_elevation_level(f, circle, radius, 2, state.effective_density());
                            f.paint_rect(circle, radius, fill_color);
                            f.commands.push(DrawCommand::Arc {
                                bounds: arc_rect,
                                start: start_a,
                                sweep,
                                stroke,
                                color: arc_color,
                                rounded: true,
                            });
                        });
                    }
                    if state.swipe_refresh_motion.len() >= 64
                        && !state.swipe_refresh_motion.contains_key(&view_key)
                    {
                        state.swipe_refresh_motion.clear();
                    }
                    state.swipe_refresh_motion.insert(view_key.clone(), srm);
                }
                if matches!(self.kind,Kind::Linear|Kind::RadioGroup|Kind::TableRow) && visible_index>0 && self.divider_flags & 4 != 0 && let Some(color)=self.divider_color {
                    let rect=if horizontal { Rect { x:if state.rtl { cursor-divider_thickness } else { cursor }, y:inner.y+divider_padding, width:divider_thickness, height:(inner.height-2.0*divider_padding).max(0.0) } } else { Rect { x:inner.x+divider_padding, y:cursor, width:(inner.width-2.0*divider_padding).max(0.0), height:divider_thickness } };
                    f.paint_rect(rect,0.0,color);
                }
                if let (Some(key),Some(child_action))=(media_key.as_ref(),action) && let Some(snapshot)=state.media_players.get(key).copied() {
                    let event=match child_action {"pause"=>Some(if snapshot.playing{MediaControllerAction::Pause}else{MediaControllerAction::Play}),"rew"=>Some(MediaControllerAction::SeekTo(snapshot.position_ms.saturating_sub(5000))),"ffwd"=>Some(MediaControllerAction::SeekTo(snapshot.position_ms.saturating_add(15000).min(snapshot.duration_ms))),"prev"=>Some(MediaControllerAction::Previous),"next"=>Some(MediaControllerAction::Next),"mediacontroller_progress"=>Some(MediaControllerAction::SeekTo((state.progress.get("mediacontroller_progress").copied().unwrap_or(0.0).clamp(0.0,1000.0) as f64*snapshot.duration_ms as f64/1000.0).round() as u64)),_=>None};
                    if let Some(event)=event {state.media_controller_actions.insert(key.clone(),event);action=Some(self.id);}
                }
                if let Some(key)=media_key.as_ref() {let slider_id=anonymous_child_id(id,view_id("mediacontroller_progress") as usize);if f.ui.active==Some(slider_id)&&!f.input.released&&!f.input.cancelled&&!f.input.blurred {if let Some(value)=state.progress.get("mediacontroller_progress").copied(){state.media_seek_progress.insert(key.clone(),value);}}else{state.media_seek_progress.remove(key);}}
                if let Some((texts,progress,secondary,icon))=media_saved {for (key,value) in ["time_current","time","pause"].into_iter().zip(texts) {if let Some(value)=value{state.text.insert(key.into(),value);}else{state.text.remove(key);}}if let Some(value)=progress{state.progress.insert("mediacontroller_progress".into(),value);}else{state.progress.remove("mediacontroller_progress");}if let Some(value)=secondary{state.secondary_progress.insert("mediacontroller_progress".into(),value);}else{state.secondary_progress.remove("mediacontroller_progress");}if let Some(value)=icon{state.image_sources.insert("pause".into(),value);}else{state.image_sources.remove("pause");}}
                if has_modal_side_sheet {
                    let now = f.time();
                    let mut any_ss_dragging = false;
                    let mut active_sheets: Vec<(usize, &'static Node, Rect, Rect, bool, bool, bool, f32)> =
                        Vec::with_capacity(modal_side_sheets.len());
                    for &(i, n, base_sr) in &modal_side_sheets {
                        let cid = child_view_id(id, i, n);
                        let sk = if n.id.is_empty() { format!("#{cid}") } else { n.id.to_owned() };
                        let phys_right = n
                            .side_sheet_spec()
                            .is_some_and(|ss| if ss.edge_start { state.rtl } else { !state.rtl });
                        let mut m = state.sheet_motion.remove(&sk).unwrap_or_default();
                        let sw = base_sr.width.max(1.0);
                        let cur_vis = |off: f32| -> Rect {
                            let sdx = if phys_right { off } else { -off };
                            Rect { x: base_sr.x + sdx, ..base_sr }
                        };
                        let vis_before = cur_vis(m.drag_offset);
                        if enabled
                            && f.input.pressed
                            && state.sheet_state.get(&sk) != Some(&SHEET_STATE_HIDDEN)
                            && f.pointer_in(vis_before)
                        {
                            m.origin = f.input.pointer;
                            m.drag_base = m.drag_offset;
                            m.dragging = false;
                            m.settle = None;
                            m.samples.clear();
                            if let Some(p) = f.input.pointer {
                                m.samples.push((now, p[0]));
                            }
                        }
                        if let (Some(origin), Some(pointer)) = (m.origin, f.input.pointer)
                            && !f.input.cancelled
                            && !f.input.blurred
                        {
                            if m.samples.last().map_or(true, |&(_, x)| (x - pointer[0]).abs() > 0.01) {
                                m.samples.push((now, pointer[0]));
                                if m.samples.len() > 8 {
                                    m.samples.remove(0);
                                }
                            }
                            let dx = pointer[0] - origin[0];
                            let dy = pointer[1] - origin[1];
                            let out_dx = if phys_right { dx } else { -dx };
                            let slop = state.dp(8.0);
                            if !m.dragging && dx.abs() > slop && dx.abs() >= dy.abs() {
                                m.dragging = true;
                                f.ui.active = None;
                            }
                            if m.dragging {
                                m.drag_offset = (m.drag_base + out_dx).clamp(0.0, sw);
                                if !f.input.released {
                                    f.continuous(b);
                                }
                            }
                        }
                        let was_dragging = m.dragging;
                        if was_dragging {
                            any_ss_dragging = true;
                        }
                        if m.origin.is_some()
                            && (f.input.released || f.input.cancelled || f.input.blurred)
                        {
                            if m.dragging && !f.input.cancelled && !f.input.blurred {
                                let vx = pointer_velocity(&m.samples, now);
                                let v_out = if phys_right { vx } else { -vx };
                                let should_dismiss =
                                    m.drag_offset > sw * 0.30 || v_out > state.dp(500.0);
                                if should_dismiss {
                                    state.sheet_state.insert(sk.clone(), SHEET_STATE_HIDDEN);
                                    m.last_state = SHEET_STATE_HIDDEN;
                                    if !f.reduced_motion() && (sw - m.drag_offset) > 0.5 {
                                        m.settle = Some((now, m.drag_offset, sw, 0.25));
                                        f.continuous(b);
                                    } else {
                                        m.settle = None;
                                        m.drag_offset = sw;
                                    }
                                } else if !f.reduced_motion() && m.drag_offset > 0.5 {
                                    m.settle = Some((now, m.drag_offset, 0.0, 0.25));
                                    f.continuous(b);
                                } else {
                                    m.settle = None;
                                    m.drag_offset = 0.0;
                                }
                            } else if m.dragging {
                                if !f.reduced_motion() && m.drag_offset > 0.5 {
                                    m.settle = Some((now, m.drag_offset, 0.0, 0.25));
                                    f.continuous(b);
                                } else {
                                    m.settle = None;
                                    m.drag_offset = 0.0;
                                }
                            }
                            m.origin = None;
                            m.dragging = false;
                            m.samples.clear();
                        }
                        let vis_sr = cur_vis(m.drag_offset);
                        let prog = (1.0 - m.drag_offset / sw).clamp(0.0, 1.0);
                        let is_hidden_now = state.sheet_state.get(&sk) == Some(&SHEET_STATE_HIDDEN);
                        let visible_now = !is_hidden_now || m.settle.is_some() || prog > 0.001;
                        state.sheet_motion.insert(sk, m);
                        if visible_now {
                            active_sheets.push((i, n, base_sr, vis_sr, phys_right, was_dragging, is_hidden_now, prog));
                        }
                    }
                    let mut scrim_hit = b;
                    let mut any_open_sheet = false;
                    for &(_, _, _, sr, phys_right, _, is_hidden_now, _) in &active_sheets {
                        if !is_hidden_now {
                            any_open_sheet = true;
                        }
                        if phys_right {
                            let right_edge = sr.x.min(scrim_hit.x + scrim_hit.width);
                            scrim_hit.width = (right_edge - scrim_hit.x).max(0.0);
                        } else {
                            let left_edge = (sr.x + sr.width).max(scrim_hit.x);
                            let right_edge = scrim_hit.x + scrim_hit.width;
                            scrim_hit.x = left_edge;
                            scrim_hit.width = (right_edge - left_edge).max(0.0);
                        }
                    }
                    let dismissed = if !any_ss_dragging && any_open_sheet {
                        let scrim_resp = f.interact(anonymous_child_id(id, 9985), scrim_hit, enabled);
                        scrim_resp.clicked || (enabled && f.pressed_key(Key::Escape))
                    } else {
                        false
                    };
                    let mut paint_modal_sheets = !active_sheets.is_empty();
                    if dismissed {
                        let mut any_animating = false;
                        for &(i, n, base_sr, _, _, _, _, _) in &active_sheets {
                            let cid = child_view_id(id, i, n);
                            let sk = if n.id.is_empty() { format!("#{cid}") } else { n.id.to_owned() };
                            state.sheet_state.insert(sk.clone(), SHEET_STATE_HIDDEN);
                            let m = state.sheet_motion.entry(sk).or_default();
                            m.last_state = SHEET_STATE_HIDDEN;
                            if !f.reduced_motion() && now > 0.0 && (base_sr.width - m.drag_offset) > 0.5 {
                                m.settle = Some((now, m.drag_offset, base_sr.width, 0.25));
                                any_animating = true;
                            } else {
                                m.settle = None;
                                m.drag_offset = base_sr.width;
                            }
                        }
                        if any_animating {
                            for sheet in &mut active_sheets {
                                sheet.6 = true;
                            }
                        } else {
                            paint_modal_sheets = false;
                        }
                        f.continuous(b);
                    }
                    if paint_modal_sheets {
                        let max_prog = active_sheets
                            .iter()
                            .map(|s| s.7)
                            .fold(0.0_f32, f32::max);
                        let scrim_a = (82.0 * max_prog).round().clamp(0.0, 255.0) as u8;
                        if scrim_a > 0 {
                            f.paint_rect(b, 0.0, Color(0, 0, 0, scrim_a));
                        }
                        f.within(b, |f| {
                            for (i, n, _, vis_sr, _, was_dragging, is_hidden_now, _) in active_sheets {
                                let cid = child_view_id(id, i, n);
                                let mut child_action = None;
                                f.within(vis_sr, |f| {
                                    if was_dragging || is_hidden_now {
                                        f.without_input(true, |f| {
                                            n.draw(f, vis_sr, state, cid);
                                        });
                                    } else {
                                        child_action = n.draw(f, vis_sr, state, cid);
                                    }
                                });
                                if child_action.is_some() {
                                    action = child_action;
                                }
                            }
                        });
                    }
                }
                if let Some(previous)=media_namespace {state.adapter_namespace=previous;}
                if let Some(previous)=media_previous_capabilities {state.media_active_capabilities=previous;}
                if let Some(prev) = prev_snackbar {
                    state.snackbar_active = prev;
                }
                }
                }
            }
            Kind::Grid => {
                let (widths,heights,positions,baselines,margins)=self.grid_tracks(state,f.ui.text_metrics,id,inner.width,!matches!(self.width,Size::Wrap),if matches!(self.height,Size::Wrap) {None} else {Some(inner.height)});
                for (i,n) in self.children.iter().enumerate() {
                    if n.effective_visibility(state)==Visibility::Gone {continue;}
                    let (row,col,row_span,col_span)=positions[i];let margin=margins[i];let key=child_view_id(id,i,n);
                    let preceding=widths.iter().take(col).sum::<f32>();let track=widths[col..col+col_span].iter().sum::<f32>();
                    let base_x=if state.rtl {inner.x+inner.width-preceding-track+margin[0]} else {inner.x+preceding+margin[0]};
                    let base_y=inner.y+heights.iter().take(row).sum::<f32>()+margin[1];
                    let max_width=(track-margin[0]-margin[2]).max(0.0);let max_height=(heights[row..row+row_span].iter().sum::<f32>()-margin[1]-margin[3]).max(0.0);
                    let [horizontal,vertical]=n.layout_gravity;
                    let width=if matches!(n.width,Size::Match) || (n.layout_gravity_explicit && horizontal==5) || n.grid_weighted_fill(true) {max_width} else {n.layout_intrinsic(true,state,max_width,f.ui.text_metrics,key).min(max_width)};
                    let height=if matches!(n.height,Size::Match) || (n.layout_gravity_explicit && vertical==5) || n.grid_weighted_fill(false) {max_height} else {n.layout_intrinsic(false,state,width,f.ui.text_metrics,key).min(max_height)};
                    let dx=(max_width-width).max(0.0);let dy=(max_height-height).max(0.0);
                    let right=match horizontal {0=>state.rtl,2=>!state.rtl,4=>true,_=>false};
                    let x=base_x+if horizontal==1 {dx*0.5} else if right {dx} else {0.0};
                    let baseline=if n.grid_uses_baseline() && !matches!(n.height,Size::Match) {n.relative_baseline(width,height,state,f.ui.text_metrics,key)} else {None};
                    let alignment=baseline.and_then(|base|baselines.iter().find(|group|group.0==row && group.1==row_span).map(|group|group.2-base-margin[1]));
                    let y=base_y+alignment.unwrap_or_else(||match vertical {1=>dy*0.5,2=>dy,_=>0.0});
                    let rect=Rect{x,y,width,height};let mut child_action=None;
                    let mut draw=|f:&mut Frame<'_>| {child_action=n.draw(f,rect,state,key);};
                    let mut child=|f:&mut Frame<'_>| {if self.clip_children_disabled {draw(f)} else {f.within(rect,&mut draw)}};
                    if !self.clip_to_padding_disabled && inner!=b {f.within(inner,&mut child)} else {child(f)};
                    if child_action.is_some() {action=child_action;}
                }
            }
            Kind::Table => {
                let metrics=f.ui.text_metrics;let columns=self.table_columns_for_width(state,metrics,id,inner.width);let mut y=inner.y;
                for (i,row) in self.children.iter().enumerate() {
                    if row.effective_visibility(state)==Visibility::Gone {continue;}
                    let row_id=child_view_id(id,i,row);let margin=row.resolved_margin(state);y+=margin[1];
                    let parent_rtl=state.rtl;
                    if matches!(row.kind,Kind::TableRow) {match row.layout_direction {1=>state.rtl=false,2=>state.rtl=true,_=>{}}}
                    let height=if matches!(row.kind,Kind::TableRow) {self.table_row_height(row,&columns,state,metrics,row_id,id)} else {row.layout_intrinsic(false,state,inner.width,metrics,row_id)};
                    let rect=Rect{x:inner.x+margin[0],y,width:(inner.width-margin[0]-margin[2]).max(0.0),height};
                    if row.effective_visibility(state)==Visibility::Invisible {state.rtl=parent_rtl;y+=height+margin[3];continue;}
                    let mut draw_row=|f:&mut Frame<'_>| {
                    if matches!(row.kind,Kind::TableRow) {
                        let row_transform = row.view_transform();
                        if let Some(tr) = row_transform {
                            let origin = [
                                rect.x + tr.px.resolve(rect.width, state),
                                rect.y + tr.py.resolve(rect.height, state),
                            ];
                            f.push_transform(
                                origin,
                                [state.dp(tr.tx_dp), state.dp(tr.ty_dp)],
                                [tr.sx, tr.sy],
                                tr.rot_deg.to_radians(),
                            );
                        }
                        let row_pushed_layer = if row.view_alpha < 1.0 {
                            let opacity = row.view_alpha.clamp(0.0, 1.0);
                            let layer_bounds = if row.clip_children_disabled || row.clip_to_padding_disabled {
                                let min_x = rect.x.min(f.clip.x);
                                let min_y = rect.y.min(f.clip.y);
                                let max_x = (rect.x + rect.width).max(f.clip.x + f.clip.width);
                                let max_y = (rect.y + rect.height).max(f.clip.y + f.clip.height);
                                Rect {
                                    x: min_x,
                                    y: min_y,
                                    width: (max_x - min_x).max(0.0),
                                    height: (max_y - min_y).max(0.0),
                                }
                            } else {
                                rect
                            };
                            f.push_layer(layer_bounds, opacity);
                            true
                        } else {
                            false
                        };
                        let row_radius=if row.radius<0.0 {row.radius} else {state.dp(row.radius)};
                        if let Some(color)=row.background {f.paint_rect(rect,row_radius.max(0.0),color);}
                        if let Some(color)=row.background_stroke {f.commands.push(DrawCommand::Outline {bounds:rect,radius:row_radius,width:state.dp(row.background_stroke_width),color});}
                        let padding=row.resolved_padding(state);
                        let (_,ascents,descents)=self.table_row_metrics(row,&columns,state,metrics,row_id,id);
                        let baseline_aligned=!row.relative_rules.iter().any(|r|r.kind==36);
                        for (j,column,span) in row.table_cells() {
                            let preceding=columns.iter().take(column).sum::<f32>();let width=columns.iter().skip(column).take(span).sum::<f32>();
                            let x=if state.rtl {rect.x+rect.width-padding[2]-preceding-width} else {rect.x+padding[0]+preceding};
                            let cell=&row.children[j];if cell.effective_visibility(state)!=Visibility::Visible || self.table_cell_collapsed(state,id,column,span) {continue;}
                            let m=cell.resolved_margin(state);let cell_id=child_view_id(row_id,j,cell);
                            let available=(width-m[0]-m[2]).max(0.0);
                            let horizontal_gravity=cell.relative_rules.iter().any(|r|r.kind==35);
                            let cell_width=if horizontal_gravity {cell.layout_intrinsic(true,state,available,metrics,cell_id).min(available)} else {available};
                            let remaining=(available-cell_width).max(0.0);let gravity=cell.layout_gravity[0];
                            let shift=if !horizontal_gravity {0.0} else if gravity==1 {remaining*0.5} else if matches!(gravity,4) || (gravity==2 && !state.rtl) || (gravity==0 && state.rtl) {remaining} else {0.0};
                            let available_height=(height-padding[1]-padding[3]-m[1]-m[3]).max(0.0);
                            let cell_height=if matches!(cell.height,Size::Match) {available_height} else {cell.layout_intrinsic(false,state,cell_width,metrics,cell_id).min(available_height)};
                            let vertical=if cell.layout_gravity_explicit {cell.layout_gravity[1]} else {row.content_gravity[1]};
                            let inner_height=(height-padding[1]-padding[3]).max(0.0);
                            let baseline=if baseline_aligned && !matches!(cell.height,Size::Match) {cell.relative_baseline(cell_width,cell_height,state,metrics,cell_id)} else {None};
                            let cell_y=match vertical {
                                1=>y+padding[1]+(inner_height-cell_height)*0.5+m[1]-m[3],
                                2=>y+height-padding[3]-cell_height-m[3]-baseline.map_or(0.0,|base|descents[2]-(cell_height-base)),
                                _=>y+padding[1]+m[1]+baseline.map_or(0.0,|base|ascents[0]-base),
                            };
                            let cell_rect=Rect{x:x+m[0]+shift,y:cell_y,width:cell_width,height:cell_height};
                            let mut draw_cell=|f:&mut Frame<'_>| {if let Some(a)=cell.draw(f,cell_rect,state,cell_id) {action=Some(a);}};
                            let mut clip_cell=|f:&mut Frame<'_>| {if row.clip_children_disabled {draw_cell(f)} else {f.within(cell_rect,&mut draw_cell)}};
                            let row_inner=Rect{x:rect.x+padding[0],y:rect.y+padding[1],width:(rect.width-padding[0]-padding[2]).max(0.0),height:(rect.height-padding[1]-padding[3]).max(0.0)};
                            if !row.clip_to_padding_disabled && row_inner!=rect {f.within(row_inner,&mut clip_cell)} else {clip_cell(f)};
                        }
                        row.draw_post(f,rect,state);
                        if row_pushed_layer {
                            f.pop_layer();
                        }
                        if row_transform.is_some() {
                            f.pop_transform();
                        }
                    } else if let Some(a)=row.draw(f,rect,state,row_id) {action=Some(a);}
                    };
                    let mut clip_row=|f:&mut Frame<'_>| {if self.clip_children_disabled {draw_row(f)} else {f.within(rect,&mut draw_row)}};
                    if !self.clip_to_padding_disabled && inner!=b {f.within(inner,&mut clip_row)} else {clip_row(f)};
                    state.rtl=parent_rtl;
                    y+=height+margin[3];
                }
            }
            Kind::Relative | Kind::DialerFilter | Kind::ConstraintLayout => {
                let dialer=matches!(self.kind,Kind::DialerFilter);
                if dialer {
                    use crate::android_dialer_filter::DialerMode;
                    let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                    f.target(id,Rect{height:inner.height.min(48.0),..inner},"Dialer filter",self.enabled,Role::TextField);
                    state.input_types.insert(key.clone(),0);
                    let data=state.dialer_filters.entry(key.clone()).or_default();let letters_primary=matches!(data.mode,DialerMode::DigitsAndLettersNoDigits|DialerMode::LettersOnly);let primary=if letters_primary{data.letters.clone()}else{data.digits.clone()};let hint=if letters_primary{data.digits.clone()}else{data.letters.clone()};let editable=data.filter_text().to_owned();let hint_visibility=match data.mode{DialerMode::DigitsAndLetters=>Visibility::Visible,DialerMode::DigitsAndLettersNoDigits|DialerMode::DigitsAndLettersNoLetters=>Visibility::Invisible,_=>Visibility::Gone};state.text.insert(key,editable);state.text.insert("android:primary".into(),primary);state.text.insert("android:hint".into(),hint);state.visibility.insert("android:primary".into(),Visibility::Visible);state.visibility.insert("android:hint".into(),hint_visibility);
                }
                let is_cl = matches!(self.kind, Kind::ConstraintLayout);
                let (ph_to_target, target_to_ph) = if is_cl {
                    resolve_constraint_placeholders(self.children, state)
                } else {
                    (Vec::new(), Vec::new())
                };
                let cl_vis = if is_cl { Some(constraint_group_visibilities(self.children, state)) } else { None };
                let positions=if is_cl { constraint_layout_positions(self,id,inner,state,f.ui.text_metrics,false,false) } else { relative_layout::layout(self,id,inner,state,f.ui.text_metrics,false,false) };
                struct ClLayerGroup {
                    layer_idx: usize,
                    members: Vec<usize>,
                    union: Rect,
                    bg_rect: Rect,
                }
                let mut layer_groups: Vec<ClLayerGroup> = Vec::new();
                let mut slot_to_layer: Vec<Option<usize>> = vec![None; self.children.len()];
                if is_cl && self.children.iter().any(is_constraint_layer) {
                    let mut id_to_slot: HashMap<&'static str, usize> = HashMap::new();
                    for (ci, cn) in self.children.iter().enumerate() {
                        if !cn.id.is_empty() {
                            let slot = target_to_ph.get(ci).copied().flatten().unwrap_or(ci);
                            id_to_slot.insert(cn.id, slot);
                        }
                    }
                    for (li, ln) in self.children.iter().enumerate() {
                        if !is_constraint_layer(ln) {
                            continue;
                        }
                        if cl_vis.as_ref().is_some_and(|v| v[li] != Visibility::Visible) {
                            continue;
                        }
                        let mut members: Vec<usize> = Vec::new();
                        for rule in ln.relative_rules.iter().filter(|r| r.kind == RULE_CONSTRAINT_LAYER) {
                            for raw_id in rule.anchor.split(',') {
                                let rid = raw_id.trim();
                                if let Some(&slot) = id_to_slot.get(rid)
                                    && slot != li
                                    && slot_to_layer[slot].is_none()
                                    && !members.contains(&slot)
                                    && !is_constraint_helper(&self.children[slot])
                                    && target_to_ph.get(slot).copied().flatten().is_none()
                                    && cl_vis.as_ref().is_none_or(|v| v[slot] == Visibility::Visible)
                                {
                                    members.push(slot);
                                }
                            }
                        }
                        if members.is_empty() {
                            continue;
                        }
                        members.sort_unstable();
                        let mut min_x = f32::INFINITY;
                        let mut min_y = f32::INFINITY;
                        let mut max_x = f32::NEG_INFINITY;
                        let mut max_y = f32::NEG_INFINITY;
                        for &ms in &members {
                            let mr = positions[ms];
                            min_x = min_x.min(mr.x);
                            min_y = min_y.min(mr.y);
                            max_x = max_x.max(mr.x + mr.width);
                            max_y = max_y.max(mr.y + mr.height);
                        }
                        let union = Rect {
                            x: min_x,
                            y: min_y,
                            width: (max_x - min_x).max(0.0),
                            height: (max_y - min_y).max(0.0),
                        };
                        let lp = ln.resolved_padding(state);
                        let bg_rect = Rect {
                            x: union.x - lp[0],
                            y: union.y - lp[1],
                            width: (union.width + lp[0] + lp[2]).max(0.0),
                            height: (union.height + lp[1] + lp[3]).max(0.0),
                        };
                        let g_idx = layer_groups.len();
                        for &ms in &members {
                            slot_to_layer[ms] = Some(g_idx);
                        }
                        layer_groups.push(ClLayerGroup {
                            layer_idx: li,
                            members,
                            union,
                            bg_rect,
                        });
                    }
                }
                let padding_clip = !self.clip_to_padding_disabled && inner != b;
                let render_slot = |slot_i: usize, f: &mut Frame<'_>, state: &mut LayoutState| -> Option<&'static str> {
                    let vis = cl_vis.as_ref().map_or_else(|| self.children[slot_i].effective_visibility(state), |v| v[slot_i]);
                    if vis == Visibility::Gone
                        || (is_cl
                            && (is_constraint_helper(&self.children[slot_i])
                                || vis == Visibility::Invisible
                                || target_to_ph.get(slot_i).copied().flatten().is_some()))
                    {
                        return None;
                    }
                    let (n, child_id) = if is_cl
                        && let Some(t_idx) = ph_to_target.get(slot_i).copied().flatten()
                    {
                        let tn = &self.children[t_idx];
                        (tn, child_view_id(id, t_idx, tn))
                    } else {
                        let cn = &self.children[slot_i];
                        (cn, child_view_id(id, slot_i, cn))
                    };
                    let rect = positions[slot_i];
                    let pointer_occluded = !dialer
                        && !f.any_scroll_dragging()
                        && f.input.pointer.is_some_and(|ptr| {
                            self.children[slot_i + 1..].iter().enumerate().any(|(offset, later)| {
                                let later_i = slot_i + 1 + offset;
                                if is_cl && target_to_ph.get(later_i).copied().flatten().is_some() {
                                    return false;
                                }
                                let lvis = cl_vis.as_ref().map_or_else(|| later.effective_visibility(state), |v| v[later_i]);
                                let eff_later = if is_cl
                                    && let Some(t_idx) = ph_to_target.get(later_i).copied().flatten()
                                {
                                    &self.children[t_idx]
                                } else {
                                    later
                                };
                                let eff_later_id = child_view_id(id, later_i, eff_later);
                                lvis == Visibility::Visible
                                    && !(is_cl && is_constraint_helper(later))
                                    && eff_later.blocks_pointer_for_lower_siblings_in(state, eff_later_id)
                                    && positions[later_i].intersect(f.clip).contains(ptr)
                            })
                        });
                    let mut child_action = None;
                    let mut draw = |f: &mut Frame<'_>| {
                        let semantics = f.ui.semantics.len();
                        f.without_input(dialer, |f| {
                            f.without_pointer_input(pointer_occluded, |f| {
                                child_action = n.draw(f, rect, state, child_id);
                            });
                        });
                        if dialer {
                            f.ui.semantics.truncate(semantics);
                        }
                    };
                    if self.clip_children_disabled {
                        draw(f);
                    } else {
                        f.within(rect, &mut draw);
                    }
                    child_action
                };
                for i in 0..self.children.len() {
                    if let Some(g_idx) = slot_to_layer[i] {
                        let group = &layer_groups[g_idx];
                        if group.members[0] != i {
                            continue;
                        }
                        let ln = &self.children[group.layer_idx];
                        let union = group.union;
                        let bg_rect = group.bg_rect;
                        let mut draw_group = |f: &mut Frame<'_>| {
                            let xml_tr = ln.view_transform();
                            let rt_scale = if !ln.id.is_empty() {
                                state.view_scale.get(ln.id).copied()
                            } else {
                                None
                            };
                            let rt_trans = if !ln.id.is_empty() {
                                state.view_translation.get(ln.id).copied()
                            } else {
                                None
                            };
                            let rt_alpha = if !ln.id.is_empty() {
                                state.view_alpha.get(ln.id).copied().unwrap_or(1.0)
                            } else {
                                1.0
                            };
                            let origin = [union.x + union.width * 0.5, union.y + union.height * 0.5];
                            let (xml_tx, xml_ty, xml_sx, xml_sy, rot_deg) = if let Some(tr) = xml_tr {
                                (tr.tx_dp, tr.ty_dp, tr.sx, tr.sy, tr.rot_deg)
                            } else {
                                (0.0, 0.0, 1.0, 1.0, 0.0)
                            };
                            let [rtx, rty] = rt_trans.unwrap_or([0.0, 0.0]);
                            let [rsx, rsy] = rt_scale.unwrap_or([1.0, 1.0]);
                            let tx = xml_tx + rtx;
                            let ty = xml_ty + rty;
                            let sx = xml_sx * rsx;
                            let sy = xml_sy * rsy;
                            let pushed_tr = if tx.abs() > 1e-4
                                || ty.abs() > 1e-4
                                || (sx - 1.0).abs() > 1e-4
                                || (sy - 1.0).abs() > 1e-4
                                || rot_deg.abs() > 1e-4
                            {
                                f.push_transform(
                                    origin,
                                    [state.dp(tx), state.dp(ty)],
                                    [sx, sy],
                                    rot_deg.to_radians(),
                                );
                                true
                            } else {
                                false
                            };
                            let eff_alpha = (ln.view_alpha * rt_alpha).clamp(0.0, 1.0);
                            let pushed_alpha = if eff_alpha < 1.0 {
                                let has_elev = ln.relative_rules.iter().any(|r| r.kind == RULE_ELEVATION);
                                let layer_bounds = if has_elev {
                                    let pad = state.dp(24.0);
                                    Rect {
                                        x: bg_rect.x - pad,
                                        y: bg_rect.y - pad,
                                        width: bg_rect.width + pad * 2.0,
                                        height: bg_rect.height + pad * 2.0,
                                    }
                                    .intersect(f.clip)
                                } else {
                                    bg_rect
                                };
                                f.push_layer(layer_bounds, eff_alpha);
                                true
                            } else {
                                false
                            };
                            ln.paint_constraint_layer_chrome(f, bg_rect, state);
                            for &ms in &group.members {
                                if let Some(a) = render_slot(ms, f, state) {
                                    action = Some(a);
                                }
                            }
                            if pushed_alpha {
                                f.pop_layer();
                            }
                            if pushed_tr {
                                f.pop_transform();
                            }
                        };
                        if padding_clip {
                            f.within(inner, &mut draw_group);
                        } else {
                            draw_group(f);
                        }
                    } else {
                        let mut child_action = None;
                        if padding_clip {
                            f.within(inner, |f| {
                                child_action = render_slot(i, f, state);
                            });
                        } else {
                            child_action = render_slot(i, f, state);
                        }
                        if child_action.is_some() {
                            action = child_action;
                        }
                    }
                }
            }
            Kind::Scroll | Kind::HorizontalScroll => {
                if matches!(self.kind, Kind::HorizontalScroll)
                    && let Some(car_spec) = self.carousel_layout_spec()
                {
                    let scroll_input = if self.clip_to_padding_disabled { b } else { inner };
                    action = self.draw_carousel_layout(f, scroll_input, state, id, car_spec, t);
                } else if let Some(child) = self.children.first().filter(|child| child.effective_visibility(state) != Visibility::Gone) {
                    let horizontal = matches!(self.kind, Kind::HorizontalScroll);
                    let margin = child.resolved_margin(state);
                    let viewport = if horizontal { inner.width } else { inner.height };
                    let along_margin = if horizontal { margin[0] + margin[2] } else { margin[1] + margin[3] };
                    let child_width = (inner.width - margin[0] - margin[2]).max(0.0);
                    let natural = child.layout_intrinsic(horizontal, state, child_width, f.ui.text_metrics,child_view_id(id,0,child)).max(0.0);
                    let length = if self.fill_viewport { natural.max((viewport - along_margin).max(0.0)) } else { natural };
                    let max_offset = (length + along_margin - viewport).max(0.0);
                    let cross_available = if horizontal { (inner.height - margin[1] - margin[3]).max(0.0) } else { (inner.width - margin[0] - margin[2]).max(0.0) };
                    let cross_size = if horizontal { child.height } else { child.width };
                    let cross = match cross_size {
                        Size::Match => cross_available,
                        Size::Wrap => child.layout_intrinsic(!horizontal, state,child_width,f.ui.text_metrics,child_view_id(id,0,child)).min(cross_available),
                        Size::Dp(value) => state.dp(value).min(cross_available),
                    }.max(0.0);
                    let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                    let offset = state.offsets.entry(key).or_default();
                    let prev_input = f.input;
                    let scroll_input = if self.clip_to_padding_disabled { b } else { inner };
                    let dragging = if horizontal { f.scroll_viewport_horizontal_rtl(id, scroll_input, offset, max_offset, state.rtl) }
                    else { f.scroll_viewport(id, scroll_input, offset, max_offset) };
                    let (stretch_s, stretch_sign) = f.scroll_stretch(id);
                    let child_bounds = if horizontal {
                        Rect { x: if state.rtl {inner.x + inner.width - margin[2] - length + *offset} else {inner.x + margin[0] - *offset}, y: inner.y + margin[1], width: length, height: cross }
                    } else {
                        Rect { x: inner.x + margin[0], y: inner.y + margin[1] - *offset, width: cross, height: length }
                    };
                    let child_id = if child.id.is_empty() { anonymous_child_id(id, 0) } else { view_id(child.id) };
                    let mut draw_child = |f: &mut Frame<'_>| {
                        f.cancel_pointer_input(dragging, |f| {
                            f.with_edge_stretch(scroll_input, horizontal, stretch_s, stretch_sign, |f| {
                                if self.clip_children_disabled { action = child.draw(f, child_bounds, state, child_id); }
                                else { f.within(child_bounds, |f| { action = child.draw(f, child_bounds, state, child_id); }); }
                            });
                        });
                    };
                    f.within(scroll_input, &mut draw_child);
                    f.input = prev_input;
                }
            }
            _ => {}
        }
        *action_out = action;
    }
}
