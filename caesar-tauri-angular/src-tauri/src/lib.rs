use caesar_core::{receiver, relay::server::start_ws, sender::{self, util::generate_random_name}};
use std::sync::Arc;
use tauri::{AppHandle, Manager};


#[tauri::command]
async fn send(app_handle: tauri::AppHandle, relay: Option<String>, files: Vec<String>) {
    let relay_string = relay.unwrap_or_else(|| "default_relay_address".to_string());
    log::info!("Using relay: {}", relay_string); 
    let relay_arc = Arc::new(relay_string);
    let files_arc = Arc::new(files);
    let transfer_name = generate_random_name();

    app_handle.emit("transfer_name_event", transfer_name.clone())
    .expect("Failed to emit event");

    sender::start_sender(relay_arc, files_arc, transfer_name.clone()).await;
}

#[tauri::command]
async fn receive(relay: Option<String>, name: String) -> Result<(), String> {
    let relay_string = relay.unwrap_or_else(|| "default_relay_address".to_string());
    
    match receiver::start_receiver(&relay_string, &name).await {
        Ok(_) => {
            println!("Receiver started successfully.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to start receiver: {:?}", e);
            Err(format!("Failed to start receiver: {:?}", e))
        }
    }
}

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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![send, serve, receive])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
