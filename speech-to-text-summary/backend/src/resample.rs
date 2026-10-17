use std::f64::consts::PI;

const TAPS: usize = 63;

pub fn downmix(channels: &[Vec<f32>]) -> Vec<f32> {
    let Some(first) = channels.first() else {
        return Vec::new();
    };
    let n = channels.len() as f32;
    (0..first.len())
        .map(|i| channels.iter().map(|c| c[i]).sum::<f32>() / n)
        .collect()
}

fn lowpass(input: &[f32], cutoff: f64) -> Vec<f32> {
    let mid = (TAPS / 2) as f64;
    let mut kernel: Vec<f64> = (0..TAPS)
        .map(|i| {
            let x = i as f64 - mid;
            let sinc = if x == 0.0 {
                2.0 * cutoff
            } else {
                (2.0 * PI * cutoff * x).sin() / (PI * x)
            };
            let window = 0.54 - 0.46 * (2.0 * PI * i as f64 / (TAPS - 1) as f64).cos();
            sinc * window
        })
        .collect();
    let sum: f64 = kernel.iter().sum();
    kernel.iter_mut().for_each(|k| *k /= sum);
    let half = TAPS / 2;
    (0..input.len())
        .map(|i| {
            let mut acc = 0.0;
            for (k, w) in kernel.iter().enumerate() {
                let j = i as isize + k as isize - half as isize;
                if j >= 0 && (j as usize) < input.len() {
                    acc += w * input[j as usize] as f64;
                }
            }
            acc as f32
        })
        .collect()
}

pub fn resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || input.is_empty() {
        return input.to_vec();
    }
    let source = if to < from {
        lowpass(input, 0.45 * to as f64 / from as f64)
    } else {
        input.to_vec()
    };
    let ratio = from as f64 / to as f64;
    let out_len = (input.len() as f64 / ratio).round() as usize;
    let last = source.len() - 1;
    (0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let idx = (pos.floor() as usize).min(last);
            let next = (idx + 1).min(last);
            let frac = (pos - idx as f64) as f32;
            source[idx] * (1.0 - frac) + source[next] * frac
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f64, rate: u32, seconds: f64) -> Vec<f32> {
        let n = (rate as f64 * seconds) as usize;
        (0..n)
            .map(|i| (2.0 * PI * freq * i as f64 / rate as f64).sin() as f32)
            .collect()
    }

    fn zero_crossings(s: &[f32]) -> usize {
        s.windows(2)
            .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
            .count()
    }

    fn rms(s: &[f32]) -> f32 {
        (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
    }

    #[test]
    fn output_length_scales_with_rate_ratio_so_duration_is_kept() {
        let out = resample(&vec![0.0; 44_100], 44_100, 16_000);
        assert_eq!(out.len(), 16_000);
        let up = resample(&vec![0.0; 8_000], 8_000, 16_000);
        assert_eq!(up.len(), 16_000);
    }

    #[test]
    fn speech_band_tone_keeps_its_pitch_after_44k1_to_16k() {
        let out = resample(&sine(1_000.0, 44_100, 1.0), 44_100, 16_000);
        let crossings = zero_crossings(&out);
        assert!((1_990..=2_010).contains(&crossings), "{crossings}");
        assert!(rms(&out[200..15_800]) > 0.65);
    }

    #[test]
    fn tone_above_new_nyquist_is_filtered_instead_of_aliasing_into_speech_band() {
        let out = resample(&sine(12_000.0, 48_000, 1.0), 48_000, 16_000);
        assert!(rms(&out[200..15_800]) < 0.05, "{}", rms(&out));
    }

    #[test]
    fn upsampling_keeps_frequency() {
        let out = resample(&sine(440.0, 8_000, 1.0), 8_000, 16_000);
        let crossings = zero_crossings(&out);
        assert!((875..=885).contains(&crossings), "{crossings}");
    }

    #[test]
    fn downmix_averages_channels_so_opposite_phase_cancels() {
        let mono = downmix(&[vec![0.5, 1.0], vec![-0.5, 0.0]]);
        assert_eq!(mono, vec![0.0, 0.5]);
    }
}
