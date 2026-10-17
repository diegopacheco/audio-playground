use crate::backend::{
    AudioBackend, DeviceList, RunningStream, StreamInfo, StreamRequest, split_id,
};
use crate::error::EngineError;
use crate::probe::{LatencyProbe, probe_by_name};
use crate::process::{InputSide, MeasureState, Mode, OutputSide, Sides};
use crate::ring::ring;
use crate::stats::{CounterSnapshot, Stats};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub const MIN_FRAMES: u32 = 16;
pub const MAX_FRAMES: u32 = 4096;

#[derive(Debug, Clone, Deserialize)]
pub struct StartRequest {
    pub mode: Mode,
    pub buffer_frames: u32,
    pub input: Option<String>,
    pub output: Option<String>,
    pub probe: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct MeasureResult {
    pub round_trip_ms: f64,
    pub samples: usize,
    pub sample_rate: u32,
    pub buffer_frames: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EngineStatus {
    pub running: bool,
    pub mode: Option<Mode>,
    pub probe: Option<String>,
    pub stream: Option<StreamInfo>,
    pub buffer_ms: Option<f64>,
    pub ring_capacity: usize,
    pub ring_fill_ratio: f64,
    pub counters: Option<CounterSnapshot>,
    pub fallback_reason: Option<String>,
    pub last_measure: Option<MeasureResult>,
}

struct Running {
    _stream: Box<dyn RunningStream>,
    info: StreamInfo,
    mode: Mode,
    probe: Box<dyn LatencyProbe>,
    stats: Arc<Stats>,
    measure: Arc<MeasureState>,
    ring_capacity: usize,
    fallback_reason: Option<String>,
}

pub struct Engine {
    fake: Box<dyn AudioBackend>,
    real: Option<Box<dyn AudioBackend>>,
    running: Option<Running>,
    last_measure: Option<MeasureResult>,
}

impl Engine {
    pub fn new(fake: Box<dyn AudioBackend>, real: Option<Box<dyn AudioBackend>>) -> Self {
        Engine {
            fake,
            real,
            running: None,
            last_measure: None,
        }
    }

    pub fn devices(&self) -> DeviceList {
        let mut list = self.fake.devices().unwrap_or_default();
        if let Some(real) = &self.real
            && let Ok(r) = real.devices()
        {
            let mut inputs = r.inputs;
            inputs.append(&mut list.inputs);
            let mut outputs = r.outputs;
            outputs.append(&mut list.outputs);
            list = DeviceList {
                inputs,
                outputs,
                default_input: r.default_input.or(list.default_input),
                default_output: r.default_output.or(list.default_output),
            };
        }
        list
    }

    fn pick_backend(&self, req: &StartRequest) -> Result<(bool, StreamRequest), EngineError> {
        let devices = self.devices();
        let input = req.input.clone().or(devices.default_input);
        let output = req.output.clone().or(devices.default_output);
        let parse = |id: &Option<String>| -> Result<(String, String), EngineError> {
            let id = id
                .as_deref()
                .ok_or_else(|| EngineError::InvalidConfig("no device available".into()))?;
            split_id(id)
                .map(|(b, n)| (b.to_string(), n.to_string()))
                .ok_or_else(|| EngineError::InvalidConfig(format!("bad device id: {id}")))
        };
        let (in_backend, in_name) = parse(&input)?;
        let (out_backend, out_name) = parse(&output)?;
        if in_backend != out_backend {
            return Err(EngineError::InvalidConfig(
                "input and output must use the same backend".into(),
            ));
        }
        let use_real = in_backend == "cpal";
        if use_real && self.real.is_none() {
            return Err(EngineError::Device("cpal backend is not available".into()));
        }
        Ok((
            use_real,
            StreamRequest {
                input: Some(in_name),
                output: Some(out_name),
                buffer_frames: req.buffer_frames,
            },
        ))
    }

    pub fn start(&mut self, req: &StartRequest) -> Result<EngineStatus, EngineError> {
        if !(MIN_FRAMES..=MAX_FRAMES).contains(&req.buffer_frames) {
            return Err(EngineError::InvalidConfig(format!(
                "buffer_frames must be within {MIN_FRAMES}..={MAX_FRAMES}"
            )));
        }
        let probe_name = req.probe.clone().unwrap_or_else(|| "correlation".into());
        if probe_by_name(&probe_name, 48_000).is_none() {
            return Err(EngineError::InvalidConfig(format!(
                "unknown probe: {probe_name}"
            )));
        }
        self.stop();
        let (use_real, stream_req) = self.pick_backend(req)?;
        let mut fallback_reason = None;
        if use_real && let Some(real) = &self.real {
            match launch(real.as_ref(), &stream_req, req.mode, &probe_name) {
                Ok(r) => {
                    self.running = Some(r);
                    return Ok(self.status());
                }
                Err(e) => fallback_reason = Some(e.to_string()),
            }
        }
        let fake_req = StreamRequest {
            input: None,
            output: None,
            buffer_frames: req.buffer_frames,
        };
        let mut r = launch(self.fake.as_ref(), &fake_req, req.mode, &probe_name)?;
        r.fallback_reason = fallback_reason;
        self.running = Some(r);
        Ok(self.status())
    }

    pub fn stop(&mut self) -> EngineStatus {
        self.running = None;
        self.status()
    }

    pub fn begin_measure(&self) -> Result<Arc<MeasureState>, EngineError> {
        let r = self.running.as_ref().ok_or(EngineError::NotRunning)?;
        if !r.measure.arm() {
            return Err(EngineError::InvalidConfig(
                "measurement already in progress".into(),
            ));
        }
        Ok(Arc::clone(&r.measure))
    }

    pub fn finish_measure(
        &mut self,
        measure: &Arc<MeasureState>,
        captured: &[f32],
    ) -> Result<MeasureResult, EngineError> {
        let r = self.running.as_ref().ok_or(EngineError::NotRunning)?;
        if !Arc::ptr_eq(&r.measure, measure) {
            return Err(EngineError::NotRunning);
        }
        let samples = r.probe.detect(captured).ok_or(EngineError::NotDetected)?;
        let result = MeasureResult {
            round_trip_ms: samples as f64 * 1000.0 / r.info.sample_rate as f64,
            samples,
            sample_rate: r.info.sample_rate,
            buffer_frames: r.info.buffer_frames,
        };
        self.last_measure = Some(result);
        Ok(result)
    }

    pub fn status(&self) -> EngineStatus {
        match &self.running {
            None => EngineStatus {
                running: false,
                mode: None,
                probe: None,
                stream: None,
                buffer_ms: None,
                ring_capacity: 0,
                ring_fill_ratio: 0.0,
                counters: None,
                fallback_reason: None,
                last_measure: self.last_measure,
            },
            Some(r) => {
                let counters = r.stats.snapshot();
                EngineStatus {
                    running: true,
                    mode: Some(r.mode),
                    probe: Some(r.probe.name().to_string()),
                    buffer_ms: Some(
                        r.info.buffer_frames as f64 * 1000.0 / r.info.sample_rate as f64,
                    ),
                    stream: Some(r.info.clone()),
                    ring_capacity: r.ring_capacity,
                    ring_fill_ratio: counters.ring_fill as f64 / r.ring_capacity as f64,
                    counters: Some(counters),
                    fallback_reason: r.fallback_reason.clone(),
                    last_measure: self.last_measure,
                }
            }
        }
    }
}

fn launch(
    backend: &dyn AudioBackend,
    req: &StreamRequest,
    mode: Mode,
    probe_name: &str,
) -> Result<Running, EngineError> {
    let info = backend.negotiate(req)?;
    let probe = probe_by_name(probe_name, info.sample_rate)
        .ok_or_else(|| EngineError::InvalidConfig(format!("unknown probe: {probe_name}")))?;
    let ring_capacity = (info.buffer_frames as usize * 8)
        .next_power_of_two()
        .max(1024);
    let (mut producer, consumer) = ring::<f32>(ring_capacity)?;
    for _ in 0..info.buffer_frames {
        producer.push(0.0);
    }
    let stats = Arc::new(Stats::default());
    let measure = Arc::new(MeasureState::new(info.sample_rate as usize / 2));
    let stimulus: Arc<[f32]> = Arc::from(probe.stimulus());
    let sides = Sides {
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
    let stream = backend.start(&info, sides)?;
    Ok(Running {
        _stream: stream,
        info,
        mode,
        probe,
        stats,
        measure,
        ring_capacity,
        fallback_reason: None,
    })
}

fn lock(engine: &Mutex<Engine>) -> Result<std::sync::MutexGuard<'_, Engine>, EngineError> {
    engine
        .lock()
        .map_err(|_| EngineError::Device("engine lock poisoned".into()))
}

pub fn with_engine<T>(
    engine: &Mutex<Engine>,
    f: impl FnOnce(&mut Engine) -> Result<T, EngineError>,
) -> Result<T, EngineError> {
    let mut guard = lock(engine)?;
    f(&mut guard)
}

pub fn measure_blocking(
    engine: &Mutex<Engine>,
    timeout: Duration,
) -> Result<MeasureResult, EngineError> {
    let state = lock(engine)?.begin_measure()?;
    let deadline = Instant::now() + timeout;
    while !state.is_done() {
        if Instant::now() > deadline {
            state.cancel();
            return Err(EngineError::MeasureTimeout);
        }
        thread::sleep(Duration::from_millis(5));
    }
    let captured = state.take();
    lock(engine)?.finish_measure(&state, &captured)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::fake::{FakeBackend, FakeConfig};

    struct BrokenBackend;

    impl AudioBackend for BrokenBackend {
        fn devices(&self) -> Result<DeviceList, EngineError> {
            Ok(DeviceList {
                inputs: vec![],
                outputs: vec![],
                default_input: Some("cpal:mic".into()),
                default_output: Some("cpal:speaker".into()),
            })
        }
        fn negotiate(&self, _: &StreamRequest) -> Result<StreamInfo, EngineError> {
            Err(EngineError::Device("permission denied".into()))
        }
        fn start(&self, _: &StreamInfo, _: Sides) -> Result<Box<dyn RunningStream>, EngineError> {
            Err(EngineError::Device("unreachable".into()))
        }
    }

    fn fake(delay: usize) -> Box<dyn AudioBackend> {
        Box::new(FakeBackend::new(FakeConfig {
            delay_samples: delay,
            realtime: false,
            ..FakeConfig::default()
        }))
    }

    fn request(mode: Mode, probe: &str) -> StartRequest {
        StartRequest {
            mode,
            buffer_frames: 128,
            input: None,
            output: None,
            probe: Some(probe.into()),
        }
    }

    fn measure(delay: usize, mode: Mode, probe: &str) -> MeasureResult {
        let engine = Mutex::new(Engine::new(fake(delay), None));
        with_engine(&engine, |e| e.start(&request(mode, probe))).unwrap();
        let result = measure_blocking(&engine, Duration::from_secs(10)).unwrap();
        with_engine(&engine, |e| Ok(e.stop())).unwrap();
        result
    }

    #[test]
    fn correlation_measures_injected_480_sample_loop_as_10ms_at_48k() {
        let r = measure(480, Mode::Measure, "correlation");
        assert_eq!(r.samples, 480);
        assert!((r.round_trip_ms - 10.0).abs() < 1e-9);
    }

    #[test]
    fn measurement_tracks_a_different_delay_so_it_is_not_a_constant() {
        let r = measure(1000, Mode::Passthrough, "correlation");
        assert_eq!(r.samples, 1000);
    }

    #[test]
    fn threshold_probe_also_recovers_the_injected_delay() {
        let r = measure(480, Mode::Measure, "threshold");
        assert!((r.samples as i64 - 480).abs() <= 2, "got {}", r.samples);
    }

    #[test]
    fn failing_real_device_falls_back_to_fake_and_reports_why() {
        let mut engine = Engine::new(fake(480), Some(Box::new(BrokenBackend)));
        let status = engine
            .start(&request(Mode::Passthrough, "correlation"))
            .unwrap();
        assert!(status.running);
        assert_eq!(status.stream.unwrap().backend, "fake");
        assert!(
            status
                .fallback_reason
                .unwrap()
                .contains("permission denied")
        );
        engine.stop();
    }

    #[test]
    fn measuring_a_stopped_engine_is_an_error_not_a_panic() {
        let engine = Mutex::new(Engine::new(fake(480), None));
        assert_eq!(
            measure_blocking(&engine, Duration::from_millis(50)),
            Err(EngineError::NotRunning)
        );
    }

    #[test]
    fn invalid_buffer_size_is_rejected_before_touching_devices() {
        let mut engine = Engine::new(fake(480), None);
        let mut req = request(Mode::Passthrough, "correlation");
        req.buffer_frames = 3;
        assert!(engine.start(&req).is_err());
        assert!(!engine.status().running);
    }

    #[test]
    fn mixing_backends_between_input_and_output_is_rejected() {
        let mut engine = Engine::new(fake(480), Some(Box::new(BrokenBackend)));
        let mut req = request(Mode::Passthrough, "correlation");
        req.input = Some("fake:loopback-in".into());
        req.output = Some("cpal:speaker".into());
        assert!(matches!(
            engine.start(&req),
            Err(EngineError::InvalidConfig(_))
        ));
    }
}
