use tracing::{debug, error};

use local_ip_address::{local_ip, local_ipv6};
use reqwest::blocking::Client;
use tokio::task;

use crate::relay::transfer::{TransferRequest, TransferResponse};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Asynchronously sends information about the sender to the specified relay.
///
/// # Arguments
///
/// * `relay` - The URL of the relay.
/// * `name` - The name of the sender.
/// * `room_id` - The ID of the room.
/// * `is_local` - Indicates whether the sender is local.
///
/// # Returns
///
/// A `Result` containing a `TransferResponse` if the request was successful, or an error if it failed.
pub async fn send_info(
    relay: &str,
    name: &str,
    room_id: &str,
    is_local: bool,
) -> Result<TransferResponse> {
    // Build the URL for the request
    let url = relay.to_string();
    
    // Get the sender's IP address
    let sender_ip = match local_ipv6() {
        Ok(ip) => ip,
        Err(_) => match local_ip() {
            Ok(ip) => ip,
            Err(e) => {
                // Log the error and return the error
                error!("Error getting local ip: {e:?}");
                return Err(Box::new(e));
            }
        },
    };
    let ip_str = sender_ip.to_owned().to_string();

    // Create the transfer request
    let transfer_request = TransferRequest {
        // Set the name of the sender
        name: String::from(name),
        // Set the IP address of the sender
        ip: ip_str,
        // Set the room ID for the local sender
        local_room_id: if is_local {
            String::from(room_id)
        } else {
            String::from("")
        },
        // Set the room ID for the relay sender
        relay_room_id: if !is_local {
            String::from(room_id)
        } else {
            String::from("")
        },
    };

    // Log the start of the request
    debug!("Trying to send Request.");
    
    // Send the request and parse the response
    let result: Result<TransferResponse> = task::spawn_blocking(move || {
        let client = Client::new();
        let response = client
            .put(format!("{}/upload", url))
            .json(&transfer_request)
            .send()?
            .json()?;
        Ok(response)
    })
    .await?;

    // Return the result
    result
}
