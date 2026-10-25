use crate::utils::faces_dir;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{json, Value};
use std::fs;
use std::sync::Mutex;

static FACE_IDX: Mutex<usize> = Mutex::new(0);

fn strip_data_uri(b64: &str) -> String {
    if b64.starts_with("data:") && b64.contains(',') {
        b64.splitn(2, ',').nth(1).unwrap_or(b64).to_string()
    } else {
        b64.to_string()
    }
}

fn is_img(name: &str) -> bool {
    let n = name.to_lowercase();
    n.ends_with(".jpg") || n.ends_with(".jpeg") || n.ends_with(".png")
}

fn face_files() -> Vec<String> {
    let dir = faces_dir();
    let mut v: Vec<String> = fs::read_dir(&dir)
        .into_iter()
        .flat_map(|rd| rd)
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| is_img(n))
        .collect();
    v.sort();
    v
}

fn safe_name(name: &str) -> Result<String, String> {
    std::path::Path::new(name)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .ok_or_else(|| "非法文件名".to_string())
}

pub fn face_add(data_base64: &str) -> Result<Value, String> {
    let bytes = B64.decode(strip_data_uri(data_base64).as_bytes()).map_err(|e| format!("base64 解码失败: {}", e))?;
    let dir = faces_dir();
    let mut n = face_files().len();
    let name = loop {
        let candidate = format!("face_{:03}.jpg", n);
        if !dir.join(&candidate).exists() {
            break candidate;
        }
        n += 1;
    };
    fs::write(dir.join(&name), &bytes).map_err(|e| format!("写入失败: {}", e))?;
    Ok(json!({ "name": name }))
}

pub fn face_list() -> Value {
    let photos = face_files().into_iter().map(|n| json!({ "name": n })).collect::<Vec<_>>();
    json!({ "photos": photos })
}

pub fn face_get(name: &str) -> Result<Value, String> {
    let safe = safe_name(name)?;
    let bytes = fs::read(faces_dir().join(&safe)).map_err(|_| "照片不存在".to_string())?;
    Ok(json!({ "data_base64": B64.encode(bytes) }))
}

pub fn face_delete(name: &str) -> Result<(), String> {
    let safe = safe_name(name)?;
    fs::remove_file(faces_dir().join(&safe)).map_err(|_| "删除失败".to_string())
}

pub fn face_next() -> Result<Value, String> {
    let files = face_files();
    if files.is_empty() {
        return Err("暂无人脸照片".into());
    }
    let mut idx = FACE_IDX.lock().unwrap();
    let i = *idx % files.len();
    *idx = idx.wrapping_add(1);
    let bytes = fs::read(faces_dir().join(&files[i])).map_err(|_| "读取照片失败".to_string())?;
    Ok(json!({ "data_base64": B64.encode(bytes), "name": files[i] }))
}
