use crate::error::AudioError;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    pub fn norm(self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    fn mul(self, o: Complex) -> Complex {
        Complex::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
}

fn bit_reverse(buf: &mut [Complex]) {
    let n = buf.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            buf.swap(i, j);
        }
    }
}

pub fn fft(buf: &mut [Complex]) -> Result<(), AudioError> {
    let n = buf.len();
    if n == 0 || !n.is_power_of_two() {
        return Err(AudioError::InvalidParameter(format!(
            "fft size {n} must be a power of two"
        )));
    }
    bit_reverse(buf);
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let twiddles: Vec<Complex> = (0..half)
            .map(|k| {
                let angle = -2.0 * PI * k as f64 / len as f64;
                Complex::new(angle.cos() as f32, angle.sin() as f32)
            })
            .collect();
        for start in (0..n).step_by(len) {
            for (k, w) in twiddles.iter().enumerate() {
                let a = buf[start + k];
                let b = buf[start + k + half].mul(*w);
                buf[start + k] = Complex::new(a.re + b.re, a.im + b.im);
                buf[start + k + half] = Complex::new(a.re - b.re, a.im - b.im);
            }
        }
        len <<= 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_on_bin_eight_puts_all_energy_in_bins_eight_and_mirror() {
        let n = 64;
        let mut buf: Vec<Complex> = (0..n)
            .map(|i| Complex::new((2.0 * PI * 8.0 * i as f64 / n as f64).sin() as f32, 0.0))
            .collect();
        fft(&mut buf).unwrap();
        for (k, c) in buf.iter().enumerate() {
            if k == 8 || k == n - 8 {
                assert!((c.norm() - n as f32 / 2.0).abs() < 1e-3);
            } else {
                assert!(c.norm() < 1e-3, "leak at bin {k}");
            }
        }
    }

    #[test]
    fn matches_naive_dft_on_arbitrary_signal() {
        let n = 16;
        let input: Vec<Complex> = (0..n)
            .map(|i| Complex::new(((i * 7) % 5) as f32 - 2.0, 0.0))
            .collect();
        let mut fast = input.clone();
        fft(&mut fast).unwrap();
        for (k, got) in fast.iter().enumerate() {
            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for (t, x) in input.iter().enumerate() {
                let a = -2.0 * PI * (k * t) as f64 / n as f64;
                re += x.re as f64 * a.cos();
                im += x.re as f64 * a.sin();
            }
            assert!((got.re as f64 - re).abs() < 1e-3);
            assert!((got.im as f64 - im).abs() < 1e-3);
        }
    }

    #[test]
    fn non_power_of_two_size_is_rejected() {
        let mut buf = vec![Complex::default(); 12];
        assert!(fft(&mut buf).is_err());
    }
}
