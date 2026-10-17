pub struct FractionalDelay {
    buffer: Vec<f32>,
    write: usize,
}

impl FractionalDelay {
    pub fn new(max_delay_samples: usize) -> Self {
        FractionalDelay {
            buffer: vec![0.0; max_delay_samples + 2],
            write: 0,
        }
    }

    pub fn push(&mut self, x: f32) {
        self.write = (self.write + 1) % self.buffer.len();
        self.buffer[self.write] = x;
    }

    pub fn tap(&self, delay: f64) -> f32 {
        let len = self.buffer.len();
        let d = delay.clamp(0.0, (len - 2) as f64);
        let whole = d.floor() as usize;
        let frac = (d - whole as f64) as f32;
        let a = self.buffer[(self.write + len - whole) % len];
        let b = self.buffer[(self.write + len - whole - 1) % len];
        a + (b - a) * frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_delay_returns_the_exact_past_sample() {
        let mut line = FractionalDelay::new(16);
        for x in [1.0, 2.0, 3.0, 4.0] {
            line.push(x);
        }
        assert_eq!(line.tap(0.0), 4.0);
        assert_eq!(line.tap(3.0), 1.0);
    }

    #[test]
    fn fractional_delay_interpolates_between_neighbours_so_doppler_is_smooth() {
        let mut line = FractionalDelay::new(16);
        for x in [0.0, 10.0, 20.0] {
            line.push(x);
        }
        assert!((line.tap(0.25) - 17.5).abs() < 1e-5);
        assert!((line.tap(1.5) - 5.0).abs() < 1e-5);
    }
}
