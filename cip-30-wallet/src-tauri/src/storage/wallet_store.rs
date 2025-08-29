use crate::crypto::encryption::{EncryptedData, WalletWrapper};
use crate::wallet::{WalletError, WalletResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Wallet metadata stored in plain text
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletMetadata {
    pub name: String,
    pub created_at: i64,
    pub wallet_id: String,
    pub master_public_key: Vec<u8>, // Store Ed25519 public keys: payment (32 bytes) + staking (32 bytes) = 64 bytes
}

/// Wallet storage manager
pub struct WalletStore {
    data_dir: PathBuf,
}

impl WalletStore {
    /// Create new wallet store with data directory
    pub fn new(data_dir: PathBuf) -> WalletResult<Self> {
        // Ensure data directory exists
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|e| {
                WalletError::StorageError(format!("Failed to create data directory: {}", e))
            })?;
        }

        Ok(Self { data_dir })
    }

    /// Get default data directory for the application
    pub fn default_data_dir() -> WalletResult<PathBuf> {
        let app_data = dirs::data_dir().ok_or_else(|| {
            WalletError::StorageError("Could not find app data directory".to_string())
        })?;

        Ok(app_data.join("thresh-wallet"))
    }

    /// Check if a wallet exists
    pub fn wallet_exists(&self, wallet_id: &str) -> bool {
        self.get_wallet_path(wallet_id).exists()
    }

    /// Get list of all wallet metadata
    pub fn list_wallets(&self) -> WalletResult<Vec<WalletMetadata>> {
        let mut wallets = Vec::new();

        eprintln!("Listing wallets in directory: {:?}", self.data_dir);
        eprintln!("Directory exists: {}", self.data_dir.exists());

        if !self.data_dir.exists() {
            eprintln!("Data directory doesn't exist, returning empty list");
            return Ok(wallets);
        }

        let entries = fs::read_dir(&self.data_dir).map_err(|e| {
            WalletError::StorageError(format!("Failed to read data directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                WalletError::StorageError(format!("Failed to read directory entry: {}", e))
            })?;

            if entry
                .file_type()
                .map_err(|e| WalletError::StorageError(format!("Failed to get file type: {}", e)))?
                .is_file()
            {
                let file_name = entry.file_name();
                let file_name_str = file_name
                    .to_str()
                    .ok_or_else(|| WalletError::StorageError("Invalid file name".to_string()))?;

                if file_name_str.ends_with(".wallet") {
                    let wallet_id = file_name_str.trim_end_matches(".wallet");

                    // Try to load wallet metadata
                    if let Ok(metadata) = self.load_wallet_metadata(wallet_id) {
                        wallets.push(metadata);
                    }
                }
            }
        }

        Ok(wallets)
    }

    /// Save encrypted wallet to storage
    pub fn save_wallet(
        &self,
        wallet_id: &str,
        wallet: &WalletWrapper,
        password: &str,
        master_public_key: &[u8; 64],
    ) -> WalletResult<()> {
        let encrypted = wallet.encrypt(password)?;

        // Save encrypted wallet data
        let wallet_path = self.get_wallet_path(wallet_id);
        let encrypted_bytes = encrypted.to_bytes();

        eprintln!("Saving wallet to path: {:?}", wallet_path);
        eprintln!("Encrypted wallet size: {} bytes", encrypted_bytes.len());

        fs::write(&wallet_path, &encrypted_bytes)
            .map_err(|e| WalletError::StorageError(format!("Failed to save wallet: {}", e)))?;

        eprintln!("Wallet file written successfully");

        // Save wallet metadata
        let metadata = WalletMetadata {
            name: wallet.name.clone(),
            created_at: wallet.created_at,
            wallet_id: wallet_id.to_string(),
            master_public_key: master_public_key.to_vec(),
        };

        self.save_wallet_metadata(wallet_id, &metadata)?;
        eprintln!("Wallet metadata saved successfully");

        Ok(())
    }

    /// Load encrypted wallet from storage
    pub fn load_wallet(&self, wallet_id: &str, password: &str) -> WalletResult<WalletWrapper> {
        let wallet_path = self.get_wallet_path(wallet_id);

        if !wallet_path.exists() {
            return Err(WalletError::NotFound);
        }

        let encrypted_bytes = fs::read(&wallet_path)
            .map_err(|e| WalletError::StorageError(format!("Failed to load wallet: {}", e)))?;

        let encrypted_data = EncryptedData::from_bytes(&encrypted_bytes)?;
        let wallet = WalletWrapper::decrypt(&encrypted_data, password)?;

        Ok(wallet)
    }

    /// Delete wallet from storage
    pub fn delete_wallet(&self, wallet_id: &str) -> WalletResult<()> {
        let wallet_path = self.get_wallet_path(wallet_id);
        let metadata_path = self.get_metadata_path(wallet_id);

        // Remove wallet file
        if wallet_path.exists() {
            fs::remove_file(&wallet_path).map_err(|e| {
                WalletError::StorageError(format!("Failed to delete wallet: {}", e))
            })?;
        }

        // Remove metadata file
        if metadata_path.exists() {
            fs::remove_file(&metadata_path).map_err(|e| {
                WalletError::StorageError(format!("Failed to delete wallet metadata: {}", e))
            })?;
        }

        Ok(())
    }

    /// Generate unique wallet ID
    pub fn generate_wallet_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        format!("wallet_{}", timestamp)
    }

    // Private helper methods

    fn get_wallet_path(&self, wallet_id: &str) -> PathBuf {
        self.data_dir.join(format!("{}.wallet", wallet_id))
    }

    fn get_metadata_path(&self, wallet_id: &str) -> PathBuf {
        self.data_dir.join(format!("{}.meta", wallet_id))
    }

    fn save_wallet_metadata(&self, wallet_id: &str, metadata: &WalletMetadata) -> WalletResult<()> {
        let metadata_path = self.get_metadata_path(wallet_id);
        let metadata_json = serde_json::to_string_pretty(metadata).map_err(|e| {
            WalletError::StorageError(format!("Failed to serialize metadata: {}", e))
        })?;

        fs::write(&metadata_path, metadata_json)
            .map_err(|e| WalletError::StorageError(format!("Failed to save metadata: {}", e)))?;

        Ok(())
    }

    fn load_wallet_metadata(&self, wallet_id: &str) -> WalletResult<WalletMetadata> {
        let metadata_path = self.get_metadata_path(wallet_id);

        if !metadata_path.exists() {
            return Err(WalletError::NotFound);
        }

        let metadata_json = fs::read_to_string(&metadata_path)
            .map_err(|e| WalletError::StorageError(format!("Failed to load metadata: {}", e)))?;

        let metadata: WalletMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| WalletError::StorageError(format!("Failed to parse metadata: {}", e)))?;

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::mnemonic::generate_mnemonic;
    use tempfile::tempdir;

    #[test]
    fn test_wallet_storage() {
        let temp_dir = tempdir().unwrap();
        let store = WalletStore::new(temp_dir.path().to_path_buf()).unwrap();

        let mnemonic = generate_mnemonic(12).unwrap();
        let wallet = WalletWrapper::new(mnemonic.clone(), "Test Wallet".to_string());
        let wallet_id = WalletStore::generate_wallet_id();
        let password = "test_password_123";

        // Derive master public key for storage
        let seed = crate::crypto::mnemonic::mnemonic_to_seed(&mnemonic, "").unwrap();
        let master_private_key =
            crate::crypto::keys::Bip32PrivateKey::from_bip39_seed(&seed).unwrap();
        let account_key = master_private_key
            .derive(1852 | 0x80000000) // purpose
            .derive(1815 | 0x80000000) // coin_type
            .derive(0 | 0x80000000); // account
        // Derive Ed25519 public keys for both payment and staking (same as in commands.rs)
        let external_chain = account_key.derive(0); // external chain
        let first_address_private = external_chain.derive(0); // first address
        let payment_signing_key = first_address_private.to_signing_key();
        let payment_ed25519_private = pallas_crypto::key::ed25519::SecretKey::from(payment_signing_key);
        let payment_ed25519_public = payment_ed25519_private.public_key();
        
        let staking_chain = account_key.derive(2); // staking chain
        let staking_private = staking_chain.derive(0); // first staking key
        let staking_signing_key = staking_private.to_signing_key();
        let staking_ed25519_private = pallas_crypto::key::ed25519::SecretKey::from(staking_signing_key);
        let staking_ed25519_public = staking_ed25519_private.public_key();
        
        let mut master_public_key = [0u8; 64];
        master_public_key[..32].copy_from_slice(payment_ed25519_public.as_ref());
        master_public_key[32..].copy_from_slice(staking_ed25519_public.as_ref());

        // Save wallet
        store
            .save_wallet(&wallet_id, &wallet, password, &master_public_key)
            .unwrap();

        // Check wallet exists
        assert!(store.wallet_exists(&wallet_id));

        // Load wallet
        let loaded_wallet = store.load_wallet(&wallet_id, password).unwrap();
        assert_eq!(wallet.name, loaded_wallet.name);
        assert_eq!(wallet.mnemonic, loaded_wallet.mnemonic);

        // List wallets
        let wallets = store.list_wallets().unwrap();
        assert_eq!(wallets.len(), 1);
        assert_eq!(wallets[0].name, "Test Wallet");

        // Delete wallet
        store.delete_wallet(&wallet_id).unwrap();
        assert!(!store.wallet_exists(&wallet_id));
    }
}
