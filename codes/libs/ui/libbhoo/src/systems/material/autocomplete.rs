//! Material Autocomplete field with filtered suggestion dropdown.
use crate::*;

/// Configuration for `Frame::autocomplete`.
#[derive(Clone, Copy, Debug)]
pub struct AutocompleteOptions {
    pub filled: bool,
    pub invalid: bool,
    pub max_visible: usize,
}

impl Default for AutocompleteOptions {
    fn default() -> Self {
        Self {
            filled: true,
            invalid: false,
            max_visible: 5,
        }
    }
}

impl Frame<'_> {
    /// Material Autocomplete text field with an elevated filtered suggestion list.
    /// Returns `(field_response, selected_option_index)` when a suggestion is clicked.
    pub fn autocomplete(
        &mut self,
        id: u64,
        label: &str,
        query: &str,
        suggestions: &[&str],
        open: &mut bool,
        options: AutocompleteOptions,
    ) -> (Response, Option<usize>) {
        let r = self.text_field(id, label, 56.0, !query.is_empty(), options.invalid, options.filled);
        if r.focused || r.clicked {
            *open = true;
        }
        if self.pressed_key(Key::Escape) {
            *open = false;
        }
        let mut chosen = None;
        if *open {
            let needle = query.to_ascii_lowercase();
            let matched: Vec<(usize, &str)> = suggestions
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, s)| needle.is_empty() || s.to_ascii_lowercase().contains(&needle))
                .take(options.max_visible.max(1))
                .collect();
            if !matched.is_empty() {
                let t = self.theme();
                let panel_h = matched.len() as f32 * 48.0 + 16.0;
                let panel = self.reserve(panel_h);
                self.elevation(panel, 8.0, 2);
                self.paint_rect(panel, 8.0, t.surface_high);
                self.within(panel, |f| {
                    f.with_gap(0.0, |f| {
                        f.reserve(8.0);
                        for (pos, &(orig_idx, text)) in matched.iter().enumerate() {
                            let item_r = f.list_item(id.wrapping_add(100 + pos as u64), text, false, true);
                            if item_r.clicked {
                                chosen = Some(orig_idx);
                                *open = false;
                            }
                        }
                    });
                });
            }
        }
        (r, chosen)
    }
}

#[cfg(test)]
#[path = "autocomplete.spec.rs"]
mod tests;
