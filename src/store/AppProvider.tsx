import { useEffect, useRef, useState, type ReactNode } from "react";
import { useAuthExpired } from "../hooks";
import {
  clearCliSession,
  getConfig,
  getSchools,
  loadSessionInfo,
  saveConfig,
  saveSession,
  sessionRemove,
} from "../services";
import {
  DEFAULT_APP_CONFIG,
  type AppConfig,
  type ReloginAsk,
  type ReloginInfo,
  type School,
  type SessionInfo,
} from "../types";
import {
  clearLoginInvalid,
  getCredential,
  markLoginInvalid,
} from "../utils/credentials";
import { AppContext, type AppContextValue, type ViewName } from "./context";
import { saveRecordsLocally } from "./localRecords";

export function AppProvider({ children }: { children: ReactNode }) {
  const [session, setSessionState] = useState<SessionInfo | null>(null);
  const [cfg, setCfgState] = useState<AppConfig>(DEFAULT_APP_CONFIG);
  const [schools, setSchools] = useState<School[]>([]);
  const [view, setView] = useState<ViewName>("login");
  const [detailRec, setDetailRec] = useState<any>(null);
  const [authHint, setAuthHint] = useState<string>("");
  const [relogin, setRelogin] = useState<ReloginInfo | null>(null);
  const [reloginAsk, setReloginAsk] = useState<ReloginAsk | null>(null);

  const sessionRef = useRef<SessionInfo | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const c = await getConfig();
        setCfgState({ ...DEFAULT_APP_CONFIG, ...c });
      } catch {

      }
      try {
        const s = await loadSessionInfo();
        if (s && s.token) {
          setSession(s);
          setView("home");
        }
      } catch {

      }
      try {
        const d = await getSchools();
        setSchools(d.schools || []);
      } catch {

      }
    })();
  }, []);

  useAuthExpired(() => {
    const s = sessionRef.current;
    sessionRef.current = null;
    setSessionState(null);
    setAuthHint("登录已失效，请重新登录（token 已过期）");
    if (s) void clearCliSession();
    setView("login");
  });

  const setSession = (s: SessionInfo | null) => {
    const prev = sessionRef.current;
    sessionRef.current = s;
    setSessionState(s);
    if (s) setAuthHint("");
    if (s) {
      void saveSession(s);
      void saveRecordsLocally(s);
    } else {
      if (prev) void sessionRemove(prev.user, prev.school_code).catch(() => {});
      void clearCliSession();
    }
    if (!s) setView("login");
  };

  const setCfg = (c: AppConfig) => {
    setCfgState(c);
    void saveConfig(c);
  };

  const show = (v: ViewName) => {

    if (!sessionRef.current && v !== "set" && v !== "login") {
      setView("login");
      return;
    }
    setView(v);
  };

  const openDetail = (rec: any) => {
    setDetailRec(rec);
    setView("detail");
  };

  const askRelogin = (a: ReloginAsk) => setReloginAsk(a);

  const confirmRelogin = () => {
    const a = reloginAsk;
    if (!a) return;
    setReloginAsk(null);
    clearLoginInvalid(a.user, a.school_code);
    setRelogin({ user: a.user, password: getCredential(a.user, a.school_code), school_code: a.school_code });
    setAuthHint(`${a.name}（${a.user}）登录已失效，请重新登录`);
    setView("login");
  };

  const cancelRelogin = () => {
    const a = reloginAsk;
    if (!a) return;
    setReloginAsk(null);
    markLoginInvalid(a.user, a.school_code);
    setAuthHint(`已将 ${a.name}（${a.user}）标记为登录失效`);
  };

  const value: AppContextValue = {
    session,
    setSession,
    cfg,
    setCfg,
    schools,
    view,
    show,
    detailRec,
    openDetail,
    authHint,
    setAuthHint,
    relogin,
    setRelogin,
    reloginAsk,
    askRelogin,
    confirmRelogin,
    cancelRelogin,
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}
