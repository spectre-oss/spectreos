//! Material persistent inline Banner with animated reveal, optional leading icon, and action buttons.
use crate::*;

/// Configuration for `Frame::banner_with_options`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BannerOptions {
    pub leading_icon: Option<MaterialIcon>,
}

impl Frame<'_> {
    /// Material inline Banner that reveals/collapses smoothly and returns the clicked action index (`0..actions.len()`).
    pub fn banner(
        &mut self,
        id: u64,
        open: bool,
        message: &str,
        actions: &[&str],
    ) -> Option<usize> {
        self.banner_with_options(id, open, message, actions, BannerOptions::default())
    }

    /// Material inline Banner with optional leading `MaterialIcon` and action buttons.
    pub fn banner_with_options(
        &mut self,
        id: u64,
        open: bool,
        message: &str,
        actions: &[&str],
        options: BannerOptions,
    ) -> Option<usize> {
        let t = self.theme();
        let mut clicked = None;
        let height = if actions.is_empty() { 56.0 } else { 96.0 };
        self.reveal(id, open, height, |f| {
            let b = f.reserve(height);
            f.paint_rect(b, 8.0, t.surface_low);
            f.paint_rect(Rect { y: b.y + b.height - 1.0, height: 1.0, ..b }, 0.0, t.outline_variant);
            let inner = Rect {
                x: b.x + 16.0,
                y: b.y + 8.0,
                width: (b.width - 32.0).max(0.0),
                height: (b.height - 16.0).max(0.0),
            };
            f.within(inner, |f| {
                let msg_rect = f.reserve(36.0);
                let text_rect = if let Some(icon) = options.leading_icon {
                    let badge = Rect { x: msg_rect.x, y: msg_rect.y, width: 36.0, height: 36.0 };
                    f.paint_rect(badge, 18.0, t.primary_container);
                    f.paint_material_icon(badge.inset(6.0), icon, t.on_primary_container);
                    Rect {
                        x: msg_rect.x + 48.0,
                        width: (msg_rect.width - 48.0).max(0.0),
                        ..msg_rect
                    }
                } else {
                    msg_rect
                };
                f.paragraph(text_rect, message, t.text, TextStyle { size: 14.0, ..TextStyle::default() }, 18.0);
                if !actions.is_empty() {
                    let cols = actions.len();
                    f.row(40.0, cols, |f, i| {
                        let r = f.button_with(
                            id.wrapping_add(10 + i as u64),
                            actions[i],
                            Options {
                                variant: if i + 1 == cols { ButtonVariant::Filled } else { ButtonVariant::Text },
                                ..Options::default()
                            },
                        );
                        if r.clicked {
                            clicked = Some(i);
                        }
                    });
                }
            });
        });
        clicked
    }
}

#[cfg(test)]
#[path = "banner.spec.rs"]
mod tests;
