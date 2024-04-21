use crate::transfer_info::transfer_info::TransferInfoRequest;
use log::{debug, error};
use reqwest::{Client, StatusCode};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct TransferNotCreatedError {
    message: String,
}

impl TransferNotCreatedError {
    fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for TransferNotCreatedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TransferNotCreatedError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub async fn send_info(relay: &str, file: &str) -> Result<String> {
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
        Ok(transfer_info.name)
    } else {
        Err(Box::new(TransferNotCreatedError::new(
            "Transfer could not be created.",
        )))
    }
}
