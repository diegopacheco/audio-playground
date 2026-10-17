use std::f32::consts::PI;

pub struct WhiteNoise {
    state: u64,
}

impl WhiteNoise {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn uniform(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        ((self.state >> 40) as f32 + 1.0) / ((1u64 << 24) as f32 + 1.0)
    }

    pub fn gaussian(&mut self) -> f32 {
        let u1 = self.uniform();
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
    }
}

pub fn add_white_noise(samples: &[f32], snr_db: f32, seed: u64) -> Vec<f32> {
    let active: Vec<f32> = samples.iter().copied().filter(|s| *s != 0.0).collect();
    let signal_power = active.iter().map(|s| s * s).sum::<f32>() / active.len().max(1) as f32;
    let noise_sigma = (signal_power / 10f32.powf(snr_db / 10.0)).sqrt();
    let mut noise = WhiteNoise::new(seed);
    samples
        .iter()
        .map(|s| s + noise.gaussian() * noise_sigma)
        .collect()
}
