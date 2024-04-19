mod cli;
mod http_client;
mod http_server;
mod receiver;
mod sender;
mod transfer_info;
use crate::cli::args::Args;
use dotenv::dotenv;
use tracing::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    env_logger::init();
    let args = Args::new();
    if let Err(e) = args.run().await {
        error!("{e}");
    }
    Ok(())
}
