use std::f32::consts::PI;

const MAX_FRACTION_OF_RATE: f32 = 0.45;

#[derive(Default)]
pub struct Resonator {
    y1: f32,
    y2: f32,
}

impl Resonator {
    pub fn process(&mut self, input: f32, frequency: f32, bandwidth: f32, rate: f32) -> f32 {
        let frequency = frequency.min(rate * MAX_FRACTION_OF_RATE);
        let radius = (-PI * bandwidth / rate).exp();
        let c = -radius * radius;
        let b = 2.0 * radius * (2.0 * PI * frequency / rate).cos();
        let a = 1.0 - b - c;
        let output = a * input + b * self.y1 + c * self.y2;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gain_at(frequency: f32, center: f32) -> f32 {
        let rate = 16000.0;
        let mut resonator = Resonator::default();
        let mut peak: f32 = 0.0;
        for i in 0..8000 {
            let x = (2.0 * PI * frequency * i as f32 / rate).sin();
            let y = resonator.process(x, center, 80.0, rate);
            if i > 4000 {
                peak = peak.max(y.abs());
            }
        }
        peak
    }

    #[test]
    fn boosts_its_formant_frequency_over_distant_ones() {
        assert!(gain_at(1000.0, 1000.0) > 5.0 * gain_at(3000.0, 1000.0));
    }

    #[test]
    fn caps_center_below_nyquist_so_the_filter_stays_stable() {
        let mut resonator = Resonator::default();
        let mut last = 0.0;
        for i in 0..2000 {
            last = resonator.process(if i == 0 { 1.0 } else { 0.0 }, 20000.0, 100.0, 16000.0);
        }
        assert!(last.is_finite() && last.abs() < 1e-3);
    }
}
