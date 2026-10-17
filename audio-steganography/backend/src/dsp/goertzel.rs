use std::f64::consts::PI;

pub fn hann(n: usize) -> Vec<f64> {
    if n < 2 {
        return vec![1.0; n];
    }
    (0..n)
        .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f64 / (n - 1) as f64).cos())
        .collect()
}

pub fn power(block: &[f64], freq: f64, sample_rate: f64) -> f64 {
    let coeff = 2.0 * (2.0 * PI * freq / sample_rate).cos();
    let (mut s1, mut s2) = (0.0, 0.0);
    for &x in block {
        let s0 = x + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    s1 * s1 + s2 * s2 - coeff * s1 * s2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(freq: f64, n: usize, sr: f64) -> Vec<f64> {
        (0..n)
            .map(|i| (2.0 * PI * freq * i as f64 / sr).sin())
            .collect()
    }

    #[test]
    fn target_frequency_dominates_a_neighbouring_bin() {
        let sr = 44_100.0;
        let block = tone(18_200.0, 352, sr);
        assert!(power(&block, 18_200.0, sr) > 100.0 * power(&block, 19_200.0, sr));
    }

    #[test]
    fn hann_window_is_zero_at_edges_and_one_in_the_middle() {
        let w = hann(5);
        assert!(w[0].abs() < 1e-12);
        assert!((w[2] - 1.0).abs() < 1e-12);
    }
}
