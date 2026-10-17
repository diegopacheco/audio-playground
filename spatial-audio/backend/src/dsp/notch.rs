use std::f64::consts::TAU;

pub struct NotchFilter {
    fs: f64,
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

impl NotchFilter {
    pub fn new(fs: f64) -> Self {
        NotchFilter {
            fs,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn set(&mut self, freq: f64, q: f64, depth_db: f64) {
        let f = freq.clamp(20.0, 0.45 * self.fs);
        let w0 = TAU * f / self.fs;
        let alpha = w0.sin() / (2.0 * q);
        let a = 10f64.powf(-depth_db.abs() / 40.0);
        let cos = w0.cos();
        let a0 = 1.0 + alpha / a;
        self.b0 = (1.0 + alpha * a) / a0;
        self.b1 = -2.0 * cos / a0;
        self.b2 = (1.0 - alpha * a) / a0;
        self.a1 = -2.0 * cos / a0;
        self.a2 = (1.0 - alpha / a) / a0;
    }

    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gain_db(freq: f64) -> f64 {
        let fs = 44_100.0;
        let mut notch = NotchFilter::new(fs);
        notch.set(8_000.0, 2.0, 12.0);
        let n = 22_050;
        let mut peak: f64 = 0.0;
        for i in 0..n {
            let y = notch.process((TAU * freq * i as f64 / fs).sin());
            if i > n / 2 {
                peak = peak.max(y.abs());
            }
        }
        20.0 * peak.log10()
    }

    #[test]
    fn pinna_notch_carves_its_center_and_leaves_speech_band_alone() {
        assert!(gain_db(8_000.0) < -10.0);
        assert!(gain_db(1_000.0).abs() < 0.5);
    }
}
