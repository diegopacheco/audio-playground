use serde::Serialize;

use crate::error::AppError;
use crate::theory::Tonality;

const NUMERALS: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];
const CHORD_BASE: i32 = 48;
const MAJOR_TRIAD: [i32; 3] = [0, 4, 7];
const MINOR_TRIAD: [i32; 3] = [0, 3, 7];
const DIMINISHED_TRIAD: [i32; 3] = [0, 3, 6];

#[derive(Debug, Clone, Serialize)]
pub struct Chord {
    pub symbol: String,
    pub root: i32,
    pub pitches: Vec<i32>,
    pub start: f64,
    pub duration: f64,
}

impl Chord {
    #[cfg(test)]
    pub fn has_pitch_class(&self, pitch: i32) -> bool {
        self.pitches
            .iter()
            .any(|p| p.rem_euclid(12) == pitch.rem_euclid(12))
    }
}

pub trait Harmony {
    fn chord(&self, numeral: &str) -> Result<Chord, AppError>;

    fn progression(&self, text: &str) -> Result<Vec<Chord>, AppError> {
        let chords = text
            .split(|c: char| {
                c == '-' || c == '–' || c == '—' || c == ',' || c == '|' || c.is_whitespace()
            })
            .filter(|token| !token.is_empty())
            .map(|token| self.chord(token))
            .collect::<Result<Vec<_>, _>>()?;
        if chords.is_empty() {
            return Err(AppError::InvalidNumeral(text.to_string()));
        }
        Ok(chords)
    }
}

pub struct RomanHarmony {
    pub tonality: Tonality,
}

impl Harmony for RomanHarmony {
    fn chord(&self, numeral: &str) -> Result<Chord, AppError> {
        let invalid = || AppError::InvalidNumeral(numeral.to_string());
        let (body, diminished) = match numeral.strip_suffix('°').or(numeral.strip_suffix('o')) {
            Some(body) => (body, true),
            None => (numeral, false),
        };
        let upper = body.chars().all(|c| c.is_ascii_uppercase());
        let lower = body.chars().all(|c| c.is_ascii_lowercase());
        if body.is_empty() || !(upper || lower) || (diminished && upper) {
            return Err(invalid());
        }
        let degree = NUMERALS
            .iter()
            .position(|n| *n == body.to_ascii_uppercase())
            .ok_or_else(invalid)?;
        let triad = match (upper, diminished) {
            (true, _) => MAJOR_TRIAD,
            (false, true) => DIMINISHED_TRIAD,
            (false, false) => MINOR_TRIAD,
        };
        let root_pc = self.tonality.root + self.tonality.scale.harmony_intervals()[degree];
        let root = CHORD_BASE + root_pc;
        Ok(Chord {
            symbol: numeral.to_string(),
            root,
            pitches: triad.iter().map(|interval| root + interval).collect(),
            start: 0.0,
            duration: 0.0,
        })
    }
}

pub fn lay_out(template: &[Chord], bars: usize, beats_per_bar: f64) -> Vec<Chord> {
    (0..bars)
        .map(|bar| {
            let mut chord = template[bar % template.len()].clone();
            chord.start = bar as f64 * beats_per_bar;
            chord.duration = beats_per_bar;
            chord
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::Scale;

    fn harmony(root: i32, scale: Scale) -> RomanHarmony {
        RomanHarmony {
            tonality: Tonality { root, scale },
        }
    }

    #[test]
    fn pop_progression_in_c_major_spells_c_g_am_f() {
        let chords = harmony(0, Scale::Major).progression("I–V–vi–IV").unwrap();
        let pcs: Vec<Vec<i32>> = chords
            .iter()
            .map(|c| c.pitches.iter().map(|p| p % 12).collect())
            .collect();
        assert_eq!(
            pcs,
            vec![vec![0, 4, 7], vec![7, 11, 2], vec![9, 0, 4], vec![5, 9, 0]]
        );
    }

    #[test]
    fn minor_key_roots_follow_the_minor_scale() {
        let chords = harmony(9, Scale::Minor).progression("i iv v VI").unwrap();
        let roots: Vec<i32> = chords.iter().map(|c| c.root % 12).collect();
        assert_eq!(roots, vec![9, 2, 4, 5]);
    }

    #[test]
    fn diminished_suffix_flattens_the_fifth() {
        let chord = harmony(0, Scale::Major).chord("vii°").unwrap();
        assert_eq!(chord.pitches, vec![59, 62, 65]);
    }

    #[test]
    fn parser_rejects_invalid_numerals_instead_of_guessing() {
        let h = harmony(0, Scale::Major);
        for bad in [
            "VIII", "IIV", "X", "Vi", "V°", "H", "", " - ", "I-IIII", "I-7",
        ] {
            assert!(h.progression(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn layout_cycles_progression_one_chord_per_bar() {
        let template = harmony(0, Scale::Major).progression("I-V").unwrap();
        let laid = lay_out(&template, 5, 4.0);
        let symbols: Vec<&str> = laid.iter().map(|c| c.symbol.as_str()).collect();
        assert_eq!(symbols, vec!["I", "V", "I", "V", "I"]);
        assert_eq!(laid[4].start, 16.0);
    }
}
