use super::delay_line::DelayLine;
use super::{Effect, decay_samples, ms_to_samples};

const COMB_MS: [f32; 4] = [29.7, 37.1, 41.1, 43.7];
const ALLPASS_MS: [f32; 2] = [5.0, 1.7];
const ALLPASS_GAIN: f32 = 0.7;

struct Comb {
    line: DelayLine,
    feedback: f32,
}

impl Comb {
    fn process(&mut self, x: f32) -> f32 {
        let y = self.line.read();
        self.line.write(x + y * self.feedback);
        y
    }
}

struct Allpass {
    line: DelayLine,
    gain: f32,
}

impl Allpass {
    fn process(&mut self, x: f32) -> f32 {
        let delayed = self.line.read();
        let w = x + self.gain * delayed;
        self.line.write(w);
        delayed - self.gain * w
    }
}

pub struct Reverb {
    combs: Vec<Comb>,
    allpasses: Vec<Allpass>,
    mix: f32,
    tail_cap: usize,
}

impl Reverb {
    pub fn new(sample_rate: u32, room_size: f32, mix: f32, tail_cap: usize) -> Self {
        let feedback = 0.7 + 0.28 * room_size;
        Reverb {
            combs: COMB_MS
                .iter()
                .map(|ms| Comb {
                    line: DelayLine::new(ms_to_samples(*ms, sample_rate)),
                    feedback,
                })
                .collect(),
            allpasses: ALLPASS_MS
                .iter()
                .map(|ms| Allpass {
                    line: DelayLine::new(ms_to_samples(*ms, sample_rate)),
                    gain: ALLPASS_GAIN,
                })
                .collect(),
            mix,
            tail_cap,
        }
    }
}

impl Effect for Reverb {
    fn process(&mut self, sample: f32) -> f32 {
        let parallel: f32 = self
            .combs
            .iter_mut()
            .map(|c| c.process(sample))
            .sum::<f32>()
            / self.combs.len() as f32;
        let wet = self
            .allpasses
            .iter_mut()
            .fold(parallel, |acc, a| a.process(acc));
        sample * (1.0 - self.mix) + wet * self.mix
    }

    fn tail_samples(&self) -> usize {
        self.combs
            .iter()
            .map(|c| decay_samples(c.line.len(), c.feedback, self.tail_cap))
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::render;

    fn last_audible(signal: &[f32]) -> usize {
        signal.iter().rposition(|s| s.abs() > 1e-3).unwrap_or(0)
    }

    #[test]
    fn reverb_tail_rings_far_beyond_the_end_of_the_input() {
        let rate = 44100;
        let input = vec![0.8; 100];
        let mut reverb = Reverb::new(rate, 0.8, 1.0, rate as usize * 10);
        let out = render(&mut reverb, &input, rate as usize * 2);
        assert!(last_audible(&out) > input.len() + rate as usize / 2);
    }

    #[test]
    fn larger_room_rings_longer_than_smaller_room() {
        let rate = 44100;
        let input = vec![0.8; 100];
        let mut small = Reverb::new(rate, 0.1, 1.0, rate as usize * 10);
        let mut large = Reverb::new(rate, 0.9, 1.0, rate as usize * 10);
        let extra = rate as usize * 4;
        let small_end = last_audible(&render(&mut small, &input, extra));
        let large_end = last_audible(&render(&mut large, &input, extra));
        assert!(large_end > small_end);
        assert!(large.tail_samples() > small.tail_samples());
    }

    #[test]
    fn dry_mix_passes_signal_unchanged() {
        let mut reverb = Reverb::new(8000, 0.5, 0.0, 80000);
        let input = [0.1, -0.2, 0.3];
        assert_eq!(render(&mut reverb, &input, 0), input.to_vec());
    }
}
