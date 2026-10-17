use std::collections::VecDeque;

pub struct LatencyQueue {
    buffer: VecDeque<f32>,
}

impl LatencyQueue {
    pub fn new(latency: usize) -> Self {
        Self {
            buffer: std::iter::repeat_n(0.0, latency).collect(),
        }
    }

    pub fn exchange(&mut self, produced: &[f32], count: usize) -> Vec<f32> {
        self.buffer.extend(produced);
        let available = count.min(self.buffer.len());
        let mut out: Vec<f32> = self.buffer.drain(..available).collect();
        out.resize(count, 0.0);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_by_the_declared_latency_so_blocks_out_match_blocks_in() {
        let mut queue = LatencyQueue::new(3);
        assert_eq!(queue.exchange(&[1.0, 2.0], 2), vec![0.0, 0.0]);
        assert_eq!(queue.exchange(&[3.0, 4.0], 2), vec![0.0, 1.0]);
        assert_eq!(queue.exchange(&[], 4), vec![2.0, 3.0, 4.0, 0.0]);
    }
}
