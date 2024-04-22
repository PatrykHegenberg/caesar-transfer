use crate::transfer_info::transfer_info::{TransferInfoBody, TransferInfoRequest};
use axum::{
    extract::{connect_info::ConnectInfo, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_client_ip::SecureClientIpSource;
use serde_json::json;
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::signal;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
struct AppState {
    data: Arc<Mutex<Vec<TransferInfoRequest>>>,
}

pub async fn start_server(port: Option<&i32>, listen_addr: Option<&String>) {
    info!("Server starting...");
    let shared_state = AppState {
        data: Arc::new(Mutex::new(Vec::new())),
    };
    let app_environemt = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = match listen_addr {
        Some(address) => address.to_string(),
        None => env::var("APP_HOST").unwrap_or("0.0.0.0".to_string()),
    };
    let app_port = match port {
        Some(port) => port.to_string(),
        None => env::var("APP_PORT").unwrap_or("8000".to_string()),
    };

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
    let app = Router::new()
        .route("/status", get(status))
        .route("/upload", post(upload_info))
        .route("/download/:name", get(download_info))
        .route("/download_success/:name", post(download_success))
        .with_state(shared_state)
        .layer(SecureClientIpSource::ConnectInfo.into_extension());
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", app_host, app_port).to_string())
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn download_info(
    State(shared_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    debug!("Get new download request from: {}", addr.ip().to_string());
    let data = shared_state.data.lock().unwrap();
    match data.iter().find(|request| request.body.name == name) {
        Some(request) => {
            debug!("Found transfer name.");
            (StatusCode::OK, Json(request.clone()))
        }
        None => {
            warn!("couldn't find transfer-name: {}", name);
            (
                StatusCode::NOT_FOUND,
                Json(TransferInfoRequest {
                    message: "error".to_string(),
                    body: TransferInfoBody {
                        keyword: "".to_string(),
                        files: "".to_string(),
                        ip: "".to_string(),
                        name: "".to_string(),
                    },
                }),
            )
        }
    }
}

async fn upload_info(
    State(shared_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<TransferInfoBody>,
) -> impl IntoResponse {
    debug!("Got upload request from {}", addr.ip().to_string());
    let mut data = shared_state.data.lock().unwrap();
    let t_request = TransferInfoRequest {
        message: "created".to_string(),
        body: TransferInfoBody {
            keyword: payload.keyword,
            files: payload.files,
            ip: payload.ip,
            name: payload.name,
        },
    };
    data.push(t_request.clone());

    debug!("New TransferRequest created");
    debug!("Actual AppState is {:#?}", *data);

    (StatusCode::CREATED, Json(t_request))
}

async fn status() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");

    let response = json!({
    "data": {
    "version": version,
    },
    "message": "Service is running..."
    });
    (StatusCode::OK, Json(response))
}

async fn download_success(
    State(shared_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let mut data = shared_state.data.lock().unwrap();
    if let Some(index) = data.iter().position(|request| request.body.name == name) {
        debug!("Found Transfer by name '{name}'");
        data.remove(index);
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
    // match data.iter().find(|request| request.body.name == name) {
    //     Some(request) => {
    //         debug!("Found transfer name.");
    //         return (
    //             StatusCode::OK,
    //             Json(json!({
    //                 "message" : "transfer deleted"
    //             })),
    //         );
    //     }
    //     None => {
    //         warn!("couldn't find transfer-name: {}", name);
    //         return (
    //             StatusCode::NOT_FOUND,
    //             Json(json!({
    //                 "message" : "transfer not found"
    //             })),
    //         );
    //     }
    // }
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
