use axum::{routing::get, Router};
use axum_client_ip::SecureClientIpSource;
use relay::server;
use shuttle_axum::ShuttleAxum;

pub mod receiver;
pub mod relay;
pub mod sender;
pub mod shared;

#[shuttle_runtime::main]
async fn axum() -> ShuttleAxum {
    // Create a new server data structure.
    let server = server::Server::new();

    // Set up the application routes.
    let app = Router::new()
        .route("/ws", get(relay::ws_handler))
        .with_state(server)
        .layer(SecureClientIpSource::ConnectInfo.into_extension());

    Ok(app.into())
}
