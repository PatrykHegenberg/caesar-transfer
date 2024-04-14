pub mod args;
pub mod http_client;
pub mod http_server;
pub mod receiver;
pub mod sender;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let args = args::Args::new();
    if let Err(e) = args.run().await {
        eprintln!("Error {e}");
    }
    Ok(())
}
