use super::{Instrument, samples_for};
use crate::dsp::exp_decay;
use std::f32::consts::PI;

pub struct Kick;

const START_HZ: f32 = 150.0;
const END_HZ: f32 = 45.0;
const PITCH_TAU: f32 = 0.03;
const AMP_TAU: f32 = 0.12;
const LENGTH: f32 = 0.6;

impl Instrument for Kick {
    fn name(&self) -> &'static str {
        "kick"
    }

    fn render(&self, sample_rate: u32) -> Vec<f32> {
        let dt = 1.0 / sample_rate as f32;
        let mut phase = 0.0f32;
        (0..samples_for(LENGTH, sample_rate))
            .map(|i| {
                let t = i as f32 * dt;
                let freq = END_HZ + (START_HZ - END_HZ) * exp_decay(t, PITCH_TAU);
                phase += 2.0 * PI * freq * dt;
                phase.sin() * exp_decay(t, AMP_TAU) * 0.95
            })
            .collect()
    }
}
