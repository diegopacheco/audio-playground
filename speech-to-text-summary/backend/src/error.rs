use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    InvalidWav(String),
    Decode(String),
    Whisper(String),
    WhisperOutput(String),
    MissingApiKey,
    Claude(String),
    BadRequest(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "io error: {e}"),
            AppError::InvalidWav(m) => write!(f, "invalid wav: {m}"),
            AppError::Decode(m) => write!(f, "audio decode failed: {m}"),
            AppError::Whisper(m) => write!(f, "whisper failed: {m}"),
            AppError::WhisperOutput(m) => write!(f, "unexpected whisper output: {m}"),
            AppError::MissingApiKey => write!(
                f,
                "ANTHROPIC_API_KEY is not set on the backend, summaries are disabled"
            ),
            AppError::Claude(m) => write!(f, "claude api error: {m}"),
            AppError::BadRequest(m) => write!(f, "bad request: {m}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}
