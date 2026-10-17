use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

pub trait Oscillator {
    fn next_sample(&mut self) -> f32;
}

pub struct WaveOscillator {
    waveform: Waveform,
    phase: f32,
    step: f32,
}

impl WaveOscillator {
    pub fn new(waveform: Waveform, frequency: f32, sample_rate: u32) -> Self {
        Self {
            waveform,
            phase: 0.0,
            step: frequency / sample_rate as f32,
        }
    }
}

impl Oscillator for WaveOscillator {
    fn next_sample(&mut self) -> f32 {
        let p = self.phase;
        let value = match self.waveform {
            Waveform::Sine => (std::f32::consts::TAU * p).sin(),
            Waveform::Square => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Sawtooth => 2.0 * p - 1.0,
            Waveform::Triangle => 1.0 - 4.0 * (p - 0.5).abs(),
        };
        self.phase = (self.phase + self.step).fract();
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_crossings(waveform: Waveform, frequency: f32) -> usize {
        let mut osc = WaveOscillator::new(waveform, frequency, 44_100);
        let samples: Vec<f32> = (0..44_100).map(|_| osc.next_sample()).collect();
        samples
            .windows(2)
            .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
            .count()
    }

    #[test]
    fn sine_at_440_crosses_zero_about_880_times_per_second_so_pitch_is_right() {
        let crossings = zero_crossings(Waveform::Sine, 440.0);
        assert!((875..=885).contains(&crossings), "got {crossings}");
    }

    #[test]
    fn every_waveform_keeps_the_same_pitch() {
        for w in [Waveform::Square, Waveform::Sawtooth, Waveform::Triangle] {
            let crossings = zero_crossings(w, 220.0);
            assert!((435..=445).contains(&crossings), "{w:?} got {crossings}");
        }
    }

    #[test]
    fn waveforms_stay_within_unit_range_to_avoid_clipping() {
        for w in [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Sawtooth,
            Waveform::Triangle,
        ] {
            let mut osc = WaveOscillator::new(w, 333.0, 44_100);
            assert!((0..10_000).all(|_| osc.next_sample().abs() <= 1.0));
        }
    }
}
