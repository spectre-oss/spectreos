//! Material DataTable & SortHeader widget.
use crate::*;

/// Sort direction for a `DataTable` column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortDirection {
    #[default]
    None,
    Ascending,
    Descending,
}

/// Caller-owned sort state for `Frame::data_table`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableSortState {
    pub column: Option<usize>,
    pub direction: SortDirection,
}

impl TableSortState {
    /// Cycle `None -> Ascending -> Descending -> None` when clicking `col`.
    pub fn toggle(&mut self, col: usize) {
        if self.column != Some(col) || self.direction == SortDirection::None {
            self.column = Some(col);
            self.direction = SortDirection::Ascending;
        } else if self.direction == SortDirection::Ascending {
            self.direction = SortDirection::Descending;
        } else {
            self.column = None;
            self.direction = SortDirection::None;
        }
    }
}

/// Column definition for `Frame::data_table`.
#[derive(Clone, Copy, Debug)]
pub struct TableColumn<'a> {
    pub label: &'a str,
    /// Relative weight fraction (normalized across columns).
    pub weight: f32,
    pub sortable: bool,
    pub align_end: bool,
}

impl Frame<'_> {
    /// Material DataTable with clickable sortable column headers, sort direction arrows,
    /// and 1dp row dividers. Returns `true` when `sort` changed.
    pub fn data_table(
        &mut self,
        id: u64,
        columns: &[TableColumn<'_>],
        row_count: usize,
        sort: &mut TableSortState,
        mut cell_text: impl FnMut(usize, usize) -> String,
    ) -> bool {
        if columns.is_empty() {
            return false;
        }
        let t = self.theme();
        let mut changed = false;
        let weight_sum: f32 = columns.iter().map(|c| c.weight.max(0.01)).sum();
        let mut header_rect = Rect::default();
        self.with_gap(0.0, |f| header_rect = f.reserve(56.0));

        let mut x = header_rect.x;
        for (i, col) in columns.iter().enumerate() {
            let w = if i + 1 == columns.len() {
                (header_rect.x + header_rect.width - x).max(0.0)
            } else {
                header_rect.width * (col.weight.max(0.01) / weight_sum)
            };
            let cell = Rect { x, width: w, ..header_rect };
            x += w;
            if col.sortable {
                let r = self.target(id.wrapping_add(i as u64), cell, col.label, true, Role::Button);
                self.state_layer(r, 4.0, t.on_surface_variant);
                self.focus_ring(r);
                if r.clicked {
                    sort.toggle(i);
                    changed = true;
                }
            }
            let align = if col.align_end { 2 } else { 0 };
            self.styled_text(
                cell,
                col.label,
                t.text,
                TextStyle { size: 14.0, weight: 500, align, padding: 16.0, letter_spacing: 0.1, ..TextStyle::default() },
            );
            if sort.column == Some(i) && sort.direction != SortDirection::None {
                let label_w = self.text_width(col.label, TextStyle::LABEL);
                let ax = if col.align_end {
                    (cell.x + cell.width - 22.0 - label_w).max(cell.x + 8.0)
                } else {
                    (cell.x + 22.0 + label_w).min(cell.x + cell.width - 8.0)
                };
                let ay = cell.y + cell.height * 0.5;
                let dir = if sort.direction == SortDirection::Descending { 1.0 } else { -1.0 };
                self.paint_line([ax, ay - 5.0], [ax, ay + 5.0], 1.5, t.on_surface_variant);
                self.paint_line([ax - 3.0, ay + dir * 2.0], [ax, ay + dir * 5.0], 1.5, t.on_surface_variant);
                self.paint_line([ax, ay + dir * 5.0], [ax + 3.0, ay + dir * 2.0], 1.5, t.on_surface_variant);
            }
        }
        self.paint_rect(
            Rect { y: header_rect.y + header_rect.height - 1.0, height: 1.0, ..header_rect },
            0.0,
            t.outline_variant,
        );
        self.with_gap(0.0, |f| {
            for row in 0..row_count {
                let b = f.reserve(52.0);
                let mut rx = b.x;
                for (c, col) in columns.iter().enumerate() {
                    let w = if c + 1 == columns.len() {
                        (b.x + b.width - rx).max(0.0)
                    } else {
                        b.width * (col.weight.max(0.01) / weight_sum)
                    };
                    let cell = Rect { x: rx, width: w, ..b };
                    rx += w;
                    let text = cell_text(row, c);
                    f.styled_text(
                        cell,
                        &text,
                        t.text,
                        TextStyle {
                            size: 14.0,
                            align: if col.align_end { 2 } else { 0 },
                            padding: 16.0,
                            letter_spacing: 0.25,
                            ..TextStyle::default()
                        },
                    );
                }
                if row + 1 < row_count {
                    f.paint_rect(Rect { y: b.y + b.height - 1.0, height: 1.0, ..b }, 0.0, t.outline_variant);
                }
            }
        });
        changed
    }
}

#[cfg(test)]
#[path = "data_table.spec.rs"]
mod tests;
