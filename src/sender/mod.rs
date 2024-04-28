/// Connects to the WebSocket server at `ws://0.0.0.0:8000/ws` with an
/// `Origin` header of `ws://0.0.0.0:8000/ws`. This is the URL that the
/// sender and receiver clients will connect to.
///
/// The `start_sender` function takes a reference to a vector of strings,
/// which are the paths to the files that the sender will send over the
/// WebSocket connection.
///
/// The function first creates a WebSocket request using the `IntoClientRequest`
/// trait from `tungstenite`, which is defined on the `IntoClientRequest` struct.
/// This struct is a type that represents a request to a WebSocket server.
///
/// The `into_client_request` function returns a `Result` because it may fail
/// to create the request. In this case, we do not handle the error, so we just
/// return if the result is an error.
///
/// Once we have a request, we insert the `Origin` header into the headers of
/// the request. This is necessary because the WebSocket protocol requires the
/// `Origin` header to be present in the handshake.
///
/// After that, we print out a message to the console indicating that we are
/// attempting to connect to the server.
///
/// Next, we call the `connect_async` function from `tokio_tungstenite` which
/// takes our request and attempts to connect to the server. This function
/// returns a `Future` that resolves to a tuple of a `WebSocketStream` and a
/// `Response` from the server. The `WebSocketStream` is a stream of
/// WebSocket messages from the server, and the `Response` is the response
/// from the server to our handshake request.
///
/// If connecting to the server fails, we print out an error message and
/// return.
///
/// If connecting to the server succeeds, we pass the `WebSocketStream` and
/// the paths to the files to the `start` function from the `sender` module.
/// The `start` function is defined in the `sender` module, and it is the
/// function that sends the files over the WebSocket connection.
///
/// The `start` function takes ownership of the `WebSocketStream` and the file
/// paths, so we pass it the `paths` vector by value.
pub mod client;
pub mod util;

use crate::sender::client as sender;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue},
};
use tracing::{debug, error};

pub async fn start_sender(relay: &str, files: &[String]) {
    match relay.into_client_request() {
        Ok(mut request) => {
            request
                .headers_mut()
                .insert("Origin", HeaderValue::from_str(relay).unwrap());

            debug!("Attempting to connect to {relay}...");

            match connect_async(request).await {
                Ok((socket, _)) => {
                    let paths = files.to_vec();
                    sender::start(socket, paths).await;
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
