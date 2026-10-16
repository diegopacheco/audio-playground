mod api;
mod backend;
mod engine;
mod error;
mod probe;
mod process;
mod ring;
mod stats;

use backend::AudioBackend;
use backend::cpal_backend::CpalBackend;
use backend::fake::{FakeBackend, FakeConfig};
use engine::Engine;
use std::sync::{Arc, Mutex};

fn real_backend() -> Option<Box<dyn AudioBackend>> {
    if std::env::var("AUDIO_BACKEND").is_ok_and(|v| v == "fake") {
        println!("AUDIO_BACKEND=fake, real devices disabled");
        return None;
    }
    let cpal = CpalBackend::new();
    if cpal.has_devices() {
        Some(Box::new(cpal))
    } else {
        println!("no audio devices found, running in fake mode");
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8115".into());
    let fake = Box::new(FakeBackend::new(FakeConfig::default()));
    let engine = Arc::new(Mutex::new(Engine::new(fake, real_backend())));
    let app = api::router(engine);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("low-latency backend on http://localhost:{port}/api");
    axum::serve(listener, app).await?;
    Ok(())
}
