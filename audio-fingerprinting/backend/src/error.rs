use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FingerprintError {
    InvalidWav(String),
    UnsupportedFormat(String),
    InvalidSampleRate(u32),
    EmptyAudio,
    UnknownTrack(usize),
    InvalidParameter(String),
    InvalidPort(String),
    LockPoisoned,
    Io(String),
}

impl fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FingerprintError::InvalidWav(msg) => write!(f, "invalid wav: {msg}"),
            FingerprintError::UnsupportedFormat(msg) => write!(f, "unsupported format: {msg}"),
            FingerprintError::InvalidSampleRate(rate) => write!(f, "invalid sample rate {rate}"),
            FingerprintError::EmptyAudio => write!(f, "audio has no samples"),
            FingerprintError::UnknownTrack(id) => write!(f, "unknown track {id}"),
            FingerprintError::InvalidParameter(msg) => write!(f, "invalid parameter: {msg}"),
            FingerprintError::InvalidPort(port) => write!(f, "invalid port {port}"),
            FingerprintError::LockPoisoned => write!(f, "library lock poisoned"),
            FingerprintError::Io(msg) => write!(f, "io error: {msg}"),
        }
    }
}

impl std::error::Error for FingerprintError {}

impl From<std::io::Error> for FingerprintError {
    fn from(err: std::io::Error) -> Self {
        FingerprintError::Io(err.to_string())
    }
}
