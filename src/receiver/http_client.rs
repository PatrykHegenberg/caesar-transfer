use hex;
use reqwest::{self, Client};
use sha2::{Digest, Sha256};
use tracing::error;

use crate::relay::transfer::TransferResponse;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn download_info(relay: &str, name: &str) -> Result<TransferResponse> {
    let url = String::from(relay);
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);

    match reqwest::get(format!("{}/download/{}", url, hashed_string)).await {
        Ok(resp) => match resp.json::<TransferResponse>().await {
            Ok(res) => Ok(res),
            Err(e) => Err(Box::new(e)),
        },
        Err(err) => {
            error!("Error: {err}");
            Err(Box::new(err))
        }
    }
}

pub async fn download_success(relay: &str, name: &str) -> Result<()> {
    let url = String::from(relay);
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);

    let client = Client::new();
    let _ = client
        .post(format!("{}/download_success/{}", url, hashed_string))
        .send()
        .await?;
    Ok(())
}
