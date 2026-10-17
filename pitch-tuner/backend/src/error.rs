use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TunerError {
    EmptyFrame,
    InvalidSampleRate(u32),
    InvalidReference(f64),
    InvalidFrequency(f64),
    UnknownDetector(String),
    InvalidPort(String),
    Io(String),
}

impl fmt::Display for TunerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TunerError::EmptyFrame => write!(f, "frame has no samples"),
            TunerError::InvalidSampleRate(rate) => write!(f, "invalid sample rate {rate}"),
            TunerError::InvalidReference(a4) => write!(f, "invalid A4 reference {a4}"),
            TunerError::InvalidFrequency(freq) => write!(f, "invalid frequency {freq}"),
            TunerError::UnknownDetector(name) => write!(f, "unknown detector {name}"),
            TunerError::InvalidPort(port) => write!(f, "invalid port {port}"),
            TunerError::Io(msg) => write!(f, "io error: {msg}"),
        }
    }
}

impl std::error::Error for TunerError {}

impl From<std::io::Error> for TunerError {
    fn from(err: std::io::Error) -> Self {
        TunerError::Io(err.to_string())
    }
}
