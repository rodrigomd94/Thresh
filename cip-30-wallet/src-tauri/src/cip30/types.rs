use pallas_primitives::alonzo::TransactionInput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Paginate {
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSignature {
    pub signature: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extension {
    pub cip: u32,
}
