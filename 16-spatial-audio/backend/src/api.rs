use axum::extract::Query;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::audio::{SAMPLE_RATE, Stereo};
use crate::error::AppError;
use crate::source::{self, SourceKind};
use crate::spatial::cues::{self, CueReport};
use crate::spatial::{self, Method};
use crate::trajectory::{self, Position, Trajectory};
use crate::wav;

const MIN_DURATION: f64 = 0.5;
const MAX_DURATION: f64 = 20.0;
const PATH_RATE: f64 = 30.0;

#[derive(Deserialize)]
pub struct RenderRequest {
    pub source: SourceKind,
    pub trajectory: Trajectory,
    pub duration: f64,
    pub method: Method,
}

#[derive(Deserialize)]
pub struct PathRequest {
    pub trajectory: Trajectory,
    pub duration: f64,
}

#[derive(Deserialize)]
pub struct CueQuery {
    pub azimuth: f64,
    #[serde(default)]
    pub elevation: f64,
    #[serde(default = "default_distance")]
    pub distance: f64,
}

fn default_distance() -> f64 {
    1.0
}

#[derive(Serialize)]
struct TrajectoryView {
    label: &'static str,
    description: &'static str,
    defaults: Trajectory,
}

#[derive(Serialize)]
struct PathPoint {
    t: f64,
    #[serde(flatten)]
    cues: CueReport,
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

fn check_duration(duration: f64) -> Result<(), AppError> {
    if (MIN_DURATION..=MAX_DURATION).contains(&duration) {
        Ok(())
    } else {
        Err(AppError::InvalidParam(format!(
            "duration must be within {MIN_DURATION}..{MAX_DURATION} s"
        )))
    }
}

pub fn render(request: &RenderRequest) -> Result<Stereo, AppError> {
    check_duration(request.duration)?;
    let mono = source::generate(request.source, SAMPLE_RATE, request.duration);
    spatial::spatializer(request.method).render(&mono, &request.trajectory)
}

async fn render_handler(Json(request): Json<RenderRequest>) -> Result<Response, AppError> {
    let bytes = wav::write_stereo(&render(&request)?)?;
    Ok(([(header::CONTENT_TYPE, "audio/wav")], bytes).into_response())
}

async fn trajectories() -> Json<Vec<TrajectoryView>> {
    Json(
        trajectory::catalog()
            .into_iter()
            .map(|t| TrajectoryView {
                label: t.label,
                description: t.description,
                defaults: t.defaults,
            })
            .collect(),
    )
}

async fn itd(Query(q): Query<CueQuery>) -> Result<Json<CueReport>, AppError> {
    if !(-90.0..=90.0).contains(&q.elevation) || q.distance < 0.3 || !q.azimuth.is_finite() {
        return Err(AppError::InvalidParam(
            "azimuth must be a number, elevation within -90..90, distance >= 0.3".into(),
        ));
    }
    Ok(Json(cues::report(&Position {
        azimuth: q.azimuth,
        elevation: q.elevation,
        distance: q.distance,
    })))
}

async fn path(Json(request): Json<PathRequest>) -> Result<Json<Vec<PathPoint>>, AppError> {
    check_duration(request.duration)?;
    request.trajectory.validate()?;
    let steps = (request.duration * PATH_RATE).round() as usize;
    let points = (0..=steps)
        .map(|i| {
            let t = i as f64 / PATH_RATE;
            PathPoint {
                t,
                cues: cues::report(&request.trajectory.position(t, request.duration)),
            }
        })
        .collect();
    Ok(Json(points))
}

pub fn router() -> Router {
    Router::new()
        .route("/api/render", post(render_handler))
        .route("/api/trajectories", get(trajectories))
        .route("/api/itd", get(itd))
        .route("/api/path", post(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_rejects_durations_outside_the_supported_window() {
        let request = RenderRequest {
            source: SourceKind::Tone,
            trajectory: Trajectory::Static {
                azimuth: 0.0,
                elevation: 0.0,
                distance: 1.0,
            },
            duration: 60.0,
            method: Method::Binaural,
        };
        assert!(render(&request).is_err());
    }

    #[test]
    fn render_produces_equal_length_channels_for_the_requested_duration() {
        let request = RenderRequest {
            source: SourceKind::VoiceLike,
            trajectory: Trajectory::Circle {
                period: 2.0,
                distance: 1.5,
            },
            duration: 1.0,
            method: Method::Hrtf,
        };
        let stereo = render(&request).unwrap();
        assert_eq!(stereo.left.len(), SAMPLE_RATE as usize);
        assert_eq!(stereo.right.len(), SAMPLE_RATE as usize);
    }
}
