use crate::dsp::fft::Complex;
use crate::dsp::stft::{SpectralProcessor, StreamingStft};
use crate::dsp::window;
use crate::effects::VoiceEffect;
use crate::effects::queue::LatencyQueue;
use crate::error::VoiceError;
use std::f32::consts::TAU;

const FRAME: usize = 1024;
const HOP: usize = 256;
const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

struct RandomPhase {
    state: u64,
}

impl RandomPhase {
    fn next_phase(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state >> 40) as f32 / (1u64 << 24) as f32 * TAU
    }
}

impl SpectralProcessor for RandomPhase {
    fn process(&mut self, bins: &mut [Complex]) {
        for bin in bins.iter_mut() {
            *bin = Complex::from_polar(bin.norm(), self.next_phase());
        }
    }
}

pub struct Whisper {
    stft: StreamingStft<RandomPhase>,
    queue: LatencyQueue,
}

impl Whisper {
    pub fn new() -> Result<Self, VoiceError> {
        let energy = window::energy(&window::hann(FRAME));
        let gain = (HOP as f32 * FRAME as f32 / (energy * energy)).sqrt();
        let processor = RandomPhase { state: SEED };
        Ok(Self {
            stft: StreamingStft::new(FRAME, HOP, HOP, gain, processor)?,
            queue: LatencyQueue::new(FRAME),
        })
    }
}

impl VoiceEffect for Whisper {
    fn process_block(&mut self, input: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(input.len() + FRAME);
        self.stft.push(input, &mut out);
        self.queue.exchange(&out, input.len())
    }

    fn latency(&self) -> usize {
        FRAME
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::{periodicity, rms};
    use crate::effects::render;
    use crate::signal;

    #[test]
    fn whisper_removes_the_pitch_but_keeps_the_loudness() {
        let rate = 48000;
        let voice = signal::voice(rate, 2.0).unwrap();
        let out = render(&mut Whisper::new().unwrap(), &voice);
        let segment = 24000..36000;
        let (_, voiced) = periodicity(&voice[segment.clone()], rate, 70.0, 400.0);
        let (_, whispered) = periodicity(&out[segment.clone()], rate, 70.0, 400.0);
        assert!(voiced > 0.8, "input clarity {voiced}");
        assert!(whispered < 0.5, "whisper clarity {whispered}");
        let gain = rms(&out[segment.clone()]) / rms(&voice[segment]);
        assert!((0.5..2.0).contains(&gain), "gain {gain}");
    }
}
