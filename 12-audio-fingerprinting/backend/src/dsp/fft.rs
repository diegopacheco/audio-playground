use crate::error::FingerprintError;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub fn norm(self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

pub fn fft(buffer: &mut [Complex]) -> Result<(), FingerprintError> {
    let n = buffer.len();
    if n == 0 || !n.is_power_of_two() {
        return Err(FingerprintError::InvalidParameter(format!(
            "fft size {n} must be a power of two"
        )));
    }
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            buffer.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let angle = -2.0 * PI / len as f32;
        let half = len / 2;
        for start in (0..n).step_by(len) {
            for k in 0..half {
                let (s, c) = (angle * k as f32).sin_cos();
                let a = buffer[start + k];
                let b = buffer[start + k + half];
                let t = Complex {
                    re: b.re * c - b.im * s,
                    im: b.re * s + b.im * c,
                };
                buffer[start + k] = Complex {
                    re: a.re + t.re,
                    im: a.im + t.im,
                };
                buffer[start + k + half] = Complex {
                    re: a.re - t.re,
                    im: a.im - t.im,
                };
            }
        }
        len <<= 1;
    }
    Ok(())
}

pub fn hann(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / size as f32).cos())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn real(values: &[f32]) -> Vec<Complex> {
        values.iter().map(|&re| Complex { re, im: 0.0 }).collect()
    }

    #[test]
    fn sine_on_bin_16_concentrates_energy_in_bin_16() {
        let n = 256;
        let mut buf = real(
            &(0..n)
                .map(|i| (2.0 * PI * 16.0 * i as f32 / n as f32).sin())
                .collect::<Vec<_>>(),
        );
        fft(&mut buf).unwrap();
        let peak = (0..n / 2)
            .max_by(|&a, &b| buf[a].norm().total_cmp(&buf[b].norm()))
            .unwrap();
        assert_eq!(peak, 16);
        assert!((buf[16].norm() - n as f32 / 2.0).abs() < 0.5);
        assert!(buf[40].norm() < 1e-2);
    }

    #[test]
    fn parseval_energy_is_preserved() {
        let samples: Vec<f32> = (0..64).map(|i| ((i * 7919) % 13) as f32 - 6.0).collect();
        let time: f32 = samples.iter().map(|v| v * v).sum();
        let mut buf = real(&samples);
        fft(&mut buf).unwrap();
        let freq: f32 = buf.iter().map(|c| c.norm() * c.norm()).sum::<f32>() / 64.0;
        assert!((time - freq).abs() / time < 1e-4);
    }

    #[test]
    fn non_power_of_two_is_rejected() {
        let mut buf = real(&[0.0; 100]);
        assert!(fft(&mut buf).is_err());
    }

    #[test]
    fn hann_window_tapers_edges_to_zero_and_peaks_at_center() {
        let w = hann(1024);
        assert!(w[0].abs() < 1e-6);
        assert!((w[512] - 1.0).abs() < 1e-6);
    }
}
