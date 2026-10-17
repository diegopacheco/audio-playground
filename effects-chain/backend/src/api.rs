use axum::extract::DefaultBodyLimit;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::base64;
use crate::effects::registry::{self, EffectConfig, EffectSpec};
use crate::error::AppError;
use crate::source;
use crate::wav::{self, Audio};

const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

#[derive(Deserialize)]
pub struct ProcessRequest {
    pub wav_base64: String,
    #[serde(default)]
    pub chain: Vec<EffectConfig>,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

fn wav_response(audio: &Audio) -> Response {
    ([(header::CONTENT_TYPE, "audio/wav")], wav::write(audio)).into_response()
}

async fn effects() -> Json<&'static [EffectSpec]> {
    Json(registry::SPECS)
}

async fn source_clip() -> Response {
    wav_response(&source::plucked_tones())
}

pub fn process(request: &ProcessRequest) -> Result<Audio, AppError> {
    let input = wav::read(&base64::decode(&request.wav_base64)?)?;
    let mut chain = registry::build_chain(&request.chain, input.sample_rate)?;
    Ok(Audio {
        sample_rate: input.sample_rate,
        samples: chain.render(&input.samples),
    })
}

async fn process_handler(Json(request): Json<ProcessRequest>) -> Result<Response, AppError> {
    let audio = tokio::task::spawn_blocking(move || process(&request))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e)))??;
    Ok(wav_response(&audio))
}

pub fn router() -> Router {
    Router::new()
        .route("/api/effects", get(effects))
        .route("/api/source", get(source_clip))
        .route("/api/process", post(process_handler))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn request(samples: Vec<f32>, chain: Vec<EffectConfig>) -> ProcessRequest {
        let bytes = wav::write(&Audio {
            sample_rate: 1000,
            samples,
        });
        ProcessRequest {
            wav_base64: base64::encode(&bytes),
            chain,
        }
    }

    #[test]
    fn process_decodes_runs_the_chain_and_keeps_the_sample_rate() {
        let mut input = vec![0.0; 20];
        input[0] = 1.0;
        let params = HashMap::from([("time_ms".to_string(), 5.0), ("mix".to_string(), 1.0)]);
        let out = process(&request(
            input,
            vec![EffectConfig {
                kind: "delay".into(),
                params,
            }],
        ))
        .unwrap();
        assert_eq!(out.sample_rate, 1000);
        assert!(out.samples[0].abs() < 1e-3);
        assert!(out.samples[5] > 0.99);
    }

    #[test]
    fn process_reports_unknown_effects_as_errors() {
        let result = process(&request(
            vec![0.0; 4],
            vec![EffectConfig {
                kind: "wah".into(),
                params: HashMap::new(),
            }],
        ));
        assert!(matches!(result, Err(AppError::UnknownEffect(_))));
    }
}
