use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Config(String),
    Io(std::io::Error),
    InvalidKey(String),
    InvalidScale(String),
    InvalidNumeral(String),
    InvalidRequest(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Config(message) => write!(f, "config error: {message}"),
            AppError::Io(err) => write!(f, "io error: {err}"),
            AppError::InvalidKey(key) => write!(f, "invalid key: {key}"),
            AppError::InvalidScale(scale) => write!(f, "invalid scale: {scale}"),
            AppError::InvalidNumeral(numeral) => write!(f, "invalid roman numeral: {numeral}"),
            AppError::InvalidRequest(message) => write!(f, "invalid request: {message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}
