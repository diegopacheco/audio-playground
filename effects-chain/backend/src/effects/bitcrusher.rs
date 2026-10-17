use super::Effect;

pub struct Bitcrusher {
    steps: f32,
    hold_every: usize,
    counter: usize,
    held: f32,
}

impl Bitcrusher {
    pub fn new(bits: u32, hold_every: usize) -> Self {
        Bitcrusher {
            steps: 2f32.powi(bits.saturating_sub(1) as i32),
            hold_every: hold_every.max(1),
            counter: 0,
            held: 0.0,
        }
    }

    fn quantize(&self, x: f32) -> f32 {
        ((x.clamp(-1.0, 1.0) * self.steps).round() / self.steps).clamp(-1.0, 1.0)
    }
}

impl Effect for Bitcrusher {
    fn process(&mut self, sample: f32) -> f32 {
        if self.counter == 0 {
            self.held = self.quantize(sample);
        }
        self.counter = (self.counter + 1) % self.hold_every;
        self.held
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn sine(n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (i as f32 * 2.0 * std::f32::consts::PI / 97.0).sin() * 0.9)
            .collect()
    }

    fn distinct(v: &[f32]) -> usize {
        v.iter()
            .map(|x| (x + 0.0).to_bits())
            .collect::<HashSet<_>>()
            .len()
    }

    #[test]
    fn low_bit_depth_collapses_a_smooth_sine_into_few_levels() {
        let input = sine(2000);
        let mut crusher = Bitcrusher::new(3, 1);
        let out: Vec<f32> = input.iter().map(|s| crusher.process(*s)).collect();
        assert!(distinct(&input) > 50);
        assert!(distinct(&out) <= 9);
    }

    #[test]
    fn fewer_bits_means_fewer_distinct_levels() {
        let input = sine(2000);
        let mut eight = Bitcrusher::new(8, 1);
        let mut two = Bitcrusher::new(2, 1);
        let a: Vec<f32> = input.iter().map(|s| eight.process(*s)).collect();
        let b: Vec<f32> = input.iter().map(|s| two.process(*s)).collect();
        assert!(distinct(&b) < distinct(&a));
    }

    #[test]
    fn sample_rate_reduction_holds_each_value_for_n_samples() {
        let mut crusher = Bitcrusher::new(16, 4);
        let out: Vec<f32> = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6]
            .iter()
            .map(|s| crusher.process(*s))
            .collect();
        assert_eq!(out[0], out[3]);
        assert_ne!(out[3], out[4]);
    }
}
