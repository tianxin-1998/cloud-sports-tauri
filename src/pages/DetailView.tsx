import { useEffect, useRef, useState } from "react";
import { useApp } from "../store";
import { runRecordDetail } from "../services";
import { loadAMap, mapClear, mapErr } from "../utils/amap";

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

export default function DetailView() {
  const { session, cfg, detailRec, show } = useApp();
  const [km, setKm] = useState("—");
  const [ok, setOk] = useState(false);
  const [start, setStart] = useState("—");
  const [pace, setPace] = useState("—");
  const [dur, setDur] = useState("—");
  const [cad, setCad] = useState("—");
  const [msg, setMsg] = useState("");
  const [msgErr, setMsgErr] = useState(false);
  const mapRef = useRef<any>(null);
  const ovRef = useRef<any[]>([]);
  const satRef = useRef<any>(null);
  const [satOn, setSatOn] = useState(false);

  const timerRef = useRef<number | null>(null);
  const playLineRef = useRef<any>(null);
  const runnerRef = useRef<any>(null);
  const rec = detailRec || {};

  useEffect(() => {
    const isOk = rec.isQualified === "1";
    setOk(isOk);
    setKm(rec.recordMileage ?? "—");
    setStart("—");
    setPace("—");
    setDur("—");
    setCad("—");
    setMsg("加载中…");
    setMsgErr(false);
    (async () => {
      if (!session) return;
      try {
        const d = await runRecordDetail(session, rec.id);
        const r = d?.data ?? {};
        setStart("开始时间: " + (r.recordStartTime || rec.recordEndTime || ""));
        setPace((r.recodePace != null ? Number(r.recodePace).toFixed(2) : "—") + "'");
        setDur(fmtDur(r.duration ?? 0));
        setCad(r.recodeCadence ?? "—");
        const clone = String(r.remake || "").split("|")[0];
        setMsg(isOk ? "您已完成本次健跑,请继续加油!" : clone && clone !== "0" ? "疑似克隆检测标记" : "本次成绩未达标");
        setMsgErr(!isOk);
        renderMap(r);
      } catch (e: any) {
        setMsg("详情加载失败：" + String(e?.message ?? e));
        setMsgErr(true);
      }
    })();

    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
      if (mapRef.current) {
        if (playLineRef.current) mapRef.current.remove(playLineRef.current);
        if (runnerRef.current) mapRef.current.remove(runnerRef.current);
      }
      playLineRef.current = null;
      runnerRef.current = null;
    };

  }, [session, detailRec]);

  const renderMap = async (r: any) => {
    const box = document.getElementById("dt-map-box");
    if (!box) return;
    try {
      await loadAMap(cfg);
    } catch (e: any) {
      mapErr(box as HTMLElement, e.message);
      return;
    }
    mapClear(box as HTMLElement);
    if (!mapRef.current) {
      mapRef.current = new window.AMap.Map("dt-map-box", { zoom: 17, mapStyle: "amap://styles/light" });

      satRef.current = new window.AMap.TileLayer.Satellite();
      mapRef.current.add(satRef.current);
    }
    if (ovRef.current.length) {
      mapRef.current.remove(ovRef.current);
      ovRef.current = [];
    }
    const pts = (r.pointsList || [])
      .map((p: any) => {
        const [lng, lat] = String(p.point).split(",").map(Number);
        return { lng, lat, sp: Number(p.speed) || 0 };
      })
      .filter((p: any) => Number.isFinite(p.lng) && Number.isFinite(p.lat));
    if (pts.length < 2) return mapErr(box as HTMLElement, "该记录无 GPS 轨迹点");
    console.log("[detail-map] pointsList=", (r.pointsList || []).length, "manageList=", (r.manageList || []).length, "pts=", pts.length, "sample=", pts.slice(0, 2), pts.slice(-1));
    const sps = pts.map((p: any) => p.sp);
    const mn = Math.min(...sps);
    const mx = Math.max(...sps);
    const ov: any[] = [];

    ov.push(new window.AMap.Polyline({ path: pts.map((p: any) => [p.lng, p.lat]), strokeColor: "#28604f", strokeWeight: 4, lineJoin: "round", bubble: true }));
    for (let i = 1; i < pts.length; i++) {
      ov.push(new window.AMap.Polyline({ path: [[pts[i - 1].lng, pts[i - 1].lat], [pts[i].lng, pts[i].lat]], strokeColor: speedColor(pts[i].sp, mn, mx), strokeWeight: 5, lineJoin: "round", bubble: true }));
    }
    const first = pts[0];
    const last = pts[pts.length - 1];
    const flag = (p: any, txt: string) =>
      new window.AMap.Marker({ position: [p.lng, p.lat], content: `<div style="background:#16283a;color:#fff;font:700 11px sans-serif;padding:3px 7px;border-radius:6px">${txt}</div>`, offset: new window.AMap.Pixel(-12, -12), bubble: true });
    ov.push(flag(first, "始"), flag(last, "终"));

    (r.manageList || []).forEach((m: any) => {
      const [lng, lat] = String(m.point).split(",").map(Number);
      if (!Number.isFinite(lng)) return;
      const pin = document.createElement("div");
      pin.style.cssText = "width:22px;height:22px;border-radius:8px;background:#22c55e;border:2px solid #fff;box-shadow:0 2px 6px rgba(0,0,0,.3);display:grid;place-items:center";
      pin.innerHTML = '<i style="width:8px;height:8px;border-radius:50%;background:#fff;display:block"></i>';
      ov.push(new window.AMap.Marker({ position: [lng, lat], content: pin, offset: new window.AMap.Pixel(-11, -11), bubble: true }));
    });
    mapRef.current.add(ov);
    ovRef.current.push(...ov);

    ov.forEach((o) => o.hide?.());

    mapRef.current.resize?.();
    mapRef.current.setFitView(ov, false, [50, 50, 50, 50]);
    setTimeout(() => {
      mapRef.current?.resize?.();
      mapRef.current?.setFitView(ov, false, [50, 50, 50, 50]);
    }, 150);

    startReplay(pts, mn, mx, ov);
  };

  const startReplay = (pts: any[], mn: number, mx: number, ov: any[]) => {
    const map = mapRef.current;
    if (!map || pts.length < 2) return;
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    if (playLineRef.current) {
      map.remove(playLineRef.current);
      playLineRef.current = null;
    }
    if (runnerRef.current) {
      map.remove(runnerRef.current);
      runnerRef.current = null;
    }
    runnerRef.current = new window.AMap.Marker({
      position: [pts[0].lng, pts[0].lat],
      content: '<div style="width:16px;height:16px;border-radius:50%;background:#2563eb;border:3px solid #fff;box-shadow:0 2px 6px rgba(0,0,0,.35)"></div>',
      offset: new window.AMap.Pixel(-8, -8),
      bubble: true,
    });
    map.add(runnerRef.current);

    const gap = Math.max(8, Math.min(40, Math.floor(5000 / pts.length)));
    let i = 0;
    timerRef.current = window.setInterval(() => {
      i++;
      if (i >= pts.length) {
        clearInterval(timerRef.current!);
        timerRef.current = null;

        ov.forEach((o) => o.show?.());
        return;
      }
      if (!playLineRef.current) {
        playLineRef.current = new window.AMap.Polyline({ path: [], strokeColor: "#2563eb", strokeWeight: 5, lineJoin: "round", bubble: true });
        map.add(playLineRef.current);
      }
      playLineRef.current.setPath(pts.slice(0, i + 1).map((p: any) => [p.lng, p.lat]));
      playLineRef.current.setOptions({ strokeColor: speedColor(pts[i].sp, mn, mx) });
      runnerRef.current?.setPosition([pts[i].lng, pts[i].lat]);
    }, gap);
  };

  return (
    <div className="view on view-detail" id="view-detail">
      <div className="dt-top">
        <button className="dt-back" onClick={() => show("home")}>
          ‹
        </button>
        <h4>健跑详情</h4>
        <span className="pill">—</span>
      </div>
      <div className="dt-hero">
        <span className="km">
          {km}
          <i>公里</i>
        </span>
        <span className={"res " + (ok ? "ok" : "no")}>{ok ? "成绩合格" : "成绩不合格"}</span>
      </div>
      <p className="dt-time">{start}</p>
      <div className="speed-bar" />
      <div className="speed-bar-lb">
        <span>慢</span>
        <span>快</span>
      </div>

      <div className="dt-stats">
        <div>
          <b>{pace}</b>
          <span>平均配速</span>
        </div>
        <div>
          <b>{dur}</b>
          <span>用时</span>
        </div>
        <div>
          <b>{cad}</b>
          <span>步频</span>
        </div>
      </div>
      <p className={"dt-msg" + (msgErr ? " no" : "")}>{msg}</p>
      <div className="dt-map">
        <div id="dt-map-box" style={{ position: "absolute", inset: 0 }} />
        <div className="dt-zoom">
          <button type="button" onClick={() => mapRef.current?.zoomIn()}>＋</button>
          <button type="button" onClick={() => mapRef.current?.zoomOut()}>－</button>
        </div>
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
      </div>
    </div>
  );
}
