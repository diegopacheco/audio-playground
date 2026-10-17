mod api;
mod compose;
mod error;
mod harmony;
mod melody;
mod render;
mod rng;
mod theory;

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
    println!("generative-music backend on http://localhost:{port}/api");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
