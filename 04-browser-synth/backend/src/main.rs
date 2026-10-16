mod api;
mod dsp;
mod error;
mod preset;
mod render;
mod store;
mod wav;

use std::net::SocketAddr;

use tower_http::cors::CorsLayer;

use crate::store::PresetStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8104".to_string())
        .parse()?;
    let app = api::router(PresetStore::with_built_ins()).layer(CorsLayer::permissive());
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("browser-synth backend listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
