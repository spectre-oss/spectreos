use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertSeverity {
    Success,
    Info,
    Warning,
    Error,
}

/// Visual surface style for Material/MUI alerts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Standard,
    Filled,
    Outlined,
}

/// Configuration for `Frame::alert_with_options`.
#[derive(Clone, Copy, Debug)]
pub struct AlertOptions<'a> {
    pub variant: AlertVariant,
    pub action_label: Option<&'a str>,
    pub dismissible: bool,
}

impl Default for AlertOptions<'_> {
    fn default() -> Self {
        Self {
            variant: AlertVariant::Standard,
            action_label: None,
            dismissible: true,
        }
    }
}

impl Frame<'_> {
    /// Dismissible inline alert; application state owns whether it is shown.
    pub fn alert(&mut self, id: u64, message: &str, severity: AlertSeverity) -> Response {
        self.alert_with_options(id, message, severity, AlertOptions::default()).0
    }

    /// Inline alert supporting `Standard`, `Filled`, and `Outlined` variants, an optional
    /// custom trailing action button, and optional dismiss button.
    /// Returns `(dismiss_response, action_clicked)`.
    pub fn alert_with_options(
        &mut self,
        id: u64,
        message: &str,
        severity: AlertSeverity,
        options: AlertOptions<'_>,
    ) -> (Response, bool) {
        let b = self.reserve(48.0);
        let tone = match severity {
            AlertSeverity::Success => Color(46, 125, 50, 255),
            AlertSeverity::Info => Color(2, 136, 209, 255),
            AlertSeverity::Warning => Color(237, 108, 2, 255),
            AlertSeverity::Error => Color(211, 47, 47, 255),
        };
        let (icon_fg, text_fg) = match options.variant {
            AlertVariant::Standard => {
                self.paint_rect(b, 4.0, tone.opacity(0.10));
                (tone, self.ui.theme.text)
            }
            AlertVariant::Filled => {
                self.paint_rect(b, 4.0, tone);
                let on_tone = Color(255, 255, 255, 255);
                (on_tone, on_tone)
            }
            AlertVariant::Outlined => {
                self.paint_rect(b, 4.0, self.ui.theme.background);
                self.commands.push(DrawCommand::Outline {
                    bounds: b,
                    radius: 4.0,
                    width: 1.0,
                    color: tone,
                });
                (tone, self.ui.theme.text)
            }
        };
        self.styled_text(
            Rect { x: b.x + 16.0, width: 24.0, ..b },
            match severity {
                AlertSeverity::Success => "✓",
                AlertSeverity::Info => "i",
                AlertSeverity::Warning => "!",
                AlertSeverity::Error => "×",
            },
            icon_fg,
            TextStyle { size: 20.0, align: 1, ..TextStyle::default() },
        );

        let mut trailing_w = 12.0;
        let mut dismiss_resp = Response { id, bounds: b, enabled: self.enabled, ..Response::default() };
        if options.dismissible {
            let button = Rect {
                x: b.x + (b.width - 40.0).max(0.0),
                y: b.y + 4.0,
                width: 40.0,
                height: 40.0,
            };
            dismiss_resp = self.target(id, button, "Dismiss alert", true, Role::Button);
            self.state_layer(dismiss_resp, 20.0, text_fg);
            self.styled_text(
                button,
                "×",
                text_fg,
                TextStyle { size: 20.0, align: 1, ..TextStyle::default() },
            );
            trailing_w = 48.0;
        }
        let mut action_clicked = false;
        if let Some(action) = options.action_label.filter(|s| !s.is_empty()) {
            let act_w = (self.text_width(action, TextStyle::LABEL) + 20.0).clamp(52.0, (b.width * 0.4).max(52.0));
            let act_rect = Rect {
                x: (b.x + b.width - trailing_w - act_w).max(b.x + 52.0),
                y: b.y + 6.0,
                width: act_w,
                height: 36.0,
            };
            let ar = self.target(id.wrapping_add(1), act_rect, action, true, Role::Button);
            self.state_layer(ar, 18.0, text_fg);
            self.focus_ring(ar);
            self.styled_text(act_rect, action, if options.variant == AlertVariant::Filled { text_fg } else { tone }, TextStyle::LABEL);
            action_clicked = ar.clicked;
            trailing_w += act_w + 8.0;
        }
        self.styled_text(
            Rect { x: b.x + 52.0, width: (b.width - 52.0 - trailing_w).max(0.0), ..b },
            message,
            text_fg,
            TextStyle { size: 14.0, ..TextStyle::default() },
        );
        (dismiss_resp, action_clicked)
    }
}

#[cfg(test)]
#[path = "alert.spec.rs"]
mod tests;
