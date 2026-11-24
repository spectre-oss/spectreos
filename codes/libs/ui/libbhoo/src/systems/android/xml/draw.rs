use super::*;

pub(super) struct DrawCtx<'a> {
    pub(super) t: Theme,
    pub(super) enabled: bool,
    pub(super) view_response: &'a mut Response,
    pub(super) view_key: String,
    pub(super) explicit_background: Option<Color>,
    pub(super) background_tint: Option<Color>,
    pub(super) scaled_radius: f32,
    pub(super) corner_radii: Option<[f32; 4]>,
    pub(super) corner_family: [bool; 4],
    pub(super) gradient: Option<GradientSpec>,
    pub(super) has_layers: bool,
    pub(super) has_stateful_layers: bool,
    pub(super) has_bg_image_rule: bool,
    pub(super) explicit_elevation: Option<f32>,
    pub(super) is_drag_handle: bool,
    pub(super) alert_dialog_icon: Option<&'static str>,
    pub(super) is_snackbar: bool,
    pub(super) is_floating_toolbar: bool,
    pub(super) is_m3_theme: bool,
    pub(super) split_role: Option<SplitButtonRole>,
    pub(super) chip_spec: Option<XmlChipSpec>,
    pub(super) toggle_seg: Option<ToggleGroupSegment>,
    pub(super) ext_fab: Option<(ExtendedFabSpec, f32)>,
    pub(super) snackbar_text_btn: bool,
    pub(super) active_btn_variant: Option<XmlButtonVariant>,
    pub(super) til_spec: Option<TextInputBoxSpec<'static>>,
    pub(super) nav_spec: Option<NavBarSpec>,
    pub(super) search_bar_hint: Option<&'static str>,
    pub(super) bottom_app_bar: Option<BottomAppBarSpec>,
    pub(super) collapsing_spec: Option<CollapsingToolbarSpec<'static>>,
    pub(super) view_pager_spec: Option<ViewPagerSpec>,
    pub(super) use_default_button_chrome: bool,
    pub(super) padding: [f32; 4],
    pub(super) inner: Rect,
    pub(super) action: &'a mut Option<&'static str>,
}

impl Node {
    pub(super) fn draw_contents(
        &'static self, f:&mut Frame<'_>, mut b:Rect, state:&mut LayoutState, id:u64, wrapped_card:bool,
    ) -> Option<&'static str> {
        let is_root_draw = state.at_root_draw;
        state.at_root_draw = false;
        if self.effective_visibility(state) != Visibility::Visible {
            state.suppress_next_elevation = false;
            if matches!(self.kind,Kind::Zoom) { state.zoom_holds.remove(&id); }
            if matches!(self.kind,Kind::ZoomControls) {
                state.zoom_holds.remove(&anonymous_child_id(id,0));
                state.zoom_holds.remove(&anonymous_child_id(id,1));
                let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                state.zoom_controls_actions.remove(&key);
            }
            return None;
        }
        let suppress_elev = std::mem::take(&mut state.suppress_next_elevation);
        if matches!(self.kind, Kind::Space) { return None; }
        if matches!(self.kind,Kind::MaterialCard) && !wrapped_card {
            let variant=match self.relative_rules.iter().find(|r|r.kind==57).map(|r|r.anchor) {Some("Filled")=>CardVariant::Filled,Some("Elevated")=>CardVariant::Elevated,_=>CardVariant::Outlined};
            let key=if self.id.is_empty() {format!("#{id}")} else {self.id.to_owned()};
            let checkable=self.relative_rules.iter().any(|r|r.kind==58);
            let checked=checkable && state.checked.get(&key).copied().unwrap_or(false);
            let dragged=state.view_dragged.get(&key).copied().unwrap_or(false);
            let rtl=match self.layout_direction {1=>false,2=>true,_=>state.rtl};
            let rule=|kind| self.relative_rules.iter().find(|r|r.kind==kind).map(|r|r.anchor);
            let icon_size=state.dp(rule(59).and_then(|s|s.parse().ok()).unwrap_or(24.0));
            let icon_margin=state.dp(rule(60).and_then(|s|s.parse().ok()).unwrap_or(8.0));
            let icon_gravity=match rule(61) {Some("TOP_START")=>CardIconGravity::TopStart,Some("BOTTOM_START")=>CardIconGravity::BottomStart,Some("BOTTOM_END")=>CardIconGravity::BottomEnd,_=>CardIconGravity::TopEnd};
            let theme=f.theme();
            let theme_color=|name:&str| match name {
                "@colorPrimary"=>Some(theme.primary),"@colorOnPrimary"=>Some(theme.on_primary),"@colorSecondary"=>Some(theme.secondary),"@colorOnSecondary"=>Some(theme.on_secondary),
                "@colorSurface"=>Some(theme.background),"@colorOnSurface"=>Some(theme.text),"@colorOutline"=>Some(theme.outline),"@colorOutlineVariant"=>Some(theme.outline_variant),
                "@colorOnSurfaceVariant"=>Some(theme.on_surface_variant),"@colorSurfaceContainerLow"=>Some(theme.surface_low),"@colorSurfaceContainerHigh"=>Some(theme.surface_high),"@colorSurfaceContainerHighest"=>Some(theme.surface_highest),_=>None};
            let read_color=|kind| rule(kind).and_then(|s| {
                if s.starts_with('@') {return theme_color(s)}
                let parts:Vec<_>=s.split(',').map(str::parse::<u8>).collect();match parts.as_slice() {[Ok(r),Ok(g),Ok(b),Ok(a)]=>Some(Color(*r,*g,*b,*a)),_=>None}
            });
            let enabled=self.enabled && f.enabled;
            let pressed=f.ui.active==Some(id) || (enabled && self.clickable && f.input.pressed && f.pointer_in(b));
            let bits=u16::from(enabled) | (u16::from(pressed)<<1) | (u16::from(f.ui.focused==Some(id))<<2)
                | (u16::from(state.view_selected.get(&key).copied().unwrap_or(false))<<3) | (u16::from(state.view_activated.get(&key).copied().unwrap_or(false))<<4)
                | (u16::from(f.input.pointer_kind!=PointerKind::Touch && f.pointer_in(b))<<5) | (u16::from(checked)<<6) | (u16::from(dragged)<<7) | (u16::from(checkable)<<8);
            let state_color=|kind| {
                let mut first=None;let mut default=None;
                for r in self.relative_rules.iter().filter(|r|r.kind==kind) {
                    let mut parts=r.anchor.split(',');
                    let parsed=(|| {
                        let required=parts.next()?.parse::<u16>().ok()?;let forbidden=parts.next()?.parse::<u16>().ok()?;let first=parts.next()?;
                        let color=if first.starts_with('@') {let alpha=parts.next()?.parse::<u8>().ok()?;theme_color(first)?.opacity(alpha as f32/255.0)}
                            else {let red=first.parse().ok()?;let green=parts.next()?.parse().ok()?;let blue=parts.next()?.parse().ok()?;let alpha=parts.next()?.parse().ok()?;Color(red,green,blue,alpha)};
                        if parts.next().is_some() {return None;}Some((required,forbidden,color))
                    })();
                    if let Some((required,forbidden,color))=parsed {
                        if first.is_none(){first=Some(color)}
                        if required==0 && forbidden==0 {default=Some(color)}
                        if bits & required == required && bits & forbidden == 0 {return Some(color)}
                    }
                }
                default.or(first)
            };
            let m3 = matches!(state.theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark);
            let icon_tint=state_color(75).or_else(||read_color(62));
            let card_bg = state_color(76).or_else(|| read_color(63)).or(self.background).or_else(|| {
                m3.then(|| match (variant, enabled) {
                    (CardVariant::Filled, true) => theme.surface_highest,
                    (CardVariant::Filled, false) => theme.surface_highest.mix(theme.surface_variant, 0.38),
                    (CardVariant::Elevated, true) => theme.surface_low,
                    (CardVariant::Elevated, false) => theme.background,
                    (CardVariant::Outlined, _) => theme.background,
                })
            });
            let card_stroke_color = state_color(77).or_else(|| read_color(64)).or(self.background_stroke).or_else(|| {
                (m3 && variant == CardVariant::Outlined).then(|| {
                    if !enabled {
                        theme.surface_low.mix(theme.outline, 0.12)
                    } else if checked {
                        theme.secondary
                    } else {
                        theme.outline_variant
                    }
                })
            });
            let card_stroke_width = rule(65)
                .and_then(|s| s.parse::<f32>().ok())
                .or_else(|| (self.background_stroke.is_some() && self.background_stroke_width > 0.0).then_some(self.background_stroke_width))
                .map(|w| state.dp(w))
                .or_else(|| (m3 && variant == CardVariant::Outlined).then(|| state.dp(1.0)));
            let card_corner_radius = rule(66)
                .and_then(|s| s.parse::<f32>().ok())
                .or_else(|| (self.radius > 0.0).then_some(self.radius))
                .map(|r| state.dp(r))
                .or_else(|| (m3 || (state.effective_density() - 1.0).abs() > f32::EPSILON).then(|| state.dp(12.0)));
            let appearance=crate::systems::material::CardAppearance {background:card_bg,foreground:state_color(79).or_else(||read_color(78)),stroke_color:card_stroke_color,stroke_width:card_stroke_width,corner_radius:card_corner_radius,elevation:rule(71).and_then(|s|s.parse::<f32>().ok()).map(|v| state.dp(v)),ripple_color:state_color(81).or_else(||read_color(80)).or_else(||self.resolved_ripple().map(|(c,_)|c))};
            let mut child_action=None;
            let (h,v)=self.card_compat_insets(state);
            let body=Rect {x:b.x+h,y:b.y+v,width:(b.width-2.0*h).max(0.0),height:(b.height-2.0*v).max(0.0)};
            let clip_card_outline = self.clip_to_outline();
            let card_r_clamped = card_corner_radius
                .filter(|v| v.is_finite())
                .unwrap_or(12.0)
                .clamp(0.0, body.width.min(body.height) * 0.5);
            let card_outline_radii = self.resolved_corner_radii(state).unwrap_or([card_r_clamped; 4]);
            let card_cut = self.resolved_corner_family();
            let card_has_cut = has_active_cut_corner(card_cut, card_outline_radii);
            let card_stroke_repaint = if clip_card_outline || card_has_cut {
                let sw = appearance
                    .stroke_width
                    .unwrap_or(if variant == CardVariant::Outlined { state.dp(1.0) } else { 0.0 })
                    .max(0.0)
                    .min(body.width.min(body.height) * 0.5);
                let sc = appearance.stroke_color.unwrap_or_else(|| {
                    if !enabled {
                        theme.surface_low.mix(theme.outline, 0.12)
                    } else if checked {
                        theme.secondary
                    } else {
                        theme.outline_variant
                    }
                });
                (sw > 0.0).then_some((sw, sc))
            } else {
                None
            };
            let response = if card_has_cut && body.width > 0.0 && body.height > 0.0 {
                let hovered = self.clickable
                    && enabled
                    && f.input.pointer_kind != PointerKind::Touch
                    && f.input.pointer.is_some_and(|p| body.intersect(f.clip).contains(p));
                let card_elev = match variant {
                    CardVariant::Elevated => if hovered && !pressed { 2 } else { 1 },
                    _ if hovered && !pressed => 1,
                    _ => 0,
                };
                let density = state.effective_density();
                if let Some(dp) = rule(71).and_then(|s| s.parse::<f32>().ok()) {
                    paint_elevation_dp(f, body, card_r_clamped, dp, density);
                } else if card_elev > 0 && (enabled || variant == CardVariant::Elevated) {
                    paint_elevation_level(f, body, card_r_clamped, card_elev, density);
                }
                if dragged && enabled {
                    paint_elevation_level(f, body, card_r_clamped, 6, density);
                }
                let surface = appearance.background.unwrap_or(match (variant, enabled) {
                    (CardVariant::Filled, true) => theme.surface_highest,
                    (CardVariant::Filled, false) => theme.surface_highest.mix(theme.surface_variant, 0.38),
                    (CardVariant::Elevated, true) => theme.surface_low,
                    (CardVariant::Elevated, false) => theme.background,
                    (CardVariant::Outlined, _) => theme.background,
                });
                paint_cut_shape_fill(f, state, body, card_outline_radii, card_cut, surface);
                f.within(body, |f| {
                    let previous = f.content_color;
                    if !enabled {
                        f.content_color = Some(previous.unwrap_or(theme.text).opacity(if previous.is_some() { 1.0 } else { 0.38 }));
                    }
                    f.with_enabled(enabled, |f| {
                        child_action = self.draw_contents(f, b, state, id, true);
                    });
                    f.content_color = previous;
                });
                if let Some(fg) = appearance.foreground {
                    paint_cut_shape_fill(f, state, body, card_outline_radii, card_cut, fg);
                } else if checked {
                    paint_cut_shape_fill(f, state, body, card_outline_radii, card_cut, theme.secondary.opacity(0.16));
                } else if dragged {
                    paint_cut_shape_fill(f, state, body, card_outline_radii, card_cut, theme.text.opacity(0.08));
                }
                if checked && icon_size.is_finite() && icon_size > 0.0 && body.width >= icon_size + 2.0 * icon_margin && body.height >= icon_size + 2.0 * icon_margin {
                    let at_end = matches!(icon_gravity, CardIconGravity::TopEnd | CardIconGravity::BottomEnd);
                    let at_bottom = matches!(icon_gravity, CardIconGravity::BottomStart | CardIconGravity::BottomEnd);
                    let icon = Rect {
                        x: if at_end != rtl { body.x + body.width - icon_margin - icon_size } else { body.x + icon_margin },
                        y: if at_bottom { body.y + body.height - icon_margin - icon_size } else { body.y + icon_margin },
                        width: icon_size,
                        height: icon_size,
                    };
                    let index = crate::beercss_shapes::SHAPES.len()
                        + crate::icons::material_navigation::ICONS.len()
                        + crate::icons::material_symbols::ICONS.len()
                        + 1
                        + crate::shapes::android_clock_shapes::SHAPES.len();
                    let tint = icon_tint.unwrap_or(if enabled { theme.secondary } else { theme.outline.opacity(0.12) });
                    f.commands.push(DrawCommand::Shape { bounds: icon, index: index as u8, color: tint, rotation: 0.0 });
                }
                if let Some((sw, sc)) = card_stroke_repaint {
                    paint_cut_shape_outline(f, body, card_outline_radii, card_cut, sw, sc);
                }
                if self.clickable {
                    let resp = f.target(id, body, self.text, enabled, Role::Button);
                    let default_ink = if checked && !resp.pressed || checkable && !checked && resp.pressed {
                        theme.secondary
                    } else {
                        theme.on_surface_variant
                    };
                    let ink = appearance.ripple_color.unwrap_or(default_ink);
                    if resp.pressed {
                        paint_cut_shape_fill(f, state, body, card_outline_radii, card_cut, ink.opacity(0.12));
                    } else if resp.hovered {
                        paint_cut_shape_fill(f, state, body, card_outline_radii, card_cut, ink.opacity(0.08));
                    }
                    f.focus_ring(resp);
                    Some(resp)
                } else {
                    None
                }
            } else {
                f.card_with_state_inset(id,body,self.text,CardOptions {variant,interactive:self.clickable,enabled:self.enabled},CardVisualState {checked,checkable,dragged,rtl,checked_icon_size:icon_size,checked_icon_margin:icon_margin,checked_icon_gravity:icon_gravity,checked_icon_tint:icon_tint,..CardVisualState::default()},0.0,appearance,|f| {
                    let child_start = f.commands.len();
                    child_action=self.draw_contents(f,b,state,id,true);
                    if clip_card_outline {
                        clip_commands_to_rounded_outline(f, child_start, body, false, card_outline_radii);
                        if let Some((sw, sc)) = card_stroke_repaint {
                            paint_outline_radii(f, body, card_outline_radii, sw, sc);
                        }
                    }
                })
            };
            if let Some(tip) = self.tooltip_text() {
                let tip_response = if let Some(r) = response {
                    r
                } else if f.ui.seen.contains(&id) {
                    Response {
                        id,
                        bounds: body,
                        enabled,
                        hovered: enabled && f.input.pointer_kind != PointerKind::Touch && f.pointer_in(body),
                        focused: enabled && !f.feedback_blocked && f.ui.focused == Some(id),
                        ..Response::default()
                    }
                } else if f.ui.hover.is_some_and(|h| h != id) {
                    Response { id, bounds: body, enabled, ..Response::default() }
                } else {
                    f.interact(id, body, self.enabled)
                };
                f.tooltip(anonymous_child_id(id, 99_991), tip_response, tip);
            }
            return child_action.or_else(||response.filter(|r|r.clicked).map(|_|self.id));
        }
        let inherited_rtl = state.rtl;
        match self.layout_direction { 1 => state.rtl = false, 2 => state.rtl = true, _ => {} }
        let t = f.theme();
        let enabled=self.enabled && f.enabled;
        let interactive_view = self.clickable
            || matches!(
                self.kind,
                Kind::Button
                    | Kind::Icon
                    | Kind::Zoom
                    | Kind::CheckBox
                    | Kind::RadioButton
                    | Kind::Switch
                    | Kind::ToggleButton
                    | Kind::SeekBar
                    | Kind::Spinner
                    | Kind::Edit
                    | Kind::AutoComplete
                    | Kind::MultiAutoComplete
            );
        let mut view_response=state.adapter_root_response.filter(|(root,_)|*root==id).map(|(_,response)|response).unwrap_or_else(||Response {
            id,bounds:b,enabled,hovered:enabled && f.input.pointer_kind!=PointerKind::Touch && f.pointer_in(b),
            pressed:enabled && !f.input.released && !f.input.cancelled && !f.input.blurred && (f.ui.active==Some(id) || interactive_view && f.input.pressed && f.pointer_in(b)),
            focused:enabled && !f.feedback_blocked && (f.ui.focused==Some(id) || interactive_view && f.input.pressed && !f.input.cancelled && !f.input.blurred && f.ui.active.is_none() && f.pointer_in(b)),
            ..Response::default()
        });
        let view_key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        let explicit_background=if wrapped_card {
            None
        } else if self.relative_rules.iter().any(|rule|rule.kind==96) {
            self.resolved_background(state,&view_key,view_response)
        } else {self.background};
        let background_tint = self.relative_rules.iter().find(|r| r.kind == RULE_BACKGROUND_TINT).and_then(|r| {
            let mut parts = r.anchor.split(',').filter_map(|p| p.trim().parse::<u8>().ok());
            Some(Color(parts.next()?, parts.next()?, parts.next()?, parts.next()?))
        });
        let scaled_radius = if self.radius < 0.0 { self.radius } else { state.dp(self.radius) };
        if let Some(sv_spec) = self.search_view_ext() {
            let sv_action = self.draw_search_view_ext(
                f,
                b,
                state,
                id,
                sv_spec,
                t,
                background_tint,
                explicit_background,
                scaled_radius,
            );
            state.rtl = inherited_rtl;
            return sv_action;
        }
        let corner_radii = self.resolved_corner_radii(state);
        let corner_family = self.resolved_corner_family();
        let gradient = self.relative_rules.iter().find(|r| r.kind == RULE_GRADIENT).and_then(|r| parse_gradient(r.anchor));
        let has_layers = self.relative_rules.iter().any(|r| r.kind == RULE_DRAWABLE_LAYER);
        let has_stateful_layers = self.relative_rules.iter().any(|r| r.kind == RULE_STATEFUL_DRAWABLE_LAYER);
        let bg_dynamic = if !wrapped_card {
            self.dynamic_drawable_bg()
        } else {
            None
        };
        let bg_anim_list = if !wrapped_card && bg_dynamic.is_none() && !self.is_image_src_kind() {
            self.animation_list_spec()
        } else {
            None
        };
        let bg_anim_vec = if !wrapped_card && bg_dynamic.is_none() && bg_anim_list.is_none() {
            self.animated_vector_spec(false)
        } else {
            None
        };
        let bg_anim_sel_frame = if !wrapped_card
            && bg_dynamic.is_none()
            && bg_anim_list.is_none()
            && bg_anim_vec.is_none()
            && self.has_animated_selector(false)
        {
            self.sample_animated_selector(f, b, state, &view_key, view_response, false)
        } else {
            None
        };
        let has_bg_image_rule = !wrapped_card
            && (bg_dynamic.is_some()
                || bg_anim_list.is_some()
                || bg_anim_vec.is_some()
                || bg_anim_sel_frame.is_some()
                || self
                    .relative_rules
                    .iter()
                    .any(|r| r.kind == RULE_BACKGROUND_IMAGE || r.kind == RULE_STATEFUL_BG_IMAGE));
        let bg_image = if bg_dynamic.is_some() {
            None
        } else if let Some(ref spec) = bg_anim_list {
            let (frame_key, next_wake) = select_animation_list_frame(spec, f.time(), f.reduced_motion());
            if let Some(wake) = next_wake {
                let vis_rect = b.intersect(f.clip);
                if vis_rect.width > 0.0 && vis_rect.height > 0.0 {
                    f.wake_at(wake);
                }
            }
            Some((frame_key, None))
        } else if let Some(ref spec) = bg_anim_vec {
            let (frame_key, next_wake) = select_animated_vector_frame(spec, f.time(), f.reduced_motion());
            if let Some(wake) = next_wake {
                let vis_rect = b.intersect(f.clip);
                if vis_rect.width > 0.0 && vis_rect.height > 0.0 {
                    f.wake_at(wake);
                }
            }
            Some((frame_key, None))
        } else if let Some(frame_key) = bg_anim_sel_frame {
            Some((frame_key, None))
        } else if has_bg_image_rule {
            self.resolved_bg_image(state, &view_key, view_response)
        } else {
            None
        };
        let explicit_elevation = if suppress_elev {
            None
        } else {
            self.relative_rules.iter().find(|r| r.kind == RULE_ELEVATION).and_then(|r| r.anchor.trim().parse::<f32>().ok()).filter(|v| v.is_finite() && *v > 0.0)
        };
        let lift_on_scroll = self.lift_on_scroll_spec();
        let lift_t = if let Some(target_opt) = lift_on_scroll {
            let scroll_off = if let Some(target_id) = target_opt {
                state
                    .offsets
                    .get(target_id)
                    .copied()
                    .filter(|v| v.is_finite())
                    .unwrap_or(0.0)
            } else {
                state
                    .collapsing_scroll_offset
                    .filter(|v| v.is_finite())
                    .unwrap_or(0.0)
            };
            let lifted_now = scroll_off > 0.5;
            let now = f.time();
            let mut lm = state.app_bar_lift_motion.get(&view_key).copied().unwrap_or_default();
            let instant = f.reduced_motion() || (now == 0.0 && lm.last_time == 0.0);
            let target = if lifted_now { 1.0 } else { 0.0 };
            if !lm.initialized {
                lm.initialized = true;
                lm.last_lifted = lifted_now;
                lm.last_time = now;
                lm.progress = target;
            } else {
                if let Some((start, from, to, dur)) = lm.anim {
                    if instant || dur <= 0.0 {
                        lm.progress = to;
                        lm.anim = None;
                    } else {
                        let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                        let ease = 1.0 - (1.0 - p) * (1.0 - p);
                        lm.progress = (from + (to - from) * ease).clamp(0.0, 1.0);
                        if p >= 1.0 {
                            lm.progress = to;
                            lm.anim = None;
                        } else {
                            f.continuous(b);
                        }
                    }
                }
                if lifted_now != lm.last_lifted {
                    lm.last_lifted = lifted_now;
                    if instant || (lm.progress - target).abs() <= 0.001 {
                        lm.progress = target;
                        lm.anim = None;
                    } else {
                        lm.anim = Some((now, lm.progress, target, 0.15));
                        f.continuous(b);
                    }
                }
                lm.last_time = now;
            }
            state.app_bar_lift_motion.insert(view_key.clone(), lm);
            lm.progress
        } else {
            0.0
        };
        let bottom_sheet_spec = self.bottom_sheet_spec();
        let side_sheet_spec = self.side_sheet_spec();
        let is_drag_handle = self.is_drag_handle();
        let alert_dialog_icon = self.alert_dialog_icon();
        let is_snackbar = self.is_snackbar();
        let is_floating_toolbar = self.is_floating_toolbar();
        let is_carousel_item = self.is_carousel_item();
        let is_m3_theme = matches!(state.theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark);
        let is_m3_picker = is_m3_theme && matches!(self.kind, Kind::DatePicker | Kind::TimePicker);
        let split_role = if matches!(self.kind, Kind::Button) { state.split_button_role } else { None };
        let scaled_radius = if self.radius == 0.0 && corner_radii.is_none() {
            if alert_dialog_icon.is_some() || is_carousel_item || is_m3_picker {
                state.dp(28.0)
            } else if is_snackbar {
                state.dp(4.0)
            } else if is_floating_toolbar {
                b.width.min(b.height) * 0.5
            } else {
                scaled_radius
            }
        } else {
            scaled_radius
        };
        let corner_radii = if corner_radii.is_none() && self.radius == 0.0 {
            if bottom_sheet_spec.is_some() {
                let r28 = state.dp(28.0);
                Some([r28, r28, 0.0, 0.0])
            } else if let Some(ss) = side_sheet_spec {
                let r16 = state.dp(16.0);
                let phys_right = if ss.edge_start { state.rtl } else { !state.rtl };
                Some(if phys_right { [r16, 0.0, 0.0, r16] } else { [0.0, r16, r16, 0.0] })
            } else {
                None
            }
        } else {
            corner_radii
        };
        let chip_spec = self.chip_style();
        let toggle_seg = if matches!(self.kind, Kind::Button) && chip_spec.is_none() {
            state.toggle_group_segment
        } else {
            None
        };
        let ext_fab: Option<(ExtendedFabSpec, f32)> =
            if toggle_seg.is_none() && chip_spec.is_none() && split_role.is_none() {
                if let Some((spec, next_m, anim)) = self.extended_fab_progress(state, id) {
                    if state.extended_fab_motion.len() >= 64
                        && !state.extended_fab_motion.contains_key(&view_key)
                    {
                        state.extended_fab_motion.clear();
                    }
                    state.extended_fab_motion.insert(view_key.clone(), next_m);
                    if anim {
                        f.continuous(b);
                    }
                    let collapsed_w = state.dp(spec.collapsed_size_dp).max(state.dp(self.min_width));
                    let collapsed_h = state.dp(spec.collapsed_size_dp).max(state.dp(self.min_height));
                    let ext_h = state.dp(56.0).max(state.dp(self.min_height));
                    let p = next_m.progress;
                    let should_size_w = is_root_draw || !matches!(self.width, Size::Wrap);
                    if should_size_w {
                        let intrinsic_ext_w = self
                            .extended_fab_intrinsic_extended_width(state, f.ui.text_metrics, id, spec)
                            .max(collapsed_w);
                        let ext_w = if spec.strategy == ExtendedFabStrategy::MatchParent {
                            b.width.max(collapsed_w)
                        } else if !spec.has_ext && is_root_draw {
                            b.width.max(collapsed_w)
                        } else if matches!(self.width, Size::Dp(_)) {
                            b.width.max(collapsed_w)
                        } else {
                            intrinsic_ext_w.min(b.width).max(collapsed_w.min(b.width))
                        };
                        let col_clamped = collapsed_w.min(b.width);
                        let anim_w = (col_clamped + (ext_w - col_clamped) * p).clamp(0.0, b.width);
                        if (anim_w - b.width).abs() > 0.01 {
                            let right = match self.layout_gravity[0] {
                                0 => state.rtl,
                                2 => !state.rtl,
                                4 => true,
                                _ => false,
                            };
                            let x = if self.layout_gravity[0] == 1 {
                                b.x + (b.width - anim_w) * 0.5
                            } else if right {
                                b.x + b.width - anim_w
                            } else {
                                b.x
                            };
                            b.x = x;
                            b.width = anim_w;
                        }
                    }
                    if is_root_draw && spec.has_ext {
                        let anim_h = (collapsed_h + (ext_h - collapsed_h) * p).clamp(0.0, b.height);
                        if (anim_h - b.height).abs() > 0.01 {
                            b.y += (b.height - anim_h) * 0.5;
                            b.height = anim_h;
                        }
                    }
                    Some((spec, p))
                } else {
                    None
                }
            } else {
                None
            };
        let snackbar_text_btn = state.snackbar_active
            && matches!(self.kind, Kind::Button)
            && chip_spec.is_none()
            && toggle_seg.is_none()
            && split_role.is_none()
            && !self.has_explicit_custom_background()
            && self.button_variant().is_none_or(|v| v == XmlButtonVariant::Text);
        let active_btn_variant = if toggle_seg.is_some() || ext_fab.is_some() || snackbar_text_btn {
            None
        } else if split_role.is_some() {
            Some(self.button_variant().unwrap_or(XmlButtonVariant::Filled))
        } else {
            self.active_button_variant()
        };
        let til_spec = self.text_input_box();
        let nav_spec = self.nav_bar_spec();
        let search_bar_hint = self.search_bar_hint();
        let bottom_app_bar = self.bottom_app_bar_spec();
        if let Some(bab) = bottom_app_bar
            && bab.hide_on_scroll
            && !state.bab_hide_applied_by_parent
        {
            let scroll_px = self.app_bar_scroll_px(state, id);
            let (hs, anim) = eval_bottom_app_bar_hide(&view_key, scroll_px, state);
            if state.bottom_app_bar_hide_motion.len() >= 64
                && !state.bottom_app_bar_hide_motion.contains_key(&view_key)
            {
                state.bottom_app_bar_hide_motion.clear();
            }
            state.bottom_app_bar_hide_motion.insert(view_key.clone(), hs);
            if anim {
                f.continuous(b);
            }
            b.y += b.height * hs.progress;
        }
        let collapsing_spec = self.collapsing_toolbar_spec();
        let view_pager_spec = self.view_pager_spec();
        let use_default_button_chrome = toggle_seg.is_none()
            && ext_fab.is_none()
            && split_role.is_none()
            && !snackbar_text_btn
            && self.uses_default_button_chrome(state);
        let has_no_custom_bg = explicit_background.is_none()
            && background_tint.is_none()
            && gradient.is_none()
            && !has_layers
            && !has_stateful_layers
            && !has_bg_image_rule;
        let use_default_toolbar_chrome = matches!(self.kind, Kind::Toolbar)
            && !state.collapsing_toolbar_child
            && !state.lifted_appbar_active
            && (state.theme_style != AndroidTheme::Bhoo || bottom_app_bar.is_some())
            && has_no_custom_bg;
        let use_default_m3_tab_surface = matches!(self.kind, Kind::TabWidget)
            && is_m3_theme
            && has_no_custom_bg;
        let use_default_nav_surface = nav_spec.is_some()
            && !(nav_spec.is_some_and(|s| s.kind == NavBarKind::Drawer) && state.drawer_panel_active)
            && has_no_custom_bg;
        let suppress_generic_bg = chip_spec.is_some()
            || toggle_seg.is_some()
            || ext_fab.is_some()
            || active_btn_variant.is_some()
            || search_bar_hint.is_some()
            || collapsing_spec.is_some()
            || is_drag_handle
            || snackbar_text_btn
            || (til_spec.is_some_and(|s| s.mode == TextInputBoxMode::Filled)
                && gradient.is_none()
                && !has_layers
                && !has_stateful_layers
                && !has_bg_image_rule);
        let lift_color = self.app_bar_lift_color();
        let mut lift_color_applied = false;
        let background = if suppress_generic_bg || gradient.is_some() || has_layers || has_stateful_layers || has_bg_image_rule || use_default_button_chrome {
            explicit_background
        } else if use_default_toolbar_chrome {
            let base = if bottom_app_bar.is_some() {
                Some(match state.theme_style {
                    AndroidTheme::Material3Light | AndroidTheme::MaterialLight => Color(0xF3, 0xED, 0xF7, 255),
                    AndroidTheme::Material3Dark | AndroidTheme::MaterialDark => Color(0x21, 0x1F, 0x26, 255),
                    AndroidTheme::Bhoo => t.surface_low,
                })
            } else {
                state.theme_style.toolbar_surface()
            };
            if lift_on_scroll.is_some() && lift_t > 0.001 {
                let lift_target = lift_color.unwrap_or(t.surface_high);
                lift_color_applied = true;
                Some(base.map_or(lift_target.opacity(lift_t), |c| c.mix(lift_target, lift_t)))
            } else {
                base
            }
        } else if lift_on_scroll.is_some() && has_no_custom_bg {
            let lift_target = lift_color.unwrap_or(t.surface_high);
            lift_color_applied = true;
            (lift_t > 0.001).then(|| lift_target.opacity(lift_t))
        } else if use_default_m3_tab_surface {
            Some(t.background)
        } else if let Some(nav) = nav_spec.filter(|_| use_default_nav_surface) {
            Some(match nav.kind {
                NavBarKind::Bar => match state.theme_style {
                    AndroidTheme::Material3Light => Color(0xF3, 0xED, 0xF7, 255),
                    AndroidTheme::Material3Dark => Color(0x21, 0x1F, 0x26, 255),
                    _ => t.surface_low,
                },
                NavBarKind::Rail => t.background,
                NavBarKind::Drawer => t.surface_low,
            })
        } else if has_no_custom_bg && (bottom_sheet_spec.is_some() || side_sheet_spec.is_some()) {
            Some(t.surface_low)
        } else if has_no_custom_bg && (alert_dialog_icon.is_some() || is_m3_picker) {
            Some(t.surface_high)
        } else if has_no_custom_bg && is_snackbar {
            Some(t.inverse_surface)
        } else if has_no_custom_bg && is_floating_toolbar {
            Some(match state.theme_style {
                AndroidTheme::Material3Light => Color(0xF3, 0xED, 0xF7, 255),
                AndroidTheme::Material3Dark => Color(0x21, 0x1F, 0x26, 255),
                _ => t.surface_low,
            })
        } else {
            let base = background_tint.or(explicit_background);
            if let Some(lc) = lift_color
                && lift_on_scroll.is_some()
                && lift_t > 0.001
            {
                lift_color_applied = true;
                Some(base.map_or(lc.opacity(lift_t), |c| c.mix(lc, lift_t)))
            } else {
                base
            }
        };
        if !use_default_button_chrome && active_btn_variant.is_none() && chip_spec.is_none() && toggle_seg.is_none() && ext_fab.is_none() && !wrapped_card {
            let elev_radius = if self.radius < 0.0 {
                b.width.min(b.height) * 0.5
            } else if scaled_radius > 0.0 {
                scaled_radius
            } else if let Some([tl, tr, br, bl]) = corner_radii {
                tl.max(tr).max(br).max(bl)
            } else {
                self.resolved_state_radius(state)
            };
            let density = state.effective_density();
            if lift_on_scroll.is_some() {
                if lift_t > 0.001 {
                    paint_elevation_level_alpha(f, b, elev_radius, 2, density, lift_t);
                }
            } else if let Some(dp) = explicit_elevation {
                paint_elevation_level(f, b, elev_radius, elevation_level_from_dp(dp), density);
            } else if bottom_sheet_spec.is_some() {
                paint_elevation_level(f, b, elev_radius, 2, density);
            } else if side_sheet_spec.is_some_and(|ss| !ss.coplanar) {
                paint_elevation_level(f, b, elev_radius, 1, density);
            } else if alert_dialog_icon.is_some() || is_snackbar || is_floating_toolbar {
                paint_elevation_level(f, b, elev_radius, 3, density);
            } else if use_default_toolbar_chrome && bottom_app_bar.is_none() && matches!(state.theme_style, AndroidTheme::MaterialLight | AndroidTheme::MaterialDark) {
                paint_elevation_dp(f, b, elev_radius, 4.0, density);
            } else if matches!(self.kind, Kind::Icon)
                && self.radius < 0.0
                && enabled
                && is_m3_theme
            {
                paint_elevation_level(f, b, elev_radius, 3, density);
            }
        }
        if ext_fab.is_some() {
            let fab_radius = if self.radius < 0.0 {
                b.width.min(b.height) * 0.5
            } else if self.radius > 0.0 {
                scaled_radius
            } else if let Some([tl, tr, br, bl]) = corner_radii {
                (tl + tr + br + bl) * 0.25
            } else {
                state.dp(16.0)
            };
            let density = state.effective_density();
            if let Some(dp) = explicit_elevation {
                paint_elevation_level(f, b, fab_radius, elevation_level_from_dp(dp), density);
            } else if enabled {
                paint_elevation_level(f, b, fab_radius, 3, density);
            }
            let is_oval = self.radius < 0.0 && corner_radii.is_none();
            if let Some(spec) = gradient {
                let radii = corner_radii.unwrap_or([fab_radius.max(0.0); 4]);
                paint_gradient(f, state, b, spec, is_oval, radii, corner_family, background_tint);
            } else if !has_layers && !has_stateful_layers && !has_bg_image_rule {
                let fill = if let Some(bg) = explicit_background {
                    bg
                } else {
                    background_tint
                        .unwrap_or(t.primary_container)
                        .opacity(if enabled { 1.0 } else { 0.38 })
                };
                paint_shape_fill_with_family(f, state, b, is_oval, corner_radii, fab_radius, corner_family, fill);
            }
            if let Some(sc) = self.background_stroke {
                let sw = if self.background_stroke_width > 0.0 { state.dp(self.background_stroke_width) } else { state.dp(1.0) };
                paint_shape_outline_with_family(f, b, is_oval, corner_radii, fab_radius, corner_family, sw, sc);
            }
        } else if let Some(variant) = active_btn_variant {
            let visual_b = if split_role.is_some() {
                let inset_y = if b.height > state.dp(40.0) {
                    ((b.height - state.dp(40.0)) * 0.5).min(state.dp(4.0))
                } else {
                    0.0
                };
                Rect {
                    x: b.x,
                    y: b.y + inset_y,
                    width: b.width,
                    height: (b.height - inset_y * 2.0).max(0.0),
                }
            } else {
                self.default_button_visual_bounds(b, state)
            };
            let btn_corner_radii = if corner_radii.is_none() && self.radius == 0.0 && let Some(sr) = split_role {
                let outer_r = visual_b.height * 0.5;
                let inner_r = state.dp(4.0).min(outer_r);
                Some(if sr.is_phys_left {
                    [outer_r, inner_r, inner_r, outer_r]
                } else {
                    [inner_r, outer_r, outer_r, inner_r]
                })
            } else {
                corner_radii
            };
            let visual_radius = if self.radius < 0.0 {
                visual_b.width.min(visual_b.height) * 0.5
            } else if self.radius > 0.0 {
                scaled_radius
            } else if let Some([tl, tr, br, bl]) = btn_corner_radii {
                (tl + tr + br + bl) * 0.25
            } else {
                visual_b.height * 0.5
            };
            if variant != XmlButtonVariant::Text {
                let density = state.effective_density();
                if let Some(dp) = explicit_elevation {
                    paint_elevation_level(f, visual_b, visual_radius, elevation_level_from_dp(dp), density);
                } else if variant == XmlButtonVariant::Elevated && enabled {
                    paint_elevation_level(f, visual_b, visual_radius, 1, density);
                }
            }
            let is_oval = self.radius < 0.0 && btn_corner_radii.is_none();
            let base_fill = match variant {
                XmlButtonVariant::Filled => Some(background_tint.unwrap_or(t.primary)),
                XmlButtonVariant::Tonal => Some(background_tint.unwrap_or(t.secondary_container)),
                XmlButtonVariant::Elevated => Some(background_tint.unwrap_or(t.surface_low)),
                XmlButtonVariant::Outlined | XmlButtonVariant::Text => background_tint.filter(|c| c.3 > 0),
            };
            if let Some(base_fill) = base_fill {
                let fill = base_fill.opacity(if enabled { 1.0 } else { 0.38 });
                paint_shape_fill_with_family(f, state, visual_b, is_oval, btn_corner_radii, visual_radius, corner_family, fill);
            }
            if variant == XmlButtonVariant::Outlined {
                let raw_stroke = self.background_stroke.unwrap_or(t.outline);
                let stroke_color = if enabled { raw_stroke } else { t.text.opacity(0.12) };
                let sw = if self.background_stroke.is_some() && self.background_stroke_width > 0.0 {
                    state.dp(self.background_stroke_width)
                } else {
                    state.dp(1.0)
                };
                paint_shape_outline_with_family(f, visual_b, is_oval, btn_corner_radii, visual_radius, corner_family, sw, stroke_color);
            } else if let Some(sc) = self.background_stroke {
                let sw = if self.background_stroke_width > 0.0 { state.dp(self.background_stroke_width) } else { state.dp(1.0) };
                paint_shape_outline_with_family(f, visual_b, is_oval, btn_corner_radii, visual_radius, corner_family, sw, sc);
            }
        } else if use_default_button_chrome {
            let visual_b = self.default_button_visual_bounds(b, state);
            let visual_radius = if self.radius < 0.0 {
                visual_b.width.min(visual_b.height) * 0.5
            } else if self.radius > 0.0 {
                scaled_radius
            } else if let Some([tl, tr, br, bl]) = corner_radii {
                (tl + tr + br + bl) * 0.25
            } else {
                match state.theme_style {
                    AndroidTheme::MaterialLight | AndroidTheme::MaterialDark => state.dp(2.0),
                    AndroidTheme::Material3Light | AndroidTheme::Material3Dark => visual_b.height * 0.5,
                    AndroidTheme::Bhoo => 0.0,
                }
            };
            let density = state.effective_density();
            if let Some(dp) = explicit_elevation {
                paint_elevation_level(f, visual_b, visual_radius, elevation_level_from_dp(dp), density);
            } else if enabled && matches!(state.theme_style, AndroidTheme::MaterialLight | AndroidTheme::MaterialDark) {
                paint_material_button_shadow(f, visual_b, visual_radius, density);
            }
            let base_fill = background_tint.unwrap_or_else(|| match state.theme_style {
                AndroidTheme::MaterialLight => Color(0xD6, 0xD7, 0xD7, 255),
                AndroidTheme::MaterialDark => Color(0x5A, 0x59, 0x5B, 255),
                AndroidTheme::Material3Light => Color(0x67, 0x50, 0xA4, 255),
                AndroidTheme::Material3Dark => Color(0xD0, 0xBC, 0xFF, 255),
                AndroidTheme::Bhoo => t.control,
            });
            let fill = base_fill.opacity(if enabled { 1.0 } else { 0.38 });
            let is_oval = self.radius < 0.0 && corner_radii.is_none();
            paint_shape_fill_with_family(f, state, visual_b, is_oval, corner_radii, visual_radius, corner_family, fill);
        } else if !suppress_generic_bg {
            let is_oval = self.radius < 0.0 && corner_radii.is_none();
            if let Some(spec) = gradient {
                let radii = corner_radii.unwrap_or([scaled_radius.max(0.0); 4]);
                paint_gradient(f, state, b, spec, is_oval, radii, corner_family, background_tint);
            } else if let Some(color) = background {
                if let Some(bab) = bottom_app_bar && bab.has_ext {
                    let fab_d = state.dp(56.0);
                    let (fab_cx, fab_cy) = bottom_app_bar_fab_center(b, bab, fab_d, state);
                    let cutout_r = fab_d * 0.5 + state.dp(bab.cradle_margin_dp);
                    match bab.anchor_mode {
                        BottomAppBarAnchorMode::Cradle => {
                            paint_bottom_app_bar_cradle(
                                f,
                                state,
                                b,
                                fab_cx - b.x,
                                state.dp(bab.cradle_voff_dp),
                                cutout_r,
                                state.dp(bab.cradle_rad_dp),
                                color,
                            );
                        }
                        BottomAppBarAnchorMode::Embed => {
                            paint_shape_fill_with_family(f, state, b, is_oval, corner_radii, scaled_radius, corner_family, color);
                            let pocket = Rect {
                                x: fab_cx - cutout_r,
                                y: fab_cy - cutout_r,
                                width: cutout_r * 2.0,
                                height: cutout_r * 2.0,
                            };
                            let pocket_fill = match state.theme_style {
                                AndroidTheme::Bhoo => t.background,
                                _ => t.surface_highest,
                            };
                            f.within(b, |f| {
                                f.commands.push(DrawCommand::Ellipse { bounds: pocket, color: pocket_fill });
                                f.commands.push(DrawCommand::Outline {
                                    bounds: pocket,
                                    radius: cutout_r,
                                    width: state.dp(1.0),
                                    color: t.outline_variant,
                                });
                            });
                        }
                    }
                } else {
                    paint_shape_fill_with_family(f, state, b, is_oval, corner_radii, scaled_radius, corner_family, color);
                }
            }
        }
        if let Some(ref spec) = bg_dynamic
            && b.width > 0.0
            && b.height > 0.0
        {
            let bg_img_start = f.commands.len();
            paint_dynamic_bg_drawable(f, state, b, &view_key, spec, background_tint);
            let is_oval = self.radius < 0.0 && corner_radii.is_none();
            let outline_radii = corner_radii.unwrap_or([scaled_radius.max(0.0); 4]);
            clip_commands_to_rounded_outline(f, bg_img_start, b, is_oval, outline_radii);
        } else if let Some((img_key, stretch)) = bg_image
            && let Some(img) = state.images.get(img_key).cloned()
            && b.width > 0.0
            && b.height > 0.0
        {
            let bg_img_start = f.commands.len();
            let density = state.image_scales.get(img_key).copied().unwrap_or(1.0) * state.effective_density();
            if let Some(ref bands) = stretch {
                paint_nine_patch_image(f, state, b, &img, bands, background_tint, 1, 255, density);
            } else {
                let tinted = tint_image(state, &img, background_tint, 1);
                let mirror_bg = bg_anim_vec.is_some() && self.is_auto_mirrored() && state.rtl;
                if mirror_bg {
                    f.push_transform(
                        [b.x + b.width * 0.5, b.y + b.height * 0.5],
                        [0.0, 0.0],
                        [-1.0, 1.0],
                        0.0,
                    );
                }
                f.commands.push(DrawCommand::Image {
                    bounds: b,
                    uv: [0.0, 0.0, 1.0, 1.0],
                    radius: 0.0,
                    color: Color(255, 255, 255, 255),
                    image: tinted,
                });
                if mirror_bg {
                    f.pop_transform();
                }
            }
            let is_oval = self.radius < 0.0 && corner_radii.is_none();
            let outline_radii = corner_radii.unwrap_or([scaled_radius.max(0.0); 4]);
            clip_commands_to_rounded_outline(f, bg_img_start, b, is_oval, outline_radii);
        }
        if has_layers {
            let d = state.effective_density();
            for rule in self.relative_rules.iter().filter(|r| r.kind == RULE_DRAWABLE_LAYER) {
                if let Some(layer) = parse_drawable_layer(rule.anchor) {
                    paint_drawable_layer(f, b, layer, background_tint, d);
                }
            }
        }
        if has_stateful_layers && bg_anim_sel_frame.is_none() {
            self.paint_stateful_drawable_layers(f, b, state, &view_key, view_response, background_tint);
        }
        if lift_on_scroll.is_some() && lift_t > 0.001 && !has_no_custom_bg && !suppress_generic_bg && !lift_color_applied {
            let is_oval = self.radius < 0.0 && corner_radii.is_none();
            let overlay = lift_color
                .map(|lc| lc.opacity(lift_t))
                .unwrap_or_else(|| t.primary.opacity(0.08 * lift_t));
            paint_shape_fill_with_family(
                f,
                state,
                b,
                is_oval,
                corner_radii,
                scaled_radius,
                corner_family,
                overlay,
            );
        }
        if !wrapped_card && active_btn_variant.is_none() && chip_spec.is_none() && toggle_seg.is_none() && ext_fab.is_none() && let Some(color) = self.background_stroke {
            let sw = state.dp(self.background_stroke_width);
            let is_oval = self.radius < 0.0 && corner_radii.is_none();
            paint_shape_outline_with_family(f, b, is_oval, corner_radii, scaled_radius, corner_family, sw, color);
        }
        let prev_lifted_appbar = state.lifted_appbar_active;
        if lift_on_scroll.is_some() && lift_t > 0.001 && matches!(self.kind, Kind::Linear | Kind::Frame) {
            state.lifted_appbar_active = true;
        }
        let mut padding = self.resolved_padding(state);
        if let Some((_, fab_p)) = ext_fab {
            let rtl = state.rtl;
            let ext_def = if rtl { [20.0, 0.0, 16.0, 0.0] } else { [16.0, 0.0, 20.0, 0.0] };
            if !self.padding_explicit[0] && self.padding_edges[0] == 0.0 && self.padding_relative[if rtl { 1 } else { 0 }].is_none() {
                padding[0] = state.dp(ext_def[0]) * fab_p;
            }
            if !self.padding_explicit[2] && self.padding_edges[2] == 0.0 && self.padding_relative[if rtl { 0 } else { 1 }].is_none() {
                padding[2] = state.dp(ext_def[2]) * fab_p;
            }
        }
        let inner = Rect { x: b.x + padding[0], y: b.y + padding[1], width: (b.width - padding[0] - padding[2]).max(0.0), height: (b.height - padding[1] - padding[3]).max(0.0) };
        let mut action = None;
        // These controls create their own target in their widget branch.
        let container_clickable = self.clickable && !matches!(self.kind,Kind::Button|Kind::Icon|Kind::Zoom|Kind::ZoomControls|Kind::CheckBox|Kind::RadioButton|Kind::Switch|Kind::SeekBar|Kind::RatingBar|Kind::ToggleButton|Kind::Edit|Kind::AutoComplete|Kind::MultiAutoComplete|Kind::MaterialCard|Kind::Spinner|Kind::ListView|Kind::GridView|Kind::QuickContactBadge|Kind::VideoView|Kind::CalendarView|Kind::DatePicker|Kind::NumberPicker|Kind::TimePicker|Kind::SearchView|Kind::ExpandableListView);
        let defer_container_target = container_clickable && (!self.children.is_empty() || self.has_clickable_text_links());
        let container_sem_start = f.ui.semantics.len();
        if container_clickable && !defer_container_target {
            let response = f.target(id, b, self.text, self.enabled, Role::Button);
            view_response=response;
            let state_radius = if self.radius < 0.0 { b.width.min(b.height) * 0.5 } else { self.resolved_state_radius(state) };
            self.paint_state_layer(f, response, state_radius, corner_radii, t.text);
            f.focus_ring(response);
            if response.clicked {
                action = Some(self.id);
            }
        }
        let clip_outline_active = !wrapped_card && (self.clip_to_outline() || is_carousel_item);
        let prev_clip_outline = if clip_outline_active {
            let old_clip = f.clip;
            let next_clip = b.intersect(old_clip);
            f.clip = next_clip;
            f.commands.push(DrawCommand::PushClip(next_clip));
            Some((old_clip, f.commands.len()))
        } else {
            None
        };
        {
            let ctx = DrawCtx {
                t,
                enabled,
                view_response: &mut view_response,
                view_key,
                explicit_background,
                background_tint,
                scaled_radius,
                corner_radii,
                corner_family,
                gradient,
                has_layers,
                has_stateful_layers,
                has_bg_image_rule,
                explicit_elevation,
                is_drag_handle,
                alert_dialog_icon,
                is_snackbar,
                is_floating_toolbar,
                is_m3_theme,
                split_role,
                chip_spec,
                toggle_seg,
                ext_fab,
                snackbar_text_btn,
                active_btn_variant,
                til_spec,
                nav_spec,
                search_bar_hint,
                bottom_app_bar,
                collapsing_spec,
                view_pager_spec,
                use_default_button_chrome,
                padding,
                inner,
                action: &mut action,
            };
            match self.kind {
                Kind::ExpandableListView
                | Kind::SlidingDrawer
                | Kind::TabHost
                | Kind::TabWidget
                | Kind::Gallery
                | Kind::StackView
                | Kind::AdapterViewFlipper
                | Kind::Toolbar
                | Kind::ActionMenuView
                | Kind::SearchView
                | Kind::CalendarView
                | Kind::TimePicker
                | Kind::NumberPicker
                | Kind::DatePicker
                | Kind::GridView
                | Kind::ListView
                | Kind::Spinner => self.draw_adapter_views(f, b, state, id, ctx),
                Kind::Linear
                | Kind::RadioGroup
                | Kind::TableRow
                | Kind::Frame
                | Kind::MediaController
                | Kind::ViewFlipper
                | Kind::ViewAnimator
                | Kind::ViewSwitcher
                | Kind::TextSwitcher
                | Kind::ImageSwitcher
                | Kind::Absolute
                | Kind::MaterialCard
                | Kind::Grid
                | Kind::Table
                | Kind::Relative
                | Kind::DialerFilter
                | Kind::ConstraintLayout
                | Kind::Scroll
                | Kind::HorizontalScroll => self.draw_containers(f, b, state, id, ctx),
                Kind::Text
                | Kind::CheckedText
                | Kind::Chronometer
                | Kind::TextClock
                | Kind::VideoView
                | Kind::Image
                | Kind::QuickContactBadge
                | Kind::AnalogClock
                | Kind::ToggleButton
                | Kind::RatingBar
                | Kind::ProgressBar
                | Kind::SeekBar
                | Kind::RadioButton
                | Kind::CheckBox
                | Kind::Switch
                | Kind::Button
                | Kind::Icon
                | Kind::Zoom
                | Kind::ZoomControls
                | Kind::Edit
                | Kind::AutoComplete
                | Kind::MultiAutoComplete
                | Kind::MaterialDivider
                | Kind::View
                | Kind::Space => self.draw_widgets(f, b, state, id, ctx),
            }
        }
        if let Some(ss) = side_sheet_spec && ss.coplanar {
            let div_w = state.dp(1.0).min(b.width);
            let phys_right = if ss.edge_start { state.rtl } else { !state.rtl };
            let div_x = if phys_right { b.x } else { b.x + b.width - div_w };
            f.paint_rect(
                Rect { x: div_x, y: b.y, width: div_w, height: b.height },
                0.0,
                t.outline_variant,
            );
        }
        if defer_container_target {
            let child_owns_pointer = action.is_some()
                || f.ui.active.is_some_and(|a| a != id)
                || f.input.pointer.is_some_and(|p| {
                    f.ui.semantics[container_sem_start..]
                        .iter()
                        .any(|s| s.enabled && s.bounds.contains(p))
                });
            let response = f.without_pointer_input(child_owns_pointer, |f| {
                f.target(id, b, self.text, self.enabled, Role::Button)
            });
            view_response = response;
            let state_radius = if self.radius < 0.0 {
                b.width.min(b.height) * 0.5
            } else {
                self.resolved_state_radius(state)
            };
            self.paint_state_layer(f, response, state_radius, corner_radii, t.text);
            f.focus_ring(response);
            if response.clicked && action.is_none() {
                action = Some(self.id);
            }
        }
        if let Some((old_clip, clip_child_start)) = prev_clip_outline {
            let is_oval = self.radius < 0.0 && corner_radii.is_none();
            let outline_radii = corner_radii.unwrap_or([scaled_radius.max(0.0); 4]);
            clip_commands_to_rounded_outline(f, clip_child_start, b, is_oval, outline_radii);
            f.commands.push(DrawCommand::PopClip);
            f.clip = old_clip;
            if active_btn_variant.is_none() && chip_spec.is_none() && toggle_seg.is_none() && let Some(color) = self.background_stroke {
                let sw = state.dp(self.background_stroke_width);
                if sw > 0.0 {
                    paint_shape_outline_with_family(f, b, is_oval, corner_radii, scaled_radius, corner_family, sw, color);
                }
            }
        }
        if !wrapped_card
            && self.nav_bar_spec().is_none()
            && self.bottom_app_bar_spec().is_none()
            && !matches!(self.kind, Kind::TabWidget)
            && let Some(badge) = self.standalone_badge(state)
        {
            let ext = self.material_badge_ext(None, self.id);
            paint_nav_badge_ext(f, b, badge, ext, state, if enabled { 1.0 } else { 0.38 });
        }
        if !wrapped_card && let Some(tip) = self.tooltip_text() {
            let tip_response = if f.ui.seen.contains(&id) {
                view_response
            } else if f.ui.hover.is_some_and(|h| h != id) {
                Response { hovered: false, ..view_response }
            } else {
                f.interact(id, b, self.enabled)
            };
            f.tooltip(anonymous_child_id(id, 99_991), tip_response, tip);
        }
        state.lifted_appbar_active = prev_lifted_appbar;
        state.rtl = inherited_rtl;
        action
    }
}
