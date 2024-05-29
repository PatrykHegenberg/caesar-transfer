pub mod client;
pub mod http_client;

use crate::{receiver::client as receiver, sender::util::replace_protocol};
use anyhow::{anyhow, Result};

use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue},
};
use tracing::{debug, error};

/// Start the receiver process.
///
/// This function initiates the receiver process by performing the following steps:
/// 1. Replaces the protocol of the given `relay` URL.
/// 2. Downloads the room information from the server.
/// 3. Connects to the local or relay server based on the platform.
/// 4. Downloads the file from the server.
///
/// # Arguments
///
/// * `filepath` - The path to the file to be received.
/// * `relay` - The URL of the relay server.
/// * `name` - The name of the receiver.
///
/// # Returns
///
/// Returns a `Result` indicating the success or failure of the receiver process.
pub async fn start_receiver(filepath: String, relay: &str, name: &str) -> Result<()> {
    let http_url = replace_protocol(relay);
    let res = http_client::download_info(http_url.as_str(), name)
        .await
        .unwrap();
    debug!("Got room_id from Server: {:?}", res);
    let res_ip = String::from("ws://") + res.ip.as_str() + ":9000";

    #[cfg(not(target_os = "android"))]
    if let Err(local_err) = start_ws_com(
        filepath.clone(),
        res_ip.as_str(),
        res.local_room_id.as_str(),
    )
    .await
    {
        debug!("Failed to connect local: {local_err}");
        if let Err(relay_err) = start_ws_com(filepath, relay, res.relay_room_id.as_str()).await {
            debug!("Failed to connect remote: {relay_err}");
        }
    }

    #[cfg(target_os = "android")]
    if let Err(relay_err) = start_ws_com(filepath, relay, res.relay_room_id.as_str()).await {
        debug!("Failed to connect remote: {relay_err}");
    }
    http_client::download_success(http_url.as_str(), name)
        .await
        .map_err(|e| anyhow!("Failed to download success: {}", e))?;

    debug!("Success");
    Ok(())
}

/// Asynchronously starts a WebSocket communication with a relay server.
///
/// # Arguments
///
/// * `filepath` - The path of the file to transfer.
/// * `relay` - The URL of the relay server.
/// * `name` - The name of the receiver.
///
/// # Returns
///
/// Returns a `Result` indicating the success or failure of the WebSocket communication.
pub async fn start_ws_com(filepath: String, relay: &str, name: &str) -> Result<()> {
    // Construct the WebSocket URL by appending "/ws" to the relay URL.
    let url = String::from(relay) + "/ws";

    // Create a WebSocket request using the constructed URL.
    let mut request = url
        .into_client_request()
        .map_err(|e| anyhow!("Failed to create request: {}", e))?;

    // Set the "Origin" header of the request to the relay URL.
    request
        .headers_mut()
        .insert("Origin", HeaderValue::from_str(relay).unwrap());

    // Print a message indicating the attempt to connect.
    println!("Attempting to connect...");

    // Attempt to establish a WebSocket connection with the relay server.
    // If the connection fails or times out, return an error.
    let _ = match tokio::time::timeout(std::time::Duration::from_secs(5), connect_async(request))
        .await
    {
        Ok(Ok((socket, _))) => {
            // Start the receiver process with the established WebSocket connection.
            receiver::start(filepath, socket, name).await;
            Ok(())
        }
        Ok(Err(e)) => {
            // Log the failure to connect.
            error!("Error: Failed to connect: {e:?}");
            Err(Box::new(e))
        }
        Err(e) => {
            // Log the timeout.
            error!("Error: Timeout reached for local connection attempt");
            Err(Box::new(e))
        }?,
    };
    Ok(())
}

