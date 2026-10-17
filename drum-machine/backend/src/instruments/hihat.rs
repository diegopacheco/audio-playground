use super::{Instrument, samples_for};
use crate::dsp::exp_decay;
use crate::dsp::filter::Biquad;
use crate::dsp::noise::Noise;

pub struct HiHat;

const CUTOFF_HZ: f32 = 7000.0;
const AMP_TAU: f32 = 0.018;
const LENGTH: f32 = 0.12;

impl Instrument for HiHat {
    fn name(&self) -> &'static str {
        "hihat"
    }

    fn render(&self, sample_rate: u32) -> Vec<f32> {
        let mut noise = Noise::new(0x5eed_0002);
        let mut first = Biquad::highpass(CUTOFF_HZ, 0.7, sample_rate);
        let mut second = Biquad::highpass(CUTOFF_HZ, 0.7, sample_rate);
        (0..samples_for(LENGTH, sample_rate))
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                let s = second.process(first.process(noise.next_sample()));
                (s * exp_decay(t, AMP_TAU) * 0.8).clamp(-1.0, 1.0)
            })
            .collect()
    }
}
