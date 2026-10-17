use crate::dsp::fft::Radix2Fft;
use crate::dsp::spectrum::{SpectrumFrame, analyze};
use crate::dsp::window::window_by_name;
use crate::error::DspError;
use crate::pcm::decode_f32_le;
use axum::extract::Query;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SpectrumRequest {
    pub samples: Vec<f32>,
    pub sample_rate: f32,
    pub window: String,
    pub bars: usize,
}

#[derive(Debug, Deserialize)]
pub struct StreamParams {
    pub sample_rate: f32,
    pub window: String,
    pub bars: usize,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for DspError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: self.to_string(),
        };
        (StatusCode::BAD_REQUEST, Json(body)).into_response()
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/spectrum", post(spectrum))
        .route("/api/stream", get(stream))
}

async fn health() -> &'static str {
    "ok"
}

fn run(
    samples: &[f32],
    sample_rate: f32,
    window: &str,
    bars: usize,
) -> Result<SpectrumFrame, DspError> {
    let window = window_by_name(window)?;
    analyze(&Radix2Fft, window.as_ref(), samples, sample_rate, bars)
}

async fn spectrum(Json(req): Json<SpectrumRequest>) -> Result<Json<SpectrumFrame>, DspError> {
    run(&req.samples, req.sample_rate, &req.window, req.bars).map(Json)
}

async fn stream(ws: WebSocketUpgrade, Query(params): Query<StreamParams>) -> Response {
    if let Err(e) = window_by_name(&params.window) {
        return e.into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, params))
}

fn reply(bytes: &[u8], params: &StreamParams) -> String {
    let result =
        decode_f32_le(bytes).and_then(|s| run(&s, params.sample_rate, &params.window, params.bars));
    let json = match result {
        Ok(frame) => serde_json::to_string(&frame),
        Err(e) => serde_json::to_string(&ErrorBody {
            error: e.to_string(),
        }),
    };
    json.unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}

async fn handle_socket(mut socket: WebSocket, params: StreamParams) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Binary(bytes) => {
                let text = reply(&bytes, &params);
                if socket.send(Message::Text(text.into())).await.is_err() {
                    return;
                }
            }
            Message::Close(_) => return,
            _ => {}
        }
    }
}
