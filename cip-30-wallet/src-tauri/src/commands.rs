use crate::crypto::{
    mnemonic::{generate_mnemonic, validate_mnemonic, mnemonic_to_seed},
    keys::{Bip32PrivateKey, Bip32PublicKey},
    encryption::WalletWrapper,
};
use crate::storage::wallet_store::{WalletStore, WalletMetadata};
use crate::wallet::{WalletError, WalletResult, derive_addresses_from_wallet};
use pallas_crypto::key::ed25519;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, sync::Mutex};
use tauri::State;
/// Application state to hold the wallet store
pub struct AppState {
    pub wallet_store: Mutex<WalletStore>,
}

/// Extended wallet metadata with master public key for address derivation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtendedWalletMetadata {
    pub wallet_id: String,
    pub name: String,
    pub created_at: i64,
    pub master_public_key: Vec<u8>, // Store extended master public key for address derivation
}

/// Response for wallet creation flow
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletCreateResponse {
    pub wallet_id: String,
}

/// Response for wallet info
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfoResponse {
    pub wallet_id: String,
    pub name: String,
    pub created_at: i64,
}

/// Address info with derivation path
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub path: String,
    pub account_index: u32,
    pub address_index: u32,
}

/// Request for wallet operations
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub name: String,
    pub password: String,
    pub mnemonic: Vec<String>,
}

// Wallet Management Commands

#[tauri::command]
pub async fn check_wallet_exists(state: State<'_, AppState>) -> Result<bool, String> {
    let store = state.wallet_store.lock().unwrap();
    
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    eprintln!("Found {} wallets", wallets.len());
    for wallet in &wallets {
        eprintln!("Wallet: {} ({})", wallet.name, wallet.wallet_id);
    }
    
    Ok(!wallets.is_empty())
}

#[tauri::command]
pub async fn list_wallets(state: State<'_, AppState>) -> Result<Vec<WalletMetadata>, String> {
    let store = state.wallet_store.lock().unwrap();
    
    store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))
}

#[tauri::command]
pub async fn get_wallet_info(
    wallet_id: String,
    state: State<'_, AppState>
) -> Result<WalletInfoResponse, String> {
    let store = state.wallet_store.lock().unwrap();
    
    // Try to load wallet metadata
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    let wallet_meta = wallets.iter()
        .find(|w| w.wallet_id == wallet_id)
        .ok_or_else(|| "Wallet not found".to_string())?;
    
    Ok(WalletInfoResponse {
        wallet_id: wallet_meta.wallet_id.clone(),
        name: wallet_meta.name.clone(),
        created_at: wallet_meta.created_at,
    })
}

// Mnemonic Commands

#[tauri::command]
pub async fn generate_new_mnemonic(word_count: usize) -> Result<Vec<String>, String> {
    if ![12, 15, 18, 21, 24].contains(&word_count) {
        return Err("Invalid word count. Must be 12, 15, 18, 21, or 24".to_string());
    }
    
    generate_mnemonic(word_count)
        .map_err(|e| format!("Failed to generate mnemonic: {}", e))
}

#[tauri::command]
pub async fn validate_mnemonic_phrase(mnemonic: Vec<String>) -> Result<bool, String> {
    match validate_mnemonic(&mnemonic) {
        Ok(()) => Ok(true),
        Err(WalletError::InvalidMnemonic) => Ok(false),
        Err(e) => Err(format!("Validation error: {}", e)),
    }
}

// Wallet Operations

#[tauri::command]
pub async fn create_wallet(
    request: CreateWalletRequest,
    state: State<'_, AppState>
) -> Result<WalletCreateResponse, String> {
    // Validate mnemonic first
    validate_mnemonic(&request.mnemonic)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;
    
    // Generate account-level keys from mnemonic to store public key
    let seed = mnemonic_to_seed(&request.mnemonic, "")
        .map_err(|e| format!("Failed to generate seed: {}", e))?;
    
    let master_private_key = Bip32PrivateKey::from_bip39_seed(&seed)
        .map_err(|e| format!("Failed to generate master key: {}", e))?;
    
    // Derive to account level: m/1852'/1815'/0' (hardened derivation)
    let purpose = master_private_key.derive(1852 | 0x80000000);  // Hardened
    let coin_type = purpose.derive(1815 | 0x80000000);           // Hardened  
    let account = coin_type.derive(0 | 0x80000000);              // Hardened, account 0
    
    // Store the account-level public key (can derive external/internal chains and addresses)
    let account_public_key = account.to_public();
    let account_public_key_bytes = account_public_key.to_extended_bytes();
    
    let store = state.wallet_store.lock().unwrap();
    
    // Create wallet wrapper (this encrypts the mnemonic/private key)
    let wallet = WalletWrapper::new(request.mnemonic.clone(), request.name);
    
    // Generate unique wallet ID
    let wallet_id = WalletStore::generate_wallet_id();
    
    // Save encrypted wallet
    eprintln!("Saving wallet with ID: {}", wallet_id);
    match store.save_wallet(&wallet_id, &wallet, &request.password, &account_public_key_bytes) {
        Ok(()) => eprintln!("Wallet saved successfully"),
        Err(e) => {
            eprintln!("Failed to save wallet: {}", e);
            return Err(format!("Failed to save wallet: {}", e));
        }
    }
    
    // TODO: Save master public key in extended metadata
    // For now, we'll store it when we enhance the wallet store
    
    Ok(WalletCreateResponse {
        wallet_id,
    })
}

// Address Derivation Commands (using public keys only)

#[tauri::command]
pub async fn derive_address_from_wallet(
    wallet_id: String,
    account_index: u32,
    address_index: u32,
    state: State<'_, AppState>
) -> Result<AddressInfo, String> {
    let store = state.wallet_store.lock().unwrap();
    
    // Use shared function to derive addresses, then return the specific one
    let addresses = derive_addresses_from_wallet(&store, &wallet_id, account_index, address_index + 1)?;
    
    // Return the address at the requested index
    addresses.into_iter()
        .find(|addr| addr.address_index == address_index)
        .ok_or_else(|| "Failed to derive address at specified index".to_string())
}


#[tauri::command]
pub async fn get_addresses_from_wallet(
    wallet_id: String,
    account_index: u32,
    count: u32,
    state: State<'_, AppState>
) -> Result<Vec<AddressInfo>, String> {
    let store = state.wallet_store.lock().unwrap();
    derive_addresses_from_wallet(&store, &wallet_id, account_index, count)
}

// Utility Commands

#[tauri::command]
pub async fn validate_wallet_password(
    wallet_id: String,
    password: String,
    state: State<'_, AppState>
) -> Result<bool, String> {
    let store = state.wallet_store.lock().unwrap();
    
    match store.load_wallet(&wallet_id, &password) {
        Ok(_) => Ok(true),
        Err(WalletError::InvalidPassword) => Ok(false),
        Err(e) => Err(format!("Error validating password: {}", e)),
    }
}

// Helper function to initialize app state
pub fn create_app_state() -> Result<AppState, String> {
    let data_dir = WalletStore::default_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;
    
    let wallet_store = WalletStore::new(data_dir)
        .map_err(|e| format!("Failed to create wallet store: {}", e))?;
    
    Ok(AppState {
        wallet_store: Mutex::new(wallet_store),
    })
}