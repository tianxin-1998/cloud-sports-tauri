import { useEffect, useRef, useState } from "react";
import { useApp } from "../store";
import { trackQuery, trackUpload, trackValidate } from "../services";
import { loadAMap, mapClear, mapErr } from "../utils/amap";
import Dropdown from "../components/Dropdown";

const ROUTE_TYPE = "loop";

export default function DrawView() {
  const { session, cfg, show } = useApp();
  const areas = session?.run_areas ?? [];
  const [areaIdx, setAreaIdx] = useState(areas.length ? "0" : "");
  const [name, setName] = useState("");
  const [found, setFound] = useState<{ text: string; cls: string }>({ text: "未查询", cls: "pill" });
  const [hud, setHud] = useState("点击「开始绘制」后，在地图上逐点点击");
  const [drawing, setDrawing] = useState(false);
  const [canSave, setCanSave] = useState(false);
  const [note, setNote] = useState("围栏与打卡点仅作参考；轨迹首尾会自动重合闭合，保存前仍会经本地校验（闭合与里程）。");
  const [noteErr, setNoteErr] = useState(false);

  const boxRef = useRef<HTMLDivElement>(null);
  const mapRef = useRef<any>(null);
  const ptsRef = useRef<[number, number][]>([]);
  const markersRef = useRef<any[]>([]);
  const lineRef = useRef<any>(null);
  const fenceRef = useRef<any[]>([]);
  const satRef = useRef<any>(null);
  const drawingRef = useRef(false);

  const area = areas[Number(areaIdx)];

  const parse = (s?: string): [number, number][] =>
    String(s || "")
      .split("|")
      .map((v) => {
        const [lng, lat] = v.split(",").map(Number);
        return [lng, lat] as [number, number];
      })
      .filter((p) => Number.isFinite(p[0]) && Number.isFinite(p[1]));

  const setHudCount = () => setHud(ptsRef.current.length + " 个点");

  const addPoint = (lng: number, lat: number) => {
    const map = mapRef.current;
    if (!map) return;
    ptsRef.current.push([lng, lat]);
    const m = new window.AMap.Marker({
      position: [lng, lat],
      content:
        '<div style="width:10px;height:10px;border-radius:50%;background:#f4633f;border:2px solid #fff;box-shadow:0 1px 3px rgba(0,0,0,.4)"></div>',
      offset: new window.AMap.Pixel(-5, -5),
    });
    markersRef.current.push(m);
    map.add(m);
    if (lineRef.current) lineRef.current.setPath(ptsRef.current);
    else {
      lineRef.current = new window.AMap.Polyline({ path: ptsRef.current, strokeColor: "#f4633f", strokeWeight: 4, bubble: true });
      map.add(lineRef.current);
    }
    setCanSave(false);
    setHudCount();
  };

  const lockMap = (lock: boolean) => {
    const map = mapRef.current;
    if (!map) return;
    try {
      map.setStatus({ dragEnable: !lock, zoomEnable: !lock, keyboardEnable: !lock });
    } catch {

    }
  };

  const ensureClosed = (): boolean => {
    const pts = ptsRef.current;
    if (pts.length < 3) return false;
    const [fLng, fLat] = pts[0];
    const [lLng, lLat] = pts[pts.length - 1];
    if (fLng === lLng && fLat === lLat) return false;
    addPoint(fLng, fLat);
    return true;
  };

  const reset = () => {
    const map = mapRef.current;
    if (map) {
      if (markersRef.current.length) map.remove(markersRef.current);
      if (lineRef.current) map.remove(lineRef.current);
    }
    ptsRef.current = [];
    markersRef.current = [];
    lineRef.current = null;
    drawingRef.current = false;
    lockMap(false);
    setDrawing(false);
    setCanSave(false);
    setHud("点击「开始绘制」后，在地图上逐点点击");
  };

  const drawFence = async () => {
    const box = boxRef.current;
    if (!box || !area) return;
    setName(area.raName || "");
    try {
      await loadAMap(cfg);
    } catch (e: any) {
      mapErr(box, e.message);
      setHud("地图不可用");
      return;
    }
    mapClear(box);
    if (!mapRef.current) {
      mapRef.current = new window.AMap.Map("dr-map-box", { zoom: 17, mapStyle: "amap://styles/light" });

      satRef.current = new window.AMap.TileLayer.Satellite();
      mapRef.current.add(satRef.current);
      mapRef.current.on("click", (ev: any) => {
        if (!drawingRef.current) return;
        const ll = ev?.lnglat;
        if (!ll) return;
        const lng = ll.getLng ? ll.getLng() : ll.lng;
        const lat = ll.getLat ? ll.getLat() : ll.lat;
        if (Number.isFinite(lng) && Number.isFinite(lat)) addPoint(lng, lat);
      });
    }
    if (fenceRef.current.length) mapRef.current.remove(fenceRef.current);
    fenceRef.current = [];
    const fence = parse(area.fence);
    const cps = parse(area.points);
    if (fence.length > 2)
      fenceRef.current.push(
        new window.AMap.Polygon({ path: fence, strokeColor: "#28604f", strokeWeight: 2, strokeStyle: "dashed", fillColor: "#28604f", fillOpacity: 0.08, bubble: true }),
      );
    cps.forEach((p, i) =>
      fenceRef.current.push(
        new window.AMap.Marker({
          position: p,
          content: `<div style="width:20px;height:20px;border-radius:50%;background:#c99a3a;color:#fff;font:700 10px sans-serif;display:grid;place-items:center;border:2px solid #fff;box-shadow:0 1px 5px rgba(0,0,0,.3)">${i + 1}</div>`,
          offset: new window.AMap.Pixel(-10, -10),
          bubble: true,
        }),
      ),
    );
    mapRef.current.add(fenceRef.current);
    if (fenceRef.current.length) mapRef.current.setFitView(fenceRef.current, false, [30, 30, 30, 30]);
  };

  const queryTrack = async () => {
    if (!session || !area) return;
    try {
      const q = await trackQuery(session.school_code, area.raName || "");
      setFound(q.found ? { text: "已有轨迹", cls: "pill ok" } : { text: "暂无轨迹", cls: "pill gold" });
      if (q.found && q.points?.length && !ptsRef.current.length) {
        q.points.forEach((p) => addPoint(p[0], p[1]));
        setHud(`已载入本地轨迹 ${ptsRef.current.length} 个点（清空重画或直接覆盖保存）`);
      }
    } catch {
      setFound({ text: "未查询", cls: "pill" });
    }
  };

  useEffect(() => {
    if (areas.length && !areaIdx) setAreaIdx("0");

  }, [areas.length]);

  useEffect(() => {
    reset();
    drawFence();
    queryTrack();

  }, [areaIdx]);

  const doCheck = async () => {
    setNoteErr(false);
    if (!session) return (setNoteErr(true), setNote("请先登录"));
    if (ptsRef.current.length < 2) return (setNoteErr(true), setNote("至少需要 2 个点"));
    ensureClosed();
    try {
      const d = await trackValidate(session.school_code, area?.raName || "", ROUTE_TYPE, ptsRef.current);
      setNote(`校验通过：${d.distance_m} 米（${d.points.length} 个点，首尾已闭合）`);
      setCanSave(true);
    } catch (e: any) {
      setNoteErr(true);
      setNote(String(e?.message ?? e));
      setCanSave(false);
    }
  };

  const doSave = async () => {
    if (!session) return;
    const nm = name.trim();
    if (!nm) return (setNoteErr(true), setNote("请填写提交名称"));
    ensureClosed();
    try {
      const d = await trackUpload(session.school_code, area?.raName || "", ROUTE_TYPE, ptsRef.current, nm);
      setNoteErr(false);
      setNote(`保存成功：${d.name} · ${d.distance_m} 米`);
      queryTrack();
    } catch (e: any) {
      setNoteErr(true);
      setNote(String(e?.message ?? e));
    }
  };

  const toggleSat = () => {
    const map = mapRef.current;
    if (!map || !window.AMap) return;
    if (satRef.current) {
      map.remove(satRef.current);
      satRef.current = null;
    } else {
      satRef.current = new window.AMap.TileLayer.Satellite();
      map.add(satRef.current);
    }
  };

  return (
    <div className="view on" id="view-draw">
      <div className="dt-top">
        <button className="dt-back" onClick={() => show("sim")}>
          ‹
        </button>
        <h4>轨迹绘制</h4>
        <span className={found.cls}>{found.text}</span>
      </div>

      <div className="m-field">
        <Dropdown
          options={areas.map((a, i) => ({ value: String(i), label: a.raName || `跑区 ${i + 1}` }))}
          value={areaIdx}
          onChange={setAreaIdx}
          placeholder="选择跑区"
        />
      </div>
      <div className="m-field" style={{ marginBottom: 10 }}>
        <input placeholder="提交名称" value={name} onChange={(e) => setName(e.target.value)} />
      </div>

      <div className="dr-map">
        <div id="dr-map-box" className="dr-map-box" ref={boxRef} />
        <button className="map-sat" type="button" onClick={toggleSat}>
          卫星
        </button>
        <div className="dr-hud">{hud}</div>
      </div>

      <div className="dr-btns">
        <button
          className="m-btn sm"
          onClick={() => {
            const on = !drawingRef.current;
            drawingRef.current = on;
            setDrawing(on);
            if (on) {
              lockMap(true);
              setHud("地图点击添加轨迹点");
            } else {
              lockMap(false);

              const closed = ensureClosed();
              setHud(closed ? "已自动补上终点，首尾重合" : "绘制已暂停");
            }
          }}
        >
          {drawing ? "停止绘制" : "开始绘制"}
        </button>
        <button className="m-btn sm" onClick={() => { ptsRef.current.pop(); const m = markersRef.current.pop(); if (m && mapRef.current) mapRef.current.remove(m); if (lineRef.current) lineRef.current.setPath(ptsRef.current); setCanSave(false); setHudCount(); }}>
          撤销
        </button>
        <button className="m-btn sm" onClick={() => { reset(); drawFence(); }}>
          清空
        </button>
        <button className="m-btn sm gold" onClick={doCheck}>
          校验
        </button>
      </div>

      <button className="m-btn sm" disabled={!canSave} style={{ width: "100%" }} onClick={doSave}>
        保存轨迹（覆盖本地已存）
      </button>
      <p className={"task-note" + (noteErr ? " err" : "")}>{note}</p>

    </div>
  );
}
