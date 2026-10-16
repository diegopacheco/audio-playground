pub fn to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    samples
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |m, s| m.max(s.abs()))
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    (sum / samples.len() as f64).sqrt() as f32
}

pub fn buckets(len: usize, count: usize) -> Vec<(usize, usize)> {
    (0..count)
        .map(|i| {
            (
                i * len / count,
                ((i + 1) * len / count).max(i * len / count),
            )
        })
        .collect()
}

pub fn envelope(mono: &[f32], points: usize) -> Vec<f32> {
    if mono.is_empty() || points == 0 {
        return Vec::new();
    }
    buckets(mono.len(), points.min(mono.len()))
        .into_iter()
        .map(|(start, end)| peak(&mono[start..end]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn sine(amp: f32, freq: f32, rate: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| amp * (2.0 * PI * freq * i as f32 / rate).sin())
            .collect()
    }

    #[test]
    fn sine_peak_equals_its_amplitude() {
        let s = sine(0.5, 100.0, 8000.0, 8000);
        assert!((peak(&s) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn sine_rms_is_amplitude_over_root_two() {
        let s = sine(0.8, 100.0, 8000.0, 8000);
        assert!((rms(&s) - 0.8 / 2f32.sqrt()).abs() < 1e-3);
    }

    #[test]
    fn square_wave_rms_equals_peak() {
        let s: Vec<f32> = (0..1000)
            .map(|i| if i % 20 < 10 { 0.6 } else { -0.6 })
            .collect();
        assert!((rms(&s) - peak(&s)).abs() < 1e-6);
    }

    #[test]
    fn silence_has_zero_levels() {
        assert_eq!(peak(&[0.0; 10]), 0.0);
        assert_eq!(rms(&[]), 0.0);
    }

    #[test]
    fn opposite_phase_stereo_cancels_in_mono() {
        assert_eq!(to_mono(&[0.5, -0.5, 1.0, 0.0], 2), vec![0.0, 0.5]);
    }

    #[test]
    fn envelope_follows_a_fade_out() {
        let n = 4000;
        let s: Vec<f32> = sine(1.0, 200.0, 8000.0, n)
            .iter()
            .enumerate()
            .map(|(i, v)| v * (1.0 - i as f32 / n as f32))
            .collect();
        let env = envelope(&s, 10);
        assert_eq!(env.len(), 10);
        assert!(env.windows(2).all(|w| w[1] < w[0]));
    }

    #[test]
    fn envelope_never_has_more_points_than_samples() {
        assert_eq!(envelope(&[0.1, 0.2, 0.3], 100).len(), 3);
    }
}
