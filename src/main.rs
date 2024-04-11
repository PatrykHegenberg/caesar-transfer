mod args;
pub mod http_client;
pub mod receiver;
pub mod sender;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = args::Args::new();
    if let Err(e) = args.run().await {
        eprintln!("Error {e}");
    }
    Ok(())
}
