import { invoke } from "./client";
import type { RunActionResult, RunPoint, SessionInfo, SimulateResult, UpstreamJson } from "../types";

export interface YunRunInput {
  session: SessionInfo;
  target_distance: number;
  ra_name: string;
  fast: boolean;
  face_base64?: string;
  hold_finish?: boolean;
  points?: RunPoint[];
  submit_distance?: number;
}

export const yunRun = (body: YunRunInput) =>
  invoke<UpstreamJson>("run", {
    session: body.session,
    targetDistance: body.target_distance,
    raName: body.ra_name,
    fast: body.fast,
    faceBase64: body.face_base64,
    holdFinish: body.hold_finish ?? false,
    points: body.points,
    submitDistance: body.submit_distance,
  });

export const finishRun = (recordId: string) => invoke<RunActionResult>("finish_run", { recordId });

export const discardRun = (recordId: string) => invoke<RunActionResult>("discard_run", { recordId });

export const simulate = (p: {
  session: SessionInfo;
  targetDistance: number;
  raName: string;
  fast: boolean;
  points?: { lng: number; lat: number }[];
}) => invoke<SimulateResult>("simulate", p);
