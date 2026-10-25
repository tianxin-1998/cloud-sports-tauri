use crate::services;
use serde_json::Value;

#[tauri::command]
pub fn face_add(data_base64: String) -> Result<Value, String> {
    services::face_add(&data_base64)
}

#[tauri::command]
pub fn face_list() -> Value {
    services::face_list()
}

#[tauri::command]
pub fn face_get(name: String) -> Result<Value, String> {
    services::face_get(&name)
}

#[tauri::command]
pub fn face_delete(name: String) -> Result<(), String> {
    services::face_delete(&name)
}

#[tauri::command]
pub fn face_next() -> Result<Value, String> {
    services::face_next()
}
