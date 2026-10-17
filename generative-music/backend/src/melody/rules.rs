use super::{
    Context, MelodyGenerator, Note, chord_classes, nearest_of_class, note, range, rhythm, velocity,
};
use crate::rng::XorShift;
use crate::theory::Tonality;

const LEAP: i32 = 3;

pub struct RuleMelody;

fn chord_tone(classes: &[i32], previous: i32, tonality: &Tonality, rng: &mut XorShift) -> i32 {
    let candidates: Vec<i32> = classes
        .iter()
        .map(|class| nearest_of_class(*class, previous, tonality))
        .collect();
    let weights: Vec<u32> = candidates
        .iter()
        .map(|degree| 12 / (1 + (degree - previous).unsigned_abs()))
        .collect();
    candidates[rng.weighted(&weights)]
}

fn approach_tonic(previous: i32, tonality: &Tonality) -> i32 {
    let len = tonality.len();
    let (low, high) = range(tonality);
    [1, -1, len - 1, len + 1]
        .into_iter()
        .filter(|degree| *degree >= low && *degree <= high)
        .min_by_key(|degree| (degree - previous).abs())
        .unwrap_or(1)
}

fn free_motion(previous: i32, last_delta: i32, tonality: &Tonality, rng: &mut XorShift) -> i32 {
    if last_delta.abs() >= LEAP {
        return previous - last_delta.signum();
    }
    let size = [1, 0, 2, 3, 4][rng.weighted(&[60, 10, 18, 8, 4])];
    let keep_direction = last_delta != 0 && rng.chance(65);
    let direction = if keep_direction {
        last_delta.signum()
    } else if rng.chance(50) {
        1
    } else {
        -1
    };
    let (low, high) = range(tonality);
    let target = previous + direction * size;
    if target < low || target > high {
        previous - direction * size
    } else {
        target
    }
}

impl MelodyGenerator for RuleMelody {
    fn generate(&self, context: &Context, rng: &mut XorShift) -> Vec<Note> {
        let tonality = &context.tonality;
        let slots = rhythm(context.chords.len(), rng);
        let mut previous = 0;
        let mut last_delta = 0;
        let mut notes = Vec::with_capacity(slots.len());
        for (index, slot) in slots.iter().enumerate() {
            let remaining = slots.len() - index - 1;
            let classes = chord_classes(&context.chords[slot.bar], tonality);
            let degree = if remaining == 0 {
                nearest_of_class(0, previous, tonality)
            } else if slot.beat == 0.0 {
                chord_tone(&classes, previous, tonality, rng)
            } else if remaining == 1 {
                approach_tonic(previous, tonality)
            } else if slot.beat == 2.0 && rng.chance(60) {
                chord_tone(&classes, previous, tonality, rng)
            } else {
                free_motion(previous, last_delta, tonality, rng)
            };
            last_delta = degree - previous;
            previous = degree;
            let velocity = velocity(slot, rng);
            notes.push(note(tonality, degree, slot, velocity));
        }
        notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theory::Scale;

    #[test]
    fn leap_is_followed_by_step_in_opposite_direction() {
        let tonality = Tonality {
            root: 0,
            scale: Scale::Major,
        };
        let mut rng = XorShift::new(1);
        assert_eq!(free_motion(5, 4, &tonality, &mut rng), 4);
        assert_eq!(free_motion(1, -3, &tonality, &mut rng), 2);
    }

    #[test]
    fn free_motion_stays_inside_the_melodic_range() {
        let tonality = Tonality {
            root: 0,
            scale: Scale::Pentatonic,
        };
        let (low, high) = range(&tonality);
        let mut rng = XorShift::new(9);
        let mut degree = 0;
        let mut delta = 0;
        for _ in 0..2000 {
            let next = free_motion(degree, delta, &tonality, &mut rng);
            assert!(next >= low && next <= high);
            delta = next - degree;
            degree = next;
        }
    }

    #[test]
    fn mostly_stepwise_motion_between_weak_beats() {
        let tonality = Tonality {
            root: 0,
            scale: Scale::Major,
        };
        let mut rng = XorShift::new(5);
        let mut degree = 3;
        let mut steps = 0;
        for _ in 0..1000 {
            let next = free_motion(degree, 0, &tonality, &mut rng);
            if (next - degree).abs() <= 1 {
                steps += 1;
            }
            degree = 3;
        }
        assert!(steps > 600, "steps {steps}");
    }
}
