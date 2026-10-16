use crate::error::AppError;
use std::f64::consts::TAU;
use std::str::FromStr;

pub trait Oscillator {
    fn sample(&self, phase: f64) -> f64;
}

pub struct Sine;
pub struct Square;
pub struct Saw;
pub struct Triangle;

impl Oscillator for Sine {
    fn sample(&self, phase: f64) -> f64 {
        (TAU * phase).sin()
    }
}

impl Oscillator for Square {
    fn sample(&self, phase: f64) -> f64 {
        if phase < 0.5 { 1.0 } else { -1.0 }
    }
}

impl Oscillator for Saw {
    fn sample(&self, phase: f64) -> f64 {
        2.0 * phase - 1.0
    }
}

impl Oscillator for Triangle {
    fn sample(&self, phase: f64) -> f64 {
        1.0 - 4.0 * ((phase + 0.25).fract() - 0.5).abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

impl Waveform {
    pub const ALL: [Waveform; 4] = [
        Waveform::Sine,
        Waveform::Square,
        Waveform::Saw,
        Waveform::Triangle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Waveform::Sine => "sine",
            Waveform::Square => "square",
            Waveform::Saw => "saw",
            Waveform::Triangle => "triangle",
        }
    }

    pub fn oscillator(self) -> Box<dyn Oscillator> {
        match self {
            Waveform::Sine => Box::new(Sine),
            Waveform::Square => Box::new(Square),
            Waveform::Saw => Box::new(Saw),
            Waveform::Triangle => Box::new(Triangle),
        }
    }
}

impl FromStr for Waveform {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Waveform::ALL
            .into_iter()
            .find(|w| w.name() == s)
            .ok_or_else(|| AppError::UnknownWaveform(s.to_string()))
    }
}

pub fn render(
    osc: &dyn Oscillator,
    frequency: f64,
    amplitude: f64,
    sample_rate: u32,
    count: usize,
) -> Vec<i16> {
    let step = frequency / f64::from(sample_rate);
    (0..count)
        .map(|i| {
            let phase = (i as f64 * step).fract();
            let value = osc.sample(phase) * amplitude * f64::from(i16::MAX);
            value.round() as i16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn one_second(w: Waveform, hz: f64) -> Vec<i16> {
        render(w.oscillator().as_ref(), hz, 1.0, 44100, 44100)
    }

    fn zero_crossings(samples: &[i16]) -> usize {
        samples
            .windows(2)
            .filter(|p| (p[0] < 0 && p[1] >= 0) || (p[0] >= 0 && p[1] < 0))
            .count()
    }

    #[test]
    fn sine_at_440hz_crosses_zero_twice_per_cycle_so_about_880_per_second() {
        let crossings = zero_crossings(&one_second(Waveform::Sine, 440.0));
        assert!((878..=882).contains(&crossings), "got {crossings}");
    }

    #[test]
    fn doubling_frequency_doubles_zero_crossings_because_pitch_is_cycles_per_second() {
        let low = zero_crossings(&one_second(Waveform::Sine, 220.0));
        let high = zero_crossings(&one_second(Waveform::Sine, 440.0));
        assert!((high as i64 - 2 * low as i64).abs() <= 2, "{low} vs {high}");
    }

    #[test]
    fn square_wave_only_has_two_levels_because_it_jumps_between_full_high_and_full_low() {
        let levels: BTreeSet<i16> = one_second(Waveform::Square, 440.0).into_iter().collect();
        assert_eq!(levels, BTreeSet::from([-i16::MAX, i16::MAX]));
    }

    #[test]
    fn square_wave_spends_half_the_time_high_so_it_has_no_dc_offset() {
        let samples = one_second(Waveform::Square, 100.0);
        let high = samples.iter().filter(|s| **s > 0).count();
        assert!((22000..=22100).contains(&high), "got {high}");
    }

    #[test]
    fn saw_ramps_up_inside_each_cycle_and_drops_only_once_per_cycle() {
        let samples = one_second(Waveform::Saw, 100.0);
        let drops = samples.windows(2).filter(|p| p[1] < p[0]).count();
        assert!((99..=101).contains(&drops), "got {drops}");
    }

    #[test]
    fn triangle_changes_direction_twice_per_cycle_because_it_rises_then_falls() {
        let samples = one_second(Waveform::Triangle, 100.0);
        let slopes: Vec<i32> = samples
            .windows(2)
            .map(|p| i32::from(p[1]) - i32::from(p[0]))
            .filter(|d| *d != 0)
            .map(i32::signum)
            .collect();
        let turns = slopes.windows(2).filter(|p| p[0] != p[1]).count();
        assert!((198..=201).contains(&turns), "got {turns}");
    }

    #[test]
    fn triangle_starts_at_zero_so_playback_has_no_initial_click() {
        assert_eq!(Triangle.sample(0.0), 0.0);
        assert_eq!(Triangle.sample(0.25), 1.0);
        assert_eq!(Triangle.sample(0.75), -1.0);
    }

    #[test]
    fn amplitude_scales_peak_so_half_amplitude_never_exceeds_half_full_scale() {
        for w in Waveform::ALL {
            let samples = render(w.oscillator().as_ref(), 440.0, 0.5, 44100, 44100);
            let peak = samples
                .iter()
                .map(|s| i32::from(*s).abs())
                .max()
                .unwrap_or(0);
            assert!((16000..=16384).contains(&peak), "{} peak {peak}", w.name());
        }
    }

    #[test]
    fn waveform_names_round_trip_so_api_strings_map_to_the_right_oscillator() {
        for w in Waveform::ALL {
            assert_eq!(w.name().parse::<Waveform>().ok(), Some(w));
        }
        assert!("noise".parse::<Waveform>().is_err());
    }
}
