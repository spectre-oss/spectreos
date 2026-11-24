//! Interactive catalog compositions. Production API coverage is recorded separately.
use bhoo::*;
#[path = "android_material_card.rs"]
mod android_material_card;
#[path = "android_material_card_compat.rs"]
mod android_material_card_compat;
#[path = "android_material_card_selector.rs"]
mod android_material_card_selector;
#[path = "android_material_card_theme.rs"]
mod android_material_card_theme;
#[path = "android_multi_autocomplete.rs"]
mod android_multi_autocomplete;
#[path = "android_calendar_view.rs"]
mod android_calendar_view;
#[path = "android_date_picker.rs"]
mod android_date_picker;
#[path = "android_number_picker.rs"]
mod android_number_picker;
#[path = "android_date_picker_spinner.rs"]
mod android_date_picker_spinner;
#[path = "android_date_picker_combined.rs"]
mod android_date_picker_combined;
#[path = "android_time_picker_spinner.rs"]
mod android_time_picker_spinner;
#[path = "android_time_picker_clock.rs"]
mod android_time_picker_clock;
#[path = "android_search_view.rs"]
mod android_search_view;
#[path = "android_toolbar.rs"]
mod android_toolbar;
#[path = "android_action_menu_view.rs"]
mod android_action_menu_view;
#[path = "android_adapter_view_flipper.rs"]
mod android_adapter_view_flipper;
#[path = "android_gallery_item.rs"]
mod android_gallery_item;
#[path = "android_dialer_filter.rs"] mod android_dialer_filter;
#[path = "android_tab_host_mixed.rs"] mod android_tab_host_mixed;
#[path = "android_tab_host_factory_first.rs"] mod android_tab_host_factory_first;
#[path = "android_tab_host_factory_second.rs"] mod android_tab_host_factory_second;
#[path = "android_tab_host_factory_third.rs"] mod android_tab_host_factory_third;
fn tab_host_demo_factory(tag:&str)-> &'static bhoo::android_xml::Node {match tag {"one"=>&android_tab_host_factory_first::LAYOUT,"two"=>&android_tab_host_factory_second::LAYOUT,_=>&android_tab_host_factory_third::LAYOUT}}
#[path = "android_tab_widget.rs"]
mod android_tab_widget;
#[path = "android_tab_widget_fixed.rs"]
mod android_tab_widget_fixed;
#[path = "android_tab_widget_margins.rs"]
mod android_tab_widget_margins;
#[path = "android_tab_widget_weighted.rs"]
mod android_tab_widget_weighted;
#[path = "android_tab_widget_wrap.rs"]
mod android_tab_widget_wrap;
#[path = "android_tab_widget_glyphs.rs"]
mod android_tab_widget_glyphs;
#[path = "android_gallery.rs"]
mod android_gallery;
#[path = "android_sliding_drawer.rs"]
mod android_sliding_drawer;
#[path = "android_sliding_drawer_nested.rs"]
mod android_sliding_drawer_nested;
#[path = "android_stack_view.rs"]
mod android_stack_view;
#[path = "android_adapter_view_flipper_custom.rs"]
mod android_adapter_view_flipper_custom;
#[path = "android_adapter_flipper_item.rs"]
mod android_adapter_flipper_item;
#[path = "android_toolbar_menu.rs"]
mod android_toolbar_menu;
#[path = "android_expandable_list.rs"]
mod android_expandable_list;
#[path = "android_expandable_header.rs"]
mod android_expandable_header;
#[path = "android_expandable_footer.rs"]
mod android_expandable_footer;
#[path = "android_expandable_group_row.rs"]
mod android_expandable_group_row;
#[path = "android_expandable_child_row.rs"]
mod android_expandable_child_row;
#[path = "android_autocomplete.rs"]
mod android_autocomplete;
#[path = "android_gridview_center.rs"]
mod android_gridview_center;
#[path = "android_gridview_team_item.rs"]
mod android_gridview_team_item;
#[path = "android_gridview_item.rs"]
mod android_gridview_item;
#[path = "android_gridview.rs"]
mod android_gridview;
#[path = "android_list_header.rs"]
mod android_list_header;
#[path = "android_list_footer.rs"]
mod android_list_footer;
#[path = "android_list_interactive_header.rs"]
mod android_list_interactive_header;
#[path = "android_list_interactive_row.rs"]
mod android_list_interactive_row;
#[path = "android_quick_contact_badge.rs"]
mod android_quick_contact_badge;
#[path = "android_video_weighted_measure.rs"]
mod android_video_weighted_measure;
#[path = "android_video_linear_measure.rs"]
mod android_video_linear_measure;
#[path = "android_video_measure.rs"]
mod android_video_measure;
#[path = "android_video_view.rs"]
mod android_video_view;
#[path = "android_media_controller.rs"]
mod android_media_controller;
#[path = "android_listview.rs"]
mod android_listview;
#[path = "android_spinner.rs"]
mod android_spinner;
#[path = "android_spinner_dialog.rs"]
mod android_spinner_dialog;
#[path = "android_spinner_wide.rs"]
mod android_spinner_wide;
#[path = "android_spinner_selected_row.rs"]
mod android_spinner_selected_row;
#[path = "android_spinner_dropdown_row.rs"]
mod android_spinner_dropdown_row;
#[path = "android_analogclock.rs"]
mod android_analogclock;
#[path = "android_grid_layout.rs"]
mod android_grid_layout;
#[path = "android_grid_nested_wrap.rs"]
mod android_grid_nested_wrap;
#[path = "android_grid_wrap.rs"]
mod android_grid_wrap;
#[path = "android_two_line_list_item.rs"]
mod android_two_line_list_item;
#[path = "android_table_standalone.rs"]
mod android_table_standalone;
#[path = "android_table_clip.rs"]
mod android_table_clip;
#[path = "android_table_layout.rs"]
mod android_table_layout;
#[path = "android_table_nested_wrap.rs"]
mod android_table_nested_wrap;
#[path = "android_table_wrap.rs"]
mod android_table_wrap;
#[path = "android_relative_layout.rs"]
mod android_relative_layout;
#[path = "android_constraint_layout.rs"]
mod android_constraint_layout;
#[path = "android_relative_nested_wrap.rs"]
mod android_relative_nested_wrap;
#[path = "android_relative_wrap.rs"]
mod android_relative_wrap;
#[path = "android_absolute_layout.rs"]
mod android_absolute_layout;
#[path = "android_zoomcontrols.rs"]
mod android_zoomcontrols;
#[path = "android_zoombutton.rs"]
mod android_zoombutton;
#[path = "android_digitalclock.rs"]
mod android_digitalclock;
#[path = "android_textclock.rs"]
mod android_textclock;
#[path = "android_chronometer.rs"]
mod android_chronometer;
#[path = "android_imageswitcher.rs"]
mod android_imageswitcher;
#[path = "android_textswitcher.rs"]
mod android_textswitcher;
#[path = "android_viewflipper.rs"]
mod android_viewflipper;
#[path = "android_viewswitcher.rs"]
mod android_viewswitcher;
#[path = "android_edittext.rs"]
mod android_edittext;
#[path = "android_scrollview.rs"]
mod android_scrollview;
#[path = "android_horizontal_scroll.rs"]
mod android_horizontal_scroll;
#[path = "android_checkbox.rs"]
mod android_checkbox;
#[path = "android_checkbox_label.rs"]
mod android_checkbox_label;
#[path = "android_switch.rs"]
mod android_switch;
#[path = "android_seekbar.rs"]
mod android_seekbar;
#[path = "android_progressbar.rs"]
mod android_progressbar;
#[path = "android_progressbar_tints.rs"]
mod android_progressbar_tints;
#[path = "android_seekbar_tint_modes.rs"]
mod android_seekbar_tint_modes;
#[path = "android_seekbar_tint_alpha.rs"]
mod android_seekbar_tint_alpha;
#[path = "android_seekbar_tints.rs"]
mod android_seekbar_tints;
#[path = "android_seekbar_ticks.rs"]
mod android_seekbar_ticks;
#[path = "android_seekbar_split.rs"]
mod android_seekbar_split;
#[path = "android_seekbar_thumb.rs"]
mod android_seekbar_thumb;
#[path = "android_seekbar_tick_selector.rs"]
mod android_seekbar_tick_selector;
#[path = "android_radio_group.rs"]
mod android_radio_group;
#[path = "android_radio_label.rs"]
mod android_radio_label;
#[path = "android_radio_group_nested.rs"]
mod android_radio_group_nested;
#[path = "android_radio_group_tint_alpha.rs"]
mod android_radio_group_tint_alpha;
#[path = "android_radio_group_tints.rs"]
mod android_radio_group_tints;
#[path = "android_radio_group_drawable.rs"]
mod android_radio_group_drawable;
#[path = "android_ratingbar.rs"]
mod android_ratingbar;
#[path = "android_ratingbar_tints.rs"]
mod android_ratingbar_tints;
#[path = "android_togglebutton.rs"]
mod android_togglebutton;
#[path = "android_button_wrap.rs"]
mod android_button_wrap;
#[path = "android_button_multiline.rs"]
mod android_button_multiline;
#[path = "android_button_ellipsis.rs"]
mod android_button_ellipsis;
#[path = "android_button_drawables.rs"]
mod android_button_drawables;
#[path = "android_button_drawable_height.rs"]
mod android_button_drawable_height;
#[path = "android_togglebutton_wrap.rs"]
mod android_togglebutton_wrap;
#[path = "android_togglebutton_style.rs"]
mod android_togglebutton_style;
#[path = "android_checked_textview.rs"]
mod android_checked_textview;
#[path = "android_checked_textview_tints.rs"]
mod android_checked_textview_tints;
#[path = "android_checked_textview_tint_modes.rs"]
mod android_checked_textview_tint_modes;
#[path = "android_checked_textview_tint_alpha.rs"]
mod android_checked_textview_tint_alpha;
#[path = "android_checked_textview_compound.rs"]
mod android_checked_textview_compound;
#[path = "android_checked_textview_drawable.rs"]
mod android_checked_textview_drawable;
#[path = "android_checkbox_tint_modes.rs"]
mod android_checkbox_tint_modes;
#[path = "android_checkbox_tint_alpha.rs"]
mod android_checkbox_tint_alpha;
#[path = "android_checkbox_tints.rs"]
mod android_checkbox_tints;
#[path = "android_checkbox_drawable.rs"]
mod android_checkbox_drawable;
#[path = "android_compound_textappearance.rs"]
mod android_compound_textappearance;
#[path = "android_switch_tints.rs"]
mod android_switch_tints;
#[path = "android_switch_text.rs"]
mod android_switch_text;
#[path = "android_switch_layout.rs"]
mod android_switch_layout;
#[path = "android_space.rs"]
mod android_space;
#[path = "android_frame_layout.rs"]
mod android_frame_layout;
#[path = "android_frame_wrap.rs"]
mod android_frame_wrap;
#[path = "android_linear_layout.rs"]
mod android_linear_layout;
#[path = "android_button_multiline_baseline.rs"]
mod android_button_multiline_baseline;
#[path = "android_linear_wrap.rs"]
mod android_linear_wrap;
#[path = "android_linear_baseline.rs"]
mod android_linear_baseline;
#[path = "android_linear_editable_baseline.rs"]
mod android_linear_editable_baseline;
#[path = "android_linear_compound_baseline.rs"]
mod android_linear_compound_baseline;
#[path = "android_linear_nested_baseline.rs"]
mod android_linear_nested_baseline;
#[path = "android_imageview_baseline.rs"]
mod android_imageview_baseline;
#[path = "android_imageview_matrix.rs"]
mod android_imageview_matrix;
#[path = "android_imageview_crop_padding.rs"]
mod android_imageview_crop_padding;
#[path = "android_imageview.rs"]
mod android_imageview;
#[path = "android_imagebutton.rs"]
mod android_imagebutton;
#[path = "android_textview.rs"]
mod android_textview;
#[path = "android_textview_grapheme.rs"]
mod android_textview_grapheme;
#[path = "android_textview_separator_width.rs"]
mod android_textview_separator_width;
#[path = "android_textappearance.rs"]
mod android_textappearance;
#[path = "android_textview_compound.rs"]
mod android_textview_compound;
#[path = "android_textview_compound_sizes.rs"]
mod android_textview_compound_sizes;
#[path = "android_textview_compound_selector.rs"]
mod android_textview_compound_selector;
#[path = "android_textview_tint_modes.rs"]
mod android_textview_tint_modes;
#[path = "android_textview_compound_tint.rs"]
mod android_textview_compound_tint;

pub const FAMILIES: &str = include_str!("families.txt");
pub struct Gallery {
    pub text: String,
    pub touched: bool,
    subscript_invalid: bool,
    subscript_started: Option<f64>,
    snackbar_host: bhoo::snackbar::Host,
    snackbar_presentation: bhoo::snackbar::Presentation,
    snackbar_sequence: u64,
    pub edit: Vec<u8>,
    selected: usize,
    rating: usize,
    large_page: usize,
    mask: u32,
    open: bool,
    overlay: OverlayState,
    child_dialog: OverlayState,
    menu_typeahead: MenuTypeaheadState,
    select_typeahead: MenuTypeaheadState,
    anchor: Rect,
    value: f32,
    page: usize,
    step: usize,
    day: usize,
    end_day: usize,
    month: i32,
    year: i32,
    selected_date: [i32; 3],
    range_end_date: [i32; 3],
    calendar_focus: usize,
    started: Option<f64>,
    notice: String,
    radio_group_notice: String,
    drawer_event_history: Vec<bhoo::android_xml::SlidingDrawerEvent>,
    media_pending_seek: Option<u64>,
    video_host_active: bool,
    video_host_command: Option<bhoo::android_xml::MediaControllerAction>,
    grid: ListState,
    pull_refresh: bhoo::pull_refresh::PullRefreshState,
    horizontal_example: bhoo::android_xml::LayoutState,
    android_checkbox_example: bhoo::android_xml::LayoutState,
    android_button_example: bhoo::android_xml::LayoutState,
    android_switch_example: bhoo::android_xml::LayoutState,
    android_seekbar_example: bhoo::android_xml::LayoutState,
    android_progressbar_example: bhoo::android_xml::LayoutState,
    android_radio_example: bhoo::android_xml::LayoutState,
    android_rating_example: bhoo::android_xml::LayoutState,
    android_toggle_example: bhoo::android_xml::LayoutState,
    android_frame_example: bhoo::android_xml::LayoutState,
    android_linear_example: bhoo::android_xml::LayoutState,
    android_image_example: bhoo::android_xml::LayoutState,
    android_imagebutton_example: bhoo::android_xml::LayoutState,
    android_text_example: bhoo::android_xml::LayoutState,
    android_edit_example: bhoo::android_xml::LayoutState,
    android_clock_example: bhoo::android_xml::LayoutState,
    time_scrollbar: ScrollbarDragState,
    demo_scrollbar: ScrollbarDragState,
    indicator_scroll: f32,
    indicator_state: ScrollbarIndicatorState,
    avatar_sample: Option<Image>,
    loading: [LoadingState; 3],
    shape_rotation: [ShapeRotationState; 3],
    wavy_shapes: [ShapeRotationState; 2],
    button_group: ButtonGroupState,
    button_group_icons: ButtonGroupState,
    button_group_connected_single: ButtonGroupState,
    button_group_connected_multi: ButtonGroupState,
    button_group_rtl: bool,
    button_group_narrow: bool,
    button_group_many: bool,
    toolbars: [ToolbarState; 4],
    docked_scroll: DockedToolbarScrollState,
    floating_fab: ToolbarState,
    floating_scroll_expansion: FloatingToolbarExpansionScrollState,
    floating_scroll_expanded: bool,
    fab_menu: FabMenuState,
    carousel: CarouselState,
    top_app_bar: TopAppBarState,
    top_bar_behavior: usize,
    bottom_bar: BottomAppBarState,
    bottom_overflow: BottomAppBarOverflowState,
    rail_initialized: bool,
    media_image: Option<Image>,
}
impl Default for Gallery {
    fn default() -> Self {
        Self {
            text: String::new(),
            touched: false,
            subscript_invalid: false,
            subscript_started: None,
            snackbar_host: bhoo::snackbar::Host::default(),
            snackbar_presentation: bhoo::snackbar::Presentation::default(),
            snackbar_sequence: 0,
            edit: vec![0; 2048],
            selected: 0,
            rating: 2,
            large_page: 25,
            mask: 0,
            open: false,
            overlay: OverlayState::default(),
            child_dialog: OverlayState::default(),
            menu_typeahead: MenuTypeaheadState::default(),
            select_typeahead: MenuTypeaheadState::default(),
            anchor: Rect::default(),
            value: 0.0,
            page: 0,
            step: 0,
            day: 0,
            end_day: 0,
            month: 0,
            year: 0,
            selected_date: [2026, 11, 9],
            range_end_date: [2026, 11, 16],
            calendar_focus: 9,
            started: None,
            notice: String::new(),
            radio_group_notice: String::new(),
            drawer_event_history: Vec::new(),
            media_pending_seek: None,
            video_host_active: false,
            video_host_command: None,
            grid: ListState::default(),
            pull_refresh: bhoo::pull_refresh::PullRefreshState::default(),
            horizontal_example: bhoo::android_xml::LayoutState::default(),
            android_checkbox_example: bhoo::android_xml::LayoutState::default(),
            android_button_example: bhoo::android_xml::LayoutState::default(),
            android_switch_example: bhoo::android_xml::LayoutState::default(),
            android_seekbar_example: bhoo::android_xml::LayoutState::default(),
            android_progressbar_example: bhoo::android_xml::LayoutState::default(),
            android_radio_example: bhoo::android_xml::LayoutState::default(),
            android_rating_example: bhoo::android_xml::LayoutState::default(),
            android_toggle_example: bhoo::android_xml::LayoutState::default(),
            android_frame_example: bhoo::android_xml::LayoutState::default(),
            android_linear_example: bhoo::android_xml::LayoutState::default(),
            android_image_example: bhoo::android_xml::LayoutState::default(),
            android_imagebutton_example: bhoo::android_xml::LayoutState::default(),
            android_text_example: bhoo::android_xml::LayoutState::default(),
            android_edit_example: bhoo::android_xml::LayoutState::default(),
            android_clock_example: bhoo::android_xml::LayoutState::default(),
            time_scrollbar: ScrollbarDragState::default(),
            demo_scrollbar: ScrollbarDragState::default(),
            indicator_scroll: 0.0,
            indicator_state: ScrollbarIndicatorState::default(),
            avatar_sample: None,
            loading: std::array::from_fn(|_| LoadingState::default()),
            shape_rotation: std::array::from_fn(|_| ShapeRotationState::default()),
            wavy_shapes: std::array::from_fn(|_| ShapeRotationState::default()),
            button_group: ButtonGroupState::default(),
            button_group_icons: ButtonGroupState::default(),
            button_group_connected_single: ButtonGroupState::default(),
            button_group_connected_multi: ButtonGroupState::default(),
            button_group_rtl: false,
            button_group_narrow: false,
            button_group_many: false,
            toolbars: std::array::from_fn(|_| ToolbarState::default()),
            docked_scroll: DockedToolbarScrollState::default(),
            floating_fab: ToolbarState::default(),
            floating_scroll_expansion: FloatingToolbarExpansionScrollState::default(),
            floating_scroll_expanded: true,
            fab_menu: FabMenuState::default(),
            carousel: CarouselState::default(),
            top_app_bar: TopAppBarState::default(),
            top_bar_behavior: 2,
            bottom_bar: BottomAppBarState::default(),
            bottom_overflow: BottomAppBarOverflowState::default(),
            rail_initialized: false,
            media_image: None,
        }
    }
}
impl Gallery {
    pub fn update_date_range_text(&mut self, text: &str) {
        fn parse_date(value: &str) -> Option<[i32; 3]> {
            let mut parts = value.trim().split('/');
            let month: i32 = parts.next()?.parse().ok()?;
            let day: i32 = parts.next()?.parse().ok()?;
            let year: i32 = parts.next()?.parse().ok()?;
            if parts.next().is_some()
                || !(1..=9999).contains(&year)
                || !(1..=12).contains(&month)
                || day < 1
                || day as usize > month_days(year, month)
            {
                return None;
            }
            Some([year, month, day])
        }
        let mut values = text.split('–');
        if let Some(start) = values.next().and_then(parse_date) {
            self.selected_date = start;
            self.year = start[0];
            self.month = start[1];
            self.day = start[2] as usize;
            self.calendar_focus = self.day;
            self.mask |= 1;
            if let Some(end) = values
                .next()
                .and_then(parse_date)
                .filter(|end| *end >= start)
            {
                self.range_end_date = end;
                self.end_day = end[2] as usize;
                self.step = 0;
            } else {
                self.range_end_date = start;
                self.step = 1;
            }
        } else {
            self.mask &= !1;
            self.step = 0;
        }
    }
    pub fn pause(&mut self) {
        for state in &mut self.loading {
            state.pause();
        }
        for state in &mut self.shape_rotation {
            state.pause();
        }
        for state in &mut self.wavy_shapes {
            state.pause();
        }
    }
    pub fn overlay_open(&self) -> bool {
        self.overlay.open
    }
    pub fn fab_menu_open(&self) -> bool {
        self.open
    }
    pub fn new() -> Self {
        Self {
            value: 40.0,
            day: 9,
            end_day: 16,
            month: 11,
            year: 2026,
            ..Self::default()
        }
    }
    pub fn set_android_calendar_locale(&mut self, data:&str) {if let Some(locale)=bhoo::android_calendar::CalendarLocale::from_host(data) {self.android_clock_example.rtl=locale.rtl;self.android_linear_example.rtl=locale.rtl;self.android_linear_example.calendar_locale=locale.clone();self.android_clock_example.calendar_locale=locale;}}
    pub fn set_android_wall_clock(&mut self, unix_seconds:f64, offset_seconds:i32, is_24_hour:bool) {
        self.android_clock_example.wall_clock=Some(bhoo::android_xml::WallClock { unix_seconds, offset_seconds, is_24_hour });self.android_linear_example.wall_clock=self.android_clock_example.wall_clock;
    }
    pub fn set_android_edit_text(&mut self, id: u64, text: &str) {
        if id==bhoo::android_xml::view_id("dialer") && self.android_linear_example.dialer_filters.contains_key("dialer") {let filter=self.android_linear_example.dialer_filters.get_mut("dialer").unwrap();filter.replace_user_text(text);self.android_linear_example.text.insert("dialer".into(),filter.filter_text().to_owned());return;}
        if let Some(name)=self.android_linear_example.text.keys().find(|name|bhoo::android_xml::view_id(name)==id).cloned() {if self.android_linear_example.search_open.contains_key(&name){self.android_linear_example.set_search_query(&name,text);}else{self.android_linear_example.text.insert(name.clone(),text.to_owned());self.android_linear_example.update_time_clock_input(&name);}return;}
        if let Some((_, value)) = self.android_edit_example.text.iter_mut().find(|(name, _)| bhoo::android_xml::view_id(name) == id) { *value = text.to_owned(); }
    }
    pub fn set_android_edit_selection(&mut self,id:u64,end:usize) {
        if let Some(name)=self.android_linear_example.text.keys().find(|name|bhoo::android_xml::view_id(name)==id).cloned() {self.android_linear_example.text_selection_end.insert(name,end);return;}
        if let Some(name)=self.android_edit_example.text.keys().find(|name|bhoo::android_xml::view_id(name)==id).cloned() {self.android_edit_example.text_selection_end.insert(name,end);}
    }
    pub fn android_edit_selection(&self,id:u64)->usize {self.android_linear_example.text_selection_end.iter().chain(self.android_edit_example.text_selection_end.iter()).find(|(name,_)|bhoo::android_xml::view_id(name)==id).map_or(0,|(_,end)|*end)}
    pub fn android_autocomplete_open(&self,id:u64)->bool {self.android_edit_example.text.keys().find(|name|bhoo::android_xml::view_id(name)==id).is_some_and(|name|self.android_edit_example.autocomplete_is_open(name))||self.android_linear_example.text.keys().find(|name|bhoo::android_xml::view_id(name)==id).is_some_and(|name|self.android_linear_example.autocomplete_is_open(name))}
    pub fn android_autocomplete_highlighted(&self,id:u64)->bool {self.android_edit_example.text.keys().find(|name|bhoo::android_xml::view_id(name)==id).is_some_and(|name|self.android_edit_example.autocomplete_has_highlight(name))||self.android_linear_example.text.keys().find(|name|bhoo::android_xml::view_id(name)==id).is_some_and(|name|self.android_linear_example.autocomplete_has_highlight(name))}
    pub fn take_android_search_focus(&mut self,id:u64)->bool {if let Some(key)=self.android_linear_example.search_open.keys().find(|name|bhoo::android_xml::view_id(name)==id).cloned(){self.android_linear_example.take_search_focus_request(&key)}else{false}}
    pub fn submit_android_search(&mut self,id:u64)->bool {if let Some(key)=self.android_linear_example.search_open.keys().find(|name|bhoo::android_xml::view_id(name)==id).cloned(){self.android_linear_example.submit_search(&key)}else{false}}
    pub fn finish_android_number_edit(&mut self,id:u64,commit:bool) {if let Some(key)=self.android_linear_example.number_picker_editing.iter().find(|name|bhoo::android_xml::view_id(name)==id).cloned(){self.android_linear_example.end_number_picker_edit(&key,commit);}}
    pub fn android_edit_input_type(&self, id: u64) -> u8 {
        if let Some((_,kind))=self.android_linear_example.input_types.iter().find(|(name,_)|bhoo::android_xml::view_id(name)==id){return *kind;}
        self.android_edit_example.input_types.iter().find(|(name, _)| bhoo::android_xml::view_id(name) == id).map_or(0, |(_, kind)| *kind)
    }
    pub fn android_edit_text(&self, id: u64) -> &str {
        if let Some((_,text))=self.android_linear_example.text.iter().find(|(name,_)|bhoo::android_xml::view_id(name)==id){return text;}
        self.android_edit_example.text.iter().find(|(name, _)| bhoo::android_xml::view_id(name) == id).map_or("", |(_, value)| value.as_str())
    }
    pub fn video_host_frame(&mut self,image:Image) {self.video_host_active=true;self.android_linear_example.video_frames.insert("video".into(),image);}
    pub fn video_host_status(&mut self,ready:bool,width:u32,height:u32,position_ms:u64,duration_ms:u64,playing:bool,buffered_percent:u8) {
        self.video_host_active=true;
        if ready {self.android_linear_example.video_ready.insert("video".into());if width>0&&height>0{self.android_linear_example.video_dimensions.insert("video".into(),(width,height));}}else{self.android_linear_example.video_ready.remove("video");self.android_linear_example.hide_floating_media_controller("controller");}
        self.android_linear_example.media_players.insert("controller".into(),bhoo::android_xml::MediaPlayerSnapshot{position_ms,duration_ms,playing,buffered_percent:buffered_percent.min(100)});
        self.android_linear_example.media_controller_capabilities.insert("controller".into(),bhoo::android_xml::MediaControllerCapabilities::default());
    }
    pub fn take_video_host_command(&mut self)->Option<bhoo::android_xml::MediaControllerAction>{self.video_host_command.take()}
    pub fn show(&mut self, f: &mut Frame<'_>, scene: u32) {
        let slug = FAMILIES
            .lines()
            .nth(scene.saturating_sub(19) as usize)
            .expect("registered scene");
        match slug {
            "floating-action-button" => {
                f.label("Create — small, regular, medium, large and extended");
                let mut rtl = self.mask & 1 != 0;
                let mut lowered = self.mask & 2 != 0;
                let mut flat = self.mask & 4 != 0;
                if f.switch(90, "Right-to-left layout", &mut rtl, true).changed {
                    self.mask ^= 1;
                }
                f.row(48.0, 2, |f, i| {
                    if i == 0 && f.switch(91, "Lowered", &mut lowered, true).changed {
                        self.mask ^= 2;
                        self.mask &= !4;
                    }
                    if i == 1 && f.switch(92, "Bar flat", &mut flat, true).changed {
                        self.mask ^= 4;
                        self.mask &= !2;
                    }
                });
                let mut round=self.mask&32!=0;if f.switch(95,"Round shape",&mut round,true).changed {self.mask^=32;}
                let elevation = if self.mask & 2 != 0 {
                    FabElevation::Lowered
                } else if self.mask & 4 != 0 {
                    FabElevation::BottomAppBar
                } else {
                    FabElevation::Standard
                };
                for (i, size) in [40.0, 56.0, 80.0, 96.0].into_iter().enumerate() {
                    if f.fab_with_options(
                        100 + i as u64,
                        ["Secondary small", "Tertiary regular", "Primary medium", "Surface large"][i],
                        AppBarIcon::Add,
                        FabOptions {
                            size,
                            corner_radius:if round{Some(size/2.0)}else{None},
                            elevation,
                            color: [FabColor::Secondary, FabColor::Tertiary, FabColor::Primary, FabColor::Surface][i],
                            ..FabOptions::default()
                        },
                    )
                    .clicked
                    {
                        self.step += 1;
                    }
                }
                let mut expanded=self.mask&8==0;if f.switch(93,"Extended label",&mut expanded,true).changed {self.mask^=8;}
                let mut narrow=self.mask&16!=0;if f.switch(94,"Narrow container",&mut narrow,true).changed {self.mask^=16;}
                let mut bounds=f.reserve(72.0);bounds.y+=8.0;bounds.height=56.0;if narrow{bounds.width=bounds.width.min(40.0);}
                f.within_unclipped(bounds,|f|{if f.fab_with_options(110,"Create item",AppBarIcon::Add,FabOptions{extended:expanded,rtl,elevation,corner_radius:if round{Some(28.0)}else{None},..FabOptions::default()}).clicked {self.step+=1;}});
                let mut large_extended=self.mask&64!=0;if f.switch(96,"Show larger extended FABs",&mut large_extended,true).changed {self.mask^=64;}
                if large_extended {for (i,size) in [80.0,96.0].into_iter().enumerate(){if f.fab_with_options(111+i as u64,["Medium create","Large create"][i],AppBarIcon::Add,FabOptions{size,extended:true,rtl,elevation,corner_radius:if round{Some(size/2.0)}else{None},..FabOptions::default()}).clicked {self.step+=1;}}}
                f.label(format!("Created: {}", self.step));
            }
            "segmented-selection-toggle-group" => {
                f.label("Single selection");
                let mut mask = 1 << self.selected;
                f.segmented(100, &["Day", "Week", "Month"], &mut mask, false, true);
                self.selected = mask.trailing_zeros() as usize;
                f.label("Multiple selection");
                f.segmented(110, &["Bold", "Italic", "Mono"], &mut self.mask, true, true);
            }
            "button-group" => {
                f.switch(90, "Right-to-left layout", &mut self.button_group_rtl, true);
                f.switch(
                    91,
                    "Narrow groups",
                    &mut self.button_group_narrow,
                    true,
                );
                f.switch(
                    92,
                    "More actions for scrolling",
                    &mut self.button_group_many,
                    true,
                );
                f.label("Standard group · press to expand");
                let mut bounds = f.reserve(48.0);
                if self.button_group_narrow {
                    bounds.width = bounds.width.min(120.0);
                }
                let actions = [
                    ToolbarAction {
                        label: "Cut",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Copy",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Paste",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Duplicate",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Rename",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Move",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Share",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Archive",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Restore",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Export",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Details",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Delete",
                        enabled: true,
                    },
                ];
                let actions = &actions[..if self.button_group_many {
                    actions.len()
                } else {
                    3
                }];
                if let Some(i) = f.button_group_with_overflow(
                    100,
                    bounds,
                    actions,
                    &mut self.button_group,
                    self.button_group_rtl,
                ) {
                    self.notice = actions[i].label.into();
                }
                f.label(&self.notice);
                f.label("Icons and labels");
                let mut icon_bounds = f.reserve(48.0);
                if self.button_group_narrow { icon_bounds.width = icon_bounds.width.min(120.0); }
                let icon_actions = [
                    ToolbarAction {
                        label: "Back",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Find",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "New",
                        enabled: true,
                    },
                ];
                let icons = [
                    Some(AppBarIcon::Back),
                    Some(AppBarIcon::Search),
                    Some(AppBarIcon::Add),
                ];
                if let Some(i) = f.button_group_with_overflow_icons(
                    150,
                    icon_bounds,
                    &icon_actions,
                    &icons,
                    &mut self.button_group_icons,
                    self.button_group_rtl,
                ) {
                    self.notice = icon_actions[i].label.into();
                }
                f.label("Connected · single selection required");
                let mut bounds = f.reserve(48.0);
                if self.button_group_narrow { bounds.width = bounds.width.min(120.0); }
                let single = [
                    ToolbarAction {
                        label: "Day",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Week",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Month",
                        enabled: true,
                    },
                ];
                let mut chosen = 1_u32 << self.selected.min(2);
                if f.connected_button_group_with_overflow(
                    110,
                    bounds,
                    &single,
                    &mut chosen,
                    false,
                    true,
                    &mut self.button_group_connected_single,
                    self.button_group_rtl,
                )
                .is_some()
                {
                    self.selected = chosen.trailing_zeros() as usize;
                }
                f.label("Connected · multiple selection");
                let mut bounds = f.reserve(48.0);
                if self.button_group_narrow { bounds.width = bounds.width.min(120.0); }
                let multiple = [
                    ToolbarAction {
                        label: "Bold",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Italic",
                        enabled: true,
                    },
                    ToolbarAction {
                        label: "Mono",
                        enabled: true,
                    },
                ];
                f.connected_button_group_with_overflow(
                    120,
                    bounds,
                    &multiple,
                    &mut self.mask,
                    true,
                    false,
                    &mut self.button_group_connected_multi,
                    self.button_group_rtl,
                );
            }
            "split-button" => {
                f.label("Split button");
                f.without_input(self.overlay.open, |f| {
                    let mut primary_disabled = self.mask & 1 != 0;
                    let mut menu_disabled = self.mask & 2 != 0;
                    let mut rtl = self.mask & 4 != 0;
                    f.row(48.0, 3, |f, i| {
                        if i == 0
                            && f.switch(90, "Primary off", &mut primary_disabled, true)
                                .changed
                        {
                            self.mask ^= 1;
                        }
                        if i == 1 && f.switch(91, "Menu off", &mut menu_disabled, true).changed {
                            self.mask ^= 2;
                        }
                        if i == 2 && f.switch(92, "RTL", &mut rtl, true).changed {
                            self.mask ^= 4;
                        }
                    });
                    let slot = f.reserve(48.0);
                    let bounds = Rect {
                        width: slot.width.min(220.0),
                        ..slot
                    };
                    if let Some((primary, trailing)) = f.split_button_with(
                        100,
                        bounds,
                        "Save",
                        "More save options",
                        SplitButtonOptions {
                            expanded: self.overlay.open && self.step == 0,
                            variant: ButtonVariant::Filled,
                            primary_enabled: !primary_disabled,
                            menu_enabled: !menu_disabled,
                            rtl,
                            ..SplitButtonOptions::default()
                        },
                    ) {
                        if primary.clicked {
                            self.notice = "Saved".into();
                        }
                        if trailing.clicked {
                            self.anchor = trailing.bounds;
                            self.overlay.open = !self.overlay.open;
                            self.step = 0;
                        }
                    }
                    f.label("More sizes and styles");
                    for (i, (label, size, variant, height)) in [
                        (
                            "Extra small",
                            SplitButtonSize::ExtraSmall,
                            ButtonVariant::Tonal,
                            48.0,
                        ),
                        (
                            "Medium",
                            SplitButtonSize::Medium,
                            ButtonVariant::Outlined,
                            56.0,
                        ),
                        ("Large", SplitButtonSize::Large, ButtonVariant::Tonal, 96.0),
                        (
                            "Extra large",
                            SplitButtonSize::ExtraLarge,
                            ButtonVariant::Elevated,
                            136.0,
                        ),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let slot = f.reserve(height);
                        let bounds = Rect {
                            width: slot.width.min(300.0),
                            ..slot
                        };
                        let config = SplitButtonOptions {
                            size,
                            variant,
                            expanded: self.overlay.open && self.step == i + 1,
                            primary_enabled: !primary_disabled,
                            menu_enabled: !menu_disabled,
                            rtl,
                            ..SplitButtonOptions::default()
                        };
                        if let Some((primary, trailing)) = f.split_button_with(
                            110 + i as u64 * 2,
                            bounds,
                            label,
                            "More options",
                            config,
                        ) {
                            if primary.clicked {
                                self.notice = format!("{label} activated");
                            }
                            if trailing.clicked {
                                self.anchor = trailing.bounds;
                                self.step = i + 1;
                                self.overlay.open = true;
                            }
                        }
                    }
                });
                f.label(&self.notice);
                self.menu(f, &["Save a copy", "Export", "Print"], false, None);
            }
            "floating-action-menu" => {
                f.label("Create something new");
                let mut rtl = self.mask & 1 != 0;
                let mut compact = self.mask & 2 != 0;
                if f.switch(90, "Right-to-left layout", &mut rtl, true).changed {
                    self.mask = (self.mask & !1) | u32::from(rtl);
                }
                if f.switch(91, "Short window · scroll actions", &mut compact, true)
                    .changed
                {
                    self.mask = (self.mask & !2) | (u32::from(compact) << 1);
                }
                let bounds = f.reserve(if compact { 156.0 } else { 300.0 });
                let actions = [
                    FabMenuAction {
                        label: "New document",
                        glyph: "+",
                        enabled: true,
                    },
                    FabMenuAction {
                        label: "New folder",
                        glyph: "+",
                        enabled: true,
                    },
                    FabMenuAction {
                        label: "Import",
                        glyph: "↓",
                        enabled: true,
                    },
                ];
                if let Some(i) = f.fab_menu_with(
                    100,
                    bounds,
                    &mut self.open,
                    &actions,
                    &mut self.fab_menu,
                    FabMenuOptions { rtl },
                ) {
                    self.notice = format!("Selected: {}", actions[i].label);
                }
                f.label(&self.notice);
            }
            "text-field"
            | "multiline-text-field"
            | "search"
            | "autocomplete"
            | "form-field-container"
            | "command-palette" => self.text_demo(f, slug),
            "select" => {
                f.label("Power profile");
                let mut long=self.mask&1!=0;
                if f.switch(98,"Long option labels",&mut long,true).changed {self.mask^=1;}
                let labels = if long {["Balanced profile with a long descriptive name", "Performance profile with a long descriptive name", "Battery saver profile with a long descriptive name"]}else{["Balanced", "Performance", "Battery saver"]};
                if !self.overlay.open {
                    let items:Vec<_>=labels.iter().map(|label|(*label,true)).collect();
                    if let Some(index)=f.select_typeahead(&mut self.select_typeahead,100,self.selected.min(labels.len()-1),&items) {self.selected=index;}
                } else {self.select_typeahead.reset();}
                let r = f.select_field(100, "Power profile", labels[self.selected.min(2)], false);
                self.anchor = r.bounds;
                let mut initial=self.selected.min(labels.len()-1);
                if r.clicked {
                    self.overlay.open = true;
                } else if r.focused && !self.overlay.open {
                    if f.pressed_key(Key::Down) {self.overlay.open=true;}
                    else if f.pressed_key(Key::Up)||f.pressed_key(Key::Home) {initial=0;self.overlay.open=true;}
                    else if f.pressed_key(Key::End) {initial=labels.len()-1;self.overlay.open=true;}
                }
                f.label("Arrow keys move through items; Escape dismisses.");
                self.overlay.initial_focus = Some(600 + initial as u64);
                self.menu(f, &labels, false, Some(self.selected));
            }
            "date-picker" => self.calendar(f, false),
            "date-range-picker" => {
                let (r, toggle) =
                    f.date_range_field(90, "Date range", &self.text, self.overlay.open);
                self.anchor = r.bounds;
                if toggle.clicked {
                    self.overlay.open = true;
                }
                let viewport = f.viewport();
                let b = Rect {
                    x: r.bounds.x.min((viewport.width - 296.0).max(0.0)),
                    y: r.bounds.y + 56.0,
                    width: 296.0_f32.min(viewport.width),
                    height: 354.0,
                };
                let mut overlay = std::mem::take(&mut self.overlay);
                overlay.kind = OverlayKind::Popover;
                overlay.style = OverlayStyle::AngularDatepicker;
                f.overlay(500, &mut overlay, b, false, |f, open| {
                    let before = self.step;
                    self.calendar(f, true);
                    if before == 1 && self.step == 0 {
                        *open = false;
                    }
                });
                self.overlay = overlay;
            }
            "time-picker" => {
                let was_open = self.overlay.open;
                let r = f.popup_field(90, "Select time", &self.text, true, FieldAffordance::Clock);
                self.anchor = r.bounds;
                if r.clicked {
                    self.overlay.open = true;
                    let selected = *self.grid.selected.get_or_insert(0);
                    let top = selected as f32 * 48.0;
                    if top < self.grid.offset {
                        self.grid.offset = top;
                    }
                    if top + 48.0 > self.grid.offset + 240.0 {
                        self.grid.offset = top + 48.0 - 240.0;
                    }
                }
                let b = Rect {
                    x: r.bounds.x,
                    y: r.bounds.y + 56.0,
                    width: r.bounds.width,
                    height: 256.0,
                };
                let mut overlay = std::mem::take(&mut self.overlay);
                overlay.kind = OverlayKind::Popover;
                overlay.style = OverlayStyle::AngularTimepicker;
                f.overlay(500, &mut overlay, b, false, |f, open| {
                    let t = f.theme();
                    let list_bounds = f.available();
                    f.within(
                        Rect {
                            width: (list_bounds.width - 15.0).max(0.0),
                            ..list_bounds
                        },
                        |f| {
                            f.virtual_list_rows(
                                510,
                                240.0,
                                48,
                                &mut self.grid,
                                48.0,
                                |f, index, active| {
                                    let label = time_picker_label(index);
                                    let row = f.reserve(48.0);
                                    let r = f.target(
                                        600 + index as u64,
                                        row,
                                        &label,
                                        true,
                                        Role::ListItem,
                                    );
                                    let opacity = if active {
                                        0.12
                                    } else if r.hovered {
                                        0.08
                                    } else {
                                        0.0
                                    };
                                    if opacity > 0.0 {
                                        f.paint_rect(row, 0.0, t.text.opacity(opacity));
                                    }
                                    f.styled_text(
                                        row,
                                        &label,
                                        t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
                                        TextStyle {
                                            padding: 16.0,
                                            letter_spacing: 0.5,
                                            ..TextStyle::default()
                                        },
                                    );
                                    if r.clicked {
                                        self.text = label;
                                        *open = false;
                                    }
                                    r
                                },
                            );
                        },
                    );
                    if was_open
                        && f.pressed_key(Key::Enter)
                        && let Some(index) = self.grid.selected
                    {
                        self.text = time_picker_label(index);
                        *open = false;
                    }
                    // Match the reference's desktop scrollbar while retaining wheel scrolling.
                    let track = Rect {
                        x: list_bounds.x + list_bounds.width - 15.0,
                        y: list_bounds.y - 8.0,
                        width: 15.0,
                        height: 256.0,
                    };
                    f.paint_rect(track, 0.0, t.background);
                    let max = (48.0_f32 * 48.0 - 240.0).max(1.0);
                    let thumb_h = 24.0;
                    let thumb = Rect {
                        x: track.x + 4.0,
                        y: track.y + 16.0
                            + (self.grid.offset / max).clamp(0.0, 1.0)
                                * (track.height - 32.0 - thumb_h),
                        width: 8.0,
                        height: thumb_h,
                    };
                    f.scrollbar_thumb_drag(
                        509,
                        thumb,
                        Rect { y: track.y + 16.0, height: track.height - 32.0, ..track },
                        max,
                        &mut self.grid.offset,
                        &mut self.time_scrollbar,
                    );
                    f.paint_rect(thumb, 4.0, t.text.opacity(0.52));
                    for (y, direction) in
                        [(track.y + 5.0, 1.0), (track.y + track.height - 5.0, -1.0)]
                    {
                        for row in 0..3 {
                            let half = row as f32 + 0.5;
                            let cy = y + direction * row as f32;
                            f.paint_line(
                                [track.x + 7.5 - half, cy],
                                [track.x + 7.5 + half, cy],
                                1.0,
                                t.text.opacity(0.52),
                            );
                        }
                    }
                });
                self.overlay = overlay;
            }
            "toolbar" | "floating-toolbar" => {
                let active = self.toolbars.iter().position(ToolbarState::overflow_open);
                let mut rtl = self.mask & 1 != 0;
                let mut narrow = self.mask & 2 != 0;
                let mut many = self.mask & 16 != 0;
                f.without_input(active.is_some(), |f| {
                    f.row(48.0, 3, |f, i| {
                        if i == 0 && f.switch(90, "RTL", &mut rtl, true).changed {
                            self.mask ^= 1;
                        }
                        if i == 1 && f.switch(91, "Narrow", &mut narrow, true).changed {
                            self.mask ^= 2;
                        }
                        if i == 2 && f.switch(94, "Many", &mut many, true).changed {
                            self.mask ^= 16;
                        }
                    });
                });
                if slug=="toolbar" {f.label("Tab to a toolbar; use arrow keys to move between actions.");}
                let mut scroll_demo=self.mask&64!=0;
                let mut vertical_demo=self.mask&1024!=0;
                if slug=="floating-toolbar" {f.row(48.0,2,|f,i|{
                    if i==0 && f.switch(96,"Scroll",&mut scroll_demo,true).changed {self.mask^=64;}
                    if i==1 && f.switch(93,"Vertical",&mut vertical_demo,true).changed {self.mask^=1024;}
                });}
                let mut actions = if slug == "toolbar" {
                    vec![
                        ToolbarAction {
                            label: "Back",
                            enabled: true,
                        },
                        ToolbarAction {
                            label: "Forward",
                            enabled: true,
                        },
                        ToolbarAction {
                            label: "Add",
                            enabled: true,
                        },
                        ToolbarAction {
                            label: "Tabs",
                            enabled: true,
                        },
                        ToolbarAction {
                            label: "More options",
                            enabled: true,
                        },
                    ]
                } else {
                    vec![
                        ToolbarAction {
                            label: "Cut",
                            enabled: true,
                        },
                        ToolbarAction {
                            label: "Copy",
                            enabled: true,
                        },
                        ToolbarAction {
                            label: "Paste",
                            enabled: true,
                        },
                        ToolbarAction {
                            label: "Share",
                            enabled: true,
                        },
                        ToolbarAction {
                            label: "Delete",
                            enabled: true,
                        },
                    ]
                };
                let mut icons = vec![
                    AppBarIcon::Back,
                    AppBarIcon::Forward,
                    AppBarIcon::Add,
                    AppBarIcon::Tab,
                    AppBarIcon::More,
                ];
                if many {
                    for (label, icon) in [
                        ("Search", AppBarIcon::Search),
                        ("Menu", AppBarIcon::Menu),
                        ("Open tab", AppBarIcon::Tab),
                        ("Add item", AppBarIcon::Add),
                        ("Options", AppBarIcon::More),
                    ] {
                        actions.push(ToolbarAction {
                            label,
                            enabled: true,
                        });
                        icons.push(icon);
                    }
                }
                let kind = if slug == "toolbar" {
                    ToolbarKind::Docked
                } else {
                    ToolbarKind::FloatingHorizontal
                };
                if slug=="floating-toolbar" && !scroll_demo {
                    let mut start=self.mask&32!=0;
                    let mut vibrant_fab=self.mask&512!=0;
                    f.without_input(active.is_some(),|f|f.row(48.0,2,|f,i|{
                        if i==0 && f.switch(95,"Start",&mut start,true).changed {self.mask^=32;}
                        if i==1 && f.switch(99,"Vibrant",&mut vibrant_fab,true).changed {self.mask^=512;}
                    }));
                    let mut expanded=self.mask&128==0;
                    f.without_input(active.is_some(),|f|{if f.switch(97,"Expanded",&mut expanded,true).changed {self.mask^=128;}});
                    let slot=f.reserve(if vertical_demo {144.0}else{88.0});
                    let bounds=if vertical_demo {Rect{width:slot.width.min(80.0),height:144.0,..slot}}else{Rect{width:slot.width.min(272.0),height:80.0,..slot}};
                    let count=actions.len().min(if vertical_demo {1}else if bounds.width>=176.0 {2}else{1});
                    let mut selected=None;
                    f.without_input(active.is_some(),|f|{
                        selected=f.floating_toolbar_with_fab_expanded(2000,bounds,if vertical_demo{ToolbarKind::FloatingVertical}else{ToolbarKind::FloatingHorizontal},vibrant_fab,&actions[..count],&mut self.floating_fab,rtl,ToolbarAction{label:"Create",enabled:true},AppBarIcon::Add,if start{ToolbarFabPosition::Start}else{ToolbarFabPosition::End},expanded);
                    });
                    if let Some(choice)=selected {self.notice=match choice {ToolbarFabSelection::Fab=>"Create selected".into(),ToolbarFabSelection::Action(i)=>format!("{} selected",actions[i].label)};}
                    f.label(if self.notice.is_empty(){"Choose an action"}else{&self.notice});
                }
                let mut scroll_view = None;
                let mut compact_scroll=self.mask&256!=0;
                if slug=="floating-toolbar" && scroll_demo && f.switch(98,"Collapse into FAB",&mut compact_scroll,true).changed {self.mask^=256;}
                let floating_fourth = if slug == "floating-toolbar" && scroll_demo {
                    f.label(if compact_scroll {"Swipe content to collapse or expand."}else{"Swipe content to hide or show the toolbar."});
                    let viewport=f.reserve(160.0);
                    scroll_view=Some(viewport);
                    let mut consumed=0.0;
                    f.without_input(active.is_some(),|f|{
                        consumed+=f.top_app_bar_scroll_input(viewport,&mut self.top_app_bar,TopAppBarVariant::Small,TopAppBarScrollBehavior::Pinned,&mut self.grid.offset,500.0);
                        consumed+=f.top_app_bar_scroll_keyboard(900,viewport,&mut self.top_app_bar,TopAppBarVariant::Small,TopAppBarScrollBehavior::Pinned,&mut self.grid.offset,500.0);
                        f.target(900,viewport,"Scrollable floating toolbar content",true,Role::Group);
                    });
                    let distance=72.0;
                    let offset=if compact_scroll {
                        self.floating_scroll_expansion.update(&mut self.floating_scroll_expanded,consumed,false,40.0,40.0);
                        0.0
                    }else{f.floating_toolbar_scroll_offset(&mut self.docked_scroll,consumed,distance)};
                    f.within(viewport,|f|{
                        f.paint_rect(viewport,0.0,f.theme().background);
                        for row in 0..14 {
                            let rect=Rect{x:viewport.x+16.0,y:viewport.y+row as f32*48.0-self.grid.offset,width:(viewport.width-32.0).max(0.0),height:48.0};
                            if rect.y+rect.height>viewport.y && rect.y<viewport.y+viewport.height {f.styled_text(rect,&format!("Content {}",row+1),f.theme().text,TextStyle::default());}
                        }
                    });
                    if compact_scroll {Some(Rect{x:viewport.x+(viewport.width-viewport.width.min(272.0))*0.5,y:viewport.y+viewport.height-88.0,width:viewport.width.min(272.0),height:80.0})}
                    else if offset<distance {Some(Rect{x:viewport.x+(viewport.width-viewport.width.min(256.0))*0.5,y:viewport.y+viewport.height-distance+offset,width:viewport.width.min(256.0),height:56.0})} else {None}
                } else {None};
                f.label(if slug == "toolbar" {
                    "Docked toolbar"
                } else {
                    "Horizontal floating toolbar"
                });
                let slot = f.reserve(64.0);
                let first = Rect {
                    width: slot.width.min(if narrow {
                        if kind == ToolbarKind::Docked {
                            128.0
                        } else {
                            112.0
                        }
                    } else if kind == ToolbarKind::Docked {
                        slot.width
                    } else {
                        256.0
                    }),
                    ..slot
                };
                f.label("Vibrant");
                let slot = f.reserve(64.0);
                let second = Rect {
                    width: slot.width.min(if narrow {
                        if kind == ToolbarKind::Docked {
                            128.0
                        } else {
                            112.0
                        }
                    } else if kind == ToolbarKind::Docked {
                        slot.width
                    } else {
                        256.0
                    }),
                    ..slot
                };
                let third = if slug == "floating-toolbar" {
                    f.label("Vertical floating toolbar");
                    let slot = f.reserve(if narrow { 112.0 } else { 256.0 });
                    Some(Rect {
                        width: slot.width.min(64.0),
                        ..slot
                    })
                } else {
                    None
                };
                let fourth = if slug == "toolbar" {
                    let mut exploration = self.mask & 4 != 0;
                    let mut inset = self.mask & 8 != 0;
                    f.without_input(active.is_some(), |f| {
                        f.row(48.0, 2, |f, i| {
                            if i == 0
                                && f.switch(92, "Touch exploration", &mut exploration, true)
                                    .changed
                            {
                                self.mask ^= 4;
                            }
                            if i == 1 && f.switch(93, "Bottom inset", &mut inset, true).changed {
                                self.mask ^= 8;
                            }
                        });
                    });
                    f.label("Scroll the content to hide or reveal the docked toolbar.");
                    let viewport = f.reserve(200.0);
                    scroll_view = Some(viewport);
                    let bar_height = 64.0 + if inset { 24.0 } else { 0.0 };
                    let content = Rect {
                        height: (viewport.height - bar_height).max(0.0),
                        ..viewport
                    };
                    let mut consumed = 0.0;
                    f.without_input(active.is_some(), |f| {
                        consumed += f.top_app_bar_scroll_input(
                            content,
                            &mut self.top_app_bar,
                            TopAppBarVariant::Small,
                            TopAppBarScrollBehavior::Pinned,
                            &mut self.grid.offset,
                            500.0,
                        );
                        consumed += f.top_app_bar_scroll_keyboard(
                            900,
                            content,
                            &mut self.top_app_bar,
                            TopAppBarVariant::Small,
                            TopAppBarScrollBehavior::Pinned,
                            &mut self.grid.offset,
                            500.0,
                        );
                        f.target(
                            900,
                            content,
                            "Scrollable content; use arrows and page keys",
                            true,
                            Role::Group,
                        );
                    });
                    let offset = f.docked_toolbar_scroll_offset(
                        &mut self.docked_scroll,
                        consumed,
                        bar_height,
                        exploration,
                    );
                    f.within(content, |f| {
                        f.paint_rect(content, 0.0, f.theme().background);
                        for i in 0..14 {
                            let row = Rect {
                                x: content.x + 16.0,
                                y: content.y + i as f32 * 48.0 - self.grid.offset,
                                width: content.width - 32.0,
                                height: 48.0,
                            };
                            if row.y + row.height > content.y && row.y < content.y + content.height
                            {
                                f.styled_text(
                                    row,
                                    &format!("Content {}", i + 1),
                                    f.theme().text,
                                    TextStyle::default(),
                                );
                            }
                        }
                    });
                    if offset < bar_height {
                        Some(Rect {
                            y: viewport.y + viewport.height - bar_height + offset,
                            height: bar_height,
                            ..viewport
                        })
                    } else {
                        None
                    }
                } else {
                    floating_fourth
                };
                f.label(if self.notice.is_empty() {
                    "Choose an action"
                } else {
                    &self.notice
                });
                // Build the open menu last so it receives input and paints above other controls.
                for i in (0..4).filter(|i| Some(*i) != active).chain(active) {
                    let Some(bounds) = [Some(first), Some(second), third, fourth][i] else {
                        continue;
                    };
                    let selected_kind = if i == 2 {
                        ToolbarKind::FloatingVertical
                    } else {
                        kind
                    };
                    let mut chosen = None;
                    let mut pair_chosen = None;
                    let mut render = |f: &mut Frame<'_>| {
                        f.without_input(active.is_some() && active != Some(i), |f| {
                            chosen = if selected_kind == ToolbarKind::Docked {
                                if i == 3 {
                                    f.docked_toolbar_with_insets(
                                        100 + i as u64 * 200,
                                        bounds,
                                        DockedToolbarInsets {
                                            bottom: if self.mask & 8 != 0 { 24.0 } else { 0.0 },
                                            ..DockedToolbarInsets::default()
                                        },
                                        false,
                                        &actions,
                                        &icons,
                                        Some(2),
                                        &mut self.toolbars[i],
                                        rtl,
                                    )
                                } else {
                                    f.docked_toolbar_with_overflow(
                                        100 + i as u64 * 200,
                                        bounds,
                                        i == 1,
                                        &actions,
                                        &icons,
                                        Some(2),
                                        &mut self.toolbars[i],
                                        rtl,
                                    )
                                }
                            } else if i==3 && slug=="floating-toolbar" && compact_scroll {
                                pair_chosen=f.floating_toolbar_with_fab_expanded(700,bounds,ToolbarKind::FloatingHorizontal,false,&actions,&mut self.toolbars[i],rtl,ToolbarAction{label:"Create",enabled:true},AppBarIcon::Add,ToolbarFabPosition::End,self.floating_scroll_expanded);
                                None
                            } else {
                                f.toolbar_with_overflow(
                                    100 + i as u64 * 200,
                                    bounds,
                                    selected_kind,
                                    i == 1,
                                    &actions,
                                    &mut self.toolbars[i],
                                    rtl,
                                )
                            };
                        })
                    };
                    if i == 3 {
                        f.within(scroll_view.unwrap(), &mut render);
                    } else {
                        render(f);
                    }
                    if let Some(i) = chosen {self.notice = format!("{} selected", actions[i].label);}
                    if let Some(choice)=pair_chosen {self.notice=match choice {ToolbarFabSelection::Fab=>"Create selected".into(),ToolbarFabSelection::Action(i)=>format!("{} selected",actions[i].label)};}
                }
            }
            "top-app-bar" => {
                f.tabs(
                    90,
                    &["Small", "Center", "Medium", "Large"],
                    &mut self.step,
                    true,
                );
                let variant = [
                    TopAppBarVariant::Small,
                    TopAppBarVariant::CenterAligned,
                    TopAppBarVariant::Medium,
                    TopAppBarVariant::Large,
                ][self.step.min(3)];
                f.tabs(
                    91,
                    &["Pinned", "Enter always", "Exit until collapsed"],
                    &mut self.top_bar_behavior,
                    true,
                );
                let behavior = [
                    TopAppBarScrollBehavior::Pinned,
                    TopAppBarScrollBehavior::EnterAlways,
                    TopAppBarScrollBehavior::ExitUntilCollapsed,
                ][self.top_bar_behavior.min(2)];
                let mut rtl = self.mask & 1 != 0;
                if f.switch(92, "Right-to-left layout", &mut rtl, true).changed {
                    self.mask ^= 1;
                }
                let nav = AppBarAction {
                    label: "Navigate back",
                    glyph: "‹",
                    enabled: true,
                };
                let actions = [
                    AppBarAction {
                        label: "More",
                        glyph: "⋮",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Search",
                        glyph: "⌕",
                        enabled: true,
                    },
                ];
                f.label("Scroll or focus the content and use arrow keys.");
                let viewport = f.reserve(320.0);
                let before = variant.expanded_height()
                    - (variant.expanded_height() - 64.0)
                        * self.top_app_bar.collapse_fraction(variant);
                let maximum = (20.0_f32 * 48.0 - (viewport.height - before).max(0.0)).max(0.0);
                let content_bounds = Rect {
                    y: viewport.y + before,
                    height: (viewport.height - before).max(0.0),
                    ..viewport
                };
                f.top_app_bar_scroll_input(
                    content_bounds,
                    &mut self.top_app_bar,
                    variant,
                    behavior,
                    &mut self.grid.offset,
                    maximum,
                );
                f.top_app_bar_scroll_keyboard(
                    130,
                    content_bounds,
                    &mut self.top_app_bar,
                    variant,
                    behavior,
                    &mut self.grid.offset,
                    maximum,
                );
                f.within(viewport, |f| {
                    f.with_gap(0.0, |f| {
                        if let Some(chosen) = f.top_app_bar_with_icons(
                            100,
                            "Page title",
                            variant,
                            &self.top_app_bar,
                            Some(nav),
                            &actions,
                            rtl,
                            &[
                                Some(AppBarIcon::Back),
                                Some(AppBarIcon::More),
                                Some(AppBarIcon::Search),
                            ],
                        ) {
                            self.notice = ["Navigate back", "More", "Search"][chosen].to_string();
                        }
                        let content = f.available();
                        let max = (20.0_f32 * 48.0 - content.height).max(0.0);
                        self.grid.offset = self.grid.offset.clamp(0.0, max);
                        let scroll_target = f.target(
                            130,
                            content,
                            "Scrollable content; use arrows, Page Up, Page Down, Home and End",
                            true,
                            Role::Group,
                        );
                        f.within(content, |f| {
                            f.paint_rect(content, 0.0, f.theme().surface_low);
                            for i in 0..20 {
                                let row = Rect {
                                    x: content.x + 16.0,
                                    y: content.y + i as f32 * 48.0 - self.grid.offset,
                                    width: (content.width - 32.0).max(0.0),
                                    height: 48.0,
                                };
                                if row.y + row.height > content.y
                                    && row.y < content.y + content.height
                                {
                                    f.styled_text(
                                        row,
                                        &format!("List item {}", i + 1),
                                        f.theme().text,
                                        TextStyle::default(),
                                    );
                                }
                            }
                        });
                        if f.keyboard_focused(scroll_target) {
                            let t = f.theme();
                            f.paint_line(
                                [content.x, content.y],
                                [content.x + content.width, content.y],
                                2.0,
                                t.primary,
                            );
                            f.paint_line(
                                [content.x, content.y + content.height],
                                [content.x + content.width, content.y + content.height],
                                2.0,
                                t.primary,
                            );
                            f.paint_line(
                                [content.x, content.y],
                                [content.x, content.y + content.height],
                                2.0,
                                t.primary,
                            );
                            f.paint_line(
                                [content.x + content.width, content.y],
                                [content.x + content.width, content.y + content.height],
                                2.0,
                                t.primary,
                            );
                        }
                    })
                });
                f.label(if self.notice.is_empty() {
                    ""
                } else {
                    &self.notice
                });
            }
            "bottom-app-bar" => {
                let blocked = self.bottom_overflow.overlay.open;
                let mut rtl = self.mask & 1 != 0;
                let mut no_fab = self.mask & 2 != 0;
                let mut scroll_hide = self.mask & 4 != 0;
                let mut narrow = self.mask & 8 != 0;
                let mut more_actions = self.mask & 16 != 0;
                f.without_input(blocked, |f| {
                    f.row(48.0, 2, |f, i| {
                        if i == 0 && f.switch(90, "RTL", &mut rtl, true).changed {
                            self.mask ^= 1;
                        }
                        if i == 1 && f.switch(91, "Hide FAB", &mut no_fab, true).changed {
                            self.mask ^= 2;
                        }
                    });
                    f.row(48.0, 2, |f, i| {
                        if i == 0 && f.switch(92, "Scroll hide", &mut scroll_hide, true).changed {
                            self.mask ^= 4;
                        }
                        if i == 1 && f.switch(93, "Narrow", &mut narrow, true).changed {
                            self.mask ^= 8;
                        }
                    });
                    if f.switch(94, "Many actions", &mut more_actions, true)
                        .changed
                    {
                        self.mask ^= 16;
                    }
                });
                let available = f.reserve(360.0);
                let viewport = Rect {
                    width: if narrow {
                        available.width.min(176.0)
                    } else {
                        available.width
                    },
                    ..available
                };
                let bar_height = viewport.height.min(80.0);
                let visible_bar = (bar_height - self.bottom_bar.hidden_offset()).max(0.0);
                let content = Rect {
                    height: (viewport.height - visible_bar).max(0.0),
                    ..viewport
                };
                let max = (20.0_f32 * 48.0 - viewport.height).max(0.0);
                if !blocked {
                    f.bottom_app_bar_scroll_input(
                        130,
                        content,
                        &mut self.bottom_bar,
                        if scroll_hide {
                            BottomAppBarScrollBehavior::ExitAlways
                        } else {
                            BottomAppBarScrollBehavior::Pinned
                        },
                        &mut self.top_app_bar,
                        &mut self.grid.offset,
                        max,
                        bar_height,
                    );
                }
                let content = Rect {
                    height: (viewport.height
                        - (bar_height - self.bottom_bar.hidden_offset()).max(0.0))
                    .max(0.0),
                    ..viewport
                };
                let mut target = Response::default();
                f.without_input(blocked, |f| {
                    target = f.target(
                        130,
                        content,
                        "Scrollable content; use arrows, Page Up, Page Down, Home and End",
                        true,
                        Role::Group,
                    );
                });
                f.within(content, |f| {
                    f.paint_rect(content, 0.0, f.theme().background);
                    for i in 0..20 {
                        let row = Rect {
                            x: content.x + 16.0,
                            y: content.y + i as f32 * 48.0 - self.grid.offset,
                            width: (content.width - 32.0).max(0.0),
                            height: 48.0,
                        };
                        if row.y + row.height > content.y && row.y < content.y + content.height {
                            f.styled_text(
                                row,
                                &format!("List item {}", i + 1),
                                f.theme().text,
                                TextStyle::default(),
                            );
                        }
                    }
                });
                if f.keyboard_focused(target) {
                    f.paint_line(
                        [content.x, content.y],
                        [content.x + content.width, content.y],
                        2.0,
                        f.theme().primary,
                    );
                }
                let actions = [
                    AppBarAction {
                        label: "Menu",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Search",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "More",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Share",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Edit",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Copy",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Move",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Archive",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Print",
                        glyph: "",
                        enabled: true,
                    },
                    AppBarAction {
                        label: "Delete",
                        glyph: "",
                        enabled: true,
                    },
                ];
                let actions = &actions[..if more_actions { actions.len() } else { 3 }];
                let fab = (!no_fab).then_some((
                    AppBarAction {
                        label: "Add",
                        glyph: "+",
                        enabled: true,
                    },
                    Some(AppBarIcon::Add),
                ));
                if let Some(chosen) = f.scrolling_bottom_app_bar_with_overflow(
                    100,
                    viewport,
                    &self.bottom_bar,
                    &mut self.bottom_overflow,
                    actions,
                    &[
                        Some(AppBarIcon::Menu),
                        Some(AppBarIcon::Search),
                        Some(AppBarIcon::More),
                    ],
                    fab,
                    rtl,
                ) {
                    self.notice = format!(
                        "{} requested",
                        actions.get(chosen).map_or("Add", |action| action.label)
                    );
                }
                f.label(&self.notice);
            }
            "navigation-drawer" => {
                let b = f.reserve(350.0);
                let t = f.theme();
                let width = 176.0_f32.min(b.width);
                let drawer = Rect { width, ..b };
                f.navigation_drawer(
                    100,
                    drawer,
                    &[("Home", None), ("Files", None), ("Settings", None)],
                    &mut self.selected,
                );
                let body = Rect {
                    x: b.x + width + 24.0,
                    y: b.y + 24.0,
                    width: (b.width - width - 48.0).max(0.0),
                    height: b.height - 48.0,
                };
                f.paragraph(
                    body,
                    [
                        "Home — overview",
                        "Files — your documents",
                        "Settings — preferences",
                    ][self.selected],
                    t.text,
                    TextStyle::default(),
                    24.0,
                );
            }
            "navigation-bar" | "navigation-rail" => {
                f.label("Destinations");
                if slug == "navigation-bar" {
                    let mut selected_only = self.mask & 1 != 0;
                    let mut rtl = self.mask & 2 != 0;
                    if f.switch(90, "Show selected label only", &mut selected_only, true)
                        .changed
                    {
                        self.mask ^= 1;
                    }
                    if f.switch(91, "Right-to-left layout", &mut rtl, true).changed {
                        self.mask ^= 2;
                    }
                    f.navigation_bar_with_icons(
                        100,
                        &[
                            NavigationDestination {
                                label: "Home",
                                icon: "⌂",
                                enabled: true,
                                badge: None,
                            },
                            NavigationDestination {
                                label: "Files",
                                icon: "▣",
                                enabled: true,
                                badge: Some((BadgeContent::Label("3"), "3 new files")),
                            },
                            NavigationDestination {
                                label: "Settings",
                                icon: "⚙",
                                enabled: true,
                                badge: None,
                            },
                        ],
                        &mut self.selected,
                        rtl,
                        !selected_only,
                        &[
                            NavigationIcon::Home,
                            NavigationIcon::Folder,
                            NavigationIcon::Settings,
                        ],
                    );
                } else {
                    if !self.rail_initialized {
                        self.selected = 2;
                        self.rail_initialized = true;
                    }
                    f.without_input(self.overlay.open, |f| {
                        let expanded = self.mask & 2 != 0;
                        let expansion = f.navigation_rail_expansion(100, expanded);
                        let mut rtl = self.mask & 4 != 0;
                        let mut modal = self.mask & 8 != 0;
                        f.row(48.0, 2, |f, i| {
                            if i == 0 && f.switch(92, "RTL", &mut rtl, true).changed {
                                self.mask ^= 4;
                            }
                            if i == 1 && f.switch(93, "Modal", &mut modal, true).changed {
                                self.mask ^= 8;
                            }
                        });
                        f.with_gap(0.0, |f| {
                            f.reserve(44.0);
                            let slot = f.reserve(48.0);
                            let width = (96.0 + 144.0 * expansion).min(slot.width);
                            let inset = 24.0 - 8.0 * expansion;
                            let x = if rtl {
                                slot.x + width - 48.0 - inset
                            } else {
                                slot.x + inset
                            };
                            let menu = Rect {
                                x,
                                y: slot.y,
                                width: 48.0,
                                height: 48.0,
                            };
                            let response = f.target(
                                91,
                                menu,
                                if expanded {
                                    "Collapse rail"
                                } else {
                                    "Expand rail"
                                },
                                true,
                                Role::Button,
                            );
                            f.state_layer(response, 24.0, f.theme().text);
                            f.paint_navigation_icon(
                                menu.inset(12.0),
                                NavigationIcon::Menu,
                                !expanded,
                                f.theme().text,
                            );
                            if response.clicked {
                                self.mask ^= 2;
                            }
                            f.reserve(8.0);
                            let slot = f.reserve(56.0);
                            let fab_width = (56.0 + 96.0 * expansion).min((width - 40.0).max(0.0));
                            let fab = Rect {
                                x: if rtl {
                                    slot.x + width - 20.0 - fab_width
                                } else {
                                    slot.x + 20.0
                                },
                                y: slot.y,
                                width: fab_width,
                                height: 56.0,
                            };
                            let r = f.target(96, fab, "Add timer", true, Role::Button);
                            f.paint_rect(fab, 16.0, f.theme().primary_container);
                            f.state_layer(r, 16.0, f.theme().on_primary_container);
                            let glyph = Rect {
                                x: if rtl {
                                    fab.x + fab.width - 40.0
                                } else {
                                    fab.x + 16.0
                                },
                                y: fab.y + 16.0,
                                width: 24.0,
                                height: 24.0,
                            };
                            f.paint_navigation_icon(
                                glyph,
                                NavigationIcon::Timer,
                                true,
                                f.theme().on_primary_container,
                            );
                            if expansion > 0.001 {
                                let label = Rect {
                                    x: if rtl {
                                        fab.x + fab.width - 144.0
                                    } else {
                                        fab.x + 48.0
                                    },
                                    width: 96.0,
                                    ..fab
                                };
                                f.within(fab, |f| {
                                    f.styled_text(
                                        label,
                                        "Add timer",
                                        f.theme().on_primary_container.opacity(expansion),
                                        TextStyle {
                                            size: 14.0,
                                            weight: 500,
                                            align: if rtl { 2 } else { 0 },
                                            ..TextStyle::default()
                                        },
                                    )
                                });
                            }
                            if r.clicked {
                                self.notice = "Add timer".to_owned();
                            }
                            f.reserve(40.0);
                        });
                        let destinations = [
                            NavigationDestination {
                                label: "Alarm",
                                icon: "",
                                enabled: true,
                                badge: None,
                            },
                            NavigationDestination {
                                label: "Clock",
                                icon: "",
                                enabled: true,
                                badge: None,
                            },
                            NavigationDestination {
                                label: "Timer",
                                icon: "",
                                enabled: true,
                                badge: None,
                            },
                            NavigationDestination {
                                label: "Stopwatch",
                                icon: "",
                                enabled: true,
                                badge: None,
                            },
                        ];
                        let icons = [
                            NavigationIcon::Alarm,
                            NavigationIcon::Clock,
                            NavigationIcon::Timer,
                            NavigationIcon::Stopwatch,
                        ];
                        f.expressive_navigation_rail_at_progress(
                            100,
                            &destinations,
                            &mut self.selected,
                            rtl,
                            &icons,
                            expansion,
                        );
                        if modal && f.button(94, "Open modal rail").clicked {
                            self.overlay.open = true;
                        }
                    });
                }
                if slug == "navigation-bar" {
                    f.label(
                        [
                            "Home — overview",
                            "Files — your documents",
                            "Settings — preferences",
                        ][self.selected.min(2)],
                    );
                }
                if slug == "navigation-rail" {
                    let viewport = f.viewport();
                    let rtl = self.mask & 4 != 0;
                    let width = viewport.width.min(240.0);
                    let bounds = Rect {
                        x: if rtl {
                            viewport.x + viewport.width - width
                        } else {
                            viewport.x
                        },
                        y: viewport.y,
                        width,
                        height: viewport.height,
                    };
                    self.overlay.kind = if rtl {
                        OverlayKind::SideSheet
                    } else {
                        OverlayKind::LeftSideSheet
                    };
                    self.overlay.style = OverlayStyle::NavigationRail;
                    self.overlay.initial_focus = Some(300);
                    let destinations = [
                        NavigationDestination {
                            label: "Alarm",
                            icon: "",
                            enabled: true,
                            badge: None,
                        },
                        NavigationDestination {
                            label: "Clock",
                            icon: "",
                            enabled: true,
                            badge: None,
                        },
                        NavigationDestination {
                            label: "Timer",
                            icon: "",
                            enabled: true,
                            badge: None,
                        },
                        NavigationDestination {
                            label: "Stopwatch",
                            icon: "",
                            enabled: true,
                            badge: None,
                        },
                    ];
                    let icons = [
                        NavigationIcon::Alarm,
                        NavigationIcon::Clock,
                        NavigationIcon::Timer,
                        NavigationIcon::Stopwatch,
                    ];
                    f.overlay(500, &mut self.overlay, bounds, true, |f, open| {
                        f.with_gap(0.0, |f| {
                            f.reserve(44.0);
                            let slot = f.reserve(48.0);
                            let menu = Rect {
                                x: if rtl {
                                    slot.x + slot.width - 64.0
                                } else {
                                    slot.x + 16.0
                                },
                                y: slot.y,
                                width: 48.0,
                                height: 48.0,
                            };
                            let r = f.target(300, menu, "Close navigation", true, Role::Button);
                            f.state_layer(r, 24.0, f.theme().text);
                            f.paint_navigation_icon(
                                menu.inset(12.0),
                                NavigationIcon::Menu,
                                false,
                                f.theme().text,
                            );
                            if r.clicked {
                                *open = false;
                            }
                            f.reserve(8.0);
                            let slot = f.reserve(56.0);
                            let width = 152.0_f32.min((slot.width - 40.0).max(0.0));
                            let fab = Rect {
                                x: if rtl {
                                    slot.x + slot.width - 20.0 - width
                                } else {
                                    slot.x + 20.0
                                },
                                y: slot.y,
                                width,
                                height: 56.0,
                            };
                            let r = f.target(305, fab, "Add timer", true, Role::Button);
                            f.paint_rect(fab, 16.0, f.theme().primary_container);
                            f.state_layer(r, 16.0, f.theme().on_primary_container);
                            let icon = Rect {
                                x: if rtl {
                                    fab.x + fab.width - 40.0
                                } else {
                                    fab.x + 16.0
                                },
                                y: fab.y + 16.0,
                                width: 24.0,
                                height: 24.0,
                            };
                            f.paint_navigation_icon(
                                icon,
                                NavigationIcon::Timer,
                                true,
                                f.theme().on_primary_container,
                            );
                            f.styled_text(
                                Rect {
                                    x: fab.x + if rtl { 8.0 } else { 48.0 },
                                    width: (fab.width - 56.0).max(0.0),
                                    ..fab
                                },
                                "Add timer",
                                f.theme().on_primary_container,
                                TextStyle {
                                    size: 14.0,
                                    weight: 500,
                                    align: if rtl { 2 } else { 0 },
                                    ..TextStyle::default()
                                },
                            );
                            if r.clicked {
                                self.notice = "Add timer".to_owned();
                            }
                            f.reserve(40.0);
                            f.expressive_navigation_rail_at_progress(
                                301,
                                &destinations,
                                &mut self.selected,
                                rtl,
                                &icons,
                                1.0,
                            );
                        });
                    });
                }
            }
            "menu" | "popover" | "context-menu-integration" => {
                let mut trigger = None;
                f.without_input(self.overlay.open, |f| {
                    trigger = Some(f.button(
                        100,
                        if slug == "context-menu-integration" {
                            "Context actions (or right click)"
                        } else {
                            "Open anchored surface"
                        },
                    ));
                });
                let r = trigger.unwrap();
                self.anchor = r.bounds;
                if r.clicked || f.pressed_key(Key::Menu) {
                    self.overlay.open = true;
                    self.selected = 0;
                    self.overlay.initial_focus = Some(600 + self.selected as u64);
                }
                f.label(&self.notice);
                self.menu(f, &["Open", "Rename", "Remove"], slug == "popover", None);
            }
            "card" => {
                f.switch(90,"Dragged state",&mut self.touched,true);
                let mut checked=self.mask & 1 != 0;
                if f.switch(91,"Checked state",&mut checked,true).changed {self.mask ^= 1;}
                let mut rtl=self.mask & 2 != 0;
                if f.switch(92,"Right-to-left layout",&mut rtl,true).changed {self.mask ^= 2;}
                let mut disabled=self.mask & 4 != 0;
                if f.switch(93,"Disable example cards",&mut disabled,true).changed {self.mask ^= 4;}
                f.label("Android XML: theme-aware card colors");
                self.android_linear_example.rtl=rtl;
                self.android_linear_example.checked.insert("theme_card".into(),checked);
                self.android_linear_example.view_dragged.insert("theme_card".into(),self.touched);
                let xml_bounds=f.reserve(72.0);
                f.with_enabled(!disabled,|f| {if android_material_card_theme::LAYOUT.render(f,xml_bounds,&mut self.android_linear_example).is_some() {self.notice="Theme-aware XML card activated".into();}});
                f.label("Android XML: stateful card colors");
                self.android_linear_example.rtl=rtl;
                self.android_linear_example.checked.insert("selector_card".into(),checked);
                self.android_linear_example.view_dragged.insert("selector_card".into(),self.touched);
                let xml_bounds=f.reserve(72.0);
                f.with_enabled(!disabled,|f| {if android_material_card_selector::LAYOUT.render(f,xml_bounds,&mut self.android_linear_example).is_some() {self.notice="Stateful XML card activated".into();}});
                f.label("Android XML: custom card elevation and appearance");
                self.android_linear_example.rtl=rtl;
                self.android_linear_example.checked.insert("example_card".into(),checked);
                let xml_bounds=f.reserve(72.0);
                if android_material_card::LAYOUT.render(f,xml_bounds,&mut self.android_linear_example).is_some() {self.notice="XML card activated".into();}
                f.label("Android XML: compat shadow padding");
                self.android_linear_example.checked.insert("compat_card".into(),checked);
                let xml_bounds=f.reserve(112.0);
                if android_material_card_compat::LAYOUT.render(f,xml_bounds,&mut self.android_linear_example).is_some() {self.notice="Compat card activated".into();}
                for (i, (label, variant)) in [
                    ("Filled card", CardVariant::Filled),
                    ("Outlined card", CardVariant::Outlined),
                    ("Elevated card", CardVariant::Elevated),
                ]
                .iter()
                .enumerate()
                {
                    let b = f.reserve(96.0);
                    f.card_with_state(
                        200 + i as u64,
                        b,
                        label,
                        CardOptions {
                            variant: *variant,
                            enabled: !disabled,
                            ..CardOptions::default()
                        },
                        CardVisualState {dragged:self.touched,checked,rtl,..CardVisualState::default()},
                        |f| {
                            f.label("Card details");
                            if f.button(100 + i as u64, *label).clicked {
                                self.selected = i;
                            }
                        },
                    );
                }
                f.label(format!("Selected card: {}", self.selected + 1));
                let b = f.reserve(88.0);
                if f.card(
                    210,
                    b,
                    "Actionable card",
                    CardOptions {
                        interactive: true,
                        ..CardOptions::default()
                    },
                    |f| {
                        f.label("Actionable card");
                    },
                )
                .is_some_and(|r| r.clicked)
                {
                    self.notice = "Card activated".into();
                }
                let b = f.reserve(88.0);
                f.card(
                    211,
                    b,
                    "Disabled card",
                    CardOptions {
                        interactive: true,
                        enabled: false,
                        ..CardOptions::default()
                    },
                    |f| {
                        f.label("Disabled card");
                    },
                );
                f.label(&self.notice);
            }
            "dialog" | "bottom-sheet" | "side-sheet" | "overlay-and-scrim" => self.modal(f, slug),
            "carousel" => {
                let image = self.media_image.get_or_insert_with(sample_media_image);
                let items = [
                    CarouselItem {
                        label: "Featured one",
                        image: Some(image),
                        enabled: true,
                    },
                    CarouselItem {
                        label: "Featured two",
                        image: None,
                        enabled: true,
                    },
                    CarouselItem {
                        label: "Featured three",
                        image: Some(image),
                        enabled: true,
                    },
                    CarouselItem {
                        label: "Featured four",
                        image: None,
                        enabled: true,
                    },
                    CarouselItem {
                        label: "Featured five",
                        image: Some(image),
                        enabled: true,
                    },
                ];
                if self.open {
                    f.label("All featured items");
                    for (i, item) in items.iter().enumerate() {
                        if f.button(150 + i as u64, item.label).clicked {
                            self.notice = format!("Selected: {}", item.label);
                        }
                    }
                    if f.button(140, "Back to carousel").clicked {
                        self.open = false;
                    }
                } else {
                    f.label("Drag, use arrows, or show all");
                    let mut full = self.mask & 1 != 0;
                    let mut rtl = self.mask & 2 != 0;
                    let mut hero = self.mask & 4 != 0;
                    let mut browse = self.mask & 8 != 0;
                    if f.switch(90, "Full-screen layout", &mut full, true).changed {
                        self.mask = (self.mask & !1) | u32::from(full);
                        if full { self.mask &= !12; }
                        self.carousel.go_to(0);
                    }
                    if f.switch(91, "Right-to-left layout", &mut rtl, true).changed {
                        self.mask = (self.mask & !2) | (u32::from(rtl) << 1);
                    }
                    if f.switch(92, "Centered hero layout", &mut hero, true).changed {
                        self.mask = (self.mask & !4) | (u32::from(hero) << 2);
                        if hero { self.mask &= !9; }
                        self.carousel.go_to(0);
                    }
                    if f.switch(93, "Multi-browse layout", &mut browse, true).changed {
                        self.mask = (self.mask & !8) | (u32::from(browse) << 3);
                        if browse { self.mask &= !5; }
                        self.carousel.go_to(0);
                    }
                    let b = f.reserve(160.0);
                    let options = CarouselOptions {
                        layout: if browse {
                            CarouselLayout::MultiBrowse
                        } else if hero {
                            CarouselLayout::CenteredHero
                        } else if full {
                            CarouselLayout::FullScreen
                        } else {
                            CarouselLayout::Uncontained
                        },
                        rtl,
                        ..CarouselOptions::default()
                    };
                    if let Some(i) = f.carousel_named(130, b, &items, &mut self.carousel, options, "Featured items") {
                        self.notice = format!("Selected: {}", items[i].label);
                    }
                    f.row(48.0, 2, |f, i| {
                        if f.button_with(
                            100 + i as u64,
                            if i == 0 { "Previous" } else { "Next" },
                            Options {
                                enabled: if i == 0 {
                                    self.carousel.can_previous()
                                } else {
                                    self.carousel.can_next()
                                },
                                ..Options::default()
                            },
                        )
                        .clicked
                        {
                            if i == 0 {
                                self.carousel.previous()
                            } else {
                                self.carousel.next()
                            }
                        }
                    });
                    f.label(format!("Position: {}", self.carousel.index() + 1));
                    if f.button(140, "Show all").clicked {
                        self.open = true;
                    }
                }
                f.label(&self.notice);
            }
            "expansion-panel-accordion" | "tree" => {
                let tree = slug == "tree";
                let count = if tree { 2 } else { 3 };
                f.with_gap(0.0, |f| {
                    let mut previous_amount = 0.0_f32;
                    for i in 0..count {
                        let label = ["Section one", "Section two", "Section three"][i];
                        let open = self.mask & (1 << i) != 0;
                        let amount = if tree {
                            f32::from(open)
                        } else {
                            f.transition_css(
                                100 + i as u64,
                                21,
                                f32::from(open),
                                0.0,
                                0.225,
                                bhoo::timing::Easing::Cubic([0.4, 0.0, 0.2, 1.0]),
                            )
                        };
                        let header_height = if tree { 48.0 } else { 48.0 + 16.0 * amount };
                        let body_height = if tree { 96.0 } else { 128.0 };
                        if !tree && i > 0 {
                            // Adjacent positive CSS margins collapse to their maximum.
                            f.reserve(16.0 * previous_amount.max(amount));
                        }
                        previous_amount = amount;
                        let b = f.reserve(header_height + body_height * amount);
                        let panel = b;
                        let t = f.theme();
                        if !tree {
                            expansion_panel_shadow(
                                f,
                                panel,
                                open || i == 0,
                                open || i + 1 == count,
                            );
                            f.paint_rect(panel, 12.0, t.background);
                            if !open && i > 0 {
                                f.paint_rect(
                                    Rect {
                                        height: panel.height / 2.0,
                                        ..panel
                                    },
                                    0.0,
                                    t.background,
                                );
                            }
                            if !open && i + 1 < count {
                                f.paint_rect(
                                    Rect {
                                        y: panel.y + panel.height / 2.0,
                                        height: panel.height / 2.0,
                                        ..panel
                                    },
                                    0.0,
                                    t.background,
                                );
                            }
                        }
                        let header = Rect {
                            height: header_height,
                            ..b
                        };
                        let r = f.target(100 + i as u64, header, label, true, Role::Button);
                        // Angular suppresses the hover/focus background on an expanded header.
                        if tree {
                            f.state_layer(r, 0.0, t.text);
                        } else if !open && r.enabled {
                            let alpha = if f.keyboard_focused(r) {
                                0.12
                            } else if r.hovered {
                                0.08
                            } else {
                                0.0
                            };
                            if alpha > 0.0 {
                                let top = Rect {
                                    height: header.height / 2.0,
                                    ..header
                                };
                                let bottom = Rect {
                                    y: top.y + top.height,
                                    height: header.height - top.height,
                                    ..header
                                };
                                f.within(top, |f| {
                                    f.paint_rect(
                                        header,
                                        if i == 0 { 12.0 } else { 0.0 },
                                        t.text.opacity(alpha),
                                    )
                                });
                                f.within(bottom, |f| {
                                    f.paint_rect(
                                        header,
                                        if i + 1 == count { 12.0 } else { 0.0 },
                                        t.text.opacity(alpha),
                                    )
                                });
                            }
                        }
                        if r.clicked {
                            self.mask ^= 1 << i;
                        }
                        f.styled_text(
                            header,
                            label,
                            t.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
                            TextStyle {
                                size: 16.0,
                                weight: if tree { 400 } else { 500 },
                                padding: if tree { 40.0 } else { 24.0 },
                                ..TextStyle::default()
                            },
                        );
                        if tree {
                            f.styled_text(
                                Rect {
                                    width: 40.0,
                                    ..header
                                },
                                if open { "⌄" } else { "›" },
                                t.on_surface_variant,
                                TextStyle {
                                    size: 24.0,
                                    align: 1,
                                    ..TextStyle::default()
                                },
                            );
                        } else {
                            let (sin, cos) = (amount * std::f32::consts::PI).sin_cos();
                            let c = [b.x + b.width - 32.0, b.y + header_height / 2.0];
                            let point = |x: f32, y: f32| {
                                [c[0] + x * cos - y * sin, c[1] + x * sin + y * cos]
                            };
                            f.paint_line(
                                point(-4.0, -2.0),
                                point(0.0, 2.0),
                                2.0,
                                t.on_surface_variant,
                            );
                            f.paint_line(
                                point(0.0, 2.0),
                                point(4.0, -2.0),
                                2.0,
                                t.on_surface_variant,
                            );
                        }
                        if amount > 0.001 {
                            let body = Rect {
                                x: b.x + if tree { 40.0 } else { 24.0 },
                                y: b.y + header_height,
                                width: (b.width - if tree { 40.0 } else { 48.0 }).max(0.0),
                                height: body_height * amount,
                            };
                            f.within(body, |f| {
                                for (j, label) in ["Child one", "Child two"].iter().enumerate() {
                                    f.styled_text(
                                        Rect {
                                            y: body.y
                                                + if tree { 0.0 } else { 8.0 }
                                                + j as f32 * 48.0,
                                            height: 48.0,
                                            ..body
                                        },
                                        label,
                                        t.text,
                                        TextStyle {
                                            padding: if tree { 0.0 } else { 16.0 },
                                            letter_spacing: if tree { 0.0 } else { 0.5 },
                                            ..TextStyle::default()
                                        },
                                    );
                                }
                            });
                        }
                    }
                });
            }
            "scaffold" => {
                f.label("SpectreOS workspace");
                f.tabs(
                    100,
                    &["Overview", "Activity", "Settings"],
                    &mut self.selected,
                    true,
                );
                let b = f.reserve(160.0);
                f.paint_rect(b, 12.0, f.theme().secondary);
                f.paint_text(
                    b,
                    [
                        "Overview content",
                        "Recent activity",
                        "Application settings",
                    ][self.selected],
                    f.theme().text,
                );
                if filled(f, 110, "+  Create").clicked {
                    self.step += 1;
                }
                f.label(format!("Created items: {}", self.step));
            }
            "expressive-loading-indicator" => {
                if f.button(
                    100,
                    if self.open {
                        "Stop loading"
                    } else {
                        "Start loading"
                    },
                )
                .clicked
                {
                    self.open = !self.open;
                    for state in &mut self.loading {
                        state.reset();
                    }
                }
                f.label("Indeterminate / contained");
                if self.open {
                    f.row(72.0, 2, |f, i| {
                        f.loading_indicator(
                            110 + i as u64,
                            "Loading",
                            &mut self.loading[i],
                            None,
                            i == 1,
                        )
                    });
                } else {
                    f.label("Ready");
                }
                f.label("Determinate / circle to soft burst");
                f.slider(120, "Progress", &mut self.value, 0.0..=100.0, 1.0, true);
                f.loading_indicator(
                    121,
                    "Progress",
                    &mut self.loading[2],
                    Some(self.value / 100.0),
                    true,
                );
                f.label("Seven upstream shapes • morphs every 650 ms");
            }
            "snackbar" => {
                let now = f.time();
                if filled(f, 100, "Archive message").clicked {
                    self.snackbar_sequence = self.snackbar_sequence.wrapping_add(1);
                    self.snackbar_host.enqueue(bhoo::snackbar::Notice {
                        id: self.snackbar_sequence,
                        message: if self.snackbar_sequence % 2 == 0 {
                            "Message archived in the project folder. You can undo this action.".into()
                        } else {
                            "Message archived".into()
                        },
                        action: Some(if self.snackbar_sequence % 3 == 0 {
                            "Restore message".into()
                        } else {
                            "Undo".into()
                        }),
                        dismiss_label: (self.snackbar_sequence % 4 == 0 || self.snackbar_sequence % 5 == 0).then(|| "Dismiss".into()),
                        duration: bhoo::snackbar::Duration::Seconds(6.15),
                    }, now);
                    self.notice = "Message archived".into();
                }
                f.label(&self.notice);
                let viewport = f.viewport();
                self.snackbar_presentation.action_on_new_line = self.snackbar_host.current()
                    .is_some_and(|notice| notice.action.as_deref() == Some("Restore message"));
                self.snackbar_presentation.right_to_left = self.snackbar_host.current()
                    .is_some_and(|notice| notice.id % 5 == 0);
                // Preserve the comparison fixture's first action target ID (110).
                f.snackbar_host(2104162448473211, viewport, &mut self.snackbar_host, &mut self.snackbar_presentation);
                while let Some(result) = self.snackbar_host.take_result() {
                    if result.outcome == bhoo::snackbar::Outcome::Action {
                        self.notice = "Archive undone".into();
                    }
                }
            }
            "banner" | "notification-surface" => {
                if f.button(100, "Show notice").clicked {
                    self.open = true;
                }
                let msg = if slug == "banner" {
                    "Network is unavailable. Your work is saved locally."
                } else {
                    "Files • Download complete. Your work is saved locally."
                };
                if let Some(i) = f.banner(130, self.open, msg, &["Dismiss", "View"]) {
                    self.notice = if i == 0 { "Dismissed" } else { "Opened" }.into();
                    self.open = false;
                }
                f.label(&self.notice);
            }
            "data-table" | "sort-header" => self.table(f),
            "paginator" => {
                let mut page_size = 10usize;
                f.paginator_with_page_size(100, &mut self.page, &mut page_size, &[10], 95, true);
                for i in 0..3 {
                    f.label(format!("Result {}", self.page * 10 + i + 1));
                }
            }
            "stepper" => {
                if self.text.trim().is_empty() {
                    self.mask = 0;
                    self.step = 0;
                }
                let unlocked = self.mask.trailing_ones().min(2) as usize;
                let root = f.available();
                let header = f.reserve(72.0);
                let labels = ["Your name", "Profile", "Done"];
                let label_style = TextStyle {
                    size: 14.0,
                    weight: 500,
                    ..TextStyle::default()
                };
                let natural = labels.map(|label| f.text_width(label, label_style).max(50.0) + 80.0);
                let total = natural.iter().sum::<f32>();
                let shrink = (total - header.width).max(0.0);
                let content = total - 144.0;
                let extra = (header.width - total).max(0.0) / 2.0;
                let mut x = header.x;
                for i in 0..3 {
                    let width = natural[i] - shrink * (natural[i] - 48.0) / content.max(1.0);
                    let b = Rect { x, width, ..header };
                    let r = f.target(100 + i as u64, b, labels[i], i <= unlocked, Role::Button);
                    if r.clicked {
                        self.step = i;
                    }
                    let t = f.theme();
                    f.state_layer(r, 0.0, t.text);
                    let circle = Rect {
                        x: x + 24.0,
                        y: b.y + 24.0,
                        width: 24.0,
                        height: 24.0,
                    };
                    f.paint_rect(
                        circle,
                        12.0,
                        if self.step == i {
                            t.primary
                        } else {
                            t.on_surface_variant
                        },
                    );
                    if self.mask & (1 << i) != 0 && self.step != i {
                        f.paint_line(
                            [circle.x + 6.0, circle.y + 12.0],
                            [circle.x + 10.0, circle.y + 16.0],
                            1.5,
                            t.on_primary,
                        );
                        f.paint_line(
                            [circle.x + 10.0, circle.y + 16.0],
                            [circle.x + 18.0, circle.y + 8.0],
                            1.5,
                            t.on_primary,
                        );
                    } else {
                        f.styled_text(
                            circle,
                            ["1", "2", "3"][i],
                            t.on_primary,
                            TextStyle {
                                align: 1,
                                ..TextStyle::default()
                            },
                        );
                    }
                    f.styled_text(
                        Rect {
                            x: x + 56.0,
                            width: (width - 80.0).max(50.0),
                            ..b
                        },
                        labels[i],
                        if self.step == i {
                            t.text
                        } else {
                            t.on_surface_variant
                        },
                        label_style,
                    );
                    if i < 2 {
                        f.paint_line(
                            [x + width - 16.0, b.y + 36.0],
                            [x + width + extra + 16.0, b.y + 36.0],
                            1.0,
                            t.outline,
                        );
                    }
                    x += width + if i < 2 { extra } else { 0.0 };
                }
                let content = Rect {
                    x: root.x + 24.0,
                    y: root.y + 72.0,
                    width: (root.width - 48.0).max(0.0),
                    height: (root.height - 72.0).max(0.0),
                };
                f.within(content, |f| {
                    if self.step == 0 {
                        f.text_field(900, "Name", 56.0, !self.text.is_empty(), false, true);
                    } else {
                        f.label(if self.step == 1 {
                            "Choose your profile"
                        } else {
                            "Ready"
                        });
                    }
                    let b = f.reserve(40.0);
                    let mut x = b.x;
                    if self.step > 0 {
                        if f.action_at(
                            110,
                            Rect {
                                x,
                                width: 64.0,
                                ..b
                            },
                            "Back",
                            Options::default(),
                        )
                        .clicked
                        {
                            self.step -= 1;
                        }
                        x += 76.0;
                    }
                    if self.step < 2
                        && f.action_at(
                            111,
                            Rect {
                                x,
                                width: 64.0,
                                ..b
                            },
                            "Next",
                            Options {
                                enabled: self.step != 0 || !self.text.trim().is_empty(),
                                ..Options::default()
                            },
                        )
                        .clicked
                    {
                        self.mask |= 1 << self.step;
                        self.step += 1;
                    }
                });
            }
            "grid-list-image-list" => {
                let items = [
                    GridListItem { label: "Files", subtitle: None, icon: Some(MaterialIcon::Description), image: None },
                    GridListItem { label: "Browser", subtitle: None, icon: Some(MaterialIcon::Search), image: None },
                    GridListItem { label: "Music", subtitle: None, icon: Some(MaterialIcon::Star), image: None },
                    GridListItem { label: "Settings", subtitle: None, icon: Some(MaterialIcon::Settings), image: None },
                ];
                if let Some(i) = f.grid_list(
                    100,
                    &items,
                    Some(self.selected),
                    GridListOptions { columns: 2, tile_height: 112.0, ..GridListOptions::default() },
                ) {
                    self.selected = i;
                }
            }
            "application-tile" => {
                for row in 0..2 {
                    f.row(120.0, 3, |f, col| {
                        let i = row * 3 + col;
                        let b = f.reserve(108.0);
                        if action(
                            f,
                            100 + i as u64,
                            b,
                            ["Files", "Settings", "Terminal", "Notes", "Music", "Photos"][i],
                            self.selected == i,
                        )
                        .clicked
                        {
                            self.selected = i;
                        }
                    });
                }
                f.label(format!("Selected: {}", self.selected + 1));
            }
            "hover-wave" => {
                f.label("Hover or focus the button.");
                if f.hover_wave_button(100, "Hover wave", true).clicked {
                    self.page += 1;
                }
                f.label(format!("Activations: {}", self.page));
            }
            "media" => {
                f.label("Image media");
                let image = self.media_image.get_or_insert_with(sample_media_image);
                f.row(100.0, 2, |f, i| {
                    let cell = f.reserve(96.0);
                    let b = Rect {
                        x: cell.x,
                        y: cell.y,
                        width: if i == 0 {
                            64.0
                        } else {
                            128.0_f32.min(cell.width)
                        },
                        height: 64.0,
                    };
                    f.paint_image(b, image, ImageFit::Cover, if i == 0 { 32.0 } else { 8.0 });
                    f.styled_text(
                        Rect {
                            y: cell.y + 68.0,
                            height: 20.0,
                            ..cell
                        },
                        if i == 0 { "Circle" } else { "Rounded" },
                        f.theme().text,
                        TextStyle {
                            size: 12.0,
                            ..TextStyle::default()
                        },
                    );
                });
                let b = f.reserve(130.0);
                f.paint_image(
                    Rect {
                        width: b.width.min(300.0),
                        height: 120.0,
                        ..b
                    },
                    image,
                    ImageFit::Cover,
                    16.0,
                );
                f.label("Caller-supplied RGBA image · cover fit");
            }
            "shape-rotation" => {
                f.label("Rotating shapes");
                let index = bhoo::beercss_shapes::SHAPES
                    .iter()
                    .position(|s| s.name == "arrow")
                    .expect("arrow shape");
                f.row(122.0, 3, |f, i| {
                    let cell = f.reserve(122.0);
                    let b = Rect {
                        x: cell.x + (cell.width - 56.0) / 2.0,
                        y: cell.y + 12.0,
                        width: 56.0,
                        height: 56.0,
                    };
                    f.rotating_shape(
                        b,
                        index,
                        f.theme().primary,
                        [24.0, 12.0, 6.0][i],
                        &mut self.shape_rotation[i],
                    );
                    f.styled_text(
                        Rect {
                            y: cell.y + 76.0,
                            height: 24.0,
                            ..cell
                        },
                        ["Slow · 24s", "Normal · 12s", "Fast · 6s"][i],
                        f.theme().text,
                        TextStyle {
                            size: 12.0,
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
                f.label("Rotation pauses when hidden or reduced motion is enabled.");
            }
            "expressive-shapes" => {
                f.label("Expressive shapes");
                f.grid(bhoo::beercss_shapes::SHAPES.len(), 6, 74.0, 4.0, |f, i| {
                    let cell = f.reserve(70.0);
                    let size = cell.width.min(44.0);
                    let b = Rect {
                        x: cell.x + (cell.width - size) / 2.0,
                        y: cell.y,
                        width: size,
                        height: size,
                    };
                    match bhoo::beercss_shapes::SHAPES[i].name {
                        "wavy-circle" => {
                            f.animated_wavy_shape(
                                b,
                                i,
                                f.theme().primary,
                                &mut self.wavy_shapes[0],
                            );
                        }
                        "wavy" => {
                            f.animated_wavy_shape(
                                b,
                                i,
                                f.theme().primary,
                                &mut self.wavy_shapes[1],
                            );
                        }
                        _ => {
                            f.paint_shape(b, i, f.theme().primary);
                        }
                    }
                    f.styled_text(
                        Rect {
                            y: cell.y + 46.0,
                            height: 18.0,
                            ..cell
                        },
                        bhoo::beercss_shapes::SHAPES[i].name,
                        f.theme().text,
                        TextStyle {
                            size: 9.0,
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
            }
            "page-transition" => {
                f.label("Page navigation");
                f.row(48.0, 2, |f, i| {
                    if f.button(100 + i as u64, if i == 0 { "Page one" } else { "Page two" })
                        .clicked
                    {
                        self.selected = i;
                    }
                });
                let bounds = f.reserve(140.0);
                for (i, name) in ["Page one", "Page two"].iter().enumerate() {
                    f.page(
                        120 + i as u64,
                        self.selected == i,
                        bounds,
                        if i == 0 {
                            PageDirection::Left
                        } else {
                            PageDirection::Right
                        },
                        |f, b| {
                            f.paint_rect(b, 12.0, f.theme().secondary);
                            f.styled_text(
                                b,
                                name,
                                f.theme().on_secondary,
                                TextStyle {
                                    size: 22.0,
                                    align: 1,
                                    ..TextStyle::default()
                                },
                            );
                        },
                    );
                }
            }
            "transition-grow" => {
                if f.button(
                    100,
                    if self.open {
                        "Hide content"
                    } else {
                        "Show content"
                    },
                )
                .clicked
                {
                    self.open = !self.open;
                }
                let b = f.reserve(100.0);
                f.grow(120, self.open, b, |f| {
                    f.paint_rect(b, 8.0, f.theme().secondary);
                    f.styled_text(
                        b,
                        "Growing content",
                        f.theme().on_secondary,
                        TextStyle {
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
            }
            "transition-slide" => {
                if f.button(
                    100,
                    if self.open {
                        "Hide content"
                    } else {
                        "Show content"
                    },
                )
                .clicked
                {
                    self.open = !self.open;
                }
                let b = f.reserve(100.0);
                f.slide_up(120, self.open, b, |f, posed| {
                    f.paint_rect(posed, 8.0, f.theme().secondary);
                    f.styled_text(
                        posed,
                        "Sliding content",
                        f.theme().on_secondary,
                        TextStyle {
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
            }
            "transition-collapse" => {
                if f.button(100, if self.open { "Collapse" } else { "Expand" })
                    .clicked
                {
                    self.open = !self.open;
                }
                let width = f.available().width;
                f.collapse(120, self.open, 96.0, |f| {
                    let b = f.reserve(96.0);
                    f.paint_rect(Rect { width, ..b }, 8.0, f.theme().secondary);
                    f.styled_text(
                        b,
                        "Collapsible content",
                        f.theme().on_secondary,
                        TextStyle {
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
                f.label("Content below the transition");
            }
            "transition-zoom" => {
                if f.button(
                    100,
                    if self.open {
                        "Hide content"
                    } else {
                        "Show content"
                    },
                )
                .clicked
                {
                    self.open = !self.open;
                }
                let b = f.reserve(100.0);
                f.zoom(120, self.open, b, |f| {
                    f.paint_rect(b, 8.0, f.theme().secondary);
                    f.styled_text(
                        b,
                        "Zooming content",
                        f.theme().on_secondary,
                        TextStyle {
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
            }
            "transition-fade" => {
                if f.button(
                    100,
                    if self.open {
                        "Hide content"
                    } else {
                        "Show content"
                    },
                )
                .clicked
                {
                    self.open = !self.open;
                }
                let b = f.reserve(100.0);
                f.fade(120, self.open, b, |f| {
                    f.paint_rect(b, 8.0, f.theme().secondary);
                    f.styled_text(
                        b,
                        "Fading content",
                        f.theme().on_secondary,
                        TextStyle {
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
            }
            "layout-box" => {
                f.label("Styled content box");
                let t = f.theme();
                f.boxed(130.0, 12.0, 16.0, t.secondary, |f| {
                    f.label("Bhoo workspace");
                    f.label("A box with padding and rounded corners.");
                });
            }
            "layout-container" => {
                f.label("Centered maximum-width container");
                f.container(240.0, 110.0, |f| {
                    let b = f.reserve(100.0);
                    f.paint_rect(b, 8.0, f.theme().secondary);
                    f.styled_text(
                        b,
                        "Max width: 240 px",
                        f.theme().on_secondary,
                        TextStyle {
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
            }
            "layout-stack" => {
                f.label("Vertical stack");
                f.stack(3, 8.0, |f, i| {
                    let b = f.reserve(40.0);
                    f.paint_rect(b, 8.0, f.theme().secondary);
                    f.styled_text(
                        b,
                        &format!("Item {}", i + 1),
                        f.theme().on_secondary,
                        TextStyle {
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
            }
            "layout-grid" => {
                f.label("Equal-width grid");
                f.grid(6, 3, 80.0, 8.0, |f, i| {
                    let b = f.reserve(72.0);
                    f.paint_rect(b, 8.0, f.theme().secondary);
                    f.styled_text(
                        b,
                        &format!("Item {}", i + 1),
                        f.theme().on_secondary,
                        TextStyle {
                            align: 1,
                            ..TextStyle::default()
                        },
                    );
                });
                f.label("Three columns, six items");
            }
            "link" => {
                f.label("Navigation links");
                if f.link(100, "Open documentation", true).clicked {
                    self.notice = "Documentation requested".into();
                }
                f.label(&self.notice);
            }
            "avatar-group" => {
                let mut limited=self.mask&1!=0;if f.switch(91,"Maximum 3 slots",&mut limited,true).changed {self.mask^=1;}
                let mut larger_total=self.mask&2!=0;if f.switch(92,"Total 10 contributors",&mut larger_total,true).changed {self.mask^=2;}
                let mut tighter=self.mask&4!=0;if f.switch(93,"Tighter overlap",&mut tighter,true).changed {self.mask^=4;}
                let mut rtl=self.mask&8!=0;if f.switch(94,"Right-to-left layout",&mut rtl,true).changed {self.mask^=8;}
                let mut square=self.mask&16!=0;if f.switch(95,"Square avatars",&mut square,true).changed {self.mask^=16;}
                f.label("Project contributors");
                let image=self.avatar_sample.get_or_insert_with(sample_media_image).clone();
                let items=[AvatarContent::Image(&image),AvatarContent::Initials("BH"),AvatarContent::Fallback,AvatarContent::Initials("DV"),AvatarContent::Initials("ER"),AvatarContent::Initials("FT")];
                let counts=f.avatar_group_with_options(&items,AvatarGroupOptions{max:if limited{3}else{5},total:larger_total.then_some(10),overlap:if tighter{16.0}else{8.0},shape:if square{AvatarShape::Square}else{AvatarShape::Circle},rtl,..AvatarGroupOptions::default()});
                f.label(format!("{} shown · {} more",counts.shown,counts.surplus));
            }
            "alert" => {
                f.label("Feedback messages");
                for (i, (message, severity)) in [
                    ("Changes saved successfully.", AlertSeverity::Success),
                    ("An update is available.", AlertSeverity::Info),
                    ("Storage is running low.", AlertSeverity::Warning),
                    ("The connection failed.", AlertSeverity::Error),
                ]
                .into_iter()
                .enumerate()
                {
                    if self.mask & (1 << i) == 0
                        && f.alert(100 + i as u64, message, severity).clicked
                    {
                        self.mask |= 1 << i;
                    }
                }
                if f.button(110, "Restore alerts").clicked {
                    self.mask = 0;
                }
            }
            "pagination" => {
                f.label("Choose a page");
                let mut page = self.page + 1;
                f.pagination(100, &mut page, 5, true);
                self.page = page - 1;
                f.label(format!("Page {page} of 5"));
                f.label("Large page count");
                f.pagination(200, &mut self.large_page, 50, true);
                f.label(format!("Page {} of 50", self.large_page));
            }
            "skeleton" => {
                f.label("Loading placeholders");
                let row = f.reserve(48.0);
                f.skeleton(
                    Rect {
                        width: 40.0,
                        height: 40.0,
                        ..row
                    },
                    20.0,
                    true,
                );
                f.skeleton(
                    Rect {
                        x: row.x + 52.0,
                        y: row.y + 5.0,
                        width: (row.width - 52.0).min(210.0),
                        height: 16.0,
                    },
                    4.0,
                    true,
                );
                f.skeleton(
                    Rect {
                        x: row.x + 52.0,
                        y: row.y + 27.0,
                        width: (row.width - 52.0).min(150.0),
                        height: 12.0,
                    },
                    4.0,
                    true,
                );
                let card = f.reserve(120.0);
                f.skeleton(
                    Rect {
                        width: card.width.min(260.0),
                        ..card
                    },
                    8.0,
                    true,
                );
                f.label("Circle, text and rounded placeholders");
            }
            "rating" => {
                f.label("Rate this experience");
                f.rating(100, &mut self.rating, 5, true);
                f.label(format!("Rating: {} of 5", self.rating));
                f.label("Choose a star, or use the arrow keys.");
            }
            "avatar" => {
                let mut rounded=self.mask&1!=0;
                if f.switch(91,"Rounded shape",&mut rounded,true).changed {if rounded{self.mask=(self.mask&!3)|1;}else{self.mask&=!1;}}
                let mut square=self.mask&2!=0;
                if f.switch(92,"Square shape",&mut square,true).changed {if square{self.mask=(self.mask&!3)|2;}else{self.mask&=!2;}}
                let mut failed=self.mask&4!=0;
                if f.switch(93,"Simulate image error",&mut failed,true).changed {self.mask^=4;}
                let shape=if square {AvatarShape::Square}else if rounded {AvatarShape::Rounded}else{AvatarShape::Circle};
                let image=self.avatar_sample.get_or_insert_with(|| {
                    let mut pixels=Vec::with_capacity(96*64*4);
                    for y in 0..64 {for x in 0..96 {
                        let color=if (x as i32-66).pow(2)+(y as i32-18).pow(2)<64 {[255,219,139,255]}
                            else if y>43+(x as i32-45).abs()/4 {[52,102,76,255]}
                            else {[150,204,198,255]};pixels.extend_from_slice(&color);
                    }}
                    Image::rgba(96,64,pixels).expect("valid local example image")
                }).clone();
                f.label("Image, initials and fallback icon");
                f.row(92.0, 3, |f, i| {
                    let mut b = f.reserve(72.0);
                    b.width = b.width.min(72.0);
                    let content=match i {0 if !failed=>AvatarContent::Image(&image),0=>AvatarContent::Initials("SP"),1=>AvatarContent::Initials("BH"),_=>AvatarContent::Fallback};
                    f.avatar_at(b,shape,content);
                });
                f.label("An image error uses initials when available.");
            }
            "android-textview" => {
                f.label("Multiline width and longest line");
                let bounds=f.reserve(80.0);android_textview_separator_width::LAYOUT.render(f,bounds,&mut self.android_text_example);
                f.label("Accent, plain letter, CJK and emoji");
                let bounds=f.reserve(56.0);android_textview_grapheme::LAYOUT.render(f,bounds,&mut self.android_text_example);
                f.switch(71,"Right-to-left layout",&mut self.android_text_example.rtl,true);
                f.switch(70, "Selected text", &mut self.touched, true);
                self.android_text_example.view_selected.insert("compound_tint_active".into(),self.touched);
                self.android_text_example.view_selected.insert("compound_selector_active".into(),self.touched);
                self.android_text_example.view_selected.insert("compound_variable".into(),self.touched);
                self.android_text_example.view_selected.insert("compound_constant".into(),self.touched);
                f.label("Drawable selector intrinsic sizes");
                let bounds=f.reserve(112.0);
                android_textview_compound_sizes::LAYOUT.render(f,bounds,&mut self.android_text_example);
                f.label("Android XML compound drawable selector");
                let bounds=f.reserve(112.0);
                android_textview_compound_selector::LAYOUT.render(f,bounds,&mut self.android_text_example);
                f.label("Android XML compound drawable tint modes");
                let bounds=f.reserve(240.0);
                android_textview_tint_modes::LAYOUT.render(f,bounds,&mut self.android_text_example);
                f.label("Android XML drawable tint");
                let bounds=f.reserve(124.0);
                android_textview_compound_tint::LAYOUT.render(f,bounds,&mut self.android_text_example);
                f.label("Android XML compound drawables");
                let bounds=f.reserve(148.0);
                android_textview_compound::LAYOUT.render(f,bounds,&mut self.android_text_example);
                self.android_text_example.view_selected.insert("label".into(),self.touched);
                self.android_text_example.view_selected.insert("action".into(),self.touched);
                let bounds = f.reserve(180.0);
                android_textview::LAYOUT.render(f, bounds, &mut self.android_text_example);
                f.label("Android XML textAppearance");
                let bounds=f.reserve(168.0);
                android_textappearance::LAYOUT.render(f,bounds,&mut self.android_text_example);
            }
            "android-imagebutton" => {
                f.label("Tap the image button");
                android_imagebutton::install_drawables(&mut self.android_imagebutton_example);
                let bounds = f.reserve(100.0);
                if let Some(id) = android_imagebutton::LAYOUT.render(f, bounds, &mut self.android_imagebutton_example) {
                    self.touched = !self.touched;
                    self.android_imagebutton_example.image_sources.insert("picture".into(), if self.touched { "switcher_day" } else { "sample" }.into());
                    self.notice = format!("Pressed: {id}");
                }
                f.label(&self.notice);
            }
            "android-imageview" => {
                android_imageview_baseline::install_drawables(&mut self.android_image_example);
                f.switch(73,"Right-to-left baseline rows",&mut self.android_image_example.rtl,true);
                f.label("Image baseline alignment");
                let bounds=f.reserve(184.0);
                android_imageview_baseline::LAYOUT.render(f,bounds,&mut self.android_image_example);
                android_imageview_matrix::install_drawables(&mut self.android_image_example);
                f.switch(71,"Scale and translate matrix",&mut self.open,true);
                let mut reflected=self.mask&1!=0;
                if f.switch(72,"Reflect horizontally",&mut reflected,true).changed {self.mask^=1;}
                let scale=if self.open {1.4}else{1.0};
                let dx=if self.open {10.0}else{0.0};let dy=if self.open {8.0}else{0.0};
                if self.open || reflected {self.android_image_example.image_matrices.insert("matrix_example".into(),[if reflected {-scale}else{scale},scale,dx+if reflected {96.0*scale}else{0.0},dy]);} else {self.android_image_example.image_matrices.remove("matrix_example");}
                f.label("ImageView matrix");
                let bounds=f.reserve(100.0);
                android_imageview_matrix::LAYOUT.render(f,bounds,&mut self.android_image_example);
                android_imageview_crop_padding::install_drawables(&mut self.android_image_example);
                f.label("Image crop to padding");
                let bounds=f.reserve(256.0);
                android_imageview_crop_padding::LAYOUT.render(f,bounds,&mut self.android_image_example);
                android_imageview::install_drawables(&mut self.android_image_example);
                f.switch(70, "Selected image", &mut self.touched, true);
                self.android_image_example.view_selected.insert("interactive".into(),self.touched);
                let bounds = f.reserve(220.0);
                android_imageview::LAYOUT.render(f, bounds, &mut self.android_image_example);
            }
            "android-linear-layout" => {
                let mut baseline=self.mask&1!=0;
                if f.switch(71,"Show baseline alignment",&mut baseline,true).changed {self.mask^=1;}
                if baseline {let mut nested=self.mask&2!=0;if f.switch(72,"Nested child baseline",&mut nested,true).changed {self.mask^=2;}}
                f.label(if baseline {"Horizontal rows with mixed text sizes"} else {"Android LinearLayout clipping to padding"});
                f.switch(70, "Right-to-left layout", &mut self.android_linear_example.rtl, true);
                f.label("Multiline button baseline");
                let bounds=f.reserve(112.0);
                android_button_multiline_baseline::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if f.button(9901,"Change nested label").clicked {
                    if self.android_linear_example.text.contains_key("nested_wrap") {self.android_linear_example.text.remove("nested_wrap");}
                    else {self.android_linear_example.text.insert("nested_wrap".into(),"Longer label".into());}
                }
                f.label("Nested wrap-content row");
                let bounds=f.reserve(56.0);
                android_linear_wrap::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                f.label("Editable and text baselines");
                let bounds=f.reserve(216.0);
                android_linear_editable_baseline::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                f.label("Control and text baselines");
                let bounds=f.reserve(216.0);
                android_linear_compound_baseline::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                let bounds = f.reserve(240.0);
                let layout=if baseline && self.mask&2!=0 {&android_linear_nested_baseline::LAYOUT}else if baseline {&android_linear_baseline::LAYOUT}else{&android_linear_layout::LAYOUT};
                if let Some(id) = layout.render(f, bounds, &mut self.android_linear_example) { self.notice = format!("Pressed: {id}"); }
                f.label(&self.notice);
            }
            "android-checked-textview" => {
                f.switch(70, "Right-to-left layout", &mut self.android_checkbox_example.rtl, true);
                f.label("Custom check-mark tint modes");
                let bounds=f.reserve(288.0);
                if let Some(id)=android_checked_textview_tint_modes::LAYOUT.render(f,bounds,&mut self.android_checkbox_example) {
                    if let Some(checked)=self.android_checkbox_example.checked.get_mut(id) {*checked=!*checked;}
                }
                f.label("Custom check-mark tint opacity");
                let bounds=f.reserve(112.0);
                if let Some(id)=android_checked_textview_tint_alpha::LAYOUT.render(f,bounds,&mut self.android_checkbox_example) {
                    if let Some(checked)=self.android_checkbox_example.checked.get_mut(id) {*checked=!*checked;}
                }
                f.label("CheckedTextView compound drawables");
                let bounds=f.reserve(112.0);
                if let Some(id)=android_checked_textview_compound::LAYOUT.render(f,bounds,&mut self.android_checkbox_example) {
                    if let Some(checked)=self.android_checkbox_example.checked.get_mut(id) {*checked=!*checked;}
                }
                let bounds = f.reserve(168.0);
                if let Some(id) = android_checked_textview::LAYOUT.render(f, bounds, &mut self.android_checkbox_example) {
                    if let Some(checked) = self.android_checkbox_example.checked.get_mut(id) { *checked = !*checked; }
                }
                f.label("XML check-mark tint and gravity");
                let bounds=f.reserve(168.0);
                if let Some(id)=android_checked_textview_tints::LAYOUT.render(f,bounds,&mut self.android_checkbox_example) {
                    if let Some(checked)=self.android_checkbox_example.checked.get_mut(id) {*checked=!*checked;}
                }
                f.label("XML check-mark drawable selector");
                let bounds=f.reserve(168.0);
                if let Some(id)=android_checked_textview_drawable::LAYOUT.render(f,bounds,&mut self.android_checkbox_example) {
                    if let Some(checked)=self.android_checkbox_example.checked.get_mut(id) {*checked=!*checked;}
                }
            }
            "android-analogclock" => {
                f.label("Local time");
                let bounds=f.reserve(220.0);
                android_analogclock::LAYOUT.render(f,bounds,&mut self.android_clock_example);
            }
            "android-zoomcontrols" => {
                f.label("Tap or hold either zoom control");
                f.switch(70, "Disable zoom in", &mut self.touched, true);
                if f.switch(71, "Hide zoom controls", &mut self.open, true).changed {
                    self.android_frame_example.set_zoom_controls_visible("zoom_controls", !self.open, f.time());
                }
                f.switch(72, "Narrow container", &mut self.button_group_narrow, true);
                self.android_frame_example.zoom_controls_enabled.insert("zoom_controls".into(), [true,!self.touched]);
                let area=f.reserve(48.0);
                let bounds=Rect { width:if self.button_group_narrow { 120.0_f32.min(area.width) } else { area.width }, ..area };
                if android_zoomcontrols::LAYOUT.render(f,bounds,&mut self.android_frame_example).is_some() {
                    match self.android_frame_example.zoom_controls_actions.get("zoom_controls") {
                        Some(1) => self.step=self.step.saturating_add(1),
                        Some(-1) => self.step=self.step.saturating_sub(1),
                        _ => {}
                    }
                }
                f.label(format!("Zoom level: {}",self.step));
            }
            "android-zoombutton" => {
                f.label("Tap or hold +; move away before holding to cancel");
                f.switch(70, "Fast repeat (250 ms)", &mut self.touched, true);
                self.android_frame_example.zoom_speed_ms.insert("zoom_in".into(), if self.touched { 250 } else { 1000 });
                let bounds=f.reserve(64.0);
                if android_zoombutton::LAYOUT.render(f,bounds,&mut self.android_frame_example).is_some() { self.step=self.step.saturating_add(1); }
                f.label(format!("Value: {}",self.step));
            }
            "android-digitalclock" => {
                f.label("Device time");
                let bounds=f.reserve(64.0);
                android_digitalclock::LAYOUT.render(f,bounds,&mut self.android_clock_example);
            }
            "android-textclock" => {
                f.label("Android TextClock patterns");
                let bounds=f.reserve(216.0);
                android_textclock::LAYOUT.render(f,bounds,&mut self.android_clock_example);
            }
            "android-chronometer" => {
                let now=f.time();
                self.android_frame_example.chronometers.entry("up".into()).or_insert(bhoo::android_xml::ChronometerState { base:now, running:false });
                self.android_frame_example.chronometers.entry("down".into()).or_insert(bhoo::android_xml::ChronometerState { base:now+15.0, running:false });
                f.row(48.0,3,|f,i| {
                    if f.button(70+i as u64,["Start", "Stop", "Reset"][i]).clicked {
                        for (key,offset) in [("up",0.0),("down",15.0)] {
                            if let Some(clock)=self.android_frame_example.chronometers.get_mut(key) {
                                if i==2 { clock.base=now+offset; clock.running=false; self.android_frame_example.text.remove(key); }
                                else { clock.running=i==0; }
                            }
                        }
                    }
                });
                let bounds=f.reserve(96.0);
                android_chronometer::LAYOUT.render(f,bounds,&mut self.android_frame_example);
            }
            "android-imageswitcher" => {
                android_imageswitcher::install_drawables(&mut self.android_image_example);
                if f.button(70, "Change image").clicked {
                    self.step = (self.step + 1) % 2;
                    android_imageswitcher::LAYOUT.set_switcher_image(&mut self.android_image_example, ["switcher_day", "sample"][self.step]);
                }
                let bounds = f.reserve(180.0);
                android_imageswitcher::LAYOUT.render(f, bounds, &mut self.android_image_example);
            }
            "android-textswitcher" => {
                if f.button(70, "Change message").clicked {
                    self.step = (self.step + 1) % 3;
                    android_textswitcher::LAYOUT.set_switcher_text(&mut self.android_frame_example, ["Short message", "This longer message wraps onto additional lines as the text changes.", "Another short message"][self.step], true);
                }
                let bounds = f.reserve(120.0);
                android_textswitcher::LAYOUT.render(f, bounds, &mut self.android_frame_example);
            }
            "android-viewflipper" => {
                let running = self.android_frame_example.flipper_running.entry("flipper".into()).or_insert(true);
                f.switch(70, "Automatic flipping", running, true);
                let bounds = f.reserve(120.0);
                android_viewflipper::LAYOUT.render(f, bounds, &mut self.android_frame_example);
            }
            "android-viewswitcher" => {
                if f.button(70, "Show next view").clicked {
                    let index = self.android_frame_example.displayed_children.entry("switcher".into()).or_default();
                    *index = (*index + 1) % 2;
                }
                let bounds = f.reserve(120.0);
                android_viewswitcher::LAYOUT.render(f, bounds, &mut self.android_frame_example);
            }
            "android-multi-autocomplete" => {
                if self.mask & 1 == 0 {
                    self.android_edit_example.set_autocomplete_items("cities",["Amsterdam","Athens","Austin","Barcelona","Berlin","Boston","Cape Town","New York","Paris","San Diego","San Francisco","Santa Fe","Tokyo","Vienna"].into_iter().map(str::to_owned).collect());
                    self.android_edit_example.comma_tokenizers.insert("cities".into());self.mask|=1;
                }
                f.label("Type at least two letters, choose a city, and add another.");
                let bounds=f.reserve(64.0);android_multi_autocomplete::LAYOUT.render(f,bounds,&mut self.android_edit_example);
            }
            "android-expandable-list" => {
                use bhoo::android_expandable_list::{ExpandableGroup,ExpandableChild,ExpandableEvent};
                if self.mask&1==0 {android_expandable_list::install_drawables(&mut self.android_linear_example);let groups=[("Fruits",&["Apple","Banana","Orange"][..]),("Vegetables",&["Carrot","Cucumber","Spinach"][..]),("Drinks",&["Coffee","Tea","Water","Milk","Juice","Cocoa","Lemonade","Smoothie"][..])].into_iter().enumerate().map(|(i,(label,children))|ExpandableGroup{id:i as u64,label:label.into(),children:children.into_iter().enumerate().map(|(j,label)|ExpandableChild{id:j as u64,label:(*label).into(),selectable:true}).collect()}).collect();self.android_linear_example.expandable_lists.entry("expandable".into()).or_default().set_groups(groups).expect("unique sample IDs");self.android_linear_example.set_expandable_item_templates("expandable",Some(bhoo::android_xml::ListItemTemplate{row:&android_expandable_group_row::LAYOUT,text_id:"group_title"}),Some(bhoo::android_xml::ListItemTemplate{row:&android_expandable_child_row::LAYOUT,text_id:"child_title"})).expect("valid XML row templates");for (group,detail) in ["Fresh fruit","Fresh vegetables","Drink selection"].into_iter().enumerate() {let count=self.android_linear_example.expandable_lists["expandable"].groups()[group].children.len();for child in 0..count {let mut binding=bhoo::android_xml::AdapterItemBinding::default();binding.text.insert("child_detail".into(),format!("{detail} · item {}",child+1));self.android_linear_example.set_expandable_item_binding("expandable",bhoo::android_expandable_list::FlatPosition::Child{group,child},Some(binding)).expect("valid row binding");}}self.android_linear_example.set_list_fixed_views("expandable",vec![bhoo::android_xml::ListFixedView{view:&android_expandable_header::LAYOUT,selectable:false,label:"Categories"}],vec![bhoo::android_xml::ListFixedView{view:&android_expandable_footer::LAYOUT,selectable:true,label:"End of list"}]).expect("valid fixed views");self.android_linear_example.expandable_lists.get_mut("expandable").unwrap().set_max_expanded(Some(1));self.mask|=1;}
                f.label("Tap a group to switch its XML vector chevron indicator.");if f.button(9813,"Toggle Fruits interception").clicked {let ids=self.android_linear_example.expandable_intercepted_groups.entry("expandable".into()).or_default();if !ids.remove(&0){ids.insert(0);}self.notice=if ids.contains(&0){"Fruits interception on"}else{"Fruits interception off"}.into();}if f.button(9812,"Reverse groups").clicked {let list=self.android_linear_example.expandable_lists.get_mut("expandable").unwrap();let mut groups=list.groups().to_vec();groups.reverse();list.set_groups(groups).expect("unique sample IDs");}if f.button(9815,"Toggle right-to-left").clicked {self.android_linear_example.rtl=!self.android_linear_example.rtl;}if f.button(9814,"Expand Drinks with scroll").clicked {let group=self.android_linear_example.expandable_lists["expandable"].groups().iter().position(|g|g.label=="Drinks").unwrap();self.android_linear_example.expand_list_group("expandable",group,true);}let bounds=f.reserve(288.0);let action=android_expandable_list::LAYOUT.render(f,bounds,&mut self.android_linear_example);if let Some((position,view))=self.android_linear_example.expandable_control_events.remove("expandable") {if let bhoo::android_expandable_list::FlatPosition::Child{group,child}=position {let list=&self.android_linear_example.expandable_lists["expandable"];let g=&list.groups()[group];let item=&g.children[child];let checked=self.android_linear_example.expandable_control_checked("expandable",g.id,Some(item.id),view).unwrap_or(false);self.notice=format!("{}: {}",item.label,if checked{"checked"}else{"unchecked"});}}else if action.is_some() && !self.android_linear_example.expandable_events.contains_key("expandable") {self.notice="Group click intercepted".into();}
                let adapter_events=self.android_linear_example.expandable_lists.get_mut("expandable").unwrap().take_adapter_events();if !adapter_events.is_empty(){self.notice=adapter_events.iter().map(|event|match event {bhoo::android_expandable_list::ExpandableAdapterEvent::Expanded(g)=>format!("Adapter expanded group {}",g+1),bhoo::android_expandable_list::ExpandableAdapterEvent::Collapsed(g)=>format!("Adapter collapsed group {}",g+1)}).collect::<Vec<_>>().join(" · ");}if !self.notice.is_empty(){f.label(&self.notice);}
                if let Some(event)=self.android_linear_example.expandable_events.get("expandable") {let text=match event {ExpandableEvent::Expanded(g)=>format!("Expanded group {}",g+1),ExpandableEvent::Collapsed(g)=>format!("Collapsed group {}",g+1),ExpandableEvent::ChildClicked{group,child}=>format!("Selected group {}, item {}",group+1,child+1)};f.label(&text);}
            }
            "android-adapter-view-flipper" => {
                if self.mask&1==0 {self.android_linear_example.set_list_adapter("adapter_flipper",vec!["Morning".into(),"Afternoon".into(),"Evening".into()],None,None).unwrap();self.android_linear_example.set_list_item_template("adapter_flipper",Some(bhoo::android_xml::ListItemTemplate{row:&android_adapter_flipper_item::LAYOUT,text_id:"flip_title"})).unwrap();self.mask|=1;}
                f.label("Toggle custom animation to compare the XML fade and slide.");
                if f.button(9961,"Previous").clicked{self.android_linear_example.adapter_flipper_step("adapter_flipper",-1);}
                if f.button(9962,"Next").clicked{self.android_linear_example.adapter_flipper_step("adapter_flipper",1);}
                if f.button(9963,"Start or stop").clicked {let running=self.android_linear_example.flipper_running.entry("adapter_flipper".into()).or_insert(true);*running=!*running;}
                if f.button(9964,"Reverse adapter items").clicked {let mut items=self.android_linear_example.list_items["adapter_flipper"].clone();items.reverse();self.android_linear_example.set_list_adapter("adapter_flipper",items,None,None).unwrap();}
                if f.button(9965,"Use host advance").clicked {self.android_linear_example.adapter_flipper_advance_by_host("adapter_flipper");}
                if self.android_linear_example.adapter_flipper_host_advanced.contains("adapter_flipper"){f.label("Host advance is active: use Next or Previous.");}
                if f.button(9966,"Toggle custom animation").clicked{self.mask^=2;}
                let mut xml_false=self.mask&4!=0;if f.switch(9967,"XML loopViews=false",&mut xml_false,true).changed {if xml_false{self.mask|=4;}else{self.mask&=!4;}}
                if xml_false {f.label("Android's flipper still loops.");}
                static DEFAULT_FALSE:bhoo::android_xml::Node=bhoo::android_xml::Node {relative_rules:&[bhoo::android_xml::RelativeRule{kind:137,anchor:"false"}],..android_adapter_view_flipper::LAYOUT};
                static CUSTOM_FALSE:bhoo::android_xml::Node=bhoo::android_xml::Node {relative_rules:&[bhoo::android_xml::RelativeRule{kind:137,anchor:"false"}],..android_adapter_view_flipper_custom::LAYOUT};
                let bounds=f.reserve(160.0);let layout=match (self.mask&2!=0,xml_false) {(true,true)=>&CUSTOM_FALSE,(true,false)=>&android_adapter_view_flipper_custom::LAYOUT,(false,true)=>&DEFAULT_FALSE,(false,false)=>&android_adapter_view_flipper::LAYOUT};layout.render(f,bounds,&mut self.android_linear_example);
                if let Some(i)=self.android_linear_example.adapter_flipper_clicked.get("adapter_flipper"){f.label(&format!("Selected item {}",i+1));}
            }
            "android-sliding-drawer" => {
                f.label("Tap or drag the handle to reveal the drawer.");
                let mut nested=self.mask&2!=0;if f.switch(9984,"Nested handle and content",&mut nested,true).changed {self.mask^=2;self.notice.clear();}
                let mut locked=self.android_linear_example.sliding_drawers.get("drawer").is_some_and(|drawer|drawer.locked);
                if f.switch(9971,"Lock handle",&mut locked,true).changed {self.android_linear_example.set_sliding_drawer_locked("drawer",locked);}
                if f.button(9972,"Open programmatically").clicked {self.android_linear_example.set_sliding_drawer_open("drawer",true,true);}
                if f.button(9973,"Close programmatically").clicked {self.android_linear_example.set_sliding_drawer_open("drawer",false,true);}
                let mut horizontal=self.mask&1!=0;
                if f.switch(9970,"Horizontal",&mut horizontal,true).changed {if horizontal{self.mask|=1;}else{self.mask&=!1;}self.android_linear_example.sliding_drawers.remove("drawer");self.android_linear_example.set_sliding_drawer_locked("drawer",locked);}
                static HORIZONTAL:bhoo::android_xml::Node=bhoo::android_xml::Node{horizontal:true,..android_sliding_drawer::LAYOUT};
                static NESTED_HORIZONTAL:bhoo::android_xml::Node=bhoo::android_xml::Node{horizontal:true,..android_sliding_drawer_nested::LAYOUT};
                let bounds=f.reserve(240.0);
                let layout=match (nested,horizontal){(true,true)=>&NESTED_HORIZONTAL,(true,false)=>&android_sliding_drawer_nested::LAYOUT,(false,true)=>&HORIZONTAL,(false,false)=>&android_sliding_drawer::LAYOUT};
                if layout.render(f,bounds,&mut self.android_linear_example)==Some("nested_action") {self.notice="Nested action pressed".into();}
                for event in self.android_linear_example.take_sliding_drawer_events("drawer") {
                    match event {bhoo::android_xml::SlidingDrawerEvent::Opened=>self.notice="Drawer opened".into(),bhoo::android_xml::SlidingDrawerEvent::Closed=>self.notice="Drawer closed".into(),_=>{}}
                    if self.drawer_event_history.len()==6 {self.drawer_event_history.remove(0);}
                    self.drawer_event_history.push(event);
                }
                if !self.notice.is_empty(){f.label(&self.notice);}
                for event in &self.drawer_event_history {
                    f.label(match event {bhoo::android_xml::SlidingDrawerEvent::ScrollStarted=>"Event: tracking started",bhoo::android_xml::SlidingDrawerEvent::ScrollEnded=>"Event: tracking ended",bhoo::android_xml::SlidingDrawerEvent::Opened=>"Event: opened",bhoo::android_xml::SlidingDrawerEvent::Closed=>"Event: closed"});
                }
            }
            "android-dialer-filter" => {
                f.label("Tap the field to type, or use the buttons to test mode changes.");
                if f.button(9980,"Type 2").clicked{self.android_linear_example.dialer_filters.entry("dialer".into()).or_default().type_char('2');}
                if f.button(9981,"Type A").clicked{self.android_linear_example.dialer_filters.entry("dialer".into()).or_default().type_char('A');}
                if f.button(9982,"Delete last").clicked{self.android_linear_example.dialer_filters.entry("dialer".into()).or_default().backspace();}
                if f.button(9983,"Clear").clicked{self.android_linear_example.dialer_filters.entry("dialer".into()).or_default().clear();}
                let bounds=f.reserve(108.0);android_dialer_filter::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if let Some(data)=self.android_linear_example.dialer_filters.get("dialer"){f.label(&format!("Mode: {:?} · Filter: {}",data.mode,data.filter_text()));}
            }
            "android-tab-host" => {
                if self.mask&1==0 {self.android_linear_example.configure_tab_host_mixed("tabhost","android:tabs",vec![("one".into(),"First".into(),bhoo::android_xml::TabHostContent::View("page_one".into())),("two".into(),"Second".into(),bhoo::android_xml::TabHostContent::Factory(tab_host_demo_factory)),("three".into(),"Third".into(),bhoo::android_xml::TabHostContent::Factory(tab_host_demo_factory))]).unwrap();self.mask|=1;}
                f.label("First tab uses an XML view; the others use lazy factories.");if f.button(9979,"Select third by tag").clicked {self.android_linear_example.set_tab_host_by_tag("tabhost","three");}f.switch(9978,"Right-to-left layout",&mut self.android_linear_example.rtl,true);
                let bounds=f.reserve(220.0);android_tab_host_mixed::LAYOUT.render(f,bounds,&mut self.android_linear_example);f.label(&format!("Factory views created: {}",self.android_linear_example.tab_host_created_count("tabhost")));
            }
            "android-tab-widget" => {
                f.label("Tap a tab to select it.");
                f.switch(9977,"Right-to-left layout",&mut self.android_linear_example.rtl,true);
                let mut invisible=self.android_linear_example.visibility.get("second")==Some(&bhoo::android_xml::Visibility::Invisible);
                if f.switch(9978,"Hide Second (keep space)",&mut invisible,true).changed {
                    self.android_linear_example.visibility.insert("second".into(),if invisible {bhoo::android_xml::Visibility::Invisible}else{bhoo::android_xml::Visibility::Visible});
                }
                let mut fixed=self.mask&1!=0;if f.switch(9979,"Fixed tab widths",&mut fixed,true).changed {self.mask^=1;}
                let mut narrow=self.mask&2!=0;if f.switch(9980,"Narrow widget",&mut narrow,true).changed {self.mask^=2;}
                let mut margins=self.mask&4!=0;if f.switch(9981,"Tab margins example",&mut margins,true).changed {self.mask^=4;}
                let mut weighted=self.mask&8!=0;if f.switch(9982,"Weighted tabs example",&mut weighted,true).changed {self.mask^=8;}
                let mut wrap=self.mask&16!=0;if f.switch(9983,"Wrap-content tabs",&mut wrap,true).changed {self.mask^=16;}
                let mut glyphs=self.mask&32!=0;if f.switch(9984,"Glyph width example",&mut glyphs,true).changed {self.mask^=32;}
                let mut bounds=f.reserve(56.0);if narrow {bounds.width=bounds.width.min(144.0);}
                let layout=if glyphs {&android_tab_widget_glyphs::LAYOUT}else if wrap {&android_tab_widget_wrap::LAYOUT}else if weighted {&android_tab_widget_weighted::LAYOUT}else if margins {&android_tab_widget_margins::LAYOUT}else if fixed {&android_tab_widget_fixed::LAYOUT}else{&android_tab_widget::LAYOUT};layout.render(f,bounds,&mut self.android_linear_example);
                let selected=self.android_linear_example.displayed_children.get("tabs").copied().unwrap_or(0);f.label(&format!("Selected tab: {}",["First","Second","Third"][selected.min(2)]));
            }
            "android-gallery" => {
                if self.mask&1==0 {self.android_linear_example.set_list_adapter("gallery",vec!["Morning".into(),"Afternoon".into(),"Evening".into(),"Night".into()],Some(vec![1,2,3,4]),None).unwrap();self.android_linear_example.set_list_item_template("gallery",Some(bhoo::android_xml::ListItemTemplate{row:&android_gallery_item::LAYOUT,text_id:"gallery_title"})).unwrap();self.mask|=1;}
                f.label("Swipe or tap an item, then use the arrow keys. XML sets spacing, opacity and animation duration.");
                f.switch(9974,"Right-to-left layout",&mut self.android_linear_example.rtl,true);
                let mut short=self.mask&2!=0;if f.switch(9975,"Two items",&mut short,true).changed {if short{self.mask|=2;}else{self.mask&=!2;}let items=if short {vec!["Morning".into(),"Afternoon".into()]}else{vec!["Morning".into(),"Afternoon".into(),"Evening".into(),"Night".into()]};let ids=if short{vec![1,2]}else{vec![1,2,3,4]};self.android_linear_example.set_list_adapter("gallery",items,Some(ids),None).unwrap();}
                if f.button(9976,"Reverse items").clicked {let mut items=self.android_linear_example.list_items["gallery"].clone();let mut ids=self.android_linear_example.list_item_ids["gallery"].clone();items.reverse();ids.reverse();self.android_linear_example.set_list_adapter("gallery",items,Some(ids),None).unwrap();}
                let bounds=f.reserve(160.0);android_gallery::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                let selected=self.android_linear_example.displayed_children.get("gallery").copied().unwrap_or(0);let count=self.android_linear_example.list_items["gallery"].len();f.label(&format!("Selected item {} of {count}",selected+1));
            }
            "android-stack-view" => {
                if self.mask&1==0 {self.android_linear_example.set_list_adapter("stack",vec!["Morning".into(),"Afternoon".into(),"Evening".into(),"Night".into()],None,None).unwrap();self.android_linear_example.set_list_item_template("stack",Some(bhoo::android_xml::ListItemTemplate{row:&android_adapter_flipper_item::LAYOUT,text_id:"flip_title"})).unwrap();self.mask|=1;}
                f.label("Swipe down or up, or use the mouse wheel. Tap the front card to count clicks.");
                let mut large=self.mask&2!=0;if f.switch(9968,"100,000 items",&mut large,true).changed {if large{self.mask|=2;}else{self.mask&=!2;}let items=if large {(1..=100_000).map(|i|format!("Item {i}")).collect()}else{vec!["Morning".into(),"Afternoon".into(),"Evening".into(),"Night".into()]};self.android_linear_example.set_list_adapter("stack",items,None,None).unwrap();}
                let mut looping=self.android_linear_example.stack_looping.get("stack").copied().unwrap_or(false);if f.switch(9960,"Loop items",&mut looping,true).changed {self.android_linear_example.stack_looping.insert("stack".into(),looping);}
                if f.button(9961,"Previous").clicked {self.android_linear_example.stack_step("stack",-1);}
                if f.button(9962,"Next").clicked {self.android_linear_example.stack_step("stack",1);}
                let bounds=f.reserve(220.0);android_stack_view::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                let index=self.android_linear_example.displayed_children.get("stack").copied().unwrap_or(0);let count=self.android_linear_example.list_items.get("stack").map_or(0,Vec::len);f.label(&format!("Item {} of {count}",index+1));
                if let Some(index)=self.android_linear_example.stack_clicked.remove("stack") {self.step+=1;self.notice=format!("Clicked item {} · {} clicks",index+1,self.step);}if !self.notice.is_empty(){f.label(&self.notice);}
            }
            "android-action-menu-view" => {
                if self.mask&1==0 {android_toolbar_menu::install_drawables(&mut self.android_linear_example);self.android_linear_example.toolbar_menu_items.insert("actions".into(),android_toolbar_menu::items());self.mask|=1;}
                f.label("Choose an action or open the overflow menu.");
                if f.button(9911,"Toggle compact width").clicked {self.mask^=2;}
                f.switch(9918,"Right-to-left layout",&mut self.android_linear_example.rtl,true);
                let mut single=self.mask&4!=0;let single_changed=f.switch(9919,"One action",&mut single,true).changed;
                let mut text_action=self.mask&8!=0;let text_changed=f.switch(9920,"Text action",&mut text_action,true).changed;
                let mut wide_config=self.mask&16!=0;if f.switch(9921,"Wide or landscape configuration",&mut wide_config,true).changed {if wide_config{self.mask|=16;}else{self.mask&=!16;}}
                let mut grouped=self.mask&32!=0;let group_changed=f.switch(9922,"Mixed action group",&mut grouped,true).changed;
                self.android_linear_example.menu_allow_text_with_icon=Some(wide_config);
                if single_changed||text_changed||group_changed {
                    if single {self.mask|=4;}else{self.mask&=!4;}if text_action {self.mask|=8;}else{self.mask&=!8;}if grouped {self.mask|=32;}else{self.mask&=!32;}
                    let mut items=android_toolbar_menu::items();if grouped {items[0].show_as_action=bhoo::android_xml::AndroidShowAsAction::Always;items[1].label="Save".into();items[1].icon=None;items[1].icon_drawable=None;}if single {items.truncate(1);}if text_action {items[0].label="Save".into();items[0].icon=None;items[0].icon_drawable=None;}self.android_linear_example.toolbar_menu_items.insert("actions".into(),items);
                }
                let mut bounds=f.reserve(56.0);if self.mask&2!=0{bounds.width=bounds.width.min(168.0);}
                android_action_menu_view::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if let Some(item)=self.android_linear_example.toolbar_menu_clicked.remove("actions") {self.notice=format!("Selected: {}",self.android_linear_example.toolbar_menu_items["actions"].iter().find(|entry|entry.id==item).map_or("action",|entry|entry.label.as_str()));}
                if !self.notice.is_empty(){f.label(&self.notice);}
            }
            "android-toolbar" => {
                if self.mask&1==0 {android_toolbar::install_drawables(&mut self.android_linear_example);android_toolbar_menu::install_drawables(&mut self.android_linear_example);self.android_linear_example.toolbar_menu_items.insert("toolbar".into(),android_toolbar_menu::items());self.mask|=1;}
                f.label("Open the menu and choose Compact or Comfortable. Only one can be checked.");
                let mut wide_config=self.mask&128!=0;if f.switch(9918,"Wide or landscape configuration",&mut wide_config,true).changed {if wide_config{self.mask|=128;}else{self.mask&=!128;}}
                self.android_linear_example.menu_allow_text_with_icon=Some(wide_config);
                let mut fewer_actions=self.mask&256!=0;if f.switch(9919,"Fewer inline actions",&mut fewer_actions,true).changed {
                    if fewer_actions {self.mask|=256;}else{self.mask&=!256;}
                    if let Some(item)=self.android_linear_example.toolbar_menu_items.get_mut("toolbar").and_then(|items|items.iter_mut().find(|item|item.id==bhoo::android_xml::view_id("archive"))) {item.visible=!fewer_actions;}
                }
                if f.button(9911,"Toggle compact width").clicked{self.mask^=2;}
                if f.button(9912,"Reverse menu order").clicked{self.android_linear_example.toolbar_menu_items.get_mut("toolbar").unwrap().reverse();}
                if f.button(9913,"Toggle icon tint").clicked{self.mask^=4;let tint=if self.mask&4!=0{bhoo::android_xml::AndroidDrawableTint::Fixed(f.theme().primary)}else{bhoo::android_xml::AndroidDrawableTint::ControlNormal};self.android_linear_example.drawable_tints.insert("toolbar_vector_search".into(),tint);}
                if f.button(9914,"Toggle long title").clicked{self.mask^=8;let title=if self.mask&8!=0{"Messages from the entire family"}else{"Inbox"};self.android_linear_example.text.insert("toolbar".into(),title.into());}
                if f.button(9915,"Toggle title colors").clicked{self.mask^=16;if self.mask&16!=0 {self.android_linear_example.toolbar_title_colors.insert("toolbar".into(),bhoo::Color(84,38,150,255));self.android_linear_example.toolbar_subtitle_colors.insert("toolbar".into(),bhoo::Color(117,75,138,255));}else{self.android_linear_example.toolbar_title_colors.remove("toolbar");self.android_linear_example.toolbar_subtitle_colors.remove("toolbar");}}
                if f.button(9916,"Enable or disable menu group").clicked {self.mask^=32;self.android_linear_example.set_toolbar_group_enabled("toolbar",bhoo::android_xml::view_id("secondary_actions"),self.mask&32!=0);}
                if f.button(9917,"Show or hide menu group").clicked {self.mask^=64;self.android_linear_example.set_toolbar_group_visible("toolbar",bhoo::android_xml::view_id("secondary_actions"),self.mask&64==0);}
                let mut bounds=f.reserve(56.0);if self.mask&2!=0{bounds.width=bounds.width.min(256.0);}let action=android_toolbar::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if let Some(item)=self.android_linear_example.toolbar_menu_clicked.remove("toolbar"){if item==bhoo::android_xml::view_id("compact")||item==bhoo::android_xml::view_id("comfortable"){self.android_linear_example.set_toolbar_item_checked("toolbar",item,true);}self.notice=format!("{} clicked",if item==bhoo::android_xml::view_id("search"){"Search"}else if item==bhoo::android_xml::view_id("archive"){"Archive"}else if item==bhoo::android_xml::view_id("settings"){"Settings"}else if item==bhoo::android_xml::view_id("compact"){"Compact"}else if item==bhoo::android_xml::view_id("comfortable"){"Comfortable"}else{"About"});}else if action==Some("toolbar_edit"){self.notice="Edit clicked".into();}else if action==Some("toolbar"){self.notice="Navigation clicked".into();}if !self.notice.is_empty(){f.label(&self.notice);}
            }
            "android-search-view" => {
                self.android_linear_example.search_submit_enabled.insert("search".into());if self.mask&1==0{self.android_linear_example.set_autocomplete_items("search",["Calendar","Camera","Cats","Contacts","Documents","Downloads","Messages","Music","Photos","Settings"].into_iter().map(str::to_owned).collect());self.mask|=1;}
                f.label("Open search and type to see local suggestions.");let query=self.android_linear_example.text.get("search").map_or("",String::as_str);f.label(&format!("Query: {query}"));if let Some(submitted)=self.android_linear_example.search_submitted.get("search"){f.label(&format!("Submitted: {submitted}"));}if let Some(index)=self.android_linear_example.search_suggestion_clicked.get("search"){let items=["Calendar","Camera","Cats","Contacts","Documents","Downloads","Messages","Music","Photos","Settings"];if let Some(label)=items.get(*index){f.label(&format!("Suggestion: {label}"));}}
                let bounds=f.reserve(48.0);android_search_view::LAYOUT.render(f,bounds,&mut self.android_linear_example);
            }
            "android-number-picker" => {
                if self.mask & 1 == 0 {self.android_linear_example.number_pickers.insert("quantity".into(),bhoo::android_number_picker::NumberPickerState::new(0,9,5).unwrap());self.mask|=1;}
                f.label("Tap above or below the number, or drag the wheel.");let bounds=f.reserve(144.0);android_number_picker::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if let Some(picker)=self.android_linear_example.number_pickers.get("quantity") {f.label(&format!("Value: {}",picker.value()));}
            }
            "android-date-picker-combined" => {
                f.label("Choose a day on the calendar or change the wheels.");let bounds=f.reserve(340.0);android_date_picker_combined::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if let Some(picker)=self.android_linear_example.date_spinners.get("date_combined") {let d=picker.date();f.label(&format!("Selected: {:04}-{:02}-{:02}",d.year,d.month,d.day));}
            }
            "android-time-picker-clock" => {
                f.label("Choose an hour on the dial, then a minute.");
                if f.button(9898,"Switch 12/24 hour").clicked {if let Some(p)=self.android_linear_example.time_clocks.get_mut("time_clock"){p.set_24_hour(!p.is_24_hour());}f.continuous(f.available());}
                let bounds=f.reserve(476.0);android_time_picker_clock::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if let Some(p)=self.android_linear_example.time_clocks.get("time_clock") {f.label(&format!("Time: {:02}:{:02}",p.hour(),p.minute()));}
            }
            "android-time-picker-spinner" => {
                f.label("Tap, drag, flick or edit the time wheels.");
                if f.button(9899,"Switch 12/24 hour").clicked {if let Some(p)=self.android_linear_example.time_spinners.get_mut("time_spinner"){p.set_24_hour(!p.is_24_hour());}f.continuous(f.available());}
                let bounds=f.reserve(144.0);android_time_picker_spinner::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if let Some(p)=self.android_linear_example.time_spinners.get("time_spinner") {f.label(&format!("Time: {:02}:{:02}",p.hour(),p.minute()));}
            }
            "android-date-picker-spinner" => {
                f.label("Tap, flick, hold or edit the month, day and year wheels.");let bounds=f.reserve(144.0);android_date_picker_spinner::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                if let Some(picker)=self.android_linear_example.date_spinners.get("date_spinner") {let d=picker.date();f.label(&format!("Selected: {:04}-{:02}-{:02}",d.year,d.month,d.day));}
            }
            "android-date-picker" => {
                f.label("Select a day, or tap the year to choose another year.");
                let bounds=f.reserve(454.0);android_date_picker::LAYOUT.render(f,bounds,&mut self.android_clock_example);
                if let Some(calendar)=self.android_clock_example.calendars.get("date") {let d=calendar.selected();f.label(&format!("Selected: {:04}-{:02}-{:02}",d.year,d.month,d.day));}
            }
            "android-calendar-view" => {
                f.label("Select a day or use the arrows to change month.");
                let bounds=f.reserve(356.0);android_calendar_view::LAYOUT.render(f,bounds,&mut self.android_clock_example);
                if let Some(calendar)=self.android_clock_example.calendars.get("calendar") {let d=calendar.selected();f.label(&format!("Selected: {:04}-{:02}-{:02}",d.year,d.month,d.day));}
            }
            "android-autocomplete" => {
                if self.mask&1==0 {
                    self.android_edit_example.set_autocomplete_items("city",["Amsterdam","Athens","Austin","Barcelona","Berlin","Boston","Cape Town","Copenhagen","New Delhi","New Orleans","New York","Paris","San Diego","San Francisco","San Jose","Santa Fe","Tokyo","Vienna"].into_iter().map(str::to_owned).collect());self.mask|=1;
                }
                f.label("Type at least two letters to see matching cities.");
                let bounds=f.reserve(64.0);
                android_autocomplete::LAYOUT.render(f,bounds,&mut self.android_edit_example);
            }
            "android-edittext" => {
                f.label("Tap the field and type");
                let bounds = f.reserve(192.0);
                android_edittext::LAYOUT.render(f, bounds, &mut self.android_edit_example);
                let name = self.android_edit_example.text.get("name").map_or("", String::as_str);
                let email = self.android_edit_example.text.get("email").map_or("", String::as_str);
                f.label(format!("Name: {name} · Email: {email}"));
            }
            "android-scrollview" => {
                f.label("ScrollView clipping to padding");
                let bounds = f.reserve(288.0);
                android_scrollview::LAYOUT.render(f, bounds, &mut self.horizontal_example);
            }
            "android-gridview" => {
                f.label("Choose workspaces. Scroll to see more.");
                let mut custom=self.mask&1!=0;if f.switch(210,"Use XML item layout",&mut custom,true).changed {self.mask^=1;}
                let mut rtl=self.mask&2!=0;if f.switch(211,"Right-to-left",&mut rtl,true).changed {self.mask^=2;}
                let mut narrow=self.mask&8!=0;if f.switch(214,"Narrow centered items",&mut narrow,true).changed {self.mask^=8;}
                let mut individual=self.mask&16!=0;
                if f.switch(215,"Mixed item layouts",&mut individual,true).changed {
                    self.mask^=16;
                    if individual {self.mask|=8;narrow=true;self.android_linear_example.images.entry("grid_alt".into()).or_insert_with(sample_media_image);android_gridview_team_item::install_drawables(&mut self.android_linear_example);}
                    for i in 0..6 {
                        let binding=if individual {
                            let mut binding=bhoo::android_xml::AdapterItemBinding::default();
                            if i%2==0 {binding.text.insert("grid_item_subtitle".into(),"Personal".into());}
                            else {binding.template=Some(bhoo::android_xml::ListItemTemplate {row:&android_gridview_team_item::LAYOUT,text_id:"team_title"});binding.text.insert("team_subtitle".into(),"Team".into());binding.images.insert("team_picture".into(),"grid_alt".into());}
                            Some(binding)
                        }else{None};
                        let _=self.android_linear_example.set_list_item_binding("workspaces_grid",i,binding);
                    }
                }
                let mut large=self.mask&4!=0;if f.switch(212,"100,000 items",&mut large,true).changed {
                    self.mask^=4;let count=if large {100_000}else{24};
                    let items=(0..count).map(|i|format!("Workspace {i}")).collect();
                    self.android_linear_example.set_list_adapter("workspaces_grid",items,None,None).expect("valid grid adapter");
                    self.android_linear_example.list_checked.remove("workspaces_grid");self.android_linear_example.offsets.insert("workspaces_grid".into(),0.0);self.notice.clear();
                }
                if f.button(213,"Clear choices").clicked {self.android_linear_example.list_checked.remove("workspaces_grid");}
                let mut interactive=self.mask&32!=0;if f.switch(216,"Interactive XML items",&mut interactive,true).changed {self.mask^=32;}
                if f.button(217,"Swap first two items").clicked && let Some(mut items)=self.android_linear_example.list_items.get("workspaces_grid").cloned() && items.len()>1 {let mut ids=self.android_linear_example.list_item_ids.get("workspaces_grid").cloned().unwrap_or_else(||items.iter().map(|label|bhoo::android_xml::view_id(label) as i64).collect());self.android_linear_example.set_list_adapter("workspaces_grid",items.clone(),Some(ids.clone()),None).expect("unique IDs");items.swap(0,1);ids.swap(0,1);self.android_linear_example.set_list_adapter("workspaces_grid",items,Some(ids),None).expect("unique IDs");}
                self.android_linear_example.rtl=rtl;
                let template=if narrow {android_gridview_item::install_drawables(&mut self.android_linear_example);Some(bhoo::android_xml::ListItemTemplate {row:&android_gridview_item::LAYOUT,text_id:"grid_item_title"})}else if custom {android_spinner_dropdown_row::install_drawables(&mut self.android_linear_example);self.android_linear_example.image_scales.insert("switcher_day".into(),0.5);Some(bhoo::android_xml::ListItemTemplate {row:&android_spinner_dropdown_row::LAYOUT,text_id:"spinner_title"})}else{None};
                if interactive {self.android_linear_example.set_interactive_list_item_template("workspaces_grid",Some(bhoo::android_xml::ListItemTemplate{row:&android_list_interactive_row::LAYOUT,text_id:"row_title"})).expect("valid interactive grid item");}else{self.android_linear_example.set_list_item_template("workspaces_grid",template).expect("valid grid template");}
                self.android_linear_example.list_items.entry("workspaces_grid".into()).or_insert_with(||(0..24).map(|i|format!("Workspace {i}")).collect());
                let count=self.android_linear_example.list_checked.get("workspaces_grid").map_or(0,Vec::len);f.label(&format!("Selected: {count}"));
                self.android_linear_example.list_long_click_enabled.insert("workspaces_grid".into(),true);
                let bounds=f.reserve(320.0);
                let layout=if narrow {&android_gridview_center::LAYOUT}else{&android_gridview::LAYOUT};
                if layout.render(f,bounds,&mut self.android_linear_example).is_some() {
                    if let Some((index,_))=self.android_linear_example.list_control_clicked.remove("workspaces_grid") {self.notice=format!("Item checkbox: {}",self.android_linear_example.list_items["workspaces_grid"][index]);}
                    else if let Some(index)=self.android_linear_example.list_long_clicked.remove("workspaces_grid") {self.notice=format!("Held: {}",self.android_linear_example.list_items["workspaces_grid"][index]);}
                    else if let Some(index)=self.android_linear_example.list_clicked.get("workspaces_grid") {self.notice=format!("Last choice: {}",self.android_linear_example.list_items["workspaces_grid"][*index]);}
                }
                f.label(&self.notice);
            }
            "android-video-view" => {
                use bhoo::android_xml::{MediaControllerAction,MediaControllerCapabilities,MediaPlayerSnapshot};
                if self.mask&1==0 {android_media_controller::install_drawables(&mut self.android_linear_example);self.android_linear_example.video_frames.insert("video".into(),sample_media_image());self.android_linear_example.video_ready.insert("video".into());self.android_linear_example.video_controllers.insert("video".into(),("controller".into(),100.0));self.android_linear_example.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:12000,duration_ms:125000,buffered_percent:65,playing:false});self.android_linear_example.media_controller_capabilities.insert("controller".into(),MediaControllerCapabilities{has_previous:true,has_next:true,..Default::default()});self.mask|=1;}
                f.label(if self.video_host_active{"Tap the video to show playback controls."}else{"Tap the simulated video frame to show or hide playback controls."});
                let mut linear=false;let mut weighted=false;
                if !self.video_host_active {let mut ready=self.android_linear_example.video_ready.contains("video");if f.switch(9970,"Player prepared",&mut ready,true).changed {if ready{self.android_linear_example.video_ready.insert("video".into());}else{self.android_linear_example.video_ready.remove("video");self.android_linear_example.hide_floating_media_controller("controller");}}
                let mut portrait=self.mask&2!=0;if f.switch(9971,"Portrait video dimensions",&mut portrait,true).changed {self.mask^=2;let image=if portrait{let mut pixels=Vec::new();for y in 0..64u8 {for x in 0..40u8{pixels.extend_from_slice(&[30+x*3,80+y*2,160,255]);}}Image::rgba(40,64,pixels).unwrap()}else{sample_media_image()};self.android_linear_example.video_frames.insert("video".into(),image);}
                linear=self.mask&4!=0;if f.switch(9969,"Linear layout with next view",&mut linear,true).changed {self.mask^=4;}
                weighted=self.mask&8!=0;if linear&&f.switch(9968,"Weighted video",&mut weighted,true).changed {self.mask^=8;}}
                let bounds=f.reserve(if linear&&weighted{300.0}else if linear{252.0}else{220.0});f.paint_rect(bounds,0.0,bhoo::Color(35,35,35,255));if linear&&weighted{android_video_weighted_measure::LAYOUT.render(f,bounds,&mut self.android_linear_example);}else if linear{android_video_linear_measure::LAYOUT.render(f,bounds,&mut self.android_linear_example);}else{android_video_measure::LAYOUT.render(f,bounds,&mut self.android_linear_example);}
                let image=&self.android_linear_example.video_frames["video"];let size=self.android_linear_example.video_dimensions.get("video").copied().unwrap_or((image.width(),image.height()));let (w,h)=bhoo::android_xml::measure_video_view(size,bhoo::android_xml::VideoMeasureSpec::Exactly(bounds.width as u32),bhoo::android_xml::VideoMeasureSpec::Exactly(if linear&&weighted{268}else{220}));let video_bounds=Rect{x:bounds.x+if linear{0.0}else{(bounds.width-w as f32)*0.5},y:bounds.y+if linear{0.0}else{(bounds.height-h as f32)*0.5},width:w as f32,height:h as f32};android_media_controller::LAYOUT.render_floating_media_controller(f,video_bounds,100.0,&mut self.android_linear_example);
                if let Some(event)=self.android_linear_example.media_controller_actions.remove("controller") {if self.video_host_active{self.video_host_command=Some(event);}else{let player=self.android_linear_example.media_players.get_mut("controller").unwrap();match event {MediaControllerAction::Play=>player.playing=true,MediaControllerAction::Pause=>player.playing=false,MediaControllerAction::SeekTo(position)=>player.position_ms=position,MediaControllerAction::Previous=>player.position_ms=0,MediaControllerAction::Next=>player.position_ms=player.duration_ms};self.notice=format!("Simulated player: {event:?}");}}if !self.notice.is_empty(){f.label(&self.notice);}
            }
            "android-media-controller" => {
                use bhoo::android_xml::{MediaControllerAction,MediaControllerCapabilities,MediaPlayerSnapshot};
                static SECOND:bhoo::android_xml::Node=bhoo::android_xml::Node{id:"second_controller",..android_media_controller::LAYOUT};
                if self.mask&1==0 {android_media_controller::install_drawables(&mut self.android_linear_example);self.android_linear_example.media_players.insert("controller".into(),MediaPlayerSnapshot{position_ms:12000,duration_ms:125000,buffered_percent:65,playing:false});self.android_linear_example.media_players.insert("second_controller".into(),MediaPlayerSnapshot{position_ms:61000,duration_ms:240000,buffered_percent:80,playing:true});for id in ["controller","second_controller"]{self.android_linear_example.media_controller_capabilities.insert(id.into(),MediaControllerCapabilities{has_previous:true,has_next:true,..Default::default()});}self.mask|=1;}
                f.label("Simulated player: play, pause, seek, or use the track controls.");
                let mut two=self.mask&2!=0;if f.switch(9979,"Show second player",&mut two,true).changed {self.mask^=2;}
                let mut floating=self.mask&4!=0;if f.switch(9975,"Floating controller",&mut floating,true).changed {self.mask^=4;if floating {self.android_linear_example.show_floating_media_controller("controller",f.time(),3000);}else{self.android_linear_example.hide_floating_media_controller("controller");}}
                let mut delayed=self.mask&8!=0;if f.switch(9973,"Delay host seek updates",&mut delayed,true).changed {self.mask^=8;}
                if let Some(position)=self.media_pending_seek {f.label(&format!("Pending seek: {} seconds",position/1000));if f.button(9972,"Apply pending seek").clicked {self.android_linear_example.media_players.get_mut("controller").unwrap().position_ms=position;self.media_pending_seek=None;}}
                let capabilities=self.android_linear_example.media_controller_capabilities.get_mut("controller").unwrap();
                f.switch(9976,"Allow pause",&mut capabilities.can_pause,true);
                let mut can_seek=capabilities.can_seek_forward&&capabilities.can_seek_backward;if f.switch(9977,"Allow seeking",&mut can_seek,true).changed {capabilities.can_seek_forward=can_seek;capabilities.can_seek_backward=can_seek;}
                if f.button(9978,"Advance simulated player 10 seconds").clicked {let player=self.android_linear_example.media_players.get_mut("controller").unwrap();player.position_ms=(player.position_ms+10000).min(player.duration_ms);}
                if floating {if f.button(9974,"Show floating controls").clicked {self.android_linear_example.show_floating_media_controller("controller",f.time(),3000);}let bounds=f.reserve(200.0);f.paint_rect(bounds,4.0,f.theme().surface_variant);android_media_controller::LAYOUT.render_floating_media_controller(f,bounds,100.0,&mut self.android_linear_example);}else{let bounds=f.reserve(100.0);android_media_controller::LAYOUT.render(f,bounds,&mut self.android_linear_example);}
                if two {let bounds=f.reserve(100.0);SECOND.render(f,bounds,&mut self.android_linear_example);}
                for id in ["controller","second_controller"] {if let Some(event)=self.android_linear_example.media_controller_actions.remove(id) {let player=self.android_linear_example.media_players.get_mut(id).unwrap();match event {MediaControllerAction::Play=>player.playing=true,MediaControllerAction::Pause=>player.playing=false,MediaControllerAction::SeekTo(position)=>if delayed&&id=="controller"{self.media_pending_seek=Some(position)}else{player.position_ms=position},MediaControllerAction::Previous=>player.position_ms=0,MediaControllerAction::Next=>player.position_ms=player.duration_ms};self.notice=format!("{id}: {event:?}");}}if !self.notice.is_empty(){f.label(&self.notice);}
            }
            "android-quick-contact-badge" => {
                use bhoo::android_xml::QuickContactTarget;
                if self.mask&1==0 {self.android_linear_example.images.insert("contact_photo".into(),bhoo::android_xml::quick_contact_default_avatar());self.android_linear_example.image_sources.insert("contact_badge".into(),"contact_photo".into());self.android_linear_example.assign_quick_contact("contact_badge",Some(QuickContactTarget::Email("alex@example.test".into())),true);self.mask|=1;}
                f.label("Tap the badge to request a lookup; simulate the host response below.");
                if f.button(9981,"Assign email (lazy)").clicked {self.android_linear_example.assign_quick_contact("contact_badge",Some(QuickContactTarget::Email("alex@example.test".into())),true);self.notice.clear();}
                if f.button(9982,"Assign phone (immediate lookup)").clicked {self.android_linear_example.assign_quick_contact("contact_badge",Some(QuickContactTarget::Phone("+15550100".into())),false);self.notice.clear();}
                if f.button(9983,"Clear contact").clicked {self.android_linear_example.assign_quick_contact("contact_badge",None,true);self.notice.clear();}
                if let Some(request)=self.android_linear_example.quick_contact_lookups.get("contact_badge").cloned(){f.label(&format!("Pending lookup: {:?}",request.target));if f.button(9984,"Simulate contact found").clicked{self.android_linear_example.resolve_quick_contact("contact_badge",request.token,Some("content://contacts/alex".into()));}if f.button(9985,"Simulate no match").clicked{self.android_linear_example.resolve_quick_contact("contact_badge",request.token,None);}}
                let mut bounds=f.reserve(80.0);bounds.width=80.0;android_quick_contact_badge::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                self.android_linear_example.quick_contact_clicked.remove("contact_badge");if let Some(action)=self.android_linear_example.quick_contact_actions.remove("contact_badge") {self.notice=format!("Host action: {action:?}");}if !self.notice.is_empty(){f.label(&self.notice);}
            }
            "android-listview" => {
                f.label("Choose workspaces. Scroll to see more.");
                let mut disable=self.mask&1!=0;
                if self.mask&8==0 && f.switch(200,"Disable Shared",&mut disable,true).changed {self.mask^=1;}
                if self.mask&8!=0 {disable=false;}
                let mut custom=self.mask&2!=0;
                if f.switch(202,"Use XML row layout",&mut custom,true).changed {self.mask^=2;}
                let mut fixed=self.mask&4!=0;
                if f.switch(204,"Show header and footer",&mut fixed,true).changed {self.mask^=4;}
                let mut large=self.mask&8!=0;
                if f.switch(205,"100,000 rows",&mut large,true).changed {
                    self.mask^=8;self.mask&=!1;
                    if large {
                        let items=(0..100_000).map(|i|if i==99_999 {"Final workspace with a longer name that wraps across more than one line".into()}else{format!("Workspace {i}")}).collect();
                        self.android_linear_example.set_list_adapter("workspaces_list",items,None,None).expect("valid large ListView adapter");
                    } else {self.android_linear_example.list_items.remove("workspaces_list");self.android_linear_example.list_item_ids.remove("workspaces_list");}
                    self.android_linear_example.list_checked.remove("workspaces_list");self.android_linear_example.list_clicked.remove("workspaces_list");self.android_linear_example.offsets.insert("workspaces_list".into(),0.0);self.notice.clear();
                }
                if f.button(201,"Clear choices").clicked {self.android_linear_example.list_checked.remove("workspaces_list");}
                let swap=f.button(203,"Swap first two").clicked;
                if swap && let Some(mut items)=self.android_linear_example.list_items.get("workspaces_list").cloned() && items.len()>1 {
                    let mut ids=self.android_linear_example.list_item_ids.get("workspaces_list").cloned().unwrap_or_else(||items.iter().map(|label|bhoo::android_xml::view_id(label) as i64).collect());
                    self.android_linear_example.set_list_adapter("workspaces_list",items.clone(),Some(ids.clone()),None).expect("unique example IDs");
                    items.swap(0,1);ids.swap(0,1);
                    self.android_linear_example.set_list_adapter("workspaces_list",items,Some(ids),None).expect("unique example IDs");
                }
                let mut interactive=self.mask&16!=0;
                if f.switch(206,"Interactive XML rows",&mut interactive,true).changed {self.mask^=16;}
                let template=if custom {
                    android_spinner_dropdown_row::install_drawables(&mut self.android_linear_example);
                    self.android_linear_example.image_scales.insert("switcher_day".into(),0.5);
                    Some(bhoo::android_xml::ListItemTemplate {row:&android_spinner_dropdown_row::LAYOUT,text_id:"spinner_title"})
                } else {None};
                if interactive {self.android_linear_example.set_interactive_list_item_template("workspaces_list",Some(bhoo::android_xml::ListItemTemplate {row:&android_list_interactive_row::LAYOUT,text_id:"row_title"})).expect("valid interactive row");}else{self.android_linear_example.set_list_item_template("workspaces_list",template).expect("valid ListView row template");}
                let mut mixed=self.mask&64!=0;
                if f.switch(208,"Checkbox only on second item",&mut mixed,true).changed {self.mask^=64;let binding=if mixed {let mut binding=bhoo::android_xml::AdapterItemBinding::default();binding.template=Some(bhoo::android_xml::ListItemTemplate{row:&android_list_interactive_row::LAYOUT,text_id:"row_title"});Some(binding)}else{None};let target=if large{"Workspace 1"}else{"Projects"};if let Some(index)=self.android_linear_example.list_items.get("workspaces_list").and_then(|items|items.iter().position(|item|item==target)){let _=self.android_linear_example.set_interactive_list_item_binding("workspaces_list",index,binding);}}
                let mut fixed_controls=self.mask&32!=0;
                if f.switch(207,"Interactive header",&mut fixed_controls,true).changed {self.mask^=32;}
                let (headers,footers)=if fixed {(vec![bhoo::android_xml::ListFixedView {view:if fixed_controls{&android_list_interactive_header::LAYOUT}else{&android_list_header::LAYOUT},selectable:false,label:"Workspaces"}],vec![bhoo::android_xml::ListFixedView {view:&android_list_footer::LAYOUT,selectable:true,label:"End of workspaces"}])}else{(vec![],vec![])};
                self.android_linear_example.set_list_fixed_views("workspaces_list",headers,footers).expect("valid fixed ListView views");
                if disable {
                    let shared=self.android_linear_example.list_items.get("workspaces_list").and_then(|items|items.iter().position(|label|label=="Shared")).unwrap_or(2);
                    let mut enabled=vec![true;shared+1];enabled[shared]=false;self.android_linear_example.list_enabled.insert("workspaces_list".into(),enabled);
                } else {self.android_linear_example.list_enabled.remove("workspaces_list");}
                let count=self.android_linear_example.list_checked.get("workspaces_list").map_or(0,Vec::len);
                f.label(&format!("Selected: {count}"));
                if let (Some(indices),Some(items))=(self.android_linear_example.list_checked.get("workspaces_list"),self.android_linear_example.list_items.get("workspaces_list")) {
                    let names=indices.iter().filter_map(|index|items.get(*index).map(String::as_str)).collect::<Vec<_>>().join(", ");
                    if !names.is_empty() {f.label(&format!("Checked: {names}"));}
                }
                self.android_linear_example.list_long_click_enabled.insert("workspaces_list".into(),true);
                let bounds=f.reserve(320.0);
                if android_listview::LAYOUT.render(f,bounds,&mut self.android_linear_example).is_some() {
                    if self.android_linear_example.list_fixed_control_clicked.remove("workspaces_list").is_some() {self.notice="Header switch changed".into();}
                    else if let Some((index,_))=self.android_linear_example.list_control_clicked.remove("workspaces_list") {self.notice=format!("Row checkbox: {}",self.android_linear_example.list_items["workspaces_list"][index]);}
                    else if let Some((true,_))=self.android_linear_example.list_fixed_clicked.remove("workspaces_list") {self.notice="Footer selected".into();}
                    else if let Some(index)=self.android_linear_example.list_long_clicked.remove("workspaces_list") {self.notice=format!("Held: {}",self.android_linear_example.list_items["workspaces_list"][index]);}
                    else if let Some(index)=self.android_linear_example.list_clicked.get("workspaces_list") {self.notice=format!("Last choice: {}",self.android_linear_example.list_items["workspaces_list"][*index]);}
                }
                f.label(&self.notice);
            }
            "android-spinner" => {
                let showing=self.android_linear_example.spinner_open.is_some();
                let mut reverse=false;
                f.without_input(showing,|f| {
                    f.switch(90,"Dialog mode",&mut self.touched,true);
                    let mut wide=self.mask&1!=0;if f.switch(91,"Wide dropdown",&mut wide,true).changed {self.mask^=1;}
                    let mut rtl=self.mask&2!=0;if f.switch(92,"Right-to-left layout",&mut rtl,true).changed {self.mask^=2;}
                    let mut disable=self.mask&4!=0;if f.switch(93,"Disable Shared",&mut disable,true).changed {self.mask^=4;}
                    let mut shorter=self.mask&8!=0;if f.switch(94,"Use shorter list",&mut shorter,true).changed {
                        self.mask^=8;
                        for key in ["workspace","wide_workspace","dialog_workspace"] {self.android_linear_example.spinner_items.remove(key);self.android_linear_example.spinner_item_ids.remove(key);}
                    }
                    let mut custom=self.mask&16!=0;if f.switch(96,"Use XML item views",&mut custom,true).changed {self.mask^=16;}
                    if custom {let mut large=self.mask&32!=0;if f.switch(97,"Large row images",&mut large,true).changed {self.mask^=32;}}
                    if f.button(95,"Reverse adapter order").clicked {reverse=true;}
                });
                let wide=self.mask&1!=0;self.android_linear_example.rtl=self.mask&2!=0;
                f.label("Choose a workspace");
                f.label("Scroll the open list for more choices.");
                f.label(&self.notice);
                let mut bounds=f.reserve(if self.mask&16!=0 && (!wide || self.touched) {160.0} else {48.0});if wide && !self.touched {bounds.width=bounds.width.min(100.0);}
                let layout=if self.touched {&android_spinner_dialog::LAYOUT} else if wide {&android_spinner_wide::LAYOUT} else {&android_spinner::LAYOUT};
                let key=if self.touched {"dialog_workspace"} else if wide {"wide_workspace"} else {"workspace"};
                if self.mask&16!=0 {android_spinner_dropdown_row::install_drawables(&mut self.android_linear_example);self.android_linear_example.image_scales.insert("switcher_day".into(),if self.mask&32!=0 {1.5}else{0.5});}
                let templates=if self.mask&16!=0 {Some(bhoo::android_xml::SpinnerItemTemplates {selected:&android_spinner_selected_row::LAYOUT,dropdown:&android_spinner_dropdown_row::LAYOUT,text_id:"spinner_title"})} else {None};
                self.android_linear_example.set_spinner_item_templates(key,templates).expect("valid Spinner example templates");
                if self.mask&8!=0 {self.android_linear_example.spinner_items.entry(key.into()).or_insert_with(||vec!["Personal".into(),"Shared".into(),"Archive".into()]);}
                if reverse && let Some(mut items)=self.android_linear_example.spinner_items.get(key).cloned() {
                    let mut ids=self.android_linear_example.spinner_item_ids.get(key).cloned().unwrap_or_else(||items.iter().map(|label|bhoo::android_xml::view_id(label) as i64).collect());
                    let _=self.android_linear_example.set_spinner_adapter(key,items.clone(),Some(ids.clone()),None);
                    items.reverse();ids.reverse();let _=self.android_linear_example.set_spinner_adapter(key,items,Some(ids),None);
                }
                let shared=self.android_linear_example.spinner_items.get(key).and_then(|items|items.iter().position(|item|item=="Shared")).unwrap_or(if self.mask&8!=0 {1} else {2});
                let mut flags=vec![true;12];if self.mask&4!=0 {flags[shared]=false;}
                self.android_linear_example.spinner_enabled.insert(key.into(),flags);
                if layout.render(f,bounds,&mut self.android_linear_example).is_some() {
                    let index=self.android_linear_example.spinner_selected.get(key).copied().unwrap_or(0);
                    let label=self.android_linear_example.spinner_items.get(key).and_then(|items|items.get(index)).map(String::as_str).unwrap_or("");
                    self.notice=format!("Selected: {label}");
                }
            }
            "android-space" => {
                f.switch(70, "Remove background-sized gap", &mut self.touched, true);
                self.android_linear_example.visibility.insert("gap".into(), if self.touched { bhoo::android_xml::Visibility::Gone } else { bhoo::android_xml::Visibility::Visible });
                let bounds = f.reserve(176.0);
                android_space::LAYOUT.render(f, bounds, &mut self.android_linear_example);
            }
            "material-standard-bottom-sheet" => {
                let viewport=f.viewport();
                if filled(f,100,"Show or hide sheet").clicked {self.overlay.open=!self.overlay.open;}
                if f.button(101,"Expand or collapse sheet").clicked {
                    self.overlay.bottom_sheet_expanded=!self.overlay.bottom_sheet_expanded;
                    self.overlay.bottom_sheet_half_expanded=false;
                }
                f.switch(102,"Taller peek height",&mut self.touched,true);
                f.label("The rest of the page stays available.");
                f.label(if self.overlay.bottom_sheet_expanded {"Sheet: expanded"} else if self.overlay.bottom_sheet_half_expanded {"Sheet: halfway"} else {"Sheet: collapsed"});
                let height=360.0_f32.min(viewport.height);let width=640.0_f32.min(viewport.width);
                let panel=Rect{x:viewport.x+(viewport.width-width)*0.5,y:viewport.y+viewport.height-height,width,height};
                self.overlay.kind=OverlayKind::BottomSheet;self.overlay.style=OverlayStyle::Material3;self.overlay.drag_to_dismiss=true;
                self.overlay.bottom_sheet_peek=Some(if self.touched {144.0} else {96.0});self.overlay.bottom_sheet_half_ratio=Some(0.3);self.overlay.bottom_sheet_content_top=Some(72.0);self.overlay.bottom_sheet_content_height=576.0;self.overlay.dismiss_policy=OverlayDismissPolicy::ExplicitOnly;self.overlay.suspended=false;self.overlay.fullscreen=false;
                f.overlay(500,&mut self.overlay,panel,false,|f,_open| {
                    let b=f.available();let theme=f.theme();
                    f.paint_rect(Rect{x:b.x+(b.width-32.0)*0.5,y:b.y+12.0,width:32.0,height:4.0},2.0,theme.on_surface_variant);
                    f.styled_text(Rect{x:b.x+24.0,y:b.y+28.0,width:(b.width-48.0).max(0.0),height:36.0},"Your workspace",theme.text,TextStyle {size:22.0,..TextStyle::default()});
                    let scroll=f.bottom_sheet_scroll_offset();
                    let area=Rect{x:b.x,y:b.y+72.0,width:b.width,height:(b.height-72.0).max(0.0)};
                    f.within(area,|f| {
                        for i in 0..12 {
                            let rect=Rect{x:b.x+16.0,y:area.y+i as f32*48.0-scroll,width:(b.width-32.0).max(0.0),height:48.0};
                            f.action_at(700+i as u64,rect,&format!("Document {:02}",i+1),Options {variant:ButtonVariant::Text,..Options::default()});
                        }
                    });
                });
            }
            "material-bottom-sheet" => {
                let viewport=f.viewport();
                f.without_input(self.overlay.open,|f|{if filled(f,100,"Open bottom sheet").clicked {self.overlay.open=true;}f.label(&self.notice);});
                let height=260.0_f32.min(viewport.height);let width=640.0_f32.min(viewport.width);
                let panel=Rect{x:viewport.x+(viewport.width-width)*0.5,y:viewport.y+viewport.height-height,width,height};
                self.overlay.kind=OverlayKind::BottomSheet;self.overlay.style=OverlayStyle::Material3;self.overlay.drag_to_dismiss=true;self.overlay.bottom_sheet_peek=None;self.overlay.bottom_sheet_half_ratio=None;self.overlay.bottom_sheet_content_top=None;
                self.overlay.dismiss_policy=OverlayDismissPolicy::EscapeOrOutside;self.overlay.suspended=false;self.overlay.fullscreen=false;
                let mut result=None;
                f.overlay(500,&mut self.overlay,panel,true,|f,open| {
                    let b=f.available();let theme=f.theme();
                    f.paint_rect(Rect{x:b.x+(b.width-32.0)*0.5,y:b.y+12.0,width:32.0,height:4.0},2.0,theme.on_surface_variant);
                    f.styled_text(Rect{x:b.x+24.0,y:b.y+32.0,width:(b.width-48.0).max(0.0),height:36.0},"Save changes?",theme.text,TextStyle {size:24.0,..TextStyle::default()});
                    f.styled_text(Rect{x:b.x+24.0,y:b.y+84.0,width:(b.width-48.0).max(0.0),height:32.0},"Drag or flick the handle down to dismiss.",theme.on_surface_variant,TextStyle::default());
                    let button_y=b.y+b.height-56.0;
                    if f.action_at(601,Rect{x:b.x+b.width-176.0,y:button_y,width:72.0,height:40.0},"Cancel",Options {variant:ButtonVariant::Text,..Options::default()}).clicked {*open=false;result=Some("Cancelled");}
                    if f.action_at(602,Rect{x:b.x+b.width-96.0,y:button_y,width:72.0,height:40.0},"Save",Options {variant:ButtonVariant::Text,..Options::default()}).clicked {*open=false;result=Some("Saved");}
                });
                if let Some(message)=result {self.notice=message.into();}
            }
            "android-grid-layout" => {
                f.switch(70,"Right-to-left layout",&mut self.android_linear_example.rtl,true);
                if f.button(9901,"Change grid label").clicked {
                    if self.android_linear_example.text.contains_key("grid_wrap") {self.android_linear_example.text.remove("grid_wrap");self.android_linear_example.text.remove("nested_grid_label");}
                    else {self.android_linear_example.text.insert("grid_wrap".into(),"Longer label".into());self.android_linear_example.text.insert("nested_grid_label".into(),"Longer nested".into());}
                }
                f.label("Wrap-content grid columns");
                let bounds=f.reserve(56.0);
                android_grid_wrap::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                f.label("Nested wrap-content grid");
                let bounds=f.reserve(56.0);
                android_grid_nested_wrap::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                f.switch(71,"Vertical placement",&mut self.touched,true);
                let mut large_gap=self.mask&1!=0;
                if f.switch(73,"Larger theme gap",&mut large_gap,true).changed {if large_gap{self.mask|=1;}else{self.mask&=!1;}}
                self.android_linear_example.grid_default_gap=if large_gap {Some(24.0)}else{None};
                let mut shared_weights=self.mask&2!=0;
                if f.switch(74,"Shared-track weights",&mut shared_weights,true).changed {if shared_weights{self.mask|=2;}else{self.mask&=!2;}}
                let mut overlapping=self.mask&4!=0;
                if f.switch(75,"Overlapping spans",&mut overlapping,true).changed {if overlapping{self.mask|=4;}else{self.mask&=!4;}}
                let mut align_bounds=self.step!=0;
                if f.switch(72,"Align cell bounds",&mut align_bounds,true).changed {self.step=usize::from(align_bounds);}
                f.label(if overlapping {"Both spans share the middle column."} else if shared_weights && self.touched {"A tall item shares a row with a weighted item."} else if shared_weights {"A wide item shares a column with a weighted item."} else {"Two shares the column's margin when bounds are aligned."});
                // The imported grid has 12dp of padding on each horizontal side.
                let mut bounds=f.reserve(360.0);if overlapping {bounds.width=bounds.width.min(144.0);}else if shared_weights && self.touched {bounds.height=200.0;}
                static VERTICAL:bhoo::android_xml::Node=bhoo::android_xml::Node {horizontal:false,relative_rules:&[bhoo::android_xml::RelativeRule {kind:41,anchor:"2"},bhoo::android_xml::RelativeRule {kind:53,anchor:""}],..android_grid_layout::LAYOUT};
                static BOUNDS:bhoo::android_xml::Node=bhoo::android_xml::Node {relative_rules:&[bhoo::android_xml::RelativeRule {kind:40,anchor:"2"},bhoo::android_xml::RelativeRule {kind:53,anchor:""},bhoo::android_xml::RelativeRule {kind:54,anchor:""}],..android_grid_layout::LAYOUT};
                static VERTICAL_BOUNDS:bhoo::android_xml::Node=bhoo::android_xml::Node {horizontal:false,relative_rules:&[bhoo::android_xml::RelativeRule {kind:41,anchor:"2"},bhoo::android_xml::RelativeRule {kind:53,anchor:""},bhoo::android_xml::RelativeRule {kind:54,anchor:""}],..android_grid_layout::LAYOUT};
                static SHARED:bhoo::android_xml::Node=bhoo::android_xml::Node {horizontal:true,relative_rules:&[bhoo::android_xml::RelativeRule {kind:40,anchor:"2"}],children:&[
                    bhoo::android_xml::Node {id:"wide",text:"Wide",width:bhoo::android_xml::Size::Dp(100.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:42,anchor:"0"},bhoo::android_xml::RelativeRule {kind:43,anchor:"0"}],..android_grid_layout::LAYOUT.children[0]},
                    bhoo::android_xml::Node {id:"left",text:"Weighted 1",width:bhoo::android_xml::Size::Dp(20.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:42,anchor:"0"},bhoo::android_xml::RelativeRule {kind:43,anchor:"1"},bhoo::android_xml::RelativeRule {kind:46,anchor:"1"}],..android_grid_layout::LAYOUT.children[0]},
                    bhoo::android_xml::Node {id:"right",text:"Weighted 2",width:bhoo::android_xml::Size::Dp(20.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:42,anchor:"1"},bhoo::android_xml::RelativeRule {kind:43,anchor:"1"},bhoo::android_xml::RelativeRule {kind:46,anchor:"1"}],..android_grid_layout::LAYOUT.children[0]},
                ],..android_grid_layout::LAYOUT};
                static SHARED_ROWS:bhoo::android_xml::Node=bhoo::android_xml::Node {horizontal:true,height:bhoo::android_xml::Size::Dp(200.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:40,anchor:"2"}],children:&[
                    bhoo::android_xml::Node {id:"tall",text:"Tall",width:bhoo::android_xml::Size::Dp(100.0),height:bhoo::android_xml::Size::Dp(100.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:42,anchor:"0"},bhoo::android_xml::RelativeRule {kind:43,anchor:"0"},bhoo::android_xml::RelativeRule {kind:49,anchor:""}],..android_grid_layout::LAYOUT.children[0]},
                    bhoo::android_xml::Node {id:"top",text:"Weighted 1",width:bhoo::android_xml::Size::Dp(100.0),height:bhoo::android_xml::Size::Dp(20.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:42,anchor:"1"},bhoo::android_xml::RelativeRule {kind:43,anchor:"0"},bhoo::android_xml::RelativeRule {kind:47,anchor:"1"}],..android_grid_layout::LAYOUT.children[0]},
                    bhoo::android_xml::Node {id:"bottom",text:"Weighted 2",width:bhoo::android_xml::Size::Dp(100.0),height:bhoo::android_xml::Size::Dp(20.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:42,anchor:"1"},bhoo::android_xml::RelativeRule {kind:43,anchor:"1"},bhoo::android_xml::RelativeRule {kind:47,anchor:"1"}],..android_grid_layout::LAYOUT.children[0]},
                ],..android_grid_layout::LAYOUT};
                static SPANS:bhoo::android_xml::Node=bhoo::android_xml::Node {horizontal:true,relative_rules:&[bhoo::android_xml::RelativeRule {kind:40,anchor:"3"}],children:&[
                    bhoo::android_xml::Node {id:"one",text:"1",width:bhoo::android_xml::Size::Dp(20.0),relative_rules:&[],..android_grid_layout::LAYOUT.children[0]},
                    bhoo::android_xml::Node {id:"two",text:"2",width:bhoo::android_xml::Size::Dp(20.0),relative_rules:&[],..android_grid_layout::LAYOUT.children[0]},
                    bhoo::android_xml::Node {id:"three",text:"3",width:bhoo::android_xml::Size::Dp(20.0),relative_rules:&[],..android_grid_layout::LAYOUT.children[0]},
                    bhoo::android_xml::Node {id:"span_left",text:"Left span",width:bhoo::android_xml::Size::Dp(100.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:42,anchor:"0"},bhoo::android_xml::RelativeRule {kind:43,anchor:"1"},bhoo::android_xml::RelativeRule {kind:44,anchor:"2"}],..android_grid_layout::LAYOUT.children[0]},
                    bhoo::android_xml::Node {id:"span_right",text:"Right span",width:bhoo::android_xml::Size::Dp(100.0),relative_rules:&[bhoo::android_xml::RelativeRule {kind:42,anchor:"1"},bhoo::android_xml::RelativeRule {kind:43,anchor:"2"},bhoo::android_xml::RelativeRule {kind:44,anchor:"2"}],..android_grid_layout::LAYOUT.children[0]},
                ],..android_grid_layout::LAYOUT};
                let layout=if overlapping {&SPANS}else if shared_weights {if self.touched {&SHARED_ROWS}else{&SHARED}} else {match (self.touched,align_bounds) {(true,true)=>&VERTICAL_BOUNDS,(true,false)=>&VERTICAL,(false,true)=>&BOUNDS,_=>&android_grid_layout::LAYOUT}};
                let action=layout.render(f,bounds,&mut self.android_linear_example);
                if let Some(id)=action {self.notice=format!("Pressed: {id}");}f.label(&self.notice);
            }
            "android-two-line-list-item" => {
                f.switch(70,"Right-to-left layout",&mut self.android_linear_example.rtl,true);
                if f.button(71,"Change details").clicked {self.step=(self.step+1)%2;}
                self.android_linear_example.text.insert("android:text1".into(),["Alex Morgan","Sam Lee"][self.step%2].into());
                self.android_linear_example.text.insert("android:text2".into(),["Meeting at 10:30","Design review tomorrow"][self.step%2].into());
                let bounds=f.reserve(96.0);
                if let Some(id)=android_two_line_list_item::LAYOUT.render(f,bounds,&mut self.android_linear_example) {self.notice=format!("Pressed: {id}");}
                f.label(&self.notice);
            }
            "android-table-layout" => {
                f.switch(70, "Right-to-left layout", &mut self.android_linear_example.rtl, true);
                if f.button(9901,"Change table label").clicked {
                    if self.android_linear_example.text.contains_key("table_wrap") {self.android_linear_example.text.remove("table_wrap");self.android_linear_example.text.remove("nested_table_label");}
                    else {self.android_linear_example.text.insert("table_wrap".into(),"Longer label".into());self.android_linear_example.text.insert("nested_table_label".into(),"Longer nested".into());}
                }
                f.label("Wrap-content shared columns");
                let bounds=f.reserve(96.0);
                android_table_wrap::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                f.label("Nested wrap-content table");
                let bounds=f.reserve(56.0);
                android_table_nested_wrap::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                f.switch(71, "Collapse second column", &mut self.touched, true);
                android_table_layout::LAYOUT.set_table_column_collapsed(&mut self.android_linear_example,1,self.touched);
                let mut hide_row=self.android_linear_example.visibility.get("bob_row")==Some(&bhoo::android_xml::Visibility::Invisible);
                f.switch(73,"Hide shaded row",&mut hide_row,true);
                self.android_linear_example.visibility.insert("bob_row".into(),if hide_row {bhoo::android_xml::Visibility::Invisible} else {bhoo::android_xml::Visibility::Visible});
                f.label("The final row includes a cell that fills the row height.");
                let bounds=f.reserve(408.0);
                if let Some(id)=android_table_layout::LAYOUT.render(f,bounds,&mut self.android_linear_example) {self.notice=format!("Pressed: {id}");}
                f.label(&self.notice);
                let mut allow_overflow=self.step!=0;
                if f.switch(72,"Show content outside table",&mut allow_overflow,true).changed {self.step=usize::from(allow_overflow);}
                let space=f.reserve(56.0);
                let clip_bounds=Rect {width:120.0,..space};
                static OPEN:bhoo::android_xml::Node=bhoo::android_xml::Node {clip_children_disabled:true,clip_to_padding_disabled:true,..android_table_clip::LAYOUT};
                let clicked=if allow_overflow {OPEN.render(f,clip_bounds,&mut self.android_linear_example)} else {android_table_clip::LAYOUT.render(f,clip_bounds,&mut self.android_linear_example)};
                if let Some(id)=clicked {self.notice=format!("Pressed: {id}");}
                f.label("Standalone TableRow");
                let row_bounds=f.reserve(48.0);
                if let Some(id)=android_table_standalone::LAYOUT.render(f,row_bounds,&mut self.android_linear_example) {self.notice=format!("Pressed: {id}");}
            }
            "android-constraint-layout" => {
                f.switch(70, "Right-to-left layout", &mut self.android_linear_example.rtl, true);
                let mut m3 = matches!(self.android_linear_example.theme_style, bhoo::android_xml::AndroidTheme::Material3Light);
                if f.switch(71, "Material 3 Light theme", &mut m3, true).changed {
                    self.android_linear_example.theme_style = if m3 {
                        bhoo::android_xml::AndroidTheme::Material3Light
                    } else {
                        bhoo::android_xml::AndroidTheme::MaterialLight
                    };
                }
                let bounds = f.reserve(248.0);
                if let Some(id) = android_constraint_layout::LAYOUT.render(f, bounds, &mut self.android_linear_example) {
                    self.notice = format!("Pressed: {id}");
                }
                f.label(&self.notice);
            }
            "android-relative-layout" => {
                f.switch(70, "Right-to-left layout", &mut self.android_linear_example.rtl, true);
                if f.button(9901,"Change relative label").clicked {
                    if self.android_linear_example.text.contains_key("relative_wrap") {self.android_linear_example.text.remove("relative_wrap");self.android_linear_example.text.remove("nested_relative_label");}
                    else {self.android_linear_example.text.insert("relative_wrap".into(),"Longer label".into());self.android_linear_example.text.insert("nested_relative_label".into(),"Longer nested".into());}
                }
                f.label("Wrap-content sibling placement");
                let bounds=f.reserve(56.0);
                android_relative_wrap::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                f.label("Nested wrap-content relative layout");
                let bounds=f.reserve(56.0);
                android_relative_nested_wrap::LAYOUT.render(f,bounds,&mut self.android_linear_example);
                f.switch(71, "Hide title", &mut self.touched, true);
                for id in ["title", "edit"] {
                    self.android_linear_example.visibility.insert(id.into(), if self.touched { bhoo::android_xml::Visibility::Gone } else { bhoo::android_xml::Visibility::Visible });
                }
                let bounds=f.reserve(400.0);
                if let Some(id)=android_relative_layout::LAYOUT.render(f,bounds,&mut self.android_linear_example) { self.notice=format!("Pressed: {id}"); }
                f.label(&self.notice);
            }
            "android-absolute-layout" => {
                f.label("Android AbsoluteLayout");
                let bounds=f.reserve(280.0);
                android_absolute_layout::LAYOUT.render(f,bounds,&mut self.android_frame_example);
            }
            "android-frame-layout" => {
                f.label("FrameLayout measuring gone children");
                f.switch(70, "Right-to-left layout", &mut self.android_frame_example.rtl, true);
                if f.button(9901,"Change frame label").clicked {
                    if self.android_frame_example.text.contains_key("frame_wrap") {self.android_frame_example.text.remove("frame_wrap");}
                    else {self.android_frame_example.text.insert("frame_wrap".into(),"Longer label".into());}
                }
                f.label("Nested wrap-content frame");
                let bounds=f.reserve(56.0);
                android_frame_wrap::LAYOUT.render(f,bounds,&mut self.android_frame_example);
                let bounds = f.reserve(248.0);
                if let Some(id) = android_frame_layout::LAYOUT.render(f, bounds, &mut self.android_frame_example) { self.notice = format!("Pressed: {id}"); }
                f.label(&self.notice);
            }
            "android-button" => {
                f.label("Android XML button wrap-content");
                f.switch(70,"Right-to-left layout",&mut self.android_button_example.rtl,true);
                f.label("Single-line ellipsis: start, middle, end");
                let bounds=f.reserve(144.0);
                if let Some(id)=android_button_ellipsis::LAYOUT.render(f,bounds,&mut self.android_button_example) {self.notice=format!("Pressed: {id}");}
                if !self.notice.is_empty(){f.label(&self.notice);}
                f.label("Multiline wrap-content button");
                let bounds=f.reserve(112.0);
                if let Some(id)=android_button_multiline::LAYOUT.render(f,bounds,&mut self.android_button_example) {self.notice=format!("Pressed: {id}");}
                f.label("Button compound drawables");
                let bounds=f.reserve(112.0);
                if let Some(id)=android_button_drawables::LAYOUT.render(f,bounds,&mut self.android_button_example) {self.notice=format!("Pressed: {id}");}
                f.label("Wrap-content drawable height");
                let bounds=f.reserve(112.0);
                if let Some(id)=android_button_drawable_height::LAYOUT.render(f,bounds,&mut self.android_button_example) {self.notice=format!("Pressed: {id}");}
                if f.button(71,"Change button label").clicked {
                    if self.android_button_example.text.contains_key("wrap_button") {self.android_button_example.text.remove("wrap_button");}
                    else {self.android_button_example.text.insert("wrap_button".into(),"A much longer button".into());}
                }
                let bounds=f.reserve(104.0);
                if let Some(id)=android_button_wrap::LAYOUT.render(f,bounds,&mut self.android_button_example) {self.notice=format!("Pressed: {id}");}
            }
            "android-togglebutton" => {
                f.label("Android XML toggle buttons");
                f.switch(70,"Right-to-left layout",&mut self.android_toggle_example.rtl,true);
                f.label("Wrap-content state labels");
                let bounds=f.reserve(56.0);
                android_togglebutton_wrap::LAYOUT.render(f,bounds,&mut self.android_toggle_example);
                let bounds = f.reserve(124.0);
                android_togglebutton::LAYOUT.render(f, bounds, &mut self.android_toggle_example);
                f.label("Text style and disabled indicator opacity");
                let bounds=f.reserve(192.0);
                android_togglebutton_style::LAYOUT.render(f,bounds,&mut self.android_toggle_example);
            }
            "android-ratingbar" => {
                f.switch(70, "Right-to-left layout", &mut self.android_rating_example.rtl, true);
                let bounds = f.reserve(160.0);
                android_ratingbar::LAYOUT.render(f, bounds, &mut self.android_rating_example);
                let value = self.android_rating_example.ratings.get("rating").copied().unwrap_or(2.5);
                f.label(format!("Rating: {value:.1} / 5"));
                let bounds=f.reserve(160.0);
                android_ratingbar_tints::LAYOUT.render(f,bounds,&mut self.android_rating_example);
            }
            "android-radio-group" => {
                f.label("Choose a delivery option");
                f.switch(70, "Right-to-left layout", &mut self.android_radio_example.rtl, true);
                if f.button(9901,"Change radio label").clicked {
                    if self.android_radio_example.text.contains_key("radio_label") {self.android_radio_example.text.remove("radio_label");}
                    else {self.android_radio_example.text.insert("radio_label".into(),"Longer radio label".into());}
                }
                f.label("Runtime label and wrap content");
                let bounds=f.reserve(96.0);
                android_radio_label::LAYOUT.render(f,bounds,&mut self.android_radio_example);
                f.label("Nested initial choice");
                if f.button(71,"Reset nested choices").clicked {
                    self.android_radio_example.radio_groups.remove(&bhoo::android_xml::view_id("nested_group"));
                    self.android_radio_example.checked.remove("nested_choice");
                    self.android_radio_example.checked.remove("direct_choice");
                    self.android_radio_example.take_radio_group_events("nested_group");
                    self.radio_group_notice.clear();
                }
                if f.button(72,"Select nested by ID").clicked {self.android_radio_example.set_radio_group_checked("nested_group",Some("nested_choice"));}
                if f.button(73,"Select direct by ID").clicked {self.android_radio_example.set_radio_group_checked("nested_group",Some("direct_choice"));}
                if f.button(74,"Clear group selection").clicked {self.android_radio_example.set_radio_group_checked("nested_group",None);}
                let bounds=f.reserve(96.0);
                android_radio_group_nested::LAYOUT.render(f,bounds,&mut self.android_radio_example);
                if let Some(event)=self.android_radio_example.take_radio_group_events("nested_group").last() {
                    self.radio_group_notice=match event {
                        None=>"Group event: cleared",
                        Some(id) if *id==bhoo::android_xml::view_id("nested_choice")=>"Group event: nested choice",
                        Some(id) if *id==bhoo::android_xml::view_id("direct_choice")=>"Group event: direct choice",
                        _=>"Group event: other ID",
                    }.into();
                }
                if !self.radio_group_notice.is_empty(){f.label(&self.radio_group_notice);}
                f.label("Custom radio button tint opacity");
                let bounds=f.reserve(192.0);
                android_radio_group_tint_alpha::LAYOUT.render(f,bounds,&mut self.android_radio_example);
                let bounds = f.reserve(144.0);
                android_radio_group::LAYOUT.render(f, bounds, &mut self.android_radio_example);
                f.label("Android XML buttonTint");
                let bounds=f.reserve(144.0);
                android_radio_group_tints::LAYOUT.render(f,bounds,&mut self.android_radio_example);
                f.label("Android XML button drawable selector");
                let bounds=f.reserve(144.0);
                android_radio_group_drawable::LAYOUT.render(f,bounds,&mut self.android_radio_example);
            }
            "android-progressbar" => {
                f.switch(70, "Right-to-left layout", &mut self.android_progressbar_example.rtl, true);
                let bounds = f.reserve(144.0);
                android_progressbar::LAYOUT.render(f, bounds, &mut self.android_progressbar_example);
                if f.button(100, "Advance download").clicked {
                    let progress = self.android_progressbar_example.progress.entry("download".into()).or_insert(30.0);
                    *progress = if *progress >= 100.0 { 0.0 } else { (*progress + 20.0).min(100.0) };
                }
                let value = self.android_progressbar_example.progress.get("download").copied().unwrap_or(30.0);
                f.label(format!("{}%", value as i32));
                let mut selected=self.android_progressbar_example.view_selected.get("tinted_download").copied().unwrap_or(false);
                f.switch(71, "Selected tint state", &mut selected, true);
                self.android_progressbar_example.view_selected.insert("tinted_download".into(), selected);
                let bounds = f.reserve(144.0);
                android_progressbar_tints::LAYOUT.render(f, bounds, &mut self.android_progressbar_example);
            }
            "android-seekbar" => {
                f.switch(70, "Right-to-left layout", &mut self.android_seekbar_example.rtl, true);
                f.label("Custom thumb and tick tint modes");
                let bounds=f.reserve(456.0);
                android_seekbar_tint_modes::LAYOUT.render(f,bounds,&mut self.android_seekbar_example);
                f.label("Custom drawable tint opacity");
                let bounds=f.reserve(176.0);
                android_seekbar_tint_alpha::LAYOUT.render(f,bounds,&mut self.android_seekbar_example);
                let bounds = f.reserve(176.0);
                android_seekbar::LAYOUT.render(f, bounds, &mut self.android_seekbar_example);
                let value = self.android_seekbar_example.progress.get("volume").copied().unwrap_or(35.0);
                f.label(format!("Volume: {}", value as i32));
                let bounds=f.reserve(176.0);
                android_seekbar_tints::LAYOUT.render(f,bounds,&mut self.android_seekbar_example);
                let bounds=f.reserve(88.0);
                android_seekbar_ticks::LAYOUT.render(f,bounds,&mut self.android_seekbar_example);
                let bounds=f.reserve(176.0);
                android_seekbar_split::LAYOUT.render(f,bounds,&mut self.android_seekbar_example);
                let bounds=f.reserve(264.0);
                android_seekbar_thumb::LAYOUT.render(f,bounds,&mut self.android_seekbar_example);
                let bounds=f.reserve(176.0);
                android_seekbar_tick_selector::LAYOUT.render(f,bounds,&mut self.android_seekbar_example);
            }
            "android-switch" => {
                f.label("Android XML switches");
                f.switch(70, "Right-to-left layout", &mut self.android_switch_example.rtl, true);
                if f.button(9901,"Change switch label").clicked {
                    if self.android_switch_example.text.contains_key("wrap_switch") {self.android_switch_example.text.remove("wrap_switch");}
                    else {self.android_switch_example.text.insert("wrap_switch".into(),"Longer switch label".into());}
                }
                f.label("Android XML switch sizing and spacing");
                let bounds=f.reserve(156.0);
                android_switch_layout::LAYOUT.render(f,bounds,&mut self.android_switch_example);
                let bounds = f.reserve(156.0);
                android_switch::LAYOUT.render(f, bounds, &mut self.android_switch_example);
                f.label("Android XML thumbTint and trackTint");
                let bounds=f.reserve(156.0);
                android_switch_tints::LAYOUT.render(f,bounds,&mut self.android_switch_example);
                f.label("Android XML switch state text");
                let bounds=f.reserve(156.0);
                android_switch_text::LAYOUT.render(f,bounds,&mut self.android_switch_example);
            }
            "android-checkbox" => {
                f.label("Green: off · Purple: on · Red: press");
                f.switch(70, "Right-to-left layout", &mut self.android_checkbox_example.rtl, true);
                if f.button(9901,"Change checkbox label").clicked {
                    if self.android_checkbox_example.text.contains_key("checkbox_label") {self.android_checkbox_example.text.remove("checkbox_label");}
                    else {self.android_checkbox_example.text.insert("checkbox_label".into(),"Longer checkbox label".into());}
                }
                f.label("Runtime label and wrap content");
                let bounds=f.reserve(48.0);
                android_checkbox_label::LAYOUT.render(f,bounds,&mut self.android_checkbox_example);
                f.label("Custom button tint modes");
                let bounds=f.reserve(288.0);
                android_checkbox_tint_modes::LAYOUT.render(f,bounds,&mut self.android_checkbox_example);
                f.label("Custom button tint opacity");
                let bounds=f.reserve(96.0);
                android_checkbox_tint_alpha::LAYOUT.render(f,bounds,&mut self.android_checkbox_example);
                let bounds = f.reserve(144.0);
                android_checkbox::LAYOUT.render(f, bounds, &mut self.android_checkbox_example);
                f.label("Android XML buttonTint");
                let bounds=f.reserve(144.0);
                android_checkbox_tints::LAYOUT.render(f,bounds,&mut self.android_checkbox_example);
                f.label("Android XML button drawable selector");
                let bounds=f.reserve(144.0);
                android_checkbox_drawable::LAYOUT.render(f,bounds,&mut self.android_checkbox_example);
                f.label("XML text appearance on controls");
                let bounds=f.reserve(168.0);
                android_compound_textappearance::LAYOUT.render(f,bounds,&mut self.android_checkbox_example);
            }
            "android-horizontal-scroll" => {
                f.label("Two independent scroll views");
                f.switch(70,"Right-to-left layout",&mut self.horizontal_example.rtl,true);
                let bounds = f.reserve(192.0);
                if let Some(id) = android_horizontal_scroll::LAYOUT.render(f, bounds, &mut self.horizontal_example) {
                    self.notice = format!("Selected: {id}");
                }
                f.label(&self.notice);
            }
            "scrollbar" => {
                let mut indicator=self.mask&1!=0;
                if f.switch(91,"Non-interactive Material indicator",&mut indicator,true).changed {self.mask^=1;self.indicator_scroll=0.0;self.indicator_state=ScrollbarIndicatorState::default();}
                if indicator {
                    let mut horizontal=self.mask&2!=0;
                    if f.switch(92,"Horizontal",&mut horizontal,true).changed {self.mask^=2;f.cancel_scroll(95);self.indicator_scroll=0.0;self.indicator_state=ScrollbarIndicatorState::default();}
                    let mut rtl=self.mask&4!=0;if f.switch(93,"Right-to-left",&mut rtl,true).changed {self.mask^=4;f.cancel_scroll(95);}
                    let mut visible=self.mask&8!=0;if f.switch(94,"Always show indicator",&mut visible,true).changed {self.mask^=8;}
                    f.label("Swipe or scroll the content to reveal its indicator.");
                    let area=f.reserve(200.0);let content=1200.0;
                    let max=(content-if horizontal{area.width}else{area.height}).max(0.0);
                    if horizontal {f.scroll_viewport_horizontal_rtl(95,area,&mut self.indicator_scroll,max,rtl);}else{f.scroll_viewport(95,area,&mut self.indicator_scroll,max);}
                    f.paint_rect(area,8.0,f.theme().surface_variant);
                    f.within(area,|f|{
                        let count=if horizontal {12}else{24};
                        for i in 0..count {
                            let rect=if horizontal {Rect{x:if rtl {area.x+area.width-(i+1) as f32*100.0+self.indicator_scroll}else{area.x+i as f32*100.0-self.indicator_scroll},y:area.y,width:100.0,height:area.height}}
                                else {Rect{x:area.x,y:area.y+i as f32*50.0-self.indicator_scroll,width:area.width,height:50.0}};
                            if rect.intersect(area).width>0.0 && rect.intersect(area).height>0.0 {
                                f.styled_text(rect,&format!("Item {}",i+1),f.theme().text,TextStyle{padding:16.0,..TextStyle::default()});
                                if horizontal {f.paint_line([rect.x+rect.width-1.0,rect.y],[rect.x+rect.width-1.0,rect.y+rect.height],1.0,f.theme().outline_variant);}
                                else {f.paint_line([rect.x,rect.y+rect.height-1.0],[rect.x+rect.width,rect.y+rect.height-1.0],1.0,f.theme().outline_variant);}
                            }
                        }
                    });
                    f.scroll_indicator(area,content,self.indicator_scroll,if horizontal {ScrollbarOrientation::Horizontal}else{ScrollbarOrientation::Vertical},rtl,!visible,&mut self.indicator_state);
                    f.label(format!("Offset: {:.0}px",self.indicator_scroll));
                } else {
                f.label("Scroll the list, drag the thumb, or tap the track to move a page.");
                let b = f.reserve(280.0);
                let bar = Rect { x: b.x + b.width - 20.0, width: 20.0, ..b };
                let (bar_response, thumb) = f.vertical_scrollbar(120, bar, "Scroll position", 4000.0,
                    b.height, &mut self.grid.offset, &mut self.demo_scrollbar);
                if bar_response.pressed || bar_response.changed { f.cancel_scroll(110); }
                let content = Rect {
                    width: (b.width - 28.0).max(0.0),
                    ..b
                };
                f.within(content, |f| {
                    f.virtual_list(110, 280.0, 100, &mut self.grid, |f, i, s| {
                        f.list_item(1000 + i as u64, &format!("Row {}", i + 1), s, true)
                    });
                });
                f.paint_rect(
                    Rect {
                        x: bar.x + 7.0,
                        width: 6.0,
                        ..bar
                    },
                    3.0,
                    f.theme().control,
                );
                f.paint_rect(
                    Rect {
                        x: bar.x + 7.0,
                        y: bar.y + self.grid.offset / (4000.0 - b.height).max(1.0) * (bar.height - thumb.height),
                        width: 6.0,
                        height: thumb.height,
                    },
                    3.0,
                    f.theme().outline,
                );
                f.label(format!("Offset: {:.0}px", self.grid.offset));
                }
            }
            "refresh-indicator" => {
                if f.button(100, "Refresh content").clicked {
                    self.open = true;
                    self.started = Some(f.time());
                }
                if self.open {
                    let end = self.started.unwrap_or(f.time()) + 2.0;
                    if f.time() >= end {
                        self.open = false;
                        self.step += 1;
                    } else {
                        f.wake_at(end);
                    }
                }
                f.label("Scroll the list; pull down at the top to refresh.");
                let area = f.reserve(220.0);
                f.paint_rect(area, 12.0, f.theme().surface_variant);
                if f.pull_refresh_scroll(111, area, &mut self.grid.offset, 480.0, &mut self.pull_refresh, self.open, |f, offset| {
                    for i in 0..8 {
                        let row = Rect { x:area.x + 16.0, y:area.y + i as f32 * 60.0 - offset, width:area.width - 32.0, height:60.0 };
                        f.paint_text(row, &format!("Item {}", i + 1), f.theme().text);
                        f.paint_line([row.x, row.y + 59.0], [row.x + row.width, row.y + 59.0], 1.0, f.theme().outline_variant);
                    }
                }) {
                    self.open = true;
                    self.started = Some(f.time());
                }
                f.label(format!("Refreshes completed: {}", self.step));
            }
            "icon" => {
                f.label("Google Material Symbols Outlined · bundled vectors");
                for icons in MaterialIcon::ALL.chunks(4) {
                    f.row(56.0, icons.len(), |f, i| {
                        let b = f.reserve(48.0);
                        f.paint_material_icon(b.inset(12.0), icons[i], f.theme().primary);
                    });
                }
                f.label("Official 24dp icons. No font download or network required.");
            }
            "surface-and-elevation" => {
                for i in 0..4u8 {
                    let b = f.reserve(70.0);
                    f.surface(b, 12.0, i);
                    f.paint_text(b, &format!("Surface level {i}"), f.theme().text);
                }
            }
            "state-layer-ripple" => {
                f.label("Hover, focus and press feedback");
                if filled(f, 100, "Press and hold").clicked {
                    self.step += 1;
                }
                f.label(format!("Activations: {}", self.step));
                f.label("Pointer-origin ink • hold to expand • release to fade.");
            }
            "theme-tokens" => {
                let t = f.theme();
                for (name, color) in [
                    ("Primary", t.primary),
                    ("Secondary", t.secondary),
                    ("Surface", t.background),
                    ("Error", t.error),
                    ("Outline", t.outline),
                ] {
                    let b = f.reserve(48.0);
                    f.paint_rect(b, 8.0, color);
                    f.paint_text(
                        b,
                        name,
                        if name == "Primary" || name == "Error" {
                            t.on_primary
                        } else {
                            t.text
                        },
                    );
                }
                f.label("Use the page theme switch to compare roles.");
            }
            "virtualized-grid" => {
                f.label("100,000 tiles — only visible rows are built");
                f.virtual_grid(
                    100,
                    300.0,
                    3,
                    100_000,
                    &mut self.grid,
                    |f, index, selected| {
                        f.list_item(
                            1000 + index as u64,
                            &format!("App {}", index + 1),
                            selected,
                            true,
                        )
                    },
                );
                f.label(format!("Row offset: {:.0}px", self.grid.offset));
            }
            "shortcut-hint" => {
                f.label("Keyboard shortcuts");
                f.shortcut_hint(&["Ctrl", "K"], "Command palette");
                f.shortcut_hint(&["Ctrl", "S"], "Save");
                f.shortcut_hint(&["Esc"], "Dismiss");
                f.shortcut_hint(&["Tab", "Shift+Tab"], "Move focus");
            }
            "split-pane-resizable-divider" => {
                f.slider(100, "Pane split", &mut self.value, 20.0..=80.0, 1.0, true);
                let mut ratio = (self.value / 100.0).clamp(0.2, 0.8);
                f.split_pane(
                    101,
                    240.0,
                    &mut ratio,
                    SplitPaneOptions::default(),
                    |f| {
                        let a = f.available();
                        f.paint_rect(a, 8.0, f.theme().secondary);
                    },
                    |f| {
                        let a = f.available();
                        f.paint_rect(a, 8.0, f.theme().control);
                    },
                );
                self.value = (ratio * 100.0).round();
                f.label(format!("Leading pane: {:.0}%", self.value));
            }
            "breadcrumb" => {
                if let Some(i) = f.breadcrumb(100, &["Home", "Projects", "Bhoo"], Some(self.selected)) {
                    self.selected = i;
                }
                f.label(["Home directory", "Projects directory", "Bhoo project"][self.selected.min(2)]);
            }
            "numeric-field-spin-box" => {
                f.label("Quantity — 0 to 100");
                f.spin_box(100, "Quantity", &mut self.value, 0.0..=100.0, 1.0, true);
                f.slider(110, "Quantity", &mut self.value, 0.0..=100.0, 1.0, true);
            }
            "color-picker" => {
                f.label("Hue — drag or use keyboard arrows");
                let color = f.color_picker(100, "Hue", &mut self.value, true);
                f.label(format!("RGB {}, {}, {}", color.0, color.1, color.2));
                f.label("Hue-only study; alpha and full color-space controls remain open.");
            }
            "file-chooser-integration" => {
                f.label("Choose a local file using your browser");
                let b = f.reserve(48.0);
                action(f, 901, b, "Choose file…", false);
                f.label(if self.text.is_empty() {
                    "No file selected"
                } else {
                    &self.text
                });
                f.label("Only the filename is read; contents are not uploaded.");
            }
            "empty-error-state" => {
                f.tabs(
                    100,
                    &["Empty", "Error", "Recovered"],
                    &mut self.selected,
                    true,
                );
                let idx = self.selected.min(2);
                let title = ["No results found", "Could not load content", "Content is available"][idx];
                let msg = ["Try a different search.", "Check your connection, then retry.", "You can continue working."][idx];
                let action_label = match idx {
                    0 => Some("Clear filters"),
                    1 => Some("Retry"),
                    _ => None,
                };
                if let Some(r) = f.empty_state(
                    110,
                    title,
                    msg,
                    EmptyStateOptions {
                        icon: Some(match idx {
                            0 => MaterialIcon::Search,
                            1 => MaterialIcon::Close,
                            _ => MaterialIcon::Check,
                        }),
                        action_label,
                        error: idx == 1,
                    },
                ) && r.clicked
                {
                    self.selected = 2;
                }
            }
            _ => unreachable!("missing gallery for {slug}"),
        }
    }
    fn text_demo(&mut self, f: &mut Frame<'_>, slug: &str) {
        let search = matches!(slug, "search" | "command-palette");
        if !search {
            if slug == "text-field" {
                f.label("Text field");
            } else if slug == "multiline-text-field" {
                f.label("Multiline text field");
            }
            let label = match slug {
                "multiline-text-field" => "Notes",
                "autocomplete" => "Search applications",
                "form-field-container" => "Required name",
                _ => "Name",
            };
            let invalid =
                slug == "form-field-container" && self.text.trim().is_empty() && self.touched;
            let response = f.text_field_required(
                900,
                label,
                if slug == "multiline-text-field" {
                    104.0
                } else {
                    56.0
                },
                !self.text.is_empty(),
                invalid,
                false,
                slug == "form-field-container",
            );
            if slug == "form-field-container" {
                let b = Rect {
                    x: response.bounds.x + 16.0,
                    y: response.bounds.y + 56.0,
                    width: response.bounds.width - 32.0,
                    height: 20.0,
                };
                if invalid != self.subscript_invalid {
                    self.subscript_invalid = invalid;
                    self.subscript_started = Some(f.time());
                }
                let elapsed = self
                    .subscript_started
                    .map(|start| (f.time() - start).max(0.0))
                    .unwrap_or(0.3);
                let progress = if f.reduced_motion() {
                    1.0
                } else {
                    bhoo::timing::Easing::Cubic([0.55, 0.0, 0.55, 0.2])
                        .sample((elapsed / 0.3).min(1.0) as f32)
                };
                if progress < 1.0 {
                    f.continuous(b);
                } else {
                    self.subscript_started = None;
                }
                f.with_opacity(progress, |f| {
                    f.styled_text(
                        Rect {
                            y: b.y - 5.0 * (1.0 - progress),
                            ..b
                        },
                        if invalid {
                            "Name is required"
                        } else {
                            "Enter your name"
                        },
                        if invalid {
                            f.theme().error
                        } else {
                            f.theme().on_surface_variant
                        },
                        TextStyle {
                            size: 12.0,
                            ..TextStyle::default()
                        },
                    )
                });
            }
            if matches!(slug, "text-field" | "multiline-text-field") {
                f.label(format!("{} characters", self.text.chars().count()));
            }
            if slug == "autocomplete" && response.focused && !f.pressed_key(Key::Escape) {
                let b = Rect {
                    x: response.bounds.x,
                    y: response.bounds.y + 56.0,
                    width: response.bounds.width,
                    height: 240.0,
                };
                f.elevation(b, 4.0, 2);
                f.paint_rect(b, 4.0, f.theme().control);
                f.within(b, |f| {
                    f.with_gap(0.0, |f| {
                        for (i, label) in ["Bhoo", "Browser", "Files", "Settings", "Terminal"]
                            .iter()
                            .enumerate()
                        {
                            if label.to_lowercase().contains(&self.text.to_lowercase())
                                && f.list_item(100 + i as u64, label, false, true).clicked
                            {
                                self.text = (*label).into();
                            }
                        }
                    })
                });
            }
            return;
        }
        let items = [
            CommandItem { label: "Bhoo", shortcut: Some("Ctrl+1"), icon: Some(MaterialIcon::Widgets) },
            CommandItem { label: "Browser", shortcut: Some("Ctrl+2"), icon: Some(MaterialIcon::Search) },
            CommandItem { label: "Files", shortcut: Some("Ctrl+3"), icon: Some(MaterialIcon::Description) },
            CommandItem { label: "Settings", shortcut: Some("Ctrl+,"), icon: Some(MaterialIcon::Settings) },
            CommandItem { label: "Terminal", shortcut: Some("Ctrl+T"), icon: Some(MaterialIcon::Code) },
        ];
        if let Some(i) = f.command_palette_inline(900, &mut self.text, &items, "Search applications") {
            self.notice = format!("Opened {}", items[i].label);
        }
        f.label(&self.notice);
    }
    fn menu(&mut self, f: &mut Frame<'_>, labels: &[&str], rich: bool, selected: Option<usize>) {
        if !self.overlay.open {
            self.menu_typeahead.reset();
        }
        let root = f.available();
        let anchor = if self.anchor.height > 0.0 {
            self.anchor
        } else {
            Rect {
                y: (root.y - 52.0).max(24.0),
                height: 40.0,
                ..root
            }
        };
        let width = labels
            .iter()
            .map(|label| f.text_width(label, TextStyle::LABEL) + if selected.is_some() {72.0} else {24.0})
            .fold(112.0_f32, f32::max)
            .min(280.0)
            .min(root.width);
        let height = labels.len() as f32 * 48.0 + 16.0;
        let b = Rect {
            x: anchor.x,
            y: (anchor.y + anchor.height).min((f.viewport().height - height - 8.0).max(0.0)),
            width: if rich { root.width.min(320.0) } else { width },
            height: if rich { 240.0 } else { height },
        };
        self.overlay.style = if rich {
            OverlayStyle::Material3
        } else {
            OverlayStyle::AngularMaterial
        };
        if self.overlay.open
            && f.focused_id()
                .is_some_and(|id| (600..600 + labels.len() as u64).contains(&id))
        {
            let mut active = f.focused_id().map_or(0, |id| (id - 600) as usize);
            let items: Vec<_> = labels.iter().map(|label| (*label, true)).collect();
            if let Some(index) = f.menu_typeahead(&mut self.menu_typeahead, 600, &items) {
                active = index;
            }
            if f.pressed_key(Key::Down) {
                active = (active + 1) % labels.len();
                f.request_focus(600 + active as u64);
            }
            if f.pressed_key(Key::Up) {
                active = (active + labels.len() - 1) % labels.len();
                f.request_focus(600 + active as u64);
            }
            if f.pressed_key(Key::Home) {
                f.request_focus(600);
            }
            if f.pressed_key(Key::End) {
                f.request_focus(600 + (labels.len() - 1) as u64);
            }
        }
        self.overlay.kind = if rich {
            OverlayKind::Popover
        } else {
            OverlayKind::Menu
        };
        let mut chosen = None;
        f.overlay(500, &mut self.overlay, b, false, |f, open| {
            if rich {
                f.label("Quick settings");
            }
            f.with_gap(0.0, |f| {
                for (i, label) in labels.iter().enumerate() {
                    f.overlay_item(i, labels.len(), |f| {
                        let response = if rich {
                            f.list_item(600 + i as u64, label, false, true)
                        } else if let Some(selected)=selected {
                            f.menu_item_with_selection(600 + i as u64, label, true, selected==i, false)
                        } else {
                            f.menu_item(600 + i as u64, label, true)
                        };
                        if response.clicked {
                            chosen = Some(i);
                            *open = false;
                        }
                    });
                }
            });
        });
        if let Some(i) = chosen {
            self.selected = i;
            self.notice = format!("Selected: {}", labels[i]);
        }
    }
    fn modal(&mut self, f: &mut Frame<'_>, slug: &str) {
        let viewport = f.viewport();
        f.without_input(self.overlay.open, |f| {
            if filled(f, 100, "Open surface").clicked {
                self.overlay.open = true;
            }
            f.label(&self.notice);
            if slug == "dialog" {
                let mut required = self.mask & 512 != 0;
                if f.switch(101, "Require explicit choice", &mut required, true).changed {
                    self.mask ^= 512;
                }
                let mut fullscreen = self.mask & 1024 != 0;
                if f.switch(102, "Full-screen dialog", &mut fullscreen, true).changed {
                    self.mask ^= 1024;
                }
            }
        });
        let sheet = slug == "bottom-sheet";
        let side = slug == "side-sheet";
        let width = if sheet {
            viewport.width
        } else if side {
            viewport.width * 0.75
        } else {
            400.0_f32.min((viewport.width - 32.0).max(0.0))
        };
        let height = if sheet {
            199.8125
        } else if side {
            viewport.height
        } else {
            195.0
        };
        let panel = Rect {
            x: if sheet {
                0.0
            } else if side {
                viewport.width - width
            } else {
                (viewport.width - width) / 2.0
            },
            y: if sheet {
                viewport.height - height
            } else if side {
                0.0
            } else {
                (viewport.height - height) / 2.0
            },
            width,
            height,
        };
        self.overlay.kind = if sheet {
            OverlayKind::BottomSheet
        } else if side {
            OverlayKind::SideSheet
        } else {
            OverlayKind::Dialog
        };
        self.overlay.style = OverlayStyle::AngularMaterial;
        self.overlay.drag_to_dismiss=false;self.overlay.bottom_sheet_peek=None;self.overlay.bottom_sheet_half_ratio=None;self.overlay.bottom_sheet_content_top=None;
        self.overlay.fullscreen = slug == "dialog" && self.mask & 1024 != 0;
        self.overlay.suspended = self.child_dialog.open;
        self.overlay.dismiss_policy = if slug == "dialog" && self.mask & 512 != 0 { OverlayDismissPolicy::ExplicitOnly } else { OverlayDismissPolicy::EscapeOrOutside };
        self.overlay.accessible_title = (self.overlay.kind == OverlayKind::Dialog).then(|| if self.overlay.fullscreen { "Edit workspace" } else { "Save changes?" }.into());
        self.overlay.accessible_description = (self.overlay.kind == OverlayKind::Dialog).then(|| if self.overlay.fullscreen { "Review your workspace and choose Close or Save." } else if self.overlay.dismiss_policy == OverlayDismissPolicy::ExplicitOnly { "Your workspace has unsaved changes. Choose Cancel or Save." } else { "Your workspace has unsaved changes. Escape or the scrim dismisses this surface." }.into());
        let explicit_choice = self.overlay.dismiss_policy == OverlayDismissPolicy::ExplicitOnly;
        let fullscreen = self.overlay.fullscreen;
        let mut result = None;
        f.overlay(500, &mut self.overlay, panel, true, |f, open| {
            let b = f.available();
            if fullscreen {
                let close = Rect { x: b.x + 4.0, y: b.y + 4.0, width: 48.0, height: 48.0 };
                let response = f.target(600, close, "Close", true, Role::Button);
                f.state_layer(response, 24.0, f.theme().text);
                f.focus_ring(response);
                f.paint_material_icon(Rect { x: close.x + 12.0, y: close.y + 12.0, width: 24.0, height: 24.0 }, MaterialIcon::Close, f.theme().text);
                if response.clicked { self.child_dialog.open = true; }
                f.styled_text(Rect { x: b.x + 72.0, y: b.y, width: (b.width - 160.0).max(0.0), height: 56.0 }, "Edit workspace", f.theme().text, TextStyle { size: 22.0, ..TextStyle::default() });
                if f.action_at(601, Rect { x: b.x + (b.width - 80.0).max(0.0), y: b.y + 8.0, width: 72.0_f32.min(b.width), height: 40.0 }, "Save", Options { variant: ButtonVariant::Text, ..Options::default() }).clicked { *open = false; result = Some(1); }
                f.paint_line([b.x, b.y + 56.0], [b.x + b.width, b.y + 56.0], 1.0, f.theme().outline_variant);
                f.styled_text(Rect { x: b.x + 24.0, y: b.y + 80.0, width: (b.width - 48.0).max(0.0), height: 40.0 }, "Review your workspace before saving.", f.theme().on_surface_variant, TextStyle::default());
                return;
            }
            f.styled_text(
                Rect {
                    x: b.x + if sheet { 16.0 } else { 24.0 },
                    y: b.y + if sheet { 27.90625 } else { 16.0 },
                    width: b.width - if sheet { 32.0 } else { 48.0 },
                    height: if sheet { 24.0 } else { 40.0 },
                },
                "Save changes?",
                f.theme().text,
                TextStyle {
                    size: 24.0,
                    weight: if sheet { 700 } else { 400 },
                    ..TextStyle::default()
                },
            );
            for (i, label) in [
                "Your workspace has unsaved changes.",
                if explicit_choice { "Choose Cancel or Save to close." } else { "Escape or the scrim dismisses this surface." },
            ]
            .iter()
            .enumerate()
            {
                f.styled_text(
                    Rect {
                        x: b.x + if sheet { 16.0 } else { 24.0 },
                        y: b.y
                            + if sheet {
                                71.8125 + i as f32 * 40.0
                            } else {
                                68.0 + i as f32 * 34.0
                            },
                        width: b.width - if sheet { 32.0 } else { 48.0 },
                        height: if sheet { 24.0 } else { 20.0 },
                    },
                    label,
                    if sheet {
                        f.theme().text
                    } else {
                        f.theme().on_surface_variant
                    },
                    TextStyle {
                        size: if sheet { 16.0 } else { 14.0 },
                        ..TextStyle::default()
                    },
                );
            }
            let save_width = 64.0;
            let cancel_width = (f.text_width("Cancel", TextStyle::LABEL) + 24.0).max(64.0);
            for (i, (label, width, x)) in [
                (
                    "Cancel",
                    cancel_width,
                    if sheet {
                        b.x + 16.0
                    } else {
                        b.x + b.width - 24.0 - save_width - 8.0 - cancel_width
                    },
                ),
                (
                    "Save",
                    save_width,
                    if sheet {
                        b.x + 16.0 + cancel_width
                    } else {
                        b.x + b.width - 24.0 - save_width
                    },
                ),
            ]
            .into_iter()
            .enumerate()
            {
                if f.action_at(
                    600 + i as u64,
                    Rect {
                        x,
                        y: b.y + b.height - if sheet { 48.0 } else { 56.0 },
                        width,
                        height: 40.0,
                    },
                    label,
                    Options {
                        variant: ButtonVariant::Text,
                        ..Options::default()
                    },
                )
                .clicked
                {
                    *open = false;
                    result = Some(i);
                }
            }
        });
        if fullscreen {
            self.child_dialog.kind = OverlayKind::Dialog;
            self.child_dialog.style = OverlayStyle::AngularMaterial;
            self.child_dialog.accessible_title = Some("Discard changes?".into());
            self.child_dialog.accessible_description = Some("Your unsaved changes will be lost.".into());
            let width = 360.0_f32.min((viewport.width - 32.0).max(0.0));
            let height = 168.0_f32.min((viewport.height - 32.0).max(0.0));
            let child_bounds = Rect { x: viewport.x + (viewport.width - width) / 2.0, y: viewport.y + (viewport.height - height) / 2.0, width, height };
            let mut discard = false;
            f.overlay(700, &mut self.child_dialog, child_bounds, true, |f, open| {
                let b = f.available();
                f.styled_text(Rect { x: b.x + 24.0, y: b.y + 16.0, width: (b.width - 48.0).max(0.0), height: 40.0 }, "Discard changes?", f.theme().text, TextStyle { size: 24.0, ..TextStyle::default() });
                f.styled_text(Rect { x: b.x + 24.0, y: b.y + 68.0, width: (b.width - 48.0).max(0.0), height: 20.0 }, "Your unsaved changes will be lost.", f.theme().on_surface_variant, TextStyle { size: 14.0, ..TextStyle::default() });
                if f.action_at(710, Rect { x: b.x + (b.width - 224.0).max(0.0), y: b.y + (b.height - 56.0).max(0.0), width: 128.0, height: 40.0 }, "Keep editing", Options { variant: ButtonVariant::Text, ..Options::default() }).clicked { *open = false; }
                if f.action_at(711, Rect { x: b.x + (b.width - 88.0).max(0.0), y: b.y + (b.height - 56.0).max(0.0), width: 72.0, height: 40.0 }, "Discard", Options { variant: ButtonVariant::Text, ..Options::default() }).clicked { *open = false; discard = true; }
            });
            if discard { self.overlay.open = false; result = Some(0); }
        } else { self.child_dialog.open = false; }
        if let Some(i) = result {
            self.notice = if i == 0 { "Cancelled" } else { "Saved" }.into();
        }
    }
    fn table(&mut self, f: &mut Frame<'_>) {
        let names = ["Bhoo", "Launcher", "SpectreOS"];
        let sizes = [86, 240, 120];
        let t = f.theme();
        let mut b = Rect::default();
        f.with_gap(0.0, |f| b = f.reserve(56.0));
        let column = b.width * 0.55;
        for (i, label) in ["Name", "Size"].iter().enumerate() {
            let cell = Rect {
                x: b.x + if i == 0 { 0.0 } else { column },
                width: if i == 0 { column } else { b.width - column },
                ..b
            };
            let r = f.target(100 + i as u64, cell, label, true, Role::Button);
            if r.clicked {
                if self.mask & 1 == 0 || self.page != i {
                    self.page = i;
                    self.mask |= 1;
                    self.open = false;
                } else if !self.open {
                    self.open = true;
                } else {
                    self.mask &= !1;
                    self.open = false;
                }
            }
            f.styled_text(
                cell,
                label,
                t.text,
                TextStyle {
                    size: 14.0,
                    weight: 500,
                    padding: 16.0,
                    letter_spacing: 0.1,
                    ..TextStyle::default()
                },
            );
            if self.mask & 1 != 0 && self.page == i {
                let x = cell.x + 22.0 + f.text_width(label, TextStyle::LABEL);
                let y = cell.y + 28.0;
                let dir = if self.open { 1.0 } else { -1.0 };
                f.paint_line([x, y - 5.0], [x, y + 5.0], 1.5, t.on_surface_variant);
                f.paint_line(
                    [x - 3.0, y + dir * 2.0],
                    [x, y + dir * 5.0],
                    1.5,
                    t.on_surface_variant,
                );
                f.paint_line(
                    [x, y + dir * 5.0],
                    [x + 3.0, y + dir * 2.0],
                    1.5,
                    t.on_surface_variant,
                );
            }
        }
        f.paint_rect(
            Rect {
                y: b.y + 55.0,
                height: 1.0,
                ..b
            },
            0.0,
            t.outline,
        );
        let mut rows = [0, 1, 2];
        if self.mask & 1 != 0 {
            rows.sort_by(|&a, &b| {
                let order = if self.page == 0 {
                    names[a].cmp(names[b])
                } else {
                    sizes[a].cmp(&sizes[b])
                };
                if self.open { order.reverse() } else { order }
            });
        }
        f.with_gap(0.0, |f| {
            for (position, index) in rows.into_iter().enumerate() {
                let b = f.reserve(52.0);
                for col in 0..2 {
                    let cell = Rect {
                        x: b.x + if col == 0 { 0.0 } else { column },
                        width: if col == 0 { column } else { b.width - column },
                        ..b
                    };
                    let label = if col == 0 {
                        names[index].to_owned()
                    } else {
                        sizes[index].to_string()
                    };
                    f.styled_text(
                        cell,
                        &label,
                        t.text,
                        TextStyle {
                            size: 14.0,
                            padding: 16.0,
                            letter_spacing: 0.25,
                            ..TextStyle::default()
                        },
                    );
                }
                if position < 2 {
                    f.paint_rect(
                        Rect {
                            y: b.y + 51.0,
                            height: 1.0,
                            ..b
                        },
                        0.0,
                        t.outline,
                    );
                }
            }
        });
    }
    fn calendar(&mut self, f: &mut Frame<'_>, range: bool) {
        const MONTHS: [&str; 12] = [
            "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
        ];
        const DAYS: [&str; 31] = [
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
            "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30",
            "31",
        ];
        let root = f.available();
        let t = f.theme();
        let width = (root.width - 16.0).max(0.0);
        let pitch = width / 7.0;
        let margin = width * 0.05;
        let control_y = root.y + 8.0 + margin;
        let control_x = root.x + 8.0 + width * (4.714_286 / 100.0) - 16.0;
        let right = root.x + root.width - (control_x - root.x);
        let period = format!("{} {}", MONTHS[(self.month - 1) as usize], self.year);
        let period_width = (f.text_width(&period, TextStyle::LABEL) + 40.0).max(64.0);
        let period_box = Rect {
            x: control_x + 8.0,
            y: control_y,
            width: period_width,
            height: 40.0,
        };
        let r = f.target(
            1100,
            period_box,
            "Choose year and month",
            true,
            Role::Button,
        );
        f.state_layer(r, 20.0, t.on_surface_variant);
        if r.clicked {
            self.page = usize::from(self.page == 0);
        }
        f.styled_text(
            period_box,
            &period,
            t.on_surface_variant,
            TextStyle {
                align: 0,
                padding: 12.0,
                ..TextStyle::LABEL
            },
        );
        let x = period_box.x + period_box.width - 16.0;
        let y = period_box.y + 20.0;
        for (i, half) in [4.0, 3.0, 2.0, 1.0].into_iter().enumerate() {
            f.paint_line(
                [x - half, y - 1.5 + i as f32],
                [x + half, y - 1.5 + i as f32],
                1.0,
                t.on_surface_variant,
            );
        }
        for i in 0..2 {
            let b = Rect {
                x: right - 80.0 + i as f32 * 40.0,
                y: control_y,
                width: 40.0,
                height: 40.0,
            };
            let r = f.target(
                1101 + i,
                b,
                if i == 0 {
                    "Previous month"
                } else {
                    "Next month"
                },
                true,
                Role::Button,
            );
            f.state_layer(r, 20.0, t.on_surface_variant);
            let dir = if i == 0 { -1.0 } else { 1.0 };
            let c = [b.x + 20.0, b.y + 20.0];
            f.paint_line(
                [c[0] - dir * 2.5, c[1] - 4.0],
                [c[0] + dir * 1.5, c[1]],
                2.0,
                t.on_surface_variant,
            );
            f.paint_line(
                [c[0] + dir * 1.5, c[1]],
                [c[0] - dir * 2.5, c[1] + 4.0],
                2.0,
                t.on_surface_variant,
            );
            if r.clicked {
                if self.page == 1 {
                    self.year += if i == 0 { -24 } else { 24 };
                } else if self.page == 2 {
                    self.year += if i == 0 { -1 } else { 1 };
                } else {
                    self.month += if i == 0 { -1 } else { 1 };
                    if self.month == 0 {
                        self.month = 12;
                        self.year -= 1;
                    }
                    if self.month == 13 {
                        self.month = 1;
                        self.year += 1;
                    }
                    self.calendar_focus =
                        self.calendar_focus.min(month_days(self.year, self.month));
                }
            }
        }
        let body_y = root.y + 8.0 + 2.0 * margin + 40.0;
        if self.page != 0 {
            let count = if self.page == 1 { 24 } else { 12 };
            let cols = 4;
            let row_height = 52.0;
            let start = self.year.div_euclid(24) * 24;
            for (i, month) in MONTHS.iter().cycle().take(count).enumerate() {
                let b = Rect {
                    x: root.x + 8.0 + (i % cols) as f32 * width / cols as f32,
                    y: body_y + (i / cols) as f32 * row_height,
                    width: width / cols as f32,
                    height: row_height,
                };
                let label = if self.page == 1 {
                    (start + i as i32).to_string()
                } else {
                    (*month).to_owned()
                };
                let r = f.target(1400 + i as u64, b, &label, true, Role::Button);
                f.state_layer(r, 26.0, t.on_surface_variant);
                f.styled_text(
                    b,
                    &label,
                    t.text,
                    TextStyle {
                        size: 14.0,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
                if r.clicked {
                    if self.page == 1 {
                        self.year = start + i as i32;
                        self.page = 2;
                    } else {
                        self.month = i as i32 + 1;
                        self.page = 0;
                    }
                }
            }
            f.reserve((body_y - root.y) + count.div_ceil(cols) as f32 * row_height);
            return;
        }
        for (i, label) in ["S", "M", "T", "W", "T", "F", "S"].iter().enumerate() {
            f.styled_text(
                Rect {
                    x: root.x + 8.0 + i as f32 * pitch,
                    y: body_y,
                    width: pitch,
                    height: 18.0,
                },
                label,
                t.on_surface_variant,
                TextStyle {
                    size: 14.0,
                    weight: 500,
                    align: 1,
                    ..TextStyle::default()
                },
            );
        }
        let days = month_days(self.year, self.month);
        let offset = (weekday(self.year, self.month, 1) + 1) % 7;
        let extra = usize::from(offset == 0);
        let grid_y = body_y + 32.0;
        let month_label = Rect {
            x: root.x + 8.0 + 18.0,
            y: grid_y,
            width: if offset == 0 {
                width - 18.0
            } else {
                pitch * offset as f32 - 18.0
            },
            height: pitch,
        };
        f.styled_text(
            month_label,
            MONTHS[(self.month - 1) as usize],
            t.text,
            TextStyle {
                size: 14.0,
                weight: 500,
                ..TextStyle::default()
            },
        );
        // Angular previews an incomplete range as soon as the pointer enters a
        // valid end-date cell. Derive it before painting so earlier cells in the
        // same frame receive the outline as well.
        let preview_end = if range && self.step == 1 && self.mask & 1 != 0 {
            (1..=days).find_map(|day| {
                let index = offset + day - 1;
                let cell = Rect {
                    x: root.x + 8.0 + (index % 7) as f32 * pitch,
                    y: grid_y + (extra + index / 7) as f32 * pitch,
                    width: pitch,
                    height: pitch,
                };
                let date = [self.year, self.month, day as i32];
                (date > self.selected_date && f.pointer_in(cell)).then_some(date)
            })
        } else {
            None
        };
        for day in 1..=days {
            let index = offset + day - 1;
            let cell = Rect {
                x: root.x + 8.0 + (index % 7) as f32 * pitch,
                y: grid_y + (extra + index / 7) as f32 * pitch,
                width: pitch,
                height: pitch,
            };
            let b = cell.inset(pitch * 0.05);
            let date = [self.year, self.month, day as i32];
            let selected = if range {
                self.mask & 1 != 0
                    && (date == self.selected_date || date == self.range_end_date && self.step == 0)
            } else {
                date == self.selected_date
            };
            let r = f.target_roving(
                1200 + day as u64,
                b,
                DAYS[day - 1],
                true,
                day == self.calendar_focus,
            );
            if let Some(end) = preview_end
                && date >= self.selected_date
                && date <= end
            {
                paint_range_preview(
                    f,
                    cell,
                    b,
                    date == self.selected_date,
                    date == end,
                    t.primary,
                );
            }
            if range
                && self.mask & 1 != 0
                && self.step == 0
                && date > self.selected_date
                && date < self.range_end_date
            {
                f.paint_rect(
                    Rect {
                        y: b.y,
                        height: b.height,
                        ..cell
                    },
                    0.0,
                    t.primary_container,
                );
            }
            if selected && range && self.step == 0 && self.selected_date != self.range_end_date {
                let left = if date == self.selected_date {
                    b.x + b.width / 2.0
                } else {
                    cell.x
                };
                let right = if date == self.range_end_date {
                    b.x + b.width / 2.0
                } else {
                    cell.x + cell.width
                };
                f.paint_rect(
                    Rect {
                        x: left,
                        y: b.y,
                        width: right - left,
                        height: b.height,
                    },
                    0.0,
                    t.primary_container,
                );
            }
            if selected {
                f.paint_rect(
                    b,
                    b.width / 2.0,
                    if r.enabled {
                        t.primary
                    } else {
                        t.text.opacity(0.12)
                    },
                );
            } else if date == [2026, 11, 9] {
                f.paint_rect(b, b.width / 2.0, t.text);
                f.paint_rect(b.inset(1.0), (b.width / 2.0 - 1.0).max(0.0), t.background);
            }
            f.state_layer(
                r,
                b.width / 2.0,
                if selected {
                    t.on_primary
                } else {
                    t.on_surface_variant
                },
            );
            f.styled_text(
                b,
                DAYS[day - 1],
                if !r.enabled {
                    t.text.opacity(0.38)
                } else if selected {
                    t.on_primary
                } else {
                    t.text
                },
                TextStyle {
                    size: 14.0,
                    align: 1,
                    ..TextStyle::default()
                },
            );
            if r.clicked {
                self.calendar_focus = day;
                self.day = day;
                if range {
                    if self.step == 1 && date >= self.selected_date {
                        self.range_end_date = date;
                        self.end_day = day;
                        self.step = 0;
                        self.text = format!(
                            "{}/{}/{} – {}/{}/{}",
                            self.selected_date[1],
                            self.selected_date[2],
                            self.selected_date[0],
                            date[1],
                            date[2],
                            date[0]
                        );
                    } else {
                        self.selected_date = date;
                        self.range_end_date = date;
                        self.mask |= 1;
                        self.step = 1;
                        self.text = format!("{}/{}/{} –", date[1], date[2], date[0]);
                    }
                } else {
                    self.selected_date = date;
                }
            }
            if r.focused {
                let mut next = day as i32;
                if f.pressed_key(Key::Left) {
                    next -= 1;
                }
                if f.pressed_key(Key::Right) {
                    next += 1;
                }
                if f.pressed_key(Key::Up) {
                    next -= 7;
                }
                if f.pressed_key(Key::Down) {
                    next += 7;
                }
                if f.pressed_key(Key::Home) {
                    next = 1;
                }
                if f.pressed_key(Key::End) {
                    next = days as i32;
                }
                next = next.clamp(1, days as i32);
                if next != day as i32 {
                    self.calendar_focus = next as usize;
                    f.request_focus(1200 + next as u64);
                }
            }
        }
        f.reserve((grid_y - root.y) + (extra + (offset + days).div_ceil(7)) as f32 * pitch);
    }
}
// CSS expansion panels have independently rounded top/bottom corners.
// Split only the shadow's clip, so square joins retain continuous side shadows.
fn expansion_panel_shadow(f: &mut Frame<'_>, panel: Rect, round_top: bool, round_bottom: bool) {
    if round_top == round_bottom {
        f.elevation(panel, if round_top { 12.0 } else { 0.0 }, 2);
        return;
    }
    let half = panel.height * 0.5;
    let margin = 24.0;
    let top = Rect {
        x: panel.x - margin,
        y: panel.y - margin,
        width: panel.width + 2.0 * margin,
        height: half + margin,
    };
    let bottom = Rect {
        x: panel.x - margin,
        y: panel.y + half,
        width: panel.width + 2.0 * margin,
        height: panel.height - half + margin,
    };
    f.within(top, |f| {
        f.elevation(panel, if round_top { 12.0 } else { 0.0 }, 2)
    });
    f.within(bottom, |f| {
        f.elevation(panel, if round_bottom { 12.0 } else { 0.0 }, 2)
    });
}

fn sample_media_image() -> Image {
    let mut pixels = Vec::with_capacity(64 * 40 * 4);
    for y in 0..40u8 {
        for x in 0..64u8 {
            pixels.extend_from_slice(&[30 + x * 3, 80 + y * 3, 160, 255]);
        }
    }
    Image::rgba(64, 40, pixels).expect("valid example image")
}

fn paint_range_preview(
    f: &mut Frame<'_>,
    cell: Rect,
    inner: Rect,
    start: bool,
    end: bool,
    color: Color,
) {
    let x0 = if start {
        inner.x + inner.width * 0.5
    } else {
        cell.x
    };
    let x1 = if end {
        inner.x + inner.width * 0.5
    } else {
        cell.x + cell.width
    };
    // Chromium distributes 3px CSS border dashes evenly over each cell's
    // border, including a dash at either end. Rectangles avoid round line caps.
    let top = inner.y + 1.0;
    let bottom = inner.y + inner.height;
    let width = x1 - x0;
    let count = ((width + 2.0) / 5.0).floor().max(1.0) as usize;
    let spacing = if count > 1 {
        (width - 3.0) / (count - 1) as f32
    } else {
        0.0
    };
    for i in 0..count {
        for y in [top, bottom] {
            let x = if end && !start {
                if y == top {
                    x0 + 5.0 * i as f32
                } else {
                    x1 - 3.0 - 5.0 * i as f32
                }
            } else {
                x0 + spacing * i as f32
            };
            let left = x.max(x0);
            let right = (x + 3.0).min(x1);
            if right > left {
                f.paint_rect(
                    Rect {
                        x: left,
                        y,
                        width: right - left,
                        height: 1.0,
                    },
                    0.0,
                    color,
                );
            }
        }
    }
    if end && !start {
        f.paint_rect(
            Rect {
                x: x0,
                y: bottom,
                width: 2.0_f32.min(width),
                height: 1.0,
            },
            0.0,
            color,
        );
    }
    let radius = (inner.height - 1.0) * 0.5;
    let cy = inner.y + inner.height * 0.5 + 1.0;
    for (enabled, cx, base) in [
        (start, x0, std::f32::consts::FRAC_PI_2),
        (end, x1, -std::f32::consts::FRAC_PI_2),
    ] {
        if enabled {
            let length = std::f32::consts::PI * radius;
            let count = (length / 5.0).ceil().max(1.0) as usize;
            for i in 0..count {
                let angle = base + 5.0 * i as f32 / radius;
                f.paint_arc(
                    Rect {
                        x: cx - inner.width / 2.0,
                        y: cy - inner.height / 2.0,
                        width: inner.width,
                        height: inner.height,
                    },
                    angle,
                    (3.0 / radius).min(base + std::f32::consts::PI - angle),
                    1.0,
                    color,
                );
            }
        }
    }
}

fn filled(f: &mut Frame<'_>, id: u64, label: &str) -> Response {
    f.button_with(
        id,
        label,
        Options {
            variant: ButtonVariant::Filled,
            ..Options::default()
        },
    )
}
fn action(f: &mut Frame<'_>, id: u64, b: Rect, label: &str, selected: bool) -> Response {
    f.action_at(
        id,
        b,
        label,
        Options {
            selected,
            variant: ButtonVariant::Tonal,
            ..Options::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_registered_scene_builds_and_disables() {
        for scene in 19..19 + FAMILIES.lines().count() as u32 {
            let mut gallery = Gallery::new();
            let mut ui = Ui::default();
            let bounds = Rect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 640.0,
            };
            for disabled in [false, true] {
                ui.frame(Input::default(), bounds, 8.0, |f| {
                    f.with_enabled(!disabled, |f| gallery.show(f, scene))
                });
                if disabled {
                    assert!(ui.semantics().iter().all(|n| !n.enabled));
                }
            }
        }
    }
    #[test]
    fn calendar_dates_and_weekdays() {
        assert_eq!(month_days(2024, 2), 29);
        assert_eq!(month_days(2100, 2), 28);
        assert_eq!(weekday(2026, 11, 9), 0);
        assert_eq!(weekday(2026, 11, 1), 6);
    }
}

#[cfg(test)]
mod wavy_tests {
    use super::*;
    #[test]
    fn shape_gallery_animates() {
        let mut gallery = Gallery::new();
        let mut ui = Ui::default();
        let scene = 19
            + FAMILIES
                .lines()
                .position(|s| s == "expressive-shapes")
                .unwrap() as u32;
        let bounds = Rect {
            x: 24.0,
            y: 24.0,
            width: 850.0,
            height: 592.0,
        };
        let commands = ui.frame(Input::default(), bounds, 12.0, |f| gallery.show(f, scene));
        let shapes = commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::Shape { .. }))
            .count();
        assert!(ui.needs_redraw(), "should animate, shapes {shapes}");
    }
}
