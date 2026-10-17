use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::StegoError;
use crate::wav::{self, Audio};
use crate::{base64, carrier, metrics, stego};

#[derive(Clone)]
pub struct AppState {
    carrier: Arc<Audio>,
}

#[derive(Deserialize)]
pub struct MethodQuery {
    method: String,
    bits: Option<u8>,
}

#[derive(Deserialize)]
pub struct EncodeRequest {
    wav: Option<String>,
    message: String,
    method: String,
    bits: Option<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapacityResponse {
    method: String,
    bits: u8,
    bytes: usize,
    samples: usize,
    sample_rate: u32,
    seconds: f64,
}

#[derive(Serialize)]
pub struct DecodeResponse {
    message: String,
    bytes: usize,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

impl IntoResponse for StegoError {
    fn into_response(self) -> Response {
        let status = match self {
            StegoError::CapacityExceeded { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            StegoError::NoPayload
            | StegoError::CrcMismatch { .. }
            | StegoError::InvalidLength { .. }
            | StegoError::InvalidUtf8 => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::BAD_REQUEST,
        };
        (
            status,
            Json(ErrorResponse {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

fn wav_response(audio: &Audio, extra: Vec<(&'static str, String)>) -> Response {
    let mut response = (
        [(header::CONTENT_TYPE, "audio/wav".to_string())],
        wav::write(audio),
    )
        .into_response();
    for (name, value) in extra {
        if let Ok(v) = value.parse() {
            response.headers_mut().insert(name, v);
        }
    }
    response
}

fn audio_or_carrier(state: &AppState, body: &[u8]) -> Result<Audio, StegoError> {
    if body.is_empty() {
        Ok(state.carrier.as_ref().clone())
    } else {
        wav::read(body)
    }
}

fn capacity_of(audio: &Audio, q: &MethodQuery) -> Result<CapacityResponse, StegoError> {
    let bits = q.bits.unwrap_or(1);
    let bytes = stego::build(&q.method, bits)?.capacity(audio)?;
    Ok(CapacityResponse {
        method: q.method.clone(),
        bits,
        bytes,
        samples: audio.samples.len(),
        sample_rate: audio.sample_rate,
        seconds: audio.samples.len() as f64 / audio.sample_rate.max(1) as f64,
    })
}

async fn get_carrier(State(state): State<AppState>) -> Response {
    wav_response(&state.carrier, Vec::new())
}

async fn get_capacity(
    State(state): State<AppState>,
    Query(q): Query<MethodQuery>,
) -> Result<Json<CapacityResponse>, StegoError> {
    Ok(Json(capacity_of(&state.carrier, &q)?))
}

async fn post_capacity(
    State(state): State<AppState>,
    Query(q): Query<MethodQuery>,
    body: Bytes,
) -> Result<Json<CapacityResponse>, StegoError> {
    let audio = audio_or_carrier(&state, &body)?;
    Ok(Json(capacity_of(&audio, &q)?))
}

async fn encode(
    State(state): State<AppState>,
    Json(req): Json<EncodeRequest>,
) -> Result<Response, StegoError> {
    let carrier = match &req.wav {
        Some(data) if !data.is_empty() => wav::read(&base64::decode(data)?)?,
        _ => state.carrier.as_ref().clone(),
    };
    let method = stego::build(&req.method, req.bits.unwrap_or(1))?;
    let encoded = method.encode(&carrier, req.message.as_bytes())?;
    let snr = metrics::snr_db(&carrier.samples, &encoded.samples);
    let headers = vec![
        ("x-snr-db", format!("{snr:.2}")),
        (
            "x-max-diff",
            metrics::max_abs_diff(&carrier.samples, &encoded.samples).to_string(),
        ),
        ("x-capacity", method.capacity(&carrier)?.to_string()),
        ("x-payload-bytes", req.message.len().to_string()),
    ];
    Ok(wav_response(&encoded, headers))
}

async fn decode(
    Query(q): Query<MethodQuery>,
    body: Bytes,
) -> Result<Json<DecodeResponse>, StegoError> {
    let audio = wav::read(&body)?;
    let payload = stego::build(&q.method, q.bits.unwrap_or(1))?.decode(&audio)?;
    let bytes = payload.len();
    let message = String::from_utf8(payload).map_err(|_| StegoError::InvalidUtf8)?;
    Ok(Json(DecodeResponse { message, bytes }))
}

pub fn router() -> Router {
    let state = AppState {
        carrier: Arc::new(carrier::generate()),
    };
    Router::new()
        .route("/api/carrier", get(get_carrier))
        .route("/api/capacity", get(get_capacity).post(post_capacity))
        .route("/api/encode", post(encode))
        .route("/api/decode", post(decode))
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024))
        .with_state(state)
}
