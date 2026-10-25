import { useEffect, useState } from "react";
import { useApp } from "../store";
import { haveYdInfo, sportInfo } from "../services";
import type { RunArea } from "../types";
import LogoMark from "../components/LogoMark";
import StampMark from "../components/StampMark";
import { fmtTime } from "../utils/timefmt";

const LS_HOME_PREF = "yunsport_home_pref";

const REC_PAGE_SIZE = 5;

function loadHomePref(): { areaIdx: string } {
  try {
    const p = JSON.parse(localStorage.getItem(LS_HOME_PREF) || "null");
    if (p && typeof p === "object") {
      return { areaIdx: typeof p.areaIdx === "string" ? p.areaIdx : "" };
    }
  } catch {

  }
  return { areaIdx: "" };
}

export default function HomeView() {
  const { session, openDetail } = useApp();
  const areas = session?.run_areas ?? [];
  const pref = loadHomePref();
  const [areaIdx, setAreaIdx] = useState(areas.length ? (pref.areaIdx || "0") : "");
  const [stats, setStats] = useState<any>({});
  const [yd, setYd] = useState<any>({});

  const [month, setMonth] = useState(0);
  const [page, setPage] = useState(0);
  const [recTip, setRecTip] = useState("登录后查看真实记录");

  const area: RunArea | undefined = areas[Number(areaIdx)];

  const loadStats = async () => {
    if (!session) return setRecTip("登录后查看真实记录");
    setRecTip("加载中…");
    try {
      const d = await sportInfo(session);
      const st = d?.stat ?? d?.data ?? {};
      setStats(st);
      console.log("[stats] getHomeRunInfo.data =", st);

      try {
        const y = await haveYdInfo(session);
        const ydata = y?.data ?? y ?? {};
        setYd(ydata);
        const rank = ydata.rank ?? [];
        setRecTip(rank.length ? "" : "本学期暂无记录");
        console.log("[stats] have_yd_info.data =", ydata);
      } catch {
        setRecTip("加载失败");
      }
    } catch {

    }
  };

  useEffect(() => {
    loadStats();

  }, [session]);

  useEffect(() => {
    if (!areas.length) return;
    if (!areaIdx || Number(areaIdx) >= areas.length) setAreaIdx("0");

  }, [areas.length]);

  useEffect(() => {
    const months = yd.rank ?? [];
    if (!months.length) return;
    if (month >= months.length) setMonth(0);

  }, [yd]);

  useEffect(() => {

    setPage(0);
  }, [month, yd]);

  useEffect(() => {
    try {
      localStorage.setItem(LS_HOME_PREF, JSON.stringify({ areaIdx }));
    } catch {

    }
  }, [areaIdx]);

  const name = session?.realName || session?.nickName || session?.studentId || session?.user || "我";

  const tgRaw = area?.raTargetNum ?? stats.raTargetNum;
  const tg = tgRaw === undefined || tgRaw === null ? "—" : Number(tgRaw);

  const rawDone = yd.sumKm;
  const doneKm = rawDone === undefined || rawDone === null ? "—" : Number(rawDone).toFixed(2);
  const rawTarget = area?.raMileageNum ?? stats.raMileageNum;
  const targetKm = rawTarget === undefined || rawTarget === null ? "—" : Number(rawTarget).toFixed(2);

  const q = yd.sumNumber === undefined || yd.sumNumber === null ? "—" : yd.sumNumber;

  const byKm = (area?.timesKilometers ?? "T") === "K";
  const barW = byKm
    ? doneKm !== "—" && targetKm !== "—" && +targetKm > 0
      ? Math.min((+doneKm / +targetKm) * 100, 100)
      : 0
    : q !== "—" && tg !== "—" && +tg > 0
      ? Math.min((+q / +tg) * 100, 100)
      : 0;

  const months = yd.rank ?? [];
  const curMonth = months[month] ?? {};
  const recList = curMonth.rankList ?? [];

  const pageCount = Math.max(1, Math.ceil(recList.length / REC_PAGE_SIZE));
  const curPage = Math.min(page, pageCount - 1);
  const pageRecs = recList.slice(curPage * REC_PAGE_SIZE, curPage * REC_PAGE_SIZE + REC_PAGE_SIZE);
  const goPage = (p: number) => setPage(Math.max(0, Math.min(p, pageCount - 1)));

  return (
    <div className="view on" id="view-home">

      <div className="greet">
        <div>
          <h3>你好，{name}</h3>
          <small>{[session?.school, session?.className].filter(Boolean).join(" · ") || "已登录"}</small>
        </div>
      </div>

      <div className="app-card task-card">
        <div className="t-bar">
          <svg
            className="t-runner"
            viewBox="0 0 24 24"
            width="24"
            height="24"
            style={{ left: `clamp(0px, calc(${barW}% - 12px), calc(100% - 24px))` }}
          >
            <path
              d="M13.49 5.48c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm-3.6 13.9l1-4.4 2.1 2v6h2v-7.5l-2.1-2 .6-3c1.3 1.5 3.3 2.5 5.5 2.5v-2c-1.9 0-3.5-1-4.3-2.4l-1-1.6c-.4-.6-1-1-1.7-1-.3 0-.5.1-.8.1l-5.2 2.2v4.7h2v-3.4l1.8-.7-1.6 8.1-4.9-1-.4 2 7 1.4z"
              fill="#2563EB"
            />
          </svg>
          <div className="progress t-progress">
            <i style={{ width: barW + "%" }} />
          </div>
        </div>
        <div className="t-km">
          <span className="km-done">{doneKm}km</span>
          <span className="km-target">{targetKm}km</span>
        </div>
        <div className="t-counts">
          <div className="tc">
            <small>有效次数</small>
            <b>{q}</b>
          </div>
          <i className="tc-div" />
          <div className="tc">
            <small>目标次数</small>
            <b>{tg}</b>
          </div>
        </div>
      </div>

      <div className="rec-title">
        <h4>运动记录</h4>
      </div>

      {months.length > 0 && (
        <div className="mchips">
          {months.map((mm: any, i: number) => (
            <button key={i} className={"mchip" + (i === month ? " on" : "")} onClick={() => setMonth(i)}>
              {mm.year}-{mm.month}
            </button>
          ))}
        </div>
      )}

      <div className="app-card" style={{ padding: "4px 16px" }}>
        {months.length === 0 ? (
          <div className="empty-tip">{recTip}</div>
        ) : recList.length === 0 ? (
          <div className="empty-tip">该月暂无记录</div>
        ) : (
          <>
            <div className="rec-list">
              {pageRecs.map((r: any, i: number) => {
                const ok = r.isQualified === "1";
                return (
                  <div key={curPage * REC_PAGE_SIZE + i} className="imp-item rec rec-card" onClick={() => openDetail(r)}>
                    <div className="ic">
                      <LogoMark size={36} />
                    </div>
                    <div className="m">
                      <small>{fmtTime(r.endTime || r.recordEndTime, curMonth.year, curMonth.month)}</small>
                      <span className="ra">📋 {r.raName || ""}</span>
                    </div>
                    <b className="rec-len">{r.recordMileage ?? "—"} km</b>
                    <StampMark ok={ok} size={34} />
                  </div>
                );
              })}
            </div>

            {pageCount > 1 && (
              <div className="rec-pager">
                <button className="pg-btn" disabled={curPage === 0} onClick={() => goPage(curPage - 1)}>
                  上一页
                </button>
                <span className="pg-info">
                  第 {curPage + 1} / {pageCount} 页 · 共 {recList.length} 条
                </span>
                <button className="pg-btn" disabled={curPage >= pageCount - 1} onClick={() => goPage(curPage + 1)}>
                  下一页
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
