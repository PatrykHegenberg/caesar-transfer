use caesar_core::{receiver, relay::server::start_ws, sender};
use std::sync::Arc;

#[tauri::command]
async fn send(relay: Option<String>, files: Vec<String>) {
    let relay_string = relay.unwrap_or_else(|| "default_relay_address".to_string());
    log::info!("Using relay: {}", relay_string); 
    let relay_arc = Arc::new(relay_string);
    let files_arc = Arc::new(files);
    sender::start_sender(relay_arc, files_arc).await;
}

// #[tauri::command]
// async fn receive(relay: Option<String>, overwrite: bool, name: String) {
//     let relay_string = relay.unwrap_or_else(|| "default_relay_address".to_string());
//     receiver::start_receiver(&relay_string, &name).await;
// }

#[tauri::command]
async fn serve(port: Option<i32>, listen_address: Option<String>) {
    let address = listen_address.unwrap_or_else(|| "localhost".to_string());
    let port = port.unwrap_or(8080);
    start_ws(&port, &address).await;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![send, serve])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
