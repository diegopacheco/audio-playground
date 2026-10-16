use super::sample_at;

fn box_lowpass(samples: &[f32], width: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(samples.len());
    let mut acc = 0.0f32;
    for (i, s) in samples.iter().enumerate() {
        acc += s;
        if i >= width {
            acc -= samples[i - width];
        }
        out.push(acc / width.min(i + 1) as f32);
    }
    out
}

pub fn resample(samples: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || samples.is_empty() {
        return samples.to_vec();
    }
    let ratio = from as f32 / to as f32;
    let width = ratio.round() as usize;
    let source = if width >= 2 {
        box_lowpass(samples, width)
    } else {
        samples.to_vec()
    };
    let out_len = (samples.len() as f64 * to as f64 / from as f64).floor() as usize;
    (0..out_len)
        .map(|i| sample_at(&source, i as f32 * ratio))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn sine(freq: f32, rate: u32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|i| (2.0 * PI * freq * i as f32 / rate as f32).sin())
            .collect()
    }

    fn rms(v: &[f32]) -> f32 {
        (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt()
    }

    #[test]
    fn halving_the_rate_halves_the_length() {
        let out = resample(&vec![0.1; 44100], 44100, 22050);
        assert_eq!(out.len(), 22050);
    }

    #[test]
    fn low_tone_survives_decimation_while_tone_above_new_nyquist_is_attenuated() {
        let low = resample(&sine(200.0, 44100, 44100), 44100, 22050);
        let high = resample(&sine(16000.0, 44100, 44100), 44100, 22050);
        assert!(rms(&low) > 0.65);
        assert!(rms(&high) < 0.4);
    }

    #[test]
    fn upsampling_preserves_duration() {
        let out = resample(&vec![0.0; 8000], 8000, 22050);
        assert_eq!(out.len(), 22050);
    }
}
