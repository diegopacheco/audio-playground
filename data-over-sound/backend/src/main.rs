mod api;
mod crc;
mod dsp;
mod error;
mod modem;
mod wav;

use error::AppError;

fn port() -> Result<u16, AppError> {
    let value = std::env::var("PORT").map_err(|_| AppError::MissingPort)?;
    value.parse().map_err(|_| AppError::InvalidPort(value))
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let addr = format!("127.0.0.1:{}", port()?);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("data-over-sound listening on http://{addr}");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
