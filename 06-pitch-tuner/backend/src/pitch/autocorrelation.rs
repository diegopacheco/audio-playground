use super::{Pitch, PitchDetector, lag_range, parabolic_offset};

pub struct Autocorrelation {
    pub peak_ratio: f64,
    pub min_clarity: f64,
}

impl Default for Autocorrelation {
    fn default() -> Self {
        Autocorrelation {
            peak_ratio: 0.9,
            min_clarity: 0.5,
        }
    }
}

impl Autocorrelation {
    fn normalized(samples: &[f32], max_lag: usize) -> Vec<f64> {
        let window = samples.len() - max_lag;
        (0..=max_lag)
            .map(|tau| {
                let mut cross = 0.0;
                let mut energy_a = 0.0;
                let mut energy_b = 0.0;
                for j in 0..window {
                    let a = f64::from(samples[j]);
                    let b = f64::from(samples[j + tau]);
                    cross += a * b;
                    energy_a += a * a;
                    energy_b += b * b;
                }
                let energy = (energy_a * energy_b).sqrt();
                if energy > 0.0 { cross / energy } else { 0.0 }
            })
            .collect()
    }

    fn local_peaks(acf: &[f64], min_lag: usize) -> Vec<usize> {
        let start = acf
            .iter()
            .position(|v| *v < 0.0)
            .unwrap_or(acf.len())
            .max(min_lag);
        (start.max(1)..acf.len().saturating_sub(1))
            .filter(|&t| acf[t] > acf[t - 1] && acf[t] >= acf[t + 1])
            .collect()
    }
}

impl PitchDetector for Autocorrelation {
    fn name(&self) -> &'static str {
        "autocorrelation"
    }

    fn estimate(&self, samples: &[f32], sample_rate: u32) -> Option<Pitch> {
        let (min_lag, max_lag) = lag_range(samples.len(), sample_rate)?;
        let acf = Self::normalized(samples, max_lag);
        let peaks = Self::local_peaks(&acf, min_lag);
        let best = peaks.iter().map(|&t| acf[t]).fold(f64::MIN, f64::max);
        if best < self.min_clarity {
            return None;
        }
        let tau = *peaks.iter().find(|&&t| acf[t] >= best * self.peak_ratio)?;
        let offset = parabolic_offset(acf[tau - 1], acf[tau], acf[tau + 1]);
        Some(Pitch {
            frequency: f64::from(sample_rate) / (tau as f64 + offset),
            clarity: acf[tau].clamp(0.0, 1.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::cents_between;
    use crate::pitch::signals::{RATE, noise, saw, sine};

    fn cents_error(signal: &[f32], expected: f64) -> f64 {
        let pitch = Autocorrelation::default()
            .detect(signal, RATE)
            .unwrap()
            .expect("pitch expected");
        cents_between(pitch.frequency, expected)
    }

    #[test]
    fn sines_are_detected_within_a_few_cents() {
        for freq in [82.41, 110.0, 440.0, 1000.0] {
            let error = cents_error(&sine(freq, 0.5), freq);
            assert!(error.abs() < 5.0, "{freq} Hz off by {error} cents");
        }
    }

    #[test]
    fn saw_wave_picks_the_first_strong_peak_not_a_subharmonic() {
        for freq in [110.0, 440.0] {
            let error = cents_error(&saw(freq, 0.5), freq);
            assert!(error.abs() < 10.0, "saw {freq} Hz off by {error} cents");
        }
    }

    #[test]
    fn silence_has_no_pitch() {
        let silence = vec![0.0f32; 2048];
        assert_eq!(Autocorrelation::default().detect(&silence, RATE), Ok(None));
    }

    #[test]
    fn white_noise_is_not_pitched() {
        assert_eq!(
            Autocorrelation::default().detect(&noise(0.5), RATE),
            Ok(None)
        );
    }
}
