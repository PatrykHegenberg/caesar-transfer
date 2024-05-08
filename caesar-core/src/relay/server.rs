use axum::{
    extract::{ws::WebSocket, Json, Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};

use futures_util::StreamExt;
use serde_json::json;
use std::{env, net::SocketAddr, sync::Arc};
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

pub async fn start_ws(port: Option<&i32>, listen_addr: Option<&String>) {
    let app_host = match listen_addr {
        Some(address) => address.to_string(),
        None => env::var("APP_HOST").unwrap_or("0.0.0.0".to_string()),
    };
    let app_port = match port {
        Some(port) => port.to_string(),
        None => env::var("APP_PORT").unwrap_or("8000".to_string()),
    };

    debug!("Server configured to accept connections on host {app_host}...",);
    debug!("Server configured to listen connections on port {app_port}...",);

    let server = AppState::new();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/upload", put(upload_info))
        .route("/download/:name", get(download_info))
        .route("/download_success/:name", post(download_success))
        .with_state(server)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    if let Ok(listener) = TcpListener::bind(&format!("{}:{}", app_host, app_port)).await {
        info!("Listening on: {}", listener.local_addr().unwrap());

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    } else {
        error!("Failed to listen on: {}:{}", app_host, app_port);
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(shared_state): State<Arc<RwLock<AppState>>>,
) -> impl IntoResponse {
    debug!("Got Request on Websocket route");
    debug!("Upgrading Connection");
    ws.on_upgrade(move |socket| handle_socket(socket, shared_state))
}

async fn handle_socket(socket: WebSocket, rooms: Arc<RwLock<AppState>>) {
    let (sender, mut receiver) = socket.split();

    let sender = Arc::new(Mutex::new(sender));
    let mut client = Client::new(sender.clone());
    while let Some(message) = receiver.next().await {
        match message {
            Ok(message) => {
                client.handle_message(&rooms, message).await;
            }
            Err(error) => {
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

pub async fn upload_info(
    State(shared_state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<TransferRequest>,
) -> impl IntoResponse {
    let mut data = shared_state.write().await;
    match data
        .transfers
        .iter_mut()
        .find(|request| request.name == payload.name)
    {
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
        None => {
            let mut local = String::from("");
            let mut relay = String::from("");
            if payload.relay_room_id.is_empty() {
                local = payload.local_room_id;
            } else {
                relay = payload.relay_room_id;
            }
            let t_request = TransferResponse {
                name: payload.name,
                ip: payload.ip,
                local_room_id: local,
                relay_room_id: relay,
            };
            data.transfers.push(t_request.clone());

            debug!("New TransferRequest created");
            debug!("Actual AppState is {:#?}", *data);

            (StatusCode::CREATED, Json(t_request))
        }
    }
}

pub async fn download_info(
    State(shared_state): State<Arc<RwLock<AppState>>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let data = shared_state.write().await;
    match data.transfers.iter().find(|request| request.name == name) {
        Some(request) => {
            debug!("Found transfer name.");
            (StatusCode::OK, Json(request.clone()))
        }
        None => {
            warn!("couldn't find transfer-name: {}", name);
            (
                StatusCode::NOT_FOUND,
                Json(TransferResponse {
                    name: String::from(""),
                    ip: String::from(""),
                    local_room_id: String::from(""),
                    relay_room_id: String::from(""),
                }),
            )
        }
    }
}

pub async fn download_success(
    State(shared_state): State<Arc<RwLock<AppState>>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let mut data = shared_state.write().await;
    if let Some(index) = data
        .transfers
        .iter()
        .position(|request| request.name == name)
    {
        debug!("Found Transfer by name '{name}'");
        data.transfers.remove(index);
        debug!("Transfer deleted");
        (
            StatusCode::OK,
            Json(json!({
                "message": "transfer deleted"
            })),
        )
    } else {
        warn!("couldn't find transfer-name: {}", name);
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "transfer not found"
            })),
        )
    }
}
