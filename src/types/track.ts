import type { AnyRecord, LatLng } from "./common";

export interface TrackLocalItem {
  name: string;
  distance_m: number;
  points: LatLng[];
  created_at?: string;
  ra_name?: string;
  record_mileage?: string | number;
  is_qualified?: string;
  record_id?: string;
}

export interface TrackListLocalResult {
  items: TrackLocalItem[];
}

export interface TrackQueryResult {
  found: boolean;
  file?: string;
  points?: LatLng[];
}

export interface TrackValidateResult {
  distance_m: number;
  points: LatLng[];
}

export interface TrackSaveResult {
  name: string;
  distance_m: number;
}

export interface TrackRefineResult {
  points: LatLng[];
  distance_m: number;
}

export interface TrackStatusResult {
  missing: string[];
}

export type TrackMeta = AnyRecord;
