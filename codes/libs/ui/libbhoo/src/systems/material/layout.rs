use crate::*;

impl Frame<'_> {
    /// Allocate a styled rectangular region and lay out children inside its padding.
    pub fn boxed(
        &mut self,
        height: f32,
        radius: f32,
        padding: f32,
        background: Color,
        build: impl FnOnce(&mut Self),
    ) {
        let bounds = self.reserve(height.max(0.0));
        self.paint_rect(bounds, radius.max(0.0), background);
        self.within(bounds.inset(padding.max(0.0)), build);
    }
    /// Center a child region up to a maximum width, keeping layout in caller control.
    pub fn container(&mut self, max_width: f32, height: f32, build: impl FnOnce(&mut Self)) {
        let bounds = self.reserve(height);
        let width = max_width.max(0.0).min(bounds.width);
        self.within(
            Rect {
                x: bounds.x + (bounds.width - width) / 2.0,
                width,
                ..bounds
            },
            build,
        );
    }
    /// Arrange children vertically using the given shared spacing.
    pub fn stack(&mut self, count: usize, spacing: f32, mut build: impl FnMut(&mut Self, usize)) {
        self.with_gap(spacing, |f| {
            for index in 0..count {
                build(f, index);
            }
        });
    }
    /// Lay out a bounded collection in equal-width columns with a shared gap.
    pub fn grid(
        &mut self,
        count: usize,
        columns: usize,
        row_height: f32,
        gap: f32,
        mut build: impl FnMut(&mut Self, usize),
    ) {
        if count == 0 || columns == 0 {
            return;
        }
        self.with_gap(gap, |f| {
            let rows = count.div_ceil(columns);
            for row in 0..rows {
                f.row(row_height, columns, |f, column| {
                    let index = row * columns + column;
                    if index < count {
                        build(f, index);
                    }
                });
            }
        });
    }
    pub fn reserve(&mut self, height: f32) -> Rect {
        self.allocate(height)
    }
    pub fn paragraph(
        &mut self,
        bounds: Rect,
        text: &str,
        color: Color,
        style: TextStyle,
        line_height: f32,
    ) {
        let metrics = self.ui.text_metrics;
        let mut index = 0;
        metrics.lines(text, bounds.width, style, |line| {
            let y = bounds.y + index as f32 * line_height;
            if y < bounds.y + bounds.height {
                self.styled_text(
                    Rect {
                        y,
                        height: line_height,
                        ..bounds
                    },
                    line,
                    color,
                    style,
                );
            }
            index += 1;
        });
    }
    pub fn with_gap(&mut self, gap: f32, build: impl FnOnce(&mut Self)) {
        let previous = self.gap;
        self.gap = gap.max(0.0);
        build(self);
        self.gap = previous;
    }
    pub fn row(&mut self, height: f32, columns: usize, mut build: impl FnMut(&mut Self, usize)) {
        if columns == 0 {
            return;
        }
        let b = self.allocate(height);
        let gap = self.gap.min(b.width / columns as f32);
        let width = ((b.width - gap * (columns - 1) as f32) / columns as f32).max(0.0);
        for i in 0..columns {
            self.within(
                Rect {
                    x: b.x + i as f32 * (width + gap),
                    width,
                    ..b
                },
                |f| build(f, i),
            );
        }
    }
}

#[cfg(test)]
#[path = "layout.spec.rs"]
mod tests;
