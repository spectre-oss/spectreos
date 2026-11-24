use crate::*;

/// Badge appearance; the accessible description is supplied separately.
#[derive(Clone, Copy, Debug)]
pub enum BadgeContent<'a> {
    Dot,
    Label(&'a str),
    /// Shorten long text to at most `max_graphemes`, including an ellipsis.
    Text { text: &'a str, max_graphemes: usize },
    /// Show a number, or the configured maximum followed by `+` when exceeded.
    Number { value: u32, max: u32 },
    /// Format digits and optionally derive the overflow maximum from a character limit.
    FormattedNumber { value: u32, limit: BadgeNumberLimit, format: BadgeNumberFormat<'a> },
}
/// The numeric overflow threshold. A character limit reserves one character for the suffix.
#[derive(Clone, Copy, Debug)]
pub enum BadgeNumberLimit { Unlimited, Max(u32), MaxCharacters(u8) }
/// Caller-supplied number formatting; grouping is disabled when `group_size` is zero.
#[derive(Clone, Copy, Debug)]
pub struct BadgeNumberFormat<'a> {
    pub digits: [&'a str; 10],
    pub group_separator: &'a str,
    pub group_size: u8,
    pub overflow_suffix: &'a str,
}
impl Default for BadgeNumberFormat<'_> {
    fn default() -> Self { Self { digits: ["0","1","2","3","4","5","6","7","8","9"], group_separator: ",", group_size: 3, overflow_suffix: "+" } }
}
fn format_badge_number(value: u32, format: BadgeNumberFormat<'_>, grouping: bool) -> String {
    let ascii = value.to_string();
    let mut result = String::new();
    for (index, digit) in ascii.bytes().enumerate() {
        if grouping && format.group_size > 0 && index > 0 && (ascii.len()-index) % usize::from(format.group_size) == 0 { result.push_str(format.group_separator); }
        result.push_str(format.digits[usize::from(digit-b'0')]);
    }
    result
}
/// Logical badge corner relative to its icon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeGravity {
    TopStart,
    #[default]
    TopEnd,
    BottomStart,
    BottomEnd,
}
/// Badge placement offsets are in dp, positive toward the anchor center.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BadgeOptions {
    pub gravity: BadgeGravity,
    pub rtl: bool,
    pub horizontal_offset: f32,
    pub vertical_offset: f32,
}

impl Frame<'_> {
    pub fn badge(&mut self, label: &str) {
        let anchor = self.allocate(28.0);
        let style = TextStyle {
            size: 11.0,
            weight: 500,
            align: 1,
            ..TextStyle::default()
        };
        let b = Rect {
            x: anchor.x + 36.0,
            y: anchor.y - 4.0,
            width: (self.text_width(label, style) + 8.0).max(16.0),
            height: 16.0,
        };
        self.rect(b, 8.0, self.ui.theme.error);
        self.styled_text(b, label, self.ui.theme.on_error, style);
    }
    /// Overlay a Material badge on an icon without changing its layout or
    /// adding a focus target. Call after the owner so its semantics can receive
    /// the localized description; `rtl` mirrors the placement.
    pub fn badge_at(
        &mut self,
        owner: u64,
        icon: Rect,
        content: BadgeContent<'_>,
        description: &str,
        rtl: bool,
    ) -> bool {
        self.badge_at_gravity(owner,icon,content,description,BadgeGravity::TopEnd,rtl)
    }
    /// Overlay a badge at a logical corner; start and end mirror in RTL.
    pub fn badge_at_gravity(
        &mut self,
        owner: u64,
        icon: Rect,
        content: BadgeContent<'_>,
        description: &str,
        gravity: BadgeGravity,
        rtl: bool,
    ) -> bool {
        self.badge_at_with_options(owner,icon,content,description,BadgeOptions{gravity,rtl,..BadgeOptions::default()})
    }
    /// Overlay a badge with logical corner and offsets toward the icon center.
    pub fn badge_at_with_options(
        &mut self,
        owner: u64,
        icon: Rect,
        content: BadgeContent<'_>,
        description: &str,
        options: BadgeOptions,
    ) -> bool {
        if !icon.x.is_finite()
            || !icon.y.is_finite()
            || !icon.width.is_finite()
            || !icon.height.is_finite()
            || !options.horizontal_offset.is_finite()
            || !options.vertical_offset.is_finite()
            || icon.width <= 0.0
            || icon.height <= 0.0
        {
            return false;
        }
        let style = TextStyle {
            size: 11.0,
            weight: 500,
            align: 1,
            ..TextStyle::default()
        };
        let owned_label;
        let (width, height, x_offset, y_offset, label) = match content {
            BadgeContent::Dot => (6.0, 6.0, 6.0, 6.0, None),
            BadgeContent::Label(label) if !label.is_empty() && label.chars().count() <= 4 => (
                (self.text_width(label, style) + 8.0).clamp(16.0, 34.0),
                16.0,
                12.0,
                14.0,
                Some(label),
            ),
            BadgeContent::Label(_) => return false,
            BadgeContent::Text { text, max_graphemes } => {
                if text.is_empty() || max_graphemes==0 {return false;}
                let label=if text.grapheme_indices(true).nth(max_graphemes).is_some() {
                    let end=text.grapheme_indices(true).nth(max_graphemes-1).map_or(text.len(),|(index,_)|index);
                    owned_label=format!("{}…",&text[..end]);owned_label.as_str()
                } else {text};
                ((self.text_width(label,style)+8.0).max(16.0),16.0,12.0,14.0,Some(label))
            }
            BadgeContent::Number { value, max } => {
                owned_label = if value > max { format!("{max}+") } else { value.to_string() };
                let label=owned_label.as_str();
                ((self.text_width(label, style)+8.0).max(16.0),16.0,12.0,14.0,Some(label))
            }
            BadgeContent::FormattedNumber { value, limit, format } => {
                if format.digits.iter().any(|digit| digit.is_empty()) { return false; }
                let max = match limit {
                    BadgeNumberLimit::Unlimited => u32::MAX,
                    BadgeNumberLimit::Max(max) => max,
                    BadgeNumberLimit::MaxCharacters(0) => return false,
                    BadgeNumberLimit::MaxCharacters(chars) => 10u32.checked_pow(u32::from(chars)-1).map_or(u32::MAX, |n| n-1),
                };
                owned_label = if value > max { let mut label = format_badge_number(max, format, false); label.push_str(format.overflow_suffix); label } else { format_badge_number(value, format, true) };
                let label = owned_label.as_str();
                ((self.text_width(label, style)+8.0).max(16.0),16.0,12.0,14.0,Some(label))
            }
        };
        let start=matches!(options.gravity,BadgeGravity::TopStart|BadgeGravity::BottomStart);
        let left=start!=options.rtl;
        let bottom=matches!(options.gravity,BadgeGravity::BottomStart|BadgeGravity::BottomEnd);
        let x = if left {
            icon.x + x_offset - width
        } else {
            icon.x + icon.width - x_offset
        };
        let b = Rect {
            x: x+if left {options.horizontal_offset}else{-options.horizontal_offset},
            y: if bottom {icon.y+icon.height-y_offset-options.vertical_offset}else{icon.y-height+y_offset+options.vertical_offset},
            width,
            height,
        };
        self.rect(b, height / 2.0, self.ui.theme.error);
        if let Some(label) = label {
            self.styled_text(b, label, self.ui.theme.on_error, style);
        }
        if !description.is_empty()
            && let Some(node) = self
                .ui
                .semantics
                .iter_mut()
                .rev()
                .find(|node| node.id == owner)
        {
            node.label = format!("{}, {}", node.label, description).into();
        }
        true
    }
}

#[cfg(test)]
#[path = "badge.spec.rs"]
mod tests;
