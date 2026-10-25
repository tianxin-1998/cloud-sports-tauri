import { invoke } from "./client";
import type {
  LatLng,
  TrackListLocalResult,
  TrackMeta,
  TrackQueryResult,
  TrackRefineResult,
  TrackSaveResult,
  TrackStatusResult,
  TrackValidateResult,
} from "../types";

export const trackStatus = (schoolCode: string, raNames: string[]) =>
  invoke<TrackStatusResult>("track_status", { schoolCode, raNames });

export const trackQuery = (schoolCode: string, raName: string) =>
  invoke<TrackQueryResult>("track_query", { schoolCode, raName });

export const trackValidate = (schoolCode: string, raName: string, routeType: string, points: LatLng[]) =>
  invoke<TrackValidateResult>("validate", { schoolCode, raName, routeType, points });

export const trackUpload = (
  schoolCode: string,
  raName: string,
  routeType: string,
  points: LatLng[],
  name: string,
) => invoke<TrackSaveResult>("upload", { schoolCode, raName, routeType, points, name });

export const trackListLocal = (schoolCode: string) =>
  invoke<TrackListLocalResult>("track_list_local", { schoolCode });

export const trackSaveLocal = (schoolCode: string, name: string, meta: TrackMeta, points: LatLng[]) =>
  invoke<TrackSaveResult>("track_save_local", { schoolCode, name, meta, points });

export const trackRefine = (points: LatLng[]) => invoke<TrackRefineResult>("track_refine", { points });
