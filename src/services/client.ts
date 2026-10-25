import { invoke as tauriInvoke, type InvokeArgs } from "@tauri-apps/api/core";

type AuthExpiredHandler = (msg: string) => void;

let authExpiredHandler: AuthExpiredHandler | null = null;
let authExpiredSuppressed = false;

export function onAuthExpired(handler: AuthExpiredHandler): void {
  authExpiredHandler = handler;
}

export async function withAuthExpiredSuppressed<T>(fn: () => Promise<T>): Promise<T> {
  authExpiredSuppressed = true;
  try {
    return await fn();
  } finally {
    authExpiredSuppressed = false;
  }
}

export function isAuthError(msg: string): boolean {
  return msg.startsWith("AUTH_EXPIRED") || msg.includes("认证失败") || msg.includes("认证已失效");
}

export function notifyAuthExpired(msg: string): void {
  if (authExpiredSuppressed) return;
  if (authExpiredHandler) authExpiredHandler(msg);
}

export async function invoke<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (e: any) {
    const msg = typeof e === "string" ? e : String(e?.message ?? e);
    if (isAuthError(msg)) notifyAuthExpired(msg);
    throw e;
  }
}
