use crate::error::AppError;

pub const KEYS: [&str; 12] = [
    "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

pub const SCALES: [&str; 4] = ["major", "minor", "dorian", "pentatonic"];

const MAJOR: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
const MINOR: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];
const DORIAN: [i32; 7] = [0, 2, 3, 5, 7, 9, 10];
const PENTATONIC: [i32; 5] = [0, 2, 4, 7, 9];

const MELODY_BASE: i32 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Major,
    Minor,
    Dorian,
    Pentatonic,
}

impl Scale {
    pub fn parse(name: &str) -> Result<Scale, AppError> {
        match name.trim().to_ascii_lowercase().as_str() {
            "major" => Ok(Scale::Major),
            "minor" => Ok(Scale::Minor),
            "dorian" => Ok(Scale::Dorian),
            "pentatonic" => Ok(Scale::Pentatonic),
            _ => Err(AppError::InvalidScale(name.to_string())),
        }
    }

    pub fn melody_intervals(self) -> &'static [i32] {
        match self {
            Scale::Major => &MAJOR,
            Scale::Minor => &MINOR,
            Scale::Dorian => &DORIAN,
            Scale::Pentatonic => &PENTATONIC,
        }
    }

    pub fn harmony_intervals(self) -> &'static [i32; 7] {
        match self {
            Scale::Major | Scale::Pentatonic => &MAJOR,
            Scale::Minor => &MINOR,
            Scale::Dorian => &DORIAN,
        }
    }
}

pub fn parse_key(name: &str) -> Result<i32, AppError> {
    let trimmed = name.trim();
    let mut chars = trimmed.chars();
    let letter = chars
        .next()
        .ok_or_else(|| AppError::InvalidKey(name.to_string()))?;
    let base = match letter.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return Err(AppError::InvalidKey(name.to_string())),
    };
    let shift: i32 = match chars.as_str() {
        "" => 0,
        "#" => 1,
        "b" => -1,
        _ => return Err(AppError::InvalidKey(name.to_string())),
    };
    Ok((base + shift).rem_euclid(12))
}

#[derive(Debug, Clone, Copy)]
pub struct Tonality {
    pub root: i32,
    pub scale: Scale,
}

impl Tonality {
    pub fn len(&self) -> i32 {
        self.scale.melody_intervals().len() as i32
    }

    pub fn degree_to_pitch(&self, degree: i32) -> i32 {
        let intervals = self.scale.melody_intervals();
        let len = intervals.len() as i32;
        let octave = degree.div_euclid(len);
        let index = degree.rem_euclid(len) as usize;
        MELODY_BASE + self.root + octave * 12 + intervals[index]
    }

    pub fn pitch_class_degree(&self, pitch_class: i32) -> Option<i32> {
        let relative = (pitch_class - self.root).rem_euclid(12);
        self.scale
            .melody_intervals()
            .iter()
            .position(|interval| *interval == relative)
            .map(|index| index as i32)
    }

    #[cfg(test)]
    pub fn contains(&self, pitch: i32) -> bool {
        self.pitch_class_degree(pitch.rem_euclid(12)).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_map_to_pitch_classes_with_enharmonics() {
        assert_eq!(parse_key("C").unwrap(), 0);
        assert_eq!(parse_key("F#").unwrap(), 6);
        assert_eq!(parse_key("Gb").unwrap(), 6);
        assert_eq!(parse_key("Cb").unwrap(), 11);
        assert!(parse_key("H").is_err());
        assert!(parse_key("C##").is_err());
    }

    #[test]
    fn degree_wraps_into_next_octave() {
        let t = Tonality {
            root: 2,
            scale: Scale::Major,
        };
        assert_eq!(t.degree_to_pitch(0), 62);
        assert_eq!(t.degree_to_pitch(7), 74);
        assert_eq!(t.degree_to_pitch(-1), 61);
    }

    #[test]
    fn dorian_has_raised_sixth_compared_to_minor() {
        let dorian = Tonality {
            root: 2,
            scale: Scale::Dorian,
        };
        let minor = Tonality {
            root: 2,
            scale: Scale::Minor,
        };
        assert!(dorian.contains(71));
        assert!(!minor.contains(71));
    }
}
