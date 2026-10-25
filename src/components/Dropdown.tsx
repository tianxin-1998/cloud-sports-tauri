import { useEffect, useMemo, useRef, useState } from "react";
import { pinyin } from "pinyin-pro";

export interface Option {
  value: string;
  label: string;
  sub?: string;
}

interface Props {
  options: Option[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  searchable?: boolean;
  searchPlaceholder?: string;
}

function pinyinIndex(text: string): { full: string; initial: string } {
  const full = pinyin(text, { toneType: "none", type: "array", nonZh: "consecutive" }).join("").toLowerCase();
  const initial = pinyin(text, { pattern: "first", toneType: "none", type: "array", nonZh: "consecutive" }).join("").toLowerCase();
  return { full, initial };
}

export default function Dropdown({
  options,
  value,
  onChange,
  placeholder = "请选择",
  searchable = false,
  searchPlaceholder = "搜索…",
}: Props) {
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const [hi, setHi] = useState(-1);
  const ref = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const pyMap = useMemo(() => {
    const m = new Map<string, { full: string; initial: string }>();
    for (const o of options) m.set(o.value, pinyinIndex(o.label));
    return m;
  }, [options]);

  useEffect(() => {
    const onDoc = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("click", onDoc);
    return () => document.removeEventListener("click", onDoc);
  }, []);

  const cur = options.find((o) => o.value === value);
  const label = cur ? cur.label : placeholder;
  const f = q.trim().toLowerCase();
  const list = useMemo(
    () =>
      options.filter((o) => {
        if (!f) return true;
        if (o.label.toLowerCase().includes(f) || (o.sub ?? "").toLowerCase().includes(f)) return true;
        const py = pyMap.get(o.value);
        return !!py && (py.full.includes(f) || py.initial.includes(f));
      }),
    [options, f, pyMap],
  );

  useEffect(() => {
    if (!open) return;
    const i = list.findIndex((o) => o.value === value);
    setHi(i >= 0 ? i : list.length ? 0 : -1);

  }, [open, q]);

  useEffect(() => {
    if (hi < 0 || !listRef.current) return;
    const el = listRef.current.querySelector<HTMLElement>(`[data-idx="${hi}"]`);
    el?.scrollIntoView({ block: "nearest" });
  }, [hi]);

  const pick = (o: Option) => {
    onChange(o.value);
    setOpen(false);
    setQ("");
  };

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (!open) return setOpen(true);
      setHi((i) => (list.length ? (i + 1) % list.length : -1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (!open) return setOpen(true);
      setHi((i) => (list.length ? (i - 1 + list.length) % list.length : -1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (!open) return setOpen(true);
      if (hi >= 0 && list[hi]) pick(list[hi]);
    } else if (e.key === "Escape") {
      setOpen(false);
    }
  };

  return (
    <div className="dd" ref={ref}>
      <button type="button" className="dd-btn" onClick={(e) => { e.stopPropagation(); setOpen((o) => !o); }}>
        <span>{label}</span>
        <i>▾</i>
      </button>
      {open && (
        <div className="dd-panel" onClick={(e) => e.stopPropagation()} onKeyDown={onKey}>
          {searchable && (
            <input autoFocus placeholder={searchPlaceholder} value={q} onChange={(e) => setQ(e.target.value)} />
          )}
          <div className="dd-list" ref={listRef}>
            {list.map((o, idx) => (
              <button
                key={o.value}
                type="button"
                data-idx={idx}
                className={"dd-item" + (o.value === value ? " on" : "") + (idx === hi ? " hi" : "")}
                onMouseEnter={() => setHi(idx)}
                onClick={() => pick(o)}
              >
                {o.label}
                {o.sub ? <i>{o.sub}</i> : null}
              </button>
            ))}
            {list.length === 0 && <div className="dd-empty">无匹配项</div>}
          </div>
        </div>
      )}
    </div>
  );
}
