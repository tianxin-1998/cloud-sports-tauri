use crate::services;
use crate::state::AppState;
use serde_json::Value;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn run(
    app: AppHandle,
    state: State<'_, AppState>,
    session: Value,
    target_distance: i64,
    ra_name: String,
    fast: bool,
    face_base64: Option<String>,
    hold_finish: Option<bool>,
    points: Option<Vec<Value>>,
    submit_distance: Option<f64>,
) -> Result<Value, String> {
    services::api_yun_run(
        state.inner(),
        &app,
        &session,
        target_distance,
        &ra_name,
        fast,
        face_base64,
        hold_finish.unwrap_or(false),
        points,
        submit_distance,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn finish_run(state: State<'_, AppState>, record_id: String) -> Result<Value, String> {
    services::api_finish_run(state.inner(), &record_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn discard_run(state: State<'_, AppState>, record_id: String) -> Result<Value, String> {
    services::api_discard_run(state.inner(), &record_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn simulate(session: Value, target_distance: i64, ra_name: String, fast: bool, points: Option<Vec<Value>>) -> Result<Value, String> {
    services::api_simulate(&session, target_distance, &ra_name, fast, points).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_task(state: State<'_, AppState>, task_id: String) -> Result<Value, String> {
    services::get_task(&state, &task_id).ok_or_else(|| "任务不存在".into())
}
