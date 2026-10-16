use crate::error::AppError;
use crate::pattern::Pattern;
use crate::sequencer::{self, RenderSettings};
use crate::store::PatternStore;
use crate::wav::encode_wav;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const SAMPLE_RATE: u32 = 44100;

#[derive(Serialize)]
struct PatternList {
    names: Vec<String>,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Deserialize)]
struct RenderRequest {
    pattern: Pattern,
    bpm: f32,
    bars: u32,
    swing: f32,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InvalidPattern(_) | AppError::InvalidRender(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

pub fn router(store: Arc<PatternStore>) -> Router {
    Router::new()
        .route("/api/patterns", get(list_patterns))
        .route("/api/patterns/{name}", get(get_pattern).put(put_pattern))
        .route("/api/render", post(render))
        .with_state(store)
}

async fn list_patterns(
    State(store): State<Arc<PatternStore>>,
) -> Result<Json<PatternList>, AppError> {
    Ok(Json(PatternList {
        names: store.names()?,
    }))
}

async fn get_pattern(
    State(store): State<Arc<PatternStore>>,
    Path(name): Path<String>,
) -> Result<Json<Pattern>, AppError> {
    Ok(Json(store.get(&name)?))
}

async fn put_pattern(
    State(store): State<Arc<PatternStore>>,
    Path(name): Path<String>,
    Json(pattern): Json<Pattern>,
) -> Result<Json<Pattern>, AppError> {
    store.put(&name, pattern.clone())?;
    Ok(Json(pattern))
}

async fn render(Json(req): Json<RenderRequest>) -> Result<Response, AppError> {
    let settings = RenderSettings {
        bpm: req.bpm,
        bars: req.bars,
        swing: req.swing,
    };
    let samples = sequencer::render(&req.pattern, &settings, SAMPLE_RATE)?;
    let wav = encode_wav(&samples, SAMPLE_RATE);
    Ok(([(header::CONTENT_TYPE, "audio/wav")], wav).into_response())
}
