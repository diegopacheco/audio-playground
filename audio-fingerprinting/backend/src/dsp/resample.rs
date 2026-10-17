use crate::error::FingerprintError;
use crate::wav::Audio;
use std::f32::consts::PI;

const TAPS: usize = 63;

fn lowpass_kernel(cutoff: f32) -> Vec<f32> {
    let mid = (TAPS / 2) as f32;
    let raw: Vec<f32> = (0..TAPS)
        .map(|i| {
            let x = i as f32 - mid;
            let sinc = if x == 0.0 {
                2.0 * cutoff
            } else {
                (2.0 * PI * cutoff * x).sin() / (PI * x)
            };
            let window = 0.54 - 0.46 * (2.0 * PI * i as f32 / (TAPS - 1) as f32).cos();
            sinc * window
        })
        .collect();
    let sum: f32 = raw.iter().sum();
    raw.iter().map(|v| v / sum).collect()
}

fn convolve(samples: &[f32], kernel: &[f32]) -> Vec<f32> {
    let half = kernel.len() / 2;
    (0..samples.len())
        .map(|i| {
            kernel
                .iter()
                .enumerate()
                .filter_map(|(k, w)| {
                    (i + k)
                        .checked_sub(half)
                        .and_then(|j| samples.get(j))
                        .map(|s| s * w)
                })
                .sum()
        })
        .collect()
}

pub fn to_rate(audio: &Audio, target: u32) -> Result<Vec<f32>, FingerprintError> {
    if audio.sample_rate == 0 || target == 0 {
        return Err(FingerprintError::InvalidSampleRate(
            audio.sample_rate.min(target),
        ));
    }
    if audio.samples.is_empty() {
        return Err(FingerprintError::EmptyAudio);
    }
    if audio.sample_rate == target {
        return Ok(audio.samples.clone());
    }
    let ratio = audio.sample_rate as f64 / target as f64;
    let filtered = if ratio > 1.0 {
        let cutoff = (0.45 / ratio) as f32;
        convolve(&audio.samples, &lowpass_kernel(cutoff))
    } else {
        audio.samples.clone()
    };
    let out_len = (filtered.len() as f64 / ratio).floor() as usize;
    let last = filtered.len() - 1;
    Ok((0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let idx = (pos.floor() as usize).min(last);
            let next = (idx + 1).min(last);
            let frac = (pos - idx as f64) as f32;
            filtered[idx] * (1.0 - frac) + filtered[next] * frac
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f32, rate: u32, secs: f32) -> Audio {
        let n = (rate as f32 * secs) as usize;
        Audio {
            sample_rate: rate,
            samples: (0..n)
                .map(|i| (2.0 * PI * freq * i as f32 / rate as f32).sin())
                .collect(),
        }
    }

    fn crossings(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
            .count()
    }

    #[test]
    fn downsampled_440hz_keeps_its_pitch_of_880_crossings_per_second() {
        let out = to_rate(&sine(440.0, 44100, 1.0), 8000).unwrap();
        assert_eq!(out.len(), 8000);
        let c = crossings(&out[100..7900]) as f32 / (7800.0 / 8000.0);
        assert!((c - 880.0).abs() < 10.0, "crossings {c}");
    }

    #[test]
    fn tone_above_target_nyquist_is_removed_instead_of_aliasing() {
        let out = to_rate(&sine(6000.0, 48000, 1.0), 8000).unwrap();
        let rms = (out[200..7800].iter().map(|v| v * v).sum::<f32>() / 7600.0).sqrt();
        assert!(rms < 0.05, "rms {rms}");
    }
}
