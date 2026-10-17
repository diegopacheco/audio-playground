mod api;
mod base64;
mod carrier;
mod crc32;
mod dsp;
mod error;
mod metrics;
mod stego;
mod wav;

use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("PORT")?.parse()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("audio-steganography listening on http://{addr}/api");
    axum::serve(listener, api::router()).await?;
    Ok(())
}
