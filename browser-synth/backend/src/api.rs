use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::dsp::SAMPLE_RATE;
use crate::error::AppError;
use crate::preset::Preset;
use crate::render::{Note, render};
use crate::store::PresetStore;
use crate::wav::encode_wav;

type AppState = Arc<PresetStore>;

#[derive(Deserialize)]
pub struct RenderRequest {
    preset: Preset,
    notes: Vec<Note>,
}

pub fn router(store: PresetStore) -> Router {
    Router::new()
        .route("/api/presets", get(list_presets).post(save_preset))
        .route("/api/presets/{name}", delete(delete_preset))
        .route("/api/render", post(render_phrase))
        .with_state(Arc::new(store))
}

async fn list_presets(State(store): State<AppState>) -> Result<Json<Vec<Preset>>, AppError> {
    Ok(Json(store.list()?))
}

async fn save_preset(
    State(store): State<AppState>,
    Json(preset): Json<Preset>,
) -> Result<impl IntoResponse, AppError> {
    let saved = store.save(preset)?;
    Ok((StatusCode::CREATED, Json(saved)))
}

async fn delete_preset(
    State(store): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    store.delete(&name)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn render_phrase(Json(req): Json<RenderRequest>) -> Result<impl IntoResponse, AppError> {
    let samples = render(&req.preset, &req.notes, SAMPLE_RATE)?;
    Ok((
        [(header::CONTENT_TYPE, "audio/wav")],
        encode_wav(&samples, SAMPLE_RATE),
    ))
}
