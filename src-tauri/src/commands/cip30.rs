use pallas_addresses::Address;
use serde_json::Value;
use tauri;

#[tauri::command]
pub async fn sign_data(data: String) -> String {
    let result = String::from("");
    result
}

#[tauri::command]
pub async fn sign_tx(tx: String) -> String {
    let result = String::from("");
    result
}

#[tauri::command]
pub async fn submit_tx(tx: String) -> String {
    let result = String::from("");
    result
}

#[tauri::command]
pub async fn get_utxos() -> String {
    let result = String::from("");
    result
}

#[tauri::command]
pub async fn get_collateral() -> String {
    let result = String::from("");
    result
}

#[tauri::command]
pub async fn get_used_addresses() -> Vec<String> {
    let address = Address::from_bech32("addr1q8saj9wppjzqq9aa8yyg4qjs0vn32zjr36ysw7zzy9y3xztl9fadz30naflhmq653up3tkz275gh5npdejwjj23l0rdqls5spd").unwrap();
    let addr_hex = address.to_hex();
    let mut used = Vec::new();
    used.push(addr_hex);
    used
}

#[tauri::command]
pub async fn get_unused_addresses() -> Vec<String> {
    let address = Address::from_bech32("addr1q8saj9wppjzqq9aa8yyg4qjs0vn32zjr36ysw7zzy9y3xztl9fadz30naflhmq653up3tkz275gh5npdejwjj23l0rdqls5spd").unwrap();
    let addr_hex = address.to_hex();
    let mut unused = Vec::new();
    unused.push(addr_hex);
    unused
}

#[tauri::command]
pub async fn get_change_address() -> String {
    let address = Address::from_bech32("addr1q8saj9wppjzqq9aa8yyg4qjs0vn32zjr36ysw7zzy9y3xztl9fadz30naflhmq653up3tkz275gh5npdejwjj23l0rdqls5spd").unwrap();
    let addr_hex = address.to_hex();
    println!("Change Address: {}", addr_hex);
    addr_hex
}

#[tauri::command]
pub async fn get_reward_address() -> String {
    let result = String::from("");
    result
}

#[tauri::command]
pub async fn get_network_id() -> i32 {
    let result = 1;
    result
}

#[tauri::command]
pub async fn get_balance() -> String {
    let result = String::from("100");
    result
}
