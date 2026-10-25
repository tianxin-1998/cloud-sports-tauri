import type { UpstreamJson } from "./common";

export interface RunArea {
  id?: string | number;
  raName?: string;
  raRunArea?: string;
  raType?: string;
  points?: string;
  fence?: string;
  raDislikes?: number;
  runFaceStatus?: string;
  raSingleMileageMin?: number;
  raSingleMileageMax?: number;
  raTargetNum?: number | string;
  raMileageNum?: number | string;
  timesKilometers?: string;
  [k: string]: unknown;
}

export interface RunPoint {
  lng: number;
  lat: number;
  mileage: number;
  time: number;
  step: number;
  speed?: string;
  ts?: string;
}

export interface RunProgressPayload {
  status: "running" | "held" | "failed" | "done" | (string & {});
  task_id?: string;
  record_id?: string;
  total?: number;
  uploaded?: number;
  points?: RunPoint[];
  result?: unknown;
}

export interface SimulateResult {
  points: RunPoint[];
  total_mileage_m: number;
  input_length_m: number;
  total_time_s: number;
  total_step: number;
  warnings?: string[];
}

export interface RunActionResult {
  msg: string;
  record_id?: string;
}

export type RunUpstream = UpstreamJson;
