pub struct StreamingResampler {
    step: f64,
    position: f64,
    pending: Vec<f32>,
}

impl StreamingResampler {
    pub fn new(step: f64) -> Self {
        Self {
            step,
            position: 0.0,
            pending: Vec::new(),
        }
    }

    pub fn push(&mut self, samples: &[f32], out: &mut Vec<f32>) {
        self.pending.extend_from_slice(samples);
        let last = self.pending.len() as f64 - 1.0;
        while self.position < last {
            let index = self.position as usize;
            let frac = (self.position - index as f64) as f32;
            let a = self.pending[index];
            let b = self.pending[index + 1];
            out.push(a + (b - a) * frac);
            self.position += self.step;
        }
        let consumed = (self.position as usize).min(self.pending.len());
        self.pending.drain(..consumed);
        self.position -= consumed as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(step: f64, input: &[f32], block: usize) -> Vec<f32> {
        let mut resampler = StreamingResampler::new(step);
        let mut out = Vec::new();
        for chunk in input.chunks(block) {
            resampler.push(chunk, &mut out);
        }
        out
    }

    #[test]
    fn step_two_keeps_every_other_sample_which_doubles_pitch_after_a_2x_stretch() {
        let input: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        let out = run(2.0, &input, 37);
        assert!((out.len() as i32 - 500).abs() <= 1);
        for (i, value) in out.iter().enumerate() {
            assert_eq!(*value, (2 * i) as f32);
        }
    }

    #[test]
    fn half_step_interpolates_between_neighbours() {
        let out = run(0.5, &[0.0, 1.0, 2.0], 1);
        assert_eq!(out, vec![0.0, 0.5, 1.0, 1.5]);
    }
}
