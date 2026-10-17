use crate::dsp::resonator::Resonator;
use crate::error::VoiceError;
use std::f32::consts::{PI, TAU};

const BASE_PITCH: f32 = 130.0;
const GLIDE_DEPTH: f32 = 0.25;
const GLIDE_HZ: f32 = 0.35;
const VIBRATO_DEPTH: f32 = 0.02;
const VIBRATO_HZ: f32 = 5.5;
const OPEN_PHASE: f32 = 0.4;
const CLOSING_PHASE: f32 = 0.16;
const SYLLABLE_SECONDS: f32 = 0.6;
const PEAK: f32 = 0.6;
const MAX_SECONDS: f32 = 30.0;
const MIN_RATE: u32 = 8000;
const MAX_RATE: u32 = 192_000;
const BANDWIDTHS: [f32; 3] = [90.0, 110.0, 170.0];
const VOWELS: [[f32; 3]; 4] = [
    [730.0, 1090.0, 2440.0],
    [270.0, 2290.0, 3010.0],
    [300.0, 870.0, 2240.0],
    [530.0, 1840.0, 2480.0],
];

fn glottal_pulse(phase: f32) -> f32 {
    if phase < OPEN_PHASE {
        0.5 * (1.0 - (PI * phase / OPEN_PHASE).cos())
    } else if phase < OPEN_PHASE + CLOSING_PHASE {
        (PI * (phase - OPEN_PHASE) / (2.0 * CLOSING_PHASE)).cos()
    } else {
        0.0
    }
}

fn formants(time: f32) -> [f32; 3] {
    let position = time / SYLLABLE_SECONDS;
    let index = position as usize;
    let current = VOWELS[index % VOWELS.len()];
    let next = VOWELS[(index + 1) % VOWELS.len()];
    let blend = ((position.fract() - 0.7) / 0.3).clamp(0.0, 1.0);
    [0, 1, 2].map(|i| current[i] + (next[i] - current[i]) * blend)
}

pub fn pitch_at(time: f32) -> f32 {
    BASE_PITCH
        * (1.0 + GLIDE_DEPTH * (TAU * GLIDE_HZ * time).sin())
        * (1.0 + VIBRATO_DEPTH * (TAU * VIBRATO_HZ * time).sin())
}

pub fn voice(rate: u32, seconds: f32) -> Result<Vec<f32>, VoiceError> {
    if !(MIN_RATE..=MAX_RATE).contains(&rate) {
        return Err(VoiceError::InvalidSampleRate(rate));
    }
    if !seconds.is_finite() || seconds <= 0.0 || seconds > MAX_SECONDS {
        return Err(VoiceError::InvalidDuration(seconds));
    }
    let fs = rate as f32;
    let count = (fs * seconds) as usize;
    let mut resonators: [Resonator; 3] = Default::default();
    let mut phase = 0.0f32;
    let mut previous = 0.0f32;
    let mut out = Vec::with_capacity(count);
    for n in 0..count {
        let time = n as f32 / fs;
        phase = (phase + pitch_at(time) / fs).fract();
        let pulse = glottal_pulse(phase);
        let mut sample = pulse - previous;
        previous = pulse;
        let centers = formants(time);
        for ((resonator, center), bandwidth) in resonators.iter_mut().zip(centers).zip(BANDWIDTHS) {
            sample = resonator.process(sample, center, bandwidth, fs);
        }
        let syllable = 0.35 + 0.65 * (PI * (time / SYLLABLE_SECONDS).fract()).sin();
        out.push(sample * syllable);
    }
    let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.0 {
        let scale = PEAK / peak;
        out.iter_mut().for_each(|s| *s *= scale);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::periodicity;

    #[test]
    fn voice_is_normalized_and_periodic_at_its_glottal_pitch() {
        let rate = 16000;
        let samples = voice(rate, 1.0).unwrap();
        let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((peak - PEAK).abs() < 1e-4);
        let (pitch, clarity) = periodicity(&samples[1000..3000], rate, 70.0, 400.0);
        let expected = pitch_at(0.125);
        assert!(
            (pitch - expected).abs() < expected * 0.05,
            "{pitch} vs {expected}"
        );
        assert!(clarity > 0.8);
    }

    #[test]
    fn voice_spectrum_is_shaped_by_formants_not_flat() {
        let rate = 16000;
        let samples = voice(rate, 0.5).unwrap();
        let band = |hz: f32| {
            let mut resonator = Resonator::default();
            let filtered: Vec<f32> = samples
                .iter()
                .map(|s| resonator.process(*s, hz, 60.0, rate as f32))
                .collect();
            filtered.iter().map(|s| s * s).sum::<f32>()
        };
        assert!(band(1090.0) > 4.0 * band(5500.0));
    }

    #[test]
    fn rejects_bad_rate_and_duration() {
        assert_eq!(voice(100, 1.0), Err(VoiceError::InvalidSampleRate(100)));
        assert_eq!(voice(16000, 0.0), Err(VoiceError::InvalidDuration(0.0)));
        assert_eq!(voice(16000, 60.0), Err(VoiceError::InvalidDuration(60.0)));
    }
}
