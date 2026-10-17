use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DspError {
    EmptyInput,
    NotPowerOfTwo(usize),
    UnknownWindow(String),
    InvalidSampleRate(f32),
    InvalidBarCount(usize),
    MisalignedPcm(usize),
}

impl fmt::Display for DspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DspError::EmptyInput => write!(f, "input frame is empty"),
            DspError::NotPowerOfTwo(n) => write!(f, "frame length {n} is not a power of two"),
            DspError::UnknownWindow(name) => write!(f, "unknown window '{name}'"),
            DspError::InvalidSampleRate(r) => write!(f, "invalid sample rate {r}"),
            DspError::InvalidBarCount(b) => write!(f, "bar count {b} must be between 1 and 512"),
            DspError::MisalignedPcm(n) => {
                write!(f, "PCM payload of {n} bytes is not a multiple of 4")
            }
        }
    }
}

impl std::error::Error for DspError {}

#[derive(Debug)]
pub enum AppError {
    MissingPort,
    InvalidPort(String),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingPort => write!(f, "PORT environment variable is not set"),
            AppError::InvalidPort(p) => write!(f, "PORT '{p}' is not a valid port"),
            AppError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}
