use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    MissingPort,
    InvalidPort(String),
    Io(std::io::Error),
    Wav(String),
    InvalidInput(String),
    NoFrame(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingPort => write!(f, "PORT environment variable is not set"),
            AppError::InvalidPort(value) => write!(f, "invalid PORT value: {value}"),
            AppError::Io(err) => write!(f, "io error: {err}"),
            AppError::Wav(msg) => write!(f, "wav error: {msg}"),
            AppError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            AppError::NoFrame(msg) => write!(f, "no frame decoded: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Wav(_) | AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::NoFrame(_) => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
