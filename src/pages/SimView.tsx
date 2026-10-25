import { useEffect, useRef, useState } from "react";
import { useApp } from "../store";
import {
  discardRun,
  faceNext,
  finishRun,
  isAuthError,
  notifyAuthExpired,
  simulate,
  trackListLocal,
  yunRun,
} from "../services";
import { useRunProgress } from "../hooks";
import { loadAMap, mapClear, mapErr } from "../utils/amap";
import Dropdown from "../components/Dropdown";
import LogoMark from "../components/LogoMark";
import { fmtTime } from "../utils/timefmt";

function fmtDur(sec: number) {
  sec = Number(sec) || 0;
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  return [h, m, s].map((v) => String(v).padStart(2, "0")).join(":");
}

function speedColor(sp: number, min: number, max: number) {
  const t = max > min ? Math.min(Math.max((sp - min) / (max - min), 0), 1) : 0.5;
  return `hsl(${Math.round(120 * (1 - t))},75%,45%)`;
}

function haversine(a: [number, number], b: [number, number]) {
  const R = 6371000;
  const dLat = ((b[1] - a[1]) * Math.PI) / 180;
  const dLng = ((b[0] - a[0]) * Math.PI) / 180;
  const la1 = (a[1] * Math.PI) / 180;
  const la2 = (b[1] * Math.PI) / 180;
  const h = Math.sin(dLat / 2) ** 2 + Math.cos(la1) * Math.cos(la2) * Math.sin(dLng / 2) ** 2;
  return 2 * R * Math.asin(Math.sqrt(h));
}

function pointsToRunData(raw: [number, number][], cfg: any): { points: any[] } {
  const pts = (raw || []).filter((p) => Number.isFinite(p[0]) && Number.isFinite(p[1]));
  if (pts.length < 2) return { points: [] };
  const pacePerKm = (((cfg?.pace_min ?? 5.5) + (cfg?.pace_max ?? 6.7)) / 2) * 60;
  const cad = ((cfg?.cadence_min ?? 164) + (cfg?.cadence_max ?? 176)) / 2;
  let cum = 0;
  const cums = [0];
  for (let i = 1; i < pts.length; i++) {
    cum += haversine(pts[i - 1], pts[i]);
    cums.push(cum);
  }
  const totalKm = cum / 1000;
  const totalT = pacePerKm * totalKm;
  const start = Math.floor(Date.now() / 1000);
  return {
    points: pts.map((p, i) => {
      const t = totalT * (cums[i] / cum);
      const prevT = i === 0 ? 0 : totalT * (cums[i - 1] / cum);
      const segM = cums[i] - cums[i - 1];
      const segT = t - prevT;
      const sp = segT > 0 ? segM / segT : 0;
      return {
        lng: p[0],
        lat: p[1],
        mileage: cums[i],
        time: Math.round(t),
        step: Math.round(cad * (t / 60)),
        speed: sp,
        ts: start + Math.round(t),
      };
    }),
  };
}

export default function SimView() {
  const { session, cfg, show } = useApp();
  const areas = session?.run_areas ?? [];
  const [areaIdx, setAreaIdx] = useState(areas.length ? "0" : "");
  const [dist, setDist] = useState(2000);

  const [mult, setMult] = useState(10);

  const [menuOpen, setMenuOpen] = useState(false);
  const menuWrapRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!menuOpen) return;
    const onDown = (e: MouseEvent) => {
      if (menuWrapRef.current && !menuWrapRef.current.contains(e.target as Node)) setMenuOpen(false);
    };
    document.addEventListener("mousedown", onDown);
    return () => document.removeEventListener("mousedown", onDown);
  }, [menuOpen]);
  const [playing, setPlaying] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [err, setErr] = useState("");
  const [busy, setBusy] = useState(false);

  const [runPhase, setRunPhase] = useState<"idle" | "running" | "verify" | "done">("idle");
  const [verify, setVerify] = useState<{ code: number | null; msg: string; data?: any } | null>(null);
  const [heldRecord, setHeldRecord] = useState("");
  const [runMsg, setRunMsg] = useState("");

  const [km, setKm] = useState("0.00");
  const [dur, setDur] = useState("00:00:00");
  const [pace, setPace] = useState("—");
  const [cad, setCad] = useState("—");
  const [step, setStep] = useState(0);
  const [idx, setIdx] = useState(0);

  const [mapReady, setMapReady] = useState(false);

  const satRef = useRef<any>(null);
  const [satOn, setSatOn] = useState(false);

  const [impOpen, setImpOpen] = useState(false);
  const [localItems, setLocalItems] = useState<any[]>([]);
  const [impNote, setImpNote] = useState("");
  const [impPage, setImpPage] = useState(0);
  const IMP_PAGE = 5;

  const mapRef = useRef<any>(null);
  const boxRef = useRef<HTMLDivElement>(null);
  const ptsRef = useRef<any[]>([]);

  const trackSourceRef = useRef<"generated" | "imported" | null>(null);
  const lineRef = useRef<any>(null);
  const runnerRef = useRef<any>(null);
  const timerRef = useRef<number | null>(null);
  const fenceRef = useRef<any>(null);

  const segRef = useRef<any>(null);

  const runPtsRef = useRef<any[]>([]);
  const runLineRef = useRef<any>(null);
  const runMarkerRef = useRef<any>(null);

  const skipClearRef = useRef(false);

  const area = areas[Number(areaIdx)];

  const parse = (s?: string): [number, number][] =>
    String(s || "")
      .split("|")
      .map((v) => {
        const [lng, lat] = v.split(",").map(Number);
        return [lng, lat] as [number, number];
      })
      .filter((p) => Number.isFinite(p[0]) && Number.isFinite(p[1]));

  useEffect(() => {
    let alive = true;
    (async () => {
      const box = boxRef.current;
      if (!box) return;
      try {
        await loadAMap(cfg);
      } catch (e: any) {
        mapErr(box as HTMLElement, e.message);
        return;
      }
      if (!alive) return;
      mapClear(box as HTMLElement);
      mapRef.current = new window.AMap.Map("sim-map-box", { zoom: 17, mapStyle: "amap://styles/light" });

      const sat = new window.AMap.TileLayer.Satellite();
      mapRef.current.add(sat);
      satRef.current = sat;
      setSatOn(true);
      setMapReady(true);
    })();
    return () => {
      alive = false;
      stopTimer();
    };

  }, []);

  useEffect(() => {
    const map = mapRef.current;
    if (!map || !window.AMap || !area) return;
    const fence = parse(area.fence);
    const cps = parse(area.points);
    if (fenceRef.current) {
      map.remove(fenceRef.current);
      fenceRef.current = null;
    }
    const overlays: any[] = [];
    if (fence.length > 2) {
      overlays.push(
        new window.AMap.Polygon({
          path: fence,
          strokeColor: "#28604f",
          strokeWeight: 2,
          strokeStyle: "dashed",
          fillColor: "#28604f",
          fillOpacity: 0.08,
          bubble: true,
        }),
      );
    }

    cps.forEach((p, i) =>
      overlays.push(
        new window.AMap.Marker({
          position: p,
          content: `<div style="width:20px;height:20px;border-radius:50%;background:#c99a3a;color:#fff;font:700 10px sans-serif;display:grid;place-items:center;border:2px solid #fff;box-shadow:0 1px 5px rgba(0,0,0,.3)">${i + 1}</div>`,
          offset: new window.AMap.Pixel(-10, -10),
          bubble: true,
        }),
      ),
    );
    if (overlays.length) {
      fenceRef.current = overlays;
      map.add(overlays);
      map.setFitView(overlays, false, [40, 40, 40, 40]);
    }

    const skip = skipClearRef.current;
    skipClearRef.current = false;
    if (skip) {
      if (lineRef.current) map.setFitView([...overlays, lineRef.current], false, [40, 40, 40, 40]);
      return;
    }

    stopTimer();
    setPlaying(false);
    setLoaded(false);
    if (lineRef.current) {
      map.remove(lineRef.current);
      lineRef.current = null;
    }
    if (runnerRef.current) {
      map.remove(runnerRef.current);
      runnerRef.current = null;
    }
    ptsRef.current = [];
    setIdx(0);
    setKm("0.00");
    setDur("00:00:00");
    setPace("—");
    setCad("—");
    setStep(0);

  }, [areaIdx, mapReady]);

  const stopTimer = () => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  };

  const clearOverlays = () => {
    const map = mapRef.current;
    if (!map) return;
    if (lineRef.current) map.remove(lineRef.current);
    if (runnerRef.current) map.remove(runnerRef.current);
    if (segRef.current) map.remove(segRef.current);
    lineRef.current = null;
    runnerRef.current = null;
    segRef.current = null;

    if (runLineRef.current) map.remove(runLineRef.current);
    if (runMarkerRef.current) map.remove(runMarkerRef.current);
    runLineRef.current = null;
    runMarkerRef.current = null;
  };

  const drawRunProgress = () => {
    const map = mapRef.current;
    const pts = runPtsRef.current;
    if (!map || !pts.length) return;
    const path = pts.map((p) => [p.lng, p.lat]);
    if (!runLineRef.current) {
      runLineRef.current = new window.AMap.Polyline({ path, strokeColor: "#ef4444", strokeWeight: 5, lineJoin: "round", bubble: true });
      map.add(runLineRef.current);
      runMarkerRef.current = new window.AMap.Marker({
        position: path[path.length - 1],
        content: '<div style="width:16px;height:16px;border-radius:50%;background:#ef4444;border:3px solid #fff;box-shadow:0 2px 6px rgba(0,0,0,.35)"></div>',
        offset: new window.AMap.Pixel(-8, -8),
        bubble: true,
      });
      map.add(runMarkerRef.current);
      map.setFitView([runLineRef.current], false, [60, 60, 60, 60]);
    } else {
      runLineRef.current.setPath(path);
      runMarkerRef.current?.setPosition(path[path.length - 1]);
    }

    const p = pts[pts.length - 1];
    const kmv = (Number(p.mileage) || 0) / 1000;
    const t = Number(p.time) || 0;
    const st = Number(p.step) || 0;
    setKm(kmv.toFixed(2));
    setDur(fmtDur(t));
    setPace(kmv > 0 ? (t / 60 / kmv).toFixed(2) + "'" : "—");
    setCad(t > 0 ? String(Math.round(st / (t / 60))) : "—");
    setStep(st);
  };

  useRunProgress((payload) => {
    if (payload?.status === "running" && Array.isArray(payload.points) && payload.points.length) {
      runPtsRef.current.push(...payload.points);
      drawRunProgress();
    } else if (payload?.status === "held") {
      let r: any = payload.result;
      try {
        r = typeof r === "string" ? JSON.parse(r) : r;
      } catch {

      }
      setRunPhase("verify");
      setVerify(r?.verify ?? null);
      setHeldRecord(r?.record_id ?? "");
    } else if (payload?.status === "failed") {
      const failMsg = String(payload.result ?? "跑步失败");
      if (isAuthError(failMsg)) notifyAuthExpired(failMsg);
      setRunPhase("idle");
      setErr(failMsg);
    }
  });

  const startRealRun = async () => {
    if (!session) return;
    if (!area) return setErr("请先选择跑区");
    if (runPhase === "running" || runPhase === "verify") return;
    setErr("");
    setRunMsg("");
    setVerify(null);
    setHeldRecord("");

    const map = mapRef.current;
    if (map) {
      if (runLineRef.current) map.remove(runLineRef.current);
      if (runMarkerRef.current) map.remove(runMarkerRef.current);
    }
    runLineRef.current = null;
    runMarkerRef.current = null;
    runPtsRef.current = [];
    setKm("0.00");
    setDur("00:00:00");
    setPace("—");
    setCad("—");
    setStep(0);
    setRunPhase("running");
    try {

      if (trackSourceRef.current !== "generated" || !ptsRef.current.length) {
        const g = await simulate({ session, targetDistance: dist, raName: area?.raName || "", fast: mult !== 1, points: undefined });
        ptsRef.current = g.points || [];
        trackSourceRef.current = "generated";
      }
      setLoaded(true);
      renderTrack(ptsRef.current);

      let face_base64: string | undefined;
      if (area?.runFaceStatus === "Y") {
        const f = await faceNext();
        face_base64 = f.data_base64;
      }
      const d: any = await yunRun({
        session,
        target_distance: dist,
        ra_name: area?.raName || "",
        fast: mult !== 1,
        face_base64,
        hold_finish: true,

        points: ptsRef.current.length
          ? ptsRef.current.map((p) => ({ lng: p.lng, lat: p.lat, mileage: p.mileage, time: p.time, step: p.step }))
          : undefined,
      });
      if (d?.task_id) {
        setRunMsg("跑步已开始，实时上传中…");
      } else if (d?.result) {
        const r = typeof d.result === "string" ? JSON.parse(d.result) : d.result;
        if (r?.held) {
          setRunPhase("verify");
          setVerify(r.verify ?? null);
          setHeldRecord(r.record_id || "");
        } else {
          setRunPhase("idle");
          setRunMsg(r?.msg || "完成");
        }
      }
    } catch (e: any) {
      setRunPhase("idle");
      setErr(String(e?.message ?? e));
    }
  };

  const confirmSubmit = async () => {
    if (!heldRecord) return;
    try {
      const d = await finishRun(heldRecord);
      setRunPhase("done");
      setVerify(null);
      setHeldRecord("");
      setRunMsg("提交完成：" + (d.msg || "已记录"));
    } catch (e: any) {
      setErr(String(e?.message ?? e));
    }
  };

  const discardSubmit = async () => {
    if (!heldRecord) return;
    try {
      await discardRun(heldRecord);
    } catch {

    }
    setRunPhase("idle");
    setVerify(null);
    setHeldRecord("");
    setRunMsg("已放弃提交，本次成绩未记录");
  };

  const renderTrack = (pts: any[]) => {
    const map = mapRef.current;
    if (!map || !pts.length) return;
    if (lineRef.current) map.remove(lineRef.current);
    if (runnerRef.current) map.remove(runnerRef.current);
    const path = pts.map((p) => [p.lng, p.lat]);
    const bg = new window.AMap.Polyline({ path, strokeColor: "#c9ced6", strokeWeight: 4, bubble: true });
    map.add(bg);
    lineRef.current = bg;
    runnerRef.current = new window.AMap.Marker({
      position: path[0],
      content: '<div style="width:16px;height:16px;border-radius:50%;background:#2563eb;border:3px solid #fff;box-shadow:0 2px 6px rgba(0,0,0,.35)"></div>',
      offset: new window.AMap.Pixel(-8, -8),
      bubble: true,
    });
    map.add(runnerRef.current);
    map.setFitView([bg], false, [50, 50, 50, 50]);
  };

  const doGenerate = async () => {
    if (!session) return;
    setBusy(true);
    setErr("");
    stopTimer();
    setPlaying(false);
    clearOverlays();
    try {

      const basePoints =
        loaded && ptsRef.current.length
          ? ptsRef.current.map((p) => ({ lng: p.lng, lat: p.lat }))
          : undefined;
      const d = await simulate({
        session,
        targetDistance: dist,
        raName: area?.raName || "",
        fast: mult !== 1,
        points: basePoints,
      });
      ptsRef.current = d.points || [];
      trackSourceRef.current = "generated";

      if (d.warnings?.length) setErr(d.warnings.join("；"));
      setIdx(0);
      setLoaded(true);

      const kmSum = (Number(d.total_mileage_m) || 0) / 1000;
      const tSum = Number(d.total_time_s) || 0;
      const stSum = Number(d.total_step) || 0;
      setKm(kmSum.toFixed(2));
      setDur(fmtDur(tSum));
      setPace(kmSum > 0 ? (tSum / 60 / kmSum).toFixed(2) + "'" : "—");
      setCad(tSum > 0 ? String(Math.round(stSum / (tSum / 60))) : "—");
      setStep(stSum);
      renderTrack(ptsRef.current);
    } catch (e: any) {
      setErr(String(e?.message ?? e));
      setLoaded(false);
    } finally {
      setBusy(false);
    }
  };

  const loadImportedPoints = (raw: [number, number][], raName?: string) => {
    const d = pointsToRunData(raw, cfg);
    if (!d.points.length) {
      setImpNote("该轨迹点数量不足（至少 2 个）");
      return;
    }

    if (raName) {
      const idx = areas.findIndex((a: any) => a.raName === raName);
      if (idx >= 0 && String(idx) !== areaIdx) {
        skipClearRef.current = true;
        setAreaIdx(String(idx));
      }
    }
    stopTimer();
    setPlaying(false);
    clearOverlays();
    ptsRef.current = d.points;
    trackSourceRef.current = "imported";
    setIdx(0);
    setLoaded(true);
    setKm("0.00");
    setDur("00:00:00");
    setPace("—");
    setCad("—");
    setStep(0);
    renderTrack(ptsRef.current);
    setImpOpen(false);
    setImpNote("");
  };

  const loadLocalSim = async () => {
    if (!session) return;
    try {
      const d = await trackListLocal(session.school_code);
      setLocalItems(d.items || []);
      setImpPage(0);
      setImpNote((d.items || []).length ? "" : "本地还没有保存的轨迹，可先在轨迹绘制页保存。");
    } catch (e: any) {
      setImpNote(String(e?.message ?? e));
    }
  };

  const startPlay = (m?: number) => {
    if (!loaded || !ptsRef.current.length) return;
    const map = mapRef.current;
    if (!map) return;

    if (segRef.current) {
      map.remove(segRef.current);
      segRef.current = null;
    }
    setIdx(0);
    setKm("0.00");
    setDur("00:00:00");
    setPace("—");
    setCad("—");
    setStep(0);
    if (ptsRef.current.length) runnerRef.current?.setPosition([ptsRef.current[0].lng, ptsRef.current[0].lat]);
    setPlaying(true);

    const seg: any = new window.AMap.Polyline({ path: [], strokeColor: "#2563eb", strokeWeight: 5, lineJoin: "round", bubble: true });
    map.add(seg);
    segRef.current = seg;
    const sps = ptsRef.current.map((p: any) => Number(p.speed) || 0);
    const mn = Math.min(...sps);
    const mx = Math.max(...sps);
    let i = 0;

    const MULT = m ?? mult;
    const delayOf = (idx: number) => {
      const cur = Number(ptsRef.current[idx]?.time) || 0;
      const prev = idx > 0 ? Number(ptsRef.current[idx - 1]?.time) || 0 : 0;
      const d = Math.max(0, cur - prev);
      return Math.max(8, (d / MULT) * 1000);
    };
    stopTimer();
    const tick = () => {
      i++;
      if (i >= ptsRef.current.length) {
        stopTimer();
        setPlaying(false);
        return;
      }
      const p = ptsRef.current[i];
      seg.setPath(ptsRef.current.slice(0, i + 1).map((q: any) => [q.lng, q.lat]));
      seg.setOptions({ strokeColor: speedColor(Number(p.speed) || 0, mn, mx) });
      runnerRef.current?.setPosition([p.lng, p.lat]);
      const mileage = Number(p.mileage) || 0;
      const t = Number(p.time) || 0;
      const st = Number(p.step) || 0;
      const kmv = mileage / 1000;
      setIdx(i);
      setKm(kmv.toFixed(2));
      setDur(fmtDur(t));
      setPace(kmv > 0 ? ((t / 60) / kmv).toFixed(2) + "'" : "—");
      setCad(t > 0 ? String(Math.round(st / (t / 60))) : "—");
      setStep(st);
      timerRef.current = window.setTimeout(tick, delayOf(i));
    };
    timerRef.current = window.setTimeout(tick, delayOf(0));
  };

  const pause = () => {
    stopTimer();
    setPlaying(false);
  };

  const total = ptsRef.current.length;

  return (
    <div className="view on view-sim" id="view-sim">
      <div className="app-card" style={{ padding: "12px 14px" }}>
        <div className="m-field" style={{ margin: "0 0 10px" }}>
          <Dropdown
            options={areas.map((a, i) => ({ value: String(i), label: a.raName || `跑区 ${i + 1}` }))}
            value={areaIdx}
            onChange={setAreaIdx}
            placeholder="选择跑区"
          />
        </div>

        <div className="sim-row">
          <div className="sim-dist">
            <input type="number" value={dist} min={1} max={4500} step={100} onChange={(e) => setDist(Number(e.target.value))} />
            <i>米</i>
          </div>
          <button
            className="m-btn sm gold"
            onClick={() => {
              setImpOpen(true);
              setImpNote("");
              setImpPage(0);
              loadLocalSim();
            }}
          >
            导入轨迹
          </button>
          <button className="m-btn sm" disabled={busy} onClick={doGenerate}>
            {busy ? "生成中…" : "生成轨迹"}
          </button>

          <div className="sim-mult" ref={menuWrapRef}>
            {!playing ? (
              <>
                <button className="m-btn sm" disabled={!loaded} onClick={() => setMenuOpen((o) => !o)}>
                  模拟跑 ▾
                </button>
                {menuOpen && (
                  <div className="sim-mult-menu">
                    {[1, 10, 30, 60].map((n) => (
                      <button
                        key={n}
                        type="button"
                        className={"sim-mult-opt" + (n === mult ? " on" : "")}
                        onClick={() => {

                          setMult(n);
                          setMenuOpen(false);
                          startPlay(n);
                        }}
                      >
                        {n} 倍速
                      </button>
                    ))}
                  </div>
                )}
              </>
            ) : (
              <button className="m-btn sm" onClick={pause}>
                暂停
              </button>
            )}
          </div>
        </div>
      </div>

      <div className="sim-stats">
        <div>
          <b>{km}</b>
          <span>公里</span>
        </div>
        <div>
          <b>{dur}</b>
          <span>用时</span>
        </div>
        <div>
          <b>{pace}</b>
          <span>平均配速</span>
        </div>
        <div>
          <b>{cad}</b>
          <span>步频</span>
        </div>
      </div>

      {runMsg && <p className="task-note">{runMsg}</p>}
      {err && <p className="task-note err">{err}</p>}

      <div className="sim-map">
        <div id="sim-map-box" ref={boxRef} style={{ position: "absolute", inset: 0 }} />
        <button
          className="map-sat"
          type="button"
          onClick={() => {
            const map = mapRef.current;
            if (!map || !window.AMap) return;
            if (satRef.current) {
              map.remove(satRef.current);
              satRef.current = null;
              setSatOn(false);
            } else {
              satRef.current = new window.AMap.TileLayer.Satellite();
              map.add(satRef.current);
              setSatOn(true);
            }
          }}
        >
          {satOn ? "标准" : "卫星"}
        </button>

        {runPhase !== "verify" && (

          <div className="run-fab-wrap">
            <button
              className={"run-fab" + (runPhase === "running" ? " running" : "")}
              disabled={runPhase === "running"}
              onClick={startRealRun}
              aria-label={runPhase === "running" ? "跑步中" : "开始跑步"}
            >
              <span className="run-fab-icon">
                <LogoMark size={52} />
              </span>
            </button>
            <span className="run-fab-tip">{runPhase === "running" ? "跑步中…" : "开始跑步"}</span>
          </div>
        )}

        {runPhase === "verify" && (
          <div className="run-confirm">
            <div className="rc-head">跑步完成 · 请确认提交</div>
            <div className="rc-body">
              {verify ? (
                <>
                  <div className={"rc-status " + (verify.code === 200 ? "ok" : "bad")}>
                    <i>{verify.code === 200 ? "✓" : "✕"}</i>
                    <span>
                      {verify.data?.msg || verify.msg || (verify.code === 200 ? "校验通过" : "校验未通过")}
                    </span>
                  </div>
                  {Array.isArray(verify.data?.list) && verify.data.list.length > 0 && (
                    <ul className="rc-list">
                      {verify.data.list.map((it: any, i: number) => (
                        <li key={i} className={it?.isStandard === "Y" ? "ok" : "bad"}>
                          <i>{it?.isStandard === "Y" ? "✓" : "✕"}</i>
                          <span>{it?.msg || "—"}</span>
                        </li>
                      ))}
                    </ul>
                  )}
                  {(!Array.isArray(verify.data?.list) || !verify.data.list?.length) && verify.msg && (
                    <div className="rc-note">{verify.msg}</div>
                  )}
                </>
              ) : (
                "点位已全部上传，等待确认"
              )}
            </div>
            <div className="rc-btns">
              <button className="m-btn sm" onClick={discardSubmit}>
                放弃提交
              </button>
              <button className="m-btn sm gold" onClick={confirmSubmit}>
                确认提交
              </button>
            </div>
          </div>
        )}
      </div>

      {impOpen && (
        <div className="imp-mask" onClick={() => setImpOpen(false)}>
          <div className="imp-box" onClick={(e) => e.stopPropagation()}>
            <div className="imp-head">
              <b>导入轨迹</b>
              <button className="tk-x" type="button" onClick={() => setImpOpen(false)}>
                ×
              </button>
            </div>
            <div className="imp-tabs">
              <span className="imp-tab-label">本地预设</span>
              <button
                type="button"
                className="m-btn sm gold"
                onClick={() => {

                  setImpOpen(false);
                  setImpNote("");
                  show("draw");
                }}
              >
                添加轨迹
              </button>
            </div>

            <div className="imp-list">
              {(() => {
                const items = localItems;
                const pageCount = Math.max(1, Math.ceil(items.length / IMP_PAGE));
                const safePage = Math.min(impPage, pageCount - 1);
                const pageItems = items.slice(safePage * IMP_PAGE, safePage * IMP_PAGE + IMP_PAGE);
                if (!items.length) return <div className="dd-empty">本地暂无预设轨迹</div>;
                return pageItems.map((it: any) => {

                  const km = it.record_mileage != null && it.record_mileage !== ""
                    ? it.record_mileage
                    : it.distance_m != null
                      ? (Number(it.distance_m) / 1000).toFixed(2)
                      : "—";
                  return (
                    <button
                      key={it.name}
                      type="button"
                      className="imp-item rec"
                      onClick={() => loadImportedPoints(it.points || [], it.ra_name || it.name)}
                    >
                      <div className="ic">
                        <LogoMark size={46} />
                      </div>
                      <div className="m">
                        <small>{it.created_at ? fmtTime(it.created_at) : ""}</small>
                        <span className="ra">📋 {it.ra_name || it.name}</span>
                      </div>

                      <b className="rec-len">{km} km</b>
                    </button>
                  );
                });
              })()}
            </div>

            {(() => {
              const items = localItems;
              const pageCount = Math.max(1, Math.ceil(items.length / IMP_PAGE));
              const safePage = Math.min(impPage, pageCount - 1);
              return (
                <div className="imp-pager">
                  <button className="m-btn sm" disabled={safePage <= 0} onClick={() => setImpPage(safePage - 1)}>
                    ‹ 上一页
                  </button>
                  <span>
                    {safePage + 1} / {pageCount}
                  </span>
                  <button className="m-btn sm" disabled={safePage >= pageCount - 1} onClick={() => setImpPage(safePage + 1)}>
                    下一页 ›
                  </button>
                </div>
              );
            })()}

            {impNote && <p className="task-note err">{impNote}</p>}
          </div>
        </div>
      )}
    </div>
  );
}
