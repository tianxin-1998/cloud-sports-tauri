use crate::services;
use serde_json::Value;

#[tauri::command]
pub fn track_status(school_code: String, ra_names: Vec<String>) -> Value {
    services::track_status(&school_code, &ra_names)
}

#[tauri::command]
pub fn track_query(school_code: String, ra_name: String) -> Value {
    services::track_query(&school_code, &ra_name)
}

#[tauri::command]
pub fn track_list_local(school_code: String) -> Value {
    services::track_list_local(&school_code)
}

#[tauri::command]
pub fn track_refine(points: Vec<[f64; 2]>) -> Result<Value, String> {
    services::track_refine(&points)
}

#[tauri::command]
pub fn validate(school_code: String, ra_name: String, route_type: String, points: Vec<[f64; 2]>) -> Result<Value, String> {
    services::validate(&school_code, &ra_name, &route_type, &points)
}

#[tauri::command]
pub fn upload(school_code: String, ra_name: String, route_type: String, points: Vec<[f64; 2]>, name: String) -> Result<Value, String> {
    services::upload(&school_code, &ra_name, &route_type, &points, &name)
}

#[tauri::command]
pub fn track_save_local(school_code: String, name: String, meta: Value, points: Vec<[f64; 2]>) -> Result<Value, String> {
    services::save_record_local(&school_code, &name, &meta, &points)
}
