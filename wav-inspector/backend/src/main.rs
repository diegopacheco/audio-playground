mod analysis;
mod api;
mod cli;
mod decode;
mod error;
#[cfg(test)]
mod fixture;
mod inspect;
mod render;
mod riff;

use cli::{CliError, Command, parse_args, text_report};
use render::AsciiRenderer;
use std::process::ExitCode;

const DEFAULT_PORT: u16 = 8102;

async fn serve(port: u16) -> Result<(), CliError> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("wav-inspector listening on http://localhost:{port}");
    axum::serve(listener, api::router()).await?;
    Ok(())
}

fn run_inspect(path: &str, width: usize, height: usize) -> Result<(), CliError> {
    let bytes = std::fs::read(path)?;
    let report = inspect::inspect(&bytes, &AsciiRenderer::new(width, height), width)?;
    print!("{}", text_report(&report));
    Ok(())
}

async fn run(args: &[String]) -> Result<(), CliError> {
    let default_port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    match parse_args(args, default_port)? {
        Command::Serve { port } => serve(port).await,
        Command::Inspect {
            path,
            width,
            height,
        } => run_inspect(&path, width, height),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
