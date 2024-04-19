use crate::transfer_info::transfer_info::TransferInfoRequest;
use log::{debug, error};
use reqwest::{Client, StatusCode};
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn send_info(relay: &str, file: &str) -> Result<()> {
    debug!("Send Request to: {:?}", relay.to_string());
    let mut map = HashMap::new();
    map.insert("keyword", "test");
    map.insert("files", file);

    let client = Client::new();
    let res = client
        .post(relay.to_string() + "/upload")
        .json(&map)
        .send()
        .await?;
    if res.status() == StatusCode::CREATED {
        let transfer_info: TransferInfoRequest = res.json().await?;
        debug!("Json Response: {:#?}", transfer_info);
        println!("Transfer name: {}", transfer_info.name);
    } else {
        error!("Error reading response");
    }

    Ok(())
}
