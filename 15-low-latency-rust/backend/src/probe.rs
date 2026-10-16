use std::f32::consts::PI;

pub trait LatencyProbe: Send + Sync {
    fn name(&self) -> &'static str;
    fn stimulus(&self) -> &[f32];
    fn detect(&self, captured: &[f32]) -> Option<usize>;
}

pub struct ThresholdProbe {
    click: Vec<f32>,
    onset: usize,
}

const MIN_LEVEL: f32 = 0.02;
const ONSET_FRACTION: f32 = 0.5;

impl ThresholdProbe {
    pub fn new() -> Self {
        let len = 32;
        let click: Vec<f32> = (0..len)
            .map(|i| 0.9 * (PI * i as f32 / len as f32).sin())
            .collect();
        let onset = first_above(&click, ONSET_FRACTION * peak(&click)).unwrap_or(0);
        ThresholdProbe { click, onset }
    }
}

impl LatencyProbe for ThresholdProbe {
    fn name(&self) -> &'static str {
        "threshold"
    }

    fn stimulus(&self) -> &[f32] {
        &self.click
    }

    fn detect(&self, captured: &[f32]) -> Option<usize> {
        let top = peak(captured);
        if top < MIN_LEVEL {
            return None;
        }
        let idx = first_above(captured, ONSET_FRACTION * top)?;
        Some(idx.saturating_sub(self.onset))
    }
}

pub struct CorrelationProbe {
    chirp: Vec<f32>,
    energy: f32,
}

impl CorrelationProbe {
    pub fn new(sample_rate: u32) -> Self {
        let len = 1024usize;
        let sr = sample_rate as f32;
        let f0 = 500.0f32;
        let f1 = (8000.0f32).min(sr * 0.45);
        let duration = len as f32 / sr;
        let chirp: Vec<f32> = (0..len)
            .map(|i| {
                let t = i as f32 / sr;
                let phase = 2.0 * PI * (f0 * t + (f1 - f0) * t * t / (2.0 * duration));
                let window = 0.5 - 0.5 * (2.0 * PI * i as f32 / (len - 1) as f32).cos();
                0.8 * window * phase.sin()
            })
            .collect();
        let energy = chirp.iter().map(|v| v * v).sum();
        CorrelationProbe { chirp, energy }
    }
}

impl LatencyProbe for CorrelationProbe {
    fn name(&self) -> &'static str {
        "correlation"
    }

    fn stimulus(&self) -> &[f32] {
        &self.chirp
    }

    fn detect(&self, captured: &[f32]) -> Option<usize> {
        let n = self.chirp.len();
        if captured.len() < n {
            return None;
        }
        let mut best = 0.0f32;
        let mut best_idx = 0usize;
        for lag in 0..=captured.len() - n {
            let c: f32 = captured[lag..lag + n]
                .iter()
                .zip(&self.chirp)
                .map(|(a, b)| a * b)
                .sum();
            if c > best {
                best = c;
                best_idx = lag;
            }
        }
        if best / self.energy < MIN_LEVEL {
            return None;
        }
        Some(best_idx)
    }
}

pub fn probe_by_name(name: &str, sample_rate: u32) -> Option<Box<dyn LatencyProbe>> {
    match name {
        "threshold" => Some(Box::new(ThresholdProbe::new())),
        "correlation" => Some(Box::new(CorrelationProbe::new(sample_rate))),
        _ => None,
    }
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |m, v| m.max(v.abs()))
}

fn first_above(samples: &[f32], level: f32) -> Option<usize> {
    samples.iter().position(|v| v.abs() >= level)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn embed(stimulus: &[f32], delay: usize, gain: f32, total: usize) -> Vec<f32> {
        let mut buf = vec![0.0f32; total];
        for (i, s) in stimulus.iter().enumerate() {
            buf[delay + i] = s * gain;
        }
        buf
    }

    fn add_hum(buf: &mut [f32], amplitude: f32) {
        for (i, v) in buf.iter_mut().enumerate() {
            *v += amplitude * (2.0 * PI * 60.0 * i as f32 / 48000.0).sin();
        }
    }

    #[test]
    fn correlation_finds_attenuated_chirp_at_its_exact_offset_despite_hum() {
        let probe = CorrelationProbe::new(48000);
        let mut buf = embed(probe.stimulus(), 777, 0.3, 8000);
        add_hum(&mut buf, 0.2);
        assert_eq!(probe.detect(&buf), Some(777));
    }

    #[test]
    fn correlation_reports_nothing_when_input_is_silent_instead_of_a_fake_zero_latency() {
        let probe = CorrelationProbe::new(48000);
        assert_eq!(probe.detect(&vec![0.0; 4000]), None);
    }

    #[test]
    fn threshold_compensates_its_own_onset_so_scaled_clicks_report_true_start() {
        let probe = ThresholdProbe::new();
        let buf = embed(probe.stimulus(), 480, 0.4, 2000);
        assert_eq!(probe.detect(&buf), Some(480));
    }

    #[test]
    fn threshold_ignores_signal_below_minimum_level() {
        let probe = ThresholdProbe::new();
        let buf = embed(probe.stimulus(), 100, 0.01, 1000);
        assert_eq!(probe.detect(&buf), None);
    }

    #[test]
    fn unknown_probe_name_is_rejected() {
        assert!(probe_by_name("fft", 48000).is_none());
        assert!(probe_by_name("threshold", 48000).is_some());
    }
}
