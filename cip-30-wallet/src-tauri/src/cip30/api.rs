use serde_json::{json, Value};
use super::types::*;
use crate::commands::{AppState, AddressInfo};
use crate::wallet::{derive_addresses_from_wallet, get_change_address_from_wallet};
use std::sync::Mutex;
use tauri::State;

pub async fn handle_cip30_request(method: &str, params: Value, app_state: Option<&AppState>) -> Result<Value, String> {
    eprintln!("[API] Handling CIP-30 method: {}", method);
    
    let result = match method {
        "getExtensions" => {
            eprintln!("[API] getExtensions called");
            get_extensions()
        },
        "getNetworkId" => {
            eprintln!("[API] getNetworkId called");
            get_network_id(app_state)
        },
        "getUtxos" => {
            eprintln!("[API] getUtxos called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            get_utxos(params)
        },
        "getBalance" => {
            eprintln!("[API] getBalance called");
            get_balance()
        },
        "getUsedAddresses" => {
            eprintln!("[API] getUsedAddresses called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            get_used_addresses(app_state).await
        },
        "getUnusedAddresses" => {
            eprintln!("[API] getUnusedAddresses called");
            get_unused_addresses(app_state).await
        },
        "getChangeAddress" => {
            eprintln!("[API] getChangeAddress called");
            get_change_address(app_state)
        },
        "getRewardAddresses" => {
            eprintln!("[API] getRewardAddresses called");
            get_reward_addresses()
        },
        "signTx" => {
            eprintln!("[API] signTx called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            sign_tx(params)
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

fn get_utxos(params: Value) -> Result<Value, String> {
    // Mock UTXOs
    let utxos = vec![
        json!({
            "tx_hash": "5d677265fa5bb21ce6d8c7502aca70b9316d10e958611f3c6b758f65ad959996",
            "tx_index": 0,
            "amount": "1000000", // 1 ADA in lovelace
            "address": "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x"
        }),
        json!({
            "tx_hash": "6d677265fa5bb21ce6d8c7502aca70b9316d10e958611f3c6b758f65ad959996",
            "tx_index": 1,
            "amount": "2000000", // 2 ADA
            "address": "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x"
        })
    ];
    
    // Return CBOR hex (mock - in real implementation, encode properly)
    Ok(json!([
        format!("{}{}{}{}{}", "82825820", "5d677265fa5bb21ce6d8c7502aca70b9316d10e958611f3c6b758f65ad959996", "00825839", "01", "1a000f4240"),
        format!("{}{}{}{}{}", "82825820", "6d677265fa5bb21ce6d8c7502aca70b9316d10e958611f3c6b758f65ad959996", "01825839", "01", "1a001e8480")
    ]))
}

fn get_balance() -> Result<Value, String> {
    // Mock balance: 3 ADA (in lovelace) as CBOR hex
    // In real implementation, properly encode as CBOR
    Ok(json!("1a002dc6c0")) // 3000000 lovelace
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
    
    eprintln!("[API] get_unused_addresses using network: {:?}", network);
    let addr_info = get_change_address_from_wallet(&store, wallet_id, 0, network)?;
    
    // Convert addresses to hex format for CIP-30
    let address = pallas_addresses::Address::from_bech32(&addr_info.address)
                .expect("Invalid address format");
    Ok(json!(address.to_hex()))
}

fn get_reward_addresses() -> Result<Value, String> {
    // Mock reward addresses
    Ok(json!([
        format!("{}{}", "e161", "6fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfg")
    ]))
}

fn sign_tx(params: Value) -> Result<Value, String> {
    // Mock signature - in real implementation, would sign the transaction
    let tx = params.get("tx").ok_or("Missing tx parameter")?;
    
    // Return mock witness set (CBOR hex)
    Ok(json!({
        "witness": format!("{}{}{}{}", "a10081825820", "7fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfg", "5840", "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
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