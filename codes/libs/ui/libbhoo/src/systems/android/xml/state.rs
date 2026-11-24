use super::*;

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            theme_style: AndroidTheme::default(),
            density: 1.0,
            font_scale: 1.0,
            dialer_filters: Default::default(),
            tab_widget_clicked: Default::default(),
            tab_hosts: Default::default(),
            tab_host_factory_cache: Default::default(),
            active_tab_host: Default::default(),
            gallery_gestures: Default::default(),
            gallery_rendered_generation: Default::default(),
            gallery_clicked: Default::default(),
            sliding_drawers: Default::default(),
            stack_clicked: Default::default(),
            stack_looping: Default::default(),
            stack_gestures: Default::default(),
            stack_pending_step: Default::default(),
            stack_rendered_generation: Default::default(),
            menu_allow_text_with_icon: Default::default(),
            grid_default_gap: Default::default(),
            video_frames: Default::default(),
            video_dimensions: Default::default(),
            video_ready: Default::default(),
            video_controllers: Default::default(),
            media_players: Default::default(),
            media_controller_actions: Default::default(),
            media_controller_capabilities: Default::default(),
            media_active_capabilities: Default::default(),
            media_seek_progress: Default::default(),
            media_floating: Default::default(),
            quick_contact_targets: Default::default(),
            quick_contact_clicked: Default::default(),
            quick_contact_pressed: Default::default(),
            quick_contact_lookups: Default::default(),
            quick_contact_actions: Default::default(),
            quick_contact_generation: Default::default(),
            rtl: Default::default(),
            table_collapsed: Default::default(),
            text: Default::default(),
            input_types: Default::default(),
            calendars: Default::default(),
            number_pickers: Default::default(),
            date_spinners: Default::default(),
            time_spinners: Default::default(),
            expandable_lists: Default::default(),
            expandable_events: Default::default(),
            toolbar_navigation_clicked: Default::default(),
            toolbar_title_colors: Default::default(),
            toolbar_subtitle_colors: Default::default(),
            toolbar_menu_items: Default::default(),
            toolbar_menu_clicked: Default::default(),
            toolbar_menu_overlays: Default::default(),
            expandable_intercepted_groups: Default::default(),
            expandable_group_clicked: Default::default(),
            expandable_templates: Default::default(),
            expandable_bindings: Default::default(),
            expandable_control_states: Default::default(),
            adapter_namespace: Default::default(),
            expandable_control_events: Default::default(),
            search_open: Default::default(),
            search_submit_enabled: Default::default(),
            search_submitted: Default::default(),
            search_suggestion_clicked: Default::default(),
            search_suggestion_threshold: Default::default(),
            search_suggestion_preview: Default::default(),
            search_pending: Default::default(),
            search_focus_requested: Default::default(),
            time_clocks: Default::default(),
            time_clock_dragging: Default::default(),
            time_clock_transition: Default::default(),
            time_clock_text_mode: Default::default(),
            time_clock_field_owner: Default::default(),
            time_clock_invalid: Default::default(),
            time_clock_changed: Default::default(),
            number_picker_motion: Default::default(),
            number_picker_editing: Default::default(),
            number_picker_committed: Default::default(),
            calendar_locale: Default::default(),
            calendar_motion: Default::default(),
            datepicker_year_open: Default::default(),
            datepicker_year_scroll: Default::default(),
            datepicker_fades: Default::default(),
            autocomplete_items: Default::default(),
            text_selection_end: Default::default(),
            comma_tokenizers: Default::default(),
            autocomplete_matches: Default::default(),
            autocomplete_anchor: Default::default(),
            autocomplete_open: Default::default(),
            autocomplete_panel: Default::default(),
            autocomplete_dismissed: Default::default(),
            autocomplete_highlight: Default::default(),
            autocomplete_scroll: Default::default(),
            autocomplete_selected: Default::default(),
            images: Default::default(),
            image_scales: Default::default(),
            image_matrices: Default::default(),
            drawable_tints: Default::default(),
            image_sources: Default::default(),
            view_selected: Default::default(),
            view_activated: Default::default(),
            view_dragged: Default::default(),
            tinted_images: Default::default(),
            tinted_image_bytes: Default::default(),
            zoom_holds: Default::default(),
            zoom_speed_ms: Default::default(),
            zoom_controls_enabled: Default::default(),
            zoom_controls_actions: Default::default(),
            zoom_controls_fades: Default::default(),
            checked: Default::default(),
            spinner_items: Default::default(),
            spinner_selected: Default::default(),
            spinner_item_ids: Default::default(),
            spinner_enabled: Default::default(),
            list_items: Default::default(),
            list_item_ids: Default::default(),
            list_checked: Default::default(),
            list_control_states: Default::default(),
            list_control_clicked: Default::default(),
            list_clicked: Default::default(),
            list_item_bindings: Default::default(),
            list_long_click_enabled: Default::default(),
            list_long_clicked: Default::default(),
            list_holds: Default::default(),
            list_enabled: Default::default(),
            list_fixed_clicked: Default::default(),
            list_fixed_control_clicked: Default::default(),
            list_fixed_control_states: Default::default(),
            list_fixed_views: Default::default(),
            adapter_root_response: Default::default(),
            list_templates: Default::default(),
            list_text_geometry: Default::default(),
            list_pending_focus: Default::default(),
            spinner_templates: Default::default(),
            spinner_row_cache: Default::default(),
            spinner_highlight: Default::default(),
            spinner_scroll: Default::default(),
            spinner_open: Default::default(),
            spinner_anchor: Default::default(),
            spinner_dialogs: Default::default(),
            spinner_closing: Default::default(),
            radio_groups: Default::default(),
            radio_group_requests: Default::default(),
            radio_group_events: Default::default(),
            progress: Default::default(),
            secondary_progress: Default::default(),
            ratings: Default::default(),
            offsets: Default::default(),
            expandable_scrolls: Default::default(),
            expandable_pending_scroll: Default::default(),
            expandable_geometry: Default::default(),
            displayed_children: Default::default(),
            transitions: Default::default(),
            flipper_running: Default::default(),
            adapter_flipper_clicked: Default::default(),
            adapter_flipper_host_advanced: Default::default(),
            adapter_flipper_data_generation: Default::default(),
            adapter_flipper_rendered_generation: Default::default(),
            adapter_flipper_animated: Default::default(),
            flipper_clocks: Default::default(),
            chronometers: Default::default(),
            wall_clock: Default::default(),
            visibility: Default::default(),
            text_input_password_visible: Default::default(),
            text_input_active_box: None,
            selected_index: Default::default(),
            nav_badges: Default::default(),
            refreshing: Default::default(),
            drawer_open: Default::default(),
            drawer_lock_mode: Default::default(),
            drawer_panel_active: false,
            toggle_group_segment: None,
            toggle_groups_initialized: Default::default(),
            range_values: Default::default(),
            range_slider_active_thumb: Default::default(),
            view_pager_gestures: Default::default(),
            collapsing_scroll_offset: None,
            collapsing_toolbar_child: false,
            sheet_state: Default::default(),
            entries: Default::default(),
            loading_states: Default::default(),
            til_dropdown_open: None,
            til_dropdown_anchor: None,
            til_dropdown_panel: None,
            tab_scroll_offsets: Default::default(),
            snackbar_active: false,
            split_button_role: None,
            chip_groups_initialized: Default::default(),
            tab_pager_links: Default::default(),
            sheet_motion: Default::default(),
            drawer_motion: Default::default(),
            active_bottom_sheet: None,
            drag_handle_toggled: false,
            tab_last_index: Default::default(),
            view_alpha: Default::default(),
            view_scale: Default::default(),
            view_translation: Default::default(),
            gradient_images: Default::default(),
            gradient_image_bytes: 0,
            cut_shape_images: Default::default(),
            cut_shape_image_bytes: 0,
            app_bar_lift_motion: Default::default(),
            lifted_appbar_active: false,
            placeholder_content: Default::default(),
            app_bar_scroll_motion: Default::default(),
            drawable_level: Default::default(),
            drawable_transition_active: Default::default(),
            drawable_transition_motion: Default::default(),
            suppress_next_elevation: false,
            animated_selector_states: Default::default(),
            fab_extended: Default::default(),
            collapsing_scrim_motion: Default::default(),
            extended_fab_motion: Default::default(),
            bottom_app_bar_hide_motion: Default::default(),
            bottom_app_bar_images: Default::default(),
            bottom_app_bar_image_bytes: 0,
            at_root_draw: false,
            bab_hide_applied_by_parent: false,
            frame_time: 0.0,
            frame_reduced_motion: false,
            frame_scroll_dragging: false,
            search_view_motion: Default::default(),
            search_bar_links: Vec::new(),
            swipe_refresh_motion: Default::default(),
            carousel_motion: Default::default(),
            last_clicked_link: None,
        }
    }
}
impl LayoutState {
    pub(super) fn linked_tab_pager_partners(&self, id: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(target) = self.tab_pager_links.get(id)
            && !target.is_empty()
            && target != id
        {
            out.push(target.clone());
        }
        for (k, v) in &self.tab_pager_links {
            if v == id && !k.is_empty() && k != id && !out.iter().any(|s| s == k) {
                out.push(k.clone());
            }
        }
        out
    }
    /// Effective display density multiplier (`<= 0.0` or non-finite defaults to `1.0`).
    pub fn effective_density(&self) -> f32 {
        if self.density.is_finite() && self.density > 0.0 { self.density } else { 1.0 }
    }
    /// Effective font scale multiplier (`<= 0.0` or non-finite defaults to `1.0`).
    pub fn effective_font_scale(&self) -> f32 {
        if self.font_scale.is_finite() && self.font_scale > 0.0 { self.font_scale } else { 1.0 }
    }
    #[inline]
    pub(super) fn dp(&self, value: f32) -> f32 {
        value * self.effective_density()
    }
    #[inline]
    pub(super) fn sp(&self, value: f32) -> f32 {
        value * self.effective_density() * self.effective_font_scale()
    }
    /// Configure view-ID tabs. XML supplies matching TabWidget children and content views.
    pub fn configure_tab_host(&mut self,id:&str,widget_id:&str,content_ids:Vec<String>)->Result<(),&'static str> {
        if id.is_empty()||widget_id.is_empty()||content_ids.is_empty()||content_ids.iter().any(|s|s.is_empty()) {return Err("TabHost needs a host, widget and content IDs");}
        let mut seen=std::collections::HashSet::new();if content_ids.iter().any(|id|!seen.insert(id)){return Err("TabHost content IDs must be unique");}
        let factories=vec![None;content_ids.len()];self.tab_host_factory_cache.retain(|(host,_),_|host!=id);self.tab_hosts.insert(id.to_owned(),TabHostState{widget_id:widget_id.to_owned(),content_ids,labels:Vec::new(),tags:Vec::new(),selected:0,factories});Ok(())
    }
    /// Configure label indicators and tags for an empty XML TabWidget.
    pub fn configure_tab_host_labels(&mut self,id:&str,widget_id:&str,tabs:Vec<(String,String,String)>)->Result<(),&'static str> {
        if tabs.is_empty()||tabs.iter().any(|(tag,label,content)|tag.is_empty()||label.is_empty()||content.is_empty()){return Err("TabHost tabs need tags, labels and content IDs");}
        let content=tabs.iter().map(|(_,_,content)|content.clone()).collect();self.configure_tab_host(id,widget_id,content)?;
        let host=self.tab_hosts.get_mut(id).unwrap();host.tags=tabs.iter().map(|(tag,_,_)|tag.clone()).collect();host.labels=tabs.into_iter().map(|(_,label,_)|label).collect();Ok(())
    }
    /// Configure lazily created XML content in an empty tabcontent FrameLayout.
    pub fn configure_tab_host_factory(&mut self,id:&str,widget_id:&str,tabs:Vec<(String,String)>,factory:fn(&str)-> &'static Node)->Result<(),&'static str> {
        let specs=tabs.into_iter().enumerate().map(|(index,(tag,label))|(tag,label,format!("#factory:{index}"))).collect();self.configure_tab_host_labels(id,widget_id,specs)?;
        let host=self.tab_hosts.get_mut(id).unwrap();host.factories.fill(Some(factory));Ok(())
    }
    /// Configure label tabs mixing existing XML views and lazy factories.
    pub fn configure_tab_host_mixed(&mut self,id:&str,widget_id:&str,tabs:Vec<(String,String,TabHostContent)>)->Result<(),&'static str> {
        let mut factories=Vec::with_capacity(tabs.len());let mut specs=Vec::with_capacity(tabs.len());
        for (index,(tag,label,content)) in tabs.into_iter().enumerate(){match content {TabHostContent::View(view)=>{factories.push(None);specs.push((tag,label,view));},TabHostContent::Factory(factory)=>{factories.push(Some(factory));specs.push((tag,label,format!("#factory:{index}")));}}}
        self.configure_tab_host_labels(id,widget_id,specs)?;self.tab_hosts.get_mut(id).unwrap().factories=factories;Ok(())
    }
    /// Number of factory-backed tabs created for this host.
    pub fn tab_host_created_count(&self,id:&str)->usize {self.tab_host_factory_cache.keys().filter(|(host,_)|host==id).count()}
    /// Select the first tab with the given tag.
    pub fn set_tab_host_by_tag(&mut self,id:&str,tag:&str)->bool {
        let index=self.tab_hosts.get(id).and_then(|host|host.tags.iter().position(|value|value==tag));index.is_some_and(|index|self.set_tab_host_current(id,index))
    }
    /// Select a configured tab; invalid indices leave the current tab unchanged.
    pub fn set_tab_host_current(&mut self,id:&str,index:usize)->bool {
        let Some(host)=self.tab_hosts.get_mut(id) else{return false;};if index>=host.content_ids.len(){return false;}host.selected=index;
        self.displayed_children.insert(host.widget_id.clone(),index);
        for (i,content) in host.content_ids.iter().enumerate(){if host.factories.get(i).is_none_or(Option::is_none){self.visibility.insert(content.clone(),if i==index{Visibility::Visible}else{Visibility::Gone});}}true
    }

    /// Show a floating controller; timeout zero leaves it visible until hidden.
    pub fn show_floating_media_controller(&mut self,id:&str,now:f64,timeout_ms:u64) {
        if !now.is_finite(){return;}self.media_floating.insert(id.to_owned(),MediaControllerFloating{visible:true,deadline:if timeout_ms==0{None}else{Some(now+timeout_ms as f64/1000.0)},dragging:false});
    }
    pub fn hide_floating_media_controller(&mut self,id:&str) {self.media_floating.remove(id);}
    pub fn floating_media_controller_is_showing(&self,id:&str)->bool {self.media_floating.get(id).is_some_and(|state|state.visible)}
    /// Assign a contact; non-lazy email or phone assignments request lookup immediately.
    pub fn assign_quick_contact(&mut self,id:&str,target:Option<QuickContactTarget>,lazy:bool) {
        self.quick_contact_pressed.remove(id);self.quick_contact_lookups.remove(id);self.quick_contact_actions.remove(id);self.quick_contact_clicked.remove(id);
        if let Some(target)=target {self.quick_contact_targets.insert(id.to_owned(),target.clone());if !lazy&&!matches!(target,QuickContactTarget::Uri(_)){self.request_quick_contact_lookup(id,target,false);}}else{self.quick_contact_targets.remove(id);}
    }
    pub(super) fn request_quick_contact_lookup(&mut self,id:&str,target:QuickContactTarget,open_after_lookup:bool) {
        self.quick_contact_generation=self.quick_contact_generation.wrapping_add(1);
        self.quick_contact_lookups.insert(id.to_owned(),QuickContactLookup{token:self.quick_contact_generation,target,open_after_lookup});
    }
    /// Complete a host lookup. Stale responses are ignored. None means no contact matched.
    pub fn resolve_quick_contact(&mut self,id:&str,token:u64,uri:Option<String>)->bool {
        let Some(request)=self.quick_contact_lookups.get(id).filter(|request|request.token==token).cloned() else{return false;};
        if self.quick_contact_targets.get(id)!=Some(&request.target){self.quick_contact_lookups.remove(id);return false;}
        self.quick_contact_lookups.remove(id);
        if let Some(uri)=uri {self.quick_contact_targets.insert(id.to_owned(),QuickContactTarget::Uri(uri.clone()));if request.open_after_lookup{self.quick_contact_actions.insert(id.to_owned(),QuickContactAction::Show(uri));}}
        else if request.open_after_lookup {self.quick_contact_actions.insert(id.to_owned(),QuickContactAction::Create(request.target));}
        true
    }
    /// Expand a group and optionally scroll its children into view, as Android's expandGroup does.
    pub fn expand_list_group(&mut self,id:&str,group:usize,animate:bool)->bool {
        let Some(list)=self.expandable_lists.get_mut(id) else {return false;};
        let Some(group_id)=list.groups().get(group).map(|group|group.id) else {return false;};
        let changed=list.expand(group);
        self.expandable_events.insert(id.to_owned(),crate::android_expandable_list::ExpandableEvent::Expanded(group));
        if animate {self.expandable_scrolls.remove(id);self.expandable_pending_scroll.insert(id.to_owned(),group_id);}
        changed
    }
    pub fn collapse_list_group(&mut self,id:&str,group:usize)->bool {
        let Some(list)=self.expandable_lists.get_mut(id) else {return false;};
        if group>=list.groups().len(){return false;}
        let changed=list.collapse(group);
        self.expandable_events.insert(id.to_owned(),crate::android_expandable_list::ExpandableEvent::Collapsed(group));
        if changed {self.expandable_scrolls.remove(id);self.expandable_pending_scroll.remove(id);}
        changed
    }
    /// Apply MenuItem.setChecked; exclusive groups select this item even for false, as Android does.
    pub fn set_toolbar_item_checked(&mut self,toolbar:&str,item_id:u64,checked:bool) {
        if let Some(items)=self.toolbar_menu_items.get_mut(toolbar) {
            if let Some(index)=items.iter().position(|item|item.id==item_id) {
                if items[index].exclusive {let group=items[index].group_id;for item in items.iter_mut().filter(|item|item.group_id==group&&item.exclusive&&item.checkable) {item.checked=item.id==item_id;}}
                else {items[index].checked=checked;}
            }
        }
    }
    pub fn set_toolbar_group_checkable(&mut self,toolbar:&str,group_id:u64,checkable:bool,exclusive:bool) {
        if let Some(items)=self.toolbar_menu_items.get_mut(toolbar) {for item in items.iter_mut().filter(|item|item.group_id==group_id) {item.checkable=checkable;item.exclusive=exclusive;}}
    }
    /// Apply Android Menu.setGroupEnabled to every item with this group ID.
    pub fn set_toolbar_group_enabled(&mut self,toolbar:&str,group_id:u64,enabled:bool) {
        if let Some(items)=self.toolbar_menu_items.get_mut(toolbar) {for item in items.iter_mut().filter(|item|item.group_id==group_id) {item.enabled=enabled;}}
    }
    /// Apply Android Menu.setGroupVisible to every item with this group ID.
    pub fn set_toolbar_group_visible(&mut self,toolbar:&str,group_id:u64,visible:bool) {
        if let Some(items)=self.toolbar_menu_items.get_mut(toolbar) {for item in items.iter_mut().filter(|item|item.group_id==group_id) {item.visible=visible;}}
    }
    /// Replace the synchronous ArrayAdapter-style suggestions for an AutoCompleteTextView.
    pub fn set_autocomplete_items(&mut self,id:&str,items:Vec<String>) {
        self.autocomplete_items.insert(id.to_owned(),items);self.autocomplete_matches.remove(id);
        self.search_suggestion_preview.remove(id);
        self.autocomplete_dismissed.remove(id);self.autocomplete_highlight.remove(id);self.autocomplete_scroll.remove(id);
    }
    /// Consume a pending request to focus the host SearchView editor.
    pub fn take_search_focus_request(&mut self,id:&str)->bool {self.search_focus_requested.remove(id)}
    /// Set a SearchView query from a host editor and request a change callback.
    pub fn set_search_query(&mut self,id:&str,text:&str) {if self.text.get(id).is_none_or(|old|old!=text){self.text.insert(id.to_owned(),text.to_owned());self.search_pending.insert(id.to_owned());}}
    /// Submit a nonblank query; the host receives the original query unchanged.
    pub fn submit_search(&mut self,id:&str)->bool {let Some(query)=self.text.get(id) else{return false;};if query.trim().is_empty(){return false;}self.search_submitted.insert(id.to_owned(),query.clone());self.search_pending.insert(id.to_owned());true}
    /// Finish a NumberPicker text edit. Empty input restores the previous value.
    pub fn end_number_picker_edit(&mut self,id:&str,commit:bool)->bool {
        if !self.number_picker_editing.remove(id){return false;}
        let changed=if commit {if let Some(picker)=self.number_pickers.get_mut(id) {self.text.get(id).and_then(|text|picker.parse_input_localized(text,&self.calendar_locale.digits)).is_some_and(|value|picker.set_value(value))}else{false}}else{false};
        if changed {self.number_picker_committed.insert(id.to_owned());}changed
    }
    /// Apply a host text update to a clock-mode TimePicker field.
    pub fn update_time_clock_input(&mut self,field_id:&str) {
        let Some((owner,part))=self.time_clock_field_owner.get(field_id).cloned() else{return;};let value=self.text.get(field_id).and_then(|s|crate::android_number_picker::parse_localized_decimal(s,&self.calendar_locale.digits));
        let Some(picker)=self.time_clocks.get_mut(&owner) else{return;};let zero=if picker.is_24_hour(){matches!(self.calendar_locale.hour_cycle_24,crate::android_time_spinner::HourCycle::H23)}else{matches!(self.calendar_locale.hour_cycle_12,crate::android_time_spinner::HourCycle::H11)};
        let result=value.and_then(|n|u8::try_from(n).ok()).and_then(|n|match part {crate::android_time_clock::DialPart::Hour=>picker.set_input_hour(n,zero).ok(),crate::android_time_clock::DialPart::Minute=>picker.set_input_minute(n).ok()});
        if let Some(changed)=result {self.time_clock_invalid.remove(field_id);if changed{self.time_clock_changed.insert(owner);}}else{self.time_clock_invalid.insert(field_id.to_owned());}
    }
    pub fn autocomplete_has_highlight(&self,id:&str)->bool {self.autocomplete_open.as_deref()==Some(id)&&self.autocomplete_highlight.contains_key(id)}
    /// Whether the suggestion popup belongs to the given XML field.
    pub fn autocomplete_is_open(&self,id:&str)->bool {self.autocomplete_open.as_deref()==Some(id)}
    /// Set noninteractive XML templates for the selected view and popup rows.
    pub fn set_spinner_item_templates(&mut self,id:&str,templates:Option<SpinnerItemTemplates>)->Result<(),&'static str> {
        if let Some(t)=templates {
            let selected=inspect_adapter_template(t.selected,t.text_id);let dropdown=inspect_adapter_template(t.dropdown,t.text_id);
            if t.text_id.is_empty() || !selected.0 || !selected.1 || !dropdown.0 || !dropdown.1 {return Err("Spinner templates require noninteractive views and a matching TextView ID");}
            self.spinner_templates.insert(id.to_owned(),t);
        } else {self.spinner_templates.remove(id);}
        Ok(())
    }
    /// Bind separate noninteractive Android XML layouts to expandable group and child rows.
    /// Template heights are measured as rows become visible.
    pub fn set_expandable_item_templates(&mut self,id:&str,group:Option<ListItemTemplate>,child:Option<ListItemTemplate>)->Result<(),&'static str> {
        for template in [group,child].into_iter().flatten() {
            let (safe,found,_)=inspect_expandable_template(template.row,template.text_id);
            if template.text_id.is_empty() || !safe || !found {return Err("ExpandableListView templates require supported views and a matching TextView ID");}
        }
        if group.is_none()&&child.is_none(){self.expandable_templates.remove(id);}else{self.expandable_templates.insert(id.to_owned(),(group,child));}
        Ok(())
    }
    /// Bind additional text, images, or a template to an expandable row.
    /// The binding follows stable group/child IDs when the adapter is reordered.
    pub fn set_expandable_item_binding(&mut self,id:&str,position:crate::android_expandable_list::FlatPosition,binding:Option<AdapterItemBinding>)->Result<(),&'static str> {
        use crate::android_expandable_list::FlatPosition;
        let list=self.expandable_lists.get(id).ok_or("ExpandableListView adapter is missing")?;
        let key=match position {
            FlatPosition::Group(group)=>(list.groups().get(group).ok_or("ExpandableListView group is out of range")?.id,None),
            FlatPosition::Child{group,child}=>{let group=list.groups().get(group).ok_or("ExpandableListView group is out of range")?;(group.id,Some(group.children.get(child).ok_or("ExpandableListView child is out of range")?.id))},
        };
        if let Some(template)=binding.as_ref().and_then(|value|value.template) {
            let (safe,found,_)=inspect_expandable_template(template.row,template.text_id);
            if template.text_id.is_empty()||!safe||!found{return Err("ExpandableListView item templates require supported views and a matching TextView ID");}
        }
        if let Some(binding)=binding {self.expandable_bindings.entry(id.to_owned()).or_default().insert(key,binding);}
        else if let Some(bindings)=self.expandable_bindings.get_mut(id){bindings.remove(&key);}
        Ok(())
    }
    /// Read a checked control in a row by stable adapter IDs and XML view ID.
    pub fn expandable_control_checked(&self,id:&str,group_id:u64,child_id:Option<u64>,view_id:&str)->Option<bool>{
        self.expandable_control_states.get(&(id.to_owned(),group_id,child_id)).and_then(|checked|checked.get(view_id).copied())
    }
    /// Match Android's fyiWillBeAdvancedByHostKThx: stop the internal timer
    /// while keeping the requested start/stop state and allowing host advances.
    pub fn adapter_flipper_advance_by_host(&mut self,id:&str) {
        self.adapter_flipper_host_advanced.insert(id.to_owned());
        self.flipper_clocks.remove(&view_id(id));
    }
    /// Advance an adapter flipper by one or more positions and reset its timer.
    pub fn adapter_flipper_step(&mut self,id:&str,delta:isize)->bool {
        let count=self.list_items.get(id).map_or(0,Vec::len);if count==0{return false;}
        let selected=self.displayed_children.entry(id.to_owned()).or_default();
        let next=(*selected as i128+delta as i128).rem_euclid(count as i128) as usize;
        let changed=*selected!=next;*selected=next;self.flipper_clocks.remove(&view_id(id));changed
    }
    /// Request a RadioGroup selection by descendant ID, or clear it with `None`.
    /// Applied when the group next renders; unknown IDs remain the group's selection,
    /// as with Android's `check`, but do not check a view.
    pub fn set_radio_group_checked(&mut self,group:&str,selection:Option<&str>) {
        self.radio_group_requests.insert(group.to_owned(),selection.map(str::to_owned));
    }
    pub(super) fn record_radio_group_event(&mut self,group:&str,selection:Option<u64>) {
        let events=self.radio_group_events.entry(group.to_owned()).or_default();
        if events.len()==32 {events.remove(0);}
        events.push(selection);
    }
    /// Drain selection notifications. `None` means cleared; the queue holds the latest 32.
    pub fn take_radio_group_events(&mut self,group:&str)->Vec<Option<u64>> {
        self.radio_group_events.remove(group).unwrap_or_default()
    }
    /// Prevent user gestures on a SlidingDrawer. Programmatic requests still work.
    pub fn set_sliding_drawer_locked(&mut self,id:&str,locked:bool) {
        let drawer=self.sliding_drawers.entry(id.to_owned()).or_default();
        drawer.locked=locked;
        if locked && drawer.drag.take().is_some() {drawer.record(SlidingDrawerEvent::ScrollEnded);drawer.dragging=false;drawer.samples.clear();drawer.position=if drawer.open{1.0}else{0.0};}
    }
    /// Open or close a SlidingDrawer on its next render, with optional animation.
    pub fn set_sliding_drawer_open(&mut self,id:&str,open:bool,animated:bool) {
        self.sliding_drawers.entry(id.to_owned()).or_default().pending=Some((open,animated));
    }
    /// Drain listener events for this drawer. The queue retains at most 32 events.
    pub fn take_sliding_drawer_events(&mut self,id:&str)->Vec<SlidingDrawerEvent> {
        self.sliding_drawers.get_mut(id).map(|drawer|std::mem::take(&mut drawer.events)).unwrap_or_default()
    }
    /// Advance the displayed StackView adapter item, respecting its loop policy.
    pub fn stack_step(&mut self,id:&str,delta:isize)->bool {
        let count=self.list_items.get(id).map_or(0,Vec::len);if count==0{return false;}
        let looping=self.stack_looping.get(id).copied().unwrap_or(false);let selected=self.displayed_children.entry(id.to_owned()).or_default();let old=(*selected).min(count-1);let value=old as i128+delta as i128;let next=if looping {value.rem_euclid(count as i128)}else{value.clamp(0,count as i128-1)} as usize;let changed=old!=next;*selected=next;if changed{self.stack_pending_step.insert(id.to_owned(),(old,delta));}changed
    }
    /// Bind a ListView row XML template to each adapter label.
    pub fn set_list_item_template(&mut self,id:&str,template:Option<ListItemTemplate>)->Result<(),&'static str> {
        if let Some(t)=template {
            let (safe,found)=inspect_adapter_template(t.row,t.text_id);
            if t.text_id.is_empty() || !safe || !found {return Err("ListView templates require noninteractive views and a matching TextView ID");}
            self.list_templates.insert(id.to_owned(),t);
        } else {self.list_templates.remove(id);}
        Ok(())
    }
    /// Install a ListView or GridView item with supported interactive checkbox, switch, or button children.
    pub fn set_interactive_list_item_template(&mut self,id:&str,template:Option<ListItemTemplate>)->Result<(),&'static str> {
        if let Some(t)=template {let (safe,found,_)=inspect_expandable_template(t.row,t.text_id);if t.text_id.is_empty()||!safe||!found{return Err("Interactive ListView templates require supported views and a matching TextView ID");}self.list_templates.insert(id.to_owned(),t);}else{self.list_templates.remove(id);}
        Ok(())
    }
    pub fn list_item_control_checked(&self,id:&str,item_id:i64,view_id:&str)->Option<bool> {
        self.list_control_states.get(&(id.to_owned(),item_id)).and_then(|controls|controls.get(view_id).copied())
    }
    pub fn list_fixed_control_checked(&self,id:&str,footer:bool,index:usize,view_id:&str)->Option<bool> {
        self.list_fixed_control_states.get(&(id.to_owned(),footer,index)).and_then(|controls|controls.get(view_id).copied())
    }
    /// Set or remove additional bindings for one adapter position.
    pub fn set_list_item_binding(&mut self,id:&str,index:usize,binding:Option<AdapterItemBinding>)->Result<(),&'static str> {
        self.set_list_item_binding_impl(id,index,binding,false)
    }
    /// Set a per-item binding that may include supported interactive XML controls.
    pub fn set_interactive_list_item_binding(&mut self,id:&str,index:usize,binding:Option<AdapterItemBinding>)->Result<(),&'static str> {
        self.set_list_item_binding_impl(id,index,binding,true)
    }
    pub(super) fn set_list_item_binding_impl(&mut self,id:&str,index:usize,binding:Option<AdapterItemBinding>,interactive:bool)->Result<(),&'static str> {
        if self.list_items.get(id).is_none_or(|items|index>=items.len()) {return Err("ListView item binding position is out of range");}
        if let Some(template)=binding.as_ref().and_then(|value|value.template) {
            let (safe,found)=if interactive {let (safe,found,_)=inspect_expandable_template(template.row,template.text_id);(safe,found)}else{inspect_adapter_template(template.row,template.text_id)};
            if template.text_id.is_empty() || !safe || !found {return Err("Adapter item template contains unsupported views or lacks a matching TextView ID");}
        }
        if let Some(binding)=binding {self.list_item_bindings.entry(id.to_owned()).or_default().insert(index,binding);}
        else if let Some(bindings)=self.list_item_bindings.get_mut(id) {bindings.remove(&index);}
        Ok(())
    }
    /// Replace fixed header and footer views, including supported interactive children.
    pub fn set_list_fixed_views(&mut self,id:&str,headers:Vec<ListFixedView>,footers:Vec<ListFixedView>)->Result<(),&'static str> {
        if headers.iter().chain(&footers).any(|fixed|!inspect_expandable_template(fixed.view,"").0) {return Err("ListView fixed views contain unsupported XML views");}
        let same=|a:&[ListFixedView],b:&[ListFixedView]|a.len()==b.len() && a.iter().zip(b).all(|(a,b)|std::ptr::eq(a.view,b.view) && a.selectable==b.selectable && a.label==b.label);
        if self.list_fixed_views.get(id).is_some_and(|(old_headers,old_footers)|same(old_headers,&headers) && same(old_footers,&footers)) {return Ok(());}
        self.list_fixed_views.insert(id.to_owned(),(headers,footers));
        self.list_fixed_control_states.retain(|(list,_,_),_|list!=id);self.list_fixed_control_clicked.remove(id);
        self.list_holds.remove(id);self.list_pending_focus.remove(id);self.list_fixed_clicked.remove(id);
        Ok(())
    }
    /// Replace a ListView adapter, preserving checked items by stable ID when
    /// both adapters provide IDs, or by position otherwise. Missing items are unchecked.
    pub fn set_list_adapter(&mut self,id:&str,items:Vec<String>,ids:Option<Vec<i64>>,enabled:Option<Vec<bool>>)->Result<(),&'static str> {
        if ids.as_ref().is_some_and(|v|v.len()!=items.len()) || enabled.as_ref().is_some_and(|v|v.len()!=items.len()) {return Err("ListView adapter arrays must have equal lengths");}
        let positions=if let Some(values)=ids.as_ref() {
            let mut positions=HashMap::with_capacity(values.len());
            for (i,value) in values.iter().enumerate() {if positions.insert(*value,i).is_some() {return Err("ListView stable IDs must be unique");}}
            Some(positions)
        } else {None};
        let old_ids=self.list_item_ids.get(id);
        if old_ids.is_some()!=ids.is_some() {
            let mut migrated=Vec::new();
            self.list_control_states.retain(|(list,item),controls|{if list!=id{return true;}let position=old_ids.and_then(|values|values.iter().position(|value|value==item)).or_else(||if old_ids.is_none()&&*item>=0{Some(*item as usize)}else{None});if let Some(position)=position.filter(|position|*position<items.len()){migrated.push((ids.as_ref().map_or(position as i64,|values|values[position]),controls.clone()));}false});
            for (item,controls) in migrated {self.list_control_states.insert((id.to_owned(),item),controls);}
        } else {
            self.list_control_states.retain(|(list,item),_|list!=id||match positions.as_ref(){Some(values)=>values.contains_key(item),None=>usize::try_from(*item).is_ok_and(|index|index<items.len())});
        }
        let mapped=|position:usize|->Option<usize> {
            match (old_ids,positions.as_ref()) {
                (Some(old),Some(new))=>old.get(position).and_then(|item|new.get(item)).copied(),
                _=>(position<items.len()).then_some(position),
            }
        };
        let mut checked=Vec::new();let mut seen=std::collections::HashSet::new();
        if let Some(previous)=self.list_checked.get(id) {for index in previous {if let Some(new)=mapped(*index) && seen.insert(new) {checked.push(new);}}}
        let gallery_selection=if self.gallery_gestures.contains_key(id) {let index=self.displayed_children.get(id).copied().unwrap_or(0);Some(mapped(index).unwrap_or_else(||index.min(items.len().saturating_sub(1))))}else{None};
        let clicked=self.list_clicked.get(id).and_then(|index|mapped(*index));
        let bindings=self.list_item_bindings.get(id).map(|old|old.iter().filter_map(|(index,value)|mapped(*index).map(|new|(new,value.clone()))).collect::<HashMap<_,_>>());
        if checked.is_empty() {self.list_checked.remove(id);}else{self.list_checked.insert(id.to_owned(),checked);}
        if let Some(index)=clicked {self.list_clicked.insert(id.to_owned(),index);} else {self.list_clicked.remove(id);}
        if let Some(values)=bindings.filter(|values|!values.is_empty()) {self.list_item_bindings.insert(id.to_owned(),values);}else{self.list_item_bindings.remove(id);}
        if let Some(index)=gallery_selection {if items.is_empty(){self.displayed_children.remove(id);}else{self.displayed_children.insert(id.to_owned(),index);}}
        self.gallery_gestures.remove(id);self.gallery_clicked.remove(id);
        self.list_items.insert(id.to_owned(),items);
        if let Some(values)=ids {self.list_item_ids.insert(id.to_owned(),values);}else{self.list_item_ids.remove(id);}
        if let Some(values)=enabled {self.list_enabled.insert(id.to_owned(),values);}else{self.list_enabled.remove(id);}
        self.list_text_geometry.remove(id);self.list_pending_focus.remove(id);self.list_holds.remove(id);self.list_long_clicked.remove(id);
        let generation=self.adapter_flipper_data_generation.entry(id.to_owned()).or_default();*generation=generation.wrapping_add(1);self.adapter_flipper_clicked.remove(id);self.stack_pending_step.remove(id);self.stack_clicked.remove(id);
        Ok(())
    }
    /// Replace a Spinner adapter transactionally, preserving selection and the
    /// open-list highlight by stable ID when both old and new adapters have IDs.
    pub fn set_spinner_adapter(&mut self,id:&str,items:Vec<String>,ids:Option<Vec<i64>>,enabled:Option<Vec<bool>>)->Result<(),&'static str> {
        if ids.as_ref().is_some_and(|v|v.len()!=items.len()) || enabled.as_ref().is_some_and(|v|v.len()!=items.len()) {return Err("Spinner adapter arrays must have equal lengths");}
        if let Some(values)=ids.as_ref() {
            let mut unique=std::collections::HashSet::with_capacity(values.len());
            if values.iter().any(|id|!unique.insert(*id)) {return Err("Spinner stable IDs must be unique");}
        }
        let old_ids=self.spinner_item_ids.get(id);
        let find_position=|position:usize| {
            let old_id=old_ids.and_then(|values|values.get(position));
            let matching=old_id.and_then(|needle|ids.as_ref().and_then(|values|values.iter().position(|v|v==needle)));
            matching.unwrap_or_else(||position.min(items.len().saturating_sub(1)))
        };
        let selected=find_position(self.spinner_selected.get(id).copied().unwrap_or(0));
        let highlighted=self.spinner_highlight.get(id).copied().map(find_position);
        self.spinner_scroll.remove(id);
        if items.is_empty() {
            self.spinner_selected.remove(id);self.spinner_highlight.remove(id);
            if self.spinner_open.as_deref()==Some(id) {self.spinner_open=None;}
            if self.spinner_closing.as_deref()==Some(id) {self.spinner_closing=None;}
        } else {
            self.spinner_selected.insert(id.to_owned(),selected);
            if let Some(index)=highlighted {self.spinner_highlight.insert(id.to_owned(),index);}
        }
        self.spinner_items.insert(id.to_owned(),items);
        if let Some(values)=ids {self.spinner_item_ids.insert(id.to_owned(),values);} else {self.spinner_item_ids.remove(id);}
        if let Some(values)=enabled {self.spinner_enabled.insert(id.to_owned(),values);} else {self.spinner_enabled.remove(id);}
        Ok(())
    }
    /// Animate a ZoomControls view to visible or gone. Time uses frame seconds.
    /// A reversal starts from the currently presented opacity.
    pub fn set_zoom_controls_visible(&mut self, id:&str, visible:bool, now:f64) {
        if self.zoom_controls_fades.get(id).is_some_and(|f| f.target_visible==visible) { return; }
        if !visible {
            let parent=view_id(id);
            self.zoom_holds.remove(&anonymous_child_id(parent,0));
            self.zoom_holds.remove(&anonymous_child_id(parent,1));
            self.zoom_controls_actions.remove(id);
        }
        let from=self.zoom_controls_fades.get(id).map(|f|f.value(now)).unwrap_or_else(|| if self.visibility.get(id)==Some(&Visibility::Gone) {0.0} else {1.0});
        if (from-f32::from(visible)).abs()<0.0001 {
            self.visibility.insert(id.to_owned(),if visible { Visibility::Visible } else { Visibility::Gone });
            self.zoom_controls_fades.remove(id);
        } else {
            self.visibility.insert(id.to_owned(),Visibility::Visible);
            self.zoom_controls_fades.insert(id.to_owned(),ZoomControlsFade { from,target_visible:visible,start:now });
        }
    }
}
