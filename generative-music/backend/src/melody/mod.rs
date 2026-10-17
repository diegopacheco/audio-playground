pub mod markov;
pub mod rules;

use serde::Serialize;

use crate::harmony::Chord;
use crate::rng::XorShift;
use crate::theory::Tonality;

pub const BEATS_PER_BAR: f64 = 4.0;

const PATTERNS: [&[f64]; 7] = [
    &[1.0, 1.0, 1.0, 1.0],
    &[2.0, 1.0, 1.0],
    &[1.0, 1.0, 2.0],
    &[1.0, 0.5, 0.5, 1.0, 1.0],
    &[0.5, 0.5, 1.0, 0.5, 0.5, 1.0],
    &[1.5, 0.5, 1.0, 1.0],
    &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
];

const FINAL_PATTERNS: [&[f64]; 3] = [&[1.0, 1.0, 2.0], &[2.0, 2.0], &[1.0, 0.5, 0.5, 2.0]];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Note {
    pub pitch: u8,
    pub start: f64,
    pub duration: f64,
    pub velocity: u8,
}

pub struct Context<'a> {
    pub tonality: Tonality,
    pub chords: &'a [Chord],
}

#[derive(Debug, Clone, Copy)]
pub struct Slot {
    pub bar: usize,
    pub beat: f64,
    pub duration: f64,
}

impl Slot {
    pub fn start(&self) -> f64 {
        self.bar as f64 * BEATS_PER_BAR + self.beat
    }
}

pub trait MelodyGenerator {
    fn generate(&self, context: &Context, rng: &mut XorShift) -> Vec<Note>;
}

pub fn rhythm(bars: usize, rng: &mut XorShift) -> Vec<Slot> {
    let mut slots = Vec::new();
    for bar in 0..bars {
        let pattern = if bar + 1 == bars {
            FINAL_PATTERNS[rng.below(FINAL_PATTERNS.len())]
        } else {
            PATTERNS[rng.below(PATTERNS.len())]
        };
        let mut beat = 0.0;
        for duration in pattern {
            slots.push(Slot {
                bar,
                beat,
                duration: *duration,
            });
            beat += duration;
        }
    }
    slots
}

pub fn range(tonality: &Tonality) -> (i32, i32) {
    (-2, tonality.len() + 4)
}

pub fn nearest_of_class(class: i32, previous: i32, tonality: &Tonality) -> i32 {
    let len = tonality.len();
    let (low, high) = range(tonality);
    (-2..=3)
        .map(|octave| class + octave * len)
        .filter(|degree| *degree >= low && *degree <= high)
        .min_by_key(|degree| ((degree - previous).abs(), *degree))
        .unwrap_or(class)
}

pub fn chord_classes(chord: &Chord, tonality: &Tonality) -> Vec<i32> {
    let classes: Vec<i32> = chord
        .pitches
        .iter()
        .filter_map(|pitch| tonality.pitch_class_degree(pitch.rem_euclid(12)))
        .collect();
    if !classes.is_empty() {
        return classes;
    }
    let root_pc = chord.root.rem_euclid(12);
    (0..tonality.len())
        .min_by_key(|class| {
            let pc = (tonality.degree_to_pitch(*class)).rem_euclid(12);
            let diff = (pc - root_pc).rem_euclid(12);
            diff.min(12 - diff)
        })
        .into_iter()
        .collect()
}

pub fn velocity(slot: &Slot, rng: &mut XorShift) -> u8 {
    if slot.beat == 0.0 {
        100
    } else if slot.beat == 2.0 {
        90
    } else {
        72 + rng.below(12) as u8
    }
}

pub fn note(tonality: &Tonality, degree: i32, slot: &Slot, velocity: u8) -> Note {
    Note {
        pitch: tonality.degree_to_pitch(degree).clamp(0, 127) as u8,
        start: slot.start(),
        duration: slot.duration,
        velocity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bar_rhythm_fills_exactly_four_beats_and_starts_on_beat_one() {
        let mut rng = XorShift::new(3);
        let slots = rhythm(16, &mut rng);
        for bar in 0..16 {
            let in_bar: Vec<&Slot> = slots.iter().filter(|s| s.bar == bar).collect();
            assert_eq!(in_bar[0].beat, 0.0);
            let total: f64 = in_bar.iter().map(|s| s.duration).sum();
            assert_eq!(total, BEATS_PER_BAR);
        }
    }

    #[test]
    fn final_bar_has_at_least_two_notes_so_beat_one_and_cadence_can_differ() {
        for seed in 0..50 {
            let mut rng = XorShift::new(seed);
            let slots = rhythm(4, &mut rng);
            assert!(slots.iter().filter(|s| s.bar == 3).count() >= 2);
        }
    }
}
