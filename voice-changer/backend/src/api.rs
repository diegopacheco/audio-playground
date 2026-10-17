use crate::effects::{self, CATALOG, EffectSpec, VoiceEffect};
use crate::error::VoiceError;
use crate::signal;
use crate::wav::{self, Wav};
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{DefaultBodyLimit, Query};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

const DEFAULT_RATE: u32 = 48000;
const DEFAULT_SECONDS: f32 = 4.0;
const MAX_BODY: usize = 64 * 1024 * 1024;
const MAX_BLOCK_SAMPLES: usize = 48000;

#[derive(Debug, Deserialize)]
pub struct VoiceQuery {
    pub effect: String,
    pub semitones: Option<f32>,
    pub rate: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessQuery {
    pub effect: String,
    pub semitones: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct SignalQuery {
    pub rate: Option<u32>,
    pub seconds: Option<f32>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub effect: &'static str,
    pub semitones: f32,
    pub rate: u32,
    pub latency_samples: usize,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for VoiceError {
    fn into_response(self) -> Response {
        let status = match self {
            VoiceError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/effects", get(list_effects))
        .route("/api/voice", get(voice_handler))
        .route("/api/process", post(process_handler))
        .route("/api/test-signal", get(test_signal_handler))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .layer(CorsLayer::permissive())
}

pub fn open_session(query: &VoiceQuery) -> Result<(Box<dyn VoiceEffect>, SessionInfo), VoiceError> {
    let spec = effects::spec(&query.effect)?;
    let rate = query.rate.unwrap_or(DEFAULT_RATE);
    if !(8000..=192_000).contains(&rate) {
        return Err(VoiceError::InvalidSampleRate(rate));
    }
    let semitones = query
        .semitones
        .unwrap_or(spec.default_semitones)
        .clamp(spec.min_semitones, spec.max_semitones);
    let effect = effects::create(spec.name, Some(semitones), rate)?;
    let info = SessionInfo {
        effect: spec.name,
        semitones,
        rate,
        latency_samples: effect.latency(),
    };
    Ok((effect, info))
}

pub fn decode_block(bytes: &[u8]) -> Result<Vec<f32>, VoiceError> {
    if !bytes.len().is_multiple_of(4) || bytes.len() / 4 > MAX_BLOCK_SAMPLES {
        return Err(VoiceError::InvalidBlock(bytes.len()));
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect())
}

pub fn encode_block(samples: &[f32]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

pub fn process_wav(bytes: &[u8], query: &ProcessQuery) -> Result<Vec<u8>, VoiceError> {
    let input = wav::decode(bytes)?;
    let spec = effects::spec(&query.effect)?;
    let semitones = query
        .semitones
        .unwrap_or(spec.default_semitones)
        .clamp(spec.min_semitones, spec.max_semitones);
    let mut effect = effects::create(spec.name, Some(semitones), input.sample_rate)?;
    let samples = effects::render(effect.as_mut(), &input.samples);
    Ok(wav::encode(&Wav {
        sample_rate: input.sample_rate,
        samples,
    }))
}

fn wav_response(bytes: Vec<u8>) -> Response {
    ([(header::CONTENT_TYPE, "audio/wav")], bytes).into_response()
}

async fn health() -> &'static str {
    "ok"
}

async fn list_effects() -> Json<Vec<EffectSpec>> {
    Json(CATALOG.to_vec())
}

async fn process_handler(
    Query(query): Query<ProcessQuery>,
    body: Bytes,
) -> Result<Response, VoiceError> {
    process_wav(&body, &query).map(wav_response)
}

async fn test_signal_handler(Query(query): Query<SignalQuery>) -> Result<Response, VoiceError> {
    let sample_rate = query.rate.unwrap_or(DEFAULT_RATE);
    let samples = signal::voice(sample_rate, query.seconds.unwrap_or(DEFAULT_SECONDS))?;
    Ok(wav_response(wav::encode(&Wav {
        sample_rate,
        samples,
    })))
}

async fn voice_handler(
    Query(query): Query<VoiceQuery>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, VoiceError> {
    let (effect, info) = open_session(&query)?;
    Ok(upgrade.on_upgrade(move |socket| stream(socket, effect, info)))
}

fn error_text(err: &VoiceError) -> String {
    serde_json::to_string(&ErrorBody {
        error: err.to_string(),
    })
    .unwrap_or_else(|_| "{\"error\":\"internal\"}".to_string())
}

async fn stream(mut socket: WebSocket, mut effect: Box<dyn VoiceEffect>, info: SessionInfo) {
    let Ok(hello) = serde_json::to_string(&info) else {
        return;
    };
    if socket.send(Message::Text(hello.into())).await.is_err() {
        return;
    }
    while let Some(Ok(message)) = socket.recv().await {
        let reply = match message {
            Message::Binary(bytes) => match decode_block(&bytes) {
                Ok(block) => Message::Binary(encode_block(&effect.process_block(&block)).into()),
                Err(err) => Message::Text(error_text(&err).into()),
            },
            Message::Close(_) => break,
            _ => continue,
        };
        if socket.send(reply).await.is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(effect: &str, semitones: Option<f32>) -> VoiceQuery {
        VoiceQuery {
            effect: effect.to_string(),
            semitones,
            rate: Some(44100),
        }
    }

    #[test]
    fn session_clamps_semitones_to_the_effect_range_and_reports_latency() {
        let (_, info) = open_session(&query("deep", Some(3.0))).unwrap();
        assert_eq!(info.semitones, -5.0);
        assert_eq!(info.rate, 44100);
        assert!(info.latency_samples > 0);
    }

    #[test]
    fn session_rejects_unknown_effects() {
        assert!(matches!(
            open_session(&query("flanger", None)),
            Err(VoiceError::UnknownEffect(_))
        ));
    }

    #[test]
    fn binary_blocks_round_trip_as_little_endian_float32() {
        let samples = vec![0.25, -1.0, 0.75];
        assert_eq!(decode_block(&encode_block(&samples)).unwrap(), samples);
        assert_eq!(decode_block(&[0, 1, 2]), Err(VoiceError::InvalidBlock(3)));
    }

    #[test]
    fn process_wav_returns_a_wav_with_the_same_rate_and_length() {
        let input = Wav {
            sample_rate: 16000,
            samples: signal::voice(16000, 1.0).unwrap(),
        };
        let body = process_wav(
            &wav::encode(&input),
            &ProcessQuery {
                effect: "chipmunk".into(),
                semitones: None,
            },
        )
        .unwrap();
        let output = wav::decode(&body).unwrap();
        assert_eq!(output.sample_rate, 16000);
        assert_eq!(output.samples.len(), input.samples.len());
        assert!(output.samples.iter().any(|s| s.abs() > 0.05));
    }
}
