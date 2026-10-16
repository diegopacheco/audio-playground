use crate::dsp::fft::{Complex, Fft};
use crate::dsp::window;
use crate::error::VoiceError;

pub trait SpectralProcessor {
    fn process(&mut self, bins: &mut [Complex]);
}

pub struct StreamingStft<P: SpectralProcessor> {
    fft: Fft,
    window: Vec<f32>,
    analysis_hop: usize,
    synthesis_hop: usize,
    gain: f32,
    pending: Vec<f32>,
    overlap: Vec<f32>,
    frame: Vec<Complex>,
    processor: P,
}

impl<P: SpectralProcessor> StreamingStft<P> {
    pub fn new(
        size: usize,
        analysis_hop: usize,
        synthesis_hop: usize,
        gain: f32,
        processor: P,
    ) -> Result<Self, VoiceError> {
        let fft = Fft::new(size)?;
        for hop in [analysis_hop, synthesis_hop] {
            if hop == 0 || hop > size {
                return Err(VoiceError::InvalidHop { size, hop });
            }
        }
        Ok(Self {
            fft,
            window: window::hann(size),
            analysis_hop,
            synthesis_hop,
            gain,
            pending: Vec::with_capacity(size * 2),
            overlap: vec![0.0; size],
            frame: vec![Complex::default(); size],
            processor,
        })
    }

    pub fn coherent_gain(size: usize, synthesis_hop: usize) -> f32 {
        synthesis_hop as f32 / window::energy(&window::hann(size))
    }

    pub fn push(&mut self, samples: &[f32], out: &mut Vec<f32>) {
        self.pending.extend_from_slice(samples);
        let size = self.fft.size();
        let half = size / 2;
        while self.pending.len() >= size {
            for ((slot, sample), w) in self.frame.iter_mut().zip(&self.pending).zip(&self.window) {
                *slot = Complex::new(sample * w, 0.0);
            }
            self.fft.forward(&mut self.frame);
            self.processor.process(&mut self.frame[..=half]);
            for k in 1..half {
                self.frame[size - k] = self.frame[k].conj();
            }
            self.fft.inverse(&mut self.frame);
            for ((acc, value), w) in self.overlap.iter_mut().zip(&self.frame).zip(&self.window) {
                *acc += value.re * w * self.gain;
            }
            let hop = self.synthesis_hop;
            out.extend_from_slice(&self.overlap[..hop]);
            self.overlap.copy_within(hop.., 0);
            self.overlap[size - hop..].fill(0.0);
            self.pending.drain(..self.analysis_hop);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Identity;

    impl SpectralProcessor for Identity {
        fn process(&mut self, _bins: &mut [Complex]) {}
    }

    #[test]
    fn untouched_spectrum_reconstructs_the_input_so_effects_only_add_their_own_change() {
        let size = 512;
        let hop = 128;
        let gain = StreamingStft::<Identity>::coherent_gain(size, hop);
        let mut stft = StreamingStft::new(size, hop, hop, gain, Identity).unwrap();
        let input: Vec<f32> = (0..8000).map(|i| (i as f32 * 0.05).sin() * 0.5).collect();
        let mut out = Vec::new();
        for block in input.chunks(100) {
            stft.push(block, &mut out);
        }
        for i in size..out.len() {
            assert!((out[i] - input[i]).abs() < 1e-3, "sample {i}");
        }
    }

    #[test]
    fn rejects_hops_larger_than_the_frame() {
        let result = StreamingStft::new(256, 300, 64, 1.0, Identity);
        assert!(matches!(
            result,
            Err(VoiceError::InvalidHop {
                size: 256,
                hop: 300
            })
        ));
    }
}
