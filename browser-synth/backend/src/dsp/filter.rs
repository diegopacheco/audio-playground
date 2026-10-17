use super::DspError;

pub trait Filter {
    fn process(&mut self, input: f32) -> f32;
}

pub struct BiquadLowpass {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl BiquadLowpass {
    pub fn new(cutoff: f32, resonance_db: f32, sample_rate: u32) -> Result<Self, DspError> {
        let nyquist = sample_rate as f32 / 2.0;
        if cutoff <= 0.0 || cutoff >= nyquist {
            return Err(DspError::CutoffOutOfRange(cutoff));
        }
        let w0 = std::f64::consts::TAU * f64::from(cutoff) / f64::from(sample_rate);
        let alpha = w0.sin() / (2.0 * 10f64.powf(f64::from(resonance_db) / 20.0));
        let cos = w0.cos();
        let a0 = 1.0 + alpha;
        Ok(Self {
            b0: (1.0 - cos) / 2.0 / a0,
            b1: (1.0 - cos) / a0,
            b2: (1.0 - cos) / 2.0 / a0,
            a1: -2.0 * cos / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        })
    }
}

impl Filter for BiquadLowpass {
    fn process(&mut self, input: f32) -> f32 {
        let x = f64::from(input);
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms_through_filter(frequency: f32, cutoff: f32) -> f32 {
        let mut filter = BiquadLowpass::new(cutoff, 0.0, 44_100).unwrap_or_else(|e| panic!("{e}"));
        let out: Vec<f32> = (0..44_100)
            .map(|i| {
                let t = i as f32 / 44_100.0;
                filter.process((std::f32::consts::TAU * frequency * t).sin())
            })
            .skip(4_410)
            .collect();
        (out.iter().map(|s| s * s).sum::<f32>() / out.len() as f32).sqrt()
    }

    #[test]
    fn tone_well_above_cutoff_is_strongly_attenuated() {
        let low = rms_through_filter(200.0, 1_000.0);
        let high = rms_through_filter(8_000.0, 1_000.0);
        assert!(high < low * 0.05, "low {low} high {high}");
    }

    #[test]
    fn tone_below_cutoff_passes_almost_unchanged() {
        let unfiltered = std::f32::consts::FRAC_1_SQRT_2;
        let passed = rms_through_filter(100.0, 2_000.0);
        assert!((passed - unfiltered).abs() < 0.02, "passed {passed}");
    }

    #[test]
    fn resonance_boosts_frequencies_near_cutoff() {
        let mut flat = BiquadLowpass::new(1_000.0, 0.0, 44_100).unwrap_or_else(|e| panic!("{e}"));
        let mut peaky = BiquadLowpass::new(1_000.0, 12.0, 44_100).unwrap_or_else(|e| panic!("{e}"));
        let energy = |f: &mut BiquadLowpass| -> f32 {
            (0..44_100)
                .map(|i| f.process((std::f32::consts::TAU * 1_000.0 * i as f32 / 44_100.0).sin()))
                .skip(4_410)
                .map(|s| s * s)
                .sum()
        };
        assert!(energy(&mut peaky) > energy(&mut flat) * 4.0);
    }

    #[test]
    fn cutoff_at_or_above_nyquist_is_rejected() {
        assert!(BiquadLowpass::new(22_050.0, 0.0, 44_100).is_err());
        assert!(BiquadLowpass::new(0.0, 0.0, 44_100).is_err());
    }
}
