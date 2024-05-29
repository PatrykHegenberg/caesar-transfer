use anyhow::{anyhow, Result};
use std::sync::Arc;

use caesar_core::receiver::start_receiver;
use caesar_core::sender::start_sender;
use rand::{seq::SliceRandom, thread_rng};

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

#[flutter_rust_bridge::frb(sync)]
pub fn generate_random_name() -> String {
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

// #[flutter_rust_bridge::frb(async)]
pub async fn start_rust_sender(name: String, relay: String, files: Vec<String>) -> Result<()> {
    let arc_relay = Arc::new(relay);
    let arc_files = Arc::new(files);
    let outcome = start_sender(name, arc_relay, arc_files).await;
    println!("Start sender result: {:?}", outcome);
    Ok(())
}

pub async fn start_rust_receiver(
    filepath: String,
    relay: String,
    transfername: String,
) -> Result<String> {
    // #[cfg(target_os = "android")]
    let outcome = start_receiver(filepath, relay.as_str(), transfername.as_str())
        .await
        .map_err(|e| anyhow!("Failed to start Caesar receiver: {}", e))?;

    // #[cfg(not(target_os = "android"))]
    // let outcome = start_receiver(relay.as_str(), transfername.as_str())
    //     .await
    //     .map_err(|e| anyhow!("Failed to start Caesar receiver: {}", e))?;
    // Konvertieren Sie outcome zu einem String
    let outcome_string = format!("{:?}", outcome);

    println!("Result of receiver is: {}", outcome_string);
    Ok(outcome_string)
}
