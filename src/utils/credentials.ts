const LS_CREDS = "yunsport_creds";
const LS_INVALID = "yunsport_acc_invalid";

export interface StoredCredential {
  user: string;
  password: string;
  school_code: string;
}

export const accKey = (user: string, schoolCode: string) => `${user}@${schoolCode}`;

function readJSON<T>(key: string, fallback: T): T {
  try {
    const v = JSON.parse(localStorage.getItem(key) || "null");
    return v ?? fallback;
  } catch {
    return fallback;
  }
}

function writeJSON(key: string, v: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(v));
  } catch {

  }
}

export function saveCredential(c: StoredCredential): void {
  const m = readJSON<Record<string, StoredCredential>>(LS_CREDS, {});
  m[accKey(c.user, c.school_code)] = { user: c.user, password: c.password, school_code: c.school_code };
  writeJSON(LS_CREDS, m);
}

export function getCredential(user: string, schoolCode: string): string {
  const c = readJSON<Record<string, StoredCredential>>(LS_CREDS, {})[accKey(user, schoolCode)];
  return typeof c?.password === "string" ? c.password : "";
}

export function removeCredential(user: string, schoolCode: string): void {
  const m = readJSON<Record<string, StoredCredential>>(LS_CREDS, {});
  delete m[accKey(user, schoolCode)];
  writeJSON(LS_CREDS, m);
}

export function listInvalidKeys(): string[] {
  const a = readJSON<unknown>(LS_INVALID, []);
  return Array.isArray(a) ? a.filter((x): x is string => typeof x === "string") : [];
}

export function markLoginInvalid(user: string, schoolCode: string): void {
  writeJSON(LS_INVALID, Array.from(new Set([...listInvalidKeys(), accKey(user, schoolCode)])));
}

export function clearLoginInvalid(user: string, schoolCode: string): void {
  const k = accKey(user, schoolCode);
  writeJSON(
    LS_INVALID,
    listInvalidKeys().filter((x) => x !== k),
  );
}

export function isLoginInvalid(user: string, schoolCode: string): boolean {
  return listInvalidKeys().includes(accKey(user, schoolCode));
}
