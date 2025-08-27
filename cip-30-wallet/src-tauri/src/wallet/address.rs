use crate::crypto::keys::Bip32PublicKey;
use crate::storage::wallet_store::WalletStore;
use crate::commands::AddressInfo;

/// Derive addresses from a wallet using stored public key
pub fn derive_addresses_from_wallet(
    wallet_store: &WalletStore,
    wallet_id: &str,
    account_index: u32,
    count: u32,
    network: pallas_addresses::Network,
) -> Result<Vec<AddressInfo>, String> {
    // Get wallet metadata with public key
    let wallets = wallet_store.list_wallets()
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
    
    // Derive payment chain: m/1852'/1815'/0'/0 (role 0 = external)
    let payment_chain = account_public_key.derive(0)
        .map_err(|e| format!("Failed to derive payment chain: {}", e))?;
    
    // Derive staking key once (same for all addresses in the account)
    let staking_chain = account_public_key.derive(2)
        .map_err(|e| format!("Failed to derive staking chain: {}", e))?;
    let staking_key = staking_chain.derive(0)
        .map_err(|e| format!("Failed to derive staking key: {}", e))?;
    let staking_pubkey_bytes = staking_key.to_bytes();
    
    let mut addresses = Vec::new();
    
    // Derive each address
    for i in 0..count {
        // Derive payment key for this index
        let payment_key = payment_chain.derive(i)
            .map_err(|e| format!("Failed to derive payment key {}: {}", i, e))?;
        
        let payment_pubkey_bytes = payment_key.to_bytes();
        let payment_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&payment_pubkey_bytes);
        let payment_part = pallas_addresses::ShelleyPaymentPart::Key(payment_pubkey_hash);

        let staking_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&staking_pubkey_bytes);
        let staking_part = pallas_addresses::ShelleyDelegationPart::Key(staking_pubkey_hash);
        
        // Create base address (payment + staking)
        let address = pallas_addresses::ShelleyAddress::new(
            network,
            payment_part,
            staking_part
        );
        
        let address_string = address.to_bech32()
            .map_err(|e| format!("Failed to encode address: {}", e))?;
        let path = format!("m/1852'/1815'/{}'/{}/{}", account_index, 0, i);
        
        addresses.push(AddressInfo {
            address: address_string,
            path,
            account_index,
            address_index: i,
        });
    }
    
    Ok(addresses)
}

pub fn get_change_address_from_wallet(
    wallet_store: &WalletStore,
    wallet_id: &str,
    account_index: u32,
    network: pallas_addresses::Network,
) -> Result<AddressInfo, String> {
    // Get wallet metadata with public key
    let wallets = wallet_store.list_wallets()
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
    
    // Derive payment chain: m/1852'/1815'/0'/1 (role 1 = internal/change)
    let payment_chain = account_public_key.derive(1)
        .map_err(|e| format!("Failed to derive payment chain: {}", e))?;
    
    // Derive staking key once (same for all addresses in the account)
    let staking_chain = account_public_key.derive(2)
        .map_err(|e| format!("Failed to derive staking chain: {}", e))?;
    let staking_key = staking_chain.derive(0)
        .map_err(|e| format!("Failed to derive staking key: {}", e))?;
    let staking_pubkey_bytes = staking_key.to_bytes();
    
    // Derive payment key for index 0 (first change address)
    let payment_key = payment_chain.derive(0)
        .map_err(|e| format!("Failed to derive payment key: {}", e))?;
    
    let payment_pubkey_bytes = payment_key.to_bytes();
    let payment_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&payment_pubkey_bytes);
    let payment_part = pallas_addresses::ShelleyPaymentPart::Key(payment_pubkey_hash);

    let staking_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&staking_pubkey_bytes);
    let staking_part = pallas_addresses::ShelleyDelegationPart::Key(staking_pubkey_hash);

    // Create base address (payment + staking)
    let address = pallas_addresses::ShelleyAddress::new(
        network,
        payment_part,
        staking_part
    );
    let address_string = address.to_bech32()
        .map_err(|e| format!("Failed to encode address: {}", e))?;
    let path = format!("m/1852'/1815'/{}'/1/0", account_index);

    Ok(AddressInfo {
        address: address_string,
        path,
        account_index,
        address_index: 0,
    })
}

pub fn get_reward_addresses_from_wallet(
    wallet_store: &WalletStore,
    wallet_id: &str,
    account_index: u32,
) -> Result<Vec<String>, String> {
    // Get wallet metadata with public key
    let wallets = wallet_store.list_wallets()
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
   
    // Derive staking key once (same for all addresses in the account)
    let staking_chain = account_public_key.derive(2)
        .map_err(|e| format!("Failed to derive staking chain: {}", e))?;
    let staking_key = staking_chain.derive(0)
        .map_err(|e| format!("Failed to derive staking key: {}", e))?;
    let staking_pubkey_bytes = staking_key.to_bytes();
    
    let mut addresses: Vec<String> = Vec::new();
    
    let staking_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&staking_pubkey_bytes);
   // let staking_part = pallas_addresses::StakePayload::Stake(staking_pubkey_hash);
    
    let staking_part = pallas_addresses::ShelleyDelegationPart::Key(staking_pubkey_hash);
    
    //TODO: add more addresses and use account index in path
    addresses.push(staking_part.to_hex());
    
    Ok(addresses)
}
