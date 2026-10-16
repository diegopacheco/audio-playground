use crate::dsp::noise::Rng;
use std::f32::consts::PI;

pub struct SongSpec {
    pub name: &'static str,
    pub bpm: f32,
    pub root: i32,
    pub scale: &'static [i32],
    pub harmonics: &'static [f32],
    pub beats_per_bar: usize,
    pub steps_per_beat: usize,
    pub progression: &'static [usize],
    pub drums: bool,
    pub seed: u64,
}

const MAJOR: &[i32] = &[0, 2, 4, 5, 7, 9, 11];
const MINOR: &[i32] = &[0, 2, 3, 5, 7, 8, 10];
const PENTATONIC: &[i32] = &[0, 2, 4, 7, 9];
const BLUES: &[i32] = &[0, 3, 5, 6, 7, 10];
const DORIAN: &[i32] = &[0, 2, 3, 5, 7, 9, 10];
const MIXOLYDIAN: &[i32] = &[0, 2, 4, 5, 7, 9, 10];

pub const SONGS: &[SongSpec] = &[
    SongSpec {
        name: "Morning Arpeggio",
        bpm: 120.0,
        root: 48,
        scale: MAJOR,
        harmonics: &[1.0, 0.4, 0.2],
        beats_per_bar: 4,
        steps_per_beat: 2,
        progression: &[0, 4, 5, 3],
        drums: true,
        seed: 11,
    },
    SongSpec {
        name: "Minor Waltz",
        bpm: 96.0,
        root: 45,
        scale: MINOR,
        harmonics: &[1.0, 0.6, 0.1, 0.2],
        beats_per_bar: 3,
        steps_per_beat: 2,
        progression: &[0, 3, 4, 0],
        drums: false,
        seed: 23,
    },
    SongSpec {
        name: "Pentatonic Run",
        bpm: 140.0,
        root: 50,
        scale: PENTATONIC,
        harmonics: &[1.0, 0.2],
        beats_per_bar: 4,
        steps_per_beat: 4,
        progression: &[0, 2, 3, 1],
        drums: true,
        seed: 37,
    },
    SongSpec {
        name: "Blues Shuffle",
        bpm: 100.0,
        root: 43,
        scale: BLUES,
        harmonics: &[1.0, 0.5, 0.35, 0.2],
        beats_per_bar: 4,
        steps_per_beat: 3,
        progression: &[0, 0, 2, 0, 4, 2, 0, 4],
        drums: true,
        seed: 41,
    },
    SongSpec {
        name: "Chord Pad Drift",
        bpm: 72.0,
        root: 52,
        scale: DORIAN,
        harmonics: &[1.0, 0.1, 0.05],
        beats_per_bar: 4,
        steps_per_beat: 2,
        progression: &[0, 3, 6, 4],
        drums: false,
        seed: 59,
    },
    SongSpec {
        name: "Square Chiptune",
        bpm: 160.0,
        root: 47,
        scale: MIXOLYDIAN,
        harmonics: &[1.0, 0.0, 0.33, 0.0, 0.2],
        beats_per_bar: 4,
        steps_per_beat: 2,
        progression: &[0, 6, 3, 4],
        drums: true,
        seed: 73,
    },
];

const MAX_PARTIAL_HZ: f32 = 3600.0;

fn midi_to_hz(midi: i32) -> f32 {
    440.0 * 2f32.powf((midi - 69) as f32 / 12.0)
}

fn degree_to_midi(spec: &SongSpec, index: i32) -> i32 {
    let len = spec.scale.len() as i32;
    let octave = index.div_euclid(len);
    spec.root + 12 * octave + spec.scale[index.rem_euclid(len) as usize]
}

struct Renderer {
    rate: f32,
    buffer: Vec<f32>,
}

impl Renderer {
    fn tone(&mut self, start: f32, dur: f32, freq: f32, amp: f32, harmonics: &[f32]) {
        let begin = (start * self.rate) as usize;
        let len = (dur * self.rate) as usize;
        let attack = 0.01 * self.rate;
        let release = 0.02 * self.rate;
        let tau = dur * 0.7 * self.rate;
        for n in 0..len {
            let Some(slot) = self.buffer.get_mut(begin + n) else {
                break;
            };
            let t = n as f32 / self.rate;
            let nf = n as f32;
            let mut env = (nf / attack).min(1.0) * (-nf / tau).exp();
            let left = (len - n) as f32;
            if left < release {
                env *= left / release;
            }
            let mut v = 0.0;
            for (h, weight) in harmonics.iter().enumerate() {
                let partial = freq * (h + 1) as f32;
                if *weight > 0.0 && partial < MAX_PARTIAL_HZ {
                    v += weight * (2.0 * PI * partial * t).sin();
                }
            }
            *slot += amp * env * v;
        }
    }

    fn kick(&mut self, start: f32) {
        let begin = (start * self.rate) as usize;
        let len = (0.15 * self.rate) as usize;
        let mut phase = 0.0;
        for n in 0..len {
            let Some(slot) = self.buffer.get_mut(begin + n) else {
                break;
            };
            let t = n as f32 / self.rate;
            let freq = 50.0 + 90.0 * (-t * 30.0).exp();
            phase += 2.0 * PI * freq / self.rate;
            *slot += 0.5 * (-t * 25.0).exp() * phase.sin();
        }
    }

    fn hat(&mut self, start: f32, rng: &mut Rng) {
        let begin = (start * self.rate) as usize;
        let len = (0.03 * self.rate) as usize;
        let mut prev = 0.0;
        for n in 0..len {
            let Some(slot) = self.buffer.get_mut(begin + n) else {
                break;
            };
            let white = rng.signed();
            let high = white - prev;
            prev = white;
            *slot += 0.06 * (-(n as f32) / (0.008 * self.rate)).exp() * high;
        }
    }
}

pub fn render(spec: &SongSpec, rate: u32, seconds: f32) -> Vec<f32> {
    let mut rng = Rng::new(spec.seed);
    let mut r = Renderer {
        rate: rate as f32,
        buffer: vec![0.0; (seconds * rate as f32) as usize],
    };
    let beat = 60.0 / spec.bpm;
    let step = beat / spec.steps_per_beat as f32;
    let bar = beat * spec.beats_per_bar as f32;
    let len = spec.scale.len() as i32;
    let bars = (seconds / bar).ceil() as usize;
    for b in 0..bars {
        let bar_start = b as f32 * bar;
        let degree = spec.progression[b % spec.progression.len()] as i32;
        for offset in [0, 2, 4] {
            let freq = midi_to_hz(degree_to_midi(spec, len + degree + offset));
            r.tone(bar_start, bar * 0.95, freq, 0.12, spec.harmonics);
        }
        for k in 0..spec.beats_per_bar {
            let t = bar_start + k as f32 * beat;
            let freq = midi_to_hz(degree_to_midi(spec, degree - len));
            r.tone(t, beat * 0.8, freq, 0.2, &[1.0, 0.3]);
            if spec.drums {
                if k % 2 == 0 {
                    r.kick(t);
                }
                r.hat(t + beat / 2.0, &mut rng);
            }
        }
    }
    let total_steps = (seconds / step) as usize;
    let mut index = 2 * len;
    let mut s = 0;
    while s < total_steps {
        let hold = 1 + rng.below(2);
        if rng.unit() > 0.12 {
            let freq = midi_to_hz(degree_to_midi(spec, index));
            r.tone(
                s as f32 * step,
                hold as f32 * step,
                freq,
                0.3,
                spec.harmonics,
            );
        }
        let jump = rng.below(7) as i32 - 3;
        index = (index + jump).clamp(len + 2, 3 * len);
        s += hold;
    }
    let peak = r.buffer.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    if peak > 0.0 {
        let gain = 0.8 / peak;
        r.buffer.iter_mut().for_each(|v| *v *= gain);
    }
    r.buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_has_requested_length_and_normalized_peak() {
        let song = render(&SONGS[0], 8000, 5.0);
        assert_eq!(song.len(), 40000);
        let peak = song.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!((peak - 0.8).abs() < 1e-3);
    }

    #[test]
    fn songs_are_different_signals() {
        let a = render(&SONGS[0], 8000, 3.0);
        let b = render(&SONGS[1], 8000, 3.0);
        let diff: f32 = a.iter().zip(&b).map(|(x, y)| (x - y).abs()).sum();
        assert!(diff / a.len() as f32 > 0.05);
    }

    #[test]
    fn rendering_is_deterministic_so_the_library_index_is_stable() {
        assert_eq!(render(&SONGS[2], 8000, 2.0), render(&SONGS[2], 8000, 2.0));
    }
}
