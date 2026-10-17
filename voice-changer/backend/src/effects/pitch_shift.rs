use crate::dsp::fft::Complex;
use crate::dsp::resample::StreamingResampler;
use crate::dsp::stft::{SpectralProcessor, StreamingStft};
use crate::effects::VoiceEffect;
use crate::effects::queue::LatencyQueue;
use crate::error::VoiceError;
use std::f32::consts::{PI, TAU};

const FRAME: usize = 2048;
const ANALYSIS_HOP: usize = 256;
const MAX_SEMITONES: f32 = 12.0;

struct PhaseVocoder {
    analysis_hop: usize,
    ratio: f32,
    last_phase: Vec<f32>,
    synth_phase: Vec<f32>,
    magnitude: Vec<f32>,
    phase: Vec<f32>,
    peaks: Vec<usize>,
}

impl PhaseVocoder {
    fn new(analysis_hop: usize, ratio: f32) -> Self {
        let bins = FRAME / 2 + 1;
        Self {
            analysis_hop,
            ratio,
            last_phase: vec![0.0; bins],
            synth_phase: vec![0.0; bins],
            magnitude: vec![0.0; bins],
            phase: vec![0.0; bins],
            peaks: Vec::with_capacity(bins / 2),
        }
    }

    fn find_peaks(&mut self) {
        self.peaks.clear();
        let m = &self.magnitude;
        for k in 1..m.len() - 1 {
            if m[k] > m[k - 1] && m[k] >= m[k + 1] && m[k] > f32::EPSILON {
                self.peaks.push(k);
            }
        }
    }

    fn advance_peak(&mut self, k: usize) {
        let expected = TAU * (k * self.analysis_hop) as f32 / FRAME as f32;
        let true_advance = expected + wrap(self.phase[k] - self.last_phase[k] - expected);
        self.synth_phase[k] = wrap(self.synth_phase[k] + true_advance * self.ratio);
    }

    fn lock_to_peaks(&mut self) {
        let count = self.magnitude.len();
        let mut start = 0;
        for (i, &peak) in self.peaks.iter().enumerate() {
            let end = self
                .peaks
                .get(i + 1)
                .map_or(count, |next| (peak + next).div_ceil(2));
            for k in start..end {
                if k != peak {
                    self.synth_phase[k] = self.synth_phase[peak] + self.phase[k] - self.phase[peak];
                }
            }
            start = end;
        }
    }
}

fn wrap(phase: f32) -> f32 {
    phase - TAU * ((phase + PI) / TAU).floor()
}

impl SpectralProcessor for PhaseVocoder {
    fn process(&mut self, bins: &mut [Complex]) {
        for (k, bin) in bins.iter().enumerate() {
            self.magnitude[k] = bin.norm();
            self.phase[k] = bin.arg();
        }
        self.find_peaks();
        if self.peaks.is_empty() {
            for k in 0..bins.len() {
                self.advance_peak(k);
            }
        } else {
            for i in 0..self.peaks.len() {
                self.advance_peak(self.peaks[i]);
            }
            self.lock_to_peaks();
        }
        for (k, bin) in bins.iter_mut().enumerate() {
            *bin = Complex::from_polar(self.magnitude[k], self.synth_phase[k]);
        }
        self.last_phase.copy_from_slice(&self.phase);
    }
}

pub struct PhaseVocoderPitchShift {
    stft: StreamingStft<PhaseVocoder>,
    resampler: StreamingResampler,
    queue: LatencyQueue,
    latency: usize,
}

impl PhaseVocoderPitchShift {
    pub fn new(semitones: f32) -> Result<Self, VoiceError> {
        if !semitones.is_finite() || semitones.abs() > MAX_SEMITONES {
            return Err(VoiceError::InvalidSemitones(semitones));
        }
        let target = 2f32.powf(semitones / 12.0);
        let synthesis_hop = (ANALYSIS_HOP as f32 * target).round() as usize;
        let ratio = synthesis_hop as f32 / ANALYSIS_HOP as f32;
        let vocoder = PhaseVocoder::new(ANALYSIS_HOP, ratio);
        let gain = StreamingStft::<PhaseVocoder>::coherent_gain(FRAME, synthesis_hop);
        let stft = StreamingStft::new(FRAME, ANALYSIS_HOP, synthesis_hop, gain, vocoder)?;
        let buffered = FRAME + ANALYSIS_HOP;
        let lead = FRAME as f32 / 2.0 * (1.0 - 1.0 / ratio);
        Ok(Self {
            stft,
            resampler: StreamingResampler::new(ratio as f64),
            queue: LatencyQueue::new(buffered),
            latency: (buffered as f32 - lead).round() as usize,
        })
    }
}

impl VoiceEffect for PhaseVocoderPitchShift {
    fn process_block(&mut self, input: &[f32]) -> Vec<f32> {
        let mut stretched = Vec::with_capacity(input.len() * 2 + FRAME);
        self.stft.push(input, &mut stretched);
        let mut shifted = Vec::with_capacity(input.len() + FRAME);
        self.resampler.push(&stretched, &mut shifted);
        self.queue.exchange(&shifted, input.len())
    }

    fn latency(&self) -> usize {
        self.latency
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::{dominant_frequency, rms};
    use crate::effects::render;

    const RATE: u32 = 48000;

    fn sine(frequency: f32, seconds: f32) -> Vec<f32> {
        (0..(RATE as f32 * seconds) as usize)
            .map(|i| 0.5 * (TAU * frequency * i as f32 / RATE as f32).sin())
            .collect()
    }

    fn shifted(semitones: f32, input: &[f32]) -> Vec<f32> {
        let mut effect = PhaseVocoderPitchShift::new(semitones).unwrap();
        render(&mut effect, input)
    }

    fn middle(samples: &[f32]) -> &[f32] {
        &samples[samples.len() / 4..samples.len() * 3 / 4]
    }

    #[test]
    fn plus_twelve_semitones_moves_a440_up_one_octave_to_880() {
        let out = shifted(12.0, &sine(440.0, 1.0));
        let frequency = dominant_frequency(middle(&out), RATE);
        assert!((frequency - 880.0).abs() < 880.0 * 0.01, "got {frequency}");
    }

    #[test]
    fn minus_twelve_and_plus_seven_land_on_the_musical_intervals() {
        let input = sine(440.0, 1.0);
        let down = dominant_frequency(middle(&shifted(-12.0, &input)), RATE);
        let fifth = dominant_frequency(middle(&shifted(7.0, &input)), RATE);
        assert!((down - 220.0).abs() < 220.0 * 0.01, "got {down}");
        assert!((fifth - 659.26).abs() < 659.26 * 0.01, "got {fifth}");
    }

    #[test]
    fn zero_semitones_preserves_frequency_and_waveform() {
        let input = sine(440.0, 1.0);
        let out = shifted(0.0, &input);
        let frequency = dominant_frequency(middle(&out), RATE);
        assert!((frequency - 440.0).abs() < 440.0 * 0.005, "got {frequency}");
        let error: f32 = middle(&out)
            .iter()
            .zip(middle(&input))
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        assert!(error < 0.01, "max error {error}");
    }

    #[test]
    fn output_length_and_loudness_match_the_input() {
        let input = sine(300.0, 0.8);
        for semitones in [-12.0, -5.0, 0.0, 5.0, 12.0] {
            let out = shifted(semitones, &input);
            assert_eq!(out.len(), input.len());
            let gain = rms(middle(&out)) / rms(middle(&input));
            assert!((0.8..1.25).contains(&gain), "{semitones}: gain {gain}");
        }
    }

    #[test]
    fn streaming_small_blocks_equals_processing_the_whole_signal() {
        let input = sine(523.25, 0.6);
        let mut whole = PhaseVocoderPitchShift::new(5.0).unwrap();
        let reference = whole.process_block(&input);
        let mut streamed = PhaseVocoderPitchShift::new(5.0).unwrap();
        let mut out = Vec::new();
        for (i, block) in input.chunks(128).enumerate() {
            let block_out = streamed.process_block(block);
            assert_eq!(block_out.len(), block.len(), "block {i}");
            out.extend(block_out);
        }
        assert_eq!(out.len(), reference.len());
        let diff = out
            .iter()
            .zip(&reference)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        assert!(diff < 1e-4, "max diff {diff}");
    }

    #[test]
    fn rejects_shifts_beyond_one_octave() {
        assert!(matches!(
            PhaseVocoderPitchShift::new(13.0),
            Err(VoiceError::InvalidSemitones(_))
        ));
    }
}
