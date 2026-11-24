use crate::*;

/// Animation mode for `Frame::skeleton_with_options`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonAnimation {
    None,
    #[default]
    Pulse,
    Wave,
}

/// Configuration for `Frame::skeleton_with_options`.
#[derive(Clone, Copy, Debug)]
pub struct SkeletonOptions {
    pub animation: SkeletonAnimation,
    pub radius: f32,
}

impl Default for SkeletonOptions {
    fn default() -> Self {
        Self {
            animation: SkeletonAnimation::Pulse,
            radius: 4.0,
        }
    }
}

impl Frame<'_> {
    /// Paint a noninteractive loading placeholder with optional MUI-style pulse.
    pub fn skeleton(&mut self, bounds: Rect, radius: f32, animated: bool) {
        self.skeleton_with_options(
            bounds,
            SkeletonOptions {
                animation: if animated { SkeletonAnimation::Pulse } else { SkeletonAnimation::None },
                radius,
            },
        );
    }

    /// Paint a loading placeholder supporting static (`None`), `Pulse`, and MUI `Wave` shimmer.
    pub fn skeleton_with_options(&mut self, bounds: Rect, options: SkeletonOptions) {
        let visible = bounds.intersect(self.clip);
        if visible.width <= 0.0 || visible.height <= 0.0 {
            return;
        }
        let anim = if self.reduced_motion() {
            SkeletonAnimation::None
        } else {
            options.animation
        };
        match anim {
            SkeletonAnimation::None => {
                self.rect(bounds, options.radius, self.ui.theme.text.opacity(0.11));
            }
            SkeletonAnimation::Pulse => {
                self.ui.continuous = true;
                let phase = ((self.ui.time - 0.5).max(0.0) % 2.0) as f32 / 2.0;
                let half = if phase < 0.5 { phase * 2.0 } else { (1.0 - phase) * 2.0 };
                let opacity = 1.0 - 0.6 * crate::timing::Easing::Cubic([0.42, 0.0, 0.58, 1.0]).sample(half);
                self.rect(bounds, options.radius, self.ui.theme.text.opacity(0.11 * opacity));
            }
            SkeletonAnimation::Wave => {
                self.ui.continuous = true;
                self.rect(bounds, options.radius, self.ui.theme.text.opacity(0.11));
                let phase = ((self.ui.time.max(0.0) % 1.6) / 1.6) as f32;
                let band_w = (bounds.width * 0.45).max(28.0);
                let sweep_x = bounds.x - band_w + (bounds.width + band_w * 2.0) * phase;
                self.within(bounds, |f| {
                    f.paint_rect(
                        Rect { x: sweep_x, y: bounds.y, width: band_w, height: bounds.height },
                        options.radius,
                        f.theme().text.opacity(0.08),
                    );
                });
            }
        }
    }
}

#[cfg(test)]
#[path = "skeleton.spec.rs"]
mod tests;
