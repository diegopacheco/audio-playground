use super::Effect;
use super::delay_line::DelayLine;

pub struct Delay {
    line: DelayLine,
    mix: f32,
}

impl Delay {
    pub fn new(delay_samples: usize, mix: f32) -> Self {
        Delay {
            line: DelayLine::new(delay_samples),
            mix,
        }
    }
}

impl Effect for Delay {
    fn process(&mut self, sample: f32) -> f32 {
        let delayed = self.line.read();
        self.line.write(sample);
        sample * (1.0 - self.mix) + delayed * self.mix
    }

    fn tail_samples(&self) -> usize {
        self.line.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::{impulse, render};

    #[test]
    fn fully_wet_delay_moves_an_impulse_exactly_n_samples_later() {
        let mut delay = Delay::new(10, 1.0);
        let out = render(&mut delay, &impulse(32), 0);
        assert_eq!(out[10], 1.0);
        let others: f32 = out
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 10)
            .map(|(_, v)| v.abs())
            .sum();
        assert_eq!(others, 0.0);
    }

    #[test]
    fn half_mix_keeps_dry_signal_and_adds_one_copy() {
        let mut delay = Delay::new(4, 0.5);
        let out = render(&mut delay, &impulse(12), 0);
        assert_eq!(out[0], 0.5);
        assert_eq!(out[4], 0.5);
        assert_eq!(out[8], 0.0);
    }
}
