use super::super::network::iroh::AppState;

#[tauri::command]
pub fn get_iroh_id(state: tauri::State<'_, AppState>) -> String {
    let state = state.iroh.node_id().to_string();
    state
}