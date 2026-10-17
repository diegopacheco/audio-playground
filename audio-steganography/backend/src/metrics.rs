pub fn max_abs_diff(original: &[i16], stego: &[i16]) -> i32 {
    original
        .iter()
        .zip(stego)
        .map(|(a, b)| (*a as i32 - *b as i32).abs())
        .max()
        .unwrap_or(0)
}

pub fn snr_db(original: &[i16], stego: &[i16]) -> f64 {
    let (signal, noise) = original
        .iter()
        .zip(stego)
        .fold((0.0f64, 0.0f64), |(s, n), (a, b)| {
            let a = *a as f64;
            let d = a - *b as f64;
            (s + a * a, n + d * d)
        });
    if noise == 0.0 {
        return f64::INFINITY;
    }
    10.0 * (signal / noise).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_signals_have_infinite_snr_and_zero_difference() {
        let s = [1i16, 2, 3];
        assert_eq!(max_abs_diff(&s, &s), 0);
        assert!(snr_db(&s, &s).is_infinite());
    }

    #[test]
    fn noise_at_one_tenth_amplitude_is_20_db() {
        let original = [1000i16, -1000, 1000, -1000];
        let stego = [1100i16, -1100, 1100, -1100];
        assert!((snr_db(&original, &stego) - 20.0).abs() < 1e-9);
        assert_eq!(max_abs_diff(&original, &stego), 100);
    }
}
