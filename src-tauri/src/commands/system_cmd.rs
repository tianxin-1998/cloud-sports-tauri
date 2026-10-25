use serde_json::{json, Value};
use tauri::AppHandle;

#[tauri::command]
pub fn ping() -> String {
    "pong".into()
}

#[tauri::command]
pub fn status() -> Value {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    json!({ "server_time": now })
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    webbrowser::open(&url).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upload_run_face_info() -> Result<String, String> {
    Ok("阶段四：人脸录入暂未实现".into())
}

#[tauri::command]
pub async fn get_run_face_status() -> Result<Value, String> {
    Ok(Value::Null)
}

#[tauri::command]
pub async fn run_face_info_comparison() -> Result<Value, String> {
    Ok(Value::Null)
}
