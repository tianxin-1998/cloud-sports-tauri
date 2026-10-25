use crate::error::AppError;
use crate::services::upstream_service::{
    field_str, gen_device_id, get_home_run_info, get_student_info, have_device_name, is_valid_token,
    login,
};
use crate::state::{AppState, SchoolInfo};
use crate::utils::protocol::list_schools;
use rand::Rng;
use serde_json::{json, Value};

pub async fn refresh_school_cache(state: &AppState) -> Result<(), AppError> {
    let uuid = gen_device_id();
    let device = have_device_name();
    let online = list_schools(&uuid, &device).await;
    let mut map = state.schools.lock().unwrap();
    for s in &online {
        let code = s.get("schoolCode").and_then(|x| x.as_str()).unwrap_or("");
        let url = s.get("schoolUrl").and_then(|x| x.as_str()).unwrap_or("");
        if !code.is_empty() && !url.is_empty() {
            map.insert(
                code.to_string(),
                SchoolInfo {
                    name: s.get("schoolName").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                    url: url.to_string(),
                },
            );
        }
    }
    drop(map);
    state.insert_fallback();
    *state.cache_loaded.lock().unwrap() = true;
    Ok(())
}

pub fn get_school_info(state: &AppState, code: &str) -> Option<SchoolInfo> {
    state.schools.lock().unwrap().get(code).cloned()
}

pub fn get_all_school_names(state: &AppState) -> Value {
    let s = state.schools.lock().unwrap();
    let mut v: Vec<Value> = s
        .iter()
        .map(|(code, info)| json!({ "schoolName": info.name, "schoolCode": code }))
        .collect();
    v.sort_by(|a, b| {
        a.get("schoolCode").unwrap_or(&Value::Null).as_str().cmp(&b.get("schoolCode").unwrap_or(&Value::Null).as_str())
    });
    json!({ "schools": v })
}

pub async fn api_login(state: &AppState, user: &str, password: &str, school_code: &str) -> Result<Value, AppError> {
    let info = match get_school_info(state, school_code) {
        Some(i) => i,
        None => {
            if !*state.cache_loaded.lock().unwrap() {
                refresh_school_cache(state).await.ok();
                match get_school_info(state, school_code) {
                    Some(i) => i,
                    None => return Err(AppError::from(format!("学校编码 {} 不存在", school_code))),
                }
            } else {
                return Err(AppError::from(format!("学校编码 {} 不存在", school_code)));
            }
        }
    };
    let school_url = info.url;
    let uuid = gen_device_id();
    let device = have_device_name();
    let token = login(user, password, &uuid, &device, &school_url, school_code).await?;
    if !is_valid_token(&token) {
        return Err(AppError::from("用户名或密码错误"));
    }
    let student = get_student_info(&school_url, &uuid, &device, &token).await;
    let mut result = student;
    result["user"] = json!(user);
    result["school_code"] = json!(school_code);
    result["school_url"] = json!(school_url);
    result["uuid"] = json!(uuid);
    result["device_name"] = json!(device);
    result["token"] = json!(token);
    let run_areas = get_home_run_info(&school_url, &uuid, &device, &token).await.unwrap_or_else(|_| Vec::new());
    result["run_areas"] = json!(run_areas);
    Ok(result)
}

pub async fn api_get_home_run_info(session: &Value) -> Result<Value, AppError> {
    let school_url = field_str(session, "school_url");
    let uuid = field_str(session, "uuid");
    let device = field_str(session, "device_name");
    let token = field_str(session, "token");
    if school_url.is_empty() || uuid.is_empty() || token.is_empty() {
        return Err(AppError::from("会话信息不完整，请重新登录"));
    }
    let areas = get_home_run_info(&school_url, &uuid, &device, &token).await?;
    Ok(json!(areas))
}

pub struct YunRunKwargs {
    pub school_url: String,
    pub school_code: String,
    pub uuid: String,
    pub device_name: String,
    pub token: String,
    pub target_distance: i64,
    pub user: String,
    pub ra_name: String,
    pub pace_range: (f64, f64),
    pub cadence_range: (f64, f64),
    pub fast: bool,
    pub face_base64: Option<String>,
}

pub async fn build_yun_run_kwargs(
    state: &AppState,
    session: &Value,
    target_distance: i64,
    ra_name: &str,
    fast: bool,
    face_base64: Option<String>,
) -> Result<YunRunKwargs, AppError> {
    if target_distance > 4500 {
        return Err(AppError::from("target_distance 不能超过 4500"));
    }

    let actual = {
        let mut rng = rand::rng();
        (target_distance + rng.random_range(0..=150)).min(4500)
    };
    let school_code = field_str(session, "school_code");
    let uuid = field_str(session, "uuid");
    let device = field_str(session, "device_name");
    let token = field_str(session, "token");
    let user = field_str(session, "user");
    let school_url = match get_school_info(state, &school_code) {
        Some(i) => i.url,
        None => {
            if !*state.cache_loaded.lock().unwrap() {
                refresh_school_cache(state).await.ok();
            }
            get_school_info(state, &school_code)
                .ok_or_else(|| AppError::from(format!("学校编码 {} 不存在", school_code)))?
                .url
        }
    };

    let cfg = crate::services::config_service::load_config();
    let pace_min = if cfg.pace_min > 0.0 { cfg.pace_min } else { 5.5 };
    let pace_max = if cfg.pace_max > pace_min { cfg.pace_max } else { pace_min + 1.0 };
    let cad_min = if cfg.cadence_min > 0.0 { cfg.cadence_min } else { 164.0 };
    let cad_max = if cfg.cadence_max > cad_min { cfg.cadence_max } else { cad_min + 1.0 };
    Ok(YunRunKwargs {
        school_url,
        school_code,
        uuid,
        device_name: device,
        token,
        target_distance: actual,
        user,
        ra_name: ra_name.to_string(),
        pace_range: (pace_min, pace_max),
        cadence_range: (cad_min, cad_max),
        fast,
        face_base64,
    })
}
