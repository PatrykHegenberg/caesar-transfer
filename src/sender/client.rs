use crate::error::TransferNotCreatedError;
use crate::transfer_info::transfer_info::TransferInfoRequest;
use hex;
use local_ip_address;
use rand::{seq::SliceRandom, thread_rng};
use reqwest::{Client, StatusCode};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::debug;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn send_info(relay: &str, file: &str) -> Result<String> {
    let sender_ip = local_ip_address::local_ip().unwrap();
    let ip_str = sender_ip.to_owned().to_string();
    let rand_name = generate_random_name();
    let hashed_name = Sha256::digest(rand_name.as_bytes());
    let hashed_string = hex::encode(hashed_name);
    debug!("local ip is: {}", sender_ip);
    debug!("Send Request to: {:?}", relay.to_string());
    let mut map = HashMap::new();
    map.insert("keyword", "test");
    map.insert("files", file);
    map.insert("ip", ip_str.as_str());
    map.insert("name", hashed_string.as_str());

    let client = Client::new();
    let res = client
        .post(format!("{}/upload", relay))
        .json(&map)
        .send()
        .await?;
    if res.status() == StatusCode::CREATED {
        let transfer_info: TransferInfoRequest = res.json().await?;
        debug!("Json Response: {:#?}", transfer_info);
        Ok(rand_name)
    } else {
        Err(Box::new(TransferNotCreatedError::new(
            "Transfer could not be created.",
        )))
    }
}

fn generate_random_name() -> String {
    let mut rng = thread_rng();
    let adjective = adjectives().choose(&mut rng).unwrap();
    // let adjective = adjectives().sample(&mut rng).unwrap();
    let noun1 = nouns1().choose(&mut rng).unwrap();
    let noun2 = nouns2().choose(&mut rng).unwrap();

    format!("{adjective}-{noun1}-{noun2}")
}

fn adjectives() -> &'static [&'static str] {
    static ADJECTIVES: &[&str] = &["funny", "smart", "creative", "friendly", "great"];
    ADJECTIVES
}

fn nouns1() -> &'static [&'static str] {
    static NOUNS1: &[&str] = &["dog", "cat", "flower", "tree", "house"];
    NOUNS1
}

fn nouns2() -> &'static [&'static str] {
    static NOUNS2: &[&str] = &["cookie", "cake", "frosting"];
    NOUNS2
}
