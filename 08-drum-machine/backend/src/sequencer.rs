use crate::error::AppError;
use crate::instruments;
use crate::pattern::{Pattern, STEPS};

pub const MAX_SWING: f32 = 0.5;

pub struct RenderSettings {
    pub bpm: f32,
    pub bars: u32,
    pub swing: f32,
}

impl RenderSettings {
    pub fn validate(&self) -> Result<(), AppError> {
        if !(40.0..=300.0).contains(&self.bpm) {
            return Err(AppError::InvalidRender("bpm must be 40..300".into()));
        }
        if !(1..=16).contains(&self.bars) {
            return Err(AppError::InvalidRender("bars must be 1..16".into()));
        }
        if !(0.0..=MAX_SWING).contains(&self.swing) {
            return Err(AppError::InvalidRender("swing must be 0..0.5".into()));
        }
        Ok(())
    }
}

pub fn step_duration(bpm: f32) -> f32 {
    60.0 / bpm / 4.0
}

pub fn step_time(step: usize, bpm: f32, swing: f32) -> f32 {
    let dur = step_duration(bpm);
    let delay = if step % 2 == 1 { swing * dur } else { 0.0 };
    step as f32 * dur + delay
}

pub fn render(
    pattern: &Pattern,
    settings: &RenderSettings,
    sample_rate: u32,
) -> Result<Vec<f32>, AppError> {
    pattern.validate()?;
    settings.validate()?;
    let total_steps = STEPS * settings.bars as usize;
    let length = step_duration(settings.bpm) * total_steps as f32;
    let mut out = vec![0.0f32; (length * sample_rate as f32).ceil() as usize];
    for inst in instruments::all() {
        let Some(steps) = pattern.tracks.get(inst.name()) else {
            continue;
        };
        let hit = inst.render(sample_rate);
        for step in 0..total_steps {
            if !steps[step % STEPS] {
                continue;
            }
            let start = (step_time(step, settings.bpm, settings.swing) * sample_rate as f32).round()
                as usize;
            for (o, h) in out.iter_mut().skip(start).zip(hit.iter()) {
                *o += h * 0.6;
            }
        }
    }
    for s in out.iter_mut() {
        *s = s.tanh();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(bpm: f32, bars: u32, swing: f32) -> RenderSettings {
        RenderSettings { bpm, bars, swing }
    }

    fn first_onset(s: &[f32], from: usize) -> usize {
        from + s[from..].iter().position(|v| v.abs() > 1e-4).unwrap()
    }

    #[test]
    fn a_step_is_a_sixteenth_note() {
        assert!((step_duration(120.0) - 0.125).abs() < 1e-6);
        assert!((step_duration(60.0) - 0.25).abs() < 1e-6);
        assert!((step_duration(150.0) - 60.0 / 150.0 / 4.0).abs() < 1e-6);
    }

    #[test]
    fn without_swing_steps_are_evenly_spaced() {
        for step in 0..16 {
            assert!((step_time(step, 120.0, 0.0) - step as f32 * 0.125).abs() < 1e-6);
        }
    }

    #[test]
    fn swing_must_delay_only_odd_steps() {
        let dur = step_duration(120.0);
        for step in 0..16 {
            let straight = step as f32 * dur;
            let swung = step_time(step, 120.0, 0.3);
            if step % 2 == 0 {
                assert!((swung - straight).abs() < 1e-6, "even step {step} moved");
            } else {
                assert!(
                    (swung - straight - 0.3 * dur).abs() < 1e-6,
                    "odd step {step} not delayed"
                );
            }
        }
    }

    #[test]
    fn rendered_length_must_equal_bars_times_sixteen_steps() {
        let p = Pattern::from_strings(&[("kick", "x...............")]);
        let out = render(&p, &settings(120.0, 2, 0.0), 8000).unwrap();
        assert_eq!(out.len(), (2.0 * 16.0 * 0.125 * 8000.0) as usize);
    }

    #[test]
    fn hits_must_land_on_their_step_sample_and_swing_must_push_odd_hits_late() {
        let sr = 8000;
        let p = Pattern::from_strings(&[("kick", ".x..............")]);
        let straight = render(&p, &settings(120.0, 1, 0.0), sr).unwrap();
        let swung = render(&p, &settings(120.0, 1, 0.4), sr).unwrap();
        assert_eq!(first_onset(&straight, 0), 1000);
        assert_eq!(first_onset(&swung, 0), 1400);
    }

    #[test]
    fn empty_pattern_renders_silence() {
        let p = Pattern::from_strings(&[("kick", "................")]);
        let out = render(&p, &settings(120.0, 1, 0.0), 8000).unwrap();
        assert!(out.iter().all(|v| *v == 0.0));
    }

    #[test]
    fn out_of_range_settings_must_be_rejected() {
        let p = Pattern::from_strings(&[("kick", "x...............")]);
        assert!(render(&p, &settings(10.0, 1, 0.0), 8000).is_err());
        assert!(render(&p, &settings(120.0, 0, 0.0), 8000).is_err());
        assert!(render(&p, &settings(120.0, 1, 0.9), 8000).is_err());
    }

    #[test]
    fn mix_must_never_exceed_full_scale() {
        let p = Pattern::from_strings(&[
            ("kick", "xxxxxxxxxxxxxxxx"),
            ("snare", "xxxxxxxxxxxxxxxx"),
            ("hihat", "xxxxxxxxxxxxxxxx"),
            ("clap", "xxxxxxxxxxxxxxxx"),
        ]);
        let out = render(&p, &settings(200.0, 1, 0.0), 8000).unwrap();
        assert!(out.iter().all(|v| v.abs() <= 1.0));
    }
}
