#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_assignments)]
use crate::utils::tracks_dir;
use geographiclib_rs::{Geodesic, InverseGeodesic};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub type GPSPoint = (f64, f64);

#[derive(Clone)]
pub struct TrackPoint {
    pub longitude: f64,
    pub latitude: f64,
    pub run_mileage_m: f64,
    pub run_time: i64,
    pub run_step: i64,
    pub pace: f64,
    pub ts: String,
}

#[derive(Clone, Serialize)]
pub struct CardPoint {
    pub isFence: String,
    pub isMock: bool,
    pub point: String,
    pub runMileage: f64,
    pub runStatus: String,
    pub runStep: i64,
    pub runTime: i64,
    pub speed: String,
    pub ts: String,
}

#[derive(Clone, Serialize)]
pub struct RunSplitPayload {
    #[serde(rename = "StepNumber")]
    pub step_number: i64,
    pub a: i64,
    pub b: Option<serde_json::Value>,
    pub c: Option<serde_json::Value>,
    pub cardPointList: Vec<CardPoint>,
    pub crsRunRecordId: String,
    pub mileage: f64,
    pub orientationNum: i64,
    pub runSteps: f64,
    pub schoolId: String,
    pub simulateNum: i64,
    pub speeds: f64,
    pub strides: f64,
    pub times: i64,
    pub userName: String,
}

const DEFAULT_CHUNK_SIZE: usize = 61;
const DEFAULT_POINT_INTERVAL_S: i64 = 2;
const DEFAULT_PACE_RANGE_MIN_PER_KM: (f64, f64) = (5.25, 6.83);
const DEFAULT_CADENCE_RANGE_SPM: (f64, f64) = (164.0, 176.0);
const DEFAULT_STRIDE_RANGE: (f64, f64) = (0.9, 1.12);
const MAX_TRACK_CLOSURE_GAP_M: f64 = 5.0;
const MIN_CLOSURE_SEGMENT_M: f64 = 0.5;
const LAP_SHIFT_LAT: f64 = 0.0000065;
const LAP_SHIFT_LON: f64 = 0.0000065;
const WAVE_LAT: f64 = 0.000011;
const WAVE_LON: f64 = 0.000013;
const LOCAL_JITTER_LAT: f64 = 0.000003;
const LOCAL_JITTER_LON: f64 = 0.000003;
const GPS_SOLVER_TOLERANCE_M: f64 = 1e-8;
const GPS_SOLVER_FAST_ITERATIONS: usize = 12;
const GPS_SOLVER_FALLBACK_ITERATIONS: usize = 32;

const M_PER_DEG_LAT: f64 = 110_574.0;

const CONNECTOR_HANDLE_RATIO: f64 = 0.35;
const CONNECTOR_HANDLE_MIN_M: f64 = 3.0;
const CONNECTOR_HANDLE_MAX_M: f64 = 120.0;

const CONNECTOR_EXTRAPOLATE_RATIO: f64 = 0.4;
const CONNECTOR_EXTRAPOLATE_MIN_M: f64 = 10.0;
const CONNECTOR_EXTRAPOLATE_MAX_M: f64 = 60.0;

const CONNECTOR_EXTRAPOLATE_SKIP_MAX_M: f64 = 20.0;

const CONNECTOR_ARC_STEP_M: f64 = 5.0;

const CONNECTOR_BBOX_MARGIN_M: f64 = 6.0;

const CONNECTOR_SAMPLE_STEP_M: f64 = 6.0;
const CONNECTOR_MIN_POINTS: usize = 6;
const CONNECTOR_MAX_POINTS: usize = 80;

fn geod() -> Geodesic {
    Geodesic::wgs84()
}

fn clamp(v: f64, low: f64, high: f64) -> f64 {
    v.max(low).min(high)
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn compute_pace_min_per_km(distance_m: f64, duration_s: f64) -> f64 {
    if distance_m <= 0.0 || duration_s <= 0.0 {
        return 0.0;
    }
    (duration_s / 60.0) / (distance_m / 1000.0)
}

fn geodesic_distance_m(p1: GPSPoint, p2: GPSPoint) -> f64 {
    let r: f64 = geod().inverse(p1.1, p1.0, p2.1, p2.0);
    if !r.is_finite() || r < 0.0 {
        panic!("无法计算有效测地距离: {:?} -> {:?}", p1, p2);
    }
    r
}

pub fn route_distance_m(points: &[GPSPoint]) -> f64 {
    let mut total = 0.0;
    for w in points.windows(2) {
        total += geodesic_distance_m(w[0], w[1]);
    }
    total
}

fn choose_json_file_by_ra_name(ra_name: &str, json_dir: &str) -> String {
    let dir = Path::new(json_dir);
    if !dir.is_dir() {
        panic!("轨迹目录不存在: {}，请为该学校配置轨迹文件", json_dir);
    }
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let p = e.path();
            let s = p.file_name()?.to_string_lossy().to_lowercase();
            if s.ends_with(".json") {
                Some(p.file_name()?.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    if files.is_empty() {
        panic!("轨迹目录为空: {}", json_dir);
    }
    files.sort_by_key(|s| s.to_lowercase());

    let norm = ra_name.trim().to_lowercase();
    let exact: Vec<&String> = files.iter().filter(|f| Path::new(f).file_stem().unwrap().to_string_lossy().to_lowercase() == norm).collect();
    if !exact.is_empty() {
        return dir.join(exact[0].clone()).to_string_lossy().to_string();
    }
    let contained: Vec<&String> = files.iter().filter(|f| {
        let stem = Path::new(f).file_stem().unwrap().to_string_lossy().to_lowercase();
        norm.contains(&stem)
    }).collect();
    if !contained.is_empty() {
        let longest = contained.iter().map(|f| Path::new(f.as_str()).file_stem().unwrap().to_string_lossy().len()).max().unwrap();
        let longest_matches: Vec<&&String> = contained.iter().filter(|f| Path::new(f.as_str()).file_stem().unwrap().to_string_lossy().len() == longest).collect();
        if longest_matches.len() == 1 {
            return dir.join(longest_matches[0].to_string()).to_string_lossy().to_string();
        }
        panic!("跑区名称匹配到多个轨迹文件: {}", ra_name);
    }
    for fb in ["通用.json", "default.json"] {
        if files.iter().any(|f| f == fb) {
            return dir.join(fb).to_string_lossy().to_string();
        }
    }
    panic!(
        "未找到与跑区名称匹配的轨迹文件: {}（可先「导入轨迹」再「生成轨迹」，或在轨迹绘制页保存）",
        ra_name
    );
}

fn load_track_points_from_json(json_path: &str) -> Vec<GPSPoint> {
    let raw: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(json_path).unwrap_or_else(|_| panic!("轨迹文件不存在: {}", json_path)),
    )
    .unwrap();
    let arr = raw.as_array().expect("轨迹文件顶层必须为 list");
    if arr.len() < 2 {
        panic!("轨迹点数量不足: {}", json_path);
    }
    let mut points: Vec<GPSPoint> = Vec::new();
    for (idx, item) in arr.iter().enumerate() {
        let pair = item.as_array().expect("轨迹点格式错误");
        let lon = pair[0].as_f64().unwrap();
        let lat = pair[1].as_f64().unwrap();
        if !lon.is_finite() || !lat.is_finite() || !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
            panic!("第 {} 个点坐标非法", idx);
        }
        let p = (lon, lat);
        if points.last() != Some(&p) {
            points.push(p);
        }
    }
    if points.len() < 2 {
        panic!("轨迹有效点数量不足: {}", json_path);
    }
    points
}

fn ensure_closed_loop(points: &[GPSPoint]) -> Vec<GPSPoint> {
    if points.len() < 3 {
        panic!("闭环轨迹至少需要 3 个有效点");
    }
    let mut closed = points.to_vec();
    let closure_gap = geodesic_distance_m(*closed.last().unwrap(), closed[0]);
    if closure_gap > MAX_TRACK_CLOSURE_GAP_M {
        panic!("轨迹未闭合，首尾相距 {:.2} 米，超过允许值", closure_gap);
    }
    if closure_gap > 0.0 {
        closed.push(closed[0]);
    } else {
        let _ = closed.pop();
        closed.push(closed[0]);
    }
    while closed.len() >= 4 && geodesic_distance_m(closed[closed.len() - 2], closed[0]) < MIN_CLOSURE_SEGMENT_M {
        closed.remove(closed.len() - 2);
    }
    closed
}

fn meters_per_deg_lon(lat: f64) -> f64 {
    111_320.0 * lat.to_radians().cos()
}

fn sanitize_open_route(points: &[GPSPoint]) -> Vec<GPSPoint> {
    let mut fwd: Vec<GPSPoint> = Vec::new();
    for &p in points {
        if fwd.last() != Some(&p) {
            fwd.push(p);
        }
    }
    if fwd.len() >= 3 && fwd.last() == Some(&fwd[0]) {
        fwd.pop();
    }
    if fwd.len() < 2 {
        panic!("开放路线至少需要 2 个不同坐标点");
    }
    let len: f64 = (1..fwd.len()).map(|i| geodesic_distance_m(fwd[i - 1], fwd[i])).sum();
    if len <= 0.0 {
        panic!("开放路线长度必须大于 0");
    }
    fwd
}

fn endpoint_tangent_m(points: &[GPSPoint], from: usize, forward: bool) -> (f64, f64) {
    let n = points.len();
    let mut i = from;
    loop {
        let (a, b) = if forward {
            if i + 1 >= n {
                break;
            }
            (points[i], points[i + 1])
        } else {
            if i == 0 {
                break;
            }
            (points[i - 1], points[i])
        };
        let dx = (b.0 - a.0) * meters_per_deg_lon(a.1);
        let dy = (b.1 - a.1) * M_PER_DEG_LAT;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 1e-9 {
            return (dx / len, dy / len);
        }
        if forward {
            i += 1;
        } else {
            i -= 1;
        }
    }
    (0.0, 1.0)
}

fn cubic_bezier_xy(
    p0: (f64, f64),
    c1: (f64, f64),
    c2: (f64, f64),
    p3: (f64, f64),
    t: f64,
) -> (f64, f64) {
    let u = 1.0 - t;
    let b0 = u * u * u;
    let b1 = 3.0 * u * u * t;
    let b2 = 3.0 * u * t * t;
    let b3 = t * t * t;
    (
        b0 * p0.0 + b1 * c1.0 + b2 * c2.0 + b3 * p3.0,
        b0 * p0.1 + b1 * c1.1 + b2 * c2.1 + b3 * p3.1,
    )
}

fn drop_degenerate_points(points: Vec<GPSPoint>) -> Vec<GPSPoint> {
    let mut out: Vec<GPSPoint> = Vec::with_capacity(points.len());
    for p in points {
        if let Some(&q) = out.last() {
            if geodesic_distance_m(q, p) < MIN_CLOSURE_SEGMENT_M {
                continue;
            }
        }
        out.push(p);
    }
    out
}

fn close_open_route(points: &[GPSPoint]) -> Vec<GPSPoint> {

    let fwd_full = sanitize_open_route(points);
    let fwd = trim_to_first_loop(&fwd_full).unwrap_or(fwd_full);
    let first = fwd[0];
    let last = *fwd.last().unwrap();
    let span = geodesic_distance_m(last, first);

    let base_len: f64 = (1..fwd.len()).map(|i| geodesic_distance_m(fwd[i - 1], fwd[i])).sum();
    if span > 20.0 {
        eprintln!(
            "[closeRoute] 底图闭合: 首尾间距={:.1}m 底图长度={:.1}m 占比={:.0}%{}",
            span,
            base_len,
            if base_len > 0.0 { span / base_len * 100.0 } else { 100.0 },
            if base_len > 0.0 && span > base_len * 0.25 { " ⚠️ 首尾跨度过大，衔接段将横穿场地，建议导入完整一圈的轨迹" } else { "" },
        );
    }
    if span <= MAX_TRACK_CLOSURE_GAP_M {

        return if fwd.len() >= 3 {
            ensure_closed_loop(&fwd)
        } else {
            vec![fwd[0], fwd[1], fwd[0]]
        };
    }
    let mx = meters_per_deg_lon(first.1);
    let my = M_PER_DEG_LAT;

    let bbox = route_bbox(&fwd, CONNECTOR_BBOX_MARGIN_M, mx, my);

    let ext_len = if span <= CONNECTOR_EXTRAPOLATE_SKIP_MAX_M {
        0.0
    } else {
        (span * CONNECTOR_EXTRAPOLATE_RATIO).clamp(CONNECTOR_EXTRAPOLATE_MIN_M, CONNECTOR_EXTRAPOLATE_MAX_M)
    };

    let arc_end = extrapolate_arc(&fwd, ext_len, mx, my, Some(bbox));

    let mut rev = fwd.clone();
    rev.reverse();
    let mut arc_into_start = extrapolate_arc(&rev, ext_len, mx, my, Some(bbox));
    arc_into_start.reverse();
    let mut merged = fwd.clone();
    merged.extend_from_slice(&arc_end);

    let mut end_chain = vec![last];
    end_chain.extend_from_slice(&arc_end);
    let (q0, t0) = match arc_end.last() {
        Some(&q) => (q, endpoint_tangent_m(&end_chain, end_chain.len() - 1, false)),
        None => (last, endpoint_tangent_m(&fwd, fwd.len() - 1, false)),
    };
    let mut start_chain = arc_into_start.clone();
    start_chain.push(first);
    let (q1, t1) = match arc_into_start.first() {
        Some(&q) => (q, endpoint_tangent_m(&start_chain, 0, true)),
        None => (first, endpoint_tangent_m(&fwd, 0, true)),
    };
    let gap = geodesic_distance_m(q0, q1);
    if gap >= MIN_CLOSURE_SEGMENT_M * 2.0 {
        let to_xy = |p: GPSPoint| ((p.0 - first.0) * mx, (p.1 - first.1) * my);
        let p0 = to_xy(q0);
        let p3 = to_xy(q1);
        let handle = (gap * CONNECTOR_HANDLE_RATIO).clamp(CONNECTOR_HANDLE_MIN_M, CONNECTOR_HANDLE_MAX_M);

        let bbox_xy = (
            (bbox.0 - first.0) * mx,
            (bbox.1 - first.1) * my,
            (bbox.2 - first.0) * mx,
            (bbox.3 - first.1) * my,
        );
        let c1 = clamp_to_bbox((p0.0 + t0.0 * handle, p0.1 + t0.1 * handle), bbox_xy);
        let c2 = clamp_to_bbox((p3.0 - t1.0 * handle, p3.1 - t1.1 * handle), bbox_xy);
        let n = ((gap / CONNECTOR_SAMPLE_STEP_M).round() as usize).clamp(CONNECTOR_MIN_POINTS, CONNECTOR_MAX_POINTS);
        for k in 1..=n {
            let t = k as f64 / (n as f64 + 1.0);
            let (x, y) = cubic_bezier_xy(p0, c1, c2, p3, t);
            merged.push((first.0 + x / mx, first.1 + y / my));
        }
    }
    merged.extend_from_slice(&arc_into_start);
    merged.push(first);
    let mut closed = drop_degenerate_points(merged);
    let anchor = closed[0];

    if closed.last() != Some(&anchor) {
        closed.push(anchor);
    }
    while closed.len() >= 4 && geodesic_distance_m(closed[closed.len() - 2], anchor) < MIN_CLOSURE_SEGMENT_M {
        closed.remove(closed.len() - 2);
    }
    if closed.len() < 3 {
        panic!("闭环轨迹至少需要 3 个有效点");
    }
    closed
}

struct MetricPolyline {
    xy: Vec<(f64, f64)>,
    cum: Vec<f64>,
    total: f64,
}

fn build_metric_polyline(points: &[GPSPoint]) -> MetricPolyline {
    let origin = points[0];
    let mx = meters_per_deg_lon(origin.1);
    let my = M_PER_DEG_LAT;
    let mut xy: Vec<(f64, f64)> = Vec::with_capacity(points.len());
    let mut cum: Vec<f64> = Vec::with_capacity(points.len());
    let mut total = 0.0f64;
    for &(lon, lat) in points {
        xy.push(((lon - origin.0) * mx, (lat - origin.1) * my));
        cum.push(total);
        if xy.len() >= 2 {
            let dx = xy[xy.len() - 1].0 - xy[xy.len() - 2].0;
            let dy = xy[xy.len() - 1].1 - xy[xy.len() - 2].1;
            total += (dx * dx + dy * dy).sqrt();
            *cum.last_mut().unwrap() = total;
        }
    }
    MetricPolyline { xy, cum, total }
}

fn metric_point_at_s(pl: &MetricPolyline, s: f64) -> (f64, f64) {
    let s = s.clamp(0.0, pl.total);
    let right = match pl.cum.binary_search_by(|c| c.partial_cmp(&s).unwrap()) {
        Ok(i) => i,
        Err(i) => i,
    };
    if right == 0 {
        return pl.xy[0];
    }
    let left = right - 1;
    let seg = (pl.cum[right] - pl.cum[left]).max(1e-9);
    let ratio = (s - pl.cum[left]) / seg;
    (
        pl.xy[left].0 + (pl.xy[right].0 - pl.xy[left].0) * ratio,
        pl.xy[left].1 + (pl.xy[right].1 - pl.xy[left].1) * ratio,
    )
}

fn detect_loop_period(pl: &MetricPolyline) -> Option<f64> {
    if pl.total < 400.0 {
        return None;
    }
    let samples = 40usize;
    let eval = |p: f64| -> f64 {
        let mut sum = 0.0;
        for k in 0..samples {
            let s = (k as f64 / (samples - 1) as f64) * (pl.total - p);
            let a = metric_point_at_s(pl, s);
            let b = metric_point_at_s(pl, s + p);
            let dx = a.0 - b.0;
            let dy = a.1 - b.1;
            sum += (dx * dx + dy * dy).sqrt();
        }
        sum / samples as f64
    };

    let p_min = (pl.total * 0.2).max(200.0);
    let p_max = pl.total * 0.9;
    let mut p = p_min;
    while p <= p_max {
        if eval(p) <= (p * 0.1).min(25.0) {
            let mut refined = p;
            let mut refined_err = eval(p);
            let mut q = (p - 6.0).max(p_min);
            while q <= p + 6.0 {
                let err = eval(q);
                if err < refined_err {
                    refined_err = err;
                    refined = q;
                }
                q += 0.5;
            }
            return Some(refined);
        }
        p += 3.0;
    }
    None
}

fn trim_to_first_loop(points: &[GPSPoint]) -> Option<Vec<GPSPoint>> {
    if points.len() < 8 {
        return None;
    }
    let pl = build_metric_polyline(points);
    let period = detect_loop_period(&pl)?;

    let mut best_phi = 0.0f64;
    let mut best_res = f64::MAX;
    let mut phi = 0.0f64;
    while phi <= pl.total - period {
        let a = metric_point_at_s(&pl, phi);
        let b = metric_point_at_s(&pl, phi + period);
        let dx = a.0 - b.0;
        let dy = a.1 - b.1;
        let res = (dx * dx + dy * dy).sqrt();
        if res < best_res {
            best_res = res;
            best_phi = phi;
        }
        phi += 2.0;
    }
    let s0 = best_phi;
    let s1 = best_phi + period;
    let mx = meters_per_deg_lon(points[0].1);
    let my = M_PER_DEG_LAT;
    let mut out: Vec<GPSPoint> = Vec::new();

    let (x0, y0) = metric_point_at_s(&pl, s0);
    out.push((points[0].0 + x0 / mx, points[0].1 + y0 / my));
    for (i, &c) in pl.cum.iter().enumerate() {
        if c > s0 && c <= s1 {
            let p = points[i];

            if geodesic_distance_m(*out.last().unwrap(), p) >= MIN_CLOSURE_SEGMENT_M {
                out.push(p);
            }
        }
    }

    let (x1, y1) = metric_point_at_s(&pl, s1);
    let tail = (points[0].0 + x1 / mx, points[0].1 + y1 / my);
    if geodesic_distance_m(*out.last().unwrap(), tail) >= MIN_CLOSURE_SEGMENT_M {
        out.push(tail);
    }
    if out.len() < 3 {
        return None;
    }
    eprintln!(
        "[closeRoute] 多圈底图: 总长={:.0}m 检测周期={:.1}m 截取相位={:.0}m 圈间残差={:.1}m 点数={}",
        pl.total, period, best_phi, best_res, out.len()
    );
    Some(out)
}

fn route_bbox(points: &[GPSPoint], margin_m: f64, mx: f64, my: f64) -> (f64, f64, f64, f64) {
    let mut min_lon = f64::MAX;
    let mut min_lat = f64::MAX;
    let mut max_lon = f64::MIN;
    let mut max_lat = f64::MIN;
    for &(lon, lat) in points {
        min_lon = min_lon.min(lon);
        max_lon = max_lon.max(lon);
        min_lat = min_lat.min(lat);
        max_lat = max_lat.max(lat);
    }
    let dlon = margin_m / mx;
    let dlat = margin_m / my;
    (min_lon - dlon, min_lat - dlat, max_lon + dlon, max_lat + dlat)
}

fn clamp_to_bbox(p: GPSPoint, bbox: (f64, f64, f64, f64)) -> GPSPoint {
    (p.0.clamp(bbox.0, bbox.2), p.1.clamp(bbox.1, bbox.3))
}

fn extrapolate_arc(
    points: &[GPSPoint],
    length_m: f64,
    mx: f64,
    my: f64,
    bbox: Option<(f64, f64, f64, f64)>,
) -> Vec<GPSPoint> {
    if points.len() < 3 || length_m <= 0.0 {
        return Vec::new();
    }
    let origin = points[0];
    let to_xy = |p: GPSPoint| ((p.0 - origin.0) * mx, (p.1 - origin.1) * my);
    let a = to_xy(points[points.len() - 3]);
    let b = to_xy(points[points.len() - 2]);
    let c = to_xy(points[points.len() - 1]);
    let mut th = (c.1 - b.1).atan2(c.0 - b.0);
    let det = 2.0 * (a.0 * (b.1 - c.1) + b.0 * (c.1 - a.1) + c.0 * (a.1 - b.1));
    let n = (length_m / CONNECTOR_ARC_STEP_M).round().max(1.0) as usize;

    let dth = if det.abs() < 1e-9 {
        0.0
    } else {
        let a2 = a.0 * a.0 + a.1 * a.1;
        let b2 = b.0 * b.0 + b.1 * b.1;
        let c2 = c.0 * c.0 + c.1 * c.1;
        let ux = (a2 * (b.1 - c.1) + b2 * (c.1 - a.1) + c2 * (a.1 - b.1)) / det;
        let uy = (a2 * (c.0 - b.0) + b2 * (a.0 - c.0) + c2 * (b.0 - a.0)) / det;
        let r = ((b.0 - ux).powi(2) + (b.1 - uy).powi(2)).sqrt();
        if !r.is_finite() || r < 1.0 {
            0.0
        } else {
            let cross = (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0);
            CONNECTOR_ARC_STEP_M / r * cross.signum()
        }
    };
    let mut pos = c;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        th += dth;
        pos = (pos.0 + th.cos() * CONNECTOR_ARC_STEP_M, pos.1 + th.sin() * CONNECTOR_ARC_STEP_M);
        let ll = (origin.0 + pos.0 / mx, origin.1 + pos.1 / my);
        if let Some(bb) = bbox {
            if ll.0 < bb.0 || ll.0 > bb.2 || ll.1 < bb.1 || ll.1 > bb.3 {
                break;
            }
        }
        out.push(ll);
    }
    out
}

fn prepare_route_points(points: &[GPSPoint], route_mode: &str) -> Vec<GPSPoint> {
    let _ = route_mode;
    close_open_route(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heading_deg(a: GPSPoint, b: GPSPoint) -> f64 {
        let dx = (b.0 - a.0) * meters_per_deg_lon(a.1);
        let dy = (b.1 - a.1) * M_PER_DEG_LAT;
        dy.atan2(dx).to_degrees()
    }

    fn turn_deg(a: GPSPoint, b: GPSPoint, c: GPSPoint) -> f64 {
        let mut d = heading_deg(b, c) - heading_deg(a, b);
        while d > 180.0 {
            d -= 360.0;
        }
        while d < -180.0 {
            d += 360.0;
        }
        d
    }

    #[test]
    fn close_open_route_closed_without_backtrack() {
        let open: Vec<GPSPoint> = vec![
            (120.100000, 30.100000),
            (120.100900, 30.100000),
            (120.100900, 30.100450),
            (120.100000, 30.100450),
        ];
        let closed = close_open_route(&open);
        assert_eq!(closed[0], *closed.last().unwrap(), "闭环首尾必须重合");
        assert!(closed.len() > open.len(), "应插入续接段");
        for i in 1..closed.len() {
            assert!(
                geodesic_distance_m(closed[i - 1], closed[i]) > 0.0,
                "存在零长度段 @{}",
                i
            );
        }
        for i in 1..closed.len() - 1 {
            let t = turn_deg(closed[i - 1], closed[i], closed[i + 1]);
            assert!(t.abs() <= 120.0, "出现掉头/急转：{:.1}° @{}", t, i);
        }
        let base_len: f64 = (1..open.len()).map(|i| geodesic_distance_m(open[i - 1], open[i])).sum();
        let loop_len = build_cumulative_distances(&closed)[closed.len() - 1];
        assert!(loop_len > base_len, "环长应大于原底图长度");

        let mx = meters_per_deg_lon(open[0].1);
        let bbox = route_bbox(&open, CONNECTOR_BBOX_MARGIN_M, mx, M_PER_DEG_LAT);
        for (i, &(lon, lat)) in closed.iter().enumerate() {
            assert!(
                lon >= bbox.0 && lon <= bbox.2 && lat >= bbox.1 && lat <= bbox.3,
                "点 {} 超出底图范围：({:.6},{:.6})",
                i,
                lon,
                lat
            );
        }
    }

    #[test]
    fn generate_full_run_without_drift_line() {
        let json_path = concat!(
            "D:/Development/MyProject/yunsport/moli-cloud-sports-tauri/src-tauri/data/",
            "tracks/219/东操场-大一女生跑.json"
        );
        let raw = std::fs::read_to_string(json_path).expect("读取底图失败");
        let coords: Vec<Vec<f64>> =
            serde_json::from_str(&raw).expect("底图 json 解析失败");
        let base: Vec<GPSPoint> = coords.iter().map(|c| (c[0], c[1])).collect();
        for seed in 0..30 {
            let pts = generate_track_points_from_base(1554, &base, 2, (0.9, 1.12), (5.25, 6.83), (164.0, 176.0), Some(seed), None);
            assert!(pts.len() > 100, "seed {} 生成点数异常: {}", seed, pts.len());
            let coords: Vec<GPSPoint> = pts.iter().map(|p| (p.longitude, p.latitude)).collect();
            let mut bad_seg = 0;
            let mut bad_turn = 0;
            for i in 1..coords.len() {
                let d = geodesic_distance_m(coords[i - 1], coords[i]);
                if d > 15.0 {
                    eprintln!("seed {} 异常长段 seg {}: {:.1} m @ {:?}", seed, i, d, coords[i]);
                    bad_seg += 1;
                }
            }
            for i in 1..coords.len() - 1 {
                let d1 = geodesic_distance_m(coords[i - 1], coords[i]);
                let d2 = geodesic_distance_m(coords[i], coords[i + 1]);
                if d1 < 1.0 || d2 < 1.0 {
                    continue;
                }
                let t = turn_deg(coords[i - 1], coords[i], coords[i + 1]);
                if t.abs() > 90.0 {
                    eprintln!("seed {} 折返 seg {}: {:.1}° @ {:?}", seed, i, t, coords[i]);
                    bad_turn += 1;
                }
            }
            assert_eq!(bad_seg, 0, "seed {} 存在横穿直线段", seed);
            assert_eq!(bad_turn, 0, "seed {} 存在折返/大转向", seed);
        }
    }
}

fn build_cumulative_distances(points: &[GPSPoint]) -> Vec<f64> {
    let mut cum = vec![0.0f64];
    let mut total = 0.0;
    for i in 1..points.len() {
        total += geodesic_distance_m(points[i - 1], points[i]);
        cum.push(total);
    }
    if total <= 0.0 {
        panic!("轨迹总长度必须大于 0");
    }
    cum
}

fn interpolate_point_along_polyline(points: &[GPSPoint], cum: &[f64], target: f64) -> GPSPoint {
    let total = cum[cum.len() - 1];
    if total <= 0.0 {
        return points[0];
    }
    let target = clamp(target, 0.0, total);
    let right = match cum.binary_search_by(|&x| x.partial_cmp(&target).unwrap()) {
        Ok(i) => i,
        Err(i) => i,
    };
    if right <= 0 {
        return points[0];
    }
    if right >= points.len() {
        return *points.last().unwrap();
    }
    let left = right - 1;
    let seg = (cum[right] - cum[left]).max(1e-9);
    let ratio = (target - cum[left]) / seg;
    let p1 = points[left];
    let p2 = points[right];
    (p1.0 + (p2.0 - p1.0) * ratio, p1.1 + (p2.1 - p1.1) * ratio)
}

fn advance_on_closed_loop_by_gps_distance(
    points: &[GPSPoint],
    cum: &[f64],
    current_loop_distance_m: f64,
    target_gps_distance_m: f64,
) -> (f64, GPSPoint, f64) {
    let loop_len = cum[cum.len() - 1];
    if loop_len <= 0.0 {
        panic!("闭环轨迹总长度必须大于 0");
    }
    let current = current_loop_distance_m % loop_len;
    let start = interpolate_point_along_polyline(points, cum, current);
    let probe_step = (target_gps_distance_m / 4.0).max(0.05);
    let mut low = 0.0;
    let mut high = probe_step;
    let max_advance = loop_len / 2.0;
    while high <= max_advance {
        let cand_loop = (current + high) % loop_len;
        let cand = interpolate_point_along_polyline(points, cum, cand_loop);
        if geodesic_distance_m(start, cand) >= target_gps_distance_m {
            break;
        }
        low = high;
        high += probe_step;
    }
    if high > max_advance {
        panic!("轨迹尺寸过小，无法生成相距 {:.3} 米的连续采样点", target_gps_distance_m);
    }
    for _ in 0..40 {
        let mid = (low + high) / 2.0;
        let mloop = (current + mid) % loop_len;
        let mp = interpolate_point_along_polyline(points, cum, mloop);
        if geodesic_distance_m(start, mp) < target_gps_distance_m {
            low = mid;
        } else {
            high = mid;
        }
    }
    let next_loop = (current + high) % loop_len;
    let next = interpolate_point_along_polyline(points, cum, next_loop);
    let actual = geodesic_distance_m(start, next);
    (next_loop, next, actual)
}

fn pace_range_to_speed_range(pace: (f64, f64)) -> (f64, f64) {
    let (lo, hi) = pace;
    let min_s = 1000.0 / (hi * 60.0);
    let max_s = 1000.0 / (lo * 60.0);
    (min_s, max_s)
}

struct RunnerProfile {
    base_speed_ms: f64,
    stride: f64,
    mid_dip: f64,
    variability: f64,
    phase: f64,
    cadence_min: f64,
    cadence_max: f64,
}

fn init_runner_profile(
    stride_range: (f64, f64),
    pace: (f64, f64),
    cadence: (f64, f64),
    rng: &mut StdRng,
) -> RunnerProfile {
    let (plo, phi) = pace;
    let base_pace = rng.random_range(plo..(plo + phi) / 2.0);
    let base_speed = 1000.0 / (base_pace * 60.0);
    let (cmin, cmax) = cadence;

    let target_cadence = rng.random_range(cmin..cmax.max(cmin + 1.0));
    let stride = (base_speed * 60.0 / target_cadence).clamp(stride_range.0, stride_range.1);
    RunnerProfile {
        base_speed_ms: base_speed,
        stride,
        mid_dip: rng.random_range(0.04..0.07),
        variability: rng.random_range(0.015..0.03),
        phase: rng.random_range(0.0..std::f64::consts::PI * 2.0),
        cadence_min: cmin,
        cadence_max: cmax,
    }
}

fn simulate_speed_ms(
    p: &RunnerProfile,
    progress: f64,
    point_no: i64,
    lap_progress: f64,
    speed_range: (f64, f64),
    rng: &mut StdRng,
) -> f64 {
    let (min_s, max_s) = speed_range;
    let trend = 1.0 - p.mid_dip * (progress * std::f64::consts::PI).sin();
    let wave1 = (point_no as f64 / 9.0 + p.phase).sin() * p.variability;
    let wave2 = (point_no as f64 / 20.0 + p.phase * 0.55).sin() * (p.variability * 0.55);
    let lap_wave = (lap_progress * std::f64::consts::PI * 2.0).sin() * 0.008;
    let noise = rng.random_range(-0.008..0.008);
    let s = p.base_speed_ms * trend * (1.0 + wave1 + wave2 + lap_wave + noise);
    clamp(s, min_s, max_s)
}

fn simulate_step_increment(
    speed_ms: f64,
    duration_s: i64,
    p: &RunnerProfile,
    previous: Option<f64>,
    rng: &mut StdRng,
) -> f64 {
    let expected = speed_ms * duration_s as f64 / p.stride;
    let mut noisy = expected + rng.random_range(-0.1..0.1);
    if let Some(prev) = previous {
        noisy = prev * 0.70 + noisy * 0.30;
    }

    let lo = p.cadence_min * duration_s as f64 / 60.0;
    let hi = p.cadence_max.max(p.cadence_min) * duration_s as f64 / 60.0;
    noisy.clamp(lo, hi).max(0.0)
}

fn weighted_choice(rng: &mut StdRng, items: &[i64], weights: &[f64]) -> i64 {
    let total: f64 = weights.iter().sum();
    let mut r = rng.random_range(0.0..total);
    for (i, w) in weights.iter().enumerate() {
        if r < *w {
            return items[i];
        }
        r -= *w;
    }
    items[items.len() - 1]
}

fn apply_gps_noise(
    prev: Option<GPSPoint>,
    prev_prev: Option<GPSPoint>,
    cur: GPSPoint,
    rng: &mut StdRng,
    drift: &mut f64,
) -> GPSPoint {

    *drift = (*drift * 0.92 + rng.random_range(-0.6..0.6) * 0.000004).clamp(-0.00004, 0.00004);

    let (nx, ny) = match prev {
        Some(p) => {
            let th = (cur.1 - p.1).atan2(cur.0 - p.0);
            (-th.sin(), th.cos())
        }
        None => (0.0, 1.0),
    };

    let jitter = (rng.random_range(-1.0..1.0) + rng.random_range(-1.0..1.0)) * 0.000007;

    let outlier = if rng.random_range(0.0..1.0) < 0.015 {
        rng.random_range(0.00003..0.000055) * if rng.random_bool(0.5) { 1.0 } else { -1.0 }
    } else {
        0.0
    };

    let off = (jitter + *drift + outlier).clamp(-0.000_022_5, 0.000_022_5);
    let r6 = |v: f64| (v * 1e6).round() / 1e6;

    let m_lat = 111_320.0_f64;
    let m_lng = 111_320.0 * cur.1.to_radians().cos().max(0.01);
    let max_turn = 45.0_f64.to_radians();
    let mut k = 1.0_f64;
    loop {
        let vl = r6(cur.0 + nx * off * k);
        let va = r6(cur.1 + ny * off * k);
        if !(-180.0..=180.0).contains(&vl) || !(-90.0..=90.0).contains(&va) {
            return (r6(cur.0), r6(cur.1));
        }
        let ok = match (prev, prev_prev) {
            (Some(p), Some(pp)) => {
                let d1 = geodesic_distance_m(pp, p);
                let d2 = geodesic_distance_m(p, (vl, va));
                if d1 < 1.0 || d2 < 1.0 {
                    true
                } else {
                    let b1 = ((p.1 - pp.1) * m_lat).atan2((p.0 - pp.0) * m_lng);
                    let b2 = ((va - p.1) * m_lat).atan2((vl - p.0) * m_lng);
                    let mut d = (b1 - b2).abs();
                    if d > std::f64::consts::PI {
                        d = std::f64::consts::TAU - d;
                    }
                    d <= max_turn
                }
            }
            _ => true,
        };
        if ok {
            return (vl, va);
        }
        if k < 0.15 {
            return (r6(cur.0), r6(cur.1));
        }
        k *= 0.6;
    }
}

fn build_fixed_interval_distance_plan(
    target_distance: f64,
    _point_interval_s: i64,
    profile: &RunnerProfile,
    speed_range: (f64, f64),
    movement_cycle_length: f64,
    rng: &mut StdRng,
) -> (Vec<i64>, Vec<f64>) {
    let (min_s, max_s) = speed_range;
    let max_iter = (target_distance / (min_s * 2.0)).ceil() as i64 + 2;
    let mut intervals = Vec::new();
    let mut raw = Vec::new();
    let mut cumulative = 0.0;
    let mut movement_progress = 0.0;
    let items = [2i64, 3, 4];
    let w = [3.0, 5.0, 1.0];

    let mut stamina = 1.0;
    let mut stamina_target = 1.0;
    for point_no in 0..max_iter {
        let interval_s = if point_no == 0 { 6 } else { weighted_choice(rng, &items, &w) };
        intervals.push(interval_s);
        let progress = cumulative / target_distance;
        let lap_progress = movement_progress / movement_cycle_length;
        if rng.random_range(0.0..1.0) < 0.05 {
            stamina_target = rng.random_range(0.93..1.07);
        }
        stamina = stamina * 0.97 + stamina_target * 0.03;
        let speed = simulate_speed_ms(profile, progress, point_no, lap_progress, speed_range, rng) * stamina;
        let mut seg = speed * interval_s as f64;
        seg *= rng.random_range(0.996..1.004);
        seg = clamp(seg, min_s * interval_s as f64, max_s * interval_s as f64);
        if !seg.is_finite() || seg <= 0.0 {
            panic!("距离规划产生了非有限或非正运动段");
        }
        raw.push(seg);
        cumulative += seg;
        movement_progress = (movement_progress + seg) % movement_cycle_length;
        if cumulative >= target_distance - 1e-9 {
            break;
        }
    }
    if raw.is_empty() {
        panic!("距离规划未达到目标");
    }
    let excess = cumulative - target_distance;
    let adjustable: f64 = (0..raw.len()).map(|i| (raw[i] - min_s * intervals[i] as f64).max(0.0)).sum();
    let adjusted = if excess <= adjustable + 1e-9 {
        if adjustable > 0.0 {
            raw.iter().enumerate().map(|(i, &d)| d - excess * (d - min_s * intervals[i] as f64).max(0.0) / adjustable).collect()
        } else {
            raw.clone()
        }
    } else {
        let scale = target_distance / cumulative;
        raw.iter().map(|&d| d * scale).collect()
    };
    let residual = target_distance - adjusted.iter().sum::<f64>();
    let mut adj = adjusted;
    let last = adj.len() - 1;
    adj[last] += residual;
    (intervals, adj)
}

fn generate_track_points(
    target_distance: i64,
    ra_name: &str,
    json_dir: &str,
    point_interval_s: i64,
    stride_range: (f64, f64),
    pace_range: (f64, f64),
    cadence_range: (f64, f64),
    random_seed: Option<i64>,
    start_epoch_s: Option<i64>,
    route_mode: &str,
) -> Vec<TrackPoint> {
    if point_interval_s != DEFAULT_POINT_INTERVAL_S {
        panic!("软件固定每 {} 秒取点", DEFAULT_POINT_INTERVAL_S);
    }
    if target_distance <= 0 {
        panic!("target_distance 必须大于 0");
    }
    let json_path = choose_json_file_by_ra_name(ra_name, json_dir);
    let base_route = prepare_route_points(&load_track_points_from_json(&json_path), route_mode);
    generate_track_points_from_base(
        target_distance,
        &base_route,
        point_interval_s,
        stride_range,
        pace_range,
        cadence_range,
        random_seed,
        start_epoch_s,
    )
}

fn resample_closed_even(points: &[GPSPoint], step_m: f64) -> Vec<GPSPoint> {
    let n = points.len();
    if n < 2 || step_m <= 0.0 {
        return points.to_vec();
    }
    let mut out: Vec<GPSPoint> = vec![points[0]];
    let mut target = step_m;
    let mut global = 0.0;
    let mut edge = 0usize;
    loop {
        let a = points[edge % n];
        let b = points[(edge + 1) % n];
        let seg = geodesic_distance_m(a, b);
        if seg <= 1e-9 {
            edge += 1;
            if edge >= n {
                break;
            }
            continue;
        }
        while target <= global + seg {
            let t = (target - global) / seg;
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            target += step_m;
        }
        global += seg;
        edge += 1;
        if edge >= n {
            break;
        }
    }

    if out.len() > 1 {
        let first = out[0];
        let last = out[out.len() - 1];
        if geodesic_distance_m(first, last) < step_m * 0.4 {
            out.pop();
        }
    }
    out
}

fn catmull_rom_closed(ctrl: &[GPSPoint], per_seg: usize) -> Vec<GPSPoint> {
    let n = ctrl.len();
    if n < 3 {
        return ctrl.to_vec();
    }
    let mut out: Vec<GPSPoint> = Vec::with_capacity(n * per_seg);
    for i in 0..n {
        let p0 = ctrl[(i + n - 1) % n];
        let p1 = ctrl[i];
        let p2 = ctrl[(i + 1) % n];
        let p3 = ctrl[(i + 2) % n];
        for j in 0..per_seg {
            let t = j as f64 / per_seg as f64;
            let t2 = t * t;
            let t3 = t2 * t;
            let x = 0.5
                * ((2.0 * p1.0)
                    + (-p0.0 + p2.0) * t
                    + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
                    + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
            let y = 0.5
                * ((2.0 * p1.1)
                    + (-p0.1 + p2.1) * t
                    + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
                    + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
            out.push((x, y));
        }
    }
    out
}

fn point_at_arc(cum: &[f64], pts: &[GPSPoint], s: f64) -> GPSPoint {
    let total = *cum.last().unwrap_or(&0.0);
    if total <= 0.0 || pts.is_empty() {
        return pts.first().copied().unwrap_or((0.0, 0.0));
    }
    let s = s.clamp(0.0, total);
    let mut lo = 0usize;
    let mut hi = cum.len() - 1;
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if cum[mid] <= s {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let seg = cum[hi] - cum[lo];
    let t = if seg > 1e-9 { (s - cum[lo]) / seg } else { 0.0 };
    (
        pts[lo].0 + (pts[hi].0 - pts[lo].0) * t,
        pts[lo].1 + (pts[hi].1 - pts[lo].1) * t,
    )
}

fn generate_track_points_from_base(
    target_distance: i64,
    base_points: &[GPSPoint],
    point_interval_s: i64,
    stride_range: (f64, f64),
    pace_range: (f64, f64),
    cadence_range: (f64, f64),
    random_seed: Option<i64>,
    start_epoch_s: Option<i64>,
) -> Vec<TrackPoint> {
    if point_interval_s != DEFAULT_POINT_INTERVAL_S {
        panic!("软件固定每 {} 秒取点", DEFAULT_POINT_INTERVAL_S);
    }
    if target_distance <= 0 {
        panic!("target_distance 必须大于 0");
    }

    let base_route = prepare_route_points(base_points, "auto");
    let effective_target: f64 = target_distance as f64;
    let mut rng = match random_seed {
        Some(s) => StdRng::seed_from_u64(s as u64),
        None => StdRng::from_rng(&mut rand::rng()),
    };
    let speed_range = pace_range_to_speed_range(pace_range);
    let profile = init_runner_profile(stride_range, pace_range, cadence_range, &mut rng);

    let ctrl = resample_closed_even(&base_route, 5.0);
    let centerline = catmull_rom_closed(&ctrl, 8);
    let center_cum = build_cumulative_distances(&centerline);
    let movement_cycle_length = *center_cum.last().unwrap_or(&0.0);
    let loop_len = movement_cycle_length;

    let (intervals, segment_distances) = build_fixed_interval_distance_plan(
        effective_target,
        point_interval_s,
        &profile,
        speed_range,
        movement_cycle_length,
        &mut rng,
    );

    let mut elapsed = 0i64;
    let mut run_step = 0i64;
    let mut step_acc = 0.0;
    let mut cumulative_distance = 0.0;
    let mut movement_progress = 0.0;
    let mut prev_step_inc: Option<f64> = None;
    let mut prev_point: Option<GPSPoint> = None;
    let mut prev_prev_point: Option<GPSPoint> = None;
    let mut gps_drift = 0.0;
    let mut track_points = Vec::new();

    for (seg_idx, &seg_target) in segment_distances.iter().enumerate() {
        let interval_s = intervals[seg_idx];

        movement_progress += seg_target;
        let s = if loop_len > 0.0 { movement_progress % loop_len } else { 0.0 };
        let next_point = point_at_arc(&center_cum, &centerline, s);

        let noisy = apply_gps_noise(prev_point, prev_prev_point, next_point, &mut rng, &mut gps_drift);
        prev_prev_point = prev_point;
        prev_point = Some(noisy);
        let step_inc = simulate_step_increment(seg_target / interval_s as f64, interval_s, &profile, prev_step_inc, &mut rng);
        step_acc += step_inc;
        run_step = run_step.max(step_acc.round() as i64);
        cumulative_distance += seg_target;
        if seg_idx + 1 == segment_distances.len() {
            cumulative_distance = effective_target;
        }
        elapsed += interval_s;
        let pace = compute_pace_min_per_km(seg_target, interval_s as f64);
        track_points.push(TrackPoint {
            longitude: noisy.0,
            latitude: noisy.1,
            run_mileage_m: cumulative_distance,
            run_time: elapsed,
            run_step,
            pace,
            ts: String::new(),
        });
        prev_step_inc = Some(step_inc);
    }
    if track_points.is_empty() {
        panic!("未生成任何轨迹点");
    }
    let start_time = start_epoch_s.unwrap_or_else(|| now_secs() - elapsed);
    for tp in &mut track_points {
        tp.ts = (start_time + tp.run_time).to_string();
    }
    track_points
}

fn split_track_points(track_points: &[TrackPoint], chunk_size: usize) -> Vec<Vec<TrackPoint>> {
    let chunk_size = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let mut chunks = Vec::new();
    for c in track_points.chunks(chunk_size) {
        chunks.push(c.to_vec());
    }
    chunks
}

fn build_split_payload(
    user_name: &str,
    school_id: &str,
    record_id: &str,
    points: &[TrackPoint],
    previous_last: Option<&TrackPoint>,
) -> RunSplitPayload {
    let mut card_points = Vec::new();
    for p in points {
        let pace = p.pace;
        let speed_str = if pace <= 0.0 { "0.0".to_string() } else { format!("{:.2}", pace.max(1.0).min(900.0)) };
        card_points.push(CardPoint {
            isFence: "Y".into(),
            isMock: false,
            point: format!("{},{}", p.longitude, p.latitude),
            runMileage: p.run_mileage_m,
            runStatus: "1".into(),
            runStep: p.run_step,
            runTime: p.run_time,
            speed: speed_str,
            ts: p.ts.clone(),
        });
    }
    let (prev_step, prev_time, prev_mileage) = previous_last
        .map(|p| (p.run_step, p.run_time, p.run_mileage_m))
        .unwrap_or((0, 0, 0.0));
    let last = points.last().unwrap();
    let step_number = last.run_step - prev_step;
    let times = last.run_time - prev_time;
    let mileage = last.run_mileage_m - prev_mileage;
    if step_number < 0 || times <= 0 || mileage <= 0.0 {
        panic!("批次累计值必须递增");
    }
    let speeds = compute_pace_min_per_km(mileage, times as f64);
    let strides = if step_number > 0 { mileage / step_number as f64 } else { 0.0 };
    let run_steps = if times > 0 { step_number as f64 / (times as f64 / 60.0) } else { 0.0 };
    RunSplitPayload {
        step_number,
        a: 0,
        b: None,
        c: None,
        cardPointList: card_points,
        crsRunRecordId: record_id.into(),
        mileage,
        orientationNum: 0,
        runSteps: run_steps,
        schoolId: school_id.into(),
        simulateNum: 0,
        speeds,
        strides,
        times,
        userName: user_name.into(),
    }
}

pub fn generate_run_data(
    target_distance: i64,
    record_id: &str,
    user: &str,
    ra_name: &str,
    school_id: &str,
    pace_range: (f64, f64),
    stride_range: (f64, f64),
    cadence_range: (f64, f64),
    json_dir: &str,
    random_seed: Option<i64>,
    start_epoch_s: Option<i64>,
    route_mode: &str,
) -> Vec<RunSplitPayload> {
    let dir = if json_dir.is_empty() {
        tracks_dir(school_id).to_string_lossy().to_string()
    } else {
        json_dir.to_string()
    };
    let track_points = generate_track_points(
        target_distance,
        ra_name,
        &dir,
        DEFAULT_POINT_INTERVAL_S,
        stride_range,
        pace_range,
        cadence_range,
        random_seed,
        start_epoch_s,
        route_mode,
    );
    let chunks = split_track_points(&track_points, DEFAULT_CHUNK_SIZE);
    let mut payloads = Vec::new();
    let mut previous: Option<TrackPoint> = None;
    for chunk in &chunks {
        let payload = build_split_payload(user, school_id, record_id, chunk, previous.as_ref());
        payloads.push(payload);
        previous = Some(chunk[chunk.len() - 1].clone());
    }
    let total: f64 = payloads.iter().map(|p| p.mileage).sum();
    if (total - target_distance as f64).abs() > 1e-6 {
        panic!("批次里程合计错误: {:.6}/{}", total, target_distance);
    }
    payloads
}

pub fn base_route_stats(points: &[GPSPoint]) -> (f64, f64, f64) {
    let mut max_seg = 0.0f64;
    let mut total = 0.0f64;
    for i in 1..points.len() {
        let d = geodesic_distance_m(points[i - 1], points[i]);
        if d > max_seg {
            max_seg = d;
        }
        total += d;
    }
    let span = if points.len() >= 2 {
        geodesic_distance_m(points[0], *points.last().unwrap())
    } else {
        0.0
    };
    (max_seg, span, total)
}

const JUMP_RATIO: f64 = 2.5;

pub struct JumpReport {
    pub max_seg: f64,
    pub median_seg: f64,
    pub max_index: usize,
    pub total_len: f64,
}

pub fn jump_report(points: &[GPSPoint]) -> Option<JumpReport> {
    if points.len() < 3 {
        return None;
    }
    let segs: Vec<f64> = (1..points.len())
        .map(|i| geodesic_distance_m(points[i - 1], points[i]))
        .collect();
    let mut max_seg = 0.0f64;
    let mut max_index = 1usize;
    for (i, d) in segs.iter().enumerate() {
        if *d > max_seg {
            max_seg = *d;
            max_index = i + 1;
        }
    }
    let mut sorted = segs.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    let median_seg = if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    };
    Some(JumpReport {
        max_seg,
        median_seg,
        max_index,
        total_len: segs.iter().sum(),
    })
}

pub fn has_abnormal_jump(points: &[GPSPoint], abs_threshold_m: f64) -> bool {
    match jump_report(points) {
        Some(r) => r.max_seg > abs_threshold_m && r.median_seg > 0.0 && r.max_seg > r.median_seg * JUMP_RATIO,
        None => false,
    }
}

pub fn generate_run_data_from_points(
    target_distance: i64,
    record_id: &str,
    user: &str,
    school_id: &str,
    pace_range: (f64, f64),
    stride_range: (f64, f64),
    cadence_range: (f64, f64),
    random_seed: Option<i64>,
    start_epoch_s: Option<i64>,
    points: &[GPSPoint],
) -> Vec<RunSplitPayload> {
    let track_points = generate_track_points_from_base(
        target_distance,
        points,
        DEFAULT_POINT_INTERVAL_S,
        stride_range,
        pace_range,
        cadence_range,
        random_seed,
        start_epoch_s,
    );
    let chunks = split_track_points(&track_points, DEFAULT_CHUNK_SIZE);
    let mut payloads = Vec::new();
    let mut previous: Option<TrackPoint> = None;
    for chunk in &chunks {
        payloads.push(build_split_payload(user, school_id, record_id, chunk, previous.as_ref()));
        previous = Some(chunk[chunk.len() - 1].clone());
    }
    payloads
}

pub fn build_batches_from_points(
    record_id: &str,
    user: &str,
    school_id: &str,
    points: &[TrackPoint],
    start_epoch_s: i64,
    submit_distance: Option<f64>,
) -> Result<Vec<RunSplitPayload>, String> {
    if points.len() < 2 {
        return Err("轨迹点数不足（至少 2 个）".into());
    }

    let kept: Vec<TrackPoint> = match submit_distance {
        Some(d) if d > 0.0 => points.iter().take_while(|p| p.run_mileage_m <= d).cloned().collect(),
        _ => points.to_vec(),
    };
    let kept = if kept.len() >= 2 { kept } else { points.to_vec() };

    let gps: Vec<GPSPoint> = kept.iter().map(|p| (p.longitude, p.latitude)).collect();
    let (max_seg, _, _) = base_route_stats(&gps);
    if max_seg > 80.0 {
        return Err(format!("轨迹存在异常跳跃（最大间隔 {:.0} 米），请重新导入或生成轨迹后再跑步", max_seg));
    }
    let mut tps = kept;
    let start_time = if start_epoch_s > 0 { start_epoch_s } else { now_secs() };
    for tp in &mut tps {
        tp.ts = (start_time + tp.run_time).to_string();
    }
    let chunks = split_track_points(&tps, DEFAULT_CHUNK_SIZE);
    let mut payloads = Vec::new();
    let mut previous: Option<TrackPoint> = None;
    for chunk in &chunks {
        payloads.push(build_split_payload(user, school_id, record_id, chunk, previous.as_ref()));
        previous = Some(chunk[chunk.len() - 1].clone());
    }
    Ok(payloads)
}
