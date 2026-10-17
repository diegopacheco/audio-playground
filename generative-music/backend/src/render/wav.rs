use std::f64::consts::TAU;

use super::Renderer;
use crate::compose::Song;
use crate::error::AppError;

pub const SAMPLE_RATE: u32 = 44_100;
const TAIL_SECONDS: f64 = 0.6;
const PEAK: f64 = 0.89;

#[derive(Clone, Copy)]
struct Envelope {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
}

const LEAD: Envelope = Envelope {
    attack: 0.01,
    decay: 0.12,
    sustain: 0.55,
    release: 0.15,
};

const PAD: Envelope = Envelope {
    attack: 0.08,
    decay: 0.3,
    sustain: 0.7,
    release: 0.35,
};

impl Envelope {
    fn held(&self, t: f64) -> f64 {
        if t < self.attack {
            t / self.attack
        } else if t < self.attack + self.decay {
            1.0 - (1.0 - self.sustain) * (t - self.attack) / self.decay
        } else {
            self.sustain
        }
    }

    fn level(&self, t: f64, length: f64) -> f64 {
        if t < length {
            self.held(t)
        } else {
            (self.held(length) * (1.0 - (t - length) / self.release)).max(0.0)
        }
    }
}

trait Voice {
    fn sample(&self, frequency: f64, t: f64) -> f64;
    fn envelope(&self) -> Envelope;
}

struct FmLead;

impl Voice for FmLead {
    fn sample(&self, frequency: f64, t: f64) -> f64 {
        let index = 0.5 + 2.5 * (-t * 6.0).exp();
        let modulator = (TAU * frequency * 2.0 * t).sin();
        (TAU * frequency * t + index * modulator).sin()
    }

    fn envelope(&self) -> Envelope {
        LEAD
    }
}

struct AdditivePad;

impl Voice for AdditivePad {
    fn sample(&self, frequency: f64, t: f64) -> f64 {
        [(1.0, 1.0), (2.0, 0.4), (3.0, 0.2)]
            .iter()
            .map(|(harmonic, amp)| amp * (TAU * frequency * harmonic * t).sin())
            .sum::<f64>()
            / 1.6
    }

    fn envelope(&self) -> Envelope {
        PAD
    }
}

pub fn midi_to_frequency(pitch: f64) -> f64 {
    440.0 * 2f64.powf((pitch - 69.0) / 12.0)
}

fn add_note(buffer: &mut [f64], voice: &dyn Voice, pitch: f64, start: f64, length: f64, gain: f64) {
    let envelope = voice.envelope();
    let frequency = midi_to_frequency(pitch);
    let rate = SAMPLE_RATE as f64;
    let first = (start * rate) as usize;
    let count = ((length + envelope.release) * rate) as usize;
    for i in 0..count {
        let Some(slot) = buffer.get_mut(first + i) else {
            break;
        };
        let t = i as f64 / rate;
        *slot += gain * envelope.level(t, length) * voice.sample(frequency, t);
    }
}

pub fn synthesize(song: &Song) -> Vec<f64> {
    let seconds_per_beat = 60.0 / song.bpm;
    let total = song.bars as f64 * 4.0 * seconds_per_beat + TAIL_SECONDS;
    let mut buffer = vec![0.0; (total * SAMPLE_RATE as f64) as usize];
    for chord in &song.chords {
        for pitch in &chord.pitches {
            add_note(
                &mut buffer,
                &AdditivePad,
                *pitch as f64,
                chord.start * seconds_per_beat,
                chord.duration * seconds_per_beat,
                0.12,
            );
        }
    }
    for note in &song.notes {
        add_note(
            &mut buffer,
            &FmLead,
            note.pitch as f64,
            note.start * seconds_per_beat,
            note.duration * seconds_per_beat * 0.95,
            0.35 * note.velocity as f64 / 127.0,
        );
    }
    let peak = buffer.iter().fold(0.0f64, |m, s| m.max(s.abs()));
    if peak > 0.0 {
        let scale = PEAK / peak;
        buffer.iter_mut().for_each(|s| *s *= scale);
    }
    buffer
}

pub fn encode(samples: &[f64]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend((36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(SAMPLE_RATE.to_le_bytes());
    out.extend((SAMPLE_RATE * 2).to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend(data_len.to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f64).round() as i16;
        out.extend(value.to_le_bytes());
    }
    out
}

pub struct WavRenderer;

impl Renderer for WavRenderer {
    fn content_type(&self) -> &'static str {
        "audio/wav"
    }

    fn render(&self, song: &Song) -> Result<Vec<u8>, AppError> {
        Ok(encode(&synthesize(song)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::{GenerateRequest, compose};

    fn song(bars: usize, bpm: f64) -> Song {
        compose(&GenerateRequest {
            key: "A".into(),
            scale: "minor".into(),
            progression: "i-VI-III-VII".into(),
            bars,
            bpm,
            generator: "markov".into(),
            seed: 5,
            order: 1,
        })
        .unwrap()
    }

    #[test]
    fn a4_is_440_and_octave_doubles() {
        assert!((midi_to_frequency(69.0) - 440.0).abs() < 1e-9);
        assert!((midi_to_frequency(81.0) - 880.0).abs() < 1e-9);
    }

    #[test]
    fn envelope_attacks_sustains_and_releases_to_silence() {
        assert_eq!(LEAD.level(0.0, 1.0), 0.0);
        assert!((LEAD.level(0.01, 1.0) - 1.0).abs() < 1e-9);
        assert!((LEAD.level(0.5, 1.0) - LEAD.sustain).abs() < 1e-9);
        assert!(LEAD.level(1.0 + LEAD.release, 1.0) < 1e-9);
    }

    #[test]
    fn duration_follows_bars_and_tempo() {
        let samples = synthesize(&song(4, 120.0));
        let expected = ((4.0 * 4.0 * 0.5 + TAIL_SECONDS) * SAMPLE_RATE as f64) as usize;
        assert_eq!(samples.len(), expected);
    }

    #[test]
    fn output_is_normalized_and_not_silent() {
        let samples = synthesize(&song(2, 100.0));
        let peak = samples.iter().fold(0.0f64, |m, s| m.max(s.abs()));
        assert!((peak - PEAK).abs() < 1e-9);
    }

    #[test]
    fn wav_header_is_16_bit_mono_pcm_with_matching_sizes() {
        let bytes = WavRenderer.render(&song(1, 120.0)).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes([bytes[20], bytes[21]]), 1);
        assert_eq!(u16::from_le_bytes([bytes[22], bytes[23]]), 1);
        assert_eq!(
            u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            SAMPLE_RATE
        );
        assert_eq!(u16::from_le_bytes([bytes[34], bytes[35]]), 16);
        let riff = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
        let data = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize;
        assert_eq!(riff + 8, bytes.len());
        assert_eq!(data + 44, bytes.len());
    }
}
