use crate::config::AppConfig;
use crate::crypto::{
    encryption::WalletWrapper,
    keys::{Bip32PrivateKey, Bip32PublicKey},
    mnemonic::{generate_mnemonic, mnemonic_to_seed, validate_mnemonic},
};
use crate::storage::wallet_store::{WalletMetadata, WalletStore};
use crate::utxorpc::UtxoRpcClient;
use crate::wallet::{derive_addresses_from_wallet, WalletError, WalletResult};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

// Global state for pending password requests
lazy_static! {
    static ref PENDING_PASSWORD_REQUESTS: Mutex<HashMap<String, Sender<Result<String, String>>>> =
        Mutex::new(HashMap::new());
}
/// Application state to hold the wallet store and configuration
pub struct AppState {
    pub wallet_store: Mutex<WalletStore>,
    pub config: AppConfig,
    pub runtime_network: Mutex<Option<pallas_addresses::Network>>,
    pub utxorpc_client: Mutex<Option<UtxoRpcClient>>,
}

impl AppState {
    /// Get the active network - UI override takes precedence over config file
    pub fn get_active_network(&self) -> pallas_addresses::Network {
        self.runtime_network
            .lock()
            .unwrap()
            .unwrap_or(self.config.get_network())
    }
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

    let wallets = store
        .list_wallets()
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

    store
        .list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))
}

#[tauri::command]
pub async fn get_wallet_info(
    wallet_id: String,
    state: State<'_, AppState>,
) -> Result<WalletInfoResponse, String> {
    let store = state.wallet_store.lock().unwrap();

    // Try to load wallet metadata
    let wallets = store
        .list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;

    let wallet_meta = wallets
        .iter()
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

    generate_mnemonic(word_count).map_err(|e| format!("Failed to generate mnemonic: {}", e))
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
    state: State<'_, AppState>,
) -> Result<WalletCreateResponse, String> {
    // Validate mnemonic first
    validate_mnemonic(&request.mnemonic).map_err(|e| format!("Invalid mnemonic: {}", e))?;

    // Generate account-level keys from mnemonic to store public key
    let seed = mnemonic_to_seed(&request.mnemonic, "")
        .map_err(|e| format!("Failed to generate seed: {}", e))?;

    let master_private_key = Bip32PrivateKey::from_bip39_seed(&seed)
        .map_err(|e| format!("Failed to generate master key: {}", e))?;

    // Derive to account level: m/1852'/1815'/0' (hardened derivation)
    let purpose = master_private_key.derive(1852 | 0x80000000); // Hardened
    let coin_type = purpose.derive(1815 | 0x80000000); // Hardened
    let account = coin_type.derive(0 | 0x80000000); // Hardened, account 0

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
    match store.save_wallet(
        &wallet_id,
        &wallet,
        &request.password,
        &account_public_key_bytes,
    ) {
        Ok(()) => eprintln!("Wallet saved successfully"),
        Err(e) => {
            eprintln!("Failed to save wallet: {}", e);
            return Err(format!("Failed to save wallet: {}", e));
        }
    }

    // TODO: Save master public key in extended metadata
    // For now, we'll store it when we enhance the wallet store

    Ok(WalletCreateResponse { wallet_id })
}

// Address Derivation Commands (using public keys only)

#[tauri::command]
pub async fn derive_address_from_wallet(
    wallet_id: String,
    account_index: u32,
    address_index: u32,
    state: State<'_, AppState>,
) -> Result<AddressInfo, String> {
    let store = state.wallet_store.lock().unwrap();

    // Use shared function to derive addresses, then return the specific one
    let network = state.get_active_network();
    let addresses = derive_addresses_from_wallet(
        &store,
        &wallet_id,
        account_index,
        address_index + 1,
        network,
    )?;

    // Return the address at the requested index
    addresses
        .into_iter()
        .find(|addr| addr.address_index == address_index)
        .ok_or_else(|| "Failed to derive address at specified index".to_string())
}

#[tauri::command]
pub async fn get_addresses_from_wallet(
    wallet_id: String,
    account_index: u32,
    count: u32,
    state: State<'_, AppState>,
) -> Result<Vec<AddressInfo>, String> {
    let store = state.wallet_store.lock().unwrap();
    let network = state.get_active_network();
    derive_addresses_from_wallet(&store, &wallet_id, account_index, count, network)
}

// Network Configuration Commands

#[tauri::command]
pub async fn get_current_network(state: State<'_, AppState>) -> Result<String, String> {
    let network = state.get_active_network();
    let network_name = match network {
        pallas_addresses::Network::Mainnet => "mainnet",
        pallas_addresses::Network::Testnet => "testnet",
        pallas_addresses::Network::Other(_) => "testnet",
    };
    Ok(network_name.to_string())
}

#[tauri::command]
pub async fn set_runtime_network(
    network: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pallas_network = match network.to_lowercase().as_str() {
        "mainnet" => pallas_addresses::Network::Mainnet,
        "testnet" => pallas_addresses::Network::Testnet,
        _ => return Err(format!("Invalid network: {}", network)),
    };

    // Set runtime network in memory
    let mut runtime_network = state.runtime_network.lock().unwrap();
    *runtime_network = Some(pallas_network);

    // Also save to a runtime file so native messaging can pick it up
    save_runtime_network(&network)?;

    eprintln!("Runtime network set to: {}", network);
    Ok(())
}

#[tauri::command]
pub async fn save_network_to_config(
    network: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Validate network
    match network.to_lowercase().as_str() {
        "mainnet" | "testnet" => {}
        _ => return Err(format!("Invalid network: {}", network)),
    }

    // Update the config
    let mut updated_config = state.config.clone();
    updated_config.network.name = network.to_lowercase();

    // Save to file
    let config_path = AppConfig::default_config_path()
        .map_err(|e| format!("Failed to get config path: {}", e))?;
    updated_config
        .save(&config_path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    eprintln!("Network saved to config: {}", network);
    Ok(())
}

#[tauri::command]
pub async fn save_utxorpc_to_config(
    mainnet_url: Option<String>,
    testnet_url: Option<String>,
    api_key: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Clone values for logging before moving them
    let mainnet_url_log = mainnet_url.clone();
    let testnet_url_log = testnet_url.clone();
    let has_api_key = api_key.is_some();

    // Update the config
    let mut updated_config = state.config.clone();
    updated_config.utxorpc = Some(crate::config::UtxoRpcConfig {
        mainnet_url,
        testnet_url,
        api_key,
    });

    // Save to file
    let config_path = AppConfig::default_config_path()
        .map_err(|e| format!("Failed to get config path: {}", e))?;
    updated_config
        .save(&config_path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    eprintln!("UTXO RPC configuration saved to config");
    if let Some(ref mainnet_url) = mainnet_url_log {
        eprintln!("  Mainnet URL: {}", mainnet_url);
    }
    if let Some(ref testnet_url) = testnet_url_log {
        eprintln!("  Testnet URL: {}", testnet_url);
    }
    if has_api_key {
        eprintln!("  API Key: [configured]");
    }
    Ok(())
}

#[tauri::command]
pub async fn get_utxorpc_config(
    state: State<'_, AppState>,
) -> Result<Option<crate::config::UtxoRpcConfig>, String> {
    Ok(state.config.utxorpc.clone())
}

#[tauri::command]
pub async fn reset_runtime_network(state: State<'_, AppState>) -> Result<(), String> {
    let mut runtime_network = state.runtime_network.lock().unwrap();
    *runtime_network = None;

    // Remove the runtime file
    clear_runtime_network()?;

    eprintln!("Runtime network reset to config default");
    Ok(())
}

#[tauri::command]
pub async fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.clone())
}

#[tauri::command]
pub async fn get_network_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let active_network = state.get_active_network();
    let config_network = state.config.get_network();
    let runtime_override = state.runtime_network.lock().unwrap().is_some();

    let active_name = match active_network {
        pallas_addresses::Network::Mainnet => "mainnet",
        pallas_addresses::Network::Testnet => "testnet",
        pallas_addresses::Network::Other(_) => "testnet",
    };

    let config_name = match config_network {
        pallas_addresses::Network::Mainnet => "mainnet",
        pallas_addresses::Network::Testnet => "testnet",
        pallas_addresses::Network::Other(_) => "testnet",
    };

    Ok(serde_json::json!({
        "active": active_name,
        "config_default": config_name,
        "has_runtime_override": runtime_override
    }))
}

// Utility Commands

#[tauri::command]
pub async fn validate_wallet_password(
    wallet_id: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let store = state.wallet_store.lock().unwrap();

    match store.load_wallet(&wallet_id, &password) {
        Ok(_) => Ok(true),
        Err(WalletError::InvalidPassword) => Ok(false),
        Err(e) => Err(format!("Error validating password: {}", e)),
    }
}

// Helper functions for runtime network persistence
fn get_runtime_network_path() -> Result<std::path::PathBuf, String> {
    let data_dir = dirs::data_dir().ok_or("Failed to get data directory")?;
    Ok(data_dir.join("thresh-wallet").join("runtime_network.txt"))
}

fn save_runtime_network(network: &str) -> Result<(), String> {
    let path = get_runtime_network_path()?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create runtime config directory: {}", e))?;
    }

    fs::write(&path, network).map_err(|e| format!("Failed to save runtime network: {}", e))?;

    eprintln!("Saved runtime network '{}' to {:?}", network, path);
    Ok(())
}

fn load_runtime_network() -> Option<pallas_addresses::Network> {
    let path = get_runtime_network_path().ok()?;

    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(&path).ok()?;
    let network_name = content.trim();

    eprintln!("Loaded runtime network '{}' from {:?}", network_name, path);

    match network_name.to_lowercase().as_str() {
        "mainnet" => Some(pallas_addresses::Network::Mainnet),
        "testnet" => Some(pallas_addresses::Network::Testnet),
        _ => None,
    }
}

fn clear_runtime_network() -> Result<(), String> {
    let path = get_runtime_network_path()?;

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove runtime network file: {}", e))?;
        eprintln!("Cleared runtime network file at {:?}", path);
    }

    Ok(())
}

// Helper function to initialize app state
pub fn create_app_state() -> Result<AppState, String> {
    // Load configuration
    let config_path = AppConfig::default_config_path()
        .map_err(|e| format!("Failed to get config path: {}", e))?;
    let config = AppConfig::load_or_create(&config_path)
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Load runtime network override if it exists
    let runtime_network = load_runtime_network();

    let data_dir = WalletStore::default_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;

    let wallet_store =
        WalletStore::new(data_dir).map_err(|e| format!("Failed to create wallet store: {}", e))?;

    // Initialize UTxO RPC client if configured
    let utxorpc_client = if let Some(utxorpc_config) = &config.utxorpc {
        eprintln!("[INIT] UTxO RPC config found, initializing client...");
        eprintln!("[INIT] Mainnet URL: {:?}", utxorpc_config.mainnet_url);
        eprintln!("[INIT] Testnet URL: {:?}", utxorpc_config.testnet_url);

        // Use blocking runtime to initialize the async client
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(UtxoRpcClient::new(utxorpc_config.clone())) {
            Ok(client) => {
                eprintln!("[INIT] UTxO RPC client successfully initialized");
                Some(client)
            }
            Err(e) => {
                eprintln!("[INIT] Failed to initialize UTxO RPC client: {}", e);
                eprintln!("[INIT] UTxO RPC functionality will be disabled");
                None
            }
        }
    } else {
        eprintln!("[INIT] No UTxO RPC config found in configuration");
        None
    };

    Ok(AppState {
        wallet_store: Mutex::new(wallet_store),
        config,
        runtime_network: Mutex::new(runtime_network),
        utxorpc_client: Mutex::new(utxorpc_client),
    })
}

// Transaction Signing Commands

#[tauri::command]
pub async fn prompt_transaction_password(
    tx_cbor: String,
    wallet_id: String,
    window: tauri::WebviewWindow,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // First, bring the window to front
    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;
    window
        .unminimize()
        .map_err(|e| format!("Failed to unminimize window: {}", e))?;
    window
        .set_focus()
        .map_err(|e| format!("Failed to focus window: {}", e))?;

    // Create a one-shot channel to receive the password from the frontend
    let (tx, rx) = std::sync::mpsc::channel::<Result<String, String>>();

    // Store the sender in the app state so the frontend can send the password back
    let tx_id = format!(
        "tx_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Emit event to frontend to show password dialog
    app_handle
        .emit(
            "request-password",
            serde_json::json!({
                "txId": tx_id,
                "walletId": wallet_id,
                "txCbor": tx_cbor,
            }),
        )
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    // Store the channel sender temporarily
    {
        let mut pending_requests = PENDING_PASSWORD_REQUESTS.lock().unwrap();
        pending_requests.insert(tx_id.clone(), tx);
    }

    // Wait for password from frontend (with timeout)
    match rx.recv_timeout(std::time::Duration::from_secs(300)) {
        // 5 minute timeout
        Ok(Ok(password)) => Ok(password),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            // Clean up on timeout
            let mut pending_requests = PENDING_PASSWORD_REQUESTS.lock().unwrap();
            pending_requests.remove(&tx_id);
            Err("Password prompt timeout".to_string())
        }
    }
}

#[tauri::command]
pub async fn submit_transaction_password(
    tx_id: String,
    password: Option<String>,
) -> Result<(), String> {
    let mut pending_requests = PENDING_PASSWORD_REQUESTS.lock().unwrap();

    if let Some(sender) = pending_requests.remove(&tx_id) {
        match password {
            Some(pwd) => {
                sender
                    .send(Ok(pwd))
                    .map_err(|_| "Failed to send password".to_string())?;
            }
            None => {
                sender
                    .send(Err("User cancelled".to_string()))
                    .map_err(|_| "Failed to send cancellation".to_string())?;
            }
        }
        Ok(())
    } else {
        Err("Invalid transaction ID".to_string())
    }
}

// Transaction Submission Commands

#[tauri::command]
pub async fn submit_tx_with_state(
    tx_cbor_hex: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    eprintln!("[CMD] submit_tx_with_state called with transaction length: {}", tx_cbor_hex.len());

    // Get current network
    let runtime_network = load_runtime_network();
    let network = runtime_network.unwrap_or_else(|| state.config.get_network());
    
    // Check if UTxO RPC is configured
    let utxorpc_config = state.config.utxorpc.as_ref()
        .ok_or("UTxO RPC not configured - cannot submit transactions")?;

    eprintln!("[CMD] Creating UTxO RPC client for transaction submission");
    
    // Create fresh client for submission
    match crate::utxorpc::UtxoRpcClient::new(utxorpc_config.clone()).await {
        Ok(mut client) => {
            eprintln!("[CMD] UTxO RPC client created, submitting transaction...");
            
            match client.submit_transaction(&tx_cbor_hex, network).await {
                Ok(tx_hash) => {
                    eprintln!("[CMD] Transaction submitted successfully with hash: {}", tx_hash);
                    Ok(tx_hash)
                }
                Err(e) => {
                    eprintln!("[CMD] Transaction submission failed: {}", e);
                    Err(format!("Transaction submission failed: {}", e))
                }
            }
        }
        Err(e) => {
            eprintln!("[CMD] Failed to create UTxO RPC client: {}", e);
            Err(format!("Failed to create UTxO RPC client: {}", e))
        }
    }
}

// Helper function to get signing key from wallet
pub fn get_signing_key_from_wallet(
    wallet_id: &str,
    password: &str,
    state: &AppState,
) -> Result<[u8; 32], String> {
    let store = state.wallet_store.lock().unwrap();

    // Load the wallet with the provided password
    let wallet = store
        .load_wallet(wallet_id, password)
        .map_err(|e| match e {
            WalletError::InvalidPassword => "Invalid password".to_string(),
            _ => format!("Failed to load wallet: {}", e),
        })?;

    // Generate seed from mnemonic
    let seed = mnemonic_to_seed(&wallet.mnemonic, "")
        .map_err(|e| format!("Failed to generate seed: {}", e))?;

    // Derive master private key
    let master_private_key = Bip32PrivateKey::from_bip39_seed(&seed)
        .map_err(|e| format!("Failed to generate master key: {}", e))?;

    //TODO:
    // For now, we'll use the first account's first external address key
    // In a real implementation, you'd determine which key to use based on the transaction
    // Derive to: m/1852'/1815'/0'/0/0
    let signing_key = master_private_key
        .derive(1852 | 0x80000000) // purpose (hardened)
        .derive(1815 | 0x80000000) // coin_type (hardened)
        .derive(0 | 0x80000000) // account (hardened)
        .derive(0) // external chain
        .derive(0) // first address
        .to_signing_key();
    Ok(signing_key)
}
