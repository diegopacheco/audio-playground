pub mod beats;
pub mod fft;
pub mod onset;
pub mod resample;
pub mod tempo;

pub fn sample_at(values: &[f32], position: f32) -> f32 {
    if position < 0.0 {
        return 0.0;
    }
    let i = position.floor() as usize;
    let frac = position - i as f32;
    match (values.get(i), values.get(i + 1)) {
        (Some(a), Some(b)) => a + (b - a) * frac,
        (Some(a), None) => *a,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_positions_interpolate_between_neighbours() {
        let v = [0.0, 10.0, 20.0];
        assert_eq!(sample_at(&v, 0.5), 5.0);
        assert_eq!(sample_at(&v, 1.25), 12.5);
        assert_eq!(sample_at(&v, 5.0), 0.0);
        assert_eq!(sample_at(&v, -1.0), 0.0);
    }
}
