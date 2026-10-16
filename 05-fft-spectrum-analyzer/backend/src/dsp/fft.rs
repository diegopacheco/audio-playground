use super::complex::Complex;
use crate::error::DspError;
use std::f64::consts::PI;

pub trait Transform {
    fn forward(&self, data: &mut [Complex]) -> Result<(), DspError>;
    fn inverse(&self, data: &mut [Complex]) -> Result<(), DspError>;
}

pub struct Radix2Fft;

impl Radix2Fft {
    fn check(len: usize) -> Result<(), DspError> {
        if len == 0 {
            return Err(DspError::EmptyInput);
        }
        if !len.is_power_of_two() {
            return Err(DspError::NotPowerOfTwo(len));
        }
        Ok(())
    }

    fn bit_reverse(data: &mut [Complex]) {
        let n = data.len();
        let bits = n.trailing_zeros();
        if bits == 0 {
            return;
        }
        for i in 0..n {
            let j = i.reverse_bits() >> (usize::BITS - bits);
            if j > i {
                data.swap(i, j);
            }
        }
    }

    fn butterflies(data: &mut [Complex], sign: f64) {
        let n = data.len();
        let mut size = 2;
        while size <= n {
            let half = size / 2;
            let step = Complex::from_polar(1.0, sign * 2.0 * PI / size as f64);
            for start in (0..n).step_by(size) {
                let mut w = Complex::new(1.0, 0.0);
                for k in 0..half {
                    let a = data[start + k];
                    let b = data[start + k + half] * w;
                    data[start + k] = a + b;
                    data[start + k + half] = a - b;
                    w = w * step;
                }
            }
            size *= 2;
        }
    }
}

impl Transform for Radix2Fft {
    fn forward(&self, data: &mut [Complex]) -> Result<(), DspError> {
        Self::check(data.len())?;
        Self::bit_reverse(data);
        Self::butterflies(data, -1.0);
        Ok(())
    }

    fn inverse(&self, data: &mut [Complex]) -> Result<(), DspError> {
        Self::check(data.len())?;
        Self::bit_reverse(data);
        Self::butterflies(data, 1.0);
        let k = 1.0 / data.len() as f64;
        for c in data.iter_mut() {
            *c = c.scale(k);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(n: usize, cycles: f64) -> Vec<Complex> {
        (0..n)
            .map(|i| Complex::new((2.0 * PI * cycles * i as f64 / n as f64).sin(), 0.0))
            .collect()
    }

    fn peak_bin(data: &[Complex]) -> usize {
        let half = &data[..data.len() / 2];
        let mut best = 0;
        for (i, c) in half.iter().enumerate() {
            if c.norm() > half[best].norm() {
                best = i;
            }
        }
        best
    }

    #[test]
    fn sine_with_k_cycles_concentrates_all_energy_in_bin_k() {
        for &k in &[1usize, 5, 37, 100] {
            let mut data = sine(256, k as f64);
            Radix2Fft.forward(&mut data).unwrap();
            assert_eq!(peak_bin(&data), k, "peak must sit at bin {k}");
            assert!(
                (data[k].norm() - 128.0).abs() < 1e-6,
                "amplitude N/2 at bin k"
            );
            let others: f64 = data[..128]
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != k)
                .map(|(_, c)| c.norm())
                .sum();
            assert!(others < 1e-6, "no leakage for an exact-bin sine");
        }
    }

    #[test]
    fn matches_naive_dft_so_butterflies_and_twiddles_are_correct() {
        let n = 32;
        let input: Vec<Complex> = (0..n)
            .map(|i| Complex::new(((i * 7) % 11) as f64 - 5.0, ((i * 3) % 5) as f64))
            .collect();
        let mut fast = input.clone();
        Radix2Fft.forward(&mut fast).unwrap();
        for (k, f) in fast.iter().enumerate() {
            let mut acc = Complex::default();
            for (t, x) in input.iter().enumerate() {
                acc = acc + *x * Complex::from_polar(1.0, -2.0 * PI * (k * t) as f64 / n as f64);
            }
            assert!((acc - *f).norm() < 1e-9, "bin {k} differs from naive DFT");
        }
    }

    #[test]
    fn parseval_energy_in_time_equals_energy_in_frequency_over_n() {
        let n = 1024;
        let input: Vec<Complex> = (0..n)
            .map(|i| {
                let t = i as f64;
                Complex::new((t * 0.37).sin() + 0.5 * (t * 1.91).cos() + 0.1, 0.0)
            })
            .collect();
        let time_energy: f64 = input.iter().map(|c| c.norm_sqr()).sum();
        let mut spec = input.clone();
        Radix2Fft.forward(&mut spec).unwrap();
        let freq_energy: f64 = spec.iter().map(|c| c.norm_sqr()).sum::<f64>() / n as f64;
        assert!(
            (time_energy - freq_energy).abs() / time_energy < 1e-9,
            "FFT must preserve energy: {time_energy} vs {freq_energy}"
        );
    }

    #[test]
    fn inverse_of_forward_returns_the_original_signal() {
        let input: Vec<Complex> = (0..512)
            .map(|i| Complex::new((i as f64 * 0.11).sin(), (i as f64 * 0.07).cos()))
            .collect();
        let mut data = input.clone();
        Radix2Fft.forward(&mut data).unwrap();
        Radix2Fft.inverse(&mut data).unwrap();
        for (a, b) in input.iter().zip(data.iter()) {
            assert!((*a - *b).norm() < 1e-9, "round trip must be lossless");
        }
    }

    #[test]
    fn rejects_lengths_that_radix2_cannot_split() {
        let mut data = vec![Complex::default(); 100];
        assert_eq!(
            Radix2Fft.forward(&mut data),
            Err(DspError::NotPowerOfTwo(100))
        );
        let mut empty: Vec<Complex> = Vec::new();
        assert_eq!(Radix2Fft.forward(&mut empty), Err(DspError::EmptyInput));
    }
}
