/// A single overshoot and return, with zero velocity at both ends.
/// Progress and start are normalized, while amplitude is in the caller's units.
pub(crate) fn settle_bounce(progress: f32, start: f32, amplitude: f32) -> f32 {
    if progress <= start || progress >= 1.0 {
        return 0.0;
    }
    let phase = (progress - start) / (1.0 - start);
    amplitude * (std::f32::consts::PI * phase).sin().powi(2)
}
