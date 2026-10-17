use super::{
    AudioBackend, DeviceInfo, DeviceList, RunningStream, StreamInfo, StreamRequest, device_id,
};
use crate::error::EngineError;
use crate::process::Sides;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, Host, SampleFormat, Stream, StreamConfig, SupportedBufferSize};
use std::time::Duration;

pub struct CpalBackend {
    host: Host,
}

struct CpalRunning {
    _input: Stream,
    _output: Stream,
}

impl RunningStream for CpalRunning {}

fn dev_err(context: &str, e: impl std::fmt::Display) -> EngineError {
    EngineError::Device(format!("{context}: {e}"))
}

fn device_name(device: &Device) -> String {
    device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

impl CpalBackend {
    pub fn new() -> Self {
        CpalBackend {
            host: cpal::default_host(),
        }
    }

    pub fn has_devices(&self) -> bool {
        self.host.default_input_device().is_some() && self.host.default_output_device().is_some()
    }

    fn find_input(&self, name: Option<&str>) -> Result<Device, EngineError> {
        match name {
            None => self
                .host
                .default_input_device()
                .ok_or_else(|| EngineError::Device("no default input device".into())),
            Some(n) => self
                .host
                .input_devices()
                .map_err(|e| dev_err("input devices", e))?
                .find(|d| device_name(d) == n)
                .ok_or_else(|| EngineError::Device(format!("input device not found: {n}"))),
        }
    }

    fn find_output(&self, name: Option<&str>) -> Result<Device, EngineError> {
        match name {
            None => self
                .host
                .default_output_device()
                .ok_or_else(|| EngineError::Device("no default output device".into())),
            Some(n) => self
                .host
                .output_devices()
                .map_err(|e| dev_err("output devices", e))?
                .find(|d| device_name(d) == n)
                .ok_or_else(|| EngineError::Device(format!("output device not found: {n}"))),
        }
    }

    fn input_channels(&self, device: &Device, rate: u32) -> Result<u16, EngineError> {
        device
            .supported_input_configs()
            .map_err(|e| dev_err("input configs", e))?
            .find(|c| {
                c.sample_format() == SampleFormat::F32
                    && c.min_sample_rate() <= rate
                    && c.max_sample_rate() >= rate
            })
            .map(|c| c.channels())
            .ok_or_else(|| {
                EngineError::Device(format!("input device has no f32 config at {rate} Hz"))
            })
    }
}

fn clamp_frames(frames: u32, supported: &SupportedBufferSize) -> u32 {
    match supported {
        SupportedBufferSize::Range { min, max } => frames.clamp(*min, *max),
        SupportedBufferSize::Unknown => frames,
    }
}

impl AudioBackend for CpalBackend {
    fn devices(&self) -> Result<DeviceList, EngineError> {
        let to_info = |d: Device| {
            let name = device_name(&d);
            DeviceInfo {
                id: device_id("cpal", &name),
                name,
                backend: "cpal".into(),
            }
        };
        let inputs = self
            .host
            .input_devices()
            .map_err(|e| dev_err("input devices", e))?
            .map(to_info)
            .collect();
        let outputs = self
            .host
            .output_devices()
            .map_err(|e| dev_err("output devices", e))?
            .map(to_info)
            .collect();
        Ok(DeviceList {
            inputs,
            outputs,
            default_input: self
                .host
                .default_input_device()
                .map(|d| device_id("cpal", &device_name(&d))),
            default_output: self
                .host
                .default_output_device()
                .map(|d| device_id("cpal", &device_name(&d))),
        })
    }

    fn negotiate(&self, request: &StreamRequest) -> Result<StreamInfo, EngineError> {
        let input = self.find_input(request.input.as_deref())?;
        let output = self.find_output(request.output.as_deref())?;
        let out_cfg = output
            .default_output_config()
            .map_err(|e| dev_err("output config", e))?;
        if out_cfg.sample_format() != SampleFormat::F32 {
            return Err(EngineError::Device(format!(
                "output sample format {} is not f32",
                out_cfg.sample_format()
            )));
        }
        let sample_rate = out_cfg.sample_rate();
        let input_channels = self.input_channels(&input, sample_rate)?;
        Ok(StreamInfo {
            backend: "cpal".into(),
            sample_rate,
            buffer_frames: clamp_frames(request.buffer_frames, out_cfg.buffer_size()),
            input_device: device_name(&input),
            output_device: device_name(&output),
            input_channels,
            output_channels: out_cfg.channels(),
        })
    }

    fn start(
        &self,
        info: &StreamInfo,
        sides: Sides,
    ) -> Result<Box<dyn RunningStream>, EngineError> {
        let input = self.find_input(Some(&info.input_device))?;
        let output = self.find_output(Some(&info.output_device))?;
        let timeout = Some(Duration::from_secs(3));
        let Sides {
            input: mut in_side,
            output: mut out_side,
        } = sides;
        let in_ch = info.input_channels as usize;
        let out_ch = info.output_channels as usize;
        let in_cfg = StreamConfig {
            channels: info.input_channels,
            sample_rate: info.sample_rate,
            buffer_size: BufferSize::Fixed(info.buffer_frames),
        };
        let out_cfg = StreamConfig {
            channels: info.output_channels,
            sample_rate: info.sample_rate,
            buffer_size: BufferSize::Fixed(info.buffer_frames),
        };
        let in_stream = input
            .build_input_stream::<f32, _, _>(
                in_cfg,
                move |data, _| in_side.process(data, in_ch),
                |e| eprintln!("input stream error: {e}"),
                timeout,
            )
            .map_err(|e| dev_err("build input stream", e))?;
        let out_stream = output
            .build_output_stream::<f32, _, _>(
                out_cfg,
                move |data, _| out_side.process(data, out_ch),
                |e| eprintln!("output stream error: {e}"),
                timeout,
            )
            .map_err(|e| dev_err("build output stream", e))?;
        in_stream.play().map_err(|e| dev_err("play input", e))?;
        out_stream.play().map_err(|e| dev_err("play output", e))?;
        Ok(Box::new(CpalRunning {
            _input: in_stream,
            _output: out_stream,
        }))
    }
}
