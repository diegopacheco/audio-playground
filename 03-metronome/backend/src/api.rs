use crate::click::{SAMPLE_RATE, SineClick, TrackSpec, render_track};
use crate::drift::{DriftReport, measure};
use crate::error::AppError;
use crate::wav::encode_mono_16;
use axum::extract::Query;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ClickTrackRequest {
    pub bpm: f64,
    pub beats_per_bar: u32,
    pub bars: u32,
    pub accent: bool,
}

#[derive(Debug, Deserialize)]
pub struct DriftQuery {
    pub bpm: f64,
    pub beats: u32,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorBody {
            error: self.to_string(),
        });
        (StatusCode::BAD_REQUEST, body).into_response()
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/click-track", post(click_track))
        .route("/api/drift", get(drift))
}

async fn click_track(Json(req): Json<ClickTrackRequest>) -> Result<Response, AppError> {
    let spec = TrackSpec {
        bpm: req.bpm,
        beats_per_bar: req.beats_per_bar,
        bars: req.bars,
        accent: req.accent,
    };
    let track = render_track(&spec, &SineClick::default(), SAMPLE_RATE)?;
    let bytes = encode_mono_16(&track, SAMPLE_RATE)?;
    Ok(([(header::CONTENT_TYPE, "audio/wav")], bytes).into_response())
}

async fn drift(Query(q): Query<DriftQuery>) -> Result<Json<DriftReport>, AppError> {
    Ok(Json(measure(q.bpm, q.beats).await?))
}
