use crate::error::TransferNotCreatedError;
use crate::transfer_info::transfer_info::TransferInfoRequest;
use local_ip_address;
use reqwest::{Client, StatusCode};
use std::collections::HashMap;
use tracing::{debug, error, info};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn send_info(relay: &str, file: &str) -> Result<String> {
    let sender_ip = local_ip_address::local_ip().unwrap();
    debug!("local ip is: {}", sender_ip);
    let ip_str = sender_ip.to_owned().to_string();
    debug!("Send Request to: {:?}", relay.to_string());
    let mut map = HashMap::new();
    map.insert("keyword", "test");
    map.insert("files", file);
    map.insert("ip", ip_str.as_str());

    let client = Client::new();
    let res = client
        .post(relay.to_string() + "/upload")
        .json(&map)
        .send()
        .await?;
    if res.status() == StatusCode::CREATED {
        let transfer_info: TransferInfoRequest = res.json().await?;
        debug!("Json Response: {:#?}", transfer_info);
        Ok(transfer_info.name)
    } else {
        Err(Box::new(TransferNotCreatedError::new(
            "Transfer could not be created.",
        )))
    }
}
