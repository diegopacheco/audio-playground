use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;

use crate::compose::{GENERATORS, GenerateRequest, MAX_BARS, MAX_BPM, MIN_BPM, Song, compose};
use crate::error::AppError;
use crate::render::Renderer;
use crate::render::midi::MidiRenderer;
use crate::render::wav::WavRenderer;
use crate::theory::{KEYS, SCALES};

const PROGRESSIONS: [&str; 6] = [
    "I-V-vi-IV",
    "I-IV-V-I",
    "ii-V-I-I",
    "I-vi-IV-V",
    "i-iv-v-i",
    "i-VI-III-VII",
];

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Serialize)]
struct Options {
    keys: &'static [&'static str],
    scales: &'static [&'static str],
    generators: &'static [&'static str],
    orders: [usize; 2],
    progressions: &'static [&'static str],
    max_bars: usize,
    min_bpm: f64,
    max_bpm: f64,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Io(_) | AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

async fn options() -> Json<Options> {
    Json(Options {
        keys: &KEYS,
        scales: &SCALES,
        generators: &GENERATORS,
        orders: [1, 2],
        progressions: &PROGRESSIONS,
        max_bars: MAX_BARS,
        min_bpm: MIN_BPM,
        max_bpm: MAX_BPM,
    })
}

async fn generate(Json(request): Json<GenerateRequest>) -> Result<Json<Song>, AppError> {
    Ok(Json(compose(&request)?))
}

fn export(
    renderer: &dyn Renderer,
    request: &GenerateRequest,
    extension: &str,
) -> Result<Response, AppError> {
    let bytes = renderer.render(&compose(request)?)?;
    let disposition = format!(
        "attachment; filename=\"generative-{}.{extension}\"",
        request.seed
    );
    Ok((
        [
            (header::CONTENT_TYPE, renderer.content_type().to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    )
        .into_response())
}

async fn export_midi(Json(request): Json<GenerateRequest>) -> Result<Response, AppError> {
    export(&MidiRenderer, &request, "mid")
}

async fn export_wav(Json(request): Json<GenerateRequest>) -> Result<Response, AppError> {
    export(&WavRenderer, &request, "wav")
}

pub fn router() -> Router {
    Router::new()
        .route("/api/options", get(options))
        .route("/api/generate", post(generate))
        .route("/api/export/midi", post(export_midi))
        .route("/api/export/wav", post(export_wav))
}
