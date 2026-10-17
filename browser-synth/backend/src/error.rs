use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::fmt;

use crate::dsp::DspError;

#[derive(Debug)]
pub enum AppError {
    InvalidPreset(String),
    InvalidNotes(String),
    NotFound(String),
    StoreUnavailable,
    Dsp(DspError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidPreset(m) => write!(f, "invalid preset: {m}"),
            AppError::InvalidNotes(m) => write!(f, "invalid notes: {m}"),
            AppError::NotFound(name) => write!(f, "preset '{name}' not found"),
            AppError::StoreUnavailable => write!(f, "preset store unavailable"),
            AppError::Dsp(e) => write!(f, "render failed: {e}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<DspError> for AppError {
    fn from(e: DspError) -> Self {
        AppError::Dsp(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InvalidPreset(_) | AppError::InvalidNotes(_) | AppError::Dsp(_) => {
                StatusCode::BAD_REQUEST
            }
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::StoreUnavailable => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
