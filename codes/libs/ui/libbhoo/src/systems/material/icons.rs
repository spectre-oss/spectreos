use crate::*;

/// Bundled 24 dp Material Icons for top app bar actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppBarIcon {
    Back,
    Forward,
    Folder,
    Tab,
    Search,
    More,
    Menu,
    Add,
}
impl AppBarIcon {
    pub(crate) fn shape_index(self, rtl: bool) -> u8 {
        let base = match self {
            Self::Back if rtl => 17,
            Self::Back => 16,
            Self::Forward if rtl => 16,
            Self::Forward => 17,
            Self::Folder => 3,
            Self::Tab => 21,
            Self::Search => 18,
            Self::More => 19,
            Self::Menu => 14,
            Self::Add => 20,
        };
        (crate::beercss_shapes::SHAPES.len() + base) as u8
    }
}

/// Official Material Symbols Outlined, regular weight at optical size 24.
/// Geometry is bundled; no platform font or network access is required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum MaterialIcon {
    Widgets,
    Code,
    Palette,
    Pause,
    DarkMode,
    LightMode,
    Download,
    Menu,
    MenuOpen,
    VerticalAlignTop,
    VerticalAlignBottom,
    Close,
    Check,
    Add,
    ArrowBack,
    ArrowForward,
    Search,
    MoreVert,
    Home,
    Settings,
    Inbox,
    Star,
    Archive,
    ChevronRight,
    Person,
    Delete,
    MarkEmailUnread,
    Mail,
    ChatBubble,
    Videocam,
    Edit,
    Sell,
    AttachFile,
    Send,
    Reply,
    MoreHoriz,
    ChevronLeft,
    Image,
    SentimentSatisfied,
    Description,
}
impl MaterialIcon {
    pub const ALL: &'static [Self] = &[
        Self::Widgets,
        Self::Code,
        Self::Palette,
        Self::Pause,
        Self::DarkMode,
        Self::LightMode,
        Self::Download,
        Self::Menu,
        Self::MenuOpen,
        Self::VerticalAlignTop,
        Self::VerticalAlignBottom,
        Self::Close,
        Self::Check,
        Self::Add,
        Self::ArrowBack,
        Self::ArrowForward,
        Self::Search,
        Self::MoreVert,
        Self::Home,
        Self::Settings,
        Self::Inbox,
        Self::Star,
        Self::Archive,
        Self::ChevronRight,
        Self::Person,
        Self::Delete,
        Self::MarkEmailUnread,
        Self::Mail,
        Self::ChatBubble,
        Self::Videocam,
        Self::Edit,
        Self::Sell,
        Self::AttachFile,
        Self::Send,
        Self::Reply,
        Self::MoreHoriz,
        Self::ChevronLeft,
        Self::Image,
        Self::SentimentSatisfied,
        Self::Description,
    ];
    pub fn name(self) -> &'static str {
        crate::icons::material_symbols::ICONS[self as usize].name
    }
}

impl Frame<'_> {
    /// Paint a bundled Material navigation icon without adding a focus target.
    pub fn paint_navigation_icon(
        &mut self,
        bounds: Rect,
        icon: NavigationIcon,
        selected: bool,
        color: Color,
    ) {
        self.commands.push(DrawCommand::Shape {
            bounds,
            index: icon.shape_index(selected),
            color,
            rotation: 0.0,
        });
    }
    /// Paint an official Material Symbol inside its original 24dp design box.
    /// The symbol is centered with its aspect ratio preserved within `bounds`.
    pub fn paint_material_icon(&mut self, bounds: Rect, icon: MaterialIcon, color: Color) {
        let size = bounds.width.min(bounds.height);
        if !size.is_finite() || size <= 0.0 { return; }
        let bounds = Rect { x: bounds.x + (bounds.width-size)/2.0,
            y: bounds.y + (bounds.height-size)/2.0, width: size, height: size };
        self.commands.push(DrawCommand::Shape {
            bounds,
            index: (crate::beercss_shapes::SHAPES.len() + crate::icons::material_navigation::ICONS.len() + icon as usize) as u8,
            color, rotation: 0.0,
        });
    }
}

#[cfg(test)]
#[path = "icons.spec.rs"]
mod tests;
