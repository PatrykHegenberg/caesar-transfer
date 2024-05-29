use crate::cli::args::Args;
use dotenvy::dotenv;
use tracing::error;
use tracing_subscriber::filter::EnvFilter;

mod cli;
mod config;

/// Entry point of the application.
///
/// This function is called when the application is started. It initializes the environment,
/// parses the command line arguments, and runs the application.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load environment variables from the `.env` file.
    dotenv().ok();

    // Initialize the logging subscriber.
    // It configures the logging level based on the `RUST_LOG` environment variable.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Parse the command line arguments.
    let args = Args::new();

    // Run the application.
    // If an error occurs, log the error message.
    if let Err(e) = args.run().await {
        error!("{e}");
    }

    Ok(())
}
