pub fn midi_to_frequency(midi: u8) -> f32 {
    440.0 * 2f32.powf((f32::from(midi) - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_is_concert_pitch_440() {
        assert!((midi_to_frequency(69) - 440.0).abs() < 1e-3);
    }

    #[test]
    fn one_octave_up_doubles_frequency_so_keys_stay_in_tune() {
        assert!((midi_to_frequency(81) - 880.0).abs() < 1e-3);
        assert!((midi_to_frequency(57) - 220.0).abs() < 1e-3);
    }

    #[test]
    fn middle_c_matches_equal_temperament() {
        assert!((midi_to_frequency(60) - 261.6256).abs() < 1e-2);
    }
}
