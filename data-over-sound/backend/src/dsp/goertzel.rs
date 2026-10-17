use std::f32::consts::PI;

pub fn power(samples: &[f32], freq: f32, sample_rate: u32) -> f32 {
    let omega = 2.0 * PI * freq / sample_rate as f32;
    let coeff = 2.0 * omega.cos();
    let mut s1 = 0.0f32;
    let mut s2 = 0.0f32;
    for &x in samples {
        let s0 = x + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - coeff * s1 * s2).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f32, sample_rate: u32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|n| (2.0 * PI * freq * n as f32 / sample_rate as f32).sin())
            .collect()
    }

    #[test]
    fn mark_tone_has_far_more_power_at_mark_than_at_space() {
        let window = sine(1200.0, 48000, 160);
        let mark = power(&window, 1200.0, 48000);
        let space = power(&window, 2200.0, 48000);
        assert!(mark > space * 100.0, "mark {mark} space {space}");
    }

    #[test]
    fn space_tone_has_far_more_power_at_space_than_at_mark() {
        let window = sine(2200.0, 48000, 160);
        assert!(power(&window, 2200.0, 48000) > power(&window, 1200.0, 48000) * 100.0);
    }

    #[test]
    fn picks_the_right_tone_among_four_fsk_tones() {
        let tones = [1200.0, 1800.0, 2400.0, 3000.0];
        for (expected, &freq) in tones.iter().enumerate() {
            let window = sine(freq, 44100, 147);
            let best = tones
                .iter()
                .enumerate()
                .max_by(|a, b| power(&window, *a.1, 44100).total_cmp(&power(&window, *b.1, 44100)))
                .map(|(i, _)| i);
            assert_eq!(best, Some(expected));
        }
    }

    #[test]
    fn power_of_full_scale_tone_is_quarter_of_window_squared() {
        let window = sine(1000.0, 8000, 80);
        let expected = (80.0f32 / 2.0).powi(2);
        let got = power(&window, 1000.0, 8000);
        assert!((got - expected).abs() / expected < 0.01, "got {got}");
    }
}
