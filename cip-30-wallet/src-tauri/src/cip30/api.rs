use serde_json::{json, Value};
use super::types::*;
use super::signing::get_private_key_for_signing;
use crate::commands::{AppState, AddressInfo};
use crate::wallet::{derive_addresses_from_wallet, get_change_address_from_wallet, get_reward_addresses_from_wallet};
use std::sync::Mutex;
use tauri::State;

pub async fn handle_cip30_request(method: &str, params: Value, app_state: Option<&AppState>, app_handle: Option<tauri::AppHandle>) -> Result<Value, String> {
    eprintln!("[API] Handling CIP-30 method: {}", method);
    
    let result = match method {
        "getExtensions" => {
            eprintln!("[API] getExtensions called");
            get_extensions()
        },
        //DONE
        "getNetworkId" => {
            eprintln!("[API] getNetworkId called");
            get_network_id(app_state)
        },
        //DONE
        "getUtxos" => {
            eprintln!("[API] getUtxos called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            get_utxos(params, app_state).await
        },
        "getBalance" => {
            eprintln!("[API] getBalance called");
            get_balance(app_state).await
        },
        //DONE
        "getUsedAddresses" => {
            eprintln!("[API] getUsedAddresses called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            get_used_addresses(app_state).await
        },
        //DONE
        "getUnusedAddresses" => {
            eprintln!("[API] getUnusedAddresses called");
            get_unused_addresses(app_state).await
        },
        //DONE
        "getChangeAddress" => {
            eprintln!("[API] getChangeAddress called");
            get_change_address(app_state)
        },
        //DONE
        "getRewardAddresses" => {
            eprintln!("[API] getRewardAddresses called");
            get_reward_addresses(app_state)
        },
        "signTx" => {
            eprintln!("[API] signTx called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            sign_tx(params, app_state, app_handle).await
        },
        "signData" => {
            eprintln!("[API] signData called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            sign_data(params)
        },
        "submitTx" => {
            eprintln!("[API] submitTx called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            submit_tx(params)
        },
        _ => {
            eprintln!("[API] Unknown method: {}", method);
            Err(format!("Unknown method: {}", method))
        },
    };
    
    match &result {
        Ok(data) => eprintln!("[API] Method {} succeeded with result: {}", method, serde_json::to_string(data).unwrap_or_else(|_| "Invalid JSON".to_string())),
        Err(error) => eprintln!("[API] Method {} failed with error: {}", method, error),
    }
    
    result
}

fn get_extensions() -> Result<Value, String> {
    Ok(json!([]))
}

fn get_network_id(app_state: Option<&AppState>) -> Result<Value, String> {
    eprintln!("[API] getNetworkId called");
    
    let app_state = match app_state {
        Some(state) => state,
        None => {
            eprintln!("[API] ERROR: App state not available");
            return Err("App state not available".to_string());
        }
    };
    
    // Force reload of runtime network to pick up UI changes
    eprintln!("[API] Loading runtime network...");
    let runtime_network = load_runtime_network_direct();
    eprintln!("[API] Runtime network result: {:?}", runtime_network);
    
    // Use runtime override if available, otherwise fall back to config
    let network = runtime_network.unwrap_or_else(|| {
        eprintln!("[API] No runtime network, using config default");
        app_state.config.get_network()
    });
    
    eprintln!("[API] Final network to use: {:?}", network);
    
    // CIP-30 specification: 1 = mainnet, 0 = testnet (return just the number)
    let network_id = match network {
        pallas_addresses::Network::Mainnet => {
            eprintln!("[API] Network is Mainnet, returning 1");
            1
        },
        pallas_addresses::Network::Testnet => {
            eprintln!("[API] Network is Testnet, returning 0");
            0
        },
        pallas_addresses::Network::Other(n) => {
            eprintln!("[API] Network is Other({}), returning 0", n);
            0
        }
    };
    
    eprintln!("[API] About to return network ID: {}", network_id);
    let result = Ok(json!(network_id));
    eprintln!("[API] Final result: {:?}", result);
    result
}

// Helper function to directly load runtime network (for CIP-30 API)
fn load_runtime_network_direct() -> Option<pallas_addresses::Network> {
    let data_dir = dirs::data_dir()?;
    let path = data_dir.join("cip-30-wallet").join("runtime_network.txt");
    
    eprintln!("[API] Checking runtime network file at: {:?}", path);
    
    if !path.exists() {
        eprintln!("[API] No runtime network file found at {:?}", path);
        return None;
    }
    
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("[API] Failed to read runtime network file: {}", e);
            return None;
        }
    };
    
    let network_name = content.trim();
    eprintln!("[API] Raw file content: '{}'", content);
    eprintln!("[API] Trimmed network name: '{}'", network_name);
    eprintln!("[API] Network name length: {}", network_name.len());
    
    let result = match network_name.to_lowercase().as_str() {
        "mainnet" => {
            eprintln!("[API] Matched mainnet");
            Some(pallas_addresses::Network::Mainnet)
        },
        "testnet" => {
            eprintln!("[API] Matched testnet");
            Some(pallas_addresses::Network::Testnet)
        },
        other => {
            eprintln!("[API] Unknown network: '{}'", other);
            None
        }
    };
    
    eprintln!("[API] Returning network: {:?}", result);
    result
}

async fn get_utxos(params: Value, app_state: Option<&AppState>) -> Result<Value, String> {
    let app_state = app_state.ok_or("App state not available")?;
    
    // Get the first wallet's stake address
    let store = app_state.wallet_store.lock().unwrap();
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    if wallets.is_empty() {
        return Err("No wallets available".to_string());
    }
    
    let wallet_id = &wallets[0].wallet_id;

    // Get current network
    let runtime_network = load_runtime_network_direct();
    let network = runtime_network.unwrap_or_else(|| app_state.config.get_network());
    
    // Get payment addresses from wallet (these hold UTxOs, not stake addresses)
    let payment_addresses = derive_addresses_from_wallet(&store, wallet_id, 0, 10, network)?;
    drop(store); // Release the lock
    
    if payment_addresses.is_empty() {
        return Err("No payment addresses found for wallet".to_string());
    }
    
    // Convert payment addresses to hex format for UTxO RPC
    let mut hex_addresses = Vec::new();
    for addr_info in &payment_addresses {
        match pallas_addresses::Address::from_bech32(&addr_info.address) {
            Ok(addr) => {
                let hex_addr = addr.to_hex();
                hex_addresses.push(hex_addr);
                eprintln!("[API] Added payment address: {} (hex: {})", addr_info.address, hex_addresses.last().unwrap());
            },
            Err(e) => {
                eprintln!("[API] Failed to convert address to hex: {} - {}", addr_info.address, e);
            }
        }
    }
    
    if hex_addresses.is_empty() {
        return Err("No valid hex addresses for UTxO query".to_string());
    }
    
    eprintln!("[API] Using {} payment addresses for UTxO query", hex_addresses.len());
    
    // Try to use UTxO RPC client if available
    if let Some(ref utxorpc_config) = app_state.config.utxorpc {
        eprintln!("[API] Creating fresh UTxO RPC client for request");
        match crate::utxorpc::UtxoRpcClient::new(utxorpc_config.clone()).await {
            Ok(mut fresh_client) => {
                eprintln!("[API] Fresh UTxO RPC client created, fetching UTxOs from {} addresses", hex_addresses.len());
                match fresh_client.fetch_utxos_by_exact_addresses(&hex_addresses, network, Some(50)).await {
                    Ok(utxos) => {
                        eprintln!("[API] Successfully fetched UTxOs via exact addresses UTxO RPC");
                        return Ok(utxos);
                    },
                    Err(e) => {
                        eprintln!("[API] UTxO RPC by exact addresses failed: {}, falling back to mock data", e);
                    }
                }
            },
            Err(e) => {
                eprintln!("[API] Failed to create fresh UTxO RPC client: {}", e);
            }
        }
    } else {
        eprintln!("[API] UTxO RPC not configured, using mock data");
    }
    
    
    Ok(json!(Vec::<String>::new()))
}

async fn get_balance(app_state: Option<&AppState>) -> Result<Value, String> {
    let app_state = app_state.ok_or("App state not available")?;
    
    // Get the first wallet (in a real implementation, this should be the connected wallet)
    let store = app_state.wallet_store.lock().unwrap();
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    if wallets.is_empty() {
        return Err("No wallets available".to_string());
    }
    
    let wallet_id = &wallets[0].wallet_id;

    // Get current network
    let runtime_network = load_runtime_network_direct();
    let network = runtime_network.unwrap_or_else(|| app_state.config.get_network());
    
    // Get payment addresses from wallet (these hold UTxOs)
    let payment_addresses = derive_addresses_from_wallet(&store, wallet_id, 0, 10, network)?;
    drop(store); // Release the lock
    
    if payment_addresses.is_empty() {
        return Err("No payment addresses found for wallet".to_string());
    }
    
    // Convert payment addresses to hex format for UTxO RPC
    let mut hex_addresses = Vec::new();
    for addr_info in &payment_addresses {
        match pallas_addresses::Address::from_bech32(&addr_info.address) {
            Ok(addr) => {
                let hex_addr = addr.to_hex();
                hex_addresses.push(hex_addr);
                eprintln!("[API] Added address for balance: {}", addr_info.address);
            },
            Err(e) => {
                eprintln!("[API] Failed to convert address to hex for balance: {} - {}", addr_info.address, e);
            }
        }
    }
    
    if hex_addresses.is_empty() {
        return Err("No valid hex addresses for balance calculation".to_string());
    }
    
    eprintln!("[API] Calculating balance from {} payment addresses", hex_addresses.len());
    
    // Try to use UTxO RPC client if available
    if let Some(ref utxorpc_config) = app_state.config.utxorpc {
        eprintln!("[API] Creating fresh UTxO RPC client for balance calculation");
        match crate::utxorpc::UtxoRpcClient::new(utxorpc_config.clone()).await {
            Ok(mut fresh_client) => {
                eprintln!("[API] Fresh UTxO RPC client created, calculating balance");
                match fresh_client.fetch_balance_from_addresses(&hex_addresses, network, Some(50)).await {
                    Ok(balance_value) => {
                        eprintln!("[API] Successfully calculated balance via UTxO RPC");
                        
                        // CBOR encode the balance Value
                        let cbor_bytes = pallas_codec::minicbor::to_vec(&balance_value)
                            .map_err(|e| format!("Failed to CBOR encode balance: {}", e))?;
                        let cbor_hex = hex::encode(cbor_bytes);
                        
                        eprintln!("[API] Balance CBOR hex: {}", cbor_hex);
                        return Ok(json!(cbor_hex));
                    },
                    Err(e) => {
                        return Err(format!("UTxO RPC balance calculation failed: {}", e));
                    }
                }
            },
            Err(e) => {
                return Err(format!("Failed to create UTxO RPC client for balance: {}", e));
            }
        }
    } else {
        return Err("UTxO RPC not configured - cannot calculate real balance".to_string());
    }
}

async fn get_used_addresses(app_state: Option<&AppState>) -> Result<Value, String> {
    let app_state = app_state.ok_or("App state not available")?;
    
    // For now, get the first wallet (in a real implementation, this should be the connected wallet)
    let store = app_state.wallet_store.lock().unwrap();
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    if wallets.is_empty() {
        return Err("No wallets available".to_string());
    }
    
    let wallet_id = &wallets[0].wallet_id;
    
    // Force reload of runtime network to pick up UI changes
    let runtime_network = load_runtime_network_direct();
    let network = runtime_network.unwrap_or_else(|| app_state.config.get_network());
    
    eprintln!("[API] get_used_addresses using network: {:?}", network);
    let addresses = derive_addresses_from_wallet(&store, wallet_id, 0, 5, network)?;
    
    // Convert addresses to hex format for CIP-30
    let hex_addresses: Vec<String> = addresses
        .iter()
        .map(|addr_info| {
            let addr = pallas_addresses::Address::from_bech32(&addr_info.address)
                .expect("Invalid address format");
            addr.to_hex()
        })
        .collect();
    
    Ok(json!(hex_addresses))
}

async fn get_unused_addresses(app_state: Option<&AppState>) -> Result<Value, String> {
    let app_state = app_state.ok_or("App state not available")?;
    
    // For now, get the first wallet (in a real implementation, this should be the connected wallet)
    let store = app_state.wallet_store.lock().unwrap();
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    if wallets.is_empty() {
        return Err("No wallets available".to_string());
    }
    
    let wallet_id = &wallets[0].wallet_id;
    
    // Force reload of runtime network to pick up UI changes
    let runtime_network = load_runtime_network_direct();
    let network = runtime_network.unwrap_or_else(|| app_state.config.get_network());
    
    eprintln!("[API] get_unused_addresses using network: {:?}", network);
    let addresses = derive_addresses_from_wallet(&store, wallet_id, 0, 5, network)?;
    
    // Convert addresses to hex format for CIP-30
    let hex_addresses: Vec<String> = addresses
        .iter()
        .map(|addr_info| {
            let addr = pallas_addresses::Address::from_bech32(&addr_info.address)
                .expect("Invalid address format");
            addr.to_hex()
        })
        .collect();
    
    Ok(json!(hex_addresses))
}

fn get_change_address(app_state: Option<&AppState>) -> Result<Value, String> {
    let app_state = app_state.ok_or("App state not available")?;
    
    // For now, get the first wallet (in a real implementation, this should be the connected wallet)
    let store = app_state.wallet_store.lock().unwrap();
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    if wallets.is_empty() {
        return Err("No wallets available".to_string());
    }
    
    let wallet_id = &wallets[0].wallet_id;
    
    // Force reload of runtime network to pick up UI changes
    let runtime_network = load_runtime_network_direct();
    let network = runtime_network.unwrap_or_else(|| app_state.config.get_network());
    
    eprintln!("[API] get_change_addresses using network: {:?}", network);
    let addr_info = get_change_address_from_wallet(&store, wallet_id, 0, network)?;
    
    // Convert addresses to hex format for CIP-30
    let address = pallas_addresses::Address::from_bech32(&addr_info.address)
                .expect("Invalid address format");
    Ok(json!(address.to_hex()))
}

fn get_reward_addresses(app_state: Option<&AppState>) -> Result<Value, String> {
   let app_state = app_state.ok_or("App state not available")?;
    
    // For now, get the first wallet (in a real implementation, this should be the connected wallet)
    let store = app_state.wallet_store.lock().unwrap();
    let wallets = store.list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;
    
    if wallets.is_empty() {
        return Err("No wallets available".to_string());
    }
    
    let wallet_id = &wallets[0].wallet_id;

    let runtime_network = load_runtime_network_direct();
    let network = runtime_network.unwrap_or_else(|| app_state.config.get_network());
    
    
    let addresses: Vec<String> = get_reward_addresses_from_wallet(&store, wallet_id, 0, network)?;
    
    Ok(json!(addresses))
}

async fn sign_tx(params: Value, app_state: Option<&AppState>, app_handle: Option<tauri::AppHandle>) -> Result<Value, String> {
    eprintln!("[API] sign_tx called with params: {}", params);
    
    // Check that we have app state - we can't sign without it
    let app_state = app_state.ok_or("App state not available - cannot sign transactions in this context")?;
    let app_handle = app_handle.ok_or("App handle not available - cannot show UI for signing")?;
    
    // Extract transaction hex
    let tx_hex = params.get("tx")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid tx parameter")?;
    
    // Decode and validate the transaction
    let tx_bytes = hex::decode(tx_hex)
        .map_err(|e| format!("Failed to decode tx hex: {}", e))?;
    let _pallas_tx = pallas_codec::minicbor::decode::<pallas_primitives::conway::Tx>(&tx_bytes)
        .map_err(|e| format!("Failed to decode tx CBOR: {}", e))?;
    
    // Show window and get private key
    eprintln!("[API] Showing password dialog for transaction signing");
    let _private_key = get_private_key_for_signing(tx_hex, app_state, &app_handle).await?;
    eprintln!("[API] Obtained private key for signing");
    
    // TODO: Use the private_key to actually sign the transaction
    // For now, return a mock witness set
    let mock_vkey = "8200582c82008202828200581c4d7b4fae42073d8175d1daa7ed8b7f46055bc8b48bbec1104110b13e";
    let mock_signature = "845846a201025820aceecdf2f7e5a0a91a6a0e06cb051d87e8827fd08cb44666c39948e2b2e3f79a5840d548cb87170df840754dc81dd5ae73f8bbb770a5826a45bb90b3186d1d8f5fb8a73ad13be8bb0a85bbf9dd1b195b9c96fc1914b3bf50e46cf473d33b730b50c";
    
    let witness_set = format!("a10081825820{}5840{}", mock_vkey, mock_signature);
    
    Ok(json!({
        "witness": witness_set
    }))
}

fn sign_data(params: Value) -> Result<Value, String> {
    // Mock data signature
    let addr = params.get("addr").ok_or("Missing addr parameter")?;
    let payload = params.get("payload").ok_or("Missing payload parameter")?;
    
    Ok(json!({
        "signature": {
            "signature": "845846a2012767616464726573735839012fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3xa166686173686564f458401234567890abcdef",
            "key": format!("{}{}", "a4010103272006215820", "8fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfg")
        }
    }))
}

fn submit_tx(params: Value) -> Result<Value, String> {
    // Mock transaction submission
    let tx = params.get("tx").ok_or("Missing tx parameter")?;
    
    // Return mock transaction hash
    Ok(json!({
        "txHash": "9fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x"
    }))
}

