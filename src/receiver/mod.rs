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
pub mod http_client;

use crate::{receiver::client as receiver, sender::util::replace_protocol};

use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue},
};
use tracing::{debug, error};

pub async fn start_receiver(relay: &str, name: &str) {
    let http_url = replace_protocol(relay);
    let res = http_client::download_info(http_url.as_str(), name)
        .await
        .unwrap();
    debug!("Got room_id from Server: {:?}", res);
    let res_ip = String::from("ws://") + res.ip.as_str() + ":9000";

    if let Err(local_err) = start_ws_com(res_ip.as_str(), res.local_room_id.as_str()).await {
        debug!("Failed to connect local: {local_err}");
        if let Err(relay_err) = start_ws_com(relay, res.relay_room_id.as_str()).await {
            debug!("Failed to connect remote: {relay_err}");
        }
    }
    let success = http_client::download_success(http_url.as_str(), name).await;
    match success {
        Ok(()) => debug!("Success"),
        Err(e) => error!("Error: {e:?}"),
    };

    // if let Err(e) = start_ws_com(res_ip.as_str(), res.local_room_id.as_str()).await {
    //     debug!("Failed to connect local with first room_id: {e}");
    //     if let Err(e) = start_ws_com(relay, res.relay_room_id.as_str()).await {
    //         debug!("Failed to connect remote with first room_id: {e}");
    //     }
    // }
}

pub async fn start_ws_com(relay: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = String::from(relay) + "/ws";
    let Ok(mut request) = url.into_client_request() else {
        println!("Error: Failed to create request.");
        return Err("Failed to create request".into());
    };

    // Insert the origin into the request headers to prevent
    // cross-origin requests.
    request
        .headers_mut()
        .insert("Origin", HeaderValue::from_str(relay).unwrap());

    println!("Attempting to connect...");

    let _ = match tokio::time::timeout(std::time::Duration::from_secs(5), connect_async(request))
        .await
    {
        Ok(Ok((socket, _))) => {
            receiver::start(socket, name).await;
            Ok(())
        }
        Ok(Err(e)) => {
            error!("Error: Failed to connect: {e:?}");
            Err(Box::new(e))
        }
        Err(e) => {
            error!("Error: Timeout reached for local connection attempt");
            Err(Box::new(e))
        }?,
    };
    // The start function is defined in the
    // receiver::client module and is the function that interacts with
    // the server to receive files.
    // receiver::start(socket, name).await
    Ok(())
}
