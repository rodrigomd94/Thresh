use ed25519_bip32::{XPrv, XPub, DerivationScheme};
use crate::wallet::{WalletError, WalletResult};

/// Secret key types following Pallas wallet pattern
#[derive(Debug, Clone)]
pub enum SecretKey {
    Normal([u8; 32]),     // Standard Ed25519 key
    Extended([u8; 64]),   // Extended key with chain code
}

impl SecretKey {
    /// Get the length of the key in bytes
    pub fn len(&self) -> usize {
        match self {
            SecretKey::Normal(_) => 32,
            SecretKey::Extended(_) => 64,
        }
    }

    /// Get raw bytes of the key
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            SecretKey::Normal(bytes) => bytes,
            SecretKey::Extended(bytes) => bytes,
        }
    }

    /// Convert to normal key (first 32 bytes)
    pub fn to_normal(&self) -> [u8; 32] {
        match self {
            SecretKey::Normal(key) => *key,
            SecretKey::Extended(key) => {
                let mut normal = [0u8; 32];
                normal.copy_from_slice(&key[..32]);
                normal
            }
        }
    }

    /// Generate public key
    pub fn to_public(&self) -> WalletResult<[u8; 32]> {
        let private_key = self.to_normal();
        
        // Use ed25519-dalek for public key generation
        use ed25519_dalek::{SigningKey, VerifyingKey};
        
        let signing_key = SigningKey::from_bytes(&private_key);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        
        Ok(verifying_key.to_bytes())
    }
}

/// BIP32 private key for Cardano (following Pallas pattern)
#[derive(Debug, Clone)]
pub struct Bip32PrivateKey(XPrv);

impl Bip32PrivateKey {
    /// Generate from BIP39 seed (following Pallas pattern)
    pub fn from_bip39_seed(seed: &[u8; 64]) -> WalletResult<Self> {
        // Use the seed directly with normalize_bytes_force3rd following Pallas pattern
        let mut xprv_bytes = [0u8; 96]; // XPrv size
        xprv_bytes[..64].copy_from_slice(seed);
        
        let xprv = XPrv::normalize_bytes_force3rd(xprv_bytes);
        
        Ok(Bip32PrivateKey(xprv))
    }

    /// Derive child key (following Pallas pattern)
    pub fn derive(&self, index: u32) -> Self {
        Self(self.0.derive(DerivationScheme::V2, index))
    }

    /// Get public key
    pub fn to_public(&self) -> Bip32PublicKey {
        Bip32PublicKey(self.0.public())
    }

    /// Get raw private key bytes
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0.extended_secret_key().clone()
    }

    /// Get signing key for transactions
    pub fn to_signing_key(&self) -> [u8; 32] {
        let extended = self.0.extended_secret_key().clone();
        let mut key = [0u8; 32];
        key.copy_from_slice(&extended[..32]);
        key
    }
}

/// BIP32 public key for Cardano (following Pallas pattern)
#[derive(Debug, Clone)]
pub struct Bip32PublicKey(XPub);

impl Bip32PublicKey {
    /// Create from extended public key bytes (including chain code)
    pub fn from_extended_bytes(bytes: &[u8; 64]) -> WalletResult<Self> {
        let xpub = XPub::from_bytes(*bytes);
        Ok(Bip32PublicKey(xpub))
    }

    /// Derive child public key (following Pallas pattern with error handling)
    pub fn derive(&self, index: u32) -> WalletResult<Self> {
        self.0
            .derive(DerivationScheme::V2, index)
            .map(Self)
            .map_err(|e| WalletError::DerivationError(format!("Public key derivation failed: {:?}", e)))
    }

    /// Get raw public key bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.public_key().clone()
    }

    /// Get extended public key bytes (including chain code)
    pub fn to_extended_bytes(&self) -> [u8; 64] {
        // Use as_ref() to get the full 64-byte representation
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(self.0.as_ref());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::mnemonic::{generate_mnemonic, mnemonic_to_entropy};

    #[test]
    fn test_bip32_key_generation() {
        let mnemonic = generate_mnemonic(12).unwrap();
        let seed = crate::crypto::mnemonic::mnemonic_to_seed(&mnemonic, "").unwrap();
        
        let master_key = Bip32PrivateKey::from_bip39_seed(&seed).unwrap();
        let public_key = master_key.to_public();
        
        assert_eq!(master_key.to_bytes().len(), 64);
        assert_eq!(public_key.to_bytes().len(), 32);
    }

    #[test]
    fn test_cardano_derivation_path() {
        let mnemonic = generate_mnemonic(24).unwrap();
        let seed = crate::crypto::mnemonic::mnemonic_to_seed(&mnemonic, "").unwrap();
        
        let master = Bip32PrivateKey::from_bip39_seed(&seed).unwrap();
        
        // Cardano derivation: m/1852'/1815'/0'
        let purpose = master.derive(1852 | 0x80000000);  // Hardened
        let coin_type = purpose.derive(1815 | 0x80000000);  // Hardened
        let account = coin_type.derive(0 | 0x80000000);  // Hardened
        
        // Derive first external address: m/1852'/1815'/0'/0/0
        let external_chain = account.derive(0);  // Non-hardened
        let address_key = external_chain.derive(0);  // Non-hardened
        
        assert_eq!(address_key.to_bytes().len(), 64);
    }
}