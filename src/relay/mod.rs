/// This function starts the WebSocket server.
///
/// It configures the server to listen on the specified host and port. If
/// these values are not specified in the environment, it falls back to using
/// the defaults of "0.0.0.0" for the host and "8000" for the port.
///
/// It then sets up the application routes for the server. In this case, the
/// only route is for the WebSocket connection.
///
/// The WebSocket route requires a `ConnectInfo` extractor to get the client's
/// IP address, which is then used to store the client in a data structure
/// keyed by their IP address. This allows for efficient lookup of clients by
/// their IP address.
///
/// Finally, it starts the server by binding to the specified host and port,
/// and running the application. If the server fails to bind to the specified
/// host and port, it logs an error and exits.
pub mod server;
use axum::{
    extract::{ws::WebSocket, ConnectInfo, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_client_ip::SecureClientIpSource;

use futures_util::StreamExt;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::{
    net::TcpListener,
    signal,
    sync::{Mutex, RwLock},
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, error, info};

use self::server::Client;

/// This function starts the WebSocket server.
///
/// It retrieves the environment variables that define how the server should
/// be configured. If any of these variables are not defined, it sets a
/// reasonable default value.
///
/// The environment variables are:
///
/// * `APP_ENVIRONMENT`: the environment the server is running in (defaults
///   to "development").
/// * `APP_HOST`: the host the server should listen on (defaults to "0.0.0.0").
/// * `APP_PORT`: the port the server should listen on (defaults to "8000").
/// * `APP_DOMAIN`: the domain the server is accessible at (defaults to "").
///
/// It then sets up the application routes for the server. In this case, the
/// only route is for the WebSocket connection.
///
/// The WebSocket route requires a `ConnectInfo` extractor to get the client's
/// IP address, which is then used to store the client in a data structure
/// keyed by their IP address. This allows for efficient lookup of clients by
/// their IP address.
///
/// Finally, it starts the server by binding to the specified host and port,
/// and running the application. If the server fails to bind to the specified
/// host and port, it logs an error and exits.
pub async fn start_ws(port: Option<&i32>, listen_addr: Option<&String>) {
    // Retrieve environment variables and set defaults if necessary.
    let app_environemt = env::var("APP_ENVIRONMENT").unwrap_or("development".to_string());
    let app_host = match listen_addr {
        Some(address) => address.to_string(),
        None => env::var("APP_HOST").unwrap_or("0.0.0.0".to_string()),
    };
    let app_port = match port {
        Some(port) => port.to_string(),
        None => env::var("APP_PORT").unwrap_or("8000".to_string()),
    };
    let app_domain = env::var("APP_DOMAIN").unwrap_or("".to_string());

    // Log information about the server's configuration.
    debug!(
        "Server configured to accept connections on host {app_host}...",
    );
    debug!(
        "Server configured to listen connections on port {app_port}...",
    );
    debug!(
        "Server configured to listen connections on port {app_domain}...",
    );

    // Based on the environment variable, set the logging level.
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

    // Create a new server data structure.
    let server = server::Server::new();

    // Set up the application routes.
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(server)
        .layer(SecureClientIpSource::ConnectInfo.into_extension())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // Attempt to bind to the specified host and port.
    if let Ok(listener) = TcpListener::bind(&format!("{}:{}", app_host, app_port)).await {
        // Log successful binding.
        info!("Listening on: {}", listener.local_addr().unwrap());

        // Run the server.
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    } else {
        // Log binding failure and exit.
        error!("Failed to listen on: {}:{}", app_host, app_port);
    }
}


/// This function is an endpoint for the WebSocket route.
///
/// This function is called whenever a client makes a WebSocket request to
/// the `/ws` endpoint.
///
/// The function takes two arguments:
///
/// - `ws`: This is the WebSocketUpgrade object, which is used to upgrade the
///   HTTP connection to a WebSocket connection.
/// - `State(shared_state)`: This is the state of the server, which is stored
///   in a read-write lock. The state is shared between all WebSocket
///   connections.
/// - `ConnectInfo(addr)`: This is the information about the client that
///   connected to the server. The function uses this information to log the
///   address of the client that connected to the server.
///
/// The function upgrades the HTTP connection to a WebSocket connection using
/// the `ws` argument. It then passes the upgraded WebSocket connection, along
/// with the state of the server, to the `handle_socket` function.
///
/// The `handle_socket` function is defined in the `src/relay/mod.rs` file. It
/// is the function that handles the WebSocket connection.
///
/// The `handle_socket` function takes three arguments:
///
/// - `socket`: This is the WebSocket connection that it should handle.
/// - `who`: This is the address of the client that connected to the server.
/// - `rooms`: This is the state of the server, which is stored in a read-write
///   lock. The state is shared between all WebSocket connections.
///
/// The `handle_socket` function handles the WebSocket connection by calling
/// the `handle_message` function on a `Client` object that it creates. The
/// `handle_message` function is defined in the `src/relay/client.rs` file. The
/// `handle_message` function handles incoming messages from the client and
/// takes care of sending the appropriate response back to the client.
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(shared_state): State<Arc<RwLock<server::Server>>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    debug!("Got Request on Websocket route");
    debug!("WebSocket connection established from:{}", addr.to_string());
    debug!("Upgrading Connection");
    ws.on_upgrade(move |socket| handle_socket(socket, addr, shared_state))
}


/// This function is called when a new WebSocket connection is established.
/// The function takes three arguments:
///
/// - `socket`: This is the WebSocket connection that it should handle.
/// - `who`: This is the address of the client that connected to the server.
/// - `rooms`: This is the state of the server, which is stored in a read-write
///   lock. The state is shared between all WebSocket connections.
///
/// The function creates a `Client` object, which will handle the WebSocket
/// connection. The `Client` object is created with an Arc-wrapped Mutex
/// containing the `sender` of the WebSocket connection. The `sender` is used to
/// send messages to the client.
///
/// The function then creates a new `split` of the WebSocket connection, which
/// is a pair of a `sender` and a `receiver`. The `sender` is used to send
/// messages to the client, and the `receiver` is used to receive messages from
/// the client. The `receiver` is wrapped in a `Stream` (which is an async
/// iterator) so that the function can use the `next` method to receive messages
/// from the client.
///
/// The function then enters a loop that receives incoming messages from the
/// client and handles them. For each received message, the function calls the
/// `handle_message` method on the `Client` object that it created. The
/// `handle_message` method is defined in the `src/relay/client.rs` file. The
/// `handle_message` method handles incoming messages from the client and
/// takes care of sending the appropriate response back to the client.
///
/// If the function encounters an error while reading a message from the
/// client, it logs the error and breaks out of the loop.
///
/// After the loop finishes (either because an error occurred or because the
/// client disconnected), the function calls the `handle_close` method on the
/// `Client` object that it created. The `handle_close` method is defined in the
/// `src/relay/client.rs` file. The `handle_close` method handles the close event
/// from the client.
async fn handle_socket(socket: WebSocket, who: SocketAddr, rooms: Arc<RwLock<server::Server>>) {
    let (sender, mut receiver) = socket.split();

    let sender = Arc::new(Mutex::new(sender));
    let mut client = Client::new(sender.clone());
    while let Some(message) = receiver.next().await {
        match message {
            Ok(message) => {
                client.handle_message(&rooms, message).await;
            }
            Err(error) => {
                error!("Failed to read message from client {}: {}", who, error);
                break;
            }
        }
    }
    // Handle the close event from the client.
    client.handle_close(&rooms).await
}


/// This function sets up a signal handler for SIGINT (Ctrl+C) and SIGTERM
/// (terminate) on Unix platforms. It does nothing on non-Unix platforms.
///
/// The function installs two signal handlers: one for SIGINT and one for
/// SIGTERM. When either of these signals is received, the signal handler
/// simply resolves the future with `()`. This allows the main function to
/// wait for the signal handler to trigger a shutdown.
///
/// The function uses the `tokio::select!` macro to wait for either of the
/// signal handlers to resolve. When the future returned by `tokio::select!`
/// resolves, the function simply drops the value and does nothing else.
///
/// The function does not actually do anything itself. It simply waits for
/// one of the signal handlers to trigger a shutdown.
async fn shutdown_signal() {
    // Install a signal handler for SIGINT (Ctrl+C). This future resolves
    // when the user presses Ctrl+C.
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // Install a signal handler for SIGTERM (terminate). This future
    // resolves when the operating system sends a SIGTERM signal to the
    // program.
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // If we are not on a Unix platform, we don't need to install a signal
    // handler for SIGTERM. Instead, we create a future that never resolves.
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either of the two signal handlers to resolve. When one of them
    // resolves, the other one may still be waiting, but it doesn't matter
    // because we don't need to do anything else.
    tokio::select! {
        // If the Ctrl+C signal handler resolves, drop the value and do
        // nothing else.
        _ = ctrl_c => {},
        // If the terminate signal handler resolves, drop the value and do
        // nothing else.
        _ = terminate => {},
    }
}

