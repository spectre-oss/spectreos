//! Keyboard shortcut keycap hint widget.
use crate::*;

impl Frame<'_> {
    /// Render a compact keycap shortcut hint row (e.g. `keys = &["Ctrl", "K"]`, `description = "Command palette"`).
    pub fn shortcut_hint(&mut self, keys: &[&str], description: &str) {
        let t = self.theme();
        let row = self.reserve(32.0);
        let key_style = TextStyle {
            size: 12.0,
            weight: 600,
            align: 1,
            family: FontFamily::Monospace,
            ..TextStyle::default()
        };
        let mut x = row.x;
        let max_x = row.x + row.width;
        for (i, &key) in keys.iter().enumerate() {
            if i > 0 {
                let plus_w = 14.0;
                if x + plus_w > max_x {
                    break;
                }
                self.styled_text(
                    Rect { x, y: row.y, width: plus_w, height: row.height },
                    "+",
                    t.on_surface_variant,
                    TextStyle { size: 12.0, align: 1, ..TextStyle::default() },
                );
                x += plus_w;
            }
            let kw = (self.text_width(key, key_style) + 14.0).max(24.0);
            if x + kw > max_x {
                break;
            }
            let cap = Rect { x, y: row.y + 4.0, width: kw, height: 24.0 };
            self.paint_rect(cap, 6.0, t.surface_highest);
            self.commands.push(DrawCommand::Outline {
                bounds: cap,
                radius: 6.0,
                width: 1.0,
                color: t.outline_variant,
            });
            self.styled_text(cap, key, t.text, key_style);
            x += kw;
        }
        if !description.is_empty() && x + 12.0 < max_x {
            self.styled_text(
                Rect { x: x + 12.0, y: row.y, width: (max_x - x - 12.0).max(0.0), height: row.height },
                description,
                t.on_surface_variant,
                TextStyle { size: 13.0, ..TextStyle::default() },
            );
        }
    }
}

#[cfg(test)]
#[path = "shortcut_hint.spec.rs"]
mod tests;
