pub mod client;
pub mod http_client;
pub mod util;

use std::{net::SocketAddr, sync::Arc};

use crate::{
    relay::{appstate::AppState, server::ws_handler},
    sender::{client as sender, util::generate_random_name},
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

pub async fn start_sender(relay: Arc<String>, files: Arc<Vec<String>>) {
    let (tx, mut rx) = mpsc::channel(1);
    debug!("Got relay: {relay}");
    let room_id = Uuid::new_v4().to_string();
    let rand_name = generate_random_name();
    let local_room_id = room_id.clone();
    let local_files = files.clone();
    let local_relay = relay.clone();
    let local_rand_name = rand_name.clone();
    let local_tx = tx.clone();
    let local_ws_thread = task::spawn(async move {
        start_local_ws().await;
    });
    let relay_thread = task::spawn(async move {
        connect_to_server(
            relay.clone(),
            files.clone(),
            Some(room_id),
            relay.clone(),
            Arc::new(rand_name.clone()),
            tx.clone(),
            false,
        )
        .await
    });
    let local_thread = task::spawn(async move {
        connect_to_server(
            Arc::new(String::from("ws://localhost:9000")),
            local_files.clone(),
            Some(local_room_id),
            local_relay.clone(),
            Arc::new(local_rand_name.clone()),
            local_tx.clone(),
            true,
        )
        .await
    });

    rx.recv().await.unwrap();
    local_ws_thread.abort();
    relay_thread.abort();
    local_thread.abort();
}

pub async fn start_local_ws() {
    let app_host = "localhost";
    let app_port = "9000";

    let server = AppState::new();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(server)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    if let Ok(listener) = TcpListener::bind(&format!("{}:{}", app_host, app_port)).await {
        info!(
            "Local Websocket listening on: {}",
            listener.local_addr().unwrap()
        );

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    } else {
        error!("Failed to listen on: {}:{}", app_host, app_port);
    }
}

async fn connect_to_server(
    relay: Arc<String>,
    files: Arc<Vec<String>>,
    room_id: Option<String>,
    message_server: Arc<String>,
    transfer_name: Arc<String>,
    tx: mpsc::Sender<()>,
    is_local: bool,
) {
    let url = format!("{}/ws", relay);
    let message_relay = format!("{}", message_server);
    let transfer_name = format!("{}", transfer_name);
    match url.clone().into_client_request() {
        Ok(mut request) => {
            request
                .headers_mut()
                .insert("Origin", HeaderValue::from_str(relay.as_ref()).unwrap());

            debug!("Attempting to connect to {url}...");
            let room_id = match room_id {
                Some(id) => id,
                None => Uuid::new_v4().to_string(),
            };

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
                    tx.send(()).await.unwrap();
                }
                Err(e) => {
                    error!("Error: Failed to connect with error: {e}");
                }
            }
        }
        Err(e) => {
            error!("Error: failed to create request with reason: {e:?}");
        }
    }
}
