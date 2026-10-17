use crate::error::TunerError;
use crate::note;
use crate::pitch;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectRequest {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub detector: String,
    pub a4: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DetectResponse {
    pub frequency: Option<f64>,
    pub note: Option<&'static str>,
    pub octave: Option<i32>,
    pub cents: Option<f64>,
    pub clarity: f64,
    pub detector: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for TunerError {
    fn into_response(self) -> Response {
        let status = match self {
            TunerError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
        .route("/api/detect", post(detect_handler))
        .route("/api/stream", get(stream_handler))
        .layer(CorsLayer::permissive())
}

pub fn analyze(request: &DetectRequest) -> Result<DetectResponse, TunerError> {
    let detector = pitch::by_name(&request.detector)?;
    if !request.a4.is_finite() || request.a4 <= 0.0 {
        return Err(TunerError::InvalidReference(request.a4));
    }
    let detected = detector.detect(&request.samples, request.sample_rate)?;
    let Some(found) = detected else {
        return Ok(DetectResponse {
            frequency: None,
            note: None,
            octave: None,
            cents: None,
            clarity: 0.0,
            detector: detector.name(),
        });
    };
    let mapped = note::from_frequency(found.frequency, request.a4)?;
    Ok(DetectResponse {
        frequency: Some(found.frequency),
        note: Some(mapped.name),
        octave: Some(mapped.octave),
        cents: Some(mapped.cents),
        clarity: found.clarity,
        detector: detector.name(),
    })
}

async fn health() -> &'static str {
    "ok"
}

async fn detect_handler(
    Json(request): Json<DetectRequest>,
) -> Result<Json<DetectResponse>, TunerError> {
    analyze(&request).map(Json)
}

async fn stream_handler(upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(stream)
}

async fn stream(mut socket: WebSocket) {
    while let Some(Ok(message)) = socket.recv().await {
        let Message::Text(text) = message else {
            continue;
        };
        let reply = match serde_json::from_str::<DetectRequest>(&text) {
            Ok(request) => match analyze(&request) {
                Ok(response) => serde_json::to_string(&response),
                Err(err) => serde_json::to_string(&ErrorBody {
                    error: err.to_string(),
                }),
            },
            Err(err) => serde_json::to_string(&ErrorBody {
                error: err.to_string(),
            }),
        };
        let Ok(body) = reply else {
            break;
        };
        if socket.send(Message::Text(body.into())).await.is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pitch::signals::{RATE, sine};

    fn request(samples: Vec<f32>, detector: &str, a4: f64) -> DetectRequest {
        DetectRequest {
            samples,
            sample_rate: RATE,
            detector: detector.to_string(),
            a4,
        }
    }

    #[test]
    fn analyze_maps_detected_pitch_to_note_and_names_the_detector() {
        let response = analyze(&request(sine(440.0, 0.5), "yin", 440.0)).unwrap();
        assert_eq!(response.note, Some("A"));
        assert_eq!(response.octave, Some(4));
        assert!(response.cents.unwrap().abs() < 1.0);
        assert_eq!(response.detector, "yin");
    }

    #[test]
    fn analyze_returns_empty_reading_for_silence() {
        let response = analyze(&request(vec![0.0; 2048], "autocorrelation", 440.0)).unwrap();
        assert_eq!(response.frequency, None);
        assert_eq!(response.note, None);
        assert_eq!(response.detector, "autocorrelation");
    }

    #[test]
    fn analyze_rejects_bad_reference_and_unknown_detector() {
        assert_eq!(
            analyze(&request(sine(440.0, 0.5), "yin", 0.0)),
            Err(TunerError::InvalidReference(0.0))
        );
        assert!(matches!(
            analyze(&request(sine(440.0, 0.5), "zcr", 440.0)),
            Err(TunerError::UnknownDetector(_))
        ));
    }
}
