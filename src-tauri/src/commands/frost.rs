use frost_ed25519::Error;


#[tauri::command]
// Creates a round 1 key package and returns the serialized public key package
pub async fn create_round1_key_package(
    max_signers: u16,
    min_signers: u16,
    participant_index: u16
) -> Vec<u8> {
    let round1_key_package = crate::utils::frost::create_round1_key_package(max_signers, min_signers, participant_index).unwrap();
    println!("Round1 Key Package: {:?}", round1_key_package);
    let key_package = round1_key_package.package.serialize().unwrap();
    key_package
}
