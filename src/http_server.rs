use axum::{
    extract::{connect_info::ConnectInfo, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_client_ip::{SecureClientIpSource};
use log::info;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::signal;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TransferRequest {
    ip: String,
    name: String,
    body: TransferBody,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TransferBody {
    keyword: String,
    files: String,
}

#[derive(Debug, Clone)]
struct AppState {
    data: Arc<Mutex<Vec<TransferRequest>>>,
}

pub async fn start_server() {
    env_logger::init();
    info!("Server starting...");
    let shared_state = AppState {
        data: Arc::new(Mutex::new(Vec::new())),
    };
    let app_environemtn = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let app_port = env::var("APP_PORT").unwrap_or("1323".to_string());

    info!("Server configured to accept connections on host {app_host}...");
    info!("Server configured to listen connections on port {app_port}...");

    match app_environemtn.as_str() {
        "development" => {
            info!("Running in development mode");
        }
        "production" => {
            info!("Running in production mode");
        }
        _ => {
            info!("Running in development mode");
        }
    }
    let app = Router::new()
        .route("/status", get(status))
        .route("/upload", post(upload_info))
        .route("/download/:name", get(download_info))
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
    info!("Get new download request from: {}", addr.ip().to_string());
    let data = shared_state.data.lock().unwrap();
    match data.iter().find(|request| request.name == name) {
        Some(request) => {
            info!("Found transfer name.");
            (StatusCode::OK, Json(request.clone()))
        }
        None => {
            info!("couldn't find transfer-name: {}", name);
            (
                StatusCode::NOT_FOUND,
                Json(TransferRequest {
                    name: "".to_string(),
                    ip: "".to_string(),
                    body: TransferBody {
                        keyword: "".to_string(),
                        files: "".to_string(),
                    },
                }),
            )
        }
    }
}

async fn upload_info(
    State(shared_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<TransferBody>,
) -> impl IntoResponse {
    info!("Got upload request from {}", addr.ip().to_string());
    let mut data = shared_state.data.lock().unwrap();
    let t_request = TransferRequest {
        ip: addr.ip().to_string(),
        name: generate_random_name(),
        body: TransferBody {
            keyword: payload.keyword,
            files: payload.files,
        },
    };
    data.push(t_request.clone());

    info!("New TransferRequest created");
    info!("Actual AppState is {:#?}", *data);

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

fn generate_random_name() -> String {
    let mut rng = thread_rng();
    let adjective = adjectives().choose(&mut rng).unwrap();
    // let adjective = adjectives().sample(&mut rng).unwrap();
    let noun1 = nouns1().choose(&mut rng).unwrap();
    let noun2 = nouns2().choose(&mut rng).unwrap();

    format!("{adjective}-{noun1}-{noun2}")
}

fn adjectives() -> &'static [&'static str] {
    static ADJECTIVES: &[&str] = &["funny", "smart", "creative", "friendly", "great"];
    ADJECTIVES
}

fn nouns1() -> &'static [&'static str] {
    static NOUNS1: &[&str] = &["dog", "cat", "flower", "tree", "house"];
    NOUNS1
}

fn nouns2() -> &'static [&'static str] {
    static NOUNS2: &[&str] = &["cookie", "cake", "frosting"];
    NOUNS2
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
