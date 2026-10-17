use super::{Instrument, samples_for};
use crate::dsp::exp_decay;
use crate::dsp::filter::Biquad;
use crate::dsp::noise::Noise;

pub struct Clap;

const BAND_HZ: f32 = 1200.0;
const BURST_GAP: f32 = 0.011;
const BURSTS: usize = 3;
const BURST_TAU: f32 = 0.004;
const TAIL_TAU: f32 = 0.06;
const LENGTH: f32 = 0.35;

fn envelope(t: f32) -> f32 {
    let tail_start = BURST_GAP * BURSTS as f32;
    if t < tail_start {
        exp_decay(t % BURST_GAP, BURST_TAU)
    } else {
        exp_decay(t - tail_start, TAIL_TAU)
    }
}

impl Instrument for Clap {
    fn name(&self) -> &'static str {
        "clap"
    }

    fn render(&self, sample_rate: u32) -> Vec<f32> {
        let mut noise = Noise::new(0x5eed_0003);
        let mut band = Biquad::bandpass(BAND_HZ, 1.2, sample_rate);
        (0..samples_for(LENGTH, sample_rate))
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (band.process(noise.next_sample()) * envelope(t) * 1.6).clamp(-1.0, 1.0)
            })
            .collect()
    }
}
