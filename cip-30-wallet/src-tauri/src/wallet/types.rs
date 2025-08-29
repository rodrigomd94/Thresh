use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub account_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub account_index: u32,
    pub external_addresses: Vec<AddressInfo>,
    pub internal_addresses: Vec<AddressInfo>,
    pub next_external_index: u32,
    pub next_internal_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub derivation_path: String,
    pub index: u32,
    pub used: bool,
}

#[derive(Debug, Clone)]
pub enum WalletError {
    NotFound,
    AlreadyExists,
    InvalidPassword,
    InvalidMnemonic,
    CryptoError(String),
    StorageError(String),
    DerivationError(String),
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::NotFound => write!(f, "Wallet not found"),
            WalletError::AlreadyExists => write!(f, "Wallet already exists"),
            WalletError::InvalidPassword => write!(f, "Invalid password"),
            WalletError::InvalidMnemonic => write!(f, "Invalid mnemonic phrase"),
            WalletError::CryptoError(e) => write!(f, "Cryptographic error: {}", e),
            WalletError::StorageError(e) => write!(f, "Storage error: {}", e),
            WalletError::DerivationError(e) => write!(f, "Key derivation error: {}", e),
        }
    }
}

impl std::error::Error for WalletError {}

pub type WalletResult<T> = Result<T, WalletError>;
