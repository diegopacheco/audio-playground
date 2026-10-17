use crate::error::VoiceError;
use std::f64::consts::PI;
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    pub fn from_polar(magnitude: f32, phase: f32) -> Self {
        Self::new(magnitude * phase.cos(), magnitude * phase.sin())
    }

    pub fn norm(self) -> f32 {
        self.re.hypot(self.im)
    }

    pub fn arg(self) -> f32 {
        self.im.atan2(self.re)
    }

    pub fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }
}

impl Add for Complex {
    type Output = Complex;
    fn add(self, other: Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }
}

impl Sub for Complex {
    type Output = Complex;
    fn sub(self, other: Complex) -> Complex {
        Complex::new(self.re - other.re, self.im - other.im)
    }
}

impl Mul for Complex {
    type Output = Complex;
    fn mul(self, other: Complex) -> Complex {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
}

pub struct Fft {
    size: usize,
    twiddles: Vec<Complex>,
    reversed: Vec<usize>,
}

impl Fft {
    pub fn new(size: usize) -> Result<Self, VoiceError> {
        if size < 2 || !size.is_power_of_two() {
            return Err(VoiceError::InvalidFftSize(size));
        }
        let twiddles = (0..size / 2)
            .map(|k| {
                let angle = -2.0 * PI * k as f64 / size as f64;
                Complex::new(angle.cos() as f32, angle.sin() as f32)
            })
            .collect();
        let shift = usize::BITS - size.trailing_zeros();
        let reversed = (0..size).map(|i| i.reverse_bits() >> shift).collect();
        Ok(Self {
            size,
            twiddles,
            reversed,
        })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn forward(&self, data: &mut [Complex]) {
        self.transform(data, false);
    }

    pub fn inverse(&self, data: &mut [Complex]) {
        self.transform(data, true);
        let scale = 1.0 / self.size as f32;
        for value in data.iter_mut() {
            value.re *= scale;
            value.im *= scale;
        }
    }

    fn transform(&self, data: &mut [Complex], inverse: bool) {
        let n = self.size.min(data.len());
        for i in 0..n {
            let j = self.reversed[i];
            if i < j {
                data.swap(i, j);
            }
        }
        let mut len = 2;
        while len <= n {
            let half = len / 2;
            let stride = self.size / len;
            for start in (0..n).step_by(len) {
                for k in 0..half {
                    let twiddle = self.twiddles[k * stride];
                    let twiddle = if inverse { twiddle.conj() } else { twiddle };
                    let a = data[start + k];
                    let b = data[start + k + half] * twiddle;
                    data[start + k] = a + b;
                    data[start + k + half] = a - b;
                }
            }
            len <<= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_sizes_that_are_not_powers_of_two() {
        assert!(matches!(
            Fft::new(1000),
            Err(VoiceError::InvalidFftSize(1000))
        ));
        assert!(Fft::new(1024).is_ok());
    }

    #[test]
    fn sine_on_bin_puts_all_energy_on_that_bin_and_its_mirror() {
        let n = 64;
        let fft = Fft::new(n).unwrap();
        let mut data: Vec<Complex> = (0..n)
            .map(|i| {
                Complex::new(
                    (2.0 * std::f32::consts::PI * 5.0 * i as f32 / n as f32).sin(),
                    0.0,
                )
            })
            .collect();
        fft.forward(&mut data);
        for (k, value) in data.iter().enumerate() {
            let expected = if k == 5 || k == n - 5 {
                n as f32 / 2.0
            } else {
                0.0
            };
            assert!((value.norm() - expected).abs() < 1e-3, "bin {k}");
        }
    }

    #[test]
    fn inverse_undoes_forward_so_stft_resynthesis_is_lossless() {
        let fft = Fft::new(256).unwrap();
        let original: Vec<Complex> = (0..256)
            .map(|i| Complex::new(((i * 37) % 11) as f32 - 5.0, 0.0))
            .collect();
        let mut data = original.clone();
        fft.forward(&mut data);
        fft.inverse(&mut data);
        for (a, b) in data.iter().zip(&original) {
            assert!((a.re - b.re).abs() < 1e-4 && a.im.abs() < 1e-4);
        }
    }
}
