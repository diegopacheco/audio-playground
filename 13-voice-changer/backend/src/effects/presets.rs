use crate::effects::{PhaseVocoderPitchShift, VoiceEffect};
use crate::error::VoiceError;

pub struct Chipmunk(PhaseVocoderPitchShift);

impl Chipmunk {
    pub fn new(semitones: f32) -> Result<Self, VoiceError> {
        Ok(Self(PhaseVocoderPitchShift::new(
            semitones.clamp(7.0, 12.0),
        )?))
    }
}

impl VoiceEffect for Chipmunk {
    fn process_block(&mut self, input: &[f32]) -> Vec<f32> {
        self.0.process_block(input)
    }

    fn latency(&self) -> usize {
        self.0.latency()
    }
}

pub struct Deep(PhaseVocoderPitchShift);

impl Deep {
    pub fn new(semitones: f32) -> Result<Self, VoiceError> {
        Ok(Self(PhaseVocoderPitchShift::new(
            semitones.clamp(-7.0, -5.0),
        )?))
    }
}

impl VoiceEffect for Deep {
    fn process_block(&mut self, input: &[f32]) -> Vec<f32> {
        self.0.process_block(input)
    }

    fn latency(&self) -> usize {
        self.0.latency()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::analysis::dominant_frequency;
    use crate::effects::render;
    use std::f32::consts::TAU;

    fn measure(effect: &mut dyn VoiceEffect) -> f32 {
        let input: Vec<f32> = (0..48000)
            .map(|i| 0.5 * (TAU * 440.0 * i as f32 / 48000.0).sin())
            .collect();
        let out = render(effect, &input);
        dominant_frequency(&out[12000..36000], 48000)
    }

    #[test]
    fn chipmunk_never_goes_below_a_fifth_up_even_when_asked_for_less() {
        let frequency = measure(&mut Chipmunk::new(2.0).unwrap());
        assert!((frequency - 659.26).abs() < 7.0, "got {frequency}");
    }

    #[test]
    fn deep_never_drops_more_than_seven_semitones() {
        let frequency = measure(&mut Deep::new(-20.0).unwrap());
        assert!((frequency - 293.66).abs() < 3.0, "got {frequency}");
    }
}
