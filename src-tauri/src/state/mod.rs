use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Clone)]
pub struct SchoolInfo {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Serialize)]
pub struct TaskRow {
    pub task_id: String,
    pub user: String,
    pub status: String,
    pub result: Option<String>,
}

pub struct AppState {
    pub schools: Mutex<HashMap<String, SchoolInfo>>,
    pub cache_loaded: Mutex<bool>,
    pub tasks: Mutex<HashMap<String, TaskRow>>,
    pub running_users: Mutex<HashSet<String>>,
    pub pending_finishes: Mutex<HashMap<String, Value>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            schools: Mutex::new(HashMap::new()),
            cache_loaded: Mutex::new(false),
            tasks: Mutex::new(HashMap::new()),
            running_users: Mutex::new(HashSet::new()),
            pending_finishes: Mutex::new(HashMap::new()),
        }
    }
}

pub const FALLBACK_SCHOOLS: &[(&str, &str, &str)] =
    &[("106", "滁州学院", "https://tiyu.chzu.edu.cn:8010/m-api")];

impl AppState {
    pub fn insert_fallback(&self) {
        let mut s = self.schools.lock().unwrap();
        for (code, name, url) in FALLBACK_SCHOOLS {
            s.entry(code.to_string()).or_insert(SchoolInfo {
                name: name.to_string(),
                url: url.to_string(),
            });
        }
    }
}
