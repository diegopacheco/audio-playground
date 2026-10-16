use std::f32::consts::PI;

pub fn zero_crossing_rate(samples: &[f32], sample_rate: u32) -> f32 {
    let crossings = samples
        .windows(2)
        .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
        .count();
    crossings as f32 * sample_rate as f32 / samples.len() as f32
}

pub fn band_energy_ratio(samples: &[f32], sample_rate: u32, lo: f32, hi: f32) -> f32 {
    let n = samples.len().min(2048);
    let bin_hz = sample_rate as f32 / n as f32;
    let mut band = 0.0;
    let mut total = 0.0;
    for k in 1..n / 2 {
        let (mut re, mut im) = (0.0f32, 0.0f32);
        for (i, s) in samples[..n].iter().enumerate() {
            let a = 2.0 * PI * k as f32 * i as f32 / n as f32;
            re += s * a.cos();
            im -= s * a.sin();
        }
        let power = re * re + im * im;
        total += power;
        let f = k as f32 * bin_hz;
        if f >= lo && f < hi {
            band += power;
        }
    }
    if total == 0.0 { 0.0 } else { band / total }
}
