mod pitch_shift;
mod presets;
mod queue;
mod robot;
mod whisper;

pub use pitch_shift::PhaseVocoderPitchShift;
pub use presets::{Chipmunk, Deep};
pub use robot::Robot;
pub use whisper::Whisper;

use crate::error::VoiceError;
use serde::Serialize;

pub trait VoiceEffect: Send {
    fn process_block(&mut self, input: &[f32]) -> Vec<f32>;
    fn latency(&self) -> usize;
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EffectSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub min_semitones: f32,
    pub max_semitones: f32,
    pub default_semitones: f32,
}

pub const CATALOG: [EffectSpec; 5] = [
    EffectSpec {
        name: "pitch",
        label: "Pitch shift",
        min_semitones: -12.0,
        max_semitones: 12.0,
        default_semitones: 4.0,
    },
    EffectSpec {
        name: "chipmunk",
        label: "Chipmunk",
        min_semitones: 7.0,
        max_semitones: 12.0,
        default_semitones: 9.0,
    },
    EffectSpec {
        name: "deep",
        label: "Deep",
        min_semitones: -7.0,
        max_semitones: -5.0,
        default_semitones: -6.0,
    },
    EffectSpec {
        name: "robot",
        label: "Robot",
        min_semitones: -12.0,
        max_semitones: 12.0,
        default_semitones: 0.0,
    },
    EffectSpec {
        name: "whisper",
        label: "Whisper",
        min_semitones: 0.0,
        max_semitones: 0.0,
        default_semitones: 0.0,
    },
];

pub fn spec(name: &str) -> Result<EffectSpec, VoiceError> {
    CATALOG
        .iter()
        .find(|spec| spec.name == name)
        .copied()
        .ok_or_else(|| VoiceError::UnknownEffect(name.to_string()))
}

pub fn create(
    name: &str,
    semitones: Option<f32>,
    rate: u32,
) -> Result<Box<dyn VoiceEffect>, VoiceError> {
    let spec = spec(name)?;
    let semitones = semitones.unwrap_or(spec.default_semitones);
    if !semitones.is_finite() {
        return Err(VoiceError::InvalidSemitones(semitones));
    }
    Ok(match spec.name {
        "chipmunk" => Box::new(Chipmunk::new(semitones)?),
        "deep" => Box::new(Deep::new(semitones)?),
        "robot" => Box::new(Robot::new(semitones, rate)?),
        "whisper" => Box::new(Whisper::new()?),
        _ => Box::new(PhaseVocoderPitchShift::new(semitones)?),
    })
}

pub fn render(effect: &mut dyn VoiceEffect, input: &[f32]) -> Vec<f32> {
    let latency = effect.latency();
    let mut out = effect.process_block(input);
    out.extend(effect.process_block(&vec![0.0; latency]));
    out.drain(..latency.min(out.len()));
    out.truncate(input.len());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_default_is_inside_its_own_range_and_builds() {
        for spec in CATALOG {
            assert!(spec.default_semitones >= spec.min_semitones);
            assert!(spec.default_semitones <= spec.max_semitones);
            assert!(create(spec.name, None, 48000).is_ok(), "{}", spec.name);
        }
    }

    #[test]
    fn unknown_effect_names_are_rejected_instead_of_falling_back() {
        assert!(matches!(
            create("reverb", None, 48000),
            Err(VoiceError::UnknownEffect(_))
        ));
        assert!(matches!(
            create("pitch", Some(f32::NAN), 48000),
            Err(VoiceError::InvalidSemitones(_))
        ));
    }

    #[test]
    fn render_returns_exactly_the_input_length_for_every_effect() {
        let input: Vec<f32> = (0..12345).map(|i| (i as f32 * 0.03).sin()).collect();
        for spec in CATALOG {
            let mut effect = create(spec.name, None, 48000).unwrap();
            assert_eq!(
                render(effect.as_mut(), &input).len(),
                input.len(),
                "{}",
                spec.name
            );
        }
    }
}
