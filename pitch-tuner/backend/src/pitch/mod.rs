mod autocorrelation;
mod yin;

#[cfg(test)]
pub mod signals;

pub use autocorrelation::Autocorrelation;
pub use yin::Yin;

use crate::error::TunerError;

pub const MIN_FREQUENCY: f64 = 40.0;
pub const MAX_FREQUENCY: f64 = 2000.0;
pub const SILENCE_RMS: f64 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pitch {
    pub frequency: f64,
    pub clarity: f64,
}

pub trait PitchDetector: Send + Sync {
    fn name(&self) -> &'static str;
    fn estimate(&self, samples: &[f32], sample_rate: u32) -> Option<Pitch>;

    fn detect(&self, samples: &[f32], sample_rate: u32) -> Result<Option<Pitch>, TunerError> {
        if samples.is_empty() {
            return Err(TunerError::EmptyFrame);
        }
        if sample_rate == 0 {
            return Err(TunerError::InvalidSampleRate(sample_rate));
        }
        if rms(samples) < SILENCE_RMS {
            return Ok(None);
        }
        Ok(self.estimate(samples, sample_rate))
    }
}

pub fn by_name(name: &str) -> Result<Box<dyn PitchDetector>, TunerError> {
    match name {
        "yin" => Ok(Box::new(Yin::default())),
        "autocorrelation" => Ok(Box::new(Autocorrelation::default())),
        other => Err(TunerError::UnknownDetector(other.to_string())),
    }
}

pub fn rms(samples: &[f32]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|s| f64::from(*s) * f64::from(*s)).sum();
    (sum / samples.len() as f64).sqrt()
}

pub fn lag_range(len: usize, sample_rate: u32) -> Option<(usize, usize)> {
    let rate = f64::from(sample_rate);
    let min_lag = ((rate / MAX_FREQUENCY).floor() as usize).max(2);
    let max_lag = ((rate / MIN_FREQUENCY).ceil() as usize).min(len / 2);
    (min_lag + 1 < max_lag).then_some((min_lag, max_lag))
}

pub fn parabolic_offset(left: f64, center: f64, right: f64) -> f64 {
    let denominator = left - 2.0 * center + right;
    if denominator.abs() < f64::EPSILON {
        return 0.0;
    }
    (0.5 * (left - right) / denominator).clamp(-0.5, 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parabolic_offset_finds_true_vertex_of_a_parabola() {
        let vertex = 0.3;
        let f = |x: f64| (x - vertex) * (x - vertex);
        let offset = parabolic_offset(f(-1.0), f(0.0), f(1.0));
        assert!((offset - vertex).abs() < 1e-12);
    }

    #[test]
    fn factory_knows_both_detectors_and_rejects_others() {
        assert_eq!(by_name("yin").unwrap().name(), "yin");
        assert_eq!(
            by_name("autocorrelation").unwrap().name(),
            "autocorrelation"
        );
        assert!(matches!(
            by_name("fft"),
            Err(TunerError::UnknownDetector(_))
        ));
    }

    #[test]
    fn detect_rejects_empty_frames_and_zero_sample_rate() {
        let yin = Yin::default();
        assert_eq!(yin.detect(&[], 44100), Err(TunerError::EmptyFrame));
        assert_eq!(
            yin.detect(&[0.5; 16], 0),
            Err(TunerError::InvalidSampleRate(0))
        );
    }
}
