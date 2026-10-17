use crate::error::FingerprintError;
use crate::library::{Identification, Library, TrackInfo};
use crate::wav;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;

pub type Shared = Arc<RwLock<Library>>;

const MAX_BODY: usize = 64 * 1024 * 1024;

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for FingerprintError {
    fn into_response(self) -> Response {
        let status = match self {
            FingerprintError::Io(_) | FingerprintError::LockPoisoned => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            FingerprintError::UnknownTrack(_) => StatusCode::NOT_FOUND,
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

#[derive(Debug, Deserialize)]
struct AddParams {
    name: String,
}

#[derive(Debug, Deserialize)]
struct IdentifyParams {
    rate: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ClipParams {
    track: usize,
    start: f64,
    len: f64,
    noise: Option<f32>,
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    tracks: usize,
    index_size: usize,
}

pub fn router(state: Shared) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/library", get(list).post(add))
        .route("/api/identify", post(identify))
        .route("/api/clip", get(clip))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

fn read_lib(state: &Shared) -> Result<std::sync::RwLockReadGuard<'_, Library>, FingerprintError> {
    state.read().map_err(|_| FingerprintError::LockPoisoned)
}

async fn health(State(state): State<Shared>) -> Result<Json<Health>, FingerprintError> {
    let lib = read_lib(&state)?;
    Ok(Json(Health {
        status: "ok",
        tracks: lib.list()?.len(),
        index_size: lib.index_size(),
    }))
}

async fn list(State(state): State<Shared>) -> Result<Json<Vec<TrackInfo>>, FingerprintError> {
    Ok(Json(read_lib(&state)?.list()?))
}

async fn add(
    State(state): State<Shared>,
    Query(params): Query<AddParams>,
    body: Bytes,
) -> Result<Json<TrackInfo>, FingerprintError> {
    let audio = wav::read(&body)?;
    let mut lib = state.write().map_err(|_| FingerprintError::LockPoisoned)?;
    Ok(Json(lib.add(&params.name, &audio)?))
}

fn decode_query(body: &[u8], rate: Option<u32>) -> Result<wav::Audio, FingerprintError> {
    if wav::is_wav(body) {
        return wav::read(body);
    }
    let rate = rate.ok_or_else(|| {
        FingerprintError::InvalidParameter("rate is required for raw Float32 PCM".into())
    })?;
    wav::read_f32_pcm(body, rate)
}

async fn identify(
    State(state): State<Shared>,
    Query(params): Query<IdentifyParams>,
    body: Bytes,
) -> Result<Json<Identification>, FingerprintError> {
    let audio = decode_query(&body, params.rate)?;
    Ok(Json(read_lib(&state)?.identify(&audio)?))
}

async fn clip(
    State(state): State<Shared>,
    Query(params): Query<ClipParams>,
) -> Result<Response, FingerprintError> {
    let audio = read_lib(&state)?.clip(
        params.track,
        params.start,
        params.len,
        params.noise.unwrap_or(0.0),
    )?;
    Ok(([(header::CONTENT_TYPE, "audio/wav")], wav::write(&audio)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_body_is_detected_without_rate() {
        let bytes = wav::write(&wav::Audio {
            sample_rate: 8000,
            samples: vec![0.1; 10],
        });
        assert_eq!(decode_query(&bytes, None).unwrap().sample_rate, 8000);
    }

    #[test]
    fn raw_pcm_without_rate_is_rejected() {
        let bytes: Vec<u8> = [0.1f32; 4].iter().flat_map(|v| v.to_le_bytes()).collect();
        assert!(decode_query(&bytes, None).is_err());
        assert_eq!(
            decode_query(&bytes, Some(48000)).unwrap().sample_rate,
            48000
        );
    }
}
