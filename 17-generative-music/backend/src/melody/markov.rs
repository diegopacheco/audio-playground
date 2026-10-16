use std::collections::HashMap;

use super::{Context, MelodyGenerator, Note, nearest_of_class, note, rhythm, velocity};
use crate::rng::XorShift;

const SEED_MELODIES: [&[usize]; 4] = [
    &[
        2, 2, 3, 4, 4, 3, 2, 1, 0, 0, 1, 2, 2, 1, 1, 2, 2, 3, 4, 4, 3, 2, 1, 0, 0, 1, 2, 1, 0, 0,
    ],
    &[
        0, 0, 4, 4, 5, 5, 4, 3, 3, 2, 2, 1, 1, 0, 4, 4, 3, 3, 2, 2, 1, 4, 4, 3, 3, 2, 2, 1, 0, 0,
    ],
    &[
        0, 1, 2, 0, 0, 1, 2, 0, 2, 3, 4, 2, 3, 4, 4, 5, 4, 3, 2, 0, 4, 5, 4, 3, 2, 0, 0, 4, 0,
    ],
    &[
        0, 1, 2, 3, 4, 5, 6, 0, 6, 4, 2, 0, 4, 5, 6, 0, 2, 1, 6, 0, 5, 3, 1, 6, 0,
    ],
];

const PENTATONIC_PROJECTION: [usize; 7] = [0, 1, 2, 2, 3, 4, 4];

pub struct MarkovMelody {
    order: usize,
    classes: usize,
    table: HashMap<Vec<usize>, Vec<u32>>,
}

impl MarkovMelody {
    pub fn train(order: usize, classes: usize) -> Self {
        let order = order.clamp(1, 2);
        let mut table: HashMap<Vec<usize>, Vec<u32>> = HashMap::new();
        for melody in SEED_MELODIES {
            let projected: Vec<usize> = melody
                .iter()
                .map(|degree| project(*degree, classes))
                .collect();
            for k in 1..=order {
                for window in projected.windows(k + 1) {
                    let counts = table
                        .entry(window[..k].to_vec())
                        .or_insert_with(|| vec![0; classes]);
                    counts[window[k]] += 1;
                }
            }
        }
        MarkovMelody {
            order,
            classes,
            table,
        }
    }

    fn next_class(&self, history: &[usize], rng: &mut XorShift) -> usize {
        for k in (1..=self.order.min(history.len())).rev() {
            if let Some(counts) = self.table.get(&history[history.len() - k..]) {
                return rng.weighted(counts);
            }
        }
        rng.below(self.classes)
    }

    #[cfg(test)]
    pub fn probabilities(&self, state: &[usize]) -> Option<&Vec<u32>> {
        self.table.get(state)
    }
}

fn project(degree: usize, classes: usize) -> usize {
    if classes == 5 {
        PENTATONIC_PROJECTION[degree % 7]
    } else {
        degree % classes
    }
}

impl MelodyGenerator for MarkovMelody {
    fn generate(&self, context: &Context, rng: &mut XorShift) -> Vec<Note> {
        let slots = rhythm(context.chords.len(), rng);
        let mut history = vec![0usize];
        let mut previous = 0;
        let mut notes = Vec::with_capacity(slots.len());
        for slot in &slots {
            let class = self.next_class(&history, rng);
            history.push(class);
            let degree = nearest_of_class(class as i32, previous, &context.tonality);
            previous = degree;
            let velocity = velocity(slot, rng);
            notes.push(note(&context.tonality, degree, slot, velocity));
        }
        notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_order_counts_reflect_seed_transitions() {
        let model = MarkovMelody::train(1, 7);
        let from_leading_tone = model.probabilities(&[6]).unwrap();
        let to_tonic = from_leading_tone[0];
        let max_other = from_leading_tone
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 0)
            .map(|(_, c)| *c)
            .max()
            .unwrap();
        assert!(
            to_tonic > max_other,
            "leading tone should mostly resolve to tonic"
        );
    }

    #[test]
    fn second_order_keys_have_two_degrees_of_context() {
        let model = MarkovMelody::train(2, 7);
        assert!(model.probabilities(&[2, 2]).is_some());
        assert!(model.probabilities(&[2]).is_some());
    }

    #[test]
    fn pentatonic_projection_never_produces_out_of_range_classes() {
        let model = MarkovMelody::train(2, 5);
        for counts in model.table.values() {
            assert_eq!(counts.len(), 5);
        }
    }
}
