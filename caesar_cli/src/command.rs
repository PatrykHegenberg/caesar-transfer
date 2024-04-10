use reqwest::{blocking::Client, StatusCode};
use std::collections::HashMap;

pub fn send_info(client: Client, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    map.insert("keyword", "test");
    map.insert("files", file);

    let res = client
        .post("http://192.168.178.43:1323/upload")
        .json(&map)
        .send()?;

    if res.status() == StatusCode::OK {
        let json: HashMap<String, String> = res.json()?;
        println!("JSON Response: {:?}", json);
    } else {
        println!("Error: Failed to send request");
    }
    Ok(())
}

pub fn download_info(client: Client, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let res = client
        .get(format!("http://192.168.178.43:1323/download/{}", filename))
        .send()?;

    if res.status() == StatusCode::OK {
        let json: HashMap<String, String> = res.json()?;
        println!("JSON Response: {:?}", json);
    }
    Ok(())
}
