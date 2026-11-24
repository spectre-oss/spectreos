//! Hierarchical Material TreeView widget with arbitrary nesting depth.
use crate::*;

/// Flat pre-order node descriptor for `Frame::tree`.
#[derive(Clone, Copy, Debug)]
pub struct TreeItem<'a> {
    pub id: u64,
    pub label: &'a str,
    pub depth: usize,
    pub has_children: bool,
    pub enabled: bool,
}

/// Caller-owned expansion and selection state for `Frame::tree`.
#[derive(Clone, Debug, Default)]
pub struct TreeState {
    pub expanded: HashSet<u64>,
    pub selected: Option<u64>,
}

impl Frame<'_> {
    /// Material TreeView supporting arbitrary depth, expand/collapse toggles, and item selection.
    /// Returns the clicked item's ID if any node was activated.
    pub fn tree(&mut self, id: u64, items: &[TreeItem<'_>], state: &mut TreeState) -> Option<u64> {
        let t = self.theme();
        let mut activated = None;
        let mut hidden_below_depth: Option<usize> = None;

        self.with_gap(0.0, |f| {
            for (idx, item) in items.iter().enumerate() {
                if let Some(min_depth) = hidden_below_depth {
                    if item.depth > min_depth {
                        continue;
                    }
                    hidden_below_depth = None;
                }
                let is_expanded = item.has_children && state.expanded.contains(&item.id);
                if item.has_children && !is_expanded {
                    hidden_below_depth = Some(item.depth);
                }
                let row = f.reserve(40.0);
                let indent = item.depth as f32 * 24.0;
                let selected = state.selected == Some(item.id);
                let row_id = id.wrapping_add(idx as u64 + 1);
                let r = f.target(row_id, row, item.label, item.enabled, Role::ListItem);
                if selected {
                    f.paint_rect(row, 8.0, t.secondary_container);
                }
                f.state_layer(r, 8.0, if selected { t.on_secondary_container } else { t.text });
                f.focus_ring(r);
                let fg = if !r.enabled {
                    t.text.opacity(0.38)
                } else if selected {
                    t.on_secondary_container
                } else {
                    t.text
                };
                if item.has_children {
                    let cx = row.x + 16.0 + indent;
                    let cy = row.y + row.height * 0.5;
                    if is_expanded {
                        f.paint_line([cx - 4.0, cy - 2.0], [cx, cy + 2.5], 1.8, fg);
                        f.paint_line([cx, cy + 2.5], [cx + 4.0, cy - 2.0], 1.8, fg);
                    } else {
                        f.paint_line([cx - 2.0, cy - 4.0], [cx + 2.5, cy], 1.8, fg);
                        f.paint_line([cx + 2.5, cy], [cx - 2.0, cy + 4.0], 1.8, fg);
                    }
                }
                let text_x = row.x + 32.0 + indent;
                f.styled_text(
                    Rect { x: text_x, width: (row.x + row.width - text_x - 12.0).max(0.0), ..row },
                    item.label,
                    fg,
                    TextStyle { size: 14.0, weight: if selected { 500 } else { 400 }, ..TextStyle::default() },
                );
                if r.clicked {
                    state.selected = Some(item.id);
                    if item.has_children {
                        if is_expanded {
                            state.expanded.remove(&item.id);
                        } else {
                            state.expanded.insert(item.id);
                        }
                    }
                    activated = Some(item.id);
                }
            }
        });
        activated
    }
}

#[cfg(test)]
#[path = "tree.spec.rs"]
mod tests;
