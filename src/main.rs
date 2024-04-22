mod cli;
mod error;
mod http_client;
mod receiver;
mod relay;
mod sender;
mod transfer_info;
use crate::cli::args::Args;
use dotenv::dotenv;
use tracing::error;
use tracing_subscriber::filter::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    // env_logger::init();
    let args = Args::new();
    if let Err(e) = args.run().await {
        error!("{e}");
    }
    Ok(())
}
