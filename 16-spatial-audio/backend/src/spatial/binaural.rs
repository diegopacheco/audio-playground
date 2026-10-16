use crate::audio::{Mono, Stereo};
use crate::error::AppError;
use crate::spatial::Spatializer;
use crate::spatial::engine::{self, Bypass};
use crate::trajectory::Trajectory;

pub struct SimpleBinaural;

impl Spatializer for SimpleBinaural {
    fn render(&self, mono: &Mono, trajectory: &Trajectory) -> Result<Stereo, AppError> {
        engine::render_binaural(mono, trajectory, &mut Bypass)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SAMPLE_RATE;
    use crate::source::{self, SourceKind, TONE_HZ};
    use crate::spatial::analysis::{crossing_rate, right_lead_samples, rms};

    fn render_static(azimuth: f64, distance: f64) -> Stereo {
        let mono = source::generate(SourceKind::Noise, SAMPLE_RATE, 1.0);
        let trajectory = Trajectory::Static {
            azimuth,
            elevation: 0.0,
            distance,
        };
        SimpleBinaural.render(&mono, &trajectory).unwrap()
    }

    fn lead_ms(s: &Stereo) -> f64 {
        right_lead_samples(&s.left, &s.right, 60) / SAMPLE_RATE as f64 * 1_000.0
    }

    fn level_db(s: &Stereo) -> f64 {
        20.0 * (rms(&s.right) / rms(&s.left)).log10()
    }

    #[test]
    fn source_at_right_reaches_right_ear_first_by_woodworth_itd() {
        let s = render_static(90.0, 1.0);
        let lead = lead_ms(&s);
        assert!((0.6..=0.7).contains(&lead), "right lead {lead} ms");
    }

    #[test]
    fn source_at_right_is_louder_in_right_ear_from_head_shadow() {
        let s = render_static(90.0, 1.0);
        assert!(level_db(&s) > 3.0, "ild {} dB", level_db(&s));
    }

    #[test]
    fn source_in_front_is_identical_in_both_ears() {
        let s = render_static(0.0, 1.0);
        assert!(lead_ms(&s).abs() < 0.01);
        assert!(level_db(&s).abs() < 0.01);
    }

    #[test]
    fn source_at_left_mirrors_source_at_right() {
        let right = render_static(90.0, 1.0);
        let left = render_static(-90.0, 1.0);
        assert!((lead_ms(&left) + lead_ms(&right)).abs() < 0.01);
        assert!((level_db(&left) + level_db(&right)).abs() < 0.01);
        assert!(level_db(&left) < -3.0);
    }

    #[test]
    fn farther_sources_are_quieter_at_every_step() {
        let levels: Vec<f64> = [1.0, 2.0, 4.0, 8.0, 16.0]
            .iter()
            .map(|d| rms(&render_static(30.0, *d).left))
            .collect();
        assert!(levels.windows(2).all(|w| w[1] < w[0]), "{levels:?}");
    }

    #[test]
    fn flyby_raises_pitch_on_approach_and_lowers_it_on_recede() {
        let fs = SAMPLE_RATE as f64;
        let mono = source::generate(SourceKind::Tone, SAMPLE_RATE, 4.0);
        let trajectory = Trajectory::Flyby {
            speed: 20.0,
            closest: 2.0,
        };
        let s = SimpleBinaural.render(&mono, &trajectory).unwrap();
        let window = |from: f64, to: f64| &s.left[(from * fs) as usize..(to * fs) as usize];
        let approach = crossing_rate(window(0.4, 1.2), fs);
        let recede = crossing_rate(window(2.8, 3.6), fs);
        let expected_up = TONE_HZ * 343.0 / (343.0 - 20.0 * 0.99);
        let expected_down = TONE_HZ * 343.0 / (343.0 + 20.0 * 0.99);
        assert!(approach > TONE_HZ * 1.03, "approach {approach} Hz");
        assert!(recede < TONE_HZ * 0.97, "recede {recede} Hz");
        assert!((approach - expected_up).abs() < 10.0, "approach {approach}");
        assert!((recede - expected_down).abs() < 10.0, "recede {recede}");
    }

    #[test]
    fn flyby_is_loudest_when_closest() {
        let fs = SAMPLE_RATE as f64;
        let mono = source::generate(SourceKind::Noise, SAMPLE_RATE, 4.0);
        let trajectory = Trajectory::Flyby {
            speed: 20.0,
            closest: 2.0,
        };
        let s = SimpleBinaural.render(&mono, &trajectory).unwrap();
        let at = |t: f64| rms(&s.left[(t * fs) as usize..((t + 0.2) * fs) as usize]);
        assert!(at(0.5) < at(1.4));
        assert!(at(1.4) < at(1.9));
        assert!(at(2.6) < at(2.1));
        assert!(at(3.5) < at(2.6));
    }
}
