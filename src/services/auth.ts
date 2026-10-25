import { invoke } from "./client";
import type { SessionInfo } from "../types";

export const login = (p: { user: string; password: string; school_code: string }) =>
  invoke<SessionInfo>("login", { user: p.user, password: p.password, schoolCode: p.school_code });

export const saveSession = (session: SessionInfo) => invoke<void>("save_session", { session });

export const loadSessionInfo = () => invoke<SessionInfo | null>("load_session_info");

export const hasCliSession = () => invoke<boolean>("has_cli_session");

export const clearCliSession = () => invoke<void>("clear_cli_session");

export const sessionAccounts = () => invoke<SessionInfo[]>("session_accounts");

export const sessionGet = (user: string, schoolCode: string) =>
  invoke<SessionInfo | null>("session_get", { user, schoolCode });

export const sessionRemove = (user: string, schoolCode: string) =>
  invoke<void>("session_remove", { user, schoolCode });
