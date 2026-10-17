use crate::dsp::fft::{Complex, fft, hann};
use crate::error::FingerprintError;

#[derive(Debug, Clone, PartialEq)]
pub struct Spectrum {
    pub frames: Vec<Vec<f32>>,
    pub bins: usize,
    pub hop: usize,
    pub sample_rate: u32,
}

impl Spectrum {
    pub fn frame_seconds(&self) -> f64 {
        self.hop as f64 / self.sample_rate as f64
    }

    pub fn bin_hz(&self) -> f64 {
        self.sample_rate as f64 / (2 * (self.bins - 1)) as f64
    }
}

pub trait Spectrogram: Send + Sync {
    fn compute(&self, samples: &[f32], sample_rate: u32) -> Result<Spectrum, FingerprintError>;
}

pub struct Stft {
    pub frame_size: usize,
    pub hop: usize,
}

impl Stft {
    pub fn new(frame_size: usize, hop: usize) -> Result<Self, FingerprintError> {
        if !frame_size.is_power_of_two() || hop == 0 || hop > frame_size {
            return Err(FingerprintError::InvalidParameter(format!(
                "frame {frame_size} hop {hop}"
            )));
        }
        Ok(Stft { frame_size, hop })
    }
}

impl Spectrogram for Stft {
    fn compute(&self, samples: &[f32], sample_rate: u32) -> Result<Spectrum, FingerprintError> {
        if sample_rate == 0 {
            return Err(FingerprintError::InvalidSampleRate(0));
        }
        let window = hann(self.frame_size);
        let bins = self.frame_size / 2 + 1;
        let mut frames = Vec::new();
        let mut buffer = vec![Complex { re: 0.0, im: 0.0 }; self.frame_size];
        let mut start = 0;
        while start + self.frame_size <= samples.len() {
            for (i, slot) in buffer.iter_mut().enumerate() {
                *slot = Complex {
                    re: samples[start + i] * window[i],
                    im: 0.0,
                };
            }
            fft(&mut buffer)?;
            frames.push(
                buffer[..bins]
                    .iter()
                    .map(|c| 20.0 * (c.norm() + 1e-6).log10())
                    .collect(),
            );
            start += self.hop;
        }
        Ok(Spectrum {
            frames,
            bins,
            hop: self.hop,
            sample_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn tone_at_1000hz_peaks_in_matching_bin_of_every_frame() {
        let rate = 8000;
        let samples: Vec<f32> = (0..8000)
            .map(|i| (2.0 * PI * 1000.0 * i as f32 / rate as f32).sin())
            .collect();
        let spec = Stft::new(1024, 256)
            .unwrap()
            .compute(&samples, rate)
            .unwrap();
        assert_eq!(spec.frames.len(), (8000 - 1024) / 256 + 1);
        let expected = (1000.0 / spec.bin_hz()).round() as usize;
        for frame in &spec.frames {
            let peak = (0..spec.bins)
                .max_by(|&a, &b| frame[a].total_cmp(&frame[b]))
                .unwrap();
            assert_eq!(peak, expected);
        }
    }

    #[test]
    fn audio_shorter_than_one_frame_yields_no_frames() {
        let spec = Stft::new(1024, 256)
            .unwrap()
            .compute(&[0.1; 500], 8000)
            .unwrap();
        assert!(spec.frames.is_empty());
    }
}
