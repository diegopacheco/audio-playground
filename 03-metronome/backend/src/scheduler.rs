use std::time::Duration;
use tokio::time::{Instant, sleep, sleep_until};

pub trait Scheduler {
    fn name(&self) -> &'static str;
    fn run(&self, interval: Duration, beats: usize) -> impl Future<Output = Vec<Duration>> + Send;
}

pub struct NaiveSleep;

pub struct AbsoluteDeadline;

impl Scheduler for NaiveSleep {
    fn name(&self) -> &'static str {
        "naive-sleep"
    }

    async fn run(&self, interval: Duration, beats: usize) -> Vec<Duration> {
        let start = Instant::now();
        let mut ticks = Vec::with_capacity(beats);
        for beat in 0..beats {
            ticks.push(start.elapsed());
            if beat + 1 < beats {
                sleep(interval).await;
            }
        }
        ticks
    }
}

impl Scheduler for AbsoluteDeadline {
    fn name(&self) -> &'static str {
        "absolute-deadline"
    }

    async fn run(&self, interval: Duration, beats: usize) -> Vec<Duration> {
        let start = Instant::now();
        let mut ticks = Vec::with_capacity(beats);
        for beat in 0..beats {
            sleep_until(start + interval * beat as u32).await;
            ticks.push(start.elapsed());
        }
        ticks
    }
}

pub fn drift_ms(ticks: &[Duration], interval: Duration) -> Vec<f64> {
    ticks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.as_secs_f64() - interval.as_secs_f64() * i as f64) * 1_000.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTERVAL: Duration = Duration::from_millis(20);
    const BEATS: usize = 25;

    #[tokio::test(flavor = "multi_thread")]
    async fn absolute_deadline_drift_stays_bounded_instead_of_accumulating() {
        let ticks = AbsoluteDeadline.run(INTERVAL, BEATS).await;
        let drift = drift_ms(&ticks, INTERVAL);
        assert_eq!(drift.len(), BEATS);
        let worst = drift.iter().cloned().fold(0.0f64, |a, b| a.max(b.abs()));
        assert!(worst < 15.0, "absolute scheduler drifted {worst} ms");
        let tail = drift[BEATS - 5..].iter().sum::<f64>() / 5.0;
        assert!(tail < 15.0, "absolute scheduler accumulated {tail} ms");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn naive_sleep_is_never_early_so_its_error_only_grows() {
        let ticks = NaiveSleep.run(INTERVAL, BEATS).await;
        let drift = drift_ms(&ticks, INTERVAL);
        for pair in ticks.windows(2) {
            assert!(pair[1] - pair[0] >= INTERVAL);
        }
        assert!(drift.iter().all(|d| *d >= 0.0));
        assert!(drift[BEATS - 1] >= drift[1]);
    }

    #[test]
    fn drift_is_measured_against_ideal_beat_grid() {
        let ticks = vec![
            Duration::from_millis(0),
            Duration::from_millis(101),
            Duration::from_millis(203),
        ];
        let d = drift_ms(&ticks, Duration::from_millis(100));
        assert!((d[1] - 1.0).abs() < 1e-9);
        assert!((d[2] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn schedulers_report_distinct_names() {
        assert_ne!(NaiveSleep.name(), AbsoluteDeadline.name());
    }
}
