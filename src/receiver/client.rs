use crate::transfer_info::transfer_info::TransferInfoRequest;
use reqwest::Client;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::copy;
use tracing::{debug, error, info};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct TransferNotFoundError {
    message: String,
}

impl TransferNotFoundError {
    fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for TransferNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TransferNotFoundError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub async fn download_info(relay: &str, filename: &str) -> Result<TransferInfoRequest> {
    match reqwest::get(format!("{}/download/{}", relay.to_string(), filename)).await {
        Ok(resp) => {
            let json = resp.json::<TransferInfoRequest>().await?;
            debug!("Json Response: {:#?}", json);
            if json.message == "error".to_string() {
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

    let resp = reqwest::get(format!("http://{}:1300/download_file", &transfer_info.ip)).await?;
    if !resp.status().is_success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to download file from {}", &transfer_info.ip),
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
    match reqwest::get(format!("http://{}:1300/ping", sender)).await {
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

pub async fn signal_success(sender: &String) -> Result<()> {
    debug!("Signaling shutdown to {:#?}", sender);
    let client = Client::new();
    let _ = client
        .post(format!("http://{}:1300/shutdown", sender))
        .send()
        .await?;
    Ok(())
}
