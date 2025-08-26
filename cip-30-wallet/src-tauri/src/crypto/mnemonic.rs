use bip39::{Mnemonic, Language};
use crate::wallet::{WalletError, WalletResult};

/// Generate a new BIP39 mnemonic phrase
pub fn generate_mnemonic(word_count: usize) -> WalletResult<Vec<String>> {
    let entropy_bits = match word_count {
        12 => 128,
        15 => 160,
        18 => 192,
        21 => 224,
        24 => 256,
        _ => return Err(WalletError::InvalidMnemonic),
    };

    let entropy_bytes = entropy_bits / 8;
    let mut entropy = vec![0u8; entropy_bytes];
    
    // Generate random entropy
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut entropy);

    let mnemonic = Mnemonic::from_entropy(&entropy)
        .map_err(|_| WalletError::InvalidMnemonic)?;
    
    let words: Vec<String> = mnemonic.words()
        .map(|s| s.to_string())
        .collect();

    Ok(words)
}

/// Validate a mnemonic phrase
pub fn validate_mnemonic(words: &[String]) -> WalletResult<()> {
    let phrase = words.join(" ");
    
    match Mnemonic::parse_in(Language::English, &phrase) {
        Ok(_) => Ok(()),
        Err(_) => Err(WalletError::InvalidMnemonic),
    }
}

/// Convert mnemonic to seed bytes
pub fn mnemonic_to_seed(words: &[String], passphrase: &str) -> WalletResult<[u8; 64]> {
    let phrase = words.join(" ");
    
    let mnemonic = Mnemonic::parse_in(Language::English, &phrase)
        .map_err(|_| WalletError::InvalidMnemonic)?;
    
    let seed = mnemonic.to_seed(passphrase);
    
    // Convert Vec<u8> to [u8; 64]
    let mut seed_array = [0u8; 64];
    if seed.len() >= 64 {
        seed_array.copy_from_slice(&seed[..64]);
    }
    Ok(seed_array)
}

/// Get entropy from mnemonic (for BIP32 key generation)
pub fn mnemonic_to_entropy(words: &[String]) -> WalletResult<Vec<u8>> {
    let phrase = words.join(" ");
    
    let mnemonic = Mnemonic::parse_in(Language::English, &phrase)
        .map_err(|_| WalletError::InvalidMnemonic)?;
    
    Ok(mnemonic.to_entropy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic_12_words() {
        let mnemonic = generate_mnemonic(12).unwrap();
        assert_eq!(mnemonic.len(), 12);
    }

    #[test]
    fn test_generate_mnemonic_24_words() {
        let mnemonic = generate_mnemonic(24).unwrap();
        assert_eq!(mnemonic.len(), 24);
    }

    #[test]
    fn test_validate_mnemonic() {
        let mnemonic = generate_mnemonic(12).unwrap();
        assert!(validate_mnemonic(&mnemonic).is_ok());
    }

    #[test]
    fn test_invalid_mnemonic() {
        let invalid_words = vec!["invalid".to_string(), "mnemonic".to_string()];
        assert!(validate_mnemonic(&invalid_words).is_err());
    }

    #[test]
    fn test_mnemonic_to_seed() {
        let mnemonic = generate_mnemonic(12).unwrap();
        let seed = mnemonic_to_seed(&mnemonic, "").unwrap();
        assert_eq!(seed.len(), 64);
    }
}