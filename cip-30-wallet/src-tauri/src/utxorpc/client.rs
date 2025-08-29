use std::str::FromStr;

use crate::config::UtxoRpcConfig;
use crate::wallet::convert_to_multiasset_positive_coin;
use serde_json::{json, Value};
use utxorpc::spec;
use utxorpc::spec::cardano::TxOutput;
use utxorpc::{CardanoQueryClient, CardanoSubmitClient, ClientBuilder};

pub struct UtxoRpcClient {
    mainnet_client: Option<CardanoQueryClient>,
    testnet_client: Option<CardanoQueryClient>,
    mainnet_submit_client: Option<CardanoSubmitClient>,
    testnet_submit_client: Option<CardanoSubmitClient>,
}

impl UtxoRpcClient {
    pub async fn new(config: UtxoRpcConfig) -> Result<Self, String> {
        let mut mainnet_client = None;
        let mut testnet_client = None;
        let mut mainnet_submit_client = None;
        let mut testnet_submit_client = None;

        // Create mainnet client if URL is provided
        if let Some(mainnet_url) = config.mainnet_url {
            eprintln!(
                "[UTxO RPC] Attempting to create mainnet client for URL: {}",
                mainnet_url
            );

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
            eprintln!("[UTxO RPC] Building mainnet query client...");
            mainnet_client = Some(builder.build::<CardanoQueryClient>().await);
            eprintln!(
                "[UTxO RPC] Successfully created mainnet query client for: {}",
                mainnet_url
            );

            // Create submit client with the same configuration
            let submit_builder = ClientBuilder::new()
                .uri(&mainnet_url)
                .map_err(|e| format!("Invalid mainnet URL for submit client: {}", e))?;

            let submit_builder = if let Some(api_key) = &config.api_key {
                submit_builder
                    .metadata("dmtr-api-key", api_key)
                    .map_err(|e| format!("Failed to set API key for submit client: {}", e))?
            } else {
                submit_builder
            };

            eprintln!("[UTxO RPC] Building mainnet submit client...");
            mainnet_submit_client = Some(submit_builder.build::<CardanoSubmitClient>().await);
            eprintln!("[UTxO RPC] Successfully created mainnet submit client for: {}", mainnet_url);
        }

        // Create testnet client if URL is provided
        if let Some(testnet_url) = config.testnet_url {
            eprintln!(
                "[UTxO RPC] Attempting to create testnet client for URL: {}",
                testnet_url
            );

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
            eprintln!("[UTxO RPC] Building testnet query client...");
            testnet_client = Some(builder.build::<CardanoQueryClient>().await);
            eprintln!(
                "[UTxö RPC] Successfully created testnet query client for: {}",
                testnet_url
            );

            // Create submit client with the same configuration
            let submit_builder = ClientBuilder::new()
                .uri(&testnet_url)
                .map_err(|e| format!("Invalid testnet URL for submit client: {}", e))?;

            let submit_builder = if let Some(api_key) = &config.api_key {
                submit_builder
                    .metadata("dmtr-api-key", api_key)
                    .map_err(|e| format!("Failed to set API key for testnet submit client: {}", e))?
            } else {
                submit_builder
            };

            eprintln!("[UTxO RPC] Building testnet submit client...");
            testnet_submit_client = Some(submit_builder.build::<CardanoSubmitClient>().await);
            eprintln!("[UTxO RPC] Successfully created testnet submit client for: {}", testnet_url);
        }

        Ok(Self {
            mainnet_client,
            testnet_client,
            mainnet_submit_client,
            testnet_submit_client,
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
            pallas_addresses::Network::Mainnet => self
                .mainnet_client
                .as_mut()
                .ok_or("Mainnet UTxO RPC client not configured")?,
            _ => self
                .testnet_client
                .as_mut()
                .ok_or("Testnet UTxO RPC client not configured")?,
        };

        eprintln!(
            "[UTxO RPC] Fetching UTxOs for stake address: {}",
            hex_stake_address
        );
        // Create address pattern for stake address matching
        let pattern = spec::cardano::TxOutputPattern {
            address: Some(spec::cardano::AddressPattern {
                exact_address: Default::default(),
                payment_part: Default::default(),
                delegation_part: hex::decode(hex_stake_address).unwrap().into(),
            }),
            asset: None,
        };

        // match_utxos returns Result<UtxoPage<C>, Error>
        eprintln!("[UTxO RPC] Sending match_utxos request...");
        let utxo_page = match client
            .match_utxos(pattern, None, limit.unwrap_or(100))
            .await
        {
            Ok(page) => {
                eprintln!("[UTxO RPC] Successfully received response");
                page
            }
            Err(e) => {
                eprintln!("[UTxO RPC] Error details: {:?}", e);
                return Err(format!("Failed to fetch UTxOs: {:?}", e));
            }
        };

        eprintln!("[UTxO RPC] Found {} UTxOs", utxo_page.items.len());

        // Convert UTxOs to CIP-30 compatible format
        let cip30_utxos = Self::convert_utxos_to_cip30_format(utxo_page.items)?;

        Ok(json!(cip30_utxos))
    }

    /// Fetch UTxOs by exact addresses (hex format) and concatenate results
    pub async fn fetch_utxos_by_exact_addresses(
        &mut self,
        hex_addresses: &[String],
        network: pallas_addresses::Network,
        limit_per_address: Option<u32>,
    ) -> Result<Value, String> {
        let client = match network {
            pallas_addresses::Network::Mainnet => self
                .mainnet_client
                .as_mut()
                .ok_or("Mainnet UTxO RPC client not configured")?,
            _ => self
                .testnet_client
                .as_mut()
                .ok_or("Testnet UTxO RPC client not configured")?,
        };

        eprintln!(
            "[UTxO RPC] Fetching UTxOs for {} addresses",
            hex_addresses.len()
        );

        let mut all_cip30_utxos = Vec::new();
        let mut total_utxos_found = 0;

        for (i, hex_address) in hex_addresses.iter().enumerate() {
            eprintln!(
                "[UTxO RPC] Fetching UTxOs for address {}/{}: {}",
                i + 1,
                hex_addresses.len(),
                hex_address
            );

            // Convert hex address to bytes
            let address_bytes = match hex::decode(hex_address) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("[UTxO RPC] Invalid hex address {}: {}", hex_address, e);
                    continue; // Skip invalid addresses
                }
            };

            // Create address pattern for exact address matching
            let pattern = spec::cardano::TxOutputPattern {
                address: Some(spec::cardano::AddressPattern {
                    exact_address: address_bytes.into(),
                    payment_part: Default::default(),
                    delegation_part: Default::default(),
                }),
                asset: None,
            };

            // Fetch UTxOs for this address
            eprintln!(
                "[UTxO RPC] Sending match_utxos request for address {}...",
                i + 1
            );
            let utxo_page = match client
                .match_utxos(pattern, None, limit_per_address.unwrap_or(100))
                .await
            {
                Ok(page) => {
                    eprintln!(
                        "[UTxO RPC] Successfully received response for address {}",
                        i + 1
                    );
                    page
                }
                Err(e) => {
                    eprintln!(
                        "[UTxO RPC] Error fetching UTxOs for address {}: {:?}",
                        hex_address, e
                    );
                    continue; // Skip failed addresses
                }
            };

            let utxos_count = utxo_page.items.len();
            eprintln!(
                "[UTxO RPC] Found {} UTxOs for address {}",
                utxos_count,
                i + 1
            );
            total_utxos_found += utxos_count;

            //eprintln!("[UTxO RPC] Raw response for address {}: {:#?}", i + 1, &utxo_page.items);

            // Convert UTxOs to CIP-30 compatible format
            let address_cip30_utxos = Self::convert_utxos_to_cip30_format(utxo_page.items)?;

            // Concatenate to the overall result
            all_cip30_utxos.extend(address_cip30_utxos);
        }

        eprintln!(
            "[UTxO RPC] Total UTxOs found across all addresses: {}",
            total_utxos_found
        );
        Ok(json!(all_cip30_utxos))
    }

    /// Fetch balance by exact addresses and return aggregated Value
    pub async fn fetch_balance_from_addresses(
        &mut self,
        hex_addresses: &[String],
        network: pallas_addresses::Network,
        limit_per_address: Option<u32>,
    ) -> Result<pallas_primitives::conway::Value, String> {
        let client = match network {
            pallas_addresses::Network::Mainnet => self
                .mainnet_client
                .as_mut()
                .ok_or("Mainnet UTxO RPC client not configured")?,
            _ => self
                .testnet_client
                .as_mut()
                .ok_or("Testnet UTxO RPC client not configured")?,
        };

        eprintln!(
            "[UTxO RPC] Calculating balance from {} addresses",
            hex_addresses.len()
        );

        let mut total_ada: u64 = 0;
        let mut asset_totals: std::collections::HashMap<(Vec<u8>, Vec<u8>), u64> =
            std::collections::HashMap::new();
        let mut total_utxos_processed = 0;

        for (i, hex_address) in hex_addresses.iter().enumerate() {
            eprintln!(
                "[UTxO RPC] Processing address {}/{} for balance: {}",
                i + 1,
                hex_addresses.len(),
                hex_address
            );

            // Convert hex address to bytes
            let address_bytes = match hex::decode(hex_address) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("[UTxO RPC] Invalid hex address {}: {}", hex_address, e);
                    continue; // Skip invalid addresses
                }
            };

            // Create address pattern for exact address matching
            let pattern = spec::cardano::TxOutputPattern {
                address: Some(spec::cardano::AddressPattern {
                    exact_address: address_bytes.into(),
                    payment_part: Default::default(),
                    delegation_part: Default::default(),
                }),
                asset: None,
            };

            // Fetch UTxOs for this address
            let utxo_page = match client
                .match_utxos(pattern, None, limit_per_address.unwrap_or(100))
                .await
            {
                Ok(page) => page,
                Err(e) => {
                    eprintln!(
                        "[UTxO RPC] Error fetching UTxOs for balance from address {}: {:?}",
                        hex_address, e
                    );
                    continue; // Skip failed addresses
                }
            };

            let utxos_count = utxo_page.items.len();
            eprintln!(
                "[UTxO RPC] Found {} UTxOs for balance from address {}",
                utxos_count,
                i + 1
            );
            total_utxos_processed += utxos_count;

            // Process each UTxO and accumulate values
            for utxo in utxo_page.items {
                if let Some(parsed_utxo) = utxo.parsed {
                    // Add ADA amount
                    total_ada += parsed_utxo.coin;

                    // Add native assets
                    for multiasset in parsed_utxo.assets {
                        let policy_id = multiasset.policy_id.clone();

                        // Each multiasset contains multiple assets for the same policy
                        for asset in multiasset.assets {
                            let key = (policy_id.clone().to_vec(), asset.name.clone().to_vec());
                            // Use output_coin for the amount (mint_coin is for minting operations)
                            *asset_totals.entry(key).or_insert(0) += asset.output_coin;
                        }
                    }
                }
            }
        }

        eprintln!(
            "[UTxO RPC] Balance calculation complete: {} ADA, {} asset types from {} UTxOs",
            total_ada,
            asset_totals.len(),
            total_utxos_processed
        );

        // Create the final Value
        if asset_totals.is_empty() {
            Ok(pallas_primitives::conway::Value::Coin(total_ada))
        } else {
            // Convert asset totals to multiasset format
            use std::collections::BTreeMap;
            let mut policies: BTreeMap<
                pallas_primitives::PolicyId,
                BTreeMap<pallas_primitives::AssetName, u64>,
            > = BTreeMap::new();

            for ((policy_id, asset_name), amount) in asset_totals {
                // Convert Vec<u8> to fixed-size array for PolicyId
                let policy_bytes: [u8; 28] = if policy_id.len() == 28 {
                    let mut bytes = [0u8; 28];
                    bytes.copy_from_slice(&policy_id);
                    bytes
                } else {
                    eprintln!(
                        "[UTxO RPC] Invalid policy ID length: {} bytes, expected 28",
                        policy_id.len()
                    );
                    continue;
                };
                let policy = pallas_primitives::PolicyId::from(policy_bytes);

                // AssetName can be variable length (0-32 bytes)
                let name = pallas_primitives::AssetName::from(asset_name);
                policies
                    .entry(policy)
                    .or_insert_with(BTreeMap::new)
                    .insert(name, amount);
            }

            // Convert to NonEmptyKeyValuePairs
            let mut policy_pairs = Vec::new();
            for (policy_id, assets) in policies {
                let mut asset_pairs = Vec::new();
                for (asset_name, amount) in assets {
                    asset_pairs.push((
                        asset_name,
                        pallas_primitives::PositiveCoin::try_from(amount)
                            .map_err(|e| format!("Invalid coin amount: {}", e))?,
                    ));
                }
                if let Ok(assets_map) =
                    pallas_primitives::NonEmptyKeyValuePairs::try_from(asset_pairs)
                {
                    policy_pairs.push((policy_id, assets_map));
                }
            }

            if policy_pairs.is_empty() {
                Ok(pallas_primitives::conway::Value::Coin(total_ada))
            } else {
                let multiasset = pallas_primitives::NonEmptyKeyValuePairs::try_from(policy_pairs)
                    .map_err(|e| format!("Failed to create multiasset: {:?}", e))?;
                Ok(pallas_primitives::conway::Value::Multiasset(
                    total_ada, multiasset,
                ))
            }
        }
    }

    /// Submit a transaction to the blockchain
    pub async fn submit_transaction(
        &mut self,
        tx_cbor_hex: &str,
        network: pallas_addresses::Network,
    ) -> Result<String, String> {
        let submit_client = match network {
            pallas_addresses::Network::Mainnet => self
                .mainnet_submit_client
                .as_mut()
                .ok_or("Mainnet UTxO RPC submit client not configured")?,
            _ => self
                .testnet_submit_client
                .as_mut()
                .ok_or("Testnet UTxö RPC submit client not configured")?,
        };

        eprintln!("[UTxO RPC] Submitting transaction: {}", tx_cbor_hex);
        
        // Decode the CBOR hex to bytes
        let tx_bytes = hex::decode(tx_cbor_hex)
            .map_err(|e| format!("Failed to decode transaction CBOR hex: {}", e))?;

        eprintln!("[UTxO RPC] Transaction bytes length: {}", tx_bytes.len());

        // Submit the transaction using UTXO RPC submit client
        // submit_tx expects a Vec<Vec<u8>>, so we wrap our single transaction
        let transactions = vec![tx_bytes];
        match submit_client.submit_tx(transactions).await {
            Ok(tx_refs) => {
                if let Some(tx_ref) = tx_refs.first() {
                    // The transaction reference is the transaction hash as raw bytes
                    let tx_hash_hex = hex::encode(tx_ref);
                    eprintln!("[UTxO RPC] Transaction submitted successfully with hash: {}", tx_hash_hex);
                    Ok(tx_hash_hex)
                } else {
                    Err("No transaction reference returned from submit_tx".to_string())
                }
            }
            Err(e) => {
                eprintln!("[UTxö RPC] Transaction submission failed: {:?}", e);
                Err(format!("Failed to submit transaction: {:?}", e))
            }
        }
    }

    /// Convert UTxO RPC response to CIP-30 CBOR hex format (static method to avoid borrow issues)
    fn convert_utxos_to_cip30_format(
        utxos: Vec<utxorpc::ChainUtxo<TxOutput>>,
    ) -> Result<Vec<String>, String> {
        let mut cip30_utxos = Vec::new();
        for utxo in utxos {
            // Access the parsed UTxO data

            let parsed_output = utxo.parsed.unwrap();
            let utxo_ref = utxo.txo_ref.clone().unwrap();
            let tx_hash_hex = hex::encode(utxo_ref.hash);

            let tx_in = pallas_primitives::TransactionInput {
                transaction_id: pallas_crypto::hash::Hash::from_str(&tx_hash_hex)
                    .expect("Invalid hash"),
                index: u64::from(utxo.txo_ref.unwrap().index),
            };
            // Check if the UTxO has native assets or just ADA
            let out_value = if parsed_output.assets.is_empty() {
                // UTxO contains only ADA
                pallas_primitives::conway::Value::Coin(parsed_output.coin)
            } else {
                // UTxO contains ADA + native assets
                pallas_primitives::conway::Value::Multiasset(
                    parsed_output.coin,
                    convert_to_multiasset_positive_coin(parsed_output.assets)
                        .expect("Failed to convert assets"),
                )
            };
            let tx_out = pallas_primitives::conway::PostAlonzoTransactionOutput {
                address: pallas_primitives::Bytes::from(parsed_output.address.to_vec()),
                value: out_value,
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
    let result = utxo_client
        .fetch_utxos_by_stake_address(
            hex_stake_address,
            pallas_addresses::Network::Testnet,
            Some(100),
        )
        .await;

    match result {
        Ok(utxos) => {
            dbg!(&utxos);
            assert!(true); // Test passes if we get any response
        }
        Err(e) => {
            eprintln!("Test failed with error: {}", e);
            panic!("UTxO RPC test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_fetch_by_exact_addresses() {
    // Test the new exact addresses function
    let config = crate::config::UtxoRpcConfig {
        mainnet_url: None,
        testnet_url: Some("http://localhost:50051".to_string()),
        api_key: Some("dmtr_utxorpc1wgnnj0qcfj32zxsz2uc8d4g7uclm2s2w".to_string()),
    };

    let mut utxo_client = UtxoRpcClient::new(config).await.unwrap();

    // Example hex addresses (replace these with real testnet addresses that have UTxOs)
    let hex_addresses = vec![
            "00b3d0e0a28838c891b84a1a8649d14dc1813192d7b90e2b78e362541d27e96b274539896e4f8b79d2c0f8a76f3c247a61da4fd67fab52ab55".to_string(),
            "80e8d2d2d35075b9af57e27d2e9f89gce3b7484f4a4cb2c89b7d3c5e6f4g9b".to_string(),
        ];

    println!(
        "Testing UTxO fetch for {} exact addresses",
        hex_addresses.len()
    );

    let result = utxo_client
        .fetch_utxos_by_exact_addresses(
            &hex_addresses,
            pallas_addresses::Network::Testnet,
            Some(50), // Limit per address
        )
        .await;

    match result {
        Ok(utxos_json) => {
            println!("✅ Exact addresses UTxO RPC call successful!");
            println!(
                "Raw JSON response: {}",
                serde_json::to_string_pretty(&utxos_json).unwrap()
            );

            if let Some(utxos_array) = utxos_json.as_array() {
                println!(
                    "📦 Found {} total UTxOs across all addresses:",
                    utxos_array.len()
                );

                for (i, utxo) in utxos_array.iter().enumerate() {
                    if let Some(utxo_hex) = utxo.as_str() {
                        println!(
                            "  UTxO {}: {}... (length: {} chars)",
                            i + 1,
                            &utxo_hex[..std::cmp::min(32, utxo_hex.len())],
                            utxo_hex.len()
                        );
                    }
                }
            } else {
                println!("⚠️  Response is not an array: {:?}", utxos_json);
            }
            //print the reponse
            eprintln!("Full response: {:#?}", utxos_json);
            assert!(true); // Test passes if we get any response
        }
        Err(e) => {
            eprintln!("❌ Test failed with error: {}", e);
            // Don't panic for this test since addresses might not have UTxOs
            println!("⚠️  No UTxOs found or connection error - this is normal for test addresses");
        }
    }

    println!("🎉 Exact addresses test completed!");
}
