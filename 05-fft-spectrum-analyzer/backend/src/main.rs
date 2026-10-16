use fft_spectrum_analyzer::api;
use fft_spectrum_analyzer::error::AppError;

fn port() -> Result<u16, AppError> {
    let raw = std::env::var("PORT").map_err(|_| AppError::MissingPort)?;
    raw.parse().map_err(|_| AppError::InvalidPort(raw))
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let port = port()?;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("fft-spectrum-analyzer listening on http://localhost:{port}");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
