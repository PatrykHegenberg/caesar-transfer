use crate::cli::args::Args;
use dotenvy::dotenv;
use tracing::error;
use tracing_subscriber::filter::EnvFilter;

mod cli;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let args = Args::new();
    if let Err(e) = args.run().await {
        error!("{e}");
    }
    Ok(())
}
