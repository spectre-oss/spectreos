use super::draw::DrawCtx;
use super::*;

impl Node {
    pub(super) fn draw_widgets(
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
            split_role,
            chip_spec,
            toggle_seg,
            ext_fab,
            snackbar_text_btn,
            active_btn_variant,
            use_default_button_chrome,
            padding,
            inner,
            view_response: view_response_out,
            action: action_out,
            ..
        } = ctx;
        let mut view_response = *view_response_out;
        let mut action = *action_out;
        match self.kind {
            Kind::Text | Kind::CheckedText | Kind::Chronometer | Kind::TextClock => {
                if matches!(self.kind, Kind::TextClock) {
                    let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                    if let Some(clock) = state.wall_clock.filter(|clock| clock.unix_seconds.is_finite()) {
                        let preferred = if clock.is_24_hour { self.clock_24 } else { self.clock_12 };
                        let fallback = if clock.is_24_hour { self.clock_12 } else { self.clock_24 };
                        let pattern = if !preferred.is_empty() { preferred } else if !fallback.is_empty() { fallback } else if clock.is_24_hour { "HH:mm" } else { "h:mm a" };
                        state.text.insert(key, format_wall_clock(clock,pattern));
                        let seconds = clock_pattern_has_seconds(pattern);
                        let period = if seconds { 1.0 } else { 60.0 };
                        let next = period-(clock.unix_seconds+f64::from(clock.offset_seconds)).rem_euclid(period);
                        let visible = b.intersect(f.clip);
                        if visible.width > 0.0 && visible.height > 0.0 { f.wake_at(f.time()+next.max(0.001)); }
                    } else { state.text.insert(key,"--:--".into()); }
                }
                if matches!(self.kind, Kind::Chronometer) {
                    let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                    let now = f.time();
                    let clock = state.chronometers.entry(key.clone()).or_insert(ChronometerState { base:now, running:false });
                    if clock.running || !state.text.contains_key(&key) {
                        // Match Android's millisecond rounding before converting to whole seconds.
                        let elapsed = if self.count_down { clock.base-now-0.499 } else { now-clock.base };
                        let rounded = (elapsed+0.5).floor();
                        let sign = if rounded < 0.0 { "-" } else { "" };
                        let seconds = rounded.abs().min(u32::MAX as f64) as u32;
                        let value = if seconds >= 3600 { format!("{sign}{}:{:02}:{:02}",seconds/3600,(seconds/60)%60,seconds%60) } else { format!("{sign}{:02}:{:02}",seconds/60,seconds%60) };
                        let value = if self.chrono_format.is_empty() { value } else { format_chronometer(self.chrono_format,&value) };
                        state.text.insert(key,value);
                    }
                    let visible = b.intersect(f.clip);
                    if clock.running && visible.width > 0.0 && visible.height > 0.0 {
                        let delay = if self.count_down {
                            let remainder = (clock.base-now) % 1.0;
                            if remainder <= 0.0 { remainder+1.0 } else { remainder }
                        } else { 1.0-(now-clock.base).abs()%1.0 };
                        f.wake_at(now+delay+0.001);
                    }
                }
                let inner = if matches!(self.kind, Kind::CheckedText) {
                    let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                    let checked = *state.checked.entry(key.clone()).or_insert(self.checked);
                    let mark_width = inner.width.min(self.check_mark_width(state));
                    let gravity=self.relative_rules.iter().find(|rule|rule.kind==157).map(|rule|rule.anchor).unwrap_or("end");
                    let mark_left=match gravity {"start"=>!state.rtl,"left"=>true,"right"=>false,_=>state.rtl};
                    let x = if mark_left { inner.x } else { inner.x + inner.width - mark_width };
                    if let Some(mark)=self.custom_check_mark(164,state,&key,view_response) {
                        if let Some((w,h,r,original))=mark {
                            let w=w.min(mark_width);let h=h.min(inner.height);
                            let mode=self.relative_rules.iter().find(|r|r.kind==185).and_then(|r|r.anchor.parse::<u8>().ok()).unwrap_or(1);
                            let color=self.resolved_progress_tint(156,state,&key,view_response).map_or(original,|tint|tint_pixel(original,tint,mode));
                            f.paint_rect(Rect{x:x+(mark_width-w)*0.5,y:inner.y+(inner.height-h)*0.5,width:w,height:h},r,color);
                        }
                    } else if checked {
                        let side = mark_width.min(inner.height).min(state.dp(24.0));
                        let color=self.resolved_progress_tint(156,state,&key,view_response).unwrap_or(t.primary.opacity(if enabled { 1.0 } else { 0.38 }));
                        f.paint_material_icon(Rect { x: x + (mark_width-side)*0.5, y: inner.y + (inner.height-side)*0.5, width:side, height:side }, MaterialIcon::Check, color);
                    }
                    Rect { x: inner.x + if mark_left { mark_width } else { 0.0 }, width: (inner.width-mark_width).max(0.0), ..inner }
                } else { inner };
                let inner=if matches!(self.kind,Kind::Text|Kind::CheckedText) {
                    let extent=self.compound_extents(state,state.rtl);
                    let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_string()};
                    let text_area=Rect{x:inner.x+extent[0],y:inner.y+extent[1],width:(inner.width-extent[0]-extent[2]).max(0.0),height:(inner.height-extent[1]-extent[3]).max(0.0)};
                    self.paint_compound_drawables(f, inner, text_area, state, &key, view_response);
                    text_area
                } else {inner};
                let text_key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let default_text_color = if state.snackbar_active { t.inverse_on_surface } else { t.text };
                let text_color=self.resolved_state_color(self.text_color_states,self.color,state,&text_key,view_response).unwrap_or(default_text_color);
                let text = self.display_text_for(state.text.get(&text_key).map_or(self.text, String::as_str), state);
                let align = self.resolved_text_alignment(state.rtl).unwrap_or_else(|| match self.content_gravity[0] {
                    0 if state.rtl => 2,
                    2 if state.rtl => 0,
                    3 => 0,
                    4 => 2,
                    _ => self.align,
                });
                let base_text_size = self.scaled_text_size(state);
                let (italic, family) = self.text_style_ext();
                let metrics = f.ui.text_metrics;
                let layout_ext = self.text_layout_ext();
                let base_style = TextStyle { size: base_text_size, weight: if self.text_bold { 700 } else { 400 }, align, letter_spacing:self.letter_spacing_em*base_text_size, italic, family, ..TextStyle::default() };
                let text_size = if let Some(ext) = layout_ext && ext.has_auto_size() {
                    self.resolve_auto_text_size(&text, inner.width, inner.height, base_style, ext, state, metrics)
                } else {
                    base_text_size
                };
                let style = TextStyle { size: text_size, weight: if self.text_bold { 700 } else { 400 }, align, letter_spacing:self.letter_spacing_em*text_size, italic, family, ..TextStyle::default() };
                let line_height = self.resolved_line_height(text_size, state);
                let line_advance = (line_height*self.line_spacing_multiplier+state.dp(self.line_spacing_extra)).max(1.0);
                let links_spec = matches!(self.kind, Kind::Text | Kind::CheckedText)
                    .then(|| self.text_links_spans())
                    .flatten()
                    .filter(|s| s.mask != 0);
                let link_color = links_spec.and_then(|s| s.link_color).unwrap_or(t.primary);
                let underline_h = state.dp(1.0);
                let mut link_counter = 0usize;
                let mut clicked_link: Option<String> = None;
                if self.single_line || (self.max_lines==1 && self.ellipsize_side!=0) {
                    let visible_text=text.replace('\u{00ad}',"");
                    let line = if self.ellipsize_end || self.ellipsize_side!=0 { ellipsize_single_line(&visible_text,inner.width,style,metrics,if self.ellipsize_end {3} else {self.ellipsize_side}) } else { visible_text };
                    let extra=(inner.height-line_height).max(0.0);
                    let y=inner.y+match self.content_gravity[1] {1=>extra*0.5,2=>extra,_=>0.0};
                    let bounds = Rect { y, height: line_height, ..inner };
                    f.within(inner, |f| {
                        if let Some(lspec) = links_spec {
                            paint_text_line_with_links(
                                f,
                                bounds,
                                &line,
                                text_color,
                                link_color,
                                style,
                                metrics,
                                lspec.mask,
                                lspec.clickable,
                                enabled,
                                id,
                                &mut link_counter,
                                &mut clicked_link,
                                underline_h,
                                false,
                            );
                        } else {
                            f.styled_text(bounds, &line, text_color, style);
                        }
                    });
                } else if let Some(ext) = layout_ext {
                    let lines = wrap_text_lines_ext(&text, inner.width, style, metrics, ext);
                    let total = lines.len();
                    let height_limit = if inner.height<=0.0 { 0 } else { ((inner.height-line_height).max(0.0)/line_advance).ceil() as usize + 1 };
                    let limit = if self.max_lines == 0 { height_limit } else { height_limit.min(self.max_lines) };
                    let shown=total.min(limit);
                    let used_height = if shown==0 { 0.0 } else { (line_height+(shown-1) as f32*line_advance).min(inner.height.max(0.0)) };
                    let extra = (inner.height - used_height).max(0.0);
                    let start_y = inner.y + match self.content_gravity[1] { 1 => extra * 0.5, 2 => extra, _ => 0.0 };
                    let justify = ext.justification == TextJustificationMode::InterWord;
                    let rtl = state.rtl;
                    f.within(inner, |f| {
                        for (line_index, line) in lines.iter().enumerate().take(limit) {
                            let bounds = Rect { y: start_y + line_index as f32 * line_advance, height: line_height, ..inner };
                            if self.ellipsize_end && line_index + 1 == limit && total > limit {
                                let source=if line.discretionary_hyphen { line.text.strip_suffix('-').unwrap_or(&line.text) } else { &line.text };
                                let shortened=ellipsize_truncated_line(source,inner.width,style,metrics);
                                if let Some(lspec) = links_spec {
                                    paint_text_line_with_links(
                                        f,
                                        bounds,
                                        &shortened,
                                        text_color,
                                        link_color,
                                        style,
                                        metrics,
                                        lspec.mask,
                                        lspec.clickable,
                                        enabled,
                                        id,
                                        &mut link_counter,
                                        &mut clicked_link,
                                        underline_h,
                                        false,
                                    );
                                } else {
                                    f.styled_text(bounds, &shortened, text_color, style);
                                }
                            } else if justify && !line.is_paragraph_end {
                                if let Some(lspec) = links_spec {
                                    paint_justified_line_with_links(
                                        f,
                                        bounds,
                                        &line.text,
                                        text_color,
                                        link_color,
                                        style,
                                        metrics,
                                        rtl,
                                        lspec.mask,
                                        lspec.clickable,
                                        enabled,
                                        id,
                                        &mut link_counter,
                                        &mut clicked_link,
                                        underline_h,
                                    );
                                } else {
                                    paint_justified_line(f, bounds, &line.text, text_color, style, metrics, rtl);
                                }
                            } else if let Some(lspec) = links_spec {
                                paint_text_line_with_links(
                                    f,
                                    bounds,
                                    &line.text,
                                    text_color,
                                    link_color,
                                    style,
                                    metrics,
                                    lspec.mask,
                                    lspec.clickable,
                                    enabled,
                                    id,
                                    &mut link_counter,
                                    &mut clicked_link,
                                    underline_h,
                                    false,
                                );
                            } else {
                                f.styled_text(bounds, &line.text, text_color, style);
                            }
                        }
                    });
                } else {
                let total = wrapped_text_lines(&text, inner.width, style, metrics, |_| {});
                let height_limit = if inner.height<=0.0 { 0 } else { ((inner.height-line_height).max(0.0)/line_advance).ceil() as usize + 1 };
                let limit = if self.max_lines == 0 { height_limit } else { height_limit.min(self.max_lines) };
                let shown=total.min(limit);
                let used_height = if shown==0 { 0.0 } else { (line_height+(shown-1) as f32*line_advance).min(inner.height.max(0.0)) };
                let extra = (inner.height - used_height).max(0.0);
                let start_y = inner.y + match self.content_gravity[1] { 1 => extra * 0.5, 2 => extra, _ => 0.0 };
                let mut line_index = 0;
                f.within(inner, |f| {
                    wrapped_text_lines(&text, inner.width, style, metrics, |line| {
                        if line_index < limit {
                            let bounds = Rect { y: start_y + line_index as f32 * line_advance, height: line_height, ..inner };
                            if self.ellipsize_end && line_index + 1 == limit && total > limit {
                                let source=if line.discretionary_hyphen { line.text.strip_suffix('-').unwrap_or(&line.text) } else { &line.text };
                                let shortened=ellipsize_truncated_line(source,inner.width,style,metrics);
                                if let Some(lspec) = links_spec {
                                    paint_text_line_with_links(
                                        f,
                                        bounds,
                                        &shortened,
                                        text_color,
                                        link_color,
                                        style,
                                        metrics,
                                        lspec.mask,
                                        lspec.clickable,
                                        enabled,
                                        id,
                                        &mut link_counter,
                                        &mut clicked_link,
                                        underline_h,
                                        false,
                                    );
                                } else {
                                    f.styled_text(bounds, &shortened, text_color, style);
                                }
                            } else if let Some(lspec) = links_spec {
                                paint_text_line_with_links(
                                    f,
                                    bounds,
                                    &line.text,
                                    text_color,
                                    link_color,
                                    style,
                                    metrics,
                                    lspec.mask,
                                    lspec.clickable,
                                    enabled,
                                    id,
                                    &mut link_counter,
                                    &mut clicked_link,
                                    underline_h,
                                    false,
                                );
                            } else { f.styled_text(bounds, &line.text, text_color, style); }
                        }
                        line_index += 1;
                    });
                });
                }
                drop(text);
                if let Some(link) = clicked_link {
                    state.last_clicked_link = Some(link);
                    action = Some(self.id);
                }
            }
            Kind::VideoView => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                f.paint_rect(inner,0.0,Color(0,0,0,255));
                if let Some(frame)=state.video_frames.get(&key) {f.paint_image(inner,frame,ImageFit::Contain,0.0);}
                if self.enabled&&f.enabled&&state.video_ready.contains(&key)&&f.input.pressed&&f.pointer_in(inner) {
                    if let Some((controller,height))=state.video_controllers.get(&key).cloned() {let showing=state.floating_media_controller_is_showing(&controller);let h=height.max(0.0).min(inner.height);let panel=Rect{x:inner.x,y:inner.y+inner.height-h,width:inner.width,height:h};
                        if !(showing&&f.input.pointer.is_some_and(|point|panel.contains(point))) {if showing{state.hide_floating_media_controller(&controller);}else{state.show_floating_media_controller(&controller,f.time(),3000);}action=Some(self.id);}
                    }
                }
            }
            Kind::Image | Kind::QuickContactBadge => {
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let target=if matches!(self.kind,Kind::QuickContactBadge){state.quick_contact_targets.get(&key).cloned()}else{None};
                if matches!(self.kind,Kind::QuickContactBadge) {view_response=f.target(id,b,self.text,self.enabled&&target.is_some(),Role::Button);if f.input.pressed && view_response.pressed && let Some(target)=target.clone(){state.quick_contact_pressed.insert(key.clone(),target);}if view_response.clicked && let Some(target)=target && (state.quick_contact_pressed.get(&key)==Some(&target)||f.input.key.is_some()) {state.quick_contact_clicked.insert(key.clone(),target.clone());state.quick_contact_actions.remove(&key);match target {QuickContactTarget::Uri(uri)=>{state.quick_contact_actions.insert(key.clone(),QuickContactAction::Show(uri));},other=>state.request_quick_contact_lookup(&key,other,true)}action=Some(self.id);}if f.input.released||f.input.cancelled||f.input.blurred {state.quick_contact_pressed.remove(&key);}}
                let dyn_spec = self.dynamic_drawable_src();
                if let Some(ref spec) = dyn_spec
                    && !state.image_sources.contains_key(&key)
                {
                    let tint = self.resolved_image_tint(state, &key, view_response);
                    let mirror = self.is_auto_mirrored() && state.rtl;
                    let clip_box = if self.relative_rules.iter().any(|r| r.kind == 188) { inner } else { b };
                    paint_dynamic_src_drawable(
                        f,
                        state,
                        inner,
                        clip_box,
                        &key,
                        spec,
                        self.image_scale,
                        tint,
                        self.image_tint_mode,
                        Color(255, 255, 255, self.image_alpha),
                        mirror,
                    );
                    if matches!(self.kind,Kind::QuickContactBadge)&&view_response.enabled {paint_quick_contact_overlay(f,b,view_response,state.rtl);}
                } else {
                let anim_spec = self.animation_list_spec();
                let anim_vec_spec = if anim_spec.is_none() { self.animated_vector_spec(true) } else { None };
                let anim_sel_frame = if anim_spec.is_none()
                    && anim_vec_spec.is_none()
                    && !state.image_sources.contains_key(&key)
                    && self.has_animated_selector(true)
                {
                    self.sample_animated_selector(f, b, state, &key, view_response, true)
                } else {
                    None
                };
                let (source, stretch_spec) = if let Some(ref spec) = anim_spec
                    && !state.image_sources.contains_key(&key)
                {
                    let (frame_key, next_wake) = select_animation_list_frame(spec, f.time(), f.reduced_motion());
                    if let Some(wake) = next_wake {
                        let vis_rect = b.intersect(f.clip);
                        if vis_rect.width > 0.0 && vis_rect.height > 0.0 {
                            f.wake_at(wake);
                        }
                    }
                    (frame_key, None)
                } else if let Some(ref spec) = anim_vec_spec
                    && !state.image_sources.contains_key(&key)
                {
                    let (frame_key, next_wake) = select_animated_vector_frame(spec, f.time(), f.reduced_motion());
                    if let Some(wake) = next_wake {
                        let vis_rect = b.intersect(f.clip);
                        if vis_rect.width > 0.0 && vis_rect.height > 0.0 {
                            f.wake_at(wake);
                        }
                    }
                    (frame_key, None)
                } else if let Some(frame_key) = anim_sel_frame {
                    (frame_key, None)
                } else {
                    let (s, st, _) = self.resolved_image_src(state, &key, view_response);
                    (s, st)
                };
                let image = if source.is_empty() { None } else { state.images.get(source).cloned() };
                let density=state.image_scales.get(source).copied().unwrap_or(1.0)*state.effective_density();
                let mirror = self.is_auto_mirrored() && state.rtl;
                if let Some(image) = image {
                    let tint=self.resolved_image_tint(state,&key,view_response);
                    if let Some(ref bands) = stretch_spec {
                        if inner.width > 0.0 && inner.height > 0.0 {
                            if mirror {
                                f.push_transform(
                                    [inner.x + inner.width * 0.5, inner.y + inner.height * 0.5],
                                    [0.0, 0.0],
                                    [-1.0, 1.0],
                                    0.0,
                                );
                            }
                            paint_nine_patch_image(f, state, inner, &image, bands, tint, self.image_tint_mode, self.image_alpha, density);
                            if mirror {
                                f.pop_transform();
                            }
                        }
                    } else if let Some((bounds, uv)) = image_geometry(inner, if self.relative_rules.iter().any(|r|r.kind==188){inner}else{b}, &image, self.image_scale, density, state.image_matrices.get(&key).copied())
                        && bounds.intersect(f.clip).width > 0.0 && bounds.intersect(f.clip).height > 0.0
                    {
                        let image=tint_image(state,&image,tint,self.image_tint_mode);
                        if mirror {
                            f.push_transform(
                                [bounds.x + bounds.width * 0.5, bounds.y + bounds.height * 0.5],
                                [0.0, 0.0],
                                [-1.0, 1.0],
                                0.0,
                            );
                        }
                        f.commands.push(DrawCommand::Image { bounds, uv, radius: 0.0, color: Color(255,255,255,self.image_alpha), image });
                        if mirror {
                            f.pop_transform();
                        }
                    }
                }
                if matches!(self.kind,Kind::QuickContactBadge)&&view_response.enabled {paint_quick_contact_overlay(f,b,view_response,state.rtl);}
                }
            }
            Kind::AnalogClock => {
                let side=inner.width.min(inner.height).max(0.0);
                let dial=Rect { x:inner.x+(inner.width-side)*0.5,y:inner.y+(inner.height-side)*0.5,width:side,height:side };
                let index=(crate::beercss_shapes::SHAPES.len()+crate::icons::material_navigation::ICONS.len()+crate::icons::material_symbols::ICONS.len()+1) as u8;
                f.commands.push(DrawCommand::Shape { bounds:dial,index,color:t.surface_variant,rotation:0.0 });
                if let Some(clock)=state.wall_clock.filter(|clock|clock.unix_seconds.is_finite()) {
                    let local=(clock.unix_seconds+i64::from(clock.offset_seconds) as f64).rem_euclid(86400.0);
                    let minutes=(local/60.0) as f32;let hours=minutes/60.0;
                    f.commands.push(DrawCommand::Shape { bounds:dial,index:index+1,color:t.primary,rotation:(hours%12.0)*std::f32::consts::TAU/12.0 });
                    f.commands.push(DrawCommand::Shape { bounds:dial,index:index+2,color:t.on_tertiary_container,rotation:(minutes%60.0)*std::f32::consts::TAU/60.0 });
                    let visible=b.intersect(f.clip);
                    if visible.width>0.0 && visible.height>0.0 { f.wake_at(f.time()+(60.0-clock.unix_seconds.rem_euclid(60.0)).max(0.001)); }
                }
            }
            Kind::ToggleButton => {
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let value = state.checked.entry(key.clone()).or_insert(self.checked);
                let response = f.target(id, b, self.text, self.enabled, Role::ToggleButton);
                view_response = response;
                if response.clicked { *value = !*value; action = Some(self.id); }
                let checked=*value;
                let fill = if checked { t.secondary } else { t.control };
                let r4 = state.dp(4.0);
                f.paint_rect(b, r4, fill.opacity(if response.enabled { 1.0 } else { 0.38 }));
                self.paint_state_layer(f, response, r4, corner_radii, t.text);
                let label = self.display_text_for(if checked { self.text_on } else { self.text_off }, state);
                let text_color=self.resolved_state_color(self.text_color_states,self.color,state,&key,response).unwrap_or_else(|| t.text.opacity(if response.enabled { 1.0 } else { 0.38 }));
                let text_size = self.scaled_text_size(state);
                let (italic, family) = self.text_style_ext();
                f.styled_text(inner, &label, text_color, TextStyle { size:text_size, align:1, weight:if self.text_bold {700}else{400},letter_spacing:self.letter_spacing_em*text_size,italic,family,..TextStyle::default() });
                let indicator = Rect { x: b.x + state.dp(8.0), y: b.y + b.height - state.dp(5.0), width: (b.width - state.dp(16.0)).max(0.0), height: state.dp(2.0) };
                let disabled_alpha=self.relative_rules.iter().find(|rule|rule.kind==162).and_then(|rule|rule.anchor.parse::<f32>().ok()).unwrap_or(0.5).clamp(0.0,1.0);
                let color=if checked {t.primary}else{t.outline.opacity(0.38)};
                f.paint_rect(indicator, 0.0, color.opacity(if response.enabled {1.0}else{disabled_alpha}));
            }
            Kind::RatingBar => {
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let mut tint_response=view_response;
                tint_response.pressed |= enabled && !self.indicator && f.input.pressed && f.pointer_in(inner);
                let colors=[self.resolved_progress_tint(149,state,&key,tint_response),self.resolved_progress_tint(150,state,&key,tint_response),self.resolved_progress_tint(151,state,&key,tint_response)];
                if state.theme_style == AndroidTheme::Bhoo {
                    let value = state.ratings.entry(key).or_insert(self.rating);
                    f.within(inner, |f| {
                        if f.rating_fractional_tinted(id, value, self.num_stars, self.step_size, self.enabled, self.indicator, state.rtl && self.mirror_for_rtl, colors).changed { action = Some(self.id); }
                    });
                } else {
                    let rtl = state.rtl && self.mirror_for_rtl;
                    let row_h = inner.height.min(state.dp(40.0));
                    let stars = self.num_stars.clamp(1, 20);
                    let star_w = (inner.width / stars as f32).min(state.dp(48.0));
                    let step = if self.step_size.is_finite() && self.step_size > 0.0 { self.step_size.min(stars as f32) } else { 0.5 };
                    let mut val = *state.ratings.entry(key.clone()).or_insert(self.rating);
                    val = if val.is_finite() { val.clamp(0.0, stars as f32) } else { 0.0 };
                    let old = val;
                    let row = Rect {
                        x: inner.x + if rtl { (inner.width - star_w * stars as f32).max(0.0) } else { 0.0 },
                        y: inner.y + (inner.height - row_h) * 0.5,
                        width: star_w * stars as f32,
                        height: row_h,
                    };
                    f.within(inner, |f| {
                        let response = f.interact(id, row, self.enabled && !self.indicator);
                        if response.enabled && f.ui.active == Some(id) && f.ui.capture == Capture::Pointer
                            && let Some(point) = f.input.pointer
                        {
                            let distance = if rtl { row.x + row.width - point[0] } else { point[0] - row.x };
                            let raw = (distance / star_w.max(1.0)).clamp(0.0, stars as f32);
                            val = ((raw / step).ceil() * step).clamp(0.0, stars as f32);
                        }
                        if let Some(key) = f.key_for(response) {
                            val = match key {
                                Key::Left if rtl => (val + step).min(stars as f32),
                                Key::Right if rtl => (val - step).max(0.0),
                                Key::Left | Key::Down => (val - step).max(0.0),
                                Key::Right | Key::Up => (val + step).min(stars as f32),
                                Key::Home => 0.0,
                                Key::End => stars as f32,
                                _ => val,
                            };
                        }
                        if (val - old).abs() > 0.0001 {
                            action = Some(self.id);
                        }
                        let foreground = colors[0].unwrap_or(t.primary.opacity(if enabled { 1.0 } else { 0.38 }));
                        let background = colors[1].unwrap_or(t.on_surface_variant.opacity(if enabled { 0.38 } else { 0.12 }));
                        let index = (crate::beercss_shapes::SHAPES.len() + crate::icons::material_navigation::ICONS.len() + crate::icons::material_symbols::ICONS.len()) as u8;
                        for i in 0..stars {
                            let physical_index = if rtl { stars - 1 - i } else { i };
                            let star = Rect { x: row.x + physical_index as f32 * star_w, width: star_w, ..row };
                            let icon_size = star.width.min(star.height);
                            let icon = Rect { x: star.x + (star.width - icon_size) * 0.5, y: star.y + (star.height - icon_size) * 0.5, width: icon_size, height: icon_size };
                            f.commands.push(DrawCommand::Shape { bounds: icon, index, color: background, rotation: 0.0 });
                            let fill = (val - i as f32).clamp(0.0, 1.0);
                            if fill > 0.0 && let Some(secondary) = colors[2] {
                                f.commands.push(DrawCommand::Shape { bounds: icon, index, color: secondary, rotation: 0.0 });
                            }
                            if fill > 0.0 {
                                let fill_width = star.width * fill;
                                let fill_x = if rtl { star.x + star.width - fill_width } else { star.x };
                                f.commands.push(DrawCommand::PushClip(Rect { x: fill_x, width: fill_width, ..star }.intersect(f.clip)));
                                f.commands.push(DrawCommand::Shape { bounds: icon, index, color: foreground, rotation: 0.0 });
                                f.commands.push(DrawCommand::PopClip);
                            }
                        }
                        f.focus_ring(response);
                    });
                    state.ratings.insert(key, val);
                }
            }
            Kind::ProgressBar => {
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let value = *state.progress.entry(key.clone()).or_insert(self.progress.clamp(self.min, self.max));
                let secondary = *state.secondary_progress.entry(key.clone()).or_insert(self.secondary_progress.clamp(self.min, self.max));
                let buffered = if self.max > self.min { (secondary - self.min).clamp(0.0, self.max - self.min) / (self.max - self.min) } else { 0.0 };
                let fraction = if self.max > self.min { (value - self.min).clamp(0.0, self.max - self.min) / (self.max - self.min) } else { 0.0 };
                let progress = if self.indeterminate { None } else { Some(fraction) };
                let mut colors=crate::progress::ClassicProgressColors::from_theme(t);
                let m3 = matches!(state.theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark);
                if m3 {
                    colors.background = t.secondary_container;
                }
                let explicit_primary = self.resolved_progress_tint(149,state,&key,view_response);
                let explicit_background = self.resolved_progress_tint(150,state,&key,view_response);
                colors.primary=explicit_primary.unwrap_or(colors.primary);
                colors.background=explicit_background.unwrap_or(colors.background);
                colors.secondary=self.resolved_progress_tint(151,state,&key,view_response).unwrap_or(colors.secondary);
                colors.indeterminate=self.resolved_progress_tint(152,state,&key,view_response).unwrap_or(colors.indeterminate);
                if let Some(contained) = self.loading_indicator_contained() {
                    let size = inner.width.min(inner.height).max(1.0);
                    let target = Rect {
                        x: inner.x + (inner.width - size) * 0.5,
                        y: inner.y + (inner.height - size) * 0.5,
                        width: size,
                        height: size,
                    };
                    let mut ls = state.loading_states.remove(&id).unwrap_or_default();
                    let cmd_start = f.commands.len();
                    let sem_start = f.ui.semantics.len();
                    let prev_bounds = f.bounds;
                    let prev_cursor = f.cursor;
                    let prev_clip = f.clip;
                    f.bounds = Rect { x: target.x, y: target.y, width: 48.0, height: 48.0 };
                    f.cursor = target.y;
                    f.clip = Rect { x: target.x, y: target.y, width: 48.0, height: 48.0 }.intersect(prev_clip);
                    if f.clip.width <= 0.0 || f.clip.height <= 0.0 {
                        f.clip = prev_clip;
                    }
                    f.loading_indicator(id, self.text, &mut ls, None, contained);
                    f.bounds = prev_bounds;
                    f.cursor = prev_cursor;
                    f.clip = prev_clip;
                    state.loading_states.insert(id, ls);
                    for cmd in &mut f.commands[cmd_start..] {
                        match cmd {
                            DrawCommand::RoundedRect { bounds, radius, .. } => {
                                *bounds = target;
                                *radius = size * 0.5;
                            }
                            DrawCommand::Loading { bounds, color, .. } => {
                                *bounds = target;
                                if let Some(tint) = explicit_primary {
                                    *color = tint;
                                }
                            }
                            _ => {}
                        }
                    }
                    for node in &mut f.ui.semantics[sem_start..] {
                        node.bounds = target;
                    }
                } else if let Some(wavy) = self.wavy_progress_spec() {
                    let d = state.effective_density();
                    let is_wavy = wavy.wavelength_dp > 0.0 && wavy.amplitude_dp > 0.0;
                    let saved_primary = f.ui.theme.primary;
                    let saved_secondary_container = f.ui.theme.secondary_container;
                    if let Some(p) = explicit_primary {
                        f.ui.theme.primary = p;
                    }
                    if let Some(bg) = explicit_background {
                        f.ui.theme.secondary_container = bg;
                    }
                    if self.horizontal {
                        let opt = crate::progress::WavyProgressOptions {
                            wavelength: if is_wavy { wavy.wavelength_dp * d } else { 40.0 * d },
                            amplitude: if is_wavy { wavy.amplitude_dp * d } else { 0.0 },
                            wave_speed: if is_wavy { wavy.wave_speed_dp * d } else { 0.0 },
                            track_thickness: 4.0 * d,
                            stop_indicator_size: wavy.stop_size_dp * d,
                            track_gap: wavy.track_gap_dp * d,
                        };
                        let alloc_h = (opt.track_thickness + opt.amplitude * 2.0).max(12.0);
                        let slot = Rect {
                            x: inner.x,
                            y: inner.y + (inner.height - alloc_h) * 0.5,
                            width: inner.width,
                            height: alloc_h,
                        };
                        let rtl = state.rtl && self.mirror_for_rtl;
                        let cmd_start = f.commands.len();
                        f.within(slot, |f| {
                            f.wavy_linear_progress(id, self.text, progress, opt);
                        });
                        if rtl {
                            let axis = inner.x + inner.width * 0.5;
                            for cmd in &mut f.commands[cmd_start..] {
                                match cmd {
                                    DrawCommand::RoundedRect { bounds, .. } => {
                                        bounds.x = 2.0 * axis - (bounds.x + bounds.width);
                                    }
                                    DrawCommand::Line { from, to, .. } => {
                                        from[0] = 2.0 * axis - from[0];
                                        to[0] = 2.0 * axis - to[0];
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        let opt_unscaled = crate::progress::WavyProgressOptions {
                            wavelength: if is_wavy { wavy.wavelength_dp } else { 16.0 },
                            amplitude: if is_wavy { wavy.amplitude_dp } else { 0.0 },
                            wave_speed: if is_wavy { wavy.wave_speed_dp } else { 0.0 },
                            track_thickness: 4.0,
                            stop_indicator_size: wavy.stop_size_dp,
                            track_gap: wavy.track_gap_dp,
                        };
                        let alloc_size = 48.0_f32.max(opt_unscaled.track_thickness + opt_unscaled.amplitude * 2.0 + 16.0);
                        let target_size = inner.width.min(inner.height).max(1.0);
                        let target = Rect {
                            x: inner.x + (inner.width - target_size) * 0.5,
                            y: inner.y + (inner.height - target_size) * 0.5,
                            width: target_size,
                            height: target_size,
                        };
                        let scale = target_size / alloc_size;
                        let cmd_start = f.commands.len();
                        let sem_start = f.ui.semantics.len();
                        let prev_bounds = f.bounds;
                        let prev_cursor = f.cursor;
                        f.bounds = Rect { x: target.x, y: target.y, width: alloc_size, height: alloc_size };
                        f.cursor = target.y;
                        f.wavy_circular_progress(id, self.text, progress, opt_unscaled);
                        f.bounds = prev_bounds;
                        f.cursor = prev_cursor;
                        if (scale - 1.0).abs() > 1e-4 {
                            let map_pt = |p: [f32; 2]| [
                                target.x + (p[0] - target.x) * scale,
                                target.y + (p[1] - target.y) * scale,
                            ];
                            for cmd in &mut f.commands[cmd_start..] {
                                match cmd {
                                    DrawCommand::Arc { bounds, stroke, .. } => {
                                        bounds.x = target.x + (bounds.x - target.x) * scale;
                                        bounds.y = target.y + (bounds.y - target.y) * scale;
                                        bounds.width *= scale;
                                        bounds.height *= scale;
                                        *stroke *= scale;
                                    }
                                    DrawCommand::Line { from, to, width, .. } => {
                                        *from = map_pt(*from);
                                        *to = map_pt(*to);
                                        *width *= scale;
                                    }
                                    _ => {}
                                }
                            }
                            for node in &mut f.ui.semantics[sem_start..] {
                                node.bounds = target;
                            }
                        }
                    }
                    f.ui.theme.primary = saved_primary;
                    f.ui.theme.secondary_container = saved_secondary_container;
                } else if state.theme_style == AndroidTheme::Bhoo {
                    f.within(inner, |f| {
                        if self.horizontal { f.progress_classic_tinted(id, self.text, progress, Some(buffered), state.rtl && self.mirror_for_rtl, colors); }
                        else { f.progress_circular_classic_tinted(id, self.text, progress, inner.width.min(inner.height).clamp(12.0, 48.0), colors); }
                    });
                } else {
                    let rtl = state.rtl && self.mirror_for_rtl;
                    let d = state.effective_density();
                    f.within(inner, |f| {
                        if self.horizontal {
                            use crate::timing::{Easing, TweenSpec};
                            let track_h = (4.0 * d).min(inner.height.max(1.0));
                            let track_r = if m3 { track_h * 0.5 } else { 0.0 };
                            let bar = Rect { x: inner.x, y: inner.y + (inner.height - track_h) * 0.5, width: inner.width, height: track_h };
                            let val = progress.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0));
                            let segments = if let Some(v) = val {
                                let v = f.ui.motion.tween(id, 3, v, v, TweenSpec::new(0.250, Easing::Cubic([0.4, 0.0, 0.6, 1.0])), TweenSpec::new(0.250, Easing::Cubic([0.4, 0.0, 0.6, 1.0])));
                                [[0.0, v], [0.0, 0.0]]
                            } else if f.continuous(bar) {
                                crate::progress::classic_linear_segments(f.ui.motion.elapsed(id, 31))
                            } else {
                                [[0.0, 0.4], [0.0, 0.0]]
                            };
                            f.paint_rect(bar, track_r, colors.background);
                            if val.is_some() && buffered.is_finite() && buffered > 0.0 {
                                let bw = bar.width * buffered.clamp(0.0, 1.0);
                                f.paint_rect(Rect { x: if rtl { bar.x + bar.width - bw } else { bar.x }, width: bw, ..bar }, track_r, colors.secondary);
                            }
                            for [tail, head] in segments {
                                if head > tail {
                                    f.paint_rect(
                                        Rect { x: bar.x + bar.width * if rtl { 1.0 - head } else { tail }, width: bar.width * (head - tail), ..bar },
                                        track_r,
                                        if val.is_some() { colors.primary } else { colors.indeterminate },
                                    );
                                }
                            }
                            f.semantic(Response { id, bounds: bar, enabled: f.enabled, ..Response::default() }, Role::Progress, self.text, false, None, val, val.map(|_| [0.0, 1.0]));
                        } else {
                            use crate::timing::{Easing, TweenSpec};
                            let diameter = inner.width.min(inner.height).clamp(12.0 * d, 48.0 * d);
                            let box_b = Rect { x: inner.x + (inner.width - diameter) * 0.5, y: inner.y + (inner.height - diameter) * 0.5, width: diameter, height: diameter };
                            let val = progress.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0));
                            let (start_a, sweep) = if let Some(v) = val {
                                let a = f.ui.motion.tween(id, 3, v, v, TweenSpec::new(0.5, Easing::StandardDecelerate), TweenSpec::new(0.5, Easing::StandardDecelerate));
                                (-std::f32::consts::FRAC_PI_2, a * std::f32::consts::TAU)
                            } else if f.continuous(box_b) {
                                crate::progress::classic_circular_sample(f.ui.motion.elapsed(id, 31))
                            } else {
                                (-std::f32::consts::FRAC_PI_2, std::f32::consts::TAU * 0.4)
                            };
                            let stroke = if m3 { (4.0 * d).min(diameter * 0.25) } else { diameter / 10.0 };
                            if m3 {
                                let gap = (8.0 * d / (std::f32::consts::PI * diameter.max(1.0)) * std::f32::consts::TAU).min(sweep);
                                let track_sweep = (std::f32::consts::TAU - sweep - 2.0 * gap).max(0.0);
                                if track_sweep > 0.001 {
                                    f.commands.push(DrawCommand::Arc { bounds: box_b, start: start_a + sweep + gap, sweep: track_sweep, stroke, color: colors.background, rounded: true });
                                }
                                f.commands.push(DrawCommand::Arc { bounds: box_b, start: start_a, sweep, stroke, color: if val.is_some() { colors.primary } else { colors.indeterminate }, rounded: true });
                            } else {
                                let inset = (stroke * 0.5).max(0.0);
                                f.commands.push(DrawCommand::Arc { bounds: box_b.inset(inset), start: start_a, sweep, stroke, color: if val.is_some() { colors.primary } else { colors.indeterminate }, rounded: false });
                            }
                            f.semantic(Response { id, bounds: box_b, enabled: f.enabled, ..Response::default() }, Role::Progress, self.text, false, None, val, val.map(|_| [0.0, 1.0]));
                        }
                    });
                }
            }
            Kind::SeekBar => {
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let mut tint_response=view_response;
                tint_response.pressed |= enabled && f.input.pressed && f.pointer_in(inner);
                let static_tick=self.relative_rules.iter().find(|rule|rule.kind==155).and_then(|rule|{
                    let mut p=rule.anchor.split(',');
                    let d=state.effective_density();
                    let w=p.next()?.parse::<f32>().ok()?*d;
                    let h=p.next()?.parse::<f32>().ok()?*d;
                    let r=p.next()?.parse::<f32>().ok()?;
                    let r=if r<0.0 {r} else {r*d};
                    Some((w,h,r,Color(p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?)))
                });
                let tint_mode=|kind|self.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|r.anchor.parse::<u8>().ok()).unwrap_or(1);
                let anim_tick = self.compound_anim_spec(CompoundAnimSlot::Tick).and_then(|spec| {
                    let (frame_key, next_wake) = select_animation_list_frame(&spec, f.time(), f.reduced_motion());
                    let raw_img = state.images.get(frame_key)?.clone();
                    let tick_tint = self.resolved_progress_tint(154, state, &key, tint_response);
                    let img = tint_image(state, &raw_img, tick_tint, tint_mode(187));
                    let scale = state.image_scales.get(frame_key).copied().unwrap_or(1.0) * state.effective_density();
                    let (tw, th) = (img.width() as f32 * scale, img.height() as f32 * scale);
                    Some((img, tw, th, next_wake))
                });
                let tick=if anim_tick.is_some() { None } else {
                    self.custom_check_mark(167,state,&key,tint_response).unwrap_or(static_tick)
                        .map(|(w,h,r,color)|(w,h,r,self.resolved_progress_tint(154,state,&key,tint_response).map_or(color,|tint|tint_pixel(color,tint,tint_mode(187)))))
                };
                let thumb_tint=self.resolved_progress_tint(153,state,&key,tint_response);
                let custom_thumb=self.custom_check_mark(166,state,&key,tint_response).map(|thumb|thumb.map(|(w,h,r,color)|
                    (w,h,r,thumb_tint.map_or(color,|tint|tint_pixel(color,tint,tint_mode(186))))));
                let default_track_bg = if matches!(state.theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark) && enabled {
                    Some(t.secondary_container)
                } else {
                    None
                };
                let colors=crate::systems::material::SliderTintColors {
                    primary:self.resolved_progress_tint(149,state,&key,tint_response),
                    background:self.resolved_progress_tint(150,state,&key,tint_response).or(default_track_bg),
                    secondary:self.resolved_progress_tint(151,state,&key,tint_response),
                    thumb:if custom_thumb.is_some(){None}else{thumb_tint},
                    tick,
                    split_track:self.relative_rules.iter().any(|rule|rule.kind==163),
                    custom_thumb,
                };
                let paint_anim_ticks = |f: &mut Frame<'_>, start_x: f32, span_w: f32, cy: f32, count: usize, enabled_alpha: f32, mirror: bool| {
                    if let Some((ref img, tw, th, next_wake)) = anim_tick && count > 0 {
                        if let Some(wake) = next_wake {
                            let vis = inner.intersect(f.clip);
                            if vis.width > 0.0 && vis.height > 0.0 {
                                f.wake_at(wake);
                            }
                        }
                        for i in 0..=count {
                            let center = start_x + span_w * i as f32 / count as f32;
                            let rect = Rect { x: center - tw * 0.5, y: cy - th * 0.5, width: tw, height: th };
                            if mirror {
                                f.push_transform([center, cy], [0.0, 0.0], [-1.0, 1.0], 0.0);
                            }
                            f.commands.push(DrawCommand::Image {
                                bounds: rect,
                                uv: [0.0, 0.0, 1.0, 1.0],
                                radius: 0.0,
                                color: Color(255, 255, 255, 255).opacity(enabled_alpha),
                                image: img.clone(),
                            });
                            if mirror {
                                f.pop_transform();
                            }
                        }
                    }
                };
                let secondary = *state.secondary_progress.entry(key.clone()).or_insert(self.secondary_progress.clamp(self.min, self.max));
                let fraction = if self.max > self.min { (secondary-self.min).clamp(0.0,self.max-self.min)/(self.max-self.min) } else { 0.0 };
                let max = if self.max > self.min { self.max } else { self.min + 2.0 };
                let slider_opt = self.slider_options();
                let sl_step = slider_opt.map_or(0.0, |o| o.step_size);
                let sl_min_sep = slider_opt.map_or(0.0, |o| o.min_separation);
                let sl_expressive = slider_opt.is_some_and(|o| o.expressive);
                let sl_label_beh = slider_opt.map(|o| o.label_behavior);
                if let Some(init_range) = self.range_slider_initial() {
                    let rtl = state.rtl;
                    let d = state.effective_density();
                    let min = self.min;
                    let range_max = if self.max > self.min { self.max } else { self.min + 100.0 };
                    let slider_enabled = self.enabled && self.max > self.min;
                    let min_sep = sl_min_sep.max(0.0).min(range_max - min);
                    let mut vals = *state
                        .range_values
                        .entry(key.clone())
                        .or_insert_with(|| [init_range[0].clamp(min, range_max), init_range[1].clamp(min, range_max)]);
                    vals[0] = if vals[0].is_finite() { vals[0].clamp(min, range_max) } else { min };
                    vals[1] = if vals[1].is_finite() { vals[1].clamp(min, range_max) } else { range_max };
                    if vals[0] > vals[1] {
                        vals.swap(0, 1);
                    }
                    if vals[1] - vals[0] < min_sep {
                        vals[1] = (vals[0] + min_sep).min(range_max);
                        if vals[1] - vals[0] < min_sep {
                            vals[0] = (vals[1] - min_sep).max(min);
                        }
                    }
                    let old = vals;
                    let pad = if sl_expressive { 10.0 * d } else { 11.0 * d };
                    let start_x = inner.x + pad;
                    let span_w = (inner.width - pad * 2.0).max(1.0);
                    let val_to_x = |v: f32| {
                        let frac = ((v - min) / (range_max - min)).clamp(0.0, 1.0);
                        start_x + span_w * if rtl { 1.0 - frac } else { frac }
                    };
                    let snap_val = |raw: f32| {
                        if sl_step > 0.0 {
                            (min + ((raw - min) / sl_step).round() * sl_step).clamp(min, range_max)
                        } else if slider_opt.is_none() && (range_max - min) >= 2.0 {
                            (min + (raw - min).round()).clamp(min, range_max)
                        } else {
                            raw.clamp(min, range_max)
                        }
                    };
                    let x_to_val = |px: f32| {
                        let frac = ((px - start_x) / span_w).clamp(0.0, 1.0);
                        let raw = min + (if rtl { 1.0 - frac } else { frac }) * (range_max - min);
                        snap_val(raw)
                    };
                    let clamp_thumb = |idx: usize, cur: f32, other: f32| {
                        if idx == 0 {
                            let upper = (other - min_sep).max(min);
                            if sl_step > 0.0 && cur > upper {
                                (min + ((upper - min) / sl_step).floor() * sl_step).clamp(min, upper)
                            } else {
                                cur.clamp(min, upper)
                            }
                        } else {
                            let lower = (other + min_sep).min(range_max);
                            if sl_step > 0.0 && cur < lower {
                                (min + ((lower - min) / sl_step).ceil() * sl_step).clamp(lower, range_max)
                            } else {
                                cur.clamp(lower, range_max)
                            }
                        }
                    };
                    f.within(inner, |f| {
                        let r = f.interact(id, inner, slider_enabled);
                        if r.enabled && !f.input.cancelled && !f.input.blurred {
                            if f.ui.active == Some(id)
                                && f.ui.capture == Capture::Pointer
                                && let Some(p) = f.input.pointer
                            {
                                let active_thumb = if f.input.pressed || !state.range_slider_active_thumb.contains_key(&id) {
                                    let x0 = val_to_x(vals[0]);
                                    let x1 = val_to_x(vals[1]);
                                    let d0 = (p[0] - x0).abs();
                                    let d1 = (p[0] - x1).abs();
                                    let idx = if (vals[0] - vals[1]).abs() < f32::EPSILON {
                                        let dir = if rtl { -1.0 } else { 1.0 };
                                        if (p[0] - x0) * dir >= 0.0 { 1 } else { 0 }
                                    } else if d0 <= d1 {
                                        0
                                    } else {
                                        1
                                    };
                                    state.range_slider_active_thumb.insert(id, idx);
                                    idx
                                } else {
                                    state.range_slider_active_thumb.get(&id).copied().unwrap_or(0)
                                };
                                let next_v = x_to_val(p[0]);
                                if active_thumb == 0 {
                                    vals[0] = clamp_thumb(0, next_v, vals[1]);
                                } else {
                                    vals[1] = clamp_thumb(1, next_v, vals[0]);
                                }
                            } else if f.ui.active != Some(id) {
                                state.range_slider_active_thumb.remove(&id);
                            }
                            if let Some(key_ev) = f.key_for(r) {
                                let step = if sl_step > 0.0 {
                                    sl_step
                                } else if (range_max - min) >= 2.0 {
                                    1.0
                                } else {
                                    (range_max - min) * 0.05
                                };
                                let idx = state.range_slider_active_thumb.get(&id).copied().unwrap_or(1);
                                let mut cur = vals[idx];
                                match key_ev {
                                    Key::Left if rtl => cur += step,
                                    Key::Right if rtl => cur -= step,
                                    Key::Left | Key::Down => cur -= step,
                                    Key::Right | Key::Up => cur += step,
                                    Key::Home => cur = if idx == 0 { min } else { vals[0] + min_sep },
                                    Key::End => cur = if idx == 0 { vals[1] - min_sep } else { range_max },
                                    _ => {}
                                }
                                let snapped = snap_val(cur);
                                if idx == 0 {
                                    vals[0] = clamp_thumb(0, snapped, vals[1]);
                                } else {
                                    vals[1] = clamp_thumb(1, snapped, vals[0]);
                                }
                            }
                        } else if f.ui.active != Some(id) {
                            state.range_slider_active_thumb.remove(&id);
                        }
                        if vals != old {
                            action = Some(self.id);
                        }
                        let cy = inner.y + inner.height * 0.5;
                        let x0 = val_to_x(vals[0]);
                        let x1 = val_to_x(vals[1]);
                        let active_left = x0.min(x1);
                        let active_right = x0.max(x1);
                        let inactive_color = colors.background.unwrap_or(if r.enabled {
                            if sl_expressive { t.secondary_container } else { t.surface_variant }
                        } else {
                            t.text.opacity(0.12)
                        });
                        let active_color = colors.primary.unwrap_or(if r.enabled {
                            t.primary
                        } else {
                            t.text.opacity(0.38)
                        });
                        let active_idx = state.range_slider_active_thumb.get(&id).copied();
                        if sl_expressive {
                            let track_x = inner.x + 8.0 * d;
                            let track_w = (inner.width - 16.0 * d).max(1.0);
                            let track_right = track_x + track_w;
                            let track_h = 16.0 * d;
                            let track_y = cy - track_h * 0.5;
                            let track_r = 8.0 * d;
                            let gap_half = 8.0 * d; // 2dp half-handle + 6dp track gap
                            let active_stop_color = if r.enabled { t.on_primary } else { t.inverse_on_surface.opacity(0.38) };
                            let inactive_stop_color = if r.enabled { t.on_secondary_container } else { t.text.opacity(0.38) };
                            let left_end = (active_left - gap_half).clamp(track_x, track_right);
                            if left_end > track_x {
                                let w = left_end - track_x;
                                f.paint_rect(Rect { x: track_x, y: track_y, width: w, height: track_h }, track_r.min(w * 0.5), inactive_color);
                            }
                            let mid_start = (active_left + gap_half).clamp(track_x, track_right);
                            let mid_end = (active_right - gap_half).clamp(track_x, track_right);
                            if mid_end > mid_start {
                                let w = mid_end - mid_start;
                                f.paint_rect(Rect { x: mid_start, y: track_y, width: w, height: track_h }, track_r.min(w * 0.5), active_color);
                            }
                            let right_start = (active_right + gap_half).clamp(track_x, track_right);
                            if track_right > right_start {
                                let w = track_right - right_start;
                                f.paint_rect(Rect { x: right_start, y: track_y, width: w, height: track_h }, track_r.min(w * 0.5), inactive_color);
                            }
                            let dot_d = 4.0 * d;
                            let dot_r = 2.0 * d;
                            if sl_step > 0.0 {
                                let steps = ((range_max - min) / sl_step).round().clamp(1.0, 128.0) as usize;
                                for i in 0..=steps {
                                    let tx = start_x + span_w * (i as f32 / steps as f32);
                                    if (tx - x0).abs() < gap_half || (tx - x1).abs() < gap_half {
                                        continue;
                                    }
                                    let in_active = tx > active_left && tx < active_right;
                                    f.paint_rect(
                                        Rect { x: tx - dot_r, y: cy - dot_r, width: dot_d, height: dot_d },
                                        dot_r,
                                        if in_active { active_stop_color } else { inactive_stop_color },
                                    );
                                }
                            } else {
                                if left_end >= track_x + 8.0 * d {
                                    f.paint_rect(Rect { x: track_x + 4.0 * d, y: cy - dot_r, width: dot_d, height: dot_d }, dot_r, inactive_stop_color);
                                }
                                if right_start <= track_right - 8.0 * d {
                                    f.paint_rect(Rect { x: track_right - 8.0 * d, y: cy - dot_r, width: dot_d, height: dot_d }, dot_r, inactive_stop_color);
                                }
                            }
                            let handle_w = 4.0 * d;
                            let handle_h = 44.0 * d;
                            let handle_color = colors.thumb.unwrap_or(active_color);
                            for tx in [x0, x1] {
                                f.paint_rect(
                                    Rect { x: tx - handle_w * 0.5, y: cy - handle_h * 0.5, width: handle_w, height: handle_h },
                                    handle_w * 0.5,
                                    handle_color,
                                );
                            }
                            let anchor_y = cy - handle_h * 0.5;
                            let wb = matches!(sl_label_beh, Some(SliderLabelBehavior::WithinBounds)).then_some(inner);
                            match sl_label_beh {
                                Some(SliderLabelBehavior::Visible) if r.enabled => {
                                    paint_xml_slider_value_bubble(f, x0, anchor_y, vals[0], d, wb);
                                    paint_xml_slider_value_bubble(f, x1, anchor_y, vals[1], d, wb);
                                }
                                Some(SliderLabelBehavior::Floating | SliderLabelBehavior::WithinBounds) if r.enabled && (r.pressed || r.focused) => {
                                    let idx = active_idx.unwrap_or(0);
                                    paint_xml_slider_value_bubble(f, if idx == 0 { x0 } else { x1 }, anchor_y, vals[idx], d, wb);
                                }
                                _ => {}
                            }
                        } else {
                            let track_h = 4.0 * d;
                            let track_r = track_h * 0.5;
                            let track_pad = 8.0 * d;
                            let track = Rect {
                                x: inner.x + track_pad,
                                y: cy - track_h * 0.5,
                                width: (inner.width - track_pad * 2.0).max(0.0),
                                height: track_h,
                            };
                            f.paint_rect(track, track_r, inactive_color);
                            f.paint_rect(
                                Rect {
                                    x: active_left,
                                    y: track.y,
                                    width: (active_right - active_left).max(0.0),
                                    height: track.height,
                                },
                                track_r,
                                active_color,
                            );
                            if anim_tick.is_some() {
                                let count = if sl_step > 0.0 {
                                    ((range_max - min) / sl_step).round().clamp(1.0, 1024.0) as usize
                                } else {
                                    (range_max - min).round().clamp(0.0, 1024.0) as usize
                                };
                                paint_anim_ticks(f, start_x, span_w, cy, count, if r.enabled { 1.0 } else { 0.38 }, self.is_auto_mirrored() && rtl);
                            } else if let Some((tw, th, trad, tc)) = colors.tick {
                                let count = if sl_step > 0.0 {
                                    ((range_max - min) / sl_step).round().clamp(1.0, 1024.0) as usize
                                } else {
                                    (range_max - min).round().clamp(0.0, 1024.0) as usize
                                };
                                if count > 1 {
                                    for i in 0..=count {
                                        let center = start_x + span_w * i as f32 / count as f32;
                                        f.paint_rect(
                                            Rect { x: center - tw * 0.5, y: cy - th * 0.5, width: tw, height: th },
                                            trad,
                                            tc,
                                        );
                                    }
                                }
                            } else if sl_step > 0.0 {
                                let steps = ((range_max - min) / sl_step).round().clamp(1.0, 128.0) as usize;
                                for i in 0..=steps {
                                    let tx = start_x + span_w * (i as f32 / steps as f32);
                                    let in_active = tx > active_left && tx < active_right;
                                    f.paint_rect(
                                        Rect { x: tx - 1.5 * d, y: cy - 1.5 * d, width: 3.0 * d, height: 3.0 * d },
                                        1.5 * d,
                                        if in_active { t.on_primary.opacity(0.6) } else { t.on_surface_variant.opacity(0.5) },
                                    );
                                }
                            }
                            let thumb_d = 20.0 * d;
                            let thumb_r = thumb_d * 0.5;
                            let halo = 40.0 * d;
                            let thumb_color = colors.thumb.unwrap_or(if r.enabled {
                                t.primary
                            } else {
                                t.text.opacity(0.38)
                            });
                            for (idx, tx) in [(0usize, x0), (1usize, x1)] {
                                let halo_rect = Rect {
                                    x: tx - halo * 0.5,
                                    y: cy - halo * 0.5,
                                    width: halo,
                                    height: halo,
                                };
                                let thumb_resp = Response {
                                    id: anonymous_child_id(id, 9700 + idx),
                                    bounds: halo_rect,
                                    hovered: r.hovered && f.pointer_in(halo_rect),
                                    pressed: r.pressed && active_idx == Some(idx),
                                    focused: r.focused && active_idx.unwrap_or(1) == idx,
                                    ..r
                                };
                                f.state_layer(thumb_resp, halo * 0.5, t.primary);
                                let thumb_rect = Rect {
                                    x: tx - thumb_r,
                                    y: cy - thumb_r,
                                    width: thumb_d,
                                    height: thumb_d,
                                };
                                if r.enabled {
                                    paint_elevation_level(f, thumb_rect, thumb_r, 1, d);
                                }
                                f.paint_rect(thumb_rect, thumb_r, thumb_color);
                            }
                            let anchor_y = cy - thumb_r;
                            let wb = matches!(sl_label_beh, Some(SliderLabelBehavior::WithinBounds)).then_some(inner);
                            match sl_label_beh {
                                Some(SliderLabelBehavior::Visible) if r.enabled => {
                                    paint_xml_slider_value_bubble(f, x0, anchor_y, vals[0], d, wb);
                                    paint_xml_slider_value_bubble(f, x1, anchor_y, vals[1], d, wb);
                                }
                                Some(SliderLabelBehavior::Floating | SliderLabelBehavior::WithinBounds) if r.enabled && (r.pressed || r.focused) => {
                                    let idx = active_idx.unwrap_or(0);
                                    paint_xml_slider_value_bubble(f, if idx == 0 { x0 } else { x1 }, anchor_y, vals[idx], d, wb);
                                }
                                _ => {}
                            }
                        }
                        f.focus_ring(r);
                        f.semantic(r, Role::Slider, self.text, false, None, Some(vals[0]), Some([min, range_max]));
                    });
                    state.range_values.insert(key, vals);
                } else if state.theme_style == AndroidTheme::Bhoo && slider_opt.is_none() {
                    let slider_enabled = self.enabled && self.max > self.min;
                    let rtl = state.rtl && self.mirror_for_rtl;
                    let value = state.progress.entry(key).or_insert(self.progress.clamp(self.min, self.max));
                    f.within(inner, |f| {
                        if f.slider_directional_tinted(id, self.text, value, self.min..=max, 1.0, slider_enabled, rtl, Some(fraction), colors).changed {
                            action = Some(self.id);
                        }
                        if anim_tick.is_some() {
                            let start_x = inner.x + 11.0;
                            let span_w = (inner.width - 22.0).max(1.0);
                            let cy = inner.y + 24.0;
                            let count = (max - self.min).round().clamp(0.0, 1024.0) as usize;
                            paint_anim_ticks(f, start_x, span_w, cy, count, if slider_enabled { 1.0 } else { 0.38 }, self.is_auto_mirrored() && rtl);
                        }
                    });
                } else {
                    let rtl = state.rtl && self.mirror_for_rtl;
                    let d = state.effective_density();
                    let slider_enabled = self.enabled && self.max > self.min;
                    let min = self.min;
                    let mut val = *state.progress.entry(key.clone()).or_insert(self.progress.clamp(self.min, self.max));
                    let old = val;
                    val = if val.is_finite() { val.clamp(min, max) } else { min };
                    let pad = if sl_expressive { 10.0 * d } else { 11.0 * d };
                    let start_x = inner.x + pad;
                    let span_w = (inner.width - pad * 2.0).max(1.0);
                    f.within(inner, |f| {
                        let r = f.interact(id, inner, slider_enabled);
                        if r.enabled && !f.input.cancelled && !f.input.blurred {
                            if f.ui.active == Some(id) && f.ui.capture == Capture::Pointer && let Some(p) = f.input.pointer {
                                let frac = ((p[0] - start_x) / span_w).clamp(0.0, 1.0);
                                val = min + (if rtl { 1.0 - frac } else { frac }) * (max - min);
                            }
                            let key_step = if sl_step > 0.0 { sl_step } else { 1.0 };
                            if let Some(key) = f.key_for(r) {
                                match key {
                                    Key::Left if rtl => val += key_step,
                                    Key::Right if rtl => val -= key_step,
                                    Key::Left | Key::Down => val -= key_step,
                                    Key::Right | Key::Up => val += key_step,
                                    Key::Home => val = min,
                                    Key::End => val = max,
                                    _ => {}
                                }
                            }
                            if val != old {
                                if sl_step > 0.0 {
                                    val = (min + ((val - min) / sl_step).round() * sl_step).clamp(min, max);
                                } else if slider_opt.is_none() {
                                    val = (min + (val - min).round()).clamp(min, max);
                                } else {
                                    val = val.clamp(min, max);
                                }
                            }
                        }
                        if val != old {
                            action = Some(self.id);
                        }
                        let amount = (val - min) / (max - min);
                        let cy = inner.y + inner.height * 0.5;
                        let x = start_x + span_w * if rtl { 1.0 - amount } else { amount };
                        if sl_expressive {
                            let track_x = inner.x + 8.0 * d;
                            let track_w = (inner.width - 16.0 * d).max(1.0);
                            let track_right = track_x + track_w;
                            let track_h = 16.0 * d;
                            let track_y = cy - track_h * 0.5;
                            let track_r = 8.0 * d;
                            let gap_half = 8.0 * d;
                            let active_color = colors.primary.unwrap_or(if r.enabled { t.primary } else { t.text.opacity(0.38) });
                            let inactive_color = colors.background.unwrap_or(if r.enabled { t.secondary_container } else { t.text.opacity(0.12) });
                            let active_stop_color = if r.enabled { t.on_primary } else { t.inverse_on_surface.opacity(0.38) };
                            let inactive_stop_color = if r.enabled { t.on_secondary_container } else { t.text.opacity(0.38) };
                            let left_end = (x - gap_half).clamp(track_x, track_right);
                            if left_end > track_x {
                                let w = left_end - track_x;
                                f.paint_rect(
                                    Rect { x: track_x, y: track_y, width: w, height: track_h },
                                    track_r.min(w * 0.5),
                                    if rtl { inactive_color } else { active_color },
                                );
                            }
                            let right_start = (x + gap_half).clamp(track_x, track_right);
                            if track_right > right_start {
                                let w = track_right - right_start;
                                f.paint_rect(
                                    Rect { x: right_start, y: track_y, width: w, height: track_h },
                                    track_r.min(w * 0.5),
                                    if rtl { active_color } else { inactive_color },
                                );
                            }
                            let dot_d = 4.0 * d;
                            let dot_r = 2.0 * d;
                            if sl_step > 0.0 {
                                let steps = ((max - min) / sl_step).round().clamp(1.0, 128.0) as usize;
                                for i in 0..=steps {
                                    let tx = start_x + span_w * (i as f32 / steps as f32);
                                    if (tx - x).abs() < gap_half {
                                        continue;
                                    }
                                    let is_active_side = if rtl { tx > x } else { tx < x };
                                    f.paint_rect(
                                        Rect { x: tx - dot_r, y: cy - dot_r, width: dot_d, height: dot_d },
                                        dot_r,
                                        if is_active_side { active_stop_color } else { inactive_stop_color },
                                    );
                                }
                            } else if !rtl && right_start <= track_right - 8.0 * d {
                                f.paint_rect(Rect { x: track_right - 8.0 * d, y: cy - dot_r, width: dot_d, height: dot_d }, dot_r, inactive_stop_color);
                            } else if rtl && left_end >= track_x + 8.0 * d {
                                f.paint_rect(Rect { x: track_x + 4.0 * d, y: cy - dot_r, width: dot_d, height: dot_d }, dot_r, inactive_stop_color);
                            }
                            let handle_w = 4.0 * d;
                            let handle_h = 44.0 * d;
                            f.paint_rect(
                                Rect { x: x - handle_w * 0.5, y: cy - handle_h * 0.5, width: handle_w, height: handle_h },
                                handle_w * 0.5,
                                colors.thumb.unwrap_or(active_color),
                            );
                            let wb = matches!(sl_label_beh, Some(SliderLabelBehavior::WithinBounds)).then_some(inner);
                            let show_bubble = match sl_label_beh {
                                Some(SliderLabelBehavior::Visible) => r.enabled,
                                Some(SliderLabelBehavior::Floating | SliderLabelBehavior::WithinBounds) => r.enabled && (r.pressed || r.focused),
                                _ => false,
                            };
                            if show_bubble {
                                paint_xml_slider_value_bubble(f, x, cy - handle_h * 0.5, val, d, wb);
                            }
                        } else {
                            let track_h = 4.0 * d;
                            let track_r = track_h * 0.5;
                            let track_pad = 8.0 * d;
                            let track = Rect { x: inner.x + track_pad, y: cy - track_h * 0.5, width: (inner.width - track_pad * 2.0).max(0.0), height: track_h };
                            let thumb_d = 20.0 * d;
                            let thumb = match colors.custom_thumb {
                                Some(Some((w, h, rad, color))) => Some((w, h, rad, colors.thumb.unwrap_or(color))),
                                Some(None) => None,
                                None => Some((thumb_d, thumb_d, thumb_d * 0.5, colors.thumb.unwrap_or(if r.enabled { t.primary } else { t.text.opacity(0.38) }))),
                            };
                            let draw_track = |f: &mut Frame<'_>| {
                                f.paint_rect(track, track_r, colors.background.unwrap_or(if r.enabled { t.surface_variant } else { t.text.opacity(0.12) }));
                                if fraction.is_finite() && fraction > 0.0 {
                                    let bw = track.width * fraction.clamp(0.0, 1.0);
                                    f.paint_rect(
                                        Rect { x: if rtl { track.x + track.width - bw } else { track.x }, width: bw, ..track },
                                        track_r,
                                        colors.secondary.unwrap_or(if r.enabled { t.primary.opacity(0.38) } else { t.text.opacity(0.12) }),
                                    );
                                }
                                f.paint_rect(
                                    Rect {
                                        x: if rtl { x } else { track.x },
                                        width: if rtl { (track.x + track.width - x).max(0.0) } else { (x - track.x).max(0.0) },
                                        ..track
                                    },
                                    track_r,
                                    colors.primary.unwrap_or(if r.enabled { t.primary } else { t.text.opacity(0.38) }),
                                );
                                if anim_tick.is_some() {
                                    let count = if sl_step > 0.0 {
                                        ((max - min) / sl_step).round().clamp(1.0, 1024.0) as usize
                                    } else {
                                        (max - min).round().clamp(0.0, 1024.0) as usize
                                    };
                                    paint_anim_ticks(f, start_x, span_w, cy, count, if r.enabled { 1.0 } else { 0.38 }, self.is_auto_mirrored() && rtl);
                                } else if let Some((tw, th, trad, tc)) = colors.tick {
                                    let count = if sl_step > 0.0 {
                                        ((max - min) / sl_step).round().clamp(1.0, 1024.0) as usize
                                    } else {
                                        (max - min).round().clamp(0.0, 1024.0) as usize
                                    };
                                    if count > 1 {
                                        for i in 0..=count {
                                            let center = start_x + span_w * i as f32 / count as f32;
                                            f.paint_rect(Rect { x: center - tw * 0.5, y: cy - th * 0.5, width: tw, height: th }, trad, tc);
                                        }
                                    }
                                } else if sl_step > 0.0 {
                                    let steps = ((max - min) / sl_step).round().clamp(1.0, 128.0) as usize;
                                    for i in 0..=steps {
                                        let tx = start_x + span_w * (i as f32 / steps as f32);
                                        let is_active_side = if rtl { tx > x } else { tx < x };
                                        f.paint_rect(
                                            Rect { x: tx - 1.5 * d, y: cy - 1.5 * d, width: 3.0 * d, height: 3.0 * d },
                                            1.5 * d,
                                            if is_active_side { t.on_primary.opacity(0.6) } else { t.on_surface_variant.opacity(0.5) },
                                        );
                                    }
                                }
                            };
                            if colors.split_track && let Some((tw, _th, _, _)) = thumb {
                                let clip = f.clip;
                                let left = x - tw * 0.5;
                                let right = x + tw * 0.5;
                                for region in [
                                    Rect { x: clip.x, y: clip.y, width: (left - clip.x).max(0.0), height: clip.height },
                                    Rect { x: right, y: clip.y, width: (clip.x + clip.width - right).max(0.0), height: clip.height },
                                ] {
                                    let region = region.intersect(clip);
                                    if region.width > 0.0 && region.height > 0.0 {
                                        f.commands.push(DrawCommand::PushClip(region));
                                        draw_track(f);
                                        f.commands.push(DrawCommand::PopClip);
                                    }
                                }
                            } else {
                                draw_track(f);
                            }
                            let halo = 40.0 * d;
                            f.state_layer(Response { bounds: Rect { x: x - halo * 0.5, y: cy - halo * 0.5, width: halo, height: halo }, ..r }, halo * 0.5, t.primary);
                            if let Some((w, h, rad, color)) = thumb {
                                f.paint_rect(Rect { x: x - w * 0.5, y: cy - h * 0.5, width: w, height: h }, rad, color);
                            }
                            let wb = matches!(sl_label_beh, Some(SliderLabelBehavior::WithinBounds)).then_some(inner);
                            let show_bubble = match sl_label_beh {
                                Some(SliderLabelBehavior::Visible) => r.enabled,
                                Some(SliderLabelBehavior::Floating | SliderLabelBehavior::WithinBounds) => r.enabled && (r.pressed || r.focused),
                                _ => false,
                            };
                            if show_bubble {
                                paint_xml_slider_value_bubble(f, x, cy - thumb_d * 0.5, val, d, wb);
                            }
                        }
                        f.focus_ring(r);
                        f.semantic(r, Role::Slider, self.text, false, None, Some(val), Some([min, max]));
                    });
                    state.progress.insert(key, val);
                }
            }
            Kind::RadioButton => {
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let label=state.text.get(&key).map_or(self.text,String::as_str).to_owned();
                let mut tint_response=view_response;
                tint_response.pressed |= enabled && f.input.pressed && f.pointer_in(inner);
                let tint=self.resolved_progress_tint(158,state,&key,tint_response);
                let anim_button_spec = self.compound_anim_spec(CompoundAnimSlot::Button);
                let custom_button=anim_button_spec.is_some() || self.relative_rules.iter().any(|rule|rule.kind==165);
                let (italic, family) = self.text_style_ext();
                if state.theme_style == AndroidTheme::Bhoo {
                    let start=f.commands.len();
                    let mut response=Response::default();
                    {
                        let value = state.checked.entry(key.clone()).or_insert(self.checked);
                        f.within(inner, |f| {
                            response=f.radio_directional_tinted(id, &label, *value, self.enabled, state.rtl,tint,custom_button);
                            if response.clicked { *value = true; action = Some(self.id); }
                        });
                    }
                    view_response = response;
                    if let Some(ref spec) = anim_button_spec {
                        let (frame_key, next_wake) = select_animation_list_frame(spec, f.time(), f.reduced_motion());
                        if let Some(img) = state.images.get(frame_key).cloned() {
                            let scale = state.image_scales.get(frame_key).copied().unwrap_or(1.0) * state.effective_density();
                            let (w, h) = (img.width() as f32 * scale, img.height() as f32 * scale);
                            let center = state.dp(20.0);
                            let cx = if state.rtl { inner.x + inner.width - center } else { inner.x + center };
                            let cy = inner.y + center;
                            let rect = Rect { x: cx - w * 0.5, y: cy - h * 0.5, width: w, height: h };
                            let mode = self.relative_rules.iter().find(|r| r.kind == 184).and_then(|r| r.anchor.parse::<u8>().ok()).unwrap_or(1);
                            let tinted = tint_image(state, &img, self.resolved_progress_tint(158, state, &key, response), mode);
                            let mirror_btn = self.is_auto_mirrored() && state.rtl;
                            f.within(inner, |f| {
                                if let Some(wake) = next_wake {
                                    let vis = rect.intersect(f.clip);
                                    if vis.width > 0.0 && vis.height > 0.0 {
                                        f.wake_at(wake);
                                    }
                                }
                                if mirror_btn {
                                    f.push_transform([cx, cy], [0.0, 0.0], [-1.0, 1.0], 0.0);
                                }
                                f.commands.push(DrawCommand::Image {
                                    bounds: rect,
                                    uv: [0.0, 0.0, 1.0, 1.0],
                                    radius: 0.0,
                                    color: Color(255, 255, 255, 255).opacity(if response.enabled { 1.0 } else { 0.38 }),
                                    image: tinted,
                                });
                                if mirror_btn {
                                    f.pop_transform();
                                }
                            });
                        }
                    } else if custom_button && let Some(Some((w,h,r,original)))=self.custom_check_mark(165,state,&key,response) {
                        let mode=self.relative_rules.iter().find(|r|r.kind==184).and_then(|r|r.anchor.parse::<u8>().ok()).unwrap_or(1);
                        let color=self.resolved_progress_tint(158,state,&key,response).map_or(original,|tint|tint_pixel(original,tint,mode));
                        let center = state.dp(20.0);
                        let x=if state.rtl {inner.x+inner.width-center-w/2.0}else{inner.x+center-w/2.0};
                        f.paint_rect(Rect{x,y:inner.y+center-h/2.0,width:w,height:h},r,color);
                    }
                    let color=self.resolved_state_color(self.text_color_states,self.color,state,&key,response);
                    let scaled_text=self.scaled_text_size(state);
                    for command in &mut f.commands[start..] {
                        if let DrawCommand::Text{text,style,color:target,..}=command && text.as_ref()==label && style.size==14.0 {
                            *text=self.display_text_for(&label, state).into_owned().into();style.size=scaled_text;style.weight=if self.text_bold{700}else{400};style.letter_spacing=self.letter_spacing_em*scaled_text;style.italic=italic;style.family=family;
                            if let Some(color)=color {*target=color;}
                        }
                    }
                } else {
                    use crate::timing::{Easing, TweenSpec};
                    let d = state.effective_density();
                    let rtl = state.rtl;
                    let scaled_text = self.scaled_text_size(state);
                    let shown = self.display_text_for(&label, state).into_owned();
                    let mut selected = *state.checked.entry(key.clone()).or_insert(self.checked);
                    let mut response = Response::default();
                    let mut label_rect = Rect::default();
                    f.within(inner, |f| {
                        response = f.interact(id, inner, self.enabled);
                        if response.clicked {
                            selected = true;
                            action = Some(self.id);
                        }
                        let ring_side = 20.0 * d;
                        let outer = Rect {
                            x: if rtl { inner.x + inner.width - 30.0 * d } else { inner.x + 10.0 * d },
                            y: inner.y + (inner.height - ring_side) * 0.5,
                            width: ring_side,
                            height: ring_side,
                        };
                        let halo = 40.0 * d;
                        f.state_layer(
                            Response { bounds: Rect { x: outer.x + ring_side * 0.5 - halo * 0.5, y: outer.y + ring_side * 0.5 - halo * 0.5, width: halo, height: halo }, ..response },
                            halo * 0.5,
                            if selected { t.primary } else { t.on_surface_variant },
                        );
                        if !custom_button {
                            let target = f32::from(selected);
                            let linear = TweenSpec::new(0.05, Easing::Linear);
                            let color_progress = f.ui.motion.css_tween(id, 210, target, target, linear, linear);
                            let opacity = f.ui.motion.css_tween(id, 211, target, target, linear, linear);
                            let scale = f.ui.motion.css_tween(id, 212, target, target, TweenSpec::new(0.3, Easing::EmphasizedDecelerate), TweenSpec::new(0.0, Easing::Linear));
                            let ring_color = tint.unwrap_or(if response.enabled { t.on_surface_variant.mix(t.primary, color_progress) } else { t.text.opacity(0.38) });
                            f.commands.push(DrawCommand::Outline { bounds: outer, radius: ring_side * 0.5, width: 2.0 * d, color: ring_color });
                            let alpha = if response.enabled { opacity } else { target };
                            let dot_scale = if !response.enabled || !selected { 1.0 } else { scale };
                            let dot_d = 10.0 * d * dot_scale;
                            if alpha > 0.001 && dot_d > 0.001 {
                                let dot = Rect { x: outer.x + (outer.width - dot_d) * 0.5, y: outer.y + (outer.height - dot_d) * 0.5, width: dot_d, height: dot_d };
                                let dot_color = tint.map(|c| c.opacity(alpha)).unwrap_or(if response.enabled { t.primary.opacity(alpha) } else { t.text.opacity(0.38) });
                                f.paint_rect(dot, dot_d * 0.5, dot_color);
                            }
                        }
                        let label_gap = 44.0 * d;
                        label_rect = Rect { x: if rtl { inner.x } else { inner.x + label_gap }, width: (inner.width - label_gap).max(0.0), ..inner };
                        f.focus_ring(response);
                        f.semantic(response, Role::Radio, &label, selected, Some(if selected { CheckState::Checked } else { CheckState::Unchecked }), None, None);
                    });
                    view_response = response;
                    state.checked.insert(key.clone(), selected);
                    let text_color = self.resolved_state_color(self.text_color_states, self.color, state, &key, response)
                        .unwrap_or_else(|| t.text.opacity(if response.enabled { 1.0 } else { 0.38 }));
                    f.within(inner, |f| {
                        f.styled_text(
                            label_rect,
                            &shown,
                            text_color,
                            TextStyle {
                                size: scaled_text,
                                weight: if self.text_bold { 700 } else { 400 },
                                align: if rtl { 2 } else { 0 },
                                letter_spacing: self.letter_spacing_em * scaled_text,
                                italic,
                                family,
                                ..TextStyle::default()
                            },
                        );
                    });
                    if let Some(ref spec) = anim_button_spec {
                        let (frame_key, next_wake) = select_animation_list_frame(spec, f.time(), f.reduced_motion());
                        if let Some(img) = state.images.get(frame_key).cloned() {
                            let scale = state.image_scales.get(frame_key).copied().unwrap_or(1.0) * d;
                            let (w, h) = (img.width() as f32 * scale, img.height() as f32 * scale);
                            let cx = if rtl { inner.x + inner.width - 20.0 * d } else { inner.x + 20.0 * d };
                            let cy = inner.y + inner.height * 0.5;
                            let rect = Rect { x: cx - w * 0.5, y: cy - h * 0.5, width: w, height: h };
                            let mode = self.relative_rules.iter().find(|r| r.kind == 184).and_then(|r| r.anchor.parse::<u8>().ok()).unwrap_or(1);
                            let tinted = tint_image(state, &img, self.resolved_progress_tint(158, state, &key, response), mode);
                            let mirror_btn = self.is_auto_mirrored() && rtl;
                            f.within(inner, |f| {
                                if let Some(wake) = next_wake {
                                    let vis = rect.intersect(f.clip);
                                    if vis.width > 0.0 && vis.height > 0.0 {
                                        f.wake_at(wake);
                                    }
                                }
                                if mirror_btn {
                                    f.push_transform([cx, cy], [0.0, 0.0], [-1.0, 1.0], 0.0);
                                }
                                f.commands.push(DrawCommand::Image {
                                    bounds: rect,
                                    uv: [0.0, 0.0, 1.0, 1.0],
                                    radius: 0.0,
                                    color: Color(255, 255, 255, 255).opacity(if response.enabled { 1.0 } else { 0.38 }),
                                    image: tinted,
                                });
                                if mirror_btn {
                                    f.pop_transform();
                                }
                            });
                        }
                    } else if custom_button && let Some(Some((w, h, rad, original))) = self.custom_check_mark(165, state, &key, response) {
                        let mode = self.relative_rules.iter().find(|r| r.kind == 184).and_then(|r| r.anchor.parse::<u8>().ok()).unwrap_or(1);
                        let color = self.resolved_progress_tint(158, state, &key, response).map_or(original, |tint| tint_pixel(original, tint, mode));
                        let cx = if rtl { inner.x + inner.width - 20.0 * d } else { inner.x + 20.0 * d };
                        let cy = inner.y + inner.height * 0.5;
                        f.paint_rect(Rect { x: cx - w * 0.5, y: cy - h * 0.5, width: w, height: h }, rad, color);
                    }
                }
            }
            Kind::CheckBox | Kind::Switch => {
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let label = state.text.get(&key).map_or(self.text,String::as_str).to_owned();
                let mut tint_response=view_response;
                tint_response.pressed |= enabled && f.input.pressed && f.pointer_in(inner);
                let tint=[self.resolved_tint_with_checked(158,state,&key,tint_response,Some(false)),self.resolved_tint_with_checked(158,state,&key,tint_response,Some(true))];
                let switch_tints=[
                    [self.resolved_tint_with_checked(159,state,&key,tint_response,Some(false)),self.resolved_tint_with_checked(159,state,&key,tint_response,Some(true))],
                    [self.resolved_tint_with_checked(160,state,&key,tint_response,Some(false)),self.resolved_tint_with_checked(160,state,&key,tint_response,Some(true))],
                ];
                let anim_button_spec = if matches!(self.kind, Kind::CheckBox) {
                    self.compound_anim_spec(CompoundAnimSlot::Button)
                } else {
                    None
                };
                let custom_button=matches!(self.kind,Kind::CheckBox) && (anim_button_spec.is_some() || self.relative_rules.iter().any(|rule|rule.kind==165));
                let sw_layout = self.switch_layout(state);
                let (italic, family) = self.text_style_ext();
                if state.theme_style == AndroidTheme::Bhoo {
                    let start=f.commands.len();
                    let mut response=Response::default();
                    {
                        let value = state.checked.entry(key.clone()).or_insert(self.checked);
                        f.within(inner, |f| {
                            response = if matches!(self.kind, Kind::Switch) {
                                f.switch_trailing_tinted(id, &label, value, self.enabled, state.rtl,switch_tints,if self.relative_rules.iter().any(|rule|rule.kind==161){Some((self.text_on,self.text_off))}else{None},sw_layout)
                            } else {
                                f.checkbox_bool_directional_tinted(id, &label, value, self.enabled, state.rtl, tint,custom_button)
                            };
                            if response.changed { action = Some(self.id); }
                        });
                    }
                    view_response = response;
                    if let Some(ref spec) = anim_button_spec {
                        let (frame_key, next_wake) = select_animation_list_frame(spec, f.time(), f.reduced_motion());
                        if let Some(img) = state.images.get(frame_key).cloned() {
                            let scale = state.image_scales.get(frame_key).copied().unwrap_or(1.0) * state.effective_density();
                            let (w, h) = (img.width() as f32 * scale, img.height() as f32 * scale);
                            let center = state.dp(20.0);
                            let cx = if state.rtl { inner.x + inner.width - center } else { inner.x + center };
                            let cy = inner.y + center;
                            let rect = Rect { x: cx - w * 0.5, y: cy - h * 0.5, width: w, height: h };
                            let mode = self.relative_rules.iter().find(|r| r.kind == 184).and_then(|r| r.anchor.parse::<u8>().ok()).unwrap_or(1);
                            let tinted = tint_image(state, &img, self.resolved_progress_tint(158, state, &key, response), mode);
                            let mirror_btn = self.is_auto_mirrored() && state.rtl;
                            f.within(inner, |f| {
                                if let Some(wake) = next_wake {
                                    let vis = rect.intersect(f.clip);
                                    if vis.width > 0.0 && vis.height > 0.0 {
                                        f.wake_at(wake);
                                    }
                                }
                                if mirror_btn {
                                    f.push_transform([cx, cy], [0.0, 0.0], [-1.0, 1.0], 0.0);
                                }
                                f.commands.push(DrawCommand::Image {
                                    bounds: rect,
                                    uv: [0.0, 0.0, 1.0, 1.0],
                                    radius: 0.0,
                                    color: Color(255, 255, 255, 255).opacity(if response.enabled { 1.0 } else { 0.38 }),
                                    image: tinted,
                                });
                                if mirror_btn {
                                    f.pop_transform();
                                }
                            });
                        }
                    } else if custom_button && let Some(Some((w,h,r,original)))=self.custom_check_mark(165,state,&key,response) {
                        let mode=self.relative_rules.iter().find(|r|r.kind==184).and_then(|r|r.anchor.parse::<u8>().ok()).unwrap_or(1);
                        let color=self.resolved_progress_tint(158,state,&key,response).map_or(original,|tint|tint_pixel(original,tint,mode));
                        let center = state.dp(20.0);
                        let x=if state.rtl {inner.x+inner.width-center-w/2.0}else{inner.x+center-w/2.0};
                        f.paint_rect(Rect{x,y:inner.y+center-h/2.0,width:w,height:h},r,color);
                    }
                    let color=self.resolved_state_color(self.text_color_states,self.color,state,&key,response);
                    let scaled_text=self.scaled_text_size(state);
                    for command in &mut f.commands[start..] {
                        if let DrawCommand::Text{text,style,color:target,..}=command && text.as_ref()==label && style.size==14.0 {
                            *text=self.display_text_for(&label, state).into_owned().into();style.size=scaled_text;style.weight=if self.text_bold{700}else{400};style.letter_spacing=self.letter_spacing_em*scaled_text;style.italic=italic;style.family=family;
                            if let Some(color)=color {*target=color;}
                        }
                    }
                } else if matches!(self.kind, Kind::CheckBox) {
                    use crate::timing::{Easing, TweenSpec};
                    let d = state.effective_density();
                    let rtl = state.rtl;
                    let scaled_text = self.scaled_text_size(state);
                    let shown = self.display_text_for(&label, state).into_owned();
                    let mut selected = *state.checked.entry(key.clone()).or_insert(self.checked);
                    let mut response = Response::default();
                    let mut label_rect = Rect::default();
                    f.within(inner, |f| {
                        response = f.interact(id, inner, self.enabled);
                        if response.clicked {
                            selected = !selected;
                            response.changed = true;
                            action = Some(self.id);
                        }
                        let target = f32::from(selected);
                        let fill_in = TweenSpec::new(0.090, Easing::Cubic([0.0, 0.0, 0.2, 1.0]));
                        let fill_out = TweenSpec::new(0.090, Easing::Cubic([0.4, 0.0, 0.6, 1.0]));
                        let amount = f.ui.motion.css_tween(id, 3, target, target, fill_in, fill_out);
                        let check_in = TweenSpec { delay: 0.090, duration: 0.090, easing: Easing::Cubic([0.0, 0.0, 0.2, 1.0]) };
                        let check_out = TweenSpec::new(0.090, Easing::Cubic([0.4, 0.0, 1.0, 1.0]));
                        let checked_amount = f.ui.motion.css_tween(id, 4, target, target, check_in, check_out);
                        let side = 18.0 * d;
                        let corner_r = 2.0 * d;
                        let stroke_w = 2.0 * d;
                        let box_rect = Rect {
                            x: if rtl { inner.x + inner.width - 29.0 * d } else { inner.x + 11.0 * d },
                            y: inner.y + (inner.height - side) * 0.5,
                            width: side,
                            height: side,
                        };
                        let halo = 40.0 * d;
                        let ink = Rect { x: box_rect.x + side * 0.5 - halo * 0.5, y: box_rect.y + side * 0.5 - halo * 0.5, width: halo, height: halo };
                        f.state_layer(Response { bounds: ink, ..response }, halo * 0.5, if selected { t.primary } else { t.on_surface_variant });
                        if !custom_button {
                            let border_color = tint[0].unwrap_or(if response.enabled { t.on_surface_variant } else { t.text.opacity(0.38) });
                            let fill_color = tint[1].unwrap_or(if response.enabled { t.primary } else { t.text.opacity(0.38) });
                            if amount < 0.999 {
                                f.commands.push(DrawCommand::Outline {
                                    bounds: box_rect,
                                    radius: corner_r,
                                    width: stroke_w,
                                    color: border_color.opacity(1.0 - amount),
                                });
                            }
                            if amount > 0.001 {
                                f.paint_rect(box_rect, corner_r, fill_color.opacity(amount));
                            }
                            if checked_amount > 0.001 {
                                let mark = if response.enabled { t.on_primary } else { t.background };
                                let inset = 2.0 * d;
                                let scale = 14.0 * d / 24.0;
                                let origin = [box_rect.x + inset, box_rect.y + inset];
                                let pts = [[1.73, 12.91], [8.1, 19.28], [22.79, 4.59]];
                                let lens = [9.0085_f32, 20.7748_f32];
                                let dist = checked_amount * (lens[0] + lens[1]);
                                let p = |v: [f32; 2]| [origin[0] + v[0] * scale, origin[1] + v[1] * scale];
                                let lerp = |a: [f32; 2], b: [f32; 2], frac: f32| [a[0] + (b[0] - a[0]) * frac, a[1] + (b[1] - a[1]) * frac];
                                let first = (dist / lens[0]).clamp(0.0, 1.0);
                                let second = ((dist - lens[0]) / lens[1]).clamp(0.0, 1.0);
                                let mut path = [p(pts[0]), p(lerp(pts[0], pts[1], first)), [0.0; 2]];
                                let count = if second > 0.0 { path[2] = p(lerp(pts[1], pts[2], second)); 3 } else { 2 };
                                f.commands.push(DrawCommand::Polyline {
                                    points: path,
                                    count,
                                    width: 3.12 * scale,
                                    color: if !selected { mark.opacity(checked_amount) } else { mark },
                                });
                            }
                        }
                        let label_gap = 44.0 * d;
                        label_rect = Rect { x: if rtl { inner.x } else { inner.x + label_gap }, width: (inner.width - label_gap).max(0.0), ..inner };
                        f.focus_ring(response);
                        f.semantic(response, Role::Checkbox, &label, selected, Some(if selected { CheckState::Checked } else { CheckState::Unchecked }), None, None);
                    });
                    view_response = response;
                    state.checked.insert(key.clone(), selected);
                    let text_color = self.resolved_state_color(self.text_color_states, self.color, state, &key, response)
                        .unwrap_or_else(|| t.text.opacity(if response.enabled { 1.0 } else { 0.38 }));
                    f.within(inner, |f| {
                        f.styled_text(
                            label_rect,
                            &shown,
                            text_color,
                            TextStyle {
                                size: scaled_text,
                                weight: if self.text_bold { 700 } else { 400 },
                                align: if rtl { 2 } else { 0 },
                                letter_spacing: self.letter_spacing_em * scaled_text,
                                italic,
                                family,
                                ..TextStyle::default()
                            },
                        );
                    });
                    if let Some(ref spec) = anim_button_spec {
                        let (frame_key, next_wake) = select_animation_list_frame(spec, f.time(), f.reduced_motion());
                        if let Some(img) = state.images.get(frame_key).cloned() {
                            let scale = state.image_scales.get(frame_key).copied().unwrap_or(1.0) * d;
                            let (w, h) = (img.width() as f32 * scale, img.height() as f32 * scale);
                            let cx = if rtl { inner.x + inner.width - 20.0 * d } else { inner.x + 20.0 * d };
                            let cy = inner.y + inner.height * 0.5;
                            let rect = Rect { x: cx - w * 0.5, y: cy - h * 0.5, width: w, height: h };
                            let mode = self.relative_rules.iter().find(|r| r.kind == 184).and_then(|r| r.anchor.parse::<u8>().ok()).unwrap_or(1);
                            let tinted = tint_image(state, &img, self.resolved_progress_tint(158, state, &key, response), mode);
                            let mirror_btn = self.is_auto_mirrored() && rtl;
                            f.within(inner, |f| {
                                if let Some(wake) = next_wake {
                                    let vis = rect.intersect(f.clip);
                                    if vis.width > 0.0 && vis.height > 0.0 {
                                        f.wake_at(wake);
                                    }
                                }
                                if mirror_btn {
                                    f.push_transform([cx, cy], [0.0, 0.0], [-1.0, 1.0], 0.0);
                                }
                                f.commands.push(DrawCommand::Image {
                                    bounds: rect,
                                    uv: [0.0, 0.0, 1.0, 1.0],
                                    radius: 0.0,
                                    color: Color(255, 255, 255, 255).opacity(if response.enabled { 1.0 } else { 0.38 }),
                                    image: tinted,
                                });
                                if mirror_btn {
                                    f.pop_transform();
                                }
                            });
                        }
                    } else if custom_button && let Some(Some((w, h, rad, original))) = self.custom_check_mark(165, state, &key, response) {
                        let mode = self.relative_rules.iter().find(|r| r.kind == 184).and_then(|r| r.anchor.parse::<u8>().ok()).unwrap_or(1);
                        let color = self.resolved_progress_tint(158, state, &key, response).map_or(original, |tint| tint_pixel(original, tint, mode));
                        let cx = if rtl { inner.x + inner.width - 20.0 * d } else { inner.x + 20.0 * d };
                        let cy = inner.y + inner.height * 0.5;
                        f.paint_rect(Rect { x: cx - w * 0.5, y: cy - h * 0.5, width: w, height: h }, rad, color);
                    }
                } else {
                    // Kind::Switch under Material3Light / Material3Dark / MaterialLight / MaterialDark
                    let theme_style = state.theme_style;
                    let m3 = matches!(theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark);
                    let show_texts = self.relative_rules.iter().any(|rule| rule.kind == 161);
                    let texts = show_texts.then_some((self.text_on, self.text_off));
                    let rtl = state.rtl;
                    let trailing = !rtl;
                    let d = state.effective_density();
                    let sp9 = state.sp(9.0);
                    let scaled_text = self.scaled_text_size(state);
                    let shown = self.display_text_for(&label, state).into_owned();
                    let mut checked_val = *state.checked.entry(key.clone()).or_insert(self.checked);
                    let mut response = Response::default();
                    let mut label_rect = Rect::default();
                    f.within(inner, |f| {
                        response = f.interact(id, inner, self.enabled);
                        if response.clicked {
                            checked_val = !checked_val;
                            response.changed = true;
                            action = Some(self.id);
                        }
                        let a = f.animate_geometry(id, 3, f32::from(checked_val), f32::from(checked_val));
                        let text_min_w = texts.map(|(on, off)| {
                            let st = TextStyle { size: sp9, ..TextStyle::default() };
                            (f.text_width(on, st).max(f.text_width(off, st)) + 2.0 * sw_layout.thumb_text_padding).max(24.0 * d)
                        }).unwrap_or(0.0);
                        let base_track_w = if m3 { 52.0 * d } else { 34.0 * d };
                        let preferred_w = (2.0 * text_min_w + 4.0 * d).max(sw_layout.min_width).max(base_track_w);
                        let track_w = preferred_w.min(inner.width.max(0.0));
                        let track_h = if m3 { 32.0 * d } else { 14.0 * d };
                        let track_r = track_h * 0.5;
                        let track_x = if trailing { (inner.x + inner.width - track_w).max(inner.x) } else { inner.x };
                        let cy = inner.y + inner.height * 0.5;
                        let track = Rect { x: track_x, y: cy - track_h * 0.5, width: track_w, height: track_h };
                        let halo = 40.0 * d;
                        let thumb_rect = if m3 {
                            let disabled_track = t.background.mix(t.text, 0.12);
                            let fill = switch_tints[1][usize::from(checked_val)].unwrap_or(if response.enabled {
                                t.surface_highest.mix(t.primary, a)
                            } else {
                                disabled_track
                            });
                            f.paint_rect(track, track_r, fill);
                            if 1.0 - a > 0.001 {
                                let border = switch_tints[1][0].unwrap_or(if response.enabled { t.outline } else { t.text.opacity(0.12) }).opacity(1.0 - a);
                                f.commands.push(DrawCommand::Outline { bounds: track, radius: track_r, width: 2.0 * d, color: border });
                            }
                            let pressed_target = f32::from(response.enabled && response.pressed);
                            let press = f.ui.motion.css_tween(id, 220, pressed_target, 0.0, crate::timing::TweenSpec::new(0.1, crate::timing::Easing::Linear), crate::timing::TweenSpec::new(0.25, crate::timing::Easing::Standard));
                            let base_d = (16.0 + 8.0 * a) * d;
                            let diameter = base_d + (28.0 * d - base_d) * press;
                            let thumb_w = diameter.max(text_min_w).min((track.width - 4.0 * d).max(0.0));
                            let start_cx = if texts.is_some() { track.x + 2.0 * d + thumb_w * 0.5 } else { track.x + track_h * 0.5 };
                            let travel = if texts.is_some() { (track.width - 4.0 * d - thumb_w).max(0.0) } else { (track.width - track_h).max(0.0) };
                            let cx = start_cx + travel * if rtl { 1.0 - a } else { a };
                            let thumb = Rect { x: cx - thumb_w * 0.5, y: cy - diameter * 0.5, width: thumb_w, height: diameter };
                            f.state_layer(Response { bounds: Rect { x: cx - halo * 0.5, y: cy - halo * 0.5, width: halo, height: halo }, ..response }, halo * 0.5, if checked_val { t.primary } else { t.text });
                            let handle = switch_tints[0][usize::from(checked_val)].unwrap_or(if response.enabled {
                                t.outline.mix(t.on_primary, a)
                            } else if checked_val {
                                t.background
                            } else {
                                t.text.opacity(0.38)
                            });
                            f.paint_rect(thumb, diameter * 0.5, handle);
                            thumb
                        } else {
                            let track_fill = switch_tints[1][usize::from(checked_val)].unwrap_or(if response.enabled {
                                t.text.opacity(0.30).mix(t.primary.opacity(0.30), a)
                            } else {
                                t.text.opacity(0.12)
                            });
                            f.paint_rect(track, track_r, track_fill);
                            let diameter = 20.0 * d;
                            let thumb_w = diameter.max(text_min_w).min(track.width.max(diameter));
                            let start_cx = track.x + diameter * 0.5;
                            let travel = (track.width - diameter).max(0.0);
                            let cx = start_cx + travel * if rtl { 1.0 - a } else { a };
                            let thumb = Rect { x: cx - thumb_w * 0.5, y: cy - diameter * 0.5, width: thumb_w, height: diameter };
                            f.state_layer(Response { bounds: Rect { x: cx - halo * 0.5, y: cy - halo * 0.5, width: halo, height: halo }, ..response }, halo * 0.5, if checked_val { t.primary } else { t.text });
                            if response.enabled {
                                paint_elevation_dp(f, thumb, diameter * 0.5, 1.5, d);
                            }
                            let unchecked_thumb = match theme_style {
                                AndroidTheme::MaterialLight => Color(0xEC, 0xEC, 0xEC, 255),
                                _ => Color(0xBD, 0xBD, 0xBD, 255),
                            };
                            let handle = switch_tints[0][usize::from(checked_val)].unwrap_or(if response.enabled {
                                unchecked_thumb.mix(t.primary, a)
                            } else {
                                unchecked_thumb.opacity(0.38)
                            });
                            f.paint_rect(thumb, diameter * 0.5, handle);
                            thumb
                        };
                        if let Some((on, off)) = texts {
                            let txt = if a > 0.5 { on } else { off };
                            f.within(thumb_rect, |f| {
                                f.styled_text(thumb_rect, txt, t.on_surface_variant.opacity(if response.enabled { 1.0 } else { 0.38 }), TextStyle { size: sp9, align: 1, ..TextStyle::default() });
                            });
                        }
                        label_rect = Rect {
                            x: if trailing { inner.x } else { inner.x + track_w + sw_layout.padding },
                            width: (inner.width - track_w - sw_layout.padding).max(0.0),
                            ..inner
                        };
                        f.focus_ring(response);
                        f.semantic(response, Role::Switch, &label, checked_val, Some(if checked_val { CheckState::Checked } else { CheckState::Unchecked }), None, None);
                    });
                    view_response = response;
                    state.checked.insert(key.clone(), checked_val);
                    let text_color = self.resolved_state_color(self.text_color_states, self.color, state, &key, response)
                        .unwrap_or_else(|| t.text.opacity(if response.enabled { 1.0 } else { 0.38 }));
                    f.within(inner, |f| {
                        f.styled_text(
                            label_rect,
                            &shown,
                            text_color,
                            TextStyle {
                                size: scaled_text,
                                weight: if self.text_bold { 700 } else { 400 },
                                align: if rtl { 2 } else { 0 },
                                letter_spacing: self.letter_spacing_em * scaled_text,
                                italic,
                                family,
                                ..TextStyle::default()
                            },
                        );
                    });
                }
            }
            Kind::Button => {
                let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let label_owned = state.text.get(&key).cloned();
                let label = label_owned.as_deref().unwrap_or(self.text);
                if let Some(seg) = toggle_seg {
                    let is_checked = state.checked.get(&key).copied().unwrap_or(self.checked);
                    let r = f.interact(id, b, self.enabled);
                    view_response = r;
                    f.semantic(
                        r,
                        Role::ToggleButton,
                        label,
                        is_checked,
                        Some(if is_checked { CheckState::Checked } else { CheckState::Unchecked }),
                        None,
                        None,
                    );
                    let inset_y = if b.height > state.dp(40.0) {
                        ((b.height - state.dp(40.0)) * 0.5).min(state.dp(4.0))
                    } else {
                        0.0
                    };
                    let seg_b = Rect {
                        x: b.x,
                        y: b.y + inset_y,
                        width: b.width,
                        height: (b.height - inset_y * 2.0).max(0.0),
                    };
                    let h_rad = seg_b.height * 0.5;
                    let seg_radii = match (seg.is_phys_left, seg.is_phys_right) {
                        (true, true) => [h_rad, h_rad, h_rad, h_rad],
                        (true, false) => [h_rad, 0.0, 0.0, h_rad],
                        (false, false) => [0.0, 0.0, 0.0, 0.0],
                        (false, true) => [0.0, h_rad, h_rad, 0.0],
                    };
                    if is_checked {
                        let fill = background_tint
                            .unwrap_or(t.secondary_container)
                            .opacity(if enabled { 1.0 } else { 0.38 });
                        paint_rounded_rect_radii(f, seg_b, seg_radii, fill);
                    } else if let Some(fill) = background_tint.filter(|c| c.3 > 0) {
                        paint_rounded_rect_radii(
                            f,
                            seg_b,
                            seg_radii,
                            fill.opacity(if enabled { 1.0 } else { 0.38 }),
                        );
                    }
                    let sw = if self.background_stroke.is_some() && self.background_stroke_width > 0.0 {
                        state.dp(self.background_stroke_width)
                    } else {
                        state.dp(1.0)
                    };
                    let stroke_color = if enabled {
                        self.background_stroke.unwrap_or(t.outline)
                    } else {
                        t.text.opacity(0.12)
                    };
                    if seg.is_phys_left {
                        paint_outline_radii(f, seg_b, seg_radii, sw, stroke_color);
                    } else {
                        let ext = Rect {
                            x: seg_b.x - sw,
                            y: seg_b.y,
                            width: seg_b.width + sw,
                            height: seg_b.height,
                        };
                        f.within(seg_b, |f| {
                            paint_outline_radii(f, ext, seg_radii, sw, stroke_color);
                        });
                    }
                    let fg_base = if is_checked { t.on_secondary_container } else { t.text };
                    let fg_color = fg_base.opacity(if enabled { 1.0 } else { 0.38 });
                    let state_r = if seg.is_phys_left || seg.is_phys_right { h_rad } else { 0.0 };
                    f.within(seg_b, |f| {
                        self.paint_state_layer(f, Response { bounds: seg_b, ..r }, state_r, Some(seg_radii), fg_base);
                    });
                    f.focus_ring(Response { bounds: seg_b, ..r });
                    let text_color = self
                        .resolved_state_color(self.text_color_states, self.color, state, &key, r)
                        .unwrap_or(fg_color);
                    let text = self.display_text_for(label, state);
                    let (text_size, weight, italic, family) = self.effective_text_font(state);
                    let style = TextStyle {
                        size: text_size,
                        weight,
                        align: 1,
                        letter_spacing: self.letter_spacing_em * text_size,
                        italic,
                        family,
                        ..TextStyle::default()
                    };
                    let pad_x = state.dp(10.0);
                    let avail_w = (seg_b.width - pad_x * 2.0).max(0.0);
                    if is_checked {
                        let icon_s = state.dp(18.0).min(seg_b.height).min(avail_w);
                        let gap = if text.is_empty() { 0.0 } else { state.dp(6.0) };
                        let text_w = f.text_width(&text, style).min((avail_w - icon_s - gap).max(0.0));
                        let content_w = icon_s + gap + text_w;
                        let start_x = seg_b.x + (seg_b.width - content_w) * 0.5;
                        let icon_x = if state.rtl {
                            start_x + content_w - icon_s
                        } else {
                            start_x
                        };
                        let icon_y = seg_b.y + (seg_b.height - icon_s) * 0.5;
                        f.paint_material_icon(
                            Rect {
                                x: icon_x,
                                y: icon_y,
                                width: icon_s,
                                height: icon_s,
                            },
                            MaterialIcon::Check,
                            text_color,
                        );
                        if !text.is_empty() && text_w > 0.0 {
                            let tx = if state.rtl { start_x } else { start_x + icon_s + gap };
                            f.within(seg_b, |f| {
                                f.styled_text(
                                    Rect {
                                        x: tx,
                                        y: seg_b.y,
                                        width: text_w,
                                        height: seg_b.height,
                                    },
                                    &text,
                                    text_color,
                                    TextStyle {
                                        align: if state.rtl { 2 } else { 0 },
                                        ..style
                                    },
                                );
                            });
                        }
                    } else {
                        f.within(seg_b, |f| {
                            f.styled_text(
                                Rect {
                                    x: seg_b.x + pad_x,
                                    y: seg_b.y,
                                    width: avail_w,
                                    height: seg_b.height,
                                },
                                &text,
                                text_color,
                                style,
                            );
                        });
                    }
                    if r.clicked {
                        action = Some(self.id);
                    }
                } else {
                let r = f.target(id, b, label, self.enabled, Role::Button);
                view_response = r;
                if let Some(chip) = chip_spec {
                    if chip.kind == XmlChipKind::Filter {
                        let entry = state.checked.entry(key.clone()).or_insert(self.checked);
                        if r.clicked {
                            *entry = !*entry;
                        }
                    }
                    let is_checked = state.checked.get(&key).copied().unwrap_or(self.checked);
                    let is_selected = is_checked || state.view_selected.get(&key).copied().unwrap_or(false);
                    let chip_h = state.dp(32.0).min(b.height);
                    let chip_rect = Rect {
                        x: b.x,
                        y: b.y + (b.height - chip_h) * 0.5,
                        width: b.width,
                        height: chip_h,
                    };
                    let chip_radius = if self.radius < 0.0 {
                        chip_rect.height * 0.5
                    } else if self.radius > 0.0 {
                        scaled_radius
                    } else if let Some([tl, tr, br, bl]) = corner_radii {
                        (tl + tr + br + bl) * 0.25
                    } else {
                        state.dp(8.0)
                    };
                    if let Some(dp) = explicit_elevation {
                        paint_elevation_level(f, chip_rect, chip_radius, elevation_level_from_dp(dp), state.effective_density());
                    }
                    let chip_is_oval = self.radius < 0.0 && corner_radii.is_none();
                    if is_selected {
                        let fill = background_tint
                            .unwrap_or(t.secondary_container)
                            .opacity(if enabled { 1.0 } else { 0.38 });
                        paint_shape_fill_with_family(f, state, chip_rect, chip_is_oval, corner_radii, chip_radius, corner_family, fill);
                        if let Some(sc) = self.background_stroke {
                            let sw = if self.background_stroke_width > 0.0 { state.dp(self.background_stroke_width) } else { state.dp(1.0) };
                            paint_shape_outline_with_family(f, chip_rect, chip_is_oval, corner_radii, chip_radius, corner_family, sw, sc);
                        }
                    } else {
                        if let Some(fill) = background_tint.or(explicit_background).filter(|c| c.3 > 0) {
                            let fill = fill.opacity(if enabled { 1.0 } else { 0.38 });
                            paint_shape_fill_with_family(f, state, chip_rect, chip_is_oval, corner_radii, chip_radius, corner_family, fill);
                        }
                        let raw_stroke = self.background_stroke.unwrap_or(t.outline);
                        let stroke_color = if enabled { raw_stroke } else { t.text.opacity(0.12) };
                        let sw = if self.background_stroke.is_some() && self.background_stroke_width > 0.0 {
                            state.dp(self.background_stroke_width)
                        } else {
                            state.dp(1.0)
                        };
                        paint_shape_outline_with_family(f, chip_rect, chip_is_oval, corner_radii, chip_radius, corner_family, sw, stroke_color);
                    }
                    let fg_base = if is_selected { t.on_secondary_container } else { t.on_surface_variant };
                    let fg_color = fg_base.opacity(if enabled { 1.0 } else { 0.38 });
                    self.paint_state_layer(f, Response { bounds: chip_rect, ..r }, chip_radius, corner_radii, fg_base);
                    f.focus_ring(Response { bounds: chip_rect, ..r });
                    let show_check = is_selected && chip.kind == XmlChipKind::Filter;
                    let icon_size = state.dp(18.0).min(chip_rect.height);
                    let icon_pad = state.dp(8.0);
                    if show_check {
                        let check_x = if state.rtl {
                            chip_rect.x + chip_rect.width - icon_pad - icon_size
                        } else {
                            chip_rect.x + icon_pad
                        };
                        let check_y = chip_rect.y + (chip_rect.height - icon_size) * 0.5;
                        f.paint_material_icon(
                            Rect { x: check_x, y: check_y, width: icon_size, height: icon_size },
                            MaterialIcon::Check,
                            fg_color,
                        );
                    }
                    if chip.close_visible {
                        let close_x = if state.rtl {
                            chip_rect.x + icon_pad
                        } else {
                            chip_rect.x + chip_rect.width - icon_pad - icon_size
                        };
                        let close_y = chip_rect.y + (chip_rect.height - icon_size) * 0.5;
                        f.paint_material_icon(
                            Rect { x: close_x, y: close_y, width: icon_size, height: icon_size },
                            MaterialIcon::Close,
                            fg_color,
                        );
                    }
                    let lead_space = if show_check { state.dp(34.0) } else { state.dp(16.0) };
                    let trail_space = if chip.close_visible { state.dp(34.0) } else { state.dp(16.0) };
                    let text_x = chip_rect.x + if state.rtl { trail_space } else { lead_space };
                    let text_w = (chip_rect.width - lead_space - trail_space).max(0.0);
                    let text_rect = Rect { x: text_x, y: chip_rect.y, width: text_w, height: chip_rect.height };
                    let text_color = self
                        .resolved_state_color(self.text_color_states, self.color, state, &key, r)
                        .unwrap_or(fg_color);
                    let text = self.display_text_for(label, state);
                    let (text_size, weight, italic, family) = self.effective_text_font(state);
                    let style = TextStyle {
                        size: text_size,
                        weight,
                        align: 1,
                        letter_spacing: self.letter_spacing_em * text_size,
                        italic,
                        family,
                        ..TextStyle::default()
                    };
                    f.within(chip_rect, |f| f.styled_text(text_rect, &text, text_color, style));
                    if r.clicked {
                        action = Some(self.id);
                    }
                } else if let Some((_fab_spec, fab_p)) = ext_fab {
                    let fab_radius = if self.radius < 0.0 {
                        b.width.min(b.height) * 0.5
                    } else if self.radius > 0.0 {
                        scaled_radius
                    } else if let Some([tl, tr, br, bl]) = corner_radii {
                        (tl + tr + br + bl) * 0.25
                    } else {
                        state.dp(16.0)
                    };
                    self.paint_state_layer(f, r, fab_radius, corner_radii, t.on_primary_container);
                    f.focus_ring(r);
                    let default_fg = if !self.has_explicit_custom_background() {
                        t.on_primary_container.opacity(if enabled { 1.0 } else { 0.38 })
                    } else {
                        t.primary.opacity(if enabled { 1.0 } else { 0.38 })
                    };
                    let fg_color = self
                        .resolved_state_color(self.text_color_states, self.color, state, &key, r)
                        .unwrap_or(default_fg);
                    let label_color = fg_color.opacity(fab_p.clamp(0.0, 1.0));
                    let shapes = self.compound_drawables(state, state.rtl);
                    let anim_slots = self.compound_anim_slots(state.rtl);
                    let has_compound = shapes.iter().any(Option::is_some) || anim_slots.iter().any(Option::is_some);
                    if fab_p <= 0.01 {
                        if has_compound {
                            let tint = self.resolved_progress_tint(176, state, &key, r);
                            let mode = self.relative_rules.iter().find(|rule| rule.kind == 177).and_then(|rule| rule.anchor.parse::<u8>().ok()).unwrap_or(1);
                            if let Some(slot) = anim_slots.into_iter().flatten().next()
                                && let Some(spec) = self.compound_anim_spec(slot)
                            {
                                let (frame_key, next_wake) = select_animation_list_frame(&spec, f.time(), f.reduced_motion());
                                if let Some(img) = state.images.get(frame_key).cloned() {
                                    let scale = state.image_scales.get(frame_key).copied().unwrap_or(1.0) * state.effective_density();
                                    let (w, h) = (img.width() as f32 * scale, img.height() as f32 * scale);
                                    let rect = Rect {
                                        x: inner.x + (inner.width - w) * 0.5,
                                        y: inner.y + (inner.height - h) * 0.5,
                                        width: w,
                                        height: h,
                                    };
                                    if let Some(wake) = next_wake {
                                        let vis = rect.intersect(f.clip);
                                        if vis.width > 0.0 && vis.height > 0.0 {
                                            f.wake_at(wake);
                                        }
                                    }
                                    let tinted = tint_image(state, &img, tint, mode);
                                    f.commands.push(DrawCommand::Image {
                                        bounds: rect,
                                        uv: [0.0, 0.0, 1.0, 1.0],
                                        radius: 0.0,
                                        color: Color(255, 255, 255, 255).opacity(if r.enabled { 1.0 } else { 0.38 }),
                                        image: tinted,
                                    });
                                }
                            } else {
                            let has_relative = self.relative_rules.iter().any(|rule| {
                                (rule.kind == 173 || rule.kind == 174)
                                    && rule.anchor.split(',').next().and_then(|n| n.parse::<f32>().ok()).is_some_and(|w| w > 0.0)
                            });
                            let sides = if has_relative {
                                if state.rtl { [174, 170, 173, 172] } else { [173, 170, 174, 172] }
                            } else {
                                [169, 170, 171, 172]
                            };
                            if let Some((i, shape)) = shapes.into_iter().enumerate().find(|(_, s)| s.is_some()) {
                                let selected = self.custom_check_mark(sides[i] + 9, state, &key, r).unwrap_or(shape);
                                if let (Some((w, h, _, _)), Some((sw, sh, radius, original))) = (shape, selected) {
                                    let radius = if radius < 0.0 { radius } else { radius * (w / sw).min(h / sh) };
                                    let color = tint.map_or(original, |value| tint_pixel(original, value, mode));
                                    let rect = Rect {
                                        x: inner.x + (inner.width - w) * 0.5,
                                        y: inner.y + (inner.height - h) * 0.5,
                                        width: w,
                                        height: h,
                                    };
                                    if radius < 0.0 { paint_oval(f, rect, color); } else { f.paint_rect(rect, radius, color); }
                                }
                            }
                            }
                        } else {
                            let icon_s = state.dp(24.0).min(inner.width).min(inner.height);
                            let icon_rect = Rect {
                                x: inner.x + (inner.width - icon_s) * 0.5,
                                y: inner.y + (inner.height - icon_s) * 0.5,
                                width: icon_s,
                                height: icon_s,
                            };
                            if !self.image_src.is_empty() && let Some(img) = state.images.get(self.image_src).cloned() {
                                let tint = self.resolved_image_tint(state, &key, r).or(Some(fg_color));
                                let tinted = tint_image(state, &img, tint, 1);
                                f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                            } else {
                                let icon = if !self.image_src.is_empty() {
                                    parse_material_icon(self.image_src).unwrap_or(self.icon)
                                } else {
                                    self.icon
                                };
                                f.paint_material_icon(icon_rect, icon, fg_color);
                            }
                        }
                    } else {
                        let text = self.display_text_for(label, state).into_owned();
                        let (text_size, weight, italic, family) = self.effective_text_font(state);
                        let align = self.resolved_text_alignment(state.rtl).unwrap_or_else(|| {
                            if self.relative_rules.iter().any(|rule| rule.kind == 168) {
                                match self.content_gravity[0] {
                                    0 if state.rtl => 2,
                                    2 if state.rtl => 0,
                                    3 => 0,
                                    4 => 2,
                                    other => other,
                                }
                            } else {
                                1
                            }
                        });
                        let style = TextStyle {
                            size: text_size,
                            weight,
                            align,
                            letter_spacing: self.letter_spacing_em * text_size,
                            italic,
                            family,
                            ..TextStyle::default()
                        };
                        if has_compound {
                            let extent = self.compound_extents(state, state.rtl);
                            let text_area = Rect {
                                x: inner.x + extent[0],
                                y: inner.y + extent[1],
                                width: (inner.width - extent[0] - extent[2]).max(0.0),
                                height: (inner.height - extent[1] - extent[3]).max(0.0),
                            };
                            self.paint_compound_drawables(f, inner, text_area, state, &key, r);
                            if !text.is_empty() {
                                f.within(inner, |f| f.styled_text(text_area, &text, label_color, style));
                            }
                        } else if !self.image_src.is_empty() {
                            let icon_s = state.dp(24.0).min(inner.height).min(inner.width);
                            let gap = if text.is_empty() { 0.0 } else { state.dp(12.0) * fab_p };
                            let text_w = f.text_width(&text, style).min((inner.width - icon_s - gap).max(0.0));
                            let group_w = icon_s + gap + text_w;
                            let group_x = inner.x + ((inner.width - group_w) * 0.5).max(0.0);
                            let icon_x = if state.rtl { group_x + group_w - icon_s } else { group_x };
                            let icon_rect = Rect {
                                x: icon_x,
                                y: inner.y + (inner.height - icon_s) * 0.5,
                                width: icon_s,
                                height: icon_s,
                            };
                            if let Some(img) = state.images.get(self.image_src).cloned() {
                                let tint = self.resolved_image_tint(state, &key, r).or(Some(fg_color));
                                let tinted = tint_image(state, &img, tint, 1);
                                f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                            } else {
                                let icon = parse_material_icon(self.image_src).unwrap_or(self.icon);
                                f.paint_material_icon(icon_rect, icon, fg_color);
                            }
                            if !text.is_empty() && text_w > 0.0 {
                                let tx = if state.rtl { group_x } else { group_x + icon_s + gap };
                                f.within(inner, |f| {
                                    f.styled_text(
                                        Rect { x: tx, y: inner.y, width: text_w, height: inner.height },
                                        &text,
                                        label_color,
                                        TextStyle { align: if state.rtl { 2 } else { 0 }, ..style },
                                    );
                                });
                            }
                        } else if !text.is_empty() {
                            f.within(inner, |f| f.styled_text(inner, &text, label_color, style));
                        }
                    }
                    if r.clicked {
                        action = Some(self.id);
                    }
                } else {
                if let Some(variant) = active_btn_variant {
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
                    let state_color = match variant {
                        XmlButtonVariant::Filled => t.on_primary,
                        XmlButtonVariant::Tonal => t.on_secondary_container,
                        XmlButtonVariant::Outlined | XmlButtonVariant::Text | XmlButtonVariant::Elevated => {
                            self.color.unwrap_or(t.primary)
                        }
                    };
                    self.paint_state_layer(f, Response { bounds: visual_b, ..r }, visual_radius, btn_corner_radii, state_color);
                } else if snackbar_text_btn {
                    self.paint_state_layer(f, r, state.dp(4.0), None, t.inverse_primary);
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
                    let state_color = match state.theme_style {
                        AndroidTheme::MaterialLight => Color(0, 0, 0, 255),
                        AndroidTheme::MaterialDark => Color(255, 255, 255, 255),
                        AndroidTheme::Material3Light | AndroidTheme::Material3Dark => t.on_primary,
                        AndroidTheme::Bhoo => t.primary,
                    };
                    self.paint_state_layer(f, Response { bounds: visual_b, ..r }, visual_radius, corner_radii, state_color);
                } else {
                    let state_radius = if self.radius < 0.0
                    || (matches!(self.kind, Kind::Icon)
                        && self.radius == 0.0
                        && corner_radii.is_none()
                        && !self.has_explicit_custom_background())
                {
                    b.width.min(b.height) * 0.5
                } else {
                    self.resolved_state_radius(state)
                };
                    self.paint_state_layer(f, r, state_radius, corner_radii, t.primary);
                }
                f.focus_ring(r);
                let shapes=self.compound_sizes(state,state.rtl);let extent=self.compound_extents(state,state.rtl);
                let text_area=Rect{x:inner.x+extent[0],y:inner.y+extent[1],width:(inner.width-extent[0]-extent[2]).max(0.0),height:(inner.height-extent[1]-extent[3]).max(0.0)};
                self.paint_compound_drawables(f, inner, text_area, state, &key, r);
                let default_text_color = if snackbar_text_btn {
                    t.inverse_primary.opacity(if enabled { 1.0 } else { 0.38 })
                } else if let Some(variant) = active_btn_variant {
                    match variant {
                        XmlButtonVariant::Filled => t.on_primary,
                        XmlButtonVariant::Tonal => t.on_secondary_container,
                        XmlButtonVariant::Outlined | XmlButtonVariant::Text | XmlButtonVariant::Elevated => t.primary,
                    }
                    .opacity(if enabled { 1.0 } else { 0.38 })
                } else {
                    match state.theme_style {
                        AndroidTheme::Bhoo => t.primary,
                        AndroidTheme::MaterialLight => Color(0x00, 0x00, 0x00, 0xDE).opacity(if enabled { 1.0 } else { 0.38 }),
                        AndroidTheme::MaterialDark => Color(0xFF, 0xFF, 0xFF, 255).opacity(if enabled { 1.0 } else { 0.38 }),
                        AndroidTheme::Material3Light | AndroidTheme::Material3Dark => {
                            let base = if use_default_button_chrome { t.on_primary } else { t.primary };
                            base.opacity(if enabled { 1.0 } else { 0.38 })
                        }
                    }
                };
                let text_color=self.resolved_state_color(self.text_color_states,self.color,state,&key,r).unwrap_or(default_text_color);
                let text = self.display_text_for(label, state);
                if split_role.is_some_and(|sr| sr.is_trailing_chevron) && text.is_empty() && shapes.iter().all(Option::is_none) {
                    let cx = b.x + b.width * 0.5;
                    let cy = b.y + b.height * 0.5;
                    let dx = state.dp(5.0).min(b.width * 0.25);
                    let dy = state.dp(2.5).min(b.height * 0.2);
                    f.commands.push(DrawCommand::Polyline {
                        points: [[cx - dx, cy - dy], [cx, cy + dy], [cx + dx, cy - dy]],
                        count: 3,
                        width: state.dp(2.0),
                        color: text_color,
                    });
                } else {
                let align = self.resolved_text_alignment(state.rtl).unwrap_or_else(|| if self.relative_rules.iter().any(|rule|rule.kind==168) {
                    match self.content_gravity[0] {0 if state.rtl=>2,2 if state.rtl=>0,3=>0,4=>2,other=>other}
                } else {1});
                let (base_text_size, weight, italic, family) = self.effective_text_font(state);
                let metrics=f.ui.text_metrics;
                let layout_ext = self.text_layout_ext();
                let base_style=TextStyle{size:base_text_size,weight,align,letter_spacing:self.letter_spacing_em*base_text_size,italic,family,..TextStyle::default()};
                let text_size = if let Some(ext) = layout_ext && ext.has_auto_size() {
                    self.resolve_auto_text_size(&text, text_area.width, text_area.height, base_style, ext, state, metrics)
                } else {
                    base_text_size
                };
                let style=TextStyle{size:text_size,weight,align,letter_spacing:self.letter_spacing_em*text_size,italic,family,..TextStyle::default()};
                let line_height=self.resolved_line_height(text_size,state);let advance=(line_height*self.line_spacing_multiplier+state.dp(self.line_spacing_extra)).max(1.0);
                let text_width=text_area.width+0.01;
                let simple=layout_ext.is_none() && !text.contains(['\n','\r','\u{000b}','\u{000c}','\u{0085}','\u{2028}','\u{2029}']) && metrics.width(&text,style)<=text_width && self.min_lines<=1 && self.ellipsize_side==0 && !self.ellipsize_end;
                if simple {
                    let draw_rect = if align == 1 && shapes.iter().all(Option::is_none) {
                        let slack = padding[0].min(padding[2]).min(state.dp(6.0));
                        Rect {
                            x: text_area.x - slack,
                            y: text_area.y,
                            width: text_area.width + slack * 2.0,
                            height: text_area.height,
                        }
                    } else {
                        text_area
                    };
                    f.styled_text(draw_rect, &text, text_color, style);
                } else if let Some(ext) = layout_ext {
                    let single=self.single_line || (self.max_lines==1 && (self.ellipsize_side!=0||self.ellipsize_end));
                    let lines = if single { Vec::new() } else { wrap_text_lines_ext(&text, text_width, style, metrics, ext) };
                    let total=if single {1} else {lines.len().max(1)};
                    let height_limit=if text_area.height<=0.0 {0}else{((text_area.height-line_height).max(0.0)/advance).ceil() as usize+1};
                    let limit=if self.max_lines==0 {height_limit}else{height_limit.min(self.max_lines)};let shown=total.min(limit);
                    let used=if shown==0 {0.0}else{(line_height+(shown-1) as f32*advance).min(text_area.height.max(0.0))};
                    let extra=(text_area.height-used).max(0.0);let gravity=if self.relative_rules.iter().any(|rule|rule.kind==168){self.content_gravity[1]}else{1};
                    let y=text_area.y+match gravity {0=>0.0,2=>extra,_=>extra*0.5};
                    let justify = ext.justification == TextJustificationMode::InterWord;
                    let rtl = state.rtl;
                    f.within(text_area,|f|{
                        if single {
                            if limit>0 {let visible=text.replace('\u{00ad}',"");let line=if self.ellipsize_end||self.ellipsize_side!=0 {ellipsize_single_line(&visible,text_area.width,style,metrics,if self.ellipsize_end {3}else{self.ellipsize_side})}else{visible};f.styled_text(Rect{y,height:line_height,..text_area},&line,text_color,style);}
                        } else {
                            for (index, line) in lines.iter().enumerate().take(limit) {
                                let bounds=Rect{y:y+index as f32*advance,height:line_height,..text_area};
                                if self.ellipsize_end && index+1==limit && total>limit {
                                    let source=if line.discretionary_hyphen {line.text.strip_suffix('-').unwrap_or(&line.text)}else{&line.text};
                                    f.styled_text(bounds,&ellipsize_truncated_line(source,text_area.width,style,metrics),text_color,style);
                                } else if justify && !line.is_paragraph_end {
                                    paint_justified_line(f, bounds, &line.text, text_color, style, metrics, rtl);
                                } else {
                                    f.styled_text(bounds,&line.text,text_color,style);
                                }
                            }
                        }
                    });
                } else {
                let single=self.single_line || (self.max_lines==1 && (self.ellipsize_side!=0||self.ellipsize_end));
                let total=if single {1}else{wrapped_text_lines(&text,text_width,style,metrics,|_|{}).max(1)};
                let height_limit=if text_area.height<=0.0 {0}else{((text_area.height-line_height).max(0.0)/advance).ceil() as usize+1};
                let limit=if self.max_lines==0 {height_limit}else{height_limit.min(self.max_lines)};let shown=total.min(limit);
                let used=if shown==0 {0.0}else{(line_height+(shown-1) as f32*advance).min(text_area.height.max(0.0))};
                let extra=(text_area.height-used).max(0.0);let gravity=if self.relative_rules.iter().any(|rule|rule.kind==168){self.content_gravity[1]}else{1};
                let y=text_area.y+match gravity {0=>0.0,2=>extra,_=>extra*0.5};
                f.within(text_area,|f|{
                    if single {
                        if limit>0 {let visible=text.replace('\u{00ad}',"");let line=if self.ellipsize_end||self.ellipsize_side!=0 {ellipsize_single_line(&visible,text_area.width,style,metrics,if self.ellipsize_end {3}else{self.ellipsize_side})}else{visible};f.styled_text(Rect{y,height:line_height,..text_area},&line,text_color,style);}
                    } else {let mut index=0;wrapped_text_lines(&text,text_width,style,metrics,|line|{if index<limit {let bounds=Rect{y:y+index as f32*advance,height:line_height,..text_area};if self.ellipsize_end && index+1==limit && total>limit {let source=if line.discretionary_hyphen {line.text.strip_suffix('-').unwrap_or(&line.text)}else{&line.text};f.styled_text(bounds,&ellipsize_truncated_line(source,text_area.width,style,metrics),text_color,style);}else{f.styled_text(bounds,&line.text,text_color,style);}}index+=1;});}
                });
                }
                }
                if r.clicked {
                    action = Some(self.id)
                }
                }
                }
            }
            Kind::Icon | Kind::Zoom => {
                let r = f.target(id, b, self.text, self.enabled, Role::Button);
                view_response = r;
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let has_stateful_src = self.relative_rules.iter().any(|rule| rule.kind == RULE_STATEFUL_IMAGE_SRC);
                let dyn_spec = self.dynamic_drawable_src();
                if let Some(ref spec) = dyn_spec
                    && !state.image_sources.contains_key(&key)
                {
                    let tint = self.resolved_image_tint(state, &key, r);
                    let alpha_color = Color(255,255,255,self.image_alpha).opacity(if r.enabled || !self.image_tint_states.is_empty() { 1.0 } else { 0.38 });
                    let mirror = self.is_auto_mirrored() && state.rtl;
                    let effective_scale = if matches!(self.kind, Kind::Icon)
                        && self.image_scale == 0
                        && !self.image_adjust_bounds
                        && self.padding_explicit == [false; 4]
                        && self.padding_relative == [None, None]
                    {
                        4
                    } else {
                        self.image_scale
                    };
                    let clip_box = if self.relative_rules.iter().any(|r| r.kind == 188) { inner } else { b };
                    paint_dynamic_src_drawable(
                        f,
                        state,
                        inner,
                        clip_box,
                        &key,
                        spec,
                        effective_scale,
                        tint,
                        self.image_tint_mode,
                        alpha_color,
                        mirror,
                    );
                } else {
                let anim_spec = self.animation_list_spec();
                let anim_vec_spec = if anim_spec.is_none() { self.animated_vector_spec(true) } else { None };
                let anim_sel_frame = if anim_spec.is_none()
                    && anim_vec_spec.is_none()
                    && !state.image_sources.contains_key(&key)
                    && self.has_animated_selector(true)
                {
                    self.sample_animated_selector(f, b, state, &key, r, true)
                } else {
                    None
                };
                let (source, stretch_spec, has_override) = if let Some(ref spec) = anim_spec
                    && !state.image_sources.contains_key(&key)
                {
                    let (frame_key, next_wake) = select_animation_list_frame(spec, f.time(), f.reduced_motion());
                    if let Some(wake) = next_wake {
                        let vis_rect = b.intersect(f.clip);
                        if vis_rect.width > 0.0 && vis_rect.height > 0.0 {
                            f.wake_at(wake);
                        }
                    }
                    (frame_key, None, true)
                } else if let Some(ref spec) = anim_vec_spec
                    && !state.image_sources.contains_key(&key)
                {
                    let (frame_key, next_wake) = select_animated_vector_frame(spec, f.time(), f.reduced_motion());
                    if let Some(wake) = next_wake {
                        let vis_rect = b.intersect(f.clip);
                        if vis_rect.width > 0.0 && vis_rect.height > 0.0 {
                            f.wake_at(wake);
                        }
                    }
                    (frame_key, None, true)
                } else if let Some(frame_key) = anim_sel_frame {
                    (frame_key, None, true)
                } else {
                    self.resolved_image_src(state, &key, r)
                };
                let image = if source.is_empty() { None } else { state.images.get(source).cloned() };
                let density=state.image_scales.get(source).copied().unwrap_or(1.0)*state.effective_density();
                let mirror = self.is_auto_mirrored() && state.rtl;
                if let Some(image) = image {
                    let tint=self.resolved_image_tint(state,&key,r);
                    let alpha_color = Color(255,255,255,self.image_alpha).opacity(if r.enabled || !self.image_tint_states.is_empty() { 1.0 } else { 0.38 });
                    if let Some(ref bands) = stretch_spec {
                        if inner.width > 0.0 && inner.height > 0.0 {
                            if mirror {
                                f.push_transform(
                                    [inner.x + inner.width * 0.5, inner.y + inner.height * 0.5],
                                    [0.0, 0.0],
                                    [-1.0, 1.0],
                                    0.0,
                                );
                            }
                            paint_nine_patch_image(f, state, inner, &image, bands, tint, self.image_tint_mode, alpha_color.3, density);
                            if mirror {
                                f.pop_transform();
                            }
                        }
                    } else {
                        let effective_scale = if matches!(self.kind, Kind::Icon)
                            && self.image_scale == 0
                            && !self.image_adjust_bounds
                            && self.padding_explicit == [false; 4]
                            && self.padding_relative == [None, None]
                        {
                            4
                        } else {
                            self.image_scale
                        };
                        if let Some((bounds, uv)) = image_geometry(inner, if self.relative_rules.iter().any(|r|r.kind==188){inner}else{b}, &image, effective_scale, density, state.image_matrices.get(&key).copied())
                            && bounds.intersect(f.clip).width > 0.0 && bounds.intersect(f.clip).height > 0.0
                        {
                            let image=tint_image(state,&image,tint,self.image_tint_mode);
                            if mirror {
                                f.push_transform(
                                    [bounds.x + bounds.width * 0.5, bounds.y + bounds.height * 0.5],
                                    [0.0, 0.0],
                                    [-1.0, 1.0],
                                    0.0,
                                );
                            }
                            f.commands.push(DrawCommand::Image { bounds, uv, radius: 0.0, color: alpha_color, image });
                            if mirror {
                                f.pop_transform();
                            }
                        }
                    }
                } else if self.image_builtin && !has_override && !has_stateful_src {
                    let base=self.color.unwrap_or(t.text);
                    let tint=self.resolved_image_tint(state,&key,r);
                    let color=tint.map_or(base,|tint| tint_pixel(base,tint,self.image_tint_mode));
                    let icon_side = state.dp(24.0);
                    let icon_rect = Rect { x: inner.x + (inner.width - icon_side) / 2., y: inner.y + (inner.height - icon_side) / 2., width: icon_side, height: icon_side };
                    if mirror {
                        f.push_transform(
                            [icon_rect.x + icon_rect.width * 0.5, icon_rect.y + icon_rect.height * 0.5],
                            [0.0, 0.0],
                            [-1.0, 1.0],
                            0.0,
                        );
                    }
                    f.paint_material_icon(
                        icon_rect,
                        self.icon,
                        color.opacity(f32::from(self.image_alpha)/255.0 * if r.enabled || !self.image_tint_states.is_empty() { 1.0 } else { 0.38 }),
                    );
                    if mirror {
                        f.pop_transform();
                    }
                }
                }
                let state_radius = if self.radius < 0.0
                    || (matches!(self.kind, Kind::Icon)
                        && self.radius == 0.0
                        && corner_radii.is_none()
                        && !self.has_explicit_custom_background())
                {
                    b.width.min(b.height) * 0.5
                } else {
                    self.resolved_state_radius(state)
                };
                self.paint_state_layer(f, r, state_radius, corner_radii, t.primary);
                f.focus_ring(r);
                if matches!(self.kind,Kind::Zoom) {
                    if zoom_activated(f,r,b,state,id,&key) { action=Some(self.id); }
                } else if r.clicked { action=Some(self.id); }
            }
            Kind::ZoomControls => {
                let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                state.zoom_controls_actions.remove(&key);
                let enabled=state.zoom_controls_enabled.get(&key).copied().unwrap_or([true,true]);
                let w=(inner.width/2.0).min(79.0);
                for i in 0..2 {
                    let x=if state.rtl { inner.x+inner.width-(i+1) as f32*w } else { inner.x+i as f32*w };
                    let rect=Rect { x,y:inner.y,width:w,height:inner.height.min(48.0) };
                    let child=anonymous_child_id(id,i);
                    let response=f.target(child,rect,if i==0 { "Zoom out" } else { "Zoom in" },self.enabled && enabled[i],Role::Button);
                    let visual=if !response.enabled { if response.focused { 4 } else { 2 } } else if response.pressed { 1 } else if response.focused { 3 } else { 0 };
                    paint_zoom_nine_patch(f,rect,zoom_control_image(i,visual),i);
                    if zoom_activated(f,response,rect,state,child,&key) {
                        state.zoom_controls_actions.insert(key.clone(),if i==0 {-1} else {1});
                        action=Some(self.id);
                    }
                }
            }
            Kind::Edit | Kind::AutoComplete | Kind::MultiAutoComplete => {
                let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                let inner = {
                    let compound_sz = self.compound_sizes(state, state.rtl);
                    if compound_sz.iter().any(Option::is_some) {
                        let extent = self.compound_extents(state, state.rtl);
                        let text_area = Rect {
                            x: inner.x + extent[0],
                            y: inner.y + extent[1],
                            width: (inner.width - extent[0] - extent[2]).max(0.0),
                            height: (inner.height - extent[1] - extent[3]).max(0.0),
                        };
                        self.paint_compound_drawables(f, inner, text_area, state, &key, view_response);
                        text_area
                    } else {
                        inner
                    }
                };
                state.input_types.insert(key.clone(), self.input_type);
                let text = state.text.entry(key.clone()).or_insert_with(|| self.text.to_string()).clone();
                let til_ctx = state.text_input_active_box;
                let password_revealed = state.text_input_password_visible.get(&key).copied().unwrap_or(false);
                let is_password = self.input_type == 4 || til_ctx.is_some_and(|c| c.password_toggle);
                let display = if is_password && !password_revealed { "•".repeat(text.chars().count()) } else { text.clone() };
                let has_explicit_bg = explicit_background.is_some()
                    || background_tint.is_some()
                    || self.background_stroke.is_some()
                    || gradient.is_some()
                    || has_layers
                    || has_stateful_layers
                    || has_bg_image_rule;
                let text_size = self.scaled_text_size(state);
                let (italic, family) = self.text_style_ext();
                let line_h = self.resolved_line_height(text_size, state);
                let field_max_h = state.dp(48.0);
                let edit_links_spec = (!is_password)
                    .then(|| self.text_links_spans())
                    .flatten()
                    .filter(|s| s.mask != 0);
                let edit_link_color = edit_links_spec.and_then(|s| s.link_color).unwrap_or(t.primary);
                let mut edit_clicked_link: Option<String> = None;
                if let Some(ctx) = til_ctx {
                    f.within(inner, |f| f.with_enabled(self.enabled, |f| {
                        let field_bounds = inner;
                        let response = f.target(id, field_bounds, self.hint, self.enabled, Role::TextField);
                        view_response = response;
                        let align = self.resolved_text_alignment(state.rtl).unwrap_or(if state.rtl { 2 } else { 0 });
                        let pad_x = if ctx.has_start_icon { state.dp(4.0) } else { state.dp(16.0) };
                        let pad_end = state.dp(16.0);
                        let text_rect = if ctx.filled && ctx.has_label {
                            let top_off = state.dp(20.0).min(field_bounds.height * 0.45);
                            Rect {
                                y: field_bounds.y + top_off,
                                height: (field_bounds.height - top_off - state.dp(2.0)).max(0.0),
                                ..field_bounds
                            }
                        } else {
                            field_bounds
                        };
                        let v_pad_top = if ctx.filled && ctx.has_label { state.dp(4.0) } else { state.dp(14.0) };
                        let v_pad_bottom = if ctx.filled && ctx.has_label { state.dp(6.0) } else { state.dp(14.0) };
                        let usable_h = (text_rect.height
                            - if self.padding_explicit[1] { 0.0 } else { v_pad_top }
                            - if self.padding_explicit[3] { 0.0 } else { v_pad_bottom })
                        .max(0.0);
                        let is_multiline = !self.single_line
                            && self.max_lines != 1
                            && (self.min_lines > 1 || display.contains('\n') || usable_h >= line_h * 1.9);
                        let style = TextStyle {
                            size: text_size,
                            weight: if self.text_bold { 700 } else { 400 },
                            align,
                            padding: pad_x,
                            letter_spacing: self.letter_spacing_em * text_size,
                            italic,
                            family,
                            ..TextStyle::default()
                        };
                        let affix_style = TextStyle { padding: 0.0, align: if state.rtl { 2 } else { 0 }, ..style };
                        let affix_color = t.on_surface_variant.opacity(if response.enabled { 1.0 } else { 0.38 });
                        let mut prefix_reserve = 0.0_f32;
                        let mut suffix_reserve = 0.0_f32;
                        if !ctx.prefix.is_empty() {
                            let pw = f.text_width(ctx.prefix, affix_style).min((text_rect.width - pad_x - pad_end).max(0.0));
                            if pw > 0.0 {
                                let px = if state.rtl {
                                    text_rect.x + text_rect.width - pad_x - pw
                                } else {
                                    text_rect.x + pad_x
                                };
                                f.styled_text(Rect { x: px, y: text_rect.y, width: pw, height: text_rect.height }, ctx.prefix, affix_color, affix_style);
                                prefix_reserve = pw + state.dp(2.0);
                            }
                        }
                        if !ctx.suffix.is_empty() {
                            let sw = f.text_width(ctx.suffix, affix_style).min((text_rect.width - pad_x - pad_end - prefix_reserve).max(0.0));
                            if sw > 0.0 {
                                let sx = if state.rtl {
                                    text_rect.x + pad_end
                                } else {
                                    text_rect.x + text_rect.width - pad_end - sw
                                };
                                f.styled_text(Rect { x: sx, y: text_rect.y, width: sw, height: text_rect.height }, ctx.suffix, affix_color, affix_style);
                                suffix_reserve = sw + state.dp(2.0);
                            }
                        }
                        let (eff_text_rect, eff_pad_x, eff_style) = if prefix_reserve > 0.0 || suffix_reserve > 0.0 {
                            let left_in = if state.rtl { pad_end + suffix_reserve } else { pad_x + prefix_reserve };
                            let right_in = if state.rtl { pad_x + prefix_reserve } else { pad_end + suffix_reserve };
                            let r = Rect {
                                x: text_rect.x + left_in,
                                y: text_rect.y,
                                width: (text_rect.width - left_in - right_in).max(0.0),
                                height: text_rect.height,
                            };
                            (r, 0.0, TextStyle { padding: 0.0, ..style })
                        } else {
                            (text_rect, pad_x, style)
                        };
                        if !text.is_empty() {
                            let text_color = self
                                .resolved_state_color(self.text_color_states, self.color, state, &key, response)
                                .unwrap_or(t.text)
                                .opacity(if response.enabled { 1.0 } else { 0.38 });
                            self.paint_edit_text(
                                f,
                                state,
                                eff_text_rect,
                                &display,
                                text_color,
                                eff_style,
                                eff_pad_x,
                                v_pad_top,
                                v_pad_bottom,
                                is_multiline,
                                edit_links_spec.map(|ls| (ls, edit_link_color, response.enabled, id, &mut edit_clicked_link, response.clicked)),
                            );
                        } else if !ctx.has_label && !self.hint.is_empty() {
                            let hint_color = t.on_surface_variant.opacity(if response.enabled { 1.0 } else { 0.38 });
                            self.paint_edit_text(
                                f,
                                state,
                                eff_text_rect,
                                self.hint,
                                hint_color,
                                eff_style,
                                eff_pad_x,
                                v_pad_top,
                                v_pad_bottom,
                                is_multiline,
                                None,
                            );
                        }
                    }));
                } else {
                let standalone_v_pad = state.dp(12.0);
                let standalone_usable_h = (inner.height
                    - if self.padding_explicit[1] { 0.0 } else { standalone_v_pad }
                    - if self.padding_explicit[3] { 0.0 } else { standalone_v_pad })
                .max(0.0);
                let standalone_multiline = !self.single_line
                    && self.max_lines != 1
                    && (self.min_lines > 1
                        || display.contains('\n')
                        || (!matches!(self.kind, Kind::AutoComplete | Kind::MultiAutoComplete)
                            && standalone_usable_h >= line_h * 1.9));
                match state.theme_style {
                    AndroidTheme::Bhoo => {
                        f.within(inner, |f| f.with_enabled(self.enabled, |f| {
                            let height = if standalone_multiline { inner.height } else { inner.height.min(field_max_h) };
                            let top_off = state.dp(16.0);
                            view_response = f.text_field(id, self.hint, height, !text.is_empty(), false, true);
                            if !text.is_empty() {
                                let bhoo_size = if (self.text_size - 16.0).abs() > 0.001 {
                                    text_size
                                } else {
                                    state.sp(TextStyle::default().size)
                                };
                                let pad_x = state.dp(12.);
                                let align = self.resolved_text_alignment(state.rtl).unwrap_or(if state.rtl { 2 } else { 0 });
                                let text_rect = Rect { y: inner.y + top_off, height: (height - top_off).max(0.0), ..inner };
                                self.paint_edit_text(
                                    f,
                                    state,
                                    text_rect,
                                    &display,
                                    t.text.opacity(if self.enabled { 1.0 } else { 0.38 }),
                                    TextStyle {
                                        size: bhoo_size,
                                        weight: if self.text_bold { 700 } else { 400 },
                                        align,
                                        padding: pad_x,
                                        letter_spacing: self.letter_spacing_em * bhoo_size,
                                        italic,
                                        family,
                                        ..TextStyle::default()
                                    },
                                    pad_x,
                                    state.dp(4.0),
                                    state.dp(6.0),
                                    standalone_multiline,
                                    edit_links_spec.map(|ls| (ls, edit_link_color, view_response.enabled, id, &mut edit_clicked_link, view_response.clicked)),
                                );
                            }
                        }));
                    }
                    AndroidTheme::MaterialLight | AndroidTheme::MaterialDark => {
                        f.within(inner, |f| f.with_enabled(self.enabled, |f| {
                            let height = if standalone_multiline { inner.height } else { inner.height.min(field_max_h) };
                            let field_bounds = Rect { height, ..inner };
                            let response = f.target(id, field_bounds, self.hint, self.enabled, Role::TextField);
                            view_response = response;
                            let focused = response.focused && response.enabled;
                            if !has_explicit_bg {
                                let thickness = state.dp(if focused { 2.0 } else { 1.0 });
                                let unfocused_color = match state.theme_style {
                                    AndroidTheme::MaterialLight => Color(0x00, 0x00, 0x00, 0x6B),
                                    _ => Color(0xFF, 0xFF, 0xFF, 0x8A),
                                };
                                let line_color = if !response.enabled {
                                    unfocused_color.opacity(0.38)
                                } else if focused {
                                    t.primary
                                } else {
                                    unfocused_color
                                };
                                let inset_x = state.dp(4.0).min(field_bounds.width * 0.25);
                                f.paint_rect(
                                    Rect {
                                        x: field_bounds.x + inset_x,
                                        y: field_bounds.y + field_bounds.height - thickness,
                                        width: (field_bounds.width - inset_x * 2.0).max(0.0),
                                        height: thickness,
                                    },
                                    0.0,
                                    line_color,
                                );
                            }
                            let align = self.resolved_text_alignment(state.rtl).unwrap_or(if state.rtl { 2 } else { 0 });
                            let pad_x = state.dp(4.0);
                            let style = TextStyle {
                                size: text_size,
                                weight: if self.text_bold { 700 } else { 400 },
                                align,
                                padding: pad_x,
                                letter_spacing: self.letter_spacing_em * text_size,
                                italic,
                                family,
                                ..TextStyle::default()
                            };
                            if !text.is_empty() {
                                let text_color = self.resolved_state_color(self.text_color_states, self.color, state, &key, response)
                                    .unwrap_or(t.text)
                                    .opacity(if response.enabled { 1.0 } else { 0.38 });
                                self.paint_edit_text(
                                    f,
                                    state,
                                    field_bounds,
                                    &display,
                                    text_color,
                                    style,
                                    pad_x,
                                    state.dp(8.0),
                                    state.dp(8.0),
                                    standalone_multiline,
                                    edit_links_spec.map(|ls| (ls, edit_link_color, response.enabled, id, &mut edit_clicked_link, response.clicked)),
                                );
                            } else if !self.hint.is_empty() {
                                let hint_color = match state.theme_style {
                                    AndroidTheme::MaterialLight => Color(0x00, 0x00, 0x00, 0x61),
                                    _ => Color(0xFF, 0xFF, 0xFF, 0x80),
                                }.opacity(if response.enabled { 1.0 } else { 0.38 });
                                self.paint_edit_text(
                                    f,
                                    state,
                                    field_bounds,
                                    self.hint,
                                    hint_color,
                                    style,
                                    pad_x,
                                    state.dp(8.0),
                                    state.dp(8.0),
                                    standalone_multiline,
                                    None,
                                );
                            }
                        }));
                    }
                    AndroidTheme::Material3Light | AndroidTheme::Material3Dark => {
                        f.within(inner, |f| f.with_enabled(self.enabled, |f| {
                            let height = if standalone_multiline { inner.height } else { inner.height.min(field_max_h) };
                            let field_bounds = Rect { height, ..inner };
                            let response = f.target(id, field_bounds, self.hint, self.enabled, Role::TextField);
                            view_response = response;
                            let focused = response.focused && response.enabled;
                            if !has_explicit_bg {
                                let r4 = state.dp(4.0);
                                let container_color = if response.enabled { t.surface_highest } else { t.text.opacity(0.04) };
                                f.paint_rect(field_bounds, r4, container_color);
                                let stroke_w = state.dp(if focused { 2.0 } else { 1.0 });
                                let stroke_color = if !response.enabled {
                                    t.text.opacity(0.12)
                                } else if focused {
                                    t.primary
                                } else {
                                    t.outline
                                };
                                f.commands.push(DrawCommand::Outline {
                                    bounds: field_bounds,
                                    radius: r4,
                                    width: stroke_w,
                                    color: stroke_color,
                                });
                                let ind_h = state.dp(if focused { 2.0 } else { 1.0 });
                                let ind_color = if !response.enabled {
                                    t.text.opacity(0.38)
                                } else if focused {
                                    t.primary
                                } else {
                                    t.on_surface_variant
                                };
                                f.paint_rect(
                                    Rect {
                                        x: field_bounds.x,
                                        y: field_bounds.y + field_bounds.height - ind_h,
                                        width: field_bounds.width,
                                        height: ind_h,
                                    },
                                    0.0,
                                    ind_color,
                                );
                            }
                            let align = self.resolved_text_alignment(state.rtl).unwrap_or(if state.rtl { 2 } else { 0 });
                            let pad12 = state.dp(12.0);
                            if !self.hint.is_empty() {
                                if text.is_empty() && !focused {
                                    self.paint_edit_text(
                                        f,
                                        state,
                                        field_bounds,
                                        self.hint,
                                        t.on_surface_variant.opacity(if response.enabled { 1.0 } else { 0.38 }),
                                        TextStyle { size: text_size, align, padding: pad12, italic, family, ..TextStyle::default() },
                                        pad12,
                                        state.dp(12.0),
                                        state.dp(12.0),
                                        standalone_multiline,
                                        None,
                                    );
                                } else {
                                    let label_color = if !response.enabled {
                                        t.text.opacity(0.38)
                                    } else if focused {
                                        t.primary
                                    } else {
                                        t.on_surface_variant
                                    };
                                    f.styled_text(
                                        Rect { y: field_bounds.y + state.dp(4.0), height: state.dp(14.0), ..field_bounds },
                                        self.hint,
                                        label_color,
                                        TextStyle { size: state.sp(12.0), align, padding: pad12, italic, family, ..TextStyle::default() },
                                    );
                                }
                            }
                            if !text.is_empty() {
                                let text_color = self.resolved_state_color(self.text_color_states, self.color, state, &key, response)
                                    .unwrap_or(t.text)
                                    .opacity(if response.enabled { 1.0 } else { 0.38 });
                                let top_off = state.dp(16.0);
                                let text_rect = if self.hint.is_empty() {
                                    field_bounds
                                } else {
                                    Rect { y: field_bounds.y + top_off, height: (field_bounds.height - top_off).max(0.0), ..field_bounds }
                                };
                                let v_pad_top = if self.hint.is_empty() { state.dp(12.0) } else { state.dp(4.0) };
                                self.paint_edit_text(
                                    f,
                                    state,
                                    text_rect,
                                    &display,
                                    text_color,
                                    TextStyle {
                                        size: text_size,
                                        weight: if self.text_bold { 700 } else { 400 },
                                        align,
                                        padding: pad12,
                                        letter_spacing: self.letter_spacing_em * text_size,
                                        italic,
                                        family,
                                        ..TextStyle::default()
                                    },
                                    pad12,
                                    v_pad_top,
                                    state.dp(8.0),
                                    standalone_multiline,
                                    edit_links_spec.map(|ls| (ls, edit_link_color, response.enabled, id, &mut edit_clicked_link, response.clicked)),
                                );
                            }
                        }));
                    }
                }
                }
                if let Some(link) = edit_clicked_link {
                    state.last_clicked_link = Some(link);
                    action = Some(self.id);
                }
                if matches!(self.kind,Kind::AutoComplete|Kind::MultiAutoComplete) && self.enabled && f.enabled && (f.focused_id()==Some(id) || state.autocomplete_open.as_deref()==Some(key.as_str())) {
                    let rule=|kind|self.relative_rules.iter().find(|r|r.kind==kind).map(|r|r.anchor);
                    state.autocomplete_anchor=Some(AutoAnchor {key,id,xml_id:self.id,bounds:Rect {height:inner.height.min(field_max_h),..inner},threshold:rule(97).and_then(|v|v.parse().ok()).unwrap_or(2),width:rule(85).unwrap_or("wrap_content"),height:rule(98).unwrap_or("wrap_content"),offset_x:rule(86).and_then(|v|v.parse().ok()).unwrap_or(0.0),offset_y:rule(87).and_then(|v|v.parse().ok()).unwrap_or(0.0),rtl:state.rtl,multi:matches!(self.kind,Kind::MultiAutoComplete),search:false});
                }
            }
            Kind::MaterialDivider => {
                let inset=|kind| state.dp(self.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|r.anchor.parse::<f32>().ok()).unwrap_or(0.0));
                f.divider_styled_at(b,crate::DividerOptions {inset_start:inset(55),inset_end:inset(56),thickness:b.height,rtl:state.rtl,color:self.divider_color,..crate::DividerOptions::default()});
            }
            Kind::View | Kind::Space => {
                if is_drag_handle {
                    let hit = f.target(
                        id,
                        b,
                        if self.text.is_empty() { "Drag handle" } else { self.text },
                        self.enabled,
                        Role::Button,
                    );
                    self.paint_state_layer(
                        f,
                        hit,
                        state.dp(24.0).min(b.width.min(b.height) * 0.5),
                        None,
                        t.on_surface_variant,
                    );
                    f.focus_ring(hit);
                    let pw = state.dp(32.0).min(inner.width);
                    let ph = state.dp(4.0).min(inner.height);
                    if pw > 0.0 && ph > 0.0 {
                        let pill = Rect {
                            x: inner.x + (inner.width - pw) * 0.5,
                            y: inner.y + (inner.height - ph) * 0.5,
                            width: pw,
                            height: ph,
                        };
                        let pill_color = background_tint
                            .unwrap_or_else(|| t.on_surface_variant.opacity(if enabled { 0.4 } else { 0.2 }));
                        f.paint_rect(pill, ph * 0.5, pill_color);
                    }
                    if hit.clicked {
                        if state.active_bottom_sheet.is_some() {
                            state.drag_handle_toggled = true;
                        }
                        action = Some(self.id);
                    }
                }
            }
            _ => {}
        }
        *view_response_out = view_response;
        *action_out = action;
    }
}
