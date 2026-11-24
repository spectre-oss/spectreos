use crate::*;

/// Size of a Material split button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitButtonSize {
    ExtraSmall,
    #[default]
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Clone, Copy, Debug)]
pub struct SplitButtonOptions {
    pub size: SplitButtonSize,
    pub variant: ButtonVariant,
    pub primary_enabled: bool,
    pub menu_enabled: bool,
    pub expanded: bool,
    pub rtl: bool,
    pub leading_icon: Option<MaterialIcon>,
}
impl Default for SplitButtonOptions {
    fn default() -> Self {
        Self {
            size: SplitButtonSize::Small,
            variant: ButtonVariant::Filled,
            primary_enabled: true,
            menu_enabled: true,
            expanded: false,
            rtl: false,
            leading_icon: None,
        }
    }
}

impl Frame<'_> {
    /// Draw a small split button with independent primary and menu actions.
    /// The caller owns expanded state and anchors its menu to the trailing response.
    pub fn split_button(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        menu_label: &str,
        options: Options,
    ) -> Option<(Response, Response)> {
        self.split_button_with(
            id,
            bounds,
            label,
            menu_label,
            SplitButtonOptions {
                variant: options.variant,
                primary_enabled: options.enabled,
                menu_enabled: options.enabled,
                expanded: options.selected,
                ..SplitButtonOptions::default()
            },
        )
    }
    /// Sized split button with independently enabled actions and RTL placement.
    pub fn split_button_with(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        menu_label: &str,
        config: SplitButtonOptions,
    ) -> Option<(Response, Response)> {
        let (visual_height, inner_default, inner_interactive, icon_size): (f32, f32, f32, f32) =
            match config.size {
                SplitButtonSize::ExtraSmall => (32.0, 4.0, 8.0, 22.0),
                SplitButtonSize::Small => (40.0, 4.0, 12.0, 22.0),
                SplitButtonSize::Medium => (56.0, 4.0, 12.0, 26.0),
                SplitButtonSize::Large => (96.0, 8.0, 20.0, 38.0),
                SplitButtonSize::ExtraLarge => (136.0, 12.0, 20.0, 50.0),
            };
        let hit_height = visual_height.max(48.0);
        let menu_width = hit_height;
        if !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width < menu_width + 50.0
            || bounds.height < hit_height
            || id == u64::MAX
        {
            return None;
        }
        let lead = Rect {
            x: bounds.x + if config.rtl { menu_width + 2.0 } else { 0.0 },
            width: bounds.width - menu_width - 2.0,
            height: hit_height,
            ..bounds
        };
        let trail = Rect {
            x: if config.rtl {
                bounds.x
            } else {
                lead.x + lead.width + 2.0
            },
            width: menu_width,
            height: hit_height,
            ..bounds
        };
        let primary = self.target(id, lead, label, config.primary_enabled, Role::Button);
        let menu_description = format!(
            "{}, {}",
            menu_label,
            if config.expanded {
                "expanded"
            } else {
                "collapsed"
            }
        );
        let menu = self.target(
            id + 1,
            trail,
            &menu_description,
            config.menu_enabled,
            Role::Button,
        );
        let t = self.ui.theme;
        for (i, r) in [primary, menu].into_iter().enumerate() {
            let b = Rect {
                y: r.bounds.y + (hit_height - visual_height) / 2.0,
                height: visual_height,
                ..r.bounds
            };
            let interactive = r.pressed || r.hovered;
            let outer = visual_height / 2.0;
            let target = if r.pressed {
                inner_interactive
            } else if i == 1 && config.expanded {
                outer
            } else if interactive {
                inner_interactive
            } else {
                inner_default
            };
            let inner = self
                .spatial(r.id, 60, target / outer, inner_default / outer)
                .clamp(0.0, 1.0)
                * outer;
            let outside_left = (i == 0) != config.rtl;
            let (left, right) = if outside_left {
                (outer, inner)
            } else {
                (inner, outer)
            };
            let (base, foreground) = if !r.enabled {
                if config.variant == ButtonVariant::Outlined {
                    (t.background, t.on_surface_variant.opacity(0.38))
                } else {
                    (t.text.opacity(0.12), t.text.opacity(0.38))
                }
            } else {
                match config.variant {
                    ButtonVariant::Tonal => (t.secondary, t.on_secondary),
                    ButtonVariant::Outlined => (t.background, t.primary),
                    ButtonVariant::Elevated => (t.surface_low, t.primary),
                    _ => (t.primary, t.on_primary),
                }
            };
            if r.enabled && config.variant == ButtonVariant::Elevated {
                self.connected_elevation(b, left, right, 1);
            }
            if config.variant == ButtonVariant::Outlined {
                self.paint_connected_outline(
                    b,
                    left,
                    right,
                    if r.enabled {
                        t.outline
                    } else {
                        t.outline.opacity(0.1)
                    },
                );
            } else {
                self.paint_connected_halves(b, left, right, base);
            }
            // Compose's checked trailing button keeps a 12% content-color
            // state layer while the menu is expanded, independently of hover.
            if i == 1 && config.expanded {
                self.paint_connected_halves(b, left, right, foreground.opacity(0.12));
            }
            let hover = self.effect(r.id, 0, f32::from(r.hovered), 0.0);
            let alpha = if r.pressed || (r.focused && self.ui.keyboard_focus) {
                0.12
            } else {
                0.08 * hover
            };
            if r.enabled && alpha > 0.001 {
                self.paint_connected_halves(b, left, right, foreground.opacity(alpha));
            }
            self.connected_ink(r, b, left, right, foreground);
            self.focus_ring(r);
            if i == 0 {
                if let Some(icon) = config.leading_icon {
                    let glyph_sz = (icon_size * 0.85).min(b.height - 6.0).max(14.0);
                    let gap = 8.0;
                    let text_w = self.text_width(label, TextStyle::LABEL);
                    let total_w = (glyph_sz + gap + text_w).min((b.width - 16.0).max(glyph_sz));
                    let start_x = b.x + (b.width - total_w) * 0.5;
                    let icon_x = if config.rtl { start_x + total_w - glyph_sz } else { start_x };
                    let text_x = if config.rtl { start_x } else { start_x + glyph_sz + gap };
                    self.paint_material_icon(
                        Rect {
                            x: icon_x,
                            y: b.y + (b.height - glyph_sz) * 0.5,
                            width: glyph_sz,
                            height: glyph_sz,
                        },
                        icon,
                        foreground,
                    );
                    self.styled_text(
                        Rect { x: text_x, width: (total_w - glyph_sz - gap).max(0.0), ..b },
                        label,
                        foreground,
                        TextStyle { align: if config.rtl { 2 } else { 0 }, padding: 0.0, ..TextStyle::LABEL },
                    );
                } else {
                    self.styled_text(
                        b,
                        label,
                        foreground,
                        TextStyle {
                            align: 1,
                            ..TextStyle::LABEL
                        },
                    );
                }
            } else {
                let rotation = self
                    .spatial(
                        r.id,
                        61,
                        f32::from(config.expanded),
                        f32::from(config.expanded),
                    )
                    .clamp(0.0, 1.0)
                    * std::f32::consts::PI;
                let (sin, cos) = rotation.sin_cos();
                let cx = b.x + b.width / 2.0;
                let cy = b.y + b.height / 2.0;
                let transform = |x: f32, y: f32| [cx + x * cos - y * sin, cy + x * sin + y * cos];
                let scale = icon_size / 22.0;
                self.paint_line(
                    transform(-5.0 * scale, -2.5 * scale),
                    transform(0.0, 2.5 * scale),
                    2.0 * scale,
                    foreground,
                );
                self.paint_line(
                    transform(0.0, 2.5 * scale),
                    transform(5.0 * scale, -2.5 * scale),
                    2.0 * scale,
                    foreground,
                );
            }
        }
        Some((primary, menu))
    }
}

#[cfg(test)]
#[path = "split_button.spec.rs"]
mod tests;
