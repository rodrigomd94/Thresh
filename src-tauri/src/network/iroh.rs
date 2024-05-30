
use iroh::client::MemIroh as Iroh;

use anyhow::{anyhow, Result};
use iroh::base::node_addr;


use iroh::client::docs::LiveEvent;
use iroh::docs::{ContentStatus};
use tauri::Manager;

pub struct AppState {
    //todos: Mutex<Option<(Todos, tokio::task::JoinHandle<()>)>>,
    pub iroh: IrohNode,
}
impl AppState {
    pub fn new(iroh: IrohNode) -> Self {
        AppState {
           // todos: Mutex::new(None),
            iroh,
        }
    }

    pub fn iroh(&self) -> Iroh {
        self.iroh.client().clone()
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