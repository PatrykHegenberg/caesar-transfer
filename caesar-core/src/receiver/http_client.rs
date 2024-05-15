use anyhow::{anyhow, Result};
use hex;
use reqwest::{self, Client};
use sha2::{Digest, Sha256};

use crate::relay::transfer::TransferResponse;

pub async fn download_info(relay: &str, name: &str) -> Result<TransferResponse> {
    let url = String::from(relay);
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);

    let resp = reqwest::get(format!("{}/download/{}", url, hashed_string))
        .await
        .map_err(|e| anyhow!("Failed to send GET request: {}", e))?;

    resp.json::<TransferResponse>()
        .await
        .map_err(|e| anyhow!("Failed to parse JSON response: {}", e))
}

pub async fn download_success(relay: &str, name: &str) -> Result<()> {
    let url = String::from(relay);
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);

    let client = Client::new();
    let _ = client
        .post(format!("{}/download_success/{}", url, hashed_string))
        .send()
        .await
        .map_err(|e| anyhow!("Failed to send POST request: {}", e))?;

    Ok(())
}
