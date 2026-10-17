use super::complex::Complex;
use super::fft::Transform;
use super::window::Window;
use crate::error::DspError;
use serde::Serialize;

pub const FLOOR_DB: f64 = -120.0;
const MIN_FREQ: f64 = 20.0;
const MAX_BARS: usize = 512;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SpectrumFrame {
    pub fft_size: usize,
    pub sample_rate: f32,
    pub bars: Vec<f32>,
    pub edges: Vec<f32>,
    pub peak_hz: f32,
    pub peak_db: f32,
}

pub fn magnitudes_db(
    transform: &dyn Transform,
    window: &dyn Window,
    samples: &[f32],
) -> Result<Vec<f64>, DspError> {
    let coeffs = window.coefficients(samples.len());
    let gain: f64 = coeffs.iter().sum();
    let mut data: Vec<Complex> = samples
        .iter()
        .zip(coeffs.iter())
        .map(|(s, w)| Complex::new(*s as f64 * w, 0.0))
        .collect();
    transform.forward(&mut data)?;
    let half = data.len() / 2;
    Ok(data[..=half]
        .iter()
        .map(|c| to_db(2.0 * c.norm() / gain))
        .collect())
}

fn to_db(amplitude: f64) -> f64 {
    if amplitude <= 0.0 {
        return FLOOR_DB;
    }
    (20.0 * amplitude.log10()).max(FLOOR_DB)
}

pub fn log_edges(sample_rate: f64, bars: usize) -> Vec<f64> {
    let max = sample_rate / 2.0;
    let ratio = max / MIN_FREQ;
    (0..=bars)
        .map(|i| MIN_FREQ * ratio.powf(i as f64 / bars as f64))
        .collect()
}

fn value_at(db: &[f64], bin: f64) -> f64 {
    let last = db.len() - 1;
    let lo = (bin.floor() as usize).min(last);
    let hi = (lo + 1).min(last);
    let t = bin - lo as f64;
    db[lo] * (1.0 - t) + db[hi] * t
}

fn bin_bars(db: &[f64], edges: &[f64], bin_hz: f64) -> Vec<f64> {
    edges
        .windows(2)
        .map(|e| {
            let lo = (e[0] / bin_hz).ceil() as usize;
            let hi = ((e[1] / bin_hz).ceil() as usize).min(db.len());
            if lo < hi {
                db[lo..hi].iter().cloned().fold(FLOOR_DB, f64::max)
            } else {
                value_at(db, (e[0] * e[1]).sqrt() / bin_hz)
            }
        })
        .collect()
}

fn peak(db: &[f64], bin_hz: f64) -> (f64, f64) {
    let mut best = 1.min(db.len() - 1);
    for (i, v) in db.iter().enumerate().skip(1) {
        if *v > db[best] {
            best = i;
        }
    }
    if best == 0 || best + 1 >= db.len() {
        return (best as f64 * bin_hz, db[best]);
    }
    let (a, b, c) = (db[best - 1], db[best], db[best + 1]);
    let denom = a - 2.0 * b + c;
    let offset = if denom.abs() > 1e-12 {
        (0.5 * (a - c) / denom).clamp(-0.5, 0.5)
    } else {
        0.0
    };
    ((best as f64 + offset) * bin_hz, b)
}

pub fn analyze(
    transform: &dyn Transform,
    window: &dyn Window,
    samples: &[f32],
    sample_rate: f32,
    bars: usize,
) -> Result<SpectrumFrame, DspError> {
    if !(sample_rate.is_finite() && sample_rate > 0.0) {
        return Err(DspError::InvalidSampleRate(sample_rate));
    }
    if bars == 0 || bars > MAX_BARS {
        return Err(DspError::InvalidBarCount(bars));
    }
    let db = magnitudes_db(transform, window, samples)?;
    let bin_hz = sample_rate as f64 / samples.len() as f64;
    let edges = log_edges(sample_rate as f64, bars);
    let bar_values = bin_bars(&db, &edges, bin_hz);
    let (peak_hz, peak_db) = peak(&db, bin_hz);
    Ok(SpectrumFrame {
        fft_size: samples.len(),
        sample_rate,
        bars: bar_values.iter().map(|v| *v as f32).collect(),
        edges: edges.iter().map(|v| *v as f32).collect(),
        peak_hz: peak_hz as f32,
        peak_db: peak_db as f32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::fft::Radix2Fft;
    use crate::dsp::window::{Hann, Rect};
    use std::f32::consts::PI;

    fn tone(freq: f32, rate: f32, n: usize, amp: f32) -> Vec<f32> {
        (0..n)
            .map(|i| amp * (2.0 * PI * freq * i as f32 / rate).sin())
            .collect()
    }

    #[test]
    fn peak_readout_finds_a_1khz_tone_within_one_bin() {
        let rate = 48000.0;
        let frame = analyze(&Radix2Fft, &Hann, &tone(1000.0, rate, 2048, 0.5), rate, 64).unwrap();
        let bin_hz = rate / 2048.0;
        assert!(
            (frame.peak_hz - 1000.0).abs() < bin_hz,
            "peak at {} Hz",
            frame.peak_hz
        );
    }

    #[test]
    fn full_scale_exact_bin_sine_reads_zero_db() {
        let rate = 8192.0;
        let frame = analyze(&Radix2Fft, &Rect, &tone(512.0, rate, 1024, 1.0), rate, 32).unwrap();
        assert!(frame.peak_db.abs() < 0.01, "got {} dB", frame.peak_db);
        let half = analyze(&Radix2Fft, &Rect, &tone(512.0, rate, 1024, 0.5), rate, 32).unwrap();
        assert!(
            (half.peak_db + 6.02).abs() < 0.05,
            "half amplitude is -6 dB"
        );
    }

    #[test]
    fn bars_are_log_spaced_from_20hz_to_nyquist() {
        let edges = log_edges(48000.0, 10);
        assert_eq!(edges.len(), 11);
        assert!((edges[0] - 20.0).abs() < 1e-9);
        assert!((edges[10] - 24000.0).abs() < 1e-6);
        let r0 = edges[1] / edges[0];
        let r9 = edges[10] / edges[9];
        assert!(
            (r0 - r9).abs() < 1e-9,
            "every bar spans the same octave ratio"
        );
    }

    #[test]
    fn the_bar_containing_the_tone_is_the_loudest_bar() {
        let rate = 48000.0;
        let frame = analyze(&Radix2Fft, &Hann, &tone(3000.0, rate, 4096, 0.8), rate, 48).unwrap();
        let loudest = frame
            .bars
            .iter()
            .enumerate()
            .fold(0, |b, (i, v)| if *v > frame.bars[b] { i } else { b });
        assert!(frame.edges[loudest] <= 3000.0 && 3000.0 < frame.edges[loudest + 1]);
    }

    #[test]
    fn silence_sits_at_the_floor() {
        let frame = analyze(&Radix2Fft, &Hann, &vec![0.0; 512], 44100.0, 16).unwrap();
        assert!(
            frame
                .bars
                .iter()
                .all(|v| (*v as f64 - FLOOR_DB).abs() < 1e-6)
        );
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        let s = vec![0.0; 512];
        assert!(matches!(
            analyze(&Radix2Fft, &Hann, &s, 0.0, 16),
            Err(DspError::InvalidSampleRate(_))
        ));
        assert!(matches!(
            analyze(&Radix2Fft, &Hann, &s, 44100.0, 0),
            Err(DspError::InvalidBarCount(0))
        ));
        assert!(matches!(
            analyze(&Radix2Fft, &Hann, &s[..300], 44100.0, 16),
            Err(DspError::NotPowerOfTwo(300))
        ));
    }
}
