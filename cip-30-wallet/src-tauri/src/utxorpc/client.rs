use std::str::FromStr;

use serde_json::{json, Value};
use utxorpc::spec::cardano::TxOutput;
use utxorpc::{CardanoQueryClient, ClientBuilder};
use utxorpc::spec;
use crate::config::UtxoRpcConfig;
use crate::wallet::convert_to_multiasset_positive_coin;

pub struct UtxoRpcClient {
    mainnet_client: Option<CardanoQueryClient>,
    testnet_client: Option<CardanoQueryClient>,
}

impl UtxoRpcClient {
    pub async fn new(config: UtxoRpcConfig) -> Result<Self, String> {
        let mut mainnet_client = None;
        let mut testnet_client = None;

        // Create mainnet client if URL is provided
        if let Some(mainnet_url) = config.mainnet_url {
            eprintln!("[UTxO RPC] Attempting to create mainnet client for URL: {}", mainnet_url);
            
            let builder = ClientBuilder::new()
                .uri(&mainnet_url)
                .map_err(|e| format!("Invalid mainnet URL: {}", e))?;

            let builder = if let Some(api_key) = &config.api_key {
                eprintln!("[UTxO RPC] Adding API key to mainnet client");
                builder
                    .metadata("dmtr-api-key", api_key)
                    .map_err(|e| format!("Failed to set API key: {}", e))?
            } else {
                eprintln!("[UTxO RPC] No API key provided for mainnet client");
                builder
            };

            // build() returns a Future that needs to be awaited
            eprintln!("[UTxO RPC] Building mainnet client...");
            mainnet_client = Some(builder.build::<CardanoQueryClient>().await);
            eprintln!("[UTxO RPC] Successfully created mainnet client for: {}", mainnet_url);
        }

        // Create testnet client if URL is provided
        if let Some(testnet_url) = config.testnet_url {
            eprintln!("[UTxO RPC] Attempting to create testnet client for URL: {}", testnet_url);
            
            let builder = ClientBuilder::new()
                .uri(&testnet_url)
                .map_err(|e| format!("Invalid testnet URL: {}", e))?;

            let builder = if let Some(api_key) = &config.api_key {
                eprintln!("[UTxO RPC] Adding API key to testnet client");
                builder
                    .metadata("dmtr-api-key", api_key)
                    .map_err(|e| format!("Failed to set API key: {}", e))?
            } else {
                eprintln!("[UTxO RPC] No API key provided for testnet client");
                builder
            };

            // build() returns a Future that needs to be awaited
            eprintln!("[UTxO RPC] Building testnet client...");
            testnet_client = Some(builder.build::<CardanoQueryClient>().await);
            eprintln!("[UTxO RPC] Successfully created testnet client for: {}", testnet_url);
        }

        Ok(Self {
            mainnet_client,
            testnet_client,
        })
    }

    /// Fetch UTxOs by stake address (hex format)
    pub async fn fetch_utxos_by_stake_address(
        &mut self,
        hex_stake_address: &str,
        network: pallas_addresses::Network,
        limit: Option<u32>,
    ) -> Result<Value, String> {
        let client = match network {
            pallas_addresses::Network::Mainnet => {
                self.mainnet_client.as_mut()
                    .ok_or("Mainnet UTxO RPC client not configured")?
            },
            _ => {
                self.testnet_client.as_mut()
                    .ok_or("Testnet UTxO RPC client not configured")?
            }
        };

        eprintln!("[UTxO RPC] Fetching UTxOs for stake address: {}", hex_stake_address);
        // Create address pattern for stake address matching
        let pattern = spec::cardano::TxOutputPattern {
            address: Some(spec::cardano::AddressPattern {
                exact_address: Default::default(),
                payment_part: Default::default(),
                delegation_part: hex::decode(hex_stake_address)
                    .unwrap()
                    .into(),
            }),
            asset: None,
        };

        // match_utxos returns Result<UtxoPage<C>, Error>
        eprintln!("[UTxO RPC] Sending match_utxos request...");
        let utxo_page = match client.match_utxos(pattern, None, limit.unwrap_or(100)).await {
            Ok(page) => {
                eprintln!("[UTxO RPC] Successfully received response");
                page
            },
            Err(e) => {
                eprintln!("[UTxO RPC] Error details: {:?}", e);
                return Err(format!("Failed to fetch UTxOs: {:?}", e));
            }
        };

        eprintln!("[UTxO RPC] Found {} UTxOs", utxo_page.items.len());
        
        // Convert UTxOs to CIP-30 compatible format
        let cip30_utxos = self.convert_utxos_to_cip30_format(utxo_page.items)?;

        Ok(json!(cip30_utxos))
    }

    /// Convert UTxO RPC response to CIP-30 CBOR hex format
    fn convert_utxos_to_cip30_format(
        &self,
        utxos: Vec<utxorpc::ChainUtxo<TxOutput>>
    ) -> Result<Vec<String>, String> {
        let mut cip30_utxos = Vec::new();
        for utxo in utxos {
            // Access the parsed UTxO data

            let parsed_output = utxo.parsed.unwrap();
            let utxo_ref = utxo.txo_ref.clone().unwrap();
            let tx_hash_hex = hex::encode(utxo_ref.hash);

            let tx_in = pallas_primitives::TransactionInput {
                transaction_id:  pallas_crypto::hash::Hash::from_str(&tx_hash_hex).expect("Invalid hash"),
                index: u64::from(utxo.txo_ref.unwrap().index),
            };
            let tx_out = pallas_primitives::conway::PostAlonzoTransactionOutput {
                address: pallas_primitives::Bytes::from(parsed_output.address.to_vec()),
                value: pallas_primitives::conway::Value::Multiasset(
                    parsed_output.coin,
                    convert_to_multiasset_positive_coin(parsed_output.assets).expect("Failed to convert assets"),
                    
                 ),
                datum_option: None,
                script_ref: None,
            };
            
            let transaction_unspent_output = (tx_in, tx_out);
            let cbor_utxo = pallas_codec::minicbor::to_vec(&transaction_unspent_output)
                .map_err(|e| format!("Failed to serialize UTxO to CBOR: {}", e))?;
            let cbor_hex = hex::encode(cbor_utxo);
            cip30_utxos.push(cbor_hex);
        }


        Ok(cip30_utxos)
    }
}

#[tokio::test]
    async fn test_match_utxos() {
        // Test using the same client initialization as production
        let config = crate::config::UtxoRpcConfig {
            mainnet_url: None,
            testnet_url: Some("http://localhost:50051".to_string()),
            api_key: Some("dmtr_utxorpc1wgnnj0qcfj32zxsz2uc8d4g7uclm2s2w".to_string()),
        };
        
        let mut utxo_client = UtxoRpcClient::new(config).await.unwrap();
        
        let hex_stake_address = "e027e96b274539896e4f8b79d2c0f8a76f3c247a61da4fd67fab52ab55";
        let result = utxo_client.fetch_utxos_by_stake_address(
            hex_stake_address,
            pallas_addresses::Network::Testnet,
            Some(100)
        ).await;
        
        match result {
            Ok(utxos) => {
                dbg!(&utxos);
                assert!(true); // Test passes if we get any response
            },
            Err(e) => {
                eprintln!("Test failed with error: {}", e);
                panic!("UTxO RPC test failed: {}", e);
            }
        }
    }