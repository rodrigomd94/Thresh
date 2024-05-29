use frost::{
    keys::{
        dkg::{round1, round2},
        KeyPackage, PublicKeyPackage, SecretShare,
    },
    round1::{SigningCommitments, SigningNonces},
    round2::SignatureShare,
    Ed25519Sha512, Identifier, Signature, SigningPackage,
};
// ANCHOR: dkg_import
use frost_ed25519 as frost;
use pallas_addresses::{self, Network, ShelleyAddress, ShelleyDelegationPart, ShelleyPaymentPart};
use pallas_crypto::{
    hash::{self, Hash, Hasher},
    key::ed25519::{self as pallas, PublicKey},
};
use rand::{rngs::ThreadRng, thread_rng};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Round1KeyPackage {
    pub secret_package: round1::SecretPackage,
    pub package: round1::Package,
}

#[derive(Debug)]
pub struct Phase2KeyPackage {
    pub secret_package: round2::SecretPackage,
    pub packages: BTreeMap<Identifier, round2::Package>,
}
#[derive(Debug)]
pub struct Distributedround1Packages {
    pub round1_secret_packages: BTreeMap<Identifier, round1::SecretPackage>,
    pub received_round1_packages:  BTreeMap<Identifier, round1::Package>,
}

pub fn create_round1_key_package(
    max_signers: u16,
    min_signers: u16,
    participant_index: u16, // mut rng: ThreadRng,
) -> Option<Round1KeyPackage> {
    let mut rng = thread_rng();
    let participant_identifier = participant_index.try_into().expect("should be nonzero");
    // ANCHOR: dkg_part1
    let part1_result =
        frost::keys::dkg::part1(participant_identifier, max_signers, min_signers, &mut rng);
        
    let (secret_package, package) = match part1_result {
        Ok((round1_secret_package, round1_package)) => (round1_secret_package, round1_package),
        Err(e) => {
            eprintln!("Error: {}", e);
            return None;
        }
    };
    Some(Round1KeyPackage {
        secret_package,
        package,
    })
}

pub fn distribute_round1_packages(
    round1_key_package: &round1::Package,
    participant_index: u16,
    max_signers: u16,
) -> Distributedround1Packages {
    let mut received_round1_packages = BTreeMap::new();
    let participant_identifier:Identifier = participant_index.try_into().expect("should be nonzero");
    let round1_secret_packages = BTreeMap::new();
    let round1_package = round1_key_package;
    for receiver_participant_index in 1..=max_signers {
        if receiver_participant_index == participant_index {
            continue;
        }
        let receiver_participant_identifier: frost::Identifier = receiver_participant_index
            .try_into()
            .expect("should be nonzero");
        // received_round1_packages
        //     .entry(receiver_participant_identifier)
        //     .or_insert_with(BTreeMap::new)
        //     .insert(participant_identifier, round1_package.clone());
        received_round1_packages.insert(receiver_participant_identifier, round1_package.clone());
    }
    Distributedround1Packages {
        round1_secret_packages,
        received_round1_packages,
    }
}

pub fn create_round2_package(
    round1_secret_package: round1::SecretPackage,
    round1_packages: &BTreeMap<Identifier, round1::Package>,
) -> Option<Phase2KeyPackage> {
    let round2_result = frost::keys::dkg::part2(round1_secret_package, &round1_packages);
    let (round2_secret_package, round2_packages) = match round2_result {
        Ok((round2_secret_package, round2_packages)) => (round2_secret_package, round2_packages),
        Err(e) => {
            eprintln!("Error: {}", e);
            return None;
        }
    };
    Some(Phase2KeyPackage {
        secret_package: round2_secret_package,
        packages: round2_packages,
    })
}

pub fn distribute_phase2_packages(
    participant_index: u16,
    round2_packages: BTreeMap<Identifier, round2::Package>,
) -> BTreeMap<Identifier, BTreeMap<Identifier, round2::Package>> {
    let participant_identifier: Identifier =
        participant_index.try_into().expect("should be nonzero");
    let mut received_round2_packages = BTreeMap::new();
    for (receiver_identifier, round2_package) in round2_packages {
        received_round2_packages
            .entry(receiver_identifier)
            .or_insert_with(BTreeMap::new)
            .insert(participant_identifier, round2_package);
    }
    received_round2_packages
}

pub fn generate_user_keys(
    participant_index: u16,
    received_round1_packages: BTreeMap<Identifier, BTreeMap<Identifier, round1::Package>>,
    received_round2_packages: BTreeMap<Identifier, BTreeMap<Identifier, round2::Package>>,
    round2_secret_package: round2::SecretPackage,
) -> Option<(KeyPackage, PublicKeyPackage)> {
    let participant_identifier = participant_index.try_into().expect("should be nonzero");
    let round1_packages = &received_round1_packages[&participant_identifier];
    let round2_packages = &received_round2_packages[&participant_identifier];
    // ANCHOR: dkg_part3
    let part3_result =
        frost::keys::dkg::part3(&round2_secret_package, round1_packages, round2_packages);
    let (key_package, pubkey_package) = match (part3_result) {
        Ok((key_package, pubkey_package)) => (key_package, pubkey_package),
        Err(e) => {
            eprintln!("Error: {}", e);
            return None;
        }
    };
    Some((key_package, pubkey_package))
}

pub fn generate_nonces_and_commitments(
    key_package: KeyPackage,
) -> (SigningNonces, SigningCommitments) {
    let mut rng = thread_rng();
    // Generate one (1) nonce and one SigningCommitments instance for each
    // participant, up to _threshold_.
    let (nonces, commitments) = frost::round1::commit(key_package.signing_share(), &mut rng);
    (nonces, commitments)
}

pub fn generate_signing_package(
    commitments_map: BTreeMap<Identifier, SigningCommitments>,
    message: &[u8],
) -> SigningPackage {
    let signing_package = frost::SigningPackage::new(commitments_map, message);
    signing_package
}

pub fn generate_signature_share(
    key_package: &KeyPackage,
    nonces: SigningNonces,
    signing_package: SigningPackage,
) -> Option<SignatureShare> {
    // Each participant generates their signature share.
    let sig_share_result = frost::round2::sign(&signing_package, &nonces, &key_package);
    let signature_share = match sig_share_result {
        Ok(signature_share) => signature_share,
        Err(e) => {
            eprintln!("Error: {}", e);
            return None;
        }
    };
    Some(signature_share)
}

pub fn generate_group_signature(
    signing_package: SigningPackage,
    signature_shares: BTreeMap<Identifier, SignatureShare>,
    pubkey_package: PublicKeyPackage,
) -> Option<pallas::Signature> {
    let sig_result = frost::aggregate(&signing_package, &signature_shares, &pubkey_package);

    let group_signature = match sig_result {
        Ok(group_signature) => group_signature,
        Err(e) => {
            eprintln!("Error: {}", e);
            return None;
        }
    };
    let sig_bytes: [u8; 64] = group_signature.serialize();
    let pallas_signature = pallas::Signature::from(sig_bytes);
    Some(pallas_signature)
}

pub fn get_group_pubkey(pubkey_package: PublicKeyPackage) -> PublicKey {
    let frost_public_key =  &pubkey_package.verifying_key();
    let frost_pubkey_bytes = frost_public_key.serialize();
    let pallas_pubkey = pallas::PublicKey::from(frost_pubkey_bytes);
    pallas_pubkey
}

mod tests_round1 {
    use super::*;

    #[test]
    fn test_create_round1_key_package() {
        let max_signers = 5;
        let min_signers = 3;
        // let participant_index = 1;

        
        // assert!(key_package.is_some());
    }
}

// -----------------------------------

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
