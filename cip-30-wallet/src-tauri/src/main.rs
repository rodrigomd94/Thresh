// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn main() {
    // Always run the unified app with both system tray and native messaging
    // The app will automatically handle both manual launches and browser communication
    thresh_lib::run_native_messaging();
}
