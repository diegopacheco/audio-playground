use std::f64::consts::TAU;

use serde::Deserialize;

use crate::audio::Mono;
use crate::dsp::resonator::Resonator;

pub const TONE_HZ: f64 = 880.0;
const CLICKS_PER_SECOND: f64 = 4.0;
const VOWELS: [[f64; 3]; 4] = [
    [730.0, 1090.0, 2440.0],
    [270.0, 2290.0, 3010.0],
    [570.0, 840.0, 2410.0],
    [300.0, 870.0, 2240.0],
];

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Noise,
    Tone,
    Clicks,
    VoiceLike,
}

struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

pub fn generate(kind: SourceKind, sample_rate: u32, duration: f64) -> Mono {
    let len = (duration * sample_rate as f64).round() as usize;
    let fs = sample_rate as f64;
    let samples = match kind {
        SourceKind::Noise => noise(len),
        SourceKind::Tone => tone(len, fs),
        SourceKind::Clicks => clicks(len, fs),
        SourceKind::VoiceLike => voice_like(len, fs),
    };
    Mono {
        sample_rate,
        samples,
    }
}

fn noise(len: usize) -> Vec<f32> {
    let mut rng = Noise(0x9E37_79B9);
    (0..len).map(|_| rng.next() * 0.5).collect()
}

fn tone(len: usize, fs: f64) -> Vec<f32> {
    (0..len)
        .map(|n| (0.5 * (TAU * TONE_HZ * n as f64 / fs).sin()) as f32)
        .collect()
}

fn clicks(len: usize, fs: f64) -> Vec<f32> {
    let mut rng = Noise(0x1234_5678);
    let period = (fs / CLICKS_PER_SECOND) as usize;
    let burst = (0.006 * fs) as usize;
    (0..len)
        .map(|n| {
            let pos = n % period;
            if pos < burst {
                let env = 1.0 - pos as f32 / burst as f32;
                rng.next() * env * env * 0.9
            } else {
                0.0
            }
        })
        .collect()
}

fn voice_like(len: usize, fs: f64) -> Vec<f32> {
    let mut formants: Vec<Resonator> = VOWELS[0]
        .iter()
        .map(|f| Resonator::new(*f, 90.0, fs))
        .collect();
    let mut phase = 0.0;
    let mut out = Vec::with_capacity(len);
    for n in 0..len {
        let t = n as f64 / fs;
        let pitch = 120.0 + 12.0 * (TAU * 5.0 * t).sin() + 20.0 * (TAU * 0.5 * t).sin();
        phase += pitch / fs;
        let pulse = if phase >= 1.0 {
            phase -= 1.0;
            1.0
        } else {
            0.0
        };
        let vowel = VOWELS[(t * 3.0) as usize % VOWELS.len()];
        let syllable = (TAU * 3.0 * t).sin().abs().powf(0.6);
        let mut sum = 0.0;
        for (res, f) in formants.iter_mut().zip(vowel) {
            res.tune(f, 90.0, fs);
            sum += res.process(pulse);
        }
        out.push(sum * syllable);
    }
    normalize(&mut out, 0.6);
    out.into_iter().map(|s| s as f32).collect()
}

fn normalize(samples: &mut [f64], peak: f64) {
    let max = samples.iter().fold(0.0f64, |m, s| m.max(s.abs()));
    if max > 0.0 {
        samples.iter_mut().for_each(|s| *s *= peak / max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_source_fills_the_requested_duration_without_clipping() {
        for kind in [
            SourceKind::Noise,
            SourceKind::Tone,
            SourceKind::Clicks,
            SourceKind::VoiceLike,
        ] {
            let mono = generate(kind, 44_100, 1.5);
            assert_eq!(mono.samples.len(), 66_150);
            assert!(mono.samples.iter().all(|s| s.abs() <= 1.0));
            assert!(mono.samples.iter().any(|s| s.abs() > 0.1));
        }
    }

    #[test]
    fn clicks_are_mostly_silence_so_onsets_are_easy_to_localize() {
        let mono = generate(SourceKind::Clicks, 44_100, 2.0);
        let silent = mono.samples.iter().filter(|s| **s == 0.0).count();
        assert!(silent as f64 / mono.samples.len() as f64 > 0.9);
    }
}
