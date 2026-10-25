use crate::models::AppConfig;
use crate::services;
use crate::state::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn get_schools(state: State<'_, AppState>) -> Result<Value, String> {
    if !*state.cache_loaded.lock().unwrap() {
        services::refresh_school_cache(state.inner())
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(services::get_all_school_names(&state))
}

#[tauri::command]
pub fn get_config() -> AppConfig {
    services::get_config()
}

#[tauri::command]
pub fn save_config(config: AppConfig) {
    services::save_config(&config);
}
