use crate::services::track::route_distance_m;
use crate::utils::tracks_dir;
use rand::Rng;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn track_path(school_code: &str, ra_name: &str) -> PathBuf {
    tracks_dir(school_code).join(format!("{}.json", ra_name))
}

fn find_track_file(school_code: &str, ra_name: &str) -> Option<PathBuf> {
    let exact = track_path(school_code, ra_name);
    if exact.exists() {
        return Some(exact);
    }
    let dir = tracks_dir(school_code);
    if !dir.is_dir() {
        return None;
    }
    let norm = ra_name.trim().to_lowercase();
    let mut best: Option<(usize, PathBuf)> = None;
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map(|x| x.eq_ignore_ascii_case("json")).unwrap_or(false) {
                let stem = p.file_stem().unwrap().to_string_lossy().to_lowercase();
                if !stem.is_empty() && norm.contains(&stem) {
                    let l = stem.len();
                    if best.as_ref().map(|(bl, _)| l > *bl).unwrap_or(true) {
                        best = Some((l, p));
                    }
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

fn load_points(p: &Path) -> Option<Vec<[f64; 2]>> {
    let raw: Value = serde_json::from_str(&fs::read_to_string(p).ok()?).ok()?;
    extract_points(&raw)
}

fn extract_points(v: &Value) -> Option<Vec<[f64; 2]>> {
    let arr = match v {
        Value::Array(a) => a,
        Value::Object(_) => v.get("points").and_then(|x| x.as_array())?,
        _ => return None,
    };
    let mut out = Vec::new();
    for it in arr {
        let a = it.as_array()?;
        if a.len() < 2 {
            return None;
        }
        out.push([a[0].as_f64()?, a[1].as_f64()?]);
    }
    Some(out)
}

pub fn track_status(school_code: &str, ra_names: &[String]) -> Value {
    let missing: Vec<String> = ra_names
        .iter()
        .filter(|n| find_track_file(school_code, n).is_none())
        .cloned()
        .collect();
    json!({ "missing": missing })
}

pub fn track_query(school_code: &str, ra_name: &str) -> Value {
    match find_track_file(school_code, ra_name) {
        Some(p) => {
            let pts = load_points(&p).unwrap_or_default();
            json!({
                "found": true,
                "file": p.file_name().unwrap().to_string_lossy(),
                "points": pts,
            })
        }
        None => json!({ "found": false }),
    }
}

pub fn validate(_school_code: &str, _ra_name: &str, _route_type: &str, points: &[[f64; 2]]) -> Result<Value, String> {
    if points.len() < 2 {
        return Err("至少需要 2 个点".into());
    }
    let gps: Vec<(f64, f64)> = points.iter().map(|p| (p[0], p[1])).collect();
    let dist = route_distance_m(&gps);
    Ok(json!({ "distance_m": dist.round() as i64, "points": points }))
}

pub fn list_local(school_code: &str) -> Value {
    let dir = tracks_dir(school_code);
    let mut items: Vec<Value> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if !p.extension().map(|x| x.eq_ignore_ascii_case("json")).unwrap_or(false) {
                continue;
            }
            let raw: Value = match serde_json::from_str(&fs::read_to_string(&p).unwrap_or_default()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let pts = match extract_points(&raw) {
                Some(x) if x.len() >= 2 => x,
                _ => continue,
            };
            let gps: Vec<(f64, f64)> = pts.iter().map(|q| (q[0], q[1])).collect();
            let name = p.file_stem().unwrap().to_string_lossy().to_string();
            let mut item = json!({
                "name": name,
                "distance_m": route_distance_m(&gps).round() as i64,
                "points": pts,
            });

            if let Some(meta) = raw.get("meta").and_then(|m| m.as_object()) {
                for k in ["created_at", "ra_name", "record_mileage", "is_qualified", "record_id"] {
                    if let Some(v) = meta.get(k) {
                        item[k] = v.clone();
                    }
                }
            }
            items.push(item);
        }
    }

    items.sort_by(|a, b| {
        let ka = a.get("created_at").and_then(|x| x.as_str()).unwrap_or("");
        let kb = b.get("created_at").and_then(|x| x.as_str()).unwrap_or("");
        match (ka.is_empty(), kb.is_empty()) {
            (false, false) => kb.cmp(ka),
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            (true, true) => a
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .cmp(b.get("name").and_then(|x| x.as_str()).unwrap_or("")),
        }
    });
    json!({ "items": items })
}

pub fn save_record_local(school_code: &str, name: &str, meta: &Value, points: &[[f64; 2]]) -> Result<Value, String> {
    if points.len() < 2 {
        return Err("至少需要 2 个点".into());
    }

    let stem: String = name.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect();
    if stem.is_empty() {
        return Err("无效的轨迹名称".into());
    }
    let dir = tracks_dir(school_code);
    let path = dir.join(format!("{}.json", stem));
    let payload = json!({ "meta": meta, "points": points });
    fs::write(&path, serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let gps: Vec<(f64, f64)> = points.iter().map(|q| (q[0], q[1])).collect();
    Ok(json!({ "name": stem, "distance_m": route_distance_m(&gps).round() as i64 }))
}

pub fn refine(points: &[[f64; 2]]) -> Result<Value, String> {
    if points.len() < 2 {
        return Err("至少需要 2 个点".into());
    }
    let cleaned = clean_real_track(points);
    let gps: Vec<(f64, f64)> = cleaned.iter().map(|p| (p[0], p[1])).collect();
    let mut rng = rand::rng();

    let step = 3.0_f64;
    let mut out: Vec<[f64; 2]> = vec![[gps[0].0, gps[0].1]];
    let mut carry = 0.0f64;
    for w in gps.windows(2) {
        let seg = route_distance_m(&[w[0], w[1]]);
        if seg <= 1e-9 {
            continue;
        }
        let mut d = step - carry;
        while d <= seg {
            let t = d / seg;
            let p = (
                w[0].0 + (w[1].0 - w[0].0) * t,
                w[0].1 + (w[1].1 - w[0].1) * t,
            );
            out.push([p.0, p.1]);
            d += step;
        }
        carry = (seg - (d - step)).max(0.0);
    }
    let last = gps[gps.len() - 1];
    if let Some(t) = out.last() {
        if (t[0] - last.0).abs() > 1e-9 || (t[1] - last.1).abs() > 1e-9 {
            out.push([last.0, last.1]);
        }
    }

    let n = out.len();
    if n < 3 {
        return Ok(json!({ "points": out, "distance_m": route_distance_m(&gps).round() as i64 }));
    }

    const MAX_LATERAL_M: f64 = 2.5;
    const MAX_TURN_RAD: f64 = std::f64::consts::FRAC_PI_4;

    let base = out.clone();

    let mut drift_target: f64 = 0.0;
    let mut drift: f64 = 0.0;
    for i in 1..n - 1 {
        if i == 1 || rng.random_range(0.0..1.0) < 0.06 {
            drift_target = rng.random_range(-MAX_LATERAL_M..MAX_LATERAL_M);
        }
        drift = drift * 0.88 + drift_target * 0.12;
        let jitter = (rng.random_range(-1.0..1.0) + rng.random_range(-1.0..1.0)) * 0.35;
        let mut off_m = (drift + jitter).clamp(-MAX_LATERAL_M, MAX_LATERAL_M);

        let seg_len = route_distance_m(&[(base[i - 1][0], base[i - 1][1]), (base[i][0], base[i][1])]);
        let cap = (seg_len * 0.12).max(0.2);
        off_m = off_m.clamp(-cap, cap);

        let lat_rad = base[i][1].to_radians();
        let m_lat = 111_320.0;
        let m_lng = 111_320.0 * lat_rad.cos().max(0.01);
        let th = ((base[i][1] - base[i - 1][1]) * m_lat).atan2((base[i][0] - base[i - 1][0]) * m_lng);
        let (nx, ny) = (-th.sin(), th.cos());
        let cand = [base[i][0] + nx * off_m / m_lng, base[i][1] + ny * off_m / m_lat];

        let mut k = 1.0f64;
        for _ in 0..8 {
            let p = [
                base[i][0] + nx * (off_m * k) / m_lng,
                base[i][1] + ny * (off_m * k) / m_lat,
            ];
            let b1 = bearing_rad(out[i - 1], p);
            let b2 = bearing_rad(p, base[i + 1]);
            if angle_diff(b1, b2) <= MAX_TURN_RAD {
                out[i] = p;
                break;
            }
            k *= 0.7;
            out[i] = p;
        }

        let b1 = bearing_rad(out[i - 1], out[i]);
        let b2 = bearing_rad(out[i], base[i + 1]);
        if angle_diff(b1, b2) > MAX_TURN_RAD {
            out[i] = base[i];
        }
        let _ = cand;
    }

    let smoothed = five_point_smooth(&out);
    let g2: Vec<(f64, f64)> = smoothed.iter().map(|p| (p[0], p[1])).collect();
    Ok(json!({ "points": smoothed, "distance_m": route_distance_m(&g2).round() as i64 }))
}

const REAL_TRACK_MAX_JUMP_M: f64 = 100.0;

pub fn clean_real_track(raw: &[[f64; 2]]) -> Vec<[f64; 2]> {
    if raw.len() < 3 {
        return raw.to_vec();
    }

    let med = median_filter_5(raw);
    let n = med.len();
    let mut out: Vec<[f64; 2]> = med.clone();

    let mut last = out[0];
    for i in 1..n {
        let d = route_distance_m(&[(last[0], last[1]), (out[i][0], out[i][1])]);
        if d > REAL_TRACK_MAX_JUMP_M {
            let lat_rad = last[1].to_radians();
            let m_lat = 111_320.0;
            let m_lng = 111_320.0 * lat_rad.cos().max(0.01);
            let bear = ((med[i][1] - last[1]) * m_lat).atan2((med[i][0] - last[0]) * m_lng);
            out[i][1] = last[1] + (bear.sin() * REAL_TRACK_MAX_JUMP_M) / m_lat;
            out[i][0] = last[0] + (bear.cos() * REAL_TRACK_MAX_JUMP_M) / m_lng;
        }
        last = out[i];
    }

    let alpha = 0.25f64;
    let mut ema = out.clone();
    for i in 1..n {
        ema[i][0] = ema[i - 1][0] * (1.0 - alpha) + out[i][0] * alpha;
        ema[i][1] = ema[i - 1][1] * (1.0 - alpha) + out[i][1] * alpha;
    }
    ema
}

fn median_filter_5(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let n = points.len();
    let mut out = points.to_vec();
    for i in 0..n {
        let lo = i.saturating_sub(2);
        let hi = (i + 2).min(n - 1);
        let mut xs: Vec<f64> = (lo..=hi).map(|k| points[k][0]).collect();
        let mut ys: Vec<f64> = (lo..=hi).map(|k| points[k][1]).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let m = (hi - lo + 1) / 2;
        out[i] = [xs[m], ys[m]];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_real_track_removes_jump() {

        let mut raw: Vec<[f64; 2]> = Vec::new();
        let base_lat = 35.2900;
        for i in 0..80 {
            raw.push([113.9456, base_lat + i as f64 * 0.0005]);
        }
        raw.insert(40, [113.9456, base_lat + 40.0 * 0.0005 + 0.005]);
        let cleaned = clean_real_track(&raw);
        for i in 1..cleaned.len() {
            let d = route_distance_m(&[(cleaned[i - 1][0], cleaned[i - 1][1]), (cleaned[i][0], cleaned[i][1])]);
            assert!(d <= 120.0, "清洗后仍存在长距离跳变 seg {}: {:.1}m @ {:?}", i, d, cleaned[i]);
        }
        assert!(cleaned.len() >= raw.len() - 2, "清洗过度删点");
    }
}

fn bearing_rad(a: [f64; 2], b: [f64; 2]) -> f64 {
    let lat_rad = a[1].to_radians();
    let m_lat = 111_320.0;
    let m_lng = 111_320.0 * lat_rad.cos().max(0.01);
    ((b[1] - a[1]) * m_lat).atan2((b[0] - a[0]) * m_lng)
}

fn angle_diff(a: f64, b: f64) -> f64 {
    let mut d = (a - b).abs();
    while d > std::f64::consts::PI {
        d = 2.0 * std::f64::consts::PI - d;
    }
    d
}

fn five_point_smooth(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let n = points.len();
    if n < 5 {
        return points.to_vec();
    }
    let mut out = points.to_vec();
    for i in 2..n - 2 {
        let w = [1.0f64, 2.0, 3.0, 2.0, 1.0];
        let mut sx = 0.0f64;
        let mut sy = 0.0f64;
        for (k, wk) in w.iter().enumerate() {
            sx += points[i - 2 + k][0] * wk;
            sy += points[i - 2 + k][1] * wk;
        }
        out[i] = [sx / 9.0, sy / 9.0];
    }
    out
}

pub fn upload(school_code: &str, ra_name: &str, _route_type: &str, points: &[[f64; 2]], name: &str) -> Result<Value, String> {
    if points.len() < 2 {
        return Err("至少需要 2 个点".into());
    }

    let file_name = if ra_name.trim().is_empty() {
        format!("{}.json", name)
    } else {
        format!("{}.json", ra_name)
    };
    let path = tracks_dir(school_code).join(&file_name);
    fs::write(&path, serde_json::to_string(&points.to_vec()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let gps: Vec<(f64, f64)> = points.iter().map(|p| (p[0], p[1])).collect();
    let dist = route_distance_m(&gps);
    Ok(json!({ "name": file_name, "distance_m": dist.round() as i64 }))
}
