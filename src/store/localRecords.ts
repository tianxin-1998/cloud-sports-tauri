import { haveYdInfo, recordTrack, trackSaveLocal } from "../services";
import type { SessionInfo, TrackMeta } from "../types";

const LS_RECORDS_SAVED = "yunsport_records_saved";

function nowStamp(): string {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

function isRecordsSaved(key: string): boolean {
  try {
    return !!JSON.parse(localStorage.getItem(LS_RECORDS_SAVED) || "{}")[key];
  } catch {
    return false;
  }
}

function markRecordsSaved(key: string): void {
  try {
    const m = JSON.parse(localStorage.getItem(LS_RECORDS_SAVED) || "{}");
    m[key] = 1;
    localStorage.setItem(LS_RECORDS_SAVED, JSON.stringify(m));
  } catch {

  }
}

export async function saveRecordsLocally(session: SessionInfo | null): Promise<void> {
  const user = session?.user || "";
  const schoolCode = session?.school_code || "";
  if (!user || !schoolCode) return;

  const key = `${user}@${schoolCode}`;
  if (isRecordsSaved(key)) return;

  try {
    const d = await haveYdInfo(session as SessionInfo);
    const data = d?.data ?? d ?? {};
    const months: any[] = data.rank ?? [];
    const all: any[] = months.flatMap((m: any) => (m.rankList || []).map((r: any) => ({ ...r })));

    for (const r of all) {
      const id = r.id ?? r.recordId;
      if (!id) continue;
      try {
        const tr = await recordTrack(session as SessionInfo, String(id));
        const pts = tr?.points;
        if (!pts || pts.length < 2) continue;
        const meta: TrackMeta = {
          source: "record",
          record_id: String(id),
          ra_name: r.raName || "",
          record_mileage: r.recordMileage ?? null,
          is_qualified: r.isQualified ?? "",
          record_end_time: r.endTime || r.recordEndTime || null,
          created_at: nowStamp(),
        };
        await trackSaveLocal(schoolCode, `rec_${id}`, meta, pts);
      } catch {

      }
    }
    markRecordsSaved(key);
  } catch {

  }
}

export function resetRecordsSaved(user: string, schoolCode: string): void {
  try {
    const m = JSON.parse(localStorage.getItem(LS_RECORDS_SAVED) || "{}");
    delete m[`${user}@${schoolCode}`];
    localStorage.setItem(LS_RECORDS_SAVED, JSON.stringify(m));
  } catch {

  }
}
