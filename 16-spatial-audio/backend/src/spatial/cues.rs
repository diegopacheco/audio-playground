use std::f64::consts::TAU;

use serde::Serialize;

use crate::dsp::shelf;
use crate::trajectory::Position;

pub const SPEED_OF_SOUND: f64 = 343.0;
pub const HEAD_RADIUS: f64 = 0.0875;
pub const SHADOW_CORNER: f64 = 2.0 * SPEED_OF_SOUND / HEAD_RADIUS;
const ALPHA_MIN: f64 = 0.1;
const THETA_MIN: f64 = 150.0;
const REFERENCE_DISTANCE: f64 = 1.0;
const ILD_PROBE_HZ: f64 = 4_000.0;

pub fn lateral_angle(pos: &Position) -> f64 {
    pos.direction().0.clamp(-1.0, 1.0).asin()
}

pub fn woodworth_itd(pos: &Position) -> f64 {
    let theta = lateral_angle(pos);
    HEAD_RADIUS / SPEED_OF_SOUND * (theta + theta.sin())
}

pub fn ear_angles(pos: &Position) -> (f64, f64) {
    let x = pos.direction().0.clamp(-1.0, 1.0);
    ((-x).acos().to_degrees(), x.acos().to_degrees())
}

pub fn shadow_gain(ear_angle_deg: f64) -> f64 {
    (1.0 + ALPHA_MIN / 2.0)
        + (1.0 - ALPHA_MIN / 2.0) * (ear_angle_deg / THETA_MIN * 180.0).to_radians().cos()
}

pub fn distance_gain(distance: f64) -> f64 {
    REFERENCE_DISTANCE / distance.max(0.3)
}

pub fn ild_db(pos: &Position, freq: f64) -> f64 {
    let (left, right) = ear_angles(pos);
    let omega = TAU * freq;
    let l = shelf::magnitude(shadow_gain(left), SHADOW_CORNER, omega);
    let r = shelf::magnitude(shadow_gain(right), SHADOW_CORNER, omega);
    20.0 * (r / l).log10()
}

#[derive(Serialize, Clone, Copy, Debug)]
pub struct CueReport {
    pub azimuth: f64,
    pub elevation: f64,
    pub distance: f64,
    pub itd_ms: f64,
    pub ild_db: f64,
    pub ild_probe_hz: f64,
}

pub fn report(pos: &Position) -> CueReport {
    CueReport {
        azimuth: pos.azimuth,
        elevation: pos.elevation,
        distance: pos.distance,
        itd_ms: woodworth_itd(pos) * 1_000.0,
        ild_db: ild_db(pos, ILD_PROBE_HZ),
        ild_probe_hz: ILD_PROBE_HZ,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(azimuth: f64) -> Position {
        Position {
            azimuth,
            elevation: 0.0,
            distance: 1.0,
        }
    }

    #[test]
    fn woodworth_itd_peaks_around_two_thirds_of_a_millisecond_at_the_side() {
        let itd = woodworth_itd(&at(90.0)) * 1_000.0;
        assert!((0.6..0.7).contains(&itd), "itd {itd}");
        assert!(woodworth_itd(&at(0.0)).abs() < 1e-12);
        assert!((woodworth_itd(&at(-90.0)) + woodworth_itd(&at(90.0))).abs() < 1e-12);
    }

    #[test]
    fn front_and_back_share_the_same_itd_which_is_the_cone_of_confusion() {
        assert!((woodworth_itd(&at(30.0)) - woodworth_itd(&at(150.0))).abs() < 1e-12);
    }

    #[test]
    fn near_ear_is_boosted_and_far_ear_shadowed() {
        let (left, right) = ear_angles(&at(90.0));
        assert!(shadow_gain(right) > 1.5);
        assert!(shadow_gain(left) < 0.5);
        assert!(ild_db(&at(90.0), 4_000.0) > 10.0);
        assert!(ild_db(&at(0.0), 4_000.0).abs() < 1e-9);
    }

    #[test]
    fn distance_gain_falls_monotonically_with_distance() {
        let gains: Vec<f64> = [0.5, 1.0, 2.0, 4.0, 8.0, 16.0]
            .iter()
            .map(|d| distance_gain(*d))
            .collect();
        assert!(gains.windows(2).all(|w| w[1] < w[0]));
        assert!((distance_gain(2.0) / distance_gain(4.0) - 2.0).abs() < 1e-12);
    }
}
