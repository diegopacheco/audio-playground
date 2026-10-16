use std::fmt;

#[derive(Debug, PartialEq)]
pub enum StegoError {
    InvalidWav(String),
    UnsupportedWav(String),
    CapacityExceeded { needed: usize, available: usize },
    NoPayload,
    InvalidLength { declared: usize, available: usize },
    CrcMismatch { stored: u32, computed: u32 },
    InvalidBits(u8),
    SampleRateTooLow(u32),
    InvalidBase64,
    InvalidUtf8,
    UnknownMethod(String),
}

impl fmt::Display for StegoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StegoError::InvalidWav(m) => write!(f, "invalid wav: {m}"),
            StegoError::UnsupportedWav(m) => write!(f, "unsupported wav: {m}"),
            StegoError::CapacityExceeded { needed, available } => write!(
                f,
                "message needs {needed} bytes but carrier holds only {available} bytes"
            ),
            StegoError::NoPayload => write!(f, "no hidden payload found (magic missing)"),
            StegoError::InvalidLength {
                declared,
                available,
            } => write!(
                f,
                "declared payload length {declared} exceeds available {available} bytes"
            ),
            StegoError::CrcMismatch { stored, computed } => write!(
                f,
                "payload corrupted: crc32 stored {stored:08x} computed {computed:08x}"
            ),
            StegoError::InvalidBits(b) => write!(f, "bits per sample must be 1-4, got {b}"),
            StegoError::SampleRateTooLow(r) => {
                write!(f, "sample rate {r} Hz too low for ultrasonic band")
            }
            StegoError::InvalidBase64 => write!(f, "invalid base64 wav data"),
            StegoError::InvalidUtf8 => write!(f, "hidden payload is not valid utf-8"),
            StegoError::UnknownMethod(m) => write!(f, "unknown method {m}"),
        }
    }
}

impl std::error::Error for StegoError {}

pub type Result<T> = std::result::Result<T, StegoError>;
