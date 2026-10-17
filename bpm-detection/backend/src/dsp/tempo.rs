use super::sample_at;
use crate::error::AudioError;

#[derive(Debug, Clone, PartialEq)]
pub struct TempoEstimate {
    pub bpm: f32,
    pub lag: f32,
    pub confidence: f32,
    pub autocorr: Vec<f32>,
}

pub trait TempoEstimator: Send + Sync {
    fn estimate(&self, envelope: &[f32], frame_rate: f32) -> Result<TempoEstimate, AudioError>;
}

pub struct AutocorrTempo {
    pub min_bpm: f32,
    pub max_bpm: f32,
}

const HARMONICS: usize = 4;
const BPM_STEP: f32 = 0.02;
const OCTAVE_RATIO: f32 = 0.5;

pub fn bpm_to_lag(bpm: f32, frame_rate: f32) -> f32 {
    60.0 * frame_rate / bpm
}

pub fn autocorrelation(envelope: &[f32], max_lag: usize) -> Vec<f32> {
    let n = envelope.len();
    let mean = envelope.iter().sum::<f32>() / n.max(1) as f32;
    let centered: Vec<f32> = envelope.iter().map(|v| v - mean).collect();
    let raw: Vec<f32> = (0..=max_lag.min(n.saturating_sub(1)))
        .map(|lag| {
            centered[..n - lag]
                .iter()
                .zip(&centered[lag..])
                .map(|(a, b)| a * b)
                .sum::<f32>()
                / n as f32
        })
        .collect();
    let energy = raw.first().copied().unwrap_or(0.0);
    if energy <= f32::EPSILON {
        return vec![0.0; raw.len()];
    }
    raw.iter().map(|v| v / energy).collect()
}

fn comb_score(autocorr: &[f32], lag: f32) -> f32 {
    let limit = (autocorr.len() - 1) as f32;
    let values: Vec<f32> = (1..=HARMONICS)
        .map(|k| k as f32 * lag)
        .take_while(|l| *l <= limit)
        .map(|l| sample_at(autocorr, l))
        .collect();
    if values.is_empty() {
        return f32::MIN;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

impl AutocorrTempo {
    fn best_in_range(&self, autocorr: &[f32], frame_rate: f32, lo: f32, hi: f32) -> f32 {
        let steps = ((hi - lo) / BPM_STEP).round() as usize;
        (0..=steps)
            .map(|i| lo + i as f32 * BPM_STEP)
            .map(|bpm| (bpm, comb_score(autocorr, bpm_to_lag(bpm, frame_rate))))
            .fold(
                (lo, f32::MIN),
                |best, c| if c.1 > best.1 { c } else { best },
            )
            .0
    }

    fn resolve_octave(&self, autocorr: &[f32], frame_rate: f32, bpm: f32) -> f32 {
        let mut current = bpm;
        while current * 2.0 <= self.max_bpm {
            let lag = bpm_to_lag(current, frame_rate);
            let here = sample_at(autocorr, lag);
            let double = sample_at(autocorr, lag / 2.0);
            if here <= 0.0 || double < OCTAVE_RATIO * here {
                break;
            }
            current = self.best_in_range(
                autocorr,
                frame_rate,
                (current * 2.0 - 2.0).max(self.min_bpm),
                (current * 2.0 + 2.0).min(self.max_bpm),
            );
        }
        current
    }
}

impl TempoEstimator for AutocorrTempo {
    fn estimate(&self, envelope: &[f32], frame_rate: f32) -> Result<TempoEstimate, AudioError> {
        let longest = bpm_to_lag(self.min_bpm, frame_rate);
        if (envelope.len() as f32) < 2.0 * longest {
            return Err(AudioError::TooShort);
        }
        let max_lag = (HARMONICS as f32 * longest).ceil() as usize + 2;
        let autocorr = autocorrelation(envelope, max_lag);
        if autocorr.first().copied().unwrap_or(0.0) <= 0.0 {
            return Err(AudioError::Silence);
        }
        let coarse = self.best_in_range(&autocorr, frame_rate, self.min_bpm, self.max_bpm);
        let bpm = self.resolve_octave(&autocorr, frame_rate, coarse);
        let lag = bpm_to_lag(bpm, frame_rate);
        let confidence = comb_score(&autocorr, lag).clamp(0.0, 1.0);
        Ok(TempoEstimate {
            bpm,
            lag,
            confidence,
            autocorr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f32 = 100.0;

    fn pulse_train(period: f32, len: usize, accent_every: usize) -> Vec<f32> {
        let mut env = vec![0.0; len];
        let mut k = 0;
        loop {
            let pos = (k as f32 * period).round() as usize;
            if pos >= len {
                break;
            }
            env[pos] = if k % accent_every == 0 { 1.0 } else { 0.6 };
            k += 1;
        }
        env
    }

    fn estimator() -> AutocorrTempo {
        AutocorrTempo {
            min_bpm: 60.0,
            max_bpm: 200.0,
        }
    }

    #[test]
    fn autocorrelation_peaks_at_the_pulse_period() {
        let r = autocorrelation(&pulse_train(50.0, 2000, 1), 120);
        assert_eq!(r[0], 1.0);
        assert!(r[50] > 0.9);
        assert!(r[25] < 0.1);
    }

    #[test]
    fn pulse_every_half_second_is_120_bpm() {
        let est = estimator()
            .estimate(&pulse_train(50.0, 3000, 1), RATE)
            .unwrap();
        assert!((est.bpm - 120.0).abs() < 0.5, "got {}", est.bpm);
        assert!(est.confidence > 0.5);
    }

    #[test]
    fn accented_downbeats_do_not_halve_the_tempo() {
        let est = estimator()
            .estimate(&pulse_train(50.0, 3000, 2), RATE)
            .unwrap();
        assert!((est.bpm - 120.0).abs() < 0.5, "got {}", est.bpm);
    }

    #[test]
    fn flat_envelope_is_reported_as_silence() {
        assert_eq!(
            estimator().estimate(&vec![0.0; 3000], RATE),
            Err(AudioError::Silence)
        );
    }

    #[test]
    fn envelope_shorter_than_two_slow_beats_is_too_short() {
        assert_eq!(
            estimator().estimate(&vec![1.0; 150], RATE),
            Err(AudioError::TooShort)
        );
    }
}
