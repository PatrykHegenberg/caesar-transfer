mod cli;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use std::collections::HashMap;

pub use crate::cli::*;

fn main() {
    let client = Client::new();
    let cli = cli::Cli::new();
    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path);
    }

    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    match &cli.command {
        Some(Commands::Send { file }) => {
            let mut map = HashMap::new();
            map.insert("keyword", "test");
            let files = match file {
                Some(name) => name.trim(),
                None => "test.txt",
            };
            map.insert("files", files);
            let res = client
                .post("http://192.168.178.43:1323/upload")
                .json(&map)
                .send()
                .expect("Error sending request");
            if res.status() == StatusCode::OK {
                let json: HashMap<String, String> =
                    res.json().expect("Error parsing JSON response");
                println!("JSON Response: {:?}", json);
            } else {
                println!("Error: Failed to send request");
            }
        }
        None => {
            let filename = match cli.name {
                Some(name) => name,
                None => "None".to_string(),
            };
            let res = client
                .get(format!("http://192.168.178.43:1323/download/{}", filename))
                .send()
                .expect("Error sending request");
            if res.status() == StatusCode::OK {
                let json: HashMap<String, String> =
                    res.json().expect("Error parsing JSON response");
                println!("Json Response: {:?}", json);
            }
        }
    }
}
