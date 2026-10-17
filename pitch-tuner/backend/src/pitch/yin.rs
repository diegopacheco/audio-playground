use super::{Pitch, PitchDetector, lag_range, parabolic_offset};

pub struct Yin {
    pub threshold: f64,
}

impl Default for Yin {
    fn default() -> Self {
        Yin { threshold: 0.15 }
    }
}

impl Yin {
    fn difference(samples: &[f32], max_lag: usize) -> Vec<f64> {
        let window = samples.len() - max_lag;
        (0..=max_lag)
            .map(|tau| {
                (0..window)
                    .map(|j| {
                        let delta = f64::from(samples[j]) - f64::from(samples[j + tau]);
                        delta * delta
                    })
                    .sum()
            })
            .collect()
    }

    fn cumulative_mean_normalized(diff: &[f64]) -> Vec<f64> {
        let mut cmnd = vec![1.0; diff.len()];
        let mut running = 0.0;
        for tau in 1..diff.len() {
            running += diff[tau];
            cmnd[tau] = if running > 0.0 {
                diff[tau] * tau as f64 / running
            } else {
                1.0
            };
        }
        cmnd
    }

    fn absolute_threshold(&self, cmnd: &[f64], min_lag: usize) -> Option<usize> {
        let last = cmnd.len() - 1;
        let mut tau = min_lag;
        while tau < last {
            if cmnd[tau] < self.threshold {
                while tau + 1 < last && cmnd[tau + 1] < cmnd[tau] {
                    tau += 1;
                }
                return Some(tau);
            }
            tau += 1;
        }
        None
    }
}

impl PitchDetector for Yin {
    fn name(&self) -> &'static str {
        "yin"
    }

    fn estimate(&self, samples: &[f32], sample_rate: u32) -> Option<Pitch> {
        let (min_lag, max_lag) = lag_range(samples.len(), sample_rate)?;
        let diff = Self::difference(samples, max_lag);
        let cmnd = Self::cumulative_mean_normalized(&diff);
        let tau = self.absolute_threshold(&cmnd, min_lag)?;
        let offset = parabolic_offset(diff[tau - 1], diff[tau], diff[tau + 1]);
        Some(Pitch {
            frequency: f64::from(sample_rate) / (tau as f64 + offset),
            clarity: (1.0 - cmnd[tau]).clamp(0.0, 1.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::cents_between;
    use crate::pitch::signals::{RATE, noise, saw, sine};

    fn detect_cents_error(signal: &[f32], expected: f64) -> f64 {
        let pitch = Yin::default()
            .detect(signal, RATE)
            .unwrap()
            .expect("pitch expected");
        cents_between(pitch.frequency, expected)
    }

    #[test]
    fn sines_across_the_tuning_range_land_within_one_cent() {
        for freq in [82.41, 110.0, 440.0, 1000.0] {
            let error = detect_cents_error(&sine(freq, 0.5), freq);
            assert!(error.abs() < 1.0, "{freq} Hz off by {error} cents");
        }
    }

    #[test]
    fn clean_sine_reports_high_clarity() {
        let pitch = Yin::default().detect(&sine(220.0, 0.5), RATE).unwrap();
        assert!(pitch.unwrap().clarity > 0.95);
    }

    #[test]
    fn saw_wave_full_of_harmonics_still_reports_its_fundamental() {
        for freq in [82.41, 110.0, 440.0] {
            let error = detect_cents_error(&saw(freq, 0.5), freq);
            assert!(error.abs() < 5.0, "saw {freq} Hz off by {error} cents");
        }
    }

    #[test]
    fn silence_has_no_pitch() {
        let silence = vec![0.0f32; 2048];
        assert_eq!(Yin::default().detect(&silence, RATE), Ok(None));
    }

    #[test]
    fn white_noise_is_not_pitched() {
        assert_eq!(Yin::default().detect(&noise(0.5), RATE), Ok(None));
    }

    #[test]
    fn cmnd_starts_at_one_so_lag_zero_is_never_picked() {
        let diff = Yin::difference(&sine(440.0, 0.5), 500);
        let cmnd = Yin::cumulative_mean_normalized(&diff);
        assert_eq!(cmnd[0], 1.0);
        let period = (f64::from(RATE) / 440.0).round() as usize;
        assert!(cmnd[period] < 0.05);
    }
}
