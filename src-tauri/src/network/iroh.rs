
use frost_ed25519::Identifier;
use iroh::client::MemIroh as Iroh;

use anyhow::{anyhow, Result};
use iroh::base::node_addr;


use iroh::client::docs::LiveEvent;
use iroh::docs::{ContentStatus};
use tauri::Manager;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
pub struct Participant {
    pub node_id: String,
    pub identifier: Identifier,
    pub round1_public_key: Option<Vec<u8>>,
    pub round2_public_key: Option<Vec<u8>>,
    pub final_public_key: Option<Vec<u8>>,
}
impl Participant {
    pub fn new(id: &str) -> Self {
        Participant {
            node_id: id.to_string(),
            identifier: Identifier::derive(id.as_bytes()).unwrap(),
            round1_public_key: None,
            round2_public_key: None,
            final_public_key: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub cbor_hex: String,
    pub signatures: HashMap<String, Vec<u8>>, // Keyed by participant ID
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub body: String,
    pub signatures: HashMap<String, Vec<u8>>, // Keyed by participant ID
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Multisig {
    pub participant_ids: Vec<String>,
    pub participants: HashMap<String, Participant>, // Keyed by participant ID
    pub transactions: Vec<Transaction>,
    pub messages: Vec<Message>,
    pub public_key: Option<Vec<u8>>,
    pub address: Option<String>,
}

pub struct AppState {
    pub iroh: IrohNode,
    pub multisigs: Mutex<HashMap<String, Multisig>>, // Keyed by multisig ID
}

impl AppState {
    pub fn new(iroh: IrohNode) -> Self {
        AppState {
            iroh,
            multisigs: Mutex::new(HashMap::new()),
        }
    }

    pub fn iroh(&self) -> Iroh {
        self.iroh.client().clone()
    }

    pub async fn initialize_multisig(&self, multisig_id: String, participant_ids: Vec<String>) {
        let mut multisigs = self.multisigs.lock().await;
        multisigs.insert(multisig_id, Multisig {
            participant_ids,
            participants: HashMap::new(),
            transactions: Vec::new(),
            messages: Vec::new(),
            public_key: None,
            address: None,
        });
    }

    pub async fn add_participant(&self, multisig_id: String, participant: Participant) -> Result<(), String> {
        let mut multisigs = self.multisigs.lock().await;
        if let Some(multisig) = multisigs.get_mut(&multisig_id) {
            if multisig.participant_ids.contains(&participant.node_id) {
                multisig.participants.insert(participant.node_id.clone(), participant);
                Ok(())
            } else {
                Err("Participant ID is not valid for this multisig session".into())
            }
        } else {
            Err("Multisig session not found".into())
        }
    }

    pub async fn add_transaction(&self, multisig_id: String, transaction: Transaction) -> Result<(), String> {
        let mut multisigs = self.multisigs.lock().await;
        if let Some(multisig) = multisigs.get_mut(&multisig_id) {
            for participant_id in transaction.signatures.keys() {
                if !multisig.participant_ids.contains(participant_id) {
                    return Err("Invalid participant ID in transaction".into());
                }
            }
            multisig.transactions.push(transaction);
            Ok(())
        } else {
            Err("Multisig session not found".into())
        }
    }

    pub async fn add_message(&self, multisig_id: String, message: Message) -> Result<(), String> {
        let mut multisigs = self.multisigs.lock().await;
        if let Some(multisig) = multisigs.get_mut(&multisig_id) {
            for participant_id in message.signatures.keys() {
                if !multisig.participant_ids.contains(participant_id) {
                    return Err("Invalid participant ID in message".into());
                }
            }
            multisig.messages.push(message);
            Ok(())
        } else {
            Err("Multisig session not found".into())
        }
    }
}



// this example uses a persistend iroh node stored in the application data directory
type IrohNode = iroh::node::Node<iroh::blobs::store::fs::Store>;

// setup an iroh node
pub async fn setup<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> Result<()> {
    // get the applicaiton data root, join with "iroh_data" to get the data root for the iroh node
    let data_root = handle
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| anyhow!("can't get application data directory"))?
        .join("iroh_data");

    // create the iroh node
    let node = iroh::node::Node::persistent(data_root)
        .await?
        .spawn()
        .await?;
    let node_addr = node.my_addr().await.unwrap();
    println!("Iroh node started, listening on: {:?}, with id: {}", node_addr, node.node_id());
    handle.manage(AppState::new(node));

    Ok(())
}