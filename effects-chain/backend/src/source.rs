use crate::wav::Audio;

pub const SAMPLE_RATE: u32 = 44100;
const NOTES: [f32; 8] = [
    261.63, 329.63, 392.00, 523.25, 392.00, 329.63, 293.66, 261.63,
];
const NOTE_GAP_SECONDS: f32 = 0.35;
const RING_SECONDS: f32 = 1.2;
const DAMPING: f32 = 0.996;

struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
}

fn pluck(freq: f32, length: usize, noise: &mut Noise) -> Vec<f32> {
    let period = ((SAMPLE_RATE as f32 / freq).round() as usize).max(2);
    let mut ring: Vec<f32> = (0..period).map(|_| noise.next()).collect();
    let mean = ring.iter().sum::<f32>() / period as f32;
    ring.iter_mut().for_each(|s| *s -= mean);
    let mut out = Vec::with_capacity(length);
    for i in 0..length {
        let a = ring[i % period];
        let b = ring[(i + 1) % period];
        out.push(a);
        ring[i % period] = DAMPING * 0.5 * (a + b);
    }
    out
}

pub fn plucked_tones() -> Audio {
    let gap = (NOTE_GAP_SECONDS * SAMPLE_RATE as f32) as usize;
    let ring = (RING_SECONDS * SAMPLE_RATE as f32) as usize;
    let mut samples = vec![0.0f32; gap * (NOTES.len() - 1) + ring];
    let mut noise = Noise(7);
    for (n, freq) in NOTES.iter().enumerate() {
        for (i, s) in pluck(*freq, ring, &mut noise).into_iter().enumerate() {
            samples[n * gap + i] += s;
        }
    }
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 0.0 {
        samples.iter_mut().for_each(|s| *s *= 0.8 / peak);
    }
    Audio {
        sample_rate: SAMPLE_RATE,
        samples,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_clip_is_normalized_and_not_silent() {
        let audio = plucked_tones();
        let peak = audio.samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((peak - 0.8).abs() < 1e-4);
        assert!(audio.samples.len() > SAMPLE_RATE as usize * 3);
    }

    #[test]
    fn source_clip_has_no_dc_offset_that_reverb_combs_would_amplify() {
        let audio = plucked_tones();
        let mean = audio.samples.iter().sum::<f32>() / audio.samples.len() as f32;
        assert!(mean.abs() < 1e-3, "dc offset {mean}");
    }

    #[test]
    fn plucked_string_decays_over_time() {
        let mut noise = Noise(1);
        let tone = pluck(440.0, SAMPLE_RATE as usize, &mut noise);
        let head: f32 = tone[..2000].iter().map(|s| s * s).sum();
        let tail: f32 = tone[tone.len() - 2000..].iter().map(|s| s * s).sum();
        assert!(tail < head * 0.1);
    }
}
