use axum::{
    routing::{get, post, put},
    Router,
};
use axum_client_ip::SecureClientIpSource;
use caesar_core::relay::appstate::AppState;
use caesar_core::relay::server::download_info;
use caesar_core::relay::server::download_success;
use caesar_core::relay::server::upload_info;
use caesar_core::relay::server::ws_handler;
use shuttle_axum::ShuttleAxum;


/// The main function that sets up the Axum application.
///
/// This function creates a new server data structure and sets up the application routes.
/// The routes include "/ws" for the websocket handler, "/upload" for the upload info handler,
/// "/download/:name" for the download info handler, and "/download_success/:name" for the download success handler.
/// The routes are associated with the corresponding handlers.
///
/// The application state is wrapped around the routes using the `with_state` method.
/// The client IP source is added as an extension using the `layer` method.
///
/// The function returns a `ShuttleAxum` result.
#[shuttle_runtime::main]
async fn axum() -> ShuttleAxum {
    // Create a new server data structure.
    let appstate = AppState::new();

    // Set up the application routes.
    let app = Router::new()
        .route("/ws", get(ws_handler)) // Route for the websocket handler
        .route("/upload", put(upload_info)) // Route for the upload info handler
        .route("/download/:name", get(download_info)) // Route for the download info handler
        .route("/download_success/:name", post(download_success)) // Route for the download success handler
        .with_state(appstate) // Wrap the routes with the application state
        .layer(SecureClientIpSource::ConnectInfo.into_extension()); // Add the client IP source as an extension

    // Return the application router wrapped in a `ShuttleAxum` result.
    Ok(app.into())
}
