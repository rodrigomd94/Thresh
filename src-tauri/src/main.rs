use std::{fs, path::PathBuf, sync::Arc};

use tauri::{
    ipc::RemoteDomainAccessScope, CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder,
    WindowUrl,
};
mod utils;
mod commands;
use frost_tauri::commands::frost::create_round1_key_package;
use frost_tauri::commands::cip30::{
    get_change_address, get_collateral, sign_data, get_network_id, get_reward_address, get_unused_addresses,
    get_used_addresses, get_utxos, sign_tx, submit_tx,
};

#[tauri::command]
fn inject_script(app: tauri::AppHandle, window: String, script: String) {
    let selected_window = app.get_window(window.as_str()).unwrap();
    selected_window
        .eval(&script.as_str())
        .expect("Failed to inject script");
    // selected_window.open_devtools();
}

#[tauri::command]
fn init_wallet_api(app: tauri::AppHandle, window: String) {
    // Path to the bundled script
    let script_path = PathBuf::from("../dist/InjectScript.es.js ");
    // Read the script content
    let script_content =
        fs::read_to_string(script_path).expect("Failed to read the bundled script");
    let selected_window = app.get_window(window.as_str()).unwrap();
    selected_window
        .eval(&script_content.as_str())
        .expect("Failed to inject script");
    selected_window.open_devtools();

    let main_window = app.get_window("main").unwrap();
    main_window
        .eval(&script_content.as_str())
        .expect("Failed to inject script");
}

#[tauri::command]
fn open_browser(app_handle: tauri::AppHandle, url: String) {
    WindowBuilder::new(
        &app_handle,
        "webview",
        WindowUrl::External(url.parse().unwrap()),
    )
    .title("Embedded Browser")
    .build()
    .expect("Failed to build new window");
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
        open_browser,
        inject_script,
        init_wallet_api,
        sign_data,
        sign_tx,
        submit_tx,
        get_utxos,
        get_collateral,
        get_used_addresses,
        get_unused_addresses,
        get_change_address,
        get_reward_address,
        get_network_id,
        create_round1_key_package,
        ])
        .setup(|app| {
            app.ipc_scope().configure_remote_access(
                RemoteDomainAccessScope::new("www.app.minswap.org")
                    .add_window("webview")
                    .enable_tauri_api()
                    .add_plugin("event")
                    .add_plugin("path"),
            );
            let main_window = app.get_window("main").unwrap();
            main_window
                .eval("window.yourCustomObject = { key: 'value' };")
                .unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
