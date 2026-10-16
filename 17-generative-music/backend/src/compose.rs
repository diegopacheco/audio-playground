use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::harmony::{Chord, Harmony, RomanHarmony, lay_out};
use crate::melody::markov::MarkovMelody;
use crate::melody::rules::RuleMelody;
use crate::melody::{BEATS_PER_BAR, Context, MelodyGenerator, Note};
use crate::rng::XorShift;
use crate::theory::{Scale, Tonality, parse_key};

pub const GENERATORS: [&str; 2] = ["markov", "rule"];
pub const MAX_BARS: usize = 64;
pub const MIN_BPM: f64 = 40.0;
pub const MAX_BPM: f64 = 240.0;

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateRequest {
    pub key: String,
    pub scale: String,
    pub progression: String,
    pub bars: usize,
    pub bpm: f64,
    pub generator: String,
    pub seed: u64,
    #[serde(default = "default_order")]
    pub order: usize,
}

fn default_order() -> usize {
    1
}

#[derive(Debug, Clone, Serialize)]
pub struct Song {
    pub notes: Vec<Note>,
    pub chords: Vec<Chord>,
    pub bpm: f64,
    pub bars: usize,
}

fn generator(
    name: &str,
    order: usize,
    tonality: &Tonality,
) -> Result<Box<dyn MelodyGenerator>, AppError> {
    match name {
        "markov" => {
            if !(1..=2).contains(&order) {
                return Err(AppError::InvalidRequest(format!(
                    "order must be 1 or 2, got {order}"
                )));
            }
            Ok(Box::new(MarkovMelody::train(
                order,
                tonality.len() as usize,
            )))
        }
        "rule" => Ok(Box::new(RuleMelody)),
        other => Err(AppError::InvalidRequest(format!(
            "unknown generator: {other}"
        ))),
    }
}

pub fn compose(request: &GenerateRequest) -> Result<Song, AppError> {
    if request.bars == 0 || request.bars > MAX_BARS {
        return Err(AppError::InvalidRequest(format!(
            "bars must be 1..={MAX_BARS}"
        )));
    }
    if !(MIN_BPM..=MAX_BPM).contains(&request.bpm) {
        return Err(AppError::InvalidRequest(format!(
            "bpm must be {MIN_BPM}..={MAX_BPM}"
        )));
    }
    let tonality = Tonality {
        root: parse_key(&request.key)?,
        scale: Scale::parse(&request.scale)?,
    };
    let harmony = RomanHarmony { tonality };
    let template = harmony.progression(&request.progression)?;
    let chords = lay_out(&template, request.bars, BEATS_PER_BAR);
    let generator = generator(&request.generator, request.order, &tonality)?;
    let mut rng = XorShift::new(request.seed);
    let notes = generator.generate(
        &Context {
            tonality,
            chords: &chords,
        },
        &mut rng,
    );
    Ok(Song {
        notes,
        chords,
        bpm: request.bpm,
        bars: request.bars,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(generator: &str, scale: &str, seed: u64) -> GenerateRequest {
        GenerateRequest {
            key: "D".into(),
            scale: scale.into(),
            progression: "I–V–vi–IV".into(),
            bars: 8,
            bpm: 110.0,
            generator: generator.into(),
            seed,
            order: 2,
        }
    }

    #[test]
    fn same_seed_reproduces_identical_melody_for_every_generator() {
        for generator in GENERATORS {
            let a = compose(&request(generator, "major", 1234)).unwrap();
            let b = compose(&request(generator, "major", 1234)).unwrap();
            assert_eq!(a.notes, b.notes, "{generator}");
        }
    }

    #[test]
    fn different_seed_changes_the_melody() {
        for generator in GENERATORS {
            let a = compose(&request(generator, "major", 1)).unwrap();
            let b = compose(&request(generator, "major", 2)).unwrap();
            assert_ne!(a.notes, b.notes, "{generator}");
        }
    }

    #[test]
    fn every_melody_note_belongs_to_the_chosen_scale() {
        for scale in ["major", "minor", "dorian", "pentatonic"] {
            for generator in GENERATORS {
                for seed in 0..30 {
                    let mut req = request(generator, scale, seed);
                    req.progression = if scale == "major" || scale == "pentatonic" {
                        "I-V-vi-IV".into()
                    } else {
                        "i-iv-v-i".into()
                    };
                    let song = compose(&req).unwrap();
                    let tonality = Tonality {
                        root: 2,
                        scale: Scale::parse(scale).unwrap(),
                    };
                    for note in &song.notes {
                        assert!(
                            tonality.contains(note.pitch as i32),
                            "{scale} {generator} {seed} pitch {}",
                            note.pitch
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn rule_generator_lands_chord_tones_on_every_downbeat_and_cadences_to_tonic() {
        for scale in ["major", "pentatonic"] {
            for seed in 0..100 {
                let song = compose(&request("rule", scale, seed)).unwrap();
                for chord in &song.chords {
                    let downbeat = song
                        .notes
                        .iter()
                        .find(|n| n.start == chord.start)
                        .expect("a note must start on beat 1 of every bar");
                    assert!(
                        chord.has_pitch_class(downbeat.pitch as i32),
                        "seed {seed} bar at {} chord {}",
                        chord.start,
                        chord.symbol
                    );
                }
                let last = song.notes.last().unwrap();
                assert_eq!(last.pitch % 12, 2, "seed {seed} must end on D");
                assert_eq!(last.start + last.duration, song.bars as f64 * BEATS_PER_BAR);
            }
        }
    }

    #[test]
    fn notes_cover_the_whole_song_without_gaps_or_overlaps() {
        let song = compose(&request("markov", "dorian", 77)).unwrap();
        let mut cursor = 0.0;
        for note in &song.notes {
            assert_eq!(note.start, cursor);
            cursor += note.duration;
        }
        assert_eq!(cursor, 32.0);
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        let mut req = request("rule", "major", 1);
        req.progression = "I-VIII".into();
        assert!(compose(&req).is_err());
        let mut req = request("nope", "major", 1);
        req.generator = "nope".into();
        assert!(compose(&req).is_err());
        let mut req = request("markov", "major", 1);
        req.order = 3;
        assert!(compose(&req).is_err());
        let mut req = request("rule", "lydian", 1);
        req.scale = "lydian".into();
        assert!(compose(&req).is_err());
        let mut req = request("rule", "major", 1);
        req.bars = 0;
        assert!(compose(&req).is_err());
    }
}
