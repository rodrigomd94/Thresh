// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn main() {
    // Check if running as native messaging host
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "--native-messaging" {
        // Run in native messaging mode
        cip_30_wallet_lib::run_native_messaging();
    } else {
        // Run normal Tauri app
        cip_30_wallet_lib::run()
    }
}
