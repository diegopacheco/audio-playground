use super::delay_line::DelayLine;
use super::{Effect, decay_samples};

pub struct Echo {
    line: DelayLine,
    feedback: f32,
    mix: f32,
    tail_cap: usize,
}

impl Echo {
    pub fn new(delay_samples: usize, feedback: f32, mix: f32, tail_cap: usize) -> Self {
        Echo {
            line: DelayLine::new(delay_samples),
            feedback,
            mix,
            tail_cap,
        }
    }
}

impl Effect for Echo {
    fn process(&mut self, sample: f32) -> f32 {
        let delayed = self.line.read();
        self.line.write(sample + delayed * self.feedback);
        sample * (1.0 - self.mix) + delayed * self.mix
    }

    fn tail_samples(&self) -> usize {
        decay_samples(self.line.len(), self.feedback, self.tail_cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::{impulse, render};

    #[test]
    fn feedback_produces_repeats_that_decay_geometrically() {
        let mut echo = Echo::new(5, 0.5, 1.0, 1000);
        let out = render(&mut echo, &impulse(30), 0);
        assert_eq!(out[5], 1.0);
        assert_eq!(out[10], 0.5);
        assert_eq!(out[15], 0.25);
        assert_eq!(out[20], 0.125);
    }

    #[test]
    fn zero_feedback_gives_a_single_repeat() {
        let mut echo = Echo::new(5, 0.0, 1.0, 1000);
        let out = render(&mut echo, &impulse(30), 0);
        assert_eq!(out[5], 1.0);
        assert_eq!(out[10], 0.0);
    }
}
