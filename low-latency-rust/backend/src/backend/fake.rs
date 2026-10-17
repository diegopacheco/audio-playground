use super::{
    AudioBackend, DeviceInfo, DeviceList, RunningStream, StreamInfo, StreamRequest, device_id,
};
use crate::error::EngineError;
use crate::process::Sides;
use std::f32::consts::PI;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const FAKE_INPUT: &str = "loopback-in";
pub const FAKE_OUTPUT: &str = "loopback-out";

#[derive(Debug, Clone, Copy)]
pub struct FakeConfig {
    pub sample_rate: u32,
    pub delay_samples: usize,
    pub loop_gain: f32,
    pub mic_level: f32,
    pub realtime: bool,
}

impl Default for FakeConfig {
    fn default() -> Self {
        FakeConfig {
            sample_rate: 48_000,
            delay_samples: 480,
            loop_gain: 0.5,
            mic_level: 0.05,
            realtime: true,
        }
    }
}

pub struct FakeBackend {
    config: FakeConfig,
}

impl FakeBackend {
    pub fn new(config: FakeConfig) -> Self {
        FakeBackend { config }
    }
}

struct DelayLine {
    buf: Vec<f32>,
    pos: usize,
}

impl DelayLine {
    fn new(len: usize) -> Self {
        DelayLine {
            buf: vec![0.0; len],
            pos: 0,
        }
    }

    fn tick(&mut self, x: f32) -> f32 {
        if self.buf.is_empty() {
            return x;
        }
        let y = self.buf[self.pos];
        self.buf[self.pos] = x;
        self.pos = (self.pos + 1) % self.buf.len();
        y
    }
}

struct FakeRunning {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl RunningStream for FakeRunning {}

impl Drop for FakeRunning {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn run_loop(config: FakeConfig, frames: usize, mut sides: Sides, stop: Arc<AtomicBool>) {
    let mut out = vec![0.0f32; frames];
    let mut inp = vec![0.0f32; frames];
    let mut delay = DelayLine::new(config.delay_samples);
    let step = 2.0 * PI * 440.0 / config.sample_rate as f32;
    let mut phase = 0.0f32;
    let block = Duration::from_secs_f64(frames as f64 / config.sample_rate as f64);
    let mut deadline = Instant::now();
    while !stop.load(Ordering::Acquire) {
        sides.output.process(&mut out, 1);
        for (o, i) in out.iter().zip(inp.iter_mut()) {
            *i = config.loop_gain * delay.tick(*o) + config.mic_level * phase.sin();
            phase = (phase + step) % (2.0 * PI);
        }
        sides.input.process(&inp, 1);
        if config.realtime {
            deadline += block;
            let now = Instant::now();
            if deadline > now {
                thread::sleep(deadline - now);
            } else {
                deadline = now;
            }
        } else {
            thread::yield_now();
        }
    }
}

impl AudioBackend for FakeBackend {
    fn devices(&self) -> Result<DeviceList, EngineError> {
        let input = DeviceInfo {
            id: device_id("fake", FAKE_INPUT),
            name: format!(
                "Fake loopback input ({} samples)",
                self.config.delay_samples
            ),
            backend: "fake".into(),
        };
        let output = DeviceInfo {
            id: device_id("fake", FAKE_OUTPUT),
            name: "Fake loopback output".into(),
            backend: "fake".into(),
        };
        Ok(DeviceList {
            default_input: Some(input.id.clone()),
            default_output: Some(output.id.clone()),
            inputs: vec![input],
            outputs: vec![output],
        })
    }

    fn negotiate(&self, request: &StreamRequest) -> Result<StreamInfo, EngineError> {
        Ok(StreamInfo {
            backend: "fake".into(),
            sample_rate: self.config.sample_rate,
            buffer_frames: request.buffer_frames,
            input_device: FAKE_INPUT.into(),
            output_device: FAKE_OUTPUT.into(),
            input_channels: 1,
            output_channels: 1,
        })
    }

    fn start(
        &self,
        info: &StreamInfo,
        sides: Sides,
    ) -> Result<Box<dyn RunningStream>, EngineError> {
        let stop = Arc::new(AtomicBool::new(false));
        let config = self.config;
        let frames = info.buffer_frames as usize;
        let flag = Arc::clone(&stop);
        let handle = thread::Builder::new()
            .name("fake-audio".into())
            .spawn(move || run_loop(config, frames, sides, flag))
            .map_err(|e| EngineError::Device(format!("fake audio thread: {e}")))?;
        Ok(Box::new(FakeRunning {
            stop,
            handle: Some(handle),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_line_shifts_signal_by_exactly_its_length() {
        let mut d = DelayLine::new(3);
        let out: Vec<f32> = [1.0, 2.0, 3.0, 4.0, 5.0]
            .iter()
            .map(|x| d.tick(*x))
            .collect();
        assert_eq!(out, vec![0.0, 0.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn zero_delay_line_is_transparent() {
        let mut d = DelayLine::new(0);
        assert_eq!(d.tick(0.25), 0.25);
    }
}
