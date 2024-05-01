use tracing::{debug, error};

use local_ip_address::{local_ip, local_ipv6};
use reqwest::blocking::Client;
use tokio::task;

use crate::relay::transfer::{TransferRequest, TransferResponse};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn send_info(
    relay: &str,
    name: &str,
    room_id: &str,
    is_local: bool,
) -> Result<TransferResponse> {
    let url = relay.to_string();
    let sender_ip = match local_ipv6() {
        Ok(ip) => ip,
        Err(_) => match local_ip() {
            Ok(ip) => ip,
            Err(e) => {
                error!("Error getting local ip: {e:?}");
                return Err(Box::new(e));
            }
        },
    };
    let ip_str = sender_ip.to_owned().to_string();

    let transfer_request = TransferRequest {
        name: String::from(name),
        ip: ip_str,
        local_room_id: if is_local {
            String::from(room_id)
        } else {
            String::from("")
        },
        relay_room_id: if !is_local {
            String::from(room_id)
        } else {
            String::from("")
        },
    };

    debug!("Trying to send Request.");
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

    result
}
