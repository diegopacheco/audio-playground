use crate::audio::{Mono, Stereo};
use crate::dsp::delay::FractionalDelay;
use crate::dsp::shelf::FirstOrderShelf;
use crate::error::AppError;
use crate::spatial::cues::{self, SHADOW_CORNER, SPEED_OF_SOUND};
use crate::trajectory::{Position, Trajectory};

pub trait EarStage {
    fn process(&mut self, pos: &Position, left: f64, right: f64) -> (f64, f64);
}

pub struct Bypass;

impl EarStage for Bypass {
    fn process(&mut self, _pos: &Position, left: f64, right: f64) -> (f64, f64) {
        (left, right)
    }
}

pub fn render_binaural(
    mono: &Mono,
    trajectory: &Trajectory,
    stage: &mut dyn EarStage,
) -> Result<Stereo, AppError> {
    trajectory.validate()?;
    let fs = mono.sample_rate as f64;
    let duration = mono.duration();
    let max_delay = (trajectory.max_distance(duration) / SPEED_OF_SOUND + 0.001) * fs;
    let mut line = FractionalDelay::new(max_delay.ceil() as usize + 4);
    let mut shadow_left = FirstOrderShelf::new(fs);
    let mut shadow_right = FirstOrderShelf::new(fs);
    let mut left = Vec::with_capacity(mono.samples.len());
    let mut right = Vec::with_capacity(mono.samples.len());
    for (n, x) in mono.samples.iter().enumerate() {
        let pos = trajectory.position(n as f64 / fs, duration);
        line.push(x * cues::distance_gain(pos.distance) as f32);
        let base = pos.distance / SPEED_OF_SOUND * fs;
        let itd = cues::woodworth_itd(&pos) * fs;
        let l = line.tap(base + itd.max(0.0)) as f64;
        let r = line.tap(base + (-itd).max(0.0)) as f64;
        let (angle_left, angle_right) = cues::ear_angles(&pos);
        shadow_left.set(cues::shadow_gain(angle_left), SHADOW_CORNER);
        shadow_right.set(cues::shadow_gain(angle_right), SHADOW_CORNER);
        let (l, r) = stage.process(&pos, shadow_left.process(l), shadow_right.process(r));
        left.push(l as f32);
        right.push(r as f32);
    }
    Ok(Stereo {
        sample_rate: mono.sample_rate,
        left,
        right,
    })
}
