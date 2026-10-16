use std::fmt;

#[derive(Debug)]
pub enum AppError {
    InvalidParam(String),
    Config(String),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidParam(m) => write!(f, "invalid parameter: {m}"),
            AppError::Config(m) => write!(f, "configuration error: {m}"),
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
