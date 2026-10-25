use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub fast_run: bool,
    pub auto_login: bool,
    pub default_school_code: String,
    pub theme: String,
    pub amap_key: String,
    pub amap_security: String,
    pub pace_min: f64,
    pub pace_max: f64,
    pub cadence_min: f64,
    pub cadence_max: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            fast_run: false,
            auto_login: false,
            default_school_code: String::new(),
            theme: "light".into(),
            amap_key: String::new(),
            amap_security: String::new(),
            pace_min: 5.5,
            pace_max: 6.7,
            cadence_min: 164.0,
            cadence_max: 176.0,
        }
    }
}
