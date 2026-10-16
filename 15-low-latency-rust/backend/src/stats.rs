use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct CallbackTimer {
    count: AtomicU64,
    last_ns: AtomicU64,
    max_ns: AtomicU64,
    total_ns: AtomicU64,
}

impl CallbackTimer {
    pub fn record(&self, ns: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.last_ns.store(ns, Ordering::Relaxed);
        self.max_ns.fetch_max(ns, Ordering::Relaxed);
        self.total_ns.fetch_add(ns, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> CallbackSnapshot {
        let count = self.count.load(Ordering::Relaxed);
        let total = self.total_ns.load(Ordering::Relaxed);
        CallbackSnapshot {
            count,
            last_us: self.last_ns.load(Ordering::Relaxed) as f64 / 1000.0,
            max_us: self.max_ns.load(Ordering::Relaxed) as f64 / 1000.0,
            avg_us: if count == 0 {
                0.0
            } else {
                total as f64 / count as f64 / 1000.0
            },
        }
    }
}

#[derive(Default)]
pub struct Stats {
    pub underruns: AtomicU64,
    pub overruns: AtomicU64,
    pub ring_fill: AtomicU64,
    pub input: CallbackTimer,
    pub output: CallbackTimer,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct CallbackSnapshot {
    pub count: u64,
    pub last_us: f64,
    pub max_us: f64,
    pub avg_us: f64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct CounterSnapshot {
    pub underruns: u64,
    pub overruns: u64,
    pub ring_fill: u64,
    pub input_callbacks: CallbackSnapshot,
    pub output_callbacks: CallbackSnapshot,
}

impl Stats {
    pub fn snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            underruns: self.underruns.load(Ordering::Relaxed),
            overruns: self.overruns.load(Ordering::Relaxed),
            ring_fill: self.ring_fill.load(Ordering::Relaxed),
            input_callbacks: self.input.snapshot(),
            output_callbacks: self.output.snapshot(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_tracks_worst_case_not_just_latest_because_xruns_come_from_spikes() {
        let t = CallbackTimer::default();
        t.record(1_000);
        t.record(9_000);
        t.record(2_000);
        let s = t.snapshot();
        assert_eq!(s.count, 3);
        assert_eq!(s.last_us, 2.0);
        assert_eq!(s.max_us, 9.0);
        assert_eq!(s.avg_us, 4.0);
    }
}
