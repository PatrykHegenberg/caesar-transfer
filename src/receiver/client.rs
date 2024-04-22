use crate::error::TransferNotFoundError;
use crate::transfer_info::transfer_info::TransferInfoRequest;
use hex;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::copy;
use tracing::{debug, error, info};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn download_info(relay: &str, name: &str) -> Result<TransferInfoRequest> {
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);

    match reqwest::get(format!("{}/download/{}", relay, hashed_string)).await {
        Ok(resp) => {
            let json = resp.json::<TransferInfoRequest>().await?;
            debug!("Json Response: {:#?}", json);
            if json.message == *"error" {
                Err(Box::new(TransferNotFoundError::new(
                    "no transfer with given name found",
                )))
            } else {
                debug!("Got Positive response");
                Ok(json)
            }
        }
        Err(err) => {
            error!("Error: {err}");
            Err(Box::new(err))
        }
    }
}

pub async fn download_file(transfer_info: &TransferInfoRequest, overwrite: &bool) -> Result<()> {
    if !*overwrite && File::open(&transfer_info.body.files).is_ok() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("File '{} already exists", &transfer_info.body.files),
        )));
    }

    let resp = reqwest::get(format!(
        "http://{}:8100/download_file",
        &transfer_info.body.ip
    ))
    .await?;
    if !resp.status().is_success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to download file from {}", &transfer_info.body.ip),
        )));
    }
    let mut dest = File::create(&transfer_info.body.files)?;
    let content = resp.text().await?;
    copy(&mut content.as_bytes(), &mut dest)?;
    info!("Download complete");
    Ok(())
}

pub async fn ping_sender(sender: &String) -> Result<bool> {
    debug!("Pinging Sender on {:#?}", sender);
    match reqwest::get(format!("http://{}:8100/ping", sender)).await {
        Ok(resp) => {
            debug!("Sender directly reachable");
            debug!("Response is: {:#?}", resp);
            Ok(true)
        }
        Err(err) => {
            error!("Error: {err}");
            Err(Box::new(err))
        }
    }
}

pub async fn signal_success_relay(relay: &str, name: &str) -> Result<()> {
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);
    debug!("Signaling success to {:#?}", relay);
    let client = Client::new();
    let _ = client
        .post(format!("{}/download_success/{}", relay, hashed_string))
        .send()
        .await?;
    Ok(())
}

pub async fn signal_success_sender(sender: &String) -> Result<()> {
    debug!("Signaling shutdown to {:#?}", sender);
    let client = Client::new();
    let _ = client
        .post(format!("http://{}:8100/shutdown", sender))
        .send()
        .await?;
    Ok(())
}
