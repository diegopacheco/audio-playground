use crate::error::AppError;
use crate::scheduler::{AbsoluteDeadline, NaiveSleep, Scheduler, drift_ms};
use serde::Serialize;
use std::time::Duration;

const MAX_RUN_SECONDS: f64 = 30.0;

#[derive(Debug, Serialize)]
pub struct BeatTiming {
    pub index: usize,
    pub ideal_ms: f64,
    pub naive_ms: f64,
    pub naive_drift_ms: f64,
    pub absolute_ms: f64,
    pub absolute_drift_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct DriftReport {
    pub bpm: f64,
    pub interval_ms: f64,
    pub naive: &'static str,
    pub absolute: &'static str,
    pub naive_max_drift_ms: f64,
    pub absolute_max_drift_ms: f64,
    pub beats: Vec<BeatTiming>,
}

pub fn validate(bpm: f64, beats: u32) -> Result<Duration, AppError> {
    if !(20.0..=300.0).contains(&bpm) {
        return Err(AppError::InvalidBpm(bpm));
    }
    if !(2..=64).contains(&beats) {
        return Err(AppError::InvalidBeats(beats));
    }
    let seconds = 60.0 / bpm * (beats - 1) as f64;
    if seconds > MAX_RUN_SECONDS {
        return Err(AppError::RunTooLong(seconds));
    }
    Ok(Duration::from_secs_f64(60.0 / bpm))
}

pub async fn measure(bpm: f64, beats: u32) -> Result<DriftReport, AppError> {
    let interval = validate(bpm, beats)?;
    let count = beats as usize;
    let naive = NaiveSleep;
    let absolute = AbsoluteDeadline;
    let (naive_ticks, absolute_ticks) =
        tokio::join!(naive.run(interval, count), absolute.run(interval, count));
    Ok(build(
        bpm,
        interval,
        &naive,
        &absolute,
        &naive_ticks,
        &absolute_ticks,
    ))
}

fn build<N: Scheduler, A: Scheduler>(
    bpm: f64,
    interval: Duration,
    naive: &N,
    absolute: &A,
    naive_ticks: &[Duration],
    absolute_ticks: &[Duration],
) -> DriftReport {
    let naive_drift = drift_ms(naive_ticks, interval);
    let absolute_drift = drift_ms(absolute_ticks, interval);
    let interval_ms = interval.as_secs_f64() * 1_000.0;
    let beats = naive_ticks
        .iter()
        .zip(absolute_ticks)
        .enumerate()
        .map(|(i, (n, a))| BeatTiming {
            index: i,
            ideal_ms: interval_ms * i as f64,
            naive_ms: n.as_secs_f64() * 1_000.0,
            naive_drift_ms: naive_drift[i],
            absolute_ms: a.as_secs_f64() * 1_000.0,
            absolute_drift_ms: absolute_drift[i],
        })
        .collect();
    DriftReport {
        bpm,
        interval_ms,
        naive: naive.name(),
        absolute: absolute.name(),
        naive_max_drift_ms: max_abs(&naive_drift),
        absolute_max_drift_ms: max_abs(&absolute_drift),
        beats,
    }
}

fn max_abs(values: &[f64]) -> f64 {
    values.iter().fold(0.0, |a, b| a.max(b.abs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_length_is_capped_so_requests_finish_in_reasonable_time() {
        assert!(validate(120.0, 16).is_ok());
        assert_eq!(validate(10.0, 16), Err(AppError::InvalidBpm(10.0)));
        assert_eq!(validate(120.0, 1), Err(AppError::InvalidBeats(1)));
        assert!(matches!(validate(20.0, 64), Err(AppError::RunTooLong(_))));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn report_pairs_each_beat_with_its_ideal_time() {
        let report = measure(300.0, 6).await.unwrap();
        assert_eq!(report.beats.len(), 6);
        assert!((report.interval_ms - 200.0).abs() < 1e-9);
        assert!((report.beats[5].ideal_ms - 1_000.0).abs() < 1e-9);
        assert!(report.naive_max_drift_ms >= report.beats[5].naive_drift_ms.abs());
    }
}
