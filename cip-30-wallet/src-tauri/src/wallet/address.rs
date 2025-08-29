use crate::commands::AddressInfo;
use crate::crypto::keys::Bip32PublicKey;
use crate::storage::wallet_store::WalletStore;

/// Derive addresses from a wallet using stored public key
pub fn derive_addresses_from_wallet(
    wallet_store: &WalletStore,
    wallet_id: &str,
    account_index: u32,
    count: u32,
    network: pallas_addresses::Network,
) -> Result<Vec<AddressInfo>, String> {
    // Get wallet metadata with public key
    let wallets = wallet_store
        .list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;

    let wallet_meta = wallets
        .iter()
        .find(|w| w.wallet_id == wallet_id)
        .ok_or_else(|| "Wallet not found".to_string())?;

    // Use stored Ed25519 public keys (64 bytes: 32 payment + 32 staking)
    if wallet_meta.master_public_key.len() != 64 {
        return Err(format!("Invalid Ed25519 public keys length: expected 64 bytes, got {}", wallet_meta.master_public_key.len()));
    }
    
    // Extract payment Ed25519 public key (first 32 bytes)
    let mut payment_ed25519_key = [0u8; 32];
    payment_ed25519_key.copy_from_slice(&wallet_meta.master_public_key[..32]);
    
    // Extract staking Ed25519 public key (second 32 bytes)
    let mut staking_ed25519_key = [0u8; 32];
    staking_ed25519_key.copy_from_slice(&wallet_meta.master_public_key[32..]);

    let mut addresses = Vec::new();

    // Generate addresses using stored Ed25519 public keys
    // Note: With the current Ed25519 key storage, we store only the first address keys
    // For multiple addresses, we would need to store or derive additional keys
    // For now, return the same address for all requested indices (typical for simple wallets)
    for i in 0..count {        
        // Use the stored Ed25519 payment public key (represents the first/main address)
        let payment_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&payment_ed25519_key);
        let payment_part = pallas_addresses::ShelleyPaymentPart::Key(payment_pubkey_hash);

        // Use the stored Ed25519 staking public key
        let staking_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&staking_ed25519_key);
        let staking_part = pallas_addresses::ShelleyDelegationPart::Key(staking_pubkey_hash);

        // Create base address (payment + staking)
        let address = pallas_addresses::ShelleyAddress::new(network, payment_part, staking_part);

        let address_string = address
            .to_bech32()
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
    let wallets = wallet_store
        .list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;

    let wallet_meta = wallets
        .iter()
        .find(|w| w.wallet_id == wallet_id)
        .ok_or_else(|| "Wallet not found".to_string())?;

    // Use stored Ed25519 public keys (64 bytes: 32 payment + 32 staking)
    if wallet_meta.master_public_key.len() != 64 {
        return Err(format!("Invalid Ed25519 public keys length: expected 64 bytes, got {}", wallet_meta.master_public_key.len()));
    }
    
    // Extract payment Ed25519 public key (first 32 bytes)
    // NOTE: For change addresses, we use the same payment key as the external addresses
    // In a full implementation, we would derive separate internal/external keys
    let mut payment_ed25519_key = [0u8; 32];
    payment_ed25519_key.copy_from_slice(&wallet_meta.master_public_key[..32]);
    
    // Extract staking Ed25519 public key (second 32 bytes)
    let mut staking_ed25519_key = [0u8; 32];
    staking_ed25519_key.copy_from_slice(&wallet_meta.master_public_key[32..]);

    // Generate change address using stored Ed25519 public keys
    let payment_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&payment_ed25519_key);
    let payment_part = pallas_addresses::ShelleyPaymentPart::Key(payment_pubkey_hash);

    let staking_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&staking_ed25519_key);
    let staking_part = pallas_addresses::ShelleyDelegationPart::Key(staking_pubkey_hash);

    // Create base address (payment + staking)
    let address = pallas_addresses::ShelleyAddress::new(network, payment_part, staking_part);
    let address_string = address
        .to_bech32()
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
    network: pallas_addresses::Network,
) -> Result<Vec<String>, String> {
    // Get wallet metadata with public key
    let wallets = wallet_store
        .list_wallets()
        .map_err(|e| format!("Failed to list wallets: {}", e))?;

    let wallet_meta = wallets
        .iter()
        .find(|w| w.wallet_id == wallet_id)
        .ok_or_else(|| "Wallet not found".to_string())?;

    // Use stored Ed25519 public keys (64 bytes: 32 payment + 32 staking)
    if wallet_meta.master_public_key.len() != 64 {
        return Err(format!("Invalid Ed25519 public keys length: expected 64 bytes, got {}", wallet_meta.master_public_key.len()));
    }
    
    // Extract staking Ed25519 public key (second 32 bytes)
    let mut staking_ed25519_key = [0u8; 32];
    staking_ed25519_key.copy_from_slice(&wallet_meta.master_public_key[32..]);

    let mut addresses: Vec<String> = Vec::new();

    let staking_pubkey_hash = pallas_crypto::hash::Hasher::<224>::hash(&staking_ed25519_key);
    //prefix e0 or e1 depending on network (e0 = testnet, e1 = mainnet)
    let prefix = match network {
        pallas_addresses::Network::Mainnet => "e1",
        pallas_addresses::Network::Testnet => "e0",
        _ => "e0", // Default to testnet prefix
    };
    let reward_address_hex = format!("{}{}", prefix, hex::encode(staking_pubkey_hash));
    //TODO: add more addresses and use account index in path
    addresses.push(reward_address_hex);

    Ok(addresses)
}
