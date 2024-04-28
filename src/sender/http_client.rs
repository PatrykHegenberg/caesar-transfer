use std::collections::HashMap;
use tracing::error;

use local_ip_address::{local_ip, local_ipv6};
use reqwest::blocking::Client;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn send_info(relay: &str, name: &str, room_id: &str) -> Result<String> {
    let sender_ip = match local_ipv6() {
        Ok(ip) => ip,
        Err(_) => match local_ip() {
            Ok(ip) => ip,
            Err(e) => {
                error!("Error getting local ip: {e:?}");
                return Err(Box::new(e));
            }
        },
    };
    let ip_str = sender_ip.to_owned().to_string();
    let mut map = HashMap::new();
    map.insert("name", String::from(name));
    map.insert("ip", ip_str);
    map.insert("room_id", String::from(room_id));
    let client = Client::new();
    let _ = match client.post(format!("{}/upload", relay)).json(&map).send() {
        Ok(_) => Ok(room_id.to_string()),
        Err(e) => Err(Box::new(e)),
    };
    Ok("".to_string())
}
