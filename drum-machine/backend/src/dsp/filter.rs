use std::f32::consts::PI;

pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    fn from_coefficients(b: [f32; 3], a: [f32; 3]) -> Self {
        Self {
            b0: b[0] / a[0],
            b1: b[1] / a[0],
            b2: b[2] / a[0],
            a1: a[1] / a[0],
            a2: a[2] / a[0],
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn omega(freq: f32, q: f32, sample_rate: u32) -> (f32, f32) {
        let safe = freq.min(sample_rate as f32 * 0.45);
        let w = 2.0 * PI * safe / sample_rate as f32;
        (w.cos(), w.sin() / (2.0 * q))
    }

    pub fn highpass(freq: f32, q: f32, sample_rate: u32) -> Self {
        let (cos, alpha) = Self::omega(freq, q, sample_rate);
        Self::from_coefficients(
            [(1.0 + cos) / 2.0, -(1.0 + cos), (1.0 + cos) / 2.0],
            [1.0 + alpha, -2.0 * cos, 1.0 - alpha],
        )
    }

    pub fn bandpass(freq: f32, q: f32, sample_rate: u32) -> Self {
        let (cos, alpha) = Self::omega(freq, q, sample_rate);
        Self::from_coefficients([alpha, 0.0, -alpha], [1.0 + alpha, -2.0 * cos, 1.0 - alpha])
    }

    pub fn process(&mut self, x: f32) -> f32 {
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

    fn sine_rms_through(filter: &mut Biquad, freq: f32) -> f32 {
        let sr = 44100;
        let n = 8820;
        let out: Vec<f32> = (0..n)
            .map(|i| filter.process((2.0 * PI * freq * i as f32 / sr as f32).sin()))
            .collect();
        let tail = &out[n / 2..];
        (tail.iter().map(|v| v * v).sum::<f32>() / tail.len() as f32).sqrt()
    }

    #[test]
    fn highpass_must_kill_bass_and_keep_treble_so_hihat_stays_bright() {
        let low = sine_rms_through(&mut Biquad::highpass(7000.0, 0.7, 44100), 100.0);
        let high = sine_rms_through(&mut Biquad::highpass(7000.0, 0.7, 44100), 12000.0);
        assert!(low < 0.01, "low {low}");
        assert!(high > 0.5, "high {high}");
    }

    #[test]
    fn cutoff_above_nyquist_must_stay_stable_instead_of_exploding() {
        let mut f = Biquad::highpass(7000.0, 0.7, 8000);
        let out: Vec<f32> = (0..8000)
            .map(|i| f.process(if i == 0 { 1.0 } else { 0.0 }))
            .collect();
        assert!(out.iter().all(|v| v.is_finite() && v.abs() < 2.0));
    }

    #[test]
    fn bandpass_must_favor_its_center_frequency() {
        let center = sine_rms_through(&mut Biquad::bandpass(1000.0, 2.0, 44100), 1000.0);
        let far = sine_rms_through(&mut Biquad::bandpass(1000.0, 2.0, 44100), 10000.0);
        assert!(center > 4.0 * far, "center {center} far {far}");
    }
}
