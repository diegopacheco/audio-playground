use serde::Deserialize;

use crate::dsp::envelope::{Adsr, Envelope};
use crate::dsp::filter::{BiquadLowpass, Filter};
use crate::dsp::note::midi_to_frequency;
use crate::dsp::oscillator::{Oscillator, WaveOscillator};
use crate::error::AppError;
use crate::preset::Preset;

pub const MASTER_LEVEL: f32 = 0.3;
pub const MAX_SECONDS: f32 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Note {
    pub midi: u8,
    pub start: f32,
    pub duration: f32,
}

fn validate_notes(notes: &[Note], release: f32) -> Result<(), AppError> {
    if notes.is_empty() {
        return Err(AppError::InvalidNotes(
            "at least one note is required".to_string(),
        ));
    }
    for n in notes {
        if n.midi > 127 {
            return Err(AppError::InvalidNotes(format!(
                "midi {} out of range",
                n.midi
            )));
        }
        if !n.start.is_finite() || n.start < 0.0 {
            return Err(AppError::InvalidNotes("start must be >= 0".to_string()));
        }
        if !n.duration.is_finite() || n.duration <= 0.0 {
            return Err(AppError::InvalidNotes("duration must be > 0".to_string()));
        }
        if n.start + n.duration + release > MAX_SECONDS {
            return Err(AppError::InvalidNotes(format!(
                "phrase must fit in {MAX_SECONDS} seconds"
            )));
        }
    }
    Ok(())
}

fn render_voice(
    preset: &Preset,
    envelope: &Adsr,
    note: &Note,
    sample_rate: u32,
    out: &mut [f32],
) -> Result<(), AppError> {
    let mut osc = WaveOscillator::new(preset.waveform, midi_to_frequency(note.midi), sample_rate);
    let mut filter = BiquadLowpass::new(preset.cutoff, preset.resonance, sample_rate)?;
    let first = (note.start * sample_rate as f32) as usize;
    let length = ((note.duration + envelope.release_time()) * sample_rate as f32).ceil() as usize;
    let level = preset.gain * MASTER_LEVEL;
    for (i, slot) in out.iter_mut().skip(first).take(length).enumerate() {
        let t = i as f32 / sample_rate as f32;
        let y = filter.process(osc.next_sample());
        *slot += y * envelope.level(t, note.duration) * level;
    }
    Ok(())
}

pub fn render(preset: &Preset, notes: &[Note], sample_rate: u32) -> Result<Vec<f32>, AppError> {
    preset.validate()?;
    let envelope = preset.adsr();
    validate_notes(notes, envelope.release_time())?;
    let end = notes
        .iter()
        .map(|n| n.start + n.duration + envelope.release_time())
        .fold(0.0f32, f32::max);
    let mut out = vec![0.0f32; (end * sample_rate as f32).ceil() as usize + 1];
    for note in notes {
        render_voice(preset, &envelope, note, sample_rate, &mut out)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::built_in_presets;

    fn note(midi: u8, start: f32, duration: f32) -> Note {
        Note {
            midi,
            start,
            duration,
        }
    }

    #[test]
    fn length_covers_last_note_plus_release_tail() {
        let preset = built_in_presets().remove(0);
        let out = render(&preset, &[note(60, 0.5, 1.0)], 44_100).unwrap_or_default();
        let expected = ((1.5 + preset.release) * 44_100.0) as usize;
        assert!(out.len().abs_diff(expected) <= 2, "len {}", out.len());
    }

    #[test]
    fn audio_is_silent_before_the_first_note_and_audible_during_it() {
        let preset = built_in_presets().remove(0);
        let out = render(&preset, &[note(69, 0.5, 1.0)], 44_100).unwrap_or_default();
        assert!(out[..22_000].iter().all(|s| *s == 0.0));
        assert!(out[30_000..40_000].iter().any(|s| s.abs() > 0.05));
    }

    #[test]
    fn chord_is_louder_than_single_note_because_voices_are_summed() {
        let preset = built_in_presets().remove(0);
        let peak = |notes: &[Note]| {
            render(&preset, notes, 44_100)
                .unwrap_or_default()
                .iter()
                .fold(0.0f32, |m, s| m.max(s.abs()))
        };
        let single = peak(&[note(60, 0.0, 0.5)]);
        let chord = peak(&[note(60, 0.0, 0.5), note(64, 0.0, 0.5), note(67, 0.0, 0.5)]);
        assert!(chord > single * 1.5);
    }

    #[test]
    fn empty_or_invalid_phrases_are_rejected() {
        let preset = built_in_presets().remove(0);
        assert!(render(&preset, &[], 44_100).is_err());
        assert!(render(&preset, &[note(200, 0.0, 1.0)], 44_100).is_err());
        assert!(render(&preset, &[note(60, 0.0, 0.0)], 44_100).is_err());
        assert!(render(&preset, &[note(60, 59.0, 5.0)], 44_100).is_err());
    }
}
