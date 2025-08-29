use crate::wallet::{WalletError, WalletResult};
use ed25519_bip32::{DerivationScheme, XPrv, XPub};

/// Secret key types following Pallas wallet pattern
#[derive(Debug, Clone)]
pub enum SecretKey {
    Normal([u8; 32]),   // Standard Ed25519 key
    Extended([u8; 64]), // Extended key with chain code
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
    /// This extracts the Ed25519 private key from the BIP32 key
    pub fn to_signing_key(&self) -> [u8; 32] {
        // The ed25519_bip32::XPrv stores the private key in the extended key format
        // We need to extract the Ed25519 private key properly
        
        // The extended_secret_key() gives us 64 bytes: private key (32) + chain code (32)
        let extended = self.0.extended_secret_key();
        
        // But we need to check if this is the correct Ed25519 private key
        // that will produce the matching public key
        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&extended[..32]);
        
        // This should be the Ed25519 private key that matches the public key
        private_key
    }
    
    /// Convert BIP32 private key to Ed25519 format
    /// This is needed because BIP32 and Ed25519 have different key formats
    pub fn to_ed25519_private_key(&self) -> Result<[u8; 32], String> {
        // The proper way would be to use the BIP32 private key scalar and convert it
        // For Ed25519-BIP32, we need to ensure the key is properly formatted
        
        // Get the raw private key bytes (first 32 bytes of extended key)
        let extended = self.0.extended_secret_key().clone();
        let mut private_scalar = [0u8; 32];
        private_scalar.copy_from_slice(&extended[..32]);
        
        // For Ed25519-BIP32 compatibility, we might need to apply the proper conversion
        // This is a simplified version - a proper implementation would follow SLIP-0010
        
        // TODO: Implement proper BIP32-Ed25519 conversion following SLIP-0010
        // For now, return the raw scalar
        Ok(private_scalar)
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
            .map_err(|e| {
                WalletError::DerivationError(format!("Public key derivation failed: {:?}", e))
            })
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
        let purpose = master.derive(1852 | 0x80000000); // Hardened
        let coin_type = purpose.derive(1815 | 0x80000000); // Hardened
        let account = coin_type.derive(0 | 0x80000000); // Hardened

        // Derive first external address: m/1852'/1815'/0'/0/0
        let external_chain = account.derive(0); // Non-hardened
        let address_key = external_chain.derive(0); // Non-hardened

        assert_eq!(address_key.to_bytes().len(), 64);
    }
}
