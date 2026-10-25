import type { AppConfig } from "../types";

declare global {
  interface Window {
    AMap?: any;
    _AMapSecurityConfig?: { securityJsCode: string };
  }
}

const AMAP_SCRIPT_ID = "amap-jsapi-script";
const AMAP_TIMEOUT_MS = 10000;

let loadedKey = "";

export async function loadAMap(cfg: AppConfig): Promise<void> {
  if (window.AMap && loadedKey === cfg.amap_key) return;
  if (!cfg.amap_key) throw new Error("请先在「设置」填入高德 Key");
  if (cfg.amap_security) window._AMapSecurityConfig = { securityJsCode: cfg.amap_security };
  document.getElementById(AMAP_SCRIPT_ID)?.remove();
  window.AMap = undefined;
  await new Promise<void>((resolve, reject) => {
    const s = document.createElement("script");
    s.id = AMAP_SCRIPT_ID;
    s.src = "https://webapi.amap.com/maps?v=2.0&key=" + encodeURIComponent(cfg.amap_key);
    s.onload = () => resolve();
    s.onerror = () => reject(new Error("高德 SDK 加载失败"));
    document.head.appendChild(s);
  });
  loadedKey = cfg.amap_key;
}

export type AmapCheckLevel = "ok" | "warn" | "error";

export interface AmapCheckResult {
  level: AmapCheckLevel;
  code: string;
  msg: string;
}

const AMAP_PROBE_ADDRESS = "北京市海淀区中关村";

const AMAP_ERR_TEXT: Record<string, string> = {
  INVALID_USER_KEY: "Key 不正确或已过期，请到控制台核对 Key",
  USER_KEY_RECYCLED: "Key 已被删除，请重新申请",
  INVALID_USER_SCODE: "安全密钥与 Key 不匹配：2021-12-02 后申请的 Key 必须与控制台生成的安全密钥配对使用",
  INVALID_USER_DOMAIN: "Key 绑定的域名与当前运行环境不符，请在控制台补充域名白名单",
  USERKEY_PLAT_NOMATCH: "Key 类型不匹配：地图需「Web端(JS API)」类型的 Key，Web服务 Key 不能用于 JS API",
  SERVICE_NOT_AVAILABLE: "该 Key 无地理编码服务权限，请到控制台开通对应服务",
  INSUFFICIENT_PRIVILEGES: "权限不足，服务请求被拒绝",
  DAILY_QUERY_OVER_LIMIT: "今日调用量已超限，次日 0 点自动恢复",
  USER_DAILY_QUERY_OVER_LIMIT: "今日调用量已超限，次日 0 点自动恢复",
  ACCESS_TOO_FREQUENT: "调用过于频繁，请稍后重试",
  NOT_SUPPORT_HTTPS: "当前 Key 不支持 HTTPS 请求",
  INVALID_PARAMS: "请求参数非法",
  MISSING_REQUIRED_PARAMS: "缺少必填参数",
  UNKNOWN_ERROR: "高德返回未知错误",
  NETWORK_ERROR: "无法连接高德服务，请检查网络或代理设置",
  SDK_LOAD_FAIL: "高德 SDK 或插件加载失败/超时，请检查网络能否访问 webapi.amap.com",
};

async function probeKeyByRest(key: string): Promise<string> {
  const ctrl = new AbortController();
  const timer = window.setTimeout(() => ctrl.abort(), AMAP_TIMEOUT_MS);
  try {
    const res = await fetch(`https://restapi.amap.com/v3/ip?key=${encodeURIComponent(key)}`, {
      signal: ctrl.signal,
      cache: "no-store",
    });
    const body: any = await res.json();
    if (body?.info === "USERKEY_PLAT_NOMATCH") return "KEY_TYPE_OK";

    if (body?.status === "1") return "WEB_SERVICE_KEY";
    return String(body?.info ?? "UNKNOWN_ERROR");
  } catch {
    return "NETWORK_ERROR";
  } finally {
    window.clearTimeout(timer);
  }
}

async function reachable(url: string): Promise<boolean> {
  const ctrl = new AbortController();
  const timer = window.setTimeout(() => ctrl.abort(), 6000);
  try {
    await fetch(url, { mode: "no-cors", signal: ctrl.signal, cache: "no-store" });
    return true;
  } catch {
    return false;
  } finally {
    window.clearTimeout(timer);
  }
}

async function probeGeocoder(): Promise<string> {
  await new Promise<void>((resolve, reject) => {
    const timer = window.setTimeout(() => reject(new Error("TIMEOUT")), AMAP_TIMEOUT_MS);
    window.AMap!.plugin(["AMap.Geocoder"], () => {
      window.clearTimeout(timer);
      resolve();
    });
  });

  return new Promise<string>((resolve) => {
    let done = false;
    const finish = (v: string) => {
      if (done) return;
      done = true;
      resolve(v);
    };
    const timer = window.setTimeout(() => finish("TIMEOUT"), AMAP_TIMEOUT_MS);
    try {
      new window.AMap!.Geocoder({}).getLocation(AMAP_PROBE_ADDRESS, (status: string, result: any) => {
        window.clearTimeout(timer);
        finish(status === "complete" && result?.info === "OK" ? "OK" : String(result?.info ?? "UNKNOWN_ERROR"));
      });
    } catch {
      window.clearTimeout(timer);
      finish("UNKNOWN_ERROR");
    }
  });
}

export async function checkAMapCredentials(cfg: AppConfig): Promise<AmapCheckResult> {
  const key = (cfg.amap_key || "").trim();
  const sec = (cfg.amap_security || "").trim();
  if (!key) return { level: "error", code: "MISSING_KEY", msg: "未填写高德 Key" };

  const probe = await probeKeyByRest(key);
  if (probe === "WEB_SERVICE_KEY") {
    return { level: "error", code: "USERKEY_PLAT_NOMATCH", msg: "该 Key 是「Web服务」类型，地图需要「Web端(JS API)」类型的 Key" };
  }
  if (probe !== "KEY_TYPE_OK") {
    return { level: "error", code: probe, msg: AMAP_ERR_TEXT[probe] ?? `Key 校验失败：${probe}` };
  }

  if (sec) window._AMapSecurityConfig = { securityJsCode: sec };
  else delete window._AMapSecurityConfig;

  let info: string;
  try {
    await loadAMap({ ...cfg, amap_key: key, amap_security: sec });
    info = await probeGeocoder();
  } catch {
    return { level: "error", code: "SDK_LOAD_FAIL", msg: AMAP_ERR_TEXT.SDK_LOAD_FAIL };
  }

  if (info === "OK") {
    return {
      level: "ok",
      code: "OK",
      msg: sec ? "Key 与安全密钥校验通过" : "Key 校验通过（未填安全密钥；若控制台已生成请一并填写）",
    };
  }
  if (info === "TIMEOUT") {
    const netOk = await reachable("https://jsapi-service.amap.com/");
    return {
      level: "warn",
      code: info,
      msg: netOk
        ? "Key 有效，但地图服务请求无响应：多半是安全密钥与 Key 不匹配，或控制台域名白名单未放行当前环境"
        : "Key 有效，但无法连接地图服务域名 jsapi-service.amap.com，请检查网络或代理",
    };
  }
  return { level: "error", code: info, msg: AMAP_ERR_TEXT[info] ?? `高德返回：${info}` };
}

export function mapErr(box: HTMLElement, msg: string) {
  box.querySelectorAll(".map-err").forEach((e) => e.remove());
  const d = document.createElement("div");
  d.className = "map-err";
  d.textContent = msg;
  box.appendChild(d);
}

export function mapClear(box: HTMLElement) {
  box.querySelectorAll(".map-err").forEach((e) => e.remove());
}
