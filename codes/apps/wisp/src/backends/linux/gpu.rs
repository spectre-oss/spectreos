//! Linux sysfs GPU devfreq interactive frequency boost backend (`/sys/class/devfreq/*gpu*`).

use crate::services::gpu::GpuBoostBackend;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct DevfreqGpuBackend {
    dir: Option<PathBuf>,
    idle_min: u64,
    boost_min: u64,
    boosted: bool,
    boost_until: Instant,
}

impl DevfreqGpuBackend {
    pub fn discover() -> Self {
        let dir = std::fs::read_dir("/sys/class/devfreq")
            .ok()
            .and_then(|entries| {
                entries.filter_map(|e| e.ok().map(|e| e.path())).find(|p| {
                    p.file_name()
                        .and_then(|s| s.to_str())
                        .is_some_and(|n| n.contains("gpu"))
                })
            });
        let (idle_min, boost_min) = dir
            .as_ref()
            .and_then(|d| std::fs::read_to_string(d.join("available_frequencies")).ok())
            .map(|s| {
                let mut freqs: Vec<u64> = s
                    .split_whitespace()
                    .filter_map(|tok| tok.parse::<u64>().ok())
                    .collect();
                freqs.sort_unstable();
                let lo = freqs.first().copied().unwrap_or(257_000_000);
                let hi = freqs.last().copied().unwrap_or(710_000_000);
                (lo, hi)
            })
            .unwrap_or((257_000_000, 710_000_000));
        if let Some(d) = &dir {
            let _ = std::fs::write(d.join("polling_interval"), "10");
        }
        Self {
            dir,
            idle_min,
            boost_min,
            boosted: false,
            boost_until: Instant::now(),
        }
    }
}

impl GpuBoostBackend for DevfreqGpuBackend {
    fn boost(&mut self, hold: Duration) {
        self.boost_until = Instant::now() + hold;
        if self.boosted {
            return;
        }
        if let Some(dir) = &self.dir
            && std::fs::write(dir.join("min_freq"), self.boost_min.to_string()).is_ok()
        {
            self.boosted = true;
        }
    }

    fn release(&mut self, force: bool) {
        if !self.boosted {
            return;
        }
        if !force && Instant::now() < self.boost_until {
            return;
        }
        if let Some(dir) = &self.dir {
            let _ = std::fs::write(dir.join("min_freq"), self.idle_min.to_string());
        }
        self.boosted = false;
    }

    fn current_mhz(&self) -> u32 {
        self.dir
            .as_ref()
            .and_then(|d| std::fs::read_to_string(d.join("cur_freq")).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|hz| (hz / 1_000_000) as u32)
            .unwrap_or(0)
    }
}
