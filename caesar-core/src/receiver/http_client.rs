use anyhow::{anyhow, Result};
use hex;
use reqwest::{self, Client};
use sha2::{Digest, Sha256};

use crate::relay::transfer::TransferResponse;

/// Fetches download information from the relay server for the given file name.
///
/// # Arguments
///
/// * `relay` - The URL of the relay server.
/// * `name` - The name of the file.
///
/// # Returns
///
/// A future that resolves to a `Result` containing the download information
/// if the request is successful, or an error if the request fails.
pub async fn download_info(relay: &str, name: &str) -> Result<TransferResponse> {
    // Convert the relay server URL and file name to strings
    let url = String::from(relay);
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);

    // Send a GET request to the relay server with the file name hash as a query parameter
    let resp = reqwest::get(format!("{}/download/{}", url, hashed_string))
        .await
        // If the request fails, return an error with the reason
        .map_err(|e| anyhow!("Failed to send GET request: {}", e))?;

    // Parse the response body as JSON into a `TransferResponse` struct
    resp.json::<TransferResponse>()
        .await
        // If the JSON parsing fails, return an error with the reason
        .map_err(|e| anyhow!("Failed to parse JSON response: {}", e))
}


/// Notifies the relay server that the file download was successful for the given file name.
///
/// # Arguments
///
/// * `relay` - The URL of the relay server.
/// * `name` - The name of the file.
///
/// # Returns
///
/// A future that resolves to a `Result` containing `Ok(())` if the request is successful,
/// or an error if the request fails.
pub async fn download_success(relay: &str, name: &str) -> Result<()> {
    // Convert the relay server URL and file name to strings
    let url = String::from(relay);
    let hashed_name = Sha256::digest(name.as_bytes());
    let hashed_string = hex::encode(hashed_name);

    // Create a new HTTP client
    let client = Client::new();

    // Send a POST request to the relay server with the file name hash as a query parameter
    let _ = client
        .post(format!("{}/download_success/{}", url, hashed_string))
        .send()
        .await
        // If the request fails, return an error with the reason
        .map_err(|e| anyhow!("Failed to send POST request: {}", e))?;

    // Return Ok(()) if the request was successful
    Ok(())
}
