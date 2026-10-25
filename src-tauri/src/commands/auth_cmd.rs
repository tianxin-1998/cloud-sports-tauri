use crate::services;
use crate::state::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn login(state: State<'_, AppState>, user: String, password: String, school_code: String) -> Result<Value, String> {
    services::api_login(&state, &user, &password, &school_code).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_session(session: Value) {
    services::save_session(&session);
}

#[tauri::command]
pub fn load_session_info() -> Value {
    services::load_session_info()
}

#[tauri::command]
pub fn has_cli_session() -> bool {
    services::has_cli_session()
}

#[tauri::command]
pub fn clear_cli_session() {
    services::clear_cli_session();
}

#[tauri::command]
pub fn session_accounts() -> Vec<Value> {
    services::list_accounts()
}

#[tauri::command]
pub fn session_get(user: String, school_code: String) -> Value {
    services::get_account(&user, &school_code)
}

#[tauri::command]
pub fn session_remove(user: String, school_code: String) {
    services::remove_account(&user, &school_code);
}
