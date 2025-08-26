use cryptoxide::chacha20poly1305::ChaCha20Poly1305;
use cryptoxide::kdf::argon2;
use crate::wallet::{WalletError, WalletResult};
use rand::RngCore;

const VERSION: u8 = 1;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;
const ITERATIONS: u32 = 2500;

/// Encrypted wallet data structure
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub version: u8,
    pub salt: [u8; SALT_SIZE],
    pub nonce: [u8; NONCE_SIZE],
    pub tag: [u8; TAG_SIZE],
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Serialize to bytes following Pallas format: version || salt || nonce || tag || ciphertext
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + SALT_SIZE + NONCE_SIZE + TAG_SIZE + self.ciphertext.len());
        bytes.push(self.version);
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.tag);
        bytes.extend_from_slice(&self.ciphertext);
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> WalletResult<Self> {
        if bytes.len() < 1 + SALT_SIZE + NONCE_SIZE + TAG_SIZE {
            return Err(WalletError::CryptoError("Invalid encrypted data size".to_string()));
        }

        let version = bytes[0];
        if version != VERSION {
            return Err(WalletError::CryptoError(format!("Unsupported version: {}", version)));
        }

        let mut salt = [0u8; SALT_SIZE];
        salt.copy_from_slice(&bytes[1..1 + SALT_SIZE]);

        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&bytes[1 + SALT_SIZE..1 + SALT_SIZE + NONCE_SIZE]);

        let mut tag = [0u8; TAG_SIZE];
        tag.copy_from_slice(&bytes[1 + SALT_SIZE + NONCE_SIZE..1 + SALT_SIZE + NONCE_SIZE + TAG_SIZE]);

        let ciphertext = bytes[1 + SALT_SIZE + NONCE_SIZE + TAG_SIZE..].to_vec();

        Ok(EncryptedData {
            version,
            salt,
            nonce,
            tag,
            ciphertext,
        })
    }
}

/// Encrypt data using password-based encryption
pub fn encrypt_with_password(data: &[u8], password: &str) -> WalletResult<EncryptedData> {
    // Generate random salt and nonce
    let mut salt = [0u8; SALT_SIZE];
    let mut nonce = [0u8; NONCE_SIZE];
    
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce);

    // Derive key using Argon2 (following Pallas pattern)
    let sym_key: [u8; 32] = argon2::argon2(
        &argon2::Params::argon2d().iterations(ITERATIONS).unwrap(),
        password.as_bytes(),
        &salt,
        &[],
        &[]
    );

    // Encrypt with ChaCha20Poly1305
    let mut cipher = ChaCha20Poly1305::new(&sym_key, &nonce, &[]);
    let mut ciphertext = vec![0u8; data.len()];
    let mut tag = [0u8; TAG_SIZE];

    cipher.encrypt(data, &mut ciphertext, &mut tag);

    Ok(EncryptedData {
        version: VERSION,
        salt,
        nonce,
        tag,
        ciphertext,
    })
}

/// Decrypt data using password
pub fn decrypt_with_password(encrypted: &EncryptedData, password: &str) -> WalletResult<Vec<u8>> {
    // Derive key using same parameters
    let sym_key: [u8; 32] = argon2::argon2(
        &argon2::Params::argon2d().iterations(ITERATIONS).unwrap(),
        password.as_bytes(),
        &encrypted.salt,
        &[],
        &[]
    );

    // Decrypt with ChaCha20Poly1305
    let mut cipher = ChaCha20Poly1305::new(&sym_key, &encrypted.nonce, &[]);
    let mut plaintext = vec![0u8; encrypted.ciphertext.len()];

    let valid = cipher.decrypt(&encrypted.ciphertext, &mut plaintext, &encrypted.tag);

    if !valid {
        return Err(WalletError::InvalidPassword);
    }

    Ok(plaintext)
}

/// Wrapper for wallet data encryption
#[derive(Debug)]
pub struct WalletWrapper {
    pub mnemonic: Vec<String>,
    pub created_at: i64,
    pub name: String,
}

impl WalletWrapper {
    pub fn new(mnemonic: Vec<String>, name: String) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            mnemonic,
            created_at,
            name,
        }
    }

    /// Serialize wallet data to JSON bytes
    pub fn to_bytes(&self) -> WalletResult<Vec<u8>> {
        let json = serde_json::json!({
            "mnemonic": self.mnemonic,
            "created_at": self.created_at,
            "name": self.name,
        });

        serde_json::to_vec(&json)
            .map_err(|e| WalletError::StorageError(format!("Serialization failed: {}", e)))
    }

    /// Deserialize wallet data from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> WalletResult<Self> {
        let json: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| WalletError::StorageError(format!("Deserialization failed: {}", e)))?;

        let mnemonic = json["mnemonic"]
            .as_array()
            .ok_or_else(|| WalletError::StorageError("Invalid mnemonic format".to_string()))?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect();

        let created_at = json["created_at"]
            .as_i64()
            .ok_or_else(|| WalletError::StorageError("Invalid created_at format".to_string()))?;

        let name = json["name"]
            .as_str()
            .ok_or_else(|| WalletError::StorageError("Invalid name format".to_string()))?
            .to_string();

        Ok(Self {
            mnemonic,
            created_at,
            name,
        })
    }

    /// Encrypt wallet data with password
    pub fn encrypt(&self, password: &str) -> WalletResult<EncryptedData> {
        let data = self.to_bytes()?;
        encrypt_with_password(&data, password)
    }

    /// Decrypt wallet data from encrypted form
    pub fn decrypt(encrypted: &EncryptedData, password: &str) -> WalletResult<Self> {
        let data = decrypt_with_password(encrypted, password)?;
        Self::from_bytes(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::mnemonic::generate_mnemonic;

    #[test]
    fn test_encryption_roundtrip() {
        let data = b"Hello, world!";
        let password = "test_password_123";

        let encrypted = encrypt_with_password(data, password).unwrap();
        let decrypted = decrypt_with_password(&encrypted, password).unwrap();

        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_wrong_password() {
        let data = b"Hello, world!";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_with_password(data, password).unwrap();
        let result = decrypt_with_password(&encrypted, wrong_password);

        assert!(matches!(result, Err(WalletError::InvalidPassword)));
    }

    #[test]
    fn test_wallet_wrapper_encryption() {
        let mnemonic = generate_mnemonic(12).unwrap();
        let wallet = WalletWrapper::new(mnemonic.clone(), "Test Wallet".to_string());
        let password = "secure_password_123";

        let encrypted = wallet.encrypt(password).unwrap();
        let decrypted = WalletWrapper::decrypt(&encrypted, password).unwrap();

        assert_eq!(wallet.mnemonic, decrypted.mnemonic);
        assert_eq!(wallet.name, decrypted.name);
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let data = b"test data";
        let password = "test123";

        let encrypted = encrypt_with_password(data, password).unwrap();
        let bytes = encrypted.to_bytes();
        let recovered = EncryptedData::from_bytes(&bytes).unwrap();

        let decrypted = decrypt_with_password(&recovered, password).unwrap();
        assert_eq!(data, decrypted.as_slice());
    }
}