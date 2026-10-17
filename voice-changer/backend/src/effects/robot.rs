use crate::dsp::fft::Complex;
use crate::dsp::stft::{SpectralProcessor, StreamingStft};
use crate::dsp::window;
use crate::effects::VoiceEffect;
use crate::effects::queue::LatencyQueue;
use crate::error::VoiceError;

const BASE_HZ: f32 = 110.0;
const MIN_FRAME: usize = 1024;
const MAX_SEMITONES: f32 = 12.0;
const MIN_RATE: u32 = 8000;

struct ZeroPhase;

impl SpectralProcessor for ZeroPhase {
    fn process(&mut self, bins: &mut [Complex]) {
        for (k, bin) in bins.iter_mut().enumerate() {
            let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
            *bin = Complex::new(bin.norm() * sign, 0.0);
        }
    }
}

pub struct Robot {
    stft: StreamingStft<ZeroPhase>,
    queue: LatencyQueue,
    latency: usize,
}

impl Robot {
    pub fn new(semitones: f32, rate: u32) -> Result<Self, VoiceError> {
        if !semitones.is_finite() || semitones.abs() > MAX_SEMITONES {
            return Err(VoiceError::InvalidSemitones(semitones));
        }
        if rate < MIN_RATE {
            return Err(VoiceError::InvalidSampleRate(rate));
        }
        let pitch = BASE_HZ * 2f32.powf(semitones / 12.0);
        let hop = (rate as f32 / pitch).round() as usize;
        let size = (2 * hop).next_power_of_two().max(MIN_FRAME);
        let gain = (hop as f32 / window::energy(&window::hann(size))).sqrt();
        Ok(Self {
            stft: StreamingStft::new(size, hop, hop, gain, ZeroPhase)?,
            queue: LatencyQueue::new(size),
            latency: size,
        })
    }
}

impl VoiceEffect for Robot {
    fn process_block(&mut self, input: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(input.len() + MIN_FRAME);
        self.stft.push(input, &mut out);
        self.queue.exchange(&out, input.len())
    }

    fn latency(&self) -> usize {
        self.latency
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::{periodicity, rms};
    use crate::effects::render;
    use crate::signal;

    const RATE: u32 = 48000;

    fn segment_pitches(samples: &[f32]) -> Vec<f32> {
        samples
            .chunks(RATE as usize / 2)
            .skip(1)
            .take(4)
            .map(|chunk| periodicity(chunk, RATE, 70.0, 400.0).0)
            .collect()
    }

    #[test]
    fn robot_flattens_a_gliding_voice_to_one_fixed_pitch() {
        let voice = signal::voice(RATE, 3.0).unwrap();
        let input_pitches = segment_pitches(&voice);
        let lowest = input_pitches.iter().cloned().fold(f32::MAX, f32::min);
        let highest = input_pitches.iter().cloned().fold(0.0, f32::max);
        assert!(
            highest / lowest > 1.15,
            "input should glide: {input_pitches:?}"
        );

        let out = render(&mut Robot::new(0.0, RATE).unwrap(), &voice);
        for pitch in segment_pitches(&out) {
            assert!((pitch - 110.0).abs() < 2.0, "robot pitch {pitch}");
        }
        let gain = rms(&out) / rms(&voice);
        assert!((0.3..3.0).contains(&gain), "gain {gain}");
    }

    #[test]
    fn semitones_move_the_robot_pitch_an_octave_up() {
        let voice = signal::voice(RATE, 2.0).unwrap();
        let out = render(&mut Robot::new(12.0, RATE).unwrap(), &voice);
        let (pitch, _) = periodicity(&out[RATE as usize / 2..RATE as usize], RATE, 70.0, 400.0);
        assert!((pitch - 220.0).abs() < 4.0, "robot pitch {pitch}");
    }
}
