use reqwest::{blocking::Client, StatusCode};
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn download_info(client: Client, filename: &str) -> Result<()> {
    let res = client
        .get(format!("http://192.168.178.43:1323/download/{}", filename))
        .send()?;

    if res.status() == StatusCode::OK {
        let json: HashMap<String, String> = res.json()?;
        println!("JSON Response: {:?}", json);
    }
    Ok(())
}
