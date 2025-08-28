mod native_messaging;
mod cip30;
mod wallet;
mod crypto;
mod storage;
mod commands;
mod config;
mod utxorpc;

use native_messaging::{NativeMessage, start_native_messaging, send_message};
use cip30::handle_cip30_request;
use commands::*;
use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder, Manager, AppHandle};

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
        .setup(setup_system_tray)
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
            // Network configuration commands
            get_current_network,
            set_runtime_network,
            save_network_to_config,
            reset_runtime_network,
            get_app_config,
            get_network_info,
            // Transaction signing commands
            prompt_transaction_password,
            submit_transaction_password,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_native_messaging() {
    eprintln!("Starting native messaging with Tauri UI support...");
    
    // Initialize app state for native messaging
    let app_state = create_app_state()
        .expect("Failed to initialize app state for native messaging");
    
    // We'll start the native messaging handler after the Tauri app is initialized
    // so we have access to the app handle for UI operations
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(|app| {
            setup_system_tray(app)?;
            
            // Start native messaging in background with app handle
            let app_handle = app.handle().clone();
            
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(start_native_messaging_handler(app_handle));
            });
            
            Ok(())
        })
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
            // Network configuration commands
            get_current_network,
            set_runtime_network,
            save_network_to_config,
            reset_runtime_network,
            get_app_config,
            get_network_info,
            // Transaction signing commands
            prompt_transaction_password,
            submit_transaction_password,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// System Tray Setup
fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create tray menu items
    let show_window = MenuItem::with_id(app, "show", "Show Thresh Wallet", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    // Create tray menu
    let menu = Menu::with_items(app, &[&show_window, &quit_item])?;
    
    // Build tray icon
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_tray_menu_event)
        .on_tray_icon_event(handle_tray_icon_event)
        .build(app)?;
    
    Ok(())
}

// Handle tray menu events
fn handle_tray_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        "show" => {
            show_main_window(app);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

// Handle tray icon events (clicks)
fn handle_tray_icon_event(tray: &tauri::tray::TrayIcon, event: tauri::tray::TrayIconEvent) {
    if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
        if button == tauri::tray::MouseButton::Left {
            // Show window on left click
            if let Some(app) = tray.app_handle().get_webview_window("main") {
                let _ = app.show();
                let _ = app.set_focus();
            }
        }
    }
}

// Helper function to show main window
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

// Helper function to hide main window  
pub fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

// Native messaging handler with app handle access
async fn start_native_messaging_handler(app_handle: AppHandle) {
    eprintln!("Starting native messaging handler with UI access...");
    
    let app_state = app_handle.state::<AppState>();
    let rx = start_native_messaging();
    
    // Main native messaging loop with app handle access
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
                        
                        // Now we have app_handle available for all requests including signTx
                        let result = handle_cip30_request(&method, params.clone(), Some(app_state.inner()), Some(app_handle.clone())).await;
                        
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
