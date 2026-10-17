mod api;
mod error;
mod note;
mod pitch;

use error::TunerError;

const DEFAULT_PORT: &str = "8106";

#[tokio::main]
async fn main() -> Result<(), TunerError> {
    let raw = std::env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
    let port: u16 = raw.parse().map_err(|_| TunerError::InvalidPort(raw))?;
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("pitch tuner listening on http://localhost:{port}");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
