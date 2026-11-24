//! Interactive GPU Frequency Boost Service (platform-agnostic).

use std::time::Duration;

/// Platform-agnostic backend contract for interactive GPU clock boosting during touch/animations.
pub trait GpuBoostBackend: Send {
    fn boost(&mut self, hold: Duration);
    fn release(&mut self, force: bool);
    fn current_mhz(&self) -> u32;
}

/// No-op backend for desktop previews, WASM, and unit tests.
pub struct NullGpuBoostBackend;

impl GpuBoostBackend for NullGpuBoostBackend {
    fn boost(&mut self, _hold: Duration) {}
    fn release(&mut self, _force: bool) {}
    fn current_mhz(&self) -> u32 {
        0
    }
}

/// Domain service wrapping GPU interactive boost requests.
pub struct GpuBoostService {
    backend: Box<dyn GpuBoostBackend>,
}

impl GpuBoostService {
    pub fn new(backend: Box<dyn GpuBoostBackend>) -> Self {
        Self { backend }
    }

    pub fn boost(&mut self, hold: Duration) {
        self.backend.boost(hold);
    }

    pub fn release(&mut self, force: bool) {
        self.backend.release(force);
    }

    pub fn current_mhz(&self) -> u32 {
        self.backend.current_mhz()
    }
}
