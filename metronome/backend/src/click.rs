use crate::error::AppError;
use std::f32::consts::TAU;

pub const SAMPLE_RATE: u32 = 44_100;
const MAX_SECONDS: f64 = 300.0;

pub trait Click {
    fn render(&self, accent: bool, sample_rate: u32) -> Vec<f32>;
}

pub struct SineClick {
    pub normal_hz: f32,
    pub accent_hz: f32,
    pub duration_ms: f32,
}

impl Default for SineClick {
    fn default() -> Self {
        SineClick {
            normal_hz: 1_000.0,
            accent_hz: 1_600.0,
            duration_ms: 30.0,
        }
    }
}

impl Click for SineClick {
    fn render(&self, accent: bool, sample_rate: u32) -> Vec<f32> {
        let hz = if accent {
            self.accent_hz
        } else {
            self.normal_hz
        };
        let gain = if accent { 0.9 } else { 0.7 };
        let sr = sample_rate as f32;
        let len = (self.duration_ms / 1_000.0 * sr) as usize;
        let decay = 5.0 / len.max(1) as f32;
        (0..len)
            .map(|i| {
                let t = i as f32 / sr;
                gain * (-(i as f32) * decay).exp() * (TAU * hz * t).cos()
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TrackSpec {
    pub bpm: f64,
    pub beats_per_bar: u32,
    pub bars: u32,
    pub accent: bool,
}

impl TrackSpec {
    pub fn validate(&self) -> Result<(), AppError> {
        if !(20.0..=300.0).contains(&self.bpm) {
            return Err(AppError::InvalidBpm(self.bpm));
        }
        if !(1..=16).contains(&self.beats_per_bar) {
            return Err(AppError::InvalidBeatsPerBar(self.beats_per_bar));
        }
        if !(1..=64).contains(&self.bars) {
            return Err(AppError::InvalidBars(self.bars));
        }
        let seconds = self.total_beats() as f64 * 60.0 / self.bpm;
        if seconds > MAX_SECONDS {
            return Err(AppError::TrackTooLong(seconds));
        }
        Ok(())
    }

    pub fn total_beats(&self) -> usize {
        (self.beats_per_bar * self.bars) as usize
    }
}

pub fn beat_interval_samples(bpm: f64, sample_rate: u32) -> usize {
    (60.0 / bpm * sample_rate as f64).round() as usize
}

pub fn render_track<C: Click>(
    spec: &TrackSpec,
    click: &C,
    sample_rate: u32,
) -> Result<Vec<f32>, AppError> {
    spec.validate()?;
    let interval = beat_interval_samples(spec.bpm, sample_rate);
    let beats = spec.total_beats();
    let normal = click.render(false, sample_rate);
    let accented = click.render(true, sample_rate);
    let mut samples = vec![0.0f32; interval * beats];
    for beat in 0..beats {
        let onset = beat * interval;
        let is_accent = spec.accent && beat % spec.beats_per_bar as usize == 0;
        let sound = if is_accent { &accented } else { &normal };
        let end = (onset + sound.len()).min(samples.len());
        samples[onset..end].copy_from_slice(&sound[..end - onset]);
    }
    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(bpm: f64) -> TrackSpec {
        TrackSpec {
            bpm,
            beats_per_bar: 4,
            bars: 4,
            accent: true,
        }
    }

    fn detect_onsets(samples: &[f32]) -> Vec<usize> {
        let mut found = Vec::new();
        let mut silent_run = usize::MAX;
        for (i, s) in samples.iter().enumerate() {
            if s.abs() > 0.05 {
                if silent_run > 1_000 {
                    found.push(i);
                }
                silent_run = 0;
            } else {
                silent_run = silent_run.saturating_add(1);
            }
        }
        found
    }

    fn zero_crossings(samples: &[f32]) -> usize {
        samples
            .windows(2)
            .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
            .count()
    }

    #[test]
    fn onsets_land_exactly_on_rounded_sample_intervals_so_no_drift_accumulates() {
        for bpm in [60.0, 97.0, 120.0, 133.3, 240.0] {
            let track = render_track(&spec(bpm), &SineClick::default(), SAMPLE_RATE).unwrap();
            let interval = (60.0 / bpm * SAMPLE_RATE as f64).round() as usize;
            let expected: Vec<usize> = (0..16).map(|i| i * interval).collect();
            assert_eq!(
                detect_onsets(&track),
                expected,
                "audible onsets at bpm {bpm}"
            );
        }
    }

    #[test]
    fn interval_at_120_bpm_is_half_a_second_of_samples() {
        assert_eq!(beat_interval_samples(120.0, 44_100), 22_050);
        assert_eq!(beat_interval_samples(97.0, 44_100), 27_278);
    }

    #[test]
    fn first_beat_of_each_bar_is_higher_pitched_so_the_downbeat_is_audible() {
        let track = render_track(&spec(120.0), &SineClick::default(), SAMPLE_RATE).unwrap();
        let len = SineClick::default().render(false, SAMPLE_RATE).len();
        let at = |beat: usize| {
            let o = beat * 22_050;
            zero_crossings(&track[o..o + len])
        };
        assert!(at(0) > at(1));
        assert!(at(4) > at(3));
        assert_eq!(at(1), at(2));
        assert_eq!(at(0), at(4));
    }

    #[test]
    fn disabling_accent_makes_every_beat_identical() {
        let mut s = spec(120.0);
        s.accent = false;
        let track = render_track(&s, &SineClick::default(), SAMPLE_RATE).unwrap();
        let len = SineClick::default().render(false, SAMPLE_RATE).len();
        let a = &track[..len];
        let b = &track[22_050..22_050 + len];
        assert_eq!(a, b);
    }

    #[test]
    fn track_length_covers_every_beat_interval() {
        let track = render_track(&spec(120.0), &SineClick::default(), SAMPLE_RATE).unwrap();
        assert_eq!(track.len(), 16 * 22_050);
    }

    #[test]
    fn out_of_range_specs_are_rejected() {
        assert_eq!(spec(10.0).validate(), Err(AppError::InvalidBpm(10.0)));
        let mut s = spec(120.0);
        s.beats_per_bar = 0;
        assert_eq!(s.validate(), Err(AppError::InvalidBeatsPerBar(0)));
        let mut s = spec(20.0);
        s.beats_per_bar = 16;
        s.bars = 64;
        assert!(matches!(s.validate(), Err(AppError::TrackTooLong(_))));
    }
}
