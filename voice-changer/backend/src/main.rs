mod api;
mod dsp;
mod effects;
mod error;
mod signal;
mod wav;

use error::VoiceError;

const DEFAULT_PORT: &str = "8113";

#[tokio::main]
async fn main() -> Result<(), VoiceError> {
    let raw = std::env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
    let port: u16 = raw.parse().map_err(|_| VoiceError::InvalidPort(raw))?;
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("voice changer listening on http://localhost:{port}");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
