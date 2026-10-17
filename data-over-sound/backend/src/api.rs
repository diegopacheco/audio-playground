use crate::error::AppError;
use crate::modem::fsk::Fsk;
use crate::modem::receiver::Receiver;
use crate::modem::{self, Demodulator, ModemConfig, Scheme};
use crate::wav::{self, Audio};
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{DefaultBodyLimit, Query};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

const BODY_LIMIT: usize = 64 * 1024 * 1024;

#[derive(Deserialize)]
struct ModulateRequest {
    text: String,
    scheme: Scheme,
    baud: u32,
}

#[derive(Deserialize)]
struct DemodulateQuery {
    scheme: Scheme,
    baud: u32,
    sample_rate: Option<u32>,
}

#[derive(Deserialize)]
struct ListenQuery {
    scheme: Scheme,
    baud: u32,
    sample_rate: u32,
}

#[derive(Serialize)]
struct DemodulateResponse {
    text: String,
    crc_ok: bool,
    snr: f32,
}

#[derive(Serialize)]
struct SchemeInfo {
    scheme: Scheme,
    tones: Vec<f32>,
    bits_per_symbol: usize,
    min_baud: u32,
    max_baud: u32,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/schemes", get(schemes))
        .route("/api/modulate", post(modulate))
        .route("/api/demodulate", post(demodulate))
        .route("/api/listen", get(listen))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
}

async fn schemes() -> Json<Vec<SchemeInfo>> {
    Json(
        Scheme::ALL
            .iter()
            .map(|&scheme| {
                let fsk = match scheme {
                    Scheme::Bfsk => Fsk::bfsk(modem::TX_SAMPLE_RATE, 300),
                    Scheme::Fsk4 => Fsk::four_level(modem::TX_SAMPLE_RATE, 300),
                };
                SchemeInfo {
                    scheme,
                    tones: fsk.tones().to_vec(),
                    bits_per_symbol: Demodulator::bits_per_symbol(&fsk),
                    min_baud: modem::MIN_BAUD,
                    max_baud: modem::MAX_BAUD,
                }
            })
            .collect(),
    )
}

async fn modulate(Json(req): Json<ModulateRequest>) -> Result<Response, AppError> {
    let config = ModemConfig {
        scheme: req.scheme,
        baud: req.baud,
        sample_rate: modem::TX_SAMPLE_RATE,
    };
    let samples = modem::transmit(&config, req.text.as_bytes())?;
    let bytes = wav::encode(&Audio {
        sample_rate: config.sample_rate,
        samples,
    });
    Ok(([(header::CONTENT_TYPE, "audio/wav")], bytes).into_response())
}

async fn demodulate(
    Query(query): Query<DemodulateQuery>,
    body: Bytes,
) -> Result<Json<DemodulateResponse>, AppError> {
    let audio = if wav::is_wav(&body) {
        wav::decode(&body)?
    } else {
        let sample_rate = query.sample_rate.ok_or_else(|| {
            AppError::InvalidInput("sample_rate is required for raw float32 pcm".into())
        })?;
        Audio {
            sample_rate,
            samples: wav::decode_f32_le(&body)?,
        }
    };
    let config = ModemConfig {
        scheme: query.scheme,
        baud: query.baud,
        sample_rate: audio.sample_rate,
    };
    let reception = modem::receive(&config, &audio.samples)?;
    Ok(Json(DemodulateResponse {
        text: reception.text,
        crc_ok: reception.crc_ok,
        snr: reception.snr,
    }))
}

async fn listen(
    Query(query): Query<ListenQuery>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let config = ModemConfig {
        scheme: query.scheme,
        baud: query.baud,
        sample_rate: query.sample_rate,
    };
    let demod = config.build()?;
    Ok(ws.on_upgrade(move |socket| run_listener(socket, demod)))
}

async fn run_listener(mut socket: WebSocket, demod: Fsk) {
    let mut receiver = Receiver::new(demod);
    while let Some(Ok(message)) = socket.recv().await {
        let samples = match message {
            Message::Binary(bytes) => match wav::decode_f32_le(&bytes) {
                Ok(samples) => samples,
                Err(_) => continue,
            },
            Message::Close(_) => break,
            _ => continue,
        };
        for event in receiver.push(&samples) {
            let Ok(text) = serde_json::to_string(&event) else {
                continue;
            };
            if socket.send(Message::Text(text.into())).await.is_err() {
                return;
            }
        }
    }
}
