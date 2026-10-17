use crate::error::TunerError;
use serde::Serialize;

const NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
const A4_MIDI: f64 = 69.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Note {
    pub name: &'static str,
    pub octave: i32,
    pub cents: f64,
}

pub fn from_frequency(frequency: f64, a4: f64) -> Result<Note, TunerError> {
    if !a4.is_finite() || a4 <= 0.0 {
        return Err(TunerError::InvalidReference(a4));
    }
    if !frequency.is_finite() || frequency <= 0.0 {
        return Err(TunerError::InvalidFrequency(frequency));
    }
    let midi = A4_MIDI + 12.0 * (frequency / a4).log2();
    let nearest = midi.round();
    let index = nearest as i64;
    let target = a4 * 2f64.powf((nearest - A4_MIDI) / 12.0);
    Ok(Note {
        name: NAMES[index.rem_euclid(12) as usize],
        octave: (index.div_euclid(12) - 1) as i32,
        cents: cents_between(frequency, target),
    })
}

pub fn cents_between(frequency: f64, reference: f64) -> f64 {
    1200.0 * (frequency / reference).log2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_reference_maps_to_a4_with_zero_cents() {
        let note = from_frequency(440.0, 440.0).unwrap();
        assert_eq!((note.name, note.octave), ("A", 4));
        assert!(note.cents.abs() < 1e-9);
    }

    #[test]
    fn low_e_string_maps_to_e2() {
        let note = from_frequency(82.41, 440.0).unwrap();
        assert_eq!((note.name, note.octave), ("E", 2));
        assert!(note.cents.abs() < 1.0);
    }

    #[test]
    fn middle_c_belongs_to_octave_four_because_octaves_start_at_c() {
        let note = from_frequency(261.63, 440.0).unwrap();
        assert_eq!((note.name, note.octave), ("C", 4));
        let b3 = from_frequency(246.94, 440.0).unwrap();
        assert_eq!((b3.name, b3.octave), ("B", 3));
    }

    #[test]
    fn ten_cents_sharp_is_reported_as_plus_ten() {
        let sharp = 440.0 * 2f64.powf(10.0 / 1200.0);
        let note = from_frequency(sharp, 440.0).unwrap();
        assert_eq!(note.name, "A");
        assert!((note.cents - 10.0).abs() < 1e-6);
    }

    #[test]
    fn forty_nine_cents_flat_stays_on_the_same_note_but_fifty_one_flips() {
        let flat49 = 440.0 * 2f64.powf(-49.0 / 1200.0);
        let flat51 = 440.0 * 2f64.powf(-51.0 / 1200.0);
        let a = from_frequency(flat49, 440.0).unwrap();
        let g_sharp = from_frequency(flat51, 440.0).unwrap();
        assert_eq!(a.name, "A");
        assert!((a.cents + 49.0).abs() < 1e-6);
        assert_eq!(g_sharp.name, "G#");
        assert!((g_sharp.cents - 49.0).abs() < 1e-6);
    }

    #[test]
    fn changing_a4_reference_shifts_the_cents_offset() {
        let note = from_frequency(440.0, 432.0).unwrap();
        assert_eq!(note.name, "A");
        let expected = cents_between(440.0, 432.0);
        assert!((note.cents - expected).abs() < 1e-9);
        assert!((note.cents - 31.77).abs() < 0.01);
    }

    #[test]
    fn octave_up_is_twelve_hundred_cents() {
        assert!((cents_between(880.0, 440.0) - 1200.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_non_positive_inputs() {
        assert_eq!(
            from_frequency(0.0, 440.0),
            Err(TunerError::InvalidFrequency(0.0))
        );
        assert_eq!(
            from_frequency(440.0, -1.0),
            Err(TunerError::InvalidReference(-1.0))
        );
    }
}
