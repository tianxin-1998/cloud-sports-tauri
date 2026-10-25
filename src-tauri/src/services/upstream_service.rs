use crate::error::AppError;
use crate::utils::crypto::gzip_bytes;
use crate::utils::protocol::{endpoint, post_bytes, post_empty, post_json};
use rand::Rng;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const DEVICE_MODELS: &[&str] = &[
    "vivo(V2366GA)", "vivo(V2415A)", "vivo(V2509A)",
    "Xiaomi(24129PN74C)", "Xiaomi(25019PNF3C)", "Xiaomi(25113PN0EC)",
    "Redmi(2407FRK8EC)", "Redmi(2510DRK44C)",
    "OPPO(PKZ110)", "iQOO(V2307A)",
];

pub fn gen_device_id() -> String {
    let chars = "0123456789abcdef";
    loop {
        let android_id: String = (0..16)
            .map(|_| chars.chars().nth(rand::rng().random_range(0..16)).unwrap())
            .collect();
        if android_id != "9774d56d682e549c" {
            return format!("{:x}", Sha256::digest(android_id.as_bytes()));
        }
    }
}

pub fn have_device_name() -> String {
    DEVICE_MODELS[rand::rng().random_range(0..DEVICE_MODELS.len())].to_string()
}

pub fn is_valid_token(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    let lens = [8usize, 4, 4, 4, 12];
    for (p, l) in parts.iter().zip(lens.iter()) {
        if p.len() != *l {
            return false;
        }
        if !p.chars().all(|c| c.is_ascii_hexdigit()) {
            return false;
        }
    }
    true
}

pub fn is_auth_failure(v: &Value) -> bool {
    if v.get("code").and_then(|c| c.as_i64()) == Some(401) {
        return true;
    }
    v.get("msg").and_then(|m| m.as_str()).map(|s| s.contains("认证失败")).unwrap_or(false)
}

pub fn auth_error_of(v: &Value) -> AppError {
    let msg = v.get("msg").and_then(|m| m.as_str()).unwrap_or("认证已失效，请重新登录").to_string();
    AppError::from(format!("AUTH_EXPIRED:{}", msg))
}

pub fn resp_result(content: Option<Value>, fallback: &str) -> Result<Value, AppError> {
    match content {
        Some(v) if is_auth_failure(&v) => Err(auth_error_of(&v)),
        Some(v) => Ok(v),
        None => Err(AppError::from(fallback.to_string())),
    }
}

pub(crate) fn field_str(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

fn get_login_paths(school_code: &str, preferred: &str) -> Vec<String> {
    let mut paths: Vec<String> = match school_code {
        "100" => vec!["/login/appLoginHGD".into(), "/login/appLogin".into()],
        "106" => vec!["/login/appLoginCHZU".into(), "/login/appLogin".into()],
        _ => vec!["/login/appLogin".into()],
    };
    if !preferred.is_empty() {
        if let Some(pos) = paths.iter().position(|p| p == preferred) {
            paths.remove(pos);
            paths.insert(0, preferred.to_string());
        }
    }
    paths
}

pub async fn login(
    user: &str,
    password: &str,
    uuid: &str,
    device_name: &str,
    school_url: &str,
    school_code: &str,
) -> Result<String, AppError> {
    let data = json!({ "password": password, "schoolId": school_code, "userName": user, "type": "1" });
    let paths = get_login_paths(school_code, "");
    let mut last_msg = "登录失败".to_string();
    eprintln!("[login] school_code={} school_url={} user={}", school_code, school_url, user);
    for path in &paths {
        let url = endpoint(school_url, path);
        eprintln!("[login] 尝试端点 {}", url);
        if let Some(content) = post_json(&url, &data, uuid, device_name, "").await {
            if let Some(token) = content.get("data").and_then(|d| d.get("token")).and_then(|t| t.as_str()) {
                eprintln!("[login] 获取 token 成功");
                return Ok(token.to_string());
            }
            eprintln!("[login] {} 未取得 token，content={}", url, content);
            if let Some(msg) = content.get("msg").and_then(|m| m.as_str()) {
                last_msg = msg.to_string();
            }
        } else {
            eprintln!("[login] {} 无有效响应", url);
        }
    }
    eprintln!("[login] 失败：{}", last_msg);
    Err(AppError::from(last_msg))
}

pub async fn get_student_info(
    school_url: &str,
    uuid: &str,
    device_name: &str,
    token: &str,
) -> Value {
    let url = endpoint(school_url, "/login/getStudentInfo");
    if let Some(content) = post_empty(&url, uuid, device_name, token).await {
        if content.get("code").and_then(|c| c.as_i64()) == Some(200) {
            let d = content.get("data").unwrap_or(&Value::Null);
            let sex = match d.get("sex").and_then(|x| x.as_i64()) {
                Some(1) => "男",
                Some(2) => "女",
                _ => "未知",
            };
            return json!({
                "nickName": d.get("nickName").and_then(|x| x.as_str()).unwrap_or(""),
                "sex": sex,
                "realName": d.get("realName").and_then(|x| x.as_str()).unwrap_or(""),
                "studentId": d.get("userName").and_then(|x| x.as_str()).unwrap_or(""),
                "school": d.get("schoolName").and_then(|x| x.as_str()).unwrap_or(""),
                "campus": d.get("campusName").and_then(|x| x.as_str()).unwrap_or(""),
                "faculty": d.get("facName").and_then(|x| x.as_str()).unwrap_or(""),
                "major": d.get("pfsName").and_then(|x| x.as_str()).unwrap_or(""),
                "className": d.get("className").and_then(|x| x.as_str()).unwrap_or(""),
            });
        }
    }
    Value::Null
}

pub async fn get_home_run_info(
    school_url: &str,
    uuid: &str,
    device_name: &str,
    token: &str,
) -> Result<Vec<Value>, AppError> {
    let url = endpoint(school_url, "/run/getHomeRunInfo");
    let v = resp_result(post_empty(&url, uuid, device_name, token).await, "获取跑区失败（网络异常）")?;
    let rdata = v.get("data").ok_or_else(|| AppError::from("获取跑区失败"))?;
    let cra = rdata.get("cralist").and_then(|c| c.as_array()).ok_or_else(|| AppError::from("该账号暂无可用跑区"))?;
    if cra.is_empty() {
        return Err(AppError::from("该账号暂无可用跑区"));
    }
    Ok(cra.clone())
}

pub async fn get_home_run_stat(
    school_url: &str,
    uuid: &str,
    device_name: &str,
    token: &str,
) -> Value {
    let url = endpoint(school_url, "/run/getHomeRunInfo");
    if let Some(content) = post_empty(&url, uuid, device_name, token).await {
        if content.get("code").and_then(|c| c.as_i64()) != Some(401) {
            return content.get("data").cloned().unwrap_or(Value::Null);
        }
    }
    Value::Null
}

pub async fn run_start(
    school_url: &str,
    uuid: &str,
    token: &str,
    ra_run_area: &str,
    ra_type: &str,
    ra_id: i64,
    device_name: &str,
) -> Value {
    let parameter = json!({ "raRunArea": ra_run_area, "raType": ra_type, "raId": ra_id.to_string() });
    let url = endpoint(school_url, "/run/start");
    if let Some(content) = post_json(&url, &parameter, uuid, device_name, token).await {
        if content.get("code").and_then(|c| c.as_i64()) == Some(200) {
            let info = content.get("data").unwrap_or(&Value::Null);
            return json!({
                "id": info.get("id"),
                "recordStartTime": info.get("recordStartTime"),
                "ip": info.get("ip"),
                "warnContent": info.get("warnContent"),
                "raId": info.get("raId"),
                "faceTime": info.get("faceTime"),
                "randomList": info.get("randomList"),
            });
        }

        return content;
    }
    json!({ "id": null, "recordStartTime": null, "ip": null, "warnContent": "接口请求失败", "raId": null })
}

pub async fn split_point_cheating(
    school_url: &str,
    token: &str,
    y_point: &Value,
    uuid: &str,
    device_name: &str,
) {
    let url = endpoint(school_url, "/run/splitPointCheating");
    let bytes = serde_json::to_vec(y_point).unwrap_or_default();
    let gz = gzip_bytes(&bytes);
    let r = post_bytes(&url, &gz, uuid, device_name, token).await;
    let n = y_point.get("cardPointList").and_then(|c| c.as_array()).map(|a| a.len()).unwrap_or(0);
    eprintln!(
        "[splitPoint] cardPointList={} json_len={} gz_len={} resp_code={:?} resp_msg={:?}",
        n,
        bytes.len(),
        gz.len(),
        r.as_ref().and_then(|c| c.get("code")),
        r.as_ref().and_then(|c| c.get("msg"))
    );
}

pub async fn is_standard(
    school_url: &str,
    uuid: &str,
    token: &str,
    device_name: &str,
    generate_data: &Value,
) -> Value {
    let url = endpoint(school_url, "/run/isStandard");
    eprintln!("[isStandard] payload={}", generate_data);
    post_json(&url, generate_data, uuid, device_name, token).await.unwrap_or(Value::Null)
}

pub async fn finish(
    school_url: &str,
    token: &str,
    generate_data: &Value,
    uuid: &str,
    device_name: &str,
) -> String {
    let url = endpoint(school_url, "/run/finish");
    eprintln!("[finish] payload={}", generate_data);
    if let Some(content) = post_json(&url, generate_data, uuid, device_name, token).await {
        return content.get("msg").and_then(|m| m.as_str()).unwrap_or("未知结果").to_string();
    }
    "finish 请求失败".into()
}

pub async fn start_face_run(school_url: &str, uuid: &str, token: &str, ra_run_area: &str, device_name: &str) -> Option<String> {
    let url = endpoint(school_url, "/run/getRlStatus");
    let parameter = json!({ "raRunArea": ra_run_area });
    let content = post_json(&url, &parameter, uuid, device_name, token).await?;
    if content.get("code").and_then(|c| c.as_i64()) != Some(200) {
        return None;
    }
    content.get("data").and_then(|d| d.get("runFaceStudentStatus")).and_then(|s| s.as_str()).map(|s| s.to_string())
}

pub async fn face_run_add_face(school_url: &str, uuid: &str, token: &str, face_base_data: &str, device_name: &str) -> Option<Value> {
    let url = endpoint(school_url, "/run/appFace/runFaceInfo");
    let parameter = json!({ "faceBaseData": face_base_data });
    post_json(&url, &parameter, uuid, device_name, token).await
}

pub async fn face_run_compare_face(
    school_url: &str,
    uuid: &str,
    token: &str,
    face_base_data: &str,
    record_id: &str,
    device_name: &str,
) -> Option<Value> {
    let url = endpoint(school_url, "/run/appFace/runFaceInfoComparison");
    let parameter = json!({ "faceBaseData": face_base_data, "recordId": record_id });
    post_json(&url, &parameter, uuid, device_name, token).await
}

pub async fn list_xn_year_xq_by_student_id(school_url: &str, token: &str, uuid: &str, device_name: &str) -> Result<Vec<Value>, AppError> {
    let url = endpoint(school_url, "/run/listXnYearXqByStudentId");
    let v = resp_result(post_empty(&url, uuid, device_name, token).await, "查询学年学期失败（网络异常）")?;
    Ok(v.get("data").and_then(|d| d.as_array()).cloned().unwrap_or_default())
}

pub async fn crs_reocord_info_list(school_url: &str, token: &str, uuid: &str, device_name: &str, table_name: &str) -> Option<Value> {
    let url = endpoint(school_url, "/run/crsReocordInfoList");
    let data = json!({ "tableName": table_name });
    post_json(&url, &data, uuid, device_name, token).await
}

pub(crate) fn strip_data_uri(b64: &str) -> String {
    if b64.starts_with("data:") && b64.contains(',') {
        b64.splitn(2, ',').nth(1).unwrap_or(b64).to_string()
    } else {
        b64.to_string()
    }
}

pub async fn have_yd_info(school_url: &str, token: &str, uuid: &str, device: &str) -> Result<Value, AppError> {
    let xn = list_xn_year_xq_by_student_id(school_url, token, uuid, device).await?;
    if xn.is_empty() {
        return Err(AppError::from("无可用学期"));
    }
    let xn_year = xn[0].get("value").and_then(|x| x.as_str()).unwrap_or("").to_string();
    match crs_reocord_info_list(school_url, token, uuid, device, &xn_year).await {
        Some(v) if is_auth_failure(&v) => Err(auth_error_of(&v)),
        Some(v) => Ok(v.get("data").cloned().unwrap_or(Value::Null)),
        None => Err(AppError::from("查询运动数据失败（网络异常）")),
    }
}
