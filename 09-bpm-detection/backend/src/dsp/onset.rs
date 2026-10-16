use super::fft::{Complex, fft};
use crate::error::AudioError;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Framing {
    pub frame_size: usize,
    pub hop: usize,
}

impl Framing {
    pub fn count(&self, len: usize) -> usize {
        if len < self.frame_size {
            0
        } else {
            (len - self.frame_size) / self.hop + 1
        }
    }

    fn frames<'a>(&self, samples: &'a [f32]) -> impl Iterator<Item = &'a [f32]> + 'a {
        let Framing { frame_size, hop } = *self;
        (0..self.count(samples.len())).map(move |i| &samples[i * hop..i * hop + frame_size])
    }
}

pub trait OnsetDetector: Send + Sync {
    fn envelope(&self, samples: &[f32]) -> Result<Vec<f32>, AudioError>;
}

fn hann(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / size as f32).cos())
        .collect()
}

const LOG_GAIN: f32 = 10.0;

pub struct SpectralFlux {
    framing: Framing,
    window: Vec<f32>,
}

impl SpectralFlux {
    pub fn new(framing: Framing) -> Self {
        Self {
            framing,
            window: hann(framing.frame_size),
        }
    }

    fn magnitudes(&self, frame: &[f32], buf: &mut [Complex]) -> Result<Vec<f32>, AudioError> {
        for ((slot, s), w) in buf.iter_mut().zip(frame).zip(&self.window) {
            *slot = Complex::new(s * w, 0.0);
        }
        fft(buf)?;
        Ok(buf[..buf.len() / 2 + 1]
            .iter()
            .map(|c| (1.0 + LOG_GAIN * c.norm()).ln())
            .collect())
    }
}

impl OnsetDetector for SpectralFlux {
    fn envelope(&self, samples: &[f32]) -> Result<Vec<f32>, AudioError> {
        let mut buf = vec![Complex::default(); self.framing.frame_size];
        let mut previous = vec![0.0; self.framing.frame_size / 2 + 1];
        let mut out = Vec::with_capacity(self.framing.count(samples.len()));
        for frame in self.framing.frames(samples) {
            let current = self.magnitudes(frame, &mut buf)?;
            let flux = current
                .iter()
                .zip(&previous)
                .map(|(c, p)| (c - p).max(0.0))
                .sum();
            out.push(flux);
            previous = current;
        }
        Ok(out)
    }
}

pub struct EnergyFlux {
    framing: Framing,
    window: Vec<f32>,
}

impl EnergyFlux {
    pub fn new(framing: Framing) -> Self {
        Self {
            framing,
            window: hann(framing.frame_size),
        }
    }
}

impl OnsetDetector for EnergyFlux {
    fn envelope(&self, samples: &[f32]) -> Result<Vec<f32>, AudioError> {
        let energies: Vec<f32> = self
            .framing
            .frames(samples)
            .map(|frame| {
                let e: f32 = frame
                    .iter()
                    .zip(&self.window)
                    .map(|(s, w)| (s * w) * (s * w))
                    .sum();
                (1.0 + LOG_GAIN * e).ln()
            })
            .collect();
        Ok(energies
            .iter()
            .scan(0.0f32, |previous, e| {
                let rise = (e - *previous).max(0.0);
                *previous = *e;
                Some(rise)
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAMING: Framing = Framing {
        frame_size: 512,
        hop: 128,
    };

    fn click_at(len: usize, at: usize) -> Vec<f32> {
        let mut v = vec![0.0; len];
        for (i, s) in v.iter_mut().enumerate().skip(at).take(400) {
            *s = ((i * 7919 % 97) as f32 / 48.5 - 1.0) * 0.8;
        }
        v
    }

    fn peak_index(env: &[f32]) -> usize {
        env.iter()
            .enumerate()
            .fold(
                (0, f32::MIN),
                |b, (i, v)| if *v > b.1 { (i, *v) } else { b },
            )
            .0
    }

    #[test]
    fn frame_count_follows_hop() {
        assert_eq!(FRAMING.count(511), 0);
        assert_eq!(FRAMING.count(512), 1);
        assert_eq!(FRAMING.count(512 + 128 * 10), 11);
    }

    #[test]
    fn spectral_flux_peaks_where_the_burst_starts() {
        let samples = click_at(8192, 4000);
        let env = SpectralFlux::new(FRAMING).envelope(&samples).unwrap();
        let peak_sample = peak_index(&env) * FRAMING.hop;
        assert!(peak_sample + FRAMING.frame_size > 4000 && peak_sample < 4000);
    }

    #[test]
    fn flux_is_half_wave_rectified_so_a_decay_adds_nothing() {
        let mut samples = vec![0.0; 8192];
        for (i, s) in samples.iter_mut().enumerate().take(2000) {
            *s = (i as f32 * 0.3).sin();
        }
        let env = SpectralFlux::new(FRAMING).envelope(&samples).unwrap();
        let tail = 2000 / FRAMING.hop + 1;
        assert!(env.iter().all(|v| *v >= 0.0));
        assert!(env[tail..].iter().all(|v| *v < 1e-6));
    }

    #[test]
    fn energy_flux_also_peaks_at_the_burst() {
        let samples = click_at(8192, 4000);
        let env = EnergyFlux::new(FRAMING).envelope(&samples).unwrap();
        let peak_sample = peak_index(&env) * FRAMING.hop;
        assert!(peak_sample + FRAMING.frame_size > 4000 && peak_sample < 4000);
    }

    #[test]
    fn silence_has_flat_zero_envelope() {
        let env = SpectralFlux::new(FRAMING).envelope(&[0.0; 4096]).unwrap();
        assert!(env.iter().all(|v| *v == 0.0));
    }
}
