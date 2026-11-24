//! Material Select (dropdown field + anchored menu with keyboard navigation and type-ahead).
use crate::*;

/// Caller-owned state for `Frame::select`.
#[derive(Default)]
pub struct SelectState {
    pub overlay: OverlayState,
    pub anchor: Rect,
    pub trigger_typeahead: MenuTypeaheadState,
    pub menu_typeahead: MenuTypeaheadState,
}

impl Frame<'_> {
    /// Material Select dropdown combining `select_field`, closed-trigger typeahead,
    /// arrow/Home/End selection, and an anchored `OverlayKind::Menu` popup.
    /// Returns `true` when `selected` changed.
    pub fn select(
        &mut self,
        id: u64,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        state: &mut SelectState,
        filled: bool,
    ) -> bool {
        if options.is_empty() {
            return false;
        }
        *selected = (*selected).min(options.len() - 1);
        let mut changed = false;

        let ta_items: Vec<(&str, bool)> = options.iter().map(|&s| (s, true)).collect();
        if !state.overlay.open {
            if let Some(idx) = self.select_typeahead(&mut state.trigger_typeahead, id, *selected, &ta_items)
                && idx != *selected
            {
                *selected = idx;
                changed = true;
            }
        }

        let trigger = self.select_field(id, label, options[*selected], filled);
        if trigger.clicked {
            state.anchor = trigger.bounds;
            state.overlay.open = !state.overlay.open;
            state.menu_typeahead.reset();
        }
        if trigger.focused && !state.overlay.open {
            if self.pressed_key(Key::Down) && *selected + 1 < options.len() {
                *selected += 1;
                changed = true;
            } else if self.pressed_key(Key::Up) && *selected > 0 {
                *selected -= 1;
                changed = true;
            } else if self.pressed_key(Key::Home) && *selected != 0 {
                *selected = 0;
                changed = true;
            } else if self.pressed_key(Key::End) && *selected + 1 != options.len() {
                *selected = options.len() - 1;
                changed = true;
            }
        }

        state.overlay.kind = OverlayKind::Menu;
        state.overlay.style = OverlayStyle::AndroidMenu;
        state.overlay.dismiss_policy = OverlayDismissPolicy::EscapeOrOutside;
        let popup_h = (options.len() as f32 * 48.0 + 16.0).min(256.0);
        let popup = Rect {
            x: state.anchor.x,
            y: state.anchor.y + state.anchor.height + 4.0,
            width: state.anchor.width.max(160.0),
            height: popup_h,
        };
        let count = options.len();
        let mut overlay = std::mem::take(&mut state.overlay);
        self.overlay(id.wrapping_add(50), &mut overlay, popup, false, |f, open| {
            f.menu_typeahead(&mut state.menu_typeahead, id.wrapping_add(100), &ta_items);
            f.with_gap(0.0, |f| {
                f.reserve(8.0);
                for (i, &opt) in options.iter().enumerate() {
                    f.overlay_item(i, count, |f| {
                        let r = f.menu_item_with_selection(
                            id.wrapping_add(100 + i as u64),
                            opt,
                            true,
                            i == *selected,
                            false,
                        );
                        if r.clicked {
                            if *selected != i {
                                *selected = i;
                                changed = true;
                            }
                            *open = false;
                        }
                    });
                }
            });
        });
        state.overlay = overlay;
        changed
    }
}

#[cfg(test)]
#[path = "select.spec.rs"]
mod tests;
