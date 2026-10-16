use crate::error::AudioError;
use std::f32::consts::PI;

pub const MIN_BPM: f32 = 40.0;
pub const MAX_BPM: f32 = 300.0;

struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

#[derive(Clone, Copy)]
enum Drum {
    Kick,
    Snare,
    HiHat,
}

fn render(drum: Drum, sample_rate: u32, noise: &mut Noise) -> Vec<f32> {
    let rate = sample_rate as f32;
    let (seconds, decay) = match drum {
        Drum::Kick => (0.3, 0.07),
        Drum::Snare => (0.25, 0.07),
        Drum::HiHat => (0.08, 0.02),
    };
    let len = (seconds * rate) as usize;
    let mut phase = 0.0f32;
    let mut previous = 0.0f32;
    (0..len)
        .map(|i| {
            let t = i as f32 / rate;
            let env = (-t / decay).exp();
            match drum {
                Drum::Kick => {
                    let freq = 50.0 + 90.0 * (-t / 0.04).exp();
                    phase += 2.0 * PI * freq / rate;
                    let click = 0.5 * (-t / 0.004).exp() * noise.next();
                    0.9 * env * phase.sin() + click
                }
                Drum::Snare => {
                    let tone = (2.0 * PI * 190.0 * t).sin();
                    0.35 * env * (0.6 * noise.next() + 0.4 * tone)
                }
                Drum::HiHat => {
                    let n = noise.next();
                    let high = n - previous;
                    previous = n;
                    0.02 * env * high
                }
            }
        })
        .collect()
}

fn mix_at(out: &mut [f32], sound: &[f32], at: usize) {
    for (o, s) in out.iter_mut().skip(at).zip(sound) {
        *o += s;
    }
}

pub fn drum_loop(bpm: f32, seconds: f32, sample_rate: u32) -> Result<Vec<f32>, AudioError> {
    if !(MIN_BPM..=MAX_BPM).contains(&bpm) {
        return Err(AudioError::InvalidParameter(format!(
            "bpm must be between {MIN_BPM} and {MAX_BPM}"
        )));
    }
    let mut noise = Noise(0x9E37_79B9);
    let kick = render(Drum::Kick, sample_rate, &mut noise);
    let snare = render(Drum::Snare, sample_rate, &mut noise);
    let hihat = render(Drum::HiHat, sample_rate, &mut noise);
    let len = (seconds * sample_rate as f32) as usize;
    let mut out = vec![0.0f32; len];
    let eighth = 30.0 / bpm * sample_rate as f32;
    let mut step = 0usize;
    loop {
        let at = (step as f32 * eighth).round() as usize;
        if at >= len {
            break;
        }
        mix_at(&mut out, &hihat, at);
        match step % 8 {
            0 | 4 => mix_at(&mut out, &kick, at),
            2 | 6 => mix_at(&mut out, &snare, at),
            _ => {}
        }
        step += 1;
    }
    let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    if peak > 0.95 {
        out.iter_mut().for_each(|v| *v *= 0.95 / peak);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_has_requested_duration_and_is_not_clipped() {
        let out = drum_loop(120.0, 4.0, 44100).unwrap();
        assert_eq!(out.len(), 4 * 44100);
        assert!(out.iter().all(|v| v.abs() <= 0.95 + 1e-6));
    }

    #[test]
    fn energy_starts_on_the_first_beat() {
        let out = drum_loop(120.0, 2.0, 44100).unwrap();
        let first: f32 = out[..2000].iter().map(|v| v * v).sum();
        let gap: f32 = out[18000..20000].iter().map(|v| v * v).sum();
        assert!(first > 10.0 * gap);
    }

    #[test]
    fn out_of_range_bpm_is_rejected() {
        assert!(drum_loop(10.0, 4.0, 44100).is_err());
        assert!(drum_loop(400.0, 4.0, 44100).is_err());
    }
}
