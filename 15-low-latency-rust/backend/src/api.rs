use crate::backend::DeviceList;
use crate::engine::{
    Engine, EngineStatus, MeasureResult, StartRequest, measure_blocking, with_engine,
};
use crate::error::EngineError;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub type Shared = Arc<Mutex<Engine>>;

pub struct ApiError(EngineError);

impl From<EngineError> for ApiError {
    fn from(e: EngineError) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            EngineError::InvalidConfig(_) => StatusCode::BAD_REQUEST,
            EngineError::NotRunning => StatusCode::CONFLICT,
            EngineError::NotDetected | EngineError::MeasureTimeout => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            EngineError::Device(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        (status, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}

pub fn router(engine: Shared) -> Router {
    Router::new()
        .route("/api/devices", get(devices))
        .route("/api/start", post(start))
        .route("/api/stop", post(stop))
        .route("/api/measure", post(measure))
        .route("/api/stats", get(stats))
        .route("/api/stats/stream", get(stats_stream))
        .with_state(engine)
}

async fn run<T: Send + 'static>(
    engine: Shared,
    f: impl FnOnce(&Mutex<Engine>) -> Result<T, EngineError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(move || f(&engine))
        .await
        .map_err(|e| EngineError::Device(format!("worker failed: {e}")))?
        .map_err(ApiError)
}

async fn devices(State(engine): State<Shared>) -> Result<Json<DeviceList>, ApiError> {
    run(engine, |e| with_engine(e, |e| Ok(e.devices())))
        .await
        .map(Json)
}

async fn start(
    State(engine): State<Shared>,
    payload: Result<Json<StartRequest>, JsonRejection>,
) -> Result<Json<EngineStatus>, ApiError> {
    let Json(req) = payload.map_err(|e| EngineError::InvalidConfig(e.body_text()))?;
    run(engine, move |e| with_engine(e, |e| e.start(&req)))
        .await
        .map(Json)
}

async fn stop(State(engine): State<Shared>) -> Result<Json<EngineStatus>, ApiError> {
    run(engine, |e| with_engine(e, |e| Ok(e.stop())))
        .await
        .map(Json)
}

async fn measure(State(engine): State<Shared>) -> Result<Json<MeasureResult>, ApiError> {
    run(engine, |e| measure_blocking(e, Duration::from_secs(3)))
        .await
        .map(Json)
}

async fn stats(State(engine): State<Shared>) -> Result<Json<EngineStatus>, ApiError> {
    run(engine, |e| with_engine(e, |e| Ok(e.status())))
        .await
        .map(Json)
}

async fn stats_stream(State(engine): State<Shared>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| push_stats(socket, engine))
}

async fn push_stats(mut socket: WebSocket, engine: Shared) {
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    loop {
        tick.tick().await;
        let payload = match with_engine(&engine, |e| Ok(e.status())) {
            Ok(status) => serde_json::to_string(&status),
            Err(e) => serde_json::to_string(&json!({ "error": e.to_string() })),
        };
        let Ok(text) = payload else { break };
        if socket.send(Message::Text(text.into())).await.is_err() {
            break;
        }
    }
}
