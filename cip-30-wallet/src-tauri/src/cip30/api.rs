use pallas_primitives::conway::VKeyWitness;
use pallas_primitives::NonEmptySet;
use serde_json::{json, Value};
use super::types::*;
use super::signing::get_private_key_for_signing;
use crate::commands::{AppState, AddressInfo};
use crate::wallet::{derive_addresses_from_wallet, get_change_address_from_wallet, get_reward_addresses_from_wallet};
use std::sync::Mutex;
use tauri::State;

// Macro to log API messages to both stderr and file
macro_rules! log_api {
    ($($arg:tt)*) => {
        {
            let message = format!($($arg)*);
            eprintln!("{}", message);
            
            // Also write to log file for native messaging mode
            let log_msg = format!(
                "{}: {}\n",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                message
            );
            
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/thresh.log")
                .and_then(|mut f| std::io::Write::write_all(&mut f, log_msg.as_bytes()));
        }
    };
}

pub async fn handle_cip30_request(method: &str, params: Value, app_state: Option<&AppState>, app_handle: Option<tauri::AppHandle>) -> Result<Value, String> {
    log_api!("[API] Handling CIP-30 method: {}", method);
    
    let result = match method {
        "getExtensions" => {
            log_api!("[API] getExtensions called");
            get_extensions()
        },
        //DONE
        "getNetworkId" => {
            log_api!("[API] getNetworkId called");
            get_network_id(app_state)
        },
        //DONE
        "getUtxos" => {
            log_api!("[API] getUtxos called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            get_utxos(params, app_state).await
        },
        "getBalance" => {
            log_api!("[API] getBalance called");
            get_balance(app_state).await
        },
        //DONE
        "getUsedAddresses" => {
            log_api!("[API] getUsedAddresses called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            get_used_addresses(app_state).await
        },
        //DONE
        "getUnusedAddresses" => {
            log_api!("[API] getUnusedAddresses called");
            get_unused_addresses(app_state).await
        },
        //DONE
        "getChangeAddress" => {
            log_api!("[API] getChangeAddress called");
            get_change_address(app_state)
        },
        //DONE
        "getRewardAddresses" => {
            log_api!("[API] getRewardAddresses called");
            get_reward_addresses(app_state)
        },
        "signTx" => {
            log_api!("[API] signTx called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            sign_tx(params, app_state, app_handle).await
        },
        "signData" => {
            log_api!("[API] signData called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            sign_data(params)
        },
        "submitTx" => {
            log_api!("[API] submitTx called with params: {}", serde_json::to_string(&params).unwrap_or_else(|_| "Invalid JSON".to_string()));
            submit_tx(params, app_state).await
        },
        _ => {
            log_api!("[API] Unknown method: {}", method);
            Err(format!("Unknown method: {}", method))
        },
    };
    
    match &result {
        Ok(data) => log_api!("[API] Method {} succeeded with result: {}", method, serde_json::to_string(data).unwrap_or_else(|_| "Invalid JSON".to_string())),
        Err(error) => log_api!("[API] Method {} failed with error: {}", method, error),
    }
    
    result
}

fn get_extensions() -> Result<Value, String> {
    Ok(json!([]))
}

fn get_network_id(app_state: Option<&AppState>) -> Result<Value, String> {
    log_api!("[API] getNetworkId called");
    
    let app_state = match app_state {
        Some(state) => state,
        None => {
            log_api!("[API] ERROR: App state not available");
            return Err("App state not available".to_string());
        }
    };
    
    // Force reload of runtime network to pick up UI changes
    log_api!("[API] Loading runtime network...");
    let runtime_network = load_runtime_network_direct();
    log_api!("[API] Runtime network result: {:?}", runtime_network);
    
    // Use runtime override if available, otherwise fall back to config
    let network = runtime_network.unwrap_or_else(|| {
        log_api!("[API] No runtime network, using config default");
        app_state.config.get_network()
    });
    
    log_api!("[API] Final network to use: {:?}", network);
    
    // CIP-30 specification: 1 = mainnet, 0 = testnet (return just the number)
    let network_id = match network {
        pallas_addresses::Network::Mainnet => {
            log_api!("[API] Network is Mainnet, returning 1");
            1
        },
        pallas_addresses::Network::Testnet => {
            log_api!("[API] Network is Testnet, returning 0");
            0
        },
        pallas_addresses::Network::Other(n) => {
            log_api!("[API] Network is Other({}), returning 0", n);
            0
        }
    };
    
    log_api!("[API] About to return network ID: {}", network_id);
    let result = Ok(json!(network_id));
    log_api!("[API] Final result: {:?}", result);
    result
}

// Helper function to directly load runtime network (for CIP-30 API)
fn load_runtime_network_direct() -> Option<pallas_addresses::Network> {
    let data_dir = dirs::data_dir()?;
    let path = data_dir.join("thresh-wallet").join("runtime_network.txt");
    
    log_api!("[API] Checking runtime network file at: {:?}", path);
    
    if !path.exists() {
        log_api!("[API] No runtime network file found at {:?}", path);
        return None;
    }
    
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            log_api!("[API] Failed to read runtime network file: {}", e);
            return None;
        }
    };
    
    let network_name = content.trim();
    log_api!("[API] Raw file content: '{}'", content);
    log_api!("[API] Trimmed network name: '{}'", network_name);
    log_api!("[API] Network name length: {}", network_name.len());
    
    let result = match network_name.to_lowercase().as_str() {
        "mainnet" => {
            log_api!("[API] Matched mainnet");
            Some(pallas_addresses::Network::Mainnet)
        },
        "testnet" => {
            log_api!("[API] Matched testnet");
            Some(pallas_addresses::Network::Testnet)
        },
        other => {
            log_api!("[API] Unknown network: '{}'", other);
            None
        }
    };
    
    log_api!("[API] Returning network: {:?}", result);
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
                log_api!("[API] Added payment address: {} (hex: {})", addr_info.address, hex_addresses.last().unwrap());
            },
            Err(e) => {
                log_api!("[API] Failed to convert address to hex: {} - {}", addr_info.address, e);
            }
        }
    }
    
    if hex_addresses.is_empty() {
        return Err("No valid hex addresses for UTxO query".to_string());
    }
    
    log_api!("[API] Using {} payment addresses for UTxO query", hex_addresses.len());
    
    // Try to use UTxO RPC client if available
    if let Some(ref utxorpc_config) = app_state.config.utxorpc {
        log_api!("[API] Creating fresh UTxO RPC client for request");
        match crate::utxorpc::UtxoRpcClient::new(utxorpc_config.clone()).await {
            Ok(mut fresh_client) => {
                log_api!("[API] Fresh UTxO RPC client created, fetching UTxOs from {} addresses", hex_addresses.len());
                match fresh_client.fetch_utxos_by_exact_addresses(&hex_addresses, network, Some(50)).await {
                    Ok(utxos) => {
                        log_api!("[API] Successfully fetched UTxOs via exact addresses UTxO RPC");
                        return Ok(utxos);
                    },
                    Err(e) => {
                        log_api!("[API] UTxO RPC by exact addresses failed: {}, falling back to mock data", e);
                    }
                }
            },
            Err(e) => {
                log_api!("[API] Failed to create fresh UTxO RPC client: {}", e);
            }
        }
    } else {
        log_api!("[API] UTxO RPC not configured, using mock data");
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
                log_api!("[API] Added address for balance: {}", addr_info.address);
            },
            Err(e) => {
                log_api!("[API] Failed to convert address to hex for balance: {} - {}", addr_info.address, e);
            }
        }
    }
    
    if hex_addresses.is_empty() {
        return Err("No valid hex addresses for balance calculation".to_string());
    }
    
    log_api!("[API] Calculating balance from {} payment addresses", hex_addresses.len());
    
    // Try to use UTxO RPC client if available
    if let Some(ref utxorpc_config) = app_state.config.utxorpc {
        log_api!("[API] Creating fresh UTxO RPC client for balance calculation");
        match crate::utxorpc::UtxoRpcClient::new(utxorpc_config.clone()).await {
            Ok(mut fresh_client) => {
                log_api!("[API] Fresh UTxO RPC client created, calculating balance");
                match fresh_client.fetch_balance_from_addresses(&hex_addresses, network, Some(50)).await {
                    Ok(balance_value) => {
                        log_api!("[API] Successfully calculated balance via UTxO RPC");
                        
                        // CBOR encode the balance Value
                        let cbor_bytes = pallas_codec::minicbor::to_vec(&balance_value)
                            .map_err(|e| format!("Failed to CBOR encode balance: {}", e))?;
                        let cbor_hex = hex::encode(cbor_bytes);
                        
                        log_api!("[API] Balance CBOR hex: {}", cbor_hex);
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
    
    log_api!("[API] get_used_addresses using network: {:?}", network);
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
    
    log_api!("[API] get_unused_addresses using network: {:?}", network);
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
    
    log_api!("[API] get_change_addresses using network: {:?}", network);
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
    log_api!("[API] sign_tx called with params: {}", params);
    
    // Check that we have app state - we can't sign without it
    let app_state = app_state.ok_or("App state not available - cannot sign transactions in this context")?;
    let app_handle = app_handle.ok_or("App handle not available - cannot show UI for signing")?;
    
    // Extract transaction hex
    let tx_hex = params.get("tx")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid tx parameter")?;
    let _is_partial = params.get("partialSign")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    // Decode and validate the transaction
    let tx_bytes = hex::decode(tx_hex)
        .map_err(|e| format!("Failed to decode tx hex: {}", e))?;
    let mut pallas_tx = pallas_codec::minicbor::decode::<pallas_primitives::conway::Tx>(&tx_bytes)
        .map_err(|e| format!("Failed to decode tx CBOR: {}", e))?;
    let tx_body_cbor = pallas_codec::minicbor::to_vec(&pallas_tx.transaction_body)
        .map_err(|e| format!("Failed to re-encode tx body to CBOR: {}", e))?;
    let tx_hash = pallas_crypto::hash::Hasher::<256>::hash(&tx_body_cbor);
        log_api!("[API] Transaction hash: {}", hex::encode(tx_hash)); 

    log_api!("[API] Decoded transaction: {:?}", pallas_tx);    
    // Show window and get private key
    log_api!("[API] Showing password dialog for transaction signing");
    let signing_key = get_private_key_for_signing(tx_hex, app_state, &app_handle).await?;
    log_api!("[API] Obtained private key for signing");
    let private_key = pallas_crypto::key::ed25519::SecretKey::from(signing_key);
    let public_key = private_key.public_key();
    let signature: [u8; pallas_crypto::key::ed25519::Signature::SIZE] = private_key
        .sign(&tx_hash)
        .as_ref()
        .try_into()
        .unwrap();

    let mut vkey_witnesses = pallas_tx
        .transaction_witness_set
        .vkeywitness
        .as_ref()
        .map(|x| x.clone().to_vec())
        .unwrap_or_default();

    vkey_witnesses.push(VKeyWitness {
        vkey: Vec::from(public_key.as_ref()).into(),
        signature: Vec::from(signature.as_ref()).into(),
    });

    pallas_tx.transaction_witness_set.vkeywitness =
                    Some(NonEmptySet::from_vec(vkey_witnesses).unwrap());

    // According to CIP-30, signTx should return only the TransactionWitnessSet, not the full transaction
    let witness_set_cbor = pallas_codec::minicbor::to_vec(&pallas_tx.transaction_witness_set)
        .map_err(|e| format!("Failed to encode witness set to CBOR: {}", e))?;
    let witness_set_hex = hex::encode(witness_set_cbor);
    log_api!("[API] Transaction witness set hex: {}", witness_set_hex);
    
    // For debugging, also log what the full signed transaction would be
    let signed_tx_cbor = pallas_codec::minicbor::to_vec(&pallas_tx)
        .map_err(|e| format!("Failed to encode signed transaction to CBOR: {}", e))?;
    let signed_tx_hex = hex::encode(signed_tx_cbor);
    log_api!("[API] Full signed transaction hex (for debugging): {}", signed_tx_hex);
    
    // Return only the witness set as per CIP-30 specification
    Ok(json!(witness_set_hex))
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

async fn submit_tx(params: Value, app_state: Option<&AppState>) -> Result<Value, String> {
    let app_state = app_state.ok_or("App state not available - cannot submit transactions")?;
    
    // Extract transaction hex
    let tx_cbor_hex = params.get("tx")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid tx parameter")?;

    log_api!("[API] Submitting transaction with length: {} characters", tx_cbor_hex.len());

    // Get current network
    let runtime_network = load_runtime_network_direct();
    let network = runtime_network.unwrap_or_else(|| app_state.config.get_network());
    
    // Check if UTxO RPC is configured
    let utxorpc_config = app_state.config.utxorpc.as_ref()
        .ok_or("UTxO RPC not configured - cannot submit transactions")?;

    log_api!("[API] Creating UTxO RPC client for transaction submission");
    
    // Create fresh client for submission
    match crate::utxorpc::UtxoRpcClient::new(utxorpc_config.clone()).await {
        Ok(mut client) => {
            log_api!("[API] UTxO RPC client created, submitting transaction...");
            
            match client.submit_transaction(tx_cbor_hex, network).await {
                Ok(tx_hash) => {
                    log_api!("[API] Transaction submitted successfully with hash: {}", tx_hash);
                    Ok(json!(tx_hash))
                }
                Err(e) => {
                    log_api!("[API] Transaction submission failed: {}", e);
                    Err(format!("Transaction submission failed: {}", e))
                }
            }
        }
        Err(e) => {
            log_api!("[API] Failed to create UTxO RPC client: {}", e);
            Err(format!("Failed to create UTxO RPC client: {}", e))
        }
    }
}

