use super::draw::DrawCtx;
use super::*;

impl Node {
    pub(super) fn draw_adapter_views(
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
            gradient,
            has_layers,
            has_stateful_layers,
            explicit_elevation,
            is_m3_theme,
            search_bar_hint,
            bottom_app_bar,
            inner,
            action: action_out,
            ..
        } = ctx;
        let mut action = *action_out;
        match self.kind {
            Kind::ExpandableListView => {
                use crate::android_expandable_list::{FlatPosition,ExpandableEvent};
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let mut indicator_images=state.images.clone();let indicator_scales=state.image_scales.clone();
                for name in self.relative_rules.iter().filter(|rule|rule.kind==131||rule.kind==132).filter_map(|rule|rule.anchor.splitn(3,',').nth(2)) {if let (Some(image),Some(tint))=(state.images.get(name).cloned(),state.drawable_tints.get(name).copied()) {let color=match tint {AndroidDrawableTint::Fixed(color)=>color,AndroidDrawableTint::ControlNormal=>t.on_surface_variant};let tinted=tint_image(state,&image,Some(color),1);indicator_images.insert(name.to_owned(),tinted);}}
                let anim_indicator = self.compound_anim_spec(CompoundAnimSlot::Indicator).and_then(|spec| {
                    let (frame_key, next_wake) = select_animation_list_frame(&spec, f.time(), f.reduced_motion());
                    let raw_img = state.images.get(frame_key)?.clone();
                    let img = if let Some(tint) = state.drawable_tints.get(frame_key).copied() {
                        let color = match tint {
                            AndroidDrawableTint::Fixed(color) => color,
                            AndroidDrawableTint::ControlNormal => t.on_surface_variant,
                        };
                        tint_image(state, &raw_img, Some(color), 1)
                    } else {
                        raw_img
                    };
                    let scale = indicator_scales.get(frame_key).copied().unwrap_or(1.0) * state.effective_density();
                    Some((img, scale, next_wake))
                });
                let templates=state.expandable_templates.get(&key).copied().unwrap_or((None,None));
                let bindings=state.expandable_bindings.get(&key);
                let (headers,footers)=state.list_fixed_views.get(&key).cloned().unwrap_or_default();
                let mut fixed_state=spinner_template_state(state,state.rtl);fixed_state.text=state.text.clone();fixed_state.visibility=state.visibility.clone();
                let header_heights:Vec<f32>=headers.iter().enumerate().map(|(i,v)|list_fixed_height(v.view,&fixed_state,inner.width,f.ui.text_metrics,anonymous_child_id(id,2048+i))).collect();
                let footer_heights:Vec<f32>=footers.iter().enumerate().map(|(i,v)|list_fixed_height(v.view,&fixed_state,inner.width,f.ui.text_metrics,anonymous_child_id(id,3072+i))).collect();
                let mut row_state=if templates.0.is_some()||templates.1.is_some()||bindings.is_some(){Some(spinner_template_state(state,state.rtl))}else{None};
                let list=state.expandable_lists.entry(key.clone()).or_default();let count=list.visible_count();
                let divider=if self.divider_color.is_some(){self.divider_thickness.max(0.0)}else{0.0};
                let header_dividers=!self.relative_rules.iter().any(|r|r.kind==89&&r.anchor=="false");
                let footer_dividers=!self.relative_rules.iter().any(|r|r.kind==90&&r.anchor=="false");
                let header_gaps=if header_dividers{headers.len().saturating_sub(usize::from(count==0&&footers.is_empty()))}else{0};
                let item_origin=header_heights.iter().sum::<f32>()+header_gaps as f32*divider;
                let footer_gap=if count>0&&!footers.is_empty()&&footer_dividers{divider}else{0.0};
                let footer_extent=footer_heights.iter().sum::<f32>()+if footer_dividers{footers.len().saturating_sub(1) as f32*divider}else{0.0};
                let child_divider=self.relative_rules.iter().find(|rule|rule.kind==110).and_then(|rule| {
                    let value=rule.anchor.strip_prefix("Some(Color(")?.strip_suffix("))")?;
                    let values:Vec<u8>=value.split(',').map(|v|v.trim().parse::<u8>()).collect::<Result<_,_>>().ok()?;
                    (values.len()==4).then(||Color(values[0],values[1],values[2],values[3]))
                }).or(self.divider_color);
                use std::hash::{Hash,Hasher};let mut hash=std::collections::hash_map::DefaultHasher::new();
                list.revision().hash(&mut hash);count.hash(&mut hash);inner.width.to_bits().hash(&mut hash);divider.to_bits().hash(&mut hash);
                for template in [templates.0,templates.1] {template.map(|t|(t.row as *const Node as usize,t.text_id)).hash(&mut hash);}
                for advance in f.ui.text_metrics.normal.iter().chain(f.ui.text_metrics.medium.iter()){advance.to_bits().hash(&mut hash);}
                if let Some(state)=row_state.as_ref(){hash_adapter_environment(&mut hash,state);}
                let signature=hash.finish();let geometry=state.expandable_geometry.entry(key.clone()).or_insert_with(||std::sync::Arc::new(ListTextGeometry::new(signature,count,divider)));
                if geometry.signature!=signature {*geometry=std::sync::Arc::new(ListTextGeometry::new(signature,count,divider));}
                let geometry=std::sync::Arc::make_mut(geometry);let offset=state.offsets.entry(key.clone()).or_default();
                if !offset.is_finite(){*offset=0.0;}let mut max=(item_origin+geometry.prefix(count)-if count>0{divider}else{0.0}+footer_gap+footer_extent-inner.height).max(0.0);*offset=offset.clamp(0.0,max);
                let dragging=f.scroll_viewport(id,inner,offset,max);let now=f.time();
                let interrupted=(f.pointer_in(inner)&&(f.input.pressed||f.input.scroll[1]!=0.0))||f.input.key.is_some()||f.input.cancelled||f.input.blurred;
                if interrupted {state.expandable_scrolls.remove(&key);state.expandable_pending_scroll.remove(&key);}
                if let Some(animation)=state.expandable_scrolls.get(&key).copied() {
                    let (value,active)=animation.sample(now);*offset=value.clamp(0.0,max);
                    if active&&!f.reduced_motion(){f.continuous(inner);}else{*offset=animation.to.clamp(0.0,max);state.expandable_scrolls.remove(&key);}
                }
                let mut flat=geometry.index_at((*offset-item_origin).max(0.0));let mut y=item_origin+geometry.prefix(flat);let mut changed=false;
                while flat<count && y<*offset+inner.height {
                    if let Some(position)=list.position_at(flat) {
                        let (label,template,binding_key)=match position {FlatPosition::Group(g)=>(&list.groups()[g].label,templates.0,(list.groups()[g].id,None)),FlatPosition::Child{group,child}=>(&list.groups()[group].children[child].label,templates.1,(list.groups()[group].id,Some(list.groups()[group].children[child].id)))};
                        let binding=bindings.and_then(|items|items.get(&binding_key));
                        changed|=geometry.measure(flat,label,inner.width,f.ui.text_metrics,template,&mut row_state,id,binding);
                    }
                    y+=geometry.heights[flat]+divider;flat+=1;
                }
                if changed{max=(item_origin+geometry.prefix(count)-if count>0{divider}else{0.0}+footer_gap+footer_extent-inner.height).max(0.0);*offset=offset.clamp(0.0,max);f.continuous(inner);}
                if let Some(group)=state.expandable_pending_scroll.remove(&key).and_then(|group_id|list.groups().iter().position(|group|group.id==group_id)) {
                    if let Some(flat)=list.flat_index(FlatPosition::Group(group)) {
                        let last=(flat+list.groups()[group].children.len()+1).min(count);
                        // Measure only enough children to know whether this group fits.
                        // Beyond one viewport the group itself is the scroll bound.
                        let group_top=item_origin+geometry.prefix(flat);let mut child_flat=flat+1;
                        while child_flat<last && item_origin+geometry.prefix(child_flat)-group_top<inner.height {
                            if let Some(FlatPosition::Child{group,child})=list.position_at(child_flat) {
                                let item=&list.groups()[group].children[child];
                                let binding=bindings.and_then(|items|items.get(&(list.groups()[group].id,Some(item.id))));
                                geometry.measure(child_flat,&item.label,inner.width,f.ui.text_metrics,templates.1,&mut row_state,id,binding);
                            }
                            child_flat+=1;
                        }
                        max=(item_origin+geometry.prefix(count)-if count>0{divider}else{0.0}+footer_gap+footer_extent-inner.height).max(0.0);
                        let destination=if group_top<*offset {group_top} else {offset.max((item_origin+geometry.prefix(last)-divider-inner.height).min(group_top))}.clamp(0.0,max);
                        if (destination-*offset).abs()>0.01 {if f.reduced_motion(){*offset=destination;}else{state.expandable_scrolls.insert(key.clone(),crate::android_expandable_list::ExpansionScroll{from:*offset,to:destination,start:now});}f.continuous(inner);}
                    }
                }
                let start=geometry.index_at((*offset-item_origin).max(0.0)).min(count);let end=geometry.index_at((*offset+inner.height-item_origin).max(0.0)).saturating_add(1).min(count);let scroll=*offset;let rtl=state.rtl;let mut chosen=None;let mut fixed_choice=None;let mut fixed_control_choice=None;let mut control_action=None;let row_state_environment=row_state.as_ref().cloned();
                let dim=|kind|self.relative_rules.iter().find(|rule|rule.kind==kind).and_then(|rule|rule.anchor.parse::<f32>().ok());
                let intrinsic=self.relative_rules.iter().filter(|rule|rule.kind==131).filter_map(|rule|rule.anchor.splitn(3,',').nth(2)).filter_map(|name|indicator_images.get(name).map(|image|image.width() as f32*indicator_scales.get(name).copied().unwrap_or(1.0))).fold(0.0_f32,f32::max);
                let (mut group_left,mut group_right)=(dim(133).unwrap_or(0.0),dim(134).unwrap_or(0.0));
                if rtl {if let Some(v)=dim(135){group_right=v;}if let Some(v)=dim(136){group_left=v;}} else {if let Some(v)=dim(135){group_left=v;}if let Some(v)=dim(136){group_right=v;}}
                if group_right==0.0 {
                    group_right=group_left+if let Some((ref img, scale, _)) = anim_indicator {
                        img.width() as f32 * scale
                    } else if self.relative_rules.iter().any(|rule|rule.kind==131){
                        intrinsic
                    } else {
                        31.0
                    };
                }
                let (mut child_left,mut child_right)=(dim(137),dim(138));
                if rtl {if let Some(v)=dim(139){child_right=Some(v);}if let Some(v)=dim(140){child_left=Some(v);}}else{if let Some(v)=dim(139){child_left=Some(v);}if let Some(v)=dim(140){child_right=Some(v);}}
                let control_states=&mut state.expandable_control_states;let fixed_control_states=&mut state.list_fixed_control_states;
                f.without_input(dragging,|f|f.within(inner,|f|{
                    let mut y=0.0;
                    for (i,fixed) in headers.iter().enumerate() {
                        let bounds=Rect{x:inner.x,y:inner.y+y-scroll,width:inner.width,height:header_heights[i]};
                        if bounds.intersect(inner).height>0.0 {let controls=fixed_control_states.entry((key.clone(),false,i)).or_default();let (clicked,control)=draw_list_fixed(f,*fixed,bounds,&mut fixed_state,anonymous_child_id(id,2048+i),self.enabled,controls);if clicked{fixed_choice=Some((false,i));}if let Some(view)=control{fixed_control_choice=Some((false,i,view));}}
                        y+=header_heights[i];if header_dividers&&(i+1<headers.len()||count>0||!footers.is_empty()){if let Some(color)=self.divider_color{f.paint_rect(Rect{x:inner.x,y:inner.y+y-scroll,width:inner.width,height:divider},0.0,color);}y+=divider;}
                    }
                    for flat in start..end {
                    let Some(position)=list.position_at(flat) else{continue;};let (label,enabled,group,expanded)=match position {FlatPosition::Group(g)=>(&list.groups()[g].label,true,true,list.is_expanded(g)),FlatPosition::Child{group,child}=>{let item=&list.groups()[group].children[child];(&item.label,item.selectable,false,false)}};
                    let rect=Rect{x:inner.x,y:inner.y+item_origin+geometry.prefix(flat)-scroll,width:inner.width,height:geometry.heights[flat]};
                    // A pressed row must keep its identity across insertions, reorders,
                    // and group expansion; a flat index can refer to a different row.
                    let mut row_hash=std::collections::hash_map::DefaultHasher::new();id.hash(&mut row_hash);
                    match position {FlatPosition::Group(g)=>{0u8.hash(&mut row_hash);list.groups()[g].id.hash(&mut row_hash);},FlatPosition::Child{group,child}=>{1u8.hash(&mut row_hash);list.groups()[group].id.hash(&mut row_hash);list.groups()[group].children[child].id.hash(&mut row_hash);}}
                    let row_id=row_hash.finish();
                    let binding_key=match position {FlatPosition::Group(g)=>(list.groups()[g].id,None),FlatPosition::Child{group,child}=>(list.groups()[group].id,Some(list.groups()[group].children[child].id))};
                    let binding=bindings.and_then(|items|items.get(&binding_key));
                    let template=adapter_item_template(if group{templates.0}else{templates.1},binding);
                    let interactive=template.is_some_and(|template|inspect_expandable_template(template.row,template.text_id).2);
                    if interactive {
                        let template=template.unwrap();let checked=control_states.entry((key.clone(),binding_key.0,binding_key.1)).or_default();
                        let mut local=row_state_environment.clone().unwrap_or_default();
                        local.adapter_namespace=Some(row_id);local.rtl=rtl;local.checked=std::mem::take(checked);
                        with_adapter_binding(&mut local,template.text_id,label,binding,|row_state| {
                            f.within(rect,|f|f.with_enabled(self.enabled&&enabled,|f|{if let Some(view)=template.row.draw(f,rect,row_state,anonymous_child_id(row_id,0)){control_action=Some((position,view));}}));
                        });
                        *checked=local.checked;
                    }
                    let hit=f.target(row_id,rect,label,self.enabled&&enabled,Role::Button);f.state_layer(hit,0.0,t.text);
                    if interactive {} else if let (Some(template),Some(row_state))=(template,row_state.as_mut()) {
                        paint_spinner_template(f,template.row,template.text_id,label,rect,row_state,anonymous_child_id(id,20000+flat),hit,expanded,binding);
                    } else {
                        let indent=if group{40.0}else{64.0};let text=Rect{x:if rtl{rect.x}else{rect.x+indent},y:rect.y,width:(rect.width-indent).max(0.0),height:rect.height};f.styled_text(text,label,t.text.opacity(if hit.enabled{1.0}else{0.38}),TextStyle{size:16.0,align:if rtl{2}else{0},padding:8.0,..TextStyle::default()});
                    }
                    let kind=if group{131}else{132};
                    let custom=self.relative_rules.iter().any(|rule|rule.kind==kind);
                    if group && let Some((ref img, scale, next_wake)) = anim_indicator {
                        let positioned=(133..=136).any(|kind|dim(kind).is_some());
                        let bounds=if positioned {
                            Rect{x:if rtl{rect.x+rect.width-group_right}else{rect.x+group_left},y:rect.y,width:(group_right-group_left).max(0.0),height:rect.height}
                        } else {
                            let w=(img.width() as f32*scale).min(rect.width);
                            let h=(img.height() as f32*scale).min(rect.height);
                            Rect{x:if rtl{rect.x+rect.width-w}else{rect.x},y:rect.y+(rect.height-h)*0.5,width:w,height:h}
                        };
                        if bounds.width>0.0 && bounds.height>0.0 {
                            if let Some(wake) = next_wake {
                                let vis = bounds.intersect(f.clip);
                                if vis.width > 0.0 && vis.height > 0.0 {
                                    f.wake_at(wake);
                                }
                            }
                            let mirror_ind = self.is_auto_mirrored() && rtl;
                            if mirror_ind {
                                f.push_transform(
                                    [bounds.x + bounds.width * 0.5, bounds.y + bounds.height * 0.5],
                                    [0.0, 0.0],
                                    [-1.0, 1.0],
                                    0.0,
                                );
                            }
                            f.commands.push(DrawCommand::Image{bounds,uv:[0.0,0.0,1.0,1.0],radius:0.0,color:Color(255,255,255,255).opacity(if self.enabled{1.0}else{0.38}),image:img.clone()});
                            if mirror_ind {
                                f.pop_transform();
                            }
                        }
                    } else if custom {
                        let flags=match position {FlatPosition::Group(g)=>u8::from(expanded)|if !expanded||list.groups()[g].children.is_empty(){2}else{0},FlatPosition::Child{group,child}=>u8::from(child+1==list.groups()[group].children.len())};
                        if let Some(name)=self.relative_rules.iter().filter(|rule|rule.kind==kind).find_map(|rule|{let mut fields=rule.anchor.splitn(3,',');let required=fields.next()?.parse::<u8>().ok()?;let forbidden=fields.next()?.parse::<u8>().ok()?;let name=fields.next()?;(flags&required==required&&flags&forbidden==0).then_some(name)}) {
                            if let Some(image)=indicator_images.get(name) {let left=if group{group_left}else{child_left.unwrap_or(group_left)};let right=if group{group_right}else{child_right.unwrap_or(group_right)};let bounds=Rect{x:if rtl{rect.x+rect.width-right}else{rect.x+left},y:rect.y,width:(right-left).max(0.0),height:rect.height};if bounds.width>0.0 {f.commands.push(DrawCommand::Image{bounds,uv:[0.0,0.0,1.0,1.0],radius:0.0,color:Color(255,255,255,255).opacity(if self.enabled{1.0}else{0.38}),image:image.clone()});}}
                        }
                    } else if group {
                        // The source nine-patch stretches its transparent outer pixels;
                        // keeping the 31dp image centered preserves its artwork at row height.
                        let size=31.0_f32.min(rect.height).min(rect.width);let positioned=(133..=136).any(|kind|dim(kind).is_some());
                        let bounds=if positioned {Rect{x:if rtl{rect.x+rect.width-group_right}else{rect.x+group_left},y:rect.y,width:(group_right-group_left).max(0.0),height:rect.height}}else{Rect{x:if rtl{rect.x+rect.width-size}else{rect.x},y:rect.y+(rect.height-size)/2.0,width:size,height:size}};
                        f.commands.push(DrawCommand::Image{bounds,uv:[0.0,0.0,1.0,1.0],radius:0.0,color:t.text.opacity(if self.enabled{1.0}else{0.38}),image:expandable_indicator(expanded).clone()});
                    }
                    if flat+1<count {
                        if divider>0.0 {
                            let child_line=!group || (expanded && matches!(position,FlatPosition::Group(g) if !list.groups()[g].children.is_empty()));
                            if let Some(color)=if child_line{child_divider}else{self.divider_color} {f.paint_rect(Rect{x:rect.x,y:rect.y+rect.height,width:rect.width,height:divider},0.0,color);}
                        } else if self.divider_color.is_none() {f.paint_line([rect.x,rect.y+rect.height-0.5],[rect.x+rect.width,rect.y+rect.height-0.5],1.0,t.outline_variant);}
                    }
                    if hit.clicked{chosen=Some(position);}
                    }
                    let mut y=item_origin+(geometry.prefix(count)-if count>0{divider}else{0.0})+footer_gap;
                    if footer_gap>0.0 {if let Some(color)=self.divider_color{f.paint_rect(Rect{x:inner.x,y:inner.y+y-footer_gap-scroll,width:inner.width,height:footer_gap},0.0,color);}}
                    for (i,fixed) in footers.iter().enumerate() {
                        let bounds=Rect{x:inner.x,y:inner.y+y-scroll,width:inner.width,height:footer_heights[i]};
                        if bounds.intersect(inner).height>0.0 {let controls=fixed_control_states.entry((key.clone(),true,i)).or_default();let (clicked,control)=draw_list_fixed(f,*fixed,bounds,&mut fixed_state,anonymous_child_id(id,3072+i),self.enabled,controls);if clicked{fixed_choice=Some((true,i));}if let Some(view)=control{fixed_control_choice=Some((true,i,view));}}
                        y+=footer_heights[i];if footer_dividers&&i+1<footers.len(){if let Some(color)=self.divider_color{f.paint_rect(Rect{x:inner.x,y:inner.y+y-scroll,width:inner.width,height:divider},0.0,color);}y+=divider;}
                    }
                }));
                if let Some(control)=fixed_control_choice {state.list_fixed_control_clicked.insert(key.clone(),control);action=Some(self.id);}
                if let Some(fixed)=fixed_choice {state.list_fixed_clicked.insert(key.clone(),fixed);action=Some(self.id);}
                if let Some((position,view))=control_action {state.expandable_events.remove(&key);state.expandable_control_events.insert(key.clone(),(position,view));action=Some(self.id);}
                if let Some(position)=chosen {
                    let intercepted=if let FlatPosition::Group(g)=position {
                        state.expandable_group_clicked.insert(key.clone(),g);
                        state.expandable_intercepted_groups.get(&key).is_some_and(|groups|groups.contains(&list.groups()[g].id))
                    }else{false};
                    if intercepted {state.expandable_events.remove(&key);action=Some(self.id);} else {
                    let event=match position {FlatPosition::Group(g)=>if list.is_expanded(g){list.collapse(g).then_some(ExpandableEvent::Collapsed(g))}else{list.expand(g).then_some(ExpandableEvent::Expanded(g))},FlatPosition::Child{group,child}=>Some(ExpandableEvent::ChildClicked{group,child})};if let Some(event)=event{
                    state.expandable_scrolls.remove(&key);
                    if let ExpandableEvent::Expanded(group)=event {state.expandable_pending_scroll.insert(key.clone(),list.groups()[group].id);}
                    state.expandable_events.insert(key,event);action=Some(self.id);f.continuous(inner);
                }}}
            }
            Kind::SlidingDrawer => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};let rule=|kind|self.relative_rules.iter().find(|rule|rule.kind==kind).map(|rule|rule.anchor);
                let handle=rule(138).and_then(|name|self.children.iter().find_map(|child|tab_host_find(child,name)));
                let content=rule(139).and_then(|name|self.children.iter().find_map(|child|tab_host_find(child,name)));
                if let (Some(handle),Some(content))=(handle,content) {
                    let mut drawer=state.sliding_drawers.remove(&key).unwrap_or_default();let now=f.time();
                    if let Some((start,from,to,duration,speed,acceleration))=drawer.motion {let elapsed=(now-start).clamp(0.0,duration) as f32;let distance=(speed*elapsed+acceleration*elapsed*elapsed).min((to-from).abs());drawer.position=from+(to-from).signum()*distance;if now-start>=duration||f.reduced_motion(){drawer.position=to;drawer.complete(to>=0.5);drawer.motion=None;}else{f.continuous(inner);}}
                    let top=rule(140).and_then(|v|v.parse::<f32>().ok()).unwrap_or(0.0);let bottom=rule(141).and_then(|v|v.parse::<f32>().ok()).unwrap_or(0.0);let horizontal=self.horizontal;
                    let axis=if horizontal{inner.width}else{inner.height};let cross=if horizontal{inner.height}else{inner.width};let handle_id=view_id(handle.id);
                    let h_axis=handle.intrinsic(horizontal,state,handle_id).clamp(0.0,axis);let h_cross=handle.intrinsic(!horizontal,state,handle_id).clamp(0.0,cross);let closed=(axis-h_axis+bottom).max(top);let travel=(closed-top).max(0.0);let pos=top+(1.0-drawer.position)*travel;
                    let handle_rect=if horizontal{Rect{x:inner.x+pos,y:inner.y+(cross-h_cross)/2.0,width:h_axis,height:h_cross}}else{Rect{x:inner.x+(cross-h_cross)/2.0,y:inner.y+pos,width:h_cross,height:h_axis}};
                    let hit=f.target(anonymous_child_id(id,9800),handle_rect,if drawer.open{"Close drawer"}else{"Open drawer"},self.enabled&&handle.enabled&&!drawer.locked,Role::Button);
                    if self.enabled&&f.enabled&&!drawer.locked&&drawer.drag.is_none()&&f.input.pressed&&f.pointer_in(handle_rect) {if let Some(pointer)=f.input.pointer {drawer.record(SlidingDrawerEvent::ScrollStarted);drawer.drag=Some((if horizontal{pointer[0]}else{pointer[1]},drawer.position));drawer.dragging=false;drawer.motion=None;drawer.samples.clear();drawer.samples.push((now,if horizontal{pointer[0]}else{pointer[1]}));}}
                    if let (Some((origin,initial)),Some(pointer))=(drawer.drag,f.input.pointer) {if !f.input.released&&!f.input.cancelled&&!f.input.blurred {let coordinate=if horizontal{pointer[0]}else{pointer[1]};let delta=coordinate-origin;if drawer.samples.last().map_or(true,|(_,v)|*v!=coordinate) {drawer.samples.push((now,coordinate));if drawer.samples.len()>8 {drawer.samples.remove(0);}}if delta.abs()>6.0 {drawer.dragging=true;drawer.position=if travel>0.0 {(initial-delta/travel).clamp(0.0,1.0)}else{initial};f.ui.active=None;f.continuous(inner);}}}
                    let dragged=drawer.dragging;let pending=drawer.pending.take();if pending.is_some_and(|(_,animated)|animated) {drawer.record(SlidingDrawerEvent::ScrollStarted);drawer.record(SlidingDrawerEvent::ScrollEnded);}let mut target=pending.map(|(open,_)|open);let mut animate=pending.map_or(true,|(_,animated)|animated);let mut release_velocity=0.0f32;
                    if drawer.drag.is_some()&&(f.input.released||f.input.cancelled||f.input.blurred||!self.enabled||!f.enabled) {if dragged {if let Some(&(last_time,last_pos))=drawer.samples.last() {if now-last_time<0.1 {if let Some(&(first_time,first_pos))=drawer.samples.iter().rev().find(|(t,_)|last_time-*t>=0.01) {release_velocity=(last_pos-first_pos)/((last_time-first_time) as f32);}}}let position=top+(1.0-drawer.position)*travel;let open=if drawer.open {!(release_velocity>200.0||(position>top+h_axis&&release_velocity> -200.0))}else{!(release_velocity>200.0||(position>axis/2.0&&release_velocity> -200.0))};target=Some(if f.input.cancelled||f.input.blurred{release_velocity=0.0;drawer.open}else{open});}drawer.record(SlidingDrawerEvent::ScrollEnded);drawer.drag=None;drawer.dragging=false;}
                    if hit.clicked&&!dragged&&!drawer.locked {let keyboard=f.input.key.is_some();let allow=rule(142)!=Some("false");if keyboard||allow {target=Some(!drawer.open);animate=!keyboard||rule(143)!=Some("false");}}
                    if let Some(open)=target {let to=if open{1.0}else{0.0};if !animate||f.reduced_motion()||travel<=0.0 {drawer.position=to;drawer.complete(open);drawer.motion=None;}else{let physical_distance=(drawer.position-to).abs()*travel;let velocity=if dragged {if open{(-release_velocity).max(0.0)}else{release_velocity.max(0.0)}}else{2000.0};let duration=((velocity*velocity+4000.0*physical_distance).sqrt()-velocity)/2000.0;let duration=duration.max(0.001) as f64;drawer.motion=Some((now,drawer.position,to,duration,if travel>0.0{velocity/travel}else{0.0},if travel>0.0{1000.0/travel}else{0.0}));f.continuous(inner);}action=Some(self.id);}
                    let pos=top+(1.0-drawer.position)*travel;let handle_rect=if horizontal{Rect{x:inner.x+pos,y:inner.y+(cross-h_cross)/2.0,width:h_axis,height:h_cross}}else{Rect{x:inner.x+(cross-h_cross)/2.0,y:inner.y+pos,width:h_cross,height:h_axis}};
                    let content_rect=if horizontal{Rect{x:inner.x+pos+h_axis,y:inner.y,width:(axis-top-h_axis).max(0.0),height:cross}}else{Rect{x:inner.x,y:inner.y+pos+h_axis,width:cross,height:(axis-top-h_axis).max(0.0)}};
                    f.within(inner,|f|{if drawer.position>0.0 {if let Some(child_action)=content.draw(f,content_rect,state,view_id(content.id)){action=Some(child_action);}}f.without_input(true,|f|{handle.draw(f,handle_rect,state,handle_id);});f.state_layer(hit,0.0,t.text);});
                    state.sliding_drawers.insert(key,drawer);
                }
            }
            Kind::TabHost => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                if let Some(host)=state.tab_hosts.get(&key).cloned() {
                    let widget=tab_host_find(self,&host.widget_id);let content=tab_host_find(self,"android:tabcontent");
                    let valid=widget.is_some_and(|node|matches!(node.kind,Kind::TabWidget)&&(node.children.len()==host.content_ids.len()||node.children.is_empty()&&host.labels.len()==host.content_ids.len()))&&content.is_some_and(|node|matches!(node.kind,Kind::Frame)&&host.content_ids.iter().enumerate().all(|(i,id)|host.factories.get(i).is_some_and(Option::is_some)||tab_host_find(node,id).is_some()));
                    if valid {state.set_tab_host_current(&key,host.selected);let previous=state.active_tab_host.replace(key.clone());
                        for (index,child) in self.children.iter().enumerate(){if let Some(child_action)=child.draw(f,inner,state,child_view_id(id,index,child)){action=Some(child_action);}}
                        state.active_tab_host=previous;
                    }
                }
            }
            Kind::TabWidget => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let m3=matches!(state.theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark);
                let tab_opt = self.tab_options();
                let tab_style = tab_opt.map(|o| o.style).unwrap_or(crate::systems::material::TabStyle::Primary);
                let tab_scrollable = tab_opt.is_some_and(|o| o.scrollable);
                let indicator_color = tab_opt.and_then(|o| o.indicator_color).unwrap_or(t.primary);
                let indicator_h = match tab_opt {
                    Some(opt) => state.dp(match opt.style {
                        crate::systems::material::TabStyle::Primary => 3.0,
                        crate::systems::material::TabStyle::Secondary => 2.0,
                    }),
                    None => state.dp(if m3 { 3.0 } else { 2.0 }),
                };
                let custom_strips=self.relative_rules.iter().any(|rule|rule.kind==191||rule.kind==192);
                let show_strip = !custom_strips && self.relative_rules.iter().all(|rule| rule.kind != 147 || rule.anchor != "false");
                if m3 || tab_opt.is_some() {
                    let div_h = state.dp(1.0).min(inner.height);
                    f.paint_rect(
                        Rect { x: inner.x, y: inner.y + inner.height - div_h, width: inner.width, height: div_h },
                        0.0,
                        t.outline_variant,
                    );
                }
                let sp14 = state.sp(14.0);
                let dp24 = state.dp(24.0);
                let dp4 = state.dp(4.0);
                let paint_tab_indicator = |f: &mut Frame<'_>, bounds: Rect, label_text: &str| {
                    let h = indicator_h.min(bounds.height);
                    if h <= 0.0 || bounds.width <= 0.0 {
                        return;
                    }
                    if tab_opt.is_some() {
                        match tab_style {
                            crate::systems::material::TabStyle::Primary => {
                                let lbl_w = f.text_width(
                                    label_text,
                                    TextStyle { size: sp14, weight: 500, ..TextStyle::default() },
                                );
                                let ind_w = lbl_w.max(dp24).min((bounds.width - dp4).max(0.0));
                                let ind_rect = Rect {
                                    x: bounds.x + (bounds.width - ind_w) * 0.5,
                                    y: bounds.y + bounds.height - h,
                                    width: ind_w,
                                    height: h,
                                };
                                paint_rounded_rect_radii(f, ind_rect, [h, h, 0.0, 0.0], indicator_color);
                            }
                            crate::systems::material::TabStyle::Secondary => {
                                f.paint_rect(
                                    Rect { x: bounds.x, y: bounds.y + bounds.height - h, width: bounds.width, height: h },
                                    0.0,
                                    indicator_color,
                                );
                            }
                        }
                    } else {
                        f.paint_rect(
                            Rect { x: bounds.x, y: bounds.y + bounds.height - h, width: bounds.width, height: h },
                            0.0,
                            indicator_color,
                        );
                    }
                };
                let mut strip_bounds=None;
                let visible:Vec<_>=self.children.iter().enumerate().filter(|(_,child)|child.effective_visibility(state)!=Visibility::Gone).collect();
                let generated=state.active_tab_host.as_ref().and_then(|host|state.tab_hosts.get(host)).filter(|host|host.widget_id==key&&self.children.is_empty()).map(|host|host.labels.clone()).unwrap_or_default();
                let resolve_tab_selected = |state: &mut LayoutState, default_idx: usize| -> (usize, bool) {
                    let last = state.tab_last_index.get(&key).copied();
                    let sel = state.selected_index.get(&key).copied();
                    let disp = state.displayed_children.get(&key).copied();
                    let chosen = match last {
                        Some(prev) => {
                            if let Some(s) = sel.filter(|&v| v != prev) {
                                s
                            } else if let Some(d) = disp.filter(|&v| v != prev) {
                                d
                            } else {
                                disp.or(sel).unwrap_or(default_idx)
                            }
                        }
                        None => disp.or(sel).unwrap_or(default_idx),
                    };
                    (chosen, last.is_some_and(|prev| prev != chosen))
                };
                let linked_partners = state.linked_tab_pager_partners(&key);
                if !generated.is_empty() {
                    let (raw_sel, sel_changed) = resolve_tab_selected(state, 0);
                    let selected = raw_sel.min(generated.len() - 1);
                    state.tab_last_index.insert(key.clone(), selected);
                    if !linked_partners.is_empty() || state.selected_index.contains_key(&key) {
                        state.displayed_children.insert(key.clone(), selected);
                        state.selected_index.insert(key.clone(), selected);
                    }
                    let lbl_style = TextStyle { size: state.sp(14.0), align: 1, ..TextStyle::default() };
                    let widths: Vec<f32> = if tab_scrollable {
                        generated.iter().map(|lbl| (f.text_width(lbl, lbl_style) + state.dp(32.0)).clamp(state.dp(72.0), state.dp(264.0))).collect()
                    } else {
                        vec![inner.width / generated.len() as f32; generated.len()]
                    };
                    let total_w: f32 = widths.iter().sum();
                    let max_scroll = if tab_scrollable { (total_w - inner.width).max(0.0) } else { 0.0 };
                    let mut scroll_offset = if tab_scrollable {
                        state.tab_scroll_offsets.get(&key).copied().unwrap_or(0.0).clamp(0.0, max_scroll)
                    } else {
                        0.0
                    };
                    let dragged = if tab_scrollable && enabled {
                        f.scroll_viewport_horizontal(anonymous_child_id(id, 10299), inner, &mut scroll_offset, max_scroll)
                    } else {
                        false
                    };
                    let mut clicked=None;
                    f.within(inner, |f| f.without_input(dragged, |f| {
                        let mut cursor = if state.rtl { inner.x + inner.width + scroll_offset } else { inner.x - scroll_offset };
                        for (index, label) in generated.iter().enumerate() {
                            let width = widths[index];
                            let x = if state.rtl { cursor -= width; cursor } else { let x = cursor; cursor += width; x };
                            let bounds = Rect { x, y: inner.y, width, height: inner.height };
                            let hit = f.target(anonymous_child_id(id, 10300 + index), bounds, label, self.enabled, Role::Button);
                            if index == selected {
                                strip_bounds = Some(bounds);
                                if !m3 && tab_opt.is_none() {
                                    f.paint_rect(bounds, 0.0, t.surface_highest);
                                }
                            }
                            let txt_color = if index == selected && (m3 || tab_opt.is_some()) {
                                match tab_style {
                                    crate::systems::material::TabStyle::Primary => indicator_color,
                                    crate::systems::material::TabStyle::Secondary => t.text,
                                }
                            } else if m3 || tab_opt.is_some() {
                                t.on_surface_variant
                            } else {
                                t.text
                            };
                            f.styled_text(bounds, label, txt_color, lbl_style);
                            if let Some(badge) = self.nav_badge_for_item(index, label, state) {
                                let ext = self.material_badge_ext(Some(index), label);
                                paint_nav_badge_ext(f, bounds, badge, ext, state, if self.enabled { 1.0 } else { 0.38 });
                            }
                            f.state_layer(hit, 0.0, t.text);
                            if index == selected && show_strip {
                                paint_tab_indicator(f, bounds, label);
                            }
                            if hit.clicked {
                                clicked = Some(index);
                            }
                        }
                    }));
                    if tab_scrollable {
                        if let Some(sel_idx) = clicked.or(sel_changed.then_some(selected)) && max_scroll > 0.0 {
                            let start_off: f32 = widths.iter().take(sel_idx).sum();
                            let end_off = start_off + widths[sel_idx];
                            if start_off < scroll_offset {
                                scroll_offset = start_off;
                            } else if end_off > scroll_offset + inner.width {
                                scroll_offset = (end_off - inner.width).min(max_scroll);
                            }
                        }
                        state.tab_scroll_offsets.insert(key.clone(), scroll_offset.clamp(0.0, max_scroll));
                    }
                    if let Some(index) = clicked {
                        state.displayed_children.insert(key.clone(), index);
                        state.selected_index.insert(key.clone(), index);
                        state.tab_last_index.insert(key.clone(), index);
                        state.tab_widget_clicked.insert(key.clone(), index);
                        if !linked_partners.is_empty() {
                            for partner in &linked_partners {
                                state.selected_index.insert(partner.clone(), index);
                                state.displayed_children.insert(partner.clone(), index);
                            }
                            f.continuous(b);
                        }
                        if let Some(host_id) = state.active_tab_host.clone() {
                            state.set_tab_host_current(&host_id, index);
                        }
                        action = Some(self.id);
                    }
                }else if !visible.is_empty() {
                    let (raw_sel, sel_changed) = resolve_tab_selected(state, visible[0].0);
                    let selected = if visible.iter().any(|(i, _)| *i == raw_sel) { raw_sel } else { visible[0].0 };
                    state.tab_last_index.insert(key.clone(), selected);
                    if !linked_partners.is_empty() || state.selected_index.contains_key(&key) {
                        state.displayed_children.insert(key.clone(), selected);
                        state.selected_index.insert(key.clone(), selected);
                    }
                    let measured = !tab_scrollable && tab_opt.is_none() && visible.iter().all(|(_,child)|matches!(child.width,Size::Dp(_)|Size::Wrap));
                    let mut widths:Vec<f32>=visible.iter().map(|(index,child)| {
                        if tab_scrollable {
                            let label = state.text.get(child.id).map(String::as_str).unwrap_or(child.text);
                            let intrinsic_w = child.layout_intrinsic(true, state, state.dp(264.0), f.ui.text_metrics, child_view_id(id, *index, child));
                            let lbl_w = f.text_width(label, TextStyle { size: state.sp(14.0), ..TextStyle::default() }) + state.dp(32.0);
                            intrinsic_w.max(lbl_w).clamp(state.dp(72.0), state.dp(264.0))
                        } else if measured {
                            child.layout_intrinsic(true,state,inner.width,f.ui.text_metrics,child_view_id(id,*index,child)).max(0.0)
                        } else {
                            inner.width/visible.len() as f32
                        }
                    }).collect();
                    if measured {
                        let margins=visible.iter().map(|(_,child)|{let m=child.resolved_margin(state);m[0]+m[2]}).sum::<f32>();
                        let mut overflow=(widths.iter().sum::<f32>()+margins-inner.width).max(0.0);
                        let mut remaining=widths.len();
                        for width in &mut widths {let reduction=(overflow/remaining as f32).min(*width);*width-=reduction;overflow-=reduction;remaining-=1;}
                        let total_weight=visible.iter().map(|(_,child)|child.weight.max(0.0)).sum::<f32>();
                        let mut remaining_weight=if self.weight_sum>0.0 {self.weight_sum}else{total_weight};
                        let mut excess=(inner.width-margins-widths.iter().sum::<f32>()).max(0.0);
                        for (width,(_,child)) in widths.iter_mut().zip(&visible) {
                            let weight=child.weight.max(0.0);
                            if weight>0.0 && remaining_weight>f32::EPSILON {
                                let share=(excess*weight/remaining_weight).max(0.0);
                                *width+=share;excess-=share;remaining_weight-=weight;
                            }
                        }
                    }
                    let total_w: f32 = widths.iter().sum();
                    let max_scroll = if tab_scrollable { (total_w - inner.width).max(0.0) } else { 0.0 };
                    let mut scroll_offset = if tab_scrollable {
                        state.tab_scroll_offsets.get(&key).copied().unwrap_or(0.0).clamp(0.0, max_scroll)
                    } else {
                        0.0
                    };
                    let dragged = if tab_scrollable && enabled {
                        f.scroll_viewport_horizontal(anonymous_child_id(id, 10299), inner, &mut scroll_offset, max_scroll)
                    } else {
                        false
                    };
                    let mut clicked=None;
                    let mut draw_tabs = |f: &mut Frame<'_>| {
                        let mut cursor = if state.rtl {
                            inner.x + inner.width + scroll_offset
                        } else {
                            inner.x - scroll_offset
                        };
                        for (slot,(index,child)) in visible.iter().enumerate(){
                            let width=widths[slot];let m=if measured {child.resolved_margin(state)}else{[0.0;4]};
                            let x=if state.rtl{cursor-=m[2]+width;let x=cursor;cursor-=m[0];x}else{cursor+=m[0];let x=cursor;cursor+=width+m[2];x};
                            let bounds=Rect{x,y:inner.y+m[1],width,height:(inner.height-m[1]-m[3]).max(0.0)};
                            if child.effective_visibility(state)==Visibility::Invisible {
                                if *index==selected {strip_bounds=Some(bounds);}
                                continue;
                            }
                            let label = state.text.get(child.id).map(String::as_str).unwrap_or(child.text).to_owned();
                            let hit = f.target(anonymous_child_id(id, 10300 + index), bounds, &label, self.enabled && child.enabled, Role::Button);
                            if *index==selected{strip_bounds=Some(bounds);if !m3 && tab_opt.is_none() {f.paint_rect(bounds,0.0,t.surface_highest);}}
                            let cmd_start = f.commands.len();
                            f.without_input(true,|f|{child.draw(f,bounds,state,child_view_id(id,*index,child));});
                            if (m3 || tab_opt.is_some()) && child.color.is_none() && child.text_color_states.is_empty() {
                                let desired = if *index == selected {
                                    match tab_style {
                                        crate::systems::material::TabStyle::Primary => indicator_color,
                                        crate::systems::material::TabStyle::Secondary => t.text,
                                    }
                                } else {
                                    t.on_surface_variant
                                };
                                for cmd in &mut f.commands[cmd_start..] {
                                    if let DrawCommand::Text { color, .. } = cmd {
                                        *color = desired;
                                    }
                                }
                            }
                            f.state_layer(hit,0.0,t.text);
                            if let Some(badge) = self.nav_badge_for_item(slot, child.id, state)
                                .or_else(|| if *index != slot { self.nav_badge_for_item(*index, child.id, state) } else { None })
                            {
                                let ext = self
                                    .material_badge_ext(Some(slot), child.id)
                                    .or_else(|| if *index != slot { self.material_badge_ext(Some(*index), child.id) } else { None });
                                paint_nav_badge_ext(f, bounds, badge, ext, state, if self.enabled && child.enabled { 1.0 } else { 0.38 });
                            }
                            if *index==selected && show_strip {
                                paint_tab_indicator(f, bounds, &label);
                            }
                            if hit.clicked{clicked=Some((*index, slot));}
                        }
                    };
                    if tab_scrollable {
                        f.within(inner, |f| f.without_input(dragged, |f| draw_tabs(f)));
                    } else {
                        draw_tabs(f);
                    }
                    if tab_scrollable {
                        let ensure_slot = clicked
                            .map(|(_, s)| s)
                            .or_else(|| sel_changed.then(|| visible.iter().position(|(i, _)| *i == selected)).flatten());
                        if let Some(sel_slot) = ensure_slot && max_scroll > 0.0 {
                            let start_off: f32 = widths.iter().take(sel_slot).sum();
                            let end_off = start_off + widths[sel_slot];
                            if start_off < scroll_offset {
                                scroll_offset = start_off;
                            } else if end_off > scroll_offset + inner.width {
                                scroll_offset = (end_off - inner.width).min(max_scroll);
                            }
                        }
                        state.tab_scroll_offsets.insert(key.clone(), scroll_offset.clamp(0.0, max_scroll));
                    }
                    if let Some((index, _)) = clicked {
                        state.displayed_children.insert(key.clone(), index);
                        state.selected_index.insert(key.clone(), index);
                        state.tab_last_index.insert(key.clone(), index);
                        state.tab_widget_clicked.insert(key.clone(), index);
                        if !linked_partners.is_empty() {
                            for partner in &linked_partners {
                                state.selected_index.insert(partner.clone(), index);
                                state.displayed_children.insert(partner.clone(), index);
                            }
                            f.continuous(b);
                        }
                        if let Some(host_id) = state.active_tab_host.clone() {
                            if state.tab_hosts.get(&host_id).is_some_and(|host| host.widget_id == key) {
                                state.set_tab_host_current(&host_id, index);
                            }
                        }
                        action = Some(self.id);
                    }
                }
                if custom_strips && self.relative_rules.iter().all(|rule|rule.kind!=147||rule.anchor!="false") {
                    if let Some(selected)=strip_bounds {
                        let d=state.effective_density();
                        for (kind,left) in [(191,true),(192,false)] {
                            let rule=self.relative_rules.iter().find(|rule|rule.kind==kind);
                            if rule.is_some_and(|rule|rule.anchor=="null") {continue;}
                            let (width,height,color)=rule.and_then(|rule|{
                                let mut fields=rule.anchor.split(',');
                                Some((fields.next()?.parse::<f32>().ok()?*d,fields.next()?.parse::<f32>().ok()?*d,Color(fields.next()?.parse().ok()?,fields.next()?.parse().ok()?,fields.next()?.parse().ok()?,fields.next()?.parse().ok()?)))
                            }).unwrap_or((0.0,indicator_h,indicator_color));
                            let (x,right)=if left {(b.x.min(selected.x-width),selected.x)}else{(selected.x+selected.width,(b.x+b.width).max(selected.x+selected.width+width))};
                            if right>x && height>0.0 {f.paint_rect(Rect{x,y:b.y+b.height-height,width:right-x,height},0.0,color);}
                        }
                    }
                }
            }
            Kind::Gallery => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let count=state.list_items.get(&key).map_or(0,Vec::len);
                let generation=state.adapter_flipper_data_generation.get(&key).copied().unwrap_or(0);
                if state.gallery_rendered_generation.insert(key.clone(),generation).is_some_and(|old|old!=generation){f.ui.active=None;}
                if count>0 {
                    let mut selected=state.displayed_children.get(&key).copied().unwrap_or(0).min(count-1);
                    let rule=|kind|self.relative_rules.iter().find(|rule|rule.kind==kind).and_then(|rule|rule.anchor.parse::<f32>().ok());
                    let template=state.list_templates.get(&key).copied();
                    let width=template.and_then(|template|match template.row.width {Size::Dp(value)=>Some(state.dp(value)),_=>None}).unwrap_or((inner.width*0.45).min(state.dp(160.0))).max(1.0);
                    let spacing=state.dp(rule(144).unwrap_or(0.0));let step=(width+spacing).max(1.0);let unselected_alpha=rule(145).unwrap_or(0.5).clamp(0.0,1.0);let duration=rule(146).filter(|value|*value>0.0).unwrap_or(400.0) as f64/1000.0;
                    let mut gesture=state.gallery_gestures.remove(&key).unwrap_or_default();let now=f.time();let direction=if state.rtl{-1.0}else{1.0};if gesture.rtl!=state.rtl {gesture=GalleryGesture{rtl:state.rtl,..GalleryGesture::default()};}
                    if self.enabled && f.enabled && f.focused_id()==Some(id) {
                        let movement=if f.pressed_key(Key::Left){Some(-1)}else if f.pressed_key(Key::Right){Some(1)}else{None};
                        if let Some(movement)=movement {
                            let next=(selected as isize+if state.rtl{-movement}else{movement}).clamp(0,count as isize-1) as usize;
                            if next!=selected {
                                let from=gesture.offset+(next as f32-selected as f32)*step;
                                selected=next;state.displayed_children.insert(key.clone(),selected);action=Some(self.id);
                                gesture.fling=None;gesture.origin=None;gesture.dragging=false;
                                if f.reduced_motion(){gesture.offset=0.0;gesture.settle=None;}else{gesture.offset=from;gesture.settle=Some((now,from,duration));f.continuous(inner);}
                            }
                        }
                        if f.pressed_key(Key::Enter){state.gallery_clicked.insert(key.clone(),selected);action=Some(self.id);}
                    }
                    if let Some((start,initial,distance,fling_duration,initial_selection))=gesture.fling {
                        let p=((now-start)/fling_duration).clamp(0.0,1.0) as f32;
                        let raw=initial+distance*gallery_spline_position(p);let min=-(count.saturating_sub(1+initial_selection) as f32)*step;let max=initial_selection as f32*step;let position=raw.clamp(min,max);let reached_edge=(distance<0.0&&raw<=min)||(distance>0.0&&raw>=max);
                        let previous=selected;selected=((initial_selection as isize)-(position/step).round() as isize).clamp(0,count as isize-1) as usize;if previous!=selected{action=Some(self.id);}
                        gesture.offset=position+(selected as f32-initial_selection as f32)*step;
                        state.displayed_children.insert(key.clone(),selected);
                        if p>=1.0||reached_edge {gesture.fling=None;if gesture.offset.abs()>0.1&&!f.reduced_motion(){gesture.settle=Some((now,gesture.offset,duration));}else{gesture.offset=0.0;}}else{f.continuous(inner);}
                    }
                    if let Some((start,from,duration))=gesture.settle {let p=((now-start)/duration).clamp(0.0,1.0) as f32;gesture.offset=from*(1.0-p)*(1.0-p);if p>=1.0{gesture.settle=None;gesture.offset=0.0;}else{f.continuous(inner);}}
                    if self.enabled&&f.enabled&&f.input.pressed&&f.pointer_in(inner){gesture.origin=f.input.pointer;gesture.dragging=false;gesture.settle=None;gesture.fling=None;gesture.drag_base=gesture.offset;gesture.samples.clear();if let Some(pointer)=f.input.pointer{gesture.samples.push((now,pointer[0]*direction));}}
                    if let (Some(origin),Some(pointer))=(gesture.origin,f.input.pointer){if !f.input.released&&!f.input.cancelled&&!f.input.blurred {let logical_pointer=pointer[0]*direction;let delta=(pointer[0]-origin[0])*direction;if gesture.samples.last().map_or(true,|(_,x)|*x!=logical_pointer){gesture.samples.push((now,logical_pointer));if gesture.samples.len()>8{gesture.samples.remove(0);}}if delta.abs()>8.0{gesture.dragging=true;gesture.offset=gesture.drag_base+delta;f.ui.active=None;f.continuous(inner);}}}
                    let dragged=gesture.dragging;
                    if gesture.origin.is_some()&&(f.input.released||f.input.cancelled||f.input.blurred){if dragged {
                        let velocity=gesture.samples.last().and_then(|&(last_t,last_x)|if now-last_t<0.1 {gesture.samples.iter().rev().find(|(t,_)|last_t-*t>=0.01).map(|&(first_t,first_x)|(last_x-first_x)/(last_t-first_t) as f32)}else{None}).unwrap_or(0.0);
                        if !f.input.cancelled&&!f.input.blurred&&!f.reduced_motion()&&velocity.abs()>=50.0 {
                            // Android Scroller's default friction and physical coefficient in dp units.
                            let rate=0.78_f32.ln()/0.9_f32.ln();let friction=0.015*9.80665*39.37*160.0*0.84;let l=(0.35*velocity.abs()/friction).ln();
                            let distance=velocity.signum()*friction*(rate/(rate-1.0)*l).exp();let fling_duration=(l/(rate-1.0)).exp().max(0.001) as f64;
                            if distance.abs()>0.1 {gesture.fling=Some((now,gesture.offset,distance,fling_duration,selected));f.continuous(inner);}
                        }
                        if gesture.fling.is_none(){let old=selected;if !f.input.cancelled&&!f.input.blurred {selected=((selected as isize)-(gesture.offset/step).round() as isize).clamp(0,count as isize-1) as usize;}gesture.offset+=(selected as f32-old as f32)*step;if selected!=old {state.displayed_children.insert(key.clone(),selected);action=Some(self.id);}if !f.reduced_motion()&&gesture.offset.abs()>0.1{gesture.settle=Some((now,gesture.offset,duration));f.continuous(inner);}else{gesture.offset=0.0;}}
                    }gesture.origin=None;gesture.dragging=false;}
                    let mut row_state=template.map(|_|spinner_template_state(state,state.rtl));
                    let items=state.list_items.get(&key).unwrap();let center=selected as f32-gesture.offset/step;let extent=(inner.width+width)/(2.0*step)+2.0;let start=(center-extent).floor().max(0.0).min(count as f32) as usize;let end=((center+extent).ceil().max(0.0) as usize).saturating_add(1).min(count);let mut clicked=None;
                    f.within(inner,|f|{for index in start..end {let x=inner.x+(inner.width-width)/2.0+((index as f32-selected as f32)*step+gesture.offset)*direction;let bounds=Rect{x,y:inner.y,width,height:inner.height};if bounds.intersect(inner).width<=0.0{continue;}let hit=f.target(anonymous_child_id(id,9500+index),bounds,&items[index],self.enabled,Role::Button);if hit.clicked&&!dragged {clicked=Some(index);}let opacity=if index==selected{1.0}else{unselected_alpha};f.with_opacity(opacity,|f|{if let (Some(template),Some(row_state))=(template,row_state.as_mut()){paint_spinner_template(f,template.row,template.text_id,&items[index],bounds,row_state,anonymous_child_id(id,9600+index),hit,false,None);}else{f.styled_text(bounds,&items[index],t.text,TextStyle{size:20.0,align:1,..TextStyle::default()});}});}});
                    let gallery_hit=f.target(id,inner,"Gallery",self.enabled,Role::Button);
                    if gallery_hit.clicked && clicked.is_none() && f.input.pointer.is_some() {f.request_focus(id);}
                    if let Some(index)=clicked {f.request_focus(id);state.gallery_clicked.insert(key.clone(),index);if index!=selected {let from=gesture.offset+(index as f32-selected as f32)*step;state.displayed_children.insert(key.clone(),index);if !f.reduced_motion(){gesture.offset=from;gesture.settle=Some((now,from,duration));f.continuous(inner);}}else if gesture.offset.abs()>0.1 {if f.reduced_motion(){gesture.offset=0.0;}else{gesture.settle=Some((now,gesture.offset,duration));f.continuous(inner);}}action=Some(self.id);}
                    state.gallery_gestures.insert(key,gesture);
                }else{state.gallery_gestures.remove(&key);state.displayed_children.remove(&key);}
            }
            Kind::StackView => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let items=state.list_items.get(&key).map(Vec::as_slice).unwrap_or(&[]);let count=items.len();
                let looping=*state.stack_looping.entry(key.clone()).or_insert_with(||self.relative_rules.iter().any(|rule|rule.kind==137&&rule.anchor=="true"));
                let generation=state.adapter_flipper_data_generation.get(&key).copied().unwrap_or(0);if state.stack_rendered_generation.insert(key.clone(),generation).is_some_and(|old|old!=generation) {state.stack_gestures.remove(&key);state.stack_pending_step.remove(&key);if f.ui.active==Some(id){f.ui.active=None;}}
                if count>0 {
                    let selected=state.displayed_children.entry(key.clone()).or_default();if *selected>=count{*selected=0;}let mut current=*selected;
                    let template=state.list_templates.get(&key).copied();let mut row_state=template.map(|_|spinner_template_state(state,state.rtl));
                    let mut motion=state.stack_gestures.remove(&key).unwrap_or_default();let now=f.time();
                    if let Some((old,direction))=state.stack_pending_step.remove(&key) {motion=StackGesture::default();if !f.reduced_motion()&&old<count {let slide=inner.height*0.7;if direction>0 {motion.outgoing=Some((now,0.0,slide,0.0,old,0.4));}else{motion.incoming=Some((now,slide,0.0,0.4));}f.continuous(inner);}}
                    if self.enabled&&f.enabled&&motion.origin.is_none()&&f.pointer_in(inner)&&f.input.scroll[1]!=0.0&&motion.last_wheel.is_none_or(|last|now-last>0.1) {
                        let direction=if f.input.scroll[1]>0.0{1}else{-1};motion.last_wheel=Some(now);
                        let old=current;let next=if looping {(current as isize+direction).rem_euclid(count as isize) as usize}else{(current as isize+direction).clamp(0,count as isize-1) as usize};
                        if next!=old {current=next;state.displayed_children.insert(key.clone(),next);motion.offset=0.0;motion.offset_x=0.0;motion.settle=None;motion.incoming=None;motion.outgoing=None;if !f.reduced_motion(){let slide=inner.height*0.7;if direction>0 {motion.outgoing=Some((now,0.0,slide,0.0,old,0.4));}else{motion.incoming=Some((now,slide,0.0,0.4));}f.continuous(inner);}action=Some(self.id);}
                    }
                    let hit=f.target(id,inner,&items[current],self.enabled,Role::Button);
                    let front=Rect{x:inner.x,y:inner.y+inner.height*0.1,width:inner.width*0.9,height:inner.height*0.9};
                    if self.enabled&&f.enabled&&f.input.pressed&&f.pointer_in(inner) {motion.origin=f.input.pointer;motion.tap_in_front=f.input.pointer.is_some_and(|point|front.contains(point));motion.offset=0.0;motion.offset_x=0.0;motion.drag_delta=0.0;motion.direction=0;motion.dragging=false;motion.settle=None;motion.outgoing=None;motion.incoming=None;}
                    if let (Some(origin),Some(pointer))=(motion.origin,f.input.pointer) {if !f.input.cancelled&&!f.input.blurred {let delta=pointer[1]-origin[1];if !motion.dragging&&!f.input.released&&delta.abs()>8.0 {motion.dragging=true;motion.direction=if delta>0.0{1}else{-1};}if motion.dragging {motion.drag_delta=delta;let direction=f32::from(motion.direction);let adjusted=(delta*direction-8.0).max(0.0)*direction;let slide=inner.height*0.7;let edge=count==1||!looping&&((direction<0.0&&current==0)||(direction>0.0&&current+1>=count));motion.offset=if edge {(adjusted*0.2).clamp(-slide*0.2,slide*0.2)}else{adjusted.clamp(-slide,slide)};motion.offset_x=((pointer[0]-origin[0])*0.2).clamp(-slide*0.4,slide*0.4);f.ui.active=None;f.continuous(inner);}}}
                    let dragged=motion.dragging;
                    if motion.origin.is_some()&&(f.input.released||f.input.cancelled||f.input.blurred||!self.enabled||!f.enabled) {
                        let offset=motion.offset;let offset_x=motion.offset_x;let direction=f32::from(motion.direction);let can_move=looping||(direction>0.0&&current+1<count)||(direction<0.0&&current>0);let commit=dragged&&f.input.released&&!f.input.cancelled&&!f.input.blurred&&count>1&&can_move&&motion.drag_delta*direction>inner.height*0.14;
                        let slide=inner.height*0.7;let max_distance=slide*(1.0_f32+0.4*0.4).sqrt();
                        if commit {let old=current;current=if offset>0.0 {(current+1)%count}else{(current+count-1)%count};state.displayed_children.insert(key.clone(),current);action=Some(self.id);if !f.reduced_motion(){if offset>0.0 {let duration=((1.0-offset.abs().min(max_distance)/max_distance.max(1.0))*0.4).max(0.001) as f64;motion.outgoing=Some((now,offset,slide,offset_x,old,duration));}else{let from=(slide+offset).max(0.0);let duration=(from/max_distance.max(1.0)*0.4).max(0.001) as f64;motion.incoming=Some((now,from,offset_x,duration));}f.continuous(inner);}motion.offset=0.0;motion.offset_x=0.0;}
                        else if dragged&&!f.reduced_motion(){let duration=(offset.hypot(offset_x)/max_distance.max(1.0)*0.4).max(0.001) as f64;motion.settle=Some((now,offset,offset_x,duration));f.continuous(inner);}else{motion.offset=0.0;motion.offset_x=0.0;}
                        motion.origin=None;motion.dragging=false;
                    }
                    if let Some((start,from,from_x,duration))=motion.settle {let p=((now-start)/duration).clamp(0.0,1.0) as f32;motion.offset=from*(1.0-p);motion.offset_x=from_x*(1.0-p);if p>=1.0{motion.settle=None;motion.offset=0.0;motion.offset_x=0.0;}else{f.continuous(inner);}}
                    let visible=if looping{count.min(4)}else{(count-current).min(4)};let child_w=inner.width*0.9;let child_h=inner.height*0.9;
                    let mut paint=|f:&mut Frame<'_>,index:usize,depth:usize,offset:f32,offset_x:f32,opacity:f32| {let r=1.0-(depth as f32/3.0);let bounds=Rect{x:inner.x+(1.0-r)*inner.width*0.1+offset_x,y:inner.y+r*inner.height*0.1+offset,width:child_w,height:child_h};let progress=(offset.abs()/(inner.height*0.7).max(1.0)).clamp(0.0,1.0);let angle=((progress-0.2)/0.8).clamp(0.0,1.0)*std::f32::consts::FRAC_PI_2;let scale=angle.cos().max(0.001);f.with_opacity(opacity,|f|f.within(inner,|f|{if scale<1.0 {f.commands.push(DrawCommand::PushScaleY{origin:bounds.y+bounds.height/2.0,scale});}if let (Some(template),Some(row_state))=(template,row_state.as_mut()) {paint_spinner_template(f,template.row,template.text_id,&items[index],bounds,row_state,anonymous_child_id(id,9100+index),hit,false,None);}else{f.styled_text(bounds,&items[index],t.text,TextStyle{size:20.0,align:1,..TextStyle::default()});}if scale<1.0 {f.commands.push(DrawCommand::PopScaleY);}}));};
                    let slide=(inner.height*0.7).max(1.0);let alpha=|offset:f32|((0.7-offset.abs()/slide)/0.7).clamp(0.0,1.0);
                    let edge_up=(count==1||!looping&&current==0)&&motion.offset<0.0;let mut front_offset=if edge_up{motion.offset}else{motion.offset.max(0.0)};let mut front_x=if motion.offset>=0.0||edge_up{motion.offset_x}else{0.0};if let Some((start,from,from_x,duration))=motion.incoming {let p=((now-start)/duration).clamp(0.0,1.0) as f32;if p<1.0 {front_offset=from*(1.0-p);front_x=from_x*(1.0-p);f.continuous(inner);}else{motion.incoming=None;}}
                    let edge_drag=count==1||!looping&&((motion.offset<0.0&&current==0)||(motion.offset>0.0&&current+1>=count));
                    for depth in (0..visible).rev() {paint(f,(current+depth)%count,depth,if depth==0 {front_offset}else{0.0},if depth==0 {front_x}else{0.0},if depth==0&&!edge_drag {alpha(front_offset)}else{1.0});}
                    if motion.offset<0.0 && count>1 && !edge_up {let previous=(current+count-1)%count;let offset=(slide+motion.offset).max(0.0);paint(f,previous,0,offset,motion.offset_x,alpha(offset));}
                    if let Some((start,from,to,from_x,old,duration))=motion.outgoing {let p=((now-start)/duration).clamp(0.0,1.0) as f32;if old<count && p<1.0 {let offset=from+(to-from)*p;paint(f,old,0,offset,from_x*(1.0-p),alpha(offset));f.continuous(inner);}else{motion.outgoing=None;}}
                    let keyboard=f.input.key.is_some_and(|input|matches!(input.key,Key::Enter|Key::Space));
                    if hit.clicked&&!dragged&&(keyboard||motion.tap_in_front&&f.input.pointer.is_some_and(|point|front.contains(point))) {state.stack_clicked.insert(key.clone(),current);action=Some(self.id);}
                    state.stack_gestures.insert(key,motion);
                }else{state.displayed_children.remove(&key);state.stack_gestures.remove(&key);}
            }
            Kind::AdapterViewFlipper => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let template=state.list_templates.get(&key).copied();let mut row_state=template.map(|_|spinner_template_state(state,state.rtl));
                let items=state.list_items.get(&key);let count=items.map_or(0,Vec::len);
                let generation=state.adapter_flipper_data_generation.get(&key).copied().unwrap_or(0);
                let changed=state.adapter_flipper_rendered_generation.insert(key.clone(),generation).is_some_and(|previous|previous!=generation);
                if changed {state.transitions.remove(&id);state.adapter_flipper_animated.remove(&key);}
                if count>0 {
                    let now=f.time();let selected=state.displayed_children.entry(key.clone()).or_default();if *selected>=count{*selected=0;}
                    let started=*state.flipper_running.entry(key.clone()).or_insert(self.auto_start);
                    let running=started&&!state.adapter_flipper_host_advanced.contains(&key);
                    let visible=inner.intersect(f.clip).width>0.0&&inner.intersect(f.clip).height>0.0;
                    let epoch=f.ui.epoch;let pause_generation=f.ui.pause_generation;
                    let clock=state.flipper_clocks.entry(id).or_insert(FlipperClock{running,next:now+self.flip_interval,epoch,pause_generation,visible});
                    if running!=clock.running||(visible&&(!clock.visible||epoch.saturating_sub(clock.epoch)>1||clock.pause_generation!=pause_generation)){*clock=FlipperClock{running,next:now+self.flip_interval,epoch,pause_generation,visible};}
                    clock.epoch=epoch;clock.pause_generation=pause_generation;clock.visible=visible;
                    if running&&visible&&count>1&&self.flip_interval>0.0 {if now>=clock.next{*selected=(*selected+1)%count;clock.next=now+self.flip_interval;}f.wake_at(clock.next);}
                    let selected=*selected;let transition=state.transitions.entry(id).or_insert(ViewTransition{current:selected,previous:selected,started:now,active:false});
                    if transition.current!=selected {*transition=ViewTransition{current:selected,previous:transition.current,started:now,active:true};}
                    let scale_in = self.scale_anim(RULE_SCALE_IN);
                    let scale_out = self.scale_anim(RULE_SCALE_OUT);
                    let rotate_in = self.rotate_anim(RULE_ROTATE_IN);
                    let rotate_out = self.rotate_anim(RULE_ROTATE_OUT);
                    let elev_in = self.elevation_anim(RULE_ELEVATION_IN);
                    let elev_out = self.elevation_anim(RULE_ELEVATION_OUT);
                    let has_in_motion = self.translate_in.is_some() || scale_in.is_some() || rotate_in.is_some() || elev_in.is_some();
                    let has_out_motion = self.translate_out.is_some() || scale_out.is_some() || rotate_out.is_some() || elev_out.is_some();
                    let incoming=self.animation_in.unwrap_or(if has_in_motion{AlphaAnimation{from:1.0,to:1.0,duration:0.0,start_offset:0.0,linear:true}}else{AlphaAnimation{from:0.0,to:1.0,duration:0.2,start_offset:0.0,linear:false}});
                    let outgoing=self.animation_out.unwrap_or(if has_out_motion{AlphaAnimation{from:1.0,to:1.0,duration:0.0,start_offset:0.0,linear:true}}else{AlphaAnimation{from:1.0,to:0.0,duration:0.2,start_offset:0.0,linear:false}});
                    let duration=incoming.total_duration().max(outgoing.total_duration()).max(self.translate_in.map_or(0.0,|a|a.total_duration())).max(self.translate_out.map_or(0.0,|a|a.total_duration())).max(scale_in.map_or(0.0,|a|a.total_duration())).max(scale_out.map_or(0.0,|a|a.total_duration())).max(rotate_in.map_or(0.0,|a|a.total_duration())).max(rotate_out.map_or(0.0,|a|a.total_duration())).max(elev_in.map_or(0.0,|a|a.total_duration())).max(elev_out.map_or(0.0,|a|a.total_duration()));
                    if transition.active && (f.reduced_motion()||now-transition.started>=duration||transition.previous>=count) {transition.active=false;state.adapter_flipper_animated.insert(key.clone());}
                    let transition=*transition;if transition.active{f.continuous(inner);}
                    let identity=state.list_item_ids.get(&key).and_then(|ids|ids.get(selected)).map_or_else(||(selected as u64)^generation.rotate_left(32),|value|*value as u64);
                    let hit=f.target(toolbar_menu_target_id(id,identity,false),inner,&items.unwrap()[selected],self.enabled,Role::Button);
                    let elapsed=(now-transition.started).max(0.0);
                    let in_alpha=if transition.active{incoming.sample(elapsed,true)}else if state.adapter_flipper_animated.contains(&key){incoming.to}else{1.0};
                    let out_alpha=if transition.active{outgoing.sample(elapsed,true)}else{0.0};
                    for (index,alpha,outgoing_item) in [(transition.previous,out_alpha,true),(selected,in_alpha,false)] {
                        if alpha<=0.0{continue;}let label=&items.unwrap()[index];
                        let translate=if outgoing_item{self.translate_out}else{self.translate_in};
                        let delta=if transition.active||(!outgoing_item&&state.adapter_flipper_animated.contains(&key)){translate.map_or([0.0,0.0],|animation|animation.sample(elapsed,inner,inner,true))}else{[0.0,0.0]};
                        let moved=Rect{x:inner.x+delta[0],y:inner.y+delta[1],..inner};
                        let item_scale = if transition.active { if outgoing_item { scale_out } else { scale_in } } else { None };
                        let item_rotate = if transition.active { if outgoing_item { rotate_out } else { rotate_in } } else { None };
                        let item_elev = if transition.active {
                            if outgoing_item { elev_out.map(|a| a.sample(elapsed)) } else { elev_in.map(|a| a.sample(elapsed)) }
                        } else if !outgoing_item && state.adapter_flipper_animated.contains(&key) {
                            elev_in.map(|a| a.to_dp.max(0.0))
                        } else {
                            None
                        };
                        f.with_opacity(alpha,|f|f.within(inner,|f|{
                            let pushed = push_transition_transforms(f, item_scale, item_rotate, elapsed, moved, inner, state, true);
                            if let Some(elev_dp) = item_elev && elev_dp > 0.0 {
                                let elev_r = template.map_or(0.0, |tpl| {
                                    if tpl.row.radius < 0.0 {
                                        moved.width.min(moved.height) * 0.5
                                    } else if tpl.row.radius > 0.0 {
                                        state.dp(tpl.row.radius)
                                    } else {
                                        tpl.row.resolved_state_radius(state)
                                    }
                                });
                                paint_elevation_dp(f, moved, elev_r, elev_dp, state.effective_density());
                            }
                            if let (Some(template),Some(row_state))=(template,row_state.as_mut()) {
                                if item_elev.is_some() {
                                    row_state.suppress_next_elevation = true;
                                }
                                paint_spinner_template(f,template.row,template.text_id,label,moved,row_state,anonymous_child_id(id,9100+index),hit,false,None);
                            }else{f.styled_text(moved,label,t.text,TextStyle{size:20.0,align:1,..TextStyle::default()});}
                            for _ in 0..pushed { f.pop_transform(); }
                        }));
                    }
                    if hit.clicked{state.adapter_flipper_clicked.insert(key,selected);action=Some(self.id);}
                } else {state.displayed_children.remove(&key);state.flipper_clocks.remove(&id);state.transitions.remove(&id);state.adapter_flipper_animated.remove(&key);}
            }
            Kind::Toolbar|Kind::ActionMenuView => {
                if let Some(bab) = bottom_app_bar {
                    let fab_align = bab.alignment;
                    let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
                    let slot_w = state.dp(48.0).min(inner.width);
                    let slot_h = state.dp(48.0).min(inner.height);
                    let cutout_r = state.dp(28.0) + state.dp(bab.cradle_margin_dp);
                    let cutout_span: Option<(f32, f32)> = bab.has_ext.then(|| {
                        let logical_cx = match fab_align {
                            BottomAppBarFabAlignment::Center => inner.width * 0.5,
                            BottomAppBarFabAlignment::End => (inner.width - state.dp(44.0)).max(0.0),
                        };
                        ((logical_cx - cutout_r).max(0.0), (logical_cx + cutout_r).min(inner.width))
                    });
                    let place_slot_logical = |cur: f32, w: f32| -> Option<f32> {
                        let mut c = cur;
                        if let Some((lo, hi)) = cutout_span
                            && c < hi
                            && c + w > lo
                        {
                            c = hi;
                        }
                        (c + w <= inner.width + 0.5).then_some(c)
                    };
                    let mut cursor = state.dp(4.0);
                    if !self.image_src.is_empty()
                        && slot_w > 0.0
                        && slot_h > 0.0
                        && let Some(nav_logical) = place_slot_logical(cursor, slot_w)
                    {
                        cursor = nav_logical;
                        let nav_x = if state.rtl {
                            inner.x + inner.width - cursor - slot_w
                        } else {
                            inner.x + cursor
                        };
                        let nav_rect = Rect {
                            x: nav_x,
                            y: inner.y + (inner.height - slot_h) * 0.5,
                            width: slot_w,
                            height: slot_h,
                        };
                        let label = self.relative_rules.iter().find(|r| r.kind == 111).map_or("Navigation", |r| r.anchor);
                        let hit = f.target(anonymous_child_id(id, 9900), nav_rect, label, self.enabled, Role::Button);
                        self.paint_state_layer(f, hit, slot_h * 0.5, None, t.on_surface_variant);
                        f.focus_ring(hit);
                        let icon_s = state.dp(24.0).min(slot_w).min(slot_h);
                        let icon_rect = Rect {
                            x: nav_rect.x + (nav_rect.width - icon_s) * 0.5,
                            y: nav_rect.y + (nav_rect.height - icon_s) * 0.5,
                            width: icon_s,
                            height: icon_s,
                        };
                        let icon_color = t.on_surface_variant.opacity(if hit.enabled { 1.0 } else { 0.38 });
                        let mirror_nav = self.is_auto_mirrored() && state.rtl;
                        if mirror_nav {
                            f.push_transform(
                                [icon_rect.x + icon_rect.width * 0.5, icon_rect.y + icon_rect.height * 0.5],
                                [0.0, 0.0],
                                [-1.0, 1.0],
                                0.0,
                            );
                        }
                        if let Some(img) = state.images.get(self.image_src).cloned() {
                            let tinted = tint_image(state, &img, Some(icon_color), 1);
                            f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                        } else {
                            let icon = parse_material_icon(self.image_src).unwrap_or(MaterialIcon::Menu);
                            f.paint_material_icon(icon_rect, icon, icon_color);
                        }
                        if mirror_nav {
                            f.pop_transform();
                        }
                        if hit.clicked {
                            state.toolbar_navigation_clicked.insert(key.clone());
                            action = Some(self.id);
                        }
                        cursor += slot_w;
                    }
                    let nav_items = self.nav_items();
                    for (i, item) in nav_items.iter().enumerate() {
                        if slot_w <= 0.0 || slot_h <= 0.0 {
                            break;
                        }
                        let Some(logical_x) = place_slot_logical(cursor, slot_w) else {
                            break;
                        };
                        cursor = logical_x + slot_w;
                        let sx = if state.rtl {
                            inner.x + inner.width - logical_x - slot_w
                        } else {
                            inner.x + logical_x
                        };
                        let slot = Rect {
                            x: sx,
                            y: inner.y + (inner.height - slot_h) * 0.5,
                            width: slot_w,
                            height: slot_h,
                        };
                        let label = if !item.title.is_empty() { item.title } else { item.id };
                        let hit = f.target(anonymous_child_id(id, 9910 + i), slot, label, self.enabled, Role::Button);
                        self.paint_state_layer(f, hit, slot_h * 0.5, None, t.on_surface_variant);
                        f.focus_ring(hit);
                        let icon_s = state.dp(24.0).min(slot.width).min(slot.height);
                        let icon_rect = Rect {
                            x: slot.x + (slot.width - icon_s) * 0.5,
                            y: slot.y + (slot.height - icon_s) * 0.5,
                            width: icon_s,
                            height: icon_s,
                        };
                        let icon_color = t.on_surface_variant.opacity(if hit.enabled { 1.0 } else { 0.38 });
                        if !item.icon_name.is_empty() && let Some(img) = state.images.get(item.icon_name).cloned() {
                            let tinted = tint_image(state, &img, Some(icon_color), 1);
                            f.paint_image(icon_rect, &tinted, ImageFit::Contain, 0.0);
                        } else {
                            let icon = parse_material_icon(item.icon_name).unwrap_or(MaterialIcon::Widgets);
                            f.paint_material_icon(icon_rect, icon, icon_color);
                        }
                        if let Some(badge) = self.nav_badge_for_item(i, item.id, state) {
                            let ext = self.material_badge_ext(Some(i), item.id);
                            paint_nav_badge_ext(f, icon_rect, badge, ext, state, if hit.enabled { 1.0 } else { 0.38 });
                        }
                        if hit.clicked {
                            action = Some(if !item.id.is_empty() { item.id } else { self.id });
                        }
                    }
                    let end_pad = state.dp(16.0);
                    for (i, child) in self.children.iter().enumerate() {
                        if child.effective_visibility(state) == Visibility::Gone {
                            continue;
                        }
                        let child_id = child_view_id(id, i, child);
                        let margin = child.resolved_margin(state);
                        let width = match child.width {
                            Size::Dp(v) => state.dp(v),
                            Size::Match => (inner.width - margin[0] - margin[2]).max(0.0),
                            Size::Wrap => child.layout_intrinsic(true, state, inner.width, f.ui.text_metrics, child_id),
                        }
                        .max(0.0)
                        .min(inner.width);
                        let height = match child.height {
                            Size::Dp(v) => state.dp(v),
                            Size::Match => (inner.height - margin[1] - margin[3]).max(0.0),
                            Size::Wrap => child.layout_intrinsic(false, state, width, f.ui.text_metrics, child_id),
                        }
                        .max(0.0)
                        .min(inner.height);
                        let x = if child.layout_gravity_explicit {
                            let right = match child.layout_gravity[0] {
                                0 => state.rtl,
                                2 => !state.rtl,
                                4 => true,
                                _ => false,
                            };
                            if child.layout_gravity[0] == 1 {
                                inner.x + (inner.width - width) * 0.5 + margin[0] - margin[2]
                            } else if right {
                                inner.x + inner.width - width - margin[2]
                            } else {
                                inner.x + margin[0]
                            }
                        } else {
                            match fab_align {
                                BottomAppBarFabAlignment::Center => inner.x + (inner.width - width) * 0.5,
                                BottomAppBarFabAlignment::End => {
                                    if state.rtl {
                                        inner.x + end_pad.max(margin[0])
                                    } else {
                                        inner.x + inner.width - width - end_pad.max(margin[2])
                                    }
                                }
                            }
                        };
                        let y = match child.layout_gravity[1] {
                            0 if child.layout_gravity_explicit => inner.y + margin[1],
                            2 if child.layout_gravity_explicit => inner.y + inner.height - height - margin[3],
                            _ => inner.y + (inner.height - height + margin[1] - margin[3]) * 0.5,
                        };
                        let rect = Rect { x, y, width, height };
                        let mut child_action = None;
                        f.within(inner, |f| {
                            f.with_enabled(self.enabled, |f| {
                                child_action = child.draw(f, rect, state, child_id);
                            });
                        });
                        if child_action.is_some() {
                            action = child_action;
                        }
                    }
                } else {
                let action_menu=matches!(self.kind,Kind::ActionMenuView);
                let collapsing_child = state.collapsing_toolbar_child;
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let title = if collapsing_child {
                    String::new()
                } else {
                    state.text.get(&key).map_or(self.text,String::as_str).replace(['\n','\r']," ").replace('\u{00ad}',"")
                };
                let subtitle=self.hint.replace(['\n','\r']," ").replace('\u{00ad}',"");
                let menu=state.toolbar_menu_items.get(&key).cloned().unwrap_or_default();
                let mut overlay=state.toolbar_menu_overlays.remove(&key).unwrap_or_default();
                if !self.enabled||!f.enabled {overlay.open=false;}
                let slot_unit = state.dp(48.0);
                let icon_unit = state.dp(24.0);
                let nav=!self.image_src.is_empty();let side=slot_unit.min(inner.height).min(inner.width);
                let left=if state.rtl{inner.x+inner.width-side}else{inner.x};
                if nav {
                    let bounds=Rect{x:left,y:inner.y+(inner.height-side)/2.0,width:side,height:side};
                    let label=self.relative_rules.iter().find(|r|r.kind==111).map_or("Navigate up",|r|r.anchor);
                    let hit=f.target(anonymous_child_id(id,9900),bounds,label,self.enabled&&!overlay.open,Role::Button);f.state_layer(hit,side/2.0,t.text);
                    let mirror_nav = self.is_auto_mirrored() && state.rtl;
                    if let Some(image)=state.images.get(self.image_src) {
                        let icon_rect = bounds.inset(((side-icon_unit)/2.0).max(0.0));
                        if mirror_nav {
                            f.push_transform(
                                [icon_rect.x + icon_rect.width * 0.5, icon_rect.y + icon_rect.height * 0.5],
                                [0.0, 0.0],
                                [-1.0, 1.0],
                                0.0,
                            );
                        }
                        f.paint_image(icon_rect,image,ImageFit::Contain,0.0);
                        if mirror_nav {
                            f.pop_transform();
                        }
                    } else if let Some(icon) = parse_material_icon(self.image_src) {
                        let icon_s = state.dp(24.0).min(side);
                        let icon_rect = Rect { x: bounds.x + (side - icon_s) * 0.5, y: bounds.y + (side - icon_s) * 0.5, width: icon_s, height: icon_s };
                        if mirror_nav {
                            f.push_transform(
                                [icon_rect.x + icon_rect.width * 0.5, icon_rect.y + icon_rect.height * 0.5],
                                [0.0, 0.0],
                                [-1.0, 1.0],
                                0.0,
                            );
                        }
                        f.paint_material_icon(
                            icon_rect,
                            icon,
                            t.text.opacity(if hit.enabled { 1.0 } else { 0.38 }),
                        );
                        if mirror_nav {
                            f.pop_transform();
                        }
                    }
                    if hit.clicked {state.toolbar_navigation_clicked.insert(key.clone());action=Some(self.id);}
                }
                // Measure custom views before reserving title space. Logical start/end
                // reflect Toolbar.LayoutParams gravity and the current direction.
                let inset=|kind|self.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|r.anchor.parse::<f32>().ok());
                let has_insets=(112..=117).any(|kind|inset(kind).is_some());
                let left_inset=inset(114).unwrap_or(0.0);let right_inset=inset(115).unwrap_or(0.0);
                let base_start=inset(112).unwrap_or(if state.rtl{right_inset}else{left_inset});
                let base_end=inset(113).unwrap_or(if state.rtl{left_inset}else{right_inset});
                let current_start=if nav{base_start.max(inset(116).unwrap_or(0.0))}else{base_start};
                let current_end=if menu.iter().any(|item|item.visible){base_end.max(inset(117).unwrap_or(0.0))}else{base_end};
                let leading=if action_menu {0.0}else if has_insets{current_start.max(if nav{side}else{0.0})}else if nav{side+16.0}else{16.0};let mut start=leading.min(inner.width);let mut end=inner.width;
                let mut measured=Vec::new();
                for (i,child) in self.children.iter().enumerate(){
                    if child.effective_visibility(state)==Visibility::Gone {continue;}
                    let child_id=child_view_id(id,i,child);let margin=child.resolved_margin(state);
                    let width=match child.width {Size::Dp(v)=>state.dp(v),Size::Match=>inner.width,_=>child.intrinsic(true,state,child_id)}.max(0.0).min(inner.width);
                    let height=match child.height {Size::Dp(v)=>state.dp(v),Size::Match=>(inner.height-margin[1]-margin[3]).max(0.0),_=>child.layout_intrinsic(false,state,width,f.ui.text_metrics,child_id)}.max(0.0).min(inner.height);
                    let gravity=match child.layout_gravity[0] {1=>1,2=>2,3=>if state.rtl{2}else{0},4=>if state.rtl{0}else{2},_=>0};
                    measured.push((i,child_id,width,height,margin,gravity));
                }
                let child_end: f32=measured.iter().filter(|(_,_,_,_,_,gravity)|*gravity==2).map(|(_,_,width,_,m,_)|width+m[0]+m[2]).sum();
                let cell=if action_menu && inner.width>=56.0 {inner.width/(inner.width/56.0).floor()}else if action_menu {56.0}else{slot_unit};
                let viewport=f.viewport();let allow_icon_text=state.menu_allow_text_with_icon.unwrap_or(viewport.width>=480.0 || viewport.width>viewport.height);
                let show_text:Vec<bool>=menu.iter().map(|item|!item.label.is_empty() && (item.icon.is_none()&&item.icon_drawable.is_none() || item.show_text_with_icon&&allow_icon_text)).collect();
                let toolbar_text_enabled=!action_menu && allow_icon_text && menu.iter().any(|item|item.visible&&item.show_text_with_icon&&(item.icon.is_some()||item.icon_drawable.is_some()));
                let slots=((inner.width-leading-child_end-if action_menu||toolbar_text_enabled {0.0}else{cell}).max(0.0)/cell).floor() as usize;
                let mut costs:Vec<usize>=menu.iter().enumerate().map(|(i,item)|if (action_menu||toolbar_text_enabled) && show_text[i] {let icon=if item.icon.is_some()||item.icon_drawable.is_some(){32.0}else{0.0};(((f.text_width(&item.label,TextStyle{size:14.0,..TextStyle::default()})+icon+8.0)/cell).ceil() as usize).max(2)}else{1}).collect();
                let (visible,overflow)=if action_menu||toolbar_text_enabled {allocate_action_menu_items(&menu,&costs,slots)}else{allocate_android_toolbar_actions(&menu,slots)};
                let more=!overflow.is_empty()&&(inner.width-leading-child_end>=cell||!visible.is_empty());
                if action_menu {
                    let used=visible.iter().map(|&i|costs[i]).sum::<usize>()+usize::from(more);let mut remaining=slots.saturating_sub(used);
                    if visible.len()==1&&!more {costs[visible[0]]+=remaining;}else {
                        while remaining>0 {let minimum=visible.iter().copied().filter(|&i|show_text[i]).map(|i|costs[i]).min();let Some(minimum)=minimum else{break;};let indices:Vec<usize>=visible.iter().copied().filter(|&i|show_text[i]&&costs[i]==minimum).collect();if indices.len()>remaining{break;}remaining-=indices.len();for i in indices {costs[i]+=1;}}
                    }
                }
                let menu_width=(visible.iter().map(|&i|costs[i]).sum::<usize>()+usize::from(more)) as f32*cell;
                let gaps=visible.len().saturating_sub(usize::from(!more));
                let spacer=if action_menu && gaps>0 {(inner.width-menu_width).max(0.0)/gaps as f32}else{0.0};
                let old_open=overlay.open;let mut chosen_menu=None;let mut overflow_bounds=None;
                let mut cursor=inner.width;
                if more {
                    cursor-=cell;let bounds=Rect{x:if state.rtl{inner.x+inner.width-cursor-cell}else{inner.x+cursor},y:inner.y+(inner.height-slot_unit)/2.0,width:cell,height:slot_unit};
                    let hit=f.target(anonymous_child_id(id,9950),bounds,"More options",self.enabled,Role::Button);f.state_layer(hit,slot_unit*0.5,t.text);f.paint_material_icon(Rect{x:bounds.x+(cell-icon_unit)/2.0,y:bounds.y+(slot_unit-icon_unit)/2.0,width:icon_unit,height:icon_unit},MaterialIcon::MoreVert,t.text);
                    if hit.clicked {overlay.open=!overlay.open;}overflow_bounds=Some(bounds);
                }
                for (position,index) in visible.iter().copied().enumerate().rev() {
                    let width=costs[index] as f32*cell;cursor-=width;let item=&menu[index];let logical_x=if action_menu {if visible.len()==1&&!more {(inner.width-width)/2.0}else{visible[..position].iter().map(|&i|costs[i] as f32*cell).sum::<f32>()+position as f32*spacer}}else{cursor};let bounds=Rect{x:if state.rtl{inner.x+inner.width-logical_x-width}else{inner.x+logical_x},y:inner.y+(inner.height-slot_unit)/2.0,width,height:slot_unit};
                    let has_icon=item.icon.is_some()||item.icon_drawable.is_some();let with_text=(action_menu||toolbar_text_enabled)&&show_text[index]&&has_icon;let label_width=f.text_width(&item.label,TextStyle{size:14.0,..TextStyle::default()}).min((width-32.0).max(0.0));let group_width=icon_unit+8.0+label_width;let group_x=bounds.x+(width-group_width).max(0.0)/2.0;let icon_x=if with_text {if state.rtl {group_x+label_width+8.0}else{group_x}}else{bounds.x+(width-icon_unit)/2.0};let icon_bounds=Rect{x:icon_x,y:bounds.y+(slot_unit-icon_unit)/2.0,width:icon_unit,height:icon_unit};
                    let hit=f.target(toolbar_menu_target_id(id,item.id,false),bounds,&item.label,self.enabled&&item.enabled&&!old_open,Role::Button);f.state_layer(hit,slot_unit*0.5,t.text);
                    if let Some((image,tint))=item.icon_drawable.as_ref().and_then(|name|state.images.get(name).cloned().map(|image|(image,state.drawable_tints.get(name).copied()))) {
                        let color=tint.map(|tint|match tint{AndroidDrawableTint::Fixed(color)=>color,AndroidDrawableTint::ControlNormal=>t.on_surface_variant});
                        let image=tint_image(state,&image,color,1);f.with_opacity(if hit.enabled{1.0}else{0.38},|f|{f.paint_image(icon_bounds,&image,ImageFit::Contain,0.0);});
                    }else if let Some(icon)=item.icon {f.paint_material_icon(icon_bounds,icon,t.text.opacity(if hit.enabled{1.0}else{0.38}));}else{f.styled_text(bounds,&item.label,t.text.opacity(if hit.enabled{1.0}else{0.38}),TextStyle{size:if action_menu{14.0}else{12.0},align:1,..TextStyle::default()});}
                    if with_text {let rect=Rect{x:if state.rtl {group_x}else{group_x+32.0},y:bounds.y,width:label_width,height:bounds.height};f.styled_text(rect,&item.label,t.text.opacity(if hit.enabled{1.0}else{0.38}),TextStyle{size:14.0,..TextStyle::default()});}
                    if hit.clicked{chosen_menu=Some(item.id);}
                }
                end=(end-menu_width.max(current_end)).max(start);
                for (_,_,width,_,margin,gravity) in &measured {if *gravity==2{end=(end-width-margin[0]-margin[2]).max(start);}}
                let style_weight=|kind|if self.relative_rules.iter().any(|r|r.kind==kind&&r.anchor=="bold"){700}else{400};
                let (italic, family) = self.text_style_ext();
                let title_style=TextStyle{size:state.sp(inset(125).unwrap_or(20.0)),weight:style_weight(126),align:if state.rtl{2}else{0},italic,family,..TextStyle::default()};
                let subtitle_style=TextStyle{size:state.sp(inset(127).unwrap_or(14.0)),weight:style_weight(128),align:if state.rtl{2}else{0},italic,family,..TextStyle::default()};
                let has_title=!title.is_empty();let has_subtitle=!subtitle.is_empty();let has_text=has_title||has_subtitle;
                let margin=inset(118).unwrap_or(0.0);let margin_start=if has_text{inset(119).unwrap_or(margin)}else{0.0};let margin_end=if has_text{inset(120).unwrap_or(margin)}else{0.0};
                let margin_top=inset(121).unwrap_or(margin);let margin_bottom=inset(122).unwrap_or(margin);
                start=(start+margin_start).min(end);
                let title_width=f.text_width(&title,title_style).max(f.text_width(&subtitle,subtitle_style));
                let available=(end-start-margin_end).max(0.0);let title_width=title_width.min(available);
                let x=if state.rtl{inner.x+inner.width-start-title_width}else{inner.x+start};
                let title_height=title_style.size*1.3;let subtitle_height=subtitle_style.size*(19.0/14.0);
                let text_height=if has_title&&has_subtitle{title_height+subtitle_height-1.0}else if has_title{title_height}else if has_subtitle{subtitle_height}else{0.0};
                let mut top=((inner.height-text_height)/2.0).max(0.0);
                if top<margin_top{top=margin_top;}else if inner.height-text_height-top<margin_bottom{top=(inner.height-text_height-margin_bottom).max(0.0);}
                let y=inner.y+top;
                let shown_title=ellipsize_single_line(&title,title_width,title_style,f.ui.text_metrics,3);
                let shown_subtitle=ellipsize_single_line(&subtitle,title_width,subtitle_style,f.ui.text_metrics,3);
                let xml_color=|kind|self.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|{let values:Vec<u8>=r.anchor.split(',').map(str::parse::<u8>).collect::<Result<_,_>>().ok()?;(values.len()==4).then(||Color(values[0],values[1],values[2],values[3]))});
                let title_color=state.toolbar_title_colors.get(&key).copied().or_else(||xml_color(123)).or_else(||xml_color(129)).unwrap_or(t.text);
                let default_subtitle=if state.theme_style!=AndroidTheme::Bhoo {t.on_surface_variant}else{t.text.opacity(0.7)};
                let subtitle_color=state.toolbar_subtitle_colors.get(&key).copied().or_else(||xml_color(124)).or_else(||xml_color(130)).unwrap_or(default_subtitle);
                f.within(inner,|f|{if has_title{f.styled_text(Rect{x,y,width:title_width,height:title_height},&shown_title,title_color,title_style);}
                    if has_subtitle{f.styled_text(Rect{x,y:y+if has_title{title_height-1.0}else{0.0},width:title_width,height:subtitle_height},&shown_subtitle,subtitle_color,subtitle_style);}});
                start=(start+title_width+margin_end).min(end);let mut trailing=(inner.width-menu_width.max(current_end)).max(0.0);
                for (i,child_id,width,height,margin,gravity) in measured {
                    let extent=width+margin[0]+margin[2];
                    let logical_x=match gravity {2=>{trailing=(trailing-extent).max(start);trailing},1=>((inner.width-extent)/2.0).max(start).min((end-extent).max(start)),_=>{let x=start;start=(start+extent).min(end);x}};
                    let x=if state.rtl{inner.x+inner.width-logical_x-extent+margin[0]}else{inner.x+logical_x+margin[0]};
                    let y=match self.children[i].layout_gravity[1] {0=>inner.y+margin[1],2=>inner.y+inner.height-height-margin[3],_=>inner.y+(inner.height-height+margin[1]-margin[3])/2.0};
                    let rect=Rect{x,y,width,height};let mut child_action=None;
                    f.without_input(overlay.open,|f|f.within(inner,|f|f.with_enabled(self.enabled,|f|child_action=self.children[i].draw(f,rect,state,child_id))));
                    if child_action.is_some(){action=child_action;}
                }
                if let Some(trigger)=overflow_bounds {
                    let viewport=f.viewport();let width=196.0_f32.min(viewport.width);let height=(overflow.len() as f32*48.0+16.0).min((viewport.height-8.0).max(0.0));
                    let x=if state.rtl{trigger.x}else{trigger.x+trigger.width-width}.clamp(viewport.x,(viewport.x+viewport.width-width).max(viewport.x));
                    let y=trigger.y.min((viewport.y+viewport.height-height).max(viewport.y));let bounds=Rect{x,y,width,height};
                    overlay.kind=OverlayKind::Menu;overlay.style=OverlayStyle::AndroidMenu;
                    let command_start=f.commands.len();
                    f.overlay(anonymous_child_id(id,9951),&mut overlay,bounds,false,|f,open| {
                        f.with_gap(0.0,|f|for index in overflow.iter().copied(){
                            let item=&menu[index];let response=f.menu_item(toolbar_menu_target_id(id,item.id,true),&item.label,item.enabled&&self.enabled);
                            if item.checkable {let b=response.bounds;let size=18.0;let rect=Rect{x:if state.rtl{b.x+12.0}else{b.x+b.width-30.0},y:b.y+(b.height-size)/2.0,width:size,height:size};let color=t.on_surface_variant.opacity(if item.enabled&&self.enabled{1.0}else{0.38});let radius=if item.exclusive{size/2.0}else{2.0};f.paint_rect(rect,radius,color);f.paint_rect(rect.inset(2.0),(radius-2.0).max(0.0),t.surface_high);if item.checked {if item.exclusive{f.paint_rect(rect.inset(5.0),size/2.0,color);}else{f.paint_material_icon(rect,MaterialIcon::Check,color);}}}
                            if response.clicked {chosen_menu=Some(item.id);*open=false;}
                        });
                    });
                    f.ui.overlays.extend(f.commands.drain(command_start..));
                } else {overlay.open=false;}
                state.toolbar_menu_overlays.insert(key.clone(),overlay);
                if let Some(item)=chosen_menu {state.toolbar_menu_clicked.insert(key,item);action=Some(self.id);}
                }
            }
            Kind::SearchView => {
                if let Some(bar_hint) = search_bar_hint {
                    let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
                    let hint = if !bar_hint.is_empty() {
                        bar_hint
                    } else if !self.hint.is_empty() {
                        self.hint
                    } else {
                        "Search"
                    };
                    state.text.entry(key.clone()).or_default();
                    state.input_types.insert(key.clone(), 0);
                    if state.search_pending.remove(&key) {
                        action = Some(self.id);
                    }
                    let query = state.text.get(&key).cloned().unwrap_or_default();
                    let bar_rect = inner;
                    let pill_r = if self.radius > 0.0 {
                        scaled_radius
                    } else {
                        bar_rect.height * 0.5
                    };
                    let container_color = background_tint
                        .or(explicit_background)
                        .unwrap_or(match state.theme_style {
                            AndroidTheme::Material3Light => Color(0xEC, 0xE6, 0xF0, 255),
                            AndroidTheme::Material3Dark => Color(0x2B, 0x29, 0x30, 255),
                            _ => t.surface_high,
                        })
                        .opacity(if enabled { 1.0 } else { 0.38 });
                    if let Some(dp) = explicit_elevation {
                        paint_elevation_level(f, bar_rect, pill_r, elevation_level_from_dp(dp), state.effective_density());
                    }
                    f.paint_rect(bar_rect, pill_r, container_color);
                    let response = f.target(id, bar_rect, hint, self.enabled, Role::TextField);
                    self.paint_state_layer(f, response, pill_r, corner_radii, t.on_surface_variant);
                    f.focus_ring(response);
                    if response.clicked {
                        f.request_focus(id);
                        state.search_open.insert(key.clone(), true);
                        state.search_focus_requested.insert(key.clone());
                        let linked_svs: Vec<String> = state
                            .search_bar_links
                            .iter()
                            .filter(|(bar_id, _)| bar_id == &key)
                            .map(|(_, sv_id)| sv_id.clone())
                            .collect();
                        for sv_key in linked_svs {
                            state.search_open.insert(sv_key.clone(), true);
                            state.search_focus_requested.insert(sv_key);
                        }
                        action = Some(self.id);
                        f.continuous(bar_rect);
                    }
                    if response.focused && f.pressed_key(Key::Enter) && state.submit_search(&key) {
                        state.search_pending.remove(&key);
                        action = Some(self.id);
                    }
                    let icon_s = state.dp(24.0).min(bar_rect.height).min(bar_rect.width);
                    let icon_pad = state.dp(16.0);
                    let icon_x = if state.rtl {
                        bar_rect.x + bar_rect.width - icon_pad - icon_s
                    } else {
                        bar_rect.x + icon_pad
                    };
                    let icon_rect = Rect {
                        x: icon_x,
                        y: bar_rect.y + (bar_rect.height - icon_s) * 0.5,
                        width: icon_s,
                        height: icon_s,
                    };
                    let icon_color = t.text.opacity(if enabled { 1.0 } else { 0.38 });
                    f.paint_material_icon(icon_rect, MaterialIcon::Search, icon_color);
                    let lead = state.dp(56.0);
                    let trail = state.dp(16.0);
                    let text_x = bar_rect.x + if state.rtl { trail } else { lead };
                    let text_w = (bar_rect.width - lead - trail).max(0.0);
                    let text_rect = Rect {
                        x: text_x,
                        y: bar_rect.y,
                        width: text_w,
                        height: bar_rect.height,
                    };
                    let (display, text_color) = if query.is_empty() {
                        (hint, t.on_surface_variant.opacity(if enabled { 1.0 } else { 0.38 }))
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
                    f.within(bar_rect, |f| {
                        f.styled_text(
                            text_rect,
                            display,
                            text_color,
                            TextStyle {
                                size: text_size,
                                weight: if self.text_bold { 700 } else { 400 },
                                align: if state.rtl { 2 } else { 0 },
                                letter_spacing: self.letter_spacing_em * text_size,
                                ..TextStyle::default()
                            },
                        );
                    });
                } else {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};let rule=|kind|self.relative_rules.iter().find(|r|r.kind==kind).map(|r|r.anchor);let iconified_default=rule(106)!=Some("false");let opened=*state.search_open.entry(key.clone()).or_insert(!iconified_default);let hint=rule(107).unwrap_or("Search");state.text.entry(key.clone()).or_default();state.input_types.insert(key.clone(),0);if state.search_pending.remove(&key){action=Some(self.id);}
                let side=48.0_f32.min(inner.height).min(inner.width);let place=|logical_end:bool|if logical_end^state.rtl {inner.x+inner.width-side}else{inner.x};
                if !opened {let rect=Rect{x:place(false),y:inner.y+(inner.height-side)/2.0,width:side,height:side};let hit=f.target(anonymous_child_id(id,9800),rect,"Open search",self.enabled,Role::Button);f.state_layer(hit,side/2.0,t.text);paint_android_search_icon(f,rect.inset((side-24.0).max(0.0)/2.0),0,t.text.opacity(if hit.enabled{1.0}else{0.38}));if hit.clicked{state.search_open.insert(key.clone(),true);state.search_focus_requested.insert(key.clone());f.request_focus(id);f.continuous(inner);}}
                else {
                    let query=state.text.get(&key).cloned().unwrap_or_default();let close_visible=!query.is_empty()||iconified_default;let submit_visible=state.search_submit_enabled.contains(&key)&&!query.trim().is_empty();let trailing=usize::from(close_visible)+usize::from(submit_visible);let leading=if iconified_default{0.0}else{side};let occupied=side*trailing as f32;let text_bounds=Rect{x:inner.x+if state.rtl{occupied}else{leading},y:inner.y,width:(inner.width-leading-occupied).max(0.0),height:inner.height};
                    if !iconified_default{let rect=Rect{x:place(false),y:inner.y+(inner.height-side)/2.0,width:side,height:side};paint_android_search_icon(f,rect.inset((side-24.0).max(0.0)/2.0),0,t.text.opacity(0.6));}
                    let response=f.target(id,text_bounds,hint,self.enabled,Role::TextField);let display=if query.is_empty(){hint}else{&query};f.styled_text(text_bounds,display,t.text.opacity(if query.is_empty(){0.55}else{1.0}),TextStyle{size:16.0,padding:8.0,..TextStyle::default()});let focused=response.focused&&response.enabled;let underline_y=inner.y+inner.height-1.0;f.paint_line([inner.x,underline_y],[inner.x+inner.width,underline_y],if focused{2.0}else{1.0},if focused{t.primary}else{t.outline});
                    if close_visible {let x=if state.rtl{inner.x+if submit_visible{side}else{0.0}}else{inner.x+inner.width-side-if submit_visible{side}else{0.0}};let rect=Rect{x,y:inner.y+(inner.height-side)/2.0,width:side,height:side};let hit=f.target(anonymous_child_id(id,9801),rect,if query.is_empty(){"Close search"}else{"Clear search"},self.enabled,Role::Button);f.state_layer(hit,side/2.0,t.text);paint_android_search_icon(f,rect.inset((side-24.0).max(0.0)/2.0),1,t.text.opacity(if hit.enabled{1.0}else{0.38}));if hit.clicked{if query.is_empty(){state.search_open.insert(key.clone(),false);}else{state.text.insert(key.clone(),String::new());state.search_focus_requested.insert(key.clone());action=Some(self.id);f.request_focus(id);}f.continuous(inner);}}
                    if submit_visible {let rect=Rect{x:place(true),y:inner.y+(inner.height-side)/2.0,width:side,height:side};let hit=f.target(anonymous_child_id(id,9802),rect,"Submit search",self.enabled,Role::Button);f.state_layer(hit,side/2.0,t.text);paint_android_search_icon(f,rect.inset((side-24.0).max(0.0)/2.0),2,t.text.opacity(if hit.enabled{1.0}else{0.38}));if hit.clicked&&state.submit_search(&key){state.search_pending.remove(&key);action=Some(self.id);}}
                    if response.focused&&f.pressed_key(Key::Enter)&&!state.autocomplete_has_highlight(&key)&&state.submit_search(&key){state.search_pending.remove(&key);action=Some(self.id);}
                    if self.enabled&&f.enabled&&state.autocomplete_items.contains_key(&key)&&(response.focused||state.autocomplete_open.as_deref()==Some(key.as_str())) {state.autocomplete_anchor=Some(AutoAnchor{key:key.clone(),id,xml_id:self.id,bounds:inner,threshold:state.search_suggestion_threshold.get(&key).copied().unwrap_or(1),width:"match_parent",height:"wrap_content",offset_x:0.0,offset_y:0.0,rtl:state.rtl,multi:false,search:true});}
                }
                }
            }
            Kind::CalendarView => {action=draw_xml_calendar(self,f,inner,state,id);}
            Kind::TimePicker if self.relative_rules.iter().any(|r|r.kind==105&&r.anchor=="clock") => {
                use crate::android_time_clock::{DialPart,TimeClockState};
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};let clock=state.wall_clock;let seconds=clock.map_or(0,|c|(c.unix_seconds.floor() as i64+i64::from(c.offset_seconds)).rem_euclid(86400));
                let tp_d = state.effective_density();
                let tp_sp = tp_d * state.effective_font_scale();
                let picker=state.time_clocks.entry(key.clone()).or_insert_with(||TimeClockState::new((seconds/3600) as u8,(seconds/60%60) as u8,clock.is_some_and(|c|c.is_24_hour)).unwrap());
                if state.time_clock_changed.remove(&key){action=Some(self.id);}
                let text_mode=state.time_clock_text_mode.contains(&key);let toggle=Rect{x:inner.x+8.0,y:inner.y+(inner.height-48.0).max(0.0),width:48.0_f32.min(inner.width),height:48.0_f32.min(inner.height)};let hit=f.target(anonymous_child_id(id,9705),toggle,if text_mode{"Show clock dial"}else{"Type time"},self.enabled,Role::Button);f.state_layer(hit,24.0,t.text);
                if hit.clicked {if text_mode{state.time_clock_text_mode.remove(&key);}else{state.time_clock_text_mode.insert(key.clone());let h=if picker.is_24_hour(){if matches!(state.calendar_locale.hour_cycle_24,crate::android_time_spinner::HourCycle::H24)&&picker.hour()==0{24}else{picker.hour()}}else if matches!(state.calendar_locale.hour_cycle_12,crate::android_time_spinner::HourCycle::H11){picker.hour()%12}else if picker.hour()%12==0{12}else{picker.hour()%12};let hk=format!("{key}:H");let mk=format!("{key}:M");state.text.insert(hk.clone(),state.calendar_locale.number(u32::from(h)));state.text.insert(mk.clone(),format!("{}{}",state.calendar_locale.digits[usize::from(picker.minute()/10)],state.calendar_locale.digits[usize::from(picker.minute()%10)]));state.input_types.insert(hk.clone(),3);state.input_types.insert(mk.clone(),3);state.time_clock_invalid.remove(&hk);state.time_clock_invalid.remove(&mk);state.time_clock_field_owner.insert(hk,(key.clone(),DialPart::Hour));state.time_clock_field_owner.insert(mk,(key.clone(),DialPart::Minute));}f.continuous(inner);}
                let icon=Rect{x:toggle.x+(toggle.width-24.0)/2.0,y:toggle.y+(toggle.height-24.0)/2.0,width:24.0,height:24.0};
                let index=crate::beercss_shapes::SHAPES.len()+crate::icons::material_navigation::ICONS.len()+crate::icons::material_symbols::ICONS.len()+2+crate::shapes::android_clock_shapes::SHAPES.len()+usize::from(text_mode);
                f.commands.push(DrawCommand::Shape{bounds:icon,index:index as u8,color:(if is_m3_theme { t.on_surface_variant } else { t.text }).opacity(if self.enabled&&f.enabled{1.0}else{0.38}),rotation:0.0});
                if text_mode {
                    if is_m3_theme {
                        let d = tp_d;
                        let box_r = 8.0 * d;
                        let title_rect = Rect { x: inner.x + 24.0 * d, y: inner.y + 16.0 * d, width: (inner.width - 48.0 * d).max(0.0), height: 24.0 * d };
                        f.styled_text(title_rect, "Enter time", t.on_surface_variant, TextStyle { size: 12.0 * tp_sp, weight: 500, ..TextStyle::default() });
                        let area = Rect { x: inner.x + 24.0 * d, y: inner.y + 56.0 * d, width: (inner.width - 48.0 * d).max(0.0), height: 80.0 * d };
                        let period_w = if picker.is_24_hour() { 0.0 } else { (52.0 * d).min(area.width * 0.25) };
                        let period_gap = if picker.is_24_hour() { 0.0 } else { 12.0 * d };
                        let main_w = (area.width - period_w - period_gap).max(0.0);
                        let sep_w = (24.0 * d).min(main_w * 0.16);
                        let part_w = ((main_w - sep_w) * 0.5).max(1.0);
                        for (i, part) in [DialPart::Hour, DialPart::Minute].into_iter().enumerate() {
                            let child = if part == DialPart::Hour { format!("{key}:H") } else { format!("{key}:M") };
                            let rect = Rect { x: area.x + i as f32 * (part_w + sep_w), y: area.y, width: part_w, height: 64.0 * d };
                            let invalid = state.time_clock_invalid.contains(&child);
                            let active = picker.part() == part;
                            f.paint_rect(rect, box_r, if active { t.primary_container } else { t.surface_highest });
                            if invalid || active {
                                f.commands.push(DrawCommand::Outline { bounds: rect, radius: box_r, width: (if invalid { 2.0 } else { 1.5 }) * d, color: if invalid { t.error } else { t.primary } });
                            }
                            f.target(view_id(&child), rect, if i == 0 { "Hour" } else { "Minute" }, self.enabled, Role::TextField);
                            let val_txt = state.text.get(&child).map(String::as_str).unwrap_or("");
                            if !val_txt.is_empty() {
                                f.styled_text(rect, val_txt, if active { t.on_primary_container } else { t.text }, TextStyle { size: 32.0 * tp_sp, align: 1, ..TextStyle::default() });
                            }
                            f.styled_text(Rect { x: rect.x, y: rect.y + rect.height + 4.0 * d, width: rect.width, height: 18.0 * d }, if invalid { "Invalid" } else if i == 0 { "Hour" } else { "Minute" }, if invalid { t.error } else { t.on_surface_variant }, TextStyle { size: 12.0 * tp_sp, align: 0, ..TextStyle::default() });
                        }
                        f.styled_text(Rect { x: area.x + part_w, y: area.y, width: sep_w, height: 64.0 * d }, &state.calendar_locale.time_separator, t.text, TextStyle { size: 32.0 * tp_sp, align: 1, ..TextStyle::default() });
                        if !picker.is_24_hour() {
                            let px = area.x + main_w + period_gap;
                            let per_rect = Rect { x: px, y: area.y, width: period_w, height: 64.0 * d };
                            let half_h = per_rect.height * 0.5;
                            for i in 0..2 {
                                let rect = Rect { x: per_rect.x, y: per_rect.y + i as f32 * half_h, width: per_rect.width, height: half_h };
                                let is_sel = usize::from(picker.hour() >= 12) == i;
                                let radii = if i == 0 { [box_r, box_r, 0.0, 0.0] } else { [0.0, 0.0, box_r, box_r] };
                                if is_sel {
                                    paint_rounded_rect_radii(f, rect, radii, t.tertiary_container);
                                }
                                let hit = f.target(anonymous_child_id(id, 9710 + i), rect, &state.calendar_locale.day_periods[i], self.enabled, Role::Button);
                                f.styled_text(rect, &state.calendar_locale.day_periods[i], if is_sel { t.on_tertiary_container } else { t.on_surface_variant }, TextStyle { size: 14.0 * tp_sp, weight: 500, align: 1, ..TextStyle::default() });
                                if hit.clicked && picker.set_period(i == 1) { action = Some(self.id); }
                            }
                            f.paint_rect(Rect { x: per_rect.x, y: per_rect.y + half_h - 0.5 * d, width: per_rect.width, height: 1.0 * d }, 0.0, t.outline);
                            f.commands.push(DrawCommand::Outline { bounds: per_rect, radius: box_r, width: 1.0 * d, color: t.outline });
                        }
                    } else {
                        let header=Rect{x:inner.x,y:inner.y,width:inner.width,height:80.0_f32.min(inner.height)};f.paint_rect(header,0.0,t.primary);f.styled_text(Rect{x:header.x+16.0,width:(header.width-32.0).max(0.0),..header},"Enter time",t.on_primary,TextStyle{size:20.0,..TextStyle::default()});
                        let area=Rect{x:inner.x+24.0,y:inner.y+112.0,width:(inner.width-48.0).max(0.0),height:64.0};let part_width=(area.width*0.32).max(1.0);let sep_width=(area.width*0.10).max(1.0);
                        for (i,part) in [DialPart::Hour,DialPart::Minute].into_iter().enumerate(){let child=if part==DialPart::Hour{format!("{key}:H")}else{format!("{key}:M")};let rect=Rect{x:area.x+i as f32*(part_width+sep_width),y:area.y,width:part_width,height:48.0};f.target(view_id(&child),rect,if i==0{"Hour"}else{"Minute"},self.enabled,Role::TextField);let invalid=state.time_clock_invalid.contains(&child);f.paint_line([rect.x,rect.y+rect.height-1.0],[rect.x+rect.width,rect.y+rect.height-1.0],1.0,if invalid{t.error}else{t.outline});f.styled_text(Rect{x:rect.x,y:rect.y+48.0,width:rect.width,height:18.0},if invalid{"Invalid"}else if i==0{"Hour"}else{"Minute"},if invalid{t.error}else{t.text},TextStyle{size:12.0,align:1,..TextStyle::default()});}
                        f.styled_text(Rect{x:area.x+part_width,y:area.y,width:sep_width,height:48.0},&state.calendar_locale.time_separator,t.text,TextStyle{size:22.0,align:1,..TextStyle::default()});
                        if !picker.is_24_hour(){let x=area.x+2.0*part_width+sep_width;let width=(area.width-2.0*part_width-sep_width).max(0.0);for i in 0..2 {let rect=Rect{x,y:area.y+i as f32*24.0,width,height:24.0};let hit=f.target(anonymous_child_id(id,9710+i),rect,&state.calendar_locale.day_periods[i],self.enabled,Role::Button);f.styled_text(rect,&state.calendar_locale.day_periods[i],if usize::from(picker.hour()>=12)==i{t.primary}else{t.text},TextStyle{size:14.0,align:1,..TextStyle::default()});if hit.clicked&&picker.set_period(i==1){action=Some(self.id);}}}
                    }
                }else{
                use crate::android_time_spinner::HourCycle;
                let cycle=if picker.is_24_hour(){state.calendar_locale.hour_cycle_24}else{state.calendar_locale.hour_cycle_12};let shown_hour=match cycle {HourCycle::H11=>picker.hour()%12,HourCycle::H12=>if picker.hour()%12==0{12}else{picker.hour()%12},HourCycle::H23=>picker.hour(),HourCycle::H24=>if picker.hour()==0{24}else{picker.hour()}};let formatted_hour=state.calendar_locale.number(u32::from(shown_hour));let padded=if picker.is_24_hour(){state.calendar_locale.hour_padded_24}else{state.calendar_locale.hour_padded_12};let hour=if padded&&shown_hour<10{format!("{}{}",state.calendar_locale.digits[0],formatted_hour)}else{formatted_hour};let minute=format!("{}{}",state.calendar_locale.digits[usize::from(picker.minute()/10)],state.calendar_locale.digits[usize::from(picker.minute()%10)]);
                let header_h = if is_m3_theme {
                    let d = tp_d;
                    let h_total = (116.0 * d).min(inner.height);
                    let pad_x = (24.0 * d).min(inner.width * 0.1);
                    let title_h = (20.0 * d).min(h_total * 0.25);
                    f.styled_text(
                        Rect { x: inner.x + pad_x, y: inner.y + 12.0 * d, width: (inner.width - pad_x * 2.0).max(0.0), height: title_h },
                        "Select time",
                        t.on_surface_variant,
                        TextStyle { size: 12.0 * tp_sp, weight: 500, align: if state.rtl { 2 } else { 0 }, ..TextStyle::default() },
                    );
                    let row_y = inner.y + 36.0 * d;
                    let row_h = (h_total - 40.0 * d).max(40.0).min(80.0 * d);
                    let row_w = (inner.width - pad_x * 2.0).max(0.0);
                    let period_w = if picker.is_24_hour() { 0.0 } else { (52.0 * d).min(row_w * 0.24) };
                    let period_gap = if picker.is_24_hour() { 0.0 } else { 12.0 * d };
                    let main_w = (row_w - period_w - period_gap).max(0.0);
                    let main_x = if !picker.is_24_hour() && state.calendar_locale.period_first {
                        inner.x + pad_x + period_w + period_gap
                    } else {
                        inner.x + pad_x
                    };
                    let sep_w = (24.0 * d).min(main_w * 0.16);
                    let box_w = ((main_w - sep_w) * 0.5).max(0.0);
                    let box_r = 8.0 * d;
                    let hour_rect = Rect { x: main_x, y: row_y, width: box_w, height: row_h };
                    let sep_rect = Rect { x: main_x + box_w, y: row_y, width: sep_w, height: row_h };
                    let minute_rect = Rect { x: main_x + box_w + sep_w, y: row_y, width: box_w, height: row_h };
                    let hour_active = picker.part() == DialPart::Hour;
                    let minute_active = picker.part() == DialPart::Minute;
                    f.paint_rect(hour_rect, box_r, if hour_active { t.primary_container } else { t.surface_highest });
                    f.paint_rect(minute_rect, box_r, if minute_active { t.primary_container } else { t.surface_highest });
                    let hour_hit = f.target(anonymous_child_id(id, 9700), hour_rect, "Select hour", self.enabled, Role::Button);
                    let minute_hit = f.target(anonymous_child_id(id, 9701), minute_rect, "Select minute", self.enabled, Role::Button);
                    f.state_layer(hour_hit, box_r, if hour_active { t.on_primary_container } else { t.text });
                    f.state_layer(minute_hit, box_r, if minute_active { t.on_primary_container } else { t.text });
                    if hour_hit.clicked { picker.set_part(DialPart::Hour); }
                    if minute_hit.clicked { picker.set_part(DialPart::Minute); }
                    f.styled_text(hour_rect, &hour, if hour_active { t.on_primary_container } else { t.text }, TextStyle { size: 42.0 * tp_sp, align: 1, ..TextStyle::default() });
                    f.styled_text(sep_rect, &state.calendar_locale.time_separator, t.text, TextStyle { size: 32.0 * tp_sp, align: 1, ..TextStyle::default() });
                    f.styled_text(minute_rect, &minute, if minute_active { t.on_primary_container } else { t.text }, TextStyle { size: 42.0 * tp_sp, align: 1, ..TextStyle::default() });
                    if !picker.is_24_hour() {
                        let px = if state.calendar_locale.period_first { inner.x + pad_x } else { inner.x + pad_x + main_w + period_gap };
                        let per_rect = Rect { x: px, y: row_y, width: period_w, height: row_h };
                        let half_h = per_rect.height * 0.5;
                        for i in 0..2 {
                            let rect = Rect { x: per_rect.x, y: per_rect.y + i as f32 * half_h, width: per_rect.width, height: half_h };
                            let is_sel = usize::from(picker.hour() >= 12) == i;
                            let radii = if i == 0 { [box_r, box_r, 0.0, 0.0] } else { [0.0, 0.0, box_r, box_r] };
                            if is_sel {
                                paint_rounded_rect_radii(f, rect, radii, t.tertiary_container);
                            }
                            let hit = f.target(anonymous_child_id(id, 9702 + i), rect, &state.calendar_locale.day_periods[i], self.enabled, Role::Button);
                            f.styled_text(rect, &state.calendar_locale.day_periods[i], if is_sel { t.on_tertiary_container } else { t.on_surface_variant }, TextStyle { size: 14.0 * tp_sp, weight: 500, align: 1, ..TextStyle::default() });
                            if hit.clicked && picker.set_period(i == 1) { action = Some(self.id); }
                        }
                        f.paint_rect(Rect { x: per_rect.x, y: per_rect.y + half_h - 0.5 * d, width: per_rect.width, height: 1.0 * d }, 0.0, t.outline);
                        f.commands.push(DrawCommand::Outline { bounds: per_rect, radius: box_r, width: 1.0 * d, color: t.outline });
                    }
                    h_total
                } else {
                    let header_h=100.0_f32.min(inner.height);let header=Rect{x:inner.x,y:inner.y,width:inner.width,height:header_h};f.paint_rect(header,0.0,t.primary);
                    let period_width=if picker.is_24_hour(){0.0}else{64.0_f32.min(header.width*0.25)};let main_width=(header.width-period_width).max(0.0);let main_x=if !picker.is_24_hour()&&state.calendar_locale.period_first {header.x+period_width}else{header.x};
                    let hour_rect=Rect{x:main_x,y:header.y,width:main_width*0.43,height:header.height};let separator=Rect{x:main_x+main_width*0.43,y:header.y,width:main_width*0.14,height:header.height};let minute_rect=Rect{x:main_x+main_width*0.57,y:header.y,width:main_width*0.43,height:header.height};
                    let hour_hit=f.target(anonymous_child_id(id,9700),hour_rect,"Select hour",self.enabled,Role::Button);let minute_hit=f.target(anonymous_child_id(id,9701),minute_rect,"Select minute",self.enabled,Role::Button);
                    if hour_hit.clicked{picker.set_part(DialPart::Hour);}if minute_hit.clicked{picker.set_part(DialPart::Minute);}
                    f.styled_text(hour_rect,&hour,t.on_primary.opacity(if picker.part()==DialPart::Hour{1.0}else{0.7}),TextStyle{size:42.0,align:1,..TextStyle::default()});f.styled_text(separator,&state.calendar_locale.time_separator,t.on_primary,TextStyle{size:32.0,align:1,..TextStyle::default()});f.styled_text(minute_rect,&minute,t.on_primary.opacity(if picker.part()==DialPart::Minute{1.0}else{0.7}),TextStyle{size:42.0,align:1,..TextStyle::default()});
                    if !picker.is_24_hour(){let x=if state.calendar_locale.period_first{header.x}else{header.x+header.width-period_width};for i in 0..2 {let rect=Rect{x,y:header.y+i as f32*header.height/2.0,width:period_width,height:header.height/2.0};let hit=f.target(anonymous_child_id(id,9702+i),rect,&state.calendar_locale.day_periods[i],self.enabled,Role::Button);f.styled_text(rect,&state.calendar_locale.day_periods[i],t.on_primary.opacity(if usize::from(picker.hour()>=12)==i{1.0}else{0.6}),TextStyle{size:14.0,align:1,..TextStyle::default()});if hit.clicked&&picker.set_period(i==1){action=Some(self.id);}}}
                    header_h
                };
                let available_h=(inner.height-header_h-16.0).max(0.0);let diameter=inner.width.min(available_h).min(296.0);let dial=Rect{x:inner.x+(inner.width-diameter)/2.0,y:inner.y+header_h+16.0,width:diameter,height:diameter};let center=[dial.x+diameter/2.0,dial.y+diameter/2.0];let radius=diameter/2.0;let outer=(radius-22.0).max(0.0);let inner_radius=(radius-58.0).max(0.0);let selector=20.0_f32.min(diameter/6.0);
                f.paint_rect(dial,radius,t.surface_highest);let _hit=f.target(anonymous_child_id(id,9704),dial,"Clock dial",self.enabled,Role::Button);
                if self.enabled&&f.enabled&&f.input.pressed&&f.pointer_in(dial){if let Some(p)=f.input.pointer{let distance=((p[0]-center[0]).powi(2)+(p[1]-center[1]).powi(2)).sqrt();let min_dist=if picker.is_24_hour()&&picker.part()==DialPart::Hour{(inner_radius-selector).max(0.0)}else{(outer-selector).max(0.0)};if distance>=min_dist{state.time_clock_dragging.insert(key.clone());}}}
                if state.time_clock_dragging.contains(&key) {if f.input.cancelled||f.input.blurred||!self.enabled||!f.enabled {state.time_clock_dragging.remove(&key);}else if let Some(p)=f.input.pointer {let dx=p[0]-center[0];let dy=p[1]-center[1];let distance=(dx*dx+dy*dy).sqrt();let min_dist=if picker.is_24_hour()&&picker.part()==DialPart::Hour{(inner_radius-selector).max(0.0)}else{(outer-selector).max(0.0)};if distance>=min_dist {let degrees=((dx.atan2(-dy).to_degrees().round() as i32).rem_euclid(360)) as u16;let on_inner=picker.is_24_hour()&&picker.part()==DialPart::Hour&&distance<=(outer+inner_radius)/2.0;if picker.select(degrees,on_inner,f.input.released){action=Some(self.id);}f.continuous(dial);}if f.input.released{state.time_clock_dragging.remove(&key);}}}
                let part=picker.part();let transition=state.time_clock_transition.entry(key.clone()).or_default();let (blend,moving)=transition.sample(f.time(),part==DialPart::Minute,f.reduced_motion());if moving{f.continuous(dial);}
                let hour_angle=f32::from(picker.hour()%12)*30.0;let minute_angle=f32::from(picker.minute())*6.0;let difference=(minute_angle-hour_angle+540.0).rem_euclid(360.0)-180.0;let angle=hour_angle+difference*blend;
                let hour_radius=if picker.is_24_hour()&&(picker.hour()==0||picker.hour()>12){inner_radius}else{outer};let selected_radius=hour_radius+(outer-hour_radius)*blend;let radians=angle.to_radians();let point=[center[0]+radians.sin()*selected_radius,center[1]-radians.cos()*selected_radius];f.paint_line(center,point,2.0,t.primary);f.paint_rect(Rect{x:point[0]-selector,y:point[1]-selector,width:selector*2.0,height:selector*2.0},selector,t.primary);let dot=if picker.minute()%5!=0{2.0*blend}else{0.0};if dot>0.01{f.paint_rect(Rect{x:point[0]-dot,y:point[1]-dot,width:dot*2.0,height:dot*2.0},dot,t.on_primary);}f.paint_rect(Rect{x:center[0]-3.0,y:center[1]-3.0,width:6.0,height:6.0},3.0,t.primary);
                for display in [DialPart::Hour,DialPart::Minute] {let alpha=if display==DialPart::Hour{1.0-blend}else{blend};if alpha<=0.001{continue;}f.with_opacity(alpha,|f|for ring in 0..if display==DialPart::Hour&&picker.is_24_hour(){2}else{1} {for i in 0..12 {let a=(i as f32*30.0).to_radians();let r=if ring==1{inner_radius}else{outer};let rect=Rect{x:center[0]+a.sin()*r-selector,y:center[1]-a.cos()*r-selector,width:selector*2.0,height:selector*2.0};let value=if display==DialPart::Minute{i*5}else if ring==1{if i==0{0}else{i+12}}else if i==0{12}else{i};let text=if (display==DialPart::Minute||ring==1)&&value<10{format!("{}{}",state.calendar_locale.digits[0],state.calendar_locale.number(value as u32))}else{state.calendar_locale.number(value as u32)};let active=if display==DialPart::Minute{usize::from(picker.minute())==value}else{usize::from(picker.hour())==if picker.is_24_hour(){value}else{value%12+if picker.hour()>=12{12}else{0}}};f.styled_text(rect,&text,if active{t.on_primary}else{t.text}.opacity(if self.enabled{1.0}else{0.38}),TextStyle{size:if ring==1{12.0}else{16.0},align:1,..TextStyle::default()});}});}

                }
            }
            Kind::TimePicker => {
                use crate::android_time_spinner::{TimeSpinnerField as Field,TimeSpinnerState};
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let clock=state.wall_clock;let seconds=clock.map_or(0,|c|(c.unix_seconds.floor() as i64+i64::from(c.offset_seconds)).rem_euclid(86400));
                let picker=state.time_spinners.entry(key.clone()).or_insert_with(||{let mut picker=TimeSpinnerState::new((seconds/3600) as u8,(seconds/60%60) as u8,clock.is_some_and(|c|c.is_24_hour)).unwrap();let _=picker.set_hour_cycles(state.calendar_locale.hour_cycle_12,state.calendar_locale.hour_cycle_24);picker});
                let mut fields=if picker.is_24_hour(){vec![Field::Hour,Field::Minute]}else if state.calendar_locale.period_first{vec![Field::Period,Field::Hour,Field::Minute]}else{vec![Field::Hour,Field::Minute,Field::Period]};
                if state.rtl {fields.reverse();}let hour_pos=fields.iter().position(|v|*v==Field::Hour).unwrap();let minute_pos=fields.iter().position(|v|*v==Field::Minute).unwrap();let separator_pos=hour_pos.min(minute_pos);let separator_width=12.0_f32.min(inner.width);let width=(inner.width-separator_width)/(fields.len() as f32);let height=inner.height.min(144.0);
                let separator=Rect{x:inner.x+(separator_pos+1) as f32*width,y:inner.y,width:separator_width,height};f.styled_text(separator,&state.calendar_locale.time_separator,t.text,TextStyle{size:20.0,align:1,..TextStyle::default()});
                for (position,field) in fields.into_iter().enumerate(){let suffix=match field {Field::Hour=>'H',Field::Minute=>'M',Field::Period=>'P'};let child_key=format!("{key}:{suffix}");let child_id=view_id(&child_key);
                    if state.number_picker_committed.remove(&child_key){if let Some(value)=state.number_pickers.get(&child_key).map(|p|p.value()){if state.time_spinners.get_mut(&key).unwrap().set_field(field,value).unwrap_or(false){action=Some(self.id);}}}
                    let range=state.time_spinners[&key].range(field);let labels=match field {Field::Period=>Some(state.calendar_locale.day_periods.to_vec()),Field::Minute=>Some((0..=59).map(|i|format!("{}{}",state.calendar_locale.digits[i/10],state.calendar_locale.digits[i%10])).collect()),Field::Hour=>Some((range.min..=range.max).map(|i|{let formatted=state.calendar_locale.number(i as u32);let padded=if state.time_spinners[&key].is_24_hour(){state.calendar_locale.hour_padded_24}else{state.calendar_locale.hour_padded_12};if padded&&i<10 {format!("{}{}",state.calendar_locale.digits[0],formatted)}else{formatted}}).collect())};
                    let wheel=state.number_pickers.entry(child_key.clone()).or_insert_with(||crate::android_number_picker::NumberPickerState::new(range.min,range.max,range.value).unwrap());wheel.set_range(range.min,range.max,labels).expect("valid time labels");wheel.set_wrap_preferred(range.wrap);wheel.set_value(range.value);
                    let rect=Rect{x:inner.x+position as f32*width+if position>separator_pos{separator_width}else{0.0},y:inner.y,width,height};let (_,changes)=draw_xml_number_picker(self,f,rect,state,child_id,child_key);
                    for value in changes {if state.time_spinners.get_mut(&key).unwrap().set_field(field,value).unwrap_or(false){action=Some(self.id);}}
                }
            }
            Kind::NumberPicker => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                action=draw_xml_number_picker(self,f,inner,state,id,key).0;
            }

            Kind::DatePicker if self.relative_rules.iter().any(|r|r.kind==102&&r.anchor=="spinner") => {
                use crate::android_calendar::CalendarDate;
                use crate::android_date_spinner::{DateSpinnerField as Field,DateSpinnerState};
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let parse=|v:&str|{let mut p=v.split('-');CalendarDate::new(p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?)};
                let rule=|kind|self.relative_rules.iter().find(|r|r.kind==kind).map(|r|r.anchor);
                let min=rule(99).and_then(parse).unwrap_or(CalendarDate{year:1900,month:1,day:1});let max=rule(100).and_then(parse).unwrap_or(CalendarDate{year:2100,month:12,day:31});
                let today=state.wall_clock.and_then(|c|parse(&format_wall_clock(c,"yyyy-MM-dd"))).unwrap_or(min);
                state.date_spinners.entry(key.clone()).or_insert_with(||DateSpinnerState::new(min,max,today).expect("validated DatePicker range"));
                let mut show_wheels=rule(103)!=Some("false");let show_calendar=rule(104)!=Some("false");if !show_wheels&&!show_calendar {show_wheels=true;}
                let calendar_gap=if show_wheels&&show_calendar {inner.width.min(44.0)}else{0.0};let wheel_area=if show_wheels&&show_calendar {(inner.width-calendar_gap).max(0.0)*0.5}else if show_wheels{inner.width}else{0.0};
                let wheel_x=if state.rtl && show_calendar {inner.x+inner.width-wheel_area}else{inner.x};
                let order=state.calendar_locale.date_order;let height=inner.height.min(144.0);let width=wheel_area/3.0;
                if show_wheels {for (position,code) in order.into_iter().enumerate(){let field=match code {'D'=>Field::Day,'M'=>Field::Month,_=>Field::Year};let visual=if state.rtl {2-position}else{position};let rect=Rect{x:wheel_x+visual as f32*width,y:inner.y+(inner.height-height)/2.0,width,height};let child_key=format!("{key}:{code}");let child_id=view_id(&child_key);
                    if state.number_picker_committed.remove(&child_key) {if let Some(value)=state.number_pickers.get(&child_key).map(|picker|picker.value()) {if state.date_spinners.get_mut(&key).unwrap().set_field(field,value).unwrap_or(false){action=Some(self.id);}}}
                    let range=state.date_spinners[&key].range(field);let labels=match field {Field::Month=>Some((range.min..=range.max).map(|i|state.calendar_locale.short_months[(i-1) as usize].clone()).collect()),Field::Day=>Some((range.min..=range.max).map(|i|format!("{}{}",state.calendar_locale.digits[(i/10) as usize],state.calendar_locale.digits[(i%10) as usize])).collect()),Field::Year=>Some((range.min..=range.max).map(|i|state.calendar_locale.number(i as u32)).collect())};
                    let picker=state.number_pickers.entry(child_key.clone()).or_insert_with(||crate::android_number_picker::NumberPickerState::new(range.min,range.max,range.value).expect("valid date range"));picker.set_range(range.min,range.max,labels).expect("valid labels");picker.set_wrap_preferred(range.wrap_preferred);picker.set_value(range.value);
                    let (_,changes)=draw_xml_number_picker(self,f,rect,state,child_id,child_key);
                    for value in changes {if state.date_spinners.get_mut(&key).unwrap().set_field(field,value).unwrap_or(false){action=Some(self.id);}}
                }}
                if show_calendar {
                    let date=state.date_spinners[&key].date();let first=rule(101).and_then(|v|v.parse().ok()).unwrap_or(state.calendar_locale.first_day_of_week);
                    let calendar=state.calendars.entry(key.clone()).or_insert_with(||crate::android_calendar::CalendarState::new(min,max,date,first).expect("valid date range"));
                    if calendar.selected()!=date {calendar.set_date(date);}
                    let cx=if state.rtl {inner.x}else{inner.x+wheel_area+calendar_gap};let cw=if show_wheels{(inner.width-wheel_area-calendar_gap).max(0.0)}else{inner.width};let rect=Rect{x:cx,y:inner.y,width:cw,height:inner.height};
                    if draw_xml_calendar(self,f,rect,state,id).is_some(){let selected=state.calendars[&key].selected();if state.date_spinners.get_mut(&key).unwrap().set_date(selected).unwrap_or(false){action=Some(self.id);f.continuous(inner);}}
                }
            }
            Kind::DatePicker => {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let header=Rect{x:inner.x,y:inner.y,width:inner.width,height:116.0_f32.min(inner.height)};
                let content=Rect{x:inner.x,y:header.y+header.height,width:inner.width,height:(inner.height-header.height).max(0.0)};
                let years_open=state.datepicker_year_open.contains(&key);let fade=state.datepicker_fades.entry(key.clone()).or_default();let (alphas,moving)=fade.sample(f.time(),years_open,f.reduced_motion());if moving{f.continuous(content);}
                if alphas[0]>0.0 || !state.calendars.contains_key(&key) {f.with_opacity(alphas[0],|f|f.without_input(years_open,|f|{action=draw_xml_calendar(self,f,content,state,id);}));}
                let selected=state.calendars[&key].selected();
                let year_rect=Rect{x:header.x+16.0,y:header.y+12.0,width:(header.width-32.0).max(0.0),height:32.0};let date_rect=Rect{x:header.x+16.0,y:header.y+44.0,width:(header.width-32.0).max(0.0),height:56.0};
                let (year_color, date_color, state_tint) = if is_m3_theme {
                    let div_h = state.dp(1.0).min(header.height);
                    f.paint_rect(Rect { x: header.x, y: header.y + header.height - div_h, width: header.width, height: div_h }, 0.0, t.outline_variant);
                    (
                        if years_open { t.primary } else { t.on_surface_variant },
                        if years_open { t.on_surface_variant } else { t.text },
                        t.text,
                    )
                } else {
                    f.paint_rect(header, 0.0, t.primary);
                    (
                        t.on_primary.opacity(if years_open { 1.0 } else { 0.7 }),
                        t.on_primary.opacity(if years_open { 0.7 } else { 1.0 }),
                        t.on_primary,
                    )
                };
                let year_hit=f.target(anonymous_child_id(id,9300),year_rect,"Choose year",self.enabled,Role::Button);f.state_layer(year_hit,0.0,state_tint);f.styled_text(year_rect,&state.calendar_locale.number(selected.year as u32),year_color,TextStyle{size:16.0,align:if state.rtl {2}else{0},..TextStyle::default()});
                let date_hit=f.target(anonymous_child_id(id,9301),date_rect,"Choose date",self.enabled,Role::Button);f.state_layer(date_hit,0.0,state_tint);let label=state.calendar_locale.date_label(selected);f.styled_text(date_rect,&label,date_color,TextStyle{size:30.0,align:if state.rtl {2}else{0},..TextStyle::default()});
                if year_hit.clicked && !years_open {state.datepicker_year_open.insert(key.clone());let (min,_)=state.calendars[&key].range();state.datepicker_year_scroll.insert(key.clone(),((selected.year-min.year) as f32*48.0-content.height/2.0+24.0).max(0.0));f.continuous(content);}
                if date_hit.clicked && years_open {state.datepicker_year_open.remove(&key);f.continuous(content);}
                if years_open || alphas[1]>0.0 {
                    let calendar=state.calendars.get_mut(&key).unwrap();let (min,max)=calendar.range();let count=(max.year-min.year+1) as usize;let max_offset=(count as f32*48.0-content.height).max(0.0);let offset=state.datepicker_year_scroll.entry(key.clone()).or_default();*offset=offset.clamp(0.0,max_offset);
                    let dragging=f.scroll_viewport(anonymous_child_id(id,9302),content,offset,max_offset);let start=(*offset/48.0).floor() as usize;let end=(((*offset+content.height)/48.0).ceil() as usize).min(count);let mut year=None;
                    f.with_opacity(alphas[1],|f|f.without_input(dragging||!years_open,|f|f.within(content,|f|for i in start..end {let value=min.year+i as i32;let rect=Rect{x:content.x,y:content.y+i as f32*48.0-*offset,width:content.width,height:48.0};let text=state.calendar_locale.number(value as u32);let hit=f.target(anonymous_child_id(id,9400+i),rect,&text,self.enabled,Role::Button);f.state_layer(hit,0.0,t.text);f.styled_text(rect,&text,if value==selected.year{t.primary}else{t.text},TextStyle{size:if value==selected.year{20.0}else{16.0},align:1,..TextStyle::default()});if hit.clicked {year=Some(value);}})));
                    if let Some(year)=year {let day=selected.day.min(crate::android_calendar::days_in_month(year,selected.month));let date=crate::android_calendar::CalendarDate{year,month:selected.month,day};calendar.set_date(date);state.datepicker_year_open.remove(&key);action=Some(self.id);f.continuous(content);}
                }
            }

            Kind::GridView => {
                if let Some(car_spec) = self.carousel_layout_spec() {
                    action = self.draw_carousel_layout(f, inner, state, id, car_spec, t);
                } else if let Some(rec_ext) = self.recycler_layout_ext() {
                    action = self.draw_recycler_ext(f, inner, state, id, rec_ext, t);
                } else {
                let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
                let template=state.list_templates.get(&key).copied();let mut row_state=if template.is_some() || state.list_item_bindings.contains_key(&key) {Some(spinner_template_state(state,state.rtl))}else{None};
                let d=state.effective_density();
                let entries=state.list_items.entry(key.clone()).or_default();
                let (columns,column_width,horizontal_spacing,outer_spacing)=grid_view_columns(self,inner.width,d);
                let vertical_spacing=self.relative_rules.iter().find(|r|r.kind==94).and_then(|r|r.anchor.parse::<f32>().ok()).unwrap_or(0.0).max(0.0)*d;
                let row_count=entries.len().div_ceil(columns);let metrics=f.ui.text_metrics;
                let signature=list_text_signature(entries,column_width,vertical_spacing,metrics,template,row_state.as_ref())^(columns as u64).wrapping_mul(0x9e3779b97f4a7c15);
                let geometry=state.list_text_geometry.entry(key.clone()).or_insert_with(||std::sync::Arc::new(ListTextGeometry::new(signature,row_count,vertical_spacing)));
                if geometry.signature!=signature {*geometry=std::sync::Arc::new(ListTextGeometry::new(signature,row_count,vertical_spacing));}let geometry=std::sync::Arc::make_mut(geometry);
                let checked=state.list_checked.entry(key.clone()).or_default();checked.retain(|i|*i<entries.len());
                let flags=state.list_enabled.get(&key);let item_ids=state.list_item_ids.get(&key);let bindings=state.list_item_bindings.get(&key);
                let mode=self.relative_rules.iter().find(|r|r.kind==88).map_or("none",|r|r.anchor);
                let offset=state.offsets.entry(key.clone()).or_default();if !offset.is_finite(){*offset=0.0;}
                let mut max=(geometry.prefix(row_count)-if row_count==0{0.0}else{vertical_spacing}-inner.height).max(0.0);
                let dragging=f.scroll_viewport(id,inner,offset,max);
                if geometry.measure_grid_window(entries,columns,*offset,inner.height,column_width,metrics,template,&mut row_state,id,bindings) {max=(geometry.prefix(row_count)-if row_count==0{0.0}else{vertical_spacing}-inner.height).max(0.0);f.continuous(inner);}
                let scroll=*offset;let start=geometry.index_at(scroll.max(0.0)).min(row_count);let end=geometry.index_at((scroll+inner.height).max(0.0)).saturating_add(1).min(row_count);
                let long_enabled=state.list_long_click_enabled.get(&key).copied().unwrap_or(false);
                let hold=state.list_holds.entry(key.clone()).or_default();
                if !long_enabled || dragging || f.input.cancelled || f.input.blurred || f.input.released || f.input.scroll!=[0.0,0.0] {*hold=None;}
                let mut chosen=None;let mut long_choice=None;let mut control_choice=None;
                let row_state_environment=row_state.as_ref().cloned();let rtl=state.rtl;let control_states=&mut state.list_control_states;
                f.without_input(dragging,|f|f.within(inner,|f|for row in start..end {
                    let y=inner.y+geometry.prefix(row)-scroll;
                    for column in 0..columns {let index=row*columns+column;if index>=entries.len(){break;}
                        let column_x=if state.rtl {inner.x+inner.width-outer_spacing-column_width-column as f32*(column_width+horizontal_spacing)}else{inner.x+outer_spacing+column as f32*(column_width+horizontal_spacing)};
                        let (item_width,item_height)=grid_item_size(template,&mut row_state,&entries[index],column_width,metrics,anonymous_child_id(id,1024+index),bindings.and_then(|items|items.get(&index)));
                        let right=match self.content_gravity[0] {0=>state.rtl,2=>!state.rtl,4=>true,_=>false};
                        let x=if self.content_gravity[0]==1 {column_x+(column_width-item_width)*0.5}else if right {column_x+column_width-item_width}else{column_x};
                        let bounds=Rect{x,y,width:item_width,height:item_height};
                        if checked.contains(&index){f.paint_rect(bounds,0.0,t.primary_container);}
                        let item_key=item_ids.and_then(|ids|ids.get(index)).map_or(index as u64,|value|*value as u64);
                        let enabled=self.enabled && flags.and_then(|v|v.get(index)).copied().unwrap_or(true);
                        let row_id=id.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(item_key);let binding=bindings.and_then(|items|items.get(&index));
                        let item_template=adapter_item_template(template,binding);let interactive=item_template.is_some_and(|template|inspect_expandable_template(template.row,template.text_id).2);
                        if interactive {let template=item_template.unwrap();let controls=control_states.entry((key.clone(),item_key as i64)).or_default();let mut local=row_state_environment.clone().unwrap_or_default();local.adapter_namespace=Some(row_id);local.rtl=rtl;local.checked=std::mem::take(controls);with_adapter_binding(&mut local,template.text_id,&entries[index],binding,|row_state|f.within(bounds,|f|f.with_enabled(enabled,|f|{if let Some(view)=template.row.draw(f,bounds,row_state,anonymous_child_id(row_id,0)){control_choice=Some((index,view));}})));*controls=local.checked;}
                        let r=f.target(row_id,bounds,&entries[index],enabled,Role::Button);
                        if adapter_item_long_press(f,r,hold,long_enabled) {long_choice=Some(index);}f.state_layer(r,0.0,t.text);
                        if interactive {} else if let (Some(template),Some(row_state))=(item_template,row_state.as_mut()) {paint_spinner_template(f,template.row,template.text_id,&entries[index],bounds,row_state,anonymous_child_id(id,1024+index),r,checked.contains(&index),binding);}
                        else {let style=TextStyle {align:if state.rtl{2}else{0},..TextStyle::default()};let width=(item_width-32.0).max(1.0);let lines=wrapped_text_lines(&entries[index],width,style,metrics,|_|{}).max(1);let mut line_y=y+(item_height-lines as f32*24.0).max(0.0)*0.5;wrapped_text_lines(&entries[index],width,style,metrics,|line|{f.styled_text(Rect{x:x+16.0,y:line_y,width,height:24.0},&line.text,t.text.opacity(if r.enabled{1.0}else{0.38}),style);line_y+=24.0;});}
                        if r.clicked {chosen=Some(index);}
                    }
                }));
                if let Some(control)=control_choice {state.list_control_clicked.insert(key.clone(),control);chosen=None;long_choice=None;action=Some(self.id);}
                if let Some(index)=long_choice {state.list_long_clicked.insert(key.clone(),index);action=Some(self.id);}
                if let Some(index)=chosen {match mode {"singleChoice"=>{checked.clear();checked.push(index);},"multipleChoice"=>{if let Some(pos)=checked.iter().position(|v|*v==index){checked.remove(pos);}else{checked.push(index);}},_=>{}}state.list_clicked.insert(key,index);action=Some(self.id);}
                let _=max;
                }
            }
            Kind::ListView => {
                if let Some(car_spec) = self.carousel_layout_spec() {
                    action = self.draw_carousel_layout(f, inner, state, id, car_spec, t);
                } else if let Some(rec_ext) = self.recycler_layout_ext() {
                    action = self.draw_recycler_ext(f, inner, state, id, rec_ext, t);
                } else {
                let key=if self.id.is_empty(){format!("#{id}")} else {self.id.to_owned()};
                let template=state.list_templates.get(&key).copied();
                let mut row_state=if template.is_some() || state.list_item_bindings.contains_key(&key) {Some(spinner_template_state(state,state.rtl))}else{None};
                let (headers,footers)=state.list_fixed_views.get(&key).cloned().unwrap_or_default();
                let mut fixed_state=spinner_template_state(state,state.rtl);fixed_state.text=state.text.clone();fixed_state.visibility=state.visibility.clone();
                let header_heights:Vec<f32>=headers.iter().enumerate().map(|(i,v)|list_fixed_height(v.view,&fixed_state,inner.width,f.ui.text_metrics,anonymous_child_id(id,2048+i))).collect();
                let footer_heights:Vec<f32>=footers.iter().enumerate().map(|(i,v)|list_fixed_height(v.view,&fixed_state,inner.width,f.ui.text_metrics,anonymous_child_id(id,3072+i))).collect();
                let divider=state.dp(self.divider_thickness.max(0.0));let metrics=f.ui.text_metrics;
                let entries=state.list_items.entry(key.clone()).or_insert_with(||self.relative_rules.iter().filter(|r|r.kind==82).map(|r|r.anchor.to_owned()).collect());
                let signature=list_text_signature(entries,inner.width,divider,metrics,template,row_state.as_ref());
                let geometry=state.list_text_geometry.entry(key.clone()).or_insert_with(||std::sync::Arc::new(ListTextGeometry::new(signature,entries.len(),divider)));
                if geometry.signature!=signature {*geometry=std::sync::Arc::new(ListTextGeometry::new(signature,entries.len(),divider));}
                let geometry=std::sync::Arc::make_mut(geometry);
                let mode=self.relative_rules.iter().find(|r|r.kind==88).map_or("none",|r|r.anchor);
                let checked=state.list_checked.entry(key.clone()).or_default();checked.retain(|i|*i<entries.len());
                let flags=state.list_enabled.get(&key);let item_ids=state.list_item_ids.get(&key);let bindings=state.list_item_bindings.get(&key);
                let enabled=|i:usize|self.enabled && flags.and_then(|f|f.get(i)).copied().unwrap_or(true);
                let header_dividers=!self.relative_rules.iter().any(|r|r.kind==89 && r.anchor=="false");
                let footer_dividers=!self.relative_rules.iter().any(|r|r.kind==90 && r.anchor=="false");
                let header_gaps=if header_dividers {headers.len().saturating_sub(usize::from(entries.is_empty() && footers.is_empty()))} else {0};
                let item_origin=header_heights.iter().sum::<f32>()+header_gaps as f32*divider;
                let fixed_footer_height=footer_heights.iter().sum::<f32>()+if footer_dividers {footers.len().saturating_sub(1) as f32*divider}else{0.0};
                let footer_gap=if !entries.is_empty() && !footers.is_empty() && footer_dividers {divider}else{0.0};
                let last_divider=if entries.is_empty(){0.0}else{divider};
                let mut item_extent=(geometry.prefix(entries.len())-last_divider).max(0.0);
                let mut footer_origin=item_origin+item_extent+footer_gap;
                let mut max=(footer_origin+fixed_footer_height-inner.height).max(0.0);
                let offset=state.offsets.entry(key.clone()).or_default();if !offset.is_finite(){*offset=0.0;}
                let dragging=f.scroll_viewport(id,inner,offset,max);
                let pending=state.list_pending_focus.remove(&key);
                let mut changed=geometry.measure_window(entries,*offset-item_origin,inner.height,inner.width,metrics,template,&mut row_state,id,bindings);
                if let Some(index)=pending.filter(|index|*index<entries.len()) {changed|=geometry.measure(index,&entries[index],inner.width,metrics,template,&mut row_state,id,bindings.and_then(|items|items.get(&index)));}
                if changed {
                    item_extent=(geometry.prefix(entries.len())-last_divider).max(0.0);footer_origin=item_origin+item_extent+footer_gap;max=(footer_origin+fixed_footer_height-inner.height).max(0.0);
                    if let Some(index)=pending.filter(|index|*index<entries.len()) {
                        let top=item_origin+geometry.prefix(index);let bottom=top+geometry.heights[index];
                        if top<*offset {*offset=top;} else if bottom>*offset+inner.height {*offset=(bottom-inner.height).min(max).max(top-(inner.height-geometry.heights[index]).max(0.0));}
                    }
                    f.continuous(inner);
                }
                let scroll=*offset;let local=scroll-item_origin;
                let (start,end)=if local+inner.height<=0.0 {(0,0)} else {(geometry.index_at(local.max(0.0)),geometry.index_at((local+inner.height).max(0.0)).saturating_add(1))};
                let start=start.min(entries.len());let end=end.min(entries.len());
                let item_id=|i:usize|id.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(item_ids.and_then(|ids|ids.get(i)).map_or(i as u64,|value|*value as u64));
                let long_enabled=state.list_long_click_enabled.get(&key).copied().unwrap_or(false);
                let hold=state.list_holds.entry(key.clone()).or_default();
                if !long_enabled || dragging || f.input.cancelled || f.input.blurred || f.input.released || f.input.scroll!=[0.0,0.0] {*hold=None;}
                let mut chosen=None;let mut fixed_choice=None;let mut fixed_control_choice=None;let mut long_choice=None;let mut navigation=None;let mut control_choice=None;
                let row_state_environment=row_state.as_ref().cloned();let rtl=state.rtl;let control_states=&mut state.list_control_states;let fixed_control_states=&mut state.list_fixed_control_states;
                f.without_input(dragging,|f|f.within(inner,|f| {
                    let mut y=0.0;
                    for (i,fixed) in headers.iter().enumerate() {
                        let bounds=Rect{x:inner.x,y:inner.y+y-scroll,width:inner.width,height:header_heights[i]};
                        if bounds.intersect(inner).height>0.0 {let controls=fixed_control_states.entry((key.clone(),false,i)).or_default();let (clicked,control)=draw_list_fixed(f,*fixed,bounds,&mut fixed_state,anonymous_child_id(id,2048+i),self.enabled,controls);if clicked{fixed_choice=Some((false,i));}if let Some(view)=control{fixed_control_choice=Some((false,i,view));}}
                        y+=header_heights[i];if header_dividers && (i+1<headers.len() || !entries.is_empty() || !footers.is_empty()) {if let Some(color)=self.divider_color {f.paint_rect(Rect{x:inner.x,y:inner.y+y-scroll,width:inner.width,height:divider},0.0,color);}y+=divider;}
                    }
                    for i in start..end {
                        let top=geometry.prefix(i);
                        let height=geometry.heights[i];
                        let row=Rect{x:inner.x,y:inner.y+item_origin+top-scroll,width:inner.width,height};
                        if checked.contains(&i) {f.paint_rect(row,0.0,t.primary_container);}
                        if pending==Some(i) && enabled(i) {f.request_focus(item_id(i));}
                        let row_id=item_id(i);
                        let binding=bindings.and_then(|items|items.get(&i));
                        let item_template=adapter_item_template(template,binding);
                        let interactive=item_template.is_some_and(|template|inspect_expandable_template(template.row,template.text_id).2);
                        if interactive {
                            let template=item_template.unwrap();let identity=item_ids.and_then(|ids|ids.get(i)).copied().unwrap_or(i as i64);
                            let controls=control_states.entry((key.clone(),identity)).or_default();let mut local=row_state_environment.clone().unwrap_or_default();
                            local.adapter_namespace=Some(row_id);local.rtl=rtl;local.checked=std::mem::take(controls);
                            with_adapter_binding(&mut local,template.text_id,&entries[i],binding,|row_state|f.within(row,|f|f.with_enabled(enabled(i),|f|{if let Some(view)=template.row.draw(f,row,row_state,anonymous_child_id(row_id,0)){control_choice=Some((i,view));}})));
                            *controls=local.checked;
                        }
                        let r=f.target(row_id,row,&entries[i],enabled(i),Role::Button);
                        if adapter_item_long_press(f,r,hold,long_enabled) {long_choice=Some(i);}
                        if f.input.key.is_some_and(|key|matches!(key.key,Key::Up|Key::Down|Key::Home|Key::End|Key::PageUp|Key::PageDown)) && let Some(key)=f.key_for(r) {navigation=Some((i,key));}
                        f.state_layer(r,0.0,t.text);
                        if interactive {} else if let (Some(template),Some(row_state))=(item_template,row_state.as_mut()) {
                            paint_spinner_template(f,template.row,template.text_id,&entries[i],row,row_state,anonymous_child_id(id,1024+i),r,checked.contains(&i),binding);
                        } else {
                            let style=TextStyle {align:if state.rtl {2}else{0},..TextStyle::default()};
                            let width=(row.width-32.0).max(1.0);let lines=wrapped_text_lines(&entries[i],width,style,metrics,|_|{}).max(1);
                            let mut line_y=row.y+(row.height-lines as f32*24.0).max(0.0)*0.5;
                            wrapped_text_lines(&entries[i],width,style,metrics,|line|{f.styled_text(Rect{x:row.x+16.0,y:line_y,width,height:24.0},&line.text,t.text.opacity(if r.enabled {1.0}else{0.38}),style);line_y+=24.0;});
                        }
                        if r.clicked {chosen=Some(i);}
                        if i+1<entries.len() && divider>0.0 && let Some(color)=self.divider_color {
                            f.paint_rect(Rect{x:row.x,y:row.y+row.height,width:row.width,height:divider},0.0,color);
                        }
                    }
                    if !entries.is_empty() && !footers.is_empty() && footer_dividers && let Some(color)=self.divider_color {f.paint_rect(Rect{x:inner.x,y:inner.y+footer_origin-divider-scroll,width:inner.width,height:divider},0.0,color);}
                    let mut y=footer_origin;
                    for (i,fixed) in footers.iter().enumerate() {
                        let bounds=Rect{x:inner.x,y:inner.y+y-scroll,width:inner.width,height:footer_heights[i]};
                        if bounds.intersect(inner).height>0.0 {let controls=fixed_control_states.entry((key.clone(),true,i)).or_default();let (clicked,control)=draw_list_fixed(f,*fixed,bounds,&mut fixed_state,anonymous_child_id(id,3072+i),self.enabled,controls);if clicked{fixed_choice=Some((true,i));}if let Some(view)=control{fixed_control_choice=Some((true,i,view));}}
                        y+=footer_heights[i];if footer_dividers && i+1<footers.len() {if let Some(color)=self.divider_color {f.paint_rect(Rect{x:inner.x,y:inner.y+y-scroll,width:inner.width,height:divider},0.0,color);}y+=divider;}
                    }
                }));
                if let Some(control)=control_choice {state.list_control_clicked.insert(key.clone(),control);chosen=None;long_choice=None;action=Some(self.id);}
                if let Some(control)=fixed_control_choice {state.list_fixed_control_clicked.insert(key.clone(),control);action=Some(self.id);}
                if let Some(fixed)=fixed_choice {state.list_fixed_clicked.insert(key.clone(),fixed);action=Some(self.id);}
                if let Some((current,key_press))=navigation {
                    let at=|i:usize|item_origin+geometry.prefix(i);
                    let find_forward=|start:usize| (start..entries.len()).find(|&i|enabled(i));
                    let find_backward=|start:usize| (0..=start.min(entries.len().saturating_sub(1))).rev().find(|&i|enabled(i));
                    let next=match key_press {
                        Key::Up=>current.checked_sub(1).and_then(find_backward),
                        Key::Down=>find_forward(current.saturating_add(1)),
                        Key::Home=>find_forward(0),
                        Key::End=>find_backward(entries.len().saturating_sub(1)),
                        Key::PageUp=>{
                            let target=(at(current)-inner.height).max(0.0);
                            let index=geometry.index_at((target-item_origin).max(0.0));
                            find_backward(index).filter(|i|*i<current).or_else(||current.checked_sub(1).and_then(find_backward))
                        },
                        Key::PageDown=>{
                            let target=at(current)+inner.height;
                            let index=geometry.index_at((target-item_origin).max(0.0));
                            find_forward(index.max(current+1)).or_else(||find_backward(entries.len().saturating_sub(1)).filter(|i|*i>current))
                        },
                        _=>None,
                    };
                    if let Some(next)=next {
                        let top=at(next);let bottom=top+geometry.heights[next];
                        if top<*offset {*offset=top;}else if bottom>*offset+inner.height {*offset=(bottom-inner.height).min(max);}
                        state.list_pending_focus.insert(key.clone(),next);f.continuous(inner);
                    }
                }
                if let Some(i)=long_choice {state.list_long_clicked.insert(key.clone(),i);action=Some(self.id);}
                if let Some(i)=chosen {
                    match mode {"singleChoice"=>{checked.clear();checked.push(i);},"multipleChoice"=>{if let Some(pos)=checked.iter().position(|v|*v==i){checked.remove(pos);}else{checked.push(i);}},_=>{}}
                    state.list_clicked.insert(key,i);action=Some(self.id);
                }
                }
            }
            Kind::Spinner => {
                let key=if self.id.is_empty(){format!("#{id}")} else {self.id.to_string()};
                let has_explicit_bg = explicit_background.is_some()
                    || background_tint.is_some()
                    || self.background_stroke.is_some()
                    || gradient.is_some()
                    || has_layers
                    || has_stateful_layers;
                let theme_style = state.theme_style;
                let d = state.effective_density();
                let rtl = state.rtl;
                let scaled_text = self.scaled_text_size(state);
                let is_open = state.spinner_open.as_deref() == Some(key.as_str());
                let paint_spinner_chrome = |f: &mut Frame<'_>, focused_or_open: bool, is_enabled: bool| {
                    match theme_style {
                        AndroidTheme::Bhoo => {
                            f.paint_rect(b, 4.0, t.surface_highest);
                        }
                        AndroidTheme::MaterialLight | AndroidTheme::MaterialDark => {
                            if !has_explicit_bg {
                                let thickness = (if focused_or_open { 2.0 } else { 1.0 }) * d;
                                let unfocused_color = match theme_style {
                                    AndroidTheme::MaterialLight => Color(0x00, 0x00, 0x00, 0x6B),
                                    _ => Color(0xFF, 0xFF, 0xFF, 0x8A),
                                };
                                let line_color = if !is_enabled {
                                    unfocused_color.opacity(0.38)
                                } else if focused_or_open {
                                    t.primary
                                } else {
                                    unfocused_color
                                };
                                let inset_x = (4.0 * d).min(b.width * 0.25);
                                f.paint_rect(
                                    Rect {
                                        x: b.x + inset_x,
                                        y: b.y + b.height - thickness,
                                        width: (b.width - inset_x * 2.0).max(0.0),
                                        height: thickness,
                                    },
                                    0.0,
                                    line_color,
                                );
                            }
                        }
                        AndroidTheme::Material3Light | AndroidTheme::Material3Dark => {
                            if !has_explicit_bg {
                                let r4 = 4.0 * d;
                                let stroke_w = (if focused_or_open { 2.0 } else { 1.0 }) * d;
                                let stroke_color = if !is_enabled {
                                    t.text.opacity(0.12)
                                } else if focused_or_open {
                                    t.primary
                                } else {
                                    t.outline
                                };
                                f.commands.push(DrawCommand::Outline {
                                    bounds: b,
                                    radius: r4,
                                    width: stroke_w,
                                    color: stroke_color,
                                });
                            }
                        }
                    }
                };
                let paint_spinner_arrow = |f: &mut Frame<'_>, is_enabled: bool, open: bool| {
                    if theme_style == AndroidTheme::Bhoo {
                        let x=if rtl {b.x+16.0} else {b.x+b.width-16.0};let y=b.y+b.height*0.5;
                        f.commands.push(DrawCommand::Polyline {points:[[x-5.0,y-2.0],[x,y+3.0],[x+5.0,y-2.0]],count:3,width:2.0,color:t.on_surface_variant});
                    } else {
                        let pad = 16.0 * d;
                        let x = if rtl { b.x + pad } else { b.x + b.width - pad };
                        let y = b.y + b.height * 0.5;
                        let dx = 5.0 * d;
                        let dy_up = 2.0 * d;
                        let dy_dn = 3.0 * d;
                        let color = if !is_enabled {
                            t.on_surface_variant.opacity(0.38)
                        } else if open && matches!(theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark) {
                            t.primary
                        } else {
                            t.on_surface_variant
                        };
                        f.commands.push(DrawCommand::Polyline {
                            points: [[x - dx, y - dy_up], [x, y + dy_dn], [x + dx, y - dy_up]],
                            count: 3,
                            width: 2.0 * d,
                            color,
                        });
                    }
                };
                let entries=state.spinner_items.entry(key.clone()).or_insert_with(||self.relative_rules.iter().filter(|r|r.kind==82).map(|r|r.anchor.to_string()).collect());
                if !entries.is_empty() {
                    let selected=state.spinner_selected.get(&key).copied().unwrap_or(0).min(entries.len()-1);
                    if state.spinner_selected.get(&key).is_some_and(|&i|i>=entries.len()) {state.spinner_selected.insert(key.clone(),selected);}
                    let label=entries[selected].clone();
                    let r=f.target(id,b,&label,self.enabled,Role::Button);
                    if r.enabled && state.spinner_open.as_ref().or(state.spinner_closing.as_ref()).is_none_or(|open|open==&key) {
                        let dialog=self.relative_rules.iter().any(|r|r.kind==83);
                        let prompt=self.relative_rules.iter().find(|r|r.kind==84).map_or("",|r|r.anchor);
                        let rule=|kind|self.relative_rules.iter().find(|r|r.kind==kind).map(|r|r.anchor);
                        state.spinner_anchor=Some(SpinnerAnchor {key:key.clone(),id,bounds:b,xml_id:self.id,dialog,prompt,rtl,width:rule(85).unwrap_or("wrap_content"),offset_x:rule(86).and_then(|s|s.parse().ok()).unwrap_or(0.0),offset_y:rule(87).and_then(|s|s.parse().ok()).unwrap_or(0.0),padding:self.resolved_view_padding(state)});
                    }
                    let focused_or_open = r.enabled && (r.focused || is_open);
                    paint_spinner_chrome(f, focused_or_open, r.enabled);
                    f.state_layer(r, 4.0 * d, t.text);
                    let arrow_reserve = if theme_style == AndroidTheme::Bhoo { 40.0 } else { 40.0 * d };
                    let label_bounds=Rect {x:b.x+if rtl {arrow_reserve} else {0.0},width:(b.width-arrow_reserve).max(0.0),..b};
                    if let Some(templates)=state.spinner_templates.get(&key).copied() {
                        let mut row_state=spinner_template_state(state,rtl);
                        paint_spinner_template(f,templates.selected,templates.text_id,&label,label_bounds,&mut row_state,anonymous_child_id(id,1023),r,false,None);
                    } else if theme_style == AndroidTheme::Bhoo {
                        f.styled_text(label_bounds,&label,t.text.opacity(if r.enabled {1.0} else {0.38}),TextStyle {padding:12.0,..TextStyle::default()});
                    } else {
                        let pad = (if matches!(theme_style, AndroidTheme::Material3Light | AndroidTheme::Material3Dark) { 16.0 } else { 12.0 }) * d;
                        f.styled_text(
                            label_bounds,
                            &label,
                            t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
                            TextStyle {
                                size: scaled_text,
                                align: if rtl { 2 } else { 0 },
                                padding: pad,
                                ..TextStyle::default()
                            },
                        );
                    }
                    paint_spinner_arrow(f, r.enabled, is_open);
                    if r.clicked {state.spinner_highlight.insert(key.clone(),selected);state.spinner_scroll.remove(&key);state.spinner_open=Some(key);}
                } else {
                    state.spinner_selected.remove(&key);state.spinner_highlight.remove(&key);state.spinner_scroll.remove(&key);
                    paint_spinner_chrome(f, false, enabled);
                    if theme_style != AndroidTheme::Bhoo {
                        paint_spinner_arrow(f, enabled, false);
                    }
                }
            }
            _ => {}
        }
        *action_out = action;
    }
}
