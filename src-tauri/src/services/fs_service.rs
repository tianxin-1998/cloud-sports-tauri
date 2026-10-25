use crate::utils::data_dir;
use serde_json::Value;
use std::fs;

const SESSION_FILE: &str = "cli_session.json";

const ACCOUNTS_FILE: &str = "accounts.json";

fn session_path() -> std::path::PathBuf {
    data_dir().join(SESSION_FILE)
}

fn accounts_path() -> std::path::PathBuf {
    data_dir().join(ACCOUNTS_FILE)
}

pub fn has_cli_session() -> bool {
    session_path().exists()
}

pub fn load_session_info() -> Value {
    let p = session_path();
    if p.exists() {
        serde_json::from_str(&fs::read_to_string(p).unwrap_or_default()).unwrap_or(Value::Null)
    } else {
        Value::Null
    }
}

pub fn save_session(v: &Value) {
    let _ = fs::write(session_path(), serde_json::to_string_pretty(v).unwrap_or_default());

    upsert_account(v);
}

pub fn clear_cli_session() {
    let _ = fs::remove_file(session_path());
}

fn read_accounts() -> Vec<Value> {
    let p = accounts_path();
    if p.exists() {
        serde_json::from_str(&fs::read_to_string(p).unwrap_or_default()).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn write_accounts(list: &[Value]) {
    let _ = fs::write(accounts_path(), serde_json::to_string_pretty(list).unwrap_or_default());
}

fn account_key(v: &Value) -> (String, String) {
    (
        v.get("user").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        v.get("school_code").and_then(|x| x.as_str()).unwrap_or("").to_string(),
    )
}

pub fn upsert_account(v: &Value) {
    let key = account_key(v);
    if key.0.is_empty() || key.1.is_empty() {
        return;
    }
    let mut list: Vec<Value> = read_accounts()
        .into_iter()
        .filter(|a| account_key(a) != key)
        .collect();
    list.insert(0, v.clone());
    write_accounts(&list);
}

pub fn list_accounts() -> Vec<Value> {
    read_accounts()
}

pub fn get_account(user: &str, school_code: &str) -> Value {
    read_accounts()
        .into_iter()
        .find(|a| account_key(a) == (user.to_string(), school_code.to_string()))
        .unwrap_or(Value::Null)
}

pub fn remove_account(user: &str, school_code: &str) {
    let list: Vec<Value> = read_accounts()
        .into_iter()
        .filter(|a| account_key(a) != (user.to_string(), school_code.to_string()))
        .collect();
    write_accounts(&list);
}
