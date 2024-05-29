//! This module is only for Rinf demonstrations.
//! You might want to remove this module in production.
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::messages;
use caesar_core::receiver::start_receiver as start_caesar_receiver;
use caesar_core::sender::start_sender as start_caesar_sender;
use caesar_core::sender::util::generate_random_name;
use rinf::debug_print;

// Disabled when applied as Rinf template.
const SHOULD_DEMONSTRATE: bool = false;

// Using the `cfg` macro enables conditional statement.
#[cfg(debug_assertions)]
const IS_DEBUG_MODE: bool = true;
#[cfg(not(debug_assertions))]
const IS_DEBUG_MODE: bool = false;

pub async fn start_sender() -> Result<bool> {
    use messages::ressource::*;
    let rand_name = generate_random_name();
    debug_print!("random name: {}", rand_name);

    Name {
        rand_name: rand_name.clone(),
    }
    .send_signal_to_dart();
    debug_print!("Starting sender");
    let mut receiver = Files::get_dart_signal_receiver();
    while let Some(dart_signal) = receiver.recv().await {
        let data = dart_signal.message;
        debug_print!("Files are: {:?}", data.filenames);

        let out = start_caesar_sender(
            rand_name.clone(),
            Arc::new(data.relay),
            Arc::new(data.filenames),
        )
        .await;
        debug_print!("Start sender result: {:?}", out);
    }
    Ok(true)
}

pub async fn start_receiver() -> Result<()> {
    use messages::ressource::*;
    let mut receiver = TransferName::get_dart_signal_receiver();
    debug_print!("Starting Receiver");
    while let Some(dart_signal) = receiver.recv().await {
        debug_print!("In loop");
        let data: TransferName = dart_signal.message;
        let transfer_name = data.transfer_name;
        let relay = data.relay;
        debug_print!("Got transfer name: {}", transfer_name);
        debug_print!("Got transfer relay: {}", relay);

        let out = start_caesar_receiver(relay.as_str(), transfer_name.as_str())
            .await
            .map_err(|e| anyhow!("Failed to start Caesar receiver: {}", e))?;

        debug_print!("Start receiver result: {:?}", out);
    }
    Ok(())
}

pub async fn print_progress(message: String) -> Result<()> {
    debug_print!("Progress: {}", message);
    Ok(())
}
// pub async fn print_progress(mut rx: mpsc::Receiver<String>) -> Result<()> {
//     while let Some(message) = rx.recv().await {
//         debug_print!("Progress: {}", message);
//     }
//     Ok(())
// }
