mod native_messaging;
mod cip30;
mod wallet;
mod crypto;
mod storage;
mod commands;

use native_messaging::{NativeMessage, start_native_messaging, send_message};
use cip30::handle_cip30_request;
use commands::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize app state
    let app_state = create_app_state()
        .expect("Failed to initialize app state");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Wallet management commands
            check_wallet_exists,
            list_wallets,
            get_wallet_info,
            // Mnemonic commands  
            generate_new_mnemonic,
            validate_mnemonic_phrase,
            // Wallet operations
            create_wallet,
            validate_wallet_password,
            // Address derivation commands
            derive_address_from_wallet,
            get_addresses_from_wallet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_native_messaging() {
    eprintln!("Starting native messaging mode...");
    
    // Initialize app state for native messaging
    let app_state = create_app_state()
        .expect("Failed to initialize app state for native messaging");
    
    let rx = start_native_messaging();
    
    // Main message loop
    loop {
        match rx.recv() {
            Ok(message) => {
                eprintln!("[RECV] Raw message: {:?}", message);
                
                let response = match message {
                    NativeMessage::Ping { request_id } => {
                        eprintln!("[PING] Received ping with requestId: {:?}", request_id);
                        NativeMessage::Pong { request_id }
                    },
                    NativeMessage::Cip30Request { request_id, method, params } => {
                        eprintln!("[CIP30] Method: {}", method);
                        eprintln!("[CIP30] RequestId: {:?}", request_id);
                        eprintln!("[CIP30] Params: {}", serde_json::to_string_pretty(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
                        
                        // Handle CIP-30 request
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let result = rt.block_on(handle_cip30_request(&method, params.clone(), Some(&app_state)));
                        
                        match result {
                            Ok(data) => {
                                eprintln!("[CIP30] Response data: {}", serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Invalid JSON".to_string()));
                                NativeMessage::Response {
                                    request_id,
                                    data,
                                    error: None,
                                }
                            },
                            Err(error) => {
                                eprintln!("[CIP30] Error: {}", error);
                                NativeMessage::Response {
                                    request_id,
                                    data: serde_json::Value::Null,
                                    error: Some(error),
                                }
                            },
                        }
                    },
                    _ => {
                        eprintln!("[ERROR] Unexpected message type");
                        continue;
                    }
                };
                
                // Send response
                eprintln!("[SEND] Sending response: {:?}", response);
                if let Err(e) = send_message(&response) {
                    eprintln!("[ERROR] Failed to send response: {}", e);
                } else {
                    eprintln!("[SEND] Response sent successfully");
                }
            },
            Err(e) => {
                eprintln!("[ERROR] Failed to receive message: {}", e);
                break;
            }
        }
    }
    
    eprintln!("Native messaging stopped");
}
