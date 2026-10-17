use crate::dsp::beats::{BeatTracker, PhaseAlignedTracker};
use crate::dsp::onset::{EnergyFlux, Framing, OnsetDetector, SpectralFlux};
use crate::dsp::resample::resample;
use crate::dsp::tempo::{AutocorrTempo, TempoEstimator};
use crate::error::AudioError;
use crate::wav::Audio;
use serde::{Deserialize, Serialize};

pub const ANALYSIS_RATE: u32 = 22050;
const FRAMING: Framing = Framing {
    frame_size: 1024,
    hop: 128,
};
const SILENCE_PEAK: f32 = 1e-4;

#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectorKind {
    #[default]
    Flux,
    Energy,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Analysis {
    pub bpm: f32,
    pub confidence: f32,
    pub detector: DetectorKind,
    pub frame_rate: f32,
    pub lag: f32,
    pub duration_sec: f32,
    pub onset_envelope: Vec<f32>,
    pub beats_sec: Vec<f32>,
    pub autocorr: Vec<f32>,
}

pub struct Analyzer {
    kind: DetectorKind,
    detector: Box<dyn OnsetDetector>,
    estimator: Box<dyn TempoEstimator>,
    tracker: Box<dyn BeatTracker>,
}

impl Analyzer {
    pub fn new(kind: DetectorKind) -> Self {
        let detector: Box<dyn OnsetDetector> = match kind {
            DetectorKind::Flux => Box::new(SpectralFlux::new(FRAMING)),
            DetectorKind::Energy => Box::new(EnergyFlux::new(FRAMING)),
        };
        Self {
            kind,
            detector,
            estimator: Box::new(AutocorrTempo {
                min_bpm: 60.0,
                max_bpm: 200.0,
            }),
            tracker: Box::new(PhaseAlignedTracker { snap_radius: 3 }),
        }
    }

    pub fn analyze(&self, audio: &Audio) -> Result<Analysis, AudioError> {
        let peak = audio.samples.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        if peak < SILENCE_PEAK {
            return Err(AudioError::Silence);
        }
        let samples = resample(&audio.samples, audio.sample_rate, ANALYSIS_RATE);
        let envelope = normalize(self.detector.envelope(&centered_frames(&samples))?)?;
        let frame_rate = ANALYSIS_RATE as f32 / FRAMING.hop as f32;
        let tempo = self.estimator.estimate(&envelope, frame_rate)?;
        let beats_sec = self
            .tracker
            .track(&envelope, tempo.lag)
            .iter()
            .map(|frame| frame / frame_rate)
            .collect();
        Ok(Analysis {
            bpm: tempo.bpm,
            confidence: tempo.confidence,
            detector: self.kind,
            frame_rate,
            lag: tempo.lag,
            duration_sec: samples.len() as f32 / ANALYSIS_RATE as f32,
            onset_envelope: envelope,
            beats_sec,
            autocorr: tempo.autocorr,
        })
    }
}

fn centered_frames(samples: &[f32]) -> Vec<f32> {
    let mut padded = vec![0.0; FRAMING.frame_size / 2];
    padded.extend_from_slice(samples);
    padded
}

fn normalize(envelope: Vec<f32>) -> Result<Vec<f32>, AudioError> {
    let max = envelope.iter().fold(0.0f32, |m, v| m.max(*v));
    if max <= f32::EPSILON {
        return Err(AudioError::Silence);
    }
    Ok(envelope.into_iter().map(|v| v / max).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::drum_loop;

    fn loop_audio(bpm: f32) -> Audio {
        Audio {
            sample_rate: 44100,
            samples: drum_loop(bpm, 12.0, 44100).unwrap(),
        }
    }

    fn assert_detects(kind: DetectorKind, bpm: f32) {
        let result = Analyzer::new(kind).analyze(&loop_audio(bpm)).unwrap();
        assert!(
            (result.bpm - bpm).abs() <= 1.0,
            "{kind:?} expected {bpm} got {}",
            result.bpm
        );
    }

    #[test]
    fn spectral_flux_detects_90_bpm_within_one_bpm() {
        assert_detects(DetectorKind::Flux, 90.0);
    }

    #[test]
    fn spectral_flux_detects_120_bpm_within_one_bpm() {
        assert_detects(DetectorKind::Flux, 120.0);
    }

    #[test]
    fn spectral_flux_detects_128_bpm_within_one_bpm() {
        assert_detects(DetectorKind::Flux, 128.0);
    }

    #[test]
    fn spectral_flux_detects_174_bpm_without_halving_to_87() {
        assert_detects(DetectorKind::Flux, 174.0);
    }

    #[test]
    fn energy_detector_agrees_on_all_reference_tempos() {
        for bpm in [90.0, 120.0, 128.0, 174.0] {
            assert_detects(DetectorKind::Energy, bpm);
        }
    }

    #[test]
    fn beats_line_up_with_the_generated_quarter_notes() {
        let bpm = 120.0;
        let result = Analyzer::new(DetectorKind::Flux)
            .analyze(&loop_audio(bpm))
            .unwrap();
        let period = 60.0 / bpm;
        assert!(result.beats_sec.len() >= 23);
        assert!(result.beats_sec[0] < 0.035, "kick at time zero was missed");
        for beat in &result.beats_sec {
            let nearest = (beat / period).round() * period;
            assert!(
                (beat - nearest).abs() < 0.035,
                "beat {beat} is off the grid"
            );
        }
    }

    #[test]
    fn decimation_from_48k_stereo_rate_keeps_tempo() {
        let samples = drum_loop(128.0, 12.0, 48000).unwrap();
        let audio = Audio {
            sample_rate: 48000,
            samples,
        };
        let result = Analyzer::new(DetectorKind::Flux).analyze(&audio).unwrap();
        assert!((result.bpm - 128.0).abs() <= 1.0, "got {}", result.bpm);
    }

    #[test]
    fn silence_is_an_error_not_a_tempo() {
        let audio = Audio {
            sample_rate: 44100,
            samples: vec![0.0; 44100 * 10],
        };
        assert_eq!(
            Analyzer::new(DetectorKind::Flux).analyze(&audio),
            Err(AudioError::Silence)
        );
    }

    #[test]
    fn near_silent_noise_floor_is_also_silence() {
        let audio = Audio {
            sample_rate: 44100,
            samples: (0..441000).map(|i| ((i % 3) as f32 - 1.0) * 1e-6).collect(),
        };
        assert_eq!(
            Analyzer::new(DetectorKind::Energy).analyze(&audio),
            Err(AudioError::Silence)
        );
    }

    #[test]
    fn clip_shorter_than_two_slow_beats_is_too_short() {
        let audio = Audio {
            sample_rate: 44100,
            samples: drum_loop(120.0, 1.0, 44100).unwrap(),
        };
        assert_eq!(
            Analyzer::new(DetectorKind::Flux).analyze(&audio),
            Err(AudioError::TooShort)
        );
    }
}
