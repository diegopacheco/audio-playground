use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AppError {
    InvalidPattern(String),
    InvalidRender(String),
    NotFound(String),
    Io(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidPattern(m) => write!(f, "invalid pattern: {m}"),
            AppError::InvalidRender(m) => write!(f, "invalid render request: {m}"),
            AppError::NotFound(m) => write!(f, "pattern not found: {m}"),
            AppError::Io(m) => write!(f, "io error: {m}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}
