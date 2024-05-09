use crate::cli::args::Args;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use serde::{self, Deserialize, Serialize};
use tracing::error;
use tracing_subscriber::filter::EnvFilter;

mod cli;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct CaesarConfig {
    app_environment: String,
    app_host: String,
    app_port: String,
    app_origin: String,
    app_relay: String,
    rust_log: String,
}

lazy_static! {
    static ref GLOBAL_CONFIG: CaesarConfig = {
        let cfg: CaesarConfig = confy::load("caesar", "caesar")
            .expect("Konfigurationsdatei konnte nicht geladen werden");
        cfg
    };
}

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
