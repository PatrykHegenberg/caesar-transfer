use std::collections::HashMap;
use tracing::{debug, error};

use local_ip_address::{local_ip, local_ipv6};
use reqwest::blocking::Client;
use tokio::task;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn send_info(relay: &str, name: &str, room_id: &str, is_local: bool) -> Result<String> {
    let url = relay.to_string();
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
    let map = {
        let mut map = HashMap::new();
        map.insert("name", String::from(name));
        map.insert("ip", ip_str);
        if is_local {
            map.insert("local_room_id", String::from(room_id));
            map.insert("relay_room_id", String::from(""));
        } else {
            map.insert("relay_room_id", String::from(room_id));
            map.insert("local_room_id", String::from(""));
        }
        map
    };
    let room_id = room_id.to_string();

    debug!("Trying to send Request.");
    let result: Result<String> = task::spawn_blocking(move || {
        let client = Client::new();
        client
            .put(format!("{}/upload", url))
            .json(&map)
            .send()?
            .text()?
            .to_string();
        Ok(room_id)
    })
    .await?;

    Ok(result?)
}
