use crate::transfer_info::transfer_info::{TransferInfoBody, TransferInfoRequest};
use axum::{
    extract::{connect_info::ConnectInfo, Json, Path, State},
    http::{self, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::{env, net::SocketAddr};
use tower_http::services::ServeFile;
use tracing::debug;

pub async fn serf_file(path: &String) {
    debug!("Sender starting...");
    let app_environemt = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = "0.0.0.0".to_string();
    let app_port = "1300".to_string();
    debug!("Server configured to accept connections on host {app_host}...");
    debug!("Server configured to listen connections on port {app_port}...");

    match app_environemt.as_str() {
        "development" => {
            debug!("Running in development mode");
        }
        "production" => {
            debug!("Running in production mode");
        }
        _ => {
            debug!("Running in development mode");
        }
    }
    let app = Router::new().route_service("/download_file", ServeFile::new(path));
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", app_host, app_port).to_string())
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
