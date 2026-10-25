use crate::error::AppError;
use crate::services::auth_service::build_yun_run_kwargs;
use crate::services::track;
use crate::services::upstream_service as up;
use crate::state::AppState;
use crate::utils::crypto::{new_uuid_upper, VERSION};
use chrono::NaiveDateTime;
use rand::Rng;
use serde_json::{json, Value};
use std::panic::{self, AssertUnwindSafe};
use tauri::{AppHandle, Emitter, Manager};

fn build_finish_from_batches(
    batches: &[track::RunSplitPayload],
    device_name: &str,
    ra_run_area: &str,
    ra_type: &str,
    record_id: &str,
    record_start_time: &str,
    ra_id: &str,
    card_points: &[String],
    ra_dislikes: i64,
    app_no_qualified_reason: Option<&str>,
) -> Value {
    let all_card_points: Vec<&track::CardPoint> = batches.iter().flat_map(|b| b.cardPointList.iter()).collect();
    let last = all_card_points.last().unwrap();
    let total_mileage_m = last.runMileage;
    let total_time_s = last.runTime;
    let total_steps = last.runStep;
    let total_km = total_mileage_m / 1000.0;
    let pace = if total_km > 0.0 { (total_time_s as f64 / 60.0) / total_km } else { 0.0 };
    let cadence = if total_time_s > 0 { (total_steps as f64 / (total_time_s as f64 / 60.0)) as i64 } else { 0 };

    let manage_list = if !card_points.is_empty() {
        let selected = if ra_dislikes > 0 && (ra_dislikes as usize) < card_points.len() {

            let mut idx: Vec<usize> = (0..card_points.len()).collect();
            let mut rng = rand::rng();
            for i in 0..(ra_dislikes as usize) {
                let j = rng.random_range(i..card_points.len());
                idx.swap(i, j);
            }
            idx[..ra_dislikes as usize].iter().map(|&i| card_points[i].clone()).collect::<Vec<_>>()
        } else {
            card_points.to_vec()
        };
        selected
            .iter()
            .enumerate()
            .map(|(i, pt)| json!({ "point": pt, "marked": "Y", "index": i.to_string() }))
            .collect::<Vec<_>>()
    } else {
        let all_points: Vec<String> = all_card_points.iter().map(|p| p.point.clone()).collect();
        let take_n = all_points.len().min(5);
        let selected = if take_n > 1 {
            (0..take_n).map(|i| all_points[(i * (all_points.len() - 1) / (take_n - 1)).min(all_points.len() - 1)].clone()).collect::<Vec<_>>()
        } else {
            all_points.iter().take(1).cloned().collect()
        };
        selected
            .iter()
            .enumerate()
            .map(|(i, pt)| json!({ "point": pt, "marked": "Y", "index": i.to_string() }))
            .collect::<Vec<_>>()
    };

    let mut payload = json!({
        "manageList": manage_list,
        "recordMileage": format!("{:.2}", total_km),
        "recodeCadence": cadence.to_string(),
        "recodePace": format!("{:.2}", pace),
        "deviceName": device_name,
        "sysEdition": "Android_15",
        "appEdition": VERSION,
        "raIsStartPoint": "Y",
        "raIsEndPoint": "Y",
        "raRunArea": ra_run_area,
        "recodeDislikes": manage_list.len().to_string(),
        "raId": ra_id,
        "raType": ra_type,
        "id": record_id,
        "duration": total_time_s.to_string(),
        "recordStartTime": record_start_time,
        "remake": "0|{}",
    });
    if let Some(reason) = app_no_qualified_reason {
        payload["appNoQualifiedReason"] = json!(reason);
    }
    payload
}

fn parse_beijing_to_epoch(s: &str) -> i64 {
    let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap_or_else(|_| NaiveDateTime::default());
    let epoch = NaiveDateTime::parse_from_str("1970-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    (dt - epoch).num_seconds() + 8 * 3600
}

async fn finalize_run(
    state: &AppState,
    school_url: &str,
    uuid: &str,
    token: &str,
    device_name: &str,
    batches: &[track::RunSplitPayload],
    ra_run_area: &str,
    ra_type: &str,
    record_id: &str,
    record_start_time: &str,
    ra_id: &str,
    card_points: &[String],
    ra_dislikes: i64,
    reason: Option<String>,
    hold_finish: bool,
) -> Result<Value, AppError> {

    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    let manage_point = build_finish_from_batches(
        batches,
        device_name,
        ra_run_area,
        ra_type,
        record_id,
        record_start_time,
        ra_id,
        card_points,
        ra_dislikes,
        reason.as_deref(),
    );

    eprintln!(
        "[finalize] id={} raId={} raType={} raRunArea={} mileage={} duration={} cadence={} pace={} dislike={} reason={:?}",
        record_id, ra_id, ra_type, ra_run_area,
        manage_point.get("recordMileage").unwrap_or(&Value::Null),
        manage_point.get("duration").unwrap_or(&Value::Null),
        manage_point.get("recodeCadence").unwrap_or(&Value::Null),
        manage_point.get("recodePace").unwrap_or(&Value::Null),
        manage_point.get("recodeDislikes").unwrap_or(&Value::Null),
        reason,
    );
    let resp = up::is_standard(school_url, uuid, token, device_name, &manage_point).await;
    if up::is_auth_failure(&resp) {
        return Err(up::auth_error_of(&resp));
    }
    let code = resp.get("code").and_then(|c| c.as_i64());
    let srv_msg = resp.get("msg").and_then(|m| m.as_str()).unwrap_or("").to_string();

    let data = resp.get("data").cloned().unwrap_or(Value::Null);
    let data_msg = data.get("msg").and_then(|m| m.as_str()).unwrap_or("").to_string();
    eprintln!(
        "[finalize] isStandard => code={:?} msg={:?} full={}",
        code, srv_msg, resp
    );
    if hold_finish {

        state.pending_finishes.lock().unwrap().insert(
            record_id.to_string(),
            json!({
                "payload": manage_point,
                "ctx": { "school_url": school_url, "uuid": uuid, "token": token, "device_name": device_name },
            }),
        );
        return Ok(json!({
            "held": true,
            "record_id": record_id,
            "record_start_time": record_start_time,
            "verify": {
                "code": code,
                "msg": if srv_msg.is_empty() { data_msg } else { srv_msg },
                "data": data,
            },
        }));
    }
    let msg = up::finish(school_url, token, &manage_point, uuid, device_name).await;
    if msg.contains("认证失败") {
        return Err(AppError::from(format!("AUTH_EXPIRED:{}", msg)));
    }
    Ok(json!({ "msg": msg, "start_time": record_start_time }))
}

pub async fn api_finish_run(state: &AppState, record_id: &str) -> Result<Value, AppError> {
    let held = state
        .pending_finishes
        .lock()
        .unwrap()
        .remove(record_id)
        .ok_or_else(|| AppError::from("没有待确认的跑步记录"))?;
    let ctx = held.get("ctx").cloned().unwrap_or(json!({}));
    let school_url = ctx.get("school_url").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let uuid = ctx.get("uuid").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let token = ctx.get("token").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let device = ctx.get("device_name").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let payload = held.get("payload").cloned().unwrap_or(json!({}));
    let msg = up::finish(&school_url, &token, &payload, &uuid, &device).await;
    if msg.contains("认证失败") {
        return Err(AppError::from(format!("AUTH_EXPIRED:{}", msg)));
    }
    Ok(json!({ "msg": msg, "record_id": record_id }))
}

pub async fn api_discard_run(state: &AppState, record_id: &str) -> Result<Value, AppError> {
    match state.pending_finishes.lock().unwrap().remove(record_id) {
        Some(_) => Ok(json!({ "msg": "已放弃提交，本次成绩未记录" })),
        None => Err(AppError::from("没有待确认的跑步记录")),
    }
}

fn jitter_distance(dist: i64, max_m: i64) -> i64 {
    if dist <= 0 || dist % 100 != 0 {
        return dist;
    }
    let base = dist - dist % 100;
    let tail: i64 = rand::rng().random_range(1..100);
    if base + tail <= max_m {
        base + tail
    } else {
        (base - (100 - tail)).max(1)
    }
}

pub fn api_simulate(
    session: &Value,
    target_distance: i64,
    ra_name: &str,
    fast: bool,
    points: Option<Vec<Value>>,
) -> Result<Value, AppError> {
    let school_code = up::field_str(session, "school_code");
    let user = up::field_str(session, "user");
    if school_code.is_empty() {
        return Err(AppError::from("会话信息不完整"));
    }
    let dist = jitter_distance(target_distance.clamp(1, 4500), 4500);
    let start_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let cfg = crate::services::config_service::load_config();
    let cadence_range = (
        if cfg.cadence_min > 0.0 { cfg.cadence_min } else { 164.0 },
        if cfg.cadence_max > cfg.cadence_min { cfg.cadence_max } else { 176.0 },
    );
    let pace_range = (
        if cfg.pace_min > 0.0 { cfg.pace_min } else { 5.5 },
        if cfg.pace_max > cfg.pace_min { cfg.pace_max } else { 6.7 },
    );
    let _ = fast;

    let base_points: Option<Vec<track::GPSPoint>> = match points {
        Some(arr) => {
            let mut v = Vec::with_capacity(arr.len());
            let mut ok = 0;
            for p in &arr {
                let lng = p.get("lng").and_then(|x| x.as_f64());
                let lat = p.get("lat").and_then(|x| x.as_f64());
                if let (Some(lng), Some(lat)) = (lng, lat) {
                    if lng.is_finite() && lat.is_finite() {
                        v.push((lng, lat));
                        ok += 1;
                    }
                }
            }
            if ok >= 2 {
                Some(v)
            } else {
                None
            }
        }
        None => None,
    };

    let mut warnings: Vec<String> = Vec::new();

    let mut input_length_m: f64 = 0.0;
    if let Some(pts) = &base_points {
        let (_, span, total_len) = track::base_route_stats(pts);
        input_length_m = total_len;
        if span > 50.0 && total_len > 0.0 && span > total_len * 0.25 {
            warnings.push(format!(
                "导入的轨迹首尾间距 {:.0} 米（未闭合），生成时会自动衔接闭合，轨迹将出现横穿场地的直线段；建议导入完整一圈的轨迹",
                span
            ));
        }
    }
    let batches = {
        if let Some(pts) = base_points {

            if let Some(rep) = track::jump_report(&pts) {
                if rep.max_seg > 300.0 {
                    return Err(AppError::from(format!(
                        "导入的轨迹存在异常跳跃（第 {} 段跨度 {:.0} 米，中位段 {:.0} 米，总长 {:.0} 米），请检查轨迹文件后重新导入",
                        rep.max_index, rep.max_seg, rep.median_seg, rep.total_len
                    )));
                }
                if track::has_abnormal_jump(&pts, 80.0) {
                    warnings.push(format!(
                        "导入的轨迹第 {} 段跨度 {:.0} 米，明显大于其它段（中位 {:.0} 米），生成时按折线插值平滑，可能出现横穿场地的直线段",
                        rep.max_index, rep.max_seg, rep.median_seg
                    ));
                }
            }
            let res = panic::catch_unwind(AssertUnwindSafe(|| {
                track::generate_run_data_from_points(
                    dist, "SIM", &user, &school_code, pace_range, (0.9, 1.12), cadence_range, None, Some(start_epoch), &pts,
                )
            }));
            match res {
                Ok(b) => b,
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "轨迹生成失败".to_string()
                    };
                    return Err(AppError::from(msg));
                }
            }
        } else {
            let res = panic::catch_unwind(AssertUnwindSafe(|| {
                track::generate_run_data(
                    dist,
                    "SIM",
                    &user,
                    ra_name,
                    &school_code,
                    pace_range,
                    (0.9, 1.12),
                    cadence_range,
                    "",
                    None,
                    Some(start_epoch),
                    "auto",
                )
            }));
            match res {
                Ok(b) => b,
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "轨迹生成失败".to_string()
                    };
                    return Err(AppError::from(msg));
                }
            }
        }
    };
    let mut pts: Vec<Value> = Vec::new();
    for b in &batches {
        for c in &b.cardPointList {
            let mut it = c.point.split(',');
            let lng: f64 = it.next().unwrap_or("").trim().parse().unwrap_or(f64::NAN);
            let lat: f64 = it.next().unwrap_or("").trim().parse().unwrap_or(f64::NAN);
            if !lng.is_finite() || !lat.is_finite() {
                continue;
            }
            pts.push(json!({
                "lng": lng,
                "lat": lat,
                "mileage": c.runMileage,
                "time": c.runTime,
                "step": c.runStep,
                "speed": c.speed,
                "ts": c.ts,
            }));
        }
    }
    if pts.len() < 2 {
        return Err(AppError::from("该跑区没有可用轨迹，请先绘制并保存轨迹"));
    }
    let last = pts.last().cloned().unwrap_or(json!({}));
    let total_mileage_m = last.get("mileage").and_then(|x| x.as_f64()).unwrap_or(0.0);
    Ok(json!({
        "points": pts,
        "total_mileage_m": total_mileage_m,
        "input_length_m": if input_length_m > 0.0 { input_length_m } else { total_mileage_m },
        "total_time_s": last.get("time").and_then(|x| x.as_i64()).unwrap_or(0),
        "total_step": last.get("step").and_then(|x| x.as_i64()).unwrap_or(0),
        "warnings": warnings,
    }))
}

fn preview_points_to_track(points: &[Value]) -> Result<Vec<track::TrackPoint>, AppError> {
    let mut tps = Vec::with_capacity(points.len());
    for (i, p) in points.iter().enumerate() {
        let lng = p.get("lng").and_then(|x| x.as_f64()).ok_or_else(|| AppError::from("点数据缺少经度"))?;
        let lat = p.get("lat").and_then(|x| x.as_f64()).ok_or_else(|| AppError::from("点数据缺少纬度"))?;
        let mileage = p.get("mileage").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let time = p.get("time").and_then(|x| x.as_i64()).unwrap_or(0);
        let step = p.get("step").and_then(|x| x.as_i64()).unwrap_or(0);
        let (prev_m, prev_t) = if i > 0 {
            (
                points[i - 1].get("mileage").and_then(|x| x.as_f64()).unwrap_or(mileage),
                points[i - 1].get("time").and_then(|x| x.as_i64()).unwrap_or(time),
            )
        } else {
            (0.0, 0)
        };
        let seg_m = (mileage - prev_m).max(0.0);
        let seg_t = (time - prev_t).max(1) as f64;
        tps.push(track::TrackPoint {
            longitude: lng,
            latitude: lat,
            run_mileage_m: mileage,
            run_time: time,
            run_step: step,
            pace: track::compute_pace_min_per_km(seg_m, seg_t),
            ts: String::new(),
        });
    }
    Ok(tps)
}

pub async fn yun_run(
    state: &AppState,
    kw: &crate::services::auth_service::YunRunKwargs,
    progress: Option<&AppHandle>,
    hold_finish: bool,
    custom_points: Option<Vec<track::TrackPoint>>,
    submit_distance: Option<f64>,
) -> Result<Value, AppError> {
    let run_info = up::get_home_run_info(&kw.school_url, &kw.uuid, &kw.device_name, &kw.token).await?;

    let item = if !kw.ra_name.is_empty() {
        run_info
            .iter()
            .find(|i| i.get("raName").and_then(|x| x.as_str()) == Some(kw.ra_name.as_str()))
            .cloned()
            .unwrap_or_else(|| run_info[0].clone())
    } else if run_info.len() >= 2 {
        run_info
            .iter()
            .find(|i| i.get("raName").and_then(|x| x.as_str()).map(|n| n.contains("蔚然运动场")).unwrap_or(false))
            .cloned()
            .unwrap_or_else(|| run_info[0].clone())
    } else {
        run_info[0].clone()
    };
    let run_area_id = item.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
    let ra_run_area = up::field_str(&item, "raRunArea");
    let ra_type = up::field_str(&item, "raType");
    let ra_name = up::field_str(&item, "raName");
    let points_str = up::field_str(&item, "points");
    let card_points: Vec<String> = points_str.split('|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let ra_dislikes = item.get("raDislikes").and_then(|x| x.as_i64()).unwrap_or(0);

    let mut face_data: Option<String> = None;
    let run_face_status = up::field_str(&item, "runFaceStatus");
    if run_face_status == "Y" {
        let fb = kw.face_base64.clone().ok_or_else(|| AppError::from("该任务需要刷脸，必须提供 face_base64"))?;
        face_data = Some(up::strip_data_uri(&fb));
        let status = up::start_face_run(&kw.school_url, &kw.uuid, &kw.token, &ra_run_area, &kw.device_name).await;
        match status.as_deref() {
            Some("N") | Some("N0") => {
                if let Some(fd) = &face_data {
                    let _ = up::face_run_add_face(&kw.school_url, &kw.uuid, &kw.token, fd, &kw.device_name).await;
                }
            }
            Some("N1") => return Err(AppError::from("身份认证审核中，无法进行跑步")),
            _ => {}
        }
    }

    let start_info = up::run_start(&kw.school_url, &kw.uuid, &kw.token, &ra_run_area, &ra_type, run_area_id, &kw.device_name).await;
    if up::is_auth_failure(&start_info) {
        return Err(up::auth_error_of(&start_info));
    }
    let record_id = match start_info.get("id").and_then(|x| x.as_str()) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return Err(AppError::from("run/start 未返回 recordId")),
    };
    let record_start_time = start_info.get("recordStartTime").and_then(|x| x.as_str()).unwrap_or("").to_string();

    let ra_id = start_info
        .get("raId")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| run_area_id.to_string());
    let start_epoch = parse_beijing_to_epoch(&record_start_time) + 6;
    let random_list: Vec<f64> = start_info
        .get("randomList")
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();

    let batches = {
        if let Some(pts) = custom_points.as_ref() {
            track::build_batches_from_points(&record_id, &kw.user, &kw.school_code, pts, start_epoch, submit_distance)
                .map_err(AppError::from)?
        } else {
            let res = panic::catch_unwind(AssertUnwindSafe(|| {
                track::generate_run_data(
                    kw.target_distance,
                    &record_id,
                    &kw.user,
                    &ra_name,
                    &kw.school_code,
                    kw.pace_range,
                    (0.9, 1.12),
                    kw.cadence_range,
                    "",
                    None,
                    Some(start_epoch),
                    "auto",
                )
            }));
            match res {
                Ok(b) => b,
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "轨迹生成失败".to_string()
                    };
                    return Err(AppError::from(msg));
                }
            }
        }
    };

    let pending_checks: Vec<f64> = random_list.iter().map(|t| *t).collect();
    let mut check_idx = 0;
    let total = batches.len();

    for (i, y_point) in batches.iter().enumerate() {
        let sleep_sec = if kw.fast { 0.05 } else { y_point.times as f64 };
        tokio::time::sleep(std::time::Duration::from_secs_f64(sleep_sec)).await;
        let cum_mileage = y_point.cardPointList.last().unwrap().runMileage;
        while face_data.is_some() && check_idx < pending_checks.len() && cum_mileage >= pending_checks[check_idx] * 1000.0 {
            let fd = face_data.clone().unwrap();
            let res = up::face_run_compare_face(&kw.school_url, &kw.uuid, &kw.token, &fd, &record_id, &kw.device_name).await;
            if res.as_ref().and_then(|d| d.get("data")).and_then(|d| d.get("status")).and_then(|s| s.as_str()) != Some("Y") {
                let reason = format!("人脸识别对比失败(抽查点 {}km)", pending_checks[check_idx]);
                let sent: Vec<track::RunSplitPayload> = batches[..i].to_vec();
                if sent.is_empty() {
                    return Err(AppError::from(format!("{}，且里程不足无法提交", reason)));
                }
                return finalize_run(state, &kw.school_url, &kw.uuid, &kw.token, &kw.device_name, &sent, &ra_run_area, &ra_type, &record_id, &record_start_time, &ra_id, &card_points, ra_dislikes, Some(reason), hold_finish).await;
            }
            check_idx += 1;
        }
        let value = serde_json::to_value(y_point).unwrap_or(Value::Null);
        up::split_point_cheating(&kw.school_url, &kw.token, &value, &kw.uuid, &kw.device_name).await;

        if let Some(app) = progress {
            let pts: Vec<Value> = y_point
                .cardPointList
                .iter()
                .filter_map(|c| {
                    let mut it = c.point.split(',');
                    let lng: f64 = it.next()?.trim().parse().ok()?;
                    let lat: f64 = it.next()?.trim().parse().ok()?;
                    Some(json!({
                        "lng": lng, "lat": lat,
                        "mileage": c.runMileage, "time": c.runTime, "step": c.runStep,
                    }))
                })
                .collect();
            let _ = app.emit(
                "run-progress",
                json!({ "status": "running", "record_id": record_id, "total": total, "uploaded": i + 1, "points": pts }),
            );
        }
        if i + 1 == total {
            return finalize_run(state, &kw.school_url, &kw.uuid, &kw.token, &kw.device_name, &batches, &ra_run_area, &ra_type, &record_id, &record_start_time, &ra_id, &card_points, ra_dislikes, None, hold_finish).await;
        }
    }
    Err(AppError::from("跑步流程异常结束"))
}

pub async fn api_yun_run(
    state: &AppState,
    app: &AppHandle,
    session: &Value,
    target_distance: i64,
    ra_name: &str,
    fast: bool,
    face_base64: Option<String>,
    hold_finish: bool,
    points: Option<Vec<Value>>,
    submit_distance: Option<f64>,
) -> Result<Value, AppError> {

    let custom_points = match points {
        Some(arr) if arr.len() >= 2 => Some(preview_points_to_track(&arr)?),
        _ => None,
    };
    let kw = build_yun_run_kwargs(state, session, target_distance, ra_name, fast, face_base64).await?;

    {
        let mut running = state.running_users.lock().unwrap();
        if running.contains(&kw.user) {
            return Err(AppError::from("该账号当前正在跑步，请稍后再试"));
        }
        running.insert(kw.user.clone());
    }

    if fast {
        let res = yun_run(state, &kw, Some(app), hold_finish, custom_points, submit_distance).await;
        state.running_users.lock().unwrap().remove(&kw.user);
        return res.map(|r| json!({ "result": r }));
    }

    let task_id = new_uuid_upper();
    {
        let mut tasks = state.tasks.lock().unwrap();
        tasks.insert(
            task_id.clone(),
            crate::state::TaskRow { task_id: task_id.clone(), user: kw.user.clone(), status: "pending".into(), result: None },
        );
    }
    let app2 = app.clone();
    let kw_user = kw.user.clone();
    let owned_kw = kw;
    let task_id_task = task_id.clone();
    tauri::async_runtime::spawn(async move {
        let st_guard = app2.state::<AppState>();
        let st = st_guard.inner();
        let res = yun_run(st, &owned_kw, Some(&app2), hold_finish, custom_points, submit_distance).await;
        let (status, result) = match res {
            Ok(v) => ("done".to_string(), Some(v.to_string())),
            Err(e) => ("failed".to_string(), Some(e.to_string())),
        };
        {
            let mut tasks = st.tasks.lock().unwrap();
            if let Some(row) = tasks.get_mut(&task_id_task) {
                row.status = status.clone();
                row.result = result.clone();
            }
        }
        st.running_users.lock().unwrap().remove(&kw_user);
        let _ = app2.emit("run-progress", json!({ "task_id": task_id_task, "status": status, "result": result }));
    });
    Ok(json!({ "task_id": task_id, "status": "pending" }))
}

pub fn get_task(state: &AppState, task_id: &str) -> Option<Value> {
    let tasks = state.tasks.lock().unwrap();
    tasks.get(task_id).map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
}
