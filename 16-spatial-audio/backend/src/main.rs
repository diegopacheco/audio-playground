mod api;
mod audio;
mod dsp;
mod error;
mod source;
mod spatial;
mod trajectory;
mod wav;

use error::AppError;

fn port() -> Result<u16, AppError> {
    let raw = std::env::var("PORT").map_err(|_| AppError::Config("PORT is not set".into()))?;
    raw.parse()
        .map_err(|_| AppError::Config(format!("PORT is not a valid port: {raw}")))
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let port = port()?;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("spatial-audio backend on http://localhost:{port}/api");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
