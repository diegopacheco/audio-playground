use std::fmt;

#[derive(Debug)]
pub enum AppError {
    UnknownWaveform(String),
    InvalidFrequency(f64),
    InvalidDuration(u32),
    InvalidAmplitude(f64),
    InvalidSampleRate(u32),
    MissingPort,
    InvalidPort(String),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::UnknownWaveform(name) => write!(f, "unknown waveform '{name}'"),
            AppError::InvalidFrequency(hz) => {
                write!(
                    f,
                    "frequency {hz} Hz must be between 20 Hz and half the sample rate"
                )
            }
            AppError::InvalidDuration(ms) => {
                write!(f, "duration {ms} ms must be between 1 and 10000 ms")
            }
            AppError::InvalidAmplitude(a) => write!(f, "amplitude {a} must be between 0 and 1"),
            AppError::InvalidSampleRate(rate) => write!(
                f,
                "sample rate {rate} must be one of 8000, 16000, 22050, 44100, 48000"
            ),
            AppError::MissingPort => write!(f, "PORT environment variable is not set"),
            AppError::InvalidPort(value) => write!(f, "PORT '{value}' is not a valid port"),
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
