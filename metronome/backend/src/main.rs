mod api;
mod click;
mod drift;
mod error;
mod scheduler;
mod wav;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT").unwrap_or_else(|_| "8103".to_string());
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    println!("metronome backend on http://localhost:{port}");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
