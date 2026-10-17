use super::sample_at;

pub trait BeatTracker: Send + Sync {
    fn track(&self, envelope: &[f32], lag: f32) -> Vec<f32>;
}

pub struct PhaseAlignedTracker {
    pub snap_radius: usize,
}

const PHASE_STEP: f32 = 0.25;

fn grid(len: usize, phase: f32, lag: f32) -> impl Iterator<Item = f32> {
    (0..)
        .map(move |k| phase + k as f32 * lag)
        .take_while(move |p| *p < len as f32)
}

fn best_phase(envelope: &[f32], lag: f32) -> f32 {
    let steps = (lag / PHASE_STEP).ceil() as usize;
    (0..steps)
        .map(|i| i as f32 * PHASE_STEP)
        .map(|phase| {
            let score: f32 = grid(envelope.len(), phase, lag)
                .map(|p| sample_at(envelope, p))
                .sum();
            (phase, score)
        })
        .fold((0.0, f32::MIN), |b, c| if c.1 > b.1 { c } else { b })
        .0
}

fn snap(envelope: &[f32], position: f32, radius: usize) -> f32 {
    let last = envelope.len().saturating_sub(1);
    let center = (position.round() as usize).min(last);
    let lo = center.saturating_sub(radius);
    let hi = (center + radius).min(last);
    (lo..=hi)
        .fold((position, envelope[center]), |b, i| {
            if envelope[i] > b.1 {
                (i as f32, envelope[i])
            } else {
                b
            }
        })
        .0
}

impl BeatTracker for PhaseAlignedTracker {
    fn track(&self, envelope: &[f32], lag: f32) -> Vec<f32> {
        if envelope.is_empty() || lag <= 0.0 {
            return Vec::new();
        }
        let phase = best_phase(envelope, lag);
        grid(envelope.len(), phase, lag)
            .map(|p| snap(envelope, p, self.snap_radius))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beats_land_on_pulses_even_with_an_offset_phase() {
        let mut env = vec![0.0; 980];
        for k in 0..20 {
            env[37 + k * 48] = 1.0;
        }
        let beats = PhaseAlignedTracker { snap_radius: 2 }.track(&env, 48.0);
        assert_eq!(beats.len(), 20);
        for (k, b) in beats.iter().enumerate() {
            assert_eq!(*b, (37 + k * 48) as f32);
        }
    }

    #[test]
    fn spacing_follows_the_tempo_lag_when_onsets_are_missing() {
        let mut env = vec![0.0; 1000];
        env[10] = 1.0;
        env[110] = 1.0;
        let beats = PhaseAlignedTracker { snap_radius: 1 }.track(&env, 50.0);
        assert_eq!(beats.len(), 20);
        assert_eq!(beats[0], 10.0);
        assert_eq!(beats[2], 110.0);
        assert!((beats[1] - 60.0).abs() <= 1.0);
    }
}
