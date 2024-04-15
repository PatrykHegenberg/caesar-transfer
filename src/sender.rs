// use crate::http_client;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn send_info(file: &str) -> Result<()> {
    let mut map = HashMap::new();
    map.insert("keyword", "test");
    map.insert("files", file);

    // let json_data = serde_json::to_string(&map)?;

    let client = Client::new();
    let res = client
        .post("http://192.168.178.43:1323/upload")
        .json(&map)
        .send()
        .await?;
    if res.status() == StatusCode::CREATED {
        let json: Value = res.json().await?;
        println!("Json Response: {:#?}", json);
    } else {
        println!("Error reading response");
    }
    // http_client::send_request(
    //     "http://192.168.178.43:1323/upload".trim(),
    //     "POST",
    //     Some(json_data),
    // )
    // .await?;

    Ok(())
}
