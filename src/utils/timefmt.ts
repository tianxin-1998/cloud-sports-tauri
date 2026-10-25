export function fmtTime(v: any, fbYear?: any, fbMonth?: any): string {
  const p = (n: number) => String(n).padStart(2, "0");
  const tail = (sec?: string) => (sec ? ":" + sec : "");
  const s = v === undefined || v === null ? "" : String(v).trim();
  if (!s) return "";

  if (typeof v === "number" || /^\d{10,13}$/.test(s)) {
    const n = Number(v);
    const d = new Date(n > 1e12 ? n : n * 1000);
    if (!isNaN(d.getTime()))
      return `${d.getFullYear()}年${p(d.getMonth() + 1)}月${p(d.getDate())}日 ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
  }

  let mt = s.match(/^(\d{4})[-/年](\d{1,2})[-/月](\d{1,2})日?\s+(\d{1,2}):(\d{2})(?::(\d{2}))?/);
  if (mt) return `${mt[1]}年${p(+mt[2])}月${p(+mt[3])}日 ${p(+mt[4])}:${mt[5]}${tail(mt[6])}`;

  mt = s.match(/^(\d{1,2})[-/月](\d{1,2})日?\s+(\d{1,2}):(\d{2})(?::(\d{2}))?/);
  if (mt && fbYear && fbMonth) return `${fbYear}年${p(+fbMonth)}月${p(+mt[1])}日 ${p(+mt[2])}:${mt[3]}${tail(mt[5])}`;

  mt = s.match(/^(\d{1,2})日\s*(\d{1,2}):(\d{2})(?::(\d{2}))?/);
  if (mt && fbYear && fbMonth) return `${fbYear}年${p(+fbMonth)}月${p(+mt[1])}日 ${p(+mt[2])}:${mt[3]}${tail(mt[4])}`;

  const d = new Date(s.replace(/-/g, "/"));
  if (!isNaN(d.getTime()))
    return `${d.getFullYear()}年${p(d.getMonth() + 1)}月${p(d.getDate())}日 ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
  return s;
}
