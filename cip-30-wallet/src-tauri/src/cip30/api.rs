use serde_json::{json, Value};
use super::types::*;

pub async fn handle_cip30_request(method: &str, params: Value) -> Result<Value, String> {
    eprintln!("[API] Handling CIP-30 method: {}", method);
    
    let result = match method {
        "getExtensions" => {
            eprintln!("[API] getExtensions called");
            get_extensions()
        },
        "getNetworkId" => {
            eprintln!("[API] getNetworkId called");
            get_network_id()
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
            get_used_addresses(params)
        },
        "getUnusedAddresses" => {
            eprintln!("[API] getUnusedAddresses called");
            get_unused_addresses()
        },
        "getChangeAddress" => {
            eprintln!("[API] getChangeAddress called");
            get_change_address()
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

fn get_network_id() -> Result<Value, String> {
    // 1 = mainnet, 0 = testnet
    Ok(json!(1))
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

fn get_used_addresses(params: Value) -> Result<Value, String> {
    // Mock used addresses (as CBOR hex)
    Ok(json!([
        format!("{}{}", "0061", "2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x"),
        format!("{}{}", "0061", "3fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x")
    ]))
}

fn get_unused_addresses() -> Result<Value, String> {
    // Mock unused addresses
    Ok(json!([
        format!("{}{}", "0061", "4fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x")
    ]))
}

fn get_change_address() -> Result<Value, String> {
    // Mock change address
    Ok(json!(format!("{}{}", "0061", "5fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x")))
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