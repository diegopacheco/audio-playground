mod analysis;
mod api;
mod dsp;
mod error;
mod synth;
mod wav;

const DEFAULT_PORT: u16 = 8109;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = match std::env::var("PORT") {
        Ok(value) => value.parse::<u16>()?,
        Err(_) => DEFAULT_PORT,
    };
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("bpm-detection backend listening on http://localhost:{port}");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
