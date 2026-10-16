use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AppError {
    InvalidBpm(f64),
    InvalidBeatsPerBar(u32),
    InvalidBars(u32),
    InvalidBeats(u32),
    TrackTooLong(f64),
    WavTooLarge(usize),
    RunTooLong(f64),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidBpm(v) => write!(f, "bpm must be between 20 and 300, got {v}"),
            AppError::InvalidBeatsPerBar(v) => {
                write!(f, "beats_per_bar must be between 1 and 16, got {v}")
            }
            AppError::InvalidBars(v) => write!(f, "bars must be between 1 and 64, got {v}"),
            AppError::InvalidBeats(v) => write!(f, "beats must be between 2 and 64, got {v}"),
            AppError::TrackTooLong(s) => {
                write!(f, "click track must be at most 300 seconds, got {s:.1}")
            }
            AppError::WavTooLarge(n) => write!(f, "wav payload too large: {n} samples"),
            AppError::RunTooLong(s) => {
                write!(f, "drift run must be at most 30 seconds, got {s:.1}")
            }
        }
    }
}

impl std::error::Error for AppError {}
