import { useEffect, useRef, useState } from "react";
import { useApp } from "../store";
import {
  faceAdd,
  faceDelete,
  faceGet,
  faceList,
  getHomeRunInfo,
  login,
  sessionAccounts,
  sessionGet,
  withAuthExpiredSuppressed,
} from "../services";
import { checkAMapCredentials } from "../utils/amap";
import { accKey, clearLoginInvalid, getCredential, listInvalidKeys } from "../utils/credentials";
import type { SessionInfo } from "../types";

const SHOW_FACE_CARD = false;

const PACE_RECOMMEND = [5.5, 6.7];
const CADENCE_RECOMMEND = [164, 176];

function compressImage(file: File, maxSide: number, quality: number): Promise<string> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      const sc = Math.min(1, maxSide / Math.max(img.width, img.height));
      const c = document.createElement("canvas");
      c.width = Math.round(img.width * sc);
      c.height = Math.round(img.height * sc);
      c.getContext("2d")!.drawImage(img, 0, 0, c.width, c.height);
      resolve(c.toDataURL("image/jpeg", quality).split(",")[1]);
    };
    img.onerror = () => reject(new Error("图片读取失败"));
    img.src = URL.createObjectURL(file);
  });
}

export default function SettingsView() {
  const { session, cfg, setCfg, setSession, show, schools, reloginAsk, askRelogin } = useApp();
  const [faces, setFaces] = useState<{ name: string }[]>([]);
  const [amapKey, setAmapKey] = useState(cfg.amap_key);
  const [amapSec, setAmapSec] = useState(cfg.amap_security);

  const [paceMin, setPaceMin] = useState(cfg.pace_min || PACE_RECOMMEND[0]);
  const [paceMax, setPaceMax] = useState(cfg.pace_max || PACE_RECOMMEND[1]);
  const [cadMin, setCadMin] = useState(cfg.cadence_min || CADENCE_RECOMMEND[0]);
  const [cadMax, setCadMax] = useState(cfg.cadence_max || CADENCE_RECOMMEND[1]);
  const [conn, setConn] = useState("未检测");
  const [checking, setChecking] = useState(false);
  const [faceHint, setFaceHint] = useState("登录后根据跑区自动判断是否需要刷脸。");
  const fileRef = useRef<HTMLInputElement>(null);

  const [faceImgs, setFaceImgs] = useState<Record<string, string>>({});

  const renderFaces = async () => {
    try {
      const d = await faceList();
      const list = d.photos ?? [];
      setFaces(list);
      const map: Record<string, string> = {};
      await Promise.all(
        list.map(async (ph) => {
          try {
            const r = await faceGet(ph.name);
            map[ph.name] = "data:image/jpeg;base64," + r.data_base64;
          } catch {

          }
        }),
      );
      setFaceImgs(map);
    } catch {

    }
  };

  useEffect(() => {

    if (SHOW_FACE_CARD) renderFaces();

  }, []);

  useEffect(() => {
    if (!session) return setFaceHint("登录后根据跑区自动判断是否需要刷脸。");
    const need = (session.run_areas ?? []).some((a) => a.runFaceStatus === "Y");
    if (need) setFaceHint("当前账号存在需要刷脸的跑区，提交跑步时会按顺序轮询携带下方照片。");
    else setFaceHint("提示：当前账号所有跑区均无需刷脸。仍可添加照片，提交时若学校要求人脸会自动轮询携带。");
  }, [session]);

  const currentCfg = () => ({
    ...cfg,
    amap_key: amapKey,
    amap_security: amapSec,
    pace_min: paceMin,
    pace_max: paceMax,
    cadence_min: cadMin,
    cadence_max: cadMax,
  });

  const saveCfg = () => setCfg(currentCfg());

  const saveCfgWithCheck = async () => {
    setCfg(currentCfg());
    if (!amapKey.trim()) {
      setConn("已保存 · 未填写高德 Key，地图与轨迹绘制不可用");
      return;
    }
    setChecking(true);
    setConn("正在检测高德 Key 与安全密钥…");
    try {
      const r = await checkAMapCredentials(currentCfg());
      const icon = r.level === "ok" ? "✅ " : r.level === "warn" ? "⚠️ " : "❌ ";
      setConn(`已保存 · ${icon}${r.msg}`);
    } catch (e: any) {
      setConn("已保存，但高德配置检测失败：" + String(e?.message ?? e));
    } finally {
      setChecking(false);
    }
  };

  const onFiles = async (files: FileList) => {
    for (const f of Array.from(files)) {
      try {
        await faceAdd(await compressImage(f, 512, 0.85));
      } catch {

      }
    }
    renderFaces();
  };

  const [accounts, setAccounts] = useState<SessionInfo[]>([]);
  const [switchHint, setSwitchHint] = useState("");
  const reloadAccounts = async () => {
    try {
      setAccounts(await sessionAccounts());
    } catch {
      setAccounts([]);
    }
  };
  useEffect(() => {
    reloadAccounts();

  }, []);
  const accName = (a: SessionInfo) => a.realName || a.nickName || a.studentId || a.user || "未知";

  const schoolDisplay = (a: SessionInfo) => {
    const nm = a.school || schools.find((s) => s.schoolCode === a.school_code)?.schoolName || "";
    return nm ? `${nm}（${a.school_code}）` : `学校编码 ${a.school_code}`;
  };
  const isCurrent = (a: SessionInfo) => !!session && a.user === session.user && a.school_code === session.school_code;

  const [invalidKeys, setInvalidKeys] = useState<string[]>(() => listInvalidKeys());
  useEffect(() => {

    setInvalidKeys(listInvalidKeys());
  }, [reloginAsk]);

  const switchAccount = async (a: SessionInfo) =>
    withAuthExpiredSuppressed(async () => {
      if (isCurrent(a)) return;
      setSwitchHint(`正在切换至 ${accName(a)}…`);

      let s: SessionInfo | null = null;
      try {
        const saved = await sessionGet(a.user, a.school_code);
        if (saved?.token) {
          try {
            const areas = await getHomeRunInfo(saved);
            s = { ...saved, run_areas: areas };
          } catch {

          }
        }
      } catch {

      }

      if (!s) {
        const pwd = getCredential(a.user, a.school_code);
        if (pwd) {
          try {
            const fresh = await login({ user: a.user, password: pwd, school_code: a.school_code });
            if (fresh?.token) {
              try {
                s = { ...fresh, run_areas: await getHomeRunInfo(fresh) };
              } catch {
                s = fresh;
              }
            }
          } catch (e) {
            console.warn("[switch] 自动重登失败:", e);
          }
        }
      }

      if (s?.token) {
        clearLoginInvalid(a.user, a.school_code);
        setInvalidKeys(listInvalidKeys());
        setSession(s);
        setSwitchHint(`已切换至 ${accName(s)}${s.user !== a.user ? `（${a.user}）` : ""}`);
        reloadAccounts();
        return;
      }

      setSwitchHint("");
      askRelogin({ user: a.user, school_code: a.school_code, name: accName(a) });
    });

  return (
    <div className="view on" id="view-set">
      {SHOW_FACE_CARD && (
      <div className="app-card">
        <span className="v-label">人脸照片（提交时轮询携带）</span>
        <div className="set-hint" style={{ marginTop: 8 }}>
          {faceHint}
        </div>
        <div className="fthumbs">
          {faces.length === 0 && (
            <div style={{ gridColumn: "1/-1", fontSize: 12, color: "var(--muted)", padding: "6px 2px" }}>暂无照片</div>
          )}
          {faces.map((ph, i) => (
            <div key={ph.name} className="fthumb">
              <img alt={ph.name} src={faceImgs[ph.name]} style={faceImgs[ph.name] ? undefined : { visibility: "hidden" }} />
              <button className="del" onClick={async () => { await faceDelete(ph.name); renderFaces(); }}>
                ✕
              </button>
              <span className="idx">{i + 1}</span>
            </div>
          ))}
        </div>
        <input ref={fileRef} type="file" accept="image/*" multiple hidden onChange={(e) => e.target.files && onFiles(e.target.files)} />
        <button className="m-btn sm" style={{ width: "100%" }} onClick={() => fileRef.current?.click()}>
          添加照片
        </button>
      </div>
      )}

      <div className="app-card">
        <div className="set-row" style={{ marginTop: 8 }}>
          <label>高德地图 Key</label>
          <input placeholder="Web端(JS API) Key" value={amapKey} onChange={(e) => setAmapKey(e.target.value)} />
        </div>
        <div className="set-row">
          <label>安全密钥</label>
          <input placeholder="securityJsCode（可选）" value={amapSec} onChange={(e) => setAmapSec(e.target.value)} />
        </div>
        <button className="m-btn sm" style={{ width: "100%", marginTop: 2 }} disabled={checking} onClick={() => void saveCfgWithCheck()}>
          {checking ? "保存并检测中…" : "保存设置"}
        </button>
        <div className="set-info">{conn}</div>
      </div>

      <div className="app-card">
        <div className="set-grid2">
          <div className="set-row">
            <label>配速上限（最快）</label>
            <div className="set-in">
              <input type="number" step="0.1" min={1} max={20} value={paceMin} onChange={(e) => setPaceMin(Number(e.target.value))} />
              <i>推荐 {PACE_RECOMMEND[0]}</i>
            </div>
          </div>
          <div className="set-row">
            <label>配速下限（最慢）</label>
            <div className="set-in">
              <input type="number" step="0.1" min={1} max={20} value={paceMax} onChange={(e) => setPaceMax(Number(e.target.value))} />
              <i>推荐 {PACE_RECOMMEND[1]}</i>
            </div>
          </div>
          <div className="set-row">
            <label>步频下限</label>
            <div className="set-in">
              <input type="number" step="1" min={60} max={260} value={cadMin} onChange={(e) => setCadMin(Number(e.target.value))} />
              <i>推荐 {CADENCE_RECOMMEND[0]}</i>
            </div>
          </div>
          <div className="set-row">
            <label>步频上限</label>
            <div className="set-in">
              <input type="number" step="1" min={60} max={260} value={cadMax} onChange={(e) => setCadMax(Number(e.target.value))} />
              <i>推荐 {CADENCE_RECOMMEND[1]}</i>
            </div>
          </div>
        </div>
        <button className="m-btn sm" style={{ width: "100%" }} onClick={saveCfg}>
          保存跑步参数
        </button>
      </div>

      <div className="app-card">
        {accounts.length === 0 ? (
          <div className="set-info">暂无本机账号，点「添加账号」登录后自动记住</div>
        ) : (
          <div style={{ display: "grid", gap: 6, marginTop: 8 }}>
            {accounts.map((a) => {

              const invalid = invalidKeys.includes(accKey(a.user, a.school_code));
              return (
                <div
                  key={`${a.user}-${a.school_code}`}
                  style={{ display: "flex", alignItems: "center", gap: 8, border: "1px solid var(--line)", borderRadius: 12, padding: "8px 10px" }}
                >
                  <div style={{ flex: 1, minWidth: 0, textAlign: "left" }}>
                    <b style={{ fontSize: 13 }}>
                      {accName(a)}
                      {isCurrent(a) && <span style={{ color: "var(--primary)", fontSize: 11, marginLeft: 6 }}>当前</span>}
                      {invalid && <span className="acc-badge">登录失效</span>}
                    </b>
                    <div style={{ fontSize: 11, color: "var(--muted)" }}>
                      {a.user} · {schoolDisplay(a)}
                    </div>
                  </div>
                  {!isCurrent(a) &&
                    (invalid ? (
                      <button className="m-btn sm" onClick={() => askRelogin({ user: a.user, school_code: a.school_code, name: accName(a) })}>
                        重新登录
                      </button>
                    ) : (
                      <button className="m-btn sm" onClick={() => switchAccount(a)}>
                        切换
                      </button>
                    ))}
                </div>
              );
            })}
          </div>
        )}
        {switchHint && <div className="set-info" style={{ padding: "2px 0 8px" }}>{switchHint}</div>}

        <div style={{ display: "flex", gap: 8 }}>
          <button className="m-btn sm" style={{ flex: "0 0 auto" }} onClick={() => show("login")}>
            添加账号
          </button>
          <button
            className="m-btn danger"
            style={{ flex: 1 }}
            disabled={!session}
            onClick={() => {
              setSwitchHint("");
              setSession(null);
            }}
          >
            退出登录
          </button>
        </div>
      </div>
    </div>
  );
}
