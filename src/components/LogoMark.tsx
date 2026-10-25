export default function LogoMark({ size = 44 }: { size?: number }) {
  return (
    <svg
      viewBox="0 0 48 48"
      width={size}
      height={size}
      aria-hidden
      style={{ display: "block", borderRadius: Math.round(size * 0.28) }}
    >
      <defs>
        <linearGradient id="lm-bg" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#818CF8" />
          <stop offset="1" stopColor="#6366F1" />
        </linearGradient>
        <radialGradient id="lm-hl" cx="0.28" cy="0.24" r="0.9">
          <stop offset="0" stopColor="#ffffff" stopOpacity="0.45" />
          <stop offset="0.55" stopColor="#ffffff" stopOpacity="0" />
        </radialGradient>
      </defs>
      <rect x="0" y="0" width="48" height="48" rx="13" fill="url(#lm-bg)" />
      <rect x="0" y="0" width="48" height="48" rx="13" fill="url(#lm-hl)" />

      <path d="M6.5 15.5h8M5 21h7" stroke="rgba(255,255,255,.55)" strokeWidth="2.4" strokeLinecap="round" fill="none" />

      <g transform="translate(4.6 4.8) scale(1.6)" fill="#fff">
        <path d="M13.49 5.48c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm-3.6 13.9l1-4.4 2.1 2v6h2v-7.5l-2.1-2 .6-3c1.3 1.5 3.3 2.5 5.5 2.5v-2c-1.9 0-3.5-1-4.3-2.4l-1-1.6c-.4-.6-1-1-1.7-1-.3 0-.5.1-.8.1l-5.2 2.2v4.7h2v-3.4l1.8-.7-1.6 8.1-4.9-1-.4 2 7 1.4z" />
      </g>
    </svg>
  );
}
