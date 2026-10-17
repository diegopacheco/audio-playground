use crate::error::DspError;
use std::f64::consts::PI;

pub trait Window: Send + Sync {
    fn weight(&self, i: usize, n: usize) -> f64;

    fn coefficients(&self, n: usize) -> Vec<f64> {
        (0..n).map(|i| self.weight(i, n)).collect()
    }
}

pub struct Rect;
pub struct Hann;
pub struct Hamming;

impl Window for Rect {
    fn weight(&self, _i: usize, _n: usize) -> f64 {
        1.0
    }
}

impl Window for Hann {
    fn weight(&self, i: usize, n: usize) -> f64 {
        if n < 2 {
            return 1.0;
        }
        0.5 - 0.5 * (2.0 * PI * i as f64 / n as f64).cos()
    }
}

impl Window for Hamming {
    fn weight(&self, i: usize, n: usize) -> f64 {
        if n < 2 {
            return 1.0;
        }
        0.54 - 0.46 * (2.0 * PI * i as f64 / n as f64).cos()
    }
}

pub fn window_by_name(name: &str) -> Result<Box<dyn Window>, DspError> {
    match name.to_ascii_lowercase().as_str() {
        "rect" => Ok(Box::new(Rect)),
        "hann" => Ok(Box::new(Hann)),
        "hamming" => Ok(Box::new(Hamming)),
        other => Err(DspError::UnknownWindow(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::complex::Complex;
    use crate::dsp::fft::{Radix2Fft, Transform};

    fn far_leakage(window: &dyn Window) -> f64 {
        let n = 1024;
        let cycles = 100.5;
        let coeffs = window.coefficients(n);
        let mut data: Vec<Complex> = (0..n)
            .map(|i| {
                let s = (2.0 * PI * cycles * i as f64 / n as f64).sin();
                Complex::new(s * coeffs[i], 0.0)
            })
            .collect();
        Radix2Fft.forward(&mut data).unwrap();
        let mags: Vec<f64> = data[..n / 2].iter().map(|c| c.norm()).collect();
        let peak = mags.iter().cloned().fold(0.0, f64::max);
        let far: f64 = mags
            .iter()
            .enumerate()
            .filter(|(i, _)| (*i as f64 - cycles).abs() > 10.0)
            .map(|(_, m)| m * m)
            .sum();
        far / (peak * peak)
    }

    #[test]
    fn hann_and_hamming_cut_leakage_of_an_off_bin_sine_far_below_rect() {
        let rect = far_leakage(&Rect);
        let hann = far_leakage(&Hann);
        let hamming = far_leakage(&Hamming);
        assert!(hann < rect / 100.0, "hann {hann} vs rect {rect}");
        assert!(hamming < rect / 10.0, "hamming {hamming} vs rect {rect}");
    }

    #[test]
    fn tapered_windows_start_near_zero_and_peak_at_center() {
        let n = 64;
        assert!(Hann.weight(0, n).abs() < 1e-12);
        assert!((Hann.weight(n / 2, n) - 1.0).abs() < 1e-12);
        assert!((Hamming.weight(0, n) - 0.08).abs() < 1e-12);
        assert!(Rect.coefficients(n).iter().all(|w| *w == 1.0));
    }

    #[test]
    fn window_names_are_parsed_and_unknown_names_rejected() {
        assert!(window_by_name("Hann").is_ok());
        assert!(window_by_name("hamming").is_ok());
        assert!(window_by_name("rect").is_ok());
        assert!(matches!(
            window_by_name("kaiser"),
            Err(DspError::UnknownWindow(_))
        ));
    }
}
