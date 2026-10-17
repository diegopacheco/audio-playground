pub mod bitcrusher;
pub mod chain;
pub mod delay;
pub mod delay_line;
pub mod distortion;
pub mod echo;
pub mod registry;
pub mod reverb;

pub trait Effect: Send {
    fn process(&mut self, sample: f32) -> f32;

    fn tail_samples(&self) -> usize {
        0
    }
}

pub fn ms_to_samples(ms: f32, sample_rate: u32) -> usize {
    ((ms * sample_rate as f32 / 1000.0).round() as usize).max(1)
}

pub fn decay_samples(delay: usize, feedback: f32, cap: usize) -> usize {
    if feedback <= 0.0 {
        return delay.min(cap);
    }
    let repeats = (0.001f32.ln() / feedback.min(0.999).ln()).ceil() as usize;
    delay.saturating_mul(repeats.max(1)).min(cap)
}

#[cfg(test)]
pub fn render(effect: &mut dyn Effect, input: &[f32], extra: usize) -> Vec<f32> {
    input
        .iter()
        .copied()
        .chain(std::iter::repeat_n(0.0, extra))
        .map(|s| effect.process(s))
        .collect()
}

#[cfg(test)]
pub fn impulse(len: usize) -> Vec<f32> {
    let mut v = vec![0.0; len];
    v[0] = 1.0;
    v
}
