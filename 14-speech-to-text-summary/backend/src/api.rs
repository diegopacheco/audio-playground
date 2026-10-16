use crate::error::AppError;
use crate::service::transcribe_bytes;
use crate::summarizer::{Mode, Summarizer};
use crate::transcriber::Transcriber;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub transcriber: Arc<dyn Transcriber>,
    pub summarizer: Arc<dyn Summarizer>,
}

#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub text: String,
    pub mode: Mode,
}

pub fn status_of(err: &AppError) -> StatusCode {
    match err {
        AppError::InvalidWav(_) | AppError::Decode(_) | AppError::BadRequest(_) => {
            StatusCode::BAD_REQUEST
        }
        AppError::MissingApiKey => StatusCode::SERVICE_UNAVAILABLE,
        AppError::Claude(_) => StatusCode::BAD_GATEWAY,
        AppError::Io(_) | AppError::Whisper(_) | AppError::WhisperOutput(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (status_of(&self), Json(json!({ "error": self.to_string() }))).into_response()
    }
}

async fn blocking<T, F>(f: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "whisper": state.transcriber.binary_available(),
        "model": state.transcriber.model_available(),
        "claude_key": state.summarizer.has_key(),
    }))
}

async fn transcribe(State(state): State<AppState>, body: Bytes) -> Response {
    let transcriber = state.transcriber.clone();
    match blocking(move || transcribe_bytes(transcriber.as_ref(), &body)).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn summarize(State(state): State<AppState>, Json(req): Json<SummarizeRequest>) -> Response {
    let summarizer = state.summarizer.clone();
    match blocking(move || summarizer.summarize(&req.text, req.mode)).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/transcribe", post(transcribe))
        .route("/api/summarize", post(summarize))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_key_maps_to_503_and_client_errors_to_400() {
        assert_eq!(
            status_of(&AppError::MissingApiKey),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            status_of(&AppError::Decode("x".into())),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_of(&AppError::Claude("x".into())),
            StatusCode::BAD_GATEWAY
        );
    }

    #[test]
    fn missing_key_message_names_the_variable_but_never_a_value() {
        let msg = AppError::MissingApiKey.to_string();
        assert!(msg.contains("ANTHROPIC_API_KEY"));
        assert!(!msg.contains("sk-"));
    }
}
