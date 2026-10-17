use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum VoiceError {
    InvalidFftSize(usize),
    InvalidHop { size: usize, hop: usize },
    InvalidSampleRate(u32),
    InvalidSemitones(f32),
    InvalidDuration(f32),
    UnknownEffect(String),
    InvalidWav(String),
    UnsupportedWav(String),
    InvalidBlock(usize),
    InvalidPort(String),
    Io(String),
}

impl fmt::Display for VoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VoiceError::InvalidFftSize(size) => write!(f, "fft size {size} is not a power of two"),
            VoiceError::InvalidHop { size, hop } => {
                write!(f, "hop {hop} is invalid for frame size {size}")
            }
            VoiceError::InvalidSampleRate(rate) => write!(f, "invalid sample rate {rate}"),
            VoiceError::InvalidSemitones(value) => write!(f, "semitones {value} out of range"),
            VoiceError::InvalidDuration(value) => write!(f, "duration {value} out of range"),
            VoiceError::UnknownEffect(name) => write!(f, "unknown effect {name}"),
            VoiceError::InvalidWav(msg) => write!(f, "invalid wav: {msg}"),
            VoiceError::UnsupportedWav(msg) => write!(f, "unsupported wav: {msg}"),
            VoiceError::InvalidBlock(len) => {
                write!(f, "binary block of {len} bytes is not a float32 array")
            }
            VoiceError::InvalidPort(port) => write!(f, "invalid port {port}"),
            VoiceError::Io(msg) => write!(f, "io error: {msg}"),
        }
    }
}

impl std::error::Error for VoiceError {}

impl From<std::io::Error> for VoiceError {
    fn from(err: std::io::Error) -> Self {
        VoiceError::Io(err.to_string())
    }
}
