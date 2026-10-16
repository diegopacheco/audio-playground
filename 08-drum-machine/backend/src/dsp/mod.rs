pub mod filter;
pub mod noise;

#[cfg(test)]
pub mod analysis;

pub fn exp_decay(t: f32, tau: f32) -> f32 {
    (-t / tau).exp()
}
