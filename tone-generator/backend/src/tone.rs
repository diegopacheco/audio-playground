use crate::error::AppError;
use crate::oscillator::{Waveform, render};
use crate::wav;
use serde::Deserialize;

pub const SAMPLE_RATES: [u32; 5] = [8000, 16000, 22050, 44100, 48000];
const MIN_FREQUENCY: f64 = 20.0;
const MAX_DURATION_MS: u32 = 10_000;

#[derive(Debug, Deserialize)]
pub struct ToneRequest {
    pub waveform: String,
    pub frequency: f64,
    pub duration_ms: u32,
    pub amplitude: f64,
    pub sample_rate: u32,
}

pub fn sample_count(sample_rate: u32, duration_ms: u32) -> usize {
    (u64::from(sample_rate) * u64::from(duration_ms) / 1000) as usize
}

pub fn generate(req: &ToneRequest) -> Result<Vec<u8>, AppError> {
    let waveform: Waveform = req.waveform.parse()?;
    if !SAMPLE_RATES.contains(&req.sample_rate) {
        return Err(AppError::InvalidSampleRate(req.sample_rate));
    }
    let nyquist = f64::from(req.sample_rate) / 2.0;
    if !(MIN_FREQUENCY..nyquist).contains(&req.frequency) {
        return Err(AppError::InvalidFrequency(req.frequency));
    }
    if req.duration_ms == 0 || req.duration_ms > MAX_DURATION_MS {
        return Err(AppError::InvalidDuration(req.duration_ms));
    }
    if !(0.0..=1.0).contains(&req.amplitude) {
        return Err(AppError::InvalidAmplitude(req.amplitude));
    }
    let count = sample_count(req.sample_rate, req.duration_ms);
    let samples = render(
        waveform.oscillator().as_ref(),
        req.frequency,
        req.amplitude,
        req.sample_rate,
        count,
    );
    Ok(wav::encode(req.sample_rate, &samples))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(frequency: f64, duration_ms: u32, sample_rate: u32) -> ToneRequest {
        ToneRequest {
            waveform: "sine".into(),
            frequency,
            duration_ms,
            amplitude: 0.8,
            sample_rate,
        }
    }

    #[test]
    fn sample_count_equals_rate_times_duration_so_playback_lasts_as_requested() {
        for (rate, ms) in [(44100, 1000), (48000, 250), (8000, 1500), (22050, 10)] {
            let bytes = generate(&request(440.0, ms, rate)).expect("valid tone");
            let samples = (bytes.len() - wav::HEADER_LEN) / 2;
            assert_eq!(samples, rate as usize * ms as usize / 1000);
        }
    }

    #[test]
    fn frequency_at_or_above_nyquist_is_rejected_because_it_would_alias() {
        assert!(matches!(
            generate(&request(4000.0, 100, 8000)),
            Err(AppError::InvalidFrequency(_))
        ));
        assert!(generate(&request(3999.0, 100, 8000)).is_ok());
    }

    #[test]
    fn out_of_range_inputs_are_rejected_instead_of_producing_broken_audio() {
        assert!(matches!(
            generate(&request(440.0, 0, 44100)),
            Err(AppError::InvalidDuration(0))
        ));
        assert!(matches!(
            generate(&request(440.0, 100, 12345)),
            Err(AppError::InvalidSampleRate(12345))
        ));
        let mut loud = request(440.0, 100, 44100);
        loud.amplitude = 1.5;
        assert!(matches!(
            generate(&loud),
            Err(AppError::InvalidAmplitude(_))
        ));
        let mut noise = request(440.0, 100, 44100);
        noise.waveform = "noise".into();
        assert!(matches!(
            generate(&noise),
            Err(AppError::UnknownWaveform(_))
        ));
    }
}
