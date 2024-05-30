use frost_ed25519::Error;


#[tauri::command]
// Creates a round 1 key package and returns the serialized public key package
pub async fn create_round1_key_package(
    max_signers: u16,
    min_signers: u16,
    iroh_node_id: String
) -> Vec<u8> {
    let round1_key_package = crate::utils::frost::create_round1_key_package(max_signers, min_signers, iroh_node_id).unwrap();
    println!("Round1 Key Package: {:?}", round1_key_package);
    let key_package = round1_key_package.package.serialize().unwrap();
    key_package
}
