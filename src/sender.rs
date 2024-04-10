use reqwest::{blocking::Client, StatusCode};
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn send_info(client: Client, file: &str) -> Result<()> {
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
        println!("Error: Faile to send request");
    }
    Ok(())
}
