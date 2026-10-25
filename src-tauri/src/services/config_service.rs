use crate::models::AppConfig;
use crate::utils::data_dir;
use std::fs;

const CONFIG_FILE: &str = "config.json";

fn config_path() -> std::path::PathBuf {
    data_dir().join(CONFIG_FILE)
}

pub fn load_config() -> AppConfig {
    let p = config_path();
    if p.exists() {
        serde_json::from_str(&fs::read_to_string(p).unwrap_or_default()).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

pub fn save_config(c: &AppConfig) {
    let _ = fs::write(config_path(), serde_json::to_string_pretty(c).unwrap_or_default());
}
