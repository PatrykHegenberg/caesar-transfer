use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use lazy_static::lazy_static;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower_http::services::ServeFile;
use tracing::debug;

lazy_static! {
    static ref SHUTDOWN_SIGNAL: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}
pub async fn serf_file(path: &String) {
    debug!("Sender starting...");
    let app_host = "0.0.0.0".to_string();
    let app_port = "8100".to_string();
    debug!("Server configured to accept connections on host {app_host}...");
    debug!("Server configured to listen connections on port {app_port}...");

    let app = Router::new()
        .route_service("/download_file", ServeFile::new(path))
        .route("/ping", get(ping))
        .route("/shutdown", post(shutdown));
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", app_host, app_port).to_string())
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        while !*SHUTDOWN_SIGNAL.lock().await {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
}

async fn ping() -> impl IntoResponse {
    let response = json!({
    "message": "pong"
    });
    (StatusCode::OK, Json(response))
}

async fn shutdown() -> impl IntoResponse {
    debug!("Initiating server shutdown...");
    *SHUTDOWN_SIGNAL.lock().await = true;
    debug!("Server is shutting down...");
    (StatusCode::OK, "Server is shutting down...")
}
