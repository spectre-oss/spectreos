use crate::*;

/// Elevation policy from Material's FAB defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabElevation {
    #[default]
    Standard,
    Lowered,
    BottomAppBar,
    /// Material floating-toolbar FAB: level 2 at rest, level 3 on hover.
    FloatingToolbar,
}

/// Material FAB container and icon color roles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Surface,
}

#[derive(Clone, Copy, Debug)]
pub struct FabOptions {
    pub size: f32,
    pub extended: bool,
    pub enabled: bool,
    pub rtl: bool,
    pub elevation: FabElevation,
    pub color: FabColor,
    /// Optional custom corner radius in logical pixels.
    pub corner_radius: Option<f32>,
}
impl Default for FabOptions {
    fn default() -> Self {
        Self {
            size: 56.0,
            extended: false,
            enabled: true,
            rtl: false,
            elevation: FabElevation::Standard,
            color: FabColor::Primary,
            corner_radius: None,
        }
    }
}

impl Frame<'_> {
    pub fn fab(
        &mut self,
        id: u64,
        label: &str,
        glyph: &str,
        size: f32,
        extended: bool,
        enabled: bool,
    ) -> Response {
        self.fab_impl(
            id,
            label,
            glyph,
            None,
            FabOptions {
                size,
                extended,
                enabled,
                ..FabOptions::default()
            },
        )
    }
    /// FAB with a bundled Material icon; extended width hugs its icon and label.
    pub fn fab_with_icon(
        &mut self,
        id: u64,
        label: &str,
        icon: AppBarIcon,
        size: f32,
        extended: bool,
        enabled: bool,
    ) -> Response {
        self.fab_with_icon_direction(id, label, icon, size, extended, enabled, false)
    }
    /// FAB with logical leading icon and trailing label for left- or right-to-left layouts.
    #[allow(clippy::too_many_arguments)]
    pub fn fab_with_icon_direction(
        &mut self,
        id: u64,
        label: &str,
        icon: AppBarIcon,
        size: f32,
        extended: bool,
        enabled: bool,
        rtl: bool,
    ) -> Response {
        self.fab_with_options(
            id,
            label,
            icon,
            FabOptions {
                size,
                extended,
                enabled,
                rtl,
                ..FabOptions::default()
            },
        )
    }
    pub fn fab_with_options(
        &mut self,
        id: u64,
        label: &str,
        icon: AppBarIcon,
        options: FabOptions,
    ) -> Response {
        self.fab_impl(id, label, "", Some(icon), options)
    }
    #[allow(clippy::too_many_arguments)]
    fn fab_impl(
        &mut self,
        id: u64,
        label: &str,
        glyph: &str,
        icon: Option<AppBarIcon>,
        options: FabOptions,
    ) -> Response {
        let FabOptions {
            size,
            extended,
            enabled,
            rtl,
            elevation,
            color,
            corner_radius,
        } = options;
        let icon_size = if size >= 96.0 { 36.0 } else if size >= 80.0 { 28.0 } else { 24.0 };
        let (leading,gap,trailing,label_style)=if size>=96.0 {(28.0,16.0,28.0,TextStyle{size:24.0,weight:400,..TextStyle::default()})}else if size>=80.0 {(26.0,12.0,26.0,TextStyle{size:22.0,weight:400,..TextStyle::default()})}else{(16.0,12.0,20.0,TextStyle::LABEL)};
        let target=f32::from(extended);
        let expansion=if icon.is_some(){self.animate_geometry(id,97,target,target).clamp(0.0,1.0)}else{target};
        let label_opacity=if icon.is_some(){self.ui.motion.effect(id,98,target,target,crate::motion::Spring::FAST_EFFECTS).clamp(0.0,1.0)}else{target};
        let mut b = self.allocate(size);
        let collapsed=b.width.min(size);
        if icon.is_some() {
            let expanded=b.width.min((leading + icon_size + gap + self.text_width(label, label_style) + trailing).max(size));
            b.width=collapsed+(expanded-collapsed)*expansion;
        } else if !extended {b.width=collapsed;}
        let default_radius = if size <= 40.0 {12.0} else if size <= 56.0 {16.0} else if size < 96.0 {20.0} else {28.0};
        let radius=corner_radius.filter(|radius|radius.is_finite()).unwrap_or(default_radius).clamp(0.0,b.width.min(b.height).max(0.0)/2.0);
        let r = self.interact(id, b, enabled);
        let t = self.ui.theme;
        // Material FABs rest at 6 dp; hover raises them to 8 dp, while press
        // and keyboard focus return to 6 dp. Shadow geometry interpolates.
        let hover_elevation = self
            .effect(
                id,
                96,
                f32::from(
                    r.enabled && r.hovered && !r.pressed && !(r.focused && self.ui.keyboard_focus),
                ),
                0.0,
            )
            .clamp(0.0, 1.0);
        if r.enabled && elevation == FabElevation::FloatingToolbar {
            self.elevation_dp(b,radius,3.0+3.0*hover_elevation);
        } else if r.enabled && elevation != FabElevation::BottomAppBar {
            let shadows = if elevation == FabElevation::Lowered {
                [
                    (
                        2.0 + hover_elevation,
                        1.0 + 2.0 * hover_elevation,
                        -1.0 - hover_elevation,
                        0.2,
                    ),
                    (
                        1.0 + 2.0 * hover_elevation,
                        1.0 + 3.0 * hover_elevation,
                        0.0,
                        0.14,
                    ),
                    (1.0, 3.0 + 5.0 * hover_elevation, 0.0, 0.12),
                ]
            } else {
                [
                    (
                        3.0 + 2.0 * hover_elevation,
                        5.0,
                        -1.0 - 2.0 * hover_elevation,
                        0.2,
                    ),
                    (6.0 + 2.0 * hover_elevation, 10.0, hover_elevation, 0.14),
                    (
                        1.0 + 2.0 * hover_elevation,
                        18.0 - 4.0 * hover_elevation,
                        2.0 * hover_elevation,
                        0.12,
                    ),
                ]
            };
            for (y, blur, spread, alpha) in shadows {
                self.shadow(
                    b,
                    radius,
                    [0.0, y],
                    blur,
                    spread,
                    Color(0, 0, 0, 255).opacity(alpha),
                );
            }
        }
        let (container, on_container) = match color {
            FabColor::Primary => (t.primary_container, t.on_primary_container),
            FabColor::Secondary => (t.secondary_container, t.on_secondary_container),
            FabColor::Tertiary => (t.tertiary_container, t.on_tertiary_container),
            FabColor::Surface => (t.surface_high, t.primary),
        };
        self.rect(
            b,
            radius,
            if r.enabled {
                container
            } else {
                t.text.opacity(0.12)
            },
        );
        self.state_layer(r, radius, on_container);
        let foreground = if r.enabled {
            on_container
        } else {
            t.text.opacity(0.38)
        };
        if let Some(icon) = icon {
            self.commands
                .push(DrawCommand::PushClip(b.intersect(self.clip)));
            let drawn_icon_size=icon_size.min(b.width).min(b.height).max(0.0);
            let icon_bounds = Rect {
                x: {let center=b.x+(b.width-drawn_icon_size)/2.0;let inset=leading.min(((b.width-drawn_icon_size)/2.0).max(0.0));let leading=if rtl{b.x+b.width-inset-drawn_icon_size}else{b.x+inset};center+(leading-center)*expansion},
                y: b.y + (b.height - drawn_icon_size) / 2.0,
                width: drawn_icon_size,
                height: drawn_icon_size,
            };
            self.commands.push(DrawCommand::Shape {
                bounds: icon_bounds,
                index: icon.shape_index(rtl),
                color: foreground,
                rotation: 0.0,
            });
            if label_opacity > 0.0 {
                self.with_opacity(label_opacity, |this| this.styled_text(
                    Rect {
                        x: if rtl {
                            b.x + trailing
                        } else {
                            b.x + leading + icon_size + gap
                        },
                        width: (b.width - leading - gap - trailing - icon_size).max(0.0),
                        ..b
                    },
                    label,
                    foreground,
                    TextStyle {
                        align: if rtl { 2 } else { 0 },
                        padding: 0.0,
                        ..label_style
                    },
                ));
            }
            self.commands.push(DrawCommand::PopClip);
        } else {
            self.styled_text(
                b,
                if extended { label } else { glyph },
                if r.enabled {
                    t.on_primary_container
                } else {
                    t.text.opacity(0.38)
                },
                if extended {
                    TextStyle::LABEL
                } else {
                    TextStyle {
                        size: 13.333_333,
                        align: 1,
                        ..TextStyle::default()
                    }
                },
            );
        }
        self.focus_ring(r);
        self.semantic(r, Role::Button, label, false, None, None, None);
        r
    }

    /// Floating Action Button placed at caller-supplied `bounds` (for anchored viewport placement).
    pub fn fab_at(
        &mut self,
        id: u64,
        bounds: Rect,
        icon: MaterialIcon,
        label: &str,
        options: FabOptions,
    ) -> Response {
        let t = self.ui.theme;
        let radius = options
            .corner_radius
            .filter(|r| r.is_finite())
            .unwrap_or_else(|| {
                let sz = bounds.width.min(bounds.height);
                if sz <= 40.0 { 12.0 } else if sz <= 56.0 { 16.0 } else if sz < 96.0 { 20.0 } else { 28.0 }
            })
            .clamp(0.0, bounds.width.min(bounds.height).max(0.0) * 0.5);
        let r = self.interact(id, bounds, options.enabled);
        if r.enabled && options.elevation != FabElevation::BottomAppBar {
            let level = match options.elevation {
                FabElevation::Lowered => 1,
                FabElevation::FloatingToolbar => 2,
                _ => 6,
            };
            self.elevation(bounds, radius, level);
        }
        let (container, on_container) = match options.color {
            FabColor::Primary => (t.primary_container, t.on_primary_container),
            FabColor::Secondary => (t.secondary_container, t.on_secondary_container),
            FabColor::Tertiary => (t.tertiary_container, t.on_tertiary_container),
            FabColor::Surface => (t.surface_high, t.primary),
        };
        let fg = if r.enabled { on_container } else { t.text.opacity(0.38) };
        self.rect(bounds, radius, if r.enabled { container } else { t.text.opacity(0.12) });
        self.state_layer(r, radius, on_container);
        let icon_sz = if bounds.height >= 96.0 { 36.0 } else if bounds.height >= 80.0 { 28.0 } else { 24.0 };
        if options.extended && !label.is_empty() {
            let lead = 16.0;
            let icon_rect = Rect {
                x: if options.rtl { bounds.x + bounds.width - lead - icon_sz } else { bounds.x + lead },
                y: bounds.y + (bounds.height - icon_sz) * 0.5,
                width: icon_sz,
                height: icon_sz,
            };
            self.paint_material_icon(icon_rect, icon, fg);
            let text_rect = Rect {
                x: if options.rtl { bounds.x + 16.0 } else { bounds.x + lead + icon_sz + 12.0 },
                width: (bounds.width - lead - icon_sz - 28.0).max(0.0),
                ..bounds
            };
            self.styled_text(text_rect, label, fg, TextStyle { align: if options.rtl { 2 } else { 0 }, padding: 0.0, ..TextStyle::LABEL });
        } else {
            let icon_rect = Rect {
                x: bounds.x + (bounds.width - icon_sz) * 0.5,
                y: bounds.y + (bounds.height - icon_sz) * 0.5,
                width: icon_sz,
                height: icon_sz,
            };
            self.paint_material_icon(icon_rect, icon, fg);
        }
        self.focus_ring(r);
        self.semantic(r, Role::Button, label, false, None, None, None);
        r
    }

    /// Anchored `MaterialIcon` FAB whose width smoothly animates between collapsed (`size × size`)
    /// and extended (`icon + label`) right- or left-aligned inside `anchor_bounds`.
    pub fn fab_at_animated(
        &mut self,
        id: u64,
        anchor_bounds: Rect,
        icon: MaterialIcon,
        label: &str,
        options: FabOptions,
    ) -> Response {
        let size = options.size.max(40.0).min(anchor_bounds.height.max(40.0));
        let icon_sz = if size >= 96.0 { 36.0 } else if size >= 80.0 { 28.0 } else { 24.0 };
        let target = f32::from(options.extended && !label.is_empty());
        let expansion = self.animate_geometry(id, 97, target, target).clamp(0.0, 1.0);
        let label_alpha = self
            .ui
            .motion
            .effect(id, 98, target, target, crate::motion::Spring::FAST_EFFECTS)
            .clamp(0.0, 1.0);
        let expanded_w = (16.0 + icon_sz + 12.0 + self.text_width(label, TextStyle::LABEL) + 20.0)
            .max(size)
            .min(anchor_bounds.width.max(size));
        let width = size + (expanded_w - size) * expansion;
        let bounds = Rect {
            x: if options.rtl {
                anchor_bounds.x
            } else {
                anchor_bounds.x + (anchor_bounds.width - width).max(0.0)
            },
            y: anchor_bounds.y + (anchor_bounds.height - size).max(0.0) * 0.5,
            width,
            height: size,
        };
        let t = self.ui.theme;
        let default_r = if size <= 40.0 { 12.0 } else if size <= 56.0 { 16.0 } else if size < 96.0 { 20.0 } else { 28.0 };
        let radius = options
            .corner_radius
            .filter(|r| r.is_finite())
            .unwrap_or(default_r)
            .clamp(0.0, bounds.width.min(bounds.height).max(0.0) * 0.5);
        let r = self.interact(id, bounds, options.enabled);
        if r.enabled && options.elevation != FabElevation::BottomAppBar {
            let level = match options.elevation {
                FabElevation::Lowered => 1,
                FabElevation::FloatingToolbar => 2,
                _ => 6,
            };
            self.elevation(bounds, radius, level);
        }
        let (container, on_container) = match options.color {
            FabColor::Primary => (t.primary_container, t.on_primary_container),
            FabColor::Secondary => (t.secondary_container, t.on_secondary_container),
            FabColor::Tertiary => (t.tertiary_container, t.on_tertiary_container),
            FabColor::Surface => (t.surface_high, t.primary),
        };
        let fg = if r.enabled { on_container } else { t.text.opacity(0.38) };
        self.rect(bounds, radius, if r.enabled { container } else { t.text.opacity(0.12) });
        self.state_layer(r, radius, on_container);
        self.within(bounds, |f| {
            let center_x = bounds.x + (bounds.width - icon_sz) * 0.5;
            let inset_x = if options.rtl { bounds.x + bounds.width - 16.0 - icon_sz } else { bounds.x + 16.0 };
            let icon_rect = Rect {
                x: center_x + (inset_x - center_x) * expansion,
                y: bounds.y + (bounds.height - icon_sz) * 0.5,
                width: icon_sz,
                height: icon_sz,
            };
            f.paint_material_icon(icon_rect, icon, fg);
            if label_alpha > 0.01 && !label.is_empty() {
                let text_rect = Rect {
                    x: if options.rtl { bounds.x + 20.0 } else { bounds.x + 16.0 + icon_sz + 12.0 },
                    width: (bounds.width - 16.0 - icon_sz - 32.0).max(0.0),
                    ..bounds
                };
                f.with_opacity(label_alpha, |f| {
                    f.styled_text(
                        text_rect,
                        label,
                        fg,
                        TextStyle { align: if options.rtl { 2 } else { 0 }, padding: 0.0, ..TextStyle::LABEL },
                    );
                });
            }
        });
        self.focus_ring(r);
        self.semantic(r, Role::Button, label, false, None, None, None);
        r
    }
}

#[cfg(test)]
#[path = "fab.spec.rs"]
mod tests;
