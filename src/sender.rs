use crate::http_client;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn send_info(file: &str) -> Result<()> {
    let mut map = HashMap::new();
    map.insert("keyword", "test");
    map.insert("files", file);

    let json_data = serde_json::to_string(&map)?;

    http_client::send_request(
        "http://192.168.178.43:1323/upload".trim(),
        "POST",
        Some(json_data),
    )
    .await?;

    Ok(())
}
