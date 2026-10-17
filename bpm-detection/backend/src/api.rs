use crate::analysis::{Analysis, Analyzer, DetectorKind};
use crate::error::AudioError;
use crate::synth::drum_loop;
use crate::wav::{read_wav, write_wav_16};
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

const MAX_BODY: usize = 100 * 1024 * 1024;
const SYNTH_RATE: u32 = 44100;
const SYNTH_SECONDS: f32 = 12.0;

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AudioError {
    fn into_response(self) -> Response {
        let status = match self {
            AudioError::Silence | AudioError::TooShort => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::BAD_REQUEST,
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

#[derive(Deserialize)]
struct AnalyzeQuery {
    #[serde(default)]
    detector: DetectorKind,
}

#[derive(Deserialize)]
struct SyntheticQuery {
    bpm: f32,
}

async fn analyze(
    Query(query): Query<AnalyzeQuery>,
    body: Bytes,
) -> Result<Json<Analysis>, AudioError> {
    let audio = read_wav(&body)?;
    Analyzer::new(query.detector).analyze(&audio).map(Json)
}

async fn synthetic(Query(query): Query<SyntheticQuery>) -> Result<Response, AudioError> {
    let samples = drum_loop(query.bpm, SYNTH_SECONDS, SYNTH_RATE)?;
    Ok((
        [(header::CONTENT_TYPE, "audio/wav")],
        write_wav_16(&samples, SYNTH_RATE),
    )
        .into_response())
}

pub fn router() -> Router {
    Router::new()
        .route("/api/analyze", post(analyze))
        .route("/api/synthetic", get(synthetic))
        .layer(DefaultBodyLimit::max(MAX_BODY))
}
