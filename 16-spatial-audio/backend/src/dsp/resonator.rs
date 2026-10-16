use std::f64::consts::{PI, TAU};

pub struct Resonator {
    a1: f64,
    a2: f64,
    gain: f64,
    y1: f64,
    y2: f64,
}

impl Resonator {
    pub fn new(freq: f64, bandwidth: f64, fs: f64) -> Self {
        let mut r = Resonator {
            a1: 0.0,
            a2: 0.0,
            gain: 0.0,
            y1: 0.0,
            y2: 0.0,
        };
        r.tune(freq, bandwidth, fs);
        r
    }

    pub fn tune(&mut self, freq: f64, bandwidth: f64, fs: f64) {
        let f = freq.min(0.45 * fs);
        let radius = (-PI * bandwidth / fs).exp();
        self.a1 = 2.0 * radius * (TAU * f / fs).cos();
        self.a2 = -radius * radius;
        self.gain = 1.0 - radius;
    }

    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.gain * x + self.a1 * self.y1 + self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}
