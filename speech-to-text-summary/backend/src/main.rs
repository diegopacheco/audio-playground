mod api;
mod convert;
mod error;
mod resample;
mod service;
mod summarizer;
mod transcriber;
mod wav;

use api::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use summarizer::ClaudeSummarizer;
use transcriber::WhisperCppTranscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8114".to_string());
    let binary = std::env::var("WHISPER_BIN").unwrap_or_else(|_| "whisper-cli".to_string());
    let model = std::env::var("WHISPER_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../models/ggml-base.en.bin"));
    let state = AppState {
        transcriber: Arc::new(WhisperCppTranscriber::new(binary, model)),
        summarizer: Arc::new(ClaudeSummarizer::from_env()),
    };
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("listening on http://localhost:{port}/api");
    axum::serve(listener, api::router(state)).await?;
    Ok(())
}
