/// This module is the entry point for the receiver command.
/// It contains a single function, `start_receiver`, which is the
/// entry point for the receiver program.
///
/// The `start_receiver` function takes a `String` which is the URL or
/// invite code for the room that the receiver should join. If the
/// URL is invalid or does not contain an invite code fragment,
/// the function falls back to using the command line arguments to get
/// the file paths to be sent.
///
/// The `start_receiver` function first creates a request to connect
/// to the WebSocket server with a specific origin. This is done to
/// prevent cross-origin requests, which are not allowed by the
/// WebSocket protocol.
///
/// If creating the request succeeds, the function inserts the origin
/// into the request headers. Then, it attempts to connect to the
/// server using the `connect_async` function from the
/// `tokio_tungstenite` crate.
///
/// If the connection attempt succeeds, the function extracts the
/// invite code fragment from the URL and passes it to the `start`
/// function in the `receiver::client` module. The `start` function is
/// defined in the `receiver::client` module and is the function that
/// interacts with the server to receive files.
///
/// If the connection attempt fails or the URL does not contain an
/// invite code fragment, the function falls back to using the command
/// line arguments to get the file paths to be sent. It then calls the
/// `start` function in the `sender::client` module with the
/// WebSocket stream and the file paths. The `start` function in the
/// `sender::client` module is defined in the `sender::client`
/// module and is the function that sends the files over the
/// WebSocket connection.
///
/// The `start` function takes ownership of the WebSocket stream and
/// the file paths, so we pass them by value.
pub mod client;

use crate::receiver::client as receiver;

use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue},
};
use tracing::error;
use url::Url;

pub async fn start_receiver(relay: &str, name: &str) {
    let argument = name;
    let Ok(mut request) = relay.into_client_request() else {
        println!("Error: Failed to create request.");
        return;
    };

    // Insert the origin into the request headers to prevent
    // cross-origin requests.
    request
        .headers_mut()
        .insert("Origin", HeaderValue::from_str(relay).unwrap());

    println!("Attempting to connect...");

    let Ok((socket, _)) = connect_async(request).await else {
        error!("Error: Failed to connect.");
        return;
    };

    // If the URL is valid and contains an invite code fragment,
    // extract it and pass it to the receiver::client::start
    // function. The start function is defined in the
    // receiver::client module and is the function that interacts with
    // the server to receive files.
    if let Ok(url) = Url::parse(argument) {
        let Some(fragment) = url.fragment() else {
            error!("Error: Missing invite code fragment in url.");
            return;
        };

        receiver::start(socket, fragment).await
    }
}
