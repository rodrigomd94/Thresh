use pallas_primitives::{
    conway::Value, AssetName, Hash, NonEmptyKeyValuePairs, PolicyId, PositiveCoin,
};
use std::collections::BTreeMap;
use std::str::FromStr;

pub fn convert_to_multiasset_positive_coin(
    out_assets: Vec<utxorpc::spec::cardano::Multiasset>,
) -> Result<pallas_primitives::conway::Multiasset<PositiveCoin>, ConversionError> {
    let btree_result: BTreeMap<Hash<28>, BTreeMap<AssetName, PositiveCoin>> = out_assets
        .into_iter()
        .fold(BTreeMap::new(), |mut acc, multiasset| {
            let policy_hex = hex::encode(multiasset.policy_id);
            let policy_id = pallas_crypto::hash::Hash::<28>::from_str(&policy_hex)
                .expect("Invalid policy ID bytes");

            let asset_map: BTreeMap<AssetName, PositiveCoin> = multiasset
                .assets
                .into_iter()
                .filter_map(|asset| {
                    let asset_name =
                        AssetName::from(pallas_primitives::Bytes::from(asset.name.to_vec()));
                    let coin_value = asset.output_coin;

                    // Skip zero values by returning None
                    PositiveCoin::try_from(coin_value)
                        .ok()
                        .map(|positive_coin| (asset_name, positive_coin))
                })
                .collect();

            // Merge assets under the same policy_id
            acc.entry(policy_id)
                .and_modify(|existing_map| {
                    for (name, coin) in &asset_map {
                        existing_map
                            .entry(name.clone())
                            .and_modify(|existing| {
                                // Extract u64 values, add them, and create new PositiveCoin
                                let existing_value: u64 = (*existing).into();
                                let coin_value: u64 = (*coin).into();
                                let new_value = existing_value + coin_value;
                                *existing = PositiveCoin::try_from(new_value)
                                    .expect("Combined value should be positive");
                            })
                            .or_insert(*coin);
                    }
                })
                .or_insert(asset_map);

            acc
        });

    // Convert BTreeMap to NonEmptyKeyValuePairs
    if btree_result.is_empty() {
        return Err(ConversionError::EmptyAssets);
    }

    let policy_pairs: Vec<(Hash<28>, NonEmptyKeyValuePairs<AssetName, PositiveCoin>)> =
        btree_result
            .into_iter()
            .filter_map(|(policy_id, asset_map)| {
                if asset_map.is_empty() {
                    return None;
                }

                let asset_pairs: Vec<(AssetName, PositiveCoin)> = asset_map.into_iter().collect();

                match NonEmptyKeyValuePairs::try_from(asset_pairs) {
                    Ok(inner_pairs) => Some((policy_id, inner_pairs)),
                    Err(_) => None,
                }
            })
            .collect();

    NonEmptyKeyValuePairs::try_from(policy_pairs).map_err(|_| ConversionError::EmptyAssets)
}

#[derive(Debug)]
pub enum ConversionError {
    EmptyAssets,
}
