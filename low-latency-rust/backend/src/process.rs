use crate::ring::{Consumer, Producer};
use crate::stats::Stats;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Passthrough,
    Measure,
}

const IDLE: u8 = 0;
const ARMED: u8 = 1;
const CAPTURING: u8 = 2;
const DONE: u8 = 3;

pub struct MeasureState {
    phase: AtomicU8,
    written: AtomicUsize,
    capture: Box<[AtomicU32]>,
}

impl MeasureState {
    pub fn new(capture_len: usize) -> Self {
        MeasureState {
            phase: AtomicU8::new(IDLE),
            written: AtomicUsize::new(0),
            capture: (0..capture_len).map(|_| AtomicU32::new(0)).collect(),
        }
    }

    pub fn arm(&self) -> bool {
        let phase = self.phase.load(Ordering::Acquire);
        if phase == ARMED || phase == CAPTURING {
            return false;
        }
        self.written.store(0, Ordering::Relaxed);
        self.phase.store(ARMED, Ordering::Release);
        true
    }

    pub fn is_done(&self) -> bool {
        self.phase.load(Ordering::Acquire) == DONE
    }

    pub fn take(&self) -> Vec<f32> {
        let n = self.written.load(Ordering::Acquire);
        let data = self.capture[..n]
            .iter()
            .map(|v| f32::from_bits(v.load(Ordering::Relaxed)))
            .collect();
        self.phase.store(IDLE, Ordering::Release);
        data
    }

    pub fn cancel(&self) {
        self.phase.store(IDLE, Ordering::Release);
    }

    fn measuring(&self) -> bool {
        let phase = self.phase.load(Ordering::Acquire);
        phase == ARMED || phase == CAPTURING
    }

    fn begin_emit(&self) -> bool {
        self.phase
            .compare_exchange(ARMED, CAPTURING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    fn record(&self, sample: f32) {
        if self.phase.load(Ordering::Acquire) != CAPTURING {
            return;
        }
        let n = self.written.load(Ordering::Relaxed);
        if n >= self.capture.len() {
            return;
        }
        self.capture[n].store(sample.to_bits(), Ordering::Relaxed);
        self.written.store(n + 1, Ordering::Release);
        if n + 1 == self.capture.len() {
            self.phase.store(DONE, Ordering::Release);
        }
    }
}

pub struct InputSide {
    pub producer: Producer<f32>,
    pub stats: Arc<Stats>,
    pub measure: Arc<MeasureState>,
}

impl InputSide {
    pub fn process(&mut self, interleaved: &[f32], channels: usize) {
        let start = Instant::now();
        let channels = channels.max(1);
        let mut overflow = false;
        for frame in interleaved.chunks(channels) {
            let sample = frame.iter().sum::<f32>() / frame.len() as f32;
            if !self.producer.push(sample) {
                overflow = true;
            }
            self.measure.record(sample);
        }
        if overflow {
            self.stats.overruns.fetch_add(1, Ordering::Relaxed);
        }
        self.stats
            .ring_fill
            .store(self.producer.len() as u64, Ordering::Relaxed);
        self.stats.input.record(start.elapsed().as_nanos() as u64);
    }
}

pub struct OutputSide {
    pub consumer: Consumer<f32>,
    pub stats: Arc<Stats>,
    pub measure: Arc<MeasureState>,
    pub stimulus: Arc<[f32]>,
    pub mode: Mode,
    pub stimulus_pos: Option<usize>,
}

impl OutputSide {
    pub fn process(&mut self, interleaved: &mut [f32], channels: usize) {
        let start = Instant::now();
        let channels = channels.max(1);
        if self.measure.begin_emit() {
            self.stimulus_pos = Some(0);
        }
        let muted = self.measure.measuring() || self.mode == Mode::Measure;
        let mut starved = false;
        for frame in interleaved.chunks_mut(channels) {
            let monitored = match self.consumer.pop() {
                Some(v) => v,
                None => {
                    starved = true;
                    0.0
                }
            };
            let value = match self.stimulus_pos {
                Some(pos) if pos < self.stimulus.len() => {
                    self.stimulus_pos = Some(pos + 1);
                    self.stimulus[pos]
                }
                _ if muted => 0.0,
                _ => monitored,
            };
            frame.fill(value);
        }
        if self.stimulus_pos.is_some_and(|p| p >= self.stimulus.len()) {
            self.stimulus_pos = None;
        }
        if starved {
            self.stats.underruns.fetch_add(1, Ordering::Relaxed);
        }
        self.stats
            .ring_fill
            .store(self.consumer.len() as u64, Ordering::Relaxed);
        self.stats.output.record(start.elapsed().as_nanos() as u64);
    }
}

pub struct Sides {
    pub input: InputSide,
    pub output: OutputSide,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ring::ring;

    fn sides(mode: Mode, capacity: usize) -> (Sides, Arc<Stats>, Arc<MeasureState>) {
        let (producer, consumer) = ring::<f32>(capacity).unwrap();
        let stats = Arc::new(Stats::default());
        let measure = Arc::new(MeasureState::new(16));
        let stimulus: Arc<[f32]> = Arc::from(vec![1.0f32, -1.0]);
        let s = Sides {
            input: InputSide {
                producer,
                stats: Arc::clone(&stats),
                measure: Arc::clone(&measure),
            },
            output: OutputSide {
                consumer,
                stats: Arc::clone(&stats),
                measure: Arc::clone(&measure),
                stimulus,
                mode,
                stimulus_pos: None,
            },
        };
        (s, stats, measure)
    }

    #[test]
    fn passthrough_copies_mono_input_to_every_output_channel() {
        let (mut s, stats, _) = sides(Mode::Passthrough, 8);
        s.input.process(&[0.2, 0.4, 0.6, 0.8], 2);
        let mut out = [0.0f32; 4];
        s.output.process(&mut out, 2);
        assert!((out[0] - 0.3).abs() < 1e-6 && (out[1] - 0.3).abs() < 1e-6);
        assert!((out[2] - 0.7).abs() < 1e-6 && (out[3] - 0.7).abs() < 1e-6);
        assert_eq!(stats.snapshot().underruns, 0);
    }

    #[test]
    fn output_faster_than_input_counts_an_underrun() {
        let (mut s, stats, _) = sides(Mode::Passthrough, 8);
        s.input.process(&[0.5], 1);
        let mut out = [9.0f32; 3];
        s.output.process(&mut out, 1);
        assert_eq!(out, [0.5, 0.0, 0.0]);
        assert_eq!(stats.snapshot().underruns, 1);
    }

    #[test]
    fn input_faster_than_output_counts_an_overrun() {
        let (mut s, stats, _) = sides(Mode::Passthrough, 4);
        s.input.process(&[0.1; 6], 1);
        let snap = stats.snapshot();
        assert_eq!(snap.overruns, 1);
        assert_eq!(snap.ring_fill, 4);
    }

    #[test]
    fn measure_mode_stays_silent_until_armed_then_emits_stimulus_once() {
        let (mut s, _, measure) = sides(Mode::Measure, 16);
        s.input.process(&[0.5; 4], 1);
        let mut out = [9.0f32; 4];
        s.output.process(&mut out, 1);
        assert_eq!(out, [0.0; 4]);
        assert!(measure.arm());
        let mut out = [9.0f32; 4];
        s.output.process(&mut out, 1);
        assert_eq!(out, [1.0, -1.0, 0.0, 0.0]);
        let mut out = [9.0f32; 4];
        s.output.process(&mut out, 1);
        assert_eq!(out, [0.0; 4]);
    }

    #[test]
    fn capture_completes_only_after_emission_and_buffer_fills() {
        let (mut s, _, measure) = sides(Mode::Measure, 64);
        assert!(measure.arm());
        s.input.process(&[0.3; 16], 1);
        assert!(!measure.is_done());
        let mut out = [0.0f32; 4];
        s.output.process(&mut out, 1);
        s.input.process(&[0.7; 16], 1);
        assert!(measure.is_done());
        let data = measure.take();
        assert_eq!(data, vec![0.7; 16]);
        assert!(!measure.is_done());
    }
}
