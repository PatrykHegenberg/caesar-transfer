pub mod client;
pub mod http_client;
pub mod util;

use std::{net::SocketAddr, sync::Arc};

use crate::{
    relay::{appstate::AppState, server::ws_handler},
    sender::client as sender,
};
use axum::{routing::get, Router};
use tokio::{net::TcpListener, sync::mpsc, task};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue},
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, error, info};
use uuid::Uuid;

/// Start the sender process.
///
/// This function starts the sender process which connects to a relay and
/// initiates file transfers. The sender also starts a local WebSocket server.
///
/// # Arguments
///
/// * `name` - The name of the sender.
/// * `relay` - The relay to connect to.
/// * `files` - The files to transfer.
pub async fn start_sender(name: String, relay: Arc<String>, files: Arc<Vec<String>>) {
    // Log the name of the sender
    debug!("Got name: {:?}", name);
    // Create a channel for communication between threads
    let (tx, mut rx) = mpsc::channel(1);
    // Generate a unique room ID
    let room_id = Uuid::new_v4().to_string();
    let local_room_id = room_id.clone();
    let local_files = files.clone();
    let local_relay = relay.clone();
    let local_rand_name = name.clone();
    let local_tx = tx.clone();
    // Start a local WebSocket server
    let local_ws_thread = task::spawn(async move {
        start_local_ws().await;
    });
    // Connect to the relay
    let relay_thread = task::spawn(async move {
        connect_to_server(
            relay.clone(),
            files.clone(),
            Some(room_id),
            relay.clone(),
            Arc::new(name.clone()),
            tx.clone(),
            false,
        )
        .await
    });
    // Connect to the local WebSocket server
    let local_thread = task::spawn(async move {
        connect_to_server(
            Arc::new(String::from("ws://0.0.0.0:9000")),
            local_files.clone(),
            Some(local_room_id),
            local_relay.clone(),
            Arc::new(local_rand_name.clone()),
            local_tx.clone(),
            true,
        )
        .await
    });

    // Wait for the sender threads to finish
    rx.recv().await.unwrap();
    // Abort the local WebSocket server thread
    local_ws_thread.abort();
    // Abort the relay thread
    relay_thread.abort();
    // Abort the local thread
    local_thread.abort();
}

/// Start a local WebSocket server.
///
/// This function initializes and runs a WebSocket server on the specified host and port.
/// It creates an instance of the `AppState` struct and uses it as the state for the router.
/// The `ws_handler` function is registered as the handler for the "/ws" route.
///
/// # Arguments
///
/// None
///
/// # Returns
///
/// This function does not return anything.
pub async fn start_local_ws() {
    // The host and port the server will listen on.
    let app_host = "0.0.0.0";
    let app_port = "9000";

    // Create an instance of the application state.
    let server = AppState::new();

    // Create the axum application.
    // The `ws_handler` function is registered as the handler for the "/ws" route.
    // The `AppState` instance is used as the state for the router.
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(server)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // Try to bind the server to the specified host and port.
    if let Ok(listener) = TcpListener::bind(&format!("{}:{}", app_host, app_port)).await {
        // Log the address the server is listening on.
        info!(
            "Local WebSocket listening on: {}",
            listener.local_addr().unwrap()
        );

        // Serve the application using the listener.
        // The `connect_info` parameter is used to include the client's socket address in the tracing spans.
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    } else {
        // Log an error if the server fails to bind.
        error!("Failed to listen on: {}:{}", app_host, app_port);
    }
}

/// Connects to the specified server and starts the file transfer.
///
/// # Arguments
///
/// * `relay` - The relay server URL.
/// * `files` - The files to be transferred.
/// * `room_id` - The room ID for the transfer. If `None`, a random UUID is generated.
/// * `message_server` - The message server URL.
/// * `transfer_name` - The name of the transfer.
/// * `tx` - The sender end of a channel to signal the completion of the transfer.
/// * `is_local` - Whether the transfer is local or not.
async fn connect_to_server(
    relay: Arc<String>,
    files: Arc<Vec<String>>,
    room_id: Option<String>,
    message_server: Arc<String>,
    transfer_name: Arc<String>,
    tx: mpsc::Sender<()>,
    is_local: bool,
) {
    // Construct the server URL.
    let url = format!("{}/ws", relay);

    // Construct the message server URL.
    let message_relay = format!("{}", message_server);

    // Construct the transfer name.
    let transfer_name = format!("{}", transfer_name);

    // Create a request to the server.
    match url.clone().into_client_request() {
        Ok(mut request) => {
            // Set the "Origin" header.
            request
                .headers_mut()
                .insert("Origin", HeaderValue::from_str(relay.as_ref()).unwrap());

            // Log the connection attempt.
            debug!("Attempting to connect to {url}...");

            // Generate a room ID if not provided.
            let room_id = match room_id {
                Some(id) => id,
                None => Uuid::new_v4().to_string(),
            };

            // Connect to the server and start the file transfer.
            match connect_async(request).await {
                Ok((socket, _)) => {
                    let paths = files.to_vec();
                    sender::start(
                        socket,
                        paths,
                        Some(room_id),
                        message_relay.to_string(),
                        transfer_name.clone(),
                        is_local,
                    )
                    .await;

                    // Signal the completion of the transfer.
                    tx.send(()).await.unwrap();
                }
                Err(e) => {
                    // Log the connection error.
                    error!("Error: Failed to connect with error: {e}");
                }
            }
        }
        Err(e) => {
            // Log the request creation error.
            error!("Error: failed to create request with reason: {e:?}");
        }
    }
}
