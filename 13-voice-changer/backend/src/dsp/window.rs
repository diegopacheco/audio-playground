use std::f64::consts::PI;

pub fn hann(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| (0.5 - 0.5 * (2.0 * PI * i as f64 / size as f64).cos()) as f32)
        .collect()
}

pub fn energy(window: &[f32]) -> f32 {
    window.iter().map(|w| w * w).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squared_hann_overlaps_to_a_constant_at_quarter_hop_so_overlap_add_has_flat_gain() {
        let size = 1024;
        let hop = size / 4;
        let window = hann(size);
        let expected = energy(&window) / hop as f32;
        for n in size..2 * size {
            let sum: f32 = (0..=n / hop)
                .filter_map(|m| n.checked_sub(m * hop))
                .filter(|offset| *offset < size)
                .map(|offset| window[offset] * window[offset])
                .sum();
            assert!((sum - expected).abs() < 1e-3, "sample {n} sums to {sum}");
        }
    }
}
