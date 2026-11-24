//! Caller-owned lifecycle for transient snackbar messages.
//!
//! Times are monotonic seconds. The host renders `current()` and calls `update`
//! whenever input or the returned deadline changes. No timer or frame loop is
//! allocated by the queue.
use std::collections::{HashSet, VecDeque};

/// Compose Material 3's standard durations; a caller can provide an
/// accessibility-adjusted duration, or an indefinite notice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Duration {
    Short,
    Long,
    Seconds(f64),
    Indefinite,
}

impl Duration {
    fn seconds(self) -> Option<f64> {
        match self {
            Self::Short => Some(4.0),
            Self::Long => Some(10.0),
            Self::Seconds(s) if s.is_finite() && s > 0.0 => Some(s),
            Self::Seconds(_) | Self::Indefinite => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notice {
    /// Unique among current and queued notices. Returned with the result.
    pub id: u64,
    pub message: String,
    pub action: Option<String>,
    /// Optional localized accessible label for a separate close control.
    pub dismiss_label: Option<String>,
    pub duration: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Action,
    Dismissed,
    TimedOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Result {
    pub id: u64,
    pub outcome: Outcome,
}

struct Active {
    notice: Notice,
    remaining: Option<f64>,
    since: f64,
    paused: bool,
}

/// FIFO host. Duplicate IDs are rejected to prevent stale action results from
/// being attributed to a newer notice. `update` returns the next wake deadline.
#[derive(Default)]
pub struct Host {
    current: Option<Active>,
    pending: VecDeque<Notice>,
    ids: HashSet<u64>,
    results: VecDeque<Result>,
}

impl Host {
    pub fn enqueue(&mut self, notice: Notice, now: f64) -> bool {
        if !self.ids.insert(notice.id) {
            return false;
        }
        self.pending.push_back(notice);
        self.start_next(now);
        true
    }

    pub fn current(&self) -> Option<&Notice> {
        self.current.as_ref().map(|active| &active.notice)
    }

    fn start_next(&mut self, now: f64) {
        if self.current.is_none()
            && let Some(notice) = self.pending.pop_front()
        {
            self.current = Some(Active {
                remaining: notice.duration.seconds(),
                notice,
                since: now,
                paused: false,
            });
        }
    }

    fn finish(&mut self, outcome: Outcome, now: f64) {
        if let Some(active) = self.current.take() {
            self.ids.remove(&active.notice.id);
            self.results.push_back(Result {
                id: active.notice.id,
                outcome,
            });
            self.start_next(now);
        }
    }

    /// Pause while an actionable notice is hovered or focused. Call on every
    /// input change and at the returned deadline, before accepting an action.
    pub fn update(&mut self, now: f64, interacting: bool) -> Option<f64> {
        let active = self.current.as_mut()?;
        let elapsed = (now - active.since).max(0.0);
        if !active.paused
            && let Some(remaining) = &mut active.remaining
        {
            *remaining = (*remaining - elapsed).max(0.0);
        }
        active.since = now;
        if active.remaining == Some(0.0) {
            self.finish(Outcome::TimedOut, now);
            return self.update(now, interacting);
        }
        active.paused = interacting;
        if interacting {
            None
        } else {
            active.remaining.map(|remaining| now + remaining)
        }
    }

    /// Only the visible notice can receive an action; an absent action cannot fire.
    pub fn action(&mut self, id: u64, now: f64) -> bool {
        if !self
            .current
            .as_ref()
            .is_some_and(|a| a.notice.id == id && a.notice.action.is_some())
        {
            return false;
        }
        self.finish(Outcome::Action, now);
        true
    }

    /// Dismiss the current notice or remove a queued notice.
    pub fn dismiss(&mut self, id: u64, now: f64) -> bool {
        if self.current.as_ref().is_some_and(|a| a.notice.id == id) {
            self.finish(Outcome::Dismissed, now);
            return true;
        }
        if let Some(index) = self.pending.iter().position(|notice| notice.id == id) {
            self.pending.remove(index);
            self.ids.remove(&id);
            self.results.push_back(Result {
                id,
                outcome: Outcome::Dismissed,
            });
            return true;
        }
        false
    }

    pub fn take_result(&mut self) -> Option<Result> {
        self.results.pop_front()
    }
}

/// Presentation state for a snackbar host. Keep it alongside the queue.
#[derive(Default)]
pub struct Presentation {
    /// Place the action below the message, even when there is room beside it.
    pub action_on_new_line: bool,
    /// Mirror leading/trailing placement for right-to-left interfaces.
    pub right_to_left: bool,
    visible: Option<Notice>,
    entered: f64,
    exiting: Option<(Notice, f64)>,
}

fn action_id(host_id: u64, notice_id: u64) -> u64 {
    (host_id ^ notice_id.wrapping_mul(0x9e3779b97f4a7c15)) & ((1_u64 << 52) - 1)
}
fn dismiss_id(host_id: u64, notice_id: u64) -> u64 {
    action_id(host_id, notice_id) ^ (1_u64 << 50)
}

impl crate::Frame<'_> {
    /// Render a queued snackbar at the bottom of `viewport`. Returns outcomes
    /// through `Host::take_result`; `host_id` must be stable and unique.
    /// A non-focusable status semantic exposes the message to host adapters.
    pub fn snackbar_host(
        &mut self,
        host_id: u64,
        viewport: crate::Rect,
        host: &mut Host,
        presentation: &mut Presentation,
    ) {
        let now = self.time();
        let bounds = host
            .current()
            .map(|notice| notice_layout(self, viewport, notice, presentation.action_on_new_line).0)
            .unwrap_or_default();
        let interacting = host.current().is_some_and(|notice| {
            self.pointer_in(bounds)
                || (notice.action.is_some()
                    && self.focused_id() == Some(action_id(host_id, notice.id)))
                || (notice.dismiss_label.is_some()
                    && self.focused_id() == Some(dismiss_id(host_id, notice.id)))
        });
        if let Some(deadline) = host.update(now, interacting) {
            self.wake_at(deadline);
        }
        let next = host.current().cloned();
        if presentation.visible != next {
            if let Some(previous) = presentation.visible.take() {
                presentation.exiting = Some((previous, now));
            }
            presentation.visible = next;
            presentation.entered = now;
        }
        let reduced = self.reduced_motion();
        if let Some((notice, started)) = presentation.exiting.as_ref() {
            let bounds = notice_layout(self, viewport, notice, presentation.action_on_new_line).0;
            let progress = if reduced {
                1.0
            } else {
                ((now - started).max(0.0) / 0.075).min(1.0) as f32
            };
            if progress < 1.0 {
                let alpha =
                    1.0 - crate::timing::Easing::Cubic([0.4, 0.0, 1.0, 1.0]).sample(progress);
                paint_notice(
                    self,
                    bounds,
                    notice,
                    host_id,
                    (alpha, 1.0),
                    false,
                    (presentation.action_on_new_line, presentation.right_to_left),
                );
                self.continuous(bounds);
            } else {
                presentation.exiting = None;
            }
        }
        if let Some(notice) = presentation.visible.as_ref() {
            let bounds = notice_layout(self, viewport, notice, presentation.action_on_new_line).0;
            let progress = if reduced {
                1.0
            } else {
                ((now - presentation.entered).max(0.0) / 0.150).min(1.0) as f32
            };
            let alpha = crate::timing::Easing::StandardDecelerate.sample(progress);
            let clicked = paint_notice(
                self,
                bounds,
                notice,
                host_id,
                (alpha, 0.8 + 0.2 * alpha),
                true,
                (presentation.action_on_new_line, presentation.right_to_left),
            );
            if progress < 1.0 {
                self.continuous(bounds);
            }
            let finished = match clicked {
                Some(Outcome::Action) => host.action(notice.id, now),
                Some(Outcome::Dismissed) => host.dismiss(notice.id, now),
                _ => false,
            };
            if finished {
                self.wake_at(now + 0.001);
            } else {
                self.semantic(
                    crate::Response {
                        id: action_id(host_id, notice.id) ^ (1_u64 << 51),
                        bounds,
                        enabled: true,
                        ..crate::Response::default()
                    },
                    crate::Role::Status,
                    &notice.message,
                    false,
                    None,
                    None,
                    None,
                );
            }
        }
    }
}

fn message_lines(frame: &crate::Frame<'_>, text: &str, width: f32) -> Vec<String> {
    let style = crate::TextStyle {
        size: 14.0,
        ..crate::TextStyle::default()
    };
    let mut lines = Vec::new();
    let space_width = frame.text_width(" ", style);
    for paragraph in text.split('\n') {
        let mut line = String::new();
        let mut line_width = 0.0;
        for word in paragraph.split_whitespace() {
            let word_width = frame.text_width(word, style);
            if !line.is_empty() && line_width + space_width + word_width > width {
                lines.push(std::mem::take(&mut line));
                line_width = 0.0;
            }
            if !line.is_empty() {
                line.push(' ');
                line_width += space_width;
            }
            for ch in word.chars() {
                let mut buffer = [0; 4];
                let char_width = frame.text_width(ch.encode_utf8(&mut buffer), style);
                if !line.is_empty() && line_width + char_width > width {
                    lines.push(std::mem::take(&mut line));
                    line_width = 0.0;
                }
                line.push(ch);
                line_width += char_width;
            }
        }
        lines.push(line);
    }
    lines
}

fn action_width(frame: &crate::Frame<'_>, notice: &Notice, width: f32) -> f32 {
    notice.action.as_ref().map_or(0.0, |label| {
        let style = crate::TextStyle {
            size: 14.0,
            weight: 500,
            letter_spacing: 0.096,
            ..crate::TextStyle::default()
        };
        (frame.text_width(label, style) + 32.0).max(72.0).min(width)
    })
}

fn notice_layout(
    frame: &crate::Frame<'_>,
    viewport: crate::Rect,
    notice: &Notice,
    force_stacked: bool,
) -> (crate::Rect, f32, Vec<String>, bool) {
    let width = (viewport.width - 16.0).clamp(0.0, 672.0);
    let dismiss_width = if notice.dismiss_label.is_some() {
        48.0_f32.min(width)
    } else {
        0.0
    };
    let action_width = action_width(frame, notice, (width - dismiss_width).max(0.0));
    let control_width = action_width + dismiss_width;
    let stacked = control_width > 0.0 && (force_stacked || width - control_width - 32.0 < 96.0);
    let lines = message_lines(
        frame,
        &notice.message,
        (width - if stacked { 0.0 } else { control_width } - 32.0).max(0.0),
    );
    let height = (28.0 + 20.0 * lines.len() as f32 + if stacked { 40.0 } else { 0.0 })
        .min(viewport.height.max(0.0));
    let bounds = crate::Rect {
        x: viewport.x + (viewport.width - width) * 0.5,
        y: viewport.y + (viewport.height - height - 8.0).max(0.0),
        width,
        height,
    };
    (bounds, action_width, lines, stacked)
}

fn paint_notice(
    frame: &mut crate::Frame<'_>,
    bounds: crate::Rect,
    notice: &Notice,
    host_id: u64,
    motion: (f32, f32),
    interactive: bool,
    options: (bool, bool),
) -> Option<Outcome> {
    let (alpha, scale) = motion;
    let (force_stacked, right_to_left) = options;
    let theme = frame.theme();
    let dismiss_width = if notice.dismiss_label.is_some() {
        48.0_f32.min(bounds.width)
    } else {
        0.0
    };
    let action_width = action_width(frame, notice, (bounds.width - dismiss_width).max(0.0));
    let control_width = action_width + dismiss_width;
    let stacked =
        control_width > 0.0 && (force_stacked || bounds.width - control_width - 32.0 < 96.0);
    let lines = message_lines(
        frame,
        &notice.message,
        (bounds.width - if stacked { 0.0 } else { control_width } - 32.0).max(0.0),
    );
    let action_bounds = crate::Rect {
        x: if right_to_left {
            bounds.x + dismiss_width + (8.0_f32.min(action_width))
        } else {
            bounds.x + bounds.width - control_width
        },
        y: bounds.y
            + if stacked {
                (bounds.height - 44.0).max(0.0)
            } else {
                (bounds.height - 40.0).max(0.0) * 0.5
            },
        width: (action_width - 8.0).max(0.0),
        height: 40.0_f32.min(bounds.height),
    };
    let dismiss_bounds = crate::Rect {
        x: if right_to_left {
            bounds.x
        } else {
            bounds.x + bounds.width - dismiss_width
        },
        width: dismiss_width,
        ..action_bounds
    };
    let mut clicked = None;
    frame.with_opacity(alpha, |frame| {
        frame.with_scale(
            [
                bounds.x + bounds.width * 0.5,
                bounds.y + bounds.height * 0.5,
            ],
            scale,
            |frame| {
                frame.elevation(bounds, 4.0, 6);
                frame.paint_rect(bounds, 4.0, theme.inverse_surface);
                let message_bounds = crate::Rect {
                    x: bounds.x
                        + if right_to_left && !stacked {
                            control_width
                        } else {
                            0.0
                        },
                    width: (bounds.width - if stacked { 0.0 } else { control_width }).max(0.0),
                    height: (bounds.height - if stacked { 44.0 } else { 0.0 }).max(0.0),
                    ..bounds
                };
                let style = crate::TextStyle {
                    size: 14.0,
                    padding: 16.0,
                    align: if right_to_left { 2 } else { 0 },
                    ..crate::TextStyle::default()
                };
                if lines.len() == 1 {
                    frame.styled_text(
                        message_bounds,
                        &notice.message,
                        theme.inverse_on_surface,
                        style,
                    );
                } else {
                    let count = lines.len();
                    frame.within(message_bounds, |frame| {
                        for (i, line) in lines.iter().enumerate() {
                            frame.styled_text(
                                crate::Rect {
                                    y: bounds.y
                                        + (message_bounds.height - count as f32 * 20.0).max(0.0)
                                            * 0.5
                                        + i as f32 * 20.0,
                                    height: 20.0,
                                    ..message_bounds
                                },
                                line,
                                theme.inverse_on_surface,
                                style,
                            );
                        }
                    });
                }
                if let Some(label) = &notice.action {
                    if interactive {
                        let response = frame.target(
                            action_id(host_id, notice.id),
                            action_bounds,
                            label,
                            true,
                            crate::Role::Button,
                        );
                        frame.state_layer(response, 20.0, theme.inverse_primary);
                        frame.focus_ring(response);
                        if response.clicked {
                            clicked = Some(Outcome::Action);
                        }
                    }
                    let style = crate::TextStyle {
                        size: 14.0,
                        weight: 500,
                        letter_spacing: 0.096,
                        align: 1,
                        ..crate::TextStyle::default()
                    };
                    if frame.text_width(label, style) > action_bounds.width {
                        frame.within(action_bounds, |frame| {
                            frame.styled_text(action_bounds, label, theme.inverse_primary, style)
                        });
                    } else {
                        frame.styled_text(action_bounds, label, theme.inverse_primary, style);
                    }
                }
                if let Some(label) = &notice.dismiss_label {
                    if interactive {
                        let response = frame.target(
                            dismiss_id(host_id, notice.id),
                            dismiss_bounds,
                            label,
                            true,
                            crate::Role::Button,
                        );
                        frame.state_layer(response, 24.0, theme.inverse_on_surface);
                        frame.focus_ring(response);
                        if response.clicked {
                            clicked = Some(Outcome::Dismissed);
                        }
                    }
                    let size = 24.0_f32
                        .min(dismiss_bounds.width)
                        .min(dismiss_bounds.height);
                    frame.paint_material_icon(
                        crate::Rect {
                            x: dismiss_bounds.x + (dismiss_bounds.width - size) * 0.5,
                            y: dismiss_bounds.y + (dismiss_bounds.height - size) * 0.5,
                            width: size,
                            height: size,
                        },
                        crate::MaterialIcon::Close,
                        theme.inverse_on_surface,
                    );
                }
            },
        )
    });
    clicked
}

#[cfg(test)]
#[path = "snackbar.spec.rs"]
mod tests;
