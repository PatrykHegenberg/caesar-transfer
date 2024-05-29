use axum::{
    extract::{ws::WebSocket, Json, Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};

use futures_util::StreamExt;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::TcpListener,
    signal,
    sync::{Mutex, RwLock},
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, error, info, warn};

use crate::relay::client::Client;
use crate::relay::transfer::TransferResponse;
use crate::relay::{appstate::AppState, transfer::TransferRequest};

/// Start the WebSocket server.
///
/// This function initializes the server and starts listening for incoming connections.
/// It configures the routes for the WebSocket handler and the upload and download routes.
/// Additionally, it sets up the tracing layer to log incoming requests.
///
/// # Arguments
///
/// * `port` - The port number to listen on.
/// * `listen_addr` - The IP address to listen on.
#[allow(clippy::unused_self)]
pub async fn start_ws(port: &i32, listen_addr: &String) {
    // Log the server configuration.
    debug!("Server configured to accept connections on host {listen_addr}...");
    debug!("Server configured to listen connections on port {port}...");

    // Create a new instance of the server state.
    let server = AppState::new();

    // Set up the routes for the server.
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/upload", put(upload_info))
        .route("/download/:name", get(download_info))
        .route("/download_success/:name", post(download_success))
        .with_state(server)
        // Set up the tracing layer to log incoming requests.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // Start listening for incoming connections.
    let addr = format!("{}:{}", listen_addr, port);
    if let Ok(listener) = TcpListener::bind(&addr).await {
        let local_addr = listener.local_addr().unwrap();
        info!("Listening on: {}", local_addr);

        // Serve the app with the listener and handle shutdown gracefully.
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    } else {
        // Log if the server failed to start.
        error!("Failed to listen on: {listen_addr}:{port}");
    }
}

/// Handler for the WebSocket route.
///
/// This function upgrades the connection to a WebSocket and handles the socket.
///
/// # Arguments
///
/// * `ws` - The WebSocketUpgrade struct containing the upgrade request.
/// * `shared_state` - The shared state of the server.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(shared_state): State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    // Log the request on the WebSocket route.
    debug!("Got Request on Websocket route");
    // Log the connection upgrade.
    debug!("Upgrading Connection");
    // Upgrade the connection to a WebSocket and handle the socket.
    // Move the shared state to the handler to avoid holding the lock during the entire connection.
    ws.on_upgrade(move |socket| handle_socket(socket, shared_state))
}


/// Handles the WebSocket connection.
///
/// This function splits the WebSocket into a sender and receiver,
/// creates a client, and handles the messages received from the client.
/// It also handles the close event from the client.
///
/// # Arguments
///
/// * `socket` - The WebSocket connection.
/// * `rooms` - The shared state of the server.
async fn handle_socket(socket: WebSocket, rooms: Arc<RwLock<AppState>>) {
    // Split the WebSocket into a sender and receiver.
    let (sender, mut receiver) = socket.split();

    // Create a new Mutex to prevent concurrent access to the sender.
    let sender = Arc::new(Mutex::new(sender));

    // Create a new client with the sender.
    let mut client = Client::new(sender.clone());

    // Handle the messages received from the client.
    while let Some(message) = receiver.next().await {
        match message {
            Ok(message) => {
                // Handle the message received from the client.
                client.handle_message(&rooms, message).await;
            }
            Err(error) => {
                // Log the error if failed to read message from the client.
                warn!("Failed to read message from client: {}", error);
                break;
            }
        }
    }

    // Handle the close event from the client.
    client.handle_close(&rooms).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Handles the upload_info route.
///
/// It updates or creates a new transfer request in the shared state.
/// If the request is found in the shared state, it updates the relay_room_id or local_room_id
/// based on the payload. If the request is not found, it creates a new transfer request and
/// adds it to the shared state.
///
/// # Arguments
///
/// * `shared_state` - The shared state containing the transfer requests.
/// * `payload` - The JSON payload containing the transfer request information.
///
/// # Returns
///
/// A tuple of the HTTP status code and the JSON response.
pub async fn upload_info(
    State(shared_state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<TransferRequest>,
) -> impl IntoResponse {
    let mut data = shared_state.write().await;

    // Find the transfer request in the shared state
    match data
        .transfers
        .iter_mut()
        .find(|request| request.name == payload.name)
    {
        // Update the relay_room_id or local_room_id if the request is found
        Some(request) => {
            debug!("Found Transfer");
            debug!("Request is: {:?}", request);
            if request.relay_room_id.is_empty() {
                request.relay_room_id = payload.relay_room_id;
                debug!("Found Transfer and updated");
                debug!("request is: {:#?}", request);
                (StatusCode::OK, Json(request.clone()))
            } else {
                request.local_room_id = payload.local_room_id;
                debug!("Found Transfer and updated");
                debug!("request is: {:#?}", request);
                (StatusCode::OK, Json(request.clone()))
            }
        }
        // Create a new transfer request if the request is not found
        None => {
            // Initialize relay and local room IDs based on the payload
            let mut local = String::from("");
            let mut relay = String::from("");
            if payload.relay_room_id.is_empty() {
                local = payload.local_room_id;
            } else {
                relay = payload.relay_room_id;
            }
            // Create a new transfer request
            let t_request = TransferResponse {
                name: payload.name,
                ip: payload.ip,
                local_room_id: local,
                relay_room_id: relay,
            };
            // Add the transfer request to the shared state
            data.transfers.push(t_request.clone());

            debug!("New TransferRequest created");
            debug!("Actual AppState is {:#?}", *data);

            // Return the created transfer request as the response
            (StatusCode::CREATED, Json(t_request))
        }
    }
}


/// Retrieve information about a transfer request based on the transfer name.
///
/// # Arguments
///
/// * `shared_state` - The shared state containing the transfer requests.
/// * `name` - The name of the transfer request.
///
/// # Returns
///
/// Returns a response containing the transfer request if found, or a not found
/// response if the transfer request is not found.
pub async fn download_info(
    State(shared_state): State<Arc<RwLock<AppState>>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    // Acquire write lock on shared state
    let data = shared_state.write().await;

    // Find transfer request by name
    match data.transfers.iter().find(|request| request.name == name) {
        // If transfer request is found, return Ok response with the transfer request
        Some(request) => {
            debug!("Found transfer name.");
            (StatusCode::OK, Json(request.clone()))
        }
        // If transfer request is not found, return not found response
        None => {
            warn!("couldn't find transfer-name: {}", name);
            (
                StatusCode::NOT_FOUND,
                Json(TransferResponse {
                    // Create a new empty transfer response
                    name: String::from(""),
                    ip: String::from(""),
                    local_room_id: String::from(""),
                    relay_room_id: String::from(""),
                }),
            )
        }
    }
}

/// Delete a transfer request by its name.
///
/// # Arguments
///
/// * `shared_state` - The shared state containing the transfer requests.
/// * `name` - The name of the transfer request.
///
/// # Returns
///
/// Returns a response containing a JSON object with a message indicating the
/// success of the deletion. If the transfer request is not found, a not found
/// response is returned.
pub async fn download_success(
    State(shared_state): State<Arc<RwLock<AppState>>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let mut data = shared_state.write().await;
    // Find the index of the transfer request
    if let Some(index) = data
        .transfers
        .iter()
        .position(|request| request.name == name)
    {
        // If the transfer request is found, remove it from the shared state
        debug!("Found Transfer by name '{name}'");
        data.transfers.remove(index);
        debug!("Transfer deleted");
        // Return a success response
        (
            StatusCode::OK,
            Json(json!({
                "message": "transfer deleted"
            })),
        )
    } else {
        // If the transfer request is not found, return a not found response
        warn!("couldn't find transfer-name: {}", name);
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "transfer not found"
            })),
        )
    }
}
