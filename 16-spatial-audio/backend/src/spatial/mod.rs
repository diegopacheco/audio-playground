pub mod binaural;
pub mod cues;
pub mod engine;
pub mod hrtf;

use serde::Deserialize;

use crate::audio::{Mono, Stereo};
use crate::error::AppError;
use crate::trajectory::Trajectory;

pub trait Spatializer {
    fn render(&self, mono: &Mono, trajectory: &Trajectory) -> Result<Stereo, AppError>;
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Binaural,
    Hrtf,
}

pub fn spatializer(method: Method) -> Box<dyn Spatializer> {
    match method {
        Method::Binaural => Box::new(binaural::SimpleBinaural),
        Method::Hrtf => Box::new(hrtf::ParametricHrtf),
    }
}

#[cfg(test)]
pub mod analysis {
    pub fn rms(x: &[f32]) -> f64 {
        (x.iter().map(|s| (*s as f64).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
    }

    pub fn right_lead_samples(left: &[f32], right: &[f32], max_lag: usize) -> f64 {
        let corr = |lag: i64| -> f64 {
            (max_lag..right.len() - max_lag)
                .map(|n| right[n] as f64 * left[(n as i64 + lag) as usize] as f64)
                .sum()
        };
        let lags: Vec<i64> = (-(max_lag as i64)..=max_lag as i64).collect();
        let values: Vec<f64> = lags.iter().map(|l| corr(*l)).collect();
        let best = (0..values.len())
            .max_by(|a, b| values[*a].total_cmp(&values[*b]))
            .unwrap_or(0);
        let mut offset = 0.0;
        if best > 0 && best + 1 < values.len() {
            let (a, b, c) = (values[best - 1], values[best], values[best + 1]);
            let denom = a - 2.0 * b + c;
            if denom != 0.0 {
                offset = 0.5 * (a - c) / denom;
            }
        }
        lags[best] as f64 + offset
    }

    pub fn crossing_rate(x: &[f32], fs: f64) -> f64 {
        let crossings = x
            .windows(2)
            .filter(|w| (w[0] <= 0.0 && w[1] > 0.0) || (w[0] >= 0.0 && w[1] < 0.0))
            .count();
        crossings as f64 * fs / (2.0 * x.len() as f64)
    }

    pub fn high_band_rms(x: &[f32]) -> f64 {
        let diff: Vec<f32> = x.windows(2).map(|w| w[1] - w[0]).collect();
        rms(&diff)
    }
}
