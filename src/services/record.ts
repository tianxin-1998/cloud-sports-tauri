import { invoke } from "./client";
import type { LatLng, RunArea, SessionInfo, UpstreamJson } from "../types";

export const getHomeRunInfo = (session: SessionInfo) => invoke<RunArea[]>("get_home_run_info", { session });

export const sportInfo = (session: SessionInfo) => invoke<UpstreamJson>("get_sport_info", { session });

export const haveYdInfo = (session: SessionInfo) => invoke<UpstreamJson>("have_yd_info", { session });

export const runRecordDetail = (session: SessionInfo, recordId: string) =>
  invoke<UpstreamJson>("get_run_record_detail", { session, recordId });

export const recordTrack = (session: SessionInfo, recordId: string) =>
  invoke<{ points: LatLng[] }>("record_track", { session, recordId });
