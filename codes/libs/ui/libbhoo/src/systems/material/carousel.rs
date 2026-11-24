use crate::*;

/// Supported carousel layouts. Material keyline parity remains in progress.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselLayout {
    #[default]
    Uncontained,
    FullScreen,
    CenteredHero,
    MultiBrowse,
}

#[derive(Clone, Copy, Debug)]
pub struct CarouselOptions {
    pub layout: CarouselLayout,
    pub item_width: f32,
    pub spacing: f32,
    pub snap: bool,
    pub rtl: bool,
    pub enabled: bool,
}
impl Default for CarouselOptions {
    fn default() -> Self {
        Self {
            layout: CarouselLayout::Uncontained,
            item_width: 220.0,
            spacing: 8.0,
            snap: true,
            rtl: false,
            enabled: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CarouselItem<'a> {
    pub label: &'a str,
    pub image: Option<&'a Image>,
    pub enabled: bool,
}

#[derive(Default)]
pub struct CarouselState {
    offset: f32,
    from: f32,
    target: f32,
    started: f64,
    animating: bool,
    index: usize,
    max_index: usize,
    pending: Option<usize>,
    pending_focus: Option<usize>,
    drag_origin: Option<f32>,
    drag_initial: f32,
    dragging: bool,
    step: f32,
    maximum: f32,
}
impl CarouselState {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn can_previous(&self) -> bool {
        self.index > 0
    }
    pub fn can_next(&self) -> bool {
        self.index < self.max_index
    }
    pub fn go_to(&mut self, index: usize) {
        self.pending = Some(index);
    }
    pub fn next(&mut self) {
        self.go_to(self.index.saturating_add(1).min(self.max_index));
    }
    pub fn previous(&mut self) {
        self.go_to(self.index.saturating_sub(1));
    }
}

// Approximate start-aligned multi-browse keylines. Centers and sizes are
// linearly interpolated so adjacent masks remain separated during dragging.
fn multi_browse_keyline(index: i32, large: f32, large_count: usize, medium: f32, small: f32, spacing: f32) -> (f32, f32) {
    if index < 0 {
        return (-spacing - small * 0.5 + (index + 1) as f32 * (small + spacing), small);
    }
    let large_count = large_count as i32;
    if index < large_count { return (index as f32 * (large + spacing) + large * 0.5, large); }
    let before = large_count as f32 * (large + spacing);
    if medium > 0.0 && index == large_count { return (before + medium * 0.5, medium); }
    let first_small = large_count + i32::from(medium > 0.0);
    let before = before + if medium > 0.0 { medium + spacing } else { 0.0 };
    (before + small * 0.5 + (index - first_small) as f32 * (small + spacing), small)
}

fn multi_browse_sample(relative: f32, large: f32, large_count: usize, medium: f32, small: f32, spacing: f32) -> (f32, f32) {
    let left = relative.floor() as i32;
    let t = relative - left as f32;
    let a = multi_browse_keyline(left, large, large_count, medium, small, spacing);
    let b = multi_browse_keyline(left + 1, large, large_count, medium, small, spacing);
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

impl Frame<'_> {
    /// A bounded, virtualized uncontained or full-screen carousel. Items are
    /// independently actionable; a drag cancels the pressed item's activation.
    pub fn carousel(
        &mut self,
        id: u64,
        bounds: Rect,
        items: &[CarouselItem<'_>],
        state: &mut CarouselState,
        options: CarouselOptions,
    ) -> Option<usize> {
        self.carousel_named(id, bounds, items, state, options, "Carousel")
    }

    /// A carousel with a caller-provided accessible region name.
    pub fn carousel_named(
        &mut self,
        id: u64,
        bounds: Rect,
        items: &[CarouselItem<'_>],
        state: &mut CarouselState,
        options: CarouselOptions,
        label: &str,
    ) -> Option<usize> {
        let count = items.len();
        if count == 0
            || count > 4096
            || id > u64::MAX - count as u64
            || !bounds.x.is_finite()
            || !bounds.y.is_finite()
            || !bounds.width.is_finite()
            || !bounds.height.is_finite()
            || bounds.width <= 0.0
            || bounds.height <= 0.0
            || !options.item_width.is_finite()
            || !options.spacing.is_finite()
        {
            return None;
        }
        let visible = bounds.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            if state.animating {
                state.offset = state.target;
            }
            state.animating = false;
            state.drag_origin = None;
            return None;
        }
        let enabled = self.enabled && options.enabled;
        let hero = options.layout == CarouselLayout::CenteredHero;
        let browse = options.layout == CarouselLayout::MultiBrowse;
        let spacing = if options.layout == CarouselLayout::FullScreen {
            0.0
        } else {
            let gap = options.spacing.clamp(0.0, 64.0);
            if hero || browse { gap.min(bounds.width / 4.0) } else { gap }
        };
        let small = if browse && count > 1 {
            (bounds.width / 8.0).clamp(40.0, 56.0).min(bounds.width / 3.0)
        } else if hero && count >= 3 && bounds.width >= 168.0 {
            (bounds.width / 6.0).clamp(40.0, 56.0)
        } else {
            0.0
        };
        let mut medium = 0.0;
        let mut large_count = 1;
        let width = if options.layout == CarouselLayout::FullScreen || (browse && count == 1) {
            bounds.width
        } else if browse {
            let available = (bounds.width - small - spacing).max(1.0);
            let target = options.item_width.clamp(small.max(1.0), available.max(small.max(1.0)));
            large_count = (((bounds.width - small + spacing) / (target + spacing)).floor() as usize)
                .clamp(1, count - 1);
            let leftover = bounds.width - large_count as f32 * target - (large_count + 1) as f32 * spacing - small;
            if count > large_count + 1 && leftover >= small && leftover < target {
                medium = leftover;
                target
            } else {
                ((bounds.width - small - large_count as f32 * spacing) / large_count as f32).max(1.0)
            }
        } else if hero {
            let neighbor_space = if small > 0.0 {
                2.0 * (small + spacing)
            } else {
                0.0
            };
            (bounds.width - neighbor_space).max(1.0)
        } else {
            options
                .item_width
                .clamp(48.0_f32.min(bounds.width), bounds.width)
        };
        let step = width + spacing;
        let maximum = if hero || browse {
            count.saturating_sub(1) as f32 * step
        } else {
            ((count as f32 * step - spacing) - bounds.width).max(0.0)
        };
        state.max_index = if hero || browse { count - 1 } else { (maximum / step).ceil() as usize };
        state.index = state.index.min(state.max_index);
        if (state.step - step).abs() > 0.001 || (state.maximum - maximum).abs() > 0.001 {
            let relative = if state.step > 0.0 {
                state.offset / state.step
            } else {
                0.0
            };
            state.offset = (relative * step).clamp(0.0, maximum);
            state.target = (state.index as f32 * step).min(maximum);
            if state.animating {
                state.from = state.offset;
                state.started = self.ui.time;
                state.animating = (state.target - state.offset).abs() > 0.01;
            }
            state.drag_origin = None;
            state.dragging = false;
            state.step = step;
            state.maximum = maximum;
        }
        if state
            .pending_focus
            .is_some_and(|index| index >= count || !items[index].enabled)
        {
            state.pending_focus = None;
        }
        state.offset = state.offset.clamp(0.0, maximum);
        if let Some(index) = state.pending.take() {
            state.index = index.min(state.max_index);
            state.from = state.offset;
            state.target = (state.index as f32 * step).min(maximum);
            state.started = self.ui.time;
            state.animating = !self.reduced_motion() && (state.target - state.offset).abs() > 0.01;
            if !state.animating {
                state.offset = state.target;
            }
        }
        if state.animating {
            let progress = ((self.ui.time - state.started) / 0.3).clamp(0.0, 1.0) as f32;
            let eased = crate::timing::Easing::Cubic([0.2, 0.0, 0.0, 1.0]).sample(progress);
            state.offset = (state.from + (state.target - state.from) * eased).clamp(0.0, maximum);
            state.animating = progress < 1.0 && !self.reduced_motion();
            if state.animating {
                self.ui.continuous = true;
            } else {
                state.offset = state.target;
            }
        }
        if enabled && self.input.pressed && self.pointer_in(bounds) {
            state.drag_origin = self.input.pointer.map(|p| p[0]);
            state.drag_initial = state.offset;
            state.dragging = false;
            state.animating = false;
        }
        if let (true, Some(origin), Some(pointer)) =
            (enabled, state.drag_origin, self.input.pointer)
        {
            let distance = if options.rtl {
                pointer[0] - origin
            } else {
                origin - pointer[0]
            };
            if distance.abs() > 4.0 {
                state.dragging = true;
            }
            if state.dragging {
                let next_offset = (state.drag_initial + distance).clamp(0.0, maximum);
                if next_offset != state.offset {
                    self.ui.continuous = true;
                }
                state.offset = next_offset;
                state.index = ((state.offset / step).round() as usize).min(state.max_index);
                self.ui.active = None;
            }
        }
        if self.input.released || self.input.cancelled || self.input.blurred || !enabled {
            if state.dragging {
                self.ui.active = None;
                if options.snap || matches!(options.layout, CarouselLayout::FullScreen | CarouselLayout::CenteredHero | CarouselLayout::MultiBrowse) {
                    state.go_to(state.index);
                    self.ui.continuous = true;
                }
            }
            state.drag_origin = None;
            state.dragging = false;
        }
        if enabled
            && self.pointer_in(bounds)
            && self.input.scroll[1].is_finite()
            && self.input.scroll[1] != 0.0
        {
            if self.input.scroll[1] > 0.0 {
                state.next();
            } else {
                state.previous();
            }
            self.ui.continuous = true;
        }
        self.semantic(
            Response {
                id,
                bounds: visible,
                enabled,
                ..Response::default()
            },
            Role::Carousel,
            label,
            false,
            None,
            Some(state.index as f32 + 1.0),
            Some([1.0, count as f32]),
        );
        let first = if hero || browse {
            ((state.offset / step).floor().max(0.0) as usize).saturating_sub(large_count + 2)
        } else {
            ((state.offset / step).floor() as usize).min(count - 1)
        };
        let last = if hero || browse {
            ((state.offset / step).ceil() as usize + large_count + 3).min(count)
        } else {
            (((state.offset + bounds.width) / step).ceil() as usize + 1).min(count)
        };
        let mut chosen = None;
        self.within(bounds, |f| {
            for (index, item) in items.iter().enumerate().take(last).skip(first) {
                let shift = index as f32 * step - state.offset;
                let item_bounds = if browse {
                    let position = state.offset / step;
                    let relative = index as f32 - position;
                    let start = multi_browse_sample(relative, width, large_count, medium, small, spacing);
                    let end = multi_browse_sample(-relative, width, large_count, medium, small, spacing);
                    let end_mix = (position - count.saturating_sub(2) as f32).clamp(0.0, 1.0);
                    let center = start.0 + (bounds.width - end.0 - start.0) * end_mix;
                    let item_width = start.1 + (end.1 - start.1) * end_mix;
                    let logical_x = center - item_width * 0.5;
                    Rect { x: if options.rtl { bounds.x + bounds.width - logical_x - item_width } else { bounds.x + logical_x }, width: item_width, ..bounds }
                } else if hero {
                    let position = state.offset / step;
                    let relative = index as f32 - position;
                    let distance = relative.abs();
                    let item_width = if small == 0.0 {
                        width
                    } else {
                        width + (small - width) * distance.min(1.0)
                    };
                    let anchor = (bounds.width - width) * 0.5;
                    let start_edge = (1.0 - position).clamp(0.0, 1.0);
                    let end_edge =
                        (1.0 - (count.saturating_sub(1) as f32 - position)).clamp(0.0, 1.0);
                    let center = bounds.width * 0.5 - anchor * start_edge + anchor * end_edge;
                    let neighbor_width = if small == 0.0 { width } else { small };
                    let neighbor_step = (width + neighbor_width) * 0.5 + spacing;
                    let displacement = if distance <= 1.0 {
                        distance * neighbor_step
                    } else {
                        neighbor_step + (distance - 1.0) * (neighbor_width + spacing)
                    };
                    let logical_x = center + relative.signum() * displacement - item_width * 0.5;
                    Rect {
                        x: if options.rtl {
                            bounds.x + bounds.width - logical_x - item_width
                        } else {
                            bounds.x + logical_x
                        },
                        width: item_width,
                        ..bounds
                    }
                } else {
                    Rect {
                        x: if options.rtl {
                            bounds.x + bounds.width - width - shift
                        } else {
                            bounds.x + shift
                        },
                        width,
                        ..bounds
                    }
                };
                let visible_item = item_bounds.intersect(f.clip);
                if visible_item.width <= 0.0 || visible_item.height <= 0.0 {
                    continue;
                }
                let r = f.target(
                    id + index as u64 + 1,
                    item_bounds,
                    item.label,
                    enabled && item.enabled,
                    Role::CarouselItem,
                );
                if let Some(node) = f.ui.semantics.last_mut() {
                    node.value = Some(index as f32 + 1.0);
                    node.range = Some([1.0, count as f32]);
                }
                if r.clicked {
                    chosen = Some(index);
                }
                if let Some(image) = item.image {
                    if hero || browse {
                        // Fit content to the large item first, then reveal the
                        // centered portion covered by the current mask. This
                        // preserves content scale as the mask changes.
                        let content = Rect { width, ..item_bounds };
                        if let Some((_, mut uv)) = crate::media::fit_image(content, image.width(), image.height(), ImageFit::Cover)
                            && item_bounds.intersect(f.clip).width > 0.0
                            && item_bounds.intersect(f.clip).height > 0.0
                        {
                            let fraction = (item_bounds.width / width).clamp(0.0, 1.0);
                            let inset = (uv[2] - uv[0]) * (1.0 - fraction) * 0.5;
                            uv[0] += inset;
                            uv[2] -= inset;
                            f.commands.push(DrawCommand::Image { bounds: item_bounds, uv, radius: 16.0, color: Color(255, 255, 255, 255), image: image.clone() });
                        }
                    } else {
                        f.paint_image(item_bounds, image, ImageFit::Cover, 16.0);
                    }
                } else {
                    f.paint_rect(
                        item_bounds,
                        16.0,
                        if index % 2 == 0 {
                            f.ui.theme.secondary
                        } else {
                            f.ui.theme.primary_container
                        },
                    );
                }
                f.state_layer(r, 16.0, f.ui.theme.text);
                let text_width = f.text_width(item.label, TextStyle { size: 14.0, ..TextStyle::default() });
                if !(hero || browse) || item_bounds.width >= text_width + 24.0 {
                    let label = Rect {
                        y: item_bounds.y + item_bounds.height - 40.0,
                        height: 40.0,
                        ..item_bounds
                    };
                    f.commands
                        .push(DrawCommand::PushClip(label.intersect(f.clip)));
                    f.paint_rect(item_bounds, 16.0, f.ui.theme.background.opacity(0.8));
                    f.commands.push(DrawCommand::PopClip);
                    f.styled_text(
                        label,
                        item.label,
                        f.ui.theme.text.opacity(if r.enabled { 1.0 } else { 0.38 }),
                        TextStyle {
                            size: 14.0,
                            padding: 12.0,
                            ..TextStyle::default()
                        },
                    );
                }
                f.focus_ring(r);
            }
        });
        if enabled
            && !self.key_consumed
            && let Some(key) = self.input.key
            && key.pressed
            && matches!(key.key, Key::Left | Key::Right | Key::Home | Key::End)
            && let Some(current) = self
                .ui
                .focused
                .and_then(|focus| focus.checked_sub(id + 1))
                .filter(|index| *index < count as u64)
        {
            let next = match key.key {
                Key::Home => (0..count).find(|&index| items[index].enabled),
                Key::End => (0..count).rev().find(|&index| items[index].enabled),
                _ if (key.key == Key::Right) != options.rtl => {
                    ((current as usize + 1)..count).find(|&index| items[index].enabled)
                }
                _ => (0..current as usize).rev().find(|&index| items[index].enabled),
            };
            if let Some(next) = next {
                state.go_to(next);
                state.pending_focus = Some(next);
                self.key_consumed = true;
                self.ui.continuous = true;
            }
        }
        if let Some(index) = state.pending_focus
            && !state.animating
            && state.pending.is_none()
            && self
                .ui
                .semantics
                .iter()
                .any(|n| n.id == id + index as u64 + 1 && n.enabled)
        {
            self.request_focus(id + index as u64 + 1);
            state.pending_focus = None;
            self.ui.continuous = true;
        }
        chosen
    }
}

#[cfg(test)]
#[path = "carousel.spec.rs"]
mod tests;
