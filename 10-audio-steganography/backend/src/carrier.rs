use std::f64::consts::PI;

use crate::wav::Audio;

pub const SAMPLE_RATE: u32 = 44_100;
const SECONDS: f64 = 6.0;
const CHORDS: [[f64; 3]; 4] = [
    [261.63, 329.63, 392.00],
    [220.00, 261.63, 329.63],
    [174.61, 220.00, 261.63],
    [196.00, 246.94, 293.66],
];

fn envelope(t: f64, length: f64) -> f64 {
    let attack = (t / 0.05).min(1.0);
    let release = ((length - t) / 0.2).clamp(0.0, 1.0);
    attack * release * (-1.2 * t).exp().max(0.35)
}

fn noise(state: &mut u32) -> f64 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*state >> 8) as f64 / (1u32 << 24) as f64 * 2.0 - 1.0
}

pub fn generate() -> Audio {
    let sr = SAMPLE_RATE as f64;
    let total = (sr * SECONDS) as usize;
    let chord_len = SECONDS / CHORDS.len() as f64;
    let mut seed = 0x5EED_u32;
    let raw: Vec<f64> = (0..total)
        .map(|i| {
            let t = i as f64 / sr;
            let index = ((t / chord_len) as usize).min(CHORDS.len() - 1);
            let local = t - index as f64 * chord_len;
            let tone: f64 = CHORDS[index]
                .iter()
                .map(|f| (2.0 * PI * f * t).sin() + 0.3 * (4.0 * PI * f * t).sin())
                .sum();
            tone * envelope(local, chord_len) + 0.01 * noise(&mut seed)
        })
        .collect();
    let peak = raw.iter().fold(0.0f64, |m, x| m.max(x.abs())).max(1e-9);
    let gain = 0.7 * i16::MAX as f64 / peak;
    Audio {
        sample_rate: SAMPLE_RATE,
        samples: raw.iter().map(|x| (x * gain).round() as i16).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrier_leaves_headroom_so_ultrasonic_mixing_does_not_clip() {
        let audio = generate();
        let peak = audio
            .samples
            .iter()
            .map(|s| (*s as i32).abs())
            .max()
            .unwrap();
        assert!(peak <= (0.71 * i16::MAX as f64) as i32);
        assert!(peak > 10_000);
        assert_eq!(audio.samples.len(), 6 * SAMPLE_RATE as usize);
    }
}
