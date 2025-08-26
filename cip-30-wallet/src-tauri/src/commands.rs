use crate::crypto::{
    mnemonic::{generate_mnemonic, validate_mnemonic, mnemonic_to_seed},
    keys::{Bip32PrivateKey, Bip32PublicKey},
    encryption::WalletWrapper,
};
use crate::storage::wallet_store::{WalletStore, WalletMetadata};
use crate::wallet::{WalletError, WalletResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
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
    pub master_public_key: [u8; 32], // Store master public key for address derivation
}

/// Response for wallet creation flow
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletCreateResponse {
    pub wallet_id: String,
    pub mnemonic: Vec<String>,
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
pub async fn check_wallet_exists() -> Result<bool, String> {
    let data_dir = WalletStore::default_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;
    
    let store = WalletStore::new(data_dir)
        .map_err(|e| format!("Failed to create wallet store: {}", e))?;
    
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    Ok(!wallets.is_empty())
}

#[tauri::command]
pub async fn list_wallets() -> Result<Vec<WalletMetadata>, String> {
    let data_dir = WalletStore::default_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;
    
    let store = WalletStore::new(data_dir)
        .map_err(|e| format!("Failed to create wallet store: {}", e))?;
    
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
    
    // Generate master keys from mnemonic to store public key
    let seed = mnemonic_to_seed(&request.mnemonic, "")
        .map_err(|e| format!("Failed to generate seed: {}", e))?;
    
    let master_private_key = Bip32PrivateKey::from_bip39_seed(&seed)
        .map_err(|e| format!("Failed to generate master key: {}", e))?;
    
    let master_public_key = master_private_key.to_public();
    let master_public_key_bytes = master_public_key.to_bytes();
    
    let store = state.wallet_store.lock().unwrap();
    
    // Create wallet wrapper (this encrypts the mnemonic/private key)
    let wallet = WalletWrapper::new(request.mnemonic.clone(), request.name);
    
    // Generate unique wallet ID
    let wallet_id = WalletStore::generate_wallet_id();
    
    // Save encrypted wallet
    store.save_wallet(&wallet_id, &wallet, &request.password)
        .map_err(|e| format!("Failed to save wallet: {}", e))?;
    
    // TODO: Save master public key in extended metadata
    // For now, we'll store it when we enhance the wallet store
    
    Ok(WalletCreateResponse {
        wallet_id,
        mnemonic: request.mnemonic,
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
    // For now, we'll need to temporarily derive the public key
    // In production, we'd store the master public key in extended metadata
    Err("Address derivation not implemented - need to store master public key in wallet metadata".to_string())
}

#[tauri::command]
pub async fn derive_address_from_mnemonic(
    mnemonic: Vec<String>,
    account_index: u32,
    address_index: u32,
) -> Result<AddressInfo, String> {
    // Validate mnemonic
    validate_mnemonic(&mnemonic)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;
    
    // Generate seed and master key
    let seed = mnemonic_to_seed(&mnemonic, "")
        .map_err(|e| format!("Failed to generate seed: {}", e))?;
    
    let master_key = Bip32PrivateKey::from_bip39_seed(&seed)
        .map_err(|e| format!("Failed to generate master key: {}", e))?;
    
    // Derive using Cardano path: m/1852'/1815'/account'/0/address_index
    let purpose = master_key.derive_hardened(1852)
        .map_err(|e| format!("Failed to derive purpose: {}", e))?;
    
    let coin_type = purpose.derive_hardened(1815)
        .map_err(|e| format!("Failed to derive coin type: {}", e))?;
    
    let account = coin_type.derive_hardened(account_index)
        .map_err(|e| format!("Failed to derive account: {}", e))?;
    
    let external_chain = account.derive_soft(0)
        .map_err(|e| format!("Failed to derive external chain: {}", e))?;
    
    let address_key = external_chain.derive_soft(address_index)
        .map_err(|e| format!("Failed to derive address key: {}", e))?;
    
    // Get public key and create address
    let public_key = address_key.to_public();
    let public_key_bytes = public_key.to_bytes();
    
    // For now, return a placeholder address format
    // In production, this would use proper Cardano address encoding with pallas-addresses
    let address = format!("addr1{}", hex::encode(&public_key_bytes[..16]));
    let path = format!("m/1852'/1815'/{}'/{}/{}", account_index, 0, address_index);
    
    Ok(AddressInfo {
        address,
        path,
        account_index,
        address_index,
    })
}

#[tauri::command] 
pub async fn get_addresses_from_mnemonic(
    mnemonic: Vec<String>,
    account_index: u32,
    count: u32,
) -> Result<Vec<AddressInfo>, String> {
    let mut addresses = Vec::new();
    
    // Validate mnemonic once
    validate_mnemonic(&mnemonic)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;
    
    // Generate seed and derive account key once for efficiency
    let seed = mnemonic_to_seed(&mnemonic, "")
        .map_err(|e| format!("Failed to generate seed: {}", e))?;
    
    let master_key = Bip32PrivateKey::from_bip39_seed(&seed)
        .map_err(|e| format!("Failed to generate master key: {}", e))?;
    
    // Derive to account level: m/1852'/1815'/account'
    let purpose = master_key.derive_hardened(1852)
        .map_err(|e| format!("Failed to derive purpose: {}", e))?;
    
    let coin_type = purpose.derive_hardened(1815)
        .map_err(|e| format!("Failed to derive coin type: {}", e))?;
    
    let account = coin_type.derive_hardened(account_index)
        .map_err(|e| format!("Failed to derive account: {}", e))?;
    
    let external_chain = account.derive_soft(0)
        .map_err(|e| format!("Failed to derive external chain: {}", e))?;
    
    // Derive each address
    for i in 0..count {
        let address_key = external_chain.derive_soft(i)
            .map_err(|e| format!("Failed to derive address key {}: {}", i, e))?;
        
        let public_key = address_key.to_public();
        let public_key_bytes = public_key.to_bytes();
        
        let address = format!("addr1{}", hex::encode(&public_key_bytes[..16]));
        let path = format!("m/1852'/1815'/{}'/{}/{}", account_index, 0, i);
        
        addresses.push(AddressInfo {
            address,
            path,
            account_index,
            address_index: i,
        });
    }
    
    Ok(addresses)
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