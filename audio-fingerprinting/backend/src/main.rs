mod api;
mod dsp;
mod error;
mod fingerprint;
mod library;
mod wav;

use error::FingerprintError;
use library::Library;
use std::sync::{Arc, RwLock};

const DEFAULT_PORT: &str = "8112";

#[tokio::main]
async fn main() -> Result<(), FingerprintError> {
    let raw = std::env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
    let port: u16 = raw
        .parse()
        .map_err(|_| FingerprintError::InvalidPort(raw))?;
    let library = Library::builtin()?;
    println!(
        "library ready: {} tracks, {} hashes",
        library.list()?.len(),
        library.index_size()
    );
    let state = Arc::new(RwLock::new(library));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("audio fingerprinting listening on http://localhost:{port}");
    axum::serve(listener, api::router(state)).await?;
    Ok(())
}
