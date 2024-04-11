use crate::http_client;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn download_info(filename: &str) -> Result<()> {
    http_client::send_request(
        format!("http://192.168.178.43:1323/download/{}", filename).trim(),
        "GET",
        None,
    )
    .await?;
    Ok(())
}
