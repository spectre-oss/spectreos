use super::*;

impl Node {
    /// Set text in a TextSwitcher, optionally switching to its next child.
    /// The switcher must have a stable XML ID when nested in another layout.
    pub fn set_switcher_text(&self, state: &mut LayoutState, text: &str, advance: bool) -> bool {
        if !matches!(self.kind, Kind::TextSwitcher) || self.children.is_empty() { return false; }
        let key = if self.id.is_empty() { format!("#{}", view_id(self.id)) } else { self.id.to_string() };
        let selected = state.displayed_children.entry(key).or_default();
        if *selected >= self.children.len() { *selected = 0; }
        if advance { *selected = (*selected + 1) % self.children.len(); }
        let child = &self.children[*selected];
        let child_key = if child.id.is_empty() { format!("#{}", anonymous_child_id(view_id(self.id), *selected)) } else { child.id.to_string() };
        state.text.insert(child_key, text.to_owned());
        true
    }
    /// Set the image of the next ImageSwitcher child and select it.
    pub fn set_switcher_image(&self, state: &mut LayoutState, source: &str) -> bool {
        if !matches!(self.kind, Kind::ImageSwitcher) || self.children.is_empty() || !state.images.contains_key(source) { return false; }
        let key = if self.id.is_empty() { format!("#{}", view_id(self.id)) } else { self.id.to_string() };
        let selected = state.displayed_children.entry(key).or_default();
        if *selected >= self.children.len() { *selected = 0; }
        *selected = (*selected + 1) % self.children.len();
        let child = &self.children[*selected];
        let child_key = if child.id.is_empty() { format!("#{}", anonymous_child_id(view_id(self.id), *selected)) } else { child.id.to_string() };
        state.image_sources.insert(child_key, source.to_owned());
        true
    }
    /// Paint an explicitly shown floating MediaController at the bottom of its anchor.
    pub fn render_floating_media_controller(&'static self,f:&mut Frame<'_>,anchor:Rect,height:f32,state:&mut LayoutState)->Option<&'static str> {
        if !matches!(self.kind,Kind::MediaController)||self.id.is_empty()||!height.is_finite(){return None;}
        let key=self.id;let now=f.time();let slider_id=anonymous_child_id(view_id(key),view_id("mediacontroller_progress") as usize);
        let mut floating=state.media_floating.remove(key)?;if !floating.visible{return None;}
        let was_dragging=floating.dragging;let dragging_before=f.ui.active==Some(slider_id);
        if !dragging_before&&!was_dragging&&floating.deadline.is_some_and(|deadline|now>=deadline){return None;}
        let h=height.max(0.0).min(anchor.height);let bounds=Rect{x:anchor.x,y:anchor.y+anchor.height-h,width:anchor.width,height:h};
        let start=f.commands.len();let action=self.render(f,bounds,state);f.ui.overlays.extend(f.commands.drain(start..));
        let dragging=f.ui.active==Some(slider_id)&&!f.input.released&&!f.input.cancelled&&!f.input.blurred;floating.dragging=dragging;
        if action.is_some()||was_dragging&&!dragging {floating.deadline=Some(now+3.0);}
        if dragging {f.continuous(bounds);} else if let Some(deadline)=floating.deadline {f.wake_at(deadline);}
        state.media_floating.insert(key.to_owned(),floating);action
    }
    pub fn render(
        &'static self,
        f: &mut Frame<'_>,
        bounds: Rect,
        state: &mut LayoutState,
    ) -> Option<&'static str> {
        let previous_theme = state
            .theme_style
            .bhoo_theme()
            .map(|theme| std::mem::replace(&mut f.ui.theme, theme));
        let action = self.render_inner(f, bounds, state);
        if let Some(theme) = previous_theme {
            f.ui.theme = theme;
        }
        action
    }
    pub(super) fn render_inner(
        &'static self,
        f: &mut Frame<'_>,
        bounds: Rect,
        state: &mut LayoutState,
    ) -> Option<&'static str> {
        state.frame_time = f.time();
        state.frame_reduced_motion = f.reduced_motion();
        state.frame_scroll_dragging = f.any_scroll_dragging();
        state.spinner_anchor=None;state.autocomplete_anchor=None;state.til_dropdown_anchor=None;
        state.search_bar_links.clear();
        collect_search_view_links(self, view_id(self.id), &mut state.search_bar_links);
        let mut action=None;
        let blocked=state.spinner_open.is_some() || state.spinner_closing.is_some();
        let popup_pointer=state.autocomplete_open.is_some() && state.autocomplete_panel.is_some_and(|panel|f.input.pointer.is_some_and(|point|panel.contains(point))) && (f.input.pressed || f.input.released);
        let popup_key=state.autocomplete_open.is_some() && f.input.key.is_some_and(|key|matches!(key.key,Key::Up|Key::Down|Key::Enter|Key::Escape));
        let til_popup_pointer = state.til_dropdown_open.is_some()
            && state.til_dropdown_panel.is_some_and(|panel| f.input.pointer.is_some_and(|point| panel.contains(point)))
            && (f.input.pressed || f.input.released);
        let til_popup_key = state.til_dropdown_open.is_some()
            && f.input.key.is_some_and(|key| matches!(key.key, Key::Escape));
        if let Some(window_bg) = state.theme_style.window_background() {
            let root_has_bg = self.background.is_some()
                || self.dynamic_drawable_bg().is_some()
                || self.has_animated_vector(false)
                || self.has_animated_selector(false)
                || self.relative_rules.iter().any(|r| {
                    r.kind == 96
                        || (!matches!(self.kind, Kind::Button) && r.kind == RULE_BACKGROUND_TINT)
                        || r.kind == RULE_DRAWABLE_LAYER
                        || r.kind == RULE_STATEFUL_DRAWABLE_LAYER
                        || r.kind == RULE_BACKGROUND_IMAGE
                        || r.kind == RULE_STATEFUL_BG_IMAGE
                        || (!self.is_image_src_kind() && r.kind == RULE_ANIMATION_LIST && parse_animation_list(r.anchor).is_some())
                        || (r.kind == RULE_GRADIENT && parse_gradient(r.anchor).is_some())
                });
            if !root_has_bg && bounds.width > 0.0 && bounds.height > 0.0 {
                f.paint_rect(bounds, 0.0, window_bg);
            }
        }
        state.at_root_draw = true;
        f.without_input(blocked || popup_pointer || popup_key || til_popup_pointer || til_popup_key,|f| {action=self.draw(f,bounds,state,view_id(self.id));});
        state.at_root_draw = false;
        if let Some(chosen)=render_autocomplete_popup(f,state) {action=Some(chosen);}
        if let Some(chosen)=render_til_dropdown_popup(f,state) {action=Some(chosen);}
        if let Some(SpinnerAnchor {key,id,bounds:anchor,xml_id,dialog,prompt,rtl,width:requested_width,offset_x,offset_y,padding})=state.spinner_anchor.clone() && (state.spinner_open.as_deref()==Some(key.as_str()) || dialog && state.spinner_closing.as_deref()==Some(key.as_str())) {
            let Some(entries)=state.spinner_items.get(&key) else {state.spinner_open=None;return action;};
            if entries.is_empty() {state.spinner_open=None;return action;}
            let flags=state.spinner_enabled.get(&key).cloned().unwrap_or_default();
            let enabled_at=|i:usize|flags.get(i).copied().unwrap_or(true);
            let templates=state.spinner_templates.get(&key).copied();
            let mut template_state=templates.map(|_|spinner_template_state(state,rtl));
            if dialog {
                let viewport=f.viewport();let title_height=if prompt.is_empty(){0.0}else{56.0};
                let width=(viewport.width-48.0).clamp(0.0,400.0);
                let rows=spinner_row_offsets(&mut state.spinner_row_cache,&key,entries,templates,&mut template_state,(width-32.0).max(0.0),f.ui.text_metrics,id,0.0);
                let list_height=rows[entries.len()].min((viewport.height-88.0-title_height).max(48.0));let height=(32.0+title_height+list_height).min(viewport.height);
                let dialog_bounds=Rect{x:viewport.x+(viewport.width-width)*0.5,y:viewport.y+(viewport.height-height)*0.5,width,height};
                let mut overlay=state.spinner_dialogs.remove(&key).unwrap_or_default();
                let was_closing=state.spinner_closing.as_deref()==Some(key.as_str());
                overlay.kind=OverlayKind::Dialog;overlay.open=state.spinner_open.as_deref()==Some(key.as_str());overlay.dismiss_policy=OverlayDismissPolicy::EscapeOrOutside;
                let selected=state.spinner_selected.get(&key).copied().unwrap_or(0).min(entries.len()-1);
                let highlighted=state.spinner_highlight.get(&key).copied().unwrap_or(selected).min(entries.len()-1);
                let mut choice=None;
                let scroll=state.spinner_scroll.entry(key.clone()).or_insert_with(|| {let mut physics=crate::scroll::ScrollPhysics::default();physics.offset=rows[highlighted].max(0.0);physics});
                f.overlay(anonymous_child_id(id,511),&mut overlay,dialog_bounds,true,|f,open| {
                    let content=f.available();
                    if !prompt.is_empty() {f.styled_text(Rect{x:content.x,y:content.y,width:content.width,height:title_height},prompt,f.theme().text,TextStyle {size:20.0,..TextStyle::default()});}
                    let list=Rect{x:content.x,y:content.y+title_height,width:content.width,height:(content.height-title_height).max(0.0)};
                    let max_scroll=(rows[entries.len()]-list.height).max(0.0);
                    if f.input.pressed && f.input.pointer_kind==PointerKind::Touch && f.pointer_in(list) && let Some(point)=f.input.pointer {scroll.begin(point[1],f.time());}
                    if scroll.touching() && let Some(point)=f.input.pointer {scroll.drag(point[1],f.time(),max_scroll,list.height);}
                    let scrolling=scroll.dragging;if scrolling {f.ui.active=None;}
                    if f.input.released && scroll.touching() {scroll.release(f.time(),f.reduced_motion());}
                    if f.pointer_in(list) && f.input.scroll[1]!=0.0 {scroll.wheel(f.input.scroll[1],max_scroll,f.reduced_motion());}
                    if scroll.advance(f.time(),max_scroll,f.reduced_motion()) {f.ui.continuous=true;}
                    let offset=scroll.offset.clamp(0.0,max_scroll);let (start,end)=spinner_visible_rows(&rows,offset,list.height);
                    f.without_input(scrolling,|f| f.within(list,|f| for (i,label) in entries.iter().enumerate().skip(start).take(end.saturating_sub(start)) {
                        let row=Rect{x:list.x,y:list.y+rows[i]-offset,width:list.width,height:rows[i+1]-rows[i]};
                        if i==highlighted && highlighted!=selected {f.paint_rect(row,0.0,f.theme().primary_container);}
                        let r=f.target(anonymous_child_id(id,i),row,label,enabled_at(i),Role::Button);f.state_layer(r,0.0,f.theme().text);
                        if let (Some(templates),Some(row_state))=(templates,template_state.as_mut()) {
                            paint_spinner_template(f,templates.dropdown,templates.text_id,label,row,row_state,anonymous_child_id(id,1024+i),r,i==selected,None);
                        } else {
                            let circle=Rect{x:row.x+8.0,y:row.y+14.0,width:20.0,height:20.0};let t=f.theme();
                            let alpha=if r.enabled {1.0}else{0.38};
                            f.paint_rect(circle,10.0,if i==selected {t.primary.opacity(alpha)}else{t.outline.opacity(alpha)});f.paint_rect(circle.inset(2.0),8.0,t.surface_high);
                            if i==selected {f.paint_rect(circle.inset(5.0),5.0,t.primary.opacity(alpha));}
                            f.styled_text(Rect{x:row.x+40.0,width:(row.width-40.0).max(0.0),..row},label,t.text.opacity(alpha),TextStyle::default());
                        }
                        if r.clicked {choice=Some(i);*open=false;}
                    }));
                    if blocked && (f.pressed_key(Key::Down)||f.pressed_key(Key::Up)) {
                        let next=spinner_next_enabled(&flags,entries.len(),highlighted,f.pressed_key(Key::Down)).unwrap_or(highlighted);
                        state.spinner_highlight.insert(key.clone(),next);let top=rows[next];let bottom=rows[next+1];
                        if top<scroll.offset {scroll.offset=top;}else if bottom>scroll.offset+list.height {scroll.offset=(bottom-list.height).min(max_scroll);}f.ui.continuous=true;
                    }
                    if blocked && f.pressed_key(Key::Enter) && enabled_at(highlighted) {choice=Some(highlighted);*open=false;}
                });
                if let Some(index)=choice {state.spinner_selected.insert(key.clone(),index);action=Some(xml_id);}
                if !overlay.open {
                    state.spinner_open=None;
                    if was_closing && !overlay.moving() {state.spinner_closing=None;} else {state.spinner_closing=Some(key.clone());}
                }
                state.spinner_dialogs.insert(key,overlay);
                return action;
            }
            let viewport=f.viewport();
            let inner_width=(anchor.width-padding[0]-padding[2]).max(0.0);
            let selected_for_width=state.spinner_selected.get(&key).copied().unwrap_or(0).min(entries.len()-1);
            let end=(selected_for_width+15).min(entries.len());let start=selected_for_width.saturating_sub(15-(end-selected_for_width));
            // Android measures up to 15 adapter *selected* views, including their XML padding.
            let measured=if requested_width=="wrap_content" {
                if let (Some(template),Some(row_state))=(templates,template_state.as_mut()) {
                    entries[start..end].iter().enumerate().map(|(i,label)| {
                        row_state.text.insert(template.text_id.to_owned(),label.clone());
                        template.selected.intrinsic(true,row_state,anonymous_child_id(id,1023+i))
                    }).fold(0.0_f32,f32::max)
                } else {entries[start..end].iter().map(|s|f.text_width(s,TextStyle::default())+32.0).fold(0.0_f32,f32::max)}
            } else {0.0};
            let wanted=match requested_width {"match_parent"=>inner_width,"wrap_content"=>inner_width.max(measured),v=>v.parse::<f32>().unwrap_or(inner_width)};
            let width=wanted.clamp(0.0,viewport.width.max(0.0));
            let rows=spinner_row_offsets(&mut state.spinner_row_cache,&key,entries,templates,&mut template_state,width,f.ui.text_metrics,id,0.0);
            let desired=rows[entries.len()].min(viewport.height);
            let below_start=(anchor.y+anchor.height+offset_y).clamp(viewport.y,viewport.y+viewport.height);
            let above_end=(anchor.y+offset_y).clamp(viewport.y,viewport.y+viewport.height);
            let below=(viewport.y+viewport.height-below_start).max(0.0);let above=(above_end-viewport.y).max(0.0);
            let use_below=below>=desired || below>=above;
            let height=desired.min(if use_below {below} else {above});let y=if use_below {below_start} else {above_end-height};
            let raw_x=if rtl {anchor.x+anchor.width-padding[2]-width-offset_x} else {anchor.x+padding[0]+offset_x};
            let panel=Rect{x:raw_x.clamp(viewport.x,viewport.x+(viewport.width-width).max(0.0)),y,width,height};
            if f.input.cancelled || f.input.blurred || f.pressed_key(Key::Escape) || f.input.pressed && !f.pointer_in(panel) {state.spinner_open=None;return action;}
            let selected=state.spinner_selected.get(&key).copied().unwrap_or(0).min(entries.len()-1);
            let highlighted=state.spinner_highlight.get(&key).copied().unwrap_or(selected).min(entries.len()-1);
            let max_scroll=(rows[entries.len()]-panel.height).max(0.0);
            let scroll=state.spinner_scroll.entry(key.clone()).or_insert_with(|| {let mut physics=crate::scroll::ScrollPhysics::default();physics.offset=rows[highlighted].clamp(0.0,max_scroll);physics});
            if f.input.pressed && f.input.pointer_kind==PointerKind::Touch && f.pointer_in(panel) && let Some(point)=f.input.pointer {scroll.begin(point[1],f.time());}
            if scroll.touching() && let Some(point)=f.input.pointer {scroll.drag(point[1],f.time(),max_scroll,panel.height);}
            let scrolling=scroll.dragging;
            if scrolling {f.ui.active=None;}
            if f.input.released && scroll.touching() {scroll.release(f.time(),f.reduced_motion());}
            if f.pointer_in(panel) && f.input.scroll[1]!=0.0 {scroll.wheel(f.input.scroll[1],max_scroll,f.reduced_motion());}
            if scroll.advance(f.time(),max_scroll,f.reduced_motion()) {f.ui.continuous=true;}
            let offset=scroll.offset.clamp(0.0,max_scroll);
            let (start,end)=spinner_visible_rows(&rows,offset,panel.height);
            f.elevation(panel,4.0,2);f.paint_rect(panel,4.0,f.theme().surface_highest);
            f.without_input(scrolling,|f|f.within(panel,|f| for (i,label) in entries.iter().enumerate().skip(start).take(end.saturating_sub(start)) {
                let r=Rect{x:panel.x,y:panel.y+rows[i]-offset,width:panel.width,height:rows[i+1]-rows[i]};
                let response=f.target(anonymous_child_id(id,i),r,label,enabled_at(i),Role::Button);
                if i==highlighted {f.paint_rect(r,0.0,f.theme().primary_container);}
                f.state_layer(response,0.0,f.theme().text);
                if let (Some(templates),Some(row_state))=(templates,template_state.as_mut()) {
                    paint_spinner_template(f,templates.dropdown,templates.text_id,label,r,row_state,anonymous_child_id(id,1024+i),response,i==selected,None);
                } else {f.styled_text(r,label,f.theme().text.opacity(if response.enabled {1.0}else{0.38}),TextStyle {padding:16.0,..TextStyle::default()});}
                if response.clicked {state.spinner_selected.insert(key.clone(),i);state.spinner_open=None;action=Some(xml_id);}
            }));
            if blocked && (f.pressed_key(Key::Down) || f.pressed_key(Key::Up)) {
                let next=spinner_next_enabled(&flags,entries.len(),highlighted,f.pressed_key(Key::Down)).unwrap_or(highlighted);
                state.spinner_highlight.insert(key.clone(),next);
                let item_top=rows[next];let item_bottom=rows[next+1];
                if item_top<scroll.offset {scroll.offset=item_top;} else if item_bottom>scroll.offset+panel.height {scroll.offset=(item_bottom-panel.height).min(max_scroll);}
                f.ui.continuous=true;
            }
            if blocked && f.pressed_key(Key::Enter) && enabled_at(highlighted) {state.spinner_selected.insert(key.clone(),highlighted);state.spinner_open=None;action=Some(xml_id);}
        } else if state.spinner_open.is_some() || state.spinner_closing.is_some() {state.spinner_open=None;state.spinner_closing=None;}
        action
    }
    pub(super) fn draw(
        &'static self,
        f: &mut Frame<'_>,
        b: Rect,
        state: &mut LayoutState,
        id: u64,
    ) -> Option<&'static str> {
        state.frame_time = f.time();
        state.frame_reduced_motion = f.reduced_motion();
        state.frame_scroll_dragging = f.any_scroll_dragging();
        let visible = self.effective_visibility(state) == Visibility::Visible
            && !matches!(self.kind, Kind::Space);
        let rt_scale = if visible && !self.id.is_empty() {
            state.view_scale.get(self.id).copied()
        } else {
            None
        };
        let rt_trans = if visible && !self.id.is_empty() {
            state.view_translation.get(self.id).copied()
        } else {
            None
        };
        let rt_alpha = if visible && !self.id.is_empty() {
            state.view_alpha.get(self.id).copied().unwrap_or(1.0)
        } else {
            1.0
        };
        let xml_tr = if visible { self.view_transform() } else { None };
        let pushed_transform = if visible && (xml_tr.is_some() || rt_scale.is_some() || rt_trans.is_some()) {
            let (origin, xml_tx, xml_ty, xml_sx, xml_sy, rot_deg) = if let Some(tr) = xml_tr {
                (
                    [
                        b.x + tr.px.resolve(b.width, state),
                        b.y + tr.py.resolve(b.height, state),
                    ],
                    tr.tx_dp,
                    tr.ty_dp,
                    tr.sx,
                    tr.sy,
                    tr.rot_deg,
                )
            } else {
                ([b.x + b.width * 0.5, b.y + b.height * 0.5], 0.0, 0.0, 1.0, 1.0, 0.0)
            };
            let [rtx, rty] = rt_trans.unwrap_or([0.0, 0.0]);
            let [rsx, rsy] = rt_scale.unwrap_or([1.0, 1.0]);
            let tx = xml_tx + rtx;
            let ty = xml_ty + rty;
            let sx = xml_sx * rsx;
            let sy = xml_sy * rsy;
            if tx.abs() > 1e-4
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
            }
        } else {
            false
        };
        let effective_alpha = (self.view_alpha * rt_alpha).clamp(0.0, 1.0);
        let pushed_layer = if visible && effective_alpha < 1.0 {
            let opacity = effective_alpha;
            let has_elev = self.relative_rules.iter().any(|r| r.kind == RULE_ELEVATION);
            let layer_bounds = if self.clip_children_disabled || self.clip_to_padding_disabled {
                let min_x = b.x.min(f.clip.x);
                let min_y = b.y.min(f.clip.y);
                let max_x = (b.x + b.width).max(f.clip.x + f.clip.width);
                let max_y = (b.y + b.height).max(f.clip.y + f.clip.height);
                Rect {
                    x: min_x,
                    y: min_y,
                    width: (max_x - min_x).max(0.0),
                    height: (max_y - min_y).max(0.0),
                }
            } else if has_elev {
                let pad = state.dp(24.0);
                Rect {
                    x: b.x - pad,
                    y: b.y - pad,
                    width: b.width + pad * 2.0,
                    height: b.height + pad * 2.0,
                }
                .intersect(f.clip)
            } else {
                b
            };
            f.push_layer(layer_bounds, opacity);
            true
        } else {
            false
        };
        let fade=if matches!(self.kind,Kind::ZoomControls) {
            let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
            state.zoom_controls_fades.get(&key).copied().map(|fade|(key,fade))
        } else { None };
        let mut action=if let Some((key,fade))=fade {
            let done=f.reduced_motion() || f.time()-fade.start>=0.5;
            let opacity=if done { f32::from(fade.target_visible) } else { fade.value(f.time()) };
            let mut action=None;
            f.with_opacity(opacity,|f| f.without_input(!fade.target_visible,|f| { action=self.draw_contents(f,b,state,id,false); }));
            if done {
                state.zoom_controls_fades.remove(&key);
                if !fade.target_visible {
                    state.visibility.insert(key,Visibility::Gone);
                    // One more frame removes the now-gone view from the parent's layout.
                    f.wake_at(f.time()+0.001);
                }
            } else { f.continuous(b); }
            action
        } else { self.draw_contents(f,b,state,id,false) };
        if self.id=="android:tabcontent"&&matches!(self.kind,Kind::Frame)&&self.effective_visibility(state)==Visibility::Visible {
            if let Some(host_id)=state.active_tab_host.clone() {if let Some(host)=state.tab_hosts.get(&host_id).cloned() {if let Some(factory)=host.factories.get(host.selected).copied().flatten() {
                let cache_key=(host_id,host.selected);let node=if let Some(node)=state.tab_host_factory_cache.get(&cache_key){*node}else{let node=factory(&host.tags[host.selected]);state.tab_host_factory_cache.insert(cache_key,node);node};
                let mut child_action=None;f.within(b,|f|{child_action=node.draw(f,b,state,child_view_id(id,host.selected,node));});if child_action.is_some(){action=child_action;}
            }}}
        }
        if !self.is_search_view_closed(state, id) {
            self.draw_post(f,b,state);
        }
        if pushed_layer {
            f.pop_layer();
        }
        if pushed_transform {
            f.pop_transform();
        }
        action
    }
    pub(super) fn draw_post(&self,f:&mut Frame<'_>,b:Rect,state:&LayoutState) {
        if self.effective_visibility(state)==Visibility::Visible && !matches!(self.kind,Kind::Space) {
            let padding=self.resolved_view_padding(state);
            let area=if self.foreground_excludes_padding { Rect { x:b.x+padding[0],y:b.y+padding[1],width:(b.width-padding[0]-padding[2]).max(0.0),height:(b.height-padding[1]-padding[3]).max(0.0) } } else { b };
            let rtl=match self.layout_direction { 1=>false,2=>true,_=>state.rtl };
            let [h,v]=self.foreground_gravity;
            let width=if h==5 { area.width } else { state.dp(self.foreground_size[0].max(0.0)) };
            let height=if v==5 { area.height } else { state.dp(self.foreground_size[1].max(0.0)) };
            let right=match h {0=>rtl,2=>!rtl,4=>true,_=>false};
            let x=if h==1 { area.x+(area.width-width)*0.5 } else if right { area.x+area.width-width } else { area.x };
            let y=match v {1=>area.y+(area.height-height)*0.5,2=>area.y+area.height-height,_=>area.y};
            let foreground_bounds=Rect { x,y,width,height };
            let fg_radii = (self.foreground_radius == 0.0)
                .then(|| self.resolved_corner_radii(state))
                .flatten();
            let fg_radius=if self.foreground_radius<0.0 { self.foreground_radius } else { state.dp(self.foreground_radius) };
            if let Some(color)=self.foreground {
                if let Some(radii) = fg_radii {
                    paint_rounded_rect_radii(f, foreground_bounds, radii, color);
                } else if fg_radius<0.0 {
                    paint_oval(f,foreground_bounds,color);
                } else {
                    f.paint_rect(foreground_bounds,fg_radius,color);
                }
            }
            if let Some(color)=self.foreground_stroke {
                let sw = state.dp(self.foreground_stroke_width);
                if let Some(radii) = fg_radii {
                    paint_outline_radii(f, foreground_bounds, radii, sw, color);
                } else {
                    f.commands.push(DrawCommand::Outline { bounds:foreground_bounds,radius:fg_radius,width:sw,color });
                }
            }
        }
    }
    pub(super) fn draw_recycler_ext(
        &'static self,
        f: &mut Frame<'_>,
        inner: Rect,
        state: &mut LayoutState,
        id: u64,
        ext: RecyclerLayoutExt,
        t: Theme,
    ) -> Option<&'static str> {
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        let template = state.list_templates.get(&key).copied();
        let mut row_state = if template.is_some() || state.list_item_bindings.contains_key(&key) {
            Some(spinner_template_state(state, state.rtl))
        } else {
            None
        };
        let d = state.effective_density();
        let divider = state.dp(self.divider_thickness.max(0.0));
        let h_gap = self
            .relative_rules
            .iter()
            .find(|r| r.kind == 93)
            .and_then(|r| r.anchor.parse::<f32>().ok())
            .unwrap_or(0.0)
            .max(0.0)
            * d;
        let v_gap = self
            .relative_rules
            .iter()
            .find(|r| r.kind == 94)
            .and_then(|r| r.anchor.parse::<f32>().ok())
            .unwrap_or(0.0)
            .max(0.0)
            * d;
        let main_gap = match ext.manager {
            RecyclerLayoutManager::Linear => {
                if divider > 0.0 {
                    divider
                } else if ext.horizontal {
                    h_gap
                } else {
                    v_gap
                }
            }
            RecyclerLayoutManager::Grid | RecyclerLayoutManager::Staggered => {
                if ext.horizontal { h_gap.max(divider) } else { v_gap.max(divider) }
            }
        };
        let cross_gap = if ext.horizontal { v_gap } else { h_gap };
        let span = match ext.manager {
            RecyclerLayoutManager::Linear => 1,
            RecyclerLayoutManager::Grid | RecyclerLayoutManager::Staggered => ext.span_count.clamp(1, 256),
        };
        let entries = state.list_items.entry(key.clone()).or_insert_with(|| {
            self.relative_rules
                .iter()
                .filter(|r| r.kind == 82)
                .map(|r| r.anchor.to_owned())
                .collect()
        });
        let entry_count = entries.len();
        let checked = state.list_checked.entry(key.clone()).or_default();
        checked.retain(|i| *i < entry_count);
        let flags = state.list_enabled.get(&key);
        let item_ids = state.list_item_ids.get(&key);
        let bindings = state.list_item_bindings.get(&key);
        let mode = self.relative_rules.iter().find(|r| r.kind == 88).map_or("none", |r| r.anchor);
        let metrics = f.ui.text_metrics;
        let cross_avail = if ext.horizontal { inner.height } else { inner.width };
        let track_size = ((cross_avail - (span - 1) as f32 * cross_gap) / span as f32).max(1.0);
        let visual_order: Vec<usize> = if ext.reverse {
            (0..entry_count).rev().collect()
        } else {
            (0..entry_count).collect()
        };
        let measure_item = |index: usize, row_state: &mut Option<LayoutState>| -> (f32, f32) {
            let label = &entries[index];
            let binding = bindings.and_then(|items| items.get(&index));
            let item_tmpl = adapter_item_template(template, binding);
            let cid = anonymous_child_id(id, 1024 + index);
            if let (Some(tmpl), Some(rs)) = (item_tmpl, row_state.as_mut()) {
                with_adapter_binding(rs, tmpl.text_id, label, binding, |rs| {
                    if ext.horizontal {
                        let h = match tmpl.row.height {
                            Size::Match => track_size,
                            Size::Dp(v) => rs.dp(v).min(track_size),
                            Size::Wrap => tmpl.row.layout_intrinsic(false, rs, inner.width.max(48.0), metrics, cid).min(track_size),
                        }
                        .max(1.0);
                        let w = match tmpl.row.width {
                            Size::Match => rs.dp(120.0).min(inner.width.max(48.0)),
                            Size::Dp(v) => rs.dp(v),
                            Size::Wrap => tmpl.row.layout_intrinsic(true, rs, inner.width.max(48.0), metrics, cid),
                        };
                        (if w.is_finite() { w.clamp(1.0, 4096.0) } else { rs.dp(120.0) }, h)
                    } else {
                        let w = match tmpl.row.width {
                            Size::Match => track_size,
                            Size::Dp(v) => rs.dp(v).min(track_size),
                            Size::Wrap => tmpl.row.layout_intrinsic(true, rs, track_size, metrics, cid).min(track_size),
                        }
                        .max(1.0);
                        let h = match tmpl.row.height {
                            Size::Match => rs.dp(48.0),
                            Size::Dp(v) => rs.dp(v),
                            Size::Wrap => tmpl.row.layout_intrinsic(false, rs, w, metrics, cid),
                        };
                        (w, if h.is_finite() { h.clamp(1.0, 4096.0) } else { rs.dp(48.0) })
                    }
                })
            } else if ext.horizontal {
                let tw = metrics.width(label, TextStyle::default());
                let w = (tw.ceil() + 34.0).clamp(48.0, 4096.0);
                (w, track_size)
            } else {
                let lines = wrapped_text_lines(label, (track_size - 32.0).max(1.0), TextStyle::default(), metrics, |_| {}).max(1);
                (track_size, (24.0 * lines as f32).clamp(48.0, 4096.0))
            }
        };
        // Each tuple: (adapter_index, main_pos, cross_pos, item_w, item_h)
        let mut placed: Vec<(usize, f32, f32, f32, f32)> = Vec::with_capacity(entry_count);
        let content_extent = match ext.manager {
            RecyclerLayoutManager::Linear => {
                let mut cursor = 0.0_f32;
                for &idx in &visual_order {
                    let (w, h) = measure_item(idx, &mut row_state);
                    let main_len = if ext.horizontal { w } else { h };
                    placed.push((idx, cursor, 0.0, w, h));
                    cursor += main_len + main_gap;
                }
                if entry_count == 0 { 0.0 } else { (cursor - main_gap).max(0.0) }
            }
            RecyclerLayoutManager::Grid => {
                let mut cursor = 0.0_f32;
                for chunk in visual_order.chunks(span) {
                    let mut chunk_max = 0.0_f32;
                    let mut sizes = Vec::with_capacity(chunk.len());
                    for &idx in chunk {
                        let (w, h) = measure_item(idx, &mut row_state);
                        chunk_max = chunk_max.max(if ext.horizontal { w } else { h });
                        sizes.push((idx, w, h));
                    }
                    for (lane, (idx, w, h)) in sizes.into_iter().enumerate() {
                        let cross_pos = lane as f32 * (track_size + cross_gap);
                        placed.push((idx, cursor, cross_pos, w, h));
                    }
                    cursor += chunk_max + main_gap;
                }
                if entry_count == 0 { 0.0 } else { (cursor - main_gap).max(0.0) }
            }
            RecyclerLayoutManager::Staggered => {
                let mut track_ends = vec![0.0_f32; span];
                let mut track_counts = vec![0usize; span];
                for &idx in &visual_order {
                    let mut best = 0usize;
                    let mut best_end = track_ends[0];
                    for (lane, &end_val) in track_ends.iter().enumerate().skip(1) {
                        if end_val + 0.01 < best_end {
                            best = lane;
                            best_end = end_val;
                        }
                    }
                    let (w, h) = measure_item(idx, &mut row_state);
                    let main_len = if ext.horizontal { w } else { h };
                    let main_pos = if track_counts[best] > 0 {
                        track_ends[best] + main_gap
                    } else {
                        0.0
                    };
                    let cross_pos = best as f32 * (track_size + cross_gap);
                    placed.push((idx, main_pos, cross_pos, w, h));
                    track_ends[best] = main_pos + main_len;
                    track_counts[best] += 1;
                }
                track_ends.into_iter().fold(0.0_f32, f32::max)
            }
        };
        let viewport_main = if ext.horizontal { inner.width } else { inner.height };
        let max_scroll = (content_extent - viewport_main).max(0.0);
        let offset = state.offsets.entry(key.clone()).or_default();
        if !offset.is_finite() {
            *offset = 0.0;
        }
        let rtl = state.rtl;
        let dragging = if ext.horizontal {
            f.scroll_viewport_horizontal_rtl(id, inner, offset, max_scroll, rtl)
        } else {
            f.scroll_viewport(id, inner, offset, max_scroll)
        };
        let scroll = *offset;
        let pad_start = if ext.stack_end {
            (viewport_main - content_extent).max(0.0)
        } else {
            0.0
        };
        let long_enabled = state.list_long_click_enabled.get(&key).copied().unwrap_or(false);
        let hold = state.list_holds.entry(key.clone()).or_default();
        if !long_enabled
            || dragging
            || f.input.cancelled
            || f.input.blurred
            || f.input.released
            || f.input.scroll != [0.0, 0.0]
        {
            *hold = None;
        }
        let mut chosen = None;
        let mut long_choice = None;
        let mut control_choice = None;
        let row_state_environment = row_state.as_ref().cloned();
        let control_states = &mut state.list_control_states;
        let (stretch_s, stretch_sign) = f.scroll_stretch(id);
        f.cancel_pointer_input(dragging, |f| {
            f.within(inner, |f| {
                f.with_edge_stretch(inner, ext.horizontal, stretch_s, stretch_sign, |f| {
                for (vis_pos, &(index, main_pos, cross_pos, item_w, item_h)) in placed.iter().enumerate() {
                    let bounds = if ext.horizontal {
                        let logical_x = pad_start + main_pos - scroll;
                        let x = if rtl {
                            inner.x + inner.width - logical_x - item_w
                        } else {
                            inner.x + logical_x
                        };
                        Rect { x, y: inner.y + cross_pos, width: item_w, height: item_h }
                    } else {
                        let x = if rtl {
                            inner.x + inner.width - cross_pos - item_w
                        } else {
                            inner.x + cross_pos
                        };
                        Rect { x, y: inner.y + pad_start + main_pos - scroll, width: item_w, height: item_h }
                    };
                    let clip_vis = bounds.intersect(inner);
                    if clip_vis.width <= 0.0 || clip_vis.height <= 0.0 {
                        continue;
                    }
                    if checked.contains(&index) {
                        f.paint_rect(bounds, 0.0, t.primary_container);
                    }
                    let item_key = item_ids
                        .and_then(|ids| ids.get(index))
                        .map_or(index as u64, |value| *value as u64);
                    let enabled = self.enabled && flags.and_then(|v| v.get(index)).copied().unwrap_or(true);
                    let row_id = id.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(item_key);
                    let binding = bindings.and_then(|items| items.get(&index));
                    let item_template = adapter_item_template(template, binding);
                    let interactive = item_template
                        .is_some_and(|tmpl| inspect_expandable_template(tmpl.row, tmpl.text_id).2);
                    if interactive {
                        let tmpl = item_template.unwrap();
                        let controls = control_states.entry((key.clone(), item_key as i64)).or_default();
                        let mut local = row_state_environment.clone().unwrap_or_default();
                        local.adapter_namespace = Some(row_id);
                        local.rtl = rtl;
                        local.checked = std::mem::take(controls);
                        with_adapter_binding(&mut local, tmpl.text_id, &entries[index], binding, |rs| {
                            f.within(bounds, |f| {
                                f.with_enabled(enabled, |f| {
                                    if let Some(view) = tmpl.row.draw(f, bounds, rs, anonymous_child_id(row_id, 0)) {
                                        control_choice = Some((index, view));
                                    }
                                });
                            });
                        });
                        *controls = local.checked;
                    }
                    let r = f.target(row_id, bounds, &entries[index], enabled, Role::Button);
                    if adapter_item_long_press(f, r, hold, long_enabled) {
                        long_choice = Some(index);
                    }
                    f.state_layer(r, 0.0, t.text);
                    if interactive {
                    } else if let (Some(tmpl), Some(rs)) = (item_template, row_state.as_mut()) {
                        paint_spinner_template(
                            f,
                            tmpl.row,
                            tmpl.text_id,
                            &entries[index],
                            bounds,
                            rs,
                            anonymous_child_id(id, 1024 + index),
                            r,
                            checked.contains(&index),
                            binding,
                        );
                    } else {
                        let style = TextStyle { align: if rtl { 2 } else { 0 }, ..TextStyle::default() };
                        let text_w = (bounds.width - 32.0).max(1.0);
                        let lines = wrapped_text_lines(&entries[index], text_w, style, metrics, |_| {}).max(1);
                        let mut line_y = bounds.y + (bounds.height - lines as f32 * 24.0).max(0.0) * 0.5;
                        wrapped_text_lines(&entries[index], text_w, style, metrics, |line| {
                            f.styled_text(
                                Rect { x: bounds.x + 16.0, y: line_y, width: text_w, height: 24.0 },
                                &line.text,
                                t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
                                style,
                            );
                            line_y += 24.0;
                        });
                    }
                    if r.clicked {
                        chosen = Some(index);
                    }
                    if ext.manager == RecyclerLayoutManager::Linear
                        && vis_pos + 1 < placed.len()
                        && divider > 0.0
                        && let Some(color) = self.divider_color
                    {
                        if ext.horizontal {
                            let dx = if rtl { bounds.x - divider } else { bounds.x + bounds.width };
                            f.paint_rect(Rect { x: dx, y: bounds.y, width: divider, height: bounds.height }, 0.0, color);
                        } else {
                            f.paint_rect(
                                Rect { x: bounds.x, y: bounds.y + bounds.height, width: bounds.width, height: divider },
                                0.0,
                                color,
                            );
                        }
                    }
                }
                });
            });
        });
        let mut action = None;
        if let Some(control) = control_choice {
            state.list_control_clicked.insert(key.clone(), control);
            chosen = None;
            long_choice = None;
            action = Some(self.id);
        }
        if let Some(index) = long_choice {
            state.list_long_clicked.insert(key.clone(), index);
            action = Some(self.id);
        }
        if let Some(index) = chosen {
            match mode {
                "singleChoice" => {
                    checked.clear();
                    checked.push(index);
                }
                "multipleChoice" => {
                    if let Some(pos) = checked.iter().position(|v| *v == index) {
                        checked.remove(pos);
                    } else {
                        checked.push(index);
                    }
                }
                _ => {}
            }
            state.list_clicked.insert(key, index);
            action = Some(self.id);
        }
        action
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_search_view_ext(
        &'static self,
        f: &mut Frame<'_>,
        b: Rect,
        state: &mut LayoutState,
        id: u64,
        spec: SearchViewExtSpec<'static>,
        t: Theme,
        background_tint: Option<Color>,
        explicit_background: Option<Color>,
        scaled_radius: f32,
    ) -> Option<&'static str> {
        let view_key = if self.id.is_empty() {
            format!("#{id}")
        } else {
            self.id.to_owned()
        };
        let is_open = state.search_open.get(&view_key).copied().unwrap_or(false);
        let now = if f.time().is_finite() { f.time().max(0.0) } else { 0.0 };
        let prev = state.search_view_motion.get(&view_key).copied().unwrap_or_default();
        let (next_m, anim) =
            eval_binary_ease_out_motion(prev, is_open, 0.24, now, f.reduced_motion());
        if state.search_view_motion.len() >= 64
            && !state.search_view_motion.contains_key(&view_key)
        {
            state.search_view_motion.clear();
        }
        state.search_view_motion.insert(view_key.clone(), next_m);
        if anim {
            f.continuous(b);
        }
        let progress = next_m.progress;
        if !is_open && progress <= 0.001 {
            return None;
        }
        let enabled = self.enabled && f.enabled;
        let rtl = state.rtl;
        let mut action = None;
        state.text.entry(view_key.clone()).or_default();
        state.input_types.insert(view_key.clone(), 0);
        if state.search_pending.remove(&view_key) {
            action = Some(self.id);
        }
        let surface_color = background_tint
            .or(explicit_background)
            .unwrap_or(match state.theme_style {
                AndroidTheme::Material3Light => Color(0xEC, 0xE6, 0xF0, 255),
                AndroidTheme::Material3Dark => Color(0x2B, 0x29, 0x30, 255),
                _ => t.surface_high,
            });
        let pushed_layer = if progress < 0.999 {
            f.push_layer(b, progress);
            true
        } else {
            false
        };
        let block_input = !is_open || progress < 0.01;
        let mut closed_now = false;
        f.without_input(block_input, |f| {
            f.paint_rect(b, scaled_radius.max(0.0), surface_color);
            let header_h = state.dp(56.0).min(b.height);
            let header_rect = Rect {
                x: b.x,
                y: b.y,
                width: b.width,
                height: header_h,
            };
            let mut query = state.text.get(&view_key).cloned().unwrap_or_default();
            let mut show_clear = !query.is_empty();
            let btn_s = state.dp(48.0).min(header_h).min(header_rect.width);
            let edge_pad = state.dp(4.0);
            if !spec.hide_nav && btn_s > 0.0 {
                let back_x = if rtl {
                    header_rect.x + header_rect.width - edge_pad - btn_s
                } else {
                    header_rect.x + edge_pad
                };
                let back_rect = Rect {
                    x: back_x,
                    y: header_rect.y + (header_h - btn_s) * 0.5,
                    width: btn_s,
                    height: btn_s,
                };
                let back_hit = f.target(
                    anonymous_child_id(id, 9810),
                    back_rect,
                    "Back",
                    enabled,
                    Role::Button,
                );
                f.state_layer(back_hit, btn_s * 0.5, t.text);
                f.focus_ring(back_hit);
                let icon_s = state.dp(24.0).min(btn_s);
                let icon_rect = Rect {
                    x: back_rect.x + (back_rect.width - icon_s) * 0.5,
                    y: back_rect.y + (back_rect.height - icon_s) * 0.5,
                    width: icon_s,
                    height: icon_s,
                };
                let back_icon = if spec.use_drawer_arrow && progress < 0.5 {
                    MaterialIcon::Menu
                } else if rtl {
                    MaterialIcon::ArrowForward
                } else {
                    MaterialIcon::ArrowBack
                };
                f.paint_material_icon(
                    icon_rect,
                    back_icon,
                    t.text.opacity(if enabled { 1.0 } else { 0.38 }),
                );
                if back_hit.clicked {
                    closed_now = true;
                }
            }
            if enabled && is_open && f.pressed_key(Key::Escape) {
                closed_now = true;
            }
            if show_clear && btn_s > 0.0 {
                let clear_x = if rtl {
                    header_rect.x + edge_pad
                } else {
                    header_rect.x + header_rect.width - edge_pad - btn_s
                };
                let clear_rect = Rect {
                    x: clear_x,
                    y: header_rect.y + (header_h - btn_s) * 0.5,
                    width: btn_s,
                    height: btn_s,
                };
                let clear_hit = f.target(
                    anonymous_child_id(id, 9811),
                    clear_rect,
                    "Clear search",
                    enabled,
                    Role::Button,
                );
                f.state_layer(clear_hit, btn_s * 0.5, t.text);
                f.focus_ring(clear_hit);
                let icon_s = state.dp(24.0).min(btn_s);
                let icon_rect = Rect {
                    x: clear_rect.x + (clear_rect.width - icon_s) * 0.5,
                    y: clear_rect.y + (clear_rect.height - icon_s) * 0.5,
                    width: icon_s,
                    height: icon_s,
                };
                f.paint_material_icon(
                    icon_rect,
                    MaterialIcon::Close,
                    t.on_surface_variant.opacity(if enabled { 1.0 } else { 0.38 }),
                );
                if clear_hit.clicked {
                    query.clear();
                    show_clear = false;
                    state.text.insert(view_key.clone(), String::new());
                    state.search_focus_requested.insert(view_key.clone());
                    f.request_focus(id);
                    action = Some(self.id);
                    f.continuous(b);
                }
            }
            let lead = if spec.hide_nav {
                state.dp(16.0)
            } else {
                state.dp(56.0)
            };
            let trail = if show_clear {
                state.dp(56.0)
            } else {
                state.dp(16.0)
            };
            let field_x = header_rect.x + if rtl { trail } else { lead };
            let field_w = (header_rect.width - lead - trail).max(0.0);
            let field_rect = Rect {
                x: field_x,
                y: header_rect.y,
                width: field_w,
                height: header_h,
            };
            let hint = if !spec.hint.is_empty() {
                spec.hint
            } else if !self.hint.is_empty() {
                self.hint
            } else {
                "Search"
            };
            let field_resp = f.target(id, field_rect, hint, enabled, Role::TextField);
            f.focus_ring(field_resp);
            if field_resp.clicked {
                f.request_focus(id);
                state.search_focus_requested.insert(view_key.clone());
            }
            if field_resp.focused
                && f.pressed_key(Key::Enter)
                && state.submit_search(&view_key)
            {
                state.search_pending.remove(&view_key);
                action = Some(self.id);
            }
            let (display, text_color) = if query.is_empty() {
                (
                    hint,
                    t.on_surface_variant.opacity(if enabled { 1.0 } else { 0.38 }),
                )
            } else {
                (
                    query.as_str(),
                    self.color
                        .unwrap_or(t.text)
                        .opacity(if enabled { 1.0 } else { 0.38 }),
                )
            };
            let text_size = if (self.text_size - 16.0).abs() < 0.001 {
                state.sp(16.0)
            } else {
                self.scaled_text_size(state)
            };
            f.within(field_rect, |f| {
                f.styled_text(
                    field_rect,
                    display,
                    text_color,
                    TextStyle {
                        size: text_size,
                        weight: if self.text_bold { 700 } else { 400 },
                        align: if rtl { 2 } else { 0 },
                        letter_spacing: self.letter_spacing_em * text_size,
                        ..TextStyle::default()
                    },
                );
            });
            let div_h = state.dp(1.0).min((b.height - header_h).max(0.0));
            if div_h > 0.0 {
                f.paint_rect(
                    Rect {
                        x: b.x,
                        y: b.y + header_h,
                        width: b.width,
                        height: div_h,
                    },
                    0.0,
                    t.outline_variant,
                );
            }
            let body_clip = Rect {
                x: b.x,
                y: b.y + header_h + div_h,
                width: b.width,
                height: (b.height - header_h - div_h).max(0.0),
            };
            if body_clip.width > 0.0 && body_clip.height > 0.0 && !self.children.is_empty() {
                let padding = self.resolved_padding(state);
                let body_inner = Rect {
                    x: body_clip.x + padding[0],
                    y: body_clip.y + padding[1],
                    width: (body_clip.width - padding[0] - padding[2]).max(0.0),
                    height: (body_clip.height - padding[1] - padding[3]).max(0.0),
                };
                let metrics = f.ui.text_metrics;
                f.within(body_clip, |f| {
                    if matches!(self.kind, Kind::Linear) {
                        let horiz = self.horizontal;
                        let main_avail = if horiz { body_inner.width } else { body_inner.height };
                        let lengths = self.linear_main_lengths(
                            horiz,
                            state,
                            main_avail,
                            body_inner.width,
                            metrics,
                            id,
                        );
                        let mut cursor = if horiz && rtl {
                            body_inner.x + body_inner.width
                        } else if horiz {
                            body_inner.x
                        } else {
                            body_inner.y
                        };
                        for (i, n) in self.children.iter().enumerate() {
                            if n.effective_visibility(state) == Visibility::Gone {
                                continue;
                            }
                            let cid = child_view_id(id, i, n);
                            let m = n.resolved_margin(state);
                            let len = lengths[i];
                            let mut r = Rect {
                                x: body_inner.x + m[0],
                                y: body_inner.y + m[1],
                                width: (body_inner.width - m[0] - m[2]).max(0.0),
                                height: (body_inner.height - m[1] - m[3]).max(0.0),
                            };
                            if horiz {
                                r.x = if rtl { cursor - m[2] - len } else { cursor + m[0] };
                                r.width = len;
                                let ext = len + m[0] + m[2];
                                if rtl { cursor -= ext; } else { cursor += ext; }
                                if let Size::Dp(h) = n.height {
                                    r.height = state.dp(h).min(r.height);
                                } else if matches!(n.height, Size::Wrap) {
                                    r.height = n
                                        .layout_intrinsic(false, state, r.width, metrics, cid)
                                        .min(r.height);
                                }
                            } else {
                                r.y = cursor + m[1];
                                r.height = len;
                                cursor += len + m[1] + m[3];
                                if let Size::Dp(w) = n.width {
                                    r.width = state.dp(w).min(r.width);
                                } else if matches!(n.width, Size::Wrap) {
                                    r.width = n
                                        .layout_intrinsic(true, state, r.width, metrics, cid)
                                        .min(r.width);
                                }
                                let right = match n.layout_gravity[0] {
                                    0 => rtl,
                                    2 => !rtl,
                                    4 => true,
                                    _ => false,
                                };
                                r.x = if n.layout_gravity[0] == 1 {
                                    body_inner.x + (body_inner.width - r.width) * 0.5 + m[0] - m[2]
                                } else if right {
                                    body_inner.x + body_inner.width - r.width - m[2]
                                } else {
                                    body_inner.x + m[0]
                                };
                            }
                            let mut child_act = None;
                            if self.clip_children_disabled {
                                child_act = n.draw(f, r, state, cid);
                            } else {
                                f.within(r, |f| {
                                    child_act = n.draw(f, r, state, cid);
                                });
                            }
                            if child_act.is_some() {
                                action = child_act;
                            }
                        }
                    } else {
                        for (i, n) in self.children.iter().enumerate() {
                            if n.effective_visibility(state) == Visibility::Gone {
                                continue;
                            }
                            let cid = child_view_id(id, i, n);
                            let m = n.resolved_margin(state);
                            let mut r = Rect {
                                x: body_inner.x + m[0],
                                y: body_inner.y + m[1],
                                width: (body_inner.width - m[0] - m[2]).max(0.0),
                                height: (body_inner.height - m[1] - m[3]).max(0.0),
                            };
                            if let Size::Dp(w) = n.width {
                                r.width = state.dp(w);
                            } else if matches!(n.width, Size::Wrap) {
                                r.width = n
                                    .layout_intrinsic(true, state, r.width, metrics, cid)
                                    .min(r.width);
                            }
                            if let Size::Dp(h) = n.height {
                                r.height = state.dp(h);
                            } else if matches!(n.height, Size::Wrap) {
                                r.height = n
                                    .layout_intrinsic(false, state, r.width, metrics, cid)
                                    .min(r.height);
                            }
                            let right = match n.layout_gravity[0] {
                                0 => rtl,
                                2 => !rtl,
                                4 => true,
                                _ => false,
                            };
                            r.x = if n.layout_gravity[0] == 1 {
                                body_inner.x + (body_inner.width - r.width) * 0.5 + m[0] - m[2]
                            } else if right {
                                body_inner.x + body_inner.width - r.width - m[2]
                            } else {
                                body_inner.x + m[0]
                            };
                            r.y = match n.layout_gravity[1] {
                                1 => body_inner.y + (body_inner.height - r.height) * 0.5 + m[1] - m[3],
                                2 => body_inner.y + body_inner.height - r.height - m[3],
                                _ => body_inner.y + m[1],
                            };
                            let mut child_act = None;
                            if self.clip_children_disabled {
                                child_act = n.draw(f, r, state, cid);
                            } else {
                                f.within(r, |f| {
                                    child_act = n.draw(f, r, state, cid);
                                });
                            }
                            if child_act.is_some() {
                                action = child_act;
                            }
                        }
                    }
                });
            }
        });
        if closed_now {
            state.search_open.insert(view_key.clone(), false);
            let (closing_m, _closing_anim) =
                eval_binary_ease_out_motion(next_m, false, 0.24, now, f.reduced_motion());
            state.search_view_motion.insert(view_key, closing_m);
            f.continuous(b);
        }
        if pushed_layer {
            f.pop_layer();
        }
        action
    }
    pub(super) fn draw_carousel_layout(
        &'static self,
        f: &mut Frame<'_>,
        inner: Rect,
        state: &mut LayoutState,
        id: u64,
        spec: CarouselLayoutSpec,
        t: Theme,
    ) -> Option<&'static str> {
        let key = if self.id.is_empty() {
            format!("#{id}")
        } else {
            self.id.to_owned()
        };
        let (child_source, child_parent_id): (Option<&'static [Node]>, u64) =
            if matches!(self.kind, Kind::HorizontalScroll) {
                if self.children.len() == 1
                    && matches!(self.children[0].kind, Kind::Linear)
                    && !self.children[0].children.is_empty()
                    && !self.children[0].is_carousel_item()
                {
                    (
                        Some(self.children[0].children),
                        child_view_id(id, 0, &self.children[0]),
                    )
                } else {
                    (Some(self.children), id)
                }
            } else if matches!(self.kind, Kind::Linear) {
                (Some(self.children), id)
            } else if !self.children.is_empty() {
                (Some(self.children), id)
            } else {
                (None, id)
            };

        let visible_children: Vec<(usize, &'static Node)> = if let Some(nodes) = child_source {
            nodes
                .iter()
                .enumerate()
                .filter(|(_, n)| n.effective_visibility(state) != Visibility::Gone)
                .collect()
        } else {
            Vec::new()
        };
        let adapter_entries: Vec<String> = if child_source.is_none() {
            state
                .list_items
                .entry(key.clone())
                .or_insert_with(|| {
                    self.relative_rules
                        .iter()
                        .filter(|r| r.kind == 82)
                        .map(|r| r.anchor.to_owned())
                        .collect()
                })
                .clone()
        } else {
            Vec::new()
        };
        let n = if child_source.is_some() {
            visible_children.len()
        } else {
            adapter_entries.len()
        };
        if n == 0 {
            return None;
        }
        let viewport_w = inner.width.max(1.0);
        let gap = state.dp(spec.spacing_dp.max(0.0));
        let (focal_w, step, max_scroll) = match spec.strategy {
            CarouselStrategy::FullScreen => {
                let fw = viewport_w;
                let st = fw + gap;
                (fw, st, ((n - 1) as f32 * st).max(0.0))
            }
            CarouselStrategy::Uncontained => {
                let fw = (viewport_w * 0.60).max(1.0);
                let st = fw + gap;
                let ms = if spec.align == CarouselAlign::Center {
                    ((n - 1) as f32 * st).max(0.0)
                } else {
                    (n as f32 * fw + (n - 1) as f32 * gap - viewport_w).max(0.0)
                };
                (fw, st, ms)
            }
            CarouselStrategy::Hero => {
                let fw = (viewport_w * 0.78).max(1.0);
                let st = fw + gap;
                (fw, st, ((n - 1) as f32 * st).max(0.0))
            }
            CarouselStrategy::MultiBrowse => {
                let fw = (viewport_w * 0.55).max(1.0);
                let st = fw + gap;
                (fw, st, ((n - 1) as f32 * st).max(0.0))
            }
        };
        let _ = focal_w;
        let now = if f.time().is_finite() { f.time().max(0.0) } else { 0.0 };
        let rtl = state.rtl;
        let enabled = self.enabled && f.enabled;
        let mut cm = state.carousel_motion.remove(&key).unwrap_or_default();
        let mut offset = state
            .offsets
            .get(&key)
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .clamp(0.0, max_scroll);
        if let Some((start, from, to, dur)) = cm.snap {
            if f.reduced_motion() || dur <= 0.0 {
                offset = to.clamp(0.0, max_scroll);
                cm.snap = None;
            } else {
                let p = ((now - start) / dur).clamp(0.0, 1.0) as f32;
                let ease = 1.0 - (1.0 - p) * (1.0 - p);
                offset = (from + (to - from) * ease).clamp(0.0, max_scroll);
                if p >= 1.0 {
                    offset = to.clamp(0.0, max_scroll);
                    cm.snap = None;
                } else {
                    f.continuous(inner);
                }
            }
        }
        let prev_input = f.input;
        let offset_before_sv = offset;
        let touch_dragging = f.scroll_viewport_horizontal_rtl(id, inner, &mut offset, max_scroll, rtl);
        let wheel_moved = (offset - offset_before_sv).abs() > 0.01 && !touch_dragging;
        if wheel_moved {
            cm.snap = None;
        }
        let compute_snap_target = |cur_scroll: f32, v_scroll: f32| -> f32 {
            if max_scroll <= 0.0 || n <= 1 {
                return 0.0;
            }
            let mut candidates: Vec<f32> = (0..n)
                .map(|k| (k as f32 * step).clamp(0.0, max_scroll))
                .collect();
            if !candidates.last().is_some_and(|&v| (v - max_scroll).abs() < 0.5) {
                candidates.push(max_scroll);
            }
            let flick_v = state.dp(400.0);
            let hint = if v_scroll > flick_v {
                (cur_scroll + step * 0.45).min(max_scroll)
            } else if v_scroll < -flick_v {
                (cur_scroll - step * 0.45).max(0.0)
            } else {
                cur_scroll
            };
            candidates
                .into_iter()
                .min_by(|a, b| {
                    (a - hint)
                        .abs()
                        .partial_cmp(&(b - hint).abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(0.0)
        };
        if touch_dragging {
            cm.was_touch_dragging = true;
            cm.snap = None;
        }
        if cm.was_touch_dragging
            && (prev_input.released || prev_input.cancelled || prev_input.blurred)
        {
            cm.was_touch_dragging = false;
            f.cancel_scroll(id);
            let target = compute_snap_target(offset, 0.0);
            if f.reduced_motion() || (offset - target).abs() <= 0.5 {
                offset = target;
                cm.snap = None;
            } else {
                cm.snap = Some((now, offset, target, 0.24));
                f.continuous(inner);
            }
        }
        let mut mouse_was_dragging = false;
        if enabled && prev_input.pointer_kind != PointerKind::Touch {
            let inside = prev_input
                .pointer
                .is_some_and(|p| inner.intersect(f.clip).contains(p));
            if prev_input.pressed && inside {
                cm.origin = prev_input.pointer;
                cm.drag_base = offset;
                cm.dragging = false;
                cm.snap = None;
                cm.samples.clear();
                if let Some(p) = prev_input.pointer {
                    cm.samples.push((now, p[0]));
                }
            }
            if let (Some(origin), Some(ptr)) = (cm.origin, prev_input.pointer)
                && !prev_input.cancelled
                && !prev_input.blurred
            {
                if cm.samples.last().map_or(true, |&(_, x)| (x - ptr[0]).abs() > 0.01) {
                    cm.samples.push((now, ptr[0]));
                    if cm.samples.len() > 8 {
                        cm.samples.remove(0);
                    }
                }
                let dx = ptr[0] - origin[0];
                let dy = ptr[1] - origin[1];
                let slop = state.dp(8.0);
                if !cm.dragging && dx.abs() > slop && dx.abs() >= dy.abs() {
                    cm.dragging = true;
                    f.ui.active = None;
                }
                if cm.dragging {
                    let delta_scroll = if rtl { dx } else { -dx };
                    offset = (cm.drag_base + delta_scroll).clamp(0.0, max_scroll);
                    if !prev_input.released {
                        f.continuous(inner);
                    }
                }
            }
            mouse_was_dragging = cm.dragging;
            if cm.origin.is_some()
                && (prev_input.released || prev_input.cancelled || prev_input.blurred)
            {
                if cm.dragging {
                    let v_scroll = if !prev_input.cancelled && !prev_input.blurred {
                        let vx = pointer_velocity(&cm.samples, now);
                        if rtl { vx } else { -vx }
                    } else {
                        0.0
                    };
                    let target = compute_snap_target(offset, v_scroll);
                    if f.reduced_motion() || (offset - target).abs() <= 0.5 {
                        offset = target;
                        cm.snap = None;
                    } else {
                        cm.snap = Some((now, offset, target, 0.24));
                        f.continuous(inner);
                    }
                }
                cm.origin = None;
                cm.dragging = false;
                cm.samples.clear();
            }
        } else if !enabled {
            cm.origin = None;
            cm.dragging = false;
            cm.samples.clear();
        }
        offset = offset.clamp(0.0, max_scroll);
        state.offsets.insert(key.clone(), offset);
        let f_idx = if step > 0.0 {
            (offset / step).clamp(0.0, (n - 1) as f32)
        } else {
            0.0
        };
        state
            .selected_index
            .insert(key.clone(), (f_idx.round() as usize).min(n - 1));
        if state.carousel_motion.len() >= 64 && !state.carousel_motion.contains_key(&key) {
            state.carousel_motion.clear();
        }
        state.carousel_motion.insert(key.clone(), cm);

        let widths: Vec<f32> = (0..n)
            .map(|i| match spec.strategy {
                CarouselStrategy::FullScreen => viewport_w,
                CarouselStrategy::Uncontained => (viewport_w * 0.60).max(1.0),
                CarouselStrategy::Hero => {
                    let large = viewport_w * 0.78;
                    let small = viewport_w * 0.18;
                    let d = (i as f32 - f_idx).abs();
                    if d <= 1.0 {
                        large + (small - large) * d
                    } else {
                        small
                    }
                }
                CarouselStrategy::MultiBrowse => {
                    let large = viewport_w * 0.55;
                    let med = viewport_w * 0.26;
                    let small = viewport_w * 0.12;
                    let d = (i as f32 - f_idx).abs();
                    if d <= 1.0 {
                        large + (med - large) * d
                    } else if d <= 2.0 {
                        med + (small - med) * (d - 1.0)
                    } else {
                        small
                    }
                }
            })
            .collect();
        let mut raw_x = Vec::with_capacity(n);
        let mut cursor_x = 0.0_f32;
        for &w in &widths {
            raw_x.push(cursor_x);
            cursor_x += w + gap;
        }
        let logical_x: Vec<f32> = match spec.strategy {
            CarouselStrategy::FullScreen => raw_x.iter().map(|&x| x - offset).collect(),
            CarouselStrategy::Uncontained if spec.align == CarouselAlign::Start => {
                raw_x.iter().map(|&x| x - offset).collect()
            }
            _ => {
                let k0 = (f_idx.floor() as usize).min(n - 1);
                let k1 = (k0 + 1).min(n - 1);
                let frac = (f_idx - k0 as f32).clamp(0.0, 1.0);
                let shift = if spec.align == CarouselAlign::Center {
                    let c0 = raw_x[k0] + widths[k0] * 0.5;
                    let c1 = raw_x[k1] + widths[k1] * 0.5;
                    viewport_w * 0.5 - (c0 + (c1 - c0) * frac)
                } else {
                    let s0 = raw_x[k0];
                    let s1 = raw_x[k1];
                    -(s0 + (s1 - s0) * frac)
                };
                raw_x.iter().map(|&x| x + shift).collect()
            }
        };

        let any_dragging = touch_dragging || mouse_was_dragging;
        let mut action = None;
        let default_mask_r = state.dp(28.0);
        if child_source.is_some() {
            f.without_input(any_dragging, |f| {
                f.within(inner, |f| {
                    for (pos, &(orig_idx, child)) in visible_children.iter().enumerate() {
                        let w = widths[pos];
                        let phys_x = if rtl {
                            inner.x + inner.width - logical_x[pos] - w
                        } else {
                            inner.x + logical_x[pos]
                        };
                        let m = child.resolved_margin(state);
                        let avail_h = (inner.height - m[1] - m[3]).max(0.0);
                        let cid = child_view_id(child_parent_id, orig_idx, child);
                        let h = match child.height {
                            Size::Dp(v) => state.dp(v).min(avail_h),
                            Size::Wrap => child
                                .layout_intrinsic(false, state, w, f.ui.text_metrics, cid)
                                .min(avail_h)
                                .max(1.0),
                            Size::Match => avail_h,
                        };
                        let y = match child.layout_gravity[1] {
                            1 => inner.y + (inner.height - h) * 0.5 + m[1] - m[3],
                            2 => inner.y + inner.height - h - m[3],
                            _ => inner.y + m[1],
                        };
                        let item_rect = Rect { x: phys_x, y, width: w, height: h };
                        let vis = item_rect.intersect(inner);
                        if vis.width <= 0.0 || vis.height <= 0.0 {
                            continue;
                        }
                        let (is_oval, mask_radii) = if let Some(cr) = child.resolved_corner_radii(state) {
                            (false, cr)
                        } else if child.radius < 0.0 {
                            (true, [0.0; 4])
                        } else if child.radius > 0.0 {
                            (false, [state.dp(child.radius); 4])
                        } else {
                            (false, [default_mask_r; 4])
                        };
                        let cmd_start = f.commands.len();
                        let mut child_act = None;
                        f.within(item_rect, |f| {
                            child_act = child.draw(f, item_rect, state, cid);
                        });
                        clip_commands_to_rounded_outline(f, cmd_start, item_rect, is_oval, mask_radii);
                        if child_act.is_some() {
                            action = child_act;
                        }
                    }
                });
            });
        } else {
            let template = state.list_templates.get(&key).copied();
            let mut row_state = if template.is_some() || state.list_item_bindings.contains_key(&key)
            {
                Some(spinner_template_state(state, rtl))
            } else {
                None
            };
            let bindings = state.list_item_bindings.get(&key).cloned();
            let flags = state.list_enabled.get(&key).cloned();
            let item_ids = state.list_item_ids.get(&key).cloned();
            let checked = state.list_checked.get(&key).cloned().unwrap_or_default();
            let mut chosen = None;
            f.without_input(any_dragging, |f| {
                f.within(inner, |f| {
                    for (pos, label) in adapter_entries.iter().enumerate() {
                        let w = widths[pos];
                        let phys_x = if rtl {
                            inner.x + inner.width - logical_x[pos] - w
                        } else {
                            inner.x + logical_x[pos]
                        };
                        let item_rect = Rect {
                            x: phys_x,
                            y: inner.y,
                            width: w,
                            height: inner.height,
                        };
                        let vis = item_rect.intersect(inner);
                        if vis.width <= 0.0 || vis.height <= 0.0 {
                            continue;
                        }
                        let item_key = item_ids
                            .as_ref()
                            .and_then(|ids| ids.get(pos))
                            .map_or(pos as u64, |v| *v as u64);
                        let item_enabled =
                            enabled && flags.as_ref().and_then(|v| v.get(pos)).copied().unwrap_or(true);
                        let row_id = id.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(item_key);
                        let binding = bindings.as_ref().and_then(|items| items.get(&pos));
                        let item_template = adapter_item_template(template, binding);
                        let mask_r = item_template
                            .filter(|tmpl| tmpl.row.radius > 0.0)
                            .map_or(default_mask_r, |tmpl| state.dp(tmpl.row.radius));
                        let cmd_start = f.commands.len();
                        f.within(item_rect, |f| {
                            let r = f.target(row_id, item_rect, label, item_enabled, Role::Button);
                            if let (Some(tmpl), Some(rs)) = (item_template, row_state.as_mut()) {
                                paint_spinner_template(
                                    f,
                                    tmpl.row,
                                    tmpl.text_id,
                                    label,
                                    item_rect,
                                    rs,
                                    anonymous_child_id(id, 1024 + pos),
                                    r,
                                    checked.contains(&pos),
                                    binding,
                                );
                            } else {
                                let fill = if checked.contains(&pos) {
                                    t.primary_container
                                } else {
                                    t.surface_high
                                };
                                f.paint_rect(item_rect, mask_r, fill);
                                f.state_layer(r, mask_r, t.text);
                                let style = TextStyle {
                                    align: if rtl { 2 } else { 0 },
                                    padding: state.dp(12.0),
                                    ..TextStyle::default()
                                };
                                f.styled_text(
                                    item_rect,
                                    label,
                                    t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
                                    style,
                                );
                            }
                            if r.clicked {
                                chosen = Some(pos);
                            }
                        });
                        clip_commands_to_rounded_outline(f, cmd_start, item_rect, false, [mask_r; 4]);
                    }
                });
            });
            if let Some(idx) = chosen {
                state.list_clicked.insert(key, idx);
                action = Some(self.id);
            }
        }
        f.input = prev_input;
        action
    }
}
