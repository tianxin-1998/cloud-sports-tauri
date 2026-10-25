const STAR_ANGLES = [205, 237.5, 270, 302.5, 335];

export default function StampMark({ ok = true, size = 46 }: { ok?: boolean; size?: number }) {
  const color = ok ? "#16a34a" : "#dc2626";
  const label = ok ? "达标" : "不合格";
  const cx = 30;
  const cy = 30;
  const r = 25;
  return (
    <svg viewBox="0 0 60 60" width={size} height={size} aria-hidden style={{ display: "block", transform: "rotate(-10deg)" }}>
      <circle cx={cx} cy={cy} r="28" fill="none" stroke={color} strokeWidth="2.2" />
      <circle cx={cx} cy={cy} r="21.5" fill="none" stroke={color} strokeWidth="0.9" />

      {STAR_ANGLES.map((deg) => {
        const rad = (deg * Math.PI) / 180;
        const x = cx + r * Math.cos(rad);
        const y = cy + r * Math.sin(rad);
        return (
          <text key={deg} x={x} y={y} textAnchor="middle" dominantBaseline="central" fontSize="5.5" fill={color}>
            ★
          </text>
        );
      })}

      <text
        x={cx}
        y={cy}
        textAnchor="middle"
        dominantBaseline="central"
        fontSize={ok ? 15 : 10.5}
        fontWeight={800}
        letterSpacing={ok ? 2 : 0.5}
        fill={color}
        style={{ fontFamily: "-apple-system, sans-serif" }}
      >
        {label}
      </text>
    </svg>
  );
}
