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
use pallas_addresses::Address;
use pallas_crypto::PublicKey;
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
    let store = state.wallet_store.lock().unwrap();
    
    // Get wallet metadata with public key
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    let wallet_meta = wallets.iter()
        .find(|w| w.wallet_id == wallet_id)
        .ok_or_else(|| "Wallet not found".to_string())?;
    
    // Create account-level public key from stored bytes (already at m/1852'/1815'/0')
    let mut key_bytes = [0u8; 64];
    if wallet_meta.master_public_key.len() != 64 {
        return Err("Invalid public key length".to_string());
    }
    key_bytes.copy_from_slice(&wallet_meta.master_public_key);
    
    let account_public_key = Bip32PublicKey::from_extended_bytes(&key_bytes)
        .map_err(|e| format!("Failed to create account public key: {}", e))?;
    
    // Derive external chain: m/1852'/1815'/0'/0 (soft derivation)
    let external_chain = account_public_key.derive(0)
        .map_err(|e| format!("Failed to derive external chain: {}", e))?;
    
    // Derive address: m/1852'/1815'/0'/0/address_index (soft derivation)
    let address_key = external_chain.derive(address_index)
        .map_err(|e| format!("Failed to derive address key: {}", e))?;
    
    // Get public key and create address
    let public_key_bytes = address_key.to_bytes();
    
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
    let purpose = master_key.derive(1852 | 0x80000000);        // Hardened
    let coin_type = purpose.derive(1815 | 0x80000000);         // Hardened  
    let account = coin_type.derive(account_index | 0x80000000); // Hardened
    
    let external_chain = account.derive(0);  // Soft derivation
    
    let address_key = external_chain.derive(address_index);  // Soft derivation
    
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
pub async fn get_addresses_from_wallet(
    wallet_id: String,
    account_index: u32,
    count: u32,
    state: State<'_, AppState>
) -> Result<Vec<AddressInfo>, String> {
    let store = state.wallet_store.lock().unwrap();
    
    // Get wallet metadata with public key
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    let wallet_meta = wallets.iter()
        .find(|w| w.wallet_id == wallet_id)
        .ok_or_else(|| "Wallet not found".to_string())?;
    
    // Create account-level public key from stored bytes (already at m/1852'/1815'/0')
    let mut key_bytes = [0u8; 64];
    if wallet_meta.master_public_key.len() != 64 {
        return Err("Invalid public key length".to_string());
    }
    key_bytes.copy_from_slice(&wallet_meta.master_public_key);
    
    let account_public_key = Bip32PublicKey::from_extended_bytes(&key_bytes)
        .map_err(|e| format!("Failed to create account public key: {}", e))?;
    
    // Derive external chain: m/1852'/1815'/0'/0 (soft derivation)
    let external_chain = account_public_key.derive(0)
        .map_err(|e| format!("Failed to derive external chain: {}", e))?;
    
    let mut addresses = Vec::new();
    
    // Derive each address
    for i in 0..count {
        let address_key = external_chain.derive(i)
            .map_err(|e| format!("Failed to derive address key {}: {}", i, e))?;
        
        let public_key_bytes = address_key.to_bytes();
        let pallas_pubkey = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| format!("Failed to create pallas public key: {}", e))?;
        let address = Address::new(
            &pallas_pubkey,
            None,
            &pallas_addresses::Network::Mainnet,
        ).map_err(|e| format!("Failed to create address: {}", e))?;
        //let address = format!("addr1{}", hex::encode(&public_key_bytes[..16]));
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
    
    // Derive to account level: m/1852'/1815'/account' (hardened derivation)
    let purpose = master_key.derive(1852 | 0x80000000);        // Hardened
    let coin_type = purpose.derive(1815 | 0x80000000);         // Hardened  
    let account = coin_type.derive(account_index | 0x80000000); // Hardened
    
    let external_chain = account.derive(0);  // Soft derivation
    
    // Derive each address
    for i in 0..count {
        let address_key = external_chain.derive(i);  // Returns Self directly, not Result
        
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