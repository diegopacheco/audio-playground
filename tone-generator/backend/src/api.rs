use crate::error::AppError;
use crate::oscillator::Waveform;
use crate::tone::{self, SAMPLE_RATES, ToneRequest};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct WaveformsResponse {
    waveforms: Vec<&'static str>,
    sample_rates: [u32; 5],
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
        .route("/api/waveforms", get(waveforms))
        .route("/api/tone", post(tone_wav))
}

async fn waveforms() -> Json<WaveformsResponse> {
    Json(WaveformsResponse {
        waveforms: Waveform::ALL.iter().map(|w| w.name()).collect(),
        sample_rates: SAMPLE_RATES,
    })
}

async fn tone_wav(Json(req): Json<ToneRequest>) -> Result<Response, AppError> {
    let bytes = tone::generate(&req)?;
    Ok(([(header::CONTENT_TYPE, "audio/wav")], bytes).into_response())
}
