use crate::services;
use crate::state::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn get_home_run_info(_state: State<'_, AppState>, session: Value) -> Result<Value, String> {
    services::api_get_home_run_info(&session).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_run_record_detail(state: State<'_, AppState>, session: Value, record_id: String) -> Result<Value, String> {
    services::api_run_record_detail(&state, &session, &record_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sport_info(state: State<'_, AppState>, session: Value) -> Result<Value, String> {
    services::api_sport_info(&state, &session).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn record_track(session: Value, record_id: String) -> Result<Value, String> {
    services::api_record_track(&session, &record_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn have_yd_info(session: Value) -> Result<Value, String> {
    services::api_have_yd_info(&session).await.map_err(|e| e.to_string())
}
