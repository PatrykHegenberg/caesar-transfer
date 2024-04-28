use hex;
use reqwest;
use sha2::{Digest, Sha256};
use tracing::error;

use crate::relay::transfer::Transfer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn download_info(relay: &str, name: &str) -> Result<String> {
    let url = String::from("http://") + relay;
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);

    match reqwest::get(format!("{}/download/{}", url, hashed_string)).await {
        Ok(resp) => match resp.json::<Transfer>().await {
            Ok(res) => Ok(res.room_id),
            Err(e) => Err(Box::new(e)),
        },
        Err(err) => {
            error!("Error: {err}");
            Err(Box::new(err))
        }
    }
}
