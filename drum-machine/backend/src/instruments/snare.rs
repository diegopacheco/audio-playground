use super::{Instrument, samples_for};
use crate::dsp::exp_decay;
use crate::dsp::filter::Biquad;
use crate::dsp::noise::Noise;
use std::f32::consts::PI;

pub struct Snare;

const TONE_HZ: f32 = 185.0;
const BAND_HZ: f32 = 1800.0;
const NOISE_TAU: f32 = 0.05;
const TONE_TAU: f32 = 0.04;
const LENGTH: f32 = 0.3;

impl Instrument for Snare {
    fn name(&self) -> &'static str {
        "snare"
    }

    fn render(&self, sample_rate: u32) -> Vec<f32> {
        let mut noise = Noise::new(0x5eed_0001);
        let mut band = Biquad::bandpass(BAND_HZ, 0.8, sample_rate);
        (0..samples_for(LENGTH, sample_rate))
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                let rattle = band.process(noise.next_sample()) * exp_decay(t, NOISE_TAU);
                let body = (2.0 * PI * TONE_HZ * t).sin() * exp_decay(t, TONE_TAU);
                (rattle * 0.9 + body * 0.35).clamp(-1.0, 1.0)
            })
            .collect()
    }
}
