use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Neutral,
    Filled,
    Tonal,
    Outlined,
    Text,
    Elevated,
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub enabled: bool,
    pub selected: bool,
    pub variant: ButtonVariant,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            enabled: true,
            selected: false,
            variant: ButtonVariant::Neutral,
        }
    }
}

impl Frame<'_> {
    pub fn button(&mut self, id: u64, text: impl AsRef<str>) -> Response {
        self.button_with(id, text, Options::default())
    }
    pub fn button_with(&mut self, id: u64, text: impl AsRef<str>, options: Options) -> Response {
        let b = self.allocate(40.0);
        self.button_at(id, b, text.as_ref(), options)
    }
    pub(crate) fn button_at(&mut self, id: u64, b: Rect, text: &str, options: Options) -> Response {
        let response = self.interact(id, b, options.enabled);
        let t = self.ui.theme;
        let radius = (b.height * 0.5).min(20.0);
        let transparent = matches!(
            options.variant,
            ButtonVariant::Text | ButtonVariant::Neutral | ButtonVariant::Outlined
        );
        let base = match options.variant {
            ButtonVariant::Filled => t.primary,
            ButtonVariant::Tonal => t.secondary,
            ButtonVariant::Elevated => t.background,
            _ => t.background,
        };
        let foreground = if !response.enabled {
            t.text.opacity(0.38)
        } else {
            match options.variant {
                ButtonVariant::Filled => t.on_primary,
                ButtonVariant::Tonal => t.on_secondary,
                _ => t.primary,
            }
        };
        if options.variant == ButtonVariant::Elevated && response.enabled {
            self.elevation(b, radius, if response.hovered { 2 } else { 1 });
        }
        if !transparent {
            self.rect(
                b,
                radius,
                if response.enabled {
                    base
                } else {
                    t.text.opacity(0.12)
                },
            );
        }
        if options.variant == ButtonVariant::Outlined {
            self.rect(
                b,
                radius,
                if response.enabled {
                    t.outline
                } else {
                    t.text.opacity(0.12)
                },
            );
            self.rect(b.inset(1.0), (radius - 1.0).max(0.0), base);
        }
        let hover = if self.feedback_blocked {
            0.0
        } else {
            self.animate(id, 0, f32::from(response.hovered), 0.0)
        };
        if response.enabled {
            let opacity = if response.focused && self.ui.keyboard_focus
                || response.pressed && self.ui.motion_preference == MotionPreference::Reduced
            {
                0.12
            } else {
                0.08 * hover
            };
            if opacity > 0.001 {
                self.rect(b, radius, foreground.opacity(opacity));
            }
        }
        if !self.feedback_blocked {
            self.ink(response, radius, foreground);
        }
        self.styled_text(b, text, foreground, TextStyle::LABEL);
        self.focus_ring(response);
        self.semantic(
            response,
            Role::Button,
            text,
            options.selected,
            None,
            None,
            None,
        );
        response
    }
    /// An accessible icon button using bundled official Material Symbols geometry.
    pub fn material_icon_button(&mut self, id:u64, icon:MaterialIcon, label:&str, options:Options) -> Response {
        let r = self.icon_button(id, "", label, options);
        let t = self.ui.theme;
        let color = if !r.enabled { t.text.opacity(0.38) } else {
            match options.variant {
                ButtonVariant::Filled => t.on_primary,
                ButtonVariant::Tonal => t.on_secondary,
                _ => if options.selected { t.primary } else { t.on_surface_variant },
            }
        };
        self.paint_material_icon(r.bounds.inset(8.0), icon, color);
        r
    }
    /// The glyph is presentation; `label` names the action for assistive technology.
    pub fn icon_button(&mut self, id: u64, glyph: &str, label: &str, options: Options) -> Response {
        let mut b = self.allocate(40.0);
        b.width = b.width.min(40.0);
        let r = self.interact(id, b, options.enabled);
        let t = self.ui.theme;
        let color = if !r.enabled {
            t.text.opacity(0.38)
        } else {
            match options.variant {
                ButtonVariant::Filled => t.on_primary,
                ButtonVariant::Tonal => t.on_secondary,
                _ => {
                    if options.selected {
                        t.primary
                    } else {
                        t.on_surface_variant
                    }
                }
            }
        };
        match options.variant {
            ButtonVariant::Filled | ButtonVariant::Tonal => self.rect(
                b,
                20.0,
                if !r.enabled {
                    t.text.opacity(0.12)
                } else if options.variant == ButtonVariant::Filled {
                    t.primary
                } else {
                    t.secondary
                },
            ),
            ButtonVariant::Outlined => {
                self.rect(
                    b,
                    20.0,
                    if r.enabled {
                        t.outline
                    } else {
                        t.text.opacity(0.12)
                    },
                );
                self.rect(b.inset(1.0), 19.0, t.background);
            }
            _ => {}
        }
        self.state_layer(r, 20.0, color);
        self.styled_text(
            b,
            glyph,
            color,
            TextStyle {
                size: 24.0,
                align: 1,
                ..TextStyle::default()
            },
        );
        self.focus_ring(r);
        self.semantic(r, Role::Button, label, options.selected, None, None, None);
        r
    }

    /// Material icon button placed at caller-supplied `bounds` using a bundled `MaterialIcon`.
    pub fn material_icon_button_at(
        &mut self,
        id: u64,
        bounds: Rect,
        icon: MaterialIcon,
        label: &str,
        options: Options,
    ) -> Response {
        let r = self.interact(id, bounds, options.enabled);
        let t = self.ui.theme;
        let radius = bounds.width.min(bounds.height) * 0.5;
        let color = if !r.enabled {
            t.text.opacity(0.38)
        } else {
            match options.variant {
                ButtonVariant::Filled => t.on_primary,
                ButtonVariant::Tonal => t.on_secondary,
                _ => {
                    if options.selected {
                        t.primary
                    } else {
                        t.on_surface_variant
                    }
                }
            }
        };
        match options.variant {
            ButtonVariant::Filled | ButtonVariant::Tonal => self.rect(
                bounds,
                radius,
                if !r.enabled {
                    t.text.opacity(0.12)
                } else if options.variant == ButtonVariant::Filled {
                    t.primary
                } else {
                    t.secondary
                },
            ),
            ButtonVariant::Outlined => {
                self.rect(bounds, radius, if r.enabled { t.outline } else { t.text.opacity(0.12) });
                self.rect(bounds.inset(1.0), (radius - 1.0).max(0.0), t.background);
            }
            _ => {}
        }
        self.state_layer(r, radius, color);
        let inset = (bounds.width.min(bounds.height) - 24.0).max(0.0) * 0.5;
        self.paint_material_icon(bounds.inset(inset), icon, color);
        self.focus_ring(r);
        self.semantic(r, Role::Button, label, options.selected, None, None, None);
        r
    }

    /// Pill button with Beer CSS / Material hover-wave effect.
    pub fn hover_wave_button(&mut self, id: u64, label: &str, enabled: bool) -> Response {
        let b = self.allocate(40.0);
        let r = self.target(id, b, label, enabled, Role::Button);
        let t = self.theme();
        let radius = (b.height * 0.5).min(20.0);
        let bg = if r.enabled { t.primary } else { t.text.opacity(0.12) };
        let fg = if r.enabled { t.on_primary } else { t.text.opacity(0.38) };
        self.paint_rect(b, radius, bg);
        self.wave(r, radius, fg);
        self.styled_text(b, label, fg, TextStyle::LABEL);
        self.focus_ring(r);
        r
    }
}

#[cfg(test)]
#[path = "button.spec.rs"]
mod tests;
