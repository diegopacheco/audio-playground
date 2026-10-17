use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    InvalidConfig(String),
    Device(String),
    NotRunning,
    MeasureTimeout,
    NotDetected,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::InvalidConfig(m) => write!(f, "invalid config: {m}"),
            EngineError::Device(m) => write!(f, "device error: {m}"),
            EngineError::NotRunning => write!(f, "engine is not running"),
            EngineError::MeasureTimeout => write!(f, "measurement timed out"),
            EngineError::NotDetected => write!(f, "stimulus not detected in captured input"),
        }
    }
}

impl std::error::Error for EngineError {}
