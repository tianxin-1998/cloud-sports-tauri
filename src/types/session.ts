import type { RunArea } from "./run";

export interface SessionInfo {
  user: string;
  school_code: string;
  school_url: string;
  uuid: string;
  device_name: string;
  token: string;
  realName?: string;
  nickName?: string;
  studentId?: string;
  school?: string;
  campus?: string;
  faculty?: string;
  className?: string;
  run_areas?: RunArea[];
  [k: string]: unknown;
}

export interface ReloginInfo {
  user: string;
  password: string;
  school_code: string;
}

export interface ReloginAsk {
  user: string;
  school_code: string;
  name: string;
}
