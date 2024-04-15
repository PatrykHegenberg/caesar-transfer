use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

// use crate::http_client;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Serialize, Deserialize)]
struct TransferInfo {
    ip: String,
    name: String,
    body: TransferInfoBody,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransferInfoBody {
    keyword: String,
    files: String,
}
pub async fn download_info(filename: &str) -> Result<()> {
    match reqwest::get(format!("http://192.168.178.43:1323/download/{}", filename)).await {
        Ok(resp) => {
            let json = resp.json::<TransferInfo>().await?;
            println!("Json Response: {:#?}", json);
        }
        Err(err) => {
            println!("Error: {err}");
        }
    }
    // http_client::send_request(
    //     format!("http://192.168.178.43:1323/download/{}", filename).trim(),
    //     "GET",
    //     None,
    // )
    // .await?;
    Ok(())
}
