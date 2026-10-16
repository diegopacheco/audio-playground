pub mod cpal_backend;
pub mod fake;

use crate::error::EngineError;
use crate::process::Sides;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub backend: String,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct DeviceList {
    pub inputs: Vec<DeviceInfo>,
    pub outputs: Vec<DeviceInfo>,
    pub default_input: Option<String>,
    pub default_output: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StreamRequest {
    pub input: Option<String>,
    pub output: Option<String>,
    pub buffer_frames: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StreamInfo {
    pub backend: String,
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub input_device: String,
    pub output_device: String,
    pub input_channels: u16,
    pub output_channels: u16,
}

pub trait RunningStream: Send {}

pub trait AudioBackend: Send {
    fn devices(&self) -> Result<DeviceList, EngineError>;
    fn negotiate(&self, request: &StreamRequest) -> Result<StreamInfo, EngineError>;
    fn start(&self, info: &StreamInfo, sides: Sides)
    -> Result<Box<dyn RunningStream>, EngineError>;
}

pub fn device_id(backend: &str, name: &str) -> String {
    format!("{backend}:{name}")
}

pub fn split_id(id: &str) -> Option<(&str, &str)> {
    id.split_once(':')
}
