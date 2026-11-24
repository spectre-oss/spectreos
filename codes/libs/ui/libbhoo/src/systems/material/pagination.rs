use crate::*;

/// Configuration for `Frame::pagination_with_options`.
#[derive(Clone, Copy, Debug)]
pub struct PaginationOptions {
    pub boundary_count: usize,
    pub sibling_count: usize,
    pub show_first_last: bool,
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            boundary_count: 1,
            sibling_count: 1,
            show_first_last: false,
        }
    }
}

/// Configuration for `Frame::page_indicator` (carousel / workspace pager dots).
#[derive(Clone, Copy, Debug)]
pub struct PageIndicatorOptions {
    pub dot_size: f32,
    pub gap: f32,
    pub active_color: Option<Color>,
    pub inactive_color: Option<Color>,
    pub interactive: bool,
}

impl Default for PageIndicatorOptions {
    fn default() -> Self {
        Self {
            dot_size: 6.0,
            gap: 12.0,
            active_color: None,
            inactive_color: None,
            interactive: false,
        }
    }
}

impl Frame<'_> {
    /// Numbered pagination with a bounded number of visible items. Values are one-based.
    pub fn pagination(&mut self, id: u64, page: &mut usize, count: usize, enabled: bool) {
        self.pagination_with_options(id, page, count, enabled, PaginationOptions::default());
    }

    /// Numbered pagination with configurable boundary/sibling counts and optional First/Last buttons.
    pub fn pagination_with_options(
        &mut self,
        id: u64,
        page: &mut usize,
        count: usize,
        enabled: bool,
        options: PaginationOptions,
    ) {
        let count = count.max(1);
        *page = (*page).clamp(1, count);
        let boundary = options.boundary_count.max(1).min(count);
        let sibling = options.sibling_count;
        let mut pages = Vec::new();
        for p in 1..=boundary {
            pages.push(p);
        }
        for p in count.saturating_sub(boundary - 1)..=count {
            pages.push(p);
        }
        if count <= boundary * 2 + sibling * 2 + 3 {
            pages.extend(1..=count);
        }
        for p in page.saturating_sub(sibling).max(1)..=page.saturating_add(sibling).min(count) {
            pages.push(p);
        }
        pages.sort_unstable();
        pages.dedup();
        let mut entries = Vec::with_capacity(pages.len() + 2);
        for p in pages {
            if let Some(Some(previous)) = entries.last().copied() {
                if p.saturating_sub(previous) == 2 {
                    entries.push(Some(previous + 1));
                } else if p.saturating_sub(previous) > 2 {
                    entries.push(None);
                }
            }
            entries.push(Some(p));
        }
        let extra = if options.show_first_last { 4 } else { 2 };
        let row = self.reserve(40.0);
        let total = entries.len() + extra;
        let width = (row.width / total as f32).min(40.0);
        for i in 0..total {
            let bounds = Rect {
                x: row.x + i as f32 * width,
                width,
                ..row
            };
            let first = options.show_first_last && i == 0;
            let previous = if options.show_first_last { i == 1 } else { i == 0 };
            let next = if options.show_first_last { i + 2 == total } else { i + 1 == total };
            let last = options.show_first_last && i + 1 == total;
            let entry_offset = if options.show_first_last { 2 } else { 1 };
            let target = if first {
                Some(1)
            } else if previous {
                Some(page.saturating_sub(1).max(1))
            } else if next {
                Some(page.saturating_add(1).min(count))
            } else if last {
                Some(count)
            } else {
                entries[i - entry_offset]
            };
            let Some(target) = target else {
                self.styled_text(
                    bounds,
                    "…",
                    self.ui.theme.text,
                    TextStyle {
                        size: 14.0,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
                continue;
            };
            let is_nav = first || previous || next || last;
            let selected = !is_nav && target == *page;
            let available = enabled
                && (!(first || previous) || *page > 1)
                && (!(next || last) || *page < count);
            let r = self.interact(id + i as u64, bounds, available);
            let label = if first {
                "First page".to_owned()
            } else if previous {
                "Previous page".to_owned()
            } else if next {
                "Next page".to_owned()
            } else if last {
                "Last page".to_owned()
            } else {
                format!("Page {target}")
            };
            if selected {
                self.paint_rect(
                    bounds.inset(4.0),
                    (width - 8.0).max(0.0) / 2.0,
                    self.ui.theme.text.opacity(0.08),
                );
            }
            self.state_layer(r, width / 2.0, self.ui.theme.text);
            self.focus_ring(r);
            let color = self
                .ui
                .theme
                .text
                .opacity(if r.enabled { 1.0 } else { 0.38 });
            if is_nav {
                let cx = bounds.x + bounds.width / 2.0;
                let cy = bounds.y + bounds.height / 2.0;
                let direction = if first || previous { -1.0 } else { 1.0 };
                let tip = [cx + direction * 2.65, cy];
                self.paint_line([cx - direction * 2.1, cy - 4.6], tip, 1.67, color);
                self.paint_line(tip, [cx - direction * 2.1, cy + 4.6], 1.67, color);
                if first || last {
                    let bar_x = cx + direction * 5.2;
                    self.paint_line([bar_x, cy - 4.8], [bar_x, cy + 4.8], 1.67, color);
                }
            } else {
                self.styled_text(
                    bounds,
                    &target.to_string(),
                    color,
                    TextStyle {
                        size: 14.0,
                        align: 1,
                        ..TextStyle::default()
                    },
                );
            }
            self.semantic(r, Role::Button, &label, selected, None, None, None);
            if r.clicked {
                *page = target;
            }
        }
    }

    /// Angular/Material Paginator bar (`"X – Y of N"` range readout + Previous/Next buttons).
    /// Returns `true` when `*page` changed.
    pub fn paginator(
        &mut self,
        id: u64,
        page: &mut usize,
        page_size: usize,
        total_items: usize,
        enabled: bool,
    ) -> bool {
        let mut size = page_size.max(1);
        self.paginator_with_page_size(id, page, &mut size, &[], total_items, enabled)
    }

    /// Angular/Material Paginator bar with optional page-size selector (`"Items per page: N"`),
    /// range readout (`"X – Y of N"`), and Previous/Next buttons.
    /// Returns `true` when `*page` or `*page_size` changed.
    pub fn paginator_with_page_size(
        &mut self,
        id: u64,
        page: &mut usize,
        page_size: &mut usize,
        page_size_options: &[usize],
        total_items: usize,
        enabled: bool,
    ) -> bool {
        *page_size = (*page_size).max(1);
        let size = *page_size;
        let pages = total_items.div_ceil(size).max(1);
        *page = (*page).min(pages - 1);
        let start = if total_items == 0 { 0 } else { *page * size + 1 };
        let end = ((*page + 1) * size).min(total_items);
        let range_label = format!("{start} – {end} of {total_items}");

        let t = self.theme();
        let bar = self.reserve(56.0);
        let mut changed = false;
        let mut left_used = 0.0;
        if !page_size_options.is_empty() {
            let sz_text = format!("Items per page: {size}");
            let btn_w = (self.text_width(&sz_text, TextStyle { size: 13.0, ..TextStyle::default() }) + 24.0)
                .clamp(96.0, (bar.width * 0.5).max(96.0));
            let sz_rect = Rect { x: bar.x + 8.0, y: bar.y + 8.0, width: btn_w, height: 40.0 };
            let r = self.target(id.wrapping_add(10), sz_rect, &sz_text, enabled, Role::Button);
            let fg = if enabled { t.on_surface_variant } else { t.text.opacity(0.38) };
            self.state_layer(r, 8.0, fg);
            self.focus_ring(r);
            self.styled_text(
                sz_rect,
                &sz_text,
                fg,
                TextStyle { size: 13.0, align: 1, ..TextStyle::default() },
            );
            if r.clicked {
                let pos = page_size_options.iter().position(|&v| v == size).unwrap_or(0);
                *page_size = page_size_options[(pos + 1) % page_size_options.len()].max(1);
                *page = 0;
                changed = true;
            }
            left_used = btn_w + 16.0;
        }

        let btn_w = 40.0;
        let next_rect = Rect { x: bar.x + bar.width - btn_w, y: bar.y + 8.0, width: btn_w, height: btn_w };
        let prev_rect = Rect { x: next_rect.x - btn_w - 4.0, y: bar.y + 8.0, width: btn_w, height: btn_w };
        let label_rect = Rect {
            x: bar.x + left_used + 8.0,
            y: bar.y,
            width: (prev_rect.x - (bar.x + left_used) - 16.0).max(0.0),
            height: bar.height,
        };
        self.styled_text(
            label_rect,
            &range_label,
            if enabled { t.on_surface_variant } else { t.text.opacity(0.38) },
            TextStyle { size: 13.0, align: 2, ..TextStyle::default() },
        );
        for (i, (rect, can, name, dir)) in [
            (prev_rect, enabled && *page > 0, "Previous page", -1.0_f32),
            (next_rect, enabled && *page + 1 < pages, "Next page", 1.0_f32),
        ]
        .into_iter()
        .enumerate()
        {
            let r = self.target(id.wrapping_add(i as u64), rect, name, can, Role::Button);
            let fg = if can { t.on_surface_variant } else { t.text.opacity(0.38) };
            self.state_layer(r, 20.0, fg);
            self.focus_ring(r);
            let cx = rect.x + rect.width * 0.5;
            let cy = rect.y + rect.height * 0.5;
            self.paint_line([cx - dir * 2.5, cy - 4.5], [cx + dir * 2.0, cy], 1.8, fg);
            self.paint_line([cx + dir * 2.0, cy], [cx - dir * 2.5, cy + 4.5], 1.8, fg);
            if r.clicked {
                if i == 0 {
                    *page = page.saturating_sub(1);
                } else {
                    *page = (*page + 1).min(pages - 1);
                }
                changed = true;
            }
        }
        changed
    }

    /// Material/Android pager indicator dots centered within `bounds`.
    /// When `options.interactive` is true, tapping cycles to the next page and returns `Some(next_page)`.
    pub fn page_indicator(
        &mut self,
        id: u64,
        bounds: Rect,
        page_count: usize,
        current: f32,
        options: PageIndicatorOptions,
    ) -> Option<usize> {
        if page_count <= 1 {
            return None;
        }
        let t = self.theme();
        let dot = options.dot_size.max(2.0);
        let gap = options.gap.max(dot + 2.0);
        let total_w = (page_count - 1) as f32 * gap + dot;
        let start_x = bounds.x + (bounds.width - total_w) * 0.5;
        let dot_y = bounds.y + (bounds.height - dot) * 0.5;
        let active_c = options.active_color.unwrap_or(t.primary);
        let inactive_c = options.inactive_color.unwrap_or(t.on_surface_variant.opacity(0.45));

        let mut next = None;
        if options.interactive {
            let r = self.target(id, bounds, "Switch page", true, Role::Button);
            if r.clicked {
                let cur = (current.round().max(0.0) as usize).min(page_count - 1);
                next = Some((cur + 1) % page_count);
            }
        }
        for i in 0..page_count {
            let active = (current - i as f32).abs() < 0.5;
            self.paint_rect(
                Rect { x: start_x + i as f32 * gap, y: dot_y, width: dot, height: dot },
                dot * 0.5,
                if active { active_c } else { inactive_c },
            );
        }
        next
    }
}

#[cfg(test)]
#[path = "pagination.spec.rs"]
mod tests;
