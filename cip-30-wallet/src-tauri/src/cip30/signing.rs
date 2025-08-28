use crate::commands::{AppState, prompt_transaction_password, get_signing_key_from_wallet};
use tauri::{AppHandle, Manager};

/// Get the private key for transaction signing with UI password prompt
pub async fn get_private_key_for_signing(
    tx_hex: &str,
    app_state: &AppState,
    app_handle: &AppHandle,
) -> Result<[u8; 32], String> {
    eprintln!("[SIGNING] Starting private key retrieval with UI");
    
    // Get the first wallet (in a real app, this would be the selected wallet)
    let store = app_state.wallet_store.lock().unwrap();
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    if wallets.is_empty() {
        return Err("No wallets available for signing".to_string());
    }
    
    let wallet_id = wallets[0].wallet_id.clone();
    drop(store); // Release the lock
    
    // Get the main window
    let window = app_handle.get_webview_window("main")
        .ok_or("Failed to get main window")?;
    
    // Prompt for password
    eprintln!("[SIGNING] Prompting for password");
    let password = prompt_transaction_password(
        tx_hex.to_string(),
        wallet_id.clone(),
        window,
        app_handle.clone(),
    ).await?;
    
    eprintln!("[SIGNING] Password received, getting signing key");
    
    // Get the signing key (this is the private key you wanted)
    let private_key = get_signing_key_from_wallet(&wallet_id, &password, app_state)?;
    
    eprintln!("[SIGNING] Private key retrieved successfully");
    
    Ok(private_key)
}