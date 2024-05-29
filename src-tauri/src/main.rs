use std::{fs, path::PathBuf, sync::Arc};

use iroh::base::node_addr;
use tauri::{
    ipc::RemoteDomainAccessScope, CustomMenuItem, Manager, Menu, MenuItem, Submenu, WindowBuilder,
    WindowUrl,
};
mod utils;
mod commands;
use frost_tauri::commands::frost::create_round1_key_package;
use frost_tauri::commands::cip30::{
    get_change_address, get_collateral, sign_data, get_network_id, get_reward_address, get_unused_addresses,
    get_used_addresses, get_utxos, sign_tx, submit_tx,
};

use iroh::client::MemIroh as Iroh;

use anyhow::{anyhow, Result};

use iroh::client::docs::LiveEvent;
use iroh::docs::{ContentStatus};

// this example uses a persistend iroh node stored in the application data directory
type IrohNode = iroh::node::Node<iroh::blobs::store::fs::Store>;

// setup an iroh node
async fn setup<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> Result<()> {
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

struct AppState {
    //todos: Mutex<Option<(Todos, tokio::task::JoinHandle<()>)>>,
    iroh: IrohNode,
}
impl AppState {
    fn new(iroh: IrohNode) -> Self {
        AppState {
           // todos: Mutex::new(None),
            iroh,
        }
    }

    fn iroh(&self) -> Iroh {
        self.iroh.client().clone()
    }
}

#[tauri::command]
fn get_iroh_id(state: tauri::State<'_, AppState>) -> String {
    let state = state.iroh.node_id().to_string();
    state
}

#[tauri::command]
fn inject_script(app: tauri::AppHandle, window: String, script: String) {
    let selected_window = app.get_window(window.as_str()).unwrap();
    selected_window
        .eval(&script.as_str())
        .expect("Failed to inject script");
    // selected_window.open_devtools();
}

#[tauri::command]
fn init_wallet_api(app: tauri::AppHandle, window: String) {
    // Path to the bundled script
    let script_path = PathBuf::from("../dist/InjectScript.es.js ");
    // Read the script content
    let script_content =
        fs::read_to_string(script_path).expect("Failed to read the bundled script");
    let selected_window = app.get_window(window.as_str()).unwrap();
    selected_window
        .eval(&script_content.as_str())
        .expect("Failed to inject script");
    selected_window.open_devtools();

    let main_window = app.get_window("main").unwrap();
    main_window
        .eval(&script_content.as_str())
        .expect("Failed to inject script");
}

#[tauri::command]
fn open_browser(app_handle: tauri::AppHandle, url: String) {
    WindowBuilder::new(
        &app_handle,
        "webview",
        WindowUrl::External(url.parse().unwrap()),
    )
    .title("Embedded Browser")
    .build()
    .expect("Failed to build new window");
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
        open_browser,
        inject_script,
        init_wallet_api,
        sign_data,
        sign_tx,
        submit_tx,
        get_utxos,
        get_collateral,
        get_used_addresses,
        get_unused_addresses,
        get_change_address,
        get_reward_address,
        get_network_id,
        create_round1_key_package,
        get_iroh_id,
        ])
        .setup(|app| {
            let handle = app.handle();
            tauri::async_runtime::spawn(async move {
                println!("starting backend...");
                if let Err(err) = setup(handle).await {
                    eprintln!("failed: {:?}", err);
                }
            });
            /* app.ipc_scope().configure_remote_access(
                RemoteDomainAccessScope::new("www.app.minswap.org")
                    .add_window("webview")
                    .enable_tauri_api()
                    .add_plugin("event")
                    .add_plugin("path"),
            ); */
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
