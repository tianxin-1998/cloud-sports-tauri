import { createContext, useContext } from "react";
import type { AppConfig, ReloginAsk, ReloginInfo, School, SessionInfo } from "../types";

export type ViewName = "login" | "home" | "set" | "draw" | "detail" | "sim";

export interface AppContextValue {
  session: SessionInfo | null;
  setSession: (s: SessionInfo | null) => void;
  cfg: AppConfig;
  setCfg: (c: AppConfig) => void;
  schools: School[];
  view: ViewName;
  show: (v: ViewName) => void;
  detailRec: any;
  openDetail: (rec: any) => void;
  authHint: string;
  setAuthHint: (s: string) => void;
  relogin: ReloginInfo | null;
  setRelogin: (v: ReloginInfo | null) => void;
  reloginAsk: ReloginAsk | null;
  askRelogin: (a: ReloginAsk) => void;
  confirmRelogin: () => void;
  cancelRelogin: () => void;
}

export const AppContext = createContext<AppContextValue | null>(null);

export function useApp(): AppContextValue {
  return useContext(AppContext)!;
}
