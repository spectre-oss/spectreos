use crate::*;

/// A destination's icon is supplied by the caller as a glyph; badge text and
/// its accessible description are optional.
#[derive(Clone, Copy, Debug)]
pub struct NavigationDestination<'a> {
    pub label: &'a str,
    pub icon: &'a str,
    pub enabled: bool,
    pub badge: Option<(BadgeContent<'a>, &'a str)>,
}

/// Bundled Google Material Icons, with filled and outlined variants.
#[derive(Clone, Copy, Debug)]
pub enum NavigationIcon {
    Home,
    Folder,
    Settings,
    Alarm,
    Clock,
    Timer,
    Stopwatch,
    Menu,
}
impl NavigationIcon {
    pub(super) fn shape_index(self, selected: bool) -> u8 {
        let base = match self {
            Self::Home => 0,
            Self::Folder => 2,
            Self::Settings => 4,
            Self::Alarm => 6,
            Self::Clock => 8,
            Self::Timer => 10,
            Self::Stopwatch => 12,
            Self::Menu => 14,
        };
        (crate::beercss_shapes::SHAPES.len() + base + usize::from(!selected)) as u8
    }
}

/// Reusable Material navigation chrome with state layers and selection motion.

impl Frame<'_> {
    /// A navigation drawer/rail row. Expansion is a continuous 0..1 value.
    #[allow(clippy::too_many_arguments)]
    pub fn material_navigation_item(
        &mut self,
        id: u64,
        b: Rect,
        label: &str,
        icon: MaterialIcon,
        selected: bool,
        expansion: f32,
        enabled: bool,
    ) -> Response {
        let r = self.interact(id, b, enabled);
        self.semantic(r, Role::ListItem, label, selected, None, None, None);
        let t = self.theme();
        let selected_alpha = self.transition(
            id,
            90,
            f32::from(selected),
            f32::from(selected),
            0.2,
            timing::Easing::Standard,
        );
        self.paint_rect(b, b.height / 2., t.secondary.opacity(selected_alpha));
        let color = if selected {
            t.on_secondary
        } else {
            t.on_surface_variant
        };
        self.state_layer(r, b.height / 2., color);
        let x = b.x + (b.width - 24.) / 2. + (16. - (b.width - 24.) / 2.) * expansion;
        self.paint_material_icon(
            Rect {
                x,
                y: b.y + (b.height - 24.) / 2.,
                width: 24.,
                height: 24.,
            },
            icon,
            color,
        );
        self.styled_text(
            Rect {
                x: b.x + 52.,
                width: (b.width - 64.).max(0.),
                ..b
            },
            label,
            color.opacity(expansion),
            TextStyle {
                size: 14.,
                weight: 500,
                ..TextStyle::default()
            },
        );
        self.focus_ring(r);
        r
    }
    /// Material Android NavigationView destination. `bounds` is the full 56dp row;
    /// the indicator has 12dp horizontal insets and icons use a 24dp design box.
    #[allow(clippy::too_many_arguments)]
    pub fn material_drawer_item(
        &mut self,
        id: u64,
        bounds: Rect,
        label: &str,
        icon: MaterialIcon,
        selected: bool,
        enabled: bool,
    ) -> Response {
        let r = self.interact(id, bounds, enabled);
        self.semantic(r, Role::ListItem, label, selected, None, None, None);
        let t = self.theme();
        let color = if selected {
            t.on_secondary
        } else {
            t.on_surface_variant
        };
        let indicator = Rect {
            x: bounds.x + 12.,
            width: (bounds.width - 24.).max(0.),
            ..bounds
        };
        let p = self.transition(
            id,
            90,
            f32::from(selected),
            f32::from(selected),
            0.2,
            timing::Easing::Standard,
        );
        self.paint_rect(indicator, indicator.height / 2., t.secondary.opacity(p));
        let state = Response {
            bounds: indicator,
            ..r
        };
        self.state_layer(state, indicator.height / 2., color);
        self.paint_material_icon(
            Rect {
                x: bounds.x + 28.,
                y: bounds.y + (bounds.height - 24.) / 2.,
                width: 24.,
                height: 24.,
            },
            icon,
            color,
        );
        self.styled_text(
            Rect {
                x: bounds.x + 64.,
                width: (bounds.width - 92.).max(0.),
                ..bounds
            },
            label,
            color,
            TextStyle {
                size: 14.,
                weight: if selected { 700 } else { 500 },
                ..TextStyle::default()
            },
        );
        self.focus_ring(r);
        r
    }
    /// Standard 80dp bottom navigation destination, including animated indicator.
    #[allow(clippy::too_many_arguments)]
    pub fn material_navigation_bar_item(
        &mut self,
        id: u64,
        b: Rect,
        label: &str,
        icon: MaterialIcon,
        selected: bool,
        enabled: bool,
    ) -> Response {
        let r = self.interact(id, b, enabled);
        self.semantic(r, Role::Tab, label, selected, None, None, None);
        let t = self.theme();
        let p = self.transition(
            id,
            91,
            f32::from(selected),
            f32::from(selected),
            0.3,
            timing::Easing::Standard,
        );
        let indicator = Rect {
            x: b.x + b.width / 2. - 32.,
            y: b.y + 8.,
            width: 64.,
            height: 32.,
        };
        self.paint_rect(
            Rect {
                x: indicator.x + 32. * (1. - p),
                width: 64. * p,
                ..indicator
            },
            16.,
            t.secondary.opacity(p),
        );
        self.state_layer(
            Response {
                bounds: indicator,
                ..r
            },
            16.,
            t.on_surface_variant,
        );
        self.paint_material_icon(
            Rect {
                x: b.x + b.width / 2. - 12.,
                y: b.y + 12.,
                width: 24.,
                height: 24.,
            },
            icon,
            if selected {
                t.on_secondary
            } else {
                t.on_surface_variant
            },
        );
        self.styled_text(
            Rect {
                y: b.y + 44.,
                height: 24.,
                ..b
            },
            label,
            t.on_surface_variant,
            TextStyle {
                size: 12.,
                weight: 500,
                align: 1,
                ..TextStyle::default()
            },
        );
        self.focus_ring(r);
        r
    }
}

impl Frame<'_> {
    /// Horizontal navigation destinations with caller-owned selection.
    pub fn navigation_bar(
        &mut self,
        id: u64,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
    ) -> bool {
        self.navigation_bar_with_labels(id, destinations, selected, rtl, true)
    }

    /// When `always_show_labels` is false, unselected icons move to the
    /// center and their labels fade out. Accessible names remain available.
    pub fn navigation_bar_with_labels(
        &mut self,
        id: u64,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
        always_show_labels: bool,
    ) -> bool {
        self.navigation_bar_with_icons(id, destinations, selected, rtl, always_show_labels, &[])
    }

    /// Optional built-in vector icons are matched by destination index. Missing
    /// entries use the destination's caller-supplied glyph.
    pub fn navigation_bar_with_icons(
        &mut self,
        id: u64,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
        always_show_labels: bool,
        icons: &[NavigationIcon],
    ) -> bool {
        let bar = self.reserve(80.0);
        self.navigation_destinations_at(
            id,
            bar,
            destinations,
            selected,
            rtl,
            always_show_labels,
            icons,
            false,
            0.0,
            false,
        )
    }

    /// A compact vertical navigation rail. The caller may position it using
    /// `within`; selection and all destination data remain caller-owned.
    pub fn navigation_rail(
        &mut self,
        id: u64,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
        always_show_labels: bool,
        icons: &[NavigationIcon],
    ) -> bool {
        let allocated = self.reserve(destinations.len() as f32 * 60.0 + 4.0);
        let bar = Rect {
            width: allocated.width.min(80.0),
            ..allocated
        };
        self.navigation_destinations_at(
            id,
            bar,
            destinations,
            selected,
            rtl,
            always_show_labels,
            icons,
            true,
            0.0,
            false,
        )
    }

    /// Narrow-to-wide rail with caller-owned expanded state. Geometry follows
    /// the current motion sample, including when the transition reverses.
    #[allow(clippy::too_many_arguments)]
    pub fn animated_navigation_rail(
        &mut self,
        id: u64,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
        always_show_labels: bool,
        icons: &[NavigationIcon],
        expanded: bool,
    ) -> bool {
        let allocated = self.reserve(destinations.len() as f32 * 60.0 + 4.0);
        let target = f32::from(expanded);
        let progress = self.spatial(id, 75, target, target).clamp(0.0, 1.0);
        let width = (80.0 + 160.0 * progress).min(allocated.width);
        let rail = Rect { width, ..allocated };
        self.navigation_destinations_at(
            id,
            rail,
            destinations,
            selected,
            rtl,
            always_show_labels,
            icons,
            true,
            progress,
            false,
        )
    }

    /// Android Material 3 Expressive collapsed/expanded destination layout.
    #[allow(clippy::too_many_arguments)]
    pub fn expressive_navigation_rail(
        &mut self,
        id: u64,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
        icons: &[NavigationIcon],
        expanded: bool,
    ) -> bool {
        let progress = self.navigation_rail_expansion(id, expanded);
        self.expressive_navigation_rail_at_progress(
            id,
            destinations,
            selected,
            rtl,
            icons,
            progress,
        )
    }

    /// Sample the rail's expansion once per frame to synchronize its header.
    pub fn navigation_rail_expansion(&mut self, id: u64, expanded: bool) -> f32 {
        let target = f32::from(expanded);
        self.spatial(id, 75, target, target).clamp(0.0, 1.0)
    }

    /// Draw a rail using a shared expansion sample from `navigation_rail_expansion`.
    pub fn expressive_navigation_rail_at_progress(
        &mut self,
        id: u64,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
        icons: &[NavigationIcon],
        progress: f32,
    ) -> bool {
        let progress = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let allocated = self.reserve(destinations.len() as f32 * 68.0 + 4.0);
        let width = (96.0 + 144.0 * progress).min(allocated.width);
        self.navigation_destinations_at(
            id,
            Rect { width, ..allocated },
            destinations,
            selected,
            rtl,
            true,
            icons,
            true,
            progress,
            true,
        )
    }

    /// Expanded, non-modal rail with 56 dp horizontal destination rows.
    /// The enclosing application controls rail placement and expansion.
    pub fn wide_navigation_rail(
        &mut self,
        id: u64,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
        icons: &[NavigationIcon],
    ) -> bool {
        let allocation = self.reserve(destinations.len() as f32 * 60.0 + 4.0);
        let rail = Rect {
            width: allocation.width.min(240.0),
            ..allocation
        };
        if destinations.is_empty() {
            return false;
        }
        *selected = (*selected).min(destinations.len() - 1);
        let old = *selected;
        let t = self.ui.theme;
        self.paint_rect(rail, 0.0, t.surface_low);
        let mut responses = Vec::with_capacity(destinations.len());
        for (i, destination) in destinations.iter().enumerate() {
            let item = Rect {
                x: rail.x + 16.0_f32.min(rail.width / 2.0),
                y: rail.y + 4.0 + i as f32 * 60.0,
                width: (rail.width - 32.0).max(0.0),
                height: 56.0,
            };
            let response = self.interact(id.wrapping_add(i as u64), item, destination.enabled);
            if response.clicked {
                *selected = i;
            }
            responses.push(response);
        }
        for (i, (destination, response)) in destinations.iter().zip(responses).enumerate() {
            let active = i == *selected;
            let item = response.bounds;
            let p = self
                .effect(response.id, 74, f32::from(active), f32::from(active))
                .clamp(0.0, 1.0);
            self.paint_rect(item, 28.0, t.secondary.opacity(p));
            let icon_color = if active {
                t.on_secondary
            } else {
                t.on_surface_variant
            }
            .opacity(if response.enabled { 1.0 } else { 0.38 });
            let label_color = if active { t.text } else { t.on_surface_variant }
                .opacity(if response.enabled { 1.0 } else { 0.38 });
            self.state_layer(response, 28.0, icon_color);
            let icon = Rect {
                x: if rtl {
                    item.x + item.width - 40.0
                } else {
                    item.x + 16.0
                },
                y: item.y + 16.0,
                width: 24.0,
                height: 24.0,
            };
            if let Some(builtin) = icons.get(i) {
                self.commands.push(DrawCommand::Shape {
                    bounds: icon,
                    index: builtin.shape_index(active),
                    color: icon_color,
                    rotation: 0.0,
                });
            } else {
                self.styled_text(
                    icon,
                    destination.icon,
                    icon_color,
                    TextStyle {
                        size: 22.0,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
            }
            let label = Rect {
                x: item.x + if rtl { 16.0 } else { 48.0 },
                y: item.y,
                width: (item.width - 64.0).max(0.0),
                height: item.height,
            };
            self.styled_text(
                label,
                destination.label,
                label_color,
                TextStyle {
                    size: 14.0,
                    weight: 500,
                    align: if rtl { 2 } else { 0 },
                    ..TextStyle::default()
                },
            );
            self.semantic(
                response,
                Role::Tab,
                destination.label,
                active,
                None,
                None,
                None,
            );
            if let Some((badge, description)) = destination.badge {
                self.badge_at(response.id, icon, badge, description, rtl);
            }
            self.focus_ring(response);
        }
        old != *selected
    }

    #[allow(clippy::too_many_arguments)]
    fn navigation_destinations_at(
        &mut self,
        id: u64,
        bar: Rect,
        destinations: &[NavigationDestination<'_>],
        selected: &mut usize,
        rtl: bool,
        always_show_labels: bool,
        icons: &[NavigationIcon],
        rail: bool,
        rail_expansion: f32,
        expressive: bool,
    ) -> bool {
        if destinations.is_empty() {
            return false;
        }
        *selected = (*selected).min(destinations.len() - 1);
        let old = *selected;
        let width = if rail {
            bar.width
        } else {
            bar.width / destinations.len() as f32
        };
        let t = self.ui.theme;
        self.paint_rect(
            bar,
            0.0,
            if expressive {
                t.background
            } else {
                t.surface_low
            },
        );
        let mut responses = Vec::with_capacity(destinations.len());
        for (i, destination) in destinations.iter().enumerate() {
            let bounds = if rail {
                Rect {
                    x: bar.x + 16.0 * rail_expansion,
                    y: bar.y + 4.0 + i as f32 * if expressive { 68.0 } else { 60.0 },
                    width: (width - 32.0 * rail_expansion).max(0.0),
                    height: if expressive { 64.0 } else { 56.0 },
                }
            } else {
                let column = if rtl { destinations.len() - 1 - i } else { i };
                Rect {
                    x: bar.x + column as f32 * width,
                    width,
                    ..bar
                }
            };
            let response = self.interact(id.wrapping_add(i as u64), bounds, destination.enabled);
            if response.clicked {
                *selected = i;
            }
            responses.push(response);
        }
        for (i, (destination, response)) in destinations.iter().zip(responses).enumerate() {
            let active = i == *selected;
            let center = response.bounds.x + response.bounds.width / 2.0;
            // NavigationBarVerticalItemTokens.ActiveIndicatorWidth is 56 dp.
            let indicator_width = 56.0_f32.min(width);
            let progress = self
                .spatial(response.id, 72, f32::from(active), f32::from(active))
                .clamp(0.0, 1.0);
            let label_opacity = self
                .effect(response.id, 73, f32::from(active), f32::from(active))
                .clamp(0.0, 1.0);
            let offset = if always_show_labels {
                0.0
            } else {
                12.0 * (1.0 - progress) * (1.0 - rail_expansion)
            };
            let expanded_width = if expressive {
                (self.text_width(
                    destination.label,
                    TextStyle {
                        size: 14.0,
                        weight: 500,
                        ..TextStyle::default()
                    },
                ) + 64.0)
                    .min((response.bounds.width - 8.0).max(0.0))
            } else {
                response.bounds.width
            };
            let indicator_width =
                indicator_width + (expanded_width - indicator_width) * rail_expansion;
            let indicator_height = 32.0 + 24.0 * rail_expansion;
            let narrow_x = center - indicator_width / 2.0;
            let expanded_x = if rtl {
                response.bounds.x + response.bounds.width - 4.0 - indicator_width
            } else {
                response.bounds.x + 4.0
            };
            let indicator = Rect {
                x: if expressive {
                    narrow_x + (expanded_x - narrow_x) * rail_expansion
                } else {
                    narrow_x
                },
                y: response.bounds.y
                    + if expressive {
                        2.0 + 2.0 * rail_expansion
                    } else if rail {
                        0.0
                    } else {
                        12.0
                    }
                    + offset,
                width: indicator_width,
                height: indicator_height,
            };
            self.paint_rect(
                Rect {
                    x: indicator.x + indicator_width * (1.0 - progress) / 2.0,
                    width: indicator_width * progress,
                    ..indicator
                },
                indicator_height / 2.0,
                t.secondary.opacity(progress),
            );
            let color = if active {
                t.on_secondary
            } else {
                t.on_surface_variant
            }
            .opacity(if response.enabled { 1.0 } else { 0.38 });
            // The label is on the bar surface, not on the indicator.
            let label_color = if active { t.text } else { t.on_surface_variant }
                .opacity(if response.enabled { 1.0 } else { 0.38 });
            self.state_layer(
                Response {
                    bounds: indicator,
                    ..response
                },
                indicator_height / 2.0,
                color,
            );
            let narrow_icon_x = center - 12.0;
            let expanded_icon_x = if rtl {
                response.bounds.x + response.bounds.width - if expressive { 44.0 } else { 40.0 }
            } else {
                response.bounds.x + if expressive { 20.0 } else { 16.0 }
            };
            let icon = Rect {
                x: narrow_icon_x + (expanded_icon_x - narrow_icon_x) * rail_expansion,
                y: response.bounds.y
                    + if rail {
                        if expressive {
                            6.0 + 14.0 * rail_expansion
                        } else {
                            4.0 + 12.0 * rail_expansion
                        }
                    } else {
                        16.0
                    }
                    + offset,
                width: 24.0,
                height: 24.0,
            };
            if let Some(builtin) = icons.get(i) {
                self.commands.push(DrawCommand::Shape {
                    bounds: icon,
                    index: builtin.shape_index(active),
                    color,
                    rotation: 0.0,
                });
            } else {
                self.styled_text(
                    icon,
                    destination.icon,
                    color,
                    TextStyle {
                        size: 22.0,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
            }
            if always_show_labels || label_opacity > 0.001 || rail_expansion > 0.001 {
                let narrow_label = Rect {
                    y: response.bounds.y
                        + if expressive {
                            42.0
                        } else if rail {
                            36.0
                        } else {
                            48.0
                        }
                        + offset,
                    height: 20.0,
                    ..response.bounds
                };
                let wide_label = Rect {
                    x: response.bounds.x
                        + if rtl {
                            16.0
                        } else if expressive {
                            52.0
                        } else {
                            48.0
                        },
                    y: response.bounds.y,
                    width: (response.bounds.width - 64.0).max(0.0),
                    height: response.bounds.height,
                };
                let mix = |a: f32, b: f32| a + (b - a) * rail_expansion;
                self.styled_text(
                    Rect {
                        x: mix(narrow_label.x, wide_label.x),
                        y: mix(narrow_label.y, wide_label.y),
                        width: mix(narrow_label.width, wide_label.width),
                        height: mix(narrow_label.height, wide_label.height),
                    },
                    destination.label,
                    label_color.opacity(if always_show_labels {
                        1.0
                    } else {
                        label_opacity.max(rail_expansion)
                    }),
                    TextStyle {
                        size: 12.0 + 2.0 * rail_expansion,
                        weight: if expressive { 400 } else { 500 },
                        align: if rail_expansion >= 0.5 {
                            if rtl { 2 } else { 0 }
                        } else {
                            1
                        },
                        ..TextStyle::default()
                    },
                );
            }
            self.semantic(
                response,
                Role::Tab,
                destination.label,
                active,
                None,
                None,
                None,
            );
            if let Some((badge, description)) = destination.badge {
                self.badge_at(response.id, icon, badge, description, rtl);
            }
            self.focus_ring(response);
        }
        old != *selected
    }
    pub fn navigation_item(
        &mut self,
        id: u64,
        b: Rect,
        label: &str,
        selected: bool,
        enabled: bool,
    ) -> Response {
        let r = self.interact(id, b, enabled);
        let t = self.ui.theme;
        let color = if selected {
            t.on_secondary
        } else {
            t.on_surface_variant
        }
        .opacity(if r.enabled { 1.0 } else { 0.38 });
        if selected {
            self.rect(b, b.height / 2.0, t.secondary);
        }
        self.state_layer(r, b.height / 2.0, color);
        self.styled_text(
            b,
            label,
            color,
            TextStyle {
                padding: 16.0,
                letter_spacing: 0.5,
                ..TextStyle::default()
            },
        );
        self.semantic(r, Role::ListItem, label, selected, None, None, None);
        r
    }

    /// Material navigation drawer account/title header block.
    pub fn navigation_drawer_header(&mut self, title: &str, subtitle: Option<&str>) {
        let t = self.theme();
        let h = if subtitle.is_some() { 112.0 } else { 80.0 };
        let head = self.reserve(h);
        self.styled_text(
            Rect { x: head.x + 16.0, y: head.y + 36.0, width: (head.width - 32.0).max(0.0), height: 32.0 },
            title,
            t.text,
            TextStyle { size: 20.0, weight: 500, ..TextStyle::default() },
        );
        if let Some(sub) = subtitle {
            self.styled_text(
                Rect { x: head.x + 16.0, y: head.y + 72.0, width: (head.width - 32.0).max(0.0), height: 24.0 },
                sub,
                t.on_surface_variant,
                TextStyle { size: 14.0, ..TextStyle::default() },
            );
        }
    }

    /// Material navigation drawer section subheader label (14sp medium, 28dp horizontal inset).
    pub fn navigation_drawer_section_header(&mut self, label: &str) {
        let t = self.theme();
        let row = self.reserve(36.0);
        self.styled_text(
            Rect { x: row.x + 28.0, width: (row.width - 56.0).max(0.0), ..row },
            label,
            t.on_surface_variant,
            TextStyle { size: 14.0, weight: 600, ..TextStyle::default() },
        );
    }

    /// Persistent Material navigation drawer surface hosting a vertical stack of destinations.
    /// Returns `true` when `*selected` changed.
    pub fn navigation_drawer(
        &mut self,
        id: u64,
        bounds: Rect,
        items: &[(&str, Option<MaterialIcon>)],
        selected: &mut usize,
    ) -> bool {
        if items.is_empty() {
            return false;
        }
        *selected = (*selected).min(items.len() - 1);
        let t = self.theme();
        self.paint_rect(bounds, 16.0, t.surface_low);
        let mut changed = false;
        let row_h = if items.iter().any(|(_, ic)| ic.is_some()) { 56.0 } else { 48.0 };
        for (i, &(label, icon)) in items.iter().enumerate() {
            let row = Rect {
                x: bounds.x,
                y: bounds.y + 8.0 + i as f32 * row_h,
                width: bounds.width,
                height: row_h,
            };
            let is_sel = *selected == i;
            let r = if let Some(ic) = icon {
                self.material_drawer_item(id.wrapping_add(i as u64), row, label, ic, is_sel, true)
            } else {
                self.navigation_item(id.wrapping_add(i as u64), row, label, is_sel, true)
            };
            if r.clicked && !is_sel {
                *selected = i;
                changed = true;
            }
        }
        changed
    }
}

#[cfg(test)]
#[path = "navigation.spec.rs"]
mod tests;
