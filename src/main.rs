use crate::cli::args::Args;
use dotenv::dotenv;
#[cfg(feature = "shuttle")]
use shuttle_axum::ShuttleAxum;
use tracing::error;
#[cfg(not(feature = "shuttle"))]
use tracing_subscriber::filter::EnvFilter;

pub mod cli;
pub mod receiver;
pub mod relay;
pub mod sender;
pub mod shared;

#[cfg(not(feature = "shuttle"))]
#[tokio::main]
// This is the entrypoint of caesar.
// The #[tokio::main] attribute is required for any async code, and it
// sets up the tokio runtime.
// The async fn main() is the entrypoint of the application, and it's where
// we kick off our program.
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load environment variables from a .env file if one is present.
    dotenv().ok();
    // Set up our logging subscriber.
    // TheEnvFilter::from_default_env reads the env variable RUST_LOG
    // and sets up the logging accordingly.
    // The default is INFO level logging.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    // Parse the command line arguments.
    let args = Args::new();
    // Run the commands based on the parsed arguments.
    // If there is an error, print it to the console with the error! macro.
    if let Err(e) = args.run().await {
        error!("{e}");
    }
    // Return an Ok result, which just means that our program exited successfully.
    Ok(())
}

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("8000".to_string());

    // Log information about the server's configuration.
    debug!("Server configured to accept connections on host {app_host}...",);
    debug!("Server configured to listen connections on port {app_port}...",);

    // Create a new server data structure.
    let server = server::Server::new();

    // Set up the application routes.
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(server)
        .layer(SecureClientIpSource::ConnectInfo.into_extension())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    Ok(app.into())
}
