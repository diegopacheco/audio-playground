pub struct FirstOrderShelf {
    k: f64,
    b0: f64,
    b1: f64,
    a1: f64,
    x1: f64,
    y1: f64,
}

impl FirstOrderShelf {
    pub fn new(fs: f64) -> Self {
        FirstOrderShelf {
            k: 2.0 * fs,
            b0: 1.0,
            b1: 0.0,
            a1: 0.0,
            x1: 0.0,
            y1: 0.0,
        }
    }

    pub fn set(&mut self, high_gain: f64, corner: f64) {
        let beta = corner.min(0.9 * self.k);
        let norm = self.k + beta;
        self.b0 = (high_gain * self.k + beta) / norm;
        self.b1 = (beta - high_gain * self.k) / norm;
        self.a1 = (beta - self.k) / norm;
    }

    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 - self.a1 * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

pub fn magnitude(high_gain: f64, corner: f64, omega: f64) -> f64 {
    ((high_gain * omega).powi(2) + corner.powi(2)).sqrt() / (omega.powi(2) + corner.powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;

    fn sine_gain(high_gain: f64, corner: f64, freq: f64) -> f64 {
        let fs = 44_100.0;
        let mut shelf = FirstOrderShelf::new(fs);
        shelf.set(high_gain, corner);
        let n = 44_100;
        let mut peak: f64 = 0.0;
        for i in 0..n {
            let y = shelf.process((TAU * freq * i as f64 / fs).sin());
            if i > n / 2 {
                peak = peak.max(y.abs());
            }
        }
        peak
    }

    #[test]
    fn shelf_keeps_lows_and_scales_highs_like_a_head_shadow() {
        let corner = 2.0 * 343.0 / 0.0875;
        assert!((sine_gain(0.3, corner, 60.0) - 1.0).abs() < 0.05);
        assert!((sine_gain(0.3, corner, 12_000.0) - 0.3).abs() < 0.05);
        assert!((sine_gain(2.0, corner, 12_000.0) - 2.0).abs() < 0.1);
    }

    #[test]
    fn analytic_magnitude_matches_the_running_filter() {
        let corner = 2.0 * 343.0 / 0.0875;
        let analytic = magnitude(0.3, corner, TAU * 1_000.0);
        assert!((sine_gain(0.3, corner, 1_000.0) - analytic).abs() < 0.03);
    }
}
