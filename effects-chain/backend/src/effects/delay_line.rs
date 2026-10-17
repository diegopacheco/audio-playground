pub struct DelayLine {
    buffer: Vec<f32>,
    pos: usize,
}

impl DelayLine {
    pub fn new(length: usize) -> Self {
        DelayLine {
            buffer: vec![0.0; length.max(1)],
            pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn read(&self) -> f32 {
        self.buffer[self.pos]
    }

    pub fn write(&mut self, sample: f32) {
        self.buffer[self.pos] = sample;
        self.pos = (self.pos + 1) % self.buffer.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_returns_the_sample_written_exactly_length_steps_ago() {
        let mut line = DelayLine::new(3);
        let mut out = Vec::new();
        for x in [1.0, 2.0, 3.0, 4.0, 5.0] {
            out.push(line.read());
            line.write(x);
        }
        assert_eq!(out, vec![0.0, 0.0, 0.0, 1.0, 2.0]);
    }
}
