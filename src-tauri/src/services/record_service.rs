use crate::error::AppError;
use crate::services::upstream_service as up;
use crate::state::AppState;
use crate::utils::protocol::{endpoint, post_json};
use serde_json::{json, Value};

pub async fn api_run_record_detail(state: &AppState, session: &Value, record_id: &str) -> Result<Value, AppError> {
    let school_url = up::field_str(session, "school_url");
    let uuid = up::field_str(session, "uuid");
    let device = up::field_str(session, "device_name");
    let token = up::field_str(session, "token");
    if school_url.is_empty() {
        return Err(AppError::from("会话信息不完整"));
    }
    let xn = up::list_xn_year_xq_by_student_id(&school_url, &token, &uuid, &device).await?;
    if xn.is_empty() {
        return Err(AppError::from("无可用学期"));
    }
    let table_name = xn[0].get("value").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let parameter = json!({ "id": record_id, "tableName": table_name });
    let url = endpoint(&school_url, "/run/crsReocordInfo");
    let content = post_json(&url, &parameter, &uuid, &device, &token).await.ok_or_else(|| AppError::from("获取记录详情失败"))?;
    if up::is_auth_failure(&content) {
        return Err(up::auth_error_of(&content));
    }
    if content.get("code").and_then(|c| c.as_i64()) != Some(200) {
        return Err(AppError::from(content.get("msg").and_then(|m| m.as_str()).unwrap_or("获取记录详情失败").to_string()));
    }
    let _ = state;
    Ok(content)
}

pub async fn api_record_track(session: &Value, record_id: &str) -> Result<Value, AppError> {
    let school_url = up::field_str(session, "school_url");
    let uuid = up::field_str(session, "uuid");
    let device = up::field_str(session, "device_name");
    let token = up::field_str(session, "token");
    if school_url.is_empty() {
        return Err(AppError::from("会话信息不完整"));
    }
    let xn = up::list_xn_year_xq_by_student_id(&school_url, &token, &uuid, &device).await?;
    if xn.is_empty() {
        return Err(AppError::from("无可用学期"));
    }
    let table_name = xn[0].get("value").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let parameter = json!({ "id": record_id, "tableName": table_name });
    let url = endpoint(&school_url, "/run/crsReocordInfo");
    let content = post_json(&url, &parameter, &uuid, &device, &token)
        .await
        .ok_or_else(|| AppError::from("获取记录详情失败"))?;
    if up::is_auth_failure(&content) {
        return Err(up::auth_error_of(&content));
    }
    if content.get("code").and_then(|c| c.as_i64()) != Some(200) {
        return Err(AppError::from(
            content.get("msg").and_then(|m| m.as_str()).unwrap_or("获取记录详情失败").to_string(),
        ));
    }
    let r = content.get("data").unwrap_or(&Value::Null);
    let pts: Vec<[f64; 2]> = (r.get("pointsList").and_then(|x| x.as_array()).cloned().unwrap_or_default())
        .iter()
        .filter_map(|p| {
            let s = p.get("point")?.as_str()?;
            let mut it = s.split(',');
            let lng: f64 = it.next()?.trim().parse().ok()?;
            let lat: f64 = it.next()?.trim().parse().ok()?;
            Some([lng, lat])
        })
        .collect();
    if pts.len() < 2 {
        return Err(AppError::from("该记录没有可用轨迹点"));
    }

    let cleaned = crate::services::track_service::clean_real_track(&pts);
    Ok(json!({ "points": cleaned }))
}

pub async fn api_sport_info(state: &AppState, session: &Value) -> Result<Value, AppError> {
    let school_url = up::field_str(session, "school_url");
    let uuid = up::field_str(session, "uuid");
    let device = up::field_str(session, "device_name");
    let token = up::field_str(session, "token");
    if school_url.is_empty() {
        return Err(AppError::from("会话信息不完整"));
    }
    let stat = up::get_home_run_stat(&school_url, &uuid, &device, &token).await;
    let records = up::have_yd_info(&school_url, &token, &uuid, &device).await?;
    let _ = state;
    Ok(json!({ "stat": stat, "records": records }))
}

pub async fn api_have_yd_info(session: &Value) -> Result<Value, AppError> {
    let school_url = up::field_str(session, "school_url");
    let uuid = up::field_str(session, "uuid");
    let device = up::field_str(session, "device_name");
    let token = up::field_str(session, "token");
    if school_url.is_empty() {
        return Err(AppError::from("会话信息不完整"));
    }
    up::have_yd_info(&school_url, &token, &uuid, &device).await
}
