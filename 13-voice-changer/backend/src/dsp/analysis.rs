use crate::dsp::fft::{Complex, Fft};
use crate::dsp::window;

pub fn dominant_frequency(samples: &[f32], rate: u32) -> f32 {
    let size = samples.len().next_power_of_two() * 4;
    let fft = Fft::new(size).unwrap();
    let window = window::hann(samples.len());
    let mut data = vec![Complex::default(); size];
    for ((slot, sample), w) in data.iter_mut().zip(samples).zip(&window) {
        *slot = Complex::new(sample * w, 0.0);
    }
    fft.forward(&mut data);
    let magnitudes: Vec<f32> = data[..size / 2].iter().map(|c| c.norm()).collect();
    let (peak, _) = magnitudes
        .iter()
        .enumerate()
        .skip(1)
        .fold(
            (0, 0.0),
            |best, (k, m)| if *m > best.1 { (k, *m) } else { best },
        );
    let (a, b, c) = (magnitudes[peak - 1], magnitudes[peak], magnitudes[peak + 1]);
    let offset = 0.5 * (a - c) / (a - 2.0 * b + c);
    (peak as f32 + offset) * rate as f32 / size as f32
}

pub fn periodicity(samples: &[f32], rate: u32, min_hz: f32, max_hz: f32) -> (f32, f32) {
    let min_lag = (rate as f32 / max_hz) as usize;
    let max_lag = (rate as f32 / min_hz) as usize;
    let span = samples.len() - max_lag;
    let mut best = (0, 0.0);
    for lag in min_lag..=max_lag {
        let (mut cross, mut a, mut b) = (0.0f64, 0.0f64, 0.0f64);
        for i in 0..span {
            let x = samples[i] as f64;
            let y = samples[i + lag] as f64;
            cross += x * y;
            a += x * x;
            b += y * y;
        }
        let score = cross / (a * b).sqrt().max(1e-12);
        if score > best.1 {
            best = (lag, score);
        }
    }
    (rate as f32 / best.0 as f32, best.1 as f32)
}

pub fn rms(samples: &[f32]) -> f32 {
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}
