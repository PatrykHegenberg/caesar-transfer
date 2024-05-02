use crate::relay::appstate::AppState;
use crate::relay::server::download_info;
use crate::relay::server::download_success;
use crate::relay::server::upload_info;
use crate::relay::server::ws_handler;
use axum::{
    routing::{get, post, put},
    Router,
};
use axum_client_ip::SecureClientIpSource;
use shuttle_axum::ShuttleAxum;

pub mod receiver;
pub mod relay;
pub mod sender;
pub mod shared;

#[shuttle_runtime::main]
async fn axum() -> ShuttleAxum {
    // Create a new server data structure.
    let appstate = AppState::new();

    // Set up the application routes.
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/upload", put(upload_info))
        .route("/download/:name", get(download_info))
        .route("/download_success/:name", post(download_success))
        .with_state(appstate)
        .layer(SecureClientIpSource::ConnectInfo.into_extension());

    Ok(app.into())
}
