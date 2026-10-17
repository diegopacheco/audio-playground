mod api;
mod dsp;
mod error;
mod instruments;
mod pattern;
mod sequencer;
mod store;
mod wav;

use error::AppError;
use std::sync::Arc;
use store::PatternStore;

fn port() -> Result<u16, AppError> {
    let raw = std::env::var("PORT").map_err(|_| AppError::Io("PORT env var is required".into()))?;
    raw.parse()
        .map_err(|_| AppError::Io(format!("PORT is not a number: {raw}")))
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let addr = format!("127.0.0.1:{}", port()?);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("drum machine backend on http://{addr}");
    axum::serve(
        listener,
        api::router(Arc::new(PatternStore::with_builtins())),
    )
    .await?;
    Ok(())
}
