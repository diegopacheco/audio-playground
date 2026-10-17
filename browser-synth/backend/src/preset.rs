use serde::{Deserialize, Serialize};

use crate::dsp::envelope::Adsr;
use crate::dsp::oscillator::Waveform;
use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub waveform: Waveform,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub cutoff: f32,
    pub resonance: f32,
    pub gain: f32,
}

fn check(name: &str, value: f32, min: f32, max: f32) -> Result<(), AppError> {
    if value.is_finite() && value >= min && value <= max {
        Ok(())
    } else {
        Err(AppError::InvalidPreset(format!(
            "{name} must be between {min} and {max}, got {value}"
        )))
    }
}

impl Preset {
    pub fn validate(&self) -> Result<(), AppError> {
        let name = self.name.trim();
        if name.is_empty() || name.len() > 40 {
            return Err(AppError::InvalidPreset(
                "name must have 1 to 40 characters".to_string(),
            ));
        }
        check("attack", self.attack, 0.0, 5.0)?;
        check("decay", self.decay, 0.0, 5.0)?;
        check("sustain", self.sustain, 0.0, 1.0)?;
        check("release", self.release, 0.0, 10.0)?;
        check("cutoff", self.cutoff, 20.0, 20_000.0)?;
        check("resonance", self.resonance, -10.0, 30.0)?;
        check("gain", self.gain, 0.0, 1.0)
    }

    pub fn adsr(&self) -> Adsr {
        Adsr {
            attack: self.attack,
            decay: self.decay,
            sustain: self.sustain,
            release: self.release,
        }
    }
}

pub fn built_in_presets() -> Vec<Preset> {
    let preset = |name: &str, waveform, a, d, s, r, cutoff, resonance, gain| Preset {
        name: name.to_string(),
        waveform,
        attack: a,
        decay: d,
        sustain: s,
        release: r,
        cutoff,
        resonance,
        gain,
    };
    vec![
        preset(
            "Soft Sine",
            Waveform::Sine,
            0.02,
            0.2,
            0.8,
            0.4,
            8_000.0,
            0.0,
            0.8,
        ),
        preset(
            "Acid Saw",
            Waveform::Sawtooth,
            0.005,
            0.25,
            0.3,
            0.15,
            900.0,
            18.0,
            0.6,
        ),
        preset(
            "Square Pluck",
            Waveform::Square,
            0.002,
            0.15,
            0.0,
            0.1,
            2_500.0,
            6.0,
            0.5,
        ),
        preset(
            "Warm Pad",
            Waveform::Triangle,
            0.8,
            0.5,
            0.7,
            1.5,
            1_800.0,
            3.0,
            0.8,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> Preset {
        built_in_presets().remove(0)
    }

    #[test]
    fn built_in_presets_are_all_valid() {
        assert!(built_in_presets().iter().all(|p| p.validate().is_ok()));
    }

    #[test]
    fn sustain_above_full_level_is_rejected() {
        let p = Preset {
            sustain: 1.5,
            ..valid()
        };
        assert!(matches!(p.validate(), Err(AppError::InvalidPreset(m)) if m.contains("sustain")));
    }

    #[test]
    fn negative_attack_is_rejected() {
        let p = Preset {
            attack: -0.1,
            ..valid()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn cutoff_outside_audible_range_is_rejected() {
        assert!(
            Preset {
                cutoff: 5.0,
                ..valid()
            }
            .validate()
            .is_err()
        );
        assert!(
            Preset {
                cutoff: 30_000.0,
                ..valid()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn non_finite_values_are_rejected() {
        assert!(
            Preset {
                resonance: f32::NAN,
                ..valid()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn blank_name_is_rejected() {
        assert!(
            Preset {
                name: "  ".to_string(),
                ..valid()
            }
            .validate()
            .is_err()
        );
    }
}
