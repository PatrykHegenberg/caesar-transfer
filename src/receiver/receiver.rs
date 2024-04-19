use crate::transfer_info::transfer_info::TransferInfoRequest;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn download_info(relay: &str, filename: &str) -> Result<()> {
    match reqwest::get(format!("{}/download/{}", relay.to_string(), filename)).await {
        Ok(resp) => {
            let json = resp.json::<TransferInfoRequest>().await?;
            println!("Json Response: {:#?}", json);
        }
        Err(err) => {
            println!("Error: {err}");
        }
    }
    Ok(())
}
