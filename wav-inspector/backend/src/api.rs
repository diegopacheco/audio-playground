use crate::inspect::inspect;
use crate::render::AsciiRenderer;
use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde::Deserialize;
use serde_json::json;
use tower_http::cors::CorsLayer;

pub const ENVELOPE_POINTS: usize = 600;
const MAX_BODY: usize = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct InspectParams {
    pub width: Option<usize>,
    pub height: Option<usize>,
}

pub async fn inspect_handler(Query(params): Query<InspectParams>, body: Bytes) -> Response {
    let renderer = AsciiRenderer::new(params.width.unwrap_or(100), params.height.unwrap_or(16));
    match inspect(&body, &renderer, ENVELOPE_POINTS) {
        Ok(report) => Json(report).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/api/inspect", post(inspect_handler))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .layer(CorsLayer::permissive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::WavBuilder;
    use axum::body::to_bytes;
    use serde_json::Value;

    async fn call(width: Option<usize>, body: Vec<u8>) -> (StatusCode, Value) {
        let params = InspectParams {
            width,
            height: Some(6),
        };
        let res = inspect_handler(Query(params), Bytes::from(body)).await;
        let status = res.status();
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn valid_wav_returns_full_report_shape() {
        let bytes = WavBuilder::pcm16(8000, 2).samples(&[0.5; 2000]).build();
        let (status, v) = call(Some(20), bytes).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(v["header"]["sample_rate"], 8000);
        assert_eq!(v["header"]["channels"], 2);
        assert_eq!(v["chunks"].as_array().unwrap().len(), 2);
        assert_eq!(v["envelope"].as_array().unwrap().len(), ENVELOPE_POINTS);
        let ascii = v["ascii"].as_str().unwrap();
        assert_eq!(ascii.lines().next().unwrap().len(), 20);
        assert!(v["duration"].as_f64().unwrap() > 0.0);
        assert!(v["rms"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn malformed_body_returns_400_with_message() {
        let (status, v) = call(None, b"not a wav file at all".to_vec()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(v["error"].as_str().unwrap().contains("RIFF"));
    }

    #[tokio::test]
    async fn empty_body_returns_400() {
        let (status, v) = call(None, Vec::new()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(v["error"].as_str().unwrap().contains("too short"));
    }
}
