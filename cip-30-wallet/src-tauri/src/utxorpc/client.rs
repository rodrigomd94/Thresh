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
            let builder = ClientBuilder::new()
                .uri(&mainnet_url)
                .map_err(|e| format!("Invalid mainnet URL: {}", e))?;

            let builder = if let Some(api_key) = &config.api_key {
                builder
                    .metadata("dmtr-api-key", api_key)
                    .map_err(|e| format!("Failed to set API key: {}", e))?
            } else {
                builder
            };

            // build() returns a Future that needs to be awaited
            mainnet_client = Some(builder.build::<CardanoQueryClient>().await);
            eprintln!("[UTxO RPC] Created mainnet client for: {}", mainnet_url);
        }

        // Create testnet client if URL is provided
        if let Some(testnet_url) = config.testnet_url {
            let builder = ClientBuilder::new()
                .uri(&testnet_url)
                .map_err(|e| format!("Invalid testnet URL: {}", e))?;

            let builder = if let Some(api_key) = &config.api_key {
                builder
                    .metadata("dmtr-api-key", api_key)
                    .map_err(|e| format!("Failed to set API key: {}", e))?
            } else {
                builder
            };

            // build() returns a Future that needs to be awaited
            testnet_client = Some(builder.build::<CardanoQueryClient>().await);
            eprintln!("[UTxO RPC] Created testnet client for: {}", testnet_url);
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

        // Convert hex stake address to bytes
        let stake_address_bytes = hex::decode(hex_stake_address)
            .map_err(|e| format!("Invalid hex stake address: {}", e))?;

        // Create address pattern for stake address matching
        let pattern = spec::cardano::TxOutputPattern {
            address: Some(spec::cardano::AddressPattern {
                exact_address: stake_address_bytes.into(),
                payment_part: Default::default(),
                delegation_part: Default::default(),
            }),
            asset: None,
        };

        // match_utxos returns Result<UtxoPage<C>, Error>
        let utxo_page = client
            .match_utxos(pattern, None, limit.unwrap_or(100))
            .await
            .map_err(|e| format!("Failed to fetch UTxOs: {:?}", e))?;

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
                //amount: pallas_primitives::alonzo::Value::Multiasset(tx_output.coin, tx_output.assets),
                //datum_hash: None, // Handle datum hash if needed
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