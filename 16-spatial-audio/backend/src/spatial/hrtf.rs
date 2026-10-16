use std::f64::consts::TAU;

use crate::audio::{Mono, Stereo};
use crate::dsp::notch::NotchFilter;
use crate::dsp::shelf::FirstOrderShelf;
use crate::error::AppError;
use crate::spatial::Spatializer;
use crate::spatial::engine::{self, EarStage};
use crate::trajectory::{Position, Trajectory};

const NOTCH_BASE_HZ: f64 = 8_000.0;
const NOTCH_HZ_PER_DEGREE: f64 = 50.0;
const NOTCH_MIN_HZ: f64 = 4_000.0;
const NOTCH_MAX_HZ: f64 = 13_000.0;
const NOTCH_Q: f64 = 2.0;
const NOTCH_DEPTH_DB: f64 = 12.0;
const BACK_TILT_DB: f64 = 5.0;
const TILT_CORNER_HZ: f64 = 2_000.0;

pub fn pinna_notch_hz(elevation: f64, fs: f64) -> f64 {
    (NOTCH_BASE_HZ + NOTCH_HZ_PER_DEGREE * elevation)
        .clamp(NOTCH_MIN_HZ, NOTCH_MAX_HZ.min(0.45 * fs))
}

pub fn back_tilt_gain(pos: &Position) -> f64 {
    let backness = (-pos.direction().1).max(0.0);
    10f64.powf(-BACK_TILT_DB * backness / 20.0)
}

struct PinnaStage {
    fs: f64,
    notches: [NotchFilter; 2],
    tilts: [FirstOrderShelf; 2],
}

impl PinnaStage {
    fn new(fs: f64) -> Self {
        PinnaStage {
            fs,
            notches: [NotchFilter::new(fs), NotchFilter::new(fs)],
            tilts: [FirstOrderShelf::new(fs), FirstOrderShelf::new(fs)],
        }
    }
}

impl EarStage for PinnaStage {
    fn process(&mut self, pos: &Position, left: f64, right: f64) -> (f64, f64) {
        let notch_hz = pinna_notch_hz(pos.elevation, self.fs);
        let tilt = back_tilt_gain(pos);
        let mut out = [left, right];
        for (i, sample) in out.iter_mut().enumerate() {
            self.notches[i].set(notch_hz, NOTCH_Q, NOTCH_DEPTH_DB);
            self.tilts[i].set(tilt, TAU * TILT_CORNER_HZ);
            *sample = self.tilts[i].process(self.notches[i].process(*sample));
        }
        (out[0], out[1])
    }
}

pub struct ParametricHrtf;

impl Spatializer for ParametricHrtf {
    fn render(&self, mono: &Mono, trajectory: &Trajectory) -> Result<Stereo, AppError> {
        let mut stage = PinnaStage::new(mono.sample_rate as f64);
        engine::render_binaural(mono, trajectory, &mut stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SAMPLE_RATE;
    use crate::source::{self, SourceKind};
    use crate::spatial::analysis::{high_band_rms, right_lead_samples, rms};
    use crate::spatial::binaural::SimpleBinaural;

    fn render(spatializer: &dyn Spatializer, azimuth: f64, elevation: f64) -> Stereo {
        let mono = source::generate(SourceKind::Noise, SAMPLE_RATE, 1.0);
        let trajectory = Trajectory::Static {
            azimuth,
            elevation,
            distance: 1.0,
        };
        spatializer.render(&mono, &trajectory).unwrap()
    }

    fn brightness(s: &Stereo) -> f64 {
        high_band_rms(&s.left) / rms(&s.left)
    }

    #[test]
    fn notch_climbs_with_elevation_and_stays_below_nyquist() {
        let fs = SAMPLE_RATE as f64;
        let below = pinna_notch_hz(-40.0, fs);
        let level = pinna_notch_hz(0.0, fs);
        let above = pinna_notch_hz(60.0, fs);
        assert!(below < level && level < above);
        assert!(pinna_notch_hz(90.0, 16_000.0) < 8_000.0);
    }

    #[test]
    fn hrtf_darkens_sources_behind_so_front_and_back_differ() {
        let front = brightness(&render(&ParametricHrtf, 0.0, 0.0));
        let back = brightness(&render(&ParametricHrtf, 180.0, 0.0));
        assert!(back < front * 0.85, "front {front} back {back}");
    }

    #[test]
    fn simple_binaural_cannot_tell_front_from_back() {
        let front = brightness(&render(&SimpleBinaural, 0.0, 0.0));
        let back = brightness(&render(&SimpleBinaural, 180.0, 0.0));
        assert!((back / front - 1.0).abs() < 0.01);
    }

    #[test]
    fn hrtf_keeps_the_interaural_time_cue() {
        let s = render(&ParametricHrtf, 90.0, 0.0);
        let lead = right_lead_samples(&s.left, &s.right, 60) / SAMPLE_RATE as f64 * 1_000.0;
        assert!((0.6..=0.7).contains(&lead), "lead {lead} ms");
        assert!(rms(&s.right) > rms(&s.left));
    }
}
