pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn signed(&mut self) -> f32 {
        self.unit() * 2.0 - 1.0
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n.max(1) as u64) as usize
    }
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|v| v * v).sum::<f32>() / samples.len() as f32).sqrt()
}

pub fn add_white_noise(samples: &[f32], ratio: f32, seed: u64) -> Vec<f32> {
    let amplitude = rms(samples) * ratio * 3f32.sqrt();
    let mut rng = Rng::new(seed);
    samples
        .iter()
        .map(|s| (s + rng.signed() * amplitude).clamp(-1.0, 1.0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_ratio_sets_noise_rms_relative_to_signal_rms() {
        let signal: Vec<f32> = (0..40000).map(|i| (i as f32 * 0.07).sin() * 0.3).collect();
        let noisy = add_white_noise(&signal, 0.5, 7);
        let diff: Vec<f32> = noisy.iter().zip(&signal).map(|(a, b)| a - b).collect();
        let ratio = rms(&diff) / rms(&signal);
        assert!((ratio - 0.5).abs() < 0.03, "ratio {ratio}");
    }

    #[test]
    fn same_seed_gives_same_noise() {
        let signal = vec![0.1f32; 100];
        assert_eq!(
            add_white_noise(&signal, 1.0, 3),
            add_white_noise(&signal, 1.0, 3)
        );
    }
}
