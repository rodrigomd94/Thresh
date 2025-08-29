use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub utxorpc: Option<UtxoRpcConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_network")]
    pub name: String, // "mainnet" or "testnet"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoRpcConfig {
    pub mainnet_url: Option<String>,
    pub testnet_url: Option<String>,
    pub api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                name: default_network(),
            },
            utxorpc: Some(UtxoRpcConfig::default()),
        }
    }
}

impl Default for UtxoRpcConfig {
    fn default() -> Self {
        Self {
            mainnet_url: Some("http://localhost:50051".to_string()),
            testnet_url: Some("http://localhost:50051".to_string()),
            api_key: None,
        }
    }
}

impl AppConfig {
    /// Load configuration from file, creating default if it doesn't exist
    pub fn load_or_create<P: AsRef<Path>>(config_path: P) -> Result<Self, String> {
        let path = config_path.as_ref();
        
        if path.exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;
            
            let config: AppConfig = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config file: {}", e))?;
            
            eprintln!("Loaded configuration from: {:?}", path);
            eprintln!("Network: {}", config.network.name);
            if let Some(ref utxorpc) = config.utxorpc {
                if let Some(ref mainnet_url) = utxorpc.mainnet_url {
                    eprintln!("UTXO RPC Mainnet URL: {}", mainnet_url);
                }
                if let Some(ref testnet_url) = utxorpc.testnet_url {
                    eprintln!("UTXO RPC Testnet URL: {}", testnet_url);
                }
            }
            
            Ok(config)
        } else {
            let config = Self::default();
            config.save(path)?;
            eprintln!("Created default configuration at: {:?}", path);
            Ok(config)
        }
    }
    
    /// Save configuration to file
    pub fn save<P: AsRef<Path>>(&self, config_path: P) -> Result<(), String> {
        let path = config_path.as_ref();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        Ok(())
    }
    
    /// Get Pallas network enum from config
    pub fn get_network(&self) -> pallas_addresses::Network {
        match self.network.name.to_lowercase().as_str() {
            "mainnet" => pallas_addresses::Network::Mainnet,
            "testnet" => pallas_addresses::Network::Testnet,
            _ => {
                eprintln!("Unknown network '{}', defaulting to mainnet", self.network.name);
                pallas_addresses::Network::Mainnet
            }
        }
    }
    
    /// Get default config file path
    pub fn default_config_path() -> Result<std::path::PathBuf, String> {
        let data_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?;
        
        Ok(data_dir.join("thresh-wallet").join("config.toml"))
    }
}

fn default_network() -> String {
    "mainnet".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.network.name, "mainnet");
        assert!(config.utxorpc.is_none());
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path();

        // Save default config
        let original_config = AppConfig::default();
        original_config.save(config_path).unwrap();

        // Load config back
        let loaded_config = AppConfig::load_or_create(config_path).unwrap();
        assert_eq!(loaded_config.network.name, "mainnet");
    }

    #[test]
    fn test_network_conversion() {
        let mainnet_config = AppConfig {
            network: NetworkConfig {
                name: "mainnet".to_string(),
            },
            utxorpc: None,
        };
        assert_eq!(mainnet_config.get_network(), pallas_addresses::Network::Mainnet);

        let testnet_config = AppConfig {
            network: NetworkConfig {
                name: "testnet".to_string(),
            },
            utxorpc: None,
        };
        assert_eq!(testnet_config.get_network(), pallas_addresses::Network::Testnet);
    }
}