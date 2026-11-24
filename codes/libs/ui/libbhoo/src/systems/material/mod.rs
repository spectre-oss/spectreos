//! Initial controls using the shared input, motion, semantics and draw systems.
//! Reference behavior and remaining gaps: docs/implementation-progress.md.
use crate::*;

mod alert;
mod app_bar;
mod autocomplete;
mod avatar;
mod badge;
mod banner;
mod breadcrumb;
mod button;
mod button_group;
mod card;
mod carousel;
mod chat;
mod checkbox;
mod chip;
mod color_picker;
mod command_palette;
mod data_table;
mod date_picker;
mod divider;
mod empty_state;
mod expansion_panel;
mod fab;
mod fab_menu;
mod grid_list;
mod helpers;
mod icons;
mod layout;
mod link;
mod list;
pub mod loading;
mod menu;
mod navigation;
mod overlay;
mod pagination;
pub mod progress;
pub mod pull_refresh;
mod radio;
mod rating;
mod scaffold;
mod scrollbar;
mod search;
mod segmented;
mod select;
mod shortcut_hint;
mod skeleton;
mod slider;
pub mod snackbar;
mod spin_box;
mod split_button;
mod split_pane;
mod stepper;
mod switch;
mod tabs;
mod text_field;
mod tile;
mod time_picker;
mod toast;
mod toolbar;
mod tooltip;
mod transitions;
mod tree;
mod window_frame;

pub use alert::{AlertOptions, AlertSeverity, AlertVariant};
pub use app_bar::{
    AppBarAction, BottomAppBarOverflowState, BottomAppBarScrollBehavior, BottomAppBarState,
    FloatingPillSlot, TopAppBarMaterialAction, TopAppBarScrollBehavior, TopAppBarState,
    TopAppBarVariant,
};
pub use autocomplete::AutocompleteOptions;
pub use avatar::{AvatarContent, AvatarGroupCounts, AvatarGroupOptions, AvatarShape};
pub use badge::{BadgeContent, BadgeGravity, BadgeNumberFormat, BadgeNumberLimit, BadgeOptions};
pub use banner::BannerOptions;
pub use button::{ButtonVariant, Options};
pub use button_group::ButtonGroupState;
pub use card::{
    CardIconGravity, CardOptions, CardVariant, CardVisualState, SwipeDismissState,
    segmented_card_radii,
};
pub(crate) use card::CardAppearance;
pub use carousel::{CarouselItem, CarouselLayout, CarouselOptions, CarouselState};
pub use chat::ChatMessage;
pub use checkbox::CheckState;
pub use chip::{ChipKind, ChipOptions};
pub use color_picker::hue_to_color;
pub use command_palette::CommandItem;
pub use data_table::{SortDirection, TableColumn, TableSortState};
pub use date_picker::{DatePickerOptions, DatePickerState, DatePickerView, month_days, weekday};
pub use empty_state::EmptyStateOptions;
pub use expansion_panel::ExpansionPanelOptions;
pub use fab::{FabColor, FabElevation, FabOptions};
pub use fab_menu::{FabMenuAction, FabMenuOptions, FabMenuState};
pub use grid_list::{GridListItem, GridListOptions};
pub use icons::{AppBarIcon, MaterialIcon};
pub use list::ListState;
pub use menu::MenuTypeaheadState;
pub use navigation::{NavigationDestination, NavigationIcon};
pub use overlay::{OverlayDismissPolicy, OverlayKind, OverlaySection, OverlayState, OverlayStyle};
pub(crate) use overlay::OverlayPhase;
pub use pagination::{PageIndicatorOptions, PaginationOptions};
pub use radio::Choice;
pub use scaffold::{ScaffoldAdaptiveOptions, ScaffoldOptions, ScaffoldSlots};
pub use scrollbar::{ScrollbarDragState, ScrollbarIndicatorState, ScrollbarOrientation};
pub use search::SearchOverlayState;
pub use select::SelectState;
pub use skeleton::{SkeletonAnimation, SkeletonOptions};
pub use slider::SliderExpressiveOptions;
pub(crate) use slider::SliderTintColors;
pub use split_button::{SplitButtonOptions, SplitButtonSize};
pub use split_pane::{SplitPaneOptions, SplitPaneOrientation};
pub use stepper::StepperState;
pub use switch::SwitchIconMode;
pub(crate) use switch::{SwitchLayout, switch_widths};
pub use tabs::{TabItem, TabOptions, TabStyle};
pub use text_field::FieldAffordance;
pub use tile::ActionTileOptions;
pub use time_picker::{
    TimePickerMode, TimePickerOptions, TimePickerState, time_picker_label, time_picker_label_24h,
    time_picker_label_step,
};
pub use toast::{ToastOptions, ToastState};
pub use toolbar::{
    DockedToolbarEdge, DockedToolbarInsets, DockedToolbarScrollState,
    FloatingToolbarExpansionScrollState, ToolbarAction, ToolbarFabPosition, ToolbarFabSelection,
    ToolbarKind, ToolbarState,
};
pub use transitions::{
    PageDirection, RouteTransitionKind, RouteTransitionState, ShapeRotationState, SlideDirection,
};
pub use tree::{TreeItem, TreeState};
pub use window_frame::{WindowFrameAction, WindowFrameOptions, WindowFrameState};

impl Frame<'_> {
    pub(super) fn control_color(&mut self, response: Response, base: Color) -> Color {
        let h = self.animate(
            response.id,
            0,
            if response.hovered { 1.0 } else { 0.0 },
            0.0,
        );
        let p = self.animate(
            response.id,
            1,
            if response.pressed { 1.0 } else { 0.0 },
            0.0,
        );
        if response.enabled {
            base.mix(
                self.ui.theme.text,
                if response.pressed {
                    0.12 * p
                } else if response.focused && self.ui.keyboard_focus {
                    0.12
                } else {
                    0.08 * h
                },
            )
        } else {
            self.ui.theme.background.mix(self.ui.theme.text, 0.12)
        }
    }
    pub fn elevation(&mut self, b: Rect, radius: f32, level: u8) {
        let black = Color(0, 0, 0, 255);
        let specs = if level >= 6 {
            [
                (0.0, 3.0, 5.0, -1.0, 0.2),
                (0.0, 6.0, 10.0, 0.0, 0.14),
                (0.0, 1.0, 18.0, 0.0, 0.12),
            ]
        } else if level <= 1 {
            [
                (0.0, 2.0, 1.0, -1.0, 0.2),
                (0.0, 1.0, 1.0, 0.0, 0.14),
                (0.0, 1.0, 3.0, 0.0, 0.12),
            ]
        } else {
            [
                (0.0, 3.0, 1.0, -2.0, 0.2),
                (0.0, 2.0, 2.0, 0.0, 0.14),
                (0.0, 1.0, 5.0, 0.0, 0.12),
            ]
        };
        for (x, y, blur, spread, alpha) in specs {
            self.shadow(b, radius, [x, y], blur, spread, black.opacity(alpha));
        }
    }
    /// Approximate a continuous Android elevation in dp using the existing
    /// shadow profiles; actual platform shadows vary by renderer and device.
    pub(crate) fn elevation_dp(&mut self,b:Rect,radius:f32,dp:f32) {
        if !dp.is_finite() || dp<=0.0 {return;}
        const LOW:[[f32;5];3]=[[0.,2.,1.,-1.,0.2],[0.,1.,1.,0.,0.14],[0.,1.,3.,0.,0.12]];
        const MID:[[f32;5];3]=[[0.,3.,1.,-2.,0.2],[0.,2.,2.,0.,0.14],[0.,1.,5.,0.,0.12]];
        const HIGH:[[f32;5];3]=[[0.,3.,5.,-1.,0.2],[0.,6.,10.,0.,0.14],[0.,1.,18.,0.,0.12]];
        let (a,b_spec,t)=if dp<=1.0 {(LOW,LOW,0.0)} else if dp<=2.0 {(LOW,MID,dp-1.0)} else {(MID,HIGH,((dp-2.0)/4.0).min(1.0))};
        let scale=if dp>6.0 {(dp/6.0).sqrt().min(2.0)} else {1.0};
        for i in 0..3 {
            let p=std::array::from_fn::<_,5,_>(|j|a[i][j]+(b_spec[i][j]-a[i][j])*t);
            self.shadow(b,radius,[p[0]*scale,p[1]*scale],p[2]*scale,p[3]*scale,Color(0,0,0,255).opacity(p[4]*dp.min(1.0)));
        }
    }
    /// Fixed-size grouping surface; children are clipped and use the same stable ID namespace.
    pub fn panel(&mut self, height: f32, build: impl FnOnce(&mut Self)) {
        let b = self.allocate(height.max(0.0));
        self.rect(b, self.ui.theme.radius, self.ui.theme.control);
        self.within(b.inset(12.0), build);
    }
    pub fn within(&mut self, bounds: Rect, build: impl FnOnce(&mut Self)) {
        let previous = (self.bounds, self.clip, self.cursor);
        self.bounds = bounds;
        self.clip = self.clip.intersect(bounds);
        self.cursor = bounds.y;
        self.commands.push(DrawCommand::PushClip(self.clip));
        build(self);
        self.commands.push(DrawCommand::PopClip);
        (self.bounds, self.clip, self.cursor) = previous;
    }
    /// Lay out children within bounds while preserving the existing paint clip.
    /// This allows effects such as shadows to extend beyond the layout bounds.
    pub fn within_unclipped(&mut self, bounds: Rect, build: impl FnOnce(&mut Self)) {
        let previous = (self.bounds, self.cursor);
        self.bounds = bounds;
        self.cursor = bounds.y;
        build(self);
        (self.bounds, self.cursor) = previous;
    }
}
