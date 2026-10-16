use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AudioError {
    InvalidWav(String),
    UnsupportedFormat(String),
    InvalidParameter(String),
    TooShort,
    Silence,
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::InvalidWav(msg) => write!(f, "invalid wav: {msg}"),
            AudioError::UnsupportedFormat(msg) => write!(f, "unsupported wav format: {msg}"),
            AudioError::InvalidParameter(msg) => write!(f, "invalid parameter: {msg}"),
            AudioError::TooShort => write!(f, "audio too short to estimate a tempo"),
            AudioError::Silence => write!(f, "no onsets found, audio is silent"),
        }
    }
}

impl std::error::Error for AudioError {}
