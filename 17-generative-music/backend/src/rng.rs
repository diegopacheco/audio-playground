pub struct XorShift {
    state: u64,
}

impl XorShift {
    pub fn new(seed: u64) -> Self {
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        XorShift {
            state: if z == 0 { 0x2545_F491_4F6C_DD1D } else { z },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    pub fn chance(&mut self, percent: u32) -> bool {
        self.below(100) < percent as usize
    }

    pub fn weighted(&mut self, weights: &[u32]) -> usize {
        let total: u64 = weights.iter().map(|w| *w as u64).sum();
        if total == 0 {
            return 0;
        }
        let mut pick = self.next_u64() % total;
        for (index, weight) in weights.iter().enumerate() {
            let weight = *weight as u64;
            if pick < weight {
                return index;
            }
            pick -= weight;
        }
        weights.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_same_stream_so_songs_are_reproducible() {
        let mut a = XorShift::new(42);
        let mut b = XorShift::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn seed_zero_still_produces_a_moving_stream() {
        let mut rng = XorShift::new(0);
        let first = rng.next_u64();
        assert_ne!(first, 0);
        assert_ne!(first, rng.next_u64());
    }

    #[test]
    fn weighted_never_picks_zero_weight() {
        let mut rng = XorShift::new(7);
        for _ in 0..1000 {
            assert_eq!(rng.weighted(&[0, 5, 0]), 1);
        }
    }
}
