use super::*;

impl Node {
    // Offset of the first rendered text baseline from the view's top edge.
    pub(super) fn relative_baseline(&self, width:f32, height:f32, state:&LayoutState, metrics:TextMetrics, id:u64) -> Option<f32> {
        let padding=self.resolved_padding(state);
        let inner_height=(height-padding[1]-padding[3]).max(0.0);
        if matches!(self.kind,Kind::Image|Kind::QuickContactBadge|Kind::Icon|Kind::Zoom) {
            if self.relative_rules.iter().any(|r|r.kind==190) {return Some(height);}
            return self.relative_rules.iter().find(|r|r.kind==189).and_then(|r|r.anchor.parse::<f32>().ok()).filter(|v|v.is_finite() && *v>=0.0).map(|v| state.dp(v));
        }
        if matches!(self.kind,Kind::Linear|Kind::RadioGroup|Kind::TableRow) {
            // LinearLayout.getBaseline() delegates to the explicitly selected child.
            // Android rejects a weighted child before that index in a vertical layout.
            let index=self.relative_rules.iter().find(|r|r.kind==148)?.anchor.parse::<usize>().ok()?;
            let child=self.children.get(index)?;
            let horizontal=self.horizontal || matches!(self.kind,Kind::TableRow);
            if child.effective_visibility(state)==Visibility::Gone || (!horizontal && self.children[..index].iter().any(|n|n.effective_visibility(state)!=Visibility::Gone && n.weight>0.0)) {return None;}
            let inner_width=(width-padding[0]-padding[2]).max(0.0);
            let size=if horizontal {inner_width}else{inner_height};
            let mut lengths=Vec::with_capacity(self.children.len());
            let divider_thickness=state.dp(self.divider_thickness);
            let mut fixed=self.divider_count(state) as f32*divider_thickness;
            let mut weight=0.0;
            for (i,n) in self.children.iter().enumerate() {
                if n.effective_visibility(state)==Visibility::Gone {lengths.push(0.0);continue;}
                let m=n.resolved_margin(state);let available=(inner_width-m[0]-m[2]).max(0.0);
                let value=n.layout_intrinsic(horizontal,state,available,metrics,child_view_id(id,i,n));
                fixed+=value+n.margin_for_axis(horizontal,state);weight+=n.weight;lengths.push(value);
            }
            let mut excess=size-fixed;let mut remaining=if self.weight_sum>0.0 {self.weight_sum}else{weight};
            for (i,n) in self.children.iter().enumerate() {
                if n.effective_visibility(state)==Visibility::Gone {continue;}
                let share=if n.weight>0.0 && remaining.abs()>f32::EPSILON {excess*n.weight/remaining}else{0.0};
                if n.weight>0.0 {excess-=share;remaining-=n.weight;}
                lengths[i]=(lengths[i]+share).max(0.0);
            }
            let margin=child.resolved_margin(state);
            let available_width=(inner_width-margin[0]-margin[2]).max(0.0);
            let child_width=if horizontal {lengths[index]} else {match child.width {
                Size::Dp(v)=>state.dp(v).min(available_width), Size::Wrap=>child.intrinsic(true,state,child_view_id(id,index,child)).min(available_width), Size::Match=>available_width,
            }};
            let available_height=(inner_height-margin[1]-margin[3]).max(0.0);
            let child_height=if !horizontal {lengths[index]} else {match child.height {
                Size::Dp(v)=>state.dp(v).min(available_height), Size::Wrap=>child.layout_intrinsic(false,state,child_width,metrics,child_view_id(id,index,child)).min(available_height), Size::Match=>available_height,
            }};
            let baseline=child.relative_baseline(child_width,child_height,state,metrics,child_view_id(id,index,child))?;
            let mut child_top=0.0;
            if !horizontal {
                let mut visible=0usize;
                for (i,n) in self.children.iter().enumerate().take(index) {
                    if n.effective_visibility(state)==Visibility::Gone {continue;}
                    if self.divider_color.is_some() && (if visible==0 {self.divider_flags&1!=0}else{self.divider_flags&2!=0}) {child_top+=divider_thickness;}
                    visible+=1;child_top+=lengths[i]+n.margin_for_axis(false,state);
                }
                let total=lengths.iter().enumerate().filter(|(i,_)|self.children[*i].effective_visibility(state)!=Visibility::Gone)
                    .map(|(i,length)|*length+self.children[i].margin_for_axis(false,state)).sum::<f32>()
                    +self.divider_count(state) as f32*divider_thickness+padding[1]+padding[3];
                match self.content_gravity[1] {
                    1=>child_top+=((height-padding[1]-padding[3])-total)/2.0,
                    2=>child_top=height-padding[3]-total,
                    _=>{},
                }
            }
            return Some(child_top+margin[1]+baseline);
        }
        let (effective_size, effective_weight, italic, family) = self.effective_text_font(state);
        let offset=if effective_weight >= 500 {metrics.medium_baseline_offset} else {metrics.normal_baseline_offset};
        if matches!(self.kind,Kind::Button) {
            let rtl=match self.layout_direction {1=>false,2=>true,_=>state.rtl};let compound=self.compound_extents(state,rtl);
            let text_height=(inner_height-compound[1]-compound[3]).max(0.0);
            let text_width=(width-padding[0]-padding[2]-compound[0]-compound[2]).max(0.0);
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
            let text=self.display_text_for(state.text.get(&key).map_or(self.text,String::as_str), state);
            let layout_ext = self.text_layout_ext();
            let base_style = TextStyle{size:effective_size,weight:effective_weight,letter_spacing:self.letter_spacing_em*effective_size,italic,family,..TextStyle::default()};
            let effective_size = if let Some(ext) = layout_ext && ext.has_auto_size() {
                self.resolve_auto_text_size(&text, text_width, text_height, base_style, ext, state, metrics)
            } else {
                effective_size
            };
            let style=TextStyle{size:effective_size,weight:effective_weight,letter_spacing:self.letter_spacing_em*effective_size,italic,family,..TextStyle::default()};
            let simple=layout_ext.is_none() && !text.contains(['\n','\r','\u{000b}','\u{000c}','\u{0085}','\u{2028}','\u{2029}'])&&metrics.width(&text,style)<=text_width+0.01&&self.min_lines<=1&&self.ellipsize_side==0&&!self.ellipsize_end;
            if simple {return Some(padding[1]+compound[1]+text_height*0.5+effective_size*offset);}
            let line_height=self.resolved_line_height(effective_size,state);let advance=(line_height*self.line_spacing_multiplier+state.dp(self.line_spacing_extra)).max(1.0);
            let single=self.single_line || (self.max_lines==1 && (self.ellipsize_side!=0||self.ellipsize_end));
            let total=if single {1} else if let Some(ext) = layout_ext {wrap_text_lines_ext(&text,text_width+0.01,style,metrics,ext).len().max(1)} else {wrapped_text_lines(&text,text_width+0.01,style,metrics,|_|{}).max(1)};
            let height_limit=if text_height<=0.0 {0}else{((text_height-line_height).max(0.0)/advance).ceil() as usize+1};
            let limit=if self.max_lines==0 {height_limit}else{height_limit.min(self.max_lines)};let shown=total.min(limit);
            let used=if shown==0 {0.0}else{(line_height+(shown-1) as f32*advance).min(text_height)};
            let extra=(text_height-used).max(0.0);let gravity=if self.relative_rules.iter().any(|rule|rule.kind==168){self.content_gravity[1]}else{1};
            let shift=match gravity {0=>0.0,2=>extra,_=>extra*0.5};
            return Some(padding[1]+compound[1]+shift+line_height*0.5+effective_size*offset);
        }
        let scaled_text=self.scaled_text_size(state);
        if matches!(self.kind,Kind::ToggleButton) {
            return Some(padding[1]+inner_height*0.5+scaled_text*offset);
        }
        if matches!(self.kind,Kind::Edit|Kind::AutoComplete|Kind::MultiAutoComplete) {
            // Editable text is painted in the lower portion of Bhoo's 48 dp field.
            let field_height=inner_height.min(state.dp(48.0));
            let top_offset=state.dp(16.0);
            return Some(padding[1]+top_offset+(field_height-top_offset).max(0.0)*0.5+state.sp(TextStyle::default().size)*metrics.normal_baseline_offset);
        }
        if matches!(self.kind,Kind::CheckBox|Kind::RadioButton|Kind::Switch) {
            // These controls render their label in their allocated 40/36 dp row,
            // clipped to the inner height, before applying the XML text style.
            let row_height=state.dp(if matches!(self.kind,Kind::Switch) {36.0_f32} else {40.0_f32});
            return Some(padding[1]+inner_height.min(row_height)*0.5+scaled_text*offset);
        }
        if !matches!(self.kind,Kind::Text|Kind::CheckedText|Kind::Chronometer|Kind::TextClock) { return None; }
        let rtl=match self.layout_direction {1=>false,2=>true,_=>state.rtl};
        let compound=if matches!(self.kind,Kind::Text|Kind::CheckedText){self.compound_extents(state,rtl)}else{[0.0;4]};
        let inner_height=(inner_height-compound[1]-compound[3]).max(0.0);
        let inner_width=(width-padding[0]-padding[2]-compound[0]-compound[2]-if matches!(self.kind,Kind::CheckedText) {self.check_mark_width(state)} else {0.0}).max(0.0);
        let key=if self.id.is_empty() {format!("#{id}")} else {self.id.to_string()};
        let text=self.display_text_for(state.text.get(&key).map_or(self.text,String::as_str), state);
        let layout_ext = self.text_layout_ext();
        let base_style=TextStyle {size:scaled_text,weight:if self.text_bold {700} else {400},letter_spacing:self.letter_spacing_em*scaled_text,italic,family,..TextStyle::default()};
        let scaled_text = if let Some(ext) = layout_ext && ext.has_auto_size() {
            self.resolve_auto_text_size(&text, inner_width, inner_height, base_style, ext, state, metrics)
        } else {
            scaled_text
        };
        let line_height=self.resolved_line_height(scaled_text,state);
        let advance=(line_height*self.line_spacing_multiplier+state.dp(self.line_spacing_extra)).max(1.0);
        let style=TextStyle {size:scaled_text,weight:if self.text_bold {700} else {400},letter_spacing:self.letter_spacing_em*scaled_text,italic,family,..TextStyle::default()};
        let used=if self.single_line || (self.max_lines==1 && self.ellipsize_side!=0) {line_height} else {
            let total=if let Some(ext) = layout_ext {wrap_text_lines_ext(&text,inner_width,style,metrics,ext).len().max(1)} else {wrapped_text_lines(&text,inner_width,style,metrics,|_| {})};
            let height_limit=if inner_height<=0.0 {0} else {((inner_height-line_height).max(0.0)/advance).ceil() as usize+1};
            let shown=total.min(if self.max_lines==0 {height_limit} else {height_limit.min(self.max_lines)});
            if shown==0 {0.0} else {(line_height+(shown-1) as f32*advance).min(inner_height)}
        };
        let extra=(inner_height-used).max(0.0);
        let shift=match self.content_gravity[1] {1=>extra*0.5,2=>extra,_=>0.0};
        Some(padding[1]+compound[1]+shift+line_height*0.5+scaled_text*offset)
    }
    pub(super) fn resolved_state_color(&self, rules:&[TintState], fallback:Option<Color>, state:&LayoutState, key:&str, response:Response) -> Option<Color> {
        if rules.is_empty() { return fallback; }
        let checkable = matches!(self.kind, Kind::CheckedText | Kind::CheckBox | Kind::RadioButton | Kind::Switch | Kind::ToggleButton)
            || self.chip_style().is_some()
            || state.toggle_group_segment.is_some();
        let bits=(u8::from(response.enabled)) | (u8::from(response.pressed)<<1)
            | (u8::from(response.focused)<<2) | (u8::from(state.view_selected.get(key).copied().unwrap_or(false))<<3)
            | (u8::from(state.view_activated.get(key).copied().unwrap_or(false))<<4) | (u8::from(response.hovered)<<5)
            | (u8::from(checkable && state.checked.get(key).copied().unwrap_or(self.checked))<<6);
        rules.iter().find(|rule| bits & rule.required == rule.required && bits & rule.forbidden == 0)
            .or_else(||rules.iter().rev().find(|rule|rule.required==0 && rule.forbidden==0))
            .or_else(||rules.first()).map(|rule|rule.color)
    }
    pub(super) fn live_state_mask(&self, state: &LayoutState, key: &str, response: Response) -> u8 {
        u8::from(response.enabled)
            | (u8::from(response.pressed) << 1)
            | (u8::from(response.focused) << 2)
            | (u8::from(state.view_selected.get(key).copied().unwrap_or(false)) << 3)
            | (u8::from(state.view_activated.get(key).copied().unwrap_or(false)) << 4)
            | (u8::from(response.hovered) << 5)
            | (u8::from(state.checked.get(key).copied().unwrap_or(self.checked)) << 6)
    }
    pub(super) fn button_variant(&self) -> Option<XmlButtonVariant> {
        if !matches!(self.kind, Kind::Button) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_BUTTON_VARIANT)
            .and_then(|r| parse_button_variant(r.anchor))
    }
    pub(super) fn chip_style(&self) -> Option<XmlChipSpec> {
        if !matches!(self.kind, Kind::Button) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_CHIP_STYLE)
            .map(|r| parse_chip_style(r.anchor))
    }
    pub(super) fn text_input_box(&self) -> Option<TextInputBoxSpec<'static>> {
        if !matches!(self.kind, Kind::Linear) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_TEXT_INPUT_BOX)
            .map(|r| parse_text_input_box(r.anchor))
    }
    pub(super) fn clip_to_outline(&self) -> bool {
        self.relative_rules
            .iter()
            .any(|r| r.kind == RULE_CLIP_TO_OUTLINE && matches!(r.anchor.trim().to_ascii_lowercase().as_str(), "true" | "1"))
    }
    pub(super) fn resolved_line_height(&self, text_size: f32, state: &LayoutState) -> f32 {
        if let Some(val) = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_LINE_HEIGHT)
            .and_then(|r| r.anchor.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
        {
            return (val * state.effective_density() * state.effective_font_scale())
                .max(text_size)
                .max(1.0);
        }
        let include_font_padding = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_INCLUDE_FONT_PADDING)
            .map(|r| !matches!(r.anchor.trim().to_ascii_lowercase().as_str(), "false" | "0"))
            .unwrap_or(true);
        let factor = if include_font_padding {
            state.theme_style.text_line_height_factor()
        } else {
            1.18
        };
        (text_size * factor).max(1.0)
    }
    pub(super) fn nav_bar_spec(&self) -> Option<NavBarSpec> {
        if !matches!(self.kind, Kind::Linear) {
            return None;
        }
        if !self.relative_rules.iter().any(|r| r.kind == RULE_NAV_ITEM) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_NAV_BAR)
            .map(|r| parse_nav_bar(r.anchor))
    }
    pub(super) fn nav_items(&self) -> Vec<NavItemSpec> {
        self.relative_rules
            .iter()
            .filter(|r| r.kind == RULE_NAV_ITEM)
            .map(|r| parse_nav_item(r.anchor))
            .collect()
    }
    pub(super) fn search_bar_hint(&self) -> Option<&'static str> {
        if !matches!(self.kind, Kind::SearchView) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_SEARCH_BAR)
            .map(|r| r.anchor)
    }
    pub(super) fn search_view_ext(&self) -> Option<SearchViewExtSpec<'static>> {
        if !matches!(self.kind, Kind::Frame | Kind::Linear) {
            return None;
        }
        self.relative_rules
            .iter()
            .rev()
            .find(|r| r.kind == RULE_SEARCH_VIEW_EXT)
            .map(|r| parse_search_view_ext(r.anchor))
    }
    pub(super) fn is_search_view_closed(&self, state: &LayoutState, id: u64) -> bool {
        if self.search_view_ext().is_none() {
            return false;
        }
        let key = if self.id.is_empty() {
            format!("#{id}")
        } else {
            self.id.to_owned()
        };
        if state.search_open.get(&key).copied().unwrap_or(false) {
            return false;
        }
        state
            .search_view_motion
            .get(&key)
            .map_or(0.0, |m| m.progress)
            <= 0.001
    }
    pub(super) fn toggle_group_spec(&self) -> Option<ToggleGroupSpec> {
        if !matches!(self.kind, Kind::Linear) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_TOGGLE_GROUP)
            .map(|r| parse_toggle_group(r.anchor))
    }
    pub(super) fn extended_fab_spec(&self) -> Option<ExtendedFabSpec> {
        if !matches!(self.kind, Kind::Button) {
            return None;
        }
        let legacy = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_EXTENDED_FAB)
            .map(|r| !matches!(r.anchor.trim().to_ascii_lowercase().as_str(), "0" | "false"));
        if let Some(ext_rule) = self
            .relative_rules
            .iter()
            .rev()
            .find(|r| r.kind == RULE_EXTENDED_FAB_EXT)
        {
            return Some(parse_extended_fab_ext(ext_rule.anchor, legacy));
        }
        legacy.map(|static_extended| ExtendedFabSpec {
            static_extended,
            collapsed_size_dp: 56.0,
            strategy: ExtendedFabStrategy::Auto,
            has_ext: false,
        })
    }
    pub(super) fn extended_fab_extended(&self) -> Option<bool> {
        self.extended_fab_spec().map(|s| s.static_extended)
    }
    pub(super) fn extended_fab_progress(&self, state: &LayoutState, id: u64) -> Option<(ExtendedFabSpec, XmlLiftMotion, bool)> {
        let spec = self.extended_fab_spec()?;
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        let target = state.fab_extended.get(&key).copied().unwrap_or(spec.static_extended);
        let prev = state.extended_fab_motion.get(&key).copied().unwrap_or_default();
        let now = if state.frame_time.is_finite() { state.frame_time.max(0.0) } else { 0.0 };
        let (next, anim) = eval_binary_ease_out_motion(prev, target, 0.22, now, state.frame_reduced_motion);
        Some((spec, next, anim))
    }
    pub(super) fn extended_fab_intrinsic_extended_width(
        &self,
        state: &LayoutState,
        metrics: TextMetrics,
        id: u64,
        spec: ExtendedFabSpec,
    ) -> f32 {
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        let label = self.display_text_for(state.text.get(&key).map_or(self.text, String::as_str), state);
        let (text_size, weight, italic, family) = self.effective_text_font(state);
        let style = TextStyle {
            size: text_size,
            weight,
            letter_spacing: self.letter_spacing_em * text_size,
            italic,
            family,
            ..TextStyle::default()
        };
        let text_w = label
            .split(['\n', '\r', '\u{000b}', '\u{000c}', '\u{0085}', '\u{2028}', '\u{2029}'])
            .map(|line| metrics.width(line, style))
            .fold(0.0_f32, f32::max);
        let rtl = match self.layout_direction { 1 => false, 2 => true, _ => state.rtl };
        let ext_def_pad = if rtl { [20.0, 0.0, 16.0, 0.0] } else { [16.0, 0.0, 20.0, 0.0] };
        let raw_pad = self.resolved_padding(state);
        let pad_l = if self.padding_explicit[0] || self.padding_edges[0] != 0.0 || self.padding_relative[if rtl { 1 } else { 0 }].is_some() {
            raw_pad[0]
        } else {
            state.dp(ext_def_pad[0])
        };
        let pad_r = if self.padding_explicit[2] || self.padding_edges[2] != 0.0 || self.padding_relative[if rtl { 0 } else { 1 }].is_some() {
            raw_pad[2]
        } else {
            state.dp(ext_def_pad[2])
        };
        let shapes = self.compound_sizes(state, rtl);
        let extent = self.compound_extents(state, rtl);
        let has_compound = shapes.iter().any(Option::is_some);
        let icon_w = if !has_compound && !self.image_src.is_empty() {
            state.dp(24.0) + if label.is_empty() { 0.0 } else { state.dp(12.0) }
        } else {
            0.0
        };
        let min_w = state.dp(self.min_width);
        let bg_min_w = state.dp(self.background_min_size[0]);
        let min_ext = state.dp(80.0).max(state.dp(spec.collapsed_size_dp));
        ((text_w + icon_w + extent[0] + extent[2])
            .max(shapes[1].map_or(0.0, |s| s.0))
            .max(shapes[3].map_or(0.0, |s| s.0))
            + pad_l
            + pad_r)
            .max(min_ext)
            .max(min_w)
            .max(bg_min_w)
    }
    pub(super) fn bottom_app_bar_spec(&self) -> Option<BottomAppBarSpec> {
        if !matches!(self.kind, Kind::Toolbar) {
            return None;
        }
        let legacy = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_BOTTOM_APP_BAR)
            .map(|r| match r.anchor.trim().to_ascii_lowercase().as_str() {
                "end" => BottomAppBarFabAlignment::End,
                _ => BottomAppBarFabAlignment::Center,
            });
        if let Some(ext_rule) = self
            .relative_rules
            .iter()
            .rev()
            .find(|r| r.kind == RULE_BOTTOM_APP_BAR_EXT)
        {
            return Some(parse_bottom_app_bar_ext(ext_rule.anchor, legacy));
        }
        legacy.map(|alignment| BottomAppBarSpec {
            alignment,
            anchor_mode: BottomAppBarAnchorMode::Cradle,
            cradle_margin_dp: 5.0,
            cradle_rad_dp: 8.0,
            cradle_voff_dp: 0.0,
            hide_on_scroll: false,
            has_ext: false,
        })
    }
    pub(super) fn contains_hide_on_scroll_bar(&self) -> bool {
        if self.bottom_app_bar_spec().is_some_and(|s| s.hide_on_scroll) {
            return true;
        }
        matches!(self.kind, Kind::Linear | Kind::Frame)
            && self
                .children
                .iter()
                .any(|c| c.bottom_app_bar_spec().is_some_and(|s| s.hide_on_scroll))
    }
    pub(super) fn collapsing_toolbar_ext(&self) -> Option<CollapsingToolbarExt<'_>> {
        if !matches!(self.kind, Kind::Frame) {
            return None;
        }
        self.relative_rules
            .iter()
            .rev()
            .filter(|r| r.kind == RULE_COLLAPSING_TOOLBAR_EXT)
            .find_map(|r| parse_collapsing_toolbar_ext(r.anchor))
    }
    pub(super) fn collapsing_child_mode(&self) -> Option<CollapsingChildMode> {
        self.relative_rules
            .iter()
            .rev()
            .filter(|r| r.kind == RULE_COLLAPSING_TOOLBAR_EXT)
            .find_map(|r| parse_collapsing_child_mode(r.anchor))
    }
    pub(super) fn recycler_layout_ext(&self) -> Option<RecyclerLayoutExt> {
        if !matches!(self.kind, Kind::ListView | Kind::GridView) {
            return None;
        }
        self.relative_rules
            .iter()
            .rev()
            .find(|r| r.kind == RULE_RECYCLER_LAYOUT_EXT)
            .map(|r| parse_recycler_layout_ext(r.anchor, matches!(self.kind, Kind::GridView)))
    }
    pub(super) fn nav_badge_for<'a>(&'a self, item_id: &str, state: &'a LayoutState) -> Option<&'a str> {
        if item_id.is_empty() {
            return None;
        }
        if let Some(runtime_text) = state.nav_badges.get(item_id) {
            return Some(runtime_text.as_str());
        }
        self.relative_rules
            .iter()
            .rev()
            .filter(|r| r.kind == RULE_NAV_BADGE)
            .find_map(|r| {
                let (bid, btext) = r.anchor.split_once(':')?;
                (bid.trim() == item_id).then_some(btext)
            })
    }
    pub(super) fn nav_badge_for_item<'a>(&'a self, item_index: usize, item_id: &str, state: &'a LayoutState) -> Option<&'a str> {
        if let Some(b) = self.nav_badge_for(item_id, state) {
            return Some(b);
        }
        self.relative_rules
            .iter()
            .rev()
            .filter(|r| r.kind == RULE_NAV_BADGE)
            .find_map(|r| {
                let (bid, btext) = r.anchor.split_once(':')?;
                (bid.trim().parse::<usize>().ok() == Some(item_index)).then_some(btext)
            })
    }
    pub(super) fn material_badge_ext(&self, item_index: Option<usize>, item_id: &str) -> Option<MaterialBadgeExt> {
        self.relative_rules
            .iter()
            .rev()
            .filter(|r| r.kind == RULE_MATERIAL_BADGE_EXT)
            .find_map(|r| {
                let (target, ext) = parse_material_badge_ext(r.anchor)?;
                let matched = match item_index {
                    Some(idx) => match target {
                        BadgeTarget::Index(i) => i == idx,
                        BadgeTarget::Id(id) => !item_id.is_empty() && id == item_id,
                        BadgeTarget::SelfView => false,
                    },
                    None => matches!(target, BadgeTarget::SelfView),
                };
                matched.then_some(ext)
            })
    }
    pub(super) fn standalone_badge<'a>(&'a self, state: &'a LayoutState) -> Option<&'a str> {
        let from_rule = self
            .relative_rules
            .iter()
            .rev()
            .filter(|r| r.kind == RULE_NAV_BADGE)
            .find_map(|r| {
                let (bid, btext) = r.anchor.split_once(':')?;
                let bid = bid.trim();
                let is_self = bid == "-1"
                    || bid.eq_ignore_ascii_case("self")
                    || (!self.id.is_empty() && bid == self.id);
                is_self.then_some(btext)
            });
        if from_rule.is_some() {
            return from_rule;
        }
        if !self.id.is_empty()
            && self.material_badge_ext(None, self.id).is_some()
            && let Some(rt) = state.nav_badges.get(self.id)
        {
            return Some(rt.as_str());
        }
        None
    }
    pub(super) fn text_layout_ext(&self) -> Option<TextLayoutExt> {
        if !matches!(self.kind, Kind::Text | Kind::Button) {
            return None;
        }
        self.relative_rules
            .iter()
            .rev()
            .find(|r| r.kind == RULE_TEXT_LAYOUT_EXT)
            .map(|r| parse_text_layout_ext(r.anchor))
            .filter(TextLayoutExt::has_custom_layout)
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve_auto_text_size(
        &self,
        text: &str,
        avail_w: f32,
        avail_h: f32,
        base_style: TextStyle,
        ext: TextLayoutExt,
        state: &LayoutState,
        metrics: TextMetrics,
    ) -> f32 {
        let min_sp = ext.auto_min_sp;
        let max_sp = ext.auto_max_sp.max(min_sp);
        let step = if ext.auto_step_sp > 0.0 { ext.auto_step_sp } else { 1.0 };
        let max_k = ((max_sp - min_sp + 1e-4) / step).floor().max(0.0) as usize;
        if avail_w <= 0.0 || avail_h <= 0.0 || max_k == 0 {
            return state.sp(min_sp);
        }
        let single = self.single_line || self.max_lines == 1;
        let visible_single = if single { text.replace('\u{00ad}', "") } else { String::new() };
        let fits = |sp: f32| -> bool {
            let px = state.sp(sp);
            let style = TextStyle {
                size: px,
                letter_spacing: self.letter_spacing_em * px,
                ..base_style
            };
            let lh = self.resolved_line_height(px, state);
            if lh > avail_h + 0.01 {
                return false;
            }
            if single {
                return metrics.width(&visible_single, style) <= avail_w + 0.01;
            }
            if ext.hyphenation == TextHyphenationFrequency::None {
                for word in text.split(breakable_space).filter(|w| !w.is_empty()) {
                    let clean = word.replace('\u{00ad}', "");
                    if metrics.width(&clean, style) > avail_w + 0.01 {
                        return false;
                    }
                }
            }
            let lines = wrap_text_lines_ext(text, avail_w, style, metrics, ext);
            let count = lines.len().max(1);
            if self.max_lines > 0 && count > self.max_lines {
                return false;
            }
            let adv = (lh * self.line_spacing_multiplier + state.dp(self.line_spacing_extra)).max(1.0);
            let used_h = lh + (count - 1) as f32 * adv;
            if used_h > avail_h + 0.01 {
                return false;
            }
            lines.iter().all(|l| metrics.width(&l.text, style) <= avail_w + 0.01)
        };
        if !fits(min_sp) {
            return state.sp(min_sp);
        }
        let mut lo = 0usize;
        let mut hi = max_k;
        while lo < hi {
            let mid = lo + (hi - lo + 1) / 2;
            let cand = (min_sp + mid as f32 * step).min(max_sp);
            if fits(cand) {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        let chosen_sp = (min_sp + lo as f32 * step).min(max_sp);
        state.sp(chosen_sp)
    }
    pub(super) fn swipe_refresh_ext(&self) -> Option<SwipeRefreshExtSpec> {
        if !matches!(self.kind, Kind::Frame) {
            return None;
        }
        self.relative_rules
            .iter()
            .rev()
            .find(|r| r.kind == RULE_SWIPE_REFRESH_EXT)
            .map(|r| parse_swipe_refresh_ext(r.anchor))
    }
    pub(super) fn is_swipe_refresh(&self) -> bool {
        matches!(self.kind, Kind::Frame)
            && self
                .relative_rules
                .iter()
                .any(|r| r.kind == RULE_SWIPE_REFRESH || r.kind == RULE_SWIPE_REFRESH_EXT)
    }
    pub(super) fn carousel_layout_spec(&self) -> Option<CarouselLayoutSpec> {
        if !matches!(
            self.kind,
            Kind::ListView | Kind::GridView | Kind::HorizontalScroll | Kind::Linear
        ) {
            return None;
        }
        self.relative_rules
            .iter()
            .rev()
            .find(|r| r.kind == RULE_CAROUSEL_LAYOUT)
            .map(|r| parse_carousel_layout(r.anchor))
    }
    pub(super) fn blocks_pointer_for_lower_siblings_in(&self, state: &LayoutState, id: u64) -> bool {
        if self.search_view_ext().is_some() {
            return !self.is_search_view_closed(state, id);
        }
        self.blocks_pointer_for_lower_siblings()
    }
    pub(super) fn is_drawer_layout(&self) -> bool {
        matches!(self.kind, Kind::Frame)
            && self.relative_rules.iter().any(|r| r.kind == RULE_DRAWER_LAYOUT)
    }
    pub(super) fn blocks_pointer_for_lower_siblings(&self) -> bool {
        self.clickable
            || matches!(
                self.kind,
                Kind::Button
                    | Kind::Icon
                    | Kind::Zoom
                    | Kind::ZoomControls
                    | Kind::CheckBox
                    | Kind::RadioButton
                    | Kind::Switch
                    | Kind::SeekBar
                    | Kind::RatingBar
                    | Kind::ToggleButton
                    | Kind::Edit
                    | Kind::AutoComplete
                    | Kind::MultiAutoComplete
                    | Kind::MaterialCard
                    | Kind::Spinner
                    | Kind::ListView
                    | Kind::GridView
                    | Kind::QuickContactBadge
                    | Kind::VideoView
                    | Kind::CalendarView
                    | Kind::DatePicker
                    | Kind::NumberPicker
                    | Kind::TimePicker
                    | Kind::SearchView
                    | Kind::ExpandableListView
                    | Kind::Toolbar
                    | Kind::ActionMenuView
                    | Kind::TabWidget
            )
            || self.bottom_sheet_spec().is_some()
            || self.side_sheet_spec().is_some()
            || self.is_snackbar()
            || self.is_floating_toolbar()
            || self.alert_dialog_icon().is_some()
            || self.nav_bar_spec().is_some()
            || self.search_bar_hint().is_some()
    }
    pub(super) fn collapsing_toolbar_spec(&self) -> Option<CollapsingToolbarSpec<'_>> {
        if !matches!(self.kind, Kind::Frame) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_COLLAPSING_TOOLBAR)
            .map(|r| parse_collapsing_toolbar(r.anchor, self.height))
    }
    pub(super) fn collapsing_toolbar_scroll_px(&self, state: &LayoutState, id: u64) -> f32 {
        let sib = state
            .collapsing_scroll_offset
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .max(0.0);
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        let own = state
            .offsets
            .get(&key)
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .max(0.0);
        sib.max(own)
    }
    pub(super) fn collapsing_toolbar_height(&self, state: &LayoutState, id: u64) -> Option<f32> {
        let spec = self.collapsing_toolbar_spec()?;
        let expanded_px = state.dp(spec.expanded_h_dp);
        let collapsed_px = state.dp(64.0).min(expanded_px);
        let scroll_px = self.collapsing_toolbar_scroll_px(state, id);
        Some((expanded_px - scroll_px).clamp(collapsed_px, expanded_px))
    }
    pub(super) fn contains_collapsing_toolbar(&self) -> bool {
        if self.collapsing_toolbar_spec().is_some() {
            return true;
        }
        matches!(self.kind, Kind::Linear | Kind::Frame)
            && self.children.iter().any(|c| c.collapsing_toolbar_spec().is_some())
    }
    pub(super) fn lift_on_scroll_spec(&self) -> Option<Option<&'static str>> {
        if !matches!(self.kind, Kind::Linear | Kind::Frame | Kind::Toolbar) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_APP_BAR_LIFT_ON_SCROLL)
            .and_then(|r| {
                let trimmed = r.anchor.trim();
                let (flag, target) = match trimmed.split_once('|') {
                    Some((a, b)) => (a.trim(), Some(b.trim())),
                    None => (trimmed, None),
                };
                if !matches!(flag.to_ascii_lowercase().as_str(), "1" | "true") {
                    return None;
                }
                Some(target.filter(|s| !s.is_empty()))
            })
    }
    pub(super) fn contains_lift_on_scroll(&self) -> bool {
        if self.lift_on_scroll_spec().is_some() {
            return true;
        }
        matches!(self.kind, Kind::Linear | Kind::Frame)
            && self.children.iter().any(|c| c.lift_on_scroll_spec().is_some())
    }
    pub(super) fn app_bar_scroll_flags(&self) -> Option<AppBarScrollFlags> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_APP_BAR_SCROLL_FLAGS)
            .map(|r| parse_app_bar_scroll_flags(r.anchor))
            .filter(|f| f.scroll)
    }
    pub(super) fn app_bar_lift_color(&self) -> Option<Color> {
        self.relative_rules
            .iter()
            .filter(|r| r.kind == RULE_APP_BAR_EXT)
            .find_map(|r| parse_app_bar_ext_lift_color(r.anchor))
    }
    pub(super) fn app_bar_scroll_effect_compress(&self) -> bool {
        self.relative_rules
            .iter()
            .any(|r| r.kind == RULE_APP_BAR_EXT && parse_app_bar_ext_compress(r.anchor))
    }
    pub(super) fn static_drawer_lock_mode(&self) -> Option<u8> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_DRAWER_LOCK_MODE)
            .and_then(|r| parse_drawer_lock_mode(r.anchor))
    }
    pub(super) fn contains_app_bar_scroll_flags(&self) -> bool {
        if self.app_bar_scroll_flags().is_some() {
            return true;
        }
        matches!(self.kind, Kind::Linear | Kind::Frame)
            && self.children.iter().any(|c| c.app_bar_scroll_flags().is_some())
    }
    pub(super) fn app_bar_scroll_px(&self, state: &LayoutState, id: u64) -> f32 {
        if let Some(sib) = state.collapsing_scroll_offset.filter(|v| v.is_finite()) {
            return sib.max(0.0);
        }
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        state
            .offsets
            .get(&key)
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .max(0.0)
    }
    pub(super) fn app_bar_child_scroll_px(&self, parent_scroll: f32, state: &LayoutState, child_id: u64) -> f32 {
        if state.collapsing_scroll_offset.is_some() || parent_scroll > 0.0 {
            return parent_scroll;
        }
        let key = if self.id.is_empty() { format!("#{child_id}") } else { self.id.to_owned() };
        state
            .offsets
            .get(&key)
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .max(0.0)
    }
    pub(super) fn app_bar_children_collapse_total(
        &self,
        state: &LayoutState,
        inner_width: f32,
        metrics: TextMetrics,
        id: u64,
    ) -> f32 {
        if !matches!(self.kind, Kind::Linear) || self.horizontal {
            return 0.0;
        }
        let parent_scroll = self.app_bar_scroll_px(state, id);
        let mut total = 0.0_f32;
        for (i, n) in self.children.iter().enumerate() {
            if n.effective_visibility(state) == Visibility::Gone
                || n.app_bar_scroll_flags().is_none()
                || n.collapsing_toolbar_spec().is_some()
            {
                continue;
            }
            let cid = child_view_id(id, i, n);
            let m = n.resolved_margin(state);
            let avail_w = (inner_width - m[0] - m[2]).max(0.0);
            let natural_h = n.layout_intrinsic(false, state, avail_w, metrics, cid);
            let scroll_px = n.app_bar_child_scroll_px(parent_scroll, state, cid);
            total += preview_app_bar_child_collapse(n, cid, natural_h, m[1] + m[3], scroll_px, state);
        }
        total
    }
    pub(super) fn drawer_header_intrinsic_height(&self, state: &LayoutState, id: u64) -> f32 {
        let mut h = 0.0_f32;
        let mut any = false;
        for (i, child) in self.children.iter().enumerate() {
            if child.effective_visibility(state) == Visibility::Gone {
                continue;
            }
            any = true;
            let m = child.resolved_margin(state);
            h += child.intrinsic(false, state, child_view_id(id, i, child)) + m[1] + m[3];
        }
        if any {
            h + state.dp(1.0)
        } else {
            0.0
        }
    }
    pub(super) fn sibling_scroll_offset(&self, state: &LayoutState, id: u64) -> Option<f32> {
        for (i, sib) in self.children.iter().enumerate() {
            if sib.effective_visibility(state) == Visibility::Gone
                || sib.contains_collapsing_toolbar()
                || sib.contains_lift_on_scroll()
                || sib.contains_app_bar_scroll_flags()
                || sib.contains_hide_on_scroll_bar()
            {
                continue;
            }
            if sib.relative_rules.iter().any(|r| r.kind == RULE_COORDINATOR_SCROLL_BEHAVIOR)
                && let Some(off) = find_scroll_view_offset(sib, state, child_view_id(id, i, sib))
            {
                return Some(off);
            }
        }
        for (i, sib) in self.children.iter().enumerate() {
            if sib.effective_visibility(state) == Visibility::Gone
                || sib.contains_collapsing_toolbar()
                || sib.contains_lift_on_scroll()
                || sib.contains_app_bar_scroll_flags()
                || sib.contains_hide_on_scroll_bar()
            {
                continue;
            }
            if let Some(off) = find_scroll_view_offset(sib, state, child_view_id(id, i, sib)) {
                return Some(off);
            }
        }
        None
    }
    pub(super) fn view_pager_spec(&self) -> Option<ViewPagerSpec> {
        if !matches!(self.kind, Kind::ViewFlipper) {
            return None;
        }
        let legacy = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_VIEW_PAGER)
            .map(|r| parse_view_pager_orientation(r.anchor));
        if let Some(ext) = self
            .relative_rules
            .iter()
            .rev()
            .find(|r| r.kind == RULE_VIEW_PAGER_EXT)
        {
            return Some(parse_view_pager_ext(ext.anchor, legacy));
        }
        legacy.map(|orient| ViewPagerSpec {
            orient,
            user_input_enabled: true,
            page_margin_dp: 0.0,
            transformer: ViewPagerTransformer::None,
            has_ext: false,
        })
    }
    pub(super) fn range_slider_initial(&self) -> Option<[f32; 2]> {
        if !matches!(self.kind, Kind::SeekBar) {
            return None;
        }
        let rule = self.relative_rules.iter().find(|r| r.kind == RULE_RANGE_SLIDER)?;
        let max = if self.max > self.min { self.max } else { self.min + 100.0 };
        Some(parse_range_slider_values(rule.anchor, self.min, max))
    }
    pub(super) fn is_drawer_panel_child(&self) -> bool {
        let has_drawer_nav = self
            .relative_rules
            .iter()
            .any(|r| r.kind == RULE_NAV_BAR && parse_nav_bar(r.anchor).kind == NavBarKind::Drawer);
        if has_drawer_nav {
            return true;
        }
        self.layout_gravity_explicit
            && matches!(self.layout_gravity[0], 0 | 2 | 3 | 4)
            && !matches!(self.width, Size::Match)
    }
    pub(super) fn drawer_panel_is_logical_end(&self, rtl: bool) -> bool {
        match self.layout_gravity[0] {
            2 => true,
            3 => rtl,
            4 => !rtl,
            _ => false,
        }
    }
    pub(super) fn resolved_drawer_lock_mode(
        &self,
        is_end: bool,
        has_start: bool,
        child_lock: Option<u8>,
        state: &LayoutState,
        key: &str,
        end_key: &str,
    ) -> u8 {
        let rt_lock = if is_end {
            if has_start {
                state.drawer_lock_mode.get(end_key).copied()
            } else {
                state
                    .drawer_lock_mode
                    .get(end_key)
                    .copied()
                    .or_else(|| state.drawer_lock_mode.get(key).copied())
            }
        } else {
            state.drawer_lock_mode.get(key).copied()
        };
        rt_lock
            .or(child_lock)
            .or_else(|| self.static_drawer_lock_mode())
            .unwrap_or(DRAWER_LOCK_MODE_UNLOCKED)
    }
    pub(super) fn drawer_panel_is_closed(&self, child: &Node, state: &LayoutState, id: u64) -> bool {
        if !self.is_drawer_layout() || !child.is_drawer_panel_child() {
            return false;
        }
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        let end_key = format!("{key}:end");
        let has_start = self
            .children
            .iter()
            .any(|c| c.effective_visibility(state) != Visibility::Gone && c.is_drawer_panel_child() && !c.drawer_panel_is_logical_end(state.rtl));
        let is_end = child.drawer_panel_is_logical_end(state.rtl);
        let lock_mode = self.resolved_drawer_lock_mode(
            is_end,
            has_start,
            child.static_drawer_lock_mode(),
            state,
            &key,
            &end_key,
        );
        if lock_mode == DRAWER_LOCK_MODE_LOCKED_CLOSED {
            return true;
        }
        if lock_mode == DRAWER_LOCK_MODE_LOCKED_OPEN {
            return false;
        }
        if is_end && has_start {
            !state.drawer_open.get(&end_key).copied().unwrap_or(false)
        } else if is_end {
            !(state.drawer_open.get(&end_key).copied().unwrap_or(false)
                || state.drawer_open.get(&key).copied().unwrap_or(false))
        } else {
            !state.drawer_open.get(&key).copied().unwrap_or(false)
        }
    }
    pub(super) fn has_animated_selector(&self, is_src: bool) -> bool {
        self.relative_rules.iter().any(|r| {
            r.kind == RULE_ANIMATED_SELECTOR
                && parse_animated_selector_transition(r.anchor).is_some_and(|t| t.is_src == is_src)
        })
    }
    pub(super) fn sample_animated_selector(
        &self,
        f: &mut Frame<'_>,
        b: Rect,
        state: &mut LayoutState,
        view_key: &str,
        response: Response,
        is_src: bool,
    ) -> Option<&'static str> {
        let transitions: Vec<AnimatedSelectorTransition<'static>> = self
            .relative_rules
            .iter()
            .filter(|r| r.kind == RULE_ANIMATED_SELECTOR)
            .filter_map(|r| parse_animated_selector_transition(r.anchor))
            .filter(|t| t.is_src == is_src)
            .collect();
        if transitions.is_empty() {
            return None;
        }
        let map_key = format!("{}:{view_key}", if is_src { "src" } else { "bg" });
        let new_mask = self.live_state_mask(state, view_key, response) & 0x7F;
        let now = if f.time().is_finite() { f.time().max(0.0) } else { 0.0 };
        let reduced = f.reduced_motion();

        let mut st = state
            .animated_selector_states
            .get(&map_key)
            .copied()
            .unwrap_or_default();

        let mut chosen_frame: Option<&'static str> = None;
        if !st.initialized {
            st.initialized = true;
            st.last_mask = new_mask;
            st.active = None;
        } else if reduced {
            st.last_mask = new_mask;
            st.active = None;
        } else {
            if new_mask != st.last_mask {
                let old_mask = st.last_mask;
                st.last_mask = new_mask;
                let matched = transitions
                    .iter()
                    .enumerate()
                    .find_map(|(idx, tr)| {
                        matches_animated_selector_transition(old_mask, new_mask, tr.from_mask, tr.to_mask)
                            .then_some((idx, false))
                    })
                    .or_else(|| {
                        transitions.iter().enumerate().find_map(|(idx, tr)| {
                            (tr.reversible
                                && matches_animated_selector_transition(
                                    old_mask,
                                    new_mask,
                                    tr.to_mask,
                                    tr.from_mask,
                                ))
                            .then_some((idx, true))
                        })
                    });
                if let Some((rule_idx, reversed)) = matched {
                    st.active = Some((now, reversed, rule_idx));
                } else if let Some((_, rev, r_idx)) = st.active {
                    if let Some(tr) = transitions.get(r_idx) {
                        let (eff_from, eff_to) = if rev {
                            (tr.to_mask, tr.from_mask)
                        } else {
                            (tr.from_mask, tr.to_mask)
                        };
                        let still_at_dest = if eff_to == 0 {
                            (new_mask & eff_from) != eff_from
                        } else {
                            (new_mask & eff_to) == eff_to
                        };
                        if !still_at_dest {
                            st.active = None;
                        }
                    } else {
                        st.active = None;
                    }
                }
            }
            if let Some((start_time, reversed, rule_idx)) = st.active {
                if let Some(tr) = transitions.get(rule_idx) {
                    let elapsed_sec = (now - start_time).max(0.0);
                    let elapsed_ms = (elapsed_sec * 1000.0 + 1e-6).floor() as u64;
                    if elapsed_ms >= tr.total_ms {
                        st.active = None;
                    } else {
                        let mut acc = 0u64;
                        let count = tr.frames.len();
                        for step in 0..count {
                            let frame = if reversed {
                                &tr.frames[count - 1 - step]
                            } else {
                                &tr.frames[step]
                            };
                            acc = acc.saturating_add(frame.duration_ms);
                            if elapsed_ms < acc {
                                chosen_frame = Some(frame.img_key);
                                let next_wake = (start_time + acc as f64 / 1000.0).max(now + 0.001);
                                let vis_rect = b.intersect(f.clip);
                                if vis_rect.width > 0.0 && vis_rect.height > 0.0 {
                                    f.wake_at(next_wake);
                                }
                                break;
                            }
                        }
                    }
                } else {
                    st.active = None;
                }
            }
        }

        if state.animated_selector_states.len() >= 256
            && !state.animated_selector_states.contains_key(&map_key)
        {
            state.animated_selector_states.retain(|_, v| v.active.is_some());
            if state.animated_selector_states.len() >= 256 {
                state.animated_selector_states.clear();
            }
        }
        state.animated_selector_states.insert(map_key, st);
        chosen_frame
    }
    pub(super) fn is_image_src_kind(&self) -> bool {
        matches!(self.kind, Kind::Image | Kind::QuickContactBadge | Kind::Icon | Kind::Zoom)
    }
    pub(super) fn animation_list_spec(&self) -> Option<AnimationListSpec<'_>> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_ANIMATION_LIST)
            .and_then(|r| parse_animation_list(r.anchor))
    }
    pub(super) fn animated_vector_spec(&self, is_src: bool) -> Option<AnimatedVectorSpec<'_>> {
        self.relative_rules
            .iter()
            .filter(|r| r.kind == RULE_ANIMATED_VECTOR)
            .filter_map(|r| parse_animated_vector(r.anchor))
            .find(|spec| spec.is_src == is_src)
    }
    pub(super) fn has_animated_vector(&self, is_src: bool) -> bool {
        self.animated_vector_spec(is_src).is_some()
    }
    pub(super) fn text_links_spans(&self) -> Option<TextLinksSpec> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_TEXT_LINKS_SPANS)
            .map(|r| parse_text_links_spans(r.anchor))
    }
    pub(super) fn has_clickable_text_links(&self) -> bool {
        matches!(self.kind, Kind::Text | Kind::CheckedText)
            && self
                .text_links_spans()
                .is_some_and(|spec| spec.mask != 0 && spec.clickable)
    }
    pub(super) fn dynamic_drawable_for(&self, wanted: DynamicDrawableTarget) -> Option<DynamicDrawableSpec<'_>> {
        let default_target = if self.is_image_src_kind() {
            DynamicDrawableTarget::Src
        } else {
            DynamicDrawableTarget::Bg
        };
        self.relative_rules
            .iter()
            .filter(|r| r.kind == RULE_DRAWABLE_DYNAMIC)
            .filter_map(|r| parse_dynamic_drawable(r.anchor, default_target))
            .find_map(|(target, spec)| (target == wanted).then_some(spec))
    }
    pub(super) fn dynamic_drawable_bg(&self) -> Option<DynamicDrawableSpec<'_>> {
        self.dynamic_drawable_for(DynamicDrawableTarget::Bg)
    }
    pub(super) fn dynamic_drawable_src(&self) -> Option<DynamicDrawableSpec<'_>> {
        self.dynamic_drawable_for(DynamicDrawableTarget::Src)
    }
    pub(super) fn placeholder_spec(&self) -> Option<PlaceholderSpec<'_>> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_CONSTRAINT_PLACEHOLDER)
            .map(|r| parse_placeholder_spec(r.anchor))
    }
    pub(super) fn has_explicit_custom_background(&self) -> bool {
        self.background.is_some()
            || self.dynamic_drawable_bg().is_some()
            || self.has_animated_vector(false)
            || self.has_animated_selector(false)
            || self.relative_rules.iter().any(|r| {
                r.kind == 96
                    || r.kind == RULE_DRAWABLE_LAYER
                    || r.kind == RULE_STATEFUL_DRAWABLE_LAYER
                    || r.kind == RULE_BACKGROUND_IMAGE
                    || r.kind == RULE_STATEFUL_BG_IMAGE
                    || (!self.is_image_src_kind() && r.kind == RULE_ANIMATION_LIST && parse_animation_list(r.anchor).is_some())
                    || (r.kind == RULE_GRADIENT && parse_gradient(r.anchor).is_some())
            })
    }
    pub(super) fn text_style_ext(&self) -> (bool, FontFamily) {
        let Some(rule) = self.relative_rules.iter().find(|r| r.kind == RULE_TEXT_STYLE_EXT) else {
            return (false, FontFamily::Sans);
        };
        let (italic_part, family_part) = rule.anchor.split_once('|').unwrap_or((rule.anchor, "sans"));
        let italic = matches!(italic_part.trim().to_ascii_lowercase().as_str(), "1" | "true" | "italic");
        let family = match family_part.trim().to_ascii_lowercase().as_str() {
            "serif" => FontFamily::Serif,
            "mono" | "monospace" => FontFamily::Monospace,
            _ => FontFamily::Sans,
        };
        (italic, family)
    }
    pub(super) fn is_auto_mirrored(&self) -> bool {
        self.relative_rules.iter().any(|r| {
            r.kind == RULE_AUTO_MIRRORED
                && matches!(r.anchor.trim().to_ascii_lowercase().as_str(), "true" | "1")
        })
    }
    pub(super) fn view_transform(&self) -> Option<ViewTransformSpec> {
        let spec = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_VIEW_TRANSFORM)
            .and_then(|r| parse_view_transform(r.anchor))?;
        let is_identity = spec.tx_dp == 0.0
            && spec.ty_dp == 0.0
            && (spec.sx - 1.0).abs() < 1e-6
            && (spec.sy - 1.0).abs() < 1e-6
            && spec.rot_deg == 0.0;
        (!is_identity).then_some(spec)
    }
    pub(super) fn tooltip_text(&self) -> Option<&'static str> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_TOOLTIP_TEXT)
            .map(|r| r.anchor)
            .filter(|s| !s.is_empty())
    }
    pub(super) fn bottom_sheet_spec(&self) -> Option<BottomSheetSpec> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_BOTTOM_SHEET)
            .map(|r| parse_bottom_sheet(r.anchor))
    }
    pub(super) fn is_drag_handle(&self) -> bool {
        self.relative_rules.iter().any(|r| r.kind == RULE_DRAG_HANDLE)
    }
    pub(super) fn side_sheet_spec(&self) -> Option<SideSheetSpec> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_SIDE_SHEET)
            .map(|r| parse_side_sheet(r.anchor))
    }
    pub(super) fn loading_indicator_contained(&self) -> Option<bool> {
        if !matches!(self.kind, Kind::ProgressBar) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_LOADING_INDICATOR)
            .map(|r| matches!(r.anchor.trim().to_ascii_lowercase().as_str(), "1" | "true"))
    }
    pub(super) fn wavy_progress_spec(&self) -> Option<WavyProgressSpec> {
        if !matches!(self.kind, Kind::ProgressBar) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_WAVY_PROGRESS)
            .map(|r| parse_wavy_progress(r.anchor))
    }
    pub(super) fn alert_dialog_icon(&self) -> Option<&'static str> {
        if !matches!(self.kind, Kind::Linear) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_ALERT_DIALOG)
            .map(|r| r.anchor.trim())
    }
    pub(super) fn is_snackbar(&self) -> bool {
        matches!(self.kind, Kind::Linear)
            && self.relative_rules.iter().any(|r| r.kind == RULE_SNACKBAR)
    }
    pub(super) fn tab_options(&self) -> Option<XmlTabOptionsSpec> {
        if !matches!(self.kind, Kind::TabWidget) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_TAB_OPTIONS)
            .map(|r| parse_tab_options(r.anchor))
    }
    pub(super) fn slider_options(&self) -> Option<SliderOptionsSpec> {
        if !matches!(self.kind, Kind::SeekBar) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_SLIDER_OPTIONS)
            .map(|r| parse_slider_options(r.anchor))
    }
    pub(super) fn text_input_ext(&self) -> Option<TextInputExtSpec<'static>> {
        if !matches!(self.kind, Kind::Linear) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_TEXT_INPUT_EXT)
            .map(|r| parse_text_input_ext(r.anchor))
    }
    pub(super) fn chip_group_wrap_spec(&self) -> Option<ChipGroupWrapSpec> {
        if !matches!(self.kind, Kind::Linear) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_CHIP_GROUP_WRAP)
            .map(|r| parse_chip_group_wrap(r.anchor))
    }
    pub(super) fn is_floating_toolbar(&self) -> bool {
        matches!(self.kind, Kind::Linear)
            && self.relative_rules.iter().any(|r| r.kind == RULE_FLOATING_TOOLBAR)
    }
    pub(super) fn button_group_spacing(&self) -> Option<f32> {
        if !matches!(self.kind, Kind::Linear) {
            return None;
        }
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_BUTTON_GROUP)
            .map(|r| {
                r.anchor
                    .trim()
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite() && *v >= 0.0)
                    .unwrap_or(12.0)
            })
    }
    pub(super) fn is_split_button(&self) -> bool {
        matches!(self.kind, Kind::Linear)
            && self.relative_rules.iter().any(|r| r.kind == RULE_SPLIT_BUTTON)
    }
    pub(super) fn is_carousel_item(&self) -> bool {
        matches!(self.kind, Kind::Frame)
            && self.relative_rules.iter().any(|r| r.kind == RULE_CAROUSEL_ITEM)
    }
    pub(super) fn linear_extra_gap(&self, state: &LayoutState) -> f32 {
        if self.is_split_button() {
            state.dp(2.0)
        } else if let Some(sp) = self.button_group_spacing() {
            state.dp(sp)
        } else {
            0.0
        }
    }
    pub(super) fn alert_dialog_top_icon_extra(&self, state: &LayoutState) -> f32 {
        if let Some(icon_name) = self.alert_dialog_icon()
            && !icon_name.is_empty()
            && (state.images.contains_key(icon_name) || parse_material_icon(icon_name).is_some())
        {
            state.dp(40.0)
        } else {
            0.0
        }
    }
    pub(super) fn scale_anim(&self, kind: u16) -> Option<ScaleAnimationSpec> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == kind)
            .and_then(|r| parse_scale_animation(r.anchor))
    }
    pub(super) fn rotate_anim(&self, kind: u16) -> Option<RotateAnimationSpec> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == kind)
            .and_then(|r| parse_rotate_animation(r.anchor))
    }
    pub(super) fn elevation_anim(&self, kind: u16) -> Option<ElevationAnimationSpec> {
        self.relative_rules
            .iter()
            .find(|r| r.kind == kind)
            .and_then(|r| parse_elevation_animation(r.anchor))
    }
    pub(super) fn is_sheet_hidden(&self, state: &LayoutState, id: u64) -> bool {
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        let active_motion = state
            .sheet_motion
            .get(&key)
            .is_some_and(|m| m.settle.is_some() || m.dragging);
        if let Some(bs) = self.bottom_sheet_spec() {
            return bs.hideable
                && state.sheet_state.get(&key) == Some(&SHEET_STATE_HIDDEN)
                && !active_motion;
        }
        if self.side_sheet_spec().is_some() {
            return state.sheet_state.get(&key) == Some(&SHEET_STATE_HIDDEN) && !active_motion;
        }
        false
    }
    pub(super) fn resolve_bottom_sheet_height(
        &self,
        spec: BottomSheetSpec,
        parent_h: f32,
        max_h: f32,
        natural_h: f32,
        state: &LayoutState,
        id: u64,
    ) -> f32 {
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        if let Some(cur_h) = state.sheet_motion.get(&key).and_then(|m| m.current_h) {
            return cur_h.clamp(0.0, max_h);
        }
        let st = state.sheet_state.get(&key).copied();
        let peek_px = if spec.peek_h_dp > 0.0 {
            Some(state.dp(spec.peek_h_dp))
        } else {
            None
        };
        let target = match st {
            Some(SHEET_STATE_HIDDEN) if spec.hideable => 0.0,
            Some(SHEET_STATE_EXPANDED) => {
                if spec.fit_to_contents {
                    natural_h
                } else {
                    max_h
                }
            }
            Some(SHEET_STATE_HALF_EXPANDED) => parent_h * spec.half_ratio,
            Some(SHEET_STATE_COLLAPSED) => peek_px.unwrap_or(natural_h),
            _ => {
                if !spec.skip_collapsed {
                    peek_px.unwrap_or(natural_h)
                } else if spec.fit_to_contents {
                    natural_h
                } else {
                    max_h
                }
            }
        };
        target.clamp(0.0, max_h)
    }
    pub(super) fn chip_group_wrap_rects(
        &self,
        inner: Rect,
        spec: ChipGroupWrapSpec,
        state: &LayoutState,
        metrics: TextMetrics,
        id: u64,
    ) -> (Vec<Option<Rect>>, f32, f32) {
        let h_gap = state.dp(spec.h_spacing_dp.max(0.0));
        let v_gap = state.dp(spec.v_spacing_dp.max(0.0));
        let mut rects: Vec<Option<Rect>> = vec![None; self.children.len()];
        let mut rows: Vec<(f32, f32, Vec<(usize, f32, f32, f32, [f32; 4])>)> = Vec::new();
        let mut cur_x = 0.0_f32;
        let mut cur_h = 0.0_f32;
        let mut cur_row: Vec<(usize, f32, f32, f32, [f32; 4])> = Vec::new();
        let mut max_row_w = 0.0_f32;
        for (i, n) in self.children.iter().enumerate() {
            if n.effective_visibility(state) == Visibility::Gone {
                continue;
            }
            let cid = child_view_id(id, i, n);
            let m = n.resolved_margin(state);
            let avail_w = (inner.width - m[0] - m[2]).max(0.0);
            let cw = match n.width {
                Size::Dp(w) => state.dp(w).min(avail_w.max(state.dp(w))),
                Size::Match => avail_w,
                Size::Wrap => n.layout_intrinsic(true, state, avail_w, metrics, cid).min(avail_w),
            };
            let ch = match n.height {
                Size::Dp(h) => state.dp(h),
                Size::Match => n.layout_intrinsic(false, state, cw, metrics, cid),
                Size::Wrap => n.layout_intrinsic(false, state, cw, metrics, cid),
            };
            let total_w = cw + m[0] + m[2];
            let total_h = ch + m[1] + m[3];
            if !spec.single_line && !cur_row.is_empty() && cur_x + h_gap + total_w > inner.width + 0.01 {
                max_row_w = max_row_w.max(cur_x);
                rows.push((cur_x, cur_h, std::mem::take(&mut cur_row)));
                cur_x = 0.0;
                cur_h = 0.0;
            }
            let x_in_row = if cur_row.is_empty() { 0.0 } else { cur_x + h_gap };
            cur_x = x_in_row + total_w;
            cur_h = cur_h.max(total_h);
            cur_row.push((i, x_in_row, cw, ch, m));
        }
        if !cur_row.is_empty() {
            max_row_w = max_row_w.max(cur_x);
            rows.push((cur_x, cur_h, cur_row));
        }
        let mut cursor_y = 0.0_f32;
        for (row_idx, (_rw, row_h, items)) in rows.into_iter().enumerate() {
            if row_idx > 0 {
                cursor_y += v_gap;
            }
            for (i, x_in_row, cw, ch, m) in items {
                let total_h = ch + m[1] + m[3];
                let rx = if state.rtl {
                    inner.x + inner.width - (x_in_row + m[0] + cw)
                } else {
                    inner.x + x_in_row + m[0]
                };
                let ry = inner.y + cursor_y + (row_h - total_h) * 0.5 + m[1];
                rects[i] = Some(Rect { x: rx, y: ry, width: cw, height: ch });
            }
            cursor_y += row_h;
        }
        (rects, max_row_w, cursor_y)
    }
    pub(super) fn resolved_bg_image<'a>(
        &'a self,
        state: &LayoutState,
        key: &str,
        response: Response,
    ) -> Option<(&'a str, Option<NinePatchBands>)> {
        let bits = self.live_state_mask(state, key, response);
        for rule in self.relative_rules.iter().filter(|r| r.kind == RULE_STATEFUL_BG_IMAGE) {
            if let Some((required, forbidden, img_key, stretch_str)) = parse_stateful_image_rule(rule.anchor) {
                if (bits & required) == required && (bits & forbidden) == 0 {
                    if img_key.is_empty() {
                        return None;
                    }
                    return Some((img_key, parse_nine_patch_stretch(stretch_str)));
                }
            }
        }
        if let Some((img_key, stretch_str)) = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_BACKGROUND_IMAGE)
            .and_then(|r| parse_bg_image_rule(r.anchor))
        {
            return Some((img_key, parse_nine_patch_stretch(stretch_str)));
        }
        None
    }
    pub(super) fn resolved_image_src<'a>(
        &'a self,
        state: &'a LayoutState,
        key: &str,
        response: Response,
    ) -> (&'a str, Option<NinePatchBands>, bool) {
        if let Some(override_src) = state.image_sources.get(key) {
            let stretch = if override_src.as_str() == self.image_src {
                self.relative_rules
                    .iter()
                    .find(|r| r.kind == RULE_IMAGE_NINE_PATCH)
                    .and_then(|r| parse_nine_patch_stretch(r.anchor))
            } else {
                None
            };
            return (override_src.as_str(), stretch, true);
        }
        let bits = self.live_state_mask(state, key, response);
        for rule in self.relative_rules.iter().filter(|r| r.kind == RULE_STATEFUL_IMAGE_SRC) {
            if let Some((required, forbidden, img_key, stretch_str)) = parse_stateful_image_rule(rule.anchor) {
                if (bits & required) == required && (bits & forbidden) == 0 {
                    return (img_key, parse_nine_patch_stretch(stretch_str), false);
                }
            }
        }
        let stretch = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_IMAGE_NINE_PATCH)
            .and_then(|r| parse_nine_patch_stretch(r.anchor));
        (self.image_src, stretch, false)
    }
    pub(super) fn resolved_intrinsic_image_src<'a>(&'a self, state: &'a LayoutState, key: &str) -> &'a str {
        if let Some(override_src) = state.image_sources.get(key) {
            return override_src.as_str();
        }
        if let Some(spec) = self.dynamic_drawable_src() {
            return match spec {
                DynamicDrawableSpec::Level(items) => {
                    let level = state.drawable_level.get(key).copied().unwrap_or(0);
                    select_level_drawable_key(&items, level)
                }
                DynamicDrawableSpec::Clip { img_key, .. }
                | DynamicDrawableSpec::Rotate { img_key, .. }
                | DynamicDrawableSpec::Scale { img_key, .. } => img_key,
                DynamicDrawableSpec::Transition { img_key_0, img_key_1, .. } => {
                    let active = state.drawable_transition_active.get(key).copied().unwrap_or(false);
                    if active && !img_key_1.is_empty() {
                        img_key_1
                    } else if !img_key_0.is_empty() {
                        img_key_0
                    } else {
                        img_key_1
                    }
                }
            };
        }
        if let Some(spec) = self.animation_list_spec() {
            return spec.frames[0].img_key;
        }
        if let Some(spec) = self.animated_vector_spec(true) {
            return if !spec.repeat && state.frame_reduced_motion {
                spec.frames[spec.frames.len() - 1]
            } else {
                spec.frames[0]
            };
        }
        if !self.image_src.is_empty() {
            return self.image_src;
        }
        self.relative_rules
            .iter()
            .filter(|r| r.kind == RULE_STATEFUL_IMAGE_SRC)
            .filter_map(|r| parse_stateful_image_rule(r.anchor))
            .find_map(|(_, _, img_key, _)| (!img_key.is_empty()).then_some(img_key))
            .or_else(|| {
                self.relative_rules
                    .iter()
                    .filter(|r| r.kind == RULE_ANIMATED_SELECTOR)
                    .filter_map(|r| parse_animated_selector_transition(r.anchor))
                    .find_map(|t| (t.is_src && !t.frames.is_empty()).then_some(t.frames[0].img_key))
            })
            .unwrap_or("")
    }
    pub(super) fn paint_constraint_layer_chrome(&self, f: &mut Frame<'_>, b: Rect, state: &mut LayoutState) {
        if b.width <= 0.0 || b.height <= 0.0 {
            return;
        }
        let background_tint = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_BACKGROUND_TINT)
            .and_then(|r| {
                let mut parts = r.anchor.split(',').filter_map(|p| p.trim().parse::<u8>().ok());
                Some(Color(parts.next()?, parts.next()?, parts.next()?, parts.next()?))
            });
        let scaled_radius = if self.radius < 0.0 { self.radius } else { state.dp(self.radius) };
        let corner_radii = self.resolved_corner_radii(state);
        let corner_family = self.resolved_corner_family();
        let gradient = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_GRADIENT)
            .and_then(|r| parse_gradient(r.anchor));
        let explicit_elevation = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_ELEVATION)
            .and_then(|r| r.anchor.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v > 0.0);
        if let Some(dp) = explicit_elevation {
            let elev_radius = if self.radius < 0.0 {
                b.width.min(b.height) * 0.5
            } else if scaled_radius > 0.0 {
                scaled_radius
            } else if let Some([tl, tr, br, bl]) = corner_radii {
                tl.max(tr).max(br).max(bl)
            } else {
                0.0
            };
            paint_elevation_level(f, b, elev_radius, elevation_level_from_dp(dp), state.effective_density());
        }
        let is_oval = self.radius < 0.0 && corner_radii.is_none();
        if let Some(spec) = gradient {
            let radii = corner_radii.unwrap_or([scaled_radius.max(0.0); 4]);
            paint_gradient(f, state, b, spec, is_oval, radii, corner_family, background_tint);
        } else if let Some(color) = background_tint.or(self.background) {
            paint_shape_fill_with_family(f, state, b, is_oval, corner_radii, scaled_radius, corner_family, color);
        }
        if let Some((img_key, stretch)) = self
            .relative_rules
            .iter()
            .find(|r| r.kind == RULE_BACKGROUND_IMAGE)
            .and_then(|r| parse_bg_image_rule(r.anchor))
            .map(|(k, s)| (k, parse_nine_patch_stretch(s)))
            && let Some(img) = state.images.get(img_key).cloned()
        {
            let bg_img_start = f.commands.len();
            let density = state.image_scales.get(img_key).copied().unwrap_or(1.0) * state.effective_density();
            if let Some(ref bands) = stretch {
                paint_nine_patch_image(f, state, b, &img, bands, background_tint, 1, 255, density);
            } else {
                let tinted = tint_image(state, &img, background_tint, 1);
                f.commands.push(DrawCommand::Image {
                    bounds: b,
                    uv: [0.0, 0.0, 1.0, 1.0],
                    radius: 0.0,
                    color: Color(255, 255, 255, 255),
                    image: tinted,
                });
            }
            let outline_radii = corner_radii.unwrap_or([scaled_radius.max(0.0); 4]);
            clip_commands_to_rounded_outline(f, bg_img_start, b, is_oval, outline_radii);
        }
        let d = state.effective_density();
        for rule in self.relative_rules.iter().filter(|r| r.kind == RULE_DRAWABLE_LAYER) {
            if let Some(layer) = parse_drawable_layer(rule.anchor) {
                paint_drawable_layer(f, b, layer, background_tint, d);
            }
        }
        if let Some(color) = self.background_stroke {
            let sw = state.dp(self.background_stroke_width);
            paint_shape_outline_with_family(f, b, is_oval, corner_radii, scaled_radius, corner_family, sw, color);
        }
    }
    pub(super) fn paint_stateful_drawable_layers(
        &self,
        f: &mut Frame<'_>,
        b: Rect,
        state: &LayoutState,
        key: &str,
        response: Response,
        tint: Option<Color>,
    ) {
        let bits = self.live_state_mask(state, key, response);
        let specs: Vec<StatefulDrawableLayerSpec> = self
            .relative_rules
            .iter()
            .filter(|r| r.kind == RULE_STATEFUL_DRAWABLE_LAYER)
            .filter_map(|r| parse_stateful_drawable_layer(r.anchor))
            .collect();
        if specs.is_empty() {
            return;
        }
        if let Some(first_idx) = specs
            .iter()
            .position(|s| (bits & s.required) == s.required && (bits & s.forbidden) == 0)
        {
            let (req, forbid) = (specs[first_idx].required, specs[first_idx].forbidden);
            let d = state.effective_density();
            for spec in &specs[first_idx..] {
                if spec.required != req || spec.forbidden != forbid {
                    break;
                }
                paint_drawable_layer(f, b, spec.layer, tint, d);
            }
        }
    }
    pub(super) fn text_input_extra_height(&self, spec: &TextInputBoxSpec<'_>, state: &LayoutState, id: u64) -> (f32, f32) {
        let key = if self.id.is_empty() { format!("#{id}") } else { self.id.to_owned() };
        let effective_hint = if !spec.hint.is_empty() {
            spec.hint
        } else if !self.hint.is_empty() {
            self.hint
        } else {
            find_text_input_edit(self, id).map_or("", |(n, _, _)| n.hint)
        };
        let label_size = state.sp(12.0);
        let label_h = label_size.max(state.dp(12.0)) + state.dp(2.0);
        let top_inset = if spec.mode == TextInputBoxMode::Outline && !effective_hint.is_empty() {
            label_h * 0.5
        } else {
            0.0
        };
        let has_error = spec.error_enabled
            && state.text.get(&format!("{key}:error")).is_some_and(|s| !s.is_empty());
        let has_counter = self.text_input_ext().is_some_and(|e| e.counter_enabled);
        let supporting_h = if has_error || !spec.helper.is_empty() || has_counter {
            state.dp(4.0) + (label_size * 1.3333).max(state.dp(16.0))
        } else {
            0.0
        };
        (top_inset, supporting_h)
    }
    pub(super) fn compute_coordinator_rects(
        &self,
        inner: Rect,
        state: &LayoutState,
        metrics: TextMetrics,
        id: u64,
    ) -> Vec<Option<Rect>> {
        let measure_frame_child = |i: usize, n: &Node, max_w: f32, max_h: f32| -> (f32, f32) {
            let mut w = max_w;
            let mut h = max_h;
            if let Size::Dp(dw) = n.width {
                w = state.dp(dw);
            }
            if let Size::Dp(dh) = n.height {
                h = state.dp(dh);
            }
            if matches!(n.kind, Kind::VideoView) {
                let name = if n.id.is_empty() { format!("#{}", child_view_id(id, i, n)) } else { n.id.to_owned() };
                let size = state
                    .video_dimensions
                    .get(&name)
                    .copied()
                    .or_else(|| state.video_frames.get(&name).map(|frame| (frame.width(), frame.height())))
                    .unwrap_or((0, 0));
                let spec = |dimension: Size, available: f32| {
                    if matches!(dimension, Size::Wrap) {
                        VideoMeasureSpec::AtMost(available.max(0.0) as u32)
                    } else {
                        VideoMeasureSpec::Exactly(available.max(0.0) as u32)
                    }
                };
                let (vw, vh) = measure_video_view(size, spec(n.width, w), spec(n.height, h));
                return (vw as f32, vh as f32);
            }
            let cid = child_view_id(id, i, n);
            if let Some(ch) = n.collapsing_toolbar_height(state, cid) {
                h = ch.min(max_h);
            } else if matches!(n.kind, Kind::Linear)
                && let Size::Dp(dp_h) = n.height
                && let Some((ci, cc)) = n
                    .children
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.collapsing_toolbar_spec().is_some())
            {
                let cc_id = child_view_id(cid, ci, cc);
                if let (Some(spec), Some(ch)) = (cc.collapsing_toolbar_spec(), cc.collapsing_toolbar_height(state, cc_id)) {
                    let delta = (state.dp(spec.expanded_h_dp) - ch).max(0.0);
                    h = (state.dp(dp_h) - delta).max(state.dp(64.0).min(state.dp(dp_h))).min(max_h);
                }
            } else if matches!(n.kind, Kind::Linear)
                && !n.horizontal
                && let Size::Dp(dp_h) = n.height
                && n.children.iter().any(|c| c.app_bar_scroll_flags().is_some())
            {
                let pad = n.resolved_padding(state);
                let inner_w = (w - pad[0] - pad[2]).max(0.0);
                let delta = n.app_bar_children_collapse_total(state, inner_w, metrics, cid);
                h = (state.dp(dp_h) - delta).max(0.0).min(max_h);
            }
            if matches!(n.width, Size::Wrap) {
                w = n.layout_intrinsic(true, state, w, metrics, cid).min(w);
            }
            if matches!(n.height, Size::Wrap) && n.collapsing_toolbar_spec().is_none() {
                h = n.layout_intrinsic(false, state, w, metrics, cid).min(h);
            }
            (w, h)
        };
        let place_gravity = |n: &Node, area: Rect, margin: [f32; 4], w: f32, h: f32| -> Rect {
            let right = match n.layout_gravity[0] {
                0 => state.rtl,
                2 => !state.rtl,
                4 => true,
                _ => false,
            };
            let x = if n.layout_gravity[0] == 1 {
                area.x + (area.width - w) * 0.5 + margin[0] - margin[2]
            } else if right {
                area.x + area.width - w - margin[2]
            } else {
                area.x + margin[0]
            };
            let y = match n.layout_gravity[1] {
                1 => area.y + (area.height - h) * 0.5 + margin[1] - margin[3],
                2 => area.y + area.height - h - margin[3],
                _ => area.y + margin[1],
            };
            Rect { x, y, width: w, height: h }
        };
        let mut rects: Vec<Option<Rect>> = vec![None; self.children.len()];
        let mut content_inner = inner;
        for (i, n) in self.children.iter().enumerate() {
            if n.effective_visibility(state) == Visibility::Gone {
                continue;
            }
            let cid = child_view_id(id, i, n);
            if let Some(ss) = n.side_sheet_spec()
                && ss.coplanar
                && !n.is_sheet_hidden(state, cid)
            {
                let margin = n.resolved_margin(state);
                let default_w = state.dp(320.0).min((inner.width * 0.8).max(0.0));
                let sw = match n.width {
                    Size::Dp(w) => state.dp(w).min(inner.width),
                    Size::Wrap => {
                        let iw = n.layout_intrinsic(true, state, inner.width, metrics, cid);
                        (if iw > 0.0 { iw } else { default_w }).min(inner.width)
                    }
                    Size::Match => default_w,
                };
                let phys_right = if ss.edge_start { state.rtl } else { !state.rtl };
                let total_sw = (sw + margin[0] + margin[2]).min(content_inner.width);
                if phys_right {
                    content_inner.width = (content_inner.width - total_sw).max(0.0);
                } else {
                    content_inner.x += total_sw;
                    content_inner.width = (content_inner.width - total_sw).max(0.0);
                }
            }
        }
        // Pass 1: layout all non-anchored visible children (including scroll behavior below appbar, bottom/side sheets).
        for (i, n) in self.children.iter().enumerate() {
            if n.effective_visibility(state) == Visibility::Gone
                || n.relative_rules.iter().any(|r| r.kind == RULE_COORDINATOR_ANCHOR)
            {
                continue;
            }
            let cid = child_view_id(id, i, n);
            if n.is_sheet_hidden(state, cid) {
                continue;
            }
            let margin = n.resolved_margin(state);
            if let Some(bs) = n.bottom_sheet_spec() {
                let max_w = (content_inner.width - margin[0] - margin[2]).max(0.0);
                let max_h = (content_inner.height - margin[1] - margin[3]).max(0.0);
                let (w, natural_h) = measure_frame_child(i, n, max_w, max_h);
                let h = n.resolve_bottom_sheet_height(bs, content_inner.height, max_h, natural_h, state, cid);
                let x = if n.layout_gravity_explicit {
                    place_gravity(n, content_inner, margin, w, h).x
                } else if w < max_w {
                    content_inner.x + (content_inner.width - w) * 0.5 + margin[0] - margin[2]
                } else {
                    content_inner.x + margin[0]
                };
                let y = content_inner.y + content_inner.height - h - margin[3];
                rects[i] = Some(Rect { x, y, width: w, height: h });
                continue;
            }
            if let Some(ss) = n.side_sheet_spec() {
                let default_w = state.dp(320.0).min((inner.width * 0.8).max(0.0));
                let sw = match n.width {
                    Size::Dp(w) => state.dp(w).min(inner.width),
                    Size::Wrap => {
                        let iw = n.layout_intrinsic(true, state, inner.width, metrics, cid);
                        (if iw > 0.0 { iw } else { default_w }).min(inner.width)
                    }
                    Size::Match => default_w,
                };
                let sh = match n.height {
                    Size::Dp(h) => state.dp(h).min(inner.height),
                    Size::Wrap | Size::Match => (inner.height - margin[1] - margin[3]).max(0.0),
                };
                let phys_right = if ss.edge_start { state.rtl } else { !state.rtl };
                let x = if phys_right {
                    inner.x + inner.width - sw - margin[2]
                } else {
                    inner.x + margin[0]
                };
                let y = inner.y + margin[1];
                rects[i] = Some(Rect { x, y, width: sw, height: sh });
                continue;
            }
            let has_scroll = n.relative_rules.iter().any(|r| r.kind == RULE_COORDINATOR_SCROLL_BEHAVIOR);
            let area = if has_scroll {
                let appbar_h = (0..i)
                    .find_map(|j| {
                        let sib = &self.children[j];
                        if sib.effective_visibility(state) == Visibility::Gone
                            || (!matches!(sib.kind, Kind::Linear | Kind::Toolbar)
                                && sib.collapsing_toolbar_spec().is_none())
                            || sib.bottom_app_bar_spec().is_some()
                            || sib.layout_gravity[1] == 2
                            || sib.bottom_sheet_spec().is_some()
                            || sib.side_sheet_spec().is_some()
                            || sib.relative_rules.iter().any(|r| {
                                matches!(r.kind, RULE_COORDINATOR_SCROLL_BEHAVIOR | RULE_COORDINATOR_ANCHOR)
                            })
                        {
                            return None;
                        }
                        let ab_rect = rects[j]?;
                        let ab_margin = sib.resolved_margin(state);
                        Some((ab_rect.y + ab_rect.height + ab_margin[3] - content_inner.y).max(0.0))
                    })
                    .unwrap_or(0.0);
                Rect {
                    x: content_inner.x,
                    y: content_inner.y + appbar_h,
                    width: content_inner.width,
                    height: (content_inner.height - appbar_h).max(0.0),
                }
            } else {
                content_inner
            };
            let max_w = (area.width - margin[0] - margin[2]).max(0.0);
            let max_h = (area.height - margin[1] - margin[3]).max(0.0);
            let (w, h) = measure_frame_child(i, n, max_w, max_h);
            let mut placed = place_gravity(n, area, margin, w, h);
            if n.bottom_app_bar_spec().is_some_and(|s| s.hide_on_scroll) {
                let bab_key = if n.id.is_empty() { format!("#{cid}") } else { n.id.to_owned() };
                let scroll_px = n.app_bar_scroll_px(state, cid);
                let (hs, _) = eval_bottom_app_bar_hide(&bab_key, scroll_px, state);
                placed.y += placed.height * hs.progress;
            }
            rects[i] = Some(placed);
        }
        // Pass 2: layout anchored children centered on their target sibling's edge/corner.
        for (i, n) in self.children.iter().enumerate() {
            if n.effective_visibility(state) == Visibility::Gone {
                continue;
            }
            let Some(anchor_rule) = n.relative_rules.iter().find(|r| r.kind == RULE_COORDINATOR_ANCHOR) else {
                continue;
            };
            let margin = n.resolved_margin(state);
            let max_w = (inner.width - margin[0] - margin[2]).max(0.0);
            let max_h = (inner.height - margin[1] - margin[3]).max(0.0);
            let (w, h) = measure_frame_child(i, n, max_w, max_h);
            let target_id = anchor_rule
                .anchor
                .split_once(':')
                .map_or(anchor_rule.anchor, |(lhs, _)| lhs)
                .trim();
            let target_hit = if target_id.is_empty() {
                None
            } else {
                self.children
                    .iter()
                    .enumerate()
                    .find(|(j, sib)| *j != i && !sib.id.is_empty() && sib.id == target_id)
                    .and_then(|(j, sib)| rects[j].map(|r| (sib, r)))
            };
            rects[i] = Some(if let Some((target_sib, target)) = target_hit {
                if let Some(bab) = target_sib.bottom_app_bar_spec() {
                    let (fab_cx, fab_cy) = bottom_app_bar_fab_center(target, bab, w, state);
                    let min_x = inner.x + margin[0];
                    let max_x = (inner.x + inner.width - w - margin[2]).max(min_x);
                    let x = (fab_cx - w * 0.5).clamp(min_x, max_x);
                    let y = (fab_cy - h * 0.5).max(inner.y + margin[1]);
                    Rect { x, y, width: w, height: h }
                } else {
                    resolve_coordinator_anchor_rect(anchor_rule.anchor, target, (w, h), inner, margin, state.rtl)
                }
            } else {
                place_gravity(n, inner, margin, w, h)
            });
        }
        // Pass 3: Snackbar-aware FAB dodge — shift bottom-aligned FAB / ExtendedFAB siblings
        // upward by the visible bottom-docked Snackbar's height + margin + 8dp gap so they never overlap.
        let mut snackbar_dodge: Option<(f32, f32)> = None;
        for (si, sn) in self.children.iter().enumerate() {
            if sn.effective_visibility(state) != Visibility::Visible || !sn.is_snackbar() {
                continue;
            }
            let Some(sr) = rects[si] else { continue };
            let docked_bottom = sn.layout_gravity[1] == 2
                || sr.y + sr.height >= inner.y + inner.height * 0.5;
            if !docked_bottom {
                continue;
            }
            let sm = sn.resolved_margin(state);
            let shift = sr.height + sm[1] + sm[3] + state.dp(8.0);
            snackbar_dodge = Some(match snackbar_dodge {
                Some((prev_top, prev_shift)) => (prev_top.min(sr.y), prev_shift.max(shift)),
                None => (sr.y, shift),
            });
        }
        if let Some((snack_top_y, shift)) = snackbar_dodge {
            let gap = state.dp(8.0);
            let is_m3 = matches!(
                state.theme_style,
                AndroidTheme::Material3Light | AndroidTheme::Material3Dark
            );
            for (i, n) in self.children.iter().enumerate() {
                if n.effective_visibility(state) == Visibility::Gone || n.is_snackbar() {
                    continue;
                }
                let elev_dp = n
                    .relative_rules
                    .iter()
                    .find(|r| r.kind == RULE_ELEVATION)
                    .and_then(|r| r.anchor.trim().parse::<f32>().ok())
                    .filter(|v| v.is_finite())
                    .unwrap_or(0.0);
                let is_fab = n.extended_fab_extended().is_some()
                    || (matches!(n.kind, Kind::Button | Kind::Icon)
                        && n.layout_gravity[1] == 2
                        && (elev_dp >= 4.0
                            || (matches!(n.kind, Kind::Icon) && n.radius < 0.0 && is_m3)));
                if !is_fab {
                    continue;
                }
                let is_bottom_aligned = n.layout_gravity[1] == 2
                    || rects[i].is_some_and(|r| r.y + r.height >= inner.y + inner.height * 0.5);
                if !is_bottom_aligned {
                    continue;
                }
                if let Some(ref mut r) = rects[i] {
                    let shifted_y = (r.y - shift).min(snack_top_y - gap - r.height);
                    r.y = shifted_y.max(inner.y);
                }
            }
        }
        rects
    }
    pub(super) fn resolved_text_alignment(&self, rtl: bool) -> Option<u8> {
        let rule = self.relative_rules.iter().find(|r| r.kind == RULE_TEXT_ALIGNMENT)?;
        match rule.anchor.trim() {
            "start" | "textStart" | "viewStart" => Some(if rtl { 2 } else { 0 }),
            "center" | "1" => Some(1),
            "end" | "textEnd" | "viewEnd" => Some(if rtl { 0 } else { 2 }),
            "left" | "0" => Some(0),
            "right" | "2" => Some(2),
            _ => None,
        }
    }
    pub(super) fn resolved_corner_radii(&self, state: &LayoutState) -> Option<[f32; 4]> {
        let rule = self.relative_rules.iter().find(|r| r.kind == RULE_CORNER_RADII)?;
        let d = state.effective_density();
        parse_corner_radii(rule.anchor).map(|r| r.map(|v| v * d))
    }
    pub(super) fn resolved_corner_family(&self) -> [bool; 4] {
        self.relative_rules
            .iter()
            .find(|r| r.kind == RULE_CORNER_FAMILY)
            .and_then(|r| parse_corner_family(r.anchor))
            .unwrap_or([false; 4])
    }
    pub(super) fn resolved_state_radius(&self, state: &LayoutState) -> f32 {
        if self.radius > 0.0 {
            state.dp(self.radius)
        } else if let Some([tl, tr, br, bl]) = self.resolved_corner_radii(state) {
            (tl + tr + br + bl) * 0.25
        } else {
            0.0
        }
    }
    pub(super) fn resolved_ripple(&self) -> Option<(Color, bool)> {
        let rule = self.relative_rules.iter().find(|r| r.kind == RULE_RIPPLE_COLOR)?;
        let mut parts = rule.anchor.split('|');
        let color_str = parts.next()?.trim();
        let mut rgba = color_str.split(',').map(|p| p.trim().parse::<u8>().ok());
        let color = Color(rgba.next()??, rgba.next()??, rgba.next()??, rgba.next()??);
        if rgba.next().is_some() {
            return None;
        }
        let unbounded = parts.next().is_some_and(|m| m.trim().eq_ignore_ascii_case("unbounded"));
        Some((color, unbounded))
    }
    pub(super) fn paint_state_layer(
        &self,
        f: &mut Frame<'_>,
        response: Response,
        default_radius: f32,
        corner_radii: Option<[f32; 4]>,
        default_color: Color,
    ) {
        let (color, unbounded) = match self.resolved_ripple() {
            Some((c, u)) => (c, u),
            None => (default_color, false),
        };
        if color.3 == 0 {
            return;
        }
        if unbounded {
            let b = response.bounds;
            let side = b.width.hypot(b.height);
            let ripple_bounds = Rect {
                x: b.x + (b.width - side) * 0.5,
                y: b.y + (b.height - side) * 0.5,
                width: side,
                height: side,
            };
            f.state_layer(Response { bounds: ripple_bounds, ..response }, side * 0.5, color);
        } else if let Some(radii) = corner_radii {
            let r = normalize_corner_radii(response.bounds, radii);
            if (r[0] - r[1]).abs() < 0.01 && (r[1] - r[2]).abs() < 0.01 && (r[2] - r[3]).abs() < 0.01 {
                f.state_layer(response, r[0], color);
            } else {
                let max_r = r.into_iter().fold(0.0_f32, f32::max);
                let start = f.commands.len();
                f.state_layer(response, max_r, color);
                if f.commands.len() > start {
                    let emitted: Vec<DrawCommand> = f.commands.drain(start..).collect();
                    for cmd in emitted {
                        match cmd {
                            DrawCommand::RoundedRect { bounds: cb, color: c, .. } => {
                                paint_rounded_rect_radii(f, cb, r, c);
                            }
                            DrawCommand::Ripple {
                                bounds: rb,
                                center,
                                radius: ink_r,
                                color: c,
                                soft,
                                ..
                            } => {
                                paint_ripple_radii(f, rb, r, center, ink_r, c, soft);
                            }
                            other => f.commands.push(other),
                        }
                    }
                }
            }
        } else {
            f.state_layer(response, default_radius, color);
        }
    }
    pub(super) fn resolved_background(&self,state:&LayoutState,key:&str,response:Response)->Option<Color> {
        let bits=u8::from(response.enabled) | (u8::from(response.pressed)<<1) | (u8::from(response.focused)<<2)
            | (u8::from(state.view_selected.get(key).copied().unwrap_or(false))<<3) | (u8::from(state.view_activated.get(key).copied().unwrap_or(false))<<4)
            | (u8::from(response.hovered)<<5) | (u8::from(matches!(self.kind,Kind::CheckedText|Kind::CheckBox|Kind::RadioButton|Kind::Switch|Kind::ToggleButton) && state.checked.get(key).copied().unwrap_or(self.checked))<<6);
        let mut first=None;let mut fallback=None;
        for rule in self.relative_rules.iter().filter(|rule|rule.kind==96) {
            let mut p=rule.anchor.split(',');
            let parsed=(||Some((p.next()?.parse::<u8>().ok()?,p.next()?.parse::<u8>().ok()?,Color(p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?))))();
            if let Some((required,forbidden,color))=parsed {
                if first.is_none(){first=Some(color);}if required==0 && forbidden==0 {fallback=Some(color);}
                if bits&required==required && bits&forbidden==0 {return Some(color);}
            }
        }
        fallback.or(first).or(self.background)
    }
    pub(super) fn resolved_progress_tint(&self, kind:u16, state:&LayoutState, key:&str, response:Response)->Option<Color> {
        self.resolved_tint_with_checked(kind,state,key,response,None)
    }
    pub(super) fn check_mark_width(&self, state: &LayoutState)->f32 {
        let mut custom=false;let mut width=0.0_f32;
        for rule in self.relative_rules.iter().filter(|r|r.kind==164) {
            custom=true;
            if let Some(w)=rule.anchor.split(',').nth(2).and_then(|s|s.parse::<f32>().ok()) {width=width.max(w);}
        }
        state.dp(if custom {if width>0.0 {width+8.0}else{0.0}} else {32.0})
    }
    pub(super) fn compound_anim_spec(&self, slot: CompoundAnimSlot) -> Option<AnimationListSpec<'static>> {
        self.relative_rules
            .iter()
            .rev()
            .filter(|r| r.kind == RULE_COMPOUND_ANIMATION_LIST)
            .find_map(|r| {
                let (s, spec) = parse_compound_animation_list(r.anchor)?;
                (s == slot).then_some(spec)
            })
    }
    pub(super) fn compound_anim_slots(&self, rtl: bool) -> [Option<CompoundAnimSlot>; 4] {
        let has_start = self.compound_anim_spec(CompoundAnimSlot::Start).is_some();
        let has_end = self.compound_anim_spec(CompoundAnimSlot::End).is_some();
        let (left, right) = if has_start || has_end {
            if rtl {
                (
                    has_end.then_some(CompoundAnimSlot::End),
                    has_start.then_some(CompoundAnimSlot::Start),
                )
            } else {
                (
                    has_start.then_some(CompoundAnimSlot::Start),
                    has_end.then_some(CompoundAnimSlot::End),
                )
            }
        } else {
            (
                self.compound_anim_spec(CompoundAnimSlot::Left).is_some().then_some(CompoundAnimSlot::Left),
                self.compound_anim_spec(CompoundAnimSlot::Right).is_some().then_some(CompoundAnimSlot::Right),
            )
        };
        let top = self.compound_anim_spec(CompoundAnimSlot::Top).is_some().then_some(CompoundAnimSlot::Top);
        let bottom = self.compound_anim_spec(CompoundAnimSlot::Bottom).is_some().then_some(CompoundAnimSlot::Bottom);
        [left, top, right, bottom]
    }
    pub(super) fn compound_anim_intrinsic_size(&self, slot: CompoundAnimSlot, state: &LayoutState) -> Option<(f32, f32)> {
        let spec = self.compound_anim_spec(slot)?;
        let first_key = spec.frames.first()?.img_key;
        let img = state.images.get(first_key)?;
        let scale = state.image_scales.get(first_key).copied().unwrap_or(1.0) * state.effective_density();
        Some((img.width() as f32 * scale, img.height() as f32 * scale))
    }
    pub(super) fn compound_drawables(&self, state: &LayoutState, rtl:bool)->[Option<(f32,f32,f32,Color)>;4] {
        let d=state.effective_density();
        let get=|kind|self.relative_rules.iter().find(|r|r.kind==kind).map(|rule|{
            let mut p=rule.anchor.split(',');
            (||Some((p.next()?.parse::<f32>().ok()?,p.next()?.parse::<f32>().ok()?,p.next()?.parse::<f32>().ok()?,Color(p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?))))().filter(|(w,h,_,_)|*w>0.0 && *h>0.0).map(|(w,h,r,c)|(w*d,h*d,if r<0.0 {r} else {r*d},c))
        });
        let start=get(173);let end=get(174);
        let (left,right)=if start.flatten().is_some() || end.flatten().is_some() {
            if rtl {(end.flatten(),start.flatten())}else{(start.flatten(),end.flatten())}
        } else {(get(169).flatten(),get(171).flatten())};
        [left,get(170).flatten(),right,get(172).flatten()]
    }
    pub(super) fn compound_sizes(&self, state: &LayoutState, rtl: bool) -> [Option<(f32, f32)>; 4] {
        let shapes = self.compound_drawables(state, rtl);
        let anims = self.compound_anim_slots(rtl);
        std::array::from_fn(|i| {
            anims[i]
                .and_then(|slot| self.compound_anim_intrinsic_size(slot, state))
                .or_else(|| shapes[i].map(|(w, h, _, _)| (w, h)))
        })
    }
    pub(super) fn compound_extents(&self, state: &LayoutState, rtl:bool)->[f32;4] {
        let sizes=self.compound_sizes(state,rtl);
        let pad=state.dp(self.relative_rules.iter().find(|r|r.kind==175).and_then(|r|r.anchor.parse::<f32>().ok()).unwrap_or(0.0).max(0.0));
        [sizes[0].map_or(0.0,|(w,_)|w+pad),sizes[1].map_or(0.0,|(_,h)|h+pad),sizes[2].map_or(0.0,|(w,_)|w+pad),sizes[3].map_or(0.0,|(_,h)|h+pad)]
    }
    pub(super) fn paint_compound_drawables(
        &self,
        f: &mut Frame<'_>,
        inner: Rect,
        text_area: Rect,
        state: &mut LayoutState,
        key: &str,
        response: Response,
    ) {
        let shapes = self.compound_drawables(state, state.rtl);
        let anim_slots = self.compound_anim_slots(state.rtl);
        if shapes.iter().all(Option::is_none) && anim_slots.iter().all(Option::is_none) {
            return;
        }
        let tint = self.resolved_progress_tint(176, state, key, response);
        let tint_mode = self
            .relative_rules
            .iter()
            .find(|r| r.kind == 177)
            .and_then(|r| r.anchor.parse::<u8>().ok())
            .unwrap_or(1);
        let has_relative = self.relative_rules.iter().any(|r| {
            (r.kind == 173 || r.kind == 174)
                && r.anchor
                    .split(',')
                    .next()
                    .and_then(|n| n.parse::<f32>().ok())
                    .is_some_and(|w| w > 0.0)
        });
        let sides = if has_relative {
            if state.rtl { [174, 170, 173, 172] } else { [173, 170, 174, 172] }
        } else {
            [169, 170, 171, 172]
        };
        let mirror = self.is_auto_mirrored() && state.rtl;
        let alpha = if response.enabled { 1.0 } else { 0.38 };
        f.within(inner, |f| {
            for i in 0..4 {
                if let Some(slot) = anim_slots[i]
                    && let Some(spec) = self.compound_anim_spec(slot)
                {
                    let (frame_key, next_wake) =
                        select_animation_list_frame(&spec, f.time(), f.reduced_motion());
                    if let Some(img) = state.images.get(frame_key).cloned() {
                        let scale = state.image_scales.get(frame_key).copied().unwrap_or(1.0)
                            * state.effective_density();
                        let (w, h) = (img.width() as f32 * scale, img.height() as f32 * scale);
                        let rect = match i {
                            0 => Rect { x: inner.x, y: text_area.y + (text_area.height - h) * 0.5, width: w, height: h },
                            1 => Rect { x: text_area.x + (text_area.width - w) * 0.5, y: inner.y, width: w, height: h },
                            2 => Rect { x: inner.x + inner.width - w, y: text_area.y + (text_area.height - h) * 0.5, width: w, height: h },
                            _ => Rect { x: text_area.x + (text_area.width - w) * 0.5, y: inner.y + inner.height - h, width: w, height: h },
                        };
                        if let Some(wake) = next_wake {
                            let vis = rect.intersect(f.clip);
                            if vis.width > 0.0 && vis.height > 0.0 {
                                f.wake_at(wake);
                            }
                        }
                        let tinted = tint_image(state, &img, tint, tint_mode);
                        if mirror {
                            f.push_transform(
                                [rect.x + rect.width * 0.5, rect.y + rect.height * 0.5],
                                [0.0, 0.0],
                                [-1.0, 1.0],
                                0.0,
                            );
                        }
                        f.commands.push(DrawCommand::Image {
                            bounds: rect,
                            uv: [0.0, 0.0, 1.0, 1.0],
                            radius: 0.0,
                            color: Color(255, 255, 255, 255).opacity(alpha),
                            image: tinted,
                        });
                        if mirror {
                            f.pop_transform();
                        }
                        continue;
                    }
                }
                let shape = shapes[i];
                let selected = self.custom_check_mark(sides[i] + 9, state, key, response).unwrap_or(shape);
                if let (Some((w, h, _, _)), Some((source_w, source_h, r, original))) = (shape, selected) {
                    let r = if r < 0.0 { r } else { r * (w / source_w).min(h / source_h) };
                    let color = tint.map_or(original, |value| tint_pixel(original, value, tint_mode));
                    let rect = match i {
                        0 => Rect { x: inner.x, y: text_area.y + (text_area.height - h) * 0.5, width: w, height: h },
                        1 => Rect { x: text_area.x + (text_area.width - w) * 0.5, y: inner.y, width: w, height: h },
                        2 => Rect { x: inner.x + inner.width - w, y: text_area.y + (text_area.height - h) * 0.5, width: w, height: h },
                        _ => Rect { x: text_area.x + (text_area.width - w) * 0.5, y: inner.y + inner.height - h, width: w, height: h },
                    };
                    if r < 0.0 {
                        paint_oval(f, rect, color);
                    } else {
                        f.paint_rect(rect, r, color);
                    }
                }
            }
        });
    }
    pub(super) fn custom_check_mark(&self,kind:u16,state:&LayoutState,key:&str,response:Response)->Option<Option<(f32,f32,f32,Color)>> {
        let d=state.effective_density();
        let bits=u8::from(response.enabled) | (u8::from(response.pressed)<<1) | (u8::from(response.focused)<<2)
            | (u8::from(state.view_selected.get(key).copied().unwrap_or(false))<<3)
            | (u8::from(state.view_activated.get(key).copied().unwrap_or(false))<<4) | (u8::from(response.hovered)<<5)
            | (u8::from(state.checked.get(key).copied().unwrap_or(self.checked))<<6);
        let mut first=None;let mut fallback=None;
        for rule in self.relative_rules.iter().filter(|r|r.kind==kind) {
            let mut p=rule.anchor.split(',');
            let parsed=(||Some((p.next()?.parse::<u8>().ok()?,p.next()?.parse::<u8>().ok()?,p.next()?.parse::<f32>().ok()?,p.next()?.parse::<f32>().ok()?,p.next()?.parse::<f32>().ok()?,Color(p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?))))();
            if let Some((required,forbidden,w,h,r,color))=parsed {
                let mark=if w>0.0 && h>0.0 {Some((w*d,h*d,if r<0.0 {r} else {r*d},color))}else{None};
                if first.is_none(){first=Some(mark);}if required==0 && forbidden==0 {fallback=Some(mark);}
                if bits&required==required && bits&forbidden==0{return Some(mark);}
            }
        }
        fallback.or_else(||first.map(|_|None))
    }
    pub(super) fn resolved_tint_with_checked(&self, kind:u16, state:&LayoutState, key:&str, response:Response, checked:Option<bool>)->Option<Color> {
        let bits=u8::from(response.enabled) | (u8::from(response.pressed)<<1) | (u8::from(response.focused)<<2)
            | (u8::from(state.view_selected.get(key).copied().unwrap_or(false))<<3)
            | (u8::from(state.view_activated.get(key).copied().unwrap_or(false))<<4) | (u8::from(response.hovered)<<5)
            | (u8::from(matches!(self.kind,Kind::CheckedText|Kind::CheckBox|Kind::RadioButton|Kind::Switch|Kind::ToggleButton) && checked.unwrap_or_else(||state.checked.get(key).copied().unwrap_or(self.checked)))<<6);
        let mut first=None; let mut fallback=None;
        for rule in self.relative_rules.iter().filter(|rule|rule.kind==kind) {
            let mut p=rule.anchor.split(',');
            let parsed=(||Some((p.next()?.parse::<u8>().ok()?,p.next()?.parse::<u8>().ok()?,Color(p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?,p.next()?.parse().ok()?))))();
            if let Some((required,forbidden,color))=parsed {
                if first.is_none(){first=Some(color);} if required==0 && forbidden==0 {fallback=Some(color);}
                if bits&required==required && bits&forbidden==0 {return Some(color);}
            }
        }
        fallback.or(first)
    }
    pub(super) fn resolved_image_tint(&self, state:&LayoutState, key:&str, response:Response) -> Option<Color> {
        self.resolved_state_color(self.image_tint_states,self.image_tint,state,key,response)
    }
    pub(super) fn effective_visibility(&self, state: &LayoutState) -> Visibility {
        if self.id.is_empty() {
            return self.visibility;
        }
        state
            .visibility
            .get(self.id)
            .copied()
            .unwrap_or(self.visibility)
    }
    pub(super) fn resolved_view_padding(&self, state: &LayoutState) -> [f32; 4] {
        let d = state.effective_density();
        if matches!(self.kind,Kind::MaterialCard) {
            // CardView ignores setPadding/setPaddingRelative and uses physical
            // contentPadding values instead (AndroidX CardView.java).
            let (h,v)=self.card_compat_insets(state);
            return std::array::from_fn(|i| self.relative_rules.iter().find(|r|r.kind==67+i as u16).and_then(|r|r.anchor.parse::<f32>().ok()).unwrap_or(0.0)*d+if i%2==0 {h} else {v});
        }
        let rtl = match self.layout_direction { 1 => false, 2 => true, _ => state.rtl };
        let mut padding = self.background_padding.unwrap_or_else(|| self.default_theme_padding(state));
        if rtl && self.background_padding.is_some() { padding.swap(0,2); }
        for (i,value) in padding.iter_mut().enumerate() {
            if self.padding_explicit[i] || self.padding_edges[i]!=0.0 { *value=self.padding_edges[i]; }
        }
        let start = if rtl { 2 } else { 0 };
        let end = if rtl { 0 } else { 2 };
        if let Some(value) = self.padding_relative[0] { padding[start] = value; }
        if let Some(value) = self.padding_relative[1] { padding[end] = value; }
        padding.map(|v| v * d)
    }
    pub(super) fn card_compat_insets(&self, state: &LayoutState)->(f32,f32) {
        let rule=|kind|self.relative_rules.iter().find(|r|r.kind==kind).map(|r|r.anchor);
        if rule(73)!=Some("true") {return (0.0,0.0);}
        let default_elevation=if rule(57)==Some("Elevated") {1.0} else {0.0};
        let elevation=rule(71).and_then(|s|s.parse::<f32>().ok()).unwrap_or(default_elevation).max(0.0);
        let max_elevation=rule(72).and_then(|s|s.parse::<f32>().ok()).unwrap_or(0.0).max(elevation);
        let radius=rule(66).and_then(|s|s.parse::<f32>().ok()).unwrap_or(12.0).max(0.0);
        let corner=if rule(74)==Some("false") {0.0} else {(1.0-std::f32::consts::FRAC_1_SQRT_2)*radius};
        let d = state.effective_density();
        ((max_elevation+corner).ceil()*d,(1.5*max_elevation+corner).ceil()*d)
    }
    pub(super) fn resolved_padding(&self, state: &LayoutState) -> [f32; 4] {
        let mut padding=self.resolved_view_padding(state);
        if matches!(self.kind,Kind::Frame | Kind::MediaController | Kind::MaterialCard|Kind::ViewAnimator|Kind::ViewSwitcher|Kind::TextSwitcher|Kind::ImageSwitcher|Kind::ViewFlipper)
            && self.foreground_gravity==[5,5] && let Some(foreground)=self.foreground_padding {
            let d = state.effective_density();
            for i in 0..4 {
                let fg = foreground[i] * d;
                padding[i]=if self.foreground_excludes_padding { padding[i]+fg } else { padding[i].max(fg) };
            }
        }
        padding
    }
    pub(super) fn resolved_margin(&self, state: &LayoutState) -> [f32; 4] {
        let mut margin = self.margin;
        let start = if state.rtl { 2 } else { 0 };
        let end = if state.rtl { 0 } else { 2 };
        if let Some(value) = self.margin_relative[0] { margin[start] = value; }
        if let Some(value) = self.margin_relative[1] { margin[end] = value; }
        let d = state.effective_density();
        margin.map(|v| v * d)
    }
    pub(super) fn margin_for_axis(&self, horizontal: bool, state: &LayoutState) -> f32 {
        let margin = self.resolved_margin(state);
        if horizontal { margin[0] + margin[2] } else { margin[1] + margin[3] }
    }
    pub(super) fn divider_count(&self, state:&LayoutState) -> usize {
        if self.divider_color.is_none() { return 0; }
        let visible=self.children.iter().filter(|n|n.effective_visibility(state)!=Visibility::Gone).count();
        if visible==0 { return 0; }
        usize::from(self.divider_flags & 1 != 0) + if self.divider_flags & 2 != 0 { visible.saturating_sub(1) } else { 0 } + usize::from(self.divider_flags & 4 != 0)
    }
    pub(super) fn intrinsic(&self, horizontal: bool, state: &LayoutState, id:u64) -> f32 {
        let rtl=match self.layout_direction {1=>false,2=>true,_=>state.rtl};
        if rtl!=state.rtl { let mut scoped=state.clone();scoped.rtl=rtl;return self.intrinsic(horizontal,&scoped,id); }
        if self.is_search_view_closed(state, id) && matches!(if horizontal { self.width } else { self.height }, Size::Wrap) {
            return 0.0;
        }
        if !horizontal && let Some(ch) = self.collapsing_toolbar_height(state, id) {
            return ch.max(state.dp(self.min_height));
        }
        let d = state.effective_density();
        match if horizontal { self.width } else { self.height } {
            Size::Dp(v) => v * d,
            _ => {
                let inner = match self.kind {
                    Kind::Linear | Kind::RadioGroup => {
                        if let Some(nav) = self.nav_bar_spec() {
                            let count = self.nav_items().len().max(1);
                            match nav.kind {
                                NavBarKind::Bar => {
                                    if horizontal {
                                        count as f32 * state.dp(80.0)
                                    } else {
                                        state.dp(80.0)
                                    }
                                }
                                NavBarKind::Rail => {
                                    if horizontal {
                                        state.dp(80.0)
                                    } else {
                                        let item_h = if nav.label_mode == NavLabelMode::Unlabeled { 48.0 } else { 56.0 };
                                        count as f32 * state.dp(item_h)
                                            + count.saturating_sub(1) as f32 * state.dp(12.0)
                                            + state.dp(24.0)
                                    }
                                }
                                NavBarKind::Drawer => {
                                    if horizontal {
                                        state.dp(300.0)
                                    } else {
                                        self.drawer_header_intrinsic_height(state, id)
                                            + count as f32 * state.dp(56.0)
                                            + state.dp(24.0)
                                    }
                                }
                            }
                        } else {
                        let visible_count = self
                            .children
                            .iter()
                            .filter(|n| n.effective_visibility(state) != Visibility::Gone)
                            .count();
                        let extra_gap_total = if self.horizontal == horizontal {
                            let gap = if let Some(cg) = self.chip_group_wrap_spec() {
                                state.dp(cg.h_spacing_dp.max(0.0))
                            } else {
                                self.linear_extra_gap(state)
                            };
                            visible_count.saturating_sub(1) as f32 * gap
                        } else {
                            0.0
                        };
                        let appbar_parent_scroll = if !horizontal && !self.horizontal && matches!(self.kind, Kind::Linear) && self.children.iter().any(|c| c.app_bar_scroll_flags().is_some()) {
                            Some(self.app_bar_scroll_px(state, id))
                        } else {
                            None
                        };
                        let base = if self.horizontal == horizontal {
                            self.children
                                .iter()
                                .enumerate()
                                .map(|(i,n)| {
                                    if n.effective_visibility(state)==Visibility::Gone { 0.0 } else {
                                        let cid = child_view_id(id,i,n);
                                        let h = n.intrinsic(horizontal,state,cid);
                                        let m_axis = n.margin_for_axis(horizontal,state);
                                        if let Some(parent_scroll) = appbar_parent_scroll
                                            && n.app_bar_scroll_flags().is_some()
                                        {
                                            let sp = n.app_bar_child_scroll_px(parent_scroll, state, cid);
                                            let collapse = preview_app_bar_child_collapse(n, cid, h, m_axis, sp, state);
                                            (h + m_axis - collapse).max(0.0)
                                        } else {
                                            h + m_axis
                                        }
                                    }
                                })
                                .sum::<f32>()
                                + extra_gap_total
                        } else {
                            self.children
                                .iter()
                                .enumerate()
                                .map(|(i,n)| {
                                    if n.effective_visibility(state)==Visibility::Gone { 0.0 } else {
                                        n.intrinsic(horizontal,state,child_view_id(id,i,n))+n.margin_for_axis(horizontal,state)
                                    }
                                })
                                .fold(0., f32::max)
                        };
                        if !horizontal && let Some(spec) = self.text_input_box() {
                            let (top_inset, supporting_h) = self.text_input_extra_height(&spec, state, id);
                            base.max(state.dp(56.0)) + top_inset + supporting_h
                        } else if !horizontal && self.alert_dialog_icon().is_some() {
                            base + self.alert_dialog_top_icon_extra(state)
                        } else {
                            base
                        }
                        }
                    }
                    Kind::Grid => {
                        let (widths,heights,_,_,_)=self.grid_tracks(state,TextMetrics::default(),id,1000000.0,false,None);
                        if horizontal {widths.iter().sum()} else {heights.iter().sum()}
                    },
                    Kind::Table => {
                        if horizontal { self.table_columns(state,TextMetrics::default(),id).iter().sum() } else {
                            self.children.iter().enumerate().filter(|(_,n)| n.effective_visibility(state)!=Visibility::Gone).map(|(i,n)|n.intrinsic(false,state,child_view_id(id,i,n))+n.margin_for_axis(false,state)).sum()
                        }
                    },
                    Kind::TableRow => self.children.iter().enumerate().filter(|(_,n)|n.effective_visibility(state)!=Visibility::Gone).map(|(i,n)|n.intrinsic(horizontal,state,child_view_id(id,i,n))+n.margin_for_axis(horizontal,state)).reduce(|a,b|if horizontal {a+b} else {a.max(b)}).unwrap_or(0.0),
                    Kind::Relative | Kind::DialerFilter | Kind::ConstraintLayout => {
                        let bounds=Rect { x:0.0,y:0.0,width:1000000.0,height:1000000.0 };
                        let is_cl = matches!(self.kind,Kind::ConstraintLayout);
                        let ph_claimed = if is_cl { Some(resolve_constraint_placeholders(self.children, state).1) } else { None };
                        let visibilities = if is_cl { Some(constraint_group_visibilities(self.children, state)) } else { None };
                        let positions=if is_cl { constraint_layout_positions(self,id,bounds,state,TextMetrics::default(),true,true) } else { relative_layout::layout(self,id,bounds,state,TextMetrics::default(),true,true) };
                        let extent=positions.iter().enumerate().filter(|(i,_)| {
                            let vis = visibilities.as_ref().map_or_else(|| self.children[*i].effective_visibility(state), |v| v[*i]);
                            vis != Visibility::Gone
                                && !(is_cl && is_constraint_helper(&self.children[*i]))
                                && !ph_claimed.as_ref().is_some_and(|m| m[*i].is_some())
                        }).map(|(i,r)| {
                            let m=self.children[i].resolved_margin(state);
                            if horizontal { if state.rtl { (bounds.x+bounds.width)-(r.x-m[0]) } else { r.x+r.width+m[2] } } else { r.y+r.height+m[3] }
                        }).fold(0.0_f32,f32::max);
                        extent
                    },
                    Kind::MaterialDivider => if horizontal {0.0} else {state.dp(self.divider_thickness.max(0.0))},
                    Kind::Spinner => state.dp(if horizontal {160.0} else {48.0}),
                    Kind::GridView if self.carousel_layout_spec().is_some() => state.dp(if horizontal { 240.0 } else { 180.0 }),
                    Kind::GridView => state.dp(if horizontal {160.0}else{48.0}),
                    Kind::ListView if self.carousel_layout_spec().is_some() => state.dp(if horizontal { 240.0 } else { 180.0 }),
                    Kind::ListView => if horizontal {state.dp(160.0)} else {
                        let key=if self.id.is_empty() {format!("#{id}")} else {self.id.to_owned()};
                        let count=state.list_items.get(&key).map_or_else(||self.relative_rules.iter().filter(|r|r.kind==82).count(),Vec::len);
                        count as f32*state.dp(48.0)+count.saturating_sub(1) as f32*state.dp(self.divider_thickness)
                    },
                    Kind::Text | Kind::CheckedText | Kind::Chronometer | Kind::TextClock => {
                        let shapes=if matches!(self.kind,Kind::Text|Kind::CheckedText){self.compound_sizes(state,state.rtl)}else{[None;4]};
                        let ext=if matches!(self.kind,Kind::Text|Kind::CheckedText){self.compound_extents(state,state.rtl)}else{[0.0;4]};
                        let text_size = self.scaled_text_size(state);
                        if horizontal {
                            let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                            let text = self.display_text_for(state.text.get(&key).map_or(self.text, String::as_str), state);
                            let count=text.chars().filter(|c|!c.is_control()).count();
                            let text=(count as f32*text_size*0.55 + count.saturating_sub(1) as f32*text_size*self.letter_spacing_em).max(0.0) + if matches!(self.kind, Kind::CheckedText) { self.check_mark_width(state) } else { 0.0 };
                            (text+ext[0]+ext[2]).max(shapes[1].map_or(0.0,|s|s.0)).max(shapes[3].map_or(0.0,|s|s.0))
                        } else {
                            let lh = self.resolved_line_height(text_size, state);
                            let text = lh + (self.min_lines.max(1)-1) as f32 * (lh*self.line_spacing_multiplier+state.dp(self.line_spacing_extra)).max(1.0);
                            text.max(shapes[0].map_or(0.0,|s|s.1)).max(shapes[2].map_or(0.0,|s|s.1))+ext[1]+ext[3]
                        }
                    }
                    Kind::CheckBox | Kind::RadioButton | Kind::Switch => if horizontal { state.dp(48.) + self.text.chars().count() as f32 * self.scaled_text_size(state) * 0.55 } else { state.dp(48.) },
                    Kind::ExpandableListView => state.dp(240.0),
                    Kind::SearchView => if self.search_bar_hint().is_some() { state.dp(if horizontal { 240.0 } else { 56.0 }) } else { state.dp(48.0) },
                    Kind::Toolbar|Kind::ActionMenuView => if self.bottom_app_bar_spec().is_some() { state.dp(if horizontal { 240.0 } else { 80.0 }) } else { state.dp(if horizontal {240.0} else {56.0}) },
                    Kind::TabWidget if self.tab_options().is_some() => state.dp(if horizontal { 240.0 } else { 48.0 }),
                    Kind::AdapterViewFlipper|Kind::StackView|Kind::Gallery|Kind::TabWidget|Kind::SlidingDrawer => state.dp(if horizontal {240.0} else {160.0}),
                    Kind::NumberPicker => state.dp(if horizontal { 80.0 } else { 144.0 }),
                    Kind::TimePicker if self.relative_rules.iter().any(|r|r.kind==105&&r.anchor=="clock") => state.dp(if horizontal { 328.0 } else { 476.0 }),
                    Kind::TimePicker => state.dp(if horizontal { 280.0 } else { 144.0 }),
                    Kind::CalendarView => state.dp(if horizontal { 348.0 } else { 338.0 }),
                    Kind::DatePicker if self.relative_rules.iter().any(|r|r.kind==102&&r.anchor=="spinner") => {
                        let wheels=self.relative_rules.iter().all(|r|r.kind!=103||r.anchor!="false");let calendar=self.relative_rules.iter().all(|r|r.kind!=104||r.anchor!="false");
                        state.dp(if horizontal {if wheels&&calendar {540.0}else{348.0}}else if calendar {338.0}else{144.0})
                    },
                    Kind::DatePicker => state.dp(if horizontal { 348.0 } else { 454.0 }),
                    Kind::AnalogClock => state.dp(160.0),
                    Kind::RatingBar => state.dp(if horizontal { self.num_stars as f32 * 48.0 } else { 48.0 }),
                    Kind::SeekBar => state.dp(if horizontal { 160. } else { 48. }),
                    Kind::ProgressBar if self.loading_indicator_contained().is_some() => state.dp(48.0),
                    Kind::ProgressBar if self.horizontal && self.wavy_progress_spec().is_some() => {
                        let spec = self.wavy_progress_spec().unwrap();
                        if horizontal {
                            state.dp(160.0)
                        } else {
                            state.dp((4.0 + spec.amplitude_dp * 2.0).max(12.0))
                        }
                    }
                    Kind::ProgressBar => state.dp(if self.horizontal && horizontal { 160. } else if self.horizontal { 8. } else { 48. }),
                    Kind::View if self.is_drag_handle() => state.dp(48.0),
                    Kind::Image | Kind::QuickContactBadge | Kind::Icon | Kind::Zoom if self.image_adjust_bounds && horizontal && matches!(self.width, Size::Wrap) && matches!(self.height, Size::Dp(_)) => {
                        let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                        let source=self.resolved_intrinsic_image_src(state,&key);
                        let padding=self.resolved_padding(state);
                        state.images.get(source).map_or(if matches!(self.kind,Kind::Icon|Kind::Zoom) {state.dp(48.0)} else {0.0},|image| {
                            let height=if let Size::Dp(h)=self.height { state.dp(h) } else {0.0};
                            (height-padding[1]-padding[3]).max(0.0)*image.width() as f32/image.height() as f32
                        })
                    },
                    Kind::Icon | Kind::Zoom if self.image_adjust_bounds => {
                        let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                        let source=self.resolved_intrinsic_image_src(state,&key);
                        state.images.get(source).map_or(state.dp(48.0),|image| {
                            let scale=state.image_scales.get(source).copied().unwrap_or(1.0)*d;
                            (if horizontal { image.width() as f32 } else { image.height() as f32 })*scale
                        })
                    },
                    Kind::Image | Kind::QuickContactBadge => {
                        let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                        let source = self.resolved_intrinsic_image_src(state, &key);
                        state.images.get(source).map_or(0.0, |image| {
                            let scale = state.image_scales.get(source).copied().unwrap_or(1.0)*d;
                            (if horizontal { image.width() as f32 } else { image.height() as f32 }) * scale
                        })
                    },
                    Kind::VideoView => {let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};let dimensions=state.video_dimensions.get(&key).copied().or_else(||state.video_frames.get(&key).map(|frame|(frame.width(),frame.height()))).unwrap_or((0,0));if horizontal{dimensions.0 as f32}else{dimensions.1 as f32}},
                    Kind::ZoomControls => state.dp(if horizontal { 158. } else { 48. }),
                    Kind::Button if self.extended_fab_spec().is_some() => {
                        let (spec, m, _) = self.extended_fab_progress(state, id).unwrap();
                        let col = state.dp(spec.collapsed_size_dp);
                        if horizontal {
                            let ext_base = state.dp(44.0).max(col);
                            col + (ext_base - col) * m.progress
                        } else {
                            let ext_h = state.dp(56.0);
                            col + (ext_h - col) * m.progress
                        }
                    }
                    Kind::Button | Kind::ToggleButton | Kind::Icon | Kind::Zoom | Kind::Edit | Kind::AutoComplete | Kind::MultiAutoComplete => state.dp(48.),
                    _ => {
                        self.children
                        .iter()
                        .enumerate()
                        .map(|(i,n)| {
                            if n.effective_visibility(state) == Visibility::Gone && !self.measure_all_children { return 0.0; }
                            if self.drawer_panel_is_closed(n, state, id) { return 0.0; }
                            n.intrinsic(horizontal, state, child_view_id(id,i,n))
                                + if matches!(self.kind,Kind::Absolute) { state.dp(n.absolute_position[if horizontal {0} else {1}]) } else { n.margin_for_axis(horizontal, state) }
                        })
                        .fold(0., f32::max)
                    }
                };
                let inner = inner + if matches!(self.kind,Kind::Linear|Kind::RadioGroup|Kind::TableRow) && self.horizontal==horizontal { self.divider_count(state) as f32 * state.dp(self.divider_thickness) } else { 0.0 };
                let mut minimum = state.dp(if horizontal { self.min_width } else { self.min_height });
                if !horizontal && self.is_snackbar() {
                    minimum = minimum.max(state.dp(48.0));
                } else if !horizontal && self.is_floating_toolbar() {
                    minimum = minimum.max(state.dp(64.0));
                }
                let bg_min = state.dp(self.background_min_size[usize::from(!horizontal)]);
                if matches!(self.kind, Kind::Space) { return minimum.max(bg_min); }
                let padding = self.resolved_padding(state);
                let mut measured=(inner + if horizontal { padding[0] + padding[2] } else { padding[1] + padding[3] }).max(minimum).max(bg_min).max(if matches!(self.kind,Kind::Frame | Kind::MediaController | Kind::MaterialCard|Kind::ViewAnimator|Kind::ViewSwitcher|Kind::TextSwitcher|Kind::ImageSwitcher|Kind::ViewFlipper) { state.dp(self.foreground_size[usize::from(!horizontal)]) } else { 0.0 });
                if horizontal && self.image_adjust_bounds && matches!(self.kind,Kind::Image|Kind::QuickContactBadge|Kind::Icon|Kind::Zoom) && matches!(self.width,Size::Wrap) {
                    if matches!(self.height,Size::Wrap) && let Some(max_height)=self.image_max_height {
                        let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
                        let source=state.image_sources.get(&key).map_or(self.image_src,String::as_str);
                        if let Some(image)=state.images.get(source) {
                            let max_content=(state.dp(max_height)-padding[1]-padding[3]).max(0.0);
                            measured=measured.min(max_content*image.width() as f32/image.height() as f32+padding[0]+padding[2]);
                        }
                    }
                    if let Some(max_width)=self.image_max_width { measured=measured.min(state.dp(max_width)); }
                }
                measured
            }
        }
    }
    pub(super) fn grid_weight(&self,kind:u16)->f32 {self.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|r.anchor.parse::<f32>().ok()).filter(|v|v.is_finite() && *v>0.0).unwrap_or(0.0)}
    pub(super) fn grid_weighted_fill(&self,horizontal:bool)->bool {
        let (weight,explicit)=if horizontal {(46,48)} else {(47,49)};
        self.grid_weight(weight)>0.0 && !self.relative_rules.iter().any(|r|r.kind==explicit)
    }
    pub(super) fn grid_margin(&self,child:&Node,state:&LayoutState)->[f32;4] {
        let mut margin=child.resolved_margin(state);
        if !self.relative_rules.iter().any(|r|r.kind==53) || matches!(child.kind,Kind::Space) {return margin;}
        let mut mask=child.relative_rules.iter().find(|r|r.kind==50).and_then(|r|r.anchor.parse::<u8>().ok()).unwrap_or(0);
        if child.margin_relative[0].is_some() {mask|=if state.rtl {4} else {1};}
        if child.margin_relative[1].is_some() {mask|=if state.rtl {1} else {4};}
        // Each undefined side receives half the host theme's default gap.
        let gap=state.grid_default_gap.filter(|gap|gap.is_finite() && *gap>=0.0).unwrap_or_else(|| state.dp(8.0));
        for (i,value) in margin.iter_mut().enumerate() {if mask & (1<<i)==0 {*value=gap/2.0;}}
        margin
    }
    pub(super) fn grid_uses_baseline(&self)->bool {
        self.layout_gravity[1]==6 || (self.layout_gravity[1]==0 && !self.relative_rules.iter().any(|r|r.kind==49) && self.grid_weight(47)==0.0)
    }
    // Initial GridLayout uses automatic placement and intrinsic span constraints.
    pub(super) fn grid_tracks(&self,state:&LayoutState,metrics:TextMetrics,id:u64,available_width:f32,expand_width:bool,available_height:Option<f32>)->(Vec<f32>,Vec<f32>,Vec<(usize,usize,usize,usize)>,Vec<(usize,usize,f32)>,Vec<[f32;4]>) {
        let count=self.children.len();
        let read=|kind|self.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|r.anchor.parse::<usize>().ok());
        let minor_count=(if self.horizontal {read(40)} else {read(41)}).map(|count|count.clamp(1,128));
        let mut occupied=vec![0usize;minor_count.unwrap_or(0)];let mut major=0usize;let mut minor=0usize;
        let mut positions=Vec::with_capacity(count);let mut cols=0;let mut rows=0;
        for child in self.children.iter() {
            let get=|kind|child.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|r.anchor.parse::<usize>().ok());
            let explicit_major=if self.horizontal {get(43)} else {get(42)};
            let explicit_minor=if self.horizontal {get(42)} else {get(43)};
            let row_span=get(45).unwrap_or(1).clamp(1,128);let col_span=get(44).unwrap_or(1).clamp(1,128);
            let major_span=if self.horizontal {row_span} else {col_span};
            let mut minor_span=if self.horizontal {col_span} else {row_span};
            if let Some(value)=explicit_major {major=value;}
            if let Some(value)=explicit_minor {minor=value;}
            if let Some(limit)=minor_count {
                if explicit_minor.is_some() && minor>=limit {minor=limit-1;}
                minor_span=minor_span.min(if explicit_minor.is_some() {limit-minor} else {limit});
                if explicit_major.is_none() || explicit_minor.is_none() {
                    while minor+minor_span>limit || occupied[minor..(minor+minor_span).min(limit)].iter().any(|&value|value>major) {
                        if explicit_minor.is_some() {major+=1;} else if minor+minor_span<=limit {minor+=1;} else {minor=0;major+=1;}
                    }
                }
                for value in &mut occupied[minor..(minor+minor_span).min(limit)] {*value=major+major_span;}
            }
            let (row,col,rs,cs)=if self.horizontal {(major,minor,major_span,minor_span)} else {(minor,major,minor_span,major_span)};
            positions.push((row,col,rs,cs));rows=rows.max(row+rs);cols=cols.max(col+cs);minor+=minor_span;
        }
        let mut margins:Vec<[f32;4]>=self.children.iter().map(|n|self.grid_margin(n,state)).collect();
        if self.relative_rules.iter().any(|r|r.kind==54) {
            let mut left=vec![0.0_f32;cols+1];let mut right=vec![0.0_f32;cols+1];
            let mut top=vec![0.0_f32;rows+1];let mut bottom=vec![0.0_f32;rows+1];
            for (i,n) in self.children.iter().enumerate() {
                if n.effective_visibility(state)==Visibility::Gone {continue;}
                let (r,c,rs,cs)=positions[i];let m=margins[i];
                left[c]=left[c].max(m[0]);right[c+cs]=right[c+cs].max(m[2]);top[r]=top[r].max(m[1]);bottom[r+rs]=bottom[r+rs].max(m[3]);
            }
            for (i,m) in margins.iter_mut().enumerate() {let (r,c,rs,cs)=positions[i];*m=[left[c],top[r],right[c+cs],bottom[r+rs]];}
        }
        let mut width_constraints=Vec::new();
        // Single-track children establish track minima; spanning children add any deficit.
        for spanning in [false,true] {for (i,n) in self.children.iter().enumerate() {
            if n.effective_visibility(state)==Visibility::Gone {continue;}let (_,col,_,span)=positions[i];if (span>1)!=spanning {continue;}let margin=margins[i];
            let available=(available_width-margin[0]-margin[2]).max(0.0);
            let wanted=n.layout_intrinsic(true,state,available,metrics,child_view_id(id,i,n)).min(available)+margin[0]+margin[2];
            width_constraints.push((col,span,wanted,n.grid_weight(46)));
        }}
        let mut widths=grid_solve_constraints(cols,&width_constraints,0.0,0.0);
        if expand_width {
            grid_distribute(&mut widths,&width_constraints,available_width);
        }
        let mut height_constraints=Vec::new();
        for spanning in [false,true] {for (i,n) in self.children.iter().enumerate() {
            if n.effective_visibility(state)==Visibility::Gone {continue;}let (row,col,span,cs)=positions[i];if (span>1)!=spanning {continue;}let margin=margins[i];
            let available=(widths[col..col+cs].iter().sum::<f32>()-margin[0]-margin[2]).max(0.0);
            let key=child_view_id(id,i,n);
            let width=if matches!(n.width,Size::Match) || (n.layout_gravity_explicit && n.layout_gravity[0]==5) || n.grid_weighted_fill(true) {available} else {n.layout_intrinsic(true,state,available,metrics,key).min(available)};
            let wanted=n.layout_intrinsic(false,state,width,metrics,key)+margin[1]+margin[3];
            height_constraints.push((row,span,wanted,n.grid_weight(47)));
        }}
        // Baseline groups are shared by children with the same row interval.
        let mut groups:Vec<(usize,usize,f32,f32)>=Vec::new();
        for (i,n) in self.children.iter().enumerate() {
            if n.effective_visibility(state)==Visibility::Gone || !n.grid_uses_baseline() || matches!(n.height,Size::Match) {continue;}
            let (row,col,rs,cs)=positions[i];let m=margins[i];let key=child_view_id(id,i,n);
            let available=(widths[col..col+cs].iter().sum::<f32>()-m[0]-m[2]).max(0.0);
            let width=if matches!(n.width,Size::Match) || (n.layout_gravity_explicit && n.layout_gravity[0]==5) || n.grid_weighted_fill(true) {available} else {n.layout_intrinsic(true,state,available,metrics,key).min(available)};
            let height=n.layout_intrinsic(false,state,width,metrics,key);
            if let Some(base)=n.relative_baseline(width,height,state,metrics,key) {
                let above=base+m[1];let below=height-base+m[3];
                if let Some(group)=groups.iter_mut().find(|g|g.0==row && g.1==rs) {group.2=group.2.max(above);group.3=group.3.max(below);} else {groups.push((row,rs,above,below));}
            }
        }
        for &(row,span,above,below) in &groups {
            height_constraints.push((row,span,above+below,0.0));
        }
        let mut heights=grid_solve_constraints(rows,&height_constraints,0.0,0.0);
        if let Some(available)=available_height {
            grid_distribute(&mut heights,&height_constraints,available);
        }
        (widths,heights,positions,groups.into_iter().map(|(row,span,above,_)|(row,span,above)).collect(),margins)
    }
    // TableRow's virtual columns include explicit gaps and repeated columns for spans.
    pub(super) fn table_cells(&self) -> Vec<(usize,usize,usize)> {
        let mut result=Vec::with_capacity(self.children.len());let mut next=0usize;
        for (index,cell) in self.children.iter().enumerate() {
            let read=|kind|cell.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|r.anchor.parse::<usize>().ok());
            let column=read(30).unwrap_or(next).max(next);let span=read(31).unwrap_or(1).clamp(1,128);
            result.push((index,column,span));next=column.saturating_add(span);
        }
        result
    }
    pub(super) fn table_cell_collapsed(&self, state:&LayoutState, id:u64, column:usize, span:usize) -> bool {
        let initial=self.relative_rules.iter().find(|r|r.kind==34).map(|r|r.anchor);
        (column..column+span).any(|index|state.table_collapsed.get(&(id,index)).copied().unwrap_or_else(||initial.is_some_and(|list|list.split(',').any(|v|v.parse::<usize>().ok()==Some(index)))))
    }
    /// Override a named table column's collapsed state. Returns false for unnamed/non-table nodes.
    pub fn set_table_column_collapsed(&self,state:&mut LayoutState,column:usize,collapsed:bool) -> bool {
        if !matches!(self.kind,Kind::Table) || self.id.is_empty() {return false;}
        state.table_collapsed.insert((view_id(self.id),column),collapsed);true
    }
    pub(super) fn table_columns(&self, state:&LayoutState, metrics:TextMetrics, id:u64) -> Vec<f32> {
        let mut columns=Vec::<f32>::new();
        for (i,row) in self.children.iter().enumerate() {
            if !matches!(row.kind,Kind::TableRow) || row.effective_visibility(state)==Visibility::Gone {continue;}
            let row_rtl=match row.layout_direction {1=>false,2=>true,_=>state.rtl};
            let scoped;let state=if row_rtl!=state.rtl {scoped={let mut copy=state.clone();copy.rtl=row_rtl;copy};&scoped} else {state};
            for (j,column,span) in row.table_cells() {
                if columns.len()<column+span {columns.resize(column+span,0.0);}
                let cell=&row.children[j];
                if span==1 && !self.table_cell_collapsed(state,id,column,span) && cell.effective_visibility(state)!=Visibility::Gone {
                    let key=child_view_id(child_view_id(id,i,row),j,cell);
                    columns[column]=columns[column].max(cell.layout_intrinsic(true,state,1_000_000.0,metrics,key)+cell.margin_for_axis(true,state));
                }
            }
        }
        columns
    }
    pub(super) fn table_columns_for_width(&self, state:&LayoutState, metrics:TextMetrics, id:u64, width:f32) -> Vec<f32> {
        let mut columns=self.table_columns(state,metrics,id);let total=columns.iter().sum::<f32>();
        let kind=if total>width {33} else if total<width {32} else {return columns};
        if let Some(rule)=self.relative_rules.iter().find(|r|r.kind==kind) {
            let selected:Vec<usize>=if rule.anchor=="*" {(0..columns.len()).collect()} else {
                rule.anchor.split(',').filter_map(|part|part.parse::<usize>().ok()).filter(|&index|index<columns.len()).collect()
            };
            if !selected.is_empty() {
                // Android uses integer pixel deltas and truncates division toward zero.
                let extra=((width-total)/selected.len() as f32).trunc();
                for index in selected {columns[index]=(columns[index]+extra).max(0.0);}
            }
        }
        columns
    }
    // Baselines use the same font metrics as the renderer. Android tracks ascent
    // and descent separately for top, centre and bottom gravity groups.
    pub(super) fn table_row_metrics(&self, row:&Node, columns:&[f32], state:&LayoutState, metrics:TextMetrics, id:u64, table_id:u64) -> (f32,[f32;3],[f32;3]) {
        let row_rtl=match row.layout_direction {1=>false,2=>true,_=>state.rtl};
        let scoped;let state=if row_rtl!=state.rtl {scoped={let mut copy=state.clone();copy.rtl=row_rtl;copy};&scoped} else {state};
        let padding=row.resolved_padding(state);let mut max_height=0.0_f32;let mut ascent=[-1.0_f32;3];let mut descent=[-1.0_f32;3];
        let baseline_aligned=!row.relative_rules.iter().any(|r|r.kind==36);
        for (j,column,span) in row.table_cells() {
            let n=&row.children[j];if n.effective_visibility(state)==Visibility::Gone || self.table_cell_collapsed(state,table_id,column,span) {continue;}
            let available=(columns.iter().skip(column).take(span).sum::<f32>()-n.margin_for_axis(true,state)).max(0.0);
            let child_id=child_view_id(id,j,n);
            let width=if n.relative_rules.iter().any(|r|r.kind==35) {n.layout_intrinsic(true,state,available,metrics,child_id).min(available)} else {available};
            let height=n.layout_intrinsic(false,state,width,metrics,child_id);let margin=n.resolved_margin(state);
            max_height=max_height.max(height+margin[1]+margin[3]);
            if baseline_aligned && let Some(baseline)=n.relative_baseline(width,height,state,metrics,child_id) {
                let gravity=if n.layout_gravity_explicit {n.layout_gravity[1]} else {row.content_gravity[1]}.min(2) as usize;
                ascent[gravity]=ascent[gravity].max(baseline);descent[gravity]=descent[gravity].max(height+margin[1]+margin[3]-baseline);
            }
        }
        let max_ascent=ascent.iter().copied().fold(-1.0_f32,f32::max);let max_descent=descent.iter().copied().fold(-1.0_f32,f32::max);
        if max_ascent>=0.0 && max_descent>=0.0 {max_height=max_height.max(max_ascent+max_descent);}
        (max_height+padding[1]+padding[3],ascent,descent)
    }
    pub(super) fn standalone_row_metrics(&self,state:&LayoutState,metrics:TextMetrics,id:u64,widths:&[f32])->(f32,[f32;3],[f32;3]) {
        let padding=self.resolved_padding(state);let mut max_height=0.0_f32;let mut ascent=[-1.0_f32;3];let mut descent=[-1.0_f32;3];
        for (i,n) in self.children.iter().enumerate() {
            if n.effective_visibility(state)==Visibility::Gone {continue;}
            let key=child_view_id(id,i,n);let width=widths.get(i).copied().unwrap_or(0.0);
            let height=n.layout_intrinsic_for_width(false,state,width,n.weight>0.0,metrics,key);let margin=n.resolved_margin(state);
            max_height=max_height.max(height+margin[1]+margin[3]);
            if !self.relative_rules.iter().any(|r|r.kind==36) && let Some(base)=n.relative_baseline(width,height,state,metrics,key) {
                let gravity=if n.layout_gravity_explicit {n.layout_gravity[1]} else {self.content_gravity[1]}.min(2) as usize;
                ascent[gravity]=ascent[gravity].max(base);descent[gravity]=descent[gravity].max(height+margin[1]+margin[3]-base);
            }
        }
        let a=ascent.iter().copied().fold(-1.0_f32,f32::max);let d=descent.iter().copied().fold(-1.0_f32,f32::max);
        if a>=0.0 && d>=0.0 {max_height=max_height.max(a+d);}
        (max_height+padding[1]+padding[3],ascent,descent)
    }
    pub(super) fn linear_main_lengths(
        &self,
        horizontal: bool,
        state: &LayoutState,
        size: f32,
        inner_width: f32,
        metrics: TextMetrics,
        id: u64,
    ) -> Vec<f32> {
        let appbar_parent_scroll = if !horizontal && matches!(self.kind, Kind::Linear) && self.children.iter().any(|c| c.app_bar_scroll_flags().is_some()) {
            Some(self.app_bar_scroll_px(state, id))
        } else {
            None
        };
        let measure_child_len = |i: usize, n: &Node| -> (f32, f32) {
            let margins = n.resolved_margin(state);
            let cid = child_view_id(id, i, n);
            let len = n.layout_intrinsic(
                horizontal,
                state,
                (inner_width - margins[0] - margins[2]).max(0.0),
                metrics,
                cid,
            );
            let m_axis = n.margin_for_axis(horizontal, state);
            if let Some(parent_scroll) = appbar_parent_scroll
                && let Some(flags) = n.app_bar_scroll_flags()
            {
                let sp = n.app_bar_child_scroll_px(parent_scroll, state, cid);
                let collapse = preview_app_bar_child_collapse(n, cid, len, m_axis, sp, state);
                let (eff_top, vis_h, eff_bot, _) = app_bar_child_collapsed_layout(
                    len,
                    margins[1],
                    margins[3],
                    collapse,
                    flags.snap_margins,
                );
                return (vis_h, eff_top + eff_bot);
            }
            (len, m_axis)
        };
        let fixed: f32 = self
            .children
            .iter()
            .enumerate()
            .filter(|(_, n)| n.effective_visibility(state) != Visibility::Gone)
            .map(|(i, n)| {
                let (len, m_axis) = measure_child_len(i, n);
                len + m_axis
            })
            .sum();
        let extra_gap = self.linear_extra_gap(state);
        let visible_children = self
            .children
            .iter()
            .filter(|n| n.effective_visibility(state) != Visibility::Gone)
            .count();
        let fixed = fixed
            + if matches!(self.kind, Kind::Linear | Kind::RadioGroup | Kind::TableRow) {
                self.divider_count(state) as f32 * state.dp(self.divider_thickness)
                    + visible_children.saturating_sub(1) as f32 * extra_gap
            } else {
                0.0
            };
        let weight: f32 = self
            .children
            .iter()
            .filter(|n| n.effective_visibility(state) != Visibility::Gone)
            .map(|n| n.weight)
            .sum();
        let mut excess = size - fixed;
        let mut remaining_weight = if self.weight_sum > 0.0 {
            self.weight_sum
        } else {
            weight
        };
        let mut lengths = Vec::with_capacity(self.children.len());
        for (i, n) in self.children.iter().enumerate() {
            if n.effective_visibility(state) == Visibility::Gone {
                lengths.push(0.0);
                continue;
            }
            let (base, _) = measure_child_len(i, n);
            let share = if n.weight > 0.0 && remaining_weight.abs() > f32::EPSILON {
                excess * n.weight / remaining_weight
            } else {
                0.0
            };
            if n.weight > 0.0 {
                excess -= share;
                remaining_weight -= n.weight;
            }
            lengths.push((base + share).max(0.0));
        }
        lengths
    }
    pub(super) fn table_row_height(&self,row:&Node,columns:&[f32],state:&LayoutState,metrics:TextMetrics,id:u64,table_id:u64)->f32 {
        self.table_row_metrics(row,columns,state,metrics,id,table_id).0
    }
    pub(super) fn effective_text_all_caps(&self, state: &LayoutState) -> bool {
        if let Some(rule) = self.relative_rules.iter().find(|r| r.kind == RULE_TEXT_ALL_CAPS) {
            return rule.anchor.trim().eq_ignore_ascii_case("true");
        }
        if self.chip_style().is_some()
            || self.button_variant().is_some()
            || self.extended_fab_extended().is_some()
        {
            return self.text_all_caps;
        }
        self.text_all_caps
            || (matches!(self.kind, Kind::Button)
                && matches!(state.theme_style, AndroidTheme::MaterialLight | AndroidTheme::MaterialDark))
    }
    pub(super) fn display_text_for<'a>(&self, text: &'a str, state: &LayoutState) -> std::borrow::Cow<'a, str> {
        let text = if self.effective_text_all_caps(state) {
            std::borrow::Cow::Owned(text.to_uppercase())
        } else {
            std::borrow::Cow::Borrowed(text)
        };
        if self.single_line && (text.contains('\n') || text.contains('\r')) {
            std::borrow::Cow::Owned(text.replace(['\n', '\r'], " "))
        } else {
            text
        }
    }
    pub(super) fn scaled_text_size(&self, state: &LayoutState) -> f32 {
        state.sp(self.text_size)
    }
    pub(super) fn effective_text_font(&self, state: &LayoutState) -> (f32, u16, bool, FontFamily) {
        let (italic, family) = self.text_style_ext();
        if matches!(self.kind, Kind::Button)
            && (state.theme_style != AndroidTheme::Bhoo
                || self.button_variant().is_some()
                || self.chip_style().is_some()
                || self.extended_fab_extended().is_some())
        {
            let size = if (self.text_size - 16.0).abs() < 0.001 { 14.0 } else { self.text_size };
            let weight = if self.text_bold { 700 } else { 500 };
            (state.sp(size), weight, italic, family)
        } else {
            (state.sp(self.text_size), if self.text_bold { 700 } else { 400 }, italic, family)
        }
    }
    pub(super) fn uses_default_button_chrome(&self, state: &LayoutState) -> bool {
        matches!(self.kind, Kind::Button)
            && state.theme_style != AndroidTheme::Bhoo
            && self.chip_style().is_none()
            && self.extended_fab_extended().is_none()
            && self.button_variant().is_none()
            && self.background_stroke.is_none()
            && !self.has_explicit_custom_background()
    }
    pub(super) fn active_button_variant(&self) -> Option<XmlButtonVariant> {
        if self.chip_style().is_some()
            || self.extended_fab_extended().is_some()
            || self.has_explicit_custom_background()
        {
            return None;
        }
        self.button_variant()
    }
    pub(super) fn default_button_visual_bounds(&self, b: Rect, state: &LayoutState) -> Rect {
        let inset_4 = state.dp(4.0);
        if self.active_button_variant().is_some() {
            let inset_y = inset_4.min(b.height * 0.25);
            return Rect {
                x: b.x,
                y: b.y + inset_y,
                width: b.width,
                height: (b.height - inset_y * 2.0).max(0.0),
            };
        }
        match state.theme_style {
            AndroidTheme::MaterialLight | AndroidTheme::MaterialDark => {
                // AOSP dimens_material.xml: button_inset_horizontal_material=4dp, button_inset_vertical_material=6dp.
                let inset_x = inset_4.min(b.width * 0.25);
                let inset_y = state.dp(6.0).min(b.height * 0.25);
                Rect {
                    x: b.x + inset_x,
                    y: b.y + inset_y,
                    width: (b.width - inset_x * 2.0).max(0.0),
                    height: (b.height - inset_y * 2.0).max(0.0),
                }
            }
            AndroidTheme::Material3Light | AndroidTheme::Material3Dark => {
                let inset_y = inset_4.min(b.height * 0.25);
                Rect {
                    x: b.x,
                    y: b.y + inset_y,
                    width: b.width,
                    height: (b.height - inset_y * 2.0).max(0.0),
                }
            }
            AndroidTheme::Bhoo => b,
        }
    }
    pub(super) fn default_theme_padding(&self, state: &LayoutState) -> [f32; 4] {
        if self.alert_dialog_icon().is_some() {
            return [24.0; 4];
        }
        if self.is_floating_toolbar() {
            return [8.0; 4];
        }
        if self.is_snackbar() {
            let rtl = match self.layout_direction { 1 => false, 2 => true, _ => state.rtl };
            return if rtl { [8.0, 6.0, 16.0, 6.0] } else { [16.0, 6.0, 8.0, 6.0] };
        }
        if self.chip_style().is_some() {
            return [0.0; 4];
        }
        if state.snackbar_active && matches!(self.kind, Kind::Button) {
            return [12.0, 8.0, 12.0, 8.0];
        }
        if let Some(spec) = self.extended_fab_spec() {
            let extended = if !self.id.is_empty() {
                state.fab_extended.get(self.id).copied().unwrap_or(spec.static_extended)
            } else {
                spec.static_extended
            };
            if !extended {
                return [0.0; 4];
            }
            let rtl = match self.layout_direction { 1 => false, 2 => true, _ => state.rtl };
            return if rtl { [20.0, 0.0, 16.0, 0.0] } else { [16.0, 0.0, 20.0, 0.0] };
        }
        if let Some(variant) = self.active_button_variant() {
            return match variant {
                XmlButtonVariant::Text => [12.0, 10.0, 12.0, 10.0],
                _ => [24.0, 10.0, 24.0, 10.0],
            };
        }
        if !self.uses_default_button_chrome(state) {
            return [0.0; 4];
        }
        match state.theme_style {
            // AOSP btn_default_material: 4dp H / 6dp V inset + 8dp H / 4dp V padding.
            AndroidTheme::MaterialLight | AndroidTheme::MaterialDark => [12.0, 10.0, 12.0, 10.0],
            // Material 3 filled button: 24dp horizontal padding, 4dp inset + 6dp vertical padding.
            AndroidTheme::Material3Light | AndroidTheme::Material3Dark => [24.0, 10.0, 24.0, 10.0],
            AndroidTheme::Bhoo => [0.0; 4],
        }
    }
    pub(super) fn default_button_min_width(&self, state: &LayoutState) -> f32 {
        if self.chip_style().is_some() {
            return state.dp(48.0);
        }
        if state.snackbar_active {
            return state.dp(48.0);
        }
        if let Some(spec) = self.extended_fab_spec() {
            let extended = if !self.id.is_empty() {
                state.fab_extended.get(self.id).copied().unwrap_or(spec.static_extended)
            } else {
                spec.static_extended
            };
            return state.dp(if extended { 80.0_f32.max(spec.collapsed_size_dp) } else { spec.collapsed_size_dp });
        }
        if let Some(variant) = self.active_button_variant() {
            return state.dp(match variant {
                XmlButtonVariant::Text => 48.0,
                _ => 64.0,
            });
        }
        let raw = if !self.uses_default_button_chrome(state) {
            48.0
        } else {
            match state.theme_style {
                AndroidTheme::MaterialLight | AndroidTheme::MaterialDark => 88.0,
                AndroidTheme::Material3Light | AndroidTheme::Material3Dark => 64.0,
                AndroidTheme::Bhoo => 48.0,
            }
        };
        state.dp(raw)
    }
    pub(super) fn switch_layout(&self, state: &LayoutState)->crate::systems::material::SwitchLayout {
        let value=|kind,default|self.relative_rules.iter().find(|r|r.kind==kind).and_then(|r|r.anchor.parse::<f32>().ok()).filter(|v|v.is_finite()&&*v>=0.0).unwrap_or(default);
        crate::systems::material::SwitchLayout{min_width:state.dp(value(193,52.0)),padding:state.dp(value(194,4.0)),thumb_text_padding:state.dp(value(195,4.0))}
    }
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_edit_text(
        &self,
        f: &mut Frame<'_>,
        state: &LayoutState,
        text_rect: Rect,
        content: &str,
        color: Color,
        style: TextStyle,
        pad_x: f32,
        v_pad_top: f32,
        v_pad_bottom: f32,
        multiline: bool,
        links: Option<(TextLinksSpec, Color, bool, u64, &mut Option<String>, bool)>,
    ) {
        let inner_w = (text_rect.width - pad_x * 2.0).max(0.0);
        if inner_w <= 0.0 || text_rect.height <= 0.0 || content.is_empty() {
            return;
        }
        let mut links = links;
        let mut link_counter = 0usize;
        let underline_h = state.dp(1.0);
        let line_h = self.resolved_line_height(style.size, state);
        let line_advance = (line_h * self.line_spacing_multiplier + state.dp(self.line_spacing_extra)).max(1.0);
        let clip_pad_x = (pad_x - state.dp(2.0)).max(0.0);
        if multiline {
            let top_pad = if self.padding_explicit[1] {
                0.0
            } else {
                v_pad_top.min(text_rect.height * 0.25)
            };
            let bottom_pad = if self.padding_explicit[3] {
                0.0
            } else {
                v_pad_bottom.min(text_rect.height * 0.25)
            };
            let area = Rect {
                x: text_rect.x + pad_x,
                y: text_rect.y + top_pad,
                width: inner_w,
                height: (text_rect.height - top_pad - bottom_pad).max(0.0),
            };
            if area.height <= 0.0 {
                return;
            }
            let clip = Rect {
                x: text_rect.x + clip_pad_x,
                y: area.y,
                width: (text_rect.width - clip_pad_x * 2.0).max(0.0),
                height: area.height,
            }
            .intersect(f.clip);
            if clip.width <= 0.0 || clip.height <= 0.0 {
                return;
            }
            let metrics = f.ui.text_metrics;
            let line_style = TextStyle { padding: 0.0, ..style };
            let total = wrapped_text_lines(content, area.width, line_style, metrics, |_| {});
            let height_limit = ((area.height - line_h).max(0.0) / line_advance).ceil() as usize + 1;
            let limit = if self.max_lines == 0 {
                height_limit
            } else {
                height_limit.min(self.max_lines)
            };
            let shown = total.min(limit);
            let used_height = if shown == 0 {
                0.0
            } else {
                (line_h + (shown - 1) as f32 * line_advance).min(area.height)
            };
            let extra = (area.height - used_height).max(0.0);
            let top_align = self.content_gravity[1] == 0
                || self.min_lines > 1
                || text_rect.height >= line_h * 2.2;
            let start_y = if self.content_gravity[1] == 2 {
                area.y + extra
            } else if top_align {
                area.y
            } else {
                area.y + extra * 0.5
            };
            f.commands.push(DrawCommand::PushClip(clip));
            let mut line_index = 0usize;
            wrapped_text_lines(content, area.width, line_style, metrics, |line| {
                if line_index < limit {
                    let line_bounds = Rect {
                        x: area.x,
                        y: start_y + line_index as f32 * line_advance,
                        width: area.width,
                        height: line_h,
                    };
                    if !line.text.is_empty() {
                        if let Some((lspec, lcol, lenabled, lid, clicked_out, field_clicked)) = links.as_mut() {
                            paint_text_line_with_links(
                                f,
                                line_bounds,
                                &line.text,
                                color,
                                *lcol,
                                line_style,
                                metrics,
                                lspec.mask,
                                lspec.clickable,
                                *lenabled,
                                *lid,
                                &mut link_counter,
                                clicked_out,
                                underline_h,
                                *field_clicked,
                            );
                        } else {
                            f.styled_text(line_bounds, &line.text, color, line_style);
                        }
                    }
                }
                line_index += 1;
            });
            f.commands.push(DrawCommand::PopClip);
        } else {
            let clip = Rect {
                x: text_rect.x + clip_pad_x,
                y: text_rect.y,
                width: (text_rect.width - clip_pad_x * 2.0).max(0.0),
                height: text_rect.height,
            }
            .intersect(f.clip);
            if clip.width <= 0.0 || clip.height <= 0.0 {
                return;
            }
            let single = if content.contains(['\n', '\r']) {
                std::borrow::Cow::Owned(content.replace(['\n', '\r'], " "))
            } else {
                std::borrow::Cow::Borrowed(content)
            };
            let metrics = f.ui.text_metrics;
            f.commands.push(DrawCommand::PushClip(clip));
            if let Some((lspec, lcol, lenabled, lid, clicked_out, field_clicked)) = links.as_mut() {
                paint_text_line_with_links(
                    f,
                    text_rect,
                    &single,
                    color,
                    *lcol,
                    style,
                    metrics,
                    lspec.mask,
                    lspec.clickable,
                    *lenabled,
                    *lid,
                    &mut link_counter,
                    clicked_out,
                    underline_h,
                    *field_clicked,
                );
            } else {
                f.styled_text(text_rect, &single, color, style);
            }
            f.commands.push(DrawCommand::PopClip);
        }
    }
    pub(super) fn layout_intrinsic(&self, horizontal: bool, state: &LayoutState, available_width: f32, metrics: TextMetrics, id:u64) -> f32 {
        self.layout_intrinsic_for_width(horizontal, state, available_width, false, metrics, id)
    }
    pub(super) fn layout_intrinsic_for_width(&self, horizontal: bool, state: &LayoutState, available_width: f32, exact_width: bool, metrics: TextMetrics, id:u64) -> f32 {
        let rtl=match self.layout_direction {1=>false,2=>true,_=>state.rtl};
        if rtl!=state.rtl { let mut scoped=state.clone();scoped.rtl=rtl;return self.layout_intrinsic_for_width(horizontal,&scoped,available_width,exact_width,metrics,id); }
        if !horizontal && let Some(ch) = self.collapsing_toolbar_height(state, id) {
            return ch.max(state.dp(self.min_height));
        }
        let d = state.effective_density();
        let min_w = state.dp(self.min_width);
        let min_h = state.dp(self.min_height);
        let bg_min_w = state.dp(self.background_min_size[0]);
        let bg_min_h = state.dp(self.background_min_size[1]);
        if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::Linear|Kind::RadioGroup|Kind::TableRow) {
            if self.nav_bar_spec().is_some() {
                return self.intrinsic(horizontal, state, id);
            }
            let padding=self.resolved_padding(state);
            let inner=(available_width-padding[0]-padding[2]).max(0.0);
            if let Some(cg) = self.chip_group_wrap_spec() {
                let (_, max_row_w, _) = self.chip_group_wrap_rects(
                    Rect { x: 0.0, y: 0.0, width: inner, height: 100_000.0 },
                    cg,
                    state,
                    metrics,
                    id,
                );
                return (max_row_w + padding[0] + padding[2]).max(min_w).max(bg_min_w);
            }
            let flow=self.horizontal || matches!(self.kind,Kind::TableRow);
            let visible_count = self.children.iter().filter(|n| n.effective_visibility(state) != Visibility::Gone).count();
            let extra_gap = if flow { visible_count.saturating_sub(1) as f32 * self.linear_extra_gap(state) } else { 0.0 };
            let sizes=self.children.iter().enumerate().filter(|(_,n)|n.effective_visibility(state)!=Visibility::Gone).map(|(i,n)| {
                let m=n.resolved_margin(state);let available=(inner-m[0]-m[2]).max(0.0);
                n.layout_intrinsic(true,state,available,metrics,child_view_id(id,i,n)).min(available)+m[0]+m[2]
            });
            let content=if flow {sizes.sum::<f32>()+extra_gap+self.divider_count(state) as f32*state.dp(self.divider_thickness)}else{sizes.fold(0.0_f32,f32::max)};
            (content+padding[0]+padding[2]).max(min_w).max(bg_min_w)
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::Table) {
            let padding=self.resolved_padding(state);let inner=(available_width-padding[0]-padding[2]).max(0.0);
            let columns=self.table_columns(state,metrics,id).iter().sum::<f32>();
            let other=self.children.iter().enumerate().filter(|(_,n)|!matches!(n.kind,Kind::TableRow)&&n.effective_visibility(state)!=Visibility::Gone).map(|(i,n)| {
                let m=n.resolved_margin(state);let available=(inner-m[0]-m[2]).max(0.0);
                n.layout_intrinsic(true,state,available,metrics,child_view_id(id,i,n)).min(available)+m[0]+m[2]
            }).fold(0.0_f32,f32::max);
            (columns.max(other)+padding[0]+padding[2]).max(min_w).max(bg_min_w)
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::Grid) {
            let padding=self.resolved_padding(state);let inner=(available_width-padding[0]-padding[2]).max(0.0);
            let (widths,_,_,_,_)=self.grid_tracks(state,metrics,id,inner,false,None);
            (widths.iter().sum::<f32>()+padding[0]+padding[2]).max(min_w).max(bg_min_w)
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::Relative|Kind::DialerFilter|Kind::ConstraintLayout) {
            let padding=self.resolved_padding(state);
            let inner=Rect{x:0.0,y:0.0,width:(available_width-padding[0]-padding[2]).max(0.0),height:1_000_000.0};
            let is_cl = matches!(self.kind,Kind::ConstraintLayout);
            let ph_claimed = if is_cl { Some(resolve_constraint_placeholders(self.children, state).1) } else { None };
            let visibilities = if is_cl { Some(constraint_group_visibilities(self.children, state)) } else { None };
            let positions=if is_cl { constraint_layout_positions(self,id,inner,state,metrics,true,true) } else { relative_layout::layout(self,id,inner,state,metrics,true,true) };
            let content=positions.iter().enumerate().filter(|(i,_)| {
                let vis = visibilities.as_ref().map_or_else(|| self.children[*i].effective_visibility(state), |v| v[*i]);
                vis != Visibility::Gone
                    && !(is_cl && is_constraint_helper(&self.children[*i]))
                    && !ph_claimed.as_ref().is_some_and(|m| m[*i].is_some())
            }).map(|(i,r)| {
                let m=self.children[i].resolved_margin(state);
                if state.rtl { (inner.x+inner.width)-(r.x-m[0]) } else { r.x+r.width+m[2] }
            }).fold(0.0_f32,f32::max);
            (content+padding[0]+padding[2]).max(min_w).max(bg_min_w)
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::Frame|Kind::MediaController|Kind::MaterialCard|Kind::ViewAnimator|Kind::ViewSwitcher|Kind::TextSwitcher|Kind::ImageSwitcher|Kind::ViewFlipper) {
            let padding=self.resolved_padding(state);let inner=(available_width-padding[0]-padding[2]).max(0.0);
            let content=self.children.iter().enumerate().filter(|(_,n)|(self.measure_all_children||n.effective_visibility(state)!=Visibility::Gone)&&!self.drawer_panel_is_closed(n, state, id)).map(|(i,n)| {
                let m=n.resolved_margin(state);let available=(inner-m[0]-m[2]).max(0.0);
                n.layout_intrinsic(true,state,available,metrics,child_view_id(id,i,n)).min(available)+m[0]+m[2]
            }).fold(0.0_f32,f32::max);
            (content+padding[0]+padding[2]).max(min_w).max(bg_min_w).max(state.dp(self.foreground_size[0]))
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::Switch) {
            let layout=self.switch_layout(state);
            let texts=if self.relative_rules.iter().any(|r|r.kind==161){Some((self.text_on,self.text_off))}else{None};
            let (_,track)=crate::systems::material::switch_widths(texts,metrics,layout);
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
            let label=self.display_text_for(state.text.get(&key).map_or(self.text,String::as_str), state);
            let text_size = self.scaled_text_size(state);
            let (italic, family) = self.text_style_ext();
            let style=TextStyle{size:text_size,weight:if self.text_bold{700}else{400},letter_spacing:self.letter_spacing_em*text_size,italic,family,..TextStyle::default()};
            let padding=self.resolved_padding(state);
            (track+(if label.is_empty(){0.0}else{layout.padding+metrics.width(&label,style)})+padding[0]+padding[2]).max(min_w).max(bg_min_w)
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::Button) {
            if let Some((spec, m, _)) = self.extended_fab_progress(state, id) {
                let collapsed_w = state.dp(spec.collapsed_size_dp).max(min_w).max(bg_min_w);
                let extended_w = if spec.strategy == ExtendedFabStrategy::MatchParent
                    && available_width.is_finite()
                    && available_width > 0.0
                {
                    available_width.max(collapsed_w)
                } else {
                    self.extended_fab_intrinsic_extended_width(state, metrics, id, spec)
                        .max(collapsed_w)
                };
                return collapsed_w + (extended_w - collapsed_w) * m.progress;
            }
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
            let label=self.display_text_for(state.text.get(&key).map_or(self.text,String::as_str), state);
            let (text_size, weight, italic, family) = self.effective_text_font(state);
            let style=TextStyle{size:text_size,weight,letter_spacing:self.letter_spacing_em*text_size,italic,family,..TextStyle::default()};
            let width=label.split(['\n','\r','\u{000b}','\u{000c}','\u{0085}','\u{2028}','\u{2029}']).map(|line|metrics.width(line,style)).fold(0.0_f32,f32::max);
            let padding=self.resolved_padding(state);
            if let Some(chip) = self.chip_style() {
                let selected = state.checked.get(&key).copied().unwrap_or(self.checked)
                    || state.view_selected.get(&key).copied().unwrap_or(false);
                let lead = if selected && chip.kind == XmlChipKind::Filter { state.dp(34.0) } else { state.dp(16.0) };
                let trail = if chip.close_visible { state.dp(34.0) } else { state.dp(16.0) };
                (width + lead + trail + padding[0] + padding[2]).max(self.default_button_min_width(state)).max(min_w).max(bg_min_w)
            } else {
                let shapes=self.compound_sizes(state,state.rtl);let extent=self.compound_extents(state,state.rtl);
                ((width+extent[0]+extent[2]).max(shapes[1].map_or(0.0,|s|s.0)).max(shapes[3].map_or(0.0,|s|s.0))+padding[0]+padding[2]).max(self.default_button_min_width(state)).max(min_w).max(bg_min_w)
            }
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::ToggleButton) {
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
            let checked=state.checked.get(&key).copied().unwrap_or(self.checked);
            let label=self.display_text_for(if checked{self.text_on}else{self.text_off}, state);
            let text_size = self.scaled_text_size(state);
            let (italic, family) = self.text_style_ext();
            let style=TextStyle{size:text_size,weight:if self.text_bold{700}else{400},letter_spacing:self.letter_spacing_em*text_size,italic,family,..TextStyle::default()};
            let padding=self.resolved_padding(state);
            (metrics.width(&label,style)+padding[0]+padding[2]).max(state.dp(48.0)).max(min_w).max(bg_min_w)
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::CheckBox|Kind::RadioButton) {
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
            let label=self.display_text_for(state.text.get(&key).map_or(self.text,String::as_str), state);
            let text_size = self.scaled_text_size(state);
            let (italic, family) = self.text_style_ext();
            let style=TextStyle{size:text_size,weight:if self.text_bold{700}else{400},letter_spacing:self.letter_spacing_em*text_size,italic,family,..TextStyle::default()};
            let padding=self.resolved_padding(state);
            (state.dp(48.0)+metrics.width(&label,style)+padding[0]+padding[2]).max(min_w).max(bg_min_w)
        } else if horizontal && matches!(self.width,Size::Wrap) && matches!(self.kind,Kind::Text|Kind::CheckedText|Kind::Chronometer|Kind::TextClock) {
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
            let text=self.display_text_for(state.text.get(&key).map_or(self.text,String::as_str), state);
            let text_size = self.scaled_text_size(state);
            let (italic, family) = self.text_style_ext();
            let style=TextStyle{size:text_size,weight:if self.text_bold{700}else{400},letter_spacing:self.letter_spacing_em*text_size,italic,family,..TextStyle::default()};
            let width=text.split(['\n','\r','\u{000b}','\u{000c}','\u{0085}','\u{2028}','\u{2029}']).map(|line|metrics.width(line,style)).fold(0.0_f32,f32::max);
            let shapes=if matches!(self.kind,Kind::Text|Kind::CheckedText){self.compound_sizes(state,state.rtl)}else{[None;4]};
            let ext=if matches!(self.kind,Kind::Text|Kind::CheckedText){self.compound_extents(state,state.rtl)}else{[0.0;4]};
            let mark=if matches!(self.kind,Kind::CheckedText){self.check_mark_width(state)}else{0.0};
            let padding=self.resolved_padding(state);
            ((width+mark+ext[0]+ext[2]).max(shapes[1].map_or(0.0,|shape|shape.0)).max(shapes[3].map_or(0.0,|shape|shape.0))+padding[0]+padding[2]).max(min_w).max(bg_min_w)
        } else if matches!(self.kind,Kind::VideoView) && (horizontal || !matches!(self.height,Size::Match)) {
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};let size=state.video_dimensions.get(&key).copied().or_else(||state.video_frames.get(&key).map(|frame|(frame.width(),frame.height()))).unwrap_or((0,0));
            let width=if exact_width {VideoMeasureSpec::Exactly(available_width.max(0.0) as u32)}else{match self.width {Size::Dp(value)=>VideoMeasureSpec::Exactly(state.dp(value).max(0.0) as u32),Size::Match=>VideoMeasureSpec::Exactly(available_width.max(0.0) as u32),Size::Wrap=>VideoMeasureSpec::AtMost(available_width.max(0.0) as u32)}};
            let height=match self.height {Size::Dp(value)=>VideoMeasureSpec::Exactly(state.dp(value).max(0.0) as u32),_=>VideoMeasureSpec::Unspecified};let measured=measure_video_view(size,width,height);
            if horizontal {measured.0 as f32}else{measured.1 as f32}
        } else if !horizontal && matches!(self.kind,Kind::GridView) && matches!(self.height,Size::Wrap) {
            if let Some(ext) = self.recycler_layout_ext() {
                let padding = self.resolved_view_padding(state);
                let v_gap = self.relative_rules.iter().find(|r| r.kind == 94).and_then(|r| r.anchor.parse::<f32>().ok()).unwrap_or(0.0).max(0.0) * d;
                if ext.horizontal {
                    let span = ext.span_count.max(1);
                    let total = span as f32 * state.dp(48.0) + span.saturating_sub(1) as f32 * v_gap;
                    return (total + padding[1] + padding[3]).max(min_h).max(bg_min_h);
                }
            }
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};let padding=self.resolved_view_padding(state);
            let width=(available_width-padding[0]-padding[2]).max(0.0);let (columns,column_width,_,_)=grid_view_columns(self,width,d);
            let spacing=state.dp(self.relative_rules.iter().find(|r|r.kind==94).and_then(|r|r.anchor.parse::<f32>().ok()).unwrap_or(0.0).max(0.0));
            let entries=state.list_items.get(&key).map(Vec::as_slice).unwrap_or(&[]);let template=state.list_templates.get(&key).copied();let mut row_state=if template.is_some() || state.list_item_bindings.contains_key(&key) {Some(spinner_template_state(state,rtl))}else{None};
            let mut total=0.0;for (row,items) in entries.chunks(columns).enumerate() {
                let mut height=0.0_f32;for (column,label) in items.iter().enumerate() {
                    let (_,h)=grid_item_size(template,&mut row_state,label,column_width,metrics,anonymous_child_id(id,1024+row*columns+column),state.list_item_bindings.get(&key).and_then(|items|items.get(&(row*columns+column))));
                    height=height.max(h);
                }
                if row>0 {total+=spacing;}total+=height;
            }
            (total+padding[1]+padding[3]).max(min_h).max(bg_min_h)
        } else if !horizontal && matches!(self.kind,Kind::ListView) && matches!(self.height,Size::Wrap) {
            if let Some(ext) = self.recycler_layout_ext()
                && ext.horizontal
            {
                let padding = self.resolved_view_padding(state);
                let v_gap = self.relative_rules.iter().find(|r| r.kind == 94).and_then(|r| r.anchor.parse::<f32>().ok()).unwrap_or(0.0).max(0.0) * d;
                let span = ext.span_count.max(1);
                let total = span as f32 * state.dp(48.0) + span.saturating_sub(1) as f32 * v_gap;
                return (total + padding[1] + padding[3]).max(min_h).max(bg_min_h);
            }
            let key=if self.id.is_empty(){format!("#{id}")} else {self.id.to_owned()};
            let template=state.list_templates.get(&key).copied();let mut row_state=spinner_template_state(state,rtl);
            let entries:Vec<&str>=match state.list_items.get(&key) {Some(items)=>items.iter().map(String::as_str).collect(),None=>self.relative_rules.iter().filter(|rule|rule.kind==82).map(|rule|rule.anchor).collect()};
            let padding=self.resolved_view_padding(state);let width=(available_width-padding[0]-padding[2]).max(0.0);let divider=state.dp(self.divider_thickness.max(0.0));
            let (headers,footers)=state.list_fixed_views.get(&key).cloned().unwrap_or_default();
            let header_dividers=!self.relative_rules.iter().any(|r|r.kind==89 && r.anchor=="false");
            let footer_dividers=!self.relative_rules.iter().any(|r|r.kind==90 && r.anchor=="false");
            let mut total=0.0;
            for (i,fixed) in headers.iter().enumerate() {total+=list_fixed_height(fixed.view,state,width,metrics,anonymous_child_id(id,2048+i));if header_dividers && (i+1<headers.len() || !entries.is_empty() || !footers.is_empty()) {total+=divider;}}
            for (i,label) in entries.iter().enumerate() {
                let binding=state.list_item_bindings.get(&key).and_then(|items|items.get(&i));
                let height=if let Some(template)=adapter_item_template(template,binding) {with_adapter_binding(&mut row_state,template.text_id,label,binding,|row_state|match template.row.height {Size::Match=>state.dp(48.0),_=>template.row.layout_intrinsic(false,row_state,width,metrics,anonymous_child_id(id,1024+i))})} else {(wrapped_text_lines(label,(width-32.0).max(1.0),TextStyle::default(),metrics,|_|{}).max(1) as f32*24.0).max(48.0)};
                total+=if height.is_finite(){height.clamp(1.0,4096.0)}else{state.dp(48.0)};
                if i+1<entries.len() || (!footers.is_empty() && footer_dividers) {total+=divider;}
            }
            for (i,fixed) in footers.iter().enumerate() {total+=list_fixed_height(fixed.view,state,width,metrics,anonymous_child_id(id,3072+i));if footer_dividers && i+1<footers.len(){total+=divider;}}
            (total+padding[1]+padding[3]).max(min_h).max(bg_min_h)
        } else if !horizontal && matches!(self.kind,Kind::Spinner) && matches!(self.height,Size::Wrap) {
            let key=if self.id.is_empty() {format!("#{id}")} else {self.id.to_owned()};
            let selected=state.spinner_selected.get(&key).copied().unwrap_or(0);
            let label=match state.spinner_items.get(&key) {
                Some(items)=>items.get(selected.min(items.len().saturating_sub(1))).map(String::as_str),
                None=>self.relative_rules.iter().filter(|rule|rule.kind==82).nth(selected).or_else(||self.relative_rules.iter().filter(|rule|rule.kind==82).last()).map(|rule|rule.anchor),
            };
            if let (Some(template),Some(label))=(state.spinner_templates.get(&key),label) {
                let mut row_state=spinner_template_state(state,rtl);
                row_state.text.insert(template.text_id.to_owned(),label.to_owned());
                let height=match template.selected.height {
                    Size::Match=>state.dp(48.0),
                    _=>template.selected.layout_intrinsic(false,&row_state,(available_width-state.dp(40.0)).max(0.0),metrics,anonymous_child_id(id,1023)),
                };
                let padding=self.resolved_view_padding(state);
                (height+padding[1]+padding[3]).max(min_h).max(bg_min_h)
            } else {self.intrinsic(horizontal,state,id)}
        } else if !horizontal && matches!(self.kind,Kind::Button) && matches!(self.height,Size::Wrap) {
            if let Some((spec, m, _)) = self.extended_fab_progress(state, id) {
                let collapsed_h = state.dp(spec.collapsed_size_dp).max(min_h).max(bg_min_h);
                if m.progress <= 0.001 {
                    return collapsed_h;
                }
                let ext_h = state.dp(56.0).max(min_h).max(bg_min_h);
                return collapsed_h + (ext_h - collapsed_h) * m.progress;
            }
            let shapes=self.compound_sizes(state,state.rtl);let extent=self.compound_extents(state,state.rtl);let padding=self.resolved_padding(state);
            let key=if self.id.is_empty(){format!("#{id}")}else{self.id.to_owned()};
            let label=self.display_text_for(state.text.get(&key).map_or(self.text,String::as_str), state);
            let (base_text_size, weight, italic, family) = self.effective_text_font(state);
            let width=if exact_width {available_width}else{match self.width {Size::Dp(value)=>state.dp(value).min(available_width),_=>available_width}};
            let text_width=(width-padding[0]-padding[2]-extent[0]-extent[2]).max(1.0);
            let layout_ext = self.text_layout_ext();
            let base_style=TextStyle{size:base_text_size,weight,letter_spacing:self.letter_spacing_em*base_text_size,italic,family,..TextStyle::default()};
            let text_size = if let Some(ext) = layout_ext && ext.has_auto_size() {
                self.resolve_auto_text_size(&label, text_width, f32::INFINITY, base_style, ext, state, metrics)
            } else {
                base_text_size
            };
            let style=TextStyle{size:text_size,weight,letter_spacing:self.letter_spacing_em*text_size,italic,family,..TextStyle::default()};
            let single=self.single_line || (self.max_lines==1 && (self.ellipsize_side!=0||self.ellipsize_end));
            let lines=if single {1} else if let Some(ext) = layout_ext {wrap_text_lines_ext(&label,text_width+0.01,style,metrics,ext).len().max(1)} else {wrapped_text_lines(&label,text_width+0.01,style,metrics,|_|{}).max(1)};
            let lines=if self.max_lines==0 {lines}else{lines.min(self.max_lines)}.max(self.min_lines.max(1));
            let line_height=self.resolved_line_height(text_size,state);let advance=(line_height*self.line_spacing_multiplier+state.dp(self.line_spacing_extra)).max(1.0);
            let content=(line_height+(lines-1) as f32*advance).max(shapes[0].map_or(0.0,|s|s.1)).max(shapes[2].map_or(0.0,|s|s.1));
            let min_btn_h = if self.extended_fab_extended().is_some() { state.dp(56.0) } else { state.dp(48.0) };
            (content+extent[1]+extent[3]+padding[1]+padding[3]).max(min_btn_h).max(min_h).max(bg_min_h)
        } else if !horizontal && matches!(self.kind,Kind::Grid) && matches!(self.height,Size::Wrap) {
            let padding=self.resolved_padding(state);let width=if exact_width {available_width}else{match self.width {Size::Dp(w)=>state.dp(w).min(available_width),_=>available_width}};
            let (_,heights,_,_,_)=self.grid_tracks(state,metrics,id,(width-padding[0]-padding[2]).max(0.0),exact_width||!matches!(self.width,Size::Wrap),None);
            (heights.iter().sum::<f32>()+padding[1]+padding[3]).max(min_h).max(bg_min_h)
        } else if !horizontal && matches!(self.kind,Kind::TableRow) && matches!(self.height,Size::Wrap) {
            let widths:Vec<f32>=self.children.iter().enumerate().map(|(i,n)|n.intrinsic(true,state,child_view_id(id,i,n))).collect();
            self.standalone_row_metrics(state,metrics,id,&widths).0.max(min_h).max(bg_min_h)
        } else if !horizontal && matches!(self.kind,Kind::Table) && matches!(self.height,Size::Wrap) {
            let padding=self.resolved_padding(state);let columns=self.table_columns_for_width(state,metrics,id,(available_width-padding[0]-padding[2]).max(0.0));
            let height=self.children.iter().enumerate().filter(|(_,n)|n.effective_visibility(state)!=Visibility::Gone).map(|(i,n)| {
                let child=child_view_id(id,i,n);
                (if matches!(n.kind,Kind::TableRow) {self.table_row_height(n,&columns,state,metrics,child,id)} else {n.layout_intrinsic(false,state,available_width,metrics,child)})+n.margin_for_axis(false,state)
            }).sum::<f32>();
            (height+padding[1]+padding[3]).max(min_h).max(bg_min_h)
        } else if !horizontal && matches!(self.kind,Kind::Relative|Kind::DialerFilter|Kind::ConstraintLayout) && matches!(self.height,Size::Wrap) {
            let padding=self.resolved_padding(state);
            let wrap_w = !exact_width && matches!(self.width, Size::Wrap);
            let width=if exact_width {available_width}else{match self.width {Size::Dp(w)=>state.dp(w).min(available_width),_=>available_width}};
            let inner=Rect {x:0.0,y:0.0,width:(width-padding[0]-padding[2]).max(0.0),height:1000000.0};
            let is_cl = matches!(self.kind,Kind::ConstraintLayout);
            let ph_claimed = if is_cl { Some(resolve_constraint_placeholders(self.children, state).1) } else { None };
            let visibilities = if is_cl { Some(constraint_group_visibilities(self.children, state)) } else { None };
            let positions=if is_cl { constraint_layout_positions(self,id,inner,state,metrics,wrap_w,true) } else { relative_layout::layout(self,id,inner,state,metrics,wrap_w,true) };
            let height=positions.iter().enumerate().filter(|(i,_)| {
                let vis = visibilities.as_ref().map_or_else(|| self.children[*i].effective_visibility(state), |v| v[*i]);
                vis != Visibility::Gone
                    && !(is_cl && is_constraint_helper(&self.children[*i]))
                    && !ph_claimed.as_ref().is_some_and(|m| m[*i].is_some())
            }).map(|(i,r)|r.y+r.height+self.children[i].resolved_margin(state)[3]).fold(0.0_f32,f32::max);
            (height+padding[1]+padding[3]).max(min_h).max(bg_min_h)
        } else if !horizontal && self.image_adjust_bounds && matches!(self.kind,Kind::Image|Kind::QuickContactBadge|Kind::Icon|Kind::Zoom) && matches!(self.height,Size::Wrap) {
            let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
            let source=self.resolved_intrinsic_image_src(state,&key);
            if let Some(image)=state.images.get(source) {
                let padding=self.resolved_padding(state);
                let width=if exact_width {
                    available_width
                } else {
                    match self.width {
                        Size::Dp(w)=>state.dp(w).min(available_width),
                        Size::Wrap=>self.intrinsic(true,state,id).min(available_width),
                        Size::Match=>available_width,
                    }
                };
                let height=(width-padding[0]-padding[2]).max(0.0)*image.height() as f32/image.width() as f32+padding[1]+padding[3];
                let height=height.max(min_h).max(bg_min_h);
                return self.image_max_height.map_or(height,|maximum|height.min(state.dp(maximum)));
            }
            self.intrinsic(horizontal,state,id)
        } else if !horizontal && matches!(self.kind, Kind::Text | Kind::CheckedText | Kind::Chronometer | Kind::TextClock) && matches!(self.height, Size::Wrap) {
            let width = if exact_width { available_width } else { match self.width { Size::Dp(width) => state.dp(width).min(available_width), _ => available_width } };
            let key=if self.id.is_empty() { format!("#{id}") } else { self.id.to_string() };
            let text = self.display_text_for(state.text.get(&key).map_or(self.text, String::as_str), state);
            let base_text_size = self.scaled_text_size(state);
            let (italic, family) = self.text_style_ext();
            let padding = self.resolved_padding(state);
            let mark_width = if matches!(self.kind, Kind::CheckedText) { self.check_mark_width(state) } else { 0.0 };
            let ext=if matches!(self.kind,Kind::Text|Kind::CheckedText){self.compound_extents(state,state.rtl)}else{[0.0;4]};
            let shapes=if matches!(self.kind,Kind::Text|Kind::CheckedText){self.compound_sizes(state,state.rtl)}else{[None;4]};
            let text_width = (width - padding[0] - padding[2] - mark_width - ext[0] - ext[2]).max(1.0);
            let layout_ext = self.text_layout_ext();
            let base_style = TextStyle { size: base_text_size, weight: if self.text_bold { 700 } else { 400 }, align: self.align, letter_spacing:self.letter_spacing_em*base_text_size, italic, family, ..TextStyle::default() };
            let text_size = if let Some(lext) = layout_ext && lext.has_auto_size() {
                self.resolve_auto_text_size(&text, text_width, f32::INFINITY, base_style, lext, state, metrics)
            } else {
                base_text_size
            };
            let style = TextStyle { size: text_size, weight: if self.text_bold { 700 } else { 400 }, align: self.align, letter_spacing:self.letter_spacing_em*text_size, italic, family, ..TextStyle::default() };
            let lines = if self.single_line { 1 } else if let Some(lext) = layout_ext { wrap_text_lines_ext(&text, text_width, style, metrics, lext).len().max(1) } else { wrapped_text_lines(&text, text_width, style, metrics, |_| {}) };
            let lines = if self.max_lines == 0 { lines } else { lines.min(self.max_lines) };
            let lines = lines.max(self.min_lines.max(1));
            let lh = self.resolved_line_height(text_size, state);
            ((lh + (lines-1) as f32*(lh*self.line_spacing_multiplier+state.dp(self.line_spacing_extra)).max(1.0)).max(shapes[0].map_or(0.0,|s|s.1)).max(shapes[2].map_or(0.0,|s|s.1))+ext[1]+ext[3]+padding[1]+padding[3]).max(min_h).max(bg_min_h)
        } else if !horizontal && matches!(self.height, Size::Wrap) && matches!(self.kind, Kind::Linear | Kind::RadioGroup | Kind::TableRow | Kind::Frame | Kind::MediaController | Kind::MaterialCard | Kind::Absolute | Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper) {
            if let Some(nav) = self.nav_bar_spec() {
                if nav.kind == NavBarKind::Drawer && !self.children.is_empty() {
                    let width = if exact_width {
                        available_width
                    } else {
                        match self.width {
                            Size::Dp(w) => state.dp(w).min(available_width),
                            _ => available_width,
                        }
                    };
                    let padding = self.resolved_padding(state);
                    let inner_w = (width - padding[0] - padding[2]).max(0.0);
                    let mut header_h = 0.0_f32;
                    let mut has_header = false;
                    for (i, child) in self.children.iter().enumerate() {
                        if child.effective_visibility(state) == Visibility::Gone {
                            continue;
                        }
                        has_header = true;
                        let m = child.resolved_margin(state);
                        let avail_w = (inner_w - m[0] - m[2]).max(0.0);
                        let cid = child_view_id(id, i, child);
                        let ch = match child.height {
                            Size::Dp(h) => state.dp(h),
                            _ => child.layout_intrinsic(false, state, avail_w, metrics, cid),
                        };
                        header_h += ch + m[1] + m[3];
                    }
                    if has_header {
                        header_h += state.dp(1.0);
                    }
                    let count = self.nav_items().len().max(1);
                    return (header_h + count as f32 * state.dp(56.0) + state.dp(24.0) + padding[1] + padding[3])
                        .max(min_h)
                        .max(bg_min_h);
                }
                return self.intrinsic(horizontal, state, id);
            }
            let width = if exact_width { available_width } else { match self.width { Size::Dp(width) => state.dp(width).min(available_width), _ => available_width } };
            let padding = self.resolved_padding(state);
            let inner_width = (width - padding[0] - padding[2]).max(0.0);
            let min_h = if self.is_snackbar() {
                min_h.max(state.dp(48.0))
            } else if self.is_floating_toolbar() {
                min_h.max(state.dp(64.0))
            } else {
                min_h
            };
            if let Some(cg) = self.chip_group_wrap_spec() {
                let (_, _, total_h) = self.chip_group_wrap_rects(
                    Rect { x: 0.0, y: 0.0, width: inner_width, height: 100_000.0 },
                    cg,
                    state,
                    metrics,
                    id,
                );
                return (total_h + padding[1] + padding[3]).max(min_h).max(bg_min_h);
            }
            let appbar_parent_scroll = if !self.horizontal
                && matches!(self.kind, Kind::Linear)
                && self.children.iter().any(|c| c.app_bar_scroll_flags().is_some())
            {
                Some(self.app_bar_scroll_px(state, id))
            } else {
                None
            };
            let horizontal_weights = if self.horizontal
                && matches!(self.kind, Kind::Linear | Kind::RadioGroup)
                && inner_width > 0.0
                && self.children.iter().any(|n| n.effective_visibility(state) != Visibility::Gone && n.weight > 0.0)
            {
                Some(self.linear_main_lengths(true, state, inner_width, inner_width, metrics, id))
            } else {
                None
            };
            let dimensions = self.children.iter().enumerate().filter(|(_,n)| (self.measure_all_children || n.effective_visibility(state) != Visibility::Gone) && !self.drawer_panel_is_closed(n, state, id)).map(|(i,n)| {
                let margin = if matches!(self.kind,Kind::Absolute) { [0.0;4] } else { n.resolved_margin(state) };
                let (child_avail_w, child_exact_w) = if n.effective_visibility(state) != Visibility::Gone
                    && let Some(ref widths) = horizontal_weights
                {
                    (widths[i], n.weight > 0.0)
                } else {
                    ((inner_width - margin[0] - margin[2]).max(0.0), false)
                };
                let cid = child_view_id(id, i, n);
                let child_h = n.layout_intrinsic_for_width(false, state, child_avail_w, child_exact_w, metrics, cid);
                let v_margin = if matches!(self.kind,Kind::Absolute) { state.dp(n.absolute_position[1]) } else { margin[1] + margin[3] };
                if let Some(parent_scroll) = appbar_parent_scroll
                    && n.app_bar_scroll_flags().is_some()
                {
                    let sp = n.app_bar_child_scroll_px(parent_scroll, state, cid);
                    let collapse = preview_app_bar_child_collapse(n, cid, child_h, v_margin, sp, state);
                    (child_h + v_margin - collapse).max(0.0)
                } else {
                    child_h + v_margin
                }
            });
            let content = if matches!(self.kind, Kind::Frame | Kind::MediaController | Kind::MaterialCard | Kind::Absolute | Kind::ViewAnimator | Kind::ViewSwitcher | Kind::TextSwitcher | Kind::ImageSwitcher | Kind::ViewFlipper) || self.horizontal || matches!(self.kind,Kind::TableRow) { dimensions.fold(0.0_f32, f32::max) } else { dimensions.sum() };
            let content = if matches!(self.kind, Kind::Frame) && self.children.iter().any(|n| n.relative_rules.iter().any(|r| r.kind == RULE_COORDINATOR_SCROLL_BEHAVIOR)) {
                let mut combined = content;
                for (i, n) in self.children.iter().enumerate() {
                    if n.effective_visibility(state) == Visibility::Gone || !n.relative_rules.iter().any(|r| r.kind == RULE_COORDINATOR_SCROLL_BEHAVIOR) {
                        continue;
                    }
                    let appbar_h = (0..i).find_map(|j| {
                        let sib = &self.children[j];
                        if sib.effective_visibility(state) == Visibility::Gone
                            || (!matches!(sib.kind, Kind::Linear | Kind::Toolbar)
                                && sib.collapsing_toolbar_spec().is_none())
                            || sib.layout_gravity[1] == 2
                            || sib.relative_rules.iter().any(|r| matches!(r.kind, RULE_COORDINATOR_SCROLL_BEHAVIOR | RULE_COORDINATOR_ANCHOR))
                        {
                            return None;
                        }
                        let m = sib.resolved_margin(state);
                        Some(sib.layout_intrinsic(false, state, (inner_width - m[0] - m[2]).max(0.0), metrics, child_view_id(id, j, sib)) + m[1] + m[3])
                    }).unwrap_or(0.0);
                    let m = n.resolved_margin(state);
                    let child_h = n.layout_intrinsic(false, state, (inner_width - m[0] - m[2]).max(0.0), metrics, child_view_id(id, i, n)) + m[1] + m[3];
                    combined = combined.max(appbar_h + child_h);
                }
                combined
            } else {
                content
            };
            let content = if let Some(spec) = self.text_input_box() {
                let (top_inset, supporting_h) = self.text_input_extra_height(&spec, state, id);
                content.max(state.dp(56.0)) + top_inset + supporting_h
            } else if self.alert_dialog_icon().is_some() {
                content + self.alert_dialog_top_icon_extra(state)
            } else {
                content
            };
            let vertical_extra_gap = if !self.horizontal && matches!(self.kind, Kind::Linear) {
                let vis = self.children.iter().filter(|n| n.effective_visibility(state) != Visibility::Gone).count();
                vis.saturating_sub(1) as f32 * self.linear_extra_gap(state)
            } else {
                0.0
            };
            let content = content + vertical_extra_gap;
            let content = if self.horizontal && matches!(self.kind,Kind::Linear|Kind::RadioGroup) && !self.relative_rules.iter().any(|r|r.kind==36) {
                let widths:Vec<f32>=if let Some(widths) = horizontal_weights {
                    widths
                } else {
                    self.children.iter().enumerate().map(|(i,n)| {
                        let margins=n.resolved_margin(state);let available=(inner_width-margins[0]-margins[2]).max(0.0);
                        n.layout_intrinsic(true,state,available,metrics,child_view_id(id,i,n)).min(available)
                    }).collect()
                };
                let (height,_,_)=self.standalone_row_metrics(state,metrics,id,&widths);
                content.max((height-padding[1]-padding[3]).max(0.0))
            } else {content};
            let dividers = if matches!(self.kind,Kind::Linear|Kind::RadioGroup|Kind::TableRow) && !self.horizontal { self.divider_count(state) as f32 * state.dp(self.divider_thickness) } else { 0.0 };
            (content + dividers + padding[1] + padding[3]).max(min_h).max(bg_min_h).max(if matches!(self.kind,Kind::Frame | Kind::MediaController | Kind::MaterialCard|Kind::ViewAnimator|Kind::ViewSwitcher|Kind::TextSwitcher|Kind::ImageSwitcher|Kind::ViewFlipper) { state.dp(self.foreground_size[1]) } else { 0.0 })
        } else { self.intrinsic(horizontal, state, id) }
    }
}
